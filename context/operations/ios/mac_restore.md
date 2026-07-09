# Mac Restore Checklist

What to back up and restore when wiping this Mac to return to stable macOS.

---

## Before wiping — back up these files

### Critical (cannot be recreated without these)

| What | Where | Why |
|------|-------|-----|
| `~/certs/` | Copy to USB/iCloud | Distribution cert private key (`zwipe-dist-key.pem`) — if lost, must revoke and recreate the cert on developer.apple.com |
| `~/.ssh/` | Copy to USB | SSH keys for server access. Can be regenerated with physical server access, but saves time |
| Git repo | `git push` all branches | Code is on GitHub, just make sure everything is pushed |

### Regenerate after restore (not dangerous to lose)

| What | How to recreate |
|------|----------------|
| `zerver/.env` | Copy from server: `ssh scadoshi@zerver cat ~/zwipe/.env` — or recreate with the values in the server's `.env` |
| `zwiper/.env` | Just `BACKEND_URL=https://api.zwipe.net` and `RUST_LOG=info` |
| Apple signing certs in Keychain | Re-import `~/certs/zwipe-dist-key.pem` + re-download `.cer` from developer.apple.com |
| Provisioning profiles | Re-download from developer.apple.com → Profiles |
| Homebrew, Rust, Xcode | Reinstall from scratch — `zcripts/dev-env/macos/setup.sh` covers most of it |
| Tailscale | Reinstall + re-auth — server access via `ssh scadoshi@zerver` |

---

## Server access — not at risk

You have **physical access** to the server. Even if you lose all SSH keys:
1. Plug in a keyboard/monitor
2. Log in as `scadoshi`
3. Add your new SSH public key to `~/.ssh/authorized_keys`

The server itself is unaffected by your Mac wipe. Nothing to worry about there.

---

## Things to watch out for

### Keychain items
- The Apple Distribution cert is tied to a private key in your login keychain
- After restore, re-import `zwipe-dist-key.pem` into the keychain:
  ```bash
  security import ~/certs/zwipe-dist-key.pem -k ~/Library/Keychains/login.keychain-db -T /usr/bin/codesign
  ```
- Then double-click the `.cer` file to pair it with the key

### Database password
- The DB password is only in `~/zwipe/.env` on the server
- If you don't have it memorized, copy it before wiping (or just SSH in after restore)

### GitHub auth
- You'll need to re-auth with GitHub after restore (`gh auth login` or SSH key)
- Your repo is public on GitHub — nothing to lose, just need push access back

### Apple Developer account
- Not tied to your Mac at all — it's your Apple ID
- Certs and profiles are re-downloadable from developer.apple.com
- The only thing that can't be re-downloaded is the private key (`zwipe-dist-key.pem`)

---

## After restore — setup order

1. Install stable macOS (not beta)
2. Install Xcode from Mac App Store (GM version)
3. Install Homebrew, Rust, cargo tools (`zcripts/dev-env/macos/setup.sh`)
4. Clone repo from GitHub
5. Copy `~/certs/` back from backup
6. Import signing key + install certs
7. Set up SSH keys (new or restored) + Tailscale
8. Create `.env` files for zerver and zwiper
9. Build and test: `dx serve --platform ios`
10. Rebuild release, sign, package, upload to App Store — follow `operations/ios/app-store/submission/build.md`
