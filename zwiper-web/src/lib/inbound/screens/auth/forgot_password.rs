//! Forgot password: the email field as a panel headed with the screen's
//! name, under the hero and beside the featured flavor card on a wide
//! window, over it below the breakpoint, `Send reset link` full width under it, then `Back to
//! login` and `Create account` as chips beneath. A sent link returns to
//! Login in this screen's place with the email filled in and says a link is
//! on its way.
//!
//! The reset itself happens on the website, from the emailed link. What the
//! screen knows and does is `zwipe_ui::screens::forgot_password`; this file
//! draws its state and performs its effects.

use components::{Button, ButtonVariant, TOAST_NORMAL, TextInput};
use dioxus::prelude::*;
use dioxus::router::Navigator;
use dioxus_primitives::toast::{ToastOptions, Toasts, use_toast};
use zwipe_client::ZwipeClient;
use zwipe_core::http::contracts::auth::HttpRequestPasswordReset;
use zwipe_ui::screens::forgot_password::{
    self, Action, BACK, CREATE_ACCOUNT, Destination, EMAIL, Effect, SEND, State,
};
use zwipe_ui::telemetry::vocabulary::{AuthScreen, Screen};
use zwipe_ui::text::sentence_case;

use crate::inbound::chrome::toast::raise;
use crate::inbound::router::Route;
use crate::inbound::screens::auth::hero::{PublicFlavor, PublicHero};
use crate::inbound::screens::auth::login::LoginPrefill;
use crate::inbound::screens::auth::register::go_back_or_login;
use crate::inbound::shell::ScreenFrame;
use crate::inbound::telemetry;

/// Forgot password's model with the handles that perform its effects. Copy,
/// so every handler and task holds its own.
#[derive(Clone, Copy)]
struct Driver {
    model: Signal<State>,
    email: Signal<String>,
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
            Effect::RequestReset { ticket, email } => {
                let request = HttpRequestPasswordReset::new(&email);
                spawn(async move {
                    let client = self.client.peek().clone();
                    // The answer never says whether the address has an
                    // account.
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
            Effect::Prefill(email) => {
                let mut login_prefill = self.login_prefill;
                login_prefill.set(Some(email));
            }
            Effect::Navigate(Destination::Previous) => go_back_or_login(self.navigator),
            Effect::Navigate(Destination::Login) => {
                self.navigator.replace(Route::Login {});
            }
            Effect::Navigate(Destination::Register) => {
                self.navigator.push(Route::Register {});
            }
            Effect::Toast(toast) => raise(self.toast, &toast),
        }
    }
}

/// Reset password: asks for an email, then leaves for Login saying a link
/// is on its way. Return in the field runs `Send reset link`.
#[component]
pub fn ForgotPassword() -> Element {
    telemetry::use_telemetry_screen(Screen::Auth(AuthScreen::ForgotPassword));
    let LoginPrefill(login_prefill) = use_context();
    let driver = Driver {
        model: use_signal(State::new),
        email: use_signal(String::new),
        navigator: use_navigator(),
        toast: use_toast(),
        client: use_context(),
        login_prefill,
    };

    use_effect(move || {
        let value = (driver.email)();
        driver.dispatch(Action::EditEmail(value));
    });

    let model = driver.model.read();
    let is_loading = model.is_loading();
    let submit_held = model.submit_held();
    let email_error = model.error();
    drop(model);

    rsx! {
        ScreenFrame { title: forgot_password::TITLE,
            div { class: "screen-content centered content-enter",
                div { class: "web-auth web-page",
                    PublicHero {}
                    div { class: "web-auth-row",
                    div { class: "container-sm text-center web-form-panel",
                        div { class: "settings-list-header web-form-head",
                            span { class: "settings-list-title", {forgot_password::TITLE} }
                        }
                        form {
                            class: "flex-col",
                            onsubmit: move |event| {
                                event.prevent_default();
                                driver.dispatch(Action::Submit);
                            },
                            TextInput {
                                value: driver.email,
                                id: "email",
                                label: EMAIL,
                                placeholder: EMAIL,
                                input_type: "email",
                                name: "email",
                                autocomplete: "email",
                                error: email_error.as_deref().map(sentence_case),
                            }
                            if is_loading {
                                div { class: "spinner" }
                            }
                            // The form's one button, so Return in the field
                            // clicks it; the click submits itself, never the
                            // page.
                            div { class: "form-submit",
                                Button {
                                    disabled: is_loading || submit_held,
                                    onclick: move |evt: MouseEvent| {
                                        evt.prevent_default();
                                        driver.dispatch(Action::Submit);
                                    },
                                    {SEND}
                                }
                            }
                        }
                        div { class: "form-links",
                            Button {
                                variant: ButtonVariant::Action,
                                onclick: move |_| driver.dispatch(Action::Back),
                                {BACK}
                            }
                            Button {
                                variant: ButtonVariant::Action,
                                onclick: move |_| driver.dispatch(Action::CreateProfile),
                                {CREATE_ACCOUNT}
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
