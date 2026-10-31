//! The problem report email every client's Help & feedback dialog opens.

use std::fmt::Write;

use zwipe_core::domain::site::SUPPORT_EMAIL;

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
    fn the_report_link_encodes_everything_but_the_unreserved_characters() {
        assert_eq!(urlencode("a b·~"), "a%20b%C2%B7~");
        assert_eq!(
            report_mailto("1.16.2", "iOS"),
            "mailto:support@zwipe.net?subject=Zwipe%20problem%20report&body=Describe%20the%20problem%3A%0A%0A%0A---%0AZwipe%201.16.2%20%C2%B7%20iOS"
        );
    }
}
