# Plan: bulk "resend verification emails" as a `zervice` subcommand

**Status: SPEC / ready to implement (updated 2026-07-11). Hand-off doc — another AI implements this.**

## Goal

Replace the untracked, prod-only `~/resend_verifications.py` with a `zervice` subcommand
(`zervice --resend-verifications [--send]`) that **reuses the app's real verification-email
path** (`AuthService::send_verification_email`) so a bulk backfill is byte-for-byte identical
to the app's per-user resend — no drift.

## Why a `zervice` subcommand, not a new bin

Earlier draft added a standalone `resend-verifications` bin. Decided against it: it's a rare,
run-by-hand backfill, and a second binary means another deploy artifact plus ~40 duplicated
lines of Config/DB/Resend/AuthService wiring. `zervice` already builds exactly that wiring and
already parses args (`--recategorize`). Folding it in adds **zero** new deploy surface and
reuses the service construction verbatim. Trade-off accepted: the tool lives inside the sync
binary rather than reading as its own thing.

## Why Rust, not keep the Python

The Python script **reimplements** logic that already exists in Rust: token minting, the
sha256 hash, the `email_verification_tokens` insert (24h expiry, delete-old-first), the
Resend send, and the template load. That parallel implementation **drifts** the moment the
app changes any of those (token format, expiry, template) — and then it silently sends
broken verification links. The Rust app already exposes the exact call we need, so the
subcommand just orchestrates it. Bonus: builds/deploys with the existing pipeline, no
`python3`/deps on prod, type-safe.

## Reuse these (do NOT reimplement)

- **`AuthService::send_verification_email(user_id: uuid::Uuid, to_email: &str) -> Result<(), anyhow::Error>`**
  — `zerver/src/lib/domain/auth/ports.rs` (~line 369). This is the whole per-user flow
  (mint token → store with 24h expiry, delete old → send via Resend with template). It is
  exactly what the per-user handler `zerver/src/lib/inbound/http/handlers/auth/resend_verification.rs`
  calls: `auth_service.send_verification_email(user.id, profile.email.as_ref())`.
- **Everything already wired in `zervice.rs`:** `Config::from_env`, logging setup, `Postgres::new`,
  `Resend::new`, `AuthService_::new(...)` are all constructed there already. The subcommand
  reuses them as-is — no new setup code.

## Design

In `zerver/src/bin/zervice.rs`:

- **Parse a new flag** alongside `--recategorize`:
  `let resend_verifications = args.iter().any(|a| a == "--resend-verifications");`
  and `let send = args.iter().any(|a| a == "--send");`
- **Short-circuit before the sync pipeline.** After `auth_service` is constructed (and before
  "step 1/5 card sync"), if `resend_verifications` is set, run the resend flow and `return`
  its result — the normal 5-step sync pipeline does NOT run in this mode.
- **Dry-run by default.** Fetch + count unverified users, print the count and a small sample
  of emails, **send nothing, write nothing.** Only `--send` actually sends. Prevents accidental
  mass-email.
- **Throttle** ~600 ms between sends (Resend allows ~2 req/s; the Python used 0.6s ≈ 1.6/s).
  Use `tokio::time::sleep`.
- **Per-user non-fatal:** a failed send logs an error and continues; tally successes/failures;
  **exit non-zero** if any failed (ops visibility), after attempting all. (Same non-fatal-tally
  shape zervice's sync pipeline already uses.)
- **Enumerate unverified users** with a direct runtime query (the bin has `db.pool`), no new
  `.sqlx` needed:
  ```sql
  SELECT id, email FROM users WHERE email_verified_at IS NULL ORDER BY created_at
  ```
  Use `sqlx::query_as::<_, (uuid::Uuid, String)>(...)` (runtime, not the `query!` macro).
  `email` is stored as text; `send_verification_email` takes `&str`, so pass `&email`.

Sketch — a helper called from `main` after `auth_service` is built:

```rust
// in main(), right after auth_service is constructed:
if resend_verifications {
    return resend_verifications_flow(&db, &auth_service, send).await;
}
// ...normal step 1/5 .. 5/5 sync pipeline continues below...

async fn resend_verifications_flow(
    db: &Postgres,
    auth_service: &AuthService_,
    send: bool,
) -> anyhow::Result<()> {
    let users: Vec<(uuid::Uuid, String)> = sqlx::query_as(
        "SELECT id, email FROM users WHERE email_verified_at IS NULL ORDER BY created_at",
    )
    .fetch_all(&db.pool)
    .await?;
    tracing::info!("{} unverified users", users.len());
    if !send {
        tracing::info!("dry-run (pass --send to actually send). sample:");
        for (_, email) in users.iter().take(10) {
            tracing::info!("  {email}");
        }
        return Ok(());
    }
    let mut failures = 0u32;
    for (i, (id, email)) in users.iter().enumerate() {
        match auth_service.send_verification_email(*id, email).await {
            Ok(_) => tracing::info!("[{}/{}] sent {email}", i + 1, users.len()),
            Err(e) => {
                failures += 1;
                tracing::error!("[{}/{}] FAILED {email}: {e:#}", i + 1, users.len());
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(600)).await;
    }
    if failures == 0 {
        tracing::info!("done: {} sent", users.len());
        Ok(())
    } else {
        anyhow::bail!("done with {failures} failed send(s)")
    }
}
```

Notes:
- Confirm `Postgres`'s `pool` field is accessible from the bin (it is, `db.clone()` is used in
  zervice today; if `pool` is private, add a small accessor or use an existing repo method).
- Reflect the mode in the startup log line, mirroring how `--recategorize` is surfaced, e.g.
  add a `[--resend-verifications]` / `[--send]` marker.

## Implementation steps

1. In `zerver/src/bin/zervice.rs`: parse `--resend-verifications` and `--send` flags.
2. Add the `resend_verifications_flow` helper (per sketch) and the early `return` in `main`
   after `auth_service` is constructed, before the sync pipeline.
3. **No `.sqlx` change** if you use runtime `sqlx::query_as` (recommended). Only run
   `cargo sqlx prepare --workspace` if you use the `query!` macro.
4. Verify: `SQLX_OFFLINE=true cargo check -p zerver --bins`,
   `cargo clippy -p zwipe-core -p zerver --all-targets -- -D warnings`, `cargo +nightly fmt`.
5. **Dry-run test** on dev: `cargo run --bin zervice -- --resend-verifications` → prints
   unverified count, sends/writes nothing. (Confirm the normal sync pipeline is skipped.)
6. `--send` test: exercise carefully — it emails **real** addresses via Resend. Prefer a dev
   DB seeded with a single test/unverified user, or a temporary `LIMIT 1`. Do **not**
   `--send` against prod casually.

## Deploy + cleanup

- Ships with the normal zerver deploy (nothing new to build). Run on prod like zervice:
  `set -a; source ~/zwipe/.env; set +a; ~/zwipe/zervice --resend-verifications [--send]`
  (adjust to the deployed zervice binary path).
- **After verified in prod, delete the untracked `~/resend_verifications.py`** — it's fully
  replaced. Its behavior is captured above; nothing else references it (not in git, no
  cron/systemd).

## Reference: the Python script being replaced

Lives only at `~/resend_verifications.py` on the prod box (NOT in git). Behavior: psql-fetch
unverified users → mint `secrets.token_hex(32)`, store `sha256(raw)` in
`email_verification_tokens` (24h, delete old first) → Resend API send using
`zerver/src/lib/domain/auth/email_templates/verify_email.html` → dry-run default, `--send`
to send, 0.6s throttle. Reads `DATABASE_URL` / `RESEND_API_KEY` / `RESEND_EMAIL_FROM` /
`WEB_BASE_URL` from env (no hardcoded secrets). The Rust version discards this reimplementation
in favor of `AuthService::send_verification_email`.
