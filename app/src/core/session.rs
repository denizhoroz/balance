//! Editing session over the workbook: the rows being edited, which field is
//! open, and the rules that keep the user's data safe (incomplete rows are
//! dropped, saves are gated, disk wins on external change).
//!
//! The UI forwards input events here, passing the text of its single editor
//! input, and applies the returned [`Step`]. Nothing here depends on gpui.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use chrono::NaiveDate;

use super::assets::{self, Snapshot};
use super::currency::Currency;
use super::date::{format_date_full, parse_date};
use super::ledger::{Action, Kind, Ledger};
use super::money::{Cents, format_plain_dec, parse_amount_dec, rescale};
use super::store::{self, SaveError};

/// An editable list of rows.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum List {
    Income,
    Expense,
    Asset,
}

impl List {
    /// Currencies a row of this list may be in: metals and crypto for
    /// assets only.
    pub fn currencies(self) -> &'static [Currency] {
        match self {
            List::Income | List::Expense => &Currency::FIAT,
            List::Asset => &Currency::ALL,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Field {
    Name,
    Category,
    Date,
    Value,
    /// Picked from the list's currencies; the editor text is a code prefix.
    Currency,
    /// A saved holding's Add / Remove amount (asset view). Not a row value:
    /// its text is kept by the session until [`Session::adjust`] applies it.
    Amount(Adjust),
}

/// Which way an amount changes a holding.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Adjust {
    Add,
    Remove,
}

/// One table row. Rows may be incomplete while being edited; only complete
/// rows (name + value, plus category for assets) are stored.
pub struct Row {
    pub id: u64,
    pub name: String,
    /// Assets only.
    pub category: String,
    pub value: Option<Cents>,
    pub currency: Currency,
    pub date: NaiveDate,
    /// New row whose currency the user has not been asked for yet.
    ask_currency: bool,
}

/// Tab order of an income/expense row in the Table. Currency is not in it:
/// existing rows change it with the row dropdown ([`Session::set_currency`]);
/// a new row is asked for it in its Enter flow, just before the value (so
/// the value is parsed with the currency's decimals).
const ACTION_FIELDS: [Field; 3] = [Field::Name, Field::Date, Field::Value];

/// Fields of a saved asset that can still change, for its whole holding,
/// then its Add / Remove amounts. Value, type and date are fixed once the
/// asset is assigned: the amounts change the holding by a new snapshot.
const HOLDING_FIELDS: [Field; 4] = [Field::Name, Field::Category, Field::Amount(Adjust::Add), Field::Amount(Adjust::Remove)];

/// Field order of a draft asset, as laid out in the asset view: name and
/// category on the left, then type (currency), value and date on the right.
/// The date defaults to today, so Enter never stops on it; Tab does.
const DRAFT_FIELDS: [Field; 5] = [Field::Name, Field::Category, Field::Currency, Field::Value, Field::Date];

/// How a row behaves while edited. Every editing rule that differs between
/// rows is decided here, from the row's kind.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum RowKind {
    /// An income or expense in the Table: edited inline; Tab moves on to
    /// the next row; an incomplete row is dropped when focus leaves it.
    Action,
    /// A new asset in the asset view: every field editable; kept while
    /// incomplete until the view is left.
    Draft,
    /// A saved asset: only its holding's name and category change, for every
    /// snapshot of the holding; value, type and date are fixed. Its Add /
    /// Remove amounts append a new snapshot instead.
    Saved,
}

impl RowKind {
    /// Fields Tab moves through, in order.
    fn fields(self) -> &'static [Field] {
        match self {
            RowKind::Action => &ACTION_FIELDS,
            RowKind::Draft => &DRAFT_FIELDS,
            RowKind::Saved => &HOLDING_FIELDS,
        }
    }

    /// Whether `field` may be opened at all.
    fn opens(self, field: Field) -> bool {
        match self {
            // The currency step sits in the Enter flow, outside Tab order.
            RowKind::Action => ACTION_FIELDS.contains(&field) || field == Field::Currency,
            RowKind::Draft | RowKind::Saved => self.fields().contains(&field),
        }
    }

    /// Where Enter goes next: the first field still to fill. A new row's
    /// type counts as missing until the user has been asked for it: an
    /// action asks right before its value, a draft in view order.
    fn next_missing(self, row: &Row, list: List) -> Option<Field> {
        match self {
            RowKind::Action => match row.missing(list) {
                Some(Field::Value) | None if row.ask_currency => Some(Field::Currency),
                missing => missing,
            },
            RowKind::Draft => DRAFT_FIELDS.into_iter().find(|f| match f {
                Field::Name => row.name.is_empty(),
                Field::Currency => row.ask_currency,
                Field::Category => row.category.is_empty(),
                Field::Value => row.value.is_none(),
                Field::Date | Field::Amount(_) => false,
            }),
            RowKind::Saved => None,
        }
    }
}

impl Row {
    fn missing(&self, list: List) -> Option<Field> {
        if self.name.is_empty() {
            Some(Field::Name)
        } else if list == List::Asset && self.category.is_empty() {
            Some(Field::Category)
        } else if self.value.is_none() {
            Some(Field::Value)
        } else {
            None
        }
    }

    fn action(&self) -> Option<Action> {
        if self.missing(List::Income).is_some() {
            return None;
        }
        Some(Action { name: self.name.clone(), value: self.value?, currency: self.currency, date: self.date })
    }

    fn snapshot(&self) -> Option<Snapshot> {
        if self.missing(List::Asset).is_some() {
            return None;
        }
        Some(Snapshot {
            name: self.name.clone(),
            category: self.category.clone(),
            value: self.value?,
            currency: self.currency,
            date: self.date,
        })
    }
}

/// What the asset view is open on. Owned by the session: the UI only asks
/// [`Session::asset_view`] what to draw.
#[derive(Clone, PartialEq, Eq, Debug)]
enum Detail {
    /// A new asset row being filled. Kept out of the ledger (so never saved)
    /// and exempt from the row-leave check until the view is left.
    Draft(u64),
    /// A saved asset row: only its holding's name and category can change.
    Row(u64),
}

/// What the asset view draws; `None` fields show their placeholder.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AssetView {
    pub name: Option<String>,
    pub currency: Option<Currency>,
    pub category: Option<String>,
    pub value: Option<Cents>,
    pub date: Option<NaiveDate>,
    /// Asset row the editable fields write to.
    pub row: Option<u64>,
    /// Fields the user may open, in Tab order.
    pub editable: &'static [Field],
}

impl AssetView {
    fn of_row(r: &Row, editable: &'static [Field]) -> AssetView {
        AssetView {
            name: Some(r.name.clone()).filter(|s| !s.is_empty()),
            currency: Some(r.currency),
            category: Some(r.category.clone()).filter(|s| !s.is_empty()),
            value: r.value,
            date: Some(r.date),
            row: Some(r.id),
            editable,
        }
    }
}

/// The field currently open in the edit popup.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Edit {
    pub list: List,
    pub id: u64,
    pub field: Field,
}

/// What the editor input should show when a field opens.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Prefill {
    pub text: String,
    pub placeholder: &'static str,
}

/// What the UI should do after an input event.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Step {
    /// Show the editor on this field, filled with the prefill.
    Open(Edit, Prefill),
    /// Close the editor and return focus to the view.
    Close,
}

type DiskStamp = Option<(SystemTime, u64)>;

fn disk_stamp(path: &Path) -> DiskStamp {
    let m = std::fs::metadata(path).ok()?;
    Some((m.modified().ok()?, m.len()))
}

pub struct Session {
    path: PathBuf,
    incomes: Vec<Row>,
    expenses: Vec<Row>,
    assets: Vec<Row>,
    /// Ledger built from the complete rows; rebuilt on every row change.
    ledger: Ledger,
    /// Row id of each `ledger.assets` snapshot (same order).
    asset_ids: Vec<u64>,
    /// Ledger as last loaded from / written to disk.
    saved: Ledger,
    /// Bad rows skipped while loading; saving is blocked while > 0.
    ignored: usize,
    /// Workbook exists but could not be read; saving is blocked.
    unreadable: bool,
    disk_stamp: DiskStamp,
    next_id: u64,
    editing: Option<Edit>,
    /// What the asset view is open on, if it is.
    detail: Option<Detail>,
    /// Text of the asset view's Add and Remove amounts; cleared when the
    /// view opens, closes, or applies one.
    amounts: [String; 2],
    /// Currency for loaded rows that have none (legacy workbooks).
    default_currency: Currency,
    /// The last load filled in default currencies; rewrite the workbook so
    /// they are explicit (subject to the same save gate).
    migrate: bool,
}

impl Session {
    /// Opens the session on the workbook at `path` (a missing file is empty).
    /// Rows without a currency take `default_currency`.
    pub fn new(path: PathBuf, default_currency: Currency) -> Self {
        let mut s = Self {
            path,
            incomes: Vec::new(),
            expenses: Vec::new(),
            assets: Vec::new(),
            ledger: Ledger::default(),
            asset_ids: Vec::new(),
            saved: Ledger::default(),
            ignored: 0,
            unreadable: false,
            disk_stamp: None,
            next_id: 0,
            editing: None,
            detail: None,
            amounts: Default::default(),
            default_currency,
            migrate: false,
        };
        s.reload();
        s
    }

    // MARK: read

    pub fn rows(&self, list: List) -> &Vec<Row> {
        match list {
            List::Income => &self.incomes,
            List::Expense => &self.expenses,
            List::Asset => &self.assets,
        }
    }

    fn rows_mut(&mut self, list: List) -> &mut Vec<Row> {
        match list {
            List::Income => &mut self.incomes,
            List::Expense => &mut self.expenses,
            List::Asset => &mut self.assets,
        }
    }

    /// Ledger built from the complete rows.
    pub fn ledger(&self) -> &Ledger {
        &self.ledger
    }

    /// In-memory edits not yet on disk.
    pub fn unsaved(&self) -> bool {
        self.ledger != self.saved
    }

    pub fn ignored(&self) -> usize {
        self.ignored
    }

    pub fn unreadable(&self) -> bool {
        self.unreadable
    }

    pub fn editing(&self) -> Option<Edit> {
        self.editing
    }

    /// What the asset view should draw, or `None` when it has nothing to
    /// show (never opened, left, or its asset is gone after a reload or
    /// delete): the UI then closes the view.
    pub fn asset_view(&self) -> Option<AssetView> {
        let row = |id: u64| self.row_index(List::Asset, id).map(|ix| &self.assets[ix]);
        match self.detail.as_ref()? {
            Detail::Draft(id) => row(*id).map(|r| AssetView::of_row(r, &DRAFT_FIELDS)),
            Detail::Row(id) => row(*id).map(|r| AssetView::of_row(r, &HOLDING_FIELDS)),
        }
    }

    /// The draft row, if the asset view is open on one.
    fn draft_id(&self) -> Option<u64> {
        match self.detail {
            Some(Detail::Draft(id)) => Some(id),
            _ => None,
        }
    }

    /// The text kept for an Add / Remove amount (shown while its editor is
    /// closed).
    pub fn amount_text(&self, dir: Adjust) -> &str {
        &self.amounts[dir as usize]
    }

    /// The holding's value after applying `text` as a `dir` amount on
    /// `today`, or `None` when it cannot be applied: no saved asset in view,
    /// an empty, invalid or zero amount, or removing more than is held.
    pub fn adjusted(&self, dir: Adjust, text: &str, today: NaiveDate) -> Option<Cents> {
        let Some(Detail::Row(id)) = self.detail else { return None };
        let row = &self.assets[self.row_index(List::Asset, id)?];
        let amount = parse_amount_dec(text, row.currency.decimals()).filter(|&v| v > 0)?;
        // What the holding is worth today (nothing if not held by then).
        let held = assets::current(&self.ledger.assets, today)
            .into_iter()
            .map(|i| &self.ledger.assets[i])
            .find(|s| s.name == row.name)
            .map_or(0, |s| s.value);
        match dir {
            Adjust::Add => held.checked_add(amount),
            Adjust::Remove => held.checked_sub(amount).filter(|&v| v >= 0),
        }
    }

    #[cfg(test)]
    fn draft(&self) -> Option<&Row> {
        let id = self.draft_id()?;
        self.row_index(List::Asset, id).map(|ix| &self.assets[ix])
    }

    /// Currency given to rows without one on future reloads.
    pub fn set_default_currency(&mut self, currency: Currency) {
        self.default_currency = currency;
    }

    // MARK: disk

    fn rebuild(&mut self) {
        // The draft joins the ledger (and so the saved file) only once the
        // asset view is left.
        let draft = self.draft_id();
        let (asset_ids, assets) = self
            .assets
            .iter()
            .filter(|r| Some(r.id) != draft)
            .filter_map(|r| Some((r.id, r.snapshot()?)))
            .unzip();
        self.asset_ids = asset_ids;
        self.ledger = Ledger {
            incomes: self.incomes.iter().filter_map(Row::action).collect(),
            expenses: self.expenses.iter().filter_map(Row::action).collect(),
            assets,
        };
    }

    /// Id of the asset row holding `name`'s value on `day` (see
    /// [`assets::current`]), if it is held by then.
    pub fn holding_row(&self, name: &str, day: NaiveDate) -> Option<u64> {
        assets::current(&self.ledger.assets, day)
            .into_iter()
            .find(|&i| self.ledger.assets[i].name == name)
            .map(|i| self.asset_ids[i])
    }

    /// Loads the workbook from disk, replacing all rows (disk wins) and
    /// closing any open edit.
    pub fn reload(&mut self) {
        let stamp = disk_stamp(&self.path);
        match store::load(&self.path, self.default_currency) {
            Ok(loaded) => {
                self.unreadable = false;
                self.ignored = loaded.ignored;
                self.saved = loaded.ledger.clone();
                self.editing = None;
                // Row ids are renumbered, so the asset view's row is gone.
                self.detail = None;
                for (kind, list) in [(Kind::Income, List::Income), (Kind::Expense, List::Expense)] {
                    let rows = loaded
                        .ledger
                        .list(kind)
                        .iter()
                        .map(|a| self.make_row(a.name.clone(), String::new(), Some(a.value), a.currency, a.date))
                        .collect();
                    *self.rows_mut(list) = rows;
                }
                self.assets = loaded
                    .ledger
                    .assets
                    .iter()
                    .map(|a| self.make_row(a.name.clone(), a.category.clone(), Some(a.value), a.currency, a.date))
                    .collect();
                self.rebuild();
                self.disk_stamp = stamp;
                self.migrate = loaded.defaulted;
                self.persist();
            }
            Err(e) => {
                eprintln!("balance: failed to read workbook: {e}");
                self.unreadable = true;
                self.disk_stamp = stamp;
            }
        }
    }

    /// Writes the ledger if it differs from disk. Blocked while rows are
    /// ignored or the file is unreadable, so the user's data is never lost.
    fn persist(&mut self) {
        if !(self.unsaved() || self.migrate) || self.ignored > 0 || self.unreadable {
            return;
        }
        match store::save(&self.path, &self.ledger) {
            Ok(()) => {
                self.saved = self.ledger.clone();
                self.migrate = false;
                self.disk_stamp = disk_stamp(&self.path);
            }
            Err(SaveError::Locked) => {}
            Err(e) => eprintln!("balance: failed to save workbook: {e}"),
        }
    }

    /// Something changed in the db folder (or the window refocused): reload
    /// if the workbook changed on disk, otherwise retry a pending save.
    /// Returns whether anything changed.
    pub fn sync(&mut self) -> bool {
        if disk_stamp(&self.path) != self.disk_stamp {
            self.reload();
            true
        } else if self.unsaved() || self.migrate {
            let before = self.unsaved();
            self.persist();
            before != self.unsaved()
        } else {
            false
        }
    }

    // MARK: rows

    fn make_row(
        &mut self,
        name: String,
        category: String,
        value: Option<Cents>,
        currency: Currency,
        date: NaiveDate,
    ) -> Row {
        let id = self.next_id;
        self.next_id += 1;
        Row { id, name, category, value, currency, date, ask_currency: false }
    }

    fn row_index(&self, list: List, id: u64) -> Option<usize> {
        self.rows(list).iter().position(|r| r.id == id)
    }

    /// Adds an empty income/expense row dated `today` in `currency`
    /// (preselected) and opens its name field; the Enter flow ends by asking
    /// for the currency. `editor_text` is the text of the edit that was open
    /// (if any). Assets are added with [`Session::open_draft`] instead.
    pub fn add_row(&mut self, list: List, today: NaiveDate, currency: Currency, editor_text: &str) -> Step {
        if list == List::Asset {
            debug_assert!(false, "assets are added with open_draft");
            return Step::Close;
        }
        let mut row = self.make_row(String::new(), String::new(), None, currency, today);
        row.ask_currency = true;
        let edit = Edit { list, id: row.id, field: Field::Name };
        self.rows_mut(list).push(row);
        let prefill = self.start_edit(edit, editor_text);
        Self::step(edit, prefill)
    }

    // MARK: asset view

    /// Opens the asset view on a new asset dated `today` in `currency`
    /// (preselected), with its name field open. It stays while incomplete
    /// and is saved only by [`Session::leave_asset_view`]; its type is asked
    /// for after the category.
    pub fn open_draft(&mut self, today: NaiveDate, currency: Currency, editor_text: &str) -> Step {
        self.leave_asset_view(editor_text);
        let mut row = self.make_row(String::new(), String::new(), None, currency, today);
        row.ask_currency = true;
        let edit = Edit { list: List::Asset, id: row.id, field: Field::Name };
        self.detail = Some(Detail::Draft(row.id));
        self.assets.push(row);
        let prefill = self.open(edit);
        Self::step(edit, prefill)
    }

    /// Opens the asset view on a saved asset row.
    pub fn open_asset(&mut self, id: u64, editor_text: &str) {
        self.leave_asset_view(editor_text);
        if self.row_index(List::Asset, id).is_some() {
            self.detail = Some(Detail::Row(id));
        }
    }

    /// Leaves the asset view: commits the open field; a draft is then kept
    /// and saved if complete, or dropped.
    pub fn leave_asset_view(&mut self, editor_text: &str) {
        if self.editing.is_some() {
            self.commit_and_close(editor_text);
        }
        self.amounts = Default::default();
        if let Some(Detail::Draft(id)) = self.detail.take() {
            self.leave_row(List::Asset, id);
        }
    }

    /// Clear (×) next to an Add / Remove amount: empties its text, closing
    /// its editor if open.
    pub fn clear_amount(&mut self, dir: Adjust) {
        if self.editing.is_some_and(|e| e.field == Field::Amount(dir)) {
            self.end_edit();
        }
        self.amounts[dir as usize].clear();
    }

    /// Add / Remove pressed (button or Enter in its amount): commits the open
    /// field, then records the holding's new value as a new snapshot dated
    /// `today`, so earlier days keep their values. The view moves to the new
    /// row, or closes when nothing is left (the holding is sold). An amount
    /// that cannot be applied (see [`Session::adjusted`]) changes nothing.
    pub fn adjust(&mut self, dir: Adjust, today: NaiveDate, editor_text: &str) -> Step {
        if self.editing.is_some() {
            self.commit_and_close(editor_text);
        }
        let Some(value) = self.adjusted(dir, self.amount_text(dir), today) else { return Step::Close };
        let Some(Detail::Row(id)) = self.detail else { return Step::Close };
        let Some(ix) = self.row_index(List::Asset, id) else { return Step::Close };
        let (name, category, currency) = {
            let r = &self.assets[ix];
            (r.name.clone(), r.category.clone(), r.currency)
        };
        let row = self.make_row(name, category, Some(value), currency, today);
        let new = row.id;
        self.assets.push(row);
        // Stable: after the day's existing rows, so this one wins the day.
        self.assets.sort_by_key(|r| r.date);
        self.amounts = Default::default();
        self.detail = (value != 0).then_some(Detail::Row(new));
        self.rebuild();
        self.persist();
        Step::Close
    }

    /// The editing rules row `id` of `list` follows.
    fn kind(&self, list: List, id: u64) -> RowKind {
        match list {
            List::Income | List::Expense => RowKind::Action,
            List::Asset if self.draft_id() == Some(id) => RowKind::Draft,
            List::Asset => RowKind::Saved,
        }
    }

    /// Sets a row's currency (from the dropdown) and saves. The shown
    /// number stays the same (`1,50` USD becomes `1,50` BTC).
    /// A saved asset's type is fixed: ignored.
    pub fn set_currency(&mut self, list: List, id: u64, currency: Currency) {
        if self.kind(list, id) == RowKind::Saved {
            return;
        }
        let Some(ix) = self.row_index(list, id) else { return };
        Self::change_currency(&mut self.rows_mut(list)[ix], currency);
        self.rebuild();
        self.persist();
    }

    fn change_currency(row: &mut Row, currency: Currency) {
        let (from, to) = (row.currency.decimals(), currency.decimals());
        if let Some(v) = row.value {
            row.value = rescale(v, from, to);
        }
        row.currency = currency;
    }

    pub fn delete_row(&mut self, list: List, id: u64) {
        if self.editing.is_some_and(|e| e.list == list && e.id == id) {
            self.editing = None;
        }
        if list == List::Asset && matches!(self.detail, Some(Detail::Draft(d) | Detail::Row(d)) if d == id) {
            self.detail = None;
        }
        if let Some(ix) = self.row_index(list, id) {
            self.rows_mut(list).remove(ix);
            self.rebuild();
            self.persist();
        }
    }

    // MARK: edit cursor

    fn step(edit: Edit, prefill: Option<Prefill>) -> Step {
        match prefill {
            Some(p) => Step::Open(edit, p),
            None => Step::Close,
        }
    }

    /// Opens a field, committing whatever was being edited with `editor_text`.
    /// Returns `None` when there is nothing to open (already open, or the
    /// row is gone).
    pub fn start_edit(&mut self, edit: Edit, editor_text: &str) -> Option<Prefill> {
        if let Some(cur) = self.editing {
            if cur == edit {
                return None;
            }
            self.commit(editor_text);
            self.editing = None;
            if (cur.list, cur.id) != (edit.list, edit.id) {
                self.leave_row(cur.list, cur.id);
            }
        }
        self.open(edit)
    }

    fn open(&mut self, edit: Edit) -> Option<Prefill> {
        if !self.kind(edit.list, edit.id).opens(edit.field) {
            return None;
        }
        let ix = self.row_index(edit.list, edit.id)?;
        let row = &self.rows(edit.list)[ix];
        // Values are edited without thousands dots; dates in full form.
        let (text, placeholder) = match edit.field {
            Field::Name => (row.name.clone(), "name"),
            Field::Category => (row.category.clone(), "category"),
            Field::Date => (format_date_full(row.date), "DD.MM.YYYY"),
            Field::Value => (
                row.value.map(|v| format_plain_dec(v, row.currency.decimals())).unwrap_or_default(),
                "0",
            ),
            Field::Currency => (row.currency.code().to_string(), "USD"),
            Field::Amount(dir) => (self.amounts[dir as usize].clone(), "amount"),
        };
        self.editing = Some(edit);
        Some(Prefill { text, placeholder })
    }

    /// Writes the editor text into the open field's row. Invalid date, value
    /// or currency input is ignored (the row keeps its previous value).
    fn commit(&mut self, editor_text: &str) {
        let Some(edit) = self.editing else { return };
        let Some(ix) = self.row_index(edit.list, edit.id) else { return };
        let text = editor_text.trim().to_string();
        if let Field::Amount(dir) = edit.field {
            self.amounts[dir as usize] = text;
            return;
        }
        if self.kind(edit.list, edit.id) == RowKind::Saved {
            self.commit_holding(ix, edit.field, text);
            self.rebuild();
            return;
        }
        let row = &mut self.rows_mut(edit.list)[ix];
        match edit.field {
            Field::Name => row.name = text,
            Field::Category => row.category = text,
            Field::Date => {
                if let Some(d) = parse_date(&text) {
                    row.date = d;
                }
            }
            Field::Value => {
                if text.is_empty() {
                    row.value = None;
                } else if let Some(v) = parse_amount_dec(&text, row.currency.decimals()) {
                    row.value = Some(v);
                }
            }
            Field::Currency => {
                if let Some(c) = Currency::match_prefix(&text, edit.list.currencies()) {
                    Self::change_currency(row, c);
                }
                row.ask_currency = false;
            }
            Field::Amount(_) => unreachable!("amounts are kept above"),
        }
        self.rebuild();
    }

    /// A saved asset's name or category changes for its whole holding (every
    /// snapshot of that name). Other fields are fixed. An empty text, or a
    /// name another holding already has, is ignored (no silent merge).
    fn commit_holding(&mut self, ix: usize, field: Field, text: String) {
        let draft = self.draft_id();
        let old = self.assets[ix].name.clone();
        let holding = |r: &Row| r.name == old && Some(r.id) != draft;
        match field {
            Field::Name if !text.is_empty() && text != old => {
                if self.assets.iter().any(|r| r.name == text && Some(r.id) != draft) {
                    return;
                }
                self.assets.iter_mut().filter(|r| holding(r)).for_each(|r| r.name = text.clone());
            }
            Field::Category if !text.is_empty() => {
                self.assets.iter_mut().filter(|r| holding(r)).for_each(|r| r.category = text.clone());
            }
            _ => {}
        }
    }

    /// Closes the open field (no commit) and runs the row-leave check.
    fn end_edit(&mut self) {
        if let Some(cur) = self.editing.take() {
            self.leave_row(cur.list, cur.id);
        }
    }

    /// Commits the editor text and closes (focus lost, click outside, end of Tab).
    pub fn commit_and_close(&mut self, editor_text: &str) {
        self.commit(editor_text);
        self.end_edit();
    }

    /// Esc: drop the editor text. An Add / Remove amount keeps it instead
    /// (Esc only closes its editor; nothing is applied).
    pub fn cancel(&mut self, editor_text: &str) {
        if self.editing.is_some_and(|e| matches!(e.field, Field::Amount(_))) {
            self.commit(editor_text);
        }
        self.end_edit();
    }

    /// Enter: commit; on an incomplete row continue to the missing field
    /// instead of closing (closing would delete the row).
    pub fn enter(&mut self, editor_text: &str) -> Step {
        let Some(edit) = self.editing else { return Step::Close };
        self.commit(editor_text);
        let kind = self.kind(edit.list, edit.id);
        let next = self
            .row_index(edit.list, edit.id)
            .and_then(|ix| kind.next_missing(&self.rows(edit.list)[ix], edit.list));
        match next {
            Some(field) if field != edit.field => {
                let next = Edit { field, ..edit };
                let prefill = self.open(next);
                Self::step(next, prefill)
            }
            _ => {
                self.end_edit();
                Step::Close
            }
        }
    }

    /// Tab order: the row kind's fields left to right; actions then go on
    /// to the next row (and back), asset rows stay in their own fields.
    /// Past either end the editor closes.
    pub fn tab(&mut self, back: bool, editor_text: &str) -> Step {
        let Some(edit) = self.editing else { return Step::Close };
        let Some(ix) = self.row_index(edit.list, edit.id) else { return Step::Close };
        let kind = self.kind(edit.list, edit.id);
        if kind != RowKind::Action {
            return self.tab_within(edit, kind.fields(), back, editor_text);
        }
        let fields = kind.fields();
        let n = fields.len();
        // The currency step sits just before Value (the last field): Tab
        // goes on to the value, back-Tab to the field before it.
        let on_currency = edit.field == Field::Currency;
        let field_pos = if on_currency { n - 1 } else { fields.iter().position(|f| *f == edit.field).unwrap_or(0) };
        let pos = ix * n + field_pos;
        let total = self.rows(edit.list).len() * n;
        let next = match (back, on_currency) {
            (false, true) => Some(pos),
            (true, _) => pos.checked_sub(1),
            (false, false) => Some(pos + 1).filter(|p| *p < total),
        };
        match next {
            Some(p) => {
                let id = self.rows(edit.list)[p / n].id;
                let target = Edit { list: edit.list, id, field: fields[p % n] };
                let prefill = self.start_edit(target, editor_text);
                if prefill.is_none() {
                    self.commit_and_close(editor_text);
                }
                Self::step(target, prefill)
            }
            None => {
                self.commit_and_close(editor_text);
                Step::Close
            }
        }
    }

    /// Tab within one row's `fields` only (the asset view); past either end
    /// the editor closes.
    fn tab_within(&mut self, edit: Edit, fields: &[Field], back: bool, editor_text: &str) -> Step {
        let pos = fields.iter().position(|f| *f == edit.field).unwrap_or(0);
        let next = if back { pos.checked_sub(1) } else { Some(pos + 1).filter(|p| *p < fields.len()) };
        match next {
            Some(p) => {
                let target = Edit { field: fields[p], ..edit };
                let prefill = self.start_edit(target, editor_text);
                Self::step(target, prefill)
            }
            None => {
                self.commit_and_close(editor_text);
                Step::Close
            }
        }
    }

    /// Focus left the row: delete it if incomplete, re-sort and save. The
    /// draft is exempt until the asset view is left.
    fn leave_row(&mut self, list: List, id: u64) {
        if self.kind(list, id) == RowKind::Draft {
            return;
        }
        let Some(ix) = self.row_index(list, id) else { return };
        if self.rows(list)[ix].missing(list).is_some() {
            self.rows_mut(list).remove(ix);
        }
        self.rows_mut(list).sort_by_key(|r| r.date);
        self.rebuild();
        self.persist();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::store::WORKBOOK_FILE;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    fn today() -> NaiveDate {
        d(2026, 11, 9)
    }

    /// Fresh scratch workbook path (folder removed first).
    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("balance-session-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir.join(WORKBOOK_FILE)
    }

    fn cleanup(path: &Path) {
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    fn open_of(step: Step) -> (Edit, Prefill) {
        match step {
            Step::Open(e, p) => (e, p),
            Step::Close => panic!("expected Open, got Close"),
        }
    }

    /// Adds a complete row via the same event sequence the UI uses,
    /// keeping the preselected currency at the currency step.
    fn add_complete(s: &mut Session, list: List, name: &str, value: &str) {
        let (_, _) = open_of(s.add_row(list, today(), Currency::Usd, ""));
        let (e, p) = open_of(s.enter(name));
        assert_eq!((e.field, p.text.as_str()), (Field::Currency, "USD"));
        let (e, _) = open_of(s.enter(&p.text));
        assert_eq!(e.field, Field::Value);
        assert_eq!(s.enter(value), Step::Close);
    }

    fn act(name: &str, value: Cents, date: NaiveDate) -> Action {
        Action { name: name.into(), value, currency: Currency::Usd, date }
    }

    #[test]
    fn draft_enter_follows_view_order_and_saves_only_on_leave() {
        let path = scratch("draft-enter");
        let mut s = Session::new(path.clone(), Currency::Usd);
        let (e, _) = open_of(s.open_draft(today(), Currency::Usd, ""));
        assert_eq!(e.field, Field::Name);
        let (e, _) = open_of(s.enter("Gold"));
        assert_eq!(e.field, Field::Category);
        let (e, p) = open_of(s.enter("Metals"));
        assert_eq!((e.field, p.text.as_str()), (Field::Currency, "USD"));
        let (e, _) = open_of(s.enter("xau"));
        assert_eq!(e.field, Field::Value);
        assert_eq!(s.enter("12,5"), Step::Close);
        // Complete but not in the ledger (so not saved, even by a sync)
        // until the view closes. A past date declares an earlier investment.
        let id = s.draft().unwrap().id;
        s.start_edit(Edit { list: List::Asset, id, field: Field::Date }, "");
        assert_eq!(s.enter("01.03.2025"), Step::Close);
        let view = s.asset_view().unwrap();
        assert_eq!((view.currency, view.date, view.editable), (Some(Currency::Xau), Some(d(2025, 3, 1)), &DRAFT_FIELDS[..]));
        assert!(s.ledger().assets.is_empty());
        assert!(!s.sync());
        assert!(store::load(&path, Currency::Usd).unwrap().ledger.assets.is_empty());
        s.leave_asset_view("");
        assert_eq!(s.asset_view(), None);
        assert_eq!(s.ledger().assets[0].date, d(2025, 3, 1));
        assert!(!s.unsaved());
        assert_eq!(store::load(&path, Currency::Usd).unwrap().ledger.assets.len(), 1);
        cleanup(&path);
    }

    #[test]
    fn incomplete_draft_survives_until_finished() {
        let path = scratch("draft-keep");
        let mut s = Session::new(path.clone(), Currency::Usd);
        open_of(s.open_draft(today(), Currency::Usd, ""));
        // Closing the field (click away / Esc) keeps the incomplete draft.
        s.commit_and_close("ETF");
        assert_eq!(s.draft().map(|r| r.name.as_str()), Some("ETF"));
        let id = s.draft().unwrap().id;
        assert!(s.start_edit(Edit { list: List::Asset, id, field: Field::Value }, "").is_some());
        s.cancel("");
        assert!(s.draft().is_some());
        s.leave_asset_view("");
        assert!(s.rows(List::Asset).is_empty());
        cleanup(&path);
    }

    #[test]
    fn draft_tab_stays_in_its_fields() {
        let path = scratch("draft-tab");
        let mut s = Session::new(path.clone(), Currency::Usd);
        let (e, _) = open_of(s.open_draft(today(), Currency::Usd, ""));
        let mut fields = vec![e.field];
        let mut step = s.tab(false, "A");
        while let Step::Open(e, _) = step {
            fields.push(e.field);
            step = s.tab(false, "");
        }
        assert_eq!(fields, DRAFT_FIELDS);
        assert_eq!(s.editing(), None);
        // Back-Tab from the name closes too.
        let id = s.draft().unwrap().id;
        s.start_edit(Edit { list: List::Asset, id, field: Field::Name }, "");
        assert_eq!(s.tab(true, "A"), Step::Close);
        assert!(s.draft().is_some());
        cleanup(&path);
    }

    #[test]
    fn asset_view_on_saved_row() {
        let path = scratch("asset-view");
        let mut s = Session::new(path.clone(), Currency::Usd);
        open_of(s.open_draft(today(), Currency::Usd, ""));
        for text in ["ETF", "Stocks", "usd"] {
            open_of(s.enter(text));
        }
        assert_eq!(s.enter("10"), Step::Close);
        s.leave_asset_view("");
        let id = s.rows(List::Asset)[0].id;

        // Saved row: name and category only; value, type and date stay shut.
        s.open_asset(id, "");
        assert_eq!(s.asset_view().unwrap().editable, &HOLDING_FIELDS[..]);
        for field in [Field::Value, Field::Currency, Field::Date] {
            assert_eq!(s.start_edit(Edit { list: List::Asset, id, field }, ""), None);
        }
        assert!(s.start_edit(Edit { list: List::Asset, id, field: Field::Category }, "").is_some());
        s.leave_asset_view("Funds");
        assert_eq!(s.ledger().assets[0].category, "Funds");

        // A row does not survive a reload (ids are renumbered).
        s.open_asset(s.rows(List::Asset)[0].id, "");
        s.reload();
        assert_eq!(s.asset_view(), None);
        cleanup(&path);
    }

    #[test]
    fn holding_row_follows_the_holding_rule() {
        let path = scratch("current-rows");
        let mut s = Session::new(path.clone(), Currency::Usd);
        let add = |s: &mut Session, name: &str, value: &str, date: &str| {
            open_of(s.open_draft(today(), Currency::Usd, ""));
            for text in [name, "Stocks", "usd", value] {
                s.enter(text);
            }
            let id = s.draft().unwrap().id;
            s.start_edit(Edit { list: List::Asset, id, field: Field::Date }, "");
            s.enter(date);
            s.leave_asset_view("");
            id
        };
        let old = add(&mut s, "ETF", "10", "01.01.2026");
        let tie_first = add(&mut s, "ETF", "11", "01.06.2026");
        let tie_last = add(&mut s, "ETF", "12", "01.06.2026"); // same date: later row wins
        let future = add(&mut s, "ETF", "99", "01.12.2026");
        let gold = add(&mut s, "Gold", "5", "01.03.2026");
        assert_eq!(s.holding_row("ETF", today()), Some(tie_last));
        assert_eq!(s.holding_row("Gold", today()), Some(gold));
        assert_eq!(s.holding_row("Bond", today()), None);
        for id in [old, tie_first, future] {
            assert_ne!(s.holding_row("ETF", today()), Some(id));
        }
        cleanup(&path);
    }

    /// Saves a holding dated `date` through the draft flow and opens the
    /// asset view on it; returns its row id.
    fn open_holding(s: &mut Session, name: &str, value: &str, date: &str) -> u64 {
        open_of(s.open_draft(today(), Currency::Usd, ""));
        for text in [name, "Stocks", "usd", value] {
            s.enter(text);
        }
        let id = s.draft().unwrap().id;
        s.start_edit(Edit { list: List::Asset, id, field: Field::Date }, "");
        s.enter(date);
        s.leave_asset_view("");
        s.open_asset(id, "");
        id
    }

    /// Types `text` into a box's amount and presses its button.
    fn press(s: &mut Session, dir: Adjust, text: &str) -> Step {
        let id = s.asset_view().unwrap().row.unwrap();
        assert!(s.start_edit(Edit { list: List::Asset, id, field: Field::Amount(dir) }, "").is_some());
        s.adjust(dir, today(), text)
    }

    fn values(s: &Session) -> Vec<(Cents, NaiveDate)> {
        s.ledger().assets.iter().map(|a| (a.value, a.date)).collect()
    }

    #[test]
    fn add_and_remove_append_snapshots_dated_today() {
        let path = scratch("adjust");
        let mut s = Session::new(path.clone(), Currency::Usd);
        open_holding(&mut s, "ETF", "10", "01.01.2026");
        assert_eq!(press(&mut s, Adjust::Add, "5,50"), Step::Close);
        // Same day again: another row, the later one wins.
        assert_eq!(press(&mut s, Adjust::Remove, "3"), Step::Close);
        assert_eq!(values(&s), [(1_000, d(2026, 1, 1)), (1_550, today()), (1_250, today())]);
        // Earlier days keep their value; the view follows the newest row.
        assert_eq!(assets::holdings_total(&s.ledger().assets, d(2026, 6, 1), crate::core::rates::Fx::new(Currency::Usd, None)), Some(1_000));
        let view = s.asset_view().unwrap();
        assert_eq!((view.value, view.date, view.row), (Some(1_250), Some(today()), s.holding_row("ETF", today())));
        assert_eq!(s.amount_text(Adjust::Add), "");
        assert!(!s.unsaved());
        assert_eq!(store::load(&path, Currency::Usd).unwrap().ledger.assets.len(), 3);
        cleanup(&path);
    }

    #[test]
    fn unusable_amounts_change_nothing() {
        let path = scratch("adjust-bad");
        let mut s = Session::new(path.clone(), Currency::Usd);
        open_holding(&mut s, "ETF", "10", "01.01.2026");
        for text in ["", "abc", "0", "10,01"] {
            assert_eq!(press(&mut s, Adjust::Remove, text), Step::Close);
            assert_eq!(s.adjusted(Adjust::Remove, text, today()), None);
        }
        assert_eq!(s.ledger().assets.len(), 1);
        // The typed text stays for another try.
        assert_eq!(s.amount_text(Adjust::Remove), "10,01");
        cleanup(&path);
    }

    #[test]
    fn removing_everything_sells_and_closes_the_view() {
        let path = scratch("adjust-sell");
        let mut s = Session::new(path.clone(), Currency::Usd);
        open_holding(&mut s, "ETF", "10", "01.01.2026");
        assert_eq!(press(&mut s, Adjust::Remove, "10"), Step::Close);
        assert_eq!(values(&s), [(1_000, d(2026, 1, 1)), (0, today())]);
        assert_eq!(s.asset_view(), None);
        cleanup(&path);
    }

    #[test]
    fn amount_text_survives_esc_and_click_away() {
        let path = scratch("adjust-keep");
        let mut s = Session::new(path.clone(), Currency::Usd);
        let id = open_holding(&mut s, "ETF", "10", "01.01.2026");
        let add = Edit { list: List::Asset, id, field: Field::Amount(Adjust::Add) };
        s.start_edit(add, "");
        s.cancel("2");
        assert_eq!(s.amount_text(Adjust::Add), "2");
        let p = s.start_edit(add, "").unwrap();
        assert_eq!((p.text.as_str(), p.placeholder), ("2", "amount"));
        s.commit_and_close("4");
        assert_eq!(s.amount_text(Adjust::Add), "4");
        // Nothing applied by either.
        assert_eq!(s.ledger().assets.len(), 1);
        // Clear empties it (closing its editor), and only its own box.
        s.start_edit(Edit { list: List::Asset, id, field: Field::Amount(Adjust::Remove) }, "");
        s.commit_and_close("1");
        s.start_edit(add, "");
        s.clear_amount(Adjust::Add);
        assert_eq!((s.amount_text(Adjust::Add), s.amount_text(Adjust::Remove), s.editing()), ("", "1", None));
        // Leaving the view forgets it.
        s.leave_asset_view("");
        assert_eq!(s.amount_text(Adjust::Remove), "");
        cleanup(&path);
    }

    #[test]
    fn draft_has_no_amounts() {
        let path = scratch("adjust-draft");
        let mut s = Session::new(path.clone(), Currency::Usd);
        open_of(s.open_draft(today(), Currency::Usd, ""));
        let id = s.draft().unwrap().id;
        for dir in [Adjust::Add, Adjust::Remove] {
            assert_eq!(s.start_edit(Edit { list: List::Asset, id, field: Field::Amount(dir) }, ""), None);
            assert_eq!(s.adjusted(dir, "1", today()), None);
        }
        assert!(!s.asset_view().unwrap().editable.contains(&Field::Amount(Adjust::Add)));
        cleanup(&path);
    }

    #[test]
    fn reload_drops_the_draft() {
        let path = scratch("draft-reload");
        let mut s = Session::new(path.clone(), Currency::Usd);
        open_of(s.open_draft(today(), Currency::Usd, ""));
        s.commit_and_close("ETF");
        s.reload();
        assert_eq!(s.asset_view(), None);
        assert!(s.rows(List::Asset).is_empty());
        cleanup(&path);
    }

    #[test]
    fn incomplete_row_dropped_on_leave() {
        let path = scratch("incomplete");
        let mut s = Session::new(path.clone(), Currency::Usd);
        open_of(s.add_row(List::Income, today(), Currency::Usd, ""));
        assert_eq!(s.rows(List::Income).len(), 1);
        // Enter on an empty name: still missing the same field, so close.
        assert_eq!(s.enter(""), Step::Close);
        assert!(s.rows(List::Income).is_empty());
        assert_eq!(s.editing(), None);
        cleanup(&path);
    }

    #[test]
    fn enter_continues_to_missing_field_then_saves() {
        let path = scratch("enter");
        let mut s = Session::new(path.clone(), Currency::Usd);
        open_of(s.add_row(List::Income, today(), Currency::Usd, ""));
        let (e, _) = open_of(s.enter("rent"));
        assert_eq!(e.field, Field::Currency);
        let (e, p) = open_of(s.enter("usd"));
        assert_eq!((e.field, p.placeholder), (Field::Value, "0"));
        assert_eq!(s.enter("12,50"), Step::Close);

        assert_eq!(s.ledger().incomes, vec![act("rent", 1_250, today())]);
        assert!(!s.unsaved());
        assert_eq!(store::load(&path, Currency::Usd).unwrap().ledger, *s.ledger());
        cleanup(&path);
    }

    #[test]
    fn draft_without_category_is_dropped() {
        let path = scratch("asset");
        let mut s = Session::new(path.clone(), Currency::Usd);
        open_of(s.open_draft(today(), Currency::Usd, ""));
        let (e, _) = open_of(s.enter("ETF"));
        assert_eq!(e.field, Field::Category);
        // Skip the category: Enter on it stays put, leaving drops the draft.
        assert_eq!(s.enter(""), Step::Close);
        s.leave_asset_view("");
        assert!(s.rows(List::Asset).is_empty());
        cleanup(&path);
    }

    #[test]
    fn leaving_row_sorts_by_date() {
        let path = scratch("sort");
        let mut s = Session::new(path.clone(), Currency::Usd);
        add_complete(&mut s, List::Income, "later", "1");
        add_complete(&mut s, List::Income, "earlier", "2");
        let id = s.rows(List::Income)[1].id;
        let prefill = s.start_edit(Edit { list: List::Income, id, field: Field::Date }, "").unwrap();
        assert_eq!(prefill.placeholder, "DD.MM.YYYY");
        s.commit_and_close("01.01.2020");
        let names: Vec<_> = s.rows(List::Income).iter().map(|r| r.name.as_str()).collect();
        assert_eq!(names, ["earlier", "later"]);
        cleanup(&path);
    }

    #[test]
    fn invalid_input_keeps_value_and_empty_value_drops_row() {
        let path = scratch("invalid");
        let mut s = Session::new(path.clone(), Currency::Usd);
        add_complete(&mut s, List::Income, "pay", "5");
        let id = s.rows(List::Income)[0].id;

        s.start_edit(Edit { list: List::Income, id, field: Field::Date }, "");
        s.commit_and_close("garbage");
        assert_eq!(s.rows(List::Income)[0].date, today());

        s.start_edit(Edit { list: List::Income, id, field: Field::Value }, "");
        s.commit_and_close("abc");
        assert_eq!(s.rows(List::Income)[0].value, Some(500));

        s.start_edit(Edit { list: List::Income, id, field: Field::Value }, "");
        s.commit_and_close("");
        assert!(s.rows(List::Income).is_empty());
        cleanup(&path);
    }

    #[test]
    fn tab_walks_fields_then_rows_and_closes_at_ends() {
        let path = scratch("tab");
        let mut s = Session::new(path.clone(), Currency::Usd);
        add_complete(&mut s, List::Income, "a", "1");
        add_complete(&mut s, List::Income, "b", "2");
        let (a, b) = (s.rows(List::Income)[0].id, s.rows(List::Income)[1].id);

        let first = Edit { list: List::Income, id: a, field: Field::Name };
        let mut text = s.start_edit(first, "").unwrap().text;
        let mut seen = vec![(a, Field::Name)];
        for _ in 0..5 {
            let (e, p) = open_of(s.tab(false, &text));
            seen.push((e.id, e.field));
            text = p.text;
        }
        assert_eq!(
            seen,
            [
                (a, Field::Name),
                (a, Field::Date),
                (a, Field::Value),
                (b, Field::Name),
                (b, Field::Date),
                (b, Field::Value),
            ]
        );
        // Past the end: commit and close.
        assert_eq!(s.tab(false, &text), Step::Close);
        assert_eq!(s.editing(), None);
        assert_eq!(s.rows(List::Income).len(), 2);

        // Back-tab at the very start closes too.
        let text = s.start_edit(first, "").unwrap().text;
        assert_eq!(s.tab(true, &text), Step::Close);
        cleanup(&path);
    }

    #[test]
    fn saved_asset_edits_name_and_category_of_its_holding() {
        let path = scratch("asset-holding");
        let mut s = Session::new(path.clone(), Currency::Usd);
        for (name, value) in [("ETF", "10"), ("ETF", "12"), ("Gold", "5")] {
            open_of(s.open_draft(today(), Currency::Usd, ""));
            open_of(s.enter(name));
            open_of(s.enter("Stocks"));
            open_of(s.enter("usd"));
            assert_eq!(s.enter(value), Step::Close);
            s.leave_asset_view("");
        }
        let id = s.rows(List::Asset)[0].id;
        let edit = |field| Edit { list: List::Asset, id, field };

        // Tab goes name, category, then the Add / Remove amounts.
        let text = s.start_edit(edit(Field::Name), "").unwrap().text;
        let (e, p) = open_of(s.tab(false, &text));
        assert_eq!(e.field, Field::Category);
        let (e, _) = open_of(s.tab(false, &p.text));
        assert_eq!(e.field, Field::Amount(Adjust::Add));
        let (e, _) = open_of(s.tab(false, ""));
        assert_eq!(e.field, Field::Amount(Adjust::Remove));
        assert_eq!(s.tab(false, ""), Step::Close);

        // Rename and re-categorise every ETF snapshot.
        s.start_edit(edit(Field::Name), "");
        s.commit_and_close("World ETF");
        s.start_edit(edit(Field::Category), "");
        s.commit_and_close("Funds");
        let names: Vec<_> = s.ledger().assets.iter().map(|a| (a.name.as_str(), a.category.as_str())).collect();
        assert_eq!(names, [("World ETF", "Funds"), ("World ETF", "Funds"), ("Gold", "Stocks")]);

        // Taking another holding's name, or clearing it, is ignored.
        s.start_edit(edit(Field::Name), "");
        s.commit_and_close("Gold");
        s.start_edit(edit(Field::Name), "");
        s.commit_and_close("");
        assert_eq!(s.rows(List::Asset)[0].name, "World ETF");
        assert_eq!(s.rows(List::Asset).len(), 3);
        assert!(!s.unsaved());
        cleanup(&path);
    }

    #[test]
    fn save_blocked_while_rows_ignored() {
        let path = scratch("ignored");
        let seed = Ledger { incomes: vec![act("ok", 100, today())], ..Ledger::default() };
        store::save(&path, &seed).unwrap();
        let mut book = umya_spreadsheet::reader::xlsx::read(&path).unwrap();
        let ws = book.sheet_by_name_mut("Incomes").unwrap();
        let row = ws.highest_row() + 1;
        ws.cell_mut((1, row)).set_value_string("broken");
        ws.cell_mut((2, row)).set_value_string("abc");
        ws.cell_mut((3, row)).set_value_string("09.11.2026");
        umya_spreadsheet::writer::xlsx::write(&book, &path).unwrap();

        let mut s = Session::new(path.clone(), Currency::Usd);
        assert_eq!(s.ignored(), 1);
        add_complete(&mut s, List::Income, "new", "5");
        assert!(s.unsaved());
        // Disk still holds only the original good row.
        assert_eq!(store::load(&path, Currency::Usd).unwrap().ledger.incomes.len(), 1);
        cleanup(&path);
    }

    #[test]
    fn save_blocked_while_unreadable() {
        let path = scratch("unreadable");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, b"not an xlsx").unwrap();
        let mut s = Session::new(path.clone(), Currency::Usd);
        assert!(s.unreadable());
        add_complete(&mut s, List::Income, "new", "5");
        assert!(s.unsaved());
        assert_eq!(std::fs::read(&path).unwrap(), b"not an xlsx");
        cleanup(&path);
    }

    #[cfg(windows)]
    #[test]
    fn locked_save_is_retried_by_sync() {
        use std::os::windows::fs::OpenOptionsExt;

        let path = scratch("locked");
        store::save(&path, &Ledger::default()).unwrap();
        let mut s = Session::new(path.clone(), Currency::Usd);

        // Exclusive handle, like Excel holding the file.
        let hold = std::fs::OpenOptions::new().read(true).share_mode(0).open(&path).unwrap();
        add_complete(&mut s, List::Income, "pay", "5");
        assert!(s.unsaved());
        assert!(!s.sync());
        assert!(s.unsaved());

        drop(hold);
        assert!(s.sync());
        assert!(!s.unsaved());
        assert_eq!(store::load(&path, Currency::Usd).unwrap().ledger.incomes.len(), 1);
        cleanup(&path);
    }

    #[test]
    fn sync_reloads_external_change_and_closes_edit() {
        let path = scratch("sync");
        let mut s = Session::new(path.clone(), Currency::Usd);
        add_complete(&mut s, List::Income, "mine", "1");
        let id = s.rows(List::Income)[0].id;
        s.start_edit(Edit { list: List::Income, id, field: Field::Name }, "").unwrap();
        assert!(!s.sync());

        let external = Ledger {
            incomes: vec![act("one", 100, today()), act("two", 200, today()), act("three", 300, today())],
            ..Ledger::default()
        };
        store::save(&path, &external).unwrap();

        assert!(s.sync());
        assert_eq!(s.editing(), None);
        assert_eq!(*s.ledger(), external);
        assert!(!s.unsaved());
        cleanup(&path);
    }

    #[test]
    fn delete_row_persists() {
        let path = scratch("delete");
        let mut s = Session::new(path.clone(), Currency::Usd);
        add_complete(&mut s, List::Income, "pay", "5");
        let id = s.rows(List::Income)[0].id;
        s.delete_row(List::Income, id);
        assert!(s.rows(List::Income).is_empty());
        assert!(store::load(&path, Currency::Usd).unwrap().ledger.incomes.is_empty());
        cleanup(&path);
    }

    #[test]
    fn set_currency_saves_and_new_rows_take_given_currency() {
        let path = scratch("currency");
        let mut s = Session::new(path.clone(), Currency::Usd);
        open_of(s.add_row(List::Income, today(), Currency::Try, ""));
        open_of(s.enter("pay"));
        // Empty text at the currency step keeps the preselected one.
        open_of(s.enter(""));
        assert_eq!(s.enter("5"), Step::Close);
        assert_eq!(s.rows(List::Income)[0].currency, Currency::Try);

        let id = s.rows(List::Income)[0].id;
        s.set_currency(List::Income, id, Currency::Eur);
        assert_eq!(s.rows(List::Income)[0].currency, Currency::Eur);
        assert!(!s.unsaved());
        assert_eq!(store::load(&path, Currency::Usd).unwrap().ledger.incomes[0].currency, Currency::Eur);

        // Unknown id is a no-op.
        s.set_currency(List::Income, id + 100, Currency::Gbp);
        assert_eq!(s.rows(List::Income)[0].currency, Currency::Eur);
        cleanup(&path);
    }

    #[test]
    fn legacy_rows_take_default_and_get_written() {
        let path = scratch("legacy");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let mut book = umya_spreadsheet::new_file();
        let ws = book.sheet_by_name_mut("Sheet1").unwrap();
        ws.set_name("Incomes");
        ws.cell_mut((1, 1)).set_value_string("Name");
        ws.cell_mut((2, 1)).set_value_string("Value");
        ws.cell_mut((3, 1)).set_value_string("Date");
        ws.cell_mut((1, 2)).set_value_string("old pay");
        ws.cell_mut((2, 2)).set_value_number(12.0);
        ws.cell_mut((3, 2)).set_value_string("01.01.2026");
        umya_spreadsheet::writer::xlsx::write(&book, &path).unwrap();

        let s = Session::new(path.clone(), Currency::Try);
        assert_eq!(s.ledger().incomes[0].currency, Currency::Try);
        assert!(!s.unsaved());
        // Migrated on open: the column is now explicit, so another default
        // no longer changes it.
        let reloaded = store::load(&path, Currency::Usd).unwrap();
        assert!(!reloaded.defaulted);
        assert_eq!(reloaded.ledger.incomes[0].currency, Currency::Try);
        cleanup(&path);
    }

    #[test]
    fn currency_step_validates_against_list() {
        let path = scratch("ask-currency");
        let mut s = Session::new(path.clone(), Currency::Usd);
        open_of(s.add_row(List::Income, today(), Currency::Usd, ""));
        open_of(s.enter("pay"));
        // Metals and crypto are assets only; unknown codes are ignored.
        open_of(s.enter("btc"));
        assert_eq!(s.enter("5"), Step::Close);
        assert_eq!(s.rows(List::Income)[0].currency, Currency::Usd);

        // Asked once: editing the value later closes without the step.
        let id = s.rows(List::Income)[0].id;
        s.start_edit(Edit { list: List::Income, id, field: Field::Value }, "");
        assert_eq!(s.enter("6"), Step::Close);

        // Prefix picks the first match.
        open_of(s.add_row(List::Income, today(), Currency::Usd, ""));
        open_of(s.enter("bonus"));
        open_of(s.enter("e"));
        assert_eq!(s.enter("1"), Step::Close);
        assert_eq!(s.ledger().incomes.iter().find(|a| a.name == "bonus").unwrap().currency, Currency::Eur);
        cleanup(&path);
    }

    #[test]
    fn tab_from_currency_step() {
        let path = scratch("tab-currency");
        let mut s = Session::new(path.clone(), Currency::Usd);
        open_of(s.add_row(List::Income, today(), Currency::Usd, ""));
        let (e, p) = open_of(s.enter("b"));
        assert_eq!(e.field, Field::Currency);
        // Tab goes on to the value of the same row, committing the pick.
        let (next, _) = open_of(s.tab(false, "eur"));
        assert_eq!((next.id, next.field), (e.id, Field::Value));
        assert_eq!(s.rows(List::Income)[0].currency, Currency::Eur);
        // Back-tab from the currency step goes to the date.
        s.start_edit(Edit { field: Field::Currency, ..e }, "");
        let (back, _) = open_of(s.tab(true, &p.text));
        assert_eq!((back.id, back.field), (e.id, Field::Date));
        cleanup(&path);
    }

    #[test]
    fn crypto_value_parsed_with_8_decimals_and_rescaled_on_change() {
        let path = scratch("crypto");
        let mut s = Session::new(path.clone(), Currency::Usd);
        open_of(s.open_draft(today(), Currency::Usd, ""));
        open_of(s.enter("Wallet"));
        open_of(s.enter("Crypto"));
        open_of(s.enter("btc"));
        assert_eq!(s.enter("0,00512"), Step::Close);
        let row = &s.rows(List::Asset)[0];
        assert_eq!((row.currency, row.value), (Currency::Btc, Some(512_000)));

        // Editing shows the full precision.
        let id = row.id;
        let edit = |field| Edit { list: List::Asset, id, field };
        let p = s.start_edit(edit(Field::Value), "").unwrap();
        assert_eq!(p.text, "0,00512");
        s.cancel("");

        // Switching the draft's type keeps the shown number, rounded to the
        // new currency's decimals.
        s.start_edit(edit(Field::Currency), "");
        s.commit_and_close("usd");
        assert_eq!(s.rows(List::Asset)[0].value, Some(1)); // 0,01
        s.start_edit(edit(Field::Currency), "");
        s.commit_and_close("eth");
        assert_eq!(s.rows(List::Asset)[0].value, Some(1_000_000)); // 0,01 ETH

        // Once saved, the type is fixed.
        s.leave_asset_view("");
        s.set_currency(List::Asset, id, Currency::Usd);
        assert_eq!(s.ledger().assets[0].currency, Currency::Eth);
        cleanup(&path);
    }
}
