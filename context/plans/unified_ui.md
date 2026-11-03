# One UI for the app and the web

**Status: SPEC COMPLETE (2026-11-03), not built.** Every screen was reviewed with the owner, most of them in the simulator; the build follows the work order below. Nothing ships to users on zwiper-web until both clients follow it.

The app and zwiper-web drifted into two layouts: the app has a full-width bottom footer of actions on 27 surfaces and a "!" / title / "?" header, while the web has zite's nav bar, a hamburger below 900px, page heads and panels with their own footers. The goal is one layout grammar, so that finishing the web is wiring and not redesign. The app moves toward the web's chrome, and the web at phone width moves toward the app.

## Global rules

These were settled with the owner before the per-screen review. A screen breaks one only when its section below says so.

### Navigation

- The app gets the web's folded nav, the same `NavBar` from the kit: the screen's title with its `?` in the middle and the hamburger at the top right. No Z mark (owner, 2026-11-03): Home is the menu's first entry and Home's hero carries the logo, so the left corner is empty on Home and the auth screens and holds the back arrow everywhere else. zwiper-web's narrow view drops its Z mark the same way; zite keeps its own. Tapping the hamburger drops down the list of sections as buttons, exactly as the web does below 900px.
- The menu lists, signed in: Home, Decks, Playstyle, Commander maybeboard, Activity, Account, Settings and About, one list with no rule. Under them a small bottom row holds a `!` for Help & feedback and a small Log out, not full menu items.
- Account and Settings (today's Config) stay two screens. Log out is also in Account's danger zone.
- The menu looks exactly like the web's folded nav: a full-width panel dropping from the header, items stacked, the current one lit.
- The app's "!" support button leaves the header for the menu's bottom row.
- Every screen but Home and the auth screens shows a back arrow in the left corner, boxed like the hamburger, calling the screen's own Back (the previous screen, or its fixed parent where it has one). Home is always the menu's first entry. Android's system back keeps working as it does now. (Decided 2026-11-03 while building)

### Header

- The header bar is about 43pt tall, down from 58pt: 0.5rem of padding around one 1.7rem row, the corner buttons' height, so every screen is the same height. The safe area above it stays, since the Dynamic Island sits in it.
- The hamburger and the back arrow are gentle: small soft boxes (2.1 by 1.7rem, 0.4rem corners) at 0.55 opacity like the old `?` and `!` corner buttons, full strength while pressed or open, the hamburger's bars short and 2px thick. The portfolio site and zite take the same look when their navs are next touched.
- The header follows the scroll, as in X: scrolling down slides it up behind the status bar point for point until only the safe area shows, and any scroll up slides it back the same way. It never snaps on a threshold. The safe area strip keeps the page background, so content doesn't run under the clock and the island.
- The back arrow, the title and the hamburger go with it; they come back together.
- Swipe screens don't scroll, and a swipe down there is undo, so their header is always shown.

### Actions

- The app's bottom footer goes away. A screen's actions (Edit, Clone, Share, Delete and the like) sit inline in the content, like the web's panel footers.
- A screen's one or two main actions can float instead: compact buttons just above the bottom edge, always on screen, with no bar behind them and the content scrolling underneath. Decks' Create and the commander lists' Swipe and Filter float.
- The action dock is a screen's icon actions in one floating squircle bar at the bottom right (Cards first). It has no minimize button: it covers only a corner and each screen keeps room to scroll clear of it. If it ever crowds content, it shrinks with the header on scroll instead.
- Icons stand in for words only where the symbol is standard (+, −, a funnel, undo, refresh, the eye). Each icon has an accessible label, and the screen's hint names each one once.
- A form's primary button (Log in, Import) sits at the end of the form, right after the last field, and scrolls with the content. The deck form is the exception, since it runs about three screens: its Save (Edit) floats from the moment something changes, and its Create (Create deck) floats once the form is valid.
- Nothing hides behind More. Each action a More sheet holds today sits inline where it belongs, destructive ones in red. The four More sheets (Deck view, Decks, Account, Maybeboard and Suggested commanders) go away.
- No footer anywhere, overlays included (owner, 2026-11-03). A bottom sheet or picker (format, deck tags, oracle tags, card filter, printings, card details, app icon, Universes Beyond) closes with a swipe down or a close button in its title bar, and its Apply or Save is a checkmark in an action dock at its bottom right; the filter's Reset is the × on its funnel, as on Cards.
- Swipe screens (Add, Remove, Swipe select) float their actions as icons (Filter, Refresh, the eye) in the bottom-right corner just above the bottom edge. Undo stays the swipe down.

### Where overlays open

- Each overlay opens from where its trigger is, and its direction says what it is for.
- From the top: leaving the screen. The main menu drops from the header.
- From the bottom: working on this screen. Pickers, filters, card details, printings, the theme and app icon sheets rise as bottom sheets.
- In the middle: a question to answer. Confirmations (Log out, Delete) are centered dialogs.
- A navigation list never rises from the bottom, and a screen's own controls never drop from the top.

### Hints

- Every hint that names a footer button or More is rewritten for where the action now sits. The deck view's welcome hint ("More imports, exports, clones, shares or deletes the deck") is the first; the build pass checks every hint in `zwipe-ui/src/hint/`.

### Home

- Home is a dashboard, not a launcher: the hero, Color map and Featured flavor. Navigation lives in the menu, apart from Home's two shortcuts (Activity, Playstyle).

### zwiper-web at phone width

- Below 900px the web matches the app: no panels, bottom sheets for pickers, the theme and filters, and the same header with the hamburger. Wide windows keep zite's centered page and panels.

## Web follow-ups

What zwiper-web and zite change to match, kept as the app changes. Items marked kit land in orangui, so zite, zwiper-web, cairn and the portfolio site take them together.

- Kit: the gentle hamburger and back arrow (2.1 by 1.7rem, 0.4rem corners, 0.55 opacity, 2px bars) in `NavBar`, so every site's folded nav matches the app.
- Kit: a tap outside an open menu closes it everywhere, and nothing that transforms the bar may trap its backdrop.
- Kit: the theme picker opens as a bottom sheet below the 60rem fold (phone width) and stays the drop-down on wide windows (owner, 2026-11-03).
- Kit: bottom sheets lose their footers: a close button in the title bar, drag down to close, Apply or Save as a dock checkmark.
- zwiper-web narrow view: no Z mark, a back arrow on screens below a section, the same menu (Home first, Help & feedback and Log out in its footer), no page footers, and the action docks, floating Cards, Data and Danger zone panels, Settings and About as in the app.
- zwiper-web narrow view: the swipe screens' layout (pile chips on the bottom row beside the dock, only prices under the card, Synergy as the dock's link icon, the From toggle chip).
- zwiper-web: Config's route becomes `/settings`; screen titles come from zwipe-ui, so Add, Remove, Cards and Settings arrive with the next merge of main.
- zite: keeps its Z mark and marketing nav; it takes only the kit items above.

## At release

zwipe.net's guides and the privacy policy describe the shipped app, so they change on the day the release that carries this goes live, not before. Lines that name the old layout (2026-11-03):

- `zite/src/pages/guides/content.rs`: Clear skips "Open the deck's More menu" (about line 348), the maybeboard "More, then Commander maybeboard" and "More, then Clear maybeboard" (592, 623), Playstyle "tap More, then Clear skips" (685), and "Open the deck and tap More" (1008).
- `zwipe-core/src/legal/mod.rs`: the privacy policy's "you can clear this anytime from the deck's More menu".
- Any guide screenshot showing the old footer, the "!" button or Config.

## Footer inventory

Every footer in the app on 2026-11-03 and where its actions go. Where an action lands inside the content is settled in each screen's review.

| Screen | Footer today | Goes to |
|---|---|---|
| Home | Decks, Account, Config | the menu |
| About 20 screens | Back, with others | the header's back arrow |
| Decks | Back, Create, More (Commander maybeboard, Playstyle) | Create floats; More's two links move to the menu |
| Deck view | Back, Edit, Cards, More | see Deck view below |
| Account | Back, Log out, More (Delete account) | a danger zone at the very bottom holds Log out and Delete account, Delete in red; the menu has Log out too |
| Export | Back, Copy | Copy inline |
| Add, Remove, Swipe select | Back, Filter, Refresh, the eye | Filter, Refresh and the eye float as icons in the bottom-right corner |
| Cards (today's Deck cards) | Back, Add, Remove, Filter, Undo, Reset filter | the action dock, floating at the bottom right: + and −, a full-height divider, the filter funnel and undo; reset is a small squircle × on the funnel, shown only while a filter is on |
| Commander maybeboard | Back, Filter, Swipe, More (Clear maybeboard) | Swipe and Filter float; Clear maybeboard inline |
| Suggested commanders, Playstyle | Back, Swipe, More (Clear skips) | Swipe floats; Clear skips inline |
| Activity, Settings, Privacy policy, dictionary, examples | Back | the back arrow, nothing else |
| Import | Back, Import | Import at the end of the form |
| Create and Edit deck | Back, and Save or Create once there is something to save | Save and Create float |
| Login, Register, Forgot password | their form buttons | each form's own button at its end; signed out, the menu lists Log in, Create profile, Forgot password and Privacy policy, and the forms carry no links to each other |
| Overlays | Back with Save, or Back, Apply, Reset | a slimmer footer of their own |

## Work order

1. Review every screen below into this file. Done 2026-11-03.
2. Build the app's new chrome once: the nav, the hiding header, the back arrow, no footer.
3. Redo the app screen by screen.
4. Bring zwiper-web's narrow layout to match, on `feat/web`, then finish the shared-model switches there.

## Screens

The review is of the app: each screen got what it shows, where its actions go and anything that breaks a global rule. zite stays the marketing site and the web's wide layouts stay their own, so the web is fitted to these screens afterwards, not compared screen by screen now. The order follows a user's path through the app.

- [x] Login (`/login`)
- [x] Register (`/register`)
- [x] Forgot password (`/forgot-password`)
- [x] Home (`/`), below
- [x] Playstyle (a full screen Home opens in the app, its own route on the web; the menu lists it in both)
- [x] Activity (`/activity`)
- [x] Decks (`/deck`)
- [x] Create deck (`/deck/create`) and Edit deck (`/deck/update/:deck_id`), with the format, deck tag and oracle tag pickers
- [x] Deck view (`/deck/get/:deck_id`), below
- [x] Deck cards (`/deck/card/view/:deck_id`), retitled Cards
- [x] Add cards (`/deck/card/add/:deck_id`), retitled Add, with the card filter sheet and the printing sheet
- [x] Remove cards (`/deck/card/remove/:deck_id`), retitled Remove
- [x] Import and Export (`/deck/card/import/:deck_id`, `/deck/card/export/:deck_id`)
- [x] Commander maybeboard (`/deck/commander-maybeboard`) and Swipe select
- [x] Suggested commanders (`/deck/suggested-commanders`)
- [x] Oracle tag dictionary and examples
- [x] Account (`/user`)
- [x] Settings (today's Config, `/config`; the web's route becomes `/settings`)
- [x] About (new, replaces Changelog in the menu), below
- [x] Privacy policy (`/privacy`), opened from About
- [x] Update required

## Home

Reviewed 2026-11-03. Home keeps its three panels and changes only by the global rules: the new header replaces "!" and the centered title, and the footer's Decks, Account and Config go, since the menu has them.

- Hero: the ASCII logo stays, with the three stats, the Live and Everybody chips and the Activity button under it. Activity stays as a shortcut even though the menu lists it.
- Color map: unchanged, with its `?`, the color chips and the Playstyle button, also kept as a shortcut.
- Featured flavor: unchanged. Each client keeps its own Buy: the app's Buy sheet, the web's TCGplayer and Card Kingdom links.
- Nothing is added; no recent decks and no continue-swiping shortcut.

## Deck view

Reviewed 2026-11-03.

- The profile panel's header row carries Edit, Clone and Share as compact buttons: the actions on the deck as a whole.
- Buy lives in the Budget panel: its TCGplayer and Card Kingdom links.
- Cards, the screen's main action, floats at the bottom like Decks' Create.
- The less used actions sit at the bottom of the page: Import and Export, then the destructive two, Clear skips and Delete, in red. The page leaves room under them so the floating Cards never covers Delete.
- The More sheet goes away.

## Screen titles

Shorter titles, set in zwipe-ui so every client takes them: Add deck cards becomes Add, Remove deck cards becomes Remove, Deck cards becomes Cards. Add and Remove keep their filter.

Config becomes Settings everywhere a user reads it: the screen title, Home's hint and the welcome hint's bullet, the app's hint title and the web's nav label. The card filter's own Config section keeps its name, so the word means one thing. Module and type names in code stay `config`.

## About

Reviewed 2026-11-03. A new screen in the menu, beside Account and Settings. It takes over Account's About list (zwipe.net, Privacy policy, Version), so Account ends with its Account list and the danger zone.

- The current version at the top, then the changelog inline below it (today's Changelog screen folds into About).
- The zwipe.net link, the Scryfall data credit and the Fan Content notice, and the Privacy policy link.
