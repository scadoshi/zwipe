//! Card filter builder for constructing complex MTG card search queries.
//!
//! Provides fluent API for building card filters with validation.

/// Getter methods for accessing filter values.
pub mod getters;
/// Setter methods for modifying filter values.
pub mod setters;

use crate::domain::{
    card::{
        scryfall_data::{
            colors::{Color, Colors},
            rarity::Rarities,
        },
        search_card::{
            card_filter::{
                error::InvalidCardFilter, price_currency::PriceCurrency, strip_punctuation,
                CardFilter, CardSortKey,
            },
            card_type::CardType,
        },
    },
    deck::Format,
};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// Errors if any value appears in both an include list and the exclude list for
/// the same attribute — a contradiction that matches zero cards. `field` names
/// the attribute for the error message; `includes` is the set of include lists
/// (e.g. `contains_any` + `contains_all`), `excludes` the exclude list.
fn check_include_exclude_clash<T: PartialEq + Debug>(
    field: &'static str,
    includes: &[Option<&[T]>],
    excludes: Option<&[T]>,
) -> Result<(), InvalidCardFilter> {
    let Some(excludes) = excludes else {
        return Ok(());
    };
    let mut clashing: Vec<&T> = Vec::new();
    for value in includes.iter().copied().flatten().flatten() {
        // Unique, order-preserving: a value may repeat across include lists.
        if excludes.contains(value) && !clashing.contains(&value) {
            clashing.push(value);
        }
    }
    if clashing.is_empty() {
        return Ok(());
    }
    let values = clashing
        .iter()
        .map(|v| format!("{v:?}"))
        .collect::<Vec<_>>()
        .join(", ");
    Err(InvalidCardFilter::Contradiction { field, values })
}

/// Errors if a substring `contains` filter and its `not_contains` counterpart
/// contradict. Since "contains C" requires the value to include C (and therefore
/// every substring of C), a `not_contains` term that is a substring of the
/// contains term matches zero cards — e.g. name contains "test" and doesn't
/// contain "test" (or "tes"). Compared punctuation/case-insensitively to mirror
/// how these fields are searched.
fn check_contains_not_contains_clash(
    field: &'static str,
    contains: Option<&str>,
    not_contains: Option<&str>,
) -> Result<(), InvalidCardFilter> {
    let (Some(contains), Some(not_contains)) = (contains, not_contains) else {
        return Ok(());
    };
    let contains = strip_punctuation(contains.trim()).to_lowercase();
    let not_contains = strip_punctuation(not_contains.trim()).to_lowercase();
    if !not_contains.is_empty() && contains.contains(&not_contains) {
        return Err(InvalidCardFilter::Contradiction {
            field,
            values: not_contains,
        });
    }
    Ok(())
}

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
/// - `promo`: unset (Scryfall flags Jumpstart, Secret Lair, and UB bonus inserts as
///   promo even though they're standard paper printings; filtering them by default
///   hides legitimate cards)
/// - `content_warning`: `false`
/// - `language`: `"en"`
/// - `limit`: 25
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
    // price (min/max against the selected currency's price)
    price_min: Option<f64>,
    price_max: Option<f64>,
    price_currency: Option<PriceCurrency>,
    // produced mana
    produced_mana_contains_any: Option<Vec<String>>,
    produced_mana_contains_all: Option<Vec<String>>,
    produced_mana_excludes: Option<Vec<String>>,
    // rarity
    rarity_equals_any: Option<Rarities>,
    rarity_excludes_any: Option<Rarities>,
    // set
    set_equals_any: Option<Vec<String>>,
    set_excludes_any: Option<Vec<String>>,
    // artist
    artist_equals_any: Option<Vec<String>>,
    artist_excludes_any: Option<Vec<String>>,
    // text
    name_contains: Option<String>,
    name_not_contains: Option<String>,
    oracle_text_contains: Option<String>,
    oracle_text_not_contains: Option<String>,
    oracle_text_contains_any: Option<Vec<String>>,
    oracle_text_contains_all: Option<Vec<String>>,
    oracle_text_excludes_any: Option<Vec<String>>,
    // keywords
    keywords_contains_any: Option<Vec<String>>,
    keywords_contains_all: Option<Vec<String>>,
    keywords_excludes: Option<Vec<String>>,
    flavor_text_contains: Option<String>,
    flavor_text_not_contains: Option<String>,
    has_flavor_text: Option<bool>,
    // types
    type_line_contains: Option<String>,
    type_line_not_contains: Option<String>,
    type_line_contains_any: Option<Vec<String>>,
    type_line_contains_all: Option<Vec<String>>,
    type_line_excludes_any: Option<Vec<String>>,
    card_type_contains_any: Option<Vec<CardType>>,
    card_type_contains_all: Option<Vec<CardType>>,
    card_type_excludes_any: Option<Vec<CardType>>,
    // flags
    is_token: Option<bool>,
    is_playable: Option<bool>,
    digital: Option<bool>,
    oversized: Option<bool>,
    promo: Option<bool>,
    content_warning: Option<bool>,
    language: Option<String>,
    // legalities
    legalities_contains_any: Option<Vec<String>>,
    // commander
    is_commander_in_format: Option<Format>,
    // partner/background/spell
    is_partner: Option<bool>,
    is_background: Option<bool>,
    is_signature_spell: Option<bool>,
    // mechanical category
    mechanical_categories_contains_any: Option<Vec<String>>,
    mechanical_categories_contains_all: Option<Vec<String>>,
    mechanical_categories_excludes: Option<Vec<String>>,
    // config
    limit: u32,
    offset: u32,
    order_by: Option<CardSortKey>,
    ascending: bool,
    synergy: bool,
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
            price_min: None,
            price_max: None,
            price_currency: None,
            produced_mana_contains_any: None,
            produced_mana_contains_all: None,
            produced_mana_excludes: None,
            rarity_equals_any: None,
            rarity_excludes_any: None,
            set_equals_any: None,
            set_excludes_any: None,
            artist_equals_any: None,
            artist_excludes_any: None,
            name_contains: None,
            name_not_contains: None,
            oracle_text_contains: None,
            oracle_text_not_contains: None,
            oracle_text_contains_any: None,
            oracle_text_contains_all: None,
            oracle_text_excludes_any: None,
            keywords_contains_any: None,
            keywords_contains_all: None,
            keywords_excludes: None,
            flavor_text_contains: None,
            flavor_text_not_contains: None,
            has_flavor_text: None,
            type_line_contains: None,
            type_line_not_contains: None,
            type_line_contains_any: None,
            type_line_contains_all: None,
            type_line_excludes_any: None,
            card_type_contains_any: None,
            card_type_contains_all: None,
            card_type_excludes_any: None,
            is_token: None,
            is_playable: Some(true),
            digital: Some(false),
            oversized: Some(false),
            promo: None,
            content_warning: Some(false),
            language: Some("en".to_string()),
            legalities_contains_any: None,
            is_commander_in_format: None,
            is_partner: None,
            is_background: None,
            is_signature_spell: None,
            mechanical_categories_contains_any: None,
            mechanical_categories_contains_all: None,
            mechanical_categories_excludes: None,
            limit: 25,
            offset: 0,
            order_by: None,
            ascending: true,
            synergy: false,
        }
    }
}

impl CardFilterBuilder {
    /// Creates a new filter builder with default values.
    ///
    /// Defaults: playable cards only, English, non-digital, non-promo, 25 result limit.
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

    /// Like `is_empty()` but ignores auto-populated deck-context filters.
    ///
    /// `legalities_contains_any` and `color_identity_within` are auto-populated
    /// from the deck's format and commander color identity on the add screen.
    /// Commander eligibility, partner, background, and signature spell filters are
    /// user-set via the filter UI and should count as non-empty.
    pub fn is_empty_ignoring_deck_context(&self) -> bool {
        let mut test = self.clone();
        test.unset_legalities_contains_any();
        test.unset_color_identity_within();
        test.is_empty()
    }

    // =================================
    // Quick Constructors (with_*)
    // =================================
    // These create a new builder with default values plus one filter.
    // Useful for single-filter searches.

    /// Creates builder with name filter (case-insensitive substring match).
    pub fn with_name_contains(name_contains: impl Into<String>) -> CardFilterBuilder {
        let s = name_contains.into();
        CardFilterBuilder {
            name_contains: if s.is_empty() { None } else { Some(s) },
            ..CardFilterBuilder::default()
        }
    }

    /// Creates builder matching any of the provided oracle text substrings (keyword abilities).
    pub fn with_oracle_text_contains_any<I, S>(oracle_text_contains_any: I) -> CardFilterBuilder
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let v: Vec<String> = oracle_text_contains_any.into_iter()
            .map(|s| s.into())
            .collect();
        CardFilterBuilder {
            oracle_text_contains_any: if v.is_empty() { None } else { Some(v) },
            ..CardFilterBuilder::default()
        }
    }

    /// Creates builder requiring all provided oracle text substrings to be present (AND logic).
    pub fn with_oracle_text_contains_all<I, S>(oracle_text_contains_all: I) -> CardFilterBuilder
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let v: Vec<String> = oracle_text_contains_all.into_iter()
            .map(|s| s.into())
            .collect();
        CardFilterBuilder {
            oracle_text_contains_all: if v.is_empty() { None } else { Some(v) },
            ..CardFilterBuilder::default()
        }
    }

    /// Creates builder matching any of the provided keywords (OR logic on keywords array).
    pub fn with_keywords_contains_any<I, S>(keywords_contains_any: I) -> CardFilterBuilder
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        CardFilterBuilder {
            keywords_contains_any: Some(
                keywords_contains_any.into_iter().map(Into::into).collect(),
            ),
            ..CardFilterBuilder::default()
        }
    }

    /// Creates builder requiring all provided keywords to be present (AND logic on keywords array).
    pub fn with_keywords_contains_all<I, S>(keywords_contains_all: I) -> CardFilterBuilder
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        CardFilterBuilder {
            keywords_contains_all: Some(
                keywords_contains_all.into_iter().map(Into::into).collect(),
            ),
            ..CardFilterBuilder::default()
        }
    }

    /// Creates builder matching cards legal in any of the provided formats (OR logic on legalities JSONB).
    pub fn with_legalities_contains_any<I, S>(legalities_contains_any: I) -> CardFilterBuilder
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        CardFilterBuilder {
            legalities_contains_any: Some(
                legalities_contains_any.into_iter().map(Into::into).collect(),
            ),
            ..CardFilterBuilder::default()
        }
    }

    /// Creates builder with oracle text filter (ability text substring match).
    pub fn with_oracle_text_contains(oracle_text_contains: impl Into<String>) -> CardFilterBuilder {
        let s = oracle_text_contains.into();
        CardFilterBuilder {
            oracle_text_contains: if s.is_empty() { None } else { Some(s) },
            ..CardFilterBuilder::default()
        }
    }

    /// Creates builder with flavor text filter.
    pub fn with_flavor_text_contains(flavor_text_contains: impl Into<String>) -> Self {
        let s: String = flavor_text_contains.into();
        Self {
            flavor_text_contains: if s.is_empty() { None } else { Some(s) },
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
        let s = type_line_contains.into();
        CardFilterBuilder {
            type_line_contains: if s.is_empty() { None } else { Some(s) },
            ..CardFilterBuilder::default()
        }
    }

    /// Creates builder matching any of the provided type line substrings.
    pub fn with_type_line_contains_any<I, S>(type_line_contains_any: I) -> CardFilterBuilder
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let v: Vec<String> = type_line_contains_any.into_iter()
            .map(|s| s.into())
            .collect();
        CardFilterBuilder {
            type_line_contains_any: if v.is_empty() { None } else { Some(v) },
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

    /// Creates builder requiring all provided type line substrings to be present (AND logic).
    pub fn with_type_line_contains_all<I, S>(type_line_contains_all: I) -> CardFilterBuilder
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        CardFilterBuilder {
            type_line_contains_all: Some(
                type_line_contains_all.into_iter().map(Into::into).collect(),
            ),
            ..CardFilterBuilder::default()
        }
    }

    /// Creates builder requiring all provided card types to be present (AND logic).
    pub fn with_card_type_contains_all<I>(card_type_contains_all: I) -> CardFilterBuilder
    where
        I: IntoIterator<Item = CardType>,
    {
        CardFilterBuilder {
            card_type_contains_all: Some(card_type_contains_all.into_iter().collect()),
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

    /// Creates builder matching cards that produce any of the listed mana colors (OR logic).
    ///
    /// Example: `with_produced_mana_contains_any(["R", "G"])` matches cards producing Red or Green.
    pub fn with_produced_mana_contains_any<I, S>(produced_mana_contains_any: I) -> CardFilterBuilder
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        CardFilterBuilder {
            produced_mana_contains_any: Some(
                produced_mana_contains_any
                    .into_iter()
                    .map(Into::into)
                    .collect(),
            ),
            ..CardFilterBuilder::default()
        }
    }

    /// Creates builder matching cards that produce all of the listed mana colors (AND logic).
    ///
    /// Example: `with_produced_mana_contains_all(["W", "U"])` matches cards producing both White and Blue.
    pub fn with_produced_mana_contains_all<I, S>(produced_mana_contains_all: I) -> CardFilterBuilder
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        CardFilterBuilder {
            produced_mana_contains_all: Some(
                produced_mana_contains_all
                    .into_iter()
                    .map(Into::into)
                    .collect(),
            ),
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
    pub fn with_order_by(order_by: CardSortKey) -> CardFilterBuilder {
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

        // Reject include/exclude contradictions: a value in both an include and
        // the exclude list for the same attribute matches zero cards. The
        // rarity filter derefs to a slice; the rest are already `Vec`s.
        let rarity_includes = self.rarity_equals_any.as_ref().map(|r| &r[..]);
        let rarity_excludes = self.rarity_excludes_any.as_ref().map(|r| &r[..]);
        check_include_exclude_clash(
            "card types",
            &[self.card_type_contains_any.as_deref(), self.card_type_contains_all.as_deref()],
            self.card_type_excludes_any.as_deref(),
        )?;
        check_include_exclude_clash(
            "type line",
            &[self.type_line_contains_any.as_deref(), self.type_line_contains_all.as_deref()],
            self.type_line_excludes_any.as_deref(),
        )?;
        check_include_exclude_clash(
            "oracle text",
            &[self.oracle_text_contains_any.as_deref(), self.oracle_text_contains_all.as_deref()],
            self.oracle_text_excludes_any.as_deref(),
        )?;
        check_include_exclude_clash(
            "keywords",
            &[self.keywords_contains_any.as_deref(), self.keywords_contains_all.as_deref()],
            self.keywords_excludes.as_deref(),
        )?;
        check_include_exclude_clash(
            "produced mana",
            &[self.produced_mana_contains_any.as_deref(), self.produced_mana_contains_all.as_deref()],
            self.produced_mana_excludes.as_deref(),
        )?;
        check_include_exclude_clash(
            "mechanical categories",
            &[
                self.mechanical_categories_contains_any.as_deref(),
                self.mechanical_categories_contains_all.as_deref(),
            ],
            self.mechanical_categories_excludes.as_deref(),
        )?;
        check_include_exclude_clash(
            "sets",
            &[self.set_equals_any.as_deref()],
            self.set_excludes_any.as_deref(),
        )?;
        check_include_exclude_clash(
            "artists",
            &[self.artist_equals_any.as_deref()],
            self.artist_excludes_any.as_deref(),
        )?;
        check_include_exclude_clash("rarities", &[rarity_includes], rarity_excludes)?;

        // Scalar substring pairs: "contains X" together with "doesn't contain X".
        check_contains_not_contains_clash(
            "name",
            self.name_contains.as_deref(),
            self.name_not_contains.as_deref(),
        )?;
        check_contains_not_contains_clash(
            "oracle text",
            self.oracle_text_contains.as_deref(),
            self.oracle_text_not_contains.as_deref(),
        )?;
        check_contains_not_contains_clash(
            "flavor text",
            self.flavor_text_contains.as_deref(),
            self.flavor_text_not_contains.as_deref(),
        )?;
        check_contains_not_contains_clash(
            "type line",
            self.type_line_contains.as_deref(),
            self.type_line_not_contains.as_deref(),
        )?;

        // Trim whitespace and strip punctuation from text fields at build time
        // (not on set, to allow typing spaces/punctuation in the UI).
        // Returns None if the cleaned value is empty.
        let clean = |s: &Option<String>| -> Option<String> {
            s.as_ref()
                .map(|v| strip_punctuation(v.trim()))
                .filter(|v| !v.is_empty())
        };
        let clean_vec = |v: &Option<Vec<String>>| -> Option<Vec<String>> {
            v.as_ref().map(|vec| {
                vec.iter()
                    .map(|s| strip_punctuation(s.trim()))
                    .filter(|s| !s.is_empty())
                    .collect()
            }).filter(|v: &Vec<String>| !v.is_empty())
        };
        // Trim only (no punctuation stripping) for exact-match fields.
        let trim = |s: &Option<String>| -> Option<String> {
            s.as_ref().map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
        };
        let trim_vec = |v: &Option<Vec<String>>| -> Option<Vec<String>> {
            v.as_ref().map(|vec| {
                vec.iter().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
            }).filter(|v: &Vec<String>| !v.is_empty())
        };

        Ok(CardFilter {
            power_equals: self.power_equals,
            power_range: self.power_range,
            toughness_equals: self.toughness_equals,
            toughness_range: self.toughness_range,
            cmc_equals: self.cmc_equals,
            cmc_range: self.cmc_range,
            color_identity_within: self.color_identity_within.clone(),
            color_identity_equals: self.color_identity_equals.clone(),
            price_min: self.price_min,
            price_max: self.price_max,
            price_currency: self.price_currency,
            produced_mana_contains_any: self.produced_mana_contains_any.clone(),
            produced_mana_contains_all: self.produced_mana_contains_all.clone(),
            produced_mana_excludes: self.produced_mana_excludes.clone(),
            rarity_equals_any: self.rarity_equals_any.clone(),
            rarity_excludes_any: self.rarity_excludes_any.clone(),
            set_equals_any: trim_vec(&self.set_equals_any),
            set_excludes_any: trim_vec(&self.set_excludes_any),
            artist_equals_any: trim_vec(&self.artist_equals_any),
            artist_excludes_any: trim_vec(&self.artist_excludes_any),
            name_contains: clean(&self.name_contains),
            name_not_contains: clean(&self.name_not_contains),
            oracle_text_contains: clean(&self.oracle_text_contains),
            oracle_text_not_contains: clean(&self.oracle_text_not_contains),
            oracle_text_contains_any: clean_vec(&self.oracle_text_contains_any),
            oracle_text_contains_all: clean_vec(&self.oracle_text_contains_all),
            oracle_text_excludes_any: clean_vec(&self.oracle_text_excludes_any),
            keywords_contains_any: trim_vec(&self.keywords_contains_any),
            keywords_contains_all: trim_vec(&self.keywords_contains_all),
            keywords_excludes: trim_vec(&self.keywords_excludes),
            flavor_text_contains: clean(&self.flavor_text_contains),
            flavor_text_not_contains: clean(&self.flavor_text_not_contains),
            has_flavor_text: self.has_flavor_text,
            type_line_contains: clean(&self.type_line_contains),
            type_line_not_contains: clean(&self.type_line_not_contains),
            type_line_contains_any: clean_vec(&self.type_line_contains_any),
            type_line_contains_all: clean_vec(&self.type_line_contains_all),
            type_line_excludes_any: clean_vec(&self.type_line_excludes_any),
            card_type_contains_any: self.card_type_contains_any.clone(),
            card_type_contains_all: self.card_type_contains_all.clone(),
            card_type_excludes_any: self.card_type_excludes_any.clone(),
            is_token: self.is_token,
            is_playable: self.is_playable,
            digital: self.digital,
            oversized: self.oversized,
            promo: self.promo,
            content_warning: self.content_warning,
            language: trim(&self.language),
            legalities_contains_any: self.legalities_contains_any.clone(),
            is_commander_in_format: self.is_commander_in_format,
            is_partner: self.is_partner,
            is_background: self.is_background,
            is_signature_spell: self.is_signature_spell,
            mechanical_categories_contains_any: self.mechanical_categories_contains_any.clone(),
            mechanical_categories_contains_all: self.mechanical_categories_contains_all.clone(),
            mechanical_categories_excludes: self.mechanical_categories_excludes.clone(),
            limit: self.limit,
            offset: self.offset,
            order_by: self.order_by,
            ascending: self.ascending,
            synergy: self.synergy,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exclude_only_is_not_empty() {
        let mut builder = CardFilterBuilder::new();
        builder.set_keywords_excludes(vec!["haste"]);
        assert!(!builder.is_empty(), "keywords_excludes alone should not be empty");
        assert!(builder.build().is_ok(), "keywords_excludes alone should build");
    }

    #[test]
    fn include_and_exclude_builds() {
        let mut builder = CardFilterBuilder::new();
        builder.set_keywords_contains_any(vec!["flying"]);
        builder.set_keywords_excludes(vec!["haste"]);
        assert!(!builder.is_empty());
        assert!(builder.build().is_ok());
    }

    #[test]
    fn card_type_include_exclude_clash_errors() {
        let mut builder = CardFilterBuilder::new();
        builder.set_card_type_contains_any(vec![CardType::Creature, CardType::Land]);
        builder.set_card_type_excludes_any(vec![CardType::Land]);
        assert!(
            matches!(builder.build(), Err(InvalidCardFilter::Contradiction { field, .. }) if field == "card types"),
            "including and excluding Land must be rejected",
        );
    }

    #[test]
    fn card_type_include_exclude_different_values_builds() {
        // Include creatures, exclude lands — different values, no clash.
        let mut builder = CardFilterBuilder::new();
        builder.set_card_type_contains_all(vec![CardType::Creature]);
        builder.set_card_type_excludes_any(vec![CardType::Land]);
        assert!(builder.build().is_ok());
    }

    #[test]
    fn keyword_include_exclude_clash_errors() {
        let mut builder = CardFilterBuilder::new();
        builder.set_keywords_contains_any(vec!["flying"]);
        builder.set_keywords_excludes(vec!["flying"]);
        assert!(matches!(
            builder.build(),
            Err(InvalidCardFilter::Contradiction { .. })
        ));
    }

    #[test]
    fn name_contains_and_not_contains_same_errors() {
        let mut builder = CardFilterBuilder::new();
        builder.set_name_contains("test");
        builder.set_name_not_contains("test");
        assert!(
            matches!(builder.build(), Err(InvalidCardFilter::Contradiction { field, .. }) if field == "name"),
            "name contains + doesn't-contain the same term must be rejected",
        );
    }

    #[test]
    fn name_not_contains_substring_of_contains_errors() {
        // "contains test" already implies "contains tes", so excluding "tes"
        // matches zero cards.
        let mut builder = CardFilterBuilder::new();
        builder.set_name_contains("Test");
        builder.set_name_not_contains("tes");
        assert!(matches!(
            builder.build(),
            Err(InvalidCardFilter::Contradiction { .. })
        ));
    }

    #[test]
    fn name_contains_and_not_contains_different_builds() {
        let mut builder = CardFilterBuilder::new();
        builder.set_name_contains("dragon");
        builder.set_name_not_contains("goblin");
        assert!(builder.build().is_ok());
    }

    #[test]
    fn synergy_is_a_mode_not_a_criterion() {
        // Synergy alone must not make the filter "active" — it's a server-side
        // membership mode, not a search criterion.
        let mut builder = CardFilterBuilder::new();
        builder.set_synergy(true);
        assert!(builder.is_empty());
        assert!(builder.build().is_err());
    }

    #[test]
    fn synergy_round_trips_through_build() {
        let mut builder = CardFilterBuilder::with_name_contains("bolt");
        builder.set_synergy(true);
        assert!(builder.build().unwrap().synergy());
    }
}
