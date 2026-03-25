use std::fmt;
use std::path::PathBuf;

/// Error when analyzing files (IO, missing files, etc.)
#[derive(Debug)]
pub enum AnalyzerError {
    IoError {
        path: PathBuf,
        source: std::io::Error,
    },
    NoFilesFound {
        path: PathBuf,
    },
    InvalidPath {
        path: PathBuf,
        reason: String,
    },
}

impl fmt::Display for AnalyzerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AnalyzerError::IoError { path, source } => {
                write!(f, "IO error reading '{}': {}", path.display(), source)
            }
            AnalyzerError::NoFilesFound { path } => {
                write!(f, "No .log files found in '{}'", path.display())
            }
            AnalyzerError::InvalidPath { path, reason } => {
                write!(f, "Invalid path '{}': {}", path.display(), reason)
            }
        }
    }
}

impl std::error::Error for AnalyzerError {}

/// Error when parsing a single log line
#[derive(Debug)]
pub struct ParseError {
    pub file: PathBuf,
    pub line_number: usize,
    pub content: String,
    pub reason: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Parse error in '{}' at line {}: {} — original: '{}'",
            self.file.display(),
            self.line_number,
            self.reason,
            self.content
        )
    }
}

impl std::error::Error for ParseError {}
