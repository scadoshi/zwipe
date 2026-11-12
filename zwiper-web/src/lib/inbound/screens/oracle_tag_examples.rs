//! Oracle tag examples: components' screen over the session's card search, with the web's
//! telemetry, one-time hint and toast.

use components::{ExamplesPage, ExamplesSource, OracleTagExamples as SharedExamples, TOAST_QUICK};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::card::search_card::card_filter::card_sort_key::CardSortKey;
use zwipe_ui::telemetry::vocabulary::{OracleTagScreen, Screen};
use zwipe_ui::timing::PAGE_SIZE;

use crate::inbound::cards::otag::use_describe_tag;
use crate::inbound::cards::printing_sheet::use_printings_source;
use crate::inbound::chrome::authed::use_authed;
use crate::inbound::chrome::one_time_hint::{OneTimeHint, use_one_time_hint};
use crate::inbound::telemetry;

/// The examples for `slug`, over the screen while `open`. Mount it fresh per tag.
#[component]
pub fn OracleTagExamples(open: Signal<bool>, slug: String) -> Element {
    telemetry::use_telemetry_screen(Screen::OracleTag(OracleTagScreen::Examples));
    let toast = use_toast();
    let authed = use_authed();
    let describe_tag = use_describe_tag();
    let hint_open = use_one_time_hint(OneTimeHint::OtagExamples);
    use_printings_source();
    use_context_provider(|| ExamplesSource {
        fetch: Callback::new(move |page: ExamplesPage| {
            let ExamplesPage {
                slug,
                offset,
                reply,
            } = page;
            let mut builder = CardQueryBuilder::new();
            builder
                .set_oracle_tags_contains_any(vec![slug])
                .set_is_token(false)
                .set_sort(CardSortKey::EdhrecRank)
                .set_limit(PAGE_SIZE)
                .set_offset(offset);
            let Ok(filter) = builder.build() else {
                reply.call(None);
                return;
            };
            spawn(async move {
                let cards = authed
                    .run("load_examples", |c, s| async move {
                        c.search_cards(&filter, &s).await
                    })
                    .await;
                reply.call(cards);
            });
        }),
    });
    let on_first = use_callback(move |()| {
        toast.info(
            "Already at the first card".to_string(),
            ToastOptions::default().duration(TOAST_QUICK),
        );
    });
    rsx! {
        SharedExamples {
            open,
            slug,
            hint_open,
            describe_tag: Some(describe_tag),
            on_first: Some(on_first),
        }
    }
}
