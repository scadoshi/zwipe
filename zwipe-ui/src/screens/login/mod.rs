//! Login: a username-or-email and password form with Log in, Create account and Forgot password under it.
//!
//! The client keeps a [`State`] while the screen is up and hands every keystroke in as [`Action::EditIdentifier`] or [`Action::EditPassword`]. The submit gate is [`account::login_gate`]. A login that goes through comes back as [`Effect::SignedIn`] carrying the session: storing it, applying its settings and the keyring are the client's. The text inputs, their focus and the password's Show and Hide stay in the client.

#[cfg(test)]
mod tests;

use zwipe_core::domain::auth::models::session::Session;

use crate::account::{self, LoginGate, Password};
use crate::generation::{Generation, Ticket};
use crate::toast::Toast;

/// The screen's title.
pub const TITLE: &str = "Log in";

/// The submit button's label.
pub const LOG_IN: &str = "Log in";

/// The submit button's label while a login is out.
pub const LOGGING_IN: &str = "Logging in…";

/// The button that opens Register.
pub const CREATE_ACCOUNT: &str = "Create account";

/// The button that opens Forgot password.
pub const FORGOT_PASSWORD: &str = "Forgot password";

/// A field on the form.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Field {
    /// The username or email, sent as typed.
    Identifier,
    /// The password, checked for presence only.
    Password,
}

impl Field {
    /// Every field, top to bottom.
    pub const ALL: [Self; 2] = [Self::Identifier, Self::Password];

    /// The label above the field.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Identifier => "Username or email",
            Self::Password => "Password",
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
        self == Self::Password
    }
}

/// Where an action sends the user. The client maps each to its own route.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Destination {
    /// Home, once a login goes through.
    Home,
    /// Register, from Create account.
    Register,
    /// Forgot password, from its button.
    ForgotPassword,
}

/// What a user does on Login, and each reply the client hands back.
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    /// Sets the identifier to what is typed.
    EditIdentifier(String),
    /// Sets the password to what is typed.
    EditPassword(Password),
    /// Fills in the identifier another screen handed over (Register's username) and empties the password.
    Prefill(String),
    /// Presses Log in.
    Submit,
    /// The login holding `ticket` went through.
    LoggedIn {
        /// The ticket from the [`Effect::SignIn`] this answers.
        ticket: Ticket,
        /// The session the server answered with.
        session: Box<Session>,
    },
    /// The login holding `ticket` failed. The client has already logged it and raised its error toast.
    Failed {
        /// The ticket from the [`Effect::SignIn`] this answers.
        ticket: Ticket,
    },
    /// Presses Create account.
    CreateProfile,
    /// Presses Forgot password.
    ForgotPassword,
}

/// What an action asks the client to do, in the order it should do it.
#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    /// Sends the login and answers with [`Action::LoggedIn`] or [`Action::Failed`] carrying this ticket.
    SignIn {
        /// The ticket to hand back.
        ticket: Ticket,
        /// The username or email as typed.
        identifier: String,
        /// The password as typed.
        password: Password,
    },
    /// Puts this text in the identifier input. The model already holds it.
    SetIdentifier(String),
    /// Empties the password input. The model already has.
    ClearPassword,
    /// Takes this session as the signed-in one: the client stores it and applies its settings.
    SignedIn(Box<Session>),
    /// Goes to this screen.
    Navigate(Destination),
    /// Raises this toast.
    Toast(Toast),
}

/// Login for one visit: both fields empty, nothing out.
///
/// Every login takes a fresh [`Ticket`]; a reply lands only while its login is the one out.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct State {
    identifier: String,
    password: Password,
    /// The login out, while one is.
    sending: Option<Ticket>,
    tickets: Generation,
}

impl State {
    /// The screen as it first draws: both fields empty, nothing out.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Applies `action` and returns what the client must do, in order.
    ///
    /// Create account and Forgot password do nothing while a login is out, as their buttons are held.
    #[must_use = "the client performs the effects"]
    pub fn update(&mut self, action: Action) -> Vec<Effect> {
        match action {
            Action::EditIdentifier(value) => {
                self.identifier = value;
                Vec::new()
            }
            Action::EditPassword(value) => {
                self.password = value;
                Vec::new()
            }
            Action::Prefill(identifier) => {
                self.identifier.clone_from(&identifier);
                self.password = Password::default();
                vec![Effect::SetIdentifier(identifier), Effect::ClearPassword]
            }
            Action::Submit => self.submit(),
            Action::LoggedIn { ticket, session } => {
                if !self.settle(ticket) {
                    return Vec::new();
                }
                self.password = Password::default();
                vec![
                    Effect::SignedIn(session),
                    Effect::ClearPassword,
                    Effect::Navigate(Destination::Home),
                ]
            }
            Action::Failed { ticket } => {
                self.settle(ticket);
                Vec::new()
            }
            Action::CreateProfile => self.leave(Destination::Register),
            Action::ForgotPassword => self.leave(Destination::ForgotPassword),
        }
    }

    /// The identifier as the model holds it.
    #[must_use]
    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    /// Whether a login is out. Every button is held meanwhile.
    #[must_use]
    pub fn is_loading(&self) -> bool {
        self.sending.is_some()
    }

    /// The submit button's label: [`LOGGING_IN`] while a login is out, else [`LOG_IN`].
    #[must_use]
    pub fn submit_label(&self) -> &'static str {
        if self.is_loading() {
            LOGGING_IN
        } else {
            LOG_IN
        }
    }

    fn submit(&mut self) -> Vec<Effect> {
        match account::login_gate(self.is_loading(), &self.identifier, self.password.read()) {
            LoginGate::Busy => Vec::new(),
            LoginGate::MissingInputs => vec![Effect::Toast(account::missing_inputs_toast())],
            LoginGate::Send => {
                let ticket = self.tickets.next_ticket();
                self.sending = Some(ticket);
                vec![Effect::SignIn {
                    ticket,
                    identifier: self.identifier.clone(),
                    password: self.password.clone(),
                }]
            }
        }
    }

    /// Ends the login holding `ticket`, and returns whether it was the one out.
    fn settle(&mut self, ticket: Ticket) -> bool {
        if self.sending != Some(ticket) {
            return false;
        }
        self.sending = None;
        true
    }

    fn leave(&self, to: Destination) -> Vec<Effect> {
        if self.is_loading() {
            return Vec::new();
        }
        vec![Effect::Navigate(to)]
    }
}
