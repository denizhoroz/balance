//! Assets: dated value snapshots of holdings, grouped into categories.
//!
//! A holding is identified by its name. Its current value is its latest
//! snapshot dated on or before the day asked about (ties: the later row in
//! the list wins). Assets are independent of the income/expense balance.
//! Values are converted to the display currency; a missing rate makes the
//! whole result unavailable (`None`) rather than silently partial.

use std::collections::HashMap;

use chrono::NaiveDate;

use super::currency::Currency;
use super::money::Cents;
use super::rates::Fx;

/// One recorded value of a holding on a date.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    pub name: String,
    pub category: String,
    pub value: Cents,
    pub currency: Currency,
    pub date: NaiveDate,
}

/// Indices of the snapshots that hold each holding's value on `day`.
pub fn current(snapshots: &[Snapshot], day: NaiveDate) -> Vec<usize> {
    let mut latest: HashMap<&str, usize> = HashMap::new();
    for (i, s) in snapshots.iter().enumerate() {
        if s.date > day {
            continue;
        }
        match latest.get(s.name.as_str()) {
            Some(&j) if snapshots[j].date > s.date => {}
            _ => {
                latest.insert(&s.name, i);
            }
        }
    }
    let mut out: Vec<usize> = latest.into_values().collect();
    out.sort_unstable();
    out
}

/// Categories in order of their first snapshot (earliest date; same date:
/// earlier row), whatever order `snapshots` is in. A new category goes
/// last, so earlier ones keep their place (and their ring color).
pub fn category_order(snapshots: &[Snapshot]) -> Vec<String> {
    let mut first: HashMap<&str, (NaiveDate, usize)> = HashMap::new();
    for (i, s) in snapshots.iter().enumerate() {
        let e = first.entry(&s.category).or_insert((s.date, i));
        *e = (*e).min((s.date, i));
    }
    let mut order: Vec<(&str, (NaiveDate, usize))> = first.into_iter().collect();
    order.sort_by_key(|(_, at)| *at);
    order.into_iter().map(|(c, _)| c.to_string()).collect()
}

/// Value per category on `day` in the display currency, largest first
/// (ties by name). Zero categories are kept; the caller decides what to draw.
pub fn categories(snapshots: &[Snapshot], day: NaiveDate, fx: Fx) -> Option<Vec<(String, Cents)>> {
    let mut sums: HashMap<String, Cents> = HashMap::new();
    for i in current(snapshots, day) {
        let s = &snapshots[i];
        *sums.entry(s.category.clone()).or_default() += fx.convert(s.value, s.currency)?;
    }
    let mut out: Vec<(String, Cents)> = sums.into_iter().collect();
    out.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    Some(out)
}

/// Total of holdings on `day` in the display currency.
pub fn holdings_total(snapshots: &[Snapshot], day: NaiveDate, fx: Fx) -> Option<Cents> {
    current(snapshots, day)
        .into_iter()
        .map(|i| fx.convert(snapshots[i].value, snapshots[i].currency))
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    use crate::core::rates::Rates;

    fn snap(name: &str, category: &str, value: Cents, date: NaiveDate) -> Snapshot {
        Snapshot { name: name.into(), category: category.into(), value, currency: Currency::Usd, date }
    }

    fn usd() -> Fx<'static> {
        Fx::new(Currency::Usd, None)
    }

    #[test]
    fn latest_snapshot_wins_and_future_ignored() {
        let s = vec![
            snap("ETF", "Stocks", 1_000, d(2026, 1, 1)),
            snap("ETF", "Stocks", 1_500, d(2026, 6, 1)),
            snap("ETF", "Stocks", 9_999, d(2026, 12, 1)), // future
            snap("Gold", "Metals", 700, d(2026, 3, 1)),
        ];
        let today = d(2026, 11, 9);
        assert_eq!(holdings_total(&s, today, usd()), Some(2_200));
        assert_eq!(holdings_total(&s, d(2026, 2, 1), usd()), Some(1_000));
        assert_eq!(holdings_total(&s, d(2025, 1, 1), usd()), Some(0));
    }

    #[test]
    fn same_date_later_row_wins() {
        let s = vec![snap("A", "X", 1, d(2026, 1, 1)), snap("A", "X", 2, d(2026, 1, 1))];
        assert_eq!(holdings_total(&s, d(2026, 1, 1), usd()), Some(2));
    }

    #[test]
    fn categories_group_and_sort() {
        let s = vec![
            snap("ETF", "Stocks", 1_000, d(2026, 1, 1)),
            snap("AAPL", "Stocks", 500, d(2026, 1, 1)),
            snap("Gold", "Metals", 200, d(2026, 1, 1)),
            // Re-categorised later: the latest snapshot's category counts.
            snap("Gold", "Commodities", 300, d(2026, 2, 1)),
        ];
        let c = categories(&s, d(2026, 11, 9), usd());
        assert_eq!(
            c.unwrap(),
            vec![
                ("Stocks".to_string(), 1_500),
                ("Commodities".to_string(), 300),
            ]
        );
    }

    #[test]
    fn category_order_by_first_snapshot_in_any_order() {
        let s = vec![
            snap("Gold", "Metals", 1, d(2026, 3, 1)),
            snap("ETF", "Stocks", 1, d(2026, 1, 1)),
            snap("Coin", "Crypto", 1, d(2026, 3, 1)), // same date as Metals, later row
            snap("ETF", "Stocks", 1, d(2026, 6, 1)),
        ];
        assert_eq!(category_order(&s), ["Stocks", "Metals", "Crypto"]);
    }

    #[test]
    fn empty_is_empty() {
        assert!(categories(&[], d(2026, 2, 1), usd()).unwrap().is_empty());
    }

    #[test]
    fn foreign_holdings_convert_or_are_unavailable() {
        let mut eur = snap("Bund", "Bonds", 1_000, d(2026, 1, 1));
        eur.currency = Currency::Eur;
        let s = vec![snap("ETF", "Stocks", 500, d(2026, 1, 1)), eur];
        let day = d(2026, 2, 1);
        assert_eq!(holdings_total(&s, day, usd()), None);
        assert_eq!(categories(&s, day, usd()), None);

        let r = Rates::new(day, [(Currency::Usd, 1.25)]);
        let fx = Fx::new(Currency::Usd, Some(&r));
        assert_eq!(holdings_total(&s, day, fx), Some(1_750));
        assert_eq!(
            categories(&s, day, fx).unwrap(),
            vec![("Bonds".to_string(), 1_250), ("Stocks".to_string(), 500)]
        );
    }
}
