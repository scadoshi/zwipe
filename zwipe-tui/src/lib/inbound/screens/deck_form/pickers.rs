//! The format, deck tag and oracle tag pickers: a search box, a definition bar, a grid of chips, Back to revert and Save to keep.

use crossterm::event::KeyCode;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::oracle_tag::{self, OracleTag};
use zwipe_core::domain::deck::{DeckTagView, Format, MAX_DECK_ORACLE_TAGS, MAX_DECK_TAGS};

use super::{DeckForm, Overlay};
use crate::domain::deck::form::{self, Slot};
use crate::inbound::app::Context;
use crate::inbound::widgets::info::InfoPopup;
use crate::inbound::widgets::text_input::TextInput;
use crate::inbound::widgets::toast::ToastKind;

/// An oracle tag search shows at most this many hits.
const ORACLE_SEARCH_LIMIT: usize = 40;

/// The chips a key moves past with Page Up and Page Down.
const PAGE: usize = 10;

/// What the format picker reverts to on Back.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct FormatSnapshot {
    pub(crate) format: Option<Format>,
    pub(crate) commander: Option<Card>,
    pub(crate) signature_spell: Option<Card>,
}

/// The format picker.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct FormatPicker {
    pub(crate) search: TextInput,
    pub(crate) focus: usize,
    /// The format whose details fill the bar, once one is tapped.
    pub(crate) detail: Option<Format>,
    pub(crate) snapshot: FormatSnapshot,
}

impl FormatPicker {
    pub(crate) fn open(form: &DeckForm) -> Self {
        Self {
            search: TextInput::default().with_placeholder("Search formats"),
            focus: 0,
            detail: None,
            snapshot: FormatSnapshot {
                format: form.format,
                commander: form.card(Slot::Commander).card.clone(),
                signature_spell: form.card(Slot::SignatureSpell).card.clone(),
            },
        }
    }

    /// The formats matching the search, in the app's order.
    pub(crate) fn formats(&self) -> Vec<Format> {
        let query = self.search.value().to_lowercase();
        Format::all()
            .iter()
            .copied()
            .filter(|f| f.display_name().to_lowercase().contains(&query))
            .collect()
    }
}

/// The deck tag picker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TagPicker {
    pub(crate) search: TextInput,
    pub(crate) focus: usize,
    /// The tag whose description fills the bar, once one is tapped.
    pub(crate) detail: Option<String>,
    pub(crate) snapshot: Vec<String>,
}

impl TagPicker {
    pub(crate) fn open(form: &DeckForm) -> Self {
        Self {
            search: TextInput::default().with_placeholder("Search tags"),
            focus: 0,
            detail: None,
            snapshot: form.tags.clone(),
        }
    }

    /// The catalog's tags whose names match the search, in served order.
    pub(crate) fn tags<'catalog>(
        &self,
        catalog: &'catalog [DeckTagView],
    ) -> Vec<&'catalog DeckTagView> {
        let query = self.search.value().to_lowercase();
        catalog
            .iter()
            .filter(|t| t.display_name.to_lowercase().contains(&query))
            .collect()
    }
}

/// The oracle tag picker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OraclePicker {
    pub(crate) search: TextInput,
    pub(crate) focus: usize,
    pub(crate) detail: Option<String>,
    pub(crate) snapshot: Vec<String>,
}

impl OraclePicker {
    pub(crate) fn open(form: &DeckForm) -> Self {
        Self {
            search: TextInput::default().with_placeholder("Search all oracle tags"),
            focus: 0,
            detail: None,
            snapshot: form.oracle_tags.clone(),
        }
    }

    /// The grid: with no search, every curated tag plus every picked one, by slug; with one, the ranked hits, at most 40.
    pub(crate) fn slugs(&self, catalog: &[OracleTag], picked: &[String]) -> Vec<String> {
        let query = self.search.value();
        if query.is_empty() {
            let mut slugs: Vec<String> = catalog
                .iter()
                .filter(|t| t.curated)
                .map(|t| t.slug.clone())
                .chain(picked.iter().cloned())
                .collect();
            slugs.sort();
            slugs.dedup();
            slugs
        } else {
            oracle_tag::search_oracle_tags(catalog, query)
                .into_iter()
                .take(ORACLE_SEARCH_LIMIT)
                .map(|t| t.slug)
                .collect()
        }
    }
}

/// Moves a grid focus with the arrows and Page keys. Returns whether the key was one of them.
fn move_focus(focus: &mut usize, count: usize, code: KeyCode) -> bool {
    let last = count.saturating_sub(1);
    match code {
        KeyCode::Left | KeyCode::Up => *focus = focus.saturating_sub(1),
        KeyCode::Right | KeyCode::Down => *focus = focus.saturating_add(1).min(last),
        KeyCode::PageUp => *focus = focus.saturating_sub(PAGE),
        KeyCode::PageDown => *focus = focus.saturating_add(PAGE).min(last),
        KeyCode::Home => *focus = 0,
        KeyCode::End => *focus = last,
        _ => return false,
    }
    true
}

/// The hint each picker shows on F1.
fn hint(overlay: &str) -> InfoPopup {
    match overlay {
        "format" => InfoPopup::new(
            "Format",
            &[
                "Tap the picked format again to clear it",
                "Changing the format clears the commander and signature spell",
            ],
        ),
        "deck tags" => InfoPopup::new(
            "Deck tags",
            &[
                "The archetypes the deck is built around, like Aristocrats or Voltron",
                "Picking one pre-selects the oracle tags that define it",
                "Up to 5",
                "Tap a tag to add or remove it. The bar up top shows what it means",
            ],
        ),
        _ => InfoPopup::new(
            "Oracle tags",
            &[
                "The specific things the deck does, like spot removal, ramp or reanimation",
                "They shape which cards we suggest",
                "Your deck tags pre-pick a starter set from about 4,500. Fine to leave as is",
            ],
        ),
    }
}

/// Keys in the format picker. Returns the picker while it stays open, or the popup it opens.
pub(crate) fn handle_format(
    form: &mut DeckForm,
    mut picker: Box<FormatPicker>,
    code: KeyCode,
    cx: &mut Context<'_>,
) -> Option<Overlay> {
    match code {
        KeyCode::Esc => {
            let changed = form.format != picker.snapshot.format
                || form.card(Slot::Commander).card != picker.snapshot.commander
                || form.card(Slot::SignatureSpell).card != picker.snapshot.signature_spell;
            if changed {
                let snapshot = picker.snapshot.clone();
                form.set_format(snapshot.format);
                form.set_card(Slot::Commander, snapshot.commander);
                form.set_card(Slot::SignatureSpell, snapshot.signature_spell);
                cx.toast(ToastKind::Info, "Format unchanged".to_owned());
            }
            return None;
        }
        KeyCode::F(2) => {
            if form.format != picker.snapshot.format {
                return None;
            }
        }
        KeyCode::F(1) => return Some(Overlay::Info(hint("format"))),
        KeyCode::Enter => {
            if let Some(format) = picker.formats().get(picker.focus).copied() {
                picker.detail = Some(format);
                if form.format == Some(format) {
                    form.set_format(None);
                } else {
                    form.set_format(Some(format));
                }
            }
        }
        other => {
            let count = picker.formats().len();
            if !move_focus(&mut picker.focus, count, other) && picker.search.handle_key(other) {
                picker.focus = 0;
            }
        }
    }
    Some(Overlay::Format(picker))
}

/// Keys in the deck tag picker.
pub(crate) fn handle_tags(
    form: &mut DeckForm,
    mut picker: TagPicker,
    code: KeyCode,
    cx: &mut Context<'_>,
) -> Option<Overlay> {
    match code {
        KeyCode::Esc => {
            if !same_set(&form.tags, &picker.snapshot) {
                form.tags = picker.snapshot;
                cx.toast(ToastKind::Info, "Deck tags unchanged".to_owned());
            }
            form.reseed(cx);
            return None;
        }
        KeyCode::F(2) => {
            if !same_set(&form.tags, &picker.snapshot) {
                form.reseed(cx);
                return None;
            }
        }
        KeyCode::F(1) => return Some(Overlay::Info(hint("deck tags"))),
        KeyCode::F(8) => {
            form.tags.clear();
            picker.detail = None;
        }
        KeyCode::Enter => {
            let slug = picker
                .tags(&form.deck_tag_catalog)
                .get(picker.focus)
                .map(|t| t.slug.clone());
            if let Some(slug) = slug {
                picker.detail = Some(slug.clone());
                picker.search.set_value("");
                picker.focus = form
                    .deck_tag_catalog
                    .iter()
                    .position(|t| t.slug == slug)
                    .unwrap_or(0);
                if let Some(at) = form.tags.iter().position(|t| *t == slug) {
                    form.tags.remove(at);
                } else if form.tags.len() >= MAX_DECK_TAGS {
                    cx.toast(
                        ToastKind::Warning,
                        "You may only choose up to 5 tags".to_owned(),
                    );
                } else {
                    form.tags.push(slug);
                }
            }
        }
        other => {
            let count = picker.tags(&form.deck_tag_catalog).len();
            if !move_focus(&mut picker.focus, count, other) && picker.search.handle_key(other) {
                picker.focus = 0;
            }
        }
    }
    Some(Overlay::DeckTags(picker))
}

/// Keys in the oracle tag picker.
pub(crate) fn handle_oracle(
    form: &mut DeckForm,
    mut picker: OraclePicker,
    code: KeyCode,
    cx: &mut Context<'_>,
) -> Option<Overlay> {
    match code {
        KeyCode::Esc => {
            if !same_set(&form.oracle_tags, &picker.snapshot) {
                form.oracle_tags = picker.snapshot;
                cx.toast(ToastKind::Info, "Oracle tags unchanged".to_owned());
            }
            return None;
        }
        KeyCode::F(2) => {
            if !same_set(&form.oracle_tags, &picker.snapshot) {
                return None;
            }
        }
        KeyCode::F(1) => return Some(Overlay::Info(hint("oracle tags"))),
        KeyCode::F(3) => open_dictionary_stub(cx),
        KeyCode::F(8) => {
            form.oracle_tags.clear();
            picker.detail = None;
        }
        KeyCode::Enter => {
            let catalog = form.oracle_catalog.clone().unwrap_or_default();
            let slug = picker
                .slugs(&catalog, &form.oracle_tags)
                .get(picker.focus)
                .cloned();
            if let Some(slug) = slug {
                picker.detail = Some(slug.clone());
                picker.search.set_value("");
                if let Some(at) = form.oracle_tags.iter().position(|t| *t == slug) {
                    form.oracle_tags.remove(at);
                } else if form.oracle_tags.len() >= MAX_DECK_ORACLE_TAGS {
                    cx.toast(
                        ToastKind::Warning,
                        "You may only choose up to 30 oracle tags".to_owned(),
                    );
                } else {
                    form.oracle_tags.push(slug.clone());
                }
                picker.focus = picker
                    .slugs(&catalog, &form.oracle_tags)
                    .iter()
                    .position(|s| *s == slug)
                    .unwrap_or(0);
            }
        }
        other => {
            let catalog = form.oracle_catalog.as_deref().unwrap_or_default();
            let count = picker.slugs(catalog, &form.oracle_tags).len();
            if !move_focus(&mut picker.focus, count, other) && picker.search.handle_key(other) {
                picker.focus = 0;
            }
        }
    }
    Some(Overlay::OracleTags(picker))
}

/// Stand-in for the Oracle Tag Dictionary overlay, which package B builds. Wire it to that overlay; its "Use" adds a tag with the toasts in spec 3 section 2.7.
pub(crate) fn open_dictionary_stub(cx: &mut Context<'_>) {
    cx.toast(
        ToastKind::Info,
        "The oracle tag dictionary is not in the terminal yet".to_owned(),
    );
}

/// Whether two tag lists hold the same tags, ignoring order.
fn same_set(a: &[String], b: &[String]) -> bool {
    let mut a = a.to_vec();
    let mut b = b.to_vec();
    a.sort();
    b.sort();
    a == b
}

/// The lines the format picker's bar shows for `detail`.
pub(crate) fn format_bar(detail: Option<Format>) -> (String, Vec<String>) {
    match detail {
        Some(format) => (
            format.display_name().to_owned(),
            form::format_details(format),
        ),
        None => (
            "Hint".to_owned(),
            vec!["Tap a format to see its details here.".to_owned()],
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_format_search_ignores_case() {
        let mut picker = FormatPicker::open(&DeckForm::new(super::super::Mode::Create));
        assert_eq!(picker.formats().len(), Format::all().len());
        picker.search.set_value("BRAWL");
        let formats = picker.formats();
        let names: Vec<&str> = formats.iter().map(Format::display_name).collect();
        assert!(names.contains(&"Brawl"));
        assert!(names.iter().all(|n| n.to_lowercase().contains("brawl")));
    }

    #[test]
    fn tag_sets_compare_without_order() {
        let tags = |v: &[&str]| v.iter().map(|&s| s.to_owned()).collect::<Vec<_>>();
        assert!(same_set(&tags(&["a", "b"]), &tags(&["b", "a"])));
        assert!(!same_set(&tags(&["a"]), &tags(&["a", "b"])));
    }

    #[test]
    fn the_format_bar_starts_with_a_hint() {
        assert_eq!(
            format_bar(None),
            (
                "Hint".to_owned(),
                vec!["Tap a format to see its details here.".to_owned()]
            )
        );
        assert_eq!(format_bar(Some(Format::Commander)).0, "Commander");
    }
}
