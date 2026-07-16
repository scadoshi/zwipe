# Oracle-tag description audit — progress & findings

Independent second-pass QA of `ORACLE_TAG_DESCRIPTIONS`. Each batch re-checks authored
descriptions against real card oracle text ([`otag_audit_workflow.js`](otag_audit_workflow.js))
and flags inaccuracies with a card example and a suggested fix. **Findings are for review; fixes
are NOT auto-applied.**

## Coverage / resume
- **Audited: 1000 / 4,357** (top tags by card population, highest-impact first).
- **Resume: re-run the top-N population pull.** It excludes every slug already in
  [`otag_audited_slugs.txt`](otag_audited_slugs.txt), so it self-heals gaps and dupes. Next up: rank ~1001+.

## Findings so far
Across 1000 audited: **820 clean, 174 suspect, 6 wrong.**

## Wrong (fix recommended)

### `unique-token` — WRONG
- **current:** A specific token variant that only one card creates.
- **issue:** claims 'only one card creates' it, but the tagged tokens are the most generic ones made by hundreds of cards
- **example:** Soldier (Token Creature, Vigilance) and Wolf/Sheep/Insect tokens are produced by many different cards, not one
- **suggested fix:** A predefined, named token creature that cards put onto the battlefield.

### `minigame` — WRONG
- **current:** Engages other players in a bet, vote, or guessing game.
- **issue:** Overspecifies 'other players'; many are solo minigames or involve a person outside the game
- **example:** Push Your Luck: 'Reveal cards from the top of your library until you decide to stop' is a solo press-your-luck game; Scavenger Hunt: 'You have ten seconds to search your library' is a solo timed search, neither engages other players
- **suggested fix:** Creates a real-world minigame such as a timed search, press-your-luck, vote, or guessing challenge to determine its effect.

### `warlord` — WRONG
- **current:** A creature whose power, and sometimes toughness, equals how many creatures you control.
- **issue:** Overspecifies to 'creatures you control'; over half the cards scale off other things (lands, color-based permanents, Clerics)
- **example:** Ashaya, Soul of the Wild: 'power and toughness are each equal to the number of lands you control'; Kithkin Rabble: 'equal to the number of white permanents you control'
- **suggested fix:** A creature whose power, and often toughness, equals the number of creatures or other permanents you control.

### `creature-ability-noncreature` — WRONG
- **current:** A noncreature permanent with abilities that matter once it becomes a creature.
- **issue:** Slug-name trap: most cards never become creatures; the theme is a noncreature carrying a creature-style ability/keyword
- **example:** Weapon Rack (Artifact): 'enters with three +1/+1 counters... Move a +1/+1 counter from this artifact onto target creature' and Darkmoss Bridge (Artifact Land): 'Indestructible' — neither becomes a creature
- **suggested fix:** A noncreature permanent that has an ability or keyword normally found on creatures, such as indestructible or +1/+1 counters.

### `hate-wide` — WRONG
- **current:** Scales up to punish opponents based on how many creatures they control.
- **issue:** Says punishes opponents by THEIR creature count; most cards scale with all creatures on the battlefield and some benefit you rather than punish
- **example:** Chain Reaction: 'deals X damage to each creature, where X is the number of creatures on the battlefield'; War Report gains you life per creature
- **suggested fix:** Scales with the number of creatures on the battlefield, often to answer go-wide boards.

### `keyword-soup` — WRONG
- **current:** A card that gains or lists most of its set's keyword abilities.
- **issue:** 'its set's keyword abilities' is misleading; cards reference a fixed list of evergreen keywords, not any expansion's keywords
- **example:** Odric, Blood-Cursed: counts 'flying, first strike, double strike, deathtouch, haste, hexproof, indestructible, lifelink, menace, reach, trample, and vigilance'
- **suggested fix:** Grants, counts, or keys off a long list of evergreen keyword abilities such as flying, first strike, and trample all at once.

## Suspect (imprecise — review)

### `removal-destroy` — suspect
- **current:** Removal that destroys its target.
- **issue:** "its target" overspecifies to single-target; the tag includes untargeted mass-destroy board wipes
- **example:** Fracturing Gust: "Destroy all artifacts and enchantments" (no single target)
- **suggested fix:** Removal that destroys what it affects, including board wipes.

### `namesake-spell` — suspect
- **current:** A spell named after a specific character.
- **issue:** Says 'spell' but the tag also covers permanents (creatures, artifacts) that aren't spells once resolved; 'card' is more accurate.
- **example:** Pharika's Mender (Creature — Gorgon) and Gerrard's Hourglass Pendant (Legendary Artifact) carry the tag but are permanents, not spells.
- **suggested fix:** A card named after a specific character.

### `repeatable-pure-draw` — suspect
- **current:** Repeatably draws cards at no extra cost.
- **issue:** "at no extra cost" is contradicted by several tagged cards that draw but also cost you life
- **example:** Tinybones, Trinket Thief: "you draw a card and you lose 1 life"; You Compleat Me emblem: "you draw a card and you lose 1 life"
- **suggested fix:** Repeatably draws you extra cards.

### `pure-draw` — suspect
- **current:** Draws a card without discarding or sacrificing another to do it.
- **issue:** Qualifier misleads: half the tagged cards visibly discard or sacrifice (themselves) to draw
- **example:** Sunbeam Spellbomb: "{1}, Sacrifice this artifact: Draw a card"; Savai Triome: "Cycling {3} ({3}, Discard this card: Draw a card.)"
- **suggested fix:** Draws one or more cards as its effect.

### `combat-trick` — suspect
- **current:** An instant-speed effect that boosts your creature or weakens an enemy in combat.
- **issue:** Underspecified: 'boosts or weakens' misses the many protection/keyword-granting tricks in this tag
- **example:** Etchings of the Chosen: 'Target creature you control gains indestructible until end of turn'; Mizzium Skin: '+0/+1 and gains hexproof'
- **suggested fix:** An instant-speed effect used in combat to strengthen, protect, or shrink a creature.

### `group-slug` — suspect
- **current:** Makes each opponent lose life or take damage.
- **issue:** Some hit each PLAYER (including you), not just opponents
- **example:** Caustic Hound: "When this creature dies, each player loses 4 life."
- **suggested fix:** Makes each opponent, or sometimes each player, lose life or take damage.

### `gives-haste` — suspect
- **current:** Grants haste to a creature.
- **issue:** Underspecified: many grant haste to multiple creatures / your whole team, not just one
- **example:** Viashino Lashclaw: "{T}, Discard a card: Creatures you control gain haste until end of turn."
- **suggested fix:** Grants haste to one or more creatures, usually ones you control.

### `tapper-creature` — suspect
- **current:** Taps down a creature so it can't attack or block.
- **issue:** Underspecified: several tap ALL of an opponent's creatures, not just one
- **example:** Bond of Discipline: "Tap all creatures your opponents control."
- **suggested fix:** Taps down one or more creatures so they can't attack or block.

### `removal-toughness` — suspect
- **current:** Kills creatures by reducing their toughness to zero.
- **issue:** Overstates: many only shrink toughness and rarely kill, and death is at zero OR LESS
- **example:** Instill Infection: "Put a -1/-1 counter on target creature. Draw a card." (a single counter seldom kills)
- **suggested fix:** Kills a creature by reducing its toughness to zero or less.

### `creaturefall` — suspect
- **current:** Triggers whenever a creature enters the battlefield.
- **issue:** Underspecified: most tagged cards trigger only on a creature YOU control entering, not any player's creature
- **example:** Yorvo, Lord of Garenbrig: 'Whenever another green creature you control enters...'; Kindred Discovery: 'Whenever a creature you control of the chosen type enters...' (6 of 8 examples are 'you control')
- **suggested fix:** Triggers whenever a creature enters the battlefield, usually one you control.

### `synergy-instant` — suspect
- **current:** Rewards playing instants.
- **issue:** Says 'playing' (instants are cast) and omits that these cards almost always reward sorceries too, unlike its sibling's wording
- **example:** Gale, Primeval Conduit: 'Whenever you cast an instant or sorcery spell, put two +1/+1 counters on target creature'
- **suggested fix:** Rewards casting instants, and often sorceries too.

### `gives-trample` — suspect
- **current:** Grants trample to a creature.
- **issue:** 'a creature' understates the many cards that grant trample to your whole team
- **example:** Aggressive Mammoth: 'Other creatures you control have trample.'
- **suggested fix:** Grants trample to one or more other creatures.

### `untapper-creature` — suspect
- **current:** Untaps a target creature.
- **issue:** 'a target creature' understates cards that untap all/multiple creatures, not a single target
- **example:** Karlach, Fury of Avernus: 'untap all attacking creatures.'
- **suggested fix:** Untaps one or more creatures.

### `block-trigger` — suspect
- **current:** Has an ability that triggers when it blocks or becomes blocked.
- **issue:** 'it blocks' misses cards that trigger when another creature you control blocks
- **example:** Perimeter Captain: 'Whenever a creature you control with defender blocks, you may gain 2 life.'
- **suggested fix:** Has an ability that triggers when it or another creature blocks or becomes blocked.

### `per-player` — suspect
- **current:** Scales up based on how many players are in the game.
- **issue:** 'scales up' overspecifies; several tagged cards reference a single player without any player-count scaling
- **example:** Feline Sovereign: 'Whenever one or more Cats you control deal combat damage to a player, destroy up to one target artifact or enchantment that player controls' (no scaling by player count)
- **suggested fix:** Applies or repeats an effect for each player or opponent in the game.

### `self-replacement-effect` — suspect
- **current:** An effect that partially or fully replaces its own resolution.
- **issue:** 'replaces its own resolution' is imprecise; self-replacement effects replace part of a game event within the effect (an 'instead' clause), not the spell's resolution
- **example:** Colossal Growth: 'Target creature gets +3/+3... If this spell was kicked, instead that creature gets +4/+4 and gains trample and haste'; Deny the Divine: 'exile it instead of putting it into its owner's graveyard'
- **suggested fix:** Modifies part of its own effect with a built-in 'instead' clause.

### `mini-refund` — suspect
- **current:** Gives you a small burst of extra mana to spend soon.
- **issue:** 'small burst to spend soon' implies one-time, but most examples are repeatable any-color mana rocks, not a burst
- **example:** Sol Grail: '{T}: Add one mana of the chosen color.' and Obelisk of Naya: '{T}: Add {R}, {G}, or {W}.' are permanent mana sources, not a spend-soon burst
- **suggested fix:** Produces extra mana for you, often of any color.

### `regrowth-creature` — suspect
- **current:** Returns a creature card from your graveyard to your hand.
- **issue:** 'to your hand' misses that some tagged cards return the creature to the battlefield
- **example:** Meren of Clan Nel Toth: '...return it to the battlefield.' returns a graveyard creature to play, not to hand
- **suggested fix:** Returns a creature card from your graveyard to your hand or the battlefield.

### `removal-land` — suspect
- **current:** Removal that destroys, exiles, or disrupts a target land.
- **issue:** 'a target land' excludes non-targeted mass land destruction that also carries the tag
- **example:** Devastation: 'Destroy all creatures and lands.' (destroys every land, not a target)
- **suggested fix:** Removal that destroys, exiles, or disrupts one or more lands.

### `firebreathing` — suspect
- **current:** Repeatably pumps a creature's power with +X/+0 until end of turn for a cost.
- **issue:** '+X/+0' reads as a variable pump; firebreathing is a fixed repeatable +1/+0 per activation, and the tag also covers team/other-creature pumps
- **example:** Dwarven Lieutenant: '{1}{R}: Target Dwarf creature gets +1/+0 until end of turn.'; Inner-Flame Igniter: '{2}{R}: Creatures you control get +1/+0 until end of turn.'
- **suggested fix:** Repeatedly boosts a creature's power with +1/+0 until end of turn for a cost.

### `gives-indestructible` — suspect
- **current:** Grants indestructible to other permanents.
- **issue:** Says 'permanents' but every carrier grants it to creatures
- **example:** Spearbreaker Behemoth: '{1}: Target creature with power 5 or greater gains indestructible until end of turn.'
- **suggested fix:** Grants indestructible to other creatures.

### `burn-with-set-s-mechanic` — suspect
- **current:** A damage spell that also carries the signature mechanic of the set it debuted in.
- **issue:** Says 'damage spell' but several carriers are creatures/permanents, not spells
- **example:** Mudbutton Torchrunner (Creature): 'When this creature dies, it deals 3 damage to any target.'; Jiwari (Legendary Creature) with Channel
- **suggested fix:** A damage-dealing card that also carries the signature mechanic of the set it debuted in.

### `gives-castable-from-exile` — suspect
- **current:** Lets you cast a card from exile instead of your hand.
- **issue:** Says only 'you' can cast, but many taggees grant the ability to any/other players (gives-, not gains-self)
- **example:** Eye of the Storm: 'that player copies each instant or sorcery card exiled... the player may cast the copy without paying its mana cost'; The Pro Tour: 'each player may play the card they drafted for as long as it remains exiled'
- **suggested fix:** Makes a card castable from exile, letting a player cast it from there instead of from their hand.

### `shapechange` — suspect
- **current:** Sets a creature's power and toughness to specific values.
- **issue:** 'specific values' is wrong for variable P/T setters, and it misses type/copy changes that this tag also covers
- **example:** Sworn Defender: 'power becomes the toughness of target creature... minus 1... toughness becomes 1 plus the power of that creature' (not a fixed value); Mindlink Mech: 'becomes a copy of target nonlegendary creature'
- **suggested fix:** Sets a creature's base power and toughness, sometimes changing its types or turning it into a copy.

### `curiosity` — suspect
- **current:** Draws you a card whenever it deals damage to a player.
- **issue:** Says "it deals damage" but many cards grant the ability to another creature (auras/equipment), not themselves
- **example:** Staggering Insight: "Enchanted creature ... has 'Whenever this creature deals combat damage to a player, draw a card.'"
- **suggested fix:** Draws you a card whenever a creature deals damage to a player, on itself or granted to another.

### `pair-commander` — suspect
- **current:** A commander meant to pair with a second commander, via Partner or a Background.
- **issue:** Only lists Partner and Background; the tag also covers Friends forever and Doctor's companion pairings
- **example:** Nyssa of Traken: "Doctor's companion (You can have two commanders if the other is the Doctor.)"; Sophina: "Partner—Friends forever"
- **suggested fix:** A card that can pair with a second commander, via Partner, a Background, Friends forever, or Doctor's companion.

### `utility-mana-rock` — suspect
- **current:** A mana rock with an extra useful ability, or a spell that makes one.
- **issue:** Says only "a spell that makes one," but creatures and vehicles that make or produce mana are also tagged
- **example:** Arbalest Engineers (creature): "Create a tapped Powerstone token"; Flywheel Racer is a Vehicle mana producer
- **suggested fix:** A mana rock with an extra useful ability, or a card that makes or produces one.

### `sacrifice-outlet-land` — suspect
- **current:** Lets you sacrifice a land, often as a one-time cost for an effect.
- **issue:** Frames it as "often a one-time cost," but a sacrifice outlet is characteristically a repeatable, at-will way to sacrifice lands
- **example:** Zuran Orb: "Sacrifice a land: You gain 2 life"; Goblin Clearcutter: "{T}, Sacrifice a Forest: Add three mana..."
- **suggested fix:** Lets you sacrifice a land for an effect, often repeatably as a sacrifice outlet.

### `removal-enchantment` — suspect
- **current:** Removal that can destroy or exile enchantments.
- **issue:** Underspecified: says only "destroy or exile" but the tag also covers enchantment removal by tucking or bouncing
- **example:** Nature Demands an Offering: chosen enchantment is put "on top of its owner's library" (a tuck, not destroy/exile)
- **suggested fix:** Removal that destroys, exiles, or otherwise gets rid of enchantments.

### `selective-group-hug` — suspect
- **current:** A group hug effect that helps some opponents while leaving others out.
- **issue:** Frames it as helping/excluding opponents, but these hand out benefits to chosen players (often including yourself) and can even punish the ones left out
- **example:** Pir's Whim: 'For each player, choose friend or foe. Each friend searches their library for a land... Each foe sacrifices an artifact or enchantment.'
- **suggested fix:** Doles out benefits to players you choose rather than helping everyone equally.

### `dnd-character` — suspect
- **current:** Depicts a named Dungeons and Dragons character in its name or text.
- **issue:** Says 'named D&D character' but several tagged cards are generic archetypes, not named individuals
- **example:** Hag of Mage's Doom (Creature — Hag Warlock) is a generic hag, not a named character like Karlach or Astarion
- **suggested fix:** Represents a character or creature from Dungeons and Dragons.

### `unblockable` — suspect
- **current:** A creature that can't be blocked.
- **issue:** Overspecified as always-unblockable; most cards only conditionally can't be blocked or grant themselves unblockable via an ability
- **example:** Frostpeak Yeti: '{1}{S}: This creature can't be blocked this turn.' and Clockwork Droid exerts to become unblockable
- **suggested fix:** A creature that can't be blocked or can make itself unable to be blocked.

### `discard-with-set-s-mechanic` — suspect
- **current:** Ties a discard effect to a mechanic specific to that card's set, like escape or kicker.
- **issue:** 'Ties a discard effect to a mechanic' overstates the link (the two are usually just bundled, not conditioned); and neither example mechanic (escape/kicker) appears on the tagged cards
- **example:** Cabal Therapy: 'Target player reveals their hand and discards all cards with that name. Flashback—Sacrifice a creature.' (discard and flashback are independent)
- **suggested fix:** A discard effect on a card that also carries a set-specific mechanic, like spectacle or flashback.

### `mill-opponent` — suspect
- **current:** Puts cards from an opponent's library into their graveyard.
- **issue:** Says 'into their graveyard' but many tagged cards exile from the library instead of milling, and some hit every player
- **example:** Knacksaw Clique: 'Target opponent exiles the top card of their library'; Etali, Primal Storm: 'exile the top card of each player's library'
- **suggested fix:** Depletes an opponent's library, putting cards into their graveyard or exile.

### `creature-count-matters` — suspect
- **current:** Gets better the more creatures you control.
- **issue:** Underspecified: some cards count all creatures on the battlefield (not just yours) and don't simply 'get better' with more of yours
- **example:** Chain Reaction: 'deals X damage to each creature, where X is the number of creatures on the battlefield' (counts every creature, a symmetric wipe)
- **suggested fix:** Cares about how many creatures are on the battlefield, usually scaling with the number you control.

### `paper-compatible` — suspect
- **current:** A digital-only card whose effect would still work if printed in paper.
- **issue:** Calls it 'a digital-only card', but many tagged cards are ordinary paper printings, so the premise is contradicted
- **example:** Hallowed Priest ('Whenever you gain life, put a +1/+1 counter on this creature') is a normal paper card, not digital-only; likewise Harrowing Swarm (manifest dread)
- **suggested fix:** A card whose effect works the same way under paper Magic rules, needing no digital-only mechanics.

### `graveyard-fuel-creature` — suspect
- **current:** Exiles creature cards from a graveyard to fuel its abilities.
- **issue:** Several examples exile creature cards as a spell's cast cost, not to fuel an ability
- **example:** Makeshift Mauler: "As an additional cost to cast this spell, exile a creature card from your graveyard." (also Gorex, the Tombshell)
- **suggested fix:** Exiles creature cards from a graveyard as a cost or to power its effects.

### `untaps-self` — suspect
- **current:** Untaps itself so it can be used again.
- **issue:** "so it can be used again" overstates the purpose; several untap for reasons unrelated to reuse
- **example:** Gustcloak Runner: "Whenever this creature becomes blocked, you may untap it and remove it from combat." (untaps to dodge combat, not to reactivate a tap ability); Futurist Operative untaps to change its form
- **suggested fix:** Untaps itself.

### `counterspell-with-set-mechanic` — suspect
- **current:** A counterspell that also carries its set's keyword mechanic.
- **issue:** Some tagged cards counter abilities, not spells, so "counterspell" is too narrow
- **example:** Trickbind: "Counter target activated or triggered ability" (also Kadena's Silencer counters abilities)
- **suggested fix:** A spell that counters a spell or ability and also carries its set's keyword mechanic.

### `damage-prevention-creature` — suspect
- **current:** Prevents damage that would be dealt to a creature.
- **issue:** Underspecified: several cards redirect rather than prevent, and some protect players/all permanents, not just a creature
- **example:** Ward of Piety: "The next 1 damage that would be dealt to enchanted creature this turn is dealt to any target instead"; Gideon's Intervention prevents damage to "you and permanents you control"
- **suggested fix:** Prevents or redirects damage that would be dealt to a creature.

### `gives-hexproof` — suspect
- **current:** Grants hexproof to other permanents so opponents can't target them.
- **issue:** Grant is typically to creatures you control, and can even include YOU the player, so 'other permanents' is loose and misses the self/player case
- **example:** Veil of Summer: 'You and permanents you control gain hexproof from blue and from black until end of turn.'
- **suggested fix:** Grants hexproof to creatures you control so your opponents can't target them.

### `copy-sorcery` — suspect
- **current:** Copies an instant or sorcery spell you cast or control.
- **issue:** 'you cast or control' is too narrow; several cards copy any player's instants/sorceries
- **example:** Eye of the Storm: 'Whenever a player casts an instant or sorcery card, exile it. Then that player copies each instant or sorcery card exiled...'
- **suggested fix:** Copies an instant or sorcery spell.

### `reanimate-cast` — suspect
- **current:** Lets you cast a permanent card straight out of your graveyard.
- **issue:** overspecified to 'permanent' when some let you cast any spell type from the graveyard
- **example:** Eye of Duskmantle: 'You may play lands and cast spells from among cards in your graveyard you've surveilled this turn.' (not limited to permanents)
- **suggested fix:** Lets you cast a card straight out of your graveyard.

### `combat-ramp` — suspect
- **current:** Generates extra mana or lands by attacking or dealing combat damage.
- **issue:** Underspecified: several cards cheat any permanent (not just mana or lands) onto the battlefield from combat
- **example:** Broodcaller Scourge: "Whenever one or more Dragons you control deal combat damage to a player, you may put a permanent card with mana value less than or equal to that damage from your hand onto the battlefield."
- **suggested fix:** Produces extra mana, Treasures, or lands, or puts permanents onto the battlefield, when you attack or deal combat damage.

### `thoughtseize` — suspect
- **current:** Makes an opponent reveal their hand and you pick a card for them to discard.
- **issue:** Says opponent/discard, but many examples target any player and exile or bottom-tuck instead of discard
- **example:** Psychotic Episode: 'Target player reveals their hand... That player puts the chosen card on the bottom of their library.' Intimidation Tactics: 'Exile that card.'
- **suggested fix:** Makes a player reveal their hand so you choose a card from it to be discarded, exiled, or removed.

### `hate-regenerate` — suspect
- **current:** Prevents a creature from being regenerated, usually while destroying it.
- **issue:** Says 'a creature' but the tag also covers artifacts and lands that can't be regenerated
- **example:** Oxidize: 'Destroy target artifact. It can't be regenerated.' Pillage: 'Destroy target artifact or land. It can't be regenerated.'
- **suggested fix:** Prevents a permanent from being regenerated, usually while destroying it.

### `punisher` — suspect
- **current:** Forces an opponent to choose between two bad outcomes.
- **issue:** Says "opponent" but many punisher cards give the choice to "any player"
- **example:** Browbeat: "Any player may have Browbeat deal 5 damage to them. If no one does, target player draws three cards." (also Temporal Extortion: "any player may pay half their life")
- **suggested fix:** Offers a player a choice between two outcomes, both of which benefit you.

### `flicker-creature` — suspect
- **current:** Exiles a creature you control and returns it to the battlefield.
- **issue:** Overspecified to 'you control'; several flicker any creature, including opponents'
- **example:** Vizier of Deferment: 'you may exile target creature if it attacked or blocked this turn' (any creature); The Windy City exiles 'target creature with flying'
- **suggested fix:** Exiles a creature, usually one you control, and returns it to the battlefield.

### `burn-player-each` — suspect
- **current:** Deals damage to each creature and each player.
- **issue:** Says 'each creature' but many hit only a subset; the invariant is each player
- **example:** Whirling Catapult: 'deals 1 damage to each creature with flying and each player'; Blockbuster hits 'each tapped creature and each player'
- **suggested fix:** Deals damage to each player, and often to creatures or a subset of them, sweeping the whole table.

### `consult-cast` — suspect
- **current:** Exiles cards from your library until you hit one you may cast.
- **issue:** 'your library' is wrong for some; also misses the free cast
- **example:** Chaos Wand: 'Target opponent exiles cards from the top of their library until they exile an instant or sorcery card. You may cast that card without paying its mana cost.'
- **suggested fix:** Exiles cards from the top of a library until hitting a castable card, then lets you cast it, often without paying its mana cost.

### `poisonous` — suspect
- **current:** A creature that gives a player poison counters when it deals them damage.
- **issue:** Says "A creature" but the tag includes non-creature enablers, and poison is not always from its own damage
- **example:** Skrelv's Hive (Enchantment): "create a 1/1 colorless Phyrexian Mite artifact creature token with toxic 1"; Bloodroot Apothecary gives poison "Whenever an opponent sacrifices a noncreature token"
- **suggested fix:** Gives players poison counters, usually through infect or toxic combat damage.

### `nightveil-theft` — suspect
- **current:** Exiles cards from another player's library or hand and lets you cast them.
- **issue:** Says 'another player's' but many hit each player's library (including yours); 'cast' understates that most let you 'play' lands too
- **example:** Mezzio Mugger: 'exile the top card of each player's library. You may play those cards this turn'
- **suggested fix:** Exiles cards from one or more players' libraries or hands and lets you play or cast them yourself.

### `life-loss-matters` — suspect
- **current:** Cares about how much life a player lost this turn, not just damage dealt.
- **issue:** 'how much' overspecifies; most cards only care WHETHER a player (usually an opponent) lost life this turn, not the amount
- **example:** Drill Bit: 'Spectacle {B} ... if an opponent lost life this turn' and Point the Way / Momentum Breaker start-your-engines: 'increases ... when an opponent loses life' are boolean checks, not amounts
- **suggested fix:** Cares about whether or how much life a player lost this turn, distinct from just damage dealt.

### `cost-ignorer` — suspect
- **current:** Lets you cast a spell without paying its mana cost, or for an alternate cost instead.
- **issue:** Says only 'cast a spell' and 'you', but the tag also covers putting permanents onto the battlefield for free, sometimes for another player
- **example:** Chaos Warp: 'The owner of target permanent shuffles it into their library, then reveals the top card... if it's a permanent card, they put it onto the battlefield.' Sharuum returns an artifact from graveyard to the battlefield.
- **suggested fix:** Lets a spell or permanent be deployed without paying its mana cost, or for an alternate cost instead.

### `synergy-forest` — suspect
- **current:** Gets better when you control Forests.
- **issue:** Underspecified: many cards use or sacrifice Forests as a resource rather than passively 'getting better' from controlling them
- **example:** Goblin Clearcutter: '{T}, Sacrifice a Forest: Add three mana...'; Elven Palisade: 'Sacrifice a Forest: Target attacking creature gets -3/-0'
- **suggested fix:** Cares about Forests, rewarding you for controlling, enchanting, or sacrificing them.

### `undergrowth` — suspect
- **current:** Gets better based on how many creature cards are in your graveyard.
- **issue:** Overspecifies 'your graveyard' and 'creature cards': some tagged cards count any player's graveyard or permanent (not creature) cards
- **example:** Corpse Augur: 'creature cards in target player's graveyard'; Terror Tide: 'the number of permanent cards in your graveyard'
- **suggested fix:** Gets better based on how many creature cards are in a graveyard.

### `mulch` — suspect
- **current:** Looks at several cards off the top of your library, keeps one, and mills the rest.
- **issue:** "keeps one" is overspecified; several cards keep more than one card
- **example:** Benefaction of Rhonas: "You may put a creature card and/or an enchantment card from among them into your hand. Put the rest into your graveyard." (Beast Hunt keeps ALL creature cards)
- **suggested fix:** Looks at or mills cards from the top of your library, keeping one or more and putting the rest into your graveyard.

### `tap-fuel-power` — suspect
- **current:** Lets you tap a creature and use an amount equal to its power to fuel an effect.
- **issue:** says "a creature" (singular) but Teamwork taps any number of creatures for total power
- **example:** Crossover Collaboration: "you may tap any number of creatures you control with total power 2 or more"
- **suggested fix:** Lets you tap one or more of your creatures and uses their power to fuel or pay for an effect.

### `tutor-card` — suspect
- **current:** Searches your library for any card, with no restriction on what you can find.
- **issue:** "no restriction" is contradicted by restricted tutors carrying the tag
- **example:** Muddle the Mixture: "Transmute... Search your library for a card with the same mana value as this card" (a mana-value restriction).
- **suggested fix:** Searches your library for a card and puts it into your hand or elsewhere, usually with few restrictions on what you can find.

### `universal-type-change` — suspect
- **current:** Changes the type of every card or permanent of one kind into another type.
- **issue:** Says 'changes' the type into another, but cards mostly ADD a type in addition to existing ones, and many hit only permanents you control rather than 'every' one
- **example:** Senator Peacock: 'Artifacts you control are Clues in addition to their other types'; Natural Affinity: 'All lands become 2/2 creatures... They're still lands.'
- **suggested fix:** Gives every permanent of one kind an additional type, usually on top of its existing types.

### `turn-face-up-trigger-self` — suspect
- **current:** Triggers an effect when this creature is turned face up from morph, megamorph, or disguise.
- **issue:** Overspecified as 'creature'; some carriers are noncreature permanents
- **example:** Concealed Weapon (Artifact — Equipment): 'When this Equipment is turned face up, attach it to target creature you control.'
- **suggested fix:** Triggers an effect when it is turned face up from morph, megamorph, or disguise.

### `flowstone` — suspect
- **current:** Grants a creature +N/-N or -N/+N, boosting one stat while cutting the other.
- **issue:** Adds an unsupported '-N/+N' option; every carrier is +N/-N (power up, toughness down)
- **example:** Sangrite Backlash: 'Enchanted creature gets +3/-3.'; Merfolk Coralsmith: '{1}: This creature gets +1/-1 until end of turn.'
- **suggested fix:** Grants a creature +N/-N, boosting its power while cutting its toughness.

### `card-types-in-graveyard-matter` — suspect
- **current:** Gets stronger based on how many different card types are in your graveyard.
- **issue:** Narrow: several tagged cards don't get stronger themselves; effect is usually a 4+ types (delirium) threshold, not a gradual scale
- **example:** Violent Urge: 'Delirium — ...that creature gains double strike' buffs a targeted creature, not itself; Fear of Burning Alive uses delirium to redirect damage
- **suggested fix:** Cares about how many different card types are among cards in your graveyard, often rewarding four or more.

### `extra-untap` — suspect
- **current:** Untaps many permanents at once or adds an extra untap step.
- **issue:** 'adds an extra untap step' is inaccurate; cards add combat phases or untap during other players' existing untap steps, none add an untap step
- **example:** Drumbellower: 'Untap all creatures you control during each other player's untap step'; Aurelia untaps then adds a combat phase, not an untap step
- **suggested fix:** Untaps many of your permanents at once, or lets them untap during other players' untap steps too.

### `synergy-blocker` — suspect
- **current:** Rewards or boosts creatures that block.
- **issue:** 'Rewards or boosts' undersells cards that merely enable extra blocks or care about blockers without buffing
- **example:** Echo Circlet: 'Equipped creature can block an additional creature each combat.'; Vizier of Deferment exiles a creature 'if it attacked or blocked this turn' rather than boosting it
- **suggested fix:** Cares about creatures that block, often boosting them or letting them block more.

### `conjure-named` — suspect
- **current:** Creates a card with a specific name into your library, hand, graveyard, or battlefield.
- **issue:** Says destination is 'your' zone, but the conjured card can go into another player's zone
- **example:** Juggernaut Peddler: 'that player exiles it and conjures a card named Juggernaut into their hand.'
- **suggested fix:** Creates a card with a specific name in a library, hand, graveyard, or on the battlefield.

### `pacifism` — suspect
- **current:** Removal that stops a creature from attacking and blocking without destroying it.
- **issue:** Overspecified: several cards only grant defender, which still lets the creature block
- **example:** Sky Tether: "Enchanted creature has defender and loses flying" (defender stops attacking but the creature can still block)
- **suggested fix:** Neutralizes a creature in combat, usually keeping it from attacking and blocking, without destroying it.

### `conjure-creature` — suspect
- **current:** Conjures a new creature card into your library, hand, or the battlefield.
- **issue:** Zone list omits exile, where some conjure effects put the card
- **example:** Dazzling Flameweaver: "conjure a random card from Dazzling Flameweaver's spellbook into exile"
- **suggested fix:** Conjures a creature card from nowhere into a zone such as your hand, library, battlefield, or exile.

### `hate-color-choose` — suspect
- **current:** Grants protection from a color of your choice.
- **issue:** Narrowed to protection, but many cards use hexproof-from-color or damage prevention/redirection, and the chooser isn't always you
- **example:** Skrelv, Defector Mite: "Choose a color. Another target creature you control gains toxic 1 and hexproof from that color" (hexproof, not protection); Pale Wayfarer chooses "color of its controller's choice"
- **suggested fix:** Defends against a color of your choice, usually by granting protection from that color.

### `catch-up` — suspect
- **current:** Rewards you with an extra effect when you're behind in lands, life, cards, or creatures.
- **issue:** Only covers 'you're behind'; misses the large 'reward/target the player who's ahead (most life)' cases
- **example:** Scourge of the Throne: 'Dethrone (Whenever this creature attacks the player with the most life... put a +1/+1 counter on it.)'; Seraphic Greatsword rewards attacking 'the player with the most life'
- **suggested fix:** Gives you an extra effect when you're behind in lands, life, cards, or creatures, or when you attack the player who's ahead.

### `hate-instant` — suspect
- **current:** Punishes or steals value from opponents' instant and sorcery spells.
- **issue:** Overspecified to 'opponents' and 'steals'; many trigger on any player's instants/sorceries or just answer/reduce them
- **example:** April O'Neil, Human Element: 'Whenever a player casts an artifact, instant, or sorcery spell...'; Forethought Amulet reduces damage from an instant or sorcery source (doesn't steal)
- **suggested fix:** Punishes, answers, or steals value from instant and sorcery spells.

### `coin-flip` — suspect
- **current:** Flips a coin, with the effect depending on whether you win or lose the flip.
- **issue:** Overspecified: many carriers flip multiple coins or resolve by heads/tails, not a single win/lose flip
- **example:** Rakdos, the Showstopper: 'flip a coin for each creature... Destroy each creature whose coin comes up tails'; Ral Zarek: 'Flip five coins... where X is the number of coins that came up heads'
- **suggested fix:** Flips one or more coins, with an effect determined by the results.

### `alternate-win-condition` — suspect
- **current:** Lets you win the game, or makes an opponent lose it, outside of reducing life to zero.
- **issue:** says 'an opponent' but the cards make 'a player' lose the game; also a couple of tagged cards win via straight damage
- **example:** Vraska, Golgari Queen emblem: 'Whenever a creature you control deals combat damage to a player, that player loses the game.'
- **suggested fix:** Lets you win the game or makes a player lose it, outside the normal path of dealing lethal damage.

### `tutor-mv` — suspect
- **current:** Searches your library for a card with a specific mana value.
- **issue:** Overspecified: most cards search 'mana value X or less' (a cap), not a specific/exact value
- **example:** Green Sun's Zenith: 'Search your library for a green creature card with mana value X or less'; only Muddle the Mixture uses 'same mana value'
- **suggested fix:** Searches your library for a card by mana value, usually one at or below a set amount.

### `hate-sorcery` — suspect
- **current:** Lets you cast an instant or sorcery card taken or exiled from an opponent.
- **issue:** Too narrow; only covers the steal-and-cast subset and misses the answer/punish (hate) cards in the tag
- **example:** Forethought Amulet: 'If an instant or sorcery source would deal 3 or more damage to you, it deals 2 damage to you instead.' Also Transcantation ('Target instant or sorcery spell becomes a copy of Lightning Bolt') and April O'Neil (payoff for any player casting a spell) do not steal-cast anything.
- **suggested fix:** Answers or exploits instant and sorcery spells, often by casting or copying an opponent's or blunting its effect.

### `man-o-war` — suspect
- **current:** A creature that returns a target creature to its owner's hand when it enters.
- **issue:** Overspecified: not always a creature, and several return any permanent rather than only a creature
- **example:** Inscription of Insight is a 'Sorcery' (not a creature): 'Return up to two target creatures to their owners' hands.' Venser, Shaper Savant returns 'target spell or permanent to its owner's hand.'
- **suggested fix:** Returns a target creature or other permanent to its owner's hand, usually as a creature enters the battlefield.

### `overrun` — suspect
- **current:** Gives creatures you control +X/+X and trample until end of turn.
- **issue:** Not always +X/+X; several grant only a power boost or just trample
- **example:** Umaro, Raging Yeti: 'Other creatures you control get +3/+0 and gain trample until end of turn.' Overwhelming Victory gives '+X/+0'; The Crowd Goes Wild only grants trample to counter-bearing creatures.
- **suggested fix:** Gives creatures you control a stat boost and trample until end of turn.

### `typal-wizard` — suspect
- **current:** Cares about the number of Wizards you control or rewards you for casting Wizard spells.
- **issue:** Underspecified: misses cards that reward controlling Wizards without counting or casting them
- **example:** Azami, Lady of Scrolls: 'Tap an untapped Wizard you control: Draw a card'; Summon: Esper Ramuh: 'Wizards you control get +1/+0'
- **suggested fix:** Cares about Wizards you control or rewards you for having and casting them.

### `maro-sorcerer` — suspect
- **current:** A creature whose power and toughness scale with how many lands, or a land type, you control.
- **issue:** 'A creature' overspecifies; tagged cards include non-creatures whose land-scaling effect isn't the card's own P/T
- **example:** Glistening Dawn (Sorcery): 'Incubate X twice, where X is the number of lands you control'; Lashwrithe (Equipment): 'Equipped creature gets +1/+1 for each Swamp you control'
- **suggested fix:** Its power, toughness, or a bonus it produces scales with how many lands, or a land type, you control.

### `life-total-matters-self` — suspect
- **current:** Cares whether your own life total is above or below a certain threshold.
- **issue:** 'above or below a certain threshold' misses cards that read your life total as a scaling value rather than checking a threshold
- **example:** Aettir and Priwen: 'Equipped creature has base power and toughness X/X, where X is your life total'
- **suggested fix:** Cares about your own life total, whether checking a threshold or using it as a value.

### `pseudo-fog` — suspect
- **current:** Stops an entire combat phase by means other than damage prevention, such as tapping attackers.
- **issue:** "Stops an entire combat phase" overstates it; many pseudo-fogs only tap or blank part of a combat
- **example:** Blustersquall — "Tap target creature you don't control" (single creature, not the whole phase); Naya Charm — "Tap all creatures target player controls"
- **suggested fix:** Blanks or deters a combat by means other than damage prevention, such as tapping a player's attackers before they can swing.

### `discard-to-exile` — suspect
- **current:** Exiles cards from an opponent's hand instead of sending them to the graveyard.
- **issue:** Says "opponent" and hand-only, but many are "target player" and also hit the graveyard
- **example:** Karn Liberated: "+4: Target player exiles a card from their hand"; Agonizing Remorse: "You choose a nonland card from it or a card from their graveyard. Exile that card."
- **suggested fix:** Exiles cards from a target player's hand, and sometimes their graveyard, instead of putting them into the graveyard.

### `scales-with-multiple` — suspect
- **current:** Gets stronger the more copies of itself you have.
- **issue:** 'Gets stronger' is too narrow; several cards instead cast extra free copies or use copies as a cost rather than gaining stats
- **example:** Surging Sentinels: 'Ripple 4 (When you cast this spell, you may reveal the top four cards... cast spells with the same name... without paying their mana costs)'; Skoa Grandeur discards another Skoa
- **suggested fix:** Rewards you for having or drawing multiple copies of itself.

### `tap-fuel-artifact` — suspect
- **current:** Lets you tap untapped artifacts you control as a cost to power an ability.
- **issue:** Says only 'artifacts' and 'ability', but many tap creatures/lands too and use it as an additional cost to cast a spell
- **example:** Guardian of the Great Door: 'As an additional cost to cast this spell, tap four untapped artifacts, creatures, and/or lands you control'; waterbend cards tap artifacts and creatures
- **suggested fix:** Lets you tap untapped artifacts you control, sometimes creatures or lands too, as a cost to cast a spell or activate an ability.

### `free-discard-outlet` — suspect
- **current:** Lets you discard a card as a cost, with no mana cost or per-turn limit.
- **issue:** "no per-turn limit" is contradicted by some cards, and the discard is sometimes restricted to specific card types
- **example:** Chainer, Nightmare Adept: "Discard a card: You may cast a creature spell from your graveyard this turn. Activate only once each turn." (also Vampire Hounds requires a creature card)
- **suggested fix:** Discards a card as a cost with no mana required, fueling an ability or effect.

### `gives-ward` — suspect
- **current:** Grants ward to a creature, forcing opponents to pay a cost to target it.
- **issue:** grants ward to permanents in general, not only creatures
- **example:** Elder Owyn Lyons: "Artifacts you control have ward {1}."
- **suggested fix:** Grants ward to one or more permanents, making opponents pay a cost or lose their spell or ability that targets them.

### `leaves-battlefield-trigger` — suspect
- **current:** Triggers an effect whenever a permanent leaves the battlefield.
- **issue:** triggers are mostly on creatures, frequently ones you control, so "a permanent" is broader than the real cards
- **example:** The Ozolith: "Whenever a creature you control leaves the battlefield..."; Vela the Night-Clad: "Whenever Vela or another creature you control leaves the battlefield..."
- **suggested fix:** Triggers an effect whenever a creature or other permanent, often one you control, leaves the battlefield.

### `tapper-artifact` — suspect
- **current:** Taps down a target artifact, keeping it from using its tap abilities.
- **issue:** 'keeping it from using its tap abilities' is editorial and often false (most are one-shot taps); also usually taps creatures/lands too
- **example:** Auriok Transfixer: '{W}, {T}: Tap target artifact.' (one-time tap, target untaps normally next turn); Malicious Advice: 'Tap X target artifacts, creatures, and/or lands.'
- **suggested fix:** Taps a target artifact, and often creatures or lands as well.

### `haven` — suspect
- **current:** Exiles your own permanents to keep them safe, then lets you bring them back later.
- **issue:** Overspecified to 'your own' and 'keep them safe'; haven often exiles opponents' permanents as temporary removal
- **example:** Battle at the Helvault: 'For each player, exile up to one target non-Saga, nonland permanent that player controls until this Saga leaves the battlefield.' (also Parallax Wave exiles any target creature)
- **suggested fix:** Exiles a permanent until this leaves the battlefield, then returns it, protecting your own or temporarily removing an opponent's.

### `synergy-basic` — suspect
- **current:** Rewards you for controlling basic lands, often scaling with how many you have.
- **issue:** Overspecified to 'rewards you for controlling'; many cards just use basics as a cost or search target rather than rewarding control
- **example:** Hermit Druid: 'Reveal cards from the top of your library until you reveal a basic land card...'; Thirst for Discovery discards a basic land as a cost
- **suggested fix:** Cares about basic lands, often rewarding or scaling with how many you control.

### `counter-fuel-charge` — suspect
- **current:** Stores charge counters you remove to power an activated ability.
- **issue:** Overspecifies 'activated ability'; several fuel triggered abilities instead
- **example:** Sun Droplet: 'At the beginning of each upkeep, you may remove a charge counter from this artifact. If you do, you gain 1 life.' (Immard likewise removes a counter on a triggered ability)
- **suggested fix:** Stores charge counters that you remove to power one of its abilities.

### `counter-doubler` — suspect
- **current:** Doubles the number of counters placed or tokens created.
- **issue:** "counters placed" undersells cards that double counters already on a permanent, not just newly placed ones
- **example:** Visions of Dominance: "Put a +1/+1 counter on target creature, then double the number of +1/+1 counters on it."
- **suggested fix:** Doubles the number of counters put on or already on a permanent, or the number of tokens created.

### `splits-on-death` — suspect
- **current:** Creates two or more creature tokens when a creature dies.
- **issue:** "two or more" is overspecified; some tagged cards make a single token when a creature dies
- **example:** Ochre Jelly: "When this creature dies, if it had two or more +1/+1 counters on it, create a token that's a copy of it..." (one token)
- **suggested fix:** Creates one or more creature tokens when a creature dies.

### `disintegrate` — suspect
- **current:** Deals damage that exiles the creature instead of letting it die.
- **issue:** Underspecified: the exile-on-death effect can apply to any permanent, not just creatures
- **example:** Torch the Tower: 'deals 2 damage to target creature or planeswalker... If a permanent dealt damage by Torch the Tower would die, exile it instead.'
- **suggested fix:** Deals damage that exiles the creature or permanent instead of letting it die.

### `synergy-activated-ability` — suspect
- **current:** Cares about or fuels the activation of abilities on other cards.
- **issue:** "on other cards" overspecifies; several payoffs just fuel/reward activating abilities generally, not specifically abilities on other cards
- **example:** Omen Hawker: "{T}: Add {C}{U}. Spend this mana only to activate abilities." (any ability, not scoped to other cards)
- **suggested fix:** Cares about or fuels the activation of abilities, often by restricting mana to that use.

### `gains-hexproof` — suspect
- **current:** Gives itself hexproof so it can't be targeted by opponents.
- **issue:** opens with "Gives" for a gains/-self tag; the sibling distinction reserves "gives" for granting to others
- **example:** Reaper of the Wilds: "{1}{G}: This creature gains hexproof until end of turn." (self, not granted to others)
- **suggested fix:** Gains hexproof itself so it can't be targeted by opponents' spells or abilities.

### `wish` — suspect
- **current:** Lets you bring in a card you own from outside the game and put it into your hand.
- **issue:** Overspecifies 'into your hand' and 'you own'; some play the card directly or the card is owned by an opponent
- **example:** Frontier Explorer: '{3}, {T}: Until end of turn, you may play one basic Plains card from outside the game' (plays it, not to hand). Last-Minute Chopping: opponent 'put a card they own from outside the game into your hand'
- **suggested fix:** Brings a card from outside the game into the game, usually into your hand.

### `keyword-errata-flash` — suspect
- **current:** Has flash, printed before flash existed as an official keyword.
- **issue:** 'Has flash' misses cards that grant flash rather than having it themselves
- **example:** Vernal Equinox: 'Any player may cast creature and enchantment spells as though they had flash.'
- **suggested fix:** Uses flash (cast at instant speed) or grants it, with wording from before flash was an official keyword.

### `mill-each` — suspect
- **current:** Mills cards from every player's library into their graveyard at once.
- **issue:** 'every player ... at once' overstates it; some mill only one specific player, repeatedly
- **example:** Mesmeric Orb: 'Whenever a permanent becomes untapped, that permanent's controller mills a card' (only that one player, per untap).
- **suggested fix:** Mills cards from each player's library into their graveyard.

### `hate-target` — suspect
- **current:** Punishes players for targeting your permanents, or protects them from being targeted.
- **issue:** Narrows to 'your permanents' and to punish/protect, but many trigger on ANY spell/ability (including your own) and simply generate value; Cowardice hits any creature, not just yours
- **example:** Cephalid Illusionist: 'Whenever this creature becomes the target of a spell or ability, mill three cards' (a self-targeting combo, not punishment); Cowardice returns ANY targeted creature
- **suggested fix:** Cares when a permanent becomes the target of a spell or ability, often punishing the source or gaining value from it.

### `mana-fix` — suspect
- **current:** Lets your lands tap for additional colors of mana.
- **issue:** Says 'your lands' and only 'tap for additional colors,' but several affect ALL lands and work by granting land types, not just extra colors
- **example:** Blanket of Night: 'Each land is a Swamp in addition to its other land types' (affects every player's lands); Stormtide Leviathan: 'All lands are Islands'
- **suggested fix:** Fixes your mana by letting lands tap for more colors or by giving lands additional land types.

### `type-errata-viashino` — suspect
- **current:** A creature that was errata'd from Viashino to Lizard in Modern Horizons 3.
- **issue:** Specific set attribution 'in Modern Horizons 3' is an unverified and likely incorrect claim; the Viashino-to-Lizard errata was part of a broad creature-type update, not tied to MH3
- **example:** Viashino Lashclaw is now 'Creature — Lizard Warrior' (formerly Viashino)
- **suggested fix:** A creature whose creature type was errata'd from Viashino to Lizard.

### `unheroic` — suspect
- **current:** Rewards you for targeting an opponent, something they control, or cards in their graveyard.
- **issue:** Frames the tag purely as commit-a-crime (targeting opponents/their stuff), but several cards reward targeting any creature including your own, and one punishes targeting
- **example:** Storm, Windrider: 'Whenever you cast a spell that targets one or more creatures, those creatures gain flying'; Cowardice: 'Whenever a creature becomes the target of a spell or ability, return that creature to its owner's hand.'
- **suggested fix:** Cares about spells or abilities that target, especially targeting an opponent or their permanents (committing a crime).

### `mana-storage` — suspect
- **current:** Stores mana as counters or tokens so you can spend it later.
- **issue:** 'as counters or tokens' misses the cards that store mana by keeping unspent mana across phases
- **example:** Fangorn, Tree Shepherd: 'You don't lose unspent green mana as steps and phases end.'; Omnath: 'If you would lose unspent mana, that mana becomes black instead.'
- **suggested fix:** Saves mana for later, whether as Treasure or mana tokens, counters, or by keeping unspent mana across phases.

### `regrowth-any` — suspect
- **current:** Returns a card of any type from your graveyard to your hand.
- **issue:** Says 'your graveyard' and 'your hand', but some cards return from any graveyard to its owner's hand or have each player return their own cards
- **example:** Naya Charm: 'Return target card from a graveyard to its owner's hand.'; Sail into the West: 'each player returns up to two cards from their graveyard to their hand'
- **suggested fix:** Returns a card of any type from a graveyard to its owner's hand.

### `theft-artifact` — suspect
- **current:** Lets you take control of an opponent's artifact, sometimes temporarily.
- **issue:** Overspecified to 'opponent's'; many target any artifact
- **example:** Pyreswipe Hawk: 'gain control of up to one target artifact for as long as you control this creature'; Infernal Captor: 'gain control of target artifact or creature until end of turn'
- **suggested fix:** Lets you gain control of an artifact, usually an opponent's, sometimes only temporarily.

### `graveyard-seal` — suspect
- **current:** Disrupts graveyards, exiling cards or blocking interaction to shut down reanimation.
- **issue:** 'to shut down reanimation' narrows a broad graveyard-hate effect that answers all graveyard strategies
- **example:** Ground Seal: 'Cards in graveyards can't be the targets of spells or abilities'; Yixlid Jailer: 'Cards in graveyards lose all abilities'
- **suggested fix:** Shuts down graveyards by exiling their cards, blocking them from being used, or stripping their abilities.

### `staple-with-set-s-mechanic` — suspect
- **current:** A common card built around one of its set's featured mechanics.
- **issue:** "common card" reads as the rarity Common; the tag means a widely-played staple, and these cards aren't all Common rarity
- **example:** Bake into a Pie: "Destroy target creature. Create a Food token." (an uncommon, not a common)
- **suggested fix:** A widely played, iconic card that showcases one of its set's featured mechanics.

### `counterspell-ability` — suspect
- **current:** Counters a target activated or triggered ability on the stack.
- **issue:** overspecified as "a target" ability; several tagged cards counter ALL abilities (or also spells), not a single targeted one
- **example:** Glen Elendra's Answer: "Counter all spells your opponents control and all abilities your opponents control."
- **suggested fix:** Counters an activated or triggered ability on the stack rather than a spell.

### `hate-enchantment` — suspect
- **current:** Destroys, counters, or protects against enchantments.
- **issue:** Underspecified: omits the tax/punish answers common to the tag
- **example:** Aura Barbs: "Each enchantment deals 2 damage to its controller..."; Aura Flux: "Other enchantments have 'At the beginning of your upkeep, sacrifice this enchantment unless you pay {2}.'" (neither destroys, counters, nor protects)
- **suggested fix:** Answers enchantments by destroying, countering, taxing, punishing, or protecting against them.

### `copy-legendary` — suspect
- **current:** Creates a nonlegendary token copy of a permanent you control, so you get to keep it.
- **issue:** 'so you get to keep it' is false for the many copies that self-sacrifice, and 'a permanent you control' misses copies of opponents' creatures
- **example:** Tempestra, Dame of Games: 'Create a token that's a copy of another target creature you control, except it isn't legendary. It gains haste. Sacrifice it at the beginning of the next end step.' (also Ember Island Production copies 'a creature an opponent controls')
- **suggested fix:** Creates a token copy of a creature that isn't legendary, sidestepping the legend rule.

### `legendary-team-up` — suspect
- **current:** A legendary creature that teams up two named characters on one card.
- **issue:** Not always a creature; some members are noncreature permanents
- **example:** The Belligerent and Useless Island (Legendary Artifact Land — Vehicle Island), not a creature
- **suggested fix:** A legendary permanent whose name joins two characters with 'and'.

### `typal-elemental` — suspect
- **current:** Rewards you for controlling or having Elemental creatures.
- **issue:** Cares about Elemental spells and cards too, not only creatures you control
- **example:** Brighthearth Banneret: 'Elemental spells and Warrior spells you cast cost {1} less'; Sunflare Shaman counts 'Elemental cards in your graveyard'
- **suggested fix:** Rewards you for casting or controlling Elementals.

### `untapper-artifact` — suspect
- **current:** Untaps a target artifact.
- **issue:** Overspecifies 'a target'; some members untap all your artifacts, not one target
- **example:** Unwinding Clock: 'Untap all artifacts you control during each other player's untap step'
- **suggested fix:** Untaps one or more artifacts.

### `deal-with-the-devil` — suspect
- **current:** A black enchantment with a powerful effect and a serious, potentially game-losing drawback.
- **issue:** 'black' is overspecified; not all are black enchantments
- **example:** Nine Lives — 'When this enchantment leaves the battlefield, you lose the game.' is a WHITE enchantment
- **suggested fix:** An enchantment with a powerful effect and a serious, potentially game-losing drawback.

### `remove-counters-other` — suspect
- **current:** Removes counters from opponents' permanents or removes a player's poison counters.
- **issue:** 'opponents' permanents' is overspecified; many remove from any/all permanents
- **example:** Aether Snap — 'Remove all counters from all permanents and exile all tokens.' (hits your own too); Gremlin Mine and Lonely End target any artifact/planeswalker
- **suggested fix:** Removes counters from a permanent, or removes a player's poison counters.

### `creature-type-name` — suspect
- **current:** A creature whose name is made up of creature types.
- **issue:** "made up of creature types" is too strong; the name only needs to contain one of the card's own creature types, not be composed entirely of type words
- **example:** Mystic Snake (type Snake) - "Mystic" is not a creature type, so the name is not "made up of creature types"; likewise Sand Golem and Kavu Primarch
- **suggested fix:** A creature whose name includes one of its own creature types.

### `impact-effect` — suspect
- **current:** Deals damage or makes a player lose life whenever a creature you control enters.
- **issue:** overspecified: not always "a creature you control," and the damage isn't limited to players
- **example:** Pandemonium: "Whenever a creature enters, that creature's controller may have it deal damage..." (any creature, any controller); Forerunner of the Empire deals 1 damage to each creature, not a player
- **suggested fix:** Deals damage or causes life loss whenever a creature enters, usually one you control.

### `leaves-graveyard-trigger` — suspect
- **current:** Triggers an effect whenever a card leaves your graveyard.
- **issue:** says 'your graveyard' but the tag also covers triggers on any player's graveyard
- **example:** Erebos's Titan: 'Whenever a creature card leaves an opponent's graveyard, you may discard a card.'
- **suggested fix:** Triggers an effect whenever a card leaves a graveyard.

### `extract` — suspect
- **current:** Exiles cards from a library, removing them from the game rather than just discarding them.
- **issue:** 'removing them from the game rather than just discarding them' is a confusing/incorrect framing (none are discard alternatives, and many exile from an opponent's library, not just a generic 'a library')
- **example:** Sealed Fate: 'Look at the top X cards of target opponent's library. Exile one of those cards...'; Neverending Torment: 'Search target player's library... and exile them.'
- **suggested fix:** Exiles cards from a player's library, either your own or an opponent's.

### `counter-preservation-self` — suspect
- **current:** Moves its own counters onto another creature you control when it dies.
- **issue:** 'you control' is overspecified; several cards put counters on ANY target creature, not just yours
- **example:** Scolding Administrator: 'When this creature dies, if it had counters on it, put those counters on up to one target creature.' (no 'you control'); Arcbound Wanderer modular targets 'target artifact creature'
- **suggested fix:** When it dies, moves its own counters onto another creature.

### `ball-lightning` — suspect
- **current:** A creature that hits hard with haste, then is gone by end of turn.
- **issue:** Calls it 'a creature' but most carriers are spells/permanents that CREATE the haste token, not creatures themselves
- **example:** Elemental Appeal (Sorcery): 'Create a 7/1 red Elemental creature token with trample and haste. Exile it at the beginning of the next end step.'
- **suggested fix:** A hard-hitting creature with haste that is sacrificed or exiled at end of turn, or a spell that makes one.

### `combat-timing-restriction` — suspect
- **current:** A spell you can only cast during a specific step of combat.
- **issue:** 'a specific step' is overspecified; some cards restrict casting to combat generally, not a named step
- **example:** Angelic Favor (Instant): 'Cast this spell only during combat.'
- **suggested fix:** A spell you can only cast during combat, often only in a specific combat step.

### `gains-protection` — suspect
- **current:** Gains protection from a color or card type, often until end of turn.
- **issue:** Protection sources broader than 'color or card type' (also names, artifacts), and some cards GRANT it to others rather than gaining it themselves
- **example:** Eight-and-a-Half-Tails: 'Target permanent you control gains protection from white'; Knight in _____ Armor: 'protection from names that start with...'
- **suggested fix:** Has or grants protection from something like a color, card type, or artifacts, often until end of turn.

### `infusion` — suspect
- **current:** Triggers a bonus effect if you gained life this turn.
- **issue:** Not always a 'triggered bonus'; some are static buffs or a drawback you avoid by gaining life
- **example:** Tragedy Feaster: 'Infusion — At the beginning of your end step, sacrifice a permanent unless you gained life this turn.'
- **suggested fix:** Cares about whether you gained life this turn, usually granting a bonus if you did.

### `synergy-poison` — suspect
- **current:** Gets stronger or unlocks an effect when an opponent has poison counters.
- **issue:** Overspecifies "opponent"; several tagged cards care about any player's poison, not just opponents
- **example:** Hidetsugu's Poison Rite: "Target player with exactly seven poison counters loses the game." (any player, and Whispering Specter keys off the poisoned player's own counters)
- **suggested fix:** Rewards poison counters, growing stronger or unlocking effects when a player (usually an opponent) is poisoned.

### `tutor-land-any` — suspect
- **current:** Searches your library for any land card, with a restriction or cost on the effect.
- **issue:** "any" means any land type (not just basics); the added "with a restriction or cost" is overspecified since many tagged cards have none
- **example:** Ulvenwald Hydra: "When this creature enters, you may search your library for a land card, put it onto the battlefield tapped" (no restriction or cost)
- **suggested fix:** Searches your library for any land card, not limited to basic lands.

### `blood-artist-ability` — suspect
- **current:** Whenever a creature dies, an opponent loses life and you often gain life.
- **issue:** says 'an opponent' but the flagship card drains 'target player' (any player), so it under/mis-states who loses life
- **example:** Blood Artist: 'Whenever this creature or another creature dies, target player loses 1 life and you gain 1 life.'
- **suggested fix:** Whenever a creature dies, a player loses life and you often gain that much life.

### `pseudo-proliferate` — suspect
- **current:** Doubles or adds extra counters on permanents, working like proliferate without using that keyword.
- **issue:** Says 'on permanents' but several also add/double counters on players (and 'you'), not just permanents
- **example:** Powerful Broker: '{T}: For each kind of counter on target permanent or player, give that permanent or player another counter of that kind.' Aetheric Amplifier: 'Double the number of each kind of counter you have.'
- **suggested fix:** Adds to or doubles the counters already on a permanent or player, mimicking proliferate without using that keyword.

### `pwdeck-sidekick` — suspect
- **current:** Gets stronger or gains an ability while you control the matching named planeswalker.
- **issue:** Undersells the tag: some carriers don't benefit from the walker but support/interact with it
- **example:** Gideon's Company: '{3}{W}: Put a loyalty counter on target Gideon planeswalker.' (buffs the walker, doesn't get stronger from it); Keral Keep Disciples triggers 'Whenever you activate a loyalty ability of a Chandra planeswalker.'
- **suggested fix:** Synergizes with a specific named planeswalker, getting stronger, unlocking an ability, or supporting it when you control one.

### `restock-creature` — suspect
- **current:** Puts a creature card from your graveyard back on top of your library.
- **issue:** Overspecified to 'on top': roughly half instead shuffle creatures into the library, and it's usually multiple cards
- **example:** Piper's Melody / Renewing Touch: 'Shuffle any number of target creature cards from your graveyard into your library'; Elvish Soultiller shuffles all of a chosen type in.
- **suggested fix:** Puts creature cards from your graveyard back into your library, on top or shuffled in.

### `unique-type-exclusion` — suspect
- **current:** Refers to a 'non-[type]' exclusion that no other card uses for that type.
- **issue:** Describes the tag's meta rarity instead of what a card does, and the 'no other card uses' uniqueness claim isn't reliably true
- **example:** Anim Pakal 'attack with one or more non-Gnome creatures' and Gideon, the Oathsworn 'attack with two or more non-Gideon creatures' both act on a non-[type] exclusion; the description explains the tag, not the effect.
- **suggested fix:** Cares about or affects permanents that aren't a specific creature or land type (a 'non-[type]' exclusion).

### `counterspell-exile` — suspect
- **current:** Counters a spell and exiles it instead of letting it go to the graveyard.
- **issue:** Some cards don't exile the countered spell itself but exile same-named copies from library/hand while the spell still hits the graveyard
- **example:** Quash: 'Counter target instant or sorcery spell. Search its controller's graveyard, hand, and library for all cards with the same name as that spell and exile them.'
- **suggested fix:** Counters a spell and exiles it, or exiles other copies of it, instead of using the graveyard.

### `painland` — suspect
- **current:** A land that deals damage to you when you tap it for mana.
- **issue:** Several tagged lands make you pay life rather than dealing damage, which the description omits (life payment and damage are distinct in the rules)
- **example:** Boseiju, Who Shelters All: '{T}, Pay 2 life: Add {C}.'
- **suggested fix:** A land that deals damage to you or makes you pay life when you tap it for mana.

### `lobotomy` — suspect
- **current:** Exiles every copy of a chosen card name from a player's hand, library, and graveyard.
- **issue:** 'every copy' and 'chosen name' overspecify; some cap the count and some derive the name from a countered spell or destroyed land rather than free choice
- **example:** Unmoored Ego: 'Search target opponent's graveyard, hand, and library for up to four cards with that name and exile them'; Counterbore counters a spell then exiles all cards with THAT spell's name
- **suggested fix:** Searches a player's graveyard, hand, and library for cards with a named card's name and exiles them.

### `potentially-free` — suspect
- **current:** Can be cast without paying its mana cost if a condition is met.
- **issue:** 'without paying its mana cost' fits only the alt-cost subset; half the examples are affinity, which is a cost reduction (you still pay the reduced cost, possibly zero), not a free cast
- **example:** Myr Enforcer: 'Affinity for artifacts (This spell costs {1} less to cast for each artifact you control.)'
- **suggested fix:** Can potentially cost no mana to cast when its cost reduction or free-cast condition is met.

### `ransom` — suspect
- **current:** Forces a player to sacrifice a permanent unless they pay a cost.
- **issue:** Overspecified to sacrifice-for-mana; several cards destroy the permanent or demand life/discard instead
- **example:** Erosion: 'destroy that land unless that player pays {1} or 1 life.'; Coral Net: 'sacrifice this creature unless they pay {2}' via discard on others
- **suggested fix:** Makes a player sacrifice or lose a permanent unless they pay a cost.

### `spite-damage` — suspect
- **current:** Deals damage in retaliation whenever it is dealt damage.
- **issue:** "it is dealt damage" is too self-focused; many make ANOTHER creature or the player retaliate, and some trigger off dealing (not receiving) damage
- **example:** Arcbond: "Choose target creature. Whenever that creature is dealt damage this turn, it deals that much damage to each other creature and each player."; Eye for an Eye retaliates when a source deals damage to YOU
- **suggested fix:** Deals retaliatory damage back when it, another creature, or you is dealt damage.

### `whirlpool` — suspect
- **current:** Shuffles your hand and graveyard into your library, then draws you a fresh hand.
- **issue:** says "your" but most of these hit EACH player, not just you
- **example:** Game Plan: "Each player shuffles their hand and graveyard into their library, then draws seven cards." (also Echo of Eons, Teferi's Puzzle Box)
- **suggested fix:** Shuffles hands and graveyards into their libraries, then everyone draws a fresh hand.

### `young-pyromancer-ability` — suspect
- **current:** Creates a creature token whenever you cast an instant or sorcery spell.
- **issue:** limits it to instant or sorcery, but half the cards trigger on any noncreature spell (and Magecraft ones also on copies)
- **example:** Kykar, Zephyr Awakener: "Whenever you cast a noncreature spell ... Create a 1/1 white Spirit creature token" (also Namor, Tellah)
- **suggested fix:** Creates a creature token whenever you cast a noncreature spell, often an instant or sorcery.

### `landhome` — suspect
- **current:** A creature bound to a land type, needing it to attack or to stay on the battlefield.
- **issue:** Implies the controller needs the land to attack; actually the DEFENDING player must control the land type
- **example:** Red Cliffs Armada: 'This creature can't attack unless defending player controls an Island'
- **suggested fix:** A creature that can attack only if the defending player controls a certain land type, and is sacrificed if its controller has none, an old 'landhome' rule.

### `self-life-loss-matters` — suspect
- **current:** Rewards you when you gained or lost life during the turn.
- **issue:** Over-broadens to 'gained or lost'; the tag is specifically about YOUR life LOSS, and the wording no longer distinguishes it from a life-gain sibling
- **example:** First Response: 'if you lost life last turn, create a 1/1 white Soldier'; Essence Channeler: 'As long as you've lost life this turn, this creature has flying and vigilance'
- **suggested fix:** Rewards you when you've lost life this turn.

### `theft-permanent` — suspect
- **current:** Takes control of an opponent's permanent for yourself.
- **issue:** Overspecified as 'an opponent's permanent'; most cards target ANY permanent (any player's, sometimes your own) or redistribute control, not strictly an opponent's
- **example:** Zealous Conscripts: 'gain control of target permanent'; Shifting Loyalties: 'Exchange control of two target permanents that share a card type'
- **suggested fix:** Takes control of a target permanent, usually one another player controls.

### `combat-ping` — suspect
- **current:** Deals a small amount of damage to a creature during combat.
- **issue:** "small amount" and "to a creature" undersell cards that deal damage equal to power and also hit the controller
- **example:** Laccolith Titan: "deal damage equal to its power to target creature"; Assembled Alphas deals "3 damage to that creature and 3 damage to that creature's controller"
- **suggested fix:** Deals damage to a creature it blocks or is blocked by, separate from its combat damage, sometimes also hitting that creature's controller.

### `sth-storyline-in-cards` — suspect
- **current:** A card whose flavor text is part of a storyline told across the set.
- **issue:** Claims the story is told via 'flavor text'; the card names/effects (assorted Stronghold cards) suggest the story is told through the cards themselves, not specifically flavor text; unverifiable claim
- **example:** Scapegoat, Hesitation, Smite, Sword of the Chosen: assorted Stronghold cards whose oracle text and names, not clearly flavor text, form the connection
- **suggested fix:** A card that is part of a storyline told through the set's cards.

### `abrade` — suspect
- **current:** A modal spell that either deals damage to a creature or destroys an artifact.
- **issue:** Overspecifies to 'damage to a creature'; several tagged cards destroy the creature (or exile the artifact) rather than dealing damage, and some target players or all creatures
- **example:** Kill! Maim! Burn!: 'Destroy target artifact / Destroy target creature / deals 3 damage to target player' has no damage-to-a-creature mode; Suplex 'Exile target artifact' exiles rather than destroys
- **suggested fix:** A modal spell that can both remove a creature and destroy an artifact.

### `typal-cleric` — suspect
- **current:** Cares about Clerics, growing stronger or gaining abilities as more are in play.
- **issue:** 'growing stronger or gaining abilities' overspecifies the payoff; many cards instead tap Clerics as a cost/resource
- **example:** Ancestor's Prophet: 'Tap five untapped Clerics you control: You gain 10 life.' (also Master Apothecary, Gangrenous Goliath tap Clerics as a cost, not growing)
- **suggested fix:** Cares about Clerics, getting stronger or using them as a resource as more are in play.

### `alt-commander` — suspect
- **current:** A legendary creature printed as a backup commander alongside a preconstructed deck's face commander.
- **issue:** Overspecified: tag also covers Partner/draft legends and even a deck's face commander, not just a precon backup
- **example:** Alela, Cunning Conqueror is the face commander of its precon yet carries this tag; Silas Renn is a Partner legend from a draftable set, not a precon backup
- **suggested fix:** A legendary creature offered as an alternate way to lead a Commander deck, such as a precon's secondary commander or a Partner legend.

### `alternate-cost-sacrifice` — suspect
- **current:** Lets you sacrifice a permanent instead of paying some or all of the spell's mana cost.
- **issue:** Says 'instead of paying' mana, but many tagged cards sacrifice as an ADDITIONAL cost on top of full mana, not in place of it
- **example:** Worthy Cost: 'As an additional cost to cast this spell, sacrifice a creature.' (also Rottenmouth Viper, Spark Harvest, Worthy Cost) — the sacrifice adds to the cost rather than replacing mana
- **suggested fix:** Lets you sacrifice one or more permanents as part of casting a spell, either instead of its mana cost or as an added cost.

### `synergy-color-each` — suspect
- **current:** Scales its effect based on how many colors a permanent or card has.
- **issue:** Overspecified to counting colors; many tagged cards reference each color individually rather than scaling by count
- **example:** Tam, Mindful First-Year: 'Each other creature you control has hexproof from each of its colors.' and Pit of Offerings: 'Add one mana of any of the exiled cards' colors.' Neither scales by a color count.
- **suggested fix:** Cares about the individual colors of a permanent or card, scaling with or referencing each color it has.

### `synergy-exile-cast` — suspect
- **current:** Rewards you for casting spells from exile.
- **issue:** Underspecified: several cards reward playing any card (including lands) from exile, not just casting spells
- **example:** Rocco, Street Chef: "Whenever a player plays a land from exile or casts a spell from exile..."; Prosper, Tome-Bound: "Whenever you play a card from exile, create a Treasure"
- **suggested fix:** Rewards you for casting spells or playing cards from exile.

### `synergy-trample` — suspect
- **current:** Rewards or boosts creatures you control that have trample.
- **issue:** Misses the granting facet: many cards conditionally give trample to your creatures rather than reward creatures that already have it
- **example:** Bleeding Effect: "creatures you control gain flying until end of turn if a creature card in your graveyard has flying. The same is true for ... trample"
- **suggested fix:** Rewards, boosts, or grants trample to creatures you control.

### `trumpet-blast` — suspect
- **current:** An instant that boosts the power of your creatures until end of turn.
- **issue:** "your creatures" is wrong for cards that pump every attacking creature regardless of controller
- **example:** Mercadia's Downfall: "Each attacking creature gets +1/+0 until end of turn for each nonbasic land defending player controls."
- **suggested fix:** An instant that gives creatures a power boost until end of turn.

### `wind-drake-with-set-s-mechanic` — suspect
- **current:** A 2/2 flyer for three mana that showcases one of its set's signature mechanics.
- **issue:** Overspecified: "for three mana" is wrong for higher-costed members whose added mechanic pushes the cost up
- **example:** Falcon Abomination is a 2/2 flyer for {4}{B} (five mana) that also makes a decayed Zombie token
- **suggested fix:** A small flyer, usually a 2/2 in the vein of Wind Drake, that showcases one of its set's signature mechanics.

### `lure` — suspect
- **current:** Forces all creatures able to block this creature to do so.
- **issue:** Says 'this creature' but many carriers force blocks on a target/equipped/enchanted creature, not themselves
- **example:** Nemesis Mask: 'All creatures able to block equipped creature do so.'; Jermane: 'all creatures able to block target creature ... do so.'
- **suggested fix:** Forces all creatures able to block a chosen creature to do so.

### `hungry-demon` — suspect
- **current:** Forces you to sacrifice a creature, usually each upkeep, unless you meet some condition.
- **issue:** Overspecifies the condition (many force sacrifice unconditionally) and misses that some versions make every player sacrifice, not just you
- **example:** Woebringer Demon: "At the beginning of each player's upkeep, that player sacrifices a creature of their choice." (Demonic Taskmaster and Abhorrent Overlord force sacrifice with no condition at all.)
- **suggested fix:** Forces a player, usually you at each upkeep, to sacrifice a creature, sometimes only if a condition isn't met.

### `typal-squirrel` — suspect
- **current:** Cares about Squirrel creatures you control.
- **issue:** 'you control' is overspecified; some effects buff all Squirrels, not just yours
- **example:** Squirrel Wrangler: 'Squirrel creatures get +1/+1 until end of turn' (any player's Squirrels)
- **suggested fix:** Cares about or boosts Squirrel creatures.

### `graveyard-fuel-artifact` — suspect
- **current:** Spends or cares about artifact cards in your graveyard.
- **issue:** Says "your graveyard" but several cards spend artifact cards from any graveyard
- **example:** Conversion Chamber: "Exile target artifact card from a graveyard"; Mastermind Plum: "exile up to one target card from a graveyard"
- **suggested fix:** Spends or cares about artifact cards in a graveyard.

### `old-lifelink` — suspect
- **current:** Gains you life equal to damage dealt, via a trigger instead of the lifelink keyword.
- **issue:** overspecifies 'you' as the life gainer; some grant life to the source's controller, not always you
- **example:** Essence Sliver: 'Whenever a Sliver deals damage, its controller gains that much life.'
- **suggested fix:** Gains life equal to the damage a source deals, via a trigger instead of the lifelink keyword.

### `three-letter-name` — suspect
- **current:** A card whose name is exactly three letters long.
- **issue:** 'exactly three letters' misfires on split cards where a face is longer
- **example:** Wax // Wane: 'Wax' is three letters but 'Wane' is four, yet the card carries the tag
- **suggested fix:** A card with a face whose name is exactly three letters long.

### `color-choose-land` — suspect
- **current:** A land that lets you choose a color as it enters, then taps for mana of that color.
- **issue:** Overspecifies to 'color'; several tagged lands instead choose a basic land type
- **example:** Multiversal Passage: 'As this land enters, choose a basic land type. ... This land is the chosen type.' (also A-Thran Portal)
- **suggested fix:** A land that lets you choose a color or basic land type as it enters, then taps for mana accordingly.

### `hate-island` — suspect
- **current:** Punishes Islands and their controllers by locking, damaging, or bouncing them.
- **issue:** Omits outright destruction (a major mode) and overstates 'punish' for cards that merely benefit from Islands
- **example:** Boil: 'Destroy all Islands.' (also Carpet of Flowers just adds mana based on Islands, no punishment)
- **suggested fix:** Destroys, locks, damages, bounces, or otherwise cares about Islands and the players who control them.

### `damage-prevention-planeswalker` — suspect
- **current:** Prevents damage that would be dealt to you and the permanents you control, including planeswalkers.
- **issue:** Overbroad 'all permanents' when several only shield you and planeswalkers; some redirect, not prevent
- **example:** Comeuppance: 'Prevent all damage that would be dealt to you and planeswalkers you control'; Gideon's Sacrifice redirects damage to a chosen permanent 'instead'
- **suggested fix:** Prevents or redirects damage that would be dealt to you and your planeswalkers, and sometimes all your permanents.

### `devour` — suspect
- **current:** Lets this creature sacrifice creatures as it enters to gain +1/+1 counters for each one.
- **issue:** Ignores the Devour N multiplier and the devour-artifact variant that sacrifices artifacts
- **example:** Caprichrome: 'Devour artifact 1 (As this creature enters, you may sacrifice any number of artifacts...)'; Feaster of Fools 'Devour 2' enters with twice that many counters
- **suggested fix:** As this creature enters, it may sacrifice creatures (or artifacts) to enter with +1/+1 counters equal to a multiple of the number sacrificed.

### `synergy-first-strike` — suspect
- **current:** Rewards or grants abilities to your creatures that have first strike.
- **issue:** Wording implies granting abilities only to creatures that already have first strike, but many cards grant first strike itself to creatures lacking it
- **example:** Priest of Possibility: 'If a card among them has flying, Priest ... perpetually gains flying. The same is true for first strike...' grants first strike, not to a creature that already has it
- **suggested fix:** Rewards your creatures for having first strike or grants first strike to them.

### `counts-as-a-type` — suspect
- **current:** An older card using deprecated wording that treats it as having an extra creature type.
- **issue:** 'deprecated wording' and 'extra creature type' don't fit all cards; some use current wording and it's an extra card type, not a subtype
- **example:** Transmogrifying Licid: 'Enchanted creature gets +1/+1 and is an artifact in addition to its other types.' (current wording, grants a card type to another creature)
- **suggested fix:** Is treated as having an additional card type beyond those printed, such as an artifact that also counts as a creature.

### `cranial-plating` — suspect
- **current:** Gets stronger based on how many artifacts or Equipment you control.
- **issue:** 'artifacts or Equipment' is redundant and off (Equipment are artifacts); cards count all artifacts, and several pump another creature
- **example:** Improvised Arsenal: 'Equipped creature gets +1/+0 for each artifact you control.'
- **suggested fix:** A creature gets stronger for each artifact you control.

### `o-ring-with-set-mechanic` — suspect
- **current:** Exiles a permanent temporarily, using another set's mechanic to power the effect.
- **issue:** 'set mechanic powers the effect' is misleading; the keyword is usually just bolted onto a standard O-ring exile, not powering it
- **example:** Detention Chariot: 'exile target artifact or creature an opponent controls until this Vehicle leaves the battlefield' plus unrelated Crew/Cycling
- **suggested fix:** Exiles a permanent until this leaves the battlefield, on a card that also carries a set's keyword mechanic.

### `processing` — suspect
- **current:** Lets you put an opponent's exiled card into their graveyard for an added benefit.
- **issue:** overfits to the process keyword; some tagged cards target any owner's exiled card, give no benefit, or return it to the library instead of a graveyard
- **example:** Pull from Eternity: 'Put target face-up exiled card into its owner's graveyard' (any owner, no benefit); Riftsweeper shuffles it into the library instead
- **suggested fix:** Moves a card out of exile, usually an opponent's into their graveyard, often for an added bonus.

### `type-errata-lord` — suspect
- **current:** A creature type tag for cards that once had the now-defunct Lord creature type.
- **issue:** Overclaims that all tagged cards once had the defunct Lord creature type; several sample cards never did
- **example:** Ghost Council of Orzhova is a Legendary Creature Spirit (printed 2006) and never carried the Lord type; likewise Caller of the Hunt (Human) and Voice of the Woods (Elf)
- **suggested fix:** Marks cards tied to the old, now removed 'Lord' creature type from before the creature type errata update.

### `bounceable-aura` — suspect
- **current:** An Aura that can return itself to its owner's hand.
- **issue:** Not always to hand; some bounce to library
- **example:** Soaring Hope: "{W}: Put this Aura on top of its owner's library."
- **suggested fix:** An Aura with an ability that returns itself to its owner's hand or library.

### `creates-oracle-token` — suspect
- **current:** Creates a token that's a full copy of a specific named card.
- **issue:** Overspecified as "full copy"; many are predefined tokens named after a card, not copies
- **example:** Disa the Restless: "create a Tarmogoyf token" (a defined {1}{G} Lhurgoyf, not a copy); Mutable Explorer: "create a tapped Mutavault token."
- **suggested fix:** Creates a token that reproduces a specific named card.

### `harmonic` — suspect
- **current:** Gets a bonus if you control both an artifact and an enchantment.
- **issue:** Overspecified to "both"; several reward artifacts and enchantments independently or reward casting them
- **example:** Starnheim Courser: "Artifact and enchantment spells you cast cost {1} less" (no both-condition); Nezumi Bladeblesser grants deathtouch for an artifact and menace for an enchantment separately.
- **suggested fix:** Rewards you for controlling artifacts and enchantments, often for controlling both.

### `synergy-defender` — suspect
- **current:** Rewards you for controlling creatures with defender.
- **issue:** underspecified: many carriers also let defenders attack or tutor them, not just reward controlling them
- **example:** Wakestone Gargoyle: "{1}{W}: Creatures you control with defender can attack this turn as though they didn't have defender."; Shield-Wall Sentinel searches for a defender
- **suggested fix:** Rewards or supports controlling creatures with defender, such as scaling with them, tutoring them, or letting them attack.

