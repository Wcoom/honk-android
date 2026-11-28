//! ELF64 Little-Endian AArch64 (`e_machine = 183`) verifier & archive extractor (`de1.java`/`hi1.java`).

use crate::MgrResult;
use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use std::io::{Cursor, Read};
use tar::Archive;
use zip::ZipArchive;

/// Hard package size ceiling (192 MiB) matching `com.boxproxy.honk` `hi1.java`.
pub const MAX_CORE_PACKAGE_BYTES: usize = 192 * 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageFormat {
    Direct,
    Gzip,
    TarGz,
    Tar,
    Zip,
}

impl PackageFormat {
    pub fn detect_from_name(name: &str) -> Self {
        let lower = name.to_ascii_lowercase();
        if lower.ends_with(".tar.gz") || lower.ends_with(".tgz") {
            Self::TarGz
        } else if lower.ends_with(".gz") {
            Self::Gzip
        } else if lower.ends_with(".tar") {
            Self::Tar
        } else if lower.ends_with(".zip") {
            Self::Zip
        } else {
            Self::Direct
        }
    }
}

/// Strictly verify ELF64 (`EI_CLASS=2`), Little-Endian (`EI_DATA=1`), `EM_AARCH64` (`e_machine=183`).
pub fn validate_elf64_aarch64(bytes: &[u8]) -> MgrResult<()> {
    if bytes.len() < 20 {
        return Err("file too small to be an ELF64 binary".to_string());
    }
    if &bytes[0..4] != b"\x7fELF" {
        return Err("invalid ELF magic header".to_string());
    }
    if bytes[4] != 2 {
        return Err(format!("expected 64-bit ELF (EI_CLASS=2), got {}", bytes[4]));
    }
    if bytes[5] != 1 {
        return Err(format!(
            "expected little-endian ELF (EI_DATA=1), got {}",
            bytes[5]
        ));
    }
    let e_machine = u16::from_le_bytes([bytes[18], bytes[19]]);
    if e_machine != 183 {
        return Err(format!(
            "expected EM_AARCH64 (183), got e_machine={e_machine}"
        ));
    }
    Ok(())
}

/// Extract core binary bytes from Direct/Gzip/TarGz/Tar/Zip, enforce 192 MiB cap & ELF64 AArch64 check.
pub fn extract_core_binary(
    raw: &[u8],
    format: PackageFormat,
    expected_bin_name: &str,
) -> MgrResult<Vec<u8>> {
    if raw.len() > MAX_CORE_PACKAGE_BYTES {
        return Err("core archive exceeds 192 MiB limit".to_string());
    }
    let extracted = match format {
        PackageFormat::Direct => raw.to_vec(),
        PackageFormat::Gzip => {
            let dec = GzDecoder::new(raw);
            let mut out = Vec::new();
            dec.take((MAX_CORE_PACKAGE_BYTES + 1) as u64)
                .read_to_end(&mut out)
                .map_err(|e| format!("gzip decompress failed: {e}"))?;
            if out.len() > MAX_CORE_PACKAGE_BYTES {
                return Err("decompressed binary exceeds 192 MiB limit".to_string());
            }
            out
        }
        PackageFormat::TarGz => {
            let dec = GzDecoder::new(raw);
            extract_from_tar(dec, expected_bin_name)?
        }
        PackageFormat::Tar => extract_from_tar(Cursor::new(raw), expected_bin_name)?,
        PackageFormat::Zip => extract_from_zip(raw, expected_bin_name)?,
    };

    validate_elf64_aarch64(&extracted)?;
    Ok(extracted)
}

fn extract_from_tar<R: Read>(reader: R, expected_name: &str) -> MgrResult<Vec<u8>> {
    let mut archive = Archive::new(reader);
    let mut fallback: Option<Vec<u8>> = None;
    for entry in archive.entries().map_err(|e| format!("tar read failed: {e}"))? {
        let entry = entry.map_err(|e| format!("tar entry failed: {e}"))?;
        if !entry.header().entry_type().is_file() {
            continue;
        }
        let path = entry
            .path()
            .map_err(|e| format!("tar path failed: {e}"))?
            .to_path_buf();
        let file_name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        let mut buf = Vec::new();
        entry
            .take((MAX_CORE_PACKAGE_BYTES + 1) as u64)
            .read_to_end(&mut buf)
            .map_err(|e| format!("tar extract failed: {e}"))?;
        if buf.len() > MAX_CORE_PACKAGE_BYTES {
            return Err("tar entry exceeds 192 MiB limit".to_string());
        }
        if file_name == expected_name.to_ascii_lowercase() && validate_elf64_aarch64(&buf).is_ok() {
            return Ok(buf);
        }
        if fallback.is_none() && validate_elf64_aarch64(&buf).is_ok() {
            fallback = Some(buf);
        }
    }
    fallback.ok_or_else(|| format!("no valid ELF64 AArch64 binary found in tar for {expected_name}"))
}

fn extract_from_zip(raw: &[u8], expected_name: &str) -> MgrResult<Vec<u8>> {
    let mut archive =
        ZipArchive::new(Cursor::new(raw)).map_err(|e| format!("zip open failed: {e}"))?;
    let mut fallback: Option<Vec<u8>> = None;
    for idx in 0..archive.len() {
        let entry = archive
            .by_index(idx)
            .map_err(|e| format!("zip entry {idx} failed: {e}"))?;
        if entry.is_dir() {
            continue;
        }
        let file_name = entry
            .name()
            .rsplit('/')
            .next()
            .unwrap_or("")
            .to_ascii_lowercase();
        let mut buf = Vec::new();
        entry
            .take((MAX_CORE_PACKAGE_BYTES + 1) as u64)
            .read_to_end(&mut buf)
            .map_err(|e| format!("zip read failed: {e}"))?;
        if buf.len() > MAX_CORE_PACKAGE_BYTES {
            return Err("zip entry exceeds 192 MiB limit".to_string());
        }
        if file_name == expected_name.to_ascii_lowercase() && validate_elf64_aarch64(&buf).is_ok() {
            return Ok(buf);
        }
        if fallback.is_none() && validate_elf64_aarch64(&buf).is_ok() {
            fallback = Some(buf);
        }
    }
    fallback.ok_or_else(|| format!("no valid ELF64 AArch64 binary found in zip for {expected_name}"))
}

/// Verify optional release SHA-256 checksum (`hi1.java` parity).
pub fn verify_release_sha256(bytes: &[u8], expected_hex: &str) -> MgrResult<()> {
    let expected = expected_hex.trim().to_ascii_lowercase();
    if expected.is_empty() {
        return Ok(());
    }
    let actual = boxproxy_root::sha256_hex(bytes);
    if actual != expected {
        return Err("SHA-256 does not match the release checksum".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::Write;

    fn fake_arm64_elf() -> Vec<u8> {
        let mut b = vec![0u8; 64];
        b[0..4].copy_from_slice(b"\x7fELF");
        b[4] = 2; // 64-bit
        b[5] = 1; // little endian
        b[18..20].copy_from_slice(&183u16.to_le_bytes()); // EM_AARCH64
        b
    }

    #[test]
    fn validates_elf64_aarch64_and_rejects_x86_64() {
        let arm64 = fake_arm64_elf();
        assert!(validate_elf64_aarch64(&arm64).is_ok());

        let mut x86 = arm64.clone();
        x86[18..20].copy_from_slice(&62u16.to_le_bytes());
        assert!(validate_elf64_aarch64(&x86).is_err());

        let mut enc = GzEncoder::new(Vec::new(), Compression::default());
        enc.write_all(&arm64).unwrap();
        let gz = enc.finish().unwrap();
        let extracted = extract_core_binary(&gz, PackageFormat::Gzip, "mihomo").unwrap();
        assert_eq!(extracted, arm64);
    }
}
