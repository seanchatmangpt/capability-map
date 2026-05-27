# Task: Milestone 2 Remediation (Gen 3 - Replacement)

## Objective
Restore the original integration tests under `tests/` exactly as they were written, and align the codebase under `src/` to make everything compile and pass cleanly without stubs/mocks.

## Code of original tests to restore:

### 1. `tests/scanner_tests.rs`
```rust
use std::fs::{self, File};
use std::io::Write;
use std::process::Command;
use tempfile::tempdir;
use capability_map::scanner::{scan_directory, FileType};

#[test]
fn test_scanner_basic_and_ignore() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // Create files
    let file_rs = root.join("main.rs");
    fs::write(&file_rs, "fn main() {}").unwrap();

    let file_txt = root.join("readme.txt");
    fs::write(&file_txt, "Hello documentation").unwrap();

    let ignore_dir = root.join(".git");
    fs::create_dir(&ignore_dir).unwrap();
    fs::write(ignore_dir.join("config"), "some git config").unwrap();

    let agents_dir = root.join(".agents");
    fs::create_dir(&agents_dir).unwrap();
    fs::write(agents_dir.join("agent_info.md"), "agent details").unwrap();

    // Scan
    let entries = scan_directory(&[root.to_path_buf()]).unwrap();

    // Verify .git and .agents are ignored, main.rs and readme.txt are found
    assert!(entries.iter().any(|e| e.path.to_str() == Some("main.rs")));
    assert!(entries.iter().any(|e| e.path.to_str() == Some("readme.txt")));
    assert!(!entries.iter().any(|e| e.path.to_str().unwrap().contains(".git")));
    assert!(!entries.iter().any(|e| e.path.to_str().unwrap().contains(".agents")));

    // Check file types
    let rs_entry = entries.iter().find(|e| e.path.to_str() == Some("main.rs")).unwrap();
    assert_eq!(rs_entry.file_type, FileType::RustSource);

    let doc_entry = entries.iter().find(|e| e.path.to_str() == Some("readme.txt")).unwrap();
    assert_eq!(doc_entry.file_type, FileType::Documentation);
}

#[test]
fn test_scanner_wasm_magic_bytes() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // Write a file with .dat extension but with WASM magic bytes
    let wasm_file = root.join("module.dat");
    let mut file = File::create(&wasm_file).unwrap();
    file.write_all(&[0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00]).unwrap(); // \0asm + version

    let entries = scan_directory(&[root.to_path_buf()]).unwrap();
    let wasm_entry = entries.iter().find(|e| e.path.to_str() == Some("module.dat")).unwrap();
    assert_eq!(wasm_entry.file_type, FileType::WasmArtifact);
}

#[test]
fn test_scanner_git_and_cargo_metadata() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // Initialize a git repo physically
    let status = Command::new("git")
        .arg("init")
        .current_dir(root)
        .status();

    if status.is_err() || !status.unwrap().success() {
        fs::create_dir_all(root.join(".git")).unwrap();
    }

    // Create a Cargo.toml representing a package
    let cargo_toml = root.join("Cargo.toml");
    fs::write(&cargo_toml, r#"
[package]
name = "test-pkg"
version = "0.1.0"
"#).unwrap();

    let src_dir = root.join("src");
    fs::create_dir(&src_dir).unwrap();
    let lib_rs = src_dir.join("lib.rs");
    fs::write(&lib_rs, "pub fn add(a: i32, b: i32) -> i32 { a + b }").unwrap();

    let entries = scan_directory(&[root.to_path_buf()]).unwrap();
    let lib_entry = entries.iter().find(|e| e.path.to_str() == Some("src/lib.rs")).unwrap();

    assert!(lib_entry.metadata.in_git_repo);
    assert_eq!(lib_entry.metadata.git_repo_root.as_ref().map(|p| p.canonicalize().unwrap()), Some(root.canonicalize().unwrap()));
    assert!(lib_entry.metadata.in_cargo_package);
    assert_eq!(lib_entry.metadata.cargo_package_name.as_deref(), Some("test-pkg"));
}
```

### 2. `tests/receipt_tests.rs`
```rust
use std::fs;
use tempfile::tempdir;
use capability_map::scanner::scan_directory;
use capability_map::receipt::{generate_receipt, verify_no_deletion};

#[test]
fn test_receipt_generation_and_no_deletion_verify() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // Create initial state of the files
    let file1 = root.join("a.rs");
    let file2 = root.join("b.rs");
    fs::write(&file1, "fn a() {}").unwrap();
    fs::write(&file2, "fn b() {}").unwrap();

    let entries_before = scan_directory(&[root.to_path_buf()]).unwrap();
    let receipt_before = generate_receipt(root, &entries_before).unwrap();

    // 1. Verify basic receipt properties
    assert_eq!(receipt_before.schema_version, "1.0.0");
    assert_eq!(receipt_before.entries.len(), 2);
    assert_eq!(receipt_before.entries[0].path, "a.rs");
    assert_eq!(receipt_before.entries[1].path, "b.rs");
    assert!(!receipt_before.root_hash.is_empty());
    
    // Verify system info is retrieved correctly and has no empty placeholder strings
    assert!(receipt_before.system_info.pid > 0);
    assert!(!receipt_before.system_info.hostname.is_empty());
    assert!(!receipt_before.system_info.os.is_empty());
    assert_ne!(receipt_before.system_info.hostname, "uuid_placeholder");
    assert_ne!(receipt_before.system_info.hostname, "TODO");

    // 2. Scan and verify without modifications
    let entries_same = scan_directory(&[root.to_path_buf()]).unwrap();
    let receipt_same = generate_receipt(root, &entries_same).unwrap();
    let report_same = verify_no_deletion(&receipt_before, &receipt_same).unwrap();
    
    assert!(report_same.is_valid);
    assert_eq!(report_same.matched_count, 2);
    assert!(report_same.added_files.is_empty());
    assert!(report_same.modified_files.is_empty());
    assert!(report_same.deleted_files.is_empty());
    assert!(report_same.violations.is_empty());

    // 3. Add a file (should be valid under non-deletion check, but listed in added)
    let file3 = root.join("c.rs");
    fs::write(&file3, "fn c() {}").unwrap();
    
    let entries_add = scan_directory(&[root.to_path_buf()]).unwrap();
    let receipt_add = generate_receipt(root, &entries_add).unwrap();
    let report_add = verify_no_deletion(&receipt_before, &receipt_add).unwrap();

    assert!(report_add.is_valid);
    assert_eq!(report_add.matched_count, 2);
    assert_eq!(report_add.added_files, vec!["c.rs".to_string()]);
    assert!(report_add.modified_files.is_empty());
    assert!(report_add.deleted_files.is_empty());
    assert!(report_add.violations.is_empty());

    // 4. Modify an existing file (should be invalid under non-deletion check)
    fs::write(&file1, "fn a_modified() {}").unwrap();
    
    let entries_mod = scan_directory(&[root.to_path_buf()]).unwrap();
    let receipt_mod = generate_receipt(root, &entries_mod).unwrap();
    let report_mod = verify_no_deletion(&receipt_before, &receipt_mod).unwrap();

    assert!(!report_mod.is_valid);
    assert_eq!(report_mod.modified_files, vec!["a.rs".to_string()]);
    assert!(report_mod.violations.iter().any(|v| v.contains("a.rs")));

    // Restore file 1 to original state, but delete file 2
    fs::write(&file1, "fn a() {}").unwrap();
    fs::remove_file(&file2).unwrap();

    let entries_del = scan_directory(&[root.to_path_buf()]).unwrap();
    let receipt_del = generate_receipt(root, &entries_del).unwrap();
    let report_del = verify_no_deletion(&receipt_before, &receipt_del).unwrap();

    assert!(!report_del.is_valid);
    assert_eq!(report_del.deleted_files, vec!["b.rs".to_string()]);
    assert!(report_del.violations.iter().any(|v| v.contains("b.rs")));
}
```

### 3. `tests/integration_test.rs`
```rust
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;
use capability_map::scanner::{scan_directory, FileType};
use capability_map::symbol::{extract_symbols, Symbol};
use capability_map::capability::detect_capabilities;
use capability_map::classification::{classify_file, Classification};
use capability_map::db::insert_catalog;
use capability_map::projection::generate_projections;
use capability_map::models::{Receipt, ReceiptEntry};

#[test]
fn test_full_pipeline_verification() {
    let dir = tempdir().unwrap();
    let root = dir.path().to_path_buf();

    // 1. Create a fixture directory with a capability seed and test file
    let fixture_dir = root.join("fixtures");
    fs::create_dir_all(&fixture_dir).unwrap();

    let cap_file = fixture_dir.join("genesis_seed.rs");
    let test_file = fixture_dir.join("seed_test.rs");

    fs::write(&cap_file, "pub fn init_genesis() {}").unwrap();
    fs::write(&test_file, "pub fn test_init_genesis() {}").unwrap();

    // Verify non-deletion invariant tracking:
    // Before scan, we have 2 files.
    let pre_scan_count = fs::read_dir(&fixture_dir).unwrap().count();
    assert_eq!(pre_scan_count, 2);

    // 2. Scan fixture directory & Hash files
    let paths = vec![fixture_dir.clone()];
    let entries = scan_directory(&paths).expect("Scan failed");
    assert_eq!(entries.len(), 2, "Should detect exactly 2 files");

    // File 1 check
    let cap_entry = entries.iter().find(|e| e.path.ends_with("genesis_seed.rs")).unwrap();
    assert_eq!(cap_entry.file_type, FileType::RustSource);
    assert!(!cap_entry.hash.is_empty(), "Hash must be populated");

    // 3. Database Operations
    let db_path = root.join("test.db");
    let mut conn = rusqlite::Connection::open(&db_path).unwrap();

    let cap_content = fs::read_to_string(&cap_file).unwrap();
    let symbols = extract_symbols(&cap_file, &cap_content);
    assert_eq!(symbols.len(), 1);
    assert_eq!(symbols[0].name, "init_genesis");

    let capabilities = detect_capabilities(&cap_file, &cap_content, &symbols);
    assert_eq!(capabilities.len(), 1);
    assert_eq!(capabilities[0].name, "init_genesis");

    // 4. Test Detection
    let classification = classify_file(&test_file, &[Symbol { name: "test_init_genesis".to_string(), kind: "function".to_string(), line: 1 }]);
    assert_eq!(classification, Classification::Partial);

    // Insert into SQLite
    insert_catalog(&mut conn, &entries, &symbols, &capabilities).unwrap();

    let count: i64 = conn.query_row("SELECT COUNT(*) FROM files", [], |row| row.get(0)).unwrap();
    assert_eq!(count, 2);

    let count: i64 = conn.query_row("SELECT COUNT(*) FROM capabilities", [], |row| row.get(0)).unwrap();
    assert_eq!(count, 1);

    // 5. Markdown Report Emission
    let out_dir = root.join("out");
    generate_projections(&conn, &out_dir).unwrap();

    assert!(out_dir.join("CAPABILITY_INVENTORY.md").exists());
    assert!(out_dir.join("PATTERN_ATLAS.md").exists());

    // 6. Receipt Emission
    let receipt = Receipt::new();
    let r_path = root.join("receipt.json");
    // In this basic version we just simulate saving it
    assert!(receipt.id.starts_with("rcpt_"));

    // 7. Verify no deletion occurred
    let post_scan_count = fs::read_dir(&fixture_dir).unwrap().count();
    assert_eq!(pre_scan_count, post_scan_count);
    assert!(cap_file.exists());
    assert!(test_file.exists());
}
```

## Alignment Steps for the Codebase:

1. **Remove `file_path` from `Symbol`**:
   - In `src/models.rs`, define `Symbol` exactly as:
     ```rust
     #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
     pub struct Symbol {
         pub name: String,
         pub kind: String,
         pub line: usize,
     }
     ```
   - Remove `file_path` references from `src/symbol.rs` where `Symbol` is instantiated (e.g. `find_matches` and specific parser blocks).

2. **Align `Classification` Enum**:
   - In `src/classification.rs`, define the enum as:
     ```rust
     #[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
     pub enum Classification {
         Live,
         Partial,
         CAPABILITY_SEED,
         LEGACY_NAME,
         Dormant,
         BROKEN_BUT_REAL,
         DocOnly,
         TestOnly,
         Ambiguous,
     }
     ```
   - Update `as_str()` and other references to match.

3. **Align `insert_catalog` in `src/db.rs`**:
   - In `insert_catalog`, map the `Symbol` objects to their respective file paths.
   - For each entry in `entries`, scan the file for its symbols, and identify which of the passed `symbols` match those extracted symbols. Use this mapping to write to the SQLite database.
   - Example helper strategy:
     ```rust
     let mut sym_to_path = std::collections::HashMap::new();
     for entry in entries {
         let path_str = entry.path.to_string_lossy().to_string();
         if let Ok(content) = std::fs::read_to_string(&entry.path) {
             let extracted = crate::symbol::extract_symbols(&entry.path, &content);
             for ext in extracted {
                 sym_to_path.insert((ext.name, ext.kind, ext.line), path_str.clone());
             }
         }
     }
     ```
   - When iterating over the passed `symbols`, retrieve the `file_path` using this lookup.

4. **Verify compile and pass**:
   - Run `cargo test` and ensure all tests compile and pass cleanly without stubs/mocks.

## Safety & Integrity
- DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task.
- A Forensic Auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.
