//! The card filter as the web draws it: components' [`CardFilterPanel`]
//! with the web's catalog fetching, toasts, the one-time `filter` hint, the
//! Oracle tags info button and the oracle tag dictionary. A host mounts one
//! beside its frame:
//!
//! ```ignore
//! CardFilterPanel {
//!     open: filter_open,
//!     filter,                        // the host's Signal<CardQueryBuilder>
//!     config: SheetConfig { show_format: true, ..SheetConfig::default() },
//!     on_apply: move |applied| { filter.set(applied); bump(); },
//! }
//! ```
//!
//! The one-time `filter` hint opens the first time the user opens a
//! section.

use components::DictionarySlot;
use dioxus::prelude::*;
use dioxus_primitives::toast::use_toast;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_ui::filter::draft::SheetConfig;
use zwipe_ui::stores::catalog::Kind;
use zwipe_ui::toast::Toast;

use crate::inbound::catalog::CatalogCache;
use crate::inbound::chrome::hint_topic::{HintTopic, InfoButton};
use crate::inbound::chrome::one_time_hint::{OneTimeHint, OneTimeHintDialog, open_and_record_hint};
use crate::inbound::chrome::toast::raise;
use crate::inbound::screens::oracle_tag_dictionary::OracleTagDictionary;

/// The filter panel. `filter` is the host's applied filter; `on_apply` takes
/// the committed draft. `quiet_reset` stages the blank filter on `Reset`
/// without a toast, as the commander maybeboard and Swipe select do.
#[component]
pub fn CardFilterPanel(
    open: Signal<bool>,
    filter: ReadSignal<CardQueryBuilder>,
    config: SheetConfig,
    on_apply: EventHandler<CardQueryBuilder>,
    #[props(default)] quiet_reset: bool,
) -> Element {
    let toast = use_toast();
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let cache: CatalogCache = use_context();
    let mut hint_open = use_signal(|| false);
    let dictionary = use_callback(|slot: DictionarySlot| {
        rsx! {
            OracleTagDictionary { open: slot.open, on_use: move |slug: String| slot.on_use.call(slug) }
        }
    });

    rsx! {
        components::CardFilterPanel {
            open,
            filter,
            config,
            on_apply,
            catalog: cache.0,
            on_need: move |kinds: &'static [Kind]| cache.ensure(kinds, client),
            quiet_reset,
            on_toast: move |t: Toast| raise(toast, &t),
            on_hint: move |()| hint_open.set(true),
            on_first_open: move |()| open_and_record_hint(OneTimeHint::Filter, session, client, hint_open),
            oracle_tags_info: rsx! {
                InfoButton { topic: HintTopic::OracleTags }
            },
            hint: rsx! {
                OneTimeHintDialog { hint: OneTimeHint::Filter, open: hint_open }
            },
            dictionary,
        }
    }
}
