//! Supported currencies: fiat money, gold and silver, Bitcoin and Ether.
//!
//! Gold (`XAU`) and silver (`XAG`) amounts are grams, kept in hundredths
//! like cents. Crypto amounts are coins with 8 decimals. Everything but
//! fiat is for asset rows only and is never a display currency.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "UPPERCASE")]
pub enum Currency {
    #[default]
    Usd,
    Eur,
    Gbp,
    Try,
    Jpy,
    Xau,
    Xag,
    Btc,
    Eth,
}

impl Currency {
    /// Money: display currencies and the currencies of incomes/expenses.
    pub const FIAT: [Currency; 5] = [
        Currency::Usd,
        Currency::Eur,
        Currency::Gbp,
        Currency::Try,
        Currency::Jpy,
    ];

    /// Gold and silver, in grams.
    pub const METALS: [Currency; 2] = [Currency::Xau, Currency::Xag];

    /// Crypto, in coins (8 decimals).
    pub const CRYPTO: [Currency; 2] = [Currency::Btc, Currency::Eth];

    /// Everything an asset row can be in.
    pub const ALL: [Currency; 9] = [
        Currency::Usd,
        Currency::Eur,
        Currency::Gbp,
        Currency::Try,
        Currency::Jpy,
        Currency::Xau,
        Currency::Xag,
        Currency::Btc,
        Currency::Eth,
    ];

    pub fn is_fiat(self) -> bool {
        Currency::FIAT.contains(&self)
    }

    pub fn is_metal(self) -> bool {
        matches!(self, Currency::Xau | Currency::Xag)
    }
    /// Decimal places of an amount in this currency (its minor unit).
    pub fn decimals(self) -> u32 {
        match self {
            Currency::Btc | Currency::Eth => 8,
            _ => 2,
        }
    }

    /// CoinGecko coin id for crypto.
    pub fn coingecko_id(self) -> Option<&'static str> {
        match self {
            Currency::Btc => Some("bitcoin"),
            Currency::Eth => Some("ethereum"),
            _ => None,
        }
    }

    pub fn symbol(self) -> &'static str {
        match self {
            Currency::Usd => "$",
            Currency::Eur => "€",
            Currency::Gbp => "£",
            Currency::Try => "₺",
            Currency::Jpy => "¥",
            Currency::Xau | Currency::Xag => "g",
            Currency::Btc => "₿",
            Currency::Eth => "Ξ",
        }
    }

    /// ISO 4217 code, e.g. `USD`, `XAU`.
    pub fn code(self) -> &'static str {
        match self {
            Currency::Usd => "USD",
            Currency::Eur => "EUR",
            Currency::Gbp => "GBP",
            Currency::Try => "TRY",
            Currency::Jpy => "JPY",
            Currency::Xau => "XAU",
            Currency::Xag => "XAG",
            Currency::Btc => "BTC",
            Currency::Eth => "ETH",
        }
    }

    /// Parses an ISO code, ignoring case and surrounding whitespace.
    pub fn from_code(code: &str) -> Option<Currency> {
        let code = code.trim();
        Currency::ALL.into_iter().find(|c| c.code().eq_ignore_ascii_case(code))
    }

    /// First of `options` whose code starts with `text` (case-insensitive),
    /// so `e` picks EUR. Empty text matches nothing.
    pub fn match_prefix(text: &str, options: &[Currency]) -> Option<Currency> {
        let text = text.trim().to_ascii_uppercase();
        if text.is_empty() {
            return None;
        }
        options.iter().copied().find(|c| c.code().starts_with(&text))
    }

    /// Dropdown label, e.g. `USD $`.
    pub fn label(self) -> String {
        format!("{} {}", self.code(), self.symbol())
    }

    /// Display-currency dropdown lookup (fiat only).
    pub fn from_label(label: &str) -> Option<Currency> {
        Currency::FIAT.into_iter().find(|c| c.label() == label)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels() {
        assert_eq!(Currency::Gbp.label(), "GBP £");
        assert_eq!(Currency::from_label("JPY ¥"), Some(Currency::Jpy));
        assert_eq!(Currency::from_label("XAU g"), None);
    }

    #[test]
    fn codes() {
        assert_eq!(Currency::from_code("eur"), Some(Currency::Eur));
        assert_eq!(Currency::from_code(" TRY "), Some(Currency::Try));
        assert_eq!(Currency::from_code("xau"), Some(Currency::Xau));
        assert_eq!(Currency::from_code("CHF"), None);
        assert_eq!(Currency::from_code(""), None);
        for c in Currency::ALL {
            assert_eq!(Currency::from_code(c.code()), Some(c));
        }
    }

    #[test]
    fn prefix_match() {
        assert_eq!(Currency::match_prefix("e", &Currency::FIAT), Some(Currency::Eur));
        assert_eq!(Currency::match_prefix(" usd ", &Currency::FIAT), Some(Currency::Usd));
        assert_eq!(Currency::match_prefix("x", &Currency::FIAT), None);
        assert_eq!(Currency::match_prefix("xa", &Currency::ALL), Some(Currency::Xau));
        assert_eq!(Currency::match_prefix("", &Currency::ALL), None);
    }

    #[test]
    fn groups() {
        assert!(Currency::METALS.iter().all(|c| c.is_metal() && !c.is_fiat()));
        assert!(Currency::CRYPTO.iter().all(|c| c.coingecko_id().is_some() && !c.is_fiat()));
        assert!(Currency::FIAT.iter().all(|c| c.is_fiat() && c.decimals() == 2));
        assert_eq!(Currency::Btc.decimals(), 8);        assert_eq!(
            Currency::FIAT.len() + Currency::METALS.len() + Currency::CRYPTO.len(),
            Currency::ALL.len()
        );
    }
}
