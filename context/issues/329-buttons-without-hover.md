# 329. Buttons and other clickables without a hover on desktop web

State: fixing | Owner: halo | PR: orangui#30, this PR, portfolio repin | Build seen: release 1.18.16

## Report
Scotty, 2026-11-09, desktop web. Several buttons give no answer under a mouse. On a deck page: the Warnings panel's Add cards and Import (outlined in the warning color), Data's Import and Export, the Danger zone's Clear and Delete (outlined red), the charts' See more, and every button in the floating action dock (+, -, filter, undo, edit, duplicate, share). Then Settings (Theme's Dark and Change, Pips, Swipe confirmations, Universes Beyond, the ? buttons), Account (Change, Log out, Delete), the Themes sheet's rows (on the web app, zwipe.net and the portfolio), the Filter sheet's fold headers and its dock, the - / + steppers, text fields, and inline text links. Scotty's rule for the fix: anything clickable on desktop web gets a simple hover; disabled things do not. The Exact / Any / Range mode toggles stay as they are.

## Cause
The kit's base button styles had no `:hover`. orangui's `.btn`, `.btn-xs`, `.btn-action`, `.stepper-btn`, `.action-dock-btn`, the theme picker's `.pref-row` and the collapse's `.collapse-header` had press styles at most. The web app had patched hover onto buttons one context at a time in `web.css` (a panel header's buttons, the nav row, chips, the ? buttons, a dialog's buttons), so a button anywhere else, such as a settings row, a chart's foot or the dock, stayed still. Danger buttons were left out of every one of those rules.

## Fix
At the root, in orangui (orangui#30), one `(hover: hover)` section reusing the kit's two hover looks, with `:focus-visible` matching:
- Bordered buttons and steppers: the accent on the text and border.
- The action dock's buttons: the accent on the glyph.
- Destructive buttons: their red kept, with the red's 12% tint behind them (the tint a status tag wears).
- Theme rows and fold headers: the `--border-primary` fill a dropdown row takes. It reads apart from a selected theme's accent border.
- Inline text links: the accent and a firmer underline, in `:where()` so a site's own link rule wins.
- Text fields: the border brightens a step. Focus is unchanged.
- Disabled things get none, and a selected or active one keeps its look.

zwipe then repins orangui (53aec89) and adds what is its own: the Filter sheet's fold headers (`.accordion-trigger`), Home's deck tiles, the back arrow, and the links whose class rules outrank the kit's (the footer note, About's credits, the privacy page) in components, and in `web.css` the warning fixes' tint (they carry the warning color inline) and the logo. The portfolio repins orangui for the theme rows and links.

A DevTools sweep of 12 web screens and sheets, forcing `:hover` on everything with a pointer cursor (transitions off), went from 5 to 55 unanswering clickables per screen to only the intended ones: the active nav item, selected chips, and pieces whose hover lights a child (a deck tile, the changelog caret inside its lit header). Rest and hover crops of Deck, Config, Account, the Themes sheet, the Filter sheet, Home and About are on the PR.

## Follow-up after live verify (21ca16a04)

The verify failed one family: inline links with no class, like Home's "Magic: The Gathering" tagline link. orangui's `:where(a:not([class])):hover` had zero specificity, so zwiper-web's resting `.web-hero-tagline a` (0,1,1) won and the link had no hover. orangui#32 makes it `:where(a):not([class]):hover` (0,2,0). That beats a resting rule like `.x a` and still loses to a site's own `.x a:hover` (0,2,1). Injected into the live app.zwipe.net, zwipe.net and scadoshi.com pages, it gave the one bare link a hover and changed no site's hover color.

Scotty also asked for no hover fill on changelog release headers, and for one card look everywhere, the app About panel's: caret, version and date on the left, the tag at the right end, a full-width rule under the header. orangui#32 adds `collapse-header-quiet`, which drops the fill, and `ReleaseCard`, which is that card. zwipe's app and zite changelogs and the portfolio's Cairn changelog all draw it now. Filter sheet fold headers keep their fill.
