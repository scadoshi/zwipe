# zwiper-web nits from the live click-through

Logged while Scotty clicks through app.zwipe.net after launch (2026-11-06). Each is a line until it is picked up; mark it done with the commit.

## Plan (2026-11-06)

Order of attack, by what users hit and what it costs:

1. **Safety first:** 24 (held arrow key fires a swipe, and a server write, per key repeat; slow the animation).
2. **Quick CSS wins, one pass:** 29 (featured card names in chips, all clients), 3 (chart title spacing), 4 (row dividers), 6 (privacy width), 8 (image preview size), 9 (panel hover), 10 (deck art hover), 13 (changelog page size), 16 (dialog padding), 18 (search box widths), 20 (dictionary letters row).
3. **The stylesheet split (a launch blocker):** the web stops inlining `zwiper/assets/main.css` and owns its rules. It settles 12 (scrollbars) and stops app CSS changes from breaking the web unseen.
4. **Sheets and drawers, one pass:** 5 (modal drag), 17 (Account changes as a drawer), 19 (pickers as the agreed drawer, no grid inside), 25 (examples' tiles), then 1 (toasts bottom left).
5. **Deck controls, one pass:** 2, 11 and 23 together (fold-all's home, the web dropping the inline Filter row), then 26 (the eye icon on every swipe).
6. **Layout:** 7 (deck tiles), 14 and 15 (Playstyle chip rows above panels, foldable panels), 27 (the footer).
7. **Skeletons:** 28 (a full alignment pass, both layouts), after the layout work so the ghosts match the final screens.
8. **Shared-code moves, scoped first:** 22 (zite's hover stack into `components`), 21 (the full filter on zwipe.net's shared deck, which needs the filter drawer in `components`).

Decided 2026-11-06: the stylesheet split goes ahead as option (d); gloo-timers joins components for the hover stack; fold-all stays after the Show chips everywhere, so 11 is won't do (21 removes its row).

Still open beyond nits: the platform tag on funnel events (check `ClientPlatform` for a web value first), and switching the CSP from report-only to enforced after a click-through with the console open.

## In progress

| Items | Machine | Branch |
| --- | --- | --- |
| 52, the swipe buttons move into the action dock (queued after 51) | laptop | `nits/laptop-52` |
| 51, swipe twice as fast, same animation speed | laptop | `nits/laptop-51` |
| 50, the deck head's ? matches Edit, Clone and Share | thinker | `nits/thinker-50` |
| 49, every clear-x input through `components::ClearWrap` | halo | `nits/halo-49` |
| 45, 47, no page heads, the page's ? in the nav, About drops Credits on wide (queued after 21) | scotland | `nits/scotland-45` |
| 46, a compact footer on both sites (queued after 40) | halo | `nits/halo-46` |
| 21, the card filter into components (plan phases 3 and 4) | laptop | `shared/card-filter` |
| cycle-trk-mission description, held until scotland's 2026-11-07 sync confirms its cards | halo | `otags/cycle-trk-mission` (PR #178) |
| 38, the footer looks like zwipe.net's | scotland | `nits/scotland-footer-look` |

The loop is in [`../development/factory.md`](../development/factory.md).

## Open

1. **Toasts cover the nav on a wide window.** They sit top right (`zwiper-web/assets/web.css`, the `.toast-container` override at `min-width: 900px`), over Log in, Create profile and Forgot password. Wanted: bottom left. Scope, CSS only in `web.css`:
   - Anchor the container `bottom: 1rem; left: 1rem; top: auto` on a wide window.
   - The kit stacks newest first and tucks older toasts up behind it with a negative `margin-top`, and drops each one in from above (`toast-drop-in`, `translateY(-1.5rem)`). Anchored at the bottom, the stack should read the other way: newest at the bottom, older ones peeking above, arriving from below. That is a `column-reverse` list, the tuck flipped to `margin-bottom`, and the entry keyframes mirrored, all scoped to the web.
   - Bottom left is where Add and Remove keep their pile chips (From, Search; Main, Maybe, Side, All) and where the deck page's fixed dock is not, so the stack needs to sit above those chips on the swipe screens or it covers them.
   - The phone layout keeps the kit's top placement.
2. **The deck page has no fold-all button on a wide window.** The app's Cards screen has it after the Show chips (Tokens, Command zone, Art); the wide deck page (`/deck/get/:id`) shows the Show row without it.
3. **Distributions: the Types chart's title chip touches the rule above it.** On the wide deck page the Types chip (and its `?`) sits flush under the Average P/T row's rule, and the Roles chip sits flush under the Types chart's labels. Chart titles need the same gap above them as between charts.
4. **Deck page card rows have no dividers.** On the wide deck page the rows inside a group (Commander, Creatures) run with no rule between them, where the app's card list separates each row with a bottom border (the shared `.card-row` rule in `components/assets/magic.css`, last row excepted). Likely a web or band style overriding it.
5. **Modals can be dragged around the screen.** Clicking and dragging a modal (the Universes Beyond exceptions picker, the other screen pickers) moves it. Likely the bottom sheet's drag-to-dismiss gesture still live on the kit's wide `SheetFrame { wide: Modal }` form, where nothing should move it; it should be off on a wide window.
6. **Privacy policy is too narrow on a wide window.** The page head and sections are held to the policy's 40rem column (the `.web-privacy` rules and the `:has(+ .screen-content .web-privacy)` head cap in `web.css`), so a large window leaves wide empty margins. Wanted wider, likely the page width other screens use (`--web-page-max`), or zite's privacy page width. The head still repeats Privacy policy above the first section's own title.
7. **Deck tiles on Decks are much too tall.** Decided 2026-11-06: the phone's row shape (art left, name and chips beside) in one column, with two columns at wide widths to compare. Six tiles to a row on a wide window, each about 11rem wide (`repeat(auto-fill, minmax(11rem, 1fr))` in `web.css`), so the chips under the name stack one per line: count, format, power, commander, then every tag. A tagged deck runs over 800px tall. Needs a plan; options to weigh:
   - Fewer, wider tiles: raise the tile minimum to about 18 to 20rem, three or four to a row, so chips wrap several to a line.
   - Let the chips flow in rows: they read as a column now even when two short ones would fit side by side; a centered wrapping row alone cuts the height a lot.
   - The app's row shape on wide too: art at the left, name and chips beside it, two or three rows across the page, the shape the phone layout already uses.
   - Cap what a tile shows: the count, format and commander, with tags behind a `+3` chip or left to the deck page.
8. **The image preview is smaller than zwipe.net's.** Tapping a card's Image (the Featured flavor's on Home, and wherever the web opens `ImagePreview`) shows the card about 435px wide on a 1280-wide window, where zite's preview on a shared deck shows it about 680px wide, near the window's height. The click, the dim and the animation already match; only the size differs. Match zite's sizing.
9. **Panels have no hover with a mouse.** zwipe.net and scottyfermo.com lift a panel on hover: `translateY(-3px)` with the medium shadow, inside `@media (hover: hover)` so touch never sticks (the portfolio's `.theme-wrapper .panel-card:hover` in `assets/main.css`, which notes it needs the extra class to outrank the kit's `transition: border-color`). The web's panels sit still. Add the same lift to the web's panels on a wide window, hover-capable pointers only.
10. **Home's recent decks: the art should react to hover.** In the Decks panel on Home, hovering a deck's art should grow it a little, the image only, so it reads as clickable.
11. **zwipe.net's shared deck: fold-all belongs at the end of the Filter row.** On `zwipe.net/deck/<id>` the fold-all button sits after the Show chips (Tokens, Command zone, Art); wanted at the end of the Filter row, after the five pips. The deck page is the shared `components::DeckControls`, so the web's wide deck page moves with it (and gains the button there, item 2). This changes the 2026-11-05 call that fold-all follows the Show chips on every client: decide whether the app's Cards screen moves too.
12. **Scrollbars: show them on a wide window, hide them below the breakpoint.** The web app shows no scrollbars on a wide window, where a desktop browser user expects them; below 900px it should keep the app's hidden scrollbars. Find where the app's no-scrollbar rule reaches the web (the app's stylesheet the web still inlines, or the kit's app sheet) and scope it to the narrow layout.
13. **About's changelog lists every release.** zite shows five at a time with the rest behind Show more (`0f98119e2`); the shared changelog component (`components/src/changelog.rs`) already takes a `page_size` for it. The web's About should pass the same size, as zite's `pages/changelog.rs` does.
14. **Playstyle: Suggested commanders' and Staples' chip rows belong above their panels.** On the wide Playstyle page the Show row (All, the pips, the guilds and shards, Art) sits inside the Suggested commanders panel, and the Type and Color rows inside Staples. Move each panel's chip rows out above it, on the grid, as Decks and the deck page keep their controls above the panels; the panels then hold only their headers and card rows.
15. **Playstyle on a wide window: Summary, Colors, Card roles and Oracle tags should fold.** Those panels are fixed open on the wide page; give them the fold arrow and collapsible body the deck page's bands use (`DeckFold`), so the page can be folded down to what is wanted.
16. **Confirm dialogs on a wide window: too much padding above and below, too little in the middle.** The Clear skips dialog (and the other `AlertDialog`s across the web) keeps a good width, but the title row has a tall band of space over it, the action row a tall one under it, and the message between two rules is pinched. Rebalance on a wide window: less above the title and below the buttons, more around the message.
17. **Account's Change username, email and password open as a side drawer on a wide window.** Below the breakpoint they stay bottom sheets, as in the app. On a wide window use the kit's `SheetFrame { wide: Drawer }`, the form the card filter already uses.
18. **Quick add search boxes run the full page width.** On Commander maybeboard (and the deck page's Quick add) the box stretches across the whole page on a wide window. Cap it to a reasonable field width, about 30 to 36rem, beside its label. Audit every search box and text field on the web's wide pages for the same stretch: Quick add on the deck page and the maybeboard, the filter's Name, the pickers' searches, Import's URL field.
19. **The deck form's pickers (Format, Oracle tags, Card roles) should be the agreed side drawer.** On a wide window they open as a full-height panel flush with the window's top and bottom (`.web-picker.swipe-select-screen` in `web.css`) with grid cells painted inside it. Use the kit's drawer form (`SheetFrame { wide: Drawer }`, inset from the top and bottom like the card filter's drawer) and drop the grid cells inside: the grid lives only on the app's own backdrop, never layered inside a panel or sheet.
20. **Oracle tags dictionary on a wide window: letters in one row, the search above them.** The dictionary modal stacks the A to Z and # letters in a six-wide grid in the left column, with the search box over them and no label. Lay the letters out as one horizontal row, and put the search box above it with a Search label chip, as the app's dictionary does.
21. **zwipe.net's shared deck should use the app's full card filter.** The share page filters with its own short Filter row (Name, six type chips, the five pips). The web app now has the full filter (the drawer from the funnel, every section), so the shared deck page opens the same drawer and the inline Filter row goes (Scotty, 2026-11-06: instead of the inline filtering, not beside it). That is a zite change, and the filter drawer lives in zwiper-web today (`zwiper-web/src/lib/inbound/card_filter/`), so it would have to move to `components` (or zite gain its own copy) first; scope that before building. It also settles where fold-all sits on that page (item 11).
22. **Card row hover on the web's deck page must match zwipe.net's shared deck exactly.** Hovering a card row on `zwipe.net/deck/<id>` and on the web's deck page should look and behave the same: the same preview, placement, timing and row highlight. The two are built separately: zite's is a stack of up to five full-art previews pinned to a top corner that dodges the cursor (`zite/src/pages/shared_deck.rs`, the hover-preview stack near line 357), while the web uses its own `card_row::HoverPreview` and `HoverPreviewHost` (`zwiper-web/src/lib/inbound/cards/card_row.rs`, `screens/deck/view/page.rs`), one preview at a time. Take zite's, ideally by moving it into `components` so both draw the same code. What the web does now: the preview appears only after the cursor settles on a row for a moment, and vanishes the instant the cursor leaves the row. zite's pushes a card onto its stack the moment a row is entered (up to five, newest on top), and each card fades out on its own 2-second timer, so previews linger after the cursor moves on.
23. **The web's deck page drops its inline Filter row.** The wide deck page shows the shared page's inline row (Name, six type chips, the five pips) though the dock's funnel already opens the full card filter. Remove the row there. The row comes from the shared deck page that zite also draws, so hide it for the web (a `DeckControls` prop or slot) and keep it on zwipe.net until item 21 gives the share page the full filter too. Fold-all then needs a home other than the end of the Filter row on the web (item 11).
24. **Holding an arrow key races through swipes.** Holding Left on a swipe screen fires a swipe on every key repeat, far faster than the animation, and each one is a server write, so it can run into the rate limits. Ignore repeated keydowns (`KeyboardData` repeat) and take no new swipe until the current card's exit animation finishes; and slow the swipe animation itself down: Scotty finds it far too fast on the web.
25. **Oracle tag Examples needs a rework on a wide window.** The examples modal paints grid tiles behind its content (the grid only lives on the app's backdrop, item 19's rule) and its layout needs a fresh look overall; Scotty flagged the full redesign as later, the tiles as now.
26. **Every swipe surface shows the eye icon, as the app does.** The wide swipe screens drop the eye (card details) icon because the details sit beside the card; Scotty wants it back on every form of swiping (Add, Remove, swipe select, the examples), matching the app. This reverses the 2026-11-04 call of no eye icon on a wide window.
27. **The web app has no footer.** zwipe.net closes every page with its footer (copyright, Privacy policy, Community, Contribute, the Scryfall credit and the Fan Content notice); the web app ends at the last panel. Add the same footer on a wide window, ideally zite's footer moved into `components` so both draw it.
28. **Skeleton alignment pass across the whole web app, both layouts.** Loading skeletons are mismatched all over: their shapes, sizes and positions don't match the screens they stand in for, so content jumps when it loads. Walk every screen at a wide and a phone width, compare each skeleton against its loaded screen, and fix the ghosts to match (the `SKELETON_DELAY_MS` build flag holds GETs so each skeleton stays up long enough to compare).
29. **Featured cards: the card's name in a tag chip, as Home's decks show theirs.** Under each featured card on a deck (the commander and the MVPs) the name is plain text over its Commander or MVP chip; Home's Decks panel puts each deck's name in a chip beside its Commander chip. Do the same for the featured cards on every client: the app's deck view (`deck-featured-card`), and the shared deck page (`components`' `FeaturedCard`, `deck-head-card*`) that zwipe.net's shared deck and the web's deck page draw, plus the web's own `screens/deck/cards/featured.rs` and zite's `pages/shared_deck.rs` where they place it. An app change, so it rides the next app release with a changelog entry.
30. **The nav bar's icon buttons have no hover on a wide window.** The top bar's icon buttons (the `!`, Home, Account, Settings, About, Theme and the light/dark toggle) sit still under a mouse, where the text pills beside them (Decks, Playstyle, Activity, Commander maybeboard, Log out) read as clickable. Give them a hover inside `@media (hover: hover)`, matching whatever the pills and the panels (item 9) settle on.
31. **Card row dividers elsewhere (follow-up to 4).** Each web card row sits in a wrapper (`.web-deck-row > .web-card-row > .card-row`), so the shared `.card-row:last-child { border-bottom: none }` strips every divider. 4 fixed the wide deck page only; the same likely happens below 900px and on Activity, Playstyle and the maybeboard lists.
32. **Text fields still uncapped (follow-up to 18).** The deck form's fields, the command zone search and the deck dialog's input were not checked for full-width stretch on a wide window.
33. **The kit's BottomSheet should not drag in modal or drawer form (follow-up to 5).** 5 was fixed in `web.css` by turning off pointer events on the header and pinning an open modal or drawer with `!important` over the drag's inline transform. The clean fix is in orangui: skip starting the drag-to-dismiss when the sheet shows as `SheetWide::Modal` or `Drawer`. That is a kit change and pin bump across zwipe, cairn and portfolio; once it lands, drop the web.css workaround. The same kit pass should drop `AlertDialog`'s hardcoded `target="_blank"` on dialog links for the wasm build, so the web's support dialog (Report a problem, Discord) opens in the same tab too (follow-up to 36).
34. **The maybeboard's row entrance stagger is broken on the web.** The web's row wrappers make each `.card-row` its wrapper's only child, so `main.css`'s `.maybeboard-list .card-row:nth-child(n)` delays all match the first, and every row enters at once.
35. **About overflows on a phone width.** At 400px the About page's panels ran off the right edge in a headless build before the split's phase 5. Not reproduced since (2026-11-06, thinker, after the split); waiting on a screenshot from a real phone. PR #183 fixed a related overflow found on the way: the phone menu's footer at 320px.
36. **Outbound links open in the same tab on zwipe.net and app.zwipe.net.** Home's "Magic: The Gathering" link and every other `target="_blank"` or programmatic new-tab open take over the current tab instead. The mobile app keeps sending links to the system browser.
37. **Every clickable chip and panel header button gets the plain button hover on the web.** Home's All decks and Playstyle header buttons, Decks' Group by options, Show pips, tag chips and fold toggles, and the same chips across the deck page, Playstyle, the filter drawer and Activity. The kit's hover (accent text and border), no transform or size change; label and count chips stay still.
38. **The web's footer looks wrong.** It sits as a solid block boxed into the page column, cuts off the grid backdrop, and its pills are gold. Match zwipe.net's footer.
39. **The web's splash plays like zwipe.net's.** zwipe.net plays its splash once a session. The web should too: a fresh app.zwipe.net tab shows it, a refresh does not, and arriving from zwipe.net does not (detected by the referrer, since session storage does not cross origins). zite mirrors it for arrivals from app.zwipe.net.
40. **Between a phone and the 900px breakpoint the web keeps the phone column.** At a half-screen window (~720px) app.zwipe.net's Home stays a narrow centered column with wide empty margins, where zwipe.net's panels fill the window with a small gutter. Below 900px the web's content fills the window as zite's does; a real phone width looks the same as today.
41. **Hovers vanish below 900px on a desktop.** On a half-width window Home's deck art hover (and the other wide-only hovers) stops. Hover rules key on `(hover: hover)`, not the window's width.
42. **Every hint button on the web gets the plain button hover.** The `?` buttons (panel headers, charts, the swipe screens' corner hint, the drawer title bar) sit still under a mouse.
43. **The two sites' favicons differ.** app.zwipe.net shows a gold Z on a dark square from its lone favicon.ico; zwipe.net a light Z from a full icon set. The web takes zwipe.net's set.
44. **A deck entry's hover squares its corners.** Hovering a deck on Decks shrinks its corner radius. Wanted: the same radius, a slight grow (no lift), a quick transition, and the background color change kept for now (Scotty may drop it later).
45. **The wide page heads go.** Each wide page opens with a panel restating the page's title and a one-line description: ceremony, no function. Remove them on every page, Settings included (keeping Home's hero, and any head that carries real controls or identity) and move the page's `?` into the nav bar's icon row.
46. **The footer is far too tall on both sites.** The copyright sits in line with the pills, the Scryfall credit in line with the Fan Content notice, the attributions use the footer's full width, and the padding shrinks. Shared `SiteFooter`, so zwipe.net and the web change together.
47. **About repeats the footer's credits.** On a wide window the Credits panel duplicates the footer's Scryfall and Fan Content lines; it goes there. Below 900px, with no footer, it stays.
48. **zwipe.net's manifest points at missing install icons.** zite's site.webmanifest lists /assets/icon-192.png and /assets/icon-512.png, which return 404 live because zite never bundles them (found by laptop on 43).
49. **Hand-rolled clear-× inputs.** The deck page's Quick add (and other inputs: the maybeboard's Quick add, the exceptions and picker searches, the dictionary, Import's URL) draw their own input and × instead of `components::ClearWrap`, so they sit slightly smaller and the × doesn't ease in. All go through `ClearWrap`.
50. **The deck page head's `?` is smaller than its neighbors.** Edit, Clone and Share are full icon buttons; the `?` beside them is the small info button. Same size and shape as the other three.
51. **Swiping on the web is too slow.** 24 drops a press until the leaving card's 0.32s exit ends. Keep the exit's speed but take the next press halfway through it, so swipes can come twice as fast; a held key still swipes once.
52. **The swipe surfaces' text buttons belong in the dock.** The "← Skip / ↑ Maybeboard / ↓ Undo / → Add" row under the card moves into the action dock at the bottom right (or a second dock beside it) as icon buttons, on every swipe surface.

## Done

- 24, held arrow swipes once and the exit slows to 0.32s: PR #152 into `release`.
- 13, 6, 3, changelog pages by five, privacy at page width, chart title gaps: PR #153 into `release`.
- 18, 20, wide fields capped at 32rem, dictionary letters in one row: PR #154 into `release`.
- 16, dialog padding rebalanced: PR #155 into `release`. 8 from the same PR matched zite only above 900px and was reopened.
- 9, 10, 4, 30, panel lift, deck art hover, wide deck page dividers, nav icon hover: PR #156 into `release`.
- 25, no grid tiles inside overlays: PR #157 into `release`. 19 from the same PR only restyled the old picker and was reopened.
- 5, 17, modals and drawers stay put on a wide window, Account changes as drawers: PR #158 into `release`.
- 29, featured card names in chips on every client: PR #159 into `release`.
- 31, 32, dividers on every web card list, dialog fields capped: PR #160 into `release`.
- 1, toasts bottom left on a wide window: PR #162 into `release`.
- Oracle tag catch-up, 11 descriptions (anime left blank): PR #161 into `release`.
- 12, scoping: [`../plans/web_stylesheet_split.md`](../plans/web_stylesheet_split.md), PR #163; the split itself waits on Scotty.
- 14, 15, Playstyle chip rows above panels, foldable profile: PR #164 into `release`.
- 26, the eye on every swipe surface, toggling the details beside the card on a wide window: PR #165 into `release`.
- 34, the maybeboard row cascade on the web: PR #166 into `release`.
- 22, 21, scoping: [`../plans/shared_hover_and_filter.md`](../plans/shared_hover_and_filter.md), PR #167.
- 27, zwipe.net's footer on the web through `components::SiteFooter`: PR #168 into `release`.
- 12, split phase 0, the empty `components::SCREENS_CSS` frame: PR #169 into `release`.
- 2, 23, the web's deck page drops the inline Filter row (`DeckControls` takes an optional `DeckFilter`) and gains fold-all: PR #170 into `release`. 11 won't do.
- 12, split phases 1 to 4, `main.css` moved into `components/assets/screens/`: PRs #171, #174, #175, #173 into `release`.
- 41, 42, 44, hovers at any width with a mouse, hint button hover, deck entries keep their corners: PR #185 into `release`.
- 40, the narrow layout fills a tablet-width window with zwipe.net's gutter: PR #186 into `release`.
- 7, Decks as rows on a wide window, two columns from 1400px: PR #187 into `release`.
- 8 (reopened), the image preview as large as zwipe.net's at every width: PR #188 into `release`.
- 43, the web's favicons match zwipe.net's: PR #189 into `release`.
- 19 (reopened), the deck form's pickers as the card filter's drawer and bottom sheet, the dictionary as the kit's modal: PR #182 into `release`.
- 12, split phase 5: the web stops loading `main.css`, `accordion.css` moves to `components`, scrollbars show on a wide window: PR #180 into `release`. The split is done.
- 39, the splash plays once a session and not on arrival from zwipe.net (zite mirrors it): PR #181 into `release`.
- 38, the footer spans the window as zwipe.net's does: PR #179 into `release`.
- 36, outbound links open in the same tab on zwipe.net and app.zwipe.net: PR #177 into `release`.
- 37, plain hover on every clickable chip and panel header button: PR #176 into `release`.
- 22, zite's hover stack in `components`, drawn by zite and the web: PR #172 into `release`.
