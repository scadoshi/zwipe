//! Zwipe's architecture as a two-slide carousel in a panel: how the six
//! crates import each other, then how the running pieces talk. zite's About
//! page and the portfolio's Zwipe page both show it, so it lives here.
//!
//! Inline SVG through the shared diagram pieces, so the boxes take the theme's
//! colors; the frame and footer are the demo gallery's ([`GalleryFrame`],
//! [`GalleryFooter`]), restyled for short wide diagrams by `.arch-carousel` in
//! the site sheet.

use dioxus::prelude::*;

use crate::{
    DiagramArrow, DiagramDefs, DiagramNode, DiagramTone, GalleryFooter, GalleryFrame, Panel,
};

/// The caption under each slide, in slide order.
const CAPTIONS: [&str; 2] = [
    "Six crates in one workspace: both apps share one typed client, one UI kit and one core, and the server shares the core",
    "Two clients, one server: the app and the site call the same API, and the card catalog syncs from Scryfall nightly",
];

/// The carousel in a panel. `title` names the panel; the eyebrow is
/// "Architecture".
#[component]
pub fn ZwipeArchitecture(
    #[props(default = "How it fits together".to_string())] title: String,
) -> Element {
    let index = use_signal(|| 0usize);
    let total = CAPTIONS.len();
    rsx! {
        Panel {
            eyebrow: "Architecture",
            title,
            actions: rsx! {
                GalleryFooter { index: index(), total, caption: CAPTIONS.get(index()).map(|c| (*c).to_string()) }
            },
            GalleryFrame { index, total, noun: "diagram", class: "arch-carousel".to_string(),
                div { key: "{index()}", class: "arch-slide diagram-scroll",
                    if index() == 0 {
                        Crates {}
                    } else {
                        Infrastructure {}
                    }
                }
            }
        }
    }
}

/// Which crate imports which: the two apps over the shared crates, the server
/// over the core alone.
#[component]
fn Crates() -> Element {
    rsx! {
        svg {
            class: "diagram",
            view_box: "0 0 720 236",
            role: "img",
            "aria-label": "zwiper and zite import the shared crates, zerver imports zwipe-core, and zwipe-client and zwipe-components both import zwipe-core",
            DiagramDefs {}
            DiagramNode { x: 30.0, y: 16.0, title: "zwiper", sub: "iOS and Android app", tone: DiagramTone::Primary }
            DiagramNode { x: 285.0, y: 16.0, title: "zerver", sub: "Axum, the REST API", tone: DiagramTone::Tertiary }
            DiagramNode { x: 540.0, y: 16.0, title: "zite", sub: "zwipe.net", tone: DiagramTone::Primary }
            g { class: "diagram-group",
                rect { x: "10", y: "136", width: "700", height: "92", rx: "10" }
                text { x: "360", y: "224", text_anchor: "middle", "shared crates, imported by both clients" }
            }
            DiagramNode { x: 30.0, y: 150.0, w: 170.0, title: "zwipe-client", sub: "typed API calls", tone: DiagramTone::Secondary }
            DiagramNode { x: 275.0, y: 150.0, w: 170.0, title: "zwipe-core", sub: "models, filters, traits", tone: DiagramTone::Success }
            DiagramNode { x: 520.0, y: 150.0, w: 170.0, title: "zwipe-components", sub: "shared Dioxus UI", tone: DiagramTone::Secondary }
            DiagramArrow { x1: 105.0, y1: 68.0, x2: 105.0, y2: 136.0, label: "imports" }
            DiagramArrow { x1: 615.0, y1: 68.0, x2: 615.0, y2: 136.0, label: "imports" }
            DiagramArrow { x1: 360.0, y1: 68.0, x2: 360.0, y2: 150.0, label: "imports" }
            DiagramArrow { x1: 200.0, y1: 176.0, x2: 275.0, y2: 176.0 }
            DiagramArrow { x1: 520.0, y1: 176.0, x2: 445.0, y2: 176.0 }
        }
    }
}

/// The running system: both clients call the server, which keeps PostgreSQL
/// and pulls the catalog from Scryfall.
#[component]
fn Infrastructure() -> Element {
    rsx! {
        svg {
            class: "diagram",
            view_box: "0 0 720 196",
            role: "img",
            "aria-label": "zwiper and zite call zerver over HTTPS; zerver reads and writes PostgreSQL and pulls the card catalog from Scryfall nightly",
            DiagramDefs {}
            DiagramNode { x: 10.0, y: 16.0, title: "Scryfall", sub: "the card catalog", tone: DiagramTone::Muted }
            DiagramNode { x: 285.0, y: 16.0, title: "zerver", sub: "Axum, the REST API", tone: DiagramTone::Tertiary }
            DiagramNode { x: 560.0, y: 16.0, title: "PostgreSQL", sub: "SQLx", tone: DiagramTone::Muted }
            DiagramNode { x: 100.0, y: 130.0, title: "zwiper", sub: "iOS and Android", tone: DiagramTone::Primary }
            DiagramNode { x: 470.0, y: 130.0, title: "zite", sub: "zwipe.net", tone: DiagramTone::Primary }
            DiagramArrow { x1: 160.0, y1: 42.0, x2: 285.0, y2: 42.0, label: "nightly" }
            DiagramArrow { x1: 435.0, y1: 42.0, x2: 560.0, y2: 42.0, label: "reads and writes" }
            DiagramArrow { x1: 175.0, y1: 130.0, x2: 330.0, y2: 68.0, label: "HTTPS" }
            DiagramArrow { x1: 545.0, y1: 130.0, x2: 390.0, y2: 68.0, label: "HTTPS" }
        }
    }
}
