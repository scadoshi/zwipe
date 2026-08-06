use crate::{
    Footer, Nav, Route,
    components::{FeaturedFlavor, PageMeta, StatsStrip, dismiss_flavor_overlay},
};
use dioxus::prelude::*;
use zwipe_components::{Banner, BannerStatus, FlippableCardImage, Panel};
use zwipe_core::domain::card::scryfall_data::{ImageSize, ScryfallData};

const LOGO_ASCII: &str = zwipe_core::domain::logo::ZWIPE;

/// App Store listing: canonical download + review source.
const APP_STORE_URL: &str = "https://apps.apple.com/us/app/zwipe-tcg/id6761341603";

/// JSON-LD `MobileApplication` markup emitted into `<head>` on the home page.
/// Drives rich app results in search: name, platforms, free price, and the
/// live App Store rating (4.8 / 4 ratings as of 2026-06-30; bump when it moves).
const JSON_LD: &str = r#"{
  "@context": "https://schema.org",
  "@type": "MobileApplication",
  "name": "Zwipe",
  "operatingSystem": "iOS, Android",
  "applicationCategory": "GameApplication",
  "url": "https://zwipe.net",
  "downloadUrl": "https://apps.apple.com/us/app/zwipe-tcg/id6761341603",
  "description": "A Magic: The Gathering deck builder built for mobile. Swipe to build Commander decks with synergy-ranked cards.",
  "offers": { "@type": "Offer", "price": "0", "priceCurrency": "USD" },
  "aggregateRating": { "@type": "AggregateRating", "ratingValue": "4.8", "ratingCount": "4" }
}"#;

#[component]
fn HomeJsonLd() -> Element {
    rsx! {
        document::Script { r#type: "application/ld+json", "{JSON_LD}" }
    }
}

/// Public App Store reviews surfaced as social proof. Five-star reviews only;
/// the four-star "set land amount" review is intentionally omitted (its request
/// has since shipped). Quotes are lightly cleaned of transcription typos.
#[component]
fn Testimonials() -> Element {
    let reviews: Vec<(&str, &str)> = vec![
        (
            "Why!? Why has there not been a utility to filter cards for decks via relevant flavors/type/effects. This app does it.",
            "Trailmix98",
        ),
        (
            "Really have struggled in the past with deck building apps on mobile but this one definitely takes the cake as best. Super easy to concept out new deck ideas without a ton of research and planning! For sure my favorite deck building tool.",
            "Arctic creature",
        ),
    ];
    rsx! {
        section { class: "testimonials",
            // One Reviews Panel holding everything: the live App Store rating
            // as a clickable share-screen-style tag, then the review cards
            // nested inside its body.
            Panel { eyebrow: "Reviews", title: "Deck builder testimonials",
                a {
                    class: "rating-tag",
                    href: APP_STORE_URL,
                    target: "_blank",
                    rel: "noopener noreferrer",
                    "aria-label": "4.8 out of 5 stars on the App Store",
                    span { class: "rating-stars", "★★★★★" }
                    span { class: "rating-score", "4.8" }
                }
                div { class: "testimonials-grid",
                    for (quote, handle) in reviews {
                        figure { class: "testimonial",
                            blockquote { class: "testimonial-quote", "“{quote}”" }
                            figcaption { class: "testimonial-author", "{handle}" }
                        }
                    }
                }
            }
        }
    }
}

const DEMO_CREATE_DECK: Asset = asset!("/assets/demo/1_create_deck.mp4");
const DEMO_DECK_CARDS_VIEW: Asset = asset!("/assets/demo/2_deck_cards.mp4");
const DEMO_ADD_DECK_CARDS: Asset = asset!("/assets/demo/3_add_deck_cards.mp4");
const DEMO_FILTER: Asset = asset!("/assets/demo/4_filter.mp4");
const DEMO_PROFILE: Asset = asset!("/assets/demo/5_profile.mp4");
const DEMO_IMPORT_DECK_STATS: Asset = asset!("/assets/demo/6_import_deck_stats.mp4");

#[component]
pub fn Home() -> Element {
    let demos: Vec<(Asset, &'static str, &'static str)> = vec![
        (
            DEMO_CREATE_DECK,
            "Create a deck and pick your commander",
            "Demo: Create a Deck",
        ),
        (
            DEMO_IMPORT_DECK_STATS,
            "Import a decklist from a link, then check your deck's stats",
            "Demo: Import & Stats",
        ),
        (
            DEMO_DECK_CARDS_VIEW,
            "Browse cards: keywords, full details, art, and printings",
            "Demo: Deck Card View",
        ),
        (
            DEMO_ADD_DECK_CARDS,
            "Swipe cards out of your deck, then swipe new ones in",
            "Demo: Swipe to Build",
        ),
        (
            DEMO_FILTER,
            "Filter the card pool, then swipe to add",
            "Demo: Filters",
        ),
        (
            DEMO_PROFILE,
            "Switch themes from your profile",
            "Demo: Themes & Profile",
        ),
    ];
    let total = demos.len();
    let mut index = use_signal(|| 0usize);
    #[allow(clippy::indexing_slicing)]
    let (current_src, current_caption, current_label) = demos[index()];

    // Featured flavor's full-art overlay state. The overlay is rendered HERE,
    // at the page's top level outside the `content-enter` tree — mirroring
    // the shared-deck page, whose overlay works. Nested inside the content
    // tree, an animated ancestor's transform becomes the containing block for
    // `position: fixed` and traps the overlay in the page column.
    let flavor_overlay: Signal<Option<ScryfallData>> = use_signal(|| None);
    let flavor_overlay_dismissing = use_signal(|| false);

    rsx! {
        PageMeta {
            title: "Mobile Magic: The Gathering & Commander Deck Builder",
            description: "Zwipe is a Magic: The Gathering deck builder built for mobile. Swipe right to add, left to skip, up to maybe, down to undo. Swipe-pick your commander, tag decks by archetype, synergy-ranked cards, Commander-ready, decks synced across sessions.",
            path: "/",
        }
        HomeJsonLd {}
        Nav {}
        div { class: "banner-stack",
            Banner {
                category: "Release",
                status: BannerStatus::Done,
                status_label: "New",
                "Version 1.7.5 just shipped. "
                Link { to: Route::Changelog {}, "See what's new" }
            }
        }
        div { class: "hero",
            // Semantic page heading for crawlers and screen readers; the ASCII
            // logo is the visual title, so this is visually hidden.
            h1 { class: "sr-only", "Zwipe, the Magic: The Gathering deck builder built for mobile" }
            div { class: "logo", "{LOGO_ASCII}" }
            div { class: "hero-card",
                p { class: "tagline",
                    "The "
                    a { href: "https://magic.wizards.com/en", target: "_blank", rel: "noopener noreferrer", "Magic: The Gathering" }
                    " deck builder built for mobile. Swipe "
                    span { class: "swipe-add", "right" }
                    " to add card to deck (or remove on remove flow), "
                    span { class: "swipe-skip", "left" }
                    " to skip card, "
                    span { class: "swipe-maybe", "up" }
                    " to add to maybeboard, "
                    span { class: "swipe-undo", "down" }
                    " to undo."
                }
                div { class: "hero-chips",
                    span { class: "hero-chip chip-value", "Free" }
                    span { class: "hero-chip chip-value", "No ads" }
                    span { class: "hero-chip chip-plat", "iOS" }
                    span { class: "hero-chip chip-plat", "Android" }
                }
                StatsStrip {}
            }
        }
        // Tap-to-open full-art overlay for the featured flavor card, copied
        // from the shared-deck page: a dimmed backdrop with the card art, tap
        // anywhere to dismiss (the flip button stops propagation, so flipping
        // never dismisses). Lives at the top level, outside `content-enter`.
        if flavor_overlay().is_some() || flavor_overlay_dismissing() {
            div { class: "sd-image-overlay-backdrop" }
            div {
                class: if flavor_overlay_dismissing() { "sd-image-overlay dismissing" } else { "sd-image-overlay" },
                onclick: move |_| {
                    dismiss_flavor_overlay(flavor_overlay, flavor_overlay_dismissing);
                },
                if let Some(sd) = flavor_overlay() {
                    FlippableCardImage {
                        sd,
                        size: ImageSize::Large,
                        class: "sd-image-overlay-img".to_string(),
                        draggable: false,
                    }
                }
            }
        }
        div { class: "page content-enter",
            // The demo and the three core sells share one band: the gallery
            // is ~70vh tall, and the stacked panels fill what used to be its
            // dead side-space instead of renting another screen below it.
            div { class: "demo-features",
                // Left column: the demo gallery with the hour's flavor card
                // tucked beneath it.
                div { class: "demo-col",
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
                    FeaturedFlavor { overlay: flavor_overlay }
                }
                // The three core sells — swiping, synergy-ordered serving,
                // and tags. Hosting basics (accounts, sync, import) are
                // assumed service table stakes, not pitched.
                div { class: "features-stack",
                    Panel { eyebrow: "Build", title: "Swipe to Build",
                        ul { class: "card-bullets",
                            li { "Right to add card to deck (or remove in remove flow)" }
                            li { "Left to skip card" }
                            li { "Up to add to maybeboard" }
                            li { "Down to undo last swipe" }
                            li { "Swipe-pick your commander, partner, background, or signature spell" }
                        }
                    }
                    Panel { eyebrow: "Synergy", title: "Served in Synergy Order",
                        ul { class: "card-bullets",
                            li { "Most synergistic cards show first based on your selected commander" }
                            li { "The order learns from swipes: crowd favorites rise as players build" }
                            li { "Color identity and per-format eligibility validated" }
                        }
                    }
                    Panel { eyebrow: "Tags", title: "Know What Every Card Does",
                        ul { class: "card-bullets",
                            li { "Every card labeled by role: ramp, removal, counterspell, tokens, and more" }
                            li { "Filter your feed by what a card does, not just its text" }
                            li { "Tap any tag for a plain-language definition" }
                            li { "Community-maintained, so the labels stay current" }
                        }
                    }
                }
            }
            Testimonials {}
        }
        Footer {}
    }
}
