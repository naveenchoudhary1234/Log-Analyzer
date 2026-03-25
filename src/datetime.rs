use std::fmt;
use std::str::FromStr;

/// Custom DateTime struct
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateTime {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

impl DateTime {
    /// Convert to total seconds
    fn to_seconds(&self) -> u64 {
        let y = self.year as u64;
        let m = self.month as u64;
        let d = self.day as u64;
        let h = self.hour as u64;
        let min = self.minute as u64;
        let s = self.second as u64;

        y * 365 * 24 * 3600 + m * 30 * 24 * 3600 + d * 24 * 3600 + h * 3600 + min * 60 + s
    }
}

impl PartialOrd for DateTime {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DateTime {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.to_seconds().cmp(&other.to_seconds())
    }
}

/// Display: "YYYY-MM-DD HH:MM:SS"
impl fmt::Display for DateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            self.year, self.month, self.day, self.hour, self.minute, self.second
        )
    }
}

/// Parse "YYYY-MM-DD HH:MM:SS"
impl FromStr for DateTime {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        // Must be exactly "YYYY-MM-DD HH:MM:SS" = 19 chars
        if s.len() != 19 {
            return Err(format!(
                "DateTime string must be 19 characters, got {}",
                s.len()
            ));
        }

        // Check separators
        if &s[4..5] != "-"
            || &s[7..8] != "-"
            || &s[10..11] != " "
            || &s[13..14] != ":"
            || &s[16..17] != ":"
        {
            return Err("DateTime format must be YYYY-MM-DD HH:MM:SS".to_string());
        }

        let parse_num = |slice: &str, field: &str| -> Result<u32, String> {
            slice
                .parse::<u32>()
                .map_err(|_| format!("{} '{}' is not a valid number", field, slice))
        };

        let year = parse_num(&s[0..4], "Year")? as u16;
        let month = parse_num(&s[5..7], "Month")? as u8;
        let day = parse_num(&s[8..10], "Day")? as u8;
        let hour = parse_num(&s[11..13], "Hour")? as u8;
        let minute = parse_num(&s[14..16], "Minute")? as u8;
        let second = parse_num(&s[17..19], "Second")? as u8;

        if !(1970..=9999).contains(&year) {
            return Err(format!("Year {} out of range (1970-9999)", year));
        }
        if !(1..=12).contains(&month) {
            return Err(format!("Month {} out of range (1-12)", month));
        }
        if !(1..=31).contains(&day) {
            return Err(format!("Day {} out of range (1-31)", day));
        }
        if hour > 23 {
            return Err(format!("Hour {} out of range (0-23)", hour));
        }
        if minute > 59 {
            return Err(format!("Minute {} out of range (0-59)", minute));
        }
        if second > 59 {
            return Err(format!("Second {} out of range (0-59)", second));
        }

        Ok(DateTime {
            year,
            month,
            day,
            hour,
            minute,
            second,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_datetime() {
        let dt: DateTime = "2024-01-15 10:23:45".parse().unwrap();
        assert_eq!(dt.year, 2024);
        assert_eq!(dt.month, 1);
        assert_eq!(dt.day, 15);
        assert_eq!(dt.hour, 10);
        assert_eq!(dt.minute, 23);
        assert_eq!(dt.second, 45);
    }

    #[test]
    fn test_reject_invalid_month_zero() {
        let result: Result<DateTime, _> = "2024-00-15 10:23:45".parse();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Month"));
    }

    #[test]
    fn test_reject_invalid_month_thirteen() {
        let result: Result<DateTime, _> = "2024-13-15 10:23:45".parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_reject_invalid_hour_24() {
        let result: Result<DateTime, _> = "2024-01-15 24:00:00".parse();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Hour"));
    }

    #[test]
    fn test_reject_invalid_hour_25() {
        let result: Result<DateTime, _> = "2024-01-15 25:00:00".parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_compare_datetimes() {
        let earlier: DateTime = "2024-01-15 10:00:00".parse().unwrap();
        let later: DateTime = "2024-01-15 11:00:00".parse().unwrap();
        assert!(earlier < later);
        assert!(later > earlier);
        assert_eq!(earlier, earlier.clone());
    }

    #[test]
    fn test_display_format() {
        let dt: DateTime = "2024-01-15 10:23:45".parse().unwrap();
        assert_eq!(dt.to_string(), "2024-01-15 10:23:45");
    }
}
