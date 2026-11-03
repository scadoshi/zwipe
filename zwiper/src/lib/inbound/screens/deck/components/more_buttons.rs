//! The view deck screen's More sheet and the Buy, Share and Clear skips
//! dialogs it opens.

use components::{AlertDialog, BottomSheet, Button, DialogAction};
use dioxus::prelude::*;
use zwipe_ui::screens::deck_view::{self, Action, Dialog, MoreItem};

/// Whether the More sheet and each of the deck view's dialogs is up.
#[derive(Clone, Copy, PartialEq)]
pub(crate) struct DeckDialogs {
    /// The More sheet.
    pub(crate) sheet: Signal<bool>,
    /// The Buy dialog.
    pub(crate) buy: Signal<bool>,
    /// The Clone dialog.
    pub(crate) clone: Signal<bool>,
    /// The Share dialog.
    pub(crate) share: Signal<bool>,
    /// The Clear skips dialog.
    pub(crate) clear_skips: Signal<bool>,
    /// The Delete dialog.
    pub(crate) delete: Signal<bool>,
}

impl DeckDialogs {
    /// Opens `dialog`.
    pub(crate) fn open(self, dialog: Dialog) {
        let mut open = match dialog {
            Dialog::Buy => self.buy,
            Dialog::Clone => self.clone,
            Dialog::Share => self.share,
            Dialog::ClearSkips => self.clear_skips,
            Dialog::Delete => self.delete,
        };
        open.set(true);
    }
}

/// The sheet and every dialog, all closed.
pub(crate) fn use_deck_dialogs() -> DeckDialogs {
    DeckDialogs {
        sheet: use_signal(|| false),
        buy: use_signal(|| false),
        clone: use_signal(|| false),
        share: use_signal(|| false),
        clear_skips: use_signal(|| false),
        delete: use_signal(|| false),
    }
}

#[component]
pub(crate) fn MoreButtons(
    dialogs: DeckDialogs,
    /// The items that can be pressed; the rest show disabled.
    enabled: Vec<MoreItem>,
    /// The `TCGplayer` and `Card Kingdom` links, once the deck has loaded.
    buy_links: Option<(String, String)>,
    buy_body: &'static str,
    /// Whether the deck has a public link.
    shared: bool,
    share_body: &'static str,
    ondispatch: EventHandler<Action>,
) -> Element {
    let DeckDialogs {
        sheet: mut show_more_sheet,
        buy: show_buy_dialog,
        share: show_share_dialog,
        clear_skips: show_clear_skips_dialog,
        ..
    } = dialogs;

    let share_deck = use_callback(move |()| {
        show_more_sheet.set(false);
        ondispatch.call(Action::Share);
    });

    let copy_shared_link = use_callback(move |()| {
        show_more_sheet.set(false);
        ondispatch.call(Action::CopyLink);
    });

    let stop_sharing = use_callback(move |()| {
        show_more_sheet.set(false);
        ondispatch.call(Action::StopShare);
    });

    let clear_skips = use_callback(move |()| {
        show_more_sheet.set(false);
        ondispatch.call(Action::ClearSkips);
    });

    let (tcg_url, ck_url) = buy_links.map_or((None, None), |(tcg, ck)| (Some(tcg), Some(ck)));
    let buy_actions: Vec<DialogAction> =
        std::iter::once(DialogAction::Dismiss(deck_view::CLOSE.into()))
            .chain(tcg_url.map(|href| DialogAction::Link {
                label: deck_view::TCGPLAYER.into(),
                href,
            }))
            .chain(ck_url.map(|href| DialogAction::Link {
                label: deck_view::CARD_KINGDOM.into(),
                href,
            }))
            .collect();

    let share_actions = if shared {
        vec![
            DialogAction::Dismiss(deck_view::CLOSE.into()),
            DialogAction::Confirm {
                label: deck_view::COPY_LINK.into(),
                danger: false,
                on_confirm: copy_shared_link,
            },
            DialogAction::Confirm {
                label: deck_view::STOP_SHARE.into(),
                danger: true,
                on_confirm: stop_sharing,
            },
        ]
    } else {
        vec![
            DialogAction::Dismiss(deck_view::CLOSE.into()),
            DialogAction::Confirm {
                label: deck_view::CREATE_LINK.into(),
                danger: false,
                on_confirm: share_deck,
            },
        ]
    };

    // Import, Export, Buy and Clone close the sheet; Share, Clear skips and
    // Delete open their dialog over it.
    let press = move |item: MoreItem| {
        if matches!(
            item,
            MoreItem::ImportCards | MoreItem::ExportCards | MoreItem::BuyDeck | MoreItem::CloneDeck
        ) {
            let mut sheet = show_more_sheet;
            sheet.set(false);
        }
        ondispatch.call(Action::More(item));
    };

    rsx! {
        AlertDialog {
            open: show_buy_dialog,
            title: Dialog::Buy.title().to_string(),
            actions: buy_actions,
            "{buy_body}"
        }

        AlertDialog {
            open: show_clear_skips_dialog,
            title: Dialog::ClearSkips.title().to_string(),
            actions: vec![
                DialogAction::Dismiss(deck_view::CANCEL.into()),
                DialogAction::Confirm {
                    label: deck_view::CLEAR.into(),
                    danger: true,
                    on_confirm: clear_skips,
                },
            ],
            {deck_view::CLEAR_SKIPS_BODY}
        }

        // One dialog covers the whole share lifecycle (mirrors the Buy-deck
        // dialog). "Stop sharing" acts directly from here; the dialog itself
        // is the deliberate surface, and the description carries the caveat,
        // so no second confirmation dialog stacks on top.
        AlertDialog {
            open: show_share_dialog,
            title: Dialog::Share.title().to_string(),
            actions: share_actions,
            "{share_body}"
        }

        BottomSheet { open: show_more_sheet, title: deck_view::MORE_TITLE,
            for item in MoreItem::ALL {
                Button {
                    key: "{item.label()}",
                    disabled: !enabled.contains(&item),
                    danger: item.is_danger(),
                    onclick: move |_| press(item),
                    {item.label()}
                }
            }
        }
    }
}
