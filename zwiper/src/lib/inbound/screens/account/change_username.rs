//! Change username bottom sheet.

use components::{ActionDock, BottomSheet, DockAction, DockPlace, TextInput, icons};
use dioxus::prelude::*;
use zwipe_core::domain::user::username;
use zwipe_ui::screens::account::{Action, Field, Sheet};
use zwipe_ui::text::sentence_case;

use super::driver::{Driver, use_field_sync};

/// Bottom sheet for updating the user's username.
#[component]
pub fn ChangeUsernameSheet(open: Signal<bool>) -> Element {
    let driver: Driver = use_context();
    let sheet = Sheet::Username;
    use_field_sync(&driver, sheet, Field::NewUsername);
    use_field_sync(&driver, sheet, Field::Password);

    let model = driver.model.read();
    let is_loading = model.is_saving(sheet);
    let save_label = model.save_label(sheet);
    let username_error = model.error(sheet, Field::NewUsername);
    let password_error = model.error(sheet, Field::Password);
    drop(model);

    rsx! {
        BottomSheet {
            open,
            title: sheet.title().to_string(),
            closable: true,

            div { class: "flex-col text-center",

                TextInput {
                    value: driver.inputs.get(sheet, Field::NewUsername),
                    id: "new_username",
                    name: "username",
                    autocomplete: "username",
                    maxlength: username::MAX_LENGTH,
                    label: Field::NewUsername.label(),
                    placeholder: Field::NewUsername.placeholder(),
                    error: username_error.as_deref().map(sentence_case),
                }

                TextInput {
                    value: driver.inputs.get(sheet, Field::Password),
                    id: "password",
                    name: "password",
                    autocomplete: "current-password",
                    label: Field::Password.label(),
                    placeholder: Field::Password.placeholder(),
                    input_type: "password",
                    error: password_error.as_deref().map(sentence_case),
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
    }
}
