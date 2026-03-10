//! Date utilities

use chrono::{Datelike, Local, NaiveDate};

/// Parse a date string with smart defaults.
///
/// Supported formats:
/// - Empty string -> Today
/// - "DD" -> Current year, Current month, Day
/// - "MM-DD" or "MM/DD" -> Current year, Month, Day
/// - "YYYY-MM-DD" or "YYYY/MM/DD" -> Full date
pub fn parse_smart_date(input: &str) -> Option<NaiveDate> {
    let input = input.trim();
    if input.is_empty() {
        return Some(Local::now().date_naive());
    }

    let now = Local::now().date_naive();
    let parts: Vec<&str> = input.split(|c| c == '-' || c == '/').collect();

    match parts.len() {
        1 => {
            // Day only: "5" -> current_year-current_month-05
            if let Ok(day) = parts[0].parse::<u32>() {
                NaiveDate::from_ymd_opt(now.year(), now.month(), day)
            } else {
                None
            }
        }
        2 => {
            // Month-Day: "3-5" -> current_year-03-05
            if let (Ok(month), Ok(day)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                NaiveDate::from_ymd_opt(now.year(), month, day)
            } else {
                None
            }
        }
        3 => {
            // Year-Month-Day: "2023-3-5"
            if let (Ok(year), Ok(month), Ok(day)) = (
                parts[0].parse::<i32>(),
                parts[1].parse::<u32>(),
                parts[2].parse::<u32>(),
            ) {
                NaiveDate::from_ymd_opt(year, month, day)
            } else {
                None
            }
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Datelike, Local};

    #[test]
    fn test_parse_smart_date() {
        let now = Local::now().date_naive();
        let current_year = now.year();
        let current_month = now.month();

        // Empty -> Today
        assert_eq!(parse_smart_date(""), Some(now));

        // Day only -> Current month/year
        assert_eq!(
            parse_smart_date("15"),
            NaiveDate::from_ymd_opt(current_year, current_month, 15)
        );

        // Month-Day -> Current year
        assert_eq!(
            parse_smart_date("12-25"),
            NaiveDate::from_ymd_opt(current_year, 12, 25)
        );
        assert_eq!(
            parse_smart_date("12/25"),
            NaiveDate::from_ymd_opt(current_year, 12, 25)
        );

        // Full date
        assert_eq!(
            parse_smart_date("2025-01-01"),
            NaiveDate::from_ymd_opt(2025, 1, 1)
        );
        assert_eq!(
            parse_smart_date("2025/01/01"),
            NaiveDate::from_ymd_opt(2025, 1, 1)
        );

        // Invalid
        assert_eq!(parse_smart_date("invalid"), None);
        assert_eq!(parse_smart_date("13-45"), None); // Invalid month/day
    }
}
