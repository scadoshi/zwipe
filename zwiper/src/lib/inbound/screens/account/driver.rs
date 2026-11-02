//! Account's model with the handles that perform its effects.

use components::{TOAST_LONG, ToastOptions, Toasts, use_toast};
use dioxus::prelude::*;
use dioxus::router::Navigator;
use zwipe_client::{ClientError, ZwipeClient};
use zwipe_core::domain::auth::models::secret::Secret;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::contracts::auth::{
    HttpChangeEmail, HttpChangePassword, HttpChangeUsername, HttpDeleteUser,
};
use zwipe_ui::generation::Ticket;
use zwipe_ui::screens::account::{
    Action, Destination, Dialog, Effect, Field, Password, Sheet, State, TICK, Typed,
};

use crate::inbound::components::auth::authed::{Authed, use_authed};
use crate::inbound::components::auth::ensure_session::EnsureFresh;
use crate::inbound::components::auth::signal_logout::SignalLogout;
use crate::inbound::components::telemetry::usage_buffer::UsageBuffer;
use crate::inbound::components::telemetry::vocabulary::{ProfileScreen, Screen, component, screen};
use crate::inbound::components::{hint_dialog, ui_toast};
use crate::inbound::router::Router;
use crate::outbound::open_url;

/// The request runners, one per telemetry screen the account reports under.
#[derive(Clone, Copy)]
struct Runners {
    main: Authed,
    username: Authed,
    email: Authed,
    password: Authed,
}

/// Whether each sheet, dialog and the More sheet is up.
#[derive(Clone, Copy)]
pub(crate) struct Overlays {
    pub(crate) hint: Signal<bool>,
    pub(crate) more: Signal<bool>,
    pub(crate) log_out: Signal<bool>,
    pub(crate) delete: Signal<bool>,
    pub(crate) confirm_password: Signal<bool>,
    pub(crate) username: Signal<bool>,
    pub(crate) email: Signal<bool>,
    pub(crate) password: Signal<bool>,
}

impl Overlays {
    /// The open flag for `sheet`.
    pub(crate) fn sheet(self, sheet: Sheet) -> Signal<bool> {
        match sheet {
            Sheet::Username => self.username,
            Sheet::Email => self.email,
            Sheet::Password => self.password,
        }
    }
}

/// What each sheet's fields hold, as the text inputs bind them.
#[derive(Clone, Copy)]
pub(crate) struct Inputs {
    new_username: Signal<String>,
    username_password: Signal<String>,
    new_email: Signal<String>,
    email_password: Signal<String>,
    current_password: Signal<String>,
    new_password: Signal<String>,
    confirm_password: Signal<String>,
}

impl Inputs {
    /// The input bound to `field` on `sheet`. A field not on the sheet gets the sheet's first.
    pub(crate) fn get(self, sheet: Sheet, field: Field) -> Signal<String> {
        match (sheet, field) {
            (Sheet::Username, Field::Password) => self.username_password,
            (Sheet::Username, _) => self.new_username,
            (Sheet::Email, Field::Password) => self.email_password,
            (Sheet::Email, _) => self.new_email,
            (Sheet::Password, Field::NewPassword) => self.new_password,
            (Sheet::Password, Field::ConfirmPassword) => self.confirm_password,
            (Sheet::Password, _) => self.current_password,
        }
    }

    /// Empties every field on `sheet`.
    fn clear(self, sheet: Sheet) {
        for field in sheet.fields() {
            self.get(sheet, *field).set(String::new());
        }
    }
}

/// Account's model with the handles that perform its effects. Copy, so every handler and task holds its own; the screen provides it as context to its sheets and dialogs.
#[derive(Clone, Copy)]
pub(crate) struct Driver {
    pub(crate) model: Signal<State>,
    pub(crate) overlays: Overlays,
    pub(crate) inputs: Inputs,
    navigator: Navigator,
    toast: Toasts,
    runners: Runners,
    usage_buffer: Signal<UsageBuffer>,
    session: Signal<Option<Session>>,
    client: Signal<ZwipeClient>,
}

/// Builds the screen's driver from the app's contexts. One call, in the screen component.
pub(crate) fn use_driver() -> Driver {
    Driver {
        model: use_signal(State::new),
        overlays: Overlays {
            hint: use_signal(|| false),
            more: use_signal(|| false),
            log_out: use_signal(|| false),
            delete: use_signal(|| false),
            confirm_password: use_signal(|| false),
            username: use_signal(|| false),
            email: use_signal(|| false),
            password: use_signal(|| false),
        },
        inputs: Inputs {
            new_username: use_signal(String::new),
            username_password: use_signal(String::new),
            new_email: use_signal(String::new),
            email_password: use_signal(String::new),
            current_password: use_signal(String::new),
            new_password: use_signal(String::new),
            confirm_password: use_signal(String::new),
        },
        navigator: use_navigator(),
        toast: use_toast(),
        runners: Runners {
            main: use_authed(Screen::Profile(ProfileScreen::Main)),
            username: use_authed(Screen::Profile(ProfileScreen::ChangeUsername)),
            email: use_authed(Screen::Profile(ProfileScreen::ChangeEmail)),
            password: use_authed(Screen::Profile(ProfileScreen::ChangePassword)),
        },
        usage_buffer: use_context(),
        session: use_context(),
        client: use_context(),
    }
}

/// Hands every change to `field`'s input on `sheet` to the model, so its line follows the typing.
pub(crate) fn use_field_sync(driver: &Driver, sheet: Sheet, field: Field) {
    let driver = *driver;
    let input = driver.inputs.get(sheet, field);
    use_effect(move || {
        let value = input();
        driver.dispatch(Action::Edit {
            sheet,
            field,
            value: Typed::new(value),
        });
    });
}

impl Driver {
    /// Applies `action` to the model and performs what it returns.
    pub(crate) fn dispatch(self, action: Action) {
        let mut model = self.model;
        let effects = model.write().update(action);
        for effect in effects {
            self.perform(effect);
        }
    }

    /// Deletes the account with `password` and waits for the reply, for the Delete account dialog's busy button. Returns whether the delete went through.
    pub(crate) async fn delete(self, password: String) -> bool {
        let mut model = self.model;
        let effects = model.write().update(Action::Delete {
            password: Password::new(password),
        });
        let mut deleted = false;
        for effect in effects {
            if let Effect::DeleteAccount { ticket, password } = effect {
                deleted = self.run_delete(ticket, password).await;
            } else {
                self.perform(effect);
            }
        }
        deleted
    }

    /// Sends the delete and feeds its reply back. Returns whether it went through.
    async fn run_delete(self, ticket: Ticket, password: Password) -> bool {
        let deleted = self
            .runners
            .main
            .run_at(
                component::DELETE_ACCOUNT_DIALOG,
                "delete_account",
                |c, s| async move {
                    c.delete_user(
                        HttpDeleteUser {
                            password: Secret::new(password.read()),
                        },
                        &s,
                    )
                    .await
                },
            )
            .await
            .is_some();
        self.dispatch(if deleted {
            Action::Deleted { ticket }
        } else {
            Action::Failed { ticket }
        });
        deleted
    }

    /// Performs one effect, feeding any reply back through [`Self::dispatch`].
    fn perform(self, effect: Effect) {
        let overlays = self.overlays;
        match effect {
            Effect::OfferHint(hint) => {
                hint_dialog::open_and_record_hint(
                    hint.key(),
                    self.session,
                    self.client,
                    overlays.hint,
                );
            }
            Effect::FetchUser(ticket) => self.fetch_user(ticket),
            Effect::CheckVerification(ticket) => self.check_verification(ticket),
            Effect::ResendVerification(ticket) => self.resend(ticket),
            Effect::Wait(ticket) => {
                spawn(async move {
                    tokio::time::sleep(TICK).await;
                    self.dispatch(Action::Tick { ticket });
                });
            }
            Effect::OpenSheet(sheet) => {
                self.inputs.clear(sheet);
                overlays.sheet(sheet).set(true);
            }
            Effect::CloseSheet(sheet) => {
                self.inputs.clear(sheet);
                overlays.sheet(sheet).set(false);
            }
            Effect::ChangeUsername {
                ticket,
                username,
                password,
            } => self.change_username(ticket, &username, &password),
            Effect::ChangeEmail {
                ticket,
                email,
                password,
            } => self.change_email(ticket, &email, &password),
            Effect::ChangePassword {
                ticket,
                current,
                new,
            } => self.change_password(ticket, &current, &new),
            Effect::DeleteAccount { ticket, password } => {
                spawn(async move {
                    self.run_delete(ticket, password).await;
                });
            }
            Effect::OpenDialog(dialog) => {
                let mut open = match dialog {
                    Dialog::ConfirmPassword => overlays.confirm_password,
                    Dialog::LogOut => overlays.log_out,
                    Dialog::DeleteAccount => overlays.delete,
                };
                open.set(true);
            }
            Effect::LogOut => self.session.logout(self.client),
            Effect::OpenUrl(url) => open_url::open(&url),
            Effect::Navigate(Destination::Previous) => self.navigator.go_back(),
            Effect::Navigate(Destination::PrivacyPolicy) => {
                self.navigator.push(Router::PrivacyPolicy {});
            }
            Effect::Navigate(Destination::Changelog) => {
                self.navigator.push(Router::Changelog {});
            }
            Effect::Toast(toast) => ui_toast::raise(self.toast, toast),
        }
    }

    /// Writes `apply` over the session's user, if a session is still up.
    fn store_user(self, apply: impl FnOnce(&mut Session)) {
        let mut session = self.session;
        let current = session.peek().clone();
        if let Some(mut current) = current {
            apply(&mut current);
            session.set(Some(current));
        }
    }

    /// Refreshes the user on opening so the verification is current without signing in again. A failure only logs.
    fn fetch_user(self, ticket: Ticket) {
        let Some(s) = self.session.peek().clone() else {
            return;
        };
        let client = self.client;
        spawn(async move {
            match client().fetch_user(&s).await {
                Ok(fresh_user) => {
                    self.store_user(|current| current.user = fresh_user.clone());
                    self.dispatch(Action::UserLoaded {
                        ticket,
                        user: Box::new(fresh_user),
                    });
                }
                Err(e) => {
                    tracing::warn!("account user fetch failed: {e}");
                    self.dispatch(Action::Failed { ticket });
                }
            }
        });
    }

    fn check_verification(self, ticket: Ticket) {
        let authed = self.runners.main;
        spawn(async move {
            let reply = authed
                .run_at(
                    component::EMAIL_VERIFICATION,
                    "check_verified",
                    |c, s| async move { c.fetch_user(&s).await },
                )
                .await;
            self.dispatch(match reply {
                Some(fresh_user) => {
                    self.store_user(|current| current.user = fresh_user.clone());
                    Action::UserLoaded {
                        ticket,
                        user: Box::new(fresh_user),
                    }
                }
                None => Action::Failed { ticket },
            });
        });
    }

    /// Sends the verification email. It stays off the runner: a 429 is an info toast that keeps the cooldown, and every other failure is a long error toast.
    fn resend(self, ticket: Ticket) {
        let Self {
            session,
            client,
            toast,
            usage_buffer,
            ..
        } = self;
        let failed = move |e: &ClientError| {
            usage_buffer.peek().report_error(
                screen::PROFILE,
                component::EMAIL_VERIFICATION,
                "resend_verification",
                e,
            );
            toast.error(
                e.to_user_message(),
                ToastOptions::default().duration(TOAST_LONG),
            );
            self.dispatch(Action::Failed { ticket });
        };
        spawn(async move {
            let s = match session.ensure_fresh(client).await {
                Ok(s) => s,
                Err(e) => {
                    failed(&e);
                    return;
                }
            };
            match client().resend_verification(&s).await {
                Ok(()) => self.dispatch(Action::Resent { ticket }),
                Err(ClientError::TooManyRequests(_)) => {
                    self.dispatch(Action::ResendLimited { ticket });
                }
                Err(e) => failed(&e),
            }
        });
    }

    fn change_username(self, ticket: Ticket, username: &str, password: &Password) {
        tracing::info!("change username to {username}");
        let request = HttpChangeUsername::new(username, password.read());
        let authed = self.runners.username;
        spawn(async move {
            let reply = authed
                .run("change_username", |c, s| async move {
                    c.change_username(request, &s).await
                })
                .await;
            self.dispatch(match reply {
                Some(updated_user) => {
                    let username = updated_user.username;
                    self.store_user(|current| current.user.username = username.clone());
                    Action::UsernameChanged { ticket, username }
                }
                None => Action::Failed { ticket },
            });
        });
    }

    fn change_email(self, ticket: Ticket, email: &str, password: &Password) {
        tracing::info!("change email");
        let request = HttpChangeEmail::new(email, password.read());
        let authed = self.runners.email;
        spawn(async move {
            let reply = authed
                .run("change_email", |c, s| async move {
                    c.change_email(request, &s).await
                })
                .await;
            self.dispatch(match reply {
                Some(updated_user) => {
                    self.store_user(|current| current.user = updated_user.clone());
                    Action::EmailChanged {
                        ticket,
                        user: Box::new(updated_user),
                    }
                }
                None => Action::Failed { ticket },
            });
        });
    }

    fn change_password(self, ticket: Ticket, current: &Password, new: &Password) {
        tracing::info!("change password");
        let request = HttpChangePassword::new(current.read(), new.read());
        let authed = self.runners.password;
        spawn(async move {
            let changed = authed
                .run("change_password", |c, s| async move {
                    c.change_password(request, &s).await
                })
                .await
                .is_some();
            self.dispatch(if changed {
                Action::PasswordChanged { ticket }
            } else {
                Action::Failed { ticket }
            });
        });
    }
}
