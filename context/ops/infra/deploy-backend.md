# Deploy — zerver + zervice

Manual deploy process for the backend binaries. Use when CI/CD is unavailable or you need to deploy a specific change by hand. The server builds the binaries itself — no cross-compilation needed.

---

## When to use this

- CI/CD pipeline is broken or not yet set up
- You need to hotfix directly on the server
- Testing a change before wiring it into the pipeline

---

## Steps

### 1. SSH into the server

```bash
ssh scadoshi@<tailscale-ip>
# or use the local alias:
zerver
```

### 2. Pull latest source

```bash
cd ~/zwipe-src && git pull
```

If `~/zwipe-src` doesn't exist yet, clone it first:
```bash
git clone <repo-url> ~/zwipe-src
cd ~/zwipe-src
```

### 3. Build release binaries

```bash
cargo build --release --bin zerver --bin zervice
```

Binaries output to `~/zwipe-src/target/release/` (workspace root, not `zerver/target/`).

First build will take several minutes. Subsequent builds are faster thanks to incremental compilation.

If `cargo` is not found: `source ~/.cargo/env`

### 4. Stop zerver

Linux blocks overwriting a running executable — must stop before copying:

```bash
sudo systemctl stop zerver
```

### 5. Copy new binaries

```bash
cp target/release/zerver target/release/zervice ~/zwipe/
```

### 6. Restart zerver

```bash
sudo systemctl start zerver
sudo systemctl status zerver
```

Confirm it shows `active (running)` with no errors in the last few log lines.

### 7. Verify the API is responding

```bash
curl https://api.zwipe.net/
# {"message":"zerver","status":"ready","version":"0.1.0"}
```

---

## Notes

- `zervice` (the nightly Scryfall sync cron) does **not** need a restart — the cron job calls
  the binary path directly, so the next scheduled run picks up the new binary automatically.
- To free disk space after deploying: `cd ~ && rm -rf ~/zwipe-src`
