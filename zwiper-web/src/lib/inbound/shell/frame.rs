//! The frames screens draw in.

use components::{InfoButton, SiteFooter};
use dioxus::prelude::*;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::user::models::hints::OneTimeHint;
use zwipe_ui::hint::one_time::content_for;

use crate::inbound::router::Route;
use crate::inbound::shell::top_nav::{PageTitle, PublicNav, Shell};

/// zwipe.net's Community page, which the footer links out to.
const COMMUNITY_URL: &str = "https://zwipe.net/community";

/// zwipe.net's Contribute page, which the footer links out to.
const CONTRIBUTE_URL: &str = "https://zwipe.net/contribute";

/// A screen's page-head description: the first line of its one-time hint,
/// the sentence the app opens that hint with.
#[must_use]
pub fn page_description(hint: OneTimeHint) -> Option<String> {
    content_for(hint).and_then(|content| content.blocks.first().map(zwipe_ui::hint::Block::plain))
}

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
/// the browser tab and the folded nav. The screen scrolls as one column
/// with no bar at its end: its actions float in a dock or sit inline, and
/// Back is the folded nav's arrow. Help is the menu's `!`. There is no site
/// footer.
///
/// Where a screen's page head goes on a wide window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HeadPlace {
    /// Over the page, the full width, before the screen's content.
    #[default]
    Page,
    /// Drawn by the screen itself, as a [`PageHead`] inside its content,
    /// so a panel can share its row.
    Content,
}

/// The page head, zite's page header: a panel with the title, the `?` at
/// its right end and the line under a rule. The frame draws it over the
/// page; a screen with [`HeadPlace::Content`] draws it in a column.
#[component]
pub fn PageHead(
    title: String,
    line: String,
    #[props(default)] on_hint: Option<Callback<()>>,
) -> Element {
    rsx! {
        div { class: "web-form-panel web-page-head",
            div { class: "panel-head web-panel-head web-page-head-bar",
                h2 { class: "panel-title", "{title}" }
                if let Some(open) = on_hint {
                    InfoButton { onclick: move |_| open.call(()) }
                }
            }
            hr { class: "panel-rule web-panel-head" }
            p { class: "web-page-head-line", "{line}" }
        }
    }
}

/// Below the breakpoint the title and the `?` sit centered in the folded
/// nav. With a `description`, a wide window opens the page with a
/// [`PageHead`]. Without one the `?` stays at the bar's right end on a wide
/// window. With `head_place` [`HeadPlace::Content`] the frame draws neither:
/// the screen places the head itself. A wide window ends the page with
/// zwipe.net's footer unless `footer` is false, for a screen whose layout
/// holds the window's height.
#[component]
pub fn ScreenFrame(
    title: String,
    #[props(default)] on_hint: Option<Callback<()>>,
    #[props(default)] description: Option<String>,
    #[props(default)] head_place: HeadPlace,
    #[props(default = true)] footer: bool,
    children: Element,
) -> Element {
    let page = try_use_context::<Signal<Option<PageTitle>>>();
    let has_head = description.is_some() || head_place == HeadPlace::Content;
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
            if let Some(line) = description.filter(|_| head_place == HeadPlace::Page) {
                div { class: "web-page-head-wrap",
                    PageHead { title, line, on_hint }
                }
            }
            {children}
            if footer {
                WebFooter {}
            }
            if let Some(open) = on_hint.filter(|_| !has_head) {
                button {
                    class: "btn-action web-screen-hint",
                    "aria-label": "About this screen",
                    onclick: move |_| open.call(()),
                    "?"
                }
            }
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
