//! Workbook persistence: `db/balance.xlsx` with sheets `Incomes` and
//! `Expenses` (columns `Name | Value | Date | Currency`) and `Assets`
//! (`Name | Category | Value | Date | Currency`), header in row 1.
//!
//! Currency is the last column so workbooks written before it existed
//! still parse: a blank or missing currency takes the caller's default.

use std::path::{Path, PathBuf};

use calamine::{Data, Reader, Xlsx, open_workbook};
use chrono::NaiveDate;

use super::assets::Snapshot;
use super::currency::Currency;
use super::date::{ISO_DATE, date_from_excel_serial, date_to_excel_serial, parse_date};
use super::ledger::{Action, Kind, Ledger};
use super::money::{Cents, parse_amount_dec, units_from_f64, units_to_f64};

pub const WORKBOOK_FILE: &str = "balance.xlsx";
pub const SETTINGS_FILE: &str = "settings.toml";
pub const ASSETS_SHEET: &str = "Assets";
const HEADERS: [&str; 4] = ["Name", "Value", "Date", "Currency"];
const ASSET_HEADERS: [&str; 5] = ["Name", "Category", "Value", "Date", "Currency"];

/// Where `db/` lives: repo root in debug builds, next to the exe in release.
/// Debug builds honor `BALANCE_DB` to point at a scratch folder for testing.
pub fn db_dir() -> PathBuf {
    if cfg!(debug_assertions) {
        if let Some(dir) = std::env::var_os("BALANCE_DB") {
            return PathBuf::from(dir);
        }
        Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("db")
    } else {
        exe_dir().join("db")
    }
}

/// Folder of the running exe (`.` if unknown); release builds keep `db/`
/// and `.cache/` here.
pub fn exe_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(Path::to_path_buf))
        .unwrap_or_else(|| PathBuf::from("."))
}

#[derive(Debug, Default)]
pub struct Loaded {
    pub ledger: Ledger,
    /// Non-empty rows that could not be parsed.
    pub ignored: usize,
    /// Some rows had no currency and took the default; the workbook should
    /// be rewritten so the currency is explicit.
    pub defaulted: bool,
}

#[derive(Debug)]
pub enum SaveError {
    /// File is held open by another process (Excel).
    Locked,
    Other(anyhow::Error),
}

impl std::fmt::Display for SaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SaveError::Locked => write!(f, "workbook is locked by another process"),
            SaveError::Other(e) => write!(f, "{e}"),
        }
    }
}

fn is_lock_error(e: &std::io::Error) -> bool {
    // 32 ERROR_SHARING_VIOLATION, 33 ERROR_LOCK_VIOLATION, 5 ERROR_ACCESS_DENIED
    matches!(e.raw_os_error(), Some(32 | 33 | 5))
}

fn cell_name(c: &Data) -> Option<String> {
    let s = match c {
        Data::String(s) => s.trim().to_string(),
        Data::Int(i) => i.to_string(),
        Data::Float(f) => f.to_string(),
        _ => return None,
    };
    (!s.is_empty()).then_some(s)
}

/// Amount in minor units of a currency with `dec` decimals.
fn cell_value(c: &Data, dec: u32) -> Option<i64> {
    match c {
        Data::Int(i) if *i >= 0 => i.checked_mul(10i64.pow(dec)),
        Data::Float(f) => units_from_f64(*f, dec),
        Data::String(s) => parse_amount_dec(s, dec),
        _ => None,
    }
}

fn cell_date(c: &Data) -> Option<NaiveDate> {
    match c {
        Data::DateTime(dt) => date_from_excel_serial(dt.as_f64()),
        Data::Float(f) => date_from_excel_serial(*f),
        Data::Int(i) => date_from_excel_serial(*i as f64),
        Data::String(s) => parse_date(s),
        Data::DateTimeIso(s) => NaiveDate::parse_from_str(s.get(..10)?, ISO_DATE).ok(),
        _ => None,
    }
}

/// `Some(None)` = blank (use the default), `None` = unknown code.
fn cell_currency(c: &Data) -> Option<Option<Currency>> {
    if is_blank(c) {
        return Some(None);
    }
    match c {
        Data::String(s) => Currency::from_code(s).map(Some),
        _ => None,
    }
}

fn is_blank(c: &Data) -> bool {
    match c {
        Data::Empty => true,
        Data::String(s) => s.trim().is_empty(),
        _ => false,
    }
}

/// Shared row loop: skips blank rows and a `Name` header in row 1; `parse`
/// returning `None` counts the row as ignored.
fn parse_with<T>(rows: &[Vec<Data>], parse: impl Fn(&[Data]) -> Option<T>) -> (Vec<T>, usize) {
    let mut out = Vec::new();
    let mut ignored = 0;
    for (i, row) in rows.iter().enumerate() {
        if row.iter().all(is_blank) {
            continue;
        }
        if i == 0 && matches!(cell(row, 0), Data::String(s) if s.trim().eq_ignore_ascii_case("name")) {
            continue;
        }
        match parse(row) {
            Some(v) => out.push(v),
            None => ignored += 1,
        }
    }
    (out, ignored)
}

fn cell(row: &[Data], n: usize) -> &Data {
    row.get(n).unwrap_or(&Data::Empty)
}

/// Rows parsed from one sheet.
pub struct Parsed<T> {
    pub items: Vec<T>,
    pub ignored: usize,
    /// Some rows had a blank currency and took the default.
    pub defaulted: bool,
}

fn parse_sheet<T>(
    rows: &[Vec<Data>],
    currency_col: usize,
    default: Currency,
    make: impl Fn(&[Data], Currency) -> Option<T>,
) -> Parsed<T> {
    let defaulted = std::cell::Cell::new(false);
    let (items, ignored) = parse_with(rows, |r| {
        let currency = match cell_currency(cell(r, currency_col))? {
            Some(c) => c,
            None => {
                defaulted.set(true);
                default
            }
        };
        make(r, currency)
    });
    Parsed { items, ignored, defaulted: defaulted.get() }
}

/// Parses rows of an income/expense sheet. Unknown currency codes and
/// non-fiat currencies (assets only) make the row ignored; a blank currency
/// takes `default`.
pub fn parse_rows(rows: &[Vec<Data>], default: Currency) -> Parsed<Action> {
    parse_sheet(rows, 3, default, |r, currency| {
        if !currency.is_fiat() {
            return None;
        }
        Some(Action {
            name: cell_name(cell(r, 0))?,
            value: cell_value(cell(r, 1), currency.decimals())?,
            currency,
            date: cell_date(cell(r, 2))?,
        })
    })
}

/// Parses rows of the `Assets` sheet. Category is required. `XAU`/`XAG`
/// values are grams; `BTC`/`ETH` coins with up to 8 decimals.
pub fn parse_asset_rows(rows: &[Vec<Data>], default: Currency) -> Parsed<Snapshot> {
    parse_sheet(rows, 4, default, |r, currency| {
        Some(Snapshot {
            name: cell_name(cell(r, 0))?,
            category: cell_name(cell(r, 1))?,
            value: cell_value(cell(r, 2), currency.decimals())?,
            currency,
            date: cell_date(cell(r, 3))?,
        })
    })
}

/// Loads the workbook. A missing file yields an empty ledger. Rows without
/// a currency take `default`.
pub fn load(path: &Path, default: Currency) -> anyhow::Result<Loaded> {
    if !path.exists() {
        return Ok(Loaded::default());
    }
    let mut wb: Xlsx<_> = open_workbook(path)?;
    let names = wb.sheet_names();
    let mut loaded = Loaded::default();
    for kind in [Kind::Income, Kind::Expense] {
        if !names.iter().any(|n| n == kind.sheet_name()) {
            continue;
        }
        let range = wb.worksheet_range(kind.sheet_name())?;
        let rows: Vec<Vec<Data>> = range.rows().map(|r| r.to_vec()).collect();
        let parsed = parse_rows(&rows, default);
        *loaded.ledger.list_mut(kind) = parsed.items;
        loaded.ignored += parsed.ignored;
        loaded.defaulted |= parsed.defaulted;
    }
    if names.iter().any(|n| n == ASSETS_SHEET) {
        let range = wb.worksheet_range(ASSETS_SHEET)?;
        let rows: Vec<Vec<Data>> = range.rows().map(|r| r.to_vec()).collect();
        let parsed = parse_asset_rows(&rows, default);
        loaded.ledger.assets = parsed.items;
        loaded.ignored += parsed.ignored;
        loaded.defaulted |= parsed.defaulted;
    }
    loaded.ledger.sort();
    Ok(loaded)
}

/// Clears the data columns (keeping everything else) and writes the header.
fn reset_sheet(ws: &mut umya_spreadsheet::Worksheet, headers: &[&str]) {
    let highest = ws.highest_row();
    for row in 1..=highest {
        for col in 1..=headers.len() as u32 {
            ws.remove_cell((col, row));
        }
    }
    for (i, h) in headers.iter().enumerate() {
        ws.cell_mut((i as u32 + 1, 1)).set_value_string(*h);
    }
}

fn write_value(ws: &mut umya_spreadsheet::Worksheet, col: u32, row: u32, value: Cents, currency: Currency) {
    let dec = currency.decimals();
    let v = ws.cell_mut((col, row));
    v.set_value_number(units_to_f64(value, dec));
    // Two decimals always; crypto shows up to 8.
    let format = if dec > 2 { "#,##0.00######" } else { "#,##0.00" };
    v.style_mut().number_format_mut().set_format_code(format);
}

fn write_date(ws: &mut umya_spreadsheet::Worksheet, col: u32, row: u32, date: NaiveDate) {
    let d = ws.cell_mut((col, row));
    d.set_value_number(date_to_excel_serial(date));
    d.style_mut().number_format_mut().set_format_code("dd.mm.yyyy");
}

fn write_sheet(ws: &mut umya_spreadsheet::Worksheet, actions: &[Action]) {
    reset_sheet(ws, &HEADERS);
    for (i, a) in actions.iter().enumerate() {
        let row = i as u32 + 2;
        ws.cell_mut((1, row)).set_value_string(a.name.clone());
        write_value(ws, 2, row, a.value, a.currency);
        write_date(ws, 3, row, a.date);
        ws.cell_mut((4, row)).set_value_string(a.currency.code());
    }
}

fn write_asset_sheet(ws: &mut umya_spreadsheet::Worksheet, assets: &[Snapshot]) {
    reset_sheet(ws, &ASSET_HEADERS);
    for (i, a) in assets.iter().enumerate() {
        let row = i as u32 + 2;
        ws.cell_mut((1, row)).set_value_string(a.name.clone());
        ws.cell_mut((2, row)).set_value_string(a.category.clone());
        write_value(ws, 3, row, a.value, a.currency);
        write_date(ws, 4, row, a.date);
        ws.cell_mut((5, row)).set_value_string(a.currency.code());
    }
}

/// Saves atomically: write a temp file in the same folder, then rename it
/// over the target. Other sheets of an existing workbook are kept.
pub fn save(path: &Path, ledger: &Ledger) -> Result<(), SaveError> {
    let other = |e: anyhow::Error| SaveError::Other(e);
    let dir = path.parent().ok_or_else(|| other(anyhow::anyhow!("no parent dir")))?;
    std::fs::create_dir_all(dir).map_err(|e| other(e.into()))?;

    // Fail fast when Excel holds the file: opening for write is refused.
    if path.exists() {
        if let Err(e) = std::fs::OpenOptions::new().write(true).open(path) {
            return Err(if is_lock_error(&e) { SaveError::Locked } else { other(e.into()) });
        }
    }

    let mut book = if path.exists() {
        umya_spreadsheet::reader::xlsx::read(path).map_err(|e| other(anyhow::anyhow!("{e}")))?
    } else {
        let mut b = umya_spreadsheet::new_file();
        b.sheet_by_name_mut("Sheet1")
            .map_err(|e| other(anyhow::anyhow!("{e}")))?
            .set_name(Kind::Income.sheet_name());
        b
    };
    for kind in [Kind::Income, Kind::Expense] {
        if book.sheet_by_name_mut(kind.sheet_name()).is_err() {
            book.new_sheet(kind.sheet_name()).map_err(|e| other(anyhow::anyhow!("{e}")))?;
        }
        let ws = book
            .sheet_by_name_mut(kind.sheet_name())
            .map_err(|e| other(anyhow::anyhow!("{e}")))?;
        write_sheet(ws, ledger.list(kind));
    }
    if book.sheet_by_name_mut(ASSETS_SHEET).is_err() {
        book.new_sheet(ASSETS_SHEET).map_err(|e| other(anyhow::anyhow!("{e}")))?;
    }
    let ws = book.sheet_by_name_mut(ASSETS_SHEET).map_err(|e| other(anyhow::anyhow!("{e}")))?;
    write_asset_sheet(ws, &ledger.assets);

    let tmp = dir.join(format!(".{WORKBOOK_FILE}.tmp-{}", std::process::id()));
    umya_spreadsheet::writer::xlsx::write(&book, &tmp).map_err(|e| other(anyhow::anyhow!("{e}")))?;
    if let Ok(f) = std::fs::File::open(&tmp) {
        let _ = f.sync_all();
    }
    if let Err(e) = std::fs::rename(&tmp, path) {
        let _ = std::fs::remove_file(&tmp);
        return Err(if is_lock_error(&e) { SaveError::Locked } else { other(e.into()) });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    fn s(v: &str) -> Data {
        Data::String(v.into())
    }

    fn act(name: &str, value: Cents, date: NaiveDate) -> Action {
        Action { name: name.into(), value, currency: Currency::Usd, date }
    }

    fn snap(name: &str, category: &str, value: Cents, currency: Currency, date: NaiveDate) -> Snapshot {
        Snapshot { name: name.into(), category: category.into(), value, currency, date }
    }

    #[test]
    fn currency_column_parses_defaults_and_rejects_unknown() {
        let rows = vec![
            vec![s("Name"), s("Value"), s("Date"), s("Currency")],
            vec![s("eur pay"), Data::Float(10.0), s("01.01.2026"), s("eur")],
            vec![s("blank"), Data::Float(10.0), s("01.01.2026"), s("")],
            vec![s("legacy"), Data::Float(10.0), s("01.01.2026")],
            vec![s("swiss"), Data::Float(10.0), s("01.01.2026"), s("CHF")],
        ];
        let p = parse_rows(&rows, Currency::Try);
        assert_eq!(p.ignored, 1);
        assert!(p.defaulted);
        let cur: Vec<_> = p.items.iter().map(|a| a.currency).collect();
        assert_eq!(cur, [Currency::Eur, Currency::Try, Currency::Try]);

        let explicit = vec![vec![s("x"), Data::Float(1.0), s("01.01.2026"), s("USD")]];
        assert!(!parse_rows(&explicit, Currency::Try).defaulted);
    }

    #[test]
    fn crypto_keeps_8_decimals_through_the_workbook() {
        let dir = std::env::temp_dir().join(format!("balance-crypto-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let path = dir.join(WORKBOOK_FILE);
        let ledger = Ledger {
            assets: vec![snap("Wallet", "Crypto", 512_345, Currency::Btc, d(2026, 1, 1))],
            ..Ledger::default()
        };
        save(&path, &ledger).unwrap();
        assert_eq!(load(&path, Currency::Usd).unwrap().ledger, ledger);
        std::fs::remove_dir_all(&dir).unwrap();

        let typed = vec![vec![s("Wallet"), s("Crypto"), s("0.005"), s("01.01.2026"), s("ETH")]];
        assert_eq!(parse_asset_rows(&typed, Currency::Usd).items[0].value, 500_000);
        let income = vec![vec![s("paid"), Data::Float(0.1), s("01.01.2026"), s("BTC")]];
        assert_eq!(parse_rows(&income, Currency::Usd).ignored, 1);
    }

    #[test]
    fn metals_only_in_assets() {
        let income = vec![vec![s("gift"), Data::Float(5.0), s("01.01.2026"), s("XAU")]];
        assert_eq!(parse_rows(&income, Currency::Usd).ignored, 1);
        let asset = vec![vec![s("Coins"), s("Metals"), Data::Float(12.5), s("01.01.2026"), s("xau")]];
        let p = parse_asset_rows(&asset, Currency::Usd);
        assert_eq!(p.items, vec![snap("Coins", "Metals", 1_250, Currency::Xau, d(2026, 1, 1))]);
    }

    #[test]
    fn parse_rows_skips_header_blank_and_bad() {
        let rows = vec![
            vec![s("Name"), s("Value"), s("Date")],
            vec![s("salary"), Data::Float(1500.5), Data::Float(45_000.0)],
            vec![Data::Empty, Data::Empty, Data::Empty],
            vec![s("coffee"), s("3,50"), s("09.11.2026")],
            vec![s(""), Data::Float(10.0), Data::Float(45_000.0)], // no name
            vec![s("bad value"), s("abc"), s("09.11.2026")],
            vec![s("no value"), Data::Empty, s("09.11.2026")],
            vec![s("bad date"), Data::Float(1.0), s("2026/11/09")],
            vec![s("negative"), Data::Float(-1.0), s("09.11.2026")],
            vec![s("zero"), Data::Int(0), s("01.01.2026")],
        ];
        let p = parse_rows(&rows, Currency::Usd);
        assert_eq!(p.ignored, 5);
        assert!(p.defaulted);
        assert_eq!(
            p.items,
            vec![
                act("salary", 150_050, d(2023, 3, 15)),
                act("coffee", 350, d(2026, 11, 9)),
                act("zero", 0, d(2026, 1, 1)),
            ]
        );
    }

    #[test]
    fn save_and_load_roundtrip() {
        let dir = std::env::temp_dir().join(format!("balance-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let path = dir.join(WORKBOOK_FILE);
        let mut ledger = Ledger {
            incomes: vec![
                act("salary", 250_000, d(2026, 11, 1)),
                Action { currency: Currency::Eur, ..act("gift", 1_050, d(2026, 10, 3)) },
            ],
            expenses: vec![Action { currency: Currency::Try, ..act("rent", 0, d(2026, 11, 2)) }],
            assets: vec![
                snap("Gold", "Metals", 55, Currency::Gbp, d(2026, 2, 1)),
                snap("ETF", "Stocks", 120_000, Currency::Usd, d(2026, 6, 1)),
            ],
        };
        save(&path, &ledger).unwrap();
        ledger.sort();
        assert_eq!(load(&path, Currency::Jpy).unwrap().ledger, ledger);

        // Shrinking the list must not leave stale rows behind.
        ledger.incomes.truncate(1);
        ledger.assets.truncate(1);
        save(&path, &ledger).unwrap();
        let loaded = load(&path, Currency::Jpy).unwrap();
        assert_eq!(loaded.ledger, ledger);
        assert_eq!(loaded.ignored, 0);
        assert!(!loaded.defaulted);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn parse_asset_rows_requires_category() {
        let rows = vec![
            vec![s("Name"), s("Category"), s("Value"), s("Date")],
            vec![s("ETF"), s("Stocks"), Data::Float(10.0), s("01.01.2026")],
            vec![s("Gold"), Data::Empty, Data::Float(5.0), s("01.01.2026")],
        ];
        let p = parse_asset_rows(&rows, Currency::Usd);
        assert_eq!(p.ignored, 1);
        assert_eq!(p.items, vec![snap("ETF", "Stocks", 1_000, Currency::Usd, d(2026, 1, 1))]);
    }

    #[test]
    fn missing_file_is_empty() {
        let l = load(Path::new("Z:/definitely/missing/balance.xlsx"), Currency::Usd).unwrap();
        assert_eq!(l.ledger, Ledger::default());
    }
}

/// Writes sample data for manual UI testing:
/// `BALANCE_DB=<dir> cargo test seed_sample -- --ignored`
#[cfg(test)]
#[test]
#[ignore]
fn seed_sample() {
    let dir = PathBuf::from(std::env::var_os("BALANCE_DB").expect("set BALANCE_DB"));
    let today = chrono::Local::now().date_naive();
    let ago = |n: u64| today.checked_sub_days(chrono::Days::new(n)).unwrap();
    let a = |name: &str, value: i64, date| Action { name: name.into(), value, currency: Currency::Usd, date };
    let eur = |name: &str, value: i64, date| Action { currency: Currency::Eur, ..a(name, value, date) };
    let ledger = Ledger {
        incomes: vec![
            a("Savings", 2_000_000, ago(400)),
            a("Salary", 450_000, ago(40)),
            a("Salary", 450_000, ago(10)),
            eur("Freelance", 125_050, ago(3)),
            a("Bonus", 100_000, today.checked_add_days(chrono::Days::new(20)).unwrap()),
        ],
        expenses: vec![
            a("Rent", 180_000, ago(38)),
            a("Rent", 180_000, ago(8)),
            a("Groceries", 32_475, ago(5)),
            a("Coffee", 450, ago(0)),
            a("Gift", 0, ago(2)),
        ],
        assets: {
            let s = |name: &str, category: &str, value: i64, date| Snapshot {
                name: name.into(),
                category: category.into(),
                value,
                currency: Currency::Usd,
                date,
            };
            // Gold in grams (hundredths), silver too.
            let g = |name: &str, metal: Currency, grams: i64, date| Snapshot {
                currency: metal,
                ..s(name, "Metals", grams, date)
            };
            vec![
                s("World ETF", "Stocks", 1_200_000, ago(200)),
                s("World ETF", "Stocks", 1_380_000, ago(20)),
                s("World ETF", "Stocks", 1_452_000, ago(1)),
                s("Tech shares", "Stocks", 640_000, ago(90)),
                g("Gold coins", Currency::Xau, 2_500, ago(300)),
                g("Gold coins", Currency::Xau, 3_250, ago(15)),
                g("Silver bar", Currency::Xag, 100_000, ago(30)),
                Snapshot { currency: Currency::Btc, ..s("Cold wallet", "Crypto", 4_250_000, ago(10)) },
                Snapshot { currency: Currency::Eth, ..s("Cold wallet ETH", "Crypto", 150_000_000, ago(10)) },
                s("Bitcoin", "Crypto", 255_000, ago(60)),
                s("Bitcoin", "Crypto", 198_000, ago(4)),
                s("Car", "Vehicles", 900_000, ago(365)),
            ]
        },
    };
    let path = dir.join(WORKBOOK_FILE);
    save(&path, &ledger).unwrap();
    // `BALANCE_SEED_BAD=1` appends a malformed row to test the ignored-rows guard.
    if std::env::var_os("BALANCE_SEED_BAD").is_some() {
        let mut book = umya_spreadsheet::reader::xlsx::read(&path).unwrap();
        let ws = book.sheet_by_name_mut("Incomes").unwrap();
        let row = ws.highest_row() + 1;
        ws.cell_mut((1, row)).set_value_string("broken");
        ws.cell_mut((2, row)).set_value_string("abc");
        ws.cell_mut((3, row)).set_value_string("09.11.2026");
        umya_spreadsheet::writer::xlsx::write(&book, &path).unwrap();
    }
}
