# Landing screen FOUC — unformatted HTML flashes before styling

**Source:** self-noted, 2026-06-29.
**Verdict:** undecided (worth doing — it's a first-impression polish item).

The landing screen in the app loads with the classic WASM/WebView flash: raw,
unstyled HTML renders for a beat, then the CSS kicks in and it snaps into the
real layout. Reads as janky on first launch — exactly the moment first
impressions are formed.

## To figure out tomorrow

- Confirm the surface: is this the in-app first screen (Dioxus WebView) or the
  `zite` landing at zwipe.net? The note says "in the application," so likely the
  app's initial render, but verify before fixing.
- Root cause is almost certainly the stylesheet loading/applying after the first
  paint of the HTML body. Options to evaluate:
  - Hold first paint until CSS is ready (hide `body` until styles applied, then
    reveal — a `visibility`/opacity gate released on load).
  - Inline the critical CSS so the first paint is already styled.
  - A lightweight splash/skeleton that covers the unstyled flash.
- Cross-check against the bottom-sheet startup work just done — same family of
  "first paint before things settle" WebView timing problems.
