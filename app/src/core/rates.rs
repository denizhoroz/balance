//! Exchange rates and conversion.
//!
//! Fiat is kept per EUR (ECB reference rates), so any pair cross-converts
//! and switching the display currency needs no refetch. Metals and coins
//! are kept in USD per unit: a metal amount is grams (in hundredths, like
//! cents), crypto amounts have 8 decimals; both convert through USD. Where
//! the rates come from is [`super::sources`].
//!
//! Totals convert at the latest rates, whatever an amount's date. Past
//! days' rates ([`History`]) value holdings on the day the Assets Table
//! compares them from.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use super::currency::Currency;
use super::date::ISO_DATE;
use super::money::Cents;

const GRAMS_PER_TROY_OUNCE: f64 = 31.103_476_8;

/// Rates of past days, keyed by the day they were fetched for. A past day's
/// rates never change, so they are kept for good.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct History {
    days: BTreeMap<NaiveDate, Rates>,
}

impl History {
    pub fn get(&self, day: NaiveDate) -> Option<&Rates> {
        self.days.get(&day)
    }

    pub fn insert(&mut self, day: NaiveDate, rates: Rates) {
        self.days.insert(day, rates);
    }

    /// Cache form, keyed `YYYY-MM-DD`.
    pub fn to_data(&self) -> BTreeMap<String, RatesData> {
        self.days.iter().map(|(d, r)| (d.format(ISO_DATE).to_string(), r.to_data())).collect()
    }

    /// Rebuilds from cached data; entries with a bad day are skipped.
    pub fn from_data(data: BTreeMap<String, RatesData>) -> History {
        let days = data
            .into_iter()
            .filter_map(|(d, r)| Some((NaiveDate::parse_from_str(&d, ISO_DATE).ok()?, Rates::from_data(r))))
            .collect();
        History { days }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Rates {
    /// ECB publication date (can lag a few days over weekends and holidays);
    /// `None` until fiat rates arrive.
    pub date: Option<NaiveDate>,
    /// Units of each fiat currency per 1 EUR. EUR itself is implicit.
    per_eur: HashMap<Currency, f64>,
    /// USD per unit of each non-fiat currency: per gram of metal, per coin.
    usd_per_unit: HashMap<Currency, f64>,
}

/// Plain, string-keyed form of [`Rates`] for the on-disk cache.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct RatesData {
    date: Option<String>,
    per_eur: BTreeMap<String, f64>,
    usd_per_unit: BTreeMap<String, f64>,
}

pub(super) fn positive(v: f64) -> bool {
    v.is_finite() && v > 0.0
}

impl Rates {
    pub fn new(date: NaiveDate, per_eur: impl IntoIterator<Item = (Currency, f64)>) -> Rates {
        Rates { date: Some(date), per_eur: per_eur.into_iter().collect(), usd_per_unit: HashMap::new() }
    }

    /// Adds a metal price given in USD per troy ounce.
    pub fn with_metal(mut self, metal: Currency, usd_per_ounce: f64) -> Rates {
        if metal.is_metal() && positive(usd_per_ounce) {
            self.usd_per_unit.insert(metal, usd_per_ounce / GRAMS_PER_TROY_OUNCE);
        }
        self
    }

    /// Adds a crypto price given in USD per coin.
    pub fn with_crypto(mut self, coin: Currency, usd_per_coin: f64) -> Rates {
        if coin.coingecko_id().is_some() && positive(usd_per_coin) {
            self.usd_per_unit.insert(coin, usd_per_coin);
        }
        self
    }

    /// Whether these rates price `c`: a fiat currency once the fiat table
    /// is in (EUR always then), any other once its price is.
    pub fn has(&self, c: Currency) -> bool {
        if c.is_fiat() {
            self.date.is_some() && (c == Currency::Eur || self.per_eur.contains_key(&c))
        } else {
            self.usd_per_unit.contains_key(&c)
        }
    }

    /// Only the fiat table (and its date).
    pub fn fiat_only(self) -> Rates {
        Rates { usd_per_unit: HashMap::new(), ..self }
    }

    /// Only the non-fiat prices `keep` picks; no fiat table.
    pub fn prices(self, keep: impl Fn(Currency) -> bool) -> Rates {
        Rates {
            date: None,
            per_eur: HashMap::new(),
            usd_per_unit: self.usd_per_unit.into_iter().filter(|(c, _)| keep(*c)).collect(),
        }
    }

    /// Cache form of these rates.
    pub fn to_data(&self) -> RatesData {
        let keyed = |m: &HashMap<Currency, f64>| m.iter().map(|(c, v)| (c.code().to_string(), *v)).collect();
        RatesData {
            date: self.date.map(|d| d.format(ISO_DATE).to_string()),
            per_eur: keyed(&self.per_eur),
            usd_per_unit: keyed(&self.usd_per_unit),
        }
    }

    /// Rebuilds rates from cached data. Entries that no longer make sense
    /// (unknown or wrong-kind currency, bad number, bad date) are skipped.
    pub fn from_data(data: RatesData) -> Rates {
        let currencies = |m: BTreeMap<String, f64>, fiat: bool| -> HashMap<Currency, f64> {
            m.into_iter()
                .filter(|(_, v)| positive(*v))
                .filter_map(|(code, v)| Some((Currency::from_code(&code).filter(|c| c.is_fiat() == fiat)?, v)))
                .collect()
        };
        Rates {
            date: data.date.and_then(|d| NaiveDate::parse_from_str(&d, ISO_DATE).ok()),
            per_eur: currencies(data.per_eur, true),
            usd_per_unit: currencies(data.usd_per_unit, false),
        }
    }

    /// Takes what `other` knows: its fiat table when it has one, and every
    /// non-fiat price. Sources arrive in any order.
    pub fn merge(&mut self, other: Rates) {
        if other.date.is_some() {
            self.date = other.date;
            self.per_eur = other.per_eur;
        }
        self.usd_per_unit.extend(other.usd_per_unit);
    }

    /// Units of `c` per 1 EUR (grams for metals, coins for crypto).
    fn per_eur(&self, c: Currency) -> Option<f64> {
        if c == Currency::Eur {
            return Some(1.0);
        }
        if !c.is_fiat() {
            let usd_per_eur = self.per_eur(Currency::Usd)?;
            return Some(usd_per_eur / self.usd_per_unit.get(&c)?);
        }
        self.per_eur.get(&c).copied()
    }
}

/// A result valued partly at past days' rates, with the past days whose
/// rates it needed but which are not loaded yet: fetch those, then value
/// again. Callers cannot get the value without seeing the days.
#[derive(Debug, Clone, PartialEq)]
pub struct Valued<T> {
    pub value: T,
    pub missing_days: BTreeSet<NaiveDate>,
}

/// Values amounts on any day: `today` (or later) at the live rates, past
/// days at that day's rates from `history`. The one place that decides
/// which rates a day uses; it records each past day whose rates were
/// needed but missing.
pub struct Valuer<'a> {
    fx: Fx<'a>,
    history: &'a History,
    today: NaiveDate,
    missing: BTreeSet<NaiveDate>,
}

impl<'a> Valuer<'a> {
    /// `fx` holds today's rates.
    pub fn new(fx: Fx<'a>, history: &'a History, today: NaiveDate) -> Valuer<'a> {
        Valuer { fx, history, today, missing: BTreeSet::new() }
    }

    /// Runs `value` with `day`'s rates. On a past day, `None` means a rate
    /// was missing: the day is recorded for fetching.
    pub fn on<T>(&mut self, day: NaiveDate, value: impl FnOnce(Fx<'a>) -> Option<T>) -> Option<T> {
        if day >= self.today {
            return value(self.fx);
        }
        let out = value(Fx::new(self.fx.to, self.history.get(day)));
        if out.is_none() {
            self.missing.insert(day);
        }
        out
    }

    /// Pairs `value` with the past days that were missing.
    pub fn finish<T>(self, value: T) -> Valued<T> {
        Valued { value, missing_days: self.missing }
    }
}

/// Converts amounts into the display currency `to`.
#[derive(Debug, Clone, Copy)]
pub struct Fx<'a> {
    pub to: Currency,
    pub rates: Option<&'a Rates>,
}

impl<'a> Fx<'a> {
    pub fn new(to: Currency, rates: Option<&'a Rates>) -> Fx<'a> {
        Fx { to, rates }
    }

    /// `units` (minor units of `from`) expressed in minor units of the
    /// display currency. Same currency needs no rates; otherwise `None`
    /// while the rate is unknown. Rounds half away from zero.
    pub fn convert(&self, units: Cents, from: Currency) -> Option<Cents> {
        if from == self.to {
            return Some(units);
        }
        let r = self.rates?;
        let digits = self.to.decimals() as i32 - from.decimals() as i32;
        let v = units as f64 * r.per_eur(self.to)? / r.per_eur(from)? * 10f64.powi(digits);
        v.is_finite().then(|| v.round() as Cents)
    }

    /// Sums the rows whose rate is known. Rows without one are left out and
    /// the total is then only [`Bound::AtLeast`].
    pub fn sum_known(&self, rows: impl Iterator<Item = (Cents, Currency)>) -> Total {
        let mut total = Total { value: 0, bound: Bound::Exact };
        for (units, from) in rows {
            match self.convert(units, from) {
                Some(v) => total.value += v,
                None => total.bound = Bound::AtLeast,
            }
        }
        total
    }
}

/// How a displayed [`Total`] relates to the true value when some rows could
/// not be converted yet (their rate is not fetched).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bound {
    /// Every row converted.
    Exact,
    /// The true value is at least this (unconverted rows only add).
    AtLeast,
    /// The true value is at most this (unconverted rows only subtract).
    AtMost,
    /// Unconverted rows pull both ways; the shown value says nothing.
    Unknown,
}

/// A sum over the rows that could be converted, plus what that means.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Total {
    pub value: Cents,
    pub bound: Bound,
}

impl Total {
    /// `self - other`, where each side is a sum of non-negative rows.
    pub fn minus(self, other: Total) -> Total {
        let bound = match (self.bound, other.bound) {
            (Bound::Unknown, _) | (_, Bound::Unknown) => Bound::Unknown,
            (a, Bound::Exact) => a,
            (Bound::Exact, Bound::AtLeast) => Bound::AtMost,
            (Bound::AtLeast, Bound::AtLeast) => Bound::Unknown,
            // Sides built with `minus` are not subtracted again.
            _ => Bound::Unknown,
        };
        Total { value: self.value - other.value, bound }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rates() -> Rates {
        Rates::new(
            NaiveDate::from_ymd_opt(2026, 11, 9).unwrap(),
            [(Currency::Usd, 1.25), (Currency::Try, 50.0), (Currency::Jpy, 160.0)],
        )
    }

    #[test]
    fn history_round_trips() {
        let mut h = History::default();
        let day = NaiveDate::from_ymd_opt(2026, 11, 9).unwrap();
        h.insert(day, rates().with_metal(Currency::Xau, 4_000.0));
        let json = serde_json::to_string(&h.to_data()).unwrap();
        let back = History::from_data(serde_json::from_str(&json).unwrap());
        assert_eq!(back, h);
        assert!(back.get(day).is_some() && back.get(day.succ_opt().unwrap()).is_none());
    }

    #[test]
    fn crypto_converts_with_8_decimals() {
        let r = rates().with_crypto(Currency::Btc, 80_000.0);
        // 0,005 BTC = 400 USD = 320 EUR.
        let half_cent_btc = 500_000; // 0,00500000
        assert_eq!(Fx::new(Currency::Usd, Some(&r)).convert(half_cent_btc, Currency::Btc), Some(40_000));
        assert_eq!(Fx::new(Currency::Eur, Some(&r)).convert(half_cent_btc, Currency::Btc), Some(32_000));
        // ETH has no price here.
        assert_eq!(Fx::new(Currency::Usd, Some(&r)).convert(100_000_000, Currency::Eth), None);
    }

    #[test]
    fn data_round_trips() {
        let r = rates().with_metal(Currency::Xau, 100.0 * GRAMS_PER_TROY_OUNCE).with_crypto(Currency::Btc, 80_000.0);
        let json = serde_json::to_string(&r.to_data()).unwrap();
        assert_eq!(Rates::from_data(serde_json::from_str(&json).unwrap()), r);
        // Fiat code in the non-fiat table (and vice versa) is dropped.
        let bad = RatesData {
            date: Some("nope".into()),
            per_eur: [("XAU".to_string(), 1.0)].into(),
            usd_per_unit: [("USD".to_string(), 1.0)].into(),
        };
        assert_eq!(Rates::from_data(bad), Rates::default());
    }

    #[test]
    fn same_currency_needs_no_rates() {
        assert_eq!(Fx::new(Currency::Try, None).convert(1_234, Currency::Try), Some(1_234));
        assert_eq!(Fx::new(Currency::Try, None).convert(1_234, Currency::Usd), None);
    }

    #[test]
    fn converts_direct_and_cross() {
        let r = rates();
        // 1 EUR = 1,25 USD
        assert_eq!(Fx::new(Currency::Usd, Some(&r)).convert(10_000, Currency::Eur), Some(12_500));
        assert_eq!(Fx::new(Currency::Eur, Some(&r)).convert(12_500, Currency::Usd), Some(10_000));
        // 1 USD = 50 / 1,25 = 40 TRY
        assert_eq!(Fx::new(Currency::Try, Some(&r)).convert(100, Currency::Usd), Some(4_000));
    }

    #[test]
    fn metals_convert_per_gram_through_usd() {
        // 100 USD per gram.
        let r = rates().with_metal(Currency::Xau, 100.0 * GRAMS_PER_TROY_OUNCE).with_metal(Currency::Xag, GRAMS_PER_TROY_OUNCE);
        // 10,00 g gold = 1.000 USD = 800 EUR = 40.000 TRY.
        assert_eq!(Fx::new(Currency::Usd, Some(&r)).convert(1_000, Currency::Xau), Some(100_000));
        assert_eq!(Fx::new(Currency::Eur, Some(&r)).convert(1_000, Currency::Xau), Some(80_000));
        assert_eq!(Fx::new(Currency::Try, Some(&r)).convert(1_000, Currency::Xau), Some(4_000_000));
        // 2,50 g silver at 1 USD/g.
        assert_eq!(Fx::new(Currency::Usd, Some(&r)).convert(250, Currency::Xag), Some(250));
    }

    #[test]
    fn metal_missing_price_or_usd_is_none() {
        let r = rates();
        assert_eq!(Fx::new(Currency::Usd, Some(&r)).convert(100, Currency::Xau), None);
        // Price known but no USD rate to bridge to EUR.
        let r = Rates::new(NaiveDate::MIN, [(Currency::Try, 50.0)]).with_metal(Currency::Xau, 4_000.0);
        assert_eq!(Fx::new(Currency::Try, Some(&r)).convert(100, Currency::Xau), None);
    }

    #[test]
    fn sources_merge_in_any_order() {
        let gold = || Rates::default().with_metal(Currency::Xau, 100.0 * GRAMS_PER_TROY_OUNCE);
        // Fiat alone converts fiat at once; gold waits for its price.
        let mut a = Rates::default();
        assert_eq!(Fx::new(Currency::Usd, Some(&a)).convert(4_000, Currency::Try), None);
        a.merge(rates());
        assert_eq!(Fx::new(Currency::Usd, Some(&a)).convert(4_000, Currency::Try), Some(100));
        assert_eq!(Fx::new(Currency::Usd, Some(&a)).convert(100, Currency::Xau), None);
        a.merge(gold());
        // Gold first, fiat later: same result; a price-only part keeps the fiat table.
        let mut b = Rates::default();
        b.merge(gold());
        b.merge(rates());
        b.merge(Rates::default().with_crypto(Currency::Btc, 1.0));
        assert_eq!(a.date, b.date);
        for r in [&a, &b] {
            assert_eq!(Fx::new(Currency::Usd, Some(r)).convert(100, Currency::Xau), Some(10_000));
        }
    }

    #[test]
    fn partial_sum_flags_missing_rates() {
        let r = rates();
        let fx = Fx::new(Currency::Usd, Some(&r));
        // 100 EUR known (125 USD); 1 g gold has no price yet.
        let rows = [(10_000, Currency::Eur), (100, Currency::Xau)];
        let t = fx.sum_known(rows.into_iter());
        assert_eq!(t, Total { value: 12_500, bound: Bound::AtLeast });
        let exact = fx.sum_known([(10_000, Currency::Eur)].into_iter());
        assert_eq!(exact.bound, Bound::Exact);
        // Missing income -> balance at least; missing expense -> at most; both -> unknown.
        assert_eq!(t.minus(exact).bound, Bound::AtLeast);
        assert_eq!(exact.minus(t).bound, Bound::AtMost);
        assert_eq!(t.minus(t).bound, Bound::Unknown);
    }

    #[test]
    fn missing_rate_is_none() {
        let r = rates();
        assert_eq!(Fx::new(Currency::Usd, Some(&r)).convert(100, Currency::Gbp), None);
        assert_eq!(Fx::new(Currency::Gbp, Some(&r)).convert(100, Currency::Usd), None);
    }

    #[test]
    fn rounds_half_away_from_zero() {
        let r = Rates::new(NaiveDate::MIN, [(Currency::Usd, 1.5)]);
        let fx = Fx::new(Currency::Usd, Some(&r));
        assert_eq!(fx.convert(1, Currency::Eur), Some(2)); // 1,5 -> 2
        assert_eq!(fx.convert(-1, Currency::Eur), Some(-2));
        assert_eq!(fx.convert(3, Currency::Eur), Some(5)); // 4,5 -> 5
    }
}
