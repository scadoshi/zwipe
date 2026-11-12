use components::{Panel, ZwipeArchitecture};
use dioxus::prelude::*;

use crate::components::PageMeta;
use crate::{Footer, Nav, Route};

// Source-material links, used inside the architecture diagram (Wikipedia-style refs).
const URL_ZWIPER: &str = "https://github.com/scadoshi/zwipe/tree/main/zwiper";
const URL_ZWIPER_WEB: &str = "https://github.com/scadoshi/zwipe/tree/main/zwiper-web";
const URL_ZITE: &str = "https://github.com/scadoshi/zwipe/tree/main/zite";
const URL_ZERVER: &str = "https://github.com/scadoshi/zwipe/tree/main/zerver";
const URL_ZWIPE_CORE: &str = "https://github.com/scadoshi/zwipe/tree/main/zwipe-core";
const URL_COMPONENTS: &str = "https://github.com/scadoshi/zwipe/tree/main/components";
const URL_ZWIPE_UI: &str = "https://github.com/scadoshi/zwipe/tree/main/zwipe-ui";
const URL_ZWIPE_CLIENT: &str = "https://github.com/scadoshi/zwipe/tree/main/zwipe-client";
const URL_ORANGUI: &str = "https://github.com/scadoshi/orangui";
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
            description: "How Zwipe is built: an open-source, full-stack Rust app, and the architecture behind it.",
            path: "/about/",
        }
        Nav {}
        main { id: "content", class: "page content-enter",
            div { class: "about-header section",
                Panel {
                    eyebrow: "About",
                    title: "Zwipe",
                    title_h1: true,
                    actions: rsx! {
                        a { class: "panel-action", href: "https://github.com/scadoshi/zwipe", "Source" span { class: "ext", "\u{2197}" } }
                        a { class: "panel-action", href: "https://scottyfermo.com", "Built by Scotty" span { class: "ext", "\u{2197}" } }
                        Link { class: "panel-action", to: Route::Privacy {}, "Privacy policy" }
                        Link { class: "panel-action", to: Route::Changelog {}, "Changelog" }
                    },
                    p {
                        "Zwipe is built and maintained by Scotty Fermo, a Commander player who
                        wanted deck building to work well on a phone. The rest of this page
                        covers how it's built."
                    }
                }
            }

            div { class: "section",
                div { class: "section-head",
                    Panel { title: "System architecture",
                        p { class: "card-summary", "Eight Rust crates in one workspace, on top of orangui, the UI kit I built and share between this app, this site and two other apps of mine. What each one does, and where it pulls from." }
                    }
                }
                div { class: "arch-figure", ZwipeArchitecture {} }
                div { class: "card-grid",
                    Panel {
                        eyebrow: "Mobile app",
                        title: "zwiper",
                        actions: rsx! {
                            a { class: "panel-action", href: URL_ZWIPER, "Source" span { class: "ext", "\u{2197}" } }
                        },
                        div { class: "arch-tags",
                            span { class: "tag tag-p1", "iOS" }
                            span { class: "tag tag-p2", "Android" }
                            a { class: "tag tag-p3", href: URL_DIOXUS, "Dioxus" }
                            a { class: "tag tag-p4", href: URL_WASM, "WASM" }
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
                                ", "
                                a { href: URL_ZWIPE_CLIENT, "zwipe-client" }
                                ", "
                                a { href: URL_ZWIPE_UI, "zwipe-ui" }
                                " and "
                                a { href: URL_COMPONENTS, "components" }
                            }
                        }
                    }
                    Panel {
                        eyebrow: "Web app",
                        title: "zwiper-web",
                        actions: rsx! {
                            a { class: "panel-action", href: URL_ZWIPER_WEB, "Source" span { class: "ext", "\u{2197}" } }
                        },
                        div { class: "arch-tags",
                            span { class: "tag tag-p1", "web" }
                            a { class: "tag tag-p2", href: URL_DIOXUS, "Dioxus" }
                            a { class: "tag tag-p3", href: URL_WASM, "WASM" }
                        }
                        ul { class: "card-bullets",
                            li {
                                "The app on the web, at "
                                a { href: "https://app.zwipe.net", "app.zwipe.net" }
                                ": the same decks and swipes in a browser, and every shared deck's page"
                            }
                            li {
                                "Talks to "
                                a { href: URL_ZERVER, "zerver" }
                                " over HTTPS"
                            }
                            li {
                                "Imports "
                                a { href: URL_ZWIPE_CORE, "zwipe-core" }
                                ", "
                                a { href: URL_ZWIPE_CLIENT, "zwipe-client" }
                                ", "
                                a { href: URL_ZWIPE_UI, "zwipe-ui" }
                                " and "
                                a { href: URL_COMPONENTS, "components" }
                            }
                        }
                    }
                    Panel {
                        eyebrow: "Website",
                        title: "zite",
                        actions: rsx! {
                            a { class: "panel-action", href: URL_ZITE, "Source" span { class: "ext", "\u{2197}" } }
                        },
                        div { class: "arch-tags",
                            span { class: "tag tag-p1", "web" }
                            a { class: "tag tag-p2", href: URL_DIOXUS, "Dioxus" }
                            a { class: "tag tag-p3", href: URL_WASM, "WASM" }
                        }
                        ul { class: "card-bullets",
                            li { "Landing and guides, the changelog, password reset, email verification and email change confirmation" }
                            li {
                                "Talks to "
                                a { href: URL_ZERVER, "zerver" }
                                " over HTTPS"
                            }
                            li {
                                "Imports "
                                a { href: URL_ZWIPE_CORE, "zwipe-core" }
                                ", "
                                a { href: URL_ZWIPE_CLIENT, "zwipe-client" }
                                " and "
                                a { href: URL_COMPONENTS, "components" }
                            }
                        }
                    }
                    Panel {
                        eyebrow: "Backend",
                        title: "zerver",
                        actions: rsx! {
                            a { class: "panel-action", href: URL_ZERVER, "Source" span { class: "ext", "\u{2197}" } }
                        },
                        div { class: "arch-tags",
                            span { class: "tag tag-p1", "REST API" }
                            a { class: "tag tag-p2", href: URL_AXUM, "Axum" }
                            a { class: "tag tag-p3", href: URL_TOKIO, "Tokio" }
                            a { class: "tag tag-p4", href: URL_SQLX, "SQLx" }
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
                        eyebrow: "Shared screen logic",
                        title: "zwipe-ui",
                        actions: rsx! {
                            a { class: "panel-action", href: URL_ZWIPE_UI, "Source" span { class: "ext", "\u{2197}" } }
                        },
                        div { class: "arch-tags",
                            span { class: "tag tag-p1", "Pure Rust" }
                            span { class: "tag tag-p2", "Any client" }
                        }
                        ul { class: "card-bullets",
                            li { "What every screen shows and does, decided once: color bars, the filter draft, the swipe stack, hint copy, timings" }
                            li { "No UI crate, so a web, desktop or terminal client draws the same screens as the app" }
                            li {
                                "Imports "
                                a { href: URL_ZWIPE_CORE, "zwipe-core" }
                                " and "
                                a { href: URL_ORANGUI, "orangui" }
                                "'s Dioxus-free half"
                            }
                        }
                    }
                    Panel {
                        eyebrow: "Shared UI",
                        title: "components",
                        actions: rsx! {
                            a { class: "panel-action", href: URL_COMPONENTS, "Source" span { class: "ext", "\u{2197}" } }
                        },
                        div { class: "arch-tags",
                            a { class: "tag tag-p1", href: URL_DIOXUS, "Dioxus" }
                            a { class: "tag tag-p2", href: URL_ORANGUI, "orangui" }
                        }
                        ul { class: "card-bullets",
                            li { "The Magic UI shared across the clients: card row, deck charts, changelog, this diagram" }
                            li {
                                "Built on "
                                a { href: URL_ORANGUI, "orangui" }
                                ", the UI kit I wrote for all my apps: themes, panels, nav, charts"
                            }
                            li {
                                "Reused beyond Zwipe on "
                                a { href: "https://scottyfermo.com", "scottyfermo.com" }
                            }
                            li {
                                "Imports "
                                a { href: URL_ZWIPE_CORE, "zwipe-core" }
                                ", "
                                a { href: URL_ZWIPE_UI, "zwipe-ui" }
                                " and "
                                a { href: URL_ORANGUI, "orangui" }
                            }
                        }
                    }
                    Panel {
                        eyebrow: "Shared API client",
                        title: "zwipe-client",
                        actions: rsx! {
                            a { class: "panel-action", href: URL_ZWIPE_CLIENT, "Source" span { class: "ext", "\u{2197}" } }
                        },
                        div { class: "arch-tags",
                            span { class: "tag tag-p1", "HTTP" }
                            span { class: "tag tag-p2", "Typed" }
                        }
                        ul { class: "card-bullets",
                            li { "Every API call the app, the web app and the site make, described once and sent by one function" }
                            li {
                                "Each endpoint declares its method, path and response type in "
                                a { href: URL_ZWIPE_CORE, "zwipe-core" }
                                ", so a client can't call it the wrong way"
                            }
                            li { "No UI framework and no platform code, so it compiles for phones and the browser alike" }
                        }
                    }
                    Panel {
                        eyebrow: "Shared domain",
                        title: "zwipe-core",
                        actions: rsx! {
                            a { class: "panel-action", href: URL_ZWIPE_CORE, "Source" span { class: "ext", "\u{2197}" } }
                        },
                        div { class: "arch-tags",
                            span { class: "tag tag-p1", "Pure Rust" }
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
                div { class: "section-head",
                    Panel { title: "Under the hood",
                        p { class: "card-summary", "The engineering discipline behind it." }
                    }
                }
                div { class: "card-grid",
                    Panel { title: "Hexagonal architecture",
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

                    Panel { title: "Testing & lint discipline",
                        p { class: "card-summary",
                            "750+ tests, 450+ in "
                            a { href: URL_ZWIPE_CORE, "zwipe-core" }
                            "."
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

                    Panel { title: "How AI is used",
                        p { class: "card-summary",
                            "I design Zwipe's structure, and agents write the code inside it. There's more of it than I could review line by line, so strict checks decide what ships, the same way every time."
                        }
                        ul { class: "card-bullets",
                            li { "Nothing deploys unless over a thousand tests and strict lints pass" }
                            li { "Queries, types and dependencies are checked before anything runs" }
                            li { "I pick the parts worth learning and go deep on those by hand" }
                            li { "All of it is open source. Read it; criticism is welcome" }
                        }
                    }

                    Panel { title: "Authentication",
                        p { class: "card-summary",
                            "Built in-house rather than from an auth service."
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

                    Panel { title: "Type safety",
                        p { class: "card-summary",
                            "User input becomes a validated type before anything trusts it."
                        }
                        ul { class: "card-bullets",
                            li {
                                code { "Username" }
                                ", "
                                code { "DeckName" }
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

                    Panel { title: "Card data pipeline",
                        p { class: "card-summary",
                            "115k+ printings synced nightly from "
                            a { href: URL_SCRYFALL, "Scryfall" }
                            "."
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

#[cfg(test)]
mod tests {
    const SOURCE: &str = include_str!("about.rs");

    /// The workspace's members, read from its root `Cargo.toml`.
    fn workspace() -> Vec<&'static str> {
        let manifest = include_str!("../../../Cargo.toml");
        let (_, after) = manifest.split_once("members = [").unwrap();
        let (members, _) = after.split_once(']').unwrap();
        let mut names: Vec<&str> = members.split('"').skip(1).step_by(2).collect();
        names.sort_unstable();
        names
    }

    /// The System architecture section: its summary, then the crate cards
    /// up to the next section.
    fn section() -> &'static str {
        let (_, after) = SOURCE.split_once("title: \"System architecture\"").unwrap();
        after.split_once("title: \"Under the hood\"").unwrap().0
    }

    #[test]
    fn the_crate_cards_are_the_workspace_members() {
        let mut cards: Vec<&str> = section()
            .split("title: \"")
            .skip(1)
            .filter_map(|rest| rest.split('"').next())
            .collect();
        cards.sort_unstable();
        assert_eq!(cards, workspace(), "one card per workspace crate");
    }

    #[test]
    fn the_summary_counts_the_crates() {
        let words = [
            "Zero", "One", "Two", "Three", "Four", "Five", "Six", "Seven", "Eight", "Nine", "Ten",
        ];
        let count = words.get(workspace().len()).unwrap();
        assert!(
            section().contains(&format!("\"{count} Rust crates in one workspace")),
            "the summary says {count}"
        );
    }
}
