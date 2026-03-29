use dioxus::document::eval;
use dioxus::prelude::*;

mod pages;
use pages::{About, Contribute, Download, Home, Privacy, Reset, Verify};

pub const API_BASE: &str = "https://api.zwipe.net";

const STYLE: Asset = asset!("/assets/style.css");
const Z_LOGO: &str = include_str!("../assets/z.txt");

#[derive(Routable, Clone, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[route("/")]
    Home {},
    #[route("/about")]
    About {},
    #[route("/contribute")]
    Contribute {},
    #[route("/download")]
    Download {},
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
                    Link { to: Route::Download {}, class: "appstore-link", "app store ↗" }
                }
            }
        }
        } // nav-wrapper
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
