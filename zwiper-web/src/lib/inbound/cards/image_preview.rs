//! The app's full-window card image, opened by a row's or a panel's
//! `Image`.

use std::time::Duration;

use components::use_overlay_back;
use dioxus::prelude::*;
use zwipe_core::domain::card::scryfall_data::{ImageSize, ScryfallData};

use crate::outbound::timer;

/// How long the dismiss animation plays before the preview clears.
const DISMISS: Duration = Duration::from_millis(200);

/// The card in `card` over everything, opened on its face (0 = front). A
/// click or Esc dismisses it; the flip control still turns a multi-faced
/// card, and F flips it too.
#[component]
pub fn ImagePreview(mut card: Signal<Option<(ScryfallData, usize)>>) -> Element {
    let mut dismissing = use_signal(|| false);
    let mut open = use_signal(|| false);
    use_overlay_back(open);
    // The back stack closes `open`; the card follows it.
    use_effect(move || {
        let shown = card().is_some();
        if shown != *open.peek() {
            open.set(shown);
        }
    });
    use_effect(move || {
        if !open() && card.peek().is_some() {
            card.set(None);
        }
    });
    let mut dismiss = move || {
        dismissing.set(true);
        spawn(async move {
            timer::sleep(DISMISS).await;
            card.set(None);
            dismissing.set(false);
        });
    };
    rsx! {
        if card().is_some() || dismissing() {
            div { class: "sheet-backdrop show web-image-preview-backdrop" }
            div {
                class: if dismissing() { "image-preview-container show dismissing" } else { "image-preview-container show" },
                tabindex: "-1",
                onmounted: move |e| async move {
                    let _ = e.set_focus(true).await;
                },
                onkeydown: move |e: Event<KeyboardData>| {
                    if e.key() == Key::Character("f".into()) || e.key() == Key::Character("F".into()) {
                        flip_shown_card();
                    }
                },
                onclick: move |_| dismiss(),
                if let Some((sd, face)) = card() {
                    components::FlippableCardImage {
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

/// Presses the flip control of the open preview, if the card has one.
fn flip_shown_card() {
    document::eval(
        r#"const flip = document.querySelector('.image-preview-container [aria-label="Flip card"]'); if (flip) flip.click();"#,
    );
}
