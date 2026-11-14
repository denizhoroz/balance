//! Color palette per theme. Dark uses neon on near-black; light uses darker
//! variants of the same hues on white (neon is unreadable on white).

use std::collections::HashMap;

use gpui_kit::{Hsla, rgb};

use crate::core::assets::{self, Snapshot};
use crate::core::ledger::Change;
use crate::core::money::Cents;
use crate::core::session::List;
use crate::core::settings::Theme;

#[derive(Clone, Copy)]
pub struct Palette {
    pub bg: Hsla,
    pub fg: Hsla,
    /// Secondary text: labels, dates, zero values.
    pub muted: Hsla,
    /// Center divider line on the main view.
    pub line: Hsla,
    pub selection: Hsla,
    /// Raised surface for popovers (dropdown list); no borders anywhere.
    pub surface: Hsla,
    pub green: Hsla,
    pub red: Hsla,
    pub yellow: Hsla,
    /// Asset category colors (ring segments). Green/red/yellow are kept for
    /// +/-/0 meaning, so categories use other hues.
    pub categories: [Hsla; 8],
}

impl Palette {
    pub fn for_theme(theme: Theme) -> Palette {
        match theme {
            Theme::Dark => Palette {
                bg: rgb(0x0A0A0A).into(),
                fg: rgb(0xEDEDED).into(),
                muted: rgb(0x7A7A7A).into(),
                line: rgb(0x3A3A3A).into(),
                selection: rgb(0x2A2A2A).into(),
                surface: rgb(0x181818).into(),
                green: rgb(0x39FF14).into(),
                red: rgb(0xFF3131).into(),
                yellow: rgb(0xFFF01F).into(),
                categories: [
                    rgb(0x00E5FF).into(), // cyan
                    rgb(0xFF8A1F).into(), // orange
                    rgb(0xB066FF).into(), // violet
                    rgb(0xFF3DCB).into(), // magenta
                    rgb(0x3D8BFF).into(), // blue
                    rgb(0xD4A373).into(), // tan
                    rgb(0xC9B3FF).into(), // lavender
                    rgb(0x7FD4FF).into(), // sky
                ],
            },
            Theme::Light => Palette {
                bg: rgb(0xFFFFFF).into(),
                fg: rgb(0x111111).into(),
                muted: rgb(0x8A8A8A).into(),
                line: rgb(0xD0D0D0).into(),
                selection: rgb(0xE6E6E6).into(),
                surface: rgb(0xF2F2F2).into(),
                green: rgb(0x00A86B).into(),
                red: rgb(0xD7263D).into(),
                yellow: rgb(0xC9A400).into(),
                categories: [
                    rgb(0x0097A7).into(),
                    rgb(0xE65100).into(),
                    rgb(0x7B1FA2).into(),
                    rgb(0xC2185B).into(),
                    rgb(0x1565C0).into(),
                    rgb(0x6D4C41).into(),
                    rgb(0x8E6CC9).into(),
                    rgb(0x2B8CC4).into(),
                ],
            },
        }
    }

    /// Green if positive, red if negative, yellow if zero.
    pub fn signed(&self, v: Cents) -> Hsla {
        match v.signum() {
            1 => self.green,
            -1 => self.red,
            _ => self.yellow,
        }
    }

    /// `color`, or muted when the value is zero.
    pub fn nonzero(&self, v: Cents, color: Hsla) -> Hsla {
        if v == 0 { self.muted } else { color }
    }

    /// Accent of a table list: income green, expense red, assets plain.
    pub fn list(&self, list: List) -> Hsla {
        match list {
            List::Income => self.green,
            List::Expense => self.red,
            List::Asset => self.fg,
        }
    }

    /// Color of a period-over-period change.
    pub fn change(&self, change: Change) -> Hsla {
        match change {
            Change::Undefined | Change::Unavailable => self.muted,
            Change::Percent(v) if v > 0.0 => self.green,
            Change::Percent(v) if v < 0.0 => self.red,
            Change::Percent(_) => self.yellow,
        }
    }

    /// Stable color per category: the palette's colors in
    /// [`assets::category_order`]. New categories take the next color, so
    /// existing ones never change color.
    pub fn category_colors(&self, snapshots: &[Snapshot]) -> HashMap<String, Hsla> {
        let n = self.categories.len();
        assets::category_order(snapshots).into_iter().enumerate().map(|(i, c)| (c, self.categories[i % n])).collect()
    }
}
