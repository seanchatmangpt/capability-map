# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`cpmp` (Computer Project Mapping Protocol) is a local, non-destructive code capability surveyor. It scans a filesystem, extracts files, symbols, capabilities, and dependencies, then outputs:
1. **RDF graphs** (Turtle and N-Quads) using public vocabularies (PROV-O, DCAT, SPDX, SKOS)
2. **SQLite database** for rapid local queries
3. **Markdown reports** and BLAKE3 hash receipts

The project enforces "hard laws": non-deletion, public vocabulary only, real BLAKE3 hashes, and gate-based admission to the RDF graph store.

## Build & Development Commands

```bash
# Build the binary
cargo build

# Build in release mode
cargo build --release

# Run the CLI
cargo run -- computer discover <path> --out <dir>
cargo run -- verify-no-deletion --before <before.toml> --after <after.toml>

# Run all tests (8 integration tests, must hit real boundaries: filesystem, oxigraph, sqlite)
cargo test

# Run a specific test
cargo test test_scan_produces_files_and_receipt -- --nocapture

# Check for compilation errors without building
cargo check

# Install binary to ~/.cargo/bin/
cargo install --path .
```

## Architecture

### Data Flow
```
Scanner → Files, Symbols, Capabilities, Receipts →
  ├── RDF Builder (rdf.rs) → cpmp-catalog.ttl, cpmp-shapes.ttl, cpmp-catalog.nq
  ├── SQLite Writer (db.rs) → workspace.sqlite (11 required tables)
  └── Report Writer (report.rs) → Markdown reports, TOML receipts
```

### Core Modules

| Module | Purpose |
|--------|---------|
| `scanner.rs` | Filesystem traversal using `walkdir`, entry point for the entire pipeline |
| `symbol.rs` | Regex-based symbol extraction (functions, classes, etc.) |
| `capability.rs` | Keyword-based capability detection and taxonomy classification |
| `models.rs` | Serializable domain models: `FileEntry`, `Symbol`, `DetectedCapability`, `ScanReceipt` |
| `receipt.rs` | BLAKE3 hashing and no-deletion verification |
| `rdf.rs` | RDF graph building with prefix formatting, Turtle/N-Quads serialization via `oxigraph` |
| `db.rs` | SQLite caching: 11 tables (repositories, files, symbols, dependencies, tests, docs, capabilities, patterns, classifications, receipts, scan_runs) |
| `projection.rs` | Projection logic (data transformation layer) |
| `gates.rs` | Open Ontologies integration (validate, load, SHACL, version commands) |
| `policy.rs` | Policy compliance checking |
| `report.rs` | Markdown report generation |
| `cmds/` | CLI command handlers (computer, graph, policy, receipt, audit, enterprise, capability, report, summary) |

### Key Constraints & Patterns

1. **Non-deletion**: Scanner is read-only; no writes to scanned directories
2. **Public vocabulary only**: All RDF predicates must use PROV-O, DCAT, SPDX, SKOS, DCTerms, DOAP, or W3C standards
3. **Real hashes**: All BLAKE3 checksums are computed from actual file bytes
4. **Interface contracts**:
   - Scanner → Database: passes arrays of `FileEntry`, `Symbol`, `DetectedCapability`, `ScanReceipt` to `db::insert_*` functions
   - Scanner → RDF: `rdf::build_and_emit` takes file entries, capabilities, receipt, outputs Turtle/N-Quads with standardized predicates
5. **Output structure**: `~/.cpmp/` contains `catalog/`, `receipts/`, `reports/` subdirectories

### Dependency Notes

- **oxigraph** (0.4): In-memory RDF store; handles Turtle/N-Quads serialization, IRI validation
- **open-ontologies**: External CLI tool (must be installed separately); validates graphs, runs SHACL, versions datasets
- **blake3**: Cryptographic file hashing for receipts and no-deletion verification
- **rusqlite**: SQLite with bundled library; all schema must support the 11 required tables
- **walkdir** / **ignore**: Filesystem traversal; respects `.gitignore` patterns

## Testing Strategy

Tests are integration tests that cross real boundaries:
1. **Filesystem scanning** → real files via `fixtures/tiny-repo/`
2. **RDF graph generation** → oxigraph serialization
3. **SQLite operations** → real database writes
4. **Receipts** → actual BLAKE3 hashes

Use `tempfile::TempDir` for isolated test output directories. Tests verify:
- File count and discovery
- Receipt generation and structure
- Turtle/N-Quads catalog content
- Symbol and capability extraction
- No-deletion verification

## Key Files to Know

| File | Role |
|------|------|
| `src/main.rs` | CLI dispatcher; routes commands to subcommand handlers |
| `src/scanner.rs` | Entry point; orchestrates scan, receipt generation, RDF/DB writing |
| `src/models.rs` | All serializable types; shared across modules |
| `src/rdf.rs` | Complex RDF building logic; manages prefix formatting for Turtle output |
| `fixtures/tiny-repo/` | Test fixture with real code samples for integration tests |
| `README.md` | User-facing documentation; includes CLI reference and output formats |
| `PROJECT.md` | Milestones and scope (6 major milestones: Fix Compilation, Complete SQLite, Align CLI, RDF Prefixes, Generate Docs, E2E Verification) |

## Common Workflows

**Add a new capability keyword**:
1. Edit `src/capability.rs` (keyword matching logic)
2. Update models if needed (`src/models.rs`)
3. Add test case to integration tests

**Add a new CLI command**:
1. Create handler in `src/cmds/<command>.rs`
2. Wire it in `src/cmds/mod.rs`
3. Add to CLI enum in `src/main.rs`
4. Test with `cargo run -- <command>`

**Debug RDF output**:
1. Run scan: `cargo run -- computer discover . --out test-out`
2. Inspect `test-out/catalog/cpmp-catalog.ttl`
3. Check prefixes in `rdf.rs` if terms are missing or malformed

**Verify no-deletion**:
1. Generate baseline: `cargo run -- receipt emit <path>`
2. Modify or delete files
3. Generate new receipt
4. Verify: `cargo run -- verify-no-deletion --before baseline.toml --after current.toml`

## Performance Considerations

- **Large directories**: `walkdir` is lazy-iterating; performance is linear in file count
- **RDF serialization**: oxigraph 0.4 can handle graphs with thousands of triples but consider memory for very large codebases
- **SQLite**: 11 tables with indexed foreign keys; queries should use indexes on file_id, symbol_id
- **Hashing**: BLAKE3 is parallelized; filesystem I/O is the bottleneck, not crypto

## Entry Points for New Features

- **Graph queries**: Extend `src/cmds/graph.rs` (currently has validate, load, query, version, drift)
- **Policy enforcement**: Extend `src/policy.rs` (currently has check logic)
- **Audit trails**: Extend `src/cmds/receipt.rs` or add new audit module
- **Enterprise features**: Extend `src/cmds/enterprise.rs` (diagnostic surface)
