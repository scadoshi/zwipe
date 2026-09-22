# Suppress the iOS shake-to-undo prompt

**Status: DONE 2026-09-22 (`90a85d5c`), owner-tested on device. Rides 1.10.2.**

Landed as planned: `zwiper/src/lib/outbound/shake_to_edit.rs`, called from a `use_hook` in `App()`. Two verification notes worth keeping, since neither is obvious:

- A plain `cargo check` compiles only the `not(target_os = "ios")` no-op, so it proves nothing. Check `--target aarch64-apple-ios --features mobile`.
- That check is itself worth falsifying. Breaking the selector on purpose and confirming the build fails is how you know the cfg branch is being compiled rather than skipped.

**One sentence:** set `applicationSupportsShakeToEdit = false` on the shared `UIApplication` so shaking the phone stops raising the system "Undo Typing" alert.

## Why

iOS turns a shake into an undo prompt for text fields. Zwipe has its own undo (swipe down on the add-card stack), so the system alert is both an interruption and a second, conflicting meaning for "undo" in an app built around gestures. Nothing in Zwipe wants the system behavior: the only text entry is short form fields where undo has no real use.

## The change

One `#[cfg(target_os = "ios")]` function. `objc2` 0.6 is already a direct dependency and `zwiper/src/lib/outbound/open_url.rs` has the pattern to copy: take a `MainThreadMarker`, `msg_send![class!(UIApplication), sharedApplication]`, then send the setter.

```rust
// [UIApplication.sharedApplication setApplicationSupportsShakeToEdit:NO]
let _: () = unsafe { msg_send![&*app, setApplicationSupportsShakeToEdit: false] };
```

Fail soft: if the marker or the app object is missing, return without doing anything. A missing shake suppression is not worth a panic.

## Where to call it

In a `use_hook` in `App()` (`zwiper/src/bin/zwipe.rs`), not in `main()`. `sharedApplication` is nil until `launch_app()` has built the app object, so calling it beside `install_panic_hook()` would silently no-op. The root component's first render is on the main thread and after the app exists, which is what the call needs.

## Verification

The Simulator can do this: **Device → Shake**, no physical device required.

- Before: shaking on a screen with a focused text field raises "Undo Typing".
- After: nothing happens, and the app's own swipe-down undo still works on the add-card stack.
- `cargo check --target aarch64-apple-ios -p zwiper` for the cfg'd code, since a normal workspace check never compiles it.

## Notes

- Global and one-way for the process; there is no screen that wants the system behavior back.
- Client-only, so it burns a client build: it rides 1.10.2 with whatever else is queued rather than justifying a release by itself.
