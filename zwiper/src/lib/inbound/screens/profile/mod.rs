pub mod change_email;
pub mod change_password;
pub mod change_username;

use crate::inbound::components::alert_dialog::{
    AlertDialogAction, AlertDialogActions, AlertDialogCancel, AlertDialogContent,
    AlertDialogDescription, AlertDialogRoot, AlertDialogTitle,
};
use crate::{
    inbound::{
        components::auth::{bouncer::Bouncer, signal_logout::SignalLogout},
        router::Router,
    },
    outbound::client::ZwipeClient,
};
use dioxus::prelude::*;
use zwipe::domain::auth::models::session::Session;

#[component]
pub fn Profile() -> Element {
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();

    let mut show_logout_dialog = use_signal(|| false);

    let navigator = use_navigator();

    rsx! {
        Bouncer {
            div { class: "page-header",
                h2 { "profile" }
            }

            div { class: "fixed top-0 left-0 h-screen flex flex-col items-center overflow-y-auto",
                style: "width: 100vw; justify-content: center; padding-top: 4rem;",
                if let Some(sesh) = session().as_ref() {
                    div { class : "container-sm",

                        div { class : "flex items-center flex-between mb-4 gap-2",
                            div { class : "flex-1",
                                label { class: "label", "username" }
                                p { class: "text-base font-light mb-1",
                                    { sesh.user.username.to_string() }
                                }
                            }
                        }

                        div { class : "flex items-center flex-between mb-4 gap-2",
                            div { class : "flex-1",
                                label { class: "label", "email" }
                                p { class: "text-base font-light mb-1",
                                    { sesh.user.email.to_string() }
                                }
                            }
                        }

                        div { class : "flex items-center flex-between mb-4 gap-2",
                            div { class : "flex-1",
                                label { class: "label", "password" }
                                p { class: "text-base font-light mb-1", "•••••••" }
                            }
                        }

                    }
                }
            }

            div { class: "util-bar",
                button {
                    class: "util-btn",
                    onclick: move |_| navigator.go_back(),
                    "back"
                }
                button {
                    class: "util-btn",
                    onclick: move |_| show_logout_dialog.set(true),
                    "logout"
                }
                button {
                    class: "util-btn",
                    onclick: move |_| {
                        navigator.push(Router::ChangeUsername {});
                    },
                    "change username"
                }
                button {
                    class: "util-btn",
                    onclick: move |_| {
                        navigator.push(Router::ChangeEmail {});
                    },
                    "change email"
                }
                button {
                    class: "util-btn",
                    onclick: move |_| {
                        navigator.push(Router::ChangePassword {});
                    },
                    "change password"
                }
           }

            AlertDialogRoot {
                open: show_logout_dialog(),
                on_open_change: move |open| show_logout_dialog.set(open),
                AlertDialogContent {
                    AlertDialogTitle { "logout" }
                    AlertDialogDescription { "are you sure you want to logout?" }
                    AlertDialogActions {
                        AlertDialogCancel {
                            on_click: move |_| show_logout_dialog.set(false),
                            "cancel"
                        }
                        AlertDialogAction {
                            on_click: move |_| session.logout(client),
                            "logout"
                        }
                    }
                }
            }
        }
    }
}
