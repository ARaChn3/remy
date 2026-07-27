/*
// Author: Aliasgar Khimani (NovusEdge)
// Project: github.com/ARaChn3/remy
//
// Copyright: MIT License
// See the LICENSE file for more info.
*/

//! Cryptographic utilities for secure communication.
//!
//! Uses AES-256-GCM for authenticated encryption. All messages between
//! the C2 server and implants are encrypted to avoid detection by
//! network monitoring tools.

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rand::RngCore;

/// Length of AES-256 key in bytes
const KEY_LEN: usize = 32;

/// Length of GCM nonce in bytes
const NONCE_LEN: usize = 12;

/// Handles encryption and decryption of messages.
///
/// Uses AES-256-GCM which provides both confidentiality and authenticity.
/// The nonce is prepended to the ciphertext for easy transmission.
#[derive(Clone)]
pub struct Crypto {
    /// The AES-GCM cipher instance
    cipher: Aes256Gcm,
}

impl Crypto {
    /// Creates a new Crypto instance with the given key.
    ///
    /// # Arguments
    /// * `key` - 32-byte AES-256 key
    ///
    /// # Panics
    /// Panics if key is not exactly 32 bytes.
    pub fn new(key: &[u8]) -> Self {
        assert_eq!(key.len(), KEY_LEN, "Key must be 32 bytes for AES-256");
        let cipher = Aes256Gcm::new_from_slice(key).expect("Invalid key length");
        Self { cipher }
    }

    /// Generates a random 32-byte key.
    ///
    /// Uses the OS random number generator for cryptographic security.
    pub fn generate_key() -> [u8; KEY_LEN] {
        let mut key = [0u8; KEY_LEN];
        OsRng.fill_bytes(&mut key);
        key
    }

    /// Encrypts plaintext and returns base64-encoded ciphertext.
    ///
    /// The nonce is randomly generated and prepended to the ciphertext.
    /// Format: base64(nonce || ciphertext)
    ///
    /// # Arguments
    /// * `plaintext` - Data to encrypt
    ///
    /// # Returns
    /// Base64-encoded string containing nonce + ciphertext
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<String, CryptoError> {
        // Generate random nonce
        let mut nonce_bytes = [0u8; NONCE_LEN];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt the plaintext
        let ciphertext = self
            .cipher
            .encrypt(nonce, plaintext)
            .map_err(|_| CryptoError::EncryptionFailed)?;

        // Prepend nonce to ciphertext and encode
        let mut result = nonce_bytes.to_vec();
        result.extend(ciphertext);

        Ok(BASE64.encode(result))
    }

    /// Decrypts base64-encoded ciphertext.
    ///
    /// Expects format: base64(nonce || ciphertext)
    ///
    /// # Arguments
    /// * `ciphertext` - Base64-encoded encrypted data
    ///
    /// # Returns
    /// Decrypted plaintext bytes
    pub fn decrypt(&self, ciphertext: &str) -> Result<Vec<u8>, CryptoError> {
        // Decode from base64
        let data = BASE64
            .decode(ciphertext)
            .map_err(|_| CryptoError::InvalidBase64)?;

        // Split nonce and ciphertext
        if data.len() < NONCE_LEN {
            return Err(CryptoError::InvalidCiphertext);
        }
        let (nonce_bytes, ciphertext) = data.split_at(NONCE_LEN);
        let nonce = Nonce::from_slice(nonce_bytes);

        // Decrypt
        self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| CryptoError::DecryptionFailed)
    }
}

/// Errors that can occur during cryptographic operations.
#[derive(Debug, Clone)]
pub enum CryptoError {
    /// Failed to encrypt data
    EncryptionFailed,
    /// Failed to decrypt data (wrong key or corrupted)
    DecryptionFailed,
    /// Input was not valid base64
    InvalidBase64,
    /// Ciphertext too short to contain nonce
    InvalidCiphertext,
}

impl std::fmt::Display for CryptoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EncryptionFailed => write!(f, "Encryption failed"),
            Self::DecryptionFailed => write!(f, "Decryption failed"),
            Self::InvalidBase64 => write!(f, "Invalid base64 encoding"),
            Self::InvalidCiphertext => write!(f, "Invalid ciphertext format"),
        }
    }
}

impl std::error::Error for CryptoError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = Crypto::generate_key();
        let crypto = Crypto::new(&key);

        let plaintext = b"Hello, World!";
        let encrypted = crypto.encrypt(plaintext).unwrap();
        let decrypted = crypto.decrypt(&encrypted).unwrap();

        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_wrong_key_fails() {
        let key1 = Crypto::generate_key();
        let key2 = Crypto::generate_key();

        let crypto1 = Crypto::new(&key1);
        let crypto2 = Crypto::new(&key2);

        let encrypted = crypto1.encrypt(b"secret").unwrap();
        assert!(crypto2.decrypt(&encrypted).is_err());
    }
}
