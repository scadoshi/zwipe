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
| [278](278-add-tag-outline.md) | The Add title tag has no outline | merging | scotland | #413 |
| [279](279-home-create-deck.md) | Create deck button on Home's Decks panel | merging | halo | #414 |
| [280](280-panel-title-sizes.md) | Panel titles render at different sizes | fixing | thinker | - |
| [282](282-stale-bundle-after-deploy.md) | Web keeps the old bundle until a hard refresh | reported | - | - |
| [283](283-collapsible-changelog-versions.md) | Collapsible versions in every changelog | fixing | laptop | - |
| [285](285-generated-share-image.md) | Generated preview image for shared decks | approved (plan) | thinker | - |

## Waiting for live verification

[`verify-1e03520ef.md`](verify-1e03520ef.md): everything merged in the 1e03520ef deploy.
