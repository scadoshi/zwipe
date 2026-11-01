//! Accordion component implementations.

use dioxus::prelude::*;
use dioxus_primitives::accordion::{
    self, AccordionContentProps, AccordionItemProps, AccordionProps, AccordionTriggerProps,
};

/// Root accordion container that manages expand/collapse state.
#[component]
pub fn Accordion(props: AccordionProps) -> Element {
    rsx! {
        // accordion.css loads once at the app root, in zwiper/src/bin/zwiper.rs.
        accordion::Accordion {
            class: "accordion",
            width: "100%",
            id: props.id,
            allow_multiple_open: props.allow_multiple_open,
            disabled: props.disabled,
            collapsible: props.collapsible,
            horizontal: props.horizontal,
            attributes: props.attributes,
            {props.children}
        }
    }
}

/// Individual accordion section containing a trigger and collapsible content.
#[component]
pub fn AccordionItem(props: AccordionItemProps) -> Element {
    rsx! {
        accordion::AccordionItem {
            class: "accordion-item",
            disabled: props.disabled,
            default_open: props.default_open,
            on_change: props.on_change,
            on_trigger_click: props.on_trigger_click,
            index: props.index,
            attributes: props.attributes,
            {props.children}
        }
    }
}

/// Clickable header that toggles the accordion item's expanded state.
#[component]
pub fn AccordionTrigger(props: AccordionTriggerProps) -> Element {
    rsx! {
        accordion::AccordionTrigger {
            class: "accordion-trigger",
            id: props.id,
            attributes: props.attributes,
            {props.children}
            svg {
                class: "accordion-expand-icon",
                view_box: "0 0 24 24",
                xmlns: "http://www.w3.org/2000/svg",
                polyline { points: "6 9 12 15 18 9" }
            }
        }
    }
}

/// Collapsible content area that appears when the accordion item is expanded.
#[component]
pub fn AccordionContent(props: AccordionContentProps) -> Element {
    rsx! {
        accordion::AccordionContent {
            class: "accordion-content",
            style: "--collapsible-content-width: 100%",
            id: props.id,
            attributes: props.attributes,
            {props.children}
        }
    }
}
