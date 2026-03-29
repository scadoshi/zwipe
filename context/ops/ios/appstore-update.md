# Upload a New App Store Version

Short guide for submitting updates after the initial submission. For first-time
setup, see [appstore-first.md](appstore-first.md).

---

## 1. Build

```bash
cd ~/Developer/zwipe/zwiper
BACKEND_URL=https://api.zwipe.net dx build --release --platform ios --device "scotland-mobile"
```

## 2. Sign

```bash
cp ~/certs/Zwipe_App_Store.mobileprovision \
   ~/Developer/zwipe/target/dx/zwipe/release/ios/Zwipe.app/embedded.mobileprovision

codesign --force --sign "<HASH-OR-NAME>" \
  --entitlements ~/Developer/zwipe/zwiper/Entitlements-Release.plist \
  ~/Developer/zwipe/target/dx/zwipe/release/ios/Zwipe.app
```

Get `<HASH-OR-NAME>` from `security find-identity -v -p codesigning` (the Apple Distribution entry).

## 3. Package

```bash
cd ~/Developer/zwipe
mkdir -p Payload
cp -r target/dx/zwipe/release/ios/Zwipe.app Payload/
zip -r Zwipe.ipa Payload
rm -rf Payload
```

## 4. Create new version in App Store Connect

1. [appstoreconnect.apple.com](https://appstoreconnect.apple.com) → your app
2. Click **+** next to "iOS App" in the sidebar → enter new version number
3. Add "What's New in This Version" text
4. Update screenshots only if the UI changed

## 5. Upload

Transporter: drag `Zwipe.ipa` → Deliver

Or CLI:
```bash
xcrun altool --upload-app -f ~/Developer/zwipe/Zwipe.ipa -t ios \
  --apiKey <KEY_ID> --apiIssuer <ISSUER_ID>
```

## 6. Submit

1. Select the new build (appears after 5–10 min)
2. Export Compliance → No
3. Submit for Review

---

## Version bumping

If Apple requires a higher version number, update `CFBundleShortVersionString` in the
generated `Info.plist` or configure it in `Dioxus.toml` before building.
