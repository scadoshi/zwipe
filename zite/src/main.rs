//! zite, the zwipe.net site: a Dioxus fullstack app that `dx build --ssg` prerenders to static
//! pages. The per-token pages (shared deck, verify, confirm email, reset) render in the browser.

use ::components::{
    BRAND_RESET_JS, COMPONENTS_CSS, Decode, MAGIC_CSS, NAV_GLIDE_JS, NavBar, NavCtaIcon, REVEAL_JS,
    Replay, SCROLL_FADE_JS, SITE_CSS, THEMES_CSS, ThemeConfig, ThemePicker, use_theme_wipe,
};
use dioxus::prelude::*;
use zwipe_core::domain::user::settings::PipStyle;

mod api;
mod components;
mod pages;
mod pip_store;
use pages::{
    About, Android, Changelog, Community, ConfirmEmail, Contribute, GuidePage, Guides, Home, Ios,
    NotFound, Privacy, Reset, SharedDeck, SharedDeckPage, Verify,
};
// Base URLs + contact points live in zwipe-core's `site` module (shared with
// zwiper and zerver so they can't drift); re-exported so pages keep importing
// them from crate root. Debug builds resolve the URLs to the local dev servers.
pub use zwipe_core::domain::site::{API_BASE, APP_BASE, DISCORD_URL, SUPPORT_EMAIL, WEB_BASE};

const STYLE: Asset = asset!("/assets/style.css");
/// The kit's input rules the card filter's sheet draws with, which zite
/// otherwise does not load.
const FILTER_INPUTS_CSS: &str = include_str!("../assets/filter-inputs.css");
const FAVICON: Asset = asset!("/assets/favicon.ico");
const FAVICON_16: Asset = asset!("/assets/favicon-16x16.png");
const FAVICON_32: Asset = asset!("/assets/favicon-32x32.png");
const APPLE_TOUCH_ICON: Asset = asset!("/assets/icon-180.png");
const MANIFEST: Asset = asset!("/assets/site.webmanifest");
const Z_LOGO: &str = zwipe_core::domain::logo::Z;

/// A store the nav links as an icon pill.
#[derive(Clone, Copy, PartialEq, Debug)]
struct Store {
    /// The link's accessible name and tooltip.
    label: &'static str,
    /// The store page the pill opens.
    href: &'static str,
    /// A filled 24-unit icon path, drawn in the pill's color.
    path: &'static str,
}

/// The nav's store links, left to right.
const STORES: [Store; 2] = [
    Store {
        label: "App Store",
        href: "https://apps.apple.com/us/app/zwipe-tcg/id6761341603",
        path: APPLE,
    },
    Store {
        label: "Play Store",
        href: "https://play.google.com/store/apps/details?id=com.scadoshi.zwipe",
        path: GOOGLE_PLAY,
    },
];

/// The Apple mark (Simple Icons).
const APPLE: &str = "M12.152 6.896c-.948 0-2.415-1.078-3.96-1.04-2.04.027-3.91 1.183-4.961 3.014-2.117 3.675-.546 9.103 1.519 12.09 1.013 1.454 2.208 3.09 3.792 3.039 1.52-.065 2.09-.987 3.935-.987 1.831 0 2.35.987 3.96.948 1.637-.026 2.676-1.48 3.676-2.948 1.156-1.688 1.636-3.325 1.662-3.415-.039-.013-3.182-1.221-3.22-4.857-.026-3.04 2.48-4.494 2.597-4.559-1.429-2.09-3.623-2.324-4.39-2.376-2-.156-3.675 1.09-4.61 1.09zM15.53 3.83c.843-1.012 1.4-2.427 1.245-3.83-1.207.052-2.662.805-3.532 1.818-.78.896-1.454 2.338-1.273 3.714 1.338.104 2.715-.688 3.559-1.701";

/// The Google Play mark (Simple Icons).
const GOOGLE_PLAY: &str = "M3.609 1.814L13.792 12 3.61 22.186a.996.996 0 0 1-.61-.92V2.734a1 1 0 0 1 .609-.92zm10.89 10.893l2.302 2.302-10.937 6.333 8.635-8.635zm3.199-3.198l2.807 1.626a1 1 0 0 1 0 1.73l-2.808 1.626L15.206 12l2.492-2.491zM5.864 2.658L16.8 8.99l-2.302 2.302-8.634-8.634z";

#[derive(Routable, Clone, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[route("/")]
    Home {},
    #[route("/guides/")]
    Guides {},
    #[route("/guides/:slug/")]
    GuidePage { slug: String },
    // The Playstyle guide's first slug, linked before the rename.
    #[redirect("/guides/suggested-commanders", || Route::GuidePage { slug: "playstyle".to_string() })]
    #[route("/about/")]
    About {},
    #[route("/changelog/")]
    Changelog {},
    #[route("/contribute/")]
    Contribute {},
    #[route("/community/")]
    Community {},
    #[redirect("/discord", || Route::Community {})]
    #[route("/download/android/")]
    Android {},
    #[route("/download/ios/")]
    Ios {},
    #[route("/privacy/")]
    Privacy {},
    // A shared deck, GitHub style. Only the token identifies it; the page
    // swaps a stale owner or slug for the current ones. The four-segment
    // shape is the only shared-deck form, clear of the one- and two-segment
    // pages above.
    #[route("/:owner/deck/:slug/:token")]
    SharedDeckPage { owner: String, slug: String, token: String },
    // The links shipped apps share: the same page, which then moves to the
    // owner, slug and token form.
    #[route("/deck/:token")]
    SharedDeck { token: String },
    #[route("/verify/:token")]
    Verify { token: String },
    #[route("/confirm-email/:token")]
    ConfirmEmail { token: String },
    #[route("/reset/:token")]
    Reset { token: String },
    // Catch-all LAST so it only matches when nothing above does. The deploy
    // copies index.html to 404.html, so unknown paths boot the shell and land
    // here instead of the router's raw match-log dump.
    #[route("/:..segments")]
    NotFound { segments: Vec<String> },
}

#[cfg_attr(
    not(all(feature = "server", not(debug_assertions))),
    expect(
        clippy::unnecessary_wraps,
        reason = "only the release server build can fail, when it looks for its prerender folder"
    )
)]
fn main() -> std::io::Result<()> {
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
                        .static_dir(prerender_dir()?)
                        .clear_cache(false)
                );
                config.enable_out_of_order_streaming()
            }
        })
        .launch(App);
    Ok(())
}

/// Where `dx build --ssg` writes the prerendered pages: `public/` beside the
/// server binary.
///
/// # Errors
///
/// Fails when the binary's own path can't be read or has no parent directory.
#[cfg(all(feature = "server", not(debug_assertions)))]
fn prerender_dir() -> std::io::Result<std::path::PathBuf> {
    let exe = std::env::current_exe()?;
    let dir = exe
        .parent()
        .ok_or_else(|| std::io::Error::other("the server binary's path has no parent directory"))?;
    Ok(dir.join("public"))
}

/// Endpoint hit by `dx build --ssg` to enumerate routes to prerender.
/// `Route::static_routes()` returns every route with no dynamic segments,
/// so `/verify/:token`, `/confirm-email/:token`, `/reset/:token` and `/deck/:token` are excluded
/// automatically, which is right: their content is per-token.
///
/// The guides are the exception. `/guides/:slug` is one dynamic route over a
/// fixed set of articles, so `static_routes()` drops all of them, and an
/// unprerendered path is served by `404.html` with a 404 status. Appending
/// them here prerenders each article at its own path. Slugs come from `GUIDES`, the same source `build.rs` checks its
/// sitemap list against.
#[expect(
    clippy::unused_async,
    reason = "`#[server]` requires an async fn; this one has nothing to await"
)]
#[server(endpoint = "static_routes")]
async fn static_routes() -> ServerFnResult<Vec<String>> {
    let mut routes: Vec<String> = Route::static_routes()
        .iter()
        .map(ToString::to_string)
        .collect();
    routes.extend(pages::guide_slugs().map(|slug| format!("/guides/{slug}/")));
    Ok(routes)
}

/// Whether this load's splash is still to come or playing. The home page
/// draws it; it goes false once the splash has faded, or at once on a load
/// that skips it, so coming back to the home page never plays it again.
#[derive(Debug, Clone, Copy)]
pub struct SiteSplash(pub Signal<bool>);

/// Whether the page shows cards, and so pips: true while a shared deck
/// with any card is on screen. The nav's pip toggle shows only then.
#[derive(Debug, Clone, Copy)]
pub struct ShowsPips(pub Signal<bool>);

/// Sends `true` on every Escape, for the root to close the top sheet. While
/// a kit dialog is open its own listener takes Escape, so this one steps
/// aside.
const ESCAPE_JS: &str = "document.addEventListener('keydown', (e) => { if (e.key === 'Escape' && !document.querySelector('.alert-dialog[data-state=\"open\"]')) dioxus.send(true); });";

/// Sends `true` on every Escape, for the nav to close its phone menu. Every
/// page mounts its own nav, so the listener removes the one before it.
const MENU_ESCAPE_JS: &str = "window.__zwipeMenuEscapeOff?.(); const onKey = (e) => { if (e.key === 'Escape') dioxus.send(true); }; document.addEventListener('keydown', onKey); window.__zwipeMenuEscapeOff = () => document.removeEventListener('keydown', onKey);";

/// Puts focus back on the phone menu's toggle once Escape closes the menu.
const MENU_TOGGLE_FOCUS_JS: &str = "document.querySelector('.nav-toggle')?.focus();";

#[component]
fn App() -> Element {
    // The picked theme, remembered in localStorage. It starts at the default
    // so the browser's first render matches the prerender, and adopts the
    // stored theme just after mount; the shell's script already put the body
    // on it, so nothing visible changes.
    let theme = ::components::use_persisted_theme("zwipe.theme");
    use_context_provider(|| theme);
    // The pip style, kept the same way under its own key.
    let pip_style = pip_store::use_persisted_pip_style();
    use_context_provider(|| pip_style);
    use_context_provider(|| ShowsPips(Signal::new(false)));
    use_context_provider(|| Replay(Signal::new(0u32)));
    // A card drawn without its image shrinks its text to fit its box.
    ::components::use_card_proxy_fit();
    // Lists marked to settle, as Guides' cards under a filter, ease into place.
    ::components::use_settle();
    // Sheets register here, so Escape closes the top one.
    let mut overlay_back_stack = ::components::use_overlay_back_stack();
    use_context_provider(|| overlay_back_stack);
    // Dialogs, such as the featured card's store dialog, draw at the root.
    let dialogs = ::components::use_dialog_host();
    use_context_provider(|| dialogs);
    use_effect(move || {
        spawn(async move {
            let mut escapes = document::eval(ESCAPE_JS);
            while escapes.recv::<bool>().await.is_ok() {
                overlay_back_stack.close_top();
            }
        });
    });
    // Closed until the splash starts to fade, so the hero's entrances run as
    // it leaves. index.html decides before first paint whether this load gets
    // the splash (the first landing on the home page in a session) and marks
    // the document `splash-skip` otherwise, which hides the prerendered
    // overlay; read here, it opens the gate at once and stands the splash down.
    let mut gate = use_context_provider(|| ::components::SplashGate(Signal::new(false))).0;
    let mut splash = use_context_provider(|| SiteSplash(Signal::new(true))).0;
    use_effect(move || {
        spawn(async move {
            let mut eval = document::eval(
                "const root = document.documentElement; root.classList.add('splash-live'); dioxus.send(root.classList.contains('splash-skip'));",
            );
            if eval.recv::<bool>().await.unwrap_or(true) {
                splash.set(false);
                gate.set(true);
            }
        });
    });

    // `hydrated` on the document releases the hero's entrance, which the
    // stylesheet holds until the app can run it.
    use_effect(move || {
        spawn(async {
            let _ = document::eval("document.documentElement.classList.add('hydrated');").await;
        });
    });

    // Track the sticky nav's real height into --nav-height so sticky content
    // (the guide gallery) docks below it even when the nav wraps taller at
    // narrow widths. A body-level ResizeObserver catches every layout change
    // and re-queries the nav fresh, so per-route nav remounts can't hold a
    // stale node. use_future: install exactly once, not per re-render.
    use_future(|| async {
        let _ = document::eval(
            r"(() => {
                const set = () => {
                    const nav = document.querySelector('.nav-wrapper');
                    if (nav) document.documentElement.style.setProperty('--nav-height', nav.offsetHeight + 'px');
                };
                new ResizeObserver(set).observe(document.body);
                set();
            })();",
        )
        .await;
    });

    // Apply the theme class to <body> so CSS variable lookups (e.g.
    // body { background-color: var(--bg-primary) }) resolve.
    use_effect(move || {
        let class = theme.read().css_class();
        spawn(async move {
            // Swap only the theme class, leaving any other class on the body.
            let _ = document::eval(&format!(
                "Array.from(document.body.classList).forEach(c => /^theme-.+-(dark|light)$/.test(c) && document.body.classList.remove(c)); document.body.classList.add('{class}');"
            ))
            .await;
        });
    });

    // The pip style's class sits on <body> beside the theme's, where magic.css
    // looks for it.
    use_effect(move || {
        let classic = pip_style() == PipStyle::Classic;
        let class = ::components::pip_style_class(PipStyle::Classic);
        spawn(async move {
            let _ = document::eval(&format!(
                "document.body.classList.toggle('{class}', {classic});"
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
        // The kit's dialogs, without the app shell.
        document::Style { {::components::DIALOG_CSS} }
        document::Style { {MAGIC_CSS} }
        document::Style { {SITE_CSS} }
        // The card filter on a shared deck: the kit's input rules it needs,
        // scoped to its sheet, then its own.
        document::Style { {FILTER_INPUTS_CSS} }
        document::Style { {::components::FILTER_CSS} }
        document::Stylesheet { href: STYLE }
        // Scroll reveal for panels below the fold; everything it does is
        // progressive.
        document::Script { {REVEAL_JS} }
        // Nav items pushed by a wider theme label slide over instead of jumping.
        document::Script { {NAV_GLIDE_JS} }
        // Edge fades on sideways scrollers where CSS can't drive them.
        document::Script { {SCROLL_FADE_JS} }
        // The backdrop, still: drawn in place on first paint. The splash's own
        // field rains in over it, the same seeded cells, so when the overlay
        // fades the backdrop is already there and nothing rains twice.
        ::components::GridCells { still: true }
        Router::<Route> {}
        ::components::DialogHostView {}
    }
}

/// The site's top bar: the brand, the page links, the store buttons and the theme controls.
#[component]
pub fn Nav() -> Element {
    let (theme, shown) = use_theme_wipe(use_context::<Signal<ThemeConfig>>(), "body");
    let pip_style = use_context::<Signal<PipStyle>>();
    // The pip style only shows on a shared deck with cards, the one page
    // that draws pips.
    let on_shared_deck = matches!(
        use_route::<Route>(),
        Route::SharedDeck { .. } | Route::SharedDeckPage { .. }
    );
    let shows_pips = use_context::<ShowsPips>().0;
    let mut open = use_signal(|| false);
    // Escape closes the phone menu and gives focus back to its toggle.
    use_future(move || async move {
        let mut keys = document::eval(MENU_ESCAPE_JS);
        while keys.recv::<bool>().await.is_ok() {
            if *open.peek() {
                open.set(false);
                let _ = document::eval(MENU_TOGGLE_FOCUS_JS).await;
            }
        }
    });
    let mut replay = use_context::<Replay>().0;
    let mut hovering = use_signal(|| false);
    rsx! {
        // First in the tab order, shown only while focused.
        a { class: "skip-link", href: "#content", "Skip to content" }
        NavBar {
            open,
            brand: rsx! {
                Link {
                    to: Route::Home {},
                    class: "nav-brand",
                    aria_label: "Zwipe",
                    onclick: move |_| {
                        open.set(false);
                        // On the home page this runs the entrance again.
                        replay += 1;
                        spawn(async {
                            let _ = document::eval(BRAND_RESET_JS).await;
                        });
                    },
                    span {
                        class: "nav-logo",
                        "aria-hidden": "true",
                        onmouseenter: move |_| hovering.set(true),
                        onmouseleave: move |_| hovering.set(false),
                        Decode { text: Z_LOGO, hover: hovering }
                    }
                }
            },
            persistent: rsx! {
                for cta in STORES {
                    NavCtaIcon {
                        key: "{cta.label}",
                        href: cta.href,
                        label: cta.label,
                        path: cta.path,
                    }
                }
                a { class: "nav-cta", href: "{APP_BASE}/login", "Log in" }
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
                    Link { to: Route::Community {}, onclick: move |_| open.set(false), "Community" }
                }
                // The store pills hide in the collapsed panel on mobile,
                // where the persistent copies beside the toggle cover them.
                for cta in STORES {
                    li { key: "{cta.label}", class: "nav-cta-item",
                        NavCtaIcon {
                            href: cta.href,
                            label: cta.label,
                            path: cta.path,
                            onclick: move |_| open.set(false),
                        }
                    }
                }
                li { class: "nav-cta-item",
                    a { class: "nav-cta", href: "{APP_BASE}/login", onclick: move |_| open.set(false), "Log in" }
                }
            },
            trailing: rsx! {
                div { class: "theme-controls",
                    ThemePicker { theme, shown }
                    if on_shared_deck && shows_pips() {
                        components::PipToggle { style: pip_style }
                    }
                }
            },
        }
    }
}

/// The site footer: the shared [`SiteFooter`](fn@::components::SiteFooter) with Privacy policy, Community and
/// Contribute as its pills.
#[component]
pub fn Footer() -> Element {
    rsx! {
        ::components::SiteFooter {
            Link { to: Route::Privacy {}, class: "nav-cta", "Privacy policy" }
            Link { to: Route::Community {}, class: "nav-cta", "Community" }
            Link { to: Route::Contribute {}, class: "nav-cta", "Contribute" }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_old_suggested_commanders_slug_redirects_to_playstyle() {
        for path in [
            "/guides/suggested-commanders",
            "/guides/suggested-commanders/",
        ] {
            let route: Route = path.parse().unwrap();
            assert!(
                route
                    == Route::GuidePage {
                        slug: "playstyle".to_string()
                    },
                "{path} went to {route}"
            );
        }
    }

    #[test]
    fn prerendered_pages_link_to_the_slashed_path_the_host_serves() {
        assert_eq!(Route::Guides {}.to_string(), "/guides/");
        assert_eq!(
            Route::GuidePage {
                slug: "filtering".to_string()
            }
            .to_string(),
            "/guides/filtering/"
        );
        assert_eq!(Route::Home {}.to_string(), "/");
        let route: Route = "/about".parse().unwrap();
        assert!(route == Route::About {}, "/about went to {route}");
    }

    #[test]
    fn the_first_paint_script_retires_the_themes_orangui_retires() {
        let html = include_str!("../index.html");
        for (from, to) in ::components::RETIRED_THEMES {
            assert!(
                html.contains(&format!("\"{from}\": \"{to}\"")),
                "index.html's retired map lacks {from} -> {to}"
            );
        }
    }
}
