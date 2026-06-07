//! Setter methods for modifying card filter values.
//!
//! All setters return `&mut Self` for method chaining. Most setters
//! have a corresponding `unset_*` method to clear the filter.
//!
//! # Empty String Handling
//!
//! Text filters (`set_name_contains`, etc.) treat empty strings as `None`
//! to avoid ineffective filters.

use super::{CardFilterBuilder, CardType, Colors, Format, OrderByOption};
use crate::domain::card::scryfall_data::rarity::Rarities;

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

    /// Sets filter matching any of multiple oracle text substrings. Empty vec = None.
    pub fn set_oracle_text_contains_any<I, S>(
        &mut self,
        oracle_text_contains_any: I,
    ) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let v: Vec<String> = oracle_text_contains_any.into_iter()
            .map(|s| s.into())
            .collect();
        self.oracle_text_contains_any = if v.is_empty() { None } else { Some(v) };
        self
    }

    /// Clears the oracle_text_contains_any filter.
    pub fn unset_oracle_text_contains_any(&mut self) -> &mut Self {
        self.oracle_text_contains_any = None;
        self
    }

    /// Sets filter requiring all of multiple oracle text substrings to be present. Empty vec = None.
    pub fn set_oracle_text_contains_all<I, S>(
        &mut self,
        oracle_text_contains_all: I,
    ) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let v: Vec<String> = oracle_text_contains_all.into_iter()
            .map(|s| s.into())
            .collect();
        self.oracle_text_contains_all = if v.is_empty() { None } else { Some(v) };
        self
    }

    /// Clears the oracle_text_contains_all filter.
    pub fn unset_oracle_text_contains_all(&mut self) -> &mut Self {
        self.oracle_text_contains_all = None;
        self
    }

    // =================================
    // Keywords Filter Setters
    // =================================

    /// Sets filter matching any of multiple keywords (OR logic). Empty vec = None.
    pub fn set_keywords_contains_any<I, S>(
        &mut self,
        keywords_contains_any: I,
    ) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let v: Vec<String> = keywords_contains_any.into_iter()
            .map(|s| s.into())
            .filter(|s| !s.is_empty())
            .collect();
        self.keywords_contains_any = if v.is_empty() { None } else { Some(v) };
        self
    }

    /// Clears the keywords_contains_any filter.
    pub fn unset_keywords_contains_any(&mut self) -> &mut Self {
        self.keywords_contains_any = None;
        self
    }

    /// Sets filter requiring all keywords to be present (AND logic). Empty vec = None.
    pub fn set_keywords_contains_all<I, S>(
        &mut self,
        keywords_contains_all: I,
    ) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let v: Vec<String> = keywords_contains_all.into_iter()
            .map(|s| s.into())
            .filter(|s| !s.is_empty())
            .collect();
        self.keywords_contains_all = if v.is_empty() { None } else { Some(v) };
        self
    }

    /// Clears the keywords_contains_all filter.
    pub fn unset_keywords_contains_all(&mut self) -> &mut Self {
        self.keywords_contains_all = None;
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
        let v: Vec<String> = type_line_contains_any.into_iter()
            .map(|s| s.into())
            .collect();
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

    /// Sets filter requiring all type line substrings to be present (AND logic). Empty vec = None.
    pub fn set_type_line_contains_all<I, S>(&mut self, type_line_contains_all: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let v: Vec<String> = type_line_contains_all.into_iter()
            .map(|s| s.into())
            .collect();
        self.type_line_contains_all = if v.is_empty() { None } else { Some(v) };
        self
    }

    /// Clears the type_line_contains_all filter.
    pub fn unset_type_line_contains_all(&mut self) -> &mut Self {
        self.type_line_contains_all = None;
        self
    }

    /// Sets filter requiring all card types to be present (AND logic). Empty vec = None.
    pub fn set_card_type_contains_all<I>(&mut self, card_type_contains_all: I) -> &mut Self
    where
        I: IntoIterator<Item = CardType>,
    {
        let v: Vec<CardType> = card_type_contains_all.into_iter().collect();
        self.card_type_contains_all = if v.is_empty() { None } else { Some(v) };
        self
    }

    /// Clears the card_type_contains_all filter.
    pub fn unset_card_type_contains_all(&mut self) -> &mut Self {
        self.card_type_contains_all = None;
        self
    }

    // =================================
    // Printing/Metadata Setters
    // =================================

    /// Sets filter matching any of multiple set names (e.g., "Modern Horizons 2"). Empty vec = None.
    pub fn set_set_equals_any(
        &mut self,
        set_equals_any: impl IntoIterator<Item = impl Into<String>>,
    ) -> &mut Self {
        let s: Vec<String> = set_equals_any.into_iter()
            .map(|x| x.into())
            .filter(|s| !s.is_empty())
            .collect();
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
        let s: Vec<String> = artist_equals_any.into_iter()
            .map(|x| x.into())
            .filter(|s| !s.is_empty())
            .collect();
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
    // Produced Mana Filter Setters
    // =================================

    /// Sets filter matching cards that produce any of the listed mana colors (OR logic). Empty vec = None.
    pub fn set_produced_mana_contains_any<I, S>(
        &mut self,
        produced_mana_contains_any: I,
    ) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let v: Vec<String> = produced_mana_contains_any
            .into_iter()
            .map(Into::into)
            .collect();
        self.produced_mana_contains_any = if v.is_empty() { None } else { Some(v) };
        self
    }

    /// Clears the produced_mana_contains_any filter.
    pub fn unset_produced_mana_contains_any(&mut self) -> &mut Self {
        self.produced_mana_contains_any = None;
        self
    }

    /// Sets filter matching cards that produce all of the listed mana colors (AND logic). Empty vec = None.
    pub fn set_produced_mana_contains_all<I, S>(
        &mut self,
        produced_mana_contains_all: I,
    ) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let v: Vec<String> = produced_mana_contains_all
            .into_iter()
            .map(Into::into)
            .collect();
        self.produced_mana_contains_all = if v.is_empty() { None } else { Some(v) };
        self
    }

    /// Clears the produced_mana_contains_all filter.
    pub fn unset_produced_mana_contains_all(&mut self) -> &mut Self {
        self.produced_mana_contains_all = None;
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
    // Legalities Filter Setters
    // =================================

    /// Sets filter matching cards legal in any of the provided formats. Empty vec = None.
    pub fn set_legalities_contains_any<I, S>(
        &mut self,
        legalities_contains_any: I,
    ) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let v: Vec<String> = legalities_contains_any.into_iter().map(Into::into).collect();
        self.legalities_contains_any = if v.is_empty() { None } else { Some(v) };
        self
    }

    /// Clears the legalities_contains_any filter.
    pub fn unset_legalities_contains_any(&mut self) -> &mut Self {
        self.legalities_contains_any = None;
        self
    }

    // =================================
    // Commander Filter Setters
    // =================================

    /// Sets the commander eligibility filter for a specific format.
    pub fn set_is_commander_in_format(&mut self, format: Format) -> &mut Self {
        self.is_commander_in_format = Some(format);
        self
    }

    /// Clears the commander eligibility filter.
    pub fn unset_is_commander_in_format(&mut self) -> &mut Self {
        self.is_commander_in_format = None;
        self
    }

    // =================================
    // Partner/Background/Spell Setters
    // =================================

    /// Sets the partner card filter.
    pub fn set_is_partner(&mut self, val: bool) -> &mut Self {
        self.is_partner = Some(val);
        self
    }

    /// Clears the partner card filter.
    pub fn unset_is_partner(&mut self) -> &mut Self {
        self.is_partner = None;
        self
    }

    /// Sets the background card filter.
    pub fn set_is_background(&mut self, val: bool) -> &mut Self {
        self.is_background = Some(val);
        self
    }

    /// Clears the background card filter.
    pub fn unset_is_background(&mut self) -> &mut Self {
        self.is_background = None;
        self
    }

    /// Sets the signature spell filter.
    pub fn set_is_signature_spell(&mut self, val: bool) -> &mut Self {
        self.is_signature_spell = Some(val);
        self
    }

    /// Clears the signature spell filter.
    pub fn unset_is_signature_spell(&mut self) -> &mut Self {
        self.is_signature_spell = None;
        self
    }

    // =================================
    // Mechanical Category Setters
    // =================================

    /// Sets the mechanical category ANY filter (cards matching at least one).
    pub fn set_mechanical_categories_contains_any<I, S>(&mut self, categories: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let v: Vec<String> = categories.into_iter().map(|s| s.into()).collect();
        self.mechanical_categories_contains_any = if v.is_empty() { None } else { Some(v) };
        self
    }

    /// Clears the mechanical category ANY filter.
    pub fn unset_mechanical_categories_contains_any(&mut self) -> &mut Self {
        self.mechanical_categories_contains_any = None;
        self
    }

    /// Sets the mechanical category ALL filter (cards matching every category).
    pub fn set_mechanical_categories_contains_all<I, S>(&mut self, categories: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let v: Vec<String> = categories.into_iter().map(|s| s.into()).collect();
        self.mechanical_categories_contains_all = if v.is_empty() { None } else { Some(v) };
        self
    }

    /// Clears the mechanical category ALL filter.
    pub fn unset_mechanical_categories_contains_all(&mut self) -> &mut Self {
        self.mechanical_categories_contains_all = None;
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
    pub fn set_order_by(&mut self, order_by: OrderByOption) -> &mut Self {
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
    // Exclude Filter Setters
    // =================================

    /// Sets name not-contains filter (punctuation-insensitive). Empty strings = None.
    pub fn set_name_not_contains(&mut self, name_not_contains: impl Into<String>) -> &mut Self {
        let s = name_not_contains.into();
        self.name_not_contains = if s.is_empty() { None } else { Some(s) };
        self
    }

    /// Clears the name not-contains filter.
    pub fn unset_name_not_contains(&mut self) -> &mut Self {
        self.name_not_contains = None;
        self
    }

    /// Sets oracle text not-contains filter (punctuation-insensitive). Empty strings = None.
    pub fn set_oracle_text_not_contains(&mut self, oracle_text_not_contains: impl Into<String>) -> &mut Self {
        let s = oracle_text_not_contains.into();
        self.oracle_text_not_contains = if s.is_empty() { None } else { Some(s) };
        self
    }

    /// Clears the oracle text not-contains filter.
    pub fn unset_oracle_text_not_contains(&mut self) -> &mut Self {
        self.oracle_text_not_contains = None;
        self
    }

    /// Sets oracle text excludes filter (punctuation-insensitive, "has none of these").
    pub fn set_oracle_text_excludes_any<I, S>(&mut self, oracle_text_excludes_any: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let v: Vec<String> = oracle_text_excludes_any.into_iter()
            .map(|s| s.into())
            .collect();
        self.oracle_text_excludes_any = if v.is_empty() { None } else { Some(v) };
        self
    }

    /// Clears the oracle text excludes filter.
    pub fn unset_oracle_text_excludes_any(&mut self) -> &mut Self {
        self.oracle_text_excludes_any = None;
        self
    }

    /// Sets keywords excludes filter ("has none of these keywords").
    pub fn set_keywords_excludes<I, S>(&mut self, keywords_excludes: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let v: Vec<String> = keywords_excludes.into_iter()
            .map(|s| s.into().trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        self.keywords_excludes = if v.is_empty() { None } else { Some(v) };
        self
    }

    /// Clears the keywords excludes filter.
    pub fn unset_keywords_excludes(&mut self) -> &mut Self {
        self.keywords_excludes = None;
        self
    }

    /// Sets flavor text not-contains filter (punctuation-insensitive). Empty strings = None.
    pub fn set_flavor_text_not_contains(&mut self, flavor_text_not_contains: impl Into<String>) -> &mut Self {
        let s = flavor_text_not_contains.into();
        self.flavor_text_not_contains = if s.is_empty() { None } else { Some(s) };
        self
    }

    /// Clears the flavor text not-contains filter.
    pub fn unset_flavor_text_not_contains(&mut self) -> &mut Self {
        self.flavor_text_not_contains = None;
        self
    }

    /// Sets type line not-contains filter (punctuation-insensitive). Empty strings = None.
    pub fn set_type_line_not_contains(&mut self, type_line_not_contains: impl Into<String>) -> &mut Self {
        let s = type_line_not_contains.into();
        self.type_line_not_contains = if s.is_empty() { None } else { Some(s) };
        self
    }

    /// Clears the type line not-contains filter.
    pub fn unset_type_line_not_contains(&mut self) -> &mut Self {
        self.type_line_not_contains = None;
        self
    }

    /// Sets type line excludes filter (punctuation-insensitive, "has none of these").
    pub fn set_type_line_excludes_any<I, S>(&mut self, type_line_excludes_any: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let v: Vec<String> = type_line_excludes_any.into_iter()
            .map(|s| s.into())
            .collect();
        self.type_line_excludes_any = if v.is_empty() { None } else { Some(v) };
        self
    }

    /// Clears the type line excludes filter.
    pub fn unset_type_line_excludes_any(&mut self) -> &mut Self {
        self.type_line_excludes_any = None;
        self
    }

    /// Sets card type excludes filter ("has none of these card types").
    pub fn set_card_type_excludes_any<I>(&mut self, card_type_excludes_any: I) -> &mut Self
    where
        I: IntoIterator<Item = CardType>,
    {
        let v: Vec<CardType> = card_type_excludes_any.into_iter().collect();
        self.card_type_excludes_any = if v.is_empty() { None } else { Some(v) };
        self
    }

    /// Clears the card type excludes filter.
    pub fn unset_card_type_excludes_any(&mut self) -> &mut Self {
        self.card_type_excludes_any = None;
        self
    }

    /// Sets mechanical categories excludes filter ("has none of these categories").
    pub fn set_mechanical_categories_excludes<I, S>(&mut self, mechanical_categories_excludes: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let v: Vec<String> = mechanical_categories_excludes.into_iter()
            .map(|s| s.into().trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        self.mechanical_categories_excludes = if v.is_empty() { None } else { Some(v) };
        self
    }

    /// Clears the mechanical categories excludes filter.
    pub fn unset_mechanical_categories_excludes(&mut self) -> &mut Self {
        self.mechanical_categories_excludes = None;
        self
    }

    /// Sets produced mana excludes filter ("does not produce any of these").
    pub fn set_produced_mana_excludes<I, S>(&mut self, produced_mana_excludes: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let v: Vec<String> = produced_mana_excludes.into_iter()
            .map(|s| s.into().trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        self.produced_mana_excludes = if v.is_empty() { None } else { Some(v) };
        self
    }

    /// Clears the produced mana excludes filter.
    pub fn unset_produced_mana_excludes(&mut self) -> &mut Self {
        self.produced_mana_excludes = None;
        self
    }

    /// Sets rarity excludes filter ("not any of these rarities").
    pub fn set_rarity_excludes_any(&mut self, rarity_excludes_any: Rarities) -> &mut Self {
        self.rarity_excludes_any = if rarity_excludes_any.is_empty() {
            None
        } else {
            Some(rarity_excludes_any)
        };
        self
    }

    /// Clears the rarity excludes filter.
    pub fn unset_rarity_excludes_any(&mut self) -> &mut Self {
        self.rarity_excludes_any = None;
        self
    }

    /// Sets set excludes filter ("not from any of these sets").
    pub fn set_set_excludes_any(
        &mut self,
        set_excludes_any: impl IntoIterator<Item = impl Into<String>>,
    ) -> &mut Self {
        let s: Vec<String> = set_excludes_any.into_iter()
            .map(|x| x.into().trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        self.set_excludes_any = if s.is_empty() { None } else { Some(s) };
        self
    }

    /// Clears the set excludes filter.
    pub fn unset_set_excludes_any(&mut self) -> &mut Self {
        self.set_excludes_any = None;
        self
    }

    /// Sets artist excludes filter ("not by any of these artists").
    pub fn set_artist_excludes_any(
        &mut self,
        artist_excludes_any: impl IntoIterator<Item = impl Into<String>>,
    ) -> &mut Self {
        let s: Vec<String> = artist_excludes_any.into_iter()
            .map(|x| x.into().trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        self.artist_excludes_any = if s.is_empty() { None } else { Some(s) };
        self
    }

    /// Clears the artist excludes filter.
    pub fn unset_artist_excludes_any(&mut self) -> &mut Self {
        self.artist_excludes_any = None;
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
