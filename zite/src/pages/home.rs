use crate::components::{PageMeta, StatsStrip};
use crate::{Footer, Nav};
use dioxus::prelude::*;

const LOGO_ASCII: &str = zwipe_core::domain::logo::ZWIPE;

const DEMO_CREATE_DECK: Asset = asset!("/assets/new-demo/1_create_deck.mp4");
const DEMO_ADD_DECK_CARDS: Asset = asset!("/assets/new-demo/2_add_deck_cards.mp4");
const DEMO_DECK_CARDS_VIEW: Asset = asset!("/assets/new-demo/3_deck_cards_view.mp4");
const DEMO_REGISTER: Asset = asset!("/assets/new-demo/4_register.mp4");
const DEMO_PROFILE: Asset = asset!("/assets/new-demo/5_profile.mp4");

#[component]
pub fn Home() -> Element {
    let demos: Vec<(Asset, &'static str, &'static str)> = vec![
        (
            DEMO_CREATE_DECK,
            "Creating a new deck",
            "Demo: Create a Deck",
        ),
        (
            DEMO_ADD_DECK_CARDS,
            "Swiping cards to add to a deck",
            "Demo: Swipe to Build",
        ),
        (
            DEMO_DECK_CARDS_VIEW,
            "Browsing the deck card list",
            "Demo: Deck Card View",
        ),
        (DEMO_REGISTER, "Creating a Zwipe account", "Demo: Register"),
        (DEMO_PROFILE, "User profile screen", "Demo: Account Profile"),
    ];
    let total = demos.len();
    let mut index = use_signal(|| 0usize);
    #[allow(clippy::indexing_slicing)]
    let (current_src, current_caption, current_label) = demos[index()];

    rsx! {
        PageMeta {
            title: "Zwipe",
            description: "Zwipe is a Magic: The Gathering deck builder built for mobile. Swipe right to add, left to skip, up to maybe, down to undo. Synergy-ranked cards, Commander-ready, decks synced across sessions.",
            path: "/",
        }
        Nav {}
        div { class: "hero",
            div { class: "logo", "{LOGO_ASCII}" }
            p { class: "tagline",
                "The "
                a { href: "https://magic.wizards.com/en", target: "_blank", rel: "noopener noreferrer", "Magic: The Gathering" }
                " deck builder built for mobile. Swipe right to add card to deck (or remove on remove flow), left to skip card, up to add to maybeboard, down to undo."
            }
            StatsStrip {}
        }
        div { class: "page content-enter",
            figure { class: "project-gallery",
                div { class: "gallery-frame",
                    div { class: "gallery-header", "Demo" }
                    hr { class: "gallery-rule" }
                    div { class: "gallery-body",
                        video {
                            // key forces a remount when index changes so autoplay re-fires
                            // for the new src instead of the browser keeping the old video.
                            key: "{index()}",
                            class: "gallery-video",
                            src: current_src,
                            "aria-label": "{current_label}",
                            autoplay: true,
                            muted: true,
                            "loop": true,
                            playsinline: true,
                            controls: true,
                            preload: "metadata",
                        }
                        if total > 1 {
                            button {
                                class: "gallery-nav gallery-prev",
                                aria_label: "Previous demo",
                                onclick: move |_| {
                                    let i = index();
                                    index.set(if i == 0 { total - 1 } else { i - 1 });
                                },
                                "←"
                            }
                            button {
                                class: "gallery-nav gallery-next",
                                aria_label: "Next demo",
                                onclick: move |_| {
                                    let i = index();
                                    index.set((i + 1) % total);
                                },
                                "→"
                            }
                        }
                    }
                    hr { class: "gallery-rule" }
                    div { class: "gallery-footer",
                        figcaption { key: "{index()}", class: "gallery-caption", "{current_caption}" }
                        if total > 1 {
                            span { class: "gallery-counter", "{index() + 1} / {total}" }
                        }
                    }
                }
            }
            div { class: "features-grid",
                div { class: "feature-card",
                    span { class: "card-category", "Build" }
                    h3 { class: "card-title", "Swipe to Build" }
                    ul { class: "card-bullets",
                        li { "Right to add card to deck (or remove in remove flow)" }
                        li { "Left to skip card" }
                        li { "Up to add to maybeboard" }
                        li { "Down to undo last swipe" }
                    }
                }
                div { class: "feature-card",
                    span { class: "card-category", "Format" }
                    h3 { class: "card-title", "Commander Ready" }
                    ul { class: "card-bullets",
                        li { "Most synergistic cards show first based on your selected commander" }
                        li { "Partners, backgrounds, Oathbreaker, and other Commander-like formats" }
                        li { "Color identity validation" }
                        li { "Per-format eligibility" }
                    }
                }
                div { class: "feature-card",
                    span { class: "card-category", "Search" }
                    h3 { class: "card-title", "Deep Filters" }
                    ul { class: "card-bullets",
                        li { "Filter on any attribute you'd want" }
                        li { "Stack and clear filters freely" }
                    }
                }
                div { class: "feature-card",
                    span { class: "card-category", "Decks" }
                    h3 { class: "card-title", "Your Decks, Synced" }
                    ul { class: "card-bullets",
                        li { "Import from an Archidekt URL or paste any decklist" }
                        li { "Real accounts, no setup friction" }
                        li { "Synced across sessions wherever you sign in" }
                    }
                }
            }
        }
        Footer {}
    }
}
