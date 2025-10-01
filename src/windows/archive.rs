use crate::context::Context;
use crate::error::Error;
use crate::manifest::Manifest;
use crate::result::Result;
use crate::utils;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use walkdir::WalkDir;
use zip::ZipWriter;
use zip::write::FileOptions;

pub fn create_zip(ctx: &Context, manifest: &Manifest) -> Result<()> {
    println!("Creating zip archive for Windows...");

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
    }

    // Create zip archive
    let archive_filename = format!("{}.zip", manifest.filename);
    let archive_path = manifest.output_folder.join(&archive_filename);

    create_zip_file(&temp_dir, &archive_path)?;

    // Clean up temp directory
    fs::remove_dir_all(&temp_dir)?;

    println!("Archive created successfully: {}", archive_path.display());
    Ok(())
}

fn create_zip_file(source_dir: &Path, output_path: &Path) -> Result<()> {
    let file = File::create(output_path)?;
    let mut zip = ZipWriter::new(file);

    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o755);

    let walkdir = WalkDir::new(source_dir);
    let it = walkdir.into_iter().filter_map(|e| e.ok());

    for entry in it {
        let path = entry.path();
        let name = path.strip_prefix(source_dir).unwrap();

        if path.is_file() {
            zip.start_file(name.to_string_lossy().to_string(), options)?;
            let mut f = File::open(path)?;
            let mut buffer = Vec::new();
            f.read_to_end(&mut buffer)?;
            zip.write_all(&buffer)?;
        } else if !name.as_os_str().is_empty() {
            zip.add_directory(name.to_string_lossy().to_string(), options)?;
        }
    }

    zip.finish()?;
    Ok(())
}
