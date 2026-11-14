//! Application state and view switching. The rest lives in submodules, each
//! adding methods to `BalanceApp`:
//! - `rates`: price feeds and their background fetch
//! - `settings`: settings file, theme, interval, display currency
//! - `disk`: watching `db/` for external changes
//! - `editing`: edit popup input forwarded to the session
//! - `row_currency`: per-row currency dropdowns

mod disk;
mod editing;
mod rates;
mod row_currency;
mod settings;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use chrono::{Local, NaiveDate};
use gpui_kit::component::input::InputState;
use gpui_kit::component::select::{SearchableVec, SelectState};
use gpui_kit::*;

use self::disk::Watcher;
use crate::core::cache;
use crate::core::feeds::RateFeeds;
use crate::core::sources::Links;
use crate::core::rates::Fx;
use crate::core::session::{List, Session};
use crate::core::settings::Settings;
use crate::core::store::{self, WORKBOOK_FILE};
use crate::ui::fonts::Fonts;
use crate::ui::palette::Palette;

actions!(balance, [GoBack]);

pub const CONTEXT: &str = "Balance";

pub fn init(cx: &mut App) {
    cx.bind_keys([KeyBinding::new("escape", GoBack, Some(CONTEXT))]);
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum View {
    // Top-level views, switched with the bottom-center icons.
    Assets,
    Balance,
    // Sub-views; back / Esc returns to the last top-level view.
    Table,
    AssetTable,
    /// One asset: what it shows is the session's (see
    /// `Session::asset_view`). Back returns to the view it was opened from.
    AssetDetail,
    Settings,
}

impl View {
    pub fn is_top_level(self) -> bool {
        matches!(self, View::Assets | View::Balance)
    }
}

/// Dropdown of currency codes (`USD`, `EUR`, ...).
pub type CurrencySelect = Entity<SelectState<SearchableVec<SharedString>>>;

pub struct BalanceApp {
    pub settings: Settings,
    pub view: View,
    /// Last top-level view; back / Esc return here.
    pub home: View,
    /// View the asset view's back returns to.
    pub detail_from: View,
    pub focus: FocusHandle,
    /// Rows, edit cursor, persistence rules.
    pub session: Session,
    /// Ring segment under the mouse on the assets view.
    pub hovered_segment: Option<usize>,
    /// Display currency dropdown on the settings page.
    pub currency_select: CurrencySelect,
    /// Single shared input used by the edit popup.
    pub editor: Entity<InputState>,
    /// One currency dropdown per table row, keyed by list and row id.
    pub row_currency: HashMap<(List, u64), (CurrencySelect, Subscription)>,
    /// Exchange rates and prices, and which feeds are current.
    pub feeds: RateFeeds,
    /// Where prices are fetched from (`db/links.json`); `None` when that
    /// file is missing or bad, and then nothing is fetched.
    links: Option<Arc<Links>>,
    db: PathBuf,
    _watcher: Option<Watcher>,
    _subs: Vec<Subscription>,
}

pub fn today() -> NaiveDate {
    Local::now().date_naive()
}

impl BalanceApp {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let db = store::db_dir();
        let _ = std::fs::create_dir_all(&db);
        let settings = settings::load(&db);

        let (currency_select, currency_sub) = Self::create_currency_select(settings.currency, window, cx);
        let (editor, editor_sub) = Self::create_editor(window, cx);
        // Retry pending saves / pick up external changes when window regains focus.
        let activation_sub = cx.observe_window_activation(window, |this, window, cx| {
            if window.is_window_active() {
                this.on_disk_event(window, cx);
                this.fetch_rates(cx);
            }
        });

        let session = Session::new(db.join(WORKBOOK_FILE), settings.currency);
        let mut app = Self {
            settings,
            view: View::Balance,
            home: View::Balance,
            detail_from: View::AssetTable,
            focus: cx.focus_handle(),
            session,
            hovered_segment: None,
            currency_select,
            editor,
            row_currency: HashMap::new(),
            feeds: RateFeeds::load(cache::dir()),
            links: Links::load(&db).map(Arc::new).inspect_err(|e| eprintln!("balance: no prices fetched: {e:#}")).ok(),
            db,
            _watcher: None,
            _subs: vec![currency_sub, editor_sub, activation_sub],
        };
        app.apply_theme(window, cx);
        app.start_watcher(window, cx);
        app.fetch_rates(cx);
        window.focus(&app.focus, cx);
        app
    }

    pub fn palette(&self) -> Palette {
        Palette::for_theme(self.settings.theme)
    }

    /// Converter into the display currency with the rates loaded so far.
    pub fn fx(&self) -> Fx<'_> {
        Fx::new(self.settings.currency, Some(&self.feeds.rates))
    }

    pub fn show(&mut self, view: View, window: &mut Window, cx: &mut Context<Self>) {
        self.view = view;
        if view.is_top_level() {
            self.home = view;
        }
        self.hovered_segment = None;
        window.focus(&self.focus, cx);
        cx.notify();
    }

    /// Opens the asset view on a new asset, its name field focused.
    pub fn open_draft(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let text = self.editor_text(cx);
        let step = self.session.open_draft(today(), self.settings.currency, &text);
        self.show_asset_view(window, cx);
        self.apply(step, window, cx);
    }

    /// Opens the asset view on a saved asset row.
    pub fn open_asset_row(&mut self, id: u64, window: &mut Window, cx: &mut Context<Self>) {
        let text = self.editor_text(cx);
        self.session.open_asset(id, &text);
        self.show_asset_view(window, cx);
    }

    fn show_asset_view(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.detail_from = self.view;
        self.show(View::AssetDetail, window, cx);
    }

    /// Closes the asset view if the session has nothing for it any more
    /// (its asset is gone after a reload); the session decides.
    pub(super) fn close_stale_asset_view(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.view == View::AssetDetail && self.session.asset_view().is_none() {
            self.show(self.detail_from, window, cx);
        }
    }

    /// Back / Esc: the asset view returns where it was opened from (the
    /// session finishes whatever it was showing); other sub-views to the last
    /// top-level view.
    pub fn back(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.view == View::AssetDetail {
            let text = self.editor_text(cx);
            self.session.leave_asset_view(&text);
            self.show(self.detail_from, window, cx);
        } else if self.view != self.home {
            self.show(self.home, window, cx);
        }
    }

    pub fn go_back(&mut self, _: &GoBack, window: &mut Window, cx: &mut Context<Self>) {
        self.back(window, cx);
    }
}

impl Focusable for BalanceApp {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Render for BalanceApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let p = self.palette();
        let fonts = Fonts::get(cx).clone();
        let content = match self.view {
            View::Assets => self.render_assets(window, cx).into_any_element(),
            View::Balance => self.render_balance(window, cx).into_any_element(),
            View::Table => self.render_table(window, cx).into_any_element(),
            View::AssetTable => self.render_asset_table(window, cx).into_any_element(),
            View::AssetDetail => self.render_asset_detail(window, cx).into_any_element(),
            View::Settings => self.render_settings(window, cx).into_any_element(),
        };
        div()
            .key_context(CONTEXT)
            .track_focus(&self.focus)
            .on_action(cx.listener(Self::go_back))
            .size_full()
            .flex()
            .flex_col()
            .bg(p.bg)
            .text_color(p.fg)
            .font_family(fonts.light_family)
            .font_weight(Fonts::LIGHT)
            .text_size(px(13.))
            .child(div().flex_1().min_h_0().child(content))
            .child(self.render_bottom_bar(window, cx))
    }
}
