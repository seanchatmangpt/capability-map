# GALL-CAP: Capability Admissibility & Process Interchangeability Layer

## Mission
Determine whether a software component is:
* decomposable
* substitutable
* externally verifiable
* process-conformant
* replay-safe
* capability-complete
* admissibly manufacturable

## Core Question
> "Can this capability survive adversarial operational substitution?"

## Capability Cell Structure
Each interchangeable part must satisfy this geometry:

| Layer                     | Description               |
| ------------------------- | ------------------------- |
| POWL route                | lawful operational motion |
| OCEL evidence             | object-centric execution  |
| PROV graph                | lineage                   |
| SHACL rules               | admissibility             |
| SPDX identity             | artifact evidence         |
| Replay fixture            | deterministic proof       |
| Sabotage corpus           | refusal proof             |
| Capability classification | SKOS concept              |
| Runtime package           | WASM/AtomVM/Rust          |
| External verifier ring    | hostile replay            |

## The Objective Function Shift
Conventional coding agents optimize for local coherence, abstraction convenience, and token completion.
GALL-CAP optimizes strictly for **operational consequence conservation**.

The same object-centric world can manufacture every legacy projection, but no legacy projection can reconstruct the full world.
