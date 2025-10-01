use clap::{Arg, ArgAction, Command};
use std::path::PathBuf;

/// Command-line arguments for the emerge tool
#[derive(Debug)]
pub struct Args {
    /// Enable verbose output
    pub verbose: bool,

    /// Create an archived setup (.tar.gz or .zip)
    pub archive: bool,

    /// Create DMG image (default on macOS)
    pub dmg: bool,

    /// Skip build commands (use existing binaries)
    pub no_build: bool,

    /// Path to Cargo.toml or directory containing it
    pub path: Option<PathBuf>,

    /// Path to alternative manifest file for emerge configuration
    pub manifest: Option<PathBuf>,
}

impl Args {
    /// Parse command-line arguments
    pub fn parse() -> Self {
        let matches = Command::new("emerge")
            .version(env!("CARGO_PKG_VERSION"))
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

        Self {
            verbose: matches.get_flag("verbose"),
            archive: matches.get_flag("archive"),
            dmg: matches.get_flag("dmg"),
            no_build: matches.get_flag("no-build"),
            path: matches.get_one::<String>("path").map(PathBuf::from),
            manifest: matches.get_one::<String>("manifest").map(PathBuf::from),
        }
    }
}
