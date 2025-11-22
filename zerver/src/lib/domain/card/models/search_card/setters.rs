use super::{CardFilterWithState, CardType, Colors, Full};
use std::marker::PhantomData;

impl<P> CardFilterWithState<P> {
    // text
    pub fn set_name_contains(self, name_contains: impl Into<String>) -> CardFilterWithState<Full> {
        CardFilterWithState {
            power_equals: self.power_equals,
            power_range: self.power_range,
            toughness_equals: self.toughness_equals,
            toughness_range: self.toughness_range,
            cmc_equals: self.cmc_equals,
            cmc_range: self.cmc_range,
            color_identity_contains_any: self.color_identity_contains_any,
            color_identity_equals: self.color_identity_equals,
            rarity_contains: self.rarity_contains,
            set_contains: self.set_contains,
            name_contains: Some(name_contains.into()),
            oracle_text_contains: self.oracle_text_contains,
            type_line_contains: self.type_line_contains,
            type_line_contains_any: self.type_line_contains_any,
            card_type_contains_any: self.card_type_contains_any,
            limit: self.limit,
            offset: self.offset,
            state: PhantomData,
        }
    }

    pub fn set_oracle_text_contains(
        self,
        oracle_text_contains: impl Into<String>,
    ) -> CardFilterWithState<Full> {
        CardFilterWithState {
            power_equals: self.power_equals,
            power_range: self.power_range,
            toughness_equals: self.toughness_equals,
            toughness_range: self.toughness_range,
            cmc_equals: self.cmc_equals,
            cmc_range: self.cmc_range,
            color_identity_contains_any: self.color_identity_contains_any,
            color_identity_equals: self.color_identity_equals,
            rarity_contains: self.rarity_contains,
            set_contains: self.set_contains,
            name_contains: self.name_contains,
            oracle_text_contains: Some(oracle_text_contains.into()),
            type_line_contains: self.type_line_contains,
            type_line_contains_any: self.type_line_contains_any,
            card_type_contains_any: self.card_type_contains_any,
            limit: self.limit,
            offset: self.offset,
            state: PhantomData,
        }
    }

    // types
    pub fn set_type_line_contains(
        self,
        type_line_contains: impl Into<String>,
    ) -> CardFilterWithState<Full> {
        CardFilterWithState {
            power_equals: self.power_equals,
            power_range: self.power_range,
            toughness_equals: self.toughness_equals,
            toughness_range: self.toughness_range,
            cmc_equals: self.cmc_equals,
            cmc_range: self.cmc_range,
            color_identity_contains_any: self.color_identity_contains_any,
            color_identity_equals: self.color_identity_equals,
            rarity_contains: self.rarity_contains,
            set_contains: self.set_contains,
            name_contains: self.name_contains,
            oracle_text_contains: self.oracle_text_contains,
            type_line_contains: Some(type_line_contains.into()),
            type_line_contains_any: self.type_line_contains_any,
            card_type_contains_any: self.card_type_contains_any,
            limit: self.limit,
            offset: self.offset,
            state: PhantomData,
        }
    }

    pub fn set_type_line_contains_any<I, S>(
        self,
        type_line_contains_any: I,
    ) -> CardFilterWithState<Full>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        CardFilterWithState {
            power_equals: self.power_equals,
            power_range: self.power_range,
            toughness_equals: self.toughness_equals,
            toughness_range: self.toughness_range,
            cmc_equals: self.cmc_equals,
            cmc_range: self.cmc_range,
            color_identity_contains_any: self.color_identity_contains_any,
            color_identity_equals: self.color_identity_equals,
            rarity_contains: self.rarity_contains,
            set_contains: self.set_contains,
            name_contains: self.name_contains,
            oracle_text_contains: self.oracle_text_contains,
            type_line_contains: self.type_line_contains,
            type_line_contains_any: Some(
                type_line_contains_any.into_iter().map(Into::into).collect(),
            ),
            card_type_contains_any: self.card_type_contains_any,
            limit: self.limit,
            offset: self.offset,
            state: PhantomData,
        }
    }

    pub fn set_card_type_contains_any<I>(
        self,
        card_type_contains_any: I,
    ) -> CardFilterWithState<Full>
    where
        I: IntoIterator<Item = CardType>,
    {
        CardFilterWithState {
            power_equals: self.power_equals,
            power_range: self.power_range,
            toughness_equals: self.toughness_equals,
            toughness_range: self.toughness_range,
            cmc_equals: self.cmc_equals,
            cmc_range: self.cmc_range,
            color_identity_contains_any: self.color_identity_contains_any,
            color_identity_equals: self.color_identity_equals,
            rarity_contains: self.rarity_contains,
            set_contains: self.set_contains,
            name_contains: self.name_contains,
            oracle_text_contains: self.oracle_text_contains,
            type_line_contains: self.type_line_contains,
            type_line_contains_any: self.type_line_contains_any,
            card_type_contains_any: Some(card_type_contains_any.into_iter().collect()),
            limit: self.limit,
            offset: self.offset,
            state: PhantomData,
        }
    }

    // printing
    pub fn set_set_contains(self, set_contains: impl Into<String>) -> CardFilterWithState<Full> {
        CardFilterWithState {
            power_equals: self.power_equals,
            power_range: self.power_range,
            toughness_equals: self.toughness_equals,
            toughness_range: self.toughness_range,
            cmc_equals: self.cmc_equals,
            cmc_range: self.cmc_range,
            color_identity_contains_any: self.color_identity_contains_any,
            color_identity_equals: self.color_identity_equals,
            rarity_contains: self.rarity_contains,
            set_contains: Some(set_contains.into()),
            name_contains: self.name_contains,
            oracle_text_contains: self.oracle_text_contains,
            type_line_contains: self.type_line_contains,
            type_line_contains_any: self.type_line_contains_any,
            card_type_contains_any: self.card_type_contains_any,
            limit: self.limit,
            offset: self.offset,
            state: PhantomData,
        }
    }

    pub fn set_rarity_contains(
        self,
        rarity_contains: impl Into<String>,
    ) -> CardFilterWithState<Full> {
        CardFilterWithState {
            power_equals: self.power_equals,
            power_range: self.power_range,
            toughness_equals: self.toughness_equals,
            toughness_range: self.toughness_range,
            cmc_equals: self.cmc_equals,
            cmc_range: self.cmc_range,
            color_identity_contains_any: self.color_identity_contains_any,
            color_identity_equals: self.color_identity_equals,
            rarity_contains: Some(rarity_contains.into()),
            set_contains: self.set_contains,
            name_contains: self.name_contains,
            oracle_text_contains: self.oracle_text_contains,
            type_line_contains: self.type_line_contains,
            type_line_contains_any: self.type_line_contains_any,
            card_type_contains_any: self.card_type_contains_any,
            limit: self.limit,
            offset: self.offset,
            state: PhantomData,
        }
    }

    // mana
    pub fn set_cmc_equals(self, cmc_equals: f64) -> CardFilterWithState<Full> {
        CardFilterWithState {
            power_equals: self.power_equals,
            power_range: self.power_range,
            toughness_equals: self.toughness_equals,
            toughness_range: self.toughness_range,
            cmc_equals: Some(cmc_equals),
            cmc_range: self.cmc_range,
            color_identity_contains_any: self.color_identity_contains_any,
            color_identity_equals: self.color_identity_equals,
            rarity_contains: self.rarity_contains,
            set_contains: self.set_contains,
            name_contains: self.name_contains,
            oracle_text_contains: self.oracle_text_contains,
            type_line_contains: self.type_line_contains,
            type_line_contains_any: self.type_line_contains_any,
            card_type_contains_any: self.card_type_contains_any,
            limit: self.limit,
            offset: self.offset,
            state: PhantomData,
        }
    }

    pub fn set_cmc_range(self, cmc_range: (f64, f64)) -> CardFilterWithState<Full> {
        CardFilterWithState {
            power_equals: self.power_equals,
            power_range: self.power_range,
            toughness_equals: self.toughness_equals,
            toughness_range: self.toughness_range,
            cmc_equals: self.cmc_equals,
            cmc_range: Some(cmc_range),
            color_identity_contains_any: self.color_identity_contains_any,
            color_identity_equals: self.color_identity_equals,
            rarity_contains: self.rarity_contains,
            set_contains: self.set_contains,
            name_contains: self.name_contains,
            oracle_text_contains: self.oracle_text_contains,
            type_line_contains: self.type_line_contains,
            type_line_contains_any: self.type_line_contains_any,
            card_type_contains_any: self.card_type_contains_any,
            limit: self.limit,
            offset: self.offset,
            state: PhantomData,
        }
    }

    pub fn set_color_identity_equals(
        self,
        color_identity_equals: Colors,
    ) -> CardFilterWithState<Full> {
        CardFilterWithState {
            power_equals: self.power_equals,
            power_range: self.power_range,
            toughness_equals: self.toughness_equals,
            toughness_range: self.toughness_range,
            cmc_equals: self.cmc_equals,
            cmc_range: self.cmc_range,
            color_identity_contains_any: self.color_identity_contains_any,
            color_identity_equals: Some(color_identity_equals),
            rarity_contains: self.rarity_contains,
            set_contains: self.set_contains,
            name_contains: self.name_contains,
            oracle_text_contains: self.oracle_text_contains,
            type_line_contains: self.type_line_contains,
            type_line_contains_any: self.type_line_contains_any,
            card_type_contains_any: self.card_type_contains_any,
            limit: self.limit,
            offset: self.offset,
            state: PhantomData,
        }
    }

    pub fn set_color_identity_contains_any(
        self,
        color_identity_contains_any: Colors,
    ) -> CardFilterWithState<Full> {
        CardFilterWithState {
            power_equals: self.power_equals,
            power_range: self.power_range,
            toughness_equals: self.toughness_equals,
            toughness_range: self.toughness_range,
            cmc_equals: self.cmc_equals,
            cmc_range: self.cmc_range,
            color_identity_contains_any: Some(color_identity_contains_any),
            color_identity_equals: self.color_identity_equals,
            rarity_contains: self.rarity_contains,
            set_contains: self.set_contains,
            name_contains: self.name_contains,
            oracle_text_contains: self.oracle_text_contains,
            type_line_contains: self.type_line_contains,
            type_line_contains_any: self.type_line_contains_any,
            card_type_contains_any: self.card_type_contains_any,
            limit: self.limit,
            offset: self.offset,
            state: PhantomData,
        }
    }

    // combat
    pub fn set_power_equals(self, power_equals: i32) -> CardFilterWithState<Full> {
        CardFilterWithState {
            power_equals: Some(power_equals),
            power_range: self.power_range,
            toughness_equals: self.toughness_equals,
            toughness_range: self.toughness_range,
            cmc_equals: self.cmc_equals,
            cmc_range: self.cmc_range,
            color_identity_contains_any: self.color_identity_contains_any,
            color_identity_equals: self.color_identity_equals,
            rarity_contains: self.rarity_contains,
            set_contains: self.set_contains,
            name_contains: self.name_contains,
            oracle_text_contains: self.oracle_text_contains,
            type_line_contains: self.type_line_contains,
            type_line_contains_any: self.type_line_contains_any,
            card_type_contains_any: self.card_type_contains_any,
            limit: self.limit,
            offset: self.offset,
            state: PhantomData,
        }
    }

    pub fn set_power_range(self, power_range: (i32, i32)) -> CardFilterWithState<Full> {
        CardFilterWithState {
            power_equals: self.power_equals,
            power_range: Some(power_range),
            toughness_equals: self.toughness_equals,
            toughness_range: self.toughness_range,
            cmc_equals: self.cmc_equals,
            cmc_range: self.cmc_range,
            color_identity_contains_any: self.color_identity_contains_any,
            color_identity_equals: self.color_identity_equals,
            rarity_contains: self.rarity_contains,
            set_contains: self.set_contains,
            name_contains: self.name_contains,
            oracle_text_contains: self.oracle_text_contains,
            type_line_contains: self.type_line_contains,
            type_line_contains_any: self.type_line_contains_any,
            card_type_contains_any: self.card_type_contains_any,
            limit: self.limit,
            offset: self.offset,
            state: PhantomData,
        }
    }

    pub fn set_toughness_equals(self, toughness_equals: i32) -> CardFilterWithState<Full> {
        CardFilterWithState {
            power_equals: self.power_equals,
            power_range: self.power_range,
            toughness_equals: Some(toughness_equals),
            toughness_range: self.toughness_range,
            cmc_equals: self.cmc_equals,
            cmc_range: self.cmc_range,
            color_identity_contains_any: self.color_identity_contains_any,
            color_identity_equals: self.color_identity_equals,
            rarity_contains: self.rarity_contains,
            set_contains: self.set_contains,
            name_contains: self.name_contains,
            oracle_text_contains: self.oracle_text_contains,
            type_line_contains: self.type_line_contains,
            type_line_contains_any: self.type_line_contains_any,
            card_type_contains_any: self.card_type_contains_any,
            limit: self.limit,
            offset: self.offset,
            state: PhantomData,
        }
    }

    pub fn set_toughness_range(self, toughness_range: (i32, i32)) -> CardFilterWithState<Full> {
        CardFilterWithState {
            power_equals: self.power_equals,
            power_range: self.power_range,
            toughness_equals: self.toughness_equals,
            toughness_range: Some(toughness_range),
            cmc_equals: self.cmc_equals,
            cmc_range: self.cmc_range,
            color_identity_contains_any: self.color_identity_contains_any,
            color_identity_equals: self.color_identity_equals,
            rarity_contains: self.rarity_contains,
            set_contains: self.set_contains,
            name_contains: self.name_contains,
            oracle_text_contains: self.oracle_text_contains,
            type_line_contains: self.type_line_contains,
            type_line_contains_any: self.type_line_contains_any,
            card_type_contains_any: self.card_type_contains_any,
            limit: self.limit,
            offset: self.offset,
            state: PhantomData,
        }
    }

    // config
    pub fn set_limit(self, limit: u32) -> CardFilterWithState<Full> {
        CardFilterWithState {
            power_equals: self.power_equals,
            power_range: self.power_range,
            toughness_equals: self.toughness_equals,
            toughness_range: self.toughness_range,
            cmc_equals: self.cmc_equals,
            cmc_range: self.cmc_range,
            color_identity_contains_any: self.color_identity_contains_any,
            color_identity_equals: self.color_identity_equals,
            rarity_contains: self.rarity_contains,
            set_contains: self.set_contains,
            name_contains: self.name_contains,
            oracle_text_contains: self.oracle_text_contains,
            type_line_contains: self.type_line_contains,
            type_line_contains_any: self.type_line_contains_any,
            card_type_contains_any: self.card_type_contains_any,
            limit,
            offset: self.offset,
            state: PhantomData,
        }
    }

    pub fn set_offset(self, offset: u32) -> CardFilterWithState<Full> {
        CardFilterWithState {
            power_equals: self.power_equals,
            power_range: self.power_range,
            toughness_equals: self.toughness_equals,
            toughness_range: self.toughness_range,
            cmc_equals: self.cmc_equals,
            cmc_range: self.cmc_range,
            color_identity_contains_any: self.color_identity_contains_any,
            color_identity_equals: self.color_identity_equals,
            rarity_contains: self.rarity_contains,
            set_contains: self.set_contains,
            name_contains: self.name_contains,
            oracle_text_contains: self.oracle_text_contains,
            type_line_contains: self.type_line_contains,
            type_line_contains_any: self.type_line_contains_any,
            card_type_contains_any: self.card_type_contains_any,
            limit: self.limit,
            offset,
            state: PhantomData,
        }
    }
}
