use dioxus::prelude::*;
use dioxus_primitives::alert_dialog::{
    self, AlertDialogActionProps, AlertDialogActionsProps, AlertDialogCancelProps,
    AlertDialogContentProps, AlertDialogDescriptionProps, AlertDialogRootProps,
    AlertDialogTitleProps,
};

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

#[component]
pub fn AlertDialogActions(props: AlertDialogActionsProps) -> Element {
    rsx! {
        alert_dialog::AlertDialogActions { class: "alert-dialog-actions", attributes: props.attributes, {props.children} }
    }
}

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

#[component]
pub fn AlertDialogAction(props: AlertDialogActionProps) -> Element {
    rsx! {
        alert_dialog::AlertDialogAction {
            class: "alert-dialog-action",
            on_click: props.on_click,
            attributes: props.attributes,
            {props.children}
        }
    }
}
