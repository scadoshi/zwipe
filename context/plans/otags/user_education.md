# User education — Deck tags, Card roles & Oracle tags

**Status: PLAN (2026-07-12).** The otag arc (Phases 1-5) shipped a lot of new
user-facing surface — deck-level oracle-tag selection, the archetype→otag seed,
the card-face role→tag drill-down, the otag filter, and otag-aware serving. This
doc plans how we teach it: the mental model, the in-app hints (existing + new),
a new persistent "?" help affordance, and the zite guides.

Owner-set direction (2026-07-12): **explain the model** (don't hide the
machinery), **lead with the suggestion-quality payoff**, plan-doc first.

**Governing decision (2026-07-12): on-demand over auto-fire.** The otag arc adds
a lot of new surface to already-busy screens (the edit form, the card profile).
Auto-firing a wave of new one-time hint dialogs there would be hint fatigue and
risks annoying users. So **we do not add new auto-firing hints for this** —
instead we surface the new field and park a persistent "?" next to it, and the
user pulls the explanation themselves when they're curious. The "?" `InfoButton`
(§3b) is the primary vehicle; auto-hints (§3a) are only *lightly* refreshed in
place, never multiplied.

## One sentence

Give players one consistent mental model — *Deck tags seed Oracle tags, Oracle
tags sharpen suggestions, Card roles are the read-side view* — delivered through
onboarding hints, on-demand "?" help buttons, and a small set of guides.

---

## 1. The mental model (the spine)

Every hint and guide reuses this exact framing and vocabulary. Three terms, one
flow:

```
 Deck tags ──seed──▶ Oracle tags ──sharpen──▶ suggested cards
 (you pick an        (granular, functional;    (Phase 4 serving)
  archetype)          also directly editable)
                          │
                          ▼ (derived, read-only)
                      Card roles  ── shown on each card as chips
                                     (coarse buckets over oracle tags)
```

- **Deck tags** — *you pick* the archetype(s) your deck is built around
  (Aristocrats, Voltron, Spellslinger…). Picking one **auto-selects** the Oracle
  tags that define it. The easy on-ramp.
- **Oracle tags** — the granular, community-maintained functional tags
  (`spot-removal`, `ramp`, `sacrifice-outlet`…). Seeded by deck tags, but power
  users can add/remove them directly. **They sharpen which cards we suggest.**
- **Card roles** — the coarse buckets (Removal, Ramp, Card advantage…) shown as
  chips on each card; tap a role to reveal the specific oracle tags under it.
  **Read-side only — you never *pick* card roles.** They're computed from the
  card's oracle tags.

The one-liner we lead with everywhere: **"Tell us what your deck does, and we'll
suggest cards that fit."**

## 2. Copy principles

- **Sentence case** for concept names: **"Deck tags", "Oracle tags", "Card
  roles"** (matches the existing screen titles + zwiper button convention).
  Never "Deck Tags" / "Oracle Tags" in mobile copy.
- **No em dashes** in zwiper/zite user-facing copy (recast with comma/colon/
  period). Code comments/docs here are exempt.
- **Lead with the benefit**, then the mechanism: "Sharpen your suggestions —
  pick the tags that describe your deck," not "Oracle tags are functional tags."
- **Hints stay terse**: one line per bullet, action-first. Hints teach the
  *flow*; the "?" buttons and guides carry the *concept*.
- **"Zwipe" capitalized**; sentence case elsewhere (2026-05 casing migration).

---

## 3. Two help patterns (keep them distinct)

**Primary = 3b (the "?" buttons). 3a is not extended for the otag surface** —
per the governing decision, we do not auto-fire new hints here. 3a entries in §5
are in-place content refreshes of dialogs that *already* fire, not new dialogs.

### 3a. One-time onboarding hints (existing pattern)
`use_one_time_hint(HINT_*)` + `HintDialog` — auto-fires once per account on first
encounter, reopenable from the screen header "?". Teaches the *flow* of a screen.
Already exists for most screens. For the otag arc we only lightly refresh copy in
dialogs that already fire (§5); we add **no new auto-firing hints**.

### 3b. Persistent "?" help buttons (NEW — the primary vehicle here)
A small inline **`InfoButton`** next to a field/section **label** that opens a
concise concept explainer on demand. Unlike onboarding hints, it is **not
one-time** — it's always there, anywhere the concept appears, so a user can dig
into "what is this?" without opening the picker or having seen an auto-hint.

**Why new:** today the "Deck tags" / "Oracle tags" explanations only live behind
the picker-overlay header "?", i.e. you must *open the picker* to learn what it
is. The edit-screen fields and the card profile show these concepts with zero
inline explanation.

**Component spec (`zwipe-components` or zwiper components):**
- `InfoButton { label_for: &str, dialog: Element }` (or a `topic` enum keyed to
  shared copy) — renders a faded, tap-target-sized "?" glyph inline after a
  label; opens the shared `HintDialog` with the concept explainer.
- Reuses the existing "?" glyph styling + `HintDialog` shell (already in
  `ScreenHeader`) — no new visual grammar, no glow (per owner aesthetic).
- Copy is centralized (one source per concept) so the edit screen, card profile,
  deck view, and filter all show identical text. Single source of truth = fewer
  drifts.

**Placement map:**
| Surface | Concept(s) | Affordance |
|---|---|---|
| Edit/create screen — Tags section (`deck_fields.rs`) | Deck tags, Oracle tags | "?" next to each field label |
| Card profile / expanded row + swipe eyeball (`CardRoleChips` / `CardRulesDialog`) | Card roles, Oracle tags | "?" next to the "Card roles" label |
| Deck view — Tags section (`deck_tags_section.rs`) | Deck tags, Oracle tags | "?" on the section (or per row) |
| Filter sheet — Oracle tags section | Oracle tags (filter) | "?" next to the filter section title |

The concept explainers behind these buttons are the canonical §1 copy, trimmed
to the surface (e.g. the card-profile "?" leads with the read-side framing).

---

## 4. Guide plan (zite `guides/content.rs`)

Existing relevant guides: `deck-tags` ("Tag decks by archetype"), `filtering`,
`synergy`, `deck-stats`. No guide yet for Oracle tags or Card roles.

**New guides:**
- **`oracle-tags`** — "Sharpen suggestions with oracle tags." What they are, how
  to select them on a deck, how deck tags seed them, and (lead) how they improve
  the cards Zwipe suggests. Cross-links deck-tags + synergy.
- **`card-roles`** — "Read a card at a glance." The role chips, the tap-to-expand
  role→oracle-tag drill-down, and the "Role distribution" chart. Read-side framing.

**Keystone guide (recommended):**
- **`tags-roles-and-oracle-tags`** — "Deck tags, card roles & oracle tags: how
  they fit." The §1 diagram in prose, the one place that disambiguates all three.
  Every other guide + the "?" explainers can point here for the full picture.

**Updates to existing guides:**
- `deck-tags` — add: picking an archetype now **seeds oracle tags**, which feed
  suggestions. Link to `oracle-tags`.
- `filtering` — add the **Oracle tags** filter (curated set + search).
- `synergy` — note that **selected oracle tags now contribute to ordering**
  alongside commander synergy (Phase 4).
- `deck-stats` — reflect the Profile/Budget/Tags reorg (card count → Profile,
  price/lands → Budget) and the **Role distribution** chart.

---

## 5. Hint audit (mobile, `zwiper` `HintDialog`)

| Screen (file) | Title | Action |
|---|---|---|
| home | Welcome to Zwipe | keep |
| profile | Your profile | keep |
| deck/list | Your decks | keep |
| deck/view | Welcome to your deck | **update** — Tags section + Budget reorg; note tags/roles live here now |
| deck/card/add | Swipe to build | **update (light)** — suggestions reflect your deck tags/oracle tags |
| deck/card/remove | Swipe to trim | keep |
| deck/card/view | Browsing your deck | **update** — the card-face Card roles chips + role→tag drill-down |
| deck/card/view | Deck MVPs | keep |
| card/filter | Filters | **update** — new Oracle tags filter |
| components/tag_select | Deck tags | **update** — the §1 model: archetype seeds oracle tags → suggestions |
| components/oracle_tag_select | Oracle tags | **update** — granular + advanced; sharpen suggestions; seeded by deck tags |
| components/deck_fields | Building a deck | **update** — Profile/Budget/Tags structure + tags/roles overview |
| components/format_select | Format | keep |
| components/swipe_select | Zwipe select | keep |
| deck/export, deck/import | Exporting / Importing | keep |

**No new auto-firing hints.** Every "update" above is an *in-place copy refresh*
of a dialog that already fires — it adds no new interruption. The Card roles
drill-down and the deck-level Oracle tags field are introduced via their inline
"?" buttons (§3b), not a new auto-hint, so a user opening the busy edit form or
card profile isn't hit with a fresh dialog. Where an existing hint is already
dense, prefer trimming a bullet and leaning on the "?" over adding one.

---

## 6. Sequencing

1. **This doc** — reviewed/approved.
2. **Canonical copy** — write the concept explainers once (Deck tags, Oracle
   tags, Card roles) as the single source for both the "?" buttons and the hint
   copy refreshes.
3. **`InfoButton` component** + wire the four placements (§3b). *The primary
   deliverable* — this is how users discover the new concepts.
4. **Hint copy refreshes** (§5) — in-place edits to already-firing dialogs; no
   new auto-hints. Ship with the next mobile release alongside 3.
5. **Guides** (§4) — new `oracle-tags`, `card-roles`, keystone; then the updates.
   Zite ships independently of the mobile release, so guides can land first.

Mobile 3-4 gate a client release; guides (2, 4/zite) can go anytime. **Step 3 is
the star**; step 4 is deliberately minimal to avoid hint fatigue.
