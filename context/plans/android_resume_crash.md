# Android resume crash — ndk-context double-init

**Status: OPEN, live in production, top crash by a wide margin.**
Not a pending verification — the 1.7.6 fix did not work.

```
panicked at ndk-context-0.1.1/src/lib.rs:87:5:
assertion failed: previous.is_none()
```

## What we know (verified on prod 2026-08-17)

**It is essentially the only crash we have.** 92 of 94 recorded crashes are this
assert, spanning 6 client versions. The other two are singletons: one iOS panic
in `profile/mod.rs:317`, one Android `tracing-subscriber` double-init (itself a
second symptom of the same "init ran twice" story).

**It is hitting real users, not the build machine.** 12 distinct Android users
had 1.9.1 sessions between 08-14 and 08-17.

**The 1.7.6 fix did not stop it.** Crashes per version/day:

| Version | Crashes | Days |
|---|---|---|
| 1.7.4 | 11 | 08-02 → 08-05 |
| 1.7.5 | 36 | 08-06 → 08-11 |
| **1.7.6 (the fix)** | **13** | 08-12 → 08-13 |
| 1.8.1 | 8 | 08-14 |
| 1.9.0 | 4 | 08-15 |
| **1.9.1** | **20** | 08-16 → 08-17 |

**Do NOT read the panic path as a device fingerprint.** The
`/Users/<name>/.cargo/...` prefix is a *compile-time* path baked into the
binary and shipped to every user. The 1.7.4/1.7.5 rows say `scottyrayfermo`
and 1.7.6+ say `scottyfermo` because the build machine changed, not because
different people crashed. (This was misread once already, which made the bug
look self-inflicted and low priority. It is neither.)

## The mechanism

`ndk_context::initialize_android_context` asserts that no context was
previously set. The assert fires when wry/tao's native init runs a second time
in one process — i.e. the Activity is created again without the process having
died.

zwiper itself is not the second initializer: it only ever *reads* the context
(`android_context()` in `back_handler.rs:174`, `android_fs.rs:12`,
`open_url.rs:77`). The double init comes from the webview stack.

## What was tried (and why it was believed to work)

`zcripts/android/back_handler.sh` patches `MainActivity.onDestroy` to call
`android.os.Process.killProcess(myPid())`, so an Activity teardown takes the
process with it and every reopen is a cold start. Shipped in 1.7.6,
**untested on device by owner call**. The data above is the first real read of
whether it worked. It didn't.

## Current dependency state (matters — the premise may have expired)

| Crate | Pinned |
|---|---|
| dioxus | 0.7.10 |
| tao | **0.34.8** |
| wry | 0.53.5 |
| ndk-context | 0.1.1 |

The workaround's own comment says "tao pins below 0.34.4 never release the
context." We are on **0.34.8**, above that line. So either tao's release path
exists now and isn't being reached, or it doesn't do what the comment assumed.
This is the first thing to check, because if upstream now releases the context
properly, the fix is a dependency question, not a Kotlin patch.

## Hypotheses, ranked, each with the cheapest test

**H1 — the patch isn't in the shipped build.** `dx` regenerates
`MainActivity.kt` on every `dx bundle`, so `back_handler.sh` must be re-run
manually between bundle and the Gradle repackage, every single release. Miss it
once and the release ships with no process kill *and* no back-gesture handling.
- *Test (free):* back-swipe on a live 1.9.1 Android install. If OS back exits
  the app instead of navigating, the patch is missing and H1 is confirmed.
- *Test (definitive):* unzip the shipped AAB and grep the dex for
  `killProcess`.
- *If true:* the real bug is a build process that depends on remembering a
  manual step. Fix by folding the script into the bundle step, not by writing
  more Kotlin.

**H2 — `onDestroy` runs but the process survives.** The kill may race the
recreation, or be skipped on some teardown paths.
- *Test:* device with Developer Options → **"Don't keep activities"** on (this
  forces exactly the destroy-Activity-keep-process case), then background and
  resume while watching `adb logcat`. Look for the ordering of `onDestroy`, the
  kill, and the next `onCreate`.

**H3 — a second Activity instance is created without a destroy.** If something
starts `MainActivity` again in the same task (deep link, share target,
notification, launcher relaunch under certain launch modes), `onCreate` runs
while the first instance is alive. Nothing was destroyed, so the kill never
fires, and native init runs twice.
- *Test:* check `launchMode` / `taskAffinity` in the generated
  `AndroidManifest.xml`; try entering via a `zwipe.net` deep link and via the
  launcher while the app is already open.

**H4 — the assert is reached before our Kotlin runs at all.** If wry's native
init happens during `WryActivity.onCreate`'s `super` call, our override may be
too late to matter for the failing path.
- *Test:* read the `WryActivity` source that dx generates alongside
  `MainActivity.kt` and confirm where native init is invoked.

## Instrumentation worth adding (needs an app train)

`crash_reports` currently holds crash_id, timestamps, client_version, platform,
message — nothing about the device or what the app was doing. To pick between
these hypotheses from field data rather than a lab repro, the useful additions
are Android SDK level and a cold-start-vs-resume marker. Only worth doing if
the lab repro below fails to reproduce.

## Repro procedure (do this before writing any fix)

1. Install a release-ish Android build on a device.
2. Developer Options → **Don't keep activities** = on.
3. Open the app, background it, open several other apps, return.
4. `adb logcat | grep -E "ndk-context|onDestroy|onCreate|zwipe"`.

If this reproduces, we have a loop that costs seconds per iteration instead of
a release cycle per iteration. **That is the whole point** — 1.7.6 shipped
blind and bought five versions of false confidence.

## Exit criteria

Zero ndk-context crashes for 7 consecutive days at comparable Android session
volume. Not "we shipped a fix."

Watch query (prod, via `ssh zerver`):

```sql
SELECT client_version, date(occurred_at) AS day, count(*) AS crashes
FROM crash_reports
WHERE message LIKE '%ndk-context%'
GROUP BY 1,2 ORDER BY 2 DESC LIMIT 14;
```

Session volume for the same window, so a drop in crashes isn't just a drop in
users:

```sql
SELECT client_version, platform, count(DISTINCT user_id) AS users
FROM refresh_tokens
WHERE created_at > now() - interval '7 days' AND platform = 'android'
GROUP BY 1,2 ORDER BY 3 DESC;
```

## Related

The **"app unresponsive after long backgrounding"** bug (iOS, owner report
2026-07-30, in `progress/todo.md`) is plausibly the same root cause wearing a
different coat: both are resume-path failures where the webview/native bridge
doesn't survive the OS reclaiming things. One panics, one wedges. Worth
re-testing that one against whatever fixes this.
