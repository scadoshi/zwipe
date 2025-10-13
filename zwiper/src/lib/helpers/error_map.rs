pub trait UserFacing {
    fn to_user_facing_string(&self) -> String;
}

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
