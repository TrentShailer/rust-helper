//! Helpers for application config.
//!

use core::{error::Error, fmt};
use std::{fs, io, path::PathBuf};

use jsonschema::ValidationOptions;
use serde::{Serialize, de::DeserializeOwned};

use crate::json::{self, OutputFormat, PositionedJsonNode, ValidationErrors};

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
pub fn try_load_config<C: ConfigFile>(output_format: OutputFormat) -> Result<C, LoadConfigError> {
    let config_path = C::config_file_path();

    if !fs::exists(&config_path).map_err(|source| LoadConfigError::ReadError {
        path: config_path.clone(),
        source,
    })? {
        return Err(LoadConfigError::FileNotFound {
            path: config_path.clone(),
        });
    }

    let raw_document =
        fs::read_to_string(&config_path).map_err(|source| LoadConfigError::ReadError {
            path: config_path.clone(),
            source,
        })?;

    // Parse the document as a node tree.
    let document = serde_json::from_str::<serde_json::Value>(&raw_document).map_err(|source| {
        LoadConfigError::InvalidJson {
            path: config_path.clone(),
            source,
        }
    })?;

    // Try parse the document as a node tree - recording node positions.
    let positioned_document = PositionedJsonNode::try_parse(&raw_document);

    // Lint
    json::validate(
        &C::schema(),
        &document,
        ValidationOptions::default(),
        positioned_document.as_ref(),
        Some(config_path.clone()),
        output_format,
    )
    .map_err(|source| LoadConfigError::ValidationError { source })?;

    // Deserialize
    let config: C = serde_json::from_value(document)
        .expect("a file validated by the JSON schema must be able to be deserialized");

    Ok(config)
}

/// Error variants from loading the config.
#[derive(Debug)]
#[non_exhaustive]
pub enum LoadConfigError {
    /// The config could not be found.
    #[non_exhaustive]
    FileNotFound {
        /// The path checked.
        path: PathBuf,
    },

    /// The config could not be read.
    #[non_exhaustive]
    ReadError {
        /// The config file path.
        path: PathBuf,
        /// The source.
        source: io::Error,
    },

    /// The config file is not valid JSON.
    #[non_exhaustive]
    InvalidJson {
        /// The config file.
        path: PathBuf,
        /// The source.
        source: serde_json::Error,
    },

    /// The config file has some validation problems.
    #[non_exhaustive]
    ValidationError {
        /// The problems.
        source: ValidationErrors,
    },
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
            Self::FileNotFound { .. } => None,
            Self::ReadError { source, .. } => Some(source),
            Self::InvalidJson { source, .. } => Some(source),
            Self::ValidationError { .. } => None,
        }
    }
}
