//! Config example
//!

use std::{fs, io, path::PathBuf};

use schemars::{JsonSchema, generate::SchemaSettings};
use serde::{Deserialize, Serialize};
use ts_rust_helper::{
    command::{Cli, Command},
    config::{ConfigFile, try_load_config},
    error::{IntoErrorReport, ReportProgramExit},
};

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(tag = "_version")]
#[serde(rename_all = "camelCase")]
struct Config {
    /// A number.
    /// Try `0`
    number: u64,
    value_array: Vec<String>,
    object: Object,
    object_array: Vec<Object>,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct Object {
    value: f64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            number: 0,
            value_array: vec![],
            object: Object { value: 0.0 },
            object_array: vec![],
        }
    }
}

impl Config {}

impl ConfigFile for Config {
    fn config_file_path() -> PathBuf {
        PathBuf::from("./examples/config.json")
    }

    fn schema() -> serde_json::Value {
        let settings = SchemaSettings::draft07();
        let generator = settings.into_generator();
        let schema = generator.into_root_schema_for::<Self>();
        serde_json::to_value(schema).unwrap()
    }

    fn delete(&self) -> io::Result<()> {
        fs::remove_file(PathBuf::from("./examples/config.json"))
    }

    fn write(&self) -> io::Result<()> {
        let json = serde_json::to_string_pretty(self).map_err(io::Error::other)?;
        fs::write(PathBuf::from("./config.json"), &json)
    }
}

fn main() -> ReportProgramExit {
    let cli = Cli::parse();
    if let Some(subcommand) = cli.subcommand {
        match subcommand {
            Command::Config(config_subcommand) => config_subcommand.execute::<Config>()?,
        }

        return Ok(());
    }

    let _config: Config = try_load_config().into_report("load config")?;

    Ok(())
}
