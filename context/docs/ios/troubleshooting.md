# iOS Troubleshooting

Common errors and fixes when building, signing, or submitting Zwipe.

## Dioxus post-build patching checklist

Dioxus `dx build` generates an incomplete `.app` bundle for App Store submission.
These patches must be applied **after every release build, before signing**:

1. **CFBundleSupportedPlatforms** — remove iPadOS, keep only iPhoneOS
2. **UIDeviceFamily** — remove iPad (2), keep only iPhone (1)
3. **App icons** — compile asset catalog with `actool`
4. **CFBundleIcons** — add icon references to Info.plist

The following are handled by `[ios.plist]` in `Dioxus.toml` (no manual patching):
- DTPlatformName, DTPlatformVersion, DTSDKName, DTXcode, DTXcodeBuild, DTCompiler
- MinimumOSVersion
- CFBundlePackageType

See `appstore-update.md` for the full build → patch → sign → package workflow.

---

## "Incorrect Platform" rejection on upload

Dioxus issue [#3817](https://github.com/DioxusLabs/dioxus/issues/3817). The binary has
macOS platform metadata instead of iOS.

**Diagnose:**
```bash
vtool -show ~/Developer/zwipe/target/dx/zwipe/release/ios/Zwipe.app/zwipe
# Should show: LC_VERSION_MIN_IPHONEOS
# Bad: shows MACOS or platform 7
```

**Fix:** Build the binary manually with the correct target:
```bash
cargo build --release --target aarch64-apple-ios -p zwiper
```
Then assemble the `.app` bundle manually using the `Info.plist` that `dx build` generates,
copy the binary in, embed the provisioning profile, and sign.

---

## Missing Info.plist keys (Transporter validation failures)

Dioxus `dx build` doesn't inject the `DT*` (Developer Tools) metadata or `MinimumOSVersion`
that Apple's Transporter requires. Xcode normally adds these during archiving, but since
we bypass Xcode, we have to provide them ourselves.

**Errors you'll see:**
- `Missing Info.plist value. A value for the key 'DTPlatformName' in bundle Zwipe.app is required.`
- `Invalid MinimumOSVersion. Apps that only support 64-bit devices must specify a deployment target of 8.0 or later. MinimumOSVersion in 'Zwipe.app' is ''.`

**Permanent fix:** These keys are now in `zwiper/Dioxus.toml` under `[ios.plist]`, so
future `dx build` runs include them automatically. If you still hit this error, check
that the `[ios.plist]` section hasn't been removed.

**Manual fix (if patching an existing build):**
```bash
/usr/libexec/PlistBuddy \
  -c "Add :DTPlatformName string iphoneos" \
  -c "Add :DTPlatformVersion string 26.4" \
  -c "Add :DTSDKName string iphoneos26.4" \
  -c "Add :DTXcode string 1640" \
  -c "Add :DTXcodeBuild string 16A242d" \
  -c "Add :DTCompiler string com.apple.compilers.llvm.clang.1_0" \
  -c "Add :MinimumOSVersion string 16.0" \
  ~/Developer/zwipe/target/dx/zwipe/release/ios/Zwipe.app/Info.plist
```

After patching, you **must** re-sign and re-package the IPA — changing the plist
invalidates the code signature.

**Version mismatch:** Dioxus generates `CFBundleShortVersionString` from `Cargo.toml`
(e.g. `0.1.0`), but App Store Connect expects what you set there (e.g. `1.0`). Patch
if needed:
```bash
/usr/libexec/PlistBuddy \
  -c "Set :CFBundleShortVersionString 1.0" \
  -c "Set :CFBundleVersion 1" \
  ~/Developer/zwipe/target/dx/zwipe/release/ios/Zwipe.app/Info.plist
```

---

## CFBundleSupportedPlatforms contains multiple values

Dioxus generates `CFBundleSupportedPlatforms` with both `iPhoneOS` and `iPadOS`. Apple
requires exactly one value. This cannot be fixed via `Dioxus.toml` (it's an array, not
a string), so it must be patched after every release build.

**Error:**
`Invalid CFBundleSupportedPlatforms value ... contains multiple platform values: [iPhoneOS, iPadOS]`

**Fix (after build, before signing):**
```bash
/usr/libexec/PlistBuddy \
  -c "Delete :CFBundleSupportedPlatforms" \
  -c "Add :CFBundleSupportedPlatforms array" \
  -c "Add :CFBundleSupportedPlatforms:0 string iPhoneOS" \
  ~/Developer/zwipe/target/dx/zwipe/release/ios/Zwipe.app/Info.plist
```

Then re-sign and re-package.

---

## CFBundlePackageType missing

Dioxus doesn't set `CFBundlePackageType` which Apple requires to identify the bundle as
an application.

**Error:**
`Invalid Bundle OS Type code. The CFBundlePackageType value ... must be one of the following Bundle OS Type codes: [APPL].`

**Permanent fix:** Added to `zwiper/Dioxus.toml` under `[ios.plist]`:
```toml
CFBundlePackageType = "APPL"
```

**Manual fix:**
```bash
/usr/libexec/PlistBuddy -c "Add :CFBundlePackageType string APPL" \
  ~/Developer/zwipe/target/dx/zwipe/release/ios/Zwipe.app/Info.plist
```

---

## UIDeviceFamily includes iPad — missing iPad icons

Dioxus sets `UIDeviceFamily` to `[1, 2]` (iPhone + iPad). If you don't want to support
iPad, Apple will still require iPad icon sizes (152×152, 167×167, etc.).

**Error:**
`Missing required icon file. The bundle does not contain an app icon for iPad of exactly '152x152' pixels...`

**Fix — remove iPad from UIDeviceFamily (after build, before signing):**
```bash
/usr/libexec/PlistBuddy \
  -c "Delete :UIDeviceFamily" \
  -c "Add :UIDeviceFamily array" \
  -c "Add :UIDeviceFamily:0 integer 1" \
  ~/Developer/zwipe/target/dx/zwipe/release/ios/Zwipe.app/Info.plist
```

This cannot be fixed via `Dioxus.toml` (array value). Must be patched after every build.

---

## Missing app icons (no Assets.car)

Dioxus doesn't run `actool` to compile app icons into an asset catalog. Without
`Assets.car` in the `.app` bundle, Apple rejects the upload.

**Error:**
`Missing required icon file. The bundle does not contain an app icon for iPhone / iPod Touch of exactly '120x120' pixels...`

**Fix — compile an asset catalog and embed it:**
```bash
# 1. Create the asset catalog source
mkdir -p /tmp/Assets.xcassets/AppIcon.appiconset
cat > /tmp/Assets.xcassets/AppIcon.appiconset/Contents.json << 'EOF'
{
  "images": [
    {"size":"20x20","idiom":"iphone","scale":"2x","filename":"icon-40.png"},
    {"size":"20x20","idiom":"iphone","scale":"3x","filename":"icon-60.png"},
    {"size":"29x29","idiom":"iphone","scale":"2x","filename":"icon-60.png"},
    {"size":"29x29","idiom":"iphone","scale":"3x","filename":"icon-87.png"},
    {"size":"40x40","idiom":"iphone","scale":"2x","filename":"icon-80.png"},
    {"size":"40x40","idiom":"iphone","scale":"3x","filename":"icon-120.png"},
    {"size":"60x60","idiom":"iphone","scale":"2x","filename":"icon-120.png"},
    {"size":"60x60","idiom":"iphone","scale":"3x","filename":"icon-180.png"},
    {"size":"1024x1024","idiom":"ios-marketing","scale":"1x","filename":"icon-1024.png"}
  ],
  "info":{"version":1,"author":"xcode"}
}
EOF
cat > /tmp/Assets.xcassets/Contents.json << 'EOF'
{"info":{"version":1,"author":"xcode"}}
EOF

# 2. Copy icon PNGs into the asset catalog
cp ~/Developer/zwipe/zwiper/assets/favicon/icon-{40,60,80,87,120,180,1024}.png \
   /tmp/Assets.xcassets/AppIcon.appiconset/

# 3. Compile with actool
actool --compile ~/Developer/zwipe/target/dx/zwipe/release/ios/Zwipe.app \
  --platform iphoneos --minimum-deployment-target 16.0 --app-icon AppIcon \
  --output-partial-info-plist /tmp/assetcatalog_generated_info.plist \
  /tmp/Assets.xcassets

# 4. Add icon references to Info.plist
/usr/libexec/PlistBuddy \
  -c "Add :CFBundleIcons dict" \
  -c "Add :CFBundleIcons:CFBundlePrimaryIcon dict" \
  -c "Add :CFBundleIcons:CFBundlePrimaryIcon:CFBundleIconFiles array" \
  -c "Add :CFBundleIcons:CFBundlePrimaryIcon:CFBundleIconFiles:0 string AppIcon60x60" \
  -c "Add :CFBundleIcons:CFBundlePrimaryIcon:CFBundleIconName string AppIcon" \
  ~/Developer/zwipe/target/dx/zwipe/release/ios/Zwipe.app/Info.plist
```

This produces `Assets.car` and `AppIcon60x60@2x.png` inside the `.app` bundle.
Must be done after every release build, before signing.

---

## LaunchScreen.storyboard missing

If the generated `Info.plist` references `UILaunchStoryboardName = LaunchScreen`, verify
the storyboard file exists in the `.app` bundle. If not, either add one or remove the
plist key.

---

## "no identity found" when signing

Usually caused by a typo or line break in the identity string. Use the **hash** instead
of the name:

```bash
security find-identity -v -p codesigning
# Copy the hex hash (e.g. D398244DC213B1CF...)

codesign --force --sign "D398244DC213B1CF..." ...
```

---

## Certificate Assistant fails

Keychain Access → Certificate Assistant consistently fails with "The specified item
could not be found in the keychain." Always use the CLI instead:

```bash
openssl genrsa -out key.pem 2048
openssl req -new -key key.pem -out CSR.certSigningRequest \
  -subj "/emailAddress=<email>,CN=<name>,C=US"
security import key.pem -k ~/Library/Keychains/login.keychain-db -T /usr/bin/codesign
```

---

## Provisioning profile disappears

macOS removes profiles if no matching private key is in Keychain. Re-import the key:

```bash
security import ~/certs/zwipe-dev-key.pem -k ~/Library/Keychains/login.keychain-db -T /usr/bin/codesign
```

Then re-download and install the profile.

---

## Team ID confusion

Xcode's "Manage Certificates" creates certs under the Personal Team, not the paid team.
The `(NVSWB62C54)` shown by `security find-identity` is the CN display name — the OU
field is the actual team ID.

For App Store submission, use the cert with `(VV74WQ89GD)` — that's the paid team.

```bash
security find-identity -v -p codesigning
# "Apple Distribution: ... (VV74WQ89GD)" ← this one
# "Apple Development: ... (NVSWB62C54)"  ← personal team, dev only
```

---

## Certificate deleted / "identity no longer valid" (0xe8008018) or "valid provisioning profile not found" (0xe8008015)

Happens when duplicate certificates are cleaned up from Keychain Access — the provisioning profile was tied to the deleted cert.

**Fix:**

1. Create a new development cert in **Xcode > Settings > Accounts > Manage Certificates > + > Apple Development**
2. Delete any remaining old/duplicate certs from Keychain Access (keep only the newest one — check the date)
3. Go to [developer.apple.com/account/resources/profiles](https://developer.apple.com/account/resources/profiles)
4. Edit (or create) the iOS Development profile for `com.scadoshi.zwipe`
5. Select the new certificate (Xcode may label it with your Mac hostname, e.g. "scotland2")
6. Make sure your device is included
7. Download the `.mobileprovision` file and **double-click** it to install
8. Delete the old cached profile and clean build:

```bash
rm /Users/scottyrayfermo/Library/Developer/Xcode/UserData/Provisioning\ Profiles/*.mobileprovision
rm -rf ~/Developer/zwipe/target/dx/zwipe/debug/ios/
cd ~/Developer/zwipe/zwiper
BACKEND_URL=https://api.zwipe.net dx build --platform ios --device "scotland-mobile"
ios-deploy --bundle ~/Developer/zwipe/target/dx/zwipe/debug/ios/Zwipe.app
```

**Why duplicates happen:** Each time you create a cert in Xcode or via Keychain Access, Apple issues a new cert with the same team ID but a different serial. The old one stays in your keychain. If `dx build` picks the wrong one (ambiguous match), the signature won't match the provisioning profile.

**Prevention:** After creating a new cert, immediately delete all older certs with the same name from Keychain Access before building.

---

## "Untrusted Developer" on first launch

Settings → VPN & Device Management → your Apple ID → Trust

---

## Developer Mode required (iOS 16+)

Settings → Privacy & Security → Developer Mode → toggle on → restart device
