//! Settings persistence, theme, interval and the display-currency dropdown.

use std::path::Path;

use gpui_kit::component::select::{SearchableVec, SelectEvent, SelectState};
use gpui_kit::component::{IndexPath, Theme as KitTheme, ThemeMode};
use gpui_kit::*;

use super::{BalanceApp, CurrencySelect};
use crate::core::settings::{Currency, Settings, Theme};
use crate::core::store::SETTINGS_FILE;
use crate::ui::fonts::Fonts;

/// Loads `settings.toml` from `db`, writing the defaults when it is absent.
pub(super) fn load(db: &Path) -> Settings {
    let path = db.join(SETTINGS_FILE);
    let settings = Settings::load(&path);
    if !path.exists() {
        let _ = settings.save(&path);
    }
    settings
}

impl BalanceApp {
    /// The display-currency dropdown of the settings page; picking a
    /// currency saves it.
    pub(super) fn create_currency_select(
        current: Currency,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> (CurrencySelect, Subscription) {
        let labels: Vec<SharedString> = Currency::FIAT.iter().map(|c| c.label().into()).collect();
        let selected = Currency::FIAT.iter().position(|c| *c == current);
        let select = cx.new(|cx| SelectState::new(SearchableVec::new(labels), selected.map(IndexPath::new), window, cx));
        let sub = cx.subscribe_in(&select, window, |this, _, ev: &SelectEvent<SearchableVec<SharedString>>, _, cx| {
            let SelectEvent::Confirm(Some(label)) = ev else { return };
            if let Some(c) = Currency::from_label(label) {
                this.settings.currency = c;
                this.session.set_default_currency(c);
                this.save_settings();
                cx.notify();
            }
        });
        (select, sub)
    }

    fn save_settings(&self) {
        if let Err(e) = self.settings.save(&self.db.join(SETTINGS_FILE)) {
            eprintln!("balance: failed to save settings: {e}");
        }
    }

    /// Pushes our palette and font into the component theme so inputs and
    /// the dropdown match the rest of the app.
    pub(super) fn apply_theme(&self, window: &mut Window, cx: &mut App) {
        let mode = match self.settings.theme {
            Theme::Dark => ThemeMode::Dark,
            Theme::Light => ThemeMode::Light,
        };
        KitTheme::change(mode, Some(window), cx);
        let p = self.palette();
        let family = Fonts::get(cx).light_family.clone();
        KitTheme::update(cx, |t| {
            t.font_family = family;
            t.font_size = px(13.);
            t.radius = px(0.);
            t.radius_lg = px(0.);
            t.shadow = false;
            t.focus_ring = false;
            t.background = p.bg;
            t.foreground = p.fg;
            t.muted_foreground = p.muted;
            // No borders unless intentionally specified: blend them away.
            t.border = p.surface;
            t.input = p.bg;
            t.ring = p.bg;
            t.caret = p.fg;
            t.selection = p.selection;
            t.popover = p.surface;
            t.popover_foreground = p.fg;
            t.list_hover = p.selection;
            t.list_active = p.selection;
            t.list_active_border = p.selection;
            t.accent = p.selection;
            t.accent_foreground = p.fg;
            t.scrollbar = p.bg;
            t.scrollbar_thumb = p.line;
            t.scrollbar_thumb_hover = p.muted;
        });
        window.refresh();
    }

    pub fn toggle_theme(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.settings.theme = match self.settings.theme {
            Theme::Dark => Theme::Light,
            Theme::Light => Theme::Dark,
        };
        self.save_settings();
        self.apply_theme(window, cx);
        cx.notify();
    }

    pub fn cycle_interval(&mut self, cx: &mut Context<Self>) {
        self.settings.interval = self.settings.interval.next();
        self.save_settings();
        cx.notify();
    }
}
