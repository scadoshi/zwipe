# 337. Taps on zwipe.net flash Chrome's blue highlight

State: fixing | Owner: thinker | PR: - | Build seen: live

## Report
Composer, 2026-11-11 (from halo's queue): zwipe.net (zite) never sets `-webkit-tap-highlight-color`, so every tap on a phone flashes Chrome's blue. app.zwipe.net sets it transparent.

## Diagnosis
thinker, 2026-11-11. The app and app.zwipe.net take the reset from components' `SCREENS_CSS` (`screens/base.css`), which zite doesn't load. zite's own `style.css` and the kit's sheets it loads never set the property. Live zwipe.net computes `rgba(51, 181, 229, 0.4)` on all 82 links and buttons on its home page.

## Fix
`zite/assets/style.css` sets `-webkit-tap-highlight-color: transparent` on `:root`. The property inherits, so every element on every zite page computes `rgba(0, 0, 0, 0)` (checked on `/` and `/about`). The portfolio site had the same gap and gets its own PR in the portfolio repo.
