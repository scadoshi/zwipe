# 333. Printings sits empty after a slow fetch, with no error or retry

State: fixing | Owner: halo | PR: (this PR) | Build seen: droid, aea5d98c4

## Report
Droid, on the app with the network throttled to EDGE or GPRS (also UMTS). Opening Printings showed the skeleton for 20 seconds or more, then the sheet sat empty with no error and no Retry, and stayed empty after the network came back. Closing and reopening the sheet fixed it.

## Cause
Two things. The request did fail, but the sheet dropped the failure: the shared model (`zwipe_ui::screens::printing_sheet`) handled `Failed` by stopping the skeleton and keeping nothing, which reads as `Status::Empty`, so the sheet drew an empty body with nothing to press. The fetch only ended at all because the app's reqwest client has a 30 second overall timeout; the web build's client has none, so there the same fetch could hang on the skeleton indefinitely.

## Fix
The model has `Status::Failed` and `Action::Retry`. A failed fetch keeps the card and printing it was for, the sheet shows the shared `LoadFailed` box ("Couldn't load printings" with Retry, the grammar the other load failures use), and Retry sends the same fetch again under a new ticket. Opening another card, or a new sheet, forgets the failure. The web sheet keeps its own signals and gets the same box and Retry.

The printings fetch goes through a new `ApiClient::call_within`, which sets a per-request timeout (`PRINTINGS_TIMEOUT`, 20 seconds) on app and web alike, so a stalled request ends in an error the sheet can show.

Tests: five model tests in `printing_sheet/tests.rs` (the failed fetch says so, Retry recovers, a retry can fail again, another card after a failure, a new sheet retries nothing), which failed first, and `a_call_within_a_timeout_gives_up_on_a_server_that_never_answers` in zwipe-client against a listener that never replies. Checked live on the web build by stopping the local server, opening Printings (the box showed), restarting it and pressing Retry (the cover flow loaded).
