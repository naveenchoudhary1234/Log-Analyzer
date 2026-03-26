use std::path::Path;
use std::process;

use log_analyzer::analyzer::LogAnalyzer;
use log_analyzer::log_level::LogLevel;
use log_analyzer::report::{format_json_report, format_text_report};

fn print_usage() {
    eprintln!("Usage: log-analyzer [OPTIONS] <FILE|DIR> [FILE|DIR ...]");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  --format <text|json>   Output format (default: text)");
    eprintln!(
        "  --min-level <level>    Minimum log level to include (trace/debug/info/warn/error/fatal)"
    );
    eprintln!("  --component <name>     Filter by component name");
    eprintln!("  --help                 Show this help message");
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() || args.iter().any(|a| a == "--help") {
        print_usage();
        process::exit(0);
    }

    
    let mut format = "text".to_string();
    let mut min_level: Option<LogLevel> = None;
    let mut component_filter: Option<String> = None;
    let mut paths: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--format" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("Error: --format requires a value (text or json)");
                    process::exit(1);
                }
                format = args[i].to_lowercase();
                if format != "text" && format != "json" {
                    eprintln!("Error: --format must be 'text' or 'json'");
                    process::exit(1);
                }
            }
            "--min-level" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("Error: --min-level requires a value");
                    process::exit(1);
                }
                match args[i].parse::<LogLevel>() {
                    Ok(level) => min_level = Some(level),
                    Err(_) => {
                        eprintln!(
                            "Error: '{}' is not a valid log level. Use: trace, debug, info, warn, error, fatal",
                            args[i]
                        );
                        process::exit(1);
                    }
                }
            }
            "--component" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("Error: --component requires a value");
                    process::exit(1);
                }
                component_filter = Some(args[i].clone());
            }
            arg if arg.starts_with("--") => {
                eprintln!("Error: Unknown option '{}'. Use --help for usage.", arg);
                process::exit(1);
            }
            _ => {
                paths.push(args[i].clone());
            }
        }
        i += 1;
    }

    if paths.is_empty() {
        eprintln!("Error: No input files or directories specified.");
        print_usage();
        process::exit(1);
    }

    // --- Process files ---
    let mut analyzer = LogAnalyzer::new();
    let mut any_success = false;

    for path_str in &paths {
        let path = Path::new(path_str);

        if path.is_dir() {
            match analyzer.process_directory(path) {
                Ok(n) => {
                    eprintln!("Processed directory '{}': {} entries", path_str, n);
                    any_success = true;
                }
                Err(e) => {
                    eprintln!("Error processing directory '{}': {}", path_str, e);
                }
            }
        } else {
            match analyzer.process_file(path) {
                Ok(n) => {
                    eprintln!("Processed file '{}': {} entries", path_str, n);
                    any_success = true;
                }
                Err(e) => {
                    eprintln!("Error processing file '{}': {}", path_str, e);
                }
            }
        }
    }

    if !any_success {
        eprintln!("No files were successfully processed.");
        process::exit(1);
    }

    // --- Apply filters ---
    let all_entries = analyzer.entries();
    let filtered: Vec<_> = all_entries
        .iter()
        .filter(|e| {
            if let Some(min) = min_level {
                if e.level < min {
                    return false;
                }
            }
            if let Some(ref comp) = component_filter {
                if !e.component.eq_ignore_ascii_case(comp) {
                    return false;
                }
            }
            true
        })
        .cloned()
        .collect();

    // Rebuild statistics from filtered entries
    let stats = log_analyzer::statistics::Statistics::from_entries(&filtered);
    let parse_errors = analyzer.parse_errors();

    // --- Output ---
    if format == "json" {
        print!("{}", format_json_report(&stats, parse_errors));
    } else {
        print!("{}", format_text_report(&stats, parse_errors));
    }
}
