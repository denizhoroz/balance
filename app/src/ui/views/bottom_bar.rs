//! Bottom bar: status text, top-level view switcher, theme/settings/back.

use gpui_kit::assets::IconName as AssetIcon;
use gpui_kit::component::IconName;
use gpui_kit::*;

use crate::core::settings::Theme;
use crate::ui::app::{BalanceApp, View};
use crate::ui::components::{icon_button, row};
use crate::ui::palette::Palette;

impl BalanceApp {
    pub fn render_bottom_bar(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let p = self.palette();
        let top_level = self.view.is_top_level();
        // Equal flex_1 sides keep the center group exactly centered.
        row()
            .h(px(36.))
            .flex_none()
            .px(px(12.))
            .child(self.render_status(&p))
            .children(top_level.then(|| self.render_switcher(&p, cx)))
            .child(self.render_actions(&p, top_level, cx))
    }

    fn status_items(&self) -> Vec<String> {
        let mut status = Vec::new();
        if self.session.unreadable() {
            status.push("file unreadable".to_string());
        }
        let ignored = self.session.ignored();
        if ignored > 0 {
            let s = if ignored == 1 { "" } else { "s" };
            status.push(format!("{ignored} row{s} ignored"));
        }
        if self.session.unsaved() {
            status.push("unsaved".to_string());
        }
        // Rates matter only when some row is in another currency.
        if self.feeds.rates.date.is_none() && self.session.ledger().uses_foreign(self.settings.currency) {
            status.push("rates unavailable".to_string());
        }
        status
    }

    fn render_status(&self, p: &Palette) -> Div {
        row()
            .flex_1()
            .gap(px(10.))
            .child(div().text_color(p.muted).child("balance"))
            .children(self.status_items().into_iter().map(|s| div().text_size(px(11.)).text_color(p.muted).child(s)))
    }

    /// Top-level view switcher. The current view's icon is bright.
    fn render_switcher(&self, p: &Palette, cx: &mut Context<Self>) -> Div {
        let tabs = [
            ("balance", View::Balance, AssetIcon::ArrowRightLeft),
            ("assets", View::Assets, AssetIcon::Wallet),
        ];
        row().flex_none().gap(px(4.)).children(tabs.into_iter().map(|(id, view, icon)| {
            let color = if self.view == view { p.fg } else { p.muted };
            icon_button(id, icon, color, p.fg, self.open(view, cx))
        }))
    }

    fn render_actions(&self, p: &Palette, top_level: bool, cx: &mut Context<Self>) -> Div {
        let right = row().flex_1().justify_end();
        if !top_level {
            let back = cx.listener(|this, _: &ClickEvent, window, cx| this.back(window, cx));
            return right.child(icon_button("back", IconName::ArrowLeft, p.muted, p.fg, back));
        }
        let theme_icon = match self.settings.theme {
            Theme::Dark => IconName::Sun,
            Theme::Light => IconName::Moon,
        };
        let toggle = cx.listener(|this, _: &ClickEvent, window, cx| this.toggle_theme(window, cx));
        right
            .child(icon_button("theme", theme_icon, p.muted, p.fg, toggle))
            .child(icon_button("settings", IconName::Settings, p.muted, p.fg, self.open(View::Settings, cx)))
    }
}
