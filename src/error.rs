use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Custom(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML parse error: {0}")]
    #[allow(dead_code)]
    TomlParse(toml::de::Error),

    #[error("TOML serialize error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    #[error("Image error: {0}")]
    Image(#[from] image::ImageError),

    #[error("Zip error: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("Walkdir error: {0}")]
    Walkdir(#[from] walkdir::Error),

    #[error("Command execution failed: {0}")]
    CommandFailed(String),

    #[error("Manifest not found at path: {0}")]
    ManifestNotFound(String),

    #[error("Invalid manifest: {0}")]
    InvalidManifest(String),

    #[error("Platform not supported: {0}")]
    #[allow(dead_code)]
    UnsupportedPlatform(String),
}

impl Error {
    #[allow(dead_code)]
    pub fn custom<T: Into<String>>(msg: T) -> Self {
        Error::Custom(msg.into())
    }
}

impl From<&str> for Error {
    fn from(err: &str) -> Self {
        Error::Custom(err.to_string())
    }
}

impl From<String> for Error {
    fn from(err: String) -> Self {
        Error::Custom(err)
    }
}
