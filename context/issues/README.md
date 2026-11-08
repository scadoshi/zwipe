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
| [278](278-add-tag-outline.md) | The Add title tag has no outline | verified | scotland | #413 |
| [279](279-home-create-deck.md) | Create deck button on Home's Decks panel | verified | halo | #414 |
| [280](280-panel-title-sizes.md) | Panel titles render at different sizes | verified | thinker | - |
| [282](282-stale-bundle-after-deploy.md) | Web keeps the old bundle until a hard refresh | verified | scotland | #454 |
| [283](283-collapsible-changelog-versions.md) | Collapsible versions in every changelog | verified | laptop | #417, portfolio#20 |
| [285](285-generated-share-image.md) | Generated preview image for shared decks | verified | thinker | #424, #425, #426 |
| [286](286-portfolio-orangui-line-anchors.md) | Portfolio orangui source line ranges are stale | verified | laptop | portfolio#21 |
| [265](265-partner-swipe-stack.md) | Partner Swipe stack still offers illegal partners | verified | laptop | #419, #430, #437 |
| [287](287-split-1-18-changelog.md) | Split the 1.18 changelog into logical patches | verified | laptop | #420 |
| [288](288-stack-refreshed-on-failed-refresh.md) | Add says Stack refreshed when the refresh failed | verified | laptop | #421 |
| [289](289-top-color-sr-text.md) | Top color pip reads Mono-red to screen readers | verified | laptop | #422 |
| [290](290-format-sheet-keeps-draft.md) | Format sheet keeps its search after Discard; Keep is unclear | verified | laptop | #423, #428 |
| [291](291-logout-revokes-all-sessions.md) | Logging out on one device logs out every device | verified | halo | #427 |
| [292](292-changelog-api-browser-cache.md) | Web app's About shows an old changelog for hours after a deploy | verified | halo | #431 |
| [293](293-empty-swipe-pile-skeleton.md) | An empty Swipe pile stays on a skeleton card | verified | laptop | #432 |
| [294](294-portfolio-zwipe-web-app.md) | Portfolio's Zwipe page leaves out the web app | verified | thinker | portfolio#22, #434 |
| [295](295-changelog-whole-header-toggles.md) | Changelog versions only toggle from the caret | verified | scotland | #436, portfolio#23 |
| [296](296-collapsed-group-tag-hover-border.md) | A collapsed group's tag loses its bottom border color on hover | verified | laptop | #433 |
| [297](297-web-decks-tag-filters-collapsible.md) | Web Decks shows every deck tag filter up front | verified | laptop | #435 |
| [298](298-web-header-at-150-zoom.md) | The web app's header breaks at 150% zoom | verified | laptop | #440 |
| [299](299-leave-guard-dead-after-about.md) | Create deck's leave guard stops working after visiting About | verified | scotland | #442 |
| [300](300-share-image-version-not-bumped.md) | Share image URL keeps its old ?v= after the image layout changes | verified | thinker | #438 |
| [301](301-form-dirty-after-discard.md) | Edit deck still asks to leave after a picker change was discarded | verified | laptop | #439 |
| [302](302-add-details-overlaps-card.md) | Add's card details panel overlaps the swipe card at 900 to 1060px | verified | laptop | #440 |
| [303](303-web-unstyled-flash-on-load.md) | The web app flashes unstyled on a hard refresh | verified | thinker | #445 |
| [304](304-home-stats-chip-gap.md) | Web Home's stat chips sit far below the counters | verified | laptop | #443 |
| [305](305-home-decks-create-label.md) | Home's Decks panel "+" should read "Create deck" | verified | laptop | #443 |
| [306](306-openframe-reuse-shell.md) | About and Privacy draw a second app frame when signed in | verified | halo | #444 |
| [307](307-about-cards-skip-zwiper-web.md) | zwipe.net About's crate cards leave out zwiper-web | verified | laptop | #447 |
| [308](308-share-image-size-for-whatsapp.md) | Share preview images may be too big for WhatsApp | parked | - | - |
| [309](309-robots-blocks-share-unfurls.md) | X shows no card for shared deck links (robots.txt) | verified | laptop | #449 |
| [310](310-guide-screenshots-refresh.md) | zwipe.net guides use old screenshots | verified | scotland | #456 |
| [311](311-guides-filter-chip-and-clear.md) | zwipe.net guides filter: plain label and a hand-rolled × | verified | thinker | #455 |
| [312](312-three-guides-without-screenshots.md) | Three guides have no screenshots | approved | droid (queued) | - |

## Waiting for live verification

[`verify-1f354453d.md`](verify-1f354453d.md): everything in the 1f354453d deploy. Earlier lists: [verify-69f38270e.md](verify-69f38270e.md), [verify-02102ab5e.md](verify-02102ab5e.md), [verify-1d7053e88.md](verify-1d7053e88.md), [verify-37d09dbf9.md](verify-37d09dbf9.md), [verify-1e03520ef.md](verify-1e03520ef.md).
