use dioxus::prelude::*;
use crate::components::PageMeta;
use crate::{Footer, Nav};

// Source-material links, used inside the architecture diagram (Wikipedia-style refs).
const URL_ZWIPER: &str = "https://github.com/scadoshi/zwipe/tree/main/zwiper";
const URL_ZITE: &str = "https://github.com/scadoshi/zwipe/tree/main/zite";
const URL_ZERVER: &str = "https://github.com/scadoshi/zwipe/tree/main/zerver";
const URL_ZWIPE_CORE: &str = "https://github.com/scadoshi/zwipe/tree/main/zwipe-core";
const URL_DIOXUS: &str = "https://dioxuslabs.com";
const URL_WASM: &str = "https://webassembly.org";
const URL_AXUM: &str = "https://github.com/tokio-rs/axum";
const URL_TOKIO: &str = "https://tokio.rs";
const URL_SQLX: &str = "https://github.com/launchbadge/sqlx";
const URL_POSTGRES: &str = "https://www.postgresql.org";
const URL_SCRYFALL: &str = "https://scryfall.com/docs/api";
const URL_MECHANICAL_CATEGORY: &str =
    "https://github.com/scadoshi/zwipe/tree/main/zwipe-core/src/domain/card/models/mechanical_category";

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
                h1 { "Scotty Fermo" }
                p { class: "tagline", "Software developer · scadoshi" }
                div { class: "profile-links",
                    a { class: "profile-link", href: "https://scottyfermo.com", target: "_blank", rel: "noopener noreferrer", "scottyfermo.com ↗" }
                    a { class: "profile-link", href: "https://github.com/scadoshi", target: "_blank", rel: "noopener noreferrer", "GitHub ↗" }
                    a { class: "profile-link", href: "https://x.com/scadoshi", target: "_blank", rel: "noopener noreferrer", "X ↗" }
                    a { class: "profile-link", href: "https://www.linkedin.com/in/scotty-fermo-41a35b141/", target: "_blank", rel: "noopener noreferrer", "LinkedIn ↗" }
                    a { class: "profile-link", href: "https://github.com/scadoshi/zwipe", target: "_blank", rel: "noopener noreferrer", "Zwipe ↗" }
                }
                div { class: "tag-row",
                    span { class: "tag", "rust" }
                    span { class: "tag", "full-stack" }
                    span { class: "tag", "ios" }
                    span { class: "tag", "postgresql" }
                    span { class: "tag", "systems" }
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
                p { class: "arch-subtitle", "Four-crate Rust workspace. Every box links to its source." }
                div { class: "arch",
                    div { class: "arch-tier arch-tier-clients",
                        div { class: "arch-box",
                            div { class: "arch-box-head",
                                span { class: "arch-box-title",
                                    a { href: URL_ZWIPER, target: "_blank", rel: "noopener noreferrer", "zwiper" }
                                }
                                span { class: "arch-box-sub",
                                    "iOS · "
                                    a { href: URL_DIOXUS, target: "_blank", rel: "noopener noreferrer", "Dioxus" }
                                    " + "
                                    a { href: URL_WASM, target: "_blank", rel: "noopener noreferrer", "WASM" }
                                }
                            }
                            ul { class: "arch-box-list",
                                li { "swipe-to-build deckbuilder" }
                                li { "card search · deck profiles" }
                                li { "authenticated user UI" }
                            }
                            div { class: "arch-imports",
                                "imports "
                                a { href: URL_ZWIPE_CORE, target: "_blank", rel: "noopener noreferrer", "zwipe-core" }
                            }
                        }
                        div { class: "arch-box",
                            div { class: "arch-box-head",
                                span { class: "arch-box-title",
                                    a { href: URL_ZITE, target: "_blank", rel: "noopener noreferrer", "zite" }
                                }
                                span { class: "arch-box-sub",
                                    "web · "
                                    a { href: URL_DIOXUS, target: "_blank", rel: "noopener noreferrer", "Dioxus" }
                                    " + "
                                    a { href: URL_WASM, target: "_blank", rel: "noopener noreferrer", "WASM" }
                                }
                            }
                            ul { class: "arch-box-list",
                                li { "marketing & landing" }
                                li { "password reset flow" }
                                li { "email verification flow" }
                            }
                            div { class: "arch-imports",
                                "imports "
                                a { href: URL_ZWIPE_CORE, target: "_blank", rel: "noopener noreferrer", "zwipe-core" }
                            }
                        }
                    }
                    div { class: "arch-flow",
                        span { class: "arch-flow-label", "HTTPS / JSON" }
                        span { class: "arch-flow-arrow", "\u{2193}" }
                    }
                    div { class: "arch-tier arch-tier-api",
                        div { class: "arch-box arch-box-emphasis",
                            div { class: "arch-box-head",
                                span { class: "arch-box-title",
                                    a { href: URL_ZERVER, target: "_blank", rel: "noopener noreferrer", "zerver" }
                                }
                                span { class: "arch-box-sub",
                                    "REST API · "
                                    a { href: URL_AXUM, target: "_blank", rel: "noopener noreferrer", "Axum" }
                                    " · "
                                    a { href: URL_TOKIO, target: "_blank", rel: "noopener noreferrer", "Tokio" }
                                    " · "
                                    a { href: URL_SQLX, target: "_blank", rel: "noopener noreferrer", "SQLx" }
                                }
                            }
                            ul { class: "arch-box-list",
                                li { "auth · sessions · JWT" }
                                li { "decks · cards · users" }
                            }
                            div { class: "arch-nested",
                                div { class: "arch-box-head",
                                    span { class: "arch-box-title arch-nested-title", "zervice" }
                                    span { class: "arch-box-sub", "scheduled job (runs on the zerver)" }
                                }
                                ul { class: "arch-box-list",
                                    li { "nightly Scryfall card-catalog sync" }
                                    li { "writes into the same Postgres" }
                                }
                            }
                            div { class: "arch-imports",
                                "imports "
                                a { href: URL_ZWIPE_CORE, target: "_blank", rel: "noopener noreferrer", "zwipe-core" }
                            }
                        }
                        div { class: "arch-box",
                            div { class: "arch-box-head",
                                span { class: "arch-box-title",
                                    a { href: URL_SCRYFALL, target: "_blank", rel: "noopener noreferrer", "Scryfall API" }
                                }
                                span { class: "arch-box-sub", "external service" }
                            }
                            ul { class: "arch-box-list",
                                li { "card catalog source" }
                                li { "nightly sync \u{2192} zervice" }
                            }
                        }
                    }
                    div { class: "arch-flow",
                        span { class: "arch-flow-label", "SQL" }
                        span { class: "arch-flow-arrow", "\u{2193}" }
                    }
                    div { class: "arch-tier arch-tier-db",
                        div { class: "arch-box arch-box-wide",
                            div { class: "arch-box-head",
                                span { class: "arch-box-title",
                                    a { href: URL_POSTGRES, target: "_blank", rel: "noopener noreferrer", "PostgreSQL" }
                                }
                                span { class: "arch-box-sub", "primary datastore" }
                            }
                            ul { class: "arch-box-list arch-box-list-inline",
                                li { "users" }
                                li { "decks" }
                                li { "cards" }
                            }
                        }
                    }
                    div { class: "arch-foundation",
                        div { class: "arch-box-head",
                            span { class: "arch-box-title",
                                a { href: URL_ZWIPE_CORE, target: "_blank", rel: "noopener noreferrer", "zwipe-core" }
                            }
                            span { class: "arch-box-sub", "shared domain crate" }
                        }
                        p { class: "arch-foundation-body",
                            "models, filter logic, traits. No server- or client-only deps. "
                            "Imported by "
                            a { href: URL_ZWIPER, target: "_blank", rel: "noopener noreferrer", "zwiper" }
                            " · "
                            a { href: URL_ZERVER, target: "_blank", rel: "noopener noreferrer", "zerver" }
                            " · "
                            a { href: URL_ZITE, target: "_blank", rel: "noopener noreferrer", "zite" }
                            "."
                        }
                    }
                    div { class: "arch-legend",
                        span { class: "arch-legend-item",
                            span { class: "arch-legend-swatch arch-legend-solid" }
                            "runtime data flow"
                        }
                        span { class: "arch-legend-item",
                            span { class: "arch-legend-swatch arch-legend-dashed" }
                            "build-time dependency (cargo)"
                        }
                    }
                }
            }

            div { class: "section",
                h2 { "Under the Hood" }
                p { class: "arch-subtitle", "What the diagram doesn't show." }
                div { class: "card-grid",
                    div { class: "card",
                        span { class: "card-category", "Design" }
                        h3 { class: "card-title", "Hexagonal Architecture" }
                        p { class: "card-summary",
                            "Ports & adapters, in practice, not just on the whiteboard."
                        }
                        ul { class: "card-bullets",
                            li {
                                a { href: URL_ZWIPE_CORE, target: "_blank", rel: "noopener noreferrer", "zwipe-core" }
                                " has zero framework deps: no Axum, no Dioxus, no sqlx"
                            }
                            li { "Inbound adapters (HTTP, UI) and outbound adapters (sqlx, HTTP client) swap freely" }
                            li { "Same domain code drives server-side SQL filtering and on-device in-memory filtering" }
                            li { "One Rust codebase compiles to iOS, Android, and web from the same source" }
                        }
                    }

                    div { class: "card",
                        span { class: "card-category", "Quality" }
                        h3 { class: "card-title", "Testing & Lint Discipline" }
                        p { class: "card-summary",
                            "505 unit tests; 346 in "
                            a { href: URL_ZWIPE_CORE, target: "_blank", rel: "noopener noreferrer", "zwipe-core" }
                            " alone. Production posture enforced by the compiler."
                        }
                        ul { class: "card-bullets",
                            li {
                                code { ".unwrap" }
                                ", "
                                code { ".expect" }
                                ", "
                                code { "panic!" }
                                ", "
                                code { "todo!" }
                                ", "
                                code { "dbg!" }
                                ", "
                                code { "print!" }
                                ", all denied at compile time"
                            }
                            li { "22 enforced Clippy rules across the workspace" }
                            li {
                                "Compile-time SQL verification via "
                                a { href: URL_SQLX, target: "_blank", rel: "noopener noreferrer", "sqlx" }
                                "'s "
                                code { "query!" }
                                " macro: bad queries fail "
                                code { "cargo build" }
                                ", not runtime"
                            }
                            li { "Commander eligibility, partner validation, deck metrics, all covered" }
                            li { "Security audit complete; nightly Cloudflare R2 backups" }
                        }
                    }

                    div { class: "card",
                        span { class: "card-category", "Auth" }
                        h3 { class: "card-title", "Authentication" }
                        p { class: "card-summary",
                            "Hand-rolled. Probably more security than a deckbuilder needs. Worth it."
                        }
                        ul { class: "card-bullets",
                            li { "Argon2id hashing with a NIST-compliant 170+ pattern password blocklist" }
                            li { "Rotating refresh tokens, replay-safe via delete-on-use" }
                            li { "JWT access tokens, short-lived; refresh tokens stored hashed" }
                            li {
                                code { "Password" }
                                " newtype is "
                                em { "consumed" }
                                " on hash so plaintext can't leak past the boundary"
                            }
                            li { "Rate limiting, audit logs, transactional email" }
                        }
                    }

                    div { class: "card",
                        span { class: "card-category", "Types" }
                        h3 { class: "card-title", "Type Safety" }
                        p { class: "card-summary",
                            "Newtypes everywhere. Invalid states can't compile, let alone reach production."
                        }
                        ul { class: "card-bullets",
                            li {
                                code { "UserId" }
                                ", "
                                code { "DeckId" }
                                ", "
                                code { "Email" }
                                ", "
                                code { "Password" }
                                ": distinct types, not "
                                code { "String" }
                                " aliases"
                            }
                            li { "Builder types enforce required-field rules at construction" }
                            li { "Format eligibility (commander, oathbreaker) modeled as enum + traits, not bool flags" }
                            li { "Newtype wrappers parse-and-validate on the boundary, no defensive checks downstream" }
                        }
                    }

                    div { class: "card",
                        span { class: "card-category", "Sync" }
                        h3 { class: "card-title", "Card Data Pipeline" }
                        p { class: "card-summary",
                            "110k+ printings synced nightly from "
                            a { href: URL_SCRYFALL, target: "_blank", rel: "noopener noreferrer", "Scryfall" }
                            ". The hard parts aren't the cron."
                        }
                        ul { class: "card-bullets",
                            li { "Five-strategy upsert chain: batch first, fall through to per-row on conflict" }
                            li { "88-column rows respect Postgres's 65k-parameter cap: ~327 cards per batch" }
                            li {
                                code { "PartialEq" }
                                " delta detection: only changed rows are written, not the whole catalog nightly"
                            }
                            li { "Materialized view refresh for deduplicated search (~35k unique cards)" }
                            li { "Zero Scryfall dependency at query time: all lookups hit Postgres" }
                        }
                    }

                    div { class: "card",
                        span { class: "card-category", "Enrichment" }
                        h3 { class: "card-title", "Mechanical Categories" }
                        p { class: "card-summary",
                            a { href: URL_SCRYFALL, target: "_blank", rel: "noopener noreferrer", "Scryfall" }
                            " ships raw oracle text. Players think in roles: "
                            em { "ramp, removal, anthem, counterspell" }
                            ". So zwipe classifies every card into one or more of 24 strategic roles."
                        }
                        ul { class: "card-bullets",
                            li {
                                "24 roles (ramp, removal, anthem, tokens, blink, mill, tutor, "
                                em { "…" }
                                "). See "
                                a { href: URL_MECHANICAL_CATEGORY, target: "_blank", rel: "noopener noreferrer", "mechanical_category" }
                            }
                            li {
                                "Multi-label: "
                                em { "Lightning Bolt" }
                                " = burn + removal; "
                                em { "Sol Ring" }
                                " = ramp"
                            }
                            li { "Deterministic heuristic classifier: oracle text + type line, no AI, runs at sync time" }
                            li { "Stored on the card row; filtering hits one Postgres column, not a runtime classifier" }
                        }
                    }
                }
            }
        }
        Footer {}
    }
}
