//! Subcommands for working with config.

use core::{error::Error, fmt};
use std::{fs, io};

use clap::Subcommand;

use crate::config::{ConfigFile, LoadConfigError, try_load_config};

/// Subcommands for application config.
#[derive(Debug, Subcommand)]
pub enum ConfigSubcommand {
    /// Initialise the config if one does not exist.
    Init,
    /// Reset all configs.
    Reset,
    /// Output the config JSON schema
    Schema,
    /// Lint the config
    Lint,
}

impl ConfigSubcommand {
    /// Execute the subcommand.
    pub fn execute<C: ConfigFile>(&self) -> Result<(), ExecuteError> {
        match &self {
            Self::Init => {
                Self::init::<C>().map_err(|source| ExecuteError::Init { source })?;
            }
            Self::Reset => {
                Self::reset::<C>().map_err(|source| ExecuteError::Reset { source })?;
            }
            Self::Schema => {
                Self::schema::<C>().map_err(|source| ExecuteError::Schema { source })?;
            }
            Self::Lint => {
                Self::lint::<C>().map_err(|source| ExecuteError::Lint { source })?;
            }
        };

        Ok(())
    }

    /// Lint the config file.
    pub fn lint<C: ConfigFile>() -> Result<(), LoadConfigError> {
        let _ = try_load_config::<C>()?;
        Ok(())
    }

    /// Output the schema
    pub fn schema<C: ConfigFile>() -> serde_json::Result<()> {
        let json = serde_json::to_string_pretty(&C::schema())?;
        println!("{json}");

        Ok(())
    }

    /// Initialise the config.
    pub fn init<C: ConfigFile>() -> Result<C, InitError> {
        if C::config_file_path()
            .try_exists()
            .map_err(|source| InitError::CheckPathExists { source })?
        {
            return Err(InitError::AlreadyInitialised);
        }

        let config = C::default();
        config
            .write()
            .map_err(|source| InitError::WriteConfig { source })?;

        Ok(config)
    }

    /// Reset the config.
    pub fn reset<C: ConfigFile>() -> Result<C, ResetError> {
        if C::config_file_path()
            .try_exists()
            .map_err(|source| ResetError::CheckPathExists { source })?
        {
            fs::remove_file(C::config_file_path())
                .map_err(|source| ResetError::DeleteConfig { source })?;
        }

        let config = C::default();
        config
            .write()
            .map_err(|source| ResetError::WriteConfig { source })?;

        Ok(config)
    }
}

/// Failed to execute the subcommand.
#[derive(Debug)]
#[non_exhaustive]
#[allow(missing_docs)]
pub enum ExecuteError {
    #[non_exhaustive]
    Reset { source: ResetError },

    #[non_exhaustive]
    Init { source: InitError },

    #[non_exhaustive]
    Schema { source: serde_json::Error },

    #[non_exhaustive]
    Lint { source: LoadConfigError },
}
impl fmt::Display for ExecuteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Self::Reset { .. } => write!(f, "could not reset config"),
            Self::Init { .. } => write!(f, "could not initialise config"),
            Self::Schema { .. } => write!(f, "could not output the JSON schema"),
            Self::Lint { source } => match source {
                LoadConfigError::ValidationError { .. } => {
                    write!(f, "linting reported that the config contained errors")
                }
                _ => write!(f, "config could not be validated"),
            },
        }
    }
}
impl Error for ExecuteError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self {
            Self::Reset { source, .. } => Some(source),
            Self::Init { source, .. } => Some(source),
            Self::Schema { source, .. } => Some(source),
            Self::Lint { source, .. } => Some(source),
        }
    }
}

/// Error variants for resetting.
#[derive(Debug)]
#[non_exhaustive]
#[allow(missing_docs)]
pub enum ResetError {
    #[non_exhaustive]
    CheckPathExists { source: io::Error },

    #[non_exhaustive]
    WriteConfig { source: io::Error },

    #[non_exhaustive]
    DeleteConfig { source: io::Error },
}
impl fmt::Display for ResetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Self::CheckPathExists { .. } => write!(f, "could not check if the config exists"),
            Self::WriteConfig { .. } => write!(f, "could not write new config"),
            Self::DeleteConfig { .. } => write!(f, "could not delete old config"),
        }
    }
}
impl Error for ResetError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self {
            Self::CheckPathExists { source, .. } => Some(source),
            Self::WriteConfig { source, .. } => Some(source),
            Self::DeleteConfig { source, .. } => Some(source),
        }
    }
}

/// Error variants for initialisation.
#[derive(Debug)]
#[non_exhaustive]
#[allow(missing_docs)]
pub enum InitError {
    #[non_exhaustive]
    CheckPathExists { source: io::Error },

    #[non_exhaustive]
    AlreadyInitialised,

    #[non_exhaustive]
    WriteConfig { source: io::Error },
}
impl fmt::Display for InitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Self::CheckPathExists { .. } => write!(f, "could not check if the config exists"),
            Self::WriteConfig { .. } => write!(f, "could not write new config"),
            Self::AlreadyInitialised { .. } => write!(f, "the config is already initialised"),
        }
    }
}
impl Error for InitError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self {
            Self::CheckPathExists { source, .. } => Some(source),
            Self::WriteConfig { source, .. } => Some(source),
            _ => None,
        }
    }
}
