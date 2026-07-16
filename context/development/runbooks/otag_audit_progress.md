# Oracle-tag description audit — progress & findings

Independent second-pass QA of `ORACLE_TAG_DESCRIPTIONS`. Each batch re-checks authored
descriptions against real card oracle text ([`otag_audit_workflow.js`](otag_audit_workflow.js))
and flags inaccuracies with a card example and a suggested fix. **Findings are for review; fixes
are NOT auto-applied.**

## Coverage / resume

> **➡️ ACTIVE TASK IS THE RE-AUDIT. Next AI: resume at rank ~1001, NOT 2201.**
> The full 0-all **re-audit** (improved two-stage workflow) supersedes this forward sweep.
> Pick up in [`otag_reaudit_progress.md`](otag_reaudit_progress.md) and continue from rank
> ~1001+ (using [`otag_reaudit_slugs.txt`](otag_reaudit_slugs.txt)). Do **not** continue the
> forward sweep below unless the re-audit is abandoned.

- **This forward sweep is PAUSED at 2200 / 4,357.** It used a mix of old and improved workflow
  passes (ranks 1-2000 old, 2001-2200 improved). Superseded by the re-audit, which re-checks
  everything with the improved workflow from the top. Its old resume point was rank ~2201; ignore
  that in favor of the re-audit.
- **Full re-audit (0-all) IN PROGRESS.** Ranks 1-2000 were audited by the *old* single-stage
  workflow (no card-data grounding, no Verify stage), which [Batch 6](#batch-6-rank-20012200-improved-two-stage-workflow)
  proved was blind to cost/color/hybrid/rarity facts. Re-running those ranks through the improved
  two-stage workflow from the top, tracked separately in
  [`otag_reaudit_progress.md`](otag_reaudit_progress.md) /
  [`otag_reaudit_slugs.txt`](otag_reaudit_slugs.txt). **Done: ranks 1-1000** (178 flags the old
  pass missed). **Next: rank ~1001+.**

## Findings so far
Across 2200 audited: **1807 clean, 365 suspect, 27 wrong** (rank 2001-2200 added by
[Batch 6](#batch-6-rank-20012200-improved-two-stage-workflow); the first 2000 were
1632 clean / 345 suspect / 23 wrong).

**Second-pass on the 23 wrong (separate session, Scryfall-checked, 2026-07-15):** not all
suggested fixes are shippable. ~16–17 apply-worthy; 3 audit flags were themselves wrong
(`x-doesn-t-matter`, `cycle-war-hybrid-planeswalker`, `cycle-fdn-draft-signpost` — keep current);
2 need a different rewrite (`unique-token`, `substance`). Details under
[Wrong (fix recommended)](#wrong-fix-recommended). **No changes applied to
`ORACLE_TAG_DESCRIPTIONS`.**

## Batch 6 (rank 2001-2200, improved two-stage workflow)

First batch run with the **card-data grounding + Verify stage** (commit `99dc2091`). 200 audited: **175 clean, 20 suspect, 4 wrong**; the Verify stage **overturned 1** auditor flag.

**Regression signal (the fix worked).** This batch was deliberately dense with `cycle-*` tags and cost/color/rarity claims, the exact category that produced the 3 false flags in the first 2000. It produced **zero** false cost/color/hybrid flags. The opposite happened: the auditor now *uses* the pulled cost/colors to make correct calls it previously guessed wrong, e.g. flagging `cycle-clb-back-enemy-legend` as mono-colored via `colors [G]/[W]/[B]`, and `cycle-arb-u-hybrid-gold` as three-color via `{2}{G/U}{W}` `colors [G,U,W]`. Overturn rate 1/25 (~4%) with all survivors genuine, table below.

> **Findings only, not applied.** `ORACLE_TAG_DESCRIPTIONS` unchanged.

### Wrong (4) — fix recommended

#### `cycle-clb-back-enemy-legend`
- **issue:** Says 'enemy-colored' but every card is mono-colored
- **example:** Erinis, Gloom Stalker, cost {2}{G}, colors [G]; Rasaad yn Bashir {2}{W} [W]; Sarevok, Deathbringer {3}{B} [B]
- **suggested fix:** A mono-colored legendary creature that can take a Background as a second commander.
- **verify note:** All five are mono-colored: Rasaad [W], Livaan [R], Renari [U], Sarevok [B], Erinis [G]

#### `cycle-c13-face-commander`
- **issue:** Claims the deck is 'named after' the commander, but the Commander 2013 decks have theme names, not the commander's name
- **example:** Prossh, Skyraider of Kher is the face of the deck named 'Power Hungry'; Oloro, Ageless Ascetic fronts 'Eternal Bargain' (deck is not named after the commander)
- **suggested fix:** The headline legendary creature that a Commander 2013 precon deck is built around.
- **verify note:** C2013 decks carry theme names (Prossh=Power Hungry, Oloro=Eternal Bargain), not the commander's name

#### `cycle-c14-historical-legend`
- **issue:** Slug-name trap: these are new mono-color legends debuting in Commander 2014, not older cards reprinted
- **example:** Feldon of the Third Path | {1}{R}{R} | Legendary Creature — Human Artificer (first printed in Commander 2014, not a reprint)
- **suggested fix:** A mono-colored legendary creature first printed as a face card in the Commander 2014 decks.
- **verify note:** Feldon {1}{R}{R}, Titania, Gisa, Geralf, Jazal all debuted in Commander 2014 and are mono-colored, not reprinted older cards

#### `cycle-arb-u-hybrid-gold`
- **issue:** Says 'two-color combination' but every card is actually three colors (a hybrid pip plus another colored pip spanning a shard)
- **example:** Messenger Falcons, cost {2}{G/U}{W}, colors [G,U,W]; Slave of Bolas, cost {3}{U/R}{B}, colors [B,R,U]
- **suggested fix:** An uncommon three-color card whose cost mixes a hybrid mana symbol with a colored pip, spanning a shard's colors.
- **verify note:** Messenger Falcons {2}{G/U}{W} colors [G,U,W] is three colors, contradicting 'two-color combination'

### Suspect (20) — minor imprecision, review before applying

| slug | issue | suggested fix |
| --- | --- | --- |
| `typal-assembly-worker` | "creating them" is unsupported; members pump, search for, or animate lands into Assembly-Workers, none create tokens | Cares about Assembly-Worker creatures, pumping them, searching them out, or animating lands into them. |
| `type-removal-cat-rakshasa` | there is no "Rakshasa" creature type; these are Demons (one is a Devil) that lost the Cat type they historically carried | A rakshasa-flavored Demon or Devil printed without the Cat creature type it historically carried. |
| `typal-sneaky` | 'together' wrongly implies you need both types; cards reward Ninjas or Rogues individually | Cares about Ninja and Rogue creatures, rewarding you for controlling and attacking with them. |
| `you-matter` | 'physical traits' overspecifies; several cards care about real-world facts that aren't physical (convention attendance, a guessing game) | Cares about you the real player, such as your height, your name, or what you know or have done. |
| `synergy-nonbasic-land` | 'rewarding you for controlling or finding them' over-narrows; some cards punish/destroy nonbasic lands or care about their types | Cares about nonbasic lands, such as by counting them, searching them out, or destroying them. |
| `cycle-bro-m-color-artifact` | Overspecifies 'with prototype'; not every member has prototype (one has unearth instead) | An artifact creature themed to a single color, usually with prototype so you can cast it smaller and cheaper in that color. |
| `cycle-c20-monster-partner` | 'grows through counters' is overspecified; 2 of 5 don't put counters on themselves | The larger, two-color half of a Commander 2020 Partner With legendary pair. |
| `bottom-deck-manipulation` | Says 'your library' and frames it as pure draw-filtering, but some members bottom an opponent's library or bottom cards from hand | Puts cards on the bottom of a library, often to filter your draws by tucking away the cards you don't keep. |
| `creature-type-phantasm` | "easily destroyed" isn't broadly true; several members have no fragility | An Illusion creature, usually with flying and sometimes carrying a self-sacrifice drawback. |
| `cycle-a25-u-legend` | Rarity claim unverified: pulled data shows two members as rare, not uncommon; 'anthology' also mislabels Masters 25 | A mono-colored legendary creature reprinted in the Masters 25 set. |
| `cycle-chk-myojin` | It only enters indestructible when cast from hand (gets the divinity counter only then), so 'enters indestructible' overstates the general case | A Spirit that enters indestructible when cast from hand and removes its divinity counter for a huge one-time effect. |
| `cycle-c21-technique` | Demonstrate copying is optional ('you may copy'), but the description states it as automatic | A sorcery with demonstrate: when you cast it you may copy it, and if you do an opponent also copies it. |
| `cycle-c16-r-partner` | Calls it the "rare cycle" but the pulled members are mixed rare and mythic | A two-color legendary creature with partner, letting it share the command zone with another partner commander. |
| `becomes-changeling` | "Turn into a creature" misses the member that is already a creature and merely gains all creature types | A permanent that becomes, or can become, a creature with all creature types. |
| `cycle-ala-r-tricolor` | Says 'rare' but the cycle includes an uncommon card, so the rarity claim is inaccurate | A signature three-color card from Alara Reborn representing a single shard's color combination. |
| `cycle-all-enemy-hate` | Overspecified to "controlling lands or permanents"; some members answer spells or trigger on casting/entering, not on control | A mono-colored card that punishes or answers its two enemy colors' lands, permanents, or spells. |
| `cycle-apc-sanctuary` | checks two OTHER colors, not 'its own colors'; each is a single-color enchantment keyed to two different colors | An enchantment that triggers a small effect each upkeep, larger if you control permanents of both of two other colors. |
| `cycle-arb-c-hybrid-gold` | Says 'color pair' but these are three-color cards (one hybrid symbol plus a colored pip), and it omits the defining hybrid mana symbol | A common multicolor card whose cost includes one hybrid mana symbol, each with its own effect. |
| `cycle-bbd-c-two-color` | Overspecified: not all common (one is uncommon) and not all have an ability (one is vanilla) | A two-color creature or spell from a Battlebond reprint cycle. |
| `cycle-blb-c-gift` | 'for an added effect' overspecifies: on one member the gift instead removes a drawback rather than adding a bonus | Lets you promise an opponent a token as you cast this spell to unlock a bonus or avoid a drawback. |

### Overturned by Verify (1) — flag rejected, keep current
- `cycle-chk-legendary-land` — auditor: says 'aid a legendary creature' but one member targets any legendary permanent, and the effects only sometimes aid | verifier: All five abilities aid (fear, damage prevention, +1/+1, first strike, untap); auditor's 'only sometimes aid' is false and 4/5 target a legendary creature, so the common-case description holds

---

## Wrong (fix recommended)

> **Second-pass verification (separate session, 2026-07-15):** Scryfall oracle + `otag:` membership
> re-check of every entry below. **Do not treat all 23 suggested fixes as ready to ship.**
> Const (`ORACLE_TAG_DESCRIPTIONS`) was **not** changed by that session or this note update.
>
> | Bucket | Slugs |
> | --- | --- |
> | **Suggested fix OK** (apply-worthy) | `minigame`, `warlord`, `creature-ability-noncreature`, `hate-wide` (nit), `keyword-soup`, `predefined-token`, `impulse-planeswalker`, `references-keyword`, `sunder`, `counterspell-enchantment`, `gives-wither`, `synergy-protection`, `cycle-mid-r-flashback`, `cycle-mh3-c-draft-signpost`, `cycle-block-rtr-off-color`, `cycle-unf-single-sticker`, `cycle-block-rav-mnn` (example partially wrong), `cycle-mh3-r-m-two-color` (Devoid nit) |
> | **Do not apply suggested; audit flag wrong — keep current** | `x-doesn-t-matter`, `cycle-war-hybrid-planeswalker`, `cycle-fdn-draft-signpost` |
> | **Do not apply suggested; needs different rewrite** | `unique-token` (collides with `predefined-token`), `substance` (keyword vs flash package) |
>
> Per-entry **second-pass** notes are under each slug. Original audit lines kept for history.

### `unique-token` — WRONG
- **current:** A specific token variant that only one card creates.
- **issue:** claims 'only one card creates' it, but the tagged tokens are the most generic ones made by hundreds of cards
- **example:** Soldier (Token Creature, Vigilance) and Wolf/Sheep/Insect tokens are produced by many different cards, not one
- **suggested fix:** A predefined, named token creature that cards put onto the battlefield.
- **second-pass (separate session, Scryfall-checked):** DO NOT APPLY suggested fix
  - Separate session (Scryfall `otag:unique-token include:extras`, ~381 token objects): original's 'only one card creates' is too strong, but the suggested fix conflates this with `predefined-token` (Blood/Clue/Food/Gold…). Needs a different rewrite — e.g. nonstandard/unique token types vs CR predefined ones — not the suggested text.

### `minigame` — WRONG
- **current:** Engages other players in a bet, vote, or guessing game.
- **issue:** Overspecifies 'other players'; many are solo minigames or involve a person outside the game
- **example:** Push Your Luck: 'Reveal cards from the top of your library until you decide to stop' is a solo press-your-luck game; Scavenger Hunt: 'You have ten seconds to search your library' is a solo timed search, neither engages other players
- **suggested fix:** Creates a real-world minigame such as a timed search, press-your-luck, vote, or guessing challenge to determine its effect.
- **second-pass (separate session, Scryfall-checked):** SUGGESTED FIX OK
  - Confirmed: Push Your Luck / Scavenger Hunt are solo Unfinity attractions; 'other players' overspecifies. Suggested fix is good.

### `warlord` — WRONG
- **current:** A creature whose power, and sometimes toughness, equals how many creatures you control.
- **issue:** Overspecifies to 'creatures you control'; over half the cards scale off other things (lands, color-based permanents, Clerics)
- **example:** Ashaya, Soul of the Wild: 'power and toughness are each equal to the number of lands you control'; Kithkin Rabble: 'equal to the number of white permanents you control'
- **suggested fix:** A creature whose power, and often toughness, equals the number of creatures or other permanents you control.
- **second-pass (separate session, Scryfall-checked):** SUGGESTED FIX OK
  - Confirmed via `otag:warlord` (includes Ashaya / lands, Kithkin Rabble / white permanents). Suggested fix matches the tagged set better than creature-count-only.

### `creature-ability-noncreature` — WRONG
- **current:** A noncreature permanent with abilities that matter once it becomes a creature.
- **issue:** Slug-name trap: most cards never become creatures; the theme is a noncreature carrying a creature-style ability/keyword
- **example:** Weapon Rack (Artifact): 'enters with three +1/+1 counters... Move a +1/+1 counter from this artifact onto target creature' and Darkmoss Bridge (Artifact Land): 'Indestructible' — neither becomes a creature
- **suggested fix:** A noncreature permanent that has an ability or keyword normally found on creatures, such as indestructible or +1/+1 counters.
- **second-pass (separate session, Scryfall-checked):** SUGGESTED FIX OK
  - Confirmed: Darkmoss Bridge (indestructible), Weapon Rack (+1/+1 counters) never become creatures. Suggested fix is good.

### `hate-wide` — WRONG
- **current:** Scales up to punish opponents based on how many creatures they control.
- **issue:** Says punishes opponents by THEIR creature count; most cards scale with all creatures on the battlefield and some benefit you rather than punish
- **example:** Chain Reaction: 'deals X damage to each creature, where X is the number of creatures on the battlefield'; War Report gains you life per creature
- **suggested fix:** Scales with the number of creatures on the battlefield, often to answer go-wide boards.
- **second-pass (separate session, Scryfall-checked):** SUGGESTED FIX OK (minor nit)
  - Confirmed: tag mixes board-scale answers (Chain Reaction) with non-hate scale effects (War Report). Suggested fix is better; still leans 'answer go-wide' while some cards only scale with creatures.

### `keyword-soup` — WRONG
- **current:** A card that gains or lists most of its set's keyword abilities.
- **issue:** 'its set's keyword abilities' is misleading; cards reference a fixed list of evergreen keywords, not any expansion's keywords
- **example:** Odric, Blood-Cursed: counts 'flying, first strike, double strike, deathtouch, haste, hexproof, indestructible, lifelink, menace, reach, trample, and vigilance'
- **suggested fix:** Grants, counts, or keys off a long list of evergreen keyword abilities such as flying, first strike, and trample all at once.
- **second-pass (separate session, Scryfall-checked):** SUGGESTED FIX OK
  - Confirmed: Odric, Blood-Cursed lists evergreen keywords, not 'this set's' keywords. Suggested fix is good.

### `predefined-token` — WRONG
- **current:** Creates a specific named token with a fixed, preset set of abilities.
- **issue:** tag marks the token cards themselves, which do not 'create' anything; description describes a creator instead of the token
- **example:** Treasure — 'Token Artifact — Treasure: {T}, Sacrifice this token: Add one mana of any color' (the card IS the token, it creates nothing)
- **suggested fix:** Is a predefined token with a standard, preset set of abilities, such as Treasure, Blood, Clue, or a Role.
- **second-pass (separate session, Scryfall-checked):** SUGGESTED FIX OK
  - Confirmed via `otag:predefined-token include:extras` (Blood, Clue, Food, Gold, …): tags the token objects themselves, not creators. Suggested fix is good.

### `impulse-planeswalker` — WRONG
- **current:** Reveals cards from the top of your library and lets you take a noncreature, nonland card.
- **issue:** overspecifies to "noncreature, nonland" but most cards grab creatures, planeswalkers, Auras, or lands
- **example:** Oath of Nissa: "You may reveal a creature, land, or planeswalker card from among them and put it into your hand"; Ajani, Mentor of Heroes: "reveal an Aura, creature, or planeswalker card"
- **suggested fix:** Looks at cards from the top of your library and lets you put a revealed card of certain types, often a planeswalker or other permanent, into your hand.
- **second-pass (separate session, Scryfall-checked):** SUGGESTED FIX OK
  - Confirmed: Oath of Nissa (creature/land/PW), Ajani, Mentor of Heroes (Aura/creature/PW). Original 'noncreature, nonland' is wrong. Suggested fix is good.

### `references-keyword` — WRONG
- **current:** Mentions a keyword or mechanic by name without actually having that ability itself.
- **issue:** Most examples reproduce/allude to a keyword's effect without naming it, and the 'behold' cards actually perform the action; both halves of the description misfire
- **example:** Persist (sorcery): 'Return target nonlegendary creature card from your graveyard to the battlefield with a -1/-1 counter on it.' — reproduces the persist keyword but never names it
- **suggested fix:** Reproduces or alludes to a named keyword ability without actually having that keyword.
- **second-pass (separate session, Scryfall-checked):** SUGGESTED FIX OK
  - Confirmed: Persist (sorcery) reproduces persist without naming it. Suggested fix is good ('named keyword' means a keyword that has a name, not that the card must print the name).

### `substance` — WRONG
- **current:** An obsolete ability from old Mirage-era cards that does nothing at all.
- **issue:** Claims it "does nothing at all"; the ability is the old Mirage-era flash rule that lets you cast at instant speed with a cleanup sacrifice, so it functions as a combat trick
- **example:** Lightning Reflexes: "You may cast this spell as though it had flash. If you cast it any time a sorcery couldn't have been cast, the controller of the permanent it becomes sacrifices it at the beginning of the next cleanup step."
- **suggested fix:** Uses the old Mirage-era flash rules, letting you cast it at instant speed but sacrificing it at the next cleanup if cast when a sorcery couldn't be.
- **second-pass (separate session, Scryfall-checked):** REWRITE — do not apply suggested as-is
  - Separate session: Substance is an obsolete keyword with no rules meaning; the Mirage flash/cleanup package is separate oracle text (Lightning Reflexes, Armor of Thorns, Necromancy). Current is pedantically correct for the keyword; suggested is better UX but misattributes the flash rules as 'substance'. Prefer something like: obsolete Mirage-era keyword (no effect); these cards use the old flash-at-instant-speed, sacrifice-at-cleanup package.

### `x-doesn-t-matter` — WRONG
- **current:** Has an {X} in its cost that the card's own text never directly references.
- **issue:** most tagged cards have no {X} in their cost; they care about the amount or colors of mana spent to cast them
- **example:** Prismatic Ending (cost {W}, no X): 'Exile target nonland permanent if its mana value is less than or equal to the number of colors of mana spent'; Gyrus enters with counters 'equal to the amount of mana spent to cast it'
- **suggested fix:** Cares about how much mana or how many colors of mana were spent to cast it.
- **second-pass (separate session, Scryfall-checked):** DO NOT APPLY — audit flag itself is wrong; keep current
  - Separate session checked all 12 `otag:x-doesnt-matter` cards: every one has {X} in the cost and never references X in oracle text (Prismatic Ending is {X}{W}, not {W}). Original description matches the tag. Suggested fix renames the concept to 'cares about mana/colors spent' — a common effect pattern, not this tag's definition. Issue/example are factually wrong.

### `sunder` — WRONG
- **current:** Destroys all Auras or Equipment attached to a permanent, leaving the permanent itself behind.
- **issue:** 'leaving the permanent itself behind' is false for many carriers that also destroy/exile the permanent
- **example:** Silence the Believers: 'Exile any number of target creatures and all Auras attached to them'; End Hostilities: 'Destroy all creatures and all permanents attached to creatures'
- **suggested fix:** Destroys or exiles the Auras and Equipment attached to a permanent, sometimes removing the permanent as well.
- **second-pass (separate session, Scryfall-checked):** SUGGESTED FIX OK
  - Confirmed: mix of leave-the-permanent (Strip Bare) and also remove the permanent (End Hostilities, Silence the Believers). Suggested fix is good.

### `counterspell-enchantment` — WRONG
- **current:** Counter magic that specifically stops enchantment spells from resolving.
- **issue:** 'specifically' wrongly implies enchantment-only; most examples counter enchantment among other spell types
- **example:** Swan Song: 'Counter target enchantment, instant, or sorcery spell.' (also Deny the Divine: creature or enchantment)
- **suggested fix:** Counters a target enchantment spell, often alongside other spell types.
- **second-pass (separate session, Scryfall-checked):** SUGGESTED FIX OK
  - Confirmed: Swan Song / Deny the Divine / Annul counter enchantment among other types; only some are enchantment-only. Suggested fix is good.

### `cycle-block-rav-mnn` — WRONG
- **current:** One of a cycle of two-color guild creatures from the Ravnica block, one for each guild.
- **issue:** Says 'two-color guild creatures' but the cycle includes non-creatures and mono-color cards
- **example:** Voidslime is 'Instant — Counter target spell, activated ability, or triggered ability'; Anthem of Rakdos is an Enchantment; Phytohydra is mono-white
- **suggested fix:** One of a cycle of Ravnica block cards of various types, one associated with each guild.
- **second-pass (separate session, Scryfall-checked):** SUGGESTED FIX OK (example partially wrong)
  - Noncreatures in cycle confirmed (Anthem of Rakdos enchantment). Audit example 'Phytohydra is mono-white' is false — Phytohydra is {2}{G}{W}{W}. Suggested fix (drop 'creatures' / allow various types) is still directionally fine.

### `cycle-block-rtr-off-color` — WRONG
- **current:** One of a cycle of Ravnica cards with a mana ability in a color outside their guild's pair.
- **issue:** not a "mana ability" and not "outside their guild's pair"; it's an activated ability costing the guild's OTHER color
- **example:** Frilled Oculus (mono-blue Simic): "{1}{G}: This creature gets +2/+2" — green is inside the Simic pair, and this produces no mana
- **suggested fix:** A monocolored Ravnica guild card whose activated ability costs mana of its guild's other color.
- **second-pass (separate session, Scryfall-checked):** SUGGESTED FIX OK
  - Confirmed: Frilled Oculus mono-U with {1}{G} activated ability — not a mana ability; other color is in-guild. Suggested fix is good.

### `cycle-fdn-draft-signpost` — WRONG
- **current:** A gold card from Foundations designed to point drafters toward its two-color archetype.
- **issue:** not all are gold cards; several signposts are mono-colored
- **example:** Good-Fortune Unicorn is a mono-white Unicorn ({2}{W}), not a gold card
- **suggested fix:** An uncommon Foundations card built to point drafters toward its two-color archetype.
- **second-pass (separate session, Scryfall-checked):** DO NOT APPLY — audit flag itself is wrong; keep current
  - Separate session: all 10 `otag:cycle-fdn-draft-signpost` cards are gold uncommons (Good-Fortune Unicorn is {1}{G}{W}, not mono-white {2}{W}). Original 'gold card…' is correct. Suggested 'uncommon…' is true but not a needed fix; issue/example are factually wrong.

### `cycle-mh3-c-draft-signpost` — WRONG
- **current:** One of a cycle of common creatures that signal a two-color draft archetype.
- **issue:** Not all creatures nor all two-color; cycle includes an Equipment and colorless Eldrazi
- **example:** Cranial Ram is 'Artifact — Equipment' (Living weapon), not a creature; Snapping Voidcraw / Writhing Chrysalis are colorless devoid Eldrazi
- **suggested fix:** One of a cycle of common cards from Modern Horizons 3 that signals a draft archetype.
- **second-pass (separate session, Scryfall-checked):** SUGGESTED FIX OK
  - Confirmed: Cranial Ram is Artifact — Equipment (Living weapon), not only creatures. Suggested fix is good.

### `cycle-mh3-r-m-two-color` — WRONG
- **current:** One of Modern Horizons 3's rare or mythic two-color creatures.
- **issue:** Overspecified as 'creatures'; cycle includes noncreature instants (one is colorless devoid, not two-color)
- **example:** Abstruse Appropriation is an 'Instant' with Devoid (colorless); Invert Polarity is also an Instant, not a creature
- **suggested fix:** One of Modern Horizons 3's rare or mythic two-color cards.
- **second-pass (separate session, Scryfall-checked):** SUGGESTED FIX OK with nit
  - Right to drop 'creatures' (Invert Polarity, Abstruse Appropriation are instants). Nit: suggested still says 'two-color' but Abstruse Appropriation has Devoid (colorless). Better tweak if applying: rare/mythic MH3 cards that use two colors of mana in their cost.

### `cycle-mid-r-flashback` — WRONG
- **current:** A rare sorcery you can cast once from your graveyard for its flashback cost, then exile.
- **issue:** Says 'sorcery' but the cycle includes instants
- **example:** Siphon Insight is type Instant with 'Flashback {1}{U}{B}'; Rite of Harmony is also an Instant with flashback
- **suggested fix:** A rare instant or sorcery you can cast once from your graveyard for its flashback cost, then exile.
- **second-pass (separate session, Scryfall-checked):** SUGGESTED FIX OK
  - Confirmed: cycle includes instants (Rite of Harmony, Galvanic Iteration). Suggested fix is good.

### `cycle-war-hybrid-planeswalker` — WRONG
- **current:** One of War of the Spark's hybrid-mana planeswalkers, castable with either of two colors.
- **issue:** Not hybrid mana; these require both colors, not 'either'
- **example:** Kiora, Behemoth Beckoner costs {2}{G}{U} (both green and blue required, no hybrid symbols)
- **suggested fix:** One of War of the Spark's two-color planeswalkers, requiring mana of both its colors to cast.
- **second-pass (separate session, Scryfall-checked):** DO NOT APPLY — audit flag itself is wrong; keep current
  - Separate session: Kiora, Behemoth Beckoner is {2}{G/U} hybrid (not {2}{G}{U}). Entire `otag:cycle-war-hybrid-planeswalker` cycle uses hybrid mana (Ashiok {1}{U/B}{U/B}, Dovin {2}{W/U}, etc.). Original is correct; suggested 'requires both colors' is false for hybrid.

### `gives-wither` — WRONG
- **current:** Grants wither to a creature, so its combat damage lands as -1/-1 counters instead.
- **issue:** says 'combat damage' but wither affects all damage the creature deals to creatures, not just combat, and only damage to creatures becomes counters
- **example:** Fists of the Demigod: 'it gets +1/+1 and has wither. (It deals damage to creatures in the form of -1/-1 counters.)' — no 'combat' restriction; Fang Skulkin grants it to enable ping/fight damage too
- **suggested fix:** Grants wither, so the creature's damage to other creatures is dealt as -1/-1 counters instead of normal damage.
- **second-pass (separate session, Scryfall-checked):** SUGGESTED FIX OK
  - Confirmed: Fists of the Demigod reminder text — wither is all damage to creatures, not combat-only. Suggested fix is good.

### `synergy-protection` — WRONG
- **current:** Rewards you for having creatures with the protection keyword.
- **issue:** Most tagged cards grant or copy the protection keyword rather than reward you for already having protection creatures
- **example:** Cairn Wanderer: 'As long as a creature card with protection is in a graveyard, this creature has protection.' (also Death-Mask Duplicant, Eater of Virtue, Priest of Possibility copy protection among a keyword list)
- **suggested fix:** Cares about the protection keyword, often granting protection or copying it from other creatures.
- **second-pass (separate session, Scryfall-checked):** SUGGESTED FIX OK
  - Confirmed: Cairn Wanderer copies protection from graveyard; not mainly 'reward for having protection'. Suggested fix is good.

### `cycle-unf-single-sticker` — WRONG
- **current:** A creature that gives you ten tickets on entering and lets you put a sticker on a nonland permanent you own.
- **issue:** says 'ten tickets' but the card gives one ticket ({TK})
- **example:** Aerialephant: 'When Aerialephant enters, you get {TK}, then you may put a sticker on a nonland permanent you own.'
- **suggested fix:** A creature that gives you a ticket when it enters and lets you put a sticker on a nonland permanent you own.
- **second-pass (separate session, Scryfall-checked):** SUGGESTED FIX OK
  - Confirmed: Aerialephant gives {TK} (one ticket), not ten. Suggested fix is good.

- **current:** A creature that gives you ten tickets on entering and lets you put a sticker on a nonland permanent you own.
- **issue:** says 'ten tickets' but the card gives one ticket ({TK})
- **example:** Aerialephant: 'When Aerialephant enters, you get {TK}, then you may put a sticker on a nonland permanent you own.'
- **suggested fix:** A creature that gives you a ticket when it enters and lets you put a sticker on a nonland permanent you own.

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

### `type-addition-noble` — suspect
- **current:** A creature type tag for cards that gained the Noble creature subtype by errata.
- **issue:** 'by errata' is an unverifiable overclaim; several carry Noble as a natural printed subtype, and the sibling tag (sorcerer) is worded without the errata claim
- **example:** Falkenrath Noble (Creature - Vampire Noble), printed with the Noble subtype natively, not gained by errata
- **suggested fix:** A creature that also has the Noble creature type.

### `buttstrike` — suspect
- **current:** Makes a creature assign combat damage equal to its toughness instead of its power.
- **issue:** underspecified: nearly all these cards affect every creature you control (or all creatures), not just 'a creature'
- **example:** High Alert: 'Each creature you control assigns combat damage equal to its toughness rather than its power.'
- **suggested fix:** Makes creatures you control assign combat damage equal to their toughness instead of their power.

### `old-damage-deathtouch` — suspect
- **current:** Destroys any creature it deals combat damage to, an older way of writing deathtouch.
- **issue:** 'it deals combat damage' overspecifies; many cards grant the destroy-on-combat-damage effect to other creatures, not the tagged card itself
- **example:** Toxin Sliver: 'Whenever a Sliver deals combat damage to a creature, destroy that creature' (Sosuke triggers on Warriors you control, not itself)
- **suggested fix:** Destroys a creature that's dealt combat damage by the relevant creature, an older basilisk-style forerunner of deathtouch.

### `tutor-instant` — suspect
- **current:** Searches your library for an instant card and puts it into your hand or exile.
- **issue:** Says 'your library' but some tutors search an opponent's library
- **example:** Knowledge Exploitation: 'Search target opponent's library for an instant or sorcery card. You may cast that card without paying its mana cost.'
- **suggested fix:** Searches a library for an instant card, then puts it into your hand or exile.

### `life-divider-you` — suspect
- **current:** Makes a player lose, draw, discard, or sacrifice roughly half of something, rounded up.
- **issue:** Not always half; some cards divide by a third, so "roughly half" is imprecise
- **example:** Pox: "Each player loses a third of their life, then discards a third of the cards in their hand, then sacrifices a third..."
- **suggested fix:** Makes a player lose, draw, discard, or sacrifice a fraction (usually half) of something, rounded up.

### `pariah` — suspect
- **current:** Redirects damage that would be dealt to you onto a creature instead.
- **issue:** Underspecified: several cards redirect damage away from a creature (not from you) onto another creature
- **example:** Hazduhr the Abbot: "The next X damage that would be dealt this turn to target white creature you control is dealt to Hazduhr instead."
- **suggested fix:** Redirects damage that would be dealt to you or a creature onto another creature instead.

### `removal-equipment` — suspect
- **current:** Removal aimed specifically at destroying or exiling Equipment.
- **issue:** "specifically" overstates; several are creature removal or board wipes that only destroy attached Equipment as a rider
- **example:** End Hostilities: "Destroy all creatures and all permanents attached to creatures." (also Turn to Slag, Soul Nova, Thief of Existence)
- **suggested fix:** Destroys or exiles Equipment, sometimes along with the creature it's attached to.

### `seek-creature` — suspect
- **current:** Puts a random creature card from your library into your hand.
- **issue:** overspecifies "into your hand"; many of these seek a creature card straight onto the battlefield
- **example:** Naktamun Shines Again: "Seek a creature card with mana value 2 or less and put it onto the battlefield." (also Vhal, Inquisitor Captain)
- **suggested fix:** Finds a creature card at random from your library, putting it into your hand or onto the battlefield.

### `synergy-enchantment-creature` — suspect
- **current:** Cares about or boosts enchantment creatures you control.
- **issue:** "you control" overspecifies; some care about any enchantment creature or ones in the graveyard, not just yours
- **example:** Bearer of Memory: "Put a +1/+1 counter on target enchantment creature." (any); Odunos River Trawler returns an enchantment creature card from your graveyard
- **suggested fix:** Cares about or boosts enchantment creatures.

### `typal-phyrexian` — suspect
- **current:** Cares about or boosts Phyrexian creatures you control.
- **issue:** Overspecified 'you control'; tag also covers cards caring about Phyrexians regardless of controller
- **example:** That's No Moonmist: 'Transform all artifacts and Phyrexian creatures... Prevent all combat damage... by creatures other than Phyrexians' (affects all players' Phyrexians)
- **suggested fix:** Cares about or rewards Phyrexian creatures.

### `conjure-artifact` — suspect
- **current:** Creates a specific artifact card from outside the game in your hand or on the battlefield.
- **issue:** Misses graveyard as a destination and the conjured card is not always an artifact
- **example:** Fallaji Antiquarian: 'conjure a duplicate of another target nontoken creature or artifact you control into your graveyard'
- **suggested fix:** Conjures a specific named artifact from outside the game onto the battlefield, into your hand, or into your graveyard.

### `synergy-reach` — suspect
- **current:** Grants or counts reach along with other keyword abilities found among your creatures.
- **issue:** Says keywords are found 'among your creatures', but sources are often graveyard, library, hand, or draft
- **example:** Bleeding Effect: 'creatures you control gain flying ... if a creature card in your graveyard has flying'; Priest of Possibility checks the top seven cards of your library
- **suggested fix:** Grants or counts reach along with a batch of other keyword abilities shared across your creatures, graveyard, or library.

### `un-forecast` — suspect
- **current:** An Un-set card whose mechanic later paved the way for a black-bordered one.
- **issue:** Overreaching, unverifiable historical claim; several of these silver-bordered joke cards never foreshadowed any black-bordered mechanic
- **example:** Volrath's Motion Sensor: 'balances this Aura on the back of that hand' (a dexterity gag that never went black-bordered)
- **suggested fix:** An Un-set (silver-bordered) card, often with an offbeat mechanic that later helped inspire a black-bordered design.

### `artist-matters` — suspect
- **current:** Cares about the artist who illustrated a card, rewarding or punishing shared artwork.
- **issue:** 'shared artwork' undersells it; most cards care about a single chosen/named artist, not two cards sharing one
- **example:** Persecute Artist: 'Choose an artist... discards all nonland cards with art by the chosen artist' (chosen artist, not shared)
- **suggested fix:** Cares about which artist illustrated a card, rewarding or punishing cards drawn by a chosen or shared artist.

### `fateseal` — suspect
- **current:** Lets you look at an opponent's top library card and choose to leave it or put it on the bottom.
- **issue:** Says 'opponent' and only 'bottom', but many tagged cards target any player and mill/exile instead
- **example:** Draugr Thought-Thief: 'look at the top card of target player's library. You may put that card into their graveyard.'; Sealed Fate exiles one of the top X cards
- **suggested fix:** Lets you look at cards on top of a player's library and decide their fate, such as putting them on the bottom or into the graveyard.

### `legends-retold` — suspect
- **current:** Part of a special set-booster cycle reimagining classic legendary creatures from the original Legends set.
- **issue:** 'set-booster cycle' overspecifies/likely wrong product framing; these appear across Commander/bonus products, not set boosters
- **example:** Tetsuo, Imperial Champion / Jedit Ojanen, Mercenary / Ayesha Tanaka, Armorer are all updated versions of original Legends (1994) characters, but not a 'set-booster cycle'
- **suggested fix:** Reimagines a classic legendary creature from the original Legends set as a new card.

### `radiate` — suspect
- **current:** Copies a single-target spell and points the copy at something else it could hit.
- **issue:** underspecifies the signature mass-copy: most copy for EACH other legal target, not just one redirected copy
- **example:** Radiate: 'Copy that spell for each other permanent or player the spell could target. Each copy targets a different one of those'
- **suggested fix:** Copies a spell that targets a single permanent or player, making a copy aimed at each other legal target.

### `zoo` — suspect
- **current:** Creates multiple creature tokens of different types.
- **issue:** "different types" is overspecified; some make same-type tokens
- **example:** Triplicate Titan: creates three Golem tokens (same type) differing only in flying/vigilance/trample
- **suggested fix:** Creates several creature tokens at once, usually of varying kinds.

### `burn-bright-with-set-mechanic` — suspect
- **current:** Gives your team +2/+0 for the turn while also plugging into the set's mechanic.
- **issue:** Overspecifies buff and target: not always +2/+0, and not always 'your team'
- **example:** Weapon Surge grants only '+1/+0 and gains first strike'; Dinosaur Stampede pumps 'Attacking creatures get +2/+0' (any player's attackers), not just yours
- **suggested fix:** Pumps creatures' power (typically +2/+0) for the turn while also feeding the set's mechanic.

### `hate-flash` — suspect
- **current:** Stops or taxes opponents from casting spells or acting outside their own turn.
- **issue:** Says 'opponents' but several cards tax/restrict ALL players symmetrically, not just opponents
- **example:** City of Solitude: 'Players can cast spells and activate abilities only during their own turns'; Defense Grid: 'Each spell costs {3} more to cast except during its controller's turn'
- **suggested fix:** Stops or taxes players from casting spells or activating abilities outside their own turn.

### `recursion-from-exile` — suspect
- **current:** Returns a card you own from exile to your hand, library, battlefield, or graveyard.
- **issue:** 'a card you own' is overspecified; several cards move any face-up exiled card regardless of owner
- **example:** Pull from Eternity — 'Put target face-up exiled card into its owner's graveyard' (any player's exiled card, not just yours); Riftsweeper likewise targets any face-up exiled card
- **suggested fix:** Returns a card from exile to another zone such as a hand, library, battlefield, or graveyard.

### `hate-swamp` — suspect
- **current:** Punishes opponents for controlling Swamps or black permanents, or rewards you for it.
- **issue:** Overspecifies 'opponents'; several cards affect any player controlling Swamps/black
- **example:** Nature's Wrath: 'Whenever a player puts a Swamp or black permanent onto the battlefield, that player sacrifices...' and Spreading Algae enchants any Swamp
- **suggested fix:** Punishes controlling Swamps or black permanents, or rewards you for opponents' Swamps.

### `sports-name` — suspect
- **current:** A card whose name is a sports term or phrase, with no shared mechanical theme.
- **issue:** Claims 'no shared mechanical theme' but a majority of the tagged cards share the Assist mechanic
- **example:** Play of the Game / Out of Bounds / Huddle Up / Game Plan / The Crowd Goes Wild all have 'Assist (Another player can pay up to {N} of this spell's cost.)'
- **suggested fix:** A card whose name is a sports term or phrase, grouped by name rather than by a single mechanic.

### `synergy-blood` — suspect
- **current:** Creates or cares about Blood tokens, which you can sacrifice to loot for a card.
- **issue:** Calls the Blood ability 'loot', but it discards then draws (rummaging), and it omits the mana and tap cost
- **example:** Bloodtithe Harvester's Blood token: '{1}, {T}, Discard a card, Sacrifice this token: Draw a card.'
- **suggested fix:** Creates or cares about Blood tokens, which you can sacrifice, discarding a card to draw one, to filter your hand.

### `synergy-hexproof` — suspect
- **current:** Rewards you when a creature you control has hexproof, among other keyword abilities.
- **issue:** Frames hexproof as a reward trigger, but these cards detect, grant, copy, or count a whole standard keyword list of which hexproof is just one member
- **example:** Odric, Blood-Cursed: 'create X Blood tokens, where X is the number of abilities from among flying, first strike, double strike, deathtouch, haste, hexproof, indestructible, lifelink, menace, reach, trample, and vigilance found among creatures you control.'
- **suggested fix:** Cares about the standard keyword suite, including hexproof, often granting, copying, or counting those keywords across your creatures.

### `copy-enchantment` — suspect
- **current:** Creates a token that is a copy of an enchantment.
- **issue:** Overspecified: says it always makes a token copy, but several cards enter/become a copy directly (no token created).
- **example:** Mirrormade: "You may have this enchantment enter as a copy of any artifact or enchantment on the battlefield." (also Estrid's Invocation, Mirage Mirror)
- **suggested fix:** Copies an enchantment, often as a token.

### `dehydration-with-set-mechanic` — suspect
- **current:** An aura that taps the enchanted creature and keeps it from untapping.
- **issue:** Says it taps on entry, but some cards only prevent untapping (no ETB tap) or tap conditionally.
- **example:** Watery Grasp: "Enchanted creature doesn't untap during its controller's untap step." (no tap on enter; also Plumes of Peace)
- **suggested fix:** An aura that keeps the enchanted creature from untapping, often tapping it as it enters.

### `gives-castable-from-library` — suspect
- **current:** Lets you cast a spell straight from your library without paying its mana cost.
- **issue:** Says 'your library' but several cards cast/play from an opponent's library, and not all are free-cast
- **example:** Knowledge Exploitation: 'Search target opponent's library for an instant or sorcery card. You may cast that card without paying its mana cost.' (Xanathar lets you play from an opponent's library while paying costs.)
- **suggested fix:** Lets you cast a spell straight from a library, usually your own and usually without paying its mana cost.

### `hate-tutor` — suspect
- **current:** Rewards you or hinders an opponent whenever they search their library.
- **issue:** Says 'hinders an opponent' but several cards restrict or punish ALL players' searching, not just opponents'
- **example:** Mindlock Orb: 'Players can't search libraries.' (The Pleasant Taxer makes searching libraries cost {1} more for every player.)
- **suggested fix:** Punishes or restricts searching a library, or rewards you when a player searches theirs.

### `hatebird` — suspect
- **current:** A flying creature around 3 mana that disrupts opponents like a hatebear.
- **issue:** 'around 3 mana' is shaky and several effects are symmetric, not opponent-only
- **example:** Hushbringer: 'Creatures entering or dying don't cause abilities to trigger' (hits all players, and it costs less than 3); Linvala is a 4-drop
- **suggested fix:** A small flying creature that disrupts the game like a hatebear, taxing or shutting down plays.

### `land-or-hand` — suspect
- **current:** Reveals your top card, putting it onto the battlefield if it is a land or into your hand otherwise.
- **issue:** overspecified: several put any permanent (not just land) and some draw instead of going to hand
- **example:** Matter Reshaper: 'put that card onto the battlefield if it's a permanent card with mana value 3 or less. Otherwise, put that card into your hand'; Thrasios: 'if it's a land card... Otherwise, draw a card'
- **suggested fix:** Reveals your top card, putting it onto the battlefield if it's a land or permanent and otherwise into your hand.

### `repeatable-powerstones` — suspect
- **current:** Creates Powerstone tokens again and again for mana that can only pay for artifacts.
- **issue:** Powerstone mana restriction is 'can't cast a nonartifact spell', not 'can only pay for artifacts' (it can pay for any ability)
- **example:** Karn, Living Legacy: Powerstone token 'Add {C}. This mana can't be spent to cast a nonartifact spell'
- **suggested fix:** Repeatedly creates Powerstone tokens, artifacts that tap for mana that can't be spent to cast nonartifact spells.

### `set-matters` — suspect
- **current:** Cares about which real-world Magic set or sets a card was printed in.
- **issue:** Underspecified: several cards care about release year, number of printings, or Universe, not just 'which set'
- **example:** Gen, Confider of Ages: 'gets +1/+1 for each different year of release among other nontoken permanents you control'; Lona cares about cards 'printed in at least five different English language Magic releases'; Byode cares about 'Universe'
- **suggested fix:** Cares about a card's real-world printing provenance, like which set or Universe it came from, its release year, or how many times it's been printed.

### `tutor-copy` — suspect
- **current:** Searches your library for a card with the same name as another creature or permanent.
- **issue:** says 'your library' but some search an opponent's library, and the named source isn't always a creature/permanent
- **example:** Dichotomancy: 'search that player's library for a card with the same name as that permanent'; Infernal Tutor matches a card revealed from hand
- **suggested fix:** Searches a library for a card with the same name as a chosen permanent or card.

### `catch-22` — suspect
- **current:** Punishes each player at their end step unless they meet a condition, like tapping out their lands.
- **issue:** Only describes the end-step-damage subset; misses the large day/night bidirectional-trigger group
- **example:** Firmament Sage: 'Whenever day becomes night or night becomes day, draw a card' (no end-step punishment at all)
- **suggested fix:** Creates a no-win situation where each player is affected no matter which choice they make, like taking damage for leaving lands untapped or a payoff that triggers whichever way day and night shifts.

### `copy-planeswalker` — suspect
- **current:** Creates a token that's a copy of a permanent, which can include a planeswalker.
- **issue:** Says 'creates a token,' but several tagged cards copy without making a token
- **example:** Spark Double: 'You may have this creature enter as a copy of a creature or planeswalker you control' (it becomes the copy itself, no token); Oko becomes a copy too
- **suggested fix:** Copies a permanent, often as a token, and the copy can be a planeswalker.

### `flicker-artifact` — suspect
- **current:** Exiles an artifact or creature and returns it to the battlefield later.
- **issue:** 'later' is imprecise; many return in the same resolution rather than at a delayed time
- **example:** Spaceshift: 'Exile target artifact or creature, then return that card to the battlefield... with a +1/+1 counter' (immediate); Mighty Thor and Scrollshift return immediately too
- **suggested fix:** Exiles an artifact or creature and returns it to the battlefield, either right away or at a set later time.

### `gifts-ungiven` — suspect
- **current:** Searches your library for several cards and lets an opponent choose which ones you keep.
- **issue:** "Searches your library" isn't universal; several cards reveal the top of your library instead
- **example:** Memories Returning: "Reveal the top five cards of your library... choose an opponent. They put one on the bottom..." and Manifold Insights reveals the top ten (no search)
- **suggested fix:** Offers an opponent several of your cards and lets them split the pile, deciding which ones you keep.

### `hate-landwalk` — suspect
- **current:** Lets creatures with a landwalk ability be blocked as though they didn't have it.
- **issue:** Underspecified: some cards remove landwalk outright rather than only allowing it to be blocked through
- **example:** Mystic Decree: "All creatures lose flying and islandwalk"; Scarwood Hag: "{T}: Target creature loses forestwalk until end of turn."
- **suggested fix:** Answers landwalk evasion, letting such creatures be blocked as though they had no landwalk or stripping the ability entirely.

### `opaline-effect` — suspect
- **current:** Lets you draw a card when an opponent's spell or ability targets your creature.
- **issue:** too narrow: many trigger off you or any permanent, not just your creature
- **example:** Reparations: "Whenever an opponent casts a spell that targets you or a creature you control, you may draw a card."; Rayne, Academy Chancellor: "Whenever you or a permanent you control becomes the target..."
- **suggested fix:** Lets you draw a card when a spell or ability an opponent controls targets you or a permanent you control.

### `tutor-land-basic-island` — suspect
- **current:** Searches your library for a basic Island card and puts it onto the battlefield or into your hand.
- **issue:** Overspecified: nearly all these cards fetch one of several basic land types by choice, Island is just one option, not a dedicated Island tutor
- **example:** Bant Panorama: 'Search your library for a basic Forest, Plains, or Island card, put it onto the battlefield tapped'
- **suggested fix:** Searches your library for a basic land card, an Island being one valid choice, and puts it onto the battlefield or into your hand.

### `tutor-land-basic-swamp` — suspect
- **current:** Searches your library for a basic Swamp card and puts it onto the battlefield or into your hand.
- **issue:** Overspecified: most of these fetch a choice of basic land types, Swamp being one option, not a dedicated Swamp tutor
- **example:** Deceptive Landscape: 'Search your library for a basic Plains, Swamp, or Forest card, put it onto the battlefield tapped'
- **suggested fix:** Searches your library for a basic land card, a Swamp being one valid choice, and puts it onto the battlefield or into your hand.

### `regrowth-planeswalker` — suspect
- **current:** Returns a creature or planeswalker card from your graveyard to your hand.
- **issue:** 'creature or planeswalker' is wrong for several cards that return only noncreature cards; the tag's unifying trait is planeswalker, not creature
- **example:** Warden of the Eye: 'return target noncreature, nonland card from your graveyard to your hand' (cannot return a creature); Monastery Loremaster is the same
- **suggested fix:** Returns a planeswalker or other nonland permanent card from your graveyard to your hand.

### `removal-noncreature` — suspect
- **current:** Destroys or exiles a noncreature permanent.
- **issue:** 'destroys or exiles' misses bounce/tuck removal that the tag also covers
- **example:** Primal Command: 'Put target noncreature permanent on top of its owner's library' (neither destroy nor exile)
- **suggested fix:** Destroys, exiles, or otherwise removes a noncreature permanent.

### `token-without-a-card` — suspect
- **current:** A token type with no card that actually creates it, likely due to errata.
- **issue:** 'likely due to errata' is a speculative and mostly-wrong reason; these are usually just tokens no printed card happens to make
- **example:** Soldier / Goblin / Myr token creatures exist as printed token cards but no card in the tag set generates that exact token; none of this stems from errata
- **suggested fix:** A token that exists as a printed card but that no other card actually creates.

### `tutor-enchantment` — suspect
- **current:** Searches your library for an enchantment card and puts it into your hand or on top.
- **issue:** omits the onto-the-battlefield outcome; several cards put the enchantment directly onto the battlefield, not just hand or top
- **example:** Lost Auramancers: 'search your library for an enchantment card, put it onto the battlefield'; Incoming! also puts them onto the battlefield
- **suggested fix:** Searches your library for an enchantment card and puts it into your hand, on top, or onto the battlefield.

### `command` — suspect
- **current:** A modal spell where you choose two of several listed effects to apply.
- **issue:** Says 'modal spell' but many carriers are creatures/permanents with a choose-two ability, not spells; also some read 'choose up to two'
- **example:** Titan of Industry (Creature): 'When this creature enters, choose two —'; Call Damage Control: 'Choose up to two.'
- **suggested fix:** Chooses two effects from a listed set of options to apply.

### `explore-like` — suspect
- **current:** Lets you look at the top card of a library and keep it or bin it, similar to explore.
- **issue:** "bin it" implies graveyard, but several cards dispose to bottom/top of library instead
- **example:** Ajani, Sleeper Agent: "Otherwise, you may put it on the bottom of your library"; S.N.E.A.K. Dispatcher puts it "on your choice of the top or bottom of its owner's library"
- **suggested fix:** Lets you look at the top card of a library and, if it meets a condition, put it into your hand, otherwise stash it away, similar to explore.

### `grim-return` — suspect
- **current:** Returns permanents from your graveyard to the battlefield if they died or left play that turn.
- **issue:** says "your graveyard" but some hit all graveyards / each player's, and several only return creatures
- **example:** Thrilling Encore: "Put onto the battlefield under your control all creature cards in all graveyards that were put there from the battlefield this turn."
- **suggested fix:** Returns cards that were put into a graveyard from the battlefield this turn back onto the battlefield.

### `hate-empty-hand` — suspect
- **current:** Rewards you when an opponent has few or no cards in hand.
- **issue:** narrows to "opponent" but many cards care about any player's empty hand, and effect isn't always a reward
- **example:** Asylum Visitor: "at the beginning of each player's upkeep, if that player has no cards in hand, you draw a card"; Lupine Prototype: "can't attack or block unless a player has no cards in hand"
- **suggested fix:** Rewards you or grows stronger when a player has few or no cards in hand.

### `mana-gorger` — suspect
- **current:** Grows with +1/+1 counters whenever a player casts a spell.
- **issue:** Overgeneralizes the trigger: most cards restrict it to an opponent's spell or a specific color/type, not 'a player casts a spell'
- **example:** Mold Adder: 'Whenever an opponent casts a blue or black spell, you may put a +1/+1 counter on this creature.' (also Titania's Chosen: green spell; Medusa: noncreature spell)
- **suggested fix:** Puts +1/+1 counters on itself whenever a qualifying spell is cast, the trigger varying by player, color, or type.

### `typal-minotaur` — suspect
- **current:** Rewards or boosts your Minotaurs together as a shared tribe.
- **issue:** "together as a shared tribe" is meaningless for a single tribe (copied from the goblin-orc dual-tribe wording)
- **example:** Rageblood Shaman: "Other Minotaur creatures you control get +1/+1 and have trample."
- **suggested fix:** Rewards, boosts, or tutors for your Minotaurs.

### `typal-rebel` — suspect
- **current:** A creature that searches your library for a Rebel and puts it onto the battlefield.
- **issue:** overspecified: claims every card is a creature that tutors a Rebel to the battlefield, but some members are non-tutoring reward cards
- **example:** Barret, Avalanche Leader: "Whenever an Equipment you control enters, create a 2/2 red Rebel creature token." (no library search)
- **suggested fix:** Rewards or searches your library for creatures of the Rebel type.

### `vigor-effect` — suspect
- **current:** Converts damage dealt to a creature into +1/+1 counters.
- **issue:** overspecifies +1/+1 counters, but several members grant +0/+1 counters instead
- **example:** Sacred Boon: "put a +0/+1 counter on that creature for each 1 damage prevented this way."
- **suggested fix:** Prevents or replaces damage dealt to a creature, putting that many +1/+1 or +0/+1 counters on it instead.

### `burn-battle` — suspect
- **current:** Deals damage to a battle, alongside or instead of creatures and planeswalkers.
- **issue:** Lists only creatures and planeswalkers as companion targets, but many of these cards hit players/opponents instead
- **example:** Joyful Stormsculptor: 'deals 1 damage to each opponent and each battle they protect'; Jeska and Kamahl: '2 damage to target opponent, battle, or planeswalker'
- **suggested fix:** Deals damage to a battle, often as one of several targets it can hit such as creatures, planeswalkers, or players.

### `hate-monocolor` — suspect
- **current:** Punishes, resists, or destroys monocolored creatures and spells.
- **issue:** Says 'creatures and spells' but effects also hit any monocolored permanent
- **example:** Vanishing Verse: 'Exile target monocolored permanent.'
- **suggested fix:** Punishes, resists, destroys, or exiles monocolored permanents and spells.

### `pseudo-equipment` — suspect
- **current:** A creature or artifact that stays tapped to give another permanent a lasting bonus.
- **issue:** "another permanent" is singular and "bonus" isn't always a bonus; some hit all creatures or shrink the target
- **example:** Thran Weaponry: "{2}, {T}: All creatures get +2/+2 for as long as this artifact remains tapped"; Ashnod's Battle Gear gives +2/-2
- **suggested fix:** A creature or artifact that stays tapped to give one or more creatures a stat change for as long as it remains tapped.

### `relentless` — suspect
- **current:** Lets you run more than the normal four copies of this card in your deck.
- **issue:** hardcodes "four copies", but the limit varies (up to seven/nine) and one card allows only two, so the number is wrong
- **example:** Mothers Yamazaki: "A Commander deck can include two of this card" (two, not more than four); Nazgul: "up to nine"
- **suggested fix:** Can be included in your deck in more copies than the normal deckbuilding limit allows.

### `bottom-of-library-matters` — suspect
- **current:** Cares about the bottom of a library, such as drawing from it or exiling cards there.
- **issue:** 'exiling cards there' reads as exiling cards TO the bottom, but the cards exile FROM the bottom
- **example:** Memory Test: 'exiles cards from the bottom of their library until they exile five nonland cards'; Lantern of Undersight: 'You draw cards from the bottom of your library'
- **suggested fix:** Cares about the bottom of a library, such as drawing or exiling cards from it.

### `coin-flips-matter` — suspect
- **current:** Cares about coin flips and rewards you for winning them.
- **issue:** Says 'rewards you for winning'; several trigger off ANY player winning, not just you
- **example:** Zndrsplt, Eye of Wisdom: 'Whenever a player wins a coin flip, draw a card.'
- **suggested fix:** Cares about coin flips and rewards winning them.

### `counter-preservation` — suspect
- **current:** Moves a creature's counters onto another permanent when it dies or leaves the battlefield.
- **issue:** Says 'a creature's counters'; source can be any permanent, not only creatures
- **example:** Resourceful Defense: 'Whenever a permanent you control leaves the battlefield, if it had counters on it, put those counters on target permanent you control.'
- **suggested fix:** Moves a permanent's counters onto another permanent when it dies or leaves the battlefield.

### `cycle-zodiac-creature` — suspect
- **current:** One of a cycle of animal creatures, each with landwalk tied to its type.
- **issue:** 'landwalk tied to its type' is misleading; landwalk isn't derived from the creature type (e.g. Goat and Dog differ in type but both Mountainwalk)
- **example:** Zodiac Goat (Goat): 'Mountainwalk' and Zodiac Dog (Dog): 'Mountainwalk'
- **suggested fix:** One of a cycle of animal creatures, each with a landwalk ability.

### `fulfilled-futureshift` — suspect
- **current:** A card first previewed as futureshifted that later got a proper printing in its own set.
- **issue:** 'in its own set' is imprecise/misleading; fulfilled futureshifts got normal printings in later real sets, not their own set
- **example:** Steamflogger Boss (Future Sight futureshifted card, later given a normal printing in Unstable); Thornweald Archer (reprinted normally after its Future Sight debut)
- **suggested fix:** A Future Sight futureshifted preview card that later received a normal printing in a real set.

### `hate-free-spell` — suspect
- **current:** Punishes spells cast without spending any mana, usually by countering or damaging their caster.
- **issue:** Underspecified: scope is narrower than the cards, which also punish reduced-mana casts, no-colored-mana casts, and creatures cheated into play without being cast
- **example:** Void Mirror: 'if no colored mana was spent to cast it, counter that spell'; Tokka & Rahzar: 'if the amount of mana spent was less than its mana value...deal 3 damage'; Containment Priest: 'If a nontoken creature would enter and it wasn't cast, exile it instead'
- **suggested fix:** Punishes spells cast for free or below full cost, or creatures cheated onto the battlefield without being cast, by countering, exiling, or dealing damage.

### `removes-indestructible` — suspect
- **current:** Strips indestructible from a permanent, usually so damage can finish it off.
- **issue:** "usually so damage can finish it off" overspecifies; half the examples strip indestructible to enable destroy/wrath effects, not damage
- **example:** Exterminatus: "Nonland permanents your opponents control lose indestructible until end of turn. Destroy all nonland permanents."
- **suggested fix:** Strips indestructible from one or more permanents, usually so damage or a destroy effect can then finish them off.

### `un-color` — suspect
- **current:** A joke card that uses fake colors like pink, gold, or orange that don't exist in normal Magic.
- **issue:** Not all are joke cards; tournament-legal cards carry this tag too, so 'joke card' is misleading
- **example:** Sword of Dungeons & Dragons (a normal black-border card): 'create a 4/4 gold Dragon creature token' and Chicago Loop: 'a 2/2 orange Bear token' are not Un-cards
- **suggested fix:** Uses or creates objects in made-up colors like pink, orange, or gold that aren't among Magic's five real colors.

### `untapper-nonland` — suspect
- **current:** Untaps a nonland permanent, letting you use it again this turn.
- **issue:** 'you use it again' assumes the permanent is yours and benefits you, but many untap opponents' permanents or don't grant reuse
- **example:** Curse of Bounty: 'Each opponent attacking that player untaps all nonland permanents they control'; The Pandorica untaps a permanent then phases it out
- **suggested fix:** Untaps a nonland permanent.

### `conjure-card` — suspect
- **current:** Conjures a duplicate of a card into your hand or onto the battlefield.
- **issue:** says 'your hand' but the conjure often goes into another player's hand
- **example:** Gutmorn, Pactbound Servant: 'they choose another player. That player conjures a duplicate of that card into their hand'; Juggle the Performance: 'each player... conjures a duplicate'
- **suggested fix:** Conjures a duplicate of a card into a hand or onto the battlefield.

### `counter-fuel-mm` — suspect
- **current:** A creature that gets a benefit, like mana or removal, from removing -1/-1 counters off itself.
- **issue:** 'off itself' is too narrow; some remove a -1/-1 counter from another creature
- **example:** Woeleecher: 'Remove a -1/-1 counter from target creature. If you do, you gain 2 life'; Quillspike: 'Remove a -1/-1 counter from a creature you control'
- **suggested fix:** A creature that removes a -1/-1 counter for a benefit like mana or removal.

### `hate-typal-elf` — suspect
- **current:** A creature or effect that scales with the number of Elves on the battlefield.
- **issue:** 'scales with the number of Elves' misses the payoffs that trigger per Elf entering rather than counting Elves on the battlefield
- **example:** Wirewood Hivemaster: 'Whenever another nontoken Elf enters, you may create a 1/1 green Insect creature token' (rewards each entry, does not scale with the total count); Elvish Vanguard similarly gets a counter per Elf entering
- **suggested fix:** Rewards you for playing Elves, often scaling with the number of Elves on the battlefield or triggering as each Elf enters.

### `phyrexian-token` — suspect
- **current:** A creature token that also has a Phyrexian, oil-slick art variant.
- **issue:** Overspecified/unverifiable art claim; the cards are generic Phyrexian-flavored tokens, not an 'oil-slick art variant' function
- **example:** Insect token: 'Infect (This creature deals damage to creatures in the form of -1/-1 counters and to players in the form of poison counters.)'; also Myr / Wurm Token Artifact Creatures
- **suggested fix:** A creature token that is Phyrexian, usually created by a Phyrexian-themed card.

### `removes-hexproof` — suspect
- **current:** Strips hexproof from opposing creatures or players so you can target them.
- **issue:** Says 'creatures or players' but the effect hits any permanents your opponents control (planeswalkers, artifacts, etc.)
- **example:** Shadowspear: 'Permanents your opponents control lose hexproof and indestructible until end of turn.'
- **suggested fix:** Strips hexproof from permanents (and sometimes players) your opponents control so you can target them.

### `restock-instant` — suspect
- **current:** Puts an instant card from your graveyard back on top of your library.
- **issue:** Says 'on top' but several cards put the card on the bottom of the library
- **example:** Ardent Dustspeaker: 'you may put an instant or sorcery card from your graveyard on the bottom of your library.'
- **suggested fix:** Puts an instant card from your graveyard back into your library, usually on top.

### `restock-sorcery` — suspect
- **current:** Puts a sorcery card from your graveyard back on top of your library.
- **issue:** Says 'on top' but several cards put the card on the bottom of the library
- **example:** Keeper of the Cadence: 'Put target artifact, instant, or sorcery card from a graveyard on the bottom of its owner's library.'
- **suggested fix:** Puts a sorcery card from your graveyard back into your library, usually on top.

### `skip-untap-step` — suspect
- **current:** Makes a player skip their next untap step, so their permanents stay tapped.
- **issue:** Says 'a player' and 'next,' but many are ongoing skips that can hit you or every player, not just one player's next step
- **example:** Stasis: 'Players skip their untap steps.'
- **suggested fix:** Makes a player (sometimes you or everyone) skip an untap step, so their permanents stay tapped.

### `theft-planeswalker` — suspect
- **current:** Takes control of an opponent's planeswalker.
- **issue:** overspecified 'opponent's'; most cards target any planeswalker, not just an opponent's
- **example:** Mass Manipulation: 'Gain control of X target creatures and/or planeswalkers'; Dragonlord Silumgar: 'gain control of target creature or planeswalker'
- **suggested fix:** Takes control of a target planeswalker, often alongside creatures.

### `turn-control` — suspect
- **current:** Lets you control an opponent's turn, seeing their cards and making their decisions for them.
- **issue:** overspecified 'opponent's'; several carriers control any target player, not only an opponent
- **example:** Mindslaver: 'You control target player during that player's next turn'; Cruel Entertainment has two players control each other
- **suggested fix:** Lets you control a player's turn, seeing their cards and making their decisions for them.

### `clothing-matters` — suspect
- **current:** A joke card whose effect depends on clothing or accessories you're actually wearing.
- **issue:** Says 'you're actually wearing', but several cards key off any player's real-world clothing (often the opponent's), and some care about non-clothing physical traits
- **example:** Hurloon Wrangler: 'Denimwalk (This creature can't be blocked as long as defending player is wearing denim.)'; Blurry Beeble keys off the defending player wearing glasses; Avatar of Me uses your height, shoe size, and eye color
- **suggested fix:** A joke card whose effect depends on clothing, accessories, or physical traits a player really has in real life.

### `cycle-2xm-draft-signpost` — suspect
- **current:** A two-color draft signpost card that showcases an archetype's mechanics.
- **issue:** cards themselves aren't all two-color/gold; they're signposts for two-color archetypes
- **example:** Weapons Trainer (mono-white Equipment payoff) and Sphinx Summoner (artifact) are not two-color cards
- **suggested fix:** A Double Masters draft signpost card that points toward a two-color archetype.

### `cycle-a25-draft-signpost` — suspect
- **current:** An uncommon gold card meant to point drafters toward a two-color archetype.
- **issue:** 'gold' is inaccurate; some signposts are mono-color, not multicolored
- **example:** Quicksilver Dagger: mono-blue 'Enchantment - Aura', not a gold card
- **suggested fix:** An uncommon draft signpost card pointing drafters toward a two-color archetype.

### `cycle-akh-draft-signpost` — suspect
- **current:** An uncommon multicolor creature built to showcase a two-color archetype in this set's draft format.
- **issue:** Says 'multicolor creature' but several cards in the cycle are mono-colored
- **example:** Weaver of Currents ('{T}: Add {C}{C}') is a mono-green creature; Ahn-Crop Champion is mono-white, so the 'multicolor' qualifier is wrong for part of the cycle
- **suggested fix:** An uncommon creature that showcases one of Amonkhet's two-color draft archetypes.

### `cycle-ala-u-two-color` — suspect
- **current:** A two-color uncommon from the Shards of Alara block.
- **issue:** Says 'block' but this cycle is the Shards of Alara set specifically, not the three-set block
- **example:** Tidehollow Sculler and Bull Cerodon are Shards of Alara set cards; the tag is set-scoped, so 'block' overstates the scope
- **suggested fix:** A two-color uncommon from the Shards of Alara set.

### `cycle-bbd-legendary-partner` — suspect
- **current:** A legendary creature with partner with that fetches its named partner to a player's hand when it enters.
- **issue:** Clumsy 'with partner with' phrasing and drops the optional 'may put' choice
- **example:** Regna, the Redeemer: 'Partner with Krav... target player may put Krav into their hand from their library, then shuffle.'
- **suggested fix:** A legendary creature with partner: when it enters, a player may tutor its named partner into their hand.

### `cycle-bfz-draft-signpost` — suspect
- **current:** A creature built to point drafters toward one of Battle for Zendikar's color-pair archetypes.
- **issue:** Says 'creature' but the cycle includes a noncreature spell
- **example:** Roil Spout is type 'Sorcery' (Put target creature on top of its owner's library. Awaken 4...), not a creature.
- **suggested fix:** A card built to point drafters toward one of Battle for Zendikar's two-color draft archetypes.

### `cycle-block-rtr-m-multicolor` — suspect
- **current:** A multicolor mythic card from the Return to Ravnica block's guild cycle.
- **issue:** "guild cycle" is misleading; these are the block's multicolor mythics, not a 10-guild cycle
- **example:** Domri Rade (Gruul planeswalker) and Sphinx's Revelation (Azorius instant) are unrelated mythics, not one-per-guild cycle members
- **suggested fix:** A multicolored mythic rare from the Return to Ravnica block.

### `cycle-block-rtr-u-hybrid` — suspect
- **current:** An uncommon that signals a two-color archetype in the Return to Ravnica block.
- **issue:** Not all uncommon, and these are hybrid guild cards, not archetype signposts
- **example:** Burning-Tree Emissary (rarity C, {R/G}{R/G}) and Merfolk of the Depths (rarity C) are common, not uncommon; cards are castable with either color rather than 'signaling an archetype'
- **suggested fix:** A two-color hybrid card from the Return to Ravnica block, one per guild, castable with either of its two colors.

### `cycle-clb-draft-signpost` — suspect
- **current:** One of Baldur's Gate's two-color legendary creatures built to support its guild's draft archetype.
- **issue:** 'guild' is Ravnica-specific jargon; Baldur's Gate has no guilds
- **example:** Mahadi, Emporium Master ({1}{B}{R}, uncommon) is a color-pair signpost, not a guild card
- **suggested fix:** An uncommon two-color legendary creature from Baldur's Gate that points drafters toward a color-pair archetype.

### `cycle-cmr-backward-partner` — suspect
- **current:** The backward half of a Commander Legends partner-legend pair, meant to team up with its forward twin.
- **issue:** 'forward twin' implies a fixed pairing, but plain Partner pairs with any Partner card
- **example:** Brinelin, the Moon Kraken: 'Partner (You can have two commanders if both have partner.)'
- **suggested fix:** A Commander Legends legendary creature with Partner, letting you pair it with any other Partner card as a second commander.

### `cycle-cn2-draft-signpost` — suspect
- **current:** A two-color creature that signals its color-pair draft archetype in Conspiracy.
- **issue:** says 'creature' but the cycle includes non-creature signposts
- **example:** Gruul War Chant is type Enchantment ('Attacking creatures you control get +1/+0 and have menace.'), not a creature
- **suggested fix:** A signpost card that points drafters toward a two-color archetype in Conspiracy: Take the Crown.

### `cycle-cns-r-two-color` — suspect
- **current:** A rare creature costed in two colors.
- **issue:** says 'creature' but at least one member is a spell
- **example:** Decimate is type Sorcery ('Destroy target artifact, target creature, target enchantment, and target land.'), not a creature
- **suggested fix:** A rare, two-color card from Conspiracy.

### `cycle-dgm-m-two-color` — suspect
- **current:** A mythic two-color gold card representing one of the set's ten guilds.
- **issue:** Claims 'mythic' but the cycle includes rares, not just mythics
- **example:** Blood Baron of Vizkopa is rarity R (rare), as is Legion's Initiative; both carry the tag
- **suggested fix:** A two-color gold card representing one of Dragon's Maze's ten guilds.

### `cycle-dual-investigate-tapland` — suspect
- **current:** A tapped dual land that can pay mana and tap to investigate for a Clue.
- **issue:** 'can pay mana and tap to investigate' reads as one combined action; the two abilities each require the single tap and can't both be used at once
- **example:** Dining Room: 'This land enters tapped. {T}: Add {R} or {G}. {4}, {T}: Investigate.'
- **suggested fix:** A dual land that enters tapped and can either tap for one of two colors or pay to investigate for a Clue.

### `cycle-ema-r-two-color` — suspect
- **current:** A two-color gold rare from Eternal Masters.
- **issue:** Calls the cycle 'gold' but several members are hybrid, not true gold multicolor
- **example:** Giant Solifuge: '({R/G} can be paid with either {R} or {G})'; Deathrite Shaman is {B/G} hybrid
- **suggested fix:** A two-color rare from Eternal Masters.

### `cycle-eoe-u-spacecraft` — suspect
- **current:** An uncommon Spacecraft artifact with an enters effect that becomes a creature once stationed.
- **issue:** 'with an enters effect' is not universal, and 'once stationed' understates the power threshold
- **example:** Galvanizing Sawship has no enters trigger, only 'Station ... It's an artifact creature at 3+. / 3+ | Flying, haste'
- **suggested fix:** An uncommon Spacecraft artifact that becomes a flying creature once enough power is stationed onto it.

### `cycle-gtc-m-two-color` — suspect
- **current:** A Gatecrash two-color mythic rare, one themed to each of the set's guilds.
- **issue:** 'one themed to each of the set's guilds' implies a five-card one-per-guild cycle, but the tag holds 10 mythics (multiple per guild)
- **example:** Both Aurelia, the Warleader and Master Biomancer are Boros/Simic-adjacent GTC mythics, so guilds get more than one each
- **suggested fix:** A Gatecrash two-color mythic rare tied to one of the set's guilds.

### `cycle-inr-draft-signpost` — suspect
- **current:** A two-color draft signpost card that showcases an archetype's mechanics.
- **issue:** Calls the card itself two-color, but many signposts are monocolored
- **example:** Markov Waltzer is mono-red ('Flying, haste ... target creatures you control each get +1/+0'); Wandering Mind and Spectral Shepherd are mono-blue
- **suggested fix:** A draft signpost card that points drafters toward a two-color archetype's mechanics.

### `cycle-ktk-draft-signpost` — suspect
- **current:** A card that signals which two-color archetype to draft in this set.
- **issue:** KTK archetypes are three-color clan wedges, not two-color; the card is two-color but points to a clan
- **example:** Ride Down (RW) points to the Mardu clan (RWB); Death Frenzy (BG) toward Sultai/Abzan; these signal three-color wedge clans
- **suggested fix:** A gold card that signals which three-color clan archetype to draft in this set.

### `cycle-lci-r-two-color` — suspect
- **current:** A rare two-color card from the Lost Caverns of Ixalan gold cycle.
- **issue:** Says every card is two-color/gold, but a mono-color member carries the tag
- **example:** Kellan, Daring Traveler // Journey On is mono-white (colors ["W"]), not a gold two-color card
- **suggested fix:** A rare from Lost Caverns of Ixalan, usually a two-color gold card, tied to a draft archetype.

### `cycle-ogw-draft-signpost` — suspect
- **current:** A two-color uncommon built to point drafters toward its color pair's archetype.
- **issue:** Overspecifies rarity and color: one member is rare, and two devoid Eldrazi are colorless, not two-color
- **example:** Reflector Mage is rarity R (not uncommon); Void Grafter and Flayer Drone have Devoid, so colors=[] (colorless) despite two-colored pips
- **suggested fix:** A signpost card that points drafters toward its two-color draft archetype.

### `cycle-one-draft-signpost` — suspect
- **current:** A two-color signpost creature marking a draft archetype in Phyrexia: All Will Be One.
- **issue:** Says 'creature' but one member is an Equipment, not a creature
- **example:** Bladehold War-Whip is an 'Artifact — Equipment' (For Mirrodin!), not a creature
- **suggested fix:** A two-color signpost card marking a draft archetype in Phyrexia: All Will Be One.

### `cycle-rav-backward-boost` — suspect
- **current:** Grants a bonus effect only if a specific colored mana was spent to cast it.
- **issue:** Frames the mana-spent condition as only a bonus; several cards impose a sacrifice PENALTY unless the mana was spent, not a bonus
- **example:** Court Hussar: 'sacrifice it unless {W} was spent to cast it'; Plaxmanta and Squealing Devil are the same (sac unless mana spent), which is a drawback, not a granted bonus
- **suggested fix:** Has an extra effect keyed to whether a specific color of mana was spent to cast it, granting a bonus or forcing a sacrifice if it wasn't.

### `cycle-rav-forward-boost` — suspect
- **current:** Gains a bonus effect if a specific color of mana was spent to cast it.
- **issue:** Says 'gains a bonus effect,' but half the cycle instead sacrifices itself unless a color was spent (a requirement, not a bonus)
- **example:** Azorius Herald: 'When this creature enters, sacrifice it unless {U} was spent to cast it.'
- **suggested fix:** Has an extra effect or avoids sacrificing itself depending on whether a specific color of mana was spent to cast it.

### `cycle-rtr-m-two-color` — suspect
- **current:** One of a cycle of mythic two-color cards, one for each of the set's guilds.
- **issue:** cycle has 10 cards (~2 mythic gold per guild), so "one for each of the set's guilds" undercounts and misdescribes the structure
- **example:** Epic Experiment and Niv-Mizzet, Dracogenius are both Izzet; Vraska the Unseen and Jarad, Golgari Lich Lord are both Golgari
- **suggested fix:** One of a cycle of mythic two-color cards from Return to Ravnica, spanning the set's five guilds.

### `cycle-spm-r-two-color` — suspect
- **current:** A rare two-color legendary creature built around its set's draft archetypes.
- **issue:** Overspecifies as 'legendary creature'; the cycle includes a non-creature Equipment
- **example:** Biorganic Carapace is 'Artifact — Equipment' (When this Equipment enters, attach it to target creature you control), not a legendary creature.
- **suggested fix:** A rare two-color card built around its set's draft archetypes.

### `cycle-vow-draft-signpost` — suspect
- **current:** An uncommon multicolor creature built to showcase a two-color archetype in this set's draft format.
- **issue:** Not all members are strictly multicolor creatures; the cycle includes mono-color-faced double-faced cards, one whose back is an Aura
- **example:** Brine Comber // Brinebound Gift is a Spirit front face with an Enchantment - Aura back face (not a creature), and its faces are mono-colored rather than gold
- **suggested fix:** An uncommon card that signposts one of this set's two-color draft archetypes.

### `hate-graveyard-cast` — suspect
- **current:** Stops players from casting spells out of graveyards.
- **issue:** Only says 'stops'; the hate- tag also includes cards that counter or punish graveyard casting, not just prevent it
- **example:** Ash Zealot: 'Whenever a player casts a spell from a graveyard, this creature deals 3 damage to that player.' Laquatus's Disdain: 'Counter target spell cast from a graveyard.'
- **suggested fix:** Stops, counters, or punishes players who cast spells from graveyards.

### `impulse-artifact-equipment` — suspect
- **current:** Digs through the top cards of your library for an Equipment or Vehicle card and puts it into your hand.
- **issue:** Says it goes 'into your hand,' but several cards put the Equipment or Vehicle onto the battlefield instead
- **example:** Nick Fury, Agent of S.H.I.E.L.D.: 'look at the top seven cards... You may put a Hero, Equipment, or Vehicle card from among them onto the battlefield.' Armored Skyhunter puts an Equipment onto the battlefield too.
- **suggested fix:** Digs through the top cards of your library for an Equipment or Vehicle card and puts it into your hand or onto the battlefield.

### `morphling` — suspect
- **current:** A shapeshifter with mana abilities that let it change its stats or gain keywords at instant speed.
- **issue:** Not all are shapeshifters and not all pay mana; some are other creature types and one pays energy, and 'mana abilities' has a misleading rules meaning
- **example:** Multiform Wonder (Artifact Creature - Construct): 'Pay {E}: This creature gains your choice of flying, vigilance, or lifelink until end of turn.' Shorecrasher Elemental is an Elemental, not a Shapeshifter.
- **suggested fix:** A creature with repeatable activated abilities that pump its stats or grant it keywords at instant speed.

### `moxen` — suspect
- **current:** A zero-cost artifact named Mox that taps for mana.
- **issue:** "named Mox" over-specifies; several tagged cards aren't named Mox
- **example:** Gleemox: "{T}: Add one mana of any color. This card is banned." (not named Mox; Jack-in-the-Mox also isn't a plain "Mox")
- **suggested fix:** A free or near-free artifact that taps to add mana, in the style of the classic Moxen.

### `synergy-colored` — suspect
- **current:** Cares about the colors or colored mana symbols on cards you cast, reveal, or mill.
- **issue:** Sources overspecified; also cares about colors of permanents you control or creatures you sacrifice, not just cast/reveal/mill
- **example:** Meteor Crater: "Choose a color of a permanent you control. Add one mana of that color."
- **suggested fix:** Cares about the colors or colored mana symbols of cards and permanents.

### `synergy-flash` — suspect
- **current:** Rewards you for casting spells or creatures with flash.
- **issue:** Too narrow: many cards reward creatures/permanents that already have flash, not just casting them
- **example:** Sonic the Hedgehog: "put a +1/+1 counter on each creature you control with flash or haste"
- **suggested fix:** Cares about the flash keyword, rewarding spells and creatures you control that have flash.

### `synergy-untapped` — suspect
- **current:** Rewards you for having untapped permanents, often boosting their stats or granting protection.
- **issue:** Says "permanents" but every card references untapped creatures you control, not lands/artifacts
- **example:** Builder's Blessing: "Untapped creatures you control get +0/+2."
- **suggested fix:** Rewards untapped creatures you control, often boosting their toughness or granting protection.

### `unique-mana-symbol` — suspect
- **current:** Uses a mana symbol that appears on no other card, like infinity or one hundred.
- **issue:** 'appears on no other card' overstates it; several tagged cards use symbols shared by many cards (Phyrexian hybrid, colorless {C})
- **example:** Ulalek, Fused Atrocity: 'you may pay {C}{C}' ({C} is common), and Tamiyo, Compleated Sage uses Phyrexian {G/U/P} found on many cards
- **suggested fix:** Uses an unusual or nonstandard mana symbol, such as infinity, one hundred, or Phyrexian mana.

### `buff-pact` — suspect
- **current:** Taps to boost a creature, but sacrifices itself if that creature leaves the battlefield that turn.
- **issue:** Overspecifies 'taps' and 'boost'; some grant an ability not a stat buff, and one card's pact runs the other way
- **example:** War Barge: '{3}: Target creature gains islandwalk... When this artifact leaves the battlefield this turn, destroy that creature' (no tap, no stat boost, and it destroys the creature rather than sacrificing itself)
- **suggested fix:** Pumps or empowers a target creature, then is sacrificed if that creature leaves the battlefield that turn.

### `chain-spell` — suspect
- **current:** An instant or sorcery that its target's controller may copy and retarget by paying a cost.
- **issue:** 'by paying a cost' is overspecified; the copy can be free or gated by a discard or sacrifice, not just a paid cost
- **example:** Chain of Acid: 'Destroy target noncreature permanent. Then that permanent's controller may copy this spell and may choose a new target for that copy' (no cost to copy)
- **suggested fix:** An instant or sorcery whose target's controller may copy it and pick a new target, sometimes by paying a cost or discarding or sacrificing.

### `dolmen-ability` — suspect
- **current:** Prevents all combat damage that would be dealt to your attacking creatures.
- **issue:** Overspecified to 'your attacking creatures'; several cards protect all your creatures, all combat damage, or even all damage
- **example:** Winds of Qal Sisma: 'Prevent all combat damage that would be dealt this turn' (general fog, not just your attackers); Loyal Unicorn: 'prevent all combat damage that would be dealt to creatures you control' (all your creatures, not only attackers); Iroas: 'Prevent all damage that would be dealt to attacking creatures you control' (all damage, not just combat)
- **suggested fix:** Prevents combat damage that would be dealt to attacking creatures, usually the ones you control, and sometimes to all your creatures or all combat damage.

### `end-turn` — suspect
- **current:** Ends the current turn early, exiling spells on the stack and skipping remaining phases.
- **issue:** Exiles all spells AND abilities from the stack, not just spells
- **example:** Time Stop: 'End the turn. (Exile all spells and abilities, including this spell...)' and Obeka: 'Exile all spells and abilities from the stack'
- **suggested fix:** Ends the current turn immediately, exiling all spells and abilities on the stack and skipping the rest of the turn.

### `form` — suspect
- **current:** An enchantment or planeswalker that transforms you, with big upsides and a real drawback.
- **issue:** 'transforms you' is loose flavor (Form cards change rules/your life, not your creature type) and not every card carries a real drawback
- **example:** Form of the Dragon: 'At the beginning of each end step, your life total becomes 5. Creatures without flying can't attack you.' (changes rules, doesn't turn you into a Dragon); Garruk the Slayer is a plain planeswalker with no personal drawback
- **suggested fix:** A 'Form of' enchantment or 'you become' planeswalker that changes what you are, usually pairing a big upside with a steep drawback.

### `loot-to-library` — suspect
- **current:** Draws you cards, then puts a card from your hand on the bottom of or shuffles it into your library.
- **issue:** Omits the 'top of library' option some cards offer, and 'draws' misses seek-based versions
- **example:** Dream Cache: 'Draw three cards, then put two cards from your hand both on top of your library or both on the bottom'; Seek New Knowledge seeks rather than draws
- **suggested fix:** Draws or seeks you cards, then returns one or more cards from your hand to the top or bottom of, or shuffled into, your library.

### `mutiny` — suspect
- **current:** Makes two creatures the same player controls fight or deal damage to each other.
- **issue:** Overspecifies 'same player controls' (some cards fight ANY two creatures) and 'each other' implies mutual, but damage versions are one-directional
- **example:** Clash of Titans: 'Target creature fights another target creature' (no same-controller restriction); Mutiny: one creature 'deals damage equal to its power to another' (only one deals)
- **suggested fix:** Makes one creature deal damage to another creature, or makes two creatures fight.

### `peek-face-down` — suspect
- **current:** Lets you look at face-down creatures you don't control.
- **issue:** Overspecified: 'you don't control' isn't universal; several cards look at ANY face-down creature
- **example:** Smoke Teller: '{1}{U}: Look at target face-down creature.' (no you-don't-control limit; also Revealing Wind looks at each attacking/blocking face-down creature)
- **suggested fix:** Lets you look at face-down creatures, often ones you don't control.

### `prevent-etb` — suspect
- **current:** Stops permanents from entering the battlefield, exiling or redirecting them instead.
- **issue:** 'exiling or redirecting them instead' isn't universal; many just stop entering with no replacement
- **example:** Grafdigger's Cage: 'Creature cards in graveyards and libraries can't enter the battlefield.' (no exile/redirect); Worms of the Earth: 'Lands can't enter the battlefield.'
- **suggested fix:** Stops certain permanents from entering the battlefield, sometimes exiling or bouncing them instead.

### `reanimate-face-down` — suspect
- **current:** Returns a creature card from a graveyard to the battlefield face down.
- **issue:** not always a creature card, and source isn't always the graveyard (some exile/mill)
- **example:** Magar of the Magic Strings: 'Note the name of target instant or sorcery card in your graveyard and put it onto the battlefield face down. It's a 3/3 creature'
- **suggested fix:** Puts a card onto the battlefield face down, usually a creature from a graveyard.

### `remove-counters-player` — suspect
- **current:** Strips all counters, often poison, from a player instead of a permanent.
- **issue:** 'all counters' overstates (some remove a few), and it's usually 'permanent or player' not 'instead of a permanent'
- **example:** Price of Betrayal: 'Remove up to five counters from target artifact, creature, planeswalker, or opponent.'
- **suggested fix:** Removes counters, often poison, from a player rather than only from a permanent.

### `synergy-color-every` — suspect
- **current:** Rewards you for having all five colors among your permanents at once.
- **issue:** underspecified: also rewards a single permanent, creature, or spell that is all colors, not just five colors among your permanents
- **example:** Iridian Maelstrom: 'Destroy each creature that isn't all colors.'
- **suggested fix:** Rewards you for having all five colors present, whether among your permanents or on a single all-colors permanent, creature, or spell.

### `tutor-cast` — suspect
- **current:** Searches your library for a card and lets you cast it without paying its mana cost.
- **issue:** Overspecified: some tagged cards let you cast the found card at full cost, not free
- **example:** Evolving Door: 'search your library for a creature card... Exile that card, then shuffle. You may cast the exiled card.' (no 'without paying its mana cost')
- **suggested fix:** Searches your library for a card and lets you cast or play it, often without paying its mana cost.

### `ante-matters` — suspect
- **current:** Uses the old ante mechanic, wagering cards from your library as part of its effect.
- **issue:** Overspecified: not all cards wager from your library; several exchange ownership of cards already in the ante or of permanents
- **example:** Darkpact: 'You own target card in the ante. Exchange that card with the top card of your library.' Bronze Tablet exchanges ownership of a nontoken permanent, not a library card.
- **suggested fix:** Uses the old ante mechanic, in which cards are wagered and can change ownership between players.

### `counterspell-sweeper` — suspect
- **current:** Counters all of your opponents' spells or abilities at once instead of just one.
- **issue:** 'your opponents'' is too narrow; several counter ALL other spells (any player's), not just opponents'
- **example:** Swift Silence: 'Counter all other spells. Draw a card for each spell countered this way.' (also Summary Dismissal 'Exile all other spells and counter all abilities')
- **suggested fix:** Counters multiple spells or abilities at once instead of a single target.

### `fake-flying` — suspect
- **current:** A creature without flying that can't be blocked except by creatures with flying or reach.
- **issue:** Overspecifies 'or reach'; many carriers are blockable only by flying (and some by Walls too), not reach
- **example:** Silhana Ledgewalker: 'This creature can't be blocked except by creatures with flying' (no reach); Elven Riders adds 'Walls and/or creatures with flying'
- **suggested fix:** A creature without flying that can only be blocked by creatures with flying, and sometimes reach.

### `hate-etb` — suspect
- **current:** Stops permanents that enter the battlefield from triggering abilities.
- **issue:** Says 'permanents' but most cards hit creatures only, and several tax/copy rather than stop the trigger
- **example:** Torpor Orb: 'Creatures entering don't cause abilities to trigger'; Strict Proctor counters the ability unless its controller pays {2} (does not stop it outright)
- **suggested fix:** Stops abilities from triggering when creatures enter the battlefield.

### `krarks-other-thumb-effect` — suspect
- **current:** Makes you roll extra dice and ignore the worst results whenever you roll.
- **issue:** 'ignore the worst results' overstates: cards ignore only ONE roll, often a chosen one, and some target another player
- **example:** Krark's Other Thumb: 'instead roll two of those dice and ignore one of those results'; Bamboozling Beeble makes a target player roll extra and 'you choose one of those rolls to ignore'
- **suggested fix:** Makes you roll one extra die and ignore one of the results whenever you roll dice.

### `perpetual-aura` — suspect
- **current:** An aura that returns to your hand when it falls off the creature it enchanted.
- **issue:** Not always a creature (some enchant lands), and it returns to owner's hand on going to graveyard, not strictly 'your hand'
- **example:** Spreading Algae: 'Enchant Swamp ... When this Aura is put into a graveyard from the battlefield, return it to its owner's hand.'
- **suggested fix:** An Aura that returns to its owner's hand when it's put into a graveyard from the battlefield instead of staying there.

### `restock-artifact` — suspect
- **current:** Puts an artifact card from a graveyard back on top of its owner's library to be drawn again.
- **issue:** says "on top" but several put the artifact on bottom or second from top
- **example:** Keeper of the Cadence: "Put target artifact, instant, or sorcery card from a graveyard on the bottom of its owner's library." (Richlau puts it second from the top)
- **suggested fix:** Returns an artifact card from a graveyard to its owner's library, usually on top, so it can be reused.

### `synergy-goad` — suspect
- **current:** Goads enemy creatures, forcing them to attack each combat and to attack a player other than you.
- **issue:** only describes the goad mechanic; misses that this synergy tag rewards you for goaded creatures, and includes cards that never goad
- **example:** Serene Sleuth: "At the beginning of combat on your turn, investigate for each goaded creature you control." (it pays off goad rather than goading anything)
- **suggested fix:** Cares about goaded creatures, both goading enemy creatures to attack a player other than you and rewarding you for the goaded creatures in play.

### `typal-non-share` — suspect
- **current:** Rewards you for casting creatures with a creature type you don't already have.
- **issue:** Says 'casting', but several members reward finding or counting new/diverse types, not only casting, and the zones checked vary (library, graveyard, battlefield).
- **example:** Radagast the Brown: 'reveal a creature card that doesn't share a creature type with a creature you control ... put it into your hand' (card selection, not casting); Menagerie Curator checks 'a creature card in your library.'
- **suggested fix:** Rewards you for creatures whose creature type isn't already shared by your other creatures or cards.

### `cycle-tr-mage` — suspect
- **current:** One of a cycle of wizards, each tutoring an artifact of a specific mana value to your hand.
- **issue:** 'specific mana value' overspecifies: several fetch ranges and one ignores mana value entirely
- **example:** Trove Mage: 'seek an artifact card from among the top ten cards of your library, then shuffle' (no mana-value restriction); Treasure Mage searches for 'mana value 6 or greater' (a range, not a specific value)
- **suggested fix:** One of a cycle of wizards, each fetching an artifact card into your hand when it enters.

### `delayed-payment` — suspect
- **current:** Play it now, then pay its mana cost at your next upkeep or lose the game.
- **issue:** Payment isn't the card's mana cost and isn't always at upkeep (super haste pays at end step)
- **example:** Slaughter Pact: cast for {0}, then 'At the beginning of your next upkeep, pay {2}{B}'; Rocket-Powered Turbo Slug (super haste) pays 'at the beginning of your next turn's end step'
- **suggested fix:** Lets the effect happen now, then requires you to pay a deferred cost on a later turn or you lose the game.

### `eternalize` — suspect
- **current:** Lets you exile a creature card from your graveyard to make a 4/4 black Zombie copy of it.
- **issue:** Overspecified: not always your graveyard, and some don't exile from a graveyard at all
- **example:** The Scarab God: 'Exile target creature card from a graveyard' (any); Hashaton: creates the 4/4 Zombie copy when you discard a creature card, no graveyard exile
- **suggested fix:** Creates a 4/4 black Zombie token that's a copy of a creature card, usually one exiled from a graveyard.

### `extra-upkeep` — suspect
- **current:** Gives you one or more additional upkeep steps.
- **issue:** Says 'you' but several give the steps to the enchanted player, who can be any player
- **example:** Paradox Haze: 'Enchant player ... that player gets an additional upkeep step'; Shadow of the Second Sun enchants a player too
- **suggested fix:** Adds one or more additional upkeep steps to a player's turn.

### `forestfall` — suspect
- **current:** Triggers a bonus effect whenever a Forest enters the battlefield under your control.
- **issue:** 'under your control' is overspecified; some trigger on any Forest entering
- **example:** Baru, Fist of Krosa: 'Whenever a Forest enters, green creatures you control get +1/+1 and gain trample'
- **suggested fix:** Triggers a bonus effect whenever a Forest enters the battlefield, usually one you control.

### `grafted-skullcap` — suspect
- **current:** Draws you an extra card each turn but forces you to discard your hand at end of turn.
- **issue:** Says 'draws you' but the discard can hit an enchanted opponent, and draw amount varies
- **example:** Curse of Obsession: 'enchanted player draws two additional cards ... that player discards their hand.'
- **suggested fix:** Draws extra cards each turn but forces the affected player to discard their hand at end of turn.

### `hate-commander` — suspect
- **current:** Destroys or bounces an opponent's commander.
- **issue:** Too narrow and 'opponent's' overspecified: several cards steal, restrict, or trigger off commanders and can hit any player
- **example:** Leadership Vacuum: 'Target player returns each commander they control ... to the command zone'; Spice Rack limits max Commander size.
- **suggested fix:** Answers or disrupts commanders, for example by destroying, bouncing, stealing, or restricting them.

### `hate-typal-dragon` — suspect
- **current:** Punishes, destroys, or blanks Dragon creatures, or seizes control of them.
- **issue:** "blanks" is unsupported by any card, and the description omits the dominant mode: granting protection from Dragons
- **example:** Dragonstalker: "Flying, protection from Dragons"; Dragon Hunter: "Protection from Dragons. This creature can block Dragons as though it had reach."
- **suggested fix:** Destroys Dragon creatures, seizes control of them, or gives protection from Dragons.

### `old-ward` — suspect
- **current:** Counters a spell or ability that targets it unless its controller pays a cost, an early form of ward.
- **issue:** Overspecified 'targets it' and omits the opponent-controlled qualifier; many protect you or any permanent you control, not just themselves
- **example:** Amulet of Safekeeping: 'Whenever you become the target of a spell or ability an opponent controls, counter that spell or ability unless its controller pays {1}.'
- **suggested fix:** Counters an opponent's spell or ability that targets you or something you control unless its controller pays a cost, an early form of ward.

### `opponent-sacrifices` — suspect
- **current:** Forces an opponent to sacrifice a permanent of their choice.
- **issue:** Says 'an opponent' and 'a permanent' when several hit each player (including you) and the sacrifice is usually a creature specifically
- **example:** Plaguecrafter: 'each player sacrifices a creature or planeswalker of their choice'; Woebringer Demon: 'that player sacrifices a creature of their choice'
- **suggested fix:** Forces one or more opponents (sometimes every player) to sacrifice a creature of their choice.

### `prevent-trigger` — suspect
- **current:** Stops permanents entering the battlefield from causing abilities to trigger.
- **issue:** Overbroad 'permanents entering' when most cards only stop creatures entering from triggering abilities
- **example:** Torpor Orb / Hushwing Gryff / Tocatli Honor Guard: 'Creatures entering don't cause abilities to trigger.'
- **suggested fix:** Stops creatures (and sometimes other permanents) entering the battlefield from causing abilities to trigger.

### `promotes-to-commander` — suspect
- **current:** Lets a creature that normally isn't a commander become one.
- **issue:** 'a creature' is overspecified; several cards promote noncreature permanents too
- **example:** Ormacar, Relic Wraith: 'Precious (You can have two commanders if the other one is a legendary noncreature artifact.)'; Barce lets any legend with one color identity be a commander
- **suggested fix:** Lets a card that normally can't be a commander become one.

### `specific-toughness-matters` — suspect
- **current:** Rewards you for having a creature or spell with a chosen power or toughness value.
- **issue:** "chosen" overspecifies: many cards use a fixed value, not a player-chosen one
- **example:** Duskana, the Rage Mother: "draw a card for each creature you control with base power and toughness 2/2" (fixed, not chosen); Sword of the Squeak: "+1/+1 for each creature you control with base power or toughness 1"
- **suggested fix:** Rewards you for having a creature or spell with a specific power or toughness value.

### `synergy-locus` — suspect
- **current:** Rewards you for controlling Locus lands like Cloudpost.
- **issue:** Locus is not only lands (Swarm of Locus is a creature), and several count Locus on the battlefield generally, not just ones you control
- **example:** Swarm of Locus (Creature — Insect Locus): "gets +1/+0 ... for each Locus you control"; Cloudpost: "Add {C} for each Locus on the battlefield"
- **suggested fix:** Rewards you for having Locus permanents like Cloudpost.

### `synergy-pw-chandra` — suspect
- **current:** Rewards you for controlling a Chandra planeswalker.
- **issue:** Says only 'controlling' but many reward activating a Chandra's loyalty ability or casting Chandra, not mere control
- **example:** Keral Keep Disciples: 'Whenever you activate a loyalty ability of a Chandra planeswalker, this creature deals 1 damage to each opponent.'
- **suggested fix:** Rewards you for controlling, casting, or activating a Chandra planeswalker.

### `transform-mirror` — suspect
- **current:** A double-faced card whose back face mirrors its front face mechanically.
- **issue:** 'mirrors its front face mechanically' is unsupported; the two faces do different or opposite things, they don't mechanically mirror
- **example:** Chalice of Life // Chalice of Death: front gains you life, back drains opponents; faces are thematic opposites, not mechanical mirrors
- **suggested fix:** A double-faced card that transforms between its front and back face.

### `tutor-legendary` — suspect
- **current:** Searches your library for a legendary card and puts it into your hand.
- **issue:** 'puts it into your hand' is overspecified; some cards put it onto the battlefield
- **example:** Sisay, Weatherlight Captain: 'Search your library for a legendary permanent card with mana value less than Sisay's power, put that card onto the battlefield'
- **suggested fix:** Searches your library for a legendary card.

### `bounce` — suspect
- **current:** Returns a permanent to its owner's hand.
- **issue:** Underspecified: bounce also returns spells on the stack, not just permanents, and often creatures already in hand/battlefield temporarily
- **example:** Brutal Expulsion: 'Return target spell or creature to its owner's hand.'
- **suggested fix:** Returns a permanent or spell to its owner's hand.

### `cycle-hbg-alora` — suspect
- **current:** One version of Alora in a cycle of legendary creatures, each with a different ability.
- **issue:** Generic 'different ability' misses the shared core all versions have
- **example:** Alora, Cheerful Swashbuckler: 'Whenever you attack, up to one target attacking creature can't be blocked this turn... return that creature to its owner's hand. If you do, create a Treasure token.'
- **suggested fix:** A legend whose attack trigger makes an attacking creature unblockable then bounces it to hand, with a bonus effect that varies by version.

### `cycle-hbg-gut` — suspect
- **current:** One version of Gut in a cycle of legendary creatures, each with a different ability.
- **issue:** Generic 'different ability' misses the shared specialize token-copy core
- **example:** Gut, Devious Fanatic: 'When this creature specializes, create a token that's a copy of a creature card exiled with this creature... has flying and haste. Sacrifice it at the beginning of your next end step.'
- **suggested fix:** A legend whose specialize trigger creates a hasty token copy of a creature exiled with it that's sacrificed next end step, with stats and keywords varying by version.

### `cycle-hbg-imoen` — suspect
- **current:** One version of Imoen in a cycle of legendary creatures, each with a different ability.
- **issue:** Generic 'different ability' misses the shared unblockable combat-damage exile core
- **example:** Imoen, Occult Trickster: 'can't be blocked. Whenever Imoen deals combat damage to a player, you may exile an instant or sorcery card from your graveyard. If you do, create a 2/2 black Zombie creature token.'
- **suggested fix:** An unblockable legend that, on dealing combat damage to a player, may exile an instant or sorcery from your graveyard for a bonus effect that varies by version.

### `cycle-hbg-rasaad` — suspect
- **current:** One version of Rasaad in a cycle of legendary creatures, each with a different ability.
- **issue:** Vague/underspecified: 'a different ability' says nothing; most forms make tokens on death
- **example:** Rasaad, Warrior Monk: 'When Rasaad, Warrior Monk dies, create three 1/1 white Soldier creature tokens.'
- **suggested fix:** A monk legend that specializes and, in most forms, creates creature tokens when it dies.

### `cycle-hbg-sarevok` — suspect
- **current:** A legendary knight that pumps a creature by your graveyard's creature count and can specialize.
- **issue:** One form counts more than creatures in the graveyard
- **example:** Sarevok, Deceitful Usurper: '+X/+0 ... where X is the number of creature, instant, and sorcery cards in your graveyard.'
- **suggested fix:** A knight legend that specializes and each combat gives a creature you control +X/+0, where X is the number of creature cards in your graveyard.

### `cycle-hbg-skanos` — suspect
- **current:** A Dragon Vassal that grants another attacker a keyword and +X/+0 equal to its power.
- **issue:** Overspecified: base and green forms grant no keyword, just +X/+0 (green untaps)
- **example:** Skanos, Dragon Vassal: 'another target attacking creature gets +X/+0 until end of turn, where X is Skanos's power.' (no keyword)
- **suggested fix:** A Dragon Vassal that specializes and, when it attacks, gives another attacking creature +X/+0 equal to its power, often also granting a keyword.

### `cycle-jud-wormfang-vertical` — suspect
- **current:** When it enters, exiles one of your permanents, returning it when it leaves the battlefield.
- **issue:** Overspecified: not all members exile a permanent; one exiles your hand and one just skips a turn
- **example:** Wormfang Behemoth: 'When this creature enters, exile all cards from your hand... return the exiled cards to their owner's hand'; Wormfang Manta: 'When this creature enters, you skip your next turn... take an extra turn after this one'
- **suggested fix:** A Nightmare creature whose enter effect (exiling a land, creature, or your hand, or skipping a turn) is undone when it leaves the battlefield.

### `cycle-m19-mare` — suspect
- **current:** A Horse creature that can't be blocked by one color and has a bonus ability.
- **issue:** one tagged member (Diamond Mare) has no color-based unblockable clause
- **example:** Diamond Mare: 'As this creature enters, choose a color. Whenever you cast a spell of the chosen color, you gain 1 life.' (no 'can't be blocked by' text)
- **suggested fix:** A Horse creature that usually can't be blocked by creatures of one color and has a color-matters bonus ability.

### `cycle-neo-shrine` — suspect
- **current:** A Shrine enchantment creature that lets you pay 1 each end step for an effect that scales with the number of Shrines you control.
- **issue:** one tagged member (Go-Shintai of Life's Origin) has no pay-1 end-step scaling ability
- **example:** Go-Shintai of Life's Origin: '{W}{U}{B}{R}{G}, {T}: Return target enchantment card from your graveyard to the battlefield.' plus a Shrine-ETB token trigger, not a pay-{1} end-step effect
- **suggested fix:** A Shrine enchantment creature, most of which let you pay 1 at your end step for an effect that scales with the number of Shrines you control.

### `cycle-spellshaped-from-fut` — suspect
- **current:** One of a Future Sight cycle of simple creatures shaped like classic Magic spells.
- **issue:** 'shaped like classic Magic spells' isn't supported; several are plain vanilla or utility creatures
- **example:** Metallic Sliver is a blank 1/1 and Llanowar Elves is a mana dork ('{T}: Add {G}'), neither spell-shaped
- **suggested fix:** One of a Future Sight cycle of small, simple low-cost creatures.

### `cycle-usg-2-cycling-land` — suspect
- **current:** A land that enters tapped for one color but can be discarded for {2} to draw a card instead.
- **issue:** 'enters tapped for one color' fails for the colorless outlier that enters untapped
- **example:** Blasted Landscape enters untapped and taps for colorless: '{T}: Add {C}. Cycling {2}'
- **suggested fix:** A land that taps for mana (most enter tapped) and can be cycled by discarding it for {2} to draw a card.

### `fallout-perk-name` — suspect
- **current:** A card in the Fallout set's cycle named after a perk from the game.
- **issue:** 'cycle' is imprecise; these are a large themed group of cards (many Auras) named after Fallout perks, not a formal one-per-color cycle
- **example:** Strong Back, Nerd Rage, Almost Perfect and others are all Auras named after Fallout perks, far more than a standard cycle
- **suggested fix:** A card from the Fallout set named after a perk from the game.

### `gains-forestwalk` — suspect
- **current:** Grants a creature forestwalk, making it unblockable while the defending player controls a Forest.
- **issue:** 'gains-' means the creature grants forestwalk to ITSELF, but wording ('Grants a creature forestwalk') reads like a gives- tag; inconsistent with the parallel gains-shadow entry
- **example:** Wormwood Dryad: '{G}: This creature gains forestwalk until end of turn'
- **suggested fix:** Gains forestwalk itself, so it can't be blocked while the defending player controls a Forest.

### `greatest-power-matters` — suspect
- **current:** Rewards you for controlling the creature with the greatest power on the battlefield.
- **issue:** Narrow: many cards use the greatest power as a scaling VALUE, not reward you for controlling that creature
- **example:** Selvala, Heart of the Wilds: '{G}, {T}: Add X mana ... where X is the greatest power among creatures you control'; Cosmic Cube casts a spell with mana value up to the greatest power among your attackers
- **suggested fix:** Cares about the greatest power among creatures, rewarding you for controlling that creature or scaling an effect off it.

### `hate-typal-werewolf` — suspect
- **current:** Punishes or removes Werewolf creatures specifically, such as with protection, destruction, or bounce.
- **issue:** 'specifically' overstates it: several cards answer Werewolves as part of broader tribal hate, not Werewolves alone
- **example:** Slayer of the Wicked: 'you may destroy target Vampire, Werewolf, or Zombie'; Elite Inquisitor: 'Protection from Vampires, from Werewolves, and from Zombies'
- **suggested fix:** Punishes or answers Werewolf creatures, such as with protection from, destruction of, or bounce of Werewolves.

### `impulse-artifact-vehicle` — suspect
- **current:** Digs into your library for Vehicle or artifact creature cards, to your hand or the battlefield.
- **issue:** Narrows to 'artifact creature' when the cards mostly fetch Equipment/Vehicle (any artifact plus Vehicle)
- **example:** Astor, Bearer of Blades: 'reveal an Equipment or Vehicle card ... put it into your hand'; Nick Fury grabs 'Hero, Equipment, or Vehicle'
- **suggested fix:** Digs through your library for artifact or Vehicle cards and puts them into your hand or onto the battlefield.

### `protects-nonland` — suspect
- **current:** Temporarily exiles a nonland permanent, then returns it to the battlefield.
- **issue:** Not all cards return the permanent to the battlefield; many exile it to be recast (airbend/play from exile)
- **example:** Airbending Lesson: "Airbend target nonland permanent. (Exile it. While it's exiled, its owner may cast it for {2}...)" and Soul Partition: "Exile target nonland permanent. For as long as that card remains exiled, its owner may play it."
- **suggested fix:** Temporarily exiles a nonland permanent so it can later return to the battlefield or be recast.

### `pseudo-exert` — suspect
- **current:** Uses an ability that skips its next untap step as the cost for a bonus effect.
- **issue:** "as the cost for a bonus effect" misfits cards where skipping untap is a pure drawback, not a cost paid for a benefit
- **example:** Apes of Rath: "Whenever this creature attacks, it doesn't untap during its controller's next untap step." (no bonus granted)
- **suggested fix:** Skips its next untap step, mimicking exert, often in exchange for an activated ability's effect.

