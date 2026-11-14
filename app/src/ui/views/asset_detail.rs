//! Asset view: one holding's header (name and category on the left; type,
//! value and date on the right).
//!
//! Opened from the Assets Table `+` on a new draft holding, where each
//! header field is clicked to edit in place; or from an Assets Table row on
//! a saved asset, where only its holding's name and category can change.

use gpui_kit::*;

use crate::core::date::format_date_full;
use crate::core::money::format_number_dec;
use crate::core::session::{AssetView, Edit, Field, List};
use crate::ui::app::{BalanceApp, today};
use crate::ui::components::{col, row};

/// Big header text (name, value).
const BIG: f32 = 24.;
const BIG_H: f32 = 32.;
/// Small header rows (category, date).
const SMALL_H: f32 = 20.;
/// Line height relative to the font size: tall enough for descenders
/// (`q`, `g`), which the clipped fields would otherwise cut off.
const LINE: f32 = 1.3;
/// Space between a big field and the small one next to / below it.
const SPACE: f32 = 6.;
/// Editor widths for the fields that do not flex.
const TYPE_W: f32 = 44.;
const CATEGORY_W: f32 = 200.;
const VALUE_W: f32 = 180.;
const DATE_W: f32 = 90.;

impl BalanceApp {
    pub fn render_asset_detail(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let p = self.palette();
        // The session decides what is shown and what can be edited.
        let header = self.session.asset_view().unwrap_or_default();
        let colors = p.category_colors(&self.session.ledger().assets);
        let currency = header.currency.unwrap_or(self.settings.currency);

        let name = self
            .header_field(&header, Field::Name, header.name.clone(), "name", p.fg, BIG, cx)
            .min_w_0();
        let kind = self.header_field(&header, Field::Currency, header.currency.map(|c| c.code().to_string()), "type", p.muted, 12., cx);
        let category_color = header.category.as_ref().and_then(|c| colors.get(c).copied()).unwrap_or(p.fg);
        let category = self.header_field(&header, Field::Category, header.category.clone(), "category", category_color, 13., cx);
        let value_color = header.value.map_or(p.fg, |v| p.nonzero(v, p.list(List::Asset)));
        let value_text = header.value.map(|v| format_number_dec(v, currency.decimals()));
        let value = self.header_field(&header, Field::Value, value_text, "value", value_color, BIG, cx).flex_none();
        // Gray like table dates; future ones dimmer (they count nowhere yet).
        let date_color = if header.date.is_some_and(|d| d > today()) { p.muted.opacity(0.5) } else { p.muted };
        let date = self.header_field(&header, Field::Date, header.date.map(format_date_full), "date", date_color, 12., cx);

        let left = col()
            .flex_1()
            .min_w_0()
            .gap(px(SPACE))
            .child(row().h(px(BIG_H)).child(name))
            .child(row().h(px(SMALL_H)).child(category));
        let right = col()
            .flex_none()
            .items_end()
            .gap(px(SPACE))
            .child(row().h(px(BIG_H)).gap(px(10.)).child(kind.flex_none()).child(value))
            .child(row().h(px(SMALL_H)).child(date));
        col()
            .size_full()
            .px(px(16.))
            .pt(px(14.))
            .child(row().items_start().gap(px(16.)).child(left).child(right))
    }

    /// One header field: its text (gray placeholder when empty), clickable
    /// when editable; the in-place editor while it is open.
    #[allow(clippy::too_many_arguments)]
    fn header_field(
        &self,
        header: &AssetView,
        field: Field,
        text: Option<String>,
        placeholder: &'static str,
        color: Hsla,
        size: f32,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let p = self.palette();
        let id = SharedString::from(format!("detail-{field:?}"));
        let row = header.row.filter(|_| header.editable.contains(&field));
        let editing = row.is_some_and(|id| self.session.editing() == Some(Edit { list: List::Asset, id, field }));
        if editing {
            return self.header_editor(id, field, color, size, cx);
        }
        let (text, color) = match text {
            Some(t) => (t, color),
            None => (placeholder.to_string(), p.muted),
        };
        let el = div()
            .id(id)
            .text_size(px(size))
            .line_height(relative(LINE))
            .text_color(color)
            .whitespace_nowrap()
            .overflow_hidden()
            .text_ellipsis()
            .child(text);
        match row {
            Some(row) => el.cursor_text().on_click(cx.listener(move |this, _: &ClickEvent, window, cx| {
                this.start_edit(Edit { list: List::Asset, id: row, field }, window, cx)
            })),
            None => el,
        }
    }

    /// The shared editor in place of a header field (behaviour from
    /// `views::editor`). The type opens as a dropdown over the header, like
    /// the table's currency step.
    fn header_editor(&self, id: SharedString, field: Field, color: Hsla, size: f32, cx: &mut Context<Self>) -> Stateful<Div> {
        let p = self.palette();
        let input = self
            .editor_input()
            .text_size(px(size))
            .line_height(relative(LINE))
            .text_color(color)
            .px_0()
            .py_0();
        let input = if matches!(field, Field::Value | Field::Date) { input.text_right() } else { input };

        let height = if size >= BIG { BIG_H } else { SMALL_H };
        match field {
            Field::Currency => {
                let menu = div()
                    .id("detail-type-menu")
                    .absolute()
                    .top_0()
                    .left(px(-6.))
                    .w(px(72.))
                    .bg(p.surface)
                    .text_size(px(12.))
                    .occlude()
                    .flex()
                    .flex_col()
                    .child(row().h(px(BIG_H)).px(px(6.)).child(input))
                    .child(self.currency_menu(List::Asset, cx));
                let menu = self.editor_shell(field, menu, cx);
                div().id(id).relative().w(px(TYPE_W)).h(px(BIG_H)).child(deferred(menu).with_priority(1))
            }
            Field::Name => self.editor_shell(field, div().id(id).flex_1().h(px(height)).flex().items_center().child(input), cx),
            _ => {
                let width = match field {
                    Field::Value => VALUE_W,
                    Field::Date => DATE_W,
                    _ => CATEGORY_W,
                };
                self.editor_shell(field, div().id(id).w(px(width)).h(px(height)).flex().items_center().child(input), cx)
            }
        }
    }
}
