# iOS Troubleshooting

Common errors and fixes when building, signing, or submitting Zwipe.

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

## Missing Info.plist keys

If Apple rejects for missing metadata, add these to the generated `Info.plist`:

```xml
<key>DTPlatformName</key>
<string>iphoneos</string>
<key>MinimumOSVersion</key>
<string>16.0</string>
<key>CFBundlePackageType</key>
<string>APPL</string>
```

Patch after build:
```bash
/usr/libexec/PlistBuddy -c "Add :DTPlatformName string iphoneos" \
  ~/Developer/zwipe/target/dx/zwipe/release/ios/Zwipe.app/Info.plist
```

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

## "Untrusted Developer" on first launch

Settings → VPN & Device Management → your Apple ID → Trust

---

## Developer Mode required (iOS 16+)

Settings → Privacy & Security → Developer Mode → toggle on → restart device
