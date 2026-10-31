//! The Help & feedback dialog's problem report link.

use std::fmt::Write;

use zwipe_core::domain::site::SUPPORT_EMAIL;

/// The platform named in a problem report.
pub const PLATFORM: &str = "Web";

/// A `mailto:` link to support with the report subject and a body that ends
/// with the app version and platform.
pub fn report_mailto(app_version: &str) -> String {
    let subject = urlencode("Zwipe problem report");
    let body = urlencode(&format!(
        "Describe the problem:\n\n\n---\nZwipe {app_version} \u{00b7} {PLATFORM}"
    ));
    format!("mailto:{SUPPORT_EMAIL}?subject={subject}&body={body}")
}

/// Percent-encodes `input` for a `mailto:` query: the RFC 3986 unreserved
/// set passes through and every other byte becomes `%XX` in uppercase hex.
fn urlencode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(char::from(byte));
            }
            _ => {
                let _ = write!(out, "%{byte:02X}");
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_unreserved_set_passes_through() {
        assert_eq!(urlencode("aZ09-_.~"), "aZ09-_.~");
    }

    #[test]
    fn everything_else_is_percent_encoded() {
        assert_eq!(urlencode("a b\n\u{00b7}"), "a%20b%0A%C2%B7");
    }

    #[test]
    fn the_report_names_the_version_and_platform() {
        let link = report_mailto("1.16.2");
        assert!(
            link.starts_with("mailto:support@zwipe.net?subject=Zwipe%20problem%20report&body=")
        );
        assert!(link.ends_with("Zwipe%201.16.2%20%C2%B7%20Web"), "{link}");
    }
}
