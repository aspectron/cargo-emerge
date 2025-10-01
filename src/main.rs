mod args;
mod cmd;
mod context;
mod error;
mod manifest;
mod platform;
mod result;
mod tpl;
mod utils;

#[cfg(target_os = "macos")]
mod macos;

// Linux module is always included for tar.gz support on all platforms
mod linux;

#[cfg(target_os = "windows")]
mod windows;

use args::Args;
use context::Context;
use manifest::Manifest;
use platform::Platform;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> result::Result<()> {
    // Parse command-line arguments
    let Args {
        verbose,
        archive: archive_flag,
        dmg: dmg_flag,
        no_build,
        path,
        manifest: emerge_manifest,
    } = Args::parse();

    // Find Cargo.toml
    let manifest_path = utils::find_manifest(path.as_deref())?;

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
