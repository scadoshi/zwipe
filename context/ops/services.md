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

## Domain Registrar

- **Registrar**: Namecheap (all domains)
- **DNS management**: Cloudflare (nameservers pointed at Cloudflare for all domains)
- **Domains**:
  - `zwipe.net` — main app domain; GitHub Pages (zweb) + Cloudflare Tunnel (api.zwipe.net)
  - `scottyfermo.com` — portfolio (GitHub Pages)
  - `scadoshi.com` — 301 redirect → scottyfermo.com (Cloudflare redirect rule)
  - `scottyrayfermo.com` — 301 redirect → scottyfermo.com (Cloudflare redirect rule)
