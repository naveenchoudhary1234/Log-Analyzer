use std::fs;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};

use crate::errors::{AnalyzerError, ParseError};
use crate::parser::{parse_log_line, LogEntry};
use crate::statistics::Statistics;

pub struct LogAnalyzer {
    entries: Vec<LogEntry>,
    errors: Vec<ParseError>,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            entries: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// Process a single log file. Returns number of successfully parsed entries.
    pub fn process_file(&mut self, path: &Path) -> Result<usize, AnalyzerError> {
        if !path.exists() {
            return Err(AnalyzerError::IoError {
                path: path.to_path_buf(),
                source: io::Error::new(io::ErrorKind::NotFound, "File not found"),
            });
        }

        if !path.is_file() {
            return Err(AnalyzerError::InvalidPath {
                path: path.to_path_buf(),
                reason: "Path is not a file".to_string(),
            });
        }

        let file = fs::File::open(path).map_err(|e| AnalyzerError::IoError {
            path: path.to_path_buf(),
            source: e,
        })?;

        let reader = io::BufReader::new(file);
        let mut count = 0;

        for (idx, line_result) in reader.lines().enumerate() {
            let line_number = idx + 1;

            let line = line_result.map_err(|e| AnalyzerError::IoError {
                path: path.to_path_buf(),
                source: e,
            })?;

            // Skip blank lines silently
            if line.trim().is_empty() {
                continue;
            }

            match parse_log_line(&line, path, line_number) {
                Ok(entry) => {
                    self.entries.push(entry);
                    count += 1;
                }
                Err(e) => {
                    self.errors.push(e);
                }
            }
        }

        Ok(count)
    }

    /// Process all *.log files in a directory. Returns total entries parsed.
    pub fn process_directory(&mut self, path: &Path) -> Result<usize, AnalyzerError> {
        if !path.exists() {
            return Err(AnalyzerError::IoError {
                path: path.to_path_buf(),
                source: io::Error::new(io::ErrorKind::NotFound, "Directory not found"),
            });
        }

        if !path.is_dir() {
            return Err(AnalyzerError::InvalidPath {
                path: path.to_path_buf(),
                reason: "Path is not a directory".to_string(),
            });
        }

        let log_files: Vec<PathBuf> = fs::read_dir(path)
            .map_err(|e| AnalyzerError::IoError {
                path: path.to_path_buf(),
                source: e,
            })?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|p| p.is_file() && p.extension().is_some_and(|ext| ext == "log"))
            .collect();

        if log_files.is_empty() {
            return Err(AnalyzerError::NoFilesFound {
                path: path.to_path_buf(),
            });
        }

        let mut total = 0;
        for file_path in &log_files {
            total += self.process_file(file_path)?;
        }

        Ok(total)
    }

    pub fn entries(&self) -> &[LogEntry] {
        &self.entries
    }

    pub fn parse_errors(&self) -> &[ParseError] {
        &self.errors
    }

    pub fn statistics(&self) -> Statistics {
        Statistics::from_entries(&self.entries)
    }
}

impl Default for LogAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    // Helper: write content to a temp .log file and return its path
    fn write_temp_log(content: &str) -> (NamedTempFile, PathBuf) {
        let mut f = NamedTempFile::new().unwrap();
        write!(f, "{}", content).unwrap();
        let path = f.path().to_path_buf();
        (f, path)
    }

    #[test]
    fn test_process_valid_file() {
        let content = "\
2024-01-15 10:00:00 [INFO] app: Application started
2024-01-15 10:00:01 [ERROR] storage: Disk full
";
        let (_f, path) = write_temp_log(content);
        let mut analyzer = LogAnalyzer::new();
        let count = analyzer.process_file(&path).unwrap();
        assert_eq!(count, 2);
        assert_eq!(analyzer.entries().len(), 2);
        assert_eq!(analyzer.parse_errors().len(), 0);
    }

    #[test]
    fn test_handle_missing_file() {
        let mut analyzer = LogAnalyzer::new();
        let result = analyzer.process_file(Path::new("/nonexistent/path/file.log"));
        assert!(result.is_err());
    }

    #[test]
    fn test_skip_malformed_lines() {
        let content = "\
2024-01-15 10:00:00 [INFO] app: Valid entry
This line has no timestamp
2024-01-15 10:00:01 [INFO] network: Another valid entry
";
        let (_f, path) = write_temp_log(content);
        let mut analyzer = LogAnalyzer::new();
        let count = analyzer.process_file(&path).unwrap();
        assert_eq!(count, 2); // 2 valid entries
        assert_eq!(analyzer.parse_errors().len(), 1); // 1 bad line stored
    }

    #[test]
    fn test_statistics_after_processing() {
        let content = "\
2024-01-15 10:00:00 [ERROR] storage: Disk full
2024-01-15 10:00:01 [INFO] app: Started
";
        let (_f, path) = write_temp_log(content);
        let mut analyzer = LogAnalyzer::new();
        analyzer.process_file(&path).unwrap();
        let stats = analyzer.statistics();
        assert_eq!(stats.total_entries, 2);
        assert_eq!(stats.error_count, 1);
    }
}
