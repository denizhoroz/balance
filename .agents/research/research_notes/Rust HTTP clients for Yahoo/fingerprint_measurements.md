# TLS / HTTP/2 fingerprints of Rust HTTP clients on Windows vs real browsers (measured, November 2026)

Measurement setup (all of it applies to every section below):
- Machine: Windows 11 Pro 10.0.26200, rustc/cargo 1.98.1 (stable-x86_64-pc-windows-msvc), 16 logical CPUs, CMake installed, no NASM and no clang on PATH.
- Measured on 2026-11-12 against `https://tls.peet.ws/api/all` and `https://tls.browserleaks.com/json`. Each client made 1 request to each endpoint, plus 1 request to Yahoo. Raw JSON responses are in `C:/Users/deniz/AppData/Local/Temp/claude/12c37cfe66099d51/426edb7f-dbee-4836-a41e-b613a55331d3/scratchpad/fingerprint-probe/` (`out-<variant>.txt`, `curl-*.json`, `edge-*.json`, `yahoo-*.txt`). The probe crate is in the `probe/` subfolder there.
- Resolved crate versions (from Cargo.lock): ureq 3.5.2, reqwest 0.13.5, rustls 0.23.46, ring 0.17.14, aws-lc-rs 1.19.0, aws-lc-sys 0.47.0, native-tls 0.2.18, schannel 0.1.30, hyper 1.12.0, h2 0.4.20, hyper-rustls 0.27.10, tokio 1.54.1, webpki-roots 1.0.12, rustls-platform-verifier 0.7.1.
- Every probe sent the same User-Agent: `Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/142.0.0.0 Safari/537.36`. This UA is deliberately browser-like and is now stale (the installed Edge is 157).
- Browser baseline: the installed Microsoft Edge 157.0.4341.46, run headless (`msedge.exe --headless=new --dump-dom <url>`) with a throwaway profile. Chrome and Firefox are not installed on this machine.

Probe variants (Cargo features; `default-features = false` on ureq and reqwest, `strip = true` in the release profile):

| Variant | Features | TLS stack |
|---|---|---|
| ureq_ring | `ureq/rustls`, `ureq/gzip` (equal to ureq's default) | rustls + ring |
| ureq_awslc | `ureq/rustls-no-provider`, `rustls-webpki-roots`, `gzip`, plus `rustls/aws-lc-rs`, with the provider set through `TlsConfig::builder().unversioned_rustls_crypto_provider(...)` | rustls + aws-lc-rs |
| ureq_native | `ureq/native-tls`, `gzip`, `TlsProvider::NativeTls` | SChannel |
| reqwest_rustls | `reqwest/default` + `blocking` (in 0.13 `default-tls` means `rustls`, which means aws-lc-rs) | rustls + aws-lc-rs |
| reqwest_ring | `rustls-no-provider` + `charset`, `http2`, `system-proxy`, `blocking`; installs `rustls::crypto::ring::default_provider()` | rustls + ring |
| reqwest_native | `native-tls` (with ALPN) + `charset`, `http2`, `system-proxy`, `blocking` | SChannel |

## Q1: What JA4/JA3 does rustls produce, and is it a known or blocked fingerprint?

### Takeaway
Measured: rustls 0.23.46 gives JA4 `t13d1010??_61a7ad8aa9b6_<c>` on Windows. With ring, c is `3fcd1a44f3e3`; with aws-lc-rs, c is `0d308c48d2a3` (it adds X25519MLKEM768 and ML-DSA signature algorithms). ureq sends **no ALPN** (`00`) and reqwest sends `h2`. It looks nothing like Chrome/Edge (`t13d1516h2_8daaf6152771_806a8c22fdea`): there is no GREASE, the cipher list is different, it offers the `0x00ff` SCSV, and it lacks ALPS, ECH, compress_certificate and SCT. The JA3 hash changes on every connection because rustls shuffles extension order. I found no public evidence that this exact rustls JA4 is on a blocklist, and the main public database (ja4db) is no longer publicly queryable.

### Cited Findings
- **Measured, ureq 3.5.2 + rustls/ring:** JA4 `t13d101000_61a7ad8aa9b6_3fcd1a44f3e3` (browserleaks). tls.peet.ws printed it as `t13d1010_61a7ad8aa9b6_3fcd1a44f3e3` and left out the `00` no-ALPN marker, which looks like a peet formatting quirk. Raw JA4: ciphers `00ff,1301,1302,1303,c02b,c02c,c02f,c030,cca8,cca9`; extensions `0005,000a,000b,000d,0017,0023,002b,002d,0033` plus SNI and no ALPN; sigalgs `0503,0403,0807,0806,0805,0804,0601,0501,0401`. Groups X25519, P-256, P-384, with one key share (X25519). peetprint `034f483a0e0ac8bc534db4bea4f59eaf`. Negotiated TLS 1.3 over HTTP/1.1 — measured via [tls.peet.ws](https://tls.peet.ws/api/all) and [browserleaks](https://tls.browserleaks.com/json)
- **Measured, ureq 3.5.2 + rustls/aws-lc-rs:** JA4 `t13d101000_61a7ad8aa9b6_0d308c48d2a3`. The groups add `X25519MLKEM768 (4588)` as the **last** group, and only an X25519 key share is sent. The sigalgs add `0603,0904,0905,0906` (ecdsa_secp521r1 and ML-DSA). peetprint `4dd7ffe772520c629942d57793dc3a15` — measured via [tls.peet.ws](https://tls.peet.ws/api/all), [browserleaks](https://tls.browserleaks.com/json)
- **Measured, reqwest 0.13.5 default (rustls + aws-lc-rs):** JA4 `t13d1011h2_61a7ad8aa9b6_0d308c48d2a3`, ALPN `h2, http/1.1`. The only differences from ureq are the ALPN extension (count 11 and `h2`); the c-hash is identical to ureq_awslc, because JA4 drops ALPN and SNI from the extension hash — measured; JA4 rule from the [FoxIO JA4 spec](https://github.com/FoxIO-LLC/ja4/blob/main/technical_details/JA4.md)
- **Measured, reqwest 0.13.5 + rustls/ring:** JA4 `t13d1011h2_61a7ad8aa9b6_3fcd1a44f3e3` — measured via [browserleaks](https://tls.browserleaks.com/json)
- **Measured, JA3 is unstable for rustls:** two consecutive ureq_ring requests gave JA3 `f86106924fa97187ba5612c3b95c0aa4` and `353b803bb026a46c36f3004d5223b577`, and every other rustls variant also differed between requests. rustls 0.23.46 source `msgs/handshake.rs` has `order_insensitive_extensions_in_random_order()` ("Returns extensions which don't need a specific order, in randomized order"). JA3N (sorted) and JA4 stay stable — measured; source read from the crate registry copy of [rustls 0.23.46](https://github.com/rustls/rustls)
- **Post-quantum ordering detail:** rustls's own default features include `prefer-post-quantum`, which would put X25519MLKEM768 first. ureq and reqwest depend on rustls with `default-features = false`, so in the measured builds ML-KEM is listed last and not key-shared. An app that also depends on rustls with default features would, through feature unification, flip that ordering (this changes JA3 but not JA4, since JA4 does not hash groups) — source: rustls 0.23.46 Cargo.toml `default = ["aws_lc_rs","logging","prefer-post-quantum", ...]`, ureq 3.5.2 / reqwest 0.13.5 Cargo.toml; JA4 fields per the [FoxIO JA4 spec](https://github.com/FoxIO-LLC/ja4/blob/main/technical_details/JA4.md)
- **Measured, real Edge 157 (Chromium/BoringSSL):** JA4 `t13d1516h2_8daaf6152771_806a8c22fdea`. Ciphers `1301,1302,1303,c02b,c02f,c02c,c030,cca9,cca8,c013,c014,009c,009d,002f,0035` plus GREASE. Extensions include GREASE, ALPS `17613`, ECH `65037`, compress_certificate `27`, SCT `18` and renegotiation_info `65281`. Groups: GREASE, X25519MLKEM768, X25519, P-256, P-384, with key shares GREASE, X25519MLKEM768 and X25519. JA3 also varied between the two requests (`2334df6f...` vs `91726e0b...`) — measured via [tls.peet.ws](https://tls.peet.ws/api/all), [browserleaks](https://tls.browserleaks.com/json)
- FoxIO's reference table lists Chrome (TCP) as `t13d1517h2_8daaf6152771_cb7bf5808d99`. The measured Edge 157 has the same cipher hash `8daaf6152771` but a different extension count and c-hash, which shows the c part drifts across browser versions — [FoxIO JA4 README](https://raw.githubusercontent.com/FoxIO-LLC/ja4/main/README.md)
- Firefox (not measured here) has a different cipher hash, `5b57614c22b0`. Firefox 127 desktop is listed as `t13d1715h2_5b57614c22b0_3d5424432f57` (third-party blog, not verified) — [ip8.com](https://ip8.com/blog/ja3-ja4-tls-fingerprinting)
- JA4 rules: a = protocol + version + SNI d/i + cipher count + extension count + ALPN first/last char, with `00` when there is no ALPN. Extensions are sorted, SNI and ALPN are removed from the c-hash, and GREASE is ignored everywhere — [FoxIO JA4 spec](https://github.com/FoxIO-LLC/ja4/blob/main/technical_details/JA4.md)
- **ja4db.com now redirects to ja4db.foxio.io. Its API (`/api/ja4/`, `/api/ja4/common/`, `/api/ja4/count/`) returned HTTP 403 "Authentication credentials were not provided."** The old `https://ja4db.com/api/read/` gives 404. So the public JA4 database could not be queried for the rustls hashes without an account — measured 2026-11-12 with curl against [ja4db.foxio.io](https://ja4db.foxio.io/)
- Web searches for the rustls cipher hash `61a7ad8aa9b6` found no public page attributing it — [search result summary, no hits]; vendor blogs only say generically that rustls is a distinct ClientHello source — [Scrappey](https://scrappey.com/qa/anti-bot/what-is-ja4-fingerprinting), [botoi](https://botoi.com/blog/detect-ai-scrapers-tls-fingerprint/)
- Upstream rustls has a feature-request thread about exposing ClientHello customization for fingerprinting (#1932). Forks such as craftls (browser profiles: Chrome 108/112, Safari 17.1, Firefox 105; last release about Jan 2024) exist, so stock rustls cannot impersonate browsers — [rustls #1932](https://www.github.com/rustls/rustls/issues/1932), [craftls](https://github.com/3andne/craftls)
- Cloudflare's JA4 Signals give each JA4 a `browser_ratio_1h` (share of requests from browser UAs over the last hour, globally) and an `h2h3_ratio_1h`. Cloudflare itself warns that "fingerprints can be easily spoofed, they change frequently" — [Cloudflare blog: JA4 signals](https://blog.cloudflare.com/ja4-signals/)
- Cloudflare JA3/JA4 fields are Enterprise + Bot Management only. Customers block, challenge or skip by fingerprint in WAF custom rules. The docs do not say that non-browser fingerprints are automatically scored as bots — [Cloudflare docs](https://developers.cloudflare.com/bots/additional-configurations/ja3-ja4-fingerprint/)

### Inferences
- Any server that compares JA4 with the User-Agent will see a "Chrome" UA on a non-Chromium ClientHello for every rustls variant. That mismatch is exactly what vendor blogs describe as a strong bot signal. The rustls fingerprint is not "malicious", but its `browser_ratio` (in Cloudflare terms) is presumably low, because rustls is used by Rust tools, CLIs and servers rather than browsers.
- If you use rustls, the choice between ring and aws-lc-rs changes only the JA4 c-hash; neither is closer to a browser. aws-lc-rs at least advertises X25519MLKEM768, as modern browsers do, but still does not key-share it first.
- JA3 blocklists cannot pin rustls reliably (randomized order), but JA3N, JA4 and peetprint can.

### Gaps
- I could not confirm whether `t13d101000_61a7ad8aa9b6_*` or `t13d1011h2_61a7ad8aa9b6_*` is labelled in ja4db or in any vendor blocklist, because ja4db now requires login.
- I found no published Cloudflare/Akamai statistic for the rustls JA4's browser ratio.
- Real headed Chrome and Firefox were not measured (not installed). Headless Edge uses the same BoringSSL network stack as headed Edge. The UA differs ("HeadlessChrome"), but I would expect the TLS to be the same. That is an inference, not verified on a headed run.

## Q2: Does Windows SChannel (native-tls, curl.exe) look like Edge/Chrome on Windows?

### Takeaway
No. Edge is Chromium and does its own TLS through BoringSSL. SChannel's ClientHello is completely different: 20 ciphers including legacy CBC/RSA-kx suites, no GREASE, no ALPS/ECH/compress_cert, three key shares (X25519, P-256, P-384), and `post_handshake_auth`. SChannel's JA4 a+b part (`t13d201100_2b729b4bf6f3`) is **exactly the prefix FoxIO publishes as the IcedID malware example**. IcedID is Windows malware that rides the OS TLS stack. So native-tls blends in with "generic Windows software" (including malware), not with browsers.

### Cited Findings
- **Measured, ureq 3.5.2 + native-tls (SChannel):** JA4 `t13d201100_2b729b4bf6f3_36bf25f296df`, JA3 `68b3ecfaf0034bb9fcbecd518b5ab8d4` (stable across both requests, since SChannel does not randomize). No ALPN and no status_request. Ciphers (in order): `1302,1301,c02c,c02b,c030,c02f,c024,c023,c028,c027,c00a,c009,c014,c013,009d,009c,003d,003c,0035,002f`. Extensions `0,43,13,35,10,11,51,49,23,65281,45`. Groups X25519, P-256, P-384, **all three key-shared**. Sigalgs include `0201` and `0203` (SHA-1). HTTP/1.1 — measured via [tls.peet.ws](https://tls.peet.ws/api/all), [browserleaks](https://tls.browserleaks.com/json)
- **Measured, reqwest 0.13.5 + native-tls (SChannel, ALPN on):** JA4 `t13d2012h2_2b729b4bf6f3_36bf25f296df`, JA3 `2e6c64f66822fc35b6a7a128b557f1de`, ALPN `h2, http/1.1`, negotiated h2 — measured via [browserleaks](https://tls.browserleaks.com/json)
- **Measured, Windows curl.exe 8.22.0 (Schannel; `curl --version`: "libcurl/8.22.0 Schannel zlib/1.3.2 WinIDN", with no HTTP2 in the Features list):** JA4 `t13d2013h1_2b729b4bf6f3_e24568c0d440`, JA3 `fae0e5d973c96ae1888b99538efa0363`. It adds `status_request (5)` and ALPN `http/1.1` and otherwise uses the SChannel cipher list. It negotiated HTTP/1.1 — measured via [tls.peet.ws](https://tls.peet.ws/api/all), [browserleaks](https://tls.browserleaks.com/json)
- **Measured, Edge 157:** `t13d1516h2_8daaf6152771_806a8c22fdea` (see Q1), which shares nothing with SChannel's `2b729b4bf6f3` cipher hash — measured
- FoxIO's JA4 README lists **IcedID Malware = `t13d201100_2b729b4bf6f3_9e7b989ebec8`**. That is the same a and b part as the measured ureq native-tls output; only the c-hash differs, probably because of a different Windows build's extension and sigalg set — [FoxIO JA4 README](https://raw.githubusercontent.com/FoxIO-LLC/ja4/main/README.md); APNIC/FoxIO blog repeats it and notes that "Windows functions will be unique" — [APNIC blog](https://blog.apnic.net/2023/11/22/ja4-network-fingerprinting/)
- Chromium Edge "no longer uses Windows' SChannel (except for IEMode tabs)", so SChannel cipher policies do not affect it — [Eric Lawrence, textslashplain](https://textslashplain.com/2020/02/26/can-i-in-the-new-edge/); a developer test likewise found that SChannel cipher changes had no effect on Edge — [jenkins96](https://jenkins96.github.io/2024-04-13-Edge-Cipher-Suites)

### Inferences
- SChannel's fingerprint is shared by a huge and varied population: WinHTTP/WinINet, PowerShell, Windows services, many .NET-on-Windows apps (SslStream uses SChannel; this is background knowledge and was not measured here), and malware. Population-ratio signals such as Cloudflare's `browser_ratio_1h` would therefore read it as "non-browser" but common. It probably does not trip "rare fingerprint" heuristics. A Chrome UA on top of it is still a cross-layer mismatch.
- native-tls is not "a cheap way to look like Edge". At best it is a cheap way to look like curl.exe or an ordinary Windows program, which compared with rustls trades a rare fingerprint for a common one. Its 3 key shares and legacy CBC suites also make it more Windows-version-dependent (the c-hash changes across Windows builds), so it will not even be stable across user machines.
- To look like Chrome at the TLS layer you need a BoringSSL-based impersonation stack (for example, the curl-impersonate / curl_cffi approach). That is outside "cheap".

### Gaps
- No measurement on Windows 10 to show how the SChannel c-hash differs across OS versions.
- No published per-fingerprint population statistics for SChannel were found.

## Q3: HTTP/1.1 vs HTTP/2 — does ureq (HTTP/1.1 only) stand out?

### Takeaway
Yes, by construction: ureq sends no ALPN at all, so its JA4 has `00` where every browser has `h2`, and there is no HTTP/2 Akamai fingerprint. reqwest negotiates h2, but its hyper/h2 Akamai fingerprint (`2:0;4:2097152;5:16384;6:16384|5177345|0|m,s,a,p`) is plainly not Chrome's (`1:65536;2:0;4:6291456;6:262144|15663105|0|m,a,s,p`), and it is identical whichever TLS backend is used. Windows curl.exe is also HTTP/1.1-only, so "SChannel + HTTP/1.1" is a common Windows-tool shape.

### Cited Findings
- **Measured, HTTP/2 Akamai fingerprints:**
  - Edge 157: `1:65536;2:0;4:6291456;6:262144|15663105|0|m,a,s,p` (hash `52d84b11737d980aef856699f885ca86`).
  - reqwest 0.13.5 (hyper 1.12 / h2 0.4.20) with all three TLS backends: `2:0;4:2097152;5:16384;6:16384|5177345|0|m,s,a,p` (hash `9b5dcd077a77c5e324b91d8cc306fd5a`).
  - ureq and curl.exe: none, because they use HTTP/1.1 — measured via [tls.peet.ws](https://tls.peet.ws/api/all), [browserleaks](https://tls.browserleaks.com/json)
- **Measured, headers on the wire:**
  - reqwest h2 sent only `accept: */*` and `user-agent`, with pseudo-header order `:method :scheme :authority :path`.
  - Edge sent 13 regular headers: sec-ch-ua, sec-ch-ua-mobile, sec-ch-ua-platform, upgrade-insecure-requests, user-agent, accept (html), sec-fetch-site/mode/user/dest, accept-encoding `gzip, deflate, br, zstd`, accept-language, priority. Its pseudo-header order is `:method :authority :scheme :path`.
  - ureq HTTP/1.1 sent `accept-encoding: gzip`, `accept: */*`, `host`, `user-agent`.
  - curl.exe sent `Host`, `User-Agent`, `Accept: */*`.
  - Measured via [tls.peet.ws](https://tls.peet.ws/api/all)
- **Derived JA4H_a prefixes** (computed by hand from the headers above using JA4H's documented format; the FoxIO JA4H page fetched only confirmed the header-count rule): ureq `ge11nn040000`, curl.exe `ge11nn030000`, reqwest `ge20nn020000`, Edge `ge20nn13enus` (approximately). The missing Accept-Language (`0000`) and the low header count are visible tells. Header-count rule from the [FoxIO JA4H spec](https://github.com/FoxIO-LLC/ja4/blob/main/technical_details/JA4H.md)
- The ALPN characters in JA4_a distinguish h2 from h1 and `00` (no ALPN) — [FoxIO JA4 spec](https://github.com/FoxIO-LLC/ja4/blob/main/technical_details/JA4.md)
- Cloudflare computes `h2h3_ratio_1h` per JA4: the share of a fingerprint's requests that use HTTP/2 or HTTP/3 — [Cloudflare blog: JA4 signals](https://blog.cloudflare.com/ja4-signals/)
- Vendor and blog sources say detection checks that HTTP/2 SETTINGS, window update, priority and pseudo-header order are consistent with the TLS fingerprint and the UA, and that a client whose h2 settings do not match the browser it claims to be fails that check — [Scrapfly HTTP/2 fingerprinting guide](https://scrapfly.io/blog/posts/http2-http3-fingerprinting-guide) (vendor blog, unverified specifics)

### Inferences
- HTTP/1.1 itself is not damning: curl.exe, PowerShell, many SDKs and health checkers use it. But a "Chrome/1xx" UA over HTTP/1.1 with no ALPN does not occur for real Chrome, so the mismatch makes it look worse than an honest UA would.
- Switching from ureq to reqwest makes you no more browser-like. It swaps "no ALPN" for a hyper h2 fingerprint that is just as non-browser.
- The cheapest consistency fix is to stop claiming Chrome in the UA (send an honest app UA), not to change the TLS stack. That approach is untested against Yahoo.

### Gaps
- No public data found on how much weight Yahoo or other CDNs give HTTP/1.1 by itself.
- I did not compute full JA4H hashes (b/c/d parts).

## Q4: Does Yahoo or its CDN use JA3/JA4-based rules?

### Takeaway
There is no first-party evidence. Yahoo Finance's API edge is Yahoo's own Apache Traffic Server (`server: ATS`, behind Envoy), not Cloudflare or Akamai. ATS ships JA3, JA4 and JAx fingerprint plugins, so the capability exists. The only "TLS fingerprinting" claims come from yfinance users and the yfinance docstring. In my one-request-per-client test from a residential IP, **all 7 clients (all rustls/native-tls variants and curl.exe) got HTTP 200** with real chart data.

### Cited Findings
- **Measured, 2026-11-12 ~13:12 UTC, `GET https://query2.finance.yahoo.com/v8/finance/chart/AAPL?range=5d&interval=1d` with the Chrome UA, one request each:** curl.exe 200 (HTTP/1.1), ureq_ring 200 (1.1), ureq_awslc 200 (1.1), ureq_native 200 (1.1), reqwest_rustls 200 (h2), reqwest_ring 200 (h2), reqwest_native 200 (h2). The bodies were valid `{"chart":{"result":[{"meta":{"currency":"USD","symbol":"AAPL",...`. 8 Yahoo requests in total, counting one extra header dump — measured; endpoint [query2.finance.yahoo.com](https://query2.finance.yahoo.com/v8/finance/chart/AAPL?range=5d&interval=1d)
- **Measured response headers:** `server: ATS`, `x-envoy-upstream-service-time`, `x-envoy-decorator-operation: finance-chart-api--mtls-production-ir2.finance-k8s:4080/*`, `y-rid`, `cache-control: public, max-age=10, stale-while-revalidate=20`. There are no Cloudflare or Akamai headers — measured
- Apache Traffic Server has a `ja3_fingerprint` plugin (adds an `X-JA3-Sig` upstream header), a `ja4_fingerprint` plugin (experimental), and a `jax_fingerprint` plugin (JA3/JA4/JA4H) — [ATS JA4 plugin docs](https://docs.trafficserver.apache.org/en/latest/admin-guide/plugins/ja4_fingerprint.en.html), [ATS JA3 plugin docs](https://docs.trafficserver.apache.org/admin-guide/plugins/ja3_fingerprint.en.html), [ATS JAx plugin docs](https://docs.trafficserver.apache.org/admin-guide/plugins/jax_fingerprint.en.html)
- The Yahoo 429 body is literally `Edge: Too Many Requests` (23 bytes), often returned from `/v1/test/getcrumb` — [PythonAnywhere forum](https://www.pythonanywhere.com/forums/topic/35819/), [yfinance #2128](https://github.com/ranaroussi/yfinance/issues/2128)
- In yfinance #2422 (April–May 2025), single `yf.download` calls failed with `YFRateLimitError('Too Many Requests')` across IPs. A user (not the maintainer) reported that "it works with curl too" but not with Python `requests`, and suspected TLS fingerprinting; the workaround was a curl_cffi `impersonate="chrome"` session — [yfinance #2422](https://github.com/ranaroussi/yfinance/issues/2422)
- yfinance's current `_http.py` returns `curl_cffi Session(impersonate="chrome")` by default. Its docstring says "plain `requests` cannot replicate the JA3/JA4 fingerprint and HTTP/2 settings that `curl_cffi` provides, so Yahoo Finance may rate-limit or block this client". It falls back to `requests` if `YF_DISABLE_CURL_CFFI` is set — [yfinance _http.py](https://raw.githubusercontent.com/ranaroussi/yfinance/main/yfinance/_http.py)
- Rate-limit detection in yfinance is `status_code == 429 or "Too Many Requests" in crumb` — [yfinance data.py](https://raw.githubusercontent.com/ranaroussi/yfinance/main/yfinance/data.py)
- Portfolio Performance forum users reported Yahoo "too many requests" errors and said `impersonate="chrome"` helped (anecdotal) — [Portfolio Performance forum](https://forum.portfolio-performance.info/t/yahoo-finance-to-many-requests-getting-errors-on-updates-desktop-only/32780)

### Inferences
- The 2025 reports fit a policy that keys on the client signature as a whole (Python-requests UA and/or OpenSSL JA3/JA4), but they do not prove it. The "works with curl" report argues against a naive "anything that isn't Chrome's JA4 gets blocked" rule, since curl is not Chrome either. My measurement agrees: rustls and SChannel fingerprints, at low volume, are served fine as of 2026-11-12.
- The likely practical risk for a desktop app is per-IP or per-session rate limiting at the ATS edge (the `Edge:` prefix), driven by request volume and the cookie/crumb flow, rather than a TLS-only block. TLS fingerprint may well be one input, since ATS can compute JA3/JA4, but there is no evidence of how it is weighted.

### Gaps
- No Yahoo statement or engineering post on whether ATS fingerprint plugins are deployed on finance endpoints.
- My Yahoo test was 1 request per client from one residential IP. It says nothing about thresholds under sustained polling, about crumb/cookie endpoints (`/v1/test/getcrumb`, `quoteSummary`), or about datacenter IPs.

## Build-cost data (Q-adjacent, useful for crate choice)

### Takeaway
On this machine, clean `cargo build --release` times were: ureq + native-tls 6 s (0.96 MiB exe), ureq + ring 12 s (2.20 MiB), reqwest + native-tls 16 s (2.71 MiB), reqwest + ring 22 s (4.13 MiB), reqwest default (aws-lc-rs) 30 s (4.98 MiB), ureq + aws-lc-rs 34 s (3.10 MiB). aws-lc-rs built without NASM or clang installed (CMake was present).

### Cited Findings
- **Measured clean release builds** (fresh `--target-dir` each, run one after another, second pass with no other load; `strip = true`, no LTO, 16 threads; crates already downloaded):

| Variant | Clean build (s), pass 2 | Pass 1 | Stripped probe.exe size |
|---|---|---|---|
| ureq_native (SChannel) | 6 | 88* | 1,001,472 B (0.96 MiB) |
| ureq_ring | 12 | 14 | 2,309,120 B (2.20 MiB) |
| reqwest_native | 16 | 19 | 2,844,672 B (2.71 MiB) |
| reqwest_ring | 22 | 19 | 4,335,104 B (4.13 MiB) |
| reqwest_rustls (aws-lc-rs, default) | 30 | 31 | 5,218,304 B (4.98 MiB) |
| ureq_awslc | 34 | 26 | 3,248,128 B (3.10 MiB) |

  \*The pass-1 ureq_native time overlapped with concurrent network probes and a cold disk cache. It was re-timed at 6 s (twice), so 88 s is an outlier. Logs are in `C:/Users/deniz/AppData/Local/Temp/claude/12c37cfe66099d51/426edb7f-dbee-4836-a41e-b613a55331d3/scratchpad/fingerprint-probe/` (`build-*.log`, `retime-*.log`, `build_summary.txt`, `retime_summary.txt`) — measured
- aws-lc-sys 0.47.0 built on x86_64-pc-windows-msvc with CMake present and **no NASM and no clang on PATH**, without errors — measured
- reqwest 0.13.5 defaults: `default = ["default-tls","charset","http2","system-proxy"]` with `default-tls = ["rustls"]` and `rustls = ["__rustls-aws-lc-rs", "dep:rustls-platform-verifier", ...]`. So the reqwest 0.13 default is rustls + aws-lc-rs + platform verifier, not native-tls as in 0.11/0.12. ureq 3.5.2 `default = ["rustls","gzip"]` with `rustls = ["rustls-no-provider","_ring","rustls-webpki-roots"]` (ring + webpki-roots). If no provider is set, ureq picks a process default, then ring — source: crate Cargo.toml and `src/tls/rustls.rs` from the crates.io registry copies of [ureq 3.5.2](https://crates.io/crates/ureq) and [reqwest 0.13.5](https://crates.io/crates/reqwest)

### Inferences
- If the TLS fingerprint does not buy browser-likeness either way, ureq + native-tls is the cheapest option on Windows: about 1 MB binary, a 6 s clean build, and the OS certificate store. It also yields a very common (non-browser) Windows fingerprint. The trade-off is the OS-version-dependent ClientHello and the shared JA4 prefix with Windows malware samples.

### Gaps
- Incremental build cost inside the real app (with its other dependencies) was not measured; these are standalone probe numbers.
