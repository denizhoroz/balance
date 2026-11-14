# Metal price APIs: alternatives to gold-api.com (XAU / XAG)

Researched 2026-11-12. Every "I called it" sample below comes from a real request made that day from a Windows machine in Europe (CDN PoP FRA). Claims are cited inline. Anything I could not confirm from a primary source is marked **unverified**.

Context: `app/src/core/rates.rs` fetches `GET https://api.gold-api.com/price/{XAU|XAG}` (keyless, USD per troy oz). `Source::Metal(m)` makes one call per metal, converts to USD per gram, and cross-converts to fiat through ECB rates from Frankfurter. Results are cached per day in `.cache/metal.json`. The app ships to end users, so any key would sit inside the binary.

---

## 1. TL;DR

1. **Primary: keep gold-api.com.** It is keyless, its ToS explicitly allows commercial use in apps, it updates every 30 s, and it adds no rate limit on `/price` beyond banning several requests per second. It also takes a `/price/XAU/EUR`-style currency suffix (19 currencies, but **not TRY**). This is the best fit found. ([FAQ](https://gold-api.com/faqs), [ToS §4, §9](https://gold-api.com/terms))
2. **Fallback A (no new host): CoinGecko, already used for crypto.** CoinGecko quotes `xau` and `xag` as vs-currencies. Add `xau,xag` to the existing `simple/price` call and you get USD/oz = `btc.usd / btc.xau`, plus XAG, at no extra cost. Alternatively, one `/exchange_rates` call returns BTC priced in USD, TRY, XAU and XAG together. The derived value was within 0.05% of gold-api spot. The app already carries the "Powered by CoinGecko" attribution obligation, so this adds nothing new. Risk: CoinGecko doesn't document where its XAU rate comes from or how fresh it is (**unverified**). ([supported_vs_currencies](https://api.coingecko.com/api/v3/simple/supported_vs_currencies), [API terms](https://www.coingecko.com/en/api_terms))
3. **Fallback B (independent, daily, CC0): fawazahmed0 currency-api.** It is a static JSON file on jsDelivr, with Cloudflare Pages as a mirror. `xau.json` gives one gold ounce in every currency, **including TRY**, and `usd.json` gives both `xau` and `xag` in one file. It has no key, no rate limit and a CC0 licence, but it updates **once a day** and was +0.25% off live spot when I checked. That is good enough as a last resort for a once-a-day cache. ([README](https://github.com/fawazahmed0/exchange-api))

Optional: a tokenized-gold ticker (Kraken / Coinbase / Binance PAXG or XAUT) is a keyless real-time *proxy*, typically 0–0.3% below spot. It covers gold only, not silver.

Do **not** use any of these for a distributed client: the keyed providers (metals.dev, metals-api, metalpriceapi, goldapi.io, api-ninjas, commoditypriceapi), whose free tiers ban commercial use and have quotas shared by every user; Kitco's `proxy.kitco.com`, which works but is undocumented and Kitco's ToS ban automated access; Yahoo; Swissquote; or Turkish scraped feeds.

---

## 2. Comparison table

| Name | Auth | Free limit | Freshness | XAU+XAG in 1 call | Currencies / unit | Client-embed OK? | Risk |
|---|---|---|---|---|---|---|---|
| **gold-api.com** (baseline) | none for `/price` | "no rate limit" on `/price`; history/OHLC 10 req/h with key | 30 s | No (1 per metal; `XAU,XAG` → 404) | USD + 18 others via `/price/XAU/EUR`; **no TRY**; troy oz | **Yes**: commercial use in apps explicitly allowed, no attribution needed | Low–med: small operator, ToS changeable, IP ban if spammy |
| **CoinGecko** (derived via vs_currency `xau`/`xag`) | none (keyless public) or free Demo key | keyless: IP-based, no number given; Demo 100/min | **unverified** for the XAU leg (BTC leg ~live) | **Yes** (`/exchange_rates` or `simple/price?vs_currencies=usd,xau,xag`) | any vs-currency incl. TRY; troy oz | Yes with "Powered by CoinGecko" attribution, which the app already needs | Med: XAU source and freshness undocumented; ~4–5 significant figures |
| **fawazahmed0 currency-api** | none | "No Rate limits" | daily | **Yes** (`usd.json` has `xau` and `xag`) | ~300 codes incl. **TRY**; troy oz ("Gold Ounce") | Yes, CC0-1.0 | Med: one maintainer, daily only, +0.25% vs spot observed |
| Kraken PAXG/XAUT | none | public (not checked) | real-time trades | No: gold proxy only (no silver) | USD, EUR | Documented public endpoint | Token tracking error (−0.25% observed); gold only |
| Coinbase PAXG | none | not checked | real-time | No: gold only | USD, **TRY**, EUR… | Public endpoint | As above |
| Binance PAXG/XAUT | none (`data-api.binance.vision`) | not checked | real-time | Yes for both tokens, but both are gold | USDT, **TRY** (PAXGTRY) | Documented public market data | As above; Binance geo-blocks some countries (**unverified** for TR) |
| **Kitco** `proxy.kitco.com/getPM` | none | none published | ~30–60 s (CloudFront `Age: 3`) | **Yes** (`symbol=AU,AG`) | USD, EUR, GBP, JPY, CHF…; **TRY → 404**; ounce or gram | **No**: undocumented, ToS §7.2 bans automated access, data is personal non-commercial use only | **High** |
| metalsdaily.com | no API | — | — | — | — | No: HTML/NetDania widgets; robots.txt blocks AI agents | High (scrape) |
| dailymetalprice.com | no documented API | — | — | — | — | No: Cloudflare challenge; robots.txt blocks AI agents | High / unverified |
| metals.dev | key | 100 req/**month** | 60 s | Yes (`/v1/latest`) | many incl. TRY; toz, g, kg… | Commercial use only with **active subscription** | High for a client: shared quota, key in binary |
| metals-api.com | key | "Free" plan exists; quota **unverified**; paid from $39.99/mo | 10 min (Bronze) – 60 s | Yes | many | **No** on free ("personal use and for testing use only") | High |
| metalpriceapi.com | key | 100 req/month, **daily** updates | daily (free) | Yes | many | **No** on free ("must not be used for any commercial activities") | High |
| goldapi.io | key (`x-access-token`) | 150 req/month | "2 sec interval" | No (`/api/XAU/USD`, one per call) | many; gives gram + karat prices | ToS **unverified** (SPA wouldn't render) | High: tiny shared quota |
| api-ninjas commodityprice | key | free = evaluation only | free: daily close per secondary source (**unverified**) | `/v1/commoditysnapshot` returns all | USD | **No**: free plan non-commercial | High |
| commoditypriceapi.com | key | no free plan seen; $34.99/mo+ | ≤1 s | Up to 5 symbols/request | — | Commercial licence from Standard tier | High (cost) |
| commodities-api.com | key | **unverified** | — | — | — | — | Unverified |
| Frankfurter / ECB | none | — | daily | — | **no XAU/XAG** (verified) | — | N/A |
| open.er-api.com | none | — | daily | — | **no XAU/XAG** (verified) | — | N/A |
| exchangerate.host | **key now** (APILayer) | — | — | — | — | — | Was keyless; no longer |
| Yahoo Finance chart (GC=F, SI=F) | none (unofficial) | — | ~live | No (1 symbol/call on chart) | USD; **futures**, not spot | No: unofficial, no API ToS | High |
| stooq.com CSV | — | — | — | — | — | — | **Dead/blocked**: 404 + JS bot challenge |
| Swissquote public-quotes | none (undocumented) | — | live bid/ask | No | USD, EUR OK; **XAU/TRY returned bad, ~10 h stale data** | No: undocumented | High |
| LBMA prices JSON | — | — | daily fix | — | USD/GBP/EUR | — | **403** (Cloudflare); licensed product |
| TCMB today.xml | none | — | daily | — | **no gold** (verified) | Official | N/A |
| TCMB EVDS | key | — | daily | — | TRY | Official, but key needed; gold series **unverified** | Med |
| truncgil finans (`finans.truncgil.com/v4/today.json`) | none | — | ~minutes | **Yes**, gram gold and silver in TRY | TRY per gram | ToS **unverified**; aggregator, scraped origin unclear | High |
| genelpara / Harem / Kapalıçarşı | — | — | — | — | — | — | 404 / Cloudflare block / no response |

---

## 3. Per-candidate details

### 3.1 gold-api.com (baseline)

- **Endpoints:** base `https://api.gold-api.com`. `GET /symbols`, `GET /price/{SYMBOL}`, `GET /price/{SYMBOL}/{CCY}` (free, keyless), plus `/history` and `/ohlc`, which need an `x-api-key` header. ([docs](https://gold-api.com/docs), [FAQ](https://gold-api.com/faqs))
- **Limits:** "The real-time price endpoint is free with unlimited requests. The price history and OHLC endpoints are also free for up to 10 requests per hour." On spam: "usually considered sending multiple requests per second to the same exact endpoint … IPs that spam the API may be banned." ([FAQ](https://gold-api.com/faqs); [ToS §4](https://gold-api.com/terms)) Premium costs $10/mo and only adds history/OHLC. ([pricing](https://gold-api.com/pricing))
- **Freshness:** "Live prices are updated every 30 seconds." I observed the response header `Cache-Control: public, max-age=25`. The FAQ calls it "the spot price". ([FAQ](https://gold-api.com/faqs))
- **Currencies/unit:** troy ounce. "Supported currencies are USD, EUR, GBP, JPY, CAD, AUD, CHF, CNY, HKD, SGD, SEK, NOK, DKK, NZD, MXN, INR, BRL, ZAR and KRW." TRY is not on that list, and I confirmed it fails (below). ([FAQ](https://gold-api.com/faqs))
- **ToS:** "Commercial use of the API is always permitted, including use in web applications, mobile apps, and third-party websites…" (§9). "Attribution of this service is not required but it's appreciated." CORS is enabled. Effective date 10 Sep 2026; governed by Utah law; terms "may be modified … at any time" (§11). ([ToS](https://gold-api.com/terms), [FAQ](https://gold-api.com/faqs))
- **Longevity:** the site says "If one of our data providers go down, the system will fallback to the next source." The operator isn't named on the site, beyond "the CEO of gold-api.com" (**unverified** who runs it). The homepage shows 5-year price history, which is weak evidence of a long-running dataset. ([home](https://gold-api.com/))
- **I called it on 2026-11-12:**
  ```
  GET /price/XAU      -> 200 {"currency":"USD","currencySymbol":"$","exchangeRate":1.0,"name":"Gold","price":3963.0,"symbol":"XAU","updatedAt":"2026-11-12T07:51:31Z","updatedAtReadable":"a few seconds ago"}
  GET /price/XAG      -> 200 {"currency":"USD",...,"name":"Silver","price":51.401001,"symbol":"XAG","updatedAt":"2026-11-12T07:51:31Z",...}
  GET /price/XAU/EUR  -> 200 {"currency":"EUR","currencySymbol":"€","exchangeRate":0.8999,"name":"Gold","price":3566.3937781902,...}
  GET /price/XAU/TRY  -> 404 {"error": "Currency not found"}
  GET /price/XAU,XAG  -> 404 {"error": "Symbol not found"}
  GET /symbols        -> 200 [{"name":"Silver","symbol":"XAG"},{"name":"Gold","symbol":"XAU"},{"name":"Bitcoin","symbol":"BTC"},{"name":"Ethereum","symbol":"ETH"},{"name":"Palladium","symbol":"XPD"},{"name":"Copper","symbol":"HG"},{"name":"Platinum","symbol":"XPT"}]
  ```
  Note: the response now has extra fields (`currencySymbol`, `exchangeRate`, `name`, `updatedAtReadable`). The current `MetalResponse { price, currency }` struct ignores them, so nothing breaks.

### 3.2 CoinGecko: XAU/XAG as vs-currencies (derived)

- **What:** CoinGecko lists `xau` and `xag` among `supported_vs_currencies`, and `/exchange_rates` labels them `"type":"commodity"` with names "Gold - Troy Ounce" and "Silver - Troy Ounce". USD per oz = `price_usd / price_xau` for any coin. ([supported_vs_currencies](https://api.coingecko.com/api/v3/simple/supported_vs_currencies), [exchange_rates docs](https://docs.coingecko.com/reference/exchange-rates))
- **Auth/limits:** keyless works, with "IP-based rate limiting — shared across all users on the same IP". A free Demo key gets "100 calls/min". ([rate limits](https://docs.coingecko.com/docs/common-errors-rate-limit))
- **ToS:** you must display "'Powered by CoinGecko' in a legible font" no smaller than 10. You may not "re-distribute or syndicate access to the CoinGecko API". On caching: "You should refresh the cache at least every 24 hours". A product offered to others needs "a binding user agreement and privacy policy". ([API terms](https://www.coingecko.com/en/api_terms)) The app already uses CoinGecko for crypto, so these obligations already apply.
- **Freshness:** **unverified**. The docs page's "Cache / Update Frequency" field is blank, and CoinGecko doesn't say where the commodity rate comes from.
- **Precision:** `xau` arrives with 4–5 significant figures (e.g. 18.539), so error is about ±0.005%, which is fine.
- **I called it on 2026-11-12:**
  ```
  GET /api/v3/simple/price?ids=bitcoin&vs_currencies=usd,try,eur,xau,xag&include_last_updated_at=true
  -> {"bitcoin":{"usd":73536,"try":3695175,"eur":66343,"xau":18.57,"xag":1434.11,"last_updated_at":1794469964}}
     => XAU = 73536/18.57 = 3960.0 USD/oz ; XAG = 73536/1434.11 = 51.28 USD/oz

  GET /api/v3/exchange_rates   (BTC base)
  -> usd 73500.094 (fiat), try 3692544.574 (fiat), xau 18.539 "Gold - Troy Ounce" (commodity), xag 1427.159 "Silver - Troy Ounce" (commodity)
     => XAU = 3964.6 USD/oz ; XAG = 51.50 USD/oz ; also gives TRY/USD

  GET /api/v3/simple/price?ids=pax-gold,tether-gold&vs_currencies=usd,try,xau,xag
  -> {"pax-gold":{"usd":3953.04,"try":198640,"xau":0.998064,"xag":77.09},"tether-gold":{"usd":3951.52,"try":198563,"xau":0.997679,"xag":77.06}}
  ```
  The `simple/price` figures (3960.0 / 51.28) were 0.08% / 0.24% from gold-api; the `exchange_rates` figures (3964.6 / 51.50) were 0.04% / 0.19%. The `xag` leg rounds to fewer digits, which may explain some of the silver gap.

### 3.3 fawazahmed0/exchange-api ("currency-api")

- **Endpoints:** `https://cdn.jsdelivr.net/npm/@fawazahmed0/currency-api@{date}/v1/currencies/{code}.json`, with fallback `https://{date}.currency-api.pages.dev/v1/currencies/{code}.json`, where `{date}` is `latest` or `YYYY-MM-DD`. The README asks you to "include Fallback mechanism in your code". It describes the data as "Daily Updated" and has "No Rate limits". Licence CC0-1.0. ([README](https://github.com/fawazahmed0/exchange-api))
- **Metals:** `currencies.json` lists `"xau": "Gold Ounce"`, `"xag": "Silver Ounce"`, `xpt`, `xpd` (I called it).
- **Freshness gotcha:** jsDelivr's `@latest` lagged. At the same moment it served `"date": "2026-11-11"` for `xau.json` while pages.dev served `2026-11-12`. Header: `cache-control: public, max-age=604800, s-maxage=43200`, `Age: 4773`. Prefer pages.dev for `latest`, or check `date`.
- **Data source:** not stated in the README (**unverified**).
- **I called it on 2026-11-12:**
  ```
  GET https://latest.currency-api.pages.dev/v1/currencies/xau.json
  -> {"date": "2026-11-12", "xau": { ..., "eur": 3586.44891428, "try": 199632.47855607, "usd": 3972.79114799, ... }}
     => 199632.48 / 31.1034768 = 6418 TRY per gram
  GET https://latest.currency-api.pages.dev/v1/currencies/usd.json
  -> {"date": "2026-11-12", "usd": { ..., "try": 50.24993037, "xag": 0.019436425, "xau": 0.0002517122, ... }}
     => XAU = 1/0.0002517122 = 3972.8 USD/oz ; XAG = 1/0.019436425 = 51.45 USD/oz
  ```
  `xau.min.json` is 7.2 KB and `usd.min.json` 6.1 KB. Gold was +0.25% vs gold-api live, as expected for a daily snapshot.

### 3.4 Tokenized gold: PAXG / XAUT on Kraken, Coinbase, Binance

- **What:** PAXG (Paxos) and XAUT (Tether) are tokens each backed by one troy oz of gold (issuer claims; **unverified** here). They are a gold proxy only; there is no silver equivalent. Exchange prices track spot but trade at a small discount or premium, and the gap widens in thin markets and on weekends, when crypto trades and gold doesn't.
- **Auth:** Kraken Ticker is public (`security: []`) ([Kraken docs](https://docs.kraken.com/api/docs/rest-api/get-ticker-information)). Binance's `data-api.binance.vision` "do[es] not require any authentication … and serve[s] only public market data" ([Binance FAQ](https://developers.binance.com/docs/binance-spot-api-docs/faqs/market_data_only)). Coinbase `/v2/prices/{pair}/spot` answered without a key (docs not checked; **unverified** ToS).
- **I called them on 2026-11-12 (spot reference: gold-api 3963.0, Kitco mid 3961.1, Swissquote ~3961.4):**
  ```
  Coinbase  GET https://api.coinbase.com/v2/prices/PAXG-USD/spot   -> {"data":{"amount":"3952.41","base":"PAXG","currency":"USD"}}
  Coinbase  GET https://api.coinbase.com/v2/prices/PAXG-TRY/spot   -> {"data":{"amount":"198813.232950838302257601","base":"PAXG","currency":"TRY"}}
  Coinbase  GET https://api.exchange.coinbase.com/products/PAXG-USD/ticker -> {"ask":"3953.46","bid":"3951.71",...,"price":"3951.85","time":"2026-11-12T07:50:25.978576370Z"}
  Kraken    GET https://api.kraken.com/0/public/Ticker?pair=PAXGUSD,XAUTUSD
            -> {"error":[],"result":{"PAXGUSD":{"a":["3952.630000",..],"b":["3951.940000",..],"c":["3951.920000",..],...},"XAUTUSD":{"a":["3950.50000",..],"b":["3950.40000",..],"c":["3951.30000",..],...}}}
  Binance   GET https://api.binance.com/api/v3/ticker/price?symbols=["PAXGUSDT","PAXGTRY"] -> [{"symbol":"PAXGUSDT","price":"3957.05000000"},{"symbol":"PAXGTRY","price":"198690.00000000"}]
  Binance   GET https://data-api.binance.vision/api/v3/ticker/price?symbols=["PAXGUSDT","XAUTUSDT"] -> [{"symbol":"PAXGUSDT","price":"3961.27000000"},{"symbol":"XAUTUSDT","price":"3959.05000000"}]
  ```
  Observed tracking error: −0.27% (Coinbase/Kraken) to about 0% (Binance), measured against spot. Binance quotes are in USDT, not USD. A one-token-per-oz reading of PAXGTRY gives 198690 / 31.1035 = 6388 TRY/g.

### 3.5 Kitco (added on request)

- **Official API?** None offered. The ToS ([kitco.com/terms-of-use](https://www.kitco.com/terms-of-use)) ban automated access: §7.2 forbids any "robot, spider, site search/retrieval application" used to reproduce content. Content is "personal, non-commercial use only", and resale or redistribution is prohibited (§8, §10). For licensing the ToS only point to Customer Service ([corp.kitco.com/contact-us.html](https://corp.kitco.com/contact-us.html)). Kitco's site navigation also links to "Free Web Quote Banners" and "Free Charts on Your Website" on kitconet.com, but these are images for web pages. robots.txt for www.kitco.com is `User-agent: * Allow: /`.
- **Legacy feeds checked on 2026-11-12:**
  - `https://proxy.kitco.com/getPM?symbol=…&currency=…&unit=…` is **live**. It is undocumented (I found no public docs) and is the quote backend behind Kitco's own pages. It returns plain-text CSV, needs no key, sets `Access-Control-Allow-Origin: *`, and sits behind CloudFront (`X-Cache: Hit from cloudfront`, `Age: 3`). It covers **AU and AG in one call**, in ounce or gram units, and in USD, EUR, GBP, JPY and CHF. **TRY returned 404.**
    ```
    GET https://proxy.kitco.com/getPM?symbol=AU,AG&currency=USD&unit=ounce
    -> AU,USD,OUNCE,2026-11-12 02:52:59,3960.10,3961.10,3962.10,-26.90,-0.67,3954.60,4001.20
       AG,USD,OUNCE,2026-11-12 02:52:59,51.24,51.37,51.49,-0.30,-0.59,50.98,52.07
    GET https://proxy.kitco.com/getPM?symbol=AU&currency=EUR&unit=gram
    -> AU,EUR,GRAM,2026-11-12 02:53:29,114.87,114.90,114.92,-0.75,-0.65,114.69,116.04
    GET https://proxy.kitco.com/getPM?symbol=AU,AG&currency=TRY&unit=gram -> 404 (IIS "File or directory not found")
    ```
    My reading of the columns, inferred and not documented: metal, currency, unit, timestamp (New York time; 02:52 EST = 07:52 UTC), bid, mid, ask, change, change %, day low, day high.
  - `https://www.kitco.com/texten/texten.html` (the classic text quotes page) now 301-redirects to `/price/precious-metals/text-quotes`, an HTML page.
  - `https://www.kitco.com/charts/livegold.html` 302-redirects to `/price/precious-metals/gold`, also HTML.
  - `https://www.kitconet.com/charts/metals/gold/t24_au_en_usoz_2.gif` is **live**, but it is a 172×114 GIF chart image (the official embeddable chart), not machine-readable.
- **Verdict:** the data is good, and the CSV even has bid/mid/ask and a gram unit. But the endpoint is undocumented and the ToS forbids automated use and redistribution. Kitco could block it at any time, and shipping it in a client violates its terms. **Do not use.**

### 3.6 metalsdaily.com (added on request)

- **Official API?** None found. Its live-price pages render NetDania streaming widgets: chart images at `balancer.netdania.com/StreamingServer/StreamingServer?chartimage&sym=XAUUSD&…&group=metalsdaily.com`. A site script (`js/prData/jquery.pData.js`) polls an internal `POST __dta/pd.ashx` every ~1.5 s. That is an undocumented private endpoint, so scraping it would be HTML/XHR scraping. `/rss-feed/` returned the 404 page. (I fetched https://www.metalsdaily.com/ and https://www.metalsdaily.com/live-prices/gold/ on 2026-11-12.)
- **robots.txt:** `Content-Signal: search=yes,ai-train=no,use=reference`, and it explicitly `Disallow: /` for ClaudeBot, anthropic-ai, Claude-Web, GPTBot and others ([robots.txt](https://www.metalsdaily.com/robots.txt)). I stopped probing after reading it, so I did **not** call `__dta/pd.ashx`. Terms of use: not found or **unverified**.
- **Verdict:** a news aggregator whose prices come from NetDania, which licenses its data commercially. Scraping risk only. **Do not use.**

### 3.7 dailymetalprice.com (added on request)

- The homepage returned **HTTP 403** with a Cloudflare "Just a moment..." JS challenge on 2026-11-12, and so did the guessed paths `metalpricescsv.php` and `metalprices.php`. Its [robots.txt](https://www.dailymetalprice.com/robots.txt) has the same Cloudflare content-signal block, disallowing ClaudeBot, anthropic-ai and others, so I stopped there.
- I found no documented API, CSV export or terms in search results (**unverified**). Anything a ureq client got would come through a bot challenge, so it is not usable from a desktop app in practice.
- **Verdict:** **not usable**, at best a scrape behind a bot wall.

### 3.8 metals.dev

- **Auth:** API key (`api_key` query param). Keyless call → `401 {"status":"failure","error_code":1101,"error_message":"Unauthorized. ... Provide a valid 'api_key' in the request."}` (I called it).
- **Limits/price:** Free plan $0, "100 requests per month", "60 second updates". Copper $1.79 for 2,000/mo; $9.99 for 10,000/mo; up to $99.99 for 500,000/mo. ([pricing](https://metals.dev/pricing))
- **Shape:** `GET https://api.metals.dev/v1/latest?api_key=…&currency=USD&unit=toz` returns `"unit":"toz","metals":{"gold":…,"silver":…,"platinum":…,"lbma_gold_am":…,"lbma_gold_pm":…,"lbma_silver":…}`. One call covers all metals. `unit` supports g, kg, toz and others, and `currency` includes TRY. ([docs](https://metals.dev/docs))
- **ToS:** you may publish rates in apps commercially "as long as you maintain an active subscription". "No attribution to Metals.Dev is required". "Multiple user accounts created by an individual … will be disabled". ([terms](https://metals.dev/policy/terms))
- **Verdict:** best data shape (both metals, gram, TRY, LBMA fixes in one call), but the key would be embedded and every user would share 100 calls/month. It only works with a paid key, and even then the key is extractable from the binary.

### 3.9 metals-api.com

- **Auth:** `access_key`. Keyless → `401 {"success":false,"error":{"code":401,"type":"invalid_access_key",...}}` (I called it).
- **Price:** Bronze $39.99/mo for 5,000 calls, "10-minute updates"; Silver $79.99/mo, "60-second updates". A free sign-up exists, but its quota is **unverified**. ([pricing](https://metals-api.com/pricing))
- **ToS:** "The Free plan cannot be used for any commercial or public use or redistribution under any circumstances. The responses provided in the free plan are licensed for personal use and for testing use only." Paid plans may "display … to individual end-users of your application(s)". ([terms](https://metals-api.com/terms))
- **Verdict:** not suitable.

### 3.10 metalpriceapi.com

- **Auth:** `api_key`. Keyless → `{"success":false,"error":{"statusCode":101,"message":"User did not supply an API Key"}}` (I called it).
- **Limits:** Free: "100 Requests / month", "Daily updates". Paid from $5/mo (billed yearly) for 1,000/mo at 30-min updates; 60 s updates start at the $32/mo tier. ([pricing](https://metalpriceapi.com/pricing))
- **ToS:** "Data accessed through the API with the Free Plan is intended solely for personal exploration and evaluation purposes and must not be used for any commercial activities." Operator: Lyker Labs LLC. ([terms](https://metalpriceapi.com/terms))
- **Verdict:** not suitable.

### 3.11 goldapi.io

- **Auth:** `x-access-token` header. Keyless → `403 {"error":"No API Key provided"}` (I called it).
- **Limits:** from the site's own JS bundle (`main.*.chunk.js`, the source of the pricing page): `REACT_APP_FREE_PLAN_REQUESTS_PER_MONTH = "150"`. Free plan "Updates: 2 sec interval", WebSockets "1 connection, 10 channels". Pro $99/mo, Business $169/mo. ([pricing](https://www.goldapi.io/pricing), a JS-rendered page)
- **Shape (sample in the bundle):** `GET https://www.goldapi.io/api/price/XAU/USD` returns `{"metal":"XAU","currency":"USD","exchange":"FOREXCOM","price":4665.825,"unit":"troy_ounce","ask":…,"bid":…,"price_per_unit":{"troy_ounce":…,"gram":150.0098,"kilogram":…},"melt_price_per_gram":{"24k":…,"22k":…,"18k":…},...}`. The [GitHub README](https://github.com/goldapi-io) shows the older path `/api/XAU/USD`. It is one metal per call. Karat-per-gram output (24k/22k/18k…) would suit Turkish users.
- **ToS:** **unverified** (the page is a SPA; WebFetch got only "GoldAPI.io").
- **Verdict:** 150 req/month shared by every install: not suitable.

### 3.12 API Ninjas (commodityprice / metalsprice)

- **Auth:** `X-Api-Key`. Keyless → `400 {"error": "Missing API Key."}` (I called it).
- **Endpoints:** `/v1/commodityprice`, `/v1/commodityspot` ("spot price for precious metals"), `/v1/commoditysnapshot` ("all supported commodities in a single call"). ([API page](https://api-ninjas.com/api/commodityprice))
- **Free plan:** "Evaluation and demo purposes only" ([pricing](https://api-ninjas.com/pricing)). ToS §3.1: on the free plan you "may NOT … Use the Services or Output in any commercial product, service, or application" ([tos](https://api-ninjas.com/tos)). According to a search-result summary of its metalsprice page, free accounts get only the last daily close (**unverified**; [metalsprice](https://api-ninjas.com/api/metalsprice)).
- **Verdict:** not suitable.

### 3.13 commoditypriceapi.com / commodities-api.com

- commoditypriceapi.com: Basic $34.99/mo for 8,000 calls, "Upto 1 second updates", "Upto 5 symbols per request". The commercial licence starts at Standard ($174.99/mo). I saw no free plan. ([pricing](https://commoditypriceapi.com/pricing))
- commodities-api.com: `/pricing` returns 404. The homepage advertises "800+ commodities and 170 currencies" with a "Get API key" button. Free tier and ToS are **unverified**. ([home](https://commodities-api.com/))
- **Verdict:** paid and keyed; not suitable.

### 3.14 Frankfurter / ECB

- Frankfurter's currency list has 30 fiat codes (incl. TRY) and **no XAU/XAG**. `GET https://api.frankfurter.dev/v1/latest?base=XAU` → `404 {"message":"not found"}`. Same for `symbols=XAU`. (I called both.) Note: `api.frankfurter.app` now 301-redirects; the app already uses `api.frankfurter.dev`. ([currencies](https://api.frankfurter.dev/v1/currencies))
- ECB euro reference rates don't include gold. This is consistent with Frankfurter, which mirrors them. I checked it through Frankfurter's list, not the ECB page directly.

### 3.15 open.er-api.com (ExchangeRate-API open access)

- Keyless and daily (`time_next_update_utc` ~24 h). The only X-codes in its rates are XAF, XCD, XCG, XDR, XOF and XPF, so there is **no XAU/XAG** (I called `https://open.er-api.com/v6/latest/USD`). ([terms](https://www.exchangerate-api.com/terms))

### 3.16 exchangerate.host

- It is now an APILayer product. `GET https://api.exchangerate.host/live?source=XAU` → `401 {"success":false,"error":{"code":101,"type":"missing_access_key",...}}`, and `/latest` redirects to APILayer docs. **No longer keyless.** (I called it; [docs](https://docs.apilayer.com/exchangerate/docs/api-documentation))

### 3.17 Yahoo Finance chart endpoint (GC=F, SI=F)

- Keyless, but needs a browser-like User-Agent. It quotes **COMEX futures** (front contract "Gold Dec 26"), not spot. Contango puts it a little above spot: 3965.4 vs 3963.0.
  ```
  GET https://query1.finance.yahoo.com/v8/finance/chart/GC=F?range=1d&interval=1d
  -> {"chart":{"result":[{"meta":{"currency":"USD","symbol":"GC=F","exchangeName":"CMX","instrumentType":"FUTURE","regularMarketPrice":3965.4,"shortName":"Gold Dec 26",...
  ```
- There is no official public API or documentation. Use rests on Yahoo's general terms ([legal.yahoo.com](https://legal.yahoo.com/us/en/yahoo/terms/otos/index.html)); the specific clause is **unverified**. The endpoint has broken or added auth requirements before (**unverified** history). **Do not use** in a shipped client.

### 3.18 stooq.com CSV

- `https://stooq.com/q/l/?s=xauusd&f=sd2t2ohlc&h&e=csv` → **404** ("The page you requested does not exist"), and the same on stooq.pl. `https://stooq.com/q/d/l/?s=xauusd&i=d` returns a JavaScript proof-of-work bot challenge (I called it on 2026-11-12). **Treat as dead or blocked** for programmatic use.

### 3.19 Swissquote public quotes feed

- Undocumented, keyless, live bid/ask per spread profile. It works for XAU/USD, XAG/USD and XAU/EUR:
  ```
  GET https://forex-data-feed.swissquote.com/public-quotes/bboquotes/instrument/XAU/USD
  -> [{"topo":{"platform":"SwissquoteCapitalMarkets","server":"Live7"},"spreadProfilePrices":[{"spreadProfile":"premium","bidSpread":25.40,"askSpread":25.40,"bid":3961.026,"ask":3961.684},...],"ts":1794469946152}, ...]
  GET .../instrument/XAG/USD -> [... "bid":51.3325,"ask":51.3805 ...]
  GET .../instrument/XAU/EUR -> [... "bid":3577.199,"ask":3578.001 ...]
  ```
- **Gotcha:** `XAU/TRY` returned `bid 23917.650` with `ts` 1794434400052, about 10 hours old. The real XAU/TRY was ~199,000, so that data is badly wrong and stale. No docs or ToS for this feed exist; it's an internal widget backend. **Do not use.**

### 3.20 LBMA prices (official fix)

- The JSON that LBMA's own site has used (`https://prices.lbma.org.uk/json/gold_pm.json`, `silver.json`) returned **403** behind Cloudflare (I called it). LBMA prices are a licensed benchmark (licensing terms **unverified**). Two possible routes: metals.dev includes `lbma_gold_am/pm` and `lbma_silver` in its keyed response ([docs](https://metals.dev/docs)), and goldapi.io advertises "Historical LBMA AM Fix prices" (from its bundle). **No free keyless feed found.**

### 3.21 TCMB (Central Bank of the Republic of Türkiye)

- `https://www.tcmb.gov.tr/kurlar/today.xml` is keyless and official. The 11.11.2026 bulletin (`Bulten_No="2026/212"`) has 22 currencies (USD … XDR) and **no gold** (grep for XAU/ALTIN/GOLD = 0). (I called it.)
- EVDS (the data warehouse) requires an API key in the request, per the [EVDS Web Service Usage Guide](https://evds2.tcmb.gov.tr/help/videos/EVDS_Web_Service_Usage_Guide.pdf). `evds2…/service/evds/…` 302-redirected without a key. Whether EVDS carries a daily gold series (e.g. `TP.MK.KUL.YTL`) is **unverified**. Even if it does, it's daily, keyed and per-user registration, so it is impractical for a shipped client.

### 3.22 Turkish market sources (gram altın in TRY)

- **truncgil finans:** `https://finans.truncgil.com/v4/today.json` is keyless and works. It gives TRY per gram gold (`GRA`), silver (`GUMUS`), quarter/half/full coins and 14/18/22-karat prices in one call, updated every few minutes. I found no ToS or licence, and the upstream data source is unknown (**unverified**, probably aggregated from Kapalıçarşı/market quotes). The `/today.json` (v3-style) variant uses Turkish keys and comma decimals as strings (`"Alış":"50,2515"`).
  ```
  GET https://finans.truncgil.com/v4/today.json
  -> {"Update_Date":"2026-11-12 10:52:01","USD":{"Buying":50.2531,"Type":"Currency","Selling":50.2619,"Change":0.05}, ...,
      "GRA":{"Selling":6402.44,"Type":"Gold","Name":"GRAMALTIN","Change":-0.53,"Buying":6401.63},
      "GUMUS":{"Selling":82.97,"Type":"Gold","Name":"GUMUS","Change":-0.72,"Buying":82.87},
      "CEYREKALTIN":{"Buying":10432,"Type":"Gold","Name":"CEYREKALTIN","Selling":10506,...}, "ONS":{"Buying":0,...,"Selling":0}, ...}
  ```
  Cross-check: gold-api 3963.0 / 31.1035 × 50.25 = 6402.5 TRY/g, which matches `GRA` almost exactly. Silver 51.40 / 31.1035 × 50.25 = 83.0, which matches `GUMUS`. So GRA/GUMUS track international spot × USDTRY, not the Kapalıçarşı physical premium. Note that the `ONS` (ounce) field was 0.
- **genelpara:** `api.genelpara.com/embed/altin.json` → 404; `/json/?list=altin` → Cloudflare "Attention Required" block.
- **Harem Altın:** `POST haremaltin.com/dashboard/ajax/doviz` → 404 (the internal endpoint has moved).
- **Kapalıçarşı** (`kapalicarsi.apiluna.org`): no response (HTTP 000).
- **Verdict:** truncgil is the only one alive. It is handy as an optional direct TRY/gram source, but it is unofficial with no ToS or SLA. Use it at most as a last-resort TRY cross-check, not as the primary.

---

## 4. Fallback design notes (no code changes made)

Relevant shape today: `Source::Metal(m)` is fetched per metal. `fetch()` returns `Rates::with_metal(m, usd_per_ounce)`, both metals share `metal.json` keyed by `m.code()`, and `parse_metal` insists on `currency == "USD"`.

- **Keep the USD/oz contract.** Every candidate above can be reduced to "USD per troy ounce for XAU/XAG". Fallbacks can then return the same `f64` into `with_metal`, and the ECB cross-conversion stays the same. Avoid sources whose only output is TRY/gram (truncgil): the app would then have to divide by its own USDTRY, which mixes ECB's daily TRY with a market TRY.
- **Suggested order inside `Source::Metal(m)`:**
  1. gold-api.com `/price/{XAU|XAG}`: live, ToS-clean.
  2. CoinGecko-derived: `simple/price?ids=bitcoin&vs_currencies=usd,xau,xag` → `usd/xau`, `usd/xag`. One host the app already depends on, and it covers both metals in one response, so XAU and XAG could share one fetch.
  3. currency-api `usd.json`, pages.dev first, then jsDelivr → `1/xau`, `1/xag`. Daily, but very stable static hosting.
  Optionally, gold could fall back to Kraken/Coinbase PAXG-USD. There is no silver equivalent.
- **Sanity check across sources.** Reject a value that differs from the last cached value by more than ~15% for gold or ~25% for silver without a second source agreeing. This catches bad data like Swissquote's stale XAU/TRY or a broken derived ratio.
- **Record provenance and staleness.** The day cache can't tell a 30-s quote from a daily snapshot. Storing which source won (and its timestamp: gold-api `updatedAt`, CoinGecko `last_updated_at`, currency-api `date`) would let the UI mark "daily rate" when a fallback was used.
- **One fetch for both metals:** fallbacks 2 and 3 return XAU and XAG together. If `Source::Metal` ever merges into a single `Source::Metals`, one request per refresh would cover both, and a per-metal partial failure (XAU ok, XAG failed) would go away.
- **Rate etiquette:** gold-api treats several requests per second to the same endpoint as spam. The app's few-per-day pattern is far below that. CoinGecko asks you to refresh any cache at least every 24 h, which the per-day cache already does.
- **Timeouts:** the existing 10-s global timeout per call means a three-step chain could block a worker for up to 30 s. That is acceptable off the UI thread, but consider a shorter per-attempt timeout for the fallbacks.

---

## 5. Sources

Primary docs, terms and pages (fetched 2026-11-12):
- gold-api.com: https://gold-api.com/docs · https://gold-api.com/faqs · https://gold-api.com/pricing · https://gold-api.com/terms · https://gold-api.com/
- CoinGecko: https://docs.coingecko.com/reference/exchange-rates · https://docs.coingecko.com/docs/common-errors-rate-limit · https://www.coingecko.com/en/api_terms · https://api.coingecko.com/api/v3/simple/supported_vs_currencies
- fawazahmed0/exchange-api: https://github.com/fawazahmed0/exchange-api
- Kraken: https://docs.kraken.com/api/docs/rest-api/get-ticker-information
- Binance: https://developers.binance.com/docs/binance-spot-api-docs/faqs/market_data_only
- Kitco: https://www.kitco.com/terms-of-use · https://www.kitco.com/robots.txt · https://corp.kitco.com/contact-us.html · https://www.kitconet.com/
- metalsdaily: https://www.metalsdaily.com/ · https://www.metalsdaily.com/live-prices/gold/ · https://www.metalsdaily.com/robots.txt
- dailymetalprice: https://www.dailymetalprice.com/robots.txt
- metals.dev: https://metals.dev/pricing · https://metals.dev/docs · https://metals.dev/policy/terms
- metals-api.com: https://metals-api.com/pricing · https://metals-api.com/terms
- metalpriceapi.com: https://metalpriceapi.com/pricing · https://metalpriceapi.com/terms
- goldapi.io: https://www.goldapi.io/pricing (JS bundle `/static/js/main.403605d8.chunk.js`) · https://github.com/goldapi-io
- API Ninjas: https://api-ninjas.com/api/commodityprice · https://api-ninjas.com/pricing · https://api-ninjas.com/tos · https://api-ninjas.com/api/metalsprice
- commoditypriceapi.com: https://commoditypriceapi.com/pricing · commodities-api.com: https://commodities-api.com/
- Frankfurter: https://api.frankfurter.dev/v1/currencies
- ExchangeRate-API: https://www.exchangerate-api.com/terms · exchangerate.host: https://docs.apilayer.com/exchangerate/docs/api-documentation
- Yahoo terms: https://legal.yahoo.com/us/en/yahoo/terms/otos/index.html
- TCMB: https://www.tcmb.gov.tr/kurlar/today.xml · https://evds2.tcmb.gov.tr/help/videos/EVDS_Web_Service_Usage_Guide.pdf

Endpoints called directly on 2026-11-12 (responses quoted above):
- https://api.gold-api.com/price/XAU · /price/XAG · /price/XAU/EUR · /price/XAU/TRY · /price/XAU,XAG · /symbols
- https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd,try,eur,xau,xag · /api/v3/exchange_rates · /simple/price?ids=pax-gold,tether-gold&…
- https://latest.currency-api.pages.dev/v1/currencies/{xau,xag,usd}.json · https://cdn.jsdelivr.net/npm/@fawazahmed0/currency-api@latest/v1/currencies/xau.json · /v1/currencies.json
- https://api.coinbase.com/v2/prices/PAXG-USD/spot · PAXG-TRY · https://api.exchange.coinbase.com/products/PAXG-USD/ticker
- https://api.kraken.com/0/public/Ticker?pair=PAXGUSD,XAUTUSD · PAXGEUR
- https://api.binance.com/api/v3/ticker/price · https://data-api.binance.vision/api/v3/ticker/price
- https://proxy.kitco.com/getPM?symbol=AU,AG&currency=USD&unit=ounce (and EUR/GBP/JPY/CHF gram, TRY → 404)
- https://forex-data-feed.swissquote.com/public-quotes/bboquotes/instrument/{XAU/USD, XAG/USD, XAU/EUR, XAU/TRY}
- https://query1.finance.yahoo.com/v8/finance/chart/GC=F
- https://stooq.com/q/l/?s=xauusd… (404) · https://stooq.com/q/d/l/?s=xauusd&i=d (bot challenge)
- https://api.frankfurter.dev/v1/latest?base=XAU (404) · https://open.er-api.com/v6/latest/USD · https://api.exchangerate.host/live?source=XAU (401)
- https://prices.lbma.org.uk/json/gold_pm.json (403)
- https://www.tcmb.gov.tr/kurlar/today.xml
- https://finans.truncgil.com/v4/today.json · https://finans.truncgil.com/today.json · api.genelpara.com (404/blocked) · haremaltin.com ajax (404) · kapalicarsi.apiluna.org (no response)
- Keyed APIs called without a key, to confirm they require auth: api.metals.dev, api.metalpriceapi.com, metals-api.com, goldapi.io, api.api-ninjas.com
