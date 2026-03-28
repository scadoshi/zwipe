# iOS Build, Sign, and Deploy

Full reference for building zwiper and deploying to a physical iPhone.

---

## Quick Deploy — against production server (use for real device testing)

`BACKEND_URL` is baked into the binary at compile time via `env!()`. The `.env` file
points at `127.0.0.1:3000` (local dev). To hit `api.zwipe.net`, pass the variable
inline — the shell sets it before `dx build` reads it:

```bash
cd /path/to/zwipe/zwiper

# 1. Build targeting physical device with prod backend URL
BACKEND_URL=https://api.zwipe.net dx build --platform ios --device "scotland-mobile"

# 2. Embed provisioning profile
cp ~/Downloads/zwipedev.mobileprovision \
  ../target/dx/main/debug/ios/Main.app/embedded.mobileprovision

# 3. Sign with correct cert + entitlements
codesign -f -s "F421F2E0FF6575A04BB18520C1A699A3F9CCEB45" \
  --entitlements zwiper/Entitlements.plist \
  ../target/dx/main/debug/ios/Main.app

# 4. Deploy to connected iPhone
ios-deploy --bundle ../target/dx/main/debug/ios/Main.app
```

Alternatively, load `.env.prod` explicitly before building:
```bash
export $(grep -v '^#' .env.prod | xargs) && \
  dx build --platform ios --device "scotland-mobile"
```

**First launch only:** iOS 16+ requires Developer Mode:
Settings → Privacy & Security → Developer Mode → toggle on → restart

**Untrusted developer prompt:** If iOS shows "Untrusted Developer" on first launch:
Settings → VPN & Device Management → your Apple ID → Trust

---

## Account and Cert Reference

| Thing | Value |
|-------|-------|
| Apple ID | `scottyfermo17@gmail.com` |
| Paid Team ID | `VV74WQ89GD` (SCOTTY RAY FERMO) |
| Bundle ID | `com.scadoshi.zwipe` |
| Signing cert fingerprint | `F421F2E0FF6575A04BB18520C1A699A3F9CCEB45` |
| Device name | `scotland-mobile` |
| Device UDID | `00008140-00166D6C3482801C` |
| Provisioning profile | `~/Downloads/zwipedev.mobileprovision` |
| Profile expiry | 2027-03-26 |

---

## Why `--device` is Required

`dx build --platform ios` defaults to the iOS Simulator target (platform 7). A simulator binary crashes immediately on real hardware:
```
Library not loaded: /usr/lib/libobjc.A.dylib
Reason: wrong platform to load into process
```

Always verify the binary targets real iOS:
```bash
vtool -show target/dx/main/debug/ios/Main.app/main
# Must show: LC_VERSION_MIN_IPHONEOS (not LC_BUILD_VERSION platform 7)
```

---

## Why Keychain Signing is Required

The `keyring` crate uses iOS Keychain for session storage. Without the `keychain-access-groups`
entitlement, every cold launch produces:
```
Platform secure storage failure: A required entitlement isn't present
```

The entitlement is in `zwiper/Entitlements.plist` — requires a paid Apple Developer account ($99/yr).

---

## Provisioning Profile Setup (one-time / yearly renewal)

1. developer.apple.com → Profiles → + → iOS App Development
2. App ID: `com.scadoshi.zwipe` (must have Keychain Sharing enabled)
3. Certificate: Apple Development cert under team `VV74WQ89GD`
4. Device: `scotland-mobile`
5. Download → `cp ~/Downloads/zwipedev.mobileprovision ~/Library/MobileDevice/Provisioning\ Profiles/`

**If the profile disappears from Provisioning Profiles:** macOS removes it if no matching private key is in Keychain. Fix: generate a CSR manually so the private key is in Keychain:

```bash
openssl genrsa -out zwipe-key.pem 2048
openssl req -new -key zwipe-key.pem -out zwipe.certSigningRequest \
  -subj "/emailAddress=scottyfermo17@gmail.com,CN=SCOTTY RAY FERMO,C=US"
security import ~/Desktop/zwipe-key.pem -k ~/Library/Keychains/login.keychain-db -T /usr/bin/codesign
```

Upload CSR to developer.apple.com → Certificates → + → Apple Development.

**Note on team IDs:** Xcode's "Manage Certificates" creates certs under the Personal Team (`NVSWB62C54`), not the paid team. The `(NVSWB62C54)` shown by `security find-identity` is the CN display name — the OU field is the actual team ID. Verify with:
```bash
security find-identity -v -p codesigning
# Entry with VV74WQ89GD in OU is the correct cert
```

---

## App Store Distribution (future)

Current setup uses a Development profile. App Store requires:
- Distribution certificate (Apple Distribution, not Apple Development)
- App Store provisioning profile
- Archive + upload via `xcrun altool` or Transporter

See `status/todo.md` for App Store submission checklist.
