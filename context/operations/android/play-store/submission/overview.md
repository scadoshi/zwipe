# Android Play Store submission

How to produce a **signed release `.aab`** and get it into the Play Console. The
Android analogue of iOS
[`app-store/submission/`](../../../ios/app-store/submission/overview.md).

| Part | What it covers |
|------|----------------|
| [build.md](build.md) | Gotchas + prerequisites + build the signed AAB (dx bundle → icons → back_handler → gradle patch → repackage → sign → smoke-test). **The core runbook.** |
| [publish.md](publish.md) | Upload to the Play Console, roll out to closed testing, recruit testers; native-debug-symbols note. |
| [form_fields.md](form_fields.md) | Store listing copy. |
| [history.md](history.md) | Per-release build log. |

Dev-env setup (JDK 21 gotcha, emulator) lives in [../../setup.md](../../setup.md)
and [../../emulator.md](../../emulator.md).
