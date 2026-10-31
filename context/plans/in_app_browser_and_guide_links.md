# In-app browser and guide links from hints

**Status: planned 2026-10-31, not started.** Two app changes that ship together: links open inside Zwipe instead of leaving it, and every `?` hint that has a guide on zwipe.net links to it.

## Why

Hints are the shortest explanation the app gives, and zite already has the long one: 22 guides (`swipe-to-build`, `filtering`, `playstyle`, `activity`, `deck-stats`, `land-targets`, `budgeting`, `oracle-tags`, `commander-maybeboard`, `remove-cards`, `share-your-deck`, `import-export` and more). A hint that links to its guide answers the quick question and leaves the long one a tap away.

Today every link leaves the app. `zwiper/src/lib/outbound/open_url.rs` calls `UIApplication.openURL` on iOS, which switches to Safari, and sends an `ACTION_VIEW` intent on Android, which switches to the default browser. Guide links would add more of those trips, so the in-app browser comes first.

## Part 1: in-app browser

`open_url::open` keeps its signature. An `http` or `https` URL opens in the in-app browser; anything else (`mailto:` and the rest) keeps going to the OS exactly as now, since an in-app browser can't handle it.

- **iOS: `SFSafariViewController`.** Safari's engine as a sheet over Zwipe with a Done button, sharing Safari's cookies and autofill. Build it with `initWithURL:` and present it from the key window's root view controller with `presentViewController:animated:completion:`, on the main thread, in the same objc2 style as the current `openURL` call. Link the SafariServices framework. If no view controller can present, fall back to `openURL`.
- **Android: Custom Tabs.** The same `ACTION_VIEW` intent plus the Custom Tabs session extra (`android.support.customtabs.extra.SESSION`, a `Bundle` holding a null binder), which asks the browser to open as a tab over the app. No AndroidX dependency. A browser without Custom Tabs support opens normally, so the worst case is today's behavior. The JNI follows the local-frame pattern from #74.
- **Desktop and web builds** are unchanged.

Every link the app already opens gets this for free: Help's links, the privacy policy, Buy links, Share.

## Part 2: guide links from hints

- **One map, shared.** A hint topic to guide slug table lives in `zwipe-ui`, the shared client logic crate (no UI dependency), so the app, zwiper-tui and zwiper-web all link the same way. A topic with no guide has no entry and shows no link.
- **A test keeps it honest.** It fails if a mapped slug isn't one of zite's guide slugs. Renamed guides already keep working through zite's `#[redirect]` routes, so old app builds don't break.
- **The dialog.** `HintDialog` takes an optional guide link and draws "Read the guide" at the bottom, opened through `open_url::open`, so it lands in the in-app browser. The hint's own copy stays as it is.
- **URL shape.** `https://zwipe.net/guides/<slug>`, built in one place from a base constant.

## Order

1. Part 1 on its own branch, device-tested on an iPhone and an Android phone: an `https` link opens in the sheet and Done or back returns to the same screen; a `mailto:` link still opens the mail app; a browser without Custom Tabs still opens.
2. Part 2 after `zwipe-ui` lands on main, so the map is never copied into zwiper first.
3. Both ship in the next app release. Changelog entries per `development/changelog_guidelines.md`.

## Not in scope

- Deep links from guides back into a screen of the app or of zwiper-web. That waits for zwiper-web to have a public URL.
- Rewriting any hint copy.
