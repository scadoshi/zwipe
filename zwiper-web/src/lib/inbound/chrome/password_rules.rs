//! The password rules, shown under a new password's fields on a wide window
//! in place of the `?` that opens them below the breakpoint.

use dioxus::prelude::*;
use zwipe_ui::hint::HintTopic;

use crate::inbound::chrome::hint_body::HintBody;

/// The `Password` hint's rules as a list that stays open, word for word
/// from zwipe-ui's copy of zwipe-core's `requirements()`. A static list:
/// core checks a password whole, so no rule is ticked as it is met. Hidden
/// below the breakpoint, where the field's `?` opens the same list.
#[component]
pub fn PasswordRules() -> Element {
    rsx! {
        div { class: "web-password-rules",
            HintBody { blocks: HintTopic::Password.blocks() }
        }
    }
}
