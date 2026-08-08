use crate::{cas::CasStore, dedup, provisioning::ThinVolume, vault::EncryptedVault};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Clone, Default)]
pub struct SovereignDriveEngine {
    pub cas: CasStore,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct IngestResult {
    pub hash: String,
}

impl SovereignDriveEngine {
    pub fn persistent(root: impl AsRef<Path>) -> Result<Self> {
        Ok(Self { cas: CasStore::persistent(root)? })
    }

    pub fn ingest_bytes(&self, data: &[u8]) -> Result<IngestResult> {
        let hash = dedup::store_chunk(&self.cas, data)?;
        Ok(IngestResult { hash })
    }

    pub fn read_bytes(&self, hash: &str) -> Result<Vec<u8>> {
        self.cas.get_content(hash)
    }

    pub fn create_thin_volume(&self, id: &str, logical_size: u64) -> ThinVolume {
        ThinVolume::new(id, logical_size)
    }

    pub fn seal_wallet_vault(&self, passphrase: &str, payload: &[u8]) -> Result<EncryptedVault> {
        EncryptedVault::seal(passphrase, payload)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ingest_and_read_round_trip() {
        let engine = SovereignDriveEngine::default();
        let data = b"hello sovereign drive";
        let result = engine.ingest_bytes(data).unwrap();
        assert_eq!(engine.read_bytes(&result.hash).unwrap(), data);
        assert_eq!(result.hash, blake3::hash(data).to_hex().to_string());
    }
}
