//! Card filter builder for constructing complex MTG card search queries.
//!
//! Provides fluent API for building card filters with validation.

/// Getter methods for accessing filter values.
pub mod getters;
/// Setter methods for modifying filter values.
pub mod setters;

use crate::domain::card::models::{
    scryfall_data::{
        colors::{Color, Colors},
        rarity::Rarities,
    },
    search_card::{
        card_filter::{error::InvalidCardFilter, CardFilter, OrderByOptions},
        card_type::CardType,
    },
};
use serde::{Deserialize, Serialize};

/// Builder for constructing card search filters with fluent API.
///
/// # Filter Categories
///
/// - **Combat**: Power/toughness (exact or range)
/// - **Mana**: CMC, color identity (within/equals)
/// - **Text**: Name, oracle text, flavor text, type line
/// - **Metadata**: Rarity, set, artist, language
/// - **Flags**: Commander-legal, token, digital, promo, etc.
/// - **Pagination**: Limit, offset, ordering
///
/// # Usage Patterns
///
/// **Quick constructor** - Use `with_*` methods for single-filter searches:
/// ```rust,ignore
/// let filter = CardFilterBuilder::with_name_contains("Lightning Bolt").build()?;
/// ```
///
/// **Fluent builder** - Chain `set_*` methods for complex searches:
/// ```rust,ignore
/// let filter = CardFilterBuilder::new()
///     .set_name_contains("Dragon")
///     .set_color_identity_within([Color::Red])
///     .set_cmc_range((4.0, 7.0))
///     .set_rarity_equals_any(Rarities::from([Rarity::Rare, Rarity::Mythic]))
///     .set_limit(50)
///     .build()?;
/// ```
///
/// # Defaults
///
/// - `is_playable`: `true` (exclude un-cards, silver-bordered)
/// - `digital`: `false` (exclude Arena-only cards)
/// - `oversized`: `false`
/// - `promo`: `false`
/// - `content_warning`: `false`
/// - `language`: `"en"`
/// - `limit`: 100
/// - `offset`: 0
/// - `ascending`: `true`
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct CardFilterBuilder {
    // Combat stats
    // combat
    power_equals: Option<i32>,
    power_range: Option<(i32, i32)>,
    toughness_equals: Option<i32>,
    toughness_range: Option<(i32, i32)>,
    // mana
    cmc_equals: Option<f64>,
    cmc_range: Option<(f64, f64)>,
    color_identity_within: Option<Colors>,
    color_identity_equals: Option<Colors>,
    // rarity
    rarity_equals_any: Option<Rarities>,
    // set
    set_equals_any: Option<Vec<String>>,
    // artist
    artist_equals_any: Option<Vec<String>>,
    // text
    name_contains: Option<String>,
    oracle_text_contains: Option<String>,
    flavor_text_contains: Option<String>,
    has_flavor_text: Option<bool>,
    // types
    type_line_contains: Option<String>,
    type_line_contains_any: Option<Vec<String>>,
    card_type_contains_any: Option<Vec<CardType>>,
    // flags
    is_valid_commander: Option<bool>,
    is_token: Option<bool>,
    is_playable: Option<bool>,
    digital: Option<bool>,
    oversized: Option<bool>,
    promo: Option<bool>,
    content_warning: Option<bool>,
    language: Option<String>,
    // config
    limit: u32,
    offset: u32,
    order_by: Option<OrderByOptions>,
    ascending: bool,
}

impl Default for CardFilterBuilder {
    fn default() -> Self {
        Self {
            power_equals: None,
            power_range: None,
            toughness_equals: None,
            toughness_range: None,
            cmc_equals: None,
            cmc_range: None,
            color_identity_within: None,
            color_identity_equals: None,
            rarity_equals_any: None,
            set_equals_any: None,
            artist_equals_any: None,
            name_contains: None,
            oracle_text_contains: None,
            flavor_text_contains: None,
            has_flavor_text: None,
            type_line_contains: None,
            type_line_contains_any: None,
            card_type_contains_any: None,
            is_valid_commander: None,
            is_token: None,
            is_playable: Some(true),
            digital: Some(false),
            oversized: Some(false),
            promo: Some(false),
            content_warning: Some(false),
            language: Some("en".to_string()),
            limit: 100,
            offset: 0,
            order_by: None,
            ascending: true,
        }
    }
}

impl CardFilterBuilder {
    /// Creates a new filter builder with default values.
    ///
    /// Defaults: playable cards only, English, non-digital, non-promo, 100 result limit.
    pub fn new() -> Self {
        Self::default()
    }

    /// Checks if filter has any search criteria (ignoring config like limit/offset).
    ///
    /// Returns `true` if only config fields are set (would match all cards).
    pub fn is_empty(&self) -> bool {
        let mut default = self.clone();
        default.retain_config();
        *self == default
    }

    // =================================
    // Quick Constructors (with_*)
    // =================================
    // These create a new builder with default values plus one filter.
    // Useful for single-filter searches.

    /// Creates builder with name filter (case-insensitive substring match).
    pub fn with_name_contains(name_contains: impl Into<String>) -> CardFilterBuilder {
        CardFilterBuilder {
            name_contains: Some(name_contains.into()),
            ..CardFilterBuilder::default()
        }
    }

    /// Creates builder with oracle text filter (ability text substring match).
    pub fn with_oracle_text_contains(oracle_text_contains: impl Into<String>) -> CardFilterBuilder {
        CardFilterBuilder {
            oracle_text_contains: Some(oracle_text_contains.into()),
            ..CardFilterBuilder::default()
        }
    }

    /// Creates builder with flavor text filter.
    pub fn with_flavor_text_contains(flavor_text_contains: impl Into<String>) -> Self {
        Self {
            flavor_text_contains: Some(flavor_text_contains.into()),
            ..Self::default()
        }
    }

    /// Creates builder filtering by presence/absence of flavor text.
    pub fn with_has_flavor_text(has_flavor_text: bool) -> Self {
        Self {
            has_flavor_text: Some(has_flavor_text),
            ..Self::default()
        }
    }

    /// Creates builder with type line filter (e.g., "Legendary Creature — Dragon").
    pub fn with_type_line_contains(type_line_contains: impl Into<String>) -> CardFilterBuilder {
        CardFilterBuilder {
            type_line_contains: Some(type_line_contains.into()),
            ..CardFilterBuilder::default()
        }
    }

    /// Creates builder matching any of the provided type line substrings.
    pub fn with_type_line_contains_any<I, S>(type_line_contains_any: I) -> CardFilterBuilder
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        CardFilterBuilder {
            type_line_contains_any: Some(
                type_line_contains_any.into_iter().map(Into::into).collect(),
            ),
            ..CardFilterBuilder::default()
        }
    }

    /// Creates builder matching any of the provided card types (Creature, Instant, etc.).
    pub fn with_card_type_contains_any<I>(card_type_contains_any: I) -> CardFilterBuilder
    where
        I: IntoIterator<Item = CardType>,
    {
        CardFilterBuilder {
            card_type_contains_any: Some(card_type_contains_any.into_iter().collect()),
            ..CardFilterBuilder::default()
        }
    }

    /// Creates builder matching any of the provided set codes (e.g., "MH2", "ONE").
    pub fn with_set_contains(
        set_equals_any: impl IntoIterator<Item = impl Into<String>>,
    ) -> CardFilterBuilder {
        CardFilterBuilder {
            set_equals_any: Some(set_equals_any.into_iter().map(|x| x.into()).collect()),
            ..CardFilterBuilder::default()
        }
    }

    /// Creates builder matching any of the provided rarities.
    pub fn with_rarity_equals_any(rarity_equals_any: Rarities) -> CardFilterBuilder {
        let rarity_equals_any = if rarity_equals_any.is_empty() {
            None
        } else {
            Some(rarity_equals_any)
        };
        CardFilterBuilder {
            rarity_equals_any,
            ..CardFilterBuilder::default()
        }
    }

    /// Creates builder with exact CMC (converted mana cost) filter.
    pub fn with_cmc_equals(cmc_equals: f64) -> CardFilterBuilder {
        CardFilterBuilder {
            cmc_equals: Some(cmc_equals),
            ..CardFilterBuilder::default()
        }
    }

    /// Creates builder with CMC range filter (inclusive).
    pub fn with_cmc_range(cmc_range: (f64, f64)) -> CardFilterBuilder {
        CardFilterBuilder {
            cmc_range: Some(cmc_range),
            ..CardFilterBuilder::default()
        }
    }

    /// Creates builder matching exact color identity (e.g., exactly W+U, not mono-W).
    pub fn with_color_identity_equals<I>(color_identity_equals: I) -> CardFilterBuilder
    where
        I: IntoIterator<Item = Color>,
    {
        CardFilterBuilder {
            color_identity_equals: Some(color_identity_equals.into_iter().collect()),
            ..CardFilterBuilder::default()
        }
    }

    /// Creates builder matching cards within color identity (subset of provided colors).
    ///
    /// Example: `within([R, G])` matches mono-R, mono-G, R+G, and colorless.
    pub fn with_color_identity_within<I>(color_identity_within: I) -> CardFilterBuilder
    where
        I: IntoIterator<Item = Color>,
    {
        CardFilterBuilder {
            color_identity_within: Some(color_identity_within.into_iter().collect()),
            ..CardFilterBuilder::default()
        }
    }

    /// Creates builder with exact power filter (creature combat stat).
    pub fn with_power_equals(power_equals: i32) -> CardFilterBuilder {
        CardFilterBuilder {
            power_equals: Some(power_equals),
            ..CardFilterBuilder::default()
        }
    }

    /// Creates builder with power range filter (inclusive).
    pub fn with_power_range(power_range: (i32, i32)) -> CardFilterBuilder {
        CardFilterBuilder {
            power_range: Some(power_range),
            ..CardFilterBuilder::default()
        }
    }

    /// Creates builder with exact toughness filter.
    pub fn with_toughness_equals(toughness_equals: i32) -> CardFilterBuilder {
        CardFilterBuilder {
            toughness_equals: Some(toughness_equals),
            ..CardFilterBuilder::default()
        }
    }

    /// Creates builder with toughness range filter (inclusive).
    pub fn with_toughness_range(toughness_range: (i32, i32)) -> CardFilterBuilder {
        CardFilterBuilder {
            toughness_range: Some(toughness_range),
            ..CardFilterBuilder::default()
        }
    }

    /// Creates builder filtering by Commander format legality.
    pub fn with_is_valid_commander(is_valid_commander: bool) -> CardFilterBuilder {
        CardFilterBuilder {
            is_valid_commander: Some(is_valid_commander),
            ..CardFilterBuilder::default()
        }
    }

    /// Creates builder filtering by token status (tokens vs. real cards).
    pub fn with_is_token(is_token: bool) -> CardFilterBuilder {
        CardFilterBuilder {
            is_token: Some(is_token),
            ..CardFilterBuilder::default()
        }
    }

    /// Creates builder filtering by playability (excludes un-cards, silver-bordered).
    pub fn with_is_playable(is_playable: bool) -> CardFilterBuilder {
        CardFilterBuilder {
            is_playable: Some(is_playable),
            ..CardFilterBuilder::default()
        }
    }

    /// Creates builder filtering by digital-only status (Arena/MTGO exclusives).
    pub fn with_digital(digital: bool) -> CardFilterBuilder {
        CardFilterBuilder {
            digital: Some(digital),
            ..CardFilterBuilder::default()
        }
    }

    /// Creates builder filtering by oversized card status.
    pub fn with_oversized(oversized: bool) -> CardFilterBuilder {
        CardFilterBuilder {
            oversized: Some(oversized),
            ..CardFilterBuilder::default()
        }
    }

    /// Creates builder filtering by promotional card status.
    pub fn with_promo(promo: bool) -> CardFilterBuilder {
        CardFilterBuilder {
            promo: Some(promo),
            ..CardFilterBuilder::default()
        }
    }

    /// Creates builder filtering by content warning flag.
    pub fn with_content_warning(content_warning: bool) -> CardFilterBuilder {
        CardFilterBuilder {
            content_warning: Some(content_warning),
            ..CardFilterBuilder::default()
        }
    }

    /// Creates builder with specific result ordering (name, CMC, rarity, etc.).
    pub fn with_order_by(order_by: OrderByOptions) -> CardFilterBuilder {
        CardFilterBuilder {
            order_by: Some(order_by),
            ..CardFilterBuilder::default()
        }
    }

    /// Builds the final `CardFilter` with validation.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidCardFilter::Empty`] if no search criteria are set
    /// (only config fields like limit/offset are present).
    pub fn build(&self) -> Result<CardFilter, InvalidCardFilter> {
        if self.is_empty() {
            return Err(InvalidCardFilter::Empty);
        }

        Ok(CardFilter {
            power_equals: self.power_equals,
            power_range: self.power_range,
            toughness_equals: self.toughness_equals,
            toughness_range: self.toughness_range,
            cmc_equals: self.cmc_equals,
            cmc_range: self.cmc_range,
            color_identity_within: self.color_identity_within.clone(),
            color_identity_equals: self.color_identity_equals.clone(),
            rarity_equals_any: self.rarity_equals_any.clone(),
            set_equals_any: self.set_equals_any.clone(),
            artist_equals_any: self.artist_equals_any.clone(),
            name_contains: self.name_contains.clone(),
            oracle_text_contains: self.oracle_text_contains.clone(),
            flavor_text_contains: self.flavor_text_contains.clone(),
            has_flavor_text: self.has_flavor_text,
            type_line_contains: self.type_line_contains.clone(),
            type_line_contains_any: self.type_line_contains_any.clone(),
            card_type_contains_any: self.card_type_contains_any.clone(),
            is_valid_commander: self.is_valid_commander,
            is_token: self.is_token,
            is_playable: self.is_playable,
            digital: self.digital,
            oversized: self.oversized,
            promo: self.promo,
            content_warning: self.content_warning,
            language: self.language.clone(),
            limit: self.limit,
            offset: self.offset,
            order_by: self.order_by,
            ascending: self.ascending,
        })
    }
}
