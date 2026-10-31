//! The Playstyle screen's pure helpers, ported from the app's `suggested_commanders.rs`: color identities and the Show row's filter, the list's empty lines, a suggestion's reasons, the profile's Summary, Card roles and Oracle tags, and the Staples panel's Type and Color filters.

#[cfg(test)]
mod tests;

use std::collections::BTreeMap;

use zwipe_core::domain::card::card_role;
use zwipe_core::domain::card::scryfall_data::colors::{self, Color};
use zwipe_core::domain::card::search_card::card_type::CardType;
use zwipe_core::domain::deck::deck_metrics::DeckMetrics;
use zwipe_core::http::contracts::suggestions::{
    HttpBuilderProfile, HttpColorAffinity, HttpProfileRole, HttpProfileTag, HttpSharedTag,
    HttpStaple, MAX_SUGGESTION_LIMIT,
};

use crate::domain::activity::Bar;
use crate::domain::color_map::ColorMapCounts;

/// Suggestions the list shows; a shorter list after a dropped row refetches.
pub(crate) const LIST_SIZE: usize = 10;

/// Suggestions fetched and held, the server's maximum; the list shows the first [`LIST_SIZE`] of them and Swipe deals them all.
pub(crate) const HELD_SIZE: u32 = MAX_SUGGESTION_LIMIT;

/// The impact the top tag reads.
pub(crate) const TOP_IMPACT: u32 = 100;

/// Rows the oracle tags table shows: the user's 10 most impactful tags. The profile sends more so each card role can reveal its own tags.
pub(crate) const TAG_TABLE_ROWS: usize = 10;

/// The Staples Type row's card types, in the row's fixed order.
pub(crate) const STAPLE_TYPES: [CardType; 7] = [
    CardType::Creature,
    CardType::Artifact,
    CardType::Enchantment,
    CardType::Instant,
    CardType::Sorcery,
    CardType::Planeswalker,
    CardType::Land,
];

/// A commander's exact color identity in WUBRG order, each color once; empty is colorless.
pub(crate) fn identity_of(colors: &[Color]) -> Vec<Color> {
    Color::all()
        .into_iter()
        .filter(|c| colors.contains(c))
        .collect()
}

/// The distinct color identities among `colors` (one entry per commander), in the deck list's group order: fewest colors first, WUBRG order between identities of the same size, colorless last.
pub(crate) fn identities_present<'a>(
    colors: impl IntoIterator<Item = &'a [Color]>,
) -> Vec<Vec<Color>> {
    let mut identities: Vec<Vec<Color>> = Vec::new();
    for commander in colors {
        let identity = identity_of(commander);
        if !identities.contains(&identity) {
            identities.push(identity);
        }
    }
    identities.sort_by_key(|identity| {
        let order: Vec<usize> = identity
            .iter()
            .map(|c| Color::all().iter().position(|a| a == c).unwrap_or(0))
            .collect();
        (identity.is_empty(), identity.len(), order)
    });
    identities
}

/// The items whose `colors` are exactly `identity`, in the order given; every item when `identity` is `None` (the All chip). The server already spreads its 100 across identities, so this is a plain predicate over that order and applies no cap.
pub(crate) fn of_identity<T: Clone>(
    items: &[T],
    colors: impl Fn(&T) -> &[Color],
    identity: Option<&[Color]>,
) -> Vec<T> {
    items
        .iter()
        .filter(|item| identity.is_none_or(|wanted| identity_of(colors(item)) == wanted))
        .cloned()
        .collect()
}

/// The first [`LIST_SIZE`] of a filtered list, the rows the screen shows.
pub(crate) fn shown<T: Clone>(filtered: &[T]) -> Vec<T> {
    filtered.iter().take(LIST_SIZE).cloned().collect()
}

/// The name shown beside an identity's pips: none for a single color, the identity's name otherwise ("Azorius", "Five-color", "Colorless").
pub(crate) fn identity_label(identity: &[Color]) -> Option<&'static str> {
    (identity.len() != 1).then(|| colors::identity_name(identity))
}

/// An identity as the app's `IdentityPips` reads in plain text: the name, then the pips' letters. A single color is its letter alone; colorless is the name alone. "Azorius WU", "R", "Colorless".
pub(crate) fn identity_text(identity: &[Color]) -> String {
    let identity = identity_of(identity);
    let pips: String = identity.iter().map(Color::to_short_name).collect();
    match (identity_label(&identity), pips.is_empty()) {
        (Some(name), true) => name.to_owned(),
        (Some(name), false) => format!("{name} {pips}"),
        (None, _) => pips,
    }
}

/// The list's empty state under a Show chip whose commanders are all gone, "No Jeskai commanders in your top 100". The names that describe rather than name (colorless, mono-color, five-color) go lowercase mid-sentence.
pub(crate) fn empty_identity_line(identity: &[Color]) -> String {
    let identity = identity_of(identity);
    let name = colors::identity_name(&identity);
    let name = match identity.len() {
        0 | 1 | 5 => name.to_lowercase(),
        _ => name.to_owned(),
    };
    format!("No {name} commanders in your top 100")
}

/// A staple's share of the user's decks for its row header, rounded half up, "67%". `None` with no decks to count against.
pub(crate) fn staple_percent(decks: u32, deck_count: u32) -> Option<String> {
    u64::from(decks)
        .checked_mul(200)
        .and_then(|doubled| doubled.checked_add(u64::from(deck_count)))
        .and_then(|sum| sum.checked_div(u64::from(deck_count).checked_mul(2)?))
        .map(|percent| format!("{percent}%"))
}

/// "deck" for one deck, "decks" otherwise.
fn decks_noun(deck_count: u32) -> &'static str {
    if deck_count == 1 { "deck" } else { "decks" }
}

/// A staple's deck count for its expanded detail, "4 of 6 decks".
pub(crate) fn staple_decks(decks: u32, deck_count: u32) -> String {
    let noun = decks_noun(deck_count);
    format!("{decks} of {deck_count} {noun}")
}

/// A WUBRG code's lowercase name, `None` for anything else.
fn color_name(code: &str) -> Option<&'static str> {
    match code {
        "W" => Some("white"),
        "U" => Some("blue"),
        "B" => Some("black"),
        "R" => Some("red"),
        "G" => Some("green"),
        _ => None,
    }
}

/// One line per color the user builds in, in the order the server sent (most decks first). A color code outside WUBRG is skipped.
pub(crate) fn color_lines(colors: &[HttpColorAffinity], deck_count: u32) -> Vec<String> {
    let noun = decks_noun(deck_count);
    colors
        .iter()
        .filter_map(|c| {
            let name = color_name(&c.color)?;
            let decks = c.decks;
            Some(format!(
                "You build in {name} ({decks} of your {deck_count} {noun})"
            ))
        })
        .collect()
}

/// A suggestion's shared tags grouped for its reasons: the roles in order of first appearance (the tags come strongest first), each role's tags, and the role-less tags for the "Other" bucket.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct SharedTags {
    pub(crate) roles: Vec<String>,
    pub(crate) by_role: BTreeMap<String, Vec<String>>,
    pub(crate) other: Vec<String>,
}

/// The shared tags in the shape the app's `CardRoleChips` takes.
pub(crate) fn group_shared_tags(tags: &[HttpSharedTag]) -> SharedTags {
    let mut grouped = SharedTags::default();
    for tag in tags {
        match &tag.role {
            Some(role) => {
                if !grouped.roles.contains(role) {
                    grouped.roles.push(role.clone());
                }
                grouped
                    .by_role
                    .entry(role.clone())
                    .or_default()
                    .push(tag.slug.clone());
            }
            None => grouped.other.push(tag.slug.clone()),
        }
    }
    grouped
}

/// A Summary row's value: an identity drawn as pips, and its deck count.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SummaryValue {
    /// The colors drawn as pips in WUBRG order; empty for colorless.
    pub(crate) pips: Vec<Color>,
    /// The deck count after the pips, "(4)".
    pub(crate) count: String,
}

/// The Summary section's rows: the deck count, then the most common color set and the top color, each absent when the profile has none.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProfileSummary {
    /// The Decks row's value, the bare count.
    pub(crate) decks: String,
    /// The top identity with its count; `None` when the profile has none.
    pub(crate) color_set: Option<SummaryValue>,
    /// The color in the most decks with its count; `None` when no color parses.
    pub(crate) color: Option<SummaryValue>,
}

/// The Summary rows from the profile. The top color is the first WUBRG color the server sent, which is the one in the most decks; codes outside WUBRG are skipped.
pub(crate) fn profile_summary(profile: &HttpBuilderProfile) -> ProfileSummary {
    let color_set = profile.top_identity.as_ref().map(|top| {
        let parsed: Vec<Color> = top
            .colors
            .iter()
            .filter_map(|code| Color::try_from(code.as_str()).ok())
            .collect();
        SummaryValue {
            pips: identity_of(&parsed),
            count: format!("({})", top.decks),
        }
    });
    let color = profile.colors.iter().find_map(|c| {
        let color = Color::try_from(c.color.as_str()).ok()?;
        Some(SummaryValue {
            pips: vec![color],
            count: format!("({})", c.decks),
        })
    });
    ProfileSummary {
        decks: profile.deck_count.to_string(),
        color_set,
        color,
    }
}

/// Whether a color map has anything to draw: a colored or colorless card.
pub(crate) fn color_map_has_data(counts: &ColorMapCounts) -> bool {
    counts.colorless > 0 || counts.cards.iter().any(|n| *n > 0)
}

/// A deck count as a bar size.
fn decks_usize(decks: u32) -> usize {
    usize::try_from(decks).unwrap_or(usize::MAX)
}

/// One bar per role in the order sent, labeled as the deck role chart labels the role and keyed by the role slug, sized against the role in the most decks with [`DeckMetrics::bar_pct`]. A role with none of `tags` under it has nothing to reveal.
pub(crate) fn role_bars(roles: &[HttpProfileRole], tags: &[HttpProfileTag]) -> Vec<Bar> {
    let max = roles
        .iter()
        .map(|r| decks_usize(r.decks))
        .max()
        .unwrap_or(0);
    roles
        .iter()
        .map(|r| Bar {
            key: r.role.clone(),
            label: card_role::role_short_label(&r.role),
            count: u64::from(r.decks),
            pct: DeckMetrics::bar_pct(decks_usize(r.decks), max),
            revealable: tags
                .iter()
                .any(|t| t.role.as_deref() == Some(r.role.as_str())),
        })
        .collect()
}

/// One row of the oracle tags table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TagRow {
    /// The oracle tag slug, the row's chip.
    pub(crate) slug: String,
    /// Profiled decks carrying the tag.
    pub(crate) decks: u32,
    /// Cards carrying the tag across those decks.
    pub(crate) cards: u32,
    /// The tag's score against the top tag's, rounded, the top tag reading [`TOP_IMPACT`]. `None` when no tag scores above 0.
    pub(crate) impact: Option<u32>,
}

/// `score` against `max` on a 0 to [`TOP_IMPACT`] scale, rounded. `None` when `max` is not above 0; a negative or NaN score reads 0.
pub(crate) fn impact(score: f64, max: f64) -> Option<u32> {
    if max.is_nan() || max <= 0.0 {
        return None;
    }
    let scaled = (score.max(0.0) / max * f64::from(TOP_IMPACT)).round();
    // There is no TryFrom from f64 to an integer, so find the whole number instead.
    Some(
        (0..=TOP_IMPACT)
            .find(|n| f64::from(*n) >= scaled)
            .unwrap_or(TOP_IMPACT),
    )
}

/// One row per oracle tag in the order sent (highest score first), the first [`TAG_TABLE_ROWS`] only, its impact scaled so the top tag reads [`TOP_IMPACT`].
pub(crate) fn tag_rows(tags: &[HttpProfileTag]) -> Vec<TagRow> {
    let max = tags.iter().map(|t| t.score).fold(0.0, f64::max);
    tags.iter()
        .take(TAG_TABLE_ROWS)
        .map(|t| TagRow {
            slug: t.slug.clone(),
            decks: t.decks,
            cards: t.cards,
            impact: impact(t.score, max),
        })
        .collect()
}

/// The profile's tags under `role`, highest score first.
pub(crate) fn role_tags(tags: &[HttpProfileTag], role: &str) -> Vec<String> {
    tags.iter()
        .filter(|t| t.role.as_deref() == Some(role))
        .map(|t| t.slug.clone())
        .collect()
}

/// A chip on the Staples Color row: one color, or colorless.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StapleColor {
    /// Cards whose color identity includes the color.
    Of(Color),
    /// Cards with an empty color identity.
    Colorless,
}

impl StapleColor {
    /// Every chip in the row's order: WUBRG, then colorless.
    pub(crate) fn all() -> [Self; 6] {
        let [white, blue, black, red, green] = Color::all();
        [
            Self::Of(white),
            Self::Of(blue),
            Self::Of(black),
            Self::Of(red),
            Self::Of(green),
            Self::Colorless,
        ]
    }

    /// Whether `identity` falls under the chip. A multicolor identity falls under each of its colors.
    pub(crate) fn matches(self, identity: &[Color]) -> bool {
        match self {
            Self::Of(color) => identity.contains(&color),
            Self::Colorless => identity.is_empty(),
        }
    }

    /// What the chip draws: the color's one pip, or no pips, which reads "Colorless".
    pub(crate) fn pips(self) -> Vec<Color> {
        match self {
            Self::Of(color) => vec![color],
            Self::Colorless => Vec::new(),
        }
    }
}

/// The front face of a type line. Scryfall joins a multi-faced card's faces with `" // "`; a single-faced line comes back whole.
fn front_face(type_line: &str) -> &str {
    type_line
        .split_once(" // ")
        .map_or(type_line, |(front, _)| front)
}

/// Whether the front face of `type_line` has `card_type` as one of its words, so "Artifact Creature" has both Artifact and Creature.
pub(crate) fn has_type(type_line: Option<&str>, card_type: CardType) -> bool {
    let word = card_type.to_string();
    type_line.is_some_and(|line| front_face(line).split_whitespace().any(|w| w == word))
}

/// A staple's type line, its first face's where the card has no top-level one.
pub(crate) fn staple_type_line(staple: &HttpStaple) -> Option<&str> {
    let data = &staple.card.scryfall_data;
    data.type_line
        .as_deref()
        .or_else(|| data.card_faces.as_ref()?.first()?.type_line.as_deref())
}

/// A staple's color identity, what the Color row reads.
pub(crate) fn staple_identity(staple: &HttpStaple) -> &[Color] {
    &staple.card.scryfall_data.color_identity
}

/// The Type row's chips: each of [`STAPLE_TYPES`] that at least one item has, in that fixed order.
pub(crate) fn staple_types_present<T>(
    items: &[T],
    type_line: impl Fn(&T) -> Option<&str>,
) -> Vec<CardType> {
    STAPLE_TYPES
        .into_iter()
        .filter(|card_type| {
            items
                .iter()
                .any(|item| has_type(type_line(item), *card_type))
        })
        .collect()
}

/// The Color row's chips: each of [`StapleColor::all`] that at least one item falls under, in that fixed order.
pub(crate) fn staple_colors_present<T>(
    items: &[T],
    colors: impl Fn(&T) -> &[Color],
) -> Vec<StapleColor> {
    StapleColor::all()
        .into_iter()
        .filter(|chip| items.iter().any(|item| chip.matches(colors(item))))
        .collect()
}

/// The first [`LIST_SIZE`] items with `card_type` and under `color`, in the order given; `None` on either row is All, and the two rows combine.
pub(crate) fn matching_staples<T: Clone>(
    items: &[T],
    type_line: impl Fn(&T) -> Option<&str>,
    colors: impl Fn(&T) -> &[Color],
    card_type: Option<CardType>,
    color: Option<StapleColor>,
) -> Vec<T> {
    items
        .iter()
        .filter(|item| {
            card_type.is_none_or(|wanted| has_type(type_line(item), wanted))
                && color.is_none_or(|chip| chip.matches(colors(item)))
        })
        .take(LIST_SIZE)
        .cloned()
        .collect()
}
