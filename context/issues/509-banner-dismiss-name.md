# 509. A banner's dismiss button has no name a screen reader can say

State: fixing | Owner: scotland | Type: issue | PR: - | Build seen: production

## Report
Surfer, on the web's new version notice (composer, 2026-11-13, taken from laptop): the banner's × has no aria-label or text. Check every banner.

## Before
orangui's `Banner` renders its dismiss button as `<button type="button" class="banner-dismiss">✕</button>`: no `aria-label`, its only content the glyph, which a screen reader reads as "multiplication x" or nothing. Every banner in the repo is this component: the web's new version notice (`zwiper-web/src/lib/inbound/chrome/fresh_build.rs`) and zwipe.net's home (`zite/src/pages/home.rs`); none is drawn by hand.

## Cause
`orangui/src/banner.rs`, `Banner`: the dismiss button carries no accessible name.

## Fix
scotland: orangui a495fe0 names the button `Dismiss` and hides the glyph from the name (`aria-hidden` on its span); its look is unchanged (the styles target the button).

## Verification
- orangui `tests/banner_name.rs`: the rendered button is `class="banner-dismiss" aria-label="Dismiss"` with the glyph in an `aria-hidden` span (red before the change).
