//! zerver-admin: account changes made from the server only, never from the app.
//!
//! Every command names an account by its exact username, its id or its exact email, and goes
//! through zerver's own services, as the app's requests do. Each one that runs writes an
//! `admin_action_log` row. It reads zerver's settings from the environment or `zerver/.env`.
//! `zerver-admin --help` lists the commands and `zerver-admin <command> --help` their options.
//!
//! - `show <account>` prints the account at a glance and changes nothing.
//! - `rename <account> <new> [--no-notice] [--reply-to <address>]` renames the account the way a
//!   user's own change does (the username rules, a name no other account holds in any case, the
//!   `username_changed` audit row) but without the password or the reserved-name check, which is
//!   how Scotty claims a reserved name. It then mails the account, from the support address, that
//!   its old name is now reserved, with replies going to the server's support address or
//!   `--reply-to`, unless `--no-notice`.
//! - `lock <account> [--reason <text>]` refuses the account's sign-in and session refresh until
//!   `unlock <account>`.
//! - `sign-out-all <account>` revokes every refresh token the account holds.
//! - `resend-verification <account>` sends a fresh verification email, as the app's resend does.
//! - `delete <account>` deletes the account and everything it owns, as the user's own delete does.
//! - `change-email <account> <new-email> [--reply-to <address>]` switches the address at once,
//!   mails the new one a verification link and tells the old one.
//!
//! A command that takes something away asks for the account's username to be typed back before
//! it acts; `--yes` skips that and `--dry-run` says what it would do and changes nothing. An error
//! prints as one `error:` line with its causes and exits non-zero.

use std::io::{self, BufRead, Write};
use std::process::ExitCode;
use std::sync::Arc;

use anyhow::{Context, bail};
use clap::{Args, Parser, Subcommand};
use zwipe::config::Config;
use zwipe::domain::auth::models::account_ref::AccountRef;
use zwipe::domain::auth::ports::AuthService;
use zwipe::domain::user::models::account_summary::AccountSummary;
use zwipe::domain::{auth, metrics, user};
use zwipe::inbound::admin::{self, NoticeOutcome, RenameNotice};
use zwipe::outbound::resend::Resend;
use zwipe::outbound::sqlx::postgres::Postgres;
use zwipe::wiring::{Auth, Metrics, Users};
use zwipe_core::domain::Email;
use zwipe_core::domain::site::SUPPORT_EMAIL;
use zwipe_core::domain::user::User;
use zwipe_core::domain::user::username::{InvalidUsername, Username};

/// Account changes made from the server only.
#[derive(Debug, Parser)]
#[command(name = "zerver-admin")]
struct Cli {
    /// What to do.
    #[command(subcommand)]
    command: Command,
}

/// One admin command.
#[derive(Debug, Subcommand)]
enum Command {
    /// Print an account at a glance; changes nothing
    Show(AccountArgs),
    /// Rename an account without its password or the reserved-name check, and mail it a notice
    Rename(RenameArgs),
    /// Refuse an account's sign-in and session refresh until it is unlocked
    Lock(LockArgs),
    /// Clear an account's lock
    Unlock(AccountArgs),
    /// Revoke every refresh token an account holds
    SignOutAll(DestructiveArgs),
    /// Send an account a fresh verification email
    ResendVerification(AccountArgs),
    /// Delete an account and everything it owns
    Delete(DestructiveArgs),
    /// Switch an account's email at once, verify the new one and tell the old one
    ChangeEmail(ChangeEmailArgs),
}

/// A command that names one account.
#[derive(Debug, Args)]
struct AccountArgs {
    /// The account: its exact username, its id or its exact email
    account: AccountRef,
}

/// A command that takes something away from one account.
#[derive(Debug, Args)]
struct DestructiveArgs {
    /// The account: its exact username, its id or its exact email
    account: AccountRef,
    /// How it confirms.
    #[command(flatten)]
    confirm: ConfirmArgs,
}

/// How a command that takes something away confirms.
#[derive(Debug, Clone, Copy, Args)]
struct ConfirmArgs {
    /// Act without typing the username back
    #[arg(long)]
    yes: bool,
    /// Say what would change, and change nothing
    #[arg(long)]
    dry_run: bool,
}

/// What `rename` asks for.
#[derive(Debug, Args)]
struct RenameArgs {
    /// The account: its exact username, its id or its exact email
    account: AccountRef,
    /// Its new username
    #[arg(value_parser = parse_username)]
    new_username: Username,
    /// Send no notice email
    #[arg(long)]
    no_notice: bool,
    /// Where replies to the notice go
    #[arg(long, value_name = "ADDRESS", help = reply_to_help())]
    reply_to: Option<Email>,
}

/// What `lock` asks for.
#[derive(Debug, Args)]
struct LockArgs {
    /// The account: its exact username, its id or its exact email
    account: AccountRef,
    /// Why, kept with the lock and shown by `show`
    #[arg(long, value_name = "TEXT")]
    reason: Option<String>,
    /// How it confirms.
    #[command(flatten)]
    confirm: ConfirmArgs,
}

/// What `change-email` asks for.
#[derive(Debug, Args)]
struct ChangeEmailArgs {
    /// The account: its exact username, its id or its exact email
    account: AccountRef,
    /// Its new email
    new_email: Email,
    /// Where replies to the old address's notice go
    #[arg(long, value_name = "ADDRESS", help = reply_to_help())]
    reply_to: Option<Email>,
    /// How it confirms.
    #[command(flatten)]
    confirm: ConfirmArgs,
}

/// The `--reply-to` help, naming the default the server fills in when it runs.
fn reply_to_help() -> String {
    format!(
        "Where replies to the notice go [default: the server's SUPPORT_EMAIL_ADDRESS, {SUPPORT_EMAIL} when unset]"
    )
}

/// Reads a username by the username rules.
fn parse_username(raw: &str) -> Result<Username, InvalidUsername> {
    Username::new(raw)
}

impl RenameArgs {
    /// The notice to send, replying to `--reply-to` or else to `support`.
    fn notice(&self, support: Email) -> RenameNotice {
        if self.no_notice {
            return RenameNotice::Skip;
        }
        RenameNotice::Send {
            reply_to: self.reply_to.clone().unwrap_or(support),
        }
    }
}

/// A change a command confirms before making it, worded for the prompt and the dry run.
#[derive(Debug)]
enum Change<'a> {
    /// `lock`.
    Lock,
    /// `sign-out-all`.
    SignOutAll,
    /// `delete`.
    Delete,
    /// `change-email`, to this address.
    Email(&'a Email),
}

impl Change<'_> {
    /// The change done to `who`: "sign zatest1 out everywhere", "delete it".
    fn phrase(&self, who: &str) -> String {
        match self {
            Self::Lock => format!("lock {who}"),
            Self::SignOutAll => format!("sign {who} out everywhere"),
            Self::Delete => format!("delete {who}"),
            Self::Email(email) => format!("move {who} to {email}"),
        }
    }
}

/// What the confirmation decided.
#[derive(Debug, PartialEq, Eq)]
enum Go {
    /// It acts.
    Ahead,
    /// `--dry-run`: it said what it would do and stops.
    DryRun,
    /// The reply was not the username; it stops with an error.
    Mismatch,
}

/// Asks for `user`'s username back before `change`, unless `--yes`; under `--dry-run` says what
/// would happen and stops.
///
/// # Errors
///
/// An I/O error reading the reply or writing the prompt.
fn confirm(
    args: ConfirmArgs,
    user: &User,
    change: &Change<'_>,
    input: &mut impl BufRead,
    output: &mut impl Write,
) -> io::Result<Go> {
    let name = &user.username;
    if args.dry_run {
        writeln!(
            output,
            "dry run: would {} ({}); nothing changed",
            change.phrase(name),
            user.id
        )?;
        return Ok(Go::DryRun);
    }
    if args.yes {
        return Ok(Go::Ahead);
    }
    write!(output, "Type {name} to {}: ", change.phrase("it"))?;
    output.flush()?;
    let mut reply = String::new();
    input.read_line(&mut reply)?;
    if reply.trim_end_matches(['\r', '\n']) == &**name {
        Ok(Go::Ahead)
    } else {
        Ok(Go::Mismatch)
    }
}

/// Confirms `change` on the terminal: `true` to go ahead, `false` after a dry run.
///
/// # Errors
///
/// A reply that is not the username, or an I/O error on the terminal.
fn confirmed(args: ConfirmArgs, user: &User, change: &Change<'_>) -> anyhow::Result<bool> {
    match confirm(
        args,
        user,
        change,
        &mut io::stdin().lock(),
        &mut io::stdout(),
    )? {
        Go::Ahead => Ok(true),
        Go::DryRun => Ok(false),
        Go::Mismatch => bail!("that is not {}; nothing changed", user.username),
    }
}

/// The account at a glance, one fact per line.
fn summary_lines(summary: &AccountSummary) -> Vec<String> {
    let user = &summary.user;
    let verified = user
        .email_verified_at
        .map_or_else(|| "no".to_owned(), |at| format!("yes, {at}"));
    let locked = summary.lock.as_ref().map_or_else(
        || "no".to_owned(),
        |lock| match &lock.reason {
            Some(reason) => format!("yes, since {} ({reason})", lock.locked_at),
            None => format!("yes, since {}", lock.locked_at),
        },
    );
    let last_active = summary
        .last_active_at
        .map_or_else(|| "never".to_owned(), |at| at.to_string());
    let mut lines = vec![
        format!("username     {}", user.username),
        format!("id           {}", user.id),
        format!("email        {}", user.email),
        format!("verified     {verified}"),
    ];
    if let Some(pending) = &user.pending_email {
        let until = user
            .pending_email_expires_at
            .map_or_else(String::new, |at| format!(" until {at}"));
        lines.push(format!("pending      {pending}{until}"));
    }
    lines.push(format!("locked       {locked}"));
    lines.push(format!("created      {}", summary.created_at));
    lines.push(format!("last active  {last_active}"));
    lines.push(format!("decks        {}", summary.deck_count));
    if summary.recent_audit.is_empty() {
        lines.push("audit        none".to_owned());
    } else {
        lines.push("audit, newest first:".to_owned());
        for entry in &summary.recent_audit {
            lines.push(format!("  {}  {}", entry.occurred_at, entry.action));
        }
    }
    lines
}

/// How an email a command sent did, for its report.
fn mailed(outcome: &NoticeOutcome, what: &str) -> String {
    match outcome {
        NoticeOutcome::Sent => format!("{what} sent"),
        NoticeOutcome::Skipped => format!("no {what} sent"),
        NoticeOutcome::Failed(e) => format!("the change stands, but the {what} failed: {e}"),
    }
}

/// Prints the command's report.
#[expect(
    clippy::print_stdout,
    reason = "a command-line tool reports what it changed"
)]
fn report(line: &str) {
    println!("{line}");
}

/// The services a command runs through.
struct Services {
    /// Accounts, sessions and their emails.
    auth: Auth,
    /// Profiles and the account summary.
    users: Users,
    /// The audit and admin action logs.
    metrics: Metrics,
    /// The server's support address, the default reply-to.
    support: Email,
}

impl Services {
    /// Connects with zerver's settings.
    ///
    /// # Errors
    ///
    /// A missing or bad setting, or no database.
    async fn connect() -> anyhow::Result<Self> {
        let config = Config::from_env()?;
        let support = Email::new(&config.support_email_address)
            .context("SUPPORT_EMAIL_ADDRESS is not an email")?;
        let db = Postgres::new(&config.database_url).await?;
        let resend = Resend::new(config.resend_api_key, config.resend_from_email);
        let auth = auth::services::Service::new(
            db.clone(),
            db.clone(),
            Arc::new(resend),
            config.jwt_secret,
            config.access_token_lifetime,
            config.web_base_url,
            config.support_email_address,
        );
        Ok(Self {
            auth,
            users: user::services::Service::new(db.clone()),
            metrics: metrics::services::Service::new(db),
            support,
        })
    }

    /// The account `account` names.
    ///
    /// # Errors
    ///
    /// No such account, or a database failure.
    async fn fetch(&self, account: &AccountRef) -> anyhow::Result<User> {
        Ok(self.auth.fetch_account(account).await?)
    }
}

/// Runs `command`.
///
/// # Errors
///
/// Whatever stopped it: a missing account, a refused change, a wrong confirmation, a failure.
async fn run(command: Command) -> anyhow::Result<()> {
    let s = Services::connect().await?;
    match command {
        Command::Show(AccountArgs { account }) => {
            let user = s.fetch(&account).await?;
            let summary = admin::show_account(&s.users, &s.metrics, &user).await?;
            for line in summary_lines(&summary) {
                report(&line);
            }
        }
        Command::Rename(rename) => {
            let user = s.fetch(&rename.account).await?;
            let notice = rename.notice(s.support.clone());
            let renamed =
                admin::rename_account(&s.auth, &s.metrics, &user, &rename.new_username, notice)
                    .await
                    .with_context(|| {
                        format!("renaming {} to {}", user.username, rename.new_username)
                    })?;
            report(&format!(
                "{} is now {} ({}); {}",
                renamed.old_username,
                renamed.user.username,
                renamed.user.id,
                mailed(&renamed.notice, "notice")
            ));
        }
        Command::Lock(LockArgs {
            account,
            reason,
            confirm,
        }) => {
            let user = s.fetch(&account).await?;
            if confirmed(confirm, &user, &Change::Lock)? {
                admin::lock_account(&s.auth, &s.metrics, &user, reason.as_deref()).await?;
                report(&format!("{} ({}) is locked", user.username, user.id));
            }
        }
        Command::Unlock(AccountArgs { account }) => {
            let user = s.fetch(&account).await?;
            admin::unlock_account(&s.auth, &s.metrics, &user).await?;
            report(&format!("{} ({}) is unlocked", user.username, user.id));
        }
        Command::SignOutAll(DestructiveArgs { account, confirm }) => {
            let user = s.fetch(&account).await?;
            if confirmed(confirm, &user, &Change::SignOutAll)? {
                admin::sign_out_all(&s.auth, &s.metrics, &user).await?;
                report(&format!(
                    "{} ({}) is signed out everywhere",
                    user.username, user.id
                ));
            }
        }
        Command::ResendVerification(AccountArgs { account }) => {
            let user = s.fetch(&account).await?;
            admin::resend_verification(&s.auth, &s.metrics, &user).await?;
            report(&format!(
                "verification email sent to {} for {}",
                user.email, user.username
            ));
        }
        Command::Delete(DestructiveArgs { account, confirm }) => {
            let user = s.fetch(&account).await?;
            if confirmed(confirm, &user, &Change::Delete)? {
                admin::delete_account(&s.auth, &s.metrics, &user).await?;
                report(&format!("{} ({}) is deleted", user.username, user.id));
            }
        }
        Command::ChangeEmail(ChangeEmailArgs {
            account,
            new_email,
            reply_to,
            confirm,
        }) => {
            let user = s.fetch(&account).await?;
            s.auth
                .check_admin_email_change(&user, &new_email)
                .await
                .with_context(|| format!("moving {} to {new_email}", user.username))?;
            if confirmed(confirm, &user, &Change::Email(&new_email))? {
                let reply_to = reply_to.unwrap_or_else(|| s.support.clone());
                let switched =
                    admin::change_email(&s.auth, &s.metrics, &user, &new_email, &reply_to)
                        .await
                        .with_context(|| format!("moving {} to {new_email}", user.username))?;
                report(&format!(
                    "{} is now at {} (was {}); {}; {}",
                    switched.user.username,
                    switched.user.email,
                    switched.old_email,
                    mailed(&switched.verification, "verification email"),
                    mailed(&switched.notice, "old address notice")
                ));
            }
        }
    }
    Ok(())
}

/// One line for `error`: the error and each cause after it, with no backtrace.
fn error_line(error: &anyhow::Error) -> String {
    format!("error: {error:#}")
}

#[tokio::main]
async fn main() -> ExitCode {
    let Cli { command } = Cli::parse();
    match run(command).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            #[expect(clippy::print_stderr, reason = "a command-line tool reports its error")]
            {
                eprintln!("{}", error_line(&e));
            }
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use clap::CommandFactory;
    use clap::error::ErrorKind;
    use zwipe::domain::auth::models::account_lock::AccountLock;
    use zwipe::domain::auth::models::account_ref::FetchAccountError;
    use zwipe::domain::user::models::account_summary::AuditEntry;

    use super::*;

    fn command(args: &[&str]) -> Result<Command, clap::Error> {
        Cli::try_parse_from(std::iter::once("zerver-admin").chain(args.iter().copied()))
            .map(|cli| cli.command)
    }

    fn parse(args: &[&str]) -> Result<RenameArgs, clap::Error> {
        match command(args)? {
            Command::Rename(rename) => Ok(rename),
            other => Err(clap::Error::raw(
                ErrorKind::InvalidSubcommand,
                format!("not a rename: {other:?}"),
            )),
        }
    }

    fn support() -> Email {
        Email::new("support@zwipe.net").unwrap()
    }

    fn user() -> User {
        User {
            id: uuid::Uuid::nil(),
            username: Username::new("zatest1").unwrap(),
            email: Email::new("dummy@test.local").unwrap(),
            email_verified_at: None,
            hints_shown: std::collections::HashMap::new(),
            pending_email: None,
            pending_email_expires_at: None,
        }
    }

    /// Runs [`confirm`] for `change` with `reply` typed in, answering what it decided and what it
    /// printed.
    fn confirm_with(args: ConfirmArgs, change: &Change<'_>, reply: &str) -> (Go, String) {
        let mut output = Vec::new();
        let go = confirm(args, &user(), change, &mut reply.as_bytes(), &mut output).unwrap();
        (go, String::from_utf8(output).unwrap())
    }

    const ASK: ConfirmArgs = ConfirmArgs {
        yes: false,
        dry_run: false,
    };

    #[test]
    fn the_command_line_is_well_formed() {
        Cli::command().debug_assert();
    }

    #[test]
    fn rename_reads_the_account_the_name_and_the_flags() {
        let rename = parse(&["rename", "Test", "naxol56196"]).unwrap();
        assert!(matches!(&rename.account, AccountRef::Username(name) if &**name == "Test"));
        assert_eq!(&*rename.new_username, "naxol56196");
        assert_eq!(
            rename.notice(support()),
            RenameNotice::Send {
                reply_to: support()
            },
            "replies go to the support address by default"
        );

        let rename = parse(&[
            "rename",
            "a@b.com",
            "newname",
            "--reply-to",
            "scotty@zwipe.net",
        ])
        .unwrap();
        assert!(matches!(rename.account, AccountRef::Email(_)));
        assert_eq!(
            rename.notice(support()),
            RenameNotice::Send {
                reply_to: Email::new("scotty@zwipe.net").unwrap()
            }
        );

        let id = uuid::Uuid::new_v4().to_string();
        let rename = parse(&["rename", &id, "newname", "--no-notice"]).unwrap();
        assert!(matches!(rename.account, AccountRef::Id(_)));
        assert_eq!(rename.notice(support()), RenameNotice::Skip);
    }

    #[test]
    fn an_unknown_flag_is_refused() {
        let err = parse(&["rename", "Test", "newname", "--dry-run"]).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::UnknownArgument, "{err}");
        let err = command(&["show", "Test", "--yes"]).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::UnknownArgument, "{err}");
        let err = command(&["unlock", "Test", "--reason", "x"]).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::UnknownArgument, "{err}");
        let err = command(&["unknown-command", "Test"]).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidSubcommand, "{err}");
    }

    #[test]
    fn missing_arguments_are_refused() {
        for args in [
            &["rename", "Test"][..],
            &["rename"][..],
            &["show"][..],
            &["lock"][..],
            &["unlock"][..],
            &["sign-out-all"][..],
            &["resend-verification"][..],
            &["delete", "--yes"][..],
            &["change-email", "Test"][..],
        ] {
            let err = command(args).unwrap_err();
            assert_eq!(
                err.kind(),
                ErrorKind::MissingRequiredArgument,
                "{args:?}: {err}"
            );
        }
        let err = command(&[]).unwrap_err();
        assert_eq!(
            err.kind(),
            ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand,
            "{err}"
        );
    }

    #[test]
    fn reply_to_and_the_new_email_need_an_email() {
        let err = parse(&["rename", "Test", "newname", "--reply-to"]).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidValue, "{err}");
        let err = parse(&["rename", "Test", "newname", "--reply-to", "nobody"]).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::ValueValidation, "{err}");
        let err = command(&["change-email", "Test", "nobody"]).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::ValueValidation, "{err}");
        let err =
            command(&["change-email", "Test", "new@test.local", "--reply-to", "x"]).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::ValueValidation, "{err}");
    }

    #[test]
    fn a_bad_new_username_is_refused() {
        let err = parse(&["rename", "Test", "ab"]).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::ValueValidation, "{err}");
    }

    #[test]
    fn help_lists_every_command_and_the_reply_to_default() {
        let err = command(&["--help"]).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::DisplayHelp);
        let help = err.to_string();
        for name in [
            "show",
            "rename",
            "lock",
            "unlock",
            "sign-out-all",
            "resend-verification",
            "delete",
            "change-email",
        ] {
            assert!(help.contains(name), "{name} in:\n{help}");
        }

        for sub in ["rename", "change-email"] {
            let err = command(&[sub, "--help"]).unwrap_err();
            assert_eq!(err.kind(), ErrorKind::DisplayHelp);
            let help = err.to_string();
            for part in [
                "--reply-to <ADDRESS>",
                "SUPPORT_EMAIL_ADDRESS",
                SUPPORT_EMAIL,
            ] {
                assert!(help.contains(part), "{sub}: {part} in:\n{help}");
            }
        }

        for sub in ["delete", "sign-out-all", "lock", "change-email"] {
            let help = command(&[sub, "--help"]).unwrap_err().to_string();
            for part in ["--yes", "--dry-run"] {
                assert!(help.contains(part), "{sub}: {part} in:\n{help}");
            }
        }
    }

    #[test]
    fn each_command_reads_its_account_and_flags() {
        assert!(matches!(
            command(&["show", "zatest1"]).unwrap(),
            Command::Show(AccountArgs {
                account: AccountRef::Username(_)
            })
        ));
        assert!(matches!(
            command(&["resend-verification", "dummy@test.local"]).unwrap(),
            Command::ResendVerification(AccountArgs {
                account: AccountRef::Email(_)
            })
        ));
        let Command::Delete(plain) = command(&["delete", "zatest1"]).unwrap() else {
            panic!("a delete");
        };
        assert!(!plain.confirm.yes && !plain.confirm.dry_run);
        let Command::SignOutAll(both) =
            command(&["sign-out-all", "zatest1", "--yes", "--dry-run"]).unwrap()
        else {
            panic!("a sign-out-all");
        };
        assert!(both.confirm.yes && both.confirm.dry_run);
        let Command::Lock(lock) = command(&["lock", "zatest1", "--reason", "spam"]).unwrap() else {
            panic!("a lock");
        };
        assert_eq!(lock.reason.as_deref(), Some("spam"));
        let Command::ChangeEmail(change) =
            command(&["change-email", "zatest1", "new@test.local"]).unwrap()
        else {
            panic!("a change-email");
        };
        assert_eq!(change.new_email.as_ref(), "new@test.local");
        assert!(change.reply_to.is_none(), "the support address fills in");
    }

    #[test]
    fn each_prompt_reads_as_a_sentence() {
        let new = Email::new("new@test.local").unwrap();
        for (change, prompt) in [
            (
                Change::SignOutAll,
                "Type zatest1 to sign it out everywhere: ",
            ),
            (Change::Delete, "Type zatest1 to delete it: "),
            (Change::Lock, "Type zatest1 to lock it: "),
            (
                Change::Email(&new),
                "Type zatest1 to move it to new@test.local: ",
            ),
        ] {
            let (go, output) = confirm_with(ASK, &change, "zatest1\n");
            assert_eq!(go, Go::Ahead);
            assert_eq!(output, prompt);
        }
    }

    #[test]
    fn a_reply_that_is_not_the_username_is_a_mismatch() {
        for reply in ["Zatest1\n", "zatest\n", "\n", "", "zatest1 extra\n"] {
            let (go, _) = confirm_with(ASK, &Change::Delete, reply);
            assert_eq!(go, Go::Mismatch, "{reply:?}");
        }
    }

    #[test]
    fn yes_skips_the_question_and_dry_run_always_stops() {
        let yes = ConfirmArgs {
            yes: true,
            dry_run: false,
        };
        let (go, output) = confirm_with(yes, &Change::Delete, "");
        assert_eq!(go, Go::Ahead);
        assert!(output.is_empty(), "{output}");

        for yes in [false, true] {
            let dry = ConfirmArgs { yes, dry_run: true };
            let (go, output) = confirm_with(dry, &Change::SignOutAll, "zatest1\n");
            assert_eq!(go, Go::DryRun);
            assert!(
                output.starts_with("dry run: would sign zatest1 out everywhere (")
                    && output.contains("nothing changed"),
                "{output}"
            );
        }
    }

    #[test]
    fn an_error_prints_as_one_line_with_its_causes() {
        let missing: anyhow::Error = FetchAccountError::NotFound("zatest1".parse().unwrap()).into();
        assert_eq!(error_line(&missing), "error: no account named zatest1");

        let wrapped = Err::<(), _>(missing)
            .context("renaming zatest1 to zatest2")
            .unwrap_err();
        let line = error_line(&wrapped);
        assert_eq!(
            line,
            "error: renaming zatest1 to zatest2: no account named zatest1"
        );
        assert!(!line.contains('\n'), "no backtrace: {line}");
    }

    #[test]
    fn the_summary_names_every_fact() {
        let at = Utc.with_ymd_and_hms(2026, 11, 12, 9, 30, 0).unwrap();
        let mut summary = AccountSummary {
            user: user(),
            lock: None,
            created_at: at,
            last_active_at: None,
            deck_count: 3,
            recent_audit: vec![AuditEntry {
                action: "username_changed".to_owned(),
                occurred_at: at,
            }],
        };
        let text = summary_lines(&summary).join("\n");
        for part in [
            "username     zatest1",
            "email        dummy@test.local",
            "verified     no",
            "locked       no",
            "last active  never",
            "decks        3",
            "username_changed",
        ] {
            assert!(text.contains(part), "{part} in:\n{text}");
        }
        summary.lock = Some(AccountLock {
            locked_at: at,
            reason: Some("spam".to_owned()),
        });
        let text = summary_lines(&summary).join("\n");
        assert!(
            text.contains("locked       yes, since 2026-11-12 09:30:00 UTC (spam)"),
            "{text}"
        );
    }
}
