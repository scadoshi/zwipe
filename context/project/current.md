# Currently Working On

Active development tasks and immediate focus areas.

**Last Updated**: 2026-03-26. App is live on iPhone hitting Pi backend at api.zwipe.net.

**Current Focus**: UX improvements from first real-device testing.

**Recent Achievements**:
- **Production deployment**: zerver running on Raspberry Pi 5 as systemd service, exposed via Cloudflare Tunnel at `api.zwipe.net`. Full setup documented in `shipping.md`.
- **iOS app on device**: Built with Dioxus, signed with Apple Development cert (`VV74WQ89GD`), deployed via `ios-deploy`. Key fix: must use `--device "scotland-mobile"` flag or dx defaults to simulator target.
- **iOS Keychain session persistence**: `keychain-access-groups` entitlement in `zwiper/Entitlements.plist`, provisioning profile has Keychain Sharing enabled. Sessions survive cold launches — verified on device.
- **Mana pip balance**: `DeckMetrics` extended with `pip_consumed`/`pip_produced` per WUBRG color. CSS vertical bar charts on ViewDeck.
- **Oracle keywords frontend**: `get_oracle_keywords` client, chip-based filter component, accordion registration complete.
- **Deck profile enhancements**: ASCII chart replaced with CSS bars across all metric sections.

---

## Active Priorities (from device testing 2026-03-26)

1. **Card image size** — images need to expand to near full-screen on mobile. Currently too small to read comfortably.

2. **Filter active count badges** — each accordion group should show active filter count (e.g. "mana (2)") so user knows where to clear without opening every section.

3. **Filter clear empties card hand** — clearing filters on add-cards screen should reset the `Vec<Cards>` in hand, not leave stale results.

---

## Backlog

- Stop-words shared between zerver and zwiper (currently duplicated)
- AI card categorization (burn, ramp, removal, draw — via Claude API in zervice)
- Integration tests for SQLx repositories (require real DB)
- CI/CD for auto-deploying zerver to Pi (low priority — manual deploy is fast)
