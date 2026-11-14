//! Holdings: every current holding with its change over the interval.
//!
//! Built from the asset snapshots; listed in the Assets Table.
//! Each holding is compared from a reference point:
//! - held at the interval's window start: its snapshot then, valued at the
//!   window start's rates;
//! - added inside the window: its first snapshot there, valued at that
//!   snapshot's own date's rates.
//!
//! The current value uses today's rates. Past days' rates come from a
//! [`History`]; until a needed day is fetched, change and profit are unknown.

use std::collections::HashMap;

use chrono::NaiveDate;

use super::assets::{self, Snapshot};
use super::currency::Currency;
use super::ledger::{Change, Interval};
use super::money::Cents;
use super::rates::{Fx, History, Total, Valued, Valuer};

/// One holding's row in the Assets Table.
#[derive(Debug, Clone, PartialEq)]
pub struct Holding {
    pub name: String,
    pub category: String,
    /// Current value in the holding's own currency.
    pub value: Cents,
    pub currency: Currency,
    /// Current value in the display currency; `None` while its rate is missing.
    pub display_value: Option<Cents>,
    /// Day the holding is compared from (see the module docs).
    pub since: NaiveDate,
    /// Reference value in the display currency at `since`'s rates; `None`
    /// while those rates are missing.
    pub start: Option<Cents>,
    pub change: Change,
    /// `now - start` in the display currency; `None` while a rate is missing.
    pub profit: Option<Cents>,
}

/// Totals over the listed holdings.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Summary {
    /// Sum of current values in the display currency.
    pub value: Total,
    pub change: Change,
    pub profit: Option<Cents>,
}

/// Each current holding (value not 0) on `today` with its reference
/// snapshot and the day to value it at.
fn references(snapshots: &[Snapshot], interval: Interval, today: NaiveDate) -> Vec<(&Snapshot, &Snapshot, NaiveDate)> {
    let start = interval.window_start(today);
    let at_start: HashMap<&str, &Snapshot> = assets::current(snapshots, start)
        .into_iter()
        .map(|i| (snapshots[i].name.as_str(), &snapshots[i]))
        .filter(|(_, s)| s.value != 0)
        .collect();
    assets::current(snapshots, today)
        .into_iter()
        .map(|i| &snapshots[i])
        .filter(|s| s.value != 0)
        .map(|now| match at_start.get(now.name.as_str()) {
            Some(old) => (now, *old, start),
            None => {
                // Not held at the start: the earliest snapshot inside the
                // window. `now` itself is one, so there always is one.
                let first = snapshots
                    .iter()
                    .filter(|s| s.name == now.name && s.value != 0 && s.date > start && s.date <= today)
                    .min_by_key(|s| s.date)
                    .unwrap_or(now);
                (now, first, first.date)
            }
        })
        .collect()
}

/// Current holdings (value not 0) on `today`, largest display value first;
/// those without a rate go last, ties by name. `fx` holds today's rates;
/// reference days not in `history` come back as missing days.
pub fn holdings(snapshots: &[Snapshot], interval: Interval, today: NaiveDate, fx: Fx, history: &History) -> Valued<Vec<Holding>> {
    let mut valuer = Valuer::new(fx, history, today);
    let mut out: Vec<Holding> = references(snapshots, interval, today)
        .into_iter()
        .map(|(now, old, since)| {
            let value = fx.convert(now.value, now.currency);
            let start = valuer.on(since, |fx| fx.convert(old.value, old.currency));
            Holding {
                name: now.name.clone(),
                category: now.category.clone(),
                value: now.value,
                currency: now.currency,
                display_value: value,
                since,
                start,
                change: Change::of(start, value),
                profit: start.zip(value).map(|(start, value)| value - start),
            }
        })
        .collect();
    // `None` sorts below every `Some`, so reversed alone it would come first.
    out.sort_by(|a, b| {
        let key = |h: &Holding| (h.display_value.is_none(), std::cmp::Reverse(h.display_value));
        key(a).cmp(&key(b)).then_with(|| a.name.cmp(&b.name))
    });
    valuer.finish(out)
}

/// Totals for the ledger footer; with no holdings, change and profit are
/// undefined.
pub fn summary(holdings: &[Holding], fx: Fx) -> Summary {
    let value = fx.sum_known(holdings.iter().map(|h| (h.value, h.currency)));
    if holdings.is_empty() {
        return Summary { value, change: Change::Undefined, profit: None };
    }
    let start: Option<Cents> = holdings.iter().map(|h| h.start).sum();
    let now: Option<Cents> = holdings.iter().map(|h| h.display_value).sum();
    Summary { value, change: Change::of(start, now), profit: start.zip(now).map(|(start, now)| now - start) }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;
    use crate::core::rates::{Bound, Rates};

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    fn snap(name: &str, category: &str, value: Cents, date: NaiveDate) -> Snapshot {
        Snapshot { name: name.into(), category: category.into(), value, currency: Currency::Usd, date }
    }

    fn usd() -> Fx<'static> {
        Fx::new(Currency::Usd, None)
    }

    fn today() -> NaiveDate {
        d(2026, 11, 9)
    }

    fn none() -> History {
        History::default()
    }

    #[test]
    fn change_and_profit_over_the_window() {
        let s = vec![
            snap("ETF", "Stocks", 1_000, d(2026, 1, 1)),
            snap("ETF", "Stocks", 1_200, d(2026, 11, 1)), // before the 1W start (02.11)
            snap("ETF", "Stocks", 1_500, d(2026, 11, 5)),
            snap("ETF", "Stocks", 9_999, d(2026, 12, 1)), // future
        ];
        let h = holdings(&s, Interval::Week, today(), usd(), &none()).value;
        assert_eq!(h.len(), 1);
        assert_eq!((h[0].value, h[0].since, h[0].start, h[0].profit), (1_500, d(2026, 11, 2), Some(1_200), Some(300)));
        assert_eq!(h[0].change, Change::Percent(25.0));
        // Over a year the start is before the first snapshot: compared from
        // the first snapshot inside the window.
        let h = holdings(&s, Interval::Year, today(), usd(), &none()).value;
        assert_eq!((h[0].since, h[0].start, h[0].change), (d(2026, 1, 1), Some(1_000), Change::Percent(50.0)));
    }

    #[test]
    fn foreign_holding_is_valued_at_its_reference_days_rates() {
        let mut gold = snap("Gold", "Metals", 1_000, d(2026, 11, 7)); // 10,00 g
        gold.currency = Currency::Xau;
        let s = vec![gold];
        let usd_per_g = |p: f64| Rates::new(today(), [(Currency::Usd, 1.0)]).with_metal(Currency::Xau, p * 31.103_476_8);
        let live = usd_per_g(110.0);
        let fx = Fx::new(Currency::Usd, Some(&live));

        // Added inside the 1W window: compared from its own date, 07.11,
        // which is asked for while missing.
        let v = holdings(&s, Interval::Week, today(), fx, &none());
        assert_eq!(v.missing_days, BTreeSet::from([d(2026, 11, 7)]));
        let h = &v.value;
        assert_eq!((h[0].display_value, h[0].start, h[0].change), (Some(110_000), None, Change::Unavailable));

        let mut history = History::default();
        history.insert(d(2026, 11, 7), usd_per_g(100.0));
        let v = holdings(&s, Interval::Week, today(), fx, &history);
        assert!(v.missing_days.is_empty());
        let h = &v.value;
        assert_eq!((h[0].start, h[0].profit, h[0].change), (Some(100_000), Some(10_000), Change::Percent(10.0)));

        // Held at the 1D start (08.11): valued at 08.11's rates instead.
        let v = holdings(&s, Interval::Day, today(), fx, &history);
        assert_eq!(v.missing_days, BTreeSet::from([d(2026, 11, 8)]));
        history.insert(d(2026, 11, 8), usd_per_g(105.0));
        assert_eq!(holdings(&s, Interval::Day, today(), fx, &history).value[0].profit, Some(5_000));
        // Same currency as the display: no rates needed.
        let xau = Fx::new(Currency::Xau, None);
        assert!(holdings(&s, Interval::Day, today(), xau, &none()).missing_days.is_empty());
    }

    #[test]
    fn added_today_needs_no_history() {
        let mut btc = snap("Coins", "Crypto", 100_000_000, today());
        btc.currency = Currency::Btc;
        assert!(holdings(&[btc], Interval::Month, today(), usd(), &none()).missing_days.is_empty());
    }

    #[test]
    fn sold_hidden_largest_first() {
        let s = vec![
            snap("Gold", "Metals", 700, d(2026, 1, 1)),
            snap("Bond", "Bonds", 300, d(2026, 1, 1)),
            snap("ETF", "Stocks", 700, d(2026, 1, 1)),
            snap("Bond", "Bonds", 0, d(2026, 6, 1)), // sold
        ];
        let names: Vec<_> = holdings(&s, Interval::Month, today(), usd(), &none()).value.into_iter().map(|h| h.name).collect();
        assert_eq!(names, ["ETF", "Gold"]);
    }

    #[test]
    fn missing_rate_is_unavailable_and_last() {
        let mut eur = snap("Bund", "Bonds", 1_000, d(2026, 1, 1));
        eur.currency = Currency::Eur;
        let s = vec![eur, snap("ETF", "Stocks", 500, d(2026, 1, 1))];
        let h = holdings(&s, Interval::Month, today(), usd(), &none()).value;
        assert_eq!(h[0].name, "ETF");
        assert_eq!((h[1].display_value, h[1].change, h[1].profit), (None, Change::Unavailable, None));
        assert_eq!(h[1].value, 1_000);
        let sum = summary(&h, usd());
        assert_eq!(sum.value, Total { value: 500, bound: Bound::AtLeast });
        assert_eq!((sum.change, sum.profit), (Change::Unavailable, None));
    }

    #[test]
    fn summary_adds_up_the_rows() {
        let s = vec![
            snap("ETF", "Stocks", 1_000, d(2026, 1, 1)),
            snap("Gold", "Metals", 500, d(2026, 1, 1)),
            snap("ETF", "Stocks", 1_300, d(2026, 11, 5)),
            snap("Gold", "Metals", 400, d(2026, 11, 6)),
            snap("New", "Cash", 2_000, d(2026, 11, 7)), // added inside the window, unchanged since
        ];
        let h = holdings(&s, Interval::Week, today(), usd(), &none()).value;
        let sum = summary(&h, usd());
        assert_eq!(sum.value, Total { value: 3_700, bound: Bound::Exact });
        // 3.500 -> 3.700; the new holding adds no profit.
        assert_eq!(sum.profit, Some(200));
        assert!(matches!(sum.change, Change::Percent(p) if (p - 200.0 / 35.0).abs() < 1e-9));
        assert_eq!(summary(&[], usd()), Summary { value: Total { value: 0, bound: Bound::Exact }, change: Change::Undefined, profit: None });
    }
}
