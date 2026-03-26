use crate::errors::ParseError;
use crate::log_level::LogLevel;
use crate::statistics::Statistics;

/// Generate a formatted text table report
pub fn format_text_report(stats: &Statistics, parse_errors: &[ParseError]) -> String {
    let mut out = String::new();

    out.push_str("================== LOG ANALYSIS REPORT ==================\n");

    // Period
    match (&stats.first_entry, &stats.last_entry) {
        (Some(first), Some(last)) => {
            out.push_str(&format!("Period: {} to {}\n", first, last));
        }
        _ => {
            out.push_str("Period: (no entries)\n");
        }
    }

    out.push('\n');
    out.push_str("SUMMARY\n");
    out.push_str("-------\n");
    out.push_str(&format!(
        "Total Entries:    {}\n",
        format_number(stats.total_entries)
    ));
    out.push_str(&format!(
        "Error Rate:       {:.1}%\n",
        stats.error_rate * 100.0
    ));

    // Peak hour
    match stats.peak_hour {
        Some(h) => {
            let count = stats.entries_by_hour.get(&h).copied().unwrap_or(0);
            out.push_str(&format!(
                "Peak Hour:        {:02}:00 ({} entries)\n",
                h,
                format_number(count)
            ));
        }
        None => out.push_str("Peak Hour:        (none)\n"),
    }

    // Most active component
    match &stats.most_active_component {
        Some(comp) => {
            let count = stats.entries_by_component.get(comp).copied().unwrap_or(0);
            out.push_str(&format!(
                "Most Active:      {} ({} entries)\n",
                comp,
                format_number(count)
            ));
        }
        None => out.push_str("Most Active:      (none)\n"),
    }

    // BY LOG LEVEL
    out.push('\n');
    out.push_str("BY LOG LEVEL\n");
    out.push_str("------------\n");

    let level_order = [
        LogLevel::Trace,
        LogLevel::Debug,
        LogLevel::Info,
        LogLevel::Warn,
        LogLevel::Error,
        LogLevel::Fatal,
    ];

    for level in &level_order {
        let count = stats.entries_by_level.get(level).copied().unwrap_or(0);
        let pct = if stats.total_entries > 0 {
            count as f64 / stats.total_entries as f64 * 100.0
        } else {
            0.0
        };
        out.push_str(&format!(
            "{:<10} {:>6}   ({:.1}%)\n",
            level.to_string(),
            format_number(count),
            pct
        ));
    }

    // BY COMPONENT
    out.push('\n');
    out.push_str("BY COMPONENT\n");
    out.push_str("------------\n");

    // Sort components by count descending
    let mut components: Vec<(&String, &usize)> = stats.entries_by_component.iter().collect();
    components.sort_by(|a, b| b.1.cmp(a.1));

    for (comp, count) in &components {
        let pct = if stats.total_entries > 0 {
            **count as f64 / stats.total_entries as f64 * 100.0
        } else {
            0.0
        };
        out.push_str(&format!(
            "{:<16} {:>6}   ({:.1}%)\n",
            comp,
            format_number(**count),
            pct
        ));
    }

    out.push('\n');
    out.push_str(&format!(
        "Parse Errors: {} lines skipped\n",
        parse_errors.len()
    ));
    out.push_str("=========================================================\n");

    out
}

/// Generate a JSON report string
pub fn format_json_report(stats: &Statistics, parse_errors: &[ParseError]) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str(&format!("  \"total_entries\": {},\n", stats.total_entries));
    out.push_str(&format!("  \"error_rate\": {:.3},\n", stats.error_rate));

    match stats.peak_hour {
        Some(h) => out.push_str(&format!("  \"peak_hour\": {},\n", h)),
        None => out.push_str("  \"peak_hour\": null,\n"),
    }

    match &stats.most_active_component {
        Some(c) => out.push_str(&format!("  \"most_active_component\": \"{}\",\n", c)),
        None => out.push_str("  \"most_active_component\": null,\n"),
    }

    // entries_by_level
    out.push_str("  \"entries_by_level\": {\n");
    let level_order = [
        LogLevel::Trace,
        LogLevel::Debug,
        LogLevel::Info,
        LogLevel::Warn,
        LogLevel::Error,
        LogLevel::Fatal,
    ];
    let level_entries: Vec<String> = level_order
        .iter()
        .map(|lvl| {
            let count = stats.entries_by_level.get(lvl).copied().unwrap_or(0);
            format!("    \"{}\": {}", lvl, count)
        })
        .collect();
    out.push_str(&level_entries.join(",\n"));
    out.push_str("\n  },\n");

    // entries_by_component
    out.push_str("  \"entries_by_component\": {\n");
    let mut components: Vec<(&String, &usize)> = stats.entries_by_component.iter().collect();
    components.sort_by(|a, b| b.1.cmp(a.1));
    let comp_entries: Vec<String> = components
        .iter()
        .map(|(name, count)| format!("    \"{}\": {}", name, count))
        .collect();
    out.push_str(&comp_entries.join(",\n"));
    out.push_str("\n  },\n");

    // period
    out.push_str("  \"period\": {\n");
    match (&stats.first_entry, &stats.last_entry) {
        (Some(f), Some(l)) => {
            out.push_str(&format!("    \"start\": \"{}\",\n", f));
            out.push_str(&format!("    \"end\": \"{}\"\n", l));
        }
        _ => {
            out.push_str("    \"start\": null,\n");
            out.push_str("    \"end\": null\n");
        }
    }
    out.push_str("  },\n");

    out.push_str(&format!("  \"parse_errors\": {}\n", parse_errors.len()));
    out.push_str("}\n");

    out
}

/// Format a usize with comma separators
fn format_number(n: usize) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, ch) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(ch);
    }
    result.chars().rev().collect()
}
