# cpmp — Computer Project Mapping Protocol

A local, **non-destructive** code capability catalog that lets LLM coding agents discover all projects, capabilities, files, symbols, tests, and docs on your computer — without asking you what exists.

## Architecture

```
cpmp (surveyor) → ggen membrane (RDF projection) → Open Ontologies (graph store)
     ↓                       ↓                              ↓
read-only scan          PROV-O/DCAT/SPDX/SKOS          validate/load/SHACL/version
BLAKE3 receipts         public vocabulary only           agent query surface
no-deletion verified    admission gates enforced         lineage preserved
```

## Install

```bash
cargo install --path .
```

## Quick Start

```bash
# Scan your home directory (read-only)
cpmp computer discover ~/my-projects --out ~/.cpmp

# With Open Ontologies admission gates (validates Turtle, loads graph, runs SHACL, versions)
cpmp computer discover ~/my-projects --out ~/.cpmp --with-gates

# Check policy compliance
cpmp policy check --catalog ~/.cpmp/catalog

# Enterprise diagnostics
cpmp enterprise doctor
```

## CLI Reference

```
cpmp computer discover <paths...> [--out <dir>] [--with-gates]
cpmp graph project     --files <inventory.json> --out <dir>
cpmp graph validate    <file.ttl>
cpmp graph load        <file.ttl>
cpmp graph query       <sparql>
cpmp graph version     <label>
cpmp graph drift       <before.ttl> <after.ttl>
cpmp policy check      --catalog <dir>
cpmp policy enforce    --catalog <dir>
cpmp tenant create     <name>
cpmp tenant list
cpmp audit lineage     [--limit N]
cpmp receipt emit      <path> [--out <dir>]
cpmp receipt verify-no-deletion <before.toml> <after.toml>
cpmp enterprise doctor [--catalog <dir>]
```

## Output

| File | Description |
|------|-------------|
| `catalog/cpmp-catalog.ttl` | Turtle graph (default graph) |
| `catalog/cpmp-catalog.nq` | N-Quads (full dataset) |
| `catalog/cpmp-shapes.ttl` | SHACL validation shapes |
| `receipts/scan-<ts>.receipt.toml` | BLAKE3 root hash receipt |
| `reports/CAPABILITY_INVENTORY.md` | Capabilities by file and classification |
| `reports/PROJECT_ATLAS.md` | Files by language |
| `reports/PATTERN_ATLAS.md` | Symbols by kind |
| `reports/capability_inventory.json` | Machine-readable capabilities |
| `reports/symbol_index.json` | Extracted symbols |
| `reports/file_inventory.json` | All scanned files |

## Hard Laws

1. **Non-deletion**: No scan command writes to scanned directories
2. **Public vocabulary only**: Every RDF predicate in the graph must use PROV-O, DCAT, SPDX, SKOS, DCTerms, or similar W3C/public ontologies
3. **Real hashes**: All checksums are BLAKE3 of real file bytes
4. **Gate admission**: No data enters Open Ontologies without passing validate → load → SHACL → version
5. **Receipt evidence**: Every scan produces an unforgeable BLAKE3 root hash receipt

## Tests

```
cargo test
```

All 8 integration tests cross real boundaries (filesystem, oxigraph, open-ontologies binary).

## Dependencies

- [oxigraph](https://github.com/oxigraph/oxigraph) — in-memory RDF store, IRI validation, Turtle/NQ serialization
- [open-ontologies](https://github.com/fabio-rovai/open-ontologies) — RDF validation, loading, SHACL, versioning (must be installed)
- [blake3](https://github.com/BLAKE3-team/BLAKE3) — cryptographic file hashing
- [clap](https://docs.rs/clap) — CLI framework
- [walkdir](https://docs.rs/walkdir) — filesystem traversal

## Enterprise Docs

See `docs/enterprise/` for:
- `ARCHITECTURE.md` — full system design
- `GAP_CLOSURE_MATRIX.md` — 15-gap closure matrix
- `POLICY_PACKS.md` — policy enforcement rules
- `PUBLIC_VOCABULARY_FIREWALL.md` — namespace enforcement rules
- `GGEN_PROJECTION_MEMBRANE.md` — RDF projection design
- `OPEN_ONTOLOGIES_ADAPTER.md` — Open Ontologies CLI adapter
- `AUDIT_AND_LINEAGE.md` — audit trail and no-deletion verification
- `TENANCY.md` — tenant namespace isolation
- `CONTROL_PLANE.md` — enterprise control plane map
- `AUTHZ.md` — authorization model
- `RETENTION_AND_BACKUP.md` — retention policy
- `ENTERPRISE_DEFINITION_OF_DONE.md` — enterprise readiness criteria
