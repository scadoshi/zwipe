# Premium tier — feature ideas

Brainstormed 2026-06-10. This directory details every candidate feature for the
paid tier (and the free features that exist to feed it). The monetization
*decision* (freemium, subscription, $3–5/mo, no ads) lives in
`../monetization.md` — this directory is the feature catalog under it.

## Guiding principles

- **The core loop is the swipe stack.** The most defensible premium features
  make the stack itself smarter, not bolt on screens users have to find.
- **Numbers free, intelligence paid.** Current price free, history + alerts
  paid. Bracket number free, "why and how to move" paid. The free metric is
  the acquisition surface; the paid layer explains and acts on it.
- **Free tier = acquisition + moat.** Import/export, sharing, basic metrics,
  collection tracking, and the bracket badge stay free — they drive installs
  and switching cost. Charging for them strangles growth.
- **Honest subscription framing.** Features with ongoing cost (AI calls, price
  polling, push notifications) justify recurring billing. Pure-math features
  are cheap sweeteners, not the headline.
- **Deck tags are the join key.** One closed vocabulary (shared with
  `card_profiles.mechanical_categories`) powers consistency math, AI prompts,
  stack ranking, and bracket heuristics. Build it once, consume it everywhere.

## Feature catalog — by decision confidence (sorted 2026-06-10)

### DECIDED: premium — cost-heavy or AI; these justify the subscription

| Feature | Ongoing cost | File |
|---|---|---|
| AI deck analysis (preset prompts, incl. bracket coaching) | API calls | `ai_analysis.md` |
| Price intelligence: history charts, custom thresholds, instant alerts | storage (cheap) | `price_intelligence.md` |
| Smart stack ordering + taste profile | storage/compute, partner API | `smart_stack.md` |
| Synergy integration (from original 2026-03 decision) | data layer | `smart_stack.md`, `progress/todo.md` |
| Consistency per-tag breakdown (headline score stays free) | none — intelligence framing | `consistency_calculator.md` |
| Deck snapshots / version history with diffs | storage (cheap) | `deck_snapshots.md` |
| Budget swaps ("$4 cousin of this $40 card") | rides price data + AI | `price_intelligence.md` |
| Cosmetics (themes, icons, card backs) — sweetener, build last | none | (too small for a file) |

### DECIDED: free — acquisition, moat, or table stakes

| Feature | Why free | File |
|---|---|---|
| Collection tracking | moat / data gravity / retention | `collection_tracking.md` |
| Bracket badge (the number + factors) | most shareable feature; acquisition | `bracket_estimate.md` |
| Current prices + deck totals | table stakes (Moxfield has them) | `price_intelligence.md` |
| Capped drop alerts on shopping-list cards (refined 2026-06-10) | push is free (APNs/FCM); affiliate impressions + upsell surface | `price_intelligence.md` |
| Consistency headline score, opening-hand simulator, mana math | zero cost; very screenshot-able — acquisition | `consistency_calculator.md` |
| "Cards I own" swipe filter | it's a filter; filters are free | `collection_tracking.md` |
| Deck tags (foundation infrastructure) | everything consumes it | `deck_tags.md` |
| Import/export, sharing, basic metrics | already free; never paywall migration | — |

### MAYBE pile — new ideas land here first

Empty as of 2026-06-10 — all previous maybes were sorted into the decided
buckets above. When a new feature idea shows up, put it here with a lean and a
one-line reason, and only promote it once explicitly decided.

## Candidate launch trio

Smart stack ordering, AI analysis with presets, and price intelligence. All
three are "intelligence," all three have ongoing costs that make the
subscription feel fair, and all three sit on the deck-tags + collection
foundations that ship free first.

## Sequencing reality check

Android ships before premium (decided 2026-06-10, see `progress/todo.md`) — a
bigger user surface first, then monetize. The tags/consistency foundation work
is free-tier and can proceed in parallel; the IAP build is the gate on
actually charging anyone.
