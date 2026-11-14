//! Edit popup: forwards input events to the editing session.

use gpui_kit::component::input::{InputEvent, InputState};
use gpui_kit::*;

use super::{BalanceApp, today};
use crate::core::session::{Edit, Field, List, Step};
use crate::core::settings::Currency;

impl BalanceApp {
    /// The single input shared by every edit popup, and the subscription
    /// routing its events (Enter, typing, blur) to the session.
    pub(super) fn create_editor(window: &mut Window, cx: &mut Context<Self>) -> (Entity<InputState>, Subscription) {
        let editor = cx.new(|cx| InputState::new(window, cx).context_menu(false));
        let sub = cx.subscribe_in(&editor, window, |this, _, ev: &InputEvent, window, cx| match ev {
            InputEvent::PressEnter { .. } => this.on_editor_enter(window, cx),
            // Currency step highlights the item matching the typed prefix.
            InputEvent::Change => {
                if this.session.editing().is_some_and(|e| e.field == Field::Currency) {
                    cx.notify();
                }
            }
            InputEvent::Blur => {
                if this.session.editing().is_some() {
                    this.commit_and_close(window, cx);
                }
            }
            _ => {}
        });
        (editor, sub)
    }

    pub(super) fn editor_text(&self, cx: &App) -> String {
        self.editor.read(cx).value().to_string()
    }

    /// Applies what the session decided: fill and focus the editor, or close it.
    pub(super) fn apply(&mut self, step: Step, window: &mut Window, cx: &mut Context<Self>) {
        match step {
            Step::Open(_, prefill) => {
                self.editor.update(cx, |s, cx| {
                    s.set_placeholder(prefill.placeholder, window, cx);
                    s.set_value(prefill.text, window, cx);
                    s.select_all(window, cx);
                });
                let handle = self.editor.read(cx).focus_handle(cx);
                window.focus(&handle, cx);
            }
            Step::Close => window.focus(&self.focus, cx),
        }
        cx.notify();
    }

    /// Hands focus back to the app after the popup closed.
    fn refocus_if_was_editing(&self, was_editing: bool, window: &mut Window, cx: &mut Context<Self>) {
        if was_editing {
            window.focus(&self.focus, cx);
        }
    }

    pub fn add_row(&mut self, list: List, window: &mut Window, cx: &mut Context<Self>) {
        let text = self.editor_text(cx);
        let step = self.session.add_row(list, today(), self.settings.currency, &text);
        self.apply(step, window, cx);
    }

    pub fn delete_row(&mut self, list: List, id: u64, cx: &mut Context<Self>) {
        self.session.delete_row(list, id);
        cx.notify();
    }

    /// Opens the popup on a field, committing whatever was being edited.
    pub fn start_edit(&mut self, edit: Edit, window: &mut Window, cx: &mut Context<Self>) {
        let text = self.editor_text(cx);
        match self.session.start_edit(edit, &text) {
            Some(prefill) => self.apply(Step::Open(edit, prefill), window, cx),
            None if self.session.editing().is_none() => self.apply(Step::Close, window, cx),
            None => cx.notify(),
        }
    }

    pub fn commit_and_close(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let was_editing = self.session.editing().is_some();
        let text = self.editor_text(cx);
        self.session.commit_and_close(&text);
        self.refocus_if_was_editing(was_editing, window, cx);
        cx.notify();
    }

    /// Esc: drop the popup text.
    pub fn cancel_edit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let was_editing = self.session.editing().is_some();
        self.session.cancel();
        self.refocus_if_was_editing(was_editing, window, cx);
        cx.notify();
    }

    fn on_editor_enter(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let text = self.editor_text(cx);
        let step = self.session.enter(&text);
        self.apply(step, window, cx);
    }

    /// Currency step: ↑/↓ moves the highlighted choice (wrapping).
    pub fn cycle_currency(&mut self, delta: isize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(edit) = self.session.editing() else { return };
        let options = edit.list.currencies();
        let text = self.editor_text(cx);
        let n = options.len() as isize;
        let next = match Currency::match_prefix(&text, options).and_then(|c| options.iter().position(|o| *o == c)) {
            Some(i) => (i as isize + delta).rem_euclid(n) as usize,
            None if delta < 0 => options.len() - 1,
            None => 0,
        };
        self.editor.update(cx, |s, cx| {
            s.set_value(options[next].code(), window, cx);
            s.select_all(window, cx);
        });
        cx.notify();
    }

    /// Currency step: a dropdown item was clicked; pick it and continue like Enter.
    pub fn pick_currency(&mut self, currency: Currency, window: &mut Window, cx: &mut Context<Self>) {
        let step = self.session.enter(currency.code());
        self.apply(step, window, cx);
    }

    pub fn on_editor_tab(&mut self, back: bool, window: &mut Window, cx: &mut Context<Self>) {
        let text = self.editor_text(cx);
        let step = self.session.tab(back, &text);
        self.apply(step, window, cx);
    }
}
