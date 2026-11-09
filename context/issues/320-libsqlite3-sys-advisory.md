# 320. Security audit fails on RUSTSEC-2026-0459 (libsqlite3-sys)

State: fixing | Owner: scotland | PR: - | Build seen: release 3a627a7c1

## Report
Composer, 2026-11-09: the Security audit workflow fails on RUSTSEC-2026-0459, libsqlite3-sys 0.30.1 (C SQLite CVE-2026-119375), fixed in 0.39.0 and later.

## Repro
1. `cargo audit` on release: one vulnerability, libsqlite3-sys 0.30.1, RUSTSEC-2026-0459.

## Finding
- libsqlite3-sys is in Cargo.lock only because Cargo locks every optional dependency. sqlx 0.9.0 (the latest release) has an optional sqlx-sqlite 0.9.0 that requires libsqlite3-sys >=0.30.1, <0.38.
- zerver enables sqlx's postgres driver alone. `cargo tree -i libsqlite3-sys --target all -e normal,build` prints nothing: no SQLite code is compiled or shipped.
- No bump reaches the fix: `cargo update -p libsqlite3-sys --precise 0.39.1` fails on sqlx-sqlite's <0.38 bound. sqlx's main branch allows <0.40 but is unreleased.

## Fix
Option A, the composer's call. `.cargo/audit.toml` ignores RUSTSEC-2026-0459 with the reason and a note to drop it on the next sqlx release. audit.yml gains a "SQLite stays out of the build" step that fails if libsqlite3-sys ever enters the build graph, so the ignore cannot go stale silently. No code or behavior change.
