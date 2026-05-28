# cpmp v30.1.1 Manufacture & Portfolio Discovery

**One command to build cpmp v30.1.1 AND scan your entire ~/portfolio:**

```bash
./scripts/cpmp-manufacture.sh --all
```

This runs:
1. **Phase Discovery** — Scans ~/100+ projects, builds capability inventory, creates master receipt chain
2. **Phase 0** — Fixes compilation errors (Foundation)
3. **Phase 1** — Implements Frozen Canon Registry (29 terms with canon_basis[])
4. **Phase 2** — Adds Receipt encryption + Gall Checkpoints GL-0..GL-4
5. **Phase 3** — Builds Capability Cells + 10-layer GALL-CAP evaluator
6. **Phase 4** — Creates Consequence Manifolds (capability topology)

All phases use Gemini API with **gemini-3.1-flash-lite-preview** for token efficiency (~20K tokens total).

## Outputs

```
.cpmp/
  gemini-outputs/           (Gemini's code responses for each phase)
  portfolio-manifest.json   (list of all projects scanned)
  portfolio-receipt-chain.toml   (no-deletion verification)
  PORTFOLIO_CAPABILITY_INVENTORY.md   (human-readable)
  capability-index.json     (machine-readable)
  PORTFOLIO_STATUS.md       (summary stats)
  manufacturing.log         (progress log)

src/                        (cpmp v30.1.1 implemented by Gemini)
  canon.rs                  (Frozen Canon Registry)
  gall_cap.rs              (GALL-CAP evaluator)
  manifold.rs              (Consequence Manifolds)
  ... (all other phases)
```

## Verification

After manufacture completes:

```bash
# cpmp compiles and tests pass
cargo check
cargo test

# Run the full pipeline on your own projects
cargo run -- computer discover ~ --out /tmp/my-portfolio

# Check policy compliance across portfolio
cargo run -- policy check --catalog /tmp/my-portfolio/catalog

# View capability inventory
cat /tmp/my-portfolio/PORTFOLIO_CAPABILITY_INVENTORY.md
```

## Phases Individually

```bash
# Discovery only (scan portfolio)
./scripts/cpmp-manufacture.sh --phase phase-discovery-portfolio

# cpmp v30.1.1 only (skip discovery)
./scripts/cpmp-manufacture.sh --all-cpmp

# Single phase
./scripts/cpmp-manufacture.sh --phase phase-0
./scripts/cpmp-manufacture.sh --phase phase-1
# etc.

# Dry run (no Gemini calls, show what would run)
CPMP_GEMINI_DRY_RUN=1 ./scripts/cpmp-manufacture.sh --all
```

## Documentation

Full details in:
- `.cpmp/GEMINI-MANUFACTURE.md` — Architecture, prompts, token budget
- `/Users/sac/.claude/plans/cpmp-v30-1-1-sharded-steele.md` — Implementation plan (13 DoD gates)

## Key Concepts

**Silent Loss Class:** A capability exists in your portfolio but is undiscoverable.
- Discovery phase detects this by cataloging all projects
- cpmp v30.1.1 implements the detection and receipt infrastructure

**Frozen Canon Registry:** The First Law — no capability executes without canon_basis[].
- 29 sacred terms with doctrinal basis
- Immutable across all scans

**Consequence Manifolds:** The topology of capability co-occurrence.
- How capabilities relate to each other across projects
- Defers to future work: call graphs, dependency trees

**Gall's Law:** Working seed before scale.
- Phase 0 must compile before Phase 1-4 can run
- Portfolio discovery establishes the baseline before refinement

## Timeline

- **Discovery phase:** 2-3 minutes (batches 100+ projects, creates receipt chain)
- **Phase 0:** 1-2 minutes (fixes compilation, runs Gemini)
- **Phases 1-4:** 5-10 minutes total (canon, receipts, gall-cap, manifolds)
- **Total:** ~10-15 minutes for complete manufacture

Afterward: `cargo test` should pass all 10 tests (~30 seconds).

---

**Begin manufacture:**
```bash
./scripts/cpmp-manufacture.sh --all
```
