
# Balance - Personal Financing App

A small desktop financing app that displays incomes and expenses.

# Tech Stack

- Rust, MSVC toolchain, Windows only (v1). Requires VS 2022 Build Tools ("Desktop development with C++") and CMake on PATH.
- UI: `gpui-kit = "=0.9.1"` (Longbridge, crates.io). Exact pin; pre-1.0 has frequent breaking changes, upgrade by hand only. Do not also depend on upstream `gpui` or override `gpui-pre-*` versions.
- xlsx read: `calamine`. xlsx write: `umya-spreadsheet`.
- File watching: `notify` + `notify-debouncer-full`.

# System Architecture

```
|-app/     # Single Cargo crate (binary)
|  |-src/core/   # Pure logic, NO gpui imports, unit-tested
|  |-src/ui/     # gpui-kit views
|-assets/  # Source assets (fonts, icons). Embedded into the exe with include_bytes!
|-db/      # balance.xlsx + settings.toml (created on first run if missing) + links.json (price URLs)
|-.cache/  # Fetched prices (exchange/metal/crypto/history .json); safe to delete
```

Installed, the exe folder has the same layout: `balance.exe`, `db/`, `.cache/`.

## db/ location

- Release builds: `<exe_dir>/db/`.
- Debug builds: repo-root `db/` (resolved via `CARGO_MANIFEST_DIR/../db`) so `cargo clean` never deletes data.
  Debug builds also honor the `BALANCE_DB` env var to point at a scratch folder for manual testing.
- `.cache/` sits next to `db/` in both: `<exe_dir>/.cache/` in release, repo-root `.cache/` in debug.
- Missing folder and `settings.toml` are created on first run. A missing workbook means an empty ledger; `balance.xlsx` is created on the first save.
- `links.json` holds every URL prices are fetched from (`core::sources`); no URL lives in code. Feeds `fiat`, `metal`, `crypto`, `fiat_day`, `prices_day`: each an ordered list of `{ "url", "format" }`, tried until one gives what the feed needs. Formats: `frankfurter`, `gold-api`, `coingecko`, `currency-api` (one parser each; a new format needs code, a new link in a known format does not). Placeholders: `{day}` (YYYY-MM-DD; day feeds, required), `{metal}` (XAU/XAG; metal feed only, required by gold-api), `{ids}` (CoinGecko ids; anywhere). Read once at startup and checked: unknown keys or formats, empty feeds, a format that cannot give what its feed needs, a placeholder its feed cannot fill. Missing or bad: an error is printed and nothing is fetched; cached prices still show. It is not created automatically, so a `BALANCE_DB` scratch folder or a release `db/` needs a copy.

## Development

- `cd app && cargo run` — runs against repo-root `db/`.
- `cargo test` — core unit tests (formatting, parsing, balance/intervals/change %, xlsx round-trip).
- Sample data for manual UI testing: `BALANCE_DB=<dir> cargo test seed_sample -- --ignored`
  (add `BALANCE_SEED_BAD=1` to append a malformed row and exercise the ignored-rows guard),
  then `BALANCE_DB=<dir> cargo run`.
- `cargo build --release` → `app/target/release/balance.exe` (no console window).
- Fonts: Inter 4.1 static `Inter-Thin.ttf` / `Inter-Light.ttf` (the 4.1 release names the text/18pt cut without the `_18pt` suffix). DirectWrite reports the family as `Inter`; weights select Thin/Light.

## Distribution

Release = `balance.exe`; fonts and icons are embedded, the app icon (`assets/icons/balance.ico`, built from `balance.png`) via `app/build.rs` (winresource). The C runtime is linked statically (`app/.cargo/config.toml`), so no VC++ Redistributable is needed.

Installer: Inno Setup 6, `installer/balance.iss`. `installer/build.ps1` builds the release exe and `dist/Balance-<version>-setup.exe` (version from `app/Cargo.toml`).
- Install mode dialog: all users (Program Files, admin; preselected) or just me (`%LOCALAPPDATA%\Programs`, no admin). In all-users mode every user gets modify rights on `db\` and `.cache\`, so all Windows users share one workbook.
- Installs `balance.exe` and `db\links.json` (from `installer/links.json`, overwritten on every install). `settings.toml`, `balance.xlsx` and `.cache\` are made by the app.
- Start menu shortcut; desktop shortcut optional (off). Finish page offers "Launch Balance" (on). Publisher `denizhoroz`.
- Uninstall removes the exe and `.cache\`, then asks whether to delete `db\` too (default: keep; silent uninstall keeps it).

## Workbook format (`db/balance.xlsx`)

The workbook holds plain data only. Charts, pivots, comments and formulas added in Excel are not supported and may be lost when the app saves (umya-spreadsheet limitation). Basic cell formatting usually survives.

- Three sheets: `Incomes` and `Expenses` (columns `Name | Value | Date`), and `Assets` (columns `Name | Category | Value | Date`). A missing `Assets` sheet means no assets; it is created on the next save.
- `Value`: non-negative number, max 2 decimals. Stored internally as `i64` cents; Excel f64 is rounded to 2 decimals on read.
- `Date`: written as a real Excel date cell with `dd.mm.yyyy` number format. On read, both date cells and `DD.MM.YYYY` text are accepted.
- A row is valid only if it has a non-empty name, a value (0 is valid) and a valid date. `Assets` rows also need a non-empty category.

## Bad rows

- Invalid rows in the workbook (missing name/value, non-numeric value, bad date, missing sheet content) are skipped.
- The bottom bar shows small gray text "N rows ignored".
- While any rows are ignored the app never overwrites the file (protects the user's data). Edits stay in memory in the "unsaved" state.

## Saving and file locking

- Excel locks an open workbook: the app can read it but not write it (os error 32/33/5).
- Write pattern: write to a temp file in `db/`, `sync_all`, then rename over the target. Map errors 32/33/5 to "locked".
- When locked: edits stay in memory, save is retried on the next edit / window focus, and the bottom bar shows small gray "unsaved" text next to the app name until a save succeeds. No popups.

## External changes

- Watch the `db/` directory (non-recursive, not the file itself, since Excel saves by replacing the file). Debounce, then confirm a real change (mtime/size), and ignore events caused by the app's own writes.
- On change: auto-reload.
- Conflict (unsaved in-memory edits + external change): the file on disk wins, in-memory edits are dropped.

## Settings storage (`db/settings.toml`)

- `currency` (default USD)
- `theme` (default dark)
- `interval` (default 1M)

# Domain rules

- Every income or expense is an **action**: `name`, `value` (cents, >= 0), `date`.
- **Balance** = sum(incomes) - sum(expenses), counting only actions with `date <= today`. There is no opening-balance field; the user adds starting money as an income action.
- **Future-dated actions** (`date > today`) are counted nowhere: not in the balance, change %, balance-view totals or table totals.
- No recurring actions in v1.
- **Intervals**: `1D`, `1W`, `1M`, `1Y`. Rolling windows ending now (e.g. 1W = last 7 days). Cycle order `1D -> 1W -> 1M -> 1Y -> 1D`. Default `1M`. Persisted.
- **Change %** = `(B_now - B_start) / |B_start|`, where `B_start` = balance from actions dated before the window start.
  - `B_start = 0` and `B_now = 0`: show `+0%` (yellow).
  - `B_start = 0` and `B_now != 0`: show `—` (gray).
  - Display: truncated, one decimal below 10% (`+2,5%`), whole number above (`+14%`); a tiny non-zero change shows `+<0,1%` so its color never lies.
- **Amount input** (table value field): comma = decimal separator, dots = thousands (`1.250,50`). A single dot followed by 1-2 digits is read as decimal (`12.5`); followed by 3 digits as thousands (`1.250`).
- **Assets**: the `Assets` sheet holds dated value **snapshots** of holdings (`name`, `category`, `value`, `date`). A holding is identified by its name; its value on a day is its latest snapshot dated on or before that day (same date: the later row wins), and its category is that snapshot's category. Recording a new value = adding a new row; selling = a snapshot with value 0. Future-dated snapshots count nowhere yet.
  - **Total assets** = sum of all holdings' current values + the cash balance. The cash balance (see Balance) is added automatically as category `Cash` (merged with a user category of the same name). A negative cash balance counts in the total but gets no ring segment.
  - **Assets change %**: same formula and display rules as the balance change %, on total assets, using the same (shared) interval setting. The window-start total is valued at that day's rates (fetched and cached like the Assets Table's past days), the current total at today's, so price moves count; `—` until that day's rates are loaded.
- **Currency** changes display only (symbol). No exchange-rate conversion. Options: USD `$`, EUR `€`, GBP `£`, TRY `₺`, JPY `¥`. Default USD.

## Number formatting

- One fixed format: dot thousands separator, comma decimal separator, symbol after: `23.246,50 $`. Decimals shown only if non-zero (`23.246 $`).
- **Abbreviation** from 1 million up: `M` (>= 1.000.000), `B` (>= 1.000.000.000), `T` (>= 10^12). 3 significant digits, comma decimal, truncated (never rounded up): `43,4M $`, `1,42B $`, `999.999.999` -> `999M $`. Not used below 1 million.
- Abbreviation applies to: balance-view balance, balance-view income/expense totals, assets total and hovered category value, table totals. Table row values always show full numbers.
- Fallback: if a number still does not fit, step the font size down until it fits. No truncation, no ellipsis.

# Design

## Main Design

Small, compact view, sharp rectangular borders. Absolutely no borders if not intentionally specified. No animations. Fast, simple view.

- **Fonts**: Inter 18pt Thin (weight 100) for the big balance number only; Inter 18pt Light (300) for everything else. Both TTFs live in `assets/fonts/` and are embedded and registered at startup with `cx.text_system().add_fonts(...)`. Reference the family name stored inside the font file and verify with `all_font_names()`. Include the Inter OFL license.
- **Colors**:
  - Dark mode (default): neon green `#39FF14`, neon red `#FF3131`, neon yellow `#FFF01F` on near-black.
  - Light mode: darker variants of the same hues on white (e.g. `#00A86B`, `#D7263D`, `#C9A400`). Neon is unreadable on white.
  - Gray for zero values, labels and secondary text.
- **Icons**: from `gpui-kit-assets`. Gray, borderless, no text, brighter on hover.

## Window

- One window, fixed 560x420 (landscape), non-resizable, native Windows title bar, title "balance". No position memory.
- On window close call `cx.quit()` explicitly (workaround for the gpui Windows hang-on-exit bug).

## Layout

Two top-level views, switched with the bottom-center icons (in this order):

- Balance (two opposing arrows icon) — the view the app opens on
- Assets (money bag icon)

Sub-views: Table (incomes/expenses), Assets Table, Asset, Settings. Back arrow and `Esc` return to the last top-level view (the Asset view returns to the view it was opened from).

Every view has a bottom bar: app name "balance" on the bottom left (plus status text "unsaved" / "N rows ignored" when relevant, small gray).
- Top-level views: the two view icons at the bottom center (current view's icon bright, others gray); Light/Dark toggle (sun/moon) and Settings at the bottom right. The toggle switches dark mode on/off and persists.
- Sub-views: a back-arrow icon at the bottom right.

### Assets View

- The same big number + change % layout as the balance view, without the income/expense totals: total assets (green/red/yellow by sign) and the assets change % with the clickable interval label to its right.
- Around them a thin ring, hollow inside, split into one segment per asset category (a donut, not a filled pie). Segment size = the category's share of the positive total. Largest first, starting at 12 o'clock, clockwise, with small gaps between segments. Categories with value <= 0 get no segment. With nothing to show, the ring is one full gray circle.
- Category colors come from a fixed category palette (not green/red/yellow, which mean +/-/0). Colors are assigned `Cash` first, then by each category's first snapshot, so a category keeps its color when others are added.
- Hovering a segment makes it thicker and swaps the center to that category: its value in the category color and `Name · 38%` below (share truncated; `<1%` for tiny shares). Leaving the ring restores the total.
- Clicking the big number opens the Assets Table.

### Assets Table

- Ledger built from the Assets sheet (Assets is the only source). One row per current holding; sold holdings (value 0), superseded and future snapshots are not shown. Sorted by current value in the display currency, largest first; holdings without a rate go last.
- Small gray header, then rows: `Name ...... Category  Value  Type  Change  Profit`.
  - Category in its ring color. Value in the holding's own currency, full number.
  - Type: the holding's currency code, small gray (`XAU`, `BTC`, `EUR`).
  - Change % and profit over the shared interval compare the current value (today's rates) with a reference value in the display currency:
    - held at the window start: its snapshot then, at the window start day's rates;
    - added inside the window: its first snapshot there, at that snapshot's date's rates (a row dated 09.11 is compared from 09.11 prices).
  - Change % follows the change % rules and colors; profit is signed and compact (`+1.250 $`), green/red/yellow by sign. Both show `—` while a needed rate is not loaded yet.
  - Past days' rates are fetched in the background when the view needs them (ECB fiat via Frankfurter `/v1/{day}`; metals and crypto via the daily fawazahmed0 currency-api) and cached for good in `.cache/history.json`. Failed days retry with backoff.
- No inline editing: clicking a row opens the Asset view on that holding's current snapshot. Right-click deletes that snapshot (the holding's previous one, if any, becomes current).
- `+` centered below the list opens the Asset view on a new holding. With no holdings, plain gray text "No holdings yet" above it.
- Pinned footer: "Holdings", the clickable interval label (under Category), the total current value in the display currency, then change % and profit summed over all rows.

### Asset View

One holding. Header only for now; more information below it is planned.

- Left: name in big text (gray `name` until written); below it, with a small gap, the category in its ring color (gray `category`).
- Right, across from the name: value in the holding's own currency (gray `value`), with its type (currency code) to its left in small gray text (gray `type`). Below the value: the date `DD.MM.YYYY` in small gray text (future dates dimmer).
- **New holding** (from the Assets Table `+`): every header field is clicked to edit in place, starting on the name. The type opens as a dropdown of the asset currencies. The date defaults to today; change it to declare an earlier investment. Enter moves to the next missing field in order name -> category -> type -> value (never stops on the date); Tab / Shift+Tab move through name -> category -> type -> value -> date; clicking outside commits; `Esc` cancels the field. Incomplete fields stay while the view is open. Nothing is written to the workbook (and nothing counts in totals) while the view is open; on back / `Esc` it is saved if complete (name, category, value), otherwise discarded. An external reload drops it and leaves the view.
- The session decides what the view shows and which fields can be edited; when it has nothing to show (e.g. its asset was removed by a reload) the view closes. A saved asset row's view closes on a reload.
- **Saved asset** (clicking an Assets Table row): value, type and date are fixed. Name and category can be edited in place; they change for the whole holding (every snapshot with that name). An empty text, or a name another holding already has, reverts. Tab / Shift+Tab move name <-> category. Saved when the field closes.

### Balance View

- Balance number centered in big font (e.g. `23.246 $`, `-32.534 $`, `1,42B $`). Positive = green, negative = red, 0 = yellow. All-time balance (date <= today).
- Below it, the change in percent in smaller font (e.g. `+14%`, `-5%`). Positive = green, negative = red, 0 = yellow (see change % rules for `—`). The change text alone is horizontally centered; the small gray interval label sits immediately to its right, vertically centered with it: label (`1D`, `1W`, `1M`, `1Y`).
  - The interval label is not a button but is clickable: clicking cycles to the next interval.
- Below that, total income and total expense **within the selected interval**, in small text, separated by a vertical line at the center. Left: income in green (gray if 0). Right: expense in red (gray if 0). Both shown as positive numbers.
  - The balance number, the change % text and the totals each open the Table view; only where text is, not the full width. The totals click area spans from the start of the income text to the end of the expense text (including the line and the gaps between). The interval label only cycles the interval. There is no separate Table button.

### Table

Displays all incomes and expenses and lets the user edit them.

- Upper section fills most of the page; a small bottom bar below.
  - Bottom bar: app name bottom left, back-arrow icon button bottom right.
- Upper section is split in half: incomes on the left, expenses on the right. Each column scrolls independently.
- Each column starts with a small gray header naming its columns: `Name  Date  Value  Type`.
- Each action is a row: `Name ........ DD.MM Value`
  - Name (editable). Empty name never displays because such a row cannot exist.
  - Date: small gray text before the value. Displayed `DD.MM` if in the current year, otherwise `DD.MM.YYYY`. Edited as text `DD.MM.YYYY`. Future dates are gray.
  - Value (editable, full number, never abbreviated). Income green, expense red, 0 gray.
- Rows are sorted by date ascending (oldest top). Re-sort happens when an edit is committed, not while typing.
- A plus icon after the last row of each column, horizontally centered in the column, adds a new action of that type, dated today. It scrolls with the list.
- Under each column, pinned (not scrolling): the total, labeled "Incomes" / "Expenses". Abbreviated per the formatting rules. Future-dated actions are excluded.
- **Editing (popup)**:
  - Cells are plain text (long names end with an ellipsis). Clicking a cell opens a popup editor laid over that row, spanning the full column width, with a small gray field label (`Name` / `Date` / `Value`) and the current text pre-selected. One popup at a time.
  - Prefill: name as-is; date as `DD.MM.YYYY`; value without thousands dots (`1250,50`, not `1.250,50`).
  - Clicking outside the popup commits it.
- **Editing rules**:
  - Value input: non-negative number, max 2 decimals. Date input: valid `DD.MM.YYYY`. Invalid input reverts to the previous value.
  - A row must have a name and a value (0 is a valid value). When focus leaves the row (not the individual field), a row with an empty name or empty value is deleted. This means a new row from `+` survives while typing, and disappears if the user clicks away without filling both. Invalid value input on a new row (nothing to revert to) counts as empty.
  - `+` opens the popup on the new row's name.
  - Right-click on a row deletes it. No confirmation, no undo.
- **Keyboard**: `Enter` commits the field and closes the popup; on an incomplete row it moves to the missing field (name, then value) instead. `Esc` cancels the edit (or returns to the last top-level view when not editing). `Tab` / `Shift+Tab` move name -> date -> value -> next row (and back); past either end the popup closes.

### Settings

Internal configurations the user can change.

- Every setting is listed as a normal row: label left, control right.
- Bottom bar: app name bottom left, back-arrow icon button bottom right.
- Settings (v1):
  - "Change Currency": dropdown with USD, EUR, GBP, TRY, JPY.

# Testing

Unit tests in `core` (no gpui):

- Balance, future-date exclusion
- Interval windows and change % (including `B_start = 0` edge cases)
- Number formatting, abbreviation thresholds and truncation
- Date parsing and formatting
- xlsx parsing, including bad and incomplete rows
- Asset snapshots: latest-wins, future exclusion, category grouping, cash merge, assets change %

No UI tests.
