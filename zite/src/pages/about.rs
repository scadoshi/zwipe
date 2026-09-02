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
            description: "Who builds Zwipe and how: one developer, full-stack Rust (Dioxus, Axum, PostgreSQL), and the architecture behind the app.",
            path: "/about",
        }
        Nav {}
        div { class: "page content-enter",
            div { class: "about-header section",
                // The profile links ride the panel's own action row rather than
                // a bespoke header block, so this hero matches every other page.
                Panel {
                    eyebrow: "About",
                    title: "Scotty Fermo",
                    title_h1: true,
                    actions: rsx! {
                        a { class: "profile-link", href: "https://scottyfermo.com", "scottyfermo.com ↗" }
                        a { class: "profile-link", href: "https://github.com/scadoshi", "GitHub ↗" }
                        a { class: "profile-link", href: "https://www.linkedin.com/in/scotty-fermo-41a35b141/", "LinkedIn ↗" }
                    },
                    div { class: "tag-row",
                        span { class: "tag", "Rust" }
                        span { class: "tag", "Full-stack" }
                        span { class: "tag", "iOS" }
                        span { class: "tag", "Android" }
                        span { class: "tag", "PostgreSQL" }
                        span { class: "tag", "Systems" }
                    }
                    p {
                        "Zwipe is a solo project: designed, built, and shipped by one person.
                        This page is the look under the hood, the architecture and the
                        engineering discipline behind a one-person, full-stack Rust app.
                        The goal was simple: make deck building feel good on mobile, and
                        build it that way from the ground up."
                    }
                }
            }

            div { class: "section",
                h2 { "System Architecture" }
                p { class: "arch-subtitle", "Five Rust crates in one workspace. What each one does, and where it pulls from." }
                div { class: "card-grid",
                    Panel {
                        eyebrow: "Mobile app",
                        title: "zwiper",
                        actions: rsx! {
                            a { class: "panel-action", href: URL_ZWIPER, "Source \u{2197}" }
                        },
                        div { class: "arch-tags",
                            span { class: "tag", "iOS" }
                            span { class: "tag", "Android" }
                            a { class: "tag", href: URL_DIOXUS, "Dioxus" }
                            a { class: "tag", href: URL_WASM, "WASM" }
                        }
                        ul { class: "card-bullets",
                            li { "Swipe to build, search cards, keep decks in sync" }
                            li {
                                "Talks to "
                                a { href: URL_ZERVER, "zerver" }
                                " over HTTPS"
                            }
                            li {
                                "Imports "
                                a { href: URL_ZWIPE_CORE, "zwipe-core" }
                                " and "
                                a { href: URL_ZWIPE_COMPONENTS, "zwipe-components" }
                            }
                        }
                    }
                    Panel {
                        eyebrow: "Website",
                        title: "zite",
                        actions: rsx! {
                            a { class: "panel-action", href: URL_ZITE, "Source \u{2197}" }
                        },
                        div { class: "arch-tags",
                            span { class: "tag", "web" }
                            a { class: "tag", href: URL_DIOXUS, "Dioxus" }
                            a { class: "tag", href: URL_WASM, "WASM" }
                        }
                        ul { class: "card-bullets",
                            li { "Marketing, landing, password reset, and email verification" }
                            li {
                                "Talks to "
                                a { href: URL_ZERVER, "zerver" }
                                " over HTTPS"
                            }
                            li {
                                "Imports "
                                a { href: URL_ZWIPE_CORE, "zwipe-core" }
                                " and "
                                a { href: URL_ZWIPE_COMPONENTS, "zwipe-components" }
                            }
                        }
                    }
                    Panel {
                        eyebrow: "Backend",
                        title: "zerver",
                        actions: rsx! {
                            a { class: "panel-action", href: URL_ZERVER, "Source \u{2197}" }
                        },
                        div { class: "arch-tags",
                            span { class: "tag", "REST API" }
                            a { class: "tag", href: URL_AXUM, "Axum" }
                            a { class: "tag", href: URL_TOKIO, "Tokio" }
                            a { class: "tag", href: URL_SQLX, "SQLx" }
                        }
                        ul { class: "card-bullets",
                            li { "The REST API behind everything: auth, sessions, decks, cards, and users" }
                            li {
                                "Reads and writes a "
                                a { href: URL_POSTGRES, "PostgreSQL" }
                                " database"
                            }
                            li {
                                "A nightly job pulls the card catalog from "
                                a { href: URL_SCRYFALL, "Scryfall" }
                            }
                            li {
                                "Imports "
                                a { href: URL_ZWIPE_CORE, "zwipe-core" }
                            }
                        }
                    }
                    Panel {
                        eyebrow: "Shared UI",
                        title: "zwipe-components",
                        actions: rsx! {
                            a { class: "panel-action", href: URL_ZWIPE_COMPONENTS, "Source \u{2197}" }
                        },
                        div { class: "arch-tags",
                            a { class: "tag", href: URL_DIOXUS, "Dioxus" }
                        }
                        ul { class: "card-bullets",
                            li { "The Dioxus UI shared across the clients: buttons, action bar, card row, changelog" }
                            li {
                                "Reused beyond Zwipe on "
                                a { href: "https://scottyfermo.com", "scottyfermo.com" }
                            }
                            li {
                                "Imports "
                                a { href: URL_ZWIPE_CORE, "zwipe-core" }
                            }
                        }
                    }
                    Panel {
                        eyebrow: "Shared domain",
                        title: "zwipe-core",
                        actions: rsx! {
                            a { class: "panel-action", href: URL_ZWIPE_CORE, "Source \u{2197}" }
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
                    Panel { eyebrow: "Design", title: "Hexagonal architecture",
                        p { class: "card-summary",
                            "Ports and adapters, in practice."
                        }
                        ul { class: "card-bullets",
                            li {
                                a { href: URL_ZWIPE_CORE, "zwipe-core" }
                                ": zero framework dependencies"
                            }
                            li { "Inbound and outbound adapters swap freely" }
                            li { "Same domain code: server SQL and on-device filtering" }
                            li { "One codebase, compiled to iOS, Android, and web" }
                        }
                    }

                    Panel { eyebrow: "Quality", title: "Testing & lint discipline",
                        p { class: "card-summary",
                            "694 tests, 406 in "
                            a { href: URL_ZWIPE_CORE, "zwipe-core" }
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
                                a { href: URL_SQLX, "sqlx" }
                                " "
                                code { "query!" }
                                " fails the build, not runtime"
                            }
                            li { "Nightly Cloudflare R2 backups" }
                        }
                    }

                    Panel { eyebrow: "Process", title: "How AI is used",
                        p { class: "card-summary",
                            "The learning comes first. The typing comes last."
                        }
                        ul { class: "card-bullets",
                            li { "It starts with research: teaching myself, cross-referencing with a model, learning the approaches before picking one" }
                            li { "Security checked at every stop, implementations validated against other models, everything tested" }
                            li { "Only once I could build it myself does the model take over the typing, and I read the code. That's where the speed comes from" }
                            li { "All of it is open source. Read it; criticism is welcome" }
                        }
                    }

                    Panel { eyebrow: "Auth", title: "Authentication",
                        p { class: "card-summary",
                            "Hand-rolled, stricter than a deckbuilder needs."
                        }
                        ul { class: "card-bullets",
                            li { "Argon2id, with length, character-class and repetition rules" }
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

                    Panel { eyebrow: "Types", title: "Type safety",
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

                    Panel { eyebrow: "Sync", title: "Card data pipeline",
                        p { class: "card-summary",
                            "110k+ printings nightly from "
                            a { href: URL_SCRYFALL, "Scryfall" }
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
                                a { href: URL_CARD_ROLE, "card_role" }
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
