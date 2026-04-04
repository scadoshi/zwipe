# Cloudflare — Tunnel, DNS, and Domains

All DNS is managed through Cloudflare. The API is exposed via Cloudflare Tunnel (no port
forwarding). This guide consolidates all Cloudflare and domain configuration.

---

## Domain Registrar

- **Registrar**: Namecheap (all domains)
- **DNS management**: Cloudflare (nameservers pointed at Cloudflare for all domains)
- **Domains**:
  - `zwipe.net` — main app domain; GitHub Pages (zite) + Cloudflare Tunnel (api.zwipe.net)
  - `scottyfermo.com` — portfolio (GitHub Pages)
  - `scadoshi.com` — 301 redirect → scottyfermo.com (Cloudflare redirect rule)
  - `scottyrayfermo.com` — 301 redirect → scottyfermo.com (Cloudflare redirect rule)

---

## DNS Records (zwipe.net)

### API — Cloudflare Tunnel

| Type | Name | Target | Proxy |
|------|------|--------|-------|
| CNAME | `api` | `<tunnel-uuid>.cfargotunnel.com` | Proxied (orange cloud) |

### Web — GitHub Pages

| Type | Name | Target | Proxy |
|------|------|--------|-------|
| A | `@` | `185.199.108.153` | DNS only |
| A | `@` | `185.199.109.153` | DNS only |
| A | `@` | `185.199.110.153` | DNS only |
| A | `@` | `185.199.111.153` | DNS only |
| CNAME | `www` | `scadoshi.github.io` | DNS only |

### Email — Resend

| Type | Name | Purpose |
|------|------|---------|
| TXT | `resend._domainkey` | DKIM — Resend signs outgoing mail |
| TXT | `@` (contains `v=spf1 include:amazonses.com`) | SPF — authorises Resend's servers |
| TXT | `_dmarc` | DMARC — required by Gmail/Yahoo/Microsoft |

DMARC record value: `v=DMARC1; p=none; rua=mailto:<support address>`

---

## Cloudflare Tunnel Setup

Cloudflare Tunnel creates an outbound-only encrypted connection from the server to
Cloudflare's edge — no port forwarding, no firewall rules, TLS handled by Cloudflare.
Requests to `api.zwipe.net` route through the tunnel to `localhost:3000`.

### Install cloudflared

```bash
curl -L https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-amd64.deb \
  -o cloudflared.deb
sudo dpkg -i cloudflared.deb
```

### Authenticate (one-time — headless, no browser)

The server has no display. `cloudflared tunnel login` prints a URL to the terminal.
Copy that URL and open it on your Mac or phone to complete the OAuth flow.

```bash
cloudflared tunnel login
# Prints: https://dash.cloudflare.com/argotunnel?callback=...
# Open that URL on your Mac/phone — select the zwipe.net zone
# Terminal confirms: "You have successfully logged in."
```

### Create the tunnel (first time only)

```bash
cloudflared tunnel create zwipe
# Prints a UUID — note it
# Writes ~/.cloudflared/<UUID>.json (credentials file)
```

### Create config file

```bash
mkdir -p ~/.cloudflared
nano ~/.cloudflared/config.yml
```

`~/.cloudflared/config.yml`:
```yaml
tunnel: <tunnel-uuid>
credentials-file: /home/<user>/.cloudflared/<tunnel-uuid>.json

ingress:
  - hostname: api.zwipe.net
    service: http://localhost:3000
  - service: http_status:404
```

### Add DNS record

Either in the Cloudflare dashboard (see DNS table above) or via CLI:
```bash
cloudflared tunnel route dns zwipe api.zwipe.net
```

### Install as systemd service

```bash
sudo mkdir -p /etc/cloudflared
sudo cp ~/.cloudflared/config.yml /etc/cloudflared/
sudo cp ~/.cloudflared/<tunnel-uuid>.json /etc/cloudflared/

# Update credentials-file path in /etc/cloudflared/config.yml:
# credentials-file: /etc/cloudflared/<tunnel-uuid>.json
sudo nano /etc/cloudflared/config.yml

sudo cloudflared service install
sudo systemctl enable cloudflared
sudo systemctl start cloudflared
sudo systemctl status cloudflared
```

### Verify

```bash
curl https://api.zwipe.net/
# {"message":"zerver","status":"ready","version":"0.1.0"}
```

---

## Re-deploying to a new machine

The tunnel UUID and credentials already exist in Cloudflare. Just:
1. `cloudflared tunnel login` (re-authenticate)
2. Copy the existing `<UUID>.json` credentials from the old server
3. Write `config.yml` with the same UUID
4. Install the service as above

**Note:** `nano` over Ghostty SSH fails with `Error opening terminal: xterm-ghostty`.
See [tips.md](tips.md) for the fix.
