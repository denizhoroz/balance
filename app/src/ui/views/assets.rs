//! Assets page: category ring with the total (or hovered category) inside.

use gpui_kit::*;

use crate::core::money::format_compact;
use crate::ui::app::{BalanceApp, View, today};
use crate::ui::components::{big_number, col, row};
use crate::ui::fonts::Fonts;
use crate::ui::format::{format_share, signed_total};
use crate::ui::views::ring::{Segment, ring};

/// Outer box size of the ring.
const RING_SIZE: f32 = 400.;
/// Width of the content inside the ring; the big number must fit in it.
const INNER_WIDTH: f32 = 312.;
const TEXT_WIDTH: f32 = 296.;

impl BalanceApp {
    pub fn render_assets(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let p = self.palette();
        let fonts = Fonts::get(cx).clone();
        let today = today();
        let ledger = self.session.ledger();
        let currency = self.settings.currency;
        let fx = self.fx();
        // The change values the window start at that day's prices.
        let change = ledger.assets_change(self.settings.interval, today, fx, &self.feeds.history);

        // A missing rate empties the ring rather than showing wrong shares.
        let categories = ledger.asset_categories(today, fx).unwrap_or_default();
        let segments = Segment::from_categories(&categories, &p.category_colors(&ledger.assets));
        let ring_total = Segment::total(&segments);
        let hovered = self.hovered_segment.filter(|i| *i < segments.len());

        // Hovering a segment swaps the center to that category.
        let (text, color, caption) = match hovered {
            Some(i) => {
                let seg = &segments[i];
                let caption = div()
                    .text_size(px(16.))
                    .text_color(p.muted)
                    .whitespace_nowrap()
                    .child(format!("{} · {}", seg.name, format_share(seg.value, ring_total)))
                    .into_any_element();
                (format_compact(seg.value, currency), seg.color, caption)
            }
            None => {
                let (text, color) = signed_total(&p, ledger.assets_partial(today, fx), currency);
                (text, color, self.change_row(change.value, None, cx).into_any_element())
            }
        };
        self.fetch_days(change.missing_days, cx);
        let center = col()
            .w(px(INNER_WIDTH))
            .items_center()
            .child(
                big_number(&fonts, text, color, TEXT_WIDTH)
                    .id("assets-total")
                    .cursor_pointer()
                    .on_click(self.open(View::AssetTable, cx)),
            )
            .child(caption);

        let app = cx.entity().downgrade();
        row().size_full().justify_center().child(
            row()
                .relative()
                .size(px(RING_SIZE))
                .justify_center()
                .child(div().absolute().top_0().left_0().size_full().child(ring(segments, hovered, p.line, app)))
                .child(center),
        )
    }
}
