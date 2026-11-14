//! Where prices come from and how their answers are read.
//!
//! `db/links.json` lists, for each feed, links tried in order until one
//! gives what the feed needs. Each link names the [`Format`] it answers in,
//! which picks the parser: a link in a known format needs no code, a new
//! format needs one parser here. The file is checked at startup, so a
//! format that cannot give what its feed needs, or a placeholder the feed
//! cannot fill, is an error then and not a wrong price later.
//!
//! Placeholders: `{day}` (`YYYY-MM-DD`; the day feeds, which must use it),
//! `{metal}` (`XAU` or `XAG`; the metal feed only) and `{ids}` (CoinGecko
//! ids of the known coins, comma-separated; any feed).
//!
//! Formats:
//! - `frankfurter`: ECB rates per EUR with their publication date.
//! - `gold-api`: one metal in USD per troy ounce; its link needs `{metal}`,
//!   since the answer does not say which metal it is.
//! - `coingecko`: `simple/price`; coins from their `usd` quote, metals from
//!   BTC's `usd` quote over its `xau` / `xag` quote.
//! - `currency-api`: units of each currency per 1 USD (`usd.json`), inverted.

use std::collections::HashMap;
use std::path::Path;

use anyhow::Context as _;
use chrono::NaiveDate;
use serde::Deserialize;

use super::currency::Currency;
use super::date::ISO_DATE;
use super::rates::{Rates, positive};

pub const LINKS_FILE: &str = "links.json";

/// How a link's answer is laid out; picks its parser.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Format {
    Frankfurter,
    GoldApi,
    Coingecko,
    CurrencyApi,
}

/// A kind of price an answer can hold.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Fiat,
    Metals,
    Crypto,
}

impl Kind {
    fn name(self) -> &'static str {
        match self {
            Kind::Fiat => "exchange rates",
            Kind::Metals => "metal prices",
            Kind::Crypto => "coin prices",
        }
    }
}

impl Format {
    fn name(self) -> &'static str {
        match self {
            Format::Frankfurter => "frankfurter",
            Format::GoldApi => "gold-api",
            Format::Coingecko => "coingecko",
            Format::CurrencyApi => "currency-api",
        }
    }

    /// The kinds of price this format's answers hold.
    fn gives(self) -> &'static [Kind] {
        match self {
            Format::Frankfurter => &[Kind::Fiat],
            Format::GoldApi => &[Kind::Metals],
            Format::Coingecko | Format::CurrencyApi => &[Kind::Metals, Kind::Crypto],
        }
    }

    /// Every usable price in `body`; missing or bad ones are left out (the
    /// feed then checks it got what it needs). `metal` is the metal asked
    /// for, which gold-api's answer does not name.
    fn parse(self, body: &str, metal: Option<Currency>) -> anyhow::Result<Rates> {
        match self {
            Format::Frankfurter => parse_frankfurter(body),
            Format::GoldApi => parse_gold_api(body, metal.context("gold-api is only for the metal feed")?),
            Format::Coingecko => parse_coingecko(body),
            Format::CurrencyApi => parse_currency_api(body),
        }
    }
}

/// One place to ask for prices.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Link {
    pub url: String,
    pub format: Format,
}

/// Every feed's links, best first (`db/links.json`).
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Links {
    /// Latest exchange rates.
    pub fiat: Vec<Link>,
    /// Latest price of one `{metal}`.
    pub metal: Vec<Link>,
    /// Latest coin prices.
    pub crypto: Vec<Link>,
    /// Exchange rates of a past `{day}`.
    pub fiat_day: Vec<Link>,
    /// Every metal and coin price of a past `{day}`.
    pub prices_day: Vec<Link>,
}

/// One list in [`Links`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Feed {
    Fiat,
    Metal,
    Crypto,
    FiatDay,
    PricesDay,
}

impl Feed {
    const ALL: [Feed; 5] = [Feed::Fiat, Feed::Metal, Feed::Crypto, Feed::FiatDay, Feed::PricesDay];

    /// Its key in `links.json`.
    fn key(self) -> &'static str {
        match self {
            Feed::Fiat => "fiat",
            Feed::Metal => "metal",
            Feed::Crypto => "crypto",
            Feed::FiatDay => "fiat_day",
            Feed::PricesDay => "prices_day",
        }
    }

    /// What every one of its links must be able to give.
    fn needs(self) -> &'static [Kind] {
        match self {
            Feed::Fiat | Feed::FiatDay => &[Kind::Fiat],
            Feed::Metal => &[Kind::Metals],
            Feed::Crypto => &[Kind::Crypto],
            Feed::PricesDay => &[Kind::Metals, Kind::Crypto],
        }
    }

    /// Placeholders it fills.
    fn fills(self) -> &'static [&'static str] {
        match self {
            Feed::Metal => &["metal", "ids"],
            Feed::FiatDay | Feed::PricesDay => &["day", "ids"],
            Feed::Fiat | Feed::Crypto => &["ids"],
        }
    }

    fn is_day(self) -> bool {
        matches!(self, Feed::FiatDay | Feed::PricesDay)
    }
}

impl Links {
    /// Reads and checks `links.json` in `db`.
    pub fn load(db: &Path) -> anyhow::Result<Links> {
        let path = db.join(LINKS_FILE);
        let text = std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
        let links: Links = serde_json::from_str(&text).with_context(|| format!("parsing {}", path.display()))?;
        links.check().with_context(|| format!("checking {}", path.display()))?;
        Ok(links)
    }

    fn of(&self, feed: Feed) -> &[Link] {
        match feed {
            Feed::Fiat => &self.fiat,
            Feed::Metal => &self.metal,
            Feed::Crypto => &self.crypto,
            Feed::FiatDay => &self.fiat_day,
            Feed::PricesDay => &self.prices_day,
        }
    }

    /// Every feed has a link, and every link can give what its feed needs
    /// with only the placeholders its feed fills.
    fn check(&self) -> anyhow::Result<()> {
        for feed in Feed::ALL {
            let key = feed.key();
            anyhow::ensure!(!self.of(feed).is_empty(), "`{key}` has no links");
            for Link { url, format } in self.of(feed) {
                for need in feed.needs() {
                    anyhow::ensure!(
                        format.gives().contains(need),
                        "`{key}`: {} answers hold no {}: {url}",
                        format.name(),
                        need.name()
                    );
                }
                for p in placeholders(url) {
                    anyhow::ensure!(feed.fills().contains(&p), "`{key}` cannot fill {{{p}}}: {url}");
                }
                anyhow::ensure!(!feed.is_day() || url.contains("{day}"), "`{key}` links need {{day}}: {url}");
                anyhow::ensure!(*format != Format::GoldApi || url.contains("{metal}"), "gold-api links need {{metal}}: {url}");
            }
        }
        Ok(())
    }
}

/// The names inside `{...}` in `url`.
fn placeholders(url: &str) -> Vec<&str> {
    url.split('{').skip(1).filter_map(|s| s.split_once('}').map(|(name, _)| name)).collect()
}

/// Placeholder values of one fetch.
#[derive(Debug, Default, Clone, Copy)]
struct Fill {
    day: Option<NaiveDate>,
    metal: Option<Currency>,
}

impl Fill {
    fn url(&self, template: &str) -> String {
        let ids: Vec<&str> = Currency::CRYPTO.iter().filter_map(|c| c.coingecko_id()).collect();
        let mut url = template.replace("{ids}", &ids.join(","));
        if let Some(day) = self.day {
            url = url.replace("{day}", &day.format(ISO_DATE).to_string());
        }
        if let Some(metal) = self.metal {
            url = url.replace("{metal}", metal.code());
        }
        url
    }
}

/// Fetches `links` in order and returns what `take` keeps of the first
/// usable answer. A link fails when it does not answer, answers badly, or
/// `take` rejects what it gave; when all fail, the error lists each reason.
fn first(
    links: &[Link],
    fill: Fill,
    get: &impl Fn(&str) -> anyhow::Result<String>,
    take: impl Fn(Rates) -> anyhow::Result<Rates>,
) -> anyhow::Result<Rates> {
    let mut errors = Vec::new();
    for link in links {
        let url = fill.url(&link.url);
        match get(&url).and_then(|body| link.format.parse(&body, fill.metal)).and_then(&take) {
            Ok(rates) => return Ok(rates),
            Err(e) => errors.push(format!("{url}: {e:#}")),
        }
    }
    anyhow::bail!("{}", errors.join("; "))
}

/// Codes of `currencies` that `rates` does not price.
fn missing(rates: &Rates, currencies: &[Currency]) -> Vec<&'static str> {
    currencies.iter().filter(|c| !rates.has(**c)).map(|c| c.code()).collect()
}

/// The whole fiat table, so a merge never drops a currency.
fn all_fiat(rates: Rates) -> anyhow::Result<Rates> {
    let missing = missing(&rates, &Currency::FIAT);
    anyhow::ensure!(missing.is_empty(), "no rate for {}", missing.join(", "));
    Ok(rates.fiat_only())
}

/// One independently fetched price feed. Each is fetched on its own so a
/// slow or failing one does not hold back the others.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Source {
    Fiat,
    Metal(Currency),
    Crypto,
}

impl Source {
    pub fn all() -> impl Iterator<Item = Source> {
        [Source::Fiat].into_iter().chain(Currency::METALS.map(Source::Metal)).chain([Source::Crypto])
    }

    pub fn name(self) -> &'static str {
        match self {
            Source::Fiat => "exchange rates",
            Source::Metal(m) => m.code(),
            Source::Crypto => "crypto prices",
        }
    }

    /// Cache file holding this feed (`.cache/<file>`). Both metals share one.
    pub fn cache_file(self) -> &'static str {
        match self {
            Source::Fiat => "exchange.json",
            Source::Metal(_) => "metal.json",
            Source::Crypto => "crypto.json",
        }
    }

    /// Key of this feed inside its cache file.
    pub fn cache_key(self) -> &'static str {
        match self {
            Source::Fiat => "fiat",
            Source::Metal(m) => m.code(),
            Source::Crypto => "crypto",
        }
    }

    /// Blocking HTTP fetch of this feed alone; run it off the UI thread.
    /// The result goes into the app's rates with [`Rates::merge`].
    pub fn fetch(self, links: &Links) -> anyhow::Result<Rates> {
        self.fetch_with(links, &http_get())
    }

    fn fetch_with(self, links: &Links, get: &impl Fn(&str) -> anyhow::Result<String>) -> anyhow::Result<Rates> {
        match self {
            Source::Fiat => first(&links.fiat, Fill::default(), get, all_fiat).context("no exchange rates"),
            Source::Metal(m) => first(&links.metal, Fill { metal: Some(m), ..Fill::default() }, get, |r| {
                anyhow::ensure!(r.has(m), "no {} price in the answer", m.code());
                Ok(r.prices(|c| c == m))
            })
            .with_context(|| format!("no {} price", m.code())),
            Source::Crypto => first(&links.crypto, Fill::default(), get, |r| {
                anyhow::ensure!(Currency::CRYPTO.iter().any(|c| r.has(*c)), "no coin price in the answer");
                Ok(r.prices(|c| Currency::CRYPTO.contains(&c)))
            })
            .context("no crypto prices"),
        }
    }
}

/// Blocking fetch of the rates as they were on a past `day`: exchange rates
/// (for ECB, the last publication on or before that day) plus every metal
/// and coin price, so a stored day is complete. Run it off the UI thread;
/// the result goes into a [`super::rates::History`].
pub fn fetch_day(links: &Links, day: NaiveDate) -> anyhow::Result<Rates> {
    fetch_day_with(links, day, &http_get())
}

fn fetch_day_with(links: &Links, day: NaiveDate, get: &impl Fn(&str) -> anyhow::Result<String>) -> anyhow::Result<Rates> {
    let fill = Fill { day: Some(day), ..Fill::default() };
    let mut rates = first(&links.fiat_day, fill, get, all_fiat).with_context(|| format!("no exchange rates for {day}"))?;
    let prices = first(&links.prices_day, fill, get, |r| {
        let missing = missing(&r, &[Currency::METALS.as_slice(), Currency::CRYPTO.as_slice()].concat());
        anyhow::ensure!(missing.is_empty(), "no price for {}", missing.join(", "));
        Ok(r.prices(|c| !c.is_fiat()))
    })
    .with_context(|| format!("no prices for {day}"))?;
    rates.merge(prices);
    Ok(rates)
}

/// Blocking GET returning the body. Bounded so a dead host cannot leave a
/// fetch pending forever.
fn http_get() -> impl Fn(&str) -> anyhow::Result<String> {
    let agent: ureq::Agent =
        ureq::Agent::config_builder().timeout_global(Some(std::time::Duration::from_secs(10))).build().into();
    move |url| Ok(agent.get(url).call()?.body_mut().read_to_string()?)
}

#[derive(Deserialize)]
struct FrankfurterAnswer {
    date: String,
    rates: HashMap<String, f64>,
}

/// Frankfurter `?base=EUR`: fiat per EUR. Unknown codes and non-positive
/// rates are skipped.
fn parse_frankfurter(body: &str) -> anyhow::Result<Rates> {
    let r: FrankfurterAnswer = serde_json::from_str(body).context("bad rates response")?;
    let date = NaiveDate::parse_from_str(&r.date, ISO_DATE).context("bad rates date")?;
    let per_eur = r
        .rates
        .into_iter()
        .filter(|(_, v)| positive(*v))
        .filter_map(|(code, v)| Some((Currency::from_code(&code).filter(|c| c.is_fiat())?, v)));
    Ok(Rates::new(date, per_eur))
}

#[derive(Deserialize)]
struct GoldApiAnswer {
    price: f64,
    currency: String,
}

/// gold-api.com `/price/<metal>`: USD per troy ounce of `metal`.
fn parse_gold_api(body: &str, metal: Currency) -> anyhow::Result<Rates> {
    let r: GoldApiAnswer = serde_json::from_str(body).context("bad metal price response")?;
    anyhow::ensure!(r.currency == "USD", "metal price not in USD: {}", r.currency);
    anyhow::ensure!(positive(r.price), "bad metal price: {}", r.price);
    Ok(Rates::default().with_metal(metal, r.price))
}

/// CoinGecko `simple/price`: each known coin's `usd` quote; each metal as
/// BTC in USD over BTC in ounces of it.
fn parse_coingecko(body: &str) -> anyhow::Result<Rates> {
    let r: HashMap<String, HashMap<String, f64>> = serde_json::from_str(body).context("bad CoinGecko response")?;
    let mut rates = Rates::default();
    for coin in Currency::CRYPTO {
        if let Some(usd) = coin.coingecko_id().and_then(|id| r.get(id)?.get("usd")) {
            rates = rates.with_crypto(coin, *usd);
        }
    }
    let btc = r.get("bitcoin");
    if let Some(usd) = btc.and_then(|q| q.get("usd")).copied().filter(|v| positive(*v)) {
        for metal in Currency::METALS {
            if let Some(ounces) = btc.and_then(|q| q.get(&metal.code().to_ascii_lowercase())).copied().filter(|v| positive(*v)) {
                rates = rates.with_metal(metal, usd / ounces);
            }
        }
    }
    Ok(rates)
}

#[derive(Deserialize)]
struct CurrencyApiAnswer {
    usd: HashMap<String, f64>,
}

/// currency-api `usd.json`: units per 1 USD, inverted into USD per ounce of
/// metal and per coin.
fn parse_currency_api(body: &str) -> anyhow::Result<Rates> {
    let r: CurrencyApiAnswer = serde_json::from_str(body).context("bad currency-api response")?;
    let per_usd = |c: Currency| r.usd.get(&c.code().to_ascii_lowercase()).copied().filter(|v| positive(*v));
    let mut rates = Rates::default();
    for metal in Currency::METALS {
        if let Some(ounces) = per_usd(metal) {
            rates = rates.with_metal(metal, 1.0 / ounces);
        }
    }
    for coin in Currency::CRYPTO {
        if let Some(coins) = per_usd(coin) {
            rates = rates.with_crypto(coin, 1.0 / coins);
        }
    }
    Ok(rates)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::rates::Fx;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    fn link(url: &str, format: Format) -> Link {
        Link { url: url.to_string(), format }
    }

    fn sample() -> Links {
        Links {
            fiat: vec![link("https://fiat.test/latest", Format::Frankfurter)],
            metal: vec![
                link("https://gold.test/{metal}", Format::GoldApi),
                link("https://gecko.test/metal", Format::Coingecko),
                link("https://daily-a.test/latest", Format::CurrencyApi),
                link("https://daily-b.test/latest", Format::CurrencyApi),
            ],
            crypto: vec![link("https://gecko.test/coins?ids={ids}", Format::Coingecko)],
            fiat_day: vec![link("https://fiat.test/{day}", Format::Frankfurter)],
            prices_day: vec![
                link("https://daily-a.test/{day}", Format::CurrencyApi),
                link("https://daily-b.test/{day}", Format::CurrencyApi),
            ],
        }
    }

    /// A fake GET answering only the given URLs.
    fn answers<'a>(up: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> anyhow::Result<String> + 'a {
        move |url| match up.iter().find(|(u, _)| *u == url) {
            Some((_, body)) => Ok(body.to_string()),
            None => anyhow::bail!("down"),
        }
    }

    fn repo_links() -> Links {
        Links::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("db")).unwrap()
    }

    #[test]
    fn repo_links_load() {
        repo_links();
    }

    #[test]
    fn check_rejects_what_a_feed_cannot_use() {
        assert!(sample().check().is_ok());
        let err = |edit: fn(&mut Links)| {
            let mut links = sample();
            edit(&mut links);
            format!("{:#}", links.check().unwrap_err())
        };
        assert!(err(|l| l.crypto[0].format = Format::GoldApi).contains("`crypto`: gold-api answers hold no coin prices"));
        assert!(err(|l| l.fiat[0].format = Format::Coingecko).contains("no exchange rates"));
        assert!(err(|l| l.fiat_day[0].url = "https://fiat.test/latest".into()).contains("`fiat_day` links need {day}"));
        assert!(err(|l| l.fiat[0].url = "https://fiat.test/{day}".into()).contains("`fiat` cannot fill {day}"));
        assert!(err(|l| l.metal[0].url = "https://gold.test/XAU".into()).contains("gold-api links need {metal}"));
        assert!(err(|l| l.prices_day.clear()).contains("`prices_day` has no links"));
        // Unknown formats and keys fail to parse.
        let bad_format = r#"{"url":"https://x.test","format":"yahoo"}"#;
        assert!(serde_json::from_str::<Link>(bad_format).is_err());
        assert!(serde_json::from_str::<Link>(r#"{"url":"https://x.test","format":"coingecko","extra":1}"#).is_err());
    }

    #[test]
    fn fill_replaces_the_placeholders() {
        let fill = Fill { day: Some(d(2026, 11, 13)), metal: Some(Currency::Xag) };
        assert_eq!(fill.url("https://x/{day}/{metal}?ids={ids}"), "https://x/2026-11-13/XAG?ids=bitcoin,ethereum");
        assert_eq!(Fill::default().url("https://x/{day}"), "https://x/{day}");
    }

    #[test]
    fn parse_frankfurter_answer() {
        let json = r#"{"amount":1.0,"base":"EUR","date":"2026-11-09",
            "rates":{"GBP":0.8343,"JPY":176.79,"TRY":56.1199,"USD":1.1193,"CHF":0.93,"XXX":-1,"XAU":0.0003}}"#;
        let r = Format::Frankfurter.parse(json, None).unwrap();
        assert_eq!(r.date, Some(d(2026, 11, 9)));
        assert!(missing(&r, &Currency::FIAT).is_empty());
        assert!(!r.has(Currency::Xau));
        assert!(Format::Frankfurter.parse("not json", None).is_err());
    }

    #[test]
    fn parse_gold_api_answer() {
        let json = r#"{"currency":"USD","currencySymbol":"$","exchangeRate":1.0,"name":"Gold",
            "price":4075.100098,"symbol":"XAU","updatedAt":"2026-11-10T11:25:32Z"}"#;
        let xau = Some(Currency::Xau);
        assert_eq!(Format::GoldApi.parse(json, xau).unwrap(), Rates::default().with_metal(Currency::Xau, 4075.100098));
        assert!(Format::GoldApi.parse(r#"{"currency":"EUR","price":1.0}"#, xau).is_err());
        assert!(Format::GoldApi.parse(r#"{"currency":"USD","price":0}"#, xau).is_err());
        assert!(Format::GoldApi.parse(r#"{"error":"Currency not found"}"#, xau).is_err());
        assert!(Format::GoldApi.parse(json, None).is_err());
    }

    #[test]
    fn parse_coingecko_answer() {
        let coins = r#"{"bitcoin":{"usd":74211},"ethereum":{"usd":2090.1},"dogecoin":{"usd":0.2}}"#;
        let expected = Rates::default().with_crypto(Currency::Btc, 74211.0).with_crypto(Currency::Eth, 2090.1);
        assert_eq!(Format::Coingecko.parse(coins, None).unwrap(), expected);
        let metals = r#"{"bitcoin":{"usd":80000,"xau":20,"xag":1600}}"#;
        let expected = Rates::default()
            .with_crypto(Currency::Btc, 80000.0)
            .with_metal(Currency::Xau, 4000.0)
            .with_metal(Currency::Xag, 50.0);
        assert_eq!(Format::Coingecko.parse(metals, None).unwrap(), expected);
        // Bad or missing quotes are left out, not an error.
        let bad = Format::Coingecko.parse(r#"{"bitcoin":{"usd":80000,"xau":0}}"#, None).unwrap();
        assert!(!bad.has(Currency::Xau) && !bad.has(Currency::Xag));
        assert!(Format::Coingecko.parse("[]", None).is_err());
    }

    #[test]
    fn parse_currency_api_answer() {
        let json = r#"{"date":"2026-11-09","usd":{"xau":0.00025,"xag":0.02,"btc":0.0000125,"eth":0.0005,"eur":0.9}}"#;
        let mut got = Rates::new(NaiveDate::MIN, [(Currency::Usd, 1.0)]);
        got.merge(Format::CurrencyApi.parse(json, None).unwrap());
        let usd = Fx::new(Currency::Usd, Some(&got));
        // 1 oz gold = 4.000 USD, so 1 g = 128,60 USD; 1 BTC = 80.000 USD; 1 ETH = 2.000 USD.
        assert_eq!(usd.convert(100, Currency::Xau), Some(12_860));
        assert_eq!(usd.convert(100_000_000, Currency::Btc), Some(8_000_000));
        assert_eq!(usd.convert(100_000_000, Currency::Eth), Some(200_000));
        let partial = Format::CurrencyApi.parse(r#"{"usd":{"xau":-1,"xag":0.02}}"#, None).unwrap();
        assert!(!partial.has(Currency::Xau) && partial.has(Currency::Xag));
    }

    #[test]
    fn metal_falls_back_in_order() {
        let links = sample();
        let gold_api = r#"{"currency":"USD","price":4100}"#;
        let gecko = r#"{"bitcoin":{"usd":80000,"xau":20,"xag":1600}}"#;
        let daily = r#"{"usd":{"xau":0.0005,"xag":0.04}}"#;
        let fetch = |up: &[(&str, &str)]| Source::Metal(Currency::Xau).fetch_with(&links, &answers(up));
        let gold = |usd_per_ounce: f64| Rates::default().with_metal(Currency::Xau, usd_per_ounce);
        let all = [
            ("https://gold.test/XAU", gold_api),
            ("https://gecko.test/metal", gecko),
            ("https://daily-a.test/latest", daily),
        ];
        assert_eq!(fetch(&all).unwrap(), gold(4100.0));
        // Only the asked metal is kept, though CoinGecko priced both.
        assert_eq!(fetch(&all[1..]).unwrap(), gold(4000.0));
        // A bad answer counts as a failure, not a price.
        let bad = [("https://gold.test/XAU", "{}"), ("https://gecko.test/metal", "{}"), all[2]];
        assert_eq!(fetch(&bad).unwrap(), gold(2000.0));
        // The last link is the last resort.
        assert_eq!(fetch(&[("https://daily-b.test/latest", daily)]).unwrap(), gold(2000.0));
        let err = format!("{:#}", fetch(&[]).unwrap_err());
        assert!(err.starts_with("no XAU price"), "{err}");
        for l in &links.metal {
            assert!(err.contains(&Fill { metal: Some(Currency::Xau), ..Fill::default() }.url(&l.url)), "{err}");
        }
    }

    #[test]
    fn fiat_needs_the_whole_table() {
        let links = sample();
        let full = r#"{"date":"2026-11-13","rates":{"USD":1.1,"GBP":0.8,"TRY":50.0,"JPY":170.0}}"#;
        let short = r#"{"date":"2026-11-13","rates":{"USD":1.1}}"#;
        let r = Source::Fiat.fetch_with(&links, &answers(&[("https://fiat.test/latest", full)])).unwrap();
        assert!(missing(&r, &Currency::FIAT).is_empty());
        let err = format!("{:#}", Source::Fiat.fetch_with(&links, &answers(&[("https://fiat.test/latest", short)])).unwrap_err());
        assert!(err.contains("no rate for"), "{err}");
    }

    #[test]
    fn crypto_keeps_only_coins() {
        let links = sample();
        let body = r#"{"bitcoin":{"usd":80000,"xau":20},"ethereum":{"usd":2000}}"#;
        let r = Source::Crypto.fetch_with(&links, &answers(&[("https://gecko.test/coins?ids=bitcoin,ethereum", body)])).unwrap();
        assert_eq!(r, Rates::default().with_crypto(Currency::Btc, 80000.0).with_crypto(Currency::Eth, 2000.0));
    }

    #[test]
    fn past_day_needs_every_price() {
        let links = sample();
        let fiat = r#"{"date":"2026-11-06","rates":{"USD":1.1,"GBP":0.8,"TRY":50.0,"JPY":170.0}}"#;
        let no_eth = r#"{"usd":{"xau":0.00025,"xag":0.02,"btc":0.0000125}}"#;
        let full = r#"{"usd":{"xau":0.00025,"xag":0.02,"btc":0.0000125,"eth":0.0005}}"#;
        let up = [
            ("https://fiat.test/2026-11-09", fiat),
            ("https://daily-a.test/2026-11-09", no_eth),
            ("https://daily-b.test/2026-11-09", full),
        ];
        let r = fetch_day_with(&links, d(2026, 11, 9), &answers(&up)).unwrap();
        assert_eq!(r.date, Some(d(2026, 11, 6)));
        assert!(missing(&r, &Currency::ALL).is_empty());
        let err = format!("{:#}", fetch_day_with(&links, d(2026, 11, 9), &answers(&up[..2])).unwrap_err());
        assert!(err.starts_with("no prices for 2026-11-09") && err.contains("no price for ETH"), "{err}");
    }

    /// Hits the real APIs: `cargo test live_fetch -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn live_fetch() {
        let links = repo_links();
        let mut r = Rates::default();
        for s in Source::all() {
            let t = std::time::Instant::now();
            match s.fetch(&links) {
                Ok(part) => r.merge(part),
                Err(e) => println!("{} failed: {e:#}", s.name()),
            }
            println!("{} took {:?}", s.name(), t.elapsed());
        }
        let usd = Fx::new(Currency::Usd, Some(&r));
        let try_ = Fx::new(Currency::Try, Some(&r));
        println!("ECB date {:?}", r.date);
        // Fiat is the one feed totals cannot do without; the others only
        // print, since a third-party outage there is not our bug.
        assert!(usd.convert(100, Currency::Try).is_some());
        for m in Currency::METALS {
            // 1,00 g in USD and TRY.
            println!("1 g {}: {:?} USD cents, {:?} TRY kuruş", m.code(), usd.convert(100, m), try_.convert(100, m));
        }
        for c in Currency::CRYPTO {
            let one = 100_000_000; // 1 coin
            println!("1 {}: {:?} USD cents, {:?} TRY kuruş", c.code(), usd.convert(one, c), try_.convert(one, c));
        }
    }

    /// Hits the real day sources: `cargo test live_fetch_day -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn live_fetch_day() {
        let day = chrono::Local::now().date_naive() - chrono::Days::new(3);
        let r = fetch_day(&repo_links(), day).unwrap();
        let usd = Fx::new(Currency::Usd, Some(&r));
        println!("{day}: ECB {:?}; 1 g XAU {:?} USD cents; 1 BTC {:?} USD cents", r.date, usd.convert(100, Currency::Xau), usd.convert(100_000_000, Currency::Btc));
        assert!(usd.convert(100, Currency::Xau).is_some() && usd.convert(100, Currency::Try).is_some());
    }

    /// Tries every real link on its own: `cargo test live_each_link -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn live_each_link() {
        let links = repo_links();
        let get = http_get();
        let alone = |l: &Link| Links {
            fiat: vec![l.clone()],
            metal: vec![l.clone()],
            crypto: vec![l.clone()],
            fiat_day: vec![l.clone()],
            prices_day: vec![l.clone()],
        };
        let day = chrono::Local::now().date_naive() - chrono::Days::new(3);
        for l in &links.fiat {
            println!("fiat {}: {:?}", l.url, Source::Fiat.fetch_with(&alone(l), &get).map(|r| r.date));
        }
        for l in &links.metal {
            for m in Currency::METALS {
                println!("{} {}: {:?}", m.code(), l.url, Source::Metal(m).fetch_with(&alone(l), &get).map(|r| r.has(m)));
            }
        }
        for l in &links.crypto {
            println!("crypto {}: {:?}", l.url, Source::Crypto.fetch_with(&alone(l), &get).map(|_| ()));
        }
        for l in links.fiat_day.iter().chain(&links.prices_day) {
            let mut one = alone(l);
            // Each day link alone, with the first link of the other day feed.
            if l.format == Format::Frankfurter {
                one.prices_day = vec![links.prices_day[0].clone()];
            } else {
                one.fiat_day = vec![links.fiat_day[0].clone()];
            }
            println!("{day} {}: {:?}", l.url, fetch_day_with(&one, day, &get).is_ok());
        }
    }
}
