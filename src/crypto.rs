use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Nonce,
};
use rand::RngCore;
use sha2::{Digest, Sha256};

pub const NONCE_LEN: usize = 12;

/// Derive a 32-byte symmetric encryption key from a human-readable room secret/passphrase.
pub fn derive_key(room_secret: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"tiktik-v1-kdf:");
    hasher.update(room_secret.as_bytes());
    let result = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&result);
    key
}

/// Encrypts plaintext bytes using ChaCha20-Poly1305 with a random 12-byte nonce.
/// Returns: `[12-byte nonce] + [ciphertext + 16-byte auth tag]`
pub fn encrypt(key: &[u8; 32], plaintext: &[u8]) -> Result<Vec<u8>, String> {
    let cipher = ChaCha20Poly1305::new(key.into());
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| format!("Encryption error: {:?}", e))?;

    let mut payload = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    payload.extend_from_slice(&nonce_bytes);
    payload.extend_from_slice(&ciphertext);
    Ok(payload)
}

/// Decrypts a payload containing `[12-byte nonce] + [ciphertext + tag]`.
pub fn decrypt(key: &[u8; 32], payload: &[u8]) -> Result<Vec<u8>, String> {
    if payload.len() < NONCE_LEN {
        return Err("Payload too short to contain nonce".into());
    }

    let (nonce_bytes, ciphertext) = payload.split_at(NONCE_LEN);
    let cipher = ChaCha20Poly1305::new(key.into());
    let nonce = Nonce::from_slice(nonce_bytes);

    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| "Decryption failed: invalid key or corrupted packet".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_roundtrip() {
        let key = derive_key("top-secret-room-123");
        let secret_msg = b"Hey, want to grab lunch at 12:30?";

        let encrypted = encrypt(&key, secret_msg).expect("encryption succeeds");
        assert_ne!(encrypted, secret_msg);

        let decrypted = decrypt(&key, &encrypted).expect("decryption succeeds");
        assert_eq!(decrypted, secret_msg);
    }

    #[test]
    fn test_invalid_key_fails_gracefully() {
        let key_a = derive_key("room-a");
        let key_b = derive_key("room-b");
        let msg = b"Classified message";

        let encrypted = encrypt(&key_a, msg).unwrap();
        assert!(decrypt(&key_b, &encrypted).is_err());
    }
}
