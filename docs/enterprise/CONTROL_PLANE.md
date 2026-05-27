# Control Plane

## Overview

The cpmp Enterprise Control Plane wraps Open Ontologies with governance, policy, tenancy, and audit surfaces.

## Components

| Module | Status | Responsibility |
|--------|--------|---------------|
| `cpmp-enterprise-auth` | Stub | OIDC/SAML token validation |
| `cpmp-enterprise-tenancy` | Partial (JSON files) | Tenant namespace management |
| `cpmp-enterprise-policy` | Implemented | Policy pack evaluation |
| `cpmp-enterprise-audit` | Partial (via open-ontologies lineage) | Audit trail and lineage |
| `cpmp-enterprise-retention` | Stub | Receipt and report retention |
| `cpmp-enterprise-backup` | Stub | Graph backup via open-ontologies save |
| `cpmp-enterprise-redaction` | Stub | PII/secret removal before graph publication |
| `cpmp-enterprise-approval` | Stub | Human-in-the-loop approval gates |
| `cpmp-enterprise-observability` | Stub | OTel traces and metrics |
| `cpmp-open-ontologies-adapter` | Implemented | CLI subprocess wrapper |
| `cpmp-ggen-projection` | Implemented (src/rdf.rs) | RDF projection with public vocabulary |
| `cpmp-public-vocabulary-firewall` | Implemented (src/policy.rs) | Policy pack enforcement |

## Admission Flow

```
cpmp computer discover <paths> --with-gates
  ↓
src/scanner.rs::scan()
  ↓ read-only filesystem walk
  ↓ BLAKE3 hash each file
  ↓ extract symbols + capabilities
  ↓
src/rdf.rs::build_and_emit()
  ↓ PROV-O, DCAT, SPDX, SKOS, DCTerms projection
  ↓ cpmp-catalog.ttl + cpmp-catalog.nq + cpmp-shapes.ttl
  ↓
src/gates.rs::run_admission_gates()
  ↓ open-ontologies validate (Gate 1)
  ↓ open-ontologies load (Gate 2)
  ↓ open-ontologies shacl (Gate 3)
  ↓ open-ontologies version v-<id> (Gate 4)
  ↓ catalog_source_hash() → report binding hash
  ↓
src/projection.rs::emit_all()
  ↓ CAPABILITY_INVENTORY.md
  ↓ PROJECT_ATLAS.md
  ↓ PATTERN_ATLAS.md
  (all bound to catalog source hash)
  ↓
Receipt emitted → verify-no-deletion available
```
