//! Login: the hero, then the two fields and `Log in` in a panel headed
//! with the screen's name, `Create profile` and `Forgot password` as chips
//! along its foot, then the featured flavor card.

use components::{
    Button, ButtonVariant, TOAST_NORMAL, TextInput, ThemeConfig, ToastOptions, Toasts, use_toast,
};
use dioxus::prelude::*;
use dioxus::router::Navigator;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::user::settings::PipColor;
use zwipe_core::http::contracts::auth::HttpAuthenticateUser;
use zwipe_ui::account::Password;
use zwipe_ui::screens::login::{self, Action, Destination, Effect, Field, State};

use crate::inbound::components::home_flavor::PublicFlavor;
use crate::inbound::components::home_hero::HomeHero;
use crate::inbound::components::screen_header::ScreenHeader;
use crate::inbound::components::ui_toast;
use crate::inbound::router::Router;
use crate::outbound::app_icon;
use crate::outbound::session::Persist;

/// An identifier for the login form to fill in once, set by a screen that
/// sends the user here (Create profile's username, Forgot password's email).
/// Provided at the root.
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
    pip_style: Signal<PipColor>,
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
                    let reply = match client.authenticate_user(request).await {
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
                new_session.infallible_save();
                let picked = components::theme_from_settings(&new_session.settings);
                app_icon::follow(&picked);
                theme.set(picked);
                pip_style.set(new_session.settings.pip_style);
                session.set(Some(*new_session));
            }
            Effect::Navigate(to) => {
                self.navigator.push(route(to));
            }
            Effect::Toast(toast) => ui_toast::raise(self.toast, toast),
        }
    }
}

/// The app's route for each place Login leads.
fn route(destination: Destination) -> Router {
    match destination {
        Destination::Home => Router::Home {},
        Destination::Register => Router::Register {},
        Destination::ForgotPassword => Router::ForgotPassword {},
    }
}

/// Login form screen for user authentication.
#[component]
pub fn Login() -> Element {
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
    // mounted under Register.
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
        div { class: "screen",
            ScreenHeader { title: login::TITLE }
            div { class: "screen-content content-enter",
            HomeHero {}
            div { class: "container-sm text-center form-panel",
                div { class: "settings-list-header",
                    span { class: "settings-list-title", {login::TITLE} }
                }
                form { class: "flex-col", onsubmit: move |evt| evt.prevent_default(),
                    TextInput {
                        value: driver.identifier,
                        id: "identity",
                        label: Field::Identifier.label(),
                        placeholder: Field::Identifier.placeholder(),
                    }
                    TextInput {
                        value: driver.password,
                        id: "password",
                        label: Field::Password.label(),
                        placeholder: Field::Password.placeholder(),
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
                        onclick: move |_| driver.dispatch(Action::CreateProfile),
                        {login::CREATE_PROFILE}
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_destination_has_its_route() {
        assert_eq!(route(Destination::Home), Router::Home {});
        assert_eq!(route(Destination::Register), Router::Register {});
        assert_eq!(
            route(Destination::ForgotPassword),
            Router::ForgotPassword {}
        );
    }
}
