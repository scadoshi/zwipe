# 298. The web app's header breaks at 150% zoom

State: verified | Owner: laptop | PR: #440 | Build seen: 1d7053e88

## Report
Scotty, 2026-11-08 (screenshot, app.zwipe.net/decks at 150% browser zoom, window ~1200 CSS px wide at that zoom): the header wraps into two rows, nav pills (Decks, Playstyle, Activity, Commander maybeboard) on top and the icon row plus Log out underneath, with an empty dark block filling the left of the header beside the logo and Back. The header takes about a third of the viewport. "A little crazy."

## Repro
Surfer, live web 1d7053e88, 2026-11-08. Method: one same-origin iframe inside the existing app tab (the app's own index.html written into an about:blank frame, so it runs signed in), resized to each CSS width. Browser zoom is treated as width: window width ÷ zoom (1440 at 150% = 960, 1280 at 150% = 853, 1440 at 125% = 1152, 1440 at 175% = 823). No windows opened or resized.

Header bands, measured every 10px from 1460 down to 380 on Decks and at the widths below on every screen:

| CSS width | Header | Screens |
|---|---|---|
| ≥ 1110 (≥ 1030 on Home) | one row, 70px | all |
| 970 to 1100 (970 to 1024 on Home) | two rows, 109px: nav pills right-aligned on top, icon row + Log out under them | all |
| 900 to 960 | three tiers, 154px: the pills wrap too, icons below; the empty block shows | all |
| ≤ 890 | hamburger, 43px | all |

Home holds one row longer because it has no Back button.

The empty block is `.nav-panel`'s own background. The panel is `flex: 1` with a surface background (darker than the nav wrapper) and spans from after Back to the right edge (at 960: x 161 to 936, 114px tall). Its child `.nav-panel-inner` is `flex-wrap: wrap; justify-content: flex-end`, so once it wraps, `.nav-links` (526px) and `.nav-trailing` (429px) each sit right-aligned on their own row. The left part of the panel stays empty and shows that background as a dark box beside the logo and Back. At 960, `.nav-trailing` also runs 12px past the panel's right edge (to x 948).

Knock-on effects of the taller header:
- Add: the card details panel is placed at a fixed top of 111px, so from 900 to 960 its title sits 43px under the header. Separately, it overlaps the swipe card; see 302.
- Sheets (Themes on Settings checked) open at a fixed top of about 84px, so at 960 to 1100 the sheet covers part of the wrapped icon row (under the scrim, so cosmetic).

Coverage (everything else was clean: no horizontal scroll, no element past the right edge, no text spilling out of a chip or button):

| Screen | Widths |
|---|---|
| Home, Decks, deck page (Surfer test), Add, Playstyle, Settings | 1920, 1600, 1440, 1309, 1280, 1152, 1110, 1100, 1024, 970, 960, 900, 890, 853, 823, 800, 720, 640, 533, 457, 430, 390 (automated); screenshots at 1920, 1100, 960, 720, 390 |
| Decks header | every 10px, 1460 to 380 |
| Settings with the Themes sheet open | 1440, 1100, 960, 390 |
| Hamburger menu open | 390 |

Heights were 800 CSS px for the automated pass (height doesn't change the header).

Shots on halo: ~/Developer/surfer-shots/298-decks-1120-one-row.png, 298-decks-1100-two-rows.png, 298-decks-960-empty-block.png.

## Decision
Scotty, 2026-11-08: switch to the hamburger below 1110 CSS px (instead of 890) so the header never wraps. Recheck 302 after.

## Fix
Branch `nits/laptop-298`, with orangui#28. The header folds behind the hamburger below 1110 CSS px (`media::NAV_FOLD_MAX_PX = 1109`), so it is one row from 1110 up and the folded bar below, with no wrapped two- or three-row band. orangui now exports its NavBar fold rules as `NAV_FOLD_CSS` (its own `COMPONENTS_CSS` still applies them at 60rem, so portfolio and cairn are unchanged); web mounts them again under `@media (max-width: 1109px)`. Web's own header queries moved with it (the folded app-nav block, `.web-nav-wide`/`.web-nav-narrow`, the bar's `?`, header_hide.js) and the 900px-to-60rem block that held the bar open to wrap is gone. Page layouts keep their 900px breakpoint. The wide drawer's top follows the measured nav height (`--web-nav-h` + 0.7rem) instead of a fixed 5.4rem, so sheets and Add's details panel start under whichever header shows. Test: `the_header_folds_below_1110_and_never_wraps`. Checked on a local stack in headless Chrome: Decks at 1120 and 1110 one row (71px, the bar ending at 1086 of 1110), 1100 and 960 folded (44px); the menu at 1100 drops as on a phone; Home folds the same.
