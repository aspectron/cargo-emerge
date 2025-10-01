use std::path::PathBuf;

/// Context passed throughout the application containing global configuration
#[derive(Clone)]
pub struct Context {
    /// Enable verbose output (show command execution details)
    pub verbose: bool,

    /// Path to the Cargo.toml manifest
    pub manifest_path: PathBuf,

    /// Base directory (directory containing Cargo.toml)
    pub base_dir: PathBuf,
}

impl Context {
    pub fn new(manifest_path: PathBuf, verbose: bool) -> Self {
        let base_dir = manifest_path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));

        Self {
            verbose,
            manifest_path,
            base_dir,
        }
    }
}
