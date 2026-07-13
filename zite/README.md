# zite

Dioxus website for Zwipe at [zwipe.net](https://zwipe.net). Statically rendered
from `Route::static_routes()`, with a few token-driven dynamic pages.

## Pages

- Landing (`/`)
- Guides index + individual guides (`/guides`, `/guides/:slug`)
- About, Changelog, Contribute, Discord (`/about`, `/changelog`, `/contribute`, `/discord`)
- iOS / Android download pages (`/download/ios`, `/download/android`)
- Privacy policy (`/privacy`, shared copy from `zwipe-core::legal`)
- Shared deck pages (`/deck/:token`)
- Email verification and password reset (`/verify/:token`, `/reset/:token`,
  shared validation from `zwipe-core`)

Shared UI and CSS (nav, changelog, theme picker, card details) come from
`zwipe-components`; the changelog and card rendering stay identical to the app.

## Build

```bash
dx build --release --platform web
```
