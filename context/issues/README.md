# Issues

Every bug, nit and small feature lives here, one file per issue, from the moment it's reported until it's verified on the live site. The rules for moving an issue along are in [`../development/factory.md`](../development/factory.md).

Numbers continue from [`../progress/web_nits.md`](../progress/web_nits.md), which is closed to new entries and kept as the record of nits 1 to 279.

## States

| State | Means | Who moves it there |
|---|---|---|
| reported | Someone saw it. The file says what, where, on which build. | composer, from Scotty, Surfer, Droid or a worker |
| approved | Scotty said fix it (or it's a clear bug inside scope he already approved). | composer |
| reproduced | Exact steps confirmed on a named build, by a worker or QA. | worker or QA |
| fixing | A worker owns it on a named branch. | composer |
| merged | The PR is in `release`, with a test that failed before the fix. | composer |
| verified | Surfer or Droid checked it on the live deploy and it passes. | composer, from QA's report |
| closed | Verified, or skipped / not a bug with Scotty's word. | composer |

An issue that fails verification goes back to `reproduced` with what QA saw.

## File

Name: `NNN-short-slug.md`. Keep it short:

```markdown
# NNN. Title

State: reported | Owner: - | PR: - | Build seen: <sha>

## Report
What happened, where, at what width or on which device. Who saw it.

## Repro
Numbered steps on a named build.

## Fix
Branch, PR, the failing test, what changed.

## Verification
Who checked, on which live build, what they saw.
```

## Open

| # | Title | State | Owner | PR |
|---|---|---|---|---|
| [308](308-share-image-size-for-whatsapp.md) | Share preview images may be too big for WhatsApp | parked | - | - |
| [313](313-decks-list-price-chip.md) | Decks list shows no price per deck | merged | scotland | #465 |
| [315](315-add-pile-flashes-stale-card.md) | Add's pile flashes a stale card after applying a filter | reproduced | scotland | #501, #516, #530 |
| [323](323-overview-lockout-columns.md) | overview.sql section 16 reads lockout columns that no longer exist | merged | thinker | #490 |
| [325](325-drawer-bottom-border-clipped.md) | The card details drawer's bottom border vanishes at 100% zoom | fixing | scotland | #505 |
| [326](326-printings-blank-tiles.md) | Printings shows blank cards until their images load | reproduced | scotland | #503, #516, #531 |
| [327](327-cancel-pending-email-change.md) | A pending email change can't be cancelled | fixing | scotland | #507, #508 |
| [328](328-generic-pip-ring.md) | Pip rings split on expanded and hovered card rows | fixing | thinker | - |
| [329](329-buttons-without-hover.md) | Buttons and other clickables without a hover on desktop web | fixing | halo | orangui#30, orangui#31, #510, orangui#32, #519 |
| [330](330-undo-remove-drops-star.md) | Undo of a remove brings the card back without its star | fixing | thinker | - |
| [331](331-stats-panel-padding.md) | Deck stats panels let their content touch the rules | fixing | scotland | #511 |
| [332](332-empty-error-toast.md) | A failed request can toast an empty red box | fixing | halo | - |
| [333](333-printings-stuck-on-slow-network.md) | Printings sits empty after a slow fetch, with no error or retry | fixing | halo | #515 |
| [334](334-deck-page-missing-name.md) | The web deck page shows no deck name | fixing | thinker | - |
| [336](336-search-no-timeout.md) | A deck card search that never answers leaves Add on the skeleton | fixing | thinker | - |
| [335](335-selector-drawer-space.md) | Selector drawers waste space above the chips | fixing | laptop | #520 |
| [341](341-printings-failed-images-never-retry.md) | Printings that failed on a slow network stay set-name cards | fixing | scotland | #525 |
| [340](340-toasts-over-load-error.md) | Stacked error toasts cover the "Couldn't load" box | fixing | laptop | #522, #528 |
| [337](337-zite-tap-highlight.md) | Taps on zwipe.net flash Chrome's blue highlight | fixing | thinker | - |
| [338](338-link-hover-inconsistent.md) | Text links hover differently across the three sites | fixing | thinker | - |
| [342](342-no-link-underlines.md) | No underlines on links | fixing | thinker | - |
| [344](344-meld-support.md) | Meld cards show their melded card | fixing | scotland | #534 |
| [343](343-web-keyboard-shortcuts.md) | Keyboard shortcuts on the web | fixing | thinker | - |
| [346](346-chart-entrance.md) | Charts animate in when they open | fixing | laptop | #539 |
| [345](345-narrow-swipe-chip.md) | Add's last-action chip is missing below the full nav | fixing | scotland | #537 |
| [347](347-swipe-fly-off-hidden.md) | Add's left, right and up swipes play no fly-off | fixing | scotland | #542, #545 |
| [352](352-back-loop.md) | Back loops between Create deck and Decks | fixing | laptop | #553 |
| [348](348-github-style-share-links.md) | GitHub-style shared deck links, "by <username>", the deal on the web deck page | fixing | thinker | - |
| [351](351-flyoff-clipped.md) | The commander swipe's fly-off is cut off before the window's edge | fixing | scotland | #550 |
| [355](355-full-width-sites.md) | The sites leave a wide window's sides empty | fixing | scotland | zite #557 |
| [350](350-deck-page-wide.md) | The web deck page leaves a wide window's sides empty | fixing | scotland | #547 |
| [353](353-stack-cart.md) | A stack has no way to buy its cards | fixing | thinker | - |
| [354](354-one-buy-dialog.md) | Buying is a different control in every spot, and the wide deck page hides it | fixing | thinker | - |
| [358](358-share-link-deck-name.md) | The Share dialog's link names every deck "deck" | fixing | thinker | - |
| [372](372-stacks-deal-in.md) | Stacks appear all at once | fixing | thinker | - |
| [402](402-logo-font.md) | zwipe.net's logos look heavier than the app's: the block characters fall back to Menlo | fixing | laptop | #616 |
| [400](400-video-skeleton.md) | zwipe.net's videos paint half-drawn, with a white block, while they load | fixing | laptop | #612 |
| [403](403-home-colors-height.md) | Home's Colors panel stops short of Decks beside it | fixing | thinker | #617 |
| [399](399-deck-list-slow-firefox.md) | The deck page's List is slow in Firefox: opening a row, folding, Art | fixing | scotland | #608 |
| [398](398-maybeboard-sort.md) | The Commander maybeboard has no ordering | fixing | laptop | #601 |
| [397](397-list-only.md) | Playstyle, Activity and the Commander maybeboard lists are List only | fixing | laptop | #599 |
| [393](393-proxy-strips.md) | No-image stack strips wrap and peek, and Home's deck tiles show a broken image | fixing | laptop | #596 |
| [391](391-proxy-fit.md) | The no-image card clips its facts row in the app's Featured card, and thumbnails break names mid-word | fixing | laptop | #594 |
| [388](388-section-stacks.md) | Playstyle and Activity stacks sit in panels, and share one View setting | fixing | laptop | #591 |
| [386](386-card-proxy.md) | One no-image card for every image spot | fixing | laptop | #589 |
| [380](380-settle-everywhere.md) | Lists and groups snap when a control reorders, adds or removes them | fixing | laptop | #603 |
| [381](381-sheets-keep-content.md) | Sheets and drawers slide out empty, and the Filter accordion drops shut | fixing | laptop | #605 |
| [382](382-drawer-crossfade.md) | The Stacks drawer snaps to another card | fixing | laptop | #609 |
| [385](385-settle-first-frame.md) | The settle paints the final layout for one frame first | fixing | laptop | #586 |
| [376](376-nav-logo-size.md) | The nav logo is a different size on each site | fixing | thinker | - |
| [366](366-swipe-select-footer.md) | Suggested commanders has no footer | fixing | thinker | - |
| [361](361-list-group-cart.md) | The List's card groups have no cart | fixing | thinker | - |
| [362](362-store-links-printing.md) | Store links send names only, not the printing picked | fixing | thinker | - |
| [357](357-stack-hover-shift.md) | Hovering a stack shifts its cards a pixel | fixing | thinker | - |
| [356](356-list-row-meld.md) | A meld card's list row has no Meld toggle | fixing | scotland | #555 |
| [359](359-key-map-gaps.md) | f and v gaps on Commander maybeboard and meld cards | fixing | scotland | #559 |
| [368](368-staple-decks.md) | A staple's deck count names no decks | fixing | laptop | #571 |
| [367](367-printings-browse-only.md) | Printings offers Select where nothing would save | fixing | laptop | #568 |
| [365](365-examples-keys-hint-slide.md) | Examples keys, d on every swipe screen, ? to the open modal, the card clear of its details | fixing | scotland | #567, #576 |
| [360](360-modal-entrance.md) | The wide modals pop in with no motion | fixing | laptop | #565, orangui#35 |
| [369](369-share-link-chip.md) | The Share dialog's link sits in an odd dark box | fixing | laptop | #564 |
| [364](364-list-row-art-meld.md) | A melded card's list row thumbnail and hover preview show the half | fixing | scotland | #563 |
| [363](363-row-keys-and-stacks.md) | Playstyle and Activity have no card keys and no Stacks view | fixing | laptop | #574 |
| [371](371-art-chip-stacks.md) | Art stays live while Stacks is on | fixing | laptop | #577 |
| [373](373-board-group-easing.md) | Board and Group by changes snap the deck page's cards | fixing | laptop | #578 |
| [374](374-deck-list-slow.md) | The deck page's List is slow to open a row, fold or toggle Art | fixing | scotland | #585 |
| [379](379-quick-add-width.md) | Quick add runs wider than the chip rows under it | fixing | laptop | #581 |
| [375](375-home-activity-button.md) | Home's counters have no way into Activity on the web | fixing | laptop | #580 |
| [378](378-swipe-image-fallback.md) | A failed swipe card image shows the card underneath | fixing | laptop | #583 |
| [383](383-ease-snaps.md) | Three small snaps: the nav, a label's clear and the Draw odds bars | fixing | thinker | #607, #610 |
| [395](395-import-width.md) | app.zwipe.net Import squeezes into the left of its page | fixing | thinker | #604 |
| [394](394-no-prose-caps.md) | Running text stops at 70ch inside the wide panels | fixing | thinker | #598, portfolio#49 |
| [396](396-home-rows.md) | app.zwipe.net Home puts its three panels in one row from 1920 | fixing | thinker | #602 |
| [387](387-home-column.md) | app.zwipe.net Home runs the window's width | fixing | thinker | #590 |
| [389](389-profile-dock.md) | A deck's Edit, Clone and Share sit in the Profile header on a phone | fixing | thinker | #593 |
| [377](377-keyboard-in-hint.md) | The keyboard shortcuts button sits on the action docks instead of in the hint | fixing | halo | orangui#38, orangui#39, #618 |
| [404](404-list-group-cart.md) | The app's List groups have no cart | fixing | halo | #620 |
| [405](405-web-stacks-masonry.md) | The web deck page's Stacks hold each row to its tallest stack | fixing | halo | #623 |
| [406](406-og-underline.md) | The link cards underline Magic: The Gathering | fixing | thinker | 6f6314bd3 |
| [411](411-icons-not-words.md) | Card buttons say Image, Flip, Meld, Star and Select in words | fixing | halo | nits/halo-411 |
| [412](412-star-color.md) | Stars take three different colors | fixing | halo | orangui 8626ccf, nits/halo-412 |
| [413](413-disabled-chip-light.md) | A disabled chip looks enabled in light themes | fixing | thinker | orangui 00a8954 |
| [408](408-stack-header-wrap.md) | Stack headers cut long names short, and the cart sits far from the price | fixing | halo | nits/halo-408 |
| [407](407-byline-chip.md) | The shared deck's owner sits on its own line, not as a chip | fixing | laptop | - |
| [414](414-sheet-icons-named.md) | The shortcuts sheet's icon rows show no names | fixing | halo | nits/halo-414 |
| [419](419-shared-guides.md) | Guides live only on zwipe.net | fixing | halo | release, in phases |
| [409](409-printings-skeleton.md) | Printings shows two different skeletons while it loads | fixing | laptop | issue/409-printings-skeleton |
| [410](410-printings-edges.md) | Printings fades its cards out short of the sheet's edges | fixing | laptop | issue/410-printings-edges |
| [417](417-featured-hover.md) | The deck head's featured cards don't answer the mouse | fixing | laptop | issue/417-featured-hover |
| [420](420-changelog-under-about.md) | zwipe.net's Changelog sits in the top nav | fixing | thinker | - |
| [421](421-stack-float.md) | The revealed card in Stacks sits still under an outline | fixing | laptop | issue/421-stack-float |
| [422](422-featured-acts-like-cards.md) | The deck head's featured cards only open their image | fixing | thinker | - |
| [423](423-reserved-usernames.md) | Anyone can register Zwipe's own names, and names that differ only in case | fixing | thinker | - |
| [424](424-overlay-layers.md) | Themes opens under the card details drawer | fixing | laptop | issue/424-overlay-layers, orangui 73ee2f6 |
| [425](425-zite-soft-404.md) | zwipe.net answers an unknown path with a 200, and Esc leaves the featured card's art open | fixing | scotland | - |
| [426](426-zerver-admin-clap.md) | zerver-admin parses its own arguments and reads the account as a string at lookup | fixing | thinker | - |
| [427](427-zerver-admin-commands.md) | zerver-admin has no way to inspect, sign out, re-verify, delete, lock or re-email an account | fixing | thinker | - |
| [428](428-narrow-web-stacks.md) | Narrow web's deck cards have no View chip, no Stacks and no group price or cart | fixing | laptop | feature/428-narrow-web-stacks |
| [429](429-narrow-sheet-keys-and-drag.md) | On the web at phone width, Printings ignores the arrow keys and Enter until clicked, and the filter can't be pulled down | fixing | scotland | orangui 4566a7e, fbfcc73 |
| [431](431-drawer-card-keys.md) | With a card's details drawer open on the web deck page, the card keys do nothing | fixing | scotland | orangui 3bd2f9b |
| [432](432-star-undo.md) | Undo on the deck page skips starring and unstarring a card | fixing | thinker | - |
| [433](433-web-app-404.md) | app.zwipe.net answers an unknown path with 200 | fixing | halo | release |
| [434](434-featured-first-tap.md) | On a phone, a featured card floats on the first tap instead of opening, and the strip clips it | fixing | thinker | - |
| [439](439-web-first-paint-lock.md) | A fresh web app tab can sit on the List skeleton for about ten seconds | fixing | halo | release |
| [435](435-softer-float.md) | The Stacks float pops up and sways like a metronome, and a tap reveal snaps open | fixing | laptop | issue/435-softer-float |
| [430](430-one-deck-route.md) | A deck's Profile and Cards are separate screens on narrow web and in the app, and the shared deck splits them with a chip row | fixing | laptop | issue/430-one-deck-route |
| [436](436-deck-page-fixes.md) | Deck page rough edges: Fix to isn't undoable, Select closes the drawer, the move toast hides the count, Quick add lands the newest printing | fixing | thinker | - |
| [438](438-one-profile.md) | At phone width the shared deck's Profile isn't the owner's, and a shared deck hides its warnings and Export | fixing | laptop | issue/438-one-profile |
| [441](441-signed-out-nav.md) | The web app's signed-out nav lists Forgot password | fixing | thinker | - |
| [437](437-examples-one-dock.md) | At phone width the Examples controls wrap, the eye on a row of its own | fixing | scotland | - |
| [442](442-held-deck.md) | Profile and Cards share one up-to-date deck so a star or a Warnings fix never reads a stale copy | fixing | laptop | issue/held-deck |

## Waiting for live verification

Next deploy: [`verify-next.md`](verify-next.md), renamed to `verify-<sha>.md` when it deploys.

[`verify-bb566fc4b.md`](verify-bb566fc4b.md): everything in the bb566fc4b deploy. Earlier lists: [verify-aeda394a4.md](verify-aeda394a4.md), [verify-1f354453d.md](verify-1f354453d.md), [verify-69f38270e.md](verify-69f38270e.md), [verify-02102ab5e.md](verify-02102ab5e.md), [verify-1d7053e88.md](verify-1d7053e88.md), [verify-37d09dbf9.md](verify-37d09dbf9.md), [verify-1e03520ef.md](verify-1e03520ef.md).
