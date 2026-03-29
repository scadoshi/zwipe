# Ops — Start Here

Reference docs for deploying and operating zwipe. If rebuilding from scratch, follow this order.

---

## Rebuild Order

### 1. `server.md` — Provision the machine
SSH setup, PostgreSQL, log directory, `.env`, SQLx migrations, build toolchain, systemd service, zervice cron, Cloudflare Tunnel.

### 2. `cicd.md` — Automated deployment infrastructure
Self-hosted GitHub Actions runner on the server — no SSH tunnels, no deploy keys.
Runner setup, systemd service install, passwordless sudo for systemctl restart.
Also covers Tailscale (used for local SSH access, not CI/CD) and SQLx offline mode.

### 3. `deploy-backend.md` — Verify you can deploy manually
Build on the server, stop/copy/start zerver. Run this at least once before relying on the CI pipeline.

### 4. `deploy-web.md` — zweb pipeline
GitHub Actions auto-deploys zweb to GitHub Pages on push. Manual trigger available.

### 5. `services.md` — External services
Resend (transactional email + DNS records), Stripe/Buy Me a Coffee/GitHub Sponsors (donations), domain registrar and DNS setup.

### 6. `ios.md` — iOS dev setup
Build, sign, and deploy zwiper to a physical iPhone. Provisioning profile, certificates, keychain entitlements.

---

## Reference

- `tips.md` — Gotchas and one-off fixes (Ghostty terminal, etc.)

---

## Future

- `deploy-appstore.md` — App Store submission: Distribution cert, App Store provisioning profile, versioning (`CFBundleShortVersionString`), upload via Transporter, App Store Connect review.
