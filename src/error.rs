use std::fmt;
use std::path::PathBuf;

#[derive(Debug)]
/// Failures distinguishable by library callers without parser-specific types.
pub enum Error {
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    Parse {
        format: &'static str,
        message: String,
    },
    Settings(String),
    Selection(String),
    EmptyScene,
    Allocation {
        width: u32,
        height: u32,
    },
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
