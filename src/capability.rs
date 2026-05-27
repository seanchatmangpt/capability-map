use crate::models::{Classification, DetectedCapability, Symbol};
use std::path::Path;

/// Named capability terms (29 required terms). Each entry: (capability_name, &[keyword_terms]).
static CAPABILITY_TERMS: &[(&str, &[&str])] = &[
    ("Genesis",        &["genesis", "Genesis", "GENESIS"]),
    ("ggen",           &["ggen", "Ggen", "GGEN"]),
    ("Truex",          &["truex", "Truex", "TRUEX"]),
    ("Receipt",        &["Receipt", "ScanReceipt", "generate_receipt", "receipt"]),
    ("Replay",         &["replay", "Replay", "REPLAY"]),
    ("Refusal",        &["refusal", "Refusal", "REFUSAL"]),
    ("Construct8",     &["Construct8", "construct8", "CONSTRUCT8"]),
    ("Pair2",          &["Pair2", "pair2", "PAIR2"]),
    ("RelationPage",   &["RelationPage", "relation_page"]),
    ("Need9",          &["Need9", "need9", "NEED9"]),
    ("Need257",        &["Need257", "need257", "NEED257"]),
    ("Shard",          &["shard", "Shard", "SHARD"]),
    ("Segment",        &["segment", "Segment", "SEGMENT"]),
    ("Corpus",         &["corpus", "Corpus", "CORPUS"]),
    ("O*",             &["O*", "Ostar", "ostar"]),
    ("mu",             &["mu", "MU", "Mu"]),
    ("AtomVM",         &["AtomVM", "atomvm", "atom_vm"]),
    ("Erlang",         &["erlang", "Erlang", "ERLANG"]),
    ("WASM",           &["wasm", "WASM", "WebAssembly", "wasmtime"]),
    ("POWL",           &["powl", "POWL", "Powl"]),
    ("OCEL",           &["ocel", "OCEL", "Ocel"]),
    ("PROV",           &["prov:", "wasGeneratedBy", "prov:Activity"]),
    ("SHACL",          &["shacl", "SHACL", "NodeShape"]),
    ("DCAT",           &["dcat:", "dcat:Catalog", "dcat:Dataset"]),
    ("Field8",         &["Field8", "field8", "FIELD8"]),
    ("Instinct8",      &["Instinct8", "instinct8", "INSTINCT8"]),
    ("Doctor",         &["doctor", "Doctor", "DOCTOR"]),
    ("Wizard",         &["wizard", "Wizard", "WIZARD"]),
    ("Telco",          &["telco", "Telco", "TELCO"]),
];

/// Classify a file path based on its location, extension, and content keywords.
pub fn classify_path(path: &Path, content: &str) -> Classification {
    let path_str = path.to_string_lossy().to_lowercase();

    if path_str.contains("/test") || path_str.ends_with("_test.rs") || path_str.contains("spec") {
        return Classification::TestOnly;
    }
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if matches!(ext, "md" | "rst" | "txt") {
        return Classification::DocOnly;
    }

    // Inline keyword checks
    if content.contains("DORMANT") || content.contains("dormant") {
        return Classification::Dormant;
    }
    if content.contains("BROKEN_BUT_REAL") || content.contains("broken_but_real") {
        return Classification::BrokenButReal;
    }
    if content.contains("CAPABILITY_SEED") || content.contains("capability_seed") {
        return Classification::CapabilitySeed;
    }
    if content.contains("LEGACY_NAME") || content.contains("legacy_name") {
        return Classification::LegacyName;
    }

    let has_fn = content.contains("fn ") || content.contains("def ") || content.contains("func ");
    let has_unimplemented = content.contains("unimplemented!") || content.contains("todo!()");

    if has_unimplemented {
        return Classification::Partial;
    }
    if has_fn {
        return Classification::Live;
    }

    Classification::Ambiguous
}

/// Detect named capabilities in a file, augmented by extracted symbols.
pub fn detect_capabilities(
    path: &Path,
    content: &str,
    symbols: &[Symbol],
) -> Vec<DetectedCapability> {
    let mut caps = Vec::new();
    let classification = classify_path(path, content);

    for (cap_name, terms) in CAPABILITY_TERMS {
        for term in *terms {
            // Check in content
            if let Some(byte_offset) = content.find(term) {
                let line = content[..byte_offset].chars().filter(|&c| c == '\n').count() + 1;
                caps.push(DetectedCapability {
                    capability: cap_name.to_string(),
                    matched_term: term.to_string(),
                    file_path: path.to_path_buf(),
                    line,
                    classification: classification.clone(),
                });
                break;
            }
            // Check in symbol names
            if symbols.iter().any(|s| s.name.contains(term)) {
                caps.push(DetectedCapability {
                    capability: cap_name.to_string(),
                    matched_term: format!("symbol:{term}"),
                    file_path: path.to_path_buf(),
                    line: 0,
                    classification: classification.clone(),
                });
                break;
            }
        }
    }

    caps
}
