# Audit and Lineage

## Lineage Model

Every scan event produces a `prov:Activity` node in the RDF graph. Every scanned file is a `spdx:File` with a `prov:wasGeneratedBy` link to that activity. This creates a complete, queryable lineage chain.

## Evidence Chain

```
prov:Activity (scan event)
  ← prov:wasGeneratedBy ← spdx:File (each scanned file)
  ← prov:wasGeneratedBy ← dcat:Catalog (the catalog itself)
```

## Audit CLI

```bash
# View lineage trail stored in Open Ontologies
cpmp audit lineage
cpmp audit lineage --limit 20
```

Internally calls `open-ontologies lineage` which exposes the Open Ontologies built-in lineage trail.

## Receipts as Audit Evidence

Every scan emits a TOML receipt at `<out>/receipts/scan-<timestamp>.receipt.toml`:

```toml
id = "rcpt_<uuid>"
timestamp = "2024-01-01T00:00:00Z"
schema_version = "1.0.0"
file_count = 42
total_bytes = 123456
root_hash = "<blake3 over sorted path:hash lines>"

[[entries]]
path = "/absolute/path/to/file.rs"
hash = "<blake3>"
size = 1234
```

The `root_hash` is BLAKE3 over sorted `path:hash\n` concatenation — deterministic and unforgeable.

## No-Deletion Audit

```bash
cpmp receipt verify-no-deletion before.receipt.toml after.receipt.toml
```

Reports:
- `deleted_files`: files in `before` missing from `after` → **FAIL**
- `added_files`: new files (informational)
- `modified_files`: files with changed hash (informational)

## Graph Version History

```bash
open-ontologies history     # list named versions
open-ontologies rollback v-<receipt_id>  # restore
```

Every `cpmp computer discover --with-gates` auto-versions the graph as `v-<receipt_id>`.
