//! The frames screens draw in.

use components::SiteFooter;
use dioxus::prelude::*;
use zwipe_core::domain::auth::models::session::Session;

use crate::inbound::router::Route;
use crate::inbound::shell::top_nav::{PageTitle, PublicNav, Shell};

/// zwipe.net's Community page, which the footer links out to.
const COMMUNITY_URL: &str = "https://zwipe.net/community";

/// zwipe.net's Contribute page, which the footer links out to.
const CONTRIBUTE_URL: &str = "https://zwipe.net/contribute";

/// The layout over the public routes: the public nav over the screen.
#[component]
pub fn PublicFrame() -> Element {
    // Provided here, over both the nav and the screen, so a screen can put
    // its title in the folded nav.
    use_context_provider(|| Signal::new(None::<PageTitle>));
    rsx! {
        div { class: "web-shell web-public",
            PublicNav {}
            main { class: "web-main",
                Outlet::<Route> {}
            }
        }
    }
}

/// The layout over a route open to everyone (Privacy policy): the
/// signed-in nav with a live session, the public nav without one.
#[component]
pub fn OpenFrame() -> Element {
    let session: Signal<Option<Session>> = use_context();
    let signed_in = session.read().as_ref().is_some_and(|s| !s.is_expired());
    if signed_in {
        rsx! { Shell {} }
    } else {
        rsx! { PublicFrame {} }
    }
}

/// A screen under the top nav. `title` names the screen for assistive tech,
/// the browser tab and the folded nav. Below the breakpoint the title and
/// the `?` (`on_hint`) sit centered in the folded nav; on a wide window the
/// `?` sits in the nav's icon row. The screen scrolls as one column, and a
/// wide window ends it with zwipe.net's footer; a swipe screen fills the
/// window, so its footer starts below the fold.
#[component]
pub fn ScreenFrame(
    title: String,
    #[props(default)] on_hint: Option<Callback<()>>,
    children: Element,
) -> Element {
    let page = try_use_context::<Signal<Option<PageTitle>>>();
    let nav_title = title.clone();
    use_effect(use_reactive!(|nav_title, on_hint| {
        if let Some(mut page) = page {
            page.set(Some(PageTitle {
                title: nav_title,
                on_hint,
            }));
        }
    }));
    use_drop(move || {
        if let Some(mut page) = page {
            page.set(None);
        }
    });
    rsx! {
        document::Title { "{title} | Zwipe" }
        div { class: "screen web-screen web-scroll-screen",
            // Under the nav a screen has no header bar; the title stays for
            // assistive tech.
            h1 { class: "sr-only", "{title}" }
            {children}
            WebFooter {}
        }
    }
}

/// zwipe.net's footer at a page's end on a wide window: Privacy policy on
/// the app's own route, Community and Contribute out to zwipe.net.
#[component]
fn WebFooter() -> Element {
    rsx! {
        SiteFooter { class: "web-site-footer".to_owned(),
            Link { to: Route::PrivacyPolicy {}, class: "nav-cta", "Privacy policy" }
            a {
                class: "nav-cta",
                href: COMMUNITY_URL,
                "Community"
            }
            a {
                class: "nav-cta",
                href: CONTRIBUTE_URL,
                "Contribute"
            }
        }
    }
}
