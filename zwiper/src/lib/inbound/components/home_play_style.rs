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
use crate::inbound::screens::deck::suggested_commanders::{
    IdentityPips, identity_label, identity_of,
};

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

/// One step of the podium: a color identity, where it placed and how many
/// decks use it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PodiumStep {
    /// 1 for the most built identity, then 2 and 3.
    pub place: usize,
    /// The colors in WUBRG order; empty for colorless.
    pub pips: Vec<Color>,
    /// How many decks use the identity.
    pub decks: u32,
}

impl PodiumStep {
    /// "1st", "2nd" or "3rd".
    pub(crate) fn place_label(&self) -> &'static str {
        match self.place {
            1 => "1st",
            2 => "2nd",
            _ => "3rd",
        }
    }

    /// The step's class, which sets its fixed height: first tallest, so the
    /// podium reads as one however lopsided the counts are.
    pub(crate) fn class(&self) -> &'static str {
        match self.place {
            1 => "podium-block podium-first",
            2 => "podium-block podium-second",
            _ => "podium-block podium-third",
        }
    }
}

/// The podium's steps from the server's identities (most built first), in
/// standing order: second, first, third. Fewer than three keep that order
/// without the missing places. Color codes outside WUBRG are skipped.
pub(crate) fn podium(identities: &[HttpColorIdentityCount]) -> Vec<PodiumStep> {
    let ranked: Vec<PodiumStep> = identities
        .iter()
        .take(3)
        .enumerate()
        .map(|(index, identity)| {
            let parsed: Vec<Color> = identity
                .colors
                .iter()
                .filter_map(|code| Color::try_from(code.as_str()).ok())
                .collect();
            PodiumStep {
                place: index + 1,
                pips: identity_of(&parsed),
                decks: identity.decks,
            }
        })
        .collect();
    [2, 1, 3]
        .iter()
        .filter_map(|place| ranked.iter().find(|step| step.place == *place).cloned())
        .collect()
}

/// Fetches the playstyle summary into the cache above the router and returns
/// this user's copy, if any, with whether the last fetch failed. A cached
/// copy shows while the refetch is out and is replaced only on success.
/// Home's panel and the Playstyle screen's podium share it.
pub(crate) fn use_play_style_summary(
    screen: Screen,
) -> (Option<HttpPlayStyleSummary>, Signal<bool>) {
    let session: Signal<Option<Session>> = use_context();
    let mut cache: Signal<Option<CachedPlayStyle>> = use_context();
    let mut failed = use_signal(|| false);
    let authed = use_authed(screen);

    let user_id = session.peek().as_ref().map(|s| s.user.id);
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
    (summary, failed)
}

/// The top color identities standing on a podium: second, first, third,
/// each step with its deck count on top and its pips, name and place under
/// it. Renders nothing without identities.
#[component]
pub(crate) fn Podium(identities: Vec<HttpColorIdentityCount>) -> Element {
    let steps = podium(&identities);
    if steps.is_empty() {
        return rsx! {};
    }
    rsx! {
        div { class: "podium",
            for step in steps {
                div { key: "{step.place}", class: "podium-step",
                    span { class: "podium-count", "{step.decks}" }
                    div { class: step.class() }
                    IdentityPips { identity: step.pips.clone(), show_name: false }
                    if let Some(name) = identity_label(&step.pips) {
                        span { class: "podium-name", "{name}" }
                    }
                    span { class: "podium-place", "{step.place_label()}" }
                }
            }
        }
    }
}

/// The panel under the hero, a sibling of the featured flavor card: the
/// title bar, then the top three color identities as a podium (second,
/// first, third, each step with its deck count on top), then an Open button. Before any deck has
/// cards it shows one line instead of the rows; a failed fetch with nothing
/// cached shows a muted line. The button is always there, so the screen
/// stays reachable from Home.
#[component]
pub fn HomePlayStyle() -> Element {
    let navigator = use_navigator();
    let (summary, failed) = use_play_style_summary(Screen::Home);

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
                        Podium { identities: summary.identities }
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
                    // Mirror the loaded panel: the title bar, the podium's
                    // three steps, then the button.
                    div { class: "skeleton-card-header", style: "display: flex; justify-content: center;",
                        div { class: "skeleton-bar skeleton-section-label", style: "width: 45%; margin-bottom: 0;" }
                    }
                    div { class: "podium",
                        for class in ["podium-block podium-second", "podium-block podium-first", "podium-block podium-third"] {
                            div { key: "{class}", class: "podium-step",
                                div { class: "skeleton-bar {class}" }
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
    use Color::{Black, Red, White};

    use super::*;

    fn identity(colors: &[&str], decks: u32) -> HttpColorIdentityCount {
        HttpColorIdentityCount {
            colors: colors.iter().map(|c| c.to_string()).collect(),
            decks,
        }
    }

    #[test]
    fn podium_stands_second_first_third() {
        let steps = podium(&[
            identity(&["R", "W"], 6),
            identity(&["U"], 1),
            identity(&["B"], 1),
        ]);
        let places: Vec<usize> = steps.iter().map(|s| s.place).collect();
        assert_eq!(places, vec![2, 1, 3]);
        assert_eq!(steps.get(1).map(|s| s.pips.clone()), Some(vec![White, Red]));
        assert_eq!(steps.get(1).map(|s| s.decks), Some(6));
    }

    #[test]
    fn podium_short_of_three_keeps_the_order() {
        let two = podium(&[identity(&["R"], 2), identity(&["B"], 1)]);
        assert_eq!(two.iter().map(|s| s.place).collect::<Vec<_>>(), vec![2, 1]);
        let one = podium(&[identity(&["X", "B"], 1)]);
        assert_eq!(one.first().map(|s| s.pips.clone()), Some(vec![Black]));
        assert!(podium(&[]).is_empty());
    }

    #[test]
    fn place_labels_and_heights_follow_the_place() {
        let steps = podium(&[
            identity(&["W"], 3),
            identity(&["U"], 2),
            identity(&["B"], 1),
        ]);
        let labels: Vec<&str> = steps.iter().map(PodiumStep::place_label).collect();
        assert_eq!(labels, vec!["2nd", "1st", "3rd"]);
        assert_eq!(
            steps.get(1).map(PodiumStep::class),
            Some("podium-block podium-first")
        );
    }
}
