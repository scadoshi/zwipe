//! The privacy policy as text: `zwipe_core::legal`'s HTML read into sections of paragraphs and bullets, with bold runs and links kept apart so the screen can style them.
//!
//! It reads only what the policy uses: `h2`, `p`, `ul` and `li`, `strong`, and `a` with an `href`. Any other tag is dropped and its text kept.

/// A run of text: plain, bold, or a link with its address.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Run {
    Plain(String),
    Bold(String),
    Link { text: String, url: String },
}

/// A paragraph or a bullet, as runs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Block {
    Paragraph(Vec<Run>),
    Bullet(Vec<Run>),
}

/// One `h2` section: its heading and its blocks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Section {
    pub(crate) heading: String,
    pub(crate) blocks: Vec<Block>,
}

/// The few entities the policy may hold, back to their characters.
fn unescape(text: &str) -> String {
    text.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}

/// The `href` of an `a` tag's attributes.
fn href(attributes: &str) -> String {
    attributes
        .split_once("href=\"")
        .and_then(|(_, rest)| rest.split_once('"'))
        .map(|(url, _)| url.to_owned())
        .unwrap_or_default()
}

/// Reads `html` into blocks, with any `h2` headings in order beside them: a heading starts a section.
pub(crate) fn parse(html: &str) -> Vec<Section> {
    let mut sections = vec![Section {
        heading: String::new(),
        blocks: Vec::new(),
    }];
    let mut runs: Vec<Run> = Vec::new();
    let mut text = String::new();
    let mut bold = false;
    let mut link: Option<String> = None;
    let mut heading = false;
    let mut rest = html;

    let flush_text = |text: &mut String, runs: &mut Vec<Run>, bold: bool, link: &Option<String>| {
        let collapsed = unescape(&text.split_whitespace().collect::<Vec<_>>().join(" "));
        let lead = text.starts_with(char::is_whitespace) && !runs.is_empty();
        let trail = text.ends_with(char::is_whitespace);
        text.clear();
        if collapsed.is_empty() {
            if lead || trail {
                runs.push(Run::Plain(" ".to_owned()));
            }
            return;
        }
        let spaced = format!(
            "{}{collapsed}{}",
            if lead { " " } else { "" },
            if trail { " " } else { "" }
        );
        runs.push(match link {
            Some(url) => Run::Link {
                text: spaced,
                url: url.clone(),
            },
            None if bold => Run::Bold(spaced),
            None => Run::Plain(spaced),
        });
    };

    while let Some(open) = rest.find('<') {
        let (before, from) = rest.split_at(open);
        text.push_str(before);
        let Some(close) = from.find('>') else {
            break;
        };
        let tag = from.get(1..close).unwrap_or_default();
        rest = from.get(close.saturating_add(1)..).unwrap_or_default();
        let (name, attributes) = tag.split_once(' ').unwrap_or((tag, ""));
        match name {
            "h2" => {
                text.clear();
                heading = true;
            }
            "/h2" => {
                let title = unescape(text.trim());
                text.clear();
                heading = false;
                sections.push(Section {
                    heading: title,
                    blocks: Vec::new(),
                });
            }
            "strong" | "/strong" | "a" | "/a" => {
                if !heading {
                    flush_text(&mut text, &mut runs, bold, &link);
                }
                match name {
                    "strong" => bold = true,
                    "/strong" => bold = false,
                    "a" => link = Some(href(attributes)),
                    _ => link = None,
                }
            }
            "/p" | "/li" => {
                flush_text(&mut text, &mut runs, bold, &link);
                let block = std::mem::take(&mut runs);
                if let Some(section) = sections.last_mut() {
                    section.blocks.push(if name == "/p" {
                        Block::Paragraph(block)
                    } else {
                        Block::Bullet(block)
                    });
                }
            }
            "p" | "li" => {
                text.clear();
                runs.clear();
            }
            _ => {}
        }
    }
    sections.retain(|s| !s.heading.is_empty() || !s.blocks.is_empty());
    sections
}

#[cfg(test)]
mod tests {
    use zwipe_core::legal::{PRIVACY_INTRO_HTML, PRIVACY_POLICY_HTML};

    use super::*;

    #[test]
    fn the_policy_reads_into_its_eight_sections_in_order() {
        let sections = parse(PRIVACY_POLICY_HTML);
        let headings: Vec<&str> = sections.iter().map(|s| s.heading.as_str()).collect();
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
        let first = sections.first().unwrap();
        assert!(matches!(
            first.blocks.first(),
            Some(Block::Bullet(runs)) if runs.first() == Some(&Run::Bold("Account data".to_owned()))
        ));
        assert!(matches!(first.blocks.last(), Some(Block::Paragraph(_))));
    }

    #[test]
    fn links_keep_their_address_and_spacing_survives() {
        let intro = parse(PRIVACY_INTRO_HTML);
        let Some(Block::Paragraph(runs)) = intro[0].blocks.first() else {
            panic!("no intro paragraph");
        };
        assert_eq!(runs[0], Run::Plain("Zwipe is a ".to_owned()));
        assert_eq!(
            runs[1],
            Run::Link {
                text: "Magic: The Gathering".to_owned(),
                url: "https://magic.wizards.com/en".to_owned()
            }
        );
        assert!(matches!(&runs[2], Run::Plain(t) if t.starts_with(" deck builder")));
    }
}
