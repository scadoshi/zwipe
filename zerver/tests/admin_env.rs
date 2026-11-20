//! `zerver-admin` reads only the settings its command needs: a command that sends no mail runs
//! with `DATABASE_URL` alone, and one that sends mail asks for the Resend settings too.
//!
//! Each test runs the built binary with an empty environment but for what it names, from a
//! directory with no `.env`. A test that reaches the accounts runs on `#[sqlx::test]`'s migrated
//! database. Requires `DATABASE_URL`: `set -a; source zerver/.env; set +a`.

#![expect(
    clippy::unwrap_used,
    reason = "a test: helpers outside #[test] fns unwrap, and a panic is the failure report"
)]

use std::path::PathBuf;
use std::process::{Command, Output};

use sqlx::{ConnectOptions, PgPool};

/// An account name no database holds.
const NOBODY: &str = "nobody_660_admin_env";

/// A directory of the test's own with no `.env` in it, so the binary sees only what it is given.
fn bare_dir() -> PathBuf {
    let dir = std::env::temp_dir().join("zerver-admin-env-test");
    std::fs::create_dir_all(&dir).unwrap();
    assert!(!dir.join(".env").exists(), "{} holds a .env", dir.display());
    dir
}

/// Runs `zerver-admin args` with only `vars` in its environment.
fn admin(args: &[&str], vars: &[(&str, String)]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_zerver-admin"))
        .args(args)
        .env_clear()
        .envs(vars.iter().map(|(k, v)| (*k, v.as_str())))
        .current_dir(bare_dir())
        .output()
        .unwrap()
}

/// The database the test suite runs against.
fn database_url() -> (&'static str, String) {
    ("DATABASE_URL", std::env::var("DATABASE_URL").unwrap())
}

/// The URL of `pool`'s database, migrated by `#[sqlx::test]`.
fn database_url_of(pool: &PgPool) -> (&'static str, String) {
    (
        "DATABASE_URL",
        pool.connect_options().to_url_lossy().to_string(),
    )
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[sqlx::test]
async fn a_command_without_mail_runs_on_the_database_url_alone(pool: PgPool) {
    let output = admin(&["show", NOBODY], &[database_url_of(&pool)]);
    let err = stderr(&output);
    assert!(!output.status.success());
    assert!(
        !err.contains("failed to get variable"),
        "show asked for a setting it does not need: {err}"
    );
    assert!(
        err.contains(NOBODY),
        "show reached the database and found no such account: {err}"
    );
}

#[sqlx::test]
async fn a_rename_without_a_notice_needs_no_mail_settings(pool: PgPool) {
    let output = admin(
        &["rename", NOBODY, "nobody_660_new", "--no-notice"],
        &[database_url_of(&pool)],
    );
    let err = stderr(&output);
    assert!(!err.contains("failed to get variable"), "{err}");
    assert!(err.contains(NOBODY), "{err}");
}

#[test]
fn a_command_that_sends_mail_asks_for_the_resend_settings() {
    for args in [
        &["resend-verification", NOBODY][..],
        &["change-email", NOBODY, "nobody@test.local", "--yes"],
        &["rename", NOBODY, "nobody_660_new"],
    ] {
        let output = admin(args, &[database_url()]);
        let err = stderr(&output);
        assert!(!output.status.success(), "{args:?}");
        assert!(
            err.contains("RESEND_API_KEY"),
            "{args:?} names the missing Resend setting: {err}"
        );
    }
}

#[test]
fn every_command_still_asks_for_the_database_url() {
    let output = admin(&["show", NOBODY], &[]);
    let err = stderr(&output);
    assert!(!output.status.success());
    assert!(err.contains("DATABASE_URL"), "{err}");
}
