//! Reading and writing the filter builder's fields the sheet's controls edit, and the mode chips that move values between paired fields.
//!
//! Every write maps empty to unset, as the app's setters do, so a cleared control never leaves an empty list behind.

use zwipe_core::domain::card::scryfall_data::colors::{Color, Colors};
use zwipe_core::domain::card::scryfall_data::rarity::{Rarities, Rarity};
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::card::search_card::card_type::{CardType, WithCardTypes};

/// A field that holds a list of strings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum StrList {
    OracleWordsAny,
    OracleWordsAll,
    OracleWordsExclude,
    KeywordsAny,
    KeywordsAll,
    KeywordsExclude,
    TypeLineAny,
    TypeLineAll,
    TypeLineExclude,
    ProducedAny,
    ProducedAll,
    ProducedExclude,
    RolesAny,
    RolesAll,
    RolesExclude,
    TagsAny,
    TagsAll,
    TagsExclude,
    ArtistInclude,
    ArtistExclude,
    SetInclude,
    SetExclude,
    /// `legalities_contains_any`, the formats a card must be legal in.
    Legal,
}

impl StrList {
    /// The field's values, empty when unset.
    pub(crate) fn get(self, b: &CardQueryBuilder) -> Vec<String> {
        let values = match self {
            Self::OracleWordsAny => b.oracle_text_contains_any(),
            Self::OracleWordsAll => b.oracle_text_contains_all(),
            Self::OracleWordsExclude => b.oracle_text_excludes_any(),
            Self::KeywordsAny => b.keywords_contains_any(),
            Self::KeywordsAll => b.keywords_contains_all(),
            Self::KeywordsExclude => b.keywords_excludes(),
            Self::TypeLineAny => b.type_line_contains_any(),
            Self::TypeLineAll => b.type_line_contains_all(),
            Self::TypeLineExclude => b.type_line_excludes_any(),
            Self::ProducedAny => b.produced_mana_contains_any(),
            Self::ProducedAll => b.produced_mana_contains_all(),
            Self::ProducedExclude => b.produced_mana_excludes(),
            Self::RolesAny => b.card_roles_contains_any(),
            Self::RolesAll => b.card_roles_contains_all(),
            Self::RolesExclude => b.card_roles_excludes(),
            Self::TagsAny => b.oracle_tags_contains_any(),
            Self::TagsAll => b.oracle_tags_contains_all(),
            Self::TagsExclude => b.oracle_tags_excludes(),
            Self::ArtistInclude => b.artist_equals_any(),
            Self::ArtistExclude => b.artist_excludes_any(),
            Self::SetInclude => b.set_equals_any(),
            Self::SetExclude => b.set_excludes_any(),
            Self::Legal => b.legalities_contains_any(),
        };
        values.map(<[String]>::to_vec).unwrap_or_default()
    }

    /// Writes `values`, or unsets the field when they are empty.
    pub(crate) fn set(self, b: &mut CardQueryBuilder, values: Vec<String>) {
        if values.is_empty() {
            self.unset(b);
            return;
        }
        match self {
            Self::OracleWordsAny => b.set_oracle_text_contains_any(values),
            Self::OracleWordsAll => b.set_oracle_text_contains_all(values),
            Self::OracleWordsExclude => b.set_oracle_text_excludes_any(values),
            Self::KeywordsAny => b.set_keywords_contains_any(values),
            Self::KeywordsAll => b.set_keywords_contains_all(values),
            Self::KeywordsExclude => b.set_keywords_excludes(values),
            Self::TypeLineAny => b.set_type_line_contains_any(values),
            Self::TypeLineAll => b.set_type_line_contains_all(values),
            Self::TypeLineExclude => b.set_type_line_excludes_any(values),
            Self::ProducedAny => b.set_produced_mana_contains_any(values),
            Self::ProducedAll => b.set_produced_mana_contains_all(values),
            Self::ProducedExclude => b.set_produced_mana_excludes(values),
            Self::RolesAny => b.set_card_roles_contains_any(values),
            Self::RolesAll => b.set_card_roles_contains_all(values),
            Self::RolesExclude => b.set_card_roles_excludes(values),
            Self::TagsAny => b.set_oracle_tags_contains_any(values),
            Self::TagsAll => b.set_oracle_tags_contains_all(values),
            Self::TagsExclude => b.set_oracle_tags_excludes(values),
            Self::ArtistInclude => b.set_artist_equals_any(values),
            Self::ArtistExclude => b.set_artist_excludes_any(values),
            Self::SetInclude => b.set_set_equals_any(values),
            Self::SetExclude => b.set_set_excludes_any(values),
            Self::Legal => b.set_legalities_contains_any(values),
        };
    }

    pub(crate) fn unset(self, b: &mut CardQueryBuilder) {
        match self {
            Self::OracleWordsAny => b.unset_oracle_text_contains_any(),
            Self::OracleWordsAll => b.unset_oracle_text_contains_all(),
            Self::OracleWordsExclude => b.unset_oracle_text_excludes_any(),
            Self::KeywordsAny => b.unset_keywords_contains_any(),
            Self::KeywordsAll => b.unset_keywords_contains_all(),
            Self::KeywordsExclude => b.unset_keywords_excludes(),
            Self::TypeLineAny => b.unset_type_line_contains_any(),
            Self::TypeLineAll => b.unset_type_line_contains_all(),
            Self::TypeLineExclude => b.unset_type_line_excludes_any(),
            Self::ProducedAny => b.unset_produced_mana_contains_any(),
            Self::ProducedAll => b.unset_produced_mana_contains_all(),
            Self::ProducedExclude => b.unset_produced_mana_excludes(),
            Self::RolesAny => b.unset_card_roles_contains_any(),
            Self::RolesAll => b.unset_card_roles_contains_all(),
            Self::RolesExclude => b.unset_card_roles_excludes(),
            Self::TagsAny => b.unset_oracle_tags_contains_any(),
            Self::TagsAll => b.unset_oracle_tags_contains_all(),
            Self::TagsExclude => b.unset_oracle_tags_excludes(),
            Self::ArtistInclude => b.unset_artist_equals_any(),
            Self::ArtistExclude => b.unset_artist_excludes_any(),
            Self::SetInclude => b.unset_set_equals_any(),
            Self::SetExclude => b.unset_set_excludes_any(),
            Self::Legal => b.unset_legalities_contains_any(),
        };
    }
}

/// A one-line text field. An empty string unsets it.
#[expect(
    clippy::enum_variant_names,
    reason = "each names the builder field it edits"
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum TextField {
    NameContains,
    NameNotContains,
    OracleContains,
    OracleNotContains,
    FlavorContains,
    FlavorNotContains,
}

impl TextField {
    pub(crate) fn placeholder(self) -> &'static str {
        match self {
            Self::NameContains => "Name contains",
            Self::NameNotContains => "Name doesn't contain",
            Self::OracleContains => "Oracle text contains",
            Self::OracleNotContains => "Oracle text doesn't contain",
            Self::FlavorContains => "Flavor text contains",
            Self::FlavorNotContains => "Flavor text doesn't contain",
        }
    }

    pub(crate) fn get(self, b: &CardQueryBuilder) -> &str {
        match self {
            Self::NameContains => b.name_contains(),
            Self::NameNotContains => b.name_not_contains(),
            Self::OracleContains => b.oracle_text_contains(),
            Self::OracleNotContains => b.oracle_text_not_contains(),
            Self::FlavorContains => b.flavor_text_contains(),
            Self::FlavorNotContains => b.flavor_text_not_contains(),
        }
        .unwrap_or_default()
    }

    /// Writes `value` on every keystroke; empty unsets.
    pub(crate) fn set(self, b: &mut CardQueryBuilder, value: &str) {
        if value.is_empty() {
            self.unset(b);
            return;
        }
        match self {
            Self::NameContains => b.set_name_contains(value),
            Self::NameNotContains => b.set_name_not_contains(value),
            Self::OracleContains => b.set_oracle_text_contains(value),
            Self::OracleNotContains => b.set_oracle_text_not_contains(value),
            Self::FlavorContains => b.set_flavor_text_contains(value),
            Self::FlavorNotContains => b.set_flavor_text_not_contains(value),
        };
    }

    pub(crate) fn unset(self, b: &mut CardQueryBuilder) {
        match self {
            Self::NameContains => b.unset_name_contains(),
            Self::NameNotContains => b.unset_name_not_contains(),
            Self::OracleContains => b.unset_oracle_text_contains(),
            Self::OracleNotContains => b.unset_oracle_text_not_contains(),
            Self::FlavorContains => b.unset_flavor_text_contains(),
            Self::FlavorNotContains => b.unset_flavor_text_not_contains(),
        };
    }
}

/// Any or All, for a list of includes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum Match {
    #[default]
    Any,
    All,
}

impl Match {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Any => "Any",
            Self::All => "All",
        }
    }

    pub(crate) fn flipped(self) -> Self {
        match self {
            Self::Any => Self::All,
            Self::All => Self::Any,
        }
    }
}

/// Exact or Range, for a number or a color identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum Exactness {
    #[default]
    Exact,
    Range,
}

impl Exactness {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Exact => "Exact",
            Self::Range => "Range",
        }
    }

    pub(crate) fn flipped(self) -> Self {
        match self {
            Self::Exact => Self::Range,
            Self::Range => Self::Exact,
        }
    }
}

/// Include or Exclude, for one list that can go either way.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum Side {
    #[default]
    Include,
    Exclude,
}

impl Side {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Include => "Include",
            Self::Exclude => "Exclude",
        }
    }

    pub(crate) fn flipped(self) -> Self {
        match self {
            Self::Include => Self::Exclude,
            Self::Exclude => Self::Include,
        }
    }
}

/// A list of includes with an Any or All chip.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum MatchGroup {
    OracleWords,
    Keywords,
    TypeLine,
    Produced,
    Roles,
    Tags,
    BasicTypes,
}

impl MatchGroup {
    /// The Any and All fields of a string group; `None` for basic types.
    fn lists(self) -> Option<(StrList, StrList)> {
        Some(match self {
            Self::OracleWords => (StrList::OracleWordsAny, StrList::OracleWordsAll),
            Self::Keywords => (StrList::KeywordsAny, StrList::KeywordsAll),
            Self::TypeLine => (StrList::TypeLineAny, StrList::TypeLineAll),
            Self::Produced => (StrList::ProducedAny, StrList::ProducedAll),
            Self::Roles => (StrList::RolesAny, StrList::RolesAll),
            Self::Tags => (StrList::TagsAny, StrList::TagsAll),
            Self::BasicTypes => return None,
        })
    }

    /// The mode the builder holds: All when the All field is set, Any when the Any field is, else `None`.
    pub(crate) fn held(self, b: &CardQueryBuilder) -> Option<Match> {
        let (any, all) = match self.lists() {
            Some((any, all)) => (!any.get(b).is_empty(), !all.get(b).is_empty()),
            None => (
                b.card_type_contains_any().is_some(),
                b.card_type_contains_all().is_some(),
            ),
        };
        if all {
            Some(Match::All)
        } else if any {
            Some(Match::Any)
        } else {
            None
        }
    }

    /// The included values as strings, from whichever field holds them.
    pub(crate) fn values(self, b: &CardQueryBuilder) -> Vec<String> {
        match self.lists() {
            Some((any, all)) => {
                let all = all.get(b);
                if all.is_empty() { any.get(b) } else { all }
            }
            None => b
                .card_type_contains_all()
                .or_else(|| b.card_type_contains_any())
                .unwrap_or_default()
                .iter()
                .map(ToString::to_string)
                .collect(),
        }
    }

    /// Writes the includes under `mode`, unsetting both fields first.
    pub(crate) fn write(self, b: &mut CardQueryBuilder, mode: Match, values: Vec<String>) {
        if let Some((any, all)) = self.lists() {
            any.unset(b);
            all.unset(b);
            match mode {
                Match::Any => any.set(b, values),
                Match::All => all.set(b, values),
            }
            return;
        }
        b.unset_card_type_contains_any();
        b.unset_card_type_contains_all();
        let types = card_types_from(&values);
        if !types.is_empty() {
            match mode {
                Match::Any => b.set_card_type_contains_any(types),
                Match::All => b.set_card_type_contains_all(types),
            };
        }
    }

    /// The matching exclude list.
    pub(crate) fn exclude(self) -> Exclude {
        match self {
            Self::OracleWords => Exclude::List(StrList::OracleWordsExclude),
            Self::Keywords => Exclude::List(StrList::KeywordsExclude),
            Self::TypeLine => Exclude::List(StrList::TypeLineExclude),
            Self::Produced => Exclude::List(StrList::ProducedExclude),
            Self::Roles => Exclude::List(StrList::RolesExclude),
            Self::Tags => Exclude::List(StrList::TagsExclude),
            Self::BasicTypes => Exclude::BasicTypes,
        }
    }
}

/// A list of excludes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Exclude {
    List(StrList),
    BasicTypes,
}

impl Exclude {
    pub(crate) fn values(self, b: &CardQueryBuilder) -> Vec<String> {
        match self {
            Self::List(list) => list.get(b),
            Self::BasicTypes => b
                .card_type_excludes_any()
                .unwrap_or_default()
                .iter()
                .map(ToString::to_string)
                .collect(),
        }
    }

    pub(crate) fn write(self, b: &mut CardQueryBuilder, values: Vec<String>) {
        match self {
            Self::List(list) => list.set(b, values),
            Self::BasicTypes => {
                let types = card_types_from(&values);
                if types.is_empty() {
                    b.unset_card_type_excludes_any();
                } else {
                    b.set_card_type_excludes_any(types);
                }
            }
        }
    }
}

/// The seven basic card types in the app's chip order.
pub(crate) fn card_types() -> Vec<CardType> {
    Vec::<CardType>::with_all_card_types()
}

/// The card types named by `values`, in chip order.
fn card_types_from(values: &[String]) -> Vec<CardType> {
    card_types()
        .into_iter()
        .filter(|t| values.contains(&t.to_string()))
        .collect()
}

/// A list that can be an include or an exclude, with an Include or Exclude chip.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum SideGroup {
    Artist,
    Set,
    Rarity,
}

impl SideGroup {
    /// Exclude when the builder holds excludes.
    pub(crate) fn held(self, b: &CardQueryBuilder) -> Option<Side> {
        let (include, exclude) = match self {
            Self::Artist => (
                b.artist_equals_any().is_some(),
                b.artist_excludes_any().is_some(),
            ),
            Self::Set => (b.set_equals_any().is_some(), b.set_excludes_any().is_some()),
            Self::Rarity => (
                b.rarity_equals_any().is_some(),
                b.rarity_excludes_any().is_some(),
            ),
        };
        if exclude {
            Some(Side::Exclude)
        } else if include {
            Some(Side::Include)
        } else {
            None
        }
    }

    /// The values from whichever side holds them; rarities by long name.
    pub(crate) fn values(self, b: &CardQueryBuilder) -> Vec<String> {
        let side = self.held(b).unwrap_or_default();
        match (self, side) {
            (Self::Artist, Side::Include) => StrList::ArtistInclude.get(b),
            (Self::Artist, Side::Exclude) => StrList::ArtistExclude.get(b),
            (Self::Set, Side::Include) => StrList::SetInclude.get(b),
            (Self::Set, Side::Exclude) => StrList::SetExclude.get(b),
            (Self::Rarity, Side::Include) => rarity_names(b.rarity_equals_any()),
            (Self::Rarity, Side::Exclude) => rarity_names(b.rarity_excludes_any()),
        }
    }

    /// Writes the values on `side`, unsetting both first.
    pub(crate) fn write(self, b: &mut CardQueryBuilder, side: Side, values: Vec<String>) {
        match self {
            Self::Artist => {
                StrList::ArtistInclude.unset(b);
                StrList::ArtistExclude.unset(b);
                match side {
                    Side::Include => StrList::ArtistInclude.set(b, values),
                    Side::Exclude => StrList::ArtistExclude.set(b, values),
                }
            }
            Self::Set => {
                StrList::SetInclude.unset(b);
                StrList::SetExclude.unset(b);
                match side {
                    Side::Include => StrList::SetInclude.set(b, values),
                    Side::Exclude => StrList::SetExclude.set(b, values),
                }
            }
            Self::Rarity => {
                b.unset_rarity_equals_any();
                b.unset_rarity_excludes_any();
                let rarities: Rarities = Rarity::all()
                    .into_iter()
                    .filter(|r| values.contains(&r.to_long_name()))
                    .collect();
                if !rarities.is_empty() {
                    match side {
                        Side::Include => b.set_rarity_equals_any(rarities),
                        Side::Exclude => b.set_rarity_excludes_any(rarities),
                    };
                }
            }
        }
    }
}

fn rarity_names(rarities: Option<&Rarities>) -> Vec<String> {
    rarities
        .map(|r| r.iter().map(Rarity::to_long_name).collect())
        .unwrap_or_default()
}

/// A field that can be one value or a range, with an Exact or Range chip.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum RangeGroup {
    ManaValue,
    ColorIdentity,
    Power,
    Toughness,
}

impl RangeGroup {
    /// Range when the builder holds the range field, Exact when it holds the exact one.
    pub(crate) fn held(self, b: &CardQueryBuilder) -> Option<Exactness> {
        let (exact, range) = match self {
            Self::ManaValue => (b.cmc_equals().is_some(), b.cmc_range().is_some()),
            Self::ColorIdentity => (
                b.color_identity_equals().is_some(),
                b.color_identity_within().is_some(),
            ),
            Self::Power => (b.power_equals().is_some(), b.power_range().is_some()),
            Self::Toughness => (
                b.toughness_equals().is_some(),
                b.toughness_range().is_some(),
            ),
        };
        if exact {
            Some(Exactness::Exact)
        } else if range {
            Some(Exactness::Range)
        } else {
            None
        }
    }

    /// Switches to `to`. Colors move to the other field; numbers unset the field they leave.
    pub(crate) fn switch(self, b: &mut CardQueryBuilder, to: Exactness) {
        match self {
            Self::ManaValue => match to {
                Exactness::Exact => b.unset_cmc_range(),
                Exactness::Range => b.unset_cmc_equals(),
            },
            Self::Power => match to {
                Exactness::Exact => b.unset_power_range(),
                Exactness::Range => b.unset_power_equals(),
            },
            Self::Toughness => match to {
                Exactness::Exact => b.unset_toughness_range(),
                Exactness::Range => b.unset_toughness_equals(),
            },
            Self::ColorIdentity => {
                let colors = identity_colors(b);
                b.unset_color_identity_equals();
                b.unset_color_identity_within();
                write_identity(b, to, colors)
            }
        };
    }

    pub(crate) fn unset(self, b: &mut CardQueryBuilder) {
        match self {
            Self::ManaValue => b.unset_cmc_equals().unset_cmc_range(),
            Self::ColorIdentity => b
                .unset_color_identity_equals()
                .unset_color_identity_within(),
            Self::Power => b.unset_power_equals().unset_power_range(),
            Self::Toughness => b.unset_toughness_equals().unset_toughness_range(),
        };
    }

    pub(crate) fn is_set(self, b: &CardQueryBuilder) -> bool {
        self.held(b).is_some()
    }
}

/// The colors the color identity filter holds, from whichever field is set.
pub(crate) fn identity_colors(b: &CardQueryBuilder) -> Vec<Color> {
    b.color_identity_equals()
        .or_else(|| b.color_identity_within())
        .map(|c| c.to_vec())
        .unwrap_or_default()
}

fn write_identity(
    b: &mut CardQueryBuilder,
    mode: Exactness,
    colors: Vec<Color>,
) -> &mut CardQueryBuilder {
    if colors.is_empty() {
        return b;
    }
    let colors = Colors::from(colors);
    match mode {
        Exactness::Exact => b.set_color_identity_equals(colors),
        Exactness::Range => b.set_color_identity_within(colors),
    }
}

/// Toggles `color` in the color identity filter, writing into `mode`'s field; empty unsets both.
pub(crate) fn toggle_identity(b: &mut CardQueryBuilder, mode: Exactness, color: Color) {
    let mut colors = identity_colors(b);
    if let Some(at) = colors.iter().position(|c| *c == color) {
        colors.remove(at);
    } else {
        colors.push(color);
    }
    let colors: Vec<Color> = Color::all()
        .into_iter()
        .filter(|c| colors.contains(c))
        .collect();
    b.unset_color_identity_equals();
    b.unset_color_identity_within();
    write_identity(b, mode, colors);
}

/// A stat with a stepper.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Stat {
    Power,
    Toughness,
}

/// Which stepper of a range.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum End {
    #[default]
    Min,
    Max,
}

impl Stat {
    pub(crate) fn group(self) -> RangeGroup {
        match self {
            Self::Power => RangeGroup::Power,
            Self::Toughness => RangeGroup::Toughness,
        }
    }

    pub(crate) fn exact(self, b: &CardQueryBuilder) -> Option<i32> {
        match self {
            Self::Power => b.power_equals(),
            Self::Toughness => b.toughness_equals(),
        }
    }

    pub(crate) fn range(self, b: &CardQueryBuilder) -> Option<(i32, i32)> {
        match self {
            Self::Power => b.power_range(),
            Self::Toughness => b.toughness_range(),
        }
    }

    /// Steps the exact value by `delta`: from unset, "-" gives -1 and "+" gives 1.
    pub(crate) fn step_exact(self, b: &mut CardQueryBuilder, delta: i32) {
        let next = self.exact(b).unwrap_or(0).saturating_add(delta);
        match self {
            Self::Power => b.set_power_equals(next),
            Self::Toughness => b.set_toughness_equals(next),
        };
    }

    /// Steps one end of the range by `delta`, both ends starting from 0.
    pub(crate) fn step_range(self, b: &mut CardQueryBuilder, end: End, delta: i32) {
        let (min, max) = self.range(b).unwrap_or((0, 0));
        let range = match end {
            End::Min => (min.saturating_add(delta), max),
            End::Max => (min, max.saturating_add(delta)),
        };
        match self {
            Self::Power => b.set_power_range(range),
            Self::Toughness => b.set_toughness_range(range),
        };
    }
}

/// One of the five config tri-toggles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Toggle {
    Playable,
    Digital,
    Oversized,
    Promo,
    ContentWarning,
}

impl Toggle {
    pub(crate) const ALL: [Self; 5] = [
        Self::Playable,
        Self::Digital,
        Self::Oversized,
        Self::Promo,
        Self::ContentWarning,
    ];

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Playable => "Is playable",
            Self::Digital => "Is digital",
            Self::Oversized => "Is oversized",
            Self::Promo => "Is promo",
            Self::ContentWarning => "Has content warning",
        }
    }

    pub(crate) fn get(self, b: &CardQueryBuilder) -> Option<bool> {
        match self {
            Self::Playable => b.is_playable(),
            Self::Digital => b.digital(),
            Self::Oversized => b.oversized(),
            Self::Promo => b.promo(),
            Self::ContentWarning => b.content_warning(),
        }
    }

    pub(crate) fn set(self, b: &mut CardQueryBuilder, value: Option<bool>) {
        match (self, value) {
            (Self::Playable, Some(v)) => b.set_is_playable(v),
            (Self::Playable, None) => b.unset_is_playable(),
            (Self::Digital, Some(v)) => b.set_digital(v),
            (Self::Digital, None) => b.unset_digital(),
            (Self::Oversized, Some(v)) => b.set_oversized(v),
            (Self::Oversized, None) => b.unset_oversized(),
            (Self::Promo, Some(v)) => b.set_promo(v),
            (Self::Promo, None) => b.unset_promo(),
            (Self::ContentWarning, Some(v)) => b.set_content_warning(v),
            (Self::ContentWarning, None) => b.unset_content_warning(),
        };
    }

    /// Taps "Yes" (`Some(true)`), "No" (`Some(false)`) or "Any" (`None`): tapping the chip already on unsets it.
    pub(crate) fn tap(self, b: &mut CardQueryBuilder, chip: Option<bool>) {
        let next = if chip.is_some() && self.get(b) == chip {
            None
        } else {
            chip
        };
        self.set(b, next);
    }
}

/// Toggles `value` in a list.
pub(crate) fn toggled(mut values: Vec<String>, value: &str) -> Vec<String> {
    if let Some(at) = values.iter().position(|v| v == value) {
        values.remove(at);
    } else {
        values.push(value.to_owned());
    }
    values
}

/// Parses a typed number as the app does on blur: empty is `Ok(None)`, a number is `Ok(Some(n))`, anything else is an error.
///
/// # Errors
///
/// Returns `Err(())` when the text is not empty and not a number.
pub(crate) fn parse_number(text: &str) -> Result<Option<f64>, ()> {
    let text = text.trim();
    if text.is_empty() {
        return Ok(None);
    }
    text.parse::<f64>().map(Some).map_err(drop)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(values: &[&str]) -> Vec<String> {
        values.iter().map(|&v| v.to_owned()).collect()
    }

    #[test]
    fn match_groups_write_one_field_and_read_their_mode() {
        let mut b = CardQueryBuilder::default();
        assert_eq!(MatchGroup::Keywords.held(&b), None);
        MatchGroup::Keywords.write(&mut b, Match::All, words(&["flying"]));
        assert_eq!(MatchGroup::Keywords.held(&b), Some(Match::All));
        assert_eq!(b.keywords_contains_any(), None);
        MatchGroup::Keywords.write(&mut b, Match::Any, words(&["flying", "haste"]));
        assert_eq!(b.keywords_contains_all(), None, "writing unsets both first");
        assert_eq!(MatchGroup::Keywords.values(&b), words(&["flying", "haste"]));
        MatchGroup::Keywords.write(&mut b, Match::Any, Vec::new());
        assert!(b.is_empty());
    }

    #[test]
    fn basic_types_round_trip_through_names() {
        let mut b = CardQueryBuilder::default();
        MatchGroup::BasicTypes.write(&mut b, Match::Any, words(&["Land", "Creature"]));
        assert_eq!(
            b.card_type_contains_any(),
            Some(&[CardType::Creature, CardType::Land][..])
        );
        assert_eq!(
            MatchGroup::BasicTypes.values(&b),
            words(&["Creature", "Land"])
        );
        MatchGroup::BasicTypes
            .exclude()
            .write(&mut b, words(&["Instant"]));
        assert_eq!(b.card_type_excludes_any(), Some(&[CardType::Instant][..]));
    }

    #[test]
    fn side_groups_move_values_between_include_and_exclude() {
        let mut b = CardQueryBuilder::default();
        SideGroup::Rarity.write(&mut b, Side::Include, words(&["Rare", "Common"]));
        assert_eq!(SideGroup::Rarity.held(&b), Some(Side::Include));
        assert_eq!(SideGroup::Rarity.values(&b), words(&["Common", "Rare"]));
        let values = SideGroup::Rarity.values(&b);
        SideGroup::Rarity.write(&mut b, Side::Exclude, values);
        assert_eq!(b.rarity_equals_any(), None);
        assert_eq!(SideGroup::Rarity.held(&b), Some(Side::Exclude));
        SideGroup::Artist.write(&mut b, Side::Exclude, words(&["Rebecca Guay"]));
        assert_eq!(b.artist_excludes_any(), Some(&words(&["Rebecca Guay"])[..]));
    }

    #[test]
    fn color_identity_toggles_into_its_mode_and_switches_fields() {
        let mut b = CardQueryBuilder::default();
        toggle_identity(&mut b, Exactness::Range, Color::Green);
        toggle_identity(&mut b, Exactness::Range, Color::White);
        assert_eq!(identity_colors(&b), [Color::White, Color::Green]);
        assert_eq!(RangeGroup::ColorIdentity.held(&b), Some(Exactness::Range));
        RangeGroup::ColorIdentity.switch(&mut b, Exactness::Exact);
        assert!(b.color_identity_within().is_none());
        assert_eq!(b.color_identity_equals().map(|c| c.len()), Some(2));
        toggle_identity(&mut b, Exactness::Exact, Color::Green);
        toggle_identity(&mut b, Exactness::Exact, Color::White);
        assert!(b.is_empty(), "the last color off unsets the field");
    }

    #[test]
    fn numbers_switch_by_unsetting_the_field_they_leave() {
        let mut b = CardQueryBuilder::default();
        b.set_cmc_equals(3.0);
        RangeGroup::ManaValue.switch(&mut b, Exactness::Range);
        assert!(b.cmc_equals().is_none());
        b.set_cmc_range((1.0, 2.0));
        assert_eq!(RangeGroup::ManaValue.held(&b), Some(Exactness::Range));
        RangeGroup::ManaValue.unset(&mut b);
        assert!(!RangeGroup::ManaValue.is_set(&b));
    }

    #[test]
    fn steppers_start_from_zero_and_step_either_way() {
        let mut b = CardQueryBuilder::default();
        Stat::Power.step_exact(&mut b, -1);
        assert_eq!(b.power_equals(), Some(-1));
        Stat::Toughness.step_exact(&mut b, 1);
        assert_eq!(b.toughness_equals(), Some(1));
        Stat::Power.step_range(&mut b, End::Max, 1);
        assert_eq!(b.power_range(), Some((0, 1)));
        Stat::Power.step_range(&mut b, End::Min, -1);
        assert_eq!(b.power_range(), Some((-1, 1)));
        b.set_power_equals(i32::MAX);
        Stat::Power.step_exact(&mut b, 1);
        assert_eq!(b.power_equals(), Some(i32::MAX), "saturating");
    }

    #[test]
    fn tri_toggles_unset_when_the_chip_already_on_is_tapped() {
        let mut b = CardQueryBuilder::default();
        assert_eq!(Toggle::Playable.get(&b), Some(true));
        Toggle::Playable.tap(&mut b, Some(true));
        assert_eq!(Toggle::Playable.get(&b), None);
        Toggle::Promo.tap(&mut b, Some(false));
        assert_eq!(Toggle::Promo.get(&b), Some(false));
        Toggle::Promo.tap(&mut b, None);
        assert_eq!(Toggle::Promo.get(&b), None);
    }

    #[test]
    fn text_fields_unset_on_empty_and_numbers_parse_or_refuse() {
        let mut b = CardQueryBuilder::default();
        TextField::FlavorContains.set(&mut b, "tree");
        assert_eq!(TextField::FlavorContains.get(&b), "tree");
        TextField::FlavorContains.set(&mut b, "");
        assert!(b.is_empty());
        assert_eq!(parse_number(" "), Ok(None));
        assert_eq!(parse_number("2.5"), Ok(Some(2.5)));
        assert_eq!(parse_number("two"), Err(()));
        assert_eq!(toggled(words(&["a"]), "b"), words(&["a", "b"]));
        assert_eq!(toggled(words(&["a", "b"]), "a"), words(&["b"]));
    }
}
