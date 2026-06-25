//! Short reminder text for MTG keyword abilities.
//!
//! Scryfall gives a card's keyword names in `scryfall_data.keywords`
//! (e.g. `["Flying", "Trample"]`). This module maps those names to a brief,
//! plain-language reminder of what each one does so newer players can read a
//! card without looking rules up. Pure and dependency-free.

/// Returns a short reminder for a keyword ability, or `None` if unknown.
///
/// Matching is case-insensitive and tolerant of trailing reminder detail
/// (e.g. `"Ward {2}"` matches `Ward`, `"Protection from red"` matches
/// `Protection`). Land-type walks (`Islandwalk`, `Swampwalk`, …) share one
/// generic reminder.
pub fn keyword_reminder(name: &str) -> Option<&'static str> {
    let key = name.trim().to_ascii_lowercase();

    // Land-type evasion: islandwalk, swampwalk, forestwalk, plainswalk, …
    if key.ends_with("walk") {
        return Some(
            "Can't be blocked as long as the defending player controls a land of the named type.",
        );
    }

    let reminder = match key.as_str() {
        // Evergreen
        "flying" => "Can only be blocked by creatures with flying or reach.",
        "first strike" => "Deals combat damage before creatures without first strike.",
        "double strike" => "Deals combat damage twice: once with first strike, once normally.",
        "deathtouch" => "Any amount of damage it deals to a creature is enough to destroy it.",
        "lifelink" => "Damage it deals also gains you that much life.",
        "vigilance" => "Doesn't tap when attacking.",
        "trample" => "Excess combat damage can be dealt to the defending player.",
        "haste" => "Can attack and use tap abilities the turn it comes under your control.",
        "reach" => "Can block creatures with flying.",
        "menace" => "Can only be blocked by two or more creatures.",
        "defender" => "Can't attack.",
        "flash" => "You can cast it any time you could cast an instant.",
        "hexproof" => "Can't be targeted by spells or abilities your opponents control.",
        "shroud" => "Can't be the target of any spells or abilities.",
        "indestructible" => "Can't be destroyed by damage or by effects that say destroy.",
        "ward" => {
            "When an opponent targets it, that spell or ability is countered unless they pay the ward cost."
        }
        "protection" => {
            "Can't be blocked, targeted, dealt damage, enchanted, or equipped by the named quality."
        }
        "prowess" => "Whenever you cast a noncreature spell, it gets +1/+1 until end of turn.",
        "equip" => "Pay the cost at sorcery speed to attach this Equipment to a creature you control.",
        "enchant" => "This Aura can only be attached to the kind of permanent it names.",

        // Deciduous and popular
        "scry" => {
            "Look at the top cards of your library; put any on the bottom and the rest back on top."
        }
        "surveil" => {
            "Look at the top cards of your library; put any into your graveyard and the rest back on top."
        }
        "flashback" => "You may cast it from your graveyard for its flashback cost, then it's exiled.",
        "cycling" => "Pay the cycling cost and discard it to draw a card.",
        "kicker" => "You may pay an additional cost as you cast it for a bonus effect.",
        "multikicker" => "You may pay its kicker cost any number of times as you cast it.",
        "convoke" => "You can tap creatures to help pay for this spell.",
        "improvise" => "You can tap artifacts to help pay for this spell.",
        "delve" => "You can exile cards from your graveyard to pay for generic mana.",
        "affinity" => "Costs less to cast for each of the named permanent you control.",
        "cascade" => {
            "When cast, exile cards until you reveal a cheaper nonland card, then cast it for free."
        }
        "storm" => "When cast, copy it for each spell cast before it this turn.",
        "suspend" => "Exile it with time counters; remove one each upkeep, then cast it free at zero.",
        "rebound" => "If cast from hand, exile it instead and cast it free on your next upkeep.",
        "madness" => "If discarded, you may cast it for its madness cost instead of discarding it.",
        "dash" => "Cast it for its dash cost to give it haste; return it to hand at end of turn.",
        "riot" => "Enters with your choice of a +1/+1 counter or haste.",
        "exalted" => "Whenever a creature attacks alone, it gets +1/+1 until end of turn.",
        "extort" => "Whenever you cast a spell, you may pay W/B to drain each opponent for 1 life.",
        "infect" => "Deals damage to players as poison counters and to creatures as -1/-1 counters.",
        "annihilator" => {
            "Whenever it attacks, the defending player sacrifices that many permanents."
        }
        "undying" => "When it dies, return it with a +1/+1 counter if it had none.",
        "persist" => "When it dies, return it with a -1/-1 counter if it had none.",
        "skulk" => "Can't be blocked by creatures with greater power.",
        "changeling" => "Is every creature type at all times.",
        "morph" => "May be cast face down as a 2/2; turn it face up any time for its morph cost.",
        "crew" => "Tap creatures with enough total power to turn this Vehicle into a creature.",
        "embalm" => "Pay the cost to exile it from your graveyard and make a token copy.",
        "eternalize" => "Pay the cost to exile it from your graveyard for a 4/4 black token copy.",
        "goad" => "Forces the creature to attack a player other than you on its controller's turn.",
        "partner" => "You may have two commanders if both have partner.",
        "overload" => "You may pay the overload cost to affect everything instead of one target.",
        "bushido" => "Whenever it blocks or is blocked, it gets +N/+N until end of turn.",
        "soulbond" => "Pair it with another creature; while paired, both gain a bonus ability.",
        "monstrosity" => "Pay its cost to put +1/+1 counters on it and make it monstrous.",
        "unleash" => "May enter with a +1/+1 counter; if it has one, it can't block.",

        _ => return None,
    };

    Some(reminder)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_keyword_returns_reminder() {
        assert!(keyword_reminder("Flying").is_some());
        assert!(keyword_reminder("Trample").is_some());
    }

    #[test]
    fn matching_is_case_insensitive() {
        assert_eq!(keyword_reminder("flying"), keyword_reminder("FLYING"));
    }

    #[test]
    fn landwalk_variants_share_a_reminder() {
        let island = keyword_reminder("Islandwalk");
        assert!(island.is_some());
        assert_eq!(island, keyword_reminder("Swampwalk"));
    }

    #[test]
    fn unknown_keyword_returns_none() {
        assert_eq!(keyword_reminder("Definitely Not A Keyword"), None);
    }
}
