# Oracle-tag mapping sweep — review

**Status: APPLIED (2026-07-13).** Machine-assisted audit (30 sub-agents, adversarially
verified against the live 4,492-tag catalog) of the two hand-authored otag mapping
tables. Populations are real card-counts from `card_oracle_tags`; every proposed slug
was confirmed to exist. **Applied to source and validated against a local `zervice`
recompute:** all Track A seed edits, all Track B root/override additions, and the 4
Track B wrong-inclusions as a new `ROLE_TAG_EXCLUSIONS` mechanism (`firebend-like` kept
in `ramp` by decision). Still open: the unmapped-archetype coverage gap below.

## Method

- **Track A** — deck-tag → seed otags (`deck_tag.rs::oracle_tag_slugs`), 53 seeded archetypes.
- **Track B** — card-role → otag roots (`derive_categories.rs` `CATEGORY_ROOTS`/`ROLE_TAG_OVERRIDES` + `oracle_tag_gaps.rs` regex), 26 roles.
- **Validity (mechanical, pre-audit):** all 145 referenced slug values exist in the catalog — **zero dead/typo'd**. This sweep is purely accuracy + completeness.
- Each batch was reviewed then a second agent pruned weak/already-covered/nonexistent proposals.

## Totals

| Track | Entries flagged | Proposed additions | Proposed removals |
|---|---|---|---|
| A (deck-tag seeds) | 40/53 | 98 | 14 |
| B (card-role roots) | 20/26 | 54 | 5 |

---

## Track A — deck-tag seed proposals

### A1. Removals (over-broad or off-theme seeds) — highest priority

Umbrella tags declared as a whole strategy pull in nearly the entire format; these are the sharpest corrections.

| Archetype | Remove | Pop | Why |
|---|---|---:|---|
| Spellslinger | `single-target-instant-sorcery` | 4631 | Too broad (4631 cards); a generic targeted-spell descriptor, not a spellslinger payoff. Dilutes the declared strategy. |
| Voltron | `evasion` | 4568 | Umbrella tag at 4568 cards; far too broad to declare a strategy, would tag nearly every creature. |
| Flying | `evasion` | 4568 | Mega-parent (verified 4568) spanning every evasion keyword; not flying-specific and dilutes the archetype's declared strategy. |
| Lifedrain | `death-trigger` | 584 | Too broad; generic Aristocrats/Sacrifice seed capturing many non-drain on-death effects. |
| Lands | `ramp` | 557 | Generic ramp (mana rocks/dorks/rituals) is not land-specific; it is the Ramp/BigMana seed and off-theme for a Lands-Matter identity. |
| Blink | `enters-in-company` | 490 | Parent is multiple-bodies (go-wide/tokens); not a flicker effect, off-theme for a flicker deck. |
| Flying | `gives-trample` | 454 | Trample (verified 454) is ground evasion, not flying; contradicts 'win in the air with evasive flying creatures'. |
| Discard | `discard-outlet` | 449 | Self-discard outlet (pitch your own cards for madness/graveyard), not opponent hand attack; off-theme for a 'strip opponents' hands' archetype. |
| Tokens | `out-of-color-token` | 415 | Color-identity metadata tag (token of a color outside the card's identity), not a token-matters payoff or generator; off-theme for the strategy seed. |
| Artifacts | `sacrifice-outlet-artifact` | 303 | This is a sacrifice-outlet tag (Sacrifice archetype), not an artifact-synergy payoff; off-theme for building an artifact engine. |
| Superfriends | `removal-planeswalker` | 148 | Off-theme/anti-synergy: cards that remove planeswalkers (planeswalker hate), the opposite of a Superfriends declared strategy. |
| ExtraCombats | `combat-ramp` | 145 | Parent is 'ramp'; denotes mana ramp produced via combat (firebending-style), not taking additional combat steps. Off-theme for the extra-combats strategy. |
| Auras | `auraify` | 64 | A type-change effect that makes non-auras into auras; niche and off the aura-voltron enlarge-a-creature intent. |
| Lifedrain | `opponent-lifegain` | 57 | Tags cards that make opponents gain life, the opposite of draining; off-theme and counterproductive. |

### A2. Additions (missing high-signal seeds)

Grouped by archetype; only archetypes with proposed adds. Confidence is the reviewer's.

**Aristocrats** _(confidence: high)_ — Intent: sacrifice your own creatures for value and incremental life drain. Current seeds sacrifice-outlet-creature, death-trigger, drain-life, and blood-artist-ability are all precise and archetype-defining. Missing the highest-signal fuel/engine tags: death-trigger-self (creatures with their own death payoffs, the fodder that makes aristocrats work) and repeatable-sacrifice-outlet (the recurring outlet that drives the engine).

| Add | Pop | Why |
|---|---:|---|
| `death-trigger-self` | 704 | Creatures that reward their own death are the core fuel of aristocrats; very high population and directly on-theme. |
| `repeatable-sacrifice-outlet` | 576 | A recurring free-to-use outlet is essential to the aristocrats loop; high-signal and defining. |

**Artifacts** _(confidence: high)_ — Intent: build the engine around artifacts and their synergies. The current seeds are surprisingly weak for this: they miss the single defining tag synergy-artifact (703) entirely, and lean on sacrifice-outlet-artifact which belongs to the Sacrifice bucket rather than to an artifact-synergy engine. Core 'artifacts matter' vocabulary (synergy-artifact-creature, affinity-for-artifacts, metalcraft) should anchor this archetype.

| Add | Pop | Why |
|---|---:|---|
| `synergy-artifact` | 703 | The single defining 'artifacts matter' tag; its absence is the biggest gap in this seed set. |
| `synergy-artifact-creature` | 146 | Directly rewards artifact creatures, a core pillar of artifact synergy decks. |
| `affinity-for-artifacts` | 42 | Classic artifact-count payoff mechanic that defines artifact-density decks. |
| `metalcraft` | 40 | Iconic 'artifacts matter' mechanic (child of synergy-artifact) rewarding artifact count. |

**AttackTriggers** _(confidence: high)_ — Intent: reward attacking with triggers that snowball each combat. Core seeds attack-trigger (2034), attacking-matters (1213), attacking-matters-self (1593) are all on-theme and archetype-defining. Missing the two most obvious payoffs that literally snowball combat: titan-trigger (a subtype of attack-trigger) and extra-combat-phase (re-trigger everything an extra time).

| Add | Pop | Why |
|---|---:|---|
| `titan-trigger` | 163 | Direct child of attack-trigger; a signature attack-trigger effect that belongs in this bucket. |
| `extra-combat-phase` | 53 | Extra combats are the definitive 'snowball each combat' payoff that reuses all attack triggers. |

**Burn** _(confidence: high)_ — Intent is 'deal direct damage with spells to creatures or players.' Current seeds (burn-any 905, burn-planeswalker 378) cover 'any target' and the narrow planeswalker case, but conspicuously omit the two highest-signal, on-intent tags: burn-player and burn-creature. Adding a repeatable-damage tag (pinger) rounds out the classic burn shell. burn-planeswalker is narrow but still on-theme, kept.

| Add | Pop | Why |
|---|---:|---|
| `burn-creature` | 1042 | Directly matches intent ('to creatures'); highest-population burn tag, archetype-defining. Verified: pop 1042 in catalog, parent tags removal-burn/removal-creature. |
| `burn-player` | 875 | Directly matches intent ('to players'); core win condition of a burn deck and higher signal than the currently-seeded burn-planeswalker. Verified: pop 875 in catalog, parent tag 'burn'. |
| `pinger` | 750 | Repeatable direct-damage sources, a staple burn engine; strong high-signal addition. Verified: pop 750 in catalog. |

**Discard** _(confidence: high)_ — Intent is opponent-facing: 'strip opponents' hands to deny resources'. discard (548) and hate-discard (134) fit. But discard-outlet (449) is a SELF-discard enabler (madness/reanimator/graveyard decks), not opponent hand attack, so it is off-theme here. The set is also thin on the hand-disruption vocabulary that defines this archetype.

| Add | Pop | Why |
|---|---:|---|
| `thoughtseize` | 144 | Targeted opponent discard (hand-disruption/peek-hand); the defining tag for proactive hand attack. |
| `discard-to-exile` | 72 | Opponent discard that exiles (hand-disruption); high-signal disruption belonging to this theme. |
| `specter-ability` | 63 | Combat-triggered opponent discard (hand-disruption/saboteur); core to aggressive discard decks. |
| `random-discard` | 39 | Forced random opponent discard (hand-disruption, e.g. Hymn effects); squarely on-theme. |

**Draw** _(confidence: high)_ — draw-engine (1482), pure-draw (1270), repeatable-pure-draw (1327), and cantrip (650) are the four strongest pure card-draw tags and match 'refill your hand with extra card draw' perfectly. The one clear gap is burst-draw: large one-shot draw (draw-3 style) is core card draw not captured by the pure/cantrip/repeatable tags.

| Add | Pop | Why |
|---|---:|---|
| `burst-draw` | 578 | One-shot multi-card draw (parent: draw); a major pillar of card-draw decks missing from the seed set. |

**Energy** _(confidence: high)_ — Intent is 'accumulate energy counters and spend them for value'. energy-generator (125) covers the accumulate side only. counter-fuel-energy (124) is the spend/payoff side and completes the accumulate+spend pair the description calls for; energy-increaser (6) and synergy-energy (2) are too small to seed.

| Add | Pop | Why |
|---|---:|---|
| `counter-fuel-energy` | 124 | Cards that spend energy counters for value; the payoff half of the archetype, complementing energy-generator's accumulation. |

**Equipment** _(confidence: high)_ — Both current seeds (synergy-equipment, french-vanilla-equipment) are correct and archetype-defining. The set is accurate but thin: it captures equipment payoffs and keyword-granting gear but misses the efficient equipment-cards themselves and the modified-matters payoff. Adding these tightens the pool without drifting off-theme.

| Add | Pop | Why |
|---|---:|---|
| `quick-equip` | 104 | Cheap/free-equip gear is the backbone of any equipment deck; direct equipment mechanic, on-theme. |
| `living-weapon` | 83 | Living Weapon equipment (self-attaching) is a defining equipment sub-type. |
| `synergy-modified` | 48 | Modified-matters payoffs reward equipped creatures directly (parent includes synergy-equipment). |

**Flying** _(confidence: high)_ — Intent: 'Win in the air with evasive flying creatures'. Current seeds are both poorly chosen. evasion (4568) is the mega-parent covering ALL evasion (unblockable, menace, shadow, etc.), far too broad to be flying-defining. gives-trample (454) is off-theme: trample is ground evasion, not flying, and contradicts 'win in the air'. Both should be removed in favor of flying-specific tags: gives-flying and gains-flying (grant/gain flight) and synergy-flying (flyers-matter payoffs), which tightly capture the air-based archetype.

| Add | Pop | Why |
|---|---:|---|
| `gives-flying` | 460 | Granting flying is the primary way a flyers deck pushes damage through the air. Verified: catalog population 460, parent 'gives-evasion'. |
| `gains-flying` | 314 | Creatures that gain flying reinforce the evasive-air game plan. Verified: catalog population 314. |
| `synergy-flying` | 114 | Flyers-matter payoffs (anthem/tribal-style) are the tight, flying-specific synergy core. Verified: catalog population 114. |

**Graveyard** _(confidence: high)_ — Current seeds (graveyard-fuel, reanimate-creature, regrowth-creature, mill-self) are all correct and on-theme for a graveyard-value deck. But the set skews toward putting-cards-in and pulling-creatures-back; it misses the two strongest general graveyard-value signals: casting spells straight from the yard and cards-that-care-about-graveyard-contents. Adding self-recursion tags rounds out the 'recur and reuse' intent.

| Add | Pop | Why |
|---|---:|---|
| `castable-from-graveyard` | 403 | Highest-signal general graveyard-value tag (cast spells from the yard); core to 'reuse cards' and larger than any current seed except reanimate-creature. |
| `reanimate-self` | 272 | Self-returning recursion (creatures that bring themselves back); fits 'recur and reuse', complements reanimate-creature without overlap. |
| `regrowth-self` | 194 | Self-recurring cards back to hand; a clean 'reuse cards' signal parallel to regrowth-creature already present. |
| `cards-in-graveyard-matter` | 174 | Direct graveyard-as-resource payoff (effects scale with yard contents); archetype-defining for a Graveyard strategy. |

**GroupSlug** _(confidence: high)_ — Only a single seed (group-slug), which under-specifies an archetype the deck_tag doc defines as 'punish the whole table with symmetric damage and taxes.' The catalog has strong, on-theme tags for both halves (symmetric burn + punisher damage, and taxes) that should seed alongside it. Avoided the too-broad 'symmetrical' (749) tag.

| Add | Pop | Why |
|---|---:|---|
| `rhystic` | 259 | Pay-or-suffer table-wide taxes (catalog parent is tax); highest-count on-theme tax tag matching the archetype's stated 'taxes' half. |
| `toll` | 179 | Table-wide taxes/tolls (catalog parent is tax); directly matches the 'and taxes' half of the archetype definition. |
| `punisher` | 139 | Punisher/choose-your-poison effects that punish the table; central to the 'punish the whole table' intent. |
| `burn-player-each` | 132 | Symmetric damage to each player is the literal core of Group Slug; catalog taxonomy even lists group-slug as a parent tag, confirming the fit. |

**Lifegain** _(confidence: high)_ — Seeds lifegain (878), repeatable-lifegain (1382), and lifegain-matters (125) form a strong, well-balanced set: the gain, the repeatable engine, and the payoffs. The one clearly missing high-signal mechanic is lifelink, the primary lifegain engine in Commander, which is tagged separately from the static/triggered 'lifegain' slug. Adding the lifelink granter/gainer slugs completes the engine side.

| Add | Pop | Why |
|---|---:|---|
| `gives-lifelink` | 230 | Granting lifelink is the dominant repeatable lifegain engine, not captured by the lifegain slug. |
| `gains-lifelink` | 104 | Creatures gaining lifelink themselves; complements gives-lifelink as a core lifegain source. |

**Mill** _(confidence: high)_ — mill-opponent, mill-any, mill-each cleanly cover the directions of decking opponents out. Correct and focused. The one clearly missing high-signal variant is exile-based mill, which is a distinct and important Mill mode (denies graveyard recursion).

| Add | Pop | Why |
|---|---:|---|
| `mill-exile` | 116 | Exile-mill is a core Mill variant (catalog parent is mill; mills into exile, beats graveyard value); complements the directional mill tags without overlap. |

**Ramp** _(confidence: high)_ — All four seeds (ramp, land-ramp, mana-dork, multi-land-ramp) are correct and archetype-defining. The clear gap is mana rocks: the pool covers land-ramp and creature dorks but omits artifact ramp entirely, which is a core EDH ramp category.

| Add | Pop | Why |
|---|---:|---|
| `utility-mana-rock` | 241 | Mana rocks are a primary EDH ramp category completely missing from the seeds; high population. |
| `mana-rock` | 109 | Broader mana-rock tag complements utility-mana-rock to cover artifact ramp. |

**Reanimator** _(confidence: high)_ — Current seeds are excellent: reanimate-creature + reanimate-cast are the payoff, discard-outlet + mill-self are the enablers that dump fatties into the yard. Nothing to remove. Missing the reanimation-variant tags that broaden coverage of how big creatures get cheated back.

| Add | Pop | Why |
|---|---:|---|
| `temporary-reanimation` | 97 | Cheat a creature in for a turn/attack; a defining Reanimator sub-line not covered by permanent reanimate-creature. |
| `reanimate-from-any` | 95 | Reanimation targeting any graveyard, a staple mode of Reanimator (steal opponents' bombs); direct variant of the core mechanic. |
| `mass-reanimation` | 56 | Mass/board-wide reanimation is the payoff ceiling of the archetype ('cheat large creatures into play' at scale). |

**Removal** _(confidence: high)_ — Intent 'lean on efficient spot removal to answer any threat.' Seeds (spot-removal 4980, sweeper 740, removal-fight 145) are correct but thin on high-signal removal vocabulary: the catalog holds much larger, more archetype-defining tags for a removal-centric deck. removal-fight is the weakest (overlaps the Fight archetype) but stays as still-valid removal. Adding recurring, exile-based, and multi-target removal better fulfills 'answer any threat.'

| Add | Pop | Why |
|---|---:|---|
| `repeatable-removal` | 1773 | Recurring answers give the removal deck inevitability; very high population and directly archetype-defining. Verified: pop 1773 in catalog, parent tag 'removal'. |
| `multi-removal` | 638 | Removes multiple threats per card, core to an efficient removal-heavy shell. Verified: pop 638 in catalog, parent tag 'removal'. |
| `removal-exile` | 448 | Exile-based removal answers indestructible/recursive threats, matching 'answer any threat.' Verified: pop 448 in catalog, parent tag 'removal'. |

**Sacrifice** _(confidence: high)_ — Intent: use sacrifice outlets to turn permanents (not just creatures) into value. Current seeds sacrifice-outlet-creature, repeatable-sacrifice-outlet, death-trigger are good. Since the theme is permanents broadly, the map should include the non-creature outlets to differentiate it from Aristocrats: sacrifice-outlet-artifact, sacrifice-outlet-land, and free-sacrifice-outlet round out the outlet coverage.

| Add | Pop | Why |
|---|---:|---|
| `sacrifice-outlet-artifact` | 303 | Broadens the outlet set beyond creatures, matching the 'turn permanents into value' intent. |
| `sacrifice-outlet-land` | 238 | Land outlets are a key non-creature outlet type reinforcing the permanents-into-value theme. |
| `free-sacrifice-outlet` | 183 | Free/no-cost repeatable outlets are the strongest engine pieces for a sacrifice deck. |

**SelfMill** _(confidence: high)_ — mill-self is the exact core; graveyard-fuel and reanimate-creature are legitimate payoffs for a self-mill shell. Correct but thin on the count-based payoffs that self-mill specifically enables. Adding a self-mill enabler and graveyard-size payoffs sharpens 'mill your own library to fuel graveyard payoffs'.

| Add | Pop | Why |
|---|---:|---|
| `cards-in-graveyard-matter` | 174 | Payoff that scales with a full yard, exactly what self-milling produces; highest-count relevant payoff. |
| `undergrowth` | 116 | Graveyard-creature-count payoff mechanic that self-mill fuels; tight self-mill signal. |
| `mulch` | 115 | Self-mill enabler that fills the yard while advancing the board (catalog parent is mill-self); the most on-theme missing engine tag for SelfMill. |
| `threshold` | 109 | Classic self-mill payoff (bonuses at 7+ cards in yard); a graveyard-size payoff tightly rewarding milling yourself. |

**Theft** _(confidence: high)_ — All three current seeds `theft-creature` (236), `theft-cast` (200), `theft-artifact` (54) are correct and core. The set is accurate but incomplete: temporary steal (threaten) and broader/mass theft variants are the strongest missing high-signal tags in the theft vocabulary.

| Add | Pop | Why |
|---|---:|---|
| `threaten` | 82 | Temporary steal (Act of Treason effects); a defining theft subtheme and the highest-count omission. |
| `theft-permanent` | 35 | General permanent theft, covering steals beyond creatures/artifacts/spells. |
| `theft-mass` | 29 | Mass steal effects (Insurrection-style), a signature Theft payoff. |
| `synergy-theft` | 28 | Direct theft-synergy payoff tag tying the strategy together. |

**Tokens** _(confidence: high)_ — Intent: 'Generate creature tokens to go wide and fuel payoffs.' `repeatable-creature-tokens` (1441) and `anthem` (466) are solid, but `out-of-color-token` (415) is a color-identity classification tag (a card makes a token outside its color), not a token-strategy payoff, and it does not define the archetype. Replacing it with creature-token payoff/synergy tags makes Tokens tighter and better differentiated from GoWide.

| Add | Pop | Why |
|---|---:|---|
| `synergy-token-creature` | 110 | Core creature-token-matters payoff tag; directly captures the 'fuel payoffs' half of the intent. |
| `synergy-token` | 104 | Broader token-synergy payoff tag; reinforces the token-matters theme distinguishing Tokens from a pure GoWide anthem plan. |

**Untap** _(confidence: high)_ — Intent: 'Untap permanents to reuse abilities and generate value'. Current seeds untapper-creature (443) and untapper-permanent (76) are both correct and archetype-defining. Set is accurate but incomplete: it omits the payoff/enabler tags that define an untap-value deck. extra-untap covers the classic Seedborn Muse / Wilderness Reclamation style extra-untap engines, untaps-self covers permanents that untap themselves to re-use abilities (Nettle Sentinel etc.), and untapper-land covers untapping lands for mana value. No removals needed.

| Add | Pop | Why |
|---|---:|---|
| `untaps-self` | 173 | Permanents that untap themselves to reuse their activated abilities, central to the untap engine. Verified: catalog population 173, parent 'untapper'. |
| `extra-untap` | 105 | Extra untap-step effects (Seedborn Muse, Murkfiend Liege) are the core untap-value payoff and directly match 'reuse abilities'. Verified: catalog population 105. |
| `untapper-land` | 99 | Untapping lands for repeated mana is a defining untap-value line, complements the permanent/creature untappers. Verified: catalog population 99, parent 'untapper'. |

**Voltron** _(confidence: high)_ — Four of five seeds are correct: protects-creature, gives-hexproof, gives-indestructible (protect the single threat) and synergy-equipment (core buff engine). However evasion is a top-level umbrella spanning 4568 cards (every evasive creature in the format); as a declared-strategy seed it massively over-selects and is not archetype-defining. Replace it with the aura half of Voltron and a targeted way to connect for commander damage.

| Add | Pop | Why |
|---|---:|---|
| `gives-trample` | 454 | Trample is the canonical Voltron connect mechanic for pushing commander damage through blockers, tighter than raw evasion. |
| `synergy-aura` | 136 | Auras are the second pillar of Voltron buffs alongside equipment; currently absent. |

**Wheels** _(confidence: high)_ — Intent: 'Force everyone to discard and draw fresh hands'. Current seeds wheel-one-sided (49), wheel-symmetrical (39), and miniwheel (46) are all precise and correct, spanning the one-sided, symmetric, and partial-wheel variants. The only clearly-belonging omission is whirlpool (parent = wheel), the shuffle-hands-and-redraw variant (Whirlpool Warrior, Winds of Change) which is a canonical wheel effect. No removals.

| Add | Pop | Why |
|---|---:|---|
| `whirlpool` | 36 | Whirlpool effects (shuffle hands into library and redraw) are a recognized wheel variant under the 'wheel' parent, filling the last wheel sub-type. Verified: catalog population 36, parent 'wheel'. |

**Aggro** _(confidence: medium)_ — Intent: cheap creatures, early pressure to win before opponents stabilize. Current seeds attacking-matters, attacking-matters-self, and gives-haste fit well. attack-trigger is retained (attacking payoffs support the plan) though it overlaps heavily with the AttackTriggers archetype. Only gap is a damage-through-enabler; gives-trample is the classic aggro finisher to close games past blockers. Seeds are otherwise solid.

| Add | Pop | Why |
|---|---:|---|
| `gives-trample` | 454 | Trample is a core aggro tool to push damage past blockers and close out games; fits early-pressure plan and is high-signal. |

**Auras** _(confidence: medium)_ — Intent: stack enchantment auras onto a creature to enlarge it (aura-voltron). synergy-aura and french-vanilla-aura are strong, on-theme seeds. auraify is a type-change effect (turning things into auras) that is tangential to a voltron-buff plan and better dropped. Missing the literal buff-aura tags that enlarge a creature: vanilla-aura and parasitic-aura, plus synergy-modified which covers aura/equipment/counter voltron payoffs.

| Add | Pop | Why |
|---|---:|---|
| `synergy-modified` | 48 | Rewards modified creatures (auras/equipment/counters), a central aura-voltron payoff (child of synergy-aura). |
| `parasitic-aura` | 47 | Buff-style auras that stack onto a creature, directly serving the voltron enlarge plan. |
| `vanilla-aura` | 45 | Plain stat-boost auras are the literal building blocks of an aura-voltron creature. |

**BigMana** _(confidence: medium)_ — All four seeds are correct: mana-sink and bottomless-mana-sink are the payoffs (where huge mana goes), ramp and land-ramp are the enablers. Accurate but the payoff side under-represents the description's explicit 'oversized threats and X spells' theme, and lacks mana-doublers which are the signature of big-mana decks.

| Add | Pop | Why |
|---|---:|---|
| `mana-increaser` | 61 | Mana doublers/increasers are the defining big-mana enabler distinct from ordinary ramp. |
| `high-x-matters` | 40 | Direct payoff for the 'X spells' half of the archetype description. |

**Blink** _(confidence: medium)_ — flicker-creature and flicker-slow are correct core flicker enablers. enters-in-company (parent multiple-bodies) is a go-wide/tokens tag about creatures entering alongside others, not a flicker mechanic; it is off-theme for a deck whose intent is explicitly 'flicker creatures out and back.' Swap it for genuine flicker tags to keep the pool on-theme.

| Add | Pop | Why |
|---|---:|---|
| `flicker-self` | 40 | Creatures that blink themselves for repeatable value are self-contained Blink payoffs. |
| `flicker-permanent` | 15 | Broadens the flicker-enabler category to any permanent, staying strictly on the flicker theme. |

**Clone** _(confidence: medium)_ — Intent 'copy the best creatures and permanents on the battlefield.' Seeds (copy-creature 358, copy-self 312, clone 72) are correct and central. Deliberately excludes spell-copy tags (copy-instant/sorcery/spell) which belong to spellslinger, not Clone. shapesharing (shapeshifters that copy creatures/permanents, parent tag 'copy') fits the intent well and strengthens coverage. conjure-duplicate was rejected: its parent tag is 'conjure', a Magic Arena / Alchemy digital-only mechanic that creates cards from nowhere rather than copying battlefield permanents, off-intent for paper Commander.

| Add | Pop | Why |
|---|---:|---|
| `shapesharing` | 69 | Shapeshifters that copy other creatures/permanents; a defining clone mechanic, comparable signal to seeded 'clone'. Verified: pop 69 in catalog, parent tag 'copy'. |

**Etb** _(confidence: medium)_ — flicker-creature (the reuse enabler) is correct; enters-in-company is defensible here as ETB-creature payload for a value deck. The set is only two seeds and misses the single strongest ETB payoff tag in the catalog (creaturefall = whenever a creature enters) plus a second reuse enabler.

| Add | Pop | Why |
|---|---:|---|
| `creaturefall` | 548 | Highest-signal ETB payoff tag: fires whenever a creature enters, exactly the 'repeatable value' engine described. |
| `flicker-slow` | 107 | Delayed/end-of-turn flicker is a key repeatable ETB-reuse enabler, complementing flicker-creature. |

**Hatebears** _(confidence: medium)_ — Current seeds hatebear (65) and tax-attack (40) are both correct and archetype-defining: small disruptive creatures plus attack taxation. Intent is 'tax and deny,' but the seed set only covers the attack-tax slice of taxation. The strongest missing dimension is spell/action taxation (Thalia-style), which is core to hatebears. Adding the tax-family slugs rounds this out without over-broadening. Kept conservative to defined 'tax' mechanics rather than the sprawling hate-* color-hate leaves, which the hatebear tag already implies.

| Add | Pop | Why |
|---|---:|---|
| `toll` | 179 | Generic pay-to-act taxation; core disruptive 'tax' mechanic for a hatebears shell. |
| `cast-tax` | 17 | Spell-cast taxation (Thalia, Grand Arbiter) is the definitional hatebears effect not covered by tax-attack. |

**Infect** _(confidence: medium)_ — Intent: 'Attack with infect creatures, dealing damage as poison counters'. Current single seed poisonous (131) is the correct core tag (the infect/toxic/poison-dealing bodies). Too thin though: it misses the poison payoff/support layer. synergy-poison covers proliferate and poison-counter payoffs that make an infect deck function, poison-opponents covers non-combat poison sources, and gives-infect lets non-infect creatures carry the win. All are tightly on-theme.

| Add | Pop | Why |
|---|---:|---|
| `synergy-poison` | 42 | Poison-counter payoffs and proliferate support that an infect win condition relies on. Verified: catalog population 42, parent 'poison-mechanics'. |
| `poison-opponents` | 21 | Direct poison-counter sources beyond combat damage, on-theme for a poison win. Verified: catalog population 21, parent 'poison-mechanics'. |
| `gives-infect` | 8 | Granting infect turns any creature into a poison threat, core to infect strategies (Grafted Exoskeleton, Triumph of the Hordes). Verified: catalog population 8, parent 'poison-mechanics'. |

**Landfall** _(confidence: medium)_ — Seeds landfall (221) and land-ramp (487) are correct: the payoff tag plus the primary enabler that drops lands to trigger it. Intent is 'trigger payoffs each time a land enters,' so the highest-value missing tags are additional trigger generators, extra land drops, and land-to-battlefield tutors (fetchland style). These directly multiply landfall triggers and are the strongest enablers by card count.

| Add | Pop | Why |
|---|---:|---|
| `tutor-land-to-battlefield` | 275 | Fetchland/land-to-play effects are the classic landfall trigger source; high population. |
| `multi-land-ramp` | 163 | Puts multiple lands into play at once, generating multiple landfall triggers per cast. |
| `play-additional-land` | 45 | Extra land drops each turn directly convert to extra landfall triggers. |

**Lands** _(confidence: medium)_ — Label is 'Lands Matter,' intent 'treat lands as the engine, ramping and recurring them for value.' Seeds landfall and land-ramp are on-theme, but 'ramp' (557) is too broad: it is generic ramp (mana rocks, dorks, rituals) and is the defining seed of the BigMana/Ramp archetypes, diluting the lands-specific identity. Critically, the definitional 'lands-matter' tag itself is absent, along with the land-count payoff and land recursion tags the intent explicitly calls out. Recommend swapping generic ramp for a tight lands-matter package.

| Add | Pop | Why |
|---|---:|---|
| `multi-land-ramp` | 163 | Land-specific ramp that grows the land engine, unlike generic ramp. |
| `lands-matter` | 102 | The definitional tag for the archetype; currently missing entirely. |
| `land-count-matters` | 91 | Payoffs scaling with number of lands, the core 'value' half of the intent. |
| `reanimate-land` | 51 | Land recursion (Crucible-style), directly serving the 'recurring them for value' intent. |

**Lifedrain** _(confidence: medium)_ — Intent: 'Drain opponents' life while padding your own total.' drain-life (365) is the correct definitional seed. But opponent-lifegain (57) is off-theme, arguably counterproductive: it tags cards that make OPPONENTS gain life, the opposite of a drain plan. And death-trigger (584) is too broad: it is the generic Aristocrats/Sacrifice seed and captures unrelated on-death effects. The precise drain mechanics that fit the intent are blood-artist-ability (the drain-on-death effect that both loses opponents life and pads yours), life-loss-matters payoffs, and combat drain. Recommend swapping the two weak seeds for these tighter ones, which also differentiates Lifedrain from Aristocrats.

| Add | Pop | Why |
|---|---:|---|
| `life-loss-matters` | 124 | Payoffs triggered when life is lost, the core reward layer for a drain strategy. |
| `blood-artist-ability` | 39 | The definitional lifedrain mechanic: drains opponents on death while padding your total (drain-life + repeatable-lifegain). |

**Ping** _(confidence: medium)_ — Current seed `pinger` (750) is the exact core of the archetype and correct. The set is too thin, though: it names the enablers (pingers) but no payoffs. Adding damage-trigger payoffs that reward repeated 1-damage pings rounds out the declared strategy while staying on-theme.

| Add | Pop | Why |
|---|---:|---|
| `combat-neutral-damage-trigger` | 108 | Triggers on noncombat/any damage; the canonical payoff for repeated pinging, the missing engine half of a Ping deck. |

**Spellslinger** _(confidence: medium)_ — `second-spell-matters` (71) and `copy-spell` (152) are correct spellslinger payoffs. But `single-target-instant-sorcery` (4631) is a card-type descriptor spanning essentially every targeted instant/sorcery (removal, burn, etc.), not a spellslinger payoff, and is far too broad to be archetype-defining. The stronger 'cast lots of noncreature spells with payoffs' tags are missing.

| Add | Pop | Why |
|---|---:|---|
| `cast-trigger-you` | 1277 | Magecraft/prowess-style 'when you cast' triggers, the central spellslinger payoff mechanism. |
| `synergy-noncreature` | 304 | Direct noncreature-spell synergy (the prowess/magecraft 'noncreature spell' vocabulary), tighter and more on-theme than the broad targeted-spell descriptor. |
| `magecraft` | 33 | The literal spellslinger keyword mechanic; small but maximally archetype-defining. |

**Superfriends** _(confidence: medium)_ — `synergy-planeswalker` (144) and `protects-planeswalker` (100) are correct and central. But `removal-planeswalker` (148) is anti-theme: its parent is `removal` and it tags cards that DESTROY planeswalkers (hate), the opposite of a deck deploying and protecting your own walkers. The intent explicitly mentions protecting loyalty, so proliferate is the natural missing piece.

| Add | Pop | Why |
|---|---:|---|
| `pseudo-proliferate` | 39 | Grows loyalty counters, directly serving the 'protect their loyalty' intent; strongest non-walker Superfriends tag. |
| `synergy-proliferate` | 7 | Explicit proliferate synergy reinforcing the loyalty-counter axis of Superfriends. |

**Toolbox** _(confidence: medium)_ — Intent: 'Tutor up the right answer from a versatile card pool'. Current seeds tutor-to-hand (557) and tutor-creature (99) are correct and central. Incomplete: the defining toolbox engine is fetching creatures/answers directly onto the battlefield (Birthing Pod, Chord of Calling) via tutor-to-battlefield, plus category tutors that fetch the situational answer (tutor-artifact, tutor-instant). Deliberately excluding land tutors (tutor-land-* / tutor-to-hand covers general), which are ramp not toolbox. No removals.

| Add | Pop | Why |
|---|---:|---|
| `tutor-to-battlefield` | 174 | Pod/Chord-style put-into-play tutors are the archetypal creature-toolbox engine. Verified: catalog population 174, parent 'tutor-to'. |
| `tutor-artifact` | 46 | Fetching the right artifact answer from a versatile pool is core toolbox behavior. Verified: catalog population 46, parent 'tutor'. |
| `tutor-instant` | 23 | Silver-bullet instant tutors provide the situational answers that define a toolbox deck. Verified: catalog population 23, parent 'tutor'. |

**Vehicles** _(confidence: medium)_ — `crew` (198) and `synergy-vehicle` (126) are correct and central. Missing tags all concern the crewing dynamic that defines the deck: self-crewing, alternative crew costs, and crewless vehicles strengthen the 'crew vehicles for efficient attackers' intent without drifting into tangential artifact-creature territory (e.g. synergy-artifact-creature is broader/off-axis).

| Add | Pop | Why |
|---|---:|---|
| `bring-your-own-crew` | 26 | Vehicles that supply their own crew; directly addresses the archetype's core crewing tension. |
| `alternative-crewing` | 21 | Alternate ways to crew, a defining Vehicles enabler. |
| `crewless-vehicle` | 21 | Vehicles that attack without crewing, a key removal-resistant-attacker payoff for the theme. |

---

## Track B — card-role root proposals

### B1. Wrong inclusions (roots dragging in off-role tags)

| Role | Drop | Pop | Why |
|---|---|---:|---|
| counters | `counter-fuel-aesthetic` | 128 | VERIFIED in counters-matter subtree, catalog pop 128, parent counter-fuel. Cosmetic/reminder counters (keyword/functional-reminder), not a +1/+1 archetype payoff; adds noise via the counters-matter pull. |
| mill | `repeatable-maps` | 6 | VERIFIED: present in the mill subtree at depth 3 via its 'surveil' parent (catalog parents: gives-pp-counters, repeatable-artifact-tokens, repeatable-noncreature-tokens, repeatable-pp-counters, surveil). This tag is about repeatable Map (artifact) token generation, not milling. Off-role grouping; population only 6 so low impact, but it mis-groups token-generator cards under Mill. |
| ramp | `firebend-like` | 54 | VERIFIED: present in ramp subtree at depth 2 via combat-ramp (pop 54), also tagged attack-trigger. It is an attack-triggered firebreathing-style pump, not true mana acceleration; off-role for the ramp group. |
| removal | `burn-self` | 14 | VERIFIED: present in removal subtree at depth 3 via burn-creature (pop 14). Denotes damage dealt to the controller (painland downside), not removal. Off-role. |
| untap | `twiddle` | 62 | VERIFIED present in the untapper subtree and dual-parented (tapper,untapper). Pure-tapper twiddle cards (a stax/tempo effect) get grouped under untap. Mild mixed inclusion, not worth removing since most twiddle cards can untap. |

### B2. Missing tags (fall into "Other", should be a root or override)

`kind=root` heads a clean subtree (subtree-expanded); `kind=override` is an exact single-leaf patch.

**aggression** _(coverage: under, confidence: high)_ — Roots (attack-trigger 2034, attacking-matters 1213, attacking-matters-self 1593, attacking-matters-any 14, gives-haste 626) form a broad, on-role attack/haste umbrella; the gives-haste subtree's inclusions (earthbend, awaken, gives-suspend/unearth/encore/riot/blitz) all grant haste and belong. Two verified gaps remain. gains-haste (276) is a distinct root from gives-haste (a creature gaining haste for itself); it is confirmed absent from the gives-haste subtree, is not a child of any aggression root, and therefore lands in 'Other' today. extra-combat-phase (53, parented under phase-manipulation) is a staple aggression finisher whose parent is not a role root, so it also falls into 'Other'. Both are proper exact-override targets. gives-menace 178 correctly stays under evasion, not aggression. Under-covers by these two leaves.

| Add | Kind | Pop | Why |
|---|---|---:|---|
| `gains-haste` | override | 276 | Verified: catalog pop 276, root-level (no parent), absent from the gives-haste subtree and every aggression root, so it currently falls into 'Other'. Creature gains haste itself, a core aggression tag. Exact override adds precisely it. |
| `extra-combat-phase` | override | 53 | Verified: catalog pop 53, parent phase-manipulation which is not a role root, so it never enters the attacking/haste subtree and lands in 'Other'. Additional combat step is a classic aggression finisher. |

**burn** _(coverage: under, confidence: high)_ — Root burn expands into the full direct-damage tree (burn-player 875, burn-creature 1042, burn-any 905, burn-planeswalker 378, bombard/bombard-self, burn-player-each) with good coverage. The fight/damage tags it drags in (one-sided-fight 161, removal-fight 145, abrade 33, outnumber 26, drain-creature 75) are all genuine damage-dealing effects that Scryfall parents under the burn subtree, so they are legitimate, not wrong inclusions. The one large verified gap is pinger (750): a flat root-level leaf (no parent) for repeatable-ping damage, confirmed absent from the burn subtree, so it lands in 'Other' despite being a textbook burn category. burn-with-set-s-mechanic (278) and combat-ping (34) are lesser burn tags also outside the tree but the set-staple tag carries over-cover risk, so only pinger is proposed.

| Add | Kind | Pop | Why |
|---|---|---:|---|
| `pinger` | override | 750 | Verified: catalog pop 750, root-level (no parent, 0 children), confirmed absent from the burn subtree, entirely in 'Other' today despite being repeatable direct damage. Override adds exactly it. |

**copy** _(coverage: good, confidence: high)_ — Well covered. Roots copy+clone subtree-expand across the whole copy family (copy-creature 358, copy-self 312, copy-instant/sorcery/spell, clone 72, shapesharing 69, copy-token, gives-myriad/storm/replicate/encore/casualty, reanimate-copy 53). Verified: reanimate-copy and gives-embalm/encore are dual copy+recursion by design; no wrong inclusions. Only real gap is conjure-duplicate (pop 70, parent 'conjure' = digital-only-mechanics), which creates a token copy but nests outside the copy/clone subtree. Confirmed in catalog, absent from every subtree, a leaf (no children). Low priority: it is an Alchemy tag, largely irrelevant to a paper Commander builder.

| Add | Kind | Pop | Why |
|---|---|---:|---|
| `conjure-duplicate` | override | 70 | VERIFIED: catalog pop 70, parent 'conjure' (digital-only), not in any subtree, leaf. Creates a token copy = genuine copy effect. Add as exact override only if Alchemy/digital cards are in scope; leaf, no subtree. |

**drain** _(coverage: under, confidence: high)_ — Under-covered. Root drain-life (365) captures blood-artist-ability (39) and gives-extort (1); drain-life is dual inverted-effects/lifegain by design, no wrong inclusions. One genuine gap confirmed: drain-creature (75, parent burn-creature/lifegain) is a literal drain effect (deal damage to a creature, gain that life) sitting outside drain-life. The proposed life-loss-matters was REJECTED on verification: it is a root-level aristocrats 'matters' payoff umbrella (heads self-life-loss-matters 35, gives-spectacle 1), not a drain effect itself, only tangentially related; it belongs to a life-loss/aristocrats concern, not drain. drain-strength (23) correctly excluded as a -X/-X debuff.

| Add | Kind | Pop | Why |
|---|---|---:|---|
| `drain-creature` | override | 75 | VERIFIED: catalog pop 75, parent burn-creature/lifegain, no children, not in drain-life subtree. Deal damage to a creature and gain that much life = genuine drain. Add as exact override leaf. |

**evasion** _(coverage: under, confidence: high)_ — Root 'evasion' (4568) captures the gains-* family (gains-flying 314, unblockable 203, gains-menace 86, french-vanilla-walker 68, unique-evasion 51, daunt 40, skulk, high-flying, fake-flying, gains-landwalk chain, etc.). But the entire GRANT-evasion family nests under 'gives-evasion' (27), whose own parent is blank, so it sits ENTIRELY OUTSIDE the evasion subtree (verified: 0 gives-* tags in the evasion subtree). That means gives-flying(460), gives-menace(178), gives-unblockable(162), gives-fear(29), gives-intimidate(18), gives-shadow(13), gives-stalking(13), gives-horsemanship(10), gives-skulk(9), gives-landwalk(9) + its walk children, gives-daunt(8), gives-nimble(4), gives-fake-flying(3) all currently fall into 'Other' on the card display. Adding 'gives-evasion' as a root cleanly folds this grant-evasion family under Evasion. Minor note: gains-protection(42) is dual-tagged evasion+protection and gets pulled in here; defensible (protection-by-color is evasive) so not flagged as a wrong inclusion.

| Add | Kind | Pop | Why |
|---|---|---:|---|
| `gives-evasion` | root | 27 | VERIFIED: exists in catalog (own pop 27, parent blank). Heads the clean 'grant evasion to a creature' subtree (gives-flying 460, gives-menace 178, gives-unblockable 162, gives-fear 29, gives-intimidate 18, gives-shadow 13, gives-stalking 13, gives-horsemanship 10, gives-skulk 9, gives-landwalk 9 + forest/island/swamp/mountain/plains/townwalk children, gives-daunt 8, gives-nimble 4, gives-fake-flying 3). Confirmed 0 gives-* tags in the current evasion subtree, so all of it lands in Other. Population is the subtree sum (cards overlap, upper bound; direct-child sum runs ~1070). Add as a root. |

**mill** _(coverage: mixed, confidence: high)_ — Root 'mill' (0 own cards) expands correctly to mill-self(332), mill-opponent(192), mill-any(187), mill-exile(116), mill-each(60), grind(10), and the self-mill enablers surveil(289), mulch(115)/repeatable-mulch(52), ingest(36). surveil under mill is defensible (self-mill/graveyard-filling). Under-cover: 'synergy-mill'(91) is parent-blank and lands in Other despite being squarely mill-themed; add it. Minor OVER-cover: 'repeatable-maps'(6) rides into the mill subtree because it lists surveil as a parent, but Maps are artifact tokens, not mill; it is a wrong inclusion (low population).

| Add | Kind | Pop | Why |
|---|---|---:|---|
| `synergy-mill` | override | 91 | VERIFIED: exists in catalog (pop 91, parent blank, no children), confirmed not in the current mill subtree. Clearly mill-themed, currently in Other. No children -> add as exact override (behaves same as root since it heads nothing). |

**protection** _(coverage: under, confidence: high)_ — Root 'protection' captures a large clean subtree (protects-*, gives-hexproof/indestructible/ward/shroud/protection, gains-hexproof/shroud/protection, damage-prevention-creature/planeswalker); overrides add fog-selective, damage-prevention, phasing. No off-role drag-in. But many high-pop protection tags are ORPHANS (empty parent) and fall outside the subtree into 'Other', so grouping under-covers. Notably gains-indestructible is missing though gives-indestructible is present. The 'damage-prevention' override is matched exactly, so its hierarchy children fog (36) and damage-redirection (51) still fall out; converting it to a root would fold both in and make fog-selective redundant.

> regex note: Gap role. Regex matches hexproof/indestructible/ward keywords plus 'protection from'/'gains hexproof'/'gains indestructible'. False negatives: regeneration (no regenerate pattern) and generic damage-prevention/fog get no Protection role unless an otag override supplies it, so regenerates-self/other and damage-prevention-* are role gaps too, not just grouping gaps; shroud only via otag. False positives low: 'protection from' in reminder/flavor text or a printed 'ward' stat could trip it, but both are genuinely defensive.

| Add | Kind | Pop | Why |
|---|---|---:|---|
| `regenerates-self` | override | 178 | VERIFIED: catalog pop=178, orphan, not in subtree. Regeneration is a destruction-protection effect analogous to indestructible. |
| `damage-prevention-you` | override | 160 | VERIFIED: catalog pop=160, orphan (parent empty, NOT a child of the damage-prevention override), not in subtree. Prevents damage to the player = protection. Falls to Other. |
| `gains-indestructible` | override | 142 | VERIFIED: catalog pop=142, orphan (parent empty), absent from all subtrees. Self/gained form of the already-covered gives-indestructible (282, under protection); unambiguous protection. Currently lands in Other. |
| `damage-prevention-self` | override | 105 | VERIFIED: catalog pop=105, orphan, not in subtree. Self damage-prevention, protection-flavored. |
| `regenerates-other` | override | 99 | VERIFIED: catalog pop=99, orphan, not in subtree. Grants regeneration to other creatures = protection. |
| `pseudo-fog` | override | 75 | VERIFIED: catalog pop=75, orphan, not in subtree. Fog-variant damage prevention; consistent with the fog-selective override already in the map. |

**pump** _(coverage: under, confidence: high)_ — Intentionally partial: root 'combat-trick' (+ giant-growth, giant-growth-with-set-mechanic) covers only instant/sorcery single-target buffs; the rest is meant to come from the regex supplement. The two largest repeatable single-target self-pump families are orphans outside the subtree AND outside the regex, so they fall to Other and often get no Pump role at all.

> regex note: Gap role. Pattern '(target|equipped|enchanted) creature .{0,15} gets? +N/+N' correctly excludes 'creatures you control' (Anthem). False negatives: firebreathing/shade-pump ('this creature gets'), and any buff with >15 chars between the noun and the +N/+N. Note +N/+0 still matches (\d+ includes 0). False-positive risk low; aura/equipment reminder text matching the pattern is genuinely pump.

| Add | Kind | Pop | Why |
|---|---|---:|---|
| `shade-pump` | override | 288 | VERIFIED: catalog pop=288, orphan, not in combat-trick subtree. Repeatable {mana}: this creature gets +1/+1 (shade-style) single-target self-pump; regex ('this creature gets') misses it. |
| `firebreathing` | override | 285 | VERIFIED: catalog pop=285, orphan, not in subtree. Classic {R}: gets +1/+0 self-pump. Outside subtree and not matched by the target/equipped/enchanted regex. |

**ramp** _(coverage: under, confidence: high)_ — Roots 'ramp' + 'mana-producer' cover a strong, clean subtree (land-ramp, mana-dork, mana-rock, ritual, mana-increaser, extra-land, combat-ramp, moxen); overrides gives-mana-ability and repeatable-treasures patch the orphan/token-nested cases well. Sinks (mana-sink, bottomless-mana-sink) and cost-reduction (cheaper-than-mv, cost-reducer) are correctly kept OUT. One large orphan acceleration tag is missed, and combat-ramp drags in a pump tag.

> regex note: Not a gap role (otag-derived, no regex heuristic).

| Add | Kind | Pop | Why |
|---|---|---:|---|
| `adds-multiple-mana` | override | 520 | VERIFIED: catalog pop=520, orphan (parent empty), not in ramp subtree; effectively a leaf (only cycle-rav-bounceland, 10 cards, nests under it). Cards that add 2+ mana at once = acceleration; those tagged only this (not mana-rock/dork/ritual) currently fall to Other. |

**recursion** _(coverage: under, confidence: high)_ — Roots 'recursion' + 'reanimate' (reanimate is a subset of recursion) yield a very comprehensive, clean subtree: reanimate-*, regrowth-*, restock-*, reanimate-copy, gives-flashback/unearth/escape/encore/jump-start, etc. Graveyard-hate and graveyard-matters synergy tags are correctly kept out. The main gap is the intrinsic 'castable from your own graveyard' family, which sits under castable-from-nonhand rather than recursion.

> regex note: Not a gap role (otag-derived, no regex heuristic).

| Add | Kind | Pop | Why |
|---|---|---:|---|
| `castable-from-graveyard` | override | 403 | VERIFIED: catalog pop=403, parent=castable-from-nonhand (itself an orphan root, NOT under recursion), absent from recursion subtree. Self-castable from graveyard (flashback/escape/disturb) = self-recursion. Subtree only has the GIVES form (gives-castable-from-graveyard, 115, under recursion). Big miss into Other. |
| `temporary-reanimation` | override | 97 | VERIFIED: catalog pop=97, orphan, not in subtree. Reanimation until end of turn, unambiguous recursion. |
| `mass-reanimation` | override | 56 | VERIFIED: catalog pop=56, orphan, not in subtree. Mass reanimate effect, clearly recursion. |

**removal** _(coverage: good, confidence: high)_ — Root 'removal' captures a huge, comprehensive, clean subtree (spot-removal, removal-destroy/exile/bounce/sacrifice/tuck/land, sweeper, multi-removal, burn-*, fight, pacifism, banish, man-o-war, doom-blade). Overlaps with wipe (sweeper) and burn (burn-*) are intended and group correctly. Sacrifice-outlet, counterspell, self-bounce, exile-self/castable-from-exile value tags are correctly excluded. Small soft-removal edges are missed: the lockdown (Pacifism/Arrest-style) family is a SEPARATE root not under removal.

> regex note: Not a gap role (otag-derived, no regex heuristic). Note: proposed 'shrink' (107) was REJECTED as tangential/redundant — its lethal form is already captured inside removal via removal-toughness (603, under removal-creature), and shrink (parent empty, heads shrink+mass-shrink) would drag in non-lethal anti-pump combat tricks.

| Add | Kind | Pop | Why |
|---|---|---:|---|
| `swap-removal` | override | 130 | VERIFIED: catalog pop=130, orphan, not in subtree. Donate/exchange-control removal effects. |
| `mass-land-denial` | override | 111 | VERIFIED: catalog pop=111, orphan leaf (no children), not in subtree. Armageddon-style mass land destruction; removal-land covers only targeted, so this mass form is a genuine gap. Stax overlap noted but destruction of permanents is on-role. |
| `lockdown` | root | 2 | VERIFIED as clean root, but stated population 107 was wrong: catalog 'lockdown' has direct count 2 (107 is its child lockdown-creature). Heads a clean subtree lockdown-creature (107), lockdown-artifact (15), lockdown-land (10), lockdown-permanent (4), lockdown-nonland (2), lockdown-planeswalker (2), dehydration-with-set-mechanic (17) ~159 cards of Pacifism/Arrest neutralization = soft removal. Sibling pacifism is already IN removal while lockdown-* falls to Other. Population corrected to catalog value 2; add as a root. |

**sacrifice** _(coverage: under, confidence: high)_ — Root `sacrifice-outlet` is accurate and well-covered for the OUTLET axis: it cleanly captures sacrifice-outlet-creature (883), repeatable-sacrifice-outlet (576), the per-type outlets (artifact 303, land 238, enchantment 62, permanent 56, token 43), plus free-sacrifice-outlet, alternate-cost-sacrifice, devour, fling, emerge, plunder/bombard. No meaningful wrong inclusions (bombard/plunder/fling are genuine sac outlets that also burn/draw). The gap is the SELF-sacrifice / payoff axis, which is entirely outside the subtree and falls into Other: synergy-sacrifice (sacrifice-matters payoffs) and the whole sacrifice-self family. Whether to pull these in is a scope decision (outlets-only vs. full sacrifice theme).

| Add | Kind | Pop | Why |
|---|---|---:|---|
| `sacrifice-self` | override | 347 | DOWNGRADED root->override. VERIFIED pop 347, parentless, not covered. But it does NOT head a clean subtree: its children are martyr (535), egg (306), bombard-self (134) AND fetchland (17, dual-parented sacrifice-self,tutor-land-to-battlefield) which is a tutor/ramp effect already in the tutor subtree. Adding as root would drag ~1300 fodder cards plus the spurious fetchland. Add as EXACT override to group the 347 self-sacrificers into the sacrifice theme without contaminating with martyr/egg/fetchland (matches reviewer's own fallback). |
| `synergy-sacrifice` | override | 104 | VERIFIED: catalog pop 104, empty parent (lands in Other), not in any subtree. Cares-about-sacrificing (aristocrats payoffs); clean single leaf, safe exact override. |
| `synergy-sacrifice-self` | override | 15 | VERIFIED: pop 15, parentless, not covered. Cares-about-self-sacrifice payoffs; minor completeness, clean override. |

**stax** _(coverage: under, confidence: high)_ — Root `tax` is intentionally a partial supplement (module docs say it captures only the tax slice): rhystic (259), toll (179), cast-tax (17), opaline-effect (16) ~= 470 cards, all accurate. But stax as an archetype is much broader and the rest of the denial/lock space sits outside the subtree and drops into Other for grouping. The regex heuristic (STAX_CANT/STAX_EACH) supplies role MEMBERSHIP from oracle text but does not group these tags. Several large, clearly-stax subtrees are un-mapped.

> regex note: STAX_CANT `(opponents?|players?) can'?t` false-positives on combat/evasion text like 'creatures your opponents control can't block' (evasion, not stax) and on wincons like 'you can't lose the game' when phrased with players. STAX_EACH `(each|all) (player|opponent).{0,30}(sacrifice|discard|lose|pay)` false-positives on 'each player loses the game' (a wincon, matches 'lose'). False NEGATIVES: it misses tag-based stax with no matching text pattern, e.g. lockdown/'doesn't untap', pillowfort 'can't attack you unless', and cost taxes ('spells cost {1} more') that only the otag tax root catches, so those still miss grouping when the tax root is absent from a card.

| Add | Kind | Pop | Why |
|---|---|---:|---|
| `mass-land-denial` | override | 111 | VERIFIED: pop 111, parentless, not covered. Armageddon/Ravages hard-stax; clean override. |
| `prevent-activation` | override | 67 | VERIFIED: pop 67, parentless, not covered. Cursed Totem / Damping Sphere ability denial; clean override. |
| `hatebear` | override | 65 | VERIFIED: pop 65, parent is `hate` (pop 0). Confirmed `hate` heads 130+ mostly non-stax leaves (hate-attacker, hate-blocker, hate-graveyard, hate-flying, etc.), so an EXACT override is required to grab the archetypal stax hatebear without dragging the hate-* family. Correct containment. |
| `prevent-cast` | override | 52 | VERIFIED: pop 52, parentless, not covered. Silence / can't-cast denial; clean override. |
| `pillowfort` | root | 19 | VERIFIED clean root: pop 19, heads tax-attack (40). Confirmed tax-attack is NOT already in the `tax` subtree (which is only rhystic/toll/cast-tax/opaline-effect), so no redundancy. ~59 cards attack-tax denial. Clean stax subtree. |
| `lockdown` | root | 2 | VERIFIED clean root: pop 2, heads lockdown-creature (107), lockdown-artifact (15), lockdown-land (10), lockdown-permanent (4), lockdown-nonland (2), lockdown-planeswalker (2), plus dehydration-with-set-mechanic (17, a genuine tap-down aura). ~157 cards of classic 'stays tapped / doesn't untap' stax. Not in any subtree. Strong clean add. |

**tokens** _(coverage: mixed, confidence: high)_ — Membership is strong: all_parts token-component detection plus roots repeatable-token-generator (heading repeatable-creature-tokens 1441, artifact 173, noncreature 35, enchantment 23, and the treasures/clues/food/blood/powerstone families) and synergy-token (synergy-token-creature 110, tokenfall 20). No wrong inclusions (treasure/food overlap with ramp/lifegain is intentional). The gap is grouping: a large set of token DESCRIPTOR tags are parentless and fall into Other on the card detail even though the card is already a tokens card.

| Add | Kind | Pop | Why |
|---|---|---:|---|
| `out-of-color-token` | override | 415 | VERIFIED: pop 415, parentless, not covered. Highest-population unmapped token-descriptor leaf; clean override. |
| `unique-token` | override | 381 | VERIFIED: pop 381, parentless, not covered. Token-descriptor leaf; clean override. |
| `temporary-token` | override | 211 | VERIFIED: pop 211, parentless, not covered. 'Exile/sac at end of turn' tokens; clean override. |
| `donate-token` | override | 184 | VERIFIED: pop 184, parentless, not covered. Gives-tokens (Forbidden Orchard); genuinely token-tagged, weakest of the set (some are opponent-facing) but taxonomically a token leaf. Clean override. |
| `unprinted-token` | override | 131 | VERIFIED: pop 131, parentless, not covered. Token-descriptor leaf; clean override. |
| `named-token` | override | 81 | VERIFIED: pop 81, parentless, not covered. Token-descriptor leaf; clean override. |
| `token-increaser` | root | 14 | VERIFIED clean root: pop 14, heads token-doubler (17). Doubling Season / Parallel Lives ~= 31 high-value token payoffs. Not covered. Clean add. |
| `token-versions-of-cards` | root | 0 | VERIFIED clean root: own pop 0, heads creates-token-of-a-card (72), has-identical-token (39), token-version-of-a-card (16) = ~127 'token copy of a card' cards. Not in any subtree. Clean add. |

**tutor** _(coverage: good, confidence: high)_ — Root `tutor` is the most comprehensive of the assigned roles: its subtree spans the full tutor-* taxonomy (tutor-to-hand 557, tutor-land-basic 339, tutor-land-to-battlefield 275, tutor-card 111, tutor-creature 99, tutor-mv 83, tutors-by-name 82, fetchland 17 and the fetchland cycles, gamble, pwdeck-tutor, and dozens of type-specific tutors). Coverage is essentially complete and accurate. Mild over-cover: it pulls in all land fetches (tutor-land-basic 339, fetchland cycles) which double as ramp/fixing, but these are genuine library searches and taxonomically correct tutors, so not true wrong inclusions. Only a couple of tiny orphaned tutor tags sit outside.

| Add | Kind | Pop | Why |
|---|---|---:|---|
| `booster-tutor` | override | 14 | VERIFIED: pop 14, parent is `wish` (outside tutor subtree, heads only booster-tutor). Ring of Ma'ruf / wish-style off-game tutors; not covered. Clean exact override. |
| `synergy-tutor` | override | 5 | VERIFIED: pop 5, parentless, not covered. Tutor-matters synergy; minor completeness. |

**untap** _(coverage: under, confidence: high)_ — Root `untapper` covers the core well: untapper-creature (443), untaps-self (173), untapper-land (99), untapper-permanent (76), twiddle (62), untapper-artifact (48), untapper-nonland (12), etb-untapper (8). One notable false-inclusion risk: twiddle (62) also parents under `tapper`, so some twiddle cards are pure tappers (a stax effect), giving a mild mixed grouping. The bigger issue is under-coverage: the largest untap-adjacent tag, extra-untap (105), plus two orphaned untapper leaves, are outside the subtree.

| Add | Kind | Pop | Why |
|---|---|---:|---|
| `extra-untap` | override | 105 | VERIFIED: pop 105, parentless, not covered. Extra untap steps / untap-additional-permanents (Seedborn Muse, Nature's Will). Largest unmapped untap tag; clean override. |
| `ritual-untap` | override | 20 | VERIFIED: pop 20, parentless, not covered. Untap-a-permanent-for-mana rituals; clean override. |
| `untapper-planeswalker` | override | 1 | VERIFIED: pop 1, empty parent (orphaned, NOT nested under untapper root), not covered. Tiny but clean taxonomy fix. |
| `untapper-equipment` | override | 1 | VERIFIED: pop 1, empty parent (orphaned, escapes untapper subtree), not covered. Clean exact override. |

**wipe** _(coverage: good, confidence: high)_ — The wipe role maps to a single root, `sweeper`, which is the canonical Scryfall board-wipe tag family. Its subtree is `sweeper` (740) plus its child `sweeper-one-sided` (228), both genuine mass-removal effects, so there is no over-cover: `sweeper`'s parent is `removal`, but only the sweeper branch is pulled, not sibling removal tags. Adjacent sweeper-named tags correctly live under other roles: `sweeper-graveyard` (83) nests under `hate-graveyard` (graveyard_hate), `counterspell-sweeper` (8) under `counterspell`, and `expansion-sweeper` (5) is deprecated. Coverage is essentially complete. The only notable population outside the subtree is `mass-land-denial` (111), a root-level tag (no parent, no children) covering Armageddon-style mass land destruction. Whether it belongs to `wipe` is a genuine judgment call, it is a board reset of lands but is also commonly treated as a stax/prison piece, so it currently falls into 'Other'. I flag it as an optional root candidate rather than a clear miss.

> regex note: role_subtrees.tsv format is root\tdepth\tnode\tpopulation; catalog.tsv is slug\tpopulation\tparent\tdisplay. Current wipe subtree = sweeper(740)+sweeper-one-sided(228). mass-land-denial has empty parent field, confirming root-level leaf.

| Add | Kind | Pop | Why |
|---|---|---:|---|
| `mass-land-denial` | root | 111 | Verified: catalog confirms population 111, empty parent column (root-level leaf, no children), and it is absent from the current sweeper subtree. Genuine Armageddon/Ruination-style mass land destruction, a board-reset effect currently landing in 'Other'. Borderline wipe vs stax as flagged; a clean standalone root worth adding only if mass land destruction should surface as a wipe. |

**counters** _(coverage: mixed, confidence: medium)_ — Mixed. The pp roots (gains 1738, gives 1322, repeatable 1670) plus counters-matter give strong +1/+1 coverage, but the role both over-reaches (cosmetic counter-fuel) and under-reaches (-1/-1 counters and core manipulation). Verified 5 genuine gaps outside the current subtree, all heading clean counter subtrees or standalone leaves. One proposed wrong inclusion (counter-fuel-energy) was REJECTED on verification: it is itself a configured root and an intended energy overlap, not an accidental pull. counter-fuel-aesthetic remains flagged as cosmetic/reminder-counter noise dragged in via counters-matter.

| Add | Kind | Pop | Why |
|---|---|---:|---|
| `gives-mm-counters` | root | 183 | VERIFIED: catalog pop 183, not in subtree. Heads clean -1/-1 subtree: mm-counter-cost (40), gives-persist (7), gives-wither-noncreature (1). counters-matter already pulls mm-counters-matter/removes-mm-counters, so excluding these mm givers is inconsistent. |
| `move-counters` | override | 67 | VERIFIED: catalog pop 67, root-level tag (empty parent), no children, not in subtree. Counter-manipulation payoff; add as exact override leaf. |
| `gains-mm-counters` | root | 50 | VERIFIED: catalog pop 50, not in subtree. Heads persist (26, parent includes gains-mm-counters). Same mm-counter inconsistency as gives-mm-counters. |
| `pseudo-proliferate` | override | 39 | VERIFIED: catalog pop 39, root-level tag, no children, not in subtree. Proliferate is the signature counters mechanic; add as exact override leaf. |
| `counter-increaser` | root | 19 | VERIFIED: catalog pop 19, not in subtree. Heads counter-doubler (63, parent=counter-increaser). Doubling counters is a top +1/+1 payoff, entirely outside the current subtree. |

**graveyard_hate** _(coverage: under, confidence: medium)_ — Root 'hate-graveyard'(299) captures sweeper-graveyard(83) and graveyard-seal(53); clean, no wrong inclusions. It correctly EXCLUDES the large graveyard-FUEL/synergy family (graveyard-fuel 181, cards-in-graveyard-matter 174, castable-from-graveyard 403, undergrowth 116, threshold 109, delve, etc.) which is graveyard payoff, not hate; adding a broad 'graveyard' root would be a major over-cover, so leaving it out is correct. One clear gap: 'hate-graveyard-cast'(10) hangs off 'hate' (not 'hate-graveyard'), so cards that stop casting/recurring from graveyards (Grafdigger's Cage style) fall into Other. Add it as an exact override.

> regex note: Regex is r"exile.{0,30}(graveyard|all cards from)". FALSE POSITIVES: self-graveyard exile as a COST is graveyard fuel, not hate, yet matches, e.g. Escape/Delve/Flashback text 'Exile four other cards from your graveyard' and 'Exile two cards from your graveyard: <ability>'. These get mis-tagged GraveyardHate. FALSE NEGATIVES: hate wordings without 'exile', e.g. replacement/prevention effects ('cards can't be cast from graveyards', 'If a card would be put into a graveyard... instead', old 'remove from the game') slip through; the hate-graveyard-cast(10) override above backstops part of this. Consider anchoring the regex to opponent/all-graveyard phrasing to shed the self-fuel false positives.

| Add | Kind | Pop | Why |
|---|---|---:|---|
| `hate-graveyard-cast` | override | 10 | VERIFIED: exists in catalog (pop 10, parent 'hate' not 'hate-graveyard', so outside the subtree and lands in Other). No children in catalog. Prevents casting/using cards from graveyards (Grafdigger's Cage style). Single leaf -> exact override. |

**lifegain** _(coverage: under, confidence: medium)_ — Root 'lifegain'(878) expands broadly and well: repeatable-lifegain(1382), gives-lifelink(230), gains-lifelink(104), old-lifelink(28), gives-lifelink-noncreature(9), gainland(7)+gainland cycles, repeatable-food(60), blood-artist-ability(39), soul-warden-ability(37), gives-extort(1). It also pulls drain-life(365) and drain-creature(75) via the hierarchy; that overlaps the 'drain' role but is intended (drain gains life), so not a wrong inclusion. Under-cover on the payoff side: 'lifegain-matters'(125) is parent-blank and heads a clean payoff subtree (infusion 42, pridemate 42, lifegain-to-damage 19) that currently lands in Other, along with lifegain-increaser(14). opponent-lifegain(57) and synergy-lifelink(22)/synergy-food(88) are more ambiguous (drawback/synergy flavor) and I did not propose them. The two proposals below are the lifegain-themed tags most worth grouping under this role.

| Add | Kind | Pop | Why |
|---|---|---:|---|
| `lifegain-matters` | root | 125 | VERIFIED: exists in catalog (pop 125, parent blank). Heads the lifegain-payoff subtree (infusion 42, pridemate 42, lifegain-to-damage 19), confirmed none in the current lifegain subtree. All in Other. Payoff-side, so medium confidence it belongs to the lifegain role vs a separate synergy grouping, but for card-detail grouping it reads as Lifegain. Add as a root. |
| `lifegain-increaser` | override | 14 | VERIFIED: exists in catalog (pop 14, parent blank, no children). Effects that multiply/increase life gained (life doublers). Outside the lifegain subtree, lands in Other. Single leaf -> exact override. |

### B3. Roles judged already-good (no change)

`anthem`, `blink`, `card_advantage`, `counterspell`, `draw`, `energy`

---

## Coverage gap — unmapped archetypes

0 of the 117 `DeckTag` variants seed **nothing** today. Some are intentional
(e.g. `Chaos`); others may be genuine gaps worth authoring later. Not scored by this sweep:



---

## Suggested application order

1. **A1 removals** — cheapest, highest-impact correctness fix (kills format-wide over-selection like Voltron/Flying `evasion`, Spellslinger `single-target-instant-sorcery`).
2. **B1 wrong inclusions** — small, fixes visibly wrong card-detail grouping.
3. **High-confidence A2 adds + B2 root adds** — the strongest completeness wins.
4. **Medium/low-confidence adds** — eyeball individually before applying.
5. (Later) decide which unmapped archetypes deserve seeds.

Then add guard tests: assert every `CATEGORY_ROOTS` root and every deck-tag seed slug exists in the catalog (today only `ROLE_TAG_OVERRIDES` is checked, at runtime).
