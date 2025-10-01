mod error;
mod result;
mod context;
mod cmd;
mod tpl;
mod utils;
mod platform;
mod manifest;

#[cfg(target_os = "macos")]
mod macos;

// Linux module is always included for tar.gz support on all platforms
mod linux;

#[cfg(target_os = "windows")]
mod windows;

use clap::{Arg, ArgAction, Command};
use context::Context;
use manifest::Manifest;
use platform::Platform;
use std::path::PathBuf;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> result::Result<()> {
    let matches = Command::new("emerge")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Aspect")
        .about("Setup generation tool for desktop Rust applications")
        .arg(
            Arg::new("path")
                .short('p')
                .long("path")
                .value_name("PATH")
                .help("Path to Cargo.toml or directory containing it")
        )
        .arg(
            Arg::new("manifest")
                .short('m')
                .long("manifest")
                .value_name("FILE")
                .help("Path to alternative manifest file (e.g., EXAMPLE.toml) for emerge configuration")
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(ArgAction::SetTrue)
                .help("Enable verbose output")
        )
        .arg(
            Arg::new("archive")
                .short('a')
                .long("archive")
                .action(ArgAction::SetTrue)
                .help("Create an archived setup (.tar.gz or .zip)")
        )
        .arg(
            Arg::new("dmg")
                .long("dmg")
                .action(ArgAction::SetTrue)
                .help("Create DMG image (default on macOS)")
        )
        .arg(
            Arg::new("no-build")
                .long("no-build")
                .action(ArgAction::SetTrue)
                .help("Skip build commands (use existing binaries)")
        )
        .get_matches();

    let verbose = matches.get_flag("verbose");
    let archive_flag = matches.get_flag("archive");
    let dmg_flag = matches.get_flag("dmg");
    let no_build = matches.get_flag("no-build");

    // Find Cargo.toml
    let path = matches.get_one::<String>("path").map(PathBuf::from);
    let manifest_path = utils::find_manifest(path.as_deref())?;

    // Get optional alternative manifest file for emerge configuration
    let emerge_manifest = matches.get_one::<String>("manifest").map(PathBuf::from);

    // Create context
    let ctx = Context::new(manifest_path, verbose);

    // Use cliclack for nice UI
    cliclack::intro("emerge")?;

    // Load manifest
    let manifest = {
        let spinner = cliclack::spinner();
        spinner.start("Loading manifest...");
        let result = if let Some(emerge_path) = emerge_manifest {
            Manifest::load_with_emerge_manifest(&ctx, &emerge_path)
        } else {
            Manifest::load(&ctx)
        };
        match result {
            Ok(m) => {
                spinner.stop(format!("Loaded manifest for {}", m.title));
                m
            }
            Err(e) => {
                spinner.error("Failed to load manifest");
                return Err(e);
            }
        }
    };

    // Execute build commands unless --no-build is specified
    if !no_build && !manifest.build_commands.is_empty() {
        let spinner = cliclack::spinner();
        spinner.start("Building application...");
        
        for command in &manifest.build_commands {
            if verbose {
                spinner.stop(format!("Running: {}", command));
            }
            
            let parts: Vec<&str> = command.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            let program = parts[0];
            let args = &parts[1..];

            cmd::execute(&ctx, program, args)?;
        }
        
        spinner.stop("Build completed");
    }

    // Determine what to build
    let current_platform = Platform::current();
    
    if archive_flag {
        // Create archive based on platform
        create_archive(&ctx, &manifest, current_platform)?;
    } else if dmg_flag || current_platform == Platform::MacOS {
        // Create DMG (default on macOS)
        if current_platform != Platform::MacOS {
            cliclack::outro_cancel("DMG creation is only available on macOS")?;
            return Ok(());
        }
        
        let spinner = cliclack::spinner();
        spinner.start("Creating DMG...");
        
        #[cfg(target_os = "macos")]
        macos::dmg::create(&ctx, &manifest)?;
        
        spinner.stop("DMG created successfully");
    } else {
        // Default behavior based on platform
        match current_platform {
            Platform::MacOS => {
                let spinner = cliclack::spinner();
                spinner.start("Creating DMG...");
                
                #[cfg(target_os = "macos")]
                macos::dmg::create(&ctx, &manifest)?;
                
                spinner.stop("DMG created successfully");
            }
            Platform::Linux => {
                create_archive(&ctx, &manifest, current_platform)?;
            }
            Platform::Windows => {
                create_archive(&ctx, &manifest, current_platform)?;
            }
        }
    }

    cliclack::outro("Setup package created successfully!")?;
    Ok(())
}

fn create_archive(ctx: &Context, manifest: &Manifest, platform: Platform) -> result::Result<()> {
    let spinner = cliclack::spinner();
    
    match platform {
        Platform::Linux | Platform::MacOS => {
            spinner.start("Creating tar.gz archive...");
            linux::archive::create_tar_gz(ctx, manifest)?;
            spinner.stop("Archive created successfully");
        }
        Platform::Windows => {
            spinner.start("Creating zip archive...");
            #[cfg(target_os = "windows")]
            windows::archive::create_zip(ctx, manifest)?;
            spinner.stop("Archive created successfully");
        }
    }
    
    Ok(())
}
