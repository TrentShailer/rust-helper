//! Helpers for working with JSON

use core::{
    error::Error,
    fmt::{self, Debug},
    ops::Range,
};
use std::{borrow::Cow, path::PathBuf};

use jsonschema::{
    ValidationError, ValidationOptions,
    error::ValidationErrorKind,
    paths::{Location, LocationSegment},
};
use serde_json::Value;
use simply_colored::{BOLD, CYAN, RED, RESET};

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
        for problem in &self.problems {
            writeln!(f, "{problem}")?;
        }

        writeln!(
            f,
            "`{}` generated {} errors",
            self.file_path.as_ref().map_or_else(
                || Cow::Owned("JSON".to_string()),
                |path| path.to_string_lossy(),
            ),
            self.problems.len()
        )
    }
}
impl Error for ValidationErrors {}

/// A validation problem.
#[derive(Debug)]
#[non_exhaustive]
pub struct ValidationProblem {
    /// Optional path to the file that caused this problem.
    pub file_path: Option<PathBuf>,

    /// The kind of validation problem.
    pub kind: ValidationErrorKind,

    /// Any notes about this validation problem.
    pub notes: Vec<String>,

    /// The JSON pointer to the source of this problem.
    pub instance_path: Location,
    /// The reconstructed JSON source of the problem
    pub source: String,
    /// The range to underline.
    pub range: Range<usize>,

    /// The format to output in.
    pub output_format: OutputFormat,
}

impl ValidationProblem {
    /// Create a new validation problem from a validation error.
    pub fn new(
        problem: ValidationError<'_>,
        schema: &Value,
        file_path: Option<PathBuf>,
        output_format: OutputFormat,
    ) -> Self {
        let ValidationError {
            instance,
            kind,
            instance_path,
            schema_path,
        } = problem;

        let notes = {
            let mut notes = Vec::new();

            if let Some(description_node) = schema.pointer(schema_path.join("description").as_str())
                && let Some(contents) = description_node.as_str()
            {
                let mut chars = contents.chars();
                notes.push(format!(
                    "this should be {}{}",
                    chars
                        .nth(0)
                        .map_or_else(|| '\0'.to_lowercase(), |v| v.to_lowercase()),
                    chars.as_str()
                ));
            };

            if let Some(help_node) = schema.pointer(schema_path.join("help").as_str())
                && let Some(contents) = help_node.as_str()
            {
                notes.push(contents.to_string());
            };

            notes
        };

        let (source, range) = {
            let key = instance_path.into_iter().last();

            let value = serde_json::to_string_pretty(&instance).unwrap_or(String::new());

            // TODO range depending on key or value problem
            let (key, range) = match key {
                Some(LocationSegment::Property(property)) => {
                    let key = format!("\"{property}\": ");
                    let range = 0..key.len() - 2;

                    (key, range)
                }

                Some(LocationSegment::Index(index)) => {
                    let key = format!("[{index}]: ");
                    let range = 0..key.len() - 2;

                    (key, range)
                }

                None => (String::new(), 0..1),
            };

            let source = format!("{key}{value}")
                .lines()
                .nth(0)
                .map_or(String::new(), |v| v.to_string());

            (source, range)
        };

        Self {
            file_path,
            kind,
            notes,
            instance_path,
            source,
            range,
            output_format,
        }
    }

    fn write_headline(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let type_string = self.instance_path.into_iter().last().map_or_else(
            || "property",
            |segment| match segment {
                LocationSegment::Property(_) => "property",
                LocationSegment::Index(_) => "item",
            },
        );

        let path = &self.instance_path;

        match self.output_format {
            OutputFormat::Basic => {
                writeln!(f, "error: invalid {type_string} `{path}`")
            }
            OutputFormat::Coloured => {
                writeln!(
                    f,
                    "{RED}{BOLD}error{RESET}{BOLD}: invalid {type_string} `{path}`{RESET}"
                )
            }
        }
    }

    fn write_file(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(path) = self.file_path.as_ref() {
            self.write_symbol(" --> ", f)?;
            writeln!(f, "{}", path.to_string_lossy())
        } else {
            Ok(())
        }
    }

    fn write_spacer(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.write_symbol("  | ", f)?;
        writeln!(f)
    }

    fn write_symbol(&self, symbol: &str, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.output_format {
            OutputFormat::Basic => write!(f, "{symbol}"),
            OutputFormat::Coloured => write!(f, "{BOLD}{CYAN}{symbol}{RESET}"),
        }
    }

    fn write_source(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.write_symbol("  | ", f)?;
        writeln!(f, "{}", self.source)
    }

    fn write_message(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.write_symbol("  | ", f)?;

        match self.output_format {
            OutputFormat::Basic => writeln!(
                f,
                "{}{} {}",
                " ".repeat(self.range.start),
                "^".repeat(self.range.len()),
                self.message()
            ),
            OutputFormat::Coloured => writeln!(
                f,
                "{}{RED}{BOLD}{} {}{RESET}",
                " ".repeat(self.range.start),
                "^".repeat(self.range.len()),
                self.message()
            ),
        }
    }

    fn message(&self) -> String {
        match &self.kind {
            ValidationErrorKind::AdditionalItems { limit } => {
                format!("this must contain less than or equal to {limit} items")
            }
            ValidationErrorKind::AdditionalProperties { unexpected } => {
                format!(
                    "this contains unknown properties [{}]",
                    unexpected.join(", ")
                )
            }
            ValidationErrorKind::AnyOf => {
                "this is not a valid instance of any of the allowed types".to_string()
            }
            ValidationErrorKind::Constant { expected_value } => {
                format!("expected `{expected_value}`")
            }
            ValidationErrorKind::Contains => "does not contain valid items".to_string(),
            ValidationErrorKind::Custom { message } => message.to_string(),
            ValidationErrorKind::Enum { options } => {
                format!("expected one of `{options}`")
            }
            ValidationErrorKind::Format { format } => {
                format!("this is not a valid `{format}`")
            }
            ValidationErrorKind::ExclusiveMaximum { limit } => {
                format!("this must be less than {limit}")
            }
            ValidationErrorKind::MaxItems { limit } => {
                format!("this must have less than or equal to {limit} items")
            }
            ValidationErrorKind::Maximum { limit } => {
                format!("this must be less than or equal to {limit}")
            }
            ValidationErrorKind::MaxLength { limit } => {
                format!("this must have less than or equal to {limit} characters")
            }
            ValidationErrorKind::MaxProperties { limit } => {
                format!("this must have less than or equal to {limit} properties")
            }
            ValidationErrorKind::ExclusiveMinimum { limit } => {
                format!("this must be greater than {limit}")
            }
            ValidationErrorKind::MinItems { limit } => {
                format!("this must have at least {limit} items")
            }
            ValidationErrorKind::Minimum { limit } => {
                format!("this must be at least {limit}")
            }
            ValidationErrorKind::MinLength { limit } => {
                format!("this must have at least {limit} characters")
            }
            ValidationErrorKind::MinProperties { limit } => {
                format!("this must have at least {limit} properties")
            }
            ValidationErrorKind::MultipleOf { multiple_of } => {
                format!("this must be a multiple of {multiple_of}")
            }
            ValidationErrorKind::Not { schema } => {
                format!("this must not be `{schema}`")
            }
            ValidationErrorKind::OneOfMultipleValid => {
                "this is valid for multiple variants".to_string()
            }
            ValidationErrorKind::OneOfNotValid => "this is not valid for any variant".to_string(),
            ValidationErrorKind::Pattern { .. } => {
                "this does not match the expected pattern".to_string()
            }
            ValidationErrorKind::Required { property } => {
                format!("this is missing required property `{property}`")
            }
            ValidationErrorKind::Type { kind } => format!("this is not a/an `{kind:?}`"),
            ValidationErrorKind::UnevaluatedItems { unexpected } => format!(
                "this contains unevaluated items [{}]",
                unexpected.join(", ")
            ),
            ValidationErrorKind::UnevaluatedProperties { unexpected } => format!(
                "this contains unevaluated properties [{}]",
                unexpected.join(", ")
            ),
            ValidationErrorKind::UniqueItems => "this contains duplicate items".to_string(),
            ValidationErrorKind::ContentEncoding { content_encoding } => {
                format!("this is not encoded as `{content_encoding}`")
            }
            ValidationErrorKind::ContentMediaType { content_media_type } => {
                format!("this is not the media type `{content_media_type}`")
            }

            ValidationErrorKind::BacktrackLimitExceeded { error } => {
                format!("this could not be validated: {error}")
            }
            ValidationErrorKind::FromUtf8 { error } => {
                format!("this could not be validated: {error}")
            }
            ValidationErrorKind::PropertyNames { error } => {
                format!("this could not be validated: {error}")
            }
            ValidationErrorKind::Referencing(error) => {
                format!("this could not be resolved: {error}")
            }
            ValidationErrorKind::FalseSchema => "this not valid".to_string(),
        }
    }
}
impl fmt::Display for ValidationProblem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.write_headline(f)?;
        self.write_file(f)?;
        self.write_spacer(f)?;
        self.write_source(f)?;
        self.write_message(f)?;

        if !self.notes.is_empty() {
            self.write_spacer(f)?;

            for note in &self.notes {
                self.write_symbol("  =", f)?;
                match self.output_format {
                    OutputFormat::Basic => writeln!(f, "note: {note}")?,
                    OutputFormat::Coloured => writeln!(f, "{BOLD}note:{RESET} {note}")?,
                }
            }
        }

        Ok(())
    }
}
