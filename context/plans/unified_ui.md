# One UI for the app and the web

**Status: built, in review on devices, ships as 1.18.0.** Every screen was reviewed with the owner on 2026-11-03, most in the simulator, and the app follows this file. zwiper-web's narrow layout follows on `feat/web`; nothing ships to users there until both clients match.

The app and zwiper-web had drifted into two layouts: the app a full-width bottom footer of actions and a "!" / title / "?" header, the web zite's nav bar with a hamburger below 900px, page heads and panels with their own footers. This file is the one layout grammar, so finishing the web is wiring and not redesign. The app took the web's chrome, and the web at phone width takes the app's.

## Global rules

Settled with the owner before the per-screen review. A screen breaks one only when its section below says so.

### Navigation

- Every screen opens with the kit's `NavBar`, folded: the screen's title with its `?` in the middle and the hamburger at the top right. No Z mark (owner, 2026-11-03): Home is in the menu and Home's hero carries the logo, so the left corner is empty on Home and the auth screens and holds the back arrow everywhere else. zwiper-web's and zite's narrow views drop their Z mark the same way; zite keeps it on wide windows.
- The menu, signed in, lists four sections as text rows: Decks, Playstyle, Activity, Commander maybeboard. The current section is lit; a deck's inner screens light Decks, the privacy policy lights About.
- Under the list one footer row holds, left to right, the `!` for Help & feedback, then Home, Account, Settings and About as icons (house, person, cog, circled i), then the theme pair (owner, 2026-11-03): a palette that opens the Themes sheet and folds the menu as it rises, and the kit's sun or moon naming the mode in use, which flips Dark/Light and saves at once through the same path as Settings' Theme row. A small Log out sits at the row's right and asks first. No theme row over the icons: one band.
- Signed out, the menu lists Log in, Create profile and Forgot password, and the footer holds only the `!`.
- Account and Settings (the `config` screen) stay two screens. Log out is also in Account's Danger zone.
- The menu looks exactly like the web's folded nav: a full-width panel dropping from the header, items stacked, the current one lit. Its open state is app-wide, so a tap on an entry only navigates and the next screen's header folds the panel over its own content.
- Every screen but Home and the auth screens shows a back arrow in the left corner, boxed like the hamburger, calling the screen's own Back (the previous screen, or its fixed parent where it has one). Android's system back and the iOS edge swipe close an open menu before they leave the screen. (Decided 2026-11-03 while building)

### Header

- The header bar is about 43pt tall: 0.5rem of padding around one 1.7rem row, the corner buttons' height, so every screen is the same height. The safe area above it stays, since the Dynamic Island sits in it.
- The hamburger and the back arrow are gentle: small soft boxes (2.1 by 1.7rem, 0.4rem corners) at 0.55 opacity, full strength while pressed or open, the hamburger's bars short and 2px thick. The portfolio site and zite take the same look when their navs are next touched.
- The header follows the scroll, as in X: scrolling down slides it up behind the status bar point for point until only the safe area shows, and any scroll up slides it back the same way. It never snaps on a threshold. The safe area strip keeps the page background, so content does not run under the clock and the island.
- The back arrow, the title and the hamburger go with it and come back together.
- Swipe screens do not scroll, and a swipe down there is undo, so their header is always shown.

### Actions

- No screen has a bottom bar. A screen's actions sit inline in the content, like the web's panel footers, or float.
- The action dock is a screen's icon actions in one floating squircle bar at the bottom right. It has no minimize button: it covers only a corner and each screen keeps room to scroll clear of it. Floating buttons rise above the keyboard.
- A screen's one or two main actions float in that dock. Decks' Create is a + there; the maybeboard's Filter floats as a chip. A list's own action (the commander lists' Swipe) sits in that list's header, beside its title, so it says what it acts on (owner, 2026-11-03).
- Icons stand in for words only where the symbol is standard (+, −, a funnel, undo, refresh, a checkmark, the eye, two sheets for Copy, a paper plane for Send, chain links for Synergy). Each icon has an accessible label, and the screen's hint names each one once.
- A form's primary action sits in the dock as an icon: Import's import, Export's Copy. The deck form's Save (Edit) is a checkmark in the dock from the moment something changes, and its Create (Create deck) once the form is valid.
- The three sign-in forms have no action dock (owner, 2026-11-03): the submit is the kit's full-width button under the last field, Log in, Create profile or Send, and the links to the other sign-in screens are chips in one centered row beneath it, so a new user finds Create profile and Forgot password without opening the menu. Login links to Create profile and Forgot password, Create profile to Back to login, Forgot password to Back to login and Create profile. Return in a field submits.
- Nothing hides behind More. Each action a More sheet held sits inline where it belongs, destructive ones in red in a Danger zone panel. There is no More sheet.
- No footer anywhere, overlays included (owner, 2026-11-03). A bottom sheet or picker (format, deck tags, oracle tags, card filter, printings, card details, app icon, Universes Beyond) closes with a swipe down or an × in its corner, and its Apply or Save is a checkmark in an action dock at its bottom right.
- The card filter sheet's Reset is an undo icon beside that checkmark in the sheet's own dock: it stages the default, Apply commits it, closing undoes it. On Cards the dock's funnel also carries a small × while a filter is on, which resets at once. Open question for the owner: the plan called for the × on the funnel only, and the sheet's undo icon shipped alongside it; keep both, or drop the ×.
- Swipe screens (Add, Remove, Swipe select, Oracle tag examples) float their actions as icons (Filter, Refresh, the eye) in the dock. Undo stays the swipe down. Add's Synergy is the link icon beside the filter.

### Where overlays open

- Each overlay opens from where its trigger is, and its direction says what it is for.
- From the top: leaving the screen. The main menu drops from the header.
- From the bottom: working on this screen. Pickers, filters, card details, printings, the theme and app icon sheets rise as bottom sheets.
- In the middle: a question to answer. Confirmations (Log out, Delete) are centered dialogs.
- A navigation list never rises from the bottom, and a screen's own controls never drop from the top.

### Hints

- Every hint names the control as it sits now: the dock's icons, Colors, the + button, Danger zone. No hint names a footer button or More. `zwipe-ui/src/hint/` holds the copy.

### Home

- Home is a dashboard, not a launcher: the hero, Decks, Colors and Featured flavor. Navigation lives in the menu, apart from Home's shortcuts (Activity in the hero, All decks and Playstyle in the panel headers).

### zwiper-web at phone width

- Below 900px the web matches the app: no panels, bottom sheets for pickers, the theme and filters, and the same header with the hamburger. Wide windows keep zite's centered page and panels.

## Web follow-ups

What zwiper-web and zite change to match. Items marked kit land in orangui, so zite, zwiper-web, cairn and the portfolio site take them together.

- Kit: the gentle hamburger and back arrow (2.1 by 1.7rem, 0.4rem corners, 0.55 opacity, 2px bars) in `NavBar`, so every site's folded nav matches the app.
- Kit: a tap outside an open menu closes it everywhere, and nothing that transforms the bar may trap its backdrop.
- Kit: the theme picker opens as a bottom sheet below the 60rem fold (phone width) and stays the drop-down on wide windows (owner, 2026-11-03).
- Kit: bottom sheets have no footers: an × in the corner, drag down to close, Apply or Save as a dock checkmark.
- zwiper-web narrow view: no Z mark, a back arrow on screens below a section, the same menu (the four sections, the footer icons, the theme pair, Help & feedback and Log out), no page footers, and the action docks, floating Cards, Data and Danger zone panels, Settings and About as in the app.
- zwiper-web narrow view: the swipe screens' layout (pile chips on the bottom row beside the dock, only prices under the card, Synergy as the dock's link icon, the From toggle chip).
- zwiper-web: Config's route becomes `/settings`; screen titles come from zwipe-ui, so Add, Remove, Cards and Settings arrive with the next merge of main.
- zite: at phone width its nav drops the Z mark, as the app's does (owner, 2026-11-03); wide windows keep it as the marketing site's brand. Otherwise it takes the kit items above.

## At release

zwipe.net's guides and the privacy policy describe the shipped app, so they change on the day 1.18.0 goes live, not before. Lines that still name the old layout (2026-11-03):

- `zite/src/pages/guides/content.rs`: "Clear skips in the deck's More menu" (about line 249), "Open the deck's More menu and tap Clear skips" (348), the maybeboard "More, then Commander maybeboard" and "More, then Clear maybeboard" (592, 623), Playstyle "tap More, then Clear skips" (685), and "Open the deck and tap More" (1008).
- `zwipe-core/src/legal/mod.rs`: the privacy policy's "you can clear this anytime from the deck's More menu".
- Any guide screenshot showing the old footer, the "!" button or Config.

## Where each screen's actions sit

| Screen | Actions |
|---|---|
| Home | none of its own; Activity in the hero, All decks (or Create a deck) and Playstyle in the panel headers |
| Decks | Create as a + in the dock |
| Deck view | see Deck view below |
| Account | a Danger zone panel at the very bottom with Log out and Delete account, Delete in red |
| Export | Copy as an icon in the dock |
| Import | Import as an icon in the dock |
| Add, Remove, Swipe select, Oracle tag examples | Filter, Refresh and the eye as icons in the dock; Add adds Synergy as the link icon |
| Cards | the dock: + and −, a divider, the funnel (with its × while a filter is on) and undo |
| Commander maybeboard | Swipe in the list's header; Filter floats as a chip; Clear maybeboard in a Danger zone panel |
| Playstyle | Swipe in the list's header; Clear skips in a Danger zone panel |
| Activity, Settings, Privacy policy, dictionary | the back arrow, nothing else |
| Create and Edit deck | Save or Create as a checkmark in the dock once there is something to save |
| Login, Register, Forgot password | the full-width submit under the form, link chips in one row beneath |
| Overlays | an × in the corner or a drag down to close; Apply or Save as a dock checkmark; the filter sheet adds its Reset undo |

## Screens

Each screen below was reviewed with the owner on 2026-11-03: what it shows, where its actions sit and anything that breaks a global rule. zite stays the marketing site and the web's wide layouts stay their own. The order follows a user's path through the app.

- [x] Login (`/login`)
- [x] Register (`/register`)
- [x] Forgot password (`/forgot-password`)
- [x] Home (`/`), below
- [x] Playstyle (a full screen Home opens in the app, its own route on the web; the menu lists it in both), below
- [x] Activity (`/activity`)
- [x] Decks (`/deck`)
- [x] Create deck (`/deck/create`) and Edit deck (`/deck/update/:deck_id`), with the format, deck tag and oracle tag pickers
- [x] Deck view (`/deck/get/:deck_id`), below
- [x] Cards (`/deck/card/view/:deck_id`)
- [x] Add (`/deck/card/add/:deck_id`), with the card filter sheet and the printing sheet
- [x] Remove (`/deck/card/remove/:deck_id`)
- [x] Import and Export (`/deck/card/import/:deck_id`, `/deck/card/export/:deck_id`)
- [x] Commander maybeboard (`/deck/commander-maybeboard`) and Swipe select
- [x] Suggested commanders (`/deck/suggested-commanders`)
- [x] Oracle tag dictionary and examples
- [x] Account (`/user`)
- [x] Settings (`/config` in the app; the web's route becomes `/settings`)
- [x] About (`/about`, replaces Changelog in the menu), below
- [x] Privacy policy (`/privacy`), opened from About
- [x] Update required

## Sign-in screens

Login, Register and Forgot password have an empty left corner and the signed-out menu on the right. Login's hero is Home's. The submit is the full-width button under the last field; the chips beneath link to the other screens. Create profile and Forgot password return to Log in in place of themselves once they succeed, with the username (or the email) filled in, so Back does not reopen the finished form.

## Home

Home keeps the hero and Featured flavor, holds a Decks panel and a Colors panel between them, and otherwise follows the global rules: the header is the menu's, with no `!` and no footer.

- Hero: the ASCII logo, the three stats, the Live and Everybody chips and the Activity button under it. Activity stays as a shortcut even though the menu lists it.
- Decks (owner, 2026-11-03): a panel between the hero and Colors. Its header centers Decks and the deck count chip as one unit, with an All decks chip alone in the right corner. Under it, the three most recently updated decks as tiles, three across: the commander's art crop in a squircle (a filled squircle with the deck's pips without one), then the name chip and the muted format chip under the image, the name on one line with an ellipsis; a tap anywhere on the tile opens the deck. With no deck the count chip hides, the corner chip is Create a deck and one muted line sits under the header. Home fetches the deck list and caches it above the router like the summary; while the fetch is out three tiles ghost, and a failed fetch with nothing held shows a muted line under the header with its All decks chip. No continue-swiping shortcut.
- Colors (owner, 2026-11-03): the panel between Decks and Featured flavor, in the Decks panel's style. Its header centers Colors and its `?`, with the Playstyle chip alone in the right corner. Under it six compact bars in WUBRG order and colorless last, each with its mana pip at the left, its fill in the color's mana variable (a muted neutral for colorless) over a visible track, the largest count at full width and the count at the bar's end. With no deck the same no-decks line sits under the header, Playstyle chip kept. While the fetch is out the whole panel ghosts. The tiles map itself lives on Playstyle.
- Featured flavor: unchanged. Each client keeps its own Buy: the app's Buy sheet, the web's TCGplayer and Card Kingdom links.

## Playstyle

Reviewed 2026-11-03 (owner). The color territory map is the screen's first panel, titled Color map with its `?`, open on entry above Summary, Card roles, Oracle tags and the commander list. It reads the same playstyle summary Home's Colors panel does, with no request of its own beyond the summary the screen already fetches. The map's skeleton stays with the screen's other ghosts. Swipe sits in the commander list's header; Clear skips is a Danger zone panel.

## Deck view

Reviewed 2026-11-03.

- The profile panel's header row carries Edit, Clone and Share as icons: the actions on the deck as a whole.
- Buy lives in the Budget panel: a Buy label with the TCGplayer and Card Kingdom chips.
- Cards, the screen's main action, floats in the dock at the bottom right as a chip.
- The less used actions sit at the bottom of the page in two panels of rows: Data holds Import and Export, Danger zone holds Clear skips and Delete, titled and buttoned in red. The page leaves room under them so the floating Cards never covers Delete.
- Back after a delete, or a deck the server no longer has, lands on Decks, not the gone deck.
- Distributions shows the top five card roles and the top five oracle tags as charts, the rest behind a See more chip that eases open to See less. Draw odds shows its top five groups the same way and names the opening hand turn 0.

## Screen titles

Shorter titles, set in zwipe-ui so every client takes them: Add, Remove and Cards for the add, remove and deck cards screens. Add and Remove keep their filter.

Config is Settings everywhere a user reads it: the screen title, Home's hint and the welcome hint's bullet, the app's hint title and the web's nav label. The card filter's own Config section keeps its name, so the word means one thing. Module and type names in code stay `config`.

## About

Reviewed 2026-11-03. The screen in the menu's footer beside Account and Settings. It holds what was Account's About list (zwipe.net, Privacy policy, Version), so Account ends with its Account list and the Danger zone.

- A top panel of rows: Version with its `?`, Website with its `?` and the zwipe.net chip, Privacy policy with a View chip.
- A Credits panel: the Scryfall data credit and the Fan Content notice. The privacy policy does not repeat the notice.
- A Changelog panel: the shared changelog as rows, the version filter first, then a release per row, from the copy fetched at startup; a skeleton while it is out, a muted note when it failed.
