//! Date parsing/formatting and Excel serial conversion.

use chrono::{Datelike, NaiveDate};

/// `YYYY-MM-DD`: dates in the price cache, price links and API answers.
pub const ISO_DATE: &str = "%Y-%m-%d";

/// Parses `DD.MM.YYYY` (1-2 digit day/month accepted).
pub fn parse_date(input: &str) -> Option<NaiveDate> {
    let mut parts = input.trim().split('.');
    let d: u32 = parts.next()?.trim().parse().ok()?;
    let m: u32 = parts.next()?.trim().parse().ok()?;
    let y_str = parts.next()?.trim();
    if parts.next().is_some() || y_str.len() != 4 {
        return None;
    }
    let y: i32 = y_str.parse().ok()?;
    NaiveDate::from_ymd_opt(y, m, d)
}

/// Full edit format: `DD.MM.YYYY`.
pub fn format_date_full(date: NaiveDate) -> String {
    date.format("%d.%m.%Y").to_string()
}

/// Display format: `DD.MM` in the current year, otherwise `DD.MM.YYYY`.
pub fn format_date_short(date: NaiveDate, today: NaiveDate) -> String {
    if date.year() == today.year() {
        date.format("%d.%m").to_string()
    } else {
        format_date_full(date)
    }
}

fn excel_epoch() -> NaiveDate {
    NaiveDate::from_ymd_opt(1899, 12, 30).unwrap()
}

/// Excel serial (1900 date system) to date. Time-of-day fraction is dropped.
pub fn date_from_excel_serial(serial: f64) -> Option<NaiveDate> {
    if !serial.is_finite() || serial < 1.0 {
        return None;
    }
    excel_epoch().checked_add_days(chrono::Days::new(serial.floor() as u64))
}

pub fn date_to_excel_serial(date: NaiveDate) -> f64 {
    (date - excel_epoch()).num_days() as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    #[test]
    fn parse() {
        assert_eq!(parse_date("09.11.2026"), Some(d(2026, 11, 9)));
        assert_eq!(parse_date("9.1.2026"), Some(d(2026, 1, 9)));
        assert_eq!(parse_date("31.02.2026"), None);
        assert_eq!(parse_date("09.11.26"), None);
        assert_eq!(parse_date("2026-11-09"), None);
        assert_eq!(parse_date(""), None);
        assert_eq!(parse_date("01.01.2026.1"), None);
    }

    #[test]
    fn format() {
        let today = d(2026, 11, 9);
        assert_eq!(format_date_full(d(2026, 1, 5)), "05.01.2026");
        assert_eq!(format_date_short(d(2026, 1, 5), today), "05.01");
        assert_eq!(format_date_short(d(2025, 1, 5), today), "05.01.2025");
    }

    #[test]
    fn excel_serial_roundtrip() {
        assert_eq!(date_from_excel_serial(45_000.0), Some(d(2023, 3, 15)));
        assert_eq!(date_from_excel_serial(45_000.75), Some(d(2023, 3, 15)));
        assert_eq!(date_to_excel_serial(d(2023, 3, 15)), 45_000.0);
        assert_eq!(date_from_excel_serial(0.0), None);
    }
}
