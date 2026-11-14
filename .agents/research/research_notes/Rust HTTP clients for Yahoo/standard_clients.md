# Standard (non-impersonating) Rust HTTP clients for a Windows gpui desktop app (state as of 2026-11-12)

Method note: version, date, license and download data come from the crates.io API (`https://crates.io/api/v1/crates/<name>`). Repo stats (stars, open issues+PRs combined, last push) come from the GitHub REST API (`https://api.github.com/repos/<owner>/<repo>`), both queried on 2026-11-12. The dependency counts and binary sizes are **my own measurements** on this Windows 11 machine (rustc 1.98.1, x86_64-pc-windows-msvc). The method is described under Q4. Each measured test binary also made one live GET request to `https://query1.finance.yahoo.com/v8/finance/chart/AAPL` with `User-Agent: x`.

## Q1. Latest version, release date, maintenance, license

### Takeaway
ureq 3.x, reqwest 0.13, hyper/hyper-util, isahc 2.x, attohttpc, minreq 3, curl-rust, nyquest and ehttp all had releases in 2026. ureq shipped 3.5.2 on the day of research. surf is effectively dead, with its last release in Nov 2021. ureq 2.x is frozen at 2.12.1 (Dec 2024).

### Cited Findings
| Crate | Latest (date) | Recent releases | License | GitHub stars / open issues+PRs / last push | 90-day downloads |
|---|---|---|---|---|---|
| ureq 3.x | **3.5.2 (2026-11-12)** | 3.5.1 11-02, 3.5.0 10-21, 3.4.3 10-21 | MIT OR Apache-2.0 | 2,233 / 45 / 2026-11-12 (algesten/ureq) | ~85.9M |
| ureq 2.x | **2.12.1 (2024-12-04)**, the last 2.x release | 2.12.0 2024-12-03 | MIT OR Apache-2.0 | same repo | — |
| reqwest | **0.13.5 (2026-09-08)** | 0.13.4 05-25, 0.13.3 04-27, 0.13.2 02-06 | MIT OR Apache-2.0 | 11,932 / 494 / 2026-11-11 (seanmonstar/reqwest) | ~261M |
| hyper | **1.12.0 (2026-10-06)** | 1.11.1 08-28, 1.11.0 07-20 | MIT | 16,430 / 234 / 2026-11-09 | ~280M |
| hyper-util | **0.1.21 (2026-09-24)** | 0.1.20 2026-02-02 | MIT | 182 / 20 / 2026-11-05 | ~211M |
| isahc | **2.0.2 (2026-10-21)** (2.0.0 on 2026-05-25; 1.8.3 maintenance release 2026-07-05) | 2.0.1 07-05 | MIT | 791 / 37 / 2026-11-06 (sagebind/isahc) | ~1.17M |
| attohttpc | **0.32.0 (2026-11-05)** | 0.31.0 2026-05-25, 0.30.1 2025-07-29 | **MPL-2.0** | 294 / 16 / 2026-11-12 (sbstp/attohttpc) | ~7.7M |
| minreq | **3.0.0 (2026-06-15)** | 3.0.0-rc.1 04-21, 2.14.1 2025-09-03 | ISC | 257 / 6 / 2026-09-09 (neonmoe/minreq) | ~1.5M |
| curl (curl-rust) | **0.4.51 (2026-10-08)** | 0.4.50 2026-06-11 | MIT | 1,137 / 104 / 2026-10-16 (sagebind/curl-rust) | ~5.9M |
| surf | **2.3.2 (2021-11-01)** | none since 2021 | MIT OR Apache-2.0 | 1,505 / 102 / **last push 2023-09-11** (http-rs/surf) | ~0.49M |
| nyquest (other notable) | **0.5.0 (2026-10-18)** | 0.4.0 2025-12-17 | MIT OR Apache-2.0 | 249 / 6 / 2026-11-06 (bdbai/nyquest) | ~2.7K |
| ehttp (other notable, emilk/egui author) | **0.7.3 (2026-11-03)** | 0.7.2 10-21 | MIT OR Apache-2.0 | 399 / 13 / 2026-11-05 | ~1.05M |
| cyper (other notable, compio) | 0.9.0 (2026-06-01); 0.10.0-beta.1 (2026-10-15) | | MIT | not queried | ~68K |

Sources: [crates.io API ureq](https://crates.io/api/v1/crates/ureq), [reqwest](https://crates.io/api/v1/crates/reqwest), [hyper](https://crates.io/api/v1/crates/hyper), [hyper-util](https://crates.io/api/v1/crates/hyper-util), [isahc](https://crates.io/api/v1/crates/isahc), [attohttpc](https://crates.io/api/v1/crates/attohttpc), [minreq](https://crates.io/api/v1/crates/minreq), [curl](https://crates.io/api/v1/crates/curl), [surf](https://crates.io/api/v1/crates/surf), [nyquest](https://crates.io/api/v1/crates/nyquest), [ehttp](https://crates.io/api/v1/crates/ehttp), [cyper](https://crates.io/api/v1/crates/cyper). GitHub stats are from [api.github.com/repos/algesten/ureq](https://api.github.com/repos/algesten/ureq) and the equivalent endpoints for the other repos.

- ureq 3.5.x changelog highlights: `Agent::shutdown()` interrupts in-flight requests (3.5.0); `RequestBuilder::header_raw` preserves validated header casing (3.5.0); `timeout_per_read` and `timeout_per_write` added (3.5.0); the TLS handshake is bounded by the connect/global timeout (3.5.0); "Send only cookie names and values in request Cookie headers" (3.5.1); response body limits now apply after decoding (3.5.2). MSRV went to 1.85 with edition 2024 in 3.3.0. The crates.io metadata lists rust_version 1.85 for 3.5.2. — [ureq CHANGELOG](https://github.com/algesten/ureq/blob/main/CHANGELOG.md)
- reqwest 0.13.0 breaking changes: "`rustls` is now the default TLS backend, instead of `native-tls`"; "rustls crypto provider defaults to aws-lc instead of ring"; `rustls-platform-verifier` is used by default; `query` and `form` became opt-in features; `native-tls` now includes ALPN. 0.13.4 added `http2_keep_alive_*` for the blocking client and TLS 1.3 for native-tls. 0.13.5 added `Error::is_dns()` and `http1_max_headers`. — [reqwest CHANGELOG](https://github.com/seanmonstar/reqwest/blob/master/CHANGELOG.md)
- isahc 2.0.0 (2026-05-25) supports multiple TLS backends, and "the default TLS backend has been changed to be rustls" (rustls inside libcurl). 2.0.1 bundles libcurl 8.21.0 to fix a DNS-hang bug in curl 8.20.0. 2.0.2 bumps webpki-root-certs to drop the compromised eMudhra CA. — [isahc CHANGELOG](https://github.com/sagebind/isahc/blob/master/CHANGELOG.md), [isahc releases](https://github.com/sagebind/isahc/releases)
- minreq 3.x targets MSRV 1.63 (Debian oldstable) under a stable per-major MSRV policy. — [minreq README](https://github.com/neonmoe/minreq/blob/master/README.md)

### Inferences
- For a pick-the-default decision, the only realistic heavyweight-vs-lightweight pair is reqwest vs ureq 3. Both are very actively maintained. reqwest's 494 open issues+PRs reflect its much larger user base, not neglect.
- surf should be excluded: no release in 5 years and no push in over 3 years.
- attohttpc is MPL-2.0 (file-level copyleft). That is usually fine for an app binary, but it differs from the MIT/Apache norm.

### Gaps
- GitHub `open_issues_count` mixes issues and PRs. I did not split them.
- Maintainer identity: these are the repo owners (algesten, seanmonstar, sagebind, sbstp, neonmoe, bdbai, emilk). I did not verify current co-maintainers.
- I did not verify any official "maintenance mode" statement for ureq 2.x. It is only inferred from the absence of 2.x releases since 2024-12.

## Q2. Blocking vs async, runtime requirements, integration into a non-tokio gpui app

### Takeaway
ureq, attohttpc, minreq and curl (Easy) are pure blocking with no runtime, and drop straight into gpui background-executor tasks, as the app does today. `reqwest::blocking` works without the app owning tokio. It does this by spawning its own internal thread with a tokio current-thread runtime, and it panics if called from inside an async runtime. isahc and nyquest offer both blocking and runtime-agnostic async APIs.

### Cited Findings
- ureq: "It uses blocking I/O instead of async I/O, because that keeps the API simple and keeps dependencies to a minimum." — [ureq README](https://github.com/algesten/ureq/blob/main/README.md)
- reqwest blocking client internals: it spawns a thread named `"reqwest-internal-sync-runtime"`, builds a `runtime::Builder::new_current_thread()` tokio runtime, and drives requests with `rt.block_on(f)` / `tokio::spawn(forward(req_fut, req_tx))`. — [reqwest src/blocking/client.rs](https://github.com/seanmonstar/reqwest/blob/master/src/blocking/client.rs)
- reqwest blocking docs: blocking functionality "must _not_ be executed within an async runtime, or it will panic when attempting to block". They recommend the async Client, or `tokio::task::spawn_blocking`, when the caller is async. The `blocking` feature must be enabled. — [docs.rs reqwest::blocking](https://docs.rs/reqwest/0.13.5/reqwest/blocking/index.html)
- reqwest `blocking` feature = `futures-channel`, `futures-util`, `tokio/sync`. tokio is a non-optional dependency of reqwest. — [crates.io reqwest 0.13.5 features](https://crates.io/api/v1/crates/reqwest/0.13.5); [docs.rs reqwest::blocking](https://docs.rs/reqwest/0.13.5/reqwest/blocking/index.html)
- attohttpc "stays away from Rust's asynchronous stack to provide a crate that's as small as possible". It is HTTP/1.1. — [attohttpc README](https://github.com/sbstp/attohttpc/blob/master/README.md)
- nyquest has "async variant, it is not tied to any specific async runtime". Backends use system event loops (WinHTTP, WinRT HttpClient, libcurl, NSURLSession, reqwest). It says to consider it if you "do not use an async runtime at all" or "do not want to pull in the whole hyper or reqwest stack". — [nyquest README](https://github.com/bdbai/nyquest/blob/main/README.md)
- nyquest features: `async`, `blocking`, `blocking-stream`, `async-stream`, `json`, `multipart`. — [crates.io nyquest 0.5.0](https://crates.io/api/v1/crates/nyquest/0.5.0)
- ehttp's `platform-verifier` feature maps to `ureq/platform-verifier`, so ehttp is a thin wrapper over ureq on native targets (plus web fetch on wasm). It has a `native-async` feature using `async-channel`. — [crates.io ehttp 0.7.3](https://crates.io/api/v1/crates/ehttp/0.7.3)
- reqwest's `system-proxy` feature is implemented in hyper-util (`hyper-util/client-proxy-system`), so hyper-util is the tokio-based connector/pool layer under reqwest. — [crates.io reqwest 0.13.5 features](https://crates.io/api/v1/crates/reqwest/0.13.5)

### Inferences
- Using hyper/hyper-util directly would mean bringing up a tokio runtime (hyper-util's legacy `Client` uses a Tokio executor and connector) and hand-assembling TLS, decompression, cookies, redirects and proxy. That is a lot of work for an app making a handful of JSON calls. (The tokio dependency of hyper-util's client comes from general knowledge of hyper-util's API, not re-verified for 0.1.21.)
- `reqwest::blocking` inside gpui background tasks is safe as long as those tasks are not themselves inside a tokio runtime. gpui's executor is not tokio, so it would work. The cost is one extra internal OS thread per blocking `Client`, plus the tokio dependency weight.
- isahc runs a libcurl multi-handle agent thread internally, so it is also usable without the app owning a runtime (from isahc's long-standing design; not re-verified in this session).
- surf depends on the async-std/http-rs ecosystem. Given its 2021 last release, it is not a fit.
- cyper is built on compio (completion-based I/O, IOCP on Windows) plus hyper. It is listed by nyquest as a community backend. It would mean adopting compio, which is niche. — [nyquest README](https://github.com/bdbai/nyquest/blob/main/README.md)

### Gaps
- I did not verify isahc 2.x's internal threading model or curl-rust's `Multi` API ergonomics for this version.
- I did not check whether gpui-kit 0.9.1 itself pulls in tokio (for example via a transitive dependency), which would change the cost calculus.

## Q3. TLS backends, defaults, and Windows (MSVC) build requirements

### Takeaway
ureq 3.5 defaults to rustls+**ring**. reqwest 0.13 defaults to rustls+**aws-lc-rs** with rustls-platform-verifier (Windows cert store). Both have opt-in native-tls, which is SChannel on Windows. attohttpc defaults to native-tls (SChannel). isahc 2 defaults to rustls inside a statically built libcurl. ring needs only MSVC Build Tools. aws-lc-rs officially lists "C/C++ Compiler & NASM" for x86_64-pc-windows-msvc, but NASM can be avoided via prebuilt objects. In my build, reqwest's default compiled with no NASM on PATH (CMake was installed).

### Cited Findings
- ureq 3.5.2 features: `default = ['rustls','gzip']`; `rustls = ['rustls-no-provider','_ring','rustls-webpki-roots']`; `native-tls`; `native-tls-no-default`; `native-tls-webpki-roots`; `platform-verifier = ['dep:rustls-platform-verifier']`; `vendored = ['native-tls?/vendored']`; `win-system-proxy = ['dep:winreg']`; `cookies = ['dep:cookie_store','_url']`; `brotli`; `gzip`; `charset`; `json`; `multipart`; `socks-proxy`. — [crates.io ureq 3.5.2](https://crates.io/api/v1/crates/ureq/3.5.2)
- ureq: "By default, ureq uses rustls crate with the ring cryptographic provider. As of Sep 2024, the ring provider has a higher chance of compiling successfully." It says "ureq does not guarantee to default to ring indefinitely". native-tls "is never picked up as a default or used by the crate level convenience calls (ureq::get etc) – it must be configured on the agent" via `TlsConfig::builder().provider(TlsProvider::NativeTls)`. — [ureq README](https://github.com/algesten/ureq/blob/main/README.md)
- ureq platform verifier: enable the `platform-verifier` feature and `.root_certs(RootCerts::PlatformVerifier)`. "Setting RootCerts::PlatformVerifier together with TlsProvider::NativeTls means also native-tls will use the OS roots instead of webpki-roots". — [ureq README](https://github.com/algesten/ureq/blob/main/README.md)
- ureq rustls without ring requires `rustls-no-provider` plus `TlsConfigBuilder::unversioned_rustls_crypto_provider`. This is marked unstable/non-semver. — [ureq README](https://github.com/algesten/ureq/blob/main/README.md)
- reqwest 0.13.5: `default = ['default-tls','charset','http2','system-proxy']`; `default-tls = ['rustls']`; `rustls = ['__rustls-aws-lc-rs','dep:rustls-platform-verifier','__rustls']`; `rustls-no-provider` (bring your own provider, for example ring); `native-tls`, `native-tls-no-alpn`, `native-tls-vendored`. — [crates.io reqwest 0.13.5](https://crates.io/api/v1/crates/reqwest/0.13.5)
- aws-lc-rs Windows build requirements table: `x86_64-pc-windows-msvc` default = "C/C++ Compiler & *NASM" ("NASM is recommended on x86-64 but can be avoided using prebuilt NASM objects"). FIPS needs CMake, NASM, Go and Ninja. `aarch64-pc-windows-msvc` needs clang-cl. — [aws-lc-rs book: Windows requirements](https://github.com/aws/aws-lc-rs/blob/main/book/src/requirements/windows.md)
- ring: Windows needs "Build Tools for Visual Studio 2022" with the "Desktop development with C++" workflow. "Perl isn't required because the output of the Perl build steps is packaged into the crate. For Windows x86 and x86-64, the packaged crate contains precompiled object files for the assembly language modules so no macro assembler." ARM64 Windows needs the VS ARM64 build tools plus clang. — [ring BUILDING.md](https://github.com/briansmith/ring/blob/main/BUILDING.md)
- **Own measurement:** on this machine (`where nasm` → not found; CMake at `C:\Program Files\CMake\bin\cmake.exe`), `reqwest 0.13.5` with default TLS (aws-lc-rs) built successfully in release.
- attohttpc 0.32.0: `default = ['compress','tls-native']`. It also offers `tls-rustls-webpki-roots`, `tls-rustls-native-roots`, `-ring` variants and `tls-native-vendored`. — [crates.io attohttpc 0.32.0](https://crates.io/api/v1/crates/attohttpc/0.32.0)
- minreq 3.0.0: no TLS by default. Options are `https`/`https-rustls` (webpki-roots), `https-rustls-probe` (rustls-platform-verifier), `https-native-tls`, `https-openssl` (vendored) and `https-openssl-probe`. — [crates.io minreq 3.0.0](https://crates.io/api/v1/crates/minreq/3.0.0); [minreq README](https://github.com/neonmoe/minreq/blob/master/README.md)
- isahc 2.0.2: `default = ['default-tls','http2','log','static-curl','text-decoding']`; `default-tls = ['rustls-tls']`; `rustls-tls = ['static-curl','tls','curl/rustls','curl-sys/rustls']`; `native-tls = ['tls','curl/ssl','curl-sys/ssl']`; `native-tls-static`. The default trust store is the OS one (`TrustStore::native`). — [crates.io isahc 2.0.2](https://crates.io/api/v1/crates/isahc/2.0.2), [isahc 2.0.2 release notes](https://github.com/sagebind/isahc/releases)
- curl crate 0.4.51: `default = ['ssl']`; `static-curl`; `static-ssl`; `windows-static-ssl` (OpenSSL via vcpkg); `rustls`; `http2`; `ntlm`; `spnego`. In curl-sys build.rs, the bundled Windows build defines `USE_WINDOWS_SSPI` and `USE_SCHANNEL` and compiles `vtls/schannel.c` (SChannel). Without `static-curl` it probes for a system libcurl, including via vcpkg on MSVC. — [crates.io curl 0.4.51](https://crates.io/api/v1/crates/curl/0.4.51); [curl-sys build.rs](https://github.com/alexcrichton/curl-rust/blob/main/curl-sys/build.rs)
- Browser-impersonation note: none of the standard crates above advertise a browser-impersonation feature in their feature lists (impersonation crates are out of scope here). — feature lists above.

### Inferences
- native-tls on Windows maps to SChannel (OS-managed TLS and cert store, smallest binary). In my measurement, ureq with native-tls was about 0.7 MB smaller than with rustls+ring (see Q4).
- The ring vs aws-lc-rs difference on MSVC x64 is now small in practice: both build with only VS Build Tools (aws-lc-rs uses its prebuilt-NASM fallback). aws-lc-rs still costs more compile time and binary size (Q4). CI machines without CMake were not tested.
- Swapping TLS stacks later is easiest with ureq 3 (provider enum on `TlsConfig`, or a custom `Connector`; see Q5) and reqwest (`tls_backend_*` builder methods, `rustls-no-provider`).

### Gaps
- I did not test an aws-lc-rs build on a machine with neither CMake nor NASM. The aws-lc-rs docs do not list CMake for non-FIPS MSVC builds.
- I did not verify whether `native-tls` on Windows supports TLS 1.3 through SChannel in each crate. Only the reqwest 0.13.4 changelog mentions "TLS 1.3 support when using native-tls backend".

## Q4. Feature matrix (HTTP/2, pooling, cookies, proxy, compression, timeouts, redirects, header order) plus size, compile time and dependency counts

### Takeaway
reqwest is the only pure-Rust option here with HTTP/2, zstd and built-in Windows system proxy by default. ureq 3 covers everything this app needs (HTTP/1.1 pooling, cookie_store jar, gzip/brotli, env and Windows-registry proxy, fine-grained timeouts, raw header casing) at roughly half reqwest's binary size and under half its clean build time. attohttpc and minreq are smaller still but have no cookie jar.

### Cited Findings (features)
- **ureq 3.5**: HTTP/1.1 only. The transport module is documented as "HTTP/1.1 data transport". An `Agent` "holds a connection pool for reuse, and a cookie store if you use the cookies feature". Cookies use `cookie_store`. Compression is gzip (default) and brotli (opt-in); there is no zstd feature. Proxy: env vars `ALL_PROXY`, `HTTPS_PROXY`, `HTTP_PROXY` and `NO_PROXY` are read automatically, and SOCKS4/4a/5/5h is available via `socks-proxy`. Timeouts are budgeted per phase: `timeout_global`, `timeout_connect` (covers proxy, DNS, TLS handshake), `timeout_per_read` and `timeout_per_write`, among others. — [ureq README](https://github.com/algesten/ureq/blob/main/README.md), [ureq transport mod.rs](https://github.com/algesten/ureq/blob/main/src/unversioned/transport/mod.rs), [crates.io ureq 3.5.2](https://crates.io/api/v1/crates/ureq/3.5.2)
- **ureq Windows system proxy**: an undocumented-in-README `win-system-proxy` feature reads `HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings` (`ProxyEnable`, etc.) via `winreg`. — [ureq src/proxy.rs](https://github.com/algesten/ureq/blob/main/src/proxy.rs)
- **ureq header casing**: 3.5.0 added `RequestBuilder::header_raw` to "Preserve validated request header casing". — [ureq CHANGELOG](https://github.com/algesten/ureq/blob/main/CHANGELOG.md)
- **ureq redirects**: 3.4.2 fixed "network-path references in redirects", 3.2.0 fixed a "body header bug on redirect", and 3.4.0 added target TLS through HTTPS CONNECT proxies. — [ureq CHANGELOG](https://github.com/algesten/ureq/blob/main/CHANGELOG.md)
- **reqwest 0.13.5**: `http2` (default), `http3` (experimental, quinn), `cookies` (`cookie_store` + `cookie`), `gzip`, `brotli`, `deflate`, `zstd` (all via tower-http), `system-proxy` (default, hyper-util), `socks`, `hickory-dns`, `json`, `form`, `query`, `multipart`, `stream`. — [crates.io reqwest 0.13.5](https://crates.io/api/v1/crates/reqwest/0.13.5). 0.13.4 fixed redirect handling to strip sensitive headers when the scheme changes. — [reqwest CHANGELOG](https://github.com/seanmonstar/reqwest/blob/master/CHANGELOG.md)
- **isahc 2.0.2**: `http2` (default, libcurl/nghttp2), `cookies` (own implementation plus `psl` for public-suffix checks), `json`, `text-decoding`, `spnego`. — [crates.io isahc 2.0.2](https://crates.io/api/v1/crates/isahc/2.0.2)
- **attohttpc 0.32**: HTTP/1.1, automatic redirects, gzip/deflate, `HTTP_PROXY`/`HTTPS_PROXY`/`NO_PROXY`, Happy Eyeballs, json/form/multipart. No cookie feature is listed. — [attohttpc README](https://github.com/sbstp/attohttpc/blob/master/README.md), [crates.io attohttpc 0.32.0](https://crates.io/api/v1/crates/attohttpc/0.32.0)
- **minreq 3.0**: features are json, punycode, `proxy`, urlencoding and TLS variants. There is no cookie or compression feature. "Without any optional features ... about 148 KB additional executable size for stripped release builds". — [minreq README](https://github.com/neonmoe/minreq/blob/master/README.md), [crates.io minreq 3.0.0](https://crates.io/api/v1/crates/minreq/3.0.0)
- **curl crate**: `http2`, `ntlm`, `spnego`, `static-curl`, SChannel on Windows (see Q3). — [crates.io curl 0.4.51](https://crates.io/api/v1/crates/curl/0.4.51)
- **nyquest**: on Windows it delegates to WinHTTP or WinRT HttpClient, so it "honor[s] the user's system settings for HTTP client, such as proxy" ("Subject to the backend's capability"). — [nyquest README](https://github.com/bdbai/nyquest/blob/main/README.md)

### Cited Findings (size, compile time, dependencies): own measurement, 2026-11-12
Setup: one fresh Cargo project per configuration, each with its own `target/`, so every clean build compiles all of its dependencies. Release profile `opt-level="s"`, `lto=true`, `codegen-units=1`, `strip=true`, target x86_64-pc-windows-msvc, rustc 1.98.1, on this Windows 11 dev machine. Each `main` makes one GET with a custom User-Agent and reads the body to a string. Dependency count = unique crates in `cargo tree -e normal --target x86_64-pc-windows-msvc`, including the root package.

| Config | Unique normal deps | Clean release build (s) | Stripped .exe size | Live Yahoo v8 chart GET (UA "x") |
|---|---|---|---|---|
| ureq 3.5.2 default (rustls+ring, gzip) | 27 | 28 | 1,674,752 B (~1.60 MiB) | OK, 35,669 B body |
| ureq 3.5.2 native-tls (SChannel)+gzip+cookies+json | 78 | 17 | 965,120 B (~0.92 MiB) | OK |
| ureq 3.5.2 + cookies,json,brotli,platform-verifier,win-system-proxy | 85 | not built | — | — |
| reqwest 0.13.5 blocking+json+cookies+gzip+brotli (default rustls+aws-lc-rs, http2, system-proxy) | 129 | 67 | 3,534,848 B (~3.37 MiB) | OK |
| reqwest 0.13.5 blocking, native-tls, http2, system-proxy, cookies, gzip, json | 114 | 36 | 1,700,864 B (~1.62 MiB) | OK |
| attohttpc 0.32.0 default (native-tls/SChannel, compress)+json | 55 | 15 | 619,520 B (~0.59 MiB) | OK |
| minreq 3.0.0 https-rustls+json | 19 | 32 | 1,724,416 B (~1.64 MiB) | OK |
| isahc 2.0.2 default (static libcurl + rustls, http2), **no custom UA set** | 92 | 45 | 3,049,472 B (~2.91 MiB) | **Failed: 23-byte body** (consistent with Yahoo's `Edge: Too Many Requests`; see Q6) |
| curl 0.4.51 static-curl | 9 | not built | — | — |
| nyquest 0.5.0 blocking+json + nyquest-preset | 30 | not built | — | — |

- Older public benchmark: the OSSystems "web-client-server binary size benchmark" compared reqwest, surf, awc and hyper using cargo-bloat. It concluded "Don't fear micro-dependencies" (individual crates add little binary size). It was last pushed 2020-07-06, so it is **stale** and did not include ureq. — [OSSystems benchmark](https://github.com/OSSystems/web-client-server-binary-size-benchmark-rs)
- Anecdote: a trivial reqwest single-GET binary was "about 4.8 MB" (date and settings unclear, unverified). — [roc-lang Zulip export](https://roc-lang.github.io/zulip-export/stream/316715-contributing/topic/basic-cli.20reqwest.20lib.20size.html)

### Inferences
- Most of the size difference is the TLS crypto provider. Moving reqwest from aws-lc-rs to native-tls roughly halved its exe (3.37 to 1.62 MiB) and build time (67 s to 36 s). ureq native-tls vs rustls+ring saved about 0.7 MB.
- Rough cost of the HTTP layer alone: ureq+native-tls is about 1 MiB versus about 1.6 MiB for reqwest+native-tls. With rustls on both, about 1.6 MiB (ring) versus about 3.4 MiB (aws-lc-rs). These are absolute exe sizes, not deltas over a hello-world baseline, which I did not measure. minreq's own README number (148 KB) is for no-TLS builds.
- The app already depends on gpui-kit (a large graphics stack), so 1 to 2 MB of HTTP client difference is likely marginal in the final installer. Compile time and dependency count matter more for iteration speed.
- Cookies: only ureq (`cookies`), reqwest (`cookies`) and isahc (`cookies`) have built-in jars among the pure-HTTP crates. The Yahoo cookie+crumb flow (fc.yahoo.com, then getcrumb) would work with ureq's `Agent` and the `cookies` feature. Alternatively, it could be done manually by capturing `Set-Cookie` with any client.

### Gaps
- I did not measure a hello-world baseline exe, so the sizes above are absolute, not deltas.
- I did not verify header *order* control for any crate. ureq and reqwest both use `http::HeaderMap`, whose iteration order is not a strict insertion-order guarantee. Only casing preservation (ureq `header_raw`) is documented.
- I did not verify per-request timeout APIs in reqwest 0.13.5 docs (`RequestBuilder::timeout` and `ClientBuilder::timeout`/`connect_timeout` are long-standing, but not re-checked). I also did not check whether isahc/libcurl auto-decompresses gzip/brotli/zstd by default.
- Measurements come from one machine and one run. Build times will vary with cache and CPU.

## Q5. ureq 3.x specifics: cookies, native-tls, User-Agent, custom TLS config, custom Transport/Connector

### Takeaway
In ureq 3 you enable `cookies` (cookie_store jar per Agent), and `native-tls` plus `TlsProvider::NativeTls` on the agent config. User-Agent is set per request with `.header("User-Agent", ...)` (verified in my build). Alternatively, set it on the Agent config (config field exists but was not re-verified). ureq 3 exposes a pluggable, **non-semver** `unversioned::transport::Connector`/`Transport` trait pair via `Agent::with_parts()`. This is the hook for plugging in another TLS stack later, though it is limited to HTTP/1.1.

### Cited Findings
- Cookies: "An Agent ... holds a connection pool for reuse, and a cookie store if you use the cookies feature". `cookies = ['dep:cookie_store', '_url']`. 3.5.1 fix: "Send only cookie names and values in request Cookie headers". — [ureq README](https://github.com/algesten/ureq/blob/main/README.md), [crates.io ureq 3.5.2](https://crates.io/api/v1/crates/ureq/3.5.2), [ureq CHANGELOG](https://github.com/algesten/ureq/blob/main/CHANGELOG.md)
- In 3.x "TLS and cookie configuration is shimmed using our own types" (unlike 2.x, which re-exported non-1.0 crates). — [ureq README](https://github.com/algesten/ureq/blob/main/README.md)
- native-tls example: `Config::builder().tls_config(TlsConfig::builder().provider(TlsProvider::NativeTls).build()).build().new_agent()`. — [ureq README](https://github.com/algesten/ureq/blob/main/README.md). Own build: `ureq::Agent::config_builder().tls_config(TlsConfig::builder().provider(TlsProvider::NativeTls).build()).build().into()` compiled and fetched Yahoo successfully with ureq 3.5.2.
- Custom transport: "ureq provides a pluggable transport layer making it possible to write bespoke transports using the HTTP/1.1 protocol from point A to B. The `Agent::with_parts()` constructor takes an implementation of the `Connector` trait". `DefaultConnector` covers TCP, SOCKS, rustls TLS and native-tls TLS. The "Connector trait anticipates a chain of connectors" (`ChainedConnector`, `.chain(next)`). The traits are `pub trait Connector<In: Transport = ()>`, `pub trait Transport`, and `ShutdownHandle`. "transport does not (yet) follow semver". — [ureq src/unversioned/transport/mod.rs](https://github.com/algesten/ureq/blob/main/src/unversioned/transport/mod.rs)
- "ureq 3.x is trying out two new traits that had no equivalent in 2.x, Transport and Resolver ... They live under the unversioned module, and do not follow semver." — [ureq README](https://github.com/algesten/ureq/blob/main/README.md)
- Custom rustls crypto provider: `TlsConfigBuilder::unversioned_rustls_crypto_provider` (with `rustls-no-provider`). If the process installs a rustls default provider, "that choice is respected". — [ureq README](https://github.com/algesten/ureq/blob/main/README.md)
- Header casing: `RequestBuilder::header_raw` (3.5.0). — [ureq CHANGELOG](https://github.com/algesten/ureq/blob/main/CHANGELOG.md)
- Timeouts: the README table lists `timeout_connect` ("Proxy DNS, socket connection attempts, proxy negotiation, and TLS handshakes"), `timeout_per_read` and `timeout_per_write` ("subject to the remaining total budgets"), plus global/phase budgets. "Timeouts are checked during network I/O". — [ureq README](https://github.com/algesten/ureq/blob/main/README.md)
- `Agent::shutdown()` (3.5.0) "interrupt[s] requests in flight and close[s] all connections". This is useful for cancelling background fetches when a gpui window closes. — [ureq CHANGELOG](https://github.com/algesten/ureq/blob/main/CHANGELOG.md)
- The app's current 3.5.1 should move to 3.5.2 (2026-11-12): it applies body limits after decoding and tries the next address on NetworkDown. — [ureq CHANGELOG](https://github.com/algesten/ureq/blob/main/CHANGELOG.md)

### Inferences
- A custom `Connector` could wrap a different TLS implementation, for example a BoringSSL-based stack, for TLS-fingerprint control. ureq would still speak HTTP/1.1 only and control the HTTP header layout, so HTTP/2-fingerprint-based blocking could not be addressed this way.
- Because the Transport API is non-semver, a custom connector should pin ureq's minor version.

### Gaps
- I did not re-verify the exact Agent-level `user_agent` config field name in 3.5.2 docs.
- I did not check whether ureq sends a default `User-Agent: ureq/3.x` when none is set (it historically did).

## Q6. Reports of these clients being blocked or rate-limited by Yahoo Finance

### Takeaway
There are documented 2025 reports in the Rust `yahoo_finance_api` crate (reqwest-based) of persistent HTTP 429 "Too Many Requests" from Yahoo, including on `/v1/test/getcrumb`. These were tied to the **TLS backend**: native-tls/OpenSSL-style client hellos got 429s, and switching reqwest to **rustls** fixed it. yfinance (Python) moved to curl_cffi browser impersonation for the same wave. In my own single-shot test today, every standard client (ureq rustls, ureq native-tls/SChannel, reqwest aws-lc-rs and native-tls, attohttpc SChannel, minreq rustls) got a 200 with a ~35.7 KB v8 chart body using `User-Agent: x`. A `curl/8.21.0` User-Agent got `429 Edge: Too Many Requests` on the same URL, and isahc with its default UA failed the same way. So the User-Agent must be set explicitly, and it must not look like curl.

### Cited Findings
- yahoo_finance_api #74 (2025-05-04): "after May 1 I started getting 429s. I did try to change the user agent ... python yfinance package ... works." Comments link yfinance's switch to curl-impersonate ("Yahoo might be using TLS & HTTP/2 fingerprinting"). It was closed in favour of #76: "Using the rustls backend in the reqwest client builder ... fixed the issue for me." — [xemwebe/yahoo_finance_api#74](https://github.com/xemwebe/yahoo_finance_api/issues/74)
- #76 (2025-05-06): enabling reqwest's `default` feature (then native-tls) or `native-tls-alpn` made Yahoo return "Too many requests" (seen as `DeserializeFailed`). Adding `.use_rustls_tls()` "solves the issue ... behaves consistently regardless of additional reqwest features". — [xemwebe/yahoo_finance_api#76](https://github.com/xemwebe/yahoo_finance_api/issues/76)
- PR #78 (2025-05-16): "Always use rustls TLS backend". The PR also made the `Default` YahooConnector private because "the user agent is not set". — [xemwebe/yahoo_finance_api#78](https://github.com/xemwebe/yahoo_finance_api/pull/78)
- #79 (2025-06-09, still open): `getcrumb` returned `429 Too Many Requests, body: "Too Many Requests"` after obtaining an `A3` cookie from Yahoo. The fix was `reqwest` with `default-features = false` plus `rustls-tls`. One user found a second reqwest-using crate (ollama-rs) re-enabling default-tls, which brought the 429 back until that crate was also switched to rustls. Another user reported intermittent getcrumb failures when requests are made in quick succession. — [xemwebe/yahoo_finance_api#79](https://github.com/xemwebe/yahoo_finance_api/issues/79)
- #42 (2024-06-01): earlier 429 wave. The user wanted to set a User-Agent but "the client is forced to default". It was fixed in crate v2.2.0. — [xemwebe/yahoo_finance_api#42](https://github.com/xemwebe/yahoo_finance_api/issues/42)
- yahoo_finance_api 4.1.0 has distinct `TooManyRequests` and `InvalidCrumb` error variants. — [docs.rs yahoo_finance_api 4.1.0 yahoo_error.rs](https://docs.rs/crate/yahoo_finance_api/4.1.0/source/src/yahoo_error.rs)
- yfinance (Python) bug reports show the crumb request itself returning "Too Many Requests"/"Edge: Too Many Requests", followed by a 429 on the data request. — [ranaroussi/yfinance#2128](https://github.com/ranaroussi/yfinance/issues/2128), [ranaroussi/yfinance#2480](https://github.com/ranaroussi/yfinance/issues/2480) (via search-result summaries; not fully read)
- A third-party connector doc claims Yahoo sends no rate-limit headers (the only signal is 429 "Too Many Requests") and recommends staying under about 2 requests per second. — [digdigdig3 yahoo README](https://docs.rs/crate/digdigdig3/0.2.3/source/src/l1/free/yahoo/README.md) (secondary, unverified)
- **Own test, User-Agent sensitivity (2026-11-12, curl CLI, same URL):** `User-Agent: curl/8.21.0` returned **HTTP 429 with body `Edge: Too Many Requests`** (23 bytes). `User-Agent: isahc/2.0.2` returned 200 and `User-Agent: x` returned 200. The isahc test binary (no UA override, so isahc's default UA was sent) also got a 23-byte body, which matches the 429 edge response. This suggests Yahoo's edge rejects UAs containing `curl`. isahc's default UA is believed to include the libcurl version (not verified in source). So **libcurl-based clients (isahc, curl crate) must override User-Agent** for Yahoo.
- **Own test (2026-11-12, this machine/IP, one request each):** `GET https://query1.finance.yahoo.com/v8/finance/chart/AAPL` with `User-Agent: x` succeeded for ureq 3.5.2 (rustls+ring), ureq 3.5.2 (native-tls/SChannel), reqwest 0.13.5 (rustls+aws-lc-rs), reqwest 0.13.5 (native-tls/SChannel), attohttpc 0.32 (SChannel) and minreq 3.0 (rustls). Each returned a body of 35,669–35,671 bytes.

### Inferences
- The 2025 reports were most likely Linux/OpenSSL native-tls users (OS not stated in the issues). SChannel's fingerprint differs from OpenSSL's, and my SChannel tests succeeded today. Still, Yahoo's fingerprint policy changes over time, so the TLS backend should stay swappable.
- The getcrumb endpoint appears more sensitive than v8 chart. The v8 chart endpoint worked without cookie/crumb today. Building the cookie+crumb flow only as a fallback, with spacing between the fc.yahoo.com, getcrumb and data calls, matches the reports.
- Because Cargo features are additive, any second crate in the tree that enables `reqwest/default-tls` or `native-tls` can silently change the TLS backend (#79). ureq avoids this by never auto-selecting native-tls from features alone ([ureq README](https://github.com/algesten/ureq/blob/main/README.md)).

### Gaps
- No load or repeat test: my Yahoo check was one request per client from one residential IP. It says nothing about thresholds.
- I did not verify isahc's exact default User-Agent string in its source.
- I found no reports specific to ureq, attohttpc or minreq being blocked by Yahoo. No GitHub search of those repos was done for "yahoo".
- Not researched: Nasdaq.com UA-dependent hangs, İş Yatırım, or TradingView scanner behaviour per client.
