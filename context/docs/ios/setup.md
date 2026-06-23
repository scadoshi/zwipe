# Apple Developer Setup

One-time setup for certificates, App ID, and signing infrastructure.

---

## Prerequisites

- Paid Apple Developer account ($99/yr) — [developer.apple.com](https://developer.apple.com)
- Xcode installed (for `codesign`, `security`, `vtool` CLI tools)
- A `~/certs/` directory for backing up signing materials (NOT in repo)

---

## 1. Register the App ID

1. developer.apple.com → Certificates, Identifiers & Profiles → Identifiers
2. Click **+** → select **App IDs** → Continue
3. Select **App** (not App Clip) → Continue
4. Fill out:
   - **Description**: Zwipe
   - **Bundle ID**: select **Explicit**, enter `com.scadoshi.zwipe`
5. Under **Capabilities**, enable **Keychain Sharing** (required for session storage via `keyring` crate)
6. Click **Continue** → **Register**

---

## 2. Create an Apple Development Certificate

Used for deploying debug builds to physical devices.

**Do NOT use Keychain Access → Certificate Assistant.** It consistently fails with
"The specified item could not be found in the keychain." Use the CLI:

```bash
openssl genrsa -out zwipe-dev-key.pem 2048
openssl req -new -key zwipe-dev-key.pem -out DevCSR.certSigningRequest \
  -subj "/emailAddress=<apple-id-email>,CN=<your-name>,C=US"
security import zwipe-dev-key.pem -k ~/Library/Keychains/login.keychain-db -T /usr/bin/codesign
```

Then on Apple's site:

1. Certificates, Identifiers & Profiles → Certificates → **+**
2. Select **Apple Development** → Continue
3. Upload `DevCSR.certSigningRequest`
4. Download the `.cer` file, double-click to install (installs silently)

Back up:
```bash
mv zwipe-dev-key.pem DevCSR.certSigningRequest ~/certs/
```

---

## 3. Register Your Device

Required for dev provisioning profiles — only registered devices can install debug builds.

1. Plug in your iPhone, open Finder, click on it in the sidebar
2. Click the device info area until you see the **UDID** — copy it
3. developer.apple.com → Certificates, Identifiers & Profiles → Devices → **+**
4. Name: `scotland-mobile`, UDID: paste from above
5. Click **Continue** → **Register**

Or via CLI:
```bash
ios-deploy -c  # Lists connected device UDIDs
```

---

## 4. Create a Development Provisioning Profile

Links the dev certificate + device + App ID so you can deploy debug builds to your phone.

1. developer.apple.com → Certificates, Identifiers & Profiles → Profiles → **+**
2. Under **Development**, select **iOS App Development** → Continue
3. Select App ID: `com.scadoshi.zwipe` → Continue
4. Select your **Apple Development** certificate → Continue
5. Select your device (`scotland-mobile`) → Continue
6. Name it "Zwipe Development" → **Generate**
7. Download the `.mobileprovision` file and **double-click** to install

The profile installs to `~/Library/Developer/Xcode/UserData/Provisioning Profiles/`. `dx build` finds it automatically by matching the bundle ID.

Back up:
```bash
cp ~/Downloads/Zwipe_Development.mobileprovision ~/certs/
```

**If you regenerate your dev certificate** (step 2), you must also regenerate this profile — edit it on Apple's site, select the new cert, download, and double-click to install.

---

## 5. Create an Apple Distribution Certificate

Used for App Store submissions. Same CLI process, different cert type.

```bash
openssl genrsa -out zwipe-dist-key.pem 2048
openssl req -new -key zwipe-dist-key.pem -out CertificateSigningRequest.certSigningRequest \
  -subj "/emailAddress=<apple-id-email>,CN=<your-name>,C=US"
security import zwipe-dist-key.pem -k ~/Library/Keychains/login.keychain-db -T /usr/bin/codesign
```

Then on Apple's site:

1. Certificates, Identifiers & Profiles → Certificates → **+**
2. Select **Apple Distribution** (not Apple Development, not iOS Distribution)
3. Upload `CertificateSigningRequest.certSigningRequest`
4. Download `distribution.cer`, double-click to install

Back up:
```bash
mv zwipe-dist-key.pem CertificateSigningRequest.certSigningRequest ~/certs/
cp ~/Downloads/distribution.cer ~/certs/
```

---

## 6. Create an App Store Provisioning Profile

1. developer.apple.com → Certificates, Identifiers & Profiles → Profiles
2. Click **+** → under **Distribution**, select **App Store Connect** (not Ad Hoc)
3. Click **Continue**
4. Select App ID: `com.scadoshi.zwipe` → **Continue**
5. Select the Apple Distribution certificate → **Continue**
6. Name it "Zwipe App Store" → click **Generate**
7. Download (file will be named something like `Zwipe_App_Store.mobileprovision`)

Back up:
```bash
cp ~/Downloads/Zwipe_App_Store.mobileprovision ~/certs/
```

---

## Verify certificates are installed

```bash
security find-identity -v -p codesigning
# Should show both:
#   "Apple Development: <your name> (...)"
#   "Apple Distribution: <your name> (VV74WQ89GD)"
```

---

## Account reference

| Thing | Value |
|-------|-------|
| Apple ID | see 1Password |
| Paid Team ID | VV74WQ89GD |
| Bundle ID | `com.scadoshi.zwipe` |
| App Store name | Zwipe MTG |
| Device name | `scotland-mobile` |

---

## What to back up

| File | Where | Why |
|------|-------|-----|
| `zwipe-dev-key.pem` | `~/certs/` | Dev cert private key — lose it and you revoke + recreate |
| `zwipe-dist-key.pem` | `~/certs/` | Dist cert private key — same |
| `distribution.cer` | `~/certs/` | Re-downloadable, but keep a copy |
| `Zwipe_App_Store.mobileprovision` | `~/certs/` | Re-downloadable, but keep a copy |

**Keep `~/certs/` backed up** (Time Machine, iCloud, etc.). Private keys cannot be
recovered from Apple.

---

## Why Keychain signing is required

The `keyring` crate uses iOS Keychain for session storage. Without the `keychain-access-groups`
entitlement in `zwiper/Entitlements.plist`, every cold launch produces:
```
Platform secure storage failure: A required entitlement isn't present
```

This requires a paid Apple Developer account — free accounts cannot use Keychain Sharing.
