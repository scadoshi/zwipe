use dioxus::prelude::*;
use crate::{Footer, Nav};

const LOGO_ASCII: &str = zwipe_core::domain::logo::ZWIPE;

const DEMO_ADD_CARDS: Asset = asset!("/assets/projects/zwipe/zwipe-demo-add-cards.mp4");
const DEMO_DECK_VIEW: Asset = asset!("/assets/projects/zwipe/zwipe-demo-deck-view.mp4");
const DEMO_DECK_PROFILE: Asset = asset!("/assets/projects/zwipe/zwipe-demo-deck-profile.mp4");
const DEMO_IMPORT_DECK: Asset = asset!("/assets/projects/zwipe/zwipe-demo-import-deck.mp4");
const DEMO_PROFILE: Asset = asset!("/assets/projects/zwipe/zwipe-demo-profile.mp4");

#[component]
pub fn Home() -> Element {
    let demos: Vec<(Asset, &'static str, &'static str)> = vec![
        (DEMO_ADD_CARDS, "Swiping cards to add to a deck", "Demo: Swipe to Build"),
        (DEMO_DECK_VIEW, "Browsing the deck card list", "Demo: Deck Card View"),
        (DEMO_DECK_PROFILE, "Editing the deck profile and commander", "Demo: Deck Profile"),
        (DEMO_IMPORT_DECK, "Importing a decklist from text", "Demo: Import Deck"),
        (DEMO_PROFILE, "Account profile screen", "Demo: Account Profile"),
    ];
    let total = demos.len();
    let mut index = use_signal(|| 0usize);
    let (current_src, current_caption, current_label) = demos[index()];

    rsx! {
        Nav {}
        div { class: "hero",
            div { class: "logo", "{LOGO_ASCII}" }
            p { class: "tagline",
                "The "
                a { href: "https://magic.wizards.com/en", target: "_blank", rel: "noopener noreferrer", "Magic: The Gathering" }
                " deck builder built for mobile. Swipe right to add, left to skip, up to maybe, down to undo."
            }
        }
        div { class: "page content-enter",
            figure { class: "project-gallery",
                div { class: "gallery-frame",
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
                            "‹"
                        }
                        button {
                            class: "gallery-nav gallery-next",
                            aria_label: "Next demo",
                            onclick: move |_| {
                                let i = index();
                                index.set((i + 1) % total);
                            },
                            "›"
                        }
                    }
                }
                div { class: "gallery-meta",
                    figcaption { class: "gallery-caption", "{current_caption}" }
                    if total > 1 {
                        span { class: "gallery-counter", "{index() + 1} / {total}" }
                    }
                }
            }
            div { class: "features-grid",
                div { class: "feature-card",
                    span { class: "card-category", "Build" }
                    h3 { class: "card-title", "Swipe to Build" }
                    p { class: "card-summary", "Browse cards one at a time. Gestures map to actions." }
                    ul { class: "card-bullets",
                        li { "Right to add to deck" }
                        li { "Left to skip" }
                        li { "Up to maybeboard" }
                        li { "Down to undo" }
                    }
                }
                div { class: "feature-card",
                    span { class: "card-category", "Search" }
                    h3 { class: "card-title", "Deep Filters" }
                    p { class: "card-summary", "Narrow 110k+ cards down to exactly what fits." }
                    ul { class: "card-bullets",
                        li { "Color, type, mana cost, oracle text" }
                        li { "Keywords, artist, set, rarity" }
                        li { "Commander eligibility, format legality" }
                        li { "Per-section clear for fine-tuned control" }
                    }
                }
                div { class: "feature-card",
                    span { class: "card-category", "Catalog" }
                    h3 { class: "card-title", "110k+ Cards" }
                    p { class: "card-summary", "Every English printing from Scryfall, synced nightly. Multiple printings per card — pick your favorite art." }
                }
                div { class: "feature-card",
                    span { class: "card-category", "Format" }
                    h3 { class: "card-title", "Commander Ready" }
                    p { class: "card-summary", "Full Commander format support across the command zone." }
                    ul { class: "card-bullets",
                        li { "Partners, backgrounds, Oathbreaker + signature spell" }
                        li { "Eligibility filtering per format" }
                        li { "Color identity validation" }
                    }
                }
                div { class: "feature-card",
                    span { class: "card-category", "Data I/O" }
                    h3 { class: "card-title", "Import / Export" }
                    p { class: "card-summary", "Paste any decklist from Moxfield or Archidekt. Maybeboard sections import and export automatically." }
                }
                div { class: "feature-card",
                    span { class: "card-category", "Account" }
                    h3 { class: "card-title", "Your Decks, Synced" }
                    p { class: "card-summary", "Account-based. Your decks are always there across sessions." }
                }
                div { class: "feature-card",
                    span { class: "card-category", "Workflow" }
                    h3 { class: "card-title", "Maybeboard" }
                    p { class: "card-summary", "Stage cards you're considering without committing them to the deck." }
                    ul { class: "card-bullets",
                        li { "Swipe up to maybe" }
                        li { "Move to deck when ready" }
                        li { "Imports + exports alongside the deck" }
                    }
                }
                div { class: "feature-card",
                    span { class: "card-category", "Art" }
                    h3 { class: "card-title", "Multiple Printings" }
                    p { class: "card-summary", "Pick your favorite art for every card in the deck." }
                    ul { class: "card-bullets",
                        li { "Swipe to browse every printing" }
                        li { "Saved per deck slot, not per oracle" }
                        li { "Works for commanders and the command zone" }
                    }
                }
                div { class: "feature-card",
                    span { class: "card-category", "Theming" }
                    h3 { class: "card-title", "14 Themes" }
                    p { class: "card-summary", "Dark and light modes across 14 color themes — including 3 colorblind-accessible options. Try the themes above." }
                }
            }
        }
        Footer {}
    }
}
