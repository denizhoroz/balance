//! Actions (incomes/expenses), asset snapshots, balance, intervals and change %.
//!
//! Totals are in the display currency (see [`Fx`]); `None` means a needed
//! exchange rate is not loaded.

use chrono::{Days, Months, NaiveDate};
use serde::{Deserialize, Serialize};

use super::assets::{self, Snapshot};
use super::currency::Currency;
use super::money::Cents;
use super::rates::{Fx, History, Total, Valued, Valuer};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    Income,
    Expense,
}

impl Kind {
    pub fn sheet_name(self) -> &'static str {
        match self {
            Kind::Income => "Incomes",
            Kind::Expense => "Expenses",
        }
    }
}

/// One income or expense. A stored action always has a non-empty name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Action {
    pub name: String,
    pub value: Cents,
    pub currency: Currency,
    pub date: NaiveDate,
}

/// Everything stored in the workbook.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Ledger {
    pub incomes: Vec<Action>,
    pub expenses: Vec<Action>,
    pub assets: Vec<Snapshot>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Interval {
    #[serde(rename = "1D")]
    Day,
    #[serde(rename = "1W")]
    Week,
    #[default]
    #[serde(rename = "1M")]
    Month,
    #[serde(rename = "1Y")]
    Year,
}

impl Interval {
    pub fn label(self) -> &'static str {
        match self {
            Interval::Day => "1D",
            Interval::Week => "1W",
            Interval::Month => "1M",
            Interval::Year => "1Y",
        }
    }

    pub fn next(self) -> Interval {
        match self {
            Interval::Day => Interval::Week,
            Interval::Week => Interval::Month,
            Interval::Month => Interval::Year,
            Interval::Year => Interval::Day,
        }
    }

    /// Rolling window: actions dated in `(start, today]` are inside it.
    /// Returns `start` (exclusive).
    pub fn window_start(self, today: NaiveDate) -> NaiveDate {
        let r = match self {
            Interval::Day => today.checked_sub_days(Days::new(1)),
            Interval::Week => today.checked_sub_days(Days::new(7)),
            Interval::Month => today.checked_sub_months(Months::new(1)),
            Interval::Year => today.checked_sub_months(Months::new(12)),
        };
        r.unwrap_or(NaiveDate::MIN)
    }
}

/// Result of the change % calculation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Change {
    /// Percentage (e.g. `14.0` = +14%).
    Percent(f64),
    /// Start balance was 0 and current is not: no % from zero.
    Undefined,
    /// A needed exchange rate is not loaded.
    Unavailable,
}

impl Change {
    /// `(now - start) / |start|`; from 0 it is `+0%` (still 0) or undefined.
    pub fn between(start: Cents, now: Cents) -> Change {
        if start == 0 {
            return if now == 0 { Change::Percent(0.0) } else { Change::Undefined };
        }
        // Multiply first so whole percentages stay exact in f64.
        Change::Percent((now - start) as f64 * 100.0 / (start.abs() as f64))
    }

    /// Like [`Change::between`], `Unavailable` when either side is.
    pub fn of(start: Option<Cents>, now: Option<Cents>) -> Change {
        match (start, now) {
            (Some(s), Some(n)) => Change::between(s, n),
            _ => Change::Unavailable,
        }
    }
}

impl Ledger {
    pub fn list(&self, kind: Kind) -> &Vec<Action> {
        match kind {
            Kind::Income => &self.incomes,
            Kind::Expense => &self.expenses,
        }
    }

    pub fn list_mut(&mut self, kind: Kind) -> &mut Vec<Action> {
        match kind {
            Kind::Income => &mut self.incomes,
            Kind::Expense => &mut self.expenses,
        }
    }

    fn sum(list: &[Action], fx: Fx, f: impl Fn(&Action) -> bool) -> Option<Cents> {
        list.iter().filter(|a| f(a)).map(|a| fx.convert(a.value, a.currency)).sum()
    }

    /// Total of a kind counting only actions dated on or before `today`.
    pub fn total(&self, kind: Kind, today: NaiveDate, fx: Fx) -> Option<Cents> {
        Self::sum(self.list(kind), fx, |a| a.date <= today)
    }

    /// Balance on `day`: incomes - expenses dated on or before `day`.
    pub fn balance_at(&self, day: NaiveDate, fx: Fx) -> Option<Cents> {
        Some(self.total(Kind::Income, day, fx)? - self.total(Kind::Expense, day, fx)?)
    }

    /// Like [`Ledger::total`], but keeps what is convertible and flags the rest.
    pub fn total_partial(&self, kind: Kind, today: NaiveDate, fx: Fx) -> Total {
        fx.sum_known(self.list(kind).iter().filter(|a| a.date <= today).map(|a| (a.value, a.currency)))
    }

    /// Like [`Ledger::balance_at`], but partial: see [`Total`].
    pub fn balance_partial(&self, day: NaiveDate, fx: Fx) -> Total {
        self.total_partial(Kind::Income, day, fx).minus(self.total_partial(Kind::Expense, day, fx))
    }

    /// Total of a kind dated inside the interval window `(start, today]`.
    /// Keeps what is convertible and flags the rest (see [`Total`]).
    pub fn total_in_partial(&self, kind: Kind, interval: Interval, today: NaiveDate, fx: Fx) -> Total {
        let start = interval.window_start(today);
        fx.sum_known(self.list(kind).iter().filter(|a| a.date > start && a.date <= today).map(|a| (a.value, a.currency)))
    }

    /// Like [`Ledger::assets_at`], but keeps what is convertible and flags the rest.
    pub fn assets_partial(&self, day: NaiveDate, fx: Fx) -> Total {
        fx.sum_known(assets::current(&self.assets, day).into_iter().map(|i| (self.assets[i].value, self.assets[i].currency)))
    }

    /// `(B_now - B_start) / |B_start|`, where `B_start` is the balance from
    /// actions dated on or before the window start. Both use today's rates.
    pub fn change(&self, interval: Interval, today: NaiveDate, fx: Fx) -> Change {
        let now = self.balance_at(today, fx);
        let start = self.balance_at(interval.window_start(today), fx);
        Change::of(start, now)
    }

    /// Asset value per category on `day`, largest first.
    pub fn asset_categories(&self, day: NaiveDate, fx: Fx) -> Option<Vec<(String, Cents)>> {
        assets::categories(&self.assets, day, fx)
    }

    /// Total assets on `day`. Independent of the income/expense balance.
    pub fn assets_at(&self, day: NaiveDate, fx: Fx) -> Option<Cents> {
        assets::holdings_total(&self.assets, day, fx)
    }

    /// Change % of total assets over the interval window. The start is valued
    /// at the window start day's rates (from `history`), so price moves count,
    /// not just recorded values; `Unavailable` until that day is loaded (it is
    /// then in the missing days).
    pub fn assets_change(&self, interval: Interval, today: NaiveDate, fx: Fx, history: &History) -> Valued<Change> {
        let mut valuer = Valuer::new(fx, history, today);
        let start_day = interval.window_start(today);
        let start = valuer.on(start_day, |fx| self.assets_at(start_day, fx));
        valuer.finish(Change::of(start, self.assets_at(today, fx)))
    }

    /// Whether any row is in a currency other than `to` (so totals need rates).
    pub fn uses_foreign(&self, to: Currency) -> bool {
        self.incomes.iter().chain(&self.expenses).any(|a| a.currency != to)
            || self.assets.iter().any(|s| s.currency != to)
    }

    /// Sorts all lists by date ascending (stable: equal dates keep order).
    pub fn sort(&mut self) {
        self.incomes.sort_by_key(|a| a.date);
        self.expenses.sort_by_key(|a| a.date);
        self.assets.sort_by_key(|a| a.date);
    }
}

/// `+14%`, `-5%`, `+0,5%`, `+0%`. One decimal below 10%, none above.
pub fn format_change(change: Change) -> String {
    match change {
        Change::Undefined | Change::Unavailable => "—".into(),
        Change::Percent(p) => {
            let sign = if p < 0.0 { "-" } else { "+" };
            let a = p.abs();
            let body = if a < 10.0 {
                let t = (a * 10.0).trunc() / 10.0;
                let s = format!("{t:.1}");
                let s = s.trim_end_matches('0').trim_end_matches('.').to_string();
                s.replace('.', ",")
            } else {
                format!("{}", a.trunc() as i64)
            };
            if body == "0" && p != 0.0 {
                // Tiny non-zero change: keep its sign visible.
                return format!("{sign}<0,1%");
            }
            format!("{sign}{body}%")
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    use crate::core::rates::Rates;

    fn act(name: &str, value: Cents, date: NaiveDate) -> Action {
        Action { name: name.into(), value, currency: Currency::Usd, date }
    }

    fn usd() -> Fx<'static> {
        Fx::new(Currency::Usd, None)
    }

    fn today() -> NaiveDate {
        d(2026, 11, 9)
    }

    #[test]
    fn balance_excludes_future() {
        let l = Ledger {
            incomes: vec![act("salary", 100_000, d(2026, 11, 1)), act("bonus", 50_000, d(2026, 12, 1))],
            expenses: vec![act("rent", 30_000, d(2026, 11, 2)), act("trip", 10_000, d(2026, 11, 20))],
            assets: vec![],
        };
        assert_eq!(l.balance_at(today(), usd()), Some(70_000));
        assert_eq!(l.total(Kind::Income, today(), usd()), Some(100_000));
        assert_eq!(l.total(Kind::Expense, today(), usd()), Some(30_000));
    }

    #[test]
    fn interval_cycle() {
        assert_eq!(Interval::Day.next(), Interval::Week);
        assert_eq!(Interval::Year.next(), Interval::Day);
    }

    #[test]
    fn window_start_is_rolling() {
        assert_eq!(Interval::Day.window_start(today()), d(2026, 11, 8));
        assert_eq!(Interval::Week.window_start(today()), d(2026, 11, 2));
        assert_eq!(Interval::Month.window_start(today()), d(2026, 10, 9));
        assert_eq!(Interval::Year.window_start(today()), d(2025, 11, 9));
        assert_eq!(Interval::Month.window_start(d(2026, 3, 31)), d(2026, 2, 28));
    }

    #[test]
    fn totals_in_window() {
        let l = Ledger {
            incomes: vec![
                act("a", 100, d(2026, 11, 9)),
                act("b", 200, d(2026, 11, 8)),
                act("c", 400, d(2026, 11, 3)),
                act("future", 800, d(2026, 11, 10)),
            ],
            expenses: vec![],
            assets: vec![],
        };
        assert_eq!(l.total_in_partial(Kind::Income, Interval::Day, today(), usd()).value, 100);
        assert_eq!(l.total_in_partial(Kind::Income, Interval::Week, today(), usd()).value, 700);
    }

    #[test]
    fn change_percent() {
        let l = Ledger {
            incomes: vec![act("start", 100_000, d(2026, 1, 1)), act("now", 14_000, d(2026, 11, 5))],
            expenses: vec![],
            assets: vec![],
        };
        assert_eq!(l.change(Interval::Week, today(), usd()), Change::Percent(14.0));
    }

    #[test]
    fn change_from_negative_uses_abs() {
        let l = Ledger {
            incomes: vec![act("pay", 50, d(2026, 11, 9))],
            expenses: vec![act("debt", 100, d(2026, 1, 1))],
            assets: vec![],
        };
        // -100 -> -50: improvement, positive change.
        assert_eq!(l.change(Interval::Day, today(), usd()), Change::Percent(50.0));
    }

    #[test]
    fn change_zero_edge_cases() {
        let empty = Ledger::default();
        assert_eq!(empty.change(Interval::Month, today(), usd()), Change::Percent(0.0));
        let l = Ledger {
            incomes: vec![act("new", 100, d(2026, 11, 9))],
            expenses: vec![],
            assets: vec![],
        };
        assert_eq!(l.change(Interval::Month, today(), usd()), Change::Undefined);
    }

    #[test]
    fn change_format() {
        assert_eq!(format_change(Change::Percent(14.0)), "+14%");
        assert_eq!(format_change(Change::Percent(-5.0)), "-5%");
        assert_eq!(format_change(Change::Percent(0.0)), "+0%");
        assert_eq!(format_change(Change::Percent(2.56)), "+2,5%");
        assert_eq!(format_change(Change::Percent(-123.9)), "-123%");
        assert_eq!(format_change(Change::Percent(0.01)), "+<0,1%");
        assert_eq!(format_change(Change::Undefined), "—");
        assert_eq!(format_change(Change::Unavailable), "—");
    }

    #[test]
    fn sort_by_date_stable() {
        let mut l = Ledger {
            incomes: vec![act("b", 1, d(2026, 2, 1)), act("a", 1, d(2026, 1, 1)), act("c", 1, d(2026, 2, 1))],
            expenses: vec![],
            assets: vec![],
        };
        l.sort();
        let names: Vec<_> = l.incomes.iter().map(|a| a.name.as_str()).collect();
        assert_eq!(names, ["a", "b", "c"]);
    }

    #[test]
    fn assets_ignore_balance_and_change() {
        let snap = |value, date| Snapshot {
            name: "ETF".into(),
            category: "Stocks".into(),
            value,
            currency: Currency::Usd,
            date,
        };
        let l = Ledger {
            incomes: vec![act("pay", 1_000, d(2026, 1, 1))],
            expenses: vec![],
            assets: vec![snap(1_000, d(2026, 1, 1)), snap(1_400, d(2026, 11, 5))],
        };
        // The 1.000 income is cash balance, not an asset.
        assert_eq!(l.assets_at(today(), usd()), Some(1_400));
        // 1.000 at window start -> 1.400 now
        let change = l.assets_change(Interval::Week, today(), usd(), &History::default());
        assert_eq!((change.value, change.missing_days.len()), (Change::Percent(40.0), 0));
        assert_eq!(l.asset_categories(today(), usd()).unwrap().len(), 1);
        assert_eq!(Ledger { incomes: l.incomes.clone(), ..Ledger::default() }.assets_at(today(), usd()), Some(0));
        assert_eq!(l.asset_categories(today(), usd()).unwrap()[0], ("Stocks".to_string(), 1_400));
    }

    #[test]
    fn assets_change_counts_price_moves() {
        // 10,00 g gold held all window, nothing recorded since.
        let gold = Snapshot { name: "Gold".into(), category: "Metals".into(), value: 1_000, currency: Currency::Xau, date: d(2026, 1, 1) };
        let l = Ledger { assets: vec![gold], ..Ledger::default() };
        let usd_per_g = |p: f64| Rates::new(today(), [(Currency::Usd, 1.0)]).with_metal(Currency::Xau, p * 31.103_476_8);
        let live = usd_per_g(110.0);
        let fx = Fx::new(Currency::Usd, Some(&live));
        let start = d(2026, 11, 2);
        // Window start not loaded yet: unavailable, and asks for that day.
        let mut history = History::default();
        let change = l.assets_change(Interval::Week, today(), fx, &history);
        assert_eq!((change.value, change.missing_days), (Change::Unavailable, BTreeSet::from([start])));
        // 100 -> 110 USD/g: +10%, though the grams did not change.
        history.insert(start, usd_per_g(100.0));
        let change = l.assets_change(Interval::Week, today(), fx, &history);
        assert_eq!((change.value, change.missing_days.len()), (Change::Percent(10.0), 0));
    }

    #[test]
    fn mixed_currencies_convert_at_todays_rate() {
        let mut eur = act("bonus", 1_000, d(2026, 11, 8));
        eur.currency = Currency::Eur;
        let l = Ledger {
            incomes: vec![act("pay", 10_000, d(2026, 1, 1)), eur],
            expenses: vec![],
            assets: vec![],
        };
        assert!(l.uses_foreign(Currency::Usd));
        assert!(!Ledger::default().uses_foreign(Currency::Usd));
        // No rates: anything touching the EUR row is unavailable.
        assert_eq!(l.balance_at(today(), usd()), None);
        assert_eq!(l.change(Interval::Week, today(), usd()), Change::Unavailable);
        // Window start (before the EUR row) needs no rate.
        assert_eq!(l.balance_at(d(2026, 11, 2), usd()), Some(10_000));

        let r = Rates::new(today(), [(Currency::Usd, 1.5)]);
        let fx = Fx::new(Currency::Usd, Some(&r));
        assert_eq!(l.balance_at(today(), fx), Some(11_500));
        assert_eq!(l.change(Interval::Week, today(), fx), Change::Percent(15.0));
        // Display in EUR: the USD row converts instead.
        let fx = Fx::new(Currency::Eur, Some(&r));
        assert_eq!(l.total(Kind::Income, today(), fx), Some(7_667));
    }
}
