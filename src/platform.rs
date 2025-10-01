use crate::result::Result;
use crate::context::Context;
use crate::manifest::Manifest;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Windows,
    Linux,
    MacOS,
}

impl Platform {
    /// Get the current platform
    pub fn current() -> Self {
        if cfg!(target_os = "macos") {
            Platform::MacOS
        } else if cfg!(target_os = "windows") {
            Platform::Windows
        } else if cfg!(target_os = "linux") {
            Platform::Linux
        } else {
            panic!("Unsupported platform");
        }
    }

    /// Get platform identifier as string
    pub fn as_str(&self) -> &'static str {
        match self {
            Platform::MacOS => "macos",
            Platform::Windows => "windows",
            Platform::Linux => "linux",
        }
    }
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Execute platform-specific build
#[allow(dead_code)]
pub fn build(ctx: &Context, manifest: &Manifest) -> Result<()> {
    match Platform::current() {
        Platform::MacOS => {
            #[cfg(target_os = "macos")]
            crate::macos::build(ctx, manifest)?;
        }
        Platform::Windows => {
            #[cfg(target_os = "windows")]
            crate::windows::build(ctx, manifest)?;
        }
        Platform::Linux => {
            #[cfg(target_os = "linux")]
            crate::linux::build(ctx, manifest)?;
        }
    }
    Ok(())
}

