# 503. A tall hint cuts its last lines with no sign it scrolls

State: fixing | Owner: scotland | Type: issue | PR: - | Build seen: production

## Report
Scotty's walkthrough triage (composer, 2026-11-13): the Playstyle hint at 390 cuts its last bullet with no sign it scrolls. The hint box grows to fit its text: no fade, no inner scroll; taller than the screen, the whole hint scrolls like a page. Check the other hints the same way at 390.

## Before
Headless, each page's hint opened from its `?`, the text its body hides under its own scroll: at 390x844, 2 of 10 (Playstyle 128px, the deck page's Welcome to your deck 177px); at 375x667, 6 of 10 (Colors 10, Building a deck 21, Activity 182, Commander maybeboard 201, Playstyle 331, Welcome to your deck 412). /guides has no hint.

## Cause
orangui's `dialog.css`: an alert dialog stops a 16rem band short of the window and its body (`.alert-dialog-description`) scrolls on its own between the pinned title and actions, so a long hint's lines go under the actions with nothing showing more is there.

## Fix
scotland:
- `components/assets/screens/components.css`: a hint (`.alert-dialog:has(.hint-body, .hint-line, .hint-bullets)`) grows to a 1rem margin short of the window, its body keeps no scroll of its own, and one taller than that scrolls as a whole, title and buttons with it.
- `zwiper-web/src/lib/inbound/app.rs`, `HINT_FOCUS_JS`: an opening hint focuses its button without scrolling to it, so a tall hint opens at its title (it opened 188px down, at its end).

## Verification
- Headless 375x667, the same ten hints: no body hides text (0 of 10); Playstyle and Welcome to your deck, taller than the window, open at the title with focus on Got it, and a swipe scrolls the whole hint to its buttons (585 to 619 of 667).
- 1440: Playstyle, Welcome to your deck and Activity fit with nothing hidden.
