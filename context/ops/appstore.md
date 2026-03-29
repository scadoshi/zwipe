# App Store Submission Guide

Step-by-step guide for submitting Zwipe to the iOS App Store. Written for a Dioxus 0.7
app with no Xcode project — just `dx build` → manual signing → upload.

---

## Prerequisites

Already done:
- [x] Paid Apple Developer account ($99/yr)
- [x] Bundle ID registered: `com.scadoshi.zwipe`
- [x] App icon: 1024×1024 master + all required sizes
- [x] App name: "Zwipe" (fixed in `Dioxus.toml` + binary rename)
- [x] Account deletion endpoint (Apple guideline 5.1.1)
- [x] Privacy policy at zwipe.net/privacy
- [x] Development signing + device deploy working

---

## Step 1 — Create a Distribution Certificate

You have a Development cert already. App Store requires a separate **Apple Distribution** cert.

**Do NOT use Keychain Access → Certificate Assistant.** It consistently fails with
"The specified item could not be found in the keychain." Generate the CSR from the
command line instead:

```bash
# 1. Generate a private key
openssl genrsa -out zwipe-dist-key.pem 2048

# 2. Create the CSR
openssl req -new -key zwipe-dist-key.pem -out CertificateSigningRequest.certSigningRequest \
  -subj "/emailAddress=<apple-id-email>,CN=<your-name>,C=US"

# 3. Import the private key into your login keychain (codesign needs it there)
security import zwipe-dist-key.pem -k ~/Library/Keychains/login.keychain-db -T /usr/bin/codesign
```

Then on Apple's site:

1. Go to [developer.apple.com](https://developer.apple.com/account) → Certificates, Identifiers & Profiles → Certificates
2. Click **+** → select **Apple Distribution** (not Apple Development, not iOS Distribution)
3. Upload `CertificateSigningRequest.certSigningRequest`
4. Download the `.cer` file and double-click to install in Keychain Access (no visible feedback — it installs silently)

Verify it installed:
```bash
security find-identity -v -p codesigning
# Look for: "Apple Distribution: <your name> (VV74WQ89GD)"
```

### What to keep

| File | Where | Why |
|------|-------|-----|
| `zwipe-dist-key.pem` | `~/certs/` (NOT in repo) | Private key backup — if your Mac dies and this is gone, you must revoke + recreate the cert |
| `distribution.cer` | `~/certs/` | Apple names it just `distribution.cer` — can re-download from developer.apple.com, but keep a copy |
| `CertificateSigningRequest.certSigningRequest` | `~/certs/` | Safe to delete after Apple issues the cert, but harmless to keep |

**Keep `~/certs/` backed up** (Time Machine, iCloud, etc.). The private key cannot be
recovered from Apple — losing it means revoking the cert and creating a new one.

---

## Step 2 — Register the App ID (if not already done)

The App ID must exist before you can create a provisioning profile. If you already
registered `com.scadoshi.zwipe` for development, skip to Step 3.

1. developer.apple.com → Certificates, Identifiers & Profiles → Identifiers
2. Click **+** → select **App IDs** → Continue
3. Select **App** (not App Clip) → Continue
4. Fill out:
   - **Description**: Zwipe
   - **Bundle ID**: select **Explicit**, enter `com.scadoshi.zwipe`
5. Under **Capabilities**, enable **Keychain Sharing** (required for session storage via `keyring` crate)
6. Click **Continue** → **Register**

---

## Step 3 — Create an App Store Provisioning Profile

1. developer.apple.com → Certificates, Identifiers & Profiles → Profiles
2. Click **+** → under **Distribution**, select **App Store Connect** (not Ad Hoc)
3. Click **Continue**
4. Select App ID: `com.scadoshi.zwipe` from the dropdown → **Continue**
5. Select the Apple Distribution certificate from Step 1 → **Continue**
6. Name it "Zwipe App Store" → click **Generate**
7. Click **Download** — file will be named something like `Zwipe_App_Store.mobileprovision`

No device UUIDs needed — App Store profiles aren't device-specific.

Back up the profile:
```bash
cp ~/Downloads/Zwipe_App_Store.mobileprovision ~/certs/
```

### Updated backup table

| File | Where | Why |
|------|-------|-----|
| `zwipe-dist-key.pem` | `~/certs/` | Private key — cannot be recovered from Apple |
| `distribution.cer` | `~/certs/` | Distribution cert — re-downloadable but keep a copy |
| `Zwipe_App_Store.mobileprovision` | `~/certs/` | App Store provisioning profile — re-downloadable but keep a copy |
| `CertificateSigningRequest.certSigningRequest` | `~/certs/` | Safe to delete after cert is issued |

---

## Step 4 — Create Release Entitlements

The current `Entitlements.plist` has `get-task-allow` set to `true` — this is a
debug-only entitlement that lets the debugger attach. Apple will reject it.

Create a release version:

```bash
cp zwiper/Entitlements.plist zwiper/Entitlements-Release.plist
```

Edit `zwiper/Entitlements-Release.plist` — change `get-task-allow` to `false`:

```xml
<key>get-task-allow</key>
<false/>
```

Everything else stays the same (`application-identifier`, `keychain-access-groups`).

---

## Step 5 — Build Release .app

All paths below are absolute from the repo root (`~/Developer/zwipe`).

```bash
cd ~/Developer/zwipe/zwiper
BACKEND_URL=https://api.zwipe.net dx build --release --platform ios --device "scotland-mobile"
```

Verify the `.app` was created:
```bash
ls ~/Developer/zwipe/target/dx/zwipe/release/ios/
# Should contain Zwipe.app
```

### Verify the binary targets iOS (not macOS)

This is a known Dioxus gotcha ([DioxusLabs/dioxus#3817](https://github.com/DioxusLabs/dioxus/issues/3817)).
If the platform metadata is wrong, Apple will reject the upload.

```bash
vtool -show ~/Developer/zwipe/target/dx/zwipe/release/ios/Zwipe.app/zwipe
# Look for: LC_VERSION_MIN_IPHONEOS — this confirms the binary targets iOS
# As of 2026-03-29 the output looks like:
#   cmd LC_VERSION_MIN_IPHONEOS
#   version 10.0
#   sdk 26.4
# Apple may change this format in future Xcode/SDK versions, but the key
# thing is seeing "IPHONEOS" somewhere in the output — NOT "MACOS" or "platform 7" (simulator)
# If it shows MACOS — see "Troubleshooting" at the bottom
```

---

## Step 6 — Sign for Distribution

```bash
# 1. Embed the App Store provisioning profile
cp ~/certs/Zwipe_App_Store.mobileprovision \
   ~/Developer/zwipe/target/dx/zwipe/release/ios/Zwipe.app/embedded.mobileprovision

# 2. Get your signing identity (either the hash or the full name works)
security find-identity -v -p codesigning
# Copy the hash (e.g. D398244D...) or the quoted name (e.g. "Apple Distribution: ...")

# 3. Re-sign with release entitlements
codesign --force --sign "<HASH-OR-NAME>" \
  --entitlements ~/Developer/zwipe/zwiper/Entitlements-Release.plist \
  ~/Developer/zwipe/target/dx/zwipe/release/ios/Zwipe.app
# Should print: "replacing existing signature"
```

---

## Step 7 — Package as IPA

```bash
cd ~/Developer/zwipe
mkdir -p Payload
cp -r target/dx/zwipe/release/ios/Zwipe.app Payload/
zip -r Zwipe.ipa Payload
rm -rf Payload
# Zwipe.ipa is now in ~/Developer/zwipe/
```

---

## Step 8 — App Store Connect Setup

Go to [appstoreconnect.apple.com](https://appstoreconnect.apple.com).

### Create App Record (one-time)

| Field | Value |
|-------|-------|
| Platform | iOS |
| Name | Zwipe MTG (note: "Zwipe" alone was taken) |
| Primary Language | English (U.S.) |
| Bundle ID | com.scadoshi.zwipe |
| SKU | zwipe001 |

### Version Information (required for review)

| Field | Notes |
|-------|-------|
| **Screenshots** | Required: 6.7" iPhone (iPhone 15 Pro Max). Take on device or simulator. At least 1, up to 10 per size. |
| **Description** | Up to 4000 chars. What the app does, key features. |
| **Keywords** | Up to 100 chars, comma-separated: `MTG,Magic the Gathering,deck builder,commander,EDH,card game` |
| **Support URL** | `https://zwipe.net` |
| **Privacy Policy URL** | `https://zwipe.net/privacy` |
| **Category** | Games → Card Games |
| **Age Rating** | Fill out questionnaire — answer No to everything. Result: 4+ |
| **Copyright** | `2026 scadoshi` |

### App Review Information

| Field | Notes |
|-------|-------|
| Contact name | Your name |
| Contact phone | Your phone |
| Contact email | Your email |
| Demo account | Provide a test email/password so reviewers can log in |
| Notes | Optional — explain anything non-obvious about the app |

### App Privacy (Data Collection)

Mandatory. Declare what data the app collects:

- **Contact Info** → Email Address (account creation)
- **Identifiers** → User ID (internal user ID)

Select "Used for App Functionality" and "Not linked to user's identity" as appropriate.

---

## Step 9 — Upload

### Option A: Transporter (recommended for first time)

1. Download **Transporter** from the Mac App Store (free, made by Apple)
2. Sign in with your Apple ID
3. Drag `Zwipe.ipa` into the window
4. Click **Deliver**

### Option B: CLI with altool

First, create an App Store Connect API key:
1. App Store Connect → Users and Access → Integrations → App Store Connect API
2. Click **+**, name it, give it Admin access
3. Download the `.p8` file → save to `~/.private_keys/AuthKey_<KEY_ID>.p8`
4. Note the **Key ID** and **Issuer ID**

Then validate and upload:
```bash
# Validate first (catches most issues without submitting)
xcrun altool --validate-app -f Zwipe.ipa -t ios \
  --apiKey <KEY_ID> --apiIssuer <ISSUER_ID>

# Upload
xcrun altool --upload-app -f Zwipe.ipa -t ios \
  --apiKey <KEY_ID> --apiIssuer <ISSUER_ID>
```

---

## Step 10 — Submit for Review

1. Back in App Store Connect, the build should appear under your app version (may take 5–10 min to process)
2. Select the build
3. **Export Compliance**: Does your app use encryption? → **No** (only HTTPS, which is exempt)
4. Click **Submit for Review**
5. Typical review time: 1–3 days

---

## Troubleshooting

### "Incorrect Platform" rejection on upload

Dioxus issue [#3817](https://github.com/DioxusLabs/dioxus/issues/3817). The binary has
macOS platform metadata instead of iOS. Fix by building the binary manually:

```bash
cargo build --release --target aarch64-apple-ios -p zwiper
```

Then assemble the `.app` bundle manually using the `Info.plist` that `dx build` generates,
copy the binary in, embed the provisioning profile, and sign.

### Missing Info.plist keys

If Apple rejects for missing metadata, you may need to add these to the generated `Info.plist`:

```xml
<key>DTPlatformName</key>
<string>iphoneos</string>
<key>MinimumOSVersion</key>
<string>16.0</string>
<key>CFBundlePackageType</key>
<string>APPL</string>
```

Check if Dioxus.toml supports `[bundle.ios.plist]` for adding extra keys — otherwise
patch the plist after build with `/usr/libexec/PlistBuddy`.

### LaunchScreen.storyboard missing

If the generated `Info.plist` references `UILaunchStoryboardName = LaunchScreen`, verify
the storyboard file exists in the `.app` bundle. If not, either add one or remove the
plist key.

---

## After Approval

- Update `zwipe.net/download` with the App Store link
- Update `README.md` status
- Update `context/status/todo.md` and `progress.md`
