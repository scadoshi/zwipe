//! Oracle tag reveal pieces, the app's `OtagDef` and `RoleTags`: a tag's definition with its Examples key, and a row of tag chips with one open and its definition beneath.
//!
//! Card rows use these inside an expanded card; Activity's and Playstyle's charts use [`RoleTags`] under a picked role. Descriptions come from the oracle tag catalog in `shared.caches`, so a screen that shows tags calls `oracle_tags::ensure_catalog` when it opens. Until the catalog loads, every tag reads "No description yet".
//!
//! [`RoleTags`] keys: Tab opens the next tag's definition and `BackTab` the previous one, and stepping past either end closes it; `x` opens the examples for the open tag.

use crossterm::event::KeyCode;
use ratatui::text::{Line, Span};

use super::action_bar;
use crate::domain::theme::Palette;
use crate::inbound::app::Context;
use crate::inbound::screens::home::cache::Catalog;
use crate::inbound::screens::otag_examples;

/// What a tag without a description reads.
pub(crate) const NO_DESCRIPTION: &str = "No description yet";

/// The tag's definition: its description from the catalog, or a muted [`NO_DESCRIPTION`], then the `x Examples` key.
pub(crate) fn def_lines(palette: &Palette, catalog: &Catalog, slug: &str) -> Vec<Line<'static>> {
    let description = match catalog.description(slug) {
        Some(description) => Line::from(description.to_owned()),
        None => Line::styled(NO_DESCRIPTION, palette.muted),
    };
    vec![description, action_bar::line(palette, &[("x", "Examples")])]
}

/// A tag as a chip: picked out in reverse while its definition is open.
pub(crate) fn chip(palette: &Palette, slug: &str, open: bool) -> Span<'static> {
    let style = if open {
        palette.accent.bold().reversed()
    } else {
        palette.accent
    };
    Span::styled(format!(" {slug} "), style)
}

/// The tag Tab opens after `open` among `count` tags: the first when none is open, none after the last.
pub(crate) fn next_open(open: Option<usize>, count: usize) -> Option<usize> {
    match open {
        None => (count > 0).then_some(0),
        Some(i) => Some(i.saturating_add(1)).filter(|&next| next < count),
    }
}

/// The tag `BackTab` opens before `open` among `count` tags: the last when none is open, none before the first.
pub(crate) fn previous_open(open: Option<usize>, count: usize) -> Option<usize> {
    match open {
        None => count.checked_sub(1),
        Some(i) => i.checked_sub(1).filter(|&previous| previous < count),
    }
}

/// A row of tag chips with at most one open, its definition shown beneath. Empty tags draw nothing.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct RoleTags {
    tags: Vec<String>,
    open: Option<usize>,
}

impl RoleTags {
    /// The chips for `tags`, none open.
    pub(crate) fn new(tags: Vec<String>) -> Self {
        Self { tags, open: None }
    }

    pub(crate) fn tags(&self) -> &[String] {
        &self.tags
    }

    /// The slug whose definition is open.
    pub(crate) fn open_tag(&self) -> Option<&str> {
        self.open.and_then(|i| self.tags.get(i)).map(String::as_str)
    }

    /// Swaps in other tags, keeping the open one open if it is among them.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "Activity and Playstyle build a fresh row with `RoleTags::new` when the tags change"
        )
    )]
    pub(crate) fn set_tags(&mut self, tags: Vec<String>) {
        let open = self.open_tag().map(str::to_owned);
        self.open = open.and_then(|slug| tags.iter().position(|t| *t == slug));
        self.tags = tags;
    }

    /// Opens the next tag's definition, closing it after the last.
    pub(crate) fn next(&mut self) {
        self.open = next_open(self.open, self.tags.len());
    }

    /// Opens the previous tag's definition, closing it before the first.
    pub(crate) fn previous(&mut self) {
        self.open = previous_open(self.open, self.tags.len());
    }

    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "Tab and BackTab close the row past either end; no screen closes it otherwise"
        )
    )]
    pub(crate) fn close(&mut self) {
        self.open = None;
    }

    /// Tab and `BackTab` move the open tag; `x` opens the open tag's examples. Returns whether the key was used.
    pub(crate) fn handle_key(&mut self, code: KeyCode, cx: &mut Context<'_>) -> bool {
        if self.tags.is_empty() {
            return false;
        }
        match code {
            KeyCode::Tab => self.next(),
            KeyCode::BackTab => self.previous(),
            KeyCode::Char('x') => {
                let Some(slug) = self.open_tag() else {
                    return false;
                };
                otag_examples::open(cx, slug);
            }
            _ => return false,
        }
        true
    }

    /// The chips on one line, then the open tag's definition. Nothing for no tags.
    pub(crate) fn lines(&self, palette: &Palette, catalog: &Catalog) -> Vec<Line<'static>> {
        if self.tags.is_empty() {
            return Vec::new();
        }
        let mut chips = Vec::new();
        for (i, tag) in self.tags.iter().enumerate() {
            if i > 0 {
                chips.push(Span::raw(" "));
            }
            chips.push(chip(palette, tag, self.open == Some(i)));
        }
        let mut lines = vec![Line::from(chips)];
        if let Some(slug) = self.open_tag() {
            lines.extend(def_lines(palette, catalog, slug));
        }
        lines
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use zwipe_core::domain::card::oracle_tag::OracleTag;

    use super::*;
    use crate::inbound::event_loop::harness::Harness;
    use crate::inbound::screens::Screen;

    fn catalog() -> Catalog {
        Catalog::Loaded {
            tags: vec![OracleTag {
                slug: "ramp".to_owned(),
                label: "Ramp".to_owned(),
                description: Some("Puts extra mana into play".to_owned()),
                parent_slugs: Vec::new(),
                curated: true,
            }],
            fetched_at: Instant::now(),
            refreshing: false,
        }
    }

    fn text(lines: &[Line<'static>]) -> Vec<String> {
        lines.iter().map(ToString::to_string).collect()
    }

    fn tags(slugs: &[&str]) -> RoleTags {
        RoleTags::new(slugs.iter().map(|&s| s.to_owned()).collect())
    }

    #[test]
    fn a_definition_reads_the_catalog_or_says_there_is_none() {
        let palette = Palette::terminal();
        assert_eq!(
            text(&def_lines(&palette, &catalog(), "ramp")),
            ["Puts extra mana into play", "x Examples  "]
        );
        assert_eq!(
            text(&def_lines(&palette, &catalog(), "anthem"))[0],
            "No description yet"
        );
        assert_eq!(
            text(&def_lines(&palette, &Catalog::Loading, "ramp"))[0],
            "No description yet"
        );
    }

    #[test]
    fn tab_walks_the_tags_and_closes_past_either_end() {
        assert_eq!(next_open(None, 0), None);
        assert_eq!(previous_open(None, 0), None);
        let mut row = tags(&["ramp", "removal"]);
        row.next();
        assert_eq!(row.open_tag(), Some("ramp"));
        row.next();
        assert_eq!(row.open_tag(), Some("removal"));
        row.next();
        assert_eq!(row.open_tag(), None);
        row.previous();
        assert_eq!(row.open_tag(), Some("removal"));
        row.close();
        assert_eq!(row.open_tag(), None);
    }

    #[test]
    fn new_tags_keep_the_open_one_when_it_is_still_there() {
        let mut row = tags(&["ramp", "removal"]);
        row.previous();
        row.set_tags(vec!["draw".to_owned(), "removal".to_owned()]);
        assert_eq!(row.open_tag(), Some("removal"));
        row.set_tags(vec!["draw".to_owned()]);
        assert_eq!(row.open_tag(), None);
        assert_eq!(row.tags(), ["draw"]);
    }

    #[test]
    fn lines_show_the_chips_and_the_open_definition() {
        let palette = Palette::terminal();
        assert!(
            RoleTags::default().lines(&palette, &catalog()).is_empty(),
            "no tags, no lines"
        );
        let mut row = tags(&["ramp", "removal"]);
        assert_eq!(text(&row.lines(&palette, &catalog())), [" ramp   removal "]);
        row.next();
        let lines = row.lines(&palette, &catalog());
        assert_eq!(text(&lines)[1], "Puts extra mana into play");
        assert!(
            lines[0].spans[0]
                .style
                .add_modifier
                .contains(ratatui::style::Modifier::REVERSED)
        );
    }

    #[test]
    fn x_opens_the_examples_only_for_an_open_tag() {
        let mut harness = Harness::on(Screen::Home(Box::default()));
        let mut row = tags(&["ramp"]);
        let mut used = Vec::new();
        harness.open(|cx| {
            used.push(row.handle_key(KeyCode::Char('x'), cx));
            used.push(row.handle_key(KeyCode::Tab, cx));
            used.push(row.handle_key(KeyCode::Char('x'), cx));
            used.push(row.handle_key(KeyCode::Char('q'), cx));
        });
        assert_eq!(used, [false, true, true, false]);
        let Screen::OtagExamples(view) = harness.screen() else {
            panic!("no examples");
        };
        assert_eq!(view.slug, "ramp");
    }
}
