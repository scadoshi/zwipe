//! The frames screens draw in.

use components::SiteFooter;
use dioxus::prelude::*;
use zwipe_core::domain::auth::models::session::Session;

use crate::inbound::chrome::shortcuts_sheet::{
    ShortcutsSheet, use_global_keys, use_shortcuts_root,
};
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

/// The layout over every route but the auth screens: the signed-in nav
/// with a live session, so the one [`Shell`] stays mounted from screen to
/// screen. Without one, a route open to everyone (Privacy policy, About,
/// Not found) gets the public nav, and an authed route draws nothing while
/// its gate sends the user to Login. The keyboard shortcuts sheet sits over
/// all of them, opened by Shift+H or a swipe screen's dock button.
#[component]
pub fn OpenFrame() -> Element {
    let session: Signal<Option<Session>> = use_context();
    let route: Route = use_route();
    let shortcuts_open = use_shortcuts_root();
    let signed_in = session.read().as_ref().is_some_and(|s| !s.is_expired());
    rsx! {
        if signed_in {
            Shell {}
        } else if is_open(&route) {
            PublicFrame {}
        } else {
            Outlet::<Route> {}
        }
        ShortcutsSheet { open: shortcuts_open }
    }
}

/// Whether `route` is open to everyone, so a signed-out visit draws it.
fn is_open(route: &Route) -> bool {
    matches!(
        route,
        Route::PrivacyPolicy {} | Route::About {} | Route::NotFound { .. }
    )
}

/// A screen under the top nav. `title` names the screen for assistive tech,
/// the browser tab and the folded nav. Below the breakpoint the title and
/// the `?` (`on_hint`) sit centered in the folded nav; on a wide window the
/// `?` sits in the nav's icon row. The screen scrolls as one column, and a
/// wide window ends it with zwipe.net's footer; a swipe screen fills the
/// window, so its footer starts below the fold. `titled` says the screen
/// draws its own heading with the title, so the frame's screen-reader one
/// stays out.
#[component]
pub fn ScreenFrame(
    title: String,
    #[props(default)] on_hint: Option<Callback<()>>,
    #[props(default)] titled: bool,
    children: Element,
) -> Element {
    let page = try_use_context::<Signal<Option<PageTitle>>>();
    // `?` opens the same hint the `?` button does.
    use_global_keys(on_hint);
    let nav_title = title.clone();
    use_effect(use_reactive!(|nav_title, on_hint| {
        if let Some(mut page) = page {
            page.set(Some(PageTitle {
                title: nav_title,
                on_hint,
                on_back: None,
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
            if !titled {
                h1 { class: "sr-only", "{title}" }
            }
            {children}
            WebFooter {}
        }
    }
}

/// zwipe.net's footer at a page's end: Privacy policy on the app's own
/// route, Community and Contribute out to zwipe.net. [`ScreenFrame`] ends
/// every screen with it, and a full-window overlay that scrolls as a screen
/// does (swipe select) ends with it too.
#[component]
pub(crate) fn WebFooter() -> Element {
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
