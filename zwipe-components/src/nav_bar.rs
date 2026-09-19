//! Sticky site nav shell: brand on the left, a link panel on the right that
//! collapses behind a hamburger below 60rem. The shell owns structure and CSS;
//! the host fills the slots and owns the `open` signal so link `onclick`s can
//! close the panel.

use dioxus::prelude::*;

/// Run on brand click: smooth-scroll to top and restart the `.logo` entrance
/// animation. One copy for every surface, via `document::eval`.
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
    /// Panel open state. Close it from your link `onclick`s.
    open: Signal<bool>,
    /// The host's home link.
    brand: Element,
    /// Content pinned outside the collapsing panel, between brand and toggle.
    persistent: Option<Element>,
    /// `li` items for the panel's `ul.nav-links`.
    links: Element,
    /// Trailing panel item after the links, typically [`ThemePicker`](crate::ThemePicker).
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
