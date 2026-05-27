# GAP CLOSURE MATRIX

| Gap | Evidence | Risk | ggen responsibility | Open Ontologies responsibility | Enterprise wrapper responsibility | Test | Status |
|-----|----------|------|---------------------|-------------------------------|----------------------------------|------|--------|
| Invalid Turtle / syntax errors | RDF parser error | Corrupt store state | Serialize using oxigraph in-memory (never hand-write Turtle) | `open-ontologies validate` gate before load | Reject admission if validation fails | `test_catalog_ttl_validates_with_open_ontologies` | ✅ |
| Illegal URI syntax | IRI construction failure | Invalid linked data | `NamedNode::new()` at construction time — oxigraph enforces validity | N/A (gate upstream) | Reject on invalid IRI | Covered by oxigraph panic boundary | ✅ |
| Namespace laundering (`prov:` as private) | Private term prefixed with `prov:` | Misleading authority claims | Only use standard predicates from PROV-O/DCAT/SPDX/SKOS/DCTerms | Lint on load | Policy pack `no-private-predicate-authority` | `test_catalog_ttl_contains_required_vocabulary` | ✅ |
| Private predicate carrying public authority | `gall:`, `ggen:status` in authority position | Unverifiable claims | Use `urn:cpmp:*` for private instances only | Lint | Policy gate scan | `test_catalog_ttl_contains_required_vocabulary` | ✅ |
| Missing SHACL `sh:ValidationReport` | No SHACL output file | Unverified graph | Emit `cpmp-shapes.ttl` with `sh:NodeShape` for all entity classes | `open-ontologies shacl` runs shapes | Gate: shapes must exist before admission | `test_scan_produces_files_and_receipt` | ✅ |
| Missing `prov:wasGeneratedBy` | No lineage link on file entities | Unverifiable provenance | Every `spdx:File` gets `prov:wasGeneratedBy <scan_node>` | Validate lineage on load | Policy pack `prov-lineage-required` | `test_catalog_ttl_contains_required_vocabulary` | ✅ |
| Missing checksum evidence | `spdx:Checksum` absent | Cannot verify file integrity | Every file entry gets BLAKE3 hash as `spdx:Checksum` | Lint | Policy pack `spdx-checksum-required` | `test_catalog_ttl_contains_required_vocabulary` | ✅ |
| Missing canonical graph hash | Reports not bound to source | Stale report risk | Compute BLAKE3 hash of `cpmp-catalog.ttl` after emit | N/A | Bind all reports to source hash | `test_policy_checks_pass_after_valid_scan` | ✅ |
| JSON-only process evidence | No RDF triples | Non-queryable evidence | Always emit RDF alongside JSON inventories | Load RDF into store | Reject JSON-only scans | `test_scan_produces_files_and_receipt` | ✅ |
| Stale report emission | Report issued before latest scan | Misleading state | Source hash in report header links to specific graph version | Diff on old vs new version | Alert if report hash ≠ current catalog hash | Partial — hash logged | 🔶 |
| Unversioned graph mutation | No version snapshot | Cannot rollback | Call `open-ontologies version v-<receipt_id>` after load | `open-ontologies version` | Gate: refuse reports if version fails | Via `--with-gates` flag | 🔶 |
| Unscoped tenant access | No tenant namespace | Cross-tenant leakage | Namespace scan results under `urn:cpmp:tenant:<name>:*` | N/A | `cpmp tenant create` + namespace scoping | `cpmp tenant create/list` | 🔶 |
| Missing no-deletion check | No before/after comparison | Silent file deletion | Receipt before and after — BLAKE3 root hash | N/A | `cpmp receipt verify-no-deletion` | `test_no_deletion_fail_when_file_removed` | ✅ |
| Missing backup/export path | No graph backup | Single point of failure | N/A | `open-ontologies save` | Module stub `cpmp-enterprise-backup` | Stub documented | 🔶 |
| Missing policy refusal artifact | Silent failures | Invisible rejections | N/A | N/A | Policy packs emit `REFUSAL` with reason | `test_policy_checks_pass_after_valid_scan` | ✅ |

**Legend**: ✅ Implemented & tested  🔶 Stubbed / partial  ❌ Not started
