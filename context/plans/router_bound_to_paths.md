# Bind the router to the shared path constants

**Status: DONE 2026-09-22.**

Landed differently from the sketch, because the sketch's test could not fail.

The plan said to oneshot each path and assert the status is not 404 or 405.
Built that, then deleted a real route to check it caught it. It did not.
`/api/card/{id}` shadows any unmatched sibling, so a deleted
`/api/card/ub-franchises` does not 404: it matches the id route and fails the
uuid parse with 422. A probe looking for 404 passes happily while the route
is gone.

What works is keying the assertion on `AUTH`, which the contract already
carries, so `FIXED_ROUTES` is `&[(Method, &str, bool)]` and generated from
the `Endpoint` impls rather than hand-written:

- **Authed routes must answer 401** without a token. The middleware decides
  that in front of the handler, and no shadow can fake it.
- **Public GETs must answer 200.** They are catalog reads needing no input.
  The featured card is allowed to 404, since a fresh test database has no
  cards; deleting its route gives 422, not 404, so the allowance costs
  nothing.
- **Public writes must answer 415.** Sent with no body, axum's Json extractor
  rejects them before any handler runs. An unrouted path answers 404.

Verified by sabotage, three times: removing an authed route (`/logout`) fails
with 404, removing a public catalog (`/artists`) fails with 422, and removing
the newest route (`/ub-franchises`) fails too. Restored after each.

**One sentence:** the server agrees with `zwipe-core`'s path constants by
coincidence; make a test say so out loud.

## The gap

`zerver/src/lib/inbound/http/routes.rs` imports **nothing** from
`zwipe_core` and hardcodes **36** route literals:

```rust
.route("/artists", get(get_artists))
.route("/oracle-tags", get(get_oracle_tags))
```

Meanwhile `GET_ARTISTS_ROUTE` lives in core, read only by the two clients.
So the "shared path constants" are shared between core and the clients, and
the server is a third party that happens to match.

## Why sharing the literals is not the fix

Axum nests: `/api/card` is declared once and `/artists` hangs off it. The
core constants are absolute (`/api/card/artists`). Making the router consume
them means either flattening the nesting, which loses the per-group
middleware layering, or splitting every const into a prefix and a leaf,
which is worse than the problem.

The answer is a test, not a refactor.

## Shape

`paths.rs` already hand-maintains a **42-element** array inside
`every_fixed_path_starts_with_a_slash`. Promote it to a public
`&[(Method, &str)]`, then in `zerver/tests` oneshot each entry against
`common::TestApp.router` and assert the status is neither 404 nor 405.

The harness already exposes `pub router: axum::Router` and drives it with
`tower::ServiceExt::oneshot`. `card_filter_parity.rs` is the same genre of
test. Roughly 30 lines.

## Current coverage, so the gap is sized honestly

44 of 66 path symbols already appear in `zerver/tests` (commit `6de59e65`),
which is real coverage, but it needs Postgres and it is incidental rather
than deliberate. Uncovered:

```
LOGOUT_ROUTE              RESEND_VERIFICATION_ROUTE      FEATURED_FLAVOR_ROUTE
PUBLIC_METRICS_ROUTE      MIN_CLIENT_VERSION_ROUTE       GET_MY_METRICS_ROUTE
MARK_HINT_SHOWN_ROUTE     SEARCH_COMMANDERS_ROUTE        GET_ORACLE_WORDS_ROUTE
CARD_ROUTE                GET_COMMANDER_MAYBEBOARD_ROUTE GET_UB_FRANCHISES_ROUTE
CLEAR_COMMANDER_MAYBEBOARD_ROUTE
share_deck_route          get_shared_deck_route          import_archidekt_deck_route
clear_deck_suppressions_route  get_deck_tokens_route     delete_deck_route
update_deck_route         delete_deck_card_route
```

`CREATE_DECK_ROUTE` and `GET_DECK_PROFILES_ROUTE` both alias `DECK_ROUTE`,
which is covered, so ignore those two. `GET_UB_FRANCHISES_ROUTE` is new as of
2026-09-22 and was not in the reassessment's list.

## The honest caveat

The array is hand-maintained, so it drifts exactly the way the router does.
Promoting it does not fix that, it just makes one hand-maintained list do two
jobs instead of one.

The `public_get!` macro could accumulate its members automatically, which
covers the easy two thirds without discipline. Worth considering, but a macro
that builds a registry is a bigger idea than the test it feeds, so do the
plain version first and see whether it drifts in practice.

## Verification

Delete a `.route(...)` line locally and confirm the new test fails. A test
that cannot fail is the failure mode here.
