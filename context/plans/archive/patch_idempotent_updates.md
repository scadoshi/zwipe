# Plan: PATCH migration — verb, idempotent quantity, Opdate wire form

**Status: DONE 2026-08-12.** All five phases complete: layers 1-3 shipped with
1.7.5 (released 2026-08-10), `MIN_CLIENT_VERSION=1.7.5` raised 2026-08-12, and
the Phase 5 cleanup merged same day (PR #24, `04c1758d`) — PATCH-only endpoints,
PUT→405, legacy Opdate dialect deleted, explicit null on non-clearable
fields→422. `zcripts/metrics/errors.sql` is the post-merge canary.

**Status as of 2026-08-05 (historical, one release train, three layers):**

- ✔ **Layer 1 (verb) + Layer 2 (absolute quantity), server side — DEPLOYED
  to prod** (`6b3d17d9`, Deploy zerver green 2026-08-05). Verified: unauth
  PATCH probes on all four route families return 401 (registered, auth
  rejects — 405 would mean missing), and a live dev-client deck rename went
  out as `PATCH /api/deck/{id}` and stuck.
- ✔ **Layers 1+2, client side — BUILT** (`aa62a374`, unpushed): all seven
  endpoints send PATCH; deck-card body is `HttpPatchDeckCard` with absolute
  quantity. Owner device-testing against prod in progress (`zwiper/.env`
  temporarily points at `https://api.zwipe.net`; flip back to
  `http://127.0.0.1:3000` when done).
- ☐ **Layer 3 (Opdate wire form) — NOT started.** Server dual-accept must
  deploy BEFORE any client with the new serialization ships (details below).
- ☐ Release (1.7.5) → store rollout → raise `MIN_CLIENT_VERSION` → quiet
  days → cleanup commit (all three layers' legacy halves die together).

## Why

Three layered wire problems, fixed on one coordinated release train:

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

3. **Opdate's wire form contradicts its own docs.** `Opdate<T>`
   (`zwipe-core/src/http/helpers.rs`) is a plain derived serde enum, so the
   derive's externally-tagged encoding ships: every `Unchanged` field
   serializes as the literal string `"Unchanged"`, and a change as
   `{"Set": value}` (clear = `{"Set": null}`). Verified live 2026-08-05: a
   deck rename posted 13 `"Unchanged"` strings. The doc comments in
   `helpers.rs` (lines ~27-33) and `contracts/deck.rs` (~line 208) describe
   the intended shape — **absent = unchanged, `null` = clear, bare value =
   set** — which was never implemented. Functionally sound (both ends share
   the Rust type; `#[serde(default)]` keeps absent-field compat), but the
   wire is chatty, the docs lie, and the dialect is hostile to any non-Rust
   consumer. Layer 3 makes the documented shape real.

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

## Phase 1 — server: add PATCH routes, change nothing else ✔ DONE

Shipped `6b3d17d9`, deployed to prod 2026-08-05 (Deploy zerver green).
Verified: unauth PATCH probes → 401 on all families; authed dev-client
rename PATCH observed working live. As built:

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

## Phase 2 — client: switch to PATCH + absolute quantity ✔ BUILT

Shipped `aa62a374` (unpushed at time of writing); owner device-testing
against prod. Rides the next client release (server must already be live —
standing "server ships first, sits a day" ordering). As built:

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

## Phase 2b — Opdate wire form (Layer 3) ✔ DONE, verified on prod

Built as specced below (custom Serialize/Deserialize in helpers.rs, all 13
Opdate fields + name normalized, docs fixed, full test set incl. the legacy
fixture; 406 core + 213 zerver tests green), deployed 2026-08-05 (`39ded717`,
Deploy zerver green), and verified live from the dev client: a rename
(single-key body), a land-target set (bare value), and a land-target clear
(null) all accepted by prod. All that remains for the train is Phase 3
onward. As designed:

Makes the documented Opdate JSON shape real: **absent = unchanged, `null` =
clear, bare value = set** — replacing the derive's accidental
`"Unchanged"`-string / `{"Set": value}` dialect. Everything lives in
`zwipe-core/src/http/helpers.rs` plus the contracts that embed `Opdate`.

**Inventory first** (verify at execution): `grep -rn "Opdate<"
zwipe-core/src/http/contracts/` — as of writing, only
`HttpUpdateDeckProfile` (contracts/deck.rs, 13 fields: commander/partner/
background/signature-spell ids, format, tags, power_level, other_tags,
oracle_tags, land_target, price_target, price_target_currency). The body is
inbound-only: the server never serializes it, so the Serialize change is
client-behavior only; the Deserialize change is server-behavior only. zite
sends no updates (verify: grep zite for Opdate/update calls, expect none).

**Custom `Serialize` for `Opdate<T>`:**
- `Set(Some(v))` → the bare value; `Set(None)` → `null`.
- `Unchanged` → **serializer error, on purpose.** Unchanged fields must
  never reach the serializer: every `Opdate` field carries
  `#[serde(default, skip_serializing_if = "Opdate::is_unchanged")]`.
  Erroring loudly turns a forgotten attr into an immediate test failure,
  instead of silently emitting `null` — which the new decode would read as
  "clear this field" and quietly wipe user data. That hazard is the sharpest
  edge in this layer.
- Attr sweep: the first five profile fields currently have NO serde attrs;
  add the pair to all 13.

**Custom `Deserialize` for `Opdate<T>` — dual-accept during the window:**
- Via a `serde_json::Value` intermediate (this API is JSON-only and
  serde_json is already a core dep):
  - `"Unchanged"` string → `Unchanged` (legacy)
  - single-key `{"Set": x}` object → `Set` per x (legacy)
  - `null` → `Set(None)` (clean)
  - anything else → `T::deserialize` → `Set(Some(v))` (clean)
  - absent field → `#[serde(default)]` → `Unchanged` (both eras)
- Known ambiguities, accepted for the window and gone at cleanup: an
  `Opdate<String>` whose legitimate value is the literal string
  `"Unchanged"` would mis-decode — current string fields are controlled
  vocab (format keys, power-level slugs), so unreachable; and no `T` is an
  object with a lone `"Set"` key (Ts are Uuid/String/Vec<String>/i32/f64/
  PriceCurrency).

**Docs + tests:**
- Rewrite the now-false doc comments (`helpers.rs` ~27-33, `deck.rs` ~208)
  to describe reality, noting the legacy dialect is accepted until cleanup.
- Tests: all four inbound shapes decode correctly; clean round-trip;
  `HttpUpdateDeckProfile::builder().name(...)` serializes to exactly
  `{"name":"x"}` (nothing else on the wire); legacy fixture strings decode
  to the same struct as clean fixtures; serializing a bare `Unchanged`
  errors.

**Sequencing (the ordering hazard):** the new Serialize and Deserialize land
in one zwipe-core commit, so the next client BUILD automatically emits the
clean shape. The server with dual-accept MUST deploy before any such client
ships — same standing order, but here it's load-bearing: a clean-shape
rename against the old server 422s. After the server deploy, re-test a deck
rename from the dev client against prod and confirm the request body in the
client log is the bare-value shape.

## Phase 3 — release + wait

- Ship clients to both stores (normal build.md / play-store runbooks) —
  1.7.5 carries Layers 1–3 client-side in one build.
- Before cutting: flip `zwiper/.env` `BACKEND_URL` back to
  `http://127.0.0.1:3000` (it points at prod for migration testing); release
  builds set `BACKEND_URL=https://api.zwipe.net` explicitly per build.md.
- Wait for approval AND actual rollout (Play staged rollout to 100%).
- Do nothing server-side during this window. Both verbs and both Opdate
  dialects keep serving.

## Phase 4 — raise the version gate

- Set `MIN_CLIENT_VERSION` on the server to the Phase 2 release version;
  restart zerver. Pre-gate clients now get the blocking update screen and
  can no longer send PUT deltas.
- **Wait a day or two.** Watch traffic/logs for any PUT hits to the seven
  routes (grep access logs); expect them to fall to zero. The client error
  reporting pipeline (`zcripts/errors.sql`) is the canary for anything the
  gate broke.

## Phase 5 — cleanup (the baked-in part)

**BUILT 2026-08-10 on branch `phase5-patch-cleanup`** — full inventory below
implemented, gates green (403 core + all zerver suites incl. new PUT→405 and
null→422 integration checks; clippy -D warnings; zwiper/zite compile
untouched, constructor signatures preserved). The null policy landed as
Opdate fields on `HttpPatchDeckCard` + `HttpUpdateDeckProfile.name` with a
shared `reject_null` helper in the patch handler. **MERGE ONLY AFTER Phase 4**
(gate at 1.7.5, quiet days) — main auto-deploys, and this branch drops the
PUT routes and legacy Opdate decode that pre-gate clients still use.

The original checklist, one commit, after Phase 4 has been quiet:

- `routes.rs`: delete the seven `.put(...)` registrations.
- Delete the old deck-card PUT handler + `HttpUpdateDeckCard` + the delta
  request path + `UpdateQuantity` / `InvalidUpdateQuanity` (fixing that
  typo'd name by deletion) + the `quantity = quantity + delta` SQL arm.
- **Opdate**: drop the legacy decode arms (`"Unchanged"` string and
  `{"Set": x}`) from the custom Deserialize — clean shape only (absent /
  null / bare value). The window ambiguities disappear with them. Update the
  helpers.rs docs to drop the "legacy accepted" caveat.
- **Explicit `null` on any non-clearable field becomes a 422** (owner
  decision, 2026-08-05) — but ONLY here, after the gate guarantees every
  deployed client speaks the clean dialect; through the window nulls stay
  silently ignored exactly as today. The policy: null is "clear" on
  nullable (Opdate) fields and an error on required ones — the standard
  JSON-Merge-Patch (RFC 7396) resolution, applied uniformly.
  Inventory of affected fields at time of writing:
  - `HttpUpdateDeckProfile.name` (a deck always has a name).
  - `HttpPatchDeckCard.quantity`, `.board`, `.scryfall_data_id`, `.mvp` —
    a deck card always has all four; null currently no-ops as absent.
  - The remaining update bodies (password/username/email changes,
    preferences, hint) use bare `String`/required types where null already
    fails type decode naturally — verify with a per-endpoint null probe,
    no code expected.
  - Implementation note: rejecting null on a plain-`Option` field needs a
    null-vs-absent distinction (plain `Option` can't tell them apart) —
    e.g. a double-`Option`, or `Opdate` with a "cannot clear" validation
    arm per field.
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
- A name-only deck rename serializes to exactly `{"name":"..."}` on the
  wire; the legacy `"Unchanged"`/`{"Set": x}` dialect is no longer decoded
  anywhere; the Opdate doc comments are true.
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
- **Layer 3's ordering is load-bearing where Layers 1–2's was not**: a
  client emitting the clean Opdate shape against a server without the
  dual-accept Deserialize breaks profile updates outright. The server
  deploy carrying Phase 2b must be live before the 1.7.5 build is cut, and
  the dev-client rename re-test is the confirmation step.
- The silent-wipe hazard: an Opdate field missing its
  `skip_serializing_if` attr would serialize `Unchanged` — the custom
  Serialize errors on that instead of emitting `null` (which the new decode
  reads as "clear"). Keep that guard; it converts a subtle data-loss bug
  into a loud test failure.
- `mark_hint_shown` and friends are POST-ish edge cases verb-purists could
  argue about; out of scope — this migration standardizes on PATCH for all
  partial updates and stops there.
