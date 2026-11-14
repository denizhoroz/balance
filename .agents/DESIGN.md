# Balance - Design Styles

The visual language of the app as built. `PROJECT.md` holds the product rules; this file holds the look.
Values come from `app/src/ui/palette.rs`, `app/src/ui/views.rs` and `BalanceApp::apply_theme` in `app/src/ui/app.rs`.

# Principles

- Small, compact and quiet. The numbers are the content; everything else is gray.
- Sharp rectangles: radius `0` everywhere, no shadows, no focus rings.
- **No borders.** Separation comes from spacing and, where needed, a slightly raised surface color. The only drawn lines are the 1px divider between the totals on the balance view and the thin assets ring.
- No animations, no transitions.
- No text on buttons: icons only, gray, brighter on hover.
- Color carries meaning only: green = income / positive, red = expense / negative, yellow = exactly zero (balance, change %), gray = zero amounts, labels, secondary text and history. Asset categories get their own hues, never green/red/yellow.

# Color

Theme tokens (`Palette`). Dark is the default.

| Token       | Dark      | Light     | Use                                                           |
|-------------|-----------|-----------|---------------------------------------------------------------|
| `bg`        | `#0A0A0A` | `#FFFFFF` | Window background, inputs                                     |
| `fg`        | `#EDEDED` | `#111111` | Primary text (names, settings labels), icon hover             |
| `muted`     | `#7A7A7A` | `#8A8A8A` | Labels, dates, zero values, app name, status text, icons     |
| `line`      | `#3A3A3A` | `#D0D0D0` | Balance-view totals divider, scrollbar thumb                     |
| `selection` | `#2A2A2A` | `#E6E6E6` | Text selection, list hover/active                             |
| `surface`   | `#181818` | `#F2F2F2` | Raised areas: edit popup, dropdown list                       |
| `green`     | `#39FF14` | `#00A86B` | Income, positive balance / change                             |
| `red`       | `#FF3131` | `#D7263D` | Expense, negative balance / change                            |
| `yellow`    | `#FFF01F` | `#C9A400` | Zero balance, `+0%` change                                    |

- Dark mode uses neon hues; light mode uses darker variants of the same hues (neon is unreadable on white).
- Category palette (`categories`, ring segments and category text), assigned in order:

| # | Dark      | Light     |
|---|-----------|-----------|
| 1 | `#00E5FF` cyan     | `#0097A7` |
| 2 | `#FF8A1F` orange   | `#E65100` |
| 3 | `#B066FF` violet   | `#7B1FA2` |
| 4 | `#FF3DCB` magenta  | `#C2185B` |
| 5 | `#3D8BFF` blue     | `#1565C0` |
| 6 | `#D4A373` tan      | `#6D4C41` |
| 7 | `#C9B3FF` lavender | `#8E6CC9` |
| 8 | `#7FD4FF` sky      | `#2B8CC4` |

  `Cash` always takes #1; other categories follow in order of their first snapshot, so adding a category never recolors existing ones. More than 8 categories wrap around.
- Future-dated rows: date text in `muted` at 50% opacity.
- Undefined change (`—`) is `muted`.
- The gpui-kit component theme is overwritten from the palette so built-in widgets match: `border = surface` (invisible against popups), `input = ring = bg`, `popover = surface`, `list_hover/active/accent = selection`, `scrollbar = bg`, `scrollbar_thumb = line` (hover `muted`).

# Typography

Inter 4.1, two static cuts embedded in the exe (`assets/fonts/`, OFL).

| Role                         | Weight       | Size                                      |
|------------------------------|--------------|-------------------------------------------|
| Balance-view balance         | Thin (100)   | 72px, steps down 64/56/48/40/34/28/24/20 to fit 520px |
| Assets total / hovered category value | Thin (100) | same steps, to fit 296px (inside the ring) |
| Change %                     | Light (300)  | 22px                                      |
| Balance-view totals             | Light        | 16px                                      |
| Body default, edit popup text, hovered category label | Light | 13px                       |
| Table rows, interval label, table totals | Light | 12px                               |
| Row dates, status text, popup field label | Light | 11px                              |

- Thin is used for the big numbers only (balance, assets); everything else is Light.
- Numbers never truncate or ellipsize; they shrink. Long row names ellipsize (`…`), single line.

# Layout

- Window: fixed 560 x 420 (landscape), non-resizable, native title bar "balance".
- Every view = content area (fills) + bottom bar.

## Bottom bar

- Height 36px, horizontal padding 12px. Three parts: `[left flex_1] [center] [right flex_1, end-aligned]`; the equal sides keep the center group exactly centered.
- Left: "balance" in `muted`, then status texts (11px, `muted`, gap 10px): `file unreadable`, `N rows ignored`, `unsaved`.
- Icon buttons everywhere: 16px icon, 6px padding, `muted` → `fg` on hover, pointer cursor.
- Right: top-level views (Assets, Balance, Investments): Sun/Moon (theme toggle), Settings. Sub-views (Table, Assets Table, Settings): ArrowLeft (back to the last top-level view).
- Center (top-level views only, 4px gap): money bag (Assets), ArrowRightLeft (Balance), ChartLine (Investments). The current view's icon is `fg`, the others `muted`.
- Icons are Lucide (`gpui-kit-assets`). The money bag is not in Lucide; it is a custom SVG in the same style (24 viewBox, 2px round stroke, `currentColor`) at `assets/icons/money-bag.svg`, embedded and served as `icons/money-bag.svg`.

## Balance view

Vertical stack, centered in the content area:

1. Balance (Thin, signed color), no wrap.
2. Change row, full width: `[flex_1 spacer] [change 22px] [flex_1: 8px gap + interval label 14px muted]`. The two flex gutters keep the change itself exactly centered; the label hangs off its right, vertically centered. Label hover → `fg`, click cycles the interval.
3. Totals row, 24px below, 16px. Click target (pointer cursor, opens the Table) is three adjacent 38px-tall pieces: income text + its 18px gap, the line, the 18px gap + expense text; the empty sides of the row are not clickable. Layout: income right-aligned in the left half (18px from center), 1x20px `line` divider at center, expense left-aligned in the right half (18px from center). Zero → `muted`.

## Assets view

- A 400x400 box centered in the content area holds the ring; the center content (312px wide) sits on top of it.
- Ring: drawn with a `canvas` as stroked polylines (`PathBuilder::stroke`), radius = half the box minus 5px. Stroke 2.5px; the hovered segment 5px. Segments start at 12 o'clock and run clockwise, largest first, with a 6px arc gap between them (no gap with a single segment). Empty state: one full circle in `line`.
- Hover hit area: within 14px of the ring line. Hover state lives in `BalanceApp::hovered_segment`.
- Center, default: total assets (Thin, signed color, clickable → Assets Table) + the shared change row (change 22px + interval label).
- Center, hovering a segment: category value (Thin, category color) + `Name · 38%` (16px, `muted`).

## Investments view

- Empty content area for now; only the bottom bar.

## Table

- Two equal columns, 16px gap, 12px side padding, 8px top padding. Each column scrolls on its own.
- Row: 24px tall, 12px text, 6px gap: `Name (flex, ellipsis) | Date (11px muted; 38px wide for DD.MM, 64px for DD.MM.YYYY) | Value (80px, right-aligned, kind color, 0 muted)`. Missing name shows `name`, missing value `—`, both muted.
- `+` add button: after the last row, 24px row, 14px Plus icon centered in the column, 8px side hit padding, `muted` → `fg` on hover.
- Column footer (pinned, not scrolling): 28px, 12px, label left in `muted` ("Incomes" / "Expenses"), total right in kind color (0 muted).
- Cells use a text cursor; clicking opens the edit popup.

## Assets table

- One full-width list, 12px side padding, 8px top padding.
- Row: `Name (flex) | Category (110px, category color; missing → "category" muted) | Date | Value (100px, right-aligned)`. Current snapshot values `fg`, superseded snapshots and 0 `muted`.
- Footer: "Holdings" (`muted`) left, total right in `fg`.

## Edit popup

- Laid over the edited row, overhanging it by 4px vertically and 6px horizontally, spanning the full column width.
- `surface` background, no border, 6px side padding, 8px gap.
- Small field label (`Name` / `Category` / `Date` / `Value`, 11px muted), then a borderless input at 13px. Input text color: `fg`, except values: green for incomes, red for expenses.
- Drawn deferred (above neighbours) and occluding, so clicks don't reach rows beneath.

## Settings

- 16px side padding, 12px top padding.
- One 36px row per setting: label left (`fg`), control right.
- Dropdown: borderless gpui-kit `Select`, small, fixed 96px wide; list on `surface`, hover/active `selection`.

# Number display

- `23.246,50 $`: dot thousands, comma decimal, currency symbol after with a space; decimals only when non-zero.
- From 1 million up, the balance and totals abbreviate (`43,4M $`, `1,42B $`); table rows always show full numbers.
- In the edit popup values show without thousands dots (`1250,50`).
