use std::fmt;

/// Error types for KiCad file parsing operations
#[derive(Debug)]
pub enum KicadError {
    /// IO error occurred while reading files
    IoError(std::io::Error),
    
    /// Parse error with descriptive message
    ParseError(String),
    
    /// Invalid file format detected
    InvalidFormat(String),
    
    /// Required field is missing from the parsed data
    MissingField(String),
    
    /// Unexpected token encountered during parsing
    UnexpectedToken(String),
}

impl fmt::Display for KicadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            KicadError::IoError(e) => write!(f, "IO error: {}", e),
            KicadError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            KicadError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            KicadError::MissingField(field) => write!(f, "Missing field: {}", field),
            KicadError::UnexpectedToken(token) => write!(f, "Unexpected token: {}", token),
        }
    }
}

impl std::error::Error for KicadError {}

impl From<std::io::Error> for KicadError {
    fn from(error: std::io::Error) -> Self {
        KicadError::IoError(error)
    }
}

/// Result type for KiCad parsing operations
pub type Result<T> = std::result::Result<T, KicadError>;