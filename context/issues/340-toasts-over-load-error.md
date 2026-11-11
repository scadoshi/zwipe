# 340. Stacked error toasts cover the "Couldn't load" box

State: fixing | Owner: laptop | PR: #522 | Build seen: live 21ca16a04

## Report
Droid, 2026-11-11, live 21ca16a04, phone web. When cards or printings fail to load, error toasts stack up and sit over the text of the "Couldn't load cards" / "Couldn't load printings" box for as long as they're up.

## Cause
Every authed call that fails toasts its error through `Authed`, on the web and in the app. A failed load gets retried on its own: Retry, the connection coming back (`use_back_online`), and a second load beside the first (Add's deck context and its search, a printing sheet reopened). Each retry fails the same way and toasts the same words again, so copies of one message stack until they reach the box. The box already shows the failure in place and has Retry, so those copies add nothing.

## Fix
`zwipe_ui::toast::RepeatGate` holds back a toast whose words are already up and remembers each shown one until its toast is gone. Both clients' `Authed` pass every error toast through one shared gate, so a failure retried shows one toast, a different failure still shows its own, and the same failure shows again once its toast has gone. Tests in `zwipe-ui/src/toast.rs` (`repeat_gate_tests`) failed first.
