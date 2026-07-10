# Integration tests — cover the untested server half

**Status: IN PROGRESS (started 2026-07-09). Server-only, no deploy risk, built
in slices.**

## ▶ Resume here (next session)

**Done:** slices 1–3 + Slice 4 (HTTP + core [repo]) → **21 integration tests green**
(`tests/auth_flows.rs` ×3, `tests/deck_flows.rs` ×5, `tests/card_serving.rs` ×6,
`tests/deck_cards.rs` ×3, `tests/repo_card.rs` ×4) + CI gating live (see below) + a
404-IDOR server fix.

**The card fixture builder is BUILT** (`tests/common/mod.rs`): `card(name)` (chainable
setters: `.mono/.colors/.color_identity/.cmc/.type_line/.mana_cost/.power/.toughness/
.keywords/.produced_mana/.rarity/.edhrec_rank/.usd/.set/.legal/.commander_legal/
.oracle/.with_ids`, plus `.id()/.oracle_id()/.name()` accessors) + `seed_cards(pool,
&[CardFixture])` (raw unchecked INSERT of all NOT NULL cols + the `card_profiles`
mate, then REFRESHes `latest_cards` + `card_signal_rollup`) + `refresh_card_views`.
Rows round-trip cleanly through `DatabaseScryfallData::try_from`.

**Run the tests:** `set -a; source zerver/.env; set +a; cargo test -p zerver`
(CI parity: prepend `SQLX_OFFLINE=true`). Needs local Postgres up.

**`tests/repo_card.rs` is BUILT** (construct `Postgres { pool }`, call
`CardRepository` directly): synergy ordering (scored-desc vs UNSCORED_ANCHOR via
`deck_id=None` = pure score, no shuffle), `exclude_oracle_ids` drop, the
**NULL-`oracle_id` deck-aware-shuffle regression** (card survives + determinism),
and `card_signal_rollup` math (net = Σ(added + 0.5·maybed − removed), shown = Σ shown;
seed `commander_card_signal` then `refresh_card_views`). Key facts learned:
`deck_id=None` → pure score order (deterministic, testable); `deck_id=Some` → banded
shuffle (BAND_SIZE=25, so <25 cards = one band ordered by the `hashtext` shuffle,
NOT score); `W_SIGNAL=0.15` but the signal term is 0 with no rollup rows;
`WILDCARD_SLOTS=1`/`DEEP_POOL_FLOOR=500` so small sets get no wildcard splice.

**Next task — Slice 5 (metrics + user + health):** usage-batch ingest → `user_week_signal`
/ `user_week_facet_signal` rows + deck_skips ride-along, `POST /api/metrics/anonymous`
(3 kinds accepted, garbage rejected, no auth), change username/email/password (password
re-auth), delete-account cascade, health endpoint, last-active debounce. Then Slice 6
(auth edges via `FakeEmailSender` token). **Still open in Slice 4** (lower priority,
grab if convenient): deck-aware serve **suppression** exclusion + land-target auto-stop
(HTTP), band-boundary/different-deck shuffle + clone card-copy ([repo]).

**Harness map:** `zerver/tests/common/mod.rs` = `TestApp` (real router via
`build_router`, driven with `tower::oneshot`), `FakeEmailSender`, the card fixtures
above, and helpers `register`/`verify_email`/`post`/`get`/`put`/`delete`. New areas
go in `tests/<area>.rs` starting with `mod common;` + `use common::{TestApp, ...};`.

**Gotchas already learned (save time):**
- Collection route is `POST/GET /api/deck` **and** deck-card create
  `POST /api/deck/{id}/card` — **no trailing slash** (the nested `/` leaf resolves
  without one; the `/{id}` and `/{sid}` paths are fine).
- Deck **update** JSON: the `Opdate` fields `commander_id`, `partner_commander_id`,
  `background_id`, `signature_spell_id`, `format` lack `#[serde(default)]`, so they
  must be present — send the string `"Unchanged"` for each.
- Fresh registered users are **unverified** (deck cap 1 / card cap 100); call
  `app.verify_email(&uid)` to lift to 20 / 250.
- In harness helpers use **unchecked `sqlx::query(...)`** (not the `query!` macro)
  so test-only queries don't need `.sqlx` offline entries.
- `latest_cards` / `card_signal_rollup` are created `WITH NO DATA` — **must be
  refreshed before any card query even with zero rows** (`seed_cards` does this;
  `seed_cards(&pool, &[])` populates the empty views so a 404-lookup test works).
- Card wire shapes: `GET /api/card/{id}` → `{card_profile, scryfall_data{...}}`;
  `colors`/`color_identity` serialize as short-name arrays (`["R"]`); `rarity` is
  lowercase long name (`"rare"`); `update_deck_card.update_quantity` is a **delta**,
  not an absolute; `DeckCard.board` serializes lowercase (`"deck"`/`"maybeboard"`).

## Progress tracker (update as slices land)

- [x] **Slice 1 — harness + auth flow** (`harness.md`): **DONE 2026-07-09.** dev-deps,
  `build_router` extraction, `FakeEmailSender`, `TestApp` (oneshot + governor
  ConnectInfo), `tests/auth_flows.rs` — 3 tests green (register → authed GET → login
  → refresh; wrong-password 401; no-token 401). Run: `set -a; source zerver/.env; set +a; cargo test -p zerver`.
- [x] **Slice 2 — CI workflow** (`ci.md`): **DONE 2026-07-09.** `.github/workflows/test.yml`
  — push-to-main + PR, Postgres 16 service, `cargo test -p zwipe-core -p zerver`. Uses
  `SQLX_OFFLINE=true` for compile (the plan's no-offline note was wrong — the service DB
  is empty, so macro validation needs the committed `.sqlx`; `#[sqlx::test]` still uses
  `DATABASE_URL` at runtime). Exact command validated locally: 108 + 3 + 358 tests green.
  **Gating (owner call 2026-07-09):** `Test` runs on PRs; deploy-zerver + deploy-zite each
  got an inline `test` job the deploy `needs:` (keeps path filters + blocks a red deploy;
  GitHub can't make a push-triggered workflow wait on a separate one). Node-24 action bumps
  (checkout@v7, cache@v6, deploy-pages@v5) rode along.
Remaining slices in **recommended build order** (see `coverage.md` for the
per-slice case list + the endpoint coverage map = the full-system target):

- [x] **Slice 3 — deck lifecycle** — **DONE 2026-07-09.** Profile half in
  `tests/deck_flows.rs` (5 tests) + the deferred deck-**card** ops in
  `tests/deck_cards.rs` (3 tests: add/bump/remove, maybeboard placement, text import
  resolved+unresolved). Added harness helpers `put`/`delete`/`verify_email`. Still
  open at repo level: **[repo]** clone card-copy + suppression eviction (folded into
  the Slice-4 [repo] file).
- [x] **Slice 4 — card serving + repo** (highest *regression* risk) — **DONE
  2026-07-09** for the core surface. `tests/card_serving.rs` (6: get-by-id round-trip,
  404, name / cmc-range / color-identity search, requires-auth) + the `card(...)` /
  `seed_cards(...)` **fixture helper** + `tests/repo_card.rs` (4: synergy ordering,
  exclude-oracle, NULL-`oracle_id` regression, rollup math). **Deferred (optional):**
  deck-aware serve suppression exclusion + land auto-stop (HTTP), band-boundary /
  different-deck shuffle + clone card-copy ([repo]).
- [ ] **Slice 5 — metrics + user + health**: usage-batch ingest + signal rows,
  anonymous events, change username/email/password (re-auth), delete-cascade,
  health, last-active debounce.
- [ ] **Slice 6 — remaining auth edges**: verify-email + password-reset via the
  captured `FakeEmailSender` token, lockout 429, IDOR spot-check (A hits B's deck).
  Lower priority — auth already has strong unit coverage.
- [ ] **Ongoing — future features land WITH tests**: share tokens, MVPs vesting,
  wildcard slot (their plan docs specify the cases; this harness is where they run).

Each slice is standalone and committable; if we stop mid-way, this checklist +
`coverage.md`'s endpoint map mark the resume point and remaining surface.

**What this builds, in one sentence:** a real-database test suite for zerver
— HTTP-level flows driven through the actual Axum router plus repo-level
tests for the tricky SQL — with a CI workflow so tests finally run
automatically.

**Why:** an external audit (2026-07-06) confirmed the codebase's strengths
(auth, sqlx hygiene, lint discipline) but found the one real weakness: of
~517 test functions, zero cover the sqlx repositories, HTTP handlers, or
most domain services, and there is no `tests/` dir and no CI test run
anywhere. The core product surface (deck building, card serving, import,
suppressions, signal ordering) is validated only at the zwipe-core model
level; its server-side orchestration is untested. Compile-time-checked
queries catch shape errors, not logic errors — the band-shuffle NULL
`oracle_id` bug (caught only by a hand-built dev harness) is exactly the
class of regression this suite exists to catch.

## Decisions (settled 2026-07-06)

- **Harness: `#[sqlx::test]`.** Each test gets its own fresh database with
  migrations auto-applied and auto-cleanup. Zero new infra: sqlx already
  ships it, local Postgres already runs, `Postgres { pool }` is directly
  constructible (`outbound/sqlx/postgres.rs:109` — public field).
- **Both layers, fully.** HTTP-level flow tests through the real router
  (highest coverage per test: handler + middleware + service + repo + real
  SQL) *and* repo-level tests for SQL whose logic doesn't surface cleanly
  through HTTP (band shuffle determinism, suppression eviction, clone
  exclusions, rollup math).
- **CI: yes, non-gating on deploy.** New GitHub Actions workflow on
  push/PR with a Postgres service container. Deploy workflows stay
  independent — server patches keep shipping in minutes.
- **Fake email adapter.** `EmailSender` is already a port
  (`domain/email/ports.rs:10`); tests inject a capturing fake — no Resend
  API calls, and captured bodies hand tests their verification/reset tokens.

## The pieces

| Piece | Doc | Order |
|---|---|---|
| Test harness (`TestApp`, router refactor, fixtures) | [`harness.md`](harness.md) | first |
| What to test, both layers, priority-ordered | [`coverage.md`](coverage.md) | second, in slices |
| GitHub Actions test workflow | [`ci.md`](ci.md) | after the first slice is green |

## Sequencing

1. Harness + the auth HTTP flow test (proves the whole stack works
   end-to-end in a test).
2. CI workflow (lock in the green state immediately).
3. Coverage slices in `coverage.md` priority order — deck lifecycle first,
   then card serving, then the rest. Each slice is a standalone session.

Nothing here touches production code paths except the small router-extraction
refactor in `harness.md` §2 (behavior-preserving, verified by the deploy
pipeline's existing build).
