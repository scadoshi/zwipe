# Ops — Start Here

Reference docs for deploying and operating zwipe. If rebuilding from scratch, follow this order.

---

## Rebuild Order

### 1. `server.md` — Provision the machine
SSH setup, PostgreSQL, log directory, `.env`, SQLx migrations, build toolchain, systemd service, zervice cron, Cloudflare Tunnel.

### 2. `cicd.md` — Automated deployment infrastructure
Tailscale (required for GitHub Actions SSH), deploy SSH key, GitHub secrets/variables, SQLx offline mode.
Do this before trusting CI — Tailscale must be running on the server first.

### 3. `deploy-backend.md` — Verify you can deploy manually
Build on the server, stop/copy/start zerver. Run this at least once before relying on the CI pipeline.

### 4. `deploy-web.md` — zweb pipeline
GitHub Actions auto-deploys zweb to GitHub Pages on push. Manual trigger available.

### 5. `services.md` — External services
Resend (transactional email + DNS records), Stripe/Buy Me a Coffee/GitHub Sponsors (donations), domain registrar and DNS setup.

### 6. `ios.md` — iOS dev setup
Build, sign, and deploy zwiper to a physical iPhone. Provisioning profile, certificates, keychain entitlements.

---

## Future

- `deploy-appstore.md` — App Store submission: Distribution cert, App Store provisioning profile, versioning (`CFBundleShortVersionString`), upload via Transporter, App Store Connect review.
