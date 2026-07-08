use dioxus::{document::eval, prelude::*};
use zwipe_components::{ThemeConfig, ThemePicker};

mod components;
mod pages;
use pages::{
    About, Android, Contribute, Discord, GuidePage, Guides, Home, Ios, Privacy, Reset, SharedDeck,
    Verify,
};

/// Debug builds (dx serve) talk to a local dev zerver so pages that fetch
/// (shared deck, verify, reset) are testable without touching prod.
#[cfg(debug_assertions)]
pub const API_BASE: &str = "http://localhost:3000";
#[cfg(not(debug_assertions))]
pub const API_BASE: &str = "https://api.zwipe.net";

/// Public web base URL, centralized so a domain change is a single edit.
pub const WEB_BASE: &str = "https://zwipe.net";

/// User-facing support email address.
pub const SUPPORT_EMAIL: &str = "support@zwipe.net";

/// Discord community invite link.
pub const DISCORD_URL: &str = "https://discord.gg/s2UReqUUeg";

const THEMES: Asset = asset!("/assets/themes.css");
const COMPONENTS: Asset = asset!("/assets/components.css");
const STYLE: Asset = asset!("/assets/style.css");
const FAVICON: Asset = asset!("/assets/favicon.ico");
const FAVICON_16: Asset = asset!("/assets/favicon-16x16.png");
const FAVICON_32: Asset = asset!("/assets/favicon-32x32.png");
const APPLE_TOUCH_ICON: Asset = asset!("/assets/icon-180.png");
const MANIFEST: Asset = asset!("/assets/site.webmanifest");
const Z_LOGO: &str = zwipe_core::domain::logo::Z;

#[derive(Routable, Clone, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[route("/")]
    Home {},
    #[route("/guides")]
    Guides {},
    #[route("/guides/:slug")]
    GuidePage { slug: String },
    #[route("/about")]
    About {},
    #[route("/contribute")]
    Contribute {},
    #[route("/discord")]
    Discord {},
    #[route("/download/android")]
    Android {},
    #[route("/download/ios")]
    Ios {},
    #[route("/privacy")]
    Privacy {},
    #[route("/deck/:token")]
    SharedDeck { token: String },
    #[route("/verify/:token")]
    Verify { token: String },
    #[route("/reset/:token")]
    Reset { token: String },
}

fn main() {
    dioxus::LaunchBuilder::new()
        .with_cfg(server_only! {
            {
                let config = dioxus::server::ServeConfig::builder();
                // The incremental cache exists for `dx build --ssg` (release):
                // it snapshots each prerendered route to public/<route>/. Under
                // `dx serve` it instead caches dynamic routes to disk on first
                // hit and 404s cache misses (with a broken trailing-slash
                // redirect on hits), so debug builds skip it and SSR every
                // request fresh.
                #[cfg(not(debug_assertions))]
                let config = config.incremental(
                    dioxus::server::IncrementalRendererConfig::new()
                        .static_dir(
                            std::env::current_exe()
                                .unwrap()
                                .parent()
                                .unwrap()
                                .join("public")
                        )
                        .clear_cache(false)
                );
                config.enable_out_of_order_streaming()
            }
        })
        .launch(App);
}

/// Endpoint hit by `dx build --ssg` to enumerate routes to prerender.
/// `Route::static_routes()` returns every route with no dynamic segments,
/// so `/verify/:token` and `/reset/:token` are excluded automatically.
#[server(endpoint = "static_routes")]
async fn static_routes() -> ServerFnResult<Vec<String>> {
    Ok(Route::static_routes()
        .iter()
        .map(ToString::to_string)
        .collect())
}

#[component]
fn App() -> Element {
    let theme = use_signal(ThemeConfig::default);
    use_context_provider(|| theme);

    // Apply theme class to <body> so all CSS variable lookups, including
    // body { background-color: var(--bg-primary) }, resolve from the theme.
    use_effect(move || {
        let class = theme.read().css_class();
        spawn(async move {
            let _ = eval(&format!("document.body.className = '{class}';")).await;
        });
    });

    rsx! {
        document::Meta { name: "viewport", content: "width=device-width, initial-scale=1, viewport-fit=cover" }
        // Tells Dark Reader to leave the site alone: theming is first-class
        // here (user-picked theme + dark/light), and Dark Reader's dynamic
        // mode mangles the color-mix()/var() palette into monochrome.
        document::Meta { name: "darkreader-lock" }
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "icon", r#type: "image/png", sizes: "16x16", href: FAVICON_16 }
        document::Link { rel: "icon", r#type: "image/png", sizes: "32x32", href: FAVICON_32 }
        document::Link { rel: "apple-touch-icon", href: APPLE_TOUCH_ICON }
        document::Link { rel: "manifest", href: MANIFEST }
        document::Stylesheet { href: THEMES }
        document::Stylesheet { href: COMPONENTS }
        document::Stylesheet { href: STYLE }
        Router::<Route> {}
    }
}

#[component]
pub fn Nav() -> Element {
    let theme: Signal<ThemeConfig> = use_context();
    let mut open = use_signal(|| false);
    let panel_class = if open() {
        "nav-panel nav-panel-open"
    } else {
        "nav-panel"
    };
    let toggle_class = if open() {
        "nav-toggle nav-toggle-open"
    } else {
        "nav-toggle"
    };
    rsx! {
        div { class: "nav-wrapper",
        nav {
            Link {
                to: Route::Home {},
                class: "nav-brand",
                onclick: move |_| {
                    open.set(false);
                    spawn(async {
                        let _ = eval(r#"
                            window.scrollTo({ top: 0, behavior: 'smooth' });
                            const el = document.querySelector('.logo');
                            if (el) {
                                el.style.animation = 'none';
                                void el.offsetHeight;
                                el.style.animation = '';
                            }
                        "#).await;
                    });
                },
                span { class: "nav-logo", "{Z_LOGO}" }
            }
            div { class: "nav-stores-persistent",
                a {
                    class: "store-link",
                    href: "https://apps.apple.com/us/app/zwipe-tcg/id6761341603",
                    target: "_blank",
                    rel: "noopener",
                    "App Store ↗"
                }
                Link { to: Route::Android {}, class: "store-link", "Play Store ↗" }
            }
            button {
                class: "{toggle_class}",
                aria_label: "Toggle navigation menu",
                aria_expanded: "{open()}",
                onclick: move |_| {
                    let next = !open();
                    open.set(next);
                },
                span { class: "nav-toggle-bar" }
                span { class: "nav-toggle-bar" }
                span { class: "nav-toggle-bar" }
            }
            div { class: "{panel_class}",
                div { class: "nav-panel-inner",
                ul { class: "nav-links",
                    li {
                        Link { to: Route::Guides {}, onclick: move |_| open.set(false), "Guides" }
                    }
                    li {
                        Link { to: Route::About {}, onclick: move |_| open.set(false), "About" }
                    }
                    li {
                        Link { to: Route::Contribute {}, onclick: move |_| open.set(false), "Contribute" }
                    }
                    li {
                        Link { to: Route::Discord {}, onclick: move |_| open.set(false), "Discord" }
                    }
                    li { class: "nav-link-store",
                        a {
                            class: "store-link",
                            href: "https://apps.apple.com/us/app/zwipe-tcg/id6761341603",
                            target: "_blank",
                            rel: "noopener",
                            onclick: move |_| open.set(false),
                            "App Store ↗"
                        }
                    }
                    li { class: "nav-link-store",
                        Link { to: Route::Android {}, class: "store-link", onclick: move |_| open.set(false), "Play Store ↗" }
                    }
                }
                ThemePicker { theme }
                } // nav-panel-inner
            }
        }
        } // nav-wrapper
    }
}

#[component]
pub fn Footer() -> Element {
    rsx! {
        footer {
            p { "© 2026 scadoshi | "
                Link { to: Route::Privacy {}, "Privacy Policy" }
            }
            p { class: "fan-content-notice",
                "Zwipe is unofficial Fan Content permitted under the "
                a {
                    href: "https://company.wizards.com/en/legal/fancontentpolicy",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    "Fan Content Policy"
                }
                ". Not approved/endorsed by Wizards. Portions of the materials used are property "
                "of Wizards of the Coast. ©Wizards of the Coast LLC."
            }
        }
    }
}
