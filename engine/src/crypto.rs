use anyhow::{anyhow, Result};
use argon2::Argon2;
use chacha20poly1305::{aead::{Aead, KeyInit, OsRng}, XChaCha20Poly1305, XNonce};
use rand::RngCore;
use zeroize::Zeroizing;

pub fn derive_key(passphrase: &str, salt_bytes: &[u8]) -> Result<Zeroizing<[u8; 32]>> {
    if passphrase.is_empty() { return Err(anyhow!("passphrase must not be empty")); }
    if salt_bytes.len() < 16 { return Err(anyhow!("salt must be at least 16 bytes")); }
    let mut out = Zeroizing::new([0u8; 32]);
    Argon2::default()
        .hash_password_into(passphrase.as_bytes(), salt_bytes, &mut *out)
        .map_err(|e| anyhow!(e.to_string()))?;
    Ok(out)
}

pub fn encrypt(key: &[u8; 32], plaintext: &[u8]) -> Result<(Vec<u8>, [u8; 24])> {
    let cipher = XChaCha20Poly1305::new(key.into());
    let mut nonce = [0u8; 24];
    OsRng.fill_bytes(&mut nonce);
    let ciphertext = cipher.encrypt(XNonce::from_slice(&nonce), plaintext)
        .map_err(|_| anyhow!("encryption failed"))?;
    Ok((ciphertext, nonce))
}

pub fn decrypt(key: &[u8; 32], nonce: &[u8; 24], ciphertext: &[u8]) -> Result<Vec<u8>> {
    let cipher = XChaCha20Poly1305::new(key.into());
    cipher.decrypt(XNonce::from_slice(nonce), ciphertext)
        .map_err(|_| anyhow!("decryption failed"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encryption_round_trip() {
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
}
