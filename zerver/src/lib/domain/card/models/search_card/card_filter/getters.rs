use crate::domain::card::models::{
    scryfall_data::{colors::Colors, language::Language, rarity::Rarities},
    search_card::{card_filter::{CardFilter, OrderByOptions}, card_type::CardType},
};

impl CardFilter {
    // text
    pub fn name_contains(&self) -> Option<&str> {
        self.name_contains.as_deref()
    }

    pub fn oracle_text_contains(&self) -> Option<&str> {
        self.oracle_text_contains.as_deref()
    }

    pub fn flavor_text_contains(&self) -> Option<&str> {
        self.flavor_text_contains.as_deref()
    }

    pub fn has_flavor_text(&self) -> Option<bool> {
        self.has_flavor_text
    }

    // types
    pub fn type_line_contains(&self) -> Option<&str> {
        self.type_line_contains.as_deref()
    }

    pub fn type_line_contains_any(&self) -> Option<&[String]> {
        self.type_line_contains_any.as_deref()
    }

    pub fn card_type_contains_any(&self) -> Option<&[CardType]> {
        self.card_type_contains_any.as_deref()
    }

    // rarity
    pub fn rarity_equals_any(&self) -> Option<&Rarities> {
        self.rarity_equals_any.as_ref()
    }

    // set
    pub fn set_equals_any(&self) -> Option<&[String]> {
        self.set_equals_any.as_deref()
    }

    // mana
    pub fn cmc_equals(&self) -> Option<f64> {
        self.cmc_equals
    }

    pub fn cmc_range(&self) -> Option<(f64, f64)> {
        self.cmc_range
    }

    pub fn color_identity_equals(&self) -> Option<&Colors> {
        self.color_identity_equals.as_ref()
    }

    pub fn color_identity_within(&self) -> Option<&Colors> {
        self.color_identity_within.as_ref()
    }

    // combat
    pub fn power_equals(&self) -> Option<i32> {
        self.power_equals
    }

    pub fn power_range(&self) -> Option<(i32, i32)> {
        self.power_range
    }

    pub fn toughness_equals(&self) -> Option<i32> {
        self.toughness_equals
    }

    pub fn toughness_range(&self) -> Option<(i32, i32)> {
        self.toughness_range
    }

    // flags
    pub fn is_valid_commander(&self) -> Option<bool> {
        self.is_valid_commander
    }

    pub fn is_token(&self) -> Option<bool> {
        self.is_token
    }

    pub fn is_playable(&self) -> Option<bool> {
        self.is_playable
    }

    pub fn digital(&self) -> Option<bool> {
        self.digital
    }

    pub fn oversized(&self) -> Option<bool> {
        self.oversized
    }

    pub fn promo(&self) -> Option<bool> {
        self.promo
    }

    pub fn content_warning(&self) -> Option<bool> {
        self.content_warning
    }

    pub fn language(&self) -> Option<Language> {
        self.language
    }

    // config
    pub fn limit(&self) -> u32 {
        self.limit
    }

    pub fn offset(&self) -> u32 {
        self.offset
    }

    pub fn order_by(&self) -> Option<OrderByOptions> {
        self.order_by
    }

    pub fn ascending(&self) -> bool {
        self.ascending
    }
}
