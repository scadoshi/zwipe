use dioxus::prelude::*;
use crate::{Footer, Nav};

#[component]
pub fn About() -> Element {
    rsx! {
        Nav {}
        div { class: "page content-enter",
            div { class: "about-header section",
                h1 { "scotty fermo" }
                p { class: "subtitle",
                    "software developer · scadoshi | "
                    a { href: "https://scottyfermo.com", target: "_blank", rel: "noopener noreferrer", "scottyfermo.com ↗" }
                    " | "
                    a { href: "https://github.com/scadoshi", target: "_blank", rel: "noopener noreferrer", "github ↗" }
                    " | "
                    a { href: "https://x.com/scadoshi", target: "_blank", rel: "noopener noreferrer", "x ↗" }
                    " | "
                    a { href: "https://www.linkedin.com/in/scotty-fermo-41a35b141/", target: "_blank", rel: "noopener noreferrer", "linkedin ↗" }
                    " | "
                    a { href: "https://github.com/scadoshi/zwipe", target: "_blank", rel: "noopener noreferrer", "zwipe ↗" }
                }
            }

            div { class: "section",
                div { class: "tag-row",
                    span { class: "tag", "rust" }
                    span { class: "tag", "full-stack" }
                    span { class: "tag", "ios" }
                    span { class: "tag", "postgresql" }
                    span { class: "tag", "systems" }
                }
                p {
                    "zwipe is a solo project — designed, built, and shipped by one person.
                    the goal was simple: make deck building feel good on mobile. every existing
                    tool is a desktop app awkwardly squeezed onto a small screen. zwipe is
                    built mobile-first from the ground up."
                }
            }

            div { class: "section",
                h2 { "the stack" }
                div { class: "card",
                    h3 { "backend — zerver" }
                    p {
                        "rust + "
                        a { href: "https://github.com/tokio-rs/axum", target: "_blank", rel: "noopener noreferrer", "axum" }
                        " rest api backed by postgresql. "
                        a { href: "https://github.com/launchbadge/sqlx", target: "_blank", rel: "noopener noreferrer", "sqlx" }
                        " with compile-time query verification. jwt access tokens + rotating refresh tokens,
                        argon2 password hashing, rate limiting via tower_governor, structured audit logging,
                        resend for transactional email. partner, background, and signature spell fields
                        on deck profiles. maybeboard with metrics exclusion. warning action system.
                        commander eligibility validation per format."
                    }
                }
                div { class: "card",
                    h3 { "frontend — zwiper" }
                    p {
                        a { href: "https://dioxuslabs.com", target: "_blank", rel: "noopener noreferrer", "dioxus" }
                        " cross-platform ui in rust — same codebase targets ios, android, and web.
                        signals-based reactivity, custom swipe gesture handling, modular filter system
                        with accordion ui, real-time deck metrics. maybeboard support with swipe-up
                        gesture. partner, background, and signature spell fields. command zone display.
                        per-section filter clear buttons. buy links and price stats."
                    }
                }
                div { class: "card",
                    h3 { "architecture" }
                    p {
                        "hexagonal (ports & adapters) throughout. domain logic lives in zwipe-core —
                        a pure shared crate with no framework dependencies. zerver and zwiper both
                        depend on zwipe-core for shared types. inbound adapters (http handlers, ui
                        screens) and outbound adapters (sqlx repositories, api clients) are swappable."
                    }
                    pre { class: "arch-diagram",
                        "  zwiper ────→ zwipe-core ←──── zerver\n  (mobile)      (domain)         (api)\n                   ↑               │\n                   │          zervice (sync)\n                 zite\n                 (web)"
                    }
                }
                div { class: "card",
                    h3 { "card data" }
                    p {
                        "110k+ english printings synced from the "
                        a { href: "https://scryfall.com/docs/api", target: "_blank", rel: "noopener noreferrer", "scryfall api" }
                        ". a background service (zervice) runs nightly to pull new sets and update
                        card data. oracle text, color identity, type lines, and image uris are all
                        stored locally in postgres — zero scryfall dependency at query time."
                    }
                }
                div { class: "card",
                    h3 { "testing" }
                    p {
                        "340+ unit tests across domain logic, value object validation, and import parsing.
                        220 in zwipe-core covering commander eligibility, partner validation, deck
                        metrics, and card filtering. newtypes enforce correctness at construction — "
                        code { "UserId" }
                        ", "
                        code { "DeckId" }
                        ", "
                        code { "EmailAddress" }
                        ", "
                        code { "Password" }
                        " — so invalid states can't be passed around."
                    }
                }
            }
        }
        Footer {}
    }
}
