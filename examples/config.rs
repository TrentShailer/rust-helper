//! Config example
//!

use std::{fs, io, path::PathBuf};

use serde::{Deserialize, Serialize};
use ts_rust_helper::{
    config::{ConfigFile, try_load_config},
    error::ReportResult,
    json::OutputFormat,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "_version")]
#[serde(rename_all = "camelCase")]
enum Config {
    V1 {
        number: u64,
        value_array: Vec<String>,
        object: Object,
        object_array: Vec<Object>,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Object {
    value: f64,
}

impl Default for Config {
    fn default() -> Self {
        Self::V1 {
            number: 0,
            value_array: vec![],
            object: Object { value: 0.0 },
            object_array: vec![],
        }
    }
}

impl Config {}

impl ConfigFile for Config {
    fn config_file_paths() -> Vec<PathBuf> {
        vec![PathBuf::from("./examples/config.json")]
    }

    fn schema(version: &str) -> Option<&'static [u8]> {
        match version {
            "v1" => Some(include_bytes!("./config.schema.json")),
            _ => None,
        }
    }

    fn update(&self) -> (Self, bool) {
        (self.clone(), false)
    }

    fn delete(&self) -> io::Result<()> {
        fs::remove_file(PathBuf::from("./examples/config.json"))
    }

    fn write(&self) -> io::Result<()> {
        let json = serde_json::to_string_pretty(self).map_err(io::Error::other)?;
        fs::write(PathBuf::from("./config.json"), &json)
    }
}

fn main() -> ReportResult<'static, ()> {
    let _config: Config = try_load_config(OutputFormat::Coloured)?;

    Ok(())
}
