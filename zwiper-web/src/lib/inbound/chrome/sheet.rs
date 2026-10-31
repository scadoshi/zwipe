//! The web forms of the app's bottom sheets.
//!
//! A sheet keeps the kit's `BottomSheet` and its parts; [`SheetFrame`] only
//! tells `web.css` where a wide window puts it. Below the breakpoint every
//! form is the app's bottom sheet again.

use dioxus::prelude::*;

/// Where a sheet sits on a wide window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SheetForm {
    /// A centered modal, for short forms and lists.
    Modal,
    /// A panel along the right edge, for a sheet that edits what stays in
    /// view beside it. Its backdrop does not dim the page.
    SidePanel,
}

impl SheetForm {
    fn class(self) -> &'static str {
        match self {
            SheetForm::Modal => "web-sheet web-sheet-modal",
            SheetForm::SidePanel => "web-sheet web-sheet-side",
        }
    }
}

/// Wraps one `BottomSheet` in `children` so a wide window draws it as
/// `form`. The wrapper has no box of its own; the sheet and its backdrop
/// stay fixed to the window.
#[component]
pub fn SheetFrame(form: SheetForm, children: Element) -> Element {
    rsx! {
        div { class: form.class(), {children} }
    }
}
