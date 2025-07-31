use argon2::{
    password_hash::{rand_core::OsRng, Error, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};

pub fn hash_password(password: &str) -> Result<String, Error> {
    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|x| x.to_string())
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, Error> {
    let argon2 = Argon2::default();
    let parsed_hash = PasswordHash::new(hash)?;

    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(()) => Ok(true),
        Err(Error::Password) => Ok(false),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hash() {
        // with some salt the same password
        // should always generate a unique hash
        assert_ne!(hash_password("test"), hash_password("test"));
    }

    #[test]
    fn verify() {
        let hash = hash_password("test").expect("failed to hash");
        assert!(verify_password("test", &hash).expect("failed to verify"));
    }
}
