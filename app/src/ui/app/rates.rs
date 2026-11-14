//! Background fetching of exchange rates and prices, with retry. The
//! bookkeeping (what is due, in flight, backing off) is `core::feeds`; the
//! URLs are `db/links.json`. Without a usable links file nothing is fetched
//! (cached prices still show).

use std::time::Instant;

use chrono::NaiveDate;
use gpui_kit::Context;

use super::{BalanceApp, today};
use crate::core::sources;

impl BalanceApp {
    /// Fetches every source not yet fetched today and not in flight, each
    /// in its own background task, so fiat totals show as soon as fiat
    /// rates land. A failed source retries itself with backoff, and on
    /// window focus.
    pub(super) fn fetch_rates(&mut self, cx: &mut Context<Self>) {
        let Some(links) = self.links.clone() else { return };
        for source in self.feeds.take_due(today()) {
            let links = links.clone();
            let task = cx.background_executor().spawn(async move { source.fetch(&links) });
            cx.spawn(async move |this, cx| {
                let result = task.await;
                this.update(cx, |this, cx| match result {
                    Ok(part) => {
                        this.feeds.succeed(source, part, today());
                        cx.notify();
                    }
                    Err(e) => {
                        let delay = this.feeds.fail(source);
                        eprintln!("balance: failed to fetch {}: {e:#}; retry in {delay:?}", source.name());
                        cx.spawn(async move |this, cx| {
                            cx.background_executor().timer(delay).await;
                            this.update(cx, |this, cx| this.fetch_rates(cx)).ok();
                        })
                        .detach();
                    }
                })
                .ok();
            })
            .detach();
        }
    }

    /// Fetches the rates of the given past days that are not stored yet,
    /// each in its own background task. Safe to call on every render: days
    /// in flight or backing off are skipped. A failed day retries itself.
    pub fn fetch_days(&mut self, days: impl IntoIterator<Item = NaiveDate>, cx: &mut Context<Self>) {
        let Some(links) = self.links.clone() else { return };
        for day in self.feeds.take_due_days(days, Instant::now()) {
            let links = links.clone();
            let task = cx.background_executor().spawn(async move { sources::fetch_day(&links, day) });
            cx.spawn(async move |this, cx| {
                let result = task.await;
                this.update(cx, |this, cx| match result {
                    Ok(part) => {
                        this.feeds.day_succeeded(day, part);
                        cx.notify();
                    }
                    Err(e) => {
                        let delay = this.feeds.day_failed(day, Instant::now());
                        eprintln!("balance: failed to fetch rates of {day}: {e:#}; retry in {delay:?}");
                        cx.spawn(async move |this, cx| {
                            cx.background_executor().timer(delay).await;
                            this.update(cx, |this, cx| this.fetch_days([day], cx)).ok();
                        })
                        .detach();
                    }
                })
                .ok();
            })
            .detach();
        }
    }
}
