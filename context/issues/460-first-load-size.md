# 460. A fresh tab's first load after a deploy downloads 2.75 MB of wasm

State: fixing | Owner: laptop | Type: feature | PR: - | Build seen: release c439fd91a

## Report
Scotty, via the composer, after 443 found that a cold first load is the wasm download: size three changes on a branch, each alone and combined: (a) immutable caching for the hashed `/assets/*`, (b) a pre-compressed brotli-11 wasm in place of Cloudflare's on-the-fly brotli, and (c) `opt-level = "z"`, LTO and one codegen unit for `wasm-release`. Report wasm size, cold load at 3 and 8 Mbps, warm load and any runtime slowdown. Nothing ships.

## Cause
- `profile.wasm-release` is plain `release`: optimized for speed, no LTO, 16 codegen units. That gives an 8.44 MB wasm.
- Cloudflare compresses it on the fly at brotli quality 3: 2.75 MB served (`cf-cache-status: DYNAMIC`). Production's wasm compressed locally at q3 is 2.747 MB.
- `_headers` sends `Cache-Control: no-cache` for everything, so every warm load revalidates each asset (a 304) although the names are content-hashed.

## Measured
Local release builds (`dx build --release --platform web`, dx 0.7.10) served by a script that emulates Pages: brotli at the given quality, ETags with 304s, and the given Cache-Control. The page is the shared 100-card Atraxa deck at 1280 in headless Chrome, with the API answered by the local zerver. Network throttled with 40 ms latency, CPU slowed 4x. Cold is a fresh context; warm reloads the same context. Times run from navigation to the deck's rows painted, median of 3.

| Build and serving | wasm raw | wasm sent | Cold 3 Mbps | Cold 8 Mbps | Warm |
|---|---|---|---|---|---|
| Today (release, q3, no-cache) | 8.44 MB | 2.74 MB | 9.1 s | 4.1 s | 0.80 s |
| (a) immutable assets | 8.44 MB | 2.74 MB | 9.2 s | 4.0 s | 0.65 s |
| (b) brotli q11 | 8.44 MB | 1.85 MB | 6.8 s | 3.1 s | 0.69 s |
| (c) opt-level z only | 6.15 MB | 2.08 MB | 7.4 s | 3.4 s | 0.74 s |
| (c) LTO + 1 codegen unit only | 8.96 MB | 2.70 MB | 9.1 s | 4.1 s | 0.70 s |
| (c) z + LTO + 1 unit | 4.98 MB | 1.75 MB | 6.5 s | 3.0 s | 0.71 s |
| All three | 4.98 MB | 1.30 MB | 5.2 s | 2.5 s | 0.66 s |

Warm is the same 0.6 to 0.8 s range everywhere: the bundle is cached either way, and immutable only drops the 304 round trips (the wasm's transfer goes from 300 B to 0).

Runtime, CPU 4x, network open, median of 3 runs per build:

| Build | Deck paint (cached) | List/Stacks x3, task time | 20 swipe skips on Add, script time |
|---|---|---|---|
| Today | 645 ms | 3.17 s | 622 ms |
| opt-level z | 712 ms | 3.10 s | 585 ms |
| LTO + 1 unit | 582 ms | 3.34 s | 566 ms |
| z + LTO + 1 unit | 681 ms | 3.11 s | 496 ms |

The runs spread by 150 to 300 ms each, so no build is measurably slower.

## Decision
Scotty, via the composer: ship (c) and (a) as on this branch; (b) stays a later step.

## Recommendation
- Ship (c), all three settings together (`opt-level = "z"`, `lto = true`, `codegen-units = 1` on `wasm-release`). The wasm drops 41% raw and 36% as Cloudflare sends it, with no runtime cost here. A release build takes about 1 minute longer on this laptop.
- Ship (a) with it: immutable `Cache-Control` for `/assets/*.wasm`, `*.js` and `*.css`. Not the whole `/assets/*`: the fonts and `og-app.png` there have no hash. It makes a warm load skip the revalidation round trips.
- (b) is worth about a third more on the wasm (1.75 MB to 1.30 MB), but Pages compresses on its own. Serving our own `.br` needs a `_headers` rule or a Function that sets `Content-Encoding: br`, and that has to be proven on a preview deploy before anyone relies on it. Next step, not part of this branch.

## Not checked
- A real phone. Production Cloudflare: the `_headers` splat rules (`/assets/*.wasm`) need a preview deploy to confirm Pages applies them and that `! Cache-Control` drops the `/*` value.
- The wasm-opt pass: dx 0.7.10 ran none here, and production's wasm is the same size as my baseline (8.41 vs 8.44 MB).

## Follow-up: the immutable rule cached a 404
Surfer, after (a) shipped: a reload right after a deploy stayed blank. Chrome held a 404 for the new build's JS with `Cache-Control: public, max-age=31536000, immutable`, and kept serving it from its cache.

Mechanism:
- The new `index.html` went live before its hashed JS reached every edge, so the browser asked an edge for an `/assets/*.js` it did not have yet.
- Pages answers a missing file with its SPA fallback: `index.html` with a 200, carrying the `_headers` rules for the requested path, the immutable `/assets/*.js` rule included.
- `zwiper-web/functions/_middleware.js` turns a 200 `text/html` on an unrouted path into a 404 and keeps the response's headers, so the 404 went out immutable for a year.
- The bad entry is keyed to that build's hashed URL: the next deploy that changes the bundle frees it, a deploy with an identical bundle does not.

Decision (the composer): keep `Cache-Control: no-cache` everywhere and do not mark `/assets` immutable again, in `_headers` or anywhere else. Setting the header from the middleware by status (immutable only on a 200 for a `-dxh<hash>` file, `no-store` on the fallback) would work, but it is an edge case only a live deploy can test, for about 100 to 150 ms of 304s on a warm load. (c) stays shipped.
