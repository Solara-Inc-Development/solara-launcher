//! Cryptographic hash helpers and atomic verified file persistence.

use anyhow::{Context, bail};
use sha1::{Digest as _, Sha1};
use sha2::Sha256;
use std::path::Path;
use tokio::io::AsyncReadExt;

pub fn sha1_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

pub async fn hash_file_sha1(path: &Path) -> anyhow::Result<String> {
    let mut file = tokio::fs::File::open(path)
        .await
        .with_context(|| format!("failed to open {} for SHA-1 verification", path.display()))?;
    let mut hasher = Sha1::new();
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = file.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex::encode(hasher.finalize()))
}

pub async fn hash_file_sha256(path: &Path) -> anyhow::Result<String> {
    let mut file = tokio::fs::File::open(path)
        .await
        .with_context(|| format!("failed to open {} for SHA-256 verification", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = file.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex::encode(hasher.finalize()))
}

pub async fn verify_sha1_file(path: &Path, expected_hex: &str) -> anyhow::Result<bool> {
    if !tokio::fs::try_exists(path).await.unwrap_or(false) {
        return Ok(false);
    }
    let actual = hash_file_sha1(path).await?;
    Ok(actual.eq_ignore_ascii_case(expected_hex.trim()))
}

pub async fn verify_sha256_file(path: &Path, expected_hex: &str) -> anyhow::Result<bool> {
    if !tokio::fs::try_exists(path).await.unwrap_or(false) {
        return Ok(false);
    }
    let actual = hash_file_sha256(path).await?;
    Ok(actual.eq_ignore_ascii_case(expected_hex.trim()))
}

/// Verifies `bytes` against an optional expected SHA-1 digest and writes atomically via a `.tmp` sibling.
pub async fn write_verified_sha1(
    dest: &Path,
    bytes: &[u8],
    expected_sha1: Option<&str>,
) -> anyhow::Result<()> {
    if let Some(expected) = expected_sha1 {
        let actual = sha1_hex(bytes);
        if !actual.eq_ignore_ascii_case(expected.trim()) {
            bail!(
                "SHA-1 mismatch for {}: expected {}, got {}",
                dest.display(),
                expected,
                actual
            );
        }
    }

    if let Some(parent) = dest.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    let tmp_name = format!(
        ".{}.{}.tmp",
        dest.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("artifact"),
        std::process::id()
    );
    let tmp_path = dest.with_file_name(tmp_name);
    tokio::fs::write(&tmp_path, bytes).await?;
    tokio::fs::rename(&tmp_path, dest).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn computes_known_digests() {
        // RFC 3174 / FIPS 180-4 standard test vectors
        assert_eq!(sha1_hex(b""), "da39a3ee5e6b4b0d3255bfef95601890afd80709");
        assert_eq!(sha1_hex(b"abc"), "a9993e364706816aba3e25717850c26c9cd0d89d");
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[tokio::test]
    async fn atomic_verified_write_rejects_mismatch_and_accepts_valid() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("nested/client.jar");
        let payload = b"minecraft-test-bytes";
        let good_sha1 = sha1_hex(payload);

        let err = write_verified_sha1(&target, payload, Some("0000000000000000000000000000000000000000"))
            .await
            .unwrap_err();
        assert!(err.to_string().contains("SHA-1 mismatch"));
        assert!(!target.exists());

        write_verified_sha1(&target, payload, Some(&good_sha1))
            .await
            .unwrap();
        assert!(verify_sha1_file(&target, &good_sha1).await.unwrap());
    }
}
