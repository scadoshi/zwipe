use components::{
    Banner, Decode, FlippableCardImage, GalleryFooter, GalleryFrame, Panel, Replay, Splash,
    SplashGate,
};
use dioxus::prelude::*;
use zwipe_core::domain::card::scryfall_data::{ImageSize, ScryfallData};

use crate::components::{FeaturedFlavor, PageMeta, StatsStrip, dismiss_flavor_overlay, sleep_ms};
use crate::{Footer, Nav, Route, SiteSplash};

const LOGO_ASCII: &str = zwipe_core::domain::logo::ZWIPE;

/// The splash's fade out, the `.splash` opacity transition in `assets/style.css`.
const SPLASH_FADE_MS: u32 = 300;

/// Newest shipped version, for the release banner. Reads `RELEASES`, never
/// `UPCOMING`: an unreleased version must not be announced as shipped. Falls
/// back to the crate version if the list is ever empty.
fn latest_release() -> String {
    zwipe_core::content::changelog::RELEASES
        .first()
        .map_or_else(
            || env!("CARGO_PKG_VERSION").to_owned(),
            |release| release.version.to_string(),
        )
}

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
  "description": "A Magic: The Gathering deck builder for phone and web. Swipe to build Commander decks with synergy-ranked cards.",
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
/// "Shipped in x.y.z" tag, the receipt that feedback lands. Quotes are
/// lightly cleaned of transcription typos; the truncated land-amount review
/// ends at its last full clause.
///
/// The track auto-scrolls (slow marquee) so every review gets seen without a
/// click: the review set renders twice, the second copy aria-hidden, and CSS
/// slides the track by exactly one set width so the loop has no visible seam. Hover
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
            // One Reviews Panel: the App Store rating as a clickable tag beside
            // the title, then the auto-scrolling review track in its body.
            Panel {
                title: "Deck builder testimonials",
                title_trailing: rsx! {
                    a {
                        class: "rating-tag",
                        href: APP_STORE_URL,
                        "aria-label": "4.8 out of 5 stars on the App Store",
                        span { class: "rating-stars", "★★★★★" }
                        span { class: "rating-score", "4.8" }
                    }
                },
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

const DEMO_SWIPE: Asset = asset!("/assets/demo/1_swipe.mp4");
const DEMO_SWIPE_POSTER: Asset = asset!("/assets/demo/1_swipe.jpg");
const DEMO_STATS: Asset = asset!("/assets/demo/2_stats.mp4");
const DEMO_STATS_POSTER: Asset = asset!("/assets/demo/2_stats.jpg");
const DEMO_PLAYSTYLE: Asset = asset!("/assets/demo/3_playstyle.mp4");
const DEMO_PLAYSTYLE_POSTER: Asset = asset!("/assets/demo/3_playstyle.jpg");
const DEMO_ACTIVITY: Asset = asset!("/assets/demo/4_activity.mp4");
const DEMO_ACTIVITY_POSTER: Asset = asset!("/assets/demo/4_activity.jpg");

/// One demo clip: the video, its first frame as the poster, the caption
/// under it and its label for screen readers.
#[derive(Clone, Copy)]
struct Demo {
    video: Asset,
    poster: Asset,
    caption: &'static str,
    label: &'static str,
}

/// The demo clips, in the order they play: build a deck, read it, then what
/// the app learns about how you play.
const DEMOS: [Demo; 4] = [
    Demo {
        video: DEMO_SWIPE,
        poster: DEMO_SWIPE_POSTER,
        caption: "Swipe right to add a card, left to skip it",
        label: "Demo: Swipe to build",
    },
    Demo {
        video: DEMO_STATS,
        poster: DEMO_STATS_POSTER,
        caption: "Price and mana curve for the deck you're building",
        label: "Demo: Deck stats",
    },
    Demo {
        video: DEMO_PLAYSTYLE,
        poster: DEMO_PLAYSTYLE_POSTER,
        caption: "The colors and Oracle tags you play most, and commanders that fit them",
        label: "Demo: Playstyle",
    },
    Demo {
        video: DEMO_ACTIVITY,
        poster: DEMO_ACTIVITY_POSTER,
        caption: "Your swipes and searches over the year, and the cards you skip most",
        label: "Demo: Activity",
    },
];

#[component]
pub fn Home() -> Element {
    let latest = latest_release();
    let total = DEMOS.len();
    let index = use_signal(|| 0usize);
    #[expect(
        clippy::indexing_slicing,
        reason = "orangui's GalleryFrame keeps index below total, the length of DEMOS"
    )]
    let current = DEMOS[index()];

    // Featured flavor's full-art overlay state. The overlay is rendered HERE,
    // at the page's top level outside the `content-enter` tree, mirroring
    // the shared-deck page, whose overlay works. Nested inside the content
    // tree, an animated ancestor's transform becomes the containing block for
    // `position: fixed` and traps the overlay in the page column.
    let flavor_overlay: Signal<Option<ScryfallData>> = use_signal(|| None);
    let flavor_overlay_dismissing = use_signal(|| false);

    // The opening. Once it has faded (the gate opens as the fade starts) it is
    // done for this load.
    let mut splash = use_context::<SiteSplash>().0;
    let gate = use_context::<SplashGate>().0;
    use_effect(move || {
        if gate() && splash() {
            spawn(async move {
                sleep_ms(SPLASH_FADE_MS).await;
                splash.set(false);
            });
        }
    });

    rsx! {
        PageMeta {
            title: "Magic: The Gathering Deck Builder for Phone and Web",
            description: "Magic: The Gathering deck builder for phone and web. Swipe right to add, left to skip. Commander-ready, with synergy-ranked cards.",
            path: "/",
        }
        HomeJsonLd {}
        // At the page's top level, outside the `content-enter` tree, so its
        // `position: fixed` covers the viewport (see the flavor overlay).
        if splash() {
            Splash { pace: 67, prerendered: true,
                div { class: "logo", "aria-hidden": "true", Decode { text: LOGO_ASCII } }
            }
        }
        Nav {}
        main { id: "content", class: "site-main",
            div { class: "banner-stack",
                Banner {
                    "{latest} is out. "
                    Link { to: Route::Changelog {}, "See what's new" }
                }
            }
            div { class: "hero",
                // Semantic page heading for crawlers and screen readers; the ASCII
                // logo is the visual title, so this is visually hidden and the
                // logo is hidden from screen readers.
                h1 { class: "sr-only", "Zwipe, the Magic: The Gathering deck builder for phone and web" }
                // Keyed on the replay count so the spacing animation runs again
                // with the decode.
                div { class: "hero-card",
                    // The mark, then the numbers it stands for, then the sentence
                    // under both.
                    div { class: "hero-head",
                        for run in [use_context::<Replay>().0()] {
                            div { key: "logo{run}", class: "logo", "aria-hidden": "true", Decode { text: LOGO_ASCII } }
                        }
                        StatsStrip {}
                    }
                    hr { class: "hero-rule" }
                    p { class: "tagline",
                        "The "
                        a { href: "https://magic.wizards.com/en", "Magic: The Gathering" }
                        " deck builder for phone and web. Swipe through cards for any format, and pick a commander to get the best synergy first."
                    }
                    div { class: "hero-chips",
                        span { class: "hero-chip chip-value", "Free" }
                        span { class: "hero-chip chip-value", "No ads" }
                        a {
                            class: "panel-action",
                            href: "https://apps.apple.com/us/app/zwipe-tcg/id6761341603",
                            "iOS"
                            span { class: "ext", "\u{2197}" }
                        }
                        a {
                            class: "panel-action",
                            href: "https://play.google.com/store/apps/details?id=com.scadoshi.zwipe",
                            "Android"
                            span { class: "ext", "\u{2197}" }
                        }
                    }
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
                // is ~70vh tall, so the stacked panels fill its side-space
                // instead of renting another screen below it.
                div { class: "demo-features",
                    div { class: "demo-col",
                        // Same Panel grammar as the rest of the band; the per-clip
                        // caption under the video does the describing.
                        Panel { title: "Watch it work",
                            GalleryFrame { index, total, noun: "demo",
                                video {
                                    // key forces a remount when index changes so autoplay re-fires
                                    // for the new src instead of the browser keeping the old video.
                                    key: "{index()}",
                                    class: "gallery-video",
                                    src: current.video,
                                    poster: current.poster,
                                    "aria-label": "{current.label}",
                                    autoplay: true,
                                    muted: true,
                                    "loop": true,
                                    playsinline: true,
                                    controls: true,
                                    preload: "metadata",
                                }
                            }
                            hr { class: "gallery-rule" }
                            GalleryFooter { index: index(), total, caption: current.caption.to_string() }
                        }
                    }
                    // The three core sells: swiping, synergy-ordered serving,
                    // and tags. Hosting basics (accounts, sync, import) are
                    // assumed service table stakes, not pitched.
                    div { class: "features-stack",
                        Panel { title: "Swipe to build",
                            ul { class: "card-bullets",
                                li { "Right to add a card" }
                                li { "Left to skip it" }
                                li { "Up to save it to your maybeboard" }
                                li { "Down to undo" }
                                li { "Removing cards works the same way, swipe by swipe" }
                                li { "Swipe-pick your commander, partner, background, or signature spell" }
                            }
                        }
                        Panel { title: "Served in synergy order",
                            ul { class: "card-bullets",
                                li { "Most synergistic cards show first based on your selected commander" }
                                li { "The order learns from swipes: crowd favorites rise as players build" }
                                li { "Only cards legal in your format and colors" }
                            }
                        }
                        Panel { title: "Know what every card does",
                            ul { class: "card-bullets",
                                li { "Every card labeled by role: ramp, removal, counterspell, tokens, and more" }
                                li { "Filter your feed by what a card does" }
                                li { "Tap any tag for a plain-language definition" }
                                li { "Built on Scryfall's community-maintained tags" }
                            }
                        }
                    }
                }
                FeaturedFlavor { overlay: flavor_overlay }
                Testimonials {}
            }
        }
        Footer {}
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::*;

    /// The most a demo clip may weigh: it autoplays on the home page.
    const MAX_CLIP_BYTES: u64 = 2 * 1024 * 1024;

    fn source(asset: Asset) -> PathBuf {
        PathBuf::from(asset.bundled().absolute_source_path())
    }

    #[test]
    fn every_demo_file_is_shown_and_every_clip_is_light() {
        let mut used: Vec<PathBuf> = Vec::new();
        for demo in DEMOS {
            let (video, poster) = (source(demo.video), source(demo.poster));
            let bytes = std::fs::metadata(&video).unwrap().len();
            assert!(
                bytes <= MAX_CLIP_BYTES,
                "{} is {bytes} bytes",
                video.display()
            );
            assert_eq!(
                video.with_extension("jpg"),
                poster,
                "a clip's poster is its first frame beside it"
            );
            used.push(video);
            used.push(poster);
        }
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/demo");
        let mut on_disk: Vec<PathBuf> = std::fs::read_dir(&dir)
            .unwrap()
            .map(|entry| entry.unwrap().path().canonicalize().unwrap())
            .filter(|path| path.file_name().is_some_and(|name| name != ".gitignore"))
            .collect();
        on_disk.sort();
        used.sort();
        assert_eq!(
            on_disk, used,
            "assets/demo holds only the clips the page shows"
        );
    }
}
