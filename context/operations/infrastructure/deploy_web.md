# Deploy: zite

Deployment for the zite Dioxus web frontend. Hosted on Cloudflare Pages (project `zwipe-site`) at `zwipe.net`.

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
4. Uploads the build output as one artifact
5. `deploy-cloudflare` unpacks it and uploads it from `zite/` with `wrangler pages deploy` to the Cloudflare Pages project `zwipe-site`, creating the project on its first run. Running from `zite/` bundles `zite/functions` with it

---

## Cloudflare Pages and shared deck links

A shared deck lives on the web app (see app.zwipe.net below). `zite/functions/deck/[token].js`, which `/:owner/deck/:slug/:token` re-exports, answers both shared deck links on zwipe.net with a 301 to the same path on app.zwipe.net (the bare token form to `/_/deck/deck/:token`, which the web app corrects to the deck's owner and slug). Phones built before the move mint zwipe.net links, so the redirect stays. `zite/tests/share_redirect.rs` pins it. Every other path is a static file, with `index.html` and a 200 for an app route, since there is no `404.html`.

zwipe.net moved to the `zwipe-site` Pages project on 2026-11-07; GitHub Pages is unpublished and its deploy job is gone. Secrets: `CLOUDFLARE_API_TOKEN` and `CLOUDFLARE_ACCOUNT_ID`, the same as `deploy-web.yml`.

The apex record is a proxied CNAME `@` → `zwipe-site.pages.dev`; Cloudflare didn't replace GitHub's A records on its own, so they were deleted by hand. Check a deploy with `curl -sI https://zwipe.net/deck/<a live token>`: `200`, `server: cloudflare`, and the deck's own `og:title`.

---

## Verify

Visit `https://zwipe.net` and confirm the latest changes are live.

---

## Notes

- Custom domains `zwipe.net` and `www.zwipe.net` are attached to `zwipe-site` in the Cloudflare dashboard

---

# Deploy: zwiper-web (app.zwipe.net)

The web app is hosted on Cloudflare Pages, project `zwipe-app`, at `app.zwipe.net`. `.github/workflows/deploy-web.yml` runs on a push to `main` that touches `zwiper-web/**`, the shared crates (`zwipe-core`, `zwipe-client`, `zwipe-ui`, `components`), `zwiper/assets/**` (the web still inlines the app's stylesheets) or `Cargo.lock`, and by hand from the Actions tab.

- `test` (`cargo test -p zwiper-web`) and `build` (`dx build --release --platform web` with `BACKEND_URL=https://api.zwipe.net`) run side by side; `deploy` waits on both and uploads the build with `wrangler pages deploy --branch main`.
- `zwiper-web/deploy/_headers` is copied into the build: the security headers, with the Content-Security-Policy in report-only until a click-through shows no violations.
- No `404.html`, so Pages answers app routes with `index.html` and a 200.
- `zwiper-web/functions/[owner]/deck/[slug]/[token].js` is a Pages Function for a shared deck's link. It reads the deck from the public `GET /api/share/deck/:token` and serves the app's shell with the deck's own title, description, preview image and canonical link, so link previews (Teams, WhatsApp, X, iMessage, Discord, Slack) show the deck. An unknown, revoked or malformed token gets the app-wide tags with a 404, and a failing API the app-wide tags with a 200. `deploy` runs `wrangler pages deploy` from `zwiper-web/` so the function uploads with the build. `zwiper-web/tests/og_function.rs` keeps its format names, commander formats, slug cap and preview URL matched to zwipe-core. Check a deploy with `curl -sA "Twitterbot/1.0" https://app.zwipe.net/<owner>/deck/<slug>/<token> | grep og:`: the deck's own `og:title`, `og:description` and `og:image`.
- Secrets: `CLOUDFLARE_API_TOKEN` (an account token, Pages Edit only, expires 2027-11-06) and `CLOUDFLARE_ACCOUNT_ID`.
- The API must list `https://app.zwipe.net` in `ALLOWED_ORIGINS` (`/etc/zwipe/zerver.env`): CORS and the cookie session routes both check it.
