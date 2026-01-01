use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use serde::{Deserialize, Serialize};

use crate::error::LumenError;

#[derive(Serialize, Deserialize)]
struct CacheEntry {
    diff_hash: String,
    summary: String,
}

fn get_cache_file_path() -> Option<PathBuf> {
    dirs::cache_dir().map(|p| p.join("lumen").join("last_explanation.json"))
}

fn calculate_diff_hash(diff: &str) -> Result<String, LumenError> {
    let mut child = Command::new("git")
        .arg("hash-object")
        .arg("--stdin")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(diff.as_bytes())?;
    }

    let output = child.wait_with_output()?;

    if !output.status.success() {
        return Err(LumenError::CommandError(
            "git hash-object failed".to_string(),
        ));
    }

    let hash = String::from_utf8(output.stdout)?.trim().to_string();

    Ok(hash)
}

pub fn save_explanation(diff: &str, summary: &str) -> Result<(), LumenError> {
    log::trace!("Attempting to save explanation to cache");
    let hash = calculate_diff_hash(diff)?;
    log::trace!("Calculated diff hash: {}", hash);
    let entry = CacheEntry {
        diff_hash: hash,
        summary: summary.to_string(),
    };

    let path = get_cache_file_path().ok_or_else(|| {
        log::error!("Could not determine cache directory");
        LumenError::ConfigurationError("Could not determine cache directory".to_string())
    })?;

    if let Some(parent) = path.parent() {
        if !parent.exists() {
            log::trace!("Creating cache directory: {:?}", parent);
            fs::create_dir_all(parent)?;
        }
    }

    log::trace!("Writing cache entry to: {:?}", path);
    let json = serde_json::to_string(&entry)?;
    fs::write(path, json)?;

    Ok(())
}

pub fn get_explanation(diff: &str) -> Option<String> {
    log::trace!("Checking cache for explanation");
    let path = get_cache_file_path()?;
    if !path.exists() {
        log::trace!("Cache file does not exist: {:?}", path);
        return None;
    }

    let current_hash = calculate_diff_hash(diff).ok()?;
    log::trace!("Current diff hash: {}", current_hash);

    let content = fs::read_to_string(&path).ok()?;
    let entry: CacheEntry = serde_json::from_str(&content).ok()?;

    if entry.diff_hash == current_hash {
        log::trace!("Cache hit! Found matching explanation");
        Some(entry.summary)
    } else {
        log::trace!(
            "Cache miss: hash mismatch (cached: {}, current: {})",
            entry.diff_hash,
            current_hash
        );
        None
    }
}
