use std::fmt;
use std::io;
use std::num::ParseIntError;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug)]
pub enum AppError {
    CommandSpawn {
        command: String,
        source: io::Error,
    },
    Io(io::Error),
    MissingDependency {
        command: String,
        install_hint: String,
    },
    OcrProducedNoText,
    ParseInt(ParseIntError),
    ProcessFailed {
        command: String,
        stderr: String,
    },
}

impl fmt::Display for AppError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::CommandSpawn { command, source } => {
                write!(formatter, "could not start `{command}`: {source}")
            }
            AppError::Io(error) => write!(formatter, "{error}"),
            AppError::MissingDependency {
                command,
                install_hint,
            } => {
                write!(
                    formatter,
                    "`{command}` is not installed. Install it with: {install_hint}"
                )
            }
            AppError::OcrProducedNoText => write!(formatter, "OCR did not find any text"),
            AppError::ParseInt(error) => write!(formatter, "{error}"),
            AppError::ProcessFailed { command, stderr } => {
                if stderr.is_empty() {
                    write!(formatter, "`{command}` failed")
                } else {
                    write!(formatter, "`{command}` failed: {stderr}")
                }
            }
        }
    }
}

impl std::error::Error for AppError {}

impl From<io::Error> for AppError {
    fn from(value: io::Error) -> Self {
        AppError::Io(value)
    }
}

impl From<ParseIntError> for AppError {
    fn from(value: ParseIntError) -> Self {
        AppError::ParseInt(value)
    }
}
