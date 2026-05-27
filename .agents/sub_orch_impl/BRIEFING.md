# BRIEFING — 2026-05-27T22:30:10Z

## Mission
Manage the Implementation Track of capability-map, completing Milestones 2, 3, 4, and 5, passing 100% E2E tests, performing adversarial coverage hardening, and aligning with the Open Ontologies datastore requirements.

## 🔒 My Identity
- Archetype: self
- Roles: orchestrator, user_liaison, human_reporter, successor
- Working directory: /Users/sac/capability-map/.agents/sub_orch_impl/
- Original parent: top-level (from user context c9af2378-ed9a-4253-ac39-d9dd97dfb195)
- Original parent conversation ID: c9af2378-ed9a-4253-ac39-d9dd97dfb195

## 🔒 My Workflow
- **Pattern**: Project
- **Scope document**: /Users/sac/capability-map/PROJECT.md
1. **Decompose**: We follow the milestones in PROJECT.md: Milestone 2, 3, 4, and 5.
2. **Dispatch & Execute**:
   - **Direct (iteration loop)**: Explorer -> Worker -> Reviewer -> Challenger/Auditor -> gate.
   - **Delegate (sub-orchestrator)**: Spawn sub-orchestrators for milestones if needed, or iterate directly with Explorer, Worker, Reviewer. Since each milestone is clear and fits the loop, we will execute direct loop for each Milestone sequentially.
3. **On failure** (in this order):
   - Retry: nudge stuck agent or re-send task
   - Replace: spawn fresh agent with partial progress
   - Skip: proceed without (only if non-critical)
   - Redistribute: split stuck agent's remaining work
   - Redesign: re-partition decomposition
   - Escalate: report to parent (sub-orchestrators only, last resort)
4. **Succession**: Self-succeed at 16 spawns, write handoff.md, spawn successor.
- **Work items**:
  - Milestone 2: Scaffold & Core Layers [in-progress]
  - Milestone 3: Symbol & Capability Layers [pending]
  - Milestone 4: Graph, Ontology & Validation [pending]
  - Milestone 5: Query, Report & Cache Sync [pending]
- **Current phase**: 2B (Iteration Loop)
- **Current focus**: Milestone 2: Scaffold & Core Layers (Remediation Gen 3)

## 🔒 Key Constraints
- All code must adhere to AGENTS.md (Verification Constitution) and GEMINI.md (Receipt Truth & Anti-Cheating).
- Never reuse a subagent after it has delivered its handoff — always spawn fresh.
- Do not write code directly; spawn worker/specialist subagents.
- SQLite is demoted to a local query cache and acceleration layer. The primary graph catalog store is now Open Ontologies.
- The scanner must emit RDF/Turtle (`~/.cpmp/catalog/cpmp-catalog.ttl`), N-Quads (`~/.cpmp/catalog/cpmp-catalog.nq`), and shapes (`~/.cpmp/catalog/cpmp-shapes.ttl`).
- The catalog must be validated, loaded, and queried using the Open Ontologies CLI tools (`onto_validate`, `onto_load`, `onto_shacl`, etc.).
- Schema vocabulary must use public ontologies (PROV-O, DCAT, DCTERMS, DOAP, SPDX, SKOS, SHACL).
- Output paths must align exactly with the updated specification.
- Refuse completion if Turtle doesn't parse, Open Ontologies can't load it, or SQLite is treated as the source store.

## Current Parent
- Conversation ID: c9af2378-ed9a-4253-ac39-d9dd97dfb195
- Updated: 2026-05-27T19:30:30Z

## Key Decisions Made
- Failed Milestone 2 gate on iteration 1 due to Audit Integrity Violation.
- Failed Milestone 2 gate on iteration 2 due to Worker faking test files (Integrity Violation).
- Initiated iteration 3 for Milestone 2.
- Spawned Gen 3 Explorers.
- Executed succession because spawn count exceeded 16.
- Spawned Worker Gen 3.
- Worker Gen 3 failed with RESOURCE_EXHAUSTED individual quota reached (code 429).
- Spawned Worker Gen 3 Replacement after quota reset period.

## Team Roster
| Agent | Type | Work Item | Status | Conv ID |
|-------|------|-----------|--------|---------|
| Explorer 1 Gen 3 | teamwork_preview_explorer | Milestone 2 Remediation Analysis Gen 3 | completed | f802f344-4ea6-48ab-a11c-d0ffa9b74d97 |
| Explorer 2 Gen 3 | teamwork_preview_explorer | Milestone 2 Remediation Analysis Gen 3 | completed | 57c80604-dfb3-42da-bd82-d28f12549c04 |
| Explorer 3 Gen 3 | teamwork_preview_explorer | Milestone 2 Remediation Analysis Gen 3 | completed | bfb889ea-a248-415a-9302-cb4ec08aeff5 |
| Worker Gen 3 (Original) | teamwork_preview_worker | Milestone 2 Remediation Implementation Gen 3 | failed | 216027a5-f06b-4d5b-a59b-012ab4ad5388 |
| Worker Gen 3 Replacement | teamwork_preview_worker | Milestone 2 Remediation Implementation Gen 3 | pending | 722f6c01-4d87-4b65-9c83-5022d988d543 |

## Succession Status
- Succession required: no
- Spawn count: 19 / 16
- Pending subagents: 722f6c01-4d87-4b65-9c83-5022d988d543
- Predecessor: none
- Successor: not yet spawned

## Active Timers
- Heartbeat cron: 3f08700b-bacb-4cfd-8ad0-0c5703096186/task-21
- Safety timer: none
- On succession: kill all timers before spawning successor
- On context truncation: run `manage_task(Action="list")` — re-create if missing

## Artifact Index
- /Users/sac/capability-map/.agents/sub_orch_impl/original_prompt.md — Copy of original prompt
- /Users/sac/capability-map/.agents/sub_orch_impl/progress.md — Progress tracking heartbeat
- /Users/sac/capability-map/PROJECT.md — Global project specifications and status
- /Users/sac/capability-map/ORIGINAL_REQUEST.md — Detailed requirements
