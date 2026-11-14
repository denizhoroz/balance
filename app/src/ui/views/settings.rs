//! Settings page.

use gpui_kit::component::Sizable as _;
use gpui_kit::component::select::Select;
use gpui_kit::*;

use crate::ui::app::BalanceApp;
use crate::ui::components::{col, fixed, row};

impl BalanceApp {
    pub fn render_settings(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let p = self.palette();
        col().size_full().px(px(16.)).pt(px(12.)).child(
            row()
                .h(px(36.))
                .justify_between()
                .child(div().text_color(p.fg).child("Change Currency"))
                .child(fixed(div(), 96.).child(Select::new(&self.currency_select).appearance(false).small().w_full())),
        )
    }
}
