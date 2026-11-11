# 339. A reply with an empty URL crashes web Add

State: closed | Owner: thinker | PR: #523 (closed) | Build seen: live 1.18.16

## Report
Composer, 2026-11-11: the web client panics with "url parse" (wasm `__wbindgen_throw`) on a `Response` whose `url` is empty, killing the Add task.

## Diagnosis
thinker, 2026-11-11. The throw is inside reqwest, not zwipe-client. reqwest 0.12's wasm fetch runs `Url::parse(&js_resp.url()).expect_throw("url parse")` on every reply (`src/wasm/client.rs:261`) before handing it back, and an empty `url` gives `RelativeUrlWithoutBase`. The browser leaves `url` empty on a `Response` that script built, which is what a fetch-wrapping extension or a DevTools override returns. zwiper-web has no service worker or fetch shim of its own. Reproduced headless on release: a script wrapping `fetch` answered the deck search with `new Response(body)`, the console showed the throw, and Add sat with no card and no message.

## Proposed fix (not taken)
`zwiper-web/public/response-url.js`, loaded first in `index.html`, would have wrapped the `Response.prototype.url` getter so an empty URL reported the page's instead (#523). Repro and how-to-check are in that PR.

## Closed
Scotty and the composer, 2026-11-11: closed as an upstream bug, with no app-wide `Response.url` patch for a rare extension case. reqwest's wasm client still parses the reply's URL with `expect_throw` as of 0.13.5. Upstream: [seanmonstar/reqwest#2489](https://github.com/seanmonstar/reqwest/issues/2489) and [seanmonstar/reqwest#1401](https://github.com/seanmonstar/reqwest/issues/1401). Revisit when a reqwest release turns the bad URL into an error.
