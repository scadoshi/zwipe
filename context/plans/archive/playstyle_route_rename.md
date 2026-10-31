# Playstyle route rename

**Done.** The commit that moved this plan into `archive/` removed `/api/deck/play-style`; only `/api/deck/playstyle` answers now.

Started 2026-10-27. Playstyle is one word in the product, so its API path is too: `GET /api/deck/play-style` becomes `GET /api/deck/playstyle`. Apps already in the stores call the old path, so the server answers both until no supported app needs the old one, then the old one goes.

## Steps

1. **Serve both, call the new one (done 2026-10-27, on main with 1.16.0's Activity work).** zerver routes `/api/deck/playstyle` and `/api/deck/play-style` to the same handler. zwipe-core's `PLAYSTYLE_ROUTE` is the new path and the app calls it; `LEGACY_PLAY_STYLE_ROUTE` names the old one and sits in `FIXED_ROUTES`, so `route_coverage` proves it still answers, and `the_first_spelling_still_answers_with_the_summary` checks both return the same body. Activity's endpoint, briefly `/api/deck/playstyle/activity` before any app called it, moved to `/api/user/activity` when Activity became its own screen, so it never had a hyphenated form.
2. **Ship 1.16.0 (pending).** The first app on the new path. Until it is live on both stores, nothing else moves.
3. **Raise the client floor to 1.16.0 (pending).** Once 1.16.0 has been out long enough that few live sessions still report an older version, set `MIN_CLIENT_VERSION=1.16.0` on the box. Check live sessions by version first, the way the 1.10.2 floor was checked (see the README's 2026-10 entries). Older apps then land on the update screen instead of calling a route that is about to vanish.
4. **Merge the sunset branch (pending, waits on step 3).** Branch `sunset-play-style-route` is pushed and holds the finished removal: the `/play-style` route and its comment out of `zerver/src/lib/inbound/http/routes.rs`, `LEGACY_PLAY_STYLE_ROUTE` out of `zwipe-core/src/http/paths.rs` and `FIXED_ROUTES`, and the legacy test out of the activity tests (since renamed `zerver/tests/activity.rs`, so the rebase moves that hunk). Rebase it on main, run the full gate, and merge. Pushing to main deploys zerver.

## When it's done

The same commit deleted this plan's todo entry and moved this file to `plans/archive/`. Note the sunset, with its date, in the README's running log when it merges.
