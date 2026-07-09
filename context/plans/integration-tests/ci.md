# Integration tests — CI workflow

New file `.github/workflows/test.yml`. Runs on push + PR to main. Does NOT
gate the deploy workflows (decision 2026-07-06: server patches keep their
ship-in-minutes latency; CI is a signal, not a lock — revisit if a red main
ever burns us).

```yaml
name: test
on:
  push:
    branches: [main]
  pull_request:

jobs:
  test:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:16        # match the VPS major version
        env:
          POSTGRES_PASSWORD: postgres
        ports: ["5432:5432"]
        options: >-
          --health-cmd pg_isready --health-interval 5s
          --health-timeout 5s --health-retries 10
    env:
      DATABASE_URL: postgres://postgres:postgres@localhost:5432/postgres
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo test -p zwipe-core -p zerver
```

## Notes

- **`-p zwipe-core -p zerver`, never `--workspace`**: on a headless Linux
  runner, compiling zwiper pulls Dioxus desktop's GTK stack (glib-sys) and
  fails — the same trap that broke the 2026-07-05 deploy verify step
  (`operations/infrastructure/cicd.md`). zwiper/zite tests remain a local
  `cargo test --workspace` concern on macOS.
- `#[sqlx::test]` creates and drops its per-test databases against the
  service container's superuser — no extra grants needed.
- The `.sqlx` offline cache is irrelevant here (tests compile against the
  live service DB via `DATABASE_URL`); do NOT set `SQLX_OFFLINE=true` in
  this workflow, or macro-checked queries stop validating against a real
  schema.
- `Swatinem/rust-cache` keeps the build under a few minutes after the
  first run. Public repo → Actions minutes are free.
- Migrations run per-test via `#[sqlx::test]`; there is no separate
  migrate step.
- Add a branch-protection status check for `test` on main **later, only if
  wanted** — that is the "gate the deploy" option we deliberately skipped.

## Local parity

Same command devs already run: `cargo test --workspace` (macOS compiles
everything) or `cargo test -p zerver` for the server slice. Both need
`DATABASE_URL` exported (`set -a; source zerver/.env; set +a`).
