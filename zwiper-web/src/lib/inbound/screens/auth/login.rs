//! Login: the form as a panel headed with the screen's name, under the hero
//! and beside the featured flavor card on a wide window, over it below the
//! breakpoint: the two fields, `Log in` full width under them, then
//! `Create account` and `Forgot password` as chips in one centered row.
//!
//! What the screen knows and does is `zwipe_ui::screens::login`; this file
//! draws its state and performs its effects.

use components::{Button, ButtonVariant, TOAST_NORMAL, TextInput, ThemeConfig};
use dioxus::prelude::*;
use dioxus::router::Navigator;
use dioxus_primitives::toast::{ToastOptions, Toasts, use_toast};
use zwipe_client::ZwipeClient;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::user::models::settings::PipStyle;
use zwipe_core::http::contracts::auth::HttpAuthenticateUser;
use zwipe_ui::account::Password;
use zwipe_ui::screens::login::{self, Action, Destination, Effect, Field, State};
use zwipe_ui::telemetry::vocabulary::{AuthScreen, Screen};

use crate::inbound::chrome::toast::raise;
use crate::inbound::router::Route;
use crate::inbound::screens::auth::hero::{PublicFlavor, PublicHero};
use crate::inbound::session::return_to;
use crate::inbound::shell::ScreenFrame;
use crate::inbound::telemetry;
use crate::outbound::session_store::SessionStore;
use crate::outbound::theme_store::ThemeStore;

/// An identifier for the login form to fill in once, set by a screen that
/// sends the user here (Create account's username, Forgot password's
/// email). Provided at the root, so it works while Login sits under them in
/// the history.
#[derive(Debug, Clone, Copy)]
pub struct LoginPrefill(pub Signal<Option<String>>);

/// Login's model with the handles that perform its effects. Copy, so every
/// handler and task holds its own.
#[derive(Clone, Copy)]
struct Driver {
    model: Signal<State>,
    identifier: Signal<String>,
    password: Signal<String>,
    navigator: Navigator,
    toast: Toasts,
    client: Signal<ZwipeClient>,
    session: Signal<Option<Session>>,
    pip_style: Signal<PipStyle>,
    theme: Signal<ThemeConfig>,
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
        let Self {
            mut identifier,
            mut password,
            mut session,
            mut pip_style,
            mut theme,
            ..
        } = self;
        match effect {
            Effect::SignIn {
                ticket,
                identifier,
                password,
            } => {
                let request = HttpAuthenticateUser::new(&identifier, password.read());
                spawn(async move {
                    let client = self.client.peek().clone();
                    let reply = match client.login_cookie(request).await {
                        Ok(new_session) => Action::LoggedIn {
                            ticket,
                            session: Box::new(new_session),
                        },
                        Err(e) => {
                            tracing::warn!("login failed: {e}");
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
            Effect::SetIdentifier(value) => identifier.set(value),
            Effect::ClearPassword => password.set(String::new()),
            Effect::SignedIn(new_session) => {
                SessionStore::browser().infallible_save(&new_session);
                let profile = components::theme_from_settings(&new_session.settings);
                ThemeStore::browser().save_synced(&profile);
                theme.set(profile);
                pip_style.set(new_session.settings.pip_style);
                session.set(Some(*new_session));
            }
            Effect::Navigate(Destination::Home) => {
                self.navigator
                    .push(return_to::take().unwrap_or(Route::Home {}));
            }
            Effect::Navigate(to) => {
                self.navigator.push(route(to));
            }
            Effect::Toast(toast) => raise(self.toast, &toast),
        }
    }
}

/// The web's route for each place Login leads.
fn route(destination: Destination) -> Route {
    match destination {
        Destination::Home => Route::Home {},
        Destination::Register => Route::Register {},
        Destination::ForgotPassword => Route::ForgotPassword {},
    }
}

/// The login screen. Return in either field runs `Log in`.
#[component]
pub fn Login() -> Element {
    telemetry::use_telemetry_screen(Screen::Auth(AuthScreen::Login));
    let driver = Driver {
        model: use_signal(State::new),
        identifier: use_signal(String::new),
        password: use_signal(String::new),
        navigator: use_navigator(),
        toast: use_toast(),
        client: use_context(),
        session: use_context(),
        pip_style: use_context(),
        theme: use_context(),
    };

    use_effect(move || {
        let value = (driver.identifier)();
        driver.dispatch(Action::EditIdentifier(value));
    });
    use_effect(move || {
        let value = (driver.password)();
        driver.dispatch(Action::EditPassword(Password::new(value)));
    });

    // Takes the prefill whenever it is set, including while this screen sits
    // mounted under Register or Forgot password.
    let LoginPrefill(mut prefill) = use_context();
    use_effect(move || {
        if let Some(identifier) = prefill() {
            driver.dispatch(Action::Prefill(identifier));
            prefill.set(None);
        }
    });

    let model = driver.model.read();
    let is_loading = model.is_loading();
    let submit_label = model.submit_label();
    drop(model);

    rsx! {
        ScreenFrame { title: login::TITLE,
            div { class: "screen-content centered content-enter",
                div { class: "web-auth web-page",
                    PublicHero {}
                    div { class: "web-auth-row",
                    div { class: "container-sm text-center web-form-panel",
                        div { class: "settings-list-header web-form-head",
                            span { class: "settings-list-title", {login::TITLE} }
                        }
                        form {
                            class: "flex-col",
                            onsubmit: move |event| {
                                event.prevent_default();
                                driver.dispatch(Action::Submit);
                            },
                            TextInput {
                                value: driver.identifier,
                                id: "identity",
                                name: "username",
                                autocomplete: "username",
                                label: Field::Identifier.label(),
                            }
                            TextInput {
                                value: driver.password,
                                id: "password",
                                name: "password",
                                autocomplete: "current-password",
                                label: Field::Password.label(),
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
                                onclick: move |_| driver.dispatch(Action::CreateProfile),
                                {login::CREATE_ACCOUNT}
                            }
                            Button {
                                variant: ButtonVariant::Action,
                                onclick: move |_| driver.dispatch(Action::ForgotPassword),
                                {login::FORGOT_PASSWORD}
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_destination_has_its_route() {
        assert_eq!(route(Destination::Home), Route::Home {});
        assert_eq!(route(Destination::Register), Route::Register {});
        assert_eq!(route(Destination::ForgotPassword), Route::ForgotPassword {});
    }
}
