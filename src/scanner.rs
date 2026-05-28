use std::path::PathBuf;
use anyhow::Result;
use ignore::WalkBuilder;
use crate::models::{FileEntry, Symbol, DetectedCapability};
use crate::db;
use crate::receipt;
use crate::projection;
use crate::symbol;
use crate::capability;

pub fn scan(paths: &[PathBuf], out: &PathBuf) -> Result<()> {
    std::fs::create_dir_all(out)?;
    let mut files = Vec::new();
    let mut all_symbols = Vec::new();
    let mut all_capabilities = Vec::new();
    for p in paths {
        let walker = WalkBuilder::new(p).hidden(false).git_ignore(true).build();
        for entry in walker.filter_map(|e| e.ok()) {
            if entry.file_type().map_or(false, |ft| ft.is_file()) {
                let path_str = entry.path().to_string_lossy().to_string();
                let content_bytes = std::fs::read(&entry.path()).unwrap_or_default();
                let hash = blake3::hash(&content_bytes).to_hex().to_string();
                let size = entry.metadata().map(|m| m.len() as i64).unwrap_or(0);
                let content = String::from_utf8_lossy(&content_bytes).to_string();
                files.push(FileEntry { path: path_str.clone(), hash, size, language: "".to_string() });
                all_symbols.extend(symbol::extract_symbols(&path_str, &content));
                all_capabilities.extend(capability::detect_capabilities(&path_str, &content));
            }
        }
    }
    let rec = receipt::generate_receipt(&files, out)?;
    db::init_db(out)?;
    db::insert_files(out, &files)?;
    db::insert_symbols(out, &all_symbols)?;
    db::insert_capabilities(out, &all_capabilities)?;
    projection::generate_reports(out, &files, &all_symbols, &all_capabilities)?;
    projection::generate_rdf_fallback(&files, &all_capabilities, &rec, out)?;
    Ok(())
}
