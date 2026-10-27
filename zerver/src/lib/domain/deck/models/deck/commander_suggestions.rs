//! Suggested commanders: every deck the user owns, in any format, summarized
//! into a play style and scored against the precomputed commander tag
//! profiles, the variety cap over the ranking, and the dismissals that hide
//! one.

use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};

use thiserror::Error;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::is_basic_land_type_line;
use zwipe_core::http::contracts::suggestions::{
    PLAY_STYLE_IDENTITIES_SENT, PROFILE_ROLE_SHOWN, PROFILE_STAPLES_SENT, PROFILE_TAG_SENT,
    STAPLE_MIN_DECKS,
};

/// Decks whose tag profile a candidate is scored against: the user's most
/// recently updated ones, as many as a verified account can own
/// (`MAX_DECKS_PER_USER`), so every deck counts. The profiles come from one
/// batched query; colors are read from every deck regardless.
pub const MAX_PROFILED_DECKS: usize = 20;

/// Scored candidates the service pulls before capping, as many as the largest
/// request, so a short list has candidates to move up once some are capped.
pub const SCORED_POOL: i64 = 100;

/// Most commanders of one exact color identity the ranking keeps in place
/// before the rest of that identity fall behind every other candidate. A user
/// whose decks share one strong theme would otherwise see ten commanders of
/// the same identity; four leaves over half of a ten-row list to the user's
/// other colors while still letting a favorite identity dominate.
pub const MAX_PER_COLOR_IDENTITY: usize = 4;

/// Errors while building suggestions or changing dismissals.
#[derive(Debug, Error)]
pub enum CommanderSuggestionsError {
    /// Database operation failed.
    #[error(transparent)]
    Database(anyhow::Error),
}

/// Errors while constructing a [`CommanderDismissal`] from a path segment.
#[derive(Debug, Error)]
pub enum InvalidCommanderDismissal {
    /// Invalid oracle ID format.
    #[error(transparent)]
    OracleId(uuid::Error),
}

/// One user's dismissal of one commander.
#[derive(Debug, Clone)]
pub struct CommanderDismissal {
    /// The dismissing user.
    pub user_id: Uuid,
    /// Oracle id of the commander, covering every printing.
    pub oracle_id: Uuid,
}

impl CommanderDismissal {
    /// Creates a dismissal request.
    pub fn new(user_id: Uuid, oracle_id: Uuid) -> Self {
        Self { user_id, oracle_id }
    }

    /// Creates a dismissal request parsing the oracle id from a path segment.
    pub fn from_path(user_id: Uuid, oracle_id: &str) -> Result<Self, InvalidCommanderDismissal> {
        let oracle_id =
            Uuid::try_parse(oracle_id.trim()).map_err(InvalidCommanderDismissal::OracleId)?;
        Ok(Self::new(user_id, oracle_id))
    }
}

/// One of the user's decks, in any format, as the profile reads it.
#[derive(Debug, Clone)]
pub struct ProfiledDeck {
    /// The commander's oracle id; `None` for a deck without one.
    pub commander_oracle_id: Option<Uuid>,
    /// The deck's colors as WUBRG short names, from [`deck_colors`].
    pub colors: Vec<String>,
    /// Oracle ids of the deck's mainboard cards.
    pub mainboard_oracle_ids: Vec<Uuid>,
    /// The deck's distinct mainboard cards counted by color identity.
    pub card_colors: CardColorCounts,
}

/// One deck's distinct mainboard cards (by oracle id) counted by color
/// identity. A multicolored card counts toward each of its colors.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CardColorCounts {
    /// Cards whose identity includes each color, in WUBRG order.
    pub colors: [u32; 5],
    /// Cards with an empty identity.
    pub colorless: u32,
}

impl CardColorCounts {
    /// Reads the six counts the profiled decks query returns: W, U, B, R, G,
    /// then colorless. A missing or negative count reads 0.
    #[must_use]
    pub fn from_counts(counts: &[i64]) -> Self {
        let count = |i: usize| {
            counts
                .get(i)
                .and_then(|n| u32::try_from(*n).ok())
                .unwrap_or_default()
        };
        Self {
            colors: [count(0), count(1), count(2), count(3), count(4)],
            colorless: count(5),
        }
    }
}

impl ProfiledDeck {
    /// The oracle ids the deck's tag profile is built from: the commander,
    /// when there is one, then the mainboard.
    pub fn profile_oracle_ids(&self) -> Vec<Uuid> {
        self.commander_oracle_id
            .into_iter()
            .chain(self.mainboard_oracle_ids.iter().copied())
            .collect()
    }
}

/// A deck's colors in WUBRG order: its commander's color identity when it has
/// a commander, otherwise the union of its mainboard cards' color identities.
/// `card_colors` may repeat a color; colors outside WUBRG sort last.
#[must_use]
pub fn deck_colors(commander_colors: Option<Vec<String>>, card_colors: Vec<String>) -> Vec<String> {
    sorted_identity(&commander_colors.unwrap_or(card_colors))
}

/// One deck's oracle-tag profile with the card counts behind it.
#[derive(Debug, Clone, PartialEq)]
pub struct DeckTagProfile {
    /// The otag to weight map the scoring query reads, built by the lift steps
    /// in [`zwipe_core::domain::deck::tag_profile`], weights summing to 1.
    /// Every tag in it falls under a card role.
    pub weights: serde_json::Value,
    /// For each tag in `weights`, how many of the deck's distinct cards carry
    /// it.
    pub cards: HashMap<String, u32>,
}

/// An oracle tag the user's decks and a commander both lean on.
#[derive(Debug, Clone, PartialEq)]
pub struct SharedTag {
    /// The oracle tag slug.
    pub slug: String,
    /// The card role the tag falls under, as the card display groups it; `None`
    /// for a tag under no role.
    pub role: Option<String>,
}

/// How many of the user's decks use one of a commander's colors.
#[derive(Debug, Clone, PartialEq)]
pub struct ColorAffinity {
    /// One of `W`, `U`, `B`, `R`, `G`.
    pub color: String,
    /// Decks whose colors include it.
    pub decks: u32,
}

/// A candidate commander as the scoring query ranks it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScoredCommander {
    /// The commander's oracle id.
    pub oracle_id: Uuid,
    /// The commander's color identity as WUBRG short names.
    pub color_identity: Vec<String>,
    /// The slugs of the strongest tags shared with the user's best-matching
    /// deck, strongest first. Each is a key of that deck's tag profile.
    pub tag_slugs: Vec<String>,
}

/// Pairs each shared tag slug with the card role `tag_roles` maps it to, in
/// order. A slug missing from the map falls under no role.
#[must_use]
pub fn shared_tags(slugs: Vec<String>, tag_roles: &HashMap<String, String>) -> Vec<SharedTag> {
    slugs
        .into_iter()
        .map(|slug| SharedTag {
            role: tag_roles.get(&slug).cloned(),
            slug,
        })
        .collect()
}

/// Reorders a ranking so that at most `cap` commanders of each exact color
/// identity keep their place; every further commander of that identity moves
/// behind the whole list, in its original order. Nothing is dropped, and
/// commanders that stay keep their relative order, so a stable ranking stays
/// stable.
#[must_use]
pub fn cap_color_identities(scored: Vec<ScoredCommander>, cap: usize) -> Vec<ScoredCommander> {
    let mut kept_per_identity: HashMap<Vec<String>, usize> = HashMap::new();
    let mut kept = Vec::with_capacity(scored.len());
    let mut capped = Vec::new();
    for commander in scored {
        let mut identity = commander.color_identity.clone();
        identity.sort_unstable();
        identity.dedup();
        let seen = kept_per_identity.entry(identity).or_default();
        if *seen < cap {
            *seen += 1;
            kept.push(commander);
        } else {
            capped.push(commander);
        }
    }
    kept.extend(capped);
    kept
}

/// One suggestion with the terms behind it.
#[derive(Debug, Clone)]
pub struct CommanderSuggestion {
    /// The commander's current printing.
    pub card: Card,
    /// Shared oracle tags, strongest first.
    pub tags: Vec<SharedTag>,
    /// The commander's colors the user's decks use, most-used first.
    pub colors: Vec<ColorAffinity>,
}

/// How many of the user's decks share one exact color identity.
#[derive(Debug, Clone, PartialEq)]
pub struct ColorIdentityCount {
    /// The identity in WUBRG order; empty for colorless.
    pub colors: Vec<String>,
    /// Decks with exactly these colors.
    pub decks: u32,
}

/// An oracle tag the user's decks lean on, with its impact and the counts
/// behind it.
#[derive(Debug, Clone, PartialEq)]
pub struct ProfileTag {
    /// The oracle tag slug.
    pub slug: String,
    /// The card role the tag falls under, as [`SharedTag::role`].
    pub role: Option<String>,
    /// Profiled decks whose profile carries the tag.
    pub decks: u32,
    /// Cards carrying the tag, summed over those decks.
    pub cards: u32,
    /// The tag's weight summed over the deck profiles.
    pub score: f64,
}

/// What the user builds, summarized from the decks the suggestions are
/// ranked against.
#[derive(Debug, Clone, PartialEq)]
pub struct BuilderProfile {
    /// Decks summarized, in any format.
    pub deck_count: u32,
    /// Each color the decks use with its deck count, most decks first,
    /// WUBRG order between equals.
    pub colors: Vec<ColorAffinity>,
    /// The most-built exact identity; `None` when every identity is unique
    /// across more than one deck.
    pub top_identity: Option<ColorIdentityCount>,
    /// The highest-scoring oracle tags, highest first, at most
    /// [`PROFILE_TAG_SENT`].
    pub tags: Vec<ProfileTag>,
    /// The card roles the profiles' tags fall under, most decks first and
    /// role between equals, at most [`PROFILE_ROLE_SHOWN`]. Counted over
    /// every tag in the profiles, not only `tags`.
    pub roles: Vec<ProfileRole>,
    /// The cards on the most decks' mainboards, in [`rank_staples`] order.
    /// [`summarize_profile`] leaves it empty; the service fills it once the
    /// cards are fetched.
    pub staples: Vec<Staple>,
    /// The cards and decks behind the color map.
    pub color_map: ColorMap,
}

/// A card on several of the user's decks' mainboards.
#[derive(Debug, Clone, PartialEq)]
pub struct Staple {
    /// The printing [`StapleCount::scryfall_data_id`] names.
    pub card: Card,
    /// Decks with the card on their mainboard.
    pub decks: u32,
}

/// One mainboard card of one of the user's decks, as [`rank_staples`] reads
/// it.
#[derive(Debug, Clone, PartialEq)]
pub struct MainboardCard {
    /// The deck the card is in.
    pub deck_id: Uuid,
    /// The card's oracle id, shared by every printing.
    pub oracle_id: Uuid,
    /// The printing the deck runs.
    pub scryfall_data_id: Uuid,
    /// The card's name, the tiebreak between equal deck counts.
    pub name: String,
    /// The printing's type line; `None` when Scryfall gives none.
    pub type_line: Option<String>,
}

/// A staple before its card is fetched.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StapleCount {
    /// The card's oracle id.
    pub oracle_id: Uuid,
    /// The printing to show: the one in the first deck, in input order, that
    /// has the card.
    pub scryfall_data_id: Uuid,
    /// Distinct decks with the card on their mainboard.
    pub decks: u32,
}

/// The user's staples from every mainboard card of every deck, the input in
/// decks' most recently updated order. A card counts once per deck however
/// many printings or copies the deck runs, and only cards in at least
/// [`STAPLE_MIN_DECKS`] decks stay. Basic lands are left out. Sorted by
/// decks, most first, then name, then oracle id, at most
/// [`PROFILE_STAPLES_SENT`]. Each keeps the printing of the first row that
/// names its oracle id, so with rows in recency order that is the printing in
/// the most recently updated deck.
#[must_use]
pub fn rank_staples(cards: &[MainboardCard]) -> Vec<StapleCount> {
    let mut by_oracle: HashMap<Uuid, (&MainboardCard, HashSet<Uuid>)> = HashMap::new();
    for card in cards {
        if card
            .type_line
            .as_deref()
            .is_some_and(is_basic_land_type_line)
        {
            continue;
        }
        by_oracle
            .entry(card.oracle_id)
            .or_insert_with(|| (card, HashSet::new()))
            .1
            .insert(card.deck_id);
    }
    let mut ranked: Vec<(&MainboardCard, u32)> = by_oracle
        .into_values()
        .map(|(first, decks)| (first, u32::try_from(decks.len()).unwrap_or(u32::MAX)))
        .filter(|(_, decks)| *decks >= STAPLE_MIN_DECKS)
        .collect();
    ranked.sort_by(|(a, a_decks), (b, b_decks)| {
        b_decks
            .cmp(a_decks)
            .then_with(|| a.name.cmp(&b.name))
            .then_with(|| a.oracle_id.cmp(&b.oracle_id))
    });
    ranked
        .into_iter()
        .take(PROFILE_STAPLES_SENT)
        .map(|(first, decks)| StapleCount {
            oracle_id: first.oracle_id,
            scryfall_data_id: first.scryfall_data_id,
            decks,
        })
        .collect()
}

/// How the user's cards and decks spread over the colors and color pairs,
/// summed across every profiled deck.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColorMap {
    /// All five colors in WUBRG order with their cards.
    pub colors: Vec<ColorCards>,
    /// Cards with an empty color identity.
    pub colorless_cards: u32,
    /// All ten color pairs in WUBRG order with their decks.
    pub pairs: Vec<ColorPairDecks>,
}

/// One color's cards across the user's decks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColorCards {
    /// One of `W`, `U`, `B`, `R`, `G`.
    pub color: String,
    /// Mainboard cards whose identity includes the color, each counted once
    /// per deck.
    pub cards: u32,
}

/// One color pair's decks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColorPairDecks {
    /// The two colors in WUBRG order.
    pub colors: [String; 2],
    /// Decks whose color identity includes both.
    pub decks: u32,
}

/// The color map of `decks`: per color the cards summed over every deck's
/// [`CardColorCounts`], the colorless cards apart, and per color pair the
/// decks whose colors hold both, so a three-color deck counts toward three
/// pairs. Every color and pair is present, zeros included.
#[must_use]
pub fn color_map(decks: &[ProfiledDeck]) -> ColorMap {
    let mut cards = [0_u32; 5];
    let mut colorless_cards = 0_u32;
    for deck in decks {
        for (total, count) in cards.iter_mut().zip(deck.card_colors.colors) {
            *total = total.saturating_add(count);
        }
        colorless_cards = colorless_cards.saturating_add(deck.card_colors.colorless);
    }
    let colors = WUBRG
        .iter()
        .zip(cards)
        .map(|(color, cards)| ColorCards {
            color: (*color).to_string(),
            cards,
        })
        .collect();
    let pairs = WUBRG
        .iter()
        .enumerate()
        .flat_map(|(i, first)| {
            WUBRG
                .iter()
                .skip(i.saturating_add(1))
                .map(move |second| (*first, *second))
        })
        .map(|(first, second)| {
            let holding = decks
                .iter()
                .filter(|deck| {
                    deck.colors.iter().any(|c| c == first)
                        && deck.colors.iter().any(|c| c == second)
                })
                .count();
            ColorPairDecks {
                colors: [first.to_string(), second.to_string()],
                decks: u32::try_from(holding).unwrap_or(u32::MAX),
            }
        })
        .collect();
    ColorMap {
        colors,
        colorless_cards,
        pairs,
    }
}

/// A card role the user's decks lean on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileRole {
    /// The card role slug.
    pub role: String,
    /// Profiled decks with at least one tag under the role.
    pub decks: u32,
}

/// The WUBRG colors in order.
const WUBRG: [&str; 5] = ["W", "U", "B", "R", "G"];

/// A color's place in WUBRG order; colors outside it sort last.
fn wubrg_index(color: &str) -> usize {
    WUBRG
        .iter()
        .position(|c| *c == color)
        .unwrap_or(WUBRG.len())
}

/// A deck's color identity deduplicated and in WUBRG order.
fn sorted_identity(colors: &[String]) -> Vec<String> {
    let mut identity: Vec<String> = colors.to_vec();
    identity.sort_by_key(|c| wubrg_index(c));
    identity.dedup();
    identity
}

/// Each exact color identity across `decks` with how many decks have it,
/// most decks first, then more colors, then WUBRG order (white before blue
/// and so on, color by color). The first entry is the profile's top identity.
#[must_use]
pub fn identity_counts(decks: &[ProfiledDeck]) -> Vec<ColorIdentityCount> {
    let mut identity_decks: HashMap<Vec<String>, u32> = HashMap::new();
    for deck in decks {
        *identity_decks
            .entry(sorted_identity(&deck.colors))
            .or_default() += 1;
    }
    let mut counts: Vec<ColorIdentityCount> = identity_decks
        .into_iter()
        .map(|(colors, decks)| ColorIdentityCount { colors, decks })
        .collect();
    counts.sort_by_cached_key(|count| {
        let order: Vec<usize> = count.colors.iter().map(|c| wubrg_index(c)).collect();
        (Reverse(count.decks), Reverse(count.colors.len()), order)
    });
    counts
}

/// The user's deck count and most built identities, from
/// [`identity_counts`] cut to [`PLAY_STYLE_IDENTITIES_SENT`].
#[must_use]
pub fn summarize_play_style(decks: &[ProfiledDeck]) -> PlayStyleSummary {
    let mut identities = identity_counts(decks);
    identities.truncate(PLAY_STYLE_IDENTITIES_SENT);
    PlayStyleSummary {
        deck_count: u32::try_from(decks.len()).unwrap_or(u32::MAX),
        identities,
    }
}

/// What Home's play style panel shows: the deck count and the most built
/// exact identities.
#[derive(Debug, Clone, PartialEq)]
pub struct PlayStyleSummary {
    /// Decks read, in any format. 0 is the cold start.
    pub deck_count: u32,
    /// The most built identities in [`identity_counts`] order, at most
    /// [`PLAY_STYLE_IDENTITIES_SENT`].
    pub identities: Vec<ColorIdentityCount>,
}

/// Summarizes the user's decks: per color the decks using it, the
/// [`color_map`], the most-built exact identity, the highest-scoring oracle tags across the
/// deck profiles, and the card roles those tags fall under. `profiles` pairs with
/// `decks` by position (shorter is fine; the rest count as unprofiled).
/// `tag_roles` maps a tag slug to its role; a slug absent from it is under
/// no role. `None` when there are no decks.
///
/// A tag's score is the sum of its weight over the profiles that carry it;
/// weights are normalized per deck, so the sum is deck-weighted importance.
/// Its cards are summed from the same profiles' card counts.
/// A role counts a deck once however many of its tags the deck carries.
/// The top identity breaks ties by more colors, then WUBRG order; it is
/// `None` when more than one deck exists and no identity repeats.
#[must_use]
pub fn summarize_profile(
    decks: &[ProfiledDeck],
    profiles: &[Option<DeckTagProfile>],
    tag_roles: &HashMap<String, String>,
) -> Option<BuilderProfile> {
    if decks.is_empty() {
        return None;
    }
    let deck_count = u32::try_from(decks.len()).unwrap_or(u32::MAX);

    let mut color_decks: HashMap<String, u32> = HashMap::new();
    for deck in decks {
        for color in sorted_identity(&deck.colors) {
            *color_decks.entry(color).or_default() += 1;
        }
    }
    let mut colors: Vec<ColorAffinity> = color_decks
        .into_iter()
        .filter(|(color, decks)| *decks > 0 && WUBRG.contains(&color.as_str()))
        .map(|(color, decks)| ColorAffinity { color, decks })
        .collect();
    colors.sort_by_key(|c| (Reverse(c.decks), wubrg_index(&c.color)));

    let top_identity = identity_counts(decks)
        .into_iter()
        .next()
        .filter(|top| decks.len() == 1 || top.decks > 1);

    let mut tag_totals: HashMap<String, ProfileTag> = HashMap::new();
    let mut role_decks: HashMap<String, u32> = HashMap::new();
    for profile in profiles.iter().flatten() {
        let Some(weights) = profile.weights.as_object() else {
            continue;
        };
        let mut deck_roles: HashSet<&String> = HashSet::new();
        for (slug, weight) in weights {
            let total = tag_totals
                .entry(slug.clone())
                .or_insert_with(|| ProfileTag {
                    slug: slug.clone(),
                    role: tag_roles.get(slug).cloned(),
                    decks: 0,
                    cards: 0,
                    score: 0.0,
                });
            total.score += weight.as_f64().unwrap_or_default();
            total.decks += 1;
            total.cards += profile.cards.get(slug).copied().unwrap_or_default();
            if let Some(role) = tag_roles.get(slug) {
                deck_roles.insert(role);
            }
        }
        for role in deck_roles {
            *role_decks.entry(role.clone()).or_default() += 1;
        }
    }
    let mut tags: Vec<ProfileTag> = tag_totals.into_values().collect();
    tags.sort_by(|a, b| {
        b.score
            .total_cmp(&a.score)
            .then_with(|| a.slug.cmp(&b.slug))
    });
    tags.truncate(PROFILE_TAG_SENT);
    let mut roles: Vec<ProfileRole> = role_decks
        .into_iter()
        .map(|(role, decks)| ProfileRole { role, decks })
        .collect();
    roles.sort_by(|a, b| b.decks.cmp(&a.decks).then_with(|| a.role.cmp(&b.role)));
    roles.truncate(PROFILE_ROLE_SHOWN);

    Some(BuilderProfile {
        deck_count,
        colors,
        top_identity,
        tags,
        roles,
        staples: vec![],
        color_map: color_map(decks),
    })
}

/// The ranked suggestions and the deck count that fed them.
#[derive(Debug, Clone)]
pub struct CommanderSuggestions {
    /// Ranked best first.
    pub suggestions: Vec<CommanderSuggestion>,
    /// The user's decks summarized, in any format. 0 means no profile could
    /// be built.
    pub deck_count: u32,
    /// The decks summarized; `None` at the cold start.
    pub profile: Option<BuilderProfile>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scored(n: u128, identity: &[&str]) -> ScoredCommander {
        ScoredCommander {
            oracle_id: Uuid::from_u128(n),
            color_identity: identity.iter().map(|c| c.to_string()).collect(),
            tag_slugs: vec![],
        }
    }

    fn ids(scored: &[ScoredCommander]) -> Vec<u128> {
        scored.iter().map(|s| s.oracle_id.as_u128()).collect()
    }

    #[test]
    fn empty_stays_empty() {
        assert!(cap_color_identities(vec![], 4).is_empty());
    }

    #[test]
    fn under_the_cap_keeps_the_order() {
        let input = vec![scored(1, &["U"]), scored(2, &["U"]), scored(3, &["R", "W"])];
        let out = cap_color_identities(input.clone(), 4);
        assert_eq!(out, input);
    }

    #[test]
    fn exactly_the_cap_keeps_the_order() {
        let input: Vec<_> = (1..=4).map(|n| scored(n, &["U"])).collect();
        let out = cap_color_identities(input.clone(), 4);
        assert_eq!(out, input);
    }

    #[test]
    fn over_the_cap_moves_the_rest_behind_everything_in_score_order() {
        let input = vec![
            scored(1, &["U"]),
            scored(2, &["U"]),
            scored(3, &["U"]),
            scored(4, &["W", "R"]),
            scored(5, &["U"]),
            scored(6, &["G"]),
            scored(7, &["R", "W"]),
            scored(8, &["U"]),
            scored(9, &["U"]),
        ];
        let out = cap_color_identities(input, 2);
        assert_eq!(
            ids(&out),
            vec![1, 2, 4, 6, 7, 3, 5, 8, 9],
            "two of each identity stay, the capped blues follow in their old order"
        );
    }

    #[test]
    fn identity_order_does_not_matter() {
        let input = vec![
            scored(1, &["W", "R"]),
            scored(2, &["R", "W"]),
            scored(3, &["R"]),
        ];
        let out = cap_color_identities(input, 1);
        assert_eq!(ids(&out), vec![1, 3, 2]);
    }

    #[test]
    fn colorless_is_an_identity_too() {
        let input = vec![scored(1, &[]), scored(2, &[]), scored(3, &["B"])];
        let out = cap_color_identities(input, 1);
        assert_eq!(ids(&out), vec![1, 3, 2]);
    }

    fn deck(n: u128, colors: &[&str]) -> ProfiledDeck {
        ProfiledDeck {
            commander_oracle_id: Some(Uuid::from_u128(n)),
            colors: colors.iter().map(|c| c.to_string()).collect(),
            mainboard_oracle_ids: vec![],
            card_colors: CardColorCounts::default(),
        }
    }

    fn strings(colors: &[&str]) -> Vec<String> {
        colors.iter().map(|c| c.to_string()).collect()
    }

    #[test]
    fn a_deck_without_a_commander_takes_its_cards_colors_in_wubrg_order() {
        assert_eq!(
            deck_colors(None, strings(&["G", "W", "G", "B", "W"])),
            strings(&["W", "B", "G"]),
            "the union of the mainboard's identities, deduplicated"
        );
        assert!(deck_colors(None, vec![]).is_empty(), "colorless cards");
    }

    #[test]
    fn a_commander_decides_the_deck_colors() {
        assert_eq!(
            deck_colors(Some(strings(&["R", "W"])), strings(&["U", "R"])),
            strings(&["W", "R"]),
            "the commander's identity, not the cards'"
        );
        assert!(
            deck_colors(Some(vec![]), strings(&["U"])).is_empty(),
            "a colorless commander keeps the deck colorless"
        );
    }

    #[test]
    fn a_deck_without_a_commander_profiles_its_mainboard_only() {
        let deck = ProfiledDeck {
            commander_oracle_id: None,
            colors: vec![],
            mainboard_oracle_ids: vec![Uuid::from_u128(2), Uuid::from_u128(3)],
            card_colors: CardColorCounts::default(),
        };
        assert_eq!(
            deck.profile_oracle_ids(),
            vec![Uuid::from_u128(2), Uuid::from_u128(3)]
        );
        let led = ProfiledDeck {
            commander_oracle_id: Some(Uuid::from_u128(1)),
            ..deck
        };
        assert_eq!(
            led.profile_oracle_ids(),
            vec![Uuid::from_u128(1), Uuid::from_u128(2), Uuid::from_u128(3)]
        );
    }

    fn affinity(color: &str, decks: u32) -> ColorAffinity {
        ColorAffinity {
            color: color.to_string(),
            decks,
        }
    }

    #[test]
    fn no_decks_is_no_profile() {
        assert_eq!(summarize_profile(&[], &[], &HashMap::new()), None);
    }

    #[test]
    fn colors_sort_by_decks_then_wubrg_and_omit_zeros() {
        let decks = vec![
            deck(1, &["R", "W"]),
            deck(2, &["W", "U", "R"]),
            deck(3, &["G"]),
            deck(4, &["R"]),
        ];
        let profile = summarize_profile(&decks, &[], &HashMap::new()).unwrap();
        assert_eq!(profile.deck_count, 4);
        assert_eq!(
            profile.colors,
            vec![
                affinity("R", 3),
                affinity("W", 2),
                affinity("U", 1),
                affinity("G", 1)
            ],
            "no black, blue before green on the tie"
        );
    }

    #[test]
    fn colorless_deck_contributes_no_color() {
        let decks = vec![deck(1, &[]), deck(2, &["B"])];
        let profile = summarize_profile(&decks, &[], &HashMap::new()).unwrap();
        assert_eq!(profile.colors, vec![affinity("B", 1)]);
    }

    #[test]
    fn top_identity_is_the_most_built_in_wubrg_order() {
        let decks = vec![
            deck(1, &["R", "W"]),
            deck(2, &["W", "R"]),
            deck(3, &["U"]),
            deck(4, &["U", "R", "W"]),
        ];
        let profile = summarize_profile(&decks, &[], &HashMap::new()).unwrap();
        assert_eq!(
            profile.top_identity,
            Some(ColorIdentityCount {
                colors: vec!["W".to_string(), "R".to_string()],
                decks: 2,
            })
        );
    }

    #[test]
    fn top_identity_tie_prefers_more_colors_then_wubrg() {
        let decks = vec![
            deck(1, &["U"]),
            deck(2, &["U"]),
            deck(3, &["R", "G"]),
            deck(4, &["G", "R"]),
            deck(5, &["W", "U"]),
            deck(6, &["U", "W"]),
        ];
        let profile = summarize_profile(&decks, &[], &HashMap::new()).unwrap();
        assert_eq!(
            profile.top_identity.map(|t| t.colors),
            Some(vec!["W".to_string(), "U".to_string()]),
            "two colors beat one, and WU comes before RG"
        );
    }

    #[test]
    fn top_identity_is_none_when_every_identity_is_unique() {
        let decks = vec![deck(1, &["U"]), deck(2, &["R"]), deck(3, &[])];
        let profile = summarize_profile(&decks, &[], &HashMap::new()).unwrap();
        assert_eq!(profile.top_identity, None);
    }

    #[test]
    fn a_single_deck_is_its_own_top_identity() {
        let decks = vec![deck(1, &["G", "B"])];
        let profile = summarize_profile(&decks, &[], &HashMap::new()).unwrap();
        assert_eq!(
            profile.top_identity,
            Some(ColorIdentityCount {
                colors: vec!["B".to_string(), "G".to_string()],
                decks: 1,
            })
        );
    }

    fn count(colors: &[&str], decks: u32) -> ColorIdentityCount {
        ColorIdentityCount {
            colors: strings(colors),
            decks,
        }
    }

    #[test]
    fn identity_counts_order_by_decks_then_more_colors_then_wubrg() {
        let decks = vec![
            deck(1, &["G", "R"]),
            deck(2, &["U"]),
            deck(3, &["R", "G"]),
            deck(4, &["U"]),
            deck(5, &["U", "W"]),
            deck(6, &["W", "U"]),
            deck(7, &[]),
            deck(8, &["B"]),
            deck(9, &["W", "U", "B", "R", "G"]),
        ];
        assert_eq!(
            identity_counts(&decks),
            vec![
                count(&["W", "U"], 2),
                count(&["R", "G"], 2),
                count(&["U"], 2),
                count(&["W", "U", "B", "R", "G"], 1),
                count(&["B"], 1),
                count(&[], 1),
            ],
            "two colors before one at two decks, WU before RG, then the singles"
        );
    }

    #[test]
    fn identity_counts_lead_with_the_top_identity() {
        let decks = vec![
            deck(1, &["U"]),
            deck(2, &["U"]),
            deck(3, &["R", "G"]),
            deck(4, &["G", "R"]),
            deck(5, &["W", "U"]),
            deck(6, &["U", "W"]),
        ];
        let profile = summarize_profile(&decks, &[], &HashMap::new()).unwrap();
        assert_eq!(
            identity_counts(&decks).first(),
            profile.top_identity.as_ref()
        );
    }

    #[test]
    fn play_style_keeps_the_top_three_and_counts_every_deck() {
        let decks = vec![
            deck(1, &["B"]),
            deck(2, &["B"]),
            deck(3, &["B"]),
            deck(4, &["R"]),
            deck(5, &["R"]),
            deck(6, &["W", "U"]),
            deck(7, &["G"]),
            deck(8, &[]),
        ];
        assert_eq!(
            summarize_play_style(&decks),
            PlayStyleSummary {
                deck_count: 8,
                identities: vec![count(&["B"], 3), count(&["R"], 2), count(&["W", "U"], 1)],
            }
        );
    }

    #[test]
    fn play_style_of_no_decks_is_empty() {
        assert_eq!(
            summarize_play_style(&[]),
            PlayStyleSummary {
                deck_count: 0,
                identities: vec![],
            }
        );
    }

    /// A deck profile from its weights, each tag carried by `cards` cards.
    fn tag_profile(weights: serde_json::Value, cards: &[(&str, u32)]) -> DeckTagProfile {
        DeckTagProfile {
            weights,
            cards: cards
                .iter()
                .map(|(slug, n)| (slug.to_string(), *n))
                .collect(),
        }
    }

    /// A deck profile whose tags each sit on one card.
    fn weights_only(weights: serde_json::Value) -> DeckTagProfile {
        let cards = weights
            .as_object()
            .map(|w| w.keys().map(|slug| (slug.clone(), 1)).collect())
            .unwrap_or_default();
        DeckTagProfile { weights, cards }
    }

    #[test]
    fn tags_sum_score_and_cards_across_decks_and_count_decks() {
        let decks = vec![deck(1, &["R"]), deck(2, &["R"]), deck(3, &["R"])];
        let profiles = vec![
            Some(tag_profile(
                serde_json::json!({ "treasure": 0.6, "sacrifice-outlet": 0.4 }),
                &[("treasure", 9), ("sacrifice-outlet", 4)],
            )),
            Some(tag_profile(
                serde_json::json!({ "sacrifice-outlet": 0.5, "lifegain": 0.5 }),
                &[("sacrifice-outlet", 7), ("lifegain", 12)],
            )),
            None,
        ];
        let profile = summarize_profile(&decks, &profiles, &HashMap::new()).unwrap();
        let got: Vec<(&str, u32, u32)> = profile
            .tags
            .iter()
            .map(|t| (t.slug.as_str(), t.decks, t.cards))
            .collect();
        assert_eq!(
            got,
            vec![
                ("sacrifice-outlet", 2, 11),
                ("treasure", 1, 9),
                ("lifegain", 1, 12)
            ],
            "0.9 over two decks, then 0.6, then 0.5; cards add up across decks"
        );
        let scores: Vec<f64> = profile.tags.iter().map(|t| t.score).collect();
        let want = [0.9, 0.6, 0.5];
        assert_eq!(scores.len(), want.len());
        for (got, want) in scores.iter().zip(want) {
            assert!((got - want).abs() < 1e-9, "score {got} want {want}");
        }
        assert!(
            scores.is_sorted_by(|a, b| a >= b),
            "highest score first: {scores:?}"
        );
        assert!(profile.tags.iter().all(|t| t.role.is_none()));
        assert!(profile.roles.is_empty(), "no role map, no roles");
    }

    #[test]
    fn score_orders_tags_not_cards_or_decks() {
        let decks = vec![deck(1, &["R"]), deck(2, &["R"])];
        // "common" sits in both decks on many cards but weighs little in
        // each; "rare" sits on two cards in one deck and weighs the most.
        let profiles = vec![
            Some(tag_profile(
                serde_json::json!({ "common": 0.2, "rare": 0.8 }),
                &[("common", 20), ("rare", 2)],
            )),
            Some(tag_profile(
                serde_json::json!({ "common": 0.3, "other": 0.7 }),
                &[("common", 25), ("other", 3)],
            )),
        ];
        let profile = summarize_profile(&decks, &profiles, &HashMap::new()).unwrap();
        let got: Vec<(&str, u32, u32)> = profile
            .tags
            .iter()
            .map(|t| (t.slug.as_str(), t.decks, t.cards))
            .collect();
        assert_eq!(
            got,
            vec![("rare", 1, 2), ("other", 1, 3), ("common", 2, 45)],
            "0.8, 0.7, then 0.5 despite the most decks and cards"
        );
    }

    #[test]
    fn a_tag_missing_from_the_card_counts_adds_no_cards() {
        let decks = vec![deck(1, &["R"])];
        let profiles = vec![Some(tag_profile(
            serde_json::json!({ "treasure": 1.0 }),
            &[],
        ))];
        let profile = summarize_profile(&decks, &profiles, &HashMap::new()).unwrap();
        let tag = profile.tags.first().unwrap();
        assert_eq!((tag.decks, tag.cards), (1, 0));
    }

    fn roles(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(slug, role)| (slug.to_string(), role.to_string()))
            .collect()
    }

    #[test]
    fn roles_count_a_deck_once_and_tie_on_name() {
        let decks = vec![deck(1, &["R"]), deck(2, &["R"]), deck(3, &["R"])];
        let profiles = vec![
            Some(weights_only(
                serde_json::json!({ "treasure": 0.3, "sacrifice-outlet": 0.3, "aristocrats": 0.4 }),
            )),
            Some(weights_only(
                serde_json::json!({ "sacrifice-outlet": 0.5, "lifegain": 0.5 }),
            )),
            Some(weights_only(serde_json::json!({ "lifegain": 1.0 }))),
        ];
        let map = roles(&[
            ("sacrifice-outlet", "sacrifice"),
            ("aristocrats", "sacrifice"),
            ("lifegain", "lifegain"),
            ("treasure", "ramp"),
        ]);
        let profile = summarize_profile(&decks, &profiles, &map).unwrap();
        let got: Vec<(&str, u32)> = profile
            .roles
            .iter()
            .map(|r| (r.role.as_str(), r.decks))
            .collect();
        assert_eq!(
            got,
            vec![("lifegain", 2), ("sacrifice", 2), ("ramp", 1)],
            "two sacrifice tags in deck 1 count it once; lifegain before sacrifice on the tie"
        );
        assert_eq!(
            profile
                .tags
                .iter()
                .find(|t| t.slug == "treasure")
                .and_then(|t| t.role.clone()),
            Some("ramp".to_string()),
            "shown tags carry their role"
        );
    }

    #[test]
    fn roles_count_tags_beyond_the_sent_cap_and_are_capped() {
        let decks = vec![deck(1, &["R"])];
        // 140 tags, each under its own role; only PROFILE_TAG_SENT tags go out.
        let weights: serde_json::Map<String, serde_json::Value> = (0..140)
            .map(|i| (format!("tag-{i:03}"), serde_json::json!(0.1)))
            .collect();
        let map: HashMap<String, String> = (0..140)
            .map(|i| (format!("tag-{i:03}"), format!("role-{i:03}")))
            .collect();
        let profiles = vec![Some(weights_only(serde_json::Value::Object(weights)))];
        let profile = summarize_profile(&decks, &profiles, &map).unwrap();
        assert_eq!(profile.tags.len(), PROFILE_TAG_SENT);
        assert_eq!(profile.roles.len(), PROFILE_ROLE_SHOWN);
        assert_eq!(
            profile.roles.first().map(|r| r.role.as_str()),
            Some("role-000")
        );
        assert_eq!(
            profile.roles.last().map(|r| r.role.as_str()),
            Some("role-009")
        );
    }

    #[test]
    fn tags_are_capped_and_tie_on_slug() {
        let decks = vec![deck(1, &["R"])];
        let weights: serde_json::Map<String, serde_json::Value> = (0..130)
            .map(|i| (format!("tag-{i:03}"), serde_json::json!(0.1)))
            .collect();
        let profiles = vec![Some(weights_only(serde_json::Value::Object(weights)))];
        let profile = summarize_profile(&decks, &profiles, &HashMap::new()).unwrap();
        let slugs: Vec<&str> = profile.tags.iter().map(|t| t.slug.as_str()).collect();
        assert_eq!(slugs.len(), PROFILE_TAG_SENT);
        assert_eq!(slugs.first(), Some(&"tag-000"));
        assert_eq!(slugs.last(), Some(&"tag-099"));
    }

    #[test]
    fn a_role_whose_tags_rank_below_the_top_ten_still_sends_them() {
        let decks = vec![deck(1, &["G"])];
        // 20 strong tags under one role, then two weak ramp tags.
        let mut weights: serde_json::Map<String, serde_json::Value> = (0..20)
            .map(|i| (format!("removal-{i:02}"), serde_json::json!(0.04)))
            .collect();
        weights.insert("mana-rock".to_string(), serde_json::json!(0.01));
        weights.insert("land-ramp".to_string(), serde_json::json!(0.02));
        let mut map: HashMap<String, String> = (0..20)
            .map(|i| (format!("removal-{i:02}"), "removal".to_string()))
            .collect();
        map.insert("mana-rock".to_string(), "ramp".to_string());
        map.insert("land-ramp".to_string(), "ramp".to_string());
        let profiles = vec![Some(weights_only(serde_json::Value::Object(weights)))];
        let profile = summarize_profile(&decks, &profiles, &map).unwrap();
        assert!(
            profile.roles.iter().any(|r| r.role == "ramp"),
            "ramp is a top role"
        );
        let ramp: Vec<(usize, &str)> = profile
            .tags
            .iter()
            .enumerate()
            .filter(|(_, t)| t.role.as_deref() == Some("ramp"))
            .map(|(i, t)| (i, t.slug.as_str()))
            .collect();
        assert_eq!(
            ramp,
            vec![(20, "land-ramp"), (21, "mana-rock")],
            "both ramp tags sit past the top ten, strongest first"
        );
    }

    /// A mainboard row: deck `deck`, oracle `oracle`, printing `printing`.
    fn row(deck: u128, oracle: u128, printing: u128, name: &str) -> MainboardCard {
        MainboardCard {
            deck_id: Uuid::from_u128(deck),
            oracle_id: Uuid::from_u128(oracle),
            scryfall_data_id: Uuid::from_u128(printing),
            name: name.to_string(),
            type_line: Some("Artifact".to_string()),
        }
    }

    fn staple_ids(staples: &[StapleCount]) -> Vec<(u128, u32)> {
        staples
            .iter()
            .map(|s| (s.oracle_id.as_u128(), s.decks))
            .collect()
    }

    #[test]
    fn no_rows_is_no_staples() {
        assert!(rank_staples(&[]).is_empty());
    }

    #[test]
    fn a_card_in_one_deck_is_not_a_staple() {
        let rows = vec![
            row(1, 10, 100, "Sol Ring"),
            row(2, 11, 110, "Arcane Signet"),
        ];
        assert!(rank_staples(&rows).is_empty(), "each card is in one deck");
    }

    #[test]
    fn a_card_counts_once_per_deck() {
        let rows = vec![
            row(1, 10, 100, "Sol Ring"),
            row(1, 10, 101, "Sol Ring"),
            row(1, 10, 100, "Sol Ring"),
            row(2, 11, 110, "Arcane Signet"),
            row(3, 11, 110, "Arcane Signet"),
        ];
        assert_eq!(
            staple_ids(&rank_staples(&rows)),
            vec![(11, 2)],
            "three Sol Ring rows in one deck are one deck"
        );
    }

    #[test]
    fn staples_sort_by_decks_then_name_then_oracle() {
        let rows = vec![
            row(1, 30, 300, "Zebra"),
            row(2, 30, 300, "Zebra"),
            row(1, 20, 200, "Apple"),
            row(2, 20, 200, "Apple"),
            row(1, 21, 210, "Apple"),
            row(2, 21, 210, "Apple"),
            row(1, 40, 400, "Middle"),
            row(2, 40, 400, "Middle"),
            row(3, 40, 400, "Middle"),
        ];
        assert_eq!(
            staple_ids(&rank_staples(&rows)),
            vec![(40, 3), (20, 2), (21, 2), (30, 2)],
            "most decks first, then name, then oracle id"
        );
    }

    #[test]
    fn at_most_a_hundred_staples() {
        let rows: Vec<MainboardCard> = (0..105u128)
            .flat_map(|n| {
                let name = format!("Card {n:03}");
                [row(1, n, n, &name), row(2, n, n, &name)]
            })
            .collect();
        let staples = rank_staples(&rows);
        assert_eq!(staples.len(), PROFILE_STAPLES_SENT);
        assert_eq!(
            staple_ids(&staples),
            (0..100u128).map(|n| (n, 2)).collect::<Vec<_>>(),
            "the first hundred by name"
        );
    }

    #[test]
    fn the_first_deck_with_the_card_picks_the_printing() {
        let rows = vec![
            row(1, 10, 101, "Sol Ring"),
            row(2, 10, 102, "Sol Ring"),
            row(3, 10, 103, "Sol Ring"),
        ];
        let staples = rank_staples(&rows);
        assert_eq!(
            staples
                .iter()
                .map(|s| s.scryfall_data_id.as_u128())
                .collect::<Vec<_>>(),
            vec![101],
            "rows arrive most recent deck first"
        );
    }

    #[test]
    fn basic_lands_are_left_out() {
        let basics = [
            "Basic Land — Forest",
            "Basic Snow Land — Island",
            "Basic Land",
        ];
        let mut rows = Vec::new();
        for (n, type_line) in (50u128..).zip(basics) {
            for deck in 1..=3 {
                rows.push(MainboardCard {
                    type_line: Some(type_line.to_string()),
                    ..row(deck, n, n, type_line)
                });
            }
        }
        for deck in 1..=2 {
            rows.push(MainboardCard {
                type_line: Some("Snow Land — Forest".to_string()),
                ..row(deck, 60, 60, "Snow Dual")
            });
            rows.push(MainboardCard {
                type_line: None,
                ..row(deck, 61, 61, "No Type")
            });
        }
        assert_eq!(
            staple_ids(&rank_staples(&rows)),
            vec![(61, 2), (60, 2)],
            "basics, snow basics and Wastes drop; a nonbasic snow land stays"
        );
    }

    fn counted(n: u128, colors: &[&str], cards: [u32; 5], colorless: u32) -> ProfiledDeck {
        ProfiledDeck {
            card_colors: CardColorCounts {
                colors: cards,
                colorless,
            },
            ..deck(n, colors)
        }
    }

    fn pair_decks(map: &ColorMap) -> Vec<(String, u32)> {
        map.pairs
            .iter()
            .map(|p| (p.colors.concat(), p.decks))
            .collect()
    }

    #[test]
    fn color_map_sums_cards_and_keeps_colorless_apart() {
        let decks = vec![
            counted(1, &["W", "R"], [10, 0, 0, 12, 0], 5),
            counted(2, &["R"], [0, 0, 0, 30, 0], 7),
        ];
        let map = color_map(&decks);
        let cards: Vec<(&str, u32)> = map
            .colors
            .iter()
            .map(|c| (c.color.as_str(), c.cards))
            .collect();
        assert_eq!(
            cards,
            vec![("W", 10), ("U", 0), ("B", 0), ("R", 42), ("G", 0)],
            "every color in WUBRG order, zeros kept"
        );
        assert_eq!(
            map.colorless_cards, 12,
            "colorless sums apart from the corners"
        );
    }

    #[test]
    fn color_map_counts_every_pair_a_deck_holds() {
        let decks = vec![
            deck(1, &["W", "B", "G"]),
            deck(2, &["W", "G"]),
            deck(3, &["U"]),
            deck(4, &[]),
        ];
        let map = color_map(&decks);
        let want: Vec<(String, u32)> = [
            ("WU", 0),
            ("WB", 1),
            ("WR", 0),
            ("WG", 2),
            ("UB", 0),
            ("UR", 0),
            ("UG", 0),
            ("BR", 0),
            ("BG", 1),
            ("RG", 0),
        ]
        .iter()
        .map(|(pair, decks)| (pair.to_string(), *decks))
        .collect();
        assert_eq!(
            pair_decks(&map),
            want,
            "a three-color deck counts toward its three pairs, a mono or colorless deck toward none"
        );
    }

    #[test]
    fn five_color_decks_light_all_ten_pairs() {
        let map = color_map(&[deck(1, &WUBRG)]);
        assert_eq!(map.pairs.len(), 10);
        assert!(map.pairs.iter().all(|p| p.decks == 1), "{map:?}");
    }

    #[test]
    fn card_color_counts_read_the_query_order() {
        assert_eq!(
            CardColorCounts::from_counts(&[1, 2, 3, 4, 5, 6]),
            CardColorCounts {
                colors: [1, 2, 3, 4, 5],
                colorless: 6,
            }
        );
        assert_eq!(
            CardColorCounts::from_counts(&[-1, 2]),
            CardColorCounts {
                colors: [0, 2, 0, 0, 0],
                colorless: 0,
            },
            "a short or negative row reads 0"
        );
    }

    #[test]
    fn the_profile_carries_the_color_map() {
        let decks = vec![counted(1, &["U", "R"], [0, 9, 0, 8, 0], 2)];
        let profile = summarize_profile(&decks, &[], &HashMap::new()).unwrap();
        assert_eq!(profile.color_map, color_map(&decks));
    }
}
