# Shipping zwipe to iOS + Pi Backend

Documented 2026-03-26. Full end-to-end record of getting zwipe running on a real iPhone
with a Raspberry Pi 5 backend served through Cloudflare Tunnel.

---

## Overview of What We Built

- **iOS app** on a real iPhone 17 (scotland-mobile), built with Dioxus, signed with a paid Apple Developer certificate
- **Backend (zerver)** cross-compiled for `aarch64-unknown-linux-gnu` and running as a systemd service on a Raspberry Pi 5
- **Cloudflare Tunnel** exposing `api.zwipe.net → localhost:3000` on the Pi — no port forwarding, TLS handled by Cloudflare
- **iOS Keychain session persistence** via `keychain-access-groups` entitlement — sessions survive cold launches

---

## Part 1: iOS Code Signing (The Hard Part)

### Why this exists

The `keyring` crate uses iOS Keychain for session storage. Without the `keychain-access-groups`
entitlement embedded in the signed app, every cold launch produces:
```
Platform secure storage failure: A required entitlement isn't present
```
The user has to log in every time. The fix requires a paid Apple Developer account.

### Apple Developer Account Setup

1. Purchased Apple Developer Program membership ($99/yr) at developer.apple.com
2. Account: `scottyfermo17@gmail.com`
3. Team ID: `VV74WQ89GD` (SCOTTY RAY FERMO)
4. **Important:** Xcode creates a separate "Personal Team" (`NVSWB62C54`) automatically when you first sign in. This is a free development team and is DIFFERENT from the paid team. Certificates created by Xcode's "Manage Certificates" button default to the personal team. This caused significant confusion — provisioning profiles from the paid portal have `VV74WQ89GD` but Xcode-created certs show `NVSWB62C54` in `security find-identity`. The OU field of the cert (not the CN) is the actual team ID used for matching.

### Registering the App ID

1. developer.apple.com → Identifiers → + → App IDs → App
2. Bundle ID (Explicit): `com.scadoshi.zwipe`
3. **Enable Keychain Sharing capability** ← critical, without this the provisioning profile won't include the entitlement
4. Register

### Creating the Entitlements Plist

`zwiper/Entitlements.plist`:
```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>application-identifier</key>
    <string>VV74WQ89GD.com.scadoshi.zwipe</string>
    <key>keychain-access-groups</key>
    <array>
        <string>VV74WQ89GD.com.scadoshi.zwipe</string>
    </array>
    <key>get-task-allow</key>
    <true/>
</dict>
</plist>
```

**Important notes:**
- Use the actual Team ID (`VV74WQ89GD`), NOT the `$(AppIdentifierPrefix)` Xcode variable — `codesign` doesn't expand Xcode build variables
- `get-task-allow: true` is required for development builds
- `application-identifier` must match what's in the provisioning profile

### Registering Your Device

1. developer.apple.com → Devices → + → iOS
2. Device Name: `scotland-mobile`
3. UDID: found via Finder → plug in iPhone → click device name line under the image until it shows UDID (format: `00008140-00166D6C3482801C`)
4. Register

### Creating the Provisioning Profile

1. developer.apple.com → Profiles → + → iOS App Development
2. App ID: `com.scadoshi.zwipe`
3. Certificate: select your Apple Development certificate (the one under `VV74WQ89GD`)
4. Device: `scotland-mobile`
5. Name: `zwipe-dev`
6. Download: `zwipedev.mobileprovision`
7. Install: `cp ~/Downloads/zwipedev.mobileprovision ~/Library/MobileDevice/Provisioning\ Profiles/`

**Why the profile kept disappearing from Provisioning Profiles:** macOS validates profiles on copy and removes them if no matching certificate+private key pair exists in Keychain. The fix was to generate a CSR manually (so the private key is in Keychain) and create the cert in the portal under the right team.

### Getting the Right Certificate in Keychain

Xcode's "Manage Certificates" button creates certs under the Personal Team, not the paid team.
The fix — generate a CSR via openssl and create the cert in the portal directly:

```bash
# Generate private key and CSR
cd ~/Desktop
openssl genrsa -out zwipe-key.pem 2048
openssl req -new -key zwipe-key.pem -out zwipe.certSigningRequest \
  -subj "/emailAddress=scottyfermo17@gmail.com,CN=SCOTTY RAY FERMO,C=US"

# Import the private key into Keychain
security import ~/Desktop/zwipe-key.pem -k ~/Library/Keychains/login.keychain-db -T /usr/bin/codesign
```

Then upload `zwipe.certSigningRequest` to developer.apple.com → Certificates → + → Apple Development → download and double-click the `.cer` to install.

**Verify:**
```bash
security find-identity -v -p codesigning
# Should show an entry with VV74WQ89GD in the OU
```

The cert that was already in the portal (created by Xcode) — fingerprint `F421F2E0FF6575A04BB18520C1A699A3F9CCEB45` — turned out to have `OU=VV74WQ89GD` in it after all. The `(NVSWB62C54)` shown by `security find-identity` is from the CN display name, not the team ID. So this cert works for signing against the provisioning profile.

**Use this fingerprint for signing:** `F421F2E0FF6575A04BB18520C1A699A3F9CCEB45`

### Building for Real Device (Critical Lesson)

`dx build --platform ios` defaults to the **iOS Simulator** target (platform 7). Running a simulator
binary on a real device crashes immediately with:
```
Library not loaded: /usr/lib/libobjc.A.dylib
Reason: wrong platform to load into process
```

**Fix:** pass `--device` to force Dioxus to target the physical device:
```bash
dx build --platform ios --device "scotland-mobile"
```

Verify the binary targets the right platform:
```bash
vtool -show target/dx/main/debug/ios/Main.app/main
# Should show: cmd LC_VERSION_MIN_IPHONEOS (not LC_BUILD_VERSION platform 7)
```

### Full Deploy Command (use every time)

```bash
cd /path/to/zwipe/zwiper

# 1. Build targeting physical device
dx build --platform ios --device "scotland-mobile"

# 2. Embed provisioning profile
cp ~/Downloads/zwipedev.mobileprovision \
  ../target/dx/main/debug/ios/Main.app/embedded.mobileprovision

# 3. Sign with correct cert + entitlements
codesign -f -s "F421F2E0FF6575A04BB18520C1A699A3F9CCEB45" \
  --entitlements zwiper/Entitlements.plist \
  ../target/dx/main/debug/ios/Main.app

# 4. Deploy to connected iPhone
ios-deploy --bundle ../target/dx/main/debug/ios/Main.app
```

**First launch:** iOS 16+ requires Developer Mode enabled:
Settings → Privacy & Security → Developer Mode → toggle on → restart

---

## Part 2: Raspberry Pi 5 Backend

### Hardware

- Raspberry Pi 5, 4GB RAM
- Debian 12 (Bookworm), aarch64
- Hostname: `pi`, user: `scottyfermo`
- Local IP: `10.0.0.78`

### PostgreSQL Setup

```bash
sudo apt update && sudo apt install -y postgresql postgresql-contrib
sudo systemctl enable postgresql
sudo systemctl start postgresql

# Create DB user and database
sudo -u postgres psql -c "CREATE USER zwipe WITH PASSWORD 'YOUR_DB_PASSWORD';"
sudo -u postgres psql -c "CREATE DATABASE zwipe OWNER zwipe;"
```

Test connection:
```bash
PGPASSWORD='YOUR_DB_PASSWORD' psql -U zwipe -h 127.0.0.1 -d zwipe -c '\l'
```

**Note:** Use `127.0.0.1` (TCP), not `localhost` (Unix socket). Peer authentication blocks socket connections for non-system users. TCP connections use password auth.

### Running Migrations

From Mac (migrations directory is in the repo):
```bash
for f in zerver/migrations/*.sql; do
  ssh scottyfermo@10.0.0.78 "PGPASSWORD='YOUR_DB_PASSWORD' psql -U zwipe -h 127.0.0.1 -d zwipe -f -" < "$f"
done
```

### Cross-Compiling zerver for aarch64

On Mac:
```bash
# Add the target
rustup target add aarch64-unknown-linux-gnu

# Install cross-linker
brew install messense/macos-cross-toolchains/aarch64-unknown-linux-gnu

# Configure Cargo linker (already in .cargo/config.toml)
# [target.aarch64-unknown-linux-gnu]
# linker = "aarch64-unknown-linux-gnu-gcc"

# Install zigbuild (handles OpenSSL cross-compilation — reqwest needs it)
brew install zig
cargo install cargo-zigbuild

# Build
cargo zigbuild --release --bin zerver --bin zervice --target aarch64-unknown-linux-gnu
```

**Why zigbuild:** `reqwest` depends on OpenSSL. Cross-compiling OpenSSL from Mac to Linux/ARM is painful. `cargo-zigbuild` uses the Zig compiler as a cross-linker and handles this automatically. We also switched reqwest to `rustls-tls` (pure Rust TLS) instead of the default `native-tls` (OpenSSL) in `Cargo.toml`:
```toml
[workspace.dependencies.reqwest]
version = "0.12.23"
features = ["json", "rustls-tls"]
default-features = false
```

### SSH Key Setup (Mac → Pi)

```bash
# Generate key (no passphrase issues)
ssh-keygen -t ed25519 -f ~/.ssh/id_ed25519

# Cache passphrase in macOS Keychain
ssh-add --apple-use-keychain ~/.ssh/id_ed25519

# Copy key to Pi
ssh-copy-id -o PubkeyAuthentication=no scottyfermo@10.0.0.78
```

### Deploying to Pi

```bash
# Create directory structure
ssh scottyfermo@10.0.0.78 "mkdir -p ~/zwipe/migrations"

# Copy binaries and migrations
scp target/aarch64-unknown-linux-gnu/release/zerver \
    target/aarch64-unknown-linux-gnu/release/zervice \
    scottyfermo@10.0.0.78:~/zwipe/

scp zerver/migrations/*.sql scottyfermo@10.0.0.78:~/zwipe/migrations/
```

### zerver .env on Pi

Located at `~/zwipe/.env`:
```
JWT_SECRET=<generated with: openssl rand -hex 32>
DATABASE_URL=postgres://zwipe:URL_ENCODED_DB_PASSWORD@127.0.0.1/zwipe
BIND_ADDRESS=0.0.0.0:3000
ALLOWED_ORIGINS=https://zwipe.net
RUST_LOG=info
RUST_BACKTRACE=1
```

**Note:** `<` and `>` in the password are URL-encoded as `%3C` and `%3E` in the connection string.

### zerver systemd Service

`/etc/systemd/system/zerver.service`:
```ini
[Unit]
Description=zerver - zwipe backend
After=network.target postgresql.service
Requires=postgresql.service

[Service]
Type=simple
User=scottyfermo
WorkingDirectory=/home/scottyfermo/zwipe
EnvironmentFile=/home/scottyfermo/zwipe/.env
ExecStart=/home/scottyfermo/zwipe/zerver
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl daemon-reload
sudo systemctl enable zerver
sudo systemctl start zerver
sudo systemctl status zerver
```

### zervice Cron (Nightly Scryfall Sync)

```bash
crontab -e
# Add: 0 4 * * * /home/scottyfermo/zwipe/zervice >> /home/scottyfermo/zwipe/zervice.log 2>&1
```

Run manually first time to seed card data:
```bash
cd ~/zwipe && ./zervice
```

---

## Part 3: Cloudflare Tunnel

### Why Cloudflare Tunnel

The Pi is behind a home router (dynamic IP, no port forwarding). Cloudflare Tunnel creates an
outbound connection from the Pi to Cloudflare's edge — no inbound firewall rules needed, no
static IP, TLS handled automatically by Cloudflare.

### Setup on Pi

```bash
# Install cloudflared
curl -L https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-arm64.deb \
  -o cloudflared.deb
sudo dpkg -i cloudflared.deb

# Authenticate (opens browser URL to authorize with Cloudflare account)
cloudflared tunnel login
# Saves cert to: ~/.cloudflared/cert.pem

# Create tunnel
cloudflared tunnel create zwipe
# Creates: ~/.cloudflared/70ba169b-8293-4a60-9b2d-e1f996a161db.json
# Tunnel ID: 70ba169b-8293-4a60-9b2d-e1f996a161db

# Route DNS (creates CNAME in Cloudflare)
cloudflared tunnel route dns zwipe api.zwipe.net
```

### Config File

`/etc/cloudflared/config.yml`:
```yaml
tunnel: 70ba169b-8293-4a60-9b2d-e1f996a161db
credentials-file: /etc/cloudflared/70ba169b-8293-4a60-9b2d-e1f996a161db.json

ingress:
  - hostname: api.zwipe.net
    service: http://localhost:3000
  - service: http_status:404
```

```bash
# Copy credentials to /etc/cloudflared
sudo mkdir -p /etc/cloudflared
sudo cp ~/.cloudflared/config.yml /etc/cloudflared/
sudo cp ~/.cloudflared/70ba169b-8293-4a60-9b2d-e1f996a161db.json /etc/cloudflared/

# Install as systemd service
sudo cloudflared service install
sudo systemctl enable cloudflared
sudo systemctl start cloudflared
```

**Note:** `nano` and other terminal programs fail over SSH from Ghostty with `Error opening terminal: xterm-ghostty`. Fix: prefix with `TERM=xterm-256color nano ...`

### Verify

```bash
curl https://api.zwipe.net/
# {"message":"zerver","status":"ready","version":"0.1.0"}
```

---

## Key Files Reference

| File | Purpose |
|------|---------|
| `zwiper/Entitlements.plist` | iOS entitlements (Keychain Sharing, app identifier) |
| `zwiper/Dioxus.toml` | Bundle ID: `com.scadoshi.zwipe` |
| `zwiper/.env` | `BACKEND_URL=https://api.zwipe.net` |
| `zcripts/pi-setup.sh` | One-shot Pi setup script |
| `~/zwipe/.env` (Pi) | zerver runtime config |
| `/etc/cloudflared/config.yml` (Pi) | Tunnel routing config |
| `~/Downloads/zwipedev.mobileprovision` | iOS dev provisioning profile (re-download yearly) |

## Cert / Account Reference

| Thing | Value |
|-------|-------|
| Apple ID | `scottyfermo17@gmail.com` |
| Paid Team ID | `VV74WQ89GD` |
| Bundle ID | `com.scadoshi.zwipe` |
| Signing cert fingerprint | `F421F2E0FF6575A04BB18520C1A699A3F9CCEB45` |
| Device UDID | `00008140-00166D6C3482801C` |
| Provisioning profile expiry | 2027-03-26 |
| Cloudflare Tunnel ID | `70ba169b-8293-4a60-9b2d-e1f996a161db` |
| Pi local IP | `10.0.0.78` |
| Pi SSH user | `scottyfermo` |
