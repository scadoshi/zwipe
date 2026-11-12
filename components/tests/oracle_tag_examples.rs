//! The Oracle tag examples ask the host's [`ExamplesSource`] for the tag's first page and deal
//! what it replies; a failed or empty reply says the tag has no examples yet.

#![expect(
    clippy::unwrap_used,
    reason = "test helpers: a failed read or render fails the test"
)]

use components::{ExamplesPage, ExamplesSource, OracleTagExamples};
use dioxus::dioxus_core::NoOpMutations;
use dioxus::prelude::*;
use zwipe_core::domain::card::Card;

/// A real card as the API serves it, shared with zwipe-ui's tests.
const CARD_JSON: &str = include_str!("../../zwipe-ui/src/test_support/card.json");

fn card() -> Card {
    serde_json::from_str(CARD_JSON).unwrap()
}

/// Renders `app` once its effects have run, inside the runtime the kit's sheets time their
/// entrance on.
fn render(app: fn() -> Element) -> String {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .unwrap();
    let _runtime = runtime.enter();
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    for _ in 0..8 {
        dom.process_events();
        dom.render_immediate(&mut NoOpMutations);
    }
    dioxus_ssr::render(&dom)
}

/// The examples of `burn` under a source that records what it was asked and replies with
/// `cards`.
fn examples(cards: Option<Vec<Card>>) -> Element {
    let dialogs = components::use_dialog_host();
    use_context_provider(|| dialogs);
    let mut asked = use_signal(Vec::new);
    use_context_provider(|| ExamplesSource {
        fetch: Callback::new(move |page: ExamplesPage| {
            asked.write().push(format!("{}@{}", page.slug, page.offset));
            page.reply.call(cards.clone());
        }),
    });
    rsx! {
        p { class: "asked", "{asked.read().join(\",\")}" }
        OracleTagExamples {
            open: use_signal(|| true),
            slug: "burn".to_string(),
            hint_open: use_signal(|| false),
        }
    }
}

#[test]
fn the_examples_ask_for_the_tag_s_first_page_and_deal_it() {
    let html = render(|| examples(Some(vec![card()])));
    assert!(html.contains(r#"<p class="asked">burn@0</p>"#), "{html}");
    assert!(
        html.contains("Research Assistant"),
        "the card deals: {html}"
    );
    assert!(html.contains("Back one"), "the dock: {html}");
}

#[test]
fn a_failed_reply_says_there_are_no_examples_yet() {
    let html = render(|| examples(None));
    assert!(
        html.contains("No example cards for this tag yet."),
        "{html}"
    );
}

#[test]
fn an_empty_reply_says_there_are_no_examples_yet() {
    let html = render(|| examples(Some(Vec::new())));
    assert!(
        html.contains("No example cards for this tag yet."),
        "{html}"
    );
}
