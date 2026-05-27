# Handoff Report: Codebase Analysis of `capability-map`

This report summarizes the findings of the codebase scan and compilation check of `/Users/sac/capability-map`.

---

## 1. Observation

### Observation 1.1: Compiler Failures in `src/db.rs` and `src/report.rs`
Running `cargo check` in `/Users/sac/capability-map` produces 22 compiler errors. The command outputs:
```text
error[E0609]: no field `scan_run_id` on type `&ScanReceipt`
   --> src/db.rs:173:21
    |
173 |             r.id, r.scan_run_id, r.timestamp, r.roots, r.file_count as i64, r.dir_count as i64,
    |                     ^^^^^^^^^^^ unknown field

error[E0599]: no associated item named `Database` found for struct `anyhow::Error` in the current scope
   --> src/db.rs:205:33
    |
205 |         .map_err(|e| CpmpError::Database(e))?;
    |                                 ^^^^^^^^ associated item not found in `anyhow::Error`

error[E0599]: no associated item named `Database` found for struct `anyhow::Error` in the current scope
  --> src/report.rs:30:40
   |
30 |     ).map_err(crate::error::CpmpError::Database)?;
   |                                        ^^^^^^^^ associated item not found in `anyhow::Error`
```
These errors prevent compilation from succeeding.

### Observation 1.2: Declared Modules in `src/lib.rs`
Viewing `src/lib.rs` shows that both `db` and `report` are declared as public modules (lines 14–15):
```rust
14: pub mod db;
15: pub mod report;
```
This forces the Rust compiler to build `src/db.rs` and `src/report.rs`.

### Observation 1.3: Struct Definition Mismatch
Viewing `src/models.rs` shows the definition of `ScanReceipt` on lines 83–94:
```rust
pub struct ScanReceipt {
    pub id: String,
    pub timestamp: String,
    pub schema_version: String,
    pub root_paths: Vec<PathBuf>,
    pub file_count: usize,
    pub total_bytes: u64,
    pub root_hash: String,
    pub entries: Vec<ReceiptEntry>,
    pub system_info: SystemInfo,
}
```
This definition lacks fields like `scan_run_id`, `roots`, `dir_count`, `hash_algo`, `catalog_version`, `command_run`, `warnings`, `refusals`, `receipt_path` which are referenced in `src/db.rs` (lines 173–175) and `src/report.rs`.

### Observation 1.4: Command Mismatch in `scripts/smoke.sh`
Viewing `scripts/smoke.sh` reveals it executes commands that do not exist in `src/main.rs`:
- Line 60: `"$BINARY" summary --db "$DB"`
- Line 65: `"$BINARY" capability find Receipt --db "$DB"`
- Line 75: `"$BINARY" receipt list --db "$DB"`
- Line 102: `"$BINARY" report emit --db "$DB" --out "$OUT/reports"`

---

## 2. Logic Chain

1. **Step 1**: The build check tool was executed. It output verbatim compilation errors in `src/db.rs` and `src/report.rs` (Observation 1.1).
2. **Step 2**: The module declarations in `src/lib.rs` were inspected. They register `db` and `report` as active modules (Observation 1.2). Therefore, compilation cannot bypass these files.
3. **Step 3**: The errors in `src/db.rs` point to missing fields on `ScanReceipt` and a missing `Database` variant on `CpmpError` / `anyhow::Error`.
4. **Step 4**: The definition of `ScanReceipt` in `src/models.rs` (Observation 1.3) confirms it does not have the fields referenced by `src/db.rs` and `src/report.rs`. Furthermore, `CpmpError` is aliased directly to `anyhow::Error` in `src/error.rs`, which lacks any variant or associated function named `Database`.
5. **Step 5**: Therefore, the database and reporting modules are out of sync with the model definitions, causing compilation to fail.
6. **Step 6**: In addition, `scripts/smoke.sh` depends on legacy commands (Observation 1.4) that are completely absent from `src/main.rs`, meaning verification/smoke tests will fail even once compilation is fixed.

---

## 3. Caveats
- I did not attempt to fix the compilation errors, as the investigation constraint specifies read-only analysis.
- I assumed that the current active CLI subcommand structure in `src/main.rs` is the desired state under the Open Ontologies correction, and the SQLite `db.rs` is what needs to be adapted or refactored.

---

## 4. Conclusion
The codebase is currently **broken** and cannot compile. The primary cause of the compilation failure is a mismatch between the database/report modules and the `ScanReceipt` and `CpmpError` models. Furthermore, `scripts/smoke.sh` references CLI subcommands that do not exist in `src/main.rs`.

---

## 5. Verification Method

### Command to Execute:
To verify compilation status independently, run the following in `/Users/sac/capability-map`:
```bash
cargo check
```
This will output the 22 compilation errors detailed in Observation 1.1.

### Files to Inspect:
- `/Users/sac/capability-map/src/db.rs` — Lines 170–180 (incorrect fields on `ScanReceipt`).
- `/Users/sac/capability-map/src/models.rs` — Lines 83–94 (actual fields on `ScanReceipt`).
- `/Users/sac/capability-map/src/lib.rs` — Lines 14–15 (active compilation registration).

### Invalidation Conditions:
If `cargo check` compiles successfully or the errors disappear, this report is invalidated.
