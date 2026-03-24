# Log Analyzer

A fast, robust log file analyzer for distributed systems written in Rust.

## Features

- Parse structured log files (`YYYY-MM-DD HH:MM:SS [LEVEL] component: message`)
- Aggregate statistics across multiple files and directories
- Output reports in **text table** or **JSON** format
- Filter by minimum log level or component name
- Graceful handling of malformed lines (stored, not crashed)
- Zero external dependencies (only `thiserror` for ergonomic errors)

## Building

```bash
cargo build --release
```

## Running Tests

```bash
cargo test
```

## Usage

```bash
# Analyze a single file (text report, default)
./log-analyzer server.log

# Analyze multiple files
./log-analyzer server1.log server2.log server3.log

# Analyze all *.log files in a directory
./log-analyzer /var/log/myapp/

# Output as JSON
./log-analyzer server.log --format json

# Show only WARN and above
./log-analyzer server.log --min-level warn

# Filter by component
./log-analyzer server.log --component storage

# Combine filters
./log-analyzer /var/log/myapp/ --min-level error --format json
```

## Sample Output

### Text Format

```
================== LOG ANALYSIS REPORT ==================
Period: 2024-01-15 10:00:00 to 2024-01-15 10:00:10

SUMMARY
-------
Total Entries:    5
Error Rate:       20.0%
Peak Hour:        10:00 (5 entries)
Most Active:      app (1 entries)

BY LOG LEVEL
------------
TRACE           0   (0.0%)
DEBUG           1   (20.0%)
INFO            2   (40.0%)
WARN            1   (20.0%)
ERROR           1   (20.0%)
FATAL           0   (0.0%)

BY COMPONENT
------------
app              1   (20.0%)
config           1   (20.0%)
network          1   (20.0%)
auth             1   (20.0%)
storage          1   (20.0%)

Parse Errors: 0 lines skipped
=========================================================
```

### JSON Format

```json
{
  "total_entries": 5,
  "error_rate": 0.200,
  "peak_hour": 10,
  "most_active_component": "app",
  "entries_by_level": {
    "TRACE": 0,
    "DEBUG": 1,
    "INFO": 2,
    "WARN": 1,
    "ERROR": 1,
    "FATAL": 0
  },
  "entries_by_component": {
    "app": 1,
    "config": 1,
    "network": 1,
    "auth": 1,
    "storage": 1
  },
  "period": {
    "start": "2024-01-15 10:00:00",
    "end": "2024-01-15 10:00:10"
  },
  "parse_errors": 0
}
```

## Project Structure

```
log-analyzer/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs          — module declarations
│   ├── main.rs         — CLI entry point
│   ├── datetime.rs     — DateTime struct (parse, display, ordering)
│   ├── log_level.rs    — LogLevel enum (parse, display, ordering)
│   ├── errors.rs       — AnalyzerError + ParseError types
│   ├── parser.rs       — parse_log_line() + LogEntry struct
│   ├── statistics.rs   — Statistics aggregator
│   ├── analyzer.rs     — LogAnalyzer (file/directory processing)
│   └── report.rs       — Text table + JSON report generation
└── tests/
    ├── integration_tests.rs
    └── fixtures/
        ├── simple.log
        └── malformed.log
```

## Log Format

```
YYYY-MM-DD HH:MM:SS [LEVEL] component: message
```

**Supported levels:** `TRACE`, `DEBUG`, `INFO`, `WARN`, `ERROR`, `FATAL` (case-insensitive)

**Valid timestamp ranges:**
- Year: 1970–9999
- Month: 1–12
- Day: 1–31
- Hour: 0–23
- Minute: 0–59
- Second: 0–59
