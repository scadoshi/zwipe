# Why tao is vendored

This is tao 0.37.0 exactly as published on crates.io, with one edit: `version` in `Cargo.toml` reads `0.34.9` instead of `0.37.0`. Nothing else differs from upstream.

## The problem

iOS 27 traps at launch on any app that has not adopted the UIScene lifecycle. The trap is in `_UIApplicationEvaluateRuntimeIssueForNoSceneLifecycleAdoption`, raised from `-[UIApplication workspace:didCreateScene:withTransitionContext:completion:]`, and it kills the process about 100ms in. 1.10.2 build 80 was rejected for it on 2026-09-23: Apple's review devices run iOS 27.0, ours run 26.x, and 26.x does not enforce.

Two things are needed to survive it, and neither works alone:

1. `UIApplicationSceneManifest` in `Info.plist`. UIKit decides adoption from that key alone, so a delegate registered at runtime is too late. It lives in `zwiper/Dioxus.toml` under `[ios.plist.UIApplicationSceneManifest]`.
2. A tao that actually serves windows from a `UISceneDelegate`. Adding the manifest to a tao that does not means UIKit connects a scene the app never attaches its `UIWindow` to, and the app draws nothing: no crash, just a black screen.

tao gained scene support in 0.35.0 (`src/platform_impl/ios/scene.rs`, `TaoSceneDelegate`) and fixed a premature release of the `UISceneConfiguration` in 0.37.0.

## Why the version is rewritten

`dioxus-desktop` 0.7.10 depends on `tao ^0.34.0`, so cargo will not accept a 0.35+ patch: it resolves the patch against that requirement and silently ignores anything outside it. Labeling the crate 0.34.9 is what lets the patch apply. `dioxus-desktop` 0.7.10 and `wry` 0.53.5 both compile against tao 0.37's API unchanged, so nothing else had to move.

## When to delete this

The moment a released `dioxus-desktop` depends on tao 0.35 or newer. At that point: drop `[patch.crates-io]` from the workspace `Cargo.toml`, delete this directory, and keep the `Info.plist` key, which is needed regardless of tao version. `cargo tree -i tao` shows which version is actually in the graph; a patch that does not apply is a warning, not an error, so check rather than assume.

## Verifying a change here

`cargo update -p tao --precise 0.34.9` after editing, or cargo keeps the locked registry copy and the patch appears to do nothing.
