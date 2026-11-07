//! Register: username, email, password and confirmation fields that check themselves as they are typed, with Back to login and Create profile under them.
//!
//! The client keeps a [`State`] while the screen is up and hands every keystroke in as an `Edit` action. Each check comes from [`crate::account`]. A field starts showing its line once it has held something, or after a submit, and from then on follows every change; the password's line covers the confirmation. A registration that goes through hands its username to Login with [`Effect::Prefill`] and goes there in place of this screen, so Back does not return to the filled form. The text inputs, their focus, the password rules' `?` and the funnel events stay in the client.

#[cfg(test)]
mod tests;

use crate::account::{self, Password};
use crate::generation::{Generation, Ticket};
use crate::toast::{Toast, ToastKind};

/// The screen's title.
pub const TITLE: &str = "Create profile";

/// The button back to Login.
pub const BACK: &str = "Back to login";

/// The submit button's label.
pub const CREATE_PROFILE: &str = "Create profile";

/// The submit button's label while a registration is out.
pub const CREATING: &str = "Creating...";

/// The toast once the server took the registration. The server answers the same whether or not the email already has an account, so this says the same either way.
pub const CHECK_INBOX: &str = "Check your inbox to verify your email, then log in";

/// A field on the form.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Field {
    /// The username, sent as typed.
    Username,
    /// The email, checked and sent trimmed.
    Email,
    /// The password. Its line covers the confirmation too.
    Password,
    /// The password again. It has no line of its own.
    ConfirmPassword,
}

impl Field {
    /// Every field, top to bottom.
    pub const ALL: [Self; 4] = [
        Self::Username,
        Self::Email,
        Self::Password,
        Self::ConfirmPassword,
    ];

    /// The label above the field.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Username => "Username",
            Self::Email => "Email",
            Self::Password => "Password",
            Self::ConfirmPassword => "Confirm password",
        }
    }

    /// The placeholder inside the empty field.
    #[must_use]
    pub fn placeholder(self) -> &'static str {
        self.label()
    }

    /// Whether the field hides what is typed.
    #[must_use]
    pub fn is_masked(self) -> bool {
        matches!(self, Self::Password | Self::ConfirmPassword)
    }
}

/// Where an action sends the user. The client maps each to its own route.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Destination {
    /// Back where the user came from, Login, from Back to login.
    Previous,
    /// Login in place of this screen, once a registration goes through.
    Login,
}

/// What a user does on Register, and each reply the client hands back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Sets the username to what is typed.
    EditUsername(String),
    /// Sets the email to what is typed.
    EditEmail(String),
    /// Sets the password to what is typed.
    EditPassword(Password),
    /// Sets the confirmation to what is typed.
    EditConfirm(Password),
    /// Presses Create profile. Every field is judged from then on; the registration goes out when all pass.
    Submit,
    /// The registration holding `ticket` went through.
    Registered {
        /// The ticket from the [`Effect::Register`] this answers.
        ticket: Ticket,
    },
    /// The registration holding `ticket` failed. The client has already logged it and raised its error toast.
    Failed {
        /// The ticket from the [`Effect::Register`] this answers.
        ticket: Ticket,
    },
    /// Presses Back to login.
    Back,
}

/// What an action asks the client to do, in the order it should do it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    /// Sends the registration and answers with [`Action::Registered`] or [`Action::Failed`] carrying this ticket.
    Register {
        /// The ticket to hand back.
        ticket: Ticket,
        /// The username as typed.
        username: String,
        /// The email, trimmed.
        email: String,
        /// The password as typed.
        password: Password,
    },
    /// Hands this username to Login to fill in.
    Prefill(String),
    /// Goes to this screen.
    Navigate(Destination),
    /// Raises this toast.
    Toast(Toast),
}

/// The registration out, with the username it carries for Login.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Sending {
    ticket: Ticket,
    username: String,
}

/// Register for one visit: every field empty, nothing judged, nothing out.
///
/// Every registration takes a fresh [`Ticket`]; a reply lands only while its registration is the one out.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct State {
    username: String,
    email: String,
    password: Password,
    confirm: Password,
    /// Whether Create profile was pressed.
    attempted: bool,
    sending: Option<Sending>,
    tickets: Generation,
}

impl State {
    /// The screen as it first draws: every field empty, nothing out.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Applies `action` and returns what the client must do, in order.
    ///
    /// Create profile does nothing while a registration is out, as its button is held.
    #[must_use = "the client performs the effects"]
    pub fn update(&mut self, action: Action) -> Vec<Effect> {
        match action {
            Action::EditUsername(value) => {
                self.username = value;
                Vec::new()
            }
            Action::EditEmail(value) => {
                self.email = value;
                Vec::new()
            }
            Action::EditPassword(value) => {
                self.password = value;
                Vec::new()
            }
            Action::EditConfirm(value) => {
                self.confirm = value;
                Vec::new()
            }
            Action::Submit => self.submit(),
            Action::Registered { ticket } => {
                let Some(username) = self.settle(ticket) else {
                    return Vec::new();
                };
                vec![
                    Effect::Prefill(username),
                    Effect::Toast(Toast::normal(ToastKind::Info, CHECK_INBOX)),
                    Effect::Navigate(Destination::Login),
                ]
            }
            Action::Failed { ticket } => {
                self.settle(ticket);
                Vec::new()
            }
            Action::Back => vec![Effect::Navigate(Destination::Previous)],
        }
    }

    /// The line under `field`, while it holds text or once a submit was tried; an emptied field shows none until then. Only [`Field::Username`], [`Field::Email`] and [`Field::Password`] have one.
    #[must_use]
    pub fn error(&self, field: Field) -> Option<String> {
        if !self.attempted && !self.holds_text(field) {
            return None;
        }
        match field {
            Field::Username => account::username_error(&self.username),
            Field::Email => account::email_error(self.email.trim()),
            Field::Password => {
                account::password_error(self.password.read(), self.confirm.read(), self.attempted)
            }
            Field::ConfirmPassword => None,
        }
    }

    /// Whether a registration is out. Both buttons are held meanwhile.
    #[must_use]
    pub fn is_loading(&self) -> bool {
        self.sending.is_some()
    }

    /// The submit button's label: [`CREATING`] while a registration is out, else [`CREATE_PROFILE`].
    #[must_use]
    pub fn submit_label(&self) -> &'static str {
        if self.is_loading() {
            CREATING
        } else {
            CREATE_PROFILE
        }
    }

    /// Whether `field` holds text now. [`Field::Password`]'s line covers the confirmation too.
    fn holds_text(&self, field: Field) -> bool {
        match field {
            Field::Username => !self.username.is_empty(),
            Field::Email => !self.email.is_empty(),
            Field::Password => !self.password.read().is_empty() || !self.confirm.read().is_empty(),
            Field::ConfirmPassword => false,
        }
    }

    fn submit(&mut self) -> Vec<Effect> {
        if self.is_loading() {
            return Vec::new();
        }
        self.attempted = true;
        if Field::ALL.iter().any(|field| self.error(*field).is_some()) {
            return Vec::new();
        }
        let ticket = self.tickets.next_ticket();
        self.sending = Some(Sending {
            ticket,
            username: self.username.clone(),
        });
        vec![Effect::Register {
            ticket,
            username: self.username.clone(),
            email: self.email.trim().to_owned(),
            password: self.password.clone(),
        }]
    }

    /// Ends the registration holding `ticket`, and returns its username if it was the one out.
    fn settle(&mut self, ticket: Ticket) -> Option<String> {
        if self.sending.as_ref().is_none_or(|s| s.ticket != ticket) {
            return None;
        }
        self.sending.take().map(|s| s.username)
    }
}
