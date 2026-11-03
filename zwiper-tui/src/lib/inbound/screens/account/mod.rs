//! The Account screen: the user's details and verification, the About rows, the sheets that change the username, email and password, the More sheet with Delete account, and the Log out confirm.
//!
//! What the screen knows and decides is zwipe-ui's [`State`]; this view adds the focus, the text boxes and which sheet or dialog is up. The model counts its two countdowns in whole seconds through [`Job::Wait`], so nothing here reads the clock.

pub(crate) mod draw;
pub(crate) mod keys;
#[cfg(test)]
mod tests;

use zwipe_client::ClientError;
use zwipe_core::domain::auth::models::secret::Secret;
use zwipe_core::domain::user::User;
use zwipe_core::http::contracts::auth::{
    HttpChangeEmail, HttpChangePassword, HttpChangeUsername, HttpDeleteUser,
};
use zwipe_ui::generation::Ticket;
use zwipe_ui::screens::account::{
    Action, CANCEL, Destination, Dialog, Effect, Field, Password, Row, Sheet, State, TICK,
};

use super::visit::Visit;
use super::{Screen, changelog, login, privacy};
use crate::inbound::app::Context;
use crate::inbound::nav::Nav;
use crate::inbound::screens;
use crate::inbound::widgets::alert::AlertDialog;
use crate::inbound::widgets::browser;
use crate::inbound::widgets::hint::{self, HintTopic};
use crate::inbound::widgets::text_input::{Label, TextInput};
use crate::inbound::widgets::toast::{self, ToastKind};
use crate::outbound::account::Account;

/// A button on one of the screen's rows, in the order the focus walks them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Stop {
    ChangeUsername,
    ChangeEmail,
    /// Sends the verification email again. Only while the email is unverified.
    Resend,
    /// Asks the server whether the email is verified yet. Only while it is unverified.
    Check,
    ChangePassword,
    Website,
    Privacy,
    Changelog,
}

impl Stop {
    /// The `?` topic of the row this button sits on, if the row has one.
    pub(crate) fn topic(self) -> Option<HintTopic> {
        match self {
            Self::ChangeEmail => Some(HintTopic::Email),
            Self::Resend | Self::Check => Some(HintTopic::Verification),
            Self::Website => Some(HintTopic::Website),
            Self::Changelog => Some(HintTopic::Version),
            _ => None,
        }
    }

    /// The buttons on `row`, left to right.
    fn on(row: Row) -> &'static [Self] {
        match row {
            Row::Username => &[Self::ChangeUsername],
            Row::Email => &[Self::ChangeEmail],
            Row::Pending => &[],
            Row::Verification => &[Self::Resend, Self::Check],
            Row::Password => &[Self::ChangePassword],
            Row::Website => &[Self::Website],
            Row::PrivacyPolicy => &[Self::Privacy],
            Row::Version => &[Self::Changelog],
        }
    }

    /// What pressing the button asks of the model.
    fn action(self) -> Action {
        match self {
            Self::ChangeUsername => Action::OpenSheet(Sheet::Username),
            Self::ChangeEmail => Action::OpenSheet(Sheet::Email),
            Self::Resend => Action::Resend,
            Self::Check => Action::CheckVerification,
            Self::ChangePassword => Action::OpenSheet(Sheet::Password),
            Self::Website => Action::Website,
            Self::Privacy => Action::PrivacyPolicy,
            Self::Changelog => Action::Changelog,
        }
    }
}

/// The Account screen's state: the model plus what only the terminal keeps.
#[derive(Debug)]
pub(crate) struct AccountView {
    pub(crate) model: State,
    pub(crate) visit: Visit,
    /// The focused button, an index into [`AccountView::stops`].
    pub(crate) focus: usize,
    pub(crate) overlay: Option<Overlay>,
}

impl AccountView {
    /// A view on a fresh model, before [`Action::Load`].
    pub(crate) fn new() -> Self {
        Self {
            model: State::new(),
            visit: Visit::next(),
            focus: 0,
            overlay: None,
        }
    }

    /// The buttons on the model's rows, in focus order.
    pub(crate) fn stops(&self) -> Vec<Stop> {
        self.model
            .rows()
            .into_iter()
            .flat_map(|row| Stop::on(row).iter().copied())
            .collect()
    }

    pub(crate) fn focused(&self) -> Option<Stop> {
        self.stops().get(self.focus).copied()
    }

    pub(crate) fn next(&mut self) {
        self.focus = self
            .focus
            .saturating_add(1)
            .min(self.stops().len().saturating_sub(1));
    }

    pub(crate) fn previous(&mut self) {
        self.focus = self.focus.saturating_sub(1);
    }

    /// Brings the focus and the open sheet's error lines in line with the model.
    fn sync(&mut self) {
        self.focus = self.focus.min(self.stops().len().saturating_sub(1));
        if let Some(Overlay::Form(form) | Overlay::ConfirmPassword { form, .. }) = &mut self.overlay
        {
            for (input, field) in form.inputs.iter_mut().zip(form.sheet.fields()) {
                input.set_error(self.model.error(form.sheet, *field));
            }
        }
    }

    /// Queues `change` under this visit.
    fn change(&self, cx: &mut Context<'_>, ticket: Ticket, change: Change) {
        self.request(cx, Job::Change { ticket, change });
    }

    /// Queues `job` under this visit.
    fn request(&self, cx: &mut Context<'_>, job: Job) {
        cx.request(
            Request {
                visit: self.visit,
                job,
            }
            .into(),
        );
    }
}

/// What is open over the Account screen.
#[derive(Debug)]
pub(crate) enum Overlay {
    /// A Change sheet.
    Form(Form),
    /// The confirm before a password change, the sheet waiting under it.
    ConfirmPassword {
        form: Form,
        alert: AlertDialog,
    },
    LogOut(AlertDialog),
    /// The More sheet, with the Delete account dialog over it when open.
    More(Option<DeleteDialog>),
}

/// An alert for `dialog` with Cancel and its confirm, the confirm focused.
fn alert(dialog: Dialog) -> AlertDialog {
    let alert = AlertDialog::new(dialog.title(), dialog.body()).with_button(CANCEL, Some('n'));
    let alert = if dialog == Dialog::LogOut {
        alert.with_danger_button(dialog.confirm(), Some('y'))
    } else {
        alert.with_button(dialog.confirm(), Some('y'))
    };
    alert.with_focus(1)
}

/// The label over `field`'s box: New password's carries the `[?]` its `F2` opens.
pub(crate) fn label(field: Field) -> Label<'static> {
    if field == Field::NewPassword {
        Label::About(field.label())
    } else {
        Label::Plain(field.label())
    }
}

/// An open Change sheet: a box per field and which has the cursor. What is typed and the lines under the boxes are the model's.
#[derive(Debug)]
pub(crate) struct Form {
    pub(crate) sheet: Sheet,
    pub(crate) inputs: Vec<TextInput>,
    pub(crate) focus: usize,
}

impl Form {
    /// A sheet with every box empty, as each open is.
    pub(crate) fn new(sheet: Sheet) -> Self {
        let inputs: Vec<TextInput> = sheet
            .fields()
            .iter()
            .map(|field| {
                if field.is_masked() {
                    TextInput::masked()
                } else {
                    TextInput::default()
                }
            })
            .collect();
        Self {
            sheet,
            inputs,
            focus: 0,
        }
    }

    /// The field whose box has the cursor.
    pub(crate) fn field(&self) -> Option<Field> {
        self.sheet.fields().get(self.focus).copied()
    }

    pub(crate) fn next_field(&mut self) {
        self.focus = self
            .focus
            .saturating_add(1)
            .checked_rem(self.inputs.len())
            .unwrap_or(0);
    }

    pub(crate) fn previous_field(&mut self) {
        self.focus = self
            .focus
            .checked_sub(1)
            .unwrap_or_else(|| self.inputs.len().saturating_sub(1));
    }

    pub(crate) fn on_last_field(&self) -> bool {
        self.focus.saturating_add(1) >= self.inputs.len()
    }
}

/// Where the Delete account dialog's focus is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DeleteFocus {
    Password,
    ShowPassword,
    Cancel,
    Delete,
}

/// The Delete account dialog: its password box and its focus. The countdown and whether the delete is out are the model's.
#[derive(Debug)]
pub(crate) struct DeleteDialog {
    pub(crate) password: TextInput,
    pub(crate) focus: DeleteFocus,
}

impl DeleteDialog {
    /// A dialog with an empty password.
    pub(crate) fn new() -> Self {
        Self {
            password: TextInput::masked(),
            focus: DeleteFocus::Password,
        }
    }
}

/// Applies `action` to the model, brings the view in line with it, and performs the effects it returns, in order.
pub(crate) fn dispatch(view: &mut AccountView, action: Action, cx: &mut Context<'_>) {
    let effects = view.model.update(action);
    view.sync();
    for effect in effects {
        perform_effect(view, effect, cx);
    }
    view.sync();
}

fn perform_effect(view: &mut AccountView, effect: Effect, cx: &mut Context<'_>) {
    match effect {
        Effect::OfferHint(offer) => {
            hint::once(cx, offer.key());
        }
        Effect::FetchUser(ticket) => view.request(cx, Job::FetchUser { ticket }),
        Effect::CheckVerification(ticket) => view.request(cx, Job::CheckVerification { ticket }),
        Effect::ResendVerification(ticket) => view.request(cx, Job::Resend { ticket }),
        Effect::Wait(ticket) => view.request(cx, Job::Wait { ticket }),
        Effect::OpenSheet(sheet) => view.overlay = Some(Overlay::Form(Form::new(sheet))),
        Effect::CloseSheet(sheet) => {
            if let Some(Overlay::Form(form) | Overlay::ConfirmPassword { form, .. }) = &view.overlay
                && form.sheet == sheet
            {
                view.overlay = None;
            }
        }
        Effect::ChangeUsername {
            ticket,
            username,
            password,
        } => view.change(cx, ticket, Change::Username { username, password }),
        Effect::ChangeEmail {
            ticket,
            email,
            password,
        } => view.change(cx, ticket, Change::Email { email, password }),
        Effect::ChangePassword {
            ticket,
            current,
            new,
        } => view.change(cx, ticket, Change::Password { current, new }),
        Effect::DeleteAccount { ticket, password } => {
            view.change(cx, ticket, Change::Delete { password });
        }
        Effect::OpenDialog(Dialog::ConfirmPassword) => {
            view.overlay = match view.overlay.take() {
                Some(Overlay::Form(form)) => Some(Overlay::ConfirmPassword {
                    form,
                    alert: alert(Dialog::ConfirmPassword),
                }),
                other => other,
            };
        }
        Effect::OpenDialog(Dialog::LogOut) => {
            view.overlay = Some(Overlay::LogOut(alert(Dialog::LogOut)));
        }
        Effect::OpenDialog(Dialog::DeleteAccount) => {
            view.overlay = Some(Overlay::More(Some(DeleteDialog::new())));
        }
        Effect::LogOut => cx.request(screens::Request::SignOut),
        Effect::OpenUrl(url) => browser::open(cx, &url),
        Effect::Navigate(Destination::Previous) => cx.back(),
        Effect::Navigate(Destination::PrivacyPolicy) => privacy::open(cx),
        Effect::Navigate(Destination::Changelog) => changelog::open(cx),
        Effect::Toast(toast) => cx.show(toast),
    }
}

/// The Account screen's work, stamped with the visit that asked for it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Request {
    pub(crate) visit: Visit,
    pub(crate) job: Job,
}

/// One piece of Account's work, each carrying its ticket back to the model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Job {
    /// Fetches the user with the session as it is, on every open. A failure says nothing.
    FetchUser {
        ticket: Ticket,
    },
    CheckVerification {
        ticket: Ticket,
    },
    Resend {
        ticket: Ticket,
    },
    /// Waits one [`TICK`] for a countdown.
    Wait {
        ticket: Ticket,
    },
    /// A change to the account, with the password the server checks it against.
    Change {
        ticket: Ticket,
        change: Change,
    },
}

/// A change to the account, the values as typed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Change {
    Username {
        username: String,
        password: Password,
    },
    Email {
        email: String,
        password: Password,
    },
    Password {
        current: Password,
        new: Password,
    },
    Delete {
        password: Password,
    },
}

impl Request {
    /// The action an error from this request is reported under, fixed so a renamed variant keeps its reports.
    pub(crate) fn action(&self) -> &'static str {
        match self.job {
            Job::FetchUser { .. } => "load_user",
            Job::CheckVerification { .. } => "check_verification",
            Job::Resend { .. } => "resend",
            Job::Wait { .. } => "wait",
            Job::Change { .. } => "change",
        }
    }
}

/// The Account screen's results, stamped with the visit that asked.
#[derive(Debug)]
pub(crate) struct Update {
    pub(crate) visit: Visit,
    pub(crate) reply: Reply,
}

/// One reply, carrying its request's ticket back to the model.
#[derive(Debug)]
pub(crate) enum Reply {
    /// A fresh copy of the user, from the open's fetch or the verification check.
    UserLoaded {
        ticket: Ticket,
        user: Box<User>,
    },
    /// The open's fetch failed, which says nothing.
    Quiet {
        ticket: Ticket,
    },
    /// A request failed with this message for an error toast.
    Failed {
        ticket: Ticket,
        error: String,
    },
    /// The verification email failed with this message for a long error toast.
    ResendFailed {
        ticket: Ticket,
        error: String,
    },
    Resent {
        ticket: Ticket,
    },
    /// The server asked to wait before another verification email.
    ResendLimited {
        ticket: Ticket,
    },
    Tick {
        ticket: Ticket,
    },
    UsernameChanged {
        ticket: Ticket,
        user: Box<User>,
    },
    EmailChanged {
        ticket: Ticket,
        user: Box<User>,
    },
    PasswordChanged {
        ticket: Ticket,
    },
    /// The account is gone and the session with it.
    Deleted {
        ticket: Ticket,
    },
}

impl From<Request> for screens::Request {
    fn from(request: Request) -> Self {
        Self::Account(request)
    }
}

impl From<Update> for screens::Update {
    fn from(update: Update) -> Self {
        Self::Account(update)
    }
}

fn pick(screen: &mut Screen) -> Option<&mut AccountView> {
    match screen {
        Screen::Account(view) => Some(view),
        _ => None,
    }
}

/// Opens Account over the current screen on the user as last known; the model fetches a fresh copy and offers the `profile` hint.
pub(crate) fn open(cx: &mut Context<'_>) {
    let Some(user) = cx.shared.user.clone() else {
        return;
    };
    let mut view = AccountView::new();
    dispatch(&mut view, Action::Load(Some(Box::new(user))), cx);
    cx.push(Screen::Account(Box::new(view)));
}

pub(crate) async fn perform(account: &Account, request: Request) -> Option<screens::Update> {
    let Request { visit, job } = request;
    let reply = match job {
        Job::Wait { ticket } => {
            tokio::time::sleep(TICK).await;
            Reply::Tick { ticket }
        }
        Job::FetchUser { ticket } => {
            // The app fetches with the session as it stands and says nothing when that fails.
            let fetched = match account.session().await {
                Some(session) => account.client.fetch_user(&session).await.ok(),
                None => None,
            };
            match fetched {
                Some(user) => {
                    account.set_user(user.clone()).await;
                    Reply::UserLoaded {
                        ticket,
                        user: Box::new(user),
                    }
                }
                None => Reply::Quiet { ticket },
            }
        }
        Job::CheckVerification { ticket } => {
            match async { account.client.fetch_user(&account.fresh().await?).await }.await {
                Ok(user) => {
                    account.set_user(user.clone()).await;
                    Reply::UserLoaded {
                        ticket,
                        user: Box::new(user),
                    }
                }
                Err(e) => return Some(failure(visit, &e, |error| Reply::Failed { ticket, error })),
            }
        }
        Job::Resend { ticket } => {
            let failed = |error| Reply::ResendFailed { ticket, error };
            let session = match account.fresh().await {
                Ok(session) => session,
                Err(e) => return Some(failure(visit, &e, failed)),
            };
            match account.client.resend_verification(&session).await {
                Ok(()) => Reply::Resent { ticket },
                Err(ClientError::TooManyRequests(_)) => Reply::ResendLimited { ticket },
                Err(e) => return Some(failure(visit, &e, failed)),
            }
        }
        Job::Change { ticket, change } => {
            return Some(change_account(account, visit, ticket, change).await);
        }
    };
    Some(Update { visit, reply }.into())
}

/// The update for a failed request: the session ending when the server rejects it, else `reply` with the message to show.
fn failure(
    visit: Visit,
    error: &ClientError,
    reply: impl FnOnce(String) -> Reply,
) -> screens::Update {
    screens::failure(error, |error| {
        Update {
            visit,
            reply: reply(error),
        }
        .into()
    })
}

/// Sends an account change. A wrong password is the server's to judge, so its refusal is a toast rather than a sign-out.
async fn change_account(
    account: &Account,
    visit: Visit,
    ticket: Ticket,
    change: Change,
) -> screens::Update {
    let client = &account.client;
    let session = match account.fresh().await {
        Ok(session) => session,
        Err(e) => return failure(visit, &e, |error| Reply::Failed { ticket, error }),
    };
    let failed = |e: &ClientError| Reply::Failed {
        ticket,
        error: screens::shown(e),
    };
    let reply = match change {
        Change::Username { username, password } => match client
            .change_username(
                HttpChangeUsername::new(&username, password.read()),
                &session,
            )
            .await
        {
            Ok(user) => {
                account.set_user(user.clone()).await;
                Reply::UsernameChanged {
                    ticket,
                    user: Box::new(user),
                }
            }
            Err(e) => failed(&e),
        },
        Change::Email { email, password } => match client
            .change_email(HttpChangeEmail::new(&email, password.read()), &session)
            .await
        {
            Ok(user) => {
                account.set_user(user.clone()).await;
                Reply::EmailChanged {
                    ticket,
                    user: Box::new(user),
                }
            }
            Err(e) => failed(&e),
        },
        Change::Password { current, new } => match client
            .change_password(
                HttpChangePassword::new(current.read(), new.read()),
                &session,
            )
            .await
        {
            Ok(()) => Reply::PasswordChanged { ticket },
            Err(e) => failed(&e),
        },
        Change::Delete { password } => {
            let request = HttpDeleteUser {
                password: Secret::new(password.read()),
            };
            match client.delete_user(request, &session).await {
                Ok(()) => {
                    account.sign_out().await;
                    Reply::Deleted { ticket }
                }
                Err(e) => failed(&e),
            }
        }
    };
    Update { visit, reply }.into()
}

/// Hands a reply to the model when its visit is still up. Toasts and the user's fresh copy land even when it has closed.
pub(crate) fn apply(nav: &mut Nav, update: Update, cx: &mut Context<'_>) {
    let Update { visit, reply } = update;
    let deleted = matches!(reply, Reply::Deleted { .. });
    let action = match reply {
        Reply::UserLoaded { ticket, user } => {
            take_user(cx, &user);
            Action::UserLoaded { ticket, user }
        }
        Reply::Quiet { ticket } => Action::Failed { ticket },
        Reply::Failed { ticket, error } => {
            cx.toast(ToastKind::Error, error);
            Action::Failed { ticket }
        }
        Reply::ResendFailed { ticket, error } => {
            cx.toast_for(ToastKind::Error, error, toast::LONG);
            Action::Failed { ticket }
        }
        Reply::Resent { ticket } => Action::Resent { ticket },
        Reply::ResendLimited { ticket } => Action::ResendLimited { ticket },
        Reply::Tick { ticket } => Action::Tick { ticket },
        Reply::UsernameChanged { ticket, user } => {
            take_user(cx, &user);
            Action::UsernameChanged {
                ticket,
                username: user.username,
            }
        }
        Reply::EmailChanged { ticket, user } => {
            take_user(cx, &user);
            Action::EmailChanged { ticket, user }
        }
        Reply::PasswordChanged { ticket } => Action::PasswordChanged { ticket },
        Reply::Deleted { ticket } => Action::Deleted { ticket },
    };
    match nav.find_mut(pick) {
        Some(view) if view.visit == visit => dispatch(view, action, cx),
        // The session is already gone; with no model to log out, go to sign-in here.
        _ if deleted => login::signed_out(cx, None),
        _ => {}
    }
}

/// Takes a fresh copy of the user for the rest of the app.
fn take_user(cx: &mut Context<'_>, user: &User) {
    cx.shared.username = Some(user.username.to_string());
    cx.shared.user = Some(user.clone());
}
