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
| [386](386-card-proxy.md) | One no-image card for every image spot | fixing | laptop | #589 |
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
| [389](389-profile-dock.md) | A deck's Edit, Clone and Share sit in the Profile header on a phone | fixing | thinker | - |

## Waiting for live verification

Next deploy: [`verify-next.md`](verify-next.md), renamed to `verify-<sha>.md` when it deploys.

[`verify-bb566fc4b.md`](verify-bb566fc4b.md): everything in the bb566fc4b deploy. Earlier lists: [verify-aeda394a4.md](verify-aeda394a4.md), [verify-1f354453d.md](verify-1f354453d.md), [verify-69f38270e.md](verify-69f38270e.md), [verify-02102ab5e.md](verify-02102ab5e.md), [verify-1d7053e88.md](verify-1d7053e88.md), [verify-37d09dbf9.md](verify-37d09dbf9.md), [verify-1e03520ef.md](verify-1e03520ef.md).
