//! The deck form's three pickers, format, deck tags and oracle tags, as the app has them: the draft each opens on, what its grid lists, the bar's words before a tap, its `?` dialog, every toast it raises, and the partner autofill a commander pick starts.
//!
//! A picker edits the form's value in place and keeps a [`Draft`] of what it opened on. Save stays off while the value matches the draft. Back puts the draft back, saying "{X} unchanged" when that throws a change away.

use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::oracle_tag::{self, OracleTag};
use zwipe_core::domain::deck::{DeckTagView, Format, MAX_DECK_ORACLE_TAGS, MAX_DECK_TAGS};

use crate::deck::form;
use crate::filter::fields;
use crate::hint::{Block, HintContent, HintTopic, Span};
use crate::timing;
use crate::toast::{Toast, ToastKind};

/// The definition bar's title before a chip is tapped, in every picker.
pub const PICKER_BAR_HINT: &str = "Hint";

/// The format picker's bar before a tap.
pub const FORMAT_BAR_EMPTY: &str = "Tap a format to see its details here.";

/// The deck tag picker's bar before a tap.
pub const DECK_TAG_BAR_EMPTY: &str = "Tap a tag to see its definition here.";

/// The oracle tag picker's bar before a tap.
pub const ORACLE_TAG_BAR_EMPTY: &str =
    "Tap a tag to see what it means. Picking a deck tag pre-selects some of these for you.";

/// The oracle tag picker's bar for a tag the catalog has no description for.
pub const ORACLE_TAG_NO_DESCRIPTION: &str = "No description for this tag.";

/// One of the deck form's pickers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Picker {
    /// The format picker: one pick, or none.
    Format,
    /// The deck tag picker, over the server's deck tag catalog.
    DeckTags,
    /// The oracle tag picker, over the oracle tag catalog.
    OracleTags,
}

impl Picker {
    /// The picker's header, which its `?` dialog shares.
    #[must_use]
    pub fn title(self) -> &'static str {
        match self {
            Self::Format => "Format",
            Self::DeckTags => "Deck tags",
            Self::OracleTags => "Oracle tags",
        }
    }

    /// The bar's text under [`PICKER_BAR_HINT`] before a chip is tapped.
    #[must_use]
    pub fn bar_empty(self) -> &'static str {
        match self {
            Self::Format => FORMAT_BAR_EMPTY,
            Self::DeckTags => DECK_TAG_BAR_EMPTY,
            Self::OracleTags => ORACLE_TAG_BAR_EMPTY,
        }
    }

    /// The toast Back raises when it throws a change away: `Format unchanged`, `Deck tags unchanged` or `Oracle tags unchanged`, info, up for [`QUICK`](crate::toast::QUICK).
    #[must_use]
    pub fn unchanged_toast(self) -> Toast {
        Toast::quick(ToastKind::Info, format!("{} unchanged", self.title()))
    }

    /// The picker's `?` dialog. The deck tag and oracle tag pickers open with the field's explainer ([`HintTopic::DeckTags`], [`HintTopic::OracleTags`]); the deck tag picker then says how its chips work. The oracle tag dialog's `Dictionary` action is the client's to add.
    #[must_use]
    pub fn hint(self) -> HintContent {
        let blocks = match self {
            Self::Format => vec![
                bullet("Tap the picked format again to clear it"),
                bullet("Changing the format clears the commander and signature spell"),
            ],
            Self::DeckTags => {
                let mut blocks = HintTopic::DeckTags.blocks();
                blocks.push(bullet(
                    "Tap a tag to add or remove it. The bar up top shows what it means",
                ));
                blocks
            }
            Self::OracleTags => HintTopic::OracleTags.blocks(),
        };
        HintContent {
            title: self.title().to_owned(),
            blocks,
        }
    }
}

/// One of the two tag pickers, which hold slugs up to a cap.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TagPicker {
    /// Deck tags, up to [`MAX_DECK_TAGS`].
    DeckTags,
    /// Oracle tags, up to [`MAX_DECK_ORACLE_TAGS`].
    OracleTags,
}

impl From<TagPicker> for Picker {
    fn from(picker: TagPicker) -> Self {
        match picker {
            TagPicker::DeckTags => Self::DeckTags,
            TagPicker::OracleTags => Self::OracleTags,
        }
    }
}

impl TagPicker {
    /// The most tags a deck takes from this picker.
    #[must_use]
    pub fn cap(self) -> usize {
        match self {
            Self::DeckTags => MAX_DECK_TAGS,
            Self::OracleTags => MAX_DECK_ORACLE_TAGS,
        }
    }

    /// The toast a tap past the cap raises: `You may only choose up to 5 tags` or `You may only choose up to 30 oracle tags`, a warning up for [`QUICK`](crate::toast::QUICK).
    #[must_use]
    pub fn cap_toast(self) -> Toast {
        let text = match self {
            Self::DeckTags => format!("You may only choose up to {MAX_DECK_TAGS} tags"),
            Self::OracleTags => {
                format!("You may only choose up to {MAX_DECK_ORACLE_TAGS} oracle tags")
            }
        };
        Toast::quick(ToastKind::Warning, text)
    }

    /// Taps `slug`'s chip: removes it when picked, adds it while there is room, and otherwise leaves `picked` alone. At the cap, raise [`cap_toast`](Self::cap_toast).
    #[must_use]
    pub fn toggle(self, picked: &mut Vec<String>, slug: &str) -> Tapped {
        if let Some(at) = picked.iter().position(|s| s == slug) {
            picked.remove(at);
            Tapped::Removed
        } else if picked.len() < self.cap() {
            picked.push(slug.to_owned());
            Tapped::Added
        } else {
            Tapped::AtCap
        }
    }
}

/// What a chip tap in a tag picker did to the picked list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tapped {
    /// The tag joined the list.
    Added,
    /// The tag came off the list.
    Removed,
    /// The list is at the picker's cap, so nothing changed.
    AtCap,
}

/// A value a picker edits, compared the way its Save and Back compare it.
pub trait Selection: Clone {
    /// Whether `self` and `other` are the same pick.
    fn same(&self, other: &Self) -> bool;
}

impl Selection for Option<Format> {
    fn same(&self, other: &Self) -> bool {
        self == other
    }
}

/// Slugs compare as sets: picking the same tags in another order is no change.
impl Selection for Vec<String> {
    fn same(&self, other: &Self) -> bool {
        fields::same_set(self, other)
    }
}

/// What a picker opened on. The form keeps the live value; the draft keeps the snapshot, so Save knows whether anything moved and Back knows what to put back.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Draft<T> {
    picker: Picker,
    snapshot: T,
}

impl Draft<Option<Format>> {
    /// The format picker's draft, opened on `current`.
    #[must_use]
    pub fn format(current: Option<Format>) -> Self {
        Self {
            picker: Picker::Format,
            snapshot: current,
        }
    }
}

impl Draft<Vec<String>> {
    /// A tag picker's draft, opened on the slugs in `current`.
    #[must_use]
    pub fn tags(picker: TagPicker, current: Vec<String>) -> Self {
        Self {
            picker: picker.into(),
            snapshot: current,
        }
    }
}

impl<T: Selection> Draft<T> {
    /// The picker this draft belongs to.
    #[must_use]
    pub fn picker(&self) -> Picker {
        self.picker
    }

    /// The value the picker opened on, which Back puts back.
    #[must_use]
    pub fn snapshot(&self) -> &T {
        &self.snapshot
    }

    /// The value the picker opened on, taken out of the draft.
    #[must_use]
    pub fn into_snapshot(self) -> T {
        self.snapshot
    }

    /// Whether `current` is still what the picker opened on, which keeps Save off.
    #[must_use]
    pub fn is_unchanged(&self, current: &T) -> bool {
        current.same(&self.snapshot)
    }

    /// The toast Back raises with `current` in the form: [`Picker::unchanged_toast`] when Back throws a change away, nothing when there is none.
    #[must_use]
    pub fn back_toast(&self, current: &T) -> Option<Toast> {
        (!self.is_unchanged(current)).then(|| self.picker.unchanged_toast())
    }
}

/// The formats whose names hold `query`, ignoring case, in [`Format::all`]'s order. An empty query lists every format.
#[must_use]
pub fn formats_matching(query: &str) -> Vec<Format> {
    let query = query.to_lowercase();
    Format::all()
        .iter()
        .copied()
        .filter(|f| query.is_empty() || f.display_name().to_lowercase().contains(&query))
        .collect()
}

/// The deck tags whose names hold `query`, ignoring case, in the catalog's order. An empty query lists the whole catalog.
#[must_use]
pub fn deck_tags_matching<'catalog>(
    catalog: &'catalog [DeckTagView],
    query: &str,
) -> Vec<&'catalog DeckTagView> {
    let query = query.to_lowercase();
    catalog
        .iter()
        .filter(|t| query.is_empty() || t.display_name.to_lowercase().contains(&query))
        .collect()
}

/// The oracle tag picker's grid. With a blank query (spaces count as blank): every curated tag plus every picked one the catalog serves, by slug. A picked slug the catalog no longer serves is not shown. Otherwise the catalog's ranked matches, at most [`timing::ORACLE_TAG_PICKER_LIMIT`].
#[must_use]
pub fn oracle_tag_results(catalog: &[OracleTag], picked: &[String], query: &str) -> Vec<OracleTag> {
    if query.trim().is_empty() {
        let mut curated: Vec<OracleTag> = catalog
            .iter()
            .filter(|t| t.curated || picked.contains(&t.slug))
            .cloned()
            .collect();
        curated.sort_by(|a, b| a.slug.cmp(&b.slug));
        curated
    } else {
        let mut matches = oracle_tag::search_oracle_tags(catalog, query);
        matches.truncate(timing::ORACLE_TAG_PICKER_LIMIT);
        matches
    }
}

/// What a tag the oracle tag dictionary's `Use` hands back does to the deck.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Adopted {
    /// The tag joined the deck.
    Added,
    /// The deck already had it.
    AlreadyAdded,
    /// The deck is at [`MAX_DECK_ORACLE_TAGS`], so it stays out.
    AtCap,
}

impl Adopted {
    /// The toast it raises, up for [`QUICK`](crate::toast::QUICK): `Tag added to deck` (success), `Tag already added` (info) or the oracle tag cap (warning).
    #[must_use]
    pub fn toast(self) -> Toast {
        match self {
            Self::Added => Toast::quick(ToastKind::Success, "Tag added to deck"),
            Self::AlreadyAdded => Toast::quick(ToastKind::Info, "Tag already added"),
            Self::AtCap => TagPicker::OracleTags.cap_toast(),
        }
    }
}

/// Adds the oracle tag `slug` the dictionary's `Use` hands back, unless the deck has it or is full. Unlike a chip tap, it never removes a tag.
#[must_use]
pub fn adopt_oracle_tag(picked: &mut Vec<String>, slug: String) -> Adopted {
    if picked.contains(&slug) {
        Adopted::AlreadyAdded
    } else if picked.len() < MAX_DECK_ORACLE_TAGS {
        picked.push(slug);
        Adopted::Added
    } else {
        Adopted::AtCap
    }
}

/// What a named partner search's results do to the form.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PartnerAutofill<'hits> {
    /// No result is the named partner.
    NoMatch,
    /// The commander changed while the search ran, or a partner is already picked, so nothing is filled.
    Stale,
    /// The partner slot takes this card, and [`partner_found_toast`] says so.
    Fill(&'hits Card),
}

/// Settles a "Partner with" commander's search for its mate. `hits` are the results for the mate's name `mate`; `picked` is the printing id of the commander the search started for; `commander` and `partner` are the form's slots now. The slot fills only while the same commander is chosen and the partner is empty, so a late reply never overwrites a newer pick.
///
/// Start the search only from a commander the user picks, never when a saved deck loads its commander, or every open of a partner deck would toast.
#[must_use]
pub fn partner_autofill<'hits>(
    hits: &'hits [Card],
    mate: &str,
    picked: Uuid,
    commander: Option<&Card>,
    partner: Option<&Card>,
) -> PartnerAutofill<'hits> {
    let Some(found) = form::named_partner(hits, mate) else {
        return PartnerAutofill::NoMatch;
    };
    let same_commander = commander.is_some_and(|c| c.scryfall_data.id == picked);
    if same_commander && partner.is_none() {
        PartnerAutofill::Fill(found)
    } else {
        PartnerAutofill::Stale
    }
}

/// The toast after the partner autofill fills the slot with `name`: `Partner found and selected: {name}`, info, up for [`NORMAL`](crate::toast::NORMAL).
#[must_use]
pub fn partner_found_toast(name: &str) -> Toast {
    Toast::normal(
        ToastKind::Info,
        format!("Partner found and selected: {name}"),
    )
}

/// A bullet of plain text.
fn bullet(text: &str) -> Block {
    Block::Bullet(vec![Span::Text(text.to_owned())])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::card;
    use crate::toast::{NORMAL, QUICK};

    fn slugs(values: &[&str]) -> Vec<String> {
        values.iter().map(|&s| s.to_owned()).collect()
    }

    fn tag(slug: &str, curated: bool) -> OracleTag {
        OracleTag {
            slug: slug.to_owned(),
            label: slug.to_owned(),
            description: None,
            parent_slugs: Vec::new(),
            curated,
        }
    }

    #[test]
    fn a_tag_draft_turns_save_on_with_a_change_and_off_again_on_revert() {
        let opened = slugs(&["ramp", "removal"]);
        let draft = Draft::tags(TagPicker::OracleTags, opened.clone());
        let mut picked = opened;
        assert!(draft.is_unchanged(&picked), "just opened");
        assert_eq!(draft.back_toast(&picked), None, "Back with no change");

        assert_eq!(
            TagPicker::OracleTags.toggle(&mut picked, "draw"),
            Tapped::Added
        );
        assert!(!draft.is_unchanged(&picked));
        assert_eq!(
            draft.back_toast(&picked),
            Some(Toast::new(ToastKind::Info, "Oracle tags unchanged", QUICK)),
        );

        assert_eq!(
            TagPicker::OracleTags.toggle(&mut picked, "draw"),
            Tapped::Removed
        );
        assert!(draft.is_unchanged(&picked), "tapped back to the snapshot");
        assert_eq!(draft.snapshot(), &slugs(&["ramp", "removal"]));
    }

    #[test]
    fn the_same_tags_in_another_order_are_no_change() {
        let draft = Draft::tags(TagPicker::DeckTags, slugs(&["a", "b"]));
        assert!(draft.is_unchanged(&slugs(&["b", "a"])));
        assert_eq!(draft.picker(), Picker::DeckTags);
    }

    #[test]
    fn a_format_draft_compares_the_format_alone() {
        let draft = Draft::format(Some(Format::Commander));
        assert!(draft.is_unchanged(&Some(Format::Commander)));
        assert_eq!(
            draft.back_toast(&None),
            Some(Toast::new(ToastKind::Info, "Format unchanged", QUICK)),
        );
        assert_eq!(
            draft.back_toast(&Some(Format::Modern)).map(|t| t.text),
            Some("Format unchanged".to_owned()),
        );
        assert_eq!(draft.into_snapshot(), Some(Format::Commander));
    }

    #[test]
    fn every_picker_names_itself_in_its_back_toast() {
        assert_eq!(
            Picker::DeckTags.unchanged_toast().text,
            "Deck tags unchanged"
        );
        assert_eq!(
            Picker::OracleTags.unchanged_toast().text,
            "Oracle tags unchanged"
        );
    }

    #[test]
    fn a_tap_past_the_cap_warns_and_changes_nothing() {
        let mut picked: Vec<String> = (0..MAX_DECK_TAGS).map(|i| format!("t{i}")).collect();
        let before = picked.clone();
        assert_eq!(
            TagPicker::DeckTags.toggle(&mut picked, "one-more"),
            Tapped::AtCap
        );
        assert_eq!(
            TagPicker::DeckTags.cap_toast(),
            Toast::new(
                ToastKind::Warning,
                format!("You may only choose up to {MAX_DECK_TAGS} tags"),
                QUICK,
            ),
        );
        assert_eq!(picked, before);
        assert_eq!(
            TagPicker::DeckTags.toggle(&mut picked, "t0"),
            Tapped::Removed
        );
        assert_eq!(
            picked.len(),
            MAX_DECK_TAGS - 1,
            "a picked tag still comes off"
        );
    }

    #[test]
    fn the_oracle_tag_cap_says_oracle_tags() {
        let mut picked: Vec<String> = (0..MAX_DECK_ORACLE_TAGS).map(|i| format!("t{i}")).collect();
        assert_eq!(
            TagPicker::OracleTags.toggle(&mut picked, "x"),
            Tapped::AtCap
        );
        let toast = TagPicker::OracleTags.cap_toast();
        assert_eq!(
            toast.text,
            format!("You may only choose up to {MAX_DECK_ORACLE_TAGS} oracle tags"),
        );
        assert_eq!((toast.kind, toast.duration), (ToastKind::Warning, QUICK));
    }

    #[test]
    fn the_dictionary_adds_once_and_stops_at_the_cap() {
        let mut picked = slugs(&["ramp"]);
        assert_eq!(
            adopt_oracle_tag(&mut picked, "draw".to_owned()),
            Adopted::Added
        );
        assert_eq!(
            adopt_oracle_tag(&mut picked, "draw".to_owned()),
            Adopted::AlreadyAdded
        );
        assert_eq!(picked, slugs(&["ramp", "draw"]));
        let mut full: Vec<String> = (0..MAX_DECK_ORACLE_TAGS).map(|i| format!("t{i}")).collect();
        assert_eq!(adopt_oracle_tag(&mut full, "x".to_owned()), Adopted::AtCap);
        assert_eq!(full.len(), MAX_DECK_ORACLE_TAGS);
        assert_eq!(
            Adopted::Added.toast(),
            Toast::new(ToastKind::Success, "Tag added to deck", QUICK)
        );
        assert_eq!(
            Adopted::AlreadyAdded.toast(),
            Toast::new(ToastKind::Info, "Tag already added", QUICK)
        );
        assert_eq!(Adopted::AtCap.toast(), TagPicker::OracleTags.cap_toast());
    }

    #[test]
    fn a_blank_search_shows_curated_and_picked_tags_by_slug() {
        let catalog = vec![
            tag("zombies", true),
            tag("lifegain", false),
            tag("aristocrats", true),
            tag("mill", false),
        ];
        let picked = slugs(&["lifegain", "gone-from-the-catalog"]);
        let grid: Vec<String> = oracle_tag_results(&catalog, &picked, "   ")
            .into_iter()
            .map(|t| t.slug)
            .collect();
        assert_eq!(
            grid,
            slugs(&["aristocrats", "lifegain", "zombies"]),
            "spaces are blank, and a slug the catalog dropped is not shown, as the app does"
        );
    }

    #[test]
    fn a_search_ranks_the_whole_catalog_up_to_the_limit() {
        let catalog: Vec<OracleTag> = (0..60)
            .map(|i| tag(&format!("ramp-{i:02}"), false))
            .collect();
        let hits = oracle_tag_results(&catalog, &[], "ramp");
        assert_eq!(hits.len(), timing::ORACLE_TAG_PICKER_LIMIT);
        assert_eq!(hits, {
            let mut ranked = oracle_tag::search_oracle_tags(&catalog, "ramp");
            ranked.truncate(timing::ORACLE_TAG_PICKER_LIMIT);
            ranked
        });
    }

    #[test]
    fn the_format_search_ignores_case_and_keeps_the_order() {
        assert_eq!(formats_matching(""), Format::all().to_vec());
        let brawl = formats_matching("BRAWL");
        assert!(brawl.contains(&Format::Brawl));
        assert!(
            brawl
                .iter()
                .all(|f| f.display_name().to_lowercase().contains("brawl"))
        );
    }

    #[test]
    fn the_deck_tag_search_matches_names() {
        let view = |slug: &str, name: &str| DeckTagView {
            slug: slug.to_owned(),
            display_name: name.to_owned(),
            description: String::new(),
            seed_otags: Vec::new(),
        };
        let catalog = vec![
            view("aristocrats", "Aristocrats"),
            view("voltron", "Voltron"),
        ];
        let hits = deck_tags_matching(&catalog, "VOL");
        assert_eq!(hits, vec![&catalog[1]]);
        assert_eq!(deck_tags_matching(&catalog, "").len(), 2);
    }

    #[test]
    fn the_bars_and_hints_read_as_the_app_has_them() {
        assert_eq!(Picker::Format.bar_empty(), FORMAT_BAR_EMPTY);
        assert_eq!(Picker::OracleTags.bar_empty(), ORACLE_TAG_BAR_EMPTY);
        assert_eq!(
            Picker::Format.hint().plain(),
            "Tap the picked format again to clear it\nChanging the format clears the commander and signature spell"
        );
        let deck_tags = Picker::DeckTags.hint();
        assert_eq!(deck_tags.title, "Deck tags");
        assert_eq!(deck_tags.blocks[..3], HintTopic::DeckTags.blocks()[..]);
        assert_eq!(
            deck_tags.blocks[3].plain(),
            "Tap a tag to add or remove it. The bar up top shows what it means"
        );
        assert_eq!(Picker::OracleTags.hint(), HintTopic::OracleTags.content());
    }

    #[test]
    fn the_partner_fills_only_for_the_same_commander_and_an_empty_slot() {
        let commander = card("Pir, Imaginative Rascal");
        let other = card("Someone Else");
        let hits = vec![card("Toothy's Pal"), card("Toothy, Imaginary Friend")];
        let picked = commander.scryfall_data.id;
        let fill = |commander: Option<&Card>, partner: Option<&Card>| {
            partner_autofill(
                &hits,
                "toothy, imaginary friend",
                picked,
                commander,
                partner,
            )
        };
        assert_eq!(
            fill(Some(&commander), None),
            PartnerAutofill::Fill(&hits[1])
        );
        assert_eq!(
            fill(Some(&other), None),
            PartnerAutofill::Stale,
            "commander changed"
        );
        assert_eq!(
            fill(None, None),
            PartnerAutofill::Stale,
            "commander cleared"
        );
        assert_eq!(fill(Some(&commander), Some(&other)), PartnerAutofill::Stale);
        assert_eq!(
            partner_autofill(&hits, "Nobody", picked, Some(&commander), None),
            PartnerAutofill::NoMatch
        );
        assert_eq!(
            partner_found_toast("Toothy, Imaginary Friend"),
            Toast::new(
                ToastKind::Info,
                "Partner found and selected: Toothy, Imaginary Friend",
                NORMAL
            )
        );
    }
}
