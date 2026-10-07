use dioxus::{document::eval, prelude::*};
use zwipe_components::{
    BRAND_RESET_JS, COMPONENTS_CSS, Decode, NAV_GLIDE_JS, NavBar, REVEAL_JS, Replay,
    SCROLL_FADE_JS, SITE_CSS, THEMES_CSS, ThemeConfig, ThemePicker, use_theme_wipe,
};

mod api;
mod components;
mod pages;
use pages::{
    About, Android, Changelog, Contribute, Discord, GuidePage, Guides, Home, Ios, NotFound,
    Privacy, Reset, SharedDeck, Verify,
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
    // Catch-all LAST so it only matches when nothing above does. The deploy
    // copies index.html to 404.html, so unknown paths boot the shell and land
    // here instead of the router's raw match-log dump.
    #[route("/:..segments")]
    NotFound { segments: Vec<String> },
}

fn main() {
    // Native only: under `dx serve` this is the server binary and the logo
    // lands in the terminal, like zwiper's does. The wasm client's stdout
    // goes nowhere, so it does not carry the art.
    #[cfg(not(target_arch = "wasm32"))]
    zwipe_core::domain::logo::Zite::print();

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
/// so `/verify/:token`, `/reset/:token` and `/deck/:token` are excluded
/// automatically, which is right: their content is per-token.
///
/// The guides are the exception. `/guides/:slug` is one dynamic route over a
/// fixed set of articles, so `static_routes()` drops all of them and the
/// deploy has nothing to serve: every `/guides/<slug>` fell back to
/// `404.html`, answering a 404 to crawlers while still hydrating fine for
/// anyone reading it. Appending them here prerenders each article at its own
/// path. Slugs come from `GUIDES`, the same source `build.rs` checks its
/// sitemap list against.
// `#[server]` requires an async fn; this one has nothing to await.
#[allow(clippy::unused_async)]
#[server(endpoint = "static_routes")]
async fn static_routes() -> ServerFnResult<Vec<String>> {
    let mut routes: Vec<String> = Route::static_routes()
        .iter()
        .map(ToString::to_string)
        .collect();
    routes.extend(pages::guide_slugs().map(|slug| format!("/guides/{slug}")));
    Ok(routes)
}

#[component]
fn App() -> Element {
    // The picked theme, remembered in localStorage. It starts at the default
    // so the browser's first render matches the prerender, and adopts the
    // stored theme just after mount; the shell's script already put the body
    // on it, so nothing visible changes.
    let theme = zwipe_components::use_persisted_theme("zwipe.theme");
    use_context_provider(|| theme);
    use_context_provider(|| Replay(Signal::new(0u32)));

    // `hydrated` on the document releases the hero's entrance, which the
    // stylesheet holds until the app can run it.
    use_effect(move || {
        spawn(async {
            let _ = eval("document.documentElement.classList.add('hydrated');").await;
        });
    });

    // Track the sticky nav's real height into --nav-height so sticky content
    // (the guide gallery) docks below it even when the nav wraps taller at
    // narrow widths. A body-level ResizeObserver catches every layout change
    // and re-queries the nav fresh, so per-route nav remounts can't hold a
    // stale node. use_future: install exactly once, not per re-render.
    use_future(|| async {
        let _ = eval(
            r#"(() => {
                const set = () => {
                    const nav = document.querySelector('.nav-wrapper');
                    if (nav) document.documentElement.style.setProperty('--nav-height', nav.offsetHeight + 'px');
                };
                new ResizeObserver(set).observe(document.body);
                set();
            })();"#,
        )
        .await;
    });

    // Apply the theme class to <body> so CSS variable lookups (e.g.
    // body { background-color: var(--bg-primary) }) resolve.
    use_effect(move || {
        let class = theme.read().css_class();
        spawn(async move {
            // Swap only the theme class, leaving any other class on the body.
            let _ = eval(&format!(
                "Array.from(document.body.classList).forEach(c => /^theme-.+-(dark|light)$/.test(c) && document.body.classList.remove(c)); document.body.classList.add('{class}');"
            ))
            .await;
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
        // Fonts are self-hosted in public/fonts (see the @font-face block in
        // style.css); preloading the two latin weights starts those fetches
        // before CSS parsing discovers them, closing the fallback-font flash.
        document::Link { rel: "preload", href: "/fonts/jetbrains-mono-latin-400-normal.woff2", r#as: "font", r#type: "font/woff2", crossorigin: "anonymous" }
        document::Link { rel: "preload", href: "/fonts/jetbrains-mono-latin-700-normal.woff2", r#as: "font", r#type: "font/woff2", crossorigin: "anonymous" }
        document::Style { {THEMES_CSS} }
        document::Style { {COMPONENTS_CSS} }
        document::Style { {SITE_CSS} }
        document::Stylesheet { href: STYLE }
        // Scroll reveal for panels below the fold; everything it does is
        // progressive.
        document::Script { {REVEAL_JS} }
        // Nav items pushed by a wider theme label slide over instead of jumping.
        document::Script { {NAV_GLIDE_JS} }
        // Edge fades on sideways scrollers where CSS can't drive them.
        document::Script { {SCROLL_FADE_JS} }
        Router::<Route> {}
    }
}

#[component]
pub fn Nav() -> Element {
    let (theme, shown) = use_theme_wipe(use_context::<Signal<ThemeConfig>>(), "body");
    let mut open = use_signal(|| false);
    let mut replay = use_context::<Replay>().0;
    let mut hovering = use_signal(|| false);
    rsx! {
        NavBar {
            open,
            brand: rsx! {
                Link {
                    to: Route::Home {},
                    class: "nav-brand",
                    onclick: move |_| {
                        open.set(false);
                        // On the home page this runs the entrance again.
                        replay += 1;
                        spawn(async {
                            let _ = eval(BRAND_RESET_JS).await;
                        });
                    },
                    span {
                        class: "nav-logo",
                        onmouseenter: move |_| hovering.set(true),
                        onmouseleave: move |_| hovering.set(false),
                        Decode { text: Z_LOGO, hover: hovering }
                    }
                }
            },
            persistent: rsx! {
                div { class: "nav-stores-persistent", "data-nav-glide": "true",
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
                ThemePicker { theme, shown }
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
                    "Fan Content Policy"
                }
                ". Not approved/endorsed by Wizards. Portions of the materials used are property "
                "of Wizards of the Coast. ©Wizards of the Coast LLC."
            }
        }
    }
}
