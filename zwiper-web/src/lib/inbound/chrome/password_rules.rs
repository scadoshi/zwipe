//! The password rules, shown in a panel beside a new password's form on a
//! wide window in place of the `?` that opens them below the breakpoint.

use dioxus::prelude::*;
use zwipe_ui::hint::HintTopic;

use crate::inbound::chrome::form_panel::FormPanelHead;
use crate::inbound::chrome::hint_body::HintBody;

/// The `Password` hint as a panel that stays open: its title over its
/// rules, word for word from zwipe-ui's copy of zwipe-core's
/// `requirements()`. A static list:
/// core checks a password whole, so no rule is ticked as it is met. Hidden
/// below the breakpoint, where the field's `?` opens the same list.
#[component]
pub fn PasswordRules() -> Element {
    rsx! {
        div { class: "web-password-rules",
            FormPanelHead { title: HintTopic::Password.title() }
            div { class: "web-password-rules-body",
                HintBody { blocks: HintTopic::Password.blocks() }
            }
        }
    }
}
