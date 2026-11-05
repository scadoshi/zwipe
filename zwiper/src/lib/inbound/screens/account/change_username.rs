//! Change username bottom sheet.

use components::{ActionDock, BottomSheet, DockAction, DockPlace, TextInput, icons};
use dioxus::prelude::*;
use zwipe_ui::screens::account::{Action, Field, Sheet};

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
                    label: Field::NewUsername.label(),
                    placeholder: Field::NewUsername.placeholder(),
                    error: username_error,
                }

                TextInput {
                    value: driver.inputs.get(sheet, Field::Password),
                    id: "password",
                    label: Field::Password.label(),
                    placeholder: Field::Password.placeholder(),
                    input_type: "password",
                    error: password_error,
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
