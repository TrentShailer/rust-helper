//! Helpers for application config.
//!

use core::{error::Error, fmt};
use std::{fs, io, path::PathBuf};

use jsonschema::ValidationOptions;
use serde::{Serialize, de::DeserializeOwned};

use crate::json::{self, OutputFormat, ValidationErrors};

/// Defined behaviours for a config file.
pub trait ConfigFile: Default + DeserializeOwned + Serialize {
    /// The places to look for existing config paths, ordered in preference.
    fn config_file_paths() -> Vec<PathBuf>;

    /// Return the JSON schema for a given config version.
    fn schema(version: &str) -> Option<&'static [u8]>;

    /// Update a config to the latest version returning if the config was updated.
    fn update(&self) -> (Self, bool);

    /// Delete the config file.
    fn delete(&self) -> io::Result<()>;

    /// Write the config file.
    fn write(&self) -> io::Result<()>;
}

/// Try get the schema for a file from it's version.
pub fn try_get_schema<C: ConfigFile>(
    document: &serde_json::Value,
) -> Result<serde_json::Value, VersionError> {
    let Some(version_node) = document.get("_version") else {
        return Err(VersionError::MissingProperty);
    };

    let Some(version) = version_node.as_str() else {
        return Err(VersionError::NotAString);
    };

    let schema_bytes = C::schema(version).ok_or(VersionError::InvalidVersion {
        version: version.to_string(),
    })?;

    let schema = serde_json::from_slice::<serde_json::Value>(schema_bytes)
        .expect("JSON schema should be valid JSON");
    Ok(schema)
}

/// Try load a config file.
pub fn try_load_config<C: ConfigFile>(output_format: OutputFormat) -> Result<C, LoadConfigError> {
    let mut config_path: Option<PathBuf> = None;
    for candidate_config_path in C::config_file_paths() {
        if fs::exists(&candidate_config_path).map_err(|source| LoadConfigError::ReadError {
            path: candidate_config_path.clone(),
            source,
        })? {
            config_path = Some(candidate_config_path);
            break;
        }
    }

    let Some(config_path) = config_path else {
        return Err(LoadConfigError::FileNotFound {
            path: C::config_file_paths().last().unwrap().to_path_buf(),
        });
    };

    // Load config
    let raw_config = {
        let contents = fs::read(&config_path).map_err(|source| LoadConfigError::ReadError {
            path: config_path.clone(),
            source,
        })?;

        serde_json::from_slice::<serde_json::Value>(&contents).map_err(|source| {
            LoadConfigError::InvalidJson {
                path: config_path.clone(),
                source,
            }
        })?
    };

    // Load schema
    let schema =
        try_get_schema::<C>(&raw_config).map_err(|source| LoadConfigError::VersionError {
            path: config_path.clone(),
            source,
        })?;

    // Lint
    json::validate(
        &schema,
        &raw_config,
        ValidationOptions::default(),
        Some(config_path.clone()),
        output_format,
    )
    .map_err(|source| LoadConfigError::ValidationError {
        error_count: source.problems.len(),
        path: config_path,
        source,
    })?;

    // Deserialize
    let config: C = serde_json::from_value(raw_config)
        .expect("a file validated by the JSON schema must be able to be deserialized");

    Ok(config)
}

/// Error variants for a documents version.
#[derive(Debug)]
#[non_exhaustive]
pub enum VersionError {
    /// The document is missing the version property.
    #[non_exhaustive]
    MissingProperty,

    /// The document version is not a string.
    #[non_exhaustive]
    NotAString,

    /// The document has a version that does not exist.
    #[non_exhaustive]
    InvalidVersion {
        /// The version.
        version: String,
    },
}
impl fmt::Display for VersionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Self::MissingProperty => write!(f, "the `_version` property is missing"),
            Self::NotAString => write!(f, "the `_version` property is not a string"),
            Self::InvalidVersion { version } => write!(f, "the version `{version}` does not exist"),
        }
    }
}
impl Error for VersionError {}

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

    /// The config file has an invalid version.
    #[non_exhaustive]
    VersionError {
        /// The config file.
        path: PathBuf,
        /// The source.
        source: VersionError,
    },

    /// The config file has some validation problems.
    #[non_exhaustive]
    ValidationError {
        /// The number of problems.
        error_count: usize,
        /// The config file.
        path: PathBuf,
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
            Self::ValidationError {
                error_count, path, ..
            } => write!(
                f,
                "config file `{}` generated {} errors",
                path.to_string_lossy(),
                error_count
            ),
            Self::VersionError { path, .. } => {
                write!(
                    f,
                    "could not determine version for config file `{}`",
                    path.to_string_lossy()
                )
            }
        }
    }
}
impl Error for LoadConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self {
            Self::FileNotFound { .. } => None,
            Self::ReadError { source, .. } => Some(source),
            Self::InvalidJson { source, .. } => Some(source),
            Self::ValidationError { source, .. } => Some(source),
            Self::VersionError { source, .. } => Some(source),
        }
    }
}
