use crate::models::{FileEntry, ScanReceipt};
use crate::{capability, receipt, rdf, symbol};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Scan the given paths, emit RDF catalog, receipts, and reports under `out_dir`.
/// Target directories are opened READ-ONLY; nothing is written into them.
pub fn scan(paths: &[PathBuf], out_dir: &Path) -> Result<ScanReceipt> {
    std::fs::create_dir_all(out_dir)?;
    std::fs::create_dir_all(out_dir.join("receipts"))?;
    std::fs::create_dir_all(out_dir.join("reports"))?;
    std::fs::create_dir_all(out_dir.join("catalog"))?;

    let mut files: Vec<FileEntry> = Vec::new();
    let mut all_symbols = Vec::new();

    for root in paths {
        let walker = WalkDir::new(root)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file());

        for entry in walker {
            let path = entry.path().to_path_buf();

            if is_ignored(&path) {
                continue;
            }

            let content = std::fs::read_to_string(&path).unwrap_or_default();
            let bytes   = std::fs::read(&path).unwrap_or_default();
            let hash    = blake3::hash(&bytes).to_hex().to_string();
            let size    = entry.metadata().map_or(0, |m| m.len());
            let language = guess_language(&path);

            let syms = symbol::extract_symbols(&path, &content);
            all_symbols.extend(syms);

            files.push(FileEntry { path, hash, size, language });
        }
    }

    // Collect capabilities
    let mut all_capabilities = Vec::new();
    for file in &files {
        let content = std::fs::read_to_string(&file.path).unwrap_or_default();
        let file_syms: Vec<_> = all_symbols
            .iter()
            .filter(|s| s.file_path == file.path)
            .cloned()
            .collect();
        let caps = capability::detect_capabilities(&file.path, &content, &file_syms);
        all_capabilities.extend(caps);
    }

    // Build receipt
    let root_arg = paths.first().map(|p| p.as_path()).unwrap_or(out_dir);
    let rec = receipt::generate_receipt(root_arg, &files)?;

    // Write scan receipt as TOML
    let ts = chrono::Utc::now().timestamp();
    let receipt_path = out_dir.join("receipts").join(format!("scan-{ts}.receipt.toml"));
    let receipt_toml = toml::to_string(&rec).context("serialising receipt")?;
    std::fs::write(&receipt_path, &receipt_toml)?;

    // Build RDF catalog
    let catalog_dir = out_dir.join("catalog");
    rdf::build_and_emit(&files, &all_capabilities, &rec, &catalog_dir)?;

    // Write JSON inventories
    let inv_path = out_dir.join("reports").join("capability_inventory.json");
    std::fs::write(&inv_path, serde_json::to_string_pretty(&all_capabilities)?)?;

    let sym_path = out_dir.join("reports").join("symbol_index.json");
    std::fs::write(&sym_path, serde_json::to_string_pretty(&all_symbols)?)?;

    let file_path = out_dir.join("reports").join("file_inventory.json");
    std::fs::write(&file_path, serde_json::to_string_pretty(&files)?)?;

    eprintln!(
        "Scan complete: {} files  {} symbols  {} capabilities",
        files.len(), all_symbols.len(), all_capabilities.len()
    );

    Ok(rec)
}

fn is_ignored(path: &Path) -> bool {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    matches!(
        ext,
        "png" | "jpg" | "jpeg" | "gif" | "svg" | "ico" | "woff" | "woff2" |
        "ttf" | "eot" | "mp4" | "mp3" | "zip" | "tar" | "gz" | "pdf"
    )
}

fn guess_language(path: &Path) -> String {
    match path.extension().and_then(|e| e.to_str()).unwrap_or("") {
        "rs"   => "rust",
        "py"   => "python",
        "ts"   => "typescript",
        "js"   => "javascript",
        "go"   => "go",
        "rb"   => "ruby",
        "java" => "java",
        "c" | "h" => "c",
        "cpp" | "hpp" => "cpp",
        "toml" => "toml",
        "yaml" | "yml" => "yaml",
        "json" => "json",
        "md"   => "markdown",
        "sh"   => "shell",
        "ttl"  => "turtle",
        "nq"   => "nquads",
        ext    => ext,
    }.to_string()
}
