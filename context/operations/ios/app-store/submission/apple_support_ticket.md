# Apple Developer Support — Detailed Report

## RESOLVED 2026-05-22 — now we wait!

Apple's submission allowlist required the very latest Xcode (26.5, released 2026-05-11). Upgrading to Xcode 26.5 build 17F42, rebuilding clean (wiping the cargo iOS device cache to force re-link against SDK 23F77), and patching the Info.plist DT keys to match cleared the "Add for Review" check. Build 13 is in "Waiting for Review" and submitted to App Review. Cases 102856406657 (Xue) and 102855955579 (Liping) both closed. See `debugging.md` for full resolution notes.

The original support report is preserved below for reference.

---

**Subject:** "Unable to Add for Review" — all builds rejected with "beta Xcode" error despite using GM toolchain

**Developer:** Scotty Fermo
**Team ID:** VV74WQ89GD
**Apple ID (app):** 6761341603 (Zwipe MTG, com.scadoshi.zwipe)
**Second test app:** Zwipe Test (com.scadoshi.zwipetest)

---

## Problem

Every build I upload is blocked by the following error when clicking "Add for Review" in App Store Connect:

> "New apps and app updates must be built with the latest public (GM) versions of Xcode, and the iOS, macOS, watchOS, and tvOS SDKs. Apps built with beta versions aren't allowed."

This error appears on both the Distribution tab and when attempting TestFlight external beta review. The same builds show "Ready to Submit" in the TestFlight builds list with no warnings.

---

## Environment

All software is the latest public GM release, installed through official channels:

- **macOS:** Tahoe 26.4 (Build 25E246) — updated via Software Update
- **Xcode 26.4:** Build 17E192 — installed from Mac App Store (verified: `kMDItemAppStoreHasReceipt = 1`)
- **Xcode 26.3:** Build 17C529 — also tested, downloaded from developer.apple.com
- **iOS SDK:** 26.4 (ships with Xcode 26.4) and 26.2 (ships with Xcode 26.3)
- **Distribution certificate:** Apple Distribution, created 2026-03-29, expires 2027-03-29, issued by Apple WWDR CA G3
- **Provisioning profile:** App Store type, created 2026-03-29, expires 2027-03-29, no beta flags

---

## What I have tried

I uploaded 9 builds across 2 different apps and 2 Xcode versions. Every single one is rejected with the same error.

### Builds 1–8: Zwipe MTG (com.scadoshi.zwipe)

- Builds 1–2: Built with Xcode 26.4 on macOS 26.3.1 (before updating to 26.4)
- Builds 3–8: Built with Xcode 26.4 on macOS 26.4 (after updating)
- Build 9: Built with Xcode 26.3 (17C529) / iOS SDK 26.2 on macOS 26.4

All rejected with the same error.

### Fresh app test: Zwipe Test (com.scadoshi.zwipetest)

To rule out a cached flag on the original app record, I:
1. Registered a new bundle ID (com.scadoshi.zwipetest)
2. Created a new App Store provisioning profile for it
3. Created a completely new app in App Store Connect
4. Uploaded a build

Same error on the fresh app.

### Native Swift binary test

To rule out my build toolchain (I use Rust/Dioxus, not Xcode's standard build system), I:
1. Compiled a minimal Swift iOS app using Apple's own `xcrun -sdk iphoneos swiftc`
2. Packaged it identically to a standard IPA
3. Uploaded it to the Zwipe Test app

**A native Swift binary built by Apple's own compiler gets the same error.** This proves the issue is not related to my build toolchain.

### altool validation

Every build I upload passes Apple's own validation tool with zero errors:

```
$ xcrun altool --validate-app -f Zwipe.ipa -t ios --apiKey <KEY> --apiIssuer <ISSUER>
VERIFY SUCCEEDED with no errors

$ xcrun altool --upload-app -f Zwipe.ipa -t ios --apiKey <KEY> --apiIssuer <ISSUER>
UPLOAD SUCCEEDED with no errors
```

The binary passes server-side validation. Only the "Add for Review" button in the App Store Connect web interface rejects.

### Binary metadata verification

I verified that my binary's metadata matches what Xcode natively produces:

```
LC_BUILD_VERSION:
  platform: IOS
  minos: 16.0
  sdk: 26.4
  tool: LD version 1266.8

Info.plist:
  DTPlatformName: iphoneos
  DTPlatformVersion: 26.4
  DTSDKName: iphoneos26.4
  DTXcode: 2640
  DTXcodeBuild: 17E192
  BuildMachineOSBuild: 25E246
  MinimumOSVersion: 16.0
  CFBundlePackageType: APPL
```

### Version and app record changes

- Created new app version 1.0.1 to clear any cached validation state — same error
- Reverted back to version 1.0 — same error
- Created entirely new app with different bundle ID — same error

---

## Summary of evidence

| Test | Result |
|------|--------|
| Xcode 26.4 (17E192) GM from Mac App Store | Rejected |
| Xcode 26.3 (17C529) from developer.apple.com | Rejected |
| macOS 26.4 (25E246) latest public | Rejected |
| Native Swift binary (not third-party toolchain) | Rejected |
| Fresh app with different bundle ID | Rejected |
| `altool --validate-app` | Passes with zero errors |
| `altool --upload-app` | Succeeds with zero errors |
| TestFlight build status | "Ready to Submit" (no warnings) |
| App Store Connect "Add for Review" | Blocked |
| 9 different builds tested | All rejected with same error |

---

## Request

Something at the account level appears to be blocking submission with a misleading "beta Xcode" error. Could you please:

1. Check if there is a flag or restriction on my account (Team ID VV74WQ89GD) that is causing this rejection
2. Verify whether there are any pending agreements or compliance requirements I may have missed
3. Confirm whether Xcode 26.4 (17E192) is whitelisted in your App Store submission validation system

I have exhausted every technical avenue on my end. The binary is valid, the toolchain is GM, and Apple's own validation tools accept the build. Only the App Store Connect web interface rejects it.

Thank you for your help.
