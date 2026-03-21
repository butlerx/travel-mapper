use argon2::{
    Argon2, PasswordHash, PasswordVerifier,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};

/// Hash a plaintext password using Argon2.
///
/// # Errors
///
/// Returns an error if hashing fails (e.g. invalid parameters or RNG failure).
pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);
    let hash = argon2.hash_password(password.as_bytes(), &salt)?;
    Ok(hash.to_string())
}

/// Verify a plaintext password against an Argon2 hash.
///
/// # Errors
///
/// Returns an error if the hash is malformed or verification fails.
pub fn verify_password(password: &str, hash: &str) -> Result<bool, argon2::password_hash::Error> {
    let argon2 = Argon2::default();
    let parsed_hash = PasswordHash::new(hash)?;
    Ok(argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}
