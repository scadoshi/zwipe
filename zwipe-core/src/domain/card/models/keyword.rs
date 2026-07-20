//! Short reminder text for MTG keyword abilities.
//!
//! Scryfall gives a card's keyword names in `scryfall_data.keywords`
//! (e.g. `["Flying", "Trample"]`). This module maps those names to a brief,
//! plain-language reminder of what each one does so newer players can read a
//! card without looking rules up. Pure and dependency-free.
//!
//! Coverage spans Scryfall's three keyword catalogs: keyword abilities,
//! keyword actions (Mill, Scry, Proliferate, …), and ability words (Landfall,
//! Delirium, …). Reminders use `N` where a card supplies a number. Every
//! keyword resolves to a hint: known ones get a real reminder, and anything
//! else gets a friendly catch-all so no chip is ever a dead end.

/// Returns a short, plain-language reminder for a keyword.
///
/// Matching is case-insensitive and tolerant of trailing reminder detail
/// (e.g. `"Ward {2}"` matches `Ward`, `"Protection from red"` matches
/// `Protection`). Land-type walks (`Islandwalk`, …) and type cycling
/// (`Landcycling`, …) share one generic reminder each. Unrecognized keywords
/// fall back to a generic note rather than returning nothing.
pub fn keyword_reminder(name: &str) -> &'static str {
    let key = name.trim().to_ascii_lowercase();

    // Land-type evasion: islandwalk, swampwalk, legendary landwalk, …
    if key.ends_with("walk") {
        return "Can't be blocked as long as the defending player controls a land of the named type.";
    }

    // Type cycling: landcycling, wizardcycling, basic landcycling, …
    // (plain "Cycling" is handled in the match below.)
    if key.ends_with("cycling") && key != "cycling" {
        return "Pay the cost and discard it to search your library for a card of the named type, then shuffle.";
    }

    match key.as_str() {
        // --- Evergreen & core combat ---
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
        "hexproof from" => {
            "Can't be targeted by spells or abilities of the named quality your opponents control."
        }
        "shroud" => "Can't be the target of any spells or abilities.",
        "indestructible" => "Can't be destroyed by damage or by effects that say destroy.",
        "ward" => {
            "When an opponent targets it, that spell or ability is countered unless they pay the ward cost."
        }
        "protection" => {
            "Can't be blocked, targeted, dealt damage, enchanted, or equipped by the named quality."
        }
        "prowess" => "Whenever you cast a noncreature spell, it gets +1/+1 until end of turn.",
        "equip" => {
            "Pay the cost at sorcery speed to attach this Equipment to a creature you control."
        }
        "reconfigure" => {
            "Pay the cost to attach this to a creature, or unattach it; on its own it's a creature."
        }
        "enchant" => "This Aura can only be attached to the kind of permanent it names.",
        "fortify" => "Pay the cost to attach this Fortification to a land you control.",

        // --- Evasion ---
        "fear" => "Can only be blocked by artifact creatures and black creatures.",
        "intimidate" => {
            "Can only be blocked by artifact creatures and creatures that share a color with it."
        }
        "horsemanship" => "Can only be blocked by creatures with horsemanship.",
        "shadow" => "Can only block or be blocked by creatures with shadow.",
        "skulk" => "Can't be blocked by creatures with greater power.",
        "changeling" => "Is every creature type at all times.",

        // --- Combat triggers & modifiers ---
        "rampage" => {
            "Whenever it becomes blocked, it gets +N/+N for each creature blocking it beyond the first."
        }
        "bushido" => "When it blocks or becomes blocked, it gets +N/+N until end of turn.",
        "flanking" => {
            "Whenever a creature without flanking blocks it, that creature gets -1/-1 until end of turn."
        }
        "exalted" => {
            "Whenever a creature you control attacks alone, it gets +1/+1 until end of turn."
        }
        "battle cry" => {
            "Whenever it attacks, each other attacking creature gets +1/+0 until end of turn."
        }
        "melee" => {
            "Whenever it attacks, it gets +1/+1 until end of turn for each opponent you attacked."
        }
        "frenzy" => "Whenever it attacks and isn't blocked, it gets +N/+0 until end of turn.",
        "afflict" => "Whenever it becomes blocked, the defending player loses N life.",
        "mentor" => {
            "Whenever it attacks, put a +1/+1 counter on an attacking creature with lesser power."
        }
        "dethrone" => {
            "Whenever it attacks the player with the most life, put a +1/+1 counter on it."
        }
        "myriad" => {
            "When it attacks, create a token copy attacking each other opponent, then exile them after combat."
        }
        "annihilator" => {
            "Whenever it attacks, the defending player sacrifices that many permanents."
        }
        "training" => {
            "Whenever it attacks with a creature of greater power, put a +1/+1 counter on it."
        }
        "enlist" => {
            "As it attacks, you may tap a non-attacking creature to add its power to this one."
        }
        "mobilize" => {
            "Whenever it attacks, create that many tapped, attacking 1/1 red Warriors, then sacrifice them after combat."
        }
        "boast" => {
            "You may activate its boast ability once each turn, only if it attacked this turn."
        }

        // --- +1/+1 counters & growth ---
        "modular" => {
            "Enters with N +1/+1 counters; when it dies, move them to another artifact creature."
        }
        "graft" => {
            "Enters with N +1/+1 counters; when another creature enters, you may move a counter onto it."
        }
        "evolve" => {
            "Whenever a creature with greater power or toughness enters under your control, put a +1/+1 counter on it."
        }
        "bloodthirst" => {
            "If an opponent was dealt damage this turn, it enters with N +1/+1 counters."
        }
        "devour" => {
            "As it enters, you may sacrifice creatures; it enters with N +1/+1 counters for each one."
        }
        "fabricate" => {
            "When it enters, put N +1/+1 counters on it or create that many 1/1 Servo tokens."
        }
        "outlast" => "Tap it and pay the cost, only as a sorcery, to put a +1/+1 counter on it.",
        "renown" => {
            "When it deals combat damage to a player, if it isn't renowned, put N +1/+1 counters on it and it becomes renowned."
        }
        "unleash" => "May enter with a +1/+1 counter; if it has one, it can't block.",
        "reinforce" => "Discard it and pay its cost to put N +1/+1 counters on a creature.",
        "riot" => "Enters with your choice of a +1/+1 counter or haste.",
        "backup" => {
            "When it enters, put N +1/+1 counters on a creature; if it's another creature, it also gains this one's abilities."
        }
        "undying" => "When it dies, return it with a +1/+1 counter if it had none.",
        "persist" => "When it dies, return it with a -1/-1 counter if it had none.",

        // --- Damage as counters / poison ---
        "wither" => "Deals damage to creatures as -1/-1 counters.",
        "infect" => {
            "Deals damage to players as poison counters and to creatures as -1/-1 counters."
        }
        "toxic" => "Combat damage it deals to players also gives them N poison counters.",
        "poisonous" => "Whenever it deals combat damage to a player, they get N poison counters.",
        "absorb" => "If a source would deal damage to it, prevent N of that damage.",
        "afterlife" => "When it dies, create N 1/1 white and black Spirit tokens with flying.",

        // --- Alternative & additional costs ---
        "cycling" => "Pay the cycling cost and discard it to draw a card.",
        "kicker" => "You may pay an additional cost as you cast it for a bonus effect.",
        "multikicker" => "You may pay its kicker cost any number of times as you cast it.",
        "convoke" => "You can tap creatures to help pay for this spell.",
        "improvise" => "You can tap artifacts to help pay for this spell.",
        "delve" => "You can exile cards from your graveyard to pay for generic mana.",
        "affinity" => "Costs less to cast for each of the named permanent you control.",
        "undaunted" => "Costs {1} less to cast for each opponent.",
        "assist" => "Another player may help pay the colored cost of this spell.",
        "offering" => {
            "Cast it any time you could cast an instant by sacrificing a creature of the named type to reduce its cost."
        }
        "entwine" => "Pay the entwine cost to choose all of its modes instead of one.",
        "escalate" => "Pay the escalate cost for each mode you choose beyond the first.",
        "spree" => "Choose one or more extra modes as you cast it, paying for each.",
        "strive" => "Costs more for each target beyond the first.",
        "buyback" => {
            "Pay the buyback cost as you cast it to return it to your hand as it resolves."
        }
        "replicate" => {
            "Pay the replicate cost any number of times to copy the spell that many times."
        }
        "conspire" => {
            "As you cast it, tap two untapped creatures that share a color with it to copy it."
        }
        "splice" => {
            "As you cast an Arcane spell, reveal this from hand and pay its splice cost to add its text."
        }
        "surge" => "Pay the surge cost if you or a teammate has cast another spell this turn.",
        "spectacle" => "Pay the spectacle cost instead if an opponent lost life this turn.",
        "prowl" => {
            "Pay the prowl cost if you dealt combat damage with a matching creature type this turn."
        }
        "freerunning" => {
            "Pay the freerunning cost if an Assassin or your commander dealt combat damage this turn."
        }
        "casualty" => {
            "As you cast it, you may sacrifice a creature with high enough power to copy it."
        }
        "cleave" => "Pay the cleave cost to cast it without the text in brackets.",
        "compleated" => {
            "If you pay 2 life instead of Phyrexian mana, it enters with fewer loyalty counters."
        }
        "sunburst" => {
            "Enters with a +1/+1 or charge counter for each color of mana spent to cast it."
        }
        "converge" => "Counts the number of colors of mana spent to cast it.",
        "more than meets the eye" => "You may cast its converted side for its alternative cost.",
        "prototype" => "You may cast it as a smaller, cheaper, differently colored creature.",
        "offspring" => "Pay an extra cost as you cast it to also create a 1/1 token copy.",
        "squad" => "Pay the squad cost any number of times to create that many token copies.",
        "bargain" => {
            "As you cast it, you may sacrifice an artifact, enchantment, or token for a bonus."
        }

        // --- Cast for less / cast and sacrifice ---
        "evoke" => "Cast it for its evoke cost, then sacrifice it when it enters.",
        "emerge" => "Cast it by sacrificing a creature and paying the rest of the cost.",
        "dash" => "Cast it for its dash cost to give it haste; return it to hand at end of turn.",
        "blitz" => {
            "Cast it for its blitz cost: it gains haste and 'when it dies, draw a card', but is sacrificed at end of turn."
        }
        "mutate" => {
            "Cast it onto a non-Human creature you own; they merge into one creature keeping all abilities."
        }

        // --- Graveyard & recursion ---
        "flashback" => {
            "You may cast it from your graveyard for its flashback cost, then it's exiled."
        }
        "jump-start" => "Cast it from your graveyard by also discarding a card, then exile it.",
        "retrace" => "You may cast it from your graveyard by also discarding a land card.",
        "aftermath" => "Cast this half only from your graveyard, then exile the card.",
        "escape" => {
            "Cast it from your graveyard for its escape cost and by exiling other cards from your graveyard."
        }
        "disturb" => {
            "Cast its back face from your graveyard for the disturb cost, then exile it if it would leave."
        }
        "embalm" => "Pay the cost to exile it from your graveyard and make a token copy.",
        "eternalize" => "Pay the cost to exile it from your graveyard for a 4/4 black token copy.",
        "encore" => {
            "Pay the cost to exile it from your graveyard, making a token copy that attacks each opponent, then sacrifice them."
        }
        "unearth" => {
            "Pay the unearth cost to return it from your graveyard with haste; exile it at end of turn."
        }
        "scavenge" => {
            "Exile it from your graveyard and pay the cost to put that many +1/+1 counters on a creature."
        }
        "recover" => {
            "When a creature dies, pay the recover cost to return this from your graveyard to hand, or exile it."
        }
        "dredge" => {
            "Instead of drawing, you may mill N cards and return this from your graveyard to your hand."
        }
        "soulshift" => {
            "When it dies, you may return a Spirit with low enough mana value from your graveyard to your hand."
        }
        "haunt" => {
            "When it dies, exile it haunting a creature; its ability triggers again when that creature dies."
        }
        "madness" => "If discarded, you may cast it for its madness cost instead of discarding it.",
        "gravestorm" => "Copy this spell for each permanent put into a graveyard this turn.",

        // --- Copy / storm / timing ---
        "storm" => "When cast, copy it for each spell cast before it this turn.",
        "cascade" => {
            "When cast, exile cards until you reveal a cheaper nonland card, then cast it for free."
        }
        "ripple" => {
            "As you cast it, you may reveal the top N cards and cast any with the same name for free."
        }
        "epic" => {
            "Copy this spell at each of your upkeeps; you can't cast other spells for the rest of the game."
        }
        "suspend" => {
            "Exile it with time counters; remove one each upkeep, then cast it free at zero."
        }
        "rebound" => "If cast from hand, exile it instead and cast it free on your next upkeep.",
        "miracle" => {
            "If it's the first card you drew this turn, you may reveal it and cast it for its miracle cost."
        }
        "split second" => {
            "While it's on the stack, players can't cast spells or activate non-mana abilities."
        }
        "foretell" => "Pay {2} to exile it face down; cast it later for its foretell cost.",
        "plot" => "Pay its plot cost to exile it face up; cast it for free on a later turn.",
        "impending" => {
            "Cast it for its impending cost; it enters with N time counters and isn't a creature until they're gone."
        }

        // --- Phasing / vanishing ---
        "phasing" => {
            "It phases out and back each turn; while phased out it's treated as not existing."
        }
        "vanishing" => {
            "Enters with N time counters; remove one each upkeep, and sacrifice it when the last is gone."
        }
        "fading" => {
            "Enters with N fade counters; remove one each upkeep, and sacrifice it when you can't."
        }
        "cumulative upkeep" => {
            "At each upkeep, put an age counter on it, then pay its cost per counter or sacrifice it."
        }
        "echo" => "Pay its echo cost at your next upkeep or sacrifice it.",

        // --- Face-down ---
        "morph" => {
            "Cast it face down as a 2/2 for {3}; turn it face up any time for its morph cost."
        }
        "megamorph" => {
            "Cast it face down as a 2/2; turn it up for its megamorph cost with a +1/+1 counter."
        }
        "disguise" => {
            "Cast it face down as a 2/2 with ward {2}; turn it up any time for its disguise cost."
        }
        "hideaway" => {
            "When it enters, look at the top N cards, exile one face down, and put the rest on the bottom; play it later."
        }

        // --- Counters & flavor mechanics ---
        "level up" => {
            "Pay the cost as a sorcery to add a level counter, raising its stats and abilities."
        }
        "forecast" => {
            "Activate its forecast ability from your hand during your main phase, once each turn."
        }
        "aura swap" => "Pay the cost to exchange this Aura with an Aura in your hand.",
        "champion" => {
            "When it enters, exile another creature you control until it leaves; sacrifice it if you don't."
        }
        "exploit" => "When it enters, you may sacrifice a creature for an effect.",
        "extort" => "Whenever you cast a spell, you may pay W/B to drain each opponent for 1 life.",
        "tribute" => {
            "As it enters, an opponent chooses to put N +1/+1 counters on it or let its other ability trigger."
        }
        "cipher" => {
            "When it deals combat damage to a player, you may encode it on a creature so its controller can copy the spell."
        }
        "provoke" => {
            "When it attacks, you may force a creature the defender controls to untap and block it."
        }
        "ninjutsu" => {
            "Return an unblocked attacker to hand and pay the cost to put this onto the battlefield attacking."
        }
        "commander ninjutsu" => "Like ninjutsu, but you may also use it from the command zone.",
        "decayed" => "Can't block; sacrifice it after it attacks.",
        "living weapon" => {
            "When it enters, create a 0/0 Phyrexian Germ token and attach this Equipment to it."
        }
        "for mirrodin!" => {
            "When it enters, create a 2/2 red Rebel and attach this Equipment to it."
        }
        "living metal" => "During your turn, this Vehicle is also an artifact creature.",
        "craft" => "Pay the craft cost and exile other permanents or cards to transform it.",
        "specialize" => {
            "Pay the specialize cost to turn it into a single-color version with extra abilities."
        }
        "saddle" => {
            "Tap creatures with total power N or more to saddle this Mount; saddled abilities work that turn."
        }
        "crew" => "Tap creatures with enough total power to turn this Vehicle into a creature.",
        "read ahead" => "Enters as a Saga with any chapter number you choose.",
        "daybound" => "It's tied to day; it becomes its night side when night falls.",
        "nightbound" => "It's tied to night; it becomes its day side when day comes.",
        "exhaust" => "Each of its exhaust abilities can be activated only once.",

        // --- Commander / partner ---
        "partner" => "You may have two commanders if both have partner.",
        "partner with" => {
            "When it enters you may search for the named card; the two can be partner commanders."
        }
        "friends forever" => "You may have two commanders if both have friends forever.",
        "choose a background" => "You may have a Background enchantment as a second commander.",
        "doctor's companion" => "You may have a Time Lord Doctor as a second commander.",
        "companion" => {
            "If your deck meets its condition, once per game you may pay {3} to put it into your hand from outside the game."
        }

        // --- Other abilities ---
        "overload" => {
            "You may pay the overload cost to affect everything that qualifies instead of one target."
        }
        "bestow" => {
            "Cast it as an Aura for its bestow cost; if the creature leaves, it stays as a creature."
        }
        "awaken" => {
            "Pay the awaken cost to also put +1/+1 counters on a land and turn it into a creature."
        }
        "soulbond" => {
            "Pair it with another creature when either enters; while paired, both gain a bonus ability."
        }
        "amplify" => {
            "As it enters, reveal creature cards of the matching type from hand to put that many +1/+1 counters on it."
        }
        "ravenous" => {
            "It enters with X +1/+1 counters; if X is 5 or more, draw a card when it enters."
        }
        "banding" => {
            "It can attack or block in a band, and you choose how blocked creatures assign their combat damage."
        }
        "ingest" => {
            "Whenever it deals combat damage to a player, that player exiles the top card of their library."
        }
        "devoid" => "It has no color.",
        "demonstrate" => {
            "When you cast it, you may copy it, then an opponent of your choice also copies it."
        }
        "gift" => {
            "As you cast it, you may promise an opponent a gift; if you do, you gain a bonus."
        }
        "umbra armor" => {
            "When the enchanted creature would be destroyed, instead remove all damage from it and destroy this Aura."
        }
        "ascend" => {
            "If you control ten or more permanents, you get the city's blessing for the rest of the game."
        }
        "fuse" => "You may cast both halves of this split card together from your hand.",
        "transmute" => {
            "Discard it and pay its transmute cost to search your library for a card with the same mana value."
        }
        "transfigure" => {
            "Sacrifice it and pay its cost to search your library for a creature with the same mana value."
        }

        // --- Keyword actions (useful shorthand) ---
        "regenerate" => {
            "The next time it would be destroyed this turn, instead tap it, remove it from combat, and heal it."
        }
        "counter" => {
            "Counter a spell or ability so it doesn't resolve and is put into its owner's graveyard."
        }
        "seek" => {
            "Search your library at random for a card matching the description and put it into your hand, without revealing your library."
        }
        "scry" => {
            "Look at the top N cards of your library; put any on the bottom and the rest back on top."
        }
        "surveil" => {
            "Look at the top N cards of your library; put any into your graveyard and the rest back on top."
        }
        "mill" => "Put the top N cards of your library into your graveyard.",
        "fateseal" => "Look at the top N cards of an opponent's library; put any on the bottom.",
        "proliferate" => {
            "Choose any permanents or players with a counter and give each another counter of a kind it already has."
        }
        "explore" => {
            "Reveal the top card: if a land, put it in hand; otherwise put a +1/+1 counter on the creature and keep or bin the card."
        }
        "investigate" => "Create a Clue token; sacrifice it and pay {2} to draw a card.",
        "populate" => "Create a token copy of a creature token you control.",
        "amass" => {
            "Put N +1/+1 counters on an Army you control, creating a 0/0 Army first if you have none."
        }
        "connive" => {
            "Draw N cards, then discard N; put a +1/+1 counter on it for each nonland card discarded."
        }
        "goad" => {
            "The creature must attack each combat if able, and a player other than you if able."
        }
        "manifest" => {
            "Put the top card face down as a 2/2; turn it face up for its cost if it's a creature card."
        }
        "manifest dread" => {
            "Look at the top two cards, manifest one and put the other into your graveyard."
        }
        "cloak" => {
            "Put a card face down as a 2/2 with ward {2}; turn it up any time for its mana cost."
        }
        "bolster" => "Put N +1/+1 counters on the creature you control with the least toughness.",
        "support" => "Put a +1/+1 counter on each of up to N other target creatures.",
        "adapt" => "If it has no +1/+1 counters, put N +1/+1 counters on it.",
        "monstrosity" => {
            "If it isn't monstrous, put N +1/+1 counters on it and it becomes monstrous."
        }
        "detain" => {
            "Until your next turn, that permanent can't attack, block, or have its abilities activated."
        }
        "incubate" => {
            "Create an Incubator token with N +1/+1 counters that can flip into a 0/0 Phyrexian artifact creature."
        }
        "fight" => "Two creatures each deal damage equal to their power to the other.",
        "clash" => {
            "You and an opponent reveal your top card; the one with the higher cost wins, and you may bottom yours."
        }
        "meld" => "Combine two specific cards into a single larger permanent.",
        "exert" => "You may exert it as it attacks; it won't untap next turn but gets a bonus.",
        "learn" => "Draw and discard a card, or get a Lesson card from outside the game.",
        "venture into the dungeon" => {
            "Enter or advance through a dungeon, taking the next room's effect."
        }
        "discover" => {
            "Exile cards from the top until a nonland with mana value N or less, then cast it free or put it in hand."
        }
        "conjure" => "Create the named card directly, usually into your hand.",
        "role token" => "Create a Role, an Aura token that grants the named bonus to a creature.",
        "suspect" => "A suspected creature has menace and can't block.",
        "collect evidence" => {
            "Exile cards with total mana value N or more from your graveyard as a cost."
        }
        "forage" => "Exile three cards from your graveyard, or sacrifice a Food, as a cost.",
        "endure" => "Put N +1/+1 counters on it, or create an N/N white Spirit Army token.",
        "time travel" => {
            "Add or remove a time counter on each suspended card and Saga you control."
        }
        "behold" => {
            "Reveal or note a creature of the named type from your hand or battlefield as a cost."
        }

        // --- Ability words (stable triggers / thresholds) ---
        "landfall" => "Whenever a land enters under your control, this ability triggers.",
        "constellation" => "Whenever an enchantment enters under your control, this triggers.",
        "magecraft" => "Whenever you cast or copy an instant or sorcery spell, this triggers.",
        "revolt" => "Active if a permanent left your control this turn.",
        "metalcraft" => "Active if you control three or more artifacts.",
        "delirium" => "Active if four or more card types are among cards in your graveyard.",
        "threshold" => "Active if seven or more cards are in your graveyard.",
        "spell mastery" => {
            "Active if two or more instant and/or sorcery cards are in your graveyard."
        }
        "undergrowth" => "Counts the number of creature cards in your graveyard.",
        "morbid" => "Active if a creature died this turn.",
        "raid" => "Active if you attacked with a creature this turn.",
        "battalion" => "Triggers when it and at least two other creatures attack.",
        "bloodrush" => "Discard it for its bloodrush cost to give an attacking creature a bonus.",
        "heroic" => "Triggers whenever you cast a spell that targets this creature.",
        "ferocious" => "Active if you control a creature with power 4 or greater.",
        "formidable" => "Active if creatures you control have total power 8 or greater.",
        "domain" => "Scales with the number of basic land types among lands you control.",
        "imprint" => "It exiles a card whose qualities its other abilities then use.",
        "hellbent" => "Active if you have no cards in hand.",
        "fateful hour" => "Active if you have 5 or less life.",
        "grandeur" => "Discard another card with the same name to activate its ability.",
        "inspired" => "Triggers whenever it becomes untapped.",
        "kinship" => {
            "At your upkeep you may reveal the top card; get a bonus if it shares a creature type with it."
        }
        "radiance" => "Affects the target and every other permanent that shares a color with it.",
        "rally" => "Triggers whenever a creature enters under your control.",
        "alliance" => "Triggers whenever another creature enters under your control.",
        "sweep" => {
            "Return any number of the named lands you control for an effect that scales with how many."
        }
        "tempting offer" => {
            "You get an effect; each opponent may let you do it again to also copy it for themselves."
        }
        "will of the council" => "Each player votes, and the option with the most votes happens.",
        "council's dilemma" => "Each player votes among options, shaping a combined effect.",
        "join forces" => "Starting with you, each player may pay mana to increase the effect.",
        "parley" => "Each player reveals their top card; the effect scales with what's revealed.",
        "addendum" => "Get a bonus if you cast it during your main phase.",
        "adamant" => "Get a bonus if at least three mana of the same color were spent to cast it.",
        "coven" => "Active if you control three or more creatures with different powers.",
        "pack tactics" => "Active if you attacked with creatures of total power 6 or greater.",
        "corrupted" => "Active if an opponent has three or more poison counters.",
        "descend" => "Cares about permanent cards in your graveyard, or whether one entered it.",
        "fathomless descent" => "Scales with the number of permanent cards in your graveyard.",
        "celebration" => {
            "Active if two or more nonland permanents entered under your control this turn."
        }
        "valiant" => {
            "Triggers the first time each turn it becomes the target of a spell or ability you control."
        }
        "flurry" => "Triggers whenever you cast your second spell each turn.",
        "eerie" => {
            "Triggers when an enchantment enters or a face-down permanent turns up under your control."
        }
        "lieutenant" => "Gets a bonus as long as you control your commander.",
        "cohort" => "Tap it and an untapped Ally you control to activate its ability.",
        "channel" => "Discard it from your hand to use its channel ability.",
        "chroma" => "Counts colored mana symbols among the relevant cards or permanents.",
        "survival" => "Triggers at the start of your second main phase if it's tapped.",
        "eminence" => "Its eminence ability works even while it's in the command zone.",
        "enrage" => "Triggers whenever it's dealt damage.",
        "void" => {
            "Active if a nonland permanent left the battlefield or a spell or ability was countered this turn."
        }
        "will of the planeswalkers" => "Players vote, and the option with the most votes happens.",

        // --- Token & game actions ---
        "treasure" => {
            "Create a Treasure token: an artifact you can sacrifice for one mana of any color."
        }
        "food" => {
            "Create a Food token: an artifact you can sacrifice, paying {2} and tapping it, to gain 3 life."
        }
        "create" => "Put the named token onto the battlefield.",
        "destroy" => "Move the permanent to its owner's graveyard.",
        "exile" => "Put it into exile, out of the game unless something returns it.",
        "sacrifice" => "Put a permanent you control into its owner's graveyard.",
        "discard" => "Put a card from your hand into your graveyard.",
        "cast" => "Put a spell onto the stack by paying its cost.",
        "play" => "Cast a spell or put a land onto the battlefield.",
        "activate" => "Use an activated ability by paying the cost listed before its colon.",
        "attach" => "Move an Aura, Equipment, or Fortification onto the chosen permanent.",
        "tap" => "Turn it sideways, usually to attack, pay a cost, or use an ability.",
        "untap" => "Return it upright so it can be used again.",
        "reveal" => "Show a card to all players; it stays where it is.",
        "shuffle" => "Randomize the order of your library.",
        "exchange" => "Swap two things, such as control of permanents or life totals.",
        "vote" => "Each player votes among the options to decide the effect.",
        "transform" => "Flip a double-faced permanent to its other side.",
        "convert" => "Turn a double-faced card to its other face.",
        "double" => "Double the named amount, such as counters, life, or mana.",
        "triple" => "Triple the named amount.",

        // --- Un-set & supplemental mechanics ---
        "assemble" => {
            "Put a Contraption from your Contraption deck into play (an Un-set mechanic)."
        }
        "augment" => "Combine this with a host creature to change it (an Un-set mechanic).",
        "double agenda" => "Secretly name two cards at the start of the game (an Un-set mechanic).",
        "hidden agenda" => "Secretly name a card at the start of the game (an Un-set mechanic).",
        "open an attraction" => {
            "Put an Attraction from your Attraction deck onto the battlefield (an Unfinity mechanic)."
        }
        "roll to visit your attractions" => {
            "Roll a six-sided die to see which of your Attractions trigger (an Unfinity mechanic)."
        }
        "set in motion" => {
            "Put a scheme card into motion to start its effect (an Archenemy mechanic)."
        }
        "abandon" => "Remove an ongoing scheme from the game (an Archenemy mechanic).",

        // --- Recent set mechanics ---
        "warp" => {
            "Cast it for its cheaper warp cost to use it now, then exile it; you can cast it from exile later."
        }
        "station" => {
            "Tap creatures to add station counters to it; it unlocks stronger abilities at certain thresholds."
        }
        "max speed" => {
            "Its bonus is active once you reach max speed (your Speed has climbed to 4)."
        }
        "start your engines!" => {
            "You get Speed 1; your Speed can rise to a maximum of 4, powering up Speed abilities."
        }
        "double team" => {
            "When it attacks, if it isn't a token, make a copy of it in your hand; then both lose double team."
        }
        "harmonize" => {
            "You may cast it again from your graveyard for its harmonize cost (a Final Fantasy mechanic)."
        }
        "mayhem" => {
            "If it was discarded, you may cast it from your graveyard for its mayhem cost, then exile it."
        }
        "job select" => {
            "Choose a job for it as it enters, granting matching abilities (a Final Fantasy mechanic)."
        }
        "web-slinging" => {
            "Cast it for its web-slinging cost by returning or tapping a creature (a Spider-Man mechanic)."
        }
        "firebending" => {
            "Whenever it attacks, add N red mana; any you don't spend is lost when combat ends."
        }
        "waterbend" => {
            "A cost that includes mana; for each generic mana in it, you may tap an untapped artifact or creature you control instead of paying it."
        }
        "airbend" => {
            "Exile target creature; its owner may cast it later for {2} instead of its mana cost."
        }
        "earthbend" => {
            "Turn target land you control into a 0/0 creature with haste that's still a land, then put N +1/+1 counters on it; if it dies, it returns tapped."
        }

        // Any other keyword still gets a friendly, honest catch-all so every chip
        // is tappable. Scryfall only ever feeds us real keyword names here.
        _ => {
            "A special keyword from this card's set. Check the card's text to see exactly what it does."
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FALLBACK: &str = "A special keyword from this card's set. Check the card's text to see exactly what it does.";

    #[test]
    fn known_keywords_get_specific_reminders() {
        for kw in ["Flying", "Trample", "Mill", "Foretell", "Treasure"] {
            assert_ne!(
                keyword_reminder(kw),
                FALLBACK,
                "{kw} should have a real hint"
            );
        }
    }

    #[test]
    fn matching_is_case_insensitive() {
        assert_eq!(keyword_reminder("flying"), keyword_reminder("FLYING"));
    }

    #[test]
    fn landwalk_variants_share_a_reminder() {
        let island = keyword_reminder("Islandwalk");
        assert_ne!(island, FALLBACK);
        assert_eq!(island, keyword_reminder("Swampwalk"));
    }

    #[test]
    fn typecycling_variants_share_a_reminder_but_plain_cycling_differs() {
        let landcycling = keyword_reminder("Landcycling");
        assert_ne!(landcycling, FALLBACK);
        assert_eq!(landcycling, keyword_reminder("Wizardcycling"));
        assert_ne!(keyword_reminder("Cycling"), FALLBACK);
        assert_ne!(keyword_reminder("Cycling"), landcycling);
    }

    #[test]
    fn unknown_keyword_gets_the_fallback_hint() {
        assert_eq!(keyword_reminder("Definitely Not A Keyword"), FALLBACK);
    }
}
