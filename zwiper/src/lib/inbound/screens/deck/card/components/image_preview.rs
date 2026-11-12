use std::time::Duration;

use components::use_overlay_back_action;
use dioxus::prelude::*;
use tokio::time;
use zwipe_core::domain::card::scryfall_data::{ImageSize, ScryfallData};

use super::flippable_card_image::FlippableCardImage;

/// Milliseconds the dismiss animation plays before the preview clears.
const DISMISS_MS: u64 = 200;

/// Fullscreen image preview overlay with dismiss animation. The payload carries
/// the face to open on (0 = front) so it lands on whichever side the caller was
/// showing; the shared flip control still cycles from there. A tap on it or
/// the OS back closes it, back going to the preview before the screen under it.
#[component]
pub(crate) fn ImagePreview(
    mut card: Signal<Option<(ScryfallData, usize)>>,
    mut dismissing: Signal<bool>,
) -> Element {
    let dismiss = use_callback(move |()| {
        dismissing.set(true);
        spawn(async move {
            time::sleep(Duration::from_millis(DISMISS_MS)).await;
            card.set(None);
            dismissing.set(false);
        });
    });
    let open = use_memo(move || card.read().is_some() && !dismissing());
    use_overlay_back_action(open.into(), dismiss);
    rsx! {
        if card().is_some() || dismissing() {
            div { class: "sheet-backdrop show" }
            div {
                class: if dismissing() {
                    "image-preview-container show dismissing"
                } else {
                    "image-preview-container show"
                },
                onclick: move |_| dismiss.call(()),
                if let Some((sd, face)) = card() {
                    FlippableCardImage {
                        sd,
                        size: ImageSize::Large,
                        class: "card-image".to_string(),
                        initial_face: face,
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use components::{OverlayBackStack, use_overlay_back_stack};
    use zwipe_core::domain::card::Card;

    use super::*;

    /// A real card, as the API serves it.
    const CARD_JSON: &str =
        include_str!("../../../../../../../../zwipe-ui/src/test_support/card.json");

    /// The root's back stack and the preview's signals, reached between renders.
    #[derive(Clone, Copy)]
    struct Probe {
        stack: OverlayBackStack,
        card: Signal<Option<(ScryfallData, usize)>>,
        dismissing: Signal<bool>,
    }

    thread_local! {
        static PROBE: Cell<Option<Probe>> = const { Cell::new(None) };
        static OPEN: Cell<bool> = const { Cell::new(true) };
    }

    #[component]
    fn Root() -> Element {
        let stack = use_overlay_back_stack();
        use_context_provider(|| stack);
        let card = use_signal(|| {
            OPEN.with(Cell::get).then(|| {
                let card: Card = serde_json::from_str(CARD_JSON).unwrap();
                (card.scryfall_data, 0)
            })
        });
        let dismissing = use_signal(|| false);
        PROBE.with(|p| {
            p.set(Some(Probe {
                stack,
                card,
                dismissing,
            }));
        });
        rsx! {
            ImagePreview { card, dismissing }
        }
    }

    fn settle(dom: &mut VirtualDom) {
        for _ in 0..8 {
            dom.process_events();
            dom.render_immediate_to_vec();
        }
    }

    fn mounted(open: bool) -> (VirtualDom, Probe) {
        OPEN.with(|o| o.set(open));
        let mut dom = VirtualDom::new(Root);
        dom.rebuild_in_place();
        settle(&mut dom);
        let probe = PROBE.with(Cell::get).unwrap();
        (dom, probe)
    }

    /// On a runtime, for the dismiss's timer.
    #[tokio::test]
    async fn back_closes_an_open_preview() {
        let (mut dom, probe) = mounted(true);
        let Probe {
            mut stack,
            dismissing,
            ..
        } = probe;
        assert!(
            dom.in_runtime(|| stack.close_top()),
            "the preview takes the back intent"
        );
        settle(&mut dom);
        assert!(
            dom.in_runtime(|| *dismissing.peek()),
            "and starts its dismiss"
        );
        assert!(
            !dom.in_runtime(|| stack.close_top()),
            "a second back goes to the router"
        );
    }

    #[test]
    fn back_passes_by_a_closed_preview() {
        let (dom, probe) = mounted(false);
        let Probe {
            mut stack, card, ..
        } = probe;
        assert!(dom.in_runtime(|| card.peek().is_none()));
        assert!(
            !dom.in_runtime(|| stack.close_top()),
            "nothing open, so back navigates"
        );
    }
}
