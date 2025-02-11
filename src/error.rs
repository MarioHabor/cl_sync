use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TomlError {
    #[error("Failed to parse TOML file: {0}")]
    ParseError(#[from] toml::de::Error),

    #[error("Failed to read TOML file: {0}")]
    FileReadError(#[from] std::io::Error),

    #[error("TOML file not found: {0}")]
    FileNotFound(PathBuf),
}
