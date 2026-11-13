//! zerver-admin: account changes made from the server only, never from the app.
//!
//! Every command names an account by its exact username, its id or its exact email, and goes
//! through zerver's own services, as the app's requests do. It reads zerver's settings from the
//! environment or `zerver/.env`. `zerver-admin --help` lists the commands and
//! `zerver-admin <command> --help` their options.
//!
//! - `show <account>` prints the account at a glance and changes nothing.
//! - `rename <account> <new> [--no-notice] [--reply-to <address>]` renames the account the way a
//!   user's own change does (the username rules, a name no other account holds in any case, the
//!   `username_changed` audit row) but without the password or the reserved-name check, which is
//!   how Scotty claims a reserved name. It then mails the account, from the support address, that
//!   its old name is now reserved, with replies going to the server's support address or
//!   `--reply-to`, unless `--no-notice`.
//! - `sign-out-all <account>` revokes every refresh token the account holds.
//! - `resend-verification <account>` sends a fresh verification email, as the app's resend does.
//! - `delete <account>` deletes the account and everything it owns, as the user's own delete does.
//!
//! A command that takes something away asks for the account's username to be typed back before
//! it acts; `--yes` skips that and `--dry-run` says what it would do and changes nothing.

use std::io::{self, BufRead, Write};
use std::sync::Arc;

use anyhow::Context;
use clap::{Args, Parser, Subcommand};
use zwipe::config::Config;
use zwipe::domain::auth::models::account_ref::AccountRef;
use zwipe::domain::auth::ports::AuthService;
use zwipe::domain::user::models::account_summary::AccountSummary;
use zwipe::domain::{auth, metrics, user};
use zwipe::inbound::admin::{self, NoticeOutcome, RenameNotice};
use zwipe::outbound::resend::Resend;
use zwipe::outbound::sqlx::postgres::Postgres;
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
    /// Revoke every refresh token an account holds
    SignOutAll(DestructiveArgs),
    /// Send an account a fresh verification email
    ResendVerification(AccountArgs),
    /// Delete an account and everything it owns
    Delete(DestructiveArgs),
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

/// Whether a command that takes something away goes ahead.
#[derive(Debug, PartialEq, Eq)]
enum Go {
    /// It acts.
    Ahead,
    /// It stops, having said why.
    Stop,
}

/// Asks for `user`'s username back before `action`, unless `--yes`; under `--dry-run` says what
/// would happen and stops. A reply that is not the username stops.
///
/// # Errors
///
/// An I/O error reading the reply or writing the prompt.
fn confirm(
    args: ConfirmArgs,
    user: &User,
    action: &str,
    input: &mut impl BufRead,
    output: &mut impl Write,
) -> io::Result<Go> {
    let name = &user.username;
    if args.dry_run {
        writeln!(
            output,
            "dry run: would {action} {name} ({}); nothing changed",
            user.id
        )?;
        return Ok(Go::Stop);
    }
    if args.yes {
        return Ok(Go::Ahead);
    }
    write!(output, "Type {name} to {action} it: ")?;
    output.flush()?;
    let mut reply = String::new();
    input.read_line(&mut reply)?;
    if reply.trim_end_matches(['\r', '\n']) == &**name {
        return Ok(Go::Ahead);
    }
    writeln!(output, "that is not {name}; nothing changed")?;
    Ok(Go::Stop)
}

/// The account at a glance, one fact per line.
fn summary_lines(summary: &AccountSummary) -> Vec<String> {
    let user = &summary.user;
    let verified = user
        .email_verified_at
        .map_or_else(|| "no".to_owned(), |at| format!("yes, {at}"));
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

/// Prints the command's report.
#[expect(
    clippy::print_stdout,
    reason = "a command-line tool reports what it changed"
)]
fn report(line: &str) {
    println!("{line}");
}

/// The account `account` names.
///
/// # Errors
///
/// No such account, or a database failure.
async fn fetch(auth: &impl AuthService, account: &AccountRef) -> anyhow::Result<User> {
    auth.fetch_account(account)
        .await
        .with_context(|| format!("finding {account}"))
}

/// Asks on the terminal, as [`confirm`] does.
///
/// # Errors
///
/// An I/O error on the terminal.
fn confirm_on_terminal(args: ConfirmArgs, user: &User, action: &str) -> io::Result<Go> {
    confirm(
        args,
        user,
        action,
        &mut io::stdin().lock(),
        &mut io::stdout(),
    )
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let Cli { command } = Cli::parse();

    let config = Config::from_env()?;
    let support = Email::new(&config.support_email_address)
        .context("SUPPORT_EMAIL_ADDRESS is not an email")?;
    let db = Postgres::new(&config.database_url).await?;
    let resend = Resend::new(config.resend_api_key, config.resend_from_email);
    let auth_service = auth::services::Service::new(
        db.clone(),
        db.clone(),
        Arc::new(resend),
        config.jwt_secret,
        config.access_token_lifetime,
        config.web_base_url,
        config.support_email_address,
    );
    let user_service = user::services::Service::new(db.clone());
    let metrics_service = metrics::services::Service::new(db);

    match command {
        Command::Show(AccountArgs { account }) => {
            let user = fetch(&auth_service, &account).await?;
            let summary = admin::show_account(&user_service, &user).await?;
            for line in summary_lines(&summary) {
                report(&line);
            }
        }
        Command::Rename(rename) => {
            let notice = rename.notice(support);
            let renamed = admin::rename_account(
                &auth_service,
                &metrics_service,
                &rename.account,
                &rename.new_username,
                notice,
            )
            .await
            .with_context(|| format!("renaming {} to {}", rename.account, rename.new_username))?;
            let notice = match &renamed.notice {
                NoticeOutcome::Sent => format!("notice sent to {}", renamed.user.email),
                NoticeOutcome::Skipped => "no notice sent".to_owned(),
                NoticeOutcome::Failed(e) => {
                    format!("the rename stands, but the notice failed: {e}")
                }
            };
            report(&format!(
                "{} is now {} ({}); {notice}",
                renamed.old_username, renamed.user.username, renamed.user.id
            ));
        }
        Command::SignOutAll(DestructiveArgs { account, confirm }) => {
            let user = fetch(&auth_service, &account).await?;
            if confirm_on_terminal(confirm, &user, "sign out everywhere")? == Go::Ahead {
                admin::sign_out_all(&auth_service, &metrics_service, &user).await?;
                report(&format!(
                    "{} ({}) is signed out everywhere",
                    user.username, user.id
                ));
            }
        }
        Command::ResendVerification(AccountArgs { account }) => {
            let user = fetch(&auth_service, &account).await?;
            admin::resend_verification(&auth_service, &metrics_service, &user).await?;
            report(&format!(
                "verification email sent to {} for {}",
                user.email, user.username
            ));
        }
        Command::Delete(DestructiveArgs { account, confirm }) => {
            let user = fetch(&auth_service, &account).await?;
            if confirm_on_terminal(confirm, &user, "delete")? == Go::Ahead {
                admin::delete_account(&auth_service, &user).await?;
                report(&format!("{} ({}) is deleted", user.username, user.id));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use clap::CommandFactory;
    use clap::error::ErrorKind;
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

    fn destructive(args: &[&str]) -> DestructiveArgs {
        match command(args).unwrap() {
            Command::SignOutAll(args) | Command::Delete(args) => args,
            other => panic!("not destructive: {other:?}"),
        }
    }

    fn support() -> Email {
        Email::new("support@zwipe.net").unwrap()
    }

    fn user() -> User {
        User {
            id: uuid::Uuid::nil(),
            username: Username::new("dummy").unwrap(),
            email: Email::new("dummy@test.local").unwrap(),
            email_verified_at: None,
            hints_shown: std::collections::HashMap::new(),
            pending_email: None,
            pending_email_expires_at: None,
        }
    }

    /// Runs [`confirm`] with `reply` typed in, answering what it decided and what it printed.
    fn confirm_with(args: ConfirmArgs, reply: &str) -> (Go, String) {
        let mut output = Vec::new();
        let go = confirm(args, &user(), "delete", &mut reply.as_bytes(), &mut output).unwrap();
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
        let err = command(&["unknown-command", "Test"]).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidSubcommand, "{err}");
    }

    #[test]
    fn missing_arguments_are_refused() {
        for args in [
            &["rename", "Test"][..],
            &["rename"][..],
            &["show"][..],
            &["sign-out-all"][..],
            &["resend-verification"][..],
            &["delete", "--yes"][..],
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
    fn reply_to_needs_an_email() {
        let err = parse(&["rename", "Test", "newname", "--reply-to"]).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidValue, "{err}");
        let err = parse(&["rename", "Test", "newname", "--reply-to", "nobody"]).unwrap_err();
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
            "sign-out-all",
            "resend-verification",
            "delete",
        ] {
            assert!(help.contains(name), "{name} in:\n{help}");
        }

        let err = command(&["rename", "--help"]).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::DisplayHelp);
        let help = err.to_string();
        for part in [
            "--no-notice",
            "--reply-to <ADDRESS>",
            "SUPPORT_EMAIL_ADDRESS",
            SUPPORT_EMAIL,
        ] {
            assert!(help.contains(part), "{part} in:\n{help}");
        }

        let err = command(&["delete", "--help"]).unwrap_err();
        let help = err.to_string();
        for part in ["--yes", "--dry-run"] {
            assert!(help.contains(part), "{part} in:\n{help}");
        }
    }

    #[test]
    fn each_command_reads_its_account_and_flags() {
        assert!(matches!(
            command(&["show", "dummy"]).unwrap(),
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
        let plain = destructive(&["delete", "dummy"]);
        assert!(!plain.confirm.yes && !plain.confirm.dry_run);
        let both = destructive(&["sign-out-all", "dummy", "--yes", "--dry-run"]);
        assert!(both.confirm.yes && both.confirm.dry_run);
    }

    #[test]
    fn a_destructive_command_goes_ahead_only_on_the_typed_username() {
        let (go, output) = confirm_with(ASK, "dummy\n");
        assert_eq!(go, Go::Ahead);
        assert!(output.contains("Type dummy to delete it"), "{output}");

        for reply in ["Dummy\n", "dumm\n", "\n", "", "dummy extra\n"] {
            let (go, output) = confirm_with(ASK, reply);
            assert_eq!(go, Go::Stop, "{reply:?}");
            assert!(output.contains("nothing changed"), "{reply:?}: {output}");
        }
    }

    #[test]
    fn yes_skips_the_question_and_dry_run_always_stops() {
        let yes = ConfirmArgs {
            yes: true,
            dry_run: false,
        };
        let (go, output) = confirm_with(yes, "");
        assert_eq!(go, Go::Ahead);
        assert!(output.is_empty(), "{output}");

        for yes in [false, true] {
            let dry = ConfirmArgs { yes, dry_run: true };
            let (go, output) = confirm_with(dry, "dummy\n");
            assert_eq!(go, Go::Stop);
            assert!(
                output.contains("dry run: would delete dummy")
                    && output.contains("nothing changed"),
                "{output}"
            );
        }
    }

    #[test]
    fn the_summary_names_every_fact() {
        let at = Utc.with_ymd_and_hms(2026, 11, 12, 9, 30, 0).unwrap();
        let summary = AccountSummary {
            user: user(),
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
            "username     dummy",
            "email        dummy@test.local",
            "verified     no",
            "last active  never",
            "decks        3",
            "username_changed",
        ] {
            assert!(text.contains(part), "{part} in:\n{text}");
        }
    }
}
