//! Rate feed bookkeeping: rates received so far (and cached), which feeds
//! and past days are due, in flight or backing off after a failure. The
//! fetching itself runs in the UI's background tasks.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use chrono::NaiveDate;

use super::cache;
use super::rates::{History, Rates};
use super::sources::Source;

/// Retry delay after the `n`th consecutive failure: 5s, 10s, 20s, ...
/// capped at 5 min.
fn backoff(failures: u32) -> Duration {
    Duration::from_secs((5u64 << failures.min(6)).min(300))
}

/// Rates received so far plus the bookkeeping of which feeds are current.
pub struct RateFeeds {
    /// Exchange rates and prices, merged per source.
    pub rates: Rates,
    /// Folder of the price cache (`.cache/`).
    cache: PathBuf,
    /// Local day each source last succeeded (refetch on a new day).
    fetched_on: HashMap<Source, NaiveDate>,
    /// Sources with a fetch in flight.
    in_flight: HashSet<Source>,
    /// Consecutive failures per source, for the retry backoff.
    failures: HashMap<Source, u32>,
    /// Rates of past days, for valuing holdings on their reference day.
    pub history: History,
    /// Past days with a fetch in flight.
    days_in_flight: HashSet<NaiveDate>,
    /// Failed past days: consecutive failures and when to try again.
    day_failures: HashMap<NaiveDate, (u32, Instant)>,
}

impl RateFeeds {
    /// Starts from the cache so prices show before any network call. A
    /// feed cached today needs no fetch; an older one is shown until its
    /// refetch lands (or keeps showing if that fails).
    pub fn load(cache: PathBuf) -> RateFeeds {
        let mut feeds = RateFeeds {
            rates: Rates::default(),
            cache,
            fetched_on: HashMap::new(),
            in_flight: HashSet::new(),
            failures: HashMap::new(),
            history: History::default(),
            days_in_flight: HashSet::new(),
            day_failures: HashMap::new(),
        };
        feeds.history = cache::load_history(&feeds.cache);
        for source in Source::all() {
            let Some(cached) = cache::load(&feeds.cache, source) else { continue };
            feeds.rates.merge(cached.rates);
            feeds.fetched_on.insert(source, cached.fetched);
        }
        feeds
    }

    /// Sources not yet fetched `today` and not in flight; marks them in flight.
    pub fn take_due(&mut self, today: NaiveDate) -> Vec<Source> {
        let due: Vec<Source> = Source::all()
            .filter(|s| !self.in_flight.contains(s) && self.fetched_on.get(s) != Some(&today))
            .collect();
        self.in_flight.extend(&due);
        due
    }

    /// A fetch landed: merge it, cache it, and clear the failure streak.
    pub fn succeed(&mut self, source: Source, part: Rates, today: NaiveDate) {
        self.in_flight.remove(&source);
        if let Err(e) = cache::store(&self.cache, source, &part, today) {
            eprintln!("balance: failed to cache {}: {e}", source.name());
        }
        self.rates.merge(part);
        self.fetched_on.insert(source, today);
        self.failures.remove(&source);
    }

    /// A fetch failed; returns how long to wait before retrying it.
    pub fn fail(&mut self, source: Source) -> Duration {
        self.in_flight.remove(&source);
        let n = self.failures.entry(source).or_insert(0);
        let delay = backoff(*n);
        *n += 1;
        delay
    }

    /// Of `days`, those not stored, not in flight and not waiting out a
    /// failure backoff; marks them in flight.
    pub fn take_due_days(&mut self, days: impl IntoIterator<Item = NaiveDate>, now: Instant) -> Vec<NaiveDate> {
        let due: Vec<NaiveDate> = days
            .into_iter()
            .filter(|d| self.history.get(*d).is_none() && !self.days_in_flight.contains(d))
            .filter(|d| self.day_failures.get(d).is_none_or(|(_, retry_at)| now >= *retry_at))
            .collect();
        self.days_in_flight.extend(&due);
        due
    }

    /// A past day's rates landed: keep and cache them.
    pub fn day_succeeded(&mut self, day: NaiveDate, part: Rates) {
        self.days_in_flight.remove(&day);
        self.day_failures.remove(&day);
        self.history.insert(day, part);
        if let Err(e) = cache::store_history(&self.cache, &self.history) {
            eprintln!("balance: failed to cache rates of {day}: {e}");
        }
    }

    /// A past day failed; returns how long until it may be tried again.
    pub fn day_failed(&mut self, day: NaiveDate, now: Instant) -> Duration {
        self.days_in_flight.remove(&day);
        let n = self.day_failures.get(&day).map_or(0, |(n, _)| *n);
        let delay = backoff(n);
        self.day_failures.insert(day, (n + 1, now + delay));
        delay
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::currency::Currency;

    fn d(day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 11, day).unwrap()
    }

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("balance-feeds-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn backoff_doubles_then_caps() {
        let secs: Vec<u64> = (0..9).map(|n| backoff(n).as_secs()).collect();
        assert_eq!(secs, [5, 10, 20, 40, 80, 160, 300, 300, 300]);
    }

    #[test]
    fn everything_is_due_without_a_cache() {
        let mut feeds = RateFeeds::load(scratch("empty"));
        assert_eq!(feeds.take_due(d(11)).len(), Source::all().count());
        // In flight now: not handed out twice.
        assert!(feeds.take_due(d(11)).is_empty());
    }

    #[test]
    fn cache_fetched_today_is_not_due_but_older_is() {
        let dir = scratch("cached");
        let fiat = Rates::new(d(9), [(Currency::Usd, 1.25)]);
        cache::store(&dir, Source::Fiat, &fiat, d(11)).unwrap();
        let btc = Rates::default().with_crypto(Currency::Btc, 80_000.0);
        cache::store(&dir, Source::Crypto, &btc, d(10)).unwrap();

        let mut feeds = RateFeeds::load(dir.clone());
        // Cached data is usable at once, even the stale crypto.
        assert_eq!(feeds.rates.date, Some(d(9)));
        let due = feeds.take_due(d(11));
        assert!(!due.contains(&Source::Fiat));
        assert!(due.contains(&Source::Crypto));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn past_days_fetch_once_and_back_off() {
        let dir = scratch("days");
        let mut feeds = RateFeeds::load(dir.clone());
        let t0 = Instant::now();
        assert_eq!(feeds.take_due_days([d(8), d(9)], t0), [d(8), d(9)]);
        // In flight: not handed out again.
        assert!(feeds.take_due_days([d(8), d(9)], t0).is_empty());
        feeds.day_succeeded(d(8), Rates::new(d(6), [(Currency::Usd, 1.1)]));
        assert_eq!(feeds.day_failed(d(9), t0), Duration::from_secs(5));
        // Stored: never again. Failed: only after the backoff.
        assert!(feeds.take_due_days([d(8), d(9)], t0 + Duration::from_secs(4)).is_empty());
        assert_eq!(feeds.take_due_days([d(8), d(9)], t0 + Duration::from_secs(5)), [d(9)]);
        // Stored days survive a restart.
        assert!(RateFeeds::load(dir.clone()).history.get(d(8)).is_some());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn failure_frees_the_source_and_backs_off() {
        let mut feeds = RateFeeds::load(scratch("fail"));
        feeds.take_due(d(11));
        assert_eq!(feeds.fail(Source::Fiat), Duration::from_secs(5));
        assert_eq!(feeds.fail(Source::Fiat), Duration::from_secs(10));
        assert_eq!(feeds.take_due(d(11)), [Source::Fiat]);
    }
}
