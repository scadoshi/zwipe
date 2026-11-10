# 332. A failed request can toast an empty red box

State: fixing | Owner: halo | PR: (this PR) | Build seen: live web, before the cancel-email-change route deployed

## Report
Scotty, 2026-11-09, live web. Account, then Pending, then Cancel. The new route wasn't deployed yet, so the request got a 404, and the error toast showed as an empty red box with only an X and no message.

## Cause
A route the server doesn't have answers 404 with an empty body. The client keeps the body as the error's message (`ClientError::NotFound("")`), and `ClientError::to_user_message` shows a 404, 409 or 422 message as the server wrote it, after `sentence_case`, and 401, 403 and 429 the same way through its catch-all arm. An empty message stays empty, so the toast had no text. The app and the web app both turn every failed request into text through `to_user_message`, so any of those statuses with an empty body did the same on either.

## Fix
`to_user_message` falls back to "Something went wrong, please try again", the line it already shows for a 400 or a 500, whenever the message it would show is blank. A message the server does send still shows as before. The test `a_failure_with_no_message_still_says_something` covers every status variant with an empty and a whitespace-only message, and failed first.
