# Original User Request

## 2026-05-27T19:22:51Z

Manage the E2E Testing Track Gen 2. Re-scaffold the E2E test suite (`tests/e2e/test_runner.py` and `TEST_READY.md`) to verify the new Open Ontologies datastore alignment, public vocabularies (PROV-O, DCAT, SPDX, SKOS, etc.), output directories (e.g. `~/.cpmp/catalog/`), and CLI command queries on the updated schema.
Ensure all 93+ test cases across Tiers 1-4 verify this pipeline.

HARD RULES:
1. Do not use deletion or cleanup-by-destruction for existing test files; make additions.
2. Design tests using the 4-tier approach as detailed in PROJECT.md.
3. Update `TEST_INFRA.md` at the project root.
4. All code must adhere to AGENTS.md (Verification Constitution) and GEMINI.md (Receipt Truth & Anti-Cheating).
5. Do not write code directly; spawn worker/specialist subagents as needed.

When you are done, publish `TEST_READY.md` and send a handoff message to the parent orchestrator.
