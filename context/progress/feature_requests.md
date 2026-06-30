# Feature Requests

Raw, user-sourced feature intake — distinct from `backlog.md` (curated/committed work).
Items here are candidates to weight and promote into the backlog once prioritized.

**Weighting legend**
- **Impact**: High / Med / Low — pull toward retention, the core swipe loop, or conversion.
- **Effort**: S (hours) / M (a day or two) / L (multi-day or new subsystem).
- **Priority**: P1 (do next) / P2 (soon) / P3 (someday) — my suggested call; adjust freely.

First source: **Reddit r/mtg launch thread, 2026-06-28** (45K views, ~300 signups). Add later sources as new sections.

---

## Swipe experience (core loop polish)

| # | Feature | Impact | Effort | Priority | Notes |
|---|---------|--------|--------|----------|-------|
| 1 | Live drag indicators — screen edge glows red (left) / green (right), text hint past threshold; extend to undo + maybe | High | S | **P1** | Cheapest high-delight win. Directly fixes "I kept forgetting which way and had to undo." |
| 2 | "Just inspire me" mode — swipe with no commander/tags set, pure discovery | High | M | P2 | Most on-brand with the "Tinder" framing. |
| 3 | Head-to-head "which is better" — pick 1 of 2 same-category cards (two ramp pieces, etc.) | Med | M | P3 | A distinct mode, not a replacement for the one-at-a-time flow. |

## Deck-building intelligence (differentiator)

| # | Feature | Impact | Effort | Priority | Notes |
|---|---------|--------|--------|----------|-------|
| 4 | Auto land base from color-pip ratio as you build | High | M | **P1** | Used on every deck. High stickiness. |
| 5 | Land count target / cap so you don't over/under-run | Med | S | P2 | Pairs naturally with #4. |
| 6 | Mana-value-aware suggestion weighting (surface lower MV as curve fills) | High | M | P2 | Makes the recommender feel smart. Manual MV-range filter already exists as a stopgap. |
| 7 | Embeddings-based auto-build / decklist analysis (assemble ~80% of a deck, swipe the rest) | High | L | P3 | North-star; aligns with the synergy/recommender roadmap. |
| 20 | Companion support — recognize the 10 companions and let the deck declare one, since the companion dictates deck composition (constraints filter the swipe pool) | Med | M | P3 | User feedback (2026-06-29). Low priority but **fully programmable**: companion set is tiny and WotC has effectively stopped adding new ones, so it's a fixed, hard-codable rule set. |

## Card data & display

| # | Feature | Impact | Effort | Priority | Notes |
|---|---------|--------|--------|----------|-------|
| 8 | Always show card name + a detail view (esp. foreign/alt-art printings) | High | S–M | **P1** | Kills two complaints at once (unidentifiable cards + "bad images"). |
| 9 | Prefer original / English printing in the swipe stack | Med | M | P2 | Overlaps with #8; printing-selection logic. |
| 18 | Printing/art display settings — toggle: only original printing, most-recent printing, exclude Secret Lair art | Med | M | P2 | User-facing superset of #9; printing-selection preferences in settings. |

## Filtering & budget

| # | Feature | Impact | Effort | Priority | Notes |
|---|---------|--------|--------|----------|-------|
| 10 | Price threshold filter (hard budget cap, EUR/USD) | Med | M | P2 | Price *sort* + MV-range filter already exist; this is the missing budget piece. |
| 19 | Land-target auto-stop — when the land count target is hit, the land filter should **stop serving lands automatically** rather than continuing to surface them | High | S | **P1** | User feedback (2026-06-29): "absolutely genius." Pairs directly with #5; the target already exists, this just gates the filter off once reached. |

## Persistence & in-build visibility

| # | Feature | Impact | Effort | Priority | Notes |
|---|---------|--------|--------|----------|-------|
| 11 | Persist skipped cards per deck across sessions | High | M | P2 | Closing the app currently resets the skip pile — real gap in the core loop. |
| 12 | Easy access to deck view / card count / mana curve while building | Med | S–M | P2 | Stats should be one tap away mid-swipe. |

## Tags

| # | Feature | Impact | Effort | Priority | Notes |
|---|---------|--------|--------|----------|-------|
| 13 | Add "typal" tag (community moved off "tribal") | Low | S | P2 | One-line label change. |
| 14 | More specific theme tags (e.g. Elves) | Low | S | P3 | Tension with keeping the tag set tight — decide direction first. |
| 15 | Browse all tags up front at deck create/edit | Low | S | P3 | Related to #2. |

## Import sources

| # | Feature | Impact | Effort | Priority | Notes |
|---|---------|--------|--------|----------|-------|
| 16 | CubeCobra ID import + cube-based suggestions | Med | M–L | P3 | Extends existing link-import; niche but loyal cube crowd. |

## Onboarding / UX nits

| # | Feature | Impact | Effort | Priority | Notes |
|---|---------|--------|--------|----------|-------|
| 17 | Password rule errors placed under the password field (not floating up top) | Low | S | P2 | Trivial signup-friction fix. |
| 21 | Clone-deck UX polish — (a) trim the hint text down to one line, e.g. "Make an exact copy of your deck"; (b) on Save, navigate straight to the newly cloned deck | Low | S | P2 | Self-noted (2026-06-29). Two small wins on the existing clone flow. |

---

## Not actionable (logged for completeness)
- **Rename to "Commandr"** — joke, no action.
- **Regional/EU availability** — ops, not a feature; tracked via the DSA trader-verification process.

## Suggested first three to ship
1. **#1 Live drag indicators** — instant polish, addresses the loudest UX confusion.
2. **#8 Card name + detail view** — kills two complaints at once.
3. **#4 Auto land base** — high stickiness, every-deck value.
