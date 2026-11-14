# BIST (Borsa Istanbul) stock price sources for a backend-less Rust desktop app (as of 2026-11-12)

All live calls below were made by me on **2026-11-12 (~10:15 UTC, ~13:15 Istanbul, BIST session open)** with curl and a generic UA `Mozilla/5.0 (Windows NT 10.0; Win64; x64) research-client`, unless noted otherwise. No auth, cookies, or bypass attempts. Raw responses are in the session scratchpad, not the repo.

**Important robots.txt caveat:** I checked several robots.txt files only *after* the first data-endpoint calls in the same batch. İş Yatırım (`/_layouts/`) and Yahoo (`query1.finance.yahoo.com`, `Disallow: /`) both disallow automated agents on the paths I called. I recorded the single calls already made and did not call those paths again apart from one Yahoo verification batch, which I ran before reading Yahoo's robots.txt output. Bigpara, TEFAS `/api/` and mynet `/api/` disallow automated access, so I did not call their data APIs (Bigpara's API refused my one call with 403 anyway).

## 1. Official: what Borsa Istanbul itself offers (Datastore, dissemination, delayed data, EOD bulletins, licensing)

### Takeaway
Borsa Istanbul has no free public price API. Market data goes out real-time, delayed or end-of-day through **licensed data vendors**. Historical and EOD files are sold through **DataStore** (datastore.borsaistanbul.com), a login-based web shop. The exchange's own public quotes are 15-min delayed. BIST asserts copyright over all market data, and any app redistributing it would formally need a vendor licence (or should source from a licensed vendor that permits it). For a personal-finance desktop app, the official route is not practical for keyless use.

### Cited Findings
- I could not reach borsaistanbul.com from this machine. On 2026-11-12, `https://www.borsaistanbul.com/` gave a TLS handshake reset ("Recv failure: Connection was reset", IP 185.76.201.119). Plain `http://` returned 302 to https. WebFetch also failed with ECONNRESET on two official pages. So the official-page claims below come from **search-result snippets**, not pages I read in full. Treat them as moderately confident. (I called it on 2026-11-12.) This could be geo-blocking or a WAF that also hits some end users, which is a reliability concern for any direct borsaistanbul.com file download.
- "Historical data of Borsa İstanbul can be accessed from datastore.borsaistanbul.com." Equity data files have been available only on DataStore since 2015-08-01 and were removed from the main website. The equity table lists daily and historical traded value, volume and contracts, member rankings, and off-exchange transactions — [Equity Market Data](https://www.borsaistanbul.com/en/data/equity-market-data), [Stock Market Data](https://www.borsaistanbul.com/en/sayfa/2732/stock-market-data) (search snippets).
- An official page lists a "Historical Data Sales" section, which means historical data is sold — [Market Data](https://borsaistanbul.com/en/sayfa/2720/market-data) (search snippet).
- DataStore is an Ember single-page app ("data-market-client", default locale tr). Its config sets `cookie_domain: borsaistanbul.com` and `cookie_secure: true`, which points to session/login-based shopping, not a REST API. The robots.txt allows all (`User-agent: *` with no Disallow). (I called `https://datastore.borsaistanbul.com/` on 2026-11-12: HTTP 200, 1.8 KB SPA shell.) I found no documented public JSON endpoint on DataStore. **Whether any DataStore EOD file downloads without login is unverified.**
- A Borsa Istanbul FAQ gives a direct URL naming pattern for daily bulletin zip files (year/month/day/session) — [Technical Side Of Equities Market Data Dissemination](https://www.borsaistanbul.com/en/faq/technical-side-equities-market-data-dissemination) (search snippet only; whether it is still live or free is unverified). Derivatives and precious-metals "Bulletin data" pages also exist — [Bulletin and Market Data (derivatives)](https://www.borsaistanbul.com/en/data/derivatives-market-data/bulletin-and-market-data), [Bulletin Data (PMDM)](https://www.borsaistanbul.com/en/data/precious-metals-and-diamond-markets/bulletin-data).
- "Market data are disseminated on real time, delayed and end of day basis through Borsa İstanbul's licensed data vendors" — [Data Dissemination](https://www.borsaistanbul.com/en/data/data-dissemination), [Market Data Products](https://www.borsaistanbul.com/en/data/data-dissemination/market-data-products) (search snippets).
- The exchange's own data page states "all data are 15 min delayed" — [Data](https://www.borsaistanbul.com/en/data) (search snippet).
- The Data Fee Schedule (Addendum 3) lists delayed-data fees per product. "Distributors that receive Delayed Data shall not be charged for the End of Day distribution of the same data", and some delayed Level 1 items are listed free of charge. **Caveat:** the search engine dated this PDF at about 2,141 days old (about 2020), so the fees may be stale — [Borsa İstanbul Data Fee Schedule (Addendum 3) PDF](https://www.borsaistanbul.com/files/borsa-istanbul-data-fee-schedule.pdf). Brochure: [Data_Dissemination.pdf](https://www.borsaistanbul.com/files/Data_Dissemination.pdf).
- Vendor-side view (Matriks): with 15-min delay, "no fee is charged". Real-time access carries "license fees determined by Borsa İstanbul depending on the content". The KD1 licence gives live last price, bid/ask and % change, while volume stays 15-min delayed — [Matriks SSS](https://www.matriksdata.com/website/egitim/sikca-sorulan-sorular), [Matriks: BIST Veri Lisansları](https://www.matriksdata.com/website/blog/borsa-istanbul-bist-veri-lisanslari) (blog about 2 years old per search metadata).
- Yapı Kredi states its BIST data comes from FOREKS with a 15-min delay — [Yapı Kredi Piyasa Takip](https://www.yapikredi.com.tr/yatirimci-kosesi/piyasa-takip-listem). Mynet sells "Anlık Borsa" real-time for 1999.90 TL/year; other platforms stay 15-min delayed — [Mynet Anlık Borsa](https://finans.mynet.com/borsa/anlikborsaverileri-abone/).
- TradingView users need a BIST-licensed data package for live BIST prices — [MEO PRO 2026 guide](https://www.bymeoman.com/pro/tradingview-verileri-gercekten-canli-mi-2026-veri-paketleri-ve-gecikme-rehberi/) (secondary blog).
- The borsapy README: "For commercial use, you must contact Borsa Istanbul to purchase an appropriate license." The library is "for personal and educational use only" — [borsapy GitHub](https://github.com/saidsurucu/borsapy).

### Inferences
- A free, open-source personal app that fetches delayed or EOD data per user from a public site sits in a grey zone. It is the same position as every hobby library (borsapy, isyatirimhisse). It is not "licensed redistribution", but each user is fetching what the site shows them publicly. Shipping cached or bundled BIST data inside the app would be clearer redistribution.
- The fee schedule's "EOD free for delayed-data distributors" rule hints that EOD closes are the cheapest tier. Still, any official route needs a vendor agreement. No keyless official endpoint exists.

### Gaps
- Current (2026) fee schedule, the vendor agreement text, and whether DataStore offers any free EOD equity file without login. I could not fetch borsaistanbul.com (TLS reset).
- Whether the bulletin zip URL pattern still serves equity closing prices publicly.

## 2. Turkish public sites/endpoints (İş Yatırım, Bigpara, mynet, uzmanpara, investing.com, KAP, Foreks, Matriks, Midas, Gedik, Fintables, borsagündem, TradingView scanner)

### Takeaway
Two keyless JSON endpoints worked on 2026-11-12:
- **İş Yatırım `HisseTekil`** gives daily OHLC, volume and USD price history by date range, in TRY.
- **İş Yatırım `OneEndeks`** gives a live-ish last price.

The **TradingView scanner** (`scanner.tradingview.com/turkey/scan`) gives latest close, labeled `delayed_streaming_900` (15-min delayed), and has no date history. None of these are official APIs. İş Yatırım's robots.txt disallows the `/_layouts/` path where both of its endpoints live. Bigpara and investing.com block automated clients (403 / Cloudflare challenge). KAP blocked me with HTTP 666. Mynet, uzmanpara and Fintables serve only HTML pages, so using them means scraping.

### Cited Findings

**İş Yatırım (isyatirim.com.tr)**
- robots.txt: `User-agent: * / Disallow: /_layouts/` (also `/_vti_bin/`, `/_catalogs/`, search and membership pages). (I called `https://www.isyatirim.com.tr/robots.txt` on 2026-11-12: HTTP 200.) **Both data endpoints sit under `/_layouts/`**, so automated use goes against robots.txt.
- **History, working:** `GET https://www.isyatirim.com.tr/_layouts/15/Isyatirim.Website/Common/Data.aspx/HisseTekil?hisse=ASELS&startdate=01-10-2026&enddate=10-10-2026`. Dates are in `DD-MM-YYYY` format. I called it on 2026-11-12: HTTP 200, `application/json`, 5,090 bytes. The response looked like `{"ok":true,...,"value":[{"HGDG_HS_KODU":"ASELS","HGDG_TARIH":"01-10-2026","HGDG_KAPANIS":370.0000,"HGDG_AOF":362.566,"HGDG_MIN":342.0,"HGDG_MAX":370.0,"HGDG_HACIM":26070898458.0,...,"DD_DOVIZ_KODU":"USD","DD_DEGER":49.0348,"DOLAR_BAZLI_FIYAT":7.5457,...}]}`.
  - It returned 7 trading days: 01-10 370.0, 02-10 362.75, 05-10 371.5, 06-10 376.0, 07-10 348.75, 08-10 348.75, 09-10 351.25. Weekends were absent.
  - Fields: HGDG_KAPANIS (close), HGDG_AOF (weighted average), MIN, MAX, HACIM (TRY volume), END_DEGER (BIST100 level that day), USD/TRY rate, USD-based price, market cap (PD), and both raw (HG_*) and adjusted (HGDG_*) price sets.
- Malformed dates fail softly. Appending `.json` to `enddate` returned HTTP 200 with `{"ok":false,"errorCode":"SYS0000","errorDescription":"Failed to convert value ... LocalDate ..."}`, a Spring/Java backend error. So the Rust client must check the `ok` field, not just the HTTP status. (I called it on 2026-11-12.)
- **Latest, working:** `GET .../Common/Data.aspx/OneEndeks?endeks=THYAO`. I called it on 2026-11-12: HTTP 200 JSON. It returned `[{"updateDate":"2026-11-12T12:59:25.000+03","bid":320,"ask":320.25,"low":315.5,"high":320.25,"last":320.25,"dayClose":314.75,"open":315.5,"weekClose":297,"monthClose":286.25,"yearClose":268,"limitUp":346,"limitDown":283.5,...}]`. The updateDate was about 15 min before my call time, which fits a 15-min delay (inferred).
- `.../Common/ChartData.aspx/IndexHistoricalAll?period=1440&from=...&endeks=XU100.I` returned HTTP 200 with an empty body (0 bytes) when I called it on 2026-11-12. That chart endpoint, as I guessed its parameters, is broken or changed (unverified which).
- Reliability: turkiye-mcp issue #10 (2026-11-10) reported its daily health check failing on the isyatirim source with "fetch failed". The maintainer called it a transient connection problem — [turkiye-mcp #10](https://github.com/YusufSizmaz/turkiye-mcp/issues/10). borsa-mcp PR #3 found İş Yatırım's corporate-actions endpoint slow (3 of 4 probes timed out at 5 s) and raised the timeout to 15 s, matching borsapy — [borsa-mcp PR #3](https://github.com/biNom-Highend-Engineering/borsa-mcp/pull/3).

**TradingView scanner (scanner.tradingview.com/turkey/scan)**
- `POST https://scanner.tradingview.com/turkey/scan` with JSON body `{"symbols":{"tickers":["BIST:THYAO","BIST:ASELS","BIST:XU100"]},"columns":["close","currency","update_mode","description"]}`. I called it on 2026-11-12: HTTP 200, keyless, no cookies. It returned `{"totalCount":3,"data":[{"s":"BIST:THYAO","d":[320.25,"TRY","delayed_streaming_900","Turk Hava Yollari A.O."]},{"s":"BIST:ASELS","d":[346.75,"TRY","delayed_streaming_900",...]},{"s":"BIST:XU100","d":[11834.06,null,"delayed_streaming_900","BIST 100 Index"]}]}`. Note that `currency` is null for the index.
- Columns `open, high, low, volume, change, Perf.W` also work (filter by `name`) (I called it on 2026-11-12). A non-browser UA `Rust-reqwest/0.12` also got HTTP 200 (I called it on 2026-11-12).
- **No date-range history.** The scanner returns current snapshot fields only. borsapy gets BIST history from the TradingView **WebSocket** API instead, which is 15-min delayed by default — [borsapy README](https://github.com/saidsurucu/borsapy). borsapy v0.8.4 notes that TradingView "always returned the last N bars", so start/end filtering happens client-side — [borsapy releases](https://github.com/saidsurucu/borsapy/releases).
- The robots.txt at www.tradingview.com has no rule for the scanner host. I did not check scanner.tradingview.com/robots.txt (gap). The scanner is undocumented and unofficial. TradingView's ToS was not reviewed here (gap).

**Bigpara (bigpara.hurriyet.com.tr)**
- robots.txt: `User-agent:* Disallow: /api/`, plus explicit `Disallow: /` for GPTBot, anthropic-ai, ClaudeBot, ChatGPT-User and others. (I called it on 2026-11-12.)
- `GET /api/v1/borsa/hisseyuzeysel/THYAO` and `/api/v1/hisse/list` returned **HTTP 403 "Forbidden — MerlinCDN"** (I called them on 2026-11-12). Bigpara states its stock data is 15-min delayed except on live pages for logged-in members — [Bigpara BIST 100 page](https://bigpara.hurriyet.com.tr/borsa/endeksler/bist-tum-100/) (search snippet).

**investing.com (tr.investing.com)**
- Even `/robots.txt` returned a Cloudflare "Just a moment..." challenge page (I called it on 2026-11-12). It is bot-protected, so I did not probe further.

**KAP (kap.org.tr)**
- `https://www.kap.org.tr/robots.txt` returned **HTTP 666** with a block page saying to report error "20295500-PPE6" to kapdestek@mkk.com.tr (I called it on 2026-11-12). KAP carries disclosures, not prices, so it is irrelevant for price data anyway.

**mynet finans (finans.mynet.com)**
- robots.txt disallows `/api/` and `/mobileapi/comments/` (I called it on 2026-11-12). The HTML page `https://finans.mynet.com/borsa/hisseler/thyao-turk-hava-yollari/` returned HTTP 200, 285 KB of HTML (I called it on 2026-11-12). It is scrape-only. Real-time is a paid subscription — [Mynet Anlık Borsa](https://finans.mynet.com/borsa/anlikborsaverileri-abone/).

**Uzmanpara (uzmanpara.milliyet.com.tr)**
- robots.txt disallows only `/iframe/ /inc/ /c/ /kap-haberi/`. `https://uzmanpara.milliyet.com.tr/borsa/hisse-senetleri/THYAO/` redirected to `/borsa/hisse-senetleri/turk-hava-yollari-thyao/`, HTTP 200, 126 KB HTML (I called it on 2026-11-12). No JSON API was identified, so it is scrape-only. `www.uzmanpara.com` gave no connection (HTTP 000).

**Fintables (fintables.com)**
- robots.txt allows all. `https://fintables.com/sirketler/THYAO` returned HTTP 200, 449 KB Next.js HTML (I called it on 2026-11-12). No public JSON API was identified (unverified). It is scrape-only.

**Foreks, Matriks, Midas, Gedik, borsagündem**
- Foreks and Matriks are licensed BIST data vendors with commercial terms (Matriks: free 15-min delayed in its app, paid real-time licences) — [Matriks SSS](https://www.matriksdata.com/website/egitim/sikca-sorulan-sorular), [Yapı Kredi/FOREKS](https://www.yapikredi.com.tr/yatirimci-kosesi/piyasa-takip-listem). I found no public keyless API for Foreks, Matriks, Midas, Gedik or borsagündem (not tested; unverified).

### Inferences
- The best keyless pair today is İş Yatırım `HisseTekil` for history and EOD closes, plus İş Yatırım `OneEndeks` or the TradingView scanner for latest price. All of them are unofficial, robots-disallowed or undocumented, and could break without notice. Build a pluggable provider with fallback.
- İş Yatırım closes for ASELS in Oct 2026 matched Yahoo `.IS` closes exactly (see section 4). That cross-validates both sources and makes Yahoo a usable fallback.
- Bot defences such as MerlinCDN 403, Cloudflare and KAP's 666 mean scraping consumer portals is fragile and inappropriate.

### Gaps
- No public ToS text from İş Yatırım or TradingView on programmatic use was reviewed.
- An İş Yatırım index-history endpoint that works for XU100 was not confirmed. `HisseTekil` does return the BIST100 level per day (END_DEGER) alongside each stock.

## 3. TCMB EVDS: BIST indices or stock prices? Key needed?

### Takeaway
EVDS carries BIST index series (third-party docs cite `TP.MK.F.BILESIK.TUM` for BIST 100 daily), but no individual stock prices. The old evds2 REST API closed at end-2025. The new evds3 backend needs a free **per-user API key**, sent as an HTTP header.

### Cited Findings
- `https://evds2.tcmb.gov.tr/robots.txt` redirects to `https://evds3.tcmb.gov.tr/`, a React SPA shell titled "EVDS" (I called it on 2026-11-12: 302, then 200 HTML).
- "The old `evds2.tcmb.gov.tr` REST API closed at the end of 2025 and now returns a redirect to an SPA HTML page." The new v3 backend uses `/igmevdsms-dis/...` paths. Keys are registered at evds3.tcmb.gov.tr, and catalog browsing works without a key — [borsapy releases v0.10.0 (04 May 2026)](https://github.com/saidsurucu/borsapy/releases).
- The key comes from evds3.tcmb.gov.tr → Benim Sayfam → Profilim → "API KEY KOPYALA". Since a 2024 change, the key goes in an HTTP header, not the URL — [borsapy README](https://github.com/saidsurucu/borsapy), [ml_booster_mcp](https://github.com/vykarakaya26/ml_booster_mcp) (search snippets).
- A borsapy README table lists `TP.MK.F.BILESIK.TUM` as the daily BIST 100 code. I found no official TCMB doc confirming it — [borsapy GitHub](https://github.com/saidsurucu/borsapy) (search snippet; unverified against TCMB).

### Inferences
- EVDS is official-ish (central bank), but it is index-only, needs a per-user key, and lags. It is not useful for per-stock prices. It might serve as a reference for BIST 100 history.

### Gaps
- I did not test an evds3 data call (it needs a key). Whether TCMB terms permit app redistribution was not checked.

## 4. Global providers' BIST support (brief)

### Takeaway
**Yahoo Finance `.IS` tickers work keyless** for both latest price and daily history in TRY (THYAO.IS, ASELS.IS, XU100.IS). Values matched İş Yatırım exactly. The API is unofficial, and query1's robots.txt is `Disallow: /`. It is UA-sensitive: curl's default UA got 429.

### Cited Findings
- `GET https://query1.finance.yahoo.com/v8/finance/chart/THYAO.IS?range=5d&interval=1d` returned HTTP 200. Meta: `"currency":"TRY","exchangeName":"IST","timezone":"TRT","regularMarketPrice":320.0,...,"longName":"Türk Hava ..."` (I called it on 2026-11-12).
- `GET https://query2.finance.yahoo.com/v8/finance/chart/XU100.IS?period1=1761955200&period2=1762819200&interval=1d` returned HTTP 200 (I called it on 2026-11-12). That window covered 2025-11-01 to 2025-11-11 (I mistyped the year). XU100 closes: 11060.40, 10914.10, 10970.40, 11073.30, 10924.50, 10789.00. This confirms at least a year of history. Live XU100 `regularMarketPrice` was 11835.0.
- ASELS.IS for 2026-10-01 to 2026-10-10 returned closes 370.0, 362.75, 371.5, 376.0, 348.75, 348.75, 351.25. These are **identical to İş Yatırım HisseTekil**, and adjclose was the same (I called it on 2026-11-12).
- UA sensitivity: UA `Rust-reqwest/0.12` got HTTP 200. curl's default UA got **HTTP 429 "Edge: Too Many Requests"** (I called it on 2026-11-12). A Rust client must set a UA.
- `https://query1.finance.yahoo.com/robots.txt` returns `User-agent: * Disallow: /` (I called it on 2026-11-12).
- Twelve Data, EODHD and FMP were not tested here (covered by another researcher).

### Inferences
- Yahoo is the most convenient single keyless source (latest plus history, same API shape as other markets). The "unofficial, robots-disallowed" status is the same as for İş Yatırım.

### Gaps
- Yahoo ToS text and rate limits for BIST were not reviewed here. Whether Yahoo's BIST quotes are 15-min delayed was not checked in this run: regularMarketTime 1794477606 is roughly the call time, so it is unclear.

## 5. Open-source projects fetching BIST data, their endpoints and recent breakage

### Takeaway
**borsapy** (saidsurucu, actively maintained in 2026) uses the TradingView WebSocket for BIST prices and history, the TradingView scanner for indicators, İş Yatırım for financials, splits and corporate actions, TEFAS for funds, and evds3 for macro data. **isyatirimhisse** (urazakgul, v5.0.1) wraps İş Yatırım. Recent breakage clusters around **TEFAS** (April 2026 site rewrite) and **EVDS** (evds2 shutdown end-2025). İş Yatırım shows intermittent slowness and timeouts rather than permanent breaks.

### Cited Findings
- borsapy sources: "BIST fiyat verileri TradingView WebSocket API üzerinden sağlanır" ("BIST price data comes via the TradingView WebSocket API"), and the same for indices. Indicators come from the TradingView Scanner API, FX from canlidoviz.com, crypto from BtcTurk, and EVDS needs `bp.set_evds_key(...)`. Data is "~15 dakika gecikmeli" (about 15 minutes delayed) by default. Docs mention a TradingView-auth section for real-time data — [borsapy README](https://github.com/saidsurucu/borsapy).
- borsapy release notes (year inferred as 2026) — [borsapy releases](https://github.com/saidsurucu/borsapy/releases):
  - **v0.8.4 (28 Mar):** TradingView history returns the last N bars, so filtering is client-side. `adjust=False` reverses TradingView split adjustment using İş Yatırım split data.
  - **v0.8.3 (09 Mar):** İş Yatırım MaliTablo batch size dropped 5 to 4, because 5-period requests returned empty results.
  - **v0.8.5 (17 Apr):** TEFAS WAF blocks retried.
  - **v0.9.0 (02 May):** TEFAS moved to Next.js in April 2026. Legacy `/api/DB/*` endpoints return 404 (breaking). History now uses `fonFiyatBilgiGetir`, max 5 years. Allocation needed Scrapling/Camoufox to pass the Akamai TSPD JS challenge.
  - **v0.10.0 (04 May):** evds2 REST closed end-2025, replaced by the evds3 `/igmevdsms-dis/...` backend.
  - **v0.10.1 (13 Jun):** some TradingView WebSocket index candles arrive without volume.
  - **v0.11.0 (07 Aug):** TEFAS allocation moved to a plain POST on the `/api/funds/*` JSON endpoint. "TEFAS returns HTTP 429 after about 4 requests, and the block lasts about 45 seconds."
- isyatirimhisse: latest PyPI version 5.0.1 — [PyPI](https://pypi.org/project/isyatirimhisse/), [GitHub](https://github.com/urazakgul/isyatirimhisse). All 16 issues on the tracker are closed. Notable ones — [isyatirimhisse issues](https://github.com/urazakgul/isyatirimhisse/issues?q=is%3Aissue):
  - #20 "10054 ve value hatası" (connection reset / value error), closed 2025-07-19 (older).
  - #23 "fetch_financials hata donuyor", closed 2026-02-21.
  - #10, a data gap from the İş Yatırım link in FetchData.py (2023, older).
  - #18 asked about fund prices (2025).
- MCP servers wrapping these sources: borsa-mcp — [saidsurucu/borsa-mcp](https://github.com/saidsurucu/borsa-mcp); turkiye-mcp health check flagged the isyatirim source on 2026-11-10 — [issue #10](https://github.com/YusufSizmaz/turkiye-mcp/issues/10).
- tvDatafeed and other "bist" libraries were not investigated in this run (gap).

### Inferences
- The ecosystem converges on TradingView (prices) plus İş Yatırım (history and fundamentals) as de facto sources. Both are unofficial. Expect maintenance churn of several breaking upstream changes per year, going by borsapy's 2026 release cadence.

### Gaps
- tvDatafeed issue history (TradingView WebSocket auth and rate-limit breakage) not checked. borsapy's own Issues page not read directly.

## 6. Bonus: TEFAS (fund NAV history)

### Takeaway
TEFAS has JSON endpoints (`/api/funds/*`, `fonFiyatBilgiGetir`) that open-source libs use for fund NAV history, apparently keyless. However, **tefas.gov.tr robots.txt disallows `/api/` for all agents**. The site was rebuilt in April 2026 (old `/api/DB/*` endpoints return 404). It rate-limits hard (429 after about 4 requests, about 45 s block) and has Akamai bot challenges on some pages. I did not call the API because of the robots.txt rule.

### Cited Findings
- `https://www.tefas.gov.tr/robots.txt`: `User-Agent: * Allow: / Disallow: /api/ Disallow: /private/ Disallow: /tefasbuild Disallow: /_next/ ...`, with explicit `/api/` disallows for GPTBot, anthropic-ai, CCBot and Google-Extended (I called it on 2026-11-12).
- TEFAS moved to Next.js in April 2026, and legacy `/api/DB/*` endpoints return 404. Fund history now comes from `fonFiyatBilgiGetir` (max 5 years) and info from `fonProfilBilgiGetir`. Some pages sit behind an Akamai TSPD JS challenge — [borsapy v0.9.0](https://github.com/saidsurucu/borsapy/releases).
- `/api/funds/*` is described as a plain HTTP POST JSON endpoint "not behind Akamai protection". It returns 429 after about 4 requests with an approximately 45 s block — [borsapy v0.11.0 and README](https://github.com/saidsurucu/borsapy/releases).

### Inferences
- A personal-finance app needing only one NAV per fund per day could stay under the limit by batching and caching. Still, the robots.txt prohibition and a breaking rewrite in 2026 make TEFAS a fragile, unofficial source.

### Gaps
- Exact current request shape and keyless status of `fonFiyatBilgiGetir` were not verified live (deliberately not called). No TEFAS ToS/API terms were found. I did not check whether TEFAS or Takasbank offers an official data service.
