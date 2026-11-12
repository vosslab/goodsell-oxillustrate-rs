use std::fmt;
use std::path::PathBuf;

#[derive(Debug)]
/// Failures distinguishable by library callers without parser-specific types.
pub enum Error {
    /// Reading or writing a named path failed.
    Io {
        /// Path involved in the failed operation.
        path: PathBuf,
        /// Operating-system error retained as the source.
        source: std::io::Error,
    },
    /// Input could not be parsed as the identified structure format.
    Parse {
        /// Format parser that reported the failure.
        format: &'static str,
        /// Human-readable parser detail.
        message: String,
    },
    /// Scene configuration failed validation.
    Settings(String),
    /// A requested model, assembly, or coordinate selection is unavailable or invalid.
    Selection(String),
    /// Selection and visibility rules produced no renderable image content.
    EmptyScene,
    /// Image dimensions or a required memory reservation could not be accommodated.
    Allocation {
        /// Requested image width in pixels.
        width: u32,
        /// Requested image height in pixels.
        height: u32,
    },
    /// PNG encoding failed; the boxed source retains the encoder error without exposing its crate type.
    Image(Box<dyn std::error::Error + Send + Sync>),
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => write!(formatter, "{}: {source}", path.display()),
            Self::Parse { format, message } => write!(formatter, "{format} input: {message}"),
            Self::Settings(message) => write!(formatter, "invalid scene settings: {message}"),
            Self::Selection(message) => write!(formatter, "invalid selection: {message}"),
            Self::EmptyScene => formatter.write_str("scene contains no visible atoms"),
            Self::Allocation { width, height } => {
                write!(formatter, "cannot allocate {width} x {height} image")
            }
            Self::Image(source) => write!(formatter, "image output: {source}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Image(source) => Some(source.as_ref()),
            _ => None,
        }
    }
}
