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
| [315](315-add-pile-flashes-stale-card.md) | Add's pile flashes a stale card after applying a filter | merged | thinker | #501 |
| [323](323-overview-lockout-columns.md) | overview.sql section 16 reads lockout columns that no longer exist | merged | thinker | #490 |
| [326](326-printings-blank-tiles.md) | Printings shows blank cards until their images load | fixing | thinker | - |
| [327](327-cancel-pending-email-change.md) | A pending email change can't be cancelled | fixing | scotland | #507 |

## Waiting for live verification

Next deploy: [`verify-next.md`](verify-next.md), renamed to `verify-<sha>.md` when it deploys.

[`verify-bb566fc4b.md`](verify-bb566fc4b.md): everything in the bb566fc4b deploy. Earlier lists: [verify-aeda394a4.md](verify-aeda394a4.md), [verify-1f354453d.md](verify-1f354453d.md), [verify-69f38270e.md](verify-69f38270e.md), [verify-02102ab5e.md](verify-02102ab5e.md), [verify-1d7053e88.md](verify-1d7053e88.md), [verify-37d09dbf9.md](verify-37d09dbf9.md), [verify-1e03520ef.md](verify-1e03520ef.md).
