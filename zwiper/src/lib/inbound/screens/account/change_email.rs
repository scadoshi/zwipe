//! Change email bottom sheet.

use components::{ActionDock, BottomSheet, DockAction, DockPlace, TextInput, icons};
use dioxus::prelude::*;
use zwipe_ui::screens::account::{Action, Field, Sheet};
use zwipe_ui::text::sentence_case;

use super::driver::{Driver, use_field_sync};

/// Bottom sheet for updating the user's email address.
#[component]
pub fn ChangeEmailSheet(open: Signal<bool>) -> Element {
    let driver: Driver = use_context();
    let sheet = Sheet::Email;
    use_field_sync(&driver, sheet, Field::NewEmail);
    use_field_sync(&driver, sheet, Field::Password);

    let model = driver.model.read();
    let is_loading = model.is_saving(sheet);
    let save_label = model.save_label(sheet);
    let email_error = model.error(sheet, Field::NewEmail);
    let password_error = model.error(sheet, Field::Password);
    drop(model);

    rsx! {
        BottomSheet {
            open,
            title: sheet.title().to_string(),
            closable: true,

            div { class: "flex-col text-center",

                TextInput {
                    value: driver.inputs.get(sheet, Field::NewEmail),
                    id: "new_email",
                    label: Field::NewEmail.label(),
                    placeholder: Field::NewEmail.placeholder(),
                    error: email_error.as_deref().map(sentence_case),
                }

                TextInput {
                    value: driver.inputs.get(sheet, Field::Password),
                    id: "password",
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
