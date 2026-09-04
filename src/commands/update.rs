use anyhow::{bail, Context, Result};
use std::cmp::Ordering;
use std::env;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::ui;

const RELEASE_DOWNLOAD_URL: &str =
    "https://github.com/GVenkatesh-12/recall/releases/download";
const LATEST_API_URL: &str =
    "https://api.github.com/repos/GVenkatesh-12/recall/releases/latest";

/// Compile-time target triple. Must match the asset names published by the
/// release workflow (`.github/workflows/release.yml`).
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
const TARGET_TRIPLE: &str = "x86_64-unknown-linux-gnu";
#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
const TARGET_TRIPLE: &str = "aarch64-unknown-linux-gnu";
#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
const TARGET_TRIPLE: &str = "x86_64-apple-darwin";
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
const TARGET_TRIPLE: &str = "aarch64-apple-darwin";
#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
const TARGET_TRIPLE: &str = "x86_64-pc-windows-msvc";

#[cfg(windows)]
const EXE_NAME: &str = "recall.exe";
#[cfg(not(windows))]
const EXE_NAME: &str = "recall";

use colored::Colorize;

/// Check for a newer release on GitHub and update recall in-place if one exists.
pub fn handle() -> Result<()> {
    let current = env!("CARGO_PKG_VERSION");
    println!();
    println!("  {}", "✦ RECALL UPDATER".bright_cyan().bold());
    println!("  {}", "─".repeat(55).dimmed());
    ui::print_info("Checking for the latest release on GitHub...");

    let latest = fetch_latest_tag()?;

    match compare_versions(&latest, current)? {
        Ordering::Equal => {
            ui::print_success(&format!("recall is already up to date (v{})", current));
        }
        Ordering::Less => {
            ui::print_warning(&format!(
                "You are running v{} which is newer than the latest GitHub release ({})",
                current, latest
            ));
        }
        Ordering::Greater => {
            ui::print_warning(&format!(
                "New version available: {} (installed: v{})",
                latest, current
            ));
            download_and_install(&latest)?;
            ui::print_success(&format!("Updated recall from v{} to {}", current, latest));
            ui::print_info("Run 'hash -r' (or restart your shell) to use the new binary in this session.");
        }
    }
    println!("  {}", "─".repeat(55).dimmed());
    println!();
    Ok(())
}

/// Query the GitHub API for the tag of the latest release.
fn fetch_latest_tag() -> Result<String> {
    let body = curl_text(&LATEST_API_URL)?;
    let value: serde_json::Value =
        serde_json::from_str(&body).with_context(|| "Failed to parse GitHub API response")?;
    let tag = value
        .get("tag_name")
        .and_then(|v| v.as_str())
        .with_context(|| "GitHub API response did not contain tag_name")?;
    Ok(tag.to_string())
}

/// Run `curl -fsSL` and return the response body.
fn curl_text(url: &str) -> Result<String> {
    let out = Command::new("curl")
        .args(["-fsSL", "--retry", "3", "-H", "User-Agent: recall-updater"])
        .arg(url)
        .output()
        .with_context(|| "Could not run curl. Install curl and try again.")?;
    if !out.status.success() {
        bail!(
            "Request failed (curl exit code {:?}): {}",
            out.status.code(),
            url
        );
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Run `curl -fsSL -o <path> <url>`.
fn curl_download(url: &str, dest: &Path) -> Result<()> {
    let status = Command::new("curl")
        .args(["-fsSL", "--retry", "3", "-o"])
        .arg(dest)
        .arg(url)
        .status()
        .with_context(|| "Could not run curl. Install curl and try again.")?;
    if !status.success() {
        bail!(
            "Download failed (curl exit code {:?}): {}",
            status.code(),
            url
        );
    }
    Ok(())
}

fn parse_version(v: &str) -> (u64, u64, u64) {
    let v = v.trim().trim_start_matches('v');
    let mut parts = v.split('.');
    let major = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let minor = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let patch = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    (major, minor, patch)
}

fn compare_versions(a: &str, b: &str) -> Result<Ordering> {
    Ok(parse_version(a).cmp(&parse_version(b)))
}

/// Download the release archive for this platform, verify its checksum,
/// extract the binary, and replace the running executable (or install to
/// ~/.local/bin if the current location is not writable).
fn download_and_install(tag: &str) -> Result<()> {
    let tmp = unique_temp_dir()?;

    let archive_name = if cfg!(windows) {
        format!("recall-{}.zip", TARGET_TRIPLE)
    } else {
        format!("recall-{}.tar.gz", TARGET_TRIPLE)
    };
    let url = format!("{}/{}/{}", RELEASE_DOWNLOAD_URL, tag, archive_name);
    let sha_url = format!("{}.sha256", url);

    ui::print_info(&format!("Downloading {}...", archive_name));
    let archive_path = tmp.join(&archive_name);
    curl_download(&url, &archive_path)?;
    let sha_path = tmp.join(format!("{}.sha256", archive_name));
    curl_download(&sha_url, &sha_path)?;

    let expected = parse_sha(&fs::read_to_string(&sha_path)?)?;
    let actual = sha256_file(&archive_path)?;
    if !expected.eq_ignore_ascii_case(&actual) {
        bail!(
            "Checksum mismatch for {} (expected {}, got {}). Aborting update.",
            archive_name,
            expected,
            actual
        );
    }
    ui::print_success("SHA-256 checksum verified");

    let exe_bytes: Vec<u8> = if cfg!(windows) {
        extract_zip_binary(&archive_path)?
    } else {
        extract_tar_gz_binary(&archive_path)?
    };

    let target = install_binary(&exe_bytes)?;
    fs::remove_dir_all(&tmp).ok();
    ui::print_success(&format!("Installed recall {} to {}", tag, target.display()));
    Ok(())
}

// ---------------------------------------------------------------------------
// sha256 helpers
// ---------------------------------------------------------------------------
use sha2::{Digest, Sha256};

fn sha256_file(path: &Path) -> Result<String> {
    let mut file = fs::File::open(path).with_context(|| "Failed to open downloaded file")?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex(&hasher.finalize()))
}

fn hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

/// Expects a `sha256sum` style line: "<64 hex chars>  <filename>".
fn parse_sha(contents: &str) -> Result<String> {
    let first = contents
        .split_whitespace()
        .next()
        .with_context(|| "Empty checksum file")?;
    if first.len() == 64 && first.chars().all(|c| c.is_ascii_hexdigit()) {
        Ok(first.to_string())
    } else {
        bail!("Unexpected checksum file format")
    }
}
// ---------------------------------------------------------------------------
// archive extraction (single binary, in memory)
// ---------------------------------------------------------------------------
fn extract_tar_gz_binary(path: &Path) -> Result<Vec<u8>> {
    let file = fs::File::open(path).with_context(|| "Failed to open downloaded archive")?;
    let decoder = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(decoder);
    let mut entries = archive
        .entries()
        .with_context(|| "Failed to read archive entries")?;
    while let Some(entry) = entries.next() {
        let mut entry = entry.with_context(|| "Failed to read archive entry")?;
        let name = entry
            .path()
            .with_context(|| "Failed to read entry path")?
            .to_string_lossy()
            .into_owned();
        let base_name = name.rsplit(['/', '\\']).next().unwrap_or(&name);
        if base_name == "recall" || base_name == "recall.exe" {
            let mut bytes = Vec::new();
            entry.read_to_end(&mut bytes)?;
            return Ok(bytes);
        }
    }
    bail!("Archive did not contain the recall binary")
}

fn extract_zip_binary(path: &Path) -> Result<Vec<u8>> {
    let file = fs::File::open(path).with_context(|| "Failed to open downloaded archive")?;
    let mut archive =
        zip::ZipArchive::new(file).with_context(|| "Failed to open downloaded archive")?;
    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .with_context(|| "Failed to read archive entry")?;
        let name = entry.name().to_string();
        let base_name = name.rsplit(['/', '\\']).next().unwrap_or(&name);
        if base_name == "recall" || base_name == "recall.exe" {
            let mut bytes = Vec::new();
            entry.read_to_end(&mut bytes)?;
            return Ok(bytes);
        }
    }
    bail!("Archive did not contain the recall binary")
}
// ---------------------------------------------------------------------------
// installation
// ---------------------------------------------------------------------------
use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};
static TEMP_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn unique_temp_dir() -> Result<PathBuf> {
    let count = TEMP_COUNTER.fetch_add(1, AtomicOrdering::Relaxed);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let dir = env::temp_dir().join(format!("recall-update-{}-{}-{}", std::process::id(), now, count));
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn dir_is_writable(dir: &Path) -> bool {
    let probe = dir.join(format!(".recall-write-test-{}", std::process::id()));
    match fs::write(&probe, b"x") {
        Ok(()) => {
            fs::remove_file(&probe).ok();
            true
        }
        Err(_) => false,
    }
}

#[cfg(unix)]
fn set_executable(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o755))
        .with_context(|| "Failed to make the binary executable")
}

#[cfg(not(unix))]
fn set_executable(_path: &Path) -> Result<()> {
    Ok(())
}

/// Replace the running binary in place. If its directory is not writable,
/// install to `~/.local/bin/recall` instead.
fn install_binary(bytes: &[u8]) -> Result<PathBuf> {
    let current = env::current_exe().with_context(|| "Could not determine current executable path")?;
    let current_dir = current
        .parent()
        .with_context(|| "Could not determine current executable directory")?
        .to_path_buf();

    if dir_is_writable(&current_dir) {
        let target = current_dir.join(EXE_NAME);
        let staged = current_dir.join(format!(".recall-update-{}.tmp", std::process::id()));
        fs::write(&staged, bytes).with_context(|| "Failed to stage the new binary")?;
        set_executable(&staged)?;
        match fs::rename(&staged, &target) {
            Ok(()) => return Ok(target),
            Err(e) => {
                fs::remove_file(&staged).ok();
                if cfg!(windows) {
                    ui::print_warning(&format!(
                        "Could not replace {} in place: {}",
                        target.display(),
                        e
                    ));
                } else {
                    return Err(e).with_context(|| {
                        format!("Failed to write new binary to {}", target.display())
                    });
                }
            }
        }
    }

    // Fallback: install to ~/.local/bin
    let home = dirs::home_dir().with_context(|| "Could not determine home directory")?;
    let bin_dir = home.join(".local").join("bin");
    fs::create_dir_all(&bin_dir).with_context(|| "Failed to create ~/.local/bin")?;
    let target = bin_dir.join(EXE_NAME);
    let staged = bin_dir.join(format!(".recall-update-{}.tmp", std::process::id()));
    fs::write(&staged, bytes).with_context(|| "Failed to stage the new binary")?;
    set_executable(&staged)?;
    fs::rename(&staged, &target)
        .with_context(|| format!("Failed to install new binary to {}", target.display()))?;

    ui::print_warning(&format!(
        "The binary you ran (at {}) could not be replaced; installed the update at {}. You may want to remove the old one.",
        current.display(),
        target.display()
    ));
    Ok(target)
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_compare_versions() {
        assert_eq!(compare_versions("v1.1.0", "1.0.2").unwrap(), Ordering::Greater);
        assert_eq!(compare_versions("v1.0.2", "1.0.2").unwrap(), Ordering::Equal);
        assert_eq!(compare_versions("v1.2.0", "2.0.0").unwrap(), Ordering::Less);
        assert_eq!(parse_version("v1.2.3"), (1, 2, 3));
        assert_eq!(parse_version("1.0"), (1, 0, 0));
    }

    #[test]
    fn test_parse_sha() {
        let good = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef  recall-x86_64-unknown-linux-gnu.tar.gz";
        assert_eq!(parse_sha(good).unwrap().len(), 64);

        let with_path = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef  dist/recall-x86_64-unknown-linux-gnu.tar.gz";
        assert_eq!(parse_sha(with_path).unwrap().len(), 64);

        assert!(parse_sha("nonsense").is_err());
    }

    #[test]
    fn test_hex() {
        assert_eq!(hex(&[0xde, 0xad]), "dead");
        assert_eq!(hex(&[]), "");
    }

    #[test]
    fn test_sha256_known_vector() {
        // sha256("abc") — standard test vector
        let dir = unique_temp_dir().unwrap();
        let file = dir.join("t.txt");
        fs::File::create(&file).unwrap().write_all(b"abc").unwrap();
        assert_eq!(
            sha256_file(&file).unwrap(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_extract_tar_gz_roundtrip() {
        let payload = b"#!/bin/sh\necho hi\n".to_vec();
        let dir = unique_temp_dir().unwrap();
        let archive = dir.join("recall-test.tar.gz");

        // Build a tar.gz containing a file named "recall"
        let mut header = tar::Header::new_gnu();
        header.set_size(payload.len() as u64);
        header.set_mode(0o755);
        header.set_cksum();
        let mut tarball = tar::Builder::new(Vec::new());
        tarball
            .append_data(&mut header, "recall", payload.as_slice())
            .unwrap();
        let tar_bytes = tarball.into_inner().unwrap();

        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(&tar_bytes).unwrap();
        let gz_bytes = encoder.finish().unwrap();
        fs::write(&archive, gz_bytes).unwrap();

        assert_eq!(extract_tar_gz_binary(&archive).unwrap(), payload);
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_extract_zip_roundtrip() {
        let payload = b"MZ fake windows exe".to_vec();
        let dir = unique_temp_dir().unwrap();
        let archive = dir.join("recall-test.zip");

        let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        writer.start_file("recall.exe", options).unwrap();
        writer.write_all(&payload).unwrap();
        let cursor = writer.finish().unwrap();
        fs::write(&archive, cursor.into_inner()).unwrap();

        assert_eq!(extract_zip_binary(&archive).unwrap(), payload);
        fs::remove_dir_all(&dir).ok();
    }
}