## 2026-05-27T18:49:44Z

# MASTER SWARM PROMPT: Build `~/capability-map`

You are a coding-agent swarm working on the local project:
`/Users/sac/capability-map`

Mission:
Build and finish a local, non-destructive **Code Capability Catalog** so LLM coding agents can see all code, capabilities, patterns, tests, docs, binaries, runtime artifacts, and finish gaps across this computer without deleting or modifying existing source assets outside the catalog project.

Working directory: `/Users/sac/capability-map`
Integrity mode: development

---

# 0. Hard Law
- **Non-deletion law**: You are forbidden to remove code or clean up by destruction. Existing code/artifacts must be cataloged read-only.
- **Allowed actions are additive**: wrappers, facades, adapters, compatibility aliases, tests, manifests, docs, catalog projections, receipts.
- Mark unused items `DORMANT`, do not delete.

---

# 1. Core Product
Build `capability-map` (Rust CLI tool preferred).
It must answer:
- Where is a capability implemented?
- Which files are live, partial, dormant, capability seeds?
- Which tests prove a capability?
- Which symbols are related to a capability?
- Which code is broken but real?

---

# 2. Required Architecture
- **2.1 Scanner Layer**: Discovers files, git repos, workspaces, packages, WASM artifacts, tests, docs.
- **2.2 Hash / Receipt Layer**: Cryptographic hashes of all files, receipts before and after to verify no-deletion.
- **2.3 Symbol Layer**: Extract functions, structs, classes, imports/exports using ripgrep or simple regex.
- **2.4 Capability Layer**: Detect named capabilities like Genesis, ggen, Truex, Receipt, Construct8, Pair2, RelationPage, AtomVM, WASM.
- **2.5 Classification Layer**: Taxonomy: LIVE, PARTIAL, CAPABILITY_SEED, LEGACY_NAME, DORMANT, BROKEN_BUT_REAL, DOC_ONLY, TEST_ONLY, AMBIGUOUS.
- **2.6 Query Layer / CLI**:
  - `capability-map scan <paths...> --out <dir>`
  - `capability-map summary --db <workspace.sqlite>`
  - `capability-map capability <name> --db <workspace.sqlite>`
  - `capability-map patterns --db <workspace.sqlite>`
  - `capability-map symbols <query> --db <workspace.sqlite>`
  - `capability-map tests <query> --db <workspace.sqlite>`
  - `capability-map receipts --db <workspace.sqlite>`
  - `capability-map verify-no-deletion --before <receipt> --after <receipt>`
- **2.7 Projection Layer**: Emits `workspace.sqlite`, `CAPABILITY_INVENTORY.md`, `PATTERN_ATLAS.md`, `LEGACY_NAME_MAP.md`, `DORMANT_CODE_REGISTER.md`, `BROKEN_BUT_REAL_REGISTER.md`, `TEST_EVIDENCE_MAP.md`, `DOC_CLAIM_MAP.md`, `NON_DELETION_RECEIPT.toml`.

---

# 3. Required Verification
Provide unit and integration tests to verify scanning, database insertion, classification mapping, receipt generation, and verify-no-deletion logic on a tiny-repo fixture.

---

Execute this plan additive-only, run tests, ensure compilation succeeds.

## 2026-05-27T19:03:31Z

### Definition of Done finish gate:

Please apply this Definition of Done as the official finish gate for the teamwork swarm. Focus on completing all 17 DoD criteria, including the required CLI command formats, SQLite table schemas, classifications taxonomy, complete set of markdown and TOML documentation/reports, and the E2E verification test suite / smoke script.

```md
# DEFINITION OF DONE: capability-map

Project:
`~/capability-map`

Mission:
Build a local, non-destructive Code Capability Catalog so coding agents can inspect, classify, query, and finish existing code capabilities across the computer without deleting code or relying on the human to remember what exists.

This project is DONE only when it improves:
1. Capability visibility
2. Agent execution accuracy
3. Developer experience
4. Documentation quality
5. Quality of life
6. Non-deletion guarantees
7. Verification and receipts

Narrative completion is not accepted.

---

# 1. Non-Deletion Done Criteria
The project is not done unless every scan and catalog command is non-destructive by default.

## Required
- No scan command modifies scanned repositories.
- No command deletes source files.
- No command removes tests, docs, fixtures, scripts, or manifests.
- No command rewrites external repositories.
- Missing files between receipts are reported as evidence, not auto-fixed.
- Every catalog update is additive or explicitly scoped to the catalog output directory.
- `verify-no-deletion` exists and works.

## Required command
```bash
capability-map verify-no-deletion --before <receipt> --after <receipt>
```

## Required output
The command must classify files as:
```text
UNCHANGED
ADDED
MODIFIED
MISSING
```
`MISSING` is a refusal condition.

---

# 2. Functional Done Criteria
v0.1 is done only when these commands exist and run successfully on `fixtures/tiny-repo`.

```bash
capability-map scan fixtures/tiny-repo --out .capability-map
capability-map summary --db .capability-map/workspace.sqlite
capability-map capability Receipt --db .capability-map/workspace.sqlite
capability-map patterns --db .capability-map/workspace.sqlite
capability-map symbols Receipt --db .capability-map/workspace.sqlite
capability-map tests Receipt --db .capability-map/workspace.sqlite
capability-map receipts --db .capability-map/workspace.sqlite
capability-map verify-no-deletion --before <receipt-a> --after <receipt-b>
```

## Required scan capabilities
The scanner must detect:
* files
* directories
* repositories
* package manifests
* language guesses
* tests
* docs
* scripts
* configs
* WASM/runtime artifacts when present
* file hashes
* file sizes
* modification timestamps
* path-relative identity

---

# 3. Storage Done Criteria
The project is done only when the catalog persists to SQLite.

Required database:
```text
.capability-map/workspace.sqlite
```

Required tables:
```text
repositories
files
symbols
dependencies
tests
docs
capabilities
patterns
classifications
receipts
scan_runs
```

## Required integrity check
```bash
sqlite3 .capability-map/workspace.sqlite "PRAGMA integrity_check;"
```
Expected:
```text
ok
```

---

# 4. Capability Classification Done Criteria
Every discovered artifact must be classifiable using this taxonomy:
```text
LIVE
PARTIAL
CAPABILITY_SEED
LEGACY_NAME
DORMANT
BROKEN_BUT_REAL
DOC_ONLY
TEST_ONLY
AMBIGUOUS
```
There is no `DELETE` classification.

## Required initial capability vocabulary
```text
Genesis
ggen
Truex
Receipt
Replay
Refusal
Construct8
Pair2
RelationPage
Need9
Need257
Shard
Segment
Corpus
O*
mu
AtomVM
Erlang
WASM
POWL
OCEL
PROV
SHACL
DCAT
Field8
Instinct8
Doctor
Wizard
Telco
```

## Done means
For each capability query, the tool returns:
* matching files
* matching symbols when available
* line numbers when available
* evidence type
* classification
* confidence
* related tests
* related docs
* finish gap

---

# 5. Docs Done Criteria
Documentation is not optional. Docs are part of the product.

The project is done only when these docs exist:
```text
README.md
docs/user/QUICKSTART.md
docs/user/COMMANDS.md
docs/user/CONFIGURATION.md
docs/user/REPORTS.md
docs/user/NON_DELETION_GUARANTEE.md
docs/user/FAQ.md

docs/dev/ARCHITECTURE.md
docs/dev/STORAGE_SCHEMA.md
docs/dev/SCANNER_PIPELINE.md
docs/dev/CAPABILITY_CLASSIFICATION.md
docs/dev/RECEIPTS.md
docs/dev/TESTING.md
docs/dev/CONTRIBUTING.md

docs/swarm/FINAL_SWARM_RECEIPT.md
```

## README must include
* what the tool does
* what the tool never does
* install/build instructions
* first scan example
* query examples
* report examples
* no-deletion guarantee
* troubleshooting

## QUICKSTART must fit on one screen
Required example:
```bash
capability-map scan ~/knhk ~/truex ~/cell8 ~/wasm4pm --out .capability-map
capability-map capability Construct8 --db .capability-map/workspace.sqlite
capability-map patterns --db .capability-map/workspace.sqlite
```

## COMMANDS must document every CLI command
For each command:
```text
Purpose
Usage
Options
Examples
Outputs
Failure modes
```

---

# 6. DX Done Criteria
Developer experience is done only when a new coding agent can enter the repo and succeed without asking the human.

## Required
* One obvious build command.
* One obvious test command.
* One obvious smoke command.
* Clear error messages.
* Stable CLI names.
* Help text for every command.
* Example fixture repo.
* Example output directory.
* Example reports committed or reproducible.
* No hidden setup steps.

## Required commands
One of these must work, depending on project language:
```bash
cargo build
cargo test
```

## Required smoke script
Provide one:
```bash
scripts/smoke.sh
```

It must:
1. scan `fixtures/tiny-repo`
2. create `.capability-map/workspace.sqlite`
3. emit reports
4. emit receipt
5. run a capability query
6. run no-deletion verification
7. print final pass/fail status

---

# 7. QoL Done Criteria
Quality of life means the tool is pleasant enough that agents will actually use it instead of improvising.

## Required QoL features
* `--help` works for every command.
* Errors include the failed path or missing argument.
* Output paths are printed after every scan.
* Scan summary is concise.
* Long output has table formatting or clear sections.
* Empty results are explained, not silent.
* Missing optional tools degrade gracefully.
* If ctags/tree-sitter/wasm tools are missing, the tool reports `OPTIONAL_TOOL_MISSING` and continues.
* Reports include “next best commands.”
* Reports include “top capabilities found.”
* Reports include “top finish gaps.”
* Reports include “no deletion status.”

## Required command behavior
Good:
```text
ERROR: workspace.sqlite not found at .capability-map/workspace.sqlite

Next:
  capability-map scan <paths...> --out .capability-map
```

---

# 8. Reports Done Criteria
The project is done only when scans emit these reports:
```text
CAPABILITY_INVENTORY.md
PATTERN_ATLAS.md
LEGACY_NAME_MAP.md
DORMANT_CODE_REGISTER.md
BROKEN_BUT_REAL_REGISTER.md
TEST_EVIDENCE_MAP.md
DOC_CLAIM_MAP.md
NON_DELETION_RECEIPT.toml
```

## CAPABILITY_INVENTORY.md must include
| Capability | Files | Symbols | Tests | Docs | Classification | Finish gap |
| ---------- | ----: | ------: | ----: | ---: | -------------- | ---------- |

## PATTERN_ATLAS.md must include
| Pattern | Locations | Evidence types | Repeated names | Likely owner | Finish action |
| ------- | --------: | -------------- | -------------- | ------------ | ------------- |

## DORMANT_CODE_REGISTER.md must include
| Path | Reason dormant | Capability value | Keep reason | Reconnect path |
| ---- | -------------- | ---------------- | ----------- | -------------- |

## BROKEN_BUT_REAL_REGISTER.md must include
| Path | Failure | Evidence of intent | Capability seed | Additive repair |
| ---- | ------- | ------------------ | --------------- | --------------- |

---

# 9. Testing Done Criteria
The test suite must verify the product behavior, not merely compile.

## Required tests
* scans fixture directory
* hashes files
* inserts files into SQLite
* detects at least one capability
* detects at least one symbol-like item
* detects at least one test file
* emits Markdown reports
* emits TOML receipt
* verifies no-deletion between identical receipts
* reports added file between receipts
* reports modified file between receipts
* reports missing file between receipts
* confirms scan does not modify source fixture
* handles empty directory
* handles missing path with clear error
* handles optional tool missing

## Required fixture
```text
fixtures/tiny-repo/
  README.md
  src/
    lib.rs or index.ts
  tests/
    receipt_test.rs or receipt.test.ts
```

The fixture must contain at least:
* `Receipt`
* `Replay`
* `Refusal`
* one test
* one doc claim

---

# 10. Agent Usability Done Criteria
The tool is done only when a coding agent can answer these without browsing the repo manually:
```bash
capability-map capability Construct8 --db .capability-map/workspace.sqlite
capability-map capability Receipt --db .capability-map/workspace.sqlite
capability-map capability WASM --db .capability-map/workspace.sqlite
capability-map patterns --db .capability-map/workspace.sqlite
capability-map tests Receipt --db .capability-map/workspace.sqlite
```

Each answer must include:
* paths
* line numbers if available
* evidence type
* classification
* confidence
* next action

---

# 11. Refusal Done Criteria
The project must explicitly refuse unsupported completion.

A command must return a clear refusal when:
* input path does not exist
* database does not exist
* receipt path does not exist
* scan output directory is not writable
* SQLite integrity check fails
* no files are discovered
* a file disappears between receipts
* optional parser is unavailable
* capability is not found
* report cannot be emitted

Refusal is success when it is accurate and actionable.

---

# 12. Performance Done Criteria
v0.1 performance target:
* fixture scan completes in under 5 seconds
* reports emit in under 5 seconds
* capability query completes in under 2 seconds

v0.2 performance target:
* can scan at least one large repo without crashing
* can resume or rerun without corrupting database
* can handle ignored directories safely
* can avoid indexing `node_modules`, `target`, `.git`, build outputs by default

---

# 13. Packaging Done Criteria
The project must be easy to run.

At minimum:
* clear install/build instructions
* version command
* help command
* smoke script
* example scan

Required:
```bash
capability-map --version
capability-map --help
capability-map scan --help
```

---

# 14. Final Swarm Receipt Done Criteria
The swarm must produce:
```text
docs/swarm/FINAL_SWARM_RECEIPT.md
```

It must include:
```text
summary
files inspected
files changed
code preserved
commands run
command results
tests passing
tests failing
reports emitted
receipts emitted
no-deletion verification
v0.1 status
v0.2 gap list
next 8 additive work packets
```

Final judgment must be one of:
```text
A. v0.1 complete and ready to scan larger repos.
B. v0.1 partially complete; specific gaps remain.
C. Catalog skeleton exists but core scan/storage/report loop is missing.
D. Project is blocked by build/runtime issues.
```

No other final judgment is allowed.

---

# 15. v0.1 Definition of Done
v0.1 is DONE when:
* CLI exists.
* `scan` works on fixture.
* SQLite catalog is emitted.
* files are hashed.
* capabilities are detected.
* symbol-like entries are detected.
* tests are detected.
* docs are detected.
* reports are emitted.
* receipt is emitted.
* no-deletion verification works.
* smoke script passes.
* README and quickstart exist.
* tests pass.
* final swarm receipt exists.

---

# 16. v0.2 Definition of Done
v0.2 is DONE when `capability-map` can scan these roots read-only:
```bash
~/knhk
~/truex
~/cell8
~/wasm4pm
```

and answer:
```bash
capability-map capability Genesis --db .capability-map/workspace.sqlite
capability-map capability ggen --db .capability-map/workspace.sqlite
capability-map capability Truex --db .capability-map/workspace.sqlite
capability-map capability Construct8 --db .capability-map/workspace.sqlite
capability-map capability Receipt --db .capability-map/workspace.sqlite
capability-map capability Replay --db .capability-map/workspace.sqlite
capability-map capability Refusal --db .capability-map/workspace.sqlite
capability-map patterns --db .capability-map/workspace.sqlite
```
without modifying those repositories.

---

# 17. Final Operating Line
`capability-map` is finished when agents no longer ask the human what exists.
They ask the catalog.

No deletion.
No cleanup-by-destruction.
No unsupported completion.
Only inventory, classification, connection, verification, receipt, replay, and refusal.
```

## 2026-05-27T19:11:21Z

### Swarm Guidance Update: Gemini CLI Actuation, OAuth, and MCP Integration

I have inspected the `speckit-ralph` workspace for the exact Gemini CLI actuation, OAuth checking, and MCP integration patterns:

1. **Gemini CLI Invocations (`npx -y @google/gemini-cli`)**:
   - Actuation is designed to run via `npx -y @google/gemini-cli -p "$PROMPT" --model <model> --yolo` (or `--approval-mode yolo`).
   - Check `[gemini-invoke.sh](file:///Users/sac/speckit-ralph/scripts/gemini-invoke.sh)` and `[gemini-do.sh](file:///Users/sac/speckit-ralph/.portfolio/tools/gemini-do.sh)`.
   - Models are dynamically routed: primary `gemini-3.1-flash-lite-preview` for explore tasks and fallback to `gemini-3.1-pro-preview` for exploit tasks or timeouts.

2. **OAuth Health Checks (`oauth-check.sh`)**:
   - Authentication checks are driven by verifying the token expiry timestamp in `~/.gemini/oauth_creds.json` against a time skew. If expired, it triggers `gemini --version` to attempt a refresh or raises a `AGENT_AUTH_EXPIRED` orange alert.
   - Refer to `[oauth-check.sh](file:///Users/sac/speckit-ralph/scripts/oauth-check.sh)`.

3. **MCP Tool Integration (`McpServer` & `McpToolBinding`)**:
   - Exposes a lightweight tool connection that parses JSON-RPC request schemas, projects parameters into a `Field8` using `BoundaryProjector8`, processes them against an `InstinctTable`, and formats consequence updates as standard JSON-RPC responses.
   - Refer to `[mcp_server.rs](file:///Users/sac/speckit-ralph/sos/crates/sos-telco/src/mcp/mcp_server.rs)`.

Use these exact structural patterns for `capability-map`'s scanning, receipts, and verification commands. Keep all CLI options compatible with these boundaries.

## 2026-05-27T19:22:33Z

### HARD ARCHITECTURAL CORRECTION: `cpmp` (capability-map) Open Ontologies Datastore Alignment

The architecture of `cpmp` (capability-map) has been corrected. **The SQLite database is no longer the primary catalog store.** It is demoted to a local cache/acceleration layer. The primary graph catalog store is now **Open Ontologies**.

Please apply this corrected master prompt and plan to the swarm immediately:

```md
# CPMP OPEN ONTOLOGIES DATASTORE CORRECTION

Update `~/capability-map` / `cpmp` architecture.

Hard correction:
`cpmp` must leverage Open Ontologies as the primary catalog datastore.
Do not make SQLite the source store.

SQLite may be used only as:
- local cache
- acceleration layer
- temporary index
- Open Ontologies internal lineage substrate
- optional report cache

The source catalog must be emitted as RDF/Turtle or N-Quads and loaded into Open Ontologies.

## Required pipeline
Implement this flow:
1. Scan local projects (R1/R2)
2. Build project/capability graph using public ontologies (PROV-O, DCAT, SPDX, DOAP, SKOS)
3. Emit `~/.cpmp/catalog/cpmp-catalog.ttl`
4. Emit `~/.cpmp/catalog/cpmp-shapes.ttl`
5. Validate with Open Ontologies (`onto_validate` / `onto_shacl`)
6. Load into Open Ontologies (`onto_load`)
7. Run SHACL checks
8. Run SPARQL queries for reports (`onto_query`)
9. Save/version the catalog (`onto_save` / `onto_version`)
10. Emit receipts (`NON_DELETION_RECEIPT.toml`)

## Required Open Ontologies integration
Support direct CLI invocation of `open-ontologies` or MCP calls to `onto_*` tools.
Required tool sequence where available:
- `onto_validate`
- `onto_load`
- `onto_stats`
- `onto_shacl_check`
- `onto_shacl`
- `onto_reason`
- `onto_lint`
- `onto_query`
- `onto_save`
- `onto_version`
- `onto_drift`
- `onto_lineage`

## Required outputs
- `~/.cpmp/catalog/cpmp-catalog.ttl`
- `~/.cpmp/catalog/cpmp-catalog.nq`
- `~/.cpmp/catalog/cpmp-shapes.ttl`
- `~/.cpmp/reports/CAPABILITY_INVENTORY.md`
- `~/.cpmp/reports/PROJECT_ATLAS.md`
- `~/.cpmp/reports/PATTERN_ATLAS.md`
- `~/.cpmp/receipts/scan-<timestamp>.receipt.toml`

## Required public vocabulary mapping
- PROV-O for scan activities, file evidence, receipt lineage
- DCAT for catalogs and distributions
- DCTERMS for titles, identifiers, dates, relations
- DOAP for projects/repositories
- SPDX for packages, files, checksums, licenses
- SKOS for capabilities and classifications
- SHACL for validation shapes
- RDF/RDFS/OWL for graph structure
Use the `cpmp:` namespace only where no stable public predicate exists.

## Required refusal conditions
Refuse completion if:
- catalog TTL does not parse
- Open Ontologies cannot load the catalog
- SHACL fails without an explicit report
- no scan receipt is emitted
- no Open Ontologies version is emitted
- reports are emitted from stale data
- SQLite is treated as the source store
- private predicates replace public vocabulary equivalents
```

Please update the project plan, tests, CLI command mappings, scanner outputs, and the integration tests to utilize this Ontostar/Open-Ontologies integration flow. All execution paths must remain non-destructive.
