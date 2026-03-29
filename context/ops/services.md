# External Services

## Contributions / Donations

All three are linked on the `zwipe.net/contribute` page.

- **Stripe Payment Link**: https://buy.stripe.com/5kQdRa5tUeNm9pd8BY9Zm00
  - Account: Fermco (org) → scadoshi (account) at stripe.com
  - Product: "Support zwipe development" — customer chooses amount
- **Buy Me a Coffee**: https://buymeacoffee.com/scadoshi
- **GitHub Sponsors**: https://github.com/sponsors/scadoshi
  - Application submitted, pending GitHub approval
  - `.github/FUNDING.yml` in repo — Sponsor button appears on repo once approved

## Email (Resend)

- **Provider**: Resend (resend.com)
- **From address**: `hello@zwipe.net`
- **Domain**: zwipe.net verified in Resend dashboard
- **DNS**: DKIM + SPF + DMARC records in Cloudflare — all verified, email lands in inbox

### Email Deliverability — DNS Records

| Record | Type | Purpose |
|--------|------|---------|
| `resend._domainkey` | TXT | DKIM — Resend signs outgoing mail |
| `@` / `v=spf1 include:amazonses.com` | TXT | SPF — authorises Resend's servers |
| `_dmarc` | TXT | DMARC — required by Gmail/Yahoo/Microsoft |

DMARC record value: `v=DMARC1; p=none; rua=mailto:hello@zwipe.net`

All three are set in Cloudflare DNS. `RESEND_EMAIL_FROM` must be `hello@zwipe.net` (not `noreply@`) — Resend flags no-reply addresses and spam filters penalise them.

## GitHub

- **Account**: scadoshi (github.com/scadoshi)
- **Repo**: scadoshi/zwipe (private)
- **GitHub Actions**: CI/CD for zerver/zervice (self-hosted runner) and zweb (GitHub Pages)
- **GitHub Pages**: hosts zwipe.net (zweb static build)
- **GitHub Sponsors**: contribution link on zwipe.net/contribute — application pending approval

## Tailscale

- **Purpose**: private mesh network for SSHing into the server from anywhere — not used
  for CI/CD (self-hosted runner eliminated that need)
- **Account**: scadoshi
- **Server IP**: `100.91.55.16` (stable, never changes even if Xfinity rotates public IP)
- **Tag**: `tag:ci` (owned by `scadoshi@github`, used for ACL rules)
- **OAuth credential**: `github-actions` with `Devices::Core::Write` + `Auth Keys::Write`
  scopes (kept for reference, not used in current workflow)
- **SSH**: `ssh scadoshi@100.91.55.16` from any machine on the tailnet

## Cloudflare

- **Account**: scottyfermo17@gmail.com
- **Purpose**: DNS management for all domains + Cloudflare Tunnel for `api.zwipe.net`
- **Tunnel**: outbound-only encrypted connection from server to Cloudflare edge — no port
  forwarding, no firewall rules needed. Requests to `api.zwipe.net` route through the tunnel
  to `localhost:3000` on the server
- **Tunnel UUID**: `70ba169b-8293-4a60-9b2d-e1f996a161db`
- **TLS**: handled by Cloudflare — server only speaks plain HTTP internally

## Domain Registrar

- **Registrar**: Namecheap (all domains)
- **DNS management**: Cloudflare (nameservers pointed at Cloudflare for all domains)
- **Domains**:
  - `zwipe.net` — main app domain; GitHub Pages (zweb) + Cloudflare Tunnel (api.zwipe.net)
  - `scottyfermo.com` — portfolio (GitHub Pages)
  - `scadoshi.com` — 301 redirect → scottyfermo.com (Cloudflare redirect rule)
  - `scottyrayfermo.com` — 301 redirect → scottyfermo.com (Cloudflare redirect rule)
