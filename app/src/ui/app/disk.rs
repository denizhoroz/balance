//! Watching `db/` for external changes.

use std::time::Duration;

use futures::StreamExt;
use gpui_kit::*;
use notify_debouncer_full::notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_full::{DebounceEventResult, Debouncer, RecommendedCache, new_debouncer};

use super::BalanceApp;

/// Keeps the file watcher alive; dropping it stops watching.
pub(super) type Watcher = Debouncer<RecommendedWatcher, RecommendedCache>;

impl BalanceApp {
    /// Something changed in `db/` (or window refocused): let the session
    /// reload or retry a pending save.
    pub(super) fn on_disk_event(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let was_editing = self.session.editing().is_some();
        if self.session.sync() {
            if was_editing && self.session.editing().is_none() {
                window.focus(&self.focus, cx);
            }
            self.close_stale_asset_view(window, cx);
            cx.notify();
        }
    }

    pub(super) fn start_watcher(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let (tx, mut rx) = futures::channel::mpsc::unbounded::<()>();
        let debouncer = new_debouncer(Duration::from_millis(300), None, move |res: DebounceEventResult| {
            if res.is_ok() {
                let _ = tx.unbounded_send(());
            }
        });
        let mut debouncer = match debouncer {
            Ok(d) => d,
            Err(e) => {
                eprintln!("balance: file watcher unavailable: {e}");
                return;
            }
        };
        if let Err(e) = debouncer.watch(&self.db, RecursiveMode::NonRecursive) {
            eprintln!("balance: cannot watch {}: {e}", self.db.display());
            return;
        }
        self._watcher = Some(debouncer);
        cx.spawn_in(window, async move |this, cx| {
            while rx.next().await.is_some() {
                // Drain bursts into one check.
                while rx.try_recv().is_ok() {}
                if this.update_in(cx, |this, window, cx| this.on_disk_event(window, cx)).is_err() {
                    break;
                }
            }
        })
        .detach();
    }
}
