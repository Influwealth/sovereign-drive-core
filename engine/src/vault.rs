use crate::crypto;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

const SALT_LEN: usize = 16;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EncryptedVault {
    pub salt: [u8; SALT_LEN],
    pub nonce: [u8; 24],
    pub ciphertext: Vec<u8>,
}

impl EncryptedVault {
    pub fn seal(passphrase: &str, payload: &[u8]) -> Result<Self> {
        let salt = crypto::generate_salt();
        let key = crypto::derive_key(passphrase, &salt)?;
        let (ciphertext, nonce) = crypto::encrypt(&key, payload)?;
        Ok(Self { salt, nonce, ciphertext })
    }

    pub fn open(&self, passphrase: &str) -> Result<Vec<u8>> {
        self.validate()?;
        let key = crypto::derive_key(passphrase, &self.salt)?;
        crypto::decrypt(&key, &self.nonce, &self.ciphertext)
    }

    pub fn validate(&self) -> Result<()> {
        if self.ciphertext.len() < 16 { return Err(anyhow!("vault ciphertext is too short")); }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seal_open_round_trip() {
        let vault = EncryptedVault::seal("correct horse", b"secret payload").unwrap();
        assert_eq!(vault.open("correct horse").unwrap(), b"secret payload");
        assert!(vault.open("wrong password").is_err());
    }

    #[test]
    fn tampering_fails() {
        let mut vault = EncryptedVault::seal("password", b"secret").unwrap();
        vault.ciphertext[0] ^= 1;
        assert!(vault.open("password").is_err());
    }

    #[test]
    fn independent_seals_use_distinct_nonce_or_salt() {
        let a = EncryptedVault::seal("password", b"secret").unwrap();
        let b = EncryptedVault::seal("password", b"secret").unwrap();
        assert!(a.salt != b.salt || a.nonce != b.nonce);
    }
}
