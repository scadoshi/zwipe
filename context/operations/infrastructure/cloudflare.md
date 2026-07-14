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
    service: http://127.0.0.1:3000   # NOT localhost — see warning below
  - service: http_status:404
```

> **Use `127.0.0.1`, not `localhost`.** On an IPv6-enabled host (e.g. the
> Hetzner VPS), `localhost` resolves to `::1`. zerver binds `0.0.0.0` (IPv4
> only), so cloudflared intermittently dials `::1` → connection refused →
> **~20% of requests 502**. Forcing IPv4 fixes it. (The old home box had no
> IPv6, so it never surfaced — discovered during the 2026-06-13 VPS cutover.)

**Current tunnels:** `zwipe-vps` (UUID `2b5d54b3-f05a-47ad-9785-5f7348987618`,
on the Hetzner VPS) serves `api.zwipe.net` as of the 2026-06-13 cutover. The
old home tunnel `zwipe` (`70ba169b-…`) is retained for rollback until home is
decommissioned.

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

---

## Cache Rules (Caching → Cache Rules)

Edge caching for backend endpoints that the origin shouldn't get hit for on
every request. Configured in the Cloudflare dashboard under
**Caching → Cache Rules**. Free-plan account; minimum custom Edge TTL is
**2 hours** (anything shorter falls back to CF's default behavior).

Each rule's shape is the same: a path match (prefix or exact) → "Eligible for
cache" + "Ignore origin Cache-Control" + a custom Edge TTL.

### Rule 1 — `Cache card metadata`

- **Condition**: `starts_with(http.request.uri.path, "/api/card/")`
- **Action**: Eligible for cache · Ignore origin Cache-Control · Edge TTL **24 hours**
- **Why**: card metadata is immutable between nightly Scryfall syncs. Origin
  gets one hit per POP per day for routes like `/api/card/{id}`, `/types`,
  `/keywords`, `/roles`, `/oracle-tags`, `/sets`, `/artists`, `/oracle-words`,
  `/languages`, `/{oracle_id}/printings`. Cache-hit responses skip the tunnel
  entirely (~5-10ms vs ~125ms public tunnel floor).
- **`/oracle-tags` freshness caveat**: unlike the rest of `/api/card/*`, the
  oracle-tag catalog's *descriptions* change on **our deploys**, not just the
  nightly sync (the in-app dictionary reads this route). With the 24h edge TTL a
  freshly deployed description batch can lag up to 24h. Default: accept it
  (descriptions aren't urgent). To push a batch live now, purge the one URL after
  the deploy + `zervice` run: `https://api.zwipe.net/api/card/oracle-tags` (Custom
  Purge → URL, or the API call below). No new rule needed — Rule 1 already covers it.
- **Compat requirement**: client must NOT send `Authorization: Bearer` on
  these requests — CF bypasses cache for authenticated requests by default.
  zwiper drops `bearer_auth` on the affected client methods; the backend
  serves these routes from `public_routes()`.
- **Verification**: `zcripts/latency/cf_cache_verify.sh` warms POPs and
  checks for `cf-cache-status: HIT`. Or one-liner:

  ```bash
  curl -sI https://api.zwipe.net/api/card/sets | grep -i cf-cache-status
  ```

### Rule 2 — `Cache marketing aggregates`

- **Condition**: `starts_with(http.request.uri.path, "/api/marketing/")`
- **Action**: Eligible for cache · Ignore origin Cache-Control · Edge TTL **2 hours** (CF free-plan minimum)
- **Why**: `/api/marketing/stats` returns app-wide sums across every user
  (`cards_swiped`, `searches`, `decks_created`) for the zwipe.net stats
  strip. Vanity totals don't move meaningfully inside a 2h window. Origin
  gets ~(POP count × 12) hits/day across the globe regardless of pageview
  volume.
- **Why not 1h**: free-plan minimum is 2h. Functionally indistinguishable
  for vanity numbers. If a milestone post ever needs immediate refresh,
  purge by URL (see below).
- **Path-prefix covers future endpoints**: any new `/api/marketing/*` we
  add (e.g. `/timeline`, `/leaderboard`) inherits the same rule.

### Rule 3 — `Cache changelog`

- **Condition**: `http.request.uri.path eq "/api/changelog"` (exact match, not
  a prefix — it's a single endpoint)
- **Action**: Eligible for cache · Ignore origin Cache-Control · Edge TTL **2 hours**
- **Why**: `/api/changelog` serves the release history (compiled into the
  server binary, `zwipe_core::content::changelog`), public and identical for
  every user, fetched once per app launch. Edge caching keeps origin cold; the
  payload is tiny.
- **Freshness caveat**: the changelog changes on **our deploys** (edit the
  const + ship `zerver`), not a sync. With the 2h edge TTL a freshly deployed
  entry can lag up to 2h. Default: accept it. To push it live now, purge the
  one URL after deploy: `https://api.zwipe.net/api/changelog` (Custom Purge →
  URL, or the API call below).
- **Compat requirement**: same as Rule 1 — the request must NOT carry
  `Authorization: Bearer` (CF bypasses cache for authenticated requests). The
  zwiper `get_changelog` client sends none, and the route is in
  `public_routes()`.
- **Client fallback**: if the fetch ever misses, zwiper falls back to the copy
  compiled into its own binary, so a stale or unreachable edge never blanks the
  changelog screen.

### Adding a new cache rule

1. Dashboard → zone `zwipe.net` → **Caching → Cache Rules → Create rule**
2. Name + condition (expression builder or `starts_with` / `eq` expression)
3. Action: **Eligible for cache** + **Ignore origin Cache-Control** + Edge TTL
4. Deploy

Free plan supports up to 10 cache rules. No `matches` regex on free —
stick to `starts_with`, `eq`, `contains` predicates.

### Purging cache on demand

**Dashboard**: Caching → Configuration → Purge Cache → **Custom Purge → URL**.
Paste the full URL (`https://api.zwipe.net/api/marketing/stats`) and submit.
Surgical — just that one cached response gets evicted across all POPs.

**API** (for automation):

```bash
curl -X POST "https://api.cloudflare.com/client/v4/zones/<ZONE_ID>/purge_cache" \
  -H "Authorization: Bearer <API_TOKEN>" \
  -H "Content-Type: application/json" \
  -d '{"files":["https://api.zwipe.net/api/marketing/stats"]}'
```

Zone ID is on the Overview page of the zone (right column). API token
needs `Zone → Cache Purge` permission.

**Avoid Purge Everything** unless something is genuinely wrong — it evicts
all CF-cached responses for the zone and forces every POP to re-fetch
from origin.

---

## Email Routing (set up 2026-06-10)

Inbound mail for zwipe.net runs through Cloudflare Email Routing (free):

- CF's MX (`route1-3.mx.cloudflare.net`) + SPF + DKIM records replaced the
  unused Namecheap `eforward*` registrar defaults.
- Routing rules forward to Scotty's personal inbox (verified destination):
  - `support@zwipe.net` — published address (App Store support contact,
    User-Agent contact strings, anything operational)
  - `scotty@zwipe.net` — human/founder address for partner outreach
- Outbound transactional mail is unchanged: Resend sends as
  `support@zwipe.net` via its own subdomain records (`send.zwipe.net`,
  `resend._domainkey`) — independent of inbound routing.
- Only one SPF TXT record may exist per hostname; the root SPF is now
  Cloudflare's. If a sender ever needs a root SPF include, merge it into
  the single record rather than adding a second.

Manage at: dash.cloudflare.com → zwipe.net → Email → Email Routing.
