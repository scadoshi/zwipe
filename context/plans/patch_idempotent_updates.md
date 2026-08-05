# Plan: PATCH verb migration + idempotent deck-card quantity

**Status (2026-08-04): Phase 1 BUILT, awaiting deploy.** Server-side PATCH
routes + the absolute-quantity deck-card contract are implemented (all seven
routes answer PATCH; deck-card PATCH is a new handler/`HttpPatchDeckCard`/
`UpdateDeckCard::patch` path; repo gains a `quantity = $qty` arm). PUT
untouched. Gate green locally (clippy -D warnings, nightly fmt, full test
suite incl. DB integration). Next: push → prod deploy → verify per Phase 1's
last bullet, then Phase 2 (client) rides the next release train.

## Why

Two layered problems, fixed in one coordinated move:

1. **Wrong verb.** Every update endpoint uses PUT for partial updates
   (`None` fields mean "leave alone"). PUT promises "full replacement,
   idempotent"; ours are PATCH-shaped. Mostly cosmetic — except:
2. **Non-idempotent quantity.** `HttpUpdateDeckCard.update_quantity` is a
   *delta* (`Some(-1)` = decrement). Replaying the request double-applies.
   Nothing in the stack auto-retries today, but any future retry layer
   (client middleware, proxy) that trusts HTTP semantics would corrupt deck
   quantities — and since the 2026-08-04 debounce, each call carries a whole
   burst, so a double-apply is a bigger error than it used to be.

End state: PATCH-only update routes; the deck-card body carries an
**absolute quantity** ("set to 3"), making the request safely replayable.

**Accepted semantic change:** two devices editing the same card concurrently
currently *merge* deltas (+1 and +1 → +2); absolute quantity is
last-writer-wins. For a single-user deck editor LWW is the saner semantic —
note it and move on.

## Current inventory (verified 2026-08-04)

PUT routes (`zerver/src/lib/inbound/http/routes.rs`):

| Line | Route | Handler | Body idempotent today? |
|------|-------|---------|------------------------|
| 410 | `/user/password` | `change_password` | yes (sets) |
| 417 | `/user/username` | `change_username` | yes |
| 424 | `/user/email` | `change_email` | yes |
| 436 | `/user/preferences` | `update_preferences` | yes |
| 437 | `/user/hint` | `mark_hint_shown` | yes |
| 480 | `/deck/{id}` | `update_deck_profile` | yes (Opdate sets) |
| 501 | `/deck/{id}/card/{card_id}` | `update_deck_card` | **no — delta qty** |

Key code:

- Contract: `HttpUpdateDeckCard` in `zwipe-core/src/http/contracts/deck_card.rs`
  (`update_quantity: Option<i32>` delta, `board`, `scryfall_data_id`, `mvp`;
  constructors `new`, `with_mvp`, `with_printing`).
- Domain: `UpdateQuantity` newtype (delta, rejects 0) in
  `zwipe-core/src/domain/deck/models/quantity.rs`. `Quantity` (absolute,
  validated ≥1) already exists beside it.
- SQL: `zerver/src/lib/outbound/sqlx/deck/mod.rs:459` — QueryBuilder pushes
  `quantity = quantity + $delta`. Runtime QueryBuilder, so **no
  `cargo sqlx prepare` needed** at any phase.
- Client verb: each module in `zwiper/src/lib/outbound/client/` builds its
  own reqwest call — grep `\.put(` to sweep them all.
- Client delta call sites: `screens/deck/card/view.rs` (debounced flush in
  `change_quantity` — the only true delta user), plus `add.rs`, `remove.rs`,
  `deck/view.rs` and `view.rs` non-qty uses (`with_mvp`, `with_printing`,
  board moves) which are already "set" semantics.
- Version gate: `MIN_CLIENT_VERSION` env var on the server
  (`zerver/src/lib/config.rs:41`), served unauthenticated at
  `/api/client/min-version`; clients hard-block below it (`UpdateRequired`
  screen). Precedent: the Phase 5S legacy-commander cleanup behind the
  1.7.0 floor.

## Phase 1 — server: add PATCH routes, change nothing else

- `routes.rs`: every table row above gains `.patch(...)` beside its
  `.put(...)` (axum `MethodRouter` chains: `put(h).patch(h)`). Six of the
  seven share one handler for both verbs — the body is already idempotent.
- **Deck card gets a new handler + contract**, not an alias:
  - New `HttpPatchDeckCard` in zwipe-core contracts: `quantity: Option<i32>`
    (**absolute**, validated through `Quantity` — server rejects <1; delete
    stays an explicit DELETE, exactly as today), `board`, `scryfall_data_id`,
    `mvp`. Mirror the `new`/`with_mvp`/`with_printing` constructors.
  - New domain request variant flowing `Quantity` (absolute) instead of
    `UpdateQuantity` (delta); repository sets `quantity = $qty` instead of
    incrementing. Reuse everything else in the update path.
  - Route: `patch(patch_deck_card)` beside the existing
    `put(update_deck_card)` at routes.rs:501. The old PUT handler is
    **untouched** — shipped clients keep working.
- Tests: contract round-trip in zwipe-core; handler test in zerver proving
  PATCH sets absolute qty and PUT still applies deltas (both live at once).
- Gate locally per commit_guidelines (nightly fmt, clippy `-D warnings` on
  zwipe-core+zerver, tests), then push → prod deploys. **Old clients are
  unaffected by design; verify with a 1.7.x client against prod.**

## Phase 2 — client: switch to PATCH + absolute quantity

Rides the next client release (server must already be live — standing
"server ships first, sits a day" ordering).

- Sweep `zwiper/src/lib/outbound/client/` for `.put(` → `.patch(` on all
  seven endpoints (reqwest has `.patch`). zite is read-only — verify with
  the same grep, expect no hits.
- Deck-card client fn sends `HttpPatchDeckCard`:
  - `view.rs` debounced flush: send `baseline + net` as the absolute target
    (equals the optimistic qty). Rollback on failure simplifies — restore
    `baseline` locally, or even just re-send (it's idempotent now).
  - Exit flush in `use_drop`: same conversion.
  - `with_mvp` / `with_printing` / board-move / `add.rs` / `remove.rs`
    sites: mechanical type swap, semantics already "set".
- Nothing else in the app changes; no user-visible behavior difference.
  Changelog: dev-note tier at most.
- Full local pass: rapid-tap burst posts one PATCH with the final absolute
  qty; kill-and-retry a request (airplane mode) and confirm re-sending is
  harmless.

## Phase 3 — release + wait

- Ship clients to both stores (normal build.md / play-store runbooks).
- Wait for approval AND actual rollout (Play staged rollout to 100%).
- Do nothing server-side during this window. Both verbs keep serving.

## Phase 4 — raise the version gate

- Set `MIN_CLIENT_VERSION` on the server to the Phase 2 release version;
  restart zerver. Pre-gate clients now get the blocking update screen and
  can no longer send PUT deltas.
- **Wait a day or two.** Watch traffic/logs for any PUT hits to the seven
  routes (grep access logs); expect them to fall to zero. The client error
  reporting pipeline (`zcripts/errors.sql`) is the canary for anything the
  gate broke.

## Phase 5 — cleanup (the baked-in part)

One commit, after Phase 4 has been quiet:

- `routes.rs`: delete the seven `.put(...)` registrations.
- Delete the old deck-card PUT handler + `HttpUpdateDeckCard` + the delta
  request path + `UpdateQuantity` / `InvalidUpdateQuanity` (fixing that
  typo'd name by deletion) + the `quantity = quantity + delta` SQL arm.
- Optionally rename `HttpPatchDeckCard` → `HttpUpdateDeckCard` for
  continuity; if renamed, sweep zwiper imports in the same commit (core and
  clients live in one workspace — atomic).
- Update tests that exercised PUT/delta; delete the dual-verb test.
- Docs sweep: grep `context/` for PUT references to these routes
  (architecture/api notes, this plan's status line → DONE).
- Gate locally, push, confirm deploy green, spot-check a 4xx/405 on PUT
  from curl and a healthy PATCH from the shipped client.

## Done criteria

- All seven update endpoints answer PATCH only; PUT returns 405.
- Deck-card quantity is absolute end to end; no delta types remain in the
  workspace (`grep -r UpdateQuantity` returns nothing).
- Shipped-store clients ≥ the gate version; gate raised; two quiet days.
- This plan moved to `context/plans/archive/` with status DONE.

## Risks / notes

- **The gate is the safety net, not the deploy order.** Nothing breaks old
  clients until Phase 4 — if anything smells wrong, hold the gate and both
  verbs coexist indefinitely at zero cost.
- Phase 4 locks out users who haven't updated; that's the same trade
  accepted for the 1.7.0 floor. Keep the Phase 3→4 window generous if store
  rollout is slow.
- Do not add client-side auto-retry middleware before Phase 2 lands — the
  whole point is that PUT deltas are not retry-safe.
- `mark_hint_shown` and friends are POST-ish edge cases verb-purists could
  argue about; out of scope — this migration standardizes on PATCH for all
  partial updates and stops there.
