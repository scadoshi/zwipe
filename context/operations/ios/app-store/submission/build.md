# Build the iOS app (release)

Produce a signed `Zwipe.ipa` ready to upload. This is the recurring build for version updates; for the one-time account/cert/App-ID setup see [first_release.md](first_release.md). To upload + submit the `.ipa` this produces, continue to [publish.md](publish.md).

---

## IMPORTANT: Always use the latest Xcode

Apple's App Store submission allowlist requires the binary to be linked against the very latest Xcode/SDK. Older GM versions get rejected at "Add for Review" with a misleading "beta Xcode" UI message (actual API error: `BUILD_SDK_NOT_ALLOWED_FOR_APP_STORE_SUBMISSION`). See [debugging.md](debugging.md) for the full investigation.

**Before building, update Xcode via the Mac App Store** and verify:

```bash
xcodebuild -version
# Make sure this matches the most recent Xcode release
```

After updating Xcode, also wipe the cargo iOS device cache so the binary actually re-links against the new SDK (otherwise Cargo will reuse the previously linked object files):

```bash
rm -rf ~/Developer/zwipe/target/aarch64-apple-ios
rm -rf ~/Developer/zwipe/target/dx/zwipe/release/ios
```

If you don't have the matching iOS simulator runtime installed, `actool` will fail to produce `Assets.car`. Install it once:

```bash
xcodebuild -downloadPlatform iOS
```

---

## 1. Build

```bash
cd ~/Developer/zwipe/zwiper
BACKEND_URL=https://api.zwipe.net dx build --release --platform ios --device "scotland-mobile"
```

From the workspace root instead, add `--package zwiper`; `dx` refuses to guess which binary crate to build and lists zerver/zite/zwiper. Same for `dx bundle --platform android`.

## 2. Patch Info.plist (Dioxus doesn't generate these correctly)

```bash
APP=~/Developer/zwipe/target/dx/zwipe/release/ios/Zwipe.app

# Remove iPadOS from supported platforms (Apple requires exactly one)
/usr/libexec/PlistBuddy \
  -c "Delete :CFBundleSupportedPlatforms" \
  -c "Add :CFBundleSupportedPlatforms array" \
  -c "Add :CFBundleSupportedPlatforms:0 string iPhoneOS" \
  $APP/Info.plist

# Remove iPad from device family (iPhone only)
/usr/libexec/PlistBuddy \
  -c "Delete :UIDeviceFamily" \
  -c "Add :UIDeviceFamily array" \
  -c "Add :UIDeviceFamily:0 integer 1" \
  $APP/Info.plist

# Set the marketing version (must match the release + the App Store Connect version
# you create) and the build number. Dioxus writes 1.0 from a stale template.
/usr/libexec/PlistBuddy \
  -c "Set :CFBundleShortVersionString <MARKETING_VERSION>" \
  -c "Set :CFBundleVersion <BUILD_NUMBER>" \
  $APP/Info.plist
# <MARKETING_VERSION> = the release version, e.g. 1.7.0 (matches Cargo.toml + the
# App Store Connect version). Do NOT leave it at 1.0; Transporter/App Store Connect
# then show "1.0 (build)" which reads as the wrong release.
# <BUILD_NUMBER> increments every upload (65, 66, ...) and must be unique.

# Patch DT/SDK keys to match the active Xcode (Dioxus hard-codes these from a
# stale template). Required by Apple's submission allowlist. Pull live values:
XCODE_BUILD=$(xcodebuild -version | awk '/Build version/ {print $3}')
XCODE_VERSION=$(xcodebuild -version | awk 'NR==1 {gsub(/\./,"",$2); printf "%s0", $2}')
SDK_VERSION=$(xcrun --sdk iphoneos --show-sdk-version)
SDK_BUILD=$(xcrun --sdk iphoneos --show-sdk-build-version)
OS_BUILD=$(sw_vers -buildVersion)

for kv in \
  "DTXcode:string:$XCODE_VERSION" \
  "DTXcodeBuild:string:$XCODE_BUILD" \
  "DTSDKName:string:iphoneos$SDK_VERSION" \
  "DTSDKBuild:string:$SDK_BUILD" \
  "DTPlatformName:string:iphoneos" \
  "DTPlatformVersion:string:$SDK_VERSION" \
  "DTPlatformBuild:string:$SDK_BUILD" \
  "BuildMachineOSBuild:string:$OS_BUILD"; do
  KEY="${kv%%:*}"; REST="${kv#*:}"; TYPE="${REST%%:*}"; VAL="${REST#*:}"
  /usr/libexec/PlistBuddy -c "Delete :$KEY" $APP/Info.plist 2>/dev/null
  /usr/libexec/PlistBuddy -c "Add :$KEY $TYPE $VAL" $APP/Info.plist
done
```

## 3. Compile app icons

```bash
~/Developer/zwipe/zcripts/ios/icons.sh $APP
```

Compiles the gruvbox primary from `zwiper/assets/favicon` and one alternate per theme from `zwiper/assets/app-icons` into `Assets.car`, and writes `CFBundleIcons` (primary plus `CFBundleAlternateIcons`) into `Info.plist`. The alternates are what Profile's App icon sheet switches between; `zcripts/ios/icon.py` re-renders them when orangui adds a theme. It should print `icons: Assets.car, primary + 30 alternates`.

## 4. Sign

```bash
# Get your signing identity
security find-identity -v -p codesigning
# Copy the hash or name for "Apple Distribution"

# Embed provisioning profile
cp ~/certs/Zwipe_App_Store.mobileprovision $APP/embedded.mobileprovision

# Sign
codesign --force --sign "<HASH-OR-NAME>" \
  --entitlements ~/Developer/zwipe/zwiper/Entitlements-Release.plist $APP
```

## 5. Package

```bash
cd ~/Developer/zwipe
rm -f Zwipe.ipa
mkdir -p Payload
cp -r target/dx/zwipe/release/ios/Zwipe.app Payload/
zip -r Zwipe.ipa Payload
rm -rf Payload
```

**Next:** `open -a Transporter ~/Developer/zwipe/Zwipe.ipa` opens it in Transporter ready to Deliver; the rest of upload + submit is in [publish.md](publish.md).

---

## Version bumping

- `CFBundleShortVersionString` = the user-facing version (1.0, 1.1, 2.0)
- `CFBundleVersion` = the build number, must increment with each upload (1, 2, 3...)
- Apple rejects uploads with a previously used build number
