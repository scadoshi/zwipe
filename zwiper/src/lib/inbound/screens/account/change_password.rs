//! Change password bottom sheet.

use components::{
    ActionDock, AlertDialog, BottomSheet, Button, ButtonVariant, DialogAction, DockAction,
    DockPlace, TextInput, icons,
};
use dioxus::prelude::*;
use zwipe_ui::screens::account::{Action, CANCEL, Dialog, FORGOT_PASSWORD, Field, Sheet};
use zwipe_ui::text::sentence_case;

use super::driver::{Driver, use_field_sync};
use crate::inbound::components::hint_host::HintTopic;
use crate::inbound::components::info_button;

/// Bottom sheet for updating the user's password. The current password is
/// never judged here, so one that predates the policy can still be changed.
#[component]
pub fn ChangePasswordSheet(open: Signal<bool>) -> Element {
    let driver: Driver = use_context();
    let sheet = Sheet::Password;
    for field in sheet.fields() {
        use_field_sync(&driver, sheet, *field);
    }
    let confirm = use_callback(move |()| driver.dispatch(Action::ConfirmPassword));

    let model = driver.model.read();
    let is_loading = model.is_saving(sheet);
    let save_label = model.save_label(sheet);
    let password_error = model.error(sheet, Field::NewPassword);
    drop(model);

    let dialog = Dialog::ConfirmPassword;
    rsx! {
        BottomSheet {
            open,
            title: sheet.title().to_string(),
            closable: true,
            // The way out for a user who no longer knows the current password.
            trailing: rsx! {
                Button {
                    variant: ButtonVariant::Action,
                    class: "page-header-corner",
                    disabled: is_loading,
                    onclick: move |_| driver.dispatch(Action::ForgotPassword),
                    {FORGOT_PASSWORD}
                }
            },

            div { class: "flex-col text-center",

                TextInput {
                    value: driver.inputs.get(sheet, Field::CurrentPassword),
                    id: "current_password",
                    name: "current-password",
                    autocomplete: "current-password",
                    label: Field::CurrentPassword.label(),
                    placeholder: Field::CurrentPassword.placeholder(),
                    input_type: "password",
                }

                TextInput {
                    value: driver.inputs.get(sheet, Field::NewPassword),
                    id: "new_password",
                    name: "new-password",
                    autocomplete: "new-password",
                    label: Field::NewPassword.label(),
                    placeholder: Field::NewPassword.placeholder(),
                    input_type: "password",
                    error: password_error.as_deref().map(sentence_case),
                    on_info: info_button::open_topic(HintTopic::Password),
                }

                TextInput {
                    value: driver.inputs.get(sheet, Field::ConfirmPassword),
                    id: "confirm_password",
                    name: "confirm-password",
                    autocomplete: "new-password",
                    label: Field::ConfirmPassword.label(),
                    placeholder: Field::ConfirmPassword.placeholder(),
                    input_type: "password",
                }
            }
            // Save floats at the sheet's bottom right.
            ActionDock { place: DockPlace::Sheet,
                DockAction {
                    label: save_label,
                    icon: icons::CHECK,
                    disabled: is_loading,
                    onclick: move |()| driver.dispatch(Action::Save(sheet)),
                }
            }
        }

        AlertDialog {
            open: driver.overlays.confirm_password,
            title: dialog.title(),
            actions: vec![
                DialogAction::Dismiss(CANCEL.into()),
                DialogAction::Confirm {
                    label: dialog.confirm().into(),
                    danger: false,
                    on_confirm: confirm,
                },
            ],
            {dialog.body()}
        }
    }
}
