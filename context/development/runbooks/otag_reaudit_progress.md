# Oracle-tag description RE-AUDIT — progress & findings

Second-pass QA re-run of `ORACLE_TAG_DESCRIPTIONS` using the **improved two-stage
workflow** (card-data grounding + skeptical Verify stage, commit `99dc2091`). Ranks
1-2000 were originally checked by the *old* single-stage workflow that was blind to
cost/color/hybrid/rarity and over-generalized cycles; this re-audit re-checks them from
the top by population to catch what the old pass missed. Companion to the forward-audit
progress in [`otag_audit_progress.md`](otag_audit_progress.md).

> **This is the ACTIVE audit task.** Ignore the paused forward sweep in
> [`otag_audit_progress.md`](otag_audit_progress.md) (do not resume that at 2201). Continue
> the re-audit here from **rank ~501+**.

> **Findings only, human-in-the-loop. Do NOT apply to the const yet.** Nothing here is
> applied to `ORACLE_TAG_DESCRIPTIONS`. `wrong`/`suspect` are the *verified-surviving* flags.
> The plan is to **finish the full 0-all re-audit first, THEN review all corrections and apply
> them to the const in one deliberate pass** (see [Applying corrections](#applying-corrections)).
> Ships [`otag_audit_workflow.js`](otag_audit_workflow.js).

## Coverage / resume
- **Re-audited: 500 / 4,357** (ranks 1-500 by card population). **Next: rank ~501+.**
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
Across 500 re-audited: **416 clean, 81 suspect, 3 wrong**; the Verify stage overturned 6 auditor flags.

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

## Applying corrections

**Deferred until the full 0-all re-audit is done.** Do not apply flags piecemeal per batch.
Once the sweep covers the whole set, review all `wrong`/`suspect` corrections together and apply
the agreed ones to `ORACLE_TAG_DESCRIPTIONS` in one deliberate `fix(otags):` commit. Reasons to
batch it: some slugs are flagged in both the forward audit and this re-audit (e.g. `unique-token`)
and need reconciling once; and a single edit pass over the const is easier to review than many.

When applying:
- Treat every flag as a *suggestion*, not a mandate. Skip any suggested rewrite you disagree with.
- Re-verify a flag against Scryfall before applying if anything looks off (the auditor is an LLM).
- Keep the const's style: one plain sentence, no em dashes, no links, quote-free (the splice style
  gate rejects double-quotes).
