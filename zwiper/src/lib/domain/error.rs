//! User-facing error message formatting.
//!
//! Converts technical validation errors into friendly, actionable messages for end users.
//! Particularly useful for form validation feedback in the UI.

/// Trait for converting errors into user-friendly error messages.
///
/// Implementations should provide clear, actionable error messages that help users
/// understand what went wrong and how to fix it.
///
/// # Example
///
/// ```rust,ignore
/// use zwiper::domain::error::UserFacing;
///
/// let result = EmailAddress::parse("invalid-email");
/// if let Err(error) = result {
///     let message = error.to_user_facing_string();
///     // Display: "missing @ symbol" instead of "MissingSeparator"
/// }
/// ```
pub trait UserFacing {
    /// Converts this error into a user-friendly error message.
    ///
    /// Should return clear, actionable text suitable for display in UI error messages.
    fn to_user_facing_string(&self) -> String;
}

/// Converts email validation errors into user-friendly messages.
///
/// Wraps the technical `email_address::Error` variants with plain language
/// explanations suitable for form validation feedback.
impl UserFacing for email_address::Error {
    fn to_user_facing_string(&self) -> String {
        match self {
            email_address::Error::InvalidCharacter => "invalid character in email".to_string(),
            email_address::Error::MissingSeparator => "missing @ symbol".to_string(),
            email_address::Error::LocalPartEmpty => "missing text before @".to_string(),
            email_address::Error::LocalPartTooLong => "text before @ is too long".to_string(),
            email_address::Error::DomainEmpty => "missing domain after @".to_string(),
            email_address::Error::DomainTooLong => "domain is too long".to_string(),
            email_address::Error::SubDomainEmpty => "empty part in domain".to_string(),
            email_address::Error::SubDomainTooLong => {
                "a part of the domain is too long".to_string()
            }
            email_address::Error::DomainTooFew => "domain must have at least one dot".to_string(),
            email_address::Error::DomainInvalidSeparator => {
                "invalid dot placement in domain".to_string()
            }
            email_address::Error::UnbalancedQuotes => "unbalanced quotes in email".to_string(),
            email_address::Error::InvalidComment => "invalid comment in email".to_string(),
            email_address::Error::InvalidIPAddress => "invalid ip address in domain".to_string(),
            email_address::Error::UnsupportedDomainLiteral => {
                "domain literal not supported".to_string()
            }
            email_address::Error::UnsupportedDisplayName => {
                "display name not supported".to_string()
            }
            email_address::Error::MissingDisplayName => "missing display name".to_string(),
            email_address::Error::MissingEndBracket => "missing closing bracket".to_string(),
        }
    }
}
