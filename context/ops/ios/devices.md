# Registering Devices + Dev Provisioning Profiles

Add test devices and create provisioning profiles so you can deploy debug builds.

---

## 1. Find the device UDID

- Connect the iPhone via USB
- Open **Xcode → Window → Devices and Simulators**
- Select the device — UDID is shown under "Identifier"

Or from CLI:
```bash
ios-deploy --detect
# Shows UDID in the output
```

---

## 2. Register the device on Apple's site

1. developer.apple.com → Certificates, Identifiers & Profiles → Devices
2. Click **+**
3. Platform: **iOS**
4. Device Name: a recognizable name (e.g. `scotland-mobile`)
5. Device ID (UDID): paste from step 1
6. Click **Continue** → **Register**

---

## 3. Create an iOS App Development provisioning profile

1. developer.apple.com → Certificates, Identifiers & Profiles → Profiles
2. Click **+** → under **Development**, select **iOS App Development**
3. Select App ID: `com.scadoshi.zwipe`
4. Select your Apple Development certificate
5. Select the device(s) to include
6. Name it (e.g. "Zwipe Dev")
7. Download the `.mobileprovision` file

Install it:
```bash
cp ~/Downloads/*.mobileprovision \
  ~/Library/Developer/Xcode/UserData/Provisioning\ Profiles/
```

`dx build` picks up profiles from this directory automatically.

---

## 4. First launch on the device

**Developer Mode (iOS 16+):**
Settings → Privacy & Security → Developer Mode → toggle on → restart

**Untrusted Developer prompt:**
If iOS shows "Untrusted Developer" on first launch:
Settings → VPN & Device Management → your Apple ID → Trust

---

## Adding new test devices

Repeat steps 1–3. You need to:
1. Register the new UDID on developer.apple.com
2. **Regenerate** the provisioning profile to include the new device
3. Re-download and install the updated profile

---

## Profile expiry

Dev provisioning profiles expire after 1 year. When expired:
1. developer.apple.com → Profiles → find the expired profile → Edit or recreate
2. Download and reinstall

You'll know it's expired when `dx build` fails with a signing error.
