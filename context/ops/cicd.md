# CI/CD — GitHub Actions Deploy

On every push to `main`, GitHub Actions builds `zerver` and `zervice` on an x86_64 Linux
runner and deploys them to the server via SCP, then restarts the systemd service.

---

## Workflow File

`.github/workflows/deploy.yml`

Currently set to `workflow_dispatch` (manual trigger only). To enable automatic deploys on push,
change the trigger:

```yaml
on:
  push:
    branches: [main]
```

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

## Port Forwarding (Xfinity)

GitHub Actions SSHes directly to the server — Cloudflare Tunnel only handles HTTP.
Port 22 must be forwarded through your router to the server's local IP.

- **Xfinity app** → WiFi → View WiFi equipment → Advanced settings → Port Forwarding
- External port: `22` → Internal IP: `10.0.0.137` → Internal port: `22` → TCP

**Public IP is dynamic** — Xfinity can change it on reconnect. If deploys start failing:
1. Check current public IP on server: `curl -4 ifconfig.me`
2. Update `DEPLOY_HOST` variable in GitHub if it changed

**Reserve the internal IP** in the Xfinity app so `10.0.0.137` never changes.

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

While `workflow_dispatch` is active, trigger a deploy from:
GitHub → Actions tab → Deploy → Run workflow → Run workflow

---

## What the Workflow Does

1. Checks out the repo
2. Installs stable Rust toolchain
3. Restores cargo cache (fast subsequent builds)
4. Builds `zerver` and `zervice` in release mode (`SQLX_OFFLINE=true`)
5. SCPs both binaries to `~/zwipe/` on the server
6. SSHes in and runs `sudo systemctl restart zerver`
