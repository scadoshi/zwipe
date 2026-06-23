# External Services

## Contributions / Donations

All three are linked on the `zwipe.net/contribute` page.

- **Stripe Payment Link**: payment link on zwipe.net/contribute — customer chooses amount
- **Buy Me a Coffee**: linked on zwipe.net/contribute
- **GitHub Sponsors**: linked on zwipe.net/contribute
  - Application submitted, pending GitHub approval
  - `.github/FUNDING.yml` in repo — Sponsor button appears on repo once approved

## Email (Resend)

- **Provider**: Resend (resend.com)
- **From address**: `support@zwipe.net`
- **Domain**: zwipe.net verified in Resend dashboard
- **DNS**: DKIM + SPF + DMARC records in Cloudflare — see [cloudflare.md](cloudflare.md)

`RESEND_EMAIL_FROM` must use a real address (not `noreply@`) — Resend flags no-reply
addresses and spam filters penalise them.

## GitHub

- **GitHub Actions**: CI/CD for zerver/zervice (self-hosted runner) and zite (GitHub Pages)
- **GitHub Pages**: hosts zwipe.net (zite static build)
- **GitHub Sponsors**: contribution link on zwipe.net/contribute — application pending approval

## Tailscale

- **Purpose**: private mesh network for SSHing into the server from anywhere — not used
  for CI/CD (self-hosted runner eliminated that need)
- **Tag**: `tag:ci` (used for ACL rules)
- **OAuth credential**: `github-actions` with `Devices::Core::Write` + `Auth Keys::Write`
  scopes (kept for reference, not used in current workflow)

## Cloudflare

See [cloudflare.md](cloudflare.md) for tunnel setup, DNS records, and domain config.
