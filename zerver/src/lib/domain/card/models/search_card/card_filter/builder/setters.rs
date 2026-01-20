use super::{CardFilterBuilder, CardType, Colors, OrderByOptions};
use crate::domain::card::models::scryfall_data::rarity::Rarities;

impl CardFilterBuilder {
    // text
    pub fn set_name_contains(&mut self, name_contains: impl Into<String>) -> &mut Self {
        let s = name_contains.into();
        self.name_contains = if s.is_empty() { None } else { Some(s) };
        self
    }

    pub fn unset_name_contains(&mut self) -> &mut Self {
        self.name_contains = None;
        self
    }

    pub fn set_oracle_text_contains(
        &mut self,
        oracle_text_contains: impl Into<String>,
    ) -> &mut Self {
        let s = oracle_text_contains.into();
        self.oracle_text_contains = if s.is_empty() { None } else { Some(s) };
        self
    }

    pub fn unset_oracle_text_contains(&mut self) -> &mut Self {
        self.oracle_text_contains = None;
        self
    }

    pub fn set_flavor_text_contains(
        &mut self,
        flavor_text_contains: impl Into<String>,
    ) -> &mut Self {
        self.flavor_text_contains = Some(flavor_text_contains.into());
        self
    }

    pub fn unset_flavor_text_contains(&mut self) -> &mut Self {
        self.flavor_text_contains = None;
        self
    }

    pub fn set_has_flavor_text(&mut self, has_flavor_text: bool) -> &mut Self {
        self.has_flavor_text = Some(has_flavor_text);
        self
    }

    pub fn unset_has_flavor_text(&mut self) -> &mut Self {
        self.has_flavor_text = None;
        self
    }

    // types
    pub fn set_type_line_contains(&mut self, type_line_contains: impl Into<String>) -> &mut Self {
        let s = type_line_contains.into();
        self.type_line_contains = if s.is_empty() { None } else { Some(s) };
        self
    }

    pub fn unset_type_line_contains(&mut self) -> &mut Self {
        self.type_line_contains = None;
        self
    }

    pub fn set_type_line_contains_any<I, S>(&mut self, type_line_contains_any: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let v: Vec<String> = type_line_contains_any.into_iter().map(Into::into).collect();
        self.type_line_contains_any = if v.is_empty() { None } else { Some(v) };
        self
    }

    pub fn unset_type_line_contains_any(&mut self) -> &mut Self {
        self.type_line_contains_any = None;
        self
    }

    pub fn set_card_type_contains_any<I>(&mut self, card_type_contains_any: I) -> &mut Self
    where
        I: IntoIterator<Item = CardType>,
    {
        let v: Vec<CardType> = card_type_contains_any.into_iter().collect();
        self.card_type_contains_any = if v.is_empty() { None } else { Some(v) };
        self
    }

    pub fn unset_card_type_contains_any(&mut self) -> &mut Self {
        self.card_type_contains_any = None;
        self
    }

    // printing
    pub fn set_set_equals_any(
        &mut self,
        set_equals_any: impl IntoIterator<Item = impl Into<String>>,
    ) -> &mut Self {
        let s: Vec<String> = set_equals_any.into_iter().map(|x| x.into()).collect();
        self.set_equals_any = if s.is_empty() { None } else { Some(s) };
        self
    }

    pub fn unset_set_equals_any(&mut self) -> &mut Self {
        self.set_equals_any = None;
        self
    }

    pub fn set_artist_equals_any(
        &mut self,
        artist_equals_any: impl IntoIterator<Item = impl Into<String>>,
    ) -> &mut Self {
        let s: Vec<String> = artist_equals_any.into_iter().map(|x| x.into()).collect();
        self.artist_equals_any = if s.is_empty() { None } else { Some(s) };
        self
    }

    pub fn unset_artist_equals_any(&mut self) -> &mut Self {
        self.artist_equals_any = None;
        self
    }

    pub fn set_rarity_equals_any(&mut self, rarity_equals_any: Rarities) -> &mut Self {
        self.rarity_equals_any = if rarity_equals_any.is_empty() {
            None
        } else {
            Some(rarity_equals_any)
        };
        self
    }

    pub fn unset_rarity_equals_any(&mut self) -> &mut Self {
        self.rarity_equals_any = None;
        self
    }

    // mana
    pub fn set_cmc_equals(&mut self, cmc_equals: f64) -> &mut Self {
        self.cmc_equals = Some(cmc_equals);
        self
    }

    pub fn unset_cmc_equals(&mut self) -> &mut Self {
        self.cmc_equals = None;
        self
    }

    pub fn set_cmc_range(&mut self, cmc_range: (f64, f64)) -> &mut Self {
        self.cmc_range = Some(cmc_range);
        self
    }

    pub fn unset_cmc_range(&mut self) -> &mut Self {
        self.cmc_range = None;
        self
    }

    pub fn set_color_identity_equals(&mut self, color_identity_equals: Colors) -> &mut Self {
        self.color_identity_equals = Some(color_identity_equals);
        self
    }

    pub fn unset_color_identity_equals(&mut self) -> &mut Self {
        self.color_identity_equals = None;
        self
    }

    pub fn set_color_identity_within(&mut self, color_identity_contains_any: Colors) -> &mut Self {
        self.color_identity_within = Some(color_identity_contains_any);
        self
    }

    pub fn unset_color_identity_within(&mut self) -> &mut Self {
        self.color_identity_within = None;
        self
    }

    // combat
    pub fn set_power_equals(&mut self, power_equals: i32) -> &mut Self {
        self.power_equals = Some(power_equals);
        self
    }

    pub fn unset_power_equals(&mut self) -> &mut Self {
        self.power_equals = None;
        self
    }

    pub fn set_power_range(&mut self, power_range: (i32, i32)) -> &mut Self {
        self.power_range = Some(power_range);
        self
    }

    pub fn unset_power_range(&mut self) -> &mut Self {
        self.power_range = None;
        self
    }

    pub fn set_toughness_equals(&mut self, toughness_equals: i32) -> &mut Self {
        self.toughness_equals = Some(toughness_equals);
        self
    }

    pub fn unset_toughness_equals(&mut self) -> &mut Self {
        self.toughness_equals = None;
        self
    }

    pub fn set_toughness_range(&mut self, toughness_range: (i32, i32)) -> &mut Self {
        self.toughness_range = Some(toughness_range);
        self
    }

    pub fn unset_toughness_range(&mut self) -> &mut Self {
        self.toughness_range = None;
        self
    }

    // flags
    pub fn set_is_valid_commander(&mut self, is_valid_commander: bool) -> &mut Self {
        self.is_valid_commander = Some(is_valid_commander);
        self
    }

    pub fn unset_is_valid_commander(&mut self) -> &mut Self {
        self.is_valid_commander = None;
        self
    }

    pub fn set_is_token(&mut self, is_token: bool) -> &mut Self {
        self.is_token = Some(is_token);
        self
    }

    pub fn unset_is_token(&mut self) -> &mut Self {
        self.is_token = None;
        self
    }

    pub fn set_is_playable(&mut self, is_playable: bool) -> &mut Self {
        self.is_playable = Some(is_playable);
        self
    }

    pub fn unset_is_playable(&mut self) -> &mut Self {
        self.is_playable = None;
        self
    }

    pub fn set_digital(&mut self, digital: bool) -> &mut Self {
        self.digital = Some(digital);
        self
    }

    pub fn unset_digital(&mut self) -> &mut Self {
        self.digital = None;
        self
    }

    pub fn set_oversized(&mut self, oversized: bool) -> &mut Self {
        self.oversized = Some(oversized);
        self
    }

    pub fn unset_oversized(&mut self) -> &mut Self {
        self.oversized = None;
        self
    }

    pub fn set_promo(&mut self, promo: bool) -> &mut Self {
        self.promo = Some(promo);
        self
    }

    pub fn unset_promo(&mut self) -> &mut Self {
        self.promo = None;
        self
    }

    pub fn set_content_warning(&mut self, content_warning: bool) -> &mut Self {
        self.content_warning = Some(content_warning);
        self
    }

    pub fn unset_content_warning(&mut self) -> &mut Self {
        self.content_warning = None;
        self
    }

    pub fn set_language(&mut self, language: impl Into<String>) -> &mut Self {
        self.language = Some(language.into());
        self
    }

    pub fn unset_language(&mut self) -> &mut Self {
        self.language = None;
        self
    }

    // config
    pub fn set_limit(&mut self, limit: u32) -> &mut Self {
        self.limit = limit;
        self
    }

    pub fn set_offset(&mut self, offset: u32) -> &mut Self {
        self.offset = offset;
        self
    }

    pub fn set_order_by(&mut self, order_by: OrderByOptions) -> &mut Self {
        self.order_by = Some(order_by);
        self
    }

    pub fn unset_order_by(&mut self) -> &mut Self {
        self.order_by = None;
        self
    }

    pub fn set_ascending(&mut self, ascending: bool) -> &mut Self {
        self.ascending = ascending;
        self
    }

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

    // clear all filters
    pub fn clear(&mut self) -> &mut Self {
        *self = Self::default();
        self
    }
}
