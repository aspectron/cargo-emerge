use crate::context::Context;
use crate::manifest::Manifest;
use crate::result::Result;
use crate::utils;
use std::fs::{self, File};
use std::path::Path;
use tar::Builder;
use flate2::Compression;
use flate2::write::GzEncoder;

pub fn create_tar_gz(ctx: &Context, manifest: &Manifest) -> Result<()> {
    println!("Creating tar.gz archive for Linux...");

    // Ensure output folder exists
    utils::ensure_dir(&manifest.output_folder)?;

    // Create temporary directory for archive contents
    let temp_dir = std::env::temp_dir().join(format!("emerge-{}", manifest.name));
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir)?;
    }
    fs::create_dir_all(&temp_dir)?;

    // Create application directory
    let app_dir = temp_dir.join(&manifest.name);
    fs::create_dir_all(&app_dir)?;

    // Copy files according to copy operations
    for (src, dst) in &manifest.copy_operations {
        let dest_path = app_dir.join(dst);
        
        if ctx.verbose {
            println!("Copying {} to {}", src.display(), dest_path.display());
        }
        
        // Ensure parent directory exists
        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        utils::copy_recursively(src, &dest_path)?;
        
        // Set executable permissions for binary files (files without common document extensions)
        #[cfg(unix)]
        {
            let dst_extension = dst.extension().and_then(|e| e.to_str());
            let is_documentation = matches!(dst_extension, Some("md" | "txt" | "pdf" | "html" | "toml" | "json" | "yml" | "yaml"));
            
            if !is_documentation
                && let Ok(metadata) = fs::metadata(&dest_path)
                && metadata.is_file() {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = metadata.permissions();
                perms.set_mode(0o755);
                fs::set_permissions(&dest_path, perms)?;
            }
        }
    }

    // Create tar.gz archive
    let archive_filename = format!("{}.tar.gz", manifest.filename);
    let archive_path = manifest.output_folder.join(&archive_filename);

    create_tar_gz_file(&temp_dir, &archive_path)?;

    // Clean up temp directory
    fs::remove_dir_all(&temp_dir)?;

    println!("Archive created successfully: {}", archive_path.display());
    Ok(())
}

fn create_tar_gz_file(source_dir: &Path, output_path: &Path) -> Result<()> {
    let tar_gz = File::create(output_path)?;
    let enc = GzEncoder::new(tar_gz, Compression::default());
    let mut tar = Builder::new(enc);

    tar.append_dir_all(".", source_dir)?;
    tar.finish()?;

    Ok(())
}

