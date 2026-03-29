# Infrastructure Guides

Server provisioning, CI/CD, deployments, backups, and external services.

---

## Rebuild Order (from scratch)

1. **[server.md](server.md)** — Provision the Ubuntu machine (SSH, PostgreSQL, .env, systemd, zervice cron)
2. **[cloudflare.md](cloudflare.md)** — Cloudflare Tunnel, DNS records, domain config
3. **[cicd.md](cicd.md)** — Self-hosted GitHub Actions runner, SQLx offline mode
4. **[deploy-backend.md](deploy-backend.md)** — Manual backend deploy (verify before relying on CI)
5. **[deploy-web.md](deploy-web.md)** — zweb → GitHub Pages pipeline
6. **[services.md](services.md)** — External services (Resend, Stripe, Tailscale, GitHub Sponsors)

## Reference

- **[backups.md](backups.md)** — Nightly PostgreSQL backups to Cloudflare R2, restore procedures
- **[manual-run.md](manual-run.md)** — Running zerver/zervice manually on the server
- **[tips.md](tips.md)** — Gotchas and one-off fixes
