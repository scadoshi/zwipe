# Deploy: zite

Deployment for the zite Dioxus web frontend. Hosted on GitHub Pages at `zwipe.net`.

The pipeline runs automatically on push to `main` when files under `zite/**`, `zwipe-core/**` or `components/**` change (or the workflow file itself). Manual trigger is also available.

---

## Automatic Deploy (push to main)

Any push to `main` that touches `zite/**`, `zwipe-core/**`, `components/**` or the workflow file triggers `.github/workflows/deploy-zite.yml` automatically.

No action needed; just push and the workflow handles the rest.

---

## Manual Trigger

To deploy without a code change (e.g. re-deploy after a config fix):

GitHub → Actions tab → **Deploy zite** → Run workflow → Run workflow

---

## What the Workflow Does

1. Installs `build-essential` (needed for proc-macro crates targeting WASM). dx downloads its own pinned `wasm-opt` regardless of PATH, so no binaryen is installed here (see [`cicd.md`](cicd.md))
2. Installs a prebuilt `dioxus-cli@0.7.10` binary via `taiki-e/install-action`, so nothing is compiled from source
3. Runs `dx build --release --platform web --ssg --force-sequential` from `zite/`. Both flags matter: `--ssg` pre-renders the static routes, and dropping `--force-sequential` lets the parallel client build overwrite the SSG output with the bare shell
4. Writes `CNAME` (`zwipe.net`) into the build output
5. Copies `index.html` → `404.html` (SPA routing, GitHub Pages serves 404.html for unknown paths, Dioxus Router takes over)
6. Uploads build output as a GitHub Pages artifact and deploys
7. `deploy-cloudflare` takes the same artifact, drops `404.html` and `CNAME`, and uploads it from `zite/` with `wrangler pages deploy` to the Cloudflare Pages project `zwipe-site`, creating the project on its first run. Running from `zite/` bundles `zite/functions` with it

---

## Cloudflare Pages and the share page's preview tags

`zite/functions/deck/[token].js` is a Pages Function for `/deck/:token`. It reads the deck from the public `GET /api/share/deck/:token` and serves the site shell with the deck's own title, description, image and canonical link, so link previews show the deck. An unknown, revoked or malformed token gets the generic tags with a 404, and a failing API gets the generic page with a 200. Every other path is a static file, with `index.html` and a 200 for an app route, since there is no `404.html`. `zite/tests/og_function.rs` keeps the Function's format names and commander formats matched to zwipe-core.

Both deploys run on every push until zwipe.net moves. Until then the Cloudflare copy serves at `zwipe-site.pages.dev`, where the preview tags can be checked with `curl`. The app there can't load a deck in the browser, since the API's CORS list doesn't include that origin. Secrets: `CLOUDFLARE_API_TOKEN` and `CLOUDFLARE_ACCOUNT_ID`, the same as `deploy-web.yml`.

Cutover (Scotty, in the dashboard):

1. Workers & Pages → `zwipe-site` → Custom domains: add `zwipe.net` and `www.zwipe.net`. Cloudflare replaces the GitHub A records and the `www` CNAME with its own.
2. Check that `curl -sI https://zwipe.net/deck/<a live token>` answers `200` with `server: cloudflare`.
3. Repository Settings → Pages: unpublish the site, then remove the `deploy` job and the CNAME and 404.html steps from `deploy-zite.yml`.

---

## Verify

Visit `https://zwipe.net` and confirm the latest changes are live.

---

## Notes

- GitHub Pages config: **Repository Settings → Pages → Source**: GitHub Actions
- Custom domain `zwipe.net` with HTTPS enforced

---

# Deploy: zwiper-web (app.zwipe.net)

The web app is hosted on Cloudflare Pages, project `zwipe-app`, at `app.zwipe.net`. `.github/workflows/deploy-web.yml` runs on a push to `main` that touches `zwiper-web/**`, the shared crates (`zwipe-core`, `zwipe-client`, `zwipe-ui`, `components`), `zwiper/assets/**` (the web still inlines the app's stylesheets) or `Cargo.lock`, and by hand from the Actions tab.

- `test` (`cargo test -p zwiper-web`) and `build` (`dx build --release --platform web` with `BACKEND_URL=https://api.zwipe.net`) run side by side; `deploy` waits on both and uploads the build with `wrangler pages deploy --branch main`.
- `zwiper-web/deploy/_headers` is copied into the build: the security headers, with the Content-Security-Policy in report-only until a click-through shows no violations.
- No `404.html`, so Pages answers app routes with `index.html` and a 200.
- Secrets: `CLOUDFLARE_API_TOKEN` (an account token, Pages Edit only, expires 2027-11-06) and `CLOUDFLARE_ACCOUNT_ID`.
- The API must list `https://app.zwipe.net` in `ALLOWED_ORIGINS` (`/etc/zwipe/zerver.env`): CORS and the cookie session routes both check it.
