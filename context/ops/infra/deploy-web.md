# Deploy — zweb

Deployment for the zweb Dioxus web frontend. Hosted on GitHub Pages at `zwipe.net`.

The pipeline runs automatically on push to `main` when files under `zweb/**` change.
Manual trigger is also available.

---

## Automatic Deploy (push to main)

Any push to `main` that touches `zweb/**` triggers `.github/workflows/deploy-zweb.yml` automatically.

No action needed — just push and the workflow handles the rest.

---

## Manual Trigger

To deploy without a code change (e.g. re-deploy after a config fix):

GitHub → Actions tab → **Deploy zweb** → Run workflow → Run workflow

---

## What the Workflow Does

1. Installs `build-essential` (needed for proc-macro crates targeting WASM)
2. Installs `dioxus-cli` from source with `--force` (uses cargo cache keyed to `zweb/Cargo.lock`)
3. Runs `dx build --release --platform web` from `zweb/`
4. Writes `CNAME` (`zwipe.net`) into the build output
5. Copies `index.html` → `404.html` (SPA routing — GitHub Pages serves 404.html for unknown paths, Dioxus Router takes over)
6. Uploads build output as a GitHub Pages artifact and deploys

---

## Verify

Visit `https://zwipe.net` — confirm the latest changes are live.

---

## Notes

- First run after a `Cargo.lock` change takes ~8–10 minutes (compiles dioxus-cli from source)
- Subsequent runs restore dx from cache and finish much faster
- GitHub Pages config: **Repository Settings → Pages → Source**: GitHub Actions
- Custom domain `zwipe.net` with HTTPS enforced
