# 323. overview.sql section 16 reads lockout columns that no longer exist

State: fixing | Owner: thinker | PR: #490 | Build seen: prod, 2026-11-09

## Report
Running `zcripts/metrics/overview.sql` against prod stops in section 16 with `column "lockout_until" does not exist`.

## Fix
Branch nits/thinker-323. The columns weren't renamed. Migration `20261022120000` dropped `users.failed_login_attempts`, `last_failed_at` and `lockout_until`, because the 2026-10-10 security audit moved the login lockout into zerver's memory, per address. No table records lockouts now, so section 16 drops the query that read them and keeps the audit-log churn, with a comment saying where lockouts live. The report stays read-only.

Checked against a freshly migrated scratch database. The old report fails with prod's exact error; the new one runs every section to the end. Every other script in `zcripts/metrics/` runs cleanly against the current schema too.
