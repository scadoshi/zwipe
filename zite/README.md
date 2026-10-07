# zite

Dioxus website for Zwipe at [zwipe.net](https://zwipe.net). Statically rendered from `Route::static_routes()`, with a few token-driven dynamic pages.

## Pages

- Landing (`/`)
- Guides index + individual guides (`/guides`, `/guides/:slug`)
- About, Changelog, Contribute, Discord (`/about`, `/changelog`, `/contribute`, `/discord`)
- iOS / Android download pages (`/download/ios`, `/download/android`)
- Privacy policy (`/privacy`, shared copy from `zwipe-core::legal`)
- Shared deck pages (`/deck/:token`)
- Email verification and password reset (`/verify/:token`, `/reset/:token`, shared validation from `zwipe-core`)

Shared UI and CSS (nav, changelog, theme picker, card details) come from `zwipe-components`; the changelog and card rendering stay identical to the app. The site kit zite shares with the portfolio comes from there too: the remembered theme (`use_persisted_theme`), the hero's `StatsStrip`, the demo and guide galleries (`GalleryFrame`, `GalleryFooter`, with `SITE_CSS`), and the scroll reveal and nav glide scripts (`REVEAL_JS`, `NAV_GLIDE_JS`), inlined in the head.

## Build

```bash
dx build --release --platform web
```
