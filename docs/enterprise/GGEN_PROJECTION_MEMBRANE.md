# ggen Projection Membrane

## Role

The `ggen` enterprise projection membrane sits between raw `cpmp` scan output and the Open Ontologies graph store. It transforms discovered filesystem facts into valid, public-vocabulary RDF that can be admitted.

## Invariant

> **No graph enters Open Ontologies until ggen proves it is valid public evidence.**

## Transformation Pipeline

```
FileEntry[] + DetectedCapability[] + ScanReceipt
  → rdf::build_and_emit()
    - Catalog node (dcat:Catalog)
    - Scan activity (prov:Activity with prov:startedAtTime)
    - File nodes (spdx:File with prov:wasGeneratedBy + spdx:checksum)
    - Checksum nodes (spdx:Checksum with spdx:checksumValue BLAKE3)
    - Capability nodes (skos:Concept with skos:prefLabel + skos:notation)
    - Project node (doap:Project)
  → cpmp-catalog.ttl (Turtle — default graph)
  → cpmp-catalog.nq (N-Quads — full dataset)
  → cpmp-shapes.ttl (SHACL shapes)
```

## Public Vocabulary Mapping

| Fact | Source | Projected As |
|------|--------|-------------|
| File path | `FileEntry.path` | `spdx:File` with `file://` IRI |
| File hash | `FileEntry.hash` | `spdx:Checksum` with `spdx:checksumValue` (BLAKE3) |
| File size | `FileEntry.size` | `spdx:fileSize xsd:integer` |
| Language | `FileEntry.language` | `dcterms:format` |
| Scan timestamp | `ScanReceipt.timestamp` | `prov:startedAtTime xsd:dateTime` |
| Capability name | `DetectedCapability.capability` | `skos:prefLabel` on `skos:Concept` |
| Classification | `DetectedCapability.classification` | `skos:notation` |
| File implements capability | Relation | `urn:cpmp:implementsCapability` (private instance predicate) |

## Guarantees

- All IRI construction uses `NamedNode::new()` — invalid IRIs panic at construction, not silently corrupt the store
- BLAKE3 hashes are computed over the actual file bytes — no placeholder values
- PROV lineage is wired automatically — every file links back to its generating scan activity
- Source graph hash is computed from the final Turtle bytes and returned for report binding
