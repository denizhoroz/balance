//! Table page: income and expense columns of editable rows.

use gpui_kit::component::select::Select;
use gpui_kit::component::{Icon, Sizable as _};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::core::date::format_date_short;
use crate::core::ledger::Kind;
use crate::core::money::format_number_dec;
use crate::core::session::{Edit, Field, List};
use crate::ui::app::{BalanceApp, today};
use crate::ui::components::{col, fixed, row};
use crate::ui::format::{format_total, or_placeholder};

/// Column widths shared by the header and the rows.
/// `DD.MM`; full `DD.MM.YYYY` dates widen their own cell to the left.
const DATE_W: f32 = 38.;
const DATE_FULL_W: f32 = 64.;
const VALUE_W: f32 = 80.;
/// Wide enough for the select trigger's padding and icon slot plus a
/// 3-letter code; narrower truncates the code to `…`.
const TYPE_W: f32 = 60.;
/// Where the select trigger draws its code; plain codes and the header align to it.
const TYPE_PAD: f32 = 8.;
/// Space between cells.
const GAP: f32 = 6.;

/// A plain-text cell; clicking it opens the edit popup on that field.
fn cell(list: List, id: u64, field: Field, cx: &mut Context<BalanceApp>) -> Stateful<Div> {
    div()
        .id(SharedString::from(format!("{list:?}-{id}-{field:?}")))
        .whitespace_nowrap()
        .overflow_hidden()
        .text_ellipsis()
        .cursor_text().on_click(cx.listener(move |this, _: &ClickEvent, window, cx| {
        this.start_edit(Edit { list, id, field }, window, cx)
    }))
}

impl BalanceApp {
    /// Incomes and expenses side by side.
    pub fn render_table(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let columns: Vec<_> = [List::Income, List::Expense].into_iter().map(|list| self.render_column(list, window, cx)).collect();
        div().flex().size_full().gap(px(16.)).px(px(12.)).pt(px(8.)).children(columns)
    }

    fn render_column(&mut self, list: List, window: &mut Window, cx: &mut Context<Self>) -> Div {
        self.sync_currency_selects(list, window, cx);
        let p = self.palette();
        let today = today();
        let ledger = self.session.ledger();
        let fx = self.fx();
        let (label, total) = match list {
            List::Income => ("Incomes", ledger.total_partial(Kind::Income, today, fx)),
            List::Expense => ("Expenses", ledger.total_partial(Kind::Expense, today, fx)),
            List::Asset => unreachable!("assets have their own table"),
        };
        let rows: Vec<AnyElement> = (0..self.session.rows(list).len()).map(|ix| self.render_row(list, ix, cx)).collect();
        let add = self.add_button(
            SharedString::from(format!("{list:?}-add")),
            cx.listener(move |this, _: &ClickEvent, window, cx| this.add_row(list, window, cx)),
        );

        // Column names, like the Assets Table header.
        let header = row()
            .h(px(20.))
            .flex_none()
            .gap(px(GAP))
            .text_size(px(11.))
            .text_color(p.muted)
            .child(div().flex_1().min_w_0().child("Name"))
            .child(fixed(div(), DATE_W).child("Date"))
            .child(fixed(div(), VALUE_W).text_right().child("Value"))
            .child(fixed(div(), TYPE_W).pl(px(TYPE_PAD)).child("Type"));

        col()
            .flex_1()
            .min_w_0()
            .h_full()
            .child(header)
            .child(
                div()
                    .id(SharedString::from(format!("{list:?}-scroll")))
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .children(rows)
                    .child(row().h(px(24.)).justify_center().child(add)),
            )
            .child(
                row()
                    .h(px(28.))
                    .flex_none()
                    .justify_between()
                    .text_size(px(12.))
                    .child(div().text_color(p.muted).child(label))
                    .child(
                        div()
                            .text_color(p.nonzero(total.value, p.list(list)))
                            .child(format_total(total, self.settings.currency)),
                    ),
            )
    }

    fn render_row(&self, list: List, ix: usize, cx: &mut Context<Self>) -> AnyElement {
        let p = self.palette();
        let today = today();
        let row_data = &self.session.rows(list)[ix];
        let id = row_data.id;

        // Empty/zero: gray.
        let value_color = p.nonzero(row_data.value.unwrap_or(0), p.list(list));
        let date_color = if row_data.date > today { p.muted.opacity(0.5) } else { p.muted };
        let date_text = format_date_short(row_data.date, today);
        let date_width = if date_text.len() > 5 { DATE_FULL_W } else { DATE_W };
        let name = or_placeholder(row_data.name.clone(), "name");
        let value_text = row_data
            .value
            .map_or_else(|| "—".to_string(), |v| format_number_dec(v, row_data.currency.decimals()));
        let name_color = if row_data.name.is_empty() { p.muted } else { p.fg };
        let currency_select = self.row_currency.get(&(list, id)).map(|(s, _)| s.clone());
        let editing = self.session.editing().filter(|e| e.list == list && e.id == id);

        row()
            .id(SharedString::from(format!("{list:?}-row-{id}")))
            .relative()
            .h(px(24.))
            .gap(px(GAP))
            .text_size(px(12.))
            .capture_any_mouse_down(cx.listener(move |this, ev: &MouseDownEvent, _, cx| {
                if ev.button == MouseButton::Right {
                    cx.stop_propagation();
                    this.delete_row(list, id, cx);
                }
            }))
            .child(cell(list, id, Field::Name, cx).flex_1().min_w_0().text_color(name_color).child(name))
            .child(fixed(cell(list, id, Field::Date, cx), date_width).text_size(px(11.)).text_color(date_color).child(date_text))
            .child(
                fixed(cell(list, id, Field::Value, cx), VALUE_W)
                    .text_right()
                    .text_color(value_color)
                    .child(value_text),
            )
            .child(
                fixed(div(), TYPE_W)
                    .text_size(px(11.))
                    .text_color(p.muted)
                    .when_some(currency_select, |d, select| {
                        // Click the code itself to open; no chevron.
                        d.child(
                            Select::new(&select)
                                .id(SharedString::from(format!("{list:?}-{id}-currency")))
                                .appearance(false)
                                .icon(Icon::empty())
                                .small()
                                .menu_width(px(72.))
                                .w_full(),
                        )
                    }),
            )
            .when_some(editing, |r, edit| r.child(self.render_edit_popup(list, edit.field, cx)))
            .into_any_element()
    }
}
