# Phase 2: drop zwiper's two compiled fallbacks

**Status: DONE 2026-09-22, rides 1.10.2.**

Landed differently from the sketch below in one place, and better for it. `KeywordChips` lives in `zwipe-components` and is shared with zite, which provides no served catalog and correctly reads the compiled table. Deleting the fallback outright would have taken zite's reminders with it.

So the compiled table became opt-out per host instead: provide the `KeywordReminders` context and it is the only source consulted; provide nothing and the compiled table answers. zwiper provides it, zite does not, and both get what the rule wants.

That exposed a second thing. The component assumed every keyword resolves to a reminder, so every chip was tappable. A served map missing a name now yields an empty string, so chips with nothing to reveal are `disabled` rather than opening an empty panel.

Both catalogs are already served and already fetched. Both keep a compiled copy for when the fetch fails. That branch goes.

## Keyword reminders

`zwipe_core::domain::card::models::keyword` holds 343 reminder entries, and `get_keyword_reminders` has served them since the oracle-tag precedent. Two files in zwiper still fall back to the compiled table.

Chips then render without reminder text when the fetch failed, which is honest: a failed catalog fetch means the card on screen came from a search that also needed the server, so the user is already in a broken state and a possibly-stale reminder does not improve it.

The table stays in core. zerver serves from it and zite reads it for card details.

## Changelog

`session_upkeep.rs` fetches the changelog at startup and falls back to `HttpChangelog::current()` on failure. The changelog screen shows its error state instead.

This is the one with a real argument for keeping it: a compiled changelog is static content that would genuinely render offline. The argument loses anyway, because reaching that screen means opening an app whose every other surface has already failed, and the compiled copy is by definition the one that shipped with this build, so it can never show anything the user has not already got.

The data stays in core for the same reason as above, and zite's static changelog page is built from `HttpChangelog::current()` directly.

## Steps

1. Delete the fallback branch at both keyword-reminder call sites.
2. Delete the changelog fallback in `session_upkeep.rs`, including the `Failed` arm's compiled-copy path and the comment describing it.
3. Check whether the surrounding state enums still need their `Failed` variants, or whether removing the fallback collapses a three-state enum into something simpler.
4. Re-read the doc comments in `catalog_cache.rs` and `session_upkeep.rs`: several describe the fallback behavior and become wrong.

## Verification

Stop the backend, open a card with keyword chips and then the changelog screen. Neither may show content. Both should surface the failure through the existing toast rather than looking empty-but-fine.
