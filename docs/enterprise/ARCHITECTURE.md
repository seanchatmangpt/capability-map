# CPMP Enterprise Architecture

## Overview

```
cpmp (surveyor)
  discovers projects, files, capabilities, tests, docs, runtimes
        ↓
ggen enterprise membrane
  normalizes, projects, validates, receipts, redacts, packages
        ↓
Open Ontologies
  RDF/OWL store, SPARQL, SHACL, lint, reason, diff, version, lineage
        ↓
Enterprise Control Plane
  auth, tenants, policy, approvals, audit, retention, backups, exports
        ↓
agents / auditors / dashboards / downstream validators
```

## Invariant

> **Open Ontologies stores and validates the project graph. ggen guarantees the graph is admissible enterprise evidence before it gets there.**

## Layer Responsibilities

| Layer               | Owner           | Responsibility                                                        |
| ------------------- | --------------- | --------------------------------------------------------------------- |
| Project discovery   | `cpmp`          | Read-only scan of projects, files, capabilities, tests, docs          |
| Projection membrane | `ggen`          | Emit valid RDF/Turtle/N-Quads/SHACL/PROV/DCAT/SPDX                    |
| Graph substrate     | Open Ontologies | validate, load, query, lint, reason, version, lineage                 |
| Enterprise shell    | wrapper modules | auth, tenancy, policy, audit, retention, backups, approval gates      |
| Agent surface       | MCP / CLI       | Let coding agents query and act from evidence                         |
| Receipt layer       | ggen + wrapper  | Prove scan → graph → report → action lineage (BLAKE3 root hashes)    |

## Admission Invariants (Hard Gates)

A scan is NOT admitted to the Open Ontologies store unless all of the following pass:

1. RDF Turtle parses cleanly (no syntax errors)
2. No illegal URI syntax (IRI validation at construction time)
3. No private predicate carries public authority
4. No local ID is laundered into `prov:`, `sh:`, `dcat:`, or other standard namespaces
5. Every `spdx:File` entity links to the scan via `prov:wasGeneratedBy`
6. Every file entity has a `spdx:Checksum` with a real `blake3` hash
7. SHACL shapes pass on the submitted graph
8. Open Ontologies successfully loads the graph
9. Open Ontologies versions the graph
10. Receipt is emitted with root hash
11. No-deletion verification passes

## CLI Surface

```bash
cpmp computer discover <paths>  --with-gates
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
