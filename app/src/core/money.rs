//! Money values and number formatting.
//!
//! All amounts are integers in minor units (`i64`): cents for money and
//! grams of metal (2 decimals), 1e-8 for crypto (8 decimals, see
//! `Currency::decimals`). The `*_dec` functions take the decimal places;
//! the plain ones assume 2. Display format is fixed: dot thousands
//! separator, comma decimal separator, symbol after the number
//! (`23.246,50 $`). Decimals are shown only when non-zero, at least two,
//! trailing zeros beyond two trimmed (`0,00512`).

use super::currency::Currency;

pub type Cents = i64;

const MILLION_CENTS: i64 = 1_000_000 * 100;

/// Groups an unsigned integer with dots: `1234567` -> `1.234.567`.
fn group_thousands(mut n: u64) -> String {
    if n == 0 {
        return "0".into();
    }
    let mut parts = Vec::new();
    while n > 0 {
        parts.push(n % 1000);
        n /= 1000;
    }
    let mut out = parts.pop().unwrap().to_string();
    while let Some(p) = parts.pop() {
        out.push('.');
        out.push_str(&format!("{p:03}"));
    }
    out
}

fn scale(dec: u32) -> u64 {
    10u64.pow(dec)
}

/// `,50`, `,00512`, or empty for a zero fraction.
fn fraction(frac: u64, dec: u32) -> String {
    if frac == 0 || dec == 0 {
        return String::new();
    }
    let mut f = format!("{frac:0width$}", width = dec as usize);
    while f.len() > 2 && f.ends_with('0') {
        f.pop();
    }
    format!(",{f}")
}

/// Full number without currency symbol: `1.250,50`, `-23.246`, `0`.
pub fn format_number(cents: Cents) -> String {
    format_number_dec(cents, 2)
}

/// [`format_number`] for an amount with `dec` decimal places.
pub fn format_number_dec(units: Cents, dec: u32) -> String {
    let abs = units.unsigned_abs();
    let sign = if units < 0 { "-" } else { "" };
    format!("{sign}{}{}", group_thousands(abs / scale(dec)), fraction(abs % scale(dec), dec))
}

/// Editing form of an amount with `dec` decimal places: no thousands dots,
/// comma decimal: `1250,50`, `23246`.
pub fn format_plain_dec(units: Cents, dec: u32) -> String {
    let abs = units.unsigned_abs();
    let sign = if units < 0 { "-" } else { "" };
    format!("{sign}{}{}", abs / scale(dec), fraction(abs % scale(dec), dec))
}

/// Re-expresses minor units for a new number of decimals, keeping the shown
/// number (`1,50` stays `1,50`). Fewer decimals round half away from zero.
pub fn rescale(units: Cents, from: u32, to: u32) -> Option<Cents> {
    if to >= from {
        units.checked_mul(10i64.checked_pow(to - from)?)
    } else {
        let d = 10i64.checked_pow(from - to)?;
        let half = d / 2;
        Some(if units >= 0 { (units + half) / d } else { (units - half) / d })
    }
}

/// Number for display in main view / totals: abbreviated from 1 million up
/// (`43,4M`, `1,42B`, `2T`), 3 significant digits, truncated toward zero.
/// Below 1 million it is the full number.
pub fn format_compact_number(cents: Cents) -> String {
    let abs = cents.unsigned_abs();
    if abs < MILLION_CENTS as u64 {
        return format_number(cents);
    }
    let units: [(u64, char); 3] = [
        (1_000_000_000_000 * 100, 'T'),
        (1_000_000_000 * 100, 'B'),
        (1_000_000 * 100, 'M'),
    ];
    let (unit, suffix) = units.into_iter().find(|(u, _)| abs >= *u).unwrap();
    let whole = abs / unit;
    let digits = whole.to_string().len() as u32;
    let decimals = 3u32.saturating_sub(digits);
    let scale = 10u64.pow(decimals);
    // Truncate: integer division drops the rest.
    let frac = ((abs % unit) as u128 * scale as u128 / unit as u128) as u64;
    let mut out = String::new();
    if cents < 0 {
        out.push('-');
    }
    out.push_str(&whole.to_string());
    if decimals > 0 {
        let mut f = format!("{frac:0width$}", width = decimals as usize);
        while f.ends_with('0') {
            f.pop();
        }
        if !f.is_empty() {
            out.push(',');
            out.push_str(&f);
        }
    }
    out.push(suffix);
    out
}

/// Compact number with currency symbol: `1,42B $`.
pub fn format_compact(cents: Cents, currency: Currency) -> String {
    format!("{} {}", format_compact_number(cents), currency.symbol())
}

/// Parses a user-typed amount into minor units with `dec` decimals. Accepts:
/// - `1250`, `1250,5`, `1.250,50` (comma = decimal, dots = thousands)
/// - `12.5`, `12.50` (single dot followed by 1-2 digits = decimal)
/// - `1.250` (dot followed by 3 digits = thousands)
///
/// With more than 2 decimals a single dot is always a decimal point
/// (`0.005` BTC), never thousands. Rejects negatives, more than `dec`
/// decimals, empty input and anything else.
pub fn parse_amount_dec(input: &str, dec: u32) -> Option<Cents> {
    let s: String = input.trim().chars().filter(|c| !c.is_whitespace()).collect();
    if s.is_empty() {
        return None;
    }
    if !s.chars().all(|c| c.is_ascii_digit() || c == '.' || c == ',') {
        return None;
    }
    let (int_part, frac_part) = if let Some((i, f)) = s.split_once(',') {
        if f.contains(',') || f.contains('.') {
            return None;
        }
        (i.replace('.', ""), f.to_string())
    } else if s.matches('.').count() == 1 {
        let (i, f) = s.split_once('.').unwrap();
        if f.len() <= 2 || dec > 2 {
            (i.to_string(), f.to_string())
        } else if f.len() == 3 {
            (format!("{i}{f}"), String::new())
        } else {
            return None;
        }
    } else {
        // Zero or many dots: all thousands separators. Validate grouping.
        let groups: Vec<&str> = s.split('.').collect();
        if groups.len() > 1 && groups[1..].iter().any(|g| g.len() != 3) {
            return None;
        }
        (s.replace('.', ""), String::new())
    };
    if int_part.is_empty() && frac_part.is_empty() {
        return None;
    }
    if frac_part.len() > dec as usize {
        return None;
    }
    let whole: i64 = if int_part.is_empty() { 0 } else { int_part.parse().ok()? };
    let frac: i64 = if frac_part.is_empty() {
        0
    } else {
        let padded = format!("{frac_part:0<width$}", width = dec as usize);
        padded.parse().ok()?
    };
    whole.checked_mul(scale(dec) as i64)?.checked_add(frac)
}

/// Converts a spreadsheet float to minor units with `dec` decimals.
pub fn units_from_f64(v: f64, dec: u32) -> Option<Cents> {
    if !v.is_finite() || v < 0.0 {
        return None;
    }
    let c = (v * scale(dec) as f64).round();
    if c > i64::MAX as f64 {
        return None;
    }
    Some(c as i64)
}

pub fn units_to_f64(units: Cents, dec: u32) -> f64 {
    units as f64 / scale(dec) as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn format_plain(cents: Cents) -> String {
        format_plain_dec(cents, 2)
    }

    fn parse_amount(input: &str) -> Option<Cents> {
        parse_amount_dec(input, 2)
    }

    fn cents_from_f64(v: f64) -> Option<Cents> {
        units_from_f64(v, 2)
    }

    #[test]
    fn full_format() {
        assert_eq!(format_number(0), "0");
        assert_eq!(format_number(2_324_600), "23.246");
        assert_eq!(format_number(2_324_650), "23.246,50");
        assert_eq!(format_number(-3_253_400), "-32.534");
        assert_eq!(format_number(5), "0,05");
        assert_eq!(format_number(99_999_999), "999.999,99");
    }

    #[test]
    fn plain_format_has_no_dots() {
        assert_eq!(format_plain(125_050), "1250,50");
        assert_eq!(format_plain(2_324_600), "23246");
        assert_eq!(format_plain(5), "0,05");
        assert_eq!(format_plain(0), "0");
        for c in [0, 5, 125_050, 2_324_600, 99_999_999] {
            assert_eq!(parse_amount(&format_plain(c)), Some(c));
        }
    }

    #[test]
    fn compact_below_million_is_full() {
        assert_eq!(format_compact_number(99_999_999), "999.999,99");
        assert_eq!(format_compact_number(-50_000_000), "-500.000");
    }

    #[test]
    fn compact_abbreviates() {
        assert_eq!(format_compact_number(1_420_000_000 * 100), "1,42B");
        assert_eq!(format_compact_number(43_400_000 * 100), "43,4M");
        assert_eq!(format_compact_number(1_000_000 * 100), "1M");
        assert_eq!(format_compact_number(-2_500_000 * 100), "-2,5M");
        assert_eq!(format_compact_number(3_000_000_000_000 * 100), "3T");
        assert_eq!(format_compact(1_420_000_000 * 100, Currency::Usd), "1,42B $");
    }

    #[test]
    fn compact_truncates_never_rounds_up() {
        assert_eq!(format_compact_number(999_999_999 * 100), "999M");
        assert_eq!(format_compact_number(1_429_999_999 * 100), "1,42B");
        assert_eq!(format_compact_number(43_499_999 * 100 + 99), "43,4M");
    }

    #[test]
    fn parse_amounts() {
        assert_eq!(parse_amount("1250"), Some(125_000));
        assert_eq!(parse_amount("1250,5"), Some(125_050));
        assert_eq!(parse_amount("1.250,50"), Some(125_050));
        assert_eq!(parse_amount("12.5"), Some(1_250));
        assert_eq!(parse_amount("12.50"), Some(1_250));
        assert_eq!(parse_amount("1.250"), Some(125_000));
        assert_eq!(parse_amount("1.250.000"), Some(125_000_000));
        assert_eq!(parse_amount(" 0 "), Some(0));
        assert_eq!(parse_amount(",5"), Some(50));
        assert_eq!(parse_amount(""), None);
        assert_eq!(parse_amount("-5"), None);
        assert_eq!(parse_amount("1,234"), None);
        assert_eq!(parse_amount("abc"), None);
        assert_eq!(parse_amount("1.25.0"), None);
        assert_eq!(parse_amount(","), None);
    }

    #[test]
    fn eight_decimals() {
        assert_eq!(format_number_dec(512_000, 8), "0,00512");
        assert_eq!(format_number_dec(150_000_000, 8), "1,50");
        assert_eq!(format_number_dec(123_456_789_012, 8), "1.234,56789012");
        assert_eq!(format_number_dec(100_000_000, 8), "1");
        assert_eq!(format_plain_dec(512_000, 8), "0,00512");
        assert_eq!(parse_amount_dec("0,00512", 8), Some(512_000));
        assert_eq!(parse_amount_dec("0.005", 8), Some(500_000));
        assert_eq!(parse_amount_dec("1.5", 8), Some(150_000_000));
        assert_eq!(parse_amount_dec("0,123456789", 8), None);
        assert_eq!(parse_amount_dec("1.234,5", 8), Some(123_450_000_000));
        for c in [0, 1, 512_000, 150_000_000, 123_456_789_012] {
            assert_eq!(parse_amount_dec(&format_plain_dec(c, 8), 8), Some(c));
        }
        assert_eq!(units_from_f64(0.00512, 8), Some(512_000));
        assert_eq!(units_to_f64(512_000, 8), 0.00512);
    }

    #[test]
    fn rescale_keeps_shown_number() {
        assert_eq!(rescale(150, 2, 8), Some(150_000_000));
        assert_eq!(rescale(150_000_000, 8, 2), Some(150));
        assert_eq!(rescale(123_456_789, 8, 2), Some(123)); // 1,23456789 -> 1,23
        assert_eq!(rescale(123_500_000, 8, 2), Some(124)); // half up
        assert_eq!(rescale(150, 2, 2), Some(150));
        assert_eq!(rescale(i64::MAX, 2, 8), None);
    }

    #[test]
    fn float_conversion() {
        assert_eq!(cents_from_f64(12.345), Some(1_235));
        assert_eq!(cents_from_f64(0.1 + 0.2), Some(30));
        assert_eq!(cents_from_f64(-1.0), None);
        assert_eq!(cents_from_f64(f64::NAN), None);
    }
}
