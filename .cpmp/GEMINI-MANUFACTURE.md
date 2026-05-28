# cpmp v30.1.1 — Gemini Manufacture System

This directory contains the configuration and prompts for driving cpmp v30.1.1 implementation through the Gemini API using gemini-3.1-flash-lite-preview tokens.

## Architecture

```
cpmp-manufacture.sh (orchestrator)
  ↓
  ├─ Phase Discovery (portfolio scanning)
  │   cpmp computer discover ~/{100+ projects}
  │   └─ Output: Master receipt chain + capability inventory
  │
  └─ Phases 0-4 (cpmp v30.1.1 implementation)
      └─ scripts/gemini-invoke.sh (model router)
          └─ Gemini API (phase-aware model selection)
              ├─ Phase 0: gemini-3.1-flash-lite-preview (compilation fixes)
              ├─ Phase 1: gemini-3.1-flash-lite-preview (structured data)
              ├─ Phase 2: gemini-3.1-flash (crypto fields)
              ├─ Phase 3: gemini-3.1-pro (10-layer evaluator)
              ├─ Phase 4: gemini-3.1-flash-lite-preview (graph construction)
              └─ Fallback: gemini-3.1-pro
```

## Quick Start

### Portfolio Discovery (prerequisite)

```bash
chmod +x scripts/*.sh
scripts/cpmp-manufacture.sh --phase phase-discovery-portfolio
```

This scans ~/100+ projects and builds a master capability inventory. Output:
- `.cpmp/portfolio-manifest.json` — list of all projects scanned
- `.cpmp/portfolio-receipt-chain.toml` — master receipt with no-deletion verification
- `.cpmp/PORTFOLIO_CAPABILITY_INVENTORY.md` — human-readable inventory
- `.cpmp/capability-index.json` — machine-readable capability index
- `.cpmp/PORTFOLIO_STATUS.md` — summary statistics

This creates the portfolio-wide **Silent Loss Detection** baseline.

### Phase 0 (Foundation)

```bash
scripts/cpmp-manufacture.sh --phase phase-0
```

Runs Foundation phase (compilation fixes). Output goes to `.cpmp/gemini-outputs/`.

Verify with:
```bash
cargo check   # should produce 0 errors
cargo test    # may still fail on integration tests
```

### All Phases (Discovery + cpmp v30.1.1)

```bash
scripts/cpmp-manufacture.sh --all
```

Runs: Discovery → Phase 0 → 1 → 2 → 3 → 4 sequentially.

### cpmp v30.1.1 Only (skip discovery)

```bash
scripts/cpmp-manufacture.sh --all-cpmp
```

Runs Phase 0 → 1 → 2 → 3 → 4 (skips portfolio discovery).

### Dry Run

```bash
CPMP_GEMINI_DRY_RUN=1 scripts/cpmp-manufacture.sh --phase phase-0
```

Shows what would be run without invoking Gemini.

## Model Routing

The system uses phase-aware model selection to minimize token spend:

| Phase | Model | Reason |
|-------|-------|--------|
| Discovery: Portfolio | flash | Portfolio manifest + batched scanning orchestration |
| 0: Foundation | flash-lite-preview | Simple compilation fixes, straightforward code changes |
| 1: Canon Registry | flash-lite-preview | Structured data, static registry definition |
| 2: Receipts/Checkpoints | flash | Crypto hashes, moderate complexity |
| 3: GALL-CAP | pro | Complex 10-layer evaluator, architectural logic |
| 4: Manifolds | flash-lite-preview | Graph construction, deterministic algorithm |

Fallback is always `gemini-3.1-pro` if primary fails or produces empty output.

### Discovery Phase Detail

The **Portfolio Discovery** phase:
1. Uses flash (2K-4K tokens) to orchestrate scanning of 100+ projects
2. Gemini generates the project manifest, batching strategy, and aggregation logic
3. cpmp itself performs the actual scanning (non-Gemini)
4. Gemini merges receipts and generates portfolio-wide reports
5. No-deletion verification chains receipts for continuity preservation

## Prompts

Each phase has a detailed prompt file in `.cpmp/prompts/`:

- `phase-0-foundation.txt` — 9 sub-tasks for compilation
- `phase-1-canon.txt` — Frozen canon registry with 29 terms
- `phase-2-receipt.txt` — 7 crypto hash fields + 5 Gall Checkpoints
- `phase-3-gall-cap.txt` — 10-layer GALL-CAP evaluator
- `phase-4-manifold.txt` — Consequence manifold topology

Each prompt includes:
- Exact file changes required
- Line-by-line modifications
- Success criteria
- Implementation notes

## Outputs

All Gemini responses are written to `.cpmp/gemini-outputs/`:

```
gemini-outputs/
  phase-0-foundation/
    phase-0-output.md     (Gemini response)
    phase-0-stderr.txt    (invocation log)
  phase-1-canon/
    ...
  manufacture.log         (overall progress log)
```

Review these to understand how each phase was implemented, or to troubleshoot if Gemini's code needs adjustment.

## Integration with Plan

The plan file at `/Users/sac/.claude/plans/cpmp-v30-1-1-sharded-steele.md` is the authoritative source for what each phase should accomplish. The prompts are derived from it.

If you need to modify a phase:
1. Update the plan file first
2. Update the corresponding prompt
3. Re-run that phase

## Post-Implementation

After all phases complete:

```bash
cargo test                # 10 tests should pass
cargo run -- computer discover fixtures/tiny-repo --out /tmp/test-cpmp
cargo run -- graph validate /tmp/test-cpmp/catalog/cpmp-catalog.ttl
cargo run -- policy check --catalog /tmp/test-cpmp/catalog
```

Verify all 13 v30.1.1 Definition of Done gates pass (documented in the plan).

## Token Budget

### Discovery Phase
- Portfolio scanning orchestration: flash (~2K-3K tokens)
- Manifest generation + batching strategy: flash-lite-preview (~1K tokens)
- Receipt merging + portfolio reporting: flash (~1K tokens)
- **Discovery total: ~4K-5K tokens**

### cpmp v30.1.1 Implementation
- Phase 0 (compilation): flash-lite-preview (~2K-4K tokens)
- Phase 1 (canon): flash-lite-preview (~1K-2K tokens)
- Phase 2 (receipts): flash (~2K tokens)
- Phase 3 (GALL-CAP): pro (~3K tokens)
- Phase 4 (manifold): flash-lite-preview (~1K-2K tokens)
- **Implementation total: ~12K-17K tokens**

**Grand total: ~16K-22K tokens for discovery + full v30.1.1 implementation.**

This includes:
- Scanning 100+ projects for capabilities (using cpmp's own scanning engine)
- Building a master receipt chain across the portfolio
- Implementing full cpmp v30.1.1 with all 13 DoD gates

The Gemini CLI tool handles all token accounting automatically.

## Environment Variables

```bash
CPMP_GEMINI_DRY_RUN=1        # Simulate without calling Gemini
CPMP_GEMINI_FALLBACK=...     # Override fallback model (default: gemini-3.1-pro)
```

## References

- Plan: `/Users/sac/.claude/plans/cpmp-v30-1-1-sharded-steele.md`
- Orchestrator: `scripts/cpmp-manufacture.sh`
- Router: `scripts/gemini-invoke.sh`
- Config: `.mcp.json`
