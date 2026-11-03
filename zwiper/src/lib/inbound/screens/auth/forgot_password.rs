//! Forgot password screen.

use components::{
    Button, ButtonVariant, Panel, TOAST_NORMAL, TextInput, ToastOptions, Toasts, use_toast,
};
use dioxus::prelude::*;
use dioxus::router::Navigator;
use zwipe_client::ZwipeClient;
use zwipe_core::http::contracts::auth::HttpRequestPasswordReset;
use zwipe_ui::screens::forgot_password::{
    self, Action, Destination, EMAIL, Effect, SEND, SENT_BODY, SENT_TITLE, State,
};

use crate::inbound::components::home_hero::HomeHero;
use crate::inbound::components::screen_header::ScreenHeader;

/// Forgot password's model with the handles that perform its effects. Copy,
/// so every handler and task holds its own.
#[derive(Clone, Copy)]
struct Driver {
    model: Signal<State>,
    email: Signal<String>,
    navigator: Navigator,
    toast: Toasts,
    client: Signal<ZwipeClient>,
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
            Effect::RequestReset { ticket, email } => {
                let request = HttpRequestPasswordReset::new(&email);
                spawn(async move {
                    let client = self.client.peek().clone();
                    let reply = match client.request_password_reset(request).await {
                        Ok(()) => Action::Sent { ticket },
                        Err(e) => {
                            tracing::warn!("password reset request failed: {e}");
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
            Effect::Navigate(Destination::Previous) => self.navigator.go_back(),
        }
    }
}

/// Forgot password screen for initiating a password reset.
#[component]
pub fn ForgotPassword() -> Element {
    let driver = Driver {
        model: use_signal(State::new),
        email: use_signal(String::new),
        navigator: use_navigator(),
        toast: use_toast(),
        client: use_context(),
    };

    use_effect(move || {
        let value = (driver.email)();
        driver.dispatch(Action::EditEmail(value));
    });

    let model = driver.model.read();
    let is_loading = model.is_loading();
    let is_sent = model.is_sent();
    let email_error = model.error();
    drop(model);

    rsx! {
        div { class: "screen",
            ScreenHeader { title: forgot_password::TITLE }
            div { class: "screen-content centered content-enter",
                HomeHero {}
                if is_sent {
                    div { class: "outcome-success",
                        Panel { title: SENT_TITLE, centered: true,
                            p { {SENT_BODY} }
                        }
                    }
                } else {
                    div { class: "container-sm text-center",
                        form { class: "flex-col",
                            TextInput {
                                value: driver.email,
                                id: "email",
                                label: EMAIL,
                                placeholder: EMAIL,
                                error: email_error,
                            }
                            if is_loading {
                                div { class: "spinner" }
                            }
                        }
                        if !is_sent {
                            div { class: "form-submit",
                                Button {
                                    variant: ButtonVariant::Action,
                                    onclick: move |_| driver.dispatch(Action::Submit),
                                    {SEND}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
