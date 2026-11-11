# 339. A reply with an empty URL crashes web Add

State: fixing | Owner: thinker | PR: - | Build seen: live 1.18.16

## Report
Composer, 2026-11-11: the web client panics with "url parse" (wasm `__wbindgen_throw`) on a `Response` whose `url` is empty, killing the Add task.

## Diagnosis
thinker, 2026-11-11. The throw is inside reqwest, not zwipe-client. reqwest 0.12's wasm fetch runs `Url::parse(&js_resp.url()).expect_throw("url parse")` on every reply (`src/wasm/client.rs:261`) before handing it back, and an empty `url` gives `RelativeUrlWithoutBase`. The browser leaves `url` empty on a `Response` that script built, which is what a fetch-wrapping extension or a DevTools override returns. zwiper-web has no service worker or fetch shim of its own. Reproduced headless on release: a script wrapping `fetch` answered the deck search with `new Response(body)`, the console showed the throw, and Add sat with no card and no message.

## Fix
`zwiper-web/public/response-url.js`, loaded first in `index.html`'s head, wraps the `Response.prototype.url` getter. A response whose URL is empty reports the page's URL instead. The request's URL isn't reachable from the prototype, and zwipe-client never reads a reply's URL; reqwest only needs one that parses. Patching the prototype covers a `Response` built anywhere, whichever wrapper runs outermost. The reply then decodes as usual: a good body deals its cards, and a bad one fails into Add's "Couldn't load cards" with Retry. `script-src 'self'` already allows the file. Guard tests in `zwiper-web/tests/response_url.rs` keep the script loading first and keep the fallback.
