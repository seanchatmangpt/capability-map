use cpmp::{capability, receipt, rdf, scanner, symbol};
use std::path::PathBuf;
use tempfile::TempDir;

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join("tiny-repo")
}

// ─── Scanner Integration Tests ────────────────────────────────────────────────

#[test]
fn test_scan_produces_files_and_receipt() {
    let out_dir = TempDir::new().unwrap();
    let fixture = fixture_path();

    let rec = scanner::scan(&[fixture.clone()], out_dir.path()).unwrap();

    // Real boundary: filesystem scan produced evidence
    assert!(rec.file_count > 0, "scanner must find at least one file");
    assert!(!rec.root_hash.is_empty(), "receipt must have root_hash");
    assert!(!rec.id.is_empty(), "receipt must have id");

    // Receipt file was written to disk
    let receipts_dir = out_dir.path().join("receipts");
    let receipt_files: Vec<_> = std::fs::read_dir(&receipts_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert!(!receipt_files.is_empty(), "receipt TOML must be written");

    // Catalog files exist
    let catalog_dir = out_dir.path().join("catalog");
    assert!(catalog_dir.join("cpmp-catalog.ttl").exists(), "Turtle catalog must exist");
    assert!(catalog_dir.join("cpmp-catalog.nq").exists(), "N-Quads catalog must exist");
    assert!(catalog_dir.join("cpmp-shapes.ttl").exists(), "SHACL shapes must exist");
}

// ─── RDF Content Tests ────────────────────────────────────────────────────────

#[test]
fn test_catalog_ttl_contains_required_vocabulary() {
    let out_dir = TempDir::new().unwrap();
    let fixture = fixture_path();
    scanner::scan(&[fixture], out_dir.path()).unwrap();

    let ttl = std::fs::read_to_string(out_dir.path().join("catalog/cpmp-catalog.ttl")).unwrap();

    // PROV-O lineage (full IRI — oxigraph 0.4 doesn't add prefix declarations)
    assert!(ttl.contains("wasGeneratedBy"), "TTL must have PROV lineage");
    // DCAT catalog
    assert!(ttl.contains("http://www.w3.org/ns/dcat#") || ttl.contains("dcat:"), "TTL must have DCAT vocab");
    // SPDX files and checksums
    assert!(ttl.contains("http://spdx.org/rdf/terms#") || ttl.contains("spdx:"), "TTL must have SPDX vocab");
    // No private predicate authority
    assert!(!ttl.contains("gall:"), "TTL must not use gall: namespace");
}

#[test]
fn test_catalog_ttl_validates_with_open_ontologies() {
    let out_dir = TempDir::new().unwrap();
    let fixture = fixture_path();
    scanner::scan(&[fixture], out_dir.path()).unwrap();

    let ttl_path = out_dir.path().join("catalog/cpmp-catalog.ttl");

    // Real boundary: open-ontologies must parse our Turtle
    let status = std::process::Command::new("open-ontologies")
        .arg("validate")
        .arg(&ttl_path)
        .status()
        .expect("open-ontologies must be installed");

    assert!(
        status.success(),
        "open-ontologies validate must succeed on generated Turtle"
    );
}

// ─── Receipt / No-Deletion Tests ──────────────────────────────────────────────

#[test]
fn test_no_deletion_pass_when_no_files_removed() {
    let fixture = fixture_path();
    let mut entries = Vec::new();

    for e in walkdir::WalkDir::new(&fixture)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = e.path().to_path_buf();
        let bytes = std::fs::read(&path).unwrap();
        let hash = blake3::hash(&bytes).to_hex().to_string();
        let size = e.metadata().unwrap().len();
        entries.push(cpmp::models::FileEntry {
            path,
            hash,
            size,
            language: "".to_string(),
        });
    }

    let before = receipt::generate_receipt(&fixture, &entries).unwrap();
    let after  = receipt::generate_receipt(&fixture, &entries).unwrap();

    let report = receipt::verify_no_deletion(&before, &after);
    assert!(report.pass, "identical scan must pass no-deletion check");
    assert!(report.deleted_files.is_empty());
}

#[test]
fn test_no_deletion_fail_when_file_removed() {
    let fixture = fixture_path();
    let entries: Vec<_> = vec![
        cpmp::models::FileEntry {
            path: fixture.join("src/main.rs"),
            hash: "aaaa".to_string(),
            size: 100,
            language: "rust".to_string(),
        },
        cpmp::models::FileEntry {
            path: fixture.join("README.md"),
            hash: "bbbb".to_string(),
            size: 50,
            language: "markdown".to_string(),
        },
    ];
    // After: one file gone
    let after_entries = entries[..1].to_vec();

    let before = receipt::generate_receipt(&fixture, &entries).unwrap();
    let after  = receipt::generate_receipt(&fixture, &after_entries).unwrap();

    let report = receipt::verify_no_deletion(&before, &after);
    assert!(!report.pass, "removed file must fail no-deletion");
    assert!(!report.deleted_files.is_empty(), "deleted_files must be populated");
}

// ─── Symbol Extraction Tests ──────────────────────────────────────────────────

#[test]
fn test_symbols_extracted_from_fixture() {
    let fixture = fixture_path().join("src/main.rs");
    let content = std::fs::read_to_string(&fixture).unwrap();
    let syms = symbol::extract_symbols(&fixture, &content);

    // The fixture has: fn main, fn compute, struct Receipt, trait Capability
    let fn_names: Vec<_> = syms.iter().filter(|s| s.kind == "fn").map(|s| &s.name).collect();
    assert!(
        fn_names.iter().any(|n| *n == "compute" || *n == "main"),
        "must detect fn symbols, got: {:?}", fn_names
    );

    let structs: Vec<_> = syms.iter().filter(|s| s.kind == "struct").collect();
    assert!(!structs.is_empty(), "must detect struct symbols");
}

// ─── Capability Detection Tests ───────────────────────────────────────────────

#[test]
fn test_capabilities_detected_in_readme() {
    let fixture = fixture_path().join("README.md");
    let content = std::fs::read_to_string(&fixture).unwrap();
    let caps = capability::detect_capabilities(&fixture, &content, &[]);

    let cap_names: Vec<_> = caps.iter().map(|c| c.capability.as_str()).collect();
    // README mentions open-ontologies, BLAKE3, Receipt
    assert!(
        cap_names.contains(&"OpenOntologies") || cap_names.contains(&"BLAKE3") || cap_names.contains(&"Receipt"),
        "must detect at least one capability in README, got: {:?}", cap_names
    );
}

// ─── Policy Pack Tests ────────────────────────────────────────────────────────

#[test]
fn test_policy_checks_pass_after_valid_scan() {
    let out_dir = TempDir::new().unwrap();
    let fixture = fixture_path();
    scanner::scan(&[fixture], out_dir.path()).unwrap();

    let catalog_dir = out_dir.path().join("catalog");
    let checks = cpmp::policy::run_policy_checks(&catalog_dir);

    let failures: Vec<_> = checks.iter().filter(|c| {
        matches!(&c.result, cpmp::policy::GateResult::Fail { .. } | cpmp::policy::GateResult::Refusal { .. })
    }).collect();

    assert!(
        failures.is_empty(),
        "all policy packs must pass after valid scan, failures: {:?}",
        failures.iter().map(|c| format!("[{}] {}", c.pack, c.result)).collect::<Vec<_>>()
    );
}
