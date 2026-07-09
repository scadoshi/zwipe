# Weekly badges — client (zwiper, 1.4.x batch)

## 1. API client

`zwiper/src/lib/outbound/client/user/weekly_recap.rs` — `get_weekly_recap()`
returning `HttpWeeklyRecap`, mirroring the existing single-GET client
modules.

## 2. "Your week" recap — the Monday moment

- **Trigger:** during session upkeep (`session_upkeep.rs`, where the
  funnel/min-version checks already run): fetch the recap; if
  `recap.week_start` is newer than the locally stored `last_recap_seen`
  (persisted alongside the other client prefs) and `badges` is non-empty,
  surface the recap. Store the week on dismiss. One fetch per app open,
  cheap; shows at most once per week.
- **Surface:** full-screen dialog styled as a card (the screenshot target),
  NOT a toast. Layout: "Your week" header + week dates, the 1–3 badges
  (title + blurb each, `WeekBadge::title()/blurb()` from core), then a
  compact stat strip (swipes by direction with the swipe-color vocabulary,
  added/maybed/removed, top category, top color as mana glyph). Terminal
  aesthetic, crisp borders, no glow, sentence case, no em dashes.
- **Share:** a "Share" button → OS share sheet with a text summary
  ("My week on Zwipe: Swipe King, The Controller. 412 swipes, 38 adds." +
  zwipe.net). The card itself is designed to screenshot well (safe margins,
  no ephemeral chrome). Rendered-image generation is explicitly deferred.

## 3. Badge history — profile screen

A "Your weeks" row in the profile screen (`profile/mod.rs` pattern) opening
a simple list: one row per week from `recap.history` — week dates + badge
titles. Empty state: "Badges land every Monday. Go swipe." No pagination
v1 (history is capped at 12 weeks server-side).

## 4. Copy + release

- All copy sentence case, no em dashes, "Zwipe" capitalized.
- Store What's New line: "Weekly recap: see your week in swipes and earn
  badges every Monday."
- zite guide section ("Your weekly recap") ships in the same release, not
  before.

## 5. Testing checklist (device)

- Fresh install, no activity → no recap dialog
- Active week → recap appears Monday (dev: fake `last_recap_seen` back)
- Dismiss → doesn't reappear until next closed week
- Share sheet text correct; screenshot of the card looks clean on a phone
- Badge history renders 0 / 1 / 12-week states
