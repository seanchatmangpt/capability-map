use std::path::PathBuf;
use anyhow::{Result, anyhow};
use crate::models::{FileEntry, Receipt};
use uuid::Uuid;
pub fn generate_receipt(files: &[FileEntry], out: &PathBuf) -> Result<Receipt> {
    let id = Uuid::new_v4().to_string();
    let total_bytes = files.iter().map(|f| f.size).sum();
    let mut aggregate_hasher = blake3::Hasher::new();
    for f in files { aggregate_hasher.update(f.hash.as_bytes()); }
    let aggregate_hash = aggregate_hasher.finalize().to_hex().to_string();
    let receipt = Receipt { id, timestamp: chrono::Utc::now().to_rfc3339(), aggregate_hash, file_count: files.len() as i64, total_bytes, files: files.to_vec() };
    std::fs::create_dir_all(out.join("receipts"))?;
    let receipt_path = out.join(format!("receipts/scan_{}.toml", receipt.id));
    let toml = toml::to_string(&receipt)?;
    std::fs::write(&receipt_path, toml)?;
    Ok(receipt)
}
pub fn verify_no_deletion(before_path: &PathBuf, after_path: &PathBuf) -> Result<()> {
    let before_content = std::fs::read_to_string(before_path)?;
    let after_content = std::fs::read_to_string(after_path)?;
    let before: Receipt = toml::from_str(&before_content)?;
    let after: Receipt = toml::from_str(&after_content)?;
    let mut missing = Vec::new();
    for bf in before.files { if !after.files.iter().any(|af| af.path == bf.path) { missing.push(bf.path); } }
    if missing.is_empty() { println!("Verification passed. No files deleted."); Ok(()) } 
    else { println!("REFUSAL: Files missing in after receipt:"); for p in missing { println!("- {}", p); } Err(anyhow!("Non-deletion law violated!")) }
}
