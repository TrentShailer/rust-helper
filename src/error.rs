//! Helpers for working with errors.

use core::{error::Error, fmt};

use simply_colored::{BOLD, RED, RESET};

/// Result that coerces any error into a report for display.
pub type ReportResult<'a, T, E = Report<'a>> = Result<T, E>;

/// Trait to log a result.
pub trait ErrorLogger {
    /// Log the result
    #[track_caller]
    fn log_error(self) -> Self;

    /// Print the result.
    fn println_error(self) -> Self;
}

impl<T, E: fmt::Display> ErrorLogger for Result<T, E> {
    #[track_caller]
    fn log_error(self) -> Self {
        if let Err(error) = self.as_ref() {
            log::error!("{error}");
        }
        self
    }

    fn println_error(self) -> Self {
        if let Err(error) = self.as_ref() {
            println!("{error}");
        }
        self
    }
}
impl<T> ErrorLogger for Option<T> {
    #[track_caller]
    fn log_error(self) -> Self {
        if self.is_none() {
            log::error!("value was None");
        }
        self
    }

    fn println_error(self) -> Self {
        if self.is_none() {
            println!("value was None");
        }
        self
    }
}

/// Extension trait for reporting a result
pub trait IntoErrorReport<'a, T>: Sized {
    /// Convert the result into a report.
    fn into_report<S: ToString>(self, style: ReportStyle, operation: S) -> ReportResult<'a, T>;
}

impl<'a, T, E: Error + 'a> IntoErrorReport<'a, T> for Result<T, E> {
    fn into_report<S: ToString>(self, style: ReportStyle, operation: S) -> ReportResult<'a, T> {
        self.map_err(|error| Report {
            error: Box::new(error),
            style,
            operation: operation.to_string(),
        })
    }
}

impl<'a, T> IntoErrorReport<'a, T> for Option<T> {
    fn into_report<S: ToString>(self, style: ReportStyle, operation: S) -> ReportResult<'a, T> {
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
            style,
            operation: operation.to_string(),
        })
    }
}

/// Report styles.
pub enum ReportStyle {
    /// Basic uncoloured.
    Basic,
    /// Coloured.
    Coloured,
    /// Single line.
    SingleLine,
}

impl ReportStyle {
    /// Write an error with this style.
    pub fn write_error(
        &self,
        f: &mut fmt::Formatter<'_>,
        index: usize,
        error: &(dyn Error),
    ) -> fmt::Result {
        match &self {
            Self::Basic => writeln!(f, "  {index}: {error}"),
            Self::Coloured => writeln!(f, "  {BOLD}{RED}{index}{RESET}{BOLD}:{RESET} {error}"),
            Self::SingleLine => write!(f, " -> {error}"),
        }
    }
}

/// Report for printing a nice error report.
pub struct Report<'a> {
    /// The error stack.
    pub error: Box<dyn Error + 'a>,
    /// The report style.
    pub style: ReportStyle,
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
            style: ReportStyle::Basic,
            operation: option_env!("CARGO_BIN_NAME")
                .unwrap_or("command")
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
            self.style.write_error(f, index, error)?;
            current_error = error.source();
            index += 1;
        }

        Ok(())
    }
}
