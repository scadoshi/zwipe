# 377. The keyboard shortcuts button sits on the action docks instead of in the hint

State: fixing | Owner: halo | PR: orangui#38, orangui#39, #618 | Build seen: a59a3e4bc

## Report
From Scotty, through the composer: the keyboard shortcuts button should move into the `?` hint modal's action bar, as the little keyboard icon next to Got it and See guide, and come off every action dock (web and app). Shift+H still opens the shortcuts list.

## Fix
halo:
- **orangui#38:** `DialogAction::Icon { label, path, on_click }`, a small icon button in a dialog's action row drawn as the kit's icon buttons are, which closes the dialog, then runs. Enter ignores it, and text actions render byte-identical.
- **orangui#39:** the icon stands as tall as the text actions beside it.
- **#618:** zwipe pins orangui at 0218bd5e. `chrome/shortcuts_sheet.rs` builds the keyboard action (`shortcuts_action`, `use_shortcuts_action`) only while a screen with keys is mounted and on a computer's wide layout (`media::COMPUTER_QUERY`, the dock button's rule). The one-time hint dialogs (`OneTimeHintDialog`, `FirstDeckHintDialog`, `SwipeSelectHint`) put it after their guide link, and the button closes the hint and opens the shortcuts sheet. `DockShortcuts` and its CSS are gone from Add, Remove and Commander swipe. The app has no shortcuts button or sheet, so nothing changes there.
- Tests, written first: the action shows only on a computer where a screen has keys, and the computer query matches the old dock rule.

## Checked (headless Chrome, Add's hint from the web's own sheets)
- 1400: Got it, See guide ↗, then the keyboard icon, level with them.
- 390: Got it and See guide only.

## Verification
-
