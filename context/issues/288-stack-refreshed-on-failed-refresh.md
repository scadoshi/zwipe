# 288. Add says Stack refreshed when the refresh failed

State: approved (easy) | Owner: laptop | PR: - | Build seen: 1e03520ef

## Report
Surfer, live web: on Add with the network down, Refresh shows 'Connection error' and also 'Stack refreshed'. Only a refresh that succeeded should say Stack refreshed. Check the app too.

## Fix
Branch `nits/laptop-288`. Add's Refresh raised "Stack refreshed" when it sent the search, so a refresh that failed said both "Connection error" and "Stack refreshed". The same was true of the Maybeboard source's "Maybeboard refreshed".

- App (zwipe-ui's Add model): a refresh's search carries `refresh: true` in place of `explains_empty: false`, and "Stack refreshed" is raised when its reply lands. A failure raises nothing beyond the request's own error toast. "Maybeboard refreshed" is raised when the maybeboard's deck fetch lands.
- Web: both toasts move into the success branch of their request.

Failing test first: `refresh_searches_again_without_explaining_an_empty_reply` (now no toast on send, the toast on landing) and `a_failed_refresh_never_says_stack_refreshed`, plus `the_maybeboard_says_refreshed_only_once_its_deck_lands`.
