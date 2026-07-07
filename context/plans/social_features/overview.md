# Social features — weekly badges + featured decks

**Status: PLANNED (2026-07-06). Not started. Two independent legs; badges
buildable now, featured decks after the share page + MVPs ship.**

**What this builds, in one sentence:** a weekly "Your week" recap that awards
each active user 1–3 personality badges computed from the signal already
being collected, and an owner-curated featured-decks showcase on zwipe.net
where standout decks appear with their MVP cards front and center.

**Why this shape:** at ~865 users there's no density for social
*infrastructure* (profiles, follows, feeds, moderation). Both legs here are
social *artifacts* — things a single user can enjoy alone and choose to show
someone — the same principle that picked the deck share page. Badges are the
retention loop (a reason to come back Monday); featured decks are the
aspiration loop (a reason to build something worth showing, and MVPs'
first public stage). Absorbs and supersedes the "Weekly Badges + Stats /
Share Cards" backlog entry (design decisions carried over).

## Design principles (carried from the 2026-07-02 backlog decisions)

- **Derive, don't collect.** Badges are computed from `user_week_signal` +
  `user_week_facet_signal` (live and accruing since 2026-07-02) plus joins
  onto tables that already exist. No new telemetry. A new counter needs a
  named badge consuming it.
- **Private first.** The recap is yours; nobody sees your badges v1. The
  shareable surface is opt-in (screenshot/share sheet). Public
  profiles/leaderboards remain a later, deliberate subsystem.
- **Featured = shared + hand-picked.** Only decks with a live `share_token`
  are eligible; the owner curates via zcript, and asks the deck's builder
  (Discord) before featuring. No algorithm until there's MVP data worth
  ranking on and enough decks to rotate.
- **Terminal aesthetic, crisp, no glow.** The badge card is a Wrapped-style
  artifact people screenshot; it has to look like Zwipe.

## The pieces

| Piece | Doc | Depends on | Ships |
|---|---|---|---|
| Badge tables, rules, week-close job, recap endpoint | [`badges_server.md`](badges_server.md) | nothing (substrate live) | anytime, server-only |
| "Your week" recap + badge history in the app | [`badges_client.md`](badges_client.md) | badges server | 1.4.x client batch |
| Featured decks: flag, public endpoint, zite showcase | [`featured_decks.md`](featured_decks.md) | [`deck_share_page`](../deck_share_page/overview.md) + [`deck_mvps`](../deck_mvps/overview.md) | after both |

## Sequencing

1. **Badges server** — deployable alone; badge history starts accruing
   silently (backfills every closed week since 2026-07-02 on first run).
2. **Badges client** — rides the next client release after; recap appears
   the first Monday after install.
3. **Featured decks** — server+zite legs can ship with/after the share
   page; the showcase gets real MVP stars once 1.4.0 clients start starring.

## Later (explicitly out of v1)

- Public profiles, follows, comments, leaderboards (density-gated).
- Rendered share-card image generation (v1 shares are the screenshot-worthy
  recap screen + OS share sheet text).
- Algorithmic/hybrid featuring rotation (needs MVP volume).
- Badges on shared deck pages / MVP-based badges ("Kingmaker: your MVP got
  featured") — natural cross-links once both legs exist.
