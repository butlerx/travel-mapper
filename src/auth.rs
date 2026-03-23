//! Encryption and password-hashing helpers — AES-256-GCM token encryption and
//! Argon2 password verification.

use aes_gcm::{
    Aes256Gcm, Key, Nonce,
    aead::{Aead, AeadCore, KeyInit, OsRng},
};
use argon2::{
    Argon2, PasswordHash, PasswordVerifier,
    password_hash::{PasswordHasher, SaltString},
};

#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("encryption failed")]
    Encrypt,

    #[error("decryption failed")]
    Decrypt,

    #[error("invalid nonce length")]
    InvalidNonceLength,

    #[error("decrypted token is not valid UTF-8")]
    InvalidUtf8(#[from] std::string::FromUtf8Error),
}

/// Encrypt a plaintext token with AES-256-GCM, returning `(ciphertext, nonce)`.
///
/// # Errors
///
/// Returns [`CryptoError::Encrypt`] if encryption fails.
pub fn encrypt_token(plaintext: &str, key: &[u8; 32]) -> Result<(Vec<u8>, Vec<u8>), CryptoError> {
    let key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_bytes())
        .map_err(|_| CryptoError::Encrypt)?;
    Ok((ciphertext, nonce.to_vec()))
}

/// Decrypt a ciphertext back to its original token string.
///
/// # Errors
///
/// Returns [`CryptoError::InvalidNonceLength`] if the nonce is not 12 bytes,
/// [`CryptoError::Decrypt`] if decryption fails, or [`CryptoError::InvalidUtf8`]
/// if the decrypted bytes are not valid UTF-8.
pub fn decrypt_token(
    ciphertext: &[u8],
    nonce: &[u8],
    key: &[u8; 32],
) -> Result<String, CryptoError> {
    if nonce.len() != 12 {
        return Err(CryptoError::InvalidNonceLength);
    }

    let key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce);
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| CryptoError::Decrypt)?;
    Ok(String::from_utf8(plaintext)?)
}

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
