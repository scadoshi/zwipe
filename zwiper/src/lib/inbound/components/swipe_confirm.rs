//! What a swipe screen says after a swipe, as the user's swipe
//! confirmations setting has it: the last action in a quiet chip at the
//! screen's top left, a toast for each, or nothing.

use components::Toasts;
use dioxus::prelude::*;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::user::settings::SwipeConfirmations;
use zwipe_ui::toast::Toast;

use crate::inbound::components::ui_toast;

/// The chip's words, with a count that changes on every confirmation so the
/// same words said twice ease in again.
pub(crate) type Said = Signal<Option<(u32, String)>>;

/// Confirms a swipe's `toast` as the session's setting says: its words in
/// the screen's chip (`said`), the toast itself, or nothing.
pub(crate) fn confirm(
    session: Signal<Option<Session>>,
    toasts: Toasts,
    mut said: Said,
    toast: Toast,
) {
    let setting = session
        .peek()
        .as_ref()
        .map(|s| s.settings.swipe_confirmations)
        .unwrap_or_default();
    match setting {
        SwipeConfirmations::Chip => {
            let count = said.peek().as_ref().map_or(0, |(n, _)| n.wrapping_add(1));
            said.set(Some((count, toast.text)));
        }
        SwipeConfirmations::Toasts => ui_toast::raise(toasts, toast),
        SwipeConfirmations::Off => {}
    }
}

/// The last confirmation at the swipe screen's top left, in the quiet chip.
/// It takes no click and draws nothing until a swipe says something.
#[component]
pub(crate) fn SwipeConfirmChip(said: Said) -> Element {
    rsx! {
        div { class: "swipe-confirm",
            if let Some((count, text)) = said() {
                span { key: "{count}", class: "chip-note swipe-confirm-chip", "{text}" }
            }
        }
    }
}
