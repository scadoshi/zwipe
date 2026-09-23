# iOS 27 requires the UIScene lifecycle

**Status: FIX IN THE TREE 2026-09-23, not yet resubmitted.**

Apple rejected 1.10.2 build 80 on 2026-09-23 under Guideline 2.1(a). The app crashed on launch on their review devices, an iPad Air 11-inch (M3) and an iPhone 17 Pro, both on 27.0. Submission ID `6c408160-5c4a-4ee7-bde4-3d282166fd73`.

## What the crash logs say

Both logs are the same crash: `EXC_BREAKPOINT (SIGTRAP)` on the main thread, 96ms into launch on the phone and 156ms on the iPad. Frame 0 is

```
UIKitCore  _UIApplicationEvaluateRuntimeIssueForNoSceneLifecycleAdoption_block_invoke
```

called from `-[UIApplication workspace:didCreateScene:withTransitionContext:completion:]` inside a `dispatch_once`. Frames 24 and below are ours: `tao`'s event loop, `dioxus_desktop::launch`, `zwipe::main`. Nothing of ours ran. UIKit killed the process as soon as it created the first scene and saw the app had not adopted the scene lifecycle.

This is not a panic and no crash reporter of ours could have caught it.

## Why our testing missed it

iOS 26 warns about this. iOS 27 traps. The owner's test device is on 26.6 and the iOS 27.0 simulator does not enforce it either, so the only machine that shows the failure is a physical device on 27.0, which is what Apple reviews on. A control build without the fix launches fine in the iOS 27.0 simulator, so the simulator cannot be used to confirm this one.

## The fix, both halves

Neither half works alone.

1. `UIApplicationSceneManifest` in `Info.plist`, set in `zwiper/Dioxus.toml` under `[ios.plist.UIApplicationSceneManifest]`. UIKit decides adoption from the bundle alone, so registering a delegate at runtime is too late.
2. A `tao` that serves its windows from a `UISceneDelegate`. Our `tao` 0.34.8 has no scene code at all. Adding the manifest on top of it would have traded the crash for a black screen: UIKit connects a scene, tao's `UIWindow` is never attached to it, and nothing is ever composited.

`tao` gained scene support in 0.35.0 and fixed a premature release of the `UISceneConfiguration` in 0.37.0. `dioxus-desktop` 0.7.10 pins `tao ^0.34.0` and 0.7.10 is the newest release, so there is no version of dioxus that can reach the fix.

So `tao` 0.37.0 is vendored at `vendor/tao`, byte-identical to the published crate except that `version` reads `0.34.9`, which is what lets `[patch.crates-io]` apply against dioxus's requirement. `dioxus-desktop` 0.7.10 and `wry` 0.53.5 compile against tao 0.37's API with no changes. The reasoning and the removal conditions are in [`../../vendor/tao/VENDOR.md`](../../vendor/tao/VENDOR.md).

`tao` 0.37 also pulls `dbus` on Linux, which is in its default features, so `libdbus-1-dev` was added to the apt line in all three workflows that run `cargo clippy --workspace`.

## What is actually verified

On the iOS 27.0 simulator, built from this tree: the app launches, stays up, and draws the login screen. `TaoSceneDelegate` is in the binary and the UIKit log shows a real `UIWindowScene` hosting the app. Since a window that is not attached to a scene is never composited in a scene-adopting app, drawing at all is the proof that the attachment works.

What is **not** verified is the trap itself, because nothing available here raises it. Confidence that the rejection is fixed rests on the crash log naming the exact check, and on the manifest plus a scene-serving tao being the documented fix.

The cheap way to close that gap before resubmitting is to update the test iPhone to iOS 27 and run a release build on it.

## Before resubmitting

- `CFBundleVersion` must increment to 81. Apple rejected build 80, and a rejected build number cannot be reused.
- The version stays 1.10.2 and the changelog entry stays as shipped. Nothing user-facing changed.
- The DT keys in `zwiper/Dioxus.toml` now read Xcode 27.0 (`DTXcode 2700`, `DTXcodeBuild 27A266a`, `DTSDKName iphoneos27.0`). The build runbook overwrites these anyway, but they no longer claim 26.4.
- Android is unaffected. 1.10.2 on Play needs nothing.
