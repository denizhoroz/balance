# Keyed stock/ETF price APIs with a free tier (as of 2026-11-12)

Scope: providers that need an API key but offer a free tier, judged for a Rust desktop app with no backend (blocking `ureq`). The key either ships in the binary or is supplied by each user (BYOK). Needs: (a) latest price, (b) daily close on a past date (mostly within the last year), for BIST (THYAO, TRY), US (AAPL, VOO) and major EU (Xetra, Euronext, LSE).

Method: official pricing, docs and ToS pages, fetched 2026-11-12, plus live keyless/demo `curl` calls made 2026-11-12 around 10:15 UTC. No accounts were created. Live results are labelled "LIVE 2026-11-12". Pricing-page figures come from pages fetched that day unless noted otherwise.

## Q1. Free-tier limits, and is a free key per-user signup only?

### Takeaway
Every provider needs a per-account signup to get a real key. The free quotas range from Marketstack's 100 requests/month up to Alpaca's 200/min. Only Alpha Vantage, Twelve Data and EODHD expose a useful public `demo` key, and those keys are limited to a handful of US sample symbols.

### Summary matrix (free tier only)

| Provider | Free limit | BIST on free? | EU on free? | US on free? | Past-date daily close on free? | Embedding a shared key allowed? |
|---|---|---|---|---|---|---|
| Twelve Data (Basic) | 8 credits/min, 800/day | **No.** Only THYAO itself is a Basic sample symbol; the exchange needs Grow | **No.** One sample symbol per exchange (e.g. VOW3 on Xetra); the rest need Grow | Yes (all US stocks/ETFs) | Yes, `time_series?date=` | No (free tier is non-commercial; credentials may not be shared; redistribution needs an add-on) |
| Alpha Vantage | 25 req/day | Unverified | LSE (`.LON`) and Xetra (`.DEX`) documented; free-key access not tested | Yes | Only last 100 days (`compact`); `full` is premium | No (personal, non-commercial licence) |
| EODHD | 20 calls/day (+ welcome bonus), 1 year of history | "Any ticker" per EODHD's llms.txt; BIST code not verified | Yes (any ticker, past year) | Yes | Yes, `/eod/{SYM}?from=&to=` | No (no redistribution/display for non-professional users) |
| Finnhub | 60 calls/min | No (international = Enterprise) | No | Real-time quote only | **No** (candles are premium) | No (personal use; no sharing) |
| FMP (Basic) | 250 calls/day, 500 MB per trailing 30 days | No | No | Only about 87 sample symbols (AAPL, TSLA, AMZN + 84) | Sample symbols only | No (explicitly forbids embedding in apps used by others and sharing the API key) |
| Massive (ex-Polygon) Stocks Basic | 5 calls/min, 2 years, EOD | No (US only) | No | Yes, EOD | Yes (aggregates) | No (individual, non-business use; no shared credentials) |
| Tiingo (Free) | 50 req/hr, 1,000/day, 500 unique symbols/month, 1 GB/month | No (US + China only) | No | Yes | Yes, `prices?startDate=&endDate=` | Shared key: no. **BYOK explicitly allowed** with attribution |
| Marketstack (Free) | 100 req/month, 1 year, EOD only | Unverified | Unverified (v2 EOD list cites US/China exchanges) | Yes | Yes | No ("Non-Commercial Use") |
| MarketData.app (Free Forever) | 100 credits/day, 1 year, 24h delayed | No (US) | No | Yes (24h delayed) | Yes, `candles/D/` | No (individual, personal use; no redistribution) |
| Alpaca (Basic) | 200 calls/min, IEX feed, history since 2016 | No (US only) | No | Yes (IEX, 15-min rule) | Yes (bars) | Unverified (ToS not checked) |
| YH Finance on RapidAPI (apidojo) | 500 req/month free | Unofficial Yahoo scrape | Unofficial | Unofficial | Unofficial | Third-party wrapper of unofficial Yahoo data; high ToS risk |
| Nasdaq Data Link | Free key, but the free equity table (WIKI) is frozen since 2018 | No | No | No live free EOD | No | n/a |
| IEX Cloud | **Dead**: shut down 2024-08-31 | – | – | – | – | – |

### Cited Findings
- **Twelve Data Basic** is "$0/month", with "8 credits per minute and 800 per day" and "Internal non-display usage" — [twelvedata.com/pricing](https://twelvedata.com/pricing). The pricing page lists Grow at $79/mo on its card but $29 in its structured data, and says "20+" markets in one place and "27" in another. Those are inconsistencies on the page itself — [twelvedata.com/pricing](https://twelvedata.com/pricing).
- **Twelve Data demo key**: `apikey=demo` works for AAPL `quote` and `time_series` (LIVE 2026-11-12). For THYAO it returns `401 "The 'demo' API key is only used for initial familiarity..."` (LIVE 2026-11-12) — [api.twelvedata.com](https://api.twelvedata.com/quote?symbol=AAPL&apikey=demo).
- **Alpha Vantage free**: "our standard API usage limit (25 API requests per day)". Premium plans start at $49.99/mo (75 req/min) and go up to $249.99/mo (1,200 req/min) — [alphavantage.co/premium](https://www.alphavantage.co/premium/).
- **Alpha Vantage demo key**: works for IBM `GLOBAL_QUOTE` and `TIME_SERIES_DAILY` and for `SYMBOL_SEARCH&keywords=tesco` (LIVE 2026-11-12). Other symbols (THYAO.IS, TSCO.LON) return `"The **demo** API key is for demo purposes only..."`. The demo key is also throttled: "Please query the demo URLs at no more than 2 requests per second." (LIVE 2026-11-12) — [alphavantage.co](https://www.alphavantage.co/query?function=GLOBAL_QUOTE&symbol=IBM&apikey=demo).
- **EODHD free**: "20 API calls per day (plus a one-time 500 welcome bonus API calls)", "1 year" of history; global data is 15-min delayed — [eodhd.com/pricing](https://eodhd.com/pricing).
- EODHD's llms.txt describes the Free plan as "EOD/splits/dividends/forex/crypto for any ticker, past year only; no fundamentals, no intraday". The paid "EOD Historical Data — All World" plan is $19.99/mo or $199/yr — [eodhd.com/llms-full.txt](https://eodhd.com/llms-full.txt).
- **EODHD call costs**: "1 call" for EOD and live quotes (per ticker); errors cost 0. Free-plan limits reset at midnight GMT — [eodhd.com/llms-full.txt](https://eodhd.com/llms-full.txt).
- **EODHD demo key** is "Limited to these tickers: AAPL.US, MSFT.US, TSLA.US, AMZN.US, VTI.US, SWPPX.US, EURUSD.FOREX, BTC-USD.CC". Other symbols return `403 Forbidden` — [eodhd.com/llms-full.txt](https://eodhd.com/llms-full.txt). Confirmed live: `THYAO.IS` with `demo` returned `Forbidden` (LIVE 2026-11-12).
- **Finnhub free** is 60 API calls/minute. The figure comes from search snippets of the official pricing page; the JS-rendered page could not be read directly, so treat it as partially verified — [finnhub.io/pricing](https://finnhub.io/pricing). Finnhub has no demo key; keyless calls return `{"error":"Please use an API key."}` (LIVE 2026-11-12).
- **FMP Basic**: "$0/month ... 250 Calls / Day End of Day Historical". Bandwidth is capped at "Free plan - 500MB" per trailing 30 days. "Stock Quote (Basic): Symbol Limited to AAPL, TSLA, AMZN and 84 more". "Stock Price and Volume Data (Basic): Symbol Limited to AAPL, TSLA, AMZN and 84 more". "Historical Data Range (Basic): 5 years" — [site.financialmodelingprep.com/pricing-plans](https://site.financialmodelingprep.com/pricing-plans).
- **FMP `demo` key no longer works.** Both `/stable/quote?symbol=AAPL&apikey=demo` and `/api/v3/quote/AAPL?apikey=demo` return `"Invalid API KEY"` (LIVE 2026-11-12) — [financialmodelingprep.com](https://financialmodelingprep.com/stable/quote?symbol=AAPL&apikey=demo).
- **Massive Stocks Basic** (from the pricing page's embedded plan JSON): "5 API Calls / Minute", "2 Years Historical Data", "End of Day Data", "All US Stocks Tickers", tagline "Great for trying our APIs" — [massive.com/pricing?product=stocks](https://massive.com/pricing?product=stocks).
- **Tiingo Free (Starter)**: 50 requests/hour, 1,000/day, 500 unique symbols/month, 1 GB/month, "30+ Years" of price history, licence "Internal Use Only" — [tiingo.com/about/pricing](https://www.tiingo.com/about/pricing).
- **Marketstack Free**: "100 Requests / mo", "1 Year History", end-of-day only, "No Support", "Non-Commercial Use" — [marketstack.com/pricing](https://marketstack.com/pricing).
- **MarketData.app Free Forever**: "100 Daily API Credits", "1 Year Historical Data", "24h Delayed Stock Data". "If you ask for multiple tickers in the same request, each ticker uses a credit" — [marketdata.app/pricing](https://www.marketdata.app/pricing/).
- **Alpaca Basic**: "200 historical API calls per minute"; equities feed "only from IEX"; history from 2016 — [docs.alpaca.markets](https://docs.alpaca.markets/docs/about-market-data-api). The marketing page says "7+ years" of history and "15 minute delay via API" — [alpaca.markets/data](https://alpaca.markets/data).
- **YH Finance (apidojo) on RapidAPI** plan JSON: a $0 plan with 500 requests/month; paid plans at $10/10k, $30/50k, $100/500k and $300/2.5M per month (fetched 2026-11-12) — [rapidapi.com/apidojo/api/yh-finance/pricing](https://rapidapi.com/apidojo/api/yh-finance/pricing).

### Inferences
- No provider's free tier can serve many installs from one shared key. 25/day (Alpha Vantage), 20/day (EODHD), 100/month (Marketstack) and 800/day (Twelve Data) would all be exhausted quickly by a few hundred installs. This holds even before ToS is considered (see Q6).
- Under BYOK, a single user's portfolio refreshed once a day fits within Twelve Data (800/day), Tiingo (1,000/day), FMP (250/day, sample symbols only) and EODHD (20/day, which is tight).

### Gaps
- Finnhub's exact free quota text could not be read; the pricing page is JS-rendered.
- Alpaca's free market-data ToS (redistribution/display) was not retrieved.

## Q2. Free-tier market coverage: BIST, EU, US. Which plan is needed for BIST?

### Takeaway
No free keyed tier tested here verifiably covers BIST broadly; the strongest candidate is EODHD. EODHD's free plan claims "any ticker, past year" at 20 calls/day; it is the only one that plausibly includes BIST, but this could not be confirmed live. Twelve Data puts BIST and EU exchanges on the **Grow** plan. Every other provider is US-only (or US+China) on its free tier.

### Cited Findings
- **Twelve Data, by exchange.** The reference endpoint `/exchanges?code=XIST&show_plan=true` returns `"access":{"global":"Grow","plan":"Grow","plan_business":"Venture"}` for Borsa İstanbul (LIVE 2026-11-12) — [api.twelvedata.com/exchanges](https://api.twelvedata.com/exchanges?code=XIST&show_plan=true).
- **Twelve Data, by symbol on BIST.** `/stocks?exchange=BIST&show_plan=true` lists 655 BIST stocks: 654 are `Grow` and only `THYAO` is `Basic` (LIVE 2026-11-12) — [api.twelvedata.com/stocks](https://api.twelvedata.com/stocks?exchange=BIST&show_plan=true). The same pattern holds on Xetra: 2,033 stocks, 2,032 `Grow`, only `VOW3` `Basic` (LIVE 2026-11-12). SAP (XETR), VOD (LSE), MC (Euronext) and VWCE (XETR ETF) are all `Grow`. VOO and all 3,106 NYSE ETFs are `Basic` (LIVE 2026-11-12).
- A Twelve Data support article reportedly lists BIST as "Level A (Grow plan)" with "end-of-day" delay. This is a search-result summary; the page returned 404 when fetched directly, so treat it as partially verified — [support.twelvedata.com/en/articles/5749822-borsa-istanbul-bist](https://support.twelvedata.com/en/articles/5749822-borsa-istanbul-bist).
- Twelve Data Grow includes "EOD global equities and ETFs market data". "Real-time EU market data" starts at Pro ($229/mo) — [twelvedata.com/pricing](https://twelvedata.com/pricing).
- **EODHD exchange codes**: `LSE`, `XETRA`, `PA`, `AS`, `BR`, `LS`, `MC`, `US`, among others; "Official authorized distributor of Deutsche Borse market data (Xetra...)" — [eodhd.com/llms-full.txt](https://eodhd.com/llms-full.txt). The llms.txt exchange table did not list Istanbul, and the Istanbul code (`IS`) could not be confirmed: the demo key is blocked from the exchanges list (LIVE 2026-11-12) — [eodhd.com/llms-full.txt](https://eodhd.com/llms-full.txt).
- **Alpha Vantage** documents examples for `.LON` (LSE), `.DEX` (XETRA), `.TRT`, `.TRV`, `.BSE`, `.SHH` and `.SHZ`, and describes daily series as "global equity data ... with 25+ years of historical depth". Istanbul is not mentioned in the portion read — [alphavantage.co/documentation](https://www.alphavantage.co/documentation/).
- **Finnhub `/quote`**: "Get real-time quote data for US stocks... Real-time stock prices for international markets are supported for Enterprise clients via our partner's feed." (from the operation JSON embedded in the docs page) — [finnhub.io/docs/api/quote](https://finnhub.io/docs/api/quote).
- **FMP**: Basic Stock Quote is limited to about 87 sample symbols; Starter's is "Symbol Limited to US Exchanges"; Premium ($69) adds "UK and Canada Coverage"; Ultimate ($139/mo, $99 billed annually) adds "Global Coverage" — [site.financialmodelingprep.com/pricing-plans](https://site.financialmodelingprep.com/pricing-plans).
- **Massive** Stocks plans all cover "All US Stocks Tickers" — [massive.com/pricing?product=stocks](https://massive.com/pricing?product=stocks).
- **Tiingo** supported_tickers.csv, downloaded 2026-11-12, contains exchanges NASDAQ, NYSE, NYSE ARCA, BATS, AMEX, OTC venues, SHE, SHG and only 11 "LSE" rows. Price currencies are USD 102,067, CNY 7,148, HKD 52, AUD 5. No BIST/Xetra/Euronext rows — [apimedia.tiingo.com/docs/tiingo/daily/supported_tickers.zip](https://apimedia.tiingo.com/docs/tiingo/daily/supported_tickers.zip).
- **Marketstack** v2 EOD docs reportedly say the endpoint covers "NASDAQ, PINK, SHG, NYSE, NYSE ARCA, OTCQB, and BATS". This comes from a third-party mirror via search, so it is unverified — [developers.qodex.ai](https://developers.qodex.ai/apilayer/marketstack/v2-tickers/tickers-eod). Marketstack marketing claims "72+ global exchanges" without naming them — [docs.apilayer.com/marketstack](https://docs.apilayer.com/marketstack/docs/api-documentation).
- **MarketData.app**: keyless calls for VOO and THYAO return `401 "Invalid token header"`; only AAPL works keyless (LIVE 2026-11-12). The pricing page describes U.S. stock/options products only — [marketdata.app/pricing](https://www.marketdata.app/pricing/).
- **Alpaca** Basic covers "US stocks and ETFs, and US options" — [docs.alpaca.markets](https://docs.alpaca.markets/docs/about-market-data-api).

### Inferences
- For BIST through a keyed provider, the cheapest verified path is **Twelve Data Grow** (EOD, about $29–79/mo; the page is inconsistent). The cheapest likely path is **EODHD's $19.99/mo "All World"**, if `THYAO.IS` resolves there. EODHD free might also serve BIST within its past-year window, but needs a live check with a real key.
- Twelve Data seems to mark one "sample" symbol per exchange as `Basic`, and THYAO happens to be BIST's. A Basic key may therefore work for THYAO only, which is misleading for testing. Inferred from the show_plan data; not live-tested with a free key.
- US coverage on free tiers is easy (Twelve Data, Tiingo, Massive, Alpaca, MarketData.app, EODHD, Alpha Vantage). EU and BIST are the limiting factor.

### Gaps
- No live confirmation of EODHD free-key access to `THYAO.IS` / `SAP.XETRA`; that needs a registered key.
- Alpha Vantage BIST support (symbol suffix such as `.IST`) is unverified. The demo key blocks `SYMBOL_SEARCH` for anything except the documented example.
- Marketstack's BIST/EU coverage is unverified; the docs pages 404'd or redirected.

## Q3. Historical daily EOD on the free tier, depth, and the "close on date X" endpoint

### Takeaway
On free tiers, past-date closes are available from Twelve Data (US), EODHD (1 year), Tiingo (US, 30+ years), Massive (US, 2 years), MarketData.app (US, 1 year), Marketstack (1 year) and Alpaca (US). Alpha Vantage free returns only the latest 100 trading days, and Finnhub free has no candles at all.

### Cited Findings
- **Twelve Data**: `GET /time_series?symbol=AAPL&interval=1day&date=2026-06-01&apikey=...` returned the single bar `{"datetime":"2026-06-01",...,"close":"306.31000"}` (LIVE 2026-11-12, demo key) — [api.twelvedata.com/time_series](https://api.twelvedata.com/time_series?symbol=AAPL&interval=1day&date=2026-06-01&apikey=demo).
- **EODHD**: `GET /api/eod/{SYMBOL}?from=YYYY-MM-DD&to=YYYY-MM-DD&fmt=json` (1 call). It returns `close` (unadjusted) and `adjusted_close`, and supports `filter=last_close` — [eodhd.com/llms-full.txt](https://eodhd.com/llms-full.txt). The free plan is limited to "1 year history only" — same source. Live demo call for `AAPL.US` 2026-06-01: `close 306.31, adjusted_close 305.8137` (LIVE 2026-11-12).
- **Alpha Vantage**: `TIME_SERIES_DAILY` with compact output returns "only the latest 100 data points". "The 'full' outputsize is available to premium keys." `TIME_SERIES_DAILY_ADJUSTED` is "a premium API function" — [alphavantage.co/documentation](https://www.alphavantage.co/documentation/). There is no per-date parameter; the client must pick the date from the returned map.
- **Finnhub**: stock candles carry `"premium":"Premium Access Required"` in the docs JSON. "Daily data will be adjusted for Splits" — [finnhub.io/docs/api/stock-candles](https://finnhub.io/docs/api/stock-candles). A user report shows free-plan AAPL candles returning `"You don't have access to this resource."` — [github.com/finnhubio/Finnhub-API/issues/546](https://github.com/finnhubio/Finnhub-API/issues/546).
- **Tiingo**: `https://api.tiingo.com/tiingo/daily/<ticker>/prices?startDate=...&endDate=...`; fields include `date`, `close` and `adjClose`. "Most US Equity prices are available at 5:30 PM EST" — [tiingo.com/documentation/end-of-day](https://www.tiingo.com/documentation/end-of-day).
- **MarketData.app**: `GET /v1/stocks/candles/D/AAPL/?from=2026-06-01&to=2026-06-03` (keyless, AAPL only) returned `{"s":"ok","t":[...],"c":[305.8137,314.6893,309.7573],...}` (LIVE 2026-11-12). Those closes equal EODHD's `adjusted_close`, so MarketData daily candles appear to be dividend-adjusted by default (inference). The URL without a trailing slash returns 301.
- **Massive Basic**: "2 Years Historical Data", "End of Day Data" — [massive.com/pricing?product=stocks](https://massive.com/pricing?product=stocks).
- **FMP Basic**: "Historical Data Range (Basic): 5 years", but only for about 87 sample symbols — [site.financialmodelingprep.com/pricing-plans](https://site.financialmodelingprep.com/pricing-plans).
- **Marketstack Free**: "1 Year History" — [marketstack.com/pricing](https://marketstack.com/pricing).
- **Nasdaq Data Link**: the free WIKI EOD table reportedly stopped updating on 2018-04-11 (vendor source, not official) — [eulerpool.com](https://eulerpool.com/financial-data-api/quandl-alternative). "QuoteMedia End of Day US Prices" is listed as "Premium" — [docs.data.nasdaq.com](https://docs.data.nasdaq.com/docs/data-organization). Keyless calls return `QEPx04 "A valid API key is required"` (LIVE 2026-11-12).

### Inferences
- The app should store **unadjusted close** for valuation. EODHD (`close`), Twelve Data (`close`) and Tiingo (`close`) expose it; MarketData.app candles seem to default to adjusted values.
- Alpha Vantage's 100-day compact window does not cover "within the last year" on the free tier.

### Gaps
- Exact free-tier history depth for Twelve Data Basic is not stated on the pricing page.

## Q4. Real-time vs delayed vs end-of-day on the free tier

### Takeaway
"Latest price" on free tiers ranges from real-time US (Twelve Data, Finnhub, Alpaca IEX) to 15-min delayed (EODHD global), end-of-day (Alpha Vantage, Massive, Marketstack, Tiingo EOD) and 24h delayed (MarketData.app).

### Cited Findings
- Twelve Data Basic lists real-time US equities and ETFs, forex and crypto. BIST on Grow is EOD — [twelvedata.com/pricing](https://twelvedata.com/pricing); [api.twelvedata.com/exchanges](https://api.twelvedata.com/exchanges?code=XIST&show_plan=true).
- Alpha Vantage: the quote endpoint updates at the end of each trading day for all users; "realtime US market data, 15-minute delayed US market data" require entitlement via its Alpha X Terminal — [alphavantage.co/premium](https://www.alphavantage.co/premium/); [alphavantage.co/documentation](https://www.alphavantage.co/documentation/). The demo IBM quote showed `"latest trading day": "2026-11-11"` at 10:15 UTC on 2026-11-12, i.e. the prior close (LIVE).
- EODHD free: "Global stock, forex, and crypto data is 15-minute delayed. Live US extended quotes are included." — [eodhd.com/pricing](https://eodhd.com/pricing).
- Finnhub: real-time US quote on free (see Q2) — [finnhub.io/docs/api/quote](https://finnhub.io/docs/api/quote).
- Massive Basic: "End of Day Data"; Starter/Developer are "15-minute Delayed Data"; Advanced ($199) is "Real-time" — [massive.com/pricing?product=stocks](https://massive.com/pricing?product=stocks).
- MarketData.app Free: "24h Delayed Stock Data" — [marketdata.app/pricing](https://www.marketdata.app/pricing/).
- Alpaca Basic: real-time over websocket from IEX only; "15 minute delay via API" — [alpaca.markets/data](https://alpaca.markets/data).
- FMP Basic: "Real-time Data (Basic): Not included." — [site.financialmodelingprep.com/pricing-plans](https://site.financialmodelingprep.com/pricing-plans).

### Inferences
- For a personal-finance app, an end-of-day "latest" price is usually acceptable, so delay matters less than coverage and ToS.

### Gaps
- None material.

## Q5. Quote currency returned and symbol format for BIST/EU

### Takeaway
Symbol formats differ by provider (`THYAO` + `exchange=BIST`/`XIST` on Twelve Data; `THYAO.IS` on EODHD, unverified; `.LON`/`.DEX` on Alpha Vantage; `.LSE`/`.XETRA`/`.PA` on EODHD). LSE prices are often in **pence (GBp/GBX)**, not GBP, and must be normalised.

### Cited Findings
- **Twelve Data** returns `currency` per instrument: THYAO `"currency":"TRY","exchange":"BIST","mic_code":"XIST"`; VOD `"currency":"GBp","exchange":"LSE"`; SAP `"EUR","XETR"`; MC `"EUR","Euronext","XPAR"` (LIVE 2026-11-12) — [api.twelvedata.com/stocks](https://api.twelvedata.com/stocks?symbol=THYAO&show_plan=true). The quote response also includes `"currency":"USD"` for AAPL (LIVE).
- **Alpha Vantage** `SYMBOL_SEARCH` (demo) returned `"1. symbol": "TSCO.LON", ..., "8. currency": "GBX"` (LIVE 2026-11-12) — [alphavantage.co](https://www.alphavantage.co/query?function=SYMBOL_SEARCH&keywords=tesco&apikey=demo). The quote/time-series responses themselves carry no currency field (LIVE IBM sample).
- **EODHD** symbols use `{TICKER}.{EXCHANGE}` (e.g. `BMW.XETRA`, `BP.LSE`, `AI.PA`, `VWCE.XETRA`). The `/eod` response has no currency field; LSE trades in "GBP/GBX" — [eodhd.com/llms-full.txt](https://eodhd.com/llms-full.txt).
- BIST conventions: Reuters `THYAO.IS`, Bloomberg `THYAO.TI` (Turkish Airlines investor relations) — [investor.turkishairlines.com](https://investor.turkishairlines.com/en/share-details/share-details); Google Finance `THYAO:IST` — [google.com/finance](https://www.google.com/finance/quote/THYAO:IST).
- **Tiingo** supported tickers carry a `priceCurrency` column (USD/CNY/HKD/AUD) — [supported_tickers.zip](https://apimedia.tiingo.com/docs/tiingo/daily/supported_tickers.zip).

### Inferences
- The app needs a per-provider symbol mapper (internal ID → provider symbol) and should take currency from instrument metadata (Twelve Data `currency`) or from a static exchange→currency table, with explicit GBp/GBX → GBP ÷100 handling.

### Gaps
- EODHD's BIST suffix (`.IS`) and Marketstack's (`.XIST`) were not confirmed from primary sources.

## Q6. ToS: embedding a key in a distributed client, personal/non-commercial limits, redistribution, attribution

### Takeaway
None of these providers permits shipping a shared free key inside a distributed app. Every free tier is personal/internal, and most ban sharing credentials or providing data to third parties. **BYOK is the only ToS-compatible model**, and Tiingo explicitly blesses it ("Each user must have their own API token", with attribution). For the others, BYOK is defensible because each user is then the licensee using data for their own personal purposes. That is an inference, not an explicit grant.

### Cited Findings
- **Alpha Vantage**: licence "for personal, non-commercial use". Use is "commercial" if "You plan to use or provide information accessed through the Alpha Vantage Platform as part of any type of commercial activity that allows individuals or entities other than User to access information directly or indirectly". The licence is "non-exclusive, non-sublicensable, non-transferable" (ToS §2–3) — [alphavantage.co/terms_of_service](https://www.alphavantage.co/terms_of_service/).
- **Twelve Data** (Terms, last updated 2026-01-01): §2.3(l) prohibits "Use Free Tier data for commercial purposes"; §2.3(m) "Share Credentials or allow unauthorized access to the Platform"; §2.3(b) "Redistribute, resell, sublicense, or transfer any Data or access rights to third parties"; §2.2(e) "Redistribute or provide external display of Data only if and as expressly authorized ... including compliance with attribution requirements" — [twelvedata.com/terms](https://twelvedata.com/terms).
- **Finnhub**: "All plan listed on Finnhub website is strictly for personal use unless explicitly stated otherwise." "You hereby agree to not redistribute or share access to data or derived results ... with anyone or any 3rd party without written approval from Finnhub." No last-updated date — [finnhub.io/terms-of-service](https://finnhub.io/terms-of-service).
- **FMP** (ToS last updated 2023-08-01), §2.2.1 Personal Use: the customer may not "integrate the Data or Services into any tools or applications accessible by any third parties". §2.2.2: "prohibited from showcasing FMP Services or Data on platforms including ... software products, or applications designed for utilization by multiple individuals". "Sharing your account login or API Key with any other individuals or entities ... is strictly prohibited." — [site.financialmodelingprep.com/terms-of-service](https://site.financialmodelingprep.com/terms-of-service). The pricing page adds: "Displaying or redistributing data sourced from FMP requires a specific Data Display and Licensing Agreement" — [pricing-plans](https://site.financialmodelingprep.com/pricing-plans).
- **Massive** Individuals ToS (last updated 2026-11-03): use is "solely for your own personal, non-commercial, and non-business purposes. You may use supported interfaces to build and run your own personal applications". §3.3: "You may not share an account or Credentials to give another person independent access." §6.1(o) forbids circumventing limits "including by creating multiple accounts or rotating Credentials" — [massive.com/legal/individuals-terms-of-service](https://massive.com/legal/individuals-terms-of-service).
- **Tiingo**: "For Basic and Power accounts, data is for internal and personal use only. You may not redistribute the data in any form." — [api.tiingo.com/documentation](https://api.tiingo.com/documentation). The developer appendix says "you will have to get a redistribution license" to redistribute without users having accounts. It also states: "Each user must have their own API token." and "Tiingo must be clearly attributed as the data source." — [tiingo.com/documentation/appendix/developers](https://www.tiingo.com/documentation/appendix/developers).
- **EODHD** Terms: "Non-Professional Users are permitted to store, manipulate, and analyze the data for private, non-commercial purposes". Prohibited: "Selling, reselling, retransmitting, redistributing, displaying, or granting access to the Information" and "Sharing access to their account with others". No attribution clause; no last-updated date (footer "© Unicorn Data Services, 2017-2026") — [eodhd.com/financial-apis/terms-conditions](https://eodhd.com/financial-apis/terms-conditions). EODHD's technical guidance: "Never embed in client-side code (no CORS support)" — [eodhd.com/llms-full.txt](https://eodhd.com/llms-full.txt).
- **MarketData.app** Terms (last updated 2026-11-06): the licence is "for your own personal non-professional use"; "Accounts are designated for individual use only"; "You may not sell, resell, retransmit, or redistribute or make our data or any third-party data available to anyone." — [marketdata.app/terms](https://www.marketdata.app/terms/).
- **Marketstack** Free: "Non-Commercial Use" — [marketstack.com/pricing](https://marketstack.com/pricing).

### Inferences
- A shipped shared key would violate the credential-sharing and third-party-access clauses at Twelve Data, FMP, Massive, EODHD, MarketData.app and Finnhub. It would also let anyone extract the key from the binary.
- With BYOK, each user signs the provider's personal-use terms and the app acts as the user's "own personal application" (Massive's wording). Tiingo's appendix is the only explicit endorsement of that pattern. The app should show "Data: Tiingo" (or similar) attribution regardless.

### Gaps
- Alpaca and Marketstack full ToS clauses were not retrieved.

## Q7. Response shapes and demo/keyless results (LIVE 2026-11-12, about 10:15 UTC)

### Takeaway
Live demo probes worked for Alpha Vantage (IBM), Twelve Data (AAPL), EODHD (AAPL.US) and MarketData.app (AAPL, keyless). FMP's demo key is dead, and Finnhub, Tiingo, Massive/Polygon, Marketstack, Alpaca and Nasdaq Data Link all require a key.

### Cited Findings
- **Alpha Vantage** `GET https://www.alphavantage.co/query?function=GLOBAL_QUOTE&symbol=IBM&apikey=demo` →
  `{"Global Quote":{"01. symbol":"IBM","05. price":"248.2200","07. latest trading day":"2026-11-11","08. previous close":"244.5300",...}}` — [alphavantage.co](https://www.alphavantage.co/query?function=GLOBAL_QUOTE&symbol=IBM&apikey=demo). `TIME_SERIES_DAILY` → `{"Meta Data":{...,"4. Output Size":"Compact"},"Time Series (Daily)":{"2026-11-11":{"4. close":"248.2200",...}}}`. All numbers come back as strings.
- **Twelve Data** `GET https://api.twelvedata.com/quote?symbol=AAPL&apikey=demo` →
  `{"symbol":"AAPL","exchange":"NASDAQ","mic_code":"XNGS","currency":"USD","datetime":"2026-11-11","close":"355.63000","previous_close":"348.51001","is_market_open":false,...}` — [api.twelvedata.com/quote](https://api.twelvedata.com/quote?symbol=AAPL&apikey=demo). Reference endpoints `/stocks`, `/etfs` and `/exchanges` (including `show_plan=true`) work **without any key** (LIVE). Errors come back as `{"code":401,"message":...,"status":"error"}`, possibly with HTTP 200; the HTTP status was not checked.
- **EODHD** `GET https://eodhd.com/api/real-time/AAPL.US?api_token=demo&fmt=json` →
  `{"code":"AAPL.US","timestamp":1794432540,"close":355.63,"previousClose":348.51,"change":7.12,"change_p":2.043}`. `/api/eod/AAPL.US?...from=2026-06-01&to=2026-06-03` → `[{"date":"2026-06-01","open":309.625,"high":310.94,"low":305.02,"close":306.31,"adjusted_close":305.8137,"volume":48849933},...]`. Non-demo symbols return plain-text `Forbidden` — [eodhd.com](https://eodhd.com/api/real-time/AAPL.US?api_token=demo&fmt=json).
- **MarketData.app** `GET https://api.marketdata.app/v1/stocks/quotes/AAPL/` (no key) →
  `{"s":"ok","symbol":["AAPL"],"ask":[355.88],"bid":[355.69],"last":[355.63],"updated":[1794430863]}`. The response is column-oriented, with arrays per field — [api.marketdata.app](https://api.marketdata.app/v1/stocks/quotes/AAPL/).
- Keyless errors: Finnhub `{"error":"Please use an API key."}`; Tiingo `{"detail":"Please supply a token"}`; Massive/Polygon `401 {"status":"ERROR","error":"API Key was not provided"}` on both `api.polygon.io` and `api.massive.com`; Marketstack `{"error":{"code":"missing_access_key",...}}`; Alpaca nginx `401`; Nasdaq Data Link `QEPx04` (all LIVE 2026-11-12).
- Cross-check: AAPL's 2026-06-01 close was 306.31 on both Twelve Data and EODHD (LIVE).

### Inferences
- Parsing burden: Alpha Vantage (string numbers, numbered keys) > MarketData.app (columnar arrays) > Twelve Data (string numbers) > EODHD/Tiingo (plain numeric JSON).

### Gaps
- Live free-key response for BIST tickers could not be tested without signup.

## Q8. Longevity and reliability signals

### Takeaway
IEX Cloud is dead (2024-08-31), and Polygon.io is now Massive (since 2025-10-30), with old hosts still working. Nasdaq Data Link's free equity data has been frozen since 2018. Alpha Vantage, Twelve Data, EODHD, Tiingo and FMP are long-running, but their free tiers have tightened over time.

### Cited Findings
- **IEX Cloud**: IEX announced on 2024-05-31 that it would retire IEX Cloud; shutdown was 2024-08-31. It "accounted for less than 2% of IEX Group revenue and had run at a loss" — [waterstechnology.com](https://waterstechnology.com/data-management/7951882/iex-cloud-closure-forces-fintech-clients-to-seek-data-alternatives); [alphavantage.co IEX analysis](https://www.alphavantage.co/iexcloud_shutdown_analysis_and_migration/). `cloud.iexapis.com` timed out (LIVE 2026-11-12).
- **Polygon.io → Massive** rebrand "effective October 30, 2025". `api.polygon.io` keeps working alongside `api.massive.com` — [einpresswire.com](https://www.einpresswire.com/article/863823068/polygon-io-is-now-massive); [fisd.net](https://fisd.net/polygon-io-is-now-massive). Both hosts answered 401 without a key (LIVE 2026-11-12).
- **Finnhub** moved stock candles to premium; user reports show free AAPL candles denied — [github.com/finnhubio/Finnhub-API/issues/546](https://github.com/finnhubio/Finnhub-API/issues/546). The date of the change is unverified.
- **Alpha Vantage** free limit is now 25/day — [alphavantage.co/premium](https://www.alphavantage.co/premium/). That it was previously higher (500/day) is from memory and unverified.
- **FMP** Basic is restricted to about 87 sample symbols, and the `demo` key no longer works (LIVE) — [pricing-plans](https://site.financialmodelingprep.com/pricing-plans).
- **EODHD** has operated under Unicorn Data Services since 2017 per its ToS footer ("2017-2026") — [eodhd.com/financial-apis/terms-conditions](https://eodhd.com/financial-apis/terms-conditions). The API docs promise "Existing fields are never removed or renamed" and "Existing endpoint URLs remain stable" — [eodhd.com/llms-full.txt](https://eodhd.com/llms-full.txt).
- **Nasdaq Data Link** WIKI was frozen 2018-04-11 (vendor source) — [eulerpool.com](https://eulerpool.com/financial-data-api/quandl-alternative).
- **Yahoo via RapidAPI**: no official Yahoo API, and the wrappers resell unofficial data. Some sources describe Yahoo's API as "shut down by the company" (vendor blogs, low reliability) — [marketdata.app/alternatives/yahoo-finance-api](https://www.marketdata.app/alternatives/yahoo-finance-api/).
- ToS churn: Massive (2026-11-03) and MarketData.app (2026-11-06) both updated terms in the past two weeks; Twelve Data's terms date from 2026-01-01 — sources in Q6.

### Inferences
- Recommended shape (for the report writer to weigh against the keyless/BIST notes):
  - Primary BYOK: **EODHD** (global EOD, one call per close, simple JSON, $19.99/mo upgrade path), or **Twelve Data** (best metadata/currency, free for US; Grow needed for BIST and EU).
  - US-only fallback: **Tiingo** (BYOK explicitly allowed, generous free quota).
  - Avoid as primary: Finnhub (no free history), FMP (sample symbols only), Alpha Vantage (25/day, 100-day window), Marketstack (100/month).
- RapidAPI Yahoo wrappers add a middleman on top of unofficial data. Weak for longevity.

### Gaps
- Founding years and funding of Twelve Data, Tiingo, FMP and MarketData.app were not researched.
- No official IEX press release was retrieved; the dates rest on trade press and Alpha Vantage.
