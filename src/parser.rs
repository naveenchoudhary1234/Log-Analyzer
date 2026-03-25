use crate::datetime::DateTime;
use crate::errors::ParseError;
use crate::log_level::LogLevel;
use std::path::{Path, PathBuf};

/// One parsed log entry
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: DateTime,
    pub level: LogLevel,
    pub component: String,
    pub message: String,
    pub source_file: PathBuf,
}

/// Format: "YYYY-MM-DD HH:MM:SS [LEVEL] component: message"
pub fn parse_log_line(
    line: &str,
    source_file: &Path,
    line_number: usize,
) -> Result<LogEntry, ParseError> {
    let make_err = |reason: &str| ParseError {
        file: source_file.to_path_buf(),
        line_number,
        content: line.to_string(),
        reason: reason.to_string(),
    };

    let line = line.trim();

    if line.is_empty() {
        return Err(make_err("Empty line"));
    }

    if line.len() < 19 {
        return Err(make_err("Line too short to contain a timestamp"));
    }

    let timestamp_str = &line[..19];
    let timestamp = timestamp_str
        .parse::<DateTime>()
        .map_err(|e| make_err(&format!("Invalid timestamp: {}", e)))?;

    let rest = line[19..].trim_start();

    if !rest.starts_with('[') {
        return Err(make_err("Expected '[' after timestamp for log level"));
    }

    let close_bracket = rest
        .find(']')
        .ok_or_else(|| make_err("Missing closing ']' for log level"))?;

    let level_str = &rest[1..close_bracket];
    let level = level_str
        .parse::<LogLevel>()
        .map_err(|e| make_err(&format!("Invalid log level: {}", e)))?;

    let rest = rest[close_bracket + 1..].trim_start();

    let colon_pos = rest
        .find(':')
        .ok_or_else(|| make_err("Missing ':' separator between component and message"))?;

    let component = rest[..colon_pos].trim().to_string();
    if component.is_empty() {
        return Err(make_err("Component name is empty"));
    }

    let message = rest[colon_pos + 1..].trim().to_string();

    Ok(LogEntry {
        timestamp,
        level,
        component,
        message,
        source_file: source_file.to_path_buf(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn dummy_path() -> &'static Path {
        Path::new("test.log")
    }

    #[test]
    fn test_parse_valid_line() {
        let line = "2024-01-15 10:23:45 [ERROR] storage: Failed to mount filesystem";
        let entry = parse_log_line(line, dummy_path(), 1).unwrap();
        assert_eq!(entry.level, LogLevel::Error);
        assert_eq!(entry.component, "storage");
        assert_eq!(entry.message, "Failed to mount filesystem");
        assert_eq!(entry.timestamp.year, 2024);
    }

    #[test]
    fn test_parse_all_levels() {
        let levels = ["TRACE", "DEBUG", "INFO", "WARN", "ERROR", "FATAL"];
        for lvl in &levels {
            let line = format!("2024-01-15 10:00:00 [{}] app: test message", lvl);
            let entry = parse_log_line(&line, dummy_path(), 1).unwrap();
            assert_eq!(entry.level.to_string(), *lvl);
        }
    }

    #[test]
    fn test_handle_extra_whitespace() {
        // Extra spaces around component / message
        let line = "2024-01-15 10:23:45 [INFO]  network :  Connection established";
        let entry = parse_log_line(line, dummy_path(), 1).unwrap();
        assert_eq!(entry.component, "network");
        assert_eq!(entry.message, "Connection established");
    }

    #[test]
    fn test_reject_no_timestamp() {
        let line = "This line has no timestamp";
        assert!(parse_log_line(line, dummy_path(), 1).is_err());
    }

    #[test]
    fn test_reject_invalid_level() {
        let line = "2024-01-15 10:00:00 [UNKNOWN] bad: Invalid level";
        let err = parse_log_line(line, dummy_path(), 1).unwrap_err();
        assert!(err.reason.contains("log level"));
    }

    #[test]
    fn test_reject_missing_colon() {
        let line = "2024-01-15 10:00:00 [INFO] component_without_colon";
        assert!(parse_log_line(line, dummy_path(), 1).is_err());
    }

    #[test]
    fn test_error_contains_line_number() {
        let line = "garbage";
        let err = parse_log_line(line, dummy_path(), 42).unwrap_err();
        assert_eq!(err.line_number, 42);
    }
}
