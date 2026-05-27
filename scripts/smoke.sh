#!/usr/bin/env bash
# scripts/smoke.sh — cpmp v0.1 smoke test
# Validates: scan, receipt, RDF catalog, no-deletion verify, enterprise doctor.
# Exit 0 = all checks passed.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BINARY="$REPO_ROOT/target/debug/cpmp"
FIXTURE="$REPO_ROOT/fixtures/tiny-repo"
OUT="$REPO_ROOT/.cpmp-smoke"
OUT2="$REPO_ROOT/.cpmp-smoke2"

echo "==============================="
echo " cpmp smoke test"
echo "==============================="
echo " binary:  $BINARY"
echo " fixture: $FIXTURE"
echo " out:     $OUT"
echo ""

# [1] Build if needed
if [ ! -f "$BINARY" ]; then
  echo "[1/7] Building cpmp..."
  cargo build --manifest-path "$REPO_ROOT/Cargo.toml" 2>&1
else
  echo "[1/7] Binary found."
fi

# Clean
rm -rf "$OUT" "$OUT2"

# [2] Scan fixture
echo ""
echo "[2/7] Scanning fixture..."
"$BINARY" computer discover "$FIXTURE" --out "$OUT"

# [3] Core outputs exist
echo ""
echo "[3/7] Checking scan outputs..."
FAIL=0

check() {
  local path="$1" label="$2"
  if [ -e "$path" ]; then
    echo "  OK: $label"
  else
    echo "  FAIL: $label ($path)"
    FAIL=1
  fi
}

check "$OUT/receipts"                              "receipts directory"
check "$OUT/reports/capability_inventory.json"    "capability_inventory.json"
check "$OUT/reports/symbol_index.json"            "symbol_index.json"
check "$OUT/reports/file_inventory.json"          "file_inventory.json"
check "$OUT/catalog/cpmp-catalog.ttl"             "cpmp-catalog.ttl (Turtle)"
check "$OUT/catalog/cpmp-catalog.nq"              "cpmp-catalog.nq (N-Quads)"
check "$OUT/catalog/cpmp-shapes.ttl"              "cpmp-shapes.ttl (SHACL)"

if [ "$FAIL" -ne 0 ]; then echo ""; echo "FAIL: Missing expected outputs"; exit 1; fi

# [4] Check capabilities were detected
echo ""
echo "[4/7] Checking capabilities detected..."
CAP_COUNT=$(python3 -c "import json; d=json.load(open('$OUT/reports/capability_inventory.json')); print(len(d))" 2>/dev/null || echo "0")
if [ "$CAP_COUNT" -eq "0" ]; then
  echo "FAIL: No capabilities detected in fixture"
  exit 1
fi
echo "  OK: $CAP_COUNT capability hits in fixture"

# Verify at least one Receipt and one Refusal hit
HAS_RECEIPT=$(python3 -c "import json; d=json.load(open('$OUT/reports/capability_inventory.json')); print(any(c['capability']=='Receipt' for c in d))" 2>/dev/null || echo "False")
HAS_REFUSAL=$(python3 -c "import json; d=json.load(open('$OUT/reports/capability_inventory.json')); print(any(c['capability']=='Refusal' for c in d))" 2>/dev/null || echo "False")
echo "  Receipt detected: $HAS_RECEIPT"
echo "  Refusal detected: $HAS_REFUSAL"
if [ "$HAS_RECEIPT" = "False" ] && [ "$HAS_REFUSAL" = "False" ]; then
  echo "  FAIL: Neither Receipt nor Refusal detected — capability detection broken"
  exit 1
fi

# [5] Receipt file present
echo ""
echo "[5/7] Checking receipt..."
R1=$(ls "$OUT/receipts/"*.receipt.toml 2>/dev/null | head -1 || echo "")
if [ -z "$R1" ]; then
  echo "FAIL: No .receipt.toml found in $OUT/receipts/"
  exit 1
fi
echo "  OK: $R1"

# [6] Verify no-deletion between two scans
echo ""
echo "[6/7] verify-no-deletion..."
"$BINARY" computer discover "$FIXTURE" --out "$OUT2" 2>/dev/null
R2=$(ls "$OUT2/receipts/"*.receipt.toml 2>/dev/null | head -1 || echo "")
if [ -z "$R2" ]; then
  echo "FAIL: No receipt from second scan"
  exit 1
fi
"$BINARY" receipt verify-no-deletion "$R1" "$R2"
echo "  OK: no-deletion verified between two scans of same fixture"

# [7] Enterprise doctor
echo ""
echo "[7/7] Enterprise doctor..."
"$BINARY" enterprise doctor

# Cleanup
rm -rf "$OUT" "$OUT2"

echo ""
echo "==============================="
echo " SMOKE TEST: PASS"
echo "==============================="
