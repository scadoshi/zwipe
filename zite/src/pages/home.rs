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

/// Public App Store reviews surfaced as social proof, every rating included.
/// Reviews that asked for a feature the app has since built carry a
/// "Shipped in x.y.z" tag — the receipt that feedback lands. Quotes are
/// lightly cleaned of transcription typos; the truncated land-amount review
/// ends at its last full clause.
///
/// The track auto-scrolls (slow marquee) so every review gets seen without a
/// click: the review set renders twice, the second copy aria-hidden, and CSS
/// slides the track by exactly one set width for a seamless loop. Hover
/// pauses it; reduced-motion gets a static wrapped grid instead.
#[component]
fn Testimonials() -> Element {
    let reviews: Vec<(&str, &str, Option<&str>)> = vec![
        (
            "This one defo has some potential. I like that the developer for it (seems like just one guy?) is super welcoming to feedback and adding features. I messaged and asked for budgeting/land count tracking and he added it a day later. And it works perfectly. That's mega.",
            "Spice mayonnaise",
            None,
        ),
        (
            "App seems super cool and a refreshing take on deck building. Main complaint so far is I wish there was a way to set a desired amount of lands before hand, say 40, and when you hit 60 no land cards in the deck it would have a pop up warning that says you only have space for x amount more lands.",
            "Caed_",
            Some("Shipped in 1.1.4"),
        ),
        (
            "Really have struggled in the past with deck building apps on mobile but this one definitely takes the cake as best. Super easy to concept out new deck ideas without a ton of research and planning! For sure my favorite deck building tool.",
            "Arctic creature",
            None,
        ),
        (
            "I love MTG and this is a great app to deck build or just pass the time. Hoping that new features include saving what cards you've been through between sessions.",
            "Mr.K to you",
            Some("Shipped in 1.3.0"),
        ),
        (
            "Why!? Why has there not been a utility to filter cards for decks via relevant flavors/type/effects. This app does it.",
            "Trailmix98",
            None,
        ),
        ("Great app to quickly build a nice deck.", "Audco02", None),
    ];
    rsx! {
        section { class: "testimonials",
            // One Reviews Panel holding everything: the live App Store rating
            // as a clickable share-screen-style tag, then the auto-scrolling
            // review track nested inside its body.
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
                div { class: "testimonials-viewport",
                    div { class: "testimonials-track",
                        for dup in [false, true] {
                            div {
                                class: if dup { "testimonials-set testimonials-set-dup" } else { "testimonials-set" },
                                "aria-hidden": "{dup}",
                                for (quote , handle , shipped) in reviews.clone() {
                                    figure { class: "testimonial",
                                        blockquote { class: "testimonial-quote", "“{quote}”" }
                                        figcaption { class: "testimonial-author",
                                            "{handle}"
                                            if let Some(tag) = shipped {
                                                span { class: "shipped-tag", "{tag}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

const DEMO_CREATE_DECK: Asset = asset!("/assets/demo/1_create_deck.mp4");
const DEMO_IMPORT: Asset = asset!("/assets/demo/2_import.mp4");
const DEMO_ADD_CARDS: Asset = asset!("/assets/demo/3_add_cards.mp4");
const DEMO_REMOVE_CARDS: Asset = asset!("/assets/demo/4_remove_cards.mp4");
const DEMO_FILTER: Asset = asset!("/assets/demo/5_filter.mp4");
const DEMO_CARD_DETAILS: Asset = asset!("/assets/demo/6_card_details.mp4");
const DEMO_ORACLE_TAGS: Asset = asset!("/assets/demo/7_oracle_tags.mp4");
const DEMO_DECK_CARDS_VIEW: Asset = asset!("/assets/demo/8_deck_cards.mp4");
const DEMO_MVPS: Asset = asset!("/assets/demo/9_mvps.mp4");
const DEMO_DECK_STATS: Asset = asset!("/assets/demo/10_deck_stats.mp4");
const DEMO_SHARE_DECK: Asset = asset!("/assets/demo/11_share_deck.mp4");
const DEMO_PROFILE: Asset = asset!("/assets/demo/12_profile.mp4");

#[component]
pub fn Home() -> Element {
    // Ordered as the build flow: start a deck, fill it, refine it, read it.
    let demos: Vec<(Asset, &'static str, &'static str)> = vec![
        (
            DEMO_CREATE_DECK,
            "Create a deck and pick your commander",
            "Demo: Create a deck",
        ),
        (
            DEMO_ADD_CARDS,
            "Swipe new cards into your deck",
            "Demo: Swipe to build",
        ),
        (
            DEMO_FILTER,
            "Filter the card pool, then swipe to add",
            "Demo: Filters",
        ),
        (
            DEMO_CARD_DETAILS,
            "Open full card details while you swipe",
            "Demo: Card details",
        ),
        (
            DEMO_ORACLE_TAGS,
            "Browse the tag dictionary and filter by oracle tag",
            "Demo: Oracle tags",
        ),
        (
            DEMO_REMOVE_CARDS,
            "Swipe cards back out of your deck",
            "Demo: Swipe to remove",
        ),
        (
            DEMO_DECK_CARDS_VIEW,
            "Browse your deck: art, printings, and groupings",
            "Demo: Deck card view",
        ),
        (
            DEMO_MVPS,
            "Star MVPs so your key cards lead the deck",
            "Demo: Deck MVPs",
        ),
        (DEMO_IMPORT, "Import a decklist from a link", "Demo: Import"),
        (
            DEMO_DECK_STATS,
            "Check your deck's stats, curve, and draw odds",
            "Demo: Deck stats",
        ),
        (
            DEMO_SHARE_DECK,
            "Share any deck with a public link",
            "Demo: Share your deck",
        ),
        (
            DEMO_PROFILE,
            "Switch themes and catch up on the changelog",
            "Demo: Themes and profile",
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
            title: "Magic: The Gathering Deck Builder for Mobile",
            description: "Magic: The Gathering deck builder for mobile. Swipe right to add, left to skip. Commander-ready, with synergy-ranked cards.",
            path: "/",
        }
        HomeJsonLd {}
        Nav {}
        div { class: "banner-stack",
            Banner {
                category: "Release",
                status: BannerStatus::Done,
                status_label: "New",
                "Version 1.9.1 just shipped. "
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
                    // Same Panel grammar as the rest of the band; the per-clip
                    // caption under the video does the describing.
                    Panel { eyebrow: "Demo", title: "Watch it work",
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
                            span { key: "{index()}", class: "gallery-caption", "{current_caption}" }
                            if total > 1 {
                                span { class: "gallery-counter", "{index() + 1} / {total}" }
                            }
                        }
                    }
                    FeaturedFlavor { overlay: flavor_overlay }
                }
                // The three core sells — swiping, synergy-ordered serving,
                // and tags. Hosting basics (accounts, sync, import) are
                // assumed service table stakes, not pitched.
                div { class: "features-stack",
                    Panel { eyebrow: "Build", title: "Swipe to build",
                        ul { class: "card-bullets",
                            li { "Right to add card to deck (or remove in remove flow)" }
                            li { "Left to skip card" }
                            li { "Up to add to maybeboard" }
                            li { "Down to undo last swipe" }
                            li { "Swipe-pick your commander, partner, background, or signature spell" }
                        }
                    }
                    Panel { eyebrow: "Synergy", title: "Served in synergy order",
                        ul { class: "card-bullets",
                            li { "Most synergistic cards show first based on your selected commander" }
                            li { "The order learns from swipes: crowd favorites rise as players build" }
                            li { "Color identity and per-format eligibility validated" }
                        }
                    }
                    Panel { eyebrow: "Tags", title: "Know what every card does",
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
