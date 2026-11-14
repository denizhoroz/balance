//! Pure text and sizing helpers for display values.

use gpui_kit::Hsla;

use crate::core::currency::Currency;
use crate::core::money::{Cents, format_compact};
use crate::core::rates::{Bound, Total};
use crate::ui::palette::Palette;

const BIG_SIZES: [f32; 9] = [72., 64., 56., 48., 40., 34., 28., 24., 20.];

/// Picks the largest font size whose estimated width fits. Inter Thin digits
/// average ~0.58em; this is a deliberate overestimate so text never clips.
pub fn big_font_size(text: &str, max_width: f32) -> f32 {
    let chars = text.chars().count() as f32;
    BIG_SIZES
        .into_iter()
        .find(|s| chars * s * 0.6 <= max_width)
        .unwrap_or(*BIG_SIZES.last().unwrap())
}

/// Compact amount in the display currency. While some rates are still
/// missing the sum of the rest is prefixed `>` (true value is bigger) or `<`;
/// `—` when nothing could be converted at all.
pub fn format_total(total: Total, currency: Currency) -> String {
    let text = format_compact(total.value, currency);
    match total.bound {
        Bound::Exact => text,
        _ if total.value == 0 => "—".to_string(),
        Bound::AtLeast => format!(">{text}"),
        Bound::AtMost => format!("<{text}"),
        Bound::Unknown => "—".to_string(),
    }
}

/// `format_total` plus the color for a signed headline number: muted when
/// unavailable, otherwise green / red / yellow by sign.
pub fn signed_total(p: &Palette, total: Total, currency: Currency) -> (String, Hsla) {
    let text = format_total(total, currency);
    let color = if text == "—" { p.muted } else { p.signed(total.value) };
    (text, color)
}

/// Signed compact profit, `+1.250 $` / `-300 $` / `0 $`; `—` when unknown.
pub fn format_profit(profit: Option<Cents>, currency: Currency) -> String {
    match profit {
        None => "—".to_string(),
        Some(v) if v > 0 => format!("+{}", format_compact(v, currency)),
        Some(v) => format_compact(v, currency),
    }
}

/// `38%`; below 1% but non-zero `<1%`.
pub fn format_share(value: Cents, total: Cents) -> String {
    if total <= 0 {
        return "0%".into();
    }
    let pct = value as i128 * 100 / total as i128;
    if pct == 0 && value > 0 { "<1%".into() } else { format!("{pct}%") }
}

/// `text`, or `placeholder` when empty.
pub fn or_placeholder(text: String, placeholder: &str) -> String {
    if text.is_empty() { placeholder.to_string() } else { text }
}
