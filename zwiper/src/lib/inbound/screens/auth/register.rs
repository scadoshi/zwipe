//! Create account: the hero, then the four fields and `Create account` in
//! a panel headed with the screen's name, `Back to log in` as a chip along
//! its foot, then the featured flavor card.

use components::{Button, ButtonVariant, TOAST_NORMAL, TextInput, ToastOptions, Toasts, use_toast};
use dioxus::prelude::*;
use dioxus::router::Navigator;
use zwipe_client::ZwipeClient;
use zwipe_core::http::contracts::auth::HttpRegisterUser;
use zwipe_core::http::contracts::metrics::AnonymousEventKind;
use zwipe_ui::account::Password;
use zwipe_ui::screens::register::{self, Action, Destination, Effect, Field, State};
use zwipe_ui::text::sentence_case;

use crate::inbound::components::hint_host::HintTopic;
use crate::inbound::components::home_flavor::PublicFlavor;
use crate::inbound::components::home_hero::HomeHero;
use crate::inbound::components::screen_header::ScreenHeader;
use crate::inbound::components::telemetry::anonymous;
use crate::inbound::components::{info_button, ui_toast};
use crate::inbound::router::Router;
use crate::inbound::screens::auth::login::LoginPrefill;

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
                // Funnel: a validated submit is going to the server (success
                // or not, registration success itself lands in user_events).
                anonymous::record_anonymous_event(
                    self.client,
                    AnonymousEventKind::RegisterSubmitted,
                );
                let request = HttpRegisterUser::new(&username, &email, password.read());
                spawn(async move {
                    let client = self.client.peek().clone();
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
            Effect::Navigate(Destination::Previous) => self.navigator.go_back(),
            Effect::Navigate(Destination::Login) => {
                self.navigator.replace(Router::Login {});
            }
            Effect::Toast(toast) => ui_toast::raise(self.toast, toast),
        }
    }
}

/// Hands every change to `input` to the model as the action `edit` builds.
fn use_field_sync(driver: &Driver, input: Signal<String>, edit: fn(String) -> Action) {
    let driver = *driver;
    use_effect(move || driver.dispatch(edit(input())));
}

/// Registration form screen for creating new user accounts.
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

    // Funnel: register screen reached (once per mount; distinct-session
    // counting on the server dedupes revisits).
    let client = driver.client;
    use_hook(|| anonymous::record_anonymous_event(client, AnonymousEventKind::RegisterViewed));

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
        div { class: "screen",
            ScreenHeader { title: register::TITLE }
            div { class: "screen-content content-enter",
            HomeHero {}
            div { class: "container-sm text-center form-panel",
                div { class: "settings-list-header",
                    span { class: "settings-list-title", {register::TITLE} }
                }
                form { class: "flex-col", onsubmit: move |evt| evt.prevent_default(),
                    TextInput {
                        value: driver.username,
                        id: "username",
                        label: Field::Username.label(),
                        placeholder: Field::Username.placeholder(),
                        error: username_error.as_deref().map(sentence_case),
                    }
                    TextInput {
                        value: driver.email,
                        id: "email",
                        label: Field::Email.label(),
                        placeholder: Field::Email.placeholder(),
                        input_type: "email",
                        error: email_error.as_deref().map(sentence_case),
                    }
                    TextInput {
                        value: driver.password,
                        id: "password",
                        label: Field::Password.label(),
                        placeholder: Field::Password.placeholder(),
                        input_type: "password",
                        error: password_error.as_deref().map(sentence_case),
                        on_info: info_button::open_topic(HintTopic::Password),
                    }
                    TextInput {
                        value: driver.confirm_password,
                        id: "confirm_password",
                        label: Field::ConfirmPassword.label(),
                        placeholder: Field::ConfirmPassword.placeholder(),
                        input_type: "password",
                    }
                    // The form's one button, so Return in a field clicks it;
                    // the click handles the submit itself, never the page.
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
