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
    #[serde(default)]
    pub package: Option<Package>,
    #[serde(default)]
    pub workspace: Option<Workspace>,
    #[serde(default)]
    pub dependencies: HashMap<String, toml::Value>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Workspace {
    #[serde(default)]
    pub package: Option<WorkspacePackage>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WorkspacePackage {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub metadata: Option<Metadata>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Package {
    #[serde(deserialize_with = "deserialize_string_or_workspace")]
    pub name: String,
    #[serde(deserialize_with = "deserialize_string_or_workspace")]
    pub version: String,
    #[serde(default, deserialize_with = "deserialize_option_string_or_workspace")]
    pub description: Option<String>,
    #[serde(default)]
    pub metadata: Option<Metadata>,
}

// Helper to deserialize either a string or a workspace reference table
fn deserialize_string_or_workspace<'de, D>(deserializer: D) -> std::result::Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrWorkspace {
        String(String),
        #[allow(dead_code)]
        Workspace(HashMap<String, toml::Value>),
    }

    match StringOrWorkspace::deserialize(deserializer)? {
        StringOrWorkspace::String(s) => Ok(s),
        StringOrWorkspace::Workspace(_) => {
            // Return a placeholder when workspace = true is used
            // This will be ignored anyway since we're loading from external manifest
            Ok(String::new())
        }
    }
}

fn deserialize_option_string_or_workspace<'de, D>(
    deserializer: D,
) -> std::result::Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrWorkspace {
        String(String),
        #[allow(dead_code)]
        Workspace(HashMap<String, toml::Value>),
    }

    match Option::<StringOrWorkspace>::deserialize(deserializer)? {
        None => Ok(None),
        Some(StringOrWorkspace::String(s)) => Ok(Some(s)),
        Some(StringOrWorkspace::Workspace(_)) => Ok(None),
    }
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

    // Path to external manifest file
    #[serde(default)]
    pub manifest: Option<String>,
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

        // Try to parse the Cargo.toml
        let cargo_toml: CargoToml = toml::from_str(&content).map_err(|e| {
            Error::InvalidManifest(format!(
                "Failed to parse Cargo.toml at {}: {}",
                ctx.manifest_path.display(),
                e
            ))
        })?;

        // First check if there's a workspace.package.metadata.emerge section with a manifest property
        if let Some(manifest_path) = cargo_toml
            .workspace
            .as_ref()
            .and_then(|w| w.package.as_ref())
            .and_then(|p| p.metadata.as_ref())
            .and_then(|m| m.emerge.as_ref())
            .and_then(|e| e.manifest.as_ref())
        {
            // Found a manifest property in workspace.package.metadata.emerge
            // Load the external manifest file
            let manifest_file = PathBuf::from(manifest_path);
            return Self::load_with_emerge_manifest(ctx, &manifest_file);
        }

        // Otherwise, try to load from package.metadata.emerge
        if let Some(package) = &cargo_toml.package {
            let emerge_config =
                package
                    .metadata
                    .clone()
                    .and_then(|m| m.emerge)
                    .ok_or_else(|| {
                        Error::InvalidManifest(
                            "Missing [package.metadata.emerge] section in Cargo.toml".to_string(),
                        )
                    })?;

            // Check if package.metadata.emerge has a manifest property
            if let Some(manifest_path) = &emerge_config.manifest {
                let manifest_file = PathBuf::from(manifest_path);
                return Self::load_with_emerge_manifest(ctx, &manifest_file);
            }

            return Self::process_manifest(ctx, package, emerge_config);
        }

        Err(Error::InvalidManifest(
            "No [package] or [workspace.package] section found in Cargo.toml".to_string(),
        ))
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

        // Try parsing as a full Cargo.toml format first (with [package] section)
        // We use a custom deserializer that ignores workspace-style values
        if let Ok(full_toml) = toml::from_str::<CargoToml>(&emerge_content) {
            // Check if it has a [package] section with actual values (not workspace references)
            if let Some(package) = &full_toml.package {
                if let Some(emerge) = package.metadata.as_ref().and_then(|m| m.emerge.as_ref()) {
                    // Use the package info from the emerge manifest itself
                    return Self::process_manifest(ctx, package, emerge.clone());
                }

                return Err(Error::InvalidManifest(format!(
                    "Missing [package.metadata.emerge] section in {}",
                    emerge_path.display()
                )));
            }
        }

        // If not a full Cargo.toml, try parsing as just the emerge section (standalone format)
        if toml::from_str::<EmergeConfig>(&emerge_content).is_ok() {
            // For standalone format, we need Cargo.toml for package info
            // But workspace Cargo.toml files don't have a [package] section, so this won't work
            return Err(Error::InvalidManifest(format!(
                "Manifest file {} must contain a [package] section with name and version, \
                 or use the full Cargo.toml format with [package.metadata.emerge] section",
                emerge_path.display()
            )));
        }

        Err(Error::InvalidManifest(format!(
            "Failed to parse emerge manifest at {}. \
             It must be either a valid Cargo.toml with [package.metadata.emerge], \
             or a standalone emerge configuration.",
            emerge_path.display()
        )))
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
