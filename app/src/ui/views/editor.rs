//! The shared field editor: everything the table popup and the asset view
//! header have in common. Callers only lay it out and style it.
//!
//! Behaviour lives here once: clicking outside the editor commits, Esc
//! cancels, Tab / Shift+Tab move to the session's next field, and on the
//! currency step ↑/↓ cycle the choices and the list picks one. The editor's
//! own blur also commits (see `app::editing`); the currency list keeps focus
//! on the input so that blur does not commit before the pick.

use gpui_kit::component::Sizable as _;
use gpui_kit::component::input::{Escape, IndentInline, Input, MoveDown, MoveUp, OutdentInline};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::core::currency::Currency;
use crate::core::session::{Field, List};
use crate::ui::app::BalanceApp;
use crate::ui::components::{capture, col, link};

impl BalanceApp {
    /// The single shared input, without chrome; callers set size and color.
    pub(super) fn editor_input(&self) -> Input {
        Input::new(&self.editor).appearance(false).small()
    }

    /// Gives `el`, the element holding the editor for `field` (and, on the
    /// currency step, its [`BalanceApp::currency_menu`]), the editing
    /// behaviour. Clicks inside `el`'s bounds do not commit.
    pub(super) fn editor_shell(&self, field: Field, el: Stateful<Div>, cx: &mut Context<Self>) -> Stateful<Div> {
        let el = el.on_mouse_down_out(cx.listener(|this, _: &MouseDownEvent, window, cx| this.commit_and_close(window, cx)));
        let el = capture::<Escape, _>(el, cx, |this, window, cx| this.cancel_edit(window, cx));
        let el = capture::<IndentInline, _>(el, cx, |this, window, cx| this.on_editor_tab(false, window, cx));
        let el = capture::<OutdentInline, _>(el, cx, |this, window, cx| this.on_editor_tab(true, window, cx));
        if field != Field::Currency {
            return el;
        }
        let el = capture::<MoveUp, _>(el, cx, |this, window, cx| this.cycle_currency(-1, window, cx));
        capture::<MoveDown, _>(el, cx, |this, window, cx| this.cycle_currency(1, window, cx))
    }

    /// Currency step: the open dropdown list under the input. The input
    /// holds the picked code (typing a prefix jumps to a match). Put it
    /// inside the shell's element so clicking it does not count as clicking
    /// out.
    pub(super) fn currency_menu(&self, list: List, cx: &mut Context<Self>) -> Div {
        let p = self.palette();
        let typed = self.editor.read(cx).value().to_string();
        let picked = Currency::match_prefix(&typed, list.currencies());
        col().pb(px(4.)).children(list.currencies().iter().map(|&c| {
            let on = picked == Some(c);
            link(SharedString::from(format!("pick-{}", c.code())), if on { p.fg } else { p.muted }, p.fg)
                .h(px(22.))
                .px(px(6.))
                .text_size(px(12.))
                .when(on, |d| d.bg(p.selection))
                .child(c.code())
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, _: &MouseDownEvent, window, cx| {
                        // Keep the editor focused so its blur does not
                        // commit before the pick.
                        window.prevent_default();
                        cx.stop_propagation();
                        this.pick_currency(c, window, cx);
                    }),
                )
        }))
    }
}
