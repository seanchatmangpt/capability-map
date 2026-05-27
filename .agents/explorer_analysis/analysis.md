# Codebase Analysis Report: `capability-map` / `cpmp`

## Executive Summary
This report analyzes the implementation state of the project `capability-map` (historically renamed to `cpmp` in the codebase) located at `/Users/sac/capability-map`. 
**The project is currently in a broken compilation state.** While some parts of the scanner and RDF generator logic exist, the inclusion of `src/db.rs` and `src/report.rs` in `src/lib.rs` triggers 22 compiler errors due to outdated struct references and missing enum variants.

---

## 1. Compilation & Test Status
- **Compilation Status**: **FAILED** (exit code 101).
  - The project does not compile because `src/db.rs` and `src/report.rs` are registered in `src/lib.rs` (lines 14–15), but they use an obsolete version of the `ScanReceipt` struct and expect a non-existent `CpmpError::Database` variant on `anyhow::Error` (which is aliased as `CpmpError` in `src/error.rs`).
  - Verbatim compiler errors include:
    - `error[E0609]: no field scan_run_id on type &ScanReceipt` (in `src/db.rs:173:21`)
    - `error[E0599]: no associated item named Database found for struct anyhow::Error in the current scope` (in `src/db.rs:205:33`)
    - `error[E0599]: no associated item named Database found for struct anyhow::Error in the current scope` (in `src/report.rs:30:40`, etc.)
- **Test Status**: Cannot be run because compilation fails.

---

## 2. Inventory of Implemented Logic

### CLI Commands (in `src/main.rs`)
- `cpmp computer discover <paths...> [--out <dir>] [--with-gates]`
  - Runs a filesystem scan on given paths.
  - Generates a `ScanReceipt` using `blake3` checksums.
  - Emits Turtle (`cpmp-catalog.ttl`), N-Quads (`cpmp-catalog.nq`), and SHACL shapes (`cpmp-shapes.ttl`).
  - If `--with-gates` is enabled, runs the `open-ontologies` admission pipeline and emits markdown reports under `reports/`.
- `cpmp graph validate <file>`: Runs `open-ontologies validate` against the RDF/Turtle file.
- `cpmp graph load <file>`: Runs `open-ontologies load` to load a Turtle file.
- `cpmp graph query <sparql>`: Runs `open-ontologies query` with the SPARQL query string.
- `cpmp graph version <label>`: Snapshot versions the current graph state via `open-ontologies version`.
- `cpmp graph drift <before> <after>`: Computes version diff via `open-ontologies diff`.
- `cpmp graph project --files <file> --out <dir>`: Standalone conversion of raw JSON file inventory to RDF.
- `cpmp policy check --catalog <dir>`: Evaluates RDF/Turtle file against standard policy rules.
- `cpmp policy enforce --catalog <dir>`: Same as check, but exits with code 1 if any policy pack fails.
- `cpmp tenant create <name>`: Writes a json file stub under `~/.cpmp/tenants/<name>.json`.
- `cpmp tenant list`: Lists created tenant files.
- `cpmp audit lineage [--limit <n>]`: Shows the Open Ontologies lineage trail.
- `cpmp receipt emit <path> [--out <dir>]`: Standalone file scan and receipt generation.
- `cpmp receipt verify-no-deletion <before> <after>`: Performs verification between two TOML receipts.
- `cpmp enterprise doctor [--catalog <dir>]`: Diagnoses system state (checks for `open-ontologies` binary, runs catalog policies, and lists enterprise stubs).

### Core Library Modules (in `src/`)
- `src/scanner.rs`:
  - `pub fn scan(paths: &[PathBuf], out_dir: &Path) -> Result<ScanReceipt>`
  - Scans files, ignores binary extensions, extracts symbols, detects capabilities, generates receipt and RDF files.
- `src/capability.rs`:
  - `pub fn detect_capabilities(path: &Path, content: &str, symbols: &[Symbol]) -> Vec<DetectedCapability>`
  - Heuristically identifies capability keywords.
  - `pub fn classify_path(path: &Path, content: &str) -> Classification`
  - Decides if file is `Live`, `Partial`, `TestOnly`, `DocOnly`, or `Ambiguous`.
- `src/symbol.rs`:
  - `pub fn extract_symbols(path: &Path, content: &str) -> Vec<Symbol>`
  - Extracts definitions (`fn`, `struct`, `enum`, `trait`, `class`, `def`) using regex.
- `src/receipt.rs`:
  - `pub fn generate_receipt(root: &Path, entries: &[FileEntry]) -> Result<ScanReceipt>`
  - Deterministically hashes (BLAKE3) sorted path/hash lines.
  - `pub fn verify_no_deletion(before: &ScanReceipt, after: &ScanReceipt) -> NoDeletionReport`
  - Computes file differences between two receipts.
- `src/rdf.rs`:
  - `pub fn build_and_emit(files: &[FileEntry], capabilities: &[DetectedCapability], receipt: &ScanReceipt, out_dir: &Path) -> Result<()>`
  - Uses Oxigraph to build and write Turtle, N-Quads, and SHACL shapes.
- `src/policy.rs`:
  - `pub fn run_policy_checks(catalog_dir: &Path) -> Vec<PolicyCheck>`
  - Performs static string matching checks on the Turtle catalog file.
- `src/projection.rs`:
  - `pub fn emit_all(reports_dir: &Path, catalog_hash: &str) -> Result<()>`
  - Generates markdown files `CAPABILITY_INVENTORY.md`, `PROJECT_ATLAS.md`, and `PATTERN_ATLAS.md` from JSON cache files.
- `src/gates.rs`:
  - `pub fn run_admission_gates(catalog_dir: &Path, receipt: &ScanReceipt) -> Result<String>`
  - Integrates validation, loading, shacl validation, and versioning with `open-ontologies`.

---

## 3. DoD Criteria & Open Ontologies Requirement Cross-Reference

| # | DoD Requirement | Status | Details / Gaps |
|---|-----------------|--------|----------------|
| 1 | Non-Deletion | **Partial** | File verification (`verify-no-deletion`) exists, but the CLI command format is `cpmp receipt verify-no-deletion <before> <after>` rather than the required `capability-map verify-no-deletion --before <receipt> --after <receipt>`. Output is JSON, not the requested text matching `UNCHANGED / ADDED / MODIFIED / MISSING`. |
| 2 | Functional Commands | **Missing** | The required CLI commands (`scan`, `summary`, `capability`, `patterns`, `symbols`, `tests`, `receipts`) under the main CLI binary do not exist. Subcommand structures are mismatched (`cpmp computer discover` instead of `scan`, etc.). |
| 3 | SQLite Storage | **Broken** | SQLite database schema is defined in `src/db.rs` and rusqlite is in `Cargo.toml`. However, compilation is completely broken, and SQLite is not actually called during `scanner::scan` runs. |
| 4 | Capability Taxonomy & Vocab | **Partial** | Taxonomy is fully implemented. However, 18 of the 29 required vocabulary terms (e.g. `Truex`, `Replay`, `Refusal`, `Need9`, `Need257`, `Shard`, `Segment`, `Corpus`, etc.) are missing from the detection logic in `src/capability.rs`. |
| 5 | Documentation | **Missing** | Out of 15 required documentation files (such as `docs/user/QUICKSTART.md`, `docs/dev/ARCHITECTURE.md`, `docs/swarm/FINAL_SWARM_RECEIPT.md`), only `README.md` exists. |
| 6 | Developer Experience (DX) | **Broken** | Compilation is currently broken, and `scripts/smoke.sh` is completely broken (calls non-existent CLI subcommands). |
| 7 | QoL Features | **Partial** | Graceful degradation on missing tools exists, but CLI help messages don't match the expected commands. |
| 8 | Reports | **Partial** | Emits `CAPABILITY_INVENTORY.md`, `PROJECT_ATLAS.md`, and `PATTERN_ATLAS.md`. However, `LEGACY_NAME_MAP.md`, `DORMANT_CODE_REGISTER.md`, `BROKEN_BUT_REAL_REGISTER.md`, `TEST_EVIDENCE_MAP.md`, and `DOC_CLAIM_MAP.md` are missing. |
| 9 | Testing | **Broken** | Integration/unit tests cannot run because compilation fails. Additionally, testing would fail on prefix string checks (see Bug A). |
| 10 | Agent Usability | **Missing** | Since the query commands (`capability`, `patterns`, etc.) do not exist in the compiled CLI, agents cannot query them. |
| 11 | Refusal | **Partial** | Actionable refusals exist for file paths, but database-related checks are absent because SQLite is not integrated. |

---

## 4. Key Bugs Identified

### Bug A: Compilation Failure (Critical)
- **Files**: `src/db.rs`, `src/report.rs`
- **Root Cause**: These files expect an older `ScanReceipt` struct layout (having fields like `scan_run_id`, `roots`, `dir_count`, `hash_algo`, `catalog_version`, `command_run`, `warnings`, `refusals`, `receipt_path`), but the version in `src/models.rs` only has:
  - `id`
  - `timestamp`
  - `schema_version`
  - `root_paths`
  - `file_count`
  - `total_bytes`
  - `root_hash`
  - `entries`
  - `system_info`
- **Also**: They use `CpmpError::Database` which is not a valid enum variant (or associated function) of `anyhow::Error` (since `CpmpError` is just an alias for `anyhow::Error`).

### Bug B: Oxigraph Prefix Serialization (Will Cause Test Failures)
- **Problem**: Oxigraph `Store::dump_graph_to_writer` writes Turtle using full IRIs by default (e.g. `<http://www.w3.org/ns/prov#wasGeneratedBy>`), rather than prefixed names (`prov:wasGeneratedBy`).
- **Impact**: The integration test `test_catalog_ttl_contains_required_vocabulary` asserts that the Turtle output contains `"prov:wasGeneratedBy"`, `"dcat:"`, and `"spdx:"`. Since these prefixes do not exist in the output, the test will fail once compilation is fixed. Additionally, `public-vocabulary-required` check in `src/policy.rs` will fail.

### Bug C: Broken Smoke Script (`scripts/smoke.sh`)
- **Problem**: The smoke script tries to run `cpmp summary`, `cpmp capability find`, `cpmp receipt list`, and `cpmp report emit` which do not exist on the `cpmp` CLI.

---

## 5. Conclusion & Actionable Next Steps
The codebase is currently broken and cannot be built or tested. To fix the project:
1. Refactor `src/db.rs` and `src/report.rs` to match the current `ScanReceipt` struct in `src/models.rs` and fix the `anyhow::Error` wrapping issues.
2. Update prefix handling in Turtle output to resolve test/policy assertion failures.
3. Align CLI subcommands with expectations or update `scripts/smoke.sh` to match.
