//! What the guide pages show, shared by zwipe.net and the web app: a guide
//! line's runs (text, `backtick` keywords and `[[icon]]` tokens), the
//! index's search and tag filter, the tag colors, the section anchors and
//! the screenshots a guide's gallery shows. The guides themselves are
//! [`zwipe_core::content::guides`]; components draws them.

use zwipe_core::content::guides::{Block, GUIDES, Guide};

/// One run of a guide line: plain text, a `backtick` keyword, or an
/// `[[icon]]` naming an icon button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Inline<'line> {
    /// Text drawn as is.
    Text(&'line str),
    /// A highlighted keyword: a filter name, an option value, a button's
    /// words.
    Keyword(&'line str),
    /// An icon button, named as [`KeyIcon::name`](crate::hint::KeyIcon::name)
    /// lists them, drawn as the button's own icon in a hint key. A name no
    /// icon has renders as the literal token so a typo shows.
    Icon(&'line str),
}

/// Splits a guide line into its runs: `backtick` pairs are keywords and
/// `[[name]]` is an icon; an unclosed `[[` stays text.
#[must_use]
pub fn tokens(text: &str) -> Vec<Inline<'_>> {
    let mut runs = Vec::new();
    for (i, part) in text.split('`').enumerate() {
        if i % 2 == 1 {
            runs.push(Inline::Keyword(part));
            continue;
        }
        let mut rest = part;
        while let Some((before, after)) = rest.split_once("[[") {
            let Some((name, tail)) = after.split_once("]]") else {
                break;
            };
            if !before.is_empty() {
                runs.push(Inline::Text(before));
            }
            runs.push(Inline::Icon(name));
            rest = tail;
        }
        if !rest.is_empty() {
            runs.push(Inline::Text(rest));
        }
    }
    runs
}

/// A guide line's words alone: keywords without their backticks, icon
/// tokens dropped.
#[must_use]
pub fn prose(text: &str) -> String {
    tokens(text)
        .into_iter()
        .filter_map(|run| match run {
            Inline::Text(s) | Inline::Keyword(s) => Some(s),
            Inline::Icon(_) => None,
        })
        .collect()
}

/// One screenshot in a guide's gallery.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Shot {
    /// The file under the shared image path, `<slug>/<name>.webp`.
    pub file: &'static str,
    /// The image's alt text.
    pub alt: &'static str,
    /// The caption under it; the alt text when the guide gives none.
    pub caption: &'static str,
}

/// A guide's screenshots in order, for its gallery. The article's text
/// leaves them out.
#[must_use]
pub fn shots(guide: &Guide) -> Vec<Shot> {
    guide
        .blocks
        .iter()
        .filter_map(|block| match *block {
            Block::Image { file, alt, caption } => Some(Shot {
                file,
                alt,
                caption: caption.unwrap_or(alt),
            }),
            _ => None,
        })
        .collect()
}

/// The guides `related` names that exist, in its order.
pub fn related(guide: &Guide) -> impl Iterator<Item = &'static Guide> + '_ {
    guide
        .related
        .iter()
        .filter_map(|slug| GUIDES.iter().find(|g| g.slug == *slug))
}

/// A section heading's id, so a link can land on it: the heading in
/// lowercase, its letters and digits kept and every run of anything else a
/// single `-`, with none at either end.
#[must_use]
pub fn heading_anchor(heading: &str) -> String {
    let mut id = String::new();
    let mut dash = false;
    for c in heading.chars() {
        if c.is_alphanumeric() {
            if dash && !id.is_empty() {
                id.push('-');
            }
            dash = false;
            id.extend(c.to_lowercase());
        } else {
            dash = true;
        }
    }
    id
}

/// The app's gesture color class for a swipe direction, shared with
/// zwipe.net's home hero, so a guide's swipe legend matches the app's hints.
#[must_use]
pub fn swipe_class(dir: &str) -> &'static str {
    match dir {
        "right" => "swipe-add",
        "left" => "swipe-skip",
        "up" => "swipe-maybe",
        "down" => "swipe-undo",
        _ => "",
    }
}

/// The index's tag chips, in display order. Each guide carries one to three
/// of them.
pub const TAGS: &[&str] = &[
    "Getting started",
    "Swiping",
    "Filtering",
    "Cards",
    "Commander",
    "Oracle tags",
    "Deck building",
    "Deck stats",
    "Importing",
];

/// A tag's color class, by its place in [`TAGS`], so a tag reads the same
/// color on every card.
#[must_use]
pub fn tag_color_class(tag: &str) -> &'static str {
    const CLASSES: [&str; 6] = ["tag-p1", "tag-p2", "tag-p3", "tag-p4", "tag-p5", "tag-p6"];
    let idx = TAGS.iter().position(|t| *t == tag).unwrap_or(0);
    idx.checked_rem(CLASSES.len())
        .and_then(|i| CLASSES.get(i))
        .unwrap_or(&"tag-p1")
}

/// Everything about a guide the index search can match: title, summary,
/// tags and the article's prose, lowercased, with the `backtick` markers
/// stripped so a search for "group by" finds `Group by`, and the `[[icon]]`
/// tokens dropped, since nobody types them.
///
/// Built per keystroke over the compiled [`GUIDES`] (a couple dozen
/// articles), cheap enough that caching would cost more than it saves.
#[must_use]
pub fn haystack(guide: &Guide) -> String {
    let mut s = String::new();
    let mut push = |t: &str| {
        s.push_str(&prose(t));
        s.push(' ');
    };
    push(guide.title);
    push(guide.summary);
    for t in guide.tags.iter().copied() {
        push(t);
    }
    for block in guide.blocks {
        match block {
            Block::Lead(t) | Block::H2(t) | Block::P(t) | Block::Note(t) => push(t),
            Block::Steps(items) | Block::Bullets(items) => {
                for it in items.iter().copied() {
                    push(it);
                }
            }
            Block::Swipe(rows) => {
                for (dir, meaning) in *rows {
                    push(dir);
                    push(meaning);
                }
            }
            // Diagrams are ASCII flow art and images live in the gallery;
            // neither reads as prose a searcher would type.
            Block::Diagram(_) | Block::Image { .. } => {}
        }
    }
    s.to_lowercase()
}

/// Whether a guide matches a lowercased query: every whitespace-separated
/// term appears somewhere in its [`haystack`], so extra words narrow the
/// list.
#[must_use]
pub fn matches_query(guide: &Guide, query: &str) -> bool {
    let hay = haystack(guide);
    query.split_whitespace().all(|term| hay.contains(term))
}

/// The guides the index shows: those carrying `tag` (any, with none) that
/// match `query`, trimmed and lowercased here, in [`GUIDES`] order.
#[must_use]
pub fn hits(tag: Option<&str>, query: &str) -> Vec<&'static Guide> {
    let query = query.trim().to_lowercase();
    GUIDES
        .iter()
        .filter(|g| tag.is_none_or(|t| g.tags.contains(&t)))
        .filter(|g| query.is_empty() || matches_query(g, &query))
        .collect()
}

/// A step through a guide's gallery.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GalleryStep {
    /// To the screenshot before, wrapping to the last.
    Back,
    /// To the screenshot after, wrapping to the first.
    Forward,
}

/// The keys that step a guide's gallery: the arrows, and `h` and `l`
/// (lowercase only, since Shift+H opens the shortcuts sheet).
pub const GALLERY_KEYS: [&str; 4] = ["ArrowLeft", "ArrowRight", "h", "l"];

/// The gallery step `key` stands for.
#[must_use]
pub fn gallery_step(key: &str) -> Option<GalleryStep> {
    match key {
        "ArrowLeft" | "h" => Some(GalleryStep::Back),
        "ArrowRight" | "l" => Some(GalleryStep::Forward),
        _ => None,
    }
}

/// The screenshot `step` lands on from `index` in a gallery of `total`,
/// wrapping at either end as the gallery's buttons do.
#[must_use]
pub fn stepped(index: usize, total: usize, step: GalleryStep) -> usize {
    if total == 0 {
        return 0;
    }
    match step {
        GalleryStep::Back => index
            .checked_sub(1)
            .unwrap_or_else(|| total.saturating_sub(1)),
        GalleryStep::Forward => index.saturating_add(1).checked_rem(total).unwrap_or(0),
    }
}

/// The index's filter as the query its address carries.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct IndexFilter {
    /// The tag chip on, one of [`TAGS`].
    pub tag: Option<&'static str>,
    /// What the search box holds, as typed.
    pub query: String,
}

impl IndexFilter {
    /// The filter a page address's query (`?tag=Commander&q=mana`, with or
    /// without the `?`) holds. A tag no chip has, and anything else in the
    /// query, are left out.
    #[must_use]
    pub fn from_search(search: &str) -> Self {
        let mut filter = Self::default();
        for pair in search.trim_start_matches('?').split('&') {
            let (name, value) = pair.split_once('=').unwrap_or((pair, ""));
            let value = decode(value);
            match name {
                "tag" => filter.tag = TAGS.iter().copied().find(|t| *t == value),
                "q" => filter.query = value,
                _ => {}
            }
        }
        filter
    }

    /// The query the address carries for this filter, with its `?`, or
    /// nothing with no tag and an empty search.
    #[must_use]
    pub fn to_search(&self) -> String {
        let mut parts = Vec::new();
        if let Some(tag) = self.tag {
            parts.push(format!("tag={}", encode(tag)));
        }
        if !self.query.is_empty() {
            parts.push(format!("q={}", encode(&self.query)));
        }
        if parts.is_empty() {
            String::new()
        } else {
            format!("?{}", parts.join("&"))
        }
    }
}

/// `text` percent-encoded for a query value, a space as `+`.
fn encode(text: &str) -> String {
    let mut out = String::new();
    for byte in text.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(char::from(byte));
            }
            b' ' => out.push('+'),
            _ => {
                out.push('%');
                for nibble in [byte >> 4, byte & 0x0f] {
                    out.extend(
                        char::from_digit(u32::from(nibble), 16).map(|c| c.to_ascii_uppercase()),
                    );
                }
            }
        }
    }
    out
}

/// A query value decoded: `+` as a space and `%XX` as its byte. A broken
/// escape stays as written.
fn decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while let Some(&byte) = bytes.get(i) {
        let escaped = (byte == b'%')
            .then(|| bytes.get(i.saturating_add(1)..i.saturating_add(3)))
            .flatten()
            .and_then(|hex| std::str::from_utf8(hex).ok())
            .and_then(|hex| u8::from_str_radix(hex, 16).ok());
        match (byte, escaped) {
            (_, Some(decoded)) => {
                out.push(decoded);
                i = i.saturating_add(3);
            }
            (b'+', None) => {
                out.push(b' ');
                i = i.saturating_add(1);
            }
            (other, None) => {
                out.push(other);
                i = i.saturating_add(1);
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod tests {
    use zwipe_core::content::guides::find;

    use super::*;
    use crate::hint::KeyIcon;

    #[test]
    fn a_heading_anchor_is_its_words_joined_by_dashes() {
        assert_eq!(heading_anchor("A skip lasts a week"), "a-skip-lasts-a-week");
        assert_eq!(
            heading_anchor("Profile, Budget & Tags"),
            "profile-budget-tags"
        );
        assert_eq!(heading_anchor("  Drilling down! "), "drilling-down");
        assert_eq!(heading_anchor("Mana (and lands)"), "mana-and-lands");
    }

    #[test]
    fn a_line_splits_into_text_keywords_and_icons() {
        assert_eq!(
            tokens("Tap the [[funnel]], then `Apply`."),
            vec![
                Inline::Text("Tap the "),
                Inline::Icon("funnel"),
                Inline::Text(", then "),
                Inline::Keyword("Apply"),
                Inline::Text("."),
            ]
        );
        assert_eq!(
            tokens("[[plus]] starts a deck"),
            vec![Inline::Icon("plus"), Inline::Text(" starts a deck")]
        );
        assert_eq!(
            tokens("an unclosed [[ stays"),
            vec![Inline::Text("an unclosed [[ stays")],
            "an unclosed token is text"
        );
        assert_eq!(tokens(""), vec![]);
    }

    #[test]
    fn every_icon_token_in_the_guides_names_a_real_icon() {
        let mut lines: Vec<&str> = Vec::new();
        for g in GUIDES {
            for b in g.blocks {
                match b {
                    Block::Lead(t) | Block::H2(t) | Block::P(t) | Block::Note(t) => lines.push(t),
                    Block::Steps(items) | Block::Bullets(items) => lines.extend(items.iter()),
                    Block::Swipe(rows) => lines.extend(rows.iter().map(|(_, m)| m)),
                    Block::Diagram(_) | Block::Image { .. } => {}
                }
            }
        }
        for line in lines {
            for run in tokens(line) {
                if let Inline::Icon(name) = run {
                    assert!(
                        KeyIcon::from_name(name).is_some(),
                        "unknown icon [[{name}]] in {line:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn the_search_haystack_drops_icon_tokens_and_backticks() {
        assert_eq!(
            prose("Tap the [[funnel]], then `Apply`."),
            "Tap the , then Apply."
        );
        let guide = find("swipe-to-build").expect("the swipe guide ships");
        let hay = haystack(guide);
        assert!(!hay.contains("[["), "{hay}");
        assert!(!hay.contains('`'), "{hay}");
        assert!(hay.contains("synergy"), "{hay}");
    }

    #[test]
    fn search_and_tag_narrow_the_same_list() {
        assert_eq!(hits(None, "  ").len(), GUIDES.len());
        let tagged = hits(Some("Commander"), "");
        assert_ne!(tagged.len(), 0, "a guide is tagged Commander");
        assert!(tagged.iter().all(|g| g.tags.contains(&"Commander")));
        let searched = hits(None, "  SYNERGY ");
        assert!(searched.iter().any(|g| g.slug == "swipe-to-build"));
        assert!(
            hits(None, "synergy zzzznothing").is_empty(),
            "every term must match"
        );
        assert!(hits(Some("Commander"), "synergy").len() <= tagged.len());
    }

    #[test]
    fn a_gallery_shot_captions_with_its_alt_when_it_has_no_caption() {
        let guide = GUIDES
            .iter()
            .find(|g| !shots(g).is_empty())
            .expect("a guide has screenshots");
        let blocks: Vec<_> = guide
            .blocks
            .iter()
            .filter_map(|b| match *b {
                Block::Image { file, alt, caption } => Some((file, alt, caption)),
                _ => None,
            })
            .collect();
        let got = shots(guide);
        assert_eq!(got.len(), blocks.len());
        for (shot, (file, alt, caption)) in got.iter().zip(blocks) {
            assert_eq!(shot.file, file);
            assert_eq!(shot.caption, caption.unwrap_or(alt));
        }
    }

    #[test]
    fn every_guide_tag_is_a_chip() {
        for g in GUIDES {
            for t in g.tags {
                assert!(TAGS.contains(t), "{}: tag {t:?} has no chip", g.slug);
            }
            assert_eq!(related(g).count(), g.related.len(), "{}", g.slug);
        }
    }

    #[test]
    fn the_gallery_steps_by_arrow_and_by_h_and_l_wrapping_at_the_ends() {
        assert_eq!(gallery_step("ArrowLeft"), Some(GalleryStep::Back));
        assert_eq!(gallery_step("l"), Some(GalleryStep::Forward));
        assert_eq!(gallery_step("H"), None, "Shift+H opens the shortcuts");
        assert_eq!(stepped(0, 3, GalleryStep::Back), 2);
        assert_eq!(stepped(2, 3, GalleryStep::Forward), 0);
        assert_eq!(stepped(1, 3, GalleryStep::Forward), 2);
        assert_eq!(stepped(0, 0, GalleryStep::Forward), 0);
        for key in GALLERY_KEYS {
            assert!(gallery_step(key).is_some(), "{key}");
        }
    }

    #[test]
    fn the_index_filter_round_trips_through_its_address() {
        let filter = IndexFilter {
            tag: Some("Deck stats"),
            query: "mana & 100% curve".to_owned(),
        };
        let search = filter.to_search();
        assert_eq!(search, "?tag=Deck+stats&q=mana+%26+100%25+curve");
        assert_eq!(IndexFilter::from_search(&search), filter);
        assert_eq!(IndexFilter::default().to_search(), "");
        assert_eq!(IndexFilter::from_search(""), IndexFilter::default());
        assert_eq!(
            IndexFilter::from_search("?tag=Nope&q=%E2%9C%93&x=1&q2"),
            IndexFilter {
                tag: None,
                query: "\u{2713}".to_owned()
            },
            "an unknown tag and other keys drop, UTF-8 decodes"
        );
        assert_eq!(
            IndexFilter::from_search("q=100%").query,
            "100%",
            "a broken escape stays"
        );
    }
}
