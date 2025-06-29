//! Helpers for working with JSON

mod location;
mod positioned_parser;
mod problem;
mod problem_messages;

pub use problem::ValidationProblem;

use core::{
    error::Error,
    fmt::{self, Debug},
};
use std::{borrow::Cow, path::PathBuf};

use jsonschema::ValidationOptions;
use serde_json::Value;

pub use positioned_parser::{Position, PositionedJsonNode};

/// The format for the validation output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// Basic uncoloured output.
    Basic,
    /// Coloured output.
    Coloured,
}

/// Validate a JSON instance against a JSON schema.
pub fn validate(
    schema: &Value,
    instance: &Value,
    validation_options: ValidationOptions,
    document: Option<&PositionedJsonNode>,
    file_path: Option<PathBuf>,
    output_format: OutputFormat,
) -> Result<(), ValidationErrors> {
    let validator = validation_options
        .build(schema)
        .expect("JSON schema must be able to create a validator");

    if !validator.is_valid(instance) {
        let mut problems = Vec::new();
        for error in validator.iter_errors(instance) {
            problems.push(ValidationProblem::new(
                error,
                schema,
                document,
                file_path.clone(),
                output_format,
            ));
        }

        return Err(ValidationErrors {
            file_path,
            problems,
        });
    }

    Ok(())
}

/// A set of problems with a JSON instance.
#[derive(Debug)]
#[non_exhaustive]
pub struct ValidationErrors {
    /// Optional path to the file that was validated.
    pub file_path: Option<PathBuf>,
    /// The set of problems.
    pub problems: Vec<ValidationProblem>,
}
impl fmt::Display for ValidationErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "`{}` generated {} errors:",
            self.file_path.as_ref().map_or_else(
                || Cow::Owned("JSON".to_string()),
                |path| path.to_string_lossy(),
            ),
            self.problems.len()
        )?;

        for problem in &self.problems {
            writeln!(f, "{problem}")?;
        }

        Ok(())
    }
}
impl Error for ValidationErrors {}
