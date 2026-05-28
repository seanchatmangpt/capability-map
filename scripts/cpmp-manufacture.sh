#!/usr/bin/env bash
# cpmp-manufacture.sh — Master orchestration for v30.1.1 implementation via Gemini
# Manufactures Phase 0-4 implementation using Gemini API as the coding engine
# Token budget: gemini-3.1-flash-lite-preview for Phases 0,1,4; flash for 2; pro for 3

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
PROMPTS_DIR="$PROJECT_ROOT/.cpmp/prompts"
OUTPUTS_DIR="$PROJECT_ROOT/.cpmp/gemini-outputs"
LOG_FILE="$OUTPUTS_DIR/manufacture.log"

mkdir -p "$PROMPTS_DIR" "$OUTPUTS_DIR"

# Log function
log() {
  local msg="$1"
  echo "[$(date '+%Y-%m-%d %H:%M:%S')] $msg" | tee -a "$LOG_FILE"
}

# Run a Gemini phase
run_phase() {
  local phase="$1"
  local phase_name="$2"
  local prompt_file="$PROMPTS_DIR/$phase.txt"
  local output_file="$OUTPUTS_DIR/$phase-output.md"
  local stderr_file="$OUTPUTS_DIR/$phase-stderr.txt"

  if [[ ! -f "$prompt_file" ]]; then
    log "ERROR: Prompt file not found: $prompt_file"
    return 1
  fi

  log "═══════════════════════════════════════════════════════════"
  log "Starting $phase_name"
  log "═══════════════════════════════════════════════════════════"

  # Invoke gemini-invoke.sh
  "$SCRIPT_DIR/gemini-invoke.sh" \
    --prompt-file "$prompt_file" \
    --output-file "$output_file" \
    --phase "$phase" \
    --stderr-file "$stderr_file"

  log "Completed $phase_name → $output_file"

  if [[ -f "$stderr_file" && -s "$stderr_file" ]]; then
    log "STDERR from phase:"
    cat "$stderr_file" | tee -a "$LOG_FILE"
  fi

  return 0
}

# Parse arguments
DRY_RUN=0
PHASES=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run)
      DRY_RUN=1
      export CPMP_GEMINI_DRY_RUN=1
      log "DRY_RUN mode enabled"
      shift;;
    --phase)
      PHASES+=("$2")
      shift 2;;
    --all)
      PHASES=(phase-discovery-portfolio phase-0 phase-1 phase-2 phase-3 phase-4)
      shift;;
    --all-cpmp)
      PHASES=(phase-0 phase-1 phase-2 phase-3 phase-4)
      shift;;
    *)
      echo "Usage: $0 [--dry-run] [--phase PHASE] [--all] [--all-cpmp]"
      echo ""
      echo "Phases:"
      echo "  phase-discovery-portfolio   Scan ~/100+ projects for capabilities (FIRST)"
      echo "  phase-0                     Foundation (compilation fixes)"
      echo "  phase-1                     Frozen Canon Registry (29 terms)"
      echo "  phase-2                     Receipt Enhancement + Gall Checkpoints"
      echo "  phase-3                     Capability Cells + GALL-CAP Evaluator"
      echo "  phase-4                     Consequence Manifolds"
      echo ""
      echo "Shortcuts:"
      echo "  --all                       Run discovery + all cpmp phases 0-4"
      echo "  --all-cpmp                  Run cpmp phases 0-4 only (skip discovery)"
      exit 1;;
  esac
done

# Default: Phase 0
[[ ${#PHASES[@]} -eq 0 ]] && PHASES=(phase-0)

log "cpmp v30.1.1 Manufacture Starting"
log "Phases to run: ${PHASES[*]}"
log "Plan file: /Users/sac/.claude/plans/cpmp-v30-1-1-sharded-steele.md"

# Check if discovery phase should run first
if [[ " ${PHASES[*]} " =~ " phase-0 " ]] || [[ " ${PHASES[*]} " =~ " phase-1 " ]] || [[ "${PHASES[0]}" == "phase-0" ]]; then
  # If any main phase is being run, offer to run discovery first
  if [[ "$DRY_RUN" == "0" && -z "${SKIP_DISCOVERY:-}" ]]; then
    log "✓ Discovery phase can run first to catalog ~/portfolio"
    log "  Set SKIP_DISCOVERY=1 to skip, or run: scripts/cpmp-manufacture.sh --phase phase-discovery-portfolio"
  fi
fi

# Run phases in order
for phase in "${PHASES[@]}"; do
  case "$phase" in
    phase-discovery-portfolio)
      run_phase "phase-discovery-portfolio" "Phase Discovery: Portfolio Capability Inventory";;
    phase-0)
      run_phase "phase-0-foundation" "Phase 0: Foundation (Compilation)";;
    phase-1)
      run_phase "phase-1-canon" "Phase 1: Frozen Canon Registry";;
    phase-2)
      run_phase "phase-2-receipt" "Phase 2: Receipt Enhancement + Gall Checkpoints";;
    phase-3)
      run_phase "phase-3-gall-cap" "Phase 3: Capability Cells + GALL-CAP";;
    phase-4)
      run_phase "phase-4-manifold" "Phase 4: Consequence Manifolds";;
    *)
      log "ERROR: Unknown phase: $phase"
      exit 1;;
  esac
done

log "═══════════════════════════════════════════════════════════"
log "Manufacture Complete"
log "All outputs: $OUTPUTS_DIR"
log "Verify with: cargo check && cargo test"
log "═══════════════════════════════════════════════════════════"
