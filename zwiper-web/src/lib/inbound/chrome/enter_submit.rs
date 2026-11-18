//! How Enter submits a form on the site: the fields sit in a real `form`, marked `novalidate`,
//! whose `onsubmit` runs the screen's submit, and [`EnterSubmit`] opens it as the form's default
//! button. Enter in a text field then submits the form, as the browser does for any form, and the
//! submit runs the screen's own checks, so a form that isn't ready shows its field errors instead
//! of sending. A textarea keeps Enter as a new line, and a confirm the submit opens still asks
//! first.

use dioxus::prelude::*;

/// The form's default button, hidden, which Enter in any of the form's fields presses. It goes
/// first in the form: the browser presses the first submit button, and a visible one held
/// disabled would otherwise swallow Enter. Every other button in the form is `type="button"`.
#[component]
pub fn EnterSubmit() -> Element {
    rsx! {
        button { r#type: "submit", hidden: true }
    }
}
