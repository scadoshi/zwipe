//! The filter sheet's fifteen sections: their titles in order, the rows each one draws, when each shows as active, and what its clear button unsets.

use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;

use super::fields::{
    Exactness, Exclude, Match, MatchGroup, RangeGroup, Side, SideGroup, Stat, TextField, Toggle,
};
use super::picker::PickerField;

/// One accordion section.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Section {
    Name,
    OracleText,
    Types,
    Mana,
    Combat,
    FlavorText,
    Artist,
    Rarity,
    CardRoles,
    OracleTags,
    Set,
    Format,
    Price,
    Sort,
    Config,
}

impl Section {
    /// Every section in the app's order.
    pub(crate) const ALL: [Self; 15] = [
        Self::Name,
        Self::OracleText,
        Self::Types,
        Self::Mana,
        Self::Combat,
        Self::FlavorText,
        Self::Artist,
        Self::Rarity,
        Self::CardRoles,
        Self::OracleTags,
        Self::Set,
        Self::Format,
        Self::Price,
        Self::Sort,
        Self::Config,
    ];

    /// The sections a host shows: Format only with `show_format`.
    pub(crate) fn shown(show_format: bool) -> Vec<Self> {
        Self::ALL
            .into_iter()
            .filter(|s| show_format || *s != Self::Format)
            .collect()
    }

    pub(crate) fn title(self) -> &'static str {
        match self {
            Self::Name => "Name",
            Self::OracleText => "Oracle text",
            Self::Types => "Types",
            Self::Mana => "Mana",
            Self::Combat => "Combat",
            Self::FlavorText => "Flavor text",
            Self::Artist => "Artist",
            Self::Rarity => "Rarity",
            Self::CardRoles => "Card roles",
            Self::OracleTags => "Oracle tags",
            Self::Set => "Set",
            Self::Format => "Format",
            Self::Price => "Price",
            Self::Sort => "Sort",
            Self::Config => "Config",
        }
    }

    /// Whether the section has anything set, which shows its clear button. Config is active when any toggle differs from the default.
    pub(crate) fn is_active(self, b: &CardQueryBuilder) -> bool {
        if self == Self::Config {
            let default = CardQueryBuilder::default();
            return Toggle::ALL
                .into_iter()
                .any(|toggle| toggle.get(b) != toggle.get(&default));
        }
        let mut cleared = b.clone();
        self.clear(&mut cleared);
        cleared != *b
    }

    /// Unsets every field the section's clear button covers.
    pub(crate) fn clear(self, b: &mut CardQueryBuilder) {
        match self {
            Self::Name => {
                b.unset_name_contains().unset_name_not_contains();
            }
            Self::OracleText => {
                b.unset_oracle_text_contains()
                    .unset_oracle_text_contains_any()
                    .unset_oracle_text_contains_all()
                    .unset_keywords_contains_any()
                    .unset_keywords_contains_all()
                    .unset_oracle_text_not_contains()
                    .unset_oracle_text_excludes_any()
                    .unset_keywords_excludes();
            }
            Self::Types => {
                b.unset_type_line_contains()
                    .unset_type_line_contains_any()
                    .unset_type_line_contains_all()
                    .unset_card_type_contains_any()
                    .unset_card_type_contains_all()
                    .unset_type_line_not_contains()
                    .unset_type_line_excludes_any()
                    .unset_card_type_excludes_any();
            }
            Self::Mana => {
                b.unset_cmc_equals()
                    .unset_cmc_range()
                    .unset_color_identity_equals()
                    .unset_color_identity_within()
                    .unset_produced_mana_contains_any()
                    .unset_produced_mana_contains_all()
                    .unset_produced_mana_excludes();
            }
            Self::Combat => {
                b.unset_power_equals()
                    .unset_power_range()
                    .unset_toughness_equals()
                    .unset_toughness_range();
            }
            Self::FlavorText => {
                b.unset_flavor_text_contains()
                    .unset_has_flavor_text()
                    .unset_flavor_text_not_contains();
            }
            Self::Artist => {
                b.unset_artist_equals_any().unset_artist_excludes_any();
            }
            Self::Rarity => {
                b.unset_rarity_equals_any().unset_rarity_excludes_any();
            }
            Self::CardRoles => {
                b.unset_card_roles_contains_any()
                    .unset_card_roles_contains_all()
                    .unset_card_roles_excludes();
            }
            Self::OracleTags => {
                b.unset_oracle_tags_contains_any()
                    .unset_oracle_tags_contains_all()
                    .unset_oracle_tags_excludes();
            }
            Self::Set => {
                b.unset_set_equals_any().unset_set_excludes_any();
            }
            Self::Format => {
                b.unset_legalities_contains_any()
                    .unset_is_commander_in_format()
                    .unset_is_partner()
                    .unset_is_background()
                    .unset_is_signature_spell();
            }
            Self::Price => {
                b.unset_price_min().unset_price_max();
            }
            Self::Sort => {
                b.unset_sort();
            }
            // The app's clear sets promo to No, while the default leaves it at Any.
            Self::Config => {
                b.set_is_playable(true)
                    .set_digital(false)
                    .set_oversized(false)
                    .set_promo(false)
                    .set_content_warning(false);
            }
        }
    }
}

/// Mode chips the sheet keeps while their fields are empty, so a chosen mode sticks until values arrive.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Modes {
    matches: Vec<(MatchGroup, Match)>,
    ranges: Vec<(RangeGroup, Exactness)>,
    sides: Vec<(SideGroup, Side)>,
}

impl Modes {
    /// The Any or All mode: the builder's when it holds values, else the one chosen here, else Any.
    pub(crate) fn match_of(&self, b: &CardQueryBuilder, group: MatchGroup) -> Match {
        group.held(b).unwrap_or_else(|| {
            self.matches
                .iter()
                .find(|(g, _)| *g == group)
                .map_or_else(Match::default, |(_, m)| *m)
        })
    }

    pub(crate) fn range_of(&self, b: &CardQueryBuilder, group: RangeGroup) -> Exactness {
        group.held(b).unwrap_or_else(|| {
            self.ranges
                .iter()
                .find(|(g, _)| *g == group)
                .map_or_else(Exactness::default, |(_, m)| *m)
        })
    }

    pub(crate) fn side_of(&self, b: &CardQueryBuilder, group: SideGroup) -> Side {
        group.held(b).unwrap_or_else(|| {
            self.sides
                .iter()
                .find(|(g, _)| *g == group)
                .map_or_else(Side::default, |(_, m)| *m)
        })
    }

    /// Flips a mode chip, moving any values to the other field.
    pub(crate) fn flip(&mut self, b: &mut CardQueryBuilder, chip: ModeChip) {
        match chip {
            ModeChip::Match(group) => {
                let to = self.match_of(b, group).flipped();
                let values = group.values(b);
                group.write(b, to, values);
                self.matches.retain(|(g, _)| *g != group);
                self.matches.push((group, to));
            }
            ModeChip::Range(group) => {
                let to = self.range_of(b, group).flipped();
                group.switch(b, to);
                self.ranges.retain(|(g, _)| *g != group);
                self.ranges.push((group, to));
            }
            ModeChip::Side(group) => {
                let to = self.side_of(b, group).flipped();
                let values = group.values(b);
                group.write(b, to, values);
                self.sides.retain(|(g, _)| *g != group);
                self.sides.push((group, to));
            }
        }
    }

    /// The chip's label as it reads now.
    pub(crate) fn label(&self, b: &CardQueryBuilder, chip: ModeChip) -> &'static str {
        match chip {
            ModeChip::Match(group) => self.match_of(b, group).label(),
            ModeChip::Range(group) => self.range_of(b, group).label(),
            ModeChip::Side(group) => self.side_of(b, group).label(),
        }
    }
}

/// A mode chip beside a label.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ModeChip {
    Match(MatchGroup),
    Range(RangeGroup),
    Side(SideGroup),
}

/// What a label's clear button unsets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Clear {
    Text(TextField),
    Includes(MatchGroup),
    Excludes(Exclude),
    Range(RangeGroup),
    Side(SideGroup),
    CommanderIn,
    Legal,
    Price,
    Sort,
}

impl Clear {
    pub(crate) fn apply(self, b: &mut CardQueryBuilder) {
        match self {
            Self::Text(field) => field.unset(b),
            Self::Includes(group) => group.write(b, Match::Any, Vec::new()),
            Self::Excludes(exclude) => exclude.write(b, Vec::new()),
            Self::Range(group) => group.unset(b),
            Self::Side(group) => group.write(b, Side::Include, Vec::new()),
            Self::CommanderIn => {
                b.unset_is_commander_in_format();
            }
            Self::Legal => {
                b.unset_legalities_contains_any();
            }
            Self::Price => {
                b.unset_price_min().unset_price_max();
            }
            Self::Sort => {
                b.unset_sort();
            }
        }
    }

    /// Whether there is anything to clear, which shows the button.
    pub(crate) fn applies(self, b: &CardQueryBuilder) -> bool {
        let mut cleared = b.clone();
        self.apply(&mut cleared);
        cleared != *b
    }
}

/// A chip row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum ChipSet {
    BasicIncludes,
    BasicExcludes,
    Colors,
    ProducedIncludes,
    ProducedExcludes,
    Rarity,
    RolesInclude,
    RolesExclude,
    CommanderIn,
    LegalIn,
    Currency,
    SortBy,
    SortOrder,
    Config(Toggle),
}

/// A typed number that is parsed when the box loses focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum NumberField {
    ManaValue,
    ManaMin,
    ManaMax,
    PriceMin,
    PriceMax,
}

impl NumberField {
    pub(crate) fn placeholder(self) -> &'static str {
        match self {
            Self::ManaValue => "Mana value",
            Self::ManaMin | Self::PriceMin => "Min",
            Self::ManaMax | Self::PriceMax => "Max",
        }
    }

    /// The value the builder holds, as the box shows it.
    pub(crate) fn text(self, b: &CardQueryBuilder) -> String {
        let value = match self {
            Self::ManaValue => b.cmc_equals(),
            Self::ManaMin => b.cmc_range().map(|r| r.0),
            Self::ManaMax => b.cmc_range().map(|r| r.1),
            Self::PriceMin => b.price_min(),
            Self::PriceMax => b.price_max(),
        };
        value.map(|v| v.to_string()).unwrap_or_default()
    }
}

/// A stepper row: one value, or a range's two ends on one line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum StepRow {
    Exact(Stat),
    Range(Stat),
}

/// One row of an open section.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Row {
    /// A small label with an optional mode chip and clear button. `controls` marks a label that can show either, which keeps the focus on it steady as they come and go.
    Label {
        text: &'static str,
        mode: Option<ModeChip>,
        clear: Option<Clear>,
        controls: bool,
    },
    Text(TextField),
    Number(NumberField),
    Chips(ChipSet),
    Stepper(StepRow),
    Picker(PickerField),
}

impl Row {
    /// Whether the row takes focus: every control, and a label that can show a chip or a clear button.
    pub(crate) fn focusable(self) -> bool {
        match self {
            Self::Label { controls, .. } => controls,
            _ => true,
        }
    }
}

/// A label that shows its clear button only while there is something to clear.
fn label(b: &CardQueryBuilder, text: &'static str, mode: Option<ModeChip>, clear: Clear) -> Row {
    Row::Label {
        text,
        mode,
        clear: clear.applies(b).then_some(clear),
        controls: true,
    }
}

/// A label whose mode chip and clear button show only once something is picked.
fn label_when_set(b: &CardQueryBuilder, text: &'static str, mode: ModeChip, clear: Clear) -> Row {
    let set = clear.applies(b);
    Row::Label {
        text,
        mode: set.then_some(mode),
        clear: set.then_some(clear),
        controls: true,
    }
}

/// A contains row and a doesn't-contain row, each under its label.
fn text_pair(b: &CardQueryBuilder, contains: TextField, not: TextField) -> Vec<Row> {
    vec![
        label(b, contains.placeholder(), None, Clear::Text(contains)),
        Row::Text(contains),
        label(b, not.placeholder(), None, Clear::Text(not)),
        Row::Text(not),
    ]
}

/// An include picker and an exclude picker for `group`, each under its label.
fn include_exclude(
    b: &CardQueryBuilder,
    group: MatchGroup,
    include: &'static str,
    exclude: &'static str,
) -> [Row; 4] {
    [
        label(
            b,
            include,
            Some(ModeChip::Match(group)),
            Clear::Includes(group),
        ),
        Row::Picker(PickerField::Include(group)),
        label(b, exclude, None, Clear::Excludes(group.exclude())),
        Row::Picker(PickerField::Exclude(group)),
    ]
}

/// The Oracle text section: the text pair, then Oracle word and keyword pickers.
fn oracle_text_rows(b: &CardQueryBuilder) -> Vec<Row> {
    let mut rows = text_pair(b, TextField::OracleContains, TextField::OracleNotContains);
    rows.extend(include_exclude(
        b,
        MatchGroup::OracleWords,
        "Oracle words include",
        "Oracle words exclude",
    ));
    rows.extend(include_exclude(
        b,
        MatchGroup::Keywords,
        "Keywords include",
        "Keywords exclude",
    ));
    rows
}

/// The Types section: basic type chips, then type line pickers.
fn types_rows(b: &CardQueryBuilder) -> Vec<Row> {
    let mut rows = vec![
        label_when_set(
            b,
            "Basic types include",
            ModeChip::Match(MatchGroup::BasicTypes),
            Clear::Includes(MatchGroup::BasicTypes),
        ),
        Row::Chips(ChipSet::BasicIncludes),
        label(
            b,
            "Basic types exclude",
            None,
            Clear::Excludes(Exclude::BasicTypes),
        ),
        Row::Chips(ChipSet::BasicExcludes),
    ];
    rows.extend(include_exclude(
        b,
        MatchGroup::TypeLine,
        "Type line include",
        "Type line exclude",
    ));
    rows
}

/// The Mana section: mana value inputs, color identity, then produced colors.
fn mana_rows(b: &CardQueryBuilder, modes: &Modes) -> Vec<Row> {
    let mut rows = vec![label(
        b,
        "Mana value",
        Some(ModeChip::Range(RangeGroup::ManaValue)),
        Clear::Range(RangeGroup::ManaValue),
    )];
    match modes.range_of(b, RangeGroup::ManaValue) {
        Exactness::Exact => rows.push(Row::Number(NumberField::ManaValue)),
        Exactness::Range => {
            rows.push(Row::Number(NumberField::ManaMin));
            rows.push(Row::Number(NumberField::ManaMax));
        }
    }
    rows.extend([
        label(
            b,
            "Color identity",
            Some(ModeChip::Range(RangeGroup::ColorIdentity)),
            Clear::Range(RangeGroup::ColorIdentity),
        ),
        Row::Chips(ChipSet::Colors),
        label(
            b,
            "Produces",
            Some(ModeChip::Match(MatchGroup::Produced)),
            Clear::Includes(MatchGroup::Produced),
        ),
        Row::Chips(ChipSet::ProducedIncludes),
        label(
            b,
            "Doesn't produce",
            None,
            Clear::Excludes(MatchGroup::Produced.exclude()),
        ),
        Row::Chips(ChipSet::ProducedExcludes),
    ]);
    rows
}

/// The Combat section: a labeled stepper each for power and toughness.
fn combat_rows(b: &CardQueryBuilder, modes: &Modes) -> Vec<Row> {
    [(Stat::Power, "Power"), (Stat::Toughness, "Toughness")]
        .into_iter()
        .flat_map(|(stat, text)| {
            let group = stat.group();
            let stepper = match modes.range_of(b, group) {
                Exactness::Exact => StepRow::Exact(stat),
                Exactness::Range => StepRow::Range(stat),
            };
            [
                label(b, text, Some(ModeChip::Range(group)), Clear::Range(group)),
                Row::Stepper(stepper),
            ]
        })
        .collect()
}

/// The Card roles section: include and exclude chips.
fn card_roles_rows(b: &CardQueryBuilder) -> Vec<Row> {
    vec![
        label_when_set(
            b,
            "Card roles include",
            ModeChip::Match(MatchGroup::Roles),
            Clear::Includes(MatchGroup::Roles),
        ),
        Row::Chips(ChipSet::RolesInclude),
        label(
            b,
            "Card roles exclude",
            None,
            Clear::Excludes(MatchGroup::Roles.exclude()),
        ),
        Row::Chips(ChipSet::RolesExclude),
    ]
}

/// The Oracle tags include label.
const TAGS_INCLUDE: &str = "Oracle tags include";

/// The Oracle tags exclude label.
const TAGS_EXCLUDE: &str = "Oracle tags exclude";

/// The tag list a row's Dictionary chip adopts into: the Oracle tags include or exclude label, or the picker under it.
pub(crate) fn dictionary_for(row: Row) -> Option<PickerField> {
    let include = PickerField::Include(MatchGroup::Tags);
    let exclude = PickerField::Exclude(MatchGroup::Tags);
    match row {
        Row::Label { text, .. } if text == TAGS_INCLUDE => Some(include),
        Row::Label { text, .. } if text == TAGS_EXCLUDE => Some(exclude),
        Row::Picker(field) if field == include || field == exclude => Some(field),
        _ => None,
    }
}

/// The Oracle tags section: include and exclude pickers, each label with its Dictionary chip.
fn oracle_tags_rows(b: &CardQueryBuilder) -> Vec<Row> {
    vec![
        label_when_set(
            b,
            TAGS_INCLUDE,
            ModeChip::Match(MatchGroup::Tags),
            Clear::Includes(MatchGroup::Tags),
        ),
        Row::Picker(PickerField::Include(MatchGroup::Tags)),
        label(
            b,
            TAGS_EXCLUDE,
            None,
            Clear::Excludes(MatchGroup::Tags.exclude()),
        ),
        Row::Picker(PickerField::Exclude(MatchGroup::Tags)),
    ]
}

/// The Config section: a labeled yes, no or any chip row per toggle.
fn config_rows() -> Vec<Row> {
    Toggle::ALL
        .into_iter()
        .flat_map(|toggle| {
            [
                Row::Label {
                    text: toggle.label(),
                    mode: None,
                    clear: None,
                    controls: false,
                },
                Row::Chips(ChipSet::Config(toggle)),
            ]
        })
        .collect()
}

/// The rows of `section` as the builder and modes stand.
pub(crate) fn rows(section: Section, b: &CardQueryBuilder, modes: &Modes) -> Vec<Row> {
    use Section::{
        Artist, CardRoles, Combat, Config, FlavorText, Format, Mana, Name, OracleTags, OracleText,
        Price, Rarity, Set, Sort, Types,
    };
    match section {
        Name => text_pair(b, TextField::NameContains, TextField::NameNotContains),
        OracleText => oracle_text_rows(b),
        Types => types_rows(b),
        Mana => mana_rows(b, modes),
        Combat => combat_rows(b, modes),
        FlavorText => text_pair(b, TextField::FlavorContains, TextField::FlavorNotContains),
        Artist => vec![
            label_when_set(
                b,
                "Artist",
                ModeChip::Side(SideGroup::Artist),
                Clear::Side(SideGroup::Artist),
            ),
            Row::Picker(PickerField::Side(SideGroup::Artist)),
        ],
        Rarity => vec![
            label_when_set(
                b,
                "Rarity",
                ModeChip::Side(SideGroup::Rarity),
                Clear::Side(SideGroup::Rarity),
            ),
            Row::Chips(ChipSet::Rarity),
        ],
        CardRoles => card_roles_rows(b),
        OracleTags => oracle_tags_rows(b),
        Set => vec![
            label_when_set(
                b,
                "Set",
                ModeChip::Side(SideGroup::Set),
                Clear::Side(SideGroup::Set),
            ),
            Row::Picker(PickerField::Side(SideGroup::Set)),
        ],
        Format => vec![
            label(b, "Is commander in", None, Clear::CommanderIn),
            Row::Chips(ChipSet::CommanderIn),
            label(b, "Is legal in", None, Clear::Legal),
            Row::Chips(ChipSet::LegalIn),
        ],
        Price => vec![
            label(b, "Price", None, Clear::Price),
            Row::Chips(ChipSet::Currency),
            Row::Number(NumberField::PriceMin),
            Row::Number(NumberField::PriceMax),
        ],
        Sort => vec![
            label(b, "Sort by", None, Clear::Sort),
            Row::Chips(ChipSet::SortBy),
            Row::Label {
                text: "Sort order",
                mode: None,
                clear: None,
                controls: false,
            },
            Row::Chips(ChipSet::SortOrder),
        ],
        Config => config_rows(),
    }
}

/// The price currency the sheet shows: the builder's, else USD.
pub(crate) fn currency(b: &CardQueryBuilder) -> PriceCurrency {
    b.price_currency().unwrap_or(PriceCurrency::Usd)
}

#[cfg(test)]
mod tests {
    use zwipe_core::domain::card::search_card::card_filter::CardSortKey;

    use super::*;

    #[test]
    fn a_default_builder_has_no_active_section() {
        let b = CardQueryBuilder::default();
        for section in Section::ALL {
            assert!(!section.is_active(&b), "{section:?}");
        }
    }

    #[test]
    fn each_section_clears_what_it_shows() {
        let mut b = CardQueryBuilder::default();
        b.set_name_contains("bolt")
            .set_cmc_equals(1.0)
            .set_sort(CardSortKey::Name)
            .set_is_playable(false)
            .set_price_min(1.0);
        b.set_type_line_contains("goblin");
        for section in [
            Section::Name,
            Section::Mana,
            Section::Sort,
            Section::Price,
            Section::Types,
        ] {
            assert!(section.is_active(&b), "{section:?}");
            section.clear(&mut b);
            assert!(!section.is_active(&b), "{section:?}");
        }
        assert!(Section::Config.is_active(&b));
        Section::Config.clear(&mut b);
        assert_eq!(b.promo(), Some(false), "the config clear sets promo to No");
        assert!(
            Section::Config.is_active(&b),
            "so Config still reads as set, as in the app"
        );
    }

    #[test]
    fn currency_alone_is_not_active() {
        let mut b = CardQueryBuilder::default();
        b.set_price_currency(PriceCurrency::Eur);
        assert!(!Section::Price.is_active(&b));
        assert_eq!(currency(&b), PriceCurrency::Eur);
    }

    #[test]
    fn format_shows_only_when_asked() {
        assert_eq!(Section::shown(false).len(), 14);
        assert!(Section::shown(true).contains(&Section::Format));
    }

    #[test]
    fn a_label_shows_its_clear_only_when_set_and_its_chip_by_rule() {
        let mut b = CardQueryBuilder::default();
        let modes = Modes::default();
        let rows = rows(Section::Types, &b, &modes);
        assert_eq!(
            rows[0],
            Row::Label {
                text: "Basic types include",
                mode: None,
                clear: None,
                controls: true,
            },
            "no chip until something is picked"
        );
        assert!(rows[0].focusable());
        assert_eq!(
            rows[4],
            Row::Label {
                text: "Type line include",
                mode: Some(ModeChip::Match(MatchGroup::TypeLine)),
                clear: None,
                controls: true,
            },
            "type line always shows its chip"
        );
        b.set_card_type_contains_any(vec![
            zwipe_core::domain::card::search_card::card_type::CardType::Land,
        ]);
        let rows = super::rows(Section::Types, &b, &modes);
        assert!(matches!(
            rows[0],
            Row::Label {
                mode: Some(_),
                clear: Some(_),
                ..
            }
        ));
    }

    #[test]
    fn mode_chips_stick_while_empty_and_move_values_once_set() {
        let mut b = CardQueryBuilder::default();
        let mut modes = Modes::default();
        modes.flip(&mut b, ModeChip::Range(RangeGroup::ManaValue));
        assert_eq!(
            rows(Section::Mana, &b, &modes)[1..3],
            [
                Row::Number(NumberField::ManaMin),
                Row::Number(NumberField::ManaMax)
            ]
        );
        b.set_keywords_contains_any(vec!["flying".to_owned()]);
        modes.flip(&mut b, ModeChip::Match(MatchGroup::Keywords));
        assert_eq!(b.keywords_contains_all(), Some(&["flying".to_owned()][..]));
        assert_eq!(
            modes.label(&b, ModeChip::Match(MatchGroup::Keywords)),
            "All"
        );
        modes.flip(&mut b, ModeChip::Side(SideGroup::Set));
        assert_eq!(modes.label(&b, ModeChip::Side(SideGroup::Set)), "Exclude");
    }

    #[test]
    fn combat_rows_follow_each_stats_mode() {
        let mut b = CardQueryBuilder::default();
        b.set_toughness_range((1, 2));
        let rows = rows(Section::Combat, &b, &Modes::default());
        assert_eq!(rows[1], Row::Stepper(StepRow::Exact(Stat::Power)));
        assert_eq!(rows[3], Row::Stepper(StepRow::Range(Stat::Toughness)));
    }
}
