use dioxus::prelude::*;
use crate::{Footer, Nav};

#[component]
pub fn About() -> Element {
    rsx! {
        Nav {}
        div { class: "page content-enter",
            div { class: "about-header section",
                h1 { "Scotty Fermo" }
                p { class: "tagline", "Software developer · scadoshi" }
                div { class: "profile-links",
                    a { class: "profile-link", href: "https://scottyfermo.com", target: "_blank", rel: "noopener noreferrer", "scottyfermo.com ↗" }
                    a { class: "profile-link", href: "https://github.com/scadoshi", target: "_blank", rel: "noopener noreferrer", "GitHub ↗" }
                    a { class: "profile-link", href: "https://x.com/scadoshi", target: "_blank", rel: "noopener noreferrer", "X ↗" }
                    a { class: "profile-link", href: "https://www.linkedin.com/in/scotty-fermo-41a35b141/", target: "_blank", rel: "noopener noreferrer", "LinkedIn ↗" }
                    a { class: "profile-link", href: "https://github.com/scadoshi/zwipe", target: "_blank", rel: "noopener noreferrer", "Zwipe ↗" }
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
                    "A solo project — designed, built, and shipped by one person.
                    The goal: make deck building feel good on mobile. Built mobile-first
                    from the ground up."
                }
            }

            div { class: "section",
                h2 { "The Stack" }
                div { class: "card-grid",
                    div { class: "card",
                        span { class: "card-category", "API" }
                        h3 { class: "card-title",
                            "Backend — "
                            a { href: "https://github.com/scadoshi/zwipe/tree/main/zerver", target: "_blank", rel: "noopener noreferrer", "zerver" }
                        }
                        p { class: "card-summary",
                            "Rust + "
                            a { href: "https://github.com/tokio-rs/axum", target: "_blank", rel: "noopener noreferrer", "axum" }
                            " REST API on PostgreSQL with "
                            a { href: "https://github.com/launchbadge/sqlx", target: "_blank", rel: "noopener noreferrer", "sqlx" }
                            "."
                        }
                        ul { class: "card-bullets",
                            li { "Compile-time query verification via sqlx" }
                            li { "JWT access + rotating refresh tokens, argon2 hashing" }
                            li { "Rate limiting, audit logs, transactional email" }
                            li { "Commander eligibility validation per format" }
                        }
                    }

                    div { class: "card",
                        span { class: "card-category", "Client" }
                        h3 { class: "card-title",
                            "Frontend — "
                            a { href: "https://github.com/scadoshi/zwipe/tree/main/zwiper", target: "_blank", rel: "noopener noreferrer", "zwiper" }
                        }
                        p { class: "card-summary",
                            a { href: "https://dioxuslabs.com", target: "_blank", rel: "noopener noreferrer", "dioxus" }
                            " in Rust — one codebase, three targets."
                        }
                        ul { class: "card-bullets",
                            li { "iOS, Android, and web from the same source" }
                            li { "Custom swipe gesture handling, signals-based reactivity" }
                            li { "Modular filter system with real-time deck metrics" }
                            li { "Command zone display + buy links and price stats" }
                        }
                    }

                    div { class: "card",
                        span { class: "card-category", "Design" }
                        h3 { class: "card-title", "Architecture" }
                        p { class: "card-summary", "Hexagonal ports & adapters throughout." }
                        ul { class: "card-bullets",
                            li {
                                a { href: "https://github.com/scadoshi/zwipe/tree/main/zwipe-core", target: "_blank", rel: "noopener noreferrer", "zwipe-core" }
                                ": shared domain crate, zero framework deps"
                            }
                            li {
                                a { href: "https://github.com/scadoshi/zwipe/tree/main/zerver", target: "_blank", rel: "noopener noreferrer", "zerver" }
                                ", "
                                a { href: "https://github.com/scadoshi/zwipe/tree/main/zwiper", target: "_blank", rel: "noopener noreferrer", "zwiper" }
                                ", and "
                                a { href: "https://github.com/scadoshi/zwipe/tree/main/zite", target: "_blank", rel: "noopener noreferrer", "zite" }
                                " all depend on "
                                a { href: "https://github.com/scadoshi/zwipe/tree/main/zwipe-core", target: "_blank", rel: "noopener noreferrer", "zwipe-core" }
                            }
                            li { "Inbound (HTTP, UI) and outbound (sqlx, API) adapters swappable" }
                        }
                        pre { class: "arch-diagram",
                            "  zwiper ────→ zwipe-core ←──── zerver\n  (mobile)      (domain)         (api)\n                   ↑               │\n                   │          zervice (sync)\n                 zite\n                 (web)"
                        }
                    }

                    div { class: "card",
                        span { class: "card-category", "Sync" }
                        h3 { class: "card-title", "Card Data" }
                        p { class: "card-summary",
                            "110k+ English printings synced nightly from the "
                            a { href: "https://scryfall.com/docs/api", target: "_blank", rel: "noopener noreferrer", "Scryfall API" }
                            "."
                        }
                        ul { class: "card-bullets",
                            li {
                                a { href: "https://github.com/scadoshi/zwipe/tree/main/zerver", target: "_blank", rel: "noopener noreferrer", "zervice" }
                                " background job pulls new sets each night"
                            }
                            li { "Oracle text, colors, type lines stored locally in Postgres" }
                            li { "Zero Scryfall dependency at query time" }
                        }
                    }

                    div { class: "card",
                        span { class: "card-category", "Quality" }
                        h3 { class: "card-title", "Testing" }
                        p { class: "card-summary",
                            "340+ unit tests; 220 in "
                            a { href: "https://github.com/scadoshi/zwipe/tree/main/zwipe-core", target: "_blank", rel: "noopener noreferrer", "zwipe-core" }
                            "'s shared domain alone."
                        }
                        ul { class: "card-bullets",
                            li { "Commander eligibility, partner validation, deck metrics" }
                            li { "Newtypes enforce correctness at construction" }
                            li {
                                code { "UserId" }
                                ", "
                                code { "DeckId" }
                                ", "
                                code { "EmailAddress" }
                                ", "
                                code { "Password" }
                                " — invalid states can't be passed around"
                            }
                        }
                    }

                    div { class: "card",
                        span { class: "card-category", "Ops" }
                        h3 { class: "card-title", "Infrastructure" }
                        p { class: "card-summary", "Self-hosted on a home Ubuntu server." }
                        ul { class: "card-bullets",
                            li { "Cloudflare Tunnel → api.zwipe.net, no public ports" }
                            li { "Self-hosted GitHub Actions runner, deploys on push to main" }
                            li { "systemd auto-restart, automatic migrations on deploy" }
                            li { "Nightly pg_dump → Cloudflare R2, 30-day retention" }
                        }
                    }
                }
            }
        }
        Footer {}
    }
}
