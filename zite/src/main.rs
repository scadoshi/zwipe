use dioxus::{document::eval, prelude::*};
use zwipe_components::{
    BRAND_RESET_JS, COMPONENTS_CSS, NavBar, THEMES_CSS, ThemeConfig, ThemePicker,
};

mod components;
mod pages;
mod theme_store;
use pages::{
    About, Android, Changelog, Contribute, Discord, GuidePage, Guides, Home, Ios, Privacy, Reset,
    SharedDeck, Verify,
};

// Base URLs + contact points live in zwipe-core's `site` module (shared with
// zwiper and zerver so they can't drift); re-exported so pages keep importing
// them from crate root. Debug builds resolve the URLs to the local dev servers.
pub use zwipe_core::domain::site::{API_BASE, DISCORD_URL, SUPPORT_EMAIL, WEB_BASE};

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
    #[route("/changelog")]
    Changelog {},
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
    // Start at the default so the client's first render matches the server's
    // (localStorage is client-only). Seeding the signal from storage here would
    // desync SSR and hydration: hydration keeps the server DOM (e.g. the theme
    // picker's "Gruvbox" label) and won't reconcile the mismatch, leaving the
    // label stuck on the default while the body themed correctly. Instead we
    // adopt the stored theme just after mount (below).
    let mut theme = use_signal(ThemeConfig::default);
    use_context_provider(|| theme);
    let mut loaded = use_signal(|| false);

    // After hydration, adopt the last-used theme from localStorage. Being a
    // post-hydration state change (not the initial render), this re-renders the
    // picker label too, not just the body class. A brief flash of the default
    // first is expected until the WASM loads.
    use_effect(move || {
        if let Some(stored) = theme_store::load() {
            theme.set(stored);
        }
        loaded.set(true);
    });

    // Apply the theme class to <body> so CSS variable lookups (e.g.
    // body { background-color: var(--bg-primary) }) resolve, and persist the
    // choice for next visit. The `loaded` guard keeps the pre-load default
    // render from clobbering the stored theme before we've read it.
    use_effect(move || {
        let cfg = theme.read().clone();
        if loaded() {
            theme_store::save(&cfg);
        }
        let class = cfg.css_class();
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
        document::Style { {THEMES_CSS} }
        document::Style { {COMPONENTS_CSS} }
        document::Stylesheet { href: STYLE }
        Router::<Route> {}
    }
}

#[component]
pub fn Nav() -> Element {
    let theme: Signal<ThemeConfig> = use_context();
    let mut open = use_signal(|| false);
    rsx! {
        NavBar {
            open,
            brand: rsx! {
                Link {
                    to: Route::Home {},
                    class: "nav-brand",
                    onclick: move |_| {
                        open.set(false);
                        spawn(async {
                            let _ = eval(BRAND_RESET_JS).await;
                        });
                    },
                    span { class: "nav-logo", "{Z_LOGO}" }
                }
            },
            persistent: rsx! {
                div { class: "nav-stores-persistent",
                    a {
                        class: "store-link",
                        href: "https://apps.apple.com/us/app/zwipe-tcg/id6761341603",
                        "App Store ↗"
                    }
                    a {
                        class: "store-link",
                        href: "https://play.google.com/store/apps/details?id=com.scadoshi.zwipe",
                        "Play Store ↗"
                    }
                }
            },
            links: rsx! {
                li {
                    Link { to: Route::Guides {}, onclick: move |_| open.set(false), "Guides" }
                }
                li {
                    Link { to: Route::About {}, onclick: move |_| open.set(false), "About" }
                }
                li {
                    Link { to: Route::Changelog {}, onclick: move |_| open.set(false), "Changelog" }
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
                        onclick: move |_| open.set(false),
                        "App Store ↗"
                    }
                }
                li { class: "nav-link-store",
                    a {
                        class: "store-link",
                        href: "https://play.google.com/store/apps/details?id=com.scadoshi.zwipe",
                        onclick: move |_| open.set(false),
                        "Play Store ↗"
                    }
                }
            },
            trailing: rsx! {
                ThemePicker { theme }
            },
        }
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
