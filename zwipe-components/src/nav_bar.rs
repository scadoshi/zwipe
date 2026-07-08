//! Sticky site nav shell: brand on the left, a link panel on the right that
//! collapses behind a hamburger toggle below the 60rem breakpoint.
//!
//! The shell owns the structure (wrapper, toggle, collapsing panel) and its
//! CSS; the host owns the content via slots — its brand link, its `li` link
//! items, and an optional trailing panel item (typically [`ThemePicker`]).
//! The host also owns the `open` signal so link `onclick`s can close the
//! panel after navigating.
//!
//! [`ThemePicker`]: crate::ThemePicker

use dioxus::prelude::*;

/// JS run on brand-click by convention: smooth-scroll to top and restart the
/// `.logo` entrance animation. Exported so every surface's brand link shares
/// one copy (`document::eval(BRAND_RESET_JS)`).
pub const BRAND_RESET_JS: &str = r#"
    window.scrollTo({ top: 0, behavior: 'smooth' });
    const el = document.querySelector('.logo');
    if (el) {
        el.style.animation = 'none';
        void el.offsetHeight;
        el.style.animation = '';
    }
"#;

/// Nav shell with a hamburger-collapsing link panel.
#[component]
pub fn NavBar(
    /// Panel open state, host-owned: pass a fresh signal and close it from
    /// your link `onclick`s.
    open: Signal<bool>,
    /// Brand element (the host's home link).
    brand: Element,
    /// Optional content pinned outside the collapsing panel, rendered between
    /// the brand and the toggle (e.g. zite's always-visible store CTAs at
    /// hamburger widths). The host styles it; the shell just places it.
    persistent: Option<Element>,
    /// Link items (`li` elements) rendered inside the panel's `ul.nav-links`.
    links: Element,
    /// Optional trailing panel item after the links (e.g. the theme picker).
    trailing: Option<Element>,
) -> Element {
    let mut open = open;
    let panel_class = if open() {
        "nav-panel nav-panel-open"
    } else {
        "nav-panel"
    };
    let toggle_class = if open() {
        "nav-toggle nav-toggle-open"
    } else {
        "nav-toggle"
    };

    rsx! {
        div { class: "nav-wrapper",
            nav {
                {brand}
                if let Some(p) = persistent {
                    {p}
                }
                button {
                    class: "{toggle_class}",
                    aria_label: "Toggle navigation menu",
                    aria_expanded: "{open()}",
                    onclick: move |_| {
                        let next = !open();
                        open.set(next);
                    },
                    span { class: "nav-toggle-bar" }
                    span { class: "nav-toggle-bar" }
                    span { class: "nav-toggle-bar" }
                }
                div { class: "{panel_class}",
                    div { class: "nav-panel-inner",
                        ul { class: "nav-links", {links} }
                        if let Some(t) = trailing {
                            {t}
                        }
                    }
                }
            }
        }
    }
}
