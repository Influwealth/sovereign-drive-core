use anyhow::{Context, Result};

pub fn compress(input: &[u8], level: i32) -> Result<Vec<u8>> {
    zstd::encode_all(input, level).context("zstd compression failed")
}

pub fn decompress(input: &[u8]) -> Result<Vec<u8>> {
    zstd::decode_all(input).context("zstd decompression failed")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let input = vec![b'x'; 8192];
        assert_eq!(decompress(&compress(&input, 8).unwrap()).unwrap(), input);
    }

    #[test]
    fn rejects_corrupt_stream() {
        assert!(decompress(b"not-zstd").is_err());
    }
}
