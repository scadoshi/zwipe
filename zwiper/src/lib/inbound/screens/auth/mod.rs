//! Authentication screens.
//!
//! Provides login and registration page components.

/// Forgot password / reset initiation screen.
pub mod forgot_password;
/// User login screen.
pub mod login;
/// New user registration screen.
pub mod register;

#[cfg(test)]
mod tests {
    /// Each auth screen's source, by name.
    const SCREENS: [(&str, &str); 3] = [
        ("login", include_str!("login.rs")),
        ("register", include_str!("register.rs")),
        ("forgot_password", include_str!("forgot_password.rs")),
    ];

    #[test]
    fn an_auth_field_shows_its_label_once_with_no_placeholder() {
        for (name, source) in SCREENS {
            assert!(source.contains("label:"), "{name} labels its fields");
            assert!(
                !source.contains("placeholder:"),
                "{name} repeats a label as the field's placeholder"
            );
        }
    }
}
