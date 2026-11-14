//! Small reusable building blocks shared by all views.

use gpui_kit::component::{Icon, IconName};
use gpui_kit::*;

use crate::core::ledger::{Change, format_change};
use crate::ui::app::{BalanceApp, View};
use crate::ui::fonts::Fonts;
use crate::ui::format::big_font_size;

/// Horizontal flex container with vertically centered children.
pub fn row() -> Div {
    div().flex().items_center()
}

/// Vertical flex container.
pub fn col() -> Div {
    div().flex().flex_col()
}

/// Fixed-width, non-shrinking element.
pub fn fixed<E: Styled>(el: E, width: f32) -> E {
    el.flex_none().w(px(width))
}

/// Clickable element: pointer cursor, `color` that turns `hover` on hover.
/// Attach the handler with `.on_click(..)`.
pub fn link(id: impl Into<ElementId>, color: Hsla, hover: Hsla) -> Stateful<Div> {
    row().id(id).cursor_pointer().text_color(color).hover(move |s| s.text_color(hover))
}

pub fn icon_button(
    id: &'static str,
    icon: impl Into<Icon>,
    color: Hsla,
    hover: Hsla,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    link(id, color, hover).p(px(6.)).child(Icon::new(icon).size(px(16.))).on_click(on_click)
}

/// Large thin-weight number whose font size shrinks to fit `max_width`.
pub fn big_number(fonts: &Fonts, text: String, color: Hsla, max_width: f32) -> Div {
    div()
        .font_family(fonts.thin_family.clone())
        .font_weight(Fonts::THIN)
        .text_size(px(big_font_size(&text, max_width)))
        .text_color(color)
        .whitespace_nowrap()
        .child(text)
}

/// Adds an action handler that stops propagation before running `f`.
/// Used for keys the popup must handle itself (Esc, Tab, arrows).
pub fn capture<A: Action, E: InteractiveElement>(
    el: E,
    cx: &mut Context<BalanceApp>,
    f: impl Fn(&mut BalanceApp, &mut Window, &mut Context<BalanceApp>) + 'static,
) -> E {
    el.capture_action(cx.listener(move |this, _: &A, window, cx| {
        cx.stop_propagation();
        f(this, window, cx);
    }))
}

impl BalanceApp {
    /// Click handler that switches to `view`.
    pub fn open(&self, view: View, cx: &mut Context<Self>) -> impl Fn(&ClickEvent, &mut Window, &mut App) + 'static {
        cx.listener(move |this, _: &ClickEvent, window, cx| this.show(view, window, cx))
    }

    /// Change % centered on its own; the clickable interval label hangs off
    /// its right side. Equal flex_1 gutters keep the change centered.
    /// With `open`, clicking the change text itself opens that view.
    pub fn change_row(&self, change: Change, open: Option<View>, cx: &mut Context<Self>) -> Div {
        let p = self.palette();
        let text = div()
            .id("change")
            .flex_none()
            .text_size(px(22.))
            .text_color(p.change(change))
            .child(format_change(change));
        let text = match open {
            Some(view) => text.cursor_pointer().on_click(self.open(view, cx)),
            None => text,
        };
        row()
            .w_full()
            .child(div().flex_1())
            .child(text)
            .child(
                row().flex_1().pl(px(8.)).child(
                    link("interval", p.muted, p.fg)
                        .text_size(px(14.))
                        .child(self.settings.interval.label())
                        .on_click(cx.listener(|this, _: &ClickEvent, _, cx| this.cycle_interval(cx))),
                ),
            )
    }

    /// Icon-only add button for table columns.
    pub fn add_button(&self, id: SharedString, on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Stateful<Div> {
        let p = self.palette();
        link(id, p.muted, p.fg).h_full().px(px(8.)).child(Icon::new(IconName::Plus).size(px(14.))).on_click(on_click)
    }
}
