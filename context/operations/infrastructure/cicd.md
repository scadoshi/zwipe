# CI/CD — GitHub Actions Deploy

On every push to `main` that touches backend code, a self-hosted GitHub Actions runner on
the server checks out the repo, builds `zerver` and `zervice` in place, copies the binaries
to `~/zwipe/`, and restarts the systemd service. No network tunnels, no deploy keys, no SCP.

---

## Workflow File

`.github/workflows/deploy-zerver.yml`

Triggers automatically on push to `main` when any of these paths change:
- `zerver/**`
- `.github/workflows/deploy-zerver.yml`

Also has `workflow_dispatch` for manual runs from the GitHub Actions tab.

### Tests gate the deploys

The test suite (`postgres:16` service + `cargo test -p zwipe-core -p zerver`, never
`--workspace` — zwiper's Dioxus/GTK stack won't build headless; `SQLX_OFFLINE=true`
compiles against the committed `.sqlx` while `#[sqlx::test]` migrates a fresh DB per
test via `DATABASE_URL`) runs in two places:

- **`.github/workflows/test.yml` (`Test`)** — on **pull requests only**, the
  pre-merge signal.
- **A `test` job inside `deploy-zerver.yml` and `deploy-zite.yml`** — the `deploy`
  (and zite's `build`) job `needs: test`, so **a red suite blocks the deploy**.
  This keeps the deploys' path filters (a docs-only push still won't redeploy) while
  gating on tests — GitHub can't make a `push`-triggered workflow wait on a *separate*
  workflow, hence the inline job. Push-time testing lives here, so `test.yml` stays
  PR-only (no double-run). `workflow_dispatch` still lets you force a deploy (it runs
  after the test job).

Plan/design: [`../../plans/integration-tests/`](../../plans/integration-tests/overview.md).

---

## What the Workflow Does

1. Checks out the repo
2. Installs stable Rust toolchain (cached)
3. Restores cargo cache (fast subsequent builds)
4. Runs SQLx migrations (`set -a && source ~/zwipe/.env` to export `DATABASE_URL`, then `cargo sqlx migrate run`)
5. Verifies the committed `.sqlx/` matches the just-migrated schema (`cargo sqlx prepare --workspace --check -- --workspace --exclude zwiper --exclude zite`) — fails fast with "query data is stale" instead of E0308 soup mid-build. The GUI crates are excluded because `--check` compiles crates to find their queries, and zwiper's Linux desktop deps (glib/GTK via pkg-config) don't exist on the headless VPS — this failed the first two verify runs (2026-07-06) before being scoped; only zerver has queries anyway
6. Builds `zerver` and `zervice` in release mode (`SQLX_OFFLINE=true`)
7. Stops zerver, copies binaries to `~/zwipe/`, starts zerver

No Tailscale, no SSH keys, no SCP — the runner is already on the server.
Migrations run before the build so new tables exist before the new binary starts.

---

## GitHub Configuration

### Secrets (Settings → Secrets and variables → Actions → Secrets)

| Name | Value |
|---|---|
| `TS_OAUTH_CLIENT_ID` | Tailscale OAuth client ID (kept for reference, not used in workflow) |
| `TS_OAUTH_SECRET` | Tailscale OAuth client secret (kept for reference, not used in workflow) |

### Variables (Settings → Secrets and variables → Actions → Variables)

None required for self-hosted runner deployment.

---

## Self-Hosted Runner Setup

The runner is a long-running process on the server that polls GitHub for jobs. It connects
outbound to GitHub — no inbound ports or tunnels needed. Run this setup once; after that
deploys are fully automatic.

### Step 1 — Generate a runner token

GitHub → Repository Settings → Actions → Runners → New self-hosted runner → Linux → x64

Copy the token shown on that page (valid for 1 hour).

### Step 2 — Download and configure the runner on the server

```bash
mkdir ~/actions-runner && cd ~/actions-runner

# Download — get the exact URL from the GitHub UI (version may change)
curl -o actions-runner-linux-x64.tar.gz -L \
  https://github.com/actions/runner/releases/download/v2.x.x/actions-runner-linux-x64-2.x.x.tar.gz

tar xzf actions-runner-linux-x64.tar.gz

# Configure — paste the token from Step 1 when prompted
./config.sh --url https://github.com/scadoshi/zwipe --token YOUR_TOKEN_HERE
# Accept defaults for runner name and work folder
```

### Step 3 — Install as a systemd service

```bash
sudo ./svc.sh install
sudo ./svc.sh start
sudo ./svc.sh status
```

The runner starts automatically on every boot. Check GitHub → Settings → Actions → Runners
to confirm it shows as **Idle** (green dot).

### Step 4 — Verify passwordless sudo for systemctl

The runner needs to restart zerver without a password prompt. This should already be
configured, but verify:

```bash
sudo visudo
# Confirm this line exists at the bottom:
# scadoshi ALL=(ALL) NOPASSWD: /bin/systemctl stop zerver, /bin/systemctl start zerver, /bin/systemctl restart zerver
```

### Re-registering after a server rebuild

If the server is rebuilt and the runner is lost:

1. Go to GitHub → Settings → Actions → Runners → find the old runner → Remove
2. Repeat Steps 1–3 above with a fresh token
3. The workflow picks it up automatically — no workflow file changes needed

---

## Tailscale (Local SSH Access)

Tailscale is used for SSHing into the server from your Mac or any network. It is **not**
used for CI/CD deploys (self-hosted runner eliminated that need).

**Current server Tailscale IP**: `100.114.251.8` (Hetzner VPS `zerver-prod`, since the 2026-06-13 migration — see `context/plans/vps_migration.md`). The old home box was `100.91.55.16` (powered off, kept as rollback). Tailscale IPs are stable and private (not publicly routable).

**Runners (post-migration):** two self-hosted runners live on the VPS — `zerver-prod` (repo `scadoshi/zwipe`, dir `~/actions-runner-zwipe`) and `zynergy-prod` (repo `scadoshi/zynergy`, dir `~/actions-runner-zynergy`), both boot-enabled. The deploy step's `sudo systemctl {stop,start} zerver` works because `/etc/sudoers.d/scadoshi` grants NOPASSWD for exactly those service-restart commands (all other admin = `ssh root@100.114.251.8`).

### Setup

**Server (one-time):**
```bash
curl -fsSL https://tailscale.com/install.sh | sh
sudo tailscale up
# Follow the auth URL printed to authenticate
sudo tailscale set --ssh   # enables Tailscale SSH (no deploy key needed)
```

**Mac:**
Install from the App Store, sign in with the same account.

**SSH into server from anywhere:**
```bash
ssh scadoshi@100.114.251.8        # service user (limited sudo)
ssh root@100.114.251.8            # admin (full sudo) — key-only, tailnet
```

### Tailscale Admin Configuration

- **Tag**: `tag:ci` (Access controls → Tags)
- **ACL rule**: `tag:ci → 100.114.251.8` all ports (kept for potential future use; was `100.91.55.16` pre-migration)
- **OAuth credential**: `github-actions` with `devices:core` + `auth_keys` scopes (kept for reference)

### Notes

- Server Tailscale IP is stable — never changes even if ISP rotates public IP
- `sshd` also listens on port 2222 via `/etc/systemd/system/ssh.socket.d/override.conf`
  (added during Xfinity troubleshooting — not required but harmless to keep)

---

## SQLx

**Migrations** run automatically on every deploy (step 4 in the workflow). The runner
sources `~/zwipe/.env` to get `DATABASE_URL` and runs `cargo sqlx migrate run`. Already-run
migrations are skipped (idempotent). New migrations land automatically on push.

**Builds** still use `SQLX_OFFLINE=true` with the committed `.sqlx/` directory so the
build step doesn't need a live database connection. After any query change on your Mac:

```bash
cargo sqlx prepare --workspace
git add .sqlx/
git commit -m "Update sqlx offline cache"
```

**One `.sqlx` directory only — the workspace root.** The macros prefer a crate-local
`zerver/.sqlx/` over the root one if it exists, and `prepare --workspace` never
refreshes a crate-local copy. A stale `zerver/.sqlx/` (leftover from an early
per-crate prepare) shadowed the root data and broke the 2026-07-05 deploy — the
first release to change an *existing* query's column types in place (daily-activity
BIGINT). Removed in `6556209d`; never run `cargo sqlx prepare` from inside `zerver/`
without `--workspace`. The verify step (workflow step 5) now catches any
offline-data drift before the build.

**Prerequisite**: `sqlx-cli` must be installed on the server (see `server.md` setup
checklist).

---

## Manual Trigger

GitHub → Actions tab → Deploy zerver → Run workflow → Run workflow

---

# zite — GitHub Pages Deploy

`.github/workflows/deploy-zite.yml`

Triggers on push to `main` when files under `zite/**` change (or the workflow file itself).
Also has `workflow_dispatch` for manual runs.

## What the Workflow Does

1. Installs `build-essential` (needed because Rust compiles proc-macro crates for the host target even when targeting WASM)
2. Installs `dioxus-cli` from source with `--force` (uses cargo cache keyed to `zite/Cargo.lock` to speed up repeat runs)
3. Runs `dx build --release --platform web` from `zite/` directory
4. Writes `CNAME` (zwipe.net) into the build output at `zite/target/dx/zite/release/web/public/`
5. Copies `index.html` → `404.html` in the same directory (SPA routing — GitHub Pages serves 404.html for unknown routes, Dioxus Router takes over)
6. Uploads the build output as a GitHub Pages artifact
7. Deploys to GitHub Pages

## GitHub Pages Configuration

- **Repository Settings → Pages → Source**: GitHub Actions
- **Custom domain**: zwipe.net
- **Enforce HTTPS**: enabled

## DNS (Cloudflare)

Four A records for the apex domain (DNS only, not proxied):
```
A  @  185.199.108.153
A  @  185.199.109.153
A  @  185.199.110.153
A  @  185.199.111.153
```
CNAME for www:
```
CNAME  www  scadoshi.github.io
```

## Notes

- First run after a Cargo.lock change takes ~8–10 minutes (compiles dioxus-cli from source)
- Subsequent runs restore dx from cache and finish much faster
- The `CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER: gcc` env var is set on both the install and build steps to resolve the host-target linker name mismatch on ubuntu-latest runners
