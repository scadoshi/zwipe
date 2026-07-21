use crate::{Footer, Nav, components::PageMeta};
use dioxus::prelude::*;
use zwipe_components::Panel;

// Source-material links, used inside the architecture diagram (Wikipedia-style refs).
const URL_ZWIPER: &str = "https://github.com/scadoshi/zwipe/tree/main/zwiper";
const URL_ZITE: &str = "https://github.com/scadoshi/zwipe/tree/main/zite";
const URL_ZERVER: &str = "https://github.com/scadoshi/zwipe/tree/main/zerver";
const URL_ZWIPE_CORE: &str = "https://github.com/scadoshi/zwipe/tree/main/zwipe-core";
const URL_ZWIPE_COMPONENTS: &str = "https://github.com/scadoshi/zwipe/tree/main/zwipe-components";
const URL_DIOXUS: &str = "https://dioxuslabs.com";
const URL_WASM: &str = "https://webassembly.org";
const URL_AXUM: &str = "https://github.com/tokio-rs/axum";
const URL_TOKIO: &str = "https://tokio.rs";
const URL_SQLX: &str = "https://github.com/launchbadge/sqlx";
const URL_POSTGRES: &str = "https://www.postgresql.org";
const URL_SCRYFALL: &str = "https://scryfall.com/docs/api";
const URL_CARD_ROLE: &str =
    "https://github.com/scadoshi/zwipe/tree/main/zwipe-core/src/domain/card/models/card_role";

#[component]
pub fn About() -> Element {
    rsx! {
        PageMeta {
            title: "About",
            description: "Zwipe is built by Scotty Fermo (scadoshi). About page covers the tech stack (Rust, Dioxus, WASM, Axum, PostgreSQL) and the architecture behind the deck builder.",
            path: "/about",
        }
        Nav {}
        div { class: "page content-enter",
            div { class: "about-header section panel",
                div { class: "about-headline",
                    h1 { "Scotty Fermo" }
                    div { class: "profile-links",
                        a { class: "profile-link", href: "https://scottyfermo.com", target: "_blank", rel: "noopener noreferrer", "scottyfermo.com ↗" }
                        a { class: "profile-link", href: "https://github.com/scadoshi", target: "_blank", rel: "noopener noreferrer", "GitHub ↗" }
                        a { class: "profile-link", href: "https://www.linkedin.com/in/scotty-fermo-41a35b141/", target: "_blank", rel: "noopener noreferrer", "LinkedIn ↗" }
                    }
                    div { class: "tag-row",
                        span { class: "tag", "Rust" }
                        span { class: "tag", "Full-stack" }
                        span { class: "tag", "iOS" }
                        span { class: "tag", "PostgreSQL" }
                        span { class: "tag", "Systems" }
                    }
                }
                p {
                    "Zwipe is a solo project: designed, built, and shipped by one person.
                    This page is the look under the hood, the architecture and the
                    engineering discipline behind a one-person, full-stack Rust app.
                    The goal was simple: make deck building feel good on mobile, and
                    build it that way from the ground up."
                }
            }

            div { class: "section",
                h2 { "System Architecture" }
                p { class: "arch-subtitle", "Five Rust crates in one workspace. What each one does, and where it pulls from." }
                div { class: "card-grid",
                    Panel {
                        eyebrow: "iOS app",
                        title: "zwiper",
                        actions: rsx! {
                            a { class: "panel-action", href: URL_ZWIPER, target: "_blank", rel: "noopener noreferrer", "Source \u{2197}" }
                        },
                        div { class: "arch-tags",
                            span { class: "tag", "iOS" }
                            a { class: "tag", href: URL_DIOXUS, target: "_blank", rel: "noopener noreferrer", "Dioxus" }
                            a { class: "tag", href: URL_WASM, target: "_blank", rel: "noopener noreferrer", "WASM" }
                        }
                        ul { class: "card-bullets",
                            li { "Swipe to build, search cards, keep decks in sync" }
                            li {
                                "Talks to "
                                a { href: URL_ZERVER, target: "_blank", rel: "noopener noreferrer", "zerver" }
                                " over HTTPS"
                            }
                            li {
                                "Imports "
                                a { href: URL_ZWIPE_CORE, target: "_blank", rel: "noopener noreferrer", "zwipe-core" }
                                " and "
                                a { href: URL_ZWIPE_COMPONENTS, target: "_blank", rel: "noopener noreferrer", "zwipe-components" }
                            }
                        }
                    }
                    Panel {
                        eyebrow: "Website",
                        title: "zite",
                        actions: rsx! {
                            a { class: "panel-action", href: URL_ZITE, target: "_blank", rel: "noopener noreferrer", "Source \u{2197}" }
                        },
                        div { class: "arch-tags",
                            span { class: "tag", "web" }
                            a { class: "tag", href: URL_DIOXUS, target: "_blank", rel: "noopener noreferrer", "Dioxus" }
                            a { class: "tag", href: URL_WASM, target: "_blank", rel: "noopener noreferrer", "WASM" }
                        }
                        ul { class: "card-bullets",
                            li { "Marketing, landing, password reset, and email verification" }
                            li {
                                "Talks to "
                                a { href: URL_ZERVER, target: "_blank", rel: "noopener noreferrer", "zerver" }
                                " over HTTPS"
                            }
                            li {
                                "Imports "
                                a { href: URL_ZWIPE_CORE, target: "_blank", rel: "noopener noreferrer", "zwipe-core" }
                                " and "
                                a { href: URL_ZWIPE_COMPONENTS, target: "_blank", rel: "noopener noreferrer", "zwipe-components" }
                            }
                        }
                    }
                    Panel {
                        eyebrow: "Backend",
                        title: "zerver",
                        actions: rsx! {
                            a { class: "panel-action", href: URL_ZERVER, target: "_blank", rel: "noopener noreferrer", "Source \u{2197}" }
                        },
                        div { class: "arch-tags",
                            span { class: "tag", "REST API" }
                            a { class: "tag", href: URL_AXUM, target: "_blank", rel: "noopener noreferrer", "Axum" }
                            a { class: "tag", href: URL_TOKIO, target: "_blank", rel: "noopener noreferrer", "Tokio" }
                            a { class: "tag", href: URL_SQLX, target: "_blank", rel: "noopener noreferrer", "SQLx" }
                        }
                        ul { class: "card-bullets",
                            li { "The REST API behind everything: auth, sessions, decks, cards, and users" }
                            li {
                                "Reads and writes a "
                                a { href: URL_POSTGRES, target: "_blank", rel: "noopener noreferrer", "PostgreSQL" }
                                " database"
                            }
                            li {
                                "A nightly job pulls the card catalog from "
                                a { href: URL_SCRYFALL, target: "_blank", rel: "noopener noreferrer", "Scryfall" }
                            }
                            li {
                                "Imports "
                                a { href: URL_ZWIPE_CORE, target: "_blank", rel: "noopener noreferrer", "zwipe-core" }
                            }
                        }
                    }
                    Panel {
                        eyebrow: "Shared UI",
                        title: "zwipe-components",
                        actions: rsx! {
                            a { class: "panel-action", href: URL_ZWIPE_COMPONENTS, target: "_blank", rel: "noopener noreferrer", "Source \u{2197}" }
                        },
                        div { class: "arch-tags",
                            a { class: "tag", href: URL_DIOXUS, target: "_blank", rel: "noopener noreferrer", "Dioxus" }
                        }
                        ul { class: "card-bullets",
                            li { "The Dioxus UI shared across the clients: buttons, action bar, card row, changelog" }
                            li {
                                "Reused beyond Zwipe on "
                                a { href: "https://scottyfermo.com", target: "_blank", rel: "noopener noreferrer", "scottyfermo.com" }
                            }
                            li {
                                "Imports "
                                a { href: URL_ZWIPE_CORE, target: "_blank", rel: "noopener noreferrer", "zwipe-core" }
                            }
                        }
                    }
                    Panel {
                        eyebrow: "Shared domain",
                        title: "zwipe-core",
                        actions: rsx! {
                            a { class: "panel-action", href: URL_ZWIPE_CORE, target: "_blank", rel: "noopener noreferrer", "Source \u{2197}" }
                        },
                        div { class: "arch-tags",
                            span { class: "tag", "Pure Rust" }
                        }
                        ul { class: "card-bullets",
                            li { "Models, filter logic, and traits, with no server- or client-only dependencies" }
                            li { "Imported by every other crate" }
                            li { "Same domain code runs SQL filtering on the server and in-memory filtering on the device" }
                        }
                    }
                }
            }

            div { class: "section",
                h2 { "Under the Hood" }
                p { class: "arch-subtitle", "The engineering discipline behind it." }
                div { class: "card-grid",
                    Panel { eyebrow: "Design", title: "Hexagonal Architecture",
                        p { class: "card-summary",
                            "Ports and adapters, in practice."
                        }
                        ul { class: "card-bullets",
                            li {
                                a { href: URL_ZWIPE_CORE, target: "_blank", rel: "noopener noreferrer", "zwipe-core" }
                                ": zero framework dependencies"
                            }
                            li { "Inbound and outbound adapters swap freely" }
                            li { "Same domain code: server SQL and on-device filtering" }
                            li { "One codebase, compiled to iOS, Android, and web" }
                        }
                    }

                    Panel { eyebrow: "Quality", title: "Testing & Lint Discipline",
                        p { class: "card-summary",
                            "627 tests, 377 in "
                            a { href: URL_ZWIPE_CORE, target: "_blank", rel: "noopener noreferrer", "zwipe-core" }
                            ". Enforced by the compiler."
                        }
                        ul { class: "card-bullets",
                            li {
                                code { ".unwrap" }
                                ", "
                                code { "panic!" }
                                ", "
                                code { "todo!" }
                                ", "
                                code { "dbg!" }
                                ", and friends denied at compile time"
                            }
                            li { "22 Clippy rules, workspace-wide" }
                            li {
                                "Compile-time SQL: "
                                a { href: URL_SQLX, target: "_blank", rel: "noopener noreferrer", "sqlx" }
                                " "
                                code { "query!" }
                                " fails the build, not runtime"
                            }
                            li { "Nightly Cloudflare R2 backups" }
                        }
                    }

                    Panel { eyebrow: "Auth", title: "Authentication",
                        p { class: "card-summary",
                            "Hand-rolled, stricter than a deckbuilder needs."
                        }
                        ul { class: "card-bullets",
                            li { "Argon2id, plus a 170-pattern password blocklist" }
                            li { "Rotating refresh tokens, replay-safe" }
                            li { "Short-lived JWTs; refresh tokens stored hashed" }
                            li {
                                code { "Password" }
                                " newtype "
                                em { "consumed" }
                                " on hash, so plaintext can't leak"
                            }
                            li { "Rate limiting, audit logs, transactional email" }
                        }
                    }

                    Panel { eyebrow: "Types", title: "Type Safety",
                        p { class: "card-summary",
                            "Newtypes everywhere. Invalid states don't compile."
                        }
                        ul { class: "card-bullets",
                            li {
                                code { "UserId" }
                                ", "
                                code { "Email" }
                                ", "
                                code { "Password" }
                                ": real types, not "
                                code { "String" }
                            }
                            li { "Builders enforce required fields at construction" }
                            li { "Formats as enums and traits, not bool flags" }
                            li { "Validate once at the boundary, trust it downstream" }
                        }
                    }

                    Panel { eyebrow: "Sync", title: "Card Data Pipeline",
                        p { class: "card-summary",
                            "110k+ printings nightly from "
                            a { href: URL_SCRYFALL, target: "_blank", rel: "noopener noreferrer", "Scryfall" }
                            ". The hard part isn't the cron."
                        }
                        ul { class: "card-bullets",
                            li { "Five-strategy upsert: batch, then per-row on conflict" }
                            li { "~327 cards per batch under Postgres's 65k-param cap" }
                            li {
                                code { "PartialEq" }
                                " delta detection: only changed rows written"
                            }
                            li { "Materialized view for dedup search (~35k unique)" }
                            li {
                                "Roles derived at sync, stored on the row: filter one indexed column (see "
                                a { href: URL_CARD_ROLE, target: "_blank", rel: "noopener noreferrer", "card_role" }
                                ")"
                            }
                        }
                    }
                }
            }
        }
        Footer {}
    }
}
