use sha1::{Digest as Sha1Digest, Sha1};
use sha2::{Digest, Sha256};
use std::path::Path;
use tokio::io::AsyncReadExt;

use crate::error::{AppError, AppResult};

pub async fn sha256_file(path: &Path) -> AppResult<String> {
    let mut file = tokio::fs::File::open(path).await?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 1024 * 256];
    loop {
        let n = file.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex::encode(hasher.finalize()))
}

pub async fn sha1_file(path: &Path) -> AppResult<String> {
    let mut file = tokio::fs::File::open(path).await?;
    let mut hasher = Sha1::new();
    let mut buf = vec![0u8; 1024 * 256];
    loop {
        let n = file.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex::encode(hasher.finalize()))
}

pub fn verify_hash(actual: &str, expected: &str) -> AppResult<()> {
    if actual.eq_ignore_ascii_case(expected) {
        Ok(())
    } else {
        Err(AppError::new(
            "Download verification failed",
            "The downloaded file did not match the official checksum.",
        )
        .with_causes(vec![
            "The download may have been interrupted",
            "The file may have been modified in transit",
        ])
        .with_technical(format!("expected {expected}, got {actual}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_mismatch_is_rejected() {
        let err = verify_hash("aaa", "bbb").unwrap_err();
        assert!(err.title.contains("verification"));
    }
}
