//! One currency dropdown per table row.

use gpui_kit::component::IndexPath;
use gpui_kit::component::select::{SearchableVec, SelectEvent, SelectState};
use gpui_kit::*;

use super::BalanceApp;
use crate::core::session::List;
use crate::core::settings::Currency;

impl BalanceApp {
    /// Keeps one currency dropdown per row of `list`, showing the row's
    /// currency; drops dropdowns of rows that are gone (deleted, reloaded).
    /// Asset rows may also pick gold/silver (grams).
    pub fn sync_currency_selects(&mut self, list: List, window: &mut Window, cx: &mut Context<Self>) {
        let options = list.currencies();
        let rows: Vec<(u64, Currency)> = self.session.rows(list).iter().map(|r| (r.id, r.currency)).collect();
        self.row_currency.retain(|(l, id), _| *l != list || rows.iter().any(|(r, _)| r == id));
        for (id, currency) in rows {
            let ix = options.iter().position(|c| *c == currency).map(IndexPath::new);
            if let Some((select, _)) = self.row_currency.get(&(list, id)) {
                let shown = select.read(cx).selected_value().map(|v| v.to_string());
                if shown.as_deref() != Some(currency.code()) {
                    select.update(cx, |s, cx| s.set_selected_index(ix, window, cx));
                }
                continue;
            }
            let codes: Vec<SharedString> = options.iter().map(|c| c.code().into()).collect();
            let select = cx.new(|cx| SelectState::new(SearchableVec::new(codes), ix, window, cx));
            let sub = cx.subscribe_in(
                &select,
                window,
                move |this, _, ev: &SelectEvent<SearchableVec<SharedString>>, _, cx| {
                    let SelectEvent::Confirm(Some(code)) = ev else { return };
                    if let Some(c) = Currency::from_code(code) {
                        this.session.set_currency(list, id, c);
                        cx.notify();
                    }
                },
            );
            self.row_currency.insert((list, id), (select, sub));
        }
    }
}
