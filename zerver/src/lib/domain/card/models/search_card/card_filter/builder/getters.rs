//! Getter methods for accessing card filter values.
//!
//! All getters return `Option` types since filters may be unset.
//! String fields return `Option<&str>`, collections return `Option<&[T]>`.

use crate::domain::card::models::{
    scryfall_data::{colors::Colors, rarity::Rarities},
    search_card::{card_filter::{builder::CardFilterBuilder, OrderByOptions}, card_type::CardType},
};

impl CardFilterBuilder {
    // =================================
    // Text Filter Getters
    // =================================

    /// Returns the name filter value.
    pub fn name_contains(&self) -> Option<&str> {
        self.name_contains.as_deref()
    }

    /// Returns the oracle text filter value.
    pub fn oracle_text_contains(&self) -> Option<&str> {
        self.oracle_text_contains.as_deref()
    }

    /// Returns the flavor text filter value.
    pub fn flavor_text_contains(&self) -> Option<&str> {
        self.flavor_text_contains.as_deref()
    }

    /// Returns the has_flavor_text filter value.
    pub fn has_flavor_text(&self) -> Option<bool> {
        self.has_flavor_text
    }

    // =================================
    // Type Filter Getters
    // =================================

    /// Returns the type line filter value.
    pub fn type_line_contains(&self) -> Option<&str> {
        self.type_line_contains.as_deref()
    }

    /// Returns the type_line_contains_any filter value.
    pub fn type_line_contains_any(&self) -> Option<&[String]> {
        self.type_line_contains_any.as_deref()
    }

    /// Returns the card_type_contains_any filter value.
    pub fn card_type_contains_any(&self) -> Option<&[CardType]> {
        self.card_type_contains_any.as_deref()
    }

    // =================================
    // Metadata Filter Getters
    // =================================

    /// Returns the set filter value.
    pub fn set_equals_any(&self) -> Option<&[String]> {
        self.set_equals_any.as_deref()
    }

    /// Returns the artist filter value.
    pub fn artist_equals_any(&self) -> Option<&[String]> {
        self.artist_equals_any.as_deref()
    }

    /// Returns the rarity filter value.
    pub fn rarity_equals_any(&self) -> Option<&Rarities> {
        self.rarity_equals_any.as_ref()
    }

    // =================================
    // Mana Filter Getters
    // =================================

    /// Returns the CMC exact match filter value.
    pub fn cmc_equals(&self) -> Option<f64> {
        self.cmc_equals
    }

    /// Returns the CMC range filter value.
    pub fn cmc_range(&self) -> Option<(f64, f64)> {
        self.cmc_range
    }

    /// Returns the color_identity_equals filter value.
    pub fn color_identity_equals(&self) -> Option<&Colors> {
        self.color_identity_equals.as_ref()
    }

    /// Returns the color_identity_within filter value.
    pub fn color_identity_within(&self) -> Option<&Colors> {
        self.color_identity_within.as_ref()
    }

    // =================================
    // Combat Stat Getters
    // =================================

    /// Returns the power exact match filter value.
    pub fn power_equals(&self) -> Option<i32> {
        self.power_equals
    }

    /// Returns the power range filter value.
    pub fn power_range(&self) -> Option<(i32, i32)> {
        self.power_range
    }

    /// Returns the toughness exact match filter value.
    pub fn toughness_equals(&self) -> Option<i32> {
        self.toughness_equals
    }

    /// Returns the toughness range filter value.
    pub fn toughness_range(&self) -> Option<(i32, i32)> {
        self.toughness_range
    }

    // =================================
    // Card Flag Getters
    // =================================

    /// Returns the is_valid_commander filter value.
    pub fn is_valid_commander(&self) -> Option<bool> {
        self.is_valid_commander
    }

    /// Returns the is_token filter value.
    pub fn is_token(&self) -> Option<bool> {
        self.is_token
    }

    /// Returns the is_playable filter value.
    pub fn is_playable(&self) -> Option<bool> {
        self.is_playable
    }

    /// Returns the digital filter value.
    pub fn digital(&self) -> Option<bool> {
        self.digital
    }

    /// Returns the oversized filter value.
    pub fn oversized(&self) -> Option<bool> {
        self.oversized
    }

    /// Returns the promo filter value.
    pub fn promo(&self) -> Option<bool> {
        self.promo
    }

    /// Returns the content_warning filter value.
    pub fn content_warning(&self) -> Option<bool> {
        self.content_warning
    }

    /// Returns the language filter value.
    pub fn language(&self) -> Option<&str> {
        self.language.as_deref()
    }

    // =================================
    // Pagination & Config Getters
    // =================================

    /// Returns the result limit.
    pub fn limit(&self) -> u32 {
        self.limit
    }

    /// Returns the result offset.
    pub fn offset(&self) -> u32 {
        self.offset
    }

    /// Returns the order_by value.
    pub fn order_by(&self) -> Option<OrderByOptions> {
        self.order_by
    }

    /// Returns the ascending sort direction flag.
    pub fn ascending(&self) -> bool {
        self.ascending
    }
}
