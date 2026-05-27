## 2026-05-27T22:28:58Z

You are the Code Completer (type: teamwork_preview_worker).
Your working directory is `/Users/sac/capability-map/.agents/worker_completion`.
Your mission is to fix the compilation of `/Users/sac/capability-map` and implement all remaining features to satisfy the 17 Definition of Done criteria (DoD) and the Open Ontologies datastore alignments in `ORIGINAL_REQUEST.md`.

MANDATORY INTEGRITY WARNING:
DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A Forensic Auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.

Please carry out these steps:
1. Fix Compilation Errors in `src/db.rs` and `src/report.rs`:
   - Replace obsolete `.map_err(crate::error::CpmpError::Database)?` calls with standard anyhow error propagation (`?` or `.context(...)?`).
   - Fix `insert_receipt` in `src/db.rs` to map `ScanReceipt` fields without referencing non-existent fields.
   - Expand database schema migration in `src/db.rs` to include all 11 required tables: `repositories`, `files`, `symbols`, `dependencies`, `tests`, `docs`, `capabilities`, `patterns`, `classifications`, `receipts`, `scan_runs`.
   - Complete insertion logic in `src/db.rs` to support inserting into these new tables.

2. Populate SQLite Cache on Scan:
   - In `src/scanner.rs` (or `scanner::scan`), once scanning is done and files, symbols, capabilities, and receipt are produced, initialize the SQLite database at `<out_dir>/workspace.sqlite`.
   - Populate all 11 SQLite tables with the scan results so that it acts as a fully populated local query cache. (e.g. insert repos, files, symbols, dependencies from Cargo.toml if any, tests, docs, capabilities, patterns, classifications, receipt, scan_run).

3. Fix RDF Turtle Prefixes (Bug B):
   - In `src/rdf.rs` (or `rdf::build_and_emit`), instead of dumping Turtle directly to the file via oxigraph, dump to a `Vec<u8>`, convert to string, prepend standard `@prefix` lines (prov, dcat, doap, spdx, skos, dcterms, rdf, xsd), and replace full IRIs (e.g. `<http://www.w3.org/ns/prov#wasGeneratedBy>`) with their prefixed forms (e.g. `prov:wasGeneratedBy`).
   - Write the final prefixed Turtle content to the catalog file. This fixes integration tests that check for `"prov:wasGeneratedBy"`, `"dcat:"`, `"spdx:"`.

4. Expand CLI Subcommands in `src/main.rs`:
   - Implement all flat CLI commands at the top level of the clap CLI: `scan`, `summary`, `capability`, `patterns`, `symbols`, `tests`, `receipts`, `verify-no-deletion`.
   - Match the command options exactly as specified in the DoD (e.g. `--db`, `--before`, `--after`).
   - Ensure the `capability` command parses both `capability <name> --db ...` (flat) and `capability find <name> --db ...` (nested/legacy) to satisfy both the DoD and the smoke script.
   - Implement the query and summary output logic (summary count, query results formatted cleanly, empty results explained).
   - Implement `verify-no-deletion` to compare two receipts, classifying files as `UNCHANGED`, `ADDED`, `MODIFIED`, or `MISSING`. Return exit code 2 and list missing files if any files disappear (refusal condition).

5. Update Capability Detection and Taxonomy in `src/capability.rs`:
   - Complete the vocabulary list to contain all 29 terms from DoD 4.
   - Add inline keyword checks to detect `DORMANT`, `BROKEN_BUT_REAL`, `CAPABILITY_SEED`, `LEGACY_NAME` from file content so they are correctly classified.

6. Generate Documentation Files:
   - Write all 14 required Markdown files under `docs/` as specified in DoD 5 (e.g. `docs/user/QUICKSTART.md`, `docs/dev/ARCHITECTURE.md`, `docs/dev/STORAGE_SCHEMA.md`, etc.).

7. Verify and Run Tests:
   - Run `cargo build` and `cargo test` using the `run_command` tool in the root. Verify that compilation succeeds and all tests pass.
   - Test running `scripts/smoke.sh` and make sure it passes.

When done, write a report detailing the changes, and send a message back to the parent orchestrator (id: "0d82ec5b-9dba-4bcc-9a6d-1a66cca17878").
