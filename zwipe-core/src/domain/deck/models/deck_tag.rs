//! Deck archetype/strategy tags.
//!
//! A deck can be labeled with a small set of these to describe its overall
//! strategy (Aggro, Tokens, Reanimator, …). Distinct from the card-level
//! `MechanicalCategory`: these describe a whole deck's game plan, not a single
//! card's role. The set is curated (from EDHREC themes and Archidekt/Moxfield
//! tags) and fixed so tags stay clean and filterable.
//!
//! The list is large on purpose — the picker is a searchable typeahead, so more
//! options cost nothing on screen. Variants are only ever added, never removed
//! or renamed, so previously-stored tags keep parsing.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Maximum number of tags a single deck may carry.
pub const MAX_DECK_TAGS: usize = 5;

/// A deck-level archetype or theme tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeckTag {
    Aggro,
    Aikido,
    AlternateWincon,
    Aristocrats,
    Artifacts,
    AttackTriggers,
    Auras,
    BigMana,
    Blink,
    Blood,
    Bounce,
    Burn,
    Cascade,
    Cheerios,
    Clone,
    Clues,
    Combo,
    CommanderMatters,
    Control,
    Counters,
    Counterspells,
    Cycling,
    Defenders,
    Devotion,
    Discard,
    Draw,
    Enchantments,
    Enchantress,
    Energy,
    Equipment,
    Etb,
    ExtraCombats,
    ExtraTurns,
    Flying,
    Food,
    GlassCannon,
    GoodStuff,
    GoWide,
    Graveyard,
    GroupHug,
    GroupSlug,
    Hatebears,
    Infect,
    LandDestruction,
    Landfall,
    Lands,
    Lifedrain,
    Lifegain,
    Midrange,
    Mill,
    Monarch,
    Multicolor,
    Pillowfort,
    Ping,
    Poison,
    Politics,
    PowerMatters,
    Prison,
    Proliferate,
    Ramp,
    Reanimator,
    Removal,
    Sacrifice,
    Sagas,
    SelfMill,
    Speed,
    Spellslinger,
    Stax,
    Stompy,
    Storm,
    Superfriends,
    Tempo,
    Theft,
    Tokens,
    Toolbox,
    ToughnessMatters,
    Toxic,
    Treasure,
    Tribal,
    Turbo,
    TurboFog,
    Untap,
    Vehicles,
    Voltron,
    Wheels,
    Wipe,
    Zoo,
}

impl DeckTag {
    /// All deck tag variants, alphabetical.
    pub fn all() -> &'static [DeckTag] {
        &[
            Self::Aggro,
            Self::Aikido,
            Self::AlternateWincon,
            Self::Aristocrats,
            Self::Artifacts,
            Self::AttackTriggers,
            Self::Auras,
            Self::BigMana,
            Self::Blink,
            Self::Blood,
            Self::Bounce,
            Self::Burn,
            Self::Cascade,
            Self::Cheerios,
            Self::Clone,
            Self::Clues,
            Self::Combo,
            Self::CommanderMatters,
            Self::Control,
            Self::Counters,
            Self::Counterspells,
            Self::Cycling,
            Self::Defenders,
            Self::Devotion,
            Self::Discard,
            Self::Draw,
            Self::Enchantments,
            Self::Enchantress,
            Self::Energy,
            Self::Equipment,
            Self::Etb,
            Self::ExtraCombats,
            Self::ExtraTurns,
            Self::Flying,
            Self::Food,
            Self::GlassCannon,
            Self::GoodStuff,
            Self::GoWide,
            Self::Graveyard,
            Self::GroupHug,
            Self::GroupSlug,
            Self::Hatebears,
            Self::Infect,
            Self::LandDestruction,
            Self::Landfall,
            Self::Lands,
            Self::Lifedrain,
            Self::Lifegain,
            Self::Midrange,
            Self::Mill,
            Self::Monarch,
            Self::Multicolor,
            Self::Pillowfort,
            Self::Ping,
            Self::Poison,
            Self::Politics,
            Self::PowerMatters,
            Self::Prison,
            Self::Proliferate,
            Self::Ramp,
            Self::Reanimator,
            Self::Removal,
            Self::Sacrifice,
            Self::Sagas,
            Self::SelfMill,
            Self::Speed,
            Self::Spellslinger,
            Self::Stax,
            Self::Stompy,
            Self::Storm,
            Self::Superfriends,
            Self::Tempo,
            Self::Theft,
            Self::Tokens,
            Self::Toolbox,
            Self::ToughnessMatters,
            Self::Toxic,
            Self::Treasure,
            Self::Tribal,
            Self::Turbo,
            Self::TurboFog,
            Self::Untap,
            Self::Vehicles,
            Self::Voltron,
            Self::Wheels,
            Self::Wipe,
            Self::Zoo,
        ]
    }

    /// Human-readable display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Aggro => "Aggro",
            Self::Aikido => "Aikido",
            Self::AlternateWincon => "Alternate Win",
            Self::Aristocrats => "Aristocrats",
            Self::Artifacts => "Artifacts",
            Self::AttackTriggers => "Attack Triggers",
            Self::Auras => "Auras",
            Self::BigMana => "Big Mana",
            Self::Blink => "Blink",
            Self::Blood => "Blood",
            Self::Bounce => "Bounce",
            Self::Burn => "Burn",
            Self::Cascade => "Cascade",
            Self::Cheerios => "Cheerios",
            Self::Clone => "Clone",
            Self::Clues => "Clues",
            Self::Combo => "Combo",
            Self::CommanderMatters => "Commander Matters",
            Self::Control => "Control",
            Self::Counters => "+1/+1 Counters",
            Self::Counterspells => "Counterspells",
            Self::Cycling => "Cycling",
            Self::Defenders => "Defenders",
            Self::Devotion => "Devotion",
            Self::Discard => "Discard",
            Self::Draw => "Draw",
            Self::Enchantments => "Enchantments",
            Self::Enchantress => "Enchantress",
            Self::Energy => "Energy",
            Self::Equipment => "Equipment",
            Self::Etb => "ETB",
            Self::ExtraCombats => "Extra Combats",
            Self::ExtraTurns => "Extra Turns",
            Self::Flying => "Flying",
            Self::Food => "Food",
            Self::GlassCannon => "Glass Cannon",
            Self::GoodStuff => "Good Stuff",
            Self::GoWide => "Go Wide",
            Self::Graveyard => "Graveyard",
            Self::GroupHug => "Group Hug",
            Self::GroupSlug => "Group Slug",
            Self::Hatebears => "Hatebears",
            Self::Infect => "Infect",
            Self::LandDestruction => "Land Destruction",
            Self::Landfall => "Landfall",
            Self::Lands => "Lands Matter",
            Self::Lifedrain => "Lifedrain",
            Self::Lifegain => "Lifegain",
            Self::Midrange => "Midrange",
            Self::Mill => "Mill",
            Self::Monarch => "Monarch",
            Self::Multicolor => "Multicolor",
            Self::Pillowfort => "Pillowfort",
            Self::Ping => "Pingers",
            Self::Poison => "Poison",
            Self::Politics => "Politics",
            Self::PowerMatters => "Power Matters",
            Self::Prison => "Prison",
            Self::Proliferate => "Proliferate",
            Self::Ramp => "Ramp",
            Self::Reanimator => "Reanimator",
            Self::Removal => "Removal",
            Self::Sacrifice => "Sacrifice",
            Self::Sagas => "Sagas",
            Self::SelfMill => "Self-Mill",
            Self::Speed => "Speed",
            Self::Spellslinger => "Spellslinger",
            Self::Stax => "Stax",
            Self::Stompy => "Stompy",
            Self::Storm => "Storm",
            Self::Superfriends => "Superfriends",
            Self::Tempo => "Tempo",
            Self::Theft => "Theft",
            Self::Tokens => "Tokens",
            Self::Toolbox => "Toolbox",
            Self::ToughnessMatters => "Toughness Matters",
            Self::Toxic => "Toxic",
            Self::Treasure => "Treasure",
            Self::Tribal => "Tribal",
            Self::Turbo => "Turbo",
            Self::TurboFog => "Turbo Fog",
            Self::Untap => "Untap",
            Self::Vehicles => "Vehicles",
            Self::Voltron => "Voltron",
            Self::Wheels => "Wheels",
            Self::Wipe => "Board Wipes",
            Self::Zoo => "Zoo",
        }
    }

    /// One-line, plain-language definition of the strategy, for the tag picker's
    /// hint dialog.
    pub fn description(&self) -> &'static str {
        match self {
            Self::Aggro => "Apply early pressure with cheap creatures to win before opponents stabilize",
            Self::Aikido => "Turn opponents' own attacks and resources against them",
            Self::AlternateWincon => "Win through a card's alternate victory condition",
            Self::Aristocrats => "Sacrifice your own creatures for value and incremental life drain",
            Self::Artifacts => "Build the engine around artifacts and their synergies",
            Self::AttackTriggers => "Reward attacking with triggers that snowball each combat",
            Self::Auras => "Stack enchantment auras onto a creature to enlarge it",
            Self::BigMana => "Generate huge mana to power out oversized threats and X spells",
            Self::Blink => "Flicker creatures out and back to reuse their enter-the-battlefield effects",
            Self::Blood => "Make Blood tokens to loot away cards and fuel payoffs",
            Self::Bounce => "Return permanents to hand to reset boards and reuse effects",
            Self::Burn => "Deal direct damage with spells to creatures or players",
            Self::Cascade => "Chain free spells off cascade and cost-reduction effects",
            Self::Cheerios => "Chain zero-cost artifacts to draw into a combo finish",
            Self::Clone => "Copy the best creatures and permanents on the battlefield",
            Self::Clues => "Investigate for Clue tokens to draw and trigger payoffs",
            Self::Combo => "Assemble two or more cards into a game-ending interaction",
            Self::CommanderMatters => "Reward casting and building around your commander",
            Self::Control => "Counter and remove threats, then win with late-game inevitability",
            Self::Counters => "Grow creatures with +1/+1 counters and counter payoffs",
            Self::Counterspells => "Hold up counterspells to stop key spells before they resolve",
            Self::Cycling => "Discard cards to draw and trigger cycling payoffs",
            Self::Defenders => "Hold the line with high-toughness walls, then turn them into a win",
            Self::Devotion => "Reward heavy colored pips with devotion-scaling effects",
            Self::Discard => "Strip opponents' hands to deny resources and options",
            Self::Draw => "Refill your hand with extra card draw to outresource the table",
            Self::Enchantments => "Build the engine around enchantments and their synergies",
            Self::Enchantress => "Draw cards whenever you cast enchantments",
            Self::Energy => "Accumulate energy counters and spend them for value",
            Self::Equipment => "Suit up creatures with equipment for repeatable buffs",
            Self::Etb => "Chain enter-the-battlefield effects for repeatable value",
            Self::ExtraCombats => "Take additional combat steps to attack again and again",
            Self::ExtraTurns => "Chain extra turns to pull ahead and close the game",
            Self::Flying => "Win in the air with evasive flying creatures",
            Self::Food => "Make Food tokens to gain life and feed sacrifice value",
            Self::GlassCannon => "Win fast with little defense if the plan is disrupted",
            Self::GoodStuff => "Play the strongest standalone cards over tight synergy",
            Self::GoWide => "Flood the board with many creatures, then buff them all at once",
            Self::Graveyard => "Use the graveyard as a resource to recur and reuse cards",
            Self::GroupHug => "Give every player resources to steer politics and the game",
            Self::GroupSlug => "Punish the whole table with symmetric damage and taxes",
            Self::Hatebears => "Disrupt opponents with small creatures that tax and deny",
            Self::Infect => "Attack with infect creatures, dealing damage as poison counters",
            Self::LandDestruction => "Destroy opponents' lands to deny mana and tempo",
            Self::Landfall => "Trigger payoffs each time a land enters the battlefield",
            Self::Lands => "Treat lands as the engine, ramping and recurring them for value",
            Self::Lifedrain => "Drain opponents' life while padding your own total",
            Self::Lifegain => "Gain life and turn that life total into payoffs",
            Self::Midrange => "Play efficient threats and removal to grind out value",
            Self::Mill => "Empty opponents' libraries to deck them out",
            Self::Monarch => "Become the monarch for an extra card each turn",
            Self::Multicolor => "Reward playing many colors and gold cards",
            Self::Pillowfort => "Defend yourself with deterrents so attacks go elsewhere",
            Self::Ping => "Repeatedly deal 1 damage to pick off creatures and players",
            Self::Poison => "Win through poison counters from infect and toxic sources",
            Self::Politics => "Make deals and incentives to steer the multiplayer table",
            Self::PowerMatters => "Reward high power and pumping your creatures' power",
            Self::Prison => "Lock opponents out with continuous resource-denial effects",
            Self::Proliferate => "Multiply every kind of existing counter with proliferate",
            Self::Ramp => "Accelerate mana to cast big spells ahead of schedule",
            Self::Reanimator => "Cheat large creatures from the graveyard into play",
            Self::Removal => "Lean on efficient spot removal to answer any threat",
            Self::Sacrifice => "Use sacrifice outlets to turn permanents into value",
            Self::Sagas => "Build around saga enchantments and their chapter effects",
            Self::SelfMill => "Mill your own library to fuel graveyard payoffs",
            Self::Speed => "Prioritize a fast clock to win before opponents set up",
            Self::Spellslinger => "Cast many instants and sorceries with spell payoffs",
            Self::Stax => "Slow the whole table with symmetric resource-denial pieces",
            Self::Stompy => "Ramp into big, efficient creatures and swing",
            Self::Storm => "Cast a chain of spells to power up storm finishers",
            Self::Superfriends => "Deploy many planeswalkers and protect their loyalty",
            Self::Tempo => "Trade efficiently and use tempo to stay a step ahead",
            Self::Theft => "Steal opponents' creatures, spells, and permanents",
            Self::Tokens => "Generate creature tokens to go wide and fuel payoffs",
            Self::Toolbox => "Tutor up the right answer from a versatile card pool",
            Self::ToughnessMatters => "Reward high toughness and defensive bodies",
            Self::Toxic => "Deal poison counters in combat with the toxic keyword",
            Self::Treasure => "Make Treasure tokens for ramp and artifact synergies",
            Self::Tribal => "Build around a single creature type and its lords",
            Self::Turbo => "Accelerate your game plan to deploy it ahead of schedule",
            Self::TurboFog => "Fog every combat and draw cards until you win late",
            Self::Untap => "Untap permanents to reuse abilities and generate value",
            Self::Vehicles => "Crew vehicles for efficient, removal-resistant attackers",
            Self::Voltron => "Pile buffs on one creature to win with commander damage",
            Self::Wheels => "Force everyone to discard and draw fresh hands",
            Self::Wipe => "Reset the board with mass removal, then rebuild first",
            Self::Zoo => "Flood the board with efficient, aggressive multicolor creatures",
        }
    }
}

/// Display as snake_case (matches serde serialization).
impl fmt::Display for DeckTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = serde_json::to_string(self).unwrap_or_default();
        // serde_json wraps in quotes, strip them.
        write!(f, "{}", s.trim_matches('"'))
    }
}

/// Error when parsing an unrecognized deck tag string.
#[derive(Debug, thiserror::Error)]
#[error("unknown deck tag: {0}")]
pub struct InvalidDeckTag(pub String);

impl TryFrom<&str> for DeckTag {
    type Error = InvalidDeckTag;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        serde_json::from_str(&format!("\"{value}\"")).map_err(|_| InvalidDeckTag(value.to_string()))
    }
}

/// Parses raw tag strings into validated tags, dropping duplicates while
/// preserving order. Returns an error on the first unrecognized string. Callers
/// enforce [`MAX_DECK_TAGS`] separately so they can surface their own error.
pub fn parse_tags(raw: &[String]) -> Result<Vec<DeckTag>, InvalidDeckTag> {
    let mut out: Vec<DeckTag> = Vec::with_capacity(raw.len());
    for s in raw {
        let tag = DeckTag::try_from(s.as_str())?;
        if !out.contains(&tag) {
            out.push(tag);
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_round_trip() {
        for tag in DeckTag::all() {
            let json = serde_json::to_string(tag).unwrap();
            let parsed: DeckTag = serde_json::from_str(&json).unwrap();
            assert_eq!(*tag, parsed);
        }
    }

    #[test]
    fn display_matches_serde() {
        assert_eq!(DeckTag::GroupHug.to_string(), "group_hug");
        assert_eq!(DeckTag::Aggro.to_string(), "aggro");
        assert_eq!(DeckTag::AttackTriggers.to_string(), "attack_triggers");
        assert_eq!(DeckTag::Etb.to_string(), "etb");
    }

    #[test]
    fn try_from_valid() {
        assert_eq!(DeckTag::try_from("group_hug").unwrap(), DeckTag::GroupHug);
        assert_eq!(DeckTag::try_from("clone").unwrap(), DeckTag::Clone);
        assert_eq!(DeckTag::try_from("draw").unwrap(), DeckTag::Draw);
    }

    #[test]
    fn try_from_invalid() {
        assert!(DeckTag::try_from("not_a_tag").is_err());
    }

    #[test]
    fn display_name_non_empty() {
        for tag in DeckTag::all() {
            assert!(!tag.display_name().is_empty());
        }
    }

    #[test]
    fn description_non_empty() {
        for tag in DeckTag::all() {
            assert!(!tag.description().is_empty());
        }
    }

    #[test]
    fn all_variants_have_unique_display_names() {
        let names: std::collections::HashSet<_> =
            DeckTag::all().iter().map(|t| t.display_name()).collect();
        assert_eq!(names.len(), DeckTag::all().len());
    }
}
