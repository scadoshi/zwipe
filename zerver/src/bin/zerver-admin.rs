//! zerver-admin: account changes made from the server only, never from the app.
//!
//! `zerver-admin rename <account> <new> [--no-notice] [--reply-to <address>]` renames the account
//! whose exact username, id or email is `<account>` to `<new>`, the way a user's own change does
//! (the username rules, a name no other account holds in any case, the `username_changed` audit
//! row) but without the password or the reserved-name check, which is how Scotty claims a reserved
//! name. It then mails the account, from the support address, that its old name is now reserved,
//! with replies going to the server's support address or `--reply-to`, unless `--no-notice`. It
//! reads zerver's settings from the environment or `zerver/.env`. `zerver-admin --help` lists the
//! commands.

use std::sync::Arc;

use anyhow::Context;
use clap::{Args, Parser, Subcommand};
use zwipe::config::Config;
use zwipe::domain::auth::models::account_ref::AccountRef;
use zwipe::domain::{auth, metrics};
use zwipe::inbound::admin::{self, NoticeOutcome, RenameNotice};
use zwipe::outbound::resend::Resend;
use zwipe::outbound::sqlx::postgres::Postgres;
use zwipe_core::domain::Email;
use zwipe_core::domain::site::SUPPORT_EMAIL;
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
    /// Rename an account without its password or the reserved-name check, and mail it a notice
    Rename(RenameArgs),
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let Cli {
        command: Command::Rename(rename),
    } = Cli::parse();

    let config = Config::from_env()?;
    let support = Email::new(&config.support_email_address)
        .context("SUPPORT_EMAIL_ADDRESS is not an email")?;
    let notice = rename.notice(support);
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
    let metrics_service = metrics::services::Service::new(db);

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
        NoticeOutcome::Failed(e) => format!("the rename stands, but the notice failed: {e}"),
    };
    #[expect(
        clippy::print_stdout,
        reason = "a command-line tool reports what it changed"
    )]
    {
        println!(
            "{} is now {} ({}); {notice}",
            renamed.old_username, renamed.user.username, renamed.user.id
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;
    use clap::error::ErrorKind;

    use super::*;

    fn parse(args: &[&str]) -> Result<RenameArgs, clap::Error> {
        let cli = Cli::try_parse_from(std::iter::once("zerver-admin").chain(args.iter().copied()))?;
        let Command::Rename(rename) = cli.command;
        Ok(rename)
    }

    fn support() -> Email {
        Email::new("support@zwipe.net").unwrap()
    }

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
    }

    #[test]
    fn missing_arguments_are_refused() {
        for args in [&["rename", "Test"][..], &["rename"][..]] {
            let err = parse(args).unwrap_err();
            assert_eq!(
                err.kind(),
                ErrorKind::MissingRequiredArgument,
                "{args:?}: {err}"
            );
        }
        let err = parse(&[]).unwrap_err();
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
    fn help_lists_rename_and_the_reply_to_default() {
        let err = parse(&["--help"]).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::DisplayHelp);
        assert!(err.to_string().contains("rename"), "{err}");

        let err = parse(&["rename", "--help"]).unwrap_err();
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
    }
}
