# Integration tests — cover the untested server half

**Status: PLANNED (2026-07-06). Not started. Server-only, no deploy risk,
buildable in slices anytime.**

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
