//! Home's Colors panel, the shared `components` piece with its "?" wired to
//! the app's hint host, and the summary the app caches above the router for
//! it.

pub(crate) use components::ColorsEmpty;
use components::HomeColors;
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::http::contracts::suggestions::HttpPlayStyleSummary;
use zwipe_ui::screens::home::PlayStyle;

use crate::inbound::components::hint_host::HintTopic;

/// The last summary fetched, kept above the router (as
/// `Signal<Option<CachedPlayStyle>>`) so a return to Home shows it at once
/// while the refetch is out.
#[derive(Debug, Clone, PartialEq)]
pub struct CachedPlayStyle {
    /// The user the summary belongs to; another user's copy is never shown.
    pub user_id: Uuid,
    /// What zerver answered.
    pub summary: HttpPlayStyleSummary,
}

/// The shared Colors panel, its "?" posting the Colors hint to the app's
/// host.
#[component]
pub fn HomePlayStyle(play_style: PlayStyle, onplaystyle: EventHandler) -> Element {
    let mut hint: Signal<Option<HintTopic>> = use_context();
    rsx! {
        HomeColors {
            play_style,
            onplaystyle,
            oninfo: move |()| hint.set(Some(HintTopic::HomeColors)),
        }
    }
}
