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
                    // TODO(wave C): the deck tag list needs a session; the deck
                    // form fetches it through the authed facade.
                    Kind::DeckTags => cell.write().deck_tags.settle(None, Utc::now()),
                }
            });
        }
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

/// Provides the [`CatalogCache`] to everything under the root. Call once,
/// in the root.
pub fn use_catalog_cache() {
    let cache = CatalogCache(use_signal(Catalog::default));
    use_context_provider(|| cache);
}
