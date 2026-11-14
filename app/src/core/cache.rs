//! On-disk cache of fetched price feeds in `.cache/`: `exchange.json`,
//! `metal.json` and `crypto.json`. Each file maps a feed key to the day it
//! was fetched and the rates it returned, so a feed is fetched at most once
//! a day and the last known prices show at once on start. Past days' rates
//! live in `history.json`, keyed by day, and are kept for good.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::Context as _;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use super::date::ISO_DATE;
use super::rates::{History, Rates, RatesData};
use super::store;
use super::sources::Source;

const HISTORY_FILE: &str = "history.json";

/// A feed as last cached.
pub struct Cached {
    /// Local day the feed was fetched.
    pub fetched: NaiveDate,
    pub rates: Rates,
}

#[derive(Serialize, Deserialize)]
struct Entry {
    fetched: String,
    rates: RatesData,
}

type File = BTreeMap<String, Entry>;

/// Where `.cache/` lives: next to `db/`, so the repo root in debug builds
/// and the exe folder in release.
pub fn dir() -> PathBuf {
    if cfg!(debug_assertions) {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join(".cache")
    } else {
        store::exe_dir().join(".cache")
    }
}

fn read(path: &Path) -> anyhow::Result<File> {
    let text = std::fs::read_to_string(path)?;
    serde_json::from_str(&text).context("bad cache file")
}

/// The cached feed, or `None` when it is missing or unreadable.
pub fn load(dir: &Path, source: Source) -> Option<Cached> {
    let entry = read(&dir.join(source.cache_file())).ok()?.remove(source.cache_key())?;
    let fetched = NaiveDate::parse_from_str(&entry.fetched, ISO_DATE).ok()?;
    Some(Cached { fetched, rates: Rates::from_data(entry.rates) })
}

/// Records `rates` as the feed's content, fetched on `fetched`. Other feeds
/// in the same file are kept. The file is replaced atomically.
pub fn store(dir: &Path, source: Source, rates: &Rates, fetched: NaiveDate) -> anyhow::Result<()> {
    std::fs::create_dir_all(dir)?;
    let path = dir.join(source.cache_file());
    // A missing or corrupt file just starts over.
    let mut file = read(&path).unwrap_or_default();
    file.insert(
        source.cache_key().to_string(),
        Entry { fetched: fetched.format(ISO_DATE).to_string(), rates: rates.to_data() },
    );
    write_atomic(&path, &serde_json::to_string_pretty(&file)?)
}

fn write_atomic(path: &Path, text: &str) -> anyhow::Result<()> {
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, text)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

/// Past days' rates; empty when the file is missing or unreadable.
pub fn load_history(dir: &Path) -> History {
    std::fs::read_to_string(dir.join(HISTORY_FILE))
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .map(History::from_data)
        .unwrap_or_default()
}

/// Replaces `history.json` with `history`, atomically.
pub fn store_history(dir: &Path, history: &History) -> anyhow::Result<()> {
    std::fs::create_dir_all(dir)?;
    write_atomic(&dir.join(HISTORY_FILE), &serde_json::to_string_pretty(&history.to_data())?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::currency::Currency;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("balance-cache-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn missing_cache_is_none() {
        let dir = scratch("missing");
        assert!(load(&dir, Source::Fiat).is_none());
    }

    #[test]
    fn store_then_load() {
        let dir = scratch("roundtrip");
        let fiat = Rates::new(d(2026, 11, 9), [(Currency::Usd, 1.25)]);
        store(&dir, Source::Fiat, &fiat, d(2026, 11, 11)).unwrap();
        let got = load(&dir, Source::Fiat).unwrap();
        assert_eq!(got.fetched, d(2026, 11, 11));
        assert_eq!(got.rates, fiat);
        assert!(dir.join("exchange.json").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn metals_share_a_file_without_clobbering() {
        let dir = scratch("metals");
        let gold = Rates::default().with_metal(Currency::Xau, 4_000.0);
        let silver = Rates::default().with_metal(Currency::Xag, 50.0);
        store(&dir, Source::Metal(Currency::Xau), &gold, d(2026, 11, 10)).unwrap();
        store(&dir, Source::Metal(Currency::Xag), &silver, d(2026, 11, 11)).unwrap();
        let g = load(&dir, Source::Metal(Currency::Xau)).unwrap();
        let s = load(&dir, Source::Metal(Currency::Xag)).unwrap();
        assert_eq!((g.fetched, g.rates), (d(2026, 11, 10), gold));
        assert_eq!((s.fetched, s.rates), (d(2026, 11, 11), silver));
        assert!(dir.join("metal.json").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn history_store_then_load() {
        let dir = scratch("history");
        assert_eq!(load_history(&dir), History::default());
        let mut h = History::default();
        h.insert(d(2026, 11, 8), Rates::new(d(2026, 11, 6), [(Currency::Usd, 1.1)]).with_metal(Currency::Xau, 4_000.0));
        store_history(&dir, &h).unwrap();
        assert_eq!(load_history(&dir), h);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn corrupt_file_is_ignored_and_replaced() {
        let dir = scratch("corrupt");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("crypto.json"), "not json").unwrap();
        assert!(load(&dir, Source::Crypto).is_none());
        let btc = Rates::default().with_crypto(Currency::Btc, 80_000.0);
        store(&dir, Source::Crypto, &btc, d(2026, 11, 11)).unwrap();
        assert_eq!(load(&dir, Source::Crypto).unwrap().rates, btc);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
