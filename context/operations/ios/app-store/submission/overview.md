# iOS App Store submission

The parts of shipping a Zwipe build to the App Store, in order. Mirrors the
Android [`play-store/submission/`](../../../android/play-store/submission/overview.md)
layout.

| Part | What it covers |
|------|----------------|
| [build.md](build.md) | Build the release `.ipa` — dx build, Info.plist patches, icons, sign, package. **The recurring runbook.** |
| [publish.md](publish.md) | Upload via Transporter + submit for review. |
| [first_release.md](first_release.md) | One-time account / certificate / App-ID setup for the very first submission. |
| [form_fields.md](form_fields.md) | Store listing copy (name, subtitle, description, keywords, What's New). |
| [testflight.md](testflight.md) | Beta App Description + tester invite message. |
| [icon_update.md](icon_update.md) | Refreshing the app icon assets. |
| [debugging.md](debugging.md) | The "beta Xcode" / SDK-allowlist rejection investigation + fixes. |
| [history.md](history.md) | Per-release build log. |

Apple support correspondence: [apple_support_ticket.md](apple_support_ticket.md),
[apple_support_reply_2026_05_13.md](apple_support_reply_2026_05_13.md).
Screenshots for the listing live in [`screenshots/`](screenshots/).

Recurring update = [build.md](build.md) → [publish.md](publish.md). Dev-device
setup (not submission) stays at [`../../`](../../README.md).
