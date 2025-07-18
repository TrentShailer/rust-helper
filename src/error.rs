//! Helpers for working with errors.

use core::{
    error::Error,
    fmt::{self, Write},
};
use std::{env::current_exe, ffi::OsStr, path::PathBuf};

use crate::style::{BOLD, RED, RESET};

/// Trait to log a result.
pub trait ErrorLogger {
    /// Log the result
    #[track_caller]
    fn log_error(self) -> Self;
}

impl<T, E: fmt::Display> ErrorLogger for Result<T, E> {
    #[track_caller]
    fn log_error(self) -> Self {
        if let Err(error) = self.as_ref() {
            #[cfg(feature = "log")]
            log::error!("{error}");
            #[cfg(not(feature = "log"))]
            println!("{error}");
        }
        self
    }
}
impl<T> ErrorLogger for Option<T> {
    #[track_caller]
    fn log_error(self) -> Self {
        if self.is_none() {
            #[cfg(feature = "log")]
            log::error!("value was None");
            #[cfg(not(feature = "log"))]
            println!("value was None");
        }
        self
    }
}

/// Type alias for a program that reports it's exit.
pub type ReportProgramExit = Result<(), ProgramReport>;

/// A report for a program exit.
pub struct ProgramReport(Box<dyn Error + 'static>);
impl<E: Error + 'static> From<E> for ProgramReport {
    fn from(value: E) -> Self {
        Self(Box::new(value))
    }
}
impl fmt::Debug for ProgramReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}
impl fmt::Display for ProgramReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let exe_path = current_exe().unwrap_or_else(|_| PathBuf::from("program"));
        let exe = exe_path
            .file_name()
            .unwrap_or_else(|| OsStr::new("program"))
            .to_string_lossy();

        let report = Report::new(exe, self.0.as_ref(), ErrorStackStyle::Stacked { indent: 2 });
        write!(f, "{report}")
    }
}

/// Extension trait for reporting a result
pub trait IntoErrorReport<'a, T>: Sized {
    /// Convert the result into a report.
    fn into_report<S: ToString>(self, operation: S) -> Result<T, Report<'a>>;
}

impl<'a, T, E: Error + 'a> IntoErrorReport<'a, T> for Result<T, E> {
    fn into_report<S: ToString>(self, operation: S) -> Result<T, Report<'a>> {
        self.map_err(|source| Report::new(operation, source, ErrorStackStyle::default()))
    }
}

impl<'a, T> IntoErrorReport<'a, T> for Option<T> {
    fn into_report<S: ToString>(self, operation: S) -> Result<T, Report<'a>> {
        #[derive(Debug)]
        struct NoneError;
        impl fmt::Display for NoneError {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "value was none")
            }
        }
        impl Error for NoneError {}

        self.ok_or_else(|| Report::new(operation, NoneError, ErrorStackStyle::default()))
    }
}

/// A report of an error.
pub struct Report<'a> {
    /// The source of the error.
    pub source: Box<dyn Error + 'a>,
    /// The style of the error.
    pub style: ErrorStackStyle<'a>,
    /// The operation this report is for.
    pub operation: String,
}
impl<'a> Report<'a> {
    /// Create a new report.
    pub fn new<S: ToString, E: Error + 'a>(
        operation: S,
        source: E,
        style: ErrorStackStyle<'a>,
    ) -> Self {
        Self {
            source: Box::new(source),
            style,
            operation: operation.to_string(),
        }
    }
}
impl Error for Report<'static> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.source.as_ref())
    }
}
impl fmt::Debug for Report<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}
impl fmt::Display for Report<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let output = self.style.display(self.source.as_ref())?;

        writeln!(f, "`{}` reported an error", self.operation)?;
        writeln!(f, "{output}")?;

        Ok(())
    }
}

/// Alias for a closure to format an error.
pub type FmtErrorClosure<'a> = Box<dyn Fn(&mut String, usize, &dyn Error) -> fmt::Result + 'a>;

/// An error stack style.
pub enum ErrorStackStyle<'a> {
    /// An inline style.
    Inline,
    /// A stacked style
    Stacked {
        /// The indent for each item in the stack.
        indent: usize,
    },
    /// A custom style
    Custom(FmtErrorClosure<'a>),
}
impl Default for ErrorStackStyle<'_> {
    fn default() -> Self {
        Self::Stacked { indent: 2 }
    }
}

impl ErrorStackStyle<'_> {
    /// Display an error in the given style.
    pub fn display(&self, source: &dyn Error) -> Result<String, fmt::Error> {
        let mut output = String::new();

        let fmt_fn = self.fmt_fn();

        let mut current_error = Some(source);
        let mut index = 1;
        while let Some(error) = current_error {
            fmt_fn(&mut output, index, error)?;
            current_error = error.source();
            index += 1;
        }

        Ok(output)
    }

    fn fmt_fn(&self) -> FmtErrorClosure<'_> {
        match &self {
            Self::Inline => Box::new(|f, i, e| write!(f, " ----- {i}. {e}")),

            Self::Stacked { indent } => Box::new(|f, i, e| {
                writeln!(
                    f,
                    "{}{BOLD}{RED}{i}{RESET}{BOLD}.{RESET} {e}",
                    " ".repeat(*indent)
                )
            }),

            Self::Custom(f) => Box::new(f),
        }
    }
}
