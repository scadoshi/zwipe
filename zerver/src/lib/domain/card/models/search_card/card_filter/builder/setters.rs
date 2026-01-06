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

    // clear all filters
    pub fn clear_all(&mut self) -> &mut Self {
        self.name_contains = None;
        self.oracle_text_contains = None;
        self.flavor_text_contains = None;
        self.has_flavor_text = None;
        self.type_line_contains = None;
        self.type_line_contains_any = None;
        self.card_type_contains_any = None;
        self.set_equals_any = None;
        self.rarity_equals_any = None;
        self.cmc_equals = None;
        self.cmc_range = None;
        self.color_identity_equals = None;
        self.color_identity_within = None;
        self.power_equals = None;
        self.power_range = None;
        self.toughness_equals = None;
        self.toughness_range = None;
        self.is_valid_commander = None;
        self.is_token = None;
        self.order_by = None;
        self.ascending = true;
        self
    }
}
