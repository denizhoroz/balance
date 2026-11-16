//! Popup editor laid over a table row. Its behaviour is the shared
//! editor's (`views::editor`); this is only the popup's look.

use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::core::session::{Field, List};
use crate::ui::app::BalanceApp;
use crate::ui::components::row;

impl BalanceApp {
    /// Popup editor laid over the whole row (full column width) so long
    /// names are easy to type. Drawn deferred so it sits above neighbours.
    pub(super) fn render_edit_popup(&self, list: List, field: Field, cx: &mut Context<Self>) -> impl IntoElement {
        let p = self.palette();
        let is_currency = field == Field::Currency;
        let (label, color) = match field {
            Field::Name => ("Name", p.fg),
            Field::Category => ("Category", p.fg),
            Field::Date => ("Date", p.fg),
            Field::Value => ("Value", p.list(list)),
            Field::Currency => ("Currency", p.fg),
            // Asset view only; never in the table popup.
            Field::Amount(_) => ("Amount", p.fg),
        };
        let field_row = row().gap(px(8.)).px(px(6.)).child(div().flex_none().text_size(px(11.)).text_color(p.muted).child(label)).child(
            div()
                .when(is_currency, |d| d.flex_none().w(px(44.)))
                .when(!is_currency, |d| d.flex_1())
                .min_w_0()
                .text_color(color)
                .child(self.editor_input().text_color(color)),
        );

        let popup = div()
            .id("edit-popup")
            .absolute()
            .top(px(-4.))
            .left(px(-6.))
            .right(px(-6.))
            .bg(p.surface)
            .text_size(px(13.))
            .occlude();
        let popup = self.editor_shell(field, popup, cx);

        // Over the row for text fields; the currency dropdown grows
        // downward from the row.
        let popup = if is_currency {
            popup.flex().flex_col().child(field_row.h(px(32.))).child(self.currency_menu(list, cx))
        } else {
            popup.bottom(px(-4.)).flex().items_center().child(field_row.flex_1())
        };
        deferred(popup).with_priority(1)
    }
}
