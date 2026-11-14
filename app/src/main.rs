// Hide the console window in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// GPUI macro output refers to `gpui::...`; resolve it through the Kit.
extern crate gpui_kit as gpui;

mod core;
mod ui;

use std::borrow::Cow;

use gpui_kit::*;

gpui_kit::assets::icon_assets!(ExtraIcons, [ArrowRightLeft, Wallet]);

/// Default component icons plus the few extras the app uses.
struct AppAssets;

impl AssetSource for AppAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if let Some(bytes) = ExtraIcons.load(path)? {
            return Ok(Some(bytes));
        }
        gpui_kit::assets::Assets.load(path)
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let mut paths = gpui_kit::assets::Assets.list(path)?;
        paths.extend(ExtraIcons.list(path)?);
        paths.sort();
        paths.dedup();
        Ok(paths)
    }
}

fn main() {
    gpui_kit::application().with_assets(AppAssets).run(|cx| {
        gpui_kit::init(cx);
        ui::fonts::register(cx);
        ui::app::init(cx);

        // Workaround: on Windows the process can hang after the last window
        // closes, so quit explicitly.
        cx.on_window_closed(|cx, _| cx.quit()).detach();

        let size = size(px(560.), px(640.));
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds::centered(None, size, cx))),
            titlebar: Some(TitlebarOptions {
                title: Some("balance".into()),
                ..Default::default()
            }),
            is_resizable: false,
            ..Default::default()
        };
        gpui_kit::open_window(options, cx, |window, cx| cx.new(|cx| ui::app::BalanceApp::new(window, cx)))
            .expect("failed to open window");
        cx.activate(true);
    });
}
