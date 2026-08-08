use crate::{cas::CasStore, compression};
use anyhow::Result;

/// Store content by the hash of its canonical/raw bytes while optionally keeping
/// a compressed representation on disk.
pub fn store_chunk(store: &CasStore, raw: &[u8]) -> Result<String> {
    let compressed = compression::compress(raw, 8)?;
    if compressed.len() < raw.len() {
        store.put_content(raw, compressed, true)
    } else {
        store.put_content(raw, raw.to_vec(), false)
    }
}
