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
    ("removal-destroy", "Removal that destroys what it hits."),
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
    (
        "synergy-artifact",
        "Cares about or benefits from artifacts, usually the ones you control.",
    ),
    ("gives-haste", "Grants haste to a creature."),
    (
        "tapper-creature",
        "Taps one or more creatures so they can't attack or block.",
    ),
    (
        "utility-land",
        "A land with a useful ability beyond making mana.",
    ),
    (
        "synergy-instant",
        "Rewards casting instants, and usually sorceries too.",
    ),
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
    (
        "removal-exile",
        "Removal that exiles what it hits instead of destroying it.",
    ),
    ("untapper-creature", "Untaps one or more creatures."),
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
        "A card whose name references a specific named character.",
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
    ("repeatable-pure-draw", "Repeatably draws extra cards."),
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
    (
        "power-boost-to-all",
        "Raises the power of your creatures, often only those of a chosen type or subgroup.",
    ),
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
        "Drains life from or deals damage to each opponent, sometimes every player.",
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
        "Has an ability that triggers when a creature dies.",
    ),
    ("burst-draw", "Draws several cards at once."),
    (
        "repeatable-sacrifice-outlet",
        "A repeatable way to sacrifice your own permanents.",
    ),
    (
        "type-addition-human",
        "A creature whose type line was updated to include the Human creature type.",
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
        "A joke or Un-set style card that is mechanically functional enough to have been printed in a normal black-bordered set.",
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
        "Casts a spell as another spell or ability resolves, rather than at its normal time.",
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
        "A named token creature with its own defined characteristics.",
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
        "Removal that destroys, exiles, or bounces an artifact.",
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
        "Gives you a bit of extra mana, usually from a small mana rock.",
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
        "A cheap permanent, usually an artifact, meant to be sacrificed for a payoff.",
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
        "A damage-dealing card that also carries the signature mechanic of the set it debuted in.",
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
        "Lets a card be cast from exile, often without paying its mana cost.",
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
        "Answers or shuts down a specific keyword or mechanic from its own set.",
    ),
    (
        "cast-trigger-self",
        "Triggers an ability the moment you cast it, before it resolves.",
    ),
    (
        "shapechange",
        "Sets a creature's base power and toughness, sometimes also changing its types.",
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
        "Draws you a card whenever a creature deals combat damage to a player.",
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
        "Removal that answers an enchantment by destroying, exiling, or bouncing it.",
    ),
    (
        "landfall",
        "Triggers an effect whenever a land enters the battlefield under your control.",
    ),
    (
        "protects-all",
        "Shields several of your permanents at once, such as granting them indestructible or hexproof or flickering them.",
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
        "A group hug effect where you choose which players get the benefits, sometimes at the excluded players' expense.",
    ),
    (
        "removal-permanent",
        "Removal that can destroy, exile, or bounce any type of permanent, not just creatures.",
    ),
    (
        "dnd-character",
        "Depicts a character or creature from Dungeons and Dragons in its name or text.",
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
        "Lets you cast or play cards from another player's hand, library, or graveyard, usually without paying their cost.",
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
        "Ties a discard effect to a mechanic from that card's set, like flashback, spectacle, or evoke.",
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
        "Returns itself from your graveyard to your hand.",
    ),
    (
        "synergy-legendary",
        "Cares about legendary creatures or permanents, often rewarding you for controlling them.",
    ),
    (
        "restricted-mana",
        "Adds mana that can only be spent on certain things.",
    ),
    (
        "mill-opponent",
        "Depletes an opponent's library by milling or exiling cards from the top of it.",
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
        "Puts +1/+1 counters on each creature a player controls at once, usually your own board.",
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
        "Gets better the more creatures are on the battlefield, usually your own.",
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
        "Exiles creature cards from a graveyard to fuel its spells and abilities.",
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
        "Cares about the cards sitting in a graveyard.",
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
    ("giant-growth", "Boosts a creature's power and toughness."),
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
        "Has a creature deal damage equal to its power to another creature or planeswalker, which deals none back.",
    ),
    (
        "type-addition-from-none",
        "A card given a creature type it originally lacked, whether through errata or by animating into a creature.",
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
        "Chooses a permanent, player, card, or card name without targeting it, so hexproof and shroud can't stop it.",
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
        "Lets you cast a card straight out of your graveyard.",
    ),
    (
        "prevent-attack",
        "Stops a creature from attacking, and usually from blocking too.",
    ),
    (
        "impulse-creature",
        "Digs through the top of your library to find a creature card.",
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
        "Makes a target player reveal their hand so you pick a card to discard, exile, or bottom.",
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
        "Stops a permanent from being regenerated, usually while destroying it.",
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
        "Grants permanents a new activated ability that requires tapping them.",
    ),
    (
        "consult-cast",
        "Exiles cards from the top of a library until you hit one you may cast, often for free.",
    ),
    (
        "humble",
        "Strips a creature's abilities and shrinks its stats as a form of removal.",
    ),
    (
        "burn-player-each",
        "Deals damage to each player, and often to each creature as well.",
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
        "Gives a player poison counters, usually when a creature it controls or creates deals them damage.",
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
        "Removes a permanent but its controller gets a replacement permanent, often a token, in return.",
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
        "Exiles cards from libraries or hands and lets you cast or play them, effectively stealing from other players.",
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
        "Cares that a player, usually an opponent, lost life this turn, not just took damage.",
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
        "Puts a spell or permanent into play without paying its mana cost, whether by casting it for free, paying an alternate cost, or putting it straight onto the battlefield.",
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
        "A creature with restrictions on whether or what it can block.",
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
        "Lets you tap creatures whose power fuels or enables an effect.",
    ),
    (
        "gives-castable-from-graveyard",
        "Lets you cast a card straight from a graveyard instead of your hand.",
    ),
    (
        "mulch",
        "Looks at cards from the top of your library, puts one or more into your hand, and mills the rest.",
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
    (
        "loot",
        "Draws you one or more cards, then makes you discard one or more cards.",
    ),
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
        "Makes every permanent of one kind gain an extra type all at once, so they all count as something new in addition to what they already are.",
    ),
    (
        "mana-rock-with-set-s-mechanic",
        "A mana rock that also plugs into that set's signature mechanic.",
    ),
    (
        "combat-neutral-damage-trigger",
        "Triggers when it deals damage, whether or not that damage is dealt in combat.",
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
        "Exiles a permanent and returns it to the battlefield later, most often at the next end step, rather than right away.",
    ),
    (
        "turn-face-up-trigger-self",
        "Triggers an effect when this permanent is turned face up from morph, megamorph, or disguise.",
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
        "Cares about how many different card types are in your graveyard, often getting stronger or gaining abilities once four or more are present.",
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
        "A creature whose type line originally used the old 'Summon' wording before errata updated it to the modern creature type format.",
    ),
    (
        "commander-set-booster-cards",
        "A card that debuted in a Commander set and was only available in that set's boosters.",
    ),
    (
        "full-refund",
        "Refunds much or all of the mana you spent to cast it, usually by untapping your lands or producing extra mana.",
    ),
    (
        "conjure-to-hand",
        "Conjures a new card directly into a player's hand.",
    ),
    (
        "minigame",
        "Involves an unusual game challenge like a vote, bet, guess, or timed task, sometimes with people outside the game.",
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
        "Returns a creature to the battlefield with haste, then removes it at end of turn.",
    ),
    (
        "pacifism",
        "Keeps a creature from attacking, and usually from blocking, without destroying it.",
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
        "Chooses a color and answers it, most often by granting protection from that color.",
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
        "Answers or punishes instant and sorcery spells, or steals value from them.",
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
        "A card whose name is made up of its own subtypes.",
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
        "A joke Un-set card (silver-bordered or Acorn-stamped) whose type line uses a creature type or card type found only in those sets.",
    ),
    (
        "trigger-from-exile",
        "Has an ability that triggers while the card sits in exile, like suspend.",
    ),
    (
        "sweeper-graveyard",
        "Empties one or more graveyards at once, usually by exiling them.",
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
        "Searches your library for a card by mana value, usually one at or below a set limit.",
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
        "Searches a library for a card with a specific name.",
    ),
    (
        "threaten",
        "Gains control of a creature until end of turn, usually untapping it with haste.",
    ),
    (
        "synergy-arcane",
        "Cares about Arcane spells, either triggering a bonus when you cast a Spirit or Arcane spell or splicing onto Arcane.",
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
        "Punishes, answers, or steals instant and sorcery spells.",
    ),
    (
        "quadratic",
        "Scales with the square of a count, since the same number multiplies against itself.",
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
        "A card whose power and toughness, or the bonus it grants, scale with the number of lands, or a land type, you control.",
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
        "Deals damage to a creature or other target and you gain life equal to the damage dealt.",
    ),
    (
        "life-total-matters-self",
        "Cares about your own life total, often whether it is above or below a threshold.",
    ),
    (
        "lhurgoyf",
        "A card whose power and toughness, or a creature it affects, scale with the number of cards in a graveyard.",
    ),
    (
        "pseudo-fog",
        "Stops an entire combat phase by means other than damage prevention, such as tapping attackers.",
    ),
    (
        "regrowth-sorcery",
        "Returns an instant or sorcery card from your graveyard to your hand.",
    ),
    (
        "cost-increaser",
        "Makes spells or abilities cost more for you or your opponents to cast or activate.",
    ),
    (
        "regrowth-instant",
        "Returns an instant or sorcery card from your graveyard to your hand.",
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
        "Exiles cards from a target player's hand (and sometimes their graveyard) instead of putting them into the graveyard.",
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
        "Lets a permanent enter the battlefield as a copy of another permanent, most often a creature copying another creature.",
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
        "A creature whose power, and often toughness, equals the number of permanents of a certain kind, most often creatures you control.",
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
        "Lets you tap untapped artifacts you control, and often creatures or lands too, to help pay a cost.",
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
        "Lets you discard a card as a cost, with no mana required, to fuel an ability.",
    ),
    (
        "gives-ward",
        "Grants ward to one or more of your permanents, usually creatures, so an opponent must pay a cost to target them or the spell is countered.",
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
        "Taps down a target artifact, and often creatures and lands too, keeping it from untapping or acting.",
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
        "Exiles a nonland permanent and can later return it or let its owner replay it, whether protecting your own or removing an opponent's.",
    ),
    (
        "color-spent-matters",
        "Grants a bonus effect if a specific color of mana was spent to pay for it.",
    ),
    (
        "synergy-basic",
        "Cares about basic lands, whether by controlling, searching for, sacrificing, or spending them.",
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
        "Destroys or otherwise removes an artifact or enchantment while also using a mechanic tied to its set.",
    ),
    (
        "impulse",
        "Lets you look at the top cards of your library and put one or more of them into your hand.",
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
        "Can become an Aura enchantment attached to another permanent, usually via bestow.",
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
        "Doubles a creature's power, sometimes its toughness too, usually until end of turn.",
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
        "Creates one or more creature tokens when a creature dies.",
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
        "Puts a permanent card from a graveyard onto the battlefield, or lets you cast permanents from there.",
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
        "Lets you bring in a card you own from outside the game, putting it into your hand or letting you play it.",
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
        "Has flash, is cast as though it had flash, or grants spells that ability, from before flash was an official keyword.",
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
        "A noncreature permanent that carries a keyword or ability usually found on creatures, like indestructible, flying, or +1/+1 counters.",
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
        "Triggers a punishing or rewarding effect whenever it or a permanent becomes the target of a spell or ability.",
    ),
    (
        "type-errata-viashino",
        "A Lizard creature that once carried the retired Viashino creature type, since folded into Lizard.",
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
        "Rewards you for casting spells that target a creature, or for targeting an opponent, their permanents, or their graveyard.",
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
        "Answers, hinders, or defends against planeswalkers, or rewards attacking them.",
    ),
    (
        "roll-d20",
        "Rolls a twenty sided die, with the result determining the outcome.",
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
        "Lets you keep mana for later, whether as Treasure tokens, counters, or mana that doesn't empty between phases.",
    ),
    (
        "theft-artifact",
        "Lets you take control of an artifact, sometimes only temporarily.",
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
        "Shuts down graveyards, exiling cards, denying graveyard use, or stopping cards from reaching a graveyard at all.",
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
        "Repeatedly digs cards off the top of your library, putting some into your hand and the rest into your graveyard.",
    ),
    (
        "prevent-cast",
        "Stops a player from casting spells or playing cards under certain conditions.",
    ),
    (
        "staple-with-set-s-mechanic",
        "A card that showcases one of its set's featured mechanics.",
    ),
    (
        "counterspell-ability",
        "Counters an activated or triggered ability on the stack.",
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
        "Makes spells you cast, usually creature spells, cost less mana.",
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
        "Destroys, counters, taxes, or otherwise punishes enchantments.",
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
        "Grants shroud to other permanents, so they can't be targeted by spells or abilities.",
    ),
    (
        "synergy-color-share",
        "Rewards you for cards or creatures that share a color with each other.",
    ),
    (
        "scry-like",
        "Lets you look at one or more cards on top of a library and choose whether they stay on top or go elsewhere, similar to scry.",
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
        "Cares about cycling and often discarding, usually triggering a bonus when you cycle or discard a card.",
    ),
    ("untapper-artifact", "Untaps a target artifact."),
    (
        "exalted",
        "Whenever a creature you control attacks alone, it gets +1/+1 until end of turn.",
    ),
    (
        "legendary-team-up",
        "A legendary card, usually a creature, that pairs two named characters on one card.",
    ),
    (
        "copy-legendary",
        "Makes a nonlegendary copy of a permanent, usually as a token but sometimes by turning creatures you control into copies, sidestepping the legend rule.",
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
        "Cares about Pirates, rewarding or interacting with Pirate cards you control or own.",
    ),
    (
        "synergy-modified",
        "Cares about modified creatures, those with a counter, Aura, or Equipment on them.",
    ),
    (
        "parasitic-aura",
        "A harmful Aura that penalizes the enchanted permanent's controller, usually by dealing them damage or making them lose life.",
    ),
    (
        "peek-library",
        "Lets you look at one or more cards from the top of a library without drawing them.",
    ),
    (
        "remove-counters-other",
        "Removes counters from a permanent other than itself, or removes a player's counters.",
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
        "An enchantment, usually black, with a powerful effect and a serious, potentially game-losing drawback.",
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
        "A card whose name is made up entirely of Magic creature types.",
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
        "A Sliver whose ability benefits Slivers you control and stacks with each additional copy.",
    ),
    (
        "leaves-graveyard-trigger",
        "Triggers an effect whenever one or more cards leave a graveyard, usually your own.",
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
        "An Aura whose main effect is changing the enchanted creature's power and toughness.",
    ),
    (
        "unblocked-trigger",
        "Triggers an effect when this creature attacks and isn't blocked.",
    ),
    (
        "bushido",
        "Whenever this creature blocks or becomes blocked, its power and toughness change until end of turn, usually a Bushido +X/+X boost.",
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
        "Digs through the top cards of your library for artifacts, putting them into your hand or onto the battlefield.",
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
        "A modal effect whose options are mirror opposites, like hitting fliers or hitting non-fliers.",
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
        "When it dies, moves its own +1/+1 counters onto another creature.",
    ),
    (
        "synergy-sticker",
        "Puts a sticker on a permanent you own, or cares about stickers.",
    ),
    (
        "unique-protection",
        "Has or grants protection from something unusual, like a chosen player or die rolls, rather than from a color.",
    ),
    (
        "combat-timing-restriction",
        "A spell you can cast only during combat, often only during a specific combat step.",
    ),
    (
        "remove-from-stack",
        "Removes a spell from the stack by exiling it, returning it to its owner's hand, or tucking it into their library, or by ending the turn.",
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
        "A hasty, hard-hitting creature, or a token it creates, that gets sacrificed or exiled at end of turn.",
    ),
    (
        "doesn-t-untap",
        "Stays tapped during your untap step unless something lets you untap it.",
    ),
    (
        "tutor-land-any",
        "Searches your library for any land card, often putting it onto the battlefield.",
    ),
    (
        "mana-egg",
        "An artifact you sacrifice to add mana, often drawing a card or giving another bonus.",
    ),
    (
        "restock-self",
        "Puts itself back into your library so you can draw and cast it again instead of losing it.",
    ),
    (
        "affinity-for-artifacts",
        "Costs 1 less to cast for each artifact you control.",
    ),
    (
        "pridemate",
        "Puts a +1/+1 counter on a creature, often itself, whenever you gain life.",
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
        "Puts a -1/-1 counter on a creature you control, either as a cost or as an enters-the-battlefield drawback.",
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
        "Synergizes with a specific named planeswalker you control, either gaining a bonus itself or supporting that planeswalker.",
    ),
    (
        "restock-creature",
        "Returns a creature card from your graveyard to your library, either on top or shuffled in.",
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
        "A land that costs you life or deals damage to you when you tap it for mana.",
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
        "Exiles cards with a chosen name, usually every copy, from a player's hand, library, and graveyard.",
    ),
    (
        "variable-effect-same-ability",
        "An ability whose effect changes based on a condition, often how many times it has resolved this turn.",
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
        "Makes a player sacrifice, destroy, or exile a permanent unless they pay a cost.",
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
        "Can potentially cost nothing to cast through cost reduction or an alternative cost.",
    ),
    (
        "typal-mount",
        "Cares about or interacts with Mounts and Vehicles you control.",
    ),
    (
        "whirlpool",
        "Shuffles hands and graveyards into libraries, then draws each player a fresh hand.",
    ),
    (
        "young-pyromancer-ability",
        "Creates a creature token whenever you cast an instant, sorcery, or other noncreature spell.",
    ),
    (
        "fog",
        "Prevents all or nearly all combat damage that would be dealt this turn.",
    ),
    (
        "protects-land",
        "Shields lands, or all your permanents, from being destroyed or targeted.",
    ),
    (
        "cycle-ust-functional-variant",
        "An Unstable card printed in multiple versions that share a name and mana cost but have different rules text.",
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
        "A creature that can't attack unless the defending player controls a certain land type, and often must be sacrificed if you control none of that type.",
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
        "A legendary creature with a five-color identity built as a commander to support a set's mechanics or themes.",
    ),
    (
        "retaliate-to-damage",
        "Rewards you or punishes an opponent whenever you take damage.",
    ),
    (
        "hate-wide",
        "Scales with the number of creatures on the battlefield, usually to punish go-wide boards.",
    ),
    (
        "theft-permanent",
        "Takes or swaps control of a permanent, usually an opponent's.",
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
        "Cares about Clerics, counting them or tapping them as a resource to power its effects.",
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
        "Exiles a card from a graveyard and lets you cast a copy of it without paying its mana cost, usually just once.",
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
        "Changes the game's loss rules, adding a new way a player can lose or removing the normal loss from having no life.",
    ),
    (
        "alternate-cost-sacrifice",
        "Lets you sacrifice one or more permanents to help cast a spell, either instead of its mana cost or as an extra cost.",
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
        "Forces a target creature to block this turn if able, without untapping it first.",
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
        "Forces all creatures able to block a given creature, whether itself or one it targets, to do so.",
    ),
    (
        "secretly-choose",
        "Has players make a hidden choice or vote that they reveal later, sometimes all at once.",
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
        "Makes damage sources deal extra damage on top of what they'd normally deal.",
    ),
    (
        "illusion-ability",
        "Sacrifices itself whenever it becomes the target of a spell or ability.",
    ),
    (
        "hungry-demon",
        "Forces you to sacrifice a creature, usually at the beginning of each of your upkeeps.",
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
        "A card with a face whose name is exactly three letters long.",
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
        "Spends or cares about artifact cards in a graveyard.",
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
        "Copies a permanent, usually by creating a token copy of it.",
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
        "Rewards you for having permanents enter face down or turning them face up, as with manifest and morph.",
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
        "Lets a creature attack past Walls, or destroys or disables Walls.",
    ),
    (
        "hate-island",
        "Punishes or answers Islands and their controllers by destroying, locking, damaging, or bouncing them.",
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
        "References a long list of common keyword abilities like flying, first strike, deathtouch, and trample, often granting or counting them.",
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
        "Makes a player discard their hand, then draw that many cards or more.",
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
        "Rewards or grants first strike among your creatures.",
    ),
    (
        "rampage",
        "Gets bonus power and toughness for each creature blocking it beyond the first.",
    ),
    (
        "recycle",
        "Lets you sacrifice a permanent to return a card from your graveyard to the battlefield or your hand.",
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
        "Makes a creature stronger based on how many artifacts you control.",
    ),
    (
        "counts-as-a-type",
        "An older creature that Oracle updates treat as having a creature type such as Wall or Sliver, letting it count for that type's effects.",
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
        "Exiles a permanent until this leaves the battlefield, bundled with a set specific mechanic.",
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
        "Cares about both artifacts and enchantments, often rewarding you for controlling or casting them.",
    ),
    (
        "bounceable-aura",
        "An Aura with a way to return itself to its owner's hand or library.",
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
        "A dormant enchantment that becomes a creature when a condition is met or a cost is paid.",
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
        "Has an easy-to-miss ability that quietly triggers off a common event such as a permanent entering or dying.",
    ),
    (
        "auto-buyback",
        "Returns itself to your hand after resolving, usually once a condition is met.",
    ),
    (
        "absorb",
        "Prevents or reduces some of the damage that would be dealt to a permanent or player.",
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
    (
        "hate-typal-non-wall",
        "Singles out creatures that aren't Walls, destroying them or dodging their blocks.",
    ),
    (
        "vigilance-counter",
        "Puts a counter on a creature that grants it vigilance.",
    ),
    (
        "ritual-untap",
        "Untaps mana-producing lands or permanents, letting you reuse that mana for more spells.",
    ),
    (
        "zoo",
        "Creates multiple creature tokens of different types.",
    ),
    (
        "doctor-who-episode-saga",
        "A Saga named after and mechanically referencing a Doctor Who episode.",
    ),
    (
        "manaless-land",
        "A land that doesn't tap for mana, instead offering some other effect.",
    ),
    (
        "team-matters",
        "Cares about which team you've joined, changing its effect based on your choice.",
    ),
    (
        "reanimate-planeswalker",
        "Returns a planeswalker card from a graveyard to the battlefield.",
    ),
    (
        "counter-increaser",
        "Adds an extra counter whenever one or more counters would be placed on a permanent you control.",
    ),
    (
        "recursion-from-exile",
        "Returns a card you own from exile to your hand, library, battlefield, or graveyard.",
    ),
    (
        "berserk",
        "Boosts a creature's power for the turn, then sacrifices or destroys it at end of turn.",
    ),
    (
        "pillowfort",
        "Discourages or limits opponents from attacking you.",
    ),
    (
        "multicolor-kicker",
        "Lets you pay one or more extra costs of different colors for a bonus effect when cast.",
    ),
    (
        "typal-outlaw",
        "Cares about or rewards you for controlling Assassins, Mercenaries, Pirates, Rogues, or Warlocks.",
    ),
    (
        "typal-coupling-distinct",
        "Cares about two or more creature types, treating each one differently.",
    ),
    (
        "burn-bright-with-set-mechanic",
        "Gives your team +2/+0 for the turn while also plugging into the set's mechanic.",
    ),
    ("hate-snow", "Punishes or cares about snow permanents."),
    (
        "hate-flash",
        "Stops or taxes opponents from casting spells or acting outside their own turn.",
    ),
    (
        "predefined-token",
        "Creates a specific named token with a fixed, preset set of abilities.",
    ),
    (
        "lifegain-to-damage",
        "Makes opponents lose life whenever you gain life.",
    ),
    (
        "crucible-of-worlds",
        "Lets you play lands straight from your graveyard.",
    ),
    (
        "repeatable-blood",
        "Repeatedly creates Blood tokens, which you can sacrifice to discard and draw a card.",
    ),
    (
        "hate-legendary",
        "Targets or punishes legendary permanents specifically.",
    ),
    (
        "conjure-instant",
        "Conjures an instant card from outside your deck into your hand, library, or exile.",
    ),
    (
        "cost-reducer-equip-ability",
        "Makes equip abilities you activate cost less.",
    ),
    (
        "conjure-random",
        "Conjures a random card from outside your deck into the game.",
    ),
    (
        "cost-reducer-artifact",
        "Makes artifact spells you cast cost less.",
    ),
    (
        "noted-tracked-information",
        "Has you note down a value or choice so a later ability can reference it.",
    ),
    (
        "genesis-effect",
        "Puts permanent cards from near the top of your library onto the battlefield.",
    ),
    (
        "multiple-kicker-costs",
        "Has two or more separate kicker costs, each paid independently for its own bonus effect.",
    ),
    (
        "conjure-to-library",
        "Conjures a brand-new card directly into your library.",
    ),
    (
        "gives-cascade",
        "Grants cascade, letting a spell cast a cheaper card for free from the top of your library.",
    ),
    (
        "gives-intimidate",
        "Grants intimidate, so the creature can only be blocked by artifact creatures or creatures sharing its color.",
    ),
    (
        "armament-ability",
        "Gets stronger based on how many Auras and Equipment are attached to a creature.",
    ),
    (
        "gives-changeling",
        "Grants a creature every creature type at once.",
    ),
    (
        "behold",
        "Gets a bonus if you control or reveal from your hand a card with a specific trait, like a Dragon.",
    ),
    (
        "external-prop",
        "Involves a real-world object or body part outside the game, like a phone, drink, or sticker.",
    ),
    (
        "impulsive-recursion",
        "Exiles a card from your graveyard that you can play for a limited time.",
    ),
    (
        "copy-permanent",
        "Copies a permanent of any card type, usually as a token.",
    ),
    (
        "synergy-hexproof",
        "Rewards you when a creature you control has hexproof, among other keyword abilities.",
    ),
    (
        "detain",
        "Stops a permanent from attacking, blocking, or using activated abilities until your next turn.",
    ),
    (
        "face-commander",
        "A legendary card that headlines a preconstructed Commander deck's box, setting its theme and colors.",
    ),
    (
        "skulk",
        "Skulk: this creature can't be blocked by creatures with greater power.",
    ),
    (
        "hate-storm",
        "Punishes or restricts casting more than one spell each turn.",
    ),
    (
        "permanentfall",
        "Triggers an effect whenever a permanent enters the battlefield under your control.",
    ),
    (
        "typal-eldrazi",
        "Cares about Eldrazi creatures, rewarding you for casting or controlling them.",
    ),
    (
        "synergy-blood",
        "Creates or cares about Blood tokens, which you can sacrifice to loot for a card.",
    ),
    (
        "synergy-adventure",
        "Cares about Adventure cards, rewarding you for casting their spell or creature side.",
    ),
    (
        "typal-ninja",
        "Cares about Ninja creatures, often pairing with ninjutsu to reward unblocked attackers.",
    ),
    (
        "tapper-nonland",
        "Taps one or more target nonland permanents.",
    ),
    (
        "hate-swamp",
        "Punishes opponents for controlling Swamps or black permanents, or rewards you for it.",
    ),
    (
        "noncreature-french-vanilla",
        "A noncreature card whose only abilities are keywords.",
    ),
    (
        "battle-cry",
        "Grants each other attacking creature +1/+0 until end of turn when this creature attacks.",
    ),
    (
        "titan-immortality",
        "Shuffles itself into its owner's library whenever it would go to the graveyard.",
    ),
    (
        "typal-neo-solo-attack",
        "Triggers a bonus effect whenever a Samurai or Warrior you control attacks alone.",
    ),
    (
        "sports-name",
        "A card whose name is a sports term or phrase, with no shared mechanical theme.",
    ),
    (
        "white-elephant",
        "A permanent that can be handed to another player, who then suffers its downside.",
    ),
    (
        "token-doubler",
        "Doubles the number of tokens created under your control.",
    ),
    (
        "animate-vehicle",
        "Turns a Vehicle into an artifact creature without crewing it.",
    ),
    (
        "enchantment-engine",
        "Draws you cards when you cast enchantments or when enchantments enter.",
    ),
    (
        "functional-reminder-counter",
        "Uses a counter to track an ongoing effect that changes what a permanent is or does.",
    ),
    (
        "addendum",
        "An instant that gets a bonus effect if you cast it during your main phase.",
    ),
    (
        "gives-affinity",
        "Grants a spell you cast a cost reduction based on permanents you control.",
    ),
    (
        "hate-tutor",
        "Rewards you or hinders an opponent whenever they search their library.",
    ),
    (
        "afflict",
        "Makes the defending player lose life when this creature becomes blocked.",
    ),
    (
        "rules-text-matters",
        "Cares about the length or content of a card's rules text or a chosen word within it.",
    ),
    (
        "legacy",
        "Uses a mechanic requiring a permanent physical change or choice made to the card itself.",
    ),
    (
        "gives-castable-from-library",
        "Lets you cast a spell straight from your library without paying its mana cost.",
    ),
    (
        "cast-tax",
        "Makes casting spells cost extra or penalizes the caster unless they pay.",
    ),
    (
        "tutor-from-opponent",
        "Searches an opponent's library for a card, usually to their detriment.",
    ),
    (
        "hate-plains",
        "Destroys or punishes Plains, or exploits players who control them.",
    ),
    (
        "dehydration-with-set-mechanic",
        "An aura that taps the enchanted creature and keeps it from untapping.",
    ),
    (
        "conjure-enchantment",
        "Conjures a specific enchantment card from outside the game into play, your hand, or elsewhere.",
    ),
    (
        "tutor-copy",
        "Searches your library for a card with the same name as another creature or permanent.",
    ),
    (
        "oil-counters-matter",
        "Cares about oil counters, growing stronger or triggering effects based on how many are present.",
    ),
    (
        "school-name",
        "A card whose name references things you'd find or do at school.",
    ),
    (
        "torment",
        "Forces an opponent to lose life unless they sacrifice a nonland permanent or discard a card.",
    ),
    (
        "hate-typal-zombie",
        "Punishes or shuts down Zombies, like exiling them or gaining protection from them.",
    ),
    (
        "synergy-saga",
        "Rewards or empowers Saga enchantments, like read ahead, replicate, or chapter-trigger payoffs.",
    ),
    ("sift", "Draws you three cards, then makes you discard one."),
    (
        "storm-like",
        "Copies or scales itself based on spells cast or events this turn, echoing the storm mechanic.",
    ),
    (
        "sneak-from-library",
        "Puts a card straight from your library onto the battlefield without casting it.",
    ),
    (
        "repeatable-powerstones",
        "Creates Powerstone tokens again and again for mana that can only pay for artifacts.",
    ),
    (
        "set-matters",
        "Cares about which real-world Magic set or sets a card was printed in.",
    ),
    (
        "wth-storyline-in-cards",
        "Marks a card tied to the Weatherlight saga's narrative rather than a shared mechanic.",
    ),
    (
        "exile-with-tax",
        "Exiles a permanent, but its owner may play it later for an added cost.",
    ),
    (
        "the-doctor",
        "A creature representing an incarnation of the Doctor from Doctor Who.",
    ),
    (
        "starting-player-matters",
        "Does something different depending on whether you were the starting player.",
    ),
    (
        "fetchland",
        "A land that sacrifices itself to search your library for another land.",
    ),
    (
        "keyword-errata-surveil",
        "Surveils, on a card printed before surveil became a keyword.",
    ),
    (
        "gives-suspend",
        "Puts time counters on a card and grants it suspend if it doesn't already have it.",
    ),
    (
        "conjure-sorcery",
        "Conjures a sorcery card into your hand or exile, creating it from outside the game.",
    ),
    (
        "copy-enchantment",
        "Creates a token that is a copy of an enchantment.",
    ),
    ("copy-land", "Copies a land, often as a token."),
    (
        "tutor-land-basic-mountain",
        "Searches your library for a basic Mountain card.",
    ),
    (
        "hatebird",
        "A flying creature around 3 mana that disrupts opponents like a hatebear.",
    ),
    (
        "marauder",
        "Enters and forces each player to sacrifice a creature.",
    ),
    (
        "land-or-hand",
        "Reveals your top card, putting it onto the battlefield if it is a land or into your hand otherwise.",
    ),
    (
        "formidable",
        "Triggers or unlocks an ability when creatures you control have total power 8 or greater.",
    ),
    (
        "synergy-transform",
        "Rewards you for transforming permanents or controlling transformed permanents.",
    ),
    (
        "flicker-artifact",
        "Exiles an artifact or creature and returns it to the battlefield later.",
    ),
    (
        "conjure-land",
        "Creates a new land card and puts it into your library, hand, or battlefield.",
    ),
    (
        "gives-castable-from-nonhand",
        "Lets you cast a card from a zone other than your hand, like a revealed hand or outside the game.",
    ),
    (
        "trading-post-like",
        "Has multiple activated abilities that feed resources into each other.",
    ),
    (
        "turn-face-up",
        "Turns a face-down creature face up, revealing its true identity.",
    ),
    (
        "tutor-land-basic-swamp",
        "Searches your library for a basic Swamp card and puts it onto the battlefield or into your hand.",
    ),
    (
        "commander-tax-matters",
        "Cares about how much extra mana you've paid in commander tax to recast your commander.",
    ),
    (
        "tutor-land-basic-island",
        "Searches your library for a basic Island card and puts it onto the battlefield or into your hand.",
    ),
    (
        "hate-mountain",
        "Cares about Mountains, punishing them, benefiting from them, or needing one to function.",
    ),
    (
        "gifts-ungiven",
        "Searches your library for several cards and lets an opponent choose which ones you keep.",
    ),
    (
        "catch-22",
        "Punishes each player at their end step unless they meet a condition, like tapping out their lands.",
    ),
    (
        "attacking-opponents-matters",
        "Triggers a bonus whenever a creature attacks one of your opponents rather than you.",
    ),
    (
        "type-errata-dinosaur",
        "A creature retroactively given the Dinosaur type it didn't have when first printed.",
    ),
    (
        "tutor-sorcery",
        "Searches a library for an instant or sorcery card.",
    ),
    (
        "conditional-tapland",
        "A land that enters tapped or untapped depending on whether you meet some condition.",
    ),
    (
        "uril-ability",
        "Boosts a creature's power and toughness for each Aura or Equipment attached to it.",
    ),
    (
        "seek-card",
        "Puts a random qualifying card from your library into your hand without searching.",
    ),
    (
        "old-banish-templating",
        "Exiles a card with one ability, then a linked ability returns it when this leaves play.",
    ),
    (
        "prevent-sacrifice",
        "Keeps permanents from being sacrificed, whether as a cost or forced by an effect.",
    ),
    (
        "first-strike-counter",
        "Puts a first strike counter on a creature, granting it first strike.",
    ),
    (
        "token-version-of-a-card",
        "A token that is a copy of an existing named card, sharing its stats and abilities.",
    ),
    (
        "instant-sorcery-dichotomous",
        "Cares about instants and sorceries as separate categories rather than lumping them together.",
    ),
    (
        "hate-typal-spirit",
        "Targets, counters, or punishes Spirit creatures and spells specifically.",
    ),
    (
        "legendfall",
        "Triggers an effect whenever a legendary permanent enters the battlefield under your control.",
    ),
    (
        "guest-designer",
        "A card created by an outside guest designer rather than Wizards' regular team.",
    ),
    (
        "rack-effect",
        "Deals damage or drains life from players who have few or no cards in hand.",
    ),
    (
        "copy-planeswalker",
        "Creates a token that's a copy of a permanent, which can include a planeswalker.",
    ),
    (
        "opaline-effect",
        "Lets you draw a card when an opponent's spell or ability targets your creature.",
    ),
    (
        "typal-snake",
        "Rewards or boosts you for having Snake creatures.",
    ),
    (
        "menace-counter",
        "Puts a menace counter on a creature, granting menace as long as it's there.",
    ),
    (
        "birthing-pod",
        "Sacrifices a permanent to fetch one with a related mana value onto the battlefield.",
    ),
    (
        "hate-landwalk",
        "Lets creatures with a landwalk ability be blocked as though they didn't have it.",
    ),
    (
        "embalm-token",
        "A token creature made by embalm, a copy of a creature card exiled from your graveyard.",
    ),
    (
        "flicker-permanent",
        "Exiles a permanent of any type, then returns it to the battlefield.",
    ),
    (
        "gives-defender",
        "Grants defender to a creature, so it can't attack.",
    ),
    (
        "synergy-sacrifice-self",
        "Triggers an effect when you sacrifice this permanent itself.",
    ),
    (
        "hand-disruption",
        "Disrupts an opponent's hand, often revealing it so you can take, exile, or use a card.",
    ),
    (
        "creature-type-ship",
        "A creature themed around ships, whether flavored as one or crewing one.",
    ),
    (
        "hate-typal-choose",
        "Names a creature type, then punishes or weakens creatures of that type.",
    ),
    (
        "awaken",
        "Lets you pay more to also turn a land into a creature with +1/+1 counters and haste.",
    ),
    (
        "lockdown-artifact",
        "Keeps a permanent or type of permanent from untapping during its controller's untap step.",
    ),
    (
        "hand-size-decrease",
        "Reduces your or an opponent's maximum hand size.",
    ),
    (
        "wannabe-dark-confidant",
        "Reveals the top card of your library into your hand, costing you life equal to its mana value.",
    ),
    (
        "unstoppable",
        "Lets a creature assign its combat damage as though it weren't blocked.",
    ),
    (
        "hate-typal-goblin",
        "Punishes or preys on Goblin creatures.",
    ),
    (
        "brainstorm",
        "Draws cards, then puts cards from your hand back on top of your library in any order.",
    ),
    (
        "arc-lightning",
        "Deals damage split among one to three targets as you choose.",
    ),
    (
        "tutor-enchantment",
        "Searches your library for an enchantment card and puts it into your hand or on top.",
    ),
    (
        "reveal-hand",
        "Makes one or more players play with their hands revealed.",
    ),
    (
        "token-without-a-card",
        "A token type with no card that actually creates it, likely due to errata.",
    ),
    (
        "impulse-to-top",
        "Lets you look at cards off the top of your library and put one back on top.",
    ),
    (
        "theft-ownership",
        "Lets you draw a card from an opponent's library or take permanent ownership of a card.",
    ),
    (
        "hate-forest",
        "Punishes or exploits Forests, such as destroying them or draining their controllers.",
    ),
    (
        "synergy-spacecraft",
        "Cares about, boosts, or interacts with Spacecraft you control.",
    ),
    ("counterspell-artifact", "Counters an artifact spell."),
    (
        "sneak",
        "Puts a permanent onto the battlefield without casting it.",
    ),
    (
        "sacrifice-outlet-nonland",
        "Lets you sacrifice a nonland permanent you control, usually for a payoff.",
    ),
    (
        "enters-and-leaves-trigger-self",
        "Triggers an effect both when it enters the battlefield and when it leaves.",
    ),
    (
        "frost-armor",
        "Makes opponents pay extra mana to target you or a permanent you control.",
    ),
    (
        "removal-noncreature",
        "Destroys or exiles a noncreature permanent.",
    ),
    (
        "regrowth-planeswalker",
        "Returns a creature or planeswalker card from your graveyard to your hand.",
    ),
    (
        "hate-equipment",
        "Destroys, exiles, or unattaches Equipment from the creatures wearing it.",
    ),
    (
        "counterspell-bounce",
        "Returns a spell on the stack to its owner's hand, effectively countering it.",
    ),
    (
        "play-from-top",
        "Lets you cast the top card of your library, often for free, without exiling it first.",
    ),
    (
        "prevents-win-loss",
        "Stops players from winning or losing the game through normal means.",
    ),
    (
        "phase-manipulation",
        "Adds, skips, or reorders phases of a turn, such as combat or upkeep.",
    ),
    (
        "onomatopoeia",
        "A card whose name is a sound effect word, like a bang or crash.",
    ),
    (
        "synergy-kicker",
        "Rewards you for casting kicked spells, or helps you pay their kicker cost.",
    ),
    (
        "emerge-from-creature",
        "Casts a creature by sacrificing another, lowering the cost by that creature's mana value.",
    ),
    (
        "divvy",
        "Splits cards or permanents into piles, then a player chooses which pile takes effect.",
    ),
    (
        "token-increaser",
        "Adds extra tokens whenever you would create tokens.",
    ),
    (
        "gives-player-protection",
        "Grants you protection so you can't be targeted, dealt damage, or enchanted.",
    ),
    (
        "lich-effect",
        "Lets you survive at 0 or less life, but forces you to sacrifice permanents or discard when you'd lose life instead.",
    ),
    (
        "grim-return",
        "Returns permanents from your graveyard to the battlefield if they died or left play that turn.",
    ),
    (
        "conditional-aura",
        "An aura whose effect on the enchanted permanent changes based on what it's attached to.",
    ),
    (
        "typal-goblin-orc",
        "Rewards or boosts your Goblins and Orcs together as a shared tribe.",
    ),
    (
        "gives-convoke",
        "Grants convoke to spells, letting you tap creatures to help pay for them.",
    ),
    (
        "typal-mercenary",
        "Rewards or tutors for creatures of the Mercenary type.",
    ),
    (
        "command",
        "A modal spell where you choose two of several listed effects to apply.",
    ),
    (
        "typal-minotaur",
        "Rewards or boosts your Minotaurs together as a shared tribe.",
    ),
    (
        "attacking-matters-any",
        "Triggers or reacts whenever any player's creatures attack, not just yours.",
    ),
    (
        "vigor-effect",
        "Converts damage dealt to a creature into +1/+1 counters.",
    ),
    (
        "lightning-bolt-redux",
        "Deals 3 damage for around one red mana, echoing Lightning Bolt.",
    ),
    (
        "mana-gorger",
        "Grows with +1/+1 counters whenever a player casts a spell.",
    ),
    (
        "typal-rebel",
        "A creature that searches your library for a Rebel and puts it onto the battlefield.",
    ),
    (
        "typal-doctor",
        "A card that cares about or supports Doctor creatures you control.",
    ),
    (
        "card-types-matter",
        "Rewards you for having or interacting with cards of many different card types.",
    ),
    (
        "tutor-creature-dragon",
        "Searches your library for a Dragon creature card.",
    ),
    (
        "synergy-flashback",
        "Supports or synergizes with cards that have flashback.",
    ),
    (
        "sneak-self",
        "Puts itself onto the battlefield or into combat outside its normal cost or timing.",
    ),
    (
        "lifegain-increaser",
        "Increases the amount of life you gain whenever you would gain life.",
    ),
    (
        "skip-turn",
        "Causes a player to skip one or more of their turns, often yourself as a cost.",
    ),
    (
        "hate-colorless",
        "Punishes, counters, or destroys colorless spells and permanents.",
    ),
    (
        "roman-numeral",
        "A card whose name includes a roman numeral, like a numbered sequel.",
    ),
    ("burn-self", "Makes a creature deal damage to itself."),
    (
        "impulse-planeswalker",
        "Reveals cards from the top of your library and lets you take a noncreature, nonland card.",
    ),
    (
        "flavor-text-matters",
        "Cares about whether a card has flavor text, often to restrict blocking or fuel an effect.",
    ),
    (
        "explore-like",
        "Lets you look at the top card of a library and keep it or bin it, similar to explore.",
    ),
    (
        "exo-storyline-in-cards",
        "A card tied into a shared in-universe storyline alongside other otherwise unrelated cards.",
    ),
    (
        "ethereal-armor",
        "Grants a bonus that scales with how many enchantments or Auras are in play.",
    ),
    (
        "references-keyword",
        "Names a specific keyword or mechanic in its rules text, for example to grant it or care about it.",
    ),
    (
        "removes-defender",
        "Strips defender from a creature so it can attack.",
    ),
    (
        "reanimate-equipment",
        "Returns an Equipment card from your graveyard to the battlefield.",
    ),
    (
        "booster-tutor",
        "Has you open a booster pack in real life and add a card from it into the game.",
    ),
    (
        "reach-counter",
        "Puts a reach counter on a creature, granting it reach.",
    ),
    (
        "pseudo-shroud",
        "Keeps a permanent or card from being targeted by certain spells or abilities, even yours.",
    ),
    (
        "recasting-commander-matters",
        "Rewards you more each time you've cast your commander from the command zone.",
    ),
    (
        "hate-empty-hand",
        "Rewards you when an opponent has few or no cards in hand.",
    ),
    (
        "phoenix-with-set-s-mechanic",
        "A phoenix creature built around its set's signature mechanic.",
    ),
    (
        "planechase-mechanic",
        "Interacts with Planechase, letting you planeswalk or affect the planar deck and dice.",
    ),
    (
        "gives-banding",
        "Grants a creature banding, letting it attack together with others in a band.",
    ),
    (
        "other-games-matter",
        "Cares about or interacts with a separate game you can see, have played, or will play.",
    ),
    (
        "typal-shaman",
        "A creature or spell that cares about Shamans or works better when you control them.",
    ),
    (
        "conjure-to-graveyard",
        "Conjures cards directly into your graveyard.",
    ),
    (
        "loner",
        "Rewards you for controlling exactly one creature, or wants to be your only creature.",
    ),
    (
        "digital-to-paper",
        "A card first made for a digital-only set that later got a physical paper printing.",
    ),
    (
        "affinity-undergrowth",
        "Costs less to cast for each creature card in your graveyard.",
    ),
    (
        "synergy-ring",
        "Cares about the Ring tempting you and your Ring-bearer.",
    ),
    (
        "hate-typal-human",
        "Punishes or targets Human creatures specifically.",
    ),
    (
        "turns-taken-matter",
        "Cares about how many turns you or the game has taken, not just whose turn it is.",
    ),
    (
        "banish-hand",
        "Exiles one or more cards from a player's hand while this stays on the battlefield.",
    ),
    (
        "mana-abilities-matter",
        "Cares about mana abilities, like tapping a permanent for mana.",
    ),
    ("typal-dwarf", "Rewards and cares about Dwarf creatures."),
    (
        "compulsive-research",
        "Draws cards, then makes you discard two unless you discard a card of a certain type.",
    ),
    ("typal-insect", "Cares about or creates Insect creatures."),
    (
        "watermark-matters",
        "Rewards or interacts with cards that carry a specific watermark.",
    ),
    (
        "gives-ingest",
        "Grants an ability that exiles a player's top card on combat damage, letting you play it.",
    ),
    (
        "typal-wall",
        "Cares about or synergizes with Wall creatures.",
    ),
    (
        "gives-shadow",
        "Grants shadow, letting a creature block and be blocked only by other shadow creatures.",
    ),
    (
        "gives-stalking",
        "Makes a creature unable to be blocked by more than one creature.",
    ),
    ("typal-mouse", "Cares about or creates Mouse creatures."),
    (
        "typal-mutant",
        "Cares about or supports creatures with the Mutant creature type.",
    ),
    (
        "gains-fear",
        "Gives itself fear, so it can only be blocked by artifact and/or black creatures.",
    ),
    (
        "typal-frog",
        "Cares about or supports creatures with the Frog creature type.",
    ),
    (
        "tutor-enchantment-aura",
        "Searches your library for an Aura card and puts it into your hand or onto the battlefield.",
    ),
    (
        "hate-monocolor",
        "Punishes, resists, or destroys monocolored creatures and spells.",
    ),
    (
        "theft-spell",
        "Takes control of a spell on the stack, letting you redirect or resolve it yourself.",
    ),
    (
        "synergy-surveil",
        "Rewards you for surveilling with bonuses like counters, card draw, or extra damage.",
    ),
    (
        "counterspell-free",
        "A counterspell you can cast for no mana, usually by paying an alternate cost instead.",
    ),
    (
        "specific-power-matters",
        "Cares about creatures or spells with a particular power, toughness, or loyalty value.",
    ),
    (
        "super-menace",
        "Can't be blocked except by three or more creatures.",
    ),
    (
        "burn-battle",
        "Deals damage to a battle, alongside or instead of creatures and planeswalkers.",
    ),
    (
        "rule-of-law",
        "Limits how many spells a player can cast each turn.",
    ),
    (
        "hunger",
        "Grows with a +1/+1 counter whenever a creature it damaged this turn dies.",
    ),
    (
        "recursion-instant",
        "Lets you cast an instant card back from a graveyard.",
    ),
    (
        "relentless",
        "Lets you run more than the normal four copies of this card in your deck.",
    ),
    (
        "pseudo-equipment",
        "A creature or artifact that stays tapped to give another permanent a lasting bonus.",
    ),
    (
        "nonfunctional-reminder-counter",
        "Uses a counter only to mark a lasting effect that stays even if the counter is removed.",
    ),
    (
        "mechanical-foreshadow",
        "References a card, card type, or mechanic that didn't exist in the game yet when it was printed.",
    ),
    (
        "eternalize-token",
        "A token made by eternalize: a 4/4 black Zombie copy of the card that keeps its abilities.",
    ),
    (
        "removal-vehicle",
        "Removal that can destroy or exile a Vehicle, not just creatures.",
    ),
    (
        "discard-matters",
        "Triggers an effect or gets stronger when you discard a card.",
    ),
    (
        "cycle-zodiac-creature",
        "One of a cycle of animal creatures, each with landwalk tied to its type.",
    ),
    (
        "greed-ability",
        "Lets you pay life and mana to draw a card.",
    ),
    (
        "hate-theft",
        "Punishes or prevents theft, letting you destroy, exile, or protect a permanent so no one else profits from controlling it.",
    ),
    (
        "gives-charge-counter",
        "Puts a charge counter on a target artifact.",
    ),
    (
        "coin-flips-matter",
        "Cares about coin flips and rewards you for winning them.",
    ),
    (
        "x-doesn-t-matter",
        "Has an {X} in its cost that the card's own text never directly references.",
    ),
    (
        "change-name",
        "Changes the name of a permanent, either its own or another.",
    ),
    (
        "un-color",
        "A joke card that uses fake colors like pink, gold, or orange that don't exist in normal Magic.",
    ),
    (
        "untapper-nonland",
        "Untaps a nonland permanent, letting you use it again this turn.",
    ),
    (
        "creature-type-townsfolk",
        "A creature that is or once was of the retired Townsfolk creature type.",
    ),
    (
        "typal-lizard",
        "Rewards you for controlling or casting Lizard creatures.",
    ),
    (
        "library-size-matters",
        "Cares about how many cards are in a library, yours or another player's.",
    ),
    (
        "typal-bat",
        "Rewards you for controlling or casting Bat creatures.",
    ),
    (
        "typal-dog",
        "Rewards you for controlling or casting Dog creatures.",
    ),
    (
        "turns-off-defender",
        "Lets creatures with defender attack as though they didn't have it.",
    ),
    (
        "typal-samurai",
        "Cares about Samurai creatures, boosting or empowering the Samurai you control.",
    ),
    (
        "counterspell-tuck",
        "Counters a spell by putting it on top or bottom of its owner's library instead of the graveyard.",
    ),
    (
        "hate-free-spell",
        "Punishes spells cast without spending any mana, usually by countering or damaging their caster.",
    ),
    (
        "synergy-land-graveyard",
        "Triggers a benefit whenever a land card is put into your graveyard from anywhere.",
    ),
    (
        "synergy-bounce",
        "Triggers a benefit whenever a permanent is returned to a player's hand.",
    ),
    (
        "synergy-attraction",
        "Cares about Attractions you've opened or visited, rewarding you for having or visiting them.",
    ),
    (
        "synergy-cave",
        "Gets cheaper, stronger, or searches your deck based on the Caves you control or have in your graveyard.",
    ),
    (
        "substance",
        "An obsolete ability from old Mirage-era cards that does nothing at all.",
    ),
    (
        "counter-preservation",
        "Moves a creature's counters onto another permanent when it dies or leaves the battlefield.",
    ),
    (
        "hate-shadow",
        "Interacts with, blocks, or destroys creatures that have shadow.",
    ),
    (
        "sol-land",
        "A land that taps for two colorless mana at once.",
    ),
    (
        "landslow",
        "Stops players from playing lands or limits how many they can play.",
    ),
    (
        "fulfilled-futureshift",
        "A card first previewed as futureshifted that later got a proper printing in its own set.",
    ),
    (
        "sacrifice-outlet-planeswalker",
        "Lets you sacrifice a creature or planeswalker, usually as a cost for another effect.",
    ),
    (
        "bounty",
        "A double-faced card, a bounty on its front and a Wanted! reward on its back.",
    ),
    (
        "rescue-artifact",
        "Returns an artifact you control to its owner's hand, often as a cost or trigger.",
    ),
    (
        "removes-indestructible",
        "Strips indestructible from a permanent, usually so damage can finish it off.",
    ),
    (
        "seek-land-any",
        "Randomly seeks a land card from your library, putting it into your hand or onto the battlefield.",
    ),
    (
        "animate-enchantment",
        "Turns noncreature enchantments into creatures.",
    ),
    (
        "flicker-nonland",
        "Exiles a nonland permanent then returns it to the battlefield, retriggering its enter effects.",
    ),
    (
        "face-up-face-down-effects",
        "Triggers or matters when a permanent is turned face down or face up.",
    ),
    (
        "bottom-of-library-matters",
        "Cares about the bottom of a library, such as drawing from it or exiling cards there.",
    ),
    (
        "removal-battle",
        "Removal that can destroy, exile, or otherwise get rid of a battle.",
    ),
    (
        "fling-self",
        "Sacrifices this creature to deal damage equal to its power.",
    ),
    (
        "fractional-power-toughness",
        "Has a power or toughness of one half, or grants it.",
    ),
    (
        "hate-colored",
        "Destroys, exiles, or otherwise punishes permanents that are one or more colors.",
    ),
    (
        "pseudo-dethrone",
        "Grants a bonus when attacking the player with the most life, or tied for it.",
    ),
    (
        "block-unlimited",
        "Lets a creature block any number of attacking creatures at once.",
    ),
    (
        "conjure-card",
        "Conjures a duplicate of a card into your hand or onto the battlefield.",
    ),
    (
        "distinct-echo-cost",
        "Has an echo cost you pay to keep it, differing from its mana cost.",
    ),
    (
        "drawlink",
        "Draws you cards equal to the combat damage a creature deals to a player.",
    ),
    (
        "hate-typal-vampire",
        "Punishes, destroys, or exiles Vampire creatures.",
    ),
    (
        "howlgeist-ability",
        "Stops creatures with lower power from blocking this creature.",
    ),
    (
        "lose-trigger",
        "Triggers an effect whenever a player loses the game.",
    ),
    (
        "commander-tax-evasion",
        "Reduces, eliminates, or replaces the extra tax you pay to recast your commander.",
    ),
    (
        "hate-typal-elf",
        "A creature or effect that scales with the number of Elves on the battlefield.",
    ),
    (
        "worship",
        "Sets a life floor: while active, damage can't reduce your life total below a set number.",
    ),
    (
        "unexile",
        "Returns a card from exile to another zone, or sends it elsewhere instead of being exiled.",
    ),
    (
        "cycle-rav-mmn",
        "One of a matched cycle of cards from Ravnica block reprinted in Modern Masters.",
    ),
    (
        "typal-otter",
        "A creature or spell that cares about Otters, often alongside Birds, Frogs, and Rats.",
    ),
    (
        "hate-sacrifice",
        "Stops players from sacrificing permanents, or punishes them for doing so.",
    ),
    (
        "typal-golem",
        "Creates Golem tokens or boosts the Golems you control.",
    ),
    (
        "tutor-creature-mercenary",
        "Searches your library for a Mercenary card and puts it into your hand or onto the battlefield.",
    ),
    (
        "tuck",
        "Puts a spell or permanent into its owner's library, often on top or bottom.",
    ),
    (
        "tribute",
        "As it enters, an opponent adds +1/+1 counters to it, or you get a bonus effect instead.",
    ),
    (
        "turn-control",
        "Lets you control an opponent's turn, seeing their cards and making their decisions for them.",
    ),
    (
        "top-deck-manipulation",
        "Lets you look at, rearrange, or otherwise change the top cards of a library.",
    ),
    (
        "theft-planeswalker",
        "Takes control of an opponent's planeswalker.",
    ),
    (
        "synergy-target",
        "Rewards you for casting spells and abilities that target, often targeting this card.",
    ),
    (
        "sunder",
        "Destroys all Auras or Equipment attached to a permanent, leaving the permanent itself behind.",
    ),
    (
        "speed-matters",
        "Cares about your current speed value, not just about hitting max speed.",
    ),
    (
        "skip-untap-step",
        "Makes a player skip their next untap step, so their permanents stay tapped.",
    ),
    (
        "four-loyalty-abilities",
        "A planeswalker with four loyalty abilities instead of the usual three.",
    ),
    (
        "counter-fuel-mm",
        "A creature that gets a benefit, like mana or removal, from removing -1/-1 counters off itself.",
    ),
    (
        "invitational-card",
        "A card originally designed by the winning player of a Magic Invitational tournament.",
    ),
    (
        "restock-sorcery",
        "Puts a sorcery card from your graveyard back on top of your library.",
    ),
    (
        "removes-hexproof",
        "Strips hexproof from opposing creatures or players so you can target them.",
    ),
    (
        "gives-islandwalk",
        "Grants islandwalk to a creature, so it can't be blocked if the defender controls an Island.",
    ),
    (
        "restock-instant",
        "Puts an instant card from your graveyard back on top of your library.",
    ),
    (
        "recursion-sorcery",
        "Lets you cast an instant or sorcery card from your graveyard again.",
    ),
    (
        "reanimate",
        "Returns a permanent card from a graveyard straight onto the battlefield.",
    ),
    (
        "quietus-effect",
        "Makes a player lose half their life when they take damage from the source.",
    ),
    (
        "gives-forestwalk",
        "Grants a creature forestwalk, so it can't be blocked while the defender controls a Forest.",
    ),
    (
        "poison-mechanics",
        "Interacts with poison counters directly, adding, removing, or changing how many cost you the game.",
    ),
    (
        "platinum-angel-effect",
        "Keeps you from losing the game and stops your opponents from winning.",
    ),
    (
        "phyrexian-token",
        "A creature token that also has a Phyrexian, oil-slick art variant.",
    ),
    (
        "heckbent",
        "Gets stronger or gains abilities as long as you have one or fewer cards in hand.",
    ),
    (
        "recurring-suspend",
        "Has suspend and, when it resolves, exiles itself again with time counters to suspend once more.",
    ),
    (
        "nevermore",
        "Names a card, then keeps spells with that name from being cast.",
    ),
    (
        "gives-exalted",
        "Grants exalted, giving a bonus to a creature that attacks alone.",
    ),
    (
        "control-blocker",
        "Lets you choose or force how creatures block during combat.",
    ),
    (
        "mob-name",
        "A card named after mafia or mob-movie slang, part of a themed naming cycle.",
    ),
    (
        "cycle-cmm-r-two-color-legend",
        "One of a two-color legendary creature cycle from Commander Masters.",
    ),
    (
        "lockdown-land",
        "Stops lands from untapping during their controller's untap step.",
    ),
    (
        "cycle-cmm-draft-signpost",
        "A legendary creature from Commander Masters built to signal a draft archetype.",
    ),
    (
        "cycle-cmm-m-mono-legend",
        "One of a mono-colored legendary creature cycle from Commander Masters.",
    ),
    (
        "eminence",
        "A commander ability that works both on the battlefield and while sitting in the command zone.",
    ),
    (
        "grind",
        "Mills a player until a set number of land cards hit the graveyard.",
    ),
    (
        "cycle-clb-draft-signpost",
        "One of Baldur's Gate's two-color legendary creatures built to support its guild's draft archetype.",
    ),
    (
        "cycle-znr-two-color-legend",
        "One of Zendikar Rising's ten two-color legendary creatures, one per color pair.",
    ),
    (
        "cycle-woe-u-2c-adventurer",
        "One of Wilds of Eldraine's uncommon two-color Adventure creatures, each pairing a spell with its own creature.",
    ),
    (
        "cycle-woe-r-2c-adventurer",
        "One of Wilds of Eldraine's rare two-color Adventure creatures, each pairing a spell with its own creature.",
    ),
    (
        "cycle-war-u-two-color-spell",
        "One of War of the Spark's uncommon two-color instants or sorceries, one per color pair.",
    ),
    (
        "cone-spell",
        "Does something three times, using the values 1, 2, and 3.",
    ),
    (
        "cycle-akh-draft-signpost",
        "An uncommon multicolor creature built to showcase a two-color archetype in this set's draft format.",
    ),
    (
        "cycle-vow-draft-signpost",
        "An uncommon multicolor creature built to showcase a two-color archetype in this set's draft format.",
    ),
    (
        "cycle-vow-r-two-color-legend",
        "A rare legendary two-color creature that anchors and rewards its color pair's draft archetype.",
    ),
    (
        "cycle-war-draft-signpost",
        "An uncommon multicolor card built to showcase a two-color archetype in this set's draft format.",
    ),
    (
        "cycle-woe-draft-signpost",
        "An uncommon multicolor creature built to showcase a two-color archetype in this set's draft format.",
    ),
    (
        "cycle-znr-draft-signpost",
        "An uncommon card built to showcase a two-color archetype in this set's draft format.",
    ),
    (
        "cycle-war-hybrid-planeswalker",
        "One of War of the Spark's hybrid-mana planeswalkers, castable with either of two colors.",
    ),
    (
        "cycle-afr-u-legend",
        "One of Adventures in the Forgotten Realms' uncommon legendary creatures.",
    ),
    (
        "cycle-vma-r-two-color",
        "One of Vintage Masters' rare two-color gold cards.",
    ),
    (
        "cycle-vma-draft-signpost",
        "A Vintage Masters card built to signal and reward a specific two-color draft archetype.",
    ),
    (
        "cycle-war-u-planeswalker",
        "One of War of the Spark's uncommon single-color planeswalkers.",
    ),
    (
        "great-designer-search-3",
        "A card from or inspired by the Great Designer Search 3 design competition.",
    ),
    (
        "cycle-tsp-c-sliver",
        "A common Sliver that gives every Sliver in play a shared ability or stat boost.",
    ),
    (
        "cycle-tmt-team-up",
        "A legendary creature representing two characters teamed up on a single card.",
    ),
    (
        "cycle-tmt-c-hybrid",
        "A common creature with a hybrid mana cost, payable with either of its two colors.",
    ),
    (
        "cycle-tmt-r-hybrid",
        "A rare legendary creature with a hybrid mana cost, payable with either of its two colors.",
    ),
    (
        "cycle-unf-draft-signpost",
        "A legendary creature that points you toward a specific draft archetype for the set.",
    ),
    (
        "commander-identity-matters",
        "Cares about your commander's color identity, often tapping for mana in those colors.",
    ),
    (
        "cycle-tla-c-tapland",
        "A common dual land that enters tapped and can later be sacrificed to draw a card.",
    ),
    (
        "cycle-tla-r-two-color",
        "A rare two-color creature whose ability reflects its character.",
    ),
    (
        "combat-arbiter",
        "Restricts how many creatures can attack or block each combat.",
    ),
    (
        "cycle-tla-c-hybrid",
        "One of a common cycle in the Avatar: The Last Airbender set, each with a hybrid mana cost.",
    ),
    (
        "cycle-tla-u-hybrid",
        "One of an uncommon cycle of legendary creatures in Avatar: The Last Airbender, each with a hybrid mana cost.",
    ),
    (
        "cycle-aer-draft-signpost",
        "A signpost uncommon that points you toward one of Aether Revolt's two-color draft archetypes.",
    ),
    (
        "cycle-ths-r-two-color",
        "One of a rare cycle in Theros, each card built around a specific two-color pair.",
    ),
    (
        "cycle-tla-u-two-color",
        "One of an uncommon cycle in Avatar: The Last Airbender, each card built around a specific two-color pair.",
    ),
    (
        "cycle-ths-draft-signpost",
        "A signpost uncommon that points you toward one of Theros's two-color draft archetypes.",
    ),
    (
        "cycle-abu-dual-land",
        "An original dual land that taps for either of two colors of mana.",
    ),
    (
        "cycle-spm-draft-signpost",
        "A two-color draft signpost that points toward its two-color archetype.",
    ),
    (
        "cycle-spm-r-two-color",
        "A rare two-color legendary creature built around its set's draft archetypes.",
    ),
    (
        "cycle-tdm-draft-signpost",
        "A two-color draft signpost that points toward its archetype's strategy.",
    ),
    (
        "cycle-tdm-r-tricolor",
        "A rare three-color card built around its set's wedge archetypes.",
    ),
    (
        "cycle-tkhm-realm-token",
        "One of a cycle of vanilla or near-vanilla creature tokens made by cards in this set.",
    ),
    (
        "gives-wither",
        "Grants wither to a creature, so its combat damage lands as -1/-1 counters instead.",
    ),
    (
        "cycle-a25-r-two-color",
        "A rare two-color gold card from a reprint cycle of classic multicolor cards.",
    ),
    (
        "cycle-a25-draft-signpost",
        "An uncommon gold card meant to point drafters toward a two-color archetype.",
    ),
    (
        "clothing-matters",
        "A joke card whose effect depends on clothing or accessories you're actually wearing.",
    ),
    (
        "cycle-soi-draft-signpost",
        "An uncommon from Shadows over Innistrad that signals a two-color draft archetype.",
    ),
    (
        "cycle-shm-liege",
        "A creature that gives +1/+1 to your other creatures of two colors.",
    ),
    (
        "cycle-spm-c-legend",
        "A common legendary creature from the Spider-Man set's character cycle.",
    ),
    (
        "cycle-thb-draft-signpost",
        "An uncommon gold card from Theros Beyond Death that signals a two-color draft archetype.",
    ),
    (
        "cycle-rvr-c-two-color",
        "One of a cycle of common two-color cards, one for each Ravnica guild.",
    ),
    (
        "cycle-c16-m-partner",
        "One of a cycle of legendary creatures with partner, letting you pair two of them as commanders.",
    ),
    (
        "cycle-rvr-r-guild-spell",
        "One of a cycle of rare cards, each a signature card for one of Ravnica's ten guilds.",
    ),
    (
        "cycle-rtr-m-two-color",
        "One of a cycle of mythic two-color cards, one for each of the set's guilds.",
    ),
    (
        "cycle-rvr-c-hybrid",
        "One of a cycle of common creatures with hybrid mana costs, playable with either of two colors.",
    ),
    (
        "cycle-rvr-r-two-color-legend",
        "One of a cycle of rare two-color legendary creatures, each leading one of Ravnica's guilds.",
    ),
    (
        "cycle-rix-c-typal-boost",
        "One of a cycle of common cards that reward or boost creatures of a specific tribal type.",
    ),
    (
        "cycle-rix-draft-signpost",
        "One of a cycle of two-color uncommons that signals a draft archetype in its set.",
    ),
    (
        "counterspell-enchantment",
        "Counter magic that specifically stops enchantment spells from resolving.",
    ),
    (
        "affinity-for-party",
        "Costs less to cast for each creature in your party of Cleric, Rogue, Warrior, and Wizard.",
    ),
    (
        "creature-type-guardian",
        "A creature that protects itself or your other permanents with defensive abilities.",
    ),
    (
        "morphling",
        "A shapeshifter with mana abilities that let it change its stats or gain keywords at instant speed.",
    ),
    (
        "cycle-cmr-r-two-color",
        "A two-color legendary creature from a rare cycle.",
    ),
    (
        "cycle-c21-mono-legend",
        "A mono-color legendary creature from a Commander cycle.",
    ),
    (
        "cycle-blb-c-typal-boost",
        "Gets a bonus effect or costs less if you control a creature of a specific type.",
    ),
    (
        "cycle-dft-nonvehicle-signpost",
        "A two-color archetype signpost card from Aetherdrift that isn't a Vehicle.",
    ),
    (
        "cycle-bbd-legendary-partner",
        "A legendary creature with partner with that fetches its named partner to a player's hand when it enters.",
    ),
    (
        "cycle-dual-surveil-land",
        "A tapped dual land that lets you surveil 1 when it enters.",
    ),
    (
        "synergy-incubator",
        "A card that creates, transforms, or otherwise builds around Incubator tokens.",
    ),
    (
        "tutor-creature-rebel",
        "A creature that searches your library for a Rebel permanent card and puts it onto the battlefield.",
    ),
    (
        "hate-graveyard-cast",
        "Stops players from casting spells out of graveyards.",
    ),
    (
        "cycle-block-rtr-m-multicolor",
        "A multicolor mythic card from the Return to Ravnica block's guild cycle.",
    ),
    (
        "cycle-ktk-draft-signpost",
        "A card that signals which two-color archetype to draft in this set.",
    ),
    (
        "cycle-blb-draft-signpost",
        "A gold creature that signals a color pair's draft archetype in this set.",
    ),
    (
        "cycle-blb-u-typal",
        "A creature that rewards controlling others of its own creature type.",
    ),
    (
        "cycle-mom-u-mono-invasion",
        "A single-color battle that flips into its back-face permanent once its defense is broken.",
    ),
    (
        "cycle-mm3-r-two-color",
        "A rare two-color gold card reprinted as part of this set's two-color rare cycle.",
    ),
    (
        "cycle-neo-draft-signpost",
        "A card that signals which two-color archetype to draft in this set.",
    ),
    (
        "cycle-dgm-m-two-color",
        "A mythic two-color gold card representing one of the set's ten guilds.",
    ),
    (
        "cycle-cn2-draft-signpost",
        "A two-color creature that signals its color-pair draft archetype in Conspiracy.",
    ),
    (
        "just-shuffle",
        "Shuffles a library as its own effect, not as the cleanup after a search.",
    ),
    (
        "cycle-cmr-tricolor-legend",
        "A three-color legendary creature built as a Commander Legends commander.",
    ),
    (
        "cycle-rav-backward-ability",
        "A Ravnica permanent with a single simple activated ability.",
    ),
    (
        "cycle-dft-team-vehicle",
        "A Vehicle from Aetherdrift's racing teams that you crew with creatures to attack.",
    ),
    (
        "cycle-cmr-forward-partner",
        "A Partner legend, the forward half of a color-pair signpost pair for draft.",
    ),
    (
        "cycle-block-rtr-guildmaster",
        "A two-color legendary creature representing one Ravnica guild.",
    ),
    (
        "cycle-thb-r-m-two-color",
        "A rare or mythic two-color card from Theros Beyond Death's cycle of gold cards.",
    ),
    (
        "cycle-rvr-u-two-color",
        "An uncommon two-color card from the Ravnica Remastered cycle of gold cards.",
    ),
    (
        "cycle-m21-draft-signpost",
        "An uncommon that points drafters toward one of Core Set 2021's two-color archetypes.",
    ),
    (
        "turn-face-down",
        "Turns a creature face down, usually making it a 2/2 with no text until it's turned face up.",
    ),
    (
        "static-effect-in-graveyard",
        "A card whose ability keeps working even while it sits in your graveyard.",
    ),
    (
        "cycle-blb-duo",
        "A Bloomburrow Duo creature with two creature types, one of a cycle of ten.",
    ),
    (
        "pile",
        "Sorts cards or permanents into separate piles for selection or grouping.",
    ),
    (
        "cycle-clu-guild-rare",
        "A rare that showcases a two-color guild's strategy.",
    ),
    (
        "cycle-mom-draft-signpost",
        "A two-color card built to point drafters toward that color pair's archetype.",
    ),
    (
        "cycle-ktk-gainland",
        "A dual land that enters tapped and gains you 1 life.",
    ),
    (
        "cycle-iko-companion",
        "A companion: if your whole deck meets its deckbuilding restriction, you can fetch it from outside the game for {3}.",
    ),
    (
        "cycle-bfz-draft-signpost",
        "A creature built to point drafters toward one of Battle for Zendikar's color-pair archetypes.",
    ),
    (
        "old-fight",
        "Deals damage equal to its power to a target creature, which deals its power back: fight before it was keyworded.",
    ),
    (
        "cycle-bbd-u-partner",
        "Has partner with a named card: on entry, a player may tutor that specific partner into their hand.",
    ),
    (
        "cycle-mh3-c-draft-signpost",
        "One of a cycle of common creatures that signal a two-color draft archetype.",
    ),
    (
        "moxen",
        "A zero-cost artifact named Mox that taps for mana.",
    ),
    (
        "cycle-dsk-r-two-color",
        "One of a cycle of two-color rares, each anchoring a different draft archetype.",
    ),
    (
        "cycle-eoe-draft-signpost",
        "One of a cycle of creatures that signal a two-color draft archetype.",
    ),
    (
        "cycle-dmr-draft-signpost",
        "One of a cycle of multicolor cards that signal a two-color draft archetype.",
    ),
    (
        "cycle-dgm-u-fuse",
        "An uncommon split card with fuse, letting you cast both halves together for their combined cost.",
    ),
    (
        "cycle-otj-draft-signpost",
        "One of a cycle of cards that signal a two-color draft archetype.",
    ),
    (
        "impulse-artifact-equipment",
        "Digs through the top cards of your library for an Equipment or Vehicle card and puts it into your hand.",
    ),
    (
        "cycle-block-rav-r-hybrid",
        "One of a cycle of rare hybrid-mana cards from the Ravnica block, playable with either color of its guild.",
    ),
    (
        "cycle-dgm-c-guild-ability",
        "One of a cycle of Dragon's Maze commons, each showcasing a different guild's signature mechanic.",
    ),
    (
        "cycle-block-rav-c-hybrid",
        "One of a cycle of common hybrid-mana cards from the Ravnica block, playable with either color of its guild.",
    ),
    (
        "cycle-block-rav-mnn",
        "One of a cycle of two-color guild creatures from the Ravnica block, one for each guild.",
    ),
    (
        "cycle-dgm-cluestones",
        "A Cluestone: an artifact that taps for either of its guild's colors, or sacrifice it paying both to draw a card.",
    ),
    (
        "cycle-rav-guildhall",
        "A Ravnica guildhall land that taps for colorless mana or pays its guild's colors for a themed ability.",
    ),
    (
        "hate-first-strike",
        "Strips first strike from a creature or stops it from having or gaining first strike.",
    ),
    (
        "cycle-block-rtr-guild-charm",
        "A two-color instant that lets you choose one of three guild-flavored effects.",
    ),
    (
        "cycle-rav-guildmaster",
        "A two-color legendary creature that leads one of the Ravnica guilds.",
    ),
    (
        "cycle-apc-c-two-color",
        "A common two-color card from a matched cycle, each with its own small effect.",
    ),
    (
        "cycle-dual-investigate-tapland",
        "A tapped dual land that can pay mana and tap to investigate for a Clue.",
    ),
    (
        "cycle-mkm-m-two-color",
        "A mythic two-color card from a matched cycle, each with its own effect.",
    ),
    (
        "cycle-clb-tricolor-legend",
        "A three-color legendary creature with a build-around ability.",
    ),
    (
        "cycle-mid-u-flashback",
        "Can be cast again from your graveyard for its flashback cost, then exiled.",
    ),
    (
        "cycle-fin-dual-town",
        "A two-color land that enters tapped and taps for either of its two colors.",
    ),
    (
        "cycle-guildgate",
        "A dual land that enters tapped and taps for either of a guild's two colors.",
    ),
    (
        "cycle-mh3-mdfc-mono-land",
        "A modal double-faced card you can instead play as a land that taps for one color.",
    ),
    (
        "cycle-cns-r-two-color",
        "A rare creature costed in two colors.",
    ),
    (
        "cycle-block-rtr-r-hybrid",
        "A rare creature or spell costed in hybrid mana, payable with either of two colors.",
    ),
    (
        "cycle-blb-hybrid",
        "A two-color hybrid creature you can cast with either of its two colors.",
    ),
    (
        "cycle-kld-draft-signpost",
        "An uncommon two-color creature or spell built to point drafters toward a specific archetype.",
    ),
    (
        "cycle-m20-draft-signpost",
        "An uncommon two-color card that signals and rewards drafting a specific archetype.",
    ),
    (
        "cycle-dft-team-captain",
        "A legendary creature that captains a two-color draft archetype.",
    ),
    (
        "cycle-clu-r-hybrid",
        "A rare hybrid-cost creature that revives a classic guild mechanic.",
    ),
    (
        "cycle-mid-r-flashback",
        "A rare sorcery you can cast once from your graveyard for its flashback cost, then exile.",
    ),
    (
        "cycle-ktk-enemy-ability",
        "A common creature with an activated ability that costs mana of an enemy color.",
    ),
    (
        "cycle-bro-draft-signpost",
        "An uncommon two-color creature built to point drafters toward a specific archetype.",
    ),
    (
        "cycle-m19-draft-signpost",
        "One of Core Set 2019's two-color uncommons built to point drafters toward that color pair's archetype.",
    ),
    (
        "cycle-khm-r-saga",
        "One of Kaldheim's rare Saga enchantments, unfolding a different effect over three chapters.",
    ),
    (
        "cycle-mom-two-color-team-up",
        "One of March of the Machine's legendary team-up creatures, pairing two characters in one two-color card.",
    ),
    (
        "cycle-khm-realm",
        "One of Kaldheim's tapped realm lands that taps for one color and can be sacrificed for a bigger two-color effect.",
    ),
    (
        "cycle-block-rtr-u-guild-kw",
        "One of Return to Ravnica block's uncommons showcasing a guild's signature keyword mechanic.",
    ),
    (
        "cycle-khm-legendary-signpost",
        "One of Kaldheim's legendary signpost creatures anchoring a two-color archetype.",
    ),
    (
        "cycle-ima-r-two-color",
        "One of Iconic Masters' two-color rares and mythics reprinted for the set.",
    ),
    (
        "cycle-rav-signet",
        "An artifact that pays one mana and taps to add two mana of its guild's colors.",
    ),
    (
        "cycle-inr-draft-signpost",
        "A two-color draft signpost card that showcases an archetype's mechanics.",
    ),
    (
        "cycle-mh2-r-two-color-new",
        "A new two-color rare card from Modern Horizons 2.",
    ),
    (
        "cycle-keyrune",
        "An artifact that taps for two guild colors and can become a creature of those colors.",
    ),
    (
        "cycle-2xm-draft-signpost",
        "A two-color draft signpost card that showcases an archetype's mechanics.",
    ),
    (
        "cycle-bbd-draft-signpost",
        "A two-color draft signpost card that showcases an archetype's mechanics.",
    ),
    (
        "cycle-block-ths-scry-land",
        "A dual land that enters tapped, scries 1, and taps for two colors of mana.",
    ),
    (
        "freeze-artifact",
        "Taps down an artifact or creature so it stays tapped through its controller's next untap step.",
    ),
    (
        "cycle-msh-draft-signpost",
        "One of a cycle of legendary uncommon creatures that signals a two-color archetype for drafting.",
    ),
    (
        "cycle-dsk-thirteenland",
        "A dual land that enters tapped unless a player has 13 or less life.",
    ),
    (
        "cycle-dmu-r-two-color-legend",
        "One of a cycle of rare two-color legendary creatures, each with its own unique ability.",
    ),
    (
        "cycle-block-rtr-c-hybrid",
        "One of a cycle of common instants and creatures with hybrid mana costs from the Return to Ravnica block.",
    ),
    (
        "cycle-block-rtr-off-color",
        "One of a cycle of Ravnica cards with a mana ability in a color outside their guild's pair.",
    ),
    (
        "cycle-mkm-draft-signpost",
        "One of a cycle of uncommon creatures that signals a two-color archetype for drafting.",
    ),
    (
        "cycle-rav-forward-ability",
        "A permanent with a small activated ability keyed to one of its colors.",
    ),
    (
        "cycle-ori-draft-signpost",
        "A two-color uncommon built to point drafters toward its color pair's archetype.",
    ),
    (
        "cycle-block-rtr-guildmage",
        "A guild creature with two activated abilities, each using both of its colors.",
    ),
    (
        "cycle-rav-forward-boost",
        "Gains a bonus effect if a specific color of mana was spent to cast it.",
    ),
    (
        "cycle-pls-r-two-color",
        "A two-color gold rare from Planeshift.",
    ),
    (
        "cycle-ogw-draft-signpost",
        "A two-color uncommon built to point drafters toward its color pair's archetype.",
    ),
    (
        "cycle-block-rav-guildmage",
        "A guild creature with two activated abilities, one for each of its colors.",
    ),
    (
        "cycle-eoe-u-spacecraft",
        "An uncommon Spacecraft artifact with an enters effect that becomes a creature once stationed.",
    ),
    (
        "cycle-block-rav-guild-champion",
        "A two-color legendary creature that leads one of Ravnica's ten guilds.",
    ),
    (
        "cycle-block-rav-colors-matter",
        "A legendary creature with a separate ability tied to each of its two colors.",
    ),
    (
        "cycle-otj-pingland",
        "A dual land that enters tapped and deals 1 damage to an opponent when it enters.",
    ),
    (
        "cycle-otj-u-legend",
        "An uncommon two-color legendary creature from Outlaws of Thunder Junction.",
    ),
    (
        "cycle-ema-r-two-color",
        "A two-color gold rare from Eternal Masters.",
    ),
    (
        "cycle-blb-r-two-color-legend",
        "A rare legendary creature pairing two colors and rewarding you for building around its creature type.",
    ),
    (
        "cycle-apc-u-two-color",
        "A two-color uncommon from Apocalypse pairing enemy colors.",
    ),
    (
        "cycle-dom-draft-signpost",
        "A legendary creature built to point drafters toward its two-color archetype.",
    ),
    (
        "cycle-msh-gainland",
        "A tapped dual land that gains you 1 life when it enters.",
    ),
    (
        "cycle-apc-r-two-color",
        "A two-color rare from Apocalypse pairing enemy colors.",
    ),
    (
        "cycle-eld-u-two-color",
        "One of a cycle of two-color uncommon cards, one for each color pair.",
    ),
    (
        "cycle-msh-hybrid",
        "A legendary hero or villain creature costed with hybrid mana.",
    ),
    (
        "cycle-dmu-mn-signpost",
        "A legendary multicolor creature built to point drafters toward its color combination.",
    ),
    (
        "cycle-mkm-u-gold-noncreature",
        "A two-color gold noncreature spell from Murders at Karlov Manor.",
    ),
    (
        "cycle-mkm-hybrid-disguise",
        "A creature from Murders at Karlov Manor with a two-color hybrid disguise cost.",
    ),
    (
        "cycle-one-draft-signpost",
        "A two-color signpost creature marking a draft archetype in Phyrexia: All Will Be One.",
    ),
    (
        "cycle-dgm-maze-runner",
        "A legendary creature from Dragon's Maze representing one of the ten Ravnica guilds.",
    ),
    (
        "typal-robot",
        "Cares about or boosts Robot creatures you control.",
    ),
    (
        "prevent-put-counter",
        "Stops counters from being put on a permanent or player.",
    ),
    (
        "cycle-cmm-tricolor-legend",
        "A three-color legendary creature reprinted in Commander Masters.",
    ),
    (
        "cycle-eld-r-m-two-color",
        "One of the ten rare or mythic two-color gold cards from Throne of Eldraine.",
    ),
    (
        "cycle-eld-hybrid",
        "One of the hybrid-mana cards from Throne of Eldraine, castable with either of two colors.",
    ),
    (
        "cycle-ema-draft-signpost",
        "A gold uncommon from Eternal Masters that points drafters toward its two-color archetype.",
    ),
    (
        "cycle-eoe-r-two-color",
        "One of the two-color rares from Edge of Eternities.",
    ),
    (
        "cycle-fdn-draft-signpost",
        "A gold card from Foundations designed to point drafters toward its two-color archetype.",
    ),
    (
        "cycle-fdn-r-two-color",
        "One of the two-color rares from Foundations found in Play Boosters.",
    ),
    (
        "cycle-fin-r-two-color",
        "One of the two-color rares from the Final Fantasy set.",
    ),
    (
        "cycle-gtc-m-two-color",
        "A Gatecrash two-color mythic rare, one themed to each of the set's guilds.",
    ),
    (
        "cycle-hbg-u-specialize",
        "A Baldur's Gate uncommon with specialize, letting you pay extra to upgrade its abilities.",
    ),
    (
        "cycle-hbg-draft-signpost",
        "A Baldur's Gate uncommon built to point drafters toward a two-color archetype.",
    ),
    (
        "synergy-colored",
        "Cares about the colors or colored mana symbols on cards you cast, reveal, or mill.",
    ),
    (
        "unique-mana-symbol",
        "Uses a mana symbol that appears on no other card, like infinity or one hundred.",
    ),
    (
        "cycle-mh3-mdfc-dual-land",
        "A Modern Horizons 3 modal double-faced card that can be cast as a spell or played as a dual land.",
    ),
    (
        "cycle-mh2-bridge",
        "A Modern Horizons 2 artifact dual land that enters tapped, is indestructible, and taps for two colors.",
    ),
    (
        "cycle-cns-draft-signpost",
        "A signpost card from Conspiracy that points drafters toward a two-color archetype.",
    ),
    (
        "cycle-mh2-c-draft-signpost",
        "A common from Modern Horizons 2 that signals a two-color draft archetype to drafters.",
    ),
    (
        "cycle-dmu-dual-land",
        "A tapped land that adds either of two colors of mana.",
    ),
    (
        "synergy-art-sticker",
        "Cares about art stickers placed on your permanents.",
    ),
    (
        "cycle-clb-u-background",
        "A Background enchantment that grants an extra ability to your commander.",
    ),
    (
        "cycle-inr-r-two-color",
        "A two-color rare that showcases its color pair's strategy.",
    ),
    (
        "sneak-aura",
        "Puts an Aura onto the battlefield attached to a creature without paying its mana cost.",
    ),
    (
        "cycle-khm-snow-tapland",
        "A dual land that enters tapped and taps for either of two colors of snow mana.",
    ),
    (
        "cycle-lci-r-two-color",
        "A rare two-color card from the Lost Caverns of Ixalan gold cycle.",
    ),
    (
        "cycle-lci-draft-signpost",
        "An uncommon that points drafters toward a two-color archetype in Lost Caverns of Ixalan.",
    ),
    (
        "cycle-mh1-r-two-color",
        "A rare two-color card from the Modern Horizons gold cycle.",
    ),
    (
        "cycle-dgm-draft-signpost",
        "An uncommon that points drafters toward one of Dragon's Maze's guild archetypes.",
    ),
    (
        "cycle-mh1-draft-signpost",
        "An uncommon that points drafters toward a two-color archetype in Modern Horizons.",
    ),
    (
        "synergy-1-1",
        "Rewards or cares about creatures that are specifically 1/1.",
    ),
    (
        "cycle-mid-draft-signpost",
        "One of Innistrad: Midnight Hunt's two-color signpost uncommons, each pointing to a draft archetype.",
    ),
    (
        "cycle-mh2-u-draft-signpost",
        "One of Modern Horizons 2's uncommon signpost cards, each built to steer you into a draft archetype.",
    ),
    (
        "cycle-mh3-u-draft-signpost",
        "One of Modern Horizons 3's uncommon signpost cards, each built to steer you into a draft archetype.",
    ),
    (
        "cycle-mh3-r-m-two-color",
        "One of Modern Horizons 3's rare or mythic two-color creatures.",
    ),
    (
        "cycle-mkm-r-two-color-legend",
        "One of Murders at Karlov Manor's rare two-color legendary creatures.",
    ),
    (
        "cycle-mkm-r-2c-noncreature",
        "One of Murders at Karlov Manor's rare two-color noncreature spells.",
    ),
    (
        "cycle-mm2-r-two-color",
        "One of Modern Masters 2015's rare two-color creatures.",
    ),
    (
        "cycle-mom-invasion-signpost",
        "A Battle card that flips into a creature or other permanent once it's been defeated.",
    ),
    (
        "synergy-flash",
        "Rewards you for casting spells or creatures with flash.",
    ),
    (
        "gives-horsemanship",
        "Grants horsemanship, so the creature can't be blocked except by other horsemanship creatures.",
    ),
    (
        "text-change-color",
        "Rewrites a permanent's or spell's text, swapping one color word for another.",
    ),
    (
        "cycle-one-r-two-color",
        "A two-color legendary rare creature from the Phyrexia: All Will Be One set.",
    ),
    (
        "cycle-clb-r-two-color-legend",
        "A two-color legendary rare creature from the Baldur's Gate set.",
    ),
    (
        "cycle-rav-bounceland",
        "A dual land that enters tapped, bounces a land you control to hand, and taps for two colors.",
    ),
    (
        "cycle-rav-backward-boost",
        "Grants a bonus effect only if a specific colored mana was spent to cast it.",
    ),
    (
        "timing-restriction",
        "A spell you can only cast under unusual timing restrictions.",
    ),
    (
        "cycle-rav-guild-artifact",
        "An iconic two-color artifact tied to one of the original ten Ravnica guilds.",
    ),
    (
        "cycle-rav-shockland",
        "A dual land that enters untapped if you pay 2 life.",
    ),
    ("typal-fungus", "Cares about or boosts Fungus creatures."),
    (
        "cycle-khm-u-saga",
        "An uncommon Saga from Kaldheim that unfolds a different effect over three chapters.",
    ),
    (
        "cycle-clu-unc-hybrid",
        "An uncommon hybrid-mana card from Ravnica: Clue Edition, one per guild pair.",
    ),
    (
        "cycle-ima-draft-signpost",
        "A card that signals a two-color archetype for drafting Iconic Masters.",
    ),
    (
        "cycle-mid-r-two-color-legend",
        "A legendary creature that signals a two-color archetype in Innistrad: Midnight Hunt limited.",
    ),
    (
        "cycle-dmu-mmn-signpost",
        "A legendary creature that signals a two-color archetype for drafting Dominaria United.",
    ),
    (
        "undergrowth-all",
        "Gets bigger or cheaper based on creature cards in all graveyards, not just your own.",
    ),
    (
        "cycle-block-rtr-u-hybrid",
        "An uncommon that signals a two-color archetype in the Return to Ravnica block.",
    ),
    (
        "cycle-cmr-draft-signpost",
        "A legendary creature that signals a two-color archetype for drafting Commander Legends.",
    ),
    (
        "cycle-mh3-landscape",
        "A cycling land that taps for colorless or sacrifices to fetch one of three basic land types.",
    ),
    (
        "cycle-cmr-backward-partner",
        "The backward half of a Commander Legends partner-legend pair, meant to team up with its forward twin.",
    ),
    (
        "cycle-ala-u-two-color",
        "A two-color uncommon from the Shards of Alara block.",
    ),
    (
        "freeze-land",
        "Keeps a land from untapping during its controller's next untap step.",
    ),
    (
        "synergy-toxic",
        "Cares about or boosts creatures you control that have toxic.",
    ),
    (
        "rummage-to-library",
        "Lets you put cards from your hand on the bottom of your library and draw that many.",
    ),
    (
        "fungusaur-effect",
        "Puts a +1/+1 counter on a creature whenever it's dealt damage and survives.",
    ),
    (
        "synergy-suspect",
        "Cares about suspected creatures, which have menace and can't block.",
    ),
    (
        "synergy-untapped",
        "Rewards you for having untapped permanents, often boosting their stats or granting protection.",
    ),
    ("copy-nonland", "Creates a copy of a nonland permanent."),
    (
        "prevent-etb",
        "Stops permanents from entering the battlefield, exiling or redirecting them instead.",
    ),
    (
        "pile-card",
        "A non-playable card used to hold counters or set-aside cards for another card's effect.",
    ),
    (
        "pillage-effect",
        "Loots or rummages cards while also creating Treasure tokens.",
    ),
    (
        "orochi-ability",
        "Taps a creature it deals combat damage to and keeps it from untapping next turn.",
    ),
    (
        "peek-face-down",
        "Lets you look at face-down creatures you don't control.",
    ),
    (
        "power-nine",
        "One of the nine Alpha cards widely considered the most powerful in the game.",
    ),
    (
        "buff-pact",
        "Taps to boost a creature, but sacrifices itself if that creature leaves the battlefield that turn.",
    ),
    (
        "no-creature-type",
        "A creature card that has no creature type at all.",
    ),
    (
        "omnivore-ability",
        "Whenever it deals combat damage to an opponent, it deals that much damage to each other opponent too.",
    ),
    (
        "mutiny",
        "Makes two creatures the same player controls fight or deal damage to each other.",
    ),
    (
        "typal-rabbit",
        "Cares about or rewards you for having Rabbit creatures.",
    ),
    (
        "typal-wolf",
        "Cares about or rewards you for having Wolf creatures.",
    ),
    (
        "loot-to-library",
        "Draws you cards, then puts a card from your hand on the bottom of or shuffles it into your library.",
    ),
    (
        "mana-cost-matters",
        "Cares about the actual symbols in a mana cost, not just its total value.",
    ),
    (
        "typal-horror",
        "Cares about or rewards you for controlling creatures of the Horror type.",
    ),
    (
        "typal-multi-sea-monster",
        "Cares about Krakens, Leviathans, Octopuses, and Serpents, sometimes Merfolk too.",
    ),
    (
        "karnstructs",
        "Creates 0/0 colorless Construct tokens that get +1/+1 for each artifact you control.",
    ),
    (
        "border-color-matters",
        "Cares about a card's border color, such as silver, black, or white borders.",
    ),
    (
        "imprinted-card-types-matter",
        "Cares about the card types of a card it exiled with a linked ability.",
    ),
    (
        "tutor-interaction",
        "Triggers or acts whenever a player searches their library, yours or an opponent's.",
    ),
    (
        "hats-matter",
        "Rewards or cares about creatures wearing hats in their card art.",
    ),
    (
        "tutor-land-desert",
        "Searches your library for a basic land or a Desert card.",
    ),
    (
        "counter-fuel-keyword",
        "Enters with a keyword counter you later remove to trigger a powerful effect.",
    ),
    (
        "hate-color-every",
        "Has or grants protection or hexproof from every color.",
    ),
    (
        "counter-fuel-fade",
        "Has fading, entering with counters you remove for value before it's sacrificed.",
    ),
    (
        "tutor-cast",
        "Searches your library for a card and lets you cast it without paying its mana cost.",
    ),
    (
        "synergy-foretell",
        "Cares about or rewards foretelling cards, exiling them face down to cast on a later turn.",
    ),
    (
        "dolmen-ability",
        "Prevents all combat damage that would be dealt to your attacking creatures.",
    ),
    (
        "gives-skulk",
        "Grants skulk to a creature, so it can't be blocked by creatures with greater power.",
    ),
    (
        "jackal-pup-ability",
        "Whenever it's dealt damage, it deals that much to its controller or you lose that much life.",
    ),
    (
        "gives-landwalk",
        "Grants landwalk, so the creature can't be blocked if the defender controls that land type.",
    ),
    (
        "empty-library",
        "Triggers or matters when a library has no cards left in it.",
    ),
    (
        "end-turn",
        "Ends the current turn early, exiling spells on the stack and skipping remaining phases.",
    ),
    (
        "eldrazi-titan",
        "A colossal Eldrazi with a powerful cast trigger and game-warping combat abilities.",
    ),
    (
        "format-matters",
        "Cares about which Magic format a card is legal in or refers to real tournament formats.",
    ),
    (
        "future-sight-engine",
        "Lets you play with your top card revealed and cast or play it straight from your library.",
    ),
    (
        "form",
        "An enchantment or planeswalker that transforms you, with big upsides and a real drawback.",
    ),
    (
        "emblem-lite",
        "Grants a lasting, game-altering effect like an emblem without actually being one.",
    ),
    (
        "cost-reducer-enchantment",
        "Makes enchantment spells you cast cost less mana.",
    ),
    (
        "synergy-protection",
        "Rewards you for having creatures with the protection keyword.",
    ),
    (
        "cost-reducer-noncreature",
        "Makes your noncreature spells cost less to cast.",
    ),
    (
        "synergy-full-hand",
        "Rewards you for having a full hand, often exactly or at least seven cards.",
    ),
    (
        "gives-myriad",
        "Grants myriad, letting an attacker make tapped attacking copies against each other opponent.",
    ),
    (
        "affinity-for-attacking",
        "Costs {1} less to cast for each creature attacking or that attacked this turn.",
    ),
    (
        "gives-lifelink-noncreature",
        "Grants lifelink to noncreature things like instants, sorceries, or planeswalkers.",
    ),
    (
        "any-zone-type-change",
        "Changes a card's types in every zone, not just while it's on the battlefield.",
    ),
    (
        "affinity-for-land-type",
        "Costs {1} less to cast for each land of a specific type you control.",
    ),
    (
        "synergy-initiative",
        "Rewards you for having the initiative.",
    ),
    (
        "super-pridemate",
        "Puts a +1/+1 counter on a creature for each life you gain, sometimes across your whole team.",
    ),
    (
        "gives-toxic",
        "Grants toxic to a creature, so its combat damage also gives players poison counters.",
    ),
    (
        "synergy-color-every",
        "Rewards you for having all five colors among your permanents at once.",
    ),
    (
        "animate-planeswalker",
        "Turns a planeswalker into a creature so it can attack and block.",
    ),
    (
        "gives-unearth",
        "Grants unearth to cards in a graveyard, letting them return to the battlefield temporarily for a cost.",
    ),
    (
        "remove-counters-player",
        "Strips all counters, often poison, from a player instead of a permanent.",
    ),
    (
        "chain-spell",
        "An instant or sorcery that its target's controller may copy and retarget by paying a cost.",
    ),
    (
        "reminder-card",
        "A helper card that tracks a game state like the monarch, initiative, or rad counters.",
    ),
    (
        "reanimate-vehicle",
        "Returns a Vehicle card from a graveyard to the battlefield.",
    ),
    (
        "reanimate-face-down",
        "Returns a creature card from a graveyard to the battlefield face down.",
    ),
    (
        "pseudo-leveler",
        "A creature that pays activated costs to grow into stronger and stronger forms.",
    ),
    (
        "sacrifice-cost-land",
        "A land that makes you sacrifice another land as part of it entering the battlefield.",
    ),
    (
        "typal-skeleton",
        "Cares about or boosts the Skeleton creature type.",
    ),
    (
        "prowess-anthem",
        "Boosts the power and toughness of creatures you control whenever you cast a noncreature spell.",
    ),
    (
        "perpetual-aura",
        "An aura that returns to your hand when it falls off the creature it enchanted.",
    ),
    (
        "typal-raccoon",
        "Rewards you for controlling or casting Raccoons and related animal types.",
    ),
    (
        "typal-non-share",
        "Rewards you for casting creatures with a creature type you don't already have.",
    ),
    (
        "nimble",
        "Can't be blocked by creatures with power 3 or greater.",
    ),
    (
        "counterspell-sweeper",
        "Counters all of your opponents' spells or abilities at once instead of just one.",
    ),
    (
        "typal-illusion",
        "Rewards you for controlling or creating Illusion creatures.",
    ),
    (
        "counterspell-planeswalker",
        "Counters a target creature or planeswalker spell.",
    ),
    (
        "mirror-gallery",
        "Voids the legend rule so you can control multiple copies of the same legendary permanent.",
    ),
    (
        "hunger-trigger",
        "Triggers an effect when a creature it damaged this turn dies.",
    ),
    (
        "krarks-other-thumb-effect",
        "Makes you roll extra dice and ignore the worst results whenever you roll.",
    ),
    (
        "tutor-planeswalker",
        "Searches your library for a planeswalker card.",
    ),
    (
        "typal-detective",
        "Cares about or builds around Detectives.",
    ),
    (
        "hate-typal-cleric",
        "Cares about Clerics on the battlefield, rewarding or removing them.",
    ),
    (
        "hexproof-counter",
        "Puts a hexproof counter on a creature, granting it hexproof.",
    ),
    (
        "tutor-land-gate",
        "Searches your library for a Gate card, and often a basic land too.",
    ),
    (
        "hate-etb",
        "Stops permanents that enter the battlefield from triggering abilities.",
    ),
    (
        "affinity-for-spells",
        "Costs less to cast for each instant and sorcery card in your graveyard.",
    ),
    (
        "hate-cycling",
        "Triggers an effect whenever any player cycles a card.",
    ),
    (
        "graveyard-fuel-land",
        "Exiles land cards from a graveyard to power an effect.",
    ),
    (
        "tongue-twister",
        "A joke tag for cards whose names are hard to say quickly.",
    ),
    (
        "gives-unstoppable",
        "Lets a creature assign combat damage as though it weren't blocked.",
    ),
    (
        "leaving-graveyard-matters",
        "Triggers or costs less when a card left your graveyard that turn.",
    ),
    (
        "tuck-outlet",
        "Lets you put a card from your hand onto or into your library.",
    ),
    (
        "differently-named-lands-matter",
        "Cares about how many differently named lands you control.",
    ),
    (
        "fractional-life-damage",
        "Deals fractional damage or gains fractional life, an Un-set joke effect.",
    ),
    (
        "etb-untapper",
        "Makes lands or other permanents enter or become untapped.",
    ),
    (
        "bablovian-faction-leader",
        "A legendary Un-set card that heads one of Bablovia's rival factions.",
    ),
    (
        "gains-banding",
        "Gains banding, so it can attack or block in a group and you assign the combat damage.",
    ),
    (
        "gives-afflict",
        "Grants afflict to other creatures, making the defending player lose life when they become blocked.",
    ),
    (
        "dnd-book",
        "A card named after a real printed Dungeons and Dragons book or adventure module.",
    ),
    (
        "gives-daunt",
        "Grants daunt, preventing creatures with power 2 or less from blocking the target.",
    ),
    (
        "fake-flying",
        "A creature without flying that can't be blocked except by creatures with flying or reach.",
    ),
    (
        "cycle-xln-draft-signpost",
        "One of Ixalan's two-color signpost cards steering drafters toward that color pair's archetype.",
    ),
    (
        "time-matters",
        "Cares about real-world or in-game time, like the current date, season, or hour.",
    ),
    (
        "synergy-vote",
        "Cares about voting: grants extra votes or rewards players whose votes match yours.",
    ),
    (
        "fallout-vault-saga",
        "A Saga with escalating chapter effects themed around a Fallout Vault.",
    ),
    (
        "cycle-tlrw-typal-token",
        "A token creature of a specific creature type, useful in typal decks.",
    ),
    (
        "synergy-goad",
        "Goads enemy creatures, forcing them to attack each combat and to attack a player other than you.",
    ),
    (
        "cycle-c19-alt-commander",
        "An alternate commander from a Commander 2019 preconstructed deck.",
    ),
    (
        "cycle-c18-alt-commander",
        "An alternate commander from a Commander 2018 preconstructed deck.",
    ),
    (
        "cycle-c17-alt-commander",
        "An alternate commander from a Commander 2017 preconstructed deck.",
    ),
    (
        "synergy-explore",
        "Triggers a bonus effect whenever a creature you control explores.",
    ),
    (
        "synergy-exhaust",
        "Cares about activating exhaust abilities, making them cheaper, copied, or more rewarding.",
    ),
    (
        "cycle-ons-typal-land",
        "A land that taps for colorless and has a small ability tied to a specific creature type.",
    ),
    (
        "ante-matters",
        "Uses the old ante mechanic, wagering cards from your library as part of its effect.",
    ),
    (
        "conjure-artifact-creature",
        "Conjures an artifact creature card into your hand, graveyard, or battlefield.",
    ),
    (
        "synergy-augment",
        "Cares about augmenting, rewarding you when you combine a card with augment onto a host.",
    ),
    (
        "cycle-lrw-typal-lord",
        "A creature that buffs other creatures of its own type, usually with +1/+1.",
    ),
    (
        "cycle-lrw-typal-legend",
        "A legendary creature from the Lorwyn tribal cycle, each tied to a creature type.",
    ),
    (
        "circle-of-protection",
        "Lets you pay mana to prevent damage to you from a chosen color, type, or creature type this turn.",
    ),
    (
        "share-counters",
        "Puts counters of a kind already on your permanents onto other creatures.",
    ),
    (
        "cycle-lrw-harbinger",
        "A creature that searches your library for a card of its tribe and puts it on top when it enters.",
    ),
    (
        "checklist-card",
        "A helper card used to mark which double-faced or meld card it represents in your deck.",
    ),
    (
        "rescue-enchantment",
        "Returns an enchantment you control to your hand, often as a cost or recurring trigger.",
    ),
    (
        "repeatable-mutagens",
        "Repeatedly creates Mutagen tokens that can be sacrificed to put a +1/+1 counter on a creature.",
    ),
    (
        "restock-artifact",
        "Puts an artifact card from a graveyard back on top of its owner's library to be drawn again.",
    ),
    (
        "restores-old-rule",
        "Brings back a rule Magic used to have, like mana burn or combat damage using the stack.",
    ),
    (
        "cda-subtype",
        "Grants itself extra creature or land types through a characteristic-defining ability.",
    ),
    (
        "type-errata-specific-insect",
        "A creature whose type line was errata'd to Insect.",
    ),
    (
        "unique-counters-matter",
        "Gets stronger or triggers based on how many different kinds of counters are present.",
    ),
    (
        "purgatory",
        "Exiles your creatures when they die and lets you return them to the battlefield later.",
    ),
    (
        "casting-restriction",
        "Can only be cast when an unusual condition is met.",
    ),
    (
        "rarity-matters",
        "Cares about a card's rarity, rewarding or punishing certain rarities.",
    ),
    (
        "pseudo-hexproof",
        "Shields a creature from being targeted by opponents, but stops short of true hexproof.",
    ),
    (
        "gives-infect",
        "Grants infect, so damage is dealt as -1/-1 counters to creatures and poison counters to players.",
    ),
    (
        "prevent-trigger",
        "Stops permanents entering the battlefield from causing abilities to trigger.",
    ),
    ("probe", "Has you draw three cards, then discard two."),
    (
        "promotes-to-commander",
        "Lets a creature that normally isn't a commander become one.",
    ),
    (
        "persecution-effect",
        "Pumps one group of creatures while shrinking another at the same time.",
    ),
    (
        "phyrexian-mana-ability",
        "Has an activated ability that can be paid with life instead of colored mana.",
    ),
    (
        "outlast-mentor",
        "Has outlast and grants a keyword ability to your creatures with +1/+1 counters.",
    ),
    (
        "opponent-sacrifices",
        "Forces an opponent to sacrifice a permanent of their choice.",
    ),
    (
        "typal-thopter",
        "Cares about or boosts the Thopter artifact creature type.",
    ),
    (
        "old-ward",
        "Counters a spell or ability that targets it unless its controller pays a cost, an early form of ward.",
    ),
    (
        "no-mercy",
        "Destroys a creature whenever it deals damage to you.",
    ),
    (
        "typal-myr",
        "Cares about or boosts the Myr artifact creature type.",
    ),
    (
        "typal-kavu",
        "Cares about Kavu creatures, rewarding or boosting the Kavu you control.",
    ),
    (
        "mana-restriction",
        "Has an unusual restriction on what kind of mana can be spent to pay for it.",
    ),
    (
        "impulse-enchantment-aura",
        "Lets you look at cards from your library and grab or play an Aura from among them.",
    ),
    (
        "keywords-matter",
        "Cares about keyword abilities, rewarding or interacting with cards that have them.",
    ),
    (
        "tutor-permanent",
        "Searches your library for any permanent card and puts it into your hand or onto the battlefield.",
    ),
    (
        "tutor-legendary",
        "Searches your library for a legendary card and puts it into your hand.",
    ),
    (
        "hate-typal-non-spirit",
        "Targets, restricts, or otherwise punishes creatures that aren't Spirits.",
    ),
    (
        "hate-typal-non-elf",
        "Punishes, damages, or destroys creatures that aren't Elves.",
    ),
    (
        "hate-typal-dragon",
        "Punishes, destroys, or blanks Dragon creatures, or seizes control of them.",
    ),
    (
        "hate-suspend",
        "Adds or removes time counters, or interacts with cards suspended in exile.",
    ),
    (
        "hate-face-down",
        "Destroys face-down creatures or stops permanents from being turned face up.",
    ),
    (
        "hate-defender",
        "Destroys creatures with defender or keeps them from blocking.",
    ),
    (
        "hate-commander",
        "Destroys or bounces an opponent's commander.",
    ),
    (
        "cycle-apc-envoy",
        "Reveals the top four cards of your library and puts matching creature type cards into your hand.",
    ),
    (
        "gives-swampwalk",
        "Grants a creature swampwalk, making it unblockable while the defender controls a Swamp.",
    ),
    (
        "grafted-skullcap",
        "Draws you an extra card each turn but forces you to discard your hand at end of turn.",
    ),
    (
        "transform-mirror",
        "A double-faced card whose back face mirrors its front face mechanically.",
    ),
    (
        "gives-rampage",
        "Grants a creature bonus power and toughness for each creature blocking it when it is blocked.",
    ),
    (
        "gives-persist",
        "Grants a creature persist, letting it return once with a -1/-1 counter when it dies.",
    ),
    (
        "hate-off-turn-cast",
        "Punishes or restricts players for casting spells outside their own turn.",
    ),
    (
        "gives-melee",
        "Grants a creature melee, giving it +1/+1 when it attacks for each opponent you attacked that combat.",
    ),
    (
        "graveyard-fuel-nonland",
        "Exiles nonland cards from a graveyard to cast or copy them.",
    ),
    (
        "tutor-creature-legendary",
        "Searches your library for a legendary creature card.",
    ),
    (
        "type-errata-specific-bird",
        "A creature whose type line was updated to specifically include Bird.",
    ),
    (
        "gives-annihilator",
        "Grants annihilator to a creature, making the defending player sacrifice permanents when it attacks.",
    ),
    (
        "gains-ward",
        "Gives itself ward, making opponents pay a cost or lose their spell or ability targeting it.",
    ),
    (
        "theft-enchantment",
        "Takes control of an enchantment from another player.",
    ),
    (
        "gains-defender",
        "Gives itself defender, so it can block but can't attack.",
    ),
    (
        "gives-drawlink",
        "Grants a creature the ability to draw you cards equal to the combat damage it deals a player.",
    ),
    (
        "tax-block",
        "Forces a creature's controller to pay extra mana to attack or block with it.",
    ),
    (
        "theft-equipment",
        "Takes control of an Equipment, often attaching it to a creature you control.",
    ),
    (
        "gives-escape",
        "Grants cards in a graveyard escape, letting you cast them back for their cost plus exiling other cards.",
    ),
    (
        "forestfall",
        "Triggers a bonus effect whenever a Forest enters the battlefield under your control.",
    ),
    (
        "affinity-for-creatures",
        "Costs less to cast for each creature you control or on the battlefield.",
    ),
    (
        "third-draw-matters",
        "Triggers an effect when you draw your third card in a turn.",
    ),
    (
        "synergy-mutate",
        "Rewards or reduces the cost of mutating creatures onto other creatures.",
    ),
    (
        "synergy-playtest",
        "Rewards casting playtest cards, often by reducing their cost or drawing you cards.",
    ),
    (
        "eternalize",
        "Lets you exile a creature card from your graveyard to make a 4/4 black Zombie copy of it.",
    ),
    (
        "synergy-proliferate",
        "Triggers a bonus effect whenever you proliferate.",
    ),
    (
        "cost-reducer-equipment",
        "Reduces the cost to cast Equipment spells or to activate equip abilities.",
    ),
    (
        "double-strike-counter",
        "Puts a counter on a creature that grants it double strike.",
    ),
    (
        "extra-upkeep",
        "Gives you one or more additional upkeep steps.",
    ),
    (
        "delayed-payment",
        "Play it now, then pay its mana cost at your next upkeep or lose the game.",
    ),
    (
        "discard-outlet-nonland",
        "Lets you discard a nonland card to fuel or trigger another effect.",
    ),
    (
        "damage-stays",
        "Makes damage stay on a creature instead of being removed during the cleanup step.",
    ),
    (
        "synergy-monocolor",
        "Cares about cards or permanents that are exactly one color.",
    ),
    (
        "gainland",
        "A land that gains you life when it enters the battlefield.",
    ),
    (
        "cycle-usg-rune-protection",
        "An enchantment that prevents damage to you from a chosen source, or can be cycled for a card.",
    ),
    (
        "gains-landwalk",
        "Gains landwalk itself, becoming unblockable while the defending player controls that land type.",
    ),
    (
        "cycle-da1-stage",
        "One of the Stages of Magic Design cycle, each card built around its own unconventional effect.",
    ),
    (
        "synergy-locus",
        "Rewards you for controlling Locus lands like Cloudpost.",
    ),
    (
        "synergy-multicolor-pair",
        "Rewards you for having permanents or spells that are exactly two colors.",
    ),
    (
        "synergy-pw-chandra",
        "Rewards you for controlling a Chandra planeswalker.",
    ),
    (
        "cycle-tr-mage",
        "One of a cycle of wizards, each tutoring an artifact of a specific mana value to your hand.",
    ),
    ("copy", "Creates a copy of a card, spell, or permanent."),
    (
        "control-attacker",
        "Lets you redirect who or what an attacking creature is attacking, or seize control of a player.",
    ),
    (
        "cycle-ori-c-spell-mastery",
        "Grants a bonus effect if you have two or more instant or sorcery cards in your graveyard.",
    ),
    (
        "synergy-face-down-cast",
        "Reduces the cost of casting face-down creature spells or helps you cast them.",
    ),
    (
        "specific-toughness-matters",
        "Rewards you for having a creature or spell with a chosen power or toughness value.",
    ),
    (
        "gives-prowess",
        "Grants prowess to other creatures, so they get +1/+1 when you cast a noncreature spell.",
    ),
    (
        "synergy-bobblehead",
        "An artifact whose ability scales with the number of Bobbleheads you control.",
    ),
    (
        "color-count-matters",
        "Cares about how many colors a card or permanent has.",
    ),
    (
        "cycle-khm-r-god",
        "A modal double-faced card with a God creature front and a legendary artifact back.",
    ),
    (
        "show-and-tell",
        "Lets each player put a permanent card from their hand onto the battlefield for free.",
    ),
    (
        "save-from-death",
        "Replaces losing the game with an alternate effect that keeps you alive.",
    ),
    (
        "shares-name-with-a-format",
        "A card whose name happens to match the name of a Magic format.",
    ),
    (
        "sneak-from-command-zone",
        "Puts a commander onto the battlefield from the command zone without casting it.",
    ),
    (
        "repeatable-junk",
        "Repeatedly makes Junk tokens, each sacrificed to exile and play your top card.",
    ),
    (
        "regrowth-equipment",
        "Returns an Aura or Equipment card from your graveyard to your hand.",
    ),
    (
        "relaxed-commander-restriction",
        "Bends Commander deckbuilding rules like color identity or deck size when it's your commander.",
    ),
    (
        "animate-dead-like",
        "An enchantment that puts a creature onto the battlefield and then becomes an Aura attached to it.",
    ),
    (
        "token-replacer",
        "Replaces the tokens you would create with different tokens instead.",
    ),
    (
        "rupture-spire",
        "A land that enters tapped and gets sacrificed unless you pay or tap another permanent.",
    ),
    (
        "alternate-cost-life",
        "Lets you pay life instead of this spell's mana cost.",
    ),
    (
        "untracked-effect",
        "Creates a lasting effect with no counter or token marking it, so you must remember it yourself.",
    ),
    (
        "typal-sneaky",
        "Rewards you for controlling Ninjas and Rogues together.",
    ),
    (
        "cycle-hbg-wilson",
        "One of a cycle of legendary bears with reach, trample, and ward that can specialize.",
    ),
    (
        "cycle-hbg-sarevok",
        "A legendary knight that pumps a creature by your graveyard's creature count and can specialize.",
    ),
    (
        "gives-flanking",
        "Grants flanking, so a blocker without flanking gets -1/-1 until end of turn.",
    ),
    (
        "cycle-bng-draft-signpost",
        "A signpost card steering drafters toward one of the set's key archetypes.",
    ),
    (
        "cycle-fut-spellshaper",
        "A creature that discards a card to create a specific token creature.",
    ),
    (
        "draw-to-seven",
        "Draws cards until your hand is refilled to a set size, no matter how few you had.",
    ),
    (
        "typal-bear",
        "A card that cares about or belongs to the Bear creature type.",
    ),
    (
        "repeatable-maps",
        "Creates Map tokens again and again, letting you explore repeatedly.",
    ),
    (
        "cycle-jud-wormfang-vertical",
        "When it enters, exiles one of your permanents, returning it when it leaves the battlefield.",
    ),
    (
        "tap-fuel-token",
        "Lets you tap untapped tokens you control as a cost to pay for mana or other effects.",
    ),
    (
        "cycle-da1-poison-tolerance",
        "Raises the number of poison counters it takes for you to lose the game.",
    ),
    (
        "fallout-perk-name",
        "A card in the Fallout set's cycle named after a perk from the game.",
    ),
    ("bounce", "Returns a permanent to its owner's hand."),
    (
        "squad-token",
        "A creature token created by the squad mechanic when you cast its card for an added cost.",
    ),
    (
        "gives-mountainwalk",
        "Grants mountainwalk, making a creature unblockable while the defender controls a Mountain.",
    ),
    (
        "cycle-m15-soul",
        "A Soul avatar with an activated ability you can also use from your graveyard by exiling it.",
    ),
    (
        "cycle-bok-genju",
        "An aura on a land that can turn it into a creature and returns to your hand if that land dies.",
    ),
    (
        "cycle-hbg-vhal",
        "A creature that gains study counters by looting, then specializes to remove them for a bonus effect.",
    ),
    (
        "cycle-unf-single-sticker",
        "A creature that gives you ten tickets on entering and lets you put a sticker on a nonland permanent you own.",
    ),
    (
        "one-off",
        "A mechanically unique card that was only ever printed once, usually as a celebration or novelty release.",
    ),
    (
        "seek-sorcery",
        "Seeks a random instant or sorcery card from your library.",
    ),
    (
        "typal-assembly-worker",
        "Cares about Assembly-Worker creatures, pumping or creating them.",
    ),
    (
        "type-errata-specific-ape",
        "A creature retroactively given the Ape creature type through errata.",
    ),
    (
        "gamble",
        "Searches your library for a card, puts it into your hand, then discards a card at random.",
    ),
    (
        "cycle-m19-mare",
        "A Horse creature that can't be blocked by one color and has a bonus ability.",
    ),
    (
        "hate-typal-coward",
        "Turns a creature into a Coward so your Warriors can't be blocked by it.",
    ),
    (
        "cycle-lea-basic-land",
        "A basic land that taps for one mana.",
    ),
    (
        "cycle-mrd-artifact-land",
        "An artifact land that taps for one mana.",
    ),
    (
        "hate-ramp",
        "Punishes players for putting lands into play, taxing ramp.",
    ),
    (
        "removes-first-strike",
        "Strips first strike from a creature.",
    ),
    (
        "synergy-vanilla",
        "Rewards or supports creatures that have no abilities.",
    ),
    (
        "type-errata-ghost",
        "A creature originally printed as a Ghost, now typed Spirit after errata.",
    ),
    (
        "cycle-dst-lucky-charm",
        "An artifact that gains you 1 life whenever any player casts a spell of a certain color or type.",
    ),
    (
        "typal-god",
        "Cares about or interacts with creatures that are Gods.",
    ),
    (
        "stronger-in-singleton-formats",
        "A card that gets better when your deck has no duplicate cards, as in singleton formats.",
    ),
    (
        "pierce",
        "Deals damage where any excess spills over to the target's controller.",
    ),
    (
        "gives-storm",
        "Grants storm to instant and sorcery spells you cast.",
    ),
    (
        "cycle-usg-2-cycling-land",
        "A land that enters tapped for one color but can be discarded for {2} to draw a card instead.",
    ),
    (
        "cycle-m21-planeswalker",
        "A planeswalker whose loyalty abilities build toward a powerful ultimate.",
    ),
    (
        "cycle-hbg-lulu",
        "A flying legend that rewards attacking with your other fliers, scaling a bonus by their count.",
    ),
    (
        "cycle-bro-command",
        "A modal sorcery or instant letting you choose two of four effects to combine in one spell.",
    ),
    (
        "cycle-hbg-amber",
        "A hasty legend that, on attack, discards your hand to draw two, scaling an effect by cards discarded.",
    ),
    (
        "cycle-hbg-lae-zel",
        "A double strike legend that triggers a bonus effect when it enters or specializes.",
    ),
    (
        "cycle-lrw-r-champion",
        "A creature that must sacrifice itself unless you exile another of its type, returning that card when it leaves play.",
    ),
    (
        "cycle-spellshaped-from-fut",
        "One of a Future Sight cycle of simple creatures shaped like classic Magic spells.",
    ),
    (
        "gains-shadow",
        "Gains shadow itself, so it blocks or is blocked only by other shadow creatures.",
    ),
    (
        "last-chance",
        "Gives you an extra turn but makes you lose the game at its end unless you win first.",
    ),
    (
        "seek-instant",
        "Seeks an instant or sorcery card from your library, usually putting it into your hand.",
    ),
    (
        "seek-permanent",
        "Seeks a permanent card from your library, usually putting it into your hand.",
    ),
    (
        "typal-warlock",
        "Cares about or boosts Warlock creatures you control.",
    ),
    (
        "alternate-cost-bounce",
        "Lets you return your own permanents to hand instead of paying the spell's mana cost.",
    ),
    (
        "typal-boar",
        "Cares about or boosts creatures of the Boar type.",
    ),
    (
        "swampfall",
        "Triggers an effect whenever a Swamp enters the battlefield under your control.",
    ),
    (
        "loot-to-exile",
        "Draws you a card, then has you exile a card from your hand instead of discarding it.",
    ),
    (
        "gives-plot",
        "Lets you exile a card so you can cast it later without paying its mana cost.",
    ),
    (
        "cycle-voice-angel",
        "One of a cycle of Angels with flying and protection from a color.",
    ),
    (
        "cycle-mh2-no-mana-cost-suspend",
        "One of a cycle of suspend cards you can cast for free once its time counters run out.",
    ),
    (
        "cycle-hbg-shadowheart",
        "One of a cycle of deathtouch Shadowheart legends that ping each player and reward losing life.",
    ),
    (
        "cycle-hbg-gale",
        "One of a cycle of Gale creatures that triggers a different effect whenever you cast an instant or sorcery.",
    ),
    (
        "cycle-hbg-karlach",
        "One of a cycle of Karlach creatures with first strike and haste that triggers a bonus effect when it specializes.",
    ),
    (
        "cycle-mmq-cateran-recruiter",
        "One of a cycle of Mercenaries that taps to search your library for a Mercenary permanent and puts it onto the battlefield.",
    ),
    (
        "cycle-tsp-no-mana-cost-suspend",
        "One of a cycle of cards with no mana cost, castable only by paying to suspend them.",
    ),
    (
        "equipless-equipment",
        "Equipment with no printed Equip cost, so it attaches to a creature some other way.",
    ),
    (
        "gives-cycling",
        "Grants cycling to cards in a player's hand that meet some condition.",
    ),
    (
        "half-mana",
        "Adds, costs, or otherwise involves half a mana symbol.",
    ),
    (
        "null-rod",
        "Stops activated abilities of artifacts from being activated.",
    ),
    (
        "regrowth-aura",
        "Returns an Aura or Equipment card from your graveyard to your hand.",
    ),
    (
        "sideboard-matters",
        "Lets you look at, cast from, or otherwise interact with a player's sideboard.",
    ),
    (
        "synergy-town",
        "Rewards you for controlling Town lands or plays well alongside them.",
    ),
    (
        "typal-citizen",
        "Cares about Citizen creatures, rewarding you for controlling or attacking with them.",
    ),
    (
        "type-errata-falcon",
        "A flying Bird creature originally printed as a Falcon before errata changed its type.",
    ),
    (
        "cycle-basic-snow-land",
        "A basic land with the snow supertype that taps for one mana.",
    ),
    (
        "creates-oracle-copy",
        "Casts a copy of a specific named card from the real Magic card pool.",
    ),
    (
        "y-value",
        "Uses Y as a second variable, often the toughness half of a +X/+Y bonus.",
    ),
    (
        "typal-ooze",
        "Cares about or rewards you for controlling Oozes.",
    ),
    (
        "turn-face-down-self",
        "Can turn itself face down, often as part of morph or a similar ability.",
    ),
    (
        "synergy-contraption",
        "Cares about or interacts with Contraptions, the sprocket-based artifact subtype.",
    ),
    (
        "royal-assassin-ability",
        "Taps to destroy a target tapped creature.",
    ),
    (
        "protects-nonland",
        "Temporarily exiles a nonland permanent, then returns it to the battlefield.",
    ),
    (
        "impulse-nonland",
        "Digs through the top of your library and puts a nonland permanent card onto the battlefield.",
    ),
    (
        "grave-pact",
        "Forces each opponent to sacrifice a creature whenever a creature you control dies.",
    ),
    (
        "freeze-permanent-any",
        "Taps a permanent, or keeps one from untapping during its controller's next untap step.",
    ),
    (
        "cycle-znr-pathway",
        "A land that enters as either of two land halves, chosen when you play it.",
    ),
    (
        "cycle-scg-mv-matters",
        "Scales its effect off the greatest mana value among permanents you control.",
    ),
    (
        "cycle-m21-sanctum",
        "A Shrine enchantment whose effect grows with the number of Shrines you control.",
    ),
    (
        "cycle-hbg-wyll",
        "A legendary Wyll that lets you sacrifice a creature or artifact for a bonus when it specializes.",
    ),
    (
        "cycle-hbg-rasaad",
        "One version of Rasaad in a cycle of legendary creatures, each with a different ability.",
    ),
    (
        "cycle-hbg-imoen",
        "One version of Imoen in a cycle of legendary creatures, each with a different ability.",
    ),
    (
        "cycle-hbg-alora",
        "One version of Alora in a cycle of legendary creatures, each with a different ability.",
    ),
    (
        "cycle-eoe-legend-before",
        "One of a cycle of paired legends depicting a character's earlier form.",
    ),
    (
        "cycle-eoe-legend-after",
        "One of a cycle of paired legends depicting a character's later form.",
    ),
    (
        "cycle-hbg-gut",
        "One version of Gut in a cycle of legendary creatures, each with a different ability.",
    ),
    (
        "cycle-hbg-klement",
        "One version of Klement in a cycle of legendary creatures, each with a different ability.",
    ),
    (
        "cycle-hbg-viconia",
        "A legendary creature that exiles graveyard cards, then specializes to conjure copies of them into your hand.",
    ),
    (
        "cycle-horizon-land",
        "An untapped dual land that costs 1 life per mana and can be sacrificed to draw a card.",
    ),
    (
        "cycle-neo-shrine",
        "A Shrine enchantment creature that lets you pay 1 each end step for an effect that scales with the number of Shrines you control.",
    ),
    (
        "cycle-ons-lord",
        "A creature that taps five untapped creatures of its type for a powerful activated ability.",
    ),
    (
        "freeze-nonland",
        "Taps a target nonland permanent and keeps it from untapping during its controller's next untap step.",
    ),
    (
        "gains-forestwalk",
        "Grants a creature forestwalk, making it unblockable while the defending player controls a Forest.",
    ),
    (
        "hate-typal-werewolf",
        "Punishes or removes Werewolf creatures specifically, such as with protection, destruction, or bounce.",
    ),
    (
        "hunger-reanimation",
        "Returns creatures it damages and kills to the battlefield under your control.",
    ),
    (
        "pseudo-exert",
        "Uses an ability that skips its next untap step as the cost for a bonus effect.",
    ),
    (
        "cycle-hbg-skanos",
        "A Dragon Vassal that grants another attacker a keyword and +X/+0 equal to its power.",
    ),
    (
        "cycle-mrd-slith",
        "A Slith that grows with a +1/+1 counter whenever it deals combat damage to a player.",
    ),
    (
        "synergy-copy",
        "Rewards you for copying spells, often by copying more or triggering off each copy.",
    ),
    (
        "hate-typal-non-choose",
        "Destroys, bounces, or weakens all creatures that aren't of a chosen creature type.",
    ),
    (
        "type-removal-cat-rakshasa",
        "A Rakshasa creature printed without the Cat type it historically carried.",
    ),
    (
        "cycle-clu-suspect",
        "A legendary creature from the Clue crossover suspect cycle, each with its own ability.",
    ),
    (
        "cycle-dom-legendary-sorcery",
        "A legendary sorcery you can only cast while controlling a legendary creature or planeswalker.",
    ),
    (
        "type-addition-frog",
        "A creature that has the Frog type in addition to its other creature types.",
    ),
    (
        "typal-plant",
        "Cares about, creates, or boosts Plant creatures you control.",
    ),
    (
        "tutor-nonland",
        "Searches your library for a nonland card and puts it into your hand or onto the battlefield.",
    ),
    (
        "synergy-nonbasic-land",
        "Cares about nonbasic lands, rewarding you for controlling or finding them.",
    ),
    (
        "pseudo-haste",
        "Lets creatures act as though they have haste, so they can attack or use abilities right away.",
    ),
    (
        "precognition",
        "Lets you look at the top card of your library any time.",
    ),
    (
        "impulse-artifact-vehicle",
        "Digs into your library for Vehicle or artifact creature cards, to your hand or the battlefield.",
    ),
    (
        "greatest-power-matters",
        "Rewards you for controlling the creature with the greatest power on the battlefield.",
    ),
    (
        "functional-art",
        "An Un-set card whose art doubles as a working tracker for how the card functions.",
    ),
    (
        "energy-increaser",
        "Increases the amount of energy or counters you get whenever you'd receive some.",
    ),
    (
        "cycle-tmc-character-select",
        "One of the Teenage Mutant Ninja Turtles commanders, each pairable as a co-commander via Partner.",
    ),
    (
        "cycle-rix-elder-dinosaur",
        "One of the Ixalan block's Elder Dinosaur legendary creatures, each a powerful mono or multicolor threat.",
    ),
    (
        "you-matter",
        "Cares about your own physical traits, like your height or your name, not anything on the battlefield.",
    ),
    (
        "cycle-hbg-jaheira",
        "A legendary creature with hexproof from artifacts and enchantments that destroys one when it specializes.",
    ),
    (
        "cycle-hbg-lukamina",
        "A shapeshifting legendary creature that specializes into an animal form and unspecializes back when it dies.",
    ),
    (
        "cycle-akh-allied-aftermath",
        "A split card whose second half can be cast later from your graveyard.",
    ),
    (
        "cycle-afr-u-class",
        "An enchantment that gains new abilities as you pay to level it up.",
    ),
    (
        "cycle-dtk-u-monocolor-dragon",
        "A megamorph Dragon with flying that puts a +1/+1 counter on your other Dragons when turned face up.",
    ),
    (
        "cycle-afr-planeswalker",
        "A legendary planeswalker themed around a Dungeons and Dragons character.",
    ),
    (
        "cycle-afr-m-dragon",
        "A legendary mythic Dragon from the Forgotten Realms cycle, each with its own signature ability.",
    ),
    (
        "cycle-dsk-m-room",
        "A mythic double-faced Room enchantment with two halves you unlock separately.",
    ),
    (
        "cycle-dual-cycling-land",
        "A tapped dual land that produces two colors of mana or can be discarded to draw a card.",
    ),
    (
        "cycle-eve-demigod-aura",
        "An Aura that buffs the enchanted creature differently depending on which of two colors it is.",
    ),
    (
        "cycle-eve-c-hybrid-1-drop",
        "A common one-mana creature castable with either of two colors, with a small ability.",
    ),
    (
        "cycle-fut-vanilla",
        "A creature with no rules text beyond power and toughness.",
    ),
    (
        "cycle-dsk-overlord",
        "A creature with impending that triggers a payoff whenever it enters or attacks.",
    ),
    (
        "cycle-afc-d20-spell",
        "Has you roll a d20 for a weaker or stronger version of its effect depending on the result.",
    ),
    (
        "cycle-afc-endeavor",
        "Rolls two dice and lets you split the results between two different effects.",
    ),
    (
        "cycle-ecc-incarnation",
        "A creature with a strong enter-the-battlefield effect and encore to attack again from the graveyard.",
    ),
    (
        "cycle-eve-c-retrace",
        "A cheap spell you can recast from your graveyard by discarding a land, thanks to retrace.",
    ),
    (
        "cycle-gn2-mythic",
        "A creature with a keyword ability and a powerful extra effect.",
    ),
    (
        "cycle-gn3-mythic",
        "A legendary creature built around a strong payoff tied to its own theme or tribe.",
    ),
    (
        "cycle-fut-sliver",
        "A Sliver that gives all Slivers an extra ability.",
    ),
    (
        "cycle-ecl-c-hybrid",
        "One of five common creatures with a hybrid mana cost, one per color pair.",
    ),
    (
        "cycle-bok-flip-creature",
        "A creature that flips into a more powerful legendary Spirit once its condition is met.",
    ),
    (
        "cycle-dsk-u-room",
        "A two-part Room enchantment where you unlock a second linked door for more value.",
    ),
    (
        "cycle-eoe-r-mono-spacecraft",
        "A mono-color Spacecraft that gains abilities as you Station it by tapping creatures for charge counters.",
    ),
    (
        "cycle-aer-legend",
        "One of five mono-color legendary creatures, one per color, each with its own powerful ability.",
    ),
    (
        "cycle-aer-implement",
        "A cheap artifact you sacrifice for a small effect, then draw a card when it hits the graveyard.",
    ),
    (
        "cycle-eve-avatar",
        "A powerful hybrid-cost Spirit Avatar with a strong evasive or game-warping ability.",
    ),
    (
        "card-style-matters",
        "Cares about a card's physical style, like foiling, alternate art, or being signed.",
    ),
    (
        "castable-from-library",
        "Lets you cast this card straight from your library instead of your hand.",
    ),
    (
        "cycle-aer-automaton",
        "One of a five-color cycle of artifact creatures with a colored activated ability.",
    ),
    (
        "cycle-aer-expertise",
        "One of a cycle that does an effect, then lets you cast a cheap spell from hand for free.",
    ),
    (
        "cycle-eve-filterland",
        "One of a cycle of filter lands that turns one mana into two of its color pair.",
    ),
    (
        "cycle-fut-utilityland",
        "One of a cycle of tapped lands, each with its own small extra ability.",
    ),
    (
        "cycle-gnt-mythic",
        "One of a cycle of creatures whose enter-the-battlefield effect scales with your number of opponents.",
    ),
    (
        "cycle-acr-saga",
        "An Assassin's Creed Saga that unfolds a new effect each chapter as lore counters accrue.",
    ),
    (
        "cycle-fut-magus",
        "A wizard creature that recreates the ability of a famous older card.",
    ),
    (
        "card-game-reference",
        "A card whose text or flavor references another trading card game.",
    ),
    (
        "cycle-a25-u-legend",
        "An uncommon legendary creature reprinted in the Masters 25 anthology set.",
    ),
    (
        "cycle-fut-grandeur-legend",
        "A legendary creature with Grandeur, letting you discard a second copy of itself for a bonus effect.",
    ),
    (
        "cycle-fut-pact",
        "An instant with a free effect now that makes you pay a cost next upkeep or lose the game.",
    ),
    (
        "cycle-fut-augur",
        "A creature you can sacrifice during your upkeep for a one-time payoff.",
    ),
    (
        "cycle-5dn-mm-attach-equipment",
        "An Equipment with a colored activated ability that lets you attach it to a creature without using equip.",
    ),
    (
        "cycle-a25-allied-enhanced",
        "A card whose extra bonus or ability ties into its allied color of mana or permanents.",
    ),
    (
        "cycle-blb-r-talent",
        "A Class enchantment that gives an ability at level 1, then unlocks stronger ones as you level it up.",
    ),
    (
        "cycle-blb-u-offspring",
        "Has offspring, letting you pay extra as you cast it to also get a 1/1 token copy when it enters.",
    ),
    (
        "cycle-fut-c-cycling",
        "Has cycling: pay a cost and discard it to draw a card.",
    ),
    (
        "cycle-fut-dual-land",
        "A land that taps for two colors of mana, often with a drawback or extra cost.",
    ),
    (
        "cycle-fut-recurring-suspend",
        "A suspend spell that re-exiles itself with time counters as it resolves, so it keeps recurring.",
    ),
    (
        "cycle-5dn-bringer",
        "A creature you can cast for one of each color that has trample and a powerful upkeep trigger.",
    ),
    (
        "cycle-5dn-beacon",
        "A spell with a strong effect that shuffles itself back into your library after resolving.",
    ),
    (
        "cycle-dsk-landcycler",
        "A creature you can discard to search your library for a land of a given basic type.",
    ),
    (
        "cycle-frf-modal-spell",
        "A spell that lets you choose one of two different effects when you cast it.",
    ),
    (
        "cycle-dsk-verge",
        "A land that taps for one color, or a second color if you control a matching basic land type.",
    ),
    (
        "cycle-frf-runemark",
        "An aura that gives +2/+2 and grants a keyword ability while you control permanents of certain colors.",
    ),
    (
        "cycle-frf-siege",
        "An enchantment that locks in one of two different ongoing effects when it enters.",
    ),
    (
        "cunning",
        "Lets an unblocked attacker deal its damage to a target creature instead of the defending player.",
    ),
    (
        "cycle-1mv-tutor",
        "A one-mana instant or sorcery that searches your library for a card.",
    ),
    (
        "cycle-frf-khan",
        "A legendary creature with a powerful ability you activate by paying hybrid mana.",
    ),
    (
        "cycle-dtk-r-two-color-dragon",
        "A rare two-color Dragon with flying and a powerful ability.",
    ),
    (
        "cycle-dtk-r-megamorpher",
        "A creature with megamorph that triggers a powerful effect when turned face up.",
    ),
    (
        "cycle-ecl-c-hybrid-changeling",
        "A common changeling creature that counts as every creature type and has one keyword ability.",
    ),
    (
        "cycle-eoe-planet",
        "A land that taps for mana and can be stationed with creatures to unlock a powerful late-game ability.",
    ),
    (
        "cycle-frf-modal-etb-creature",
        "A creature that lets you choose between getting a +1/+1 counter or a different effect when it enters.",
    ),
    (
        "cycle-frf-u-4mm-dragon",
        "A flying Dragon with an extra ability tied to attacking, dealing combat damage, or dying.",
    ),
    (
        "cycle-dft-tyrant",
        "A legendary creature from the Tyrant cycle, each with its own unique abilities.",
    ),
    (
        "cycle-bbd-bond-land",
        "A dual land that enters untapped if you're in a multiplayer game with two or more opponents.",
    ),
    (
        "cycle-bbd-c-two-color",
        "A common creature or spell from a two-color cycle, each with a simple standalone ability.",
    ),
    (
        "cycle-bbd-friend-foe",
        "Labels each player friend or foe, giving friends a benefit and foes a drawback.",
    ),
    (
        "cycle-bbd-r-two-color",
        "Marks a two-color rare from the Battlebond cycle.",
    ),
    (
        "cycle-bfz-blighted-land",
        "A land that taps for colorless mana or sacrifices for a powerful one-time effect.",
    ),
    (
        "cycle-bfz-color-landfall",
        "Triggers a landfall effect, boosted when the entering land is a matching basic type.",
    ),
    (
        "cycle-bfz-landfall-2-2-pump",
        "Gets +2/+2 until end of turn whenever a land enters under your control.",
    ),
    (
        "cycle-bfz-retreat",
        "An enchantment that lets you choose one of two effects whenever a land you control enters.",
    ),
    (
        "cycle-bfz-tangoland",
        "A dual land that enters tapped unless you control two or more basic lands.",
    ),
    (
        "cycle-bfz-utilityland",
        "A land that enters tapped and grants a one-time bonus effect when it enters.",
    ),
    (
        "cycle-blb-c-gift",
        "Lets you give an opponent a token as you cast this spell for an added effect.",
    ),
    (
        "cycle-blb-c-offspring",
        "Lets you pay extra to create a 1/1 token copy of the creature when it enters.",
    ),
    (
        "cycle-blb-m-mono-calamity",
        "A mono-colored legendary creature with a powerful, game-defining ability.",
    ),
    (
        "cycle-fut-scry-reveal",
        "Scries, then reveals the top card of your library to trigger an effect based on it.",
    ),
    (
        "cycle-blb-season",
        "A modal spell where you spend up to five points across repeatable effects of increasing cost.",
    ),
    (
        "cycle-gpt-leyline",
        "An enchantment you may put onto the battlefield for free if it starts in your opening hand.",
    ),
    (
        "cycle-blb-u-talent",
        "A Class enchantment that gains stronger abilities as you pay to level it up.",
    ),
    (
        "cycle-blb-valley-caller",
        "A creature whose ability keys off a group of four small creature types you control.",
    ),
    (
        "cycle-blb-village",
        "A land tapping for colorless or a creatures-only color, plus a typal creature-type ability.",
    ),
    (
        "cycle-a25-same-name-enhanced",
        "A card that grows stronger or fetches copies based on how many share its exact name.",
    ),
    (
        "cycle-dsk-glimmer",
        "A creature that returns to the battlefield as a noncreature enchantment when it dies.",
    ),
    (
        "cycle-fin-u-summon",
        "A Saga that's also a creature, ticking through chapter effects before it's sacrificed.",
    ),
    (
        "cycle-block-bfz-creatureland",
        "A land that enters tapped, taps for two colors, and can become a creature for the turn by paying mana.",
    ),
    (
        "cycle-acr-auto-equip",
        "Equipment that attaches itself to a creature you control automatically when it enters the battlefield.",
    ),
    (
        "cycle-block-zen-monocolor-pw",
        "A monocolor planeswalker from the Zendikar block cycle, each built around a distinct theme.",
    ),
    (
        "cycle-bng-archetype",
        "Grants a keyword ability to your creatures while stripping that same keyword from opponents' creatures.",
    ),
    (
        "cycle-bng-devotion-x",
        "An effect that scales with X, where X equals your devotion to one color.",
    ),
    (
        "cycle-eld-syr-legend",
        "A legendary Human Knight from Throne of Eldraine whose name starts with Syr.",
    ),
    (
        "cycle-aer-aether-servo-creator",
        "A creature that enters with energy and can spend it when attacking to create a 1/1 Servo token.",
    ),
    (
        "cycle-bng-fated-spell",
        "An instant with a strong effect that also lets you scry 2 if it's your turn.",
    ),
    (
        "cycle-bng-inspired-token",
        "A creature with inspired that lets you pay mana to create a token whenever it becomes untapped.",
    ),
    (
        "cycle-bng-minor-god",
        "A god creature that's only a creature while your devotion to its colors is high enough, and is indestructible.",
    ),
    (
        "cycle-bng-nyxborn",
        "A creature that can be cast for its bestow cost as an Aura granting a stat boost instead.",
    ),
    (
        "cycle-bng-tapping-aura",
        "An Aura that grants the enchanted creature a tap ability, like draining life or drawing a card.",
    ),
    (
        "cycle-bng-u-bestow",
        "A creature with bestow that grants the enchanted creature a stat boost plus an extra ability.",
    ),
    (
        "cycle-bng-u-tribute",
        "A creature with tribute: an opponent chooses whether to buff it or let its enters-the-battlefield effect happen.",
    ),
    (
        "cycle-bok-baku",
        "A Spirit that gains ki counters from Spirit and Arcane spells to fuel a removal or utility ability.",
    ),
    (
        "cycle-force-elemental",
        "A large creature whose effect triggers at the beginning of each player's upkeep.",
    ),
    (
        "cycle-dom-memorial",
        "A land that enters tapped, taps for one color, and sacrifices itself for a bigger effect.",
    ),
    (
        "cycle-dtk-regent",
        "A flying Dragon with a strong enters-the-battlefield or triggered ability.",
    ),
    (
        "cycle-bok-kami-patron",
        "A legendary Spirit you cast by sacrificing a matching creature to cover part of its cost.",
    ),
    (
        "cycle-bok-m-sac-spirit",
        "A Spirit you can sacrifice for a small one-shot effect.",
    ),
    (
        "cycle-bok-nomana-splice-arcane",
        "An Arcane instant you can splice onto another Arcane spell by paying a non-mana cost.",
    ),
    (
        "cycle-bok-shoal",
        "Cast it by exiling a same-color card; its effect scales with that card's mana value.",
    ),
    (
        "cycle-bro-basic-land-count",
        "An effect that scales with how many of a specific basic land type you control.",
    ),
    (
        "cycle-bro-c-mulch",
        "Mills three cards when it enters, letting you grab a matching card into hand or grow with a +1/+1 counter instead.",
    ),
    (
        "creature-type-phantasm",
        "A creature of the Illusion type, often flying and easily destroyed.",
    ),
    (
        "cycle-bro-m-color-artifact",
        "An artifact creature with prototype, castable at reduced size and cost in a single color.",
    ),
    (
        "cycle-afr-chromatic-dragon",
        "A flying Dragon with an enters-the-battlefield breath effect keyed to its color.",
    ),
    (
        "cycle-afr-color-hate",
        "An effect that gets stronger against permanents of one specific color.",
    ),
    (
        "cycle-dsk-leyline",
        "An enchantment you may start the game with in play for free, granting a powerful passive effect.",
    ),
    (
        "cycle-afr-creatureland",
        "A land that can pay mana to become a creature until end of turn while staying a land.",
    ),
    (
        "cycle-c13-alt-commander",
        "An alternate legendary creature offered as a commander option in a Commander 2013 deck.",
    ),
    (
        "cycle-c13-curse",
        "An Aura Curse from the Commander 2013 cycle that enchants a player and triggers when that player is attacked.",
    ),
    (
        "cycle-c13-face-commander",
        "The headline legendary creature that a Commander 2013 precon deck is built around and named after.",
    ),
    (
        "cycle-c13-reprint-commander",
        "A previously printed legendary creature reprinted as a commander in Commander 2013.",
    ),
    (
        "cycle-c13-tempting-offer",
        "A sorcery that gives you an effect and invites each opponent to copy it, paying you off again for each who accepts.",
    ),
    (
        "cycle-c14-alt-commander",
        "An alternate legendary creature offered as a commander option in a Commander 2014 deck.",
    ),
    (
        "cycle-c14-historical-legend",
        "A notable legendary creature from Magic's earlier sets, reprinted in the Commander 2014 cycle.",
    ),
    (
        "cycle-c14-lieutenant",
        "Grants itself +2/+2 and a bonus ability as long as you control your commander.",
    ),
    (
        "cycle-c14-offering",
        "Lets you choose an opponent to share two symmetrical effects with.",
    ),
    (
        "cycle-eld-u-adamant-spell",
        "Gains a bonus effect if you spent at least three mana of one color casting it.",
    ),
    (
        "cycle-frf-c-monocolor-manifest",
        "Manifests a card onto the battlefield face down as a 2/2 creature.",
    ),
    (
        "cycle-arb-u-tricolor",
        "A three-color card, one per shard, whose effects vary widely.",
    ),
    (
        "cycle-dom-m-legend",
        "A legendary creature built around a signature enters or ongoing ability.",
    ),
    (
        "cycle-dtk-dragonlord",
        "A legendary Elder Dragon with flying and a powerful signature ability.",
    ),
    (
        "cycle-arb-shard-blade",
        "A creature that gets +1/+1 and a bonus keyword while you control another multicolored permanent.",
    ),
    (
        "cycle-arb-sojourner",
        "A cycling creature that triggers a small bonus effect both when you cycle it and when it dies.",
    ),
    (
        "cycle-arb-u-cascade",
        "Has cascade: casting it exiles cards from your library until you hit a cheaper nonland card you can cast free.",
    ),
    (
        "cycle-csp-kindle-spell",
        "A spell whose effect scales up for each copy of itself in any graveyard.",
    ),
    (
        "cycle-dmu-wedge-kicker",
        "Has a two-color kicker you can pay once or twice, giving a bigger bonus each time.",
    ),
    (
        "cycle-eld-r-adventurer",
        "A creature with an Adventure: cast its instant or sorcery first, then cast the creature later from exile.",
    ),
    (
        "cycle-frf-c-two-color",
        "A two-color instant with a modest effect that rewards playing both its colors.",
    ),
    (
        "cycle-eld-court-artifact",
        "A legendary artifact with a built-in cost reduction and a strong ongoing effect.",
    ),
    (
        "cycle-arb-hybrid-cycler",
        "Has cycling for a hybrid mana cost, letting you discard it to draw a card.",
    ),
    (
        "cycle-arb-dual-landcycler",
        "Can be discarded and a mana cost paid to search your library for either of two basic land types.",
    ),
    (
        "bottom-deck-manipulation",
        "Lets you filter through cards and put the ones you don't keep on the bottom of your library.",
    ),
    (
        "cycle-eld-court-leader",
        "A legendary creature that rewards you for building around its own color.",
    ),
    (
        "cycle-eld-color-equipment",
        "A cheap, mono-colored Equipment that boosts the creature it's attached to.",
    ),
    (
        "cycle-dtk-enemy-hate",
        "Removal or disruption aimed specifically at creatures or permanents of two enemy colors.",
    ),
    (
        "cycle-eld-color-hate",
        "Punishes or resists a specific color, dealing more damage, costing less, or countering that color.",
    ),
    (
        "cycle-arb-c-enemy",
        "A two-color enemy-color common, each with its own ability, from a printed cycle.",
    ),
    (
        "cycle-con-backward-synergy",
        "Gets a bonus if you control a permanent of a specific off-color.",
    ),
    (
        "cycle-arb-c-cascade",
        "Has cascade: when you cast it, you cast a cheaper spell from your library for free.",
    ),
    (
        "cycle-eld-paladin",
        "A knight that enters with a +1/+1 counter if you spent three or more mana of its color to cast it.",
    ),
    (
        "cycle-ema-tutor",
        "Searches your library for a card, putting it into your hand, on top, or onto the battlefield.",
    ),
    (
        "cycle-frf-clan-enhanced",
        "Gains an extra ability as long as you control a permanent of either of two specific colors.",
    ),
    (
        "cycle-arb-borderposts",
        "An artifact that enters tapped, taps for one of two colors, and can be cast for {1} plus a returned basic land.",
    ),
    (
        "cycle-cn2-r-monarch",
        "A creature that makes you the monarch when it enters, drawing you a card each end step until someone takes the crown.",
    ),
    (
        "cycle-arb-allied-equipment",
        "Equipment that boosts the equipped creature or grants it a keyword or combat ability.",
    ),
    (
        "cycle-ecl-u-changeling",
        "A changeling creature, counting as every creature type, with an added enters or dies trigger.",
    ),
    (
        "cycle-apc-volver",
        "A kicker creature that enters with extra +1/+1 counters and abilities for each kicker cost you paid.",
    ),
    (
        "cycle-eld-adamant-land",
        "A land that enters tapped unless you control three or more other lands of its basic type, giving a bonus when it enters untapped.",
    ),
    (
        "cycle-fin-crystal",
        "A legendary artifact that discounts spells of one color, boosts that color's signature effect, and has a tap ability using that color.",
    ),
    (
        "block-without-creature",
        "Makes an attacking creature become blocked without any creature actually blocking it.",
    ),
    (
        "cycle-apc-sanctuary",
        "An enchantment that triggers a small effect each upkeep, larger if you control both of its colors.",
    ),
    (
        "cycle-apc-split-card",
        "A split card that lets you cast either of its two halves.",
    ),
    (
        "cycle-dmu-u-back-en-kicker",
        "Has a kicker cost that upgrades or adds to its effect if you pay the extra mana.",
    ),
    (
        "cycle-dmu-u-for-ally-kicker",
        "Has a kicker cost that adds a bonus effect on top of its base effect if you pay the extra mana.",
    ),
    (
        "cycle-dmu-u-for-en-kicker",
        "Has a kicker cost that adds a bonus effect on top of its base effect if you pay the extra mana.",
    ),
    (
        "cycle-dtk-command",
        "A modal instant that lets you choose two of four listed effects to resolve.",
    ),
    (
        "cycle-fin-landcycler",
        "A creature you can discard to search your library for a land of a basic land type.",
    ),
    (
        "cycle-dtk-draft-signpost",
        "A flying Dragon whose ability signals and rewards a two-color draft strategy.",
    ),
    (
        "cycle-eld-c-adamant-spell",
        "A spell with a bonus effect if you spent at least three mana of one color casting it.",
    ),
    (
        "cycle-emn-allied-creature",
        "A creature from a cycle of allied two-color gold cards.",
    ),
    (
        "cycle-apc-disciple",
        "A creature with two tap abilities, each activated with a different color of mana.",
    ),
    (
        "cycle-dmu-u-saga",
        "A Saga that lets you choose which chapter to start on, then plays out its chapters in order.",
    ),
    (
        "cycle-ecl-typal-signpost",
        "A creature that boosts or triggers off other creatures sharing its creature type.",
    ),
    (
        "cycle-fem-storage-land",
        "A land that enters tapped and can hoard storage counters to release as extra mana later.",
    ),
    (
        "cycle-all-u-two-color",
        "A two-color card belonging to an allied color-pair cycle.",
    ),
    (
        "cycle-all-replacement-land",
        "A land that enters only if you sacrifice a matching basic land instead.",
    ),
    (
        "cycle-dmu-u-back-ally-kicker",
        "Has kicker: pay extra as you cast it for a stronger effect if it's kicked.",
    ),
    ("cycle-all-r-tricolor", "A three-color card."),
    (
        "cycle-all-pitch-spell",
        "A spell you can cast by exiling a matching-colored card from your hand instead of paying its mana cost.",
    ),
    (
        "cycle-dom-r-mmm-creature",
        "A mono-colored creature with unusually strong stats or a strong ability for its mana cost.",
    ),
    (
        "cycle-ecl-u-behold",
        "A creature that costs 2 more unless you behold a creature of its type to cast it.",
    ),
    (
        "cycle-fin-adventure-land",
        "A land with an Adventure spell you can cast first, then later play the land from exile.",
    ),
    (
        "cycle-fin-sidequest",
        "An enchantment that transforms into its back face once you complete its side quest.",
    ),
    (
        "cycle-frf-dragonlord",
        "A legendary Dragon that triggers an effect whenever a Dragon you control attacks.",
    ),
    (
        "cycle-clb-r-background",
        "A Background enchantment that grants abilities or bonuses to the commander creatures you own.",
    ),
    (
        "cycle-all-enemy-hate",
        "Punishes players for controlling lands or permanents of specific enemy colors.",
    ),
    (
        "cycle-ala-u-tricolor",
        "A three-color creature with efficient stats and at most a simple keyword or ability.",
    ),
    (
        "cycle-clb-legend-spell",
        "A cycle of instants tied to Baldur's Gate legends, each with a unique powerful effect.",
    ),
    (
        "cycle-dmu-jumpstart",
        "A cycle of Jumpstart-exclusive Dominaria United cards, one per color with a themed ability.",
    ),
    (
        "cycle-ecl-r-dfc",
        "A cycle of rare double-faced cards, each a legendary creature on both faces.",
    ),
    (
        "cycle-clb-r-mono-legend",
        "A cycle of mono-colored legendary creatures that can pair with a Background as your commander.",
    ),
    (
        "cycle-clb-thriving-gate",
        "A cycle of Gate lands that enter tapped and let you choose a second color to add.",
    ),
    (
        "cycle-clb-u-d20",
        "A cycle of spells and creatures that roll a d20, with stronger effects on higher results.",
    ),
    (
        "cycle-clb-u-initiative",
        "A cycle of cards that give you the initiative when they enter or resolve.",
    ),
    (
        "cycle-clu-clue-equipment",
        "A Clue-themed Equipment that buffs the wearer and can be sacrificed to draw a card.",
    ),
    (
        "cycle-clb-invoker",
        "A creature with a costly activated ability that unleashes a themed magical effect.",
    ),
    (
        "cycle-cmb1-dual-land",
        "A land that taps for either of two colors, usually with a drawback or bonus effect.",
    ),
    (
        "cycle-cmd-alt-commander",
        "A legendary creature offered as an alternate commander choice for its deck's theme.",
    ),
    (
        "cycle-cmd-enemy-legend",
        "A legendary creature whose color identity is an enemy color pair.",
    ),
    (
        "cycle-cmd-face-commander",
        "The headline legendary creature built to lead its preconstructed Commander deck.",
    ),
    (
        "cycle-cmd-join-forces",
        "Uses join forces: each player may pay mana to scale up a shared effect.",
    ),
    (
        "cycle-cmd-vow",
        "Grants an enchanted creature a stat boost and a keyword, but it can't attack you or your planeswalkers.",
    ),
    (
        "cycle-apc-painland",
        "A land that taps for colorless freely or for one of two colors while dealing 1 damage to you.",
    ),
    (
        "cycle-dtk-khan",
        "A legendary creature or planeswalker showcasing one of the Tarkir clans' mechanics.",
    ),
    (
        "cycle-cmm-sliver",
        "A Sliver that gives all your Slivers a shared extra ability.",
    ),
    (
        "cycle-cmr-artifact-partner",
        "A legendary artifact-focused creature with partner, letting it pair as one of two commanders.",
    ),
    (
        "cycle-cmr-bond-land",
        "A dual land that enters untapped only if you have two or more opponents.",
    ),
    (
        "cycle-cmr-court",
        "Makes you the monarch on entry, with a stronger upkeep effect while you stay monarch.",
    ),
    (
        "cycle-apc-r-tricolor",
        "A three-color rare from the Apocalypse cycle of five tricolor cards.",
    ),
    (
        "cycle-cmr-familiar",
        "A cycle of Partner legends that buff or protect the commanders you control.",
    ),
    (
        "cycle-cmr-m-partner",
        "A cycle of mythic Partner commanders, each built around a unique signature ability.",
    ),
    (
        "cycle-cmr-m-sorcery",
        "A cycle of splashy, high-impact mythic sorceries.",
    ),
    (
        "cycle-cmr-r-partner",
        "A cycle of rare Partner commanders, each built around a unique signature ability.",
    ),
    (
        "cycle-cmr-vow",
        "A cycle of Auras that pump the enchanted creature, grant it a keyword, and stop it attacking you.",
    ),
    (
        "cycle-cmr-will",
        "A cycle of modal spells that let you choose both modes instead of one if you control a commander.",
    ),
    (
        "cycle-cn2-c-draft",
        "A common with a draft-matters ability that tracks or reveals info as you pick cards.",
    ),
    (
        "cycle-cn2-color-conspiracy",
        "A hidden agenda that secretly names a card, granting creatures with that name a bonus ability.",
    ),
    (
        "cycle-cn2-r-draft",
        "A rare with a draft-matters ability that pays off in play based on how you drafted.",
    ),
    (
        "cycle-ala-shardland",
        "A tapped land that adds one of three colors matching an Alara shard.",
    ),
    (
        "cycle-cn2-u-draft",
        "An uncommon that lets you note info about cards as you draft, then rewards you for it in play.",
    ),
    (
        "cycle-cns-m-monocolor",
        "A powerful monocolored creature reprinted in the Conspiracy set.",
    ),
    (
        "cycle-cns-pp-counter-recycler",
        "Enters with +1/+1 counters set by a game-state count, then removes one for a repeatable effect.",
    ),
    (
        "bible-reference",
        "A card whose name or effect is a direct reference to a passage from the Bible.",
    ),
    (
        "cycle-con-basic-landcycling",
        "One of a cycle of commons you can discard to search your library for a basic land.",
    ),
    (
        "cycle-con-c-domain-spell",
        "One of a common cycle using domain, scaling its effect with your basic land types.",
    ),
    (
        "cycle-con-c-two-color",
        "One of a cycle of common two-color cards, each with a small effect tied to its color pair.",
    ),
    (
        "cycle-con-enemy-hate",
        "One of a common cycle that damages or disables creatures and permanents of one specific color pair.",
    ),
    (
        "cycle-arb-c-hybrid-gold",
        "One of a cycle of common gold cards, each with a unique effect tied to its color pair.",
    ),
    (
        "cycle-arb-crossed-shard",
        "One of an Alara Reborn cycle pairing an ability from one shard with the colors of its opposing shard.",
    ),
    (
        "cycle-eld-castle",
        "A land that enters tapped unless you control a matching basic land type, and taps for one color.",
    ),
    (
        "cycle-con-outlander",
        "A creature with protection from one color, part of a five-card color-hosing cycle.",
    ),
    (
        "cycle-con-r-tricolor",
        "A rare three-color creature with its own unique, powerful ability, part of a cycle.",
    ),
    (
        "cycle-con-shard-ability",
        "A creature with an activated ability that costs two colored mana, part of a matching cycle.",
    ),
    (
        "cycle-con-u-tricolor",
        "An uncommon three-color card with its own distinct ability, part of a cycle.",
    ),
    (
        "cycle-con-u-two-color",
        "An uncommon two-color card with its own distinct ability, part of a cycle.",
    ),
    (
        "cycle-arb-legend",
        "A legendary three-color creature with a powerful signature ability, part of a cycle.",
    ),
    (
        "cycle-csp-allied-cumulative",
        "A permanent with cumulative upkeep in an allied color pair whose payoff grows with its age counters.",
    ),
    (
        "cycle-csp-enemy-hate",
        "A card that hoses spells, permanents, or players of two enemy colors.",
    ),
    (
        "cycle-clb-gem-dragon",
        "A Dragon creature paired with an Adventure spell, part of a gem-themed dragon cycle.",
    ),
    (
        "cycle-csp-martyr",
        "A creature you sacrifice while revealing X cards of its color for an effect that scales with X.",
    ),
    (
        "cycle-csp-pitchspell",
        "A spell you can cast for free by exiling two cards of its color instead of paying its cost.",
    ),
    (
        "cycle-csp-r-tricolor",
        "A rare creature built around a three-color combination.",
    ),
    (
        "cycle-arb-u-hybrid-gold",
        "An uncommon gold or hybrid-mana card built around a two-color combination.",
    ),
    (
        "cycle-csp-snow-tapland",
        "A two-color snow land that enters tapped and taps for either of its colors.",
    ),
    (
        "cycle-csp-surging-spell",
        "Has ripple: reveal the top cards of your library and cast same-named spells for free.",
    ),
    (
        "cycle-csp-u-two-color",
        "A two-color card, one of a cycle spanning different color pairs.",
    ),
    (
        "cycle-da1-charm",
        "A modal spell that lets you choose one of three different effects.",
    ),
    (
        "cycle-da1-commander-tax",
        "Scales its effect based on the highest commander tax you've paid among your commanders.",
    ),
    (
        "cycle-da1-detective",
        "Gets stronger or unlocks its ability once all of your commanders have been revealed.",
    ),
    (
        "cycle-da1-mono-eminence",
        "Has Mono Eminence, granting a bonus while your deck's color identity is a single color.",
    ),
    (
        "affinity-for-domain",
        "Costs less to cast for each basic land type among the lands you control.",
    ),
    (
        "cycle-clb-forward-enemy-legend",
        "A legendary creature that can pair with a Background as your commander.",
    ),
    (
        "cycle-da1-spell-commander",
        "A sorcery that can serve as your commander in addition to its normal effect.",
    ),
    (
        "cycle-da1-taught-by",
        "An aura that attaches to your commander, draws you a card when cast, and grants it an ability.",
    ),
    (
        "cycle-da1-unclaimed",
        "Has one effect if you're on the Mirran team and a different effect if you're on the Phyrexian team.",
    ),
    (
        "cycle-dft-gearhulk",
        "An artifact creature that triggers a powerful effect when it enters the battlefield.",
    ),
    (
        "cycle-dft-roads",
        "A land that enters tapped unless you control a Mount or Vehicle, and can be sacrificed for a Pilot token.",
    ),
    (
        "cycle-dft-surveyor",
        "A creature with start your engines that you exile from your graveyard at max speed to draw a card.",
    ),
    (
        "cycle-ala-shard-ultimatum",
        "A costly sorcery in a shard's three colors with a powerful, game-swinging effect.",
    ),
    (
        "cycle-dft-verge",
        "A dual land that taps for one color freely, and for its second color only if you control a land of a matching basic type.",
    ),
    (
        "cycle-dgm-gatekeeper",
        "A creature that gives you a bonus effect when it enters if you control two or more Gates.",
    ),
    (
        "cycle-dgm-maze-elemental",
        "A creature with a keyword that also grants that keyword to your multicolored creatures.",
    ),
    (
        "cycle-dgm-r-fuse",
        "A split card with fuse, letting you cast both halves together for their combined mana cost.",
    ),
    (
        "cycle-dis-eidolon",
        "A creature you sacrifice with one colored mana for a small effect, then return from your graveyard by casting a multicolored spell.",
    ),
    (
        "cycle-dis-r-split",
        "A split card letting you cast one of two spells in a guild's two colors.",
    ),
    (
        "cycle-dis-u-split",
        "A split card letting you cast one of two spells in a guild's two colors.",
    ),
    (
        "cycle-dka-allied-flashback",
        "Can be cast from your graveyard via flashback for a cost in an allied color.",
    ),
    (
        "cycle-dka-enemy-flashback",
        "Can be cast from your graveyard via flashback for a cost in an enemy color.",
    ),
    (
        "cycle-dka-enemy-utilityland",
        "A land that taps for colorless mana or pays a two-color cost for a stronger effect.",
    ),
    (
        "cycle-dka-increasing-flashback",
        "Doubles its effect when cast from your graveyard using flashback.",
    ),
    (
        "cycle-dst-echoing-spell",
        "Affects target permanent and every other permanent sharing its name.",
    ),
    (
        "cycle-fdn-enemy-hate",
        "Punishes or removes spells and creatures of specific enemy colors.",
    ),
    (
        "cycle-mir-two-color-enemy-hate",
        "A creature that resists or punishes cards of a specific enemy color.",
    ),
    (
        "cycle-mmq-u-spellshaper",
        "A Spellshaper: pay a cost, tap, and discard a card to fire a one-shot ability.",
    ),
    (
        "cycle-mh3-saga",
        "A Saga that gains lore counters each turn, unlocking a new chapter effect at each stage.",
    ),
    (
        "cycle-mir-monocolor-enemy-hate",
        "A card that punishes or destroys cards of a single chosen enemy color.",
    ),
    (
        "cycle-mmq-enemy-hate",
        "An enchantment that taxes or punishes two specific enemy colors.",
    ),
    (
        "cycle-mmq-storage-land",
        "A land that enters tapped and stockpiles storage counters you can cash in later for a burst of mana.",
    ),
    (
        "cycle-mh1-talisman",
        "A two-color mana rock that taps for colorless freely or for either of its colors at the cost of 1 life.",
    ),
    (
        "cycle-mh3-r-mono-land",
        "A land that enters tapped unless you control a matching basic land type, taps for one mana, and has a bonus activated ability.",
    ),
    (
        "cycle-mir-dragon",
        "A flying Dragon with its own extra ability, from the Mirage cycle of five colored Dragons.",
    ),
    (
        "cycle-mir-instantment",
        "An Aura you may cast with flash, but if cast at instant speed it's sacrificed at end of turn.",
    ),
    (
        "cycle-mm3-r-tricolor",
        "A three-color rare, one of a five-card multicolor cycle.",
    ),
    (
        "cycle-mmq-depletion-land",
        "A land that enters tapped with two depletion counters, tapping for double colored mana until they run out, then sacrificing itself.",
    ),
    (
        "cycle-mmq-legate",
        "A creature you can cast for free if you control one basic land type and an opponent controls another.",
    ),
    (
        "cycle-mmq-ramosian-recruiter",
        "A Rebel that taps to search your library for a cheaper Rebel permanent and puts it onto the battlefield.",
    ),
    (
        "cycle-mbs-sun-zenith",
        "An X spell with a scalable effect that shuffles itself back into your library after resolving.",
    ),
    (
        "cycle-mh1-monocolor-legend",
        "A single-color legendary creature that anchors its color's strategy.",
    ),
    (
        "cycle-mh1-u-sliver",
        "A Sliver that grants all your Slivers a shared ability.",
    ),
    (
        "cycle-mh3-flip-walker",
        "A creature that transforms into a planeswalker version of the same character.",
    ),
    (
        "cycle-mid-adversary",
        "A creature you can pay extra for on entry to scale up counters and a bonus effect.",
    ),
    (
        "cycle-mir-diamond",
        "An artifact that enters tapped and taps for one mana of a single color.",
    ),
    (
        "cycle-mir-enemy-forward-hate",
        "Deals extra damage or a harsher effect against cards of one specific color.",
    ),
    (
        "cycle-mir-guildmage",
        "A creature with two activated abilities, each costing one mana and tapping.",
    ),
    (
        "cycle-mkm-r-case",
        "An enchantment that gains a bonus effect once you meet its solve condition.",
    ),
    (
        "cycle-mm2-enemy-hate",
        "An effect that only hits spells or permanents of two specific colors.",
    ),
    (
        "cycle-mmq-ability-wall",
        "A defender creature that can't attack but has an activated ability.",
    ),
    (
        "cycle-mmq-c-spellshaper",
        "A creature that taps and discards a card to use a spell-like ability.",
    ),
    (
        "cycle-m20-iconic-legend",
        "A powerful mono-color legendary creature, one per color.",
    ),
    (
        "cycle-mmq-flash-aura",
        "An Aura you can cast at instant speed to enchant a creature.",
    ),
    (
        "cycle-m20-same-name-enhanced",
        "A card that grows stronger the more cards sharing its own name you control or bury.",
    ),
    (
        "cycle-mmq-ramos-artifact",
        "A mana rock that taps for one color, or can be sacrificed for that same color.",
    ),
    (
        "cycle-m21-precon-tutor",
        "Searches your library and/or graveyard for one specific named planeswalker into your hand.",
    ),
    (
        "cycle-mbs-color-artifact",
        "A colorless artifact creature with an activated ability keyed to a single color of mana.",
    ),
    (
        "cycle-mh1-force",
        "Lets you exile a card of its color from your hand to cast it free when it isn't your turn.",
    ),
    (
        "cycle-m12-c-pw-signature",
        "A common flavored after a planeswalker, echoing that walker's signature effect.",
    ),
    (
        "cycle-m13-legend-spell",
        "A card whose effect mirrors the signature ability of a same-set legendary creature.",
    ),
    (
        "cycle-m13-sedge-creature",
        "Gets +1/+1 while you control a matching basic land, plus a color-costed activated ability.",
    ),
    (
        "cycle-mh2-basic-landcycler",
        "A card with basic landcycling, letting you discard it to search for a basic land.",
    ),
    (
        "cycle-mh3-flare",
        "A spell you may cast by sacrificing a matching-colored nontoken creature instead of paying its mana cost.",
    ),
    (
        "cycle-mic-visions",
        "A sorcery with flashback that costs less based on your commander's mana value, letting you recast it from the graveyard.",
    ),
    (
        "cycle-m14-planeswalker",
        "A mono-colored planeswalker with three loyalty abilities built around its color's theme.",
    ),
    (
        "cycle-mid-alt-transform",
        "A double-faced creature that transforms into a different card, changing its abilities or even its card type.",
    ),
    (
        "cycle-mir-charm",
        "An instant that lets you choose one of three small modal effects.",
    ),
    (
        "cycle-lrw-vivid-land",
        "A land that enters tapped with charge counters, tapping for its color or, by removing a counter, any color.",
    ),
    (
        "cycle-mir-enemy-backward-hate",
        "A card that specifically punishes spells, creatures, or hands tied to one enemy color.",
    ),
    (
        "cycle-m15-paragon",
        "A creature that boosts other same-colored creatures and can tap to grant one a keyword ability.",
    ),
    (
        "cycle-mir-fetchland",
        "A land that enters tapped and can be sacrificed to search for one of two basic land types.",
    ),
    (
        "cycle-m15-wall",
        "A Wall with defender that also carries its own extra ability.",
    ),
    (
        "cycle-mir-x-allied-spell",
        "A two-color spell whose cost includes X, scaling its effect by that amount.",
    ),
    (
        "cycle-m19-precon-planeswalker",
        "A planeswalker included in one of Core Set 2019's preconstructed planeswalker decks.",
    ),
    (
        "cycle-mkm-split",
        "A split card offering two related spells on one card, letting you cast either half.",
    ),
    (
        "cycle-mm3-u-tricolor",
        "A three-color creature with strong stats and a simple keyword or ability.",
    ),
    (
        "cycle-lgn-invoker",
        "A creature with a costly activated ability you can use repeatedly for a big effect.",
    ),
    (
        "cycle-ltr-u-saga",
        "A Saga enchantment that delivers a different effect as its lore counters increase each turn.",
    ),
    (
        "cycle-mmq-alpha-spellshaper",
        "A creature that taps and discards a card to produce a spell-like effect.",
    ),
    (
        "cycle-m20-doubles",
        "A card built around doing something twice, like searching, copying, or returning two things.",
    ),
    (
        "cycle-m11-leyline",
        "An enchantment you can put onto the battlefield for free from your opening hand.",
    ),
    (
        "cycle-m10-typal-lord",
        "A creature that buffs all other creatures you control of its own tribe, usually with +1/+1.",
    ),
    (
        "cycle-m20-wedge-legend",
        "A legendary three-color creature built around a signature synergy for its colors.",
    ),
    (
        "cycle-m20-precon-u",
        "A common or uncommon creature or spell with a small self-contained payoff.",
    ),
    (
        "cycle-m20-precon-tutor",
        "Searches your library and graveyard for one specific named card and puts it into your hand.",
    ),
    (
        "cycle-m20-protection-creature",
        "A creature with protection from a single color.",
    ),
    (
        "cycle-mmq-r-spellshaper",
        "A creature that discards a card and taps to trigger a one-shot effect, usable again each turn.",
    ),
    (
        "cycle-m13-shandalar-ring",
        "An Equipment that grants a keyword ability and puts a +1/+1 counter on the creature each upkeep if it's that ring's color.",
    ),
    (
        "cycle-m21-precon-planeswalker",
        "A planeswalker printed in a Core Set 2021 preconstructed deck.",
    ),
    (
        "cycle-m21-precon-u",
        "A creature from a Core Set 2021 precon deck whose ability synergizes with that deck's signature planeswalker.",
    ),
    (
        "cycle-mb2-dual-land",
        "A two-color dual land that enters tapped and taps for either of its two colors.",
    ),
    (
        "cycle-m11-u-pw-signature",
        "An uncommon Magic 2011 card that synergizes with a specific planeswalker's theme.",
    ),
    (
        "cycle-m12-mage",
        "A Human creature from Magic 2012 with a repeatable activated ability you pay mana to use.",
    ),
    (
        "becomes-changeling",
        "A permanent that can turn into a creature with all creature types.",
    ),
    (
        "cycle-lci-hidden-land",
        "A Cave land from Lost Caverns of Ixalan that enters tapped and can sacrifice itself to Discover 4.",
    ),
    (
        "cycle-lrw-typal-cantrip",
        "A kindred spell that draws you a card if you control a creature of its matching type.",
    ),
    (
        "cycle-lrw-token-fuel",
        "A creature that makes tribal tokens on entry, with a way to sacrifice or tap them for a bonus.",
    ),
    (
        "cycle-m13-legend",
        "A mono-colored legendary creature from the Magic 2013 rare cycle.",
    ),
    (
        "cycle-lci-restless-land",
        "A dual land that enters tapped, taps for two colors, and can become a creature with a combat trigger.",
    ),
    (
        "cycle-m13-pw-signature",
        "A card named after a planeswalker that embodies that planeswalker's signature mechanical theme.",
    ),
    (
        "cycle-lea-lucky-charm",
        "An artifact that lets you pay 1 to gain 1 life whenever a player casts a spell of one color.",
    ),
    (
        "cycle-m13-pw-hallmark",
        "A card that reflects a planeswalker's iconic mechanic, like mill or lifegain, without naming them.",
    ),
    (
        "cycle-mh2-incarnation",
        "An Elemental Incarnation with a strong enters-the-battlefield effect and an evoke cost of exiling a card of its color.",
    ),
    (
        "cycle-m14-pw-signature",
        "A creature whose ability echoes a specific planeswalker's signature effect.",
    ),
    (
        "cycle-mic-curse",
        "An Aura Curse that enchants a player and imposes an ongoing effect tied to them.",
    ),
    (
        "cycle-m14-iconic-creature",
        "A flagship rare or mythic creature that headlines its color in the set.",
    ),
    (
        "cycle-m14-magus-staff",
        "An artifact that gains you 1 life whenever you cast a spell or a land enters of one color.",
    ),
    (
        "cycle-lea-ward",
        "An Aura that grants the enchanted creature protection from one color.",
    ),
    (
        "cycle-leg-banding-land",
        "A land that gives your legendary creatures of one color banding with other legendary creatures.",
    ),
    (
        "cycle-mid-c-typal",
        "A spell or creature whose effect improves when a specific creature type is involved.",
    ),
    (
        "cycle-mid-slowland",
        "A dual land that enters tapped unless you already control two or more other lands.",
    ),
    (
        "cycle-leg-legendary-land",
        "A legendary land that taps for mana and also has a unique utility ability.",
    ),
    (
        "cycle-leg-elder-dragon",
        "A flying Elder Dragon you must sacrifice each upkeep unless you pay a three-color cost.",
    ),
    (
        "cycle-leg-color-wash-instant",
        "An instant that changes one or more target creatures to a single color until end of turn.",
    ),
    (
        "cycle-m15-planeswalker",
        "A legendary planeswalker with plus, minus, and ultimate loyalty abilities.",
    ),
    (
        "cycle-akh-cartouche",
        "An aura that grants a keyword and +1/+1 while triggering a one-time effect when it enters.",
    ),
    (
        "cycle-akh-god",
        "An indestructible legendary God that can't attack or block until a specific condition is met.",
    ),
    (
        "cycle-leg-glyph",
        "An instant that affects a Wall in combat, often the creatures it blocked this turn.",
    ),
    (
        "cycle-mir-enemy-fw-protection",
        "A creature with protection from, or that can't be targeted by, an enemy color.",
    ),
    (
        "cycle-m10-enemy-hate",
        "A card that punishes, destroys, or counters spells and permanents of two enemy colors.",
    ),
    (
        "cycle-m15-sliver",
        "A Sliver whose ability affects every Sliver you control.",
    ),
    (
        "cycle-m20-color-artifact",
        "An artifact, often an Equipment, that creates a colored creature token.",
    ),
    (
        "cycle-m19-elder-dragon",
        "A three-color legendary Elder Dragon from Core Set 2019 with a powerful unique ability.",
    ),
    (
        "cycle-m19-dig-spell",
        "Digs five cards deep for a card of one color while also doing something else, like removal or a buff.",
    ),
    (
        "cycle-m19-precon-c",
        "A simple common creature or artifact from the M19 planeswalker precon decks.",
    ),
    (
        "cycle-m19-planeswalker",
        "One of the five signature planeswalkers from Core Set 2019, each with a build-around theme and an ultimate.",
    ),
    (
        "cycle-m20-cavalier",
        "A five-mana Elemental Knight from M20 with a strong enter-the-battlefield effect and a payoff when it dies.",
    ),
    (
        "cycle-ala-c-tricolor",
        "A three-color common creature from Alara block, most with simple keyword abilities.",
    ),
    (
        "cycle-ala-c-two-color",
        "A two-color common spell from Alara block that combines two smaller effects, one per color.",
    ),
    (
        "cycle-ala-charm",
        "A modal instant that lets you choose one of three effects.",
    ),
    (
        "conjure-to-exile",
        "Conjures cards directly into exile, often to play or cast for a limited time.",
    ),
    (
        "cycle-ala-forward-ability",
        "A creature with an activated ability, usually to pump or grant itself a keyword.",
    ),
    (
        "cycle-ltr-r-saga",
        "A Saga that resolves three chapter effects on successive turns.",
    ),
    (
        "cycle-m19-typal-lord",
        "A creature that gives other creatures of its type +1/+1.",
    ),
    (
        "cycle-lgn-u-sliver",
        "A Sliver that grants an ability or bonus to every Sliver in play, not just yours.",
    ),
    (
        "cost-reducer-aura",
        "Makes the Aura spells you cast cost less to cast.",
    ),
    (
        "cycle-lgn-r-sliver",
        "A Sliver whose powerful ability keys off or applies to every Sliver on the battlefield.",
    ),
    (
        "cycle-m11-c-pw-signature",
        "A common card themed around one of the M11 planeswalkers' signature effects.",
    ),
    (
        "cycle-ala-obelisk",
        "An artifact that taps for any one of three colors in a Shard's mana combination.",
    ),
    (
        "cycle-ala-panorama",
        "A land that taps for colorless or sacrifices to fetch a basic land of a Shard's three colors.",
    ),
    (
        "cycle-lrw-incarnation",
        "An Elemental Incarnation with a powerful ability that shuffles into its owner's library from any graveyard.",
    ),
    (
        "cycle-m20-enemy-hate",
        "A spell with extra power or utility against permanents or spells of two specific enemy colors.",
    ),
    (
        "cycle-m20-planeswalker",
        "One of the five signature Core Set 2020 planeswalkers, each with loyalty abilities tied to its color's themes.",
    ),
    (
        "cycle-m20-leyline",
        "A powerful enchantment you may begin the game with on the battlefield if it's in your opening hand.",
    ),
    (
        "cycle-m20-precon-common",
        "A common creature from an M20 planeswalker deck built to support that deck's theme.",
    ),
    (
        "cycle-m20-precon-planeswalker",
        "A planeswalker built as the centerpiece of an M20 planeswalker deck.",
    ),
    (
        "cycle-lrw-clash-counter-creature",
        "A creature that clashes with an opponent when it enters, gaining a +1/+1 counter if you win.",
    ),
    (
        "cycle-lrw-command",
        "A modal spell that lets you choose two of four effects to resolve.",
    ),
    (
        "cycle-mmq-monger",
        "A creature with a repeatable activated ability that any player, not just its controller, can use.",
    ),
    (
        "cycle-mmq-pitchspell",
        "A spell you can cast for free by exiling a card of the matching color from your hand instead of paying its cost.",
    ),
    (
        "balance",
        "Forces each player to sacrifice lands, creatures, and discard down to the lowest count among all players.",
    ),
    (
        "cycle-m12-planeswalker",
        "One of the five planeswalkers from Magic 2012, each with its own loyalty abilities.",
    ),
    (
        "cycle-m15-sedge-creature",
        "A creature that gets +1/+1 while you control a matching basic land and has a mana sink ability.",
    ),
    (
        "cycle-m21-precon-c",
        "A common creature from one of the Magic 2021 planeswalker preconstructed decks.",
    ),
    (
        "activate-from-exile",
        "Has an ability you can activate while the card sits in exile.",
    ),
    (
        "cycle-lrw-typal-dual-land",
        "A dual land that enters tapped unless you reveal a card of its creature type from hand.",
    ),
    (
        "cycle-lci-c-craft",
        "A common double-faced artifact you can craft into its back face by exiling other cards as payment.",
    ),
    (
        "cycle-m21-r-typal",
        "A rare creature whose ability cares about others of its creature type.",
    ),
    (
        "cycle-lci-landcycler",
        "A Dinosaur you can discard to search your library for a basic land.",
    ),
    (
        "cycle-lgn-gempalm",
        "A creature that can be cycled for a card and triggers a tribal boost or damage effect when cycled.",
    ),
    (
        "cycle-lrw-lorwyn-five",
        "One of the five original planeswalkers introduced in Lorwyn.",
    ),
    (
        "cycle-m11-typal-lord",
        "A creature that pumps other creatures of its type by +1/+1.",
    ),
    (
        "cycle-c14-planeswalker",
        "A planeswalker from Commander 2014 that can be your commander.",
    ),
    (
        "cycle-c15-alt-commander",
        "A legendary creature from Commander 2015 designed as an alternate commander option.",
    ),
    (
        "cycle-c15-commander-reference",
        "Named after and themed around a specific Commander 2015 legendary creature, without being that commander.",
    ),
    (
        "cycle-c15-confluence",
        "Lets you choose three effects from a list, choosing the same one more than once if you want.",
    ),
    (
        "cycle-c15-experience-commander",
        "A commander that gives you experience counters and grows stronger the more you collect.",
    ),
    (
        "cycle-c15-myriad-creature",
        "Has myriad, letting it create tapped, attacking token copies against your other opponents.",
    ),
    (
        "cycle-c16-basic-landcycling",
        "Can be discarded to search your library for a basic land, on top of its normal effect.",
    ),
    (
        "cycle-lea-boon",
        "A simple, efficient one-mana effect from Magic's original Alpha cycle, one per color.",
    ),
    (
        "cycle-m12-r-pw-signature",
        "Named after and thematically tied to a planeswalker from Magic 2012.",
    ),
    (
        "cycle-lea-moxen",
        "A 0-cost artifact that taps for one mana of a single color, from the original Moxen cycle.",
    ),
    (
        "cycle-c16-u-monocolor",
        "One of five unrelated monocolored uncommons from the Commander 2016 cycle.",
    ),
    (
        "cycle-c16-undaunted-spell",
        "Has undaunted, costing 1 less to cast for each opponent you have.",
    ),
    (
        "cycle-c17-curse",
        "An Aura Curse that rewards you and each attacker whenever the enchanted player is attacked.",
    ),
    (
        "cycle-m13-planeswalker",
        "A planeswalker from the Magic 2013 cycle, each with three loyalty abilities.",
    ),
    (
        "cycle-c17-kindred-spell",
        "Lets you choose a creature type, then gives an effect tied to creatures of that type.",
    ),
    (
        "cycle-c18-commander-storm",
        "Copies itself once for each time you've cast your commander from the command zone this game.",
    ),
    (
        "cycle-c18-lieutenant",
        "Triggers a bonus effect at the start of combat while you control your commander.",
    ),
    (
        "cycle-lea-lace",
        "Changes the color of a target spell or permanent.",
    ),
    (
        "cycle-lea-circle-protection",
        "Pays mana to prevent the next damage a source of one color would deal to you this turn.",
    ),
    (
        "cycle-mh2-converge",
        "Gets bigger or better the more different colors of mana you spent to cast it.",
    ),
    (
        "cycle-leg-battery",
        "Stores charge counters over time, then taps to release them as a burst of one color of mana.",
    ),
    (
        "cycle-lrw-u-changeling",
        "A creature that counts as every creature type at once.",
    ),
    (
        "cycle-m14-r-enemy-hate",
        "A creature with a built-in edge or punisher effect against two specific opposing colors.",
    ),
    (
        "cycle-m15-same-color-enhancer",
        "A spell that rewards you for a color you control or only affects off-color creatures.",
    ),
    (
        "cycle-ltc-alt-commander",
        "A legendary creature offered as an alternate commander for a Lord of the Rings deck.",
    ),
    (
        "cycle-c20-bonder-partner",
        "The support half of a Partner With pair that generates value alongside its partner.",
    ),
    (
        "cycle-c20-face-commander",
        "A standalone legendary creature built as a commander around its own theme.",
    ),
    (
        "cycle-c20-free-spell",
        "Lets you cast it without paying its mana cost as long as you control a commander.",
    ),
    (
        "cycle-c20-impetus",
        "An aura that pumps the enchanted creature and goads it to attack another player each combat.",
    ),
    (
        "cycle-c20-monster-partner",
        "The aggressive half of a Partner With pair that grows into a threat through counters.",
    ),
    (
        "cycle-c20-planeswalker",
        "A legendary planeswalker from the Commander 2020 cycle.",
    ),
    (
        "cycle-c21-alt-commander",
        "A legendary creature from Strixhaven's alternate-commander cycle, another commander for its college's colors.",
    ),
    (
        "cycle-c21-college-spell",
        "A powerful instant or sorcery from the Strixhaven Commander cycle, one per college.",
    ),
    (
        "cycle-leg-anti-landwalk-enchant",
        "An enchantment that lets creatures with a specific landwalk type be blocked as though they lacked it.",
    ),
    (
        "cycle-c21-technique",
        "A sorcery with demonstrate: you copy it on cast and an opponent also copies it.",
    ),
    (
        "cycle-mid-r-mono-werewolf",
        "A mono-colored rare Human Werewolf double-faced card from the Midnight Hunt cycle.",
    ),
    (
        "cycle-chk-deceiver",
        "A Spirit that peeks at your library and, if the top card is a land, gains a combat boost for the turn.",
    ),
    (
        "cycle-chk-dragon",
        "A Dragon Spirit that flies and triggers a powerful effect when it dies.",
    ),
    (
        "cycle-chk-flash-aura",
        "An Aura with flash you can cast at instant speed to buff or hinder a creature.",
    ),
    (
        "cycle-chk-honden",
        "A Shrine that gives a bigger effect each upkeep for every Shrine you control.",
    ),
    (
        "cycle-chk-legendary-land",
        "A land that taps for one color and activates to aid a legendary creature.",
    ),
    (
        "cycle-chk-myojin",
        "A Spirit that enters indestructible and removes its counter for a huge one-time effect.",
    ),
    (
        "cycle-chk-napland",
        "A land that taps for colorless, or for one of two colors if it skips its next untap.",
    ),
    (
        "cycle-chk-r-flip",
        "A creature that flips into a stronger legendary version when a condition is met.",
    ),
    (
        "cycle-chk-u-flip",
        "A creature that flips into a more powerful legendary version once its flip condition is met.",
    ),
    (
        "cycle-chk-zubera",
        "A Zubera whose death trigger scales with the number of Zubera that died this turn.",
    ),
    (
        "cycle-clb-adventurer",
        "A creature that gives you the initiative on entry and grows stronger once you've completed a dungeon.",
    ),
    (
        "cycle-m14-r-sliver",
        "A Sliver that grants all your Slivers a shared keyword or stat boost.",
    ),
    (
        "cycle-clb-ancient-dragon",
        "A flying dragon that rolls a d20 when it hits a player, scaling a bonus by the result.",
    ),
    (
        "cycle-akh-dual-cycling-land",
        "A dual-type land that enters tapped and can be cycled to draw a card.",
    ),
    (
        "cycle-clb-backward-ally-legend",
        "A legendary creature built to pair with a Background as your second commander.",
    ),
    (
        "cycle-akh-enemy-aftermath",
        "A split card whose second half has aftermath, letting you cast it from your graveyard later.",
    ),
    (
        "cycle-clb-c-d20",
        "Has you roll a d20, with a bigger effect the higher you roll.",
    ),
    (
        "cycle-clb-dethrone-background",
        "A Background that gives your commander creatures a bonus when they attack the player with the most life.",
    ),
    (
        "cycle-clb-monument",
        "No cards carry this tag in the catalog, so no functional description can be authored.",
    ),
    (
        "cycle-clb-forward-ally-legend",
        "A legendary creature commander that can pair with a Background as a second commander.",
    ),
    (
        "cycle-ltr-legendary-land",
        "A legendary land that enters tapped unless you control a legendary creature, then taps for mana and a bonus ability.",
    ),
    (
        "cycle-akh-monument",
        "A legendary artifact that discounts creature spells of one color and triggers an effect whenever you cast a creature.",
    ),
    (
        "cycle-m11-titan",
        "A big creature that triggers a powerful effect whenever it enters or attacks.",
    ),
    (
        "cycle-akh-trial",
        "An enchantment that triggers a powerful effect when it enters, then returns to your hand when a Cartouche enters.",
    ),
    (
        "cycle-m19-precon-pw-enhanced",
        "A creature that gains an extra ability or bonus as long as you control a specific planeswalker.",
    ),
    (
        "cycle-ala-4mnno",
        "A big, powerful multicolor creature with a game-swinging ability.",
    ),
    (
        "cycle-lgn-muse",
        "A Spirit creature with a strong recurring ability tied to upkeep, hand size, or untapping.",
    ),
    (
        "cycle-ala-allied-2-drop",
        "A two-mana, allied two-color creature with keyword abilities.",
    ),
    (
        "cycle-ltr-landcycler",
        "A creature or spell you can discard to search your library for a land of a specific type.",
    ),
    (
        "cycle-ala-backward-ability",
        "A creature with an activated ability costing mana in a color it doesn't normally use.",
    ),
    (
        "cycle-lrw-legend",
        "Tags the Lorwyn cycle of monocolored legendary creatures, one per color.",
    ),
    (
        "cycle-ala-battlemage",
        "Tags the Alara battlemage cycle: two-color creatures with a tap ability in each color.",
    ),
    (
        "cycle-m19-monocolor-legend",
        "Tags the Core Set 2019 cycle of monocolored legendary creatures, one per color.",
    ),
    (
        "cycle-ala-c-1-drop",
        "Tags the Alara cycle of common one-mana creatures, one per color.",
    ),
    (
        "cycle-m10-checkland",
        "Tags checklands: dual lands that enter untapped only if you control a matching land type.",
    ),
    (
        "cycle-dmr-r-two-color",
        "Tags the Dominaria Remastered cycle of two-color rares, one per allied color pair.",
    ),
    (
        "cycle-dmr-tricolor-legend",
        "Tags the Dominaria Remastered cycle of three-color legendary creatures, one per shard.",
    ),
    (
        "cycle-dmu-c-back-ally-kicker",
        "Has kicker: pay extra as you cast for an added bonus effect for you.",
    ),
    (
        "cycle-dmu-c-back-en-kicker",
        "Has kicker: pay extra for an added effect, often an enters-the-battlefield boost.",
    ),
    (
        "cycle-dmu-c-for-ally-kicker",
        "Has kicker: pay extra when casting to add a bonus for you on top of the base effect.",
    ),
    (
        "cycle-dmu-c-for-en-kicker",
        "Has kicker: pay extra for an added effect, often an enters-the-battlefield bonus.",
    ),
    (
        "cycle-dmu-cost-reduction",
        "A creature that costs less to cast based on things like creatures, graveyard cards, or land types you control.",
    ),
    (
        "cycle-dmu-defiler",
        "Lets you pay 2 life for a mana discount on permanent spells of its color, rewarding each cast.",
    ),
    (
        "cycle-dmu-lord",
        "Boosts other creatures of its type by +1/+1 and has an extra ability tied to that creature type.",
    ),
    (
        "cycle-dmu-r-m-saga",
        "A Saga with read ahead, letting you start on a later chapter and skip its earlier effects.",
    ),
    (
        "cycle-dmu-tricolor-legend",
        "A three-color legendary creature from Dominaria United.",
    ),
    (
        "cycle-ala-herald",
        "Sacrifices three creatures of specific colors to put its matching big creature onto the battlefield.",
    ),
    (
        "cycle-m19-precon-r",
        "A rare creature that headlines one of the set's planeswalker decks.",
    ),
    (
        "cycle-m19-signature-spell",
        "A spell tied to one of the set's signature planeswalkers.",
    ),
    (
        "cycle-ala-legend",
        "A three-color legendary creature representing one Alara shard's signature strategy.",
    ),
    (
        "cycle-dst-affinity-golem",
        "An artifact Golem that costs 1 less to cast for each matching land type you control.",
    ),
    (
        "cycle-dst-pulse",
        "An instant or sorcery with an effect that returns to your hand if an opponent is still ahead of you.",
    ),
    (
        "cycle-dtk-behold-dragon",
        "Gains a bonus effect if you reveal a Dragon from hand or control one as you cast it.",
    ),
    (
        "cycle-dtk-monument",
        "A mana rock that can pay extra mana to become a 4/4 flying Dragon creature until end of turn.",
    ),
    (
        "cycle-ecl-champion",
        "A creature that exiles another of its type as an added cost, returning it when this creature leaves.",
    ),
    (
        "cycle-ecl-command",
        "A Kindred spell that picks two of four modes, one of which copies a creature of its type.",
    ),
    (
        "cycle-ecl-eclipsed",
        "When this enters, look at your top four cards and may take a matching creature-type or land card.",
    ),
    (
        "cycle-ecl-hybrid-signpost",
        "A creature built around a two-color archetype, often costed or powered with hybrid mana.",
    ),
    (
        "cycle-ecl-incarnation",
        "A hybrid creature with an enter effect for each color you paid double, plus evoke.",
    ),
    (
        "cycle-ecl-student",
        "A legendary student creature with its own unique ability.",
    ),
    (
        "cycle-ecl-typal-convoke",
        "A convoke spell that has you choose a creature type, then keys its effect off it.",
    ),
    (
        "cycle-ecl-typal-kindred",
        "A Kindred card built around a single creature type, making and caring about that type.",
    ),
    (
        "cycle-ala-r-tricolor",
        "One of Alara Reborn's rare, signature three-color cards representing a single shard.",
    ),
    (
        "cycle-ala-resounding-spell",
        "A spell with a bigger bonus effect when you cycle it instead of casting it normally.",
    ),
    (
        "cycle-emn-draft-signpost",
        "A common or uncommon built to point drafters toward a specific two-color archetype.",
    ),
    (
        "cycle-lrw-hideaway-land",
        "A land with hideaway that lets you play the exiled card for free once you meet its condition.",
    ),
    (
        "cycle-lrw-c-changeling",
        "A changeling shapeshifter that counts as every creature type.",
    ),
    (
        "cycle-m11-enemy-hate",
        "Removal or protection aimed specifically at your two enemy colors.",
    ),
    (
        "cycle-eve-hedge-mage",
        "A creature whose enters triggers vary based on which basic land types you control.",
    ),
    (
        "cycle-eve-hybrid-modal",
        "A hybrid spell whose effect changes based on which color of mana you spend to cast it.",
    ),
    (
        "cycle-eve-mimic",
        "A shapeshifter that gains a keyword and boosted stats when you cast a spell in both its colors.",
    ),
    (
        "cycle-eve-monocolor-hybrid",
        "A creature with a hybrid mana ability that can be activated with either of two colors.",
    ),
    (
        "cycle-eve-r-chroma",
        "Scales an effect by the number of one color's mana symbols among relevant cards.",
    ),
    (
        "cycle-eve-skulkin",
        "Pays mana to grant a target creature of a specific color a keyword or bonus until end of turn.",
    ),
    (
        "cycle-eve-u-hybrid-3-drop",
        "A three-mana hybrid creature with a keyword or hybrid-cost ability tied to its color pair.",
    ),
    (
        "cycle-eve-untapper",
        "An ability creature that untaps itself whenever you cast a spell of its color.",
    ),
    (
        "cycle-exo-keeper",
        "Pays mana and taps to target an opponent ahead of you in some resource, then gains you an edge.",
    ),
    (
        "cycle-exo-oath",
        "Each upkeep lets a player target an opponent ahead of them in some way to trigger an optional bonus.",
    ),
    (
        "cycle-exo-retriever",
        "Returns a target card of a specific type from your graveyard to your hand when it enters.",
    ),
    (
        "cycle-extraplanar-praetor",
        "A legendary Phyrexian Praetor whose signature ability rewards you and punishes opponents.",
    ),
    (
        "cycle-fdn-planeswalker",
        "A legendary planeswalker with loyalty abilities built around its color's core strategies.",
    ),
    (
        "cycle-fem-artifact-boon",
        "A cheap artifact you tap and sacrifice for a one-time effect like damage, life, mana, or a card.",
    ),
    (
        "cycle-fem-sacland",
        "A land that enters tapped and can be sacrificed for two mana of its color instead of tapping for one.",
    ),
    (
        "cycle-frf-hybrid-ability",
        "A creature with an activated ability whose cost includes hybrid mana.",
    ),
    (
        "cycle-lrw-r-changeling",
        "A rare Shapeshifter with changeling, counting as every creature type, plus its own extra ability.",
    ),
    (
        "cycle-ltc-face-commander",
        "A legendary creature designed as the face commander of a Lord of the Rings Commander deck.",
    ),
    (
        "cycle-lci-r-land-dfc",
        "A legendary double-faced permanent whose back face is a land.",
    ),
    (
        "cycle-m21-mono-teferi-legend",
        "One of a cycle of mono-colored legendary creatures, each built around its color's signature strategy.",
    ),
    (
        "cycle-gpt-magemark",
        "An Aura that buffs every creature you control that's also enchanted, not just the one it's on.",
    ),
    (
        "cycle-gpt-nephilim",
        "A four-color Nephilim creature with a splashy triggered ability.",
    ),
    (
        "cycle-gpt-rusalka",
        "Pay mana and sacrifice a creature to produce a small colored effect.",
    ),
    (
        "cycle-grn-c-guild-ability",
        "A common card built around one Ravnica guild's signature keyword mechanic.",
    ),
    (
        "cycle-grn-guild-champion",
        "A legendary creature that represents one Ravnica guild.",
    ),
    (
        "cycle-grn-guildmage",
        "A creature with two activated abilities, one for each of its guild's colors.",
    ),
    (
        "cycle-grn-guildmaster",
        "A legendary creature or planeswalker that leads one of Ravnica's guilds.",
    ),
    (
        "cycle-grn-hybrid-creature",
        "A common creature with a hybrid mana cost, castable with either of its guild's two colors.",
    ),
    (
        "cycle-grn-locket",
        "An artifact that taps for either of its guild's colors and can be sacrificed to draw two cards.",
    ),
    (
        "cycle-grn-m-guild-spell",
        "A splashy, high-impact multicolor mythic card for its guild.",
    ),
    (
        "cycle-grn-mmnn",
        "An uncommon two-color creature representing its guild.",
    ),
    (
        "cycle-lci-god",
        "A modal double-faced card with a legendary God on one face and a land on the other.",
    ),
    (
        "cycle-grn-r-monocolor-care",
        "A card that rewards you for casting or controlling permanents of one specific color.",
    ),
    (
        "cycle-grn-r-split",
        "A rare split card from Guilds of Ravnica, letting you cast either of two spells.",
    ),
    (
        "cycle-grn-u-split",
        "An uncommon split card from Guilds of Ravnica, letting you cast either of two spells.",
    ),
    (
        "cycle-gtc-denizen",
        "A creature that triggers whenever another creature of its color enters under your control.",
    ),
    (
        "cycle-gtc-land-aura",
        "An Aura that enchants a land and grants it an extra ability or effect.",
    ),
    (
        "cycle-gtc-m-monocolor",
        "A powerful mono-colored mythic rare built around a splashy, game-swinging effect.",
    ),
    (
        "cycle-gtc-primordial",
        "A creature whose enters-the-battlefield trigger steals, destroys, or exiles something from each opponent.",
    ),
    (
        "cycle-gtc-x-spell",
        "An X spell whose size or effect scales with the amount of mana spent on X.",
    ),
    (
        "cycle-hbg-gate",
        "A Gate that enters tapped, taps for one color, and can pay to seek a nonland card once.",
    ),
    (
        "cycle-hbg-legend",
        "A legendary creature with its own unique, often graveyard-recursive or card-advantage ability.",
    ),
    (
        "cycle-hml-triland",
        "A land that taps for colorless free, one color for {1}, or either of two colors for {2}.",
    ),
    (
        "cycle-hou-allied-aftermath",
        "A split card in allied colors that can be cast again from your graveyard via aftermath.",
    ),
    (
        "cycle-hou-defeat",
        "An instant or sorcery that answers a specific color, with a bonus if it hits that color's planeswalker.",
    ),
    (
        "cycle-hou-desert-cycling-land",
        "A Desert that enters tapped, taps for one color, and can cycle away for a card when unneeded.",
    ),
    (
        "cycle-hou-desert-painland",
        "A Desert land that taps for colorless free or a color for 1 life, and can sacrifice a Desert for a bonus effect.",
    ),
    (
        "cycle-hou-deserts-matter",
        "Gets a bonus if you control a Desert or have one in your graveyard.",
    ),
    (
        "cycle-hou-draft-signpost",
        "A two-color card built to point drafters toward that color pair's archetype.",
    ),
    (
        "cycle-hou-enemy-aftermath",
        "An Aftermath split card in enemy colors whose second half is cast from the graveyard.",
    ),
    (
        "cycle-c15-reprint-commander",
        "A multicolor legendary creature reprinted in a Commander 2015 deck.",
    ),
    (
        "cycle-hou-gods-last-act",
        "A powerful sorcery that keeps your lands from untapping next turn as its cost.",
    ),
    (
        "cycle-hou-hour",
        "A splashy 'Hour of' spell from the Hour of Devastation set.",
    ),
    (
        "cycle-ice-scarab",
        "An aura that stops one color from blocking the creature and grants +2/+2 while an opponent controls that color.",
    ),
    (
        "cycle-ice-talisman",
        "Lets you pay to untap a permanent whenever any player casts a spell of that color.",
    ),
    (
        "cycle-ice-two-color-enemy-hate",
        "An enchantment that hinders a specific enemy color, directly or indirectly.",
    ),
    (
        "cycle-iko-apex",
        "A legendary mutate creature with a powerful trigger whenever it mutates.",
    ),
    (
        "cycle-iko-apex-spell",
        "An instant or sorcery with a strong one-shot effect, sometimes with cycling.",
    ),
    (
        "cycle-iko-c-keyword-boost",
        "A common spell that costs less or does more if you control a creature with a certain keyword.",
    ),
    (
        "cycle-iko-c-mutate",
        "A common creature with mutate and a bonus effect whenever it mutates.",
    ),
    (
        "cycle-iko-counter-cycler",
        "A cycling creature that, when cycled, puts a counter of its own keyword on a creature you control.",
    ),
    (
        "cycle-iko-crystal",
        "A cycling artifact that taps for one of three colors in its wedge.",
    ),
    (
        "cycle-iko-keyword-bonder",
        "A Human with a keyword that boosts your other creatures or spells sharing that keyword.",
    ),
    (
        "cycle-iko-keyword-mentor",
        "A Human that gives a non-Human creature a keyword counter on entry, then pumps every creature you control with that keyword.",
    ),
    (
        "cycle-iko-keyword-monster",
        "A creature that shares a keyword with your team and triggers or activates a payoff for every creature you control that has it.",
    ),
    (
        "cycle-iko-legendary-human",
        "A legendary Human creature from Ikoria.",
    ),
    (
        "cycle-iko-modal-creature",
        "A creature that enters with your choice of one of two keyword counters.",
    ),
    (
        "cycle-iko-mutate-hybrid",
        "A mutate creature with a hybrid-mana mutate cost and a bonus effect when it mutates.",
    ),
    (
        "cycle-iko-mutate-x",
        "A mutate creature whose effect scales with X, the number of times it has mutated.",
    ),
    (
        "cycle-iko-mythos",
        "A spell with a bonus effect if two specific colors of mana were spent to cast it.",
    ),
    (
        "cycle-iko-r-mutate",
        "A mutate creature with an evergreen keyword and a removal or value effect when it mutates.",
    ),
    (
        "cycle-iko-signpost-creature",
        "A creature that showcases a two-color archetype from Ikoria.",
    ),
    (
        "cycle-iko-signpost-noncreature",
        "A noncreature spell that showcases a two-color archetype from Ikoria.",
    ),
    (
        "cycle-iko-triome",
        "A land that enters tapped, taps for one of three colors, and can be cycled for a card.",
    ),
    (
        "cycle-inv-djinn",
        "A creature with a keyword ability that gets -2/-2 while a certain color is most common among all permanents.",
    ),
    (
        "cycle-inv-domain-spell",
        "Scales its effect by the number of basic land types among lands you control.",
    ),
    (
        "cycle-inv-dragon-attendant",
        "An artifact creature you sacrifice to add three specific colors of mana.",
    ),
    (
        "cycle-inv-dual-tapland",
        "A tapland that produces either of two colors of mana.",
    ),
    (
        "cycle-inv-emissary",
        "A creature that destroys or bounces a permanent when it enters, but only if you paid its kicker cost.",
    ),
    (
        "cycle-inv-flash-sorcery",
        "A sorcery you can cast at instant speed if you pay an extra cost.",
    ),
    (
        "cycle-inv-forward-ability",
        "A creature that pays one mana to give itself a keyword or buff until end of turn.",
    ),
    (
        "cycle-inv-split-card",
        "A split card offering two spells on one card; cast either half.",
    ),
    (
        "cycle-inv-weaver",
        "A creature that can pump or grant a keyword to a creature of one of two colors.",
    ),
    (
        "cycle-isd-allied-flashback",
        "An allied two-color spell you can recast from your graveyard for its flashback cost.",
    ),
    (
        "cycle-isd-allied-utilityland",
        "A land that taps for colorless and has an allied two-color activated ability.",
    ),
    (
        "cycle-isd-checkland",
        "A dual land that enters untapped if you control one of its two basic land types.",
    ),
    (
        "cycle-isd-draft-signpost",
        "A flashback spell that points drafters toward a two-color draft archetype.",
    ),
    (
        "cycle-isd-r-flashback",
        "A rare spell with a steep flashback cost letting you recast it from your graveyard.",
    ),
    (
        "cycle-j21-perpetual",
        "Lets you choose a creature card in your hand to perpetually gain an ability or bonus.",
    ),
    (
        "cycle-j21-planeswalker",
        "A legendary planeswalker built on Alchemy digital mechanics like perpetual, conjure, and seek.",
    ),
    (
        "cycle-j22-hybrid-legend",
        "A legendary creature with hybrid mana abilities usable with either of its two colors.",
    ),
    (
        "cycle-j22-m-mono-legend",
        "A mono colored legendary creature.",
    ),
    (
        "cycle-jmp-hybrid-legend",
        "A legendary creature with hybrid mana abilities usable with either of its two colors.",
    ),
    (
        "cycle-jmp-thriving-land",
        "A land that enters tapped and taps for one of two colors, one fixed and one chosen on entry.",
    ),
    (
        "cycle-jou-c-heroic-grower",
        "Puts +1/+1 counters on itself whenever you cast a spell that targets it.",
    ),
    (
        "cycle-tsp-m-suspend-creature",
        "A suspend creature you exile with time counters, then cast free with haste when they run out.",
    ),
    (
        "cycle-unh-u-gotcha",
        "Returns itself from your graveyard to hand when an opponent says one of its trigger words aloud.",
    ),
    (
        "cycle-tmp-c-sliver",
        "A Sliver that grants all Slivers a shared keyword or bonus.",
    ),
    (
        "cycle-tsp-forward-flashback",
        "Can be cast again from your graveyard for its flashback cost, then exiled.",
    ),
    (
        "cycle-uds-lobotomy-spell",
        "Removes a permanent or spell, then exiles every other copy from its controller's deck, hand, and graveyard.",
    ),
    (
        "cycle-unh-minigame",
        "Has you play a real-life minigame against an opponent, with an effect if you win.",
    ),
    (
        "cycle-ths-god-weapon",
        "A legendary artifact that buffs your creatures with a keyword or bonus and has a tap ability.",
    ),
    (
        "cycle-tmc-face-commander",
        "A legendary creature with Partner, Character select, letting it pair with another as co-commanders.",
    ),
    (
        "cycle-tmt-signpost-noncreature",
        "A noncreature spell that signposts a two-color draft archetype.",
    ),
    (
        "cycle-tsp-flash-aura",
        "An Aura with flash, letting you cast it at instant speed.",
    ),
    (
        "cycle-tsp-spellshaper",
        "A creature that discards a card to activate a repeatable ability.",
    ),
    (
        "cycle-uds-growing-enchantment",
        "An Aura that grows stronger each of your upkeeps by adding a counter that fuels its effect.",
    ),
    (
        "cycle-unf-c-blank",
        "A creature that lets you attach a name sticker, gaining a bonus for each unique vowel on it.",
    ),
    (
        "cycle-unh-donkey",
        "A Donkey creature with a quirky, standalone joke ability from an Un-set.",
    ),
    (
        "cycle-thb-r-m-saga",
        "A Saga enchantment that resolves a different effect for each numbered chapter.",
    ),
    (
        "cycle-ths-emissary",
        "A creature with bestow that can enchant a creature, giving it +3/+3 and a bonus ability.",
    ),
    (
        "cycle-tla-basic-count",
        "An effect that scales with how many basic lands of one type you control.",
    ),
    (
        "cycle-tle-bending-master",
        "A legendary creature that gains experience counters and unleashes a bigger effect for each one.",
    ),
    (
        "cycle-tmp-r-two-color",
        "A rare Tempest creature, one of a cycle each with its own quirky ability.",
    ),
    (
        "cycle-tmt-signpost-legend",
        "A legendary creature that anchors and rewards its deck's core tribe or mechanic.",
    ),
    (
        "cycle-tor-threshold-etb",
        "A creature that gains an enters-the-battlefield ability once your graveyard holds seven cards.",
    ),
    (
        "cycle-tsp-allied-sliver",
        "A Sliver that gives all Slivers an extra ability.",
    ),
    (
        "cycle-tsp-magus",
        "A creature that copies the effect of a famous artifact through an activated ability.",
    ),
    (
        "cycle-tsp-r-split-second",
        "A split second spell: while it's on the stack, players can't cast spells or activate abilities.",
    ),
    (
        "cycle-tsp-totem",
        "A mana rock that can pay to turn into a creature until end of turn.",
    ),
    (
        "cycle-tstx-mascot",
        "A basic vanilla or near-vanilla token creature representing its set's flagship creature type.",
    ),
    (
        "cycle-ugl-double-spell",
        "An Unglued spell whose effect repeats in your next game against the same player.",
    ),
    (
        "cycle-und-enemy-colored-legend",
        "An enemy-color legendary creature from Unsanctioned with an activated ability.",
    ),
    (
        "cycle-unf-etb-attraction",
        "Opens an Attraction from your Attraction deck onto the battlefield when it enters.",
    ),
    (
        "cycle-unh-c-gotcha",
        "Returns itself from your graveyard to hand if you catch an opponent doing a specific silly action.",
    ),
    (
        "cycle-spm-saga",
        "A Saga that unfolds a different multi-turn effect on each of its chapters.",
    ),
    (
        "cycle-tdm-khan",
        "A three-color legendary creature that rewards its Tarkir clan's signature playstyle.",
    ),
    (
        "cycle-thb-u-saga",
        "A Saga that unfolds a different multi-turn effect on each of its chapters.",
    ),
    (
        "cycle-ths-double-tactics",
        "A combat trick that affects up to two target creatures at once.",
    ),
    (
        "cycle-stx-a-a-b-b",
        "A two-color creature whose ability blends its two colors' mechanical identities.",
    ),
    (
        "cycle-ths-self-hate",
        "A spell that removes or counters something of its own color.",
    ),
    (
        "cycle-stx-dean",
        "A Strixhaven double-faced legendary creature pairing two of a college's dean characters.",
    ),
    (
        "cycle-tla-utility-land",
        "A land that enters tapped unless you control a basic and has an extra activated ability.",
    ),
    (
        "cycle-tmp-enemy-forward-hate",
        "A card that punishes or shuts down a single color's spells, creatures, or lands.",
    ),
    (
        "cycle-tmp-pain-tapland",
        "A land that enters tapped, taps for colorless free, or two colors for 1 damage to you.",
    ),
    (
        "cycle-tmp-u-sliver",
        "A Sliver that gives all Slivers in play a keyword or activated ability.",
    ),
    (
        "cycle-tmt-r-technique",
        "A spell with sneak: cast it cheaper by returning an unblocked attacker to your hand.",
    ),
    (
        "cycle-tor-c-flashback",
        "A spell you can cast again from your graveyard for its flashback cost and 3 life, then it's exiled.",
    ),
    (
        "cycle-tor-r-dreams",
        "An X spell whose cost is discarding X cards, scaling an effect like damage, search, or bounce.",
    ),
    (
        "cycle-tsp-addendum",
        "Gets a stronger effect if you cast it during your main phase.",
    ),
    (
        "cycle-tdm-c-omen",
        "A dragon whose Omen half you can cast as a spell that shuffles back into your library.",
    ),
    (
        "cycle-tdm-monument",
        "An artifact that fetches a basic land on entry, then taps and sacrifices for a wedge-colored token effect.",
    ),
    (
        "cycle-thb-omen",
        "A flash enchantment with an ETB effect that you can later sacrifice to scry 2.",
    ),
    (
        "cycle-tsp-perpetual-aura",
        "An aura that returns to your hand instead of staying in the graveyard when it leaves the battlefield.",
    ),
    (
        "cycle-tsp-r-sliver",
        "A Sliver that gives all Slivers on the battlefield a shared ability.",
    ),
    (
        "cycle-tdm-u-wedge-dragon",
        "A flying Dragon with an ability themed to one of the three-color wedge clans.",
    ),
    (
        "cycle-tsp-storage-land",
        "A land that taps for colorless mana and can bank storage counters to cash in later for colored mana.",
    ),
    (
        "cycle-tdm-u-twobrid",
        "A spell with a hybrid pip you can pay with one colored mana or two generic.",
    ),
    (
        "cycle-tsp-u-split-second",
        "A spell with split second, so no one can respond with spells or abilities while it's on the stack.",
    ),
    (
        "cycle-tdm-utility-land",
        "A tapland that produces one color and enters untapped only if you control the right basic types, plus a bonus activated ability.",
    ),
    (
        "cycle-uds-seer",
        "A creature that taps to reveal cards of its color from your hand for an effect that scales with how many you reveal.",
    ),
    (
        "cycle-ulg-creatureland",
        "A land that enters tapped and can pay mana to become a creature until end of turn.",
    ),
    (
        "cycle-ulg-sleeping-enchantment",
        "Waits until a battlefield condition is met, then sacrifices itself for a powerful one-time effect.",
    ),
    (
        "cycle-thb-nymph",
        "A Nymph enchantment creature with a small ability tied to its color.",
    ),
    (
        "cycle-unf-c-employee",
        "A creature that opens an Attraction when it enters the battlefield.",
    ),
    (
        "cycle-unf-myra-s-marvels",
        "A partner legendary creature that names something as it enters and rewards you when you cast a spell matching it.",
    ),
    (
        "cycle-unf-u-blank",
        "A card that lets you put a name sticker on it, whose effect depends on that sticker.",
    ),
    (
        "cycle-som-1mv-combat-trick",
        "A one-mana instant or sorcery that gives a creature a temporary boost or keyword.",
    ),
    (
        "cycle-spe-u-legend",
        "A cycle of uncommon legendary creatures, each with its own distinct ability.",
    ),
    (
        "cycle-spe-face-legend",
        "A cycle of legendary creatures depicting signature heroes and villains, each with its own unique ability.",
    ),
    (
        "cycle-stx-dragon-founder",
        "A cycle of legendary Elder Dragons, one per Strixhaven college, each with flying and a college-themed ability.",
    ),
    (
        "cycle-spm-c-hybrid",
        "A cycle of common cards with hybrid mana costs, each with a different minor effect.",
    ),
    (
        "cycle-spm-r-hybrid",
        "A cycle of rare cards with hybrid mana costs, each with a different powerful effect.",
    ),
    (
        "cycle-ths-c-allied-ability",
        "A cycle of common creatures, each with its own activated ability.",
    ),
    (
        "cycle-ths-cantrip-aura",
        "A cycle of Auras that draw you a card when they enter the battlefield.",
    ),
    (
        "cycle-spm-tmdfc",
        "A double-faced legendary creature that transforms between two forms.",
    ),
    (
        "cycle-sth-allied-sliver",
        "A Sliver that grants all Slivers a shared ability, like life gain, shroud, or a damage ability.",
    ),
    (
        "cycle-ths-major-god",
        "An indestructible god creature that stops being a creature at low devotion and has a color-defining ability.",
    ),
    (
        "cycle-ths-ordeal",
        "An aura that grows the enchanted creature each attack, then sacrifices itself at three counters for a bonus effect.",
    ),
    (
        "cycle-som-replica",
        "An artifact creature you can sacrifice for a one-shot effect like removal, damage, or card draw.",
    ),
    (
        "cycle-stx-c-2c-noncreature",
        "A two-color common instant or sorcery that gives you a burst of card advantage or board impact.",
    ),
    (
        "cycle-tla-m-saga",
        "A Saga that transforms into a legendary creature after its final chapter resolves.",
    ),
    (
        "cycle-tla-shrine",
        "A Shrine whose effect scales with how many Shrines you control and repeats each time another enters.",
    ),
    (
        "cycle-stx-u-lesson",
        "A Lesson that removes or neutralizes a permanent, sometimes creating a token.",
    ),
    (
        "cycle-tmp-enemy-backward-hate",
        "Hoses a specific color, weakening that color's cards or rewarding you when an opponent uses it.",
    ),
    (
        "cycle-tmp-medallion",
        "A cheap artifact that makes spells of one color you cast cost 1 less.",
    ),
    (
        "cycle-tmp-napland",
        "A land that taps for colorless freely, or for one of two colors if it stays tapped through your next untap.",
    ),
    (
        "cycle-soc-face-commander",
        "A legendary creature or planeswalker built to lead its own themed Commander deck.",
    ),
    (
        "cycle-stx-summoning",
        "A Lesson sorcery that creates a specific creature token.",
    ),
    (
        "cycle-tmt-enemy-ability",
        "A legendary creature with its own unique signature ability.",
    ),
    (
        "cycle-tmt-landcycler",
        "A creature you can discard to search your library for a land of a given basic type.",
    ),
    (
        "cycle-tdm-devotee",
        "A creature that pays 1 mana once each turn to add any of its three colors.",
    ),
    (
        "cycle-tmt-u-technique",
        "A spell you can cast cheaper by returning an unblocked attacker to hand during declare blockers.",
    ),
    (
        "cycle-tor-c-madness",
        "A card with madness, which you can cast for its madness cost when you discard it.",
    ),
    (
        "cycle-tor-disorder-enchantment",
        "An enchantment you activate by discarding a card, or sacrifice once for the same effect.",
    ),
    (
        "cycle-tdm-c-twobrid",
        "A creature with a twobrid mana cost, castable with 2 generic mana or the listed color.",
    ),
    (
        "cycle-tor-u-madness",
        "A card with madness, letting you cast it for its madness cost when you discard it instead of putting it in your graveyard.",
    ),
    (
        "cycle-tdc-will",
        "A modal spell that lets you choose both modes instead of one if you control a commander.",
    ),
    (
        "cycle-tdm-c-behold",
        "A spell that gets a bonus effect if you behold a Dragon, revealing one from your hand or choosing one you control.",
    ),
    (
        "cycle-soc-octoland",
        "A dual land that enters tapped unless your opponents control eight or more lands.",
    ),
    (
        "cycle-sos-r-2c-legend",
        "A rare two-color legendary creature that anchors its own build-around strategy.",
    ),
    (
        "cycle-sos-mascot",
        "A Mascot creature with an animal type that grows or triggers off a specific kind of play.",
    ),
    (
        "cycle-tdm-u-wedge-nondragon",
        "An uncommon card tied to a three-color wedge clan, and not itself a Dragon.",
    ),
    (
        "cycle-soi-c-typal-boost",
        "A creature that rewards you for controlling another creature of its own type.",
    ),
    (
        "cycle-sos-emeritus",
        "A double-faced card that's a creature on one side and a classic reprinted spell on the other.",
    ),
    (
        "cycle-tdm-saga",
        "A Saga enchantment that triggers a new effect each time a lore counter is added.",
    ),
    (
        "cycle-tdm-u-omen",
        "A creature whose card has an Omen spell you may cast, then shuffle into your library.",
    ),
    (
        "cycle-tsp-u-sliver",
        "A Sliver that grants all Slivers a shared ability.",
    ),
    (
        "cycle-soi-r-two-color",
        "A two-color rare card, one of a cycle covering each color pair.",
    ),
    (
        "cycle-soi-tapland",
        "A dual land that enters tapped and taps for two colors of mana.",
    ),
    (
        "cycle-thb-nyxborn",
        "A vanilla enchantment creature with no abilities.",
    ),
    (
        "cycle-uds-scent",
        "Reveals cards of one color from your hand to scale that spell's effect by how many you reveal.",
    ),
    (
        "cycle-sos-mono-u-legend",
        "A monocolored legendary creature, one per color, each with its own signature ability.",
    ),
    (
        "cycle-ugl-multiplayer",
        "A cycle of Unglued cards built around teams and targeting in multiplayer games.",
    ),
    (
        "cycle-soi-reveal-land",
        "A dual land that enters untapped if you reveal a card of either of its two basic land types.",
    ),
    (
        "cycle-ulg-perpetual-aura",
        "An Aura that returns to your hand instead of staying in the graveyard when it's put there from the battlefield.",
    ),
    (
        "cycle-thb-c-removal-aura",
        "A common Aura used as creature removal.",
    ),
    (
        "cycle-thb-intervention",
        "An instant or sorcery offering a choice of two modes, each scaling with an X you choose.",
    ),
    (
        "cycle-thb-demigod",
        "An enchantment creature whose power or toughness equals your devotion to its color.",
    ),
    (
        "cycle-soi-vessel",
        "A cheap enchantment you sacrifice for a one-time effect in its color.",
    ),
    (
        "cycle-thb-monocolor-god",
        "An indestructible god that isn't a creature unless your devotion to its color is at least five.",
    ),
    (
        "cycle-unf-i-spy",
        "An Unfinity card whose effect scales with real objects you can see from your seat.",
    ),
    (
        "cycle-sok-epic-sorcery",
        "A sorcery with epic: it ends your spellcasting but copies itself free each upkeep.",
    ),
    (
        "cycle-unf-sticker-activation",
        "A creature that lets you place a sticker, then has an ability that keys off stickered creatures.",
    ),
    (
        "cycle-snc-r-two-color",
        "One of five two-color rares, one signature creature or spell for each Capenna faction.",
    ),
    (
        "cycle-snc-c-tapland",
        "A dual land that enters tapped, taps for two colors, and can be sacrificed to draw a card.",
    ),
    (
        "cycle-snc-fixer",
        "A creature you can exile from hand to fix a land's mana, then cast it later from exile.",
    ),
    (
        "cycle-sok-maro-creature",
        "A legendary creature whose power and toughness equal cards in a player's hand.",
    ),
    (
        "cycle-sos-dragon",
        "An Elder Dragon that gives your instant and sorcery spells an extra keyword ability.",
    ),
    (
        "cycle-som-spellbomb",
        "A cheap artifact with a sacrifice effect that also lets you pay to draw when it dies.",
    ),
    (
        "cycle-sos-student",
        "A modal double-faced card playable as a creature or cast instead as a sorcery.",
    ),
    (
        "cycle-spe-c-creature",
        "A common creature from the Spider-Man set with a small triggered ability.",
    ),
    (
        "cycle-sok-upkeep-rescue",
        "A creature that returns one of your creatures of its color to your hand each upkeep.",
    ),
    (
        "cycle-sok-onna",
        "A Spirit that triggers an effect when it enters and can bounce itself to hand when you cast a Spirit or Arcane spell.",
    ),
    (
        "cycle-snc-fetchland",
        "A land that sacrifices itself to fetch one of three basic land types tapped and gain 1 life.",
    ),
    (
        "cycle-sok-maro-spell",
        "A card whose effect scales with the number of cards in your or an opponent's hand.",
    ),
    (
        "cycle-sok-shinen",
        "A creature with an ability you can also grant to another creature by discarding it with channel.",
    ),
    (
        "cycle-spm-color-coded-artifact",
        "An artifact with an ability keyed to a specific color of mana.",
    ),
    (
        "cycle-snc-color-hate",
        "Costs less to cast when it targets a permanent of a specific color.",
    ),
    (
        "cycle-ths-c-enemy-ability",
        "A creature with an activated ability, from a Theros common cycle.",
    ),
    (
        "cycle-som-smith",
        "A creature that triggers a bonus effect whenever you cast an artifact spell.",
    ),
    (
        "cycle-spm-surveil-dual",
        "A tapped dual land that can pay 4 and tap to surveil 1.",
    ),
    (
        "cycle-eve-hatchling",
        "A creature that enters with four -1/-1 counters, removing one each time you cast a spell in its two colors.",
    ),
    (
        "cycle-spm-u-hybrid",
        "One of a themed cycle of uncommon creatures and spells from the Spider-Man set.",
    ),
    (
        "cycle-snc-r-tricolor",
        "A three-color rare creature from a New Capenna cycle.",
    ),
    (
        "cycle-stx-double-keyword",
        "A creature with two abilities, usually a pair of simple keywords.",
    ),
    (
        "cycle-sth-wall",
        "A Wall with defender that carries an extra ability.",
    ),
    (
        "cycle-ths-nymph",
        "A Nymph with bestow that grants the enchanted creature +2/+2 and a keyword.",
    ),
    (
        "cycle-snc-r-shard-removal",
        "A rare three-color removal spell from Streets of New Capenna.",
    ),
    (
        "cycle-som-color-artifact",
        "An artifact that pays one colored mana for a bonus ability until end of turn.",
    ),
    (
        "cycle-stx-apprentice",
        "A one-drop creature with magecraft that triggers whenever you cast or copy an instant or sorcery.",
    ),
    (
        "cycle-stx-c-2c-creature",
        "A common two-drop creature tied to Strixhaven's counters, graveyard, or spells themes.",
    ),
    (
        "cycle-stx-command",
        "An instant or sorcery that lets you choose two different effects from a list of four.",
    ),
    (
        "cycle-tla-landcycler",
        "A creature you can discard to search your library for a land of the named type.",
    ),
    (
        "cycle-stx-c-hybrid-spell",
        "A common spell castable with either half of a hybrid mana symbol.",
    ),
    (
        "cycle-stx-campus",
        "A dual land that enters tapped, taps for two colors, and can later scry for {4}.",
    ),
    (
        "cycle-stx-hhhh",
        "A card that supports one of the five colleges' two-color archetypes.",
    ),
    (
        "cycle-stx-mastery",
        "A spell with an alternative, reduced cost that lets an opponent gain a benefit if you pay it.",
    ),
    (
        "cycle-sos-charm",
        "An instant that lets you choose one of three modal effects.",
    ),
    (
        "cycle-tdm-spirit-dragon",
        "A legendary Spirit Dragon with flying and a distinct, powerful ability.",
    ),
    (
        "unique-token-type",
        "Creates a token with a creature type found on essentially no other card.",
    ),
    (
        "cycle-rna-guild-champion",
        "A legendary creature that embodies its guild's identity and strategy.",
    ),
    (
        "cycle-rna-guild-color-ability",
        "A common creature with an activated ability that gives itself a temporary boost for mana.",
    ),
    (
        "cycle-rna-guildmage",
        "A creature with two activated abilities, one for each of its guild's colors.",
    ),
    (
        "cycle-rna-guildmaster",
        "A splashy legendary leader of its guild, a planeswalker or marquee creature.",
    ),
    (
        "cycle-rna-hybrid-creature",
        "A common hybrid creature castable with either of its guild's colors, with a simple ability.",
    ),
    (
        "cycle-rna-locket",
        "An artifact that taps for either of two guild colors and can be sacrificed to draw two cards.",
    ),
    (
        "cycle-rna-m-guild-spell",
        "One of Ravnica Allegiance's five multicolor guild mythics, one for each guild.",
    ),
    (
        "cycle-rna-m-monocolor",
        "One of Ravnica Allegiance's cycle of splashy monocolored mythics, one per color.",
    ),
    (
        "cycle-rna-mmnn",
        "One of Ravnica Allegiance's two-color uncommon creatures, one per guild.",
    ),
    (
        "cycle-rna-r-split",
        "One of Ravnica Allegiance's rare split cards, pairing two spells from a single guild's colors.",
    ),
    (
        "cycle-rna-u-split",
        "One of Ravnica Allegiance's uncommon split cards, pairing two spells from a single guild's colors.",
    ),
    (
        "cycle-roe-invoker",
        "A creature with an expensive {8} activated ability that produces a powerful effect.",
    ),
    (
        "cycle-roe-r-leveler",
        "A creature with level up that gains counters to grow stronger and gain new abilities.",
    ),
    (
        "cycle-rtr-land-aura",
        "An Aura that enchants a land and grants it a new activated or triggered ability.",
    ),
    (
        "cycle-rtr-m-monocolor",
        "A powerful mono-colored mythic rare.",
    ),
    (
        "cycle-rtr-uncounterable-spell",
        "A spell that can't be countered.",
    ),
    (
        "cycle-rvr-r-mono-legend",
        "A mono-colored legendary creature.",
    ),
    (
        "cycle-s99-vanilla-1-drop",
        "A 1 mana creature with no abilities, just power and toughness.",
    ),
    (
        "cycle-scd-face-commander",
        "A legendary creature designed to lead and define its own Commander deck.",
    ),
    (
        "cycle-scg-5m-landcycler",
        "A creature with basic landcycling: discard it to search for a basic land of that type into your hand.",
    ),
    (
        "cycle-scg-c-storm-instant",
        "An instant with storm, copying itself for each spell cast earlier this turn.",
    ),
    (
        "cycle-scg-decree",
        "A cycling spell whose cycling trigger delivers a smaller version of its main effect.",
    ),
    (
        "cycle-scg-dragon-aura",
        "An Aura that boosts the creature it enchants and returns from your graveyard when a big creature enters.",
    ),
    (
        "cycle-scg-warchief",
        "A creature that makes spells of its creature type cost less and aids other creatures of that type.",
    ),
    (
        "cycle-shm-allied-conspire",
        "A spell with conspire: tap two creatures that share a color with it to copy it as you cast it.",
    ),
    (
        "cycle-shm-allied-scarecrow",
        "An artifact creature that gains keyword abilities while you control creatures of matching colors.",
    ),
    (
        "cycle-shm-allied-selfboost",
        "A creature with a hybrid-mana ability to pump its own power or toughness until end of turn.",
    ),
    (
        "cycle-shm-avatar",
        "A rare Spirit Avatar with a powerful ability tied to its colors.",
    ),
    (
        "cycle-shm-basic-land-count",
        "An effect that scales with how many of a certain basic land type you control.",
    ),
    (
        "cycle-shm-c-hybrid-1-drop",
        "A common one-mana creature with a hybrid mana cost.",
    ),
    (
        "cycle-shm-cohort",
        "A creature that gets +1/+1 as long as you control another creature of its color.",
    ),
    (
        "cycle-shm-demigod-aura",
        "An aura that grants +1/+1 and a different keyword depending on the enchanted creature's color.",
    ),
    (
        "cycle-shm-duo",
        "A creature that gets +1/+1 from casting one color of spell and a keyword from casting another.",
    ),
    (
        "cycle-shm-filterland",
        "Taps for colorless, or filters one mana into two mana of its two colors.",
    ),
    (
        "cycle-shm-hate-enhanced-spell",
        "A removal or disruption spell that gets a bonus effect against a specific pair of colors.",
    ),
    (
        "cycle-shm-hideaway-creature",
        "A creature card designed to be found and cast free from a hideaway land.",
    ),
    (
        "cycle-shm-hybrid-modal",
        "A spell with two modes chosen by which hybrid mana color you spend, doing both if you spend both.",
    ),
    (
        "cycle-shm-initiate",
        "Lets you pay 1 to trigger a small effect when any player casts a spell of its color.",
    ),
    (
        "cycle-shm-mentor",
        "A creature that grants an ability to all your creatures of its color.",
    ),
    (
        "cycle-shm-monocolored-conspire",
        "A monocolored spell with conspire: tap two creatures of its color as you cast it to copy it.",
    ),
    (
        "cycle-shm-permanent-count",
        "A creature whose power and toughness equal the number of permanents of one color you control.",
    ),
    (
        "cycle-shm-r-persist",
        "A creature with a strong enters-the-battlefield effect and persist, letting it return once after dying.",
    ),
    (
        "cycle-shm-reflection",
        "An enchantment that doubles one of your resources, like mana, cards drawn, life gained, or damage dealt.",
    ),
    (
        "cycle-shm-twobrid-spell",
        "A spell with a twobrid cost, payable with its colored mana or any two mana instead.",
    ),
    (
        "cycle-shm-u-hybrid-3-drop",
        "A three-mana hybrid creature castable with either of its two colors.",
    ),
    (
        "cycle-shm-unblockable-enemy",
        "A creature that can't be blocked by creatures of one specific color.",
    ),
    (
        "cycle-shm-utilityland",
        "A tapped land that taps for colored mana and has a small bonus ability once you control two or more permanents of that color.",
    ),
    (
        "cycle-shm-wisps",
        "Changes a target creature's color, grants it a related bonus, and draws you a card.",
    ),
    (
        "cycle-shm-witch",
        "A creature with a two-color activated ability.",
    ),
    (
        "cycle-snc-ascendancy",
        "An enchantment that builds ongoing value around its guild's signature mechanic.",
    ),
    (
        "cycle-snc-c-signpost",
        "A two-color creature whose ability showcases its guild's strategy.",
    ),
    (
        "cycle-snc-charm",
        "A modal instant that lets you choose one of three effects for its guild.",
    ),
    (
        "cycle-snc-hideaway",
        "An enchantment with hideaway that exiles a card and lets you play it once a guild condition is met.",
    ),
    (
        "cycle-snc-hybrid-legend",
        "A legendary creature with a hybrid mana cost built around its guild's mechanics.",
    ),
    (
        "cycle-snc-initiate",
        "A creature with a hybrid-mana activated ability that boosts itself or grants a keyword.",
    ),
    (
        "cycle-snc-leader",
        "A three-color legendary creature that leads its faction with a powerful signature ability.",
    ),
    (
        "cycle-snc-triland",
        "A tapped land that produces one of three colors of mana and has cycling {3}.",
    ),
    (
        "cycle-snc-u-legend",
        "An uncommon three-color legendary creature with a strong ability tied to its faction's theme.",
    ),
    (
        "cycle-snc-u-mno-henchman",
        "An uncommon creature that generates value when it enters or dies.",
    ),
    (
        "cycle-snc-u-signpost-creature",
        "A creature whose ability rewards its faction's draft archetype.",
    ),
    (
        "cycle-snc-u-signpost-nonc",
        "A noncreature spell or permanent that rewards its faction's draft archetype.",
    ),
    (
        "cycle-soc-alt-commander",
        "A legendary creature designed as an alternate commander, with its own build-around ability.",
    ),
    (
        "cycle-soc-guest-lecturer",
        "A double-faced card pairing a creature with a classic sorcery you can cast instead.",
    ),
    (
        "cycle-soc-two-color-prepare",
        "A two-color double-faced card pairing a creature with a spell on its other side.",
    ),
    (
        "cycle-soi-crazed-creature",
        "Triggers a small effect when it enters, then repeats it each opponent's upkeep once you have delirium.",
    ),
    (
        "cycle-soi-r-tdfc",
        "A transforming double-faced creature that flips into a stronger form when its condition is met.",
    ),
    (
        "cycle-soi-typal-lord",
        "A creature that rewards or strengthens the other creatures you control of its tribe.",
    ),
    (
        "cycle-sok-flip-ascendant",
        "A legendary creature that flips into a legendary enchantment once its condition is met.",
    ),
    (
        "cycle-sok-ghostlit-kami",
        "A Spirit with a repeatable tap ability, or a stronger version you get by discarding it via channel.",
    ),
    (
        "cycle-sok-legendary-kirin",
        "A flying legendary that triggers off the mana value of Spirit or Arcane spells you cast.",
    ),
    (
        "cycle-som-c-metalcraft",
        "Gains a bonus effect if you control three or more artifacts.",
    ),
    (
        "cycle-som-fastland",
        "A dual land that enters untapped only while you have two or fewer other lands.",
    ),
    (
        "cycle-som-trigon",
        "An artifact that stores charge counters to later fuel a repeatable one-color effect.",
    ),
    (
        "cycle-som-vanilla-creature",
        "A creature with no abilities, just power and toughness for its cost.",
    ),
    (
        "cycle-sorcery-magus",
        "A creature that sacrifices or exiles itself to recreate a famous sorcery's effect.",
    ),
    (
        "cycle-sos-a-a-b-b",
        "A creature from a cycle that generates card advantage or value.",
    ),
    (
        "cycle-sos-lesson",
        "A Lesson sorcery that exiles itself so you can recast a free copy from exile each main phase after.",
    ),
    (
        "cycle-sos-surveil-dual",
        "A dual land that enters tapped and can pay mana and tap to surveil 1.",
    ),
    (
        "cycle-sth-licid",
        "A creature that can turn into an Aura attached to a creature, granting or removing an ability while attached.",
    ),
    (
        "cycle-stx-m-college-spell",
        "A mythic Strixhaven card with a big, splashy payoff, one per college.",
    ),
    (
        "cycle-stx-m-mdfc",
        "A mythic modal double-faced card from Strixhaven with two castable faces.",
    ),
    (
        "cycle-stx-pledgemage",
        "A creature with magecraft that gets a bonus whenever you cast or copy an instant or sorcery.",
    ),
    (
        "cycle-stx-r-learner",
        "Lets you learn: fetch a Lesson card from outside the game, or discard then draw.",
    ),
    (
        "cycle-stx-r-lesson",
        "A Lesson sorcery you can fetch from outside the game with learn effects.",
    ),
    (
        "cycle-stx-r-mdfc",
        "A modal double-faced card, playable as its front permanent or its back-side spell.",
    ),
    (
        "cycle-stx-reveal-land",
        "A dual land that enters untapped if you reveal a matching basic land type from your hand.",
    ),
    (
        "cycle-stx-student",
        "A legendary two-color creature tied to a Strixhaven college, with a build-around ability.",
    ),
    (
        "type-errata-specific-horse",
        "A creature that used to be a distinct horse subtype but was errata'd to just plain Horse.",
    ),
    (
        "cycle-stx-u-mono-magecraft",
        "An uncommon creature with magecraft, triggering a bonus whenever you cast or copy an instant or sorcery.",
    ),
    (
        "cycle-sword-ally-color",
        "An equipment from an ally-colored Sword cycle that grants +2/+2, protection from two colors, and a combat damage trigger.",
    ),
    (
        "cycle-sword-enemy-color",
        "An equipment from the original enemy-colored Sword cycle that grants +2/+2, protection from two colors, and a combat damage trigger.",
    ),
    (
        "cycle-tangoland",
        "A dual land that enters tapped unless you control two or more basic lands.",
    ),
    (
        "cycle-tbng-inspired-token",
        "A creature token created by a card from this cycle.",
    ),
    (
        "cycle-tdc-alt-commander",
        "A legendary creature from this set designed as an alternate commander option for its deck.",
    ),
    (
        "cycle-tdc-face-commander",
        "A legendary creature from this set designed as its precon deck's primary commander.",
    ),
    (
        "cycle-tdm-dragonstorm",
        "An enchantment that gives a one time effect when it enters and returns to your hand when a Dragon you control enters.",
    ),
    (
        "cycle-tdm-m-clan-spell",
        "A mythic spell that delivers a big, splashy effect themed to one of Tarkir's clans.",
    ),
    (
        "cycle-tdm-r-dragon",
        "A rare Dragon creature card, sometimes with a spell half you can cast first.",
    ),
    (
        "cycle-tdm-siege",
        "An enchantment that lets you pick one of two clan-themed ongoing effects as it enters.",
    ),
    (
        "cycle-tmp-licid",
        "A creature that can tap to become an Aura, granting the enchanted creature an effect.",
    ),
    (
        "cycle-tmp-u-two-color",
        "An uncommon card that's one color but splashes a second color in its abilities.",
    ),
    (
        "cycle-tmt-class",
        "An enchantment that starts at level 1 and can be leveled up for mana to gain new abilities.",
    ),
    (
        "cycle-tmt-gainland",
        "A tapland that gains you 1 life when it enters and taps for two colors.",
    ),
    (
        "cycle-tmt-u-equipment",
        "One of a cycle of Equipment that boosts its wearer with stats or a keyword ability.",
    ),
    (
        "type-errata-specific-fish",
        "A creature errata'd to carry the Fish creature type specifically.",
    ),
    (
        "typal-artificer",
        "Rewards you for controlling Artificer creatures or cares how many you have.",
    ),
    (
        "type-errata-specific-elemental",
        "A creature errata'd to carry the Elemental creature type specifically.",
    ),
    (
        "cycle-tsp-r-buyback",
        "One of a cycle of rare spells with buyback, letting you pay extra to return it to your hand.",
    ),
    (
        "cycle-tsp-r-morph-creature",
        "One of a cycle of rare creatures with morph, letting you cast it face down and flip it up later.",
    ),
    (
        "cycle-tsp-two-color-legend",
        "One of a cycle of two-color legendary creatures.",
    ),
    (
        "type-errata-shark",
        "A creature that was retroactively given the Shark creature type.",
    ),
    (
        "type-errata-egg",
        "A creature that was retroactively given the Egg creature type.",
    ),
    (
        "cycle-unk-devoted",
        "A creature whose ability scales with your devotion to its three-color wedge.",
    ),
    (
        "cycle-usg-embrace",
        "An Aura that boosts the enchanted creature's stats and grants it keyword abilities.",
    ),
    (
        "cycle-usg-enemy-backward-hate",
        "A card that hoses a specific enemy color.",
    ),
    (
        "cycle-usg-enemy-forward-hate",
        "A card that hoses a specific enemy color.",
    ),
    (
        "cycle-usg-legendary-land",
        "A powerful legendary land with a color-themed mana or utility ability.",
    ),
    (
        "cycle-usg-perpetual-aura",
        "An Aura that returns to its owner's hand when it's put into a graveyard from the battlefield.",
    ),
    (
        "cycle-usg-r-growing-enchant",
        "Builds up verse counters each upkeep, then you sacrifice it for an effect that scales with how long you waited.",
    ),
    (
        "cycle-usg-u-growing-enchant",
        "Builds up verse counters each upkeep, then you sacrifice it for an effect that scales with how long you waited.",
    ),
    (
        "cycle-ust-dice-host",
        "A creature that rolls a six-sided die when it enters and applies an effect equal to the result.",
    ),
    (
        "cycle-ust-flavor-variant",
        "A card printed as multiple versions with identical rules text but different art or flavor text.",
    ),
    (
        "cycle-ust-m-contraption",
        "A Contraption whose effect triggers whenever you crank it.",
    ),
    (
        "cycle-ust-outside-assistance",
        "A card whose effect depends on a choice or answer from a person outside the game.",
    ),
    (
        "cycle-ust-u-watermark-matters",
        "Cares about permanents or spells that carry a specific watermark.",
    ),
    (
        "cycle-vis-charm",
        "An instant offering a choice of several small effects, pick one when you cast it.",
    ),
    (
        "cycle-vis-enemy-backward-hate",
        "Restricts or taxes what opponents' creatures and lands are allowed to do.",
    ),
    (
        "cycle-vis-karoo-land",
        "A land that enters tapped, returns a basic land to your hand, and taps for two mana.",
    ),
    (
        "cycle-voc-soulbond",
        "Pairs with another creature to share a bonus ability as long as you control both.",
    ),
    (
        "cycle-vow-c-typal-boost",
        "Gets cheaper, stronger, or gains an ability when you control a creature of a specific type.",
    ),
    (
        "cycle-vow-cemetery",
        "Exiles a card from a graveyard, then rewards you for playing spells or lands that share its card type.",
    ),
    (
        "cycle-vow-enemy-cleave",
        "A Crimson Vow cleave spell, stronger for its two-color cost, weaker for its cheaper base cost.",
    ),
    (
        "cycle-vow-m-tdfc",
        "One of the mythic transforming double-faced creature cards from Crimson Vow.",
    ),
    (
        "cycle-vow-slowland",
        "A dual land that enters tapped unless you control two or more other lands.",
    ),
    (
        "cycle-vow-typal-hate",
        "One of five Crimson Vow cards that punish or answer a specific creature type.",
    ),
    (
        "cycle-w16-r",
        "One of the reprinted rare creatures from the Welcome Deck 2016 cycle.",
    ),
    (
        "cycle-w16-u",
        "One of the reprinted uncommon cards from the Welcome Deck 2016 cycle.",
    ),
    (
        "cycle-w17-r-creature",
        "One of the reprinted rare creatures from the Welcome Deck 2017 cycle.",
    ),
    (
        "cycle-war-bond",
        "A sorcery that pairs two different effects on a single spell.",
    ),
    (
        "cycle-war-color-artifact",
        "An artifact built around a single color's strategy.",
    ),
    (
        "cycle-war-finale",
        "An X spell that gets a much bigger bonus effect if X is 10 or more.",
    ),
    (
        "cycle-war-gatewatch",
        "A planeswalker with a passive ability alongside its loyalty abilities.",
    ),
    (
        "cycle-war-god",
        "A legendary God that returns third from the top of its library when it dies or is exiled.",
    ),
    (
        "cycle-war-ravnican-legend",
        "A card depicting a notable Ravnican character or location.",
    ),
    (
        "cycle-war-triumph",
        "An instant or sorcery that gets a stronger effect if you control a matching planeswalker.",
    ),
    (
        "cycle-woc-court",
        "Makes you the monarch and grants a bonus effect each upkeep while you hold that title.",
    ),
    (
        "tutor-creature-green",
        "Searches your library for a green creature card.",
    ),
    (
        "cycle-woe-r-two-color",
        "A two-color rare from the Wilds of Eldraine cycle.",
    ),
    (
        "cycle-woe-restless-land",
        "A dual land that can become a creature and triggers an effect whenever it attacks.",
    ),
    (
        "cycle-woe-u-saga",
        "An uncommon Saga that unfolds a different effect over three turns as lore counters are added.",
    ),
    (
        "cycle-woe-virtue",
        "An enchantment with an Adventure: cast the cheaper instant or sorcery first, then the enchantment later.",
    ),
    (
        "cycle-wth-sac-aura",
        "An Aura that grants an ongoing effect and can be sacrificed for a one-time bonus.",
    ),
    (
        "cycle-wwk-allied-land-bonus",
        "Gains a bonus, like a keyword or trigger, as long as you control a basic land of a specific type.",
    ),
    (
        "cycle-wwk-c-multikicker",
        "Has multikicker and enters with a +1/+1 counter for each time it was kicked.",
    ),
    (
        "cycle-wwk-creatureland",
        "A land that can pay mana to turn itself into a creature for a turn.",
    ),
    (
        "cycle-wwk-enemy-hate-trap",
        "An instant trap with a reduced alternate cost that triggers off an opponent's specific action.",
    ),
    (
        "cycle-wwk-landfall-instant",
        "An instant with landfall that has a stronger effect if you played a land this turn.",
    ),
    (
        "cycle-wwk-utilityland",
        "A land that enters tapped but gives a small extra effect when it enters.",
    ),
    (
        "cycle-wwk-zendikon",
        "An aura that turns the enchanted land into a creature, returning to hand if it dies.",
    ),
    (
        "cycle-xln-c-explorer",
        "A creature that explores when it enters, taking a revealed land to hand or growing itself.",
    ),
    (
        "cycle-xln-keeper",
        "A creature you can sacrifice, paying seven mana and tapping, for a big late-game payoff.",
    ),
    (
        "cycle-xln-legendary-transform",
        "A legendary enchantment that transforms into a legendary land once its condition is met.",
    ),
    (
        "cycle-ymkm-incorporate",
        "Permanently adds mana to a card in your hand or graveyard and grants it a bonus effect.",
    ),
    (
        "cycle-yotj-collector",
        "A creature that conjures a specific Mox card into your hand once a condition is met.",
    ),
    (
        "cycle-zen-ascension",
        "An enchantment that gains quest counters, unlocking a powerful effect at a threshold.",
    ),
    (
        "cycle-zen-c-growing-ally",
        "An Ally that gets a +1/+1 counter whenever it or another Ally you control enters.",
    ),
    (
        "cycle-zen-c-kicker",
        "An instant with kicker that gets a stronger effect if you pay the extra cost.",
    ),
    (
        "cycle-zen-expedition",
        "An enchantment that gains quest counters from landfall, then sacrifices for a powerful effect.",
    ),
    (
        "cycle-zen-fetchland",
        "A land that sacrifices itself to search your library for one of two basic land types.",
    ),
    (
        "cycle-zen-landfall-self-pump",
        "A creature that gets +2/+2 until end of turn whenever a land enters under your control.",
    ),
    (
        "cycle-zen-namedland",
        "A land that enters tapped, produces one color of mana, and carries a powerful bonus ability.",
    ),
    (
        "cycle-zen-quest",
        "An enchantment that collects quest counters from a trigger, then sacrifices for a big payoff.",
    ),
    (
        "cycle-zen-r-ally",
        "An Ally creature whose ability keys off the Allies you control.",
    ),
    (
        "cycle-zen-refugeland",
        "A tapland that gains you 1 life when it enters and taps for two colors of mana.",
    ),
    (
        "cycle-zen-strong-kicker",
        "A kicker spell whose effect scales dramatically bigger if you pay the extra kicker cost.",
    ),
    (
        "cycle-zen-u-ally",
        "An Ally that triggers whenever it or another Ally you control enters, often scaling with your Allies.",
    ),
    (
        "cycle-zen-utilityland",
        "A tapland with a minor enters effect like a combat buff, life gain, or life loss.",
    ),
    (
        "cycle-znr-boltland",
        "A modal double-faced card you can cast as a sorcery or play as a land.",
    ),
    (
        "cycle-znr-m-mono-creature",
        "A powerful mono-colored mythic creature built around its color's strengths.",
    ),
    (
        "tap-fuel-permanent",
        "Lets you tap another untapped permanent you control to power an ability.",
    ),
    (
        "cycle-znr-m-mono-legend",
        "A mono-colored mythic legendary creature or planeswalker from Zendikar Rising's cycle.",
    ),
    (
        "cycle-znr-r-mdfc",
        "A rare modal double-faced card that's a spell on one side and a land on the other.",
    ),
    (
        "cycle-znr-u-mdfc-creature",
        "An uncommon modal double-faced card that's a creature on one side and a land on the other.",
    ),
    (
        "synergy-tutor",
        "Cares about searching your library or cards being put into your hand from it.",
    ),
    (
        "cycle-akh-cycle-effect",
        "Has a bonus effect that triggers when you cycle this card.",
    ),
    (
        "synergy-shadow",
        "Cares about shadow creatures, which only block or are blocked by other shadow creatures.",
    ),
    (
        "cycle-akh-monocolor-aftermath",
        "A mono-colored aftermath split card whose second half is cast from your graveyard.",
    ),
    (
        "deprecated-untapped-artifact",
        "An old artifact whose ability only functions while it stays untapped.",
    ),
    (
        "dice-reroll",
        "Lets you or another player reroll a die that was already rolled.",
    ),
    (
        "synergy-pw-tyvar",
        "Cares about Tyvar cards, grouping them alongside your Elves.",
    ),
    (
        "discard-outlet-artifact",
        "Lets you discard an artifact card to pay for an effect.",
    ),
    (
        "discard-outlet-last-drawn",
        "Lets you discard the last card you drew this turn to trigger an effect.",
    ),
    (
        "dual-land",
        "A land that can tap for two different colors of mana.",
    ),
    (
        "synergy-pw-liliana",
        "Rewards you for controlling a Liliana planeswalker.",
    ),
    (
        "synergy-pw-gideon",
        "Cares about or supports a Gideon planeswalker you control.",
    ),
    (
        "exiletouch",
        "Exiles a creature it deals damage to instead of just destroying it.",
    ),
    (
        "expansion-sweeper",
        "Destroys, exiles, or sacrifices all permanents originally printed in a chosen Magic set.",
    ),
    (
        "extra-draft",
        "A draft card that lets you draft extra cards or add extra boosters to the draft.",
    ),
    (
        "fire-drake",
        "A flying drake that can pump its power or toughness by one, once per turn.",
    ),
    (
        "synergy-pw-bolas",
        "Cares about or supports a Bolas planeswalker you control.",
    ),
    (
        "typal-zubera",
        "A Zubera that triggers an effect when it dies, scaled by how many Zubera died that turn.",
    ),
    (
        "synergy-power-up",
        "Cares about power-up abilities, one-time activations that usually add +1/+1 counters.",
    ),
    (
        "synergy-planet",
        "Cares about Planet permanents and the charge counters they carry.",
    ),
    (
        "synergy-ninjutsu",
        "Cares about the ninjutsu mechanic, swapping an unblocked attacker for a Ninja from your hand.",
    ),
    (
        "synergy-landwalk",
        "Cares about the landwalk keyword, which makes a creature unblockable if a defender controls the matching land type.",
    ),
    (
        "synergy-exert",
        "Cares about the exert mechanic, where an attacker skips its next untap in exchange for a bonus effect.",
    ),
    (
        "synergy-disturb",
        "Cares about the disturb mechanic, letting a creature card cast from the graveyard as a transformed spirit.",
    ),
    (
        "gains-intimidate",
        "Causes a creature to gain intimidate, so it can only be blocked by artifact creatures or creatures sharing its color.",
    ),
    (
        "unusual-landwalk",
        "Can't be blocked while the defending player controls a nonbasic land type, like legendary, desert, or snow.",
    ),
    (
        "synergy-dfc",
        "Cares about or interacts with double-faced cards.",
    ),
    (
        "synergy-curse",
        "Supports Curse Auras, rewarding you for attaching or piling them on players.",
    ),
    (
        "synergy-counterspell",
        "Rewards or empowers you whenever a spell or ability you control counters a spell.",
    ),
    (
        "synergy-chorus",
        "A Chorus spell whose intensity rises as you cast more Chorus cards, scaling its effect.",
    ),
    (
        "synergy-banding",
        "Grants, shares, or supports banding, letting creatures attack together as a band.",
    ),
    (
        "supercycle-legendary-land",
        "A legendary utility land that produces mana and carries a powerful extra ability.",
    ),
    (
        "storage-land",
        "A land that banks mana as counters over time to release for a big payout later.",
    ),
    (
        "sneak-equipment",
        "Puts an Equipment onto the battlefield without paying its mana cost.",
    ),
    (
        "siege-modal",
        "Enters with a named mode chosen from two options, granting an ongoing effect tied to that choice.",
    ),
    (
        "shardland",
        "A land that taps for any of three allied colors, a shard.",
    ),
    (
        "seek-artifact",
        "Finds an artifact card at random in your library and puts it into your hand or play.",
    ),
    (
        "gives-cumulative-upkeep",
        "Grants cumulative upkeep, forcing a growing payment each upkeep or the permanent is sacrificed.",
    ),
    (
        "gives-demonstrate",
        "Grants demonstrate, letting you copy a spell and have a chosen opponent copy it too.",
    ),
    (
        "roll-d4",
        "Has an effect that involves rolling a four-sided die (d4).",
    ),
    (
        "repeatable-roles",
        "Repeatedly creates Role tokens that attach to your creatures and buff them.",
    ),
    (
        "gives-double-team",
        "Grants double team, conjuring a duplicate into your hand when the creature attacks.",
    ),
    (
        "removes-mm-counters",
        "Removes -1/-1 counters from a creature.",
    ),
    (
        "regrowth-historic",
        "Returns a historic card (artifact, legendary, or Saga) from your graveyard to your hand.",
    ),
    (
        "reanimate-artifact-creature",
        "Returns an artifact creature card from a graveyard to the battlefield.",
    ),
    (
        "pseudo-cycling",
        "Lets you discard the card to draw a card, like cycling without the keyword.",
    ),
    (
        "player-spotlight",
        "A card made to celebrate a World Championship winner, with no shared mechanical theme.",
    ),
    (
        "outside-objects-matter",
        "Counts real-world objects you can see from your seat to power its effect.",
    ),
    (
        "gives-improvise",
        "Grants improvise, letting you tap untapped artifacts to help pay for other spells.",
    ),
    (
        "nonstandard-bestow",
        "A bestow creature that does more than just boost power and toughness when attached.",
    ),
    (
        "mountainfall",
        "Triggers an effect whenever a Mountain you control enters.",
    ),
    (
        "gives-player-shroud",
        "Gives you shroud, so you can't be the target of spells or abilities.",
    ),
    (
        "gives-poisonous",
        "Grants a creature poisonous, so it gives players poison counters on combat damage.",
    ),
    (
        "gives-rebound",
        "Grants instant and sorcery spells rebound, so you recast them free from exile next turn.",
    ),
    ("life-doubler", "Doubles a player's life total."),
    (
        "leyline",
        "A permanent you may put onto the battlefield for free if it starts in your opening hand.",
    ),
    (
        "islandfall",
        "Triggers an effect whenever an Island you control enters the battlefield.",
    ),
    (
        "instant-loyalty-ability",
        "Lets you activate a planeswalker's loyalty abilities any time you could cast an instant.",
    ),
    (
        "impulsive-mill",
        "Mills a card and lets you play it before the end of the turn.",
    ),
    (
        "impulse-legendary",
        "Digs through the top of your library and lets you grab a legendary card from among them.",
    ),
    (
        "impulse-creature-elf",
        "Digs through the top of your library and lets you grab an Elf card from among them.",
    ),
    (
        "impulse-creature-dragon",
        "Digs through the top of your library and lets you grab a Dragon card from among them.",
    ),
    (
        "impulse-battle",
        "Digs through the top of your library and lets you grab a noncreature, nonland card from among them.",
    ),
    (
        "hate-typal-wizard",
        "Punishes, taxes, or targets Wizard creatures specifically.",
    ),
    (
        "typal-werewolf",
        "Cares about or boosts Werewolf creatures you control.",
    ),
    (
        "hate-typal-non-zombie",
        "Punishes or destroys creatures that aren't Zombies.",
    ),
    (
        "hate-arcane",
        "Targets, punishes, or protects against Spirit and Arcane spells.",
    ),
    (
        "hate-suspect",
        "Removes or exploits a creature's suspected status.",
    ),
    (
        "cycle-c16-face-commander",
        "A four-color legendary creature from the Commander 2016 face-commander cycle.",
    ),
    (
        "hate-spacecraft",
        "Destroys, exiles, or otherwise disrupts Spacecraft permanents.",
    ),
    (
        "cycle-c16-r-partner",
        "A two-color legendary creature with partner from the Commander 2016 rare cycle.",
    ),
    (
        "hate-deathtouch",
        "Strips deathtouch from opponents' creatures or protects yours from it.",
    ),
    (
        "cycle-c20-alt-commander",
        "A legendary creature built to serve as an alternate commander for its Commander 2020 deck.",
    ),
    (
        "cycle-c21-face-commander",
        "A legendary creature designed as the headline commander of its Commander 2021 deck.",
    ),
    (
        "cycle-clb-back-enemy-legend",
        "An enemy-colored legendary creature that can pair with a Background as a second commander.",
    ),
    (
        "cycle-clb-c-background",
        "A Background enchantment that grants your commander an extra ability when paired with it.",
    ),
    (
        "typal-turtle",
        "Rewards you for controlling Turtle creatures.",
    ),
    (
        "typal-serpent",
        "Rewards you for controlling Serpent creatures.",
    ),
    (
        "tutor-creature-doctor",
        "Searches your library for a Doctor card and puts it into your hand.",
    ),
    (
        "typal-ogre",
        "Rewards or requires controlling an Ogre, often punishing you if you don't.",
    ),
    (
        "typal-non-wall",
        "Cares about creatures that aren't Walls, boosting or targeting them specifically.",
    ),
    (
        "cycle-hou-power-eternalizer",
        "A creature with a power-based effect that can eternalize from your graveyard.",
    ),
    (
        "typal-nightmare",
        "Cares about Nightmare creatures, boosting them or triggering off them.",
    ),
    (
        "cycle-ice-depletion-land",
        "A dual land that taps for either of two colors but stays tapped an extra turn.",
    ),
    (
        "cycle-ice-painland",
        "A dual land that taps for colorless freely or either of two colors for 1 life.",
    ),
    (
        "cycle-ice-r-tricolor",
        "One of a cycle of three-color Ice Age rares, each with a unique, powerful ability.",
    ),
    (
        "cycle-ice-r-two-color",
        "One of a cycle of two-color Ice Age rares, each with a distinct powerful ability.",
    ),
    (
        "cycle-iko-ultimatum",
        "One of a cycle of costly three-color sorceries from Ikoria with a huge, game-swinging effect.",
    ),
    (
        "cycle-iko-wedge-enchantment",
        "One of a cycle of three-color Ikoria enchantments that build ongoing card or value advantage.",
    ),
    (
        "toughness-matters-total",
        "Cares about the combined toughness of all the creatures you control.",
    ),
    (
        "cycle-inv-allied-2-2",
        "One of a cycle of Invasion 2/2 creatures, each with protection from an allied color.",
    ),
    (
        "cycle-inv-allied-t-ability",
        "One of a cycle of Invasion creatures, each with a tap ability tied to an allied color.",
    ),
    (
        "cycle-inv-apprentice",
        "A creature with two tap abilities, each activated by a different color of mana.",
    ),
    (
        "cycle-inv-backward-ability",
        "A creature with an activated ability that costs a single colored mana.",
    ),
    (
        "cycle-inv-c-ally-kicker-spell",
        "A spell with an allied-color kicker that adds an extra effect if you pay it.",
    ),
    (
        "cycle-inv-c-nonc-kicker-spell",
        "A noncreature spell with a generic kicker cost that boosts its effect if paid.",
    ),
    (
        "cycle-inv-c-two-color",
        "A common two-color card from Invasion.",
    ),
    (
        "cycle-inv-cameo",
        "An artifact that taps to add one of two colors of mana.",
    ),
    (
        "cycle-inv-leech",
        "A creature that makes spells of one color cost you more to cast.",
    ),
    (
        "cycle-inv-master",
        "A creature with two tap abilities, each activated with a different color of mana.",
    ),
    (
        "cycle-inv-primeval-dragon",
        "A flying dragon that triggers a powerful bonus effect when it deals combat damage to a player.",
    ),
    (
        "cycle-inv-sac-enchantment",
        "An enchantment with a passive effect that can be sacrificed for a one-time bonus.",
    ),
    (
        "cycle-inv-sacland",
        "A land that enters tapped for one color, or can be sacrificed for two colors of mana.",
    ),
    (
        "cycle-inv-selfbounce-aura",
        "An aura with a small effect that can pay to return itself to your hand.",
    ),
    (
        "cycle-jou-c-strive",
        "An instant that buffs any number of target creatures, costing more mana per target beyond the first.",
    ),
    (
        "cycle-jou-dictate",
        "A flash enchantment with a strong symmetrical or one-sided battlefield effect.",
    ),
    (
        "cycle-jou-draft-signpost",
        "A two-color card built to point drafters toward its guild's archetype.",
    ),
    (
        "cycle-jou-font",
        "An enchantment you sacrifice for a one-shot effect matching its color.",
    ),
    (
        "cycle-jou-land-type-matters",
        "A creature whose stats or enters-the-battlefield effect scale with a specific basic land type you control.",
    ),
    (
        "cycle-jou-minor-god",
        "An indestructible god that stops being a creature unless your devotion to its colors is high enough.",
    ),
    (
        "cycle-jou-nymph",
        "A Nymph with constellation, triggering whenever it or another enchantment you control enters.",
    ),
    (
        "cycle-jou-u-bestow",
        "A bestow creature whose granted buff comes with a matching drawback for the creature it's attached to.",
    ),
    (
        "cycle-jud-1mv-martyr",
        "A cheap creature you sacrifice for a small one-shot effect.",
    ),
    (
        "typal-mutant-ninja-turtle",
        "Cares about or supports creatures with the Mutant, Ninja, or Turtle creature type.",
    ),
    (
        "cycle-jud-incarnation",
        "A creature that grants your whole team its keyword ability from the graveyard if you control the matching basic land type.",
    ),
    (
        "cycle-jud-wish",
        "Lets you reveal a card of a certain type from outside the game and put it into your hand.",
    ),
    (
        "tapper-planeswalker",
        "Taps a planeswalker and stops its loyalty abilities from being activated.",
    ),
    (
        "cycle-khm-living-weapon",
        "Equipment that can pay a cost as it enters to make a creature token and attach to it.",
    ),
    (
        "cycle-khm-m-foretell-spell",
        "A mythic spell with foretell, giving a bigger payoff if you cast it after foretelling it.",
    ),
    (
        "cycle-khm-m-god",
        "A modal double-faced card with a God creature on one face and a different permanent on the other.",
    ),
    (
        "cycle-khm-rune",
        "An Aura Rune that draws you a card and grants a keyword or bonus to the enchanted permanent or its equipped creature.",
    ),
    (
        "cycle-khm-snow-scaler",
        "A snow spell whose effect scales up based on how much snow mana was spent to cast it.",
    ),
    (
        "cycle-kld-color-artifact",
        "An artifact creature with an activated ability that grants itself a keyword until end of turn.",
    ),
    (
        "cycle-kld-fastland",
        "A dual land that enters untapped only if you control two or fewer other lands.",
    ),
    (
        "cycle-kld-gearhulk",
        "A large artifact creature with a keyword and a powerful enters-the-battlefield trigger.",
    ),
    (
        "cycle-kld-puzzleknot",
        "An artifact that gives a small bonus when it enters and lets you sacrifice it for the same effect again.",
    ),
    (
        "cycle-kld-thriving-creature",
        "A creature that enters giving you two energy counters, letting it pay energy to grow when it attacks.",
    ),
    (
        "cycle-ktk-ascendancy",
        "One enchantment per Khans clan, each granting a unique ongoing effect tied to that clan's strategy.",
    ),
    (
        "cycle-ktk-banner",
        "A mana rock that taps for any of its clan's three colors, or sacrifices itself to draw a card.",
    ),
    (
        "cycle-ktk-c-3mno-creature",
        "A common creature with morph and a three-color morph cost tied to its clan.",
    ),
    (
        "cycle-ktk-charm",
        "An instant that lets you choose one of three modal effects matching its clan's colors.",
    ),
    (
        "cycle-ktk-khan",
        "A legendary creature leading a Khans clan, built around that clan's signature strategy.",
    ),
    (
        "cycle-ktk-r-2mno-creature",
        "A rare three-color creature that rewards its clan's strategy.",
    ),
    (
        "cycle-ktk-r-two-color",
        "A rare card keyed to one of the five enemy two-color pairs.",
    ),
    (
        "cycle-ktk-reveal-morpher",
        "Morph creature that turns face up by revealing a card of its color, triggering a bonus effect.",
    ),
    (
        "cycle-ktk-u-2mno-creature",
        "A two-color creature with a helpful ability that triggers when it enters or attacks.",
    ),
    (
        "cycle-ktk-wedgeland",
        "A tapped land that taps for any of three colors in one of Khans of Tarkir's wedges.",
    ),
    (
        "cycle-lea-slush-art",
        "An early card whose design was built around leftover art rather than a shared mechanic.",
    ),
    (
        "cycle-lgn-c-sliver",
        "A common Sliver that grants every Sliver in play a shared ability.",
    ),
    (
        "typal-horse",
        "A card that cares about or boosts Horse creatures.",
    ),
    (
        "cycle-lrw-typal-revealer",
        "A creature that costs 3 more to cast unless you reveal a card of its type from your hand.",
    ),
    (
        "cycle-m15-legend",
        "A legendary creature with a build-around signature ability, one per color from the M15 cycle.",
    ),
    (
        "typal-griffin",
        "Cares about Griffin creatures, rewarding or boosting them.",
    ),
    (
        "typal-goat",
        "Cares about Goat creatures, rewarding or boosting them.",
    ),
    (
        "cycle-mmq-unwilling-creature",
        "A creature that lets each opponent gain a free benefit when it enters.",
    ),
    (
        "cycle-moc-alt-commander",
        "A two-named legendary creature offered as an alternate commander for its deck.",
    ),
    (
        "cycle-moc-face-commander",
        "A legendary creature designed as a deck's flagship commander around a signature strategy.",
    ),
    (
        "cycle-moc-path",
        "A sorcery that resolves an effect, then has each player vote for planeswalk or chaos.",
    ),
    (
        "cycle-moc-talent",
        "An aura that enchants a planeswalker, giving it a new loyalty ability and a bonus effect.",
    ),
    (
        "cycle-mom-c-dfc",
        "A common creature that transforms into a corrupted Phyrexian version of itself.",
    ),
    (
        "cycle-mom-c-landcycler",
        "A common creature with basic landcycling: discard it to search your library for a basic land.",
    ),
    (
        "cycle-mom-corrupted-dfc-leg",
        "A legendary creature that transforms into a corrupted Phyrexian version of itself.",
    ),
    (
        "cycle-mom-enemy-backward-dfc",
        "A creature in enemy colors that transforms into a corrupted Phyrexian version of itself.",
    ),
    (
        "cycle-mom-enemy-forward-dfc",
        "A creature in enemy colors that transforms into a corrupted Phyrexian version of itself.",
    ),
    (
        "cycle-mom-enemy-hate",
        "A card that gets stronger or more effective against your two enemy colors.",
    ),
    (
        "cycle-mom-praetor",
        "A double-faced legendary Phyrexian Praetor that transforms into an enchantment Saga.",
    ),
    (
        "cycle-mom-tricolor-team-up",
        "A three-color legendary creature that combines two named characters into one.",
    ),
    (
        "cycle-monocolor-atog",
        "A creature that repeatedly pays a cost, often a sacrifice, to give itself a temporary boost.",
    ),
    (
        "cycle-monocolor-gatewatch-oath",
        "A legendary enchantment with an enters-the-battlefield effect and an ongoing planeswalker payoff.",
    ),
    (
        "cycle-mor-0-0-elemental",
        "An Elemental that enters with +1/+1 counters it removes as a cost to power an ability.",
    ),
    (
        "cycle-mor-banneret",
        "A creature that reduces the cost of spells sharing one of its two creature types.",
    ),
    (
        "cycle-mor-c-changeling",
        "A changeling with every creature type that has or can gain a keyword ability.",
    ),
    (
        "cycle-mor-c-kinship",
        "A creature with kinship: each upkeep you may reveal a top-library card sharing its creature type for a bonus.",
    ),
    (
        "cycle-mor-clashback-spell",
        "A spell that clashes with an opponent after resolving, returning to its owner's hand if you win.",
    ),
    (
        "cycle-mor-typal-counter-lord",
        "Gives creatures of its type extra +1/+1 counters and grants your counter-bearing creatures a bonus.",
    ),
    (
        "cycle-mor-typal-equipment",
        "Equipment that you may attach to a creature of its type whenever one enters.",
    ),
    (
        "cycle-mor-typal-reward-spell",
        "An effect that scales with how many permanents you control of a creature type you choose.",
    ),
    (
        "cycle-mor-u-evoke",
        "An uncommon creature you can cast cheaply via evoke, sacrificing it to trigger a leaves-the-battlefield effect.",
    ),
    (
        "cycle-mor-u-kinship",
        "An uncommon creature with kinship: each upkeep you may reveal a top-library card sharing its creature type for a bonus.",
    ),
    (
        "cycle-morphling",
        "A creature with several cheap activated abilities that adjust its stats and grant keywords.",
    ),
    (
        "cycle-mrd-c-entwine",
        "A modal spell offering two effects, with an entwine cost letting you pay extra to get both.",
    ),
    (
        "cycle-mrd-golem",
        "An artifact creature with one activated ability that pumps it or grants a keyword for a turn.",
    ),
    (
        "cycle-mrd-mana-myr",
        "An artifact creature that taps to add one mana of a single color.",
    ),
    (
        "cycle-mrd-r-color-artifact",
        "An artifact whose ability is themed to one color of Magic.",
    ),
    (
        "cycle-mrd-replica",
        "An artifact creature you can sacrifice for a one-shot removal or disruption effect in its color.",
    ),
    (
        "cycle-mrd-shard",
        "An artifact with a small repeatable ability you pay for with generic or its color's mana.",
    ),
    (
        "cycle-mrd-spellbomb",
        "A cheap artifact you sacrifice for a small colored effect, or for {1} to draw a card.",
    ),
    (
        "cycle-mrd-talisman",
        "A mana rock that taps for colorless freely or for one of two colors at the cost of 1 damage to you.",
    ),
    (
        "cycle-mrd-tower",
        "An artifact you tap and pay eight mana to activate for a big repeatable effect.",
    ),
    (
        "cycle-msc-landcycler",
        "A spell you can instead discard for {2} to search your library for a basic land.",
    ),
    (
        "cycle-msc-origin",
        "A Saga that grants a different effect over its three chapters as lore counters are added.",
    ),
    (
        "cycle-msh-basic-dual",
        "A land that taps for colorless anytime, or for one of two colors if it just entered or you control a basic.",
    ),
    (
        "cycle-msh-c-modal-teamwork",
        "A modal spell that lets you tap creatures with enough total power to choose both modes instead of one.",
    ),
    (
        "cycle-msh-landcycler",
        "Discard it for a small cost to search your library for a basic land and put it into your hand.",
    ),
    (
        "cycle-msh-u-plan",
        "A Plan enchantment that accumulates counters from a repeated trigger, then sacrifices itself for a bigger payoff.",
    ),
    (
        "cycle-ncc-alt-commander",
        "A legendary creature offered as the alternate commander for a preconstructed Commander deck.",
    ),
    (
        "cycle-ncc-confluence",
        "A modal spell where you choose three effects from a list, repeats allowed.",
    ),
    (
        "cycle-ncc-enemy-multicolored",
        "A multicolored card in the set's enemy color pair cycle.",
    ),
    (
        "cycle-ncc-face-commander",
        "A legendary creature designed as the headline commander for its Commander deck.",
    ),
    (
        "cycle-ncc-r-two-color-legend",
        "A rare two-color legendary creature that anchors its own build-around theme.",
    ),
    (
        "cycle-ncc-tri-legend-reprint",
        "A reprinted three color legendary creature.",
    ),
    (
        "cycle-nec-myojin",
        "A legendary creature that enters indestructible if cast from hand, then sacrifices that protection for one huge effect.",
    ),
    (
        "cycle-nem-c-spellshaper",
        "A creature that taps and discards a card to trigger a small repeatable effect.",
    ),
    (
        "cycle-nem-fading-creature",
        "A creature with fading that loses a counter each upkeep until sacrificed, and can spend counters early for a bonus.",
    ),
    (
        "cycle-nem-free-spell",
        "A spell you can cast without paying its mana cost if you and an opponent each control the right basic land types.",
    ),
    (
        "typal-druid",
        "Cares about Druid creatures you control or cast.",
    ),
    (
        "cycle-nem-seal",
        "An enchantment you sacrifice later to trigger its effect.",
    ),
    (
        "cycle-neo-dragon",
        "A legendary flying dragon that lets you choose one of two powerful effects when it dies.",
    ),
    (
        "cycle-neo-invoke",
        "A single-color Kamigawa 'Invoke' instant or sorcery that delivers one large effect.",
    ),
    (
        "cycle-neo-legendary-land",
        "A legendary land you can discard to channel for a powerful effect, cheaper per legendary creature you control.",
    ),
    (
        "cycle-neo-march",
        "An instant you can discount by exiling cards of its color from hand, scaling its effect with X.",
    ),
    (
        "cycle-nph-chancellor",
        "A Phyrexian creature you can reveal from your opening hand for an early bonus effect.",
    ),
    (
        "cycle-nph-exarch",
        "A Phyrexian Cleric that lets you choose one of two small effects when it enters.",
    ),
    (
        "cycle-nph-praetor",
        "A legendary Phyrexian Praetor with a powerful static or triggered effect built around one color's theme.",
    ),
    (
        "cycle-nph-shrine",
        "An artifact that gains charge counters from your upkeep and spells of one color, then sacrifices for a scaling effect.",
    ),
    (
        "cycle-nph-souleater",
        "A creature with an activated ability payable with mana or 2 life via Phyrexian mana.",
    ),
    (
        "cycle-ody-allied-atog",
        "A creature that sacrifices, discards, or exiles cards to pump itself +1/+1 until end of turn.",
    ),
    (
        "cycle-ody-ally-filterland",
        "A land that taps for two allied colors after paying 1 generic mana.",
    ),
    (
        "cycle-ody-burst",
        "A spell that gets stronger for each copy of itself in any graveyard.",
    ),
    (
        "cycle-ody-desire",
        "An aura that alters the enchanted creature, gaining an extra effect once you have threshold.",
    ),
    (
        "cycle-ody-egg",
        "An artifact you sacrifice to add two colors of mana and draw a card.",
    ),
    (
        "cycle-ody-hound",
        "One of a cycle of common Dog creatures, each with its own small color-tied ability.",
    ),
    (
        "cycle-ody-land-aura",
        "One of a cycle of Auras that enchant a land and grant it a unique activated or triggered ability.",
    ),
    (
        "cycle-ody-lhurgoyf",
        "One of a cycle of Lhurgoyfs whose power and toughness equal the count of one card type in all graveyards.",
    ),
    (
        "cycle-ody-mmm-lord",
        "One of a cycle of creatures with an ability activated by tapping another untapped creature of their type.",
    ),
    (
        "cycle-ody-r-two-color",
        "One of a cycle of powerful two-color rares from Odyssey block, one per color pair.",
    ),
    (
        "typal-devil",
        "Cares about or affects Devil creatures, or is itself a Devil.",
    ),
    (
        "cycle-ody-retriever",
        "One of a cycle of creatures that return a card of one type from your graveyard to your hand when they enter.",
    ),
    (
        "cycle-ody-rites",
        "Discards any number of cards to fuel an effect that scales with how many you discarded.",
    ),
    (
        "cycle-ody-sacland",
        "Enters tapped, taps for one color, or sacrifices to add one mana of any color.",
    ),
    (
        "cycle-ody-shrine",
        "Triggers whenever any player casts a spell, scaling its effect with copies of that spell in graveyards.",
    ),
    (
        "cycle-ody-sphere",
        "Prevents 2 damage from sources of one color each time they'd hurt you.",
    ),
    (
        "cycle-ody-threshold-painland",
        "Taps for colored mana at the cost of 1 damage, and sacrifices for a bonus once you reach threshold.",
    ),
    (
        "cycle-ody-wincon-enchantment",
        "An enchantment that wins you the game outright once you meet its unusual condition.",
    ),
    (
        "cycle-ogw-tapland",
        "Enters tapped and taps for either of two colors.",
    ),
    (
        "cycle-one-colored-sphere",
        "A tapped land that adds one color of mana or can be sacrificed to draw a card.",
    ),
    (
        "cycle-one-dominus",
        "A legendary creature that doubles a specific effect and can pay a cost to become indestructible.",
    ),
    (
        "cycle-one-skullbomb",
        "An artifact you sacrifice to draw a card, or sacrifice with mana for a bonus effect plus a card.",
    ),
    (
        "cycle-one-sun-twilight",
        "An X spell with a scaling effect that becomes far stronger once X is 5 or more.",
    ),
    (
        "cycle-ons-aura-crown",
        "An Aura that affects the enchanted creature and can be sacrificed to extend that effect to creatures of its type.",
    ),
    (
        "cycle-ons-avatar",
        "A creature whose power and toughness each equal the number of a given creature type on the battlefield.",
    ),
    (
        "cycle-ons-c-cycle-effect",
        "A card with cycling that grants a smaller version of its main effect when you cycle it.",
    ),
    (
        "cycle-ons-chain-spell",
        "A spell whose affected player may copy it and choose a new target for the copy.",
    ),
    (
        "cycle-ons-charm",
        "A modal instant offering a choice of three small effects.",
    ),
    (
        "cycle-ons-courier",
        "A creature you can leave tapped to give a same-type creature +2/+2 and a keyword.",
    ),
    (
        "cycle-ons-cycling-matters",
        "Triggers an effect whenever any player cycles a card.",
    ),
    (
        "cycle-ons-fetchland",
        "A land you sacrifice, paying 1 life, to search your library for one of two basic land types.",
    ),
    (
        "cycle-ons-m-cycling-land",
        "A land that enters tapped and taps for one color, or can be cycled for a single mana of that color.",
    ),
    (
        "cycle-ons-pit-fighter",
        "A powerful standalone legendary creature with a strong built-in ability.",
    ),
    (
        "cycle-ons-u-cycle-effect",
        "A cycling card that also triggers a smaller version of its effect when you cycle it.",
    ),
    (
        "cycle-ons-word",
        "For {1}, replaces your next draw this turn with another effect, like damage, life, or a token.",
    ),
    (
        "cycle-ori-flip-walker",
        "A creature that transforms into a planeswalker once its flip condition is met.",
    ),
    (
        "cycle-ori-mentor",
        "A legendary creature from a themed cycle, each with a strong built-in ability tied to its color.",
    ),
    (
        "cycle-ori-pivotal-moment",
        "A splashy spell from a cycle built to swing the game in a single big moment.",
    ),
    (
        "cycle-ori-plane-enchantment",
        "An enchantment from a plane-themed cycle that gives you a repeatable value engine.",
    ),
    (
        "cycle-ori-r-spell-mastery",
        "Has spell mastery: gains a bonus effect if you have two or more instant or sorcery cards in your graveyard.",
    ),
    (
        "cycle-ori-same-name-enhanced",
        "Rewards having other cards with the same name in play or in your graveyard.",
    ),
    (
        "cycle-ori-u-spell-mastery",
        "Gains a bonus if two or more instant or sorcery cards are in your graveyard.",
    ),
    (
        "cycle-otj-join-up",
        "A legendary enchantment with an enter effect plus an ongoing payoff for your legendary creatures.",
    ),
    (
        "cycle-otj-r-monocolor-mount",
        "A single-colored Mount creature whose attack trigger improves once it's saddled.",
    ),
    (
        "cycle-pcy-ability-losing-creature",
        "A creature with a keyword ability that any player can pay to strip away until end of turn.",
    ),
    (
        "cycle-pcy-avatar",
        "A large creature that costs six less to cast when a specific board or life condition is met.",
    ),
    (
        "cycle-pcy-field-aura",
        "An aura that enchants a land, granting it a tap ability.",
    ),
    (
        "cycle-pcy-legendary-spellshaper",
        "A legendary creature that taps and discards two cards to power a strong activated ability.",
    ),
    (
        "cycle-pcy-pitchspell",
        "A spell you can cast by discarding a basic land card instead of paying its mana cost.",
    ),
    (
        "cycle-pcy-rhystic-bonus",
        "Has a bonus effect that happens unless a player pays extra mana to prevent it.",
    ),
    (
        "cycle-pcy-spellshaper-aura",
        "A flash Aura that gives the enchanted creature a keyword, stat change, or ability.",
    ),
    (
        "cycle-pcy-wind",
        "A powerful mono-color spell with a sweeping effect like mass removal, big pump, or heavy damage.",
    ),
    (
        "cycle-pip-enemy-filterland",
        "A land that taps, plus one generic mana, to add two mana of an enemy color pair.",
    ),
    (
        "cycle-plc-alternate-reality-legend",
        "A legendary creature reimagining an existing character in a different color or role.",
    ),
    (
        "cycle-plc-c-sliver",
        "A Sliver that grants an ability to every Sliver on the battlefield.",
    ),
    (
        "cycle-plc-charm",
        "A modal instant that lets you choose one of several small effects.",
    ),
    (
        "cycle-plc-enemy-sliver",
        "A Sliver that grants every Sliver an extra ability, often one you sacrifice for value.",
    ),
    (
        "cycle-plc-magus",
        "A creature that recreates a famous card's ability as its own.",
    ),
    (
        "cycle-plc-primeval-dragon",
        "A flying legendary Dragon that triggers a colored payoff when it hits a player.",
    ),
    (
        "cycle-plc-spellshaper",
        "A creature that taps and discards a card to produce a spell-like effect.",
    ),
    (
        "cycle-plc-suspend-x-creature",
        "A creature with variable stats that you can suspend for X to cast it after a delay.",
    ),
    (
        "cycle-plc-tsb-colorshift",
        "A Planar Chaos card that is a colorshifted version of a Time Spiral Timeshifted card.",
    ),
    (
        "cycle-plc-u-vanishing-creature",
        "A creature with vanishing that enters with time counters and is sacrificed once they run out.",
    ),
    (
        "cycle-pls-battlemage",
        "A creature with two kicker costs that triggers a separate effect for each kicker paid.",
    ),
    (
        "cycle-pls-c-gating-creature",
        "A creature that bounces one of your other creatures back to hand when it enters.",
    ),
    (
        "cycle-pls-c-two-color",
        "A two-color spell from Planeshift's gold spell cycle.",
    ),
    (
        "cycle-pls-cantrip",
        "A removal or disruption spell that also draws you a card.",
    ),
    (
        "cycle-pls-familiar",
        "A creature that reduces the cost of spells in two specific colors you cast.",
    ),
    (
        "cycle-pls-lair",
        "A land that taps for one of three colors, sacrificed unless you return another non-Lair land to hand.",
    ),
    (
        "cycle-pls-primeval-charm",
        "A modal instant that lets you choose one of three different effects.",
    ),
    (
        "cycle-pls-pw-enchantment",
        "An enchantment you activate to make an opponent reveal a card at random from their hand, with an effect scaled to its mana value.",
    ),
    (
        "cycle-pls-r-tricolor",
        "A three-color card built around its specific color combination.",
    ),
    (
        "cycle-pls-sacland-kicker-spell",
        "A spell you can kick by sacrificing one or more lands to make its effect hit an extra target or grow stronger.",
    ),
    (
        "cycle-pls-u-gating-creature",
        "A creature that returns one of your creatures to your hand when it enters.",
    ),
    (
        "cycle-rav-etb-aura",
        "An Aura with a one-time effect when it enters, plus an ongoing effect on the enchanted permanent.",
    ),
    (
        "cycle-rav-hunted-creature",
        "A creature that gives an opponent tokens when it enters the battlefield.",
    ),
    (
        "typal-berserker",
        "Rewards you for controlling, attacking with, or losing Berserkers.",
    ),
    (
        "cycle-rix-legendary-transform",
        "A legendary enchantment that transforms into a legendary land once its goal is met.",
    ),
    (
        "cycle-rix-m-monocolor",
        "One of Rivals of Ixalan's monocolored mythics, each with its own signature ability.",
    ),
    (
        "cycle-rna-c-guild-ability",
        "A common from Ravnica Allegiance showcasing its guild's signature mechanic.",
    ),
    (
        "affinity-for-creature-type",
        "Costs less to cast for each creature of a specific type you control.",
    ),
    (
        "typal-wraith",
        "Rewards you for controlling or attacking with Wraiths.",
    ),
    (
        "unique-doubler",
        "Doubles a value, resource, or effect, often in an unusual way.",
    ),
    (
        "sneak-planeswalker",
        "Puts a planeswalker card onto the battlefield from your hand without casting it.",
    ),
    (
        "cycle-thunderscape-vertical",
        "One of a vertical cycle of red Thunderscape creatures with black and green abilities.",
    ),
    (
        "sleeves-matter",
        "Cares about whether your physical deck is sleeved or unsleeved.",
    ),
    (
        "typal-aurochs",
        "An Aurochs that grows stronger when attacking alongside other Aurochs.",
    ),
    (
        "cycle-tla-ascension",
        "A cycle of enchantments that gain quest counters each turn and unlock a bonus at four.",
    ),
    (
        "shakedown",
        "Each combat opponents may pay a price to tap it down, or it attacks and grows.",
    ),
    (
        "cycle-arn-djinn",
        "A Djinn creature with a strong ability balanced by a built-in drawback.",
    ),
    (
        "cycle-arn-efreet",
        "An Efreet creature with a strong ability balanced by a built-in drawback.",
    ),
    (
        "cycle-tla-bending-lesson",
        "A Lesson spell tied to one of the four bending elements, each with its own twist.",
    ),
    (
        "gives-casualty",
        "Grants casualty to spells you cast, letting you sacrifice a creature to copy them.",
    ),
    (
        "cycle-lcc-alt-commander",
        "A legendary creature offered as an alternate commander for its precon deck.",
    ),
    (
        "scooch-dice",
        "Lets you adjust a die roll's result up or down by one.",
    ),
    (
        "seek-to-exile",
        "Seeks a card from your library and exiles it, often letting you play it later.",
    ),
    (
        "cycle-avr-legend-angel",
        "A legendary flying Angel from Avacyn Restored's mythic cycle, each with its own keywords.",
    ),
    (
        "roll-d8",
        "Has you roll an eight-sided die and applies an effect based on the result.",
    ),
    (
        "cycle-lcc-face-commander",
        "One of a cycle of legendary creatures built to headline their own Commander deck.",
    ),
    (
        "typal-imp",
        "Cares about Imps, rewarding you for controlling or casting creatures of that type.",
    ),
    (
        "rescue",
        "Returns a permanent you control to your hand, often to save it or reuse its effects.",
    ),
    (
        "cycle-tla-vertical-aang",
        "One of a cycle of alternate versions of Aang, each a different take on the same character.",
    ),
    (
        "repeatable-gold",
        "Creates Gold tokens again and again over the course of the game.",
    ),
    (
        "cycle-tmc-m-turtle",
        "One of the four monocolor Ninja Turtle legends, each with its own signature keyword and ability.",
    ),
    (
        "cycle-tmc-teamup",
        "One of four cycle cards pairing a Turtle with an ally, letting you choose one or both modes on entering.",
    ),
    (
        "removal-token",
        "Destroys or forces the sacrifice of a token.",
    ),
    (
        "removes-landwalk",
        "Takes away a creature's landwalk ability.",
    ),
    (
        "removes-trample",
        "Takes away a creature's trample ability.",
    ),
    (
        "cycle-ody-restock-creature",
        "A creature that puts cards from your graveyard on the bottom of your library.",
    ),
    (
        "cycle-ody-thought-beast",
        "A flying creature that shrinks your maximum hand size.",
    ),
    (
        "cycle-ody-x-of-the-y",
        "A sorcery that creates a creature token and can be cast again from your graveyard via flashback.",
    ),
    (
        "cycle-ons-symbiotic-vertical",
        "A creature that creates a swarm of 1/1 Insect tokens when it dies.",
    ),
    (
        "cycle-phyrexian-er",
        "A creature with an ability that triggers when damage is dealt to it.",
    ),
    (
        "cycle-plc-split-vertical",
        "A split card giving you a choice between two related spells on one card.",
    ),
    (
        "cycle-restock-legendary-land",
        "A legendary land that puts a card of a specific type from your graveyard on top of your library.",
    ),
    (
        "cycle-shm-untap",
        "Lets you untap this creature by putting a -1/-1 counter on it.",
    ),
    (
        "cycle-soi-x-madness-sorcery",
        "An X spell with madness whose effect scales higher if cast for its madness cost.",
    ),
    (
        "cycle-stx-treasure-spell",
        "A spell you can later discard for its hybrid mana cost to create a Treasure token.",
    ),
    (
        "cycle-tor-black-dreams",
        "Discards X cards as an additional cost for an effect that scales with X.",
    ),
    (
        "cycle-tor-red-punisher",
        "Lets an opponent choose between two harsh outcomes, punishing them either way.",
    ),
    (
        "cycle-triple-one-drop",
        "A one-mana creature with an unusually strong ability for its cost.",
    ),
    (
        "cycle-tsp-time-removal",
        "Removal that only destroys a nonblack creature that entered the battlefield recently.",
    ),
    (
        "cycle-ugl-rock-paper-scissors",
        "One of a cyclical trio where each creature stops a specific other one from attacking or blocking.",
    ),
    (
        "cycle-unf-stickered",
        "Gains a keyword ability as long as you control a permanent with a sticker on it.",
    ),
    (
        "cycle-woe-boon",
        "A legendary artifact with a strong activated or sacrifice ability tied to its witch's theme.",
    ),
    (
        "cycle-xln-dino-avatar",
        "A Dinosaur Avatar with a powerful effect when it enters, like damage, a sweeper, or life gain.",
    ),
    (
        "cycle-xln-sun-aspect",
        "A creature that boosts your Dinosaurs with cost reduction, combat bonuses, or a big sacrifice ability.",
    ),
    (
        "cycle-ylci-typal-seek",
        "Draws you a card, or seeks a card of a specific creature type instead if a condition is met.",
    ),
    (
        "cycle-yneo-costs-2-less",
        "Costs 2 less to cast when your hand meets a specific condition, like having no other lands or spells of its type.",
    ),
    (
        "cycle-ywoe-little-pig",
        "A creature from a three-card cycle themed after the Three Little Pigs.",
    ),
    (
        "cycle-znr-inscription",
        "A kicker spell offering three effects to choose from, letting you pick more if kicked.",
    ),
    (
        "cycle-znr-multiclass-creature",
        "A creature that counts as every Party class: Cleric, Rogue, Warrior, and Wizard.",
    ),
    (
        "damage-prevention-permanent",
        "Protects a permanent from damage, whether by prevention or a shield counter.",
    ),
    (
        "deprecated-legend-restriction",
        "Changes a creature's type to any type other than Wall, a pre-supertype legend rule holdover.",
    ),
    (
        "flavor-matters",
        "A joke card that asks a person outside the game a question about story or flavor.",
    ),
    (
        "flicker-enchantment",
        "Exiles then returns an enchantment (or other permanent) to the battlefield.",
    ),
    (
        "fourth-spell-matters",
        "Triggers a bonus effect when you cast your fourth spell in a turn.",
    ),
    (
        "gains-islandwalk",
        "Grants a creature islandwalk, unblockable while the defender controls an Island.",
    ),
    (
        "gains-suspend",
        "Gives a card suspend: exile it with time counters, then cast it free once they're gone.",
    ),
    (
        "gains-swampwalk",
        "Grants a creature swampwalk, unblockable while the defender controls a Swamp.",
    ),
    (
        "gains-wither",
        "Grants a creature wither, so damage it deals to creatures becomes -1/-1 counters.",
    ),
    (
        "gatefall",
        "Triggers a reward whenever a Gate you control enters the battlefield.",
    ),
    (
        "gatewatch-oath",
        "An Oath enchantment that gives an entry bonus and a lasting benefit tied to planeswalkers.",
    ),
    (
        "gating",
        "Returns a creature or enchantment you control to its owner's hand when this permanent enters.",
    ),
    (
        "gives-blitz",
        "Lets you cast a creature spell for its blitz cost, gaining haste but sacrificing it end of turn.",
    ),
    (
        "gives-conspire",
        "Lets a spell you cast be copied by tapping two creatures that share a color with it.",
    ),
    (
        "gives-decayed",
        "Grants decayed, so the creature can't block and is sacrificed after it attacks.",
    ),
    (
        "gives-evolve",
        "Grants evolve, adding a +1/+1 counter whenever a bigger creature you control enters.",
    ),
    (
        "gives-fake-flying",
        "Makes a creature unblockable except by flying or reach creatures, without granting it flying.",
    ),
    (
        "gives-jump-start",
        "Lets you cast an instant or sorcery from your graveyard by discarding a card, then exile it.",
    ),
    (
        "gives-mentor",
        "Grants mentor, so it puts a +1/+1 counter on a weaker attacking creature when it attacks.",
    ),
    (
        "gives-outlast",
        "Grants outlast, letting creatures tap to put a +1/+1 counter on themselves as a sorcery.",
    ),
    (
        "gives-provoke",
        "Grants provoke, letting an attacker force a chosen blocker to untap and block it.",
    ),
    (
        "gives-scavenge",
        "Grants scavenge: exile a creature card from your graveyard and pay its cost to put +1/+1 counters equal to its power on a creature.",
    ),
    (
        "gives-sunburst",
        "Grants sunburst, so a permanent enters with a counter for each color of mana spent to cast it.",
    ),
    (
        "gives-warp",
        "Grants warp, letting you cast a card for a warp cost, then exile it at end of turn to recast later.",
    ),
    (
        "graveyard-fuel-enchantment",
        "An enchantment that exiles cards from your graveyard to create or empower Zombie tokens.",
    ),
    (
        "graveyard-fuel-noncreature",
        "Lets you exile a noncreature, nonland card from your graveyard so you can cast it.",
    ),
    (
        "hate-adventure",
        "Targets or disrupts cards that have an Adventure.",
    ),
    (
        "hate-battle",
        "Targets or manipulates defense counters on a battle.",
    ),
    (
        "hate-color-non-share",
        "Punishes permanents or spells that don't share a color with something you control.",
    ),
    (
        "hate-exile-cast",
        "Stops or taxes players from casting spells out of exile or graveyards.",
    ),
    (
        "hate-infect",
        "Removes infect from creatures or otherwise shuts down the infect mechanic.",
    ),
    (
        "hate-life-payment",
        "Stops players from paying life to cast spells or activate abilities.",
    ),
    (
        "hate-phasing",
        "Interacts with the phasing mechanic, forcing or preventing permanents from phasing.",
    ),
    (
        "hate-protection",
        "Interacts with protection, often gaining it based on nearby creatures.",
    ),
    (
        "hate-typal-demon",
        "Punishes or interacts specifically with Demon creatures.",
    ),
    (
        "hate-typal-goat",
        "Punishes or interacts specifically with Goat creatures.",
    ),
    (
        "hate-typal-kavu",
        "Punishes or interacts specifically with Kavu creatures.",
    ),
    (
        "hate-typal-mercenary",
        "Punishes or interacts specifically with Mercenary creatures.",
    ),
    (
        "hate-typal-non-demon",
        "Punishes or destroys creatures that aren't Demons.",
    ),
    (
        "hate-typal-robot",
        "Grants protection from Robots to hose Robot creatures.",
    ),
    (
        "hate-typal-skeleton",
        "Exiles or destroys Skeleton creatures, sometimes alongside other undead.",
    ),
    (
        "high-five-matters",
        "Rewards you for high-fiving someone in real life, an Un-set joke mechanic.",
    ),
    (
        "impulse-creature-goblin",
        "Digs into your library for Goblin cards, putting them into your hand or onto the battlefield.",
    ),
    (
        "impulse-enchantment-saga",
        "Digs into your library to find a Saga card, putting it into your hand or onto the battlefield.",
    ),
    (
        "impulse-historic",
        "Digs through your library and grabs a historic card: an artifact, legendary, or Saga.",
    ),
    (
        "impulse-noncreature",
        "Digs through your library and grabs a noncreature card to hand or battlefield.",
    ),
    (
        "infernal-spawn-family",
        "A creature that lets you deal damage to a player by revealing it from your hand or library and speaking its catchphrase.",
    ),
    (
        "ip-matters",
        "Cares about Universes Beyond cards, rewarding or counting the different Universes among your permanents.",
    ),
    (
        "keyword-errata-hexproof",
        "A creature with hexproof, printed before the keyword existed and later updated to use it.",
    ),
    (
        "keyword-errata-scry",
        "A card that scries, printed before scry existed as a keyword or became evergreen.",
    ),
    (
        "land-kavu",
        "A creature that turns a target land into a basic land type until end of turn.",
    ),
    (
        "low-x-matters",
        "A spell with an X cost that gains an extra effect when X is a low value.",
    ),
    (
        "nemesis-mega-cycle",
        "A creature that chooses a player as it enters, then affects that chosen player.",
    ),
    (
        "night-matters",
        "Cares whether it is night, granting a bonus or discount when it is.",
    ),
    (
        "noncreature-virtual-vanilla",
        "A noncreature permanent with minimal, low-impact abilities that plays nearly blank.",
    ),
    (
        "pitch-spell",
        "Lets you exile or discard other cards instead of paying its mana cost.",
    ),
    (
        "prevent-damage-redirection",
        "Deals damage that can't be prevented or redirected to another permanent or player.",
    ),
    (
        "reanimate-noncreature",
        "Returns a noncreature permanent card from a graveyard to the battlefield.",
    ),
    (
        "regrowth-noncreature",
        "Returns a noncreature card from your graveyard to your hand.",
    ),
    (
        "reminder-text-matters",
        "Cares whether a card has reminder text, the italic notes in parentheses.",
    ),
    (
        "removal-nonenchantment",
        "Destroys, exiles, or shuffles away a permanent that isn't an enchantment.",
    ),
    (
        "removal-spacecraft",
        "Destroys or exiles a creature or Spacecraft.",
    ),
    (
        "repeatable-landers",
        "Creates Lander tokens again and again, which you sacrifice to fetch basic lands.",
    ),
    (
        "repeatable-token-generator",
        "Creates tokens over and over rather than just once.",
    ),
    (
        "repeatable-vibranium",
        "Creates Vibranium tokens again and again for extra colorless mana.",
    ),
    (
        "repeated-effect",
        "Repeats the same effect multiple times within a single spell or ability.",
    ),
    (
        "rescue-aura",
        "Returns a creature and any Auras attached to it to hand, saving them from removal.",
    ),
    (
        "restock-land",
        "Puts land cards from a graveyard back on top of a library to be drawn again.",
    ),
    (
        "roll-d10",
        "Rolls a ten sided die and applies an effect based on the result.",
    ),
    (
        "roll-d12",
        "Rolls a twelve sided die and applies an effect based on the result.",
    ),
    (
        "seek-land-forest",
        "Seeks a Forest card, pulling one from your library without searching or revealing.",
    ),
    (
        "seek-to-graveyard",
        "Seeks a card from your library and puts it straight into your graveyard.",
    ),
    (
        "singleton-matters",
        "Rewards or requires a deck with only single copies, no duplicate card names.",
    ),
    (
        "sneak-enchantment",
        "Puts an enchantment onto the battlefield without casting it.",
    ),
    (
        "snowfall",
        "Triggers a bonus whenever a snow permanent enters the battlefield under your control.",
    ),
    (
        "stock-turn",
        "Lets you skip a turn now in exchange for taking an extra turn later.",
    ),
    (
        "stun-counters-matter",
        "Cares about stun counters, which make a permanent skip untapping until one is removed.",
    ),
    (
        "synergy-ability-sticker",
        "Cares about putting ability stickers on permanents to grant them extra abilities.",
    ),
    (
        "synergy-boast",
        "Cares about boast, abilities you can activate only once, and only after that creature attacks.",
    ),
    (
        "synergy-cascade",
        "Cares about casting spells with cascade, which exile cards until a cheaper one to cast free.",
    ),
    (
        "synergy-collect-evidence",
        "Cares about collecting evidence, exiling cards from your graveyard of a set total mana value.",
    ),
    (
        "synergy-colorless-mana",
        "Rewards you for producing or spending colorless mana.",
    ),
    (
        "synergy-convoke",
        "Rewards you for casting spells with convoke.",
    ),
    (
        "synergy-flanking",
        "Rewards or supports creatures with flanking.",
    ),
    (
        "synergy-leaves-creature",
        "Triggers or grows stronger when a creature you control left the battlefield this turn.",
    ),
    (
        "synergy-level-up",
        "Rewards you for having creatures with level counters from level up.",
    ),
    (
        "synergy-modal",
        "Rewards you for casting spells that let you choose one or more modes.",
    ),
    (
        "synergy-pw-jace",
        "Grows stronger or does more while you control a Jace planeswalker.",
    ),
    (
        "synergy-pw-nissa",
        "Cares about or boosts effects while you control a Nissa planeswalker.",
    ),
    (
        "synergy-renown",
        "Cares about renown, which marks a creature after it first deals combat damage to a player.",
    ),
    (
        "synergy-seek",
        "Cares about seeking cards, a random tutor that puts matching cards into your hand.",
    ),
    (
        "synergy-self-burn",
        "Rewards you when your own sources deal damage to you.",
    ),
    (
        "synergy-shroud",
        "Cares about creatures with shroud, granting or copying that keyword.",
    ),
    (
        "synergy-sphere",
        "Cares about Locus and Sphere lands, rewarding you for playing them or having them in your graveyard.",
    ),
    (
        "synergy-unearth",
        "Cares about creatures with unearth or that were returned to play through unearth.",
    ),
    (
        "theft-aura",
        "Lets you take control of an Aura or Equipment that's already attached to a permanent.",
    ),
    (
        "theft-mana",
        "Forces another player to tap their lands for mana that you take or spend instead.",
    ),
    (
        "theft-noncreature",
        "Lets you take control of another player's noncreature permanent.",
    ),
    (
        "third-spell-matters",
        "Triggers an effect when you cast your third spell in a turn.",
    ),
    (
        "toys-matter",
        "Cares about toys you own, turning them into creatures or granting them abilities.",
    ),
    (
        "tutor-augment",
        "Searches your library for a card with augment.",
    ),
    ("tutor-battle", "Searches your library for a battle card."),
    (
        "tutor-creature-colorless",
        "Searches your library for a colorless creature card.",
    ),
    (
        "tutor-creature-demon",
        "Searches your library for a Demon card.",
    ),
    (
        "tutor-creature-merfolk",
        "Searches your library for a Merfolk card.",
    ),
    (
        "tutor-creature-power",
        "Searches your library for a creature card that meets a power requirement.",
    ),
    (
        "tutor-creature-spirit",
        "Searches your library for a Spirit card.",
    ),
    (
        "tutor-creature-toughness",
        "Searches your library for a creature card that meets a toughness requirement.",
    ),
    (
        "tutor-enchantment-shrine",
        "Searches your library for a Shrine card.",
    ),
    (
        "typal-advisor",
        "Rewards or synergizes with Advisor creatures you control.",
    ),
    (
        "typal-eldrazi-scion",
        "Creates, sacrifices, or otherwise cares about Eldrazi Scion tokens.",
    ),
    (
        "typal-egg",
        "Creates, sacrifices, or otherwise cares about Egg creature tokens.",
    ),
    ("typal-fractal", "Creates or grows Fractal creature tokens."),
    (
        "typal-gamma",
        "Rewards or synergizes with Gamma creatures you control.",
    ),
    (
        "typal-lhurgoyf",
        "Rewards or synergizes with Lhurgoyf creatures you control.",
    ),
    (
        "typal-non-spirit",
        "Cares specifically about creatures that are not Spirits.",
    ),
    (
        "typal-octopus",
        "Cares about or rewards you for controlling Octopus creatures.",
    ),
    (
        "typal-phoenix",
        "Cares about or rewards you for controlling Phoenix creatures.",
    ),
    (
        "typal-praetor",
        "Cares about or rewards you for controlling Praetor creatures.",
    ),
    (
        "typal-shapeshifter",
        "Cares about or rewards you for controlling Shapeshifter creatures, including changelings.",
    ),
    (
        "typal-slug",
        "Cares about or rewards you for controlling Slug creatures.",
    ),
    (
        "typal-unicorn",
        "Cares about or rewards you for controlling Unicorn creatures.",
    ),
    (
        "type-errata-cheetah",
        "A creature that was once type Cheetah before Wizards folded that type into Cat.",
    ),
    (
        "type-errata-cobra",
        "A creature retyped by errata to Snake, though its name and history call it a Cobra.",
    ),
    (
        "type-errata-ghoul",
        "A creature retyped by errata to Zombie, though its name and history call it a Ghoul.",
    ),
    (
        "type-errata-lion",
        "A creature retyped by errata to Cat, though its name and history call it a Lion.",
    ),
    (
        "type-errata-specific-ouphe",
        "A creature retyped by errata to the specific creature type Ouphe.",
    ),
    (
        "type-errata-specific-snake",
        "A creature retyped by errata to the specific creature type Snake.",
    ),
    (
        "type-errata-specific-zombie",
        "A creature retyped by errata to the specific creature type Zombie.",
    ),
    (
        "wedgeland",
        "A land that taps for a three-color wedge: two allied colors plus their shared enemy color.",
    ),
    (
        "gives-deathtouch-noncreature",
        "Grants deathtouch to your instant or sorcery spells.",
    ),
    (
        "synergy-emblem",
        "Rewards you for having or creating emblems.",
    ),
    (
        "synergy-energy",
        "Cares about having or gaining energy counters, without spending them.",
    ),
    (
        "gives-bushido",
        "Grants bushido, so the creature gets bigger whenever it blocks or becomes blocked.",
    ),
    (
        "cycle-aer-precon-tutor",
        "Searches your library and graveyard for a specific named card and puts it into your hand.",
    ),
    (
        "tutor-creature-god",
        "Searches your library for God creature cards.",
    ),
    (
        "quick-attach",
        "Lets you equip or attach at instant speed instead of only as a sorcery.",
    ),
    (
        "impulse-creature-dwarf",
        "Reveals cards from your library and lets you take Dwarf cards from among them.",
    ),
    (
        "seek-creature-dragon",
        "Seeks a Dragon card, putting one at random from your library into your hand.",
    ),
    (
        "seek-creature-elf",
        "Seeks an Elf card, putting one at random from your library into your hand.",
    ),
    (
        "reanimate-battle",
        "Returns a Battle card from your graveyard to the battlefield.",
    ),
    (
        "reanimate-historic",
        "Returns a historic permanent (artifact, legendary, or Saga) from your graveyard to play.",
    ),
    (
        "tutor-creature-myr",
        "Searches your library for a Myr card and puts it into your hand or onto the battlefield.",
    ),
    (
        "affinity-for-enchantments",
        "Costs 1 less to cast for each enchantment you control.",
    ),
    (
        "lockdown",
        "Keeps a permanent from untapping during its controller's untap step.",
    ),
    (
        "lockdown-nonland",
        "Keeps nonland permanents from untapping during their controllers' untap steps.",
    ),
    (
        "synergy-forage",
        "Rewards you for foraging, exiling three cards from your graveyard or sacrificing a Food.",
    ),
    (
        "synergy-freerunning",
        "Rewards you for casting Assassin spells or spells with freerunning.",
    ),
    (
        "tutor-creature-praetor",
        "Searches your library for a Praetor creature card and puts it into your hand.",
    ),
    (
        "typal-tiefling",
        "Rewards you for controlling Tiefling creatures, or boosts the Tieflings you control.",
    ),
    (
        "type-errata-specific-cat",
        "A creature whose type line was updated to include the Cat creature type.",
    ),
    (
        "tutor-creature-sliver",
        "Searches your library for a Sliver card and puts it into your hand.",
    ),
    (
        "synergy-horsemanship",
        "Cares about creatures with horsemanship, rewarding or protecting them.",
    ),
    (
        "synergy-hybrid",
        "Cares about hybrid mana symbols or spells cast with hybrid mana.",
    ),
    (
        "synergy-infect",
        "Rewards or strengthens creatures with infect.",
    ),
    (
        "hate-typal-rogue",
        "Punishes or fights back against Rogue creatures.",
    ),
    (
        "reanimate-spacecraft",
        "Returns a Spacecraft card from your graveyard to the battlefield.",
    ),
    (
        "synergy-investigate",
        "Triggers extra value whenever you investigate to create Clue tokens.",
    ),
    (
        "synergy-islandwalk",
        "Cares about islandwalk, which makes a creature unblockable while the defender controls an Island.",
    ),
    (
        "tutor-creature-wizard",
        "Searches your library for a Wizard card and puts it into your hand.",
    ),
    (
        "recursion-artifact",
        "Returns an artifact card from your graveyard to your hand.",
    ),
    (
        "synergy-junk",
        "Cares about Junk tokens, sacrificial artifacts that let you exile and play the top card of your library.",
    ),
    (
        "synergy-lander",
        "Cares about Lander tokens, sacrificial artifacts that search your library for a basic land.",
    ),
    (
        "gains-bushido",
        "Grants bushido, giving a creature +X/+X until end of turn whenever it blocks or becomes blocked.",
    ),
    (
        "block-when-tapped",
        "Lets tapped creatures block as though they were untapped, or restricts blocking to tapped creatures.",
    ),
    (
        "tutor-flash",
        "Searches your library for an instant or a card with flash and puts it into your hand.",
    ),
    (
        "seek-land-basic",
        "Randomly finds a basic land card from your library and puts it onto the battlefield.",
    ),
    ("tutor-green", "Searches your library for a green card."),
    (
        "gains-battle-cry",
        "Grants itself battle cry, so other attacking creatures get +1/+0 when it attacks.",
    ),
    (
        "synergy-life-payment",
        "Triggers or grows stronger whenever you pay life.",
    ),
    (
        "lockdown-planeswalker",
        "Taps down a planeswalker and stops it from untapping or using its abilities.",
    ),
    (
        "impulse-green",
        "Looks at cards off the top of your library and lets you take a green one.",
    ),
    (
        "banish-spell",
        "Exiles a target spell on the stack, then returns it when this permanent leaves the battlefield.",
    ),
    (
        "tutor-land-snow",
        "Searches your library for a snow land card.",
    ),
    (
        "synergy-manifest-dread",
        "Triggers or gains a bonus whenever you manifest dread.",
    ),
    (
        "tutor-land-town",
        "Searches your library for a basic land or Town card.",
    ),
    (
        "seek-land-mountain",
        "Finds a Mountain or matching land at random from your library, with no searching or revealing.",
    ),
    (
        "type-errata-vulture",
        "A creature card whose creature type was errata'd from Vulture to Bird.",
    ),
    (
        "creature-type-lycanthrope",
        "A creature card typed Werewolf.",
    ),
    (
        "tutor-red",
        "Searches your library for a red card and puts it into your hand or play.",
    ),
    (
        "tutor-rune",
        "Searches your library for a Rune card and puts it into your hand or onto the battlefield.",
    ),
    (
        "synergy-modular",
        "Cares about modular creatures and the +1/+1 counters modular grants.",
    ),
    (
        "synergy-morph",
        "Cares about face-down creatures and cards with morph or megamorph.",
    ),
    (
        "synergy-multicolor-trio",
        "Rewards you for casting or having spells that are exactly three colors.",
    ),
    (
        "gains-annihilator",
        "A creature that has or can gain annihilator, making the defending player sacrifice permanents when it attacks.",
    ),
    (
        "impulse-instant-sorcery-lesson",
        "Looks at the top of your library and puts a chosen card such as a Lesson into your hand.",
    ),
    (
        "typal-ape",
        "Cares about Apes or Monkeys, boosting or being boosted by creatures of that type.",
    ),
    (
        "typal-archer",
        "Cares about Archers, boosting or being boosted by creatures of that type.",
    ),
    (
        "synergy-p-t-sticker",
        "Cares about power and toughness stickers placed on your permanents.",
    ),
    (
        "regrowth-artifact-creature",
        "Returns an artifact creature card from your graveyard to your hand.",
    ),
    (
        "mana-source-type",
        "Adds mana when it resolves or is activated.",
    ),
    (
        "synergy-pin",
        "Cares about Pin tokens or pins you're wearing, rewarding you for having more of them.",
    ),
    (
        "gives-player-ability",
        "Grants a keyword ability to a player instead of a creature.",
    ),
    ("typal-barbarian", "Boosts or supports Barbarian creatures."),
    (
        "gives-exploit",
        "Grants exploit, letting a creature sacrifice a creature when it enters.",
    ),
    (
        "regrowth-legendary",
        "Returns a legendary card from a graveyard to hand.",
    ),
    (
        "buttcrew",
        "Crews Vehicles using toughness instead of power.",
    ),
    (
        "synergy-powerstone",
        "Rewards you for making or controlling Powerstone tokens.",
    ),
    (
        "synergy-pw-ajani",
        "Gets stronger or triggers an effect while you control an Ajani planeswalker.",
    ),
    (
        "gives-bloodthirst",
        "Grants bloodthirst, so a creature enters with +1/+1 counters if an opponent took damage this turn.",
    ),
    (
        "gives-evoke",
        "Grants evoke, letting a creature be cast for an alternate cost, then sacrificed as it enters.",
    ),
    (
        "shadow-counter",
        "Puts a shadow counter on a creature, so it can only block or be blocked by shadow.",
    ),
    (
        "gives-battle-cry",
        "Grants battle cry, so the creature pumps your other attackers when it attacks.",
    ),
    (
        "counters-remain",
        "Keeps its counters as it changes zones, other than to a hand or library.",
    ),
    (
        "synergy-pw-choose",
        "Lets you choose a planeswalker type for its effect to apply to.",
    ),
    (
        "regrowth-nonland-permanent",
        "Returns a nonland permanent card from your graveyard to your hand.",
    ),
    ("typal-crab", "Cares about or creates Crab creatures."),
    (
        "counter-fuel-shield",
        "Lets you remove a shield counter from a permanent to power an additional effect.",
    ),
    (
        "typal-demigod",
        "Rewards you for controlling a Demigod, tying its effect to that creature type.",
    ),
    (
        "gives-tantrum",
        "Grants tantrum, letting a blocking creature assign excess damage like trample.",
    ),
    (
        "hate-typal-ox",
        "Interacts specifically with Oxen, such as evading their blocks or punishing them.",
    ),
    (
        "gives-offspring",
        "Grants offspring, so paying extra makes a 1/1 token copy when the creature enters.",
    ),
    (
        "counter-fuel-loyalty",
        "Lets you remove a loyalty counter from a planeswalker to power an additional effect.",
    ),
    (
        "exchange-dice-roll",
        "Lets you swap a rolled die result for a number the card already has stored.",
    ),
    (
        "gives-ninjutsu",
        "Grants ninjutsu to creature cards in your hand.",
    ),
    (
        "typal-salamander",
        "Cares about or creates Salamander creatures.",
    ),
    (
        "shroud-from-white",
        "Can't be targeted by white spells or abilities from white sources.",
    ),
    (
        "emerge-from-artifact",
        "Lets you cast it by sacrificing an artifact, reducing the cost by that artifact's mana value.",
    ),
    (
        "gives-afterlife",
        "Grants afterlife, leaving a flying Spirit token behind when the creature dies.",
    ),
    (
        "typal-employee",
        "Cares about or creates Employee creatures.",
    ),
    ("typal-shark", "Cares about or creates Shark creatures."),
    (
        "un-set-mechanics",
        "Uses a joke mechanic that only appears in Un-sets.",
    ),
    ("typal-fox", "Cares about Foxes you control or cast."),
    (
        "removes-banding",
        "Strips banding and bands with other abilities from a target creature.",
    ),
    (
        "gives-training",
        "Grants training, so a creature gains a +1/+1 counter when it attacks with a stronger ally.",
    ),
    (
        "type-errata-specific-yeti",
        "A creature with the Yeti creature type.",
    ),
    (
        "gives-encore",
        "Grants encore to a creature card in your graveyard, so you can exile it for token copies that attack each opponent.",
    ),
    (
        "regrowth-vehicle",
        "Returns a creature or Vehicle card from your graveyard to your hand.",
    ),
    (
        "regrowth-nonland",
        "Returns a nonland card from your graveyard to your hand.",
    ),
    (
        "cycle-avr-r-miracle",
        "Has miracle: cast it for its miracle cost if it's the first card you draw this turn.",
    ),
    (
        "gains-infect",
        "Gains infect itself, dealing damage to creatures as -1/-1 counters and to players as poison counters.",
    ),
    (
        "copy-noncreature",
        "Lets you copy a noncreature spell you cast, often letting you choose new targets for the copy.",
    ),
    (
        "pseudo-intimidate",
        "Can't be blocked except by artifact creatures and creatures of one color.",
    ),
    (
        "cycle-avr-u-miracle",
        "A miracle cycle: a spell you may cast cheaply if it's the first card you draw this turn.",
    ),
    (
        "gives-firebending",
        "Grants firebending, letting an attacking creature add extra red mana until end of combat.",
    ),
    (
        "gives-foretell",
        "Grants foretell to cards in hand, letting you exile them face down and cast them later for a reduced cost.",
    ),
    (
        "typal-hamster",
        "A card that cares about or creates Hamster creatures.",
    ),
    (
        "prevent-extra-turns",
        "Stops a player from taking an extra turn, making them skip it instead.",
    ),
    (
        "typal-halfling",
        "A card that cares about or creates Halfling creatures.",
    ),
    (
        "tappable-enchantment",
        "An enchantment with a tap ability, letting it tap itself for an effect.",
    ),
    (
        "gives-high-flying",
        "Grants a creature flying, but restricts it to blocking only creatures with flying.",
    ),
    (
        "cycle-tmt-avatar-vertical",
        "One of a cycle of Avatar creatures, each pairing a keyword with matching triggered abilities.",
    ),
    (
        "cycle-blc-alt-commander",
        "One of a cycle of legendary creatures built as alternate commander options, each around a different strategy.",
    ),
    (
        "affinity-for-equipment",
        "Costs 1 less to cast for each Equipment you control.",
    ),
    (
        "old-mana-burn-clause",
        "Adds mana that doesn't empty as phases and steps end, dodging the old mana burn rule.",
    ),
    (
        "cycle-blc-face-commander",
        "One of a cycle of legendary creatures designed as a deck's primary commander, each with a distinct strategy.",
    ),
    (
        "gives-miracle",
        "Gives cards in your hand miracle, letting you cast them cheaply if drawn first this turn.",
    ),
    (
        "cycle-blc-talent",
        "A Class enchantment cycle that gains new abilities each time you level it up.",
    ),
    (
        "non-integer",
        "A card whose cost, stats, or text include a non-whole number, like a fraction or infinity.",
    ),
    (
        "gives-nimble",
        "Grants a creature the ability to dodge being blocked by creatures with power 3 or greater.",
    ),
    (
        "multiclass-party-member",
        "A creature that counts as a Cleric, Rogue, Warrior, and Wizard for your party.",
    ),
    (
        "multicast",
        "Lets you cast several spells for free from a set of exiled or revealed cards.",
    ),
    (
        "cycle-m21-basri-vertical",
        "Part of the M21 Basri cycle, built around putting +1/+1 counters on your creatures.",
    ),
    (
        "gives-phasing",
        "Gives a permanent phasing, so it phases out and in, treated as gone while phased out.",
    ),
    (
        "cycle-m21-chandra-vertical",
        "A card from M21's Chandra planeswalker deck cycle that rewards you for dealing noncombat damage to opponents.",
    ),
    (
        "marvel-storyline-name",
        "A card named after a Marvel Comics storyline or crossover event.",
    ),
    (
        "cycle-m21-garruk-vertical",
        "A card from M21's Garruk planeswalker deck cycle that rewards you for having big creatures with power 4 or greater.",
    ),
    (
        "cycle-m21-liliana-vertical",
        "A card from M21's Liliana planeswalker deck cycle that rewards you when your creatures die.",
    ),
    (
        "cycle-block-inv-counterspell",
        "An Invasion block counterspell that counters a spell and adds a bonus effect like life gain, damage, or life loss.",
    ),
    (
        "mana-leak-series",
        "Counters a spell unless its controller pays extra mana.",
    ),
    (
        "lockdown-permanent",
        "Keeps permanents from untapping during their controller's untap step.",
    ),
    (
        "cycle-m21-teferi-vertical",
        "A Teferi-themed blue card from the Core Set 2021 vertical cycle.",
    ),
    (
        "liliana-s-pact-demon",
        "A powerful flying Demon whose strong ability costs you life.",
    ),
    (
        "cycle-m3c-alt-commander",
        "An alternate commander option printed alongside a Modern Horizons 3 deck's face commander.",
    ),
    (
        "gives-replicate",
        "Grants replicate to spells you cast, letting you pay extra to cast copies alongside them.",
    ),
    (
        "gives-retrace",
        "Grants retrace to cards, letting you cast them from your graveyard by discarding a land.",
    ),
    (
        "gives-riot",
        "Grants riot to creatures, letting them enter with a +1/+1 counter or haste.",
    ),
    (
        "cycle-m3c-face-commander",
        "The headline commander featured on a Modern Horizons 3 Commander precon deck.",
    ),
    (
        "cycle-bng-enemy-ability",
        "One of a Born of the Gods cycle of creatures, each with a bonus ability tied to a splashed enemy color.",
    ),
    (
        "tutor-creature-goblin",
        "Searches your library for a Goblin card.",
    ),
    (
        "cycle-mh3-energy-reservedlist",
        "One of a Modern Horizons 3 cycle that remakes a Reserved List card using energy counters.",
    ),
    (
        "gives-split-second",
        "Gives spells split second, so players can't respond with spells or abilities while they're on the stack.",
    ),
    (
        "you-make-the-card",
        "A card created through a community vote in a You Make the Card event.",
    ),
    (
        "typal-flagbearer",
        "Forces opponents to target a Flagbearer you control when choosing targets, if able.",
    ),
    (
        "cycle-mkc-alt-commander",
        "One of a cycle of alternate versions of existing commanders, each with new abilities.",
    ),
    (
        "impulse-creature-warrior",
        "Digs through the top of your library for a Warrior card and puts it into your hand.",
    ),
    (
        "impulse-creature-human",
        "Digs through the top of your library for a Human card and puts it into your hand.",
    ),
    (
        "impulse-creature-hero",
        "Digs through the top of your library for a Hero card and puts it into your hand or onto the battlefield.",
    ),
    (
        "cycle-mkc-face-commander",
        "One of a cycle of Murders at Karlov Manor Detective legends built to lead your commander deck.",
    ),
    (
        "cost-reducer-planeswalker",
        "Reduces the cost of planeswalker spells you cast.",
    ),
    (
        "typal-fish",
        "Creates, buffs, or otherwise cares about Fish creatures you control.",
    ),
    (
        "gives-undying",
        "Grants undying to a creature, so it returns with a +1/+1 counter if it dies without one.",
    ),
    (
        "cycle-tmt-donatello-vertical",
        "One of a cycle of legendary Donatello creatures, each with a different artifact-themed ability.",
    ),
    (
        "cycle-rix-forerunner",
        "Tutors a card of its creature type to your library's top, then rewards more of that type entering.",
    ),
    (
        "hate-typal-sliver",
        "Removal or disruption aimed specifically at Sliver creatures.",
    ),
    (
        "hate-typal-share",
        "Affects every creature that shares a creature type with a key creature.",
    ),
    (
        "cycle-bro-mishra-vertical",
        "One of a cycle of legendary Mishra creatures, each with a distinct artifact or aggression ability.",
    ),
    (
        "hate-typal-orc",
        "Removal or disruption aimed specifically at Orc creatures.",
    ),
    (
        "cycle-rix-immortal-sun-purpose",
        "One of a cycle of creatures, each flavored around a different Ixalan faction.",
    ),
    (
        "banish-graveyard",
        "Exiles cards from a graveyard, often returning them if the source leaves the battlefield.",
    ),
    (
        "graveyard-fuel-permanent",
        "Exiles a permanent card from your graveyard to fuel an effect like copying it or gaining value.",
    ),
    (
        "hate-typal-non-human",
        "Punishes or restricts creatures that aren't Humans.",
    ),
    (
        "gunk",
        "Shuffles Gunk cards into opponents' libraries to clog their draws.",
    ),
    (
        "hate-typal-non-dragon",
        "Punishes or destroys creatures that aren't Dragons.",
    ),
    (
        "haste-counter",
        "Grants haste by putting a haste counter on a creature.",
    ),
    (
        "cycle-bro-urza-vertical",
        "A member of the vertical Urza cycle from The Brothers' War, including the meld pair.",
    ),
    (
        "typal-dalek",
        "Rewards you for controlling Daleks, often making tokens or granting them abilities.",
    ),
    ("hate-typal-god", "Punishes or removes God creatures."),
    (
        "hate-attraction",
        "Destroys or otherwise punishes Attraction cards.",
    ),
    (
        "hate-typal-beast",
        "Punishes or restricts Beast creatures, or rewards you when one enters.",
    ),
    (
        "flicker-land",
        "Exiles a land and returns it to the battlefield, retriggering its enter effects.",
    ),
    (
        "cycle-tmt-leonardo-vertical",
        "One of a cycle of Leonardo cards at different rarities or power levels within a set.",
    ),
    (
        "typal-construct",
        "Rewards you for controlling Construct creatures or helps you find and make them.",
    ),
    (
        "hate-shuffle",
        "Triggers an effect whenever a player shuffles their library.",
    ),
    (
        "hate-nonhand-cast",
        "Cares about or stops players casting spells from anywhere but their hand.",
    ),
    (
        "cycle-rix-typal-revealer",
        "Costs less to cast if you reveal a card of a given creature type from your hand.",
    ),
    (
        "hate-horsemanship",
        "Destroys, damages, or evades creatures that have horsemanship.",
    ),
    (
        "hate-enchantment-creature",
        "Destroys or otherwise punishes enchantment creatures.",
    ),
    (
        "hate-desert",
        "Cares about or interacts with Desert lands any player controls.",
    ),
    (
        "hate-creatureland",
        "Destroys, exiles, or defends against lands that are creatures.",
    ),
    (
        "cycle-tmt-michelangelo-vert",
        "A card named Michelangelo, one of several same-named versions with different abilities.",
    ),
    (
        "cycle-c17-eminence-commander",
        "A commander whose static or triggered ability works from the command zone as well as the battlefield.",
    ),
    (
        "cycle-c18-planeswalker",
        "A planeswalker legal to use as your commander.",
    ),
    (
        "cycle-c18-signature-spell",
        "A spell built to synergize directly with one specific planeswalker commander.",
    ),
    (
        "cycle-c18-sub-commander",
        "A legendary creature made to be legal as a commander despite an unusual drawback or built-in twist.",
    ),
    (
        "cycle-c19-face-commander",
        "The lead legendary creature designed as the primary commander of a preconstructed deck.",
    ),
    (
        "cycle-c19-planeswalker",
        "A planeswalker card reprinted in a preconstructed Commander deck.",
    ),
    (
        "cycle-c19-secondary-commander",
        "Marks a legendary creature from Commander 2019 meant to serve as a second, alternate commander for its precon deck.",
    ),
    (
        "cycle-c19-signature-spell",
        "A signature card from Commander 2019 designed to headline the strategy of its matching precon deck.",
    ),
    (
        "cycle-tmt-raphael-vertical",
        "One of several Raphael cards printed at different rarities, each with its own take on the character.",
    ),
    (
        "synergy-conjure",
        "Rewards you for conjuring, which creates new cards from outside the game into your hand or library.",
    ),
    (
        "cycle-cardinal-paladin",
        "One of a cycle of Paladins that taps to destroy a creature or permanent of a specific color.",
    ),
    (
        "cycle-clb-alt-commander",
        "A legendary creature from Baldur's Gate that can be your commander alongside a Background card.",
    ),
    (
        "deprecated-wall-restriction",
        "Changes a creature's type to anything except Wall, dating from before defender, when Walls couldn't attack.",
    ),
    (
        "cycle-tor-possessed-creature",
        "A creature that at threshold turns black and can destroy a creature of a specific color.",
    ),
    (
        "cycle-clb-face-commander",
        "A legendary creature built to lead its own Commander Legends: Baldur's Gate precon deck.",
    ),
    (
        "synergy-clash",
        "Triggers or rewards you for clashing, revealing top library cards to compare mana values.",
    ),
    (
        "bid",
        "Lets players bid life against each other, with the high bidder paying that life for an effect.",
    ),
    (
        "cycle-clb-precon-background",
        "A Background enchantment that grants your commander creatures an extra ability.",
    ),
    (
        "cycle-cmm-alt-commander",
        "A legendary creature printed as an alternate commander option in a Commander Masters precon deck.",
    ),
    (
        "cycle-cmm-face-commander",
        "A legendary creature or planeswalker built to lead its own Commander Masters precon deck.",
    ),
    (
        "tutor-creature-elf",
        "Searches your library for an Elf creature card.",
    ),
    (
        "typal-time-lord",
        "Rewards you for controlling Time Lord creatures.",
    ),
    (
        "type-errata-tiger",
        "A creature card whose type line was errata'd to Cat.",
    ),
    (
        "cycle-con-forward-synergy",
        "Has a bonus effect if you control a permanent of a specific other color.",
    ),
    (
        "typal-thrull",
        "Rewards you for controlling or sacrificing Thrull creatures.",
    ),
    (
        "cycle-tor-tainted-land",
        "A land that taps for colorless, or for one of two colors if you control a Swamp.",
    ),
    (
        "typal-reveler",
        "Rewards you for controlling Reveler creatures.",
    ),
    (
        "cycle-dka-m-monster",
        "A mythic creature from Dark Ascension's cycle of powerful standalone monsters.",
    ),
    (
        "cycle-dka-monster-lord",
        "A creature that boosts other creatures of its own tribe, from Dark Ascension's lord cycle.",
    ),
    (
        "synergy-phasing",
        "Cares about permanents phasing in or out.",
    ),
    (
        "tutor-creature-dinosaur",
        "Searches your library for a Dinosaur card.",
    ),
    (
        "cycle-dsc-alt-commander",
        "A secondary legendary creature built as an alternate commander for its preconstructed deck.",
    ),
    (
        "cycle-dsc-face-commander",
        "The primary legendary creature built to lead its preconstructed Commander deck.",
    ),
    (
        "cycle-eld-brawler",
        "A multicolor legendary creature from Throne of Eldraine designed as a Brawl commander.",
    ),
    (
        "cycle-elemental-servant",
        "One of a cycle of Elemental creatures, each with a distinct ability tied to its element.",
    ),
    ("tutor-black", "Searches your library for a black card."),
    (
        "buttfight",
        "Deals damage equal to a creature's toughness rather than its power.",
    ),
    (
        "typal-pest",
        "Cares about or boosts Pest creatures you control.",
    ),
    (
        "cycle-fic-alt-commander",
        "One of a cycle of Final Fantasy legendary creatures built as an alternate commander option.",
    ),
    (
        "cycle-fic-face-commander",
        "One of a cycle of Final Fantasy protagonist legendary creatures built to lead your deck as commander.",
    ),
    (
        "typal-pegasus",
        "Cares about or boosts Pegasus creatures you control.",
    ),
    (
        "wheel-symmetrical-optional",
        "Lets each player choose whether to discard their hand and draw a fresh one.",
    ),
    (
        "tutor-artifact-vehicle",
        "Searches your library for a Vehicle card.",
    ),
    (
        "cover-card",
        "A reminder card used to cover a face-down creature at the table.",
    ),
    (
        "collector-number-matters",
        "Cares about the collector numbers printed on cards, such as odd, even, or lowest.",
    ),
    ("synergy-omen", "Rewards you for casting Omen spells."),
    (
        "tutor-land-cave",
        "Searches your library for a Cave card, or sometimes a basic land.",
    ),
    (
        "divides-battlefield",
        "Splits creatures on the battlefield into separate sides that often can't block each other.",
    ),
    (
        "cycle-apc-bloodfire-vertical",
        "A creature you sacrifice to deal damage to each creature.",
    ),
    (
        "type-errata-specific-wizard",
        "A creature retroactively given the Wizard creature type through errata.",
    ),
    (
        "cycle-ulg-vertical-phyrexian",
        "A creature you sacrifice to weaken a target creature until end of turn.",
    ),
    (
        "cycle-iko-u-mutate",
        "A creature with mutate that triggers a bonus effect whenever it mutates.",
    ),
    (
        "cycle-un-psychographics",
        "A legendary Gamer creature whose ability reflects a real style of Magic player.",
    ),
    (
        "eat-food",
        "Cares about eating actual food or Food tokens, rewarding you while you're eating.",
    ),
    (
        "synergy-library-cast",
        "Lets you cast spells from the top of your library, or rewards casting from there.",
    ),
    (
        "temporary-counter",
        "Adds +1/+1 or -1/-1 counters that are removed again at a later step or end of turn.",
    ),
    (
        "confluence",
        "A modal spell where you choose three effects from a list, and can pick the same one more than once.",
    ),
    (
        "cycle-unk-vegas",
        "Each upkeep lets you pick a mode from a list, never repeating one already chosen.",
    ),
    (
        "cycle-unspeakable",
        "One of a cycle of Arcane instants that, cast in sequence, put The Unspeakable onto the battlefield.",
    ),
    (
        "sword-of-x-and-y",
        "An Equipment that boosts power and toughness, grants protection from two things, and triggers a bonus on combat damage.",
    ),
    (
        "synergy-pw-garruk",
        "A card that gets stronger or gains value from having a Garruk planeswalker on your side.",
    ),
    (
        "cycle-stormscape-vertical",
        "One of a cycle of blue creatures whose kicker or activated abilities splash into white and black.",
    ),
    (
        "cycle-otc-alt-commander",
        "A legendary creature offered as an alternate commander for its Commander deck.",
    ),
    (
        "cycle-otc-face-commander",
        "A legendary creature designed as the face commander for its Commander deck.",
    ),
    (
        "synergy-fight",
        "Rewards you when your creatures fight, boosting them or drawing cards.",
    ),
    (
        "cycle-msc-f4-spell",
        "A spell that costs less to cast when a certain condition is met.",
    ),
    (
        "typal-eldrazi-spawn",
        "Cares about Eldrazi Spawn tokens you control or create.",
    ),
    (
        "cycle-inv-mutation",
        "Destroys or bounces a permanent, then makes Saproling tokens equal to its mana value.",
    ),
    (
        "cycle-msc-fantastic-four",
        "A legendary Hero creature that triggers a bonus in combat if you've cast a noncreature spell that turn.",
    ),
    (
        "synergy-fear",
        "Grants or copies the fear keyword across your creatures.",
    ),
    (
        "cycle-pc2-legend",
        "A cycle of powerful legendary creatures built to lead their own multicolor deck.",
    ),
    (
        "cycle-znr-expeditioner",
        "A cycle of creatures that gain a bonus ability while you control another of their type.",
    ),
    (
        "cycle-afc-alt-commander",
        "A cycle of legendary creatures designed as alternate commanders for the deck.",
    ),
    (
        "cycle-msh-fantastic-four",
        "A cycle of Fantastic Four Hero creatures that reward drawing cards, tokens, or counters.",
    ),
    (
        "synergy-exploit",
        "Cares about the exploit mechanic, letting you sacrifice a creature as it enters for a payoff.",
    ),
    (
        "extra-draw-step",
        "Gives you an additional untap, upkeep, or draw step during your turn.",
    ),
    (
        "cycle-afc-face-commander",
        "A legendary creature built as the headline commander of an Adventures precon deck.",
    ),
    (
        "cycle-wwk-quest",
        "An enchantment that gains quest counters from repeated triggers, unlocking a payoff once it has enough.",
    ),
    (
        "unstable-killbot",
        "A vanilla artifact creature sharing a collector number with identical killbots of other names.",
    ),
    (
        "cycle-znr-relic",
        "A colorless Relic artifact from a Zendikar Rising cycle.",
    ),
    (
        "typal-chimera",
        "A Chimera you can sacrifice to permanently grant a keyword and a +2/+2 counter to another Chimera.",
    ),
    (
        "cycle-pip-alt-commander",
        "A legendary creature offered as an alternate commander in a Fallout precon deck.",
    ),
    (
        "cycle-sunscape-vertical",
        "One of a matched Apprentice, Familiar, Battlemage, or Master cycle with green and blue abilities.",
    ),
    (
        "cycle-pip-face-commander",
        "One of a cycle of Fallout legendary creatures built to lead your deck as commander.",
    ),
    (
        "flagbearer",
        "Forces opponents to aim at least one target at a Flagbearer when they cast spells or activate abilities.",
    ),
    (
        "cycle-vis-chimera",
        "One of a cycle of Chimeras you can sacrifice to give another Chimera a permanent +2/+2 counter and its keyword.",
    ),
    (
        "typal-each",
        "Scales its effect with the number of distinct creature types among creatures in play.",
    ),
    (
        "cycle-40k-alt-commander",
        "One of a cycle of Warhammer 40,000 legendary creatures built to lead your deck as commander.",
    ),
    (
        "cycle-khm-pathway",
        "One of the Kaldheim Pathway lands: choose which of two colors it enters as when you play it.",
    ),
    (
        "type-errata-chicken",
        "A creature once printed as a Chicken, now errata'd to Bird, usually with silly or dice-based abilities.",
    ),
    (
        "cycle-plc-extortion-vertical",
        "Lets any player pay a steep cost to counter the spell as you cast it.",
    ),
    (
        "synergy-connive",
        "Cares about your creatures conniving, rewarding you when they draw, discard, and grow.",
    ),
    (
        "cycle-xln-m-legend",
        "A legendary Ixalan creature that headlines one of the set's four tribes.",
    ),
    (
        "cycle-plc-rescue-vertical",
        "Returns one or more of your creatures to hand when it enters, letting you reuse their abilities.",
    ),
    (
        "typal-kobold",
        "Cares about Kobold creatures, typically boosting or granting abilities to other Kobolds you control.",
    ),
    (
        "cycle-nightscape-vertical",
        "A black creature that spends blue and red mana for extra effects.",
    ),
    (
        "cycle-40k-face-commander",
        "A powerful legendary creature built to headline a Warhammer 40,000 Commander deck.",
    ),
    (
        "type-errata-specific-spirit",
        "An older card whose creature type was errata'd specifically to Spirit.",
    ),
    (
        "cycle-40k-saga",
        "A Warhammer 40,000 Saga that resolves three chapter abilities, then is sacrificed.",
    ),
    (
        "deathtouch-to-planeswalkers",
        "Destroys a planeswalker whenever a creature with deathtouch deals it damage.",
    ),
    (
        "cycle-5dn-station",
        "A Station artifact that taps for an effect and untaps itself on a trigger.",
    ),
    (
        "square-stats-matter",
        "Rewards or checks for creatures whose power equals their toughness.",
    ),
    (
        "cycle-thornscape-vertical",
        "A green creature from Invasion block with red and white activated abilities.",
    ),
    (
        "cycle-jud-gorger-vertical",
        "A creature from the Judgment cycle with a costly enter effect that reverses when it leaves.",
    ),
    (
        "activate-from-stack",
        "Has an ability you can activate while the card is still on the stack.",
    ),
    (
        "affinity-for-devotion",
        "Costs less to cast based on your devotion to a color.",
    ),
    (
        "age-matters",
        "Its cost or effect scales with your real-life age or years playing Magic.",
    ),
    (
        "alternate-cost-gain-life",
        "Lets you cast it for free by having your opponents gain life instead of paying mana.",
    ),
    (
        "animate-battle",
        "Temporarily turns a permanent you control into a creature during combat.",
    ),
    (
        "animate-instant",
        "Turns an instant card into a creature on the battlefield.",
    ),
    (
        "animate-sorcery",
        "Turns a sorcery card into a creature on the battlefield.",
    ),
    (
        "any-zone-color-change",
        "Changes the color of cards and permanents across every zone.",
    ),
    (
        "arena-effect",
        "Makes a creature you control and one an opponent controls fight each other.",
    ),
    (
        "board-reset",
        "Resets the whole game, clearing permanents and hands and setting life totals.",
    ),
    (
        "cost-reducer-legendary",
        "Reduces the cost of legendary spells you cast.",
    ),
    (
        "counter-fuel-time",
        "Stores or spends time counters as fuel for an effect.",
    ),
    (
        "counterspell-aura",
        "Counters an Aura spell, and sometimes other spell types too.",
    ),
    (
        "creature-type-bodyguard",
        "A creature that redirects damage that would hit you onto itself.",
    ),
    (
        "creature-type-enchantress",
        "Marks an Enchantress-style creature that typically draws you a card when you cast an enchantment.",
    ),
    (
        "creature-type-hero",
        "Marks a creature of the original pre-2007 Hero creature type.",
    ),
    (
        "creature-type-monster",
        "Marks a monstrous creature such as a Kraken, Manticore, or Leviathan.",
    ),
    (
        "creature-type-undead",
        "Marks an undead creature such as a Zombie, Skeleton, or Wraith.",
    ),
    (
        "cycle-ala-esper-capsule",
        "An Esper cycle of artifacts you tap and sacrifice for a one-shot effect like drawing cards or destroying a permanent.",
    ),
    (
        "cycle-ala-jund-scavenger",
        "A Jund cycle of creatures that get a +1/+1 counter whenever another creature dies.",
    ),
    (
        "cycle-ala-naya-cycler",
        "A Naya cycle of creatures with cycling, letting you discard them to draw a card.",
    ),
    (
        "cycle-ala-naya-druid",
        "A mana dork that gives a bonus if you control a creature with power 5 or greater.",
    ),
    (
        "cycle-ala-naya-soul-spell",
        "An effect that scales with a creature's power.",
    ),
    (
        "cycle-apc-penumbra-vertical",
        "Creates a black creature token when this creature dies.",
    ),
    (
        "cycle-apc-phyrexian-vertical",
        "Draws you a card at the cost of losing life.",
    ),
    (
        "cycle-apc-whirlpool-vertical",
        "Shuffles your hand into your library, then draws that many cards.",
    ),
    (
        "cycle-arch-charm",
        "A modal instant with three modes that costs three mana of a single color.",
    ),
    (
        "cycle-bok-glasskite-vertical",
        "A flier that counters the first spell or ability that targets it each turn.",
    ),
    (
        "cycle-bro-meld",
        "A large permanent formed when two cards meld together into one.",
    ),
    (
        "cycle-bro-tron-worker",
        "An artifact creature that gets a bonus when you control all three named Workers.",
    ),
    (
        "cycle-clb-god",
        "A God that's indestructible while at half starting life or less and triggers when your other creatures die.",
    ),
    (
        "cycle-clb-orb-of-dragonkind",
        "A mana rock whose mana grants a bonus when spent to cast a Dragon.",
    ),
    (
        "cycle-con-esper-artifact-boost",
        "An artifact creature that gets bigger or better based on the artifacts you have.",
    ),
    (
        "cycle-con-esper-scepter",
        "An artifact with a tap ability that taps, draws, or discards depending on its color.",
    ),
    (
        "cycle-con-wubrg-ability",
        "A creature with a five-color activated ability that pumps it and grants new abilities.",
    ),
    (
        "cycle-csp-rimewind-vertical",
        "A wizard with a tap ability usable only if you control four or more snow permanents.",
    ),
    (
        "cycle-dft-raceway",
        "A land that taps for colorless mana and gains a bonus effect once you reach max speed.",
    ),
    (
        "cycle-emn-escalate-borrowed",
        "A modal spell with escalate: pay extra mana to choose more than one of its effects.",
    ),
    (
        "cycle-emn-escalate-collective",
        "A modal spell with escalate: pay an extra cost per mode to choose more than one effect.",
    ),
    (
        "cycle-frf-manifest-form",
        "An enchantment that manifests your top library card and attaches itself to it as an aura.",
    ),
    (
        "cycle-fut-noncreature-morph",
        "A noncreature permanent with morph, letting you also cast it face down as a 2/2 creature.",
    ),
    (
        "cycle-hou-god",
        "A legendary God creature that returns to your hand instead of staying in the graveyard when it dies.",
    ),
    (
        "activate-from-command-zone",
        "Lets you put this permanent onto the battlefield straight from the command zone by paying a cost.",
    ),
    (
        "cycle-hou-modal-spell",
        "A spell that lets you choose one of two or more listed effects when you cast it.",
    ),
    (
        "cycle-hou-torment",
        "Makes a player lose life unless they sacrifice a nonland permanent or discard a card.",
    ),
    (
        "cycle-inr-delver",
        "A creature that transforms into a stronger form once a condition is met, Delver style.",
    ),
    (
        "cycle-kld-module",
        "An artifact that works with energy and +1/+1 counters to grow your board.",
    ),
    (
        "cycle-lea-red-escalate",
        "A red creature in an early cycle that shares stats but gains more abilities as rarity rises.",
    ),
    (
        "cycle-lrw-typal-land",
        "A land that enters untapped if you reveal a card of its chosen creature type, and helps cast that type.",
    ),
    (
        "cycle-lrw-velis-vel",
        "A changeling instant that gives creatures all creature types plus a stat or keyword boost.",
    ),
    (
        "cycle-m12-artifact-of-empires",
        "An artifact whose ability grows far stronger when you also control its two matching companion artifacts.",
    ),
    (
        "cycle-m12-phantasmal",
        "A creature that sacrifices itself the moment it becomes the target of a spell or ability.",
    ),
    (
        "cycle-m19-bolas-reign",
        "A high-impact mono-colored spell from the Nicol Bolas cycle in Core Set 2019.",
    ),
    (
        "cycle-m20-chandra",
        "One of the Chandra planeswalkers printed in Magic 2020, each with fire-based loyalty abilities.",
    ),
    (
        "cycle-m3c-lhurgoyf",
        "A Lhurgoyf whose power and toughness scale with the number of card types in all graveyards.",
    ),
    (
        "cycle-mid-black-werewolf",
        "A black double-faced Human Werewolf that transforms to gain new abilities on its back face.",
    ),
    (
        "cycle-mmq-flailing-creature",
        "A creature any player can pay to give +1/+1 or -1/-1 until end of turn.",
    ),
    (
        "cycle-mmq-rishadan-pirate",
        "Forces each opponent to sacrifice a permanent unless they pay a tax.",
    ),
    (
        "cycle-msc-face-commander",
        "A legendary creature built to lead and reward a specific tribe or theme.",
    ),
    (
        "cycle-nem-parallax",
        "An enchantment with fade counters you spend to exile a target, returning it when the enchantment leaves.",
    ),
    (
        "cycle-nem-recruiter",
        "Pays mana and taps to put a permanent of its set creature type from your library onto the battlefield.",
    ),
    (
        "hate-color",
        "Punishes or defends against opponents playing one or more specific colors.",
    ),
    (
        "hate-typal-dalek",
        "Targets, destroys, or otherwise punishes Dalek creatures.",
    ),
    (
        "cycling-non-mana",
        "Has a cycling cost paid with something other than mana, like life or a sacrifice.",
    ),
    (
        "hate-typal-dinosaur",
        "Targets, destroys, or otherwise punishes Dinosaur creatures.",
    ),
    (
        "hate-typal-djinn",
        "Targets, destroys, or otherwise punishes Djinn creatures.",
    ),
    (
        "gains-toxic",
        "Gives itself toxic, adding poison counters when it deals combat damage.",
    ),
    (
        "hate-typal-efreet",
        "Destroys or otherwise punishes Djinn and Efreet creatures.",
    ),
    (
        "hate-typal-eldrazi",
        "Protects against or punishes Eldrazi creatures.",
    ),
    (
        "gives-mobilize",
        "Grants mobilize, creating temporary attacking creature tokens when it attacks.",
    ),
    (
        "cost-reducer-historic",
        "Makes your historic spells (artifacts, legendaries, and Sagas) cost less to cast.",
    ),
    (
        "substitute-card",
        "A placeholder card used to represent a double-faced card in your deck.",
    ),
    (
        "cost-reducer-instant",
        "Makes your instant spells cost less to cast.",
    ),
    (
        "impulse-creature-legendary",
        "Digs through the top of your library to find a legendary creature card and take it.",
    ),
    (
        "gains-vanishing",
        "Gains vanishing itself, entering with time counters and sacrificed when the last is removed.",
    ),
    (
        "synergy-backup",
        "Triggers or copies effects whenever one of your creatures becomes the target of a backup ability.",
    ),
    (
        "hate-double-strike",
        "Gains double strike and other keywords while a creature card with them sits in a graveyard or exile.",
    ),
    (
        "conjure-battle",
        "Conjures a duplicate of a permanent into your hand.",
    ),
    (
        "animate-token",
        "Turns a token you control into a creature with new power, toughness, or abilities.",
    ),
    (
        "hate-flashback",
        "Exiles cards with flashback from graveyards or taxes opponents' flashback costs.",
    ),
    (
        "impulse-creature-soldier",
        "Reveals cards off the top of your library and puts any Soldiers found into your hand.",
    ),
    (
        "impulse-creature-turtle",
        "Lets you dig through the top cards of your library to grab a Turtle or land card.",
    ),
    (
        "hate-typal-knight",
        "Punishes or targets Knights, such as destroying them or being unblockable by them.",
    ),
    (
        "synergy-class",
        "A Class enchantment that gains new abilities as you pay to level it up.",
    ),
    (
        "hate-food",
        "Punishes making Food or turns Food against its owner.",
    ),
    (
        "hate-typal-mutant",
        "Protects against Mutants or turns creatures into Mutant tokens.",
    ),
    (
        "synergy-cumulative-upkeep",
        "Cares about cumulative upkeep, either carrying the cost or helping you pay it.",
    ),
    (
        "bounceland",
        "A land that enters tapped and taps for two mana, but makes you return a land to hand when it enters.",
    ),
    (
        "impulse-pw-tyvar",
        "Looks at the top cards of your library and lets you grab an Elf, Warrior, or Tyvar card.",
    ),
    (
        "synergy-devoid",
        "Cares about devoid, the ability that makes an otherwise colored card colorless.",
    ),
    (
        "hate-haste",
        "Gains haste, among other keywords, while a creature with haste sits in a graveyard or exile.",
    ),
    (
        "synergy-discover",
        "Triggers an extra effect or deals damage whenever you discover cards.",
    ),
    (
        "conjure-planeswalker",
        "Conjures a duplicate of a permanent into your hand.",
    ),
    (
        "synergy-doctor-s-companion",
        "Cares about Doctor cards and cards that have doctor's companion.",
    ),
    (
        "cycle-cmr-face-commander",
        "A legendary creature built as the face commander of a Commander Legends precon deck.",
    ),
    (
        "hate-typal-non-elemental",
        "Damages, bounces, or otherwise punishes creatures that aren't Elementals.",
    ),
    (
        "hate-library-cast",
        "Stops players from casting spells directly out of a library.",
    ),
    (
        "hate-typal-non-giant",
        "Damages, destroys, or otherwise punishes creatures that aren't Giants.",
    ),
    (
        "off-turn-attack",
        "Lets you attack during an opponent's turn instead of only your own.",
    ),
    (
        "ownership-change-in-hand",
        "Puts a card into a hand belonging to a player who isn't its owner.",
    ),
    (
        "plainsfall",
        "Triggers an added effect whenever a Plains enters the battlefield under your control.",
    ),
    (
        "prepare-matters",
        "Cares about creatures being prepared, a status only prepare creatures can gain.",
    ),
    (
        "prevent-transform",
        "Stops a creature from transforming into its other face.",
    ),
    (
        "cycle-drc-alt-commander",
        "A legendary creature offered as an alternate commander choice in a themed cycle.",
    ),
    (
        "hate-typal-non-gorgon",
        "Forces an opponent to sacrifice or lose a creature that isn't a Gorgon.",
    ),
    (
        "gives-devoid",
        "Grants devoid to your creatures, making them colorless no matter their mana cost.",
    ),
    (
        "pseudo-legendary",
        "Isn't typed legendary but still prevents multiple copies of it from being in play at once.",
    ),
    (
        "hate-typal-non-kraken",
        "Bounces every creature except Krakens, Leviathans, and similar sea creatures.",
    ),
    (
        "punchcard",
        "A tracking permanent that marks off steps or boxes instead of using ordinary counters.",
    ),
    (
        "hate-typal-non-leviathan",
        "Bounces, destroys, or damages creatures that aren't Leviathans.",
    ),
    (
        "hate-typal-non-merfolk",
        "Bounces, destroys, or damages creatures that aren't Merfolk.",
    ),
    (
        "hate-typal-non-octopus",
        "Bounces, destroys, or damages creatures that aren't Octopuses.",
    ),
    (
        "hate-typal-non-pirate",
        "Bounces, destroys, or damages creatures that aren't Pirates.",
    ),
    (
        "hate-typal-non-rat",
        "Bounces, destroys, or damages creatures that aren't Rats.",
    ),
    (
        "hate-typal-non-serpent",
        "Bounces, destroys, or damages creatures that aren't Serpents.",
    ),
    (
        "cycle-drc-face-commander",
        "A legendary creature meant to be the face commander of its preconstructed deck.",
    ),
    (
        "hate-morph",
        "Punishes morph, turning face-down creatures up or making morph costs cost more.",
    ),
    (
        "hate-nonartifact",
        "Makes nonartifact spells cost more or restricts players from casting them.",
    ),
    (
        "cycle-emn-escalate-alliance",
        "An escalate instant letting you pay extra to combine multiple modes in one spell.",
    ),
    (
        "cycle-grn-precon-tutor",
        "A removal or bounce spell that also lets you search for a specific named legend.",
    ),
    (
        "typal-gorgon",
        "Cares about or rewards you for controlling Gorgon creatures.",
    ),
    (
        "typal-gremlin",
        "Cares about or rewards you for controlling Gremlin creatures.",
    ),
    (
        "cycle-infinity-stone",
        "An indestructible legendary artifact that taps for mana and can be harnessed for a powerful ongoing ability.",
    ),
    (
        "gives-embalm",
        "Grants embalm to a creature card in a graveyard, so you can exile it for a white Zombie copy.",
    ),
    (
        "removes-protection",
        "Strips protection from a permanent, making it targetable, blockable, or damageable again.",
    ),
    (
        "removes-shadow",
        "Strips shadow from a creature, letting it block or be blocked normally.",
    ),
    (
        "typal-hydra",
        "Cares about or boosts Hydra creatures you control.",
    ),
    (
        "removes-shroud",
        "Strips shroud from a permanent, making it a legal target for spells and abilities again.",
    ),
    (
        "hate-typal-rat",
        "Punishes or shuts down Rat creatures and Rat-based strategies.",
    ),
    (
        "synergy-pw-teferi",
        "Rewards you for controlling a Teferi planeswalker.",
    ),
    (
        "synergy-pw-tezzeret",
        "Gets stronger or gains abilities when you control a Tezzeret planeswalker.",
    ),
    ("typal-kraken", "Cares about or creates Kraken creatures."),
    ("typal-leech", "Cares about or creates Leech creatures."),
    (
        "hate-typal-rebel",
        "Punishes or shuts down Rebel creatures.",
    ),
    (
        "synergy-pw-vivien",
        "Gets stronger or gains abilities when you control a Vivien planeswalker.",
    ),
    (
        "synergy-pw-vraska",
        "Triggers a bonus effect when you control a Vraska planeswalker.",
    ),
    ("typal-moogle", "Cares about or creates Moogle creatures."),
    (
        "hate-typal-reflection",
        "Destroys Reflection tokens to punish decks built around them.",
    ),
    (
        "synergy-pw-yanling",
        "Gets stronger or gains abilities as long as you control a Yanling planeswalker.",
    ),
    (
        "hate-reach",
        "Gains reach itself while a creature with reach is in a graveyard or exile.",
    ),
    (
        "restart-game",
        "Restarts the game, sometimes leaving certain permanents on the battlefield.",
    ),
    (
        "typal-non-angel",
        "Targets or affects a creature you control that isn't an Angel.",
    ),
    (
        "synergy-rune",
        "Searches for, discounts, or otherwise supports Rune cards in your deck.",
    ),
    (
        "synergy-skulk",
        "Grants or cares about skulk, keeping a creature from being blocked by higher-power creatures.",
    ),
    (
        "color-ward",
        "Grants a creature protection from one or more colors.",
    ),
    (
        "synergy-soulbond",
        "Cares about being paired with another creature through soulbond.",
    ),
    (
        "restock-battle",
        "Puts a battle card from your graveyard on top of your library.",
    ),
    (
        "restock-enchantment",
        "Puts an enchantment card from your graveyard on top of your library.",
    ),
    (
        "cycle-khc-face-commander",
        "A featured legendary creature designed as the face commander of its Kaldheim Commander deck.",
    ),
    (
        "synergy-teamwork",
        "Cares about casting spells or paying costs using the teamwork mechanic.",
    ),
    ("typal-orc", "Cares about Orc creatures."),
    (
        "typal-ox",
        "Cares about or boosts creatures with the Ox creature type.",
    ),
    (
        "synergy-tuck",
        "Cares about cards being put into a library from anywhere.",
    ),
    (
        "gains-mountainwalk",
        "Gains mountainwalk itself, unblockable while the defender controls a Mountain.",
    ),
    (
        "graveyard-fuel-historic",
        "Uses historic cards in your graveyard, artifacts, legendaries, and Sagas, as fuel.",
    ),
    (
        "graveyard-fuel-legendary",
        "Uses legendary cards in your graveyard as fuel, often by exiling them.",
    ),
    (
        "synergy-wastes",
        "Cares about lands named Wastes that you control.",
    ),
    (
        "roll-to-visit",
        "Lets you roll to visit your Attractions outside your normal main phase roll.",
    ),
    (
        "gains-myriad",
        "Gains myriad, creating a tapped attacking copy for each opponent other than the one it attacks.",
    ),
    (
        "tap-outlet",
        "Has an ability whose cost includes tapping another permanent you control.",
    ),
    (
        "cycle-young-planeswalker",
        "A creature from the 'Young' mage cycle with a spellcasting or graveyard payoff.",
    ),
    (
        "typal-scarecrow",
        "Cares about or supports Scarecrow creatures you control.",
    ),
    (
        "hate-typal-treefolk",
        "Punishes or specifically targets Treefolk creatures and Forests.",
    ),
    (
        "tearing",
        "Has you physically tear the card apart as part of using its effect.",
    ),
    (
        "hate-artifact-land",
        "Grants unblockability against players who control an artifact land.",
    ),
    (
        "hate-transform",
        "Prevents a creature from transforming, often alongside a lock on attacking, blocking, or activated abilities.",
    ),
    (
        "typal-servo",
        "Cares about or supports Servo tokens you control.",
    ),
    (
        "gains-prowess",
        "Gains prowess itself, getting +1/+1 whenever you cast a noncreature spell.",
    ),
    (
        "transcendental-life-damage",
        "Deals damage or changes life totals in transcendental numbers like π.",
    ),
    (
        "triggers-at-cleanup-step",
        "Triggers an effect at the beginning of the next cleanup step instead of the end step.",
    ),
    (
        "hate-treasure",
        "Punishes or disrupts opponents for creating or spending mana from Treasure tokens.",
    ),
    (
        "typal-sphinx",
        "Cares about or supports Sphinx creatures you control.",
    ),
    (
        "hate-untapped",
        "Removal that destroys or damages untapped creatures.",
    ),
    (
        "tutor-artifact-creature",
        "Searches your library for an artifact creature card and puts it into your hand or onto the battlefield.",
    ),
    (
        "hate-vigilance",
        "A creature that gains vigilance while a creature with vigilance is in a graveyard or exile.",
    ),
    (
        "tutor-artifact-legendary",
        "Searches your library for a legendary artifact card and puts it onto the battlefield.",
    ),
    (
        "gains-rampage",
        "A creature that gains rampage, growing when blocked by more than one creature.",
    ),
    (
        "hate-typal-assassin",
        "Punishes or targets Assassin creatures specifically.",
    ),
    (
        "tutor-aura-curse",
        "Searches your library for a Curse Aura and puts it into play attached to a player.",
    ),
    (
        "typal-wurm",
        "Boosts or supports Wurm creatures you control.",
    ),
    (
        "type-addition-rabbit",
        "A card whose creature type includes Rabbit.",
    ),
    (
        "type-errata-mammoth",
        "A card whose creature type was changed to Elephant instead of Mammoth.",
    ),
    (
        "type-errata-roc",
        "A card whose creature type is Bird instead of Roc.",
    ),
    ("tutor-blue", "Searches your library for a blue card."),
    (
        "tutor-creature-ally",
        "Searches your library for an Ally creature card.",
    ),
    (
        "hunter-trigger",
        "Triggers a bonus effect whenever a creature it dealt damage to this turn dies.",
    ),
    (
        "impulse-artifact-creature",
        "Exiles cards from your library and lets you cast an artifact creature or Vehicle from among them.",
    ),
    (
        "cycle-znc-face-commander",
        "A face commander from the Zendikar Rising Commander deck cycle.",
    ),
    (
        "hate-typal-bird",
        "Punishes or interacts negatively with Bird creatures, such as granting protection from them.",
    ),
    (
        "hate-typal-dog",
        "Punishes or interacts negatively with Dog creatures, such as granting protection from them.",
    ),
    (
        "hate-typal-eldrazi-scion",
        "Punishes or interacts negatively with Eldrazi Scion creatures, such as being unblockable by them.",
    ),
    (
        "hate-typal-elk",
        "Punishes or interacts negatively with Elk creatures, such as granting protection from them.",
    ),
    (
        "hate-typal-giant",
        "Reacts whenever any player casts a Giant spell.",
    ),
    (
        "hate-typal-glimmer",
        "Hinders Glimmer creatures, such as being unblockable by them.",
    ),
    (
        "hate-typal-gorgon",
        "Grants protection from Gorgons or otherwise shuts down Gorgon creatures.",
    ),
    (
        "hate-typal-homarid",
        "Grants protection from Homarids, keeping it safe from their damage and targeting.",
    ),
    (
        "hate-typal-inkling",
        "Interacts with Inkling tokens, such as stopping them from attacking you.",
    ),
    (
        "hate-typal-kithkin",
        "Reacts whenever any player casts a Kithkin spell, draining a player's life.",
    ),
    (
        "hate-typal-lizard",
        "Grants a creature protection from Lizards.",
    ),
    (
        "hate-typal-ninja",
        "Destroys or removes a target Ninja creature.",
    ),
    (
        "hate-typal-non-assassin",
        "Targets or exiles a creature that isn't an Assassin.",
    ),
    (
        "hate-typal-non-cat",
        "Restricts creatures that aren't Cats, such as limiting which ones can block.",
    ),
    (
        "hate-typal-non-devil",
        "Destroys creatures that aren't Demons, Devils, or Imps.",
    ),
    (
        "hate-typal-non-faerie",
        "Counters a spell that isn't a Faerie spell.",
    ),
    (
        "hate-typal-non-god",
        "Forces sacrifice or removal of creatures that aren't Gods.",
    ),
    (
        "hate-typal-non-imp",
        "Destroys creatures that aren't Demons, Devils, or Imps.",
    ),
    (
        "hate-typal-non-rogue",
        "Restricts creatures that aren't Rogues, such as preventing them from blocking.",
    ),
    (
        "hate-typal-non-soldier",
        "Punishes or restricts creatures that aren't Soldiers, such as preventing their combat damage.",
    ),
    (
        "hate-typal-non-vampire",
        "Punishes or restricts creatures that aren't Vampires, such as forcing them to be sacrificed.",
    ),
    (
        "hate-typal-non-villain",
        "Punishes or restricts creatures that aren't Villains, such as dealing damage to them.",
    ),
    (
        "hate-typal-non-werewolf",
        "Punishes or restricts creatures that aren't Werewolves.",
    ),
    (
        "hate-typal-phyrexian",
        "Punishes or restricts Phyrexian creatures, such as granting protection from them.",
    ),
    (
        "hate-typal-pirate",
        "Punishes or restricts Pirate creatures, such as dealing damage to them.",
    ),
    (
        "hate-typal-salamander",
        "Punishes or interacts with Salamander creatures, such as granting you protection from them.",
    ),
    (
        "hate-typal-saproling",
        "Punishes or interacts with Saproling creatures, such as being unblockable by them.",
    ),
    (
        "hate-typal-scarecrow",
        "Punishes or interacts with Scarecrow creatures, such as destroying one.",
    ),
    (
        "hate-typal-serf",
        "Punishes or interacts with Serf creatures, such as exiling all Serfs when it leaves play.",
    ),
    (
        "hate-typal-soldier",
        "Punishes or interacts with Soldier creatures, such as caring whenever one enters.",
    ),
    (
        "hate-typal-spider",
        "Punishes or interacts with Spider creatures, such as destroying one it damages.",
    ),
    (
        "hate-typal-survivor",
        "Punishes or interacts with Survivor creatures, such as stopping your opponents' Survivors from blocking or attacking you.",
    ),
    (
        "hate-typal-thrull",
        "Buffs all Thrull creatures and can seize control of a Thrull.",
    ),
    (
        "hate-typal-warlock",
        "Deals damage to Warlocks and other outlaw creatures.",
    ),
    (
        "hate-typal-warrior",
        "Creates and buffs goaded Warrior tokens for each player.",
    ),
    (
        "hate-typal-wolf",
        "Bounces a Wolf or Werewolf back to its owner's hand.",
    ),
    (
        "hate-typal-yeti",
        "Grants protection from Yetis and other unusual creature types.",
    ),
    (
        "hate-ward",
        "Lets you target creatures with ward as though they had none, ignoring their ward triggers.",
    ),
    (
        "hate-warp",
        "Sends an exiled card with warp to the bottom of its owner's library.",
    ),
    (
        "hexproof-soft",
        "Makes a permanent harder, but not impossible, for opponents to target.",
    ),
    (
        "impulse-artifact-legendary",
        "Looks at the top of your library and lets you grab a legendary artifact card into your hand.",
    ),
    (
        "impulse-artifact-spacecraft",
        "Looks at the top of your library and lets you grab a Spacecraft card into your hand.",
    ),
    (
        "impulse-black",
        "Looks at the top of your library and lets you grab a black card into your hand.",
    ),
    (
        "impulse-blue",
        "Looks at the top of your library and lets you grab a blue or artifact card into your hand.",
    ),
    (
        "impulse-colorless",
        "Looks at the top of your library and lets you grab a colorless card into your hand.",
    ),
    (
        "impulse-creature-ally",
        "Looks at the top of your library and lets you grab an Ally card into your hand.",
    ),
    (
        "impulse-creature-angel",
        "Looks through the top cards of your library for an Angel card and puts it into your hand.",
    ),
    (
        "impulse-creature-assassin",
        "Looks through the top cards of your library for an Assassin card and puts it into your hand.",
    ),
    (
        "impulse-creature-demon",
        "Looks through the top cards of your library for a Demon card and puts it into your hand.",
    ),
    (
        "impulse-creature-dinosaur",
        "Looks through the top cards of your library for a Dinosaur card and puts it into your hand.",
    ),
    (
        "impulse-creature-elemental",
        "Looks through the top cards of your library for an Elemental card and puts it into your hand.",
    ),
    (
        "impulse-creature-kavu",
        "Reveals cards off the top of your library and puts any Kavu cards found into your hand.",
    ),
    (
        "impulse-creature-knight",
        "Looks through the top cards of your library for a Knight card and puts it into your hand.",
    ),
    (
        "impulse-creature-merfolk",
        "Digs into your library and grabs Merfolk cards it finds into your hand.",
    ),
    (
        "impulse-creature-mutant",
        "Digs into your library and lets you grab a Mutant, Ninja, Turtle, or land into your hand.",
    ),
    (
        "impulse-creature-ninja",
        "Digs into your library and lets you grab a Mutant, Ninja, Turtle, or land into your hand.",
    ),
    (
        "impulse-creature-pirate",
        "Digs into your library and grabs an Assassin, Pirate, or Vehicle card into your hand.",
    ),
    (
        "impulse-creature-rat",
        "Digs into your library and grabs Rat cards it finds into your hand.",
    ),
    (
        "impulse-creature-zombie",
        "Digs into your library and grabs Zombie cards it finds into your hand.",
    ),
    (
        "impulse-enchantment-shrine",
        "Digs into your library and grabs a Lesson, Saga, or Shrine card into your hand.",
    ),
    (
        "impulse-multicolor",
        "Reveals cards from the top of your library and puts the multicolor ones into your hand.",
    ),
    (
        "impulse-pw-garruk",
        "Digs through the top of your library for a creature or Garruk planeswalker card to put into your hand.",
    ),
    (
        "impulse-red",
        "Looks at the top of your library and puts a red card into your hand.",
    ),
    (
        "impulse-white",
        "Looks at the top of your library and puts a white card into your hand.",
    ),
    (
        "indefinite-effect",
        "Creates an effect that lasts indefinitely with no fixed duration.",
    ),
    (
        "location-matters",
        "Does something different depending on the real-world location where you are playing.",
    ),
    (
        "lockdown-spacecraft",
        "Takes control of a Spacecraft and keeps it from untapping.",
    ),
    (
        "match-points-matter",
        "Changes how many points this match is worth.",
    ),
    (
        "protects-token",
        "Keeps your tokens safe, such as granting them indestructible.",
    ),
    (
        "protects-vehicle",
        "Grants a Vehicle indestructible or otherwise shields it from removal.",
    ),
    (
        "pseudo-ante",
        "Exchanges ownership of cards between players, an ante-style effect.",
    ),
    (
        "pseudo-vehicle",
        "A creature that needs another creature tapped to attack or block, like a Vehicle needing crew.",
    ),
    (
        "reanimate-instant-sorcery",
        "Puts an instant or sorcery card from a graveyard onto the battlefield as a creature.",
    ),
    (
        "reanimate-legendary",
        "Returns a legendary permanent card from a graveyard straight onto the battlefield.",
    ),
    (
        "reanimate-saga",
        "Returns a Saga card from a graveyard to the battlefield, often resuming its chapters.",
    ),
    (
        "recursion-creature",
        "Returns a creature card from your graveyard to your hand.",
    ),
    (
        "recursion-enchantment",
        "Returns an enchantment card from your graveyard to your hand.",
    ),
    (
        "recursion-permanent",
        "Returns a permanent card from your graveyard to your hand.",
    ),
    (
        "regrowth-arcane",
        "Returns an Arcane card from your graveyard to your hand.",
    ),
    (
        "regrowth-battle",
        "Returns a battle card from your graveyard to your hand.",
    ),
    (
        "regrowth-food",
        "Returns a Food card from your graveyard to your hand.",
    ),
    (
        "regrowth-spacecraft",
        "Returns a Spacecraft card from your graveyard to your hand.",
    ),
    (
        "removes-deathtouch",
        "Strips deathtouch from your opponents' creatures.",
    ),
    (
        "removes-double-strike",
        "Strips first strike and double strike from your opponents' creatures.",
    ),
    (
        "removes-infect",
        "Strips infect from your opponents' creatures.",
    ),
    (
        "removes-toxic",
        "Strips toxic from your own creatures, often upgrading it to infect.",
    ),
    (
        "removes-ward",
        "Strips ward and other protective abilities from your opponents' permanents.",
    ),
    (
        "repeatable-draw",
        "Lets you draw extra cards again and again, not just once.",
    ),
    (
        "rescue-nonartifact",
        "Returns one of your own permanents to your hand so you can replay it for value.",
    ),
    (
        "restart-turn",
        "Restarts the current turn from the beginning, undoing everything that happened so far.",
    ),
    (
        "restock-noncreature",
        "Returns a noncreature, nonland card from your graveyard to the top of your library.",
    ),
    (
        "restock-nonland",
        "Shuffles nonland cards from your graveyard back into your library.",
    ),
    (
        "restock-permanent",
        "Returns a permanent card from your graveyard to the top of your library.",
    ),
    (
        "roll-planar-die",
        "Has you roll the planar die to trigger a chaos or planar effect.",
    ),
    (
        "sacrifice-outlet-treasure",
        "Lets you sacrifice Treasure tokens as the cost to trigger an effect.",
    ),
    (
        "seek-artifact-spacecraft",
        "Finds a Spacecraft card at random from your library and puts it into your hand.",
    ),
    (
        "seek-cast",
        "Searches your library for a card and lets you play it when you cast a qualifying spell.",
    ),
    (
        "seek-creature-kithkin",
        "Searches your library for a Kithkin card and puts it into your hand.",
    ),
    (
        "seek-creature-merfolk",
        "Searches your library for a Merfolk card and puts it into your hand.",
    ),
    (
        "seek-creature-pirate",
        "Searches your library for a Pirate card and puts it into your hand.",
    ),
    (
        "seek-creature-rat",
        "Searches your library for a Rat card and puts it into your hand.",
    ),
    (
        "seek-creature-survivor",
        "Searches your library for a Survivor card and puts it into your hand.",
    ),
    (
        "seek-creature-vampire",
        "Searches your library for a Vampire card and puts it into your hand.",
    ),
    (
        "seek-enchantment",
        "Seeks creature, enchantment, or planeswalker cards and puts them onto the battlefield.",
    ),
    (
        "seek-enchantment-saga",
        "Seeks a random historic card (artifact, legendary, or Saga) from your library.",
    ),
    (
        "seek-land-island",
        "Seeks a random land card with the basic land type Island from your library.",
    ),
    (
        "seek-land-nonbasic",
        "Seeks a random nonbasic land card from your library.",
    ),
    (
        "seek-land-plains",
        "Seeks a random land card with the basic land type Plains from your library.",
    ),
    (
        "seek-land-swamp",
        "Seeks a random land card with the basic land type Swamp from your library.",
    ),
    (
        "seek-land-urza-s",
        "Seeks a random Urza's land card (Mine, Power Plant, or Tower) from your library.",
    ),
    (
        "seek-legendary",
        "Seeks a legendary or historic card from your library into your hand.",
    ),
    (
        "seek-noncreature",
        "Seeks a noncreature card from your library and puts it into your hand.",
    ),
    (
        "seek-planeswalker",
        "Seeks a planeswalker card from your library, often alongside other card types.",
    ),
    (
        "seek-self",
        "Seeks another copy of this card from your library.",
    ),
    (
        "shockland",
        "A land that enters tapped unless you pay 2 life.",
    ),
    (
        "shroud-from-black",
        "Can't be targeted by black spells or abilities from black sources.",
    ),
    (
        "shroud-from-blue",
        "Makes a creature unable to be targeted by blue spells or abilities from blue sources.",
    ),
    (
        "shroud-from-nongreen",
        "Can't be targeted by nongreen spells or abilities from nongreen sources.",
    ),
    (
        "shroud-from-red",
        "Can't be targeted by red spells or abilities from red sources.",
    ),
    (
        "sneak-artifact-creature",
        "A synergy tag for artifact creatures built around cheating them onto the battlefield.",
    ),
    (
        "synergy-affinity",
        "A synergy tag for cards that support or reward the affinity mechanic.",
    ),
    (
        "synergy-airbending",
        "A synergy tag for cards themed around airbending abilities.",
    ),
    (
        "synergy-awaken",
        "A synergy tag for cards that support or reward the awaken mechanic.",
    ),
    (
        "synergy-banana",
        "A synergy tag for cards that care about Banana tokens.",
    ),
    (
        "synergy-bargain",
        "Rewards or grants bargain, sacrificing an artifact, enchantment, or token to boost a spell.",
    ),
    (
        "synergy-blitz",
        "Rewards or grants blitz, giving a creature haste, then sacrificing it to draw a card.",
    ),
    (
        "synergy-boon",
        "Rewards or grants a boon, a one-time bonus effect that lingers until it triggers once.",
    ),
    (
        "synergy-bushido",
        "Rewards or grants bushido, boosting a creature's power and toughness when it blocks or is blocked.",
    ),
    (
        "synergy-case",
        "Rewards or interacts with solving a Case, triggering an effect when a Case's conditions are met.",
    ),
    (
        "synergy-changeling",
        "Rewards or grants changeling, treating a creature as every creature type at once.",
    ),
    (
        "synergy-color-non-share",
        "Cares about not sharing a color with something, often granting a bonus when colors differ.",
    ),
    (
        "synergy-craft",
        "Rewards or supports using the craft mechanic to exile permanents and transform.",
    ),
    (
        "synergy-damage-prevention",
        "Rewards you for preventing damage that would otherwise be dealt.",
    ),
    (
        "synergy-dash",
        "Rewards or supports casting creatures with the dash mechanic.",
    ),
    (
        "synergy-devour",
        "Rewards or supports using devour, sacrificing creatures for +1/+1 counters.",
    ),
    (
        "synergy-disguise",
        "Rewards or supports playing creatures face down with the disguise mechanic.",
    ),
    (
        "synergy-earthbending",
        "Rewards or supports using earthbending abilities.",
    ),
    (
        "synergy-echo",
        "Rewards or supports permanents that use the echo mechanic.",
    ),
    (
        "synergy-embalm",
        "Rewards you for having or activating embalm on your creature cards.",
    ),
    (
        "synergy-enlist",
        "Rewards you for attacking with a creature that enlisted another creature.",
    ),
    (
        "synergy-equipment-legendary",
        "Rewards you when a creature wearing a legendary Equipment deals combat damage.",
    ),
    (
        "synergy-eternalize",
        "Rewards you for having or activating eternalize on your creature cards.",
    ),
    (
        "synergy-firebending",
        "Rewards you when your firebending triggers go off.",
    ),
    ("synergy-gift", "Rewards you whenever you give a gift."),
    (
        "synergy-haunt",
        "Rewards you for haunting a creature or effects tied to haunted creatures.",
    ),
    (
        "synergy-ingest",
        "Grants or cares about the ingest keyword on your creatures.",
    ),
    (
        "synergy-intimidate",
        "Grants or cares about the intimidate keyword on your creatures.",
    ),
    (
        "synergy-jump-start",
        "Grants jump-start or rewards you for casting cards with jump-start.",
    ),
    (
        "synergy-madness",
        "Rewards you for discarding or casting cards with madness.",
    ),
    (
        "synergy-manifest",
        "Helps you cast face-down creatures or turn manifested cards face up.",
    ),
    ("synergy-map", "Cares about you controlling a Map token."),
    (
        "synergy-mentor",
        "Grants mentor or rewards you when a creature mentors another.",
    ),
    (
        "synergy-partner",
        "Rewards creatures that have keyword abilities, such as partner, with stat boosts.",
    ),
    ("synergy-plan", "Gains a benefit while you control a Plan."),
    (
        "synergy-plot",
        "Reduces the cost of plotting cards or casting spells you previously plotted.",
    ),
    (
        "synergy-pw-angrath",
        "Gets stronger or gains an ability as long as you control an Angrath planeswalker.",
    ),
    (
        "synergy-pw-ashiok",
        "Gets stronger or gains an ability as long as you control an Ashiok planeswalker.",
    ),
    (
        "synergy-pw-basri",
        "Gets stronger or gains an ability as long as you control a Basri planeswalker.",
    ),
    (
        "synergy-pw-davriel",
        "Gets stronger or gains an ability as long as you control a Davriel planeswalker.",
    ),
    (
        "synergy-pw-domri",
        "Gains power, toughness, or an ability while you control a Domri planeswalker.",
    ),
    (
        "synergy-pw-dovin",
        "Gains power, toughness, or vigilance while you control a Dovin planeswalker.",
    ),
    (
        "synergy-pw-elspeth",
        "Gets a stat boost while you control an Elspeth planeswalker.",
    ),
    (
        "synergy-pw-huatli",
        "Grants a bigger bonus or extra effect when you control a Huatli planeswalker.",
    ),
    (
        "synergy-pw-lukka",
        "A card that works with Lukka planeswalkers, such as casting creatures they exile.",
    ),
    (
        "synergy-pw-oko",
        "Unlocks a bonus effect or bigger boost while you control an Oko planeswalker.",
    ),
    (
        "synergy-pw-ral",
        "Gets a power boost or extra effect while you control a Ral planeswalker.",
    ),
    (
        "synergy-pw-rowan",
        "Cares about controlling a Rowan planeswalker to boost itself or unlock an effect.",
    ),
    (
        "synergy-pw-sarkhan",
        "Triggers or powers up when you activate a Sarkhan planeswalker's ability.",
    ),
    (
        "synergy-pw-tamiyo",
        "Cares about or interacts with Tamiyo planeswalker cards, such as exiling them from your graveyard.",
    ),
    (
        "synergy-pw-ugin",
        "Cares about Ugin planeswalker cards, such as returning one from your graveyard to your hand.",
    ),
    (
        "synergy-pw-yanggu",
        "Cares about controlling a Yanggu planeswalker to unlock an activated ability.",
    ),
    (
        "synergy-regenerate",
        "A creature that can regenerate itself, protecting it from destruction.",
    ),
    (
        "synergy-role",
        "Cares about Role enchantments attached to a creature.",
    ),
    (
        "synergy-sneak",
        "Cares about the sneak mechanic, letting you cast creature cards from your graveyard for an alternate cost.",
    ),
    (
        "synergy-spectacle",
        "Cares about the spectacle mechanic, letting you cast spells cheaper if an opponent lost life this turn.",
    ),
    (
        "synergy-start-your-engines",
        "Cares about the start your engines mechanic, which gives you Speed on the way to max speed.",
    ),
    (
        "synergy-tantrum",
        "Cares about the tantrum keyword granted to your creatures.",
    ),
    (
        "synergy-type-change",
        "Cares about changing a permanent's card type, such as turning a creature into an artifact.",
    ),
    (
        "synergy-urza-s",
        "Cares about Urza's lands, the Tron-style cycle that produces extra mana together.",
    ),
    (
        "synergy-villainous-choice",
        "Cares about the villainous choice mechanic, where an opponent must pick between two bad options.",
    ),
    (
        "synergy-waterbending",
        "Rewards or combos with cards that use the waterbending mechanic.",
    ),
    (
        "synergy-wither",
        "Grants wither to your creatures or rewards you for having it.",
    ),
    (
        "tapper-spacecraft",
        "Taps a Spacecraft and keeps it from untapping.",
    ),
    (
        "tax",
        "Makes players pay extra resources to act, or generates value for you when they do.",
    ),
    (
        "theft-artifact-creature",
        "Gives you control of an artifact creature.",
    ),
    (
        "theft-commander",
        "Lets you seize control of one or more commanders.",
    ),
    (
        "theft-spacecraft",
        "Gives you control of a creature or Spacecraft and keeps it from untapping.",
    ),
    (
        "triland",
        "A land that taps for three different colors of mana.",
    ),
    (
        "tutor-artifact-colored",
        "Searches your library for a colored artifact card.",
    ),
    (
        "tutor-artifact-food",
        "Searches your library for a Food card.",
    ),
    (
        "tutor-artifact-land",
        "Searches your library for an artifact land card.",
    ),
    (
        "tutor-artifact-noncreature",
        "Searches your library for a noncreature artifact card.",
    ),
    (
        "tutor-aura-creature",
        "Searches your library for an Aura that enchants a creature.",
    ),
    (
        "tutor-creature-assembly-worker",
        "Searches your library for an Assembly-Worker creature card.",
    ),
    (
        "tutor-creature-aurochs",
        "Searches your library for an Aurochs card and puts it into your hand.",
    ),
    (
        "tutor-creature-bird",
        "Searches your library for a Bird card and puts it into your hand.",
    ),
    (
        "tutor-creature-construct",
        "Searches your library for a Construct card and puts it into your hand.",
    ),
    (
        "tutor-creature-deathtouch",
        "Searches your library for a creature card with deathtouch and puts it on top of your library.",
    ),
    (
        "tutor-creature-demigod",
        "Searches your library for a Demigod card and puts it into your hand.",
    ),
    (
        "tutor-creature-dwarf",
        "Searches your library for Dwarf cards and puts them on top of your library.",
    ),
    (
        "tutor-creature-eldrazi",
        "Searches your library for an Eldrazi card and puts it into your hand.",
    ),
    (
        "tutor-creature-elemental",
        "Searches your library for an Elemental card and puts it on top of your library.",
    ),
    (
        "tutor-creature-embalm",
        "Searches your library for a creature card with embalm and puts it into your graveyard.",
    ),
    (
        "tutor-creature-eternalize",
        "Searches your library for a creature card with eternalize and puts it into your graveyard.",
    ),
    (
        "tutor-creature-faerie",
        "Searches your library for a Faerie card and puts it on top of your library.",
    ),
    (
        "tutor-creature-flying",
        "Searches your library for a creature card with flying and puts it into your hand.",
    ),
    (
        "tutor-creature-giant",
        "Searches your library for a Giant card and puts it on top of your library.",
    ),
    (
        "tutor-creature-hexproof",
        "Searches your library for a creature card with hexproof and puts it on top of your library.",
    ),
    (
        "tutor-creature-kithkin",
        "Searches your library for a Kithkin card.",
    ),
    (
        "tutor-creature-minotaur",
        "Searches your library for Minotaur creature cards.",
    ),
    (
        "tutor-creature-mount",
        "Searches your library for a Mount or Vehicle card.",
    ),
    (
        "tutor-creature-ninja",
        "Searches your library for a Ninja card.",
    ),
    (
        "tutor-creature-noble",
        "Searches your library for a Noble card.",
    ),
    (
        "tutor-creature-phyrexian",
        "Searches your library for Phyrexian cards.",
    ),
    (
        "tutor-creature-pirate",
        "Searches your library for a Pirate card.",
    ),
    (
        "tutor-creature-rat",
        "Searches your library for a Rat card and puts it into your hand.",
    ),
    (
        "tutor-creature-reach",
        "Searches your library for a creature card with reach and puts it into your hand or on top.",
    ),
    (
        "tutor-creature-squirrel",
        "Searches your library for a Squirrel card and puts it into your hand.",
    ),
    (
        "tutor-creature-trample",
        "Searches your library for a creature card with trample and puts it into your hand or on top.",
    ),
    (
        "tutor-creature-treefolk",
        "Searches your library for a Treefolk card and puts it into your hand or on top.",
    ),
    (
        "tutor-creature-vampire",
        "Searches your library for a Vampire card and puts it into your hand or on top.",
    ),
    (
        "tutor-creature-vanilla",
        "Searches your library for a creature card with no abilities and puts it into your hand.",
    ),
    (
        "tutor-creature-zombie",
        "Searches your library for a Zombie card and puts it into your hand.",
    ),
    (
        "tutor-enchantment-plan",
        "Searches your library for a Plan card and puts it into your hand.",
    ),
    (
        "tutor-enchantment-room",
        "Searches your library for a Room card and puts it into your hand.",
    ),
    (
        "tutor-enchantment-saga",
        "Searches your library for a Saga card and puts it into your hand.",
    ),
    (
        "tutor-flashback",
        "Searches a player's library for cards with flashback and puts them into that player's graveyard or hand.",
    ),
    (
        "tutor-host",
        "Searches your library for a host card and puts it into your hand.",
    ),
    (
        "tutor-instant-sorcery-arcane",
        "Searches your library for an Arcane instant or sorcery and puts it into your hand.",
    ),
    (
        "tutor-instant-sorcery-lesson",
        "Searches your library for a Lesson card and puts it into your hand.",
    ),
    (
        "tutor-instant-sorcery-trap",
        "Searches your library for a Trap card and puts it into your hand.",
    ),
    (
        "tutor-land-specific",
        "Searches your library for one specific named land and puts it into your hand.",
    ),
    (
        "tutor-monocolored",
        "Searches your library for one or more monocolored cards.",
    ),
    (
        "tutor-multicolored",
        "Searches your library for one or more multicolored cards.",
    ),
    (
        "tutor-permanent-snow",
        "Searches your library for a snow permanent card and puts it into your hand.",
    ),
    (
        "tutor-white",
        "Searches your library for a white card and puts it into your hand.",
    ),
    (
        "typal-aetherborn",
        "Cares about Aetherborn creatures, often boosting or triggering off them.",
    ),
    (
        "typal-alicorn",
        "Cares about Alicorn creatures, often boosting or triggering off them.",
    ),
    (
        "typal-alien",
        "Cares about Alien creatures, often boosting or triggering off them.",
    ),
    (
        "typal-archon",
        "Cares about Archon creatures, often boosting or triggering off them.",
    ),
    (
        "typal-atog",
        "Cares about Atog creatures, often boosting or triggering off them.",
    ),
    (
        "typal-balloon",
        "Cares about Balloon creatures, often boosting or triggering off them.",
    ),
    (
        "typal-bard",
        "Cares about Bard creatures, often boosting or triggering off them.",
    ),
    (
        "typal-beeble",
        "Cares about Beeble creatures, tiny tokens usually made and multiplied in swarms.",
    ),
    (
        "typal-bison",
        "Cares about Bison creatures, helping you find or put them into play.",
    ),
    (
        "typal-blinkmoth",
        "Cares about Blinkmoth creatures, artifact fliers that lands can become or pump.",
    ),
    (
        "typal-brainiac",
        "Cares about Brainiac creatures, tokens made and boosted through die rolls.",
    ),
    (
        "typal-camarid",
        "Cares about Camarid creatures, small blue tokens tied to Homarid and Cephalid tribes.",
    ),
    (
        "typal-caribou",
        "Cares about Caribou tokens, small creatures you make and sacrifice for life.",
    ),
    (
        "typal-centaur",
        "Cares about Centaur creatures, boosting or supporting the tribe as a group.",
    ),
    ("typal-clown", "Cares about Clown creatures."),
    (
        "typal-donkey",
        "Cares about or synergizes with Donkey creatures.",
    ),
    (
        "typal-drake",
        "Cares about or synergizes with Drake creatures.",
    ),
    (
        "typal-elder-dragon",
        "Cares about or synergizes with Elder Dragon creatures.",
    ),
    (
        "typal-elephant",
        "Cares about or synergizes with Elephant creatures.",
    ),
    (
        "typal-gamer",
        "Cares about or synergizes with Gamer creatures.",
    ),
    (
        "typal-glimmer",
        "Cares about or synergizes with Glimmer creatures.",
    ),
    (
        "typal-guest",
        "Cares about or synergizes with Guest creatures.",
    ),
    (
        "typal-homarid",
        "A card that cares about or boosts Homarid creatures.",
    ),
    (
        "typal-homunculus",
        "A card that cares about or interacts with Homunculus creatures.",
    ),
    (
        "typal-human-werewolf",
        "A card that cares about Human Werewolf creatures, often transforming between the two.",
    ),
    (
        "typal-jackal",
        "A card that cares about or boosts Jackal creatures.",
    ),
    (
        "typal-juggernaut",
        "A card that cares about or boosts Juggernaut creatures.",
    ),
    (
        "typal-killbot",
        "A card that cares about or interacts with Killbot creatures.",
    ),
    (
        "typal-kor",
        "A card that cares about or boosts Kor creatures.",
    ),
    (
        "typal-minion",
        "Cares about or boosts creatures with the Minion creature type.",
    ),
    (
        "typal-monk",
        "Cares about or boosts creatures with the Monk creature type.",
    ),
    (
        "typal-monkey",
        "Cares about or boosts creatures with the Monkey or Ape creature type.",
    ),
    (
        "typal-moonfolk",
        "Cares about or boosts creatures with the Moonfolk creature type.",
    ),
    (
        "typal-nautilid",
        "Cares about or boosts creatures with the Nautilid creature type.",
    ),
    (
        "typal-nightstalker",
        "Cares about or boosts permanents with the Nightstalker creature type.",
    ),
    (
        "typal-noble",
        "Cares about or boosts creatures with the Noble creature type.",
    ),
    (
        "typal-non-archon",
        "Cares about or affects creatures that are not Archons.",
    ),
    (
        "typal-non-assassin",
        "Cares about or affects cards or creatures that are not Assassins.",
    ),
    (
        "typal-non-horse",
        "Cares about or affects creatures that are not Horses.",
    ),
    (
        "typal-non-kree",
        "Cares about or affects creatures that are not Kree.",
    ),
    (
        "typal-non-lemur",
        "Cares about or affects creatures that are not Lemurs.",
    ),
    (
        "typal-non-pilot",
        "Cares about or affects creatures that are not Pilots.",
    ),
    (
        "typal-non-villain",
        "Cares about or affects creatures that are not Villains.",
    ),
    (
        "typal-non-zombie",
        "Cares about creatures that aren't Zombies, such as sacrificing them.",
    ),
    (
        "typal-pentavite",
        "Cares about Pentavite tokens, a 1/1 colorless flying artifact creature type.",
    ),
    (
        "typal-performer",
        "Cares about creatures of the Performer type on the battlefield.",
    ),
    (
        "typal-pilot",
        "Cares about Pilot creatures, which can crew and pair with Vehicles.",
    ),
    (
        "typal-pony",
        "Cares about Pony creatures and related types like Horses and Unicorns.",
    ),
    (
        "typal-prism",
        "Cares about Prism tokens, small artifact creatures usually sacrificed for mana.",
    ),
    (
        "typal-reflection",
        "Cares about Reflection creatures, token copies created by certain effects.",
    ),
    (
        "typal-rhino",
        "Cares about or boosts creatures with the Rhino type you control.",
    ),
    (
        "typal-rigger",
        "Cares about or boosts creatures with the Rigger type you control.",
    ),
    (
        "typal-satyr",
        "Cares about or boosts creatures with the Satyr type you control.",
    ),
    (
        "typal-scion",
        "Cares about or boosts creatures with the Scion type you control.",
    ),
    (
        "typal-scout",
        "Cares about or boosts creatures with the Scout type you control.",
    ),
    (
        "typal-sculpture",
        "Cares about or boosts creatures with the Sculpture type you control.",
    ),
    (
        "typal-seal",
        "Cares about or boosts creatures with the Seal type you control.",
    ),
    (
        "typal-shade",
        "A card that cares about or works with Shade creatures.",
    ),
    (
        "typal-snail",
        "A card that cares about or works with Snail creatures.",
    ),
    (
        "typal-spawn",
        "A card that cares about or works with Spawn creatures.",
    ),
    (
        "typal-specter",
        "A card that cares about or works with Specter creatures.",
    ),
    (
        "typal-survivor",
        "A card that cares about or works with Survivor creatures.",
    ),
    (
        "typal-symbiote",
        "A card that cares about or works with Symbiote creatures.",
    ),
    (
        "typal-teddy-bear",
        "A card that cares about or works with Teddy Bear creatures.",
    ),
    (
        "typal-tentacle",
        "A card that creates or cares about Tentacle creature tokens.",
    ),
    (
        "typal-transformer",
        "A card that cares about or represents Transformer characters like Autobots and Decepticons.",
    ),
    (
        "typal-tyranid",
        "A card that creates or cares about Tyranid creatures.",
    ),
    (
        "typal-whale",
        "A card that cares about Fish, Octopus, Otter, Seal, Serpent, or Whale creatures you control.",
    ),
    (
        "typal-worm",
        "A card that creates or cares about Insects, Leeches, Slugs, or Worms you control.",
    ),
    (
        "undaunted",
        "Costs 1 less to cast for each opponent you have.",
    ),
    (
        "unstable-secret-base",
        "A land that taps for colorless or for a color usable only on spells sharing its watermark.",
    ),
    ("untapper-equipment", "Untaps a piece of Equipment."),
    ("untapper-planeswalker", "Untaps a planeswalker."),
    (
        "affinity-for-allies",
        "Costs 1 less to cast for each Ally you control.",
    ),
    (
        "affinity-for-birds",
        "Costs 1 less to cast for each Bird you control.",
    ),
    (
        "affinity-for-cats",
        "Costs 1 less to cast for each Cat you control.",
    ),
    (
        "affinity-for-caves",
        "Costs 1 less to cast for each Cave you control.",
    ),
    (
        "worse-in-multiplayer",
        "A card that scales down or backfires as more opponents are in the game.",
    ),
    (
        "affinity-for-citizens",
        "Costs 1 less to cast for each Citizen you control.",
    ),
    (
        "affinity-for-daleks",
        "Costs 1 less to cast for each Dalek you control.",
    ),
    (
        "affinity-for-graveyard",
        "Costs 1 less to cast for each noncreature, nonland card in your graveyard.",
    ),
    (
        "affinity-for-humans",
        "Costs 1 less to cast for each Human you control.",
    ),
    (
        "affinity-for-knights",
        "Costs 1 less to cast for each Knight you control.",
    ),
    (
        "affinity-for-outlaws",
        "Costs 1 less to cast for each Assassin, Mercenary, Pirate, Rogue, or Warlock you control.",
    ),
    (
        "affinity-for-phyrexians",
        "Costs 1 less to cast for each Phyrexian you control.",
    ),
    (
        "affinity-for-slivers",
        "Costs 1 less to cast for each Sliver you control.",
    ),
    (
        "affinity-for-spirits",
        "Costs 1 less to cast for each Spirit you control.",
    ),
    (
        "affinity-for-tokens",
        "Costs 1 less to cast for each token you control.",
    ),
    (
        "affinity-for-towns",
        "Costs 1 less to cast for each Town you control.",
    ),
    (
        "artifact-matters",
        "A card that cares about the artifacts you control.",
    ),
    (
        "boltland",
        "A land you can pay 3 life or take 3 damage to make enter untapped.",
    ),
    (
        "buttfling",
        "Sacrifices a creature and deals damage equal to its toughness instead of its power.",
    ),
    (
        "buttlink",
        "Gains you life equal to a creature's toughness when it deals combat damage to a player.",
    ),
    (
        "buttsaddle",
        "Lets a creature saddle Mounts or crew Vehicles using its toughness instead of its power.",
    ),
    (
        "buttstation",
        "Lets a creature station permanents or assign combat damage using its toughness instead of its power.",
    ),
    (
        "conjure-nonland",
        "Conjures a duplicate of a nonland card into your hand.",
    ),
    (
        "conjure-permanent",
        "Conjures a duplicate of a permanent card into your hand.",
    ),
    (
        "cost-reducer-battle",
        "Reduces the cost of spells of a chosen non-creature, non-land card type, such as battles.",
    ),
    (
        "cost-reducer-lesson",
        "Makes Lesson spells you cast cost less.",
    ),
    (
        "cost-reducer-nonland",
        "Reduces the mana cost of a nonland card in your hand.",
    ),
    (
        "cost-reducer-sorcery",
        "Reduces the mana cost of sorcery spells you cast.",
    ),
    (
        "cost-reducer-vehicle",
        "Reduces the mana cost of Vehicle spells you cast.",
    ),
    (
        "counter-fuel-lore",
        "Lets you remove lore counters from Sagas to power an effect.",
    ),
    (
        "counter-fuel-pt",
        "Lets you remove +1/+0 or other p/t counters to power an effect.",
    ),
    (
        "counter-fuel-stun",
        "Lets you remove stun counters from permanents to power an effect.",
    ),
    (
        "counterspell-battle",
        "Counters a battle spell, often unless its controller pays a cost.",
    ),
    (
        "counterspell-loyalty-ability",
        "Counters a target loyalty ability of a planeswalker.",
    ),
    (
        "creature-type-fungusaur",
        "A creature that is both a Fungus and a Dinosaur.",
    ),
    (
        "cross-game-card",
        "A card crossing over from another game or franchise outside core Magic.",
    ),
    (
        "cycle-innistrad-legend-angel",
        "One of a cycle of legendary Angels tied to Innistrad.",
    ),
    (
        "decayed-counter",
        "Puts decayed counters on creatures, so they can't block and are sacrificed after they attack.",
    ),
    (
        "digital-replacement",
        "A card needing a physical component, like a sticker, that is handled differently in digital play.",
    ),
    (
        "exalted-counter",
        "Puts an exalted counter on a creature, adding an exalted instance that pumps whatever attacks alone.",
    ),
    (
        "flicker",
        "Exiles a permanent and returns it to the battlefield, retriggering its enter-the-battlefield abilities.",
    ),
    (
        "flicker-nonenchantment",
        "Exiles a nonenchantment permanent and returns it to the battlefield soon after.",
    ),
    (
        "flicker-planeswalker",
        "Exiles a creature or planeswalker and returns it to the battlefield, often stronger than before.",
    ),
    (
        "flicker-vehicle",
        "Exiles a creature or Vehicle and returns it to the battlefield shortly after.",
    ),
    (
        "fun",
        "A card with a playful, quirky effect meant to make games more fun.",
    ),
    (
        "gains-cascade",
        "Gives itself cascade: when cast, exile cards until a cheaper spell you can cast free.",
    ),
    (
        "gains-dethrone",
        "Gains dethrone: gets a +1/+1 counter when it attacks the player with the most life.",
    ),
    (
        "gains-exploit",
        "Gives a creature exploit, letting you sacrifice a creature when it enters for a bonus effect.",
    ),
    (
        "gains-firebending",
        "Gives a creature firebending, adding red mana when it attacks that lasts until end of combat.",
    ),
    (
        "gains-for-mirrodin",
        "Grants an Equipment For Mirrodin!, creating a Rebel token and attaching it for free when it enters.",
    ),
    (
        "gains-living-weapon",
        "Grants an Equipment Living Weapon, creating a 0/0 Germ token equipped to it when it enters.",
    ),
    (
        "gains-provoke",
        "Grants a creature provoke, letting it force a chosen creature to untap and block when it attacks.",
    ),
    (
        "gains-soulshift",
        "Grants a creature soulshift, letting you return a Spirit card from your graveyard to hand when it dies.",
    ),
    (
        "gains-split-second",
        "Grants a spell split second, stopping any player from casting spells or activating abilities while it's on the stack.",
    ),
    (
        "gains-umbra-armor",
        "An Aura that protects its enchanted creature from destruction, destroying itself instead.",
    ),
    (
        "gains-undying",
        "Grants undying: when the creature dies with no +1/+1 counters, it returns with one.",
    ),
    (
        "gender-matters",
        "A spell or ability whose effect changes based on the gender of characters or players involved.",
    ),
    (
        "gives-absorb",
        "Grants creatures absorb, preventing a set amount of damage from each source dealt to them.",
    ),
    (
        "gives-adventure",
        "Lets you cast creature cards in hand as a bonus Adventure spell before casting them as creatures.",
    ),
    (
        "gives-basic-landcycling",
        "Lets a card be discarded to search your library for a basic land and put it into your hand.",
    ),
    (
        "gives-delve",
        "Lets you cast spells by exiling cards from your graveyard to help pay their generic cost.",
    ),
    (
        "gives-dethrone",
        "Grants dethrone: a creature gets a +1/+1 counter when it attacks the player with the most life.",
    ),
    (
        "gives-devour",
        "Grants devour, letting a creature enter with +1/+1 counters for each creature sacrificed as it enters.",
    ),
    (
        "gives-disguise",
        "Grants disguise, letting a creature be cast face down as a 2/2 and later turned up for its disguise cost.",
    ),
    (
        "gives-dredge",
        "Grants dredge: return a card from your graveyard to hand by milling instead of drawing.",
    ),
    (
        "gives-echo",
        "Grants echo, making a creature's controller pay its echo cost at their next upkeep or sacrifice it.",
    ),
    (
        "gives-emerge",
        "Grants emerge, letting a creature spell be cast by sacrificing a creature to reduce its cost.",
    ),
    (
        "gives-epic",
        "Grants epic, copying a spell each upkeep while stopping you from casting other spells that turn.",
    ),
    (
        "gives-extort",
        "Grants extort to your other creatures, letting you pay to drain each opponent for 1 whenever you cast a spell.",
    ),
    (
        "gives-fabricate",
        "Grants fabricate to your other nontoken creatures, letting them enter with a +1/+1 counter or make a 1/1 Servo token.",
    ),
    (
        "gives-fossilize",
        "Grants fossilize to creature cards in your graveyard, letting you return them to the battlefield as artifacts.",
    ),
    (
        "gives-freerunning",
        "Grants freerunning to spells, letting you cast them for an alternative cost after dealing combat damage with an Assassin or commander.",
    ),
    (
        "gives-frenzy",
        "Grants frenzy to creatures, giving them extra power when they attack and aren't blocked.",
    ),
    (
        "gives-harmonize",
        "Grants harmonize to a card in your graveyard, letting you cast it from there for its mana cost.",
    ),
    (
        "gives-madness",
        "Grants madness to cards, letting you cast them for their madness cost when discarded instead of sending them to your graveyard.",
    ),
    (
        "gives-mayhem",
        "Lets cards in your graveyard be cast for their mayhem cost if you discarded them this turn.",
    ),
    (
        "gives-modular",
        "Grants modular to artifact creatures, so they enter with a +1/+1 counter that moves to another artifact creature when they die.",
    ),
    (
        "gives-plainswalk",
        "Grants plainswalk, making a creature unblockable while the defending player controls a Plains.",
    ),
    (
        "gives-prowl",
        "Lets you cast spells for their cheaper prowl cost after dealing combat damage to a player with a matching creature type.",
    ),
    (
        "gives-read-ahead",
        "Grants read ahead, letting your Sagas enter with extra lore counters so you can start on a later chapter.",
    ),
    (
        "gives-relentless",
        "Lets your deck include any number of copies of certain cards instead of the usual four-copy limit.",
    ),
    (
        "gives-renown",
        "Grants renown, putting a +1/+1 counter on a creature the first time it deals combat damage to a player.",
    ),
    (
        "gives-ripple",
        "Grants ripple, letting you cast same-named cards revealed atop your library without paying their costs.",
    ),
    (
        "gives-sneak",
        "Grants sneak, letting you cast creature cards from your graveyard for their sneak cost.",
    ),
    (
        "gives-spectacle",
        "Grants spectacle, letting cards be cast for a cheaper cost if an opponent lost life this turn.",
    ),
    (
        "gives-super-haste",
        "Grants super haste, letting a creature enter tapped and attacking straight from your hand.",
    ),
    (
        "gives-thoughtweft",
        "Grants thoughtweft, giving a creature all printed keyword abilities of your other thoughtweft creatures.",
    ),
    (
        "gives-townwalk",
        "Grants townwalk, making a creature unblockable while the defending player controls a Town.",
    ),
    (
        "gives-triple-strike",
        "Grants triple strike, letting a creature deal combat damage three times in one combat.",
    ),
    (
        "gives-umbra-armor",
        "Grants umbra armor to your Auras, so an Aura is destroyed in place of the permanent it enchants.",
    ),
    (
        "gives-undaunted",
        "Grants undaunted, making a spell cost 1 less to cast for each opponent you have.",
    ),
    (
        "gives-unleash",
        "Grants unleash, letting creatures enter with a +1/+1 counter in exchange for being unable to block.",
    ),
    (
        "gives-web-slinging",
        "Grants web-slinging, letting a spell be cast for an alternative cost by tapping an untapped creature.",
    ),
    (
        "gives-wither-noncreature",
        "Grants wither to your noncreature spells, so they deal damage to creatures as -1/-1 counters.",
    ),
    (
        "graveyard-fuel-saga",
        "Exiles Saga cards from a graveyard to copy one of their chapter abilities.",
    ),
    (
        "hand-size-hate",
        "Punishes opponents based on the number of cards in their hand.",
    ),
    (
        "hate-backup",
        "Stops opponents' creatures from triggering backup abilities when they enter.",
    ),
    (
        "hate-blood",
        "Punishes or interacts with Blood tokens your opponents control.",
    ),
    (
        "hate-clue",
        "Punishes or replaces the Clue tokens your opponents would create.",
    ),
    (
        "hate-conspiracy",
        "Rewards you for hitting a player who has a conspiracy in their command zone.",
    ),
    (
        "hate-contraption",
        "Destroys or interacts with a Contraption.",
    ),
    (
        "hate-counters",
        "Punishes creatures that have +1/+1 counters on them.",
    ),
    ("hate-curse", "Destroys or removes Curses attached to you."),
    ("hate-dice", "Grants a creature protection from die rolls."),
    (
        "hate-disturb",
        "Removal that can exile a Spirit, creature with disturb, or enchantment.",
    ),
    (
        "hate-fear",
        "A creature that has fear while a creature card with fear is in a graveyard.",
    ),
    (
        "hate-flanking",
        "Strips flanking from creatures that attack you and punishes non-flying attackers.",
    ),
    (
        "hate-goad",
        "Cares about goaded creatures you control, rewarding you and clearing their goaded status.",
    ),
    (
        "hate-hybrid",
        "Grants a creature protection from hybrid mana.",
    ),
    ("hate-kicker", "Counters a spell only if it was kicked."),
    (
        "hate-lifelink",
        "Gains lifelink for itself while a creature with lifelink is in a graveyard.",
    ),
    (
        "hate-menace",
        "Gains menace for itself while a creature with menace is in a graveyard.",
    ),
    (
        "hate-mm-counter",
        "Reduces how many -1/-1 counters get placed on your creatures.",
    ),
    (
        "hate-planeswalker-bolas",
        "Lets you exile a Bolas and proliferate three times.",
    ),
    (
        "hate-planeswalker-chandra",
        "Removal that gives extra value when aimed at a Chandra planeswalker.",
    ),
    (
        "hate-planeswalker-jace",
        "Counters a blue spell, then scries 2 if it was a Jace planeswalker.",
    ),
    (
        "hate-plot",
        "Can send an opponent's face-up plotted card to their graveyard.",
    ),
    ("hate-room", "Removal aimed at Room permanents."),
    (
        "hate-saga",
        "Removal aimed at Sagas and other historic permanents.",
    ),
    (
        "hate-scry",
        "Punishes an opponent whenever they scry, surveil, or search their library.",
    ),
    (
        "hate-speed",
        "Reduces an opponent's speed, setting back their high speed rewards.",
    ),
    (
        "hate-splice",
        "Makes an opponent discard cards matching ones they spliced onto a spell.",
    ),
    (
        "hate-surveil",
        "Punishes an opponent whenever they surveil their library.",
    ),
    (
        "hate-town",
        "Grants townwalk, so creatures can't be blocked while the defending player controls a Town.",
    ),
    (
        "hate-toxic",
        "Deals extra damage or otherwise punishes creatures that have toxic.",
    ),
    (
        "hate-trap",
        "Makes a player discard all Trap cards from their hand.",
    ),
    (
        "hate-typal-alien",
        "Grants protection from Aliens and other listed creature types.",
    ),
    (
        "hate-typal-angel",
        "Destroys or otherwise removes an Angel.",
    ),
    ("hate-typal-coyote", "Grants protection from Coyotes."),
    (
        "hate-typal-cyclops",
        "Destroys or otherwise removes a Cyclops.",
    ),
    (
        "hate-typal-devil",
        "Grants protection from Devils and Demons.",
    ),
    (
        "old-typeline",
        "Brings back an old, retired type line like Summon, for nostalgia or humor.",
    ),
    (
        "hate-banding",
        "Strips banding and bands-with-other abilities from a target creature.",
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
