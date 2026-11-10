# 316. The web side drawer covers the footer

State: verified | Owner: scotland | PR: #478 (orangui#29) | Build seen: 1fd04e479

## Report
Scotty, 2026-11-09 (screenshot, web Account, Change email drawer): the side drawer runs to the bottom of the window and sits over the footer. Like toasts (284), it should stop at the footer's top when the footer is in view; full height otherwise.

Scotty, 2026-11-09: happens on every drawer (Add Filter drawer too); fix in the shared drawer so all drawers and their docks stop above the footer.

## Fix
#478, with orangui#29: the kit's drawer ends at `calc(1rem + var(--sheet-drawer-floor, 0px))`, web.css sets the floor from the footer in view (`--web-footer-lift`), and components' narrow every-width drawer (Add's Filter) uses the same floor. Docks sit inside the drawer, so they rise with it. Tests: orangui `tests/drawer_floor.rs`, zwiper-web `every_drawer_stops_at_the_footer_s_top`. Shipped in 1.18.15.

## Verification
Surfer, live aeda394a4 ([`verify-aeda394a4.md`](verify-aeda394a4.md)): PASS. Change email on Account and Filter on Add end 16px above the footer at page end, wide; the deck page's Filter runs full height at the top of the page; docks sit inside. Below the breakpoint the drawer is a full-width bottom sheet that covers the footer under its scrim, which is the phone pattern, not the side drawer.
