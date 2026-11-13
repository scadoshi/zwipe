# 496. Home has no `?` hint

State: fixing | Owner: thinker | Type: polish | PR: - (pushed to release) | Build seen: 8096283c4

## Report
surfer's walkthrough (`halo:~/surfer-shots/walkthrough/README.md`, item 4), triaged by Scotty: no `?` hint on Home, while other screens have one. Add a basic one, short and plain, on the web and in the app.

## Before
Headless Chrome at 1440x900, signed-in fake session, Home, 3 of 3: the nav holds no live `?` (its slot is held empty) and the `?` key opens nothing.

## Cause
Home's one-time hint, the first-login welcome (`OneTimeHint::FirstLogin`), opens by itself once, but Home never handed its frame a hint: `zwiper-web/src/lib/inbound/screens/home/mod.rs` gave `ScreenFrame` no `on_hint`, and `zwiper/src/lib/inbound/screens/home.rs` gave `ScreenHeader` no `hint`. Every other screen's `?` reopens its own one-time hint the same way.

## Fix
- Home's `?` (the button and the key) opens the welcome, on the web and in the app.
- The welcome's copy is shorter: four lines for Decks, Colors and Playstyle, Account and Settings together, and verifying the email (`zwipe_ui::hint::one_time`).
- Tests: `home_s_hint_is_short_and_plain` pins the lines (red before); `the_hints_name_the_shipped_controls` follows the Playstyle line.

## After
Same setup, 3 of 3 at 1440 and once at 390: the nav's `?` is live and the `?` key opens "Welcome to Zwipe".

## Not checked
- The app on a device (the same header `hint` other screens pass).
