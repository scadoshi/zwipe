# Building from a second Mac

How to set up an **additional** machine that can produce App Store builds, while
the current one keeps working. This is not a migration: see
[mac_restore.md](mac_restore.md) for wiping and replacing a machine.

The short version: nothing here is device-bound, so a second Mac costs the first
one nothing. The two ways to break that are covered under
[What would invalidate the other Mac](#what-would-invalidate-the-other-mac).

---

## What to copy over

Copy, don't regenerate. Everything below is happy to exist on both machines at
once.

| From | To | Notes |
|------|----|-------|
| `~/certs/` (whole directory) | same path | `zwipe-dist-key.pem` + `distribution.cer` + `Zwipe_App_Store.mobileprovision`, plus the Android `zwipe-upload.jks` |
| `~/.private_keys/AuthKey_<KEY_ID>.p8` | same path | Only needed for the `altool` fallback; copy it anyway |

Move them over something private (AirDrop, USB, an encrypted disk image). The
`.pem` and `.jks` are unrecoverable private keys, so no email, no Slack, no
public cloud folder.

Everything else the build needs is already in git: `Entitlements-Release.plist`,
`Entitlements.plist`, and the nine icons under `zwiper/assets/favicon/`.

## Install on the new machine

| What | Why it's version-sensitive |
|------|---------------------------|
| **Xcode, latest from the Mac App Store** | Apple's submission allowlist rejects binaries linked against anything older, with a misleading "beta Xcode" message. See [debugging.md](app-store/submission/debugging.md) |
| `xcodebuild -downloadPlatform iOS` | Without the iOS platform installed, `actool` can't produce `Assets.car` and the icon step fails |
| **Transporter** (Mac App Store, free) | The sanctioned upload path in [publish.md](app-store/submission/publish.md) |
| Rust + `rustup target add aarch64-apple-ios aarch64-apple-ios-sim` | Both targets; the sim target is needed for screenshots |
| **`dx` pinned to the same version as the other Mac** | The Info.plist patch list in [build.md](app-store/submission/build.md) is written against what one specific `dx` release emits. Install the exact version, not `@latest`, or the patch steps drift |
| `zwiper/.env` with `BACKEND_URL=https://api.zwipe.net` | Gitignored. `build.md` passes `BACKEND_URL` inline, so this is only for `dx serve` |

Check `xcodebuild -version` and `dx --version` on both machines and keep them
matched. A build made on a machine running an older Xcode gets rejected at "Add
for Review", not at build time, so the mismatch surfaces late.

## Import the signing identity

Copying `~/certs/` is not enough on its own — `codesign` reads the keychain, not
that directory:

```bash
security import ~/certs/zwipe-dist-key.pem \
  -k ~/Library/Keychains/login.keychain-db -T /usr/bin/codesign
open ~/certs/distribution.cer   # pairs the cert with the imported key
```

Confirm before trusting the machine with a release:

```bash
security find-identity -v -p codesigning
# must list: "Apple Distribution: SCOTTY RAY FERMO (VV74WQ89GD)"
```

If the Apple Distribution line is missing, the key didn't import — fix that
before building, rather than discovering it at the `codesign` step.

---

## What would invalidate the other Mac

Two things, both avoidable:

**1. Creating a new distribution certificate.** Apple caps how many Apple
Distribution certs an account may hold. Hit the cap, and the UI offers to revoke
an existing one to make room — revoking is what kills the other Mac, because
every provisioning profile built on that cert dies with it. So on the new
machine, never click "Create Certificate" on developer.apple.com and never let
Xcode's *Automatically manage signing* fix a signing error for you. There is no
reason to: the identity you copied is the identity you want. Zwipe's release
build is plain CLI `codesign` with no Xcode project, so automatic signing should
never enter the picture unless you go looking for it.

**2. Reusing a build number.** `CFBundleVersion` must be unique per upload
forever, and two machines make it easy to build 70 twice. Read the last shipped
number off [history.md](app-store/submission/history.md) before every build, and
log the new one there in the same commit as the release. That file is the
coordination point between the machines; a stale checkout on one of them is the
failure mode to watch for, so `git pull` first.

Nothing else is at risk:

- **The Mac itself is never registered with Apple.** Only test *iPhones* get
  registered, and only for development profiles. Adding a Mac is not an event
  Apple knows about.
- **The App Store provisioning profile contains no device UDIDs** — that's what
  distinguishes an App Store profile from a development one. Copying it changes
  nothing about where it's valid.
- **The API key and Transporter's Apple ID sign-in** both work from any number of
  machines simultaneously.
- **The cert and profile expire 2027-03-29**, on the same day, regardless of how
  many machines hold copies. Renewing then is one job, not two: renew once, copy
  the new pair to both.

## No iPhone pairing needed

`build.md`'s build command ends with `--device "scotland-mobile"`. That reads like
a requirement, but `dx build` doesn't upload anything, so the flag only steers the
target to `aarch64-apple-ios` instead of the simulator — the named device does not
have to be present, or even known to that Mac. Release builds on the current
machine routinely run with the phone unplugged.

So the second Mac needs no device pairing and no edit to the command. Leave the
flag as written and the two machines stay on one identical runbook.
