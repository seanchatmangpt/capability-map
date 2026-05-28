#!/usr/bin/env bash
# gemini-invoke.sh — cpmp v30.1.1 Gemini actuation
# Routes implementation work through Gemini API with phase-aware model selection
# Phase model routing:
#   Phase 0 (Foundation): gemini-3.1-flash-lite-preview (compilation fixes)
#   Phase 1 (Canon Registry): gemini-3.1-flash-lite-preview (structured data)
#   Phase 2 (Receipts/Checkpoints): gemini-3.1-flash (crypto fields, complex logic)
#   Phase 3 (GALL-CAP): gemini-3.1-pro (10-layer evaluator)
#   Phase 4 (Manifolds): gemini-3.1-flash (graph construction)
# Fallback: gemini-3.1-pro

set -euo pipefail

FLASH_LITE="gemini-3.1-flash-lite-preview"
FLASH="gemini-3.1-flash"
PRO="gemini-3.1-pro"
FALLBACK_MODEL="${CPMP_GEMINI_FALLBACK:-gemini-3.1-pro}"

prompt_file=""
output_file=""
phase=""
model=""
stderr_file=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --prompt-file) prompt_file="$2"; shift 2;;
    --output-file) output_file="$2"; shift 2;;
    --phase) phase="$2"; shift 2;;
    --model) model="$2"; shift 2;;
    --stderr-file) stderr_file="$2"; shift 2;;
    *) echo "gemini-invoke: unknown arg $1" >&2; exit 2;;
  esac
done

[[ -n "$prompt_file" && -n "$output_file" ]] || { echo "gemini-invoke: prompt/output required" >&2; exit 2; }

# Route by phase if model not explicitly provided
if [[ -z "$model" ]]; then
  case "$phase" in
    phase-0|phase-1|phase-4)
      model="$FLASH_LITE";;
    phase-2)
      model="$FLASH";;
    phase-3)
      model="$PRO";;
    *)
      model="$FLASH_LITE";;
  esac
fi

prompt_text=$(cat "$prompt_file")

run_with_model() {
  local m="$1"
  local cmd=(npx -y @google/gemini-cli -p "$prompt_text" --model "$m" --approval-mode yolo)
  if [[ -n "$stderr_file" ]]; then
    mkdir -p "$(dirname "$stderr_file")"
    "${cmd[@]}" >"$output_file" 2>>"$stderr_file"
  else
    "${cmd[@]}" >"$output_file"
  fi
}

if [[ "${CPMP_GEMINI_DRY_RUN:-0}" == "1" ]]; then
  printf '%s\n' "DRY_RUN gemini-invoke phase=$phase model=$model fallback=$FALLBACK_MODEL"
  exit 0
fi

if [[ -n "$stderr_file" ]]; then mkdir -p "$(dirname "$stderr_file")"; : >"$stderr_file"; fi

# Primary attempt; retry once on fallback if fails or empty
if run_with_model "$model" && [[ -s "$output_file" ]]; then
  exit 0
fi

[[ -n "$stderr_file" ]] && printf '\n[gemini-invoke] %s phase failed or empty; retrying on %s\n' "$phase" "$FALLBACK_MODEL" >>"$stderr_file"
run_with_model "$FALLBACK_MODEL"
