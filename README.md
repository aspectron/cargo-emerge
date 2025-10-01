# emerge

A setup generation tool for desktop Rust applications that creates professional, platform-specific setup packages including DMG images for macOS, zip archives for Windows, and tar.gz archives for Linux.

## Features

- **macOS DMG Creation**: Generate beautiful, customizable DMG disk images with custom backgrounds, window positioning, and icon placement
- **Automatic Icon Conversion**: Convert PNG, JPEG, or other image formats to .icns with proper retina support
- **Windows ZIP Archives**: Create zip archives with your application and resources
- **Linux TAR.GZ Archives**: Generate compressed tar archives for Linux distribution
- **Template Variables**: Support for dynamic file naming with `$VARIABLE` syntax
- **Build Integration**: Execute build commands before packaging
- **File Copying**: Flexible file and directory copying for resources
- **Modern CLI**: Beautiful command-line interface powered by cliclack

## Installation

```bash
cargo install --path .
```

## Usage

### Basic Usage

Navigate to your project directory and run:

```bash
emerge
```

This will:
1. Load the configuration from `Cargo.toml`
2. Execute build commands
3. Create a platform-appropriate setup package (DMG on macOS, tar.gz on Linux, zip on Windows)

### Command Line Options

```bash
emerge [OPTIONS]

Options:
  -p, --path <PATH>       Path to Cargo.toml or directory containing it
  -m, --manifest <FILE>   Path to alternative manifest file for emerge configuration
  -v, --verbose           Enable verbose output
  -a, --archive           Create an archived setup (.tar.gz or .zip)
      --dmg               Create DMG image (default on macOS)
      --no-build          Skip build commands (use existing binaries)
  -h, --help              Print help
  -V, --version           Print version
```

### Examples

```bash
# Create DMG on macOS with verbose output
emerge --verbose

# Create archive instead of DMG
emerge --archive

# Use a specific Cargo.toml
emerge --path /path/to/project

# Use an alternative manifest file for emerge configuration
emerge --manifest my-setup-config.toml

# Skip build and use existing binaries
emerge --no-build
```

### Alternative Manifest Files

You can use the `--manifest` flag to specify an alternative TOML file containing emerge configuration. This is useful for:
- Managing multiple build configurations
- Keeping setup configuration separate from Cargo.toml
- Testing different setups without modifying your project

The manifest file can be in two formats:

**Format 1: Standalone emerge configuration**
```toml
title = "My App"
filename = "myapp-$PLATFORM-$VERSION"
output-folder = "dist"
build = ["cargo build --release"]
copy = []
icon = "icon.png"
```

**Format 2: Full Cargo.toml format**
```toml
[package.metadata.emerge]
title = "My App"
filename = "myapp-$PLATFORM-$VERSION"
output-folder = "dist"
build = ["cargo build --release"]
copy = []
icon = "icon.png"
```

When using `--manifest`, emerge will:
1. Read package information (name, version) from your Cargo.toml
2. Read emerge configuration from the specified manifest file
3. Merge them together for the build process

## Configuration

Add a `[package.metadata.emerge]` section to your `Cargo.toml`:

```toml
[package.metadata.emerge]
title = "My Application"
filename = "my-application-$PLATFORM-$VERSION"
output-folder = "setup"

# Build commands to execute before packaging
build = [
    "cargo build --release"
]

# Files to copy (source = destination)
copy = [
    { "resources" = "resources" },
    { "README.md" = "README.md" }
]

# Optional: Path to application icon
# Supports .icns, .png, .jpg, and other formats
# Will automatically convert to .icns with proper retina sizes
icon = "assets/icon.png"

# DMG-specific configuration (macOS only)
[package.metadata.emerge.dmg]
background = "assets/dmg-background.png"
window_position = [100, 100]
window_size = [600, 400]
app_position = [150, 200]
applications_position = [450, 200]
```

### Template Variables

The following variables are available for use in the configuration:

- `$NAME` - Package name from Cargo.toml
- `$VERSION` - Package version from Cargo.toml
- `$PLATFORM` - Current platform (macos, linux, or windows)

Example:
```toml
filename = "$NAME-$PLATFORM-$VERSION"
# Results in: myapp-macos-1.0.0.dmg
```

### DMG Configuration

For macOS DMG images, you can customize:

- **background**: Path to a PNG image for the DMG window background
- **window_position**: [x, y] position of the DMG window when opened
- **window_size**: [width, height] size of the DMG window
- **app_position**: [x, y] position of your application icon in the DMG
- **applications_position**: [x, y] position of the Applications folder link

## Architecture

The tool is organized into the following modules:

- **context**: Global configuration passed throughout the application
- **cmd**: Command execution with output streaming
- **tpl**: Template variable processing
- **utils**: General utility functions
- **platform**: Platform detection and routing
- **manifest**: Cargo.toml parsing and configuration
- **macos/dmg**: DMG creation for macOS
- **linux/archive**: tar.gz creation for Linux
- **windows/archive**: zip creation for Windows

## Requirements

### macOS
- `hdiutil` (included with macOS)
- `osascript` (included with macOS)

### Linux
- Standard build tools

### Windows
- Standard build tools

## License

See LICENSE file for details.

## Credits

Inspired by [cargo-nw](https://github.com/aspectron/cargo-nw)