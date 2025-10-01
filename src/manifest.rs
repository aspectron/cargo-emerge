use crate::context::Context;
use crate::error::Error;
use crate::result::Result;
use crate::tpl::Tpl;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize)]
pub struct CargoToml {
    pub package: Package,
    #[serde(default)]
    pub dependencies: HashMap<String, toml::Value>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Package {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub metadata: Option<Metadata>,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct Metadata {
    #[serde(default)]
    pub emerge: Option<EmergeConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EmergeConfig {
    #[serde(default)]
    pub title: Option<String>,

    #[serde(default)]
    pub filename: Option<String>,

    #[serde(default)]
    pub build: Vec<String>,

    #[serde(default)]
    pub copy: Vec<HashMap<String, String>>,

    #[serde(rename = "output-folder", default)]
    pub output_folder: Option<String>,

    // Icon configuration
    #[serde(default)]
    pub icon: Option<String>,

    // DMG-specific configuration
    #[serde(default)]
    pub dmg: Option<DmgConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DmgConfig {
    #[serde(default)]
    pub background: Option<String>,

    #[serde(default)]
    pub window_position: Option<(i32, i32)>,

    #[serde(default)]
    pub window_size: Option<(i32, i32)>,

    #[serde(default)]
    pub app_position: Option<(i32, i32)>,

    #[serde(default)]
    pub applications_position: Option<(i32, i32)>,

    #[serde(default)]
    pub additional_files: Vec<DmgFile>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DmgFile {
    pub source: String,
    pub position: (i32, i32),
}

/// Parsed and processed manifest information
pub struct Manifest {
    pub name: String,
    pub version: String,
    #[allow(dead_code)]
    pub description: String,
    pub title: String,
    pub filename: String,
    pub build_commands: Vec<String>,
    pub copy_operations: Vec<(PathBuf, PathBuf)>,
    pub output_folder: PathBuf,
    pub icon: Option<PathBuf>,
    pub dmg: Option<DmgConfig>,
}

impl Manifest {
    /// Load and parse the manifest from Cargo.toml
    pub fn load(ctx: &Context) -> Result<Self> {
        let content = fs::read_to_string(&ctx.manifest_path)?;
        let cargo_toml: CargoToml = toml::from_str(&content)?;

        let emerge_config = cargo_toml
            .package
            .metadata
            .clone()
            .and_then(|m| m.emerge)
            .ok_or_else(|| {
                Error::InvalidManifest(
                    "Missing [package.metadata.emerge] section in Cargo.toml".to_string(),
                )
            })?;

        Self::process_manifest(ctx, &cargo_toml.package, emerge_config)
    }

    /// Load manifest with package info from Cargo.toml and emerge config from alternative file
    pub fn load_with_emerge_manifest(
        ctx: &Context,
        emerge_manifest_path: &PathBuf,
    ) -> Result<Self> {
        // Read the alternative manifest file for emerge configuration
        let emerge_path = if emerge_manifest_path.is_absolute() {
            emerge_manifest_path.clone()
        } else {
            std::env::current_dir()?.join(emerge_manifest_path)
        };

        if !emerge_path.exists() {
            return Err(Error::ManifestNotFound(format!(
                "Emerge manifest not found at: {}",
                emerge_path.display()
            )));
        }

        let emerge_content = fs::read_to_string(&emerge_path)?;

        // Try parsing as a full Cargo.toml format first
        if let Ok(full_toml) = toml::from_str::<CargoToml>(&emerge_content) {
            // Check if it has a [package] section
            // If it does, use it instead of loading Cargo.toml from workspace
            let emerge_config = full_toml
                .package
                .metadata
                .clone()
                .and_then(|m| m.emerge)
                .ok_or_else(|| {
                    Error::InvalidManifest(format!(
                        "Missing [package.metadata.emerge] section in {}",
                        emerge_path.display()
                    ))
                })?;

            // Use the package info from the emerge manifest itself
            return Self::process_manifest(ctx, &full_toml.package, emerge_config);
        }

        // If not a full Cargo.toml, try parsing as just the emerge section (standalone format)
        let emerge_config = toml::from_str::<EmergeConfig>(&emerge_content).map_err(|e| {
            Error::InvalidManifest(format!(
                "Failed to parse emerge manifest at {}: {}",
                emerge_path.display(),
                e
            ))
        })?;

        // For standalone format, we still need Cargo.toml for package info
        let cargo_content = fs::read_to_string(&ctx.manifest_path)?;
        let cargo_toml: CargoToml = toml::from_str(&cargo_content)?;

        Self::process_manifest(ctx, &cargo_toml.package, emerge_config)
    }

    /// Process the manifest data and create the Manifest struct
    fn process_manifest(
        ctx: &Context,
        package: &Package,
        emerge_config: EmergeConfig,
    ) -> Result<Self> {
        // Setup template processor
        let mut tpl = Tpl::new();
        tpl.register("NAME", &package.name);
        tpl.register("VERSION", &package.version);
        tpl.register("PLATFORM", crate::utils::platform_string());

        // Process template variables
        let title = emerge_config
            .title
            .map(|t| tpl.parse(&t))
            .unwrap_or_else(|| package.name.clone());

        let filename = emerge_config
            .filename
            .map(|f| tpl.parse(&f))
            .unwrap_or_else(|| {
                format!(
                    "{}-{}-{}",
                    package.name,
                    crate::utils::platform_string(),
                    package.version
                )
            });

        let description = package.description.clone().unwrap_or_default();

        let build_commands = tpl.parse_vec(&emerge_config.build);

        // Process copy operations
        let mut copy_operations = Vec::new();
        for copy_map in &emerge_config.copy {
            for (src, dst) in copy_map {
                let src_path = ctx.base_dir.join(tpl.parse(src));
                let dst_path = PathBuf::from(tpl.parse(dst));
                copy_operations.push((src_path, dst_path));
            }
        }

        let output_folder = emerge_config
            .output_folder
            .map(|f| ctx.base_dir.join(tpl.parse(&f)))
            .unwrap_or_else(|| ctx.base_dir.join("setup"));

        let icon = emerge_config.icon.map(|i| ctx.base_dir.join(tpl.parse(&i)));

        Ok(Manifest {
            name: package.name.clone(),
            version: package.version.clone(),
            description,
            title,
            filename,
            build_commands,
            copy_operations,
            output_folder,
            icon,
            dmg: emerge_config.dmg,
        })
    }
}
