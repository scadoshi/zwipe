# CI/CD — GitHub Actions Deploy

On every push to `main` that touches backend code, GitHub Actions builds `zerver` and `zervice`
on an x86_64 Linux runner and deploys them to the server via SCP over Tailscale, then restarts
the systemd service.

---

## Workflow File

`.github/workflows/deploy.yml`

Triggers automatically on push to `main` when any of these paths change:
- `zerver/**`
- `zwipe/**`
- `zervice/**`
- `.github/workflows/deploy.yml`

Also has `workflow_dispatch` for manual runs from the GitHub Actions tab.

---

## GitHub Configuration

### Secrets (Settings → Secrets and variables → Actions → Secrets)

| Name | Value |
|---|---|
| `DEPLOY_SSH_KEY` | Full contents of `~/.ssh/zwipe-deploy` (private key, including `-----BEGIN/END-----` lines) |

### Variables (Settings → Secrets and variables → Actions → Variables)

| Name | Value |
|---|---|
| `DEPLOY_HOST` | Server public IP (e.g. `67.185.151.50`) — check with `curl -4 ifconfig.me` on server |
| `DEPLOY_USER` | `scadoshi` |

---

## SSH Deploy Key

A dedicated key pair is used so GitHub Actions never has access to your personal keys.

**Generate (on Mac, one-time):**
```bash
ssh-keygen -t ed25519 -C "github-actions" -f ~/.ssh/zwipe-deploy
# No passphrase
```

**Add public key to server:**
```bash
cat ~/.ssh/zwipe-deploy.pub
# Copy output, then on server:
echo "paste-public-key-here" >> ~/.ssh/authorized_keys
```

**Add private key to GitHub:**
```bash
cat ~/.ssh/zwipe-deploy
# Copy full output (including BEGIN/END lines) → GitHub secret DEPLOY_SSH_KEY
```

---

## Tailscale (SSH Access for CI/CD)

Port forwarding via Xfinity is unreliable — xFi Advanced Security and CGNAT silently block
inbound connections regardless of router config. Tailscale is used instead.

Tailscale creates a private WireGuard mesh network (tailnet). Every device gets a stable
`100.x.x.x` IP reachable from anywhere — no router config, no inbound ports, no ISP interference.
All connections originate outbound so NAT is irrelevant. Traffic is encrypted end-to-end.

**Current server Tailscale IP: `100.91.55.16`**

---

### Step 1 — Create a Tailscale account

Sign up at tailscale.com. The free plan supports up to 100 devices and is sufficient for this setup.

---

### Step 2 — Install Tailscale on the server

SSH into the server locally and run:

```bash
curl -fsSL https://tailscale.com/install.sh | sh
sudo tailscale up
```

It will print an auth URL — open it in your browser and authenticate. The installer automatically
configures `tailscaled` as a systemd service so Tailscale reconnects on reboot.

Get the server's stable Tailscale IP:
```bash
tailscale ip -4
# e.g. 100.91.55.16
```

---

### Step 3 — Install Tailscale on your Mac

Install from the App Store (recommended — handles system extension permissions correctly).
The brew CLI version (`brew install tailscale`) requires `sudo brew services start tailscale`
and can have launchd permission issues on newer macOS. App Store version is more reliable.

Sign in with the same Tailscale account. Both devices will appear in the Tailscale admin
under Machines. You can now SSH to the server via its Tailscale IP from any network:

```bash
ssh scadoshi@100.91.55.16
```

Update your local `zerver` alias to use the Tailscale IP so it works from anywhere, not just
home WiFi.

---

### Step 4 — Update DEPLOY_HOST in GitHub

Go to GitHub → Repository Settings → Secrets and variables → Actions → Variables.
Update `DEPLOY_HOST` to the server's Tailscale IP (`100.91.55.16`).

---

### Step 5 — Create an OAuth credential in Tailscale

GitHub Actions runners are ephemeral VMs that need to join the tailnet for the duration of
a deploy workflow. An OAuth credential lets the workflow authenticate without interactive login.

1. Go to Tailscale admin → Access controls → Tags tab
2. Create a tag named `tag:ci` — set owner to your GitHub user (`scadoshi@github`)
3. Go to Settings → Trust credentials → New credential → OAuth
4. Name it `github-actions`
5. Under Scopes, grant only: **Devices → Core → Write**
6. Under Tags, select `tag:ci`
7. Save — copy the **Client ID** and **Client Secret** immediately (secret is shown once)

Add both to GitHub as secrets:

| GitHub Secret | Value |
|---|---|
| `TS_OAUTH_CLIENT_ID` | Client ID from Tailscale |
| `TS_OAUTH_SECRET` | Client Secret from Tailscale |

---

### Step 6 — Add Tailscale step to deploy.yml

Add this step before the Build step in `.github/workflows/deploy.yml`:

```yaml
- name: Connect to Tailscale
  uses: tailscale/github-action@v3
  with:
    oauth-client-id: ${{ secrets.TS_OAUTH_CLIENT_ID }}
    oauth-secret: ${{ secrets.TS_OAUTH_SECRET }}
    tags: tag:ci
```

The GitHub Actions runner joins the tailnet as an ephemeral `tag:ci` device, reaches the
server at its Tailscale IP, then automatically removes itself from the tailnet when the job ends.

---

### Notes

- **Server IP is stable** — `100.91.55.16` never changes even if Xfinity rotates your public IP
- **Works from any network** — laptop at a coffee shop, GitHub Actions runner, anywhere
- **No router config ever needed** — Xfinity, CGNAT, Advanced Security all become irrelevant
- **sshd also listens on port 2222** (in addition to 22) via a systemd socket override at
  `/etc/systemd/system/ssh.socket.d/override.conf` — this was added during Xfinity troubleshooting
  but is no longer needed for external access now that Tailscale is in place

---

## SQLx Offline Mode

The GitHub Actions runner has no database. Builds use `SQLX_OFFLINE=true` with the
committed `.sqlx/` directory. After any query change on your Mac:

```bash
cargo sqlx prepare --workspace
git add .sqlx/
git commit -m "Update sqlx offline cache"
```

---

## Manual Trigger

To trigger a deploy without pushing code:
GitHub → Actions tab → Deploy → Run workflow → Run workflow

---

## What the Workflow Does

1. Checks out the repo
2. Installs stable Rust toolchain
3. Restores cargo cache (fast subsequent builds)
4. Builds `zerver` and `zervice` in release mode (`SQLX_OFFLINE=true`)
5. SCPs both binaries to `~/zwipe/` on the server
6. SSHes in and runs `sudo systemctl restart zerver`

---

# zweb — GitHub Pages Deploy

`.github/workflows/deploy-zweb.yml`

Triggers on push to `main` when files under `zweb/**` change (or the workflow file itself). Also has `workflow_dispatch` for manual runs.

## What the Workflow Does

1. Installs `build-essential` (needed because Rust compiles proc-macro crates for the host target even when targeting WASM)
2. Installs `dioxus-cli` from source with `--force` (uses cargo cache keyed to `zweb/Cargo.lock` to speed up repeat runs)
3. Runs `dx build --release --platform web` from `zweb/` directory
4. Writes `CNAME` (zwipe.net) into the build output at `zweb/target/dx/zweb/release/web/public/`
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
