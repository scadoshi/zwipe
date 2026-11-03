//! The format, deck tag and oracle tag pickers: a search box, a definition bar, a grid of chips, Back to put back what the picker opened on and Save to keep the pick. The model holds the pick and its draft; a picker here keeps only its search box, the focused chip and the chip its bar defines.

use crossterm::event::KeyCode;
use zwipe_core::domain::card::oracle_tag::OracleTag;
use zwipe_core::domain::deck::{DeckTagView, Format};
use zwipe_ui::deck::form;
use zwipe_ui::deck::pickers::{self, Picker, TagPicker};
use zwipe_ui::screens::deck_form::Action;

use super::{DeckForm, Overlay, dispatch};
use crate::inbound::app::Context;
use crate::inbound::screens::oracle_tags;
use crate::inbound::widgets::hint;
use crate::inbound::widgets::text_input::TextInput;

/// The chips a key moves past with Page Up and Page Down.
const PAGE: usize = 10;

/// The format list over the form: a search box, the focused chip, and the format last tapped, whose details fill the bar.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct FormatPicker {
    pub(crate) search: TextInput,
    pub(crate) focus: usize,
    pub(crate) detail: Option<Format>,
}

impl FormatPicker {
    /// The formats matching the search, in the app's order.
    pub(crate) fn formats(&self) -> Vec<Format> {
        pickers::formats_matching(self.search.value())
    }
}

/// A tag picker's search box, the focused chip, and the tag last tapped, whose definition fills the bar.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct TagGrid {
    pub(crate) search: TextInput,
    pub(crate) focus: usize,
    pub(crate) detail: Option<String>,
}

impl TagGrid {
    /// The deck tags whose names match the search, in served order.
    pub(crate) fn deck_tags<'catalog>(
        &self,
        catalog: &'catalog [DeckTagView],
    ) -> Vec<&'catalog DeckTagView> {
        pickers::deck_tags_matching(catalog, self.search.value())
    }

    /// The oracle tag grid: with no search, every curated tag plus every picked one; with one, the ranked matches.
    pub(crate) fn oracle_tags(&self, catalog: &[OracleTag], picked: &[String]) -> Vec<OracleTag> {
        pickers::oracle_tag_results(catalog, picked, self.search.value())
    }

    /// The slugs `which` lists, in grid order.
    fn slugs(&self, form: &DeckForm, which: TagPicker) -> Vec<String> {
        match which {
            TagPicker::DeckTags => self
                .deck_tags(&form.deck_tag_catalog)
                .into_iter()
                .map(|t| t.slug.clone())
                .collect(),
            TagPicker::OracleTags => self
                .oracle_tags(
                    form.oracle_catalog.as_deref().unwrap_or_default(),
                    form.model.tags(TagPicker::OracleTags),
                )
                .into_iter()
                .map(|t| t.slug)
                .collect(),
        }
    }
}

/// Opens `picker` on the form's value.
pub(crate) fn open(form: &mut DeckForm, picker: Picker, cx: &mut Context<'_>) {
    dispatch(form, Action::OpenPicker(picker), cx);
    form.overlay = Some(match picker {
        Picker::Format => Overlay::Format(FormatPicker::default()),
        Picker::DeckTags => Overlay::Tags(TagPicker::DeckTags, TagGrid::default()),
        Picker::OracleTags => Overlay::Tags(TagPicker::OracleTags, TagGrid::default()),
    });
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

/// Esc, F2 and F6, which every picker shares: Back, Save while the pick moved, and the picker's hint. Returns `None` once the picker closes, or `Some(handled)`.
fn shared_key(
    form: &mut DeckForm,
    picker: Picker,
    code: KeyCode,
    cx: &mut Context<'_>,
) -> Option<bool> {
    match code {
        KeyCode::Esc => {
            dispatch(form, Action::BackPicker, cx);
            None
        }
        KeyCode::F(2) if !form.model.picker_unchanged() => {
            dispatch(form, Action::SavePicker, cx);
            None
        }
        KeyCode::F(2) => Some(true),
        KeyCode::F(6) => {
            hint::open(cx, picker.hint());
            Some(true)
        }
        _ => Some(false),
    }
}

/// Keys in the format picker. Returns the picker while it stays open.
pub(crate) fn handle_format(
    form: &mut DeckForm,
    mut picker: FormatPicker,
    code: KeyCode,
    cx: &mut Context<'_>,
) -> Option<Overlay> {
    if shared_key(form, Picker::Format, code, cx)? {
        return Some(Overlay::Format(picker));
    }
    if code == KeyCode::Enter {
        if let Some(format) = picker.formats().get(picker.focus).copied() {
            picker.detail = Some(format);
            dispatch(form, Action::TapFormat(format), cx);
        }
    } else {
        let count = picker.formats().len();
        if !move_focus(&mut picker.focus, count, code) && picker.search.handle_key(code) {
            picker.focus = 0;
        }
    }
    Some(Overlay::Format(picker))
}

/// Keys in a tag picker. F8 empties the pick and, in the oracle tag picker, F3 opens the dictionary. Returns the picker while it stays open.
pub(crate) fn handle_tags(
    form: &mut DeckForm,
    which: TagPicker,
    mut grid: TagGrid,
    code: KeyCode,
    cx: &mut Context<'_>,
) -> Option<Overlay> {
    if shared_key(form, which.into(), code, cx)? {
        return Some(Overlay::Tags(which, grid));
    }
    match code {
        KeyCode::F(8) => {
            dispatch(form, Action::ClearTags(which), cx);
            grid.detail = None;
        }
        KeyCode::F(3) if which == TagPicker::OracleTags => oracle_tags::open(cx),
        KeyCode::Enter => {
            if let Some(slug) = grid.slugs(form, which).get(grid.focus).cloned() {
                grid.detail = Some(slug.clone());
                grid.search.set_value("");
                dispatch(
                    form,
                    Action::ToggleTag {
                        picker: which,
                        slug: slug.clone(),
                    },
                    cx,
                );
                grid.focus = grid
                    .slugs(form, which)
                    .iter()
                    .position(|s| *s == slug)
                    .unwrap_or(0);
            }
        }
        other => {
            let count = grid.slugs(form, which).len();
            if !move_focus(&mut grid.focus, count, other) && grid.search.handle_key(other) {
                grid.focus = 0;
            }
        }
    }
    Some(Overlay::Tags(which, grid))
}

/// The format picker's bar for `detail`: the format's details once one is tapped, else the hint.
pub(crate) fn format_bar(detail: Option<Format>) -> (String, Vec<String>) {
    match detail {
        Some(format) => (
            format.display_name().to_owned(),
            form::format_details(format),
        ),
        None => (
            pickers::PICKER_BAR_HINT.to_owned(),
            vec![Picker::Format.bar_empty().to_owned()],
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
