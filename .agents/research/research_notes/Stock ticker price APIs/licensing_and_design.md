# Licensing rules, ticker symbology and fallback design for a distributed desktop stock-price feed (as of Nov 2026)

Scope: non-provider-specific rules and conventions for a Rust desktop personal-finance app with no backend, where each user's machine fetches prices for that user's own holdings and values them daily. Specific API providers and Borsa Istanbul sources are covered by other researchers. Research date: 2026-11-12.

Method note: the exchange policy PDFs (Nasdaq, NYSE, Deutsche Börse, Alpha Vantage ToS) were downloaded and converted to text with `pdftotext`, so the quotes below are verbatim from those documents. Some exchange documents are several years old; their dates are given inline.

## 1. Exchange market-data licensing: real-time vs 15-min delayed vs end-of-day

### Takeaway
At NYSE, Nasdaq, Deutsche Börse, Euronext and LSE, delayed data (15 minutes or more) and end-of-day data generally carry no per-user display or data fees for end users. That is not the same as "licence-free". The exchanges license **redistributors/vendors**: those still sign agreements, sometimes pay distribution licence fees even for delayed data (Deutsche Börse), and must show a "delayed" notice. Real-time data is fee-liable per user, and exchanges price non-professional (natural person, personal use) users much lower than professionals. Borsa Istanbul follows the same distributor model: delayed data needs no per-user fee, but distribution rights come through a distributor contract.

### Cited Findings

**Nasdaq (US)**: *US Equities and Options Data Policies*, Version 2.6, last revision entry dated June 1, 2024 (older than 2026; check for a newer version)
- Delayed data may be free, depending on the product: "if a data product is eligible for delayed pricing, and the data is delayed as per the Delayed Data Policy, there may not be a charge for the usage of the delayed data, depending upon the product selected." — [Nasdaq US Data Policies v2.6](https://www.nasdaqtrader.com/content/AdministrationSupport/Policy/USEquitiesandOptionsDataPolicies.pdf)
- A delay notice is required: "Nasdaq requires that an appropriate delay message be provided to Subscribers … The delay message must prominently appear on all displays containing Delayed Data, such as at or near the top of the page." The examples given include "Data Delayed 15 minutes" and "Data Delayed 24 hours". — [Nasdaq US Data Policies v2.6](https://www.nasdaqtrader.com/content/AdministrationSupport/Policy/USEquitiesandOptionsDataPolicies.pdf)
- Recognised delay intervals: 15 minutes, 24 hours, Midnight Local Time, and "Not Eligible for Delayed Pricing". End-of-Day is defined as "Information from the current day that is disseminated after the market has closed for the current day and after the Delay Interval". Nasdaq Basic's delay interval is 15 minutes; TotalView and Level 2 are "Midnight LT". — [Nasdaq US Data Policies v2.6](https://www.nasdaqtrader.com/content/AdministrationSupport/Policy/USEquitiesandOptionsDataPolicies.pdf)
- After the delay interval, distribution "may not be fee-liable and may not need to be reported to Nasdaq on a monthly basis". "Distributors that provide End-of-Day Information … on an Uncontrolled Product are not currently required to obtain a Nasdaq Global Data Agreement from such Data feed Recipients." The waiver "does not limit the Distributor's indemnification obligations". — [Nasdaq US Data Policies v2.6](https://www.nasdaqtrader.com/content/AdministrationSupport/Policy/USEquitiesandOptionsDataPolicies.pdf)
- A Non-Professional Subscriber is "Any natural person who is NOT" registered with the SEC, CFTC, state securities agencies or exchanges, not an "investment advisor", and not employed by an exempt bank in a role that would require registration. "All Subscribers are deemed to be Professional unless they are qualified as Non-Professional Subscribers." "For the Non-Professional Subscriber, Information is licensed only for personal use." — [Nasdaq US Data Policies v2.6](https://www.nasdaqtrader.com/content/AdministrationSupport/Policy/USEquitiesandOptionsDataPolicies.pdf)
- Nasdaq's European data policies (Sept 2026 version), per a search-result snippet only: "Display fees are applicable for the right to use real time information for display purposes. Display fees are not applicable for Delayed use cases." The default delay is 15 minutes. Nordic/Baltic delayed data from the website is generally free, but external commercial or value-added redistribution requires a Global Data Agreement. — [Nasdaq European Data Policies Sept 2026](https://www.nasdaq.com/docs/data/nasdaq-european-data-policies-september-2026) (snippet only; full PDF not read)

**NYSE**: *NYSE Proprietary Market Data Comprehensive Policy Package*, version 2.3, Mar 21, 2022 (older; Delayed Data Policy added Aug 2021)
- "NYSE permits Redistributors to delay real-time NYSE Market Information from datafeeds ("Delayed Information") and disseminate it to data recipients. Data recipients that use Delayed Information may do so solely in accordance with the terms and conditions set forth in their market data license agreement." — [NYSE Comprehensive Policy Package](https://www.nyse.com/publicdocs/nyse/data/NYSE_Proprietary_Market_Data_Comprehensive_Policy_Package.pdf)
- Delay intervals are 15 minutes and Midnight EST. Even delayed datafeeds need "Prior approval and license agreement". — [NYSE Comprehensive Policy Package](https://www.nyse.com/publicdocs/nyse/data/NYSE_Proprietary_Market_Data_Comprehensive_Policy_Package.pdf)
- "Redistributors who are licensed to externally redistribute Delayed Information to Professional and Non-Professional data recipients for display device use can do so without seeking prior approval … do not need to report Professional and Non-Professional data recipients on a monthly basis". They must, however, keep entitlement audit trails and show a disclaimer such as "Data Delayed 15 minutes". — [NYSE Comprehensive Policy Package](https://www.nyse.com/publicdocs/nyse/data/NYSE_Proprietary_Market_Data_Comprehensive_Policy_Package.pdf)
- For consolidated (CTA/SIP) data, the CTA Plan's delayed policy (Feb 2016) sets the delay at 15 minutes after the markets first make the information available. — [CTA Policy: Delayed Market Data](https://ctaplan.com/publicdocs/ctaplan/Policy_Delayed_Market_Data.pdf) (search snippet; not read in full)

**Deutsche Börse (Xetra)**: *Price List to the Market Data Dissemination Agreement*, "Valid as of 1 January 2026" (footer Version 15_3)
- "C.1.1 General Provisions Data Fees Professional Customers — No Data Fees are charged for Delayed Data." — [DB MDDA Price List 2026](https://www.mds.deutsche-boerse.com/resource/blob/4694154/8d7eccd97e978f07f993cc8b7a0ae4a1/data/MDDA_Price_List_15_2.pdf)
- Vendors still pay distribution licence fees for delayed data. Section B.2 has separate "Real-time" and "Delayed" columns: Xetra Core is 4,553.00 real-time vs 2,341.00 delayed, and Xetra Ultra is 4,955.00 in both columns (EUR/month per the "/Month" header). "Distribution License Fees apply for the right of Onward Dissemination of Licensed Information." — [DB MDDA Price List 2026](https://www.mds.deutsche-boerse.com/resource/blob/4694154/8d7eccd97e978f07f993cc8b7a0ae4a1/data/MDDA_Price_List_15_2.pdf)
- Real-time Non-Professional data fees per Access ID per month: Xetra Ultra Level 2 24.70, Ultra Level 1 20.90, Core Level 1 5.90, ETFs & ETPs Level 2 19.90. Professional fees are higher, e.g. Xetra Ultra Level 2 at 116.05 per Access ID. Column alignment in the extracted text is imperfect, so treat the exact row/value pairing as approximate. — [DB MDDA Price List 2026](https://www.mds.deutsche-boerse.com/resource/blob/4694154/8d7eccd97e978f07f993cc8b7a0ae4a1/data/MDDA_Price_List_15_2.pdf)
- Webhosting of a limited set of up to 25 instruments costs 64.00/month per hosted domain. Onward dissemination of delayed data via webhosting falls under Sections 24 et seq. of the GTC. — [DB MDDA Price List 2026](https://www.mds.deutsche-boerse.com/resource/blob/4694154/8d7eccd97e978f07f993cc8b7a0ae4a1/data/MDDA_Price_List_15_2.pdf)
- Under MiFID II, DB provides free-of-charge delayed pre- and post-trade data for its regulated exchanges, with a 15-minute delay. — [DB MD+S Delayed data](https://www.mds.deutsche-boerse.com/mds-en/real-time-data/Delayed-data) (search snippet)

**Euronext**
- Euronext is "required to make delayed data available free of charge", with "a maximum time delay of 15 minutes after initial publication", through a view service (live.euronext.com) and a file download service. Files stay available for at least 24 hours, with timestamps in UTC. The page does not address third-party display or redistribution; it points to the fee schedule and market data agreements. — [Euronext MiFID II compliant data](https://www.euronext.com/en/data/pricing-specs-agreements/mifid-ii-compliant-data)

**London Stock Exchange (LSEG)**
- LSEG makes pre- and post-trade data available 15 minutes after initial publication, as required by MiFIR/MiFID II. Users must accept licence terms, and redistribution or commercial use requires a licence. — [LSEG Delayed Market Data T&Cs](https://dmd.lseg.com/delayed-market-data/terms-and-conditions.htm) (search snippet; page not read in full)
- An older LSE-group price list (Borsa Italiana, 2016) says: "If Level 1 or 2 data is delayed by 15 minutes or more prior to dissemination and display, Data Charges are not payable, subject to completion of a Delayed Licence Agreement." — [Borsa Italiana price list 2016](https://www.borsaitalia.it/borsaitaliana/information-services/pricing/rtdpricelist2016.pdf) (search snippet; dated 2016)

**Borsa Istanbul**
- BIST market data reaches investors through licensed distributors in real-time, delayed and end-of-day form ("eş-anlı, gecikmeli ve günsonu"). — [Borsa İstanbul Veri Yayını](https://borsaistanbul.com/veriler/veri-yayini) (search snippet)
- "Alt Dağıtıcı kullanıcı başına ücrete tabi olmayan veriler (sınırlı düzey 1, gecikmeli veri, referans veri vb.)" (sub-distributors may obtain rights for data **not subject to per-user fees**, such as limited level 1, delayed data and reference data). Distributors sign the "Veri Dağıtım Sözleşmesi" directly with BIST; sub-distributors go through an authorised distributor. — [BIST FAQ: Veri dağıtım ve endeks lisanslama](https://borsaistanbul.com/sss/veri-dagitim-ve-endeks-lisanslama)
- The fee schedule is in Annex 3 of the distribution agreement. — [BIST FAQ](https://borsaistanbul.com/sss/veri-dagitim-ve-endeks-lisanslama) (via search summary)
- Retail brokers sell BIST real-time licence packages. İşbank says a single licence cannot show real-time data on two channels at once, and the earlier session drops to a 15-minute delay. — [İşbank BIST veri lisansı paketleri](https://www.isbank.com.tr/borsa-istanbul-veri-lisansi-paketleri)
- Yahoo Finance lists Borsa Istanbul (.IS) at a 15-minute delay, LSE (.L) at 20 minutes, XETRA (.DE), Euronext Paris (.PA) and Frankfurt (.F) at 15 minutes, and Nasdaq as real-time. ICE Data Services is the provider. — [Yahoo Help: Exchanges and data providers](https://help.yahoo.com/kb/SLN2310.html)

### Inferences
- For an app that shows daily valuations (previous close or end-of-day), the exchange-fee question mostly goes away. Every exchange checked treats delayed and end-of-day data as free of per-user display or data fees. What remains is the **vendor's** licence, and its ToS (section 2) decides whether the app may use it.
- Showing intraday "live" prices is where risk starts. A vendor that delivers real-time data to a non-professional end user is normally liable for per-user exchange fees. Labelling prices "Delayed" or "As of close [date]" fits the delay-notice rules at Nasdaq and NYSE.
- The non-professional category matters only for real-time data. The user of a personal app is a natural person using data for personal purposes, which is exactly Nasdaq's Non-Professional definition. The developer, however, is not a licensee under exchange policies unless they redistribute data themselves.

### Gaps
- I did not read NYSE's or Nasdaq's 2026 fee schedules; the Nasdaq US policy v2.6 (2024) and NYSE package v2.3 (2022) are the latest I retrieved.
- I did not retrieve the current text of the LSEG and Euronext market data policies or fee schedules. The LSE delayed-licence rule comes from a 2016 group price list.
- I did not retrieve the fee figures in BIST's Annex 3 or a direct BIST statement on whether apps displaying EOD data need a licence. The borsaistanbul.com/en/data page failed with ECONNRESET.

## 2. Who bears the licence when a client app fetches from a vendor API; vendor ToS on personal use vs third-party display

### Takeaway
Exchanges license the **vendor (distributor/redistributor)**. The vendor passes the obligations down through its ToS to its **API customer**, the holder of the API key. If the developer embeds one shared key in a distributed app, the developer is the customer, and serving data to all users is "redistribution/external display", which free and personal tiers forbid. If each user enters their own key (BYOK), each user is the customer and uses the data personally, which is what personal and free tiers allow.

### Cited Findings
- Twelve Data Terms (last updated January 1, 2026): the licence is granted "solely for Internal Use", defined as "solely for Customer's internal business purposes and not for redistribution or external commercial purposes" (§1, §2.1). A customer may "Redistribute or provide external display of Data only if and as expressly authorized by a Redistribution Rights Add-On" (§2.2(e)). Forbidden: "Use Free Tier data for commercial purposes" (§2.3(l)). The customer "is solely responsible for ensuring" compliance with third-party/exchange licensing and fees (§3.2(f)), must represent they are "qualified to receive exchange information" (§3.3(a)), and indemnifies Twelve Data (§9.1(e)). — [Twelve Data Terms](https://twelvedata.com/terms)
- Alpha Vantage ToS: the licence covers "personal, non-commercial use". Use counts as commercial if it "goes beyond investment analysis, research, testing, monitoring, and any other activities that are private and individual in nature", is "on behalf of a corporation", or provides information "as part of any type of commercial activity that allows individuals or entities other than User to access information directly or indirectly". The licence is "non-sublicensable, non-transferable" and takes effect when "User clicks 'Get Free API Key'". — [Alpha Vantage Terms of Service](https://www.alphavantage.co/terms_of_service/) (undated)
- Yahoo Developer API Terms forbid using the APIs to "derive income from the use or provision of the Yahoo APIs … unless the API Documents specifically permit otherwise". Yahoo "may change, suspend, or discontinue any aspect of the Yahoo APIs at any time". This page is the generic Yahoo developer ToS and does not cover Yahoo Finance specifically. — [Yahoo Developer API Terms](https://legal.yahoo.com/us/en/yahoo/terms/product-atos/apiforydn/index.html)
- Yahoo Finance help: "All data provided on Yahoo Finance is provided for informational purposes only, and is not intended for trading or investing purposes." — [Yahoo Help SLN2310](https://help.yahoo.com/kb/SLN2310.html)
- On the exchange side, Nasdaq places obligations on the "Distributor", which must report subscribers, classify them as professional or non-professional, show delay notices and indemnify Nasdaq. — [Nasdaq US Data Policies v2.6](https://www.nasdaqtrader.com/content/AdministrationSupport/Policy/USEquitiesandOptionsDataPolicies.pdf)

### Inferences
- A no-backend app that ships **its own** API key to every installation is redistributing data to third parties under its developer's account. That breaches Twelve Data's free/Internal Use terms and Alpha Vantage's personal-use grant, and it would be explicitly "commercial" if the app is sold.
- BYOK keeps each user as the licensee using data for "private and individual" purposes, which is the cleanest structure for both free and commercial distribution of the app. The app itself then distributes no data.
- Unofficial endpoints such as Yahoo's chart and quote JSON and Stooq CSV have no API ToS for this use. They are legally grey and can break at any time, so they belong behind a user opt-in or as a fallback, not as a guaranteed feature of a commercial product.

### Gaps
- I did not read Finnhub, FMP or EODHD ToS clauses; the provider researchers should confirm their "personal vs display" wording.
- I found no court or regulator statement about BYOK desktop apps specifically. The inference above rests on how the ToS clauses are written.

## 3. BYOK pattern in open-source finance apps: default sources, fallbacks, breakage handling

### Takeaway
Mature open-source portfolio tools default to keyless Yahoo Finance and add keyed providers (Alpha Vantage, Finnhub, Twelve Data, EODHD, FMP) as **user-supplied-key** options. Wealthfolio, a Rust/Tauri app and the closest analogue, has the most complete design: a canonical InstrumentId keyed by MIC, per-provider symbol resolution, priority ordering the user can override, retry classes, a per-provider circuit breaker, rate limiting and quote validation. Finance::Quote has failover across modules that serve the same market, switched on by default.

### Cited Findings
**Wealthfolio** (Rust; `crates/market-data`; AGPL-3.0; repo now `wealthfolio/wealthfolio`, last pushed 2026-11-11)
- Providers in the crate: Yahoo, Alpha Vantage, Finnhub, MarketData.app, Börse Frankfurt, MOEX, AMFI, MetalPriceAPI, OpenFIGI, US Treasury calc, plus user-defined "custom" scraper providers. — [Wealthfolio market-data lib.rs](https://github.com/wealthfolio/wealthfolio/blob/main/crates/market-data/src/lib.rs)
- Architecture per lib.rs: Domain → `InstrumentId` ("canonical identity") → Resolver ("chain of responsibility") → `ProviderInstrument` ("provider-specific") → Provider → Quote. Type aliases: `Mic` ("Market Identifier Code (ISO 10383)"), `Currency` (ISO 4217) and `ProviderSymbol`. — [Wealthfolio lib.rs](https://github.com/wealthfolio/wealthfolio/blob/main/crates/market-data/src/lib.rs)
- The registry fetch order is: filter by asset-kind capability, sort by preferred provider then priority, check the circuit breaker, resolve the symbol, apply rate limiting, fetch, validate, and on failure "try next provider based on retry class". User priorities override defaults ("Lower values = higher priority"). "A security assigned to a custom provider is served by it alone". — [Wealthfolio provider_registry.rs](https://github.com/wealthfolio/wealthfolio/blob/main/crates/market-data/src/registry/provider_registry.rs)
- Retry classes: `Never` (bad symbol or validation error, no failover); `FailoverWithPenalty` ("transient errors like rate limiting (429) or timeout", which counts toward the circuit breaker); `NextProvider` ("no data for range, resolution failed", no penalty); `CircuitOpen` (skip). — [Wealthfolio errors/retry.rs](https://github.com/wealthfolio/wealthfolio/blob/main/crates/market-data/src/errors/retry.rs)
- The circuit breaker opens after 5 consecutive failures (`DEFAULT_FAILURE_THRESHOLD`), waits 60 s before half-open, and needs 2 successes to close. It is "in-memory and resets on application restart". — [Wealthfolio circuit_breaker.rs](https://github.com/wealthfolio/wealthfolio/blob/main/crates/market-data/src/registry/circuit_breaker.rs)
- The validator checks OHLC invariants (high ≥ low, open and close within range), non-negative values and reasonable ranges. A hard failure means "reject quote, try next provider"; staleness checks are listed as "Future". — [Wealthfolio validator.rs](https://github.com/wealthfolio/wealthfolio/blob/main/crates/market-data/src/registry/validator.rs)
- Provider API keys go into a shared secret store. — [Wealthfolio README](https://github.com/wealthfolio/wealthfolio/blob/main/README.md)

**Ghostfolio** (TypeScript/NestJS, self-hosted server)
- Data-provider services: alpha-vantage, coingecko, eod-historical-data, financial-modeling-prep, ghostfolio (premium), google-sheets, manual, rapid-api, yahoo-finance, plus a "data-enhancer". — [Ghostfolio data-provider dir](https://github.com/ghostfolio/ghostfolio/tree/main/apps/api/src/services/data-provider)
- Defaults: `DATA_SOURCES` = `[COINGECKO, MANUAL, YAHOO]`; `DATA_SOURCE_EXCHANGE_RATES` and `DATA_SOURCE_IMPORT` both default to YAHOO. Keys are optional env vars: `API_KEY_ALPHA_VANTAGE`, `API_KEY_EOD_HISTORICAL_DATA`, `API_KEY_FINANCIAL_MODELING_PREP`, `API_KEY_OPEN_FIGI`, `API_KEY_RAPID_API`, `API_KEY_COINGECKO_DEMO/PRO`. `CACHE_QUOTES_TTL` defaults to 1 minute. — [Ghostfolio configuration.service.ts](https://github.com/ghostfolio/ghostfolio/blob/main/apps/api/src/services/configuration/configuration.service.ts)
- The README documents `REQUEST_TIMEOUT` for data-provider requests, default 3000 ms. Each asset carries a `dataSource` (COINGECKO | GHOSTFOLIO | MANUAL | YAHOO). — [Ghostfolio README](https://github.com/ghostfolio/ghostfolio)

**Portfolio Performance** (Java desktop)
- Quote feeds in `online/impl`: Yahoo Finance, Yahoo Finance Adjusted Close (`YAHOO-ADJUSTEDCLOSE`), Alpha Vantage, Finnhub, Twelve Data, EODHD, Leeway, Quandl, CoinGecko, Binance, Bitfinex, Kraken, MEXC, ECB Data Portal, Eurostat HICP, AMFI India, MFAPI, DivvyDiary, the "Portfolio Performance"/Portfolio Report feeds, generic JSON and HTML-table scrapers, and Manual. — [PP online/impl dir](https://github.com/portfolio-performance/portfolio/tree/master/name.abuchen.portfolio/src/name/abuchen/portfolio/online/impl)
- Keyed feeds refuse to run without a key: Twelve Data logs `MsgErrorTwelveDataAPIKeyMissing`, and EODHD and Finnhub return `MsgErrorMissingAPIKey`. — [PP TwelveDataQuoteFeed.java](https://github.com/portfolio-performance/portfolio/blob/master/name.abuchen.portfolio/src/name/abuchen/portfolio/online/impl/TwelveDataQuoteFeed.java); [PP EODHistoricalDataQuoteFeed.java](https://github.com/portfolio-performance/portfolio/blob/master/name.abuchen.portfolio/src/name/abuchen/portfolio/online/impl/EODHistoricalDataQuoteFeed.java); [PP FinnhubQuoteFeed.java](https://github.com/portfolio-performance/portfolio/blob/master/name.abuchen.portfolio/src/name/abuchen/portfolio/online/impl/FinnhubQuoteFeed.java)
- PP also ships `MarketIdentifierCodes.java` and a `market-identifier-codes.properties` file. — [PP online/impl dir](https://github.com/portfolio-performance/portfolio/tree/master/name.abuchen.portfolio/src/name/abuchen/portfolio/online/impl)

**Finance::Quote** (Perl; GnuCash's quote engine. The GnuCash link was not re-verified in this session.)
- Modules include AlphaVantage, Finnhub, TwelveData, YahooJSON, YahooWeb, Stooq, XETRA, Tradegate, OnVista, Comdirect, Consorsbank, BorsaItaliana, Bourso, SIX, TMX, ASX, NZX, MarketWatch, GoogleWeb, FinanceAPI, StockData, and others. — [Finance::Quote modules dir](https://github.com/finance-quote/finance-quote/tree/master/lib/Finance/Quote)
- Failover: "Failover is when the `fetch` method attempts to retrieve quote information for a security from alternate sources when the requested method fails". Methods like 'nasdaq' or 'europe' map to several modules, while 'alphavantage' maps to one. The constructor sets `FAILOVER = 1`. In the loop, each module is tried in turn, and only the still-failed symbols are passed to the next one ("last unless $this->{FAILOVER}; last unless @failed_stocks;"). — [Finance::Quote Quote.pm](https://github.com/finance-quote/finance-quote/blob/master/lib/Finance/Quote.pm)
- API keys are passed per module: `Finance::Quote->new('alphavantage' => {API_KEY => '...'})`. Modules declare `$FEATURES = {'API_KEY' => 'registered user API key'}`, and Finnhub also reads the `FINNHUB_API_KEY` environment variable. — [Quote.pm](https://github.com/finance-quote/finance-quote/blob/master/lib/Finance/Quote.pm); [Finnhub.pm](https://github.com/finance-quote/finance-quote/blob/master/lib/Finance/Quote/Finnhub.pm)
- To survive Yahoo breakage, YahooJSON fetches `https://www.yahoo.com/`, then `query2.finance.yahoo.com/v1/test/getcrumb`, then calls `v11/finance/quoteSummary`, which requires the cookie+crumb flow. — [Finance::Quote YahooJSON.pm](https://github.com/finance-quote/finance-quote/blob/master/lib/Finance/Quote/YahooJSON.pm)

**Rotki**
- Its `externalapis` modules are crypto/on-chain sources (coingecko, cryptocompare, defillama, etherscan, etc.) plus FX sources (frankfurter, xratescom). I saw no stock-quote providers. — [rotki externalapis dir](https://github.com/rotki/rotki/tree/develop/rotkehlchen/externalapis)

**Keyless-source breakage examples**
- Stooq: "As March 2026, Stooq requires an API key for data downloads". Requests now return an HTML page asking for a key instead of CSV, which made `pandas-datareader` fail with `ParserError` (issue opened Apr 13, 2026). — [pandas-datareader #1012](https://github.com/pydata/pandas-datareader/issues/1012)

### Inferences
- The common pattern is a keyless default (Yahoo) plus optional BYOK providers plus manual entry as the last resort. Wealthfolio's registry (priority, retry class, circuit breaker, validator) translates directly to Rust and is worth reading in full before designing the app's trait.
- Detect breakage by **content type and shape**, not only HTTP status. Stooq returned HTML with status 200 instead of CSV, and Yahoo needs a cookie+crumb flow.
- "Manual" is a provider in Ghostfolio, Wealthfolio (custom) and Portfolio Performance alike. A manual or user-entered price should be a first-class source in the fallback chain.

### Gaps
- I did not research tickrs, Kubera-like tools or "Stockfinlands". I could not find the last one, so its existence is unverified.
- I did not read Ghostfolio's runtime fallback logic, i.e. whether it falls back across providers per symbol.
- I did not verify Wealthfolio's default provider priorities or whether Yahoo is enabled by default.

## 4. Ticker symbology across providers; canonical key; free symbol mapping

### Takeaway
Yahoo-style "TICKER.SUFFIX" is the de facto lingua franca, but suffixes differ per provider. The same London listing is VOD.L on Yahoo, VOD.LON on Alpha Vantage, VOD.LSE on EODHD and vod.uk on Stooq, while Twelve Data takes the bare symbol plus a `mic_code`. Use **ticker + ISO 10383 MIC** (the listing) as the canonical key, keep ISIN as an attribute and lookup aid, and map to each provider with a per-provider suffix table. An ISIN identifies the issue, not the listing or currency: one ISIN trades on many venues in different currencies. OpenFIGI offers free, keyless ISIN→ticker/exchange mapping, and its identifiers are public domain.

### Cited Findings
- Wealthfolio's exchange registry maps one MIC to a per-provider suffix and currency (76 exchanges):
  - XLON: Yahoo `.L` (Yahoo exchange codes LSE, IOB), Alpha Vantage `.LON`; registry currency `GBp`, with Alpha Vantage's currency recorded as `GBP`.
  - XETR: Yahoo `.DE` (codes GER, XETRA), Alpha Vantage `.DEX`.
  - XPAR: Yahoo `.PA` (codes PAR, ENX), Alpha Vantage `.PAR`.
  - XIST: Yahoo `.IS` (code IST), TRY, timezone Europe/Istanbul. Only Yahoo is mapped.
  - XNYS: bare symbols for Yahoo, Alpha Vantage, Finnhub and MarketData.app.

  — [Wealthfolio resolver/exchanges.json](https://github.com/wealthfolio/wealthfolio/blob/main/crates/market-data/src/resolver/exchanges.json)
- Wealthfolio tests: `yahoo_suffix_to_mic("L") == "XLON"`, `"DE" → "XETR"`, `"PA" → "XPAR"`, and a code comment "London - Note: Yahoo returns GBp (pence)". — [Wealthfolio exchange_suffixes.rs](https://github.com/wealthfolio/wealthfolio/blob/main/crates/market-data/src/resolver/exchange_suffixes.rs)
- Yahoo suffixes: .L (LSE; .IL for IOB), .DE (XETRA), .F (Frankfurt), .PA (Euronext Paris), .IS (Borsa Istanbul); US listings have no suffix. — [Yahoo Help SLN2310](https://help.yahoo.com/kb/SLN2310.html)
- EODHD uses `SYMBOL.EXCHANGE` with **its own codes, not MICs**: "our exchange code rather than a MIC". Examples: AAPL.US (".US suffix may be omitted"), BP.LSE, SAP.XETRA, and PA for Paris. An unknown ticker returns HTTP 404. — [EODHD historical data API](https://eodhd.com/financial-apis/api-for-historical-data-and-volumes)
- Twelve Data takes symbol plus `mic_code`. Portfolio Performance splits "TICKER.XXXX" at the dot and calls `/price?symbol=TICKER&mic_code=XXXX`, and its code comments show `"mic_code": "XNAS"`. — [PP TwelveDataQuoteFeed.java](https://github.com/portfolio-performance/portfolio/blob/master/name.abuchen.portfolio/src/name/abuchen/portfolio/online/impl/TwelveDataQuoteFeed.java)
- Stooq uses lowercase symbols with country-style suffixes (.us, .uk, .de, .jp, .hk, .hu), e.g. aapl.us, from `https://stooq.com/q/d/l/?s=SYMBOL&i=d`. — [Stooq Q API description (apis.io)](https://apis.io/apis/stooq/stooq-q-api/) (secondary source). An API key has been required since March 2026 — [pandas-datareader #1012](https://github.com/pydata/pandas-datareader/issues/1012)
- OpenFIGI API v3:
  - Rate limits: `/v3/mapping` allows 25 requests/min and 10 jobs per request without a key, or 250 requests/min and 100 jobs per request with one. `/v3/search` and `/v3/filter` allow 5/min without a key and 20/min with one. Limits are counted per key, or per IP when no key is sent.
  - idTypes include ID_ISIN, ID_CUSIP, ID_SEDOL, ID_WERTPAPIER (WKN), TICKER, ID_EXCH_SYMBOL, COMPOSITE_ID_BB_GLOBAL and more.
  - Results can be filtered by `exchCode` (Bloomberg exchange code) **or** `micCode` (ISO MIC), but not both in one job.
  - The API "is free and open to the public".

  — [OpenFIGI API docs](https://www.openfigi.com/api/documentation)
- OpenFIGI ToS: FIGI identifiers may be used "for any purpose, commercial or non-commercial, and in any way … including redistribution of the FIGI Identifiers to your customers", and are made available "to the public at large for free". The terms impose no attribution requirement, but the FIGI/OPENFIGI marks may not be used in a way that implies Bloomberg's endorsement. Bloomberg's liability is capped at US$50. — [OpenFIGI Terms of Service](https://www.openfigi.com/docs/terms-of-service)
- Ghostfolio (`API_KEY_OPEN_FIGI`) and Wealthfolio (`provider/openfigi`) both integrate OpenFIGI. — [Ghostfolio configuration.service.ts](https://github.com/ghostfolio/ghostfolio/blob/main/apps/api/src/services/configuration/configuration.service.ts); [Wealthfolio lib.rs](https://github.com/wealthfolio/wealthfolio/blob/main/crates/market-data/src/lib.rs)

### Inferences
- Canonical key: `(ticker, MIC)` for the listing, with ISIN stored as a separate attribute. MIC identifies venue, currency and timezone; ISIN alone cannot pick between, for example, SAP on XETR (EUR) and SAP on XNYS (USD ADR). Use OpenFIGI (ISIN + micCode → ticker) to help users who only know an ISIN.
- Keep a static MIC → {provider: suffix/code, quote currency} table, like Wealthfolio's exchanges.json, and store a per-instrument override for provider symbols that don't follow the rule.
- Expected forms for the requested examples. These follow from the rules above, not from live API calls:

  | Instrument | Yahoo | EODHD | Alpha Vantage | Twelve Data |
  |---|---|---|---|---|
  | THYAO (Borsa Istanbul) | THYAO.IS | THYAO.IS (unverified) | unverified | THYAO + mic_code XIST |
  | SAP (Xetra) | SAP.DE | SAP.XETRA | SAP.DEX | — |
  | VOD (LSE) | VOD.L | VOD.LSE | VOD.LON | — |
  | AIR (Euronext Paris) | AIR.PA | AIR.PA | AIR.PAR | — |

### Gaps
- I did not verify Finnhub and FMP symbol formats for non-US listings in this session; the provider researchers should confirm them. Wealthfolio maps Finnhub only for US bare symbols.
- I did not confirm EODHD's Istanbul exchange code ("IS"), or whether Alpha Vantage supports BIST.
- Twelve Data's own symbology documentation page did not render. The mic_code usage comes from Portfolio Performance's client code.

## 5. Design guidance for fallbacks with differing data (currency units, adjustments, holidays, timezones)

### Takeaway
Providers disagree on currency units (GBp vs GBP), adjustment (raw vs split/dividend-adjusted close) and the timestamp of the "daily close". A robust feed should normalise each provider's output into a canonical quote: MIC-local trading date, a raw close in the major currency unit, and a flag recording whether the value is adjusted. It should validate the quote, and carry the last trading day's close forward over weekends and holidays.

### Cited Findings
- **GBX/GBp vs GBP**: Yahoo returns LSE prices in GBp (pence) — [Wealthfolio exchange_suffixes.rs](https://github.com/wealthfolio/wealthfolio/blob/main/crates/market-data/src/resolver/exchange_suffixes.rs). Portfolio Performance's Yahoo feed has `convertBritishPounds`: it multiplies by 100 when the quote is GBP and the security is GBX, and divides by 100 when the quote is "GBp" and the security is GBP — [PP YahooFinanceQuoteFeed.java](https://github.com/portfolio-performance/portfolio/blob/master/name.abuchen.portfolio/src/name/abuchen/portfolio/online/impl/YahooFinanceQuoteFeed.java). Finance::Quote's Stooq module scales prices "if currency is GBX (GBp) or USX (USc)" — [Finance::Quote Stooq.pm](https://github.com/finance-quote/finance-quote/blob/master/lib/Finance/Quote/Stooq.pm). Wealthfolio's currency→MIC index treats "GBP", "GBX" and "GBp" all as XLON, and records Alpha Vantage's LSE currency as GBP while Yahoo's is GBp — [Wealthfolio exchanges.json](https://github.com/wealthfolio/wealthfolio/blob/main/crates/market-data/src/resolver/exchanges.json)
- **Adjusted vs raw close**: EODHD's `close` is "Closing price, as traded — not adjusted", its `adjusted_close` is "adjusted for both splits and dividends", and its volume is split-adjusted only. Adjusted values are recomputed when new dividends occur, so past values change, and EODHD advises against using them as stable keys — [EODHD historical data API](https://eodhd.com/financial-apis/api-for-historical-data-and-volumes). Portfolio Performance offers a separate "Yahoo Finance Adjusted Close" feed that reads Yahoo's `adjclose` array; high, low and volume are not available in that mode — [PP YahooFinanceAdjustedCloseQuoteFeed.java](https://github.com/portfolio-performance/portfolio/blob/master/name.abuchen.portfolio/src/name/abuchen/portfolio/online/impl/YahooFinanceAdjustedCloseQuoteFeed.java). Wealthfolio's registry fetches split and dividend history as separate operations, returning an empty vec rather than an error "if no provider supports splits" — [Wealthfolio provider_registry.rs](https://github.com/wealthfolio/wealthfolio/blob/main/crates/market-data/src/registry/provider_registry.rs)
- **Timezone and close time per venue**: Wealthfolio stores an IANA timezone and local close time per MIC: XNYS America/New_York 16:00, XLON Europe/London 16:30, XIST Europe/Istanbul 18:00 — [Wealthfolio exchanges.json](https://github.com/wealthfolio/wealthfolio/blob/main/crates/market-data/src/resolver/exchanges.json). Nasdaq's "Midnight Local Time" delay is defined in "the time zone where the Information is originally distributed", and Euronext's delayed files use UTC timestamps — [Nasdaq US Data Policies v2.6](https://www.nasdaqtrader.com/content/AdministrationSupport/Policy/USEquitiesandOptionsDataPolicies.pdf); [Euronext MiFID II data](https://www.euronext.com/en/data/pricing-specs-agreements/mifid-ii-compliant-data)
- **Validation before accepting a fallback quote**: OHLC invariants, non-negative values and range sanity; hard failures move on to the next provider — [Wealthfolio validator.rs](https://github.com/wealthfolio/wealthfolio/blob/main/crates/market-data/src/registry/validator.rs)

### Inferences
- **Holding valuation should use the raw (unadjusted) close.** It multiplies by a share count that already reflects splits, provided the app records split events against the holding. Adjusted close suits performance charts but rewrites history, so never mix adjusted values from one provider with raw values from another in one series. Store an `adjusted: bool` flag per stored price.
- Normalise currency at ingestion. Map GBp/GBX to GBP (÷100), and similarly ZAc and ILA if supported, and store the major unit plus the original provider unit. A quick sanity check between providers: a ratio near 100 between two providers' prices means a units mismatch, not a real price move.
- Weekend and holiday rule: valuation date D uses the latest close with trading date ≤ D in the listing's MIC timezone, and the UI shows the as-of date. Do not treat a missing weekend bar as a provider failure; that would trip the circuit breakers needlessly. A per-MIC holiday calendar is optional when you use the "latest bar ≤ D" rule.
- Key "daily close" by the exchange-local trading date, not by the UTC date of the timestamp. A US close at 16:00 New York is 21:00 UTC, and an Asia-Pacific bar can carry a timestamp from the previous UTC day.
- Make the UI wording follow the exchange rules: "Delayed" / "Close as of {date}".

### Gaps
- I did not verify which providers return GBp vs GBP for LSE beyond Yahoo, and Alpha Vantage as recorded in Wealthfolio's map.
- I did not find a standard free holiday-calendar source (e.g. exchange_calendars or pandas_market_calendars equivalents for Rust); not researched.
- I did not check provider-specific timestamp conventions for daily bars (midnight UTC vs exchange-local).
