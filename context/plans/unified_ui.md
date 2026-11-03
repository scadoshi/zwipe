# One UI for the app and the web

**Status: SPEC COMPLETE (2026-11-03), not built.** Every screen was reviewed with the owner, most of them in the simulator; the build follows the work order below. Nothing ships to users on zwiper-web until both clients follow it.

The app and zwiper-web drifted into two layouts: the app has a full-width bottom footer of actions on 27 surfaces and a "!" / title / "?" header, while the web has zite's nav bar, a hamburger below 900px, page heads and panels with their own footers. The goal is one layout grammar, so that finishing the web is wiring and not redesign. The app moves toward the web's chrome, and the web at phone width moves toward the app.

## Global rules

These were settled with the owner before the per-screen review. A screen breaks one only when its section below says so.

### Navigation

- The app gets the web's folded nav, the same `NavBar` from the kit: the Z mark at the left, the screen's title with its `?` in the middle, and the hamburger at the top right. Tapping the hamburger drops down the list of sections as buttons, exactly as the web does below 900px.
- The menu lists, signed in, in two groups split by a thin rule: Decks, Playstyle, Commander maybeboard and Activity; then Account, Settings and About. Under them a small bottom row holds a `!` for Help & feedback and a small Log out, not full menu items.
- Account and Settings (today's Config) stay two screens. The Z mark goes Home. Log out is also in Account's danger zone.
- The menu looks exactly like the web's folded nav: a full-width panel dropping from the header, items stacked, the current one lit.
- The app's "!" support button leaves the header for the menu's bottom row.
- On a screen below a top-level section (deck view, Add, Edit, Import, the dictionary and so on), the Z mark's spot becomes a back arrow. Android's system back keeps working as it does now.

### Header

- The header hides on scroll down, leaving only the safe area (the notch), and comes back on any scroll up, as in Safari.
- The back arrow, the title and the hamburger go with it; they come back together.
- Swipe screens don't scroll, and a swipe down there is undo, so their header is always shown.

### Actions

- The app's bottom footer goes away. A screen's actions (Edit, Clone, Share, Delete and the like) sit inline in the content, like the web's panel footers.
- A screen's one or two main actions can float instead: compact buttons just above the bottom edge, always on screen, with no bar behind them and the content scrolling underneath. Decks' Create and the commander lists' Swipe and Filter float.
- Icons stand in for words only where the symbol is standard (+, −, a funnel, undo, refresh, the eye). Each icon has an accessible label, and the screen's hint names each one once.
- A form's primary button (Log in, Import) sits at the end of the form, right after the last field, and scrolls with the content. The deck form is the exception, since it runs about three screens: its Save (Edit) floats from the moment something changes, and its Create (Create deck) floats once the form is valid.
- Nothing hides behind More. Each action a More sheet holds today sits inline where it belongs, destructive ones in red. The four More sheets (Deck view, Decks, Account, Maybeboard and Suggested commanders) go away.
- Overlays (bottom sheets and pickers: format, deck tags, oracle tags, card filter, printings, app icon, Universes Beyond) keep their own footer, made slimmer: the same compact buttons with less padding. Their Back becomes a close button or a swipe down.
- Swipe screens (Add, Remove, Swipe select) float their actions as icons (Filter, Refresh, the eye) in the bottom-right corner just above the bottom edge. Undo stays the swipe down.

### Hints

- Every hint that names a footer button or More is rewritten for where the action now sits. The deck view's welcome hint ("More imports, exports, clones, shares or deletes the deck") is the first; the build pass checks every hint in `zwipe-ui/src/hint/`.

### Home

- Home is a dashboard, not a launcher: the hero, Color map and Featured flavor. Navigation lives in the menu, apart from Home's two shortcuts (Activity, Playstyle).

### zwiper-web at phone width

- Below 900px the web matches the app: no panels, bottom sheets for pickers, the theme and filters, and the same header with the hamburger. Wide windows keep zite's centered page and panels.

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
| Cards (today's Deck cards) | Back, Add, Remove, Filter, Undo, Reset filter | a floating icon pill, bottom center: + and −, a divider, the filter funnel and undo; reset is a small × on the funnel, shown only while a filter is on |
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
