use dioxus::document::eval;
use dioxus::prelude::*;
use zwipe_core::domain::user::models::theme::ThemeConfig;
use zwipe_core::domain::user::preferences::ALLOWED_THEMES;

mod pages;
use pages::{About, Android, Contribute, Discord, Home, Ios, Privacy, Reset, Verify};

pub const API_BASE: &str = "https://api.zwipe.net";

const THEMES: Asset = asset!("/assets/themes.css");
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
    #[route("/about")]
    About {},
    #[route("/contribute")]
    Contribute {},
    #[route("/discord")]
    Discord {},
    #[route("/download/ios")]
    Ios {},
    #[route("/download/android")]
    Android {},
    #[route("/privacy")]
    Privacy {},
    #[route("/verify/:token")]
    Verify { token: String },
    #[route("/reset/:token")]
    Reset { token: String },
}

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let theme = use_signal(ThemeConfig::default);
    use_context_provider(|| theme);

    // Apply theme class to <body> so all CSS variable lookups — including
    // body { background-color: var(--bg-primary) } — resolve from the theme.
    use_effect(move || {
        let class = theme.read().css_class();
        spawn(async move {
            let _ = eval(&format!(
                "document.body.className = '{class}';"
            )).await;
        });
    });

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "icon", r#type: "image/png", sizes: "16x16", href: FAVICON_16 }
        document::Link { rel: "icon", r#type: "image/png", sizes: "32x32", href: FAVICON_32 }
        document::Link { rel: "apple-touch-icon", href: APPLE_TOUCH_ICON }
        document::Link { rel: "manifest", href: MANIFEST }
        document::Stylesheet { href: THEMES }
        document::Stylesheet { href: STYLE }
        Router::<Route> {}
    }
}

#[component]
pub fn Nav() -> Element {
    rsx! {
        div { class: "nav-wrapper",
        nav {
            Link {
                to: Route::Home {},
                class: "nav-brand",
                onclick: move |_| {
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
            ul { class: "nav-links",
                li {
                    Link { to: Route::About {}, "about" }
                }
                li {
                    Link { to: Route::Contribute {}, "contribute" }
                }
                li {
                    Link { to: Route::Discord {}, "discord" }
                }
                li {
                    Link { to: Route::Ios {}, class: "store-link", "app store ↗" }
                }
                li {
                    Link { to: Route::Android {}, class: "store-link", "play store ↗" }
                }
            }
        }
        } // nav-wrapper
    }
}

#[component]
pub fn ThemePicker() -> Element {
    let mut theme: Signal<ThemeConfig> = use_context();
    let current = theme.read().name.clone();
    let is_dark = theme.read().is_dark;
    let has_light = theme.read().has_light_mode();

    rsx! {
        div { class: "theme-picker",
            for name in ALLOWED_THEMES {
                button {
                    class: if current == *name { "theme-btn selected" } else { "theme-btn" },
                    onclick: move |_| {
                        let dark = theme.read().is_dark;
                        theme.set(ThemeConfig {
                            name: name.to_string(),
                            is_dark: dark,
                        });
                    },
                    "{name}"
                }
            }
        }
        if has_light {
            div { class: "theme-toggle",
                button {
                    class: "theme-btn",
                    onclick: move |_| {
                        let current = theme.read().clone();
                        theme.set(ThemeConfig {
                            name: current.name,
                            is_dark: !current.is_dark,
                        });
                    },
                    if is_dark { "light mode" } else { "dark mode" }
                }
            }
        }
    }
}

#[component]
pub fn Footer() -> Element {
    rsx! {
        footer {
            ThemePicker {}
            p { "© 2026 scadoshi · "
                Link { to: Route::Privacy {}, "privacy policy" }
            }
        }
    }
}
