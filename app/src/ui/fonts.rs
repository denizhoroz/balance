//! Embedded Inter fonts (Thin for the balance number, Light for the rest).

use std::borrow::Cow;

use gpui_kit::{App, FontWeight, Global, SharedString};

static INTER_THIN: &[u8] = include_bytes!("../../../assets/fonts/Inter-Thin.ttf");
static INTER_LIGHT: &[u8] = include_bytes!("../../../assets/fonts/Inter-Light.ttf");

#[derive(Clone)]
pub struct Fonts {
    pub thin_family: SharedString,
    pub light_family: SharedString,
}

impl Global for Fonts {}

impl Fonts {
    pub const THIN: FontWeight = FontWeight::THIN;
    pub const LIGHT: FontWeight = FontWeight::LIGHT;

    pub fn get(cx: &App) -> &Fonts {
        cx.global::<Fonts>()
    }
}

/// Registers the embedded fonts and records the family names the platform
/// reports for them (DirectWrite may expose `Inter` or `Inter Thin`).
pub fn register(cx: &mut App) {
    if let Err(e) = cx
        .text_system()
        .add_fonts(vec![Cow::Borrowed(INTER_THIN), Cow::Borrowed(INTER_LIGHT)])
    {
        eprintln!("balance: failed to load fonts: {e}");
    }
    let names = cx.text_system().all_font_names();
    let pick = |specific: &str| -> SharedString {
        if names.iter().any(|n| n == specific) {
            SharedString::from(specific.to_string())
        } else {
            SharedString::from("Inter")
        }
    };
    let fonts = Fonts {
        thin_family: pick("Inter Thin"),
        light_family: pick("Inter Light"),
    };
    if cfg!(debug_assertions) {
        let inter: Vec<_> = names.iter().filter(|n| n.contains("Inter")).collect();
        eprintln!("balance: Inter families {inter:?}");
    }
    cx.set_global(fonts);
}
