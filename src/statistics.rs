use crate::datetime::DateTime;
use crate::log_level::LogLevel;
use crate::parser::LogEntry;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Statistics {
    pub total_entries: usize,
    pub entries_by_level: HashMap<LogLevel, usize>,
    pub entries_by_component: HashMap<String, usize>,
    pub entries_by_hour: HashMap<u8, usize>,
    pub error_count: usize,
    pub error_rate: f64,
    pub most_active_component: Option<String>,
    pub peak_hour: Option<u8>,
    pub first_entry: Option<DateTime>,
    pub last_entry: Option<DateTime>,
}

impl Statistics {
    pub fn from_entries(entries: &[LogEntry]) -> Self {
        if entries.is_empty() {
            return Statistics {
                total_entries: 0,
                entries_by_level: HashMap::new(),
                entries_by_component: HashMap::new(),
                entries_by_hour: HashMap::new(),
                error_count: 0,
                error_rate: 0.0,
                most_active_component: None,
                peak_hour: None,
                first_entry: None,
                last_entry: None,
            };
        }

        let mut entries_by_level: HashMap<LogLevel, usize> = HashMap::new();
        let mut entries_by_component: HashMap<String, usize> = HashMap::new();
        let mut entries_by_hour: HashMap<u8, usize> = HashMap::new();
        let mut first_entry: Option<DateTime> = None;
        let mut last_entry: Option<DateTime> = None;

        for entry in entries {
            // Count by level
            *entries_by_level.entry(entry.level).or_insert(0) += 1;

            // Count by component
            *entries_by_component
                .entry(entry.component.clone())
                .or_insert(0) += 1;

            // Count by hour
            *entries_by_hour.entry(entry.timestamp.hour).or_insert(0) += 1;

            // Track first / last timestamp
            match &first_entry {
                None => first_entry = Some(entry.timestamp.clone()),
                Some(f) if entry.timestamp < *f => first_entry = Some(entry.timestamp.clone()),
                _ => {}
            }
            match &last_entry {
                None => last_entry = Some(entry.timestamp.clone()),
                Some(l) if entry.timestamp > *l => last_entry = Some(entry.timestamp.clone()),
                _ => {}
            }
        }

        let total_entries = entries.len();

        // error_count = ERROR + FATAL
        let error_count = entries_by_level.get(&LogLevel::Error).copied().unwrap_or(0)
            + entries_by_level.get(&LogLevel::Fatal).copied().unwrap_or(0);

        let error_rate = if total_entries > 0 {
            error_count as f64 / total_entries as f64
        } else {
            0.0
        };

        // most_active_component
        let most_active_component = entries_by_component
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(name, _)| name.clone());

        // peak_hour
        let peak_hour = entries_by_hour
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(hour, _)| *hour);

        Statistics {
            total_entries,
            entries_by_level,
            entries_by_component,
            entries_by_hour,
            error_count,
            error_rate,
            most_active_component,
            peak_hour,
            first_entry,
            last_entry,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::datetime::DateTime;
    use crate::log_level::LogLevel;
    use crate::parser::LogEntry;
    use std::path::PathBuf;

    fn make_entry(ts: &str, level: LogLevel, component: &str) -> LogEntry {
        LogEntry {
            timestamp: ts.parse::<DateTime>().unwrap(),
            level,
            component: component.to_string(),
            message: "test".to_string(),
            source_file: PathBuf::from("test.log"),
        }
    }

    #[test]
    fn test_empty_input() {
        let stats = Statistics::from_entries(&[]);
        assert_eq!(stats.total_entries, 0);
        assert_eq!(stats.error_rate, 0.0);
        assert!(stats.most_active_component.is_none());
        assert!(stats.peak_hour.is_none());
    }

    #[test]
    fn test_basic_statistics() {
        let entries = vec![
            make_entry("2024-01-15 10:00:00", LogLevel::Info, "network"),
            make_entry("2024-01-15 10:00:01", LogLevel::Error, "storage"),
            make_entry("2024-01-15 10:00:02", LogLevel::Fatal, "kernel"),
            make_entry("2024-01-15 11:00:00", LogLevel::Info, "network"),
            make_entry("2024-01-15 11:00:01", LogLevel::Info, "network"),
        ];

        let stats = Statistics::from_entries(&entries);
        assert_eq!(stats.total_entries, 5);
        assert_eq!(stats.error_count, 2);
        assert!((stats.error_rate - 0.4).abs() < 1e-9);
        assert_eq!(stats.most_active_component, Some("network".to_string()));
    }

    #[test]
    fn test_correct_percentages() {
        let entries = vec![
            make_entry("2024-01-15 10:00:00", LogLevel::Error, "a"),
            make_entry("2024-01-15 10:00:01", LogLevel::Error, "a"),
            make_entry("2024-01-15 10:00:02", LogLevel::Info, "a"),
            make_entry("2024-01-15 10:00:03", LogLevel::Info, "a"),
        ];
        let stats = Statistics::from_entries(&entries);
        assert!((stats.error_rate - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_peak_hour() {
        let entries = vec![
            make_entry("2024-01-15 10:00:00", LogLevel::Info, "a"),
            make_entry("2024-01-15 10:00:01", LogLevel::Info, "a"),
            make_entry("2024-01-15 11:00:00", LogLevel::Info, "a"),
        ];
        let stats = Statistics::from_entries(&entries);
        assert_eq!(stats.peak_hour, Some(10));
    }

    #[test]
    fn test_first_and_last_entry() {
        let entries = vec![
            make_entry("2024-01-15 12:00:00", LogLevel::Info, "a"),
            make_entry("2024-01-15 08:00:00", LogLevel::Info, "a"),
            make_entry("2024-01-15 20:00:00", LogLevel::Info, "a"),
        ];
        let stats = Statistics::from_entries(&entries);
        assert_eq!(
            stats.first_entry.unwrap().to_string(),
            "2024-01-15 08:00:00"
        );
        assert_eq!(stats.last_entry.unwrap().to_string(), "2024-01-15 20:00:00");
    }
}
