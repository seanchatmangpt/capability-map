# Open Ontologies Adapter

## Role

The `cpmp-open-ontologies-adapter` is the boundary module that translates between cpmp's internal scan evidence format and Open Ontologies' CLI/MCP interface.

## Commands Used

| Operation | CLI | Notes |
|-----------|-----|-------|
| Validate Turtle | `open-ontologies validate <file>` | Gate 1 — must pass before load |
| Load graph | `open-ontologies load <file>` | Gate 2 |
| SHACL validation | `open-ontologies shacl <shapes>` | Gate 3 |
| Version snapshot | `open-ontologies version <label>` | Gate 4 |
| SPARQL query | `open-ontologies query <sparql>` | Report generation |
| Lineage trail | `open-ontologies lineage` | Audit surface |
| Drift detection | `open-ontologies diff <before> <after>` | `cpmp graph drift` |
| History | `open-ontologies history` | Rollback point enumeration |
| Rollback | `open-ontologies rollback <snapshot>` | Emergency recovery |

## Admission Protocol

```
scan evidence
  → cpmp::rdf::build_and_emit (in-memory oxigraph → Turtle/NQ/SHACL)
  → cpmp::gates::gate_validate (open-ontologies validate)
  → cpmp::gates::gate_load (open-ontologies load)
  → cpmp::gates::gate_shacl (open-ontologies shacl)
  → cpmp::gates::gate_version (open-ontologies version v-<receipt_id>)
  → catalog_source_hash (BLAKE3 of TTL)
  → report emission from hash-bound data
```

## MCP Compatibility

Open Ontologies exposes 43 MCP tools via `open-ontologies serve`. Future adapter should use MCP directly for tighter integration. Current implementation uses subprocess CLI.

## Error Handling

All gate functions return `anyhow::Result<()>`. Failures produce a structured error including the command that failed and the exit reason. The admission pipeline aborts at the first gate failure and emits a `REFUSAL` artifact.
