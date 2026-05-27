use crate::models::{FileEntry, NoDeletionReport, ReceiptEntry, ScanReceipt, SystemInfo};
use anyhow::Result;
use blake3::Hasher;
use std::path::Path;

/// Generate a `ScanReceipt` for the given file list rooted at `root`.
/// The root_hash is BLAKE3 over (sorted path:hash\n) lines — deterministic.
pub fn generate_receipt(root: &Path, entries: &[FileEntry]) -> Result<ScanReceipt> {
    let id = format!("rcpt_{}", uuid::Uuid::new_v4());
    let timestamp = chrono::Utc::now().to_rfc3339();

    let hostname = std::process::Command::new("hostname")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "localhost".to_string());

    let mut receipt_entries: Vec<ReceiptEntry> = entries
        .iter()
        .map(|f| ReceiptEntry {
            path: f.path.to_string_lossy().to_string(),
            hash: f.hash.clone(),
            size: f.size,
        })
        .collect();
    receipt_entries.sort_by(|a, b| a.path.cmp(&b.path));

    // Root hash — BLAKE3 over sorted "path:hash\n" entries
    let mut hasher = Hasher::new();
    for e in &receipt_entries {
        hasher.update(format!("{}:{}\n", e.path, e.hash).as_bytes());
    }
    let root_hash = hasher.finalize().to_hex().to_string();

    let total_bytes: u64 = entries.iter().map(|f| f.size).sum();

    Ok(ScanReceipt {
        id,
        timestamp,
        schema_version: "1.0.0".to_string(),
        root_paths: vec![root.to_path_buf()],
        file_count: entries.len(),
        total_bytes,
        root_hash,
        entries: receipt_entries,
        system_info: SystemInfo {
            pid: std::process::id(),
            hostname,
            os: std::env::consts::OS.to_string(),
        },
    })
}

/// Verify no-deletion between two receipts.
pub fn verify_no_deletion(before: &ScanReceipt, after: &ScanReceipt) -> NoDeletionReport {
    use std::collections::HashMap;

    let before_map: HashMap<&str, &str> = before
        .entries
        .iter()
        .map(|e| (e.path.as_str(), e.hash.as_str()))
        .collect();
    let after_map: HashMap<&str, &str> = after
        .entries
        .iter()
        .map(|e| (e.path.as_str(), e.hash.as_str()))
        .collect();

    let deleted_files: Vec<String> = before_map
        .keys()
        .filter(|p| !after_map.contains_key(*p))
        .map(|s| s.to_string())
        .collect();

    let added_files: Vec<String> = after_map
        .keys()
        .filter(|p| !before_map.contains_key(*p))
        .map(|s| s.to_string())
        .collect();

    let modified_files: Vec<String> = before_map
        .iter()
        .filter(|(p, h)| after_map.get(*p).map_or(false, |ah| ah != *h))
        .map(|(p, _)| p.to_string())
        .collect();

    let pass = deleted_files.is_empty();

    NoDeletionReport {
        before_id: before.id.clone(),
        after_id: after.id.clone(),
        pass,
        deleted_files,
        added_files,
        modified_files,
    }
}
