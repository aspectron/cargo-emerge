use crate::result::Result;
use std::path::{Path, PathBuf};
use std::fs;

/// Copy a file or directory recursively
pub fn copy_recursively(source: &Path, destination: &Path) -> Result<()> {
    if source.is_dir() {
        if !destination.exists() {
            fs::create_dir_all(destination)?;
        }

        for entry in fs::read_dir(source)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            let src_path = entry.path();
            let dst_path = destination.join(entry.file_name());

            if file_type.is_dir() {
                copy_recursively(&src_path, &dst_path)?;
            } else {
                fs::copy(&src_path, &dst_path)?;
            }
        }
    } else {
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(source, destination)?;
    }

    Ok(())
}

/// Find Cargo.toml in the current directory or specified path
pub fn find_manifest(path: Option<&Path>) -> Result<PathBuf> {
    let base_path = path.map(PathBuf::from).unwrap_or_else(|| std::env::current_dir().unwrap());

    let manifest_path = if base_path.is_file() && base_path.file_name().unwrap() == "Cargo.toml" {
        base_path
    } else {
        base_path.join("Cargo.toml")
    };

    if !manifest_path.exists() {
        return Err(crate::error::Error::ManifestNotFound(
            manifest_path.display().to_string(),
        ));
    }

    Ok(manifest_path)
}

/// Ensure a directory exists, creating it if necessary
pub fn ensure_dir(path: &Path) -> Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

/// Get the current platform identifier as a string
pub fn platform_string() -> &'static str {
    if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        "unknown"
    }
}

