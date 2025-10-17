use std::str::FromStr;

use chrono::{Duration, Utc};
use email_address::EmailAddress;
use uuid::Uuid;
use zwipe::domain::{
    auth::models::{
        access_token::{AccessToken, Jwt},
        refresh_token::RefreshToken,
        session::Session,
    },
    user::models::{username::Username, User},
};

pub trait Spoof {
    fn spoof() -> Self;
}

impl Spoof for Session {
    fn spoof() -> Self
    where
        Self: Sized,
    {
        let user = User {
            id: Uuid::new_v4(),
            username: Username::new("pedro").unwrap(),
            email: EmailAddress::from_str("pleebo@dog.net").unwrap(),
        };

        let refresh_token = RefreshToken {
            value: "123".to_string(),
            expires_at: (Utc::now() + Duration::days(123)).naive_utc(),
        };

        let access_token = AccessToken {
            jwt: Jwt::from_str("123.456.789").unwrap(),
            expires_at: (Utc::now() + Duration::days(123)).naive_utc(),
        };

        Self {
            user,
            refresh_token,
            access_token,
        }
    }
}
