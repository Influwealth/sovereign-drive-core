use anyhow::{anyhow, Context, Result};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::{Path, PathBuf}, sync::Arc, time::{SystemTime, UNIX_EPOCH}};

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
        recover_temporary_objects(&root)?;
        Ok(Self { inner: Arc::new(RwLock::new(HashMap::new())), root: Some(Arc::new(root)) })
    }

    pub fn put(&self, payload: Vec<u8>, compressed: bool) -> Result<String> {
        let hash = blake3::hash(&payload).to_hex().to_string();
        self.put_hashed(hash, payload, compressed)
    }

    pub fn put_content(&self, content: &[u8], stored_payload: Vec<u8>, compressed: bool) -> Result<String> {
        let hash = blake3::hash(content).to_hex().to_string();
        self.put_hashed(hash, stored_payload, compressed)
    }

    fn put_hashed(&self, hash: String, payload: Vec<u8>, compressed: bool) -> Result<String> {
        let obj = CasObject { hash: hash.clone(), compressed, size: payload.len(), payload };
        let mut guard = self.inner.write();
        if guard.contains_key(&hash) { return Ok(hash); }

        if let Some(root) = &self.root {
            let path = object_path(root, &hash)?;
            if path.is_file() {
                guard.insert(hash.clone(), obj_from_disk(&hash, &path)?);
                return Ok(hash);
            }
            if let Some(parent) = path.parent() { fs::create_dir_all(parent)?; }
            let tmp = temporary_path(&path);
            let encoded = encode_object(&obj)?;
            let mut file = fs::OpenOptions::new().write(true).create_new(true).open(&tmp)
                .with_context(|| format!("create temporary CAS object {hash}"))?;
            use std::io::Write;
            file.write_all(&encoded)?;
            file.sync_all().with_context(|| "sync CAS object")?;
            drop(file);
            match fs::rename(&tmp, &path) {
                Ok(()) => {
                    if let Some(parent) = path.parent() {
                        if let Ok(dir) = fs::File::open(parent) { let _ = dir.sync_all(); }
                    }
                }
                Err(_error) if path.is_file() => { let _ = fs::remove_file(&tmp); }
                Err(error) => { let _ = fs::remove_file(&tmp); return Err(error).with_context(|| format!("commit CAS object {hash}")); }
            }
        }
        guard.insert(hash.clone(), obj);
        Ok(hash)
    }

    pub fn get(&self, hash: &str) -> Option<CasObject> {
        if let Some(obj) = self.inner.read().get(hash).cloned() { return Some(obj); }
        let root = self.root.as_ref()?;
        let path = object_path(root, hash).ok()?;
        let obj = obj_from_disk(hash, &path).ok()?;
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

/// Validate an encoded CAS object without writing it. Used by fuzz/property harnesses.
pub fn validate_encoded_object(hash: &str, bytes: &[u8]) -> Result<CasObject> {
    decode_object(hash, bytes)
}

fn object_path(root: &Path, hash: &str) -> Result<PathBuf> {
    if hash.len() != 64 || !hash.bytes().all(|b| b.is_ascii_hexdigit()) { return Err(anyhow!("invalid CAS hash")); }
    Ok(root.join("objects").join(&hash[..2]).join(hash))
}

fn temporary_path(path: &Path) -> PathBuf {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
    path.with_extension(format!("tmp-{}-{}", std::process::id(), nanos))
}

fn recover_temporary_objects(root: &Path) -> Result<()> {
    let objects = root.join("objects");
    if !objects.is_dir() { return Ok(()); }
    for prefix in fs::read_dir(objects)? {
        let prefix = prefix?.path();
        if !prefix.is_dir() { continue; }
        for entry in fs::read_dir(&prefix)? {
            let path = entry?.path();
            if path.extension().and_then(|v| v.to_str()).is_some_and(|v| v.starts_with("tmp-")) { let _ = fs::remove_file(path); }
        }
    }
    Ok(())
}

fn obj_from_disk(hash: &str, path: &Path) -> Result<CasObject> {
    let bytes = fs::read(path).with_context(|| format!("read CAS object {hash}"))?;
    decode_object(hash, &bytes)
}

fn encode_object(obj: &CasObject) -> Result<Vec<u8>> {
    let size = u64::try_from(obj.size).context("CAS payload too large")?;
    let mut out = Vec::with_capacity(HEADER_LEN + obj.payload.len());
    out.extend_from_slice(MAGIC);
    out.push(u8::from(obj.compressed));
    out.extend_from_slice(&size.to_le_bytes());
    out.extend_from_slice(&obj.payload);
    Ok(out)
}

fn decode_object(hash: &str, bytes: &[u8]) -> Result<CasObject> {
    if bytes.len() < HEADER_LEN || &bytes[..5] != MAGIC { return Err(anyhow!("invalid CAS object")); }
    let compressed = bytes[5] != 0;
    let size = u64::from_le_bytes(bytes[6..14].try_into().context("invalid CAS header")?);
    let size = usize::try_from(size).context("CAS object size exceeds platform limit")?;
    let payload = bytes[14..].to_vec();
    if size != payload.len() { return Err(anyhow!("CAS object size mismatch")); }
    Ok(CasObject { hash: hash.to_owned(), compressed, size, payload })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{sync::Arc, thread};

    #[test]
    fn persistent_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let store = CasStore::persistent(dir.path()).unwrap();
        let hash = store.put_content(b"hello", b"hello".to_vec(), false).unwrap();
        let reopened = CasStore::persistent(dir.path()).unwrap();
        assert_eq!(reopened.get_content(&hash).unwrap(), b"hello");
    }

    #[test]
    fn concurrent_duplicate_puts_are_safe() {
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(CasStore::persistent(dir.path()).unwrap());
        let mut workers = Vec::new();
        for _ in 0..8 {
            let store = Arc::clone(&store);
            workers.push(thread::spawn(move || store.put_content(b"same", b"same".to_vec(), false).unwrap()));
        }
        let hashes: Vec<_> = workers.into_iter().map(|h| h.join().unwrap()).collect();
        assert!(hashes.iter().all(|h| h == &hashes[0]));
        assert_eq!(store.get_content(&hashes[0]).unwrap(), b"same");
    }

    #[test]
    fn removes_orphaned_temporary_files_on_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let objects = dir.path().join("objects").join("aa");
        fs::create_dir_all(&objects).unwrap();
        fs::write(objects.join("dead.tmp-crash"), b"partial").unwrap();
        CasStore::persistent(dir.path()).unwrap();
        assert!(!objects.join("dead.tmp-crash").exists());
    }

    #[test]
    fn rejects_invalid_hash_paths() {
        let dir = tempfile::tempdir().unwrap();
        let store = CasStore::persistent(dir.path()).unwrap();
        assert!(!store.exists("../escape"));
    }
}
