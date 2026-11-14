//! User settings persisted in `db/settings.toml`.

use std::path::Path;

use serde::{Deserialize, Serialize};

pub use super::currency::Currency;
use super::ledger::Interval;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    #[default]
    Dark,
    Light,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub currency: Currency,
    pub theme: Theme,
    pub interval: Interval,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            currency: Currency::Usd,
            theme: Theme::Dark,
            interval: Interval::Month,
        }
    }
}

impl Settings {
    /// Loads settings; a missing or broken file yields defaults. A non-fiat
    /// display currency is not allowed and falls back to the default.
    pub fn load(path: &Path) -> Settings {
        let mut s: Settings = std::fs::read_to_string(path)
            .ok()
            .and_then(|s| toml::from_str(&s).ok())
            .unwrap_or_default();
        if !s.currency.is_fiat() {
            s.currency = Currency::default();
        }
        s
    }

    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        let text = toml::to_string_pretty(self)?;
        std::fs::write(path, text)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toml_roundtrip() {
        let s = Settings {
            currency: Currency::Try,
            theme: Theme::Light,
            interval: Interval::Week,
        };
        let text = toml::to_string_pretty(&s).unwrap();
        assert!(text.contains("currency = \"TRY\""));
        assert!(text.contains("interval = \"1W\""));
        let back: Settings = toml::from_str(&text).unwrap();
        assert_eq!(back, s);
    }

    #[test]
    fn partial_file_uses_defaults() {
        let s: Settings = toml::from_str("currency = \"EUR\"").unwrap();
        assert_eq!(s.currency, Currency::Eur);
        assert_eq!(s.theme, Theme::Dark);
        assert_eq!(s.interval, Interval::Month);
    }
}
