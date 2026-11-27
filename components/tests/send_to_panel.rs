//! Send to shows in a card's details panel wherever the panel shows on a page that offers it:
//! the drawers and the wide panes beside Add, Remove, the swipe screens and the Oracle tag
//! examples, all one shared panel. A page that doesn't provide Send to, as zwipe.net doesn't,
//! shows none.

#![expect(
    clippy::unwrap_used,
    reason = "a test: the shared fixture card parses, and a panic is the failure report"
)]

use components::{CardDetailsPanel, SendTo};
use dioxus::dioxus_core::NoOpMutations;
use dioxus::prelude::*;
use zwipe_core::domain::card::Card;

/// A real card, as the API serves it.
const CARD_JSON: &str = include_str!("../../zwipe-ui/src/test_support/card.json");

fn card() -> Card {
    serde_json::from_str(CARD_JSON).unwrap()
}

/// Renders `app` once its effects have run.
fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    for _ in 0..4 {
        dom.process_events();
        dom.render_immediate(&mut NoOpMutations);
    }
    dioxus_ssr::render(&dom)
}

#[test]
fn a_wide_pane_s_panel_offers_send_to_where_the_page_does() {
    let html = render(|| {
        use_context_provider(|| SendTo {
            open: Signal::new(false),
            card: Signal::new(None),
            picked: Signal::new(false),
        });
        rsx! { CardDetailsPanel { card: card() } }
    });
    assert!(
        html.contains(r#"data-action="send-to""#),
        "the panel offers Send to: {html}"
    );
}

#[test]
fn a_page_without_send_to_shows_none() {
    let html = render(|| rsx! { CardDetailsPanel { card: card() } });
    assert!(!html.contains(r#"data-action="send-to""#), "{html}");
}
