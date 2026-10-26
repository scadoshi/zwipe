//! Home's Playstyle panel: the user's deck count and most built color
//! sets, with a button that opens the full screen.

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

/// The row labels for the identities, in the server's order.
const SET_LABELS: [&str; 3] = ["Top set", "2nd set", "3rd set"];

/// One identity row: its label, the pips drawn the way the Summary draws
/// them, and the deck count after them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SetRow {
    /// "Top set", "2nd set" or "3rd set".
    pub label: &'static str,
    /// The colors in WUBRG order; empty for colorless.
    pub pips: Vec<Color>,
    /// The deck count, "(4)".
    pub count: String,
}

/// The identity rows for the panel, one per identity the server sent and at
/// most one per label. Color codes outside WUBRG are skipped.
pub(crate) fn set_rows(identities: &[HttpColorIdentityCount]) -> Vec<SetRow> {
    SET_LABELS
        .iter()
        .zip(identities)
        .map(|(label, identity)| {
            let parsed: Vec<Color> = identity
                .colors
                .iter()
                .filter_map(|code| Color::try_from(code.as_str()).ok())
                .collect();
            SetRow {
                label,
                pips: identity_of(&parsed),
                count: format!("({})", identity.decks),
            }
        })
        .collect()
}

/// The panel under the hero, a sibling of the featured flavor card: the
/// title bar, then the Decks row and up to three color set rows in the
/// Playstyle Summary's grammar, then an Open button. Before any deck has
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
                        div { style: "display:flex;flex-direction:column;",
                            div { class: "settings-row",
                                span { class: "settings-row-label", "Decks" }
                                span { class: "info-row-value", "{summary.deck_count}" }
                            }
                            for row in set_rows(&summary.identities) {
                                div { key: "{row.label}", class: "settings-row",
                                    span { class: "settings-row-label", "{row.label}" }
                                    span { class: "info-row-value",
                                        IdentityPips { identity: row.pips, count: row.count }
                                    }
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
                    // Mirror the loaded panel: the title bar, the Decks row
                    // and three set rows, then the button.
                    div { class: "skeleton-card-header", style: "display: flex; justify-content: center;",
                        div { class: "skeleton-bar skeleton-section-label", style: "width: 45%; margin-bottom: 0;" }
                    }
                    div { style: "display:flex;flex-direction:column;",
                        for i in 0..4 {
                            div { key: "{i}", class: "skeleton-info-row",
                                div { class: "skeleton-bar skeleton-info-row-label" }
                                div { class: "skeleton-bar skeleton-info-row-value" }
                            }
                        }
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
    use Color::{Black, Blue, Green, Red, White};

    use super::*;

    fn identity(colors: &[&str], decks: u32) -> HttpColorIdentityCount {
        HttpColorIdentityCount {
            colors: colors.iter().map(|c| c.to_string()).collect(),
            decks,
        }
    }

    #[test]
    fn set_rows_label_in_order_and_draw_wubrg_pips() {
        let rows = set_rows(&[
            identity(&["R", "W"], 4),
            identity(&["U"], 2),
            identity(&[], 1),
        ]);
        assert_eq!(
            rows,
            vec![
                SetRow {
                    label: "Top set",
                    pips: vec![White, Red],
                    count: "(4)".to_string(),
                },
                SetRow {
                    label: "2nd set",
                    pips: vec![Blue],
                    count: "(2)".to_string(),
                },
                SetRow {
                    label: "3rd set",
                    pips: vec![],
                    count: "(1)".to_string(),
                },
            ]
        );
    }

    #[test]
    fn set_rows_stop_at_three_and_skip_unknown_codes() {
        let rows = set_rows(&[
            identity(&["B", "X"], 3),
            identity(&["G"], 2),
            identity(&["W"], 1),
            identity(&["U"], 1),
        ]);
        assert_eq!(rows.len(), 3, "a fourth identity has no label");
        assert_eq!(rows.first().map(|r| r.pips.clone()), Some(vec![Black]));
        assert_eq!(rows.get(1).map(|r| r.pips.clone()), Some(vec![Green]));
    }

    #[test]
    fn no_identities_is_no_rows() {
        assert!(set_rows(&[]).is_empty());
    }
}
