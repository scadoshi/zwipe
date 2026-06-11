# Price intelligence — tracking, alerts, shopping list

**Tier: split — refined 2026-06-10: drop alerts go to FREE users too.**
Current prices, deck totals, and capped automated drop alerts on
shopping-list cards are **free**. Premium buys **control and depth**: custom
thresholds, arbitrary watches, instant delivery, history charts, budget swaps.

Rationale for free alerts: the pipeline's recurring cost is near zero (price
data rides the existing Scryfall sync; alert evaluation is one daily SQL diff;
**APNs/FCM delivery is free** — no per-message cost, unlike email). Meanwhile
every drop alert is an affiliate-link impression at the moment of purchase
intent, so free alerts monetize free users AND are the best premium upsell
surface ("want a custom threshold? →"). Cost was never the real gate; the only
free-tier limiter is notification fatigue, handled by capping cadence.

## The killer combo

**Deck minus collection = shopping list.** Every deck has a computable "cards
you still need" set (see `collection-tracking.md`), and that's the thing worth
price-tracking. The headline notification:

> "Rhystic Study on your shopping list dropped 22% this week."

A purchase trigger the user is *grateful* for — it re-engages them on our
schedule instead of theirs, and notification-driven DAU is the lifeblood of a
subscription app.

## Free tier

- Current prices + deck totals (table stakes — Moxfield shows them).
- **Automated drop alerts on shopping-list cards** — threshold we choose
  (e.g. "dropped >15%"), capped cadence (weekly digest or max a few
  pushes/week). Every alert carries an affiliate link.

## Premium tier — control and depth

- **Custom watch thresholds** — "alert me when this is under $5."
- **Arbitrary per-card watches** beyond the shopping list (maybeboard
  auto-watch, speculation watches).
- **Instant alerts** instead of the free digest cadence.
- **Price history charts** per card and per deck (deck value over time).
- **Budget swaps** — "this $40 card has a $4 functional cousin" (pairs with
  AI analysis or curated swap data).

## Implementation — cheaper than it sounds

- **No new API deals.** Scryfall bulk data already carries daily TCGplayer /
  Cardmarket / Cardhoarder prices, and zervice already syncs Scryfall. Price
  *tracking* = snapshotting prices into a history table on the sync we already
  run. Retention policy (e.g. daily for 90 days, weekly beyond) keeps the
  table sane.
- **Push notifications are the genuinely new infrastructure** — APNs for iOS,
  FCM when Android lands. One-time build, reused by everything afterward
  (security notices, social features someday).
- Alert evaluation is a zervice job after each price sync: diff against
  watches/thresholds, enqueue notifications.

## Affiliate revenue

TCGplayer has an affiliate program. Drop alert → tap → buy through our link is
a **second revenue stream on the same feature**, and since free users get
alerts too, it monetizes the whole user base. Worth applying for early —
approval can lag. Expectation check: commission is a few percent (a $20
pickup earns ~$1) — real margin at scale, but gravy; the subscription stays
the meal.
