# Project: Code Capability Catalog (`cpmp`)

## Architecture
`cpmp` is a command-line tool that scans a codebase, extracts files, symbols, dependencies, and capabilities, and maps them to:
1. **Primary Graph Store**: Open Ontologies RDF/Turtle file format (validated using `onto_validate`, loaded with `onto_load`, SHACL checked with `onto_shacl`).
2. **Local Cache / Acceleration Store**: SQLite database (`.capability-map/workspace.sqlite` or similar custom location) holding tables for rapid CLI queries.

Data Flow:
`Scanner` -> Files, Symbols, Capabilities, Receipts ->
  ├── `RDF Builder` -> `cpmp-catalog.ttl` & `cpmp-shapes.ttl`
  ├── `SQLite Writer` -> `workspace.sqlite` (11 required tables)
  └── `Report Writer` -> Markdown reports and TOML receipts

## Milestones

| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| 1 | Fix Compilation & Models | Resolve `src/db.rs` and `src/report.rs` errors, matching the `ScanReceipt` and `CpmpError` definitions. | None | PLANNED |
| 2 | Complete SQLite Tables | Expand database migration to create and populate all 11 required tables (`repositories`, `files`, `symbols`, `dependencies`, `tests`, `docs`, `capabilities`, `patterns`, `classifications`, `receipts`, `scan_runs`). Integrate SQLite caching into the scan process. | M1 | PLANNED |
| 3 | Align CLI Commands | Implement all flat and nested CLI commands in `src/main.rs` to satisfy both the Definition of Done and legacy smoke test configurations. | M2 | PLANNED |
| 4 | Resolve RDF Prefixes & Terms | Implement prefix formatting in `src/rdf.rs` to serialize Turtle output using prefixed names, and add missing capability terms to `src/capability.rs`. | M1 | PLANNED |
| 5 | Generate Documentation | Create all 14 required Markdown files under `docs/` covering user guides, developer schemas, and the final swarm receipt. | None | PLANNED |
| 6 | E2E & Smoke Verification | Verify that `scripts/smoke.sh` and cargo tests pass, verifying non-deletion and correct outputs. | M3, M4, M5 | PLANNED |

## Interface Contracts
- **Scanner ↔ Database**: The scanner passes arrays of `FileEntry`, `Symbol`, `DetectedCapability`, and the `ScanReceipt` to `db::insert_...` functions.
- **Scanner ↔ RDF**: `rdf::build_and_emit` takes `&[FileEntry]`, `&[DetectedCapability]`, `&ScanReceipt`, and outputs Turtle/N-Quads files with standardized public predicates (PROV-O, DCAT, DCTERMS, DOAP, SPDX, SKOS).

## Code Layout
- `src/main.rs`: CLI Entry point.
- `src/lib.rs`: Library module declarations.
- `src/scanner.rs`: Filesystem scanning.
- `src/db.rs`: SQLite caching and query functions.
- `src/rdf.rs`: RDF/Turtle graph building and prefix formatting.
- `src/models.rs`: Domain models and serializable structures.
- `src/capability.rs`: Keyword capability matching and taxonomy classification.
- `src/symbol.rs`: Regex symbol extraction.
- `src/receipt.rs`: BLAKE3 receipt hashing and verification.
- `src/report.rs`: Markdown report emission.
- `src/gates.rs`: Open Ontologies integration wrappers.
- `src/error.rs`: Alias to `anyhow::Error` and result type.
