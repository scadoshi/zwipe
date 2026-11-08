//! Register: the four fields as a panel headed with the screen's name,
//! under the hero and beside the featured flavor card on a wide window,
//! over it below the breakpoint, `Create account` full width under them, then `Back to
//! login` as a chip beneath. The password rules are the `?` beside
//! `Password`, as in the app. A profile created returns to Login in this
//! screen's place with the username filled in.
//!
//! What the screen knows and does is `zwipe_ui::screens::register`; this
//! file draws its state and performs its effects.

use components::{Button, ButtonVariant, TOAST_NORMAL, TextInput};
use dioxus::prelude::*;
use dioxus::router::Navigator;
use dioxus_primitives::toast::{ToastOptions, Toasts, use_toast};
use zwipe_client::ZwipeClient;
use zwipe_core::domain::user::{email, username};
use zwipe_core::http::contracts::auth::HttpRegisterUser;
use zwipe_core::http::contracts::metrics::AnonymousEventKind;
use zwipe_ui::account::Password;
use zwipe_ui::screens::register::{self, Action, Destination, Effect, Field, State};
use zwipe_ui::text::sentence_case;

use crate::inbound::chrome::hint_topic::{self, HintTopic};
use crate::inbound::chrome::toast::raise;
use crate::inbound::router::Route;
use crate::inbound::screens::auth::hero::{PublicFlavor, PublicHero};
use crate::inbound::screens::auth::login::LoginPrefill;
use crate::inbound::session::funnel;
use crate::inbound::shell::ScreenFrame;

/// Register's model with the handles that perform its effects. Copy, so
/// every handler and task holds its own.
#[derive(Clone, Copy)]
struct Driver {
    model: Signal<State>,
    username: Signal<String>,
    email: Signal<String>,
    password: Signal<String>,
    confirm_password: Signal<String>,
    navigator: Navigator,
    toast: Toasts,
    client: Signal<ZwipeClient>,
    login_prefill: Signal<Option<String>>,
}

impl Driver {
    /// Applies `action` to the model and performs what it returns.
    fn dispatch(self, action: Action) {
        let mut model = self.model;
        let effects = model.write().update(action);
        for effect in effects {
            self.perform(effect);
        }
    }

    fn perform(self, effect: Effect) {
        match effect {
            Effect::Register {
                ticket,
                username,
                email,
                password,
            } => {
                funnel::record(self.client, AnonymousEventKind::RegisterSubmitted);
                let request = HttpRegisterUser::new(&username, &email, password.read());
                spawn(async move {
                    let client = self.client.peek().clone();
                    // The same answer whether or not the email already has
                    // an account, so the screen says the same either way.
                    let reply = match client.register(request).await {
                        Ok(_) => Action::Registered { ticket },
                        Err(e) => {
                            tracing::warn!("register failed: {e}");
                            self.toast.error(
                                e.to_user_message(),
                                ToastOptions::default().duration(TOAST_NORMAL),
                            );
                            Action::Failed { ticket }
                        }
                    };
                    self.dispatch(reply);
                });
            }
            Effect::Prefill(username) => {
                let mut login_prefill = self.login_prefill;
                login_prefill.set(Some(username));
            }
            Effect::Navigate(Destination::Previous) => go_back_or_login(self.navigator),
            Effect::Navigate(Destination::Login) => {
                self.navigator.replace(Route::Login {});
            }
            Effect::Toast(toast) => raise(self.toast, &toast),
        }
    }
}

/// Back in the browser's history, or to Login when this page was opened
/// straight from its address.
pub(super) fn go_back_or_login(navigator: Navigator) {
    if navigator.can_go_back() {
        navigator.go_back();
    } else {
        navigator.replace(Route::Login {});
    }
}

/// Hands every change to `input` to the model as the action `edit` builds.
fn use_field_sync(driver: &Driver, input: Signal<String>, edit: fn(String) -> Action) {
    let driver = *driver;
    use_effect(move || driver.dispatch(edit(input())));
}

/// Create account. Each field starts showing its error line once it has
/// held text, or after a submit attempt, and rechecks on every change.
/// Return in any field runs `Create account`.
#[component]
pub fn Register() -> Element {
    let LoginPrefill(login_prefill) = use_context();
    let driver = Driver {
        model: use_signal(State::new),
        username: use_signal(String::new),
        email: use_signal(String::new),
        password: use_signal(String::new),
        confirm_password: use_signal(String::new),
        navigator: use_navigator(),
        toast: use_toast(),
        client: use_context(),
        login_prefill,
    };

    let client = driver.client;
    use_hook(|| funnel::record(client, AnonymousEventKind::RegisterViewed));

    use_field_sync(&driver, driver.username, Action::EditUsername);
    use_field_sync(&driver, driver.email, Action::EditEmail);
    use_field_sync(&driver, driver.password, |value| {
        Action::EditPassword(Password::new(value))
    });
    use_field_sync(&driver, driver.confirm_password, |value| {
        Action::EditConfirm(Password::new(value))
    });

    let model = driver.model.read();
    let is_loading = model.is_loading();
    let submit_label = model.submit_label();
    let username_error = model.error(Field::Username);
    let email_error = model.error(Field::Email);
    let password_error = model.error(Field::Password);
    drop(model);

    rsx! {
        ScreenFrame { title: register::TITLE,
            div { class: "screen-content content-enter",
                div { class: "web-auth web-page",
                    PublicHero {}
                    div { class: "web-auth-row",
                    div { class: "container-sm text-center web-form-panel",
                        div { class: "settings-list-header web-form-head",
                            span { class: "settings-list-title", {register::TITLE} }
                        }
                        form {
                            class: "flex-col",
                            onsubmit: move |event| {
                                event.prevent_default();
                                driver.dispatch(Action::Submit);
                            },
                            TextInput {
                                value: driver.username,
                                id: "username",
                                maxlength: username::MAX_LENGTH,
                                name: "username",
                                autocomplete: "username",
                                label: Field::Username.label(),
                                placeholder: Field::Username.placeholder(),
                                error: username_error.as_deref().map(sentence_case),
                            }
                            TextInput {
                                value: driver.email,
                                id: "email",
                                maxlength: email::MAX_LENGTH,
                                name: "email",
                                autocomplete: "email",
                                label: Field::Email.label(),
                                placeholder: Field::Email.placeholder(),
                                input_type: "email",
                                error: email_error.as_deref().map(sentence_case),
                            }
                            TextInput {
                                value: driver.password,
                                id: "password",
                                name: "new-password",
                                autocomplete: "new-password",
                                label: Field::Password.label(),
                                placeholder: Field::Password.placeholder(),
                                input_type: "password",
                                error: password_error.as_deref().map(sentence_case),
                                on_info: hint_topic::open_topic(HintTopic::Password),
                            }
                            TextInput {
                                value: driver.confirm_password,
                                id: "confirm_password",
                                name: "confirm-password",
                                autocomplete: "new-password",
                                label: Field::ConfirmPassword.label(),
                                placeholder: Field::ConfirmPassword.placeholder(),
                                input_type: "password",
                            }
                            // The form's one button, so Return in a field
                            // clicks it; the click submits itself, never the
                            // page.
                            div { class: "form-submit",
                                Button {
                                    disabled: is_loading,
                                    onclick: move |evt: MouseEvent| {
                                        evt.prevent_default();
                                        driver.dispatch(Action::Submit);
                                    },
                                    {submit_label}
                                }
                            }
                        }
                        div { class: "form-links",
                            Button {
                                variant: ButtonVariant::Action,
                                onclick: move |_| driver.dispatch(Action::Back),
                                {register::BACK}
                            }
                        }
                    }
                    PublicFlavor {}
                    }
                }
            }
        }
    }
}
