//! The privacy policy cut into one card per section, with the headings the
//! contents column lists.

use zwipe_core::legal::PRIVACY_POLICY_HTML;

/// The last card's heading, which the policy HTML does not carry.
pub const CONTACT: &str = "Contact";

/// One `<h2>` section of the policy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicySection {
    /// The heading's text.
    pub heading: String,
    /// The section's HTML, its heading first with a rule under it.
    pub html: String,
}

/// The policy split on `<h2>`, one section per heading in order. As in the
/// app, each keeps its heading and gets an `<hr>` under it.
pub fn sections() -> Vec<PolicySection> {
    split(PRIVACY_POLICY_HTML)
}

fn split(html: &str) -> Vec<PolicySection> {
    html.split("<h2>")
        .map(str::trim)
        .filter(|chunk| !chunk.is_empty())
        .map(|chunk| PolicySection {
            heading: chunk
                .split_once("</h2>")
                .map_or(chunk, |(heading, _)| heading)
                .trim()
                .to_string(),
            html: format!("<h2>{chunk}").replacen("</h2>", "</h2>\n<hr>", 1),
        })
        .collect()
}

/// The element id a heading's card takes, so the contents column can scroll
/// to it: `privacy-` and the heading in lowercase, every run of other
/// characters a single `-`.
pub fn anchor(heading: &str) -> String {
    let mut id = String::from("privacy");
    let mut dash = true;
    for c in heading.chars() {
        if c.is_ascii_alphanumeric() {
            if dash {
                id.push('-');
                dash = false;
            }
            id.push(c.to_ascii_lowercase());
        } else {
            dash = true;
        }
    }
    id
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_policy_has_its_eight_sections_in_order() {
        let headings: Vec<String> = sections().into_iter().map(|s| s.heading).collect();
        assert_eq!(
            headings,
            [
                "Data we collect",
                "How we use your data",
                "Third-party services",
                "Data retention",
                "Where your data lives",
                "Security",
                "Children",
                "Fan content",
            ]
        );
    }

    #[test]
    fn each_section_keeps_its_heading_with_a_rule_under_it() {
        let cut = split("<h2>One</h2><p>a</p><h2>Two</h2><p>b</p>");
        assert_eq!(cut.len(), 2);
        assert_eq!(cut[0].html, "<h2>One</h2>\n<hr><p>a</p>");
        assert_eq!(cut[1].heading, "Two");
    }

    #[test]
    fn anchors_are_safe_ids() {
        assert_eq!(
            anchor("Third-party services"),
            "privacy-third-party-services"
        );
        assert_eq!(anchor("Data we collect"), "privacy-data-we-collect");
        assert_eq!(anchor(CONTACT), "privacy-contact");
    }
}
