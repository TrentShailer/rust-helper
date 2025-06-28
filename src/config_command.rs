//! Subcommands for working with config.

use core::{error::Error, fmt};
use std::{fs, io};

use clap::Subcommand;

use crate::{
    config::{ConfigFile, LoadConfigError, try_load_config},
    json::OutputFormat,
};

/// Subcommands for application config.
#[derive(Debug, Subcommand)]
pub enum ConfigSubcommand {
    /// Initialise the config if one does not exist.
    Init,
    /// Reset all configs.
    Reset,
    /// Update the config to the latest version.
    Update,
}

impl ConfigSubcommand {
    /// Execute the subcommand.
    pub fn execute<C: ConfigFile>(
        &self,
        output_format: OutputFormat,
    ) -> Result<(C, bool), ExecuteError> {
        match &self {
            Self::Init => {
                let config = Self::init().map_err(|source| ExecuteError::Init { source })?;
                Ok((config, true))
            }
            Self::Reset => {
                let config = Self::reset().map_err(|source| ExecuteError::Reset { source })?;
                Ok((config, true))
            }
            Self::Update => {
                Self::update(output_format).map_err(|source| ExecuteError::Update { source })
            }
        }
    }

    /// Update the config.
    pub fn update<C: ConfigFile>(output_format: OutputFormat) -> Result<(C, bool), UpdateError> {
        let old_config = try_load_config::<C>(output_format)
            .map_err(|source| UpdateError::LoadConfig { source })?;

        let (new_config, updated) = old_config.update();

        if updated {
            old_config
                .delete()
                .map_err(|source| UpdateError::DeleteConfig { source })?;
            new_config
                .write()
                .map_err(|source| UpdateError::WriteConfig { source })?;
        }

        Ok((new_config, updated))
    }

    /// Initialise the config.
    pub fn init<C: ConfigFile>() -> Result<C, InitError> {
        for path in C::config_file_paths() {
            if path
                .try_exists()
                .map_err(|source| InitError::CheckPathExists { source })?
            {
                return Err(InitError::AlreadyInitialised);
            }
        }

        let config = C::default();
        config
            .write()
            .map_err(|source| InitError::WriteConfig { source })?;

        Ok(config)
    }

    /// Reset the config.
    pub fn reset<C: ConfigFile>() -> Result<C, ResetError> {
        for path in C::config_file_paths() {
            if path
                .try_exists()
                .map_err(|source| ResetError::CheckPathExists { source })?
            {
                fs::remove_file(path).map_err(|source| ResetError::DeleteConfig { source })?;
            }
        }

        let config = C::default();
        config
            .write()
            .map_err(|source| ResetError::WriteConfig { source })?;

        Ok(config)
    }
}

#[derive(Debug)]
#[non_exhaustive]
/// Failed to execute the subcommand.
pub enum ExecuteError {
    /// Reset failed.
    #[non_exhaustive]
    Reset {
        /// The source.
        source: ResetError,
    },

    /// Initialise failed.
    #[non_exhaustive]
    Init {
        /// The source.
        source: InitError,
    },

    /// Update failed.
    #[non_exhaustive]
    Update {
        /// The source.
        source: UpdateError,
    },
}
impl fmt::Display for ExecuteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Self::Reset { .. } => write!(f, "could not reset config"),
            Self::Init { .. } => write!(f, "could not initialise config"),
            Self::Update { .. } => write!(f, "could not update config"),
        }
    }
}
impl Error for ExecuteError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self {
            Self::Reset { source, .. } => Some(source),
            Self::Init { source, .. } => Some(source),
            Self::Update { source, .. } => Some(source),
        }
    }
}

/// Error variants for resetting.
#[derive(Debug)]
#[non_exhaustive]
pub enum ResetError {
    /// Could not check if a config already exists.
    #[non_exhaustive]
    CheckPathExists {
        /// The source.
        source: io::Error,
    },

    /// Could not write the new config.
    #[non_exhaustive]
    WriteConfig {
        /// The source.
        source: io::Error,
    },

    /// Could not delete a config.
    #[non_exhaustive]
    DeleteConfig {
        /// The source.
        source: io::Error,
    },
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
pub enum InitError {
    /// Could not check if a config already exists.
    #[non_exhaustive]
    CheckPathExists {
        /// The source.
        source: io::Error,
    },

    /// A config file already exists.
    #[non_exhaustive]
    AlreadyInitialised,

    /// Failed to write the new config.
    #[non_exhaustive]
    WriteConfig {
        /// The source.
        source: io::Error,
    },
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

/// Error variants for updating.
#[derive(Debug)]
#[non_exhaustive]
pub enum UpdateError {
    /// Could not load the config.
    #[non_exhaustive]
    LoadConfig {
        /// The source.
        source: LoadConfigError,
    },

    /// Could not write the updated config.
    #[non_exhaustive]
    WriteConfig {
        /// The source.
        source: io::Error,
    },

    /// Could not delete the old config.
    #[non_exhaustive]
    DeleteConfig {
        /// The source.
        source: io::Error,
    },
}
impl fmt::Display for UpdateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Self::LoadConfig { .. } => write!(f, "could not load the existing config"),
            Self::WriteConfig { .. } => write!(f, "could not write new config"),
            Self::DeleteConfig { .. } => write!(f, "could not delete old config"),
        }
    }
}
impl Error for UpdateError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self {
            Self::LoadConfig { source, .. } => Some(source),
            Self::WriteConfig { source, .. } => Some(source),
            Self::DeleteConfig { source, .. } => Some(source),
        }
    }
}
