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
    Adventures,
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
    Chaos,
    Cheerios,
    Clone,
    Clues,
    CoinFlips,
    Colorless,
    Combo,
    CommanderMatters,
    Control,
    CostReduction,
    Counters,
    Counterspells,
    Curses,
    Cycling,
    Deathtouch,
    Defenders,
    Devotion,
    DiceRolling,
    Discard,
    Domain,
    Draw,
    Dungeons,
    Enchantments,
    Enchantress,
    Energy,
    Equipment,
    Etb,
    ExtraCombats,
    ExtraTurns,
    Fight,
    Flash,
    Flying,
    Food,
    GlassCannon,
    Goad,
    GoodStuff,
    GoWide,
    Graveyard,
    GroupHug,
    GroupSlug,
    Hatebears,
    Infect,
    Kicker,
    LandDestruction,
    Landfall,
    Lands,
    LegendsMatter,
    Lifedrain,
    Lifegain,
    Madness,
    Midrange,
    Mill,
    Monarch,
    Morph,
    Mounts,
    Multicolor,
    Mutate,
    Outlaws,
    Party,
    Pillowfort,
    Ping,
    Plot,
    Poison,
    Politics,
    PowerMatters,
    Powerstones,
    Prison,
    Proliferate,
    Ramp,
    Reanimator,
    Removal,
    Roles,
    Rooms,
    Sacrifice,
    Sagas,
    SelfMill,
    Shrines,
    Snow,
    Speed,
    Spellslinger,
    Stax,
    Stompy,
    Storm,
    Superfriends,
    Surveil,
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
    Werewolves,
    Wheels,
    Wipe,
    XSpells,
    Zoo,
}

impl DeckTag {
    /// All deck tag variants, alphabetical.
    pub fn all() -> &'static [DeckTag] {
        &[
            Self::Adventures,
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
            Self::Chaos,
            Self::Cheerios,
            Self::Clone,
            Self::Clues,
            Self::CoinFlips,
            Self::Colorless,
            Self::Combo,
            Self::CommanderMatters,
            Self::Control,
            Self::CostReduction,
            Self::Counters,
            Self::Counterspells,
            Self::Curses,
            Self::Cycling,
            Self::Deathtouch,
            Self::Defenders,
            Self::Devotion,
            Self::DiceRolling,
            Self::Discard,
            Self::Domain,
            Self::Draw,
            Self::Dungeons,
            Self::Enchantments,
            Self::Enchantress,
            Self::Energy,
            Self::Equipment,
            Self::Etb,
            Self::ExtraCombats,
            Self::ExtraTurns,
            Self::Fight,
            Self::Flash,
            Self::Flying,
            Self::Food,
            Self::GlassCannon,
            Self::Goad,
            Self::GoodStuff,
            Self::GoWide,
            Self::Graveyard,
            Self::GroupHug,
            Self::GroupSlug,
            Self::Hatebears,
            Self::Infect,
            Self::Kicker,
            Self::LandDestruction,
            Self::Landfall,
            Self::Lands,
            Self::LegendsMatter,
            Self::Lifedrain,
            Self::Lifegain,
            Self::Madness,
            Self::Midrange,
            Self::Mill,
            Self::Monarch,
            Self::Morph,
            Self::Mounts,
            Self::Multicolor,
            Self::Mutate,
            Self::Outlaws,
            Self::Party,
            Self::Pillowfort,
            Self::Ping,
            Self::Plot,
            Self::Poison,
            Self::Politics,
            Self::PowerMatters,
            Self::Powerstones,
            Self::Prison,
            Self::Proliferate,
            Self::Ramp,
            Self::Reanimator,
            Self::Removal,
            Self::Roles,
            Self::Rooms,
            Self::Sacrifice,
            Self::Sagas,
            Self::SelfMill,
            Self::Shrines,
            Self::Snow,
            Self::Speed,
            Self::Spellslinger,
            Self::Stax,
            Self::Stompy,
            Self::Storm,
            Self::Superfriends,
            Self::Surveil,
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
            Self::Werewolves,
            Self::Wheels,
            Self::Wipe,
            Self::XSpells,
            Self::Zoo,
        ]
    }

    /// Human-readable display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Adventures => "Adventures",
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
            Self::Chaos => "Chaos",
            Self::Cheerios => "Cheerios",
            Self::Clone => "Clone",
            Self::Clues => "Clues",
            Self::CoinFlips => "Coin Flips",
            Self::Colorless => "Colorless",
            Self::Combo => "Combo",
            Self::CommanderMatters => "Commander Matters",
            Self::Control => "Control",
            Self::CostReduction => "Cost Reduction",
            Self::Counters => "+1/+1 Counters",
            Self::Counterspells => "Counterspells",
            Self::Curses => "Curses",
            Self::Cycling => "Cycling",
            Self::Deathtouch => "Deathtouch",
            Self::Defenders => "Defenders",
            Self::Devotion => "Devotion",
            Self::DiceRolling => "Dice Rolling",
            Self::Discard => "Discard",
            Self::Domain => "Domain",
            Self::Draw => "Draw",
            Self::Dungeons => "Dungeons",
            Self::Enchantments => "Enchantments",
            Self::Enchantress => "Enchantress",
            Self::Energy => "Energy",
            Self::Equipment => "Equipment",
            Self::Etb => "ETB",
            Self::ExtraCombats => "Extra Combats",
            Self::ExtraTurns => "Extra Turns",
            Self::Fight => "Fight",
            Self::Flash => "Flash",
            Self::Flying => "Flying",
            Self::Food => "Food",
            Self::GlassCannon => "Glass Cannon",
            Self::Goad => "Goad",
            Self::GoodStuff => "Good Stuff",
            Self::GoWide => "Go Wide",
            Self::Graveyard => "Graveyard",
            Self::GroupHug => "Group Hug",
            Self::GroupSlug => "Group Slug",
            Self::Hatebears => "Hatebears",
            Self::Infect => "Infect",
            Self::Kicker => "Kicker",
            Self::LandDestruction => "Land Destruction",
            Self::Landfall => "Landfall",
            Self::Lands => "Lands Matter",
            Self::LegendsMatter => "Legends Matter",
            Self::Lifedrain => "Lifedrain",
            Self::Lifegain => "Lifegain",
            Self::Madness => "Madness",
            Self::Midrange => "Midrange",
            Self::Mill => "Mill",
            Self::Monarch => "Monarch",
            Self::Morph => "Morph",
            Self::Mounts => "Mounts",
            Self::Multicolor => "Multicolor",
            Self::Mutate => "Mutate",
            Self::Outlaws => "Outlaws",
            Self::Party => "Party",
            Self::Pillowfort => "Pillowfort",
            Self::Ping => "Pingers",
            Self::Plot => "Plot",
            Self::Poison => "Poison",
            Self::Politics => "Politics",
            Self::PowerMatters => "Power Matters",
            Self::Powerstones => "Powerstones",
            Self::Prison => "Prison",
            Self::Proliferate => "Proliferate",
            Self::Ramp => "Ramp",
            Self::Reanimator => "Reanimator",
            Self::Removal => "Removal",
            Self::Roles => "Roles",
            Self::Rooms => "Rooms",
            Self::Sacrifice => "Sacrifice",
            Self::Sagas => "Sagas",
            Self::SelfMill => "Self-Mill",
            Self::Shrines => "Shrines",
            Self::Snow => "Snow",
            Self::Speed => "Speed",
            Self::Spellslinger => "Spellslinger",
            Self::Stax => "Stax",
            Self::Stompy => "Stompy",
            Self::Storm => "Storm",
            Self::Superfriends => "Superfriends",
            Self::Surveil => "Surveil",
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
            Self::Werewolves => "Werewolves",
            Self::Wheels => "Wheels",
            Self::Wipe => "Board Wipes",
            Self::XSpells => "X Spells",
            Self::Zoo => "Zoo",
        }
    }

    /// One-line, plain-language definition of the strategy, for the tag picker's
    /// hint dialog.
    pub fn description(&self) -> &'static str {
        match self {
            Self::Adventures => {
                "Cast creatures' adventure halves first, then the creature later, for two-for-one value"
            }
            Self::Aggro => {
                "Apply early pressure with cheap creatures to win before opponents stabilize"
            }
            Self::Aikido => "Turn opponents' own attacks and resources against them",
            Self::AlternateWincon => "Win through a card's alternate victory condition",
            Self::Aristocrats => {
                "Sacrifice your own creatures for value and incremental life drain"
            }
            Self::Artifacts => "Build the engine around artifacts and their synergies",
            Self::AttackTriggers => "Reward attacking with triggers that snowball each combat",
            Self::Auras => "Stack enchantment auras onto a creature to enlarge it",
            Self::BigMana => "Generate huge mana to power out oversized threats and X spells",
            Self::Blink => {
                "Flicker creatures out and back to reuse their enter-the-battlefield effects"
            }
            Self::Blood => "Make Blood tokens to loot away cards and fuel payoffs",
            Self::Bounce => "Return permanents to hand to reset boards and reuse effects",
            Self::Burn => "Deal direct damage with spells to creatures or players",
            Self::Cascade => "Chain free spells off cascade and cost-reduction effects",
            Self::Chaos => "Warp the game with random, symmetric effects that hit the whole table",
            Self::Cheerios => "Chain zero-cost artifacts to draw into a combo finish",
            Self::Clone => "Copy the best creatures and permanents on the battlefield",
            Self::Clues => "Investigate for Clue tokens to draw and trigger payoffs",
            Self::CoinFlips => "Flip coins and lean on coin-flip payoffs to win",
            Self::Colorless => {
                "Lean on colorless and devoid permanents with colorless-matters payoffs"
            }
            Self::Combo => "Assemble two or more cards into a game-ending interaction",
            Self::CommanderMatters => "Reward casting and building around your commander",
            Self::Control => "Counter and remove threats, then win with late-game inevitability",
            Self::CostReduction => "Slash spell costs to deploy threats far ahead of curve",
            Self::Counters => "Grow creatures with +1/+1 counters and counter payoffs",
            Self::Counterspells => "Hold up counterspells to stop key spells before they resolve",
            Self::Curses => "Saddle opponents with Curse enchantments that punish them over time",
            Self::Cycling => "Discard cards to draw and trigger cycling payoffs",
            Self::Deathtouch => "Pair deathtouch with pingers and forced blocks to trade up",
            Self::Defenders => "Hold the line with high-toughness walls, then turn them into a win",
            Self::Devotion => "Reward heavy colored pips with devotion-scaling effects",
            Self::DiceRolling => "Roll dice and reward high rolls with d20 payoffs",
            Self::Discard => "Strip opponents' hands to deny resources and options",
            Self::Domain => "Reward many basic land types with domain-scaling effects",
            Self::Draw => "Refill your hand with extra card draw to outresource the table",
            Self::Dungeons => "Venture through dungeons and take the initiative for staged payoffs",
            Self::Enchantments => "Build the engine around enchantments and their synergies",
            Self::Enchantress => "Draw cards whenever you cast enchantments",
            Self::Energy => "Accumulate energy counters and spend them for value",
            Self::Equipment => "Suit up creatures with equipment for repeatable buffs",
            Self::Etb => "Chain enter-the-battlefield effects for repeatable value",
            Self::ExtraCombats => "Take additional combat steps to attack again and again",
            Self::ExtraTurns => "Chain extra turns to pull ahead and close the game",
            Self::Fight => "Bite opponents' creatures by fighting them with your bigger ones",
            Self::Flash => "Hold up instants and flash threats to play on your opponents' turns",
            Self::Flying => "Win in the air with evasive flying creatures",
            Self::Food => "Make Food tokens to gain life and feed sacrifice value",
            Self::GlassCannon => "Win fast with little defense if the plan is disrupted",
            Self::Goad => {
                "Force opponents' creatures to attack each other and profit from the chaos"
            }
            Self::GoodStuff => "Play the strongest standalone cards over tight synergy",
            Self::GoWide => "Flood the board with many creatures, then buff them all at once",
            Self::Graveyard => "Use the graveyard as a resource to recur and reuse cards",
            Self::GroupHug => "Give every player resources to steer politics and the game",
            Self::GroupSlug => "Punish the whole table with symmetric damage and taxes",
            Self::Hatebears => "Disrupt opponents with small creatures that tax and deny",
            Self::Infect => "Attack with infect creatures, dealing damage as poison counters",
            Self::Kicker => "Pay kicker and multikicker for bigger, scalable spell effects",
            Self::LandDestruction => "Destroy opponents' lands to deny mana and tempo",
            Self::Landfall => "Trigger payoffs each time a land enters the battlefield",
            Self::Lands => "Treat lands as the engine, ramping and recurring them for value",
            Self::LegendsMatter => "Flood the board with legendary permanents and their payoffs",
            Self::Lifedrain => "Drain opponents' life while padding your own total",
            Self::Lifegain => "Gain life and turn that life total into payoffs",
            Self::Madness => "Discard cards to cast them for their cheaper madness cost",
            Self::Midrange => "Play efficient threats and removal to grind out value",
            Self::Mill => "Empty opponents' libraries to deck them out",
            Self::Monarch => "Become the monarch for an extra card each turn",
            Self::Morph => "Play creatures face-down and flip them with morph and disguise",
            Self::Mounts => "Saddle Mounts to turn them on for combat payoffs",
            Self::Multicolor => "Reward playing many colors and gold cards",
            Self::Mutate => "Stack mutating creatures into one body that piles up triggers",
            Self::Outlaws => {
                "Commit crimes with Assassins, Mercenaries, Pirates, Rogues, and Warlocks"
            }
            Self::Party => {
                "Assemble a full party of Cleric, Rogue, Warrior, and Wizard for payoffs"
            }
            Self::Pillowfort => "Defend yourself with deterrents so attacks go elsewhere",
            Self::Ping => "Repeatedly deal 1 damage to pick off creatures and players",
            Self::Plot => "Plot spells on earlier turns to cast them for free later",
            Self::Poison => "Win through poison counters from infect and toxic sources",
            Self::Politics => "Make deals and incentives to steer the multiplayer table",
            Self::PowerMatters => "Reward high power and pumping your creatures' power",
            Self::Powerstones => "Make Powerstone tokens to ramp into artifacts and big spells",
            Self::Prison => "Lock opponents out with continuous resource-denial effects",
            Self::Proliferate => "Multiply every kind of existing counter with proliferate",
            Self::Ramp => "Accelerate mana to cast big spells ahead of schedule",
            Self::Reanimator => "Cheat large creatures from the graveyard into play",
            Self::Removal => "Lean on efficient spot removal to answer any threat",
            Self::Roles => "Hand out Role tokens to buff your creatures or shrink theirs",
            Self::Rooms => "Build around Room enchantments and unlock both doors for value",
            Self::Sacrifice => "Use sacrifice outlets to turn permanents into value",
            Self::Sagas => "Build around saga enchantments and their chapter effects",
            Self::SelfMill => "Mill your own library to fuel graveyard payoffs",
            Self::Shrines => "Stack Shrine enchantments that scale with each Shrine you control",
            Self::Snow => "Build around snow permanents and their scaling payoffs",
            Self::Speed => "Prioritize a fast clock to win before opponents set up",
            Self::Spellslinger => "Cast many instants and sorceries with spell payoffs",
            Self::Stax => "Slow the whole table with symmetric resource-denial pieces",
            Self::Stompy => "Ramp into big, efficient creatures and swing",
            Self::Storm => "Cast a chain of spells to power up storm finishers",
            Self::Superfriends => "Deploy many planeswalkers and protect their loyalty",
            Self::Surveil => "Surveil to fill the graveyard while smoothing your draws",
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
            Self::Werewolves => "Flip Werewolves with daybound and nightbound to swing big",
            Self::Wheels => "Force everyone to discard and draw fresh hands",
            Self::Wipe => "Reset the board with mass removal, then rebuild first",
            Self::XSpells => "Pour mana into X spells that scale into the late game",
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
