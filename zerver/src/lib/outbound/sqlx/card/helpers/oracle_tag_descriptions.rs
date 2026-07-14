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
