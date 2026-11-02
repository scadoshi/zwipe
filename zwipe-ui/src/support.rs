//! Help & feedback: the dialog's words, and the problem report email it opens.
//!
//! The dialog has no model: it holds nothing and decides nothing. Each client opens it its own way (the app's `!` in the screen header, the terminal's `!` and F1) and opens its links its own way.

use std::fmt::Write;

use zwipe_core::domain::site::SUPPORT_EMAIL;

/// The dialog's title.
pub const TITLE: &str = "Help & feedback";

/// The dialog's body.
pub const BODY: &str = "Reach the team or join the community.";

/// The glyph on the button that opens the dialog, mirroring the `?` hint button.
pub const GLYPH: &str = "!";

/// The accessible name of the button that opens the dialog.
pub const BUTTON_LABEL: &str = "Help and feedback";

/// The button that closes the dialog.
pub const CLOSE: &str = "Close";

/// The button that opens the problem report email from [`report_mailto`].
pub const REPORT: &str = "Report a problem \u{2197}";

/// The button that opens the Discord invite.
pub const DISCORD: &str = "Join the Discord \u{2197}";

/// The report email's subject.
const SUBJECT: &str = "Zwipe problem report";

/// The problem report's `mailto:` link, its body ending with the client's
/// `version` and `platform`, such as `iOS`, `Android`, `Web` or `Terminal`.
#[must_use]
pub fn report_mailto(version: &str, platform: &str) -> String {
    let subject = urlencode(SUBJECT);
    let body = urlencode(&format!(
        "Describe the problem:\n\n\n---\nZwipe {version} \u{00b7} {platform}"
    ));
    format!("mailto:{SUPPORT_EMAIL}?subject={subject}&body={body}")
}

/// Percent-encodes `input` for a `mailto:` query: RFC 3986's unreserved
/// characters pass through, every other byte becomes `%XX`.
#[must_use]
pub fn urlencode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for byte in input.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            out.push(char::from(byte));
        } else {
            // Writing to a String never fails.
            let _ = write!(out, "%{byte:02X}");
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_copy_matches_the_app() {
        assert_eq!(TITLE, "Help & feedback");
        assert_eq!(BODY, "Reach the team or join the community.");
        assert_eq!(GLYPH, "!");
        assert_eq!(BUTTON_LABEL, "Help and feedback");
        assert_eq!(CLOSE, "Close");
        assert_eq!(REPORT, "Report a problem ↗");
        assert_eq!(DISCORD, "Join the Discord ↗");
    }

    #[test]
    fn the_report_link_encodes_everything_but_the_unreserved_characters() {
        assert_eq!(urlencode("a b·~"), "a%20b%C2%B7~");
        assert_eq!(
            report_mailto("1.16.2", "iOS"),
            "mailto:support@zwipe.net?subject=Zwipe%20problem%20report&body=Describe%20the%20problem%3A%0A%0A%0A---%0AZwipe%201.16.2%20%C2%B7%20iOS"
        );
    }
}
