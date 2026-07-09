# Build the iOS app (release)

Produce a signed `Zwipe.ipa` ready to upload. This is the recurring build for
version updates; for the one-time account/cert/App-ID setup see
[first_release.md](first_release.md). To upload + submit the `.ipa` this
produces, continue to [publish.md](publish.md).

---

## IMPORTANT: Always use the latest Xcode

Apple's App Store submission allowlist requires the binary to be linked against the
very latest Xcode/SDK. Older GM versions get rejected at "Add for Review" with a
misleading "beta Xcode" UI message (actual API error:
`BUILD_SDK_NOT_ALLOWED_FOR_APP_STORE_SUBMISSION`). See [debugging.md](debugging.md)
for the full investigation.

**Before building, update Xcode via the Mac App Store** and verify:

```bash
xcodebuild -version
# Make sure this matches the most recent Xcode release
```

After updating Xcode, also wipe the cargo iOS device cache so the binary actually
re-links against the new SDK (otherwise Cargo will reuse the previously linked
object files):

```bash
rm -rf ~/Developer/zwipe/target/aarch64-apple-ios
rm -rf ~/Developer/zwipe/target/dx/zwipe/release/ios
```

If you don't have the matching iOS simulator runtime installed, `actool` will fail
to produce `Assets.car`. Install it once:

```bash
xcodebuild -downloadPlatform iOS
```

---

## 1. Build

```bash
cd ~/Developer/zwipe/zwiper
BACKEND_URL=https://api.zwipe.net dx build --release --platform ios --device "scotland-mobile"
```

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

# Fix version to match App Store Connect (Dioxus uses Cargo.toml version)
/usr/libexec/PlistBuddy \
  -c "Set :CFBundleShortVersionString 1.0" \
  -c "Set :CFBundleVersion <BUILD_NUMBER>" \
  $APP/Info.plist
# Increment BUILD_NUMBER each upload (1, 2, 3...). Version can stay 1.0 for patches.

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
cp ~/Developer/zwipe/zwiper/assets/favicon/icon-{40,60,80,87,120,180,1024}.png \
   /tmp/Assets.xcassets/AppIcon.appiconset/

actool --compile $APP --platform iphoneos --minimum-deployment-target 16.0 \
  --app-icon AppIcon --output-partial-info-plist /tmp/assetcatalog_generated_info.plist \
  /tmp/Assets.xcassets

/usr/libexec/PlistBuddy \
  -c "Add :CFBundleIcons dict" \
  -c "Add :CFBundleIcons:CFBundlePrimaryIcon dict" \
  -c "Add :CFBundleIcons:CFBundlePrimaryIcon:CFBundleIconFiles array" \
  -c "Add :CFBundleIcons:CFBundlePrimaryIcon:CFBundleIconFiles:0 string AppIcon60x60" \
  -c "Add :CFBundleIcons:CFBundlePrimaryIcon:CFBundleIconName string AppIcon" \
  $APP/Info.plist
```

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

**Next:** upload + submit the `.ipa` → [publish.md](publish.md).

---

## Version bumping

- `CFBundleShortVersionString` = the user-facing version (1.0, 1.1, 2.0)
- `CFBundleVersion` = the build number, must increment with each upload (1, 2, 3...)
- Apple rejects uploads with a previously used build number
