# Yahoo Finance blocking of non-browser clients, and how open-source finance tools and Rust Yahoo crates coped (state as of 12 Nov 2026)

Method note: these notes combine primary sources (GitHub issues/PRs/releases, crate/npm source tarballs, forum threads) with a small live probe (19 requests total, 12 Nov 2026, single non-US residential connection; EU location NOT verified; curl 8.21.0 on Windows using Schannel TLS over HTTP/1.1, plus Python 3 urllib on OpenSSL 3.5.7). The live-probe results are first-hand observations by this researcher, with the probed endpoint given as the "source".

## Q1. Which blocking mechanisms has Yahoo used, when, and which signal triggered the 2025 blocks?

### Takeaway
Yahoo has stacked several mechanisms over time: cookie+crumb auth (mid/late 2023), volume/IP-based 429s (recurring, e.g. Nov 2024), and in late Apr–May 2025 a block that was best explained by **client fingerprinting (TLS fingerprint, plus User-Agent strings)**. Python `requests`/OpenSSL and reqwest+native-tls got 429 while curl, browsers and reqwest+rustls worked. As of Nov 2026 the dominant gate on the v8 chart endpoint (from this vantage point) is a **User-Agent denylist at Yahoo's edge**: known library UAs (curl, python-requests, Python-urllib, httpx, Java, Go-http-client, empty) get an instant `429 "Edge: Too Many Requests"`, but any other UA (including `reqwest/0.12`) gets 200, even on a non-browser TLS stack.

### Cited Findings

**Timeline**
- **Jul 2023**: yfinance 0.2.23 (2023-07-13) "Fix 'Unauthorized' error". This was the first sign of the auth tightening — [yfinance releases](https://github.com/ranaroussi/yfinance/releases)
- **Nov 2023**: yfinance 0.2.32 (2023-11-18) "Add cookie & crumb to requests, fix various HTTP errors. #1657". The crumb/cookie era starts, followed by 0.2.33 (2023-12-06) cookie fixes — [yfinance releases](https://github.com/ranaroussi/yfinance/releases)
- **Jun 2024**: yahoo_finance_api (Rust) issue #42 "429: Too Many Requests" was opened 2024-06-01. The reporter had about a week of use, then persistent 429s, and asked to be allowed to set a custom user agent. The maintainer closed it 2024-06-17: "The issue is solved with version 2.2.0". The cause was not documented — [xemwebe/yahoo_finance_api#42](https://github.com/xemwebe/yahoo_finance_api/issues/42)
- **Nov 2024**: yfinance #2128 "New rate-limiting" (2024-11-15, v0.2.49): 429 after about 950 tickers, with the crumb value literally `'Edge: Too Many Requests'` and the client toggling between the csrf and basic cookie strategies. It was closed as a duplicate of #2125 — [yfinance#2128](https://github.com/ranaroussi/yfinance/issues/2128)
- **Jan 2025**: yfinance 0.2.52 (2025-01-18) "raise `YfRateLimitError` if rate limited #2108" — [yfinance releases](https://github.com/ranaroussi/yfinance/releases)
- **18 Feb 2025**: yfinance #2276 "not being able to fetch popular stocks such as AAPL" (opened 21:05Z). It was fixed within about an hour by PR #2277 "Change user-agent headers" (merged 22:14Z, released as 0.2.54). The PR replaced the hard-coded ancient UA `...Chrome/39.0.2171.95...` with a random pick from a list of current Chrome 133/Firefox 135/Safari 18.3/Edge UAs. **This is a UA-based block** — [yfinance PR #2277](https://github.com/ranaroussi/yfinance/pull/2277), [yfinance#2276](https://github.com/ranaroussi/yfinance/issues/2276)
- **28 Feb–1 Mar 2025**: GnuCash/Finance::Quote users saw getcrumb failures on `query2.finance.yahoo.com/v1/test/getcrumb`. With a hard-coded crumb they then got "HTTP response 429 (Too Many Requests)" — [gnucash-user, 1 Mar 2025](https://lists.gnucash.org/pipermail/gnucash-user/2025-March/115735.html)
- **~28–29 Apr 2025 (major episode)**:
  - yfinance #2422 "[0.2.57] YFRateLimitError('Too Many Requests" was opened 2025-04-30; the reporter tried several IPs — [yfinance#2422](https://github.com/ranaroussi/yfinance/issues/2422)
  - The Portfolio Performance (PP) forum thread starts 30 Apr 2025, with "429 on all Yahoo updates" in PP 0.75.1 — [PP forum 32780](https://forum.portfolio-performance.info/t/yahoo-finance-too-many-requests-getting-errors-on-updates-desktop-only/32780)
  - The German PP thread dates the start to about 28 Apr — [PP forum 32845](https://forum.portfolio-performance.info/t/downloadfehler-bei-versuch-1-yahoo-finance/32845)
- **30 Apr 2025**: a key diagnostic comment on yfinance #2422 (TianqiMikeHu): the endpoint "works for me on a browser. It works with curl too. But somehow it just doesn't work with the Python requests module (with identical headers in all tests), consistently getting 429… I was suspecting they implemented some kind of TLS fingerprinting mechanism." The fix was `requests.Session(impersonate="chrome")` from curl_cffi — [yfinance#2422 comment](https://github.com/ranaroussi/yfinance/issues/2422#issuecomment-2840759533)
- **May 2025 (Rust)**: yahoo_finance_api #74 "429: Too Many Requests, TLS fingerprinting issue" (2025-05-04). The reporter made about 10 requests/day and started getting 429s after 1 May; changing the UA did not help, while Python yfinance (curl_cffi) worked. In #76 (2025-05-06), enabling reqwest's `native-tls`/`native-tls-alpn` produced the 429-driven failures, and forcing `.use_rustls_tls()` fixed them ("I can confirm that switching to the rustls backend works here", 2025-05-11). The maintainer committed b6eb431 "Use fixed TLS backend (fixes #76)" on 2025-05-17 — [yahoo_finance_api#74](https://github.com/xemwebe/yahoo_finance_api/issues/74), [yahoo_finance_api#76](https://github.com/xemwebe/yahoo_finance_api/issues/76)
- **6 May 2025 (UA evidence from the PP forum)**:
  - A user's curl tests showed PP's long-standing UA `Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.3.1 Safari/605.1.15` → **429**, while old UAs (Chrome/32, Safari 7.0.1, Firefox/25) → **200**. This was reproducible "on different systems and country IPs".
  - Another user said a current Edge/Chrome 136 UA still got 429.
  - So particular (heavily used) UA strings were being denylisted, independent of TLS — [PP forum 32845](https://forum.portfolio-performance.info/t/downloadfehler-bei-versuch-1-yahoo-finance/32845)
- **6 May 2025**: the PP developer (AndreasB) noted that fingerprinting would explain why the PP mobile app (OS HTTP stack) kept working while desktop (Java) failed. He also said "Not sure if there is a Java implementation" (of impersonation) — [PP forum 32780 p1](https://forum.portfolio-performance.info/t/yahoo-finance-too-many-requests-getting-errors-on-updates-desktop-only/32780)
- **9–13 May 2025**: yfinance follow-ups:
  - The crumb endpoint itself returned 429 (or a crumb body of "Too Many Requests"), handled in #2441/#2480/#2491 (merged 2025-05-13).
  - DNS blocking of `fc.yahoo.com` by ad-blockers was handled in #2483 (2025-05-11).
  - [yfinance PR #2491](https://github.com/ranaroussi/yfinance/pull/2491), [yfinance PR #2483](https://github.com/ranaroussi/yfinance/pull/2483)
- **Jun 2025 (Rust)**: yahoo_finance_api #79 "get_ticker_info(): getcrumb(): 'Too Many Requests'" (2025-06-09, still open):
  - Some users fixed it by forcing rustls.
  - Another user found that a different dependency (ollama-rs) re-enabled native-tls through feature unification, and setting it to rustls fixed it.
  - One user thought rapid cookie→crumb→request sequencing triggered it.
  - On 2026-06-04 the maintainer asked whether this is per-user rate limiting rather than a bug.
  - [yahoo_finance_api#79](https://github.com/xemwebe/yahoo_finance_api/issues/79)
- **31 Dec 2025–1 Jan 2026 (yahoo-finance2/Node)**: #977 "Failed to get crumb, status 429" was opened by dtslvr (the Ghostfolio maintainer).
  - A browser UA, or a `compatible; name` UA, worked, while the library's own UA got 429.
  - One user saw Docker over IPv4 blocked while the host over IPv6 worked.
  - It was fixed in 3.11.1/3.11.2 (2026-01-01) with "defaults: honest working user-agent (fixes #977)". The maintainer added on 2026-01-09: "There is no need to set the user-agent on the latest release."
  - [yahoo-finance2#977](https://github.com/gadicc/yahoo-finance2/issues/977), [yahoo-finance2 releases](https://github.com/gadicc/yahoo-finance2/releases?q=user-agent&expanded=true)
- **21–25 Jan 2026**: yahoo-finance2 #982 "[HTTPError]: Edge: Too Many Requests (temporary Yahoo issue)".
  - The maintainer's own VPN tests from US/UK/FR/ZA/AU/JP worked. He judged it "a temporary issue with Yahoo servers"; affected users were mostly on old v2 / `@next-major` tags.
  - A 28 Feb 2026 report showed one Singapore server IP still blocked while other IPs worked, which suggests per-IP blocks.
  - [yahoo-finance2#982](https://github.com/gadicc/yahoo-finance2/issues/982)

**Live probe, 12 Nov 2026** (`GET https://query1.finance.yahoo.com/v8/finance/chart/AAPL?range=5d&interval=1d`, no cookie/crumb, curl/Schannel, HTTP/1.1) — [endpoint](https://query1.finance.yahoo.com/v8/finance/chart/AAPL?range=5d&interval=1d)
- `curl/8.21.0` (default UA) → **429**, body `Edge: Too Many Requests` (23 bytes). The same happened on query2.
- Empty UA → **429** `Too Many Requests`
- `python-requests/2.32.3`, `python-httpx/0.27.0`, `Java/17.0.2`, `Go-http-client/1.1`, `Python-urllib/3.12` → **429** `Edge: Too Many Requests`
- `Mozilla/5.0 (Windows…) Chrome/130…` → **200** JSON. The same happened on query2, and with forced `--http1.1`.
- `Mozilla/5.0` (bare), `reqwest/0.12`, `okhttp/4.12.0` → **200** JSON
- Python urllib on OpenSSL 3.5.7 with a Chrome UA → **200**. The same stack with its default `Python-urllib/3.12` UA → **429**.

The 429 was instant and deterministic per UA, within seconds of 200 responses from the same IP, so it is not a volume limit. It is an edge rule keyed on the UA string.

**Other mechanisms**
- yfinance's own HTTP abstraction (2026) still documents its reasoning: "plain `requests` cannot replicate the JA3/JA4 fingerprint and HTTP/2 settings that `curl_cffi` provides, so Yahoo Finance may rate-limit or block this client" — [yfinance/_http.py](https://raw.githubusercontent.com/ranaroussi/yfinance/main/yfinance/_http.py)
- IP/volume limits are real but undocumented. Examples: about 950 tickers before 429 in Nov 2024 ([#2128](https://github.com/ranaroussi/yfinance/issues/2128)), and per-IP blocks in Jan–Feb 2026 ([yahoo-finance2#982](https://github.com/gadicc/yahoo-finance2/issues/982)). yfinance 1.8.0 (2026-11-03) warns that `Tickers.calendar`/`Tickers.info` are "use at your own risk of being spam-blocked" — [yfinance releases](https://github.com/ranaroussi/yfinance/releases)

### Inferences
- The 2025 block was not one signal. Feb 2025 was UA-based: the stale Chrome/39 UA was rejected. In late Apr–May 2025 there were two signals:
  - (a) TLS-stack fingerprinting. Python requests/urllib3+OpenSSL and reqwest+native-tls(OpenSSL) were blocked, while curl, browsers and reqwest+rustls passed. The rustls-vs-native-tls result inside the same Rust crate, with identical headers, is the cleanest evidence that TLS fingerprinting mattered then.
  - (b) A denylist of specific UA strings (PP's Safari 17.3.1 UA was blocked even from curl).
  - Dec 2025 was again UA-based (the library-name UA was rejected).
- By Nov 2026, from this vantage point, OpenSSL-based Python with a browser UA passes. So the TLS-fingerprint rule was either relaxed, narrowed, or keyed to fingerprints that newer OpenSSL no longer matches. Treat this as a moving target: one IP and one day of observations.
- The pattern "429 with tiny body `Edge: Too Many Requests`" is Yahoo's edge reject. It is used for both bot-classification and true rate limits, so clients cannot tell the two apart by status code.

### Gaps
- No direct packet-level evidence (JA3/JA4 hashes) of which fingerprints Yahoo blocked in 2025 was found. The evidence is behavioural (same headers, different TLS stack → different result).
- Not tested in Nov 2026: whether reqwest+native-tls (OpenSSL) or Java's JSSE with a non-denylisted UA is blocked today (not tested to stay within the request budget).
- No evidence of HTTP/2 SETTINGS/pseudo-header fingerprinting specifically. It is mentioned only as speculation ([yahoo_finance_api#74](https://github.com/xemwebe/yahoo_finance_api/issues/74)) and in yfinance's code comment. My successful probes used HTTP/1.1, so HTTP/2 is not required.

## Q2. Does the v8 chart endpoint need a cookie/crumb? Does EU consent (guce.yahoo.com) affect API endpoints?

### Takeaway
No. As of 12 Nov 2026 `v8/finance/chart` returns full JSON with no cookie and no crumb, given a non-denylisted UA. By contrast `v7/finance/quote` and `v10/finance/quoteSummary` return 401 without a crumb. EU consent redirects (guce/consent.yahoo.com) affect the HTML/cookie bootstrap (finance.yahoo.com pages, and the CSRF cookie strategy), not the query1/query2 JSON endpoints themselves.

### Cited Findings
- Live probes on 12 Nov 2026:
  - v8 chart with no cookies → 200 JSON on both query1 and query2.
  - `v7/finance/quote?symbols=AAPL` with a browser UA → **401** `{"code":"Unauthorized","description":"User is unable to access this feature - https://bit.ly/yahoo-finance-api-feedback"}`
  - `v10/finance/quoteSummary/AAPL?modules=price` → **401** `"Invalid Crumb"`
  - `GET https://fc.yahoo.com/` → 404 but sets an `A3` cookie. `GET query1…/v1/test/getcrumb` with that cookie → 200 with a crumb string.
  - [v7 quote](https://query1.finance.yahoo.com/v7/finance/quote?symbols=AAPL), [getcrumb](https://query1.finance.yahoo.com/v1/test/getcrumb)
- yfinance's own code (2026) states that "fc.yahoo.com is non-critical: chart API works without its cookie". If getcrumb is rate-limited it continues, because "the target endpoint may not need a crumb (e.g. chart API)". (yfinance still attaches a crumb when it has one.) — [yfinance/data.py](https://raw.githubusercontent.com/ranaroussi/yfinance/main/yfinance/data.py)
- yahoo-finance2 (v4.0.6) marks only the `options`, `quote`, `quoteSummary` and `screener` modules `needsCrumb: true`. `chart` (`https://${YF_QUERY_HOST}/v8/finance/chart/`) does not — [yahoo-finance2 on npm](https://www.npmjs.com/package/yahoo-finance2)
- Portfolio Performance's Yahoo feed uses only `query1.finance.yahoo.com/v8/finance/chart/{symbol}`, for both latest and historical quotes, with no cookie or crumb code at all — [YahooFinanceQuoteFeed.java](https://github.com/portfolio-performance/portfolio/blob/master/name.abuchen.portfolio/src/name/abuchen/portfolio/online/impl/YahooFinanceQuoteFeed.java)
- **Consent handling:**
  - yfinance's "csrf" cookie strategy goes to `https://guce.yahoo.com/consent`, posts to `consent.yahoo.com/v2/collectConsent` and then `guce.yahoo.com/copyConsent`, and discards the consent cookie afterward. The "basic" strategy uses `fc.yahoo.com` — [yfinance/data.py](https://raw.githubusercontent.com/ranaroussi/yfinance/main/yfinance/data.py)
  - yahoo-finance2's getCrumb fetches `https://finance.yahoo.com/quote/AAPL`. If redirected to `guce.yahoo`, it follows `collectConsent`, submits the hidden form fields (max 5 consent redirects), then calls `https://query1.finance.yahoo.com/v1/test/getcrumb` — [yahoo-finance2 on npm](https://www.npmjs.com/package/yahoo-finance2)
  - yfinance-rs uses `https://fc.yahoo.com/consent` as its cookie URL — [yfinance-rs 0.9.1 source](https://docs.rs/crate/yfinance-rs/0.9.1/source/src/core/client/constants.rs)

### Inferences
- For a price-history-only client (the PP model), the v8 chart endpoint lets you skip the whole cookie/crumb/consent dance. That removes the getcrumb endpoint, which has been the most frequent 429 trigger: [yfinance#2128](https://github.com/ranaroussi/yfinance/issues/2128), [yahoo-finance2#977](https://github.com/gadicc/yahoo-finance2/issues/977), [yahoo_finance_api#79](https://github.com/xemwebe/yahoo_finance_api/issues/79).
- Consent redirects matter only if the client bootstraps cookies from finance.yahoo.com HTML from an EU IP. The `fc.yahoo.com` route avoided consent in the libraries above.

### Gaps
- Not tested from a verified EU IP. Whether query1/query2 JSON ever 302s to guce for EU IPs could not be confirmed; no source reported that it does.
- The crumb was obtained, but I did not test how long it stays valid.

## Q3. What each project uses as transport, what it changed, and whether it held

### Takeaway
Every project survived mainly by fixing its **User-Agent** and, in 2025, its **TLS stack**:
- yfinance switched to curl_cffi Chrome impersonation in 0.2.58 (2 May 2025), and it still holds as of 1.8.0 (Nov 2026).
- Portfolio Performance (Apache HttpClient 5 on Java) went to a randomized `Mozilla/5.0 (NNNNNN)` UA.
- yahoo_finance_api (Rust) pinned reqwest to rustls in 4.0.0 (May 2025), and Wealthfolio inherited that fix.
- yahoo-finance2 (Node fetch) moved to `Mozilla/5.0 (compatible; yahoo-finance2/x)` in 3.11.1 (1 Jan 2026).

None of these projects needed full browser emulation, apart from yfinance's precautionary curl_cffi.

### Cited Findings

**yfinance (Python)**
- **0.2.54 (2025-02-18)**: "Hotfix fetch error #2277", a randomized modern browser UA — [releases](https://github.com/ranaroussi/yfinance/releases), [PR #2277](https://github.com/ranaroussi/yfinance/pull/2277)
- **0.2.58 (2025-05-02)**: "switch to `curl_cffi` session to fix false rate-limit errors #2430 [reason = TLS fingerprinting]". PR #2430 "Incorporate curl_cffi to avoid rate limiting and cookie errors" was opened 2025-05-01 and merged 2025-05-02, making curl_cffi the default — [releases](https://github.com/ranaroussi/yfinance/releases), [PR #2430](https://github.com/ranaroussi/yfinance/pull/2430)
- **0.2.59 (2025-05-07)**: "Stop overriding `curl_cffi` user-agent - helps with rate-limit". The impersonated TLS fingerprint should be paired with the matching browser UA. A mismatch hurt — [releases](https://github.com/ranaroussi/yfinance/releases)
- **0.2.60 (2025-05-11)**: cookie reuse fix ("The curl_cffi change broke cookie caching & reuse") and DNS-blocked `fc.yahoo.com` handling — [PR #2483](https://github.com/ranaroussi/yfinance/pull/2483)
- **0.2.62 (2025-06-08)**: "detect rate-limit during crumb fetch #2491" — [CHANGELOG](https://raw.githubusercontent.com/ranaroussi/yfinance/main/CHANGELOG.rst)
- **Side effects**: curl_cffi brought new failure modes. Examples: Windows `curl: (77) error setting certificate verify locations` ([#2450](https://github.com/ranaroussi/yfinance/issues/2450)), curl_cffi 0.14 blocked in 1.0, `curl_cffi>=0.15` forced in 1.2.1 over a CVE, and a fix "with curl_cffi>=0.16" in 1.5.2 — [CHANGELOG](https://raw.githubusercontent.com/ranaroussi/yfinance/main/CHANGELOG.rst)
- **2026**:
  - 1.0 (2025-12-22). 1.4.0 (2026-05-23) made curl_cffi optional, with fallback to `requests` plus a Chrome/131 UA and a warning that Yahoo "may rate-limit or block this client" (#2802, closes #2692); `YF_DISABLE_CURL_CFFI` env var. It also added `yf.Auth` Yahoo-account login.
  - 1.8.0 (2026-11-03) "Force `curl_cffi` >= 0.17". The default session is `Session(impersonate="chrome")`.
  - [releases](https://github.com/ranaroussi/yfinance/releases), [_http.py](https://raw.githubusercontent.com/ranaroussi/yfinance/main/yfinance/_http.py)
- **v8 chart and the crumb**: the crumb is not needed, but yfinance attaches it when available (see Q2) — [data.py](https://raw.githubusercontent.com/ranaroussi/yfinance/main/yfinance/data.py)

**Portfolio Performance (Java desktop)**
- **Transport**: `WebAccess` wraps Apache HttpClient 5 (`org.apache.hc.client5…HttpClientBuilder`, `PoolingHttpClientConnectionManager`, `BasicCookieStore`), so the TLS stack is Java's JSSE — [WebAccess.java](https://github.com/portfolio-performance/portfolio/blob/master/name.abuchen.portfolio/src/name/abuchen/portfolio/util/WebAccess.java)
- **Yahoo feed**: v8 chart only. Latest quote uses params `lang=en-US, region=US, corsDomain=finance.yahoo.com`; history uses `range` 1mo–30y with `interval=1d`. There is no crumb and no 429/backoff handling in the feed — [YahooFinanceQuoteFeed.java](https://github.com/portfolio-performance/portfolio/blob/master/name.abuchen.portfolio/src/name/abuchen/portfolio/online/impl/YahooFinanceQuoteFeed.java)
- **UA trick (current code)**: `getUserAgent()` returns the Safari 17.3.1 UA (the one shown to get 429 in May 2025). Yahoo instead gets `getYahooFinanceUserAgent()`, which returns `"Mozilla/5.0 (" + ThreadLocalRandom.current().nextInt(100000, 999999) + ")"`. This is a random, unique-per-call pseudo-UA that dodges a UA denylist — [OnlineHelper.java](https://github.com/portfolio-performance/portfolio/blob/master/name.abuchen.portfolio/src/name/abuchen/portfolio/util/OnlineHelper.java)
- The same random-UA idea (`Mozilla/5.0 (123456)`) was suggested on the forum on 6 May 2025 — [PP forum 32845](https://forum.portfolio-performance.info/t/downloadfehler-bei-versuch-1-yahoo-finance/32845)
- Commit 97709d8 (2025-06-09) "Add special user agent to generic JSON feed if host is Yahoo finance" extended it to the JSON feed — [OnlineHelper commit history](https://github.com/portfolio-performance/portfolio/commits/master/name.abuchen.portfolio/src/name/abuchen/portfolio/util/OnlineHelper.java)
- **2025 handling**:
  - On 30 Apr AndreasB said PP already paces requests ("not doing more than 1 in parallel"). On 1 May: "Looks like Yahoo has updated its behavior…" — [forum p1](https://forum.portfolio-performance.info/t/yahoo-finance-too-many-requests-getting-errors-on-updates-desktop-only/32780)
  - GitHub #4691 (2025-05-06) asked for throttling in WebAccess. It was closed 2025-05-07 with "a temporary fix has been provided for the desktop version" — [PP#4691](https://github.com/portfolio-performance/portfolio/issues/4691)
  - Forum users described 0.76.1 as an "experimental fix". Results were mixed on 8–9 May; some users still saw 429 on the lookup/search endpoint, and 0.76.2 (9 May 2025) fixed Yahoo security search — [forum p5](https://forum.portfolio-performance.info/t/yahoo-finance-too-many-requests-getting-errors-on-updates-desktop-only/32780?page=5), [newreleases 0.76.2](https://newreleases.io/project/github/portfolio-performance/portfolio/release/0.76.2)
- **Long-term**: no Yahoo-specific breakage thread for 2026 turned up in my searches. A June 2026 PP thread about failing quote updates concerned Boursorama (410; it needs an `X-Requested-With` header), not Yahoo — [PP forum 39448](https://forum.portfolio-performance.info/t/securities-price-update-not-working-since-may-29/39448)

**Wealthfolio (Rust/Tauri)** (the repo has moved to `wealthfolio/wealthfolio`, about 9.4k stars, last pushed 2026-11-11)
- **Market-data crate**: `crates/market-data`, with the provider at `src/provider/yahoo/mod.rs`. It depends on `yahoo_finance_api = "5.0.0"` and workspace `reqwest = 0.13.5`, `default-features = false`, features `["json","rustls","query","form","stream","charset","http2","system-proxy"]` (i.e. rustls, HTTP/2 enabled) — [workspace Cargo.toml](https://github.com/wealthfolio/wealthfolio/blob/main/Cargo.toml), [market-data Cargo.toml](https://github.com/wealthfolio/wealthfolio/blob/main/crates/market-data/Cargo.toml)
- **Client**: `wealthfolio_http::client_builder()` is `reqwest::Client::builder().tls_certs_only(...)` using bundled `webpki-root-certs`. The Yahoo client sets `.https_only(true)`, UA `Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36` and a 30 s timeout, then `YahooConnectorBuilder::build_with_client(client)` — [yahoo/mod.rs](https://github.com/wealthfolio/wealthfolio/blob/main/crates/market-data/src/provider/yahoo/mod.rs), [crates/http](https://github.com/wealthfolio/wealthfolio/tree/main/crates/http)
- **Crumb**: its own implementation.
  - It gets a cookie from `https://fc.yahoo.com` (the `Set-Cookie` header), then `GET query1…/v1/test/getcrumb` with the explicit `Cookie` + UA headers.
  - The crumb is cached in a global `RwLock<Option<CrumbData>>` and cleared on auth failure.
  - It is used for v10 quoteSummary and also appended to a v8 chart dividends call (`…/v8/finance/chart/{}?interval=1d&period1=…&events=div&crumb=…`), even though chart doesn't need it.
  - 429 is detected by string-matching the error message ("rate limit"/"429") → `MarketDataError::RateLimited`.
  - The declared provider rate limit is 2000 req/min, max concurrency 10, min delay 25 ms.
  - [yahoo/mod.rs](https://github.com/wealthfolio/wealthfolio/blob/main/crates/market-data/src/provider/yahoo/mod.rs)
- **2025 fix**: issue #243 (2025-05-19) "Market Data failed to Update - Provider error 429". The maintainer pointed to v1.1.1, and the reporter confirmed it fixed the issue the same day — [wealthfolio#243](https://github.com/wealthfolio/wealthfolio/issues/243)
  - v1.1.1 (19 May 2025) notes: "Yahoo Finance: Upgraded to version 4.0 for better data reliability"; market-data batch size reduced to 3 — [v1.1.1 release](https://github.com/wealthfolio/wealthfolio/releases/tag/v1.1.1)
  - yahoo_finance_api 4.0.0 was published 2025-05-17, the day of the "Use fixed TLS backend (fixes #76)" rustls commit — [crates.io](https://crates.io/crates/yahoo_finance_api), [yahoo_finance_api#76](https://github.com/xemwebe/yahoo_finance_api/issues/76)

**Ghostfolio (Node) / yahoo-finance2**
- Ghostfolio 3.99.0 pins `"yahoo-finance2": "4.0.5"` and instantiates `new YahooFinance({ suppressNotices: ['yahooSurvey'] })` with default options (no custom UA or fetch) — [ghostfolio package.json](https://github.com/ghostfolio/ghostfolio/blob/main/package.json), [yahoo-finance.service.ts](https://github.com/ghostfolio/ghostfolio/blob/main/apps/api/src/services/data-provider/data-enhancer/yahoo-finance/yahoo-finance.service.ts)
- yahoo-finance2:
  - The latest is 4.0.6 (2026-11-08); the 4.x line started 2026-07-11.
  - Transport: `moduleOpts.fetch || envFetch || this._opts.fetch || globalThis.fetch`, i.e. Node's built-in fetch (undici; OpenSSL TLS) by default.
  - Cookies: `tough-cookie` ^6 plus `tough-cookie-file-store` (the `~/yf2-cookies.json` file mentioned in #982).
  - Defaults: `YF_QUERY_HOST: "query2.finance.yahoo.com"`, queue concurrency 4, and UA `Mozilla/5.0 (compatible; yahoo-finance2/${version})`. The code comment reads: "Big thanks to @nocodehummel who figured out a User-Agent that both works but still allows us to identify ourselves honestly". The commented-out previous UA was `${pkg.name}/${pkg.version} (+${repository})`.
  - [yahoo-finance2 on npm](https://www.npmjs.com/package/yahoo-finance2)
- **Fix history**: 3.11.1 (2026-01-01) "defaults: honest working user-agent (fixes #977)". 3.11.2 was the same day; 3.13.0 (2026-01-24) added `--version` and deprecated `@next-major`. Old 2.x users kept hitting 429 for weeks until they upgraded — [yahoo-finance2#977](https://github.com/gadicc/yahoo-finance2/issues/977), [yahoo-finance2#982](https://github.com/gadicc/yahoo-finance2/issues/982)
- A related later issue, #1003 "Yahoo API response drift: getCrumb" (2026-05-29, closed the same day), turned up in a GitHub search; details were not retrieved — [yahoo-finance2#1003](https://github.com/gadicc/yahoo-finance2/issues/1003)

### Inferences
- Long-term durability in 2025–2026 came from (1) not sending a denylisted or stale UA and (2) not using an OpenSSL-default TLS client in mid-2025. Of the four projects, only yfinance adopted browser impersonation, and in 2026 it made impersonation optional.
- PP's random-number UA and yahoo-finance2's `compatible; name/version` UA both still look like a "Mozilla/5.0" UA while not copying any real browser. Combined with my Nov 2026 probe (`reqwest/0.12` → 200), this means Yahoo's current rule is a denylist of common library UAs and a few abused browser UAs, not an allowlist of browsers.

### Gaps
- The exact contents of PP 0.76.1 (May 2025) were not retrieved: the newreleases page for 0.76.1 was a 404, and the GitHub file history listing seemed truncated. It is likely, but unconfirmed, that 0.76.1 introduced `getYahooFinanceUserAgent()`.
- I did not find a Wealthfolio issue reporting Yahoo 429s after May 2025 (GitHub search returned only #243 for "yahoo 429").

## Q4. Rust Yahoo crates: versions, transport, crumb support, blocking issues, working status (Nov 2026)

### Takeaway
`yahoo_finance_api` (xemwebe) is the de facto Rust crate: 5.0.0 (2026-08-31), reqwest 0.13 with rustls (aws-lc-rs), a cookie store, crumb support, an optional `governor` rate limiter and a Chrome/122 UA. `yfinance-rs` (gramistella, 0.9.1, 2026-07-11) uses reqwest 0.12 + rustls with native roots, its own crumb flow, a Chrome/122 UA and retry on 429. The old `yahoo-finance` crate (fbriden, 0.3.0, 2020) is abandoned. Rust evidence from 2025: **reqwest+native-tls was blocked; reqwest+rustls worked**. Feature unification can silently switch a crate back to native-tls.

### Cited Findings

**yahoo_finance_api (xemwebe)**
- **Versions**: 5.0.0 (2026-08-31), 4.2.0 (2026-08-08), 4.1.1 (2026-06-04), 4.1.0 (2025-07-29), 4.0.0 (2025-05-17), 3.0.0 (2025-02-07); about 242k downloads — [crates.io](https://crates.io/crates/yahoo_finance_api)
- **5.0.0 dependencies**:
  - reqwest `0.13.4`, `default-features=false`, features `["json","default-tls","cookies"]`. The lockfile contains `rustls 0.23.40`, `aws-lc-rs 1.17.0` and `ring`, i.e. reqwest 0.13's default TLS is rustls.
  - Optional features: `blocking`, `decimal`, `governor` ("rate-limit requests to avoid HTTP 429 responses", default 10 req/s), `debug`.
  - [docs.rs source 5.0.0](https://docs.rs/crate/yahoo_finance_api/5.0.0/source/Cargo.toml)
- **Endpoints and UA (5.0.0)**: `YCHART_URL = https://query1.finance.yahoo.com/v8/finance/chart`, `Y_GET_COOKIE_URL = https://fc.yahoo.com`, `Y_GET_CRUMB_URL = https://query1.finance.yahoo.com/v1/test/getcrumb`, and a default UA of Chrome/122 on Linux (overridable via the builder `.user_agent()`). `build_with_client` accepts a custom reqwest client (added after #43, Jun 2024).
  - Crumb retry logic refreshes cookie+crumb on 401/403/"Invalid Crumb"/429 for crumb endpoints (quoteSummary, financial events).
  - [docs.rs source 5.0.0](https://docs.rs/crate/yahoo_finance_api/5.0.0/source/src/lib.rs)
- **Maintainer statement**: on 2026-06-04 he said 4.1.1 "updates reqwest to 13.4 and makes rustls the default TLS". Issue #79 (crumb 429) remains open, and PR #92 was merged on 2026-08-08 — [yahoo_finance_api#79](https://github.com/xemwebe/yahoo_finance_api/issues/79)
- **Proxy**: PR #89 "Fix/proxy for all requests" (2026-07-19 → 2026-08-08). PR #94 (2026-08-31) added 20 quoteSummary modules and "harden error handling" — [GitHub issue search results](https://github.com/xemwebe/yahoo_finance_api/issues)
- **Blocking issue history**: #42 (Jun 2024, 429, fixed in 2.2.0), #74/#76 (May 2025, TLS → rustls fix), #79 (Jun 2025, getcrumb 429, open). The details are in Q1 — [#42](https://github.com/xemwebe/yahoo_finance_api/issues/42), [#74](https://github.com/xemwebe/yahoo_finance_api/issues/74), [#76](https://github.com/xemwebe/yahoo_finance_api/issues/76), [#79](https://github.com/xemwebe/yahoo_finance_api/issues/79)

**yfinance-rs (gramistella)**
- **Versions**: 0.9.1 (2026-07-11), 0.9.0 (2026-06-10), 0.8.0 (2026-05-27), 0.7.x (Oct 2025); about 19k downloads — [crates.io](https://crates.io/crates/yfinance-rs)
- **Transport**: reqwest `0.12.24`, `default-features=false`, features `["json","rustls-tls-native-roots","gzip","brotli","deflate"]`. A `stream` feature adds tokio-tungstenite websockets — [docs.rs 0.9.1 Cargo.toml](https://docs.rs/crate/yfinance-rs/0.9.1/source/Cargo.toml)
- **Constants**: the default UA is Chrome/122 on Linux, "to avoid trivial bot blocking". The cookie URL is `https://fc.yahoo.com/consent`, the crumb URL is `query1…/v1/test/getcrumb`, and it also has v7 quote and v10 quoteSummary bases. The retry policy retries on `[408, 429, 500, 502, 503, 504]`. A dedicated `YfError::RateLimited` is returned for 429 — [docs.rs 0.9.1 source](https://docs.rs/crate/yfinance-rs/0.9.1/source/src/core/client/constants.rs)
- **Changelog**: crumb-auth retries are centralized. Stale crumbs are cleared on 401/403 or invalid-crumb bodies, and the stored cookie is sent explicitly during crumb acquisition "so custom reqwest clients no longer need" a cookie store — [docs.rs 0.9.1 CHANGELOG](https://docs.rs/crate/yfinance-rs/0.9.1/source/CHANGELOG.md)
- A GitHub issue search for "429" on gramistella/yfinance-rs returned no issues.

**Others**
- `yahoo-finance` (fbriden/yahoo-finance-rs): 0.3.0 (2020-09-09), unmaintained, and predates crumbs — [crates.io](https://crates.io/crates/yahoo-finance)
- `yfinance` (elminson): 0.1.0 (2024-02-14) only — [crates.io](https://crates.io/crates/yfinance)
- `eeyf`: 0.1.0 (2025-10-06), 322 downloads — [crates.io](https://crates.io/crates/eeyf)
- `finalytics` has its own Yahoo module (`src/data/yahoo/web.rs`) — [docs.rs finalytics 0.9.0](https://docs.rs/crate/finalytics/0.9.0/source/src/data/yahoo/web.rs)

**Rust-specific blocked-vs-working reports**
- May 2025: reqwest + `native-tls`/`native-tls-alpn` → 429 (surfacing as `DeserializeFailed(... line 1 column 1)` because the 429 body isn't JSON); `.use_rustls_tls()` → works. In the words of one user: "different TLS backends behave differently" — [yahoo_finance_api#76](https://github.com/xemwebe/yahoo_finance_api/issues/76)
- "Changing the user agent" alone did not fix the May 2025 block for reqwest+native-tls — [yahoo_finance_api#74](https://github.com/xemwebe/yahoo_finance_api/issues/74)
- Jun 2025: an unrelated dependency (ollama-rs default features) pulled native-tls back in through Cargo feature unification and re-triggered 429. Setting it to rustls fixed it — [yahoo_finance_api#79](https://github.com/xemwebe/yahoo_finance_api/issues/79)
- Nov 2026 live probe: UA `reqwest/0.12` on a non-browser TLS stack → 200 on v8 chart; `curl/…` → 429. The UA string, not "being Rust", is what matters today — [endpoint](https://query1.finance.yahoo.com/v8/finance/chart/AAPL?range=5d&interval=1d)

### Inferences
- A Rust client in Nov 2026 is very likely fine with reqwest+rustls, a non-denylisted UA (browser-like or `Mozilla/5.0 (compatible; app/x)`), and v8 chart without a crumb. It should be checked with `cargo tree -e features -i native-tls` to make sure native-tls isn't unified in.
- Whether rustls's own fingerprint remains acceptable is not guaranteed. The 2025 rustls fix worked because rustls's ClientHello differed from the blocked OpenSSL fingerprint, not because it looked like a browser.

### Gaps
- No public report was found of rustls-based clients being blocked by Yahoo as of Nov 2026.
- No measurements were found of JA3/JA4 for rustls+aws-lc-rs vs Chrome in a Yahoo context (out of scope here; another researcher covers impersonation crates).

## Q5. Official Yahoo statements

### Takeaway
I found no official Yahoo statement on rate limits, fingerprinting or unofficial API use for 2023–2026. The only "official" signal is the 401 body itself, which points to a feedback link.

### Cited Findings
- Yahoo's v7 quote endpoint returns `"User is unable to access this feature - https://bit.ly/yahoo-finance-api-feedback"` without a crumb (live probe, 12 Nov 2026) — [v7 quote](https://query1.finance.yahoo.com/v7/finance/quote?symbols=AAPL)
- Third parties state that Yahoo does not publish its limits ("rate-limits its endpoints at its own discretion and does not publish the limits"). This is a vendor comparison, not Yahoo — [Massive vs yfinance](https://massive.com/compare/massive-vs-yfinance)
- In May 2025 a PP forum user linked Yahoo's DSP (ads) API rate-limit documentation, then noted it is not the quotes API — [PP forum 32780 p2](https://forum.portfolio-performance.info/t/yahoo-finance-too-many-requests-getting-errors-on-updates-desktop-only/32780?page=2)

### Inferences
- All mechanisms described above are reverse-engineered from client behaviour. Any design should assume rules change without notice. Episodes so far: Jul 2023, Nov 2023, Jun 2024, Nov 2024, Feb 2025, Apr–May 2025, Dec 2025–Jan 2026.

### Gaps
- No Yahoo press or developer statement was found. Searches surfaced only scraper vendors and SEO articles, which I excluded as unreliable.
- I did not check Reddit r/algotrading threads; search results were dominated by GitHub, forums and vendors.
