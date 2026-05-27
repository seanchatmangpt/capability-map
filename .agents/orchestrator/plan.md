# Project Plan: Code Capability Catalog (`cpmp`) Completion

## Objective
Lead the swarm to finish building the Code Capability Catalog. Complete all 17 Definition of Done criteria, ensuring Open Ontologies is the primary datastore (RDF/Turtle/N-Quads outputs, validation via `onto_validate`, loading via `onto_load`, SHACL checks, SPARQL report queries, etc.) and SQLite is used strictly as a local acceleration cache.

## Execution Strategy
We will use a Project Orchestrator pattern.
1. **Phase 1: Exploration and Gap Analysis**
   - Spawn an Explorer agent to review the existing code, dependencies, command-line arguments, open-ontologies integration, and tests.
   - Run a test verification command (via Explorer/Worker) to see what passes, fails, or is missing.
2. **Phase 2: Milestone Decomposition and PROJECT.md Scoping**
   - Define 3-7 concrete, sequentially executable milestones based on the exploration findings.
   - Create `PROJECT.md` at the project root documenting the architecture, milestones, and interface contracts.
3. **Phase 3: Implementation & Validation Tracks (Parallel)**
   - **Track A: E2E Testing Track**: Build and document the E2E test suite (Tiers 1-4) on `fixtures/tiny-repo` and publish `TEST_READY.md`.
   - **Track B: Implementation Track**:
     - *Milestone 1*: Align CLI command vocabulary and schema mapping (ensure SQLite cache holds all required tables: repositories, files, symbols, dependencies, tests, docs, capabilities, patterns, classifications, receipts, scan_runs).
     - *Milestone 2*: Complete Open Ontologies RDF/Turtle generation, validation (`onto_validate` / `onto_shacl`), loading (`onto_load`), querying (`onto_query`), and receipts (`NON_DELETION_RECEIPT.toml`).
     - *Milestone 3*: Implement all required markdown and TOML report generation.
     - *Milestone 4*: Implement E2E validation script (`scripts/smoke.sh`) and fix all remaining gaps.
4. **Phase 4: Adversarial Hardening and Forensic Audit**
   - Perform White-Box Adversarial Hardening (Tier 5).
   - Execute Forensic Audit validation.
5. **Phase 5: Success & Handoff**
   - Generate `docs/swarm/FINAL_SWARM_RECEIPT.md`.
   - Present final success report.

## Iteration Status
- Current iteration: 0 / 32
