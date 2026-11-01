//! Account's button labels: the resend cooldown and the delete countdown.

/// The Resend button's label while a send is out, while the cooldown runs,
/// and otherwise.
pub fn resend_label(sending: bool, cooldown: u32) -> String {
    if sending {
        "Sending...".to_string()
    } else if cooldown > 0 {
        format!("Resend in {cooldown}s")
    } else {
        "Resend".to_string()
    }
}

/// The Delete button's label with `remaining` seconds on its countdown.
pub fn delete_label(remaining: u8) -> String {
    match remaining {
        0 => "Delete".to_string(),
        n => format!("Delete ({n})"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resend_says_what_it_is_doing() {
        assert_eq!(resend_label(true, 60), "Sending...");
        assert_eq!(resend_label(false, 42), "Resend in 42s");
        assert_eq!(resend_label(false, 0), "Resend");
    }

    #[test]
    fn delete_counts_down_then_unlocks() {
        assert_eq!(delete_label(5), "Delete (5)");
        assert_eq!(delete_label(1), "Delete (1)");
        assert_eq!(delete_label(0), "Delete");
    }
}
