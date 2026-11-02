//! User login screen.

use components::{
    ActionBar, Button, ButtonVariant, TOAST_NORMAL, TextInput, ThemeConfig, ToastOptions, use_toast,
};
use dioxus::prelude::*;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::user::preferences::PipStyle;
use zwipe_core::http::contracts::auth::HttpAuthenticateUser;
use zwipe_ui::account::{self, LoginGate};

use crate::inbound::components::home_hero::HomeHero;
use crate::inbound::components::screen_header::ScreenHeader;
use crate::inbound::components::ui_toast;
use crate::inbound::router::Router;
use crate::outbound::app_icon;
use crate::outbound::session::Persist;

/// An identifier for the login form to fill in once, set by a screen that
/// sends the user here (registration). Provided at the root.
#[derive(Debug, Clone, Copy)]
pub struct LoginPrefill(pub Signal<Option<String>>);

/// Login form screen for user authentication.
#[component]
pub fn Login() -> Element {
    let navigator = use_navigator();

    let mut session: Signal<Option<Session>> = use_context();
    let auth_client: Signal<ZwipeClient> = use_context();
    let mut pip_style: Signal<PipStyle> = use_context();
    let mut theme: Signal<ThemeConfig> = use_context();

    let mut username_or_email = use_signal(String::new);
    let mut password = use_signal(String::new);

    // Takes the prefill whenever it is set, including while this screen sits
    // mounted under Register.
    let LoginPrefill(mut prefill) = use_context();
    use_effect(move || {
        if let Some(identifier) = prefill() {
            username_or_email.set(identifier);
            password.set(String::new());
            prefill.set(None);
        }
    });

    let mut is_loading = use_signal(|| false);
    let toast = use_toast();

    // Login applies no registration policy to what's typed; see
    // `account::login_gate`. The server is the only judge of a credential.
    let mut attempt_submit = move || {
        match account::login_gate(*is_loading.peek(), &username_or_email(), &password()) {
            LoginGate::Busy => return,
            LoginGate::MissingInputs => {
                ui_toast::raise(toast, account::missing_inputs_toast());
                return;
            }
            LoginGate::Send => {}
        }
        is_loading.set(true);
        let request = HttpAuthenticateUser::new(&username_or_email(), &password());
        spawn(async move {
            match auth_client().authenticate_user(request).await {
                Ok(new_session) => {
                    new_session.infallible_save();
                    // Apply theme from preferences
                    let picked = components::theme_from_preferences(&new_session.preferences);
                    app_icon::follow(&picked);
                    theme.set(picked);
                    pip_style.set(new_session.preferences.pip_style);
                    session.set(Some(new_session));
                    // The password has done its job; the signal outlives the
                    // screen in the router cache.
                    password.set(String::new());
                    is_loading.set(false);
                    navigator.push(Router::Home {});
                }
                Err(e) => {
                    tracing::warn!("login failed: {e}");
                    toast.error(
                        e.to_user_message(),
                        ToastOptions::default().duration(TOAST_NORMAL),
                    );
                    is_loading.set(false);
                }
            }
        });
    };

    rsx! {
        div { class: "screen",
            ScreenHeader { title: "Login" }
            div { class: "screen-content centered content-enter",
            HomeHero {}
            div { class : "container-sm text-center",
                form { class: "flex-col",
                    TextInput {
                        value: username_or_email,
                        id: "identity",
                        label: "Username or email",
                        placeholder: "Username or email",
                    }
                    TextInput {
                        value: password,
                        id: "password",
                        label: "Password",
                        placeholder: "Password",
                        input_type: "password",
                    }
                }
            }
        }
        ActionBar {
            Button {
                variant: ButtonVariant::Action,
                disabled: is_loading(),
                onclick: move |_| attempt_submit(),
                if is_loading() { "Logging in..." } else { "Log in" }
            },
            Button {
                variant: ButtonVariant::Action,
                disabled: is_loading(),
                onclick: move |_| {
                navigator.push(Router::Register {});
                },
                "Create profile"
            }
            Button {
                variant: ButtonVariant::Action,
                disabled: is_loading(),
                onclick: move |_| { navigator.push(Router::ForgotPassword {}); },
                "Forgot password"
            }
        }
    }
    }
}
