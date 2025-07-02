//! Helpers for application config.
//!

use core::{error::Error, fmt};
use std::{
    fs, io,
    path::{Path, PathBuf},
};

use jsonschema::ValidationOptions;
use serde::{Serialize, de::DeserializeOwned};

use crate::json::{self, PositionedJsonNode, ValidationErrors};

/// Defined behaviours for a config file.
pub trait ConfigFile: Default + DeserializeOwned + Serialize {
    /// The path to the config file.
    fn config_file_path() -> PathBuf;

    /// Return the JSON schema for the config.
    fn schema() -> serde_json::Value;

    /// Delete the config file.
    fn delete(&self) -> io::Result<()>;

    /// Write the config file.
    fn write(&self) -> io::Result<()>;
}

/// Try load a config file.
pub fn try_load_config<C: ConfigFile>() -> Result<C, LoadConfigError> {
    let path = C::config_file_path();

    if !fs::exists(&path).map_err(|source| LoadConfigError::read_error(&path, source))? {
        return Err(LoadConfigError::file_not_found(&path));
    }

    let raw_document =
        fs::read_to_string(&path).map_err(|source| LoadConfigError::read_error(&path, source))?;

    // Parse the document as a node tree.
    let document = serde_json::from_str::<serde_json::Value>(&raw_document)
        .map_err(|source| LoadConfigError::invalid_json(&path, source))?;

    // Try parse the document as a node tree - recording node positions.
    let positioned_document = PositionedJsonNode::try_parse(&raw_document);

    // Lint
    json::validate(
        &C::schema(),
        &document,
        ValidationOptions::default(),
        positioned_document.as_ref(),
        Some(path.clone()),
    )
    .map_err(LoadConfigError::validation_error)?;

    // Deserialize
    let config: C = serde_json::from_value(document)
        .expect("a file validated by the JSON schema must be able to be deserialized");

    Ok(config)
}

/// Error variants from loading the config.
#[derive(Debug)]
#[non_exhaustive]
#[allow(missing_docs)]
pub enum LoadConfigError {
    #[non_exhaustive]
    FileNotFound { path: PathBuf },

    #[non_exhaustive]
    ReadError { path: PathBuf, source: io::Error },

    #[non_exhaustive]
    InvalidJson {
        path: PathBuf,
        source: serde_json::Error,
    },

    #[non_exhaustive]
    ValidationError { source: ValidationErrors },
}
impl LoadConfigError {
    #![allow(missing_docs)]
    pub fn file_not_found(path: &Path) -> Self {
        Self::FileNotFound {
            path: path.to_owned(),
        }
    }
    pub fn read_error(path: &Path, source: io::Error) -> Self {
        Self::ReadError {
            path: path.to_owned(),
            source,
        }
    }
    pub fn invalid_json(path: &Path, source: serde_json::Error) -> Self {
        Self::InvalidJson {
            path: path.to_owned(),
            source,
        }
    }
    pub fn validation_error(source: ValidationErrors) -> Self {
        Self::ValidationError { source }
    }
}
impl fmt::Display for LoadConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Self::FileNotFound { path, .. } => {
                write!(f, "config file `{}` does not exist", path.to_string_lossy())
            }
            Self::ReadError { path, .. } => {
                write!(f, "could not read config file `{}`", path.to_string_lossy())
            }
            Self::InvalidJson { path, .. } => write!(
                f,
                "config file `{}` is not valid JSON",
                path.to_string_lossy()
            ),
            Self::ValidationError { source, .. } => write!(f, "{source}"),
        }
    }
}
impl Error for LoadConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self {
            Self::ReadError { source, .. } => Some(source),
            Self::InvalidJson { source, .. } => Some(source),
            _ => None,
        }
    }
}
