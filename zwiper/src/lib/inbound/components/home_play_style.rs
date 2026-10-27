//! Home's Playstyle panel: the user's top colors, with a button that opens
//! the full screen.

use components::Chip;
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::scryfall_data::colors::{Color, identity_name};
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
    /// "1st", "2nd" or "3rd", for the block's accessible label.
    pub(crate) fn place_label(&self) -> &'static str {
        match self.place {
            1 => "1st",
            2 => "2nd",
            _ => "3rd",
        }
    }

    /// The step's class: a fixed height, first tallest, so the podium reads
    /// as one however lopsided the counts are, and a palette accent per
    /// place (status colors would read as an outcome).
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
/// each step a block in its place's accent with the deck count inside and
/// the pips and a name tag under it. Renders nothing without identities.
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
                    // The block, count inside, in a fixed-height stand, so
                    // every block sits on the same floor.
                    div { class: "podium-stand",
                        div {
                            class: step.class(),
                            aria_label: "{step.place_label()}, {step.decks} decks",
                            "{step.decks}"
                        }
                    }
                    IdentityPips { identity: step.pips.clone(), show_name: false }
                    // Every step is named, Mono-red included, so the three
                    // read alike.
                    span { class: "stat-chip", "{identity_name(&step.pips)}" }
                }
            }
        }
    }
}

/// The panel under the hero, a sibling of the featured flavor card: the
/// title bar with an Open chip, then the top three color identities as a
/// podium. Before any deck has cards it shows one line instead; a failed
/// fetch with nothing cached shows a muted line. The Open chip is always
/// there, so the screen stays reachable from Home.
#[component]
pub fn HomePlayStyle() -> Element {
    let navigator = use_navigator();
    let (summary, failed) = use_play_style_summary(Screen::Home);

    rsx! {
        div { class: "container-sm text-center flex-col home-flavor content-enter-delayed",
            match summary {
                Some(summary) => rsx! {
                    PanelHeader { onclick: move |_| { navigator.push(Router::SuggestedCommanders {}); } }
                    if summary.deck_count == 0 {
                        div { class: "message-empty", style: "padding: 1rem;",
                            "Build a deck to see your playstyle"
                        }
                    } else {
                        Podium { identities: summary.identities }
                    }
                },
                None if failed() => rsx! {
                    PanelHeader { onclick: move |_| { navigator.push(Router::SuggestedCommanders {}); } }
                    div { class: "message-empty", style: "padding: 1rem;",
                        "Could not load your playstyle"
                    }
                },
                None => rsx! {
                    // Mirror the loaded panel: the title bar, then the
                    // podium's three steps.
                    div { class: "skeleton-card-header", style: "display: flex; justify-content: center;",
                        div { class: "skeleton-bar skeleton-section-label", style: "width: 45%; margin-bottom: 0;" }
                    }
                    div { class: "podium",
                        for class in ["podium-second", "podium-first", "podium-third"] {
                            div { key: "{class}", class: "podium-step",
                                div { class: "podium-stand",
                                    div { class: "skeleton-bar podium-skeleton {class}" }
                                }
                            }
                        }
                    }
                },
            }
        }
    }
}

/// The panel's title bar: Playstyle centered, Open as a chip at its right
/// end, the way the deck view's Budget header carries its currency chips.
#[component]
fn PanelHeader(onclick: EventHandler<MouseEvent>) -> Element {
    rsx! {
        div { class: "settings-list-header home-panel-header",
            span { class: "settings-list-title", "Playstyle" }
            span { class: "home-panel-open",
                Chip { selected: false, onclick: move |e| onclick.call(e), "Open" }
            }
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
