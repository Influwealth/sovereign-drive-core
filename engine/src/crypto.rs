use anyhow::{anyhow, Result};
use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::{aead::{Aead, KeyInit, OsRng}, XChaCha20Poly1305, XNonce};
use rand::RngCore;
use zeroize::Zeroizing;

const KEY_LEN: usize = 32;
const MIN_SALT_LEN: usize = 16;

pub fn derive_key(passphrase: &str, salt_bytes: &[u8]) -> Result<Zeroizing<[u8; KEY_LEN]>> {
    if passphrase.is_empty() { return Err(anyhow!("passphrase must not be empty")); }
    if salt_bytes.len() < MIN_SALT_LEN { return Err(anyhow!("salt must be at least 16 bytes")); }
    let params = Params::new(19_456, 2, 1, Some(KEY_LEN)).map_err(|e| anyhow!(e.to_string()))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut out = Zeroizing::new([0u8; KEY_LEN]);
    argon2.hash_password_into(passphrase.as_bytes(), salt_bytes, &mut *out)
        .map_err(|e| anyhow!(e.to_string()))?;
    Ok(out)
}

pub fn generate_salt() -> [u8; MIN_SALT_LEN] {
    let mut salt = [0u8; MIN_SALT_LEN];
    OsRng.fill_bytes(&mut salt);
    salt
}

pub fn encrypt(key: &[u8; KEY_LEN], plaintext: &[u8]) -> Result<(Vec<u8>, [u8; 24])> {
    let cipher = XChaCha20Poly1305::new(key.into());
    let mut nonce = [0u8; 24];
    OsRng.fill_bytes(&mut nonce);
    let ciphertext = cipher.encrypt(XNonce::from_slice(&nonce), plaintext)
        .map_err(|_| anyhow!("encryption failed"))?;
    Ok((ciphertext, nonce))
}

pub fn decrypt(key: &[u8; KEY_LEN], nonce: &[u8; 24], ciphertext: &[u8]) -> Result<Vec<u8>> {
    let cipher = XChaCha20Poly1305::new(key.into());
    cipher.decrypt(XNonce::from_slice(nonce), ciphertext)
        .map_err(|_| anyhow!("decryption failed"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encryption_round_trip_and_tamper_detection() {
        let key = derive_key("test-passphrase", b"0123456789abcdef").unwrap();
        let (ciphertext, nonce) = encrypt(&key, b"secret").unwrap();
        assert_eq!(decrypt(&key, &nonce, &ciphertext).unwrap(), b"secret");
        assert!(decrypt(&key, &nonce, b"tampered").is_err());
    }

    #[test]
    fn rejects_weak_inputs() {
        assert!(derive_key("", b"0123456789abcdef").is_err());
        assert!(derive_key("pass", b"short").is_err());
    }

    #[test]
    fn deterministic_for_same_salt() {
        let a = derive_key("same", b"0123456789abcdef").unwrap();
        let b = derive_key("same", b"0123456789abcdef").unwrap();
        assert_eq!(&*a, &*b);
    }

    #[test]
    fn generated_salts_are_sized_correctly() {
        assert_eq!(generate_salt().len(), MIN_SALT_LEN);
    }
}
