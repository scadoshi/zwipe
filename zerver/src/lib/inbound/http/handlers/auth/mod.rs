//! Authentication and account management handlers.

pub mod authenticate_user;
pub mod change_email;
pub mod change_password;
pub mod change_username;
pub mod check_reset_token;
pub mod confirm_email_change;
pub mod cookie_session;
pub mod delete_user;
pub mod refresh_session;
pub mod register_user;
pub mod request_password_reset;
pub mod resend_verification;
pub mod reset_password;
pub mod revoke_sessions;
pub mod verify_email;

pub use authenticate_user::authenticate_user;
pub use change_email::change_email;
pub use change_password::change_password;
pub use change_username::change_username;
pub use check_reset_token::check_reset_token;
pub use confirm_email_change::confirm_email_change;
pub use cookie_session::{cookie_login, cookie_logout, cookie_refresh};
pub use delete_user::delete_user;
pub use refresh_session::refresh_session;
pub use register_user::register_user;
pub use request_password_reset::request_password_reset;
pub use resend_verification::resend_verification;
pub use reset_password::reset_password;
pub use revoke_sessions::revoke_sessions;
pub use verify_email::verify_email;
