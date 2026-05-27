# Enterprise Definition of Done

`cpmp + ggen + Open Ontologies` is enterprise-ready only when the following full pipeline executes without skipping any step:

```
filesystem scan (read-only)
→ ggen public-vocabulary projection (PROV-O / DCAT / SPDX / SKOS / SHACL)
→ RDF parser pass (open-ontologies validate)
→ SHACL pass with sh:ValidationReport artifact
→ PROV lineage closure (every spdx:File has prov:wasGeneratedBy)
→ Checksum closure (every file has spdx:Checksum with BLAKE3)
→ Open Ontologies load
→ Open Ontologies SPARQL query
→ Open Ontologies version snapshot
→ Report emission from validated + versioned graph
→ Receipt emission (BLAKE3 root hash, TOML)
→ No-deletion verification
→ Policy pack evaluation (all packs PASS or explicit REFUSAL artifact)
```

No step may be skipped. No "trust me" layer.

## Final Judgment Categories

A. Enterprise wrapper design complete; implementation stubs and gates exist.
B. Partial wrapper exists; specific gaps remain.
C. Open Ontologies integration exists but enterprise closure is missing.
D. Blocked by missing Open Ontologies binary, API, or project structure.

**Current judgment: A** — Core scanner + RDF projection + admission gates + policy packs + CLI + receipts + no-deletion + enterprise docs all exist. Remaining stubs: tenant namespace scoping, backup/export path, SHACL report output file (gates call `open-ontologies shacl` but don't capture the `sh:ValidationReport` to disk).
