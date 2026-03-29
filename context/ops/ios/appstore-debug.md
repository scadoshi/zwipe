# App Store Submission Debug — "Beta Xcode" Rejection

Tracking what we know, what we don't, and tests to run.

---

## The Error

```
Unable to Add for Review
New apps and app updates must be built with the latest public (GM) versions of Xcode,
and the iOS, macOS, watchOS, and tvOS SDKs. Apps built with beta versions aren't allowed.
```

This error appears on the **Distribution** tab in App Store Connect.
The same builds show **"Ready to Submit"** on the **TestFlight** tab with no warnings.

---

## What We Know

### Environment (verified 2026-03-29)
- macOS Tahoe 26.4 (Build 25E246) — latest public release (released Mar 24)
- Xcode 26.4 (Build 17E192) — installed from Mac App Store (confirmed GM)
- iOS SDK 26.4 (iphoneos26.4) — ships with Xcode 26.4
- Rust stable toolchain (aarch64-apple-darwin)
- Dioxus 0.7.4 — `dx build --release --platform ios`

### Binary metadata (build 6)
- `LC_BUILD_VERSION platform IOS, minos 16.0, sdk 26.4, tool LD 1266.8`
- No `LC_VERSION_MIN_IPHONEOS` (replaced via vtool)
- Linked libs: libobjc, WebKit, UIKit, Foundation, CoreFoundation, Security, libiconv, libSystem

### Info.plist (build 6)
- `DTPlatformName: iphoneos`
- `DTPlatformVersion: 26.4`
- `DTSDKName: iphoneos26.4`
- `DTXcode: 2640`
- `DTXcodeBuild: 17E192`
- `DTCompiler: com.apple.compilers.llvm.clang.1_0`
- `BuildMachineOSBuild: 25E246`
- `MinimumOSVersion: 16.0`
- `CFBundlePackageType: APPL`
- `CFBundleSupportedPlatforms: [iPhoneOS]`
- `UIDeviceFamily: [1]` (iPhone only)

### What we've tried (all failed)
1. Updated macOS from 26.3.1 to 26.4
2. Set DT* values to match real Xcode (DTXcode 2640, DTXcodeBuild 17E192)
3. Added BuildMachineOSBuild (25E246)
4. Replaced LC_VERSION_MIN_IPHONEOS with LC_BUILD_VERSION via vtool
5. Set correct linker version (ld 1266.8)
6. Created new app version (1.0.1) to clear cached state
7. Uploaded 7 different builds (all rejected with same error in App Store Connect UI)
8. `xcrun altool --validate-app` passes with ZERO errors on build 7
9. `xcrun altool --upload-app` succeeds with ZERO errors on build 7
10. Created new app version 1.0.1 — same error
11. Binary metadata is identical to a native Xcode-compiled binary (confirmed via Test 1)

### Conclusion
**The binary is valid.** Apple's own validation tool confirms it. The "beta Xcode" error
is stuck on the app record in App Store Connect, likely because the app was first created
and uploaded from macOS 26.3.1 before Xcode 26.4 went GM on Mar 24.

**Next step:** Delete the app record entirely and recreate it fresh (Test 5).

### Key observation
**TestFlight accepts all builds as "Ready to Submit"** but Distribution rejects them.
This suggests the error may not be about the binary at all.

### Test 1 result: Binary comparison (completed 2026-03-29)
Built a minimal Swift binary with `xcrun -sdk iphoneos swiftc` and compared Mach-O metadata.
**The Zwipe binary is IDENTICAL to a native Xcode-compiled binary:**
- Both: `LC_BUILD_VERSION platform IOS, minos 16.0, sdk 26.4, tool LD 1266.8`
- Same framework versions (UIKit 9126.4.27, Foundation 4424.1.101, libSystem 1356.0.0)
- No `LC_VERSION_MIN_IPHONEOS` in either

**Conclusion: The binary is NOT the problem.** The rejection is happening at a layer above the binary.

### Test 4 result: Cert/profile inspection (completed 2026-03-29)
- Provisioning profile: Created 2026-03-29, expires 2027-03-29, platform [iOS, xrOS, visionOS], no beta flags
- Distribution cert: Valid 2026-03-29 to 2027-03-29, issued by Apple WWDR CA G3
- `beta-reports-active: true` is normal (TestFlight crash reporting, not a beta indicator)

**Conclusion: Signing infrastructure is clean.**

### Remaining theories
1. **App record flagged at creation time** — first upload was from macOS 26.3.1 (before 26.4 released Mar 24). Apple may have flagged the app record itself.
2. **Known Apple bug with Xcode 26.4** — it released Mar 24, just 5 days ago. May be a validation bug on Apple's end.
3. **Non-Xcode toolchain detection** — Apple may have a check beyond Mach-O metadata that detects the binary wasn't produced by Xcode's build system.

---

## What We Don't Know

1. **Is the error actually about the binary?** TestFlight accepts it — maybe Distribution has a different/additional check, or maybe it's an account/app-level flag.

2. **Is there a hidden flag on the app record?** The app was first created while on macOS 26.3.1 (pre-update). Could Apple have flagged the app itself?

3. **What exactly triggers this error?** Is it a plist value, a Mach-O load command, a linked library version, or something else entirely?

4. **Would a native Xcode-built app also get rejected?** If yes, the issue is account/environment. If no, the issue is Rust/Dioxus-specific.

5. **Does the provisioning profile or certificate carry beta metadata?** They were created on macOS 26.3.1.

6. **Is this error actually a blocker?** Some App Store Connect errors are warnings that don't prevent submission — but this one explicitly blocks "Add for Review."

---

## Tests To Run

### Test 1: Build a minimal Xcode project and submit
**Purpose:** Determine if the issue is Rust/Dioxus-specific or account/environment-wide.
```
1. Open Xcode → New Project → iOS App → "TestApp"
2. Bundle ID: com.scadoshi.testapp (register on developer.apple.com first)
3. Product → Archive → Distribute → App Store Connect
4. See if it also gets the beta Xcode error
```
**If it fails:** Issue is account/cert/environment, not Rust.
**If it succeeds:** Issue is something specific to our Rust-built binary.

### Test 2: Compare Mach-O metadata between Xcode binary and our binary
**Purpose:** Find what's different.
```bash
# Build the TestApp from Test 1, then:
otool -l TestApp > /tmp/xcode-binary.txt
otool -l ~/Developer/zwipe/target/dx/zwipe/release/ios/Zwipe.app/zwipe > /tmp/rust-binary.txt
diff /tmp/xcode-binary.txt /tmp/rust-binary.txt
```

### Test 3: Compare Info.plist between Xcode archive and our bundle
**Purpose:** Find missing or incorrect plist keys.
```bash
# After archiving TestApp in Xcode:
# Find the .app in ~/Library/Developer/Xcode/Archives/
plutil -p /path/to/TestApp.app/Info.plist > /tmp/xcode-plist.txt
plutil -p ~/Developer/zwipe/target/dx/zwipe/release/ios/Zwipe.app/Info.plist > /tmp/rust-plist.txt
diff /tmp/xcode-plist.txt /tmp/rust-plist.txt
```

### Test 4: Check if certs/profiles carry beta metadata
**Purpose:** Rule out signing infrastructure.
```bash
# Inspect the provisioning profile
security cms -D -i ~/certs/Zwipe_App_Store.mobileprovision
# Look for any beta/development flags or expiry issues

# Inspect the distribution cert
openssl x509 -in ~/certs/distribution.cer -inform DER -text -noout
# Check validity dates and issuer
```

### Test 5: Delete app entirely and recreate
**Purpose:** Rule out a stuck app-level flag.
```
1. App Store Connect → delete "Zwipe MTG" entirely
2. Create a new app record with the same bundle ID
3. Upload build 6
4. Try Add for Review
```
**Warning:** May lose the app name reservation.

### Test 6: Use xcrun altool for detailed validation
**Purpose:** Get more specific error messages than App Store Connect shows.
```
1. App Store Connect → Users and Access → Integrations → App Store Connect API
2. Create API key (Admin access)
3. Download .p8 file → ~/.private_keys/AuthKey_<KEY_ID>.p8
4. Run:
   xcrun altool --validate-app -f ~/Developer/zwipe/Zwipe.ipa -t ios \
     --apiKey <KEY_ID> --apiIssuer <ISSUER_ID>
```

### Test 7: Check Apple Developer Forums / contact support
**Purpose:** See if others have this issue with Xcode 26.4 right after GM release.
```
Search queries:
- "built with beta versions" Xcode 26.4 site:developer.apple.com/forums
- "Unable to Add for Review" "beta" 2026 site:developer.apple.com/forums
- dioxus app store submission
- rust ios app store rejection beta xcode
```

---

## Recommended Order

1. **Test 4** (quick — just inspect cert/profile, 2 min)
2. **Test 7** (quick — search forums, 5 min)
3. **Test 6** (medium — need API key, 10 min)
4. **Test 1 + 2 + 3** (medium — build Xcode project, 20 min)
5. **Test 5** (last resort — destructive)
