# iOS Simulator Quick Reference

Managing simulator devices for screenshots, testing different screen sizes, etc.

---

## Simulator.app is gone as of Xcode 27

Xcode 27 ships no `Simulator.app`. `DeviceHub.app` replaced it, at `/Applications/Xcode.app/Contents/Applications/DeviceHub.app`, and `open -a Simulator` now fails outright. Nothing was misconfigured; Apple removed it.

If the old window is wanted back, install Xcode 26 alongside 27 and use its `Contents/Developer/Applications/Simulator.app`. Both versions coexist and `xcode-select` decides which owns the toolchain. An older Simulator may refuse a newer runtime, so pair it with a runtime it knows.

## Default device: iPhone 11 Pro Max (6.5")

The project default for running and screenshotting is the **6.5" iPhone 11 Pro Max** (the App Store's required 6.5" size, 1242×2688). Screenshot size comes from the device model, not the runtime, so any installed iOS runtime will do.

`dx serve --platform ios` installs to the *booted* simulator and will **not boot one for you**, which is what makes it look like nothing happens:

```bash
zcripts/ios/sim.sh              # shuts others down, boots the 11 Pro Max
zcripts/ios/sim.sh "iPhone 17"  # or any other created device
```

Then `cd zwiper && dx serve --platform ios`.

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

# iPhone 11 Pro Max on iOS 18.6 (6.5" for App Store screenshots)
xcrun simctl create "iPhone 11 Pro Max" \
  "com.apple.CoreSimulator.SimDeviceType.iPhone-11-Pro-Max" \
  "com.apple.CoreSimulator.SimRuntime.iOS-18-6"
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

- **In Simulator window**: `Cmd+S`: saves to Desktop
- **From CLI**:
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
| 6.7" | 1290×2796 | iPhone 15/16 Pro Max |
| 6.5" | 1242×2688 | iPhone 11 Pro Max, XS Max |
| 5.5" | 1242×2208 | iPhone 8 Plus, 7 Plus |

App Store Connect requires at least the 6.5" size. Uploading 6.7" screenshots works; they get scaled down automatically.
