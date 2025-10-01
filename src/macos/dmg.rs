use crate::cmd;
use crate::context::Context;
use crate::error::Error;
use crate::manifest::Manifest;
use crate::result::Result;
use crate::utils;
use icns::{IconFamily, IconType};
use image::ImageReader;
use std::fs;
use std::path::{Path, PathBuf};

pub fn create(ctx: &Context, manifest: &Manifest) -> Result<()> {
    println!("Creating DMG for macOS...");

    // Ensure output folder exists
    utils::ensure_dir(&manifest.output_folder)?;

    // Create temporary directory for DMG contents
    let temp_dir = std::env::temp_dir().join(format!("emerge-{}", manifest.name));
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir)?;
    }
    fs::create_dir_all(&temp_dir)?;

    // Pre-process icon to ICNS if needed (before any file operations)
    let processed_icon_path = if let Some(icon_path) = &manifest.icon {
        if icon_path.exists() {
            let icon_temp_dir = std::env::temp_dir().join(format!("emerge-icon-{}", manifest.name));
            if icon_temp_dir.exists() {
                fs::remove_dir_all(&icon_temp_dir)?;
            }
            fs::create_dir_all(&icon_temp_dir)?;

            let processed_icon = icon_temp_dir.join("icon.icns");

            if icon_path.extension().and_then(|e| e.to_str()) == Some("icns") {
                fs::copy(icon_path, &processed_icon)?;
            } else {
                if ctx.verbose {
                    println!("Converting icon to ICNS format...");
                }
                generate_icns_from_image(icon_path, &processed_icon)?;
            }

            Some(processed_icon)
        } else {
            None
        }
    } else {
        None
    };

    // Create the .app bundle structure
    let app_name = format!("{}.app", manifest.title);
    let app_path = temp_dir.join(&app_name);
    let (macos_dir, resources_dir) = create_app_bundle_structure(ctx, manifest, &app_path)?;

    // Copy the processed ICNS icon first (if available)
    if let Some(ref processed_icon) = processed_icon_path {
        let icon_dest = resources_dir.join("icon.icns");
        if ctx.verbose {
            println!("Copying processed icon to {}", icon_dest.display());
        }
        fs::copy(processed_icon, icon_dest)?;
    }

    // Copy files according to copy operations
    // For macOS DMG, files are copied into the .app bundle's MacOS folder
    // unless they have specific extensions (like .md, .txt, etc.) which go to DMG root
    for (src, dst) in &manifest.copy_operations {
        let dst_extension = dst.extension().and_then(|e| e.to_str());

        // Determine if file should go to DMG root or app bundle
        let is_documentation =
            matches!(dst_extension, Some("md" | "txt" | "pdf" | "html" | "toml"));

        let dest_path = if is_documentation {
            // Documentation files go to DMG root alongside the .app
            temp_dir.join(dst)
        } else {
            // Executable and other files go into the app bundle's MacOS folder
            macos_dir.join(dst)
        };

        if ctx.verbose {
            println!("Copying {} to {}", src.display(), dest_path.display());
        }

        // Ensure parent directory exists
        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent)?;
        }

        utils::copy_recursively(src, &dest_path)?;

        // Set executable permissions for files in MacOS folder
        #[cfg(unix)]
        if dest_path.starts_with(&macos_dir)
            && let Ok(metadata) = fs::metadata(&dest_path)
            && metadata.is_file()
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = metadata.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&dest_path, perms)?;
        }
    }

    // Create symbolic link to /Applications
    let applications_link = temp_dir.join("Applications");
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        symlink("/Applications", &applications_link)?;
    }

    // Create DMG
    let dmg_filename = format!("{}.dmg", manifest.filename);
    let dmg_path = manifest.output_folder.join(&dmg_filename);

    if dmg_path.exists() {
        fs::remove_file(&dmg_path)?;
    }

    create_dmg_image(
        ctx,
        manifest,
        &temp_dir,
        &dmg_path,
        processed_icon_path.as_ref(),
    )?;

    // Clean up temp directory
    fs::remove_dir_all(&temp_dir)?;

    // Clean up icon temp directory if it was created
    if let Some(ref icon_dir) = processed_icon_path.as_ref().and_then(|p| p.parent()) {
        let _ = fs::remove_dir_all(icon_dir);
    }

    println!("DMG created successfully: {}", dmg_path.display());
    Ok(())
}

fn create_app_bundle_structure(
    ctx: &Context,
    manifest: &Manifest,
    app_path: &Path,
) -> Result<(std::path::PathBuf, std::path::PathBuf)> {
    // Create .app structure
    let contents_dir = app_path.join("Contents");
    let macos_dir = contents_dir.join("MacOS");
    let resources_dir = contents_dir.join("Resources");

    fs::create_dir_all(&macos_dir)?;
    fs::create_dir_all(&resources_dir)?;

    if ctx.verbose {
        println!("Created app bundle structure at {}", app_path.display());
    }

    // Create Info.plist
    create_info_plist(manifest, &contents_dir)?;

    // Note: Icon is now pre-processed and copied separately before other file operations

    Ok((macos_dir, resources_dir))
}

fn create_info_plist(manifest: &Manifest, contents_dir: &Path) -> Result<()> {
    let plist_content = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>CFBundleExecutable</key>
    <string>{}</string>
    <key>CFBundleIdentifier</key>
    <string>com.{}.{}</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>{}</string>
    <key>CFBundleDisplayName</key>
    <string>{}</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>{}</string>
    <key>CFBundleVersion</key>
    <string>{}</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.13</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>CFBundleIconFile</key>
    <string>icon.icns</string>
</dict>
</plist>
"#,
        manifest.name,
        manifest.name,
        manifest.name,
        manifest.title,
        manifest.title,
        manifest.version,
        manifest.version,
    );

    let plist_path = contents_dir.join("Info.plist");
    fs::write(plist_path, plist_content)?;

    Ok(())
}

/// Generate an ICNS file from a source image (PNG, JPEG, etc.)
/// Supports multiple icon sizes as required by macOS
fn generate_icns_from_image(source_path: &Path, output_path: &Path) -> Result<()> {
    // Load the source image
    let img = ImageReader::open(source_path)?
        .with_guessed_format()?
        .decode()?;

    // Create a new IconFamily
    let mut icon_family = IconFamily::new();

    // Define the icon sizes we want to generate
    // macOS uses multiple sizes for different contexts
    let icon_types = vec![
        (IconType::RGBA32_16x16, 16),
        (IconType::RGBA32_16x16_2x, 32),
        (IconType::RGBA32_32x32, 32),
        (IconType::RGBA32_32x32_2x, 64),
        (IconType::RGBA32_128x128, 128),
        (IconType::RGBA32_128x128_2x, 256),
        (IconType::RGBA32_256x256, 256),
        (IconType::RGBA32_256x256_2x, 512),
        (IconType::RGBA32_512x512, 512),
        (IconType::RGBA32_512x512_2x, 1024),
    ];

    for (icon_type, size) in icon_types {
        // Resize the image
        let resized = img.resize_exact(size, size, image::imageops::FilterType::Lanczos3);

        // Convert to RGBA8
        let rgba = resized.to_rgba8();
        let raw_data = rgba.into_raw();

        // Create ICNS image using the encode method
        let icns_image = icns::Image::from_data(icns::PixelFormat::RGBA, size, size, raw_data)?;

        // Encode and add to icon family
        icon_family.add_icon_with_type(&icns_image, icon_type)?;
    }

    // Write the ICNS file
    let output_file = fs::File::create(output_path)?;
    icon_family.write(output_file)?;

    Ok(())
}

fn create_dmg_image(
    ctx: &Context,
    manifest: &Manifest,
    source_dir: &Path,
    output_path: &Path,
    processed_icon: Option<&PathBuf>,
) -> Result<()> {
    // Create initial DMG using hdiutil
    let temp_dmg = output_path.with_extension("temp.dmg");

    if ctx.verbose {
        println!("Creating temporary DMG...");
    }

    cmd::execute(
        ctx,
        "hdiutil",
        &[
            "create",
            "-srcfolder",
            source_dir.to_str().unwrap(),
            "-volname",
            &manifest.title,
            "-fs",
            "HFS+",
            "-fsargs",
            "-c c=64,a=16,e=16",
            "-format",
            "UDRW",
            temp_dmg.to_str().unwrap(),
        ],
    )?;

    // Mount the DMG
    if ctx.verbose {
        println!("Mounting DMG for customization...");
    }

    let mount_output = cmd::execute_with_output(
        ctx,
        "hdiutil",
        &[
            "attach",
            "-readwrite",
            "-noverify",
            "-noautoopen",
            temp_dmg.to_str().unwrap(),
        ],
    )?;

    // Extract mount point from hdiutil output
    // Output format: /dev/diskN    GUID_partition_scheme
    //                /dev/diskNs1  Apple_HFS                /Volumes/VolumeName (may have spaces)
    let mount_point = mount_output
        .lines()
        .find(|line| line.contains("/Volumes/"))
        .and_then(|line| {
            // Find the /Volumes/ part and take everything from there to the end
            line.find("/Volumes/").map(|pos| line[pos..].trim())
        })
        .ok_or_else(|| {
            Error::Custom("Failed to determine mount point from hdiutil output".to_string())
        })?;

    if ctx.verbose {
        println!("Mounted at: {}", mount_point);
    }

    // Customize DMG appearance
    customize_dmg_appearance(ctx, manifest, mount_point)?;

    // Note: Icon is already copied to the .app bundle's Resources folder
    // Volume icon for DMG itself is optional and not set here to avoid space issues
    let _ = processed_icon; // Suppress unused variable warning

    // Sync to ensure all data is flushed to disk before unmounting
    // This is critical to prevent corruption and ensure the DMG is properly unmountable
    // Reference: cargo-nw dmg.rs implementation
    if ctx.verbose {
        println!("Syncing filesystem...");
    }
    cmd::execute(ctx, "sync", &[])?;

    // Give the filesystem a moment to complete sync operations
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Detach the DMG
    if ctx.verbose {
        println!("Detaching DMG...");
    }

    cmd::execute(ctx, "hdiutil", &["detach", mount_point])?;

    // Convert to compressed read-only DMG
    if ctx.verbose {
        println!("Compressing DMG...");
    }

    cmd::execute(
        ctx,
        "hdiutil",
        &[
            "convert",
            temp_dmg.to_str().unwrap(),
            "-format",
            "UDZO",
            "-imagekey",
            "zlib-level=9",
            "-o",
            output_path.to_str().unwrap(),
        ],
    )?;

    // Remove temporary DMG
    fs::remove_file(temp_dmg)?;

    Ok(())
}

fn customize_dmg_appearance(ctx: &Context, manifest: &Manifest, mount_point: &str) -> Result<()> {
    let mount_path = Path::new(mount_point);

    // Get DMG configuration or use defaults
    let window_pos = manifest
        .dmg
        .as_ref()
        .and_then(|d| d.window_position)
        .unwrap_or((100, 100));

    let window_size = manifest
        .dmg
        .as_ref()
        .and_then(|d| d.window_size)
        .unwrap_or((600, 400));

    let app_pos = manifest
        .dmg
        .as_ref()
        .and_then(|d| d.app_position)
        .unwrap_or((150, 200));

    let apps_pos = manifest
        .dmg
        .as_ref()
        .and_then(|d| d.applications_position)
        .unwrap_or((450, 200));

    // Copy background image if provided
    if let Some(dmg_config) = &manifest.dmg
        && let Some(bg_path) = &dmg_config.background
    {
        let bg_src = ctx.base_dir.join(bg_path);
        if bg_src.exists() {
            let background_dir = mount_path.join(".background");
            fs::create_dir_all(&background_dir)?;
            let bg_dst = background_dir.join("background.png");
            fs::copy(&bg_src, &bg_dst)?;
        }
    }

    // Create AppleScript to set window properties
    let app_name = format!("{}.app", manifest.title);
    let applescript = format!(
        r#"
        tell application "Finder"
            tell disk "{}"
                open
                set current view of container window to icon view
                set toolbar visible of container window to false
                set statusbar visible of container window to false
                set the bounds of container window to {{{}, {}, {}, {}}}
                set viewOptions to the icon view options of container window
                set arrangement of viewOptions to not arranged
                set icon size of viewOptions to 72
                {}
                set position of item "{}" to {{{}, {}}}
                set position of item "Applications" to {{{}, {}}}
                close
                open
                update without registering applications
                delay 2
            end tell
        end tell
    "#,
        manifest.title,
        window_pos.0,
        window_pos.1,
        window_pos.0 + window_size.0,
        window_pos.1 + window_size.1,
        if manifest
            .dmg
            .as_ref()
            .and_then(|d| d.background.as_ref())
            .is_some()
        {
            "set background picture of viewOptions to file \".background:background.png\""
        } else {
            ""
        },
        app_name,
        app_pos.0,
        app_pos.1,
        apps_pos.0,
        apps_pos.1,
    );

    // Execute AppleScript
    let script_path = mount_path.join(".setup_script.applescript");
    fs::write(&script_path, applescript)?;

    cmd::execute(ctx, "osascript", &[script_path.to_str().unwrap()])?;

    // Clean up script
    fs::remove_file(script_path)?;

    Ok(())
}
