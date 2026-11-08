//! Account: the user's username, email and password with a sheet to change each, the email's verification with its resend cooldown, logging out, and deleting the account behind its countdown.
//!
//! The client keeps a [`State`] while the screen is up and sends [`Action::Load`] with the session's user when it opens. Every check and label comes from [`crate::account`] and the two waits from [`crate::timing`]. The client keeps the clock: each [`Effect::Wait`] asks it to wait one [`TICK`] and send [`Action::Tick`] back, so the model counts whole seconds and never reads the time. The sheets' and dialogs' visibility, the password typed into Delete account, and the version string stay in the client.

mod form;
#[cfg(test)]
mod tests;

use std::time::Duration;

use chrono::{DateTime, Utc};
use zwipe_core::domain::user::hints::OneTimeHint;
use zwipe_core::domain::user::{User, Username};

use self::form::Form;
pub use self::form::{Field, SAVE, SAVING, Sheet, Typed};
pub use crate::account::Password;
use crate::generation::{Generation, Ticket};
use crate::toast::{Toast, ToastKind};
use crate::{account, timing};

/// The screen's title.
pub const TITLE: &str = "Account";

/// The list's title.
pub const ACCOUNT: &str = "Account";

/// The button on the Username, Email and Password rows that opens their sheet.
pub const CHANGE: &str = "Change";

/// The Email row's badge once the address is verified.
pub const VERIFIED: &str = "Verified";

/// The Email row's badge until the address is verified.
pub const UNVERIFIED: &str = "Unverified";

/// The Password row's value.
pub const PASSWORD_MASK: &str = "•••••••";

/// The button that asks to log out.
pub const LOG_OUT: &str = "Log out";

/// The title of the panel at the foot of the screen that holds Log out and Delete account.
pub const DANGER_ZONE: &str = "Danger zone";

/// The Delete account row's button.
pub const DELETE: &str = "Delete";

/// The button that closes a dialog without acting.
pub const CANCEL: &str = "Cancel";

/// The Delete account dialog's password placeholder.
pub const DELETE_PASSWORD: &str = "Password";

/// The Delete account dialog's button label while the delete is out.
pub const DELETING: &str = "Deleting…";

/// The toast once a verification email went out.
pub const VERIFICATION_SENT: &str = "Verification email sent";

/// The toast when the server says a verification email went out too recently.
pub const RESEND_LIMITED: &str = "Wait a moment and try again";

/// The toast when a check finds the email verified.
pub const EMAIL_VERIFIED: &str = "Email verified";

/// The toast when a check finds the email still unverified.
pub const NOT_VERIFIED: &str = "Not verified yet. Check your inbox";

/// The toast once the password changed.
pub const PASSWORD_CHANGED: &str = "Password changed";
/// The Change password sheet's header chip, for a user who no longer knows the current one.
pub const FORGOT_PASSWORD: &str = "Forgot?";
/// The toast once the reset link went out, before the address it went to.
pub const RESET_LINK_SENT: &str = "Reset link sent to";

/// How long the client waits between [`Effect::Wait`] and the [`Action::Tick`] it sends back.
pub const TICK: Duration = Duration::from_secs(1);

/// Where an action sends the user. The client maps each to its own route.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Destination {
    /// Wherever the user came from, from Back.
    Previous,
}

/// A row on the screen, in the order [`State::rows`] lists them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Row {
    /// The username with its Change button.
    Username,
    /// The address, its badge and its Change button.
    Email,
    /// The address waiting on its confirmation link, with the note from [`State::pending`] under it.
    Pending,
    /// Resend and the check, while the email is unverified.
    Verification,
    /// The masked password with its Change button.
    Password,
}

impl Row {
    /// The row's label.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Username => "Username",
            Self::Email => "Email",
            Self::Pending => "Pending",
            Self::Verification => "Verification",
            Self::Password => "Password",
        }
    }
}

/// An action in the Danger zone panel besides Log out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MoreItem {
    /// Opens the Delete account dialog.
    DeleteAccount,
}

impl MoreItem {
    /// Every item, top to bottom.
    pub const ALL: [Self; 1] = [Self::DeleteAccount];

    /// The item's button text.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::DeleteAccount => "Delete account",
        }
    }
}

/// A dialog the screen opens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Dialog {
    /// Confirms a password change, from Change password's Save.
    ConfirmPassword,
    /// Confirms logging out.
    LogOut,
    /// The password and the countdown before the account is deleted.
    DeleteAccount,
}

impl Dialog {
    /// The dialog's title.
    #[must_use]
    pub fn title(self) -> &'static str {
        match self {
            Self::ConfirmPassword => "Change password",
            Self::LogOut => "Log out",
            Self::DeleteAccount => "Delete account",
        }
    }

    /// The dialog's line.
    #[must_use]
    pub fn body(self) -> &'static str {
        match self {
            Self::ConfirmPassword => {
                "Changing your password will log you out on all other devices."
            }
            Self::LogOut => "Are you sure you want to log out?",
            Self::DeleteAccount => {
                "This will permanently delete your account, all decks, and all card data. This can't be undone."
            }
        }
    }

    /// The button that goes ahead, after [`CANCEL`]. Delete account's counts down: see [`State::delete_label`].
    #[must_use]
    pub fn confirm(self) -> &'static str {
        match self {
            Self::ConfirmPassword => "Confirm",
            Self::LogOut => "Log out",
            Self::DeleteAccount => "Delete",
        }
    }
}

/// The pending email row's contents.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Pending {
    /// The address waiting on its confirmation link.
    pub email: String,
    /// How long the link lasts, when the server said.
    pub note: Option<String>,
}

/// What a user does on Account, and each reply the client hands back.
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    /// Opens the screen with the session's user, offering the hint and fetching the user again so the verification is current. `None` draws nothing and fetches nothing.
    Load(Option<Box<User>>),
    /// The user arrived for the fetch holding `ticket`. The client has already stored them in its session.
    UserLoaded {
        /// The ticket from the [`Effect::FetchUser`] or [`Effect::CheckVerification`] this answers.
        ticket: Ticket,
        /// The user as the server has them now.
        user: Box<User>,
    },
    /// The request holding `ticket` failed. The client has already reported it and raised any error toast.
    Failed {
        /// The ticket from the effect this answers.
        ticket: Ticket,
    },
    /// One [`TICK`] passed for the wait holding `ticket`.
    Tick {
        /// The ticket from the [`Effect::Wait`] this answers.
        ticket: Ticket,
    },
    /// Sends the verification email again, unless one is out or the cooldown runs.
    Resend,
    /// The verification email holding `ticket` went out.
    Resent {
        /// The ticket from the [`Effect::ResendVerification`] this answers.
        ticket: Ticket,
    },
    /// The server refused the resend holding `ticket` because one went out too recently. The cooldown keeps running.
    ResendLimited {
        /// The ticket from the [`Effect::ResendVerification`] this answers.
        ticket: Ticket,
    },
    /// Asks the server whether the email is verified yet, unless a check is out.
    CheckVerification,
    /// Opens a change sheet with every field empty.
    OpenSheet(Sheet),
    /// Sets a field on a sheet to what is typed.
    Edit {
        /// The sheet the field is on.
        sheet: Sheet,
        /// The field.
        field: Field,
        /// The field's whole text.
        value: Typed,
    },
    /// Presses a sheet's Save. Every field is judged; Change username and Change email send when all pass, and Change password asks [`Dialog::ConfirmPassword`] first.
    Save(Sheet),
    /// Confirms the password change, from [`Dialog::ConfirmPassword`].
    ConfirmPassword,
    /// Presses the Change password sheet's Forgot password: asks for a reset link to the account's email, unless one is out.
    ForgotPassword,
    /// The reset link holding `ticket` went out.
    ResetLinkSent {
        /// The ticket from the [`Effect::RequestPasswordReset`] this answers.
        ticket: Ticket,
    },
    /// The username change holding `ticket` went through.
    UsernameChanged {
        /// The ticket from the [`Effect::ChangeUsername`] this answers.
        ticket: Ticket,
        /// The new username, as the server sent it back. The client has already stored it in its session.
        username: Username,
    },
    /// The email change holding `ticket` went through: the new address waits on its link.
    EmailChanged {
        /// The ticket from the [`Effect::ChangeEmail`] this answers.
        ticket: Ticket,
        /// The user as the server sent them back. The client has already stored them in its session.
        user: Box<User>,
    },
    /// The password change holding `ticket` went through.
    PasswordChanged {
        /// The ticket from the [`Effect::ChangePassword`] this answers.
        ticket: Ticket,
    },
    /// Goes back to where the user came from.
    Back,
    /// Asks to log out.
    LogOut,
    /// Logs out, from [`Dialog::LogOut`].
    ConfirmLogOut,
    /// Presses a Danger zone action.
    More(MoreItem),
    /// The Delete account dialog closed without deleting. Its countdown stops.
    CancelDelete,
    /// Deletes the account with the password as typed, once the countdown is done and no delete is out.
    Delete {
        /// The password as typed.
        password: Password,
    },
    /// The delete holding `ticket` went through.
    Deleted {
        /// The ticket from the [`Effect::DeleteAccount`] this answers.
        ticket: Ticket,
    },
}

/// What an action asks the client to do, in the order it should do it.
#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    /// Opens this one-time hint if the user hasn't seen it, and records it as seen.
    OfferHint(OneTimeHint),
    /// Fetches the user, stores them in the session, and answers with [`Action::UserLoaded`] carrying this ticket. A failure only logs.
    FetchUser(Ticket),
    /// Fetches the user, stores them in the session, and answers with [`Action::UserLoaded`] or [`Action::Failed`] carrying this ticket.
    CheckVerification(Ticket),
    /// Sends the verification email and answers with [`Action::Resent`], [`Action::ResendLimited`] when the server says too many, or [`Action::Failed`].
    ResendVerification(Ticket),
    /// Waits one [`TICK`], then answers with [`Action::Tick`] carrying this ticket. A wait nobody needs any more is answered and ignored, so the client never cancels one.
    Wait(Ticket),
    /// Opens this sheet. Its fields are already empty in the model.
    OpenSheet(Sheet),
    /// Closes this sheet.
    CloseSheet(Sheet),
    /// Changes the username, stores the new one in the session, and answers with [`Action::UsernameChanged`] or [`Action::Failed`].
    ChangeUsername {
        /// The ticket to hand back.
        ticket: Ticket,
        /// The username as typed.
        username: String,
        /// The password as typed.
        password: Password,
    },
    /// Changes the email, stores the returned user in the session, and answers with [`Action::EmailChanged`] or [`Action::Failed`].
    ChangeEmail {
        /// The ticket to hand back.
        ticket: Ticket,
        /// The address as typed.
        email: String,
        /// The password as typed.
        password: Password,
    },
    /// Changes the password and answers with [`Action::PasswordChanged`] or [`Action::Failed`].
    ChangePassword {
        /// The ticket to hand back.
        ticket: Ticket,
        /// The current password as typed.
        current: Password,
        /// The new password as typed.
        new: Password,
    },
    /// Asks for a password reset link to this email and answers with [`Action::ResetLinkSent`] or [`Action::Failed`].
    RequestPasswordReset {
        /// The ticket to hand back.
        ticket: Ticket,
        /// The account's email.
        email: String,
    },
    /// Deletes the account and answers with [`Action::Deleted`] or [`Action::Failed`].
    DeleteAccount {
        /// The ticket to hand back.
        ticket: Ticket,
        /// The password as typed.
        password: Password,
    },
    /// Opens this dialog.
    OpenDialog(Dialog),
    /// Logs out: the client ends the session, and its sign-in gate takes over.
    LogOut,
    /// Goes to this screen.
    Navigate(Destination),
    /// Raises this toast.
    Toast(Toast),
}

/// A request out, waiting for its reply.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Request {
    /// The user fetch on opening.
    Refresh,
    /// The verification check.
    Check,
    /// The verification email.
    Resend,
    /// A username change.
    Username,
    /// An email change to this address as typed.
    Email(String),
    /// A password change.
    Password,
    /// The reset link to this email.
    Reset(String),
    /// The delete.
    Delete,
}

impl Request {
    /// The sheet whose Save sent the request, if one did.
    fn sheet(&self) -> Option<Sheet> {
        match self {
            Self::Username => Some(Sheet::Username),
            Self::Email(_) => Some(Sheet::Email),
            Self::Password => Some(Sheet::Password),
            Self::Refresh | Self::Check | Self::Resend | Self::Reset(_) | Self::Delete => None,
        }
    }
}

/// A count of whole seconds running down, one [`Action::Tick`] at a time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Countdown {
    remaining: u64,
    /// The wait answering for this count. `None` until the count starts.
    wait: Option<Ticket>,
}

impl Countdown {
    /// A count from `from` whose waits start later.
    fn held(from: Duration) -> Self {
        Self {
            remaining: from.as_secs(),
            wait: None,
        }
    }

    /// Takes one second off if `ticket` is this count's wait, and returns the next wait while seconds are left. A wait from an earlier count changes nothing.
    fn tick(&mut self, ticket: Ticket) -> Option<Effect> {
        if self.wait != Some(ticket) {
            return None;
        }
        self.remaining = self.remaining.saturating_sub(1);
        if self.remaining == 0 {
            self.wait = None;
            return None;
        }
        Some(Effect::Wait(ticket))
    }
}

/// Whether the visit has offered the one-time hint yet.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
enum HintOffer {
    #[default]
    Waiting,
    Offered,
}

/// The three sheets' forms, each independent of the others.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct Forms {
    username: Form,
    email: Form,
    password: Form,
}

impl Forms {
    fn get(&self, sheet: Sheet) -> &Form {
        match sheet {
            Sheet::Username => &self.username,
            Sheet::Email => &self.email,
            Sheet::Password => &self.password,
        }
    }

    fn get_mut(&mut self, sheet: Sheet) -> &mut Form {
        match sheet {
            Sheet::Username => &mut self.username,
            Sheet::Email => &mut self.email,
            Sheet::Password => &mut self.password,
        }
    }
}

/// Account for one visit. Starts with no user; send [`Action::Load`] with the session's.
///
/// Every request and wait takes a fresh [`Ticket`] from one [`Generation`], so a ticket names exactly one of them. A reply lands only while its request is on the waiting list, and [`Action::Load`] clears the list.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct State {
    user: Option<User>,
    waiting: Vec<(Ticket, Request)>,
    /// The resend cooldown, while it runs.
    cooldown: Option<Countdown>,
    /// The Delete account dialog's countdown, while the dialog is open.
    delete: Option<Countdown>,
    forms: Forms,
    hint: HintOffer,
    tickets: Generation,
}

impl State {
    /// The screen as it first draws: no user, nothing out.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Applies `action` and returns what the client must do, in order.
    ///
    /// The hint is offered once per visit, on the first [`Action::Load`]. The resend cooldown starts as Resend is pressed, so a second press can't race the first, and runs down from the reply; a failed send clears it.
    #[must_use = "the client performs the effects"]
    pub fn update(&mut self, action: Action) -> Vec<Effect> {
        match action {
            Action::Load(user) => self.load(user.map(|u| *u)),
            Action::UserLoaded { ticket, user } => self.user_loaded(ticket, *user),
            Action::Failed { ticket } => {
                self.failed(ticket);
                Vec::new()
            }
            Action::Tick { ticket } => self.tick(ticket),
            Action::Resend => self.resend(),
            Action::Resent { ticket } => self.resent(ticket, VERIFICATION_SENT, ToastKind::Success),
            Action::ResendLimited { ticket } => {
                self.resent(ticket, RESEND_LIMITED, ToastKind::Info)
            }
            Action::CheckVerification => {
                if self.is_checking() {
                    return Vec::new();
                }
                let ticket = self.send(Request::Check);
                vec![Effect::CheckVerification(ticket)]
            }
            Action::OpenSheet(sheet) => {
                *self.forms.get_mut(sheet) = Form::default();
                vec![Effect::OpenSheet(sheet)]
            }
            Action::Edit {
                sheet,
                field,
                value,
            } => {
                if sheet.fields().contains(&field) {
                    self.forms.get_mut(sheet).edit(field, value.into_string());
                }
                Vec::new()
            }
            Action::Save(sheet) => self.save(sheet),
            Action::ForgotPassword => self.forgot_password(),
            Action::ResetLinkSent { ticket } => self.reset_link_sent(ticket),
            Action::ConfirmPassword => self.change_password(),
            Action::UsernameChanged { ticket, username } => {
                if self.take(ticket, |r| *r == Request::Username).is_none() {
                    return Vec::new();
                }
                let toast = Toast::quick(
                    ToastKind::Success,
                    format!("Username changed to {username}"),
                );
                if let Some(user) = self.user.as_mut() {
                    user.username = username;
                }
                self.changed(Sheet::Username, toast)
            }
            Action::EmailChanged { ticket, user } => {
                let Some(Request::Email(typed)) =
                    self.take(ticket, |r| matches!(r, Request::Email(_)))
                else {
                    return Vec::new();
                };
                self.user = Some(*user);
                let toast = Toast::normal(
                    ToastKind::Success,
                    format!("Check {typed} for a link to confirm the change"),
                );
                self.changed(Sheet::Email, toast)
            }
            Action::PasswordChanged { ticket } => {
                if self.take(ticket, |r| *r == Request::Password).is_none() {
                    return Vec::new();
                }
                let toast = Toast::quick(ToastKind::Success, PASSWORD_CHANGED);
                self.changed(Sheet::Password, toast)
            }
            Action::Back => vec![Effect::Navigate(Destination::Previous)],
            Action::LogOut => vec![Effect::OpenDialog(Dialog::LogOut)],
            Action::ConfirmLogOut => vec![Effect::LogOut],
            Action::More(MoreItem::DeleteAccount) => self.open_delete(),
            Action::CancelDelete => {
                self.delete = None;
                Vec::new()
            }
            Action::Delete { password } => {
                if !self.can_delete() {
                    return Vec::new();
                }
                let ticket = self.send(Request::Delete);
                vec![Effect::DeleteAccount { ticket, password }]
            }
            Action::Deleted { ticket } => {
                if self.take(ticket, |r| *r == Request::Delete).is_none() {
                    return Vec::new();
                }
                self.delete = None;
                vec![Effect::LogOut]
            }
        }
    }

    fn load(&mut self, user: Option<User>) -> Vec<Effect> {
        self.waiting.clear();
        self.cooldown = None;
        self.delete = None;
        let mut effects = Vec::new();
        if self.hint == HintOffer::Waiting {
            self.hint = HintOffer::Offered;
            effects.push(Effect::OfferHint(OneTimeHint::Profile));
        }
        let signed_in = user.is_some();
        self.user = user;
        if signed_in {
            let ticket = self.send(Request::Refresh);
            effects.push(Effect::FetchUser(ticket));
        }
        effects
    }

    fn user_loaded(&mut self, ticket: Ticket, user: User) -> Vec<Effect> {
        let wanted = |r: &Request| matches!(r, Request::Refresh | Request::Check);
        match self.take(ticket, wanted) {
            Some(Request::Refresh) => {
                self.user = Some(user);
                Vec::new()
            }
            Some(Request::Check) => {
                let toast = if user.email_verified_at.is_some() {
                    Toast::normal(ToastKind::Success, EMAIL_VERIFIED)
                } else {
                    Toast::normal(ToastKind::Info, NOT_VERIFIED)
                };
                self.user = Some(user);
                vec![Effect::Toast(toast)]
            }
            _ => Vec::new(),
        }
    }

    fn failed(&mut self, ticket: Ticket) {
        if self.take(ticket, |_| true) == Some(Request::Resend) {
            self.cooldown = None;
        }
    }

    fn tick(&mut self, ticket: Ticket) -> Vec<Effect> {
        let mut effects = Vec::new();
        if let Some(count) = self.cooldown.as_mut() {
            effects.extend(count.tick(ticket));
            if count.remaining == 0 {
                self.cooldown = None;
            }
        }
        if let Some(count) = self.delete.as_mut() {
            effects.extend(count.tick(ticket));
        }
        effects
    }

    fn resend(&mut self) -> Vec<Effect> {
        if !self.can_resend() {
            return Vec::new();
        }
        self.cooldown = Some(Countdown::held(timing::RESEND_COOLDOWN));
        let ticket = self.send(Request::Resend);
        vec![Effect::ResendVerification(ticket)]
    }

    /// Lands a resend's reply with `text` as its toast and starts the cooldown running.
    fn resent(&mut self, ticket: Ticket, text: &str, kind: ToastKind) -> Vec<Effect> {
        if self.take(ticket, |r| *r == Request::Resend).is_none() {
            return Vec::new();
        }
        let mut effects = vec![Effect::Toast(Toast::normal(kind, text))];
        let wait = self.tickets.next_ticket();
        let count = self
            .cooldown
            .get_or_insert_with(|| Countdown::held(timing::RESEND_COOLDOWN));
        count.wait = Some(wait);
        effects.push(Effect::Wait(wait));
        effects
    }

    fn save(&mut self, sheet: Sheet) -> Vec<Effect> {
        if self.is_saving(sheet) {
            return Vec::new();
        }
        let form = self.forms.get_mut(sheet);
        form.attempt();
        if !form.passes(sheet, self.user.as_ref()) {
            return Vec::new();
        }
        let password = Password::new(form.value(Field::Password));
        match sheet {
            Sheet::Username => {
                let username = form.value(Field::NewUsername).to_owned();
                let ticket = self.send(Request::Username);
                vec![Effect::ChangeUsername {
                    ticket,
                    username,
                    password,
                }]
            }
            Sheet::Email => {
                let email = form.value(Field::NewEmail).to_owned();
                let ticket = self.send(Request::Email(email.clone()));
                vec![Effect::ChangeEmail {
                    ticket,
                    email,
                    password,
                }]
            }
            Sheet::Password => vec![Effect::OpenDialog(Dialog::ConfirmPassword)],
        }
    }

    fn change_password(&mut self) -> Vec<Effect> {
        if self.is_saving(Sheet::Password) {
            return Vec::new();
        }
        let form = self.forms.get_mut(Sheet::Password);
        form.attempt();
        if !form.passes(Sheet::Password, self.user.as_ref()) {
            return Vec::new();
        }
        let current = Password::new(form.value(Field::CurrentPassword));
        let new = Password::new(form.value(Field::NewPassword));
        let ticket = self.send(Request::Password);
        vec![Effect::ChangePassword {
            ticket,
            current,
            new,
        }]
    }

    /// Asks for a reset link to the account's email, unless one is already out or no user is loaded.
    fn forgot_password(&mut self) -> Vec<Effect> {
        let Some(user) = self.user.as_ref() else {
            return Vec::new();
        };
        if self.is_waiting(|r| matches!(r, Request::Reset(_))) {
            return Vec::new();
        }
        let email = user.email.to_string();
        let ticket = self.send(Request::Reset(email.clone()));
        vec![Effect::RequestPasswordReset { ticket, email }]
    }

    /// Says where the link went and closes the sheet, since the next step is the inbox.
    fn reset_link_sent(&mut self, ticket: Ticket) -> Vec<Effect> {
        let Some(Request::Reset(email)) = self.take(ticket, |r| matches!(r, Request::Reset(_)))
        else {
            return Vec::new();
        };
        *self.forms.get_mut(Sheet::Password) = Form::default();
        vec![
            Effect::Toast(Toast::normal(
                ToastKind::Success,
                format!("{RESET_LINK_SENT} {email}"),
            )),
            Effect::CloseSheet(Sheet::Password),
        ]
    }

    /// Empties `sheet` and closes it after its change went through.
    fn changed(&mut self, sheet: Sheet, toast: Toast) -> Vec<Effect> {
        *self.forms.get_mut(sheet) = Form::default();
        vec![Effect::Toast(toast), Effect::CloseSheet(sheet)]
    }

    fn open_delete(&mut self) -> Vec<Effect> {
        let wait = self.tickets.next_ticket();
        let mut count = Countdown::held(timing::DELETE_COUNTDOWN);
        count.wait = Some(wait);
        self.delete = Some(count);
        vec![
            Effect::OpenDialog(Dialog::DeleteAccount),
            Effect::Wait(wait),
        ]
    }

    /// Puts `request` on the waiting list under a fresh ticket.
    fn send(&mut self, request: Request) -> Ticket {
        let ticket = self.tickets.next_ticket();
        self.waiting.push((ticket, request));
        ticket
    }

    /// Takes the request holding `ticket` off the waiting list when it is there and `wanted` accepts it, so a reply of the wrong kind leaves it waiting.
    fn take(&mut self, ticket: Ticket, wanted: impl Fn(&Request) -> bool) -> Option<Request> {
        let at = self
            .waiting
            .iter()
            .position(|(t, request)| *t == ticket && wanted(request))?;
        Some(self.waiting.remove(at).1)
    }

    fn is_waiting(&self, matches: impl Fn(&Request) -> bool) -> bool {
        self.waiting.iter().any(|(_, request)| matches(request))
    }

    /// The signed-in user, once [`Action::Load`] named one. The screen draws nothing without.
    #[must_use]
    pub fn user(&self) -> Option<&User> {
        self.user.as_ref()
    }

    /// Whether the user's email is verified. False with no user.
    #[must_use]
    pub fn is_verified(&self) -> bool {
        self.user
            .as_ref()
            .is_some_and(|u| u.email_verified_at.is_some())
    }

    /// The Email row's badge: [`VERIFIED`] or [`UNVERIFIED`].
    #[must_use]
    pub fn badge(&self) -> &'static str {
        if self.is_verified() {
            VERIFIED
        } else {
            UNVERIFIED
        }
    }

    /// The rows that draw, top to bottom: Pending only while an address waits on its link, Verification only while the email is unverified. Empty with no user.
    #[must_use]
    pub fn rows(&self) -> Vec<Row> {
        let Some(user) = self.user.as_ref() else {
            return Vec::new();
        };
        let mut rows = vec![Row::Username, Row::Email];
        if user.pending_email.is_some() {
            rows.push(Row::Pending);
        }
        if user.email_verified_at.is_none() {
            rows.push(Row::Verification);
        }
        rows.push(Row::Password);
        rows
    }

    /// The Pending row as of `now`, while an address waits on its link. The note is [`account::pending_note`], left out when the server sent no expiry.
    #[must_use]
    pub fn pending(&self, now: DateTime<Utc>) -> Option<Pending> {
        let user = self.user.as_ref()?;
        let email = user.pending_email.as_ref()?;
        Some(Pending {
            email: email.to_string(),
            note: user
                .pending_email_expires_at
                .map(|at| account::pending_note(at, now)),
        })
    }

    /// Whether a verification email is on its way.
    #[must_use]
    pub fn is_sending(&self) -> bool {
        self.is_waiting(|r| *r == Request::Resend)
    }

    /// Seconds left before Resend can be pressed again; 0 once it can.
    #[must_use]
    pub fn cooldown_secs(&self) -> u64 {
        self.cooldown.map_or(0, |count| count.remaining)
    }

    /// Whether Resend takes a press: nothing on its way and no cooldown.
    #[must_use]
    pub fn can_resend(&self) -> bool {
        !self.is_sending() && self.cooldown_secs() == 0
    }

    /// The Resend button's label, [`account::resend_label`].
    #[must_use]
    pub fn resend_label(&self) -> String {
        account::resend_label(self.is_sending(), self.cooldown_secs())
    }

    /// Whether the verification check is out. Its button is off meanwhile.
    #[must_use]
    pub fn is_checking(&self) -> bool {
        self.is_waiting(|r| *r == Request::Check)
    }

    /// What `field` on `sheet` holds.
    #[must_use]
    pub fn value(&self, sheet: Sheet, field: Field) -> &str {
        self.forms.get(sheet).value(field)
    }

    /// The line under `field` on `sheet`, if any: shown once the field has held something or Save was pressed.
    #[must_use]
    pub fn error(&self, sheet: Sheet, field: Field) -> Option<String> {
        self.forms
            .get(sheet)
            .error(sheet, field, self.user.as_ref())
    }

    /// Whether an error line other than a password's is showing on `sheet`, which holds its Save.
    #[must_use]
    pub fn submit_held(&self, sheet: Sheet) -> bool {
        sheet
            .fields()
            .iter()
            .any(|&field| !field.is_password() && self.error(sheet, field).is_some())
    }

    /// Whether `sheet`'s change is out. Its Back and Save are off meanwhile.
    #[must_use]
    pub fn is_saving(&self, sheet: Sheet) -> bool {
        self.is_waiting(|r| r.sheet() == Some(sheet))
    }

    /// `sheet`'s Save label: [`SAVING`] while its change is out, else [`SAVE`].
    #[must_use]
    pub fn save_label(&self, sheet: Sheet) -> &'static str {
        if self.is_saving(sheet) { SAVING } else { SAVE }
    }

    /// Seconds left on the Delete account dialog's countdown; 0 once it is done or the dialog is closed.
    #[must_use]
    pub fn delete_secs(&self) -> u64 {
        self.delete.map_or(0, |count| count.remaining)
    }

    /// The Delete button's label, [`account::delete_label`]. A client whose button shows its own busy label puts [`DELETING`] there while [`Self::is_deleting`].
    #[must_use]
    pub fn delete_label(&self) -> String {
        account::delete_label(self.delete_secs())
    }

    /// Whether the delete is out.
    #[must_use]
    pub fn is_deleting(&self) -> bool {
        self.is_waiting(|r| *r == Request::Delete)
    }

    /// Whether Delete takes a press: the dialog is open, its countdown is done, and no delete is out.
    #[must_use]
    pub fn can_delete(&self) -> bool {
        self.delete.is_some() && self.delete_secs() == 0 && !self.is_deleting()
    }
}
