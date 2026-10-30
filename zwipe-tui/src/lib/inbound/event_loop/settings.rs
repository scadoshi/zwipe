//! The account and config screens' keys, results and requests.

use crossterm::event::KeyCode;
use zwipe_client::ClientError;
use zwipe_core::domain::auth::models::secret::Secret;
use zwipe_core::domain::card::scryfall_data::universe::UbFranchiseView;
use zwipe_core::domain::user::User;
use zwipe_core::domain::user::preferences::{PipStyle, UserPreferences};
use zwipe_core::http::contracts::auth::{
    HttpChangeEmail, HttpChangePassword, HttpChangeUsername, HttpDeleteUser,
};
use zwipe_core::http::contracts::user::HttpUpdatePreferences;

use super::{EventLoop, Flow, Password, Request, Update, failure};
use crate::domain::theme::Palette;
use crate::inbound::app::{
    AccountRow, AccountView, App, ConfigRow, Dialog, ExceptionsEditor, Form, FormKind,
    PreferenceChange, Screen, ThemePicker,
};
use crate::outbound::account::Account;
use crate::outbound::theme_file::RememberedTheme;

/// The account and config screens' requests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum SettingsRequest {
    /// Fetches the signed-in user.
    LoadUser,
    /// Sends the verification email again.
    Resend,
    /// Fetches the user to see whether the email is verified yet.
    CheckVerification,
    ChangeAccount(AccountChange),
    LoadPreferences,
    SavePreferences {
        seq: u64,
        change: PreferenceChange,
    },
    /// Fetches the franchises the exceptions can name.
    LoadFranchises,
}

/// A change to the account, with the password the server checks it against.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum AccountChange {
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

impl EventLoop {
    fn push_settings(&mut self, request: SettingsRequest) {
        self.pending.push(Request::Settings(request));
    }

    /// Opens the account screen on the user as last known, and fetches a fresh copy.
    pub(super) fn open_account(&mut self, app: &mut App) {
        let Some(user) = app.user.clone() else {
            return;
        };
        app.status.clear();
        app.screen = Screen::Account(Box::new(AccountView::new(user)));
        self.push_settings(SettingsRequest::LoadUser);
    }

    /// Opens the config screen and fetches the preferences it shows.
    pub(super) fn open_config(&mut self, app: &mut App) {
        app.status.clear();
        app.screen = Screen::Config(Box::default());
        self.push_settings(SettingsRequest::LoadPreferences);
    }

    /// Draws in `preferences` from now on and remembers their theme for the next launch.
    pub(super) fn use_preferences(&mut self, app: &mut App, preferences: &UserPreferences) {
        app.palette = Palette::for_preferences(preferences);
        self.pending.push(Request::RememberTheme(RememberedTheme {
            theme: preferences.theme.clone(),
            dark_mode: preferences.dark_mode,
        }));
    }

    pub(super) fn on_user(app: &mut App, user: User, message: Option<String>) {
        app.username = Some(user.username.to_string());
        if let Screen::Account(view) = &mut app.screen {
            view.set_user(user.clone());
        }
        app.user = Some(user);
        if let Some(message) = message {
            app.status = message;
        }
    }

    pub(super) fn on_account_changed(app: &mut App, user: Option<User>, message: String) {
        if let Screen::Account(view) = &mut app.screen {
            view.dialog = None;
        }
        match user {
            Some(user) => Self::on_user(app, user, Some(message)),
            None => app.status = message,
        }
    }

    pub(super) fn on_account_change_failed(app: &mut App, error: String) {
        let Screen::Account(view) = &mut app.screen else {
            return;
        };
        let form = match view.dialog.take() {
            Some(Dialog::Form(form) | Dialog::ConfirmPassword(form)) => Some(form),
            other => {
                view.dialog = other;
                None
            }
        };
        if let Some(mut form) = form {
            form.busy = false;
            form.error = Some(error);
            view.dialog = Some(Dialog::Form(form));
        }
    }

    pub(super) fn on_resend_failed(app: &mut App, error: String) {
        if let Screen::Account(view) = &mut app.screen {
            view.resend_ready_at = None;
        }
        app.status = error;
    }

    /// Takes the preferences the server has: draws in them, and fills the config screen when it is waiting for them.
    pub(super) fn on_preferences(&mut self, app: &mut App, preferences: UserPreferences) {
        if let Screen::Config(view) = &mut app.screen {
            if view.saved.is_some() {
                return;
            }
            view.saved = Some(preferences.clone());
            view.shown = Some(preferences.clone());
        }
        self.use_preferences(app, &preferences);
    }

    pub(super) fn on_preferences_saved(
        &mut self,
        app: &mut App,
        seq: u64,
        preferences: UserPreferences,
        message: &str,
    ) {
        let Screen::Config(view) = &mut app.screen else {
            return;
        };
        if view.confirm(seq, preferences.clone()) {
            message.clone_into(&mut app.status);
            self.use_preferences(app, &preferences);
        }
    }

    pub(super) fn on_preferences_failed(app: &mut App, seq: u64, error: String) {
        let Screen::Config(view) = &mut app.screen else {
            return;
        };
        if view.revert(seq) {
            if let Some(shown) = &view.shown {
                app.palette = Palette::for_preferences(shown);
            }
            app.status = error;
        }
    }

    pub(super) fn on_franchises(app: &mut App, franchises: Vec<UbFranchiseView>) {
        if let Screen::Config(view) = &mut app.screen
            && let Some(editor) = &mut view.exceptions
        {
            editor.franchises = Some(franchises);
        }
    }

    pub(super) fn handle_account_key(&mut self, app: &mut App, code: KeyCode) -> Flow {
        let Screen::Account(view) = &mut app.screen else {
            return Flow::Continue;
        };
        match view.dialog.take() {
            Some(Dialog::Form(form)) => view.dialog = self.handle_form_key(form, code),
            Some(Dialog::ConfirmPassword(mut form)) => match code {
                KeyCode::Enter | KeyCode::Char('y') => {
                    form.busy = true;
                    self.push_settings(SettingsRequest::ChangeAccount(AccountChange::Password {
                        current: Password(form.value(0).to_owned()),
                        new: Password(form.value(1).to_owned()),
                    }));
                    view.dialog = Some(Dialog::ConfirmPassword(form));
                }
                KeyCode::Esc | KeyCode::Char('n') => view.dialog = Some(Dialog::Form(form)),
                _ => view.dialog = Some(Dialog::ConfirmPassword(form)),
            },
            Some(Dialog::LogOut) => match code {
                KeyCode::Enter | KeyCode::Char('y') => {
                    "Logging out...".clone_into(&mut app.status);
                    self.pending.push(Request::SignOut);
                }
                KeyCode::Esc | KeyCode::Char('n') => {}
                _ => view.dialog = Some(Dialog::LogOut),
            },
            None => match code {
                KeyCode::Char('q') => return Flow::Quit,
                KeyCode::Esc => self.open_home(app),
                KeyCode::Down | KeyCode::Char('j') => view.next_row(),
                KeyCode::Up | KeyCode::Char('k') => view.previous_row(),
                KeyCode::Enter => {
                    if let Some(row) = view.selected() {
                        self.activate_account_row(app, row);
                    }
                }
                _ => {}
            },
        }
        Flow::Continue
    }

    fn activate_account_row(&mut self, app: &mut App, row: AccountRow) {
        let Screen::Account(view) = &mut app.screen else {
            return;
        };
        match row {
            AccountRow::Username => view.dialog = Some(Dialog::Form(Form::new(FormKind::Username))),
            AccountRow::Email => view.dialog = Some(Dialog::Form(Form::new(FormKind::Email))),
            AccountRow::Password => view.dialog = Some(Dialog::Form(Form::new(FormKind::Password))),
            AccountRow::Delete => view.dialog = Some(Dialog::Form(Form::new(FormKind::Delete))),
            AccountRow::LogOut => view.dialog = Some(Dialog::LogOut),
            AccountRow::Resend => {
                if view.resend_wait().is_none() {
                    view.start_resend_cooldown();
                    "Sending...".clone_into(&mut app.status);
                    self.push_settings(SettingsRequest::Resend);
                }
            }
            AccountRow::Check => {
                "Checking...".clone_into(&mut app.status);
                self.push_settings(SettingsRequest::CheckVerification);
            }
        }
    }

    /// Keys in an open form. Returns what stays open: the form, its password confirmation, or nothing.
    fn handle_form_key(&mut self, mut form: Form, code: KeyCode) -> Option<Dialog> {
        if form.busy {
            return Some(Dialog::Form(form));
        }
        match code {
            KeyCode::Esc => return None,
            KeyCode::Tab | KeyCode::Down => form.next_field(),
            KeyCode::BackTab | KeyCode::Up => form.previous_field(),
            KeyCode::Backspace => {
                if let Some(field) = form.focused_mut() {
                    field.pop();
                }
            }
            KeyCode::Char(c) => {
                if let Some(field) = form.focused_mut() {
                    field.push(c);
                }
            }
            KeyCode::Enter if !form.on_last_field() => form.next_field(),
            KeyCode::Enter => return Some(self.submit_form(form)),
            _ => {}
        }
        Some(Dialog::Form(form))
    }

    fn submit_form(&mut self, mut form: Form) -> Dialog {
        if form.delete_wait().is_some() {
            return Dialog::Form(form);
        }
        if let Some(problem) = form.problem() {
            form.error = Some(problem);
            return Dialog::Form(form);
        }
        form.error = None;
        let password = |field: usize| Password(form.value(field).to_owned());
        let change = match form.kind {
            FormKind::Password => return Dialog::ConfirmPassword(form),
            FormKind::Username => AccountChange::Username {
                username: form.value(0).trim().to_owned(),
                password: password(1),
            },
            FormKind::Email => AccountChange::Email {
                email: form.value(0).trim().to_owned(),
                password: password(1),
            },
            FormKind::Delete => AccountChange::Delete {
                password: password(0),
            },
        };
        form.busy = true;
        self.push_settings(SettingsRequest::ChangeAccount(change));
        Dialog::Form(form)
    }

    pub(super) fn handle_config_key(&mut self, app: &mut App, code: KeyCode) -> Flow {
        let Screen::Config(view) = &mut app.screen else {
            return Flow::Continue;
        };
        if let Some(mut picker) = view.picker.take() {
            self.handle_picker_key(app, &mut picker, code);
            return Flow::Continue;
        }
        if let Some(editor) = view.exceptions.take() {
            let editor = self.handle_exceptions_key(app, editor, code);
            if let Screen::Config(view) = &mut app.screen {
                view.exceptions = editor;
            }
            return Flow::Continue;
        }
        match code {
            KeyCode::Char('q') => return Flow::Quit,
            KeyCode::Esc => self.open_home(app),
            KeyCode::Down | KeyCode::Char('j') => view.next_row(),
            KeyCode::Up | KeyCode::Char('k') => view.previous_row(),
            KeyCode::Enter | KeyCode::Char(' ') => self.activate_config_row(app),
            _ => {}
        }
        Flow::Continue
    }

    fn activate_config_row(&mut self, app: &mut App) {
        let Screen::Config(view) = &mut app.screen else {
            return;
        };
        let Some(shown) = view.shown.clone() else {
            return;
        };
        let change = match view.selected() {
            ConfigRow::Theme => {
                view.picker = Some(ThemePicker::opening(&shown.theme));
                return;
            }
            ConfigRow::Exceptions => {
                if view.exceptions_open() {
                    view.exceptions = Some(ExceptionsEditor::opening(
                        &shown.universes_beyond_exceptions,
                    ));
                    self.push_settings(SettingsRequest::LoadFranchises);
                }
                return;
            }
            ConfigRow::Mode => PreferenceChange::DarkMode {
                theme: shown.theme.clone(),
                dark_mode: !shown.dark_mode,
            },
            ConfigRow::Pips => PreferenceChange::Pips(match shown.pip_style {
                PipStyle::Theme => PipStyle::Natural,
                PipStyle::Natural => PipStyle::Theme,
            }),
            ConfigRow::UniversesBeyond => {
                PreferenceChange::HideUniversesBeyond(!shown.exclude_universes_beyond)
            }
        };
        self.change_preference(app, change);
    }

    /// Shows `change` at once and sends it; a failure puts the last saved preferences back.
    fn change_preference(&mut self, app: &mut App, change: PreferenceChange) {
        let Screen::Config(view) = &mut app.screen else {
            return;
        };
        let Some(seq) = view.begin(&change) else {
            return;
        };
        if let Some(shown) = &view.shown {
            app.palette = Palette::for_preferences(shown);
        }
        self.push_settings(SettingsRequest::SavePreferences { seq, change });
    }

    /// Keys in the theme list. Moving previews the theme; Enter saves it; Esc puts the old one back.
    fn handle_picker_key(&mut self, app: &mut App, picker: &mut ThemePicker, code: KeyCode) {
        let Screen::Config(view) = &mut app.screen else {
            return;
        };
        let Some(shown) = view.shown.clone() else {
            return;
        };
        let preview = |theme: &str| {
            let mut preferences = shown.clone();
            theme.clone_into(&mut preferences.theme);
            Palette::for_preferences(&preferences)
        };
        match code {
            KeyCode::Down | KeyCode::Char('j') => picker.down(),
            KeyCode::Up | KeyCode::Char('k') => picker.up(),
            KeyCode::Enter => {
                if let Some(theme) = picker.theme()
                    && theme != picker.original
                {
                    self.change_preference(
                        app,
                        PreferenceChange::Theme {
                            theme: theme.to_owned(),
                            dark_mode: shown.dark_mode,
                        },
                    );
                }
                return;
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                app.palette = Palette::for_preferences(&shown);
                if picker.theme() != Some(picker.original.as_str()) {
                    "Theme unchanged".clone_into(&mut app.status);
                }
                return;
            }
            _ => {}
        }
        if let Some(theme) = picker.theme() {
            app.palette = preview(theme);
        }
        if let Screen::Config(view) = &mut app.screen {
            view.picker = Some(ThemePicker {
                row: picker.row,
                original: picker.original.clone(),
            });
        }
    }

    /// Keys in the exceptions list. Returns the editor while it stays open.
    fn handle_exceptions_key(
        &mut self,
        app: &mut App,
        mut editor: ExceptionsEditor,
        code: KeyCode,
    ) -> Option<ExceptionsEditor> {
        match code {
            KeyCode::Down | KeyCode::Char('j') => editor.down(),
            KeyCode::Up | KeyCode::Char('k') => editor.up(),
            KeyCode::Char(' ') => editor.toggle(),
            KeyCode::Char('x') => editor.draft.clear(),
            KeyCode::Enter => {
                if editor.changed() {
                    self.change_preference(app, PreferenceChange::Exceptions(editor.draft));
                }
                return None;
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                if editor.changed() {
                    "Exceptions unchanged".clone_into(&mut app.status);
                }
                return None;
            }
            _ => {}
        }
        Some(editor)
    }
}

impl From<PreferenceChange> for HttpUpdatePreferences {
    fn from(change: PreferenceChange) -> Self {
        let mut request = Self {
            theme: None,
            dark_mode: None,
            exclude_universes_beyond: None,
            universes_beyond_exceptions: None,
            pip_style: None,
        };
        match change {
            PreferenceChange::Theme { theme, dark_mode }
            | PreferenceChange::DarkMode { theme, dark_mode } => {
                request.theme = Some(theme);
                request.dark_mode = Some(dark_mode);
            }
            PreferenceChange::Pips(style) => request.pip_style = Some(style),
            PreferenceChange::HideUniversesBeyond(hide) => {
                request.exclude_universes_beyond = Some(hide);
            }
            PreferenceChange::Exceptions(slugs) => {
                request.universes_beyond_exceptions = Some(slugs);
            }
        }
        request
    }
}

/// Runs one of the account and config screens' requests.
pub(super) async fn perform(account: &Account, request: SettingsRequest) -> Option<Update> {
    let client = &account.client;
    match request {
        SettingsRequest::LoadUser => {
            match async { client.fetch_user(&account.fresh().await?).await }.await {
                Ok(user) => {
                    account.set_user(user.clone()).await;
                    Some(Update::User {
                        user: Box::new(user),
                        message: None,
                    })
                }
                Err(e) => Some(failure(&e, Update::Status)),
            }
        }
        SettingsRequest::CheckVerification => {
            match async { client.fetch_user(&account.fresh().await?).await }.await {
                Ok(user) => {
                    account.set_user(user.clone()).await;
                    let message = if user.email_verified_at.is_some() {
                        "Email verified"
                    } else {
                        "Not verified yet. Check your inbox"
                    };
                    Some(Update::User {
                        user: Box::new(user),
                        message: Some(message.to_owned()),
                    })
                }
                Err(e) => Some(failure(&e, Update::Status)),
            }
        }
        SettingsRequest::Resend => match account.fresh().await {
            Err(e) => Some(failure(&e, Update::ResendFailed)),
            Ok(session) => Some(match client.resend_verification(&session).await {
                Ok(()) => Update::Status("Verification email sent".to_owned()),
                Err(ClientError::TooManyRequests(_)) => {
                    Update::Status("Please wait a moment".to_owned())
                }
                Err(e) => failure(&e, Update::ResendFailed),
            }),
        },
        SettingsRequest::ChangeAccount(change) => Some(change_account(account, change).await),
        SettingsRequest::LoadPreferences => {
            match async { client.fetch_preferences(&account.fresh().await?).await }.await {
                Ok(preferences) => {
                    account.set_preferences(preferences.clone()).await;
                    Some(Update::Preferences(Box::new(preferences)))
                }
                // A launch that cannot fetch the preferences keeps the session's copy.
                Err(e) => Some(failure(&e, |_| Update::PreferencesUnavailable)),
            }
        }
        SettingsRequest::SavePreferences { seq, change } => {
            let message = change.saved_message();
            let result = async {
                let session = account.fresh().await?;
                client
                    .update_preferences(HttpUpdatePreferences::from(change), &session)
                    .await
            }
            .await;
            Some(match result {
                Ok(preferences) => {
                    account.set_preferences(preferences.clone()).await;
                    Update::PreferencesSaved {
                        seq,
                        preferences: Box::new(preferences),
                        message,
                    }
                }
                Err(e) => failure(&e, |message| Update::PreferencesFailed { seq, message }),
            })
        }
        SettingsRequest::LoadFranchises => client
            .fetch_ub_franchises()
            .await
            .ok()
            .map(Update::Franchises),
    }
}

/// Sends an account change. The password is the server's to judge, so its refusal is the form's error rather than a sign-out.
async fn change_account(account: &Account, change: AccountChange) -> Update {
    let client = &account.client;
    let session = match account.fresh().await {
        Ok(session) => session,
        Err(e) => return failure(&e, Update::AccountChangeFailed),
    };
    let result = match change {
        AccountChange::Username { username, password } => client
            .change_username(HttpChangeUsername::new(&username, &password.0), &session)
            .await
            .map(|user| {
                let message = format!("Username changed to {}", user.username);
                (Some(user), message)
            }),
        AccountChange::Email { email, password } => client
            .change_email(HttpChangeEmail::new(&email, &password.0), &session)
            .await
            .map(|user| {
                (
                    Some(user),
                    format!("Check {email} for a link to confirm the change"),
                )
            }),
        AccountChange::Password { current, new } => client
            .change_password(HttpChangePassword::new(&current.0, &new.0), &session)
            .await
            .map(|()| (None, "Password change successful".to_owned())),
        AccountChange::Delete { password } => {
            let request = HttpDeleteUser {
                password: Secret::new(&password.0),
            };
            return match client.delete_user(request, &session).await {
                Ok(()) => {
                    account.forget().await;
                    Update::AccountDeleted
                }
                Err(e) => Update::AccountChangeFailed(e.to_user_message()),
            };
        }
    };
    match result {
        Ok((user, message)) => {
            if let Some(user) = &user {
                account.set_user(user.clone()).await;
            }
            Update::AccountChanged {
                user: user.map(Box::new),
                message,
            }
        }
        Err(e) => Update::AccountChangeFailed(e.to_user_message()),
    }
}
