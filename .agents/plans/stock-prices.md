# Stock prices (STCK) — temporary plan

Status: draft, to be reviewed before implementation. Nothing here is decided until we go through it.

Background research:
- `.agents/research/stock-price-apis.md`: price sources, fallback chains per market, feed design.
- `.agents/research/rust-http-clients.md`: HTTP client choice, Yahoo blocking behaviour, escalation ladder.

## Goal

Asset rows can hold stocks/ETFs: a share count plus a ticker. Their value on any day is
`shares × close(ticker, day)` in the quote currency, converted to the display currency with
the existing fiat rates. Investments shows the ticker; change and profit work like other holdings.

## Decisions to confirm

- Currency code `STCK` as a new `Currency` variant (not fiat/metal/crypto; assets only).
- Shares stored with 4 decimals (fractional shares).
- Tickers entered in Yahoo format: `THYAO.IS`, `AAPL`, `SAP.DE`, `SHEL.L`.
  Later: canonical (ticker, MIC) key with a per-provider suffix table.
- Quote currency must be one of the supported fiats (USD, EUR, GBP, TRY, JPY);
  anything else shows as `Unavailable` until more fiats are added.
- First version uses Yahoo only; fallbacks come later (see step 6).
- Close used for valuation = raw `quote.close`, never `adjclose`.

## Data model

- `Currency::Stck`: `decimals() = 4`, `is_fiat() = false`, in `Currency::ALL`, excluded from
  income/expense parsing.
- `Snapshot.ticker: Option<String>`; required when `currency == Stck`, ignored otherwise.
- Assets sheet: optional 6th column `Ticker` (after `Name | Category | Value | Date | Currency`).
  Old workbooks load unchanged; rows with `STCK` and no ticker are ignored (or flagged).
- `Rates.stock: HashMap<String, Quote>` with `Quote { price: f64, currency: Currency }` =
  one share's close in its quote currency. `RatesData` gets the string-keyed equivalent, so
  `History` (one `Rates` per day in `.cache/history.json`) stores past closes for free.
- `Fx::value(&Snapshot) -> Option<Cents>`: for `Stck`, shares × quote price → quote-currency
  minor units → existing fiat conversion; otherwise `convert(value, currency)`.
  Switch callers: `assets::categories`, ledger partial/change, `investments`.

## Fetching (`core/stocks.rs`)

- Endpoint: `GET https://query2.finance.yahoo.com/v8/finance/chart/{T}?range=1y&interval=1d&events=div,splits`
  (query1 as alternate host). No cookie, no crumb.
- HTTP client: existing ureq agent, upgraded 3.5.1 → 3.5.2, rustls + ring.
- User-Agent: honest, non-library, e.g. `Mozilla/5.0 (compatible; balance/<version>)`.
  Never curl-like, library-default or empty (instant 429).
- Parse into `Chart { currency, regular_market_price, bars: Vec<(NaiveDate, f64)> }`:
  - dates in exchange-local time via `meta.gmtoffset`;
  - skip `null`/≤0 closes; drop bars after the requested end;
  - sub-units `GBp`/`ZAc`/`ILA` × 0.01 → GBP/ZAR/ILS, unless `regularMarketPrice / last close`
    shows the series is already in major units (yfinance's 10% check);
  - validate shape/content, not just status (HTML with 200, `chart.error.description`).
- One call backfills a year: every bar → `history[day].stock[T]`. Valuation on day D uses the
  latest bar on or before D (weekends/holidays). A gap of several trading days → ask again later.
- Today's price: `meta.regularMarketPrice` → live `Rates`, refreshed with the other rates.
  Today's bar is provisional until the exchange's close time; don't freeze it in history early.
- Errors:
  - 404 → unknown ticker; stop, show invalid.
  - instant 429 on a session's first request → client classified; escalate (see below).
  - 429 after many requests / 5xx / timeout → existing backoff (5 s doubling, cap 300 s).
- Requests serial, low volume; per-source circuit breaker (Wealthfolio pattern: open after 5
  failures, retry after 60 s, close after 2 successes).

## Escalation ladder (only when needed)

1. ureq + rustls + honest UA (default).
2. Same agent with `TlsProvider::NativeTls` (SChannel), compiled in as a runtime switch.
3. Optional: prebuilt libcurl-impersonate MSVC DLL via our own small FFI (blocking, same
   engine as yfinance/curl_cffi). Only if Yahoo returns to fingerprint blocking.
   Not wreq (needs tokio + CMake/NASM/libclang), not impit (patches rustls/hyper), not reqwest.

## UI

- Asset editor: `STCK` in the currency picker; picking it shows a Ticker field and labels value
  as shares. Validate ticker non-empty.
- Investments: Ticker column shows the ticker; Value shows shares, Type shows `STCK`.
- Assets ring/change: automatic once callers use `Fx::value`.
- Status: label stock prices "Delayed" / "Close as of {date}"; settings line
  "Prices: Yahoo Finance (unofficial)".

## Steps (each compiles and passes tests)

1. `Currency::Stck`, `Snapshot.ticker`, Assets sheet `Ticker` column, read/write round trip.
2. `core/stocks.rs` chart parser + tests on recorded JSON (THYAO.IS, AAPL, SHEL.L in GBp,
   404, 429, HTML-with-200).
3. `Rates.stock` + `History` persistence + `Fx::value`; switch all callers; valuation tests
   (past day, weekend/holiday, missing quote → Unavailable).
4. Fetch wiring in `ui/app/rates.rs`: tickers needed → fetch once per ticker, fill history,
   backoff; ignored live test against Yahoo.
5. Editor + Investments UI.
6. Later: İş Yatırım (`OneEndeks`, `HisseTekil`, 15 s timeout, check `ok`) for `.IS`;
   Nasdaq API for US; TradingView scanner for current BIST; SChannel switch; BYOK
   (EODHD/Tiingo) slot.

## Checks after dependency changes

- `cargo tree -e features -i native-tls`, `-i rustls`, `-i aws-lc-rs` — feature unification has
  silently changed TLS stacks in other projects and brought back Yahoo 429s.

## Open questions

- Splits: raw close × shares held on that day is correct only if the user records split share
  counts; how to handle (manual snapshot vs auto from `events.splits`)?
- Is Yahoo's `quote.close` unadjusted around splits? Verify against a known split.
- Should unsupported quote currencies (CHF, SEK, …) be added to `Currency`, or stay Unavailable?
- PROJECT.md updates (data model, Investments ticker column, sheet format).
