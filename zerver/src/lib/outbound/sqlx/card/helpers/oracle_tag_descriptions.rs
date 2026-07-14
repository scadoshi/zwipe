//! Our own oracle-tag descriptions.
//!
//! Scryfall describes only ~29% of the ~4,500 oracle tags, and its
//! [`sync_oracle_tags`](super::oracle_tags::sync_oracle_tags) does a full
//! `DELETE` + re-`INSERT` of the catalog every run, so anything written into
//! `oracle_tags.description` out of band is wiped on the next sync. To make our
//! text durable, we author it here (compiled into the binary, shipped by a normal
//! deploy) and overlay it onto the catalog **inside the same sync transaction**,
//! right after the reinsert. Ours always wins: it replaces Scryfall's description
//! where we have one and fills the blanks where we don't. The end state is that
//! every tag is described by us; grow [`ORACLE_TAG_DESCRIPTIONS`] over time,
//! priority order = highest card population first.
//!
//! Because the column itself carries the merged text post-sync, every reader
//! (the `get_oracle_tags` catalog endpoint, the deck picker's definition bar, a
//! future tag dictionary) picks it up with no serve-time merge and no client
//! change. Slugs here that the fresh catalog doesn't carry are warned about in the
//! `zervice` log (a typo would otherwise match nothing silently).

/// Authored `slug -> description` map, overlaid onto `oracle_tags.description`
/// after each Scryfall sync. Descriptions are user-facing (shown in-app), so keep
/// them short, plain, and em-dash-free. Keyed by a real oracle-tag slug; the slug
/// side is warn-checked at sync time against the freshly loaded catalog.
pub const ORACLE_TAG_DESCRIPTIONS: &[(&str, &str)] = &[
    (
        "triggered-ability",
        "Cards with a triggered ability: an effect that happens on its own when a condition is met (worded when, whenever, or at).",
    ),
    (
        "attack-trigger",
        "Has an ability that triggers whenever a creature attacks.",
    ),
    (
        "removal-creature",
        "Removal aimed specifically at creatures.",
    ),
    (
        "repeatable-removal",
        "Removal you can use more than once, usually from a permanent's ability or recursion.",
    ),
    ("removal-destroy", "Removal that destroys its target."),
    ("burn-creature", "Deals direct damage to creatures."),
    ("repeatable-lifegain", "A repeatable source of life gain."),
    ("gains-pp-counters", "Gains +1/+1 counters."),
    ("opponent-loses-life", "Makes an opponent lose life."),
    (
        "sacrifice-outlet-creature",
        "Sacrifices creatures for an effect, often repeatably.",
    ),
    ("burn-player", "Deals direct damage to players."),
    (
        "mana-value-matters",
        "Cares about the mana value of cards or permanents.",
    ),
    (
        "toughness-boost-to-all",
        "Raises the toughness of your creatures.",
    ),
    ("synergy-artifact", "Rewards playing artifacts."),
    ("gives-haste", "Grants haste to a creature."),
    (
        "tapper-creature",
        "Taps down a creature so it can't attack or block.",
    ),
    (
        "utility-land",
        "A land with a useful ability beyond making mana.",
    ),
    ("synergy-instant", "Rewards playing instants."),
    (
        "synergy-sorcery",
        "Rewards casting sorceries, and often instants too.",
    ),
    ("gives-flying", "Grants flying to a creature."),
    ("gives-trample", "Grants trample to a creature."),
    (
        "discard-outlet",
        "Lets you discard cards for a benefit, often repeatably.",
    ),
    ("removal-exile", "Removal that exiles its target."),
    ("untapper-creature", "Untaps a target creature."),
    (
        "removal-nonland",
        "Removal that can hit any nonland permanent.",
    ),
    (
        "removal-sacrifice",
        "Removal that forces a player to sacrifice.",
    ),
    ("burn-planeswalker", "Deals direct damage to planeswalkers."),
    ("damage-prevention", "Prevents damage."),
    ("hate-attacker", "Punishes or hinders attacking creatures."),
    ("hate-blocker", "Punishes or hinders blocking creatures."),
    (
        "removal-bounce",
        "Removal that returns its target to its owner's hand.",
    ),
    ("copy-creature", "Copies a creature."),
    (
        "activated-ability",
        "Has an activated ability: an effect you turn on by paying a cost.",
    ),
    ("spot-removal", "Removal aimed at a single target."),
    (
        "single-target-instant-sorcery",
        "An instant or sorcery that usually aims at a single target.",
    ),
    (
        "evasion",
        "A creature that is hard to block, such as by flying or menace.",
    ),
    ("alliteration", "Has an alliterative name."),
    (
        "repeatable-crime",
        "Repeatably commits crimes by targeting opponents or their permanents.",
    ),
    (
        "unique-type-line",
        "The only card with its exact type line.",
    ),
    (
        "intervening-if-clause",
        "Has a trigger that only does something if a condition is true.",
    ),
    (
        "hand-neutral",
        "Replaces itself so your hand size stays the same.",
    ),
    (
        "repeatable-pp-counters",
        "Repeatably puts +1/+1 counters on creatures.",
    ),
    ("noncreature-typal", "Cares about creature types."),
    (
        "namesake-spell",
        "A spell named after a specific character.",
    ),
    ("attacking-matters-self", "Cares about itself attacking."),
    (
        "virtual-vanilla",
        "Plays like a vanilla creature once in play, aside from an enters or cast trigger.",
    ),
    (
        "french-vanilla",
        "A creature whose only abilities are keywords.",
    ),
    ("multiple-targets", "Can target more than one thing."),
    ("draw-engine", "Draws extra cards repeatedly over time."),
    (
        "repeatable-creature-tokens",
        "Repeatably makes creature tokens.",
    ),
    (
        "cheaper-than-mv",
        "Can often be cast or used for less than its mana value.",
    ),
    ("drawback", "Comes with a built-in downside."),
    (
        "repeatable-pure-draw",
        "Repeatably draws cards at no extra cost.",
    ),
    (
        "gives-pp-counters",
        "Puts +1/+1 counters on other creatures.",
    ),
    ("single-english-word-name", "Has a one-word English name."),
    (
        "virtual-french-vanilla",
        "Has only keyword abilities once in play, aside from an enters or cast trigger.",
    ),
    (
        "cast-trigger-you",
        "Triggers whenever you cast a certain kind of spell.",
    ),
    (
        "pure-draw",
        "Draws a card without discarding or sacrificing another to do it.",
    ),
    ("attacking-matters", "Cares about creatures attacking."),
    ("delayed-trigger", "Sets up an ability that triggers later."),
    (
        "hand-positive",
        "Card advantage that leaves you with more cards in hand.",
    ),
    (
        "more-expensive-than-mv",
        "Can cost more to cast or use than its mana value.",
    ),
    ("power-boost-to-all", "Raises the power of your creatures."),
    ("burn-any", "Deals direct damage to any target."),
    ("lifegain", "Gains you life."),
    ("exile-self", "Exiles itself."),
    (
        "bottomless-mana-sink",
        "A mana sink that can win the game if you pour enough mana in.",
    ),
    (
        "mana-sink",
        "A repeatable effect you can pour extra mana into each turn.",
    ),
    ("pinger", "Repeatably deals 1 or 2 damage."),
    ("symmetrical", "Affects all players equally."),
    (
        "sweeper",
        "Removal that wipes many or all permanents at once.",
    ),
    (
        "protects-creature",
        "Protects a creature, such as with hexproof or indestructible.",
    ),
    (
        "combat-trick",
        "An instant-speed effect that boosts your creature or weakens an enemy in combat.",
    ),
    (
        "death-trigger-self",
        "Has an ability that triggers when it dies.",
    ),
    (
        "group-slug",
        "Makes each opponent lose life or take damage.",
    ),
    ("cantrip", "Draws you a card when it resolves or enters."),
    (
        "multi-removal",
        "Removal that hits several targets at once, but not all.",
    ),
    (
        "removal-toughness",
        "Kills creatures by reducing their toughness to zero.",
    ),
    ("modal", "Lets you choose from two or more effects."),
    (
        "death-trigger",
        "Has an ability that triggers when a permanent dies.",
    ),
    ("burst-draw", "Draws several cards at once."),
    (
        "repeatable-sacrifice-outlet",
        "A repeatable way to sacrifice your own permanents.",
    ),
    (
        "type-addition-human",
        "A creature that gained the Human type on its type line to match what it always was.",
    ),
    (
        "tutor-to-hand",
        "Searches your library for a card and puts it into your hand.",
    ),
    (
        "ramp",
        "Increases the mana you have available this turn or in future turns.",
    ),
    (
        "creaturefall",
        "Triggers whenever a creature enters the battlefield.",
    ),
    (
        "discard",
        "Forces a player to discard cards from their hand.",
    ),
    (
        "saboteur",
        "Triggers when a creature deals combat damage to a player.",
    ),
    (
        "martyr",
        "A creature with an ability that sacrifices a creature for a benefit.",
    ),
    (
        "activate-from-hand",
        "Has an activated ability you can use straight from your hand.",
    ),
    (
        "reanimate-creature",
        "Returns a creature card from a graveyard straight to the battlefield.",
    ),
    ("adds-multiple-mana", "Adds more than one mana at once."),
    ("punny-name", "A card whose name is a pun or play on words."),
    (
        "potentially-black-border",
        "A joke or un-set style card that could still pass as a normal, tournament-legal card.",
    ),
    (
        "type-errata",
        "A card whose printed type line has since been officially updated.",
    ),
    (
        "enters-in-company",
        "A creature that brings extra creatures onto the battlefield when it enters, without reanimating them.",
    ),
    (
        "land-ramp",
        "Puts extra lands onto the battlefield for you.",
    ),
    (
        "keyword-anthem",
        "Grants a keyword ability to your creatures.",
    ),
    (
        "scry",
        "Lets you look at the top few cards of your library and put any on the bottom.",
    ),
    (
        "anthem",
        "A static effect that boosts your creatures' power and toughness.",
    ),
    (
        "mana-dork",
        "A creature that produces or helps pay for extra mana.",
    ),
    (
        "castable-from-exile",
        "Lets you cast a card straight from exile.",
    ),
    (
        "block-trigger",
        "Has an ability that triggers when it blocks or becomes blocked.",
    ),
    (
        "multiple-bodies",
        "Puts two or more creatures onto the battlefield at once.",
    ),
    (
        "out-of-color-token",
        "Creates a token in a color the card itself isn't.",
    ),
    (
        "free-cast-another",
        "Lets you cast another card without paying its mana cost.",
    ),
    (
        "cast-on-resolution",
        "Casts another card as one of its spells or abilities resolves.",
    ),
    (
        "castable-from-graveyard",
        "A card you can cast straight from your graveyard.",
    ),
    (
        "per-player",
        "Scales up based on how many players are in the game.",
    ),
    (
        "unique-token",
        "A specific token variant that only one card creates.",
    ),
    (
        "fun-ruling",
        "Has an official ruling where the rules team had some fun.",
    ),
    (
        "drain-life",
        "Makes a player lose life and gains you that much life.",
    ),
    (
        "self-replacement-effect",
        "An effect that partially or fully replaces its own resolution.",
    ),
    (
        "pp-counters-matter",
        "Cares about +1/+1 counters on creatures.",
    ),
    (
        "sacrifice-self",
        "Has an ability that makes you sacrifice this permanent.",
    ),
    (
        "life-payment",
        "Costs you life to cast, activate, or trigger its effect.",
    ),
    (
        "removal-artifact",
        "Removal aimed at destroying or exiling an artifact.",
    ),
    (
        "tutor-land-basic",
        "Searches your library for a basic land.",
    ),
    (
        "regrowth-creature",
        "Returns a creature card from your graveyard to your hand.",
    ),
    (
        "mini-refund",
        "Gives you a small burst of extra mana to spend soon.",
    ),
    (
        "offcolor-ability",
        "Has an ability with a mana cost outside the card's own colors.",
    ),
    (
        "mill-self",
        "Puts cards from the top of your own library into your graveyard.",
    ),
    (
        "unique-mana-cost",
        "Has a mana cost that no other card shares.",
    ),
    (
        "untracked-indefinite-effect",
        "Creates a lasting effect with nothing on the battlefield tracking it.",
    ),
    (
        "tap-fuel-creature",
        "Taps a creature you control to pay for or activate an effect.",
    ),
    ("gains-flying", "Gains flying for itself."),
    ("copy-self", "Makes a copy of itself."),
    (
        "reflexive-trigger",
        "A when-you-do ability that fires from an action taken while resolving another effect.",
    ),
    (
        "egg",
        "A cheap artifact meant to be sacrificed for a payoff.",
    ),
    (
        "digital-only-mechanics",
        "Uses a mechanic built for Magic Arena's digital-only environment.",
    ),
    (
        "aesthetic-counter",
        "Uses a custom, card-specific counter type rather than a standard one.",
    ),
    (
        "synergy-noncreature",
        "Cares about noncreature spells you cast, like prowess and cast triggers.",
    ),
    (
        "sacrifice-outlet-artifact",
        "Lets you sacrifice artifacts you control, often as a repeatable outlet.",
    ),
    (
        "removal-land",
        "Removal that destroys, exiles, or disrupts a target land.",
    ),
    (
        "power-matters",
        "Cares about the power of creatures, yours or others'.",
    ),
    (
        "hate-graveyard",
        "Disrupts graveyards by exiling or otherwise shutting off cards in them.",
    ),
    ("power-matters-self", "Cares about its own power."),
    (
        "bear-with-set-s-mechanic",
        "A small two-mana creature whose main feature is one of the set's signature mechanics.",
    ),
    (
        "front-card",
        "Marks the front face of a two-sided token or theme card, not a playable double-faced spell.",
    ),
    (
        "surveil",
        "Looks at cards off the top of your library and sends any you choose to the graveyard.",
    ),
    (
        "shade-pump",
        "Repeatedly pumps a creature's power and toughness until end of turn, usually for mana.",
    ),
    (
        "counters-matter",
        "Cares about counters, rewarding you for having or adding them.",
    ),
    (
        "firebreathing",
        "Repeatably pumps a creature's power with +X/+0 until end of turn for a cost.",
    ),
    (
        "gives-indestructible",
        "Grants indestructible to other permanents.",
    ),
    (
        "gives-first-strike",
        "Grants first strike to other creatures.",
    ),
    ("gives-vigilance", "Grants vigilance to other creatures."),
    ("life-for-cards", "Trades your own life for extra cards."),
    (
        "type-change",
        "Changes what type a permanent is, usually giving a creature a new creature type.",
    ),
    (
        "force-draw",
        "Forces a player to draw cards, whether they want to or not.",
    ),
    (
        "burn-with-set-s-mechanic",
        "A damage spell that also carries the signature mechanic of the set it debuted in.",
    ),
    (
        "gains-haste",
        "Grants itself haste, letting it attack or tap the turn it enters.",
    ),
    (
        "tutor-land-to-battlefield",
        "Searches your library for a land and puts it straight onto the battlefield.",
    ),
    (
        "reanimate-self",
        "Returns itself from your graveyard to the battlefield.",
    ),
    (
        "refund",
        "Gives you mana back to help pay for other spells.",
    ),
    (
        "gives-castable-from-exile",
        "Lets you cast a card from exile instead of your hand.",
    ),
    (
        "rhystic",
        "An effect opponents can pay mana to stop or ignore.",
    ),
    (
        "prevent-blocker",
        "Stops a creature from being able to block.",
    ),
    (
        "hate-set-mechanic",
        "Specifically counters a mechanic from the set it was printed in.",
    ),
    (
        "cast-trigger-self",
        "Triggers an ability the moment you cast it, before it resolves.",
    ),
    (
        "shapechange",
        "Sets a creature's power and toughness to specific values.",
    ),
    (
        "hand-negative",
        "Makes you discard one or more cards from your hand.",
    ),
    (
        "cda-power",
        "A creature whose power is calculated by a rule on the card, not a fixed number.",
    ),
    (
        "repeatable-card-advantage",
        "Gives you extra cards repeatedly over time, not just once.",
    ),
    (
        "pair-commander",
        "A commander meant to pair with a second commander, via Partner or a Background.",
    ),
    (
        "curiosity",
        "Draws you a card whenever it deals damage to a player.",
    ),
    (
        "repeatable-loot",
        "Lets you draw and discard cards repeatedly.",
    ),
    (
        "utility-mana-rock",
        "A mana rock with an extra useful ability, or a spell that makes one.",
    ),
    (
        "sacrifice-outlet-land",
        "Lets you sacrifice a land, often as a one-time cost for an effect.",
    ),
    (
        "theft-creature",
        "An effect that takes control of another player's creature.",
    ),
    (
        "virtual-legendary",
        "Represents a one-of-a-kind character or object even though it isn't printed as legendary.",
    ),
    (
        "type-addition-phyrexian",
        "Has the Phyrexian creature type added to it.",
    ),
    ("gives-lifelink", "Grants lifelink to other creatures."),
    (
        "upkeep-cost",
        "Makes you pay a cost during your upkeep or suffer a consequence.",
    ),
    (
        "unnoted-tracked-information",
        "Tracks game events for its effect without a visible counter or marker.",
    ),
    (
        "sweeper-one-sided",
        "A sweeper that tends to wipe out your opponents' permanents while sparing your own.",
    ),
    (
        "removal-enchantment",
        "Removal that can destroy or exile enchantments.",
    ),
    (
        "landfall",
        "Triggers an effect whenever a land enters the battlefield under your control.",
    ),
    (
        "protects-all",
        "Shields several of your creatures at once, such as with indestructible or a flicker.",
    ),
    (
        "synergy-white",
        "Rewards you for playing white cards or permanents.",
    ),
    (
        "temporary-token",
        "Creates a token that only sticks around for a limited time.",
    ),
    (
        "selective-group-hug",
        "A group hug effect that helps some opponents while leaving others out.",
    ),
    (
        "removal-permanent",
        "Removal that can destroy, exile, or bounce any type of permanent, not just creatures.",
    ),
    (
        "dnd-character",
        "Depicts a named Dungeons and Dragons character in its name or text.",
    ),
    (
        "manaless-value",
        "Gives you value without paying its normal mana cost.",
    ),
    (
        "discount-self",
        "Reduces its own casting cost under the right conditions.",
    ),
    (
        "deprecated-legend-type",
        "A creature that was printed with the old Legend creature type, before Legendary existed.",
    ),
    (
        "repeatable-treasures",
        "Repeatedly creates Treasure tokens you can sacrifice for mana.",
    ),
    ("unblockable", "A creature that can't be blocked."),
    (
        "theft-cast",
        "Lets you cast a spell from another player's hand, library, or graveyard.",
    ),
    (
        "enlarge",
        "Gives a creature a big boost to power and toughness.",
    ),
    (
        "name-matters",
        "Cares about card names: matching them, differing them, or naming a specific one.",
    ),
    (
        "discard-with-set-s-mechanic",
        "Ties a discard effect to a mechanic specific to that card's set, like escape or kicker.",
    ),
    (
        "crew",
        "A Vehicle you turn into a creature by tapping creatures to crew it.",
    ),
    (
        "synergy-green",
        "Rewards you for playing green cards or working alongside green strategies.",
    ),
    (
        "synergy-commander",
        "Cares about your commander itself, such as buffing it or bringing it back.",
    ),
    (
        "regrowth-self",
        "Returns itself from your graveyard to your hand or the battlefield.",
    ),
    (
        "synergy-legendary",
        "Cares about legendary creatures or permanents you control.",
    ),
    (
        "restricted-mana",
        "Adds mana that can only be spent on certain things.",
    ),
    (
        "mill-opponent",
        "Puts cards from an opponent's library into their graveyard.",
    ),
    (
        "activate-from-graveyard",
        "Has an activated ability you can use while it's in your graveyard.",
    ),
    (
        "hate-artifact",
        "Hoses artifacts, such as by destroying, tapping, or punishing their controller.",
    ),
    (
        "cda-toughness",
        "Its toughness is set by a characteristic-defining ability instead of a fixed number.",
    ),
    (
        "gives-pp-counters-to-all",
        "Puts +1/+1 counters on all of your creatures at once.",
    ),
    ("jump", "Grants a creature flying until end of turn."),
    (
        "synergy-red",
        "Works especially well in red decks or strategies.",
    ),
    (
        "mill-any",
        "Mills cards from a library you choose, yours or an opponent's.",
    ),
    (
        "creature-count-matters",
        "Gets better the more creatures you control.",
    ),
    ("gives-deathtouch", "Grants deathtouch to other creatures."),
    (
        "group-hug",
        "Gives resources or benefits to every player at the table, not just you.",
    ),
    (
        "freeze-creature",
        "Stops a creature from untapping for one or more of its untap steps.",
    ),
    (
        "donate-token",
        "Creates a token under another player's control.",
    ),
    (
        "paper-compatible",
        "A digital-only card whose effect would still work if printed in paper.",
    ),
    (
        "gives-mm-counters",
        "Puts -1/-1 counters on other creatures.",
    ),
    (
        "free-sacrifice-outlet",
        "A repeatable sacrifice outlet with no extra cost to activate.",
    ),
    (
        "non-mana-ability-mana",
        "Produces mana through an ability that isn't a true mana ability, so it can be responded to.",
    ),
    (
        "synergy-black",
        "Rewards you for playing black cards or leaning into black's themes.",
    ),
    (
        "graveyard-fuel",
        "Exiles cards from a graveyard to fuel its spells and abilities.",
    ),
    (
        "graveyard-fuel-creature",
        "Exiles creature cards from a graveyard to fuel its abilities.",
    ),
    (
        "synergy-equipment",
        "Cares about Equipment, such as by rewarding equipping or moving it around.",
    ),
    (
        "toll",
        "Punishes or profits off actions that players can't easily avoid.",
    ),
    (
        "disenchant-naturalize",
        "Removal that destroys, exiles, or otherwise answers an artifact or enchantment.",
    ),
    (
        "regenerates-self",
        "Can regenerate itself to survive destruction.",
    ),
    ("gives-menace", "Grants menace to other creatures."),
    (
        "typal-coupling",
        "Cares about two or more different creature types at once.",
    ),
    (
        "tutor-to-battlefield",
        "Searches your library for a card and puts it straight onto the battlefield.",
    ),
    (
        "cards-in-graveyard-matter",
        "Cares about the cards in a graveyard, not just how many are there.",
    ),
    (
        "repeatable-artifact-tokens",
        "Repeatedly creates artifact tokens.",
    ),
    ("untaps-self", "Untaps itself so it can be used again."),
    (
        "repeatable-impulse",
        "Repeatedly digs into the top of your library to pull out cards you want.",
    ),
    (
        "giant-growth",
        "A combat trick that gives a creature a temporary boost to power and toughness.",
    ),
    (
        "repeatable-impulsive-draw",
        "Repeatedly lets you exile and play cards impulsively drawn from your library.",
    ),
    (
        "force-attacker",
        "Forces a creature to attack, whether it wants to or not.",
    ),
    (
        "synergy-blocker-self",
        "A creature that gains a bonus or triggers an ability when it blocks.",
    ),
    (
        "synergy-blue",
        "Works especially well in decks built around blue strategies.",
    ),
    (
        "vanity-card",
        "A card named after a real person, place, or thing as a nod or easter egg.",
    ),
    (
        "counterspell-with-set-mechanic",
        "A counterspell that also carries its set's keyword mechanic.",
    ),
    (
        "titan-trigger",
        "Has an ability that triggers both when it enters and when it attacks.",
    ),
    (
        "multi-land-ramp",
        "Land ramp that can net you two or more lands at once.",
    ),
    ("gives-unblockable", "Makes another creature unblockable."),
    ("color-change", "Changes the color of a permanent or spell."),
    (
        "damage-prevention-creature",
        "Prevents damage that would be dealt to a creature.",
    ),
    ("bounce-self", "Returns itself to its owner's hand."),
    (
        "one-sided-fight",
        "Deals damage equal to one creature's power to another creature, with no chance to fight back.",
    ),
    (
        "type-addition-from-none",
        "A creature originally printed with no creature type, later given one by errata.",
    ),
    (
        "damage-prevention-you",
        "Prevents damage that would be dealt to you.",
    ),
    (
        "gains-first-strike",
        "Gains first strike, hitting in combat before creatures without it.",
    ),
    (
        "gives-hexproof",
        "Grants hexproof to other permanents so opponents can't target them.",
    ),
    (
        "leaves-trigger-self",
        "Has an ability that triggers when it leaves the battlefield.",
    ),
    (
        "rhyming-name",
        "A card whose name contains a rhyme or comes close to one.",
    ),
    (
        "mix-and-match",
        "Blends two or more non-evergreen mechanics together on the same card.",
    ),
    ("copy-instant", "Copies an instant or sorcery spell."),
    (
        "renew",
        "Has an ability you can use by exiling this card from your graveyard.",
    ),
    (
        "synergy-enchantment",
        "Rewards you for playing or controlling enchantments.",
    ),
    (
        "hate-tapped",
        "Exploits or punishes permanents that are tapped, usually creatures.",
    ),
    (
        "hate-high-pt",
        "Punishes or removes creatures with high power or toughness.",
    ),
    (
        "rescue-creature",
        "Returns a creature you control to its owner's hand, often to reuse its ability or dodge removal.",
    ),
    ("cost-reducer", "Makes other spells cost less to cast."),
    (
        "aikido",
        "Turns an opponent's own creatures, power, or damage against them.",
    ),
    (
        "color-break",
        "Lets a color do something outside its normal identity, breaking the usual color pie.",
    ),
    (
        "hate-black",
        "Punishes or protects you against black spells and black permanents.",
    ),
    (
        "copy-sorcery",
        "Copies an instant or sorcery spell you cast or control.",
    ),
    (
        "copy-spell",
        "Duplicates a spell on the stack so its effect happens again.",
    ),
    (
        "faux-targeting",
        "Chooses a permanent or player without targeting, so hexproof and shroud don't stop it.",
    ),
    (
        "gives-double-strike",
        "Grants double strike to a creature or creatures.",
    ),
    (
        "gains-trample",
        "A creature that gains trample, often as part of a bigger boost or condition.",
    ),
    (
        "cast-trigger",
        "Triggers an effect whenever any player casts a spell.",
    ),
    (
        "reanimate-cast",
        "Lets you cast a permanent card straight out of your graveyard.",
    ),
    (
        "prevent-attack",
        "Stops a creature from attacking, and usually from blocking too.",
    ),
    (
        "impulse-creature",
        "Reveals cards from the top of your library and lets you grab a creature.",
    ),
    (
        "removal-planeswalker",
        "Removal that can destroy or remove a planeswalker, often alongside creatures.",
    ),
    (
        "uninspired",
        "Triggers an effect whenever a permanent becomes tapped.",
    ),
    (
        "unique-counter",
        "Uses a special kind of counter that no other card in Magic uses.",
    ),
    (
        "removal-tuck",
        "Removal that puts a permanent into its owner's library instead of destroying it.",
    ),
    (
        "cda-color",
        "A card whose color comes from a characteristic-defining ability instead of its mana cost.",
    ),
    (
        "synergy-artifact-creature",
        "Rewards you for having artifact creatures or cares about them specifically.",
    ),
    (
        "portmanteau",
        "A card whose name blends two words together into one new word.",
    ),
    (
        "combat-ramp",
        "Generates extra mana or lands by attacking or dealing combat damage.",
    ),
    (
        "removal-fight",
        "Removal where your creature fights another, each dealing damage equal to its power.",
    ),
    (
        "trigger-from-graveyard",
        "Has an ability that triggers while the card sits in your graveyard, not on the battlefield.",
    ),
    (
        "multiplayer",
        "Scales with or interacts with every player in the game, not just one opponent.",
    ),
    (
        "synergy-planeswalker",
        "Cares about or supports the planeswalkers you control.",
    ),
    (
        "thoughtseize",
        "Makes an opponent reveal their hand and you pick a card for them to discard.",
    ),
    (
        "toughness-matters",
        "Cares about a creature's toughness, such as gaining life or dealing damage equal to it.",
    ),
    (
        "gains-indestructible",
        "Gives itself indestructible, usually as a temporary or conditional ability.",
    ),
    (
        "hate-regenerate",
        "Prevents a creature from being regenerated, usually while destroying it.",
    ),
    (
        "ditch-hand",
        "Lets you discard or empty your whole hand, often to refill it with new cards.",
    ),
    (
        "typal-dragon",
        "Rewards you for controlling or casting Dragons.",
    ),
    (
        "burn-you",
        "Deals damage to yourself, usually as the cost or downside of an effect.",
    ),
    (
        "punisher",
        "Forces an opponent to choose between two bad outcomes.",
    ),
    (
        "hate-flying",
        "Fights flying creatures with reach, removal, or bonuses against them.",
    ),
    (
        "real-life-animal-name",
        "A card whose name matches a real-world animal.",
    ),
    (
        "giant-growth-with-set-mechanic",
        "A combat pump spell that also involves the set's signature mechanic.",
    ),
    (
        "synergy-aura",
        "Cares about Auras you control or cast, rewarding you for playing or attaching them.",
    ),
    (
        "heroic",
        "Triggers a bonus whenever you cast a spell that targets this creature.",
    ),
    (
        "hate-discard",
        "Rewards you for discarding cards, turning forced or chosen discards into an advantage.",
    ),
    (
        "bombard-self",
        "Lets you sacrifice this permanent to deal damage.",
    ),
    (
        "counterspell-soft",
        "Counters a spell unless its controller pays a cost or meets a condition to save it.",
    ),
    (
        "flicker-creature",
        "Exiles a creature you control and returns it to the battlefield.",
    ),
    (
        "gives-tap-ability",
        "Grants creatures a new ability they can activate by tapping.",
    ),
    (
        "consult-cast",
        "Exiles cards from your library until you hit one you may cast.",
    ),
    (
        "humble",
        "Strips a creature's abilities and shrinks its stats as a form of removal.",
    ),
    (
        "burn-player-each",
        "Deals damage to each creature and each player.",
    ),
    (
        "repeatable-plunder",
        "Lets you sacrifice a permanent again and again for value like cards or life.",
    ),
    (
        "typal-spirit",
        "Cares about or boosts creatures of the Spirit type.",
    ),
    (
        "poisonous",
        "A creature that gives a player poison counters when it deals them damage.",
    ),
    (
        "unprinted-token",
        "Creates a token that hasn't been printed as an official token card in paper.",
    ),
    (
        "mana-filter",
        "Converts mana you already have into mana of a different color.",
    ),
    ("rummage", "Discards a card, then draws a card."),
    (
        "typal-elf",
        "Cares about Elves, rewarding or supporting them.",
    ),
    (
        "swap-removal",
        "Removes a creature but gives its controller a replacement permanent in return.",
    ),
    (
        "hate-red",
        "Punishes or defends against red creatures and spells.",
    ),
    (
        "meme",
        "A card that has become a popular meme in the Magic community.",
    ),
    (
        "morbid",
        "Gets a bonus effect if a creature died this turn.",
    ),
    (
        "tome",
        "An artifact that taps to draw you cards for incremental advantage over time.",
    ),
    (
        "nightveil-theft",
        "Exiles cards from another player's library or hand and lets you cast them.",
    ),
    (
        "counter-fuel-aesthetic",
        "Builds up counters, then removes them as a resource to pay for an effect.",
    ),
    (
        "40k-model",
        "A card from the Warhammer 40,000 crossover set, styled after a tabletop model.",
    ),
    (
        "roll-d6",
        "Has you roll a six-sided die and resolves an effect based on the result.",
    ),
    (
        "gives-protection",
        "Grants protection from a color or other quality to a creature or permanent.",
    ),
    ("synergy-vehicle", "Rewards or cares about Vehicles."),
    (
        "deanimate-self",
        "A creature that can stop being a creature, such as through bestow or reconfigure.",
    ),
    (
        "lifegain-matters",
        "Cares about how much life you gained this turn.",
    ),
    (
        "cast-trigger-other",
        "Triggers an effect whenever another player casts a spell.",
    ),
    ("energy-generator", "Gives you energy counters."),
    ("typal-zombie", "Cares about or supports Zombie creatures."),
    (
        "life-loss-matters",
        "Cares about how much life a player lost this turn, not just damage dealt.",
    ),
    ("typal-sliver", "Cares about or supports Sliver creatures."),
    (
        "counter-fuel-energy",
        "Generates energy counters you can spend to pay for its own or other abilities.",
    ),
    (
        "hate-counterspell",
        "Dodges or shuts down counterspells, often by being uncounterable or using split second.",
    ),
    (
        "impulse-onto-battlefield",
        "Looks at cards from the top of your library and puts some straight onto the battlefield.",
    ),
    (
        "hand-size-matters",
        "Cares about how many cards are in a hand, scaling its effect to that number.",
    ),
    (
        "cost-ignorer",
        "Lets you cast a spell without paying its mana cost, or for an alternate cost instead.",
    ),
    (
        "exponential",
        "Grows or multiplies its effect at an accelerating, exponential rate if left unchecked.",
    ),
    (
        "repeatable-rummage",
        "Lets you discard and draw cards over and over, filtering your hand repeatedly.",
    ),
    (
        "animate-self",
        "A noncreature permanent that can turn itself into a creature.",
    ),
    (
        "rainbow-land",
        "A land that taps for one or more mana of any color.",
    ),
    (
        "tutored-by-name",
        "A card that another card can fetch from your library by name, such as a partner.",
    ),
    ("synergy-forest", "Gets better when you control Forests."),
    ("mill-exile", "Exiles cards from the top of a library."),
    (
        "restricted-blocker",
        "A creature that can only block under specific conditions.",
    ),
    (
        "hate-white",
        "Punishes white cards or grants protection from white.",
    ),
    (
        "ferocious",
        "Triggers or grants a bonus when you control a creature with power 4 or greater.",
    ),
    (
        "undergrowth",
        "Gets better based on how many creature cards are in your graveyard.",
    ),
    (
        "tap-fuel-power",
        "Lets you tap a creature and use an amount equal to its power to fuel an effect.",
    ),
    (
        "gives-castable-from-graveyard",
        "Lets you cast a card straight from a graveyard instead of your hand.",
    ),
    (
        "mulch",
        "Looks at several cards off the top of your library, keeps one, and mills the rest.",
    ),
    (
        "synergy-flying",
        "Rewards, boosts, or otherwise cares about creatures with flying.",
    ),
    (
        "restricted-attacker",
        "A creature that can only attack when a specific condition is met.",
    ),
    (
        "exile-on-resolution",
        "Exiles itself after it finishes resolving, rather than going to the graveyard.",
    ),
    (
        "personal-text",
        "Uses personal pronouns like he, she, or him in its rules text instead of it.",
    ),
    (
        "typal-goblin",
        "Cares about Goblins, whether you control them or how many you have.",
    ),
    (
        "mass-land-denial",
        "Destroys, exiles, bounces, or locks down lands on a mass scale to choke off mana.",
    ),
    (
        "convoke",
        "Lets you tap your creatures to help pay this spell's cost.",
    ),
    (
        "tutor-card",
        "Searches your library for any card, with no restriction on what you can find.",
    ),
    (
        "repeatable-rescue",
        "Repeatedly returns a permanent to hand to rescue your own or bounce a threat.",
    ),
    (
        "synergy-token-creature",
        "Cares about or empowers the creature tokens you control.",
    ),
    (
        "threshold",
        "Grants a bonus as long as you have seven or more cards in your graveyard.",
    ),
    (
        "french-vanilla-aura",
        "An aura that only grants stat boosts and/or keywords, with no other effects.",
    ),
    (
        "banish",
        "Exiles a permanent for as long as this card stays on the battlefield.",
    ),
    (
        "long-term-impulsive-draw",
        "Exiles cards you can play beyond this turn, giving you extra time to use them.",
    ),
    (
        "gives-mana-ability",
        "Grants a permanent a tap ability to add mana.",
    ),
    ("loot", "Draws you a card, then makes you discard a card."),
    (
        "mana-rock",
        "A noncreature artifact that taps for mana to help you cast spells.",
    ),
    (
        "self-discard-matters",
        "Rewards you for discarding cards from your own hand.",
    ),
    (
        "universal-type-change",
        "Changes the type of every card or permanent of one kind into another type.",
    ),
    (
        "mana-rock-with-set-s-mechanic",
        "A mana rock that also plugs into that set's signature mechanic.",
    ),
    (
        "combat-neutral-damage-trigger",
        "Triggers whenever this creature deals damage, in combat or otherwise.",
    ),
    (
        "animate-land",
        "Turns a land into a creature while it stays a land.",
    ),
    (
        "opponent-chooses",
        "Forces an opponent to make a choice that affects the outcome.",
    ),
    (
        "flicker-slow",
        "Exiles a permanent and returns it to the battlefield at the beginning of the next end step.",
    ),
    (
        "turn-face-up-trigger-self",
        "Triggers an effect when this creature is turned face up from morph, megamorph, or disguise.",
    ),
    (
        "lockdown-creature",
        "Keeps a creature from untapping during its controller's untap step, often tapping it first.",
    ),
    (
        "shrink",
        "Reduces a creature's power, usually without changing its toughness.",
    ),
    (
        "nonbasic-basic-land-type",
        "A nonbasic land that also has one or more basic land types like Island or Forest.",
    ),
    (
        "flowstone",
        "Grants a creature +N/-N or -N/+N, boosting one stat while cutting the other.",
    ),
    (
        "draw-matters",
        "Triggers or alters an effect when you draw a card.",
    ),
    (
        "extra-untap",
        "Untaps many permanents at once or adds an extra untap step.",
    ),
    (
        "mana-producer",
        "A nonland card that adds mana you can spend.",
    ),
    (
        "damage-prevention-self",
        "Prevents damage that would be dealt to itself.",
    ),
    (
        "card-types-in-graveyard-matter",
        "Gets stronger based on how many different card types are in your graveyard.",
    ),
    (
        "synergy-sacrifice",
        "Rewards you for sacrificing your own permanents.",
    ),
    (
        "typal-human",
        "Cares about Humans you control or being a Human.",
    ),
    (
        "synergy-token",
        "Cares about creating, having, or sacrificing tokens.",
    ),
    (
        "cheat-death-self",
        "Returns itself to the battlefield or your hand the moment it dies.",
    ),
    (
        "quick-equip",
        "Attaches Equipment through an ability other than paying its equip cost.",
    ),
    (
        "gains-lifelink",
        "Grants itself lifelink, usually only under certain conditions or until end of turn.",
    ),
    ("exhaust", "Has an ability you can activate only once."),
    (
        "impulse-land",
        "Digs through the top of your library and puts a land into your hand or onto the battlefield.",
    ),
    (
        "lands-matter",
        "Cares about how many lands, or which land types, you control.",
    ),
    ("synergy-blocker", "Rewards or boosts creatures that block."),
    (
        "counter-fuel-pp",
        "Lets you remove a +1/+1 counter from this creature to power an ability.",
    ),
    (
        "protects-planeswalker",
        "Shields your planeswalkers from being attacked or damaged.",
    ),
    (
        "amount-spent-matters",
        "Gets a bonus based on how much mana you spend.",
    ),
    (
        "type-errata-summon-creature",
        "A creature whose type line once read Summon Creature under old Portal-era templating.",
    ),
    (
        "commander-set-booster-cards",
        "A card that debuted in a Commander set and was only available in that set's boosters.",
    ),
    (
        "full-refund",
        "Gives back mana equal to or more than what you spent to cast it.",
    ),
    (
        "conjure-to-hand",
        "Conjures a new card directly into a player's hand.",
    ),
    (
        "minigame",
        "Engages other players in a bet, vote, or guessing game.",
    ),
    (
        "regenerates-other",
        "Regenerates another creature or permanent, saving it from destruction.",
    ),
    (
        "tutor-creature",
        "Searches your library for a creature card.",
    ),
    (
        "cheat-death",
        "Returns a creature or permanent to the battlefield or hand right as it dies.",
    ),
    ("untapper-land", "Untaps one or more lands."),
    ("gains-vigilance", "Grants itself vigilance."),
    (
        "leaves-body-behind",
        "Creates a creature when it dies or leaves the battlefield.",
    ),
    (
        "dnd-monster",
        "A card named after and depicting a monster from Dungeons and Dragons.",
    ),
    (
        "conjure-named",
        "Creates a card with a specific name into your library, hand, graveyard, or battlefield.",
    ),
    (
        "temporary-reanimation",
        "Returns a creature from a graveyard to the battlefield, then removes it at end of turn.",
    ),
    (
        "pacifism",
        "Removal that stops a creature from attacking and blocking without destroying it.",
    ),
    (
        "bombard",
        "Lets you sacrifice another permanent to deal damage.",
    ),
    (
        "multiple-species-types",
        "A creature that has two or more creature types counted as species, like Human and Elf.",
    ),
    (
        "strive",
        "Costs more mana for each additional target you choose beyond the first.",
    ),
    (
        "counterspell",
        "Counters a target spell, stopping it from resolving.",
    ),
    (
        "specialized",
        "A creature that gains new abilities or triggers once it specializes.",
    ),
    (
        "hate-color-choose",
        "Grants protection from a color of your choice.",
    ),
    (
        "conjure-creature",
        "Conjures a new creature card into your library, hand, or the battlefield.",
    ),
    (
        "sneak-creature",
        "Puts a creature card from your hand onto the battlefield without paying its cost.",
    ),
    (
        "reanimate-from-any",
        "Returns a creature card from any graveyard to the battlefield under your control.",
    ),
    (
        "scales-with-power",
        "An effect that grows stronger based on a creature's power.",
    ),
    (
        "imprint",
        "Exiles a card to reference later, powering an effect tied to what was exiled.",
    ),
    (
        "multi-character-card",
        "Depicts two or more distinct named characters on a single card.",
    ),
    (
        "synergy-swamp",
        "Gets stronger or unlocks effects when you control Swamps.",
    ),
    (
        "hate-blue",
        "Punishes or protects you against blue spells, sources, or creatures.",
    ),
    (
        "catch-up",
        "Rewards you with an extra effect when you're behind in lands, life, cards, or creatures.",
    ),
    (
        "gives-reach",
        "Grants a creature reach so it can block creatures with flying.",
    ),
    (
        "typal-choose",
        "Lets you choose a creature type, then keys an effect off that type.",
    ),
    (
        "passive-ability",
        "A planeswalker with a non-loyalty ability alongside its loyalty abilities.",
    ),
    (
        "artifactfall",
        "Triggers an effect whenever an artifact enters the battlefield.",
    ),
    (
        "hate-instant",
        "Punishes or steals value from opponents' instant and sorcery spells.",
    ),
    (
        "synergy-mill",
        "Rewards putting cards from a library into a graveyard, yours or an opponent's.",
    ),
    (
        "hate-green",
        "Targets, weakens, or punishes green creatures and spells.",
    ),
    (
        "peek-hand",
        "Reveals a player's hand to look at their cards.",
    ),
    (
        "land-count-matters",
        "Scales an effect based on how many lands you control.",
    ),
    (
        "eponymous",
        "A creature whose name is made up of its own creature types.",
    ),
    (
        "exile-self-dfc-transform",
        "A double-faced card that exiles itself as part of transforming into its other side.",
    ),
    (
        "synergy-mountain",
        "Rewards or cares about Mountains you control.",
    ),
    (
        "coin-flip",
        "Flips a coin, with the effect depending on whether you win or lose the flip.",
    ),
    (
        "misnomer",
        "A card whose name suggests a mechanic, color, or type it doesn't actually have.",
    ),
    (
        "synergy-food",
        "Creates, sacrifices, or otherwise cares about Food tokens.",
    ),
    (
        "mixed-subtypes",
        "Has subtypes from different card types on one type line, like a creature type on a land.",
    ),
    (
        "weaker-in-singleton-formats",
        "Relies on multiples of the same card name, so it does less in formats that only allow one copy.",
    ),
    (
        "color-change-self",
        "Changes its own color, sometimes becoming a creature of a new color as well.",
    ),
    (
        "repeatable-clues",
        "Can create Clue tokens over and over, letting you draw extra cards later.",
    ),
    (
        "forced-attacker",
        "Must attack every combat if it's able to.",
    ),
    (
        "repeated-keyword",
        "Has the same keyword ability stated more than once or grants a keyword multiple times.",
    ),
    (
        "gains-deathtouch",
        "Gains deathtouch, making any damage it deals enough to destroy a creature.",
    ),
    (
        "seek-to-hand",
        "Seeks a card, putting a random matching card from your library straight into your hand.",
    ),
    (
        "gains-menace",
        "A creature that gains menace itself, so it can't be blocked except by two or more creatures.",
    ),
    (
        "typal-vampire",
        "Cares about or rewards you for having Vampire creatures.",
    ),
    (
        "offcolor-additional-cost",
        "Has a kicker or optional cost paid in a color outside the card's own colors for a bonus effect.",
    ),
    (
        "typal-ally",
        "Cares about or rewards you for having Ally creatures.",
    ),
    (
        "hellbending",
        "Empties your hand or helps you get there, often through discard outlets.",
    ),
    (
        "typal-merfolk",
        "Cares about or rewards you for having Merfolk creatures.",
    ),
    (
        "un-type-line",
        "A joke card with a silly card type only found in silver-bordered or Acorn-stamped cards.",
    ),
    (
        "trigger-from-exile",
        "Has an ability that triggers while the card sits in exile, like suspend.",
    ),
    (
        "sweeper-graveyard",
        "Exiles an entire graveyard, or all graveyards, at once.",
    ),
    (
        "alternate-win-condition",
        "Lets you win the game, or makes an opponent lose it, outside of reducing life to zero.",
    ),
    (
        "living-weapon",
        "An Equipment that creates a creature token and attaches itself to it when it enters.",
    ),
    (
        "tutor-mv",
        "Searches your library for a card with a specific mana value.",
    ),
    (
        "curiosity-like",
        "Gives you card advantage when a creature deals combat damage to a player.",
    ),
    (
        "tapland-with-set-s-mechanic",
        "A land that enters tapped and also carries the set's signature mechanic, like cycling.",
    ),
    (
        "synergy-island",
        "Rewards you for controlling Islands or cares how many you have.",
    ),
    (
        "hate-nonbasic-land",
        "Destroys, stops from untapping, or otherwise punishes nonbasic lands.",
    ),
    (
        "tutors-by-name",
        "Searches your library for a card with a specific name.",
    ),
    (
        "threaten",
        "Gains control of a creature until end of turn, usually untapping it with haste.",
    ),
    (
        "synergy-arcane",
        "Triggers a bonus effect whenever you cast a Spirit or Arcane spell.",
    ),
    (
        "named-token",
        "A token with its own specific name instead of one taken from its type line.",
    ),
    (
        "land-conversion",
        "Makes a land become a different or additional land type.",
    ),
    (
        "homeward-effect",
        "Temporarily exiles a permanent, then returns it to the battlefield under its owner's control.",
    ),
    (
        "hate-sorcery",
        "Lets you cast an instant or sorcery card taken or exiled from an opponent.",
    ),
    (
        "quadratic",
        "Grows in strength based on a count that itself increases, so its effect compounds each time.",
    ),
    (
        "ramp-with-set-s-mechanic",
        "Ramp that puts lands onto the battlefield while working in that set's signature mechanic.",
    ),
    (
        "man-o-war",
        "A creature that returns a target creature to its owner's hand when it enters.",
    ),
    (
        "overrun",
        "Gives creatures you control +X/+X and trample until end of turn.",
    ),
    (
        "notorious-templating",
        "Has rules text so wordy or convoluted it is notoriously hard to parse at a glance.",
    ),
    (
        "type-errata-hound",
        "A creature that used to be a Hound but was errata'd to the Dog creature type.",
    ),
    (
        "top-matters",
        "Cares about the top card of your library for something other than drawing or playing it.",
    ),
    (
        "animate-artifact",
        "Turns a noncreature artifact into a creature.",
    ),
    (
        "typal-share",
        "Cares about creatures that share a creature type with each other.",
    ),
    (
        "typal-wizard",
        "Cares about the number of Wizards you control or rewards you for casting Wizard spells.",
    ),
    (
        "synergy-multicolor",
        "Rewards you for casting or controlling multicolored cards.",
    ),
    (
        "maro-sorcerer",
        "A creature whose power and toughness scale with how many lands, or a land type, you control.",
    ),
    (
        "counterspell-reusable",
        "A permanent or repeatable ability that can counter spells again and again.",
    ),
    (
        "sneak-land",
        "Puts a land from your hand onto the battlefield without playing it.",
    ),
    ("untapper-permanent", "Untaps a target permanent."),
    (
        "donate",
        "Gives control of a permanent you control to another player.",
    ),
    (
        "drain-creature",
        "Deals damage to a creature and you gain life equal to the damage dealt.",
    ),
    (
        "life-total-matters-self",
        "Cares whether your own life total is above or below a certain threshold.",
    ),
    (
        "lhurgoyf",
        "A creature whose power or toughness scales with the number of cards in a graveyard.",
    ),
    (
        "pseudo-fog",
        "Stops an entire combat phase by means other than damage prevention, such as tapping attackers.",
    ),
    (
        "regrowth-sorcery",
        "Returns a sorcery card from your graveyard to your hand.",
    ),
    (
        "cost-increaser",
        "Makes spells or abilities cost more for you or your opponents to cast or activate.",
    ),
    (
        "regrowth-instant",
        "Returns an instant card from your graveyard to your hand.",
    ),
    (
        "enchantmentfall",
        "Triggers an effect whenever an enchantment enters the battlefield under your control.",
    ),
    (
        "enrage",
        "Triggers an effect whenever the creature is dealt damage.",
    ),
    (
        "creates-token-of-a-card",
        "Creates a token whose characteristics match a real printed card.",
    ),
    (
        "discard-to-exile",
        "Exiles cards from an opponent's hand instead of sending them to the graveyard.",
    ),
    (
        "phasing",
        "Phases a permanent out so it's treated as nonexistent until it phases back in.",
    ),
    (
        "death-trigger-opponent",
        "Triggers whenever a permanent an opponent controls is put into a graveyard.",
    ),
    (
        "clone",
        "Lets a creature enter the battlefield as a copy of another creature.",
    ),
    ("plunder", "Lets you sacrifice a permanent to draw cards."),
    (
        "reanimate-artifact",
        "Returns an artifact card from your graveyard to the battlefield.",
    ),
    (
        "turns-off-defender-self",
        "Lets this creature attack despite having defender, usually only under a condition.",
    ),
    (
        "mana-ability-with-extra-effect",
        "A mana ability that also causes an extra effect, like damage, alongside adding mana.",
    ),
    (
        "inverted-effects",
        "Applies opposite effects to two sides at once, helping one and hindering the other.",
    ),
    (
        "unique-plane-type",
        "A Planechase plane card with a subtype that appears on no other plane.",
    ),
    (
        "warlord",
        "A creature whose power, and sometimes toughness, equals how many creatures you control.",
    ),
    (
        "second-spell-matters",
        "Triggers or gets better when you cast your second spell in a turn.",
    ),
    (
        "unique-cr-reference",
        "A card so unusual it needed its own dedicated rule written into the comprehensive rules.",
    ),
    (
        "conjure-duplicate",
        "Conjures a duplicate of an existing card rather than a specific named card.",
    ),
    (
        "tap-fuel-artifact",
        "Lets you tap untapped artifacts you control as a cost to power an ability.",
    ),
    (
        "scales-with-multiple",
        "Gets stronger the more copies of itself you have.",
    ),
    (
        "synergy-low-power",
        "Cares about creatures with low power, rewarding or targeting them specifically.",
    ),
    (
        "free-discard-outlet",
        "Lets you discard a card as a cost, with no mana cost or per-turn limit.",
    ),
    (
        "gives-ward",
        "Grants ward to a creature, forcing opponents to pay a cost to target it.",
    ),
    (
        "type-errata-name-self",
        "A creature that once had its own name as its creature type, before type errata.",
    ),
    (
        "shapesharing",
        "Makes a permanent already on the battlefield become a copy of another creature.",
    ),
    (
        "werewolf-mechanic",
        "Transforms to its werewolf side if no one casts a spell for a turn, and back if two spells are cast in a turn.",
    ),
    (
        "copy-artifact",
        "Makes a copy of an artifact, often as a token.",
    ),
    (
        "french-vanilla-walker",
        "A creature whose only ability is a single landwalk keyword.",
    ),
    (
        "tapper-artifact",
        "Taps down a target artifact, keeping it from using its tap abilities.",
    ),
    (
        "unique-keyword",
        "Uses a keyword ability or variant that appears on no other card.",
    ),
    (
        "leaves-battlefield-trigger",
        "Triggers an effect whenever a permanent leaves the battlefield.",
    ),
    (
        "scales-with-damage-dealt",
        "An effect that gets bigger the more damage the creature deals.",
    ),
    (
        "move-counters",
        "Moves counters from one permanent onto another instead of making new ones.",
    ),
    (
        "prevent-activation",
        "Stops activated abilities from being used.",
    ),
    (
        "dexterity",
        "Requires a physical action outside normal play, like flicking cards or rolling dice.",
    ),
    (
        "defector",
        "A permanent that hands control of itself to another player.",
    ),
    (
        "second-draw-matters",
        "Triggers or grows when you draw your second card in a turn.",
    ),
    (
        "rescue-land",
        "Returns a land you control to your hand, often as a cost or additional cost for another effect.",
    ),
    (
        "haven",
        "Exiles your own permanents to keep them safe, then lets you bring them back later.",
    ),
    (
        "color-spent-matters",
        "Grants a bonus effect if a specific color of mana was spent to pay for it.",
    ),
    (
        "synergy-basic",
        "Rewards you for controlling basic lands, often scaling with how many you have.",
    ),
    (
        "restock-to-top",
        "Puts cards from a graveyard back on top of a library so they'll be drawn again.",
    ),
    (
        "state-trigger",
        "Triggers off an ongoing condition being true, like controlling no lands of a type, not a single event.",
    ),
    (
        "naturalize-with-set-mechanic",
        "Destroys an artifact or enchantment while also using a mechanic specific to its set.",
    ),
    (
        "impulse",
        "Lets you look at the top cards of your library and put one into your hand.",
    ),
    (
        "artifactify",
        "Turns a card or permanent into an artifact, in addition to or instead of its other types.",
    ),
    (
        "hatebear",
        "A cheap, small creature whose ability disrupts opponents' strategies.",
    ),
    (
        "ritual",
        "An instant or sorcery that adds mana when it resolves.",
    ),
    (
        "counter-fuel-charge",
        "Stores charge counters you remove to power an activated ability.",
    ),
    (
        "auraify",
        "Turns a creature or permanent into an Aura enchantment.",
    ),
    (
        "gives-flash",
        "Lets you cast certain spells as though they had flash.",
    ),
    (
        "support",
        "Puts a +1/+1 counter on each of up to a number of target creatures.",
    ),
    (
        "set-life-total",
        "Sets a player's life total to a specific number or exchanges it with another value.",
    ),
    (
        "consult-onto-battlefield",
        "Reveals cards from a library until one of a certain type, then puts it onto the battlefield.",
    ),
    (
        "extra-turn",
        "Gives a player an additional turn after the current one.",
    ),
    (
        "usg-storyline-in-cards",
        "A card whose flavor ties into the Urza's Saga storyline.",
    ),
    (
        "specter-ability",
        "Makes a player discard a card when a creature deals combat damage to them.",
    ),
    (
        "power-doubler",
        "Doubles a creature's power until end of turn.",
    ),
    (
        "doom-blade",
        "Destroys a creature, but only if it meets some condition or restriction.",
    ),
    (
        "typal-dinosaur",
        "Cares about or boosts Dinosaur creatures.",
    ),
    (
        "counter-doubler",
        "Doubles the number of counters placed or tokens created.",
    ),
    (
        "synergy-colorless",
        "Rewards you for casting or controlling colorless spells and creatures.",
    ),
    (
        "madness",
        "Lets you cast a discarded card for its madness cost instead of putting it in the graveyard.",
    ),
    (
        "splits-on-death",
        "Creates two or more creature tokens when a creature dies.",
    ),
    (
        "gains-double-strike",
        "Gains double strike itself under some condition.",
    ),
    (
        "hate-full-hand",
        "Rewards you or punishes an opponent based on how many cards are in their hand.",
    ),
    (
        "sacrifice-outlet-enchantment",
        "Lets you sacrifice an enchantment as a cost or trigger to power an effect.",
    ),
    (
        "disintegrate",
        "Deals damage that exiles the creature instead of letting it die.",
    ),
    (
        "reanimate-permanent",
        "Returns a permanent card from a graveyard straight to the battlefield.",
    ),
    (
        "auto-equip",
        "An Equipment that attaches itself to a creature you control as it enters.",
    ),
    (
        "synergy-activated-ability",
        "Cares about or fuels the activation of abilities on other cards.",
    ),
    (
        "protects-artifact",
        "Shields your artifacts from being targeted, destroyed, or removed.",
    ),
    ("twiddle", "Taps or untaps a target permanent."),
    (
        "speech-matters",
        "Cares about what players actually say out loud during the game.",
    ),
    ("draw-hate", "Punishes or restricts players who draw cards."),
    (
        "regrowth-permanent",
        "Returns a permanent card from your graveyard to your hand.",
    ),
    (
        "regrowth-artifact",
        "Returns an artifact card from your graveyard to your hand.",
    ),
    (
        "gains-hexproof",
        "Gives itself hexproof so it can't be targeted by opponents.",
    ),
    (
        "mana-increaser",
        "Adds extra mana whenever a land or other mana source is tapped for mana.",
    ),
    (
        "wish",
        "Lets you bring in a card you own from outside the game and put it into your hand.",
    ),
    (
        "dnd-item",
        "A card named after and representing a Dungeons and Dragons magic item.",
    ),
    (
        "tmp-storyline-in-cards",
        "A card tied to Tempest block's storyline, without a mechanical theme of its own.",
    ),
    (
        "tuck-self",
        "Puts itself on top of or into its owner's library instead of another zone.",
    ),
    (
        "keyword-errata-flash",
        "Has flash, printed before flash existed as an official keyword.",
    ),
    (
        "fog-selective",
        "Prevents damage this turn, but only from some sources or to some targets, not everything.",
    ),
    (
        "typal-army",
        "Cares about Army creatures, often by amassing them into a growing token.",
    ),
    (
        "tapland",
        "A nonbasic land that always enters the battlefield tapped.",
    ),
    (
        "repeatable-food",
        "Lets you create Food tokens again and again, not just once.",
    ),
    (
        "class-type-only",
        "A creature typed only by its class, like Wizard or Knight, with no race type.",
    ),
    (
        "mill-each",
        "Mills cards from every player's library into their graveyard at once.",
    ),
    (
        "slith-ability",
        "Puts a +1/+1 counter on a creature whenever it deals combat damage to a player.",
    ),
    (
        "creature-ability-noncreature",
        "A noncreature permanent with abilities that matter once it becomes a creature.",
    ),
    (
        "synergy-snow",
        "Cares about how many snow permanents or how much snow mana you control.",
    ),
    (
        "delayed-cantrip",
        "Draws you a card at the beginning of the next turn's upkeep instead of immediately.",
    ),
    (
        "synergy-burn",
        "Rewards or synergizes with damage your sources deal, especially burn.",
    ),
    (
        "synergy-plains",
        "Gets better or unlocks an effect when you control a Plains.",
    ),
    (
        "hate-target",
        "Punishes players for targeting your permanents, or protects them from being targeted.",
    ),
    (
        "type-errata-viashino",
        "A creature that was errata'd from Viashino to Lizard in Modern Horizons 3.",
    ),
    (
        "place-sticker",
        "Lets you put a sticker on a permanent or card you own.",
    ),
    (
        "discarded-type-matters",
        "Has an effect that changes based on the card type of what gets discarded.",
    ),
    (
        "mana-fix",
        "Lets your lands tap for additional colors of mana.",
    ),
    (
        "conjure-to-battlefield",
        "Creates a new card from outside the game and puts it directly onto the battlefield.",
    ),
    (
        "opponent-lifegain",
        "Causes an opponent to gain life along with or instead of you.",
    ),
    (
        "dnd-mechanic",
        "A card named after and representing a Dungeons and Dragons ability or mechanic.",
    ),
    (
        "mass-reanimation",
        "Returns many or all cards of a kind from graveyards to the battlefield at once.",
    ),
    ("typal-warrior", "Cares about or rewards you for Warriors."),
    (
        "unheroic",
        "Rewards you for targeting an opponent, something they control, or cards in their graveyard.",
    ),
    (
        "tapper-land",
        "Taps lands or other permanents, often to deny mana or keep them from untapping.",
    ),
    (
        "sacrifice-outlet-permanent",
        "Lets you sacrifice any permanent, usually for a benefit.",
    ),
    (
        "hate-planeswalker",
        "Removal or restrictions aimed specifically at planeswalkers.",
    ),
    (
        "roll-d20",
        "Rolls a twenty sided die and resolves a different effect based on the result.",
    ),
    (
        "copy-token",
        "Creates a token that's a copy of a token already in play, like populate.",
    ),
    (
        "hand-size-increase",
        "Raises or removes your maximum hand size.",
    ),
    (
        "mana-storage",
        "Stores mana as counters or tokens so you can spend it later.",
    ),
    (
        "theft-artifact",
        "Lets you take control of an opponent's artifact, sometimes temporarily.",
    ),
    (
        "hellbent",
        "Triggers or improves when a player has no cards in hand.",
    ),
    (
        "regrowth-any",
        "Returns a card of any type from your graveyard to your hand.",
    ),
    (
        "typal-soldier",
        "Cares about or boosts creatures with the Soldier type.",
    ),
    ("charm", "Lets you choose one of three listed modes."),
    (
        "card-advantage",
        "Nets you extra cards or resources beyond a one-for-one trade.",
    ),
    (
        "extra-combat-phase",
        "Gives you an additional combat phase this turn.",
    ),
    (
        "discard-outlet-land",
        "Lets you discard a land card as a cost, enabling retrace and similar effects.",
    ),
    (
        "non-mana-ward",
        "Ward that makes an opponent pay a non-mana cost, like discarding a card or losing life, to target it.",
    ),
    (
        "reanimate-copy",
        "Brings back a copy of a card from a graveyard instead of the original, often as a token.",
    ),
    (
        "tutor-to-top",
        "Searches your library for a card and puts it on top instead of into your hand.",
    ),
    (
        "damage-multiplier",
        "Doubles or triples the damage a source would deal.",
    ),
    (
        "changeling",
        "A card that has every creature type at once, in every zone.",
    ),
    (
        "graveyard-seal",
        "Disrupts graveyards, exiling cards or blocking interaction to shut down reanimation.",
    ),
    (
        "day-night",
        "Cares about or triggers when day becomes night or night becomes day.",
    ),
    ("earthquake", "Deals damage to creatures without flying."),
    (
        "interrupt",
        "A legacy spell that resolved before the stack existed, now functioning as an instant.",
    ),
    (
        "precognition-engine",
        "Lets you see the top card of your library and cast or play it under certain conditions.",
    ),
    (
        "repeatable-mulch",
        "Repeatedly reveals top cards, keeps some and puts the rest in your graveyard.",
    ),
    (
        "prevent-cast",
        "Stops a player from casting spells or playing cards under certain conditions.",
    ),
    (
        "staple-with-set-s-mechanic",
        "A common card built around one of its set's featured mechanics.",
    ),
    (
        "counterspell-ability",
        "Counters a target activated or triggered ability on the stack.",
    ),
    (
        "threaten-with-set-s-mechanic",
        "Takes control of a creature until end of turn, then untaps it and gives it haste.",
    ),
    (
        "french-vanilla-equipment",
        "Equipment that only grants a stat boost and/or a keyword with no other effects.",
    ),
    (
        "rules-nightmare",
        "A card notorious for interactions that are difficult to resolve under the rules.",
    ),
    (
        "reanimate-land",
        "Returns land cards from a graveyard to the battlefield.",
    ),
    (
        "unique-evasion",
        "Has an unusual, narrow form of evasion, like protection or an odd unblockable clause.",
    ),
    (
        "synergy-treasure",
        "Rewards you for creating, sacrificing, or otherwise using Treasure tokens.",
    ),
    (
        "synergy-haste",
        "Cares about or rewards creatures with haste.",
    ),
    (
        "synergy-tapped",
        "Cares about or rewards tapped permanents you control.",
    ),
    (
        "tapper-permanent",
        "Taps a target permanent, often with an option to untap it instead.",
    ),
    (
        "damage-redirection",
        "Redirects damage from its intended target to a different creature, player, or planeswalker.",
    ),
    (
        "cost-reducer-creature",
        "Makes creature spells you cast cost less mana.",
    ),
    (
        "raid",
        "Gives you a bonus if you attacked with a creature this turn.",
    ),
    (
        "counterspell-creature",
        "Counters a creature spell before it resolves.",
    ),
    (
        "copy-ability",
        "Copies an activated or triggered ability, letting you choose new targets for the copy.",
    ),
    (
        "gains-mm-counters",
        "A creature that puts -1/-1 counters on itself, sometimes shedding them later.",
    ),
    (
        "gains-flash",
        "Lets you cast this spell any time you could cast an instant.",
    ),
    (
        "hate-enchantment",
        "Destroys, counters, or protects against enchantments.",
    ),
    (
        "tutor-land-forest",
        "Searches your library for a Forest card and puts it into your hand or onto the battlefield.",
    ),
    (
        "division",
        "Uses a value that's divided or halved, sometimes rounding the result.",
    ),
    (
        "wheel-one-sided",
        "Has you discard your hand and draw a fresh one, usually without affecting opponents.",
    ),
    (
        "typal-knight",
        "Cares about or boosts creatures with the Knight creature type.",
    ),
    (
        "unique-creature-type",
        "A creature with a creature type that no other card uses.",
    ),
    (
        "gives-shroud",
        "Grants shroud to another permanent, so it can't be targeted by spells or abilities.",
    ),
    (
        "synergy-color-share",
        "Rewards you for cards or creatures that share a color with each other.",
    ),
    (
        "scry-like",
        "Lets you look at the top card of a library and choose where it goes, similar to scry.",
    ),
    (
        "the-ring-tempts-you",
        "Tempts you with the Ring, leveling up its emblem to grant your Ring-bearer new abilities.",
    ),
    (
        "catalog",
        "Draws you two cards, then makes you discard a card.",
    ),
    (
        "counter-fuel-other",
        "Removes counters from your other permanents to pay for its effects.",
    ),
    (
        "synergy-cycling",
        "Triggers a bonus whenever a card is cycled or discarded.",
    ),
    ("untapper-artifact", "Untaps a target artifact."),
    (
        "exalted",
        "Whenever a creature you control attacks alone, it gets +1/+1 until end of turn.",
    ),
    (
        "legendary-team-up",
        "A legendary creature that teams up two named characters on one card.",
    ),
    (
        "copy-legendary",
        "Creates a nonlegendary token copy of a permanent you control, so you get to keep it.",
    ),
    (
        "typal-elemental",
        "Rewards you for controlling or having Elemental creatures.",
    ),
    (
        "lure-limited",
        "Forces a creature to be blocked if able, but not necessarily by every creature.",
    ),
    (
        "typal-pirate",
        "Cares about the number or presence of Pirates you control.",
    ),
    (
        "synergy-modified",
        "Cares about modified creatures, those with a counter, Aura, or Equipment on them.",
    ),
    (
        "parasitic-aura",
        "An Aura that damages the enchanted permanent's controller or makes them lose life.",
    ),
    (
        "peek-library",
        "Lets you look at one or more cards from the top of a library without drawing them.",
    ),
    (
        "remove-counters-other",
        "Removes counters from opponents' permanents or removes a player's poison counters.",
    ),
    (
        "miniwheel",
        "Makes you discard your hand and draw fewer than seven new cards.",
    ),
    (
        "reanimate-from-opponent",
        "Puts a creature card from an opponent's graveyard onto the battlefield under your control.",
    ),
    (
        "deal-with-the-devil",
        "A black enchantment with a powerful effect and a serious, potentially game-losing drawback.",
    ),
    (
        "token-errata",
        "Creates a token whose type line has since been changed by an official update.",
    ),
    (
        "change-target",
        "Lets you change the target of a spell or ability already on the stack.",
    ),
    (
        "hate-low-power",
        "Punishes, restricts, or removes creatures with low power.",
    ),
    (
        "tutor-artifact",
        "Searches your library for an artifact card and puts it into your hand or onto the battlefield.",
    ),
    (
        "type-addition-book",
        "An artifact card with the Book subtype.",
    ),
    (
        "mass-shrink",
        "Reduces the power of many creatures at once, often your opponents' or the attackers.",
    ),
    (
        "tutor-land-plains",
        "Searches your library for a Plains card and puts it into your hand or battlefield.",
    ),
    (
        "exchange-control",
        "Swaps control of permanents between players.",
    ),
    (
        "open-attraction",
        "Opens an Attraction, putting the top card of your Attraction deck onto the battlefield.",
    ),
    (
        "creature-type-name",
        "A creature whose name is made up of creature types.",
    ),
    (
        "monarch-matters",
        "Makes you the monarch, drawing a card each end step until someone takes the crown.",
    ),
    (
        "impact-effect",
        "Deals damage or makes a player lose life whenever a creature you control enters.",
    ),
    (
        "sliver-stackable",
        "A Sliver whose ability benefits all Slivers, stacking with each additional copy.",
    ),
    (
        "leaves-graveyard-trigger",
        "Triggers an effect whenever a card leaves your graveyard.",
    ),
    (
        "synergy-graveyard-cast",
        "Rewards or enables casting spells straight from your graveyard.",
    ),
    (
        "discard-outlet-random",
        "Lets you discard a card at random to pay a cost or fuel an effect.",
    ),
    (
        "play-additional-land",
        "Lets you play an extra land beyond your normal one per turn.",
    ),
    (
        "firebend-like",
        "Produces mana or Treasure whenever your creatures attack.",
    ),
    ("anagram", "A card whose name is an intentional anagram."),
    (
        "synergy-solo-attack",
        "Rewards you when a creature you control attacks alone.",
    ),
    (
        "restock-to-bottom",
        "Puts a card from a graveyard onto the bottom of its owner's library.",
    ),
    (
        "flying-counter",
        "Puts a flying counter on a creature, granting it flying.",
    ),
    (
        "synergy-historic",
        "Cares about historic spells or permanents: artifacts, legendaries, and Sagas.",
    ),
    (
        "restock-all",
        "Shuffles a player's entire graveyard into their library.",
    ),
    (
        "vanilla-aura",
        "An Aura whose only effect is changing the enchanted creature's power and toughness.",
    ),
    (
        "unblocked-trigger",
        "Triggers an effect when this creature attacks and isn't blocked.",
    ),
    (
        "bushido",
        "Gets +X/+X until end of turn whenever this creature blocks or becomes blocked.",
    ),
    (
        "ninjutsu",
        "Return an unblocked attacker to hand to put this creature onto the battlefield tapped and attacking.",
    ),
    (
        "remove-counters-you",
        "Removes counters from a permanent or player.",
    ),
    (
        "shares-name-with-a-mechanic",
        "A card whose name matches a keyword mechanic, whether or not it uses that mechanic.",
    ),
    (
        "impulse-artifact",
        "Digs through the top cards of your library for an artifact and puts it into your hand.",
    ),
    (
        "extract",
        "Exiles cards from a library, removing them from the game rather than just discarding them.",
    ),
    (
        "prevent-mass-blockers",
        "Stops a wide swath of creatures from blocking, often for the whole turn.",
    ),
    (
        "hate-noncreature",
        "Taxes or punishes casting noncreature spells.",
    ),
    (
        "modal-inverse-choices",
        "A modal spell whose options are mirror opposites, like hitting fliers or hitting non-fliers.",
    ),
    (
        "guess",
        "Makes a player guess at hidden information, like a card name or which fact is a lie.",
    ),
    (
        "sacrifice-outlet-token",
        "Lets you sacrifice a token, often as a cost to power a bonus effect.",
    ),
    (
        "armoring",
        "Pumps a creature's toughness for the turn, usually as an activated ability.",
    ),
    (
        "typal-hero",
        "Rewards you for controlling, casting, or attacking with Hero creatures.",
    ),
    (
        "paradox",
        "Cares about casting spells or playing lands from anywhere other than your hand.",
    ),
    (
        "counter-preservation-self",
        "Moves its own counters onto another creature you control when it dies.",
    ),
    (
        "synergy-sticker",
        "Puts a sticker on a permanent you own, or cares about stickers.",
    ),
    (
        "unique-protection",
        "Grants protection from something unusual, like a chosen player or die rolls, instead of a color.",
    ),
    (
        "combat-timing-restriction",
        "A spell you can only cast during a specific step of combat.",
    ),
    (
        "remove-from-stack",
        "Takes a spell off the stack by exiling, bouncing, or forcing the turn to end.",
    ),
    (
        "dnd-spell",
        "A card from a Dungeons and Dragons crossover set, named after a spell from that game.",
    ),
    (
        "gains-protection",
        "Gains protection from a color or card type, often until end of turn.",
    ),
    (
        "infusion",
        "Triggers a bonus effect if you gained life this turn.",
    ),
    (
        "ball-lightning",
        "A creature that hits hard with haste, then is gone by end of turn.",
    ),
    (
        "doesn-t-untap",
        "Stays tapped during your untap step unless something lets you untap it.",
    ),
    (
        "tutor-land-any",
        "Searches your library for any land card, with a restriction or cost on the effect.",
    ),
    (
        "mana-egg",
        "An artifact you sacrifice to add mana, often drawing a card or giving another bonus.",
    ),
    (
        "restock-self",
        "Puts itself back into your hand or library so you can use it again.",
    ),
    (
        "affinity-for-artifacts",
        "Costs 1 less to cast for each artifact you control.",
    ),
    (
        "pridemate",
        "Puts a +1/+1 counter on itself whenever you gain life.",
    ),
    (
        "synergy-poison",
        "Gets stronger or unlocks an effect when an opponent has poison counters.",
    ),
    (
        "copy-permanent-spell",
        "Creates a token copy of a permanent spell you cast.",
    ),
    (
        "trigger-doubler",
        "Causes a triggered ability to trigger an additional time.",
    ),
    (
        "day-zero-errata",
        "Received errata before or immediately after its initial release.",
    ),
    (
        "regrowth-land",
        "Returns a land card from a graveyard to its owner's hand.",
    ),
    (
        "phyrexian-mana-cost",
        "Its cost includes Phyrexian mana, payable with colored mana or 2 life per symbol.",
    ),
    (
        "any-player-ability",
        "Has an activated ability that any player, not just its controller, may activate.",
    ),
    (
        "repeatable-seek",
        "Repeatedly seeks a random card matching a condition from your library.",
    ),
    (
        "planeswalker-deck-face-card",
        "The featured planeswalker on the cover of a planeswalker deck product.",
    ),
    (
        "named-choice",
        "Presents a choice or vote between options identified by names rather than by their effects.",
    ),
    (
        "high-x-matters",
        "A spell with a chosen X value that gets a bonus effect if X is high enough.",
    ),
    (
        "daunt",
        "Can't be blocked by creatures with power 2 or less.",
    ),
    (
        "metalcraft",
        "Gets a bonus effect as long as you control three or more artifacts.",
    ),
    (
        "power-matters-individual",
        "Cares about a single creature's power, not the total power of your board.",
    ),
    (
        "tax-attack",
        "Makes creatures pay a cost in order to attack.",
    ),
    (
        "mm-counter-cost",
        "Puts a -1/-1 counter on your own creature as a cost or requirement for an effect.",
    ),
    (
        "cost-reducer-instant-sorcery",
        "Makes your instant and sorcery spells cost less to cast.",
    ),
    (
        "inscryption-achievement",
        "Shares a name with an achievement from the video game Inscryption.",
    ),
    (
        "type-errata-naga",
        "A creature retyped from Naga to Snake in a rules update.",
    ),
    (
        "flicker-self",
        "Exiles itself and returns to the battlefield, often to reset or retrigger its abilities.",
    ),
    (
        "graveyard-fuel-instant",
        "Exiles an instant or sorcery card from a graveyard to power its effect.",
    ),
    (
        "deprecated-p-t-counter",
        "Uses an old-style power or toughness counter other than +1/+1 or -1/-1.",
    ),
    (
        "wheel-symmetrical",
        "Makes each player empty their hand and draw a fresh new one.",
    ),
    (
        "random-discard",
        "Makes a player discard cards chosen at random instead of by choice.",
    ),
    (
        "synergy-party",
        "Cares about your party, up to one each of Cleric, Rogue, Warrior, and Wizard.",
    ),
    (
        "blood-artist-ability",
        "Whenever a creature dies, an opponent loses life and you often gain life.",
    ),
    (
        "unique-type-exclusion",
        "Refers to a 'non-[type]' exclusion that no other card uses for that type.",
    ),
    (
        "abyss",
        "Repeatedly forces players to sacrifice or destroy a permanent each turn, usually a creature.",
    ),
    (
        "filterland",
        "A land that taps for colorless but can also convert mana into other colors, sometimes more of it.",
    ),
    (
        "has-identical-token",
        "A card that also exists as a token with the exact same name and abilities.",
    ),
    (
        "pseudo-proliferate",
        "Doubles or adds extra counters on permanents, working like proliferate without using that keyword.",
    ),
    (
        "polymorph",
        "Removes a permanent, then puts a random replacement permanent onto the battlefield.",
    ),
    (
        "donate-mana",
        "Gives mana, Treasure, or similar resources to other players, not just yourself.",
    ),
    ("typal-bird", "Cares about or rewards Bird creatures."),
    (
        "transform-improvement",
        "A double-faced card whose back side does what the front does, only better.",
    ),
    (
        "unique-p-t",
        "A creature with a power and toughness combination not printed on any other card.",
    ),
    ("typal-rat", "Cares about or rewards Rat creatures."),
    (
        "pwdeck-sidekick",
        "Gets stronger or gains an ability while you control the matching named planeswalker.",
    ),
    (
        "restock-creature",
        "Puts a creature card from your graveyard back on top of your library.",
    ),
    (
        "enchantmentize",
        "Turns a permanent into an enchantment, often stripping its other card types.",
    ),
    (
        "counterspell-exile",
        "Counters a spell and exiles it instead of letting it go to the graveyard.",
    ),
    (
        "seek-mv",
        "Seeks a card of a certain mana value, putting one at random from your library into your hand.",
    ),
    (
        "graveyard-fuel-sorcery",
        "Exiles instant or sorcery cards from a graveyard to power an effect.",
    ),
    (
        "unique-planeswalker-type",
        "A planeswalker with its own unique subtype instead of a shared planeswalker type.",
    ),
    (
        "stasis",
        "Keeps some or all permanents from untapping during their untap steps.",
    ),
    (
        "soul-warden-ability",
        "Gains you life whenever a creature enters the battlefield.",
    ),
    (
        "regrowth-enchantment",
        "Returns a target enchantment card from your graveyard to your hand.",
    ),
    (
        "painland",
        "A land that deals damage to you when you tap it for mana.",
    ),
    (
        "counterspell-automatic",
        "Counters a spell on its own when a trigger condition is met, without you spending mana.",
    ),
    (
        "power-matters-total",
        "Cares about the combined power of a group of creatures.",
    ),
    (
        "clash-like",
        "Reveals top library cards and compares their mana values for an effect.",
    ),
    ("mimic", "Grants a permanent the abilities of other cards."),
    (
        "lobotomy",
        "Exiles every copy of a chosen card name from a player's hand, library, and graveyard.",
    ),
    (
        "variable-effect-same-ability",
        "An ability that does something extra the second or third time it resolves in a turn.",
    ),
    (
        "spite-damage",
        "Deals damage in retaliation whenever it is dealt damage.",
    ),
    (
        "extra-land",
        "Puts a land onto the battlefield outside your normal once-per-turn land play.",
    ),
    (
        "soothsaying",
        "Lets you look at the top cards of a library and put them back in any order.",
    ),
    (
        "pwdeck-tutor",
        "Searches your library and graveyard for its matching Planeswalker Deck planeswalker and puts it in hand.",
    ),
    (
        "typal-villain",
        "Cares about or boosts the Villain creatures you control.",
    ),
    (
        "ingest",
        "A creature that exiles cards from the top of an opponent's library when it deals combat damage.",
    ),
    (
        "counter-fuel-any",
        "Removes counters of any kind from a permanent to pay for an effect.",
    ),
    (
        "ransom",
        "Forces a player to sacrifice a permanent unless they pay a cost.",
    ),
    (
        "rescue-permanent",
        "Returns a permanent you control to its owner's hand.",
    ),
    (
        "rescue-nonland",
        "Returns a nonland permanent to its owner's hand.",
    ),
    (
        "protects-permanent",
        "Grants your permanents protection, hexproof, or indestructible.",
    ),
    (
        "potentially-free",
        "Can be cast without paying its mana cost if a condition is met.",
    ),
    (
        "typal-mount",
        "Cares about or interacts with Mounts and Vehicles you control.",
    ),
    (
        "whirlpool",
        "Shuffles your hand and graveyard into your library, then draws you a fresh hand.",
    ),
    (
        "young-pyromancer-ability",
        "Creates a creature token whenever you cast an instant or sorcery spell.",
    ),
    (
        "fog",
        "Prevents all or nearly all combat damage that would be dealt this turn.",
    ),
    (
        "protects-land",
        "Shields your lands or all your permanents from being destroyed or targeted.",
    ),
    (
        "cycle-ust-functional-variant",
        "An Unstable card with an alternate version sharing its collector number but different cost or text.",
    ),
    (
        "monstrosity",
        "Lets you pay mana to put +1/+1 counters on this creature and make it monstrous once.",
    ),
    (
        "regrowth",
        "Returns a card from your graveyard to your hand.",
    ),
    (
        "unpreventable-damage",
        "Deals damage that can't be prevented, or makes damage unpreventable.",
    ),
    (
        "earthbend",
        "Animates a land you control as a creature and loads it with +1/+1 counters.",
    ),
    ("removes-flying", "Takes away a creature's flying."),
    (
        "voting",
        "Has players vote, then resolves an effect based on the results.",
    ),
    (
        "old-blocking-deathtouch",
        "Destroys a creature it blocks or that blocks it, an old form of deathtouch.",
    ),
    (
        "breaks-ktk-morph-rule",
        "A morph creature that flips for under five mana to win combat, breaking a Khans design rule.",
    ),
    (
        "vanilla-equipment",
        "An Equipment that only changes the equipped creature's power and toughness.",
    ),
    (
        "landhome",
        "A creature bound to a land type, needing it to attack or to stay on the battlefield.",
    ),
    (
        "high-flying",
        "A flying creature that can block only creatures with flying.",
    ),
    (
        "repeatable-noncreature-tokens",
        "Creates noncreature tokens again and again over the course of the game.",
    ),
    (
        "mutual-sacrifice",
        "Forces every player, not just opponents, to sacrifice something.",
    ),
    (
        "hurricane",
        "Deals damage to each creature with flying, and often to players too.",
    ),
    (
        "revolt",
        "Gets a bonus if a permanent left the battlefield under your control this turn.",
    ),
    (
        "consult",
        "Digs through your library until you find a card meeting a condition, then takes it.",
    ),
    (
        "seek-nonland",
        "Puts a random nonland card from your library into your hand.",
    ),
    (
        "self-life-loss-matters",
        "Rewards you when you gained or lost life during the turn.",
    ),
    (
        "start-of-game",
        "Has an effect tied to your opening hand or the very start of the game.",
    ),
    (
        "powerstone-mana",
        "Creates mana that can't pay to cast nonartifact spells.",
    ),
    (
        "5c-set-mechanic-commander",
        "A five-color legendary creature built as a commander to support a set's mechanics or themes.",
    ),
    (
        "retaliate-to-damage",
        "Rewards you or punishes an opponent whenever you take damage.",
    ),
    (
        "hate-wide",
        "Scales up to punish opponents based on how many creatures they control.",
    ),
    (
        "theft-permanent",
        "Takes control of an opponent's permanent for yourself.",
    ),
    (
        "delayed-replacement-effect",
        "Sets up a one-time replacement for the next matching event this turn.",
    ),
    (
        "eponymous-planeswalker",
        "A planeswalker card named simply after its character, with no epithet.",
    ),
    (
        "inspired",
        "Triggers an effect whenever a permanent becomes untapped.",
    ),
    ("hate-lifegain", "Stops players from gaining life."),
    (
        "graveyard-fuel-self",
        "Lets you exile it from your graveyard, usually when it dies, for an extra effect.",
    ),
    (
        "synergy-clue",
        "Rewards you for creating, controlling, or sacrificing Clue tokens.",
    ),
    (
        "typal-giant",
        "Cares about Giant creatures, whether you control them or cast Giant spells.",
    ),
    (
        "typal-assassin",
        "Cares about Assassin creatures, rewarding you for controlling or attacking with them.",
    ),
    (
        "tutor-to-graveyard",
        "Searches your library for a card and puts it straight into your graveyard.",
    ),
    (
        "combat-ping",
        "Deals a small amount of damage to a creature during combat.",
    ),
    (
        "tormenting-voice",
        "Lets you discard a card to draw two cards, filtering through your hand.",
    ),
    (
        "sth-storyline-in-cards",
        "A card whose flavor text is part of a storyline told across the set.",
    ),
    (
        "opponent-discard-matters",
        "Rewards you whenever an opponent discards a card.",
    ),
    (
        "typal-cleric",
        "Cares about Clerics, growing stronger or gaining abilities as more are in play.",
    ),
    (
        "reanimate-enchantment",
        "Returns enchantment cards from a graveyard to the battlefield.",
    ),
    (
        "counterspell-noncreature",
        "Counters a noncreature spell, leaving creature spells unaffected.",
    ),
    (
        "tutor-land-mountain",
        "Searches your library for a Mountain card and puts it into your hand or onto the battlefield.",
    ),
    (
        "magecraft",
        "Triggers an effect whenever you cast or copy an instant or sorcery spell.",
    ),
    (
        "untapped-matters-self",
        "Only works while this permanent itself is untapped, losing its effect once tapped.",
    ),
    (
        "life-and-death-trigger-self",
        "Triggers an effect both when it enters the battlefield and when it dies.",
    ),
    (
        "abrade",
        "A modal spell that either deals damage to a creature or destroys an artifact.",
    ),
    (
        "protects-enchantment",
        "Shields your permanents from removal, such as with phasing, protection, or indestructible.",
    ),
    (
        "typal-faerie",
        "Rewards you for controlling Faeries or cares about Faerie creatures.",
    ),
    (
        "synergy-gate",
        "Rewards you for controlling Gates or helps you find and play them.",
    ),
    (
        "gives-uncounterable",
        "Makes spells you control unable to be countered.",
    ),
    (
        "impulse-permanent",
        "Digs into the top of your library and grabs a permanent for your hand or the battlefield.",
    ),
    (
        "sneak-permanent",
        "Lets you put a permanent card from your hand onto the battlefield without casting it.",
    ),
    (
        "mirrored-knight",
        "A knight card with a flavor-matched rival knight elsewhere in Magic's card pool.",
    ),
    (
        "buff-mana",
        "Produces mana that grants an extra bonus when spent on a specific kind of spell.",
    ),
    (
        "synergy-battle",
        "Cares about battles, the permanent type you attack to defeat and flip.",
    ),
    (
        "impulsive-draw",
        "Exiles cards off the top of your library that you may play only until end of turn.",
    ),
    (
        "game-name",
        "A card whose name is a nod to gaming terminology.",
    ),
    (
        "unique-enchant-target",
        "An aura that can only enchant an unusual target, like a specific card type or land.",
    ),
    (
        "alt-commander",
        "A legendary creature printed as a backup commander alongside a preconstructed deck's face commander.",
    ),
    (
        "wingman",
        "A flying creature that grants flying to another creature whenever it attacks.",
    ),
    (
        "fling",
        "Lets you sacrifice a creature to deal damage equal to its power to any target.",
    ),
    (
        "tutor-land-basic-plains",
        "Searches your library for a basic Plains card.",
    ),
    (
        "shares-name-with-a-set",
        "A card whose name matches the name of a Magic set.",
    ),
    (
        "demilich-effect",
        "Exiles a card from a graveyard and lets you cast a copy of it, usually just once.",
    ),
    (
        "synergy-exile-cast",
        "Rewards you for casting spells from exile.",
    ),
    (
        "counterspell-sacrifice",
        "A creature you sacrifice as a cost to counter a spell.",
    ),
    (
        "hate-damaged",
        "Removal that targets a creature which was already dealt damage this turn.",
    ),
    (
        "tapped-matters-self",
        "A permanent whose own ability triggers or changes based on whether it's tapped.",
    ),
    (
        "synergy-desert",
        "Cares about Deserts you control or have in your graveyard.",
    ),
    (
        "players-outside-game-matter",
        "Involves someone outside the game, like asking a bystander to make a choice.",
    ),
    (
        "tutor-land-swamp",
        "Searches your library for a Swamp card and puts it into your hand or battlefield.",
    ),
    (
        "off-turn-casting-matters",
        "Rewards casting a spell during a turn that isn't the caster's own.",
    ),
    (
        "alternate-loss-condition",
        "Sets up a new way you could lose the game, separate from running out of life.",
    ),
    (
        "alternate-cost-sacrifice",
        "Lets you sacrifice a permanent instead of paying some or all of the spell's mana cost.",
    ),
    (
        "synergy-trample",
        "Rewards or boosts creatures you control that have trample.",
    ),
    (
        "wind-drake-with-set-s-mechanic",
        "A 2/2 flyer for three mana that showcases one of its set's signature mechanics.",
    ),
    ("mana-dork-egg", "A creature you can sacrifice to add mana."),
    (
        "force-blocker",
        "Forces a creature to block this turn whether its controller wants to or not.",
    ),
    (
        "provoke-lite",
        "Forces a target creature to block this combat, without untapping it first.",
    ),
    (
        "counterspell-instant",
        "Counters a target instant or sorcery spell.",
    ),
    (
        "hate-token",
        "Damages, destroys, or otherwise punishes tokens specifically.",
    ),
    (
        "lure",
        "Forces all creatures able to block this creature to do so.",
    ),
    (
        "secretly-choose",
        "Has players choose or vote in secret, then reveal simultaneously.",
    ),
    (
        "synergy-color-each",
        "Scales its effect based on how many colors a permanent or card has.",
    ),
    (
        "synergy-deathtouch",
        "Rewards you for controlling or attacking with deathtouch creatures.",
    ),
    (
        "trumpet-blast",
        "An instant that boosts the power of your creatures until end of turn.",
    ),
    (
        "synergy-dice",
        "Rewards you for rolling dice or makes dice rolls better.",
    ),
    (
        "silence",
        "Stops a player from casting spells for a period of time.",
    ),
    (
        "block-additional",
        "Lets a creature block more than one attacker in combat.",
    ),
    (
        "typal-demon",
        "Cares about Demon creatures you control or interacts with the Demon type.",
    ),
    (
        "delve",
        "Lets you exile cards from your graveyard to pay for part of this spell's cost.",
    ),
    (
        "damage-increaser",
        "Makes your damage sources deal extra damage on top of what they'd normally deal.",
    ),
    (
        "illusion-ability",
        "Sacrifices itself whenever it becomes the target of a spell or ability.",
    ),
    (
        "hungry-demon",
        "Forces you to sacrifice a creature, usually each upkeep, unless you meet some condition.",
    ),
    (
        "vivid",
        "Scales its effect with the number of colors among permanents you control.",
    ),
    (
        "theft-mass",
        "Lets you gain control of several permanents at once.",
    ),
    (
        "typal-squirrel",
        "Cares about Squirrel creatures you control.",
    ),
    (
        "mm-counters-matter",
        "Cares about creatures having -1/-1 counters on them.",
    ),
    (
        "gives-fear",
        "Grants fear to a creature, so it can only be blocked by artifact or black creatures.",
    ),
    (
        "turn-face-up-trigger",
        "Triggers an effect when a face-down permanent is turned face up.",
    ),
    (
        "quote-name",
        "A card whose name is taken from an existing quote or phrase.",
    ),
    (
        "alternate-equip-cost",
        "Equipment with an equip cost other than plain mana, or an extra alternate equip cost.",
    ),
    (
        "typal-non-human",
        "Cares about or boosts creatures that are not Humans.",
    ),
    (
        "kismet-effect",
        "Makes certain permanents, often your opponents', enter the battlefield tapped.",
    ),
    (
        "typal-spider",
        "Cares about or boosts creatures of the Spider type.",
    ),
    (
        "three-letter-name",
        "A card whose name is exactly three letters long.",
    ),
    (
        "tokenlink",
        "Creates a number of tokens equal to the combat damage dealt.",
    ),
    (
        "creatureland",
        "A land that can turn itself into a creature.",
    ),
    (
        "hate-vehicle",
        "Targets or destroys Vehicles as well as creatures.",
    ),
    (
        "lifelink-counter",
        "Puts a counter on a creature that grants it lifelink.",
    ),
    (
        "cr-107-3f-x-card",
        "Lets you choose the value of X when a spell or ability doesn't otherwise define it.",
    ),
    (
        "synergy-dungeon",
        "Rewards you for venturing into or completing a dungeon.",
    ),
    (
        "counterspell-sorcery",
        "Counters a target instant or sorcery spell.",
    ),
    (
        "old-lifelink",
        "Gains you life equal to damage dealt, via a trigger instead of the lifelink keyword.",
    ),
    (
        "tutor-land-island",
        "Searches your library for a card with the Island land type.",
    ),
    (
        "graveyard-fuel-artifact",
        "Spends or cares about artifact cards in your graveyard.",
    ),
    (
        "synergy-shrine",
        "Rewards you for controlling multiple Shrines.",
    ),
    (
        "synergy-theft",
        "Rewards you for controlling or casting cards you don't own.",
    ),
    (
        "tunneling",
        "Makes a creature with power 2 or less unblockable this turn.",
    ),
    (
        "copy-equipment",
        "Creates a token that's a copy of a permanent.",
    ),
    (
        "maro",
        "Has power and toughness equal to the number of cards in a hand.",
    ),
    (
        "storm-count-matters",
        "Grows stronger or gains bonus effects based on how many spells have been cast this turn.",
    ),
    (
        "sunburst",
        "Enters with a counter for each color of mana spent to cast it.",
    ),
    (
        "landfall-other",
        "Triggers whenever a land enters the battlefield, even one an opponent controls.",
    ),
    (
        "lands-in-graveyard-matter",
        "Cares about land cards sitting in a graveyard, often getting stronger or returning them.",
    ),
    (
        "removal-aura",
        "Destroys, exiles, or bounces an Aura, freeing whatever it was attached to.",
    ),
    (
        "donate-rampant-growth",
        "Ramps another player, even an opponent, fetching lands onto the battlefield.",
    ),
    (
        "typal-kithkin",
        "Cares about Kithkin creatures, rewarding you for casting or controlling them.",
    ),
    (
        "discard-outlet-creature",
        "Lets you discard a creature card as a cost to pay for an ability or spell.",
    ),
    (
        "discard-symmetrical",
        "Makes every player discard a card, not just your opponents.",
    ),
    (
        "synergy-lesson",
        "Cares about Lesson cards you cast or have in your graveyard.",
    ),
    (
        "deck-requirement",
        "Playable, or usable as your commander or companion, only if your deck meets a building restriction.",
    ),
    (
        "synergy-face-down",
        "Cares about creatures you put onto the battlefield face down, like those from manifest or morph.",
    ),
    (
        "synergy-vigilance",
        "Rewards or grants vigilance on creatures you control.",
    ),
    (
        "art-matters",
        "Cares about what's depicted in a card's artwork.",
    ),
    (
        "hate-typal-wall",
        "Lets a creature ignore Walls when attacking, or destroys your opponents' Walls.",
    ),
    (
        "hate-island",
        "Punishes Islands and their controllers by locking, damaging, or bouncing them.",
    ),
    (
        "playtest-forecast",
        "A playtest card that previewed a mechanic later released in a real set.",
    ),
    (
        "hate-color-share",
        "Affects or punishes cards and permanents that share a color with another object.",
    ),
    (
        "hate-activation",
        "Taxes, punishes, or shuts down activated abilities other than mana abilities.",
    ),
    (
        "hate-aura",
        "Removes, moves, or punishes Auras attached to permanents.",
    ),
    (
        "gives-evasion",
        "Grants a creature an ability that makes it harder or impossible to block.",
    ),
    (
        "indestructible-counter",
        "Puts an indestructible counter on a creature to keep it safe from destruction.",
    ),
    (
        "typal-lupine",
        "Cares about or boosts creatures that are Wolves or Werewolves.",
    ),
    (
        "gains-reach",
        "Gains reach itself, letting it block creatures with flying.",
    ),
    (
        "tutor-land-basic-forest",
        "Searches your library for a basic Forest card and puts it into play or your hand.",
    ),
    (
        "naya-ferocious",
        "Cares about creatures with power 5 or greater.",
    ),
    (
        "type-errata-cephalid",
        "A creature once typed Cephalid, now an Octopus by errata.",
    ),
    (
        "color-choose-land",
        "A land that lets you choose a color as it enters, then taps for mana of that color.",
    ),
    (
        "stalking",
        "A creature that can't be blocked by more than one creature.",
    ),
    (
        "keyword-soup",
        "A card that gains or lists most of its set's keyword abilities.",
    ),
    (
        "tutor-to-exile",
        "Searches your library for a card and exiles it for you to use later.",
    ),
    (
        "outnumber",
        "Deals damage equal to the number of creatures you control.",
    ),
    (
        "theft-land",
        "Takes control of an opponent's land, or trades lands between players.",
    ),
    (
        "bring-your-own-crew",
        "A vehicle or spacecraft that creates its own creature token to crew or station it.",
    ),
    (
        "wheel",
        "Has you discard your hand, then draw that many cards or more.",
    ),
    (
        "synergy-room",
        "Triggers a bonus whenever an enchantment you control enters or you fully unlock a Room.",
    ),
    (
        "persist",
        "Returns this creature to the battlefield with a -1/-1 counter when it dies, if it had none.",
    ),
    (
        "synergy-first-strike",
        "Rewards or grants abilities to your creatures that have first strike.",
    ),
    (
        "rampage",
        "Gets bonus power and toughness for each creature blocking it beyond the first.",
    ),
    (
        "recycle",
        "Lets you sacrifice a permanent to return a card from your graveyard to the battlefield.",
    ),
    (
        "leveler",
        "Grows stronger and gains new abilities as you pay to add level counters to it.",
    ),
    (
        "devour",
        "Lets this creature sacrifice creatures as it enters to gain +1/+1 counters for each one.",
    ),
    (
        "damage-prevention-planeswalker",
        "Prevents damage that would be dealt to you and the permanents you control, including planeswalkers.",
    ),
    (
        "special-action",
        "Offers a special action you take without using the stack, such as paying a cost to avoid an effect.",
    ),
    (
        "typal-treefolk",
        "Cares about or rewards controlling Treefolk creatures.",
    ),
    (
        "impulse-enchantment",
        "Lets you dig through your library and grab an enchantment card into your hand.",
    ),
    (
        "text-change",
        "Directly rewrites a card's rules text, such as its creature types or numbers.",
    ),
    (
        "cranial-plating",
        "Gets stronger based on how many artifacts or Equipment you control.",
    ),
    (
        "counts-as-a-type",
        "An older card using deprecated wording that treats it as having an extra creature type.",
    ),
    (
        "tutor-self",
        "Searches your library for another copy of itself and puts it into your hand or onto the battlefield.",
    ),
    (
        "tap-fuel-land",
        "Taps a land to pay a cost other than a mana cost.",
    ),
    (
        "useless-in-singleton-formats",
        "Relies on having multiple copies of the same card name, so it does little in Singleton formats like Commander.",
    ),
    (
        "typal-beast",
        "Cares about or boosts creatures with the Beast creature type.",
    ),
    (
        "cycle-mm3-draft-signpost",
        "A two-color draft signpost card from Modern Masters 2017 pointing toward that color pair's archetype.",
    ),
    (
        "counter-fuel-oil",
        "Puts oil counters on a permanent, then spends them to trigger a powerful effect.",
    ),
    (
        "synergy-suspend",
        "Rewards or interacts with cards that have or gain suspend and its time counters.",
    ),
    (
        "battalion",
        "Triggers a bonus whenever this creature attacks alongside at least two other creatures.",
    ),
    (
        "o-ring-with-set-mechanic",
        "Exiles a permanent temporarily, using another set's mechanic to power the effect.",
    ),
    (
        "restock-any",
        "Returns any cards from the graveyard to the library, regardless of card type.",
    ),
    (
        "remove-from-combat",
        "Removes a creature from combat so it deals and takes no combat damage this turn.",
    ),
    (
        "even-odd-matters",
        "Cares whether a value like mana value or counters is even or odd, with zero counting as even.",
    ),
    (
        "processing",
        "Lets you put an opponent's exiled card into their graveyard for an added benefit.",
    ),
    (
        "type-errata-lord",
        "A creature type tag for cards that once had the now-defunct Lord creature type.",
    ),
    (
        "improvise",
        "Lets you tap your artifacts to help pay this spell's generic mana cost.",
    ),
    (
        "impulse-cast",
        "Lets you cast a spell from the top of your library without paying its mana cost.",
    ),
    (
        "scene",
        "A cosmetic scene-art printing with no effect on how the card plays.",
    ),
    (
        "creates-oracle-token",
        "Creates a token that's a full copy of a specific named card.",
    ),
    (
        "synergy-double-strike",
        "Cares about creatures that have double strike, often among other keywords.",
    ),
    (
        "type-addition-noble",
        "A creature type tag for cards that gained the Noble creature subtype by errata.",
    ),
    (
        "random-card",
        "Involves a card chosen at random, often a copy from a preset list or the whole pool.",
    ),
    (
        "synergy-defender",
        "Rewards you for controlling creatures with defender.",
    ),
    (
        "harmonic",
        "Gets a bonus if you control both an artifact and an enchantment.",
    ),
    (
        "bounceable-aura",
        "An Aura that can return itself to its owner's hand.",
    ),
    (
        "synergy-exiling",
        "Triggers or rewards you whenever one or more cards are put into exile.",
    ),
    (
        "type-addition-sorcerer",
        "A creature that also has the Sorcerer creature type.",
    ),
    (
        "typal-rogue",
        "Rewards you for controlling multiple Rogues.",
    ),
    (
        "cost-reducer-activated-ability",
        "Reduces the cost to activate one or more activated abilities.",
    ),
    ("copy-aura", "Creates a token that's a copy of a permanent."),
    (
        "typal-saproling",
        "Creates, cares about, or sacrifices Saproling creature tokens.",
    ),
    (
        "sleeping-enchantment",
        "An enchantment that permanently becomes a creature once a trigger condition is met.",
    ),
    (
        "seek-to-battlefield",
        "Seeks a card from your library and puts it onto the battlefield.",
    ),
    (
        "serpent-like",
        "A creature that can't attack unless a specific condition, like an Island, is met.",
    ),
    (
        "sneaky-self-trigger",
        "Has an easy-to-miss ability that quietly untaps or benefits itself off a common event.",
    ),
    (
        "auto-buyback",
        "Returns itself to your hand after resolving, usually once a condition is met.",
    ),
    (
        "absorb",
        "A static ability that prevents a set amount of damage dealt to a permanent or player.",
    ),
    (
        "damage-prevention-player",
        "Prevents damage that would be dealt to a player.",
    ),
    (
        "mana-spent-matters",
        "Cares about the amount, color, or source of mana spent to cast or activate something.",
    ),
    (
        "theft-nonland",
        "Lets you take control of or cast a nonland permanent or card another player owns.",
    ),
    (
        "borrow-ability",
        "Gives itself or another creature a keyword ability only if something else already has it.",
    ),
    (
        "buttstrike",
        "Makes a creature assign combat damage equal to its toughness instead of its power.",
    ),
    (
        "bribery",
        "Offers each opponent a benefit, giving you a bigger reward for each one who accepts.",
    ),
    (
        "gives-player-hexproof",
        "Gives you hexproof so opponents can't target you with spells or abilities.",
    ),
    (
        "take-the-initiative",
        "Lets you take the initiative, venturing deeper into the Undercity on your upkeep.",
    ),
    (
        "synergy-menace",
        "Cares about creatures with menace, rewarding or copying that ability across your team.",
    ),
    (
        "un-keyword",
        "A card with a joke keyword mechanic found only in the Un-sets.",
    ),
    (
        "removes-mm-counters-self",
        "Removes -1/-1 counters from itself only, not from other creatures.",
    ),
    (
        "tutor-instant",
        "Searches your library for an instant card and puts it into your hand or exile.",
    ),
    (
        "repeatable-enchantment-tokens",
        "Creates enchantment or enchantment creature tokens again and again.",
    ),
    (
        "graveyard-order-matters",
        "Cares about the order of cards in a graveyard, or rearranges it.",
    ),
    (
        "typal-cat",
        "Rewards you for controlling Cat creatures or turns creatures into Cats.",
    ),
    (
        "draft-matters",
        "Has an effect tied to how you draft it or interacts with the booster draft itself.",
    ),
    (
        "drain-strength",
        "Makes one creature bigger while making another creature smaller.",
    ),
    (
        "old-damage-deathtouch",
        "Destroys any creature it deals combat damage to, an older way of writing deathtouch.",
    ),
    (
        "hate-named",
        "Lets you name a specific card to counter, discard, or otherwise shut down.",
    ),
    (
        "quick-enchant",
        "Attaches an Aura to a permanent through an ability instead of casting it normally.",
    ),
    (
        "typal-phyrexian",
        "Cares about or boosts Phyrexian creatures you control.",
    ),
    (
        "synergy-lifelink",
        "Cares about lifelink among a group of keyword abilities your creatures have.",
    ),
    (
        "synergy-enchantment-creature",
        "Cares about or boosts enchantment creatures you control.",
    ),
    (
        "pariah",
        "Redirects damage that would be dealt to you onto a creature instead.",
    ),
    (
        "seek-creature",
        "Puts a random creature card from your library into your hand.",
    ),
    (
        "synergy-warp",
        "Cares about spells cast for their warp cost.",
    ),
    (
        "mathy-name",
        "A card whose name references a mathematical concept.",
    ),
    (
        "conjure-spellbook",
        "Conjures a card from this card's own fixed spellbook, not from your deck.",
    ),
    (
        "fact-or-fiction",
        "Splits revealed cards into piles for card advantage, with an opponent choosing or making the piles.",
    ),
    (
        "transferrable-aura",
        "An aura with its own built-in way to move itself onto a different permanent.",
    ),
    (
        "removal-equipment",
        "Removal aimed specifically at destroying or exiling Equipment.",
    ),
    (
        "synergy-color-choose",
        "Choose a color as it enters, then reward cards or mana of that color.",
    ),
    (
        "x-cost-matters",
        "Cares about spells or costs that include X, rewarding or enabling them.",
    ),
    (
        "synergy-scry",
        "Triggers an additional effect whenever you scry.",
    ),
    (
        "life-divider-you",
        "Makes a player lose, draw, discard, or sacrifice roughly half of something, rounded up.",
    ),
    (
        "reanimate-matters",
        "Triggers when a creature card enters or leaves a graveyard, including reanimation.",
    ),
    (
        "hate-multicolor",
        "Punishes or gets protection from multicolored permanents and spells.",
    ),
    (
        "neo-regenerate",
        "Grants a creature indestructible and taps it, the modern version of regenerate.",
    ),
    (
        "unique-noncreature-token",
        "Creates a specific, named noncreature token instead of a generic one.",
    ),
    (
        "synergy-name-sticker",
        "Cares about permanents' names, including name stickers placed on them.",
    ),
    (
        "four-plus-creature-types",
        "A non-Changeling creature that has four or more creature types.",
    ),
    (
        "discard-to-library",
        "Puts a card from a target player's hand onto their library.",
    ),
    (
        "conjure-artifact",
        "Creates a specific artifact card from outside the game in your hand or on the battlefield.",
    ),
    (
        "typal-angel",
        "Rewards you for having Angels or helps your Angels specifically.",
    ),
    (
        "surge",
        "Costs less or grants a bonus if you or a teammate already cast another spell this turn.",
    ),
    (
        "poison-opponents",
        "Gives poison counters to opponents directly, not through infect or toxic combat damage.",
    ),
    (
        "reanimate-aura",
        "Returns an Aura card from a graveyard to the battlefield, attached to a creature.",
    ),
    (
        "un-forecast",
        "An Un-set card whose mechanic later paved the way for a black-bordered one.",
    ),
    (
        "trample-counter",
        "Places a counter on a creature that grants it trample.",
    ),
    (
        "deanimate",
        "Turns a creature into a noncreature permanent, stripping its creature type and often its abilities.",
    ),
    (
        "expertise",
        "Lets you cast a spell from your hand without paying its mana cost, usually tied to another effect.",
    ),
    (
        "hate-low-toughness",
        "Punishes creatures that have low toughness.",
    ),
    (
        "crewless-vehicle",
        "A Vehicle that can become a creature without needing to be crewed.",
    ),
    (
        "deathtouch-counter",
        "Places a counter on a creature that grants it deathtouch.",
    ),
    (
        "magic-term-name",
        "A card whose name is itself a Magic term or piece of community slang.",
    ),
    (
        "alternative-crewing",
        "A Vehicle that has crew but can also become a creature another way.",
    ),
    (
        "ablative-armor",
        "Prevents damage to this creature by removing a counter instead of taking it.",
    ),
    (
        "synergy-reach",
        "Grants or counts reach along with other keyword abilities found among your creatures.",
    ),
    (
        "cost-reducer-colored-mana",
        "Reduces only the colored mana you pay for spells you cast.",
    ),
    (
        "hate-artifact-creature",
        "Punishes or gets around artifact creatures.",
    ),
    (
        "tutor-artifact-equipment",
        "Searches your library for an Equipment card.",
    ),
    (
        "reanimate-nonland",
        "Returns nonland permanent cards from your graveyard to the battlefield.",
    ),
    (
        "converge",
        "Scales its effect based on the number of colors of mana spent to cast it.",
    ),
    (
        "toughness-matters-self",
        "A creature whose own toughness fuels its abilities or effects.",
    ),
    (
        "preexisting-dnd-background",
        "A card named after a Dungeons and Dragons character background that already existed.",
    ),
    (
        "doctor-who-episode-name",
        "A card named after an episode of Doctor Who.",
    ),
    (
        "gains-shroud",
        "Gains shroud itself, so it can't be targeted by spells or abilities.",
    ),
    (
        "synergy-creatureland",
        "Rewards or boosts lands that have become creatures.",
    ),
    (
        "radiate",
        "Copies a single-target spell and points the copy at something else it could hit.",
    ),
    (
        "artist-matters",
        "Cares about the artist who illustrated a card, rewarding or punishing shared artwork.",
    ),
    (
        "spell-with-no-casting-cost",
        "A card with no mana cost printed on it, usually cast only through suspend or a similar alternate cost.",
    ),
    (
        "instant-speed-discard",
        "Makes a player discard at instant speed, letting you strip a card before they can use it.",
    ),
    (
        "skip-draw-step",
        "Makes a player skip their draw step, often in exchange for another benefit.",
    ),
    (
        "legends-retold",
        "Part of a special set-booster cycle reimagining classic legendary creatures from the original Legends set.",
    ),
    (
        "cycle-2xm-r-two-color",
        "Belongs to a two-color rare cycle reprinted in Double Masters.",
    ),
    (
        "sneak-artifact",
        "Puts an artifact card onto the battlefield without casting it.",
    ),
    (
        "tokenfall",
        "Triggers an effect whenever a token enters the battlefield, often copying it.",
    ),
    (
        "impulse-instant",
        "Exiles cards from your library and lets you cast an instant from among them.",
    ),
    (
        "renown",
        "Gets +1/+1 counters and becomes renowned the first time it hits a player in combat.",
    ),
    (
        "impulse-sorcery",
        "Exiles cards from your library and lets you cast a sorcery from among them.",
    ),
    (
        "offcolor-mana-generation",
        "Produces mana in a color other than the card's own color.",
    ),
    (
        "cycle-2x2-draft-signpost",
        "A two-color gold card that steers drafters toward its color pair's archetype.",
    ),
    (
        "cycle-war-r-two-color",
        "A two-color rare from War of the Spark, one per guild pair in the cycle.",
    ),
    (
        "fateseal",
        "Lets you look at an opponent's top library card and choose to leave it or put it on the bottom.",
    ),
    (
        "offspring-token",
        "A 1/1 token copy created by the offspring keyword when its source enters.",
    ),
    (
        "cycle-mm2-draft-signpost",
        "A two-color signpost card from Modern Masters 2 that points drafters toward a guild archetype.",
    ),
    (
        "impulsive-curiosity",
        "Exiles your top card when a creature deals combat damage to a player, letting you play it briefly.",
    ),
    (
        "theft",
        "Lets you take control of or use resources that belong to an opponent.",
    ),
    (
        "cards-in-exile-matter",
        "Cares about the cards sitting in exile.",
    ),
    (
        "cycle-fin-draft-signpost",
        "A two-color legendary signpost creature that signals and rewards a specific draft archetype.",
    ),
    (
        "cycle-dsk-draft-signpost",
        "A two-color signpost card that signals and rewards a specific draft archetype.",
    ),
    (
        "blood-moon-effect",
        "Strips lands of their types or abilities, often turning nonbasic lands into basics.",
    ),
    (
        "cycle-ltr-draft-signpost",
        "A two-color signpost card that signals and rewards a specific draft archetype.",
    ),
    (
        "cycle-ltr-r-two-color",
        "A rare two-color legendary creature built around one of the set's draft archetypes.",
    ),
    (
        "table-order-matters",
        "An effect that cares about seating or turn order, like choosing left or right.",
    ),
    (
        "life-divider-opponent",
        "Makes a player lose a fraction of their life, like half or a third.",
    ),
    (
        "gives-flashback",
        "Grants flashback to an instant or sorcery in your graveyard so you can cast it once.",
    ),
    (
        "synergy-indestructible",
        "Cares about creatures with indestructible, often granting or counting that keyword among them.",
    ),
];

/// Flatten [`ORACLE_TAG_DESCRIPTIONS`] into parallel `(slug, description)` arrays
/// for the overlay `unnest`.
pub fn description_pairs() -> (Vec<String>, Vec<String>) {
    let mut slugs = Vec::with_capacity(ORACLE_TAG_DESCRIPTIONS.len());
    let mut descriptions = Vec::with_capacity(ORACLE_TAG_DESCRIPTIONS.len());
    for (slug, description) in ORACLE_TAG_DESCRIPTIONS {
        slugs.push((*slug).to_string());
        descriptions.push((*description).to_string());
    }
    (slugs, descriptions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    /// A duplicate slug would make one authored description silently shadow the
    /// other in the overlay `UPDATE`; forbid it.
    #[test]
    fn slugs_are_unique() {
        let mut seen = HashSet::new();
        for (slug, _) in ORACLE_TAG_DESCRIPTIONS {
            assert!(seen.insert(*slug), "duplicate authored slug: {slug}");
        }
    }

    /// No blank descriptions (a blank would overwrite Scryfall's with nothing).
    #[test]
    fn descriptions_are_non_blank() {
        for (slug, desc) in ORACLE_TAG_DESCRIPTIONS {
            assert!(!desc.trim().is_empty(), "blank description for {slug}");
        }
    }
}
