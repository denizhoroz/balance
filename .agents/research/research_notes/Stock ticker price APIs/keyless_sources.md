# Keyless / unofficial stock & ETF price sources (state as of 2026-11-12)

All live calls below were made by me on **2026-11-12, ~10:14–10:22 UTC** from a single residential-type IP, using `curl` with a generic browser User-Agent (`Mozilla/5.0 (Windows NT 10.0; Win64; x64) ... Chrome/130.0 Safari/537.36`) unless stated otherwise. No cookies, no personal data. "Called it" = my own observation; everything else has a URL.

Test tickers: AAPL, VOO (US), THYAO.IS (BIST, TRY), SAP.DE (Xetra), plus MC.PA, ASML.AS, IWDA.AS, SHEL.L, VUSA.L for EU coverage.

## Yahoo Finance v8 chart endpoint (query1/query2.finance.yahoo.com)

### Takeaway
As of 2026-11-12 the v8 chart endpoint works **keyless and crumb-less** for current price and daily history, covers all required markets (BIST `.IS` in TRY, US, Xetra `.DE`, Euronext `.PA`/`.AS`, LSE `.L`), and absorbed a 50-request parallel burst without a 429 — **but only if a User-Agent header is sent** (empty UA → 429). The other Yahoo endpoints (v7 quote, v10 quoteSummary, v7 download) are crumb/login-gated. Yahoo's ToS explicitly prohibit automated collection without permission, and Yahoo has repeatedly broken/throttled non-browser clients (2024–2025), so it is the best-coverage source but carries ToS and breakage risk.

### Cited Findings
**Live calls (current price):**
- I called it on 2026-11-12: `GET https://query1.finance.yahoo.com/v8/finance/chart/AAPL?range=5d&interval=1d` → **HTTP 200**. Trimmed: `{"chart":{"result":[{"meta":{"currency":"USD","symbol":"AAPL","exchangeName":"NMS","fullExchangeName":"NasdaqGS","instrumentType":"EQUITY","firstTradeDate":345479400,"regularMarketTime":1794430801,...,"exchangeTimezoneName":"America/New_York","regularMarketPrice":355.63,...,"longName":"Apple Inc.` 
- I called it on 2026-11-12: `.../chart/THYAO.IS?range=5d&interval=1d` → **200**, `"currency":"TRY","symbol":"THYAO.IS","exchangeName":"IST","fullExchangeName":"Istanbul","exchangeTimezoneName":"Europe/Istanbul","regularMarketPrice":320.25,...,"longName":"Türk Hava...`
- I called it on 2026-11-12: `.../chart/SAP.DE?...` → **200**, `"currency":"EUR","exchangeName":"GER","fullExchangeName":"XETRA","regularMarketPrice":211.15`
- I called it on 2026-11-12: `.../chart/VOO?...` → **200**, `"currency":"USD","exchangeName":"PCX","fullExchangeName":"NYSEArca","instrumentType":"ETF","regularMarketPrice":698.35`
- I called it on 2026-11-12: query2 host returned identical 200 responses for all four symbols.
- I called it on 2026-11-12 (`range=1d`): MC.PA → 200 `EUR / Paris / 410.55`; ASML.AS → 200 `EUR / Amsterdam / 1372.8`; IWDA.AS → 200 `EUR / Amsterdam / 130.9`; VUSA.L → 200 `GBP / LSE / 108.535`; **SHEL.L → 200 `currency:"GBp"` / 3657.5** (pence, not pounds — client must divide by 100 when `currency == "GBp"`).
- I called it on 2026-11-12: unknown symbol `XYZNOTREAL` → **HTTP 404** with body `{"chart":{"error":{"code":"Not Found","description":"No data found, symbol may be delisted"}}}`.

**Live calls (history):**
- I called it on 2026-11-12: `.../chart/THYAO.IS?period1=1762128000&period2=1762560000&interval=1d&events=div,split` (2025-11-03..2025-11-08 UTC) → **200**. `result[0]` keys: `meta`, `timestamp`, `indicators`. `timestamp=[1762151400,1762237800,1762324200,1762410600,1762497000]` (session-start times, 06:30 UTC = 09:30 TRT), `indicators.quote[0].close=[294.0,289.0,290.0,292.5,289.5]`, `indicators.adjclose[0].adjclose` same values. `meta.validRanges=['1d','5d','1mo','3mo','6mo','1y','2y','5y','10y','ytd','max']`.
- I called it on 2026-11-12: `.../chart/AAPL?range=1y&interval=1d` → 200, **251 daily points**, 2025-11-12 → 2026-11-11. `meta.firstTradeDate=345479400` (1980) indicates history to listing date via `range=max` (not separately fetched).
- Cross-check: Nasdaq API (below) reports AAPL 11/11/2026 close `$355.63`, identical to Yahoo `regularMarketPrice` 355.63 — I called both on 2026-11-12.

**User-Agent / gating behavior:**
- I called it on 2026-11-12: same AAPL URL with **empty User-Agent → HTTP 429 "Too Many Requests"**; with `User-Agent: ureq/2.10.0` → **200**. So a non-empty UA is required; a browser UA is not (at least today). Note ureq sets its own `ureq/x.y` UA by default.
- I called it on 2026-11-12: `https://query1.finance.yahoo.com/v7/finance/quote?symbols=AAPL` → **401** `"User is unable to access this feature"`; `https://query2.finance.yahoo.com/v10/finance/quoteSummary/AAPL?modules=price` → **401** `"Invalid Crumb"`; `https://query1.finance.yahoo.com/v7/finance/download/AAPL?...` → **401** `"User is not logged in"`. Only v8 chart is open.

**Rate limits observed:**
- I called it on 2026-11-12: 40 sequential AAPL requests → all 200; 50 requests in 10 waves of 5 parallel (MSFT, range=1mo) → all 200. No documented limit exists; I found no official Yahoo statement.
- yfinance #2422 "[0.2.57] YFRateLimitError('Too Many Requests')", opened 2025-04-30, closed 2025-05-09: users got 429 from Python `requests` while browser/curl worked; community attributed it to **TLS fingerprinting** and fixed it by switching to `curl_cffi` with Chrome impersonation, which yfinance then adopted — [GitHub #2422](https://github.com/ranaroussi/yfinance/issues/2422) (older info, 2025).
- yfinance #2128 debug log shows crumb fetch returning `'Edge: Too Many Requests'` and query2 `/v8/finance/chart/` returning 429 — [GitHub #2128](https://github.com/ranaroussi/yfinance/issues/2128) (older, date not re-verified).
- yfinance #2297 (Feb 2025) crumb-based 429 even on v0.2.54; #2480 YFRateLimitError persisting 0.2.57–0.2.59 — [#2297](https://github.com/ranaroussi/yfinance/issues/2297), [#2480](https://github.com/ranaroussi/yfinance/issues/2480) (older, per search snippets, not fetched).
- Portfolio Performance (desktop app) forum: users reported 429s on query1 chart requests for several days in April 2025 while the same URL worked in a browser — [forum thread](https://forum.portfolio-performance.info/t/yahoo-finance-too-many-requests-getting-errors-on-updates-desktop-only/32780) (older, per search snippet).
- The 25 most recently closed yfinance issues (closed Jul 14 – Nov 3, 2026) contain **no** 429/rate-limit/crumb/cookie reports; recent Yahoo-side problems are data-quality: #2964 "Missing daily session for 2026-08-28 across multiple tickers", #2966 "Historical data broken for Nuam markets since ~2026-07-17, live quotes unaffected", #3004 `/xhr/ncp` news endpoint now 404, #2925 "Last days data missing", #2895 NaN OHLC for most recent daily bar before exchange open — [yfinance issues list](https://github.com/ranaroussi/yfinance/issues?q=is%3Aissue+sort%3Acreated-desc).
- Rust crate `yahoo_finance_api` (xemwebe) source defines a crumb URL on Yahoo's `/v1/test/getcrumb` — [docs.rs source 4.1.0](https://docs.rs/crate/yahoo_finance_api/4.1.0/source/src/lib.rs); its tracker has a closed issue #42 "429: Too Many Requests" — [ecosyste.ms listing](https://issues.ecosyste.ms/hosts/GitHub/repositories/xemwebe%2Fyahoo_finance_api/issues) (body/date not verified).
- node-yahoo-finance2 discussion #599: HTTP 401 when chaining multiple historical requests — [discussion #599](https://www.github.com/gadicc/node-yahoo-finance2/discussions/599) (date not verified). I found no 2026-dated yahoo-finance2 breakage report.

**Terms of service:**
- Yahoo Terms, §2(d) item 9: users must not "access or collect data, or attempt to access or collect data, from our Services using any automated means, including but not limited to robots, spiders, scrapers, data mining tools, or data gathering or extraction tools, for any purpose without our express, prior permission." §2(h): "Unless you have explicit written permission, you must not reproduce, modify, rent, lease, sell, trade, distribute, or exploit for any commercial purposes, any portion or use of, or access to, the Services". Page references dates 6 May 2025 / May 15, 2025 / January 2, 2026 — [Yahoo Terms of Service](https://legal.yahoo.com/us/en/yahoo/terms/otos/index.html).

### Inferences
- For the app's two needs: (a) current price = `meta.regularMarketPrice` (+ `meta.currency`, `meta.regularMarketTime`); (b) close on past date D = request `period1..period2` window around D (e.g. D-7d..D+1d to cover holidays/weekends), take the last `timestamp` whose exchange-local date (apply `meta.gmtoffset`) ≤ D, use `indicators.quote[0].close` (unadjusted) for valuation. One request per ticker for `range=1y` covers a whole year of valuation days and is the cheapest pattern.
- Must handle: `GBp` pence, `null` entries inside `close` arrays (missing bars, see yfinance #2895/#2964), 404 for unknown symbols, 429 as transient (back off, cache), and always send a UA.
- Risk for a distributed desktop client: each user hits Yahoo from their own IP at low volume, which is the pattern least likely to trigger throttling — but the ToS explicitly forbids automated collection, there is no SLA, and Yahoo has a 2024–2025 history of crumb/TLS-fingerprint gating that broke clients for days. ureq's rustls TLS fingerprint differs from browsers; if Yahoo re-enables TLS fingerprinting (as in April 2025), ureq would likely break and there is no simple Rust-side fix (curl_cffi-style impersonation is not available in ureq). Treat as "best effort, may break without notice"; ship with a fallback and a user-visible "price unavailable" state.

### Gaps
- No official Yahoo rate-limit documentation exists; my burst test is one IP on one day.
- Did not test `range=max` depth for BIST/EU tickers, nor whether `adjclose` differs from `close` after dividends for these tickers.
- Could not verify whether ureq's rustls TLS fingerprint specifically is treated differently from curl's (I tested with curl only, varying UA).

## Stooq (stooq.com CSV)

### Takeaway
Stooq is **no longer usable keyless**: as of 2026-11-12 every `/q/d/l/` CSV request returns an HTML JavaScript proof-of-work "verify your browser" page, and since ~March 2026 downloads additionally require an API key obtained by solving a CAPTCHA. Not shippable.

### Cited Findings
- I called it on 2026-11-12: `https://stooq.com/q/d/l/?s=aapl.us&i=d&d1=20251103&d2=20251107` (also `voo.us`, `sap.de`, `thyao.tr`, `shel.uk`, `vusa.uk`, `mc.fr`) → **HTTP 200 but `Content-Type: text/html`**, 796-byte body: `<noscript>This site requires JavaScript to verify your browser. Please enable JavaScript and reload.</noscript><script>... crypto.subtle.digest("SHA-256", ...) ... fetch("/__verify",{method:"POST",...})` — a SHA-256 proof-of-work challenge (server `openresty`). Same result on `stooq.pl` and with curl's default UA.
- I called it on 2026-11-12: legacy light-quote URL `https://stooq.com/q/l/?s=aapl.us&f=sd2t2ohlcv&h&e=csv` → **404** "The page you requested does not exist or has been moved".
- pandas-datareader #1012 "Stooq now requires API key - Documentation update needed", opened 2026-04-13: `DataReader(ticker,'stooq',...)` fails with `ParserError` because Stooq returns HTML with instructions to request an API key; reporter says the requirement started March 2026 — [GitHub #1012](https://github.com/pydata/pandas-datareader/issues/1012).
- Key page is `stooq.com/q/d/?s=spy.us&get_apikey` (CAPTCHA), key appended as `apikey=` parameter; a daily quota returns "Exceeded the daily hits limit" — per search-result summaries of [GitHub #1012](https://github.com/pydata/pandas-datareader/issues/1012) and [Qiita guide](https://qiita.com/katafuchix/items/c74793e5041e1b5e3051) (summary only, not fetched in full).

### Inferences
- Getting through would require programmatically solving the anti-bot challenge and having each end-user solve a CAPTCHA for a key — that is circumventing an access control and is not acceptable to ship. Drop Stooq.
- Coverage (historically .us, .de, .uk, .fr; BIST unclear) is moot.

### Gaps
- Could not confirm whether a CAPTCHA-obtained apikey also bypasses the JS PoW wall, and did not try (would require circumvention).

## Nasdaq.com API (api.nasdaq.com)

### Takeaway
Works keyless for **US-listed** stocks and ETFs (current quote and daily history, ~5+ years), returns string-formatted prices with `$` and thousands separators, and hangs on curl's default UA. Historical queries behave oddly: short windows in the past return 0 rows, but windows ending today return full data. No BIST or EU coverage (only US ADRs like SAP on NYSE, in USD).

### Cited Findings
- I called it on 2026-11-12: `https://api.nasdaq.com/api/quote/AAPL/info?assetclass=stocks` → **200**: `{"data":{"symbol":"AAPL","companyName":"Apple Inc. Common Stock","stockType":"Common Stock","exchange":"NASDAQ-GS",...,"primaryData":{"lastSalePrice":"$355.20","netChange":"-0.16","percentageChange":"-0.05%",...`
- I called it on 2026-11-12: `.../quote/VOO/info?assetclass=etf` → **200**, `"exchange":"PSE","primaryData":{"lastSalePrice":"$695.99"...`; `.../quote/SAP/info?assetclass=stocks` → **200** `"companyName":"SAP  SE ADS","stockType":"American Depositary Shares","exchange":"NYSE"` (US ADR, not Xetra).
- I called it on 2026-11-12: `.../quote/AAPL/historical?assetclass=stocks&fromdate=2025-11-12&todate=2026-11-11&limit=9999` → **200**, `totalRecords:251`, rows like `{"date":"11/11/2026","close":"$355.63","volume":"34,453,480","open":"$349.545","high":"$355.91","low":"$349.325"}`; `fromdate=2021-11-12..2026-11-11` → 1254 rows back to 11/12/2021.
- I called it on 2026-11-12: **quirk** — `fromdate=2025-11-03&todate=2025-11-07` → 200 with `totalRecords:0, rows:null`; `2020-01-02..2020-01-10` → 0 rows; `2026-11-02..2026-11-06` → only 2 rows (11/05, 11/06). But `fromdate=2025-11-03&todate=2026-11-11` → 258 rows including 11/03/2025 `$269.05`. So fetch "from D to today" and filter client-side.
- I called it on 2026-11-12: VOO ETF historical rows have **no `$`** (`"close":"675.99"`) while stock rows have `$` — inconsistent formatting.
- I called it on 2026-11-12: empty UA → 200; `ureq/2.10.0` UA → 200; **curl's default UA → hung until 20s timeout (HTTP 000)**.
- Nasdaq terms of use: I attempted `https://www.nasdaq.com/terms-of-use` (WebFetch timed out; curl got 404). **Unverified** — I could not read Nasdaq's ToS.

### Inferences
- Usable as a US-only fallback/cross-check for Yahoo. Parsing must strip `$` and `,`. Brittle undocumented internal API used by nasdaq.com front-end; Akamai-fronted (hang behavior consistent with bot filtering, unverified).

### Gaps
- Nasdaq ToS text and any rate limit not verified. Did not burst-test Nasdaq.

## Google Finance

### Takeaway
No usable API. The old `finance.google.com/finance/info` JSON endpoint redirects; the quote page returns HTML with data embedded in JS blobs (scraping only). `GOOGLEFINANCE` is a Sheets-only function and Google states historical data cannot be accessed via the Sheets API or Apps Script.

### Cited Findings
- I called it on 2026-11-12: `https://finance.google.com/finance/info?client=ig&q=NASDAQ:AAPL` → **302** (legacy API gone).
- I called it on 2026-11-12: `https://www.google.com/finance/quote/AAPL:NASDAQ` → **200**, ~1.49 MB HTML, `<title>Apple Inc (AAPL) Stock Price & News - Google Finance`; data is inside `AF_initDataCallback`/`ds:N` batchexecute blobs (e.g. `'ds:4' : {id:'gXxkFd',request:[["AAPL","NASDAQ"]]}`), no stable price attribute found by simple grep. `THYAO:IST` and `SAP:ETR` pages also returned 200.
- Google Docs Help (GOOGLEFINANCE): "Historical data cannot be downloaded or accessed via the Sheets API or Apps Script."; "Quotes are not sourced from all markets and may be delayed up to 20 minutes."; information is "provided 'as is' and solely for informational purposes, not for trading purposes or advice." — [Google Docs Help: GOOGLEFINANCE](https://support.google.com/docs/answer/3093281).

### Inferences
- Scraping 1.5 MB obfuscated pages per ticker is fragile and against Google's ToS (ToS not fetched). Not viable.

### Gaps
- Google ToS clause on automated access not fetched.

## CNBC quote API (quote.cnbc.com / ts-api.cnbc.com)

### Takeaway
Keyless JSON for US and Xetra (current quote + daily bars), but **no BIST coverage** found; CNBC/NBCU terms prohibit scraping/integration into other products. Secondary fallback for US/EU only.

### Cited Findings
- I called it on 2026-11-12: `https://quote.cnbc.com/quote-html-webservice/restQuote/symbolType/symbol?symbols=AAPL|THYAO-TR|SAP-DE&requestMethod=itv&noform=1&partnerId=2&fund=1&exthrs=1&output=json&events=1` → **200**. AAPL: `last:"355.36", last_time:"2026-11-11", currencyCode:"USD", exchange:"NASDAQ"`; SAP-DE: `last:"211.00", last_time:"2026-11-12T11:02:20.000+0100", currencyCode:"EUR", exchange:"XETRA"`; THYAO-TR: `code:3`, all fields null.
- I called it on 2026-11-12: BIST variants `THYAO-IS`, `THYAO.IS`, `THYAO` → `code:3`; `THYAO-TK`, `THYAO-IST` → `code:1`; none returned data.
- I called it on 2026-11-12: `https://ts-api.cnbc.com/harmony/app/bars/AAPL/1D/20251103000000/20251107000000/adjusted/EST5EDT.json` → **200**, `{"barData":{"priceBars":[{"open":"270.4200","high":"270.8500","low":"266.2500","close":"269.0500","volume":50194583,"tradeTime":"20251103000000",...` (matches Nasdaq's 11/03/2025 close $269.05). Same for `SAP-DE` → 200, 2025-11-03 close `"228.8500"`.
- NBCUniversal/CNBC terms (supplemental CNBC Pro / Investing Club / CNBC+ sections) state "You may not scrape, repackage, reproduce, republish, recirculate, offer for sale, sell, create derivative works from, integrate into other products or services, or otherwise share with others for profit or commercial gain ... data" — [NBCUniversal Terms of Service](https://www.cnbc.com/nbcuniversal-terms-of-service/) (I grepped the page on 2026-11-12; the matched clauses are in supplemental sections, the general-section wording was not isolated).

### Inferences
- The `adjusted` path segment suggests adjusted bars; an `unadjusted` variant likely exists (unverified). Symbol mapping for LSE/Euronext not tested.

### Gaps
- LSE/Euronext CNBC symbol formats and rate limits not tested.

## Boerse Frankfurt API (api.boerse-frankfurt.de)

### Takeaway
Current-price endpoint is keyless today (by ISIN + MIC, Xetra and Frankfurt venues, including US shares quoted in EUR), but the history endpoint returned empty — history appears gated. Requires ISIN, not ticker. Undocumented internal API.

### Cited Findings
- I called it on 2026-11-12: `https://api.boerse-frankfurt.de/v1/data/price_information/single?isin=DE0007164600&mic=XETR` (SAP) → **200**: `{"changeToPrevDayAbsolute":0.1500,"changeToPrevDayInPercent":0.0700,"closingPricePrevTradingDay":210.95,"currency":{"originalValue":"EUR",...},"dayHigh":213,"dayLow":210.6...`
- I called it on 2026-11-12: `...single?isin=US0378331005&mic=XFRA` (Apple on Frankfurt) → **200**, `closingPricePrevTradingDay":320.2,"currency":{"originalValue":"EUR"...` (EUR listing, not USD).
- I called it on 2026-11-12: `https://api.boerse-frankfurt.de/v1/data/price_history?limit=10&offset=0&isin=DE0007164600&mic=XETR&minDate=2025-11-03&maxDate=2025-11-07` → **200 with body `{}`** (no data).

### Inferences
- Community clients historically added computed `X-Client-TraceId`/`X-Security` headers for this API; the empty history response is consistent with that gating (unverified — I did not find/cite a source this session). Reverse-engineering those headers would be circumvention. Not recommended beyond an optional Xetra current-price fallback.

### Gaps
- Boerse Frankfurt ToS and history-gating mechanism not verified.

## Euronext Live, FT, Investing.com, MarketWatch, Business Insider, Morningstar

### Takeaway
None is a viable keyless JSON source in Nov 2026: Euronext now returns an encrypted payload, Investing.com and MarketWatch block non-browser clients (403/401), FT and Business Insider are HTML-only scraping targets with no BIST via simple symbol forms.

### Cited Findings
- I called it on 2026-11-12: `https://live.euronext.com/en/ajax/getDetailedQuote/FR0000121014-XPAR` (LVMH) → **200** but body is `{"ct":"Ui0tjuuMOvNvvarWoWhDg509yUNc..."}` — an encrypted/obfuscated ciphertext blob, not HTML/JSON prices. `.../ajax/getHistoricalPriceDownload/FR0000121014-XPAR?format=csv&...&startdate=2025-11-03&enddate=2025-11-07` → **404** HTML.
- I called it on 2026-11-12: `https://api.investing.com/api/financialdata/6408/historical/chart/?interval=P1D&pointscount=60` → **403**; `https://www.investing.com/equities/apple-computer-inc` → **403**.
- I called it on 2026-11-12: `https://www.marketwatch.com/investing/stock/aapl` → **401** (`<title>marketwatch.com`, bot wall).
- I called it on 2026-11-12: `https://markets.ft.com/data/equities/tearsheet/summary?s=AAPL:NSQ` → **200** HTML, price in `mod-ui-data-list__value">355.63`; `s=THYAO:IST`, `THYAO:IST:TRY`, `THYAO.IS`, `THYAO:IS`, `SAP:GER` → **302** redirects (no data). FT history needs an internal numeric `xid` (not tested).
- I called it on 2026-11-12: `https://markets.businessinsider.com/stocks/aapl-stock` → **200** HTML (`Apple Stock Price | AAPL Stock Quote, News, and History | Markets Insider`); no JSON API tested.
- Morningstar: not tested this session.

### Inferences
- These are scraping targets with active anti-bot measures (encryption, 401/403 walls); shipping them in a distributed client means frequent breakage and clear ToS exposure.

### Gaps
- Morningstar, LSE (londonstockexchange.com), Borsa Istanbul official site, and Turkish broker public endpoints (e.g. İş Yatırım) were not tested in this session; BIST alternatives to Yahoo therefore remain unverified.
