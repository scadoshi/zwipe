# 497. Account's Log out sits in the red Danger zone

State: fixing | Owner: thinker | Type: polish | PR: - (pushed to release) | Build seen: 8096283c4

## Report
surfer's walkthrough (`halo:~/surfer-shots/walkthrough/README.md`, item 5), triaged by Scotty: Account's Log out sits in "Danger zone" in the same red as Delete account; logging out isn't dangerous. Move it out, on the web and in the app.

## Before
Headless Chrome at 1440x900, signed-in fake session, `/account` with the user read mocked, 3 of 3: the Account list holds Username, Email and Password; the Danger zone holds Log out and Delete account, Log out's button `btn-action btn-action-danger`.

## Cause
`zwiper-web/src/lib/inbound/screens/account/mod.rs` and `zwiper/src/lib/inbound/screens/account/mod.rs` draw Log out as the Danger zone's first row with `danger: true`.

## Fix
Log out is the Account list's last row, after Password, with the plain action button; the Danger zone holds only Delete account. Log out still asks first. `zwipe_ui::screens::account`'s docs say where each lives.

## After
Same setup, 3 of 3 at 1440 and once at 390: the Account list is Username, Email, Password, Log out (`btn-action`); the Danger zone is Delete account alone.

## Not checked
- The app on a device: the same rows and classes in its own screen.
- A unit test: zwiper-web and zwiper have no harness to render the screen; the headless runs are the check.
