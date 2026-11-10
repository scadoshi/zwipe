# 325. The card details drawer's bottom border vanishes at 100% zoom

State: fixing | Owner: scotland | PR: #505, #513 | Build seen: live 1.18.15+ (Surfer), local release 3a627a7c1+ (scotland)

## Report
Scotty, 2026-11-10, web, desktop Chrome at about 1260x800: the card details side drawer (Dreamtide Whale, opened from a deck) shows no bottom border at 100% browser zoom; zooming in brings it back. Screenshot `target/scotty-shots/325-drawer-no-bottom-border.webp`.

## Repro
1. Wide window, 100% zoom, DPR 1. Open a card's details from a deck (or the docked details on Add).
2. The panel's bottom edge has no visible border line; at 125% zoom it shows.

## Diagnosis
Nothing clips the border and nothing covers it. Surfer (live, DPR 1) and scotland (local headless Chrome, 1260x800, 1280x720, 1366x768, 1440x900 and 1536x864 at zoom 1.0 and 1.25) found the same thing:

- The border is there, but the panel's edges sit on fractional CSS pixels. At DPR 1 a 1px border on a fractional edge is antialiased across two device rows at half strength each. It's rgb(40,39,39) on a rgb(20,19,19) body, faint to begin with, so the half-strength rows vanish. Zooming raises the effective DPR and the edge lands near a whole device pixel, so it sharpens.
- Surfer, live: every edge of `.card-details-drawer` is fractional (top 53.59, bottom 612.81, left and right at .41).
- Scotland, local: on `.web-swipe-details` (Add), `scrollHeight == clientHeight` and the inner `.card-details-panel` ends exactly at the box's bottom (no clipping). The box spans 81.781 to 523.266 at zoom 1.0. At 1.25 the bottom is 496.859 CSS px, about 621.07 device px, so nearly whole.

Scotty's screenshot is `.card-details-drawer` (`zwiper-web/src/lib/inbound/cards/card_info.rs:57`), a bottom sheet the kit turns into its side drawer on a wide window. The same cause applies to it and to the docked `.web-swipe-details`. Where the fractions come from:

- Top: `zwiper-web/assets/web.css:919` sets `--sheet-drawer-top: calc(var(--web-nav-h, 4.7rem) + 0.7rem)`, and `--web-nav-h` is the nav's unrounded `getBoundingClientRect().height`, written by `NAV_HEIGHT_JS` in `zwiper-web/src/lib/inbound/shell/top_nav.rs`. `.web-swipe-details` inherits it through `--web-dock-panel-top` (`web.css:1906`, `:1913`).
- Bottom of the drawer: orangui `assets/components.css`, `.sheet-form-drawer > .bottom-sheet { bottom: calc(1rem + var(--sheet-drawer-floor, 0px)) }` (orangui#29, issue 316). The floor is `--web-footer-lift` (`web.css`, from 316), which `zwiper-web/src/lib/inbound/shell/footer_lift.rs` computes from the footer's fractional top. With no footer in view the floor is 0 and the bottom is whole; with the footer in view it's fractional.
- Bottom of `.web-swipe-details`: the box is as tall as its content (441.484 px here), so its bottom stays fractional even with a whole top.
- Not hover: no `:hover` rule touches either element. The old hover lift excludes `.web-swipe-details` (`web.css:499`, `:506`).

## Proposed fix
1. Snap the measured offsets to whole pixels: `NAV_HEIGHT_JS` writes `Math.round(height)`, and `footer_lift` returns a whole number. Both drawer edges, the docked pane's top, and the toast lift (284) then land on whole pixels at DPR 1.
2. For the content-sized `.web-swipe-details` bottom, and as insurance at fractional DPRs, raise the panel border's contrast against `--bg-sink` / `--bg-primary` so a half-strength row still reads, or give the panel a whole-pixel height.
3. Check the border color against each theme's body. 40,39,39 on 20,19,19 is too faint even when crisp.

Test: a failing-first unit test in `footer_lift.rs` that `footer_lift` returns whole pixels for fractional inputs (for example 700.0 and 522.4 give 178.0). The nav height snap lives in a JS string and the visual result needs a browser, so that part gets a headless check (edges are whole numbers at DPR 1, the border shows in a crop) rather than a unit test.

## Fix
scotland, #505: `NAV_HEIGHT_JS` writes `--web-nav-h` as `Math.round` of the nav's height, and web.css rounds `--sheet-drawer-top` (the nav plus 0.7rem) to a whole pixel with `round(..., 1px)`, since 0.7rem alone is fractional. `footer_lift` rounds the footer's reach up to a whole pixel, so the drawer's floor clears the footer; failing-first test `a_fractional_footer_top_lifts_by_whole_pixels`. The card details panel (`magic.css`) and the card details drawer (`filter.css`, after the narrow drawer's border rule it has to beat) take `--border-primary` in place of `--border-secondary`, the border chips and buttons use, so a content-sized bottom edge that stays fractional still reads. Headless at 1260x800, DPR 1: the docked pane's top is 82 (was 82.19) and its border is rgb(98,94,90) on Kanagawa Dragon dark (was rgb(40,39,39)); the border shows on all four edges on Kanagawa Dragon dark and light, Matte Black dark and Material dark and light, at 1260 and 390. Not a wire change. Verify: the Repro shows the bottom border at 100% zoom.

## Follow-up
Surfer, live after #505: the wide drawer and panels are crisp, but below 960 the floating drawer is still fractional (left .406, top 53.594, bottom 612.813 at 800 wide; `~/Developer/surfer-shots/325v-drawer-bottom-800.png` on halo). Its rule (components `filter.css`, `.drawer-every-width` below 60rem) offsets by `2.75rem + 0.6rem` at the top and a 0.6rem gap at the right, and only the wide `--sheet-drawer-top` was rounded.

scotland, #513: the narrow drawer's top, bottom and right go through `round(..., 1px)`, and so does web.css's bottom for the drawer over Add and Remove's two docks, which overrides the kit's. The width (`min(100vw - 3rem, 26rem)`) is already whole, so the left edge is too. Headless, the card details drawer on Remove, now: left 374, top 54, right 790, bottom 593 at 800x720, and 38, 54, 380, 717 at 390x844. Rounding only the kit's rule left the bottom at 592.81 there, from the two-dock override. Not a wire change; no changelog line beyond 325's.
