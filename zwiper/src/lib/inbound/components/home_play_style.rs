//! Home's Playstyle panel: the user's top colors, with a button that opens
//! the full screen.

use components::{Button, ButtonVariant};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::scryfall_data::colors::Color;
use zwipe_core::http::contracts::suggestions::{HttpColorIdentityCount, HttpPlayStyleSummary};

use crate::inbound::components::auth::authed::use_authed;
use crate::inbound::components::telemetry::vocabulary::Screen;
use crate::inbound::router::Router;
use crate::inbound::screens::deck::suggested_commanders::{IdentityPips, identity_of};

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

/// The panel's one row: the user's most built color identity, its pips drawn
/// the way the Summary draws them, and the deck count after them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TopColors {
    /// The colors in WUBRG order; empty for colorless.
    pub pips: Vec<Color>,
    /// The deck count, "(4)".
    pub count: String,
}

/// The top colors from the server's identities, most built first. Color
/// codes outside WUBRG are skipped; no identities is no row.
pub(crate) fn top_colors(identities: &[HttpColorIdentityCount]) -> Option<TopColors> {
    let identity = identities.first()?;
    let parsed: Vec<Color> = identity
        .colors
        .iter()
        .filter_map(|code| Color::try_from(code.as_str()).ok())
        .collect();
    Some(TopColors {
        pips: identity_of(&parsed),
        count: format!("({})", identity.decks),
    })
}

/// The panel under the hero, a sibling of the featured flavor card: the
/// title bar, then a Top colors row in the Playstyle Summary's grammar, then
/// an Open button. Before any deck has
/// cards it shows one line instead of the rows; a failed fetch with nothing
/// cached shows a muted line. The button is always there, so the screen
/// stays reachable from Home.
#[component]
pub fn HomePlayStyle() -> Element {
    let navigator = use_navigator();
    let session: Signal<Option<Session>> = use_context();
    let mut cache: Signal<Option<CachedPlayStyle>> = use_context();
    let mut failed = use_signal(|| false);
    let authed = use_authed(Screen::Home);

    let user_id = session.peek().as_ref().map(|s| s.user.id);
    // A copy for this user shows while the refetch is out and is replaced
    // only on success.
    use_effect(move || {
        let Some(user_id) = user_id else {
            return;
        };
        spawn(async move {
            match authed
                .run_quiet("play_style_summary", |c, s| async move {
                    c.play_style_summary(&s).await
                })
                .await
            {
                Some(summary) => cache.set(Some(CachedPlayStyle { user_id, summary })),
                None => failed.set(true),
            }
        });
    });

    let summary = cache
        .read()
        .as_ref()
        .filter(|cached| Some(cached.user_id) == user_id)
        .map(|cached| cached.summary.clone());

    rsx! {
        div { class: "container-sm text-center flex-col home-flavor content-enter-delayed",
            match summary {
                Some(summary) => rsx! {
                    div { class: "settings-list-header", style: "justify-content: center;",
                        span { class: "settings-list-title", "Playstyle" }
                    }
                    if summary.deck_count == 0 {
                        div { class: "message-empty", style: "padding: 1rem;",
                            "Build a deck to see your playstyle"
                        }
                    } else {
                        if let Some(top) = top_colors(&summary.identities) {
                            div { class: "settings-row",
                                span { class: "settings-row-label", "Top colors" }
                                span { class: "info-row-value",
                                    IdentityPips { identity: top.pips, count: top.count }
                                }
                            }
                        }
                    }
                    OpenRow { onclick: move |_| { navigator.push(Router::SuggestedCommanders {}); } }
                },
                None if failed() => rsx! {
                    div { class: "settings-list-header", style: "justify-content: center;",
                        span { class: "settings-list-title", "Playstyle" }
                    }
                    div { class: "message-empty", style: "padding: 1rem;",
                        "Could not load your playstyle"
                    }
                    OpenRow { onclick: move |_| { navigator.push(Router::SuggestedCommanders {}); } }
                },
                None => rsx! {
                    // Mirror the loaded panel: the title bar, the Top
                    // colors row, then the button.
                    div { class: "skeleton-card-header", style: "display: flex; justify-content: center;",
                        div { class: "skeleton-bar skeleton-section-label", style: "width: 45%; margin-bottom: 0;" }
                    }
                    div { class: "skeleton-info-row",
                        div { class: "skeleton-bar skeleton-info-row-label" }
                        div { class: "skeleton-bar skeleton-info-row-value" }
                    }
                    div { class: "flavor-buy-row",
                        div { class: "skeleton-bar skeleton-chip skeleton-chip-md" }
                    }
                },
            }
        }
    }
}

/// The panel's last row: the Open button, centered like the flavor card's
/// name and price row.
#[component]
fn OpenRow(onclick: EventHandler<MouseEvent>) -> Element {
    rsx! {
        div { class: "flavor-buy-row",
            Button { variant: ButtonVariant::Action, onclick: move |e| onclick.call(e), "Open" }
        }
    }
}

#[cfg(test)]
mod tests {
    use Color::{Black, Red, White};

    use super::*;

    fn identity(colors: &[&str], decks: u32) -> HttpColorIdentityCount {
        HttpColorIdentityCount {
            colors: colors.iter().map(|c| c.to_string()).collect(),
            decks,
        }
    }

    #[test]
    fn top_colors_take_the_first_identity_in_wubrg_order() {
        let top = top_colors(&[identity(&["R", "W"], 4), identity(&["U"], 2)]);
        assert_eq!(
            top,
            Some(TopColors {
                pips: vec![White, Red],
                count: "(4)".to_string(),
            })
        );
    }

    #[test]
    fn top_colors_skip_unknown_codes_and_keep_colorless() {
        let top = top_colors(&[identity(&["B", "X"], 3)]);
        assert_eq!(top.map(|t| t.pips), Some(vec![Black]));
        let colorless = top_colors(&[identity(&[], 1)]);
        assert_eq!(colorless.map(|t| t.pips), Some(vec![]));
    }

    #[test]
    fn no_identities_is_no_row() {
        assert_eq!(top_colors(&[]), None);
    }
}
