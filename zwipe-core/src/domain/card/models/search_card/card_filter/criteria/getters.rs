//! Getter methods for accessing criteria values.

use crate::domain::card::scryfall_data::colors::Colors;
use crate::domain::card::scryfall_data::rarity::Rarities;
use crate::domain::card::search_card::card_filter::criteria::CardCriteria;
use crate::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use crate::domain::card::search_card::card_type::CardType;
use crate::domain::deck::Format;

#[expect(
    missing_docs,
    reason = "each getter returns the criteria field of the same name"
)]
impl CardCriteria {
    // text
    #[must_use]
    pub fn name_contains(&self) -> Option<&str> {
        self.name_contains.as_deref()
    }

    #[must_use]
    pub fn oracle_text_contains(&self) -> Option<&str> {
        self.oracle_text_contains.as_deref()
    }

    #[must_use]
    pub fn oracle_text_contains_any(&self) -> Option<&[String]> {
        self.oracle_text_contains_any.as_deref()
    }

    #[must_use]
    pub fn oracle_text_contains_all(&self) -> Option<&[String]> {
        self.oracle_text_contains_all.as_deref()
    }

    #[must_use]
    pub fn flavor_text_contains(&self) -> Option<&str> {
        self.flavor_text_contains.as_deref()
    }

    // keywords
    #[must_use]
    pub fn keywords_contains_any(&self) -> Option<&[String]> {
        self.keywords_contains_any.as_deref()
    }

    #[must_use]
    pub fn keywords_contains_all(&self) -> Option<&[String]> {
        self.keywords_contains_all.as_deref()
    }

    #[must_use]
    pub fn has_flavor_text(&self) -> Option<bool> {
        self.has_flavor_text
    }

    // types
    #[must_use]
    pub fn type_line_contains(&self) -> Option<&str> {
        self.type_line_contains.as_deref()
    }

    #[must_use]
    pub fn type_line_contains_any(&self) -> Option<&[String]> {
        self.type_line_contains_any.as_deref()
    }

    #[must_use]
    pub fn card_type_contains_any(&self) -> Option<&[CardType]> {
        self.card_type_contains_any.as_deref()
    }

    #[must_use]
    pub fn type_line_contains_all(&self) -> Option<&[String]> {
        self.type_line_contains_all.as_deref()
    }

    #[must_use]
    pub fn card_type_contains_all(&self) -> Option<&[CardType]> {
        self.card_type_contains_all.as_deref()
    }

    // rarity
    #[must_use]
    pub fn rarity_equals_any(&self) -> Option<&Rarities> {
        self.rarity_equals_any.as_ref()
    }

    // set
    #[must_use]
    pub fn set_equals_any(&self) -> Option<&[String]> {
        self.set_equals_any.as_deref()
    }

    // artist
    #[must_use]
    pub fn artist_equals_any(&self) -> Option<&[String]> {
        self.artist_equals_any.as_deref()
    }

    // mana
    #[must_use]
    pub fn cmc_equals(&self) -> Option<f64> {
        self.cmc_equals
    }

    #[must_use]
    pub fn cmc_range(&self) -> Option<(f64, f64)> {
        self.cmc_range
    }

    // price
    #[must_use]
    pub fn price_min(&self) -> Option<f64> {
        self.price_min
    }

    #[must_use]
    pub fn price_max(&self) -> Option<f64> {
        self.price_max
    }

    #[must_use]
    pub fn price_currency(&self) -> Option<PriceCurrency> {
        self.price_currency
    }

    #[must_use]
    pub fn color_identity_equals(&self) -> Option<&Colors> {
        self.color_identity_equals.as_ref()
    }

    #[must_use]
    pub fn color_identity_within(&self) -> Option<&Colors> {
        self.color_identity_within.as_ref()
    }

    // produced mana
    #[must_use]
    pub fn produced_mana_contains_any(&self) -> Option<&[String]> {
        self.produced_mana_contains_any.as_deref()
    }

    #[must_use]
    pub fn produced_mana_contains_all(&self) -> Option<&[String]> {
        self.produced_mana_contains_all.as_deref()
    }

    // combat
    #[must_use]
    pub fn power_equals(&self) -> Option<i32> {
        self.power_equals
    }

    #[must_use]
    pub fn power_range(&self) -> Option<(i32, i32)> {
        self.power_range
    }

    #[must_use]
    pub fn toughness_equals(&self) -> Option<i32> {
        self.toughness_equals
    }

    #[must_use]
    pub fn toughness_range(&self) -> Option<(i32, i32)> {
        self.toughness_range
    }

    // flags
    #[must_use]
    pub fn is_token(&self) -> Option<bool> {
        self.is_token
    }

    #[must_use]
    pub fn is_playable(&self) -> Option<bool> {
        self.is_playable
    }

    #[must_use]
    pub fn digital(&self) -> Option<bool> {
        self.digital
    }

    #[must_use]
    pub fn oversized(&self) -> Option<bool> {
        self.oversized
    }

    #[must_use]
    pub fn promo(&self) -> Option<bool> {
        self.promo
    }

    #[must_use]
    pub fn content_warning(&self) -> Option<bool> {
        self.content_warning
    }

    #[must_use]
    pub fn language(&self) -> Option<&str> {
        self.language.as_deref()
    }

    // legalities
    #[must_use]
    pub fn legalities_contains_any(&self) -> Option<&[String]> {
        self.legalities_contains_any.as_deref()
    }

    #[must_use]
    pub fn legalities_contains_all(&self) -> Option<&[String]> {
        self.legalities_contains_all.as_deref()
    }

    // commander
    #[must_use]
    pub fn is_commander_in_format(&self) -> Option<&Format> {
        self.is_commander_in_format.as_ref()
    }

    // partner/background/spell
    #[must_use]
    pub fn is_partner(&self) -> Option<bool> {
        self.is_partner
    }

    #[must_use]
    pub fn is_background(&self) -> Option<bool> {
        self.is_background
    }

    #[must_use]
    pub fn is_signature_spell(&self) -> Option<bool> {
        self.is_signature_spell
    }

    // mechanical category
    #[must_use]
    pub fn card_roles_contains_any(&self) -> Option<&[String]> {
        self.card_roles_contains_any.as_deref()
    }

    #[must_use]
    pub fn card_roles_contains_all(&self) -> Option<&[String]> {
        self.card_roles_contains_all.as_deref()
    }

    #[must_use]
    pub fn card_roles_excludes(&self) -> Option<&[String]> {
        self.card_roles_excludes.as_deref()
    }

    // oracle tags
    #[must_use]
    pub fn oracle_tags_contains_any(&self) -> Option<&[String]> {
        self.oracle_tags_contains_any.as_deref()
    }

    #[must_use]
    pub fn oracle_tags_contains_all(&self) -> Option<&[String]> {
        self.oracle_tags_contains_all.as_deref()
    }

    #[must_use]
    pub fn oracle_tags_excludes(&self) -> Option<&[String]> {
        self.oracle_tags_excludes.as_deref()
    }

    // excludes
    #[must_use]
    pub fn name_not_contains(&self) -> Option<&str> {
        self.name_not_contains.as_deref()
    }

    #[must_use]
    pub fn oracle_text_not_contains(&self) -> Option<&str> {
        self.oracle_text_not_contains.as_deref()
    }

    #[must_use]
    pub fn oracle_text_excludes_any(&self) -> Option<&[String]> {
        self.oracle_text_excludes_any.as_deref()
    }

    #[must_use]
    pub fn keywords_excludes(&self) -> Option<&[String]> {
        self.keywords_excludes.as_deref()
    }

    #[must_use]
    pub fn flavor_text_not_contains(&self) -> Option<&str> {
        self.flavor_text_not_contains.as_deref()
    }

    #[must_use]
    pub fn type_line_not_contains(&self) -> Option<&str> {
        self.type_line_not_contains.as_deref()
    }

    #[must_use]
    pub fn type_line_excludes_any(&self) -> Option<&[String]> {
        self.type_line_excludes_any.as_deref()
    }

    #[must_use]
    pub fn card_type_excludes_any(&self) -> Option<&[CardType]> {
        self.card_type_excludes_any.as_deref()
    }

    #[must_use]
    pub fn produced_mana_excludes(&self) -> Option<&[String]> {
        self.produced_mana_excludes.as_deref()
    }

    #[must_use]
    pub fn rarity_excludes_any(&self) -> Option<&Rarities> {
        self.rarity_excludes_any.as_ref()
    }

    #[must_use]
    pub fn set_excludes_any(&self) -> Option<&[String]> {
        self.set_excludes_any.as_deref()
    }

    #[must_use]
    pub fn artist_excludes_any(&self) -> Option<&[String]> {
        self.artist_excludes_any.as_deref()
    }
}
