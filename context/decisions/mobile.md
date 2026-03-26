# Mobile Platform Decision

**Decided: Dioxus (Rust) — shipped to real iPhone 2026-03-26**

---

## What We're Running

- **Framework**: Dioxus 0.7.3 (`dx` CLI)
- **Target**: iOS physical device (`aarch64-apple-ios`)
- **Distribution**: Manual via `ios-deploy` (development), App Store TBD
- **Session storage**: `keyring` crate → iOS Keychain with `keychain-access-groups` entitlement

## iOS Build Process

Dioxus 0.7 does NOT generate an Xcode project. It produces a `.app` bundle directly.
The full deploy pipeline is in `shipping.md`, but the critical flags:

```bash
# MUST include --device flag — without it, dx targets the simulator (crashes on real device)
dx build --platform ios --device "scotland-mobile"
```

Verify the binary targets real iOS (not simulator):
```bash
vtool -show target/dx/main/debug/ios/Main.app/main
# Must show: LC_VERSION_MIN_IPHONEOS (not LC_BUILD_VERSION platform 7)
```

## iOS Signing

- Bundle ID: `com.scadoshi.zwipe` (set in `zwiper/Dioxus.toml`)
- Apple Developer Team: `VV74WQ89GD` (SCOTTY RAY FERMO)
- Signing cert fingerprint: `F421F2E0FF6575A04BB18520C1A699A3F9CCEB45`
- Entitlements: `zwiper/Entitlements.plist` — Keychain Sharing + application-identifier
- Provisioning profile: `~/Downloads/zwipedev.mobileprovision` (expires 2027-03-26)

## Why Dioxus

Single language (Rust) across backend and frontend. Same types, same error handling, same
mental model. For a solo developer this is the right call — no context switching, shared
domain types between zerver and zwiper, compile-time safety on both sides.

## Full Reference

See `context/project/shipping.md` for the complete iOS signing walkthrough.
