use dioxus::prelude::*;

mod pages;
use pages::{About, Contribute, Home, Privacy, Reset, Verify};

pub const API_BASE: &str = "https://api.zwipe.net";

// App Store URL — update once live on the App Store
pub const APP_STORE_URL: &str = "#";

const STYLE: Asset = asset!("/assets/style.css");

#[derive(Routable, Clone, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[route("/")]
    Home {},
    #[route("/about")]
    About {},
    #[route("/contribute")]
    Contribute {},
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

fn App() -> Element {
    rsx! {
        document::Stylesheet { href: STYLE }
        Router::<Route> {}
    }
}

#[component]
pub fn Nav() -> Element {
    rsx! {
        nav {
            Link { to: Route::Home {}, class: "nav-brand",
                span { "zwipe" }
            }
            ul { class: "nav-links",
                li { class: "hide-mobile",
                    Link { to: Route::Home {}, "home" }
                }
                li {
                    Link { to: Route::About {}, "about" }
                }
                li {
                    Link { to: Route::Contribute {}, "contribute" }
                }
                li {
                    a { href: APP_STORE_URL, class: "appstore-link", "app store ↗" }
                }
            }
        }
    }
}

#[component]
pub fn Footer() -> Element {
    rsx! {
        footer {
            p { "© 2026 scadoshi · "
                Link { to: Route::Privacy {}, "privacy policy" }
            }
        }
    }
}
