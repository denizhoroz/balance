# Rust browser-impersonation HTTP clients (TLS JA3/JA4 + HTTP/2 Akamai fingerprint), Windows x86_64 MSVC, as of 2026-11-12

Method note: Version, date, license and dependency facts come from the crates.io API (`https://crates.io/api/v1/crates/<name>`), the GitHub REST API (`https://api.github.com/repos/<owner>/<repo>`), and raw READMEs and manifests, all queried on 2026-11-12. Star counts are a snapshot from that date. I did not compile anything; build findings come from CI files, manifests and docs. Fingerprint accuracy is outside this scope (another researcher is measuring it).

## Q1. Which candidates exist, and how current and maintained are they (version, date, license, stars, bus factor)?

### Takeaway
Only three Rust-native options are mature and active as of November 2026: **wreq + wreq-util** (BoringSSL, the largest profile set, Apache-2.0 since mid-2026), **impit** (Apify, patched rustls, but not published as a usable crate), and **primp 2.x** (rustls fork, a native crate since August 2026). A fourth route is the **lexiforest curl-impersonate** C library over FFI. It has excellent prebuilt Windows MSVC binaries, but there is no maintained Rust binding. Everything else is either stale (craftls, reqwest-impersonate, rquest, lwthiker curl-impersonate) or tiny.

### Cited Findings
**wreq (0x676e67): successor of rquest and reqwest-impersonate**
- Latest stable is **wreq 0.16.3**, published 2026-10-30 (0.16.2 was yanked the same day). The **6.0.0-rc.x** prerelease line also exists; its last release is 6.0.0-rc.31 (2026-08-18). The stable 0.15.x/0.16.x line started on 2026-08-22, so the project apparently restarted its version numbers when it left the RC series. The license is Apache-2.0 and total downloads are about 2.5M. — [crates.io wreq](https://crates.io/api/v1/crates/wreq)
- A "chore: release v0.17.0" PR was open on 2026-10-28, so v0.17 is imminent. — [wreq PR #1356](https://github.com/0x676e67/wreq/pull/1356)
- GitHub: 1,106 stars, last push 2026-11-12, 29 open issues/PRs, Apache-2.0, not archived. — [GitHub API 0x676e67/wreq](https://api.github.com/repos/0x676e67/wreq)
- wreq describes itself as a hard fork of reqwest. It is BoringSSL-based and offers "customizable TLS, JA3/JA4, and HTTP/2 signature capabilities". Its sponsors include Hyper Solutions (antibot tokens for Akamai, DataDome, Kasada and Incapsula), Captcha.fun and Byteful (proxies). The maintainer offers commercial support. — [wreq README](https://raw.githubusercontent.com/0x676e67/wreq/main/README.md)
- The predecessors are dead. `rquest`'s last release is 5.2.0 (2025-07-11), and the crate now shows max_version 0.0.0, which looks like a tombstone or yank. `reqwest-impersonate`'s last release is 0.11.91 (2024-07-25), and it also shows 0.0.0. — [crates.io rquest](https://crates.io/api/v1/crates/rquest), [crates.io reqwest-impersonate](https://crates.io/api/v1/crates/reqwest-impersonate)
- TLS comes from **btls**, wreq's own fork of Cloudflare's boring (btls/btls-sys/tokio-btls 0.5.6, 2026-04-20, Apache-2.0, 59 stars, last push 2026-11-12). wreq 0.16.3 depends on btls ^0.5.6, tokio-btls, and its own `http2` fork (^0.5.17), plus `wreq-proto` and `wreq-rt`. — [crates.io wreq 0.16.3 deps](https://crates.io/api/v1/crates/wreq/0.16.3/dependencies), [btls README](https://raw.githubusercontent.com/0x676e67/btls/main/README.md), [GitHub API btls](https://api.github.com/repos/0x676e67/btls)

**wreq-util: the browser emulation profiles**
- Latest stable is **0.2.1** (2026-11-10). Prerelease line: 3.0.0-rc.14 (2026-07-08). It depends on wreq ^0.16.0. 102 stars, last push 2026-11-10. — [crates.io wreq-util](https://crates.io/api/v1/crates/wreq-util), [GitHub API wreq-util](https://api.github.com/repos/0x676e67/wreq-util)
- **The license changed over time, so watch which version you pin:**

  | wreq-util version(s) | License |
  |---|---|
  | 3.0.0-rc.7 and rc.8 (Nov–Dec 2025) | GPL-3.0 |
  | rc.9 to rc.11 (Dec 2025 – May 2026) | LGPL-3.0 |
  | rc.12 to rc.14 | Apache-2.0 |
  | 0.1.0 (2026-08-22) | GPL-3.0 |
  | 0.2.0 and 0.2.1 | Apache-2.0 |

  **Pin wreq-util ≥ 0.2.0 (or ≥ 3.0.0-rc.12) to stay Apache-2.0.** — [crates.io wreq-util versions](https://crates.io/api/v1/crates/wreq-util/versions?per_page=100)
- Both the README and Cargo metadata now say Apache-2.0. — [wreq-util README](https://raw.githubusercontent.com/0x676e67/wreq-util/main/README.md)

**impit (Apify)**
- GitHub: 639 stars, last push 2026-11-12, Apache-2.0 (GitHub license field), 7 open issues. It is backed by a company (Apify), not a single individual. — [GitHub API apify/impit](https://api.github.com/repos/apify/impit)
- **The Rust crate is not consumable from crates.io.** The repo's `impit/Cargo.toml` still says `version = "0.1.0"`, and the README tells Rust users to use a git dependency on `master`. The crates.io name `impit` has only one release, 0.1.0 (2025-01-10, MIT, no repository link, about 1.8k downloads). I could not confirm that this is Apify's crate. — [impit Cargo.toml](https://raw.githubusercontent.com/apify/impit/master/impit/Cargo.toml), [impit README](https://raw.githubusercontent.com/apify/impit/master/README.md), [crates.io impit](https://crates.io/api/v1/crates/impit)
- Apify does ship actively from the same core: the Python `impit` 0.15.3 was uploaded 2026-11-11, with win_amd64 wheels for CPython 3.10–3.15. A Node package also exists. — [PyPI impit JSON](https://pypi.org/pypi/impit/json)

**primp (deedy5)**
- **primp 2.0.1** (2026-09-12, MIT). Version 2.0.0 (2026-08-26) introduced a "native `primp` crate (workspace restructure)"; before that, primp was mainly a Python package. 627 stars, last push 2026-09-13, 6 open issues. — [crates.io primp](https://crates.io/api/v1/crates/primp), [primp PR #160](https://github.com/deedy5/primp/pull/160), [GitHub API deedy5/primp](https://api.github.com/repos/deedy5/primp)
- It is built on its own forks, `primp-rustls` 0.23.43 (default features include `aws_lc_rs` and `impersonate`) and `primp-h2`, on top of hyper 1.8 and tokio. — [crates.io primp 2.0.1 deps](https://crates.io/api/v1/crates/primp/2.0.1/dependencies), [crates.io primp-rustls](https://crates.io/api/v1/crates/primp-rustls)

**curl-impersonate (C library) and curl_cffi**
- The **lexiforest fork is the live one**: v2.3.2 (2026-10-30), v2.3.1 (2026-10-25, a security release: brotli "http2shell" bug, and BoringSSL now honours the Windows revoked-cert list), and v2.3.0 (2026-10-20, curl 8.23, "9 new fingerprints"). It has 2,871 stars, MIT license, and was last pushed 2026-10-29. — [lexiforest/curl-impersonate releases API](https://api.github.com/repos/lexiforest/curl-impersonate/releases?per_page=3), [GitHub API lexiforest/curl-impersonate](https://api.github.com/repos/lexiforest/curl-impersonate)
- The original **lwthiker/curl-impersonate is stale**: last push 2024-07-18, 82 open issues. — [GitHub API lwthiker/curl-impersonate](https://api.github.com/repos/lwthiker/curl-impersonate)
- curl_cffi 0.17.0 (PyPI, 2026-10-26) ships win_amd64 and win_arm64 wheels. The repo has 6,852 stars and was last pushed 2026-10-27. — [PyPI curl-cffi JSON](https://pypi.org/pypi/curl-cffi/json), [GitHub API curl_cffi](https://api.github.com/repos/lexiforest/curl_cffi)
- The only Rust wrappers are tiny:

  | Crate | Latest | Downloads | What it is |
  |---|---|---|---|
  | `impersonate-rs` | 0.1.3 (2026-01-27) | 872 | wrapper |
  | `cuimp` | 0.1.2 (2025-11-15) | 1,969 | wrapper |
  | `curl-impersonate-cli` | 0.4.0 (2026-08-06) | 305 | runs the curl_chromeNNN binaries as subprocesses |

  No `curl-impersonate-sys` crate exists on crates.io (lookup returned not found). — [crates.io search curl-impersonate](https://crates.io/api/v1/crates?q=curl-impersonate&per_page=10)

**boring (Cloudflare): the do-it-yourself route**
- boring, boring-sys, tokio-boring and hyper-boring **5.5.0** were all published 2026-11-12. Licenses: boring is Apache-2.0, boring-sys is MIT, and tokio-boring and hyper-boring are MIT OR Apache-2.0. The repo has 532 stars, was pushed 2026-11-12, and has 43 open issues. — [crates.io boring](https://crates.io/api/v1/crates/boring), [GitHub API cloudflare/boring](https://api.github.com/repos/cloudflare/boring)

**rustls-based ClientHello customization**
- **craftls** ("Rustls with customizable ClientHello fingerprint") is stale. Its last release is 0.0.2+rustls.0.22.0 (2024-01-16), its last push was 2024-01-19, and it has 26 stars. — [crates.io craftls](https://crates.io/api/v1/crates/craftls), [GitHub API 3andne/craftls](https://api.github.com/repos/3andne/craftls)
- `ja3-rustls` 0.0.5 dates from 2022. No crate named `rustls-fingerprint` exists on crates.io (lookup returned not found). — [crates.io search ja3](https://crates.io/api/v1/crates?q=ja3&per_page=15)
- The live rustls-fork approaches are impit's `apify/rustls` git fork and primp's `primp-rustls` (above).

**Others found**
- **rama** (plabayo): a modular proxy/service framework, 1,260 stars, v0.4.0 (2026-08-19). It includes `rama-tls-boring` and `rama-ua` (user-agent emulation) crates. It is a framework, not a drop-in client. — [crates.io search tls fingerprint](https://crates.io/api/v1/crates?q=tls%20fingerprint&per_page=15), [GitHub API plabayo/rama](https://api.github.com/repos/plabayo/rama)
- **hpx / hpx-emulation** (longcipher): 2.6.1 (2026-11-07), 12 stars, about 8k downloads. Too small to rely on. — [crates.io search ja3](https://crates.io/api/v1/crates?q=ja3&per_page=15), [GitHub API longcipher/hpx](https://api.github.com/repos/longcipher/hpx)
- `turbo-surf-wreq` / `turbo-surf-wreq-util` (0.16.1/0.2.0, 2026-10-21) appear to be republished copies of wreq. — [crates.io search ja3](https://crates.io/api/v1/crates?q=ja3&per_page=15)
- **bogdanfinn/tls-client** (Go; usable through its C shared library): 1,940 stars, BSD-3-Clause, pushed 2026-11-11. Deprioritized because it is not Rust. — [GitHub API bogdanfinn/tls-client](https://api.github.com/repos/bogdanfinn/tls-client)

### Inferences
- wreq, wreq-util and btls all come from the same maintainer (0x676e67). Commits are frequent, but the bus factor is effectively one person. The project's business ties (antibot and proxy sponsors) suggest it will keep tracking browsers.
- The wreq-util license history (GPL → LGPL → Apache) is the biggest hidden risk. A `Cargo.lock` that resolves to 0.1.0 or an old RC would pull in GPL-3.0 code.
- impit's maturity mostly shows up in its Python and Node products. Its Rust API is a secondary, unpublished surface.

### Gaps
- Exact contributor counts per repo: I did not query the contributors API.
- Whether crates.io `impit` 0.1.0 belongs to Apify.

## Q2. Which browser profiles are supported, how current are they, and how often are they updated?

### Takeaway
wreq-util has the widest and most current catalog. impit and primp are close behind on Chrome versions, but impit has no Edge profiles. curl-impersonate/curl_cffi is well curated, but its open-source catalog lags (Edge stops at edge101, Chrome at 152). All of them can follow a new Chrome release within weeks.

### Cited Findings
- **wreq-util (main branch source):**

  | Browser | Profiles |
  |---|---|
  | Chrome | Chrome100 to **Chrome154** (with gaps) |
  | Edge | Edge101 to **Edge148** |
  | Firefox | Firefox109 to **Firefox152**, plus FirefoxAndroid135 and FirefoxPrivate135/136 |
  | Safari | Safari15_3 to **Safari26_4**, plus iOS and iPad variants |
  | Opera | Opera116 to **Opera131** |
  | OkHttp | 3.9 to 5 |

  — [wreq-util src/emulate.rs](https://raw.githubusercontent.com/0x676e67/wreq-util/main/src/emulate.rs)
- The README example uses `Emulation::Chrome147`. It also notes that Chrome 150+ profiles advertise ML-DSA signature algorithms, and that Chrome 152+ profiles add signature-algorithm GREASE and an empty `trust_anchors` extension. An optional `emulation-chromium-pki` feature sends Chromium Trust Anchor IDs. The wreq README claims "100+ browser device emulation profiles". — [wreq-util README](https://raw.githubusercontent.com/0x676e67/wreq-util/main/README.md), [wreq README](https://raw.githubusercontent.com/0x676e67/wreq/main/README.md)
- **impit (master):**

  | Browser | Profiles |
  |---|---|
  | Chrome | chrome_100 to **chrome_157** (100, 101, 104, 107, 110, 116, 124, 125, 131, 133, 136, 142, 151, 155, 156, 157) |
  | Firefox | 128, 133, 135, 144, **158** |
  | Safari | ios_18 |
  | OkHttp | 3, 4, 5 |
  | Edge | none |

  The firefox158 fingerprint landed 2026-10-19. — [impit fingerprint database](https://github.com/apify/impit/tree/master/impit/src/fingerprint/database), [impit PR #587](https://github.com/apify/impit/pull/587)
- **primp:**

  | Browser | Profiles |
  |---|---|
  | Chrome | chrome_144 to **153** |
  | Edge | edge_144 to **153** |
  | Firefox | firefox_140 and 146 to **151** |
  | Safari | 18.5, 26, 26.3, 26.4 |

  An OS axis (android, ios, linux, macos, windows, random) is also exposed. — [primp README](https://raw.githubusercontent.com/deedy5/primp/main/README.md)
- **curl_cffi / curl-impersonate (lexiforest), open-source targets:**

  | Browser | Targets |
  |---|---|
  | Chrome | chrome99 to **chrome152**, with separate `_macos` variants for 151/152 |
  | Edge | edge99, **edge101** only |
  | Firefox | up to **firefox155** |
  | Safari | up to 26/27 |
  | Tor | tor145 |

  The `chrome` alias "selects the Windows profile" and always resolves to the latest. Commercial impersonate.pro offers a "weekly updated list of browser profiles". — [curl_cffi supported targets](https://curl-cffi.readthedocs.io/en/latest/impersonate/targets.html)
- curl_cffi docs recommend using unversioned aliases (`chrome`, `safari`) so you always get the newest profile. yfinance uses `impersonate="chrome"`. — [curl_cffi FAQ](https://curl-cffi.readthedocs.io/en/latest/faq.html), [yfinance issue #2456](https://github.com/ranaroussi/yfinance/issues/2456)

### Inferences
- Update cadence, judged from release history: wreq-util cut 0.2.0 (2026-08-22) and 0.2.1 (2026-11-10); the RC line ran about every 2–6 weeks. impit adds fingerprints in PRs roughly monthly. curl-impersonate shipped three releases in October 2026. primp has had no release since 2026-09-12.
- impit's highest Chrome number (157) exceeds wreq-util's (154) and primp's (153). The projects may label versions differently, so the fingerprint-measurement researcher should check what is actually emitted rather than compare labels.

### Gaps
- Which Chrome version is current stable in November 2026: not verified here.
- Whether wreq-util's highest profiles have been released to crates.io 0.2.1 or exist only on main: I read the main-branch source.

## Q3. API shape: async or blocking, which runtime, and features (cookies, proxy, compression, timeouts, HTTP/2)?

### Takeaway
All three Rust-native clients are **async and need tokio**. wreq can alternatively run on compio. None documents a blocking API. Since the app has no tokio runtime, it would need a dedicated background thread running a small tokio current-thread runtime that `block_on`s each request. The curl-impersonate easy API is natively blocking and would slot directly into the existing ureq-style background tasks.

### Cited Findings
- **wreq:**
  - Runtimes are "Multiple Runtime (Tokio, Compio)". Default features are `webpki-roots` and `tokio-rt`; `compio-rt` is optional. The README mentions no blocking API.
  - Features listed: cookie store, redirect policy, header-case preservation ("Original Header"), rotating proxies, Tower middleware, WebSocket, "HTTP/2 over TLS Parity", and a certificate store with mTLS.
  - Cargo features: `cookies`, `gzip`, `brotli`, `zstd`, `deflate`, `socks`, `system-proxy`, `hickory-dns`, `json`, `multipart`, `stream`, `ws`, `prefix-symbols`.
  - Optional deps include `windows-registry` and `system-configuration`, which are used for system proxy settings.
  - Supports HTTP/1.1 and HTTP/2 only (no HTTP/3), per a downstream adapter's docs.

  — [wreq README](https://raw.githubusercontent.com/0x676e67/wreq/main/README.md), [crates.io wreq 0.16.3](https://crates.io/api/v1/crates/wreq/0.16.3), [wreq PR #1175 (compio runtime)](https://github.com/0x676e67/wreq/pull/1175), [hex.pm cloaked_req](https://readme.hex.pm/cloaked_req/0.4.0)
- **wreq MSRV is 1.98, edition 2024**, with a rolling MSRV policy (current and previous compiler). — [wreq README](https://raw.githubusercontent.com/0x676e67/wreq/main/README.md)
- **impit:**
  - Built "on top of `reqwest`, `rustls` and `tokio`", with an async builder API (`Impit::<Jar>::builder().with_fingerprint(..)`). It supports custom headers, proxies, timeouts, a cookie `Jar`, HTTP/1.1, HTTP/2, and HTTP/3 behind a feature that needs `--cfg reqwest_unstable`.
  - Its reqwest dependency enables json, gzip, brotli, zstd, deflate, cookies, stream and socks.
  - It patches `h2`, `rustls`, `tower-http`, `hyper-util` and `reqwest` from git forks via `[patch.crates-io]`.

  — [impit README](https://raw.githubusercontent.com/apify/impit/master/README.md), [impit Cargo.toml](https://raw.githubusercontent.com/apify/impit/master/impit/Cargo.toml), [impit workspace Cargo.toml](https://raw.githubusercontent.com/apify/impit/master/Cargo.toml)
- **primp:**
  - The example uses `#[tokio::main]` and `.impersonate(Impersonate::ChromeV146)`. It depends on tokio, hyper 1.8 and tower.
  - Optional features: cookie and cookie_store, hickory-resolver, HTTP/3 (h3/quinn), serde_json and others.
  - I found no blocking API in the README.

  — [primp README](https://raw.githubusercontent.com/deedy5/primp/main/README.md), [crates.io primp 2.0.1 deps](https://crates.io/api/v1/crates/primp/2.0.1/dependencies)
- **curl-impersonate:** the Windows libcurl-impersonate package ships the standard curl headers (`easy.h`, `multi.h`, …) plus `curl-impersonate/webcrypto.h`. It links nghttp2, nghttp3/ngtcp2 (HTTP/3), brotli, zstd and c-ares. — [libcurl-impersonate v2.3.2 x86_64-win32 archive](https://github.com/lexiforest/curl-impersonate/releases/download/v2.3.2/libcurl-impersonate-v2.3.2.x86_64-win32.tar.gz) (contents listed locally with `tar tzv`)

### Inferences
- **impit's `[patch.crates-io]` is a practical trap.** Cargo applies patches only from the root workspace, so a consuming app must copy them. That patch would also swap the app's own `rustls` 0.23 (used by ureq 3.x) and `hyper-util` for Apify's forks across the whole build.
- Integrating wreq or primp means adding tokio (rt + net + time) to a currently tokio-free binary. A single `tokio::runtime::Builder::new_current_thread()` per background worker is the usual bridge. This is an inference; none of the READMEs document a blocking API.
- With the curl route, `curl_easy_perform` is synchronous, matching the app's current blocking model. libcurl also brings its own cookie engine, proxies, timeouts and HTTP/2.

### Gaps
- Whether primp 2.x exposes a `blocking` module: only README text was checked.
- Exact timeout APIs per crate: not confirmed beyond the README mentions.

## Q4. What does each need to build on Windows x86_64 MSVC, and are there prebuilt options and known issues?

### Takeaway
The BoringSSL-based routes (wreq/btls, boring) need a full native toolchain on Windows: **CMake ≥ 3.22, NASM, MSVC (VS 2022+), and libclang for bindgen**. wreq's CI proves they build on `windows-latest`. The rustls-fork routes (impit, primp) avoid BoringSSL, but default to aws-lc-rs (primp). The curl-impersonate route is the only one with **official prebuilt MSVC-compatible Windows binaries**, but you must write the Rust binding yourself.

### Cited Findings
- **BoringSSL's own prerequisites:**
  - CMake 3.22+, with Ninja recommended. The Visual Studio generator "may also work, but it not tested regularly".
  - "On Windows only, NASM is required" (it can be set via `CMAKE_ASM_NASM_COMPILER`).
  - MSVC from Visual Studio 2022+ with Windows 10 SDK 2104+.
  - It also supports a shared-library build (`BORINGSSL_SHARED_LIBRARY` on Windows).

  The extract I read listed no Perl or Go requirement. — [BoringSSL BUILDING.md](https://boringssl.googlesource.com/boringssl/+/HEAD/BUILDING.md)
- **btls-sys 0.5.6 and boring-sys 5.5.0 build-dependencies:** `bindgen ^0.72`, `cmake ^0.1.5x`, `fs_extra`, `fslock`. bindgen requires libclang at build time. — [crates.io btls-sys deps](https://crates.io/api/v1/crates/btls-sys/0.5.6/dependencies), [crates.io boring-sys deps](https://crates.io/api/v1/crates/boring-sys/5.5.0/dependencies)
- boring-sys "builds the BoringSSL library (or optionally links a pre-built version)". It has a FIPS path via `BORING_BSSL_FIPS_PATH`. The boring README changelog notes that `fips-precompiled` merged into `fips`. — [boring-sys README](https://raw.githubusercontent.com/cloudflare/boring/master/boring-sys/README.md), [boring README](https://raw.githubusercontent.com/cloudflare/boring/master/README.md)
- **wreq on Windows:**
  - The README gives only a Linux apt list (`build-essential cmake perl pkg-config libclang-dev musl-tools git`) and points to CI for Windows.
  - wreq's CI matrix includes `windows-latest` with a step "Install NASM (Windows): `choco install nasm -y`". No other Windows-specific step showed up in a grep for nasm, cmake, llvm or clang; the GitHub runner image already ships CMake and LLVM.

  — [wreq README](https://raw.githubusercontent.com/0x676e67/wreq/main/README.md), [wreq ci.yml](https://raw.githubusercontent.com/0x676e67/wreq/main/.github/workflows/ci.yml)
- **Known boring Windows issues:**
  - #71 "Failed to cross-compile from Linux to Windows" (open since 2022), not relevant for native builds.
  - #283, a boring-sys build failure under WSL2 (2024).
  - PR #450 "Cache Windows builds harder" (merged Jan 2026) and PR #529 (Oct 2026) touched Windows CI.

  A GitHub search of wreq issues and PRs for "windows" found no open Windows build bugs, only dependency bumps. — [GitHub issue search cloudflare/boring windows](https://api.github.com/search/issues?q=repo:cloudflare/boring+windows), [GitHub issue search wreq windows](https://api.github.com/search/issues?q=repo:0x676e67/wreq+windows)
- **curl-impersonate prebuilt Windows binaries:**
  - Release v2.3.2 ships `libcurl-impersonate-v2.3.2.x86_64-win32.tar.gz` (7.1 MB), plus i686 and arm64 builds and matching CLI archives.
  - The archive contains MSVC-format `libcurl-impersonate.dll` (4.1 MB), the import library `libcurl-impersonate_imp.lib`, the static `libcurl-impersonate.lib` (3.1 MB), and static BoringSSL `crypto.lib`/`ssl.lib` plus nghttp2, brotli, zstd, c-ares and ngtcp2 `.lib` files.
  - It also contains a `libcurl-impersonate.pc` and license files (MIT, BoringSSL, curl, …).
  - The release notes mention "Use vswhere to detect Visual Studio", so the build uses MSVC.

  — [lexiforest releases API](https://api.github.com/repos/lexiforest/curl-impersonate/releases?per_page=3), [x86_64-win32 archive](https://github.com/lexiforest/curl-impersonate/releases/download/v2.3.2/libcurl-impersonate-v2.3.2.x86_64-win32.tar.gz)
- **The `curl` crate cannot simply point at that library.** `curl-sys` (0.4.91, bundling curl 8.22.0) either probes vcpkg or pkg-config for a system libcurl, or builds its vendored copy with `cc`. — [curl-sys build.rs](https://raw.githubusercontent.com/sagebind/curl-rust/main/curl-sys/build.rs), [crates.io curl-sys](https://crates.io/api/v1/crates/curl-sys)
- **impit and primp on Windows:**
  - impit's Python wheels are built for win_amd64, so its Rust core compiles for Windows under Apify's CI.
  - primp-rustls enables `aws_lc_rs` by default; `ring` is an alternative optional feature.
  - primp issue #80 "TLS not matching chrome windows" (Feb 2025, closed) concerned fingerprint parity, not the build.

  — [PyPI impit JSON](https://pypi.org/pypi/impit/json), [crates.io primp-rustls](https://crates.io/api/v1/crates/primp-rustls), [primp issue #80](https://github.com/deedy5/primp/issues/80)

### Inferences
- **To use the curl route from Rust on MSVC**, you would:
  - write your own `-sys` crate or `build.rs` that links `libcurl-impersonate_imp.lib` (DLL) or the static `.lib`s;
  - declare the impersonate-specific symbol (`curl_easy_impersonate`) yourself;
  - or use `curl-sys` with renamed libraries, which is fragile.

  Shipping the DLL beside the exe is the simplest option. The curl crate's safe `Easy` API could in principle be reused if `curl-sys` is made to link the impersonate library under the name `libcurl`. This is unverified.
- **The local developer machine needs:** CMake (it is on PATH: `C:/Program Files/CMake/bin` showed up in the shell PATH), plus NASM and LLVM/libclang (`LIBCLANG_PATH`) for wreq or boring. Neither NASM nor LLVM was confirmed installed.
- primp's default aws-lc-rs and impit's patched rustls (its crypto provider is unverified) may pull in `aws-lc-sys`, which has its own CMake/NASM-style native build on Windows. This is unverified for current aws-lc-sys versions.

### Gaps
- Real compile-time and binary-size numbers for each option: nothing measured, and none of the projects publish them.
- Whether btls-sys supports linking a prebuilt BoringSSL (like boring-sys's "pre-built version" option) on Windows: not checked.
- Which crypto provider impit's rustls fork uses.

## Q5. Will they conflict with OpenSSL, ring or aws-lc in the same binary?

### Takeaway
BoringSSL exports OpenSSL-named symbols. wreq warns that linking alongside **openssl-sys** can cause link failures, and its `prefix-symbols` fix is documented only for Linux and Android. On Windows MSVC, keep openssl-sys out of the dependency tree. ring and aws-lc are not the same problem, because both prefix their symbols. That last point is an inference.

### Cited Findings
- wreq: "Compiling alongside openssl-sys can cause symbol conflicts with boringssl that lead to link failures, and on Linux and Android this can be avoided by enabling the `prefix-symbols` feature." — [wreq README](https://raw.githubusercontent.com/0x676e67/wreq/main/README.md), [cloudflare/boring issue #197](https://github.com/cloudflare/boring/issues/197)
- btls-sys has a `prefix-symbols` feature; boring-sys 5.5.0 does not. Upstream boring's "Add symbol prefixing feature for BoringSSL" PR #401 has been open since 2025-10-14. — [crates.io btls-sys 0.5.6](https://crates.io/api/v1/crates/btls-sys/0.5.6), [crates.io boring-sys 5.5.0](https://crates.io/api/v1/crates/boring-sys/5.5.0), [boring PR #401](https://github.com/cloudflare/boring/pull/401)
- The libcurl-impersonate Windows package statically bundles its own BoringSSL (`crypto.lib`, `ssl.lib`) inside the DLL build. — [x86_64-win32 archive](https://github.com/lexiforest/curl-impersonate/releases/download/v2.3.2/libcurl-impersonate-v2.3.2.x86_64-win32.tar.gz)

### Inferences
- Using the **DLL** form of libcurl-impersonate isolates BoringSSL's symbols behind the DLL boundary on Windows. That makes it the safest option if the app ever links OpenSSL or schannel-based native-tls.
- ureq 3.x with rustls uses ring or aws-lc-rs, and both use prefixed symbol names. So wreq/btls should coexist with the existing ureq in one binary. Two TLS stacks also add size.

### Gaps
- No documented Windows-specific symbol-clash reports for wreq + openssl-sys were found, in either direction.

## Q6. Has anyone used these successfully against Yahoo Finance, Cloudflare or Akamai?

### Takeaway
I found **no public report** of wreq, impit, primp or a curl-impersonate Rust binding being used specifically against Yahoo Finance's v8 chart endpoint. The established Yahoo precedent is Python yfinance with curl_cffi `impersonate="chrome"`.

### Cited Findings
- yfinance creates its session with curl_cffi `impersonate="chrome"`. It raises its rate-limit error when the crumb request returns HTTP 429 or "Too Many Requests". One user reports that hitting fc.yahoo.com once and reusing the crumb works. — [yfinance issue #2456](https://github.com/ranaroussi/yfinance/issues/2456)
- curl_cffi impersonate-target mismatches (for example "chrome136 not supported") come up when yfinance and curl_cffi versions drift apart. This is a secondary blog post. — [softhints](https://softhints.com/how-fix-yfinance-impersonateerror-chrome136-not-supported)
- **Antibot positioning:**
  - wreq's sponsor list centres on antibot bypass (Akamai, DataDome, Kasada, Incapsula token APIs).
  - impit is Apify's own scraping-platform client.
  - Downstream projects use curl-impersonate to get past Cloudflare; for example, a Royal Road downloader PR is titled "fetch Royal Road through Cloudflare".

  — [wreq README](https://raw.githubusercontent.com/0x676e67/wreq/main/README.md), [GitHub API apify/impit](https://api.github.com/repos/apify/impit), [rr-to-epub PR #3](https://github.com/gabcoyne/rr-to-epub/pull/3)
- A web search for wreq, rquest or impit with Yahoo Finance returned nothing relevant. — [web search results incl. rquest docs](https://docs.rs/crate/rquest/2.1.0/source/README.md)

### Inferences
- Matching curl_cffi's yfinance behaviour most closely in Rust means libcurl-impersonate with the latest Windows Chrome target, the same engine as curl_cffi. wreq-util's Chrome profiles are the closest pure-Rust equivalent.
- Throttling may be IP-based, as the yfinance discussion suggests. In that case, no fingerprint client fixes it, and session and crumb reuse plus backoff matter as much.

### Gaps
- No primary evidence on whether Yahoo's 2024–2025 throttling keyed on JA3/JA4 versus Akamai HTTP/2 fingerprints. The empirical researcher should cover this.
