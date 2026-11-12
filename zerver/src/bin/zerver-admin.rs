//! zerver-admin: account changes made from the server only, never from the app.
//!
//! `zerver-admin rename <account> <new> [--no-notice] [--reply-to <address>]` renames the account
//! whose exact username, id or email is `<account>` to `<new>`, the way a user's own change does
//! (the username rules, a name no other account holds in any case, the `username_changed` audit
//! row) but without the password or the reserved-name check, which is how Scotty claims a reserved
//! name. It then mails the account, from the support address, that its old name is now reserved,
//! unless `--no-notice`. It reads zerver's settings from the environment or `zerver/.env`.

use std::env;
use std::sync::Arc;

use anyhow::{Context, bail};
use zwipe::config::Config;
use zwipe::domain::{auth, metrics};
use zwipe::inbound::admin::{self, NoticeOutcome, RenameNotice};
use zwipe::outbound::resend::Resend;
use zwipe::outbound::sqlx::postgres::Postgres;
use zwipe_core::domain::user::username::Username;

/// How to call it.
const USAGE: &str = "usage: zerver-admin rename <username|id|email> <new-username> [--no-notice] [--reply-to <address>]";

/// What the command line asks for.
struct Rename {
    /// The account to rename.
    account: String,
    /// Its new name.
    new_username: Username,
    /// Whether and how to tell the account.
    notice: RenameNotice,
}

/// Reads `rename <account> <new> [--no-notice] [--reply-to <address>]`.
fn parse(args: &[String]) -> anyhow::Result<Rename> {
    let [command, account, new, flags @ ..] = args else {
        bail!("{USAGE}");
    };
    if command != "rename" {
        bail!("{USAGE}");
    }
    let new_username = Username::new(new).context("the new username")?;
    let mut send = true;
    let mut reply_to = None;
    let mut rest = flags.iter();
    while let Some(flag) = rest.next() {
        match flag.as_str() {
            "--no-notice" => send = false,
            "--reply-to" => {
                let address = rest.next().context("--reply-to takes an address")?;
                reply_to = Some(address.clone());
            }
            other => bail!("unknown option {other}\n{USAGE}"),
        }
    }
    let notice = if send {
        RenameNotice::Send { reply_to }
    } else {
        RenameNotice::Skip
    };
    Ok(Rename {
        account: account.clone(),
        new_username,
        notice,
    })
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    let Rename {
        account,
        new_username,
        notice,
    } = parse(&args)?;

    let config = Config::from_env()?;
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
        &account,
        &new_username,
        notice,
    )
    .await
    .with_context(|| format!("renaming {account} to {new_username}"))?;

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
