//! Alert dialog component implementations.

use dioxus::prelude::*;
use dioxus_primitives::alert_dialog::{
    self, AlertDialogActionsProps, AlertDialogCancelProps, AlertDialogContentProps,
    AlertDialogDescriptionProps, AlertDialogRootProps, AlertDialogTitleProps,
};

/// Root container for alert dialogs, managing open/closed state.
#[component]
pub fn AlertDialogRoot(props: AlertDialogRootProps) -> Element {
    rsx! {
        // Context: the AlertDialog element could not load its .css with this method for some
        // reason.. After some time spent debugging, I decided to just load in
        // zwiper/src/bin/main.rs for now and attend to later. All other elements with their own
        // style sheets will get the same treatment to keep consistency for now..
        //
        //document::Link { rel: "stylesheet", href: asset!("/assets/alert-dialog.css") }
        alert_dialog::AlertDialogRoot {
            id: props.id,
            default_open: props.default_open,
            open: props.open,
            on_open_change: props.on_open_change,
            attributes: props.attributes,
            {props.children}
        }
    }
}

/// Container for the alert dialog's visible content (title, description, actions).
#[component]
pub fn AlertDialogContent(props: AlertDialogContentProps) -> Element {
    rsx! {
        alert_dialog::AlertDialogContent {
            id: props.id,
            class: props.class.unwrap_or_default() + " alert-dialog",
            attributes: props.attributes,
            {props.children}
        }
    }
}

/// Title heading for the alert dialog.
#[component]
pub fn AlertDialogTitle(props: AlertDialogTitleProps) -> Element {
    rsx! {
        alert_dialog::AlertDialogTitle {
            class: "alert-dialog-title",
            attributes: props.attributes,
            {props.children}
        }
    }
}

/// Descriptive text explaining the alert dialog's purpose.
#[component]
pub fn AlertDialogDescription(props: AlertDialogDescriptionProps) -> Element {
    rsx! {
        alert_dialog::AlertDialogDescription {
            class: "alert-dialog-description",
            attributes: props.attributes,
            {props.children}
        }
    }
}

/// Container for action buttons (confirm/cancel) in the alert dialog.
#[component]
pub fn AlertDialogActions(props: AlertDialogActionsProps) -> Element {
    rsx! {
        alert_dialog::AlertDialogActions { class: "alert-dialog-actions", attributes: props.attributes, {props.children} }
    }
}

/// Cancel button that closes the dialog without taking action.
#[component]
pub fn AlertDialogCancel(props: AlertDialogCancelProps) -> Element {
    rsx! {
        alert_dialog::AlertDialogCancel {
            on_click: props.on_click,
            class: "alert-dialog-cancel",
            attributes: props.attributes,
            {props.children}
        }
    }
}

/// Confirm/action button that triggers the primary action.
///
/// Pass `danger: true` for irreversible destructive operations — the button
/// will render with the `alert-dialog-action-danger` style (red border + text).
#[component]
pub fn AlertDialogAction(
    on_click: EventHandler<MouseEvent>,
    #[props(default = false)] danger: bool,
    children: Element,
) -> Element {
    let class = if danger {
        "alert-dialog-action-danger"
    } else {
        "alert-dialog-action"
    };
    rsx! {
        alert_dialog::AlertDialogAction {
            class: class,
            on_click: on_click,
            {children}
        }
    }
}
