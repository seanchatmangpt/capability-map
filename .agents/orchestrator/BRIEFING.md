# BRIEFING — 2026-05-27T22:25:00Z

## Mission
Lead the swarm to finish building the Code Capability Catalog with Open Ontologies integration.

## 🔒 My Identity
- Archetype: teamwork_preview_orchestrator
- Roles: orchestrator, user_liaison, human_reporter, successor
- Working directory: /Users/sac/capability-map/.agents/orchestrator
- Original parent: main agent
- Original parent conversation ID: 811650d7-10ee-4d63-8fbb-984d7a630c5f

## 🔒 My Workflow
- **Pattern**: Project
- **Scope document**: /Users/sac/capability-map/PROJECT.md
1. **Decompose**: Decompose the capability-map project into distinct milestones for code exploration, database/RDF schema completion, CLI command alignment, and validation checks.
2. **Dispatch & Execute**:
   - **Direct (iteration loop)**: Explorer → Worker → Reviewer → test → gate
   - **Delegate (sub-orchestrator)**: spawn a sub-orchestrator for each milestone
3. **On failure** (in this order):
   - Retry: nudge stuck agent or re-send task
   - Replace: spawn fresh agent with partial progress
   - Skip: proceed without (only if non-critical)
   - Redistribute: split stuck agent's remaining work
   - Redesign: re-partition decomposition
   - Escalate: report to parent (sub-orchestrators only, last resort)
4. **Succession**: at 16 spawns, write handoff.md, spawn successor
- **Work items**:
  1. Explore current codebase [done]
  2. Define and verify milestones in PROJECT.md [done]
  3. Dispatch milestone implementation [in-progress]
- **Current phase**: 2
- **Current focus**: Dispatch milestone implementation

## 🔒 Key Constraints
- Non-deletion law: You are forbidden to remove code or clean up by destruction. Existing code/artifacts must be cataloged read-only.
- Mark unused items DORMANT, do not delete.
- Never reuse a subagent after it has delivered its handoff — always spawn fresh

## Current Parent
- Conversation ID: 811650d7-10ee-4d63-8fbb-984d7a630c5f
- Updated: not yet

## Key Decisions Made
- None

## Team Roster
| Agent | Type | Work Item | Status | Conv ID |
|-------|------|-----------|--------|---------|
| explorer_analysis | teamwork_preview_explorer | Codebase exploration and gap check | completed | 75f1b685-4496-4c41-a754-2d09602ef694 |
| worker_completion | teamwork_preview_worker | Code completion and build fixes | pending | 9dc8dfb1-68bb-40ce-ba65-57a952840a8f |

## Succession Status
- Succession required: no
- Spawn count: 2 / 16
- Pending subagents: [9dc8dfb1-68bb-40ce-ba65-57a952840a8f]
- Predecessor: none
- Successor: not yet spawned

## Active Timers
- Heartbeat cron: 0d82ec5b-9dba-4bcc-9a6d-1a66cca17878/task-23
- Safety timer: none
- On succession: kill all timers before spawning successor
- On context truncation: run `manage_task(Action="list")` — re-create if missing

## Artifact Index
- /Users/sac/capability-map/.agents/orchestrator/original_prompt.md — Original User Request
- /Users/sac/capability-map/.agents/orchestrator/BRIEFING.md — Persistent memory state
- /Users/sac/capability-map/.agents/orchestrator/progress.md — Swarm progress tracking
- /Users/sac/capability-map/.agents/orchestrator/plan.md — Detailed orchestration steps
