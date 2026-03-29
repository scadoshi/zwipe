# Upload a New App Store Version

Short guide for submitting updates after the initial submission. For first-time
setup, see [appstore-first.md](appstore-first.md).

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

## 6. Upload

Open **Transporter** (Mac App Store) → drag `Zwipe.ipa` → **Deliver**.

## 7. Submit

1. App Store Connect → your app → build appears after 5–10 min
2. Create new version if needed (click **+** next to iOS App)
3. Select the build
4. Export Compliance → **No**
5. **Submit for Review**

---

## Version bumping

- `CFBundleShortVersionString` = the user-facing version (1.0, 1.1, 2.0)
- `CFBundleVersion` = the build number, must increment with each upload (1, 2, 3...)
- Apple rejects uploads with a previously used build number
