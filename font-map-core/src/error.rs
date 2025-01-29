//! Error type and related utilities

/// Result type for parsing
pub type ParseResult<T> = Result<T, ParseError>;

/// Error type for parsing errors
#[derive(Debug)]
pub enum ParseError {
    /// Unexpected EOF while parsing
    UnexpectedEof {
        /// Byte position of the error in the data
        pos: usize,

        /// Number of bytes expected
        size: usize,

        /// Description of the error
        desc: Option<&'static str>,
    },

    /// Invalid value while parsing
    InvalidValue {
        /// Byte position of the error in the data
        pos: usize,

        /// The invalid value
        value: u32,

        /// Name of the value being parsed
        name: &'static str,
    },

    /// Error while parsing
    Parse {
        /// Byte position of the error in the data
        pos: usize,

        /// Error message
        message: String,
    },

    /// IO Error
    Io(std::io::Error),
}
impl ParseError {
    /// Returns a new error with the given description
    #[must_use]
    pub fn with_desc(self, desc: &'static str) -> ParseError {
        match self {
            ParseError::UnexpectedEof { pos, size, .. } => ParseError::UnexpectedEof {
                pos,
                size,
                desc: Some(desc),
            },
            other => other,
        }
    }
}
impl std::error::Error for ParseError {}
impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ParseError::UnexpectedEof {
                pos,
                size,
                desc: Some(desc),
            } => {
                write!(
                    f,
                    "Unexpected EOF trying to read {size} bytes from {pos} while parsing {desc}"
                )
            }
            ParseError::UnexpectedEof { pos, size, .. } => {
                write!(f, "Unexpected EOF trying to read {size} bytes from {pos}")
            }
            ParseError::InvalidValue { pos, value, name } => {
                write!(f, "Invalid value {value:#0x} at {pos} while parsing {name}")
            }
            ParseError::Parse { pos, message } => {
                write!(f, "Error at {pos}: {message}")
            }
            ParseError::Io(err) => {
                write!(f, "IO Error: {err:#}")
            }
        }
    }
}

impl From<std::io::Error> for ParseError {
    fn from(err: std::io::Error) -> ParseError {
        ParseError::Io(err)
    }
}
