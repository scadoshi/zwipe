use argon2::{
    password_hash::{rand_core::OsRng, Error as ArgonError, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};

pub fn hash_password(password: &str) -> Result<String, ArgonError> {
    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|x| x.to_string())
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, ArgonError> {
    let argon2 = Argon2::default();
    let parsed_hash = PasswordHash::new(hash)?;

    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(()) => Ok(true),
        Err(ArgonError::Password) => Ok(false),
        Err(e) => Err(e),
    }
}
