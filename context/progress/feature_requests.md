# Feature Requests

Raw, user-sourced feature intake — distinct from `backlog.md` (curated/committed work).
Items here are candidates to weight and promote into the backlog once prioritized.

**Weighting legend**
- **Impact**: High / Med / Low — pull toward retention, the core swipe loop, or conversion.
- **Effort**: S (hours) / M (a day or two) / L (multi-day or new subsystem).
- **Priority**: P1 (do next) / P2 (soon) / P3 (someday) — my suggested call; adjust freely.

First source: **Reddit r/mtg launch thread, 2026-06-28** (45K views, ~300 signups). Add later sources as new sections.

Second source: **App Store review, 2026-07-02** (5-star "Awesome app" by Mr.K): "Hoping that new features include saving what cards you've been through between sessions." Maps directly to #11 (swipe memory, built that same day, ships in 1.2.3) — skips and removals persist per deck; adds and maybes were already durable (deck/maybeboard). Independent validation of the P1 call.

---

## Shipped so far

Marked ✅ in the tables below; numbers kept stable (plans/commits reference them).

- **#8 Card name + oracle/stats detail** — name always shown; a util-bar eye
  button opens a dialog with oracle text (mana/tap/symbol glyphs), type + rarity
  + keyword chips, and P/T or loyalty. Reuses the expanded card-row detail markup.
- **#5 Land count target** — land-target stepper in the deck form + a one-time
  toast when the mainboard crosses the target (1.2.0).
- **#19 Land-target auto-stop** — lands drop out of the swipe pool once the target
  is met (`ensure_lands_excluded`, 1.2.0).
- **#17 Per-field validation errors** — inline red outline + message under each
  field (register / change-email / change-password / forgot-password).
- **#15 Browse all tags up front** — tag-picker hint dialog lists every tag with
  a `DeckTag::description()`.
- **#10 Price threshold filter** — price min/max range in the filter sheet
  (`filter/price.rs`).
- **#21 Clone-deck polish** — one-line hint + navigates to the cloned deck on
  Save (`clone_deck_dialog.rs`).
- **#14 More theme tags** — covered by ongoing tag expansion (85→117 in 1.2.0);
  add specific tags as they surface.

---

## Swipe experience (core loop polish)

| # | Feature | Impact | Effort | Priority | Notes |
|---|---------|--------|--------|----------|-------|
| 1 | Live drag indicators — screen edge glows red (left) / green (right), text hint past threshold; extend to undo + maybe | High | S | **P1** (parked) | Cheapest high-delight win. Cue works on `feat/qol-drag-indicators`; visual style undecided, parked pending complaints. Fixes "I kept forgetting which way and had to undo." |
| 2 | "Just inspire me" mode — swipe with no commander/tags set, pure discovery | High | M | P2 | Most on-brand with the "Tinder" framing. |
| 3 | Head-to-head "which is better" — pick 1 of 2 same-category cards (two ramp pieces, etc.) | Med | M | P3 | A distinct mode, not a replacement for the one-at-a-time flow. |

## Deck-building intelligence (differentiator)

| # | Feature | Impact | Effort | Priority | Notes |
|---|---------|--------|--------|----------|-------|
| 4 | Auto land base from color-pip ratio as you build | High | M | **P1** | Used on every deck. High stickiness. |
| 5 | Land count target / cap so you don't over/under-run | Med | S | ✅ Shipped | Land-target stepper in deck form + crossing toast (1.2.0). Pairs with #4/#19. |
| 6 | Mana-value-aware suggestion weighting (surface lower MV as curve fills) | High | M | P2 | Makes the recommender feel smart. Manual MV-range filter already exists as a stopgap. |
| 7 | Embeddings-based auto-build / decklist analysis (assemble ~80% of a deck, swipe the rest) | High | L | P3 | North-star; aligns with the synergy/recommender roadmap. |
| 20 | Companion support — recognize the 10 companions and let the deck declare one, since the companion dictates deck composition (constraints filter the swipe pool) | Med | M | P3 | User feedback (2026-06-29). Low priority but **fully programmable**: companion set is tiny and WotC has effectively stopped adding new ones, so it's a fixed, hard-codable rule set. |

## Card data & display

| # | Feature | Impact | Effort | Priority | Notes |
|---|---------|--------|--------|----------|-------|
| 8 | Always show card name + a detail view (esp. foreign/alt-art printings) | High | S–M | ✅ Shipped | Name always shown + util-bar eye → oracle/stats dialog. Kills the unidentifiable-card complaint. |
| 9 | Prefer original / English printing in the swipe stack | Med | M | P2 | Overlaps with #8; printing-selection logic. |
| 18 | Printing/art display settings — toggle: only original printing, most-recent printing, exclude Secret Lair art | Med | M | P2 | User-facing superset of #9; printing-selection preferences in settings. |

## Filtering & budget

| # | Feature | Impact | Effort | Priority | Notes |
|---|---------|--------|--------|----------|-------|
| 10 | Price threshold filter (hard budget cap, EUR/USD) | Med | M | ✅ Shipped | Price min/max range in the filter sheet (`filter/price.rs`). |
| 19 | Land-target auto-stop — when the land count target is hit, the land filter should **stop serving lands automatically** rather than continuing to surface them | High | S | ✅ Shipped | Lands excluded from the swipe pool once the target is met (1.2.0). User feedback (2026-06-29): "absolutely genius." |

## Persistence & in-build visibility

| # | Feature | Impact | Effort | Priority | Notes |
|---|---------|--------|--------|----------|-------|
| 11 | Persist skipped cards per deck across sessions | High | M | ✅ Built (2026-07-02, on main awaiting release) | Skips **and deliberate removals** persist server-side (`deck_card_suppressions`); the deck-aware search stops serving them, with a "Clear skips" button in the deck view's More sheet. Design + as-built notes: [`../plans/swipe_memory.md`](../plans/swipe_memory.md). Requested a second time via App Store review 2026-07-02 (see sources above). Mark shipped once the release is out. |
| 12 | Easy access to deck view / card count / mana curve while building | Med | S–M | P2 | Stats should be one tap away mid-swipe. |

## Tags

| # | Feature | Impact | Effort | Priority | Notes |
|---|---------|--------|--------|----------|-------|
| 13 | Add "typal" tag (community moved off "tribal") | Low | S | ❌ Won't do | Sticking with "tribal" (owner call, 2026-07-01). |
| 14 | More specific theme tags (e.g. Elves) | Low | S | ✅ Closed | Ongoing tag expansion (85→117 in 1.2.0) covers this; add specific tags as they come up. |
| 15 | Browse all tags up front at deck create/edit | Low | S | ✅ Shipped | Tag-picker hint dialog lists every tag with a description. Related to #2. |

## Import sources

| # | Feature | Impact | Effort | Priority | Notes |
|---|---------|--------|--------|----------|-------|
| 16 | CubeCobra ID import + cube-based suggestions | Med | M–L | P3 | Extends existing link-import; niche but loyal cube crowd. |

## Onboarding / UX nits

| # | Feature | Impact | Effort | Priority | Notes |
|---|---------|--------|--------|----------|-------|
| 17 | Password rule errors placed under the password field (not floating up top) | Low | S | ✅ Shipped | Per-field inline validation (red outline + message) across auth forms. |
| 21 | Clone-deck UX polish — (a) trim the hint text down to one line, e.g. "Make an exact copy of your deck"; (b) on Save, navigate straight to the newly cloned deck | Low | S | ✅ Shipped | One-line hint + navigates to the cloned deck on Save (`clone_deck_dialog.rs`). |

---

## Not actionable (logged for completeness)
- **Rename to "Commandr"** — joke, no action.
- **Regional/EU availability** — ops, not a feature; tracked via the DSA trader-verification process.

## Suggested next to ship
(#5, #8, #15, #17, #19 shipped; see "Shipped so far" above.)
1. **#4 Auto land base** — high stickiness, every-deck value; unblocks #6.
2. **#12 Deck stats mid-build** — planned (qol bundle D): util-bar button → stats bottom sheet reusing the deck charts.
3. **#1 Live drag indicators** — parked on `feat/qol-drag-indicators` pending a visual-style call; revive if it resurfaces.
