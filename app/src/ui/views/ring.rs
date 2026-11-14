//! Thin category ring drawn on a canvas, with hover detection.

use std::collections::HashMap;
use std::f32::consts::TAU;

use gpui_kit::*;

use crate::core::money::Cents;
use crate::ui::app::BalanceApp;

const STROKE: f32 = 2.5;
const STROKE_HOVER: f32 = 5.;
/// Arc gap between segments.
const GAP: f32 = 6.;
/// Mouse distance from the ring line that still counts as hovering it.
const HIT: f32 = 14.;

/// One ring segment; angles in radians, clockwise from 12 o'clock.
#[derive(Clone)]
pub struct Segment {
    pub name: String,
    pub value: Cents,
    pub color: Hsla,
    start: f32,
    sweep: f32,
}

impl Segment {
    /// Segments for the positive categories, sized by share of their sum.
    pub fn from_categories(categories: &[(String, Cents)], colors: &HashMap<String, Hsla>) -> Vec<Segment> {
        let positive: Vec<_> = categories.iter().filter(|(_, v)| *v > 0).collect();
        let total: Cents = positive.iter().map(|(_, v)| *v).sum();
        let mut start = 0.0;
        positive
            .into_iter()
            .map(|(name, value)| {
                let sweep = (*value as f64 / total as f64) as f32 * TAU;
                let seg = Segment { name: name.clone(), value: *value, color: colors[name], start, sweep };
                start += sweep;
                seg
            })
            .collect()
    }

    pub fn total(segments: &[Segment]) -> Cents {
        segments.iter().map(|s| s.value).sum()
    }
}

/// Ring split into category segments. Tracks which segment the mouse is
/// over and stores it in `BalanceApp::hovered_segment`.
pub fn ring(
    segments: Vec<Segment>,
    hovered: Option<usize>,
    empty: Hsla,
    app: WeakEntity<BalanceApp>,
) -> impl IntoElement {
    canvas(
        |_, _, _| (),
        move |bounds, _, window, _| {
            let center = bounds.center();
            let side = f32::from(bounds.size.width).min(f32::from(bounds.size.height));
            let r = side / 2.0 - STROKE_HOVER;
            let at = |a: f32| point(center.x + px(r * a.sin()), center.y - px(r * a.cos()));
            let arc = |start: f32, sweep: f32, width: f32, color: Hsla, window: &mut Window| {
                let steps = (sweep / TAU * 240.0).ceil().max(2.0) as usize;
                let mut b = PathBuilder::stroke(px(width));
                b.move_to(at(start));
                for i in 1..=steps {
                    b.line_to(at(start + sweep * i as f32 / steps as f32));
                }
                if let Ok(path) = b.build() {
                    window.paint_path(path, color);
                }
            };

            if segments.is_empty() {
                arc(0.0, TAU, STROKE, empty, window);
            } else {
                let gap = if segments.len() > 1 { GAP / r } else { 0.0 };
                for (i, s) in segments.iter().enumerate() {
                    let g = gap.min(s.sweep * 0.5);
                    let width = if hovered == Some(i) { STROKE_HOVER } else { STROKE };
                    arc(s.start + g / 2.0, s.sweep - g, width, s.color, window);
                }
            }

            let spans: Vec<(f32, f32)> = segments.iter().map(|s| (s.start, s.sweep)).collect();
            window.on_mouse_event(move |ev: &MouseMoveEvent, phase, _, cx| {
                if phase != DispatchPhase::Bubble {
                    return;
                }
                let dx = f32::from(ev.position.x - center.x);
                let dy = f32::from(ev.position.y - center.y);
                let dist = (dx * dx + dy * dy).sqrt();
                let hit = if (dist - r).abs() <= HIT {
                    let a = dx.atan2(-dy).rem_euclid(TAU);
                    spans.iter().position(|(s, w)| a >= *s && a < s + w)
                } else {
                    None
                };
                app.update(cx, |this, cx| {
                    if this.hovered_segment != hit {
                        this.hovered_segment = hit;
                        cx.notify();
                    }
                })
                .ok();
            });
        },
    )
    .size_full()
}
