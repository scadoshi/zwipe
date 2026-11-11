# 336. A deck card search that never answers leaves Add on the skeleton

State: fixing | Owner: thinker | PR: - | Build seen: live 1.18.16

## Report
Composer, 2026-11-11: card search has no client timeout. A deck card search that never answers leaves Add on the skeleton with no message for good; Refresh recovers.

## Diagnosis
thinker, 2026-11-11. `ZwipeClient::search_deck_cards` used `call`, which has no timeout of its own. In the browser, fetch never gives up, so a stalled search held web Add on the skeleton indefinitely. The native client gives up only at its 30-second limit for every call. Add's failure path (the "Couldn't load cards" card with Retry, and a toast) was already wired on both clients and only needed the call to end.

## Fix
`search_deck_cards` goes through `call_within` with `SEARCH_TIMEOUT` (20 seconds), as 333 did for printings, on every platform. Load-more pages use the same call. A stalled search becomes `ClientError::Network`, which Add already turns into "Couldn't load cards" + Retry and a connection-error toast. Test first in `zwipe-client`: `a_deck_card_search_gives_up_on_a_server_that_never_answers` runs on tokio's paused clock against a server that never answers. It failed before at 30 seconds and passes at 20.
