# Website: wire the live Google Play link (Android launched 2026-07-11)

**Status: READY TO BUILD. Small, self-contained. Android is now PUBLISHED on Google
Play, so the site + download page need to point at the real store instead of the
closed-beta tester flow.**

## One sentence

Android is live on Google Play (`com.scadoshi.zwipe`), so swap zite's `/download/android`
page from the beta tester-group flow to a real "Get it on Google Play" page, and add a
Play Store button to the landing page next to the App Store one.

## The Play Store URL

Canonical (locale-adaptive, store this): `https://play.google.com/store/apps/details?id=com.scadoshi.zwipe`
(the `&hl=en_US` the owner had just forces English; drop it.)

## Key finding — the client is ALREADY wired, do NOT touch it

`zwiper/src/lib/inbound/components/update_required.rs` already routes the min-version
upgrade screen through the site per platform:
- `STORE_PATH = "/download/android"` (android cfg) and `"/download/ios"` (ios cfg).

So the mobile client already points at zite's `/download/*` (the single redirect layer,
exactly as intended). **No client change is needed** for the upgrade screen. (Do a quick
grep for any *other* store CTA in the client — e.g. a "rate us" link — but the min-version
path is done.)

## What to change (all in zite)

1. **`zite/src/pages/android.rs`** — currently renders the **closed-beta flow** (join the
   Google Group → opt in → install; uses `GROUP_URL`). Replace it with a simple public
   download page mirroring **`zite/src/pages/ios.rs`**: headline + "Get it on Google Play"
   button linking to the Play Store URL, screenshots/copy as on iOS. Update `PageMeta`
   (drop "open beta" wording; it's live). The route `/download/android` already exists in
   `main.rs`, so this is a body swap only.
2. **`zite/src/main.rs`** (landing) — there are two **App Store ↗** buttons (~lines 151 &
   175, `https://apps.apple.com/us/app/zwipe-tcg/id6761341603`). Add a **Google Play**
   button alongside each so the landing offers both stores.
3. **`zite/src/pages/home.rs`** — has `APP_STORE_URL` const + `MobileApplication` JSON-LD
   with `downloadUrl` = the App Store. Add a `PLAY_STORE_URL` const and decide how to
   represent two platforms in the JSON-LD (`operatingSystem`, or a second entry). Update
   any "iOS only" / "coming to Android" copy.
4. **Consider centralizing** the two store URLs in `zwipe_core::domain::site` (that module
   already holds `WEB_BASE`/`SUPPORT_EMAIL`/`DISCORD_URL`) so the app store + play store
   URLs live in one place — optional, but tidy given both zite and (potentially) the client
   reference stores.

## Assets

Use the official **"Get it on Google Play"** badge per Google's brand guidelines (don't
hand-roll the wordmark). Mirror how the iOS page presents the App Store badge/button.

## Out of scope / already done

- Client upgrade-screen wiring (`update_required.rs`) — **done**, leave it.
- The `/download/android` route in `main.rs` — **exists**, leave it.
- Announcement/marketing post — tracked separately in `progress/todo.md` (Android section).

## Verify

`dx build --release --platform web --ssg` builds clean; `/download/android` shows the Play
button and links to the live listing; landing shows both store buttons; the client's
min-version screen (already pointing at `/download/android`) now lands on a real page that
redirects/links to Google Play.
