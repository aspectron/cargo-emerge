use crate::result::Result;
use crate::error::Error;
use crate::tpl::Tpl;
use crate::context::Context;
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

#[derive(Debug, Deserialize, Serialize, Default)]
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
            .and_then(|m| m.emerge)
            .ok_or_else(|| Error::InvalidManifest(
                "Missing [package.metadata.emerge] section in Cargo.toml".to_string()
            ))?;

        // Setup template processor
        let mut tpl = Tpl::new();
        tpl.register("NAME", &cargo_toml.package.name);
        tpl.register("VERSION", &cargo_toml.package.version);
        tpl.register("PLATFORM", crate::utils::platform_string());

        // Process template variables
        let title = emerge_config.title
            .map(|t| tpl.parse(&t))
            .unwrap_or_else(|| cargo_toml.package.name.clone());

        let filename = emerge_config.filename
            .map(|f| tpl.parse(&f))
            .unwrap_or_else(|| format!("{}-{}-{}", 
                cargo_toml.package.name, 
                crate::utils::platform_string(),
                cargo_toml.package.version
            ));

        let description = cargo_toml.package.description
            .clone()
            .unwrap_or_default();

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

        let output_folder = emerge_config.output_folder
            .map(|f| ctx.base_dir.join(tpl.parse(&f)))
            .unwrap_or_else(|| ctx.base_dir.join("setup"));

        let icon = emerge_config.icon
            .map(|i| ctx.base_dir.join(tpl.parse(&i)));

        Ok(Manifest {
            name: cargo_toml.package.name,
            version: cargo_toml.package.version,
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

