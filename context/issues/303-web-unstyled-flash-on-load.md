# 303. The web app flashes unstyled on a hard refresh

State: approved | Owner: thinker | PR: - | Build seen: 1d7053e88

## Report
Scotty, 2026-11-08 (screenshot, app.zwipe.net Home, hard refresh): for a moment the app renders with its markup but not its stylesheets. Serif browser font, the Z logo as a row of blue squares, a default blue underlined link, empty icon buttons, unstyled stats panel. "Strikes again": he has seen it before. Likely the app's stylesheets are attached by the wasm app at runtime (document::Stylesheet or similar), so the DOM paints before they load. index.html already has a first-frame style for before the wasm arrives.

## Fix direction
Make every stylesheet the app needs render-blocking in index.html's head (link rel=stylesheet with hashed names from the build, or inline the critical ones), so nothing paints unstyled. Check fonts too (font-display, preload).

## Fix
No prerender was involved: app.zwipe.net serves only the dx shell, whose body is an empty `#main`. The app inserted all its CSS at runtime (`document::Style` for the shared sheets and fonts, `document::Link` for `web.css`), and a frame probe on the live site found two to eight frames per load with markup painted in Times New Roman. Every sheet is now a CSS asset declared `with_static_head` in `zwiper-web/src/lib/inbound/sheets.rs`, so dx writes them as blocking, hash-named `<link rel="stylesheet">`s in the built head. dx bundles only files inside the crate, so the shared sheets are copied to `zwiper-web/assets/sheets`; a test keeps each copy equal to its constant (`ZWIPE_UPDATE_SHEETS=1 cargo test -p zwiper-web sheets` rewrites them). The fonts load from a static sheet, unhashed, and index.html preloads them.
