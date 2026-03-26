use log_analyzer::analyzer::LogAnalyzer;
use log_analyzer::log_level::LogLevel;
use std::path::Path;

// ---- Fixture paths ----
fn simple_log() -> &'static Path {
    Path::new("tests/fixtures/simple.log")
}

fn malformed_log() -> &'static Path {
    Path::new("tests/fixtures/malformed.log")
}

#[test]
fn test_process_simple_log_file() {
    let mut analyzer = LogAnalyzer::new();
    let count = analyzer.process_file(simple_log()).unwrap();
    assert_eq!(count, 5);
    assert_eq!(analyzer.parse_errors().len(), 0);
}

#[test]
fn test_simple_log_statistics() {
    let mut analyzer = LogAnalyzer::new();
    analyzer.process_file(simple_log()).unwrap();
    let stats = analyzer.statistics();

    assert_eq!(stats.total_entries, 5);
    assert_eq!(stats.error_count, 1);
    assert!((stats.error_rate - 0.2).abs() < 1e-9);
}

#[test]
fn test_simple_log_components() {
    let mut analyzer = LogAnalyzer::new();
    analyzer.process_file(simple_log()).unwrap();
    let stats = analyzer.statistics();

    // Verify known components exist
    assert!(stats.entries_by_component.contains_key("app"));
    assert!(stats.entries_by_component.contains_key("network"));
    assert!(stats.entries_by_component.contains_key("storage"));
}

#[test]
fn test_process_malformed_log() {
    let mut analyzer = LogAnalyzer::new();
    let count = analyzer.process_file(malformed_log()).unwrap();

    // 2 valid entries, 3 bad lines
    assert_eq!(count, 2);
    assert_eq!(analyzer.parse_errors().len(), 3);
}

#[test]
fn test_parse_errors_contain_details() {
    let mut analyzer = LogAnalyzer::new();
    analyzer.process_file(malformed_log()).unwrap();
    let errors = analyzer.parse_errors();

    // Every error should have a non-empty reason
    for err in errors {
        assert!(!err.reason.is_empty(), "Error reason should not be empty");
        assert!(err.line_number > 0, "Line number should be > 0");
    }
}

#[test]
fn test_missing_file_returns_error() {
    let mut analyzer = LogAnalyzer::new();
    let result = analyzer.process_file(Path::new("nonexistent_file.log"));
    assert!(result.is_err());
}

#[test]
fn test_multiple_files_accumulate() {
    let mut analyzer = LogAnalyzer::new();
    analyzer.process_file(simple_log()).unwrap();
    analyzer.process_file(malformed_log()).unwrap();

    // simple.log: 5 valid, malformed.log: 2 valid
    assert_eq!(analyzer.entries().len(), 7);
}

// ---- Log level ordering tests ----

#[test]
fn test_log_level_ordering() {
    assert!(LogLevel::Trace < LogLevel::Debug);
    assert!(LogLevel::Warn > LogLevel::Info);
    assert!(LogLevel::Fatal >= LogLevel::Error);
}

// ---- Statistics edge cases ----

#[test]
fn test_statistics_empty_gives_zero_error_rate() {
    let stats = log_analyzer::statistics::Statistics::from_entries(&[]);
    assert_eq!(stats.error_rate, 0.0);
    assert!(stats.first_entry.is_none());
    assert!(stats.last_entry.is_none());
}

// ---- JSON / Text report smoke tests ----

#[test]
fn test_text_report_is_non_empty() {
    let mut analyzer = LogAnalyzer::new();
    analyzer.process_file(simple_log()).unwrap();
    let stats = analyzer.statistics();
    let report = log_analyzer::report::format_text_report(&stats, analyzer.parse_errors());
    assert!(report.contains("LOG ANALYSIS REPORT"));
    assert!(report.contains("Total Entries"));
}

#[test]
fn test_json_report_is_valid_looking() {
    let mut analyzer = LogAnalyzer::new();
    analyzer.process_file(simple_log()).unwrap();
    let stats = analyzer.statistics();
    let json = log_analyzer::report::format_json_report(&stats, analyzer.parse_errors());
    assert!(json.trim_start().starts_with('{'));
    assert!(json.trim_end().ends_with('}'));
    assert!(json.contains("total_entries"));
    assert!(json.contains("error_rate"));
}
