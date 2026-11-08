//! Forgot password: an email field with Send reset link under it, then Back to log in and Create account. The reset itself happens on the website, from the emailed link.
//!
//! The client keeps a [`State`] while the screen is up and hands every keystroke in as [`Action::EditEmail`]. The check is [`account::reset_email_error`], presence only: the address is sent as typed and the server judges it. Neither button waits on the request, so a second Send reset link sends again; every reply lands, and the first success hands the email to Login with [`Effect::Prefill`], toasts [`LINK_SENT`] and goes there in place of this screen. The text input and its focus stay in the client.

#[cfg(test)]
mod tests;

use crate::account;
use crate::generation::{Generation, Ticket};
use crate::toast::{Toast, ToastKind};

/// The screen's title.
pub const TITLE: &str = "Forgot password";

/// The email field's label and placeholder.
pub const EMAIL: &str = "Email";

/// The button back to Login.
pub const BACK: &str = "Back to log in";

/// The submit button.
pub const SEND: &str = "Send reset link";

/// The button to Register.
pub const CREATE_ACCOUNT: &str = "Create account";

/// The toast once a request went in. It never says whether the address has an account.
pub const LINK_SENT: &str = "If that email is registered, a reset link is on its way";

/// Where an action sends the user. The client maps each to its own route.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Destination {
    /// Back where the user came from, Login, from Back to log in.
    Previous,
    /// Login in place of this screen, once a request went in.
    Login,
    /// Register, from Create account.
    Register,
}

/// What a user does on Forgot password, and each reply the client hands back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Sets the email to what is typed.
    EditEmail(String),
    /// Presses Send reset link. The email is judged from then on; the request goes out when it passes.
    Submit,
    /// The request holding `ticket` went in.
    Sent {
        /// The ticket from the [`Effect::RequestReset`] this answers.
        ticket: Ticket,
    },
    /// The request holding `ticket` failed. The client has already logged it and raised its error toast.
    Failed {
        /// The ticket from the [`Effect::RequestReset`] this answers.
        ticket: Ticket,
    },
    /// Presses Back to log in.
    Back,
    /// Presses Create account.
    CreateProfile,
}

/// What an action asks the client to do, in the order it should do it.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Effect {
    /// Asks for a reset link and answers with [`Action::Sent`] or [`Action::Failed`] carrying this ticket.
    RequestReset {
        /// The ticket to hand back.
        ticket: Ticket,
        /// The email as typed.
        email: String,
    },
    /// Hands this email to Login to fill in.
    Prefill(String),
    /// Goes to this screen.
    Navigate(Destination),
    /// Raises this toast.
    Toast(Toast),
}

/// Forgot password for one visit: the field empty, nothing judged, nothing out.
///
/// Every request takes a fresh [`Ticket`] from one [`Generation`] and waits in a list, so each reply finds its own request and an unknown or repeated ticket changes nothing.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct State {
    email: String,
    /// Whether Send reset link was pressed, after which the email shows its line even empty.
    attempted: bool,
    /// Whether the spinner turns: set by a send, cleared by any reply.
    loading: bool,
    /// The requests out.
    waiting: Vec<Ticket>,
    /// Whether a request went in: the first success leaves for Login, later replies change nothing.
    sent: bool,
    tickets: Generation,
}

impl State {
    /// The screen as it first draws: the field empty, nothing out.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Applies `action` and returns what the client must do, in order.
    ///
    /// Send reset link does nothing once a request went in, as the screen is leaving.
    #[must_use = "the client performs the effects"]
    pub fn update(&mut self, action: Action) -> Vec<Effect> {
        match action {
            Action::EditEmail(value) => {
                self.email = value;
                Vec::new()
            }
            Action::Submit => self.submit(),
            Action::Sent { ticket } => {
                if !self.settle(ticket) || self.sent {
                    return Vec::new();
                }
                self.sent = true;
                vec![
                    Effect::Prefill(self.email.clone()),
                    Effect::Toast(Toast::normal(ToastKind::Info, LINK_SENT)),
                    Effect::Navigate(Destination::Login),
                ]
            }
            Action::Failed { ticket } => {
                self.settle(ticket);
                Vec::new()
            }
            Action::Back => vec![Effect::Navigate(Destination::Previous)],
            Action::CreateProfile => vec![Effect::Navigate(Destination::Register)],
        }
    }

    /// The line under the email, while it holds text or once a send was tried; an emptied field shows none until then.
    #[must_use]
    pub fn error(&self) -> Option<String> {
        if !self.attempted && self.email.is_empty() {
            return None;
        }
        account::reset_email_error(&self.email)
    }

    /// Whether the spinner turns under the field.
    #[must_use]
    pub fn is_loading(&self) -> bool {
        self.loading
    }

    /// Whether a request went in, so Send reset link does nothing more.
    #[must_use]
    pub fn is_sent(&self) -> bool {
        self.sent
    }

    fn submit(&mut self) -> Vec<Effect> {
        if self.sent {
            return Vec::new();
        }
        self.attempted = true;
        if self.error().is_some() {
            return Vec::new();
        }
        let ticket = self.tickets.next_ticket();
        self.waiting.push(ticket);
        self.loading = true;
        vec![Effect::RequestReset {
            ticket,
            email: self.email.clone(),
        }]
    }

    /// Takes the request holding `ticket` off the waiting list and stops the spinner, and returns whether it was waiting.
    fn settle(&mut self, ticket: Ticket) -> bool {
        let before = self.waiting.len();
        self.waiting.retain(|t| *t != ticket);
        if self.waiting.len() == before {
            return false;
        }
        self.loading = false;
        true
    }
}
