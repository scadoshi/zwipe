//! Setter methods for modifying card filter values.
//!
//! All setters return `&mut Self` for method chaining. Most setters
//! have a corresponding `unset_*` method to clear the filter.
//!
//! # Empty String Handling
//!
//! Text filters (`set_name_contains`, etc.) treat empty strings as `None`
//! to avoid ineffective filters.

use super::{CardFilterBuilder, CardType, Colors, OrderByOptions};
use crate::domain::card::models::scryfall_data::rarity::Rarities;

impl CardFilterBuilder {
    // =================================
    // Text Filter Setters
    // =================================

    /// Sets card name filter (case-insensitive substring). Empty strings = None.
    pub fn set_name_contains(&mut self, name_contains: impl Into<String>) -> &mut Self {
        let s = name_contains.into();
        self.name_contains = if s.is_empty() { None } else { Some(s) };
        self
    }

    /// Clears the name filter.
    pub fn unset_name_contains(&mut self) -> &mut Self {
        self.name_contains = None;
        self
    }

    /// Sets oracle text filter (ability text substring). Empty strings = None.
    pub fn set_oracle_text_contains(
        &mut self,
        oracle_text_contains: impl Into<String>,
    ) -> &mut Self {
        let s = oracle_text_contains.into();
        self.oracle_text_contains = if s.is_empty() { None } else { Some(s) };
        self
    }

    /// Clears the oracle text filter.
    pub fn unset_oracle_text_contains(&mut self) -> &mut Self {
        self.oracle_text_contains = None;
        self
    }

    /// Sets flavor text filter.
    pub fn set_flavor_text_contains(
        &mut self,
        flavor_text_contains: impl Into<String>,
    ) -> &mut Self {
        self.flavor_text_contains = Some(flavor_text_contains.into());
        self
    }

    /// Clears the flavor text filter.
    pub fn unset_flavor_text_contains(&mut self) -> &mut Self {
        self.flavor_text_contains = None;
        self
    }

    /// Sets filter for presence/absence of flavor text.
    pub fn set_has_flavor_text(&mut self, has_flavor_text: bool) -> &mut Self {
        self.has_flavor_text = Some(has_flavor_text);
        self
    }

    /// Clears the has_flavor_text filter.
    pub fn unset_has_flavor_text(&mut self) -> &mut Self {
        self.has_flavor_text = None;
        self
    }

    // =================================
    // Type Filter Setters
    // =================================

    /// Sets type line filter (e.g., "Legendary Creature"). Empty strings = None.
    pub fn set_type_line_contains(&mut self, type_line_contains: impl Into<String>) -> &mut Self {
        let s = type_line_contains.into();
        self.type_line_contains = if s.is_empty() { None } else { Some(s) };
        self
    }

    /// Clears the type line filter.
    pub fn unset_type_line_contains(&mut self) -> &mut Self {
        self.type_line_contains = None;
        self
    }

    /// Sets filter matching any of multiple type line substrings. Empty vec = None.
    pub fn set_type_line_contains_any<I, S>(&mut self, type_line_contains_any: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let v: Vec<String> = type_line_contains_any.into_iter().map(Into::into).collect();
        self.type_line_contains_any = if v.is_empty() { None } else { Some(v) };
        self
    }

    /// Clears the type_line_contains_any filter.
    pub fn unset_type_line_contains_any(&mut self) -> &mut Self {
        self.type_line_contains_any = None;
        self
    }

    /// Sets filter matching any of multiple card types (Creature, Instant, etc.). Empty vec = None.
    pub fn set_card_type_contains_any<I>(&mut self, card_type_contains_any: I) -> &mut Self
    where
        I: IntoIterator<Item = CardType>,
    {
        let v: Vec<CardType> = card_type_contains_any.into_iter().collect();
        self.card_type_contains_any = if v.is_empty() { None } else { Some(v) };
        self
    }

    /// Clears the card_type_contains_any filter.
    pub fn unset_card_type_contains_any(&mut self) -> &mut Self {
        self.card_type_contains_any = None;
        self
    }

    // =================================
    // Printing/Metadata Setters
    // =================================

    /// Sets filter matching any of multiple set codes (e.g., "MH2", "ONE"). Empty vec = None.
    pub fn set_set_equals_any(
        &mut self,
        set_equals_any: impl IntoIterator<Item = impl Into<String>>,
    ) -> &mut Self {
        let s: Vec<String> = set_equals_any.into_iter().map(|x| x.into()).collect();
        self.set_equals_any = if s.is_empty() { None } else { Some(s) };
        self
    }

    /// Clears the set_equals_any filter.
    pub fn unset_set_equals_any(&mut self) -> &mut Self {
        self.set_equals_any = None;
        self
    }

    /// Sets filter matching any of multiple artist names. Empty vec = None.
    pub fn set_artist_equals_any(
        &mut self,
        artist_equals_any: impl IntoIterator<Item = impl Into<String>>,
    ) -> &mut Self {
        let s: Vec<String> = artist_equals_any.into_iter().map(|x| x.into()).collect();
        self.artist_equals_any = if s.is_empty() { None } else { Some(s) };
        self
    }

    /// Clears the artist_equals_any filter.
    pub fn unset_artist_equals_any(&mut self) -> &mut Self {
        self.artist_equals_any = None;
        self
    }

    /// Sets filter matching any of multiple rarities. Empty = None.
    pub fn set_rarity_equals_any(&mut self, rarity_equals_any: Rarities) -> &mut Self {
        self.rarity_equals_any = if rarity_equals_any.is_empty() {
            None
        } else {
            Some(rarity_equals_any)
        };
        self
    }

    /// Clears the rarity_equals_any filter.
    pub fn unset_rarity_equals_any(&mut self) -> &mut Self {
        self.rarity_equals_any = None;
        self
    }

    // =================================
    // Mana Filter Setters
    // =================================

    /// Sets CMC exact match filter.
    pub fn set_cmc_equals(&mut self, cmc_equals: f64) -> &mut Self {
        self.cmc_equals = Some(cmc_equals);
        self
    }

    /// Clears the cmc_equals filter.
    pub fn unset_cmc_equals(&mut self) -> &mut Self {
        self.cmc_equals = None;
        self
    }

    /// Sets CMC range filter (inclusive).
    pub fn set_cmc_range(&mut self, cmc_range: (f64, f64)) -> &mut Self {
        self.cmc_range = Some(cmc_range);
        self
    }

    /// Clears the cmc_range filter.
    pub fn unset_cmc_range(&mut self) -> &mut Self {
        self.cmc_range = None;
        self
    }

    /// Sets exact color identity filter (e.g., exactly W+U).
    pub fn set_color_identity_equals(&mut self, color_identity_equals: Colors) -> &mut Self {
        self.color_identity_equals = Some(color_identity_equals);
        self
    }

    /// Clears the color_identity_equals filter.
    pub fn unset_color_identity_equals(&mut self) -> &mut Self {
        self.color_identity_equals = None;
        self
    }

    /// Sets color identity within filter (subset of provided colors).
    pub fn set_color_identity_within(&mut self, color_identity_contains_any: Colors) -> &mut Self {
        self.color_identity_within = Some(color_identity_contains_any);
        self
    }

    /// Clears the color_identity_within filter.
    pub fn unset_color_identity_within(&mut self) -> &mut Self {
        self.color_identity_within = None;
        self
    }

    // =================================
    // Combat Stat Setters
    // =================================

    /// Sets exact power filter.
    pub fn set_power_equals(&mut self, power_equals: i32) -> &mut Self {
        self.power_equals = Some(power_equals);
        self
    }

    /// Clears the power_equals filter.
    pub fn unset_power_equals(&mut self) -> &mut Self {
        self.power_equals = None;
        self
    }

    /// Sets power range filter (inclusive).
    pub fn set_power_range(&mut self, power_range: (i32, i32)) -> &mut Self {
        self.power_range = Some(power_range);
        self
    }

    /// Clears the power_range filter.
    pub fn unset_power_range(&mut self) -> &mut Self {
        self.power_range = None;
        self
    }

    /// Sets exact toughness filter.
    pub fn set_toughness_equals(&mut self, toughness_equals: i32) -> &mut Self {
        self.toughness_equals = Some(toughness_equals);
        self
    }

    /// Clears the toughness_equals filter.
    pub fn unset_toughness_equals(&mut self) -> &mut Self {
        self.toughness_equals = None;
        self
    }

    /// Sets toughness range filter (inclusive).
    pub fn set_toughness_range(&mut self, toughness_range: (i32, i32)) -> &mut Self {
        self.toughness_range = Some(toughness_range);
        self
    }

    /// Clears the toughness_range filter.
    pub fn unset_toughness_range(&mut self) -> &mut Self {
        self.toughness_range = None;
        self
    }

    // =================================
    // Card Flag Setters
    // =================================

    /// Sets filter for Commander format legality.
    pub fn set_is_valid_commander(&mut self, is_valid_commander: bool) -> &mut Self {
        self.is_valid_commander = Some(is_valid_commander);
        self
    }

    /// Clears the is_valid_commander filter.
    pub fn unset_is_valid_commander(&mut self) -> &mut Self {
        self.is_valid_commander = None;
        self
    }

    /// Sets filter for token status.
    pub fn set_is_token(&mut self, is_token: bool) -> &mut Self {
        self.is_token = Some(is_token);
        self
    }

    /// Clears the is_token filter.
    pub fn unset_is_token(&mut self) -> &mut Self {
        self.is_token = None;
        self
    }

    /// Sets filter for playability (excludes un-cards).
    pub fn set_is_playable(&mut self, is_playable: bool) -> &mut Self {
        self.is_playable = Some(is_playable);
        self
    }

    /// Clears the is_playable filter (defaults to true).
    pub fn unset_is_playable(&mut self) -> &mut Self {
        self.is_playable = None;
        self
    }

    /// Sets filter for digital-only cards.
    pub fn set_digital(&mut self, digital: bool) -> &mut Self {
        self.digital = Some(digital);
        self
    }

    /// Clears the digital filter (defaults to false).
    pub fn unset_digital(&mut self) -> &mut Self {
        self.digital = None;
        self
    }

    /// Sets filter for oversized cards.
    pub fn set_oversized(&mut self, oversized: bool) -> &mut Self {
        self.oversized = Some(oversized);
        self
    }

    /// Clears the oversized filter (defaults to false).
    pub fn unset_oversized(&mut self) -> &mut Self {
        self.oversized = None;
        self
    }

    /// Sets filter for promotional cards.
    pub fn set_promo(&mut self, promo: bool) -> &mut Self {
        self.promo = Some(promo);
        self
    }

    /// Clears the promo filter (defaults to false).
    pub fn unset_promo(&mut self) -> &mut Self {
        self.promo = None;
        self
    }

    /// Sets filter for content warning flag.
    pub fn set_content_warning(&mut self, content_warning: bool) -> &mut Self {
        self.content_warning = Some(content_warning);
        self
    }

    /// Clears the content_warning filter (defaults to false).
    pub fn unset_content_warning(&mut self) -> &mut Self {
        self.content_warning = None;
        self
    }

    /// Sets card language filter (e.g., "en", "ja").
    pub fn set_language(&mut self, language: impl Into<String>) -> &mut Self {
        self.language = Some(language.into());
        self
    }

    /// Clears the language filter (defaults to "en").
    pub fn unset_language(&mut self) -> &mut Self {
        self.language = None;
        self
    }

    // =================================
    // Pagination & Config Setters
    // =================================

    /// Sets result limit (max cards returned).
    pub fn set_limit(&mut self, limit: u32) -> &mut Self {
        self.limit = limit;
        self
    }

    /// Sets result offset (pagination).
    pub fn set_offset(&mut self, offset: u32) -> &mut Self {
        self.offset = offset;
        self
    }

    /// Sets result ordering (name, CMC, rarity, etc.).
    pub fn set_order_by(&mut self, order_by: OrderByOptions) -> &mut Self {
        self.order_by = Some(order_by);
        self
    }

    /// Clears the order_by filter.
    pub fn unset_order_by(&mut self) -> &mut Self {
        self.order_by = None;
        self
    }

    /// Sets sort direction (ascending vs descending).
    pub fn set_ascending(&mut self, ascending: bool) -> &mut Self {
        self.ascending = ascending;
        self
    }

    // =================================
    // Utility Methods
    // =================================

    /// Clears all search filters, keeps only config (limit, offset, flags, language).
    ///
    /// Useful for resetting search while preserving pagination state and defaults.
    pub fn retain_config(&mut self) -> &mut Self {
        let default = Self {
            limit: self.limit,
            offset: self.offset,
            is_valid_commander: self.is_valid_commander,
            is_token: self.is_token,
            is_playable: self.is_playable,
            digital: self.digital,
            oversized: self.oversized,
            promo: self.promo,
            content_warning: self.content_warning,
            language: self.language.clone(),
            order_by: self.order_by,
            ascending: self.ascending,
            ..Self::default()
        };
        *self = default;
        self
    }

    /// Resets to defaults (clears everything).
    ///
    /// Returns builder to initial state with default pagination and flags.
    pub fn clear(&mut self) -> &mut Self {
        *self = Self::default();
        self
    }
}
