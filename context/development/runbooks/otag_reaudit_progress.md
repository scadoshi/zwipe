# Oracle-tag description RE-AUDIT — progress & findings

Second-pass QA re-run of `ORACLE_TAG_DESCRIPTIONS` using the **improved two-stage
workflow** (card-data grounding + skeptical Verify stage, commit `99dc2091`). Ranks
1-2000 were originally checked by the *old* single-stage workflow that was blind to
cost/color/hybrid/rarity and over-generalized cycles; this re-audit re-checks them from
the top by population to catch what the old pass missed. Companion to the forward-audit
progress in [`otag_audit_progress.md`](otag_audit_progress.md).

> **This is the ACTIVE audit task.** Ignore the paused forward sweep in
> [`otag_audit_progress.md`](otag_audit_progress.md) (do not resume that at 2201). Continue
> the re-audit here from **rank ~1001+**.

> **APPLIED: ranks 1-1000 (all 181 wrong+suspect fixes) were applied to
> `ORACLE_TAG_DESCRIPTIONS` on 2026-07-16.** The batches below are kept as the record of what
> changed. **Do not re-apply them.** New batches (rank ~1001+) remain findings-only until applied.
> `wrong`/`suspect` are the *verified-surviving* flags. Ships
> [`otag_audit_workflow.js`](otag_audit_workflow.js).

## Coverage / resume
- **Re-audited: 1000 / 4,357** (ranks 1-1000 by card population). **Next: rank ~1001+.**
- **Resume (do this to continue):** pull the top-N `(slug, description)` pairs by card
  population from the DB, **excluding** every slug already in
  [`otag_reaudit_slugs.txt`](otag_reaudit_slugs.txt) (this is the re-audit's own tracker,
  separate from the forward `otag_audited_slugs.txt`), and run them through
  [`otag_audit_workflow.js`](otag_audit_workflow.js). Then append the returned `audited` slugs
  to `otag_reaudit_slugs.txt` and log the findings here.
- Run in 2x250 shards concurrently (args payload stays paste-able); ~500/session is comfortable.
- Resume is self-healing: the tracker lists only confirmed re-audited slugs, so a partial run
  re-queues whatever didn't finish, no gaps or dupes.

## Findings so far
Cumulative across 1000 re-audited: **815 clean, 171 suspect, 10 wrong**; Verify overturned 16.
Per-batch detail below.

## Batch: ranks 1-500
500 re-audited: **416 clean, 81 suspect, 3 wrong**; the Verify stage overturned 6 auditor flags.

**What the old pass missed.** The improved auditor flagged a high rate of
**over/under-specification** on these terse, high-visibility head tags: singular
"a creature"/"its target" wording where the tag also covers mass/any-permanent effects,
"opponent" where it's "each player", "spell" where carriers are permanents, and
sibling-overlap gaps (instant vs instant-or-sorcery). These are exactly the nuances the
old single-stage workflow did not surface. Overturn rate 6/90 (~7%), all overturns are
correct rejections (e.g. `gives-indestructible`: creatures ARE permanents, so "other
permanents" is a fine superset).

### Wrong (3) — fix recommended

#### `unique-token`
- **current:** A specific token variant that only one card creates.
- **issue:** Tagged objects are the token cards themselves, and 'only one card creates' is false for generic tokens like Insect, Soldier and Wolf that many cards create
- **example:** Insect: type 'Token Creature — Insect', colors ['G'], a vanilla 1/1 green token created by many different cards, not just one
- **suggested fix:** A named token creature with its own defined characteristics.
- **verify note:** Pulled objects are token cards themselves (Insect, Soldier, Wolf, Sheep, Wurm) that many different cards create, so 'only one card creates' is false.

#### `hate-sorcery`
- **current:** Lets you cast an instant or sorcery card taken or exiled from an opponent.
- **issue:** Only covers steal-and-cast; misses the large 'punish/answer instants and sorceries' half of this hate tag (damage prevention, redirection, discard)
- **example:** Forethought Amulet: "If an instant or sorcery source would deal 3 or more damage to you, it deals 2 damage to you instead." (also Harsh Judgment redirects instant/sorcery damage, Refraction Trap punishes red instants/sorceries)
- **suggested fix:** Punishes, answers, or steals instant and sorcery spells.
- **verify note:** Forethought Amulet reduces instant/sorcery damage to you, Harsh Judgment redirects, Collective Brutality forces discard: a large punish/answer half is missing

#### `regrowth-sorcery`
- **current:** Returns a sorcery card from your graveyard to your hand.
- **issue:** Nearly every carrier returns 'instant or sorcery,' not just sorcery; description drops instants
- **example:** Relearn: 'Return target instant or sorcery card from your graveyard to your hand.'
- **suggested fix:** Returns an instant or sorcery card from your graveyard to your hand.
- **verify note:** Relearn, Flood of Recollection, Scribe of the Mindful all return 'instant or sorcery', not just sorcery

### Suspect (81) — minor imprecision, review before applying

| slug | issue | suggested fix |
| --- | --- | --- |
| `removal-destroy` | "its target" overspecifies; some members are non-targeted mass destruction | Removal that destroys what it hits. |
| `namesake-spell` | "spell" underspecifies: many tagged cards are permanents (creatures, artifacts), not instants/sorceries | A card whose name references a specific named character. |
| `repeatable-pure-draw` | "at no extra cost" is imprecise; several members draw with a life-loss rider, so it isn't cost-free | Repeatably draws extra cards. |
| `power-boost-to-all` | 'your creatures' implies the whole team, but roughly half the cards only pump a chosen type or subgroup, not all your creatures | Raises the power of your creatures, often only those of a chosen type or subgroup. |
| `synergy-artifact` | 'playing' implies casting, but most of these reward controlling/having artifacts, not casting them | Cares about or benefits from artifacts, usually the ones you control. |
| `group-slug` | Says "each opponent" but several members drain or damage EACH PLAYER, including you | Drains life from or deals damage to each opponent, sometimes every player. |
| `tapper-creature` | Singular "a creature" understates members that tap ALL of an opponent's creatures | Taps one or more creatures so they can't attack or block. |
| `death-trigger` | Only creatures "die"; the common case is a creature dying, not any "permanent" | Has an ability that triggers when a creature dies. |
| `type-addition-human` | Editorial/unverifiable 'to match what it always was' and describes metadata rather than a gameplay action; these are older creatures retroactively given the Human type | A creature whose type line was updated to include the Human creature type. |
| `potentially-black-border` | "tournament-legal" is the wrong criterion; the tag marks joke/Un cards that are mechanically functional enough to be black-bordered, most of which are NOT tournament legal | A joke or Un-set style card that is mechanically functional enough to have been printed in a normal black-bordered set. |
| `synergy-instant` | understates the sibling overlap; the tagged cards almost all care about 'instant or sorcery', so it should mirror synergy-sorcery and note sorceries too (and 'casting' is more precise than 'playing') | Rewards casting instants, and usually sorceries too. |
| `removal-exile` | "its target" is too narrow; some are mass exile with no single target | Removal that exiles what it hits instead of destroying it. |
| `untapper-creature` | "a target creature" undersells cards that untap all your creatures, not a single target | Untaps one or more creatures. |
| `cast-on-resolution` | Says 'another card', but rebound/suspend/miracle members re-cast the SAME card as an ability resolves; also 'one of its spells' is awkward wording | Casts a spell as another spell or ability resolves, rather than at its normal time. |
| `mini-refund` | Says a one-time 'burst of extra mana to spend soon,' but the majority of tagged cards are repeatable mana rocks that produce ongoing mana, not a single burst. | Gives you a bit of extra mana, usually from a small mana rock. |
| `removal-artifact` | Says only 'destroying or exiling,' but the tag also covers bounce and tuck removal of artifacts. | Removal that destroys, exiles, or bounces an artifact. |
| `egg` | "artifact" slightly overspecifies; some eggs are non-artifact sac tokens | A cheap permanent, usually an artifact, meant to be sacrificed for a payoff. |
| `burn-with-set-s-mechanic` | Says "spell" but the tag also covers damage-dealing permanents (creatures, enchantments), not just instants/sorceries | A damage-dealing card that also carries the signature mechanic of the set it debuted in. |
| `gives-castable-from-exile` | says 'you' but several let ANY player cast, and misses that it's usually without paying mana cost; 'instead of your hand' is awkward | Lets a card be cast from exile, often without paying its mana cost. |
| `hate-set-mechanic` | 'counters' wrongly implies countering spells; these answer/shut down a keyword, and it targets any player's, not just opponents' | Answers or shuts down a specific keyword or mechanic from its own set. |
| `shapechange` | 'specific values' is wrong for variable cases, and it omits that these set BASE stats and sometimes change types/form | Sets a creature's base power and toughness, sometimes also changing its types. |
| `curiosity` | Says 'it' draws when it deals damage, but the ability is usually GRANTED to enchanted/other creatures or tokens, and it's specifically combat damage | Draws you a card whenever a creature deals combat damage to a player. |
| `removal-enchantment` | limits to 'destroy or exile' but the tag also covers bounce and sacrifice-tax answers to enchantments | Removal that answers an enchantment by destroying, exiling, or bouncing it. |
| `protects-all` | Says 'your creatures' but the tag protects any permanent type (artifacts, planeswalkers, all permanents) | Shields several of your permanents at once, such as granting them indestructible or hexproof or flickering them. |
| `selective-group-hug` | Frames it as opponent-only and merely 'leaving out', but you can direct benefits to any player including yourself and some variants punish the excluded | A group hug effect where you choose which players get the benefits, sometimes at the excluded players' expense. |
| `dnd-character` | Says 'named' character, but many tagged cards are generic non-legendary Dungeons and Dragons creatures, not named unique characters | Depicts a character or creature from Dungeons and Dragons in its name or text. |
| `theft-cast` | Too narrow: many let you PLAY a card (including lands), not just cast a spell, and some hit every player's library, not only another player's | Lets you cast or play cards from another player's hand, library, or graveyard, usually without paying their cost. |
| `discard-with-set-s-mechanic` | The cited examples (escape, kicker) don't appear on the pulled cards and kicker isn't really set-specific; actual mechanics are Flashback, Spectacle, Evoke, Party, Powerstone | Ties a discard effect to a mechanic from that card's set, like flashback, spectacle, or evoke. |
| `regrowth-self` | "or the battlefield" is unsupported; every pulled card returns itself to HAND (battlefield returns are the activate-from-graveyard sibling) | Returns itself from your graveyard to your hand. |
| `synergy-legendary` | "you control" is overspecified; several members target any legendary permanent, not only ones you control | Cares about legendary creatures or permanents, often rewarding you for controlling them. |
| `mill-opponent` | "into their graveyard" is not always true; several members exile from the top of the library instead of milling to the graveyard | Depletes an opponent's library by milling or exiling cards from the top of it. |
| `gives-pp-counters-to-all` | Overspecifies 'your' creatures; some target any player or only a subtype | Puts +1/+1 counters on each creature a player controls at once, usually your own board. |
| `creature-count-matters` | Narrows to 'you control' but many count all creatures on the battlefield or compare totals | Gets better the more creatures are on the battlefield, usually your own. |
| `graveyard-fuel-creature` | says only 'abilities' but several members fuel a SPELL via an additional casting cost, not an activated ability | Exiles creature cards from a graveyard to fuel its spells and abilities. |
| `cards-in-graveyard-matter` | the qualifier "not just how many" is contradicted by several tagged cards that care about raw graveyard count | Cares about the cards sitting in a graveyard. |
| `giant-growth` | "temporary" and "combat trick" don't hold for the permanent-counter and Aura members | Boosts a creature's power and toughness. |
| `one-sided-fight` | underspecified: targets can be planeswalkers too, and some use two creatures' or double power | Has a creature deal damage equal to its power to another creature or planeswalker, which deals none back. |
| `type-addition-from-none` | 'by errata' framing misses man-lands in the tag, and 'a creature' excludes lands that animate into typed creatures | A card given a creature type it originally lacked, whether through errata or by animating into a creature. |
| `faux-targeting` | Underspecified: many cards choose a card name or multiple creatures, not just 'a permanent or player' | Chooses a permanent, player, card, or card name without targeting it, so hexproof and shroud can't stop it. |
| `reanimate-cast` | Overspecified: says 'permanent card' but some cards cast any spell type from the graveyard | Lets you cast a card straight out of your graveyard. |
| `impulse-creature` | 'Reveals' is imprecise; many of these cards only look at the top cards, not reveal them | Digs through the top of your library to find a creature card. |
| `thoughtseize` | Overspecified to 'discard'; many members exile or bottom-tuck the chosen card, and several target any player not just an opponent | Makes a target player reveal their hand so you pick a card to discard, exile, or bottom. |
| `hate-regenerate` | Says 'a creature' but the tag also covers artifacts and lands that can't be regenerated | Stops a permanent from being regenerated, usually while destroying it. |
| `consult-cast` | Says 'your library' but several exile from an opponent's library | Exiles cards from the top of a library until you hit one you may cast, often for free. |
| `gives-tap-ability` | Says 'creatures' but many grant the tap ability to lands or tokens | Grants permanents a new activated ability that requires tapping them. |
| `burn-player-each` | Overspecified: many carriers damage only each player, not each creature | Deals damage to each player, and often to each creature as well. |
| `poisonous` | Overspecified as 'A creature'; several carriers are noncreature permanents/spells | Gives a player poison counters, usually when a creature it controls or creates deals them damage. |
| `swap-removal` | 'Removes a creature' is too narrow; some remove any permanent, and the replacement isn't always given | Removes a permanent but its controller gets a replacement permanent, often a token, in return. |
| `combat-neutral-damage-trigger` | Not always a creature and many trigger only on damage to a player/opponent, not literally any damage | Triggers when it deals damage, whether or not that damage is dealt in combat. |
| `nightveil-theft` | Says "another player's" but several hit each player including you, and cards are often "played" not only "cast" | Exiles cards from libraries or hands and lets you cast or play them, effectively stealing from other players. |
| `life-loss-matters` | 'how much' overspecifies; most cards only care WHETHER a player lost life this turn, not the amount | Cares that a player, usually an opponent, lost life this turn, not just took damage. |
| `cost-ignorer` | Only mentions casting; several tagged cards cheat permanents straight onto the battlefield without casting | Puts a spell or permanent into play without paying its mana cost, whether by casting it for free, paying an alternate cost, or putting it straight onto the battlefield. |
| `restricted-blocker` | 'can only block under specific conditions' misses the common case of creatures that block freely but can't block a certain subset | A creature with restrictions on whether or what it can block. |
| `tap-fuel-power` | 'use an amount equal to its power' fits Station's scaling but misrepresents Teamwork/Saddle, which just tap creatures whose TOTAL power meets a fixed threshold | Lets you tap creatures whose power fuels or enables an effect. |
| `mulch` | "keeps one" is overspecified; many keep multiple cards | Looks at cards from the top of your library, puts one or more into your hand, and mills the rest. |
| `loot` | Singular 'a card...a card' understates the many multi-card looters and connive cards carrying this tag | Draws you one or more cards, then makes you discard one or more cards. |
| `universal-type-change` | Most cards ADD a type in addition to existing types rather than replacing/converting it, so 'changes the type into another' is slightly misleading | Makes every permanent of one kind gain an extra type all at once, so they all count as something new in addition to what they already are. |
| `flicker-slow` | Overspecifies the return timing; many members return at a different time, not the next end step | Exiles a permanent and returns it to the battlefield later, most often at the next end step, rather than right away. |
| `turn-face-up-trigger-self` | Says "this creature" but some members are noncreature permanents (Equipment) when they trigger | Triggers an effect when this permanent is turned face up from morph, megamorph, or disguise. |
| `card-types-in-graveyard-matter` | 'gets stronger based on how many' implies scaling and that the card itself grows, but most are a fixed delirium threshold (4+ types) and many buff OTHER creatures or deal damage | Cares about how many different card types are in your graveyard, often getting stronger or gaining abilities once four or more are present. |
| `full-refund` | "equal to or more than what you spent" is an overclaim; many are mana rocks or land-untappers, and some refund a variable/conditional amount that can be less than the cost | Refunds much or all of the mana you spent to cast it, usually by untapping your lands or producing extra mana. |
| `type-errata-summon-creature` | 'Portal-era' is misleading; the old 'Summon' type-line wording spanned Alpha through 5th Edition, not just Portal, and included non-Portal cards | A creature whose type line originally used the old 'Summon' wording before errata updated it to the modern creature type format. |
| `minigame` | Not all engage other players; several are solo or out-of-game tasks | Involves an unusual game challenge like a vote, bet, guess, or timed task, sometimes with people outside the game. |
| `temporary-reanimation` | Some return from hand and to hand, and one returns Auras, not just graveyard creatures | Returns a creature to the battlefield with haste, then removes it at end of turn. |
| `pacifism` | Overspecifies 'and blocking'; several members only grant defender, which stops attacking but still allows blocking | Keeps a creature from attacking, and usually from blocking, without destroying it. |
| `hate-color-choose` | Overspecified to 'protection'; several members choose a color to answer it in other ways (bounce, damage redirect/prevention, hexproof), and one picks the color at random | Chooses a color and answers it, most often by granting protection from that color. |
| `hate-instant` | Overspecifies to "opponents'"; several cards answer instant/sorcery spells from any source, not just opponents | Answers or punishes instant and sorcery spells, or steals value from them. |
| `eponymous` | Overspecifies to "creature"; the tag also covers non-creature permanents whose name matches their subtype | A card whose name is made up of its own subtypes. |
| `un-type-line` | says 'silly card type' but most have normal card types with Un-only creature subtypes; the trait is an Un-set type line | A joke Un-set card (silver-bordered or Acorn-stamped) whose type line uses a creature type or card type found only in those sets. |
| `tutor-mv` | Says 'a specific mana value' (exact), but most cards search for a card at or below a limit (a range), not an exact value | Searches your library for a card by mana value, usually one at or below a set limit. |
| `sweeper-graveyard` | Says 'Exiles', but several members empty a graveyard by shuffling it into the library instead of exiling | Empties one or more graveyards at once, usually by exiling them. |
| `tutors-by-name` | says 'your library' but the dominant Partner-with pattern here fetches from a target player's library into hand | Searches a library for a card with a specific name. |
| `synergy-arcane` | only describes the 'whenever you cast' payoff, missing the splice-onto-Arcane enabler cards that also carry the tag | Cares about Arcane spells, either triggering a bonus when you cast a Spirit or Arcane spell or splicing onto Arcane. |
| `quadratic` | 'compounds each time' implies iterative growth over turns, but many are one-shot square-of-a-count effects with no repetition | Scales with the square of a count, since the same number multiplies against itself. |
| `maro-sorcerer` | Overspecified as 'a creature'; many carriers are Auras, Equipment, or sorceries that grant a land-count bonus, not creatures with self-scaling P/T | A card whose power and toughness, or the bonus it grants, scale with the number of lands, or a land type, you control. |
| `lhurgoyf` | Says 'a creature' but the tag also covers Auras and sorceries that scale a creature's power/toughness | A card whose power and toughness, or a creature it affects, scale with the number of cards in a graveyard. |
| `drain-creature` | Says 'to a creature' but many carriers deal to any target, not just creatures | Deals damage to a creature or other target and you gain life equal to the damage dealt. |
| `life-total-matters-self` | 'above or below a threshold' misses cards that use your life total as a raw value, and some carriers check 'a player' rather than only you | Cares about your own life total, often whether it is above or below a threshold. |
| `discard-to-exile` | Says 'opponent's hand', but several target any player and also exile from the graveyard, not just the hand | Exiles cards from a target player's hand (and sometimes their graveyard) instead of putting them into the graveyard. |
| `regrowth-instant` | Underspecified: nearly every card returns an instant OR sorcery, not just an instant | Returns an instant or sorcery card from your graveyard to your hand. |
| `clone` | Not only creatures copying creatures: non-creature permanents (Equipment, lands) and copies of artifacts also carry the tag | Lets a permanent enter the battlefield as a copy of another permanent, most often a creature copying another creature. |

### Overturned by Verify (6) — flag rejected, keep current
- `repeatable-crime` — auditor: underspecified: a crime also covers cards in an opponent's graveyard, and the description reads as if the ability always aims at opponents when many tagged abilities are buffs (any creature) that merely CAN target opponent permanents | verifier: Coeurl/Walking Sponge/Wyluli Wolf are repeatable targeted abilities; current conditional wording is accurate and no pulled card shows graveyard targeting (incompleteness, not inaccuracy)
- `lifegain` — auditor: Says 'you' but some cards give the life to another player (creature's owner), not necessarily you | verifier: 10 of 12 pulled cards gain YOU life; only Misfortune's Gain 'Its owner gains 4 life' routes it elsewhere, a rare drawback-removal outlier
- `self-replacement-effect` — auditor: technically correct but opaque jargon ('replaces its own resolution') a player won't parse; the cards are 'instead' clauses gated on a condition | verifier: Every card is a genuine self-replacement effect (Colossal Growth/Overload 'if kicked... instead', Shower of Coals threshold 'instead', Deny the Divine 'exile it instead'); the flag itself concedes the wording is technically correct, so no accuracy defect.
- `gives-indestructible` — auditor: Says "permanents" but every card grants indestructible to creatures specifically | verifier: Creatures are permanents, so 'other permanents' is a correct superset (Rootborn Defenses 'Creatures you control gain indestructible', etc.), not inaccurate; only 12 of 282 seen, cannot prove creatures-only
- `mixed-subtypes` — auditor: Core claim is correct, but the illustrative example ('creature type on a land') is the rarest pattern in the tag; most members are Saga creatures or Equipment creatures, so the example under-represents what the tag actually holds. | verifier: Jasconian Isle is genuinely 'Land Creature — Island Fish', so 'creature type on a land' is a real, accurate member of the tag; flag is about representativeness, not accuracy
- `man-o-war` — auditor: Overspecified: not all are creatures with an enter trigger, and several bounce any nonland permanent rather than a creature | verifier: 11 of 12 pulled (Mist Raven, Roaming Ghostlight, etc.) are creatures that bounce a creature on enter; only Inscription of Insight deviates

---

## Batch: ranks 501-700

200 re-audited: **156 clean, 39 suspect, 5 wrong**; Verify overturned 4. Same pattern as 1-500: over/under-specification on terse head tags ("creatures you control" where cards count any permanent, "opponent's" where it's any player, singular "target" where effects are mass, wrong set/color/rarity assertions).

> **Findings only, not applied.** `ORACLE_TAG_DESCRIPTIONS` unchanged.

### Wrong (5) — fix recommended

#### `warlord`
- **current:** 
- **issue:** Overspecified to 'creatures you control'; most cards count lands, permanents, or creatures across the whole battlefield
- **example:** Ashaya, Soul of the Wild: "Ashaya's power and toughness are each equal to the number of lands you control." Also Kithkin Rabble: "power and toughness are each equal to the number of white permanents you control."
- **suggested fix:** A creature whose power, and often toughness, equals the number of permanents of a certain kind, most often creatures you control.
- **verify note:** Ashaya counts 'lands you control', Kithkin Rabble 'white permanents', Yavimaya Kavu 'red creatures on the battlefield' — not just creatures you control

#### `creature-ability-noncreature`
- **current:** 
- **issue:** Most tagged cards never become creatures; the tag is about noncreature permanents bearing keywords usually found on creatures, not 'abilities that matter once it becomes a creature'
- **example:** Darksteel Relic {0} Artifact: "Indestructible" (never becomes a creature); Weapon Rack: "enters with three +1/+1 counters"; Tanglepool Bridge: Indestructible artifact land
- **suggested fix:** A noncreature permanent that carries a keyword or ability usually found on creatures, like indestructible, flying, or +1/+1 counters.
- **verify note:** Darksteel Relic (Indestructible), Tanglepool Bridge (Indestructible land), and Weapon Rack (+1/+1 counters) never become creatures; the ability matters while they stay noncreatures

#### `type-errata-viashino`
- **current:** 
- **issue:** Wrong set attribution: the Viashino-to-Lizard errata predates Modern Horizons 3
- **example:** Kylox, Visionary Inventor is "Legendary Creature — Lizard Artificer" and was printed in The Lost Caverns of Ixalan (Nov 2023), before MH3 (June 2024), showing the Lizard convention already applied
- **suggested fix:** A Lizard creature that once carried the retired Viashino creature type, since folded into Lizard.
- **verify note:** Kylox is 'Legendary Creature — Lizard Artificer' (LCI, Nov 2023), already Lizard before MH3 (June 2024); the Viashino type retired in 2023, so the MH3 attribution is wrong

#### `remove-counters-other`
- **current:** 
- **issue:** Restricts to 'opponents' permanents' but most cards remove counters from ANY permanent (yours or theirs)
- **example:** Heartless Act: "Remove up to three counters from target creature." Clockspinning: "Remove that counter from that permanent or card." Shivan Sand-Mage: "Remove two time counters from target permanent." None are opponent-only.
- **suggested fix:** Removes counters from a permanent other than itself, or removes a player's counters.
- **verify note:** Most target ANY permanent: Heartless Act 'Remove up to three counters from target creature', Shivan Sand-Mage 'Remove two time counters from target permanent', Clockspinning 'Remove that counter from that permanent'; not opponent-restricted

#### `deal-with-the-devil`
- **current:** 
- **issue:** Asserts 'black' but pulled members include a white and a red enchantment
- **example:** Nine Lives: {1}{W}{W}, colors ['W']. Experimental Frenzy: {3}{R}, colors ['R']. Both are non-black.
- **suggested fix:** An enchantment, usually black, with a powerful effect and a serious, potentially game-losing drawback.
- **verify note:** Nine Lives is colors ['W'] ({1}{W}{W}) and Experimental Frenzy is colors ['R'] ({3}{R}), so the blanket 'black' is inaccurate

### Suspect (39) — minor imprecision, review before applying

| slug | issue | suggested fix |
| --- | --- | --- |
| `tap-fuel-artifact` | 'to power an ability' misses that most carriers tap artifacts as an additional cost to CAST a spell (waterbend), and they tap creatures/lands too, not just artifacts | Lets you tap untapped artifacts you control, and often creatures or lands too, to help pay a cost. |
| `free-discard-outlet` | 'no per-turn limit' is false for at least one carrier, and the discard is often restricted to a specific card type rather than 'a card' | Lets you discard a card as a cost, with no mana required, to fuel an ability. |
| `gives-ward` | Grants ward to permanents broadly (artifacts, multicolored creatures, tribes), not just "a creature" | Grants ward to one or more of your permanents, usually creatures, so an opponent must pay a cost to target them or the spell is countered. |
| `tapper-artifact` | Overspecifies "target" and "tap abilities"; most also tap creatures and lands, and some tap all or force enter-tapped | Taps down a target artifact, and often creatures and lands too, keeping it from untapping or acting. |
| `synergy-basic` | Overspecified to 'controlling and scaling'; many cards search for, sacrifice, or spend basic lands rather than reward controlling them | Cares about basic lands, whether by controlling, searching for, sacrificing, or spending them. |
| `impulse` | 'put one into your hand' is too narrow; some cards put multiple cards into hand | Lets you look at the top cards of your library and put one or more of them into your hand. |
| `haven` | 'your own permanents to keep them safe' overspecifies; several exile any nonland permanent, including opponents' as removal | Exiles a nonland permanent and can later return it or let its owner replay it, whether protecting your own or removing an opponent's. |
| `naturalize-with-set-mechanic` | Says 'destroys' but several members exile or sacrifice the artifact/enchantment instead | Destroys or otherwise removes an artifact or enchantment while also using a mechanic tied to its set. |
| `power-doubler` | Many double power AND toughness, and one is perpetual not until end of turn | Doubles a creature's power, sometimes its toughness too, usually until end of turn. |
| `auraify` | Phrasing implies transforming another object; really the card itself becomes an Aura, usually via bestow | Can become an Aura enchantment attached to another permanent, usually via bestow. |
| `splits-on-death` | Overspecified: says 'two or more' but some make exactly one token | Creates one or more creature tokens when a creature dies. |
| `reanimate-permanent` | "straight to the battlefield" excludes the cast/play-from-graveyard cards the tag also covers | Puts a permanent card from a graveyard onto the battlefield, or lets you cast permanents from there. |
| `wish` | 'put it into your hand' overspecifies; several members let you play the card from outside the game instead of drawing it to hand | Lets you bring in a card you own from outside the game, putting it into your hand or letting you play it. |
| `keyword-errata-flash` | 'Has flash' misses the members that don't literally have flash but grant it or are cast 'as though it had flash' | Has flash, is cast as though it had flash, or grants spells that ability, from before flash was an official keyword. |
| `hate-target` | Overspecifies as 'your permanents' and only 'punishes'; several trigger on any creature being targeted by anyone, and some reward you rather than punish | Triggers a punishing or rewarding effect whenever it or a permanent becomes the target of a spell or ability. |
| `unheroic` | Only describes the 'crime' subset; misses the large 'Repartee' subset that rewards casting spells targeting ANY creature (including your own), not an opponent | Rewards you for casting spells that target a creature, or for targeting an opponent, their permanents, or their graveyard. |
| `hate-planeswalker` | 'specifically at planeswalkers' overstates it (many hit creatures/players too) and 'removal or restrictions' misses defensive protection-from and attack-reward effects | Answers, hinders, or defends against planeswalkers, or rewards attacking them. |
| `mana-storage` | 'as counters or tokens' overspecifies; a whole subset instead keeps mana from emptying between phases | Lets you keep mana for later, whether as Treasure tokens, counters, or mana that doesn't empty between phases. |
| `parasitic-aura` | Overspecifies to damage/life loss; some members instead force a sacrifice or destroy the enchanted permanent | A harmful Aura that penalizes the enchanted permanent's controller, usually by dealing them damage or making them lose life. |
| `roll-d20` | 'different effect based on result' misses cards where the roll sets a QUANTITY, not a branching effect | Rolls a twenty sided die, with the result determining the outcome. |
| `theft-artifact` | says 'opponent's artifact' but many target any artifact, not just an opponent's | Lets you take control of an artifact, sometimes only temporarily. |
| `graveyard-seal` | 'shut down reanimation' narrows a broad graveyard-hate/denial tag to one use case | Shuts down graveyards, exiling cards, denying graveyard use, or stopping cards from reaching a graveyard at all. |
| `staple-with-set-s-mechanic` | 'common' reads as the rarity but many carriers are uncommon/rare/mythic | A card that showcases one of its set's featured mechanics. |
| `repeatable-mulch` | 'reveals' is imprecise; most carriers look privately or mill rather than reveal | Repeatedly digs cards off the top of your library, putting some into your hand and the rest into your graveyard. |
| `counterspell-ability` | "a target" overspecifies; several members counter ALL abilities untargeted, and some are activated-only or also hit spells | Counters an activated or triggered ability on the stack. |
| `cost-reducer-creature` | Overspecifies to 'creature spells'; several members reduce other spell types | Makes spells you cast, usually creature spells, cost less mana. |
| `hate-enchantment` | underspecified: many cards tax or punish enchantments rather than destroy/counter/protect | Destroys, counters, taxes, or otherwise punishes enchantments. |
| `gives-shroud` | 'another permanent' understates cases that grant shroud to many permanents of various types (creatures, lands, artifacts, enchantments) | Grants shroud to other permanents, so they can't be targeted by spells or abilities. |
| `scry-like` | Says singular 'top card' but many tagged cards look at the top TWO or THREE cards | Lets you look at one or more cards on top of a library and choose whether they stay on top or go elsewhere, similar to scry. |
| `typal-pirate` | Overspecified with 'you control'; several cards care about Pirate cards in graveyard, hand, or library, not just ones you control | Cares about Pirates, rewarding or interacting with Pirate cards you control or own. |
| `copy-legendary` | 'token' and 'you get to keep it' miss the cards that turn existing creatures into copies | Makes a nonlegendary copy of a permanent, usually as a token but sometimes by turning creatures you control into copies, sidestepping the legend rule. |
| `synergy-cycling` | 'or discarded' and 'triggers' overreach; several care only about cycling and some reward cycling cards statically | Cares about cycling and often discarding, usually triggering a bonus when you cycle or discard a card. |
| `legendary-team-up` | not every member is a creature; the pairing motif also covers noncreature legends | A legendary card, usually a creature, that pairs two named characters on one card. |
| `sliver-stackable` | 'benefits all Slivers' overstates scope; many members only affect Slivers you control | A Sliver whose ability benefits Slivers you control and stacks with each additional copy. |
| `creature-type-name` | Overspecifies to "a creature"; some tagged cards aren't creatures at all | A card whose name is made up entirely of Magic creature types. |
| `leaves-graveyard-trigger` | Overspecified to 'your' graveyard; a variant triggers off an opponent's graveyard | Triggers an effect whenever one or more cards leave a graveyard, usually your own. |
| `vanilla-aura` | Not every tagged Aura is purely a P/T modifier; some carry extra abilities | An Aura whose main effect is changing the enchanted creature's power and toughness. |
| `bushido` | Overspecified as +X/+X; the tag also includes block/blocked triggers that shift stats differently | Whenever this creature blocks or becomes blocked, its power and toughness change until end of turn, usually a Bushido +X/+X boost. |
| `impulse-artifact` | Says "into your hand" but several members put the artifacts onto the battlefield | Digs through the top cards of your library for artifacts, putting them into your hand or onto the battlefield. |

### Overturned by Verify (4) — flag rejected, keep current
- `tuck-self` — auditor: "instead of another zone" is vague filler; the defining trait is simply that the card returns itself to its library (top or shuffled in) | verifier: All members (Sensei's Divining Top on top, Elixir of Immortality/Black Sun's Zenith shuffled in) put themselves back to library; 'instead of another zone' is filler but not inaccurate
- `synergy-tapped` — auditor: "you control" overspecifies; some members care about tapped creatures generally, not just yours | verifier: ~10 of 12 (Lydia Frye, Oak Street Innkeeper, all web-slinging, etc.) specify 'tapped creatures you control'; Split Up is a minority exception
- `untapper-artifact` — auditor: 'a target artifact' excludes cards that untap all your artifacts | verifier: Common case is a single target: Voltaic Key '{1}, {T}: Untap target artifact', Manifold Key 'Untap another target artifact'; Unwinding Clock's untap-all is a lone outlier among 48
- `impact-effect` — auditor: 'a creature you control enters' is overspecified; the marquee card triggers on ANY creature entering | verifier: 11 of 12 read 'a creature/Zombie/Dragon you control enters' (Witty Roastmaster, Ayara, Corpse Knight, etc.); Pandemonium's any-creature trigger is a single outlier, so the 'you control' common case stands

---

## Batch: ranks 701-1000

300 re-audited (2x150 shards): **247 clean, 51 suspect, 2 wrong**; Verify overturned 6. Deeper into the tail the tags get more niche and the descriptions hold up better (only 2 wrong), but the same over/under-specification pattern persists: "itself"/"your"/"this creature" where the tag also covers other permanents or any player, and missing a defining detail (free recast, sacrifice-as-extra-cost).

> **Findings only, not applied.** `ORACLE_TAG_DESCRIPTIONS` unchanged.

### Wrong (2) — fix recommended

#### `keyword-soup`
- **current:** 
- **issue:** "its set's keyword abilities" is invented; it's a fixed evergreen keyword list, and cards often count/move/reference rather than gain them
- **example:** Odric, Blood-Cursed: "create X Blood tokens, where X is the number of abilities from among flying, first strike, double strike, deathtouch, haste, hexproof, indestructible, lifelink, menace, reach, trample, and vigilance found among creatures you control"
- **suggested fix:** References a long list of common keyword abilities like flying, first strike, deathtouch, and trample, often granting or counting them.
- **verify note:** Odric counts and Kathril moves counters over a fixed evergreen list (flying/first strike/deathtouch/trample), not 'its set's keyword abilities'

#### `harmonic`
- **current:** 
- **issue:** Says 'control both,' but most tagged cards care about artifacts and enchantments separately or via OR, not a joint condition
- **example:** Starnheim Courser: 'Artifact and enchantment spells you cast cost {1} less to cast.' and Flutterfox: 'As long as you control an artifact or enchantment, this creature has flying.'
- **suggested fix:** Cares about both artifacts and enchantments, often rewarding you for controlling or casting them.
- **verify note:** Flutterfox 'artifact or enchantment', Nezumi Bladeblesser and Shinechaser treat them separately, not a joint both-condition

### Suspect (51) — minor imprecision, review before applying

| slug | issue | suggested fix |
| --- | --- | --- |
| `references-keyword` | The restriction 'without actually having that ability itself' is false for several cards that reference a keyword they also possess/grant | Names a specific keyword or mechanic in its rules text, for example to grant it or care about it. |
| `pridemate` | Overspecified to 'itself'; many carriers put counters on other creatures, not just themselves | Puts a +1/+1 counter on a creature, often itself, whenever you gain life. |
| `counter-preservation-self` | "you control" restriction isn't universal; modular and several others target any creature | When it dies, moves its own +1/+1 counters onto another creature. |
| `modal-inverse-choices` | not all are spells; many are triggered/ETB abilities on permanents | A modal effect whose options are mirror opposites, like hitting fliers or hitting non-fliers. |
| `tutor-land-any` | "with a restriction or cost" isn't universal; several fetch freely, and some search onto the battlefield vs hand | Searches your library for any land card, often putting it onto the battlefield. |
| `ball-lightning` | underspecified: many are spells/abilities that MAKE such a token, not creatures themselves | A hasty, hard-hitting creature, or a token it creates, that gets sacrificed or exiled at end of turn. |
| `unique-protection` | "Grants" implies conferring on others, but most cards HAVE the unusual protection themselves (and a couple grant it to your team) | Has or grants protection from something unusual, like a chosen player or die rolls, rather than from a color. |
| `remove-from-stack` | Omits the tuck-to-library method: several cards put the spell on top/bottom of its owner's library, which is neither exile, bounce-to-hand, nor ending the turn | Removes a spell from the stack by exiling it, returning it to its owner's hand, or tucking it into their library, or by ending the turn. |
| `restock-self` | "hand" is unsupported; every card returns itself to the LIBRARY (via shuffle or tucking), typically from the graveyard or the stack, not to hand | Puts itself back into your library so you can draw and cast it again instead of losing it. |
| `combat-timing-restriction` | Overspecified: says 'a specific step of combat' but several cards only restrict to combat generally, not a named step | A spell you can cast only during combat, often only during a specific combat step. |
| `mm-counter-cost` | Says the -1/-1 counter is always a cost or requirement, but many carriers place it as an enters-the-battlefield drawback, not a cost | Puts a -1/-1 counter on a creature you control, either as a cost or as an enters-the-battlefield drawback. |
| `restock-creature` | Overspecified: says 'on top' but many taggers shuffle the creature into the library (or return it elsewhere) | Returns a creature card from your graveyard to your library, either on top or shuffled in. |
| `pwdeck-sidekick` | Underspecified: not all get stronger or gain an ability; some instead feed loyalty to the matching planeswalker | Synergizes with a specific named planeswalker you control, either gaining a bonus itself or supporting that planeswalker. |
| `painland` | Underspecified: about half the tagged lands cost life via a 'Pay N life' activation cost rather than dealing damage to you; description only mentions damage | A land that costs you life or deals damage to you when you tap it for mana. |
| `variable-effect-same-ability` | Overspecified to 'nth time resolved'; several tagged cards vary by a game-state condition instead, not resolution count | An ability whose effect changes based on a condition, often how many times it has resolved this turn. |
| `lobotomy` | "every copy" overstates; some cap the number | Exiles cards with a chosen name, usually every copy, from a player's hand, library, and graveyard. |
| `cycle-ust-functional-variant` | Variants share the same cost (only text differs), so "different cost" is contradicted; collector-number claim is dubious | An Unstable card printed in multiple versions that share a name and mana cost but have different rules text. |
| `ransom` | Not always "sacrifice a permanent"; several destroy or exile instead of sacrifice | Makes a player sacrifice, destroy, or exile a permanent unless they pay a cost. |
| `potentially-free` | Most carriers reduce their cost toward zero (affinity, cost-less) rather than skipping the mana cost entirely; description only fits the alternative-cost minority | Can potentially cost nothing to cast through cost reduction or an alternative cost. |
| `protects-land` | "your lands" overspecifies; some carriers protect lands any player controls | Shields lands, or all your permanents, from being destroyed or targeted. |
| `young-pyromancer-ability` | Underspecified: many tagged cards trigger on any noncreature spell, not just instants/sorceries | Creates a creature token whenever you cast an instant, sorcery, or other noncreature spell. |
| `whirlpool` | Frames it as self-only ('your hand', 'draws you'), but the signature and majority effect is symmetric (each player) | Shuffles hands and graveyards into libraries, then draws each player a fresh hand. |
| `landhome` | "needing it to attack" is misleading: the attack condition requires the DEFENDING player to control the land type, not the creature's controller | A creature that can't attack unless the defending player controls a certain land type, and often must be sacrificed if you control none of that type. |
| `hate-wide` | Overstates: many cards scale on ALL creatures on the battlefield (including yours), not just opponents', and several reward you rather than punish | Scales with the number of creatures on the battlefield, usually to punish go-wide boards. |
| `theft-permanent` | Overspecified: many target any permanent (not just opponents') and several swap control or take it only temporarily | Takes or swaps control of a permanent, usually an opponent's. |
| `5c-set-mechanic-commander` | Calls them five-color creatures, but many are mono-colored or colorless by card color (only their color identity is WUBRG) | A legendary creature with a five-color identity built as a commander to support a set's mechanics or themes. |
| `typal-cleric` | Elaboration ('growing stronger or gaining abilities') fits only a couple cards; most tap Clerics as a resource or count them, which isn't captured | Cares about Clerics, counting them or tapping them as a resource to power its effects. |
| `demilich-effect` | Omits the defining detail that the copy is cast WITHOUT paying its mana cost (a free recast), which undersells the whole payoff | Exiles a card from a graveyard and lets you cast a copy of it without paying its mana cost, usually just once. |
| `alternate-loss-condition` | Says only 'you' lose, but the tag also covers making opponents lose and cards that remove/replace the normal 0-life loss | Changes the game's loss rules, adding a new way a player can lose or removing the normal loss from having no life. |
| `alternate-cost-sacrifice` | Says sacrifice is paid 'instead of' mana, but several tagged cards make sacrifice an ADDITIONAL cost on top of full mana, or a cost reduction | Lets you sacrifice one or more permanents to help cast a spell, either instead of its mana cost or as an extra cost. |
| `lure` | says 'this creature' but most members apply the effect to a chosen/enchanted/equipped creature, not the source itself | Forces all creatures able to block a given creature, whether itself or one it targets, to do so. |
| `secretly-choose` | Overspecifies 'reveal simultaneously'; many cards make a lone hidden choice revealed later, not a synchronized reveal | Has players make a hidden choice or vote that they reveal later, sometimes all at once. |
| `damage-increaser` | 'your damage sources' overspecifies; some are symmetric and boost any player's sources | Makes damage sources deal extra damage on top of what they'd normally deal. |
| `hungry-demon` | 'unless you meet some condition' implies an out, but most are flat forced sacrifices each upkeep with no escape | Forces you to sacrifice a creature, usually at the beginning of each of your upkeeps. |
| `three-letter-name` | Split/multi-part cards carry the tag on a three-letter face, but their full name is longer, so 'whose name is exactly three letters long' is technically false for them | A card with a face whose name is exactly three letters long. |
| `graveyard-fuel-artifact` | Says 'your graveyard' but many cards target any graveyard | Spends or cares about artifact cards in a graveyard. |
| `copy-equipment` | Over-narrows to token creation; several members copy by entering as or becoming a copy, not by creating a token | Copies a permanent, usually by creating a token copy of it. |
| `synergy-face-down` | Underspecified: rewards face-down permanents entering AND turning them face up, not only creatures you put down face-down | Rewards you for having permanents enter face down or turning them face up, as with manifest and morph. |
| `hate-typal-wall` | "your opponents' Walls" overspecifies: the single-target destroy cards hit ANY Wall, not just opponents' | Lets a creature attack past Walls, or destroys or disables Walls. |
| `hate-island` | omits destruction, a prominent mode of the tag (lists only locking, damaging, bouncing) | Punishes or answers Islands and their controllers by destroying, locking, damaging, or bouncing them. |
| `recycle` | Says returns 'to the battlefield' but several members return to hand instead | Lets you sacrifice a permanent to return a card from your graveyard to the battlefield or your hand. |
| `sleeping-enchantment` | 'permanently' is wrong (one member reverts to enchantment) and 'trigger condition' misses members that flip via a paid cost | A dormant enchantment that becomes a creature when a condition is met or a cost is paid. |
| `provoke-lite` | Cards say "this turn" not "this combat," and several force a creature to block a DIFFERENT creature, not necessarily this one | Forces a target creature to block this turn if able, without untapping it first. |
| `synergy-first-strike` | Muddled; most cards grant or count first strike itself, not "grant abilities to creatures that already have first strike" | Rewards or grants first strike among your creatures. |
| `cranial-plating` | 'Gets stronger' implies the card boosts ITSELF, but the namesake and several members GRANT the artifact-scaled bonus to another creature (gives vs gains) | Makes a creature stronger based on how many artifacts you control. |
| `counts-as-a-type` | Claims the cards use visible 'deprecated wording', but the pulled cards show no such text; they simply carry an Oracle-granted creature type in their type line | An older creature that Oracle updates treat as having a creature type such as Wall or Sliver, letting it count for that type's effects. |
| `sneaky-self-trigger` | 'untaps or benefits itself' over-narrows the pattern; most cards trigger their OWN granted/static ability off their own event and the payoff may not be a self-benefit | Has an easy-to-miss ability that quietly triggers off a common event such as a permanent entering or dying. |
| `o-ring-with-set-mechanic` | "using another set's mechanic to power the effect" is misleading; the set mechanic is bundled onto the card, it doesn't power the exile | Exiles a permanent until this leaves the battlefield, bundled with a set specific mechanic. |
| `wheel` | "Has you discard" overspecifies; many wheels make another player discard and redraw, not you | Makes a player discard their hand, then draw that many cards or more. |
| `bounceable-aura` | Not always to hand; some return themselves to the library instead | An Aura with a way to return itself to its owner's hand or library. |
| `absorb` | "set amount" is misleading; several prevent a variable amount (X) or all-but-1, and some reduce rather than prevent | Prevents or reduces some of the damage that would be dealt to a permanent or player. |

### Overturned by Verify (6) — flag rejected, keep current
- `extract` — auditor: 'removing them from the game' is imprecise (exile is a real zone, not out of the game), and the discard contrast is odd; otherwise the exile-from-library idea is right | verifier: Cards do exile from a library (Extract, Mana Severance); 'removing from the game' is a rules-technicality quibble, not a proven factual error
- `blood-artist-ability` — auditor: Says 'an opponent' but the namesake and many carriers say 'target player' (any player) or 'each opponent', not a single opponent | verifier: 10 of 12 read 'each opponent'/'target opponent' loses life; only Blood Artist and Falkenrath Noble say 'target player', so 'an opponent' fairly describes the dominant case
- `ingest` — auditor: the damaged player exiles from their own library, and ingest exiles just the top card, not "cards" | verifier: Tag includes multi-card exilers (Raven Guild Master top ten, Kotis top X, Bismuth until nonland), so 'cards' plural is correct; damaged player is functionally the opponent attacked
- `sth-storyline-in-cards` — auditor: Overspecifies the mechanism as flavor text; it is a meta tag for cards tied to the Stronghold set's story, and the narrative link is not necessarily flavor text | verifier: Pulled data has only oracle text, no flavor text, so it cannot disprove the current 'flavor text is part of a storyline'; the auditor's objection is speculative and unproven
- `wind-drake-with-set-s-mechanic` — auditor: overspecifies exact 2/2 P/T (unverifiable in data) and 'flyer'; several members aren't 2/2 flying creatures at all | verifier: no P/T in data so '2/2' cannot be disproven (auditor admits it's unverifiable), and most pulled cards are 3-mana flyers matching the archetype; Cloudform is one outlier
- `devour` — auditor: "for each one" understates devour 2 and 3, which give twice or three times that many counters per sacrifice | verifier: Current 'gain +1/+1 counters for each one' fixes no ratio; devour 2/3 (Preyseizer, Gigantotherium) still gain counters for each creature sacrificed, so it is not clearly inaccurate.

---

## Applying corrections

**Status: ranks 1-1000 APPLIED 2026-07-16** (181 fixes: all 10 wrong + 171 suspect). Applied
programmatically slug-by-slug: each fix was gated for style (quote-free, no em dash, no link,
terminal period, <=200 chars) and only written when the const's current text still matched the
audited description (0 mismatches). `nightly fmt` clean. **The change is to the compiled const
only; the live `oracle_tags.description` column updates on the next `zervice` sync/deploy.** Future
batches (rank ~1001+) stay findings-only until a similar apply pass.

> Watch item: `unique-token` was flagged in both the forward audit and this re-audit with
> different suggested rewrites. The re-audit's ("A named token creature with its own defined
> characteristics.") was applied; if the forward audit's `predefined-token` reconciliation
> matters, eyeball it.

When applying (future batches):
- Treat every flag as a *suggestion*, not a mandate. Skip any suggested rewrite you disagree with.
- Re-verify a flag against Scryfall before applying if anything looks off (the auditor is an LLM).
- Keep the const's style: one plain sentence, no em dashes, no links, quote-free (the splice style
  gate rejects double-quotes).
