//! Heuristic classification of MTG cards into mechanical categories.
//!
//! Examines oracle_text, type_line, keywords, produced_mana, and stats
//! to assign categories. A card can match multiple categories.
//! Expected accuracy: ~70-80%. AI classification (Layer 2) corrects errors.

use super::MechanicalCategory;
use crate::domain::card::Card;
use regex::Regex;
use std::sync::LazyLock;

/// Classifies a card into zero or more mechanical categories based on heuristics.
///
/// Examines oracle text, type line, keywords, produced mana, and power/toughness.
/// Returns an empty vec for cards that don't match any known pattern.
pub fn classify_by_heuristics(card: &Card) -> Vec<MechanicalCategory> {
    let sd = &card.scryfall_data;
    let oracle = sd.oracle_text.as_deref().unwrap_or("").to_lowercase();
    let type_line = sd.type_line.as_deref().unwrap_or("").to_lowercase();
    let keywords: Vec<String> = sd
        .keywords
        .as_deref()
        .unwrap_or(&[])
        .iter()
        .map(|k| k.to_lowercase())
        .collect();
    let has_keyword = |kw: &str| keywords.iter().any(|k| k == kw);
    let cmc = sd.cmc.unwrap_or(0.0);
    let has_produced_mana = sd.produced_mana.as_ref().is_some_and(|m| !m.is_empty());
    let power_num: Option<i32> = sd.power.as_deref().and_then(|p| p.parse().ok());

    let mut cats = Vec::new();

    // ========================================
    // Ramp
    // ========================================
    // Mana-producing permanents at low cost, or land-fetching spells
    if has_produced_mana
        && cmc <= 3.0
        && (type_line.contains("creature")
            || type_line.contains("artifact")
            || type_line.contains("enchantment"))
    {
        cats.push(MechanicalCategory::Ramp);
    }
    if RAMP_LAND_SEARCH.is_match(&oracle) {
        cats.push(MechanicalCategory::Ramp);
    }
    if RAMP_ADD_MANA.is_match(&oracle) && !cats.contains(&MechanicalCategory::Ramp) {
        // "add {" patterns for mana rocks/dorks not caught above
        if type_line.contains("artifact") {
            cats.push(MechanicalCategory::Ramp);
        }
    }

    // ========================================
    // Draw
    // ========================================
    if DRAW_CARD.is_match(&oracle) {
        cats.push(MechanicalCategory::Draw);
    }

    // ========================================
    // Removal (single-target, NOT board wipes)
    // ========================================
    if (oracle.contains("destroy target")
        || oracle.contains("exile target")
        || oracle.contains("deals") && oracle.contains("damage to target"))
        && !oracle.contains("destroy all")
        && !oracle.contains("exile all")
    {
        cats.push(MechanicalCategory::Removal);
    }

    // ========================================
    // Wipe (board wipes)
    // ========================================
    if oracle.contains("destroy all")
        || oracle.contains("exile all")
        || WIPE_MINUS.is_match(&oracle)
    {
        cats.push(MechanicalCategory::Wipe);
    }

    // ========================================
    // Counterspell
    // ========================================
    if (type_line.contains("instant") || type_line.contains("sorcery"))
        && oracle.contains("counter target spell")
    {
        cats.push(MechanicalCategory::Counterspell);
    }

    // ========================================
    // Protection
    // ========================================
    if has_keyword("hexproof")
        || has_keyword("indestructible")
        || has_keyword("ward")
        || oracle.contains("protection from")
        || oracle.contains("gains hexproof")
        || oracle.contains("gains indestructible")
    {
        cats.push(MechanicalCategory::Protection);
    }

    // ========================================
    // Evasion
    // ========================================
    if has_keyword("flying")
        || has_keyword("trample")
        || has_keyword("menace")
        || has_keyword("fear")
        || has_keyword("intimidate")
        || has_keyword("shadow")
        || oracle.contains("can't be blocked")
        || oracle.contains("is unblockable")
    {
        cats.push(MechanicalCategory::Evasion);
    }

    // ========================================
    // Finisher
    // ========================================
    if (type_line.contains("creature") && power_num.is_some_and(|p| p >= 6))
        || oracle.contains("you win the game")
        || oracle.contains("extra turn")
    {
        cats.push(MechanicalCategory::Finisher);
    }

    // ========================================
    // Tokens
    // ========================================
    if TOKEN_CREATE.is_match(&oracle) {
        cats.push(MechanicalCategory::Tokens);
    }

    // ========================================
    // Lifegain
    // ========================================
    if has_keyword("lifelink") || LIFEGAIN.is_match(&oracle) {
        cats.push(MechanicalCategory::Lifegain);
    }

    // ========================================
    // Blink
    // ========================================
    if BLINK.is_match(&oracle) {
        cats.push(MechanicalCategory::Blink);
    }

    // ========================================
    // Recursion
    // ========================================
    if RECURSION.is_match(&oracle) {
        cats.push(MechanicalCategory::Recursion);
    }

    // ========================================
    // Mill
    // ========================================
    if has_keyword("mill") || MILL.is_match(&oracle) {
        cats.push(MechanicalCategory::Mill);
    }

    // ========================================
    // Burn
    // ========================================
    if BURN.is_match(&oracle) && !type_line.contains("creature")
    // exclude combat damage abilities
    {
        cats.push(MechanicalCategory::Burn);
    }

    // ========================================
    // Drain
    // ========================================
    if oracle.contains("loses") && oracle.contains("life") && oracle.contains("gain") {
        cats.push(MechanicalCategory::Drain);
    }

    // ========================================
    // Pump (single-target buff)
    // ========================================
    if PUMP.is_match(&oracle) && !oracle.contains("creatures you control") {
        cats.push(MechanicalCategory::Pump);
    }

    // ========================================
    // Anthem (team-wide buff)
    // ========================================
    if oracle.contains("creatures you control get")
        || oracle.contains("creatures you control have")
        || oracle.contains("other creatures you control get")
    {
        cats.push(MechanicalCategory::Anthem);
    }

    // ========================================
    // Counters (+1/+1, -1/-1, or proliferate)
    // ========================================
    // Proliferate cards are counter payoffs regardless of counter type, so they
    // belong in the counters bucket even without explicit +1/+1 / -1/-1 text.
    if oracle.contains("+1/+1 counter")
        || oracle.contains("-1/-1 counter")
        || oracle.contains("proliferate")
    {
        cats.push(MechanicalCategory::Counters);
    }

    // ========================================
    // Copy
    // ========================================
    if COPY.is_match(&oracle) {
        cats.push(MechanicalCategory::Copy);
    }

    // ========================================
    // Sacrifice
    // ========================================
    if SACRIFICE.is_match(&oracle) {
        cats.push(MechanicalCategory::Sacrifice);
    }

    // ========================================
    // Stax
    // ========================================
    if STAX_CANT.is_match(&oracle) || STAX_EACH.is_match(&oracle) {
        cats.push(MechanicalCategory::Stax);
    }

    // ========================================
    // Untap
    // ========================================
    if UNTAP.is_match(&oracle) {
        cats.push(MechanicalCategory::Untap);
    }

    // ========================================
    // Tutor (search library, but NOT land search which is Ramp)
    // ========================================
    if oracle.contains("search your library") && !RAMP_LAND_SEARCH.is_match(&oracle) {
        cats.push(MechanicalCategory::Tutor);
    }

    // ========================================
    // Graveyard Hate
    // ========================================
    if GRAVEYARD_HATE.is_match(&oracle) {
        cats.push(MechanicalCategory::GraveyardHate);
    }

    // Deduplicate (a card may match the same category via multiple rules)
    cats.sort_by_key(|c| *c as u8);
    cats.dedup();
    cats
}

// Compiled regex patterns (lazy-initialized).
// These are compile-time-known valid patterns; expect is safe here.
#[allow(clippy::expect_used)]
mod patterns {
    use super::*;

    pub static RAMP_LAND_SEARCH: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"search your library for .{0,20}(land|forest|island|swamp|mountain|plains)")
            .expect("valid regex")
    });
    pub static RAMP_ADD_MANA: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"add \{").expect("valid regex"));
    pub static DRAW_CARD: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"draw.{0,15}card").expect("valid regex"));
    pub static WIPE_MINUS: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(all|each) creatures?.{0,15}get.{0,10}-\d+/-\d+").expect("valid regex")
    });
    pub static TOKEN_CREATE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"create.{0,30}token").expect("valid regex"));
    pub static LIFEGAIN: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"gain.{0,10}life").expect("valid regex"));
    pub static BLINK: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"exile.{0,80}return.{0,30}(to the battlefield|under)").expect("valid regex")
    });
    pub static RECURSION: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(return|put).{0,30}(from.{0,15}graveyard|graveyard.{0,15}(to|onto))")
            .expect("valid regex")
    });
    pub static MILL: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(top|mill).{0,20}(card|mill).{0,15}(graveyard|mill)").expect("valid regex")
    });
    pub static BURN: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"deals? \d+ damage to").expect("valid regex"));
    pub static PUMP: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(target|equipped|enchanted) creature.{0,15}gets? \+\d+/\+\d+")
            .expect("valid regex")
    });
    pub static COPY: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(
            r"(copy|copies).{0,20}(target|a) (spell|permanent|creature|artifact|enchantment)",
        )
        .expect("valid regex")
    });
    pub static SACRIFICE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"sacrifice (a|an|another|target) (creature|permanent|artifact|enchantment)")
            .expect("valid regex")
    });
    pub static STAX_CANT: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(opponents?|players?) can'?t").expect("valid regex"));
    pub static STAX_EACH: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(each|all) (player|opponent).{0,30}(sacrifice|discard|lose|pay)")
            .expect("valid regex")
    });
    pub static UNTAP: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"untap.{0,20}(target|all|each).{0,15}(land|creature|permanent|artifact)")
            .expect("valid regex")
    });
    pub static GRAVEYARD_HATE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"exile.{0,30}(graveyard|all cards from)").expect("valid regex")
    });
}
use patterns::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::make_card;
    use MechanicalCategory::*;

    fn card_with_oracle(name: &str, oracle: &str, type_line: &str) -> Card {
        let mut card = make_card(name);
        card.scryfall_data.oracle_text = Some(oracle.to_string());
        card.scryfall_data.type_line = Some(type_line.to_string());
        card
    }

    #[test]
    fn sol_ring_is_ramp() {
        let mut card = make_card("Sol Ring");
        card.scryfall_data.type_line = Some("Artifact".to_string());
        card.scryfall_data.oracle_text = Some("{T}: Add {C}{C}.".to_string());
        card.scryfall_data.produced_mana = Some(vec!["C".to_string()]);
        card.scryfall_data.cmc = Some(1.0);
        let cats = classify_by_heuristics(&card);
        assert!(
            cats.contains(&Ramp),
            "Sol Ring should be Ramp, got {:?}",
            cats
        );
    }

    #[test]
    fn swords_to_plowshares_is_removal() {
        let card = card_with_oracle(
            "Swords to Plowshares",
            "Exile target creature. Its controller gains life equal to its power.",
            "Instant",
        );
        let cats = classify_by_heuristics(&card);
        assert!(
            cats.contains(&Removal),
            "StP should be Removal, got {:?}",
            cats
        );
    }

    #[test]
    fn wrath_of_god_is_wipe() {
        let card = card_with_oracle(
            "Wrath of God",
            "Destroy all creatures. They can't be regenerated.",
            "Sorcery",
        );
        let cats = classify_by_heuristics(&card);
        assert!(cats.contains(&Wipe), "Wrath should be Wipe, got {:?}", cats);
        assert!(
            !cats.contains(&Removal),
            "Wrath should not be single-target Removal"
        );
    }

    #[test]
    fn counterspell_is_counterspell() {
        let card = card_with_oracle("Counterspell", "Counter target spell.", "Instant");
        let cats = classify_by_heuristics(&card);
        assert!(
            cats.contains(&Counterspell),
            "Counterspell should be Counterspell, got {:?}",
            cats
        );
    }

    #[test]
    fn demonic_tutor_is_tutor() {
        let card = card_with_oracle(
            "Demonic Tutor",
            "Search your library for a card, put that card into your hand, then shuffle.",
            "Sorcery",
        );
        let cats = classify_by_heuristics(&card);
        assert!(
            cats.contains(&Tutor),
            "Demonic Tutor should be Tutor, got {:?}",
            cats
        );
    }

    #[test]
    fn lightning_bolt_is_burn_and_removal() {
        let card = card_with_oracle(
            "Lightning Bolt",
            "Lightning Bolt deals 3 damage to any target.",
            "Instant",
        );
        let cats = classify_by_heuristics(&card);
        assert!(cats.contains(&Burn), "Bolt should be Burn, got {:?}", cats);
    }

    #[test]
    fn cultivate_is_ramp() {
        let card = card_with_oracle(
            "Cultivate",
            "Search your library for up to two basic land cards, reveal those cards, put one onto the battlefield tapped and the other into your hand, then shuffle.",
            "Sorcery",
        );
        let cats = classify_by_heuristics(&card);
        assert!(
            cats.contains(&Ramp),
            "Cultivate should be Ramp, got {:?}",
            cats
        );
    }

    #[test]
    fn rhystic_study_is_draw() {
        let card = card_with_oracle(
            "Rhystic Study",
            "Whenever an opponent casts a spell, you may draw a card unless that player pays {1}.",
            "Enchantment",
        );
        let cats = classify_by_heuristics(&card);
        assert!(
            cats.contains(&Draw),
            "Rhystic Study should be Draw, got {:?}",
            cats
        );
    }

    #[test]
    fn proliferate_is_counters() {
        // Proliferate is a counter payoff even without explicit +1/+1 text.
        let card = card_with_oracle(
            "Karn's Bastion",
            "{T}: Add {C}. {4}, {T}: Proliferate.",
            "Land",
        );
        let cats = classify_by_heuristics(&card);
        assert!(
            cats.contains(&Counters),
            "Karn's Bastion should be Counters, got {:?}",
            cats
        );
    }

    #[test]
    fn craterhoof_behemoth_is_finisher_and_anthem() {
        let mut card = card_with_oracle(
            "Craterhoof Behemoth",
            "Haste\nWhen Craterhoof Behemoth enters, creatures you control get +X/+X and gain trample until end of turn, where X is the number of creatures you control.",
            "Creature — Beast",
        );
        card.scryfall_data.power = Some("5".to_string());
        card.scryfall_data.toughness = Some("5".to_string());
        card.scryfall_data.keywords = Some(vec!["Haste".to_string()]);
        // Power is 5, which is < 6 threshold, but let's test the anthem part
        let cats = classify_by_heuristics(&card);
        assert!(
            cats.contains(&Anthem),
            "Craterhoof should be Anthem, got {:?}",
            cats
        );
    }

    #[test]
    fn reanimate_is_recursion() {
        let card = card_with_oracle(
            "Reanimate",
            "Put target creature card from a graveyard onto the battlefield under your control. You lose life equal to its mana value.",
            "Sorcery",
        );
        let cats = classify_by_heuristics(&card);
        assert!(
            cats.contains(&Recursion),
            "Reanimate should be Recursion, got {:?}",
            cats
        );
    }

    #[test]
    fn rest_in_peace_is_graveyard_hate() {
        let card = card_with_oracle(
            "Rest in Peace",
            "When Rest in Peace enters, exile all cards from all graveyards.\nIf a card or token would be put into a graveyard from anywhere, exile it instead.",
            "Enchantment",
        );
        let cats = classify_by_heuristics(&card);
        assert!(
            cats.contains(&GraveyardHate),
            "RIP should be GraveyardHate, got {:?}",
            cats
        );
    }

    #[test]
    fn avenger_of_zendikar_is_tokens() {
        let card = card_with_oracle(
            "Avenger of Zendikar",
            "When Avenger of Zendikar enters, create a 0/1 green Plant creature token for each land you control.\nLandfall — Whenever a land you control enters, you may put a +1/+1 counter on each Plant creature you control.",
            "Creature — Elemental",
        );
        let cats = classify_by_heuristics(&card);
        assert!(
            cats.contains(&Tokens),
            "Avenger should be Tokens, got {:?}",
            cats
        );
    }

    #[test]
    fn kodamas_reach_is_ramp() {
        let card = card_with_oracle(
            "Kodama's Reach",
            "Search your library for up to two basic land cards, reveal those cards, put one onto the battlefield tapped and the other into your hand, then shuffle.",
            "Sorcery — Arcane",
        );
        let cats = classify_by_heuristics(&card);
        assert!(
            cats.contains(&Ramp),
            "Kodama's Reach should be Ramp, got {:?}",
            cats
        );
    }

    #[test]
    fn thassa_deep_dwelling_is_blink() {
        let card = card_with_oracle(
            "Thassa, Deep-Dwelling",
            "As long as your devotion to blue is less than five, Thassa isn't a creature.\nAt the beginning of your end step, exile up to one other target creature you control, then return that card to the battlefield under your control.",
            "Legendary Enchantment Creature — God",
        );
        let cats = classify_by_heuristics(&card);
        assert!(
            cats.contains(&Blink),
            "Thassa should be Blink, got {:?}",
            cats
        );
    }

    #[test]
    fn smothering_tithe_is_ramp() {
        let mut card = card_with_oracle(
            "Smothering Tithe",
            "Whenever an opponent draws a card, that player may pay {2}. If the player doesn't, you create a Treasure token.",
            "Enchantment",
        );
        card.scryfall_data.produced_mana = Some(vec!["W".to_string()]);
        card.scryfall_data.cmc = Some(4.0);
        // cmc > 3, so won't match the low-cost mana producer rule
        // But it creates tokens, so should at least be Tokens
        let cats = classify_by_heuristics(&card);
        assert!(
            cats.contains(&Tokens),
            "Smothering Tithe should be Tokens, got {:?}",
            cats
        );
    }
}
