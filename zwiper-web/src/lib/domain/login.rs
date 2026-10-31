//! What a Log in click does.

/// The toast for a Log in click with a field left empty.
pub const MISSING_INPUTS: &str = "Enter your username or email and password";

/// The outcome of a Log in click.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubmitGate {
    /// A request is already out: ignore the click.
    Busy,
    /// Nothing to send: toast [`MISSING_INPUTS`].
    MissingInputs,
    /// Send the request.
    Send,
}

/// Decides a Log in click from the loading flag and the two fields.
///
/// The identifier counts as empty when it is only whitespace, but it is sent
/// as typed. The password is checked for presence only: login applies no
/// policy, since the server is the only judge of a credential.
pub fn submit_gate(is_loading: bool, identifier: &str, password: &str) -> SubmitGate {
    if is_loading {
        SubmitGate::Busy
    } else if identifier.trim().is_empty() || password.is_empty() {
        SubmitGate::MissingInputs
    } else {
        SubmitGate::Send
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_click_while_logging_in_sends_nothing() {
        assert_eq!(submit_gate(true, "alice", "pw"), SubmitGate::Busy);
        assert_eq!(submit_gate(true, "", ""), SubmitGate::Busy);
    }

    #[test]
    fn an_empty_field_asks_for_both() {
        assert_eq!(submit_gate(false, "", "pw"), SubmitGate::MissingInputs);
        assert_eq!(submit_gate(false, "   ", "pw"), SubmitGate::MissingInputs);
        assert_eq!(submit_gate(false, "alice", ""), SubmitGate::MissingInputs);
    }

    #[test]
    fn a_password_of_spaces_still_sends() {
        assert_eq!(submit_gate(false, "alice", "  "), SubmitGate::Send);
    }
}
