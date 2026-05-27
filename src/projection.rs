use anyhow::Result;
use std::path::Path;

/// Emit `CAPABILITY_INVENTORY.md` from the JSON inventory.
pub fn emit_capability_inventory(reports_dir: &Path, catalog_hash: &str) -> Result<()> {
    let inv_path = reports_dir.join("capability_inventory.json");
    if !inv_path.exists() {
        return Ok(());
    }

    let raw = std::fs::read_to_string(&inv_path)?;
    let caps: Vec<serde_json::Value> = serde_json::from_str(&raw)?;

    let mut md = String::from("# Capability Inventory\n\n");
    md.push_str(&format!("> Source graph hash: `{catalog_hash}`\n\n"));
    md.push_str("| Capability | Classification | File | Line | Matched Term |\n");
    md.push_str("|-----------|---------------|------|------|-------------|\n");

    for cap in &caps {
        let name   = cap["capability"].as_str().unwrap_or("?");
        let cls    = cap["classification"].as_str().unwrap_or("?");
        let file   = cap["file_path"].as_str().unwrap_or("?");
        let line   = cap["line"].as_u64().unwrap_or(0);
        let term   = cap["matched_term"].as_str().unwrap_or("?");
        md.push_str(&format!("| {name} | {cls} | `{file}` | {line} | `{term}` |\n"));
    }

    std::fs::write(reports_dir.join("CAPABILITY_INVENTORY.md"), md)?;
    Ok(())
}

/// Emit `PROJECT_ATLAS.md` summarising the file inventory.
pub fn emit_project_atlas(reports_dir: &Path, catalog_hash: &str) -> Result<()> {
    let inv_path = reports_dir.join("file_inventory.json");
    if !inv_path.exists() {
        return Ok(());
    }

    let raw = std::fs::read_to_string(&inv_path)?;
    let files: Vec<serde_json::Value> = serde_json::from_str(&raw)?;

    let mut lang_counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for f in &files {
        let lang = f["language"].as_str().unwrap_or("unknown");
        *lang_counts.entry(lang).or_insert(0) += 1;
    }

    let mut md = String::from("# Project Atlas\n\n");
    md.push_str(&format!("> Source graph hash: `{catalog_hash}`\n\n"));
    md.push_str(&format!("Total files: {}\n\n", files.len()));
    md.push_str("## By Language\n\n| Language | Files |\n|----------|-------|\n");

    let mut langs: Vec<_> = lang_counts.iter().collect();
    langs.sort_by(|a, b| b.1.cmp(a.1));
    for (lang, count) in langs {
        md.push_str(&format!("| {lang} | {count} |\n"));
    }

    std::fs::write(reports_dir.join("PROJECT_ATLAS.md"), md)?;
    Ok(())
}

/// Emit `PATTERN_ATLAS.md` from the symbol index.
pub fn emit_pattern_atlas(reports_dir: &Path, catalog_hash: &str) -> Result<()> {
    let sym_path = reports_dir.join("symbol_index.json");
    if !sym_path.exists() {
        return Ok(());
    }

    let raw = std::fs::read_to_string(&sym_path)?;
    let syms: Vec<serde_json::Value> = serde_json::from_str(&raw)?;

    let mut kind_counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for s in &syms {
        let kind = s["kind"].as_str().unwrap_or("unknown");
        *kind_counts.entry(kind).or_insert(0) += 1;
    }

    let mut md = String::from("# Pattern Atlas\n\n");
    md.push_str(&format!("> Source graph hash: `{catalog_hash}`\n\n"));
    md.push_str(&format!("Total symbols extracted: {}\n\n", syms.len()));
    md.push_str("## Symbol Kinds\n\n| Kind | Count |\n|------|-------|\n");

    let mut kinds: Vec<_> = kind_counts.iter().collect();
    kinds.sort_by(|a, b| b.1.cmp(a.1));
    for (kind, count) in kinds {
        md.push_str(&format!("| {kind} | {count} |\n"));
    }

    md.push_str("\n## Sample Symbols\n\n| Name | Kind | File | Line |\n|------|------|------|------|\n");
    for sym in syms.iter().take(50) {
        let name = sym["name"].as_str().unwrap_or("?");
        let kind = sym["kind"].as_str().unwrap_or("?");
        let file = sym["file_path"].as_str().unwrap_or("?");
        let line = sym["line"].as_u64().unwrap_or(0);
        md.push_str(&format!("| {name} | {kind} | `{file}` | {line} |\n"));
    }

    std::fs::write(reports_dir.join("PATTERN_ATLAS.md"), md)?;
    Ok(())
}

/// Emit all standard reports from validated graph data.
pub fn emit_all(reports_dir: &Path, catalog_hash: &str) -> Result<()> {
    emit_capability_inventory(reports_dir, catalog_hash)?;
    emit_project_atlas(reports_dir, catalog_hash)?;
    emit_pattern_atlas(reports_dir, catalog_hash)?;
    Ok(())
}
