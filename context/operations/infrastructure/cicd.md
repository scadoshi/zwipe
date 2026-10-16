# CI/CD: GitHub Actions Deploy

A self-hosted GitHub Actions runner on the server deploys zerver on every push to `main` that touches backend code.

- It builds `zerver` and `zervice` in place, installs them to `/usr/local/bin` and restarts zerver.
- If the new zerver does not answer `/health`, it puts the old binaries back.
- It runs as its own `runner` user, not `scadoshi`.
- No tunnels, deploy keys or SCP: the runner is already on the box.

---

## Workflow File

`.github/workflows/deploy-zerver.yml` runs on push to `main` when any of these change:

- `zerver/**`
- `zwipe-core/**`
- `Cargo.toml`
- `Cargo.lock`
- `.sqlx/**`
- `.github/workflows/deploy-zerver.yml`

`workflow_dispatch` also allows manual runs from the Actions tab.

### Tests and lint gate the deploys

The test suite runs `cargo test -p zwipe-core -p zerver` against a `postgres:18` service.

- Never `--workspace`: zwiper's Dioxus/GTK stack does not build headless.
- `SQLX_OFFLINE=true` compiles against the committed `.sqlx`, and `#[sqlx::test]` migrates a fresh DB per test through `DATABASE_URL`.

It runs in two places:

- `.github/workflows/test.yml` (`Test`) runs on pull requests only, as the pre-merge signal.
- A `test` job inside `deploy-zerver.yml` and `deploy-zite.yml`. The `deploy` job (zite's `build`) has `needs: [test, lint]`, so a red suite or lint blocks the deploy.

The gate lives inside the deploy workflows because GitHub can't make a `push` workflow wait on a separate one. That keeps the path filters (a docs-only push still doesn't redeploy) and keeps `test.yml` PR-only, so nothing runs twice. `workflow_dispatch` still forces a deploy, after the gate jobs.

A parallel `lint` job in all three workflows runs:

- `cargo +nightly fmt --check` across the workspace. It needs nightly because `rustfmt.toml` sets the unstable `imports_granularity = "Crate"`.
- `cargo clippy --workspace --all-targets -- -D warnings`, with `SQLX_OFFLINE=true` and the WebKitGTK libraries installed so zwiper compiles. `deploy-zite.yml` also lints zite with `--features server`.

The lint job dates from 2026-07-10, after fmt was found to have drifted silently.

CI uses the newest stable clippy, so run `rustup update stable` locally to see new lints before CI does. Clippy 1.97 flagged a `useless_borrows_in_formatting` that 1.94 didn't, and rightly blocked a deploy until it was fixed.

The design notes are in [`../../archive/integration-tests/`](../../archive/integration-tests/overview.md).

### Security audit

`.github/workflows/audit.yml` (`Security audit`) runs `cargo audit --ignore RUSTSEC-2023-0071` against the RustSec DB.

- It runs weekly (Mon 08:00 UTC), on any `Cargo.toml` or `Cargo.lock` change, and on manual dispatch.
- A vulnerability fails the run, and GitHub emails you.
- Unmaintained or unsound warnings (the GTK3 desktop stack, `anyhow`, `rand 0.7`) don't fail it.
- The one ignore, `RUSTSEC-2023-0071` (the rsa Marvin timing side channel), is deliberate. JWTs are signed with HS256, never RSA. rsa only comes in as an unused path through jsonwebtoken's `rust_crypto` backend and sqlx's never-compiled `mysql` driver.
- Dependabot alerts (Settings → Code security) are the passive backstop.

---

## What the Workflow Does

1. Checks out the repo
2. Installs the stable Rust toolchain (cached)
3. Restores the cargo cache
4. Runs migrations: `source /etc/zwipe/migrate.env` for `DATABASE_URL`, then `cargo sqlx migrate run --source zerver/migrations`. New tables exist before the new binary starts
5. Checks that the committed `.sqlx/` matches the migrated schema with `cargo sqlx prepare --workspace --check -- --workspace --exclude zwiper --exclude zite`, failing fast with "query data is stale". The GUI crates are excluded because `--check` compiles them and their GTK deps don't exist on the headless VPS, which failed the first two verify runs (2026-07-06). Only zerver has queries anyway
6. Builds `zerver` and `zervice` in release mode (`SQLX_OFFLINE=true`)
7. Copies the binaries to `/home/runner/deploy`, keeping the installed ones as `*.previous`. Then it stops zerver, installs both to `/usr/local/bin`, starts zerver and waits up to 20 seconds for `/health`. If that never answers, it prints zerver's last 50 journal lines, reinstalls the previous binaries, restarts and fails the run

---

## GitHub Configuration

### Pinned actions and fork approval

The deploy job runs on the production box as `runner`, whose sudo rule can swap the zerver binary. Two settings keep that out of reach of anyone but the owner.

Every `uses:` in `.github/workflows/` names a full commit SHA, with the version in a trailing comment (`actions/checkout@3d3c42e5… # v7`).

- A tag can be moved to new code by whoever owns the action's repo (tj-actions/changed-files, March 2025). A commit can't.
- `.github/dependabot.yml` opens a weekly PR when a pinned action has a new release. Read the diff of the pin before merging, not just the title.
- `dtolnay/rust-toolchain` is pinned to its `master` commit and gets the channel through `with: toolchain:`. Its `@stable` and `@nightly` refs are branches, which can't be pinned by SHA.

Fork pull request workflows from outside collaborators are set to **Require approval for all outside collaborators** (Settings → Actions → General).

- The repo is public and `test.yml` runs on `pull_request`. Without approval, a fork PR could change `runs-on:` to `self-hosted` and run on the production box.
- Approving a PR's workflow run is the moment to read its workflow diff.
- The default `GITHUB_TOKEN` is read-only (Settings → Actions → General → Workflow permissions), and every workflow also declares `permissions: contents: read`. `deploy-zite.yml` grants `pages: write` and `id-token: write` to its deploy job only.

### Secrets and variables

The self-hosted deploy needs none. `TS_OAUTH_CLIENT_ID` and `TS_OAUTH_SECRET` are still set (Settings → Secrets and variables → Actions) from the old Tailscale deploy, and no workflow reads them.

---

## Self-Hosted Runner Setup

The runner is a long-running process that polls GitHub for jobs. It connects outbound, so no inbound ports or tunnels are needed. Set it up once and deploys are automatic after that.

### Step 1: Get a runner token

GitHub → Repository Settings → Actions → Runners → New self-hosted runner → Linux → x64. The token on that page is valid for an hour.

With `gh`: `gh api -X POST repos/scadoshi/zwipe/actions/runners/registration-token --jq .token`.

### Step 2: Download and configure the runner

The runner lives in the `runner` user's home (`server.md`, Service Users), with its own Rust toolchain and sqlx-cli for the migration steps:

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

It starts on every boot. GitHub → Settings → Actions → Runners should show it as **Idle** (green dot).

### Step 4: The runner's sudo rule

The runner gets a few root commands without a password, and nothing else. They live in `/etc/sudoers.d/runner`, shared with zynergy's runner:

- stop, start and restart of each service
- `install` of each binary from `/home/runner/deploy` to `/usr/local/bin`
- reading the service's last 50 journal lines

For zwipe:

```
runner ALL=(root) NOPASSWD: /usr/bin/systemctl stop zerver, /usr/bin/systemctl start zerver, /usr/bin/systemctl restart zerver, /usr/bin/install -m 755 /home/runner/deploy/zerver /usr/local/bin/zerver, /usr/bin/install -m 755 /home/runner/deploy/zervice /usr/local/bin/zervice, /usr/bin/journalctl -u zerver -n 50 --no-pager
```

Write it non-interactively and check it before it takes effect, because a broken sudoers file locks sudo out:

```bash
sudo tee /tmp/runner.sudoers >/dev/null    # paste the rule, then Ctrl-D
sudo visudo -cf /tmp/runner.sudoers && sudo install -m 440 /tmp/runner.sudoers /etc/sudoers.d/runner
```

- A deploy step that calls sudo for anything else fails with `sudo: a password is required`, so change the rule and the workflow together.
- The runner reads `DATABASE_URL` from `/etc/zwipe/migrate.env`, the only env file it can open.
- `scadoshi` has no passwordless sudo at all.

### Re-registering after a server rebuild

1. GitHub → Settings → Actions → Runners → find the old runner → Remove
2. Repeat Steps 1–4 with a fresh token
3. The workflow picks it up with no file changes

---

## Tailscale (SSH Access)

Tailscale is the only way to SSH into the server: ufw allows nothing but `tailscale0`. Deploys don't use it.

- The server is the Hetzner VPS `zerver-prod`, since the 2026-06-13 migration. The old home box is now scotland-server and is not a rollback.
- Its tailnet address is written as `<server-tailnet-ip>` because this repo is public (the same goes for the `192.168.1.XXX` LAN addresses). `tailscale status` on any tailnet device lists it, and the owner supplies it when a session needs it.
- Tailscale IPs are stable and private, even when the ISP rotates the public IP.
- Two runners live on the VPS, both boot-enabled and running as `runner`, with sudo from `/etc/sudoers.d/runner` (Step 4):
  - `zerver-prod`: repo `scadoshi/zwipe`, dir `/home/runner/actions-runner-zwipe`
  - `zynergy-prod`: repo `scadoshi/zynergy`, dir `/home/runner/actions-runner-zynergy`
- All other admin is `ssh scadoshi@<server-tailnet-ip>`, with sudo and its password. Root does not log in.
- SSH settings are in `/etc/ssh/sshd_config.d/10-hardening.conf`: no root login, keys from anywhere, passwords from the tailnet only. `server.md` (SSH Access) has the file.
- Tailscale SSH is off. Plain `sshd` over the tailnet answers.

### Setup

Server, once:

```bash
curl -fsSL https://tailscale.com/install.sh | sh
sudo tailscale up
# Follow the auth URL printed to authenticate
```

Mac: install from the App Store and sign in with the same account. Then from anywhere:

```bash
ssh scadoshi@<server-tailnet-ip>        # admin: full sudo with a password
```

### Tailscale admin configuration

Left over from the old Tailscale deploy and kept for possible future use:

- Tag `tag:ci` (Access controls → Tags)
- ACL rule `tag:ci → <server-tailnet-ip>`, all ports (it pointed at `<old-box-tailnet-ip>` before the migration)
- OAuth credential `github-actions` with `devices:core` and `auth_keys` scopes

---

## SQLx

Migrations run on every deploy (workflow step 4). Already-run migrations are skipped, so a new one lands just by pushing it.

Builds use `SQLX_OFFLINE=true` with the committed `.sqlx/`, so the build doesn't need a live database. After any query change:

```bash
cargo sqlx prepare --workspace
git add .sqlx/
git commit -m "Update sqlx offline cache"
```

Keep one `.sqlx` directory, at the workspace root.

- The macros prefer a crate-local `zerver/.sqlx/` over the root one, and `prepare --workspace` never refreshes it. A stale copy, left from an early per-crate prepare, shadowed the root data and broke the 2026-07-05 deploy: the first release to change an existing query's column types in place (daily-activity BIGINT). It was removed in `2e7fd985`.
- Never run `cargo sqlx prepare` from inside `zerver/` without `--workspace`.
- Workflow step 5 catches offline-data drift before the build.

`sqlx-cli` must be installed on the server (`server.md` setup checklist).

---

## Manual Trigger

GitHub → Actions tab → Deploy zerver → Run workflow → Run workflow

---

# zite: GitHub Pages Deploy

`.github/workflows/deploy-zite.yml` runs on push to `main` when `zite/**`, `zwipe-core/**`, `components/**` or the workflow file changes. `workflow_dispatch` allows manual runs.

## What the Workflow Does

1. Installs `build-essential`, since Rust compiles proc-macro crates for the host even when targeting WASM. Binaryen isn't needed: dx 0.7.10 downloads and runs its own pinned binaryen 129, so the apt package (installed here until 2026-09-23) was never read. Caching that download was tried and dropped: nothing named `wasm-opt` exists under `$HOME` after a Linux build, and the fetch costs 2.4s
2. Installs a prebuilt `dioxus-cli@0.7.10` through `taiki-e/install-action`, so nothing compiles from source. Keep the pin matched to the workspace dioxus version and to `dx --version` on the build Macs
3. Runs `dx build --release --platform web --ssg --force-sequential` from `zite/`. `--ssg` pre-renders every route from the app's `static_routes` server function. `--force-sequential` is required: without it the parallel client build finishes last and overwrites the pre-rendered `public/index.html` with the bare shell
4. Writes `CNAME` (zwipe.net) into `target/dx/zite/release/web/public/` (workspace root)
5. Copies `index.html` to `404.html` there. GitHub Pages serves 404.html for unknown routes, and the Dioxus Router takes over
6. Uploads the build output as a GitHub Pages artifact
7. Deploys to GitHub Pages

The build step alone sets `CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER: gcc`, which fixes the host-target linker name mismatch on ubuntu-latest runners.

## GitHub Pages Configuration

- Source: GitHub Actions (Repository Settings → Pages)
- Custom domain: zwipe.net
- Enforce HTTPS: on

## DNS (Cloudflare)

Four A records for the apex domain, DNS only (not proxied):

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
