# Deploy — zite

Deployment for the zite Dioxus web frontend. Hosted on GitHub Pages at `zwipe.net`.

The pipeline runs automatically on push to `main` when files under `zite/**` change.
Manual trigger is also available.

---

## Automatic Deploy (push to main)

Any push to `main` that touches `zite/**` triggers `.github/workflows/deploy-zite.yml` automatically.

No action needed — just push and the workflow handles the rest.

---

## Manual Trigger

To deploy without a code change (e.g. re-deploy after a config fix):

GitHub → Actions tab → **Deploy zite** → Run workflow → Run workflow

---

## What the Workflow Does

1. Installs `build-essential` (needed for proc-macro crates targeting WASM) and `binaryen`, which supplies the `wasm-opt` dx would otherwise download from GitHub releases mid-build
2. Installs a prebuilt `dioxus-cli@0.7.10` binary via `taiki-e/install-action`, so nothing is compiled from source
3. Runs `dx build --release --platform web --ssg --force-sequential` from `zite/`. Both flags matter: `--ssg` pre-renders the static routes, and dropping `--force-sequential` lets the parallel client build overwrite the SSG output with the bare shell
4. Writes `CNAME` (`zwipe.net`) into the build output
5. Copies `index.html` → `404.html` (SPA routing — GitHub Pages serves 404.html for unknown paths, Dioxus Router takes over)
6. Uploads build output as a GitHub Pages artifact and deploys

---

## Verify

Visit `https://zwipe.net` — confirm the latest changes are live.

---

## Notes

- GitHub Pages config: **Repository Settings → Pages → Source**: GitHub Actions
- Custom domain `zwipe.net` with HTTPS enforced
