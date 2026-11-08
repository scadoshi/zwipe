# Web tracking parity

Nit 269: zwiper-web sends exactly what the app (zwiper) sends, with the same fields and timing, tagged `web`. This is the audit, written before the fix. The Status column says what the fix PR does.

Every usage event below lands in one `HttpUsageBatch` (`POST` record usage), built by `zwipe_ui::telemetry::UsageBuffer`. The buffer stamps each error report with `ClientPlatform::CURRENT` and the build version, so the same type on wasm32 tags `web`. Web uses the endpoints and types the app already uses. The one contract change is additive: `HttpAnonymousEvent` gains an optional `platform` (below).

## Usage batch

| Event | App: where, fields, when | Web before | Status |
|---|---|---|---|
| Swipe counts (`swipes_left/right/up/down`) | Add, Remove and Swipe select stacks: one count per committed swipe in its direction, including a down swipe (undo) and Remove's down pull on the end ghost | Missing | Added on web's Add, Remove and Swipe select |
| Searches (`searches`) | Add's search and each load more; deck cards view's Quick add search; deck form command-zone typeahead (per debounced search); Swipe select search (signed in only); Commander maybeboard Quick add | Missing | Added at the same points |
| Suggestion signals (`signals`: deck, card oracle id, added / skipped / maybed) | Add, Search source only: every committed left, right or up swipe | Missing | Added |
| Swipe select signals (`select_signals`: card oracle id, chosen / skipped) | Swipe select: right (chosen) and left (skipped); up and down send none | Missing | Added |
| Removals (`removals`: deck, card oracle id) | Remove: a right swipe; deck cards view: a row removed from the deck (model's `RecordRemoval`) | Missing | Added |
| Pile ends (`pile_ends`: deck, empty / exhausted, synergy) | Add: the model's `PileEnd` effect (a search that served nothing, or load more that found nothing new) | Missing | Added at web's matching points |
| Handled errors (`client_errors`: screen, component, action, kind, message, count, platform, version) | Every failed authed call through `Authed` (screen pinned per screen, component and op per call), plus direct reports on Add, Remove, deck view, printing sheet and Account's resend. Network errors are logged only; decode messages keep their shape only | Missing: web's `Authed` logs and toasts but reports nothing | Web's `Authed` reports through the shared buffer with the same screen vocabulary |

## Flushing

| Trigger | App | Web before | Status |
|---|---|---|---|
| Timer | Every 30 s (`FLUSH_INTERVAL_SECS`), delayed ticks coalesce | Missing | Same 30 s timer |
| Background | App lifecycle goes to background | Missing | `visibilitychange` to hidden (the browser's background) |
| Before a search | Add flushes buffered usage before its search, since signals feed synergy order | Missing | Same |
| Before Clear skips | Deck view flushes so this window's skips are cleared too | Missing | Same |
| Logout | Buffer emptied when the session ends, so counts never go out under the next account | Missing | Same |
| Failure | Batch dropped (no retry); no session drops it | n/a | Same |

## Outside the batch

| Event | App | Web before | Status |
|---|---|---|---|
| Anonymous funnel: `app_opened` | Once per launch while signed out, with a per-launch random session id held in memory | Wired but switched off: events carried no platform, so web visits would have counted as app funnel | On, once per page load while signed out |
| Anonymous funnel: `register_viewed` | Register mounts | Wired, switched off | On |
| Anonymous funnel: `register_submitted` | A validated register submit goes out | Wired, switched off | On |
| Funnel platform | None sent | None sent | `HttpAnonymousEvent.platform` (optional, `serde(default)`), stamped by `zwipe-client` with `ClientPlatform::CURRENT` for the app and web alike; stored in the new nullable `anonymous_events.platform` (NULL is an older client) |
| Crash report (`HttpCrashReport`: crash id, version, platform, message, time) | Panic hook writes it to disk; the next launch posts it and clears it only on a 2xx | Missing | Panic hook writes to localStorage; the next load posts and clears it the same way |
| Login platform and version | `zwipe-client` stamps `ClientPlatform::CURRENT` and the version on every login | Same path (`login_cookie` calls the same stamp): tags `web` | Already at parity |
| Refresh version | `refresh` sends the version | `refresh_cookie` sends the version | Already at parity |
| `last_active_at`, activity heatmap, stats counters | Server-side: derived from authed requests, deck card writes and the usage batches above | Same requests; heatmap and counters were missing the web's swipes and searches | Follows from the batch |

## Shared code

The lock around zwipe-ui's buffer and the status to report-kind mapping move out of zwiper into zwipe-ui (`telemetry::SharedUsageBuffer`, `ErrorKind::from_status`), so both clients run one path. zwipe-ui can't depend on zwipe-client, so the shared handle takes a `Failure` (network, decode, or a status with its message), and each client turns its `ClientError` into one with `ClientError::status()`. The flush itself stays per client: the app's timer and lifecycle are tokio and mobile, web's are the browser's.
