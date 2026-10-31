//! The card filter: the sheet's Apply decision, the Add screen's deck-context defaults, and the oracle tag dictionary's hand-off, ported from the app.

pub(crate) mod fields;
pub(crate) mod picker;
pub(crate) mod sections;

use zwipe_core::domain::card::scryfall_data::colors::Colors;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::card::search_card::card_filter::error::InvalidCardCriteria;
use zwipe_core::domain::card::search_card::card_type::CardType;
use zwipe_core::domain::deck::Format;

/// What Apply does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ApplyAction {
    /// Commit the draft. `cleared` means it empties a filter that was on, which only changes the toast.
    Commit { cleared: bool },
    /// Refuse: the draft narrows nothing and neither did the filter it replaces.
    RefuseEmpty,
    /// The draft is what was applied: close without refetching, keeping the user's place.
    NoChange,
}

/// Whether a filter narrows the search.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FilterState {
    Empty,
    Set,
}

impl FilterState {
    /// `Set` when the filter has search intent.
    pub(crate) fn of(b: &CardQueryBuilder) -> Self {
        if b.has_search_intent() {
            Self::Set
        } else {
            Self::Empty
        }
    }
}

/// Decides Apply from the filter that was on when the sheet opened (`from`) and the draft (`to`). Judging the transition, not `to` alone, lets a deliberate clear through.
pub(crate) fn apply_action(
    validate_before_apply: bool,
    from: FilterState,
    to: FilterState,
    unchanged: bool,
) -> ApplyAction {
    if validate_before_apply && from == FilterState::Empty && to == FilterState::Empty {
        return ApplyAction::RefuseEmpty;
    }
    if unchanged {
        return ApplyAction::NoChange;
    }
    ApplyAction::Commit {
        cleared: from == FilterState::Set && to == FilterState::Empty,
    }
}

/// The toast for a draft that includes and excludes the same value, or `None` when it has no contradiction.
pub(crate) fn contradiction(b: &CardQueryBuilder) -> Option<String> {
    match b.build() {
        Err(InvalidCardCriteria::Contradiction { field, .. }) => {
            Some(format!("Filter can't both include and exclude {field}"))
        }
        _ => None,
    }
}

/// What the Filter key's dot means on the deck cards view and Remove: a filter or a sort is on.
pub(crate) fn shows_dot(b: &CardQueryBuilder) -> bool {
    !b.is_empty() || b.sort().is_some()
}

/// The deck facts Add's search filter is built from.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct AddContext {
    pub(crate) format: Option<Format>,
    /// The commander's combined color identity, when the format checks it.
    pub(crate) colors: Option<Colors>,
    pub(crate) has_commander: bool,
    /// Whether the main board has reached its land target.
    pub(crate) lands_at_target: bool,
}

impl AddContext {
    /// Whether Add's Filter key shows its dot: a user filter or a sort, not counting deck legality, commander identity, synergy or the automatic land exclusion.
    pub(crate) fn shows_dot(&self, b: &CardQueryBuilder) -> bool {
        !b.is_empty_ignoring_deck_context_and_auto_lands(self.lands_at_target) || b.sort().is_some()
    }

    /// Whether Add serves cards for this filter: a commander deck with a commander always, else only with search intent.
    pub(crate) fn serves(&self, b: &CardQueryBuilder) -> bool {
        let commander_format = self.format.is_some_and(|f| f.has_commander());
        (commander_format && self.has_commander) || b.has_search_intent()
    }

    /// Puts the deck's legality, commander identity and land exclusion back on `b`.
    pub(crate) fn apply_deck_context(&self, b: &mut CardQueryBuilder) {
        if let Some(format) = self.format {
            b.set_legalities_contains_any(vec![format.to_legality_key().to_owned()]);
        }
        if let Some(colors) = &self.colors {
            b.set_color_identity_within(colors.clone());
        }
        if self.lands_at_target {
            ensure_lands_excluded(b);
        }
    }

    /// The fresh Search filter: deck context, and synergy on with a commander.
    pub(crate) fn fresh_search(&self) -> CardQueryBuilder {
        let mut b = CardQueryBuilder::default();
        self.apply_deck_context(&mut b);
        b.set_synergy(self.has_commander);
        b
    }

    /// Reset on Search: back to the deck-context default, keeping synergy. `Err` when it already is there.
    ///
    /// # Errors
    ///
    /// Returns [`AlreadyDefault`] when nothing but deck context is set and there is no sort.
    pub(crate) fn reset_search(&self, b: &mut CardQueryBuilder) -> Result<(), AlreadyDefault> {
        let at_default = b.is_empty_ignoring_deck_context_and_auto_lands(self.lands_at_target);
        if at_default && b.sort().is_none() {
            return Err(AlreadyDefault);
        }
        let synergy = b.synergy();
        b.clear();
        self.apply_deck_context(b);
        b.set_synergy(synergy);
        Ok(())
    }
}

/// Reset on Add's maybeboard: back to blank, keeping synergy. `Err` when it already is blank.
///
/// # Errors
///
/// Returns [`AlreadyDefault`] when the filter is empty and there is no sort.
pub(crate) fn reset_maybeboard(b: &mut CardQueryBuilder) -> Result<(), AlreadyDefault> {
    if b.is_empty() && b.sort().is_none() {
        return Err(AlreadyDefault);
    }
    let synergy = b.synergy();
    b.clear();
    b.set_synergy(synergy);
    Ok(())
}

/// The filter already is at its default, so Reset has nothing to do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AlreadyDefault;

/// Drops Land from the type includes and adds it to the type excludes, so an include never clashes with the automatic exclusion.
pub(crate) fn ensure_lands_excluded(b: &mut CardQueryBuilder) {
    let without_land = |types: Option<&[CardType]>| -> Option<Vec<CardType>> {
        types.map(|t| t.iter().copied().filter(|t| *t != CardType::Land).collect())
    };
    if let Some(types) = without_land(b.card_type_contains_any()) {
        b.set_card_type_contains_any(types);
    }
    if let Some(types) = without_land(b.card_type_contains_all()) {
        b.set_card_type_contains_all(types);
    }
    let mut excludes = b
        .card_type_excludes_any()
        .map(<[_]>::to_vec)
        .unwrap_or_default();
    if !excludes.contains(&CardType::Land) {
        excludes.push(CardType::Land);
    }
    b.set_card_type_excludes_any(excludes);
}

/// What the oracle tag dictionary's Use did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Adopted {
    /// The tag was already in the list: "Already in filter".
    Already,
    /// The tag was added: "Added to filter".
    Added,
}

/// Puts a tag picked in the oracle tag dictionary into the filter: the excludes, or the includes in whichever of All or Any is in use.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "the oracle tag dictionary, in the Home package, calls it"
    )
)]
pub(crate) fn adopt_oracle_tag(b: &mut CardQueryBuilder, slug: &str, exclude: bool) -> Adopted {
    let list = if exclude {
        fields::StrList::TagsExclude
    } else if b.oracle_tags_contains_all().is_some() {
        fields::StrList::TagsAll
    } else {
        fields::StrList::TagsAny
    };
    let mut values = list.get(b);
    if values.iter().any(|v| v == slug) {
        return Adopted::Already;
    }
    values.push(slug.to_owned());
    list.set(b, values);
    Adopted::Added
}

#[cfg(test)]
mod tests {
    use zwipe_core::domain::card::scryfall_data::colors::Color;
    use zwipe_core::domain::card::search_card::card_filter::CardSortKey;

    use super::*;

    #[test]
    fn apply_refuses_empty_to_empty_only_when_validating() {
        use FilterState::{Empty, Set};
        assert_eq!(
            apply_action(true, Empty, Empty, false),
            ApplyAction::RefuseEmpty
        );
        assert_eq!(
            apply_action(true, Empty, Empty, true),
            ApplyAction::RefuseEmpty
        );
        assert_eq!(
            apply_action(false, Empty, Empty, true),
            ApplyAction::NoChange
        );
        assert_eq!(
            apply_action(true, Set, Empty, false),
            ApplyAction::Commit { cleared: true }
        );
        assert_eq!(
            apply_action(false, Empty, Set, false),
            ApplyAction::Commit { cleared: false }
        );
        assert_eq!(apply_action(true, Set, Set, true), ApplyAction::NoChange);
    }

    #[test]
    fn a_contradiction_names_its_field() {
        let mut b = CardQueryBuilder::default();
        b.set_card_type_contains_any(vec![CardType::Land]);
        assert_eq!(contradiction(&b), None);
        b.set_card_type_excludes_any(vec![CardType::Land]);
        assert_eq!(
            contradiction(&b).as_deref(),
            Some("Filter can't both include and exclude card types")
        );
    }

    fn commander_deck() -> AddContext {
        AddContext {
            format: Some(Format::Commander),
            colors: Some(Colors::from(vec![Color::Green])),
            has_commander: true,
            lands_at_target: true,
        }
    }

    #[test]
    fn the_fresh_search_carries_deck_context_and_synergy() {
        let b = commander_deck().fresh_search();
        assert!(b.synergy());
        assert_eq!(
            b.legalities_contains_any(),
            Some(&["commander".to_owned()][..])
        );
        assert_eq!(b.card_type_excludes_any(), Some(&[CardType::Land][..]));
        assert!(
            !commander_deck().shows_dot(&b),
            "deck context is not a user filter"
        );
    }

    #[test]
    fn the_gate_serves_a_commander_deck_and_refuses_a_blank_query() {
        let blank = CardQueryBuilder::default();
        assert!(commander_deck().serves(&blank));
        let pile = AddContext::default();
        assert!(!pile.serves(&blank));
        let mut legal_only = blank.clone();
        legal_only.set_legalities_contains_any(vec!["modern".to_owned()]);
        assert!(!pile.serves(&legal_only), "legality alone is deck context");
        let mut sorted = blank;
        sorted.set_sort(CardSortKey::Name);
        assert!(pile.serves(&sorted));
    }

    #[test]
    fn reset_keeps_synergy_and_deck_context_or_says_it_is_at_default() {
        let context = commander_deck();
        let mut b = context.fresh_search();
        assert_eq!(context.reset_search(&mut b), Err(AlreadyDefault));
        b.set_name_contains("elf").set_synergy(false);
        assert_eq!(context.reset_search(&mut b), Ok(()));
        assert_eq!(b.name_contains(), None);
        assert!(!b.synergy(), "synergy is a mode, kept as the user left it");
        assert!(b.color_identity_within().is_some());

        let mut maybe = CardQueryBuilder::default();
        maybe.set_synergy(true);
        assert_eq!(reset_maybeboard(&mut maybe), Err(AlreadyDefault));
        maybe.set_sort(CardSortKey::Name);
        assert_eq!(reset_maybeboard(&mut maybe), Ok(()));
        assert!(maybe.synergy() && maybe.sort().is_none());
    }

    #[test]
    fn lands_leave_the_includes_when_excluded() {
        let mut b = CardQueryBuilder::default();
        b.set_card_type_contains_any(vec![CardType::Land]);
        b.set_card_type_contains_all(vec![CardType::Land, CardType::Creature]);
        ensure_lands_excluded(&mut b);
        assert_eq!(b.card_type_contains_any(), None);
        assert_eq!(b.card_type_contains_all(), Some(&[CardType::Creature][..]));
        ensure_lands_excluded(&mut b);
        assert_eq!(b.card_type_excludes_any(), Some(&[CardType::Land][..]));
    }

    #[test]
    fn the_dictionary_adds_a_tag_once_into_the_mode_in_use() {
        let mut b = CardQueryBuilder::default();
        assert_eq!(adopt_oracle_tag(&mut b, "ramp", false), Adopted::Added);
        assert_eq!(adopt_oracle_tag(&mut b, "ramp", false), Adopted::Already);
        b.set_oracle_tags_contains_all(vec!["draw".to_owned()]);
        adopt_oracle_tag(&mut b, "tutor", false);
        assert_eq!(b.oracle_tags_contains_all().map(<[String]>::len), Some(2));
        adopt_oracle_tag(&mut b, "ramp", true);
        assert_eq!(b.oracle_tags_excludes(), Some(&["ramp".to_owned()][..]));
        assert!(shows_dot(&b));
    }
}
