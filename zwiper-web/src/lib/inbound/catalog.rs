//! The catalog lists the root keeps for the visit: zwipe-ui's [`Catalog`]
//! in a signal, fetched on demand by the screens that read it.
//!
//! zwipe-ui decides when a list needs fetching (missing, failed or a day
//! old) and keeps one fetch per list in flight; this module runs the fetches.

use chrono::Utc;
use dioxus::core::spawn_forever;
use dioxus::prelude::*;
use zwipe_client::ZwipeClient;
use zwipe_ui::stores::catalog::{Catalog, Kind};

use crate::inbound::chrome::authed::Authed;
use crate::inbound::root_signal::use_root_signal;

/// The root's catalog lists. `Copy`; read a list with
/// `cache.0.read().oracle_tags.ready()`.
#[derive(Debug, Clone, Copy)]
pub struct CatalogCache(pub Signal<Catalog>);

impl CatalogCache {
    /// Fetches each of `kinds` that is missing, failed or stale, unless a
    /// fetch for it is already out. A fetch outlives the screen that asked,
    /// so a screen leaving mid-fetch never strands a list.
    pub fn ensure(mut self, kinds: &[Kind], client: Signal<ZwipeClient>) {
        let claimed = self.0.write().claim(kinds, Utc::now());
        for kind in claimed {
            let mut cell = self.0;
            spawn_forever(async move {
                let http = client.peek().clone();
                let label = format!("{kind:?}");
                macro_rules! settle {
                    ($slot:ident, $fetch:expr) => {{
                        let answer = $fetch
                            .await
                            .inspect_err(|e| tracing::warn!("catalog {label} not fetched: {e}"))
                            .ok();
                        cell.write().$slot.settle(answer, Utc::now());
                    }};
                }
                match kind {
                    Kind::Keywords => settle!(keywords, http.fetch_keywords()),
                    Kind::KeywordReminders => {
                        settle!(keyword_reminders, http.fetch_keyword_reminders());
                    }
                    Kind::OracleWords => settle!(oracle_words, http.fetch_oracle_words()),
                    Kind::CardTypes => settle!(card_types, http.fetch_card_types()),
                    Kind::Artists => settle!(artists, http.fetch_artists()),
                    Kind::Sets => settle!(sets, http.fetch_sets()),
                    Kind::CardRoles => settle!(card_roles, http.fetch_card_roles()),
                    Kind::OracleTags => settle!(oracle_tags, http.fetch_oracle_tags()),
                    Kind::UbFranchises => settle!(ub_franchises, http.fetch_ub_franchises()),
                    // The deck tag list needs a session: see `ensure_deck_tags`.
                    Kind::DeckTags => cell.write().deck_tags.settle(None, Utc::now()),
                }
            });
        }
    }

    /// Fetches the deck tag catalog when it needs it, through the authed
    /// facade since the list needs a session. Quiet: the picker shows
    /// `No results` until it loads, as in the app.
    pub fn ensure_deck_tags(mut self, authed: Authed) {
        if !self.0.write().deck_tags.claim(Utc::now()) {
            return;
        }
        let mut cell = self.0;
        spawn_forever(async move {
            let answer = authed
                .run_quiet("load_deck_tags", |c, s| async move {
                    c.fetch_deck_tags(&s).await
                })
                .await;
            cell.write().deck_tags.settle(answer, Utc::now());
        });
    }

    /// Fetches the oracle tag catalog when it needs it, for the screens that
    /// describe tags.
    pub fn ensure_oracle_tags(self, client: Signal<ZwipeClient>) {
        self.ensure(&[Kind::OracleTags], client);
    }

    /// The description of the oracle tag `slug`, once the catalog has loaded
    /// and the tag has one.
    pub fn describe(self, slug: &str) -> Option<String> {
        self.0
            .read()
            .oracle_tags
            .ready()?
            .iter()
            .find(|tag| tag.slug == slug)
            .and_then(|tag| tag.description.clone())
    }
}

/// Provides the [`CatalogCache`] to everything under the root, owned by the
/// root scope its fetches run in. Call once, in the root.
pub fn use_catalog_cache() {
    let cache = CatalogCache(use_root_signal(Catalog::default));
    use_context_provider(|| cache);
}
