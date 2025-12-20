use crate::domain::card::models::{
    scryfall_data::colors::Colors,
    search_card::{card_filter::CardFilter, card_type::CardType},
};

impl CardFilter {
    // text
    pub fn name_contains(&self) -> Option<&str> {
        self.name_contains.as_deref()
    }

    pub fn oracle_text_contains(&self) -> Option<&str> {
        self.oracle_text_contains.as_deref()
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

    // printing
    pub fn set_contains(&self) -> Option<&str> {
        self.set_contains.as_deref()
    }

    pub fn rarity_contains(&self) -> Option<&str> {
        self.rarity_contains.as_deref()
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

    // config
    pub fn limit(&self) -> u32 {
        self.limit
    }

    pub fn offset(&self) -> u32 {
        self.offset
    }
}
