//! Helpers for working with errors.

use core::{error::Error, fmt};

use crate::style::{BOLD, RED, RESET};

/// Result that coerces any error into a report for display.
pub type ReportResult<'a, T, E = Report<'a>> = Result<T, E>;

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

/// Extension trait for reporting a result
pub trait IntoErrorReport<'a, T>: Sized {
    /// Convert the result into a report.
    fn into_report<S: ToString>(self, operation: S) -> ReportResult<'a, T>;
}

impl<'a, T, E: Error + 'a> IntoErrorReport<'a, T> for Result<T, E> {
    fn into_report<S: ToString>(self, operation: S) -> ReportResult<'a, T> {
        self.map_err(|error| Report {
            error: Box::new(error),
            operation: operation.to_string(),
        })
    }
}

impl<'a, T> IntoErrorReport<'a, T> for Option<T> {
    fn into_report<S: ToString>(self, operation: S) -> ReportResult<'a, T> {
        #[derive(Debug)]
        struct NoneError;
        impl fmt::Display for NoneError {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "")
            }
        }
        impl Error for NoneError {}

        self.ok_or_else(|| Report {
            error: Box::new(NoneError),
            operation: operation.to_string(),
        })
    }
}

/// Report for printing a nice error report.
pub struct Report<'a> {
    /// The error stack.
    pub error: Box<dyn Error + 'a>,
    /// The operation.
    pub operation: String,
}

impl<'a, E> From<E> for Report<'a>
where
    E: Error + 'a,
{
    fn from(value: E) -> Self {
        Self {
            error: Box::new(value),
            operation: option_env!("CARGO_BIN_NAME")
                .unwrap_or("process")
                .to_string(),
        }
    }
}

impl fmt::Debug for Report<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

impl fmt::Display for Report<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut current_error = Some(self.error.as_ref());

        writeln!(f, "`{}` returned unsuccessfully", self.operation)?;

        let mut index = 1;
        while let Some(error) = current_error {
            writeln!(f, "  {BOLD}{RED}{index}{RESET}{BOLD}:{RESET} {error}")?;
            current_error = error.source();
            index += 1;
        }

        Ok(())
    }
}
