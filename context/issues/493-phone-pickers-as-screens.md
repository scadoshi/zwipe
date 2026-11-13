# 493. On phone web the deck form's pickers slide in as a sheet; the app shows them as a screen

State: fixing | Owner: scotland | Type: issue | PR: - | Build seen: production

## Report
Scotty, through the composer, 2026-11-13: on phone web, deck create and edit's pickers slide in as a full-width sheet from the right with an ×. The app shows them as a route change with a back arrow, which is what he wants: no slide, a back arrow top left, Back closes it like a screen (457's route feel). Wide web unchanged.

## Before
Headless at 390, a deck's edit page: Format (its field reads Commander), Deck tags and Oracle tags each fill the window and slide in from the right (0.3s transform, 133px in after 120ms), with the × leading the title bar, 3 of 3; the form stays painted under them. The Oracle tag dictionary and the command zone's Swipe already open as screens with a back arrow (the dictionary at once, Swipe with the app's 0.2s fade).

## Cause
`zwiper-web/src/lib/inbound/screens/deck/form/pickers.rs`, `PickerSheet`: every picker is components' `DrawerSheet` at every width, which on a narrow window `web.css` stretched over the whole window, so it kept the drawer's slide and ×.

## Fix
scotland:
- Below 600px (where the drawer took the whole window) `PickerSheet` is a screen over the form, as the app's `TagSelect` is: `.screen.swipe-select-screen`, fading in with no slide, the page header's back arrow and `?`, the body and the checkmark dock. From 600px it stays the drawer.
- The form's `ScreenFrame` takes `covered` while a narrow picker is open (`use_pickers_cover`): not painted, focused or tapped (`under-layer`, `inert`), and its entrances run again when the picker closes (`screen_return`).
- The narrow-drawer rules in `web.css` go.

## Verification
- Headless at 390: Format, Deck tags and Oracle tags open as a screen with a back arrow and no ×, no transform (the opacity eases), the form hidden and inert under them; Back closes each, 3 of 3, and the form comes back with its entrances running. The tag grid scrolls within the screen and ends above the dock.
- 700 and 1440: the pickers are the drawer, as before.
