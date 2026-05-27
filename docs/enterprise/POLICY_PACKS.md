# Policy Packs

## Overview

Policy packs are named enforcement rules that evaluate the catalog output for compliance. Each pack emits one of: `PASS`, `FAIL`, `WARNING`, `REFUSAL`, `NOT_APPLICABLE`.

## Current Packs

| Pack | What It Checks | Failure Consequence |
|------|---------------|---------------------|
| `public-vocabulary-required` | Turtle contains `prov:`, `spdx:`, `dcat:` | FAIL — admission blocked |
| `no-private-predicate-authority` | No `gall:`, `ggen:` predicates in authority position | FAIL — admission blocked |
| `prov-lineage-required` | Turtle contains `prov:wasGeneratedBy` | FAIL — lineage broken |
| `spdx-checksum-required` | Turtle contains `spdx:checksum` | FAIL — integrity unverifiable |
| `shacl-report-required` | `cpmp-shapes.ttl` exists | FAIL — no SHACL validation possible |
| `no-deletion-required` | Scan receipts directory has at least one receipt | WARNING — cannot verify no-deletion |

## Adding a Policy Pack

All packs are implemented in `src/policy.rs` inside `run_policy_checks()`. Add a new `PolicyCheck` to the returned `Vec`. Each check must:

1. Specify the pack name (kebab-case, matches the directory listing in `policy-packs/`)
2. Evaluate a real observable condition
3. Return a typed `GateResult` — never return `Pass` without checking

## CLI Usage

```bash
# Report only
cpmp policy check --catalog ~/.cpmp/catalog

# Hard enforcement (exit 1 on any failure)
cpmp policy enforce --catalog ~/.cpmp/catalog
```

## Refusal Artifacts

When a pack returns `REFUSAL`, the admission pipeline aborts and logs the refusal to stderr with the pack name and reason. Future versions will write a `refusal-<id>.json` artifact to `reports/`.
