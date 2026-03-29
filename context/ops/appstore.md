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

1. Open **Keychain Access** → Certificate Assistant → Request a Certificate From a Certificate Authority
2. Enter your Apple ID email, select "Saved to disk", save the `.certSigningRequest` file
3. Go to [developer.apple.com](https://developer.apple.com/account) → Certificates, Identifiers & Profiles → Certificates
4. Click **+** → select **Apple Distribution** (not Apple Development, not iOS Distribution)
5. Upload your CSR, download the `.cer` file
6. Double-click the `.cer` to install in Keychain Access

Verify it installed:
```bash
security find-identity -v -p codesigning
# Look for: "Apple Distribution: <your name> (VV74WQ89GD)"
```

---

## Step 2 — Create an App Store Provisioning Profile

1. developer.apple.com → Certificates, Identifiers & Profiles → Profiles
2. Click **+** → under **Distribution**, select **App Store** (not Ad Hoc)
3. Select App ID: `com.scadoshi.zwipe`
4. Select the Apple Distribution certificate from Step 1
5. Name it "Zwipe App Store", click Generate
6. Download the `.mobileprovision` file

No device UUIDs needed — App Store profiles aren't device-specific.

---

## Step 3 — Create Release Entitlements

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

## Step 4 — Build Release .app

```bash
cd zwiper

# Build release targeting physical iOS device
BACKEND_URL=https://api.zwipe.net dx build --release --platform ios --device "scotland-mobile"
```

Find the `.app` bundle:
```bash
ls ../target/dx/zwipe/release/ios/
# Should contain Zwipe.app
```

### Verify the binary targets iOS (not macOS)

This is a known Dioxus gotcha ([DioxusLabs/dioxus#3817](https://github.com/DioxusLabs/dioxus/issues/3817)).
If the platform metadata is wrong, Apple will reject the upload.

```bash
vtool -show ../target/dx/zwipe/release/ios/Zwipe.app/zwipe
# Must show: platform: IOS
# If it shows: platform: MACOS — see "Troubleshooting" at the bottom
```

---

## Step 5 — Sign for Distribution

```bash
# 1. Embed the App Store provisioning profile
cp ~/path/to/ZwipeAppStore.mobileprovision \
   ../target/dx/zwipe/release/ios/Zwipe.app/embedded.mobileprovision

# 2. Re-sign with your Apple Distribution cert + release entitlements
codesign --force --sign "Apple Distribution: <your name> (VV74WQ89GD)" \
  --entitlements zwiper/Entitlements-Release.plist \
  ../target/dx/zwipe/release/ios/Zwipe.app
```

Replace `<your name>` with the CN from `security find-identity -v -p codesigning`.

---

## Step 6 — Package as IPA

```bash
mkdir -p Payload
cp -r ../target/dx/zwipe/release/ios/Zwipe.app Payload/
zip -r Zwipe.ipa Payload
rm -rf Payload
```

---

## Step 7 — App Store Connect Setup

Go to [appstoreconnect.apple.com](https://appstoreconnect.apple.com).

### Create App Record (one-time)

| Field | Value |
|-------|-------|
| Platform | iOS |
| Name | Zwipe |
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

## Step 8 — Upload

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

## Step 9 — Submit for Review

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
