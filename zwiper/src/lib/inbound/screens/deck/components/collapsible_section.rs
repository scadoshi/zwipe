//! Collapsible card section for the deck view — a titled card whose body eases
//! open/closed (height + opacity) like the edit-deck commander fields, with a
//! left arrow that rotates like the deck cards screen. Accordion-style: a shared
//! `open_section` signal keeps only one section open at a time.

use dioxus::prelude::*;

#[component]
pub(crate) fn CollapsibleSection(
    title: String,
    /// Optional summary shown next to the title (e.g. a count).
    #[props(default)]
    badge: Option<String>,
    /// Use the warning border accent.
    #[props(default = false)]
    warn: bool,
    /// Optional controls rendered on the right of the header bar; fades in with
    /// the section's open state and is not part of the expand/collapse hit area.
    #[props(default)]
    header_accessory: Option<Element>,
    /// Shared accordion state — holds the title of the one open section, if any.
    mut open_section: Signal<Option<String>>,
    children: Element,
) -> Element {
    let id = title.clone();
    let is_open = open_section() == Some(id.clone());
    let border = if warn {
        "var(--border-warning)"
    } else {
        "var(--border-secondary)"
    };

    rsx! {
        // Horizontal padding lives on the header/body (not the card) so the
        // divider can span edge-to-edge; the collapsible clip would otherwise
        // cut off a negative-margin full-bleed rule.
        div { style: format!("width:100%;background:var(--bg-primary);box-shadow:var(--shadow-sm);border:1px solid {border};border-radius:0.5rem;padding:0.75rem 0;display:flex;flex-direction:column;"),
            div {
                style: "display:flex;align-items:center;gap:0.5rem;padding:0 0.75rem;",
                // Toggle hit area (arrow + title + optional badge). The accessory
                // sits outside it so its controls don't collapse the section.
                div {
                    style: "display:flex;align-items:center;gap:0.5rem;cursor:pointer;user-select:none;flex:1;min-width:0;",
                    onclick: move |_| {
                        if is_open {
                            open_section.set(None);
                        } else {
                            open_section.set(Some(id.clone()));
                        }
                    },
                    span {
                        class: "card-row-arrow",
                        style: if is_open { "transform:rotate(90deg);" } else { "transform:rotate(0deg);" },
                        "▸"
                    }
                    span { class: "card-title", "{title}" }
                    if let Some(badge) = badge.as_ref() {
                        span { style: "font-size:0.65rem;opacity:0.5;", "{badge}" }
                    }
                }
                if let Some(accessory) = header_accessory {
                    div {
                        style: format!(
                            "transition:opacity 0.25s ease;opacity:{};pointer-events:{};",
                            if is_open { "1" } else { "0" },
                            if is_open { "auto" } else { "none" },
                        ),
                        {accessory}
                    }
                }
            }
            // Height + opacity ease via the grid-template-rows 0fr<->1fr technique
            // (same as the edit-deck commander fields). Body stays mounted.
            // No horizontal padding here so dividers (section rule + info-row
            // borders) bleed to the card edges; each child supplies its own inset.
            div { class: if is_open { "collapsible open" } else { "collapsible" },
                div { class: "collapsible-inner",
                    hr { style: "border:none;border-top:1px solid var(--border-secondary);margin:0.5rem 0 0.6rem 0;" }
                    div { style: "display:flex;flex-direction:column;gap:0.75rem;", {children} }
                }
            }
        }
    }
}
