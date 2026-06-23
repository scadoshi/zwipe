# First App Store Submission

Full walkthrough for the initial App Store submission. For subsequent version
uploads, see [appstore_update.md](appstore_update.md).

**Prerequisites:** All steps in [setup.md](setup.md) completed (certs, App ID, App Store
provisioning profile).

---

## Step 1 — Create Release Entitlements

The debug `Entitlements.plist` has `get-task-allow` set to `true` (allows debugger
to attach). Apple rejects this. Create a release version:

```bash
cp ~/Developer/zwipe/zwiper/Entitlements.plist ~/Developer/zwipe/zwiper/Entitlements-Release.plist
```

Edit `Entitlements-Release.plist` — change `get-task-allow` to `false`:

```xml
<key>get-task-allow</key>
<false/>
```

This only needs to be done once — the file is committed to the repo.

---

## Step 2 — Build Release .app

```bash
cd ~/Developer/zwipe/zwiper
BACKEND_URL=https://api.zwipe.net dx build --release --platform ios --device "scotland-mobile"
```

Verify:
```bash
ls ~/Developer/zwipe/target/dx/zwipe/release/ios/
# Should contain Zwipe.app
```

### Verify the binary targets iOS

```bash
vtool -show ~/Developer/zwipe/target/dx/zwipe/release/ios/Zwipe.app/zwipe
# Look for: LC_VERSION_MIN_IPHONEOS
# See troubleshooting.md if it shows MACOS
```

---

## Step 3 — Sign for Distribution

```bash
# 1. Embed the App Store provisioning profile
cp ~/certs/Zwipe_App_Store.mobileprovision \
   ~/Developer/zwipe/target/dx/zwipe/release/ios/Zwipe.app/embedded.mobileprovision

# 2. Get your signing identity
security find-identity -v -p codesigning
# Copy the hash or quoted name for the "Apple Distribution" entry

# 3. Re-sign with release entitlements
codesign --force --sign "<HASH-OR-NAME>" \
  --entitlements ~/Developer/zwipe/zwiper/Entitlements-Release.plist \
  ~/Developer/zwipe/target/dx/zwipe/release/ios/Zwipe.app
# Should print: "replacing existing signature"
```

---

## Step 4 — Package as IPA

```bash
cd ~/Developer/zwipe
mkdir -p Payload
cp -r target/dx/zwipe/release/ios/Zwipe.app Payload/
zip -r Zwipe.ipa Payload
rm -rf Payload
# Zwipe.ipa is now in ~/Developer/zwipe/
```

---

## Step 5 — App Store Connect Setup (one-time)

Go to [appstoreconnect.apple.com](https://appstoreconnect.apple.com).

### Create App Record

| Field | Value |
|-------|-------|
| Platform | iOS |
| Name | Zwipe MTG (note: "Zwipe" alone was taken) |
| Primary Language | English (U.S.) |
| Bundle ID | com.scadoshi.zwipe |
| SKU | zwipe001 |

### Version Information

| Field | Notes |
|-------|-------|
| **Screenshots** | 6.5" iPhone required (1242×2688). Use iPhone 11 Pro Max simulator. See [simulator.md](simulator.md). |
| **Description** | Up to 4000 chars. What the app does, key features. |
| **Keywords** | Up to 100 chars: `MTG,Magic the Gathering,deck builder,commander,EDH,card game` |
| **Support URL** | `https://zwipe.net` |
| **Privacy Policy URL** | `https://zwipe.net/privacy` |
| **Category** | Games → Card Games |
| **Age Rating** | Answer No to everything. Result: 4+ |
| **Copyright** | `2026 scadoshi` |

### App Review Information

| Field | Notes |
|-------|-------|
| Contact name | Your name |
| Contact phone | Your phone |
| Contact email | Your email |
| Demo account | Provide a test email/password so reviewers can log in |
| Notes | Optional |

### App Privacy (Data Collection)

- **Contact Info** → Email Address (account creation)
- **Identifiers** → User ID (internal user ID)
- Select "Used for App Functionality"

---

## Step 6 — Upload

### Prerequisites: App Store Connect API key

If you haven't created one yet:
1. App Store Connect → Users and Access → Integrations → App Store Connect API
2. Click **+**, name it "CLI Upload", give it **Admin** access
3. Download the `.p8` file (one-time download!) → `~/.private_keys/AuthKey_<KEY_ID>.p8`
4. Note the **Key ID** and **Issuer ID**

### Upload via Transporter (recommended)

**Do NOT use `xcrun altool`** — it is deprecated and causes metadata parsing errors
that can trigger false "beta Xcode" rejections in App Store Connect.

**Do NOT use `xcrun iTMSTransporter`** — it expects `.itmsp` directories, not `.ipa` files.

1. Download **Transporter** from the Mac App Store (free, by Apple)
2. Open Transporter, sign in with your Apple ID
3. Drag `~/Developer/zwipe/Zwipe.ipa` into the window
4. Click **Deliver** — validates and uploads in one step
5. Wait for "Upload Successful" confirmation

The build will appear in App Store Connect after 5–10 minutes.

### Fallback: altool (deprecated — use only if Transporter is unavailable)

```bash
xcrun altool --validate-app -f ~/Developer/zwipe/Zwipe.ipa -t ios \
  --apiKey C2L47TDDPV --apiIssuer 644db668-17b6-4d50-ac1a-70f8ea838d0d

xcrun altool --upload-app -f ~/Developer/zwipe/Zwipe.ipa -t ios \
  --apiKey C2L47TDDPV --apiIssuer 644db668-17b6-4d50-ac1a-70f8ea838d0d
```

API key file: `~/.private_keys/AuthKey_C2L47TDDPV.p8`

---

## Step 7 — Submit for Review

1. Back in App Store Connect, the build appears under your app version (may take 5–10 min)
2. Select the build
3. **Export Compliance**: Does your app use encryption? → **No** (HTTPS is exempt)
4. Click **Submit for Review**
5. Typical review time: 1–3 days

---

## After Approval

- Update `zwipe.net/download` with the App Store link
- Update `README.md` status
- Update `context/progress/todo.md` and `overview.md`
