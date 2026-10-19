# iOS Simulator Quick Reference

Managing simulator devices for screenshots, testing different screen sizes, etc.

---

## Simulator.app is gone as of Xcode 27

Xcode 27 ships no `Simulator.app`. `DeviceHub.app` replaced it, at `/Applications/Xcode.app/Contents/Applications/DeviceHub.app`, and `open -a Simulator` now fails outright. Nothing was misconfigured; Apple removed it.

If the old window is wanted back, install Xcode 26 alongside 27 and use its `Contents/Developer/Applications/Simulator.app`. Both versions coexist and `xcode-select` decides which owns the toolchain. An older Simulator may refuse a newer runtime, so pair it with a runtime it knows.

## Default device: iPhone 16 Pro Max (6.9")

The project default for running and screenshotting is the **6.9" iPhone 16 Pro Max** (the App Store's required 6.9" size, 1320×2868; the 11 Pro Max's 6.5" set is still accepted but no longer the required one, so the 11 Pro Max device was deleted 2026-10-19). Screenshot size comes from the device model, not the runtime, so any installed iOS runtime will do.

`dx serve --platform ios` installs to the *booted* simulator and will **not boot one for you**, which is what makes it look like nothing happens:

```bash
zcripts/ios/sim.sh              # shuts others down, boots the 16 Pro Max
zcripts/ios/sim.sh "iPhone 17"  # or any other created device
```

Then `cd zwiper && dx serve --platform ios`.

dx installs the bundle with no icons at all, so the App icon sheet has nothing to switch to and Save says "No alternate icons in this build". Once the app is up, `zcripts/ios/sim_icons.sh` compiles the icons into that bundle and reinstalls it; the simulator's home screen then shows the pick. A full rebuild writes a fresh bundle, so run it again after one. Hot reloads keep the icons.

The script takes the newest iOS runtime carrying that device. It used to pin iOS 18.6, which broke silently the moment an Xcode update dropped that runtime: the lookup failed, nothing booted, and `dx serve` had nothing to install to.

---

## List what's available

```bash
# Installed runtimes (iOS versions)
xcrun simctl list runtimes

# Available device types (phone models)
xcrun simctl list devicetypes | grep -i iphone

# Created simulator instances (what you can actually boot)
xcrun simctl list devices available
```

---

## Create a simulator

Device types and runtimes must already be installed (they come with Xcode).

```bash
xcrun simctl create "<NAME>" "<DEVICE_TYPE>" "<RUNTIME>"
```

Examples:
```bash
# iPhone 15 Pro Max on iOS 26.4
xcrun simctl create "iPhone 15 Pro Max" \
  "com.apple.CoreSimulator.SimDeviceType.iPhone-15-Pro-Max" \
  "com.apple.CoreSimulator.SimRuntime.iOS-26-4"

# iPhone 16 Pro Max (6.9" for App Store screenshots)
xcrun simctl create "iPhone 16 Pro Max" \
  "com.apple.CoreSimulator.SimDeviceType.iPhone-16-Pro-Max" \
  "com.apple.CoreSimulator.SimRuntime.iOS-26-4"
```

---

## Boot, switch, and shut down

```bash
# Boot a specific simulator
xcrun simctl boot "<NAME>"
open -a Simulator

# Switch to a different device (shut down current first)
xcrun simctl shutdown all
xcrun simctl boot "<OTHER_NAME>"
open -a Simulator

# Shut down everything
xcrun simctl shutdown all
```

---

## Take screenshots

- In Simulator window: `Cmd+S`: saves to Desktop
- From CLI: 
  ```bash
  xcrun simctl io booted screenshot ~/Desktop/screenshot.png
  ```

---

## Delete a simulator

```bash
xcrun simctl delete "<NAME>"

# Or nuke all simulators and start fresh
xcrun simctl delete all
```

---

## App Store screenshot sizes

| Display | Resolution | Device examples |
|---------|-----------|-----------------|
| 6.9" | 1320×2868 | iPhone 16/17 Pro Max |
| 6.7" | 1290×2796 | iPhone 15 Pro Max |
| 6.5" | 1242×2688 | iPhone 11 Pro Max, XS Max |
| 5.5" | 1242×2208 | iPhone 8 Plus, 7 Plus |

App Store Connect requires the 6.9" size; a 6.5" set is still accepted and scaled, which is how every submission through 2026-10-19 went in.
