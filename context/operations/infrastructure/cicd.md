# CI/CD: GitHub Actions Deploy

On every push to `main` that touches backend code, a self-hosted GitHub Actions runner on the server checks out the repo, builds `zerver` and `zervice` in place, installs them to `/usr/local/bin`, and restarts zerver, putting the old binaries back if the new zerver does not answer `/health`. The runner runs as its own `runner` user, not as `scadoshi`. No network tunnels, no deploy keys, no SCP.

---

## Workflow File

`.github/workflows/deploy-zerver.yml`

Triggers automatically on push to `main` when any of these paths change:
- `zerver/**`
- `zwipe-core/**`
- `Cargo.toml`
- `Cargo.lock`
- `.sqlx/**`
- `.github/workflows/deploy-zerver.yml`

Also has `workflow_dispatch` for manual runs from the GitHub Actions tab.

### Tests + lint gate the deploys

The test suite (`postgres:18` service + `cargo test -p zwipe-core -p zerver`, never `--workspace`, since zwiper's Dioxus/GTK stack won't build headless; `SQLX_OFFLINE=true` compiles against the committed `.sqlx` while `#[sqlx::test]` migrates a fresh DB per test via `DATABASE_URL`) runs in two places:

- **`.github/workflows/test.yml` (`Test`)**: on **pull requests only**, the pre-merge signal.
- **A `test` job inside `deploy-zerver.yml` and `deploy-zite.yml`**: the `deploy` (and zite's `build`) job `needs: [test, lint]`, so **a red suite or lint blocks the deploy**. This keeps the deploys' path filters (a docs-only push still won't redeploy) while gating, since GitHub can't make a `push`-triggered workflow wait on a *separate* workflow, hence the inline jobs. Push-time testing lives here, so `test.yml` stays PR-only (no double-run). `workflow_dispatch` still lets you force a deploy (it runs after the gate jobs).

A parallel **`lint` job** (in all three of `test.yml` / `deploy-zerver.yml` / `deploy-zite.yml`) runs `cargo +nightly fmt --check` (workspace-wide; nightly because `rustfmt.toml` sets the unstable `imports_granularity = "Crate"`) + `cargo clippy --workspace --all-targets -- -D warnings` (the lint job installs the WebKitGTK libraries so zwiper compiles; `SQLX_OFFLINE=true`; deploy-zite.yml also lints zite with `--features server`). Added 2026-07-10 after finding fmt had silently drifted. **Note: CI rides newest-stable clippy**: a local `rustup update stable` keeps you from being surprised by new lints (e.g. clippy 1.97 flagged a `useless_borrows_in_formatting` that 1.94 didn't, and correctly blocked a deploy until fixed).

Plan/design: [`../../archive/integration-tests/`](../../archive/integration-tests/overview.md).

### Security audit

`.github/workflows/audit.yml` (`Security audit`) runs `cargo audit --ignore RUSTSEC-2023-0071` (prebuilt binary via `taiki-e/install-action`) **weekly** (Mon 08:00 UTC), on any `Cargo.toml`/`Cargo.lock` change, and on manual dispatch. It scans `Cargo.lock` against the RustSec DB and **fails (→ GitHub emails you) on vulnerabilities**. Unmaintained or unsound *warnings* (e.g. the GTK3 desktop stack, `anyhow`, `rand 0.7`) don't fail. The one ignore, `RUSTSEC-2023-0071` (rsa Marvin timing sidechannel), is intentional: we sign JWTs with HS256/HMAC, never RSA; rsa only rides in as an unexercised code path via jsonwebtoken's `rust_crypto` backend + sqlx's never-compiled `mysql` driver. Backstop: turn on **Dependabot alerts** (repo Settings → Code security) for the passive GitHub-Advisory feed.

---

## What the Workflow Does

1. Checks out the repo
2. Installs stable Rust toolchain (cached)
3. Restores cargo cache (fast subsequent builds)
4. Runs SQLx migrations (`source /etc/zwipe/migrate.env` to export `DATABASE_URL`, then `cargo sqlx migrate run --source zerver/migrations`)
5. Verifies the committed `.sqlx/` matches the just-migrated schema (`cargo sqlx prepare --workspace --check -- --workspace --exclude zwiper --exclude zite`): fails fast with "query data is stale" instead of E0308 soup mid-build. The GUI crates are excluded because `--check` compiles crates to find their queries, and zwiper's Linux desktop deps (glib/GTK via pkg-config) don't exist on the headless VPS, this failed the first two verify runs (2026-07-06) before being scoped; only zerver has queries anyway
6. Builds `zerver` and `zervice` in release mode (`SQLX_OFFLINE=true`)
7. Copies the binaries to `/home/runner/deploy` (keeping the installed ones as `*.previous`), stops zerver, installs both to `/usr/local/bin`, starts zerver, and waits up to 20 seconds for `/health`. If it never answers, it prints zerver's last 50 journal lines, reinstalls the previous binaries, restarts, and fails the run

No Tailscale, no SSH keys, no SCP: the runner is already on the server. Migrations run before the build so new tables exist before the new binary starts.

---

## GitHub Configuration

### Actions are pinned by commit, and fork PRs wait for approval

The deploy job runs on the production box as `runner`, whose sudo rule can swap the zerver binary. Two settings keep that from being reachable by anyone but the owner:

- Every `uses:` in `.github/workflows/` names a full commit SHA with the version in a trailing comment (`actions/checkout@3d3c42e5… # v7`). A tag can be moved to new code by whoever controls the action's repo (tj-actions/changed-files, March 2025); a commit cannot. `.github/dependabot.yml` opens a weekly PR when a pinned action has a new release; merge those by reading the diff of the pin, not the title. `dtolnay/rust-toolchain` is pinned to its `master` commit and takes the channel through `with: toolchain:` (the `@stable` / `@nightly` ref forms are branches, so they cannot be pinned by SHA and still choose the channel).
- Settings → Actions → General → "Fork pull request workflows from outside collaborators" is **Require approval for all outside collaborators**. The repo is public and `test.yml` runs on `pull_request`, so without this a fork PR that edits a workflow's `runs-on:` to `self-hosted` would execute on the production box. Approving a PR's workflow run is the moment to read its workflow diff. The default `GITHUB_TOKEN` is read-only (Settings → Actions → General → Workflow permissions) and every workflow also says `permissions: contents: read`; `deploy-zite.yml` grants `pages: write` and `id-token: write` to its deploy job only.

### Secrets (Settings → Secrets and variables → Actions → Secrets)

| Name | Value |
|---|---|
| `TS_OAUTH_CLIENT_ID` | Tailscale OAuth client ID (kept for reference, not used in workflow) |
| `TS_OAUTH_SECRET` | Tailscale OAuth client secret (kept for reference, not used in workflow) |

### Variables (Settings → Secrets and variables → Actions → Variables)

None required for self-hosted runner deployment.

---

## Self-Hosted Runner Setup

The runner is a long-running process on the server that polls GitHub for jobs. It connects outbound to GitHub, so no inbound ports or tunnels are needed. Run this setup once; after that deploys are fully automatic.

### Step 1: Generate a runner token

GitHub → Repository Settings → Actions → Runners → New self-hosted runner → Linux → x64

Copy the token shown on that page (valid for 1 hour). With `gh` on your Mac, `gh api -X POST repos/scadoshi/zwipe/actions/runners/registration-token --jq .token` prints one.

### Step 2: Download and configure the runner on the server

The runner lives in the `runner` user's home (see `server.md`, Service Users), with its own Rust toolchain and sqlx-cli for the migration steps:

```bash
sudo -iu runner
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
~/.cargo/bin/cargo install sqlx-cli --no-default-features --features rustls,postgres

mkdir ~/actions-runner-zwipe && cd ~/actions-runner-zwipe

# Download: get the exact URL from the GitHub UI (version may change)
curl -o actions-runner-linux-x64.tar.gz -L \
  https://github.com/actions/runner/releases/download/v2.x.x/actions-runner-linux-x64-2.x.x.tar.gz

tar xzf actions-runner-linux-x64.tar.gz

# Configure: paste the token from Step 1 when prompted
./config.sh --url https://github.com/scadoshi/zwipe --token YOUR_TOKEN_HERE
# Name it zerver-prod; accept the default work folder
exit
```

### Step 3: Install as a systemd service

Back as `scadoshi`:

```bash
cd /home/runner/actions-runner-zwipe
sudo ./svc.sh install runner
sudo ./svc.sh start
sudo ./svc.sh status
```

The runner starts automatically on every boot. Check GitHub → Settings → Actions → Runners to confirm it shows as **Idle** (green dot).

### Step 4: The runner's sudo rule

The runner needs a few root commands without a password prompt, and nothing else. They live in `/etc/sudoers.d/runner`, shared with zynergy's runner: stop, start and restart of each service, `install` of each binary from `/home/runner/deploy` to `/usr/local/bin`, and reading the service's last 50 journal lines. For zwipe, that is:

```
runner ALL=(root) NOPASSWD: /usr/bin/systemctl stop zerver, /usr/bin/systemctl start zerver, /usr/bin/systemctl restart zerver, /usr/bin/install -m 755 /home/runner/deploy/zerver /usr/local/bin/zerver, /usr/bin/install -m 755 /home/runner/deploy/zervice /usr/local/bin/zervice, /usr/bin/journalctl -u zerver -n 50 --no-pager
```

Write it non-interactively and check it before it takes effect, since a broken sudoers file locks sudo out:

```bash
sudo tee /tmp/runner.sudoers >/dev/null    # paste the rule, then Ctrl-D
sudo visudo -cf /tmp/runner.sudoers && sudo install -m 440 /tmp/runner.sudoers /etc/sudoers.d/runner
```

A deploy step that calls sudo for anything else fails with `sudo: a password is required`. Change the rule and the workflow together. The runner reads `DATABASE_URL` from `/etc/zwipe/migrate.env`, the only env file it can open.

`scadoshi` has no passwordless sudo at all.

### Re-registering after a server rebuild

If the server is rebuilt and the runner is lost:

1. Go to GitHub → Settings → Actions → Runners → find the old runner → Remove
2. Repeat Steps 1–4 above with a fresh token
3. The workflow picks it up automatically, no workflow file changes needed

---

## Tailscale (Local SSH Access)

Tailscale is the only way to SSH into the server: ufw allows nothing but `tailscale0`. It is **not** used for CI/CD deploys (self-hosted runner eliminated that need).

**Current server**: Hetzner VPS `zerver-prod`, since the 2026-06-13 migration. Its Tailscale address is written here as `<server-tailnet-ip>`: tailnet addresses are redacted because this repo is public, the same convention as the `192.168.1.XXX` LAN addresses below. `tailscale status` on any tailnet device lists them, and the owner supplies the value when a session needs it. The old home box was rebuilt as scotland-server and is no longer a rollback. Tailscale IPs are stable and private (not publicly routable).

**Runners (post-migration):** two self-hosted runners live on the VPS: `zerver-prod` (repo `scadoshi/zwipe`, dir `/home/runner/actions-runner-zwipe`) and `zynergy-prod` (repo `scadoshi/zynergy`, dir `/home/runner/actions-runner-zynergy`), both boot-enabled and running as `runner`. Their sudo is `/etc/sudoers.d/runner` (Step 4). All other admin is `ssh scadoshi@<server-tailnet-ip>` with sudo and its password; root does not log in.

### Setup

**Server (one-time):**
```bash
curl -fsSL https://tailscale.com/install.sh | sh
sudo tailscale up
# Follow the auth URL printed to authenticate
```

**Mac:** Install from the App Store, sign in with the same account.

**SSH into server from anywhere:**
```bash
ssh scadoshi@<server-tailnet-ip>        # admin: full sudo with a password
```

### Tailscale Admin Configuration

- **Tag**: `tag:ci` (Access controls → Tags)
- **ACL rule**: `tag:ci → <server-tailnet-ip>` all ports (kept for potential future use; was `<old-box-tailnet-ip>` pre-migration)
- **OAuth credential**: `github-actions` with `devices:core` + `auth_keys` scopes (kept for reference)

### Notes

- Server Tailscale IP is stable, never changes even if ISP rotates public IP
- SSH settings are `/etc/ssh/sshd_config.d/10-hardening.conf`: no root login, keys from anywhere, passwords from the tailnet only. `server.md` (SSH Access) has the file
- Tailscale SSH is off; plain `sshd` over the tailnet is what answers

---

## SQLx

**Migrations** run automatically on every deploy (step 4 in the workflow). The runner sources `/etc/zwipe/migrate.env` to get `DATABASE_URL` and runs `cargo sqlx migrate run`. Already-run migrations are skipped (idempotent). New migrations land automatically on push.

**Builds** still use `SQLX_OFFLINE=true` with the committed `.sqlx/` directory so the build step doesn't need a live database connection. After any query change on your Mac:

```bash
cargo sqlx prepare --workspace
git add .sqlx/
git commit -m "Update sqlx offline cache"
```

**One `.sqlx` directory only: the workspace root.** The macros prefer a crate-local `zerver/.sqlx/` over the root one if it exists, and `prepare --workspace` never refreshes a crate-local copy. A stale `zerver/.sqlx/` (leftover from an early per-crate prepare) shadowed the root data and broke the 2026-07-05 deploy. The first release to change an *existing* query's column types in place (daily-activity BIGINT). Removed in `2e7fd985`; never run `cargo sqlx prepare` from inside `zerver/` without `--workspace`. The verify step (workflow step 5) now catches any offline-data drift before the build.

**Prerequisite**: `sqlx-cli` must be installed on the server (see `server.md` setup checklist).

---

## Manual Trigger

GitHub → Actions tab → Deploy zerver → Run workflow → Run workflow

---

# zite: GitHub Pages Deploy

`.github/workflows/deploy-zite.yml`

Triggers on push to `main` when files under `zite/**`, `zwipe-core/**`, or `zwipe-components/**` change (or the workflow file itself). Also has `workflow_dispatch` for manual runs.

## What the Workflow Does

1. Installs `build-essential` (Rust compiles proc-macro crates for the host target even when targeting WASM). `binaryen` was installed here until 2026-09-23 on the belief that dx would use a PATH copy of `wasm-opt`; it does not. dx 0.7.10 downloads its own pinned binaryen 129 and runs that, so the apt package was never read. Caching the download instead was tried and abandoned: on the Linux runner nothing named `wasm-opt` exists under `$HOME` after a build, and the fetch costs 2.4s
2. Installs a prebuilt `dioxus-cli@0.7.10` binary via `taiki-e/install-action`, so nothing compiles from source. Keep the pin matched to the workspace dioxus version and to `dx --version` on the build Macs
3. Runs `dx build --release --platform web --ssg --force-sequential` from `zite/` directory. `--ssg` pre-renders every route from the app's `static_routes` server function; `--force-sequential` is not optional, because without it the parallel client build finishes last and overwrites the SSG'd `public/index.html` with the bare shell
4. Writes `CNAME` (zwipe.net) into the build output at `target/dx/zite/release/web/public/` (workspace root)
5. Copies `index.html` → `404.html` in the same directory (SPA routing, GitHub Pages serves 404.html for unknown routes, Dioxus Router takes over)
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

- The `CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER: gcc` env var is set on the build step only, to resolve the host-target linker name mismatch on ubuntu-latest runners
