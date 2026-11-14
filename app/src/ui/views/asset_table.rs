//! Assets Table: a ledger of the current holdings (from the asset
//! snapshots) with each one's change and profit over the interval. Clicking
//! a row opens the asset view on its current snapshot; `+` adds one.

use std::collections::HashMap;

use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::core::holdings::{self, Holding};
use crate::core::ledger::format_change;
use crate::core::money::{Cents, format_number_dec};
use crate::core::session::List;
use crate::ui::app::{BalanceApp, today};
use crate::ui::components::{col, fixed, link, row};
use crate::ui::format::{format_profit, format_total};
use crate::ui::palette::Palette;

/// Space between columns.
const GAP: f32 = 12.;
const CATEGORY_W: f32 = 72.;
const VALUE_W: f32 = 84.;
const CURRENCY_W: f32 = 34.;
const CHANGE_W: f32 = 48.;
const PROFIT_W: f32 = 76.;

impl BalanceApp {
    pub fn render_asset_table(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let p = self.palette();
        let (interval, currency, today) = (self.settings.interval, self.settings.currency, today());
        let ledger = self.session.ledger();
        let fx = self.fx();
        // Past days a holding is compared from need that day's prices.
        let valued = holdings::holdings(&ledger.assets, interval, today, fx, &self.feeds.history);
        let (holdings, missing_days) = (valued.value, valued.missing_days);
        let summary = holdings::summary(&holdings, fx);
        let colors = p.category_colors(&ledger.assets);
        self.fetch_days(missing_days, cx);

        let header = row()
            .h(px(20.))
            .flex_none()
            .gap(px(GAP))
            .text_size(px(11.))
            .text_color(p.muted)
            .child(div().flex_1().min_w_0().child("Name"))
            .child(fixed(div(), CATEGORY_W).child("Category"))
            .child(fixed(div(), VALUE_W).text_right().child("Value"))
            .child(fixed(div(), CURRENCY_W).child("Type"))
            .child(fixed(div(), CHANGE_W).text_right().child("Change"))
            .child(fixed(div(), PROFIT_W).text_right().child("Profit"));

        let rows: Vec<AnyElement> = if holdings.is_empty() {
            vec![row().h(px(48.)).justify_center().text_size(px(12.)).text_color(p.muted).child("No holdings yet").into_any_element()]
        } else {
            holdings.iter().map(|h| self.render_holding(h, &colors, cx)).collect()
        };
        let add = self.add_button("asset-add".into(), cx.listener(|this, _: &ClickEvent, window, cx| this.open_draft(window, cx)));

        let footer = row()
            .h(px(28.))
            .flex_none()
            .gap(px(GAP))
            .text_size(px(12.))
            .child(div().flex_1().text_color(p.muted).child("Holdings"))
            .child(
                fixed(row(), CATEGORY_W).child(
                    link("interval", p.muted, p.fg)
                        .text_size(px(11.))
                        .child(self.settings.interval.label())
                        .on_click(cx.listener(|this, _: &ClickEvent, _, cx| this.cycle_interval(cx))),
                ),
            )
            // In the display currency, so it spans the value and currency columns.
            .child(
                fixed(div(), VALUE_W + GAP + CURRENCY_W)
                    .text_right()
                    .whitespace_nowrap()
                    .text_color(p.nonzero(summary.value.value, p.fg))
                    .child(format_total(summary.value, currency)),
            )
            .child(fixed(div(), CHANGE_W).text_right().text_color(p.change(summary.change)).child(format_change(summary.change)))
            .child(fixed(div(), PROFIT_W).text_right().text_color(profit_color(&p, summary.profit)).child(format_profit(summary.profit, currency)));

        col()
            .size_full()
            .px(px(12.))
            .pt(px(8.))
            .child(header)
            .child(
                div()
                    .id("asset-scroll")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .children(rows)
                    .child(row().h(px(24.)).justify_center().child(add)),
            )
            .child(footer)
    }

    /// One holding; clicking it opens the asset view on its current
    /// snapshot, right-clicking deletes that snapshot.
    fn render_holding(&self, h: &Holding, colors: &HashMap<String, Hsla>, cx: &mut Context<Self>) -> AnyElement {
        let p = self.palette();
        let cell = |width: f32| fixed(div(), width).whitespace_nowrap().overflow_hidden().text_ellipsis();
        let id = self.session.holding_row(&h.name, today());
        row()
            .id(SharedString::from(format!("holding-{}", h.name)))
            .h(px(24.))
            .gap(px(GAP))
            .text_size(px(12.))
            .when_some(id, |r, id| {
                r.cursor_pointer()
                    .on_click(cx.listener(move |this, _: &ClickEvent, window, cx| this.open_asset_row(id, window, cx)))
                    .capture_any_mouse_down(cx.listener(move |this, ev: &MouseDownEvent, _, cx| {
                        if ev.button == MouseButton::Right {
                            cx.stop_propagation();
                            this.delete_row(List::Asset, id, cx);
                        }
                    }))
            })
            .child(div().flex_1().min_w_0().whitespace_nowrap().overflow_hidden().text_ellipsis().child(h.name.clone()))
            .child(cell(CATEGORY_W).text_color(colors.get(&h.category).copied().unwrap_or(p.muted)).child(h.category.clone()))
            .child(cell(VALUE_W).text_right().child(format_number_dec(h.value, h.currency.decimals())))
            .child(cell(CURRENCY_W).text_size(px(11.)).text_color(p.muted).child(h.currency.code()))
            .child(cell(CHANGE_W).text_right().text_color(p.change(h.change)).child(format_change(h.change)))
            .child(
                cell(PROFIT_W)
                    .text_right()
                    .text_color(profit_color(&p, h.profit))
                    .child(format_profit(h.profit, self.settings.currency)),
            )
            .into_any_element()
    }
}

/// Green / red / yellow by sign; muted when unknown.
fn profit_color(p: &Palette, profit: Option<Cents>) -> Hsla {
    profit.map_or(p.muted, |v| p.signed(v))
}
