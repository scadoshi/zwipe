# Android crash — ndk-context double-init

**Status: FIXED AND VERIFIED ON DEVICE 2026-08-17.** Ships in 1.9.2.
Was mis-titled a "resume crash" for five versions. Resume is not involved.

```
panicked at ndk-context-0.1.1/src/lib.rs:87:5:
assertion failed: previous.is_none()
```

## Root cause

`MainActivity` declares **no `launchMode`**, so it defaults to `standard`. Any
explicit start of the component while an instance already exists creates a
**second Activity in the live process**. The app is a `NativeActivity`
(`android.app.func_name=ANativeActivity_onCreate`), so that second creation
re-runs wry/tao's native init, which calls
`ndk_context::initialize_android_context` — and that asserts the context was
not already set.

The first Activity is never destroyed in this path. It sits on the back stack
while the second one blows up beside it.

## Why the 1.7.6 fix could never have worked

`zcripts/android/back_handler.sh` kills the process in `MainActivity.onDestroy`.
In the failing path **nothing is destroyed**, so `onDestroy` never runs and the
kill never fires. The fix was aimed at a lifecycle event the bug doesn't touch.

It shipped untested on device and bought five versions of false confidence
(1.7.6 → 1.9.1, 92 crashes).

## Evidence (Pixel 6, store build 1.9.1 vc37, 2026-08-17)

Same binary, same device, one variable changed:

| Action | Result |
|---|---|
| Cold start via launcher intent | clean |
| HOME, then relaunch via launcher | clean, **same pid**, no destroy |
| `am start -n …/MainActivity` (2nd instance) | **PANIC**, pid dies |
| `am start -n … --activity-single-top` (reuse) | **clean**, same pid |

The last two lines are the whole proof: identical launch, one flag apart. Reuse
the instance and it's fine; create a second and it crashes.

Logcat at the moment of failure — note the panicking pid is the one that was
already running:

```
18108 I RustStdoutStderr: thread '<unnamed>' (18108) panicked at ndk-context-0.1.1/src/lib.rs:87:5:
18108 I RustStdoutStderr: assertion failed: previous.is_none()
 1689 I ActivityManager: Process com.scadoshi.zwipe (pid 18108) has died: fg TOP
```

## Real-world triggers

Anything that starts the component explicitly rather than resuming the task:
notification taps, another app opening Zwipe, app shortcuts, and the **Play
Store's "Open" button after an update** — which would explain crash clusters
right after each release.

## Second, separate bug found the same session: the app vanishes on theme change

The generated manifest carries
`configChanges="orientation|screenLayout|screenSize|keyboardHidden"` —
**`uiMode` is missing**. So a system dark/light switch recreates the Activity,
and *there* `onDestroy` does fire, so the 1.7.6 process kill executes and the
app silently disappears. Verified on device: toggling night mode left no
process, and no crash report — it looks like the user closed the app.

Auto dark mode at sunset does this on a schedule. `locale`, `density` and
`fontScale` are missing too and would behave the same way.

## The fix

Both live in the generated `AndroidManifest.xml`:

1. `android:launchMode="singleTask"` on `MainActivity` — an existing instance
   is reused via `onNewIntent` instead of a duplicate being created. This is
   the standard arrangement for `NativeActivity` apps precisely because their
   native init is not re-entrant.
2. Add `uiMode` to `configChanges` (and `locale|density|fontScale` while
   there) so a theme change doesn't tear the Activity down.

`dx` regenerates the manifest on every `dx bundle`, exactly like
`MainActivity.kt` and `res/`. That makes this the **fourth** manual
post-bundle patch step, and the reason this bug survived so long is a build
process that depends on remembering steps. Fold them into one script the build
calls once.

## Open question: is the process kill still wanted?

With `singleTask` in place the double-init path is gone, and with `uiMode`
handled the theme-change teardown is gone. The `onDestroy` kill was a
workaround for a crash it never actually prevented, and it has a real cost:
any genuine Activity teardown takes the whole process, so the user loses their
place. Worth removing once the manifest fix is proven — but change one thing at
a time, and prove the crash is gone first.

## Verification

Device loop (seconds per iteration, no release cycle):

```bash
# harness: scratchpad/crash_repro.sh
adb shell am force-stop com.scadoshi.zwipe
adb shell monkey -p com.scadoshi.zwipe -c android.intent.category.LAUNCHER 1
adb shell am start -n com.scadoshi.zwipe/dev.dioxus.main.MainActivity   # must NOT panic
adb logcat -d | grep ndk-context                                        # must be empty
```

Field exit criteria: zero ndk-context crashes for 7 days at comparable Android
session volume.

```sql
SELECT client_version, date(occurred_at) AS day, count(*) AS crashes
FROM crash_reports WHERE message LIKE '%ndk-context%'
GROUP BY 1,2 ORDER BY 2 DESC LIMIT 14;
```

## Note for whoever reads crash rows

The `/Users/<name>/.cargo/...` panic prefix is a **compile-time** path shipped
inside every binary, not a device fingerprint. `scottyrayfermo` on 1.7.4/1.7.5
vs `scottyfermo` on 1.7.6+ is a build-machine changeover. Misreading it as
"these are the owner's own devices" is what made this look benign.


## Verification (Pixel 6, patched build, 2026-08-17)

Same device, same binary, only the manifest changed. `launchMode=2`
(singleTask) and `configChanges=0x400017a4` (now including `uiMode`) confirmed
present in the built APK before testing.

| Test | Before | After |
|---|---|---|
| `am start -n …/MainActivity` while running | **panic**, pid died | clean, **same pid** |
| Same, five times in a row | — | clean, same pid |
| System dark mode on | app **vanished** | **survives**, same pid |
| System dark mode off | app **vanished** | **survives**, same pid |
| Total `ndk-context` / panic lines across all of it | many | **0** |

One process (pid 19347) survived every case that previously killed the app.

The fix is a build-time manifest patch (`zcripts/android/manifest.sh`, run by
`zcripts/android/patch_bundle.sh`), so it lands on the next Android release
build. Nothing in Rust changed.

## Follow-ups

- **Field confirmation:** watch `crash_reports` for 7 days after 1.9.2 reaches
  users, against Android session volume (queries above). Do not call it done on
  the strength of the lab result alone — that is the 1.7.6 mistake in reverse.
- **Revisit the `onDestroy` process kill.** It was a workaround for a crash it
  never prevented, and it costs the user their place whenever a genuine
  teardown happens. With `uiMode` handled it fires far less often. Remove it
  only after 1.9.2 proves out, and change one thing at a time.
- **Re-test the iOS "unresponsive after long backgrounding" bug** against this
  build. Different platform and no shared cause established, but both are
  webview-lifecycle failures and it is worth one look.
