use core::{fmt, ops::Range};
use std::path::PathBuf;

use jsonschema::{ValidationError, error::ValidationErrorKind, paths::Location};
use serde_json::Value;
use simply_colored::{BOLD, CYAN, RED, RESET};

use crate::json::{
    OutputFormat,
    location::LocationExtensions,
    positioned_parser::{Position, PositionedJsonNode},
    problem_messages::ProblemMessage,
};

#[derive(Debug)]
#[non_exhaustive]
pub struct FileLocation {
    pub path: PathBuf,
    pub position: Option<Position>,
}

/// A validation problem.
#[derive(Debug)]
#[non_exhaustive]
pub struct ValidationProblem {
    /// Optional file location.
    pub location: Option<FileLocation>,

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
                self.write_symbol(" = ", f)?;
                match self.output_format {
                    OutputFormat::Basic => writeln!(f, "note: {note}")?,
                    OutputFormat::Coloured => writeln!(f, "{BOLD}note:{RESET} {note}")?,
                }
            }
        }

        Ok(())
    }
}

impl ValidationProblem {
    /// Create a new validation problem from a validation error.
    pub fn new(
        problem: ValidationError<'_>,
        schema: &Value,
        document: Option<&PositionedJsonNode>,
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

            if let Some(parent) = schema_path.parent()
                && let Some(node) = schema.pointer(parent.join("description").as_str())
                && let Some(contents) = node.as_str()
            {
                let mut lines = contents.split('\n');

                if let Some(expected) = lines.next() {
                    let mut chars = expected.chars();
                    notes.push(format!(
                        "this should be {}{}",
                        chars
                            .nth(0)
                            .map_or_else(|| '\0'.to_lowercase(), |v| v.to_lowercase()),
                        chars.as_str()
                    ));
                }

                for line in lines {
                    notes.push(line.to_string());
                }
            };

            notes
        };

        let (source, range) = {
            let source = instance_path
                .reconstruct(&instance)
                .lines()
                .nth(0)
                .map_or(String::new(), |v| v.to_string());

            let range = source.find(": ").map(|v| v + 2).unwrap_or(0)..source.len();

            (source, range)
        };

        let location = if let Some(document) = document
            && let Some(path) = file_path
        {
            let position = document
                .evaluate(&instance_path)
                .map(|node| node.position());
            Some(FileLocation { path, position })
        } else {
            None
        };

        Self {
            location,
            kind,
            notes,
            instance_path,
            source,
            range,
            output_format,
        }
    }

    fn indent(&self) -> usize {
        if let Some(location) = &self.location
            && let Some(position) = location.position
        {
            position.line.to_string().len()
        } else {
            1
        }
    }

    fn write_headline(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let headline = self.kind.headline();

        let node = self.instance_path.pointing_at();

        match self.output_format {
            OutputFormat::Basic => {
                writeln!(f, "error: `{node}` {headline}")
            }
            OutputFormat::Coloured => {
                writeln!(
                    f,
                    "{RED}{BOLD}error{RESET}{BOLD}: `{node}` {headline}{RESET}"
                )
            }
        }
    }

    fn write_file(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(location) = self.location.as_ref() {
            self.write_symbol("--> ", f)?;
            write!(f, "{}", location.path.to_string_lossy())?;
            if let Some(position) = location.position {
                write!(f, ":{}:{}", position.line, position.column)?;
            }
            writeln!(f)
        } else {
            Ok(())
        }
    }

    fn write_spacer(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.write_symbol(" | ", f)?;
        writeln!(f)
    }

    fn write_symbol(&self, symbol: &str, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent = " ".repeat(self.indent());
        match self.output_format {
            OutputFormat::Basic => write!(f, "{indent}{symbol}"),
            OutputFormat::Coloured => write!(f, "{indent}{BOLD}{CYAN}{symbol}{RESET}"),
        }
    }

    fn write_source(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(location) = &self.location
            && let Some(position) = location.position
        {
            let line = position.line;
            match self.output_format {
                OutputFormat::Basic => write!(f, "{line}")?,
                OutputFormat::Coloured => write!(f, "{BOLD}{CYAN}{line}{RESET}")?,
            }
        }

        match self.output_format {
            OutputFormat::Basic => write!(f, " | ")?,
            OutputFormat::Coloured => write!(f, "{BOLD}{CYAN} | {RESET}")?,
        }

        writeln!(f, "{}", self.source)
    }

    fn write_message(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.write_symbol(" | ", f)?;

        match self.output_format {
            OutputFormat::Basic => write!(
                f,
                "{}{}",
                " ".repeat(self.range.start),
                "^".repeat(self.range.len()),
            )?,
            OutputFormat::Coloured => write!(
                f,
                "{}{RED}{BOLD}{}{RESET}",
                " ".repeat(self.range.start),
                "^".repeat(self.range.len()),
            )?,
        }

        if let Some(message) = self.kind.message() {
            match self.output_format {
                OutputFormat::Basic => writeln!(f, " {message}")?,
                OutputFormat::Coloured => writeln!(f, " {RED}{BOLD}{message}{RESET}")?,
            }
        } else {
            writeln!(f)?
        }

        Ok(())
    }
}
