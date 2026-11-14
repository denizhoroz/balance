//! Balance page: net balance, change over the interval, income/expense totals.

use gpui_kit::*;

use crate::core::ledger::Kind;
use crate::ui::app::{BalanceApp, View, today};
use crate::ui::components::{big_number, col, row};
use crate::ui::fonts::Fonts;
use crate::ui::format::{format_total, signed_total};

/// Width available to the balance number.
const BALANCE_MAX_WIDTH: f32 = 520.;

impl BalanceApp {
    pub fn render_balance(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let p = self.palette();
        let fonts = Fonts::get(cx).clone();
        let today = today();
        let ledger = self.session.ledger();
        let currency = self.settings.currency;
        let interval = self.settings.interval;
        let fx = self.fx();

        let (balance_text, balance_color) = signed_total(&p, ledger.balance_partial(today, fx), currency);
        let change = ledger.change(interval, today, fx);
        let income = ledger.total_in_partial(Kind::Income, interval, today, fx);
        let expense = ledger.total_in_partial(Kind::Expense, interval, today, fx);
        // Each text opens the income/expense table: balance, change, totals.
        let change_row = self.change_row(change, Some(View::Table), cx);

        let number = div()
            .id("balance-number")
            .cursor_pointer()
            .on_click(self.open(View::Table, cx))
            .child(big_number(&fonts, balance_text, balance_color, BALANCE_MAX_WIDTH));
        // Click target of the totals line: income text .. line .. expense
        // text (three adjacent pieces, padding included so there are no gaps).
        let mut target =
            |id: &'static str| row().id(id).h(px(38.)).cursor_pointer().on_click(self.open(View::Table, cx));
        let income_text = format_total(income, currency);
        let expense_text = format_total(expense, currency);

        col()
            .size_full()
            .items_center()
            .justify_center()
            .child(number)
            .child(change_row)
            .child(
                // Equal flex_1 halves keep the line centered.
                row()
                    .mt(px(24.))
                    .w_full()
                    .text_size(px(16.))
                    .child(
                        row()
                            .flex_1()
                            .justify_end()
                            .child(target("totals-in").pr(px(18.)).text_color(p.nonzero(income.value, p.green)).child(income_text)),
                    )
                    .child(target("totals-line").child(div().w(px(1.)).h(px(20.)).bg(p.line)))
                    .child(row().flex_1().child(target("totals-out").pl(px(18.)).text_color(p.nonzero(expense.value, p.red)).child(expense_text))),
            )
    }
}
