use dioxus::prelude::*;
use zwipe::domain::ascii_logo;
use zwiper::components::login::Login;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(Layout)]
    #[route("/login")]
    Login {},
    #[route("/")]
    #[redirect("/", || Route::Login {})]
    Home {},
}

const FAVICON: Asset = asset!("/assets/favicon/favicon.ico");
const APP_ICON: Asset = asset!("/assets/favicon/android-chrome-192x192.png");
const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        Router::<Route> {}
    }
}

#[component]
fn Home() -> Element {
    rsx! {}
}

#[component]
fn Layout() -> Element {
    let ascii_logo = ascii_logo::logo();

    rsx! {
        div { class: "app-layout",
            header { class: "app-header",
                pre { class: "ascii-logo", "{ascii_logo}" }
            }
            main { class: "app-content",
                Outlet::<Route> {}
            }
        }
    }
}
