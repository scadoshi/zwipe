# Deploy a Build to Phone

Build and install on a connected iPhone. **Debug** is the daily workflow; **release** is for verifying the store-parity build on-device.

**Prerequisites:** Device registered and dev profile installed (see [devices.md](devices.md)).

---

## Build and deploy

One script per profile, because the profile picks the output directory as well as the compiler flags:

```bash
zcripts/ios/deploy_debug.sh      # the daily one
zcripts/ios/deploy_release.sh    # optimized, what the store will run
```

Both build against `https://api.zwipe.net` unless `BACKEND_URL` is set in the environment, find the iPhone on the cable, and install to it by UDID. With two phones plugged in and no `--device`, they list both and stop rather than guessing: `--device matthew` picks one by any part of its name. A phone this Mac has paired before but is not plugged in does not count.

Both are thin wrappers over `zcripts/ios/deploy.sh`, which takes `--release` if you prefer one entry point.

No backup step, unlike cairn: every deck lives on the server, so a reinstall has nothing on the phone to lose.

### Doing it by hand

Paste as a **single line**. Multi-line `\` continuations get mangled on paste into zsh.

```bash
cd ~/Developer/zwipe/zwiper && BACKEND_URL=https://api.zwipe.net dx build --platform ios --device true && ios-deploy --bundle ~/Developer/zwipe/target/dx/zwipe/debug/ios/Zwipe.app
```

The bundle path has to match the profile: `--release` writes to `target/dx/zwipe/release/ios/`, debug to `target/dx/zwipe/debug/ios/`. Build one and install the other and the build succeeds, the install succeeds, and the phone runs whatever was last built the other way (or fails with `Error 0xe8008014: The executable contains an invalid signature`). That reads exactly like a build that ignored your changes, and it is why the scripts exist.

Bare `ios-deploy` with no `--id` picks a phone on its own when two are attached, which is how a build lands on the wrong one.

---

## Why `--device` is required

`dx build --platform ios` (without `--device`) targets the iOS Simulator. A simulator binary crashes immediately on real hardware:
```
Library not loaded: /usr/lib/libobjc.A.dylib
Reason: wrong platform to load into process
```

---

## Verify the binary targets iOS

If something seems off, check the platform metadata:
```bash
vtool -show ~/Developer/zwipe/target/dx/zwipe/debug/ios/Zwipe.app/zwipe
# Should show: LC_VERSION_MIN_IPHONEOS
# NOT: LC_BUILD_VERSION platform 7 (simulator) or MACOS
```

---

## Using a different backend

The `BACKEND_URL` is baked into the binary at compile time via `env!()`.

- **Production**: `BACKEND_URL=https://api.zwipe.net`
- **Local dev**: omit `BACKEND_URL` (defaults to `.env` value, typically `127.0.0.1:3000`)

---

## Notes

- `dx build` handles code signing automatically using the provisioning profile from `~/Library/Developer/Xcode/UserData/Provisioning Profiles/`
- No manual `codesign` step needed for dev builds
- Install `ios-deploy` with `brew install ios-deploy` if not present
