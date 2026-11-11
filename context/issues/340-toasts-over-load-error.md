# 340. Stacked error toasts cover the "Couldn't load" box

State: fixing | Owner: laptop | PR: #522, #528 | Build seen: live 21ca16a04, failed on 1a5e45a44

## Report
Droid, 2026-11-11, live 21ca16a04, phone web. When cards or printings fail to load, error toasts stack up and sit over the text of the "Couldn't load cards" / "Couldn't load printings" box for as long as they're up.

## Cause
Every authed call that fails toasts its error through `Authed`, on the web and in the app. A failed load gets retried on its own: Retry, the connection coming back (`use_back_online`), and a second load beside the first (Add's deck context and its search, a printing sheet reopened). Each retry fails the same way and toasts the same words again, so copies of one message stack until they reach the box. The box already shows the failure in place and has Retry, so those copies add nothing.

## Fix
`zwipe_ui::toast::RepeatGate` holds back a toast whose words are already up and remembers each shown one until its toast is gone. Both clients' `Authed` pass every error toast through one shared gate, so a failure retried shows one toast, a different failure still shows its own, and the same failure shows again once its toast has gone. Tests in `zwipe-ui/src/toast.rs` (`repeat_gate_tests`) failed first.

## Single toast (follow-up)
One toast alone, measured in the real web client (local zerver, then the server stopped so the load fails), at 390, 411, 768, 900 and 1440 wide:
- Add, "Couldn't load cards": no overlap at any width. The box sits mid-screen, and at 390 the toast's top is 235px below the box text.
- Printings, "Couldn't load printings": below the modal breakpoint (60rem) the sheet is a bottom sheet with no `.sheet-dock` (its checkmark is in its own foot), so the "end above the toast" rule for docked sheets never applied. The toast covered the sheet's title and the box text (24px overlap at 390).
- Fix: below 60rem, a sheet without a `.sheet-dock` also ends its content above the toast band while a toast is up (`web.css`). The band is 12.5rem, which clears a three-line toast (the connection error), and the docked rule moves from 10.5rem to 12.5rem in both clients for the same reason. After the fix, the text and Retry clear the toast by 21px at 390, 411 and 768. At 900 and 1440 the sheet is a centered modal and the bottom-left toast never reaches it.
- App: its Printings sheet has a `.sheet-dock`, so the raised docked rule covers it. It wasn't run here.

## Failed verify on 1a5e45a44 (narrow)
Surfer, at 390, on Add, then Card details, then Printings: the three-line toast (y498-605) covered Retry (y497-529). The #522 rule was in effect (sheet padding 200px = 12.5rem), but on Add the stack stands 124px up (`--toast-bottom: 7.75rem`, over the swipe dock pair), not 4.5rem. A fixed 12.5rem only cleared a stack at 4.5rem.

## Fix 2
`--toast-bottom` moves from `.toast-container` onto `body`, so every rule that raises the stack (over a dock, a swipe bottom row, the narrow web dock pair) sets it once, for the stack and for anything that needs to clear it. `--toast-band-top` on `body` is `--toast-bottom + 8rem` (a three-line toast is about 6.75rem, plus a gap) plus the safe-area inset. Both clear-the-toast rules (docked sheets in both clients, dockless sheets below 60rem on the web) pad by it, so a sheet clears the stack wherever a screen puts it.

Measured in the real web client (local zerver, then the server stopped), on Remove, which raises the stack the same way as Add (Add's local search never filled its pile here, so Card details stayed disabled):
- Remove, then Card details, then Printings, at 390, 411 and 500: the toast stands 124px up, the sheet pads 252px, and Retry clears the toast by 21px.
- Plain Printings (Cards, then a row's Printings) at 390: the toast stands 72px up, the sheet pads 200px (the same as before), and Retry clears it by 21px.
