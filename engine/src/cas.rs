use anyhow::{anyhow, Context, Result};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::{Path, PathBuf}, sync::Arc};

const MAGIC: &[u8; 5] = b"SDCO1";
const HEADER_LEN: usize = 14;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CasObject {
    pub hash: String,
    pub compressed: bool,
    pub size: usize,
    pub payload: Vec<u8>,
}

#[derive(Clone, Default)]
pub struct CasStore {
    inner: Arc<RwLock<HashMap<String, CasObject>>>,
    root: Option<Arc<PathBuf>>,
}

impl CasStore {
    pub fn persistent(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref().to_path_buf();
        fs::create_dir_all(root.join("objects")).with_context(|| "create CAS root")?;
        Ok(Self { inner: Arc::new(RwLock::new(HashMap::new())), root: Some(Arc::new(root)) })
    }

    /// Store an already-canonical payload. The returned hash identifies `payload`.
    pub fn put(&self, payload: Vec<u8>, compressed: bool) -> Result<String> {
        let hash = blake3::hash(&payload).to_hex().to_string();
        self.put_hashed(hash, payload, compressed)
    }

    /// Store a representation of canonical content. The hash always identifies `content`.
    pub fn put_content(&self, content: &[u8], stored_payload: Vec<u8>, compressed: bool) -> Result<String> {
        let hash = blake3::hash(content).to_hex().to_string();
        self.put_hashed(hash, stored_payload, compressed)
    }

    fn put_hashed(&self, hash: String, payload: Vec<u8>, compressed: bool) -> Result<String> {
        if self.exists(&hash) { return Ok(hash); }
        let obj = CasObject { hash: hash.clone(), compressed, size: payload.len(), payload };
        if let Some(root) = &self.root {
            let path = object_path(root, &hash)?;
            if let Some(parent) = path.parent() { fs::create_dir_all(parent)?; }
            let tmp = path.with_extension(format!("tmp-{}", std::process::id()));
            fs::write(&tmp, encode_object(&obj)?)?;
            fs::rename(&tmp, &path).with_context(|| format!("commit CAS object {hash}"))?;
        }
        self.inner.write().insert(hash.clone(), obj);
        Ok(hash)
    }

    pub fn get(&self, hash: &str) -> Option<CasObject> {
        if let Some(obj) = self.inner.read().get(hash).cloned() { return Some(obj); }
        let root = self.root.as_ref()?;
        let path = object_path(root, hash).ok()?;
        let bytes = fs::read(path).ok()?;
        let obj = decode_object(hash, &bytes).ok()?;
        self.inner.write().insert(hash.to_owned(), obj.clone());
        Some(obj)
    }

    pub fn get_content(&self, hash: &str) -> Result<Vec<u8>> {
        let obj = self.get(hash).ok_or_else(|| anyhow!("CAS object not found: {hash}"))?;
        let content = if obj.compressed { crate::compression::decompress(&obj.payload)? } else { obj.payload };
        let actual = blake3::hash(&content).to_hex().to_string();
        if actual != hash { return Err(anyhow!("CAS integrity check failed for {hash}")); }
        Ok(content)
    }

    pub fn exists(&self, hash: &str) -> bool {
        if self.inner.read().contains_key(hash) { return true; }
        self.root.as_ref().and_then(|root| object_path(root, hash).ok()).is_some_and(|p| p.is_file())
    }

    pub fn len(&self) -> usize { self.inner.read().len() }
}

fn object_path(root: &Path, hash: &str) -> Result<PathBuf> {
    if hash.len() != 64 || !hash.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(anyhow!("invalid CAS hash"));
    }
    Ok(root.join("objects").join(&hash[..2]).join(hash))
}

fn encode_object(obj: &CasObject) -> Result<Vec<u8>> {
    let mut out = Vec::with_capacity(HEADER_LEN + obj.payload.len());
    out.extend_from_slice(MAGIC);
    out.push(u8::from(obj.compressed));
    out.extend_from_slice(&(obj.size as u64).to_le_bytes());
    out.extend_from_slice(&obj.payload);
    Ok(out)
}

fn decode_object(hash: &str, bytes: &[u8]) -> Result<CasObject> {
    if bytes.len() < HEADER_LEN || &bytes[..5] != MAGIC { return Err(anyhow!("invalid CAS object")); }
    let compressed = bytes[5] != 0;
    let size = u64::from_le_bytes(bytes[6..14].try_into().unwrap()) as usize;
    let payload = bytes[14..].to_vec();
    if size != payload.len() { return Err(anyhow!("CAS object size mismatch")); }
    Ok(CasObject { hash: hash.to_owned(), compressed, size, payload })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn persistent_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let store = CasStore::persistent(dir.path()).unwrap();
        let hash = store.put_content(b"hello", b"hello".to_vec(), false).unwrap();
        let reopened = CasStore::persistent(dir.path()).unwrap();
        assert_eq!(reopened.get_content(&hash).unwrap(), b"hello");
        assert!(reopened.exists(&hash));
    }

    #[test]
    fn compressed_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let store = CasStore::persistent(dir.path()).unwrap();
        let raw = vec![b'a'; 4096];
        let compressed = crate::compression::compress(&raw, 8).unwrap();
        let hash = store.put_content(&raw, compressed, true).unwrap();
        assert_eq!(store.get_content(&hash).unwrap(), raw);
    }

    #[test]
    fn rejects_invalid_hash_paths() {
        let dir = tempfile::tempdir().unwrap();
        let store = CasStore::persistent(dir.path()).unwrap();
        assert!(!store.exists("../escape"));
    }
}
