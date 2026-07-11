## Commit Guidelines

- Concise, one-line messages (multi-line only when many changes)
- Group related files logically
- No emojis
- Use `git diff` to understand changes before committing
- **Never** include AI-agent signatures in your commits
    - Example: "Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
    - Never commit with attribution metadata

## CI: how your commits get checked (run these BEFORE you push)

Pushing to `main` triggers the deploy workflows (`Deploy zerver`, `Deploy zite`), and **a
push to `main` auto-deploys prod.** Each workflow gates its `deploy` job on `test` + `lint`
(`.github/workflows/deploy-zerver.yml`). A red check means the deploy is **silently skipped**
— prod stays on the old build. Reproduce the gate locally first:

### 1. Format with **nightly** — the one that bites
CI runs `cargo +nightly fmt --check`, **nightly and workspace-wide**. `rustfmt.toml` enables
`imports_granularity = "Crate"`, an *unstable* option, so **stable `cargo fmt` silently skips
it** — your code passes locally but fails CI and the deploy is skipped. Because the check is
workspace-wide, *any* crate's bad formatting (even zite/zwiper) blocks the zerver deploy.

```bash
cargo +nightly fmt        # NOT `cargo fmt` — stable can't apply the Crate imports rule
```

### 2. Clippy — the exact CI command, warnings are errors
```bash
cargo clippy -p zwipe-core -p zerver --all-targets -- -D warnings
```

### 3. Tests
```bash
set -a; source zerver/.env; set +a        # integration tests (#[sqlx::test]) need DATABASE_URL
cargo test -p zwipe-core -p zerver         # what CI runs (postgres:16 service, SQLX_OFFLINE=true)
```

### 4. SQLx offline data (only if you touched a query macro)
If you added/changed a `query!` / `query_as!` / `query_scalar!` **macro**, the deploy verifies
offline data (`cargo sqlx prepare --workspace --check`) and fails if it's stale. Regenerate +
commit from the workspace root:
```bash
cargo sqlx prepare --workspace            # commit the resulting .sqlx/ changes
```
Runtime queries (`sqlx::query`, `QueryBuilder`) don't use `.sqlx` — no prepare needed.

### Deploy stage
Once `test` + `lint` pass, `deploy` runs the migrations (`zerver/migrations`) against **prod**
then builds + ships. So migrations must be additive/forward-only, and **you should confirm
with the owner before pushing to `main`** — it deploys production.
