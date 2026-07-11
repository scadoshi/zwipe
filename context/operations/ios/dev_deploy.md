# Deploy a Build to Phone

Build and install on a connected iPhone. **Debug** is the daily workflow; **release**
is for verifying the store-parity build on-device.

**Prerequisites:** Device registered and dev profile installed (see [devices.md](devices.md)).

---

## Build and deploy

Paste each as a **single line**. Multi-line `\` continuations get mangled on paste into
zsh (`--device` and the `.app` path split off into their own commands → `command not
found: --device` / `permission denied: …Zwipe.app`).

**Debug** — outputs to `target/dx/zwipe/debug/ios/`:

```bash
cd ~/Developer/zwipe/zwiper && BACKEND_URL=https://api.zwipe.net dx build --platform ios --device "scotland-mobile" && ios-deploy --bundle ~/Developer/zwipe/target/dx/zwipe/debug/ios/Zwipe.app
```

**Release** — outputs to `target/dx/zwipe/release/ios/` (note `release/`, not `debug/`):

```bash
cd ~/Developer/zwipe/zwiper && BACKEND_URL=https://api.zwipe.net dx build --release --platform ios --device "scotland-mobile" && ios-deploy --bundle ~/Developer/zwipe/target/dx/zwipe/release/ios/Zwipe.app
```

> A `--release` build lands in `release/ios/`. Deploying the `debug/ios/` path after a
> release build installs the stale/unsigned debug app and fails with
> `Error 0xe8008014: The executable contains an invalid signature`.

That's it. The app is installed and ready to open.


---

## Why `--device` is required

`dx build --platform ios` (without `--device`) targets the iOS Simulator. A simulator
binary crashes immediately on real hardware:
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

- `dx build` handles code signing automatically using the provisioning profile
  from `~/Library/Developer/Xcode/UserData/Provisioning Profiles/`
- No manual `codesign` step needed for dev builds
- Install `ios-deploy` with `brew install ios-deploy` if not present
