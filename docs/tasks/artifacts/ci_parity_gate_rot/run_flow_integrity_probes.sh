#!/usr/bin/env bash
# docs/tasks/artifacts/ci_parity_gate_rot/run_flow_integrity_probes.sh
#
# CI-PARITY-GATE-ROT.8 — RED/GREEN/CONTROL arms for the `FLOW-INTEGRITY` doctrine
# (`scripts/check_flow_integrity.sh`).
#
# ⭐ EACH RED ARM RE-CREATES ONE INCIDENT THIS CAMPAIGN ACTUALLY HAD. That is the point: the doctrine
# is not a list of good ideas, it is a list of things that already went wrong, and every arm here
# asserts that the specific historical defect would now be caught at commit time instead of after
# 1,371 commits, four sessions, or a 71-minute aggregate run.
#
# ⛔ THE WORKING TREE IS RESTORED BY AN EXIT TRAP whatever happens; the git index is never touched.
#
# Usage: bash docs/tasks/artifacts/ci_parity_gate_rot/run_flow_integrity_probes.sh
# Exit 0 iff every arm reaches its expected verdict.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"; cd "$ROOT"

CHECK="scripts/check_flow_integrity.sh"
REGISTER="rust/test_data/grammar_quality/flow_integrity_register_v0.json"
PARITY="rust/scripts/ci_workflow_local_gate.sh"
ACTION=".github/actions/regenerate-parsers/action.yml"
WORK="rust/target/ci_parity_gate_rot_probe/flow_arms"
rm -rf "$WORK"; mkdir -p "$WORK"

for f in "$REGISTER" "$PARITY" "$ACTION" rust/Makefile \
         .github/workflows/performance-gate.yml \
         .github/workflows/branch-protection-contract-gate.yml \
         rust/scripts/sv_failure_context_contract_gate.sh; do
  mkdir -p "$WORK/$(dirname "$f")"; cp "$ROOT/$f" "$WORK/$f"
done
restore() { (cd "$WORK" && find . -type f -print0) | while IFS= read -r -d '' r; do
              cp "$WORK/${r#./}" "$ROOT/${r#./}"; done; }
trap 'restore' EXIT

pass=0; fail=0
arm() {  # arm <label> <expected PASS|FAIL> [needle]
  local label="$1" want="$2" needle="${3:-}" out rc got
  out="$(bash "$ROOT/$CHECK" 2>&1)"; rc=$?
  [ "$rc" -eq 0 ] && got=PASS || got=FAIL
  if [ "$got" != "$want" ]; then
    fail=$((fail+1)); printf '✗ %-46s expected %s, got %s\n' "$label" "$want" "$got"
    printf '%s\n' "$out" | head -4 | sed 's/^/      /'; return 0; fi
  if [ -n "$needle" ] && ! printf '%s\n' "$out" | grep -qF -- "$needle"; then
    fail=$((fail+1)); printf '✗ %-46s %s but never mentions "%s"\n' "$label" "$got" "$needle"
    printf '%s\n' "$out" | head -4 | sed 's/^/      /'; return 0; fi
  pass=$((pass+1)); printf '✓ %-46s %s\n' "$label" "$got"
}

printf '%s\n' "=============================================================================="
printf 'CI-PARITY-GATE-ROT.8 — FLOW-INTEGRITY probes (each RED replays a real incident)\n'
printf '%s\n' "=============================================================================="

arm "GREEN  untouched tree" PASS "flow-integrity: OK"

# RED-1 — the `.4` incident: 14 of 15 workflows could not build.
grep -v 'uses: ./.github/actions/regenerate-parsers' "$WORK/.github/workflows/performance-gate.yml" \
  > "$ROOT/.github/workflows/performance-gate.yml"
arm "RED-1  workflow loses its regeneration step" FAIL "declares no regeneration step"
restore

# RED-2 — "the 3 that pass must NOT pay for it": an exempt job acquiring a 236s step.
printf '      - name: x\n        uses: ./.github/actions/regenerate-parsers\n' \
  >> "$ROOT/.github/workflows/branch-protection-contract-gate.yml"
arm "RED-2  exempt workflow acquires the step" FAIL "MEASURED-EXEMPT"
restore

# RED-3 — the `.6` incident: the flagship budgeted 60 min for a 143-min job.
python3 - "$ROOT/.github/workflows/performance-gate.yml" <<'PY'
import re,sys; p=sys.argv[1]; s=open(p).read()
open(p,"w").write(re.sub(r'timeout-minutes: \d+','timeout-minutes: 5',s,count=1))
PY
arm "RED-3  timeout below the floor" FAIL "below the 30-minute floor"
restore

# RED-4 — the recipe getting a second home (it was about to gain ten).
printf '      - name: x\n        run: make -C rust regex_parser_bootstrap\n' \
  >> "$ROOT/.github/workflows/performance-gate.yml"
arm "RED-4  recipe re-inlined in a workflow" FAIL "second home"
restore

# RED-5 — the recipe losing its home entirely.
python3 - "$ROOT/$ACTION" <<'PY'
import sys; p=sys.argv[1]; s=open(p).read()
open(p,"w").write(s.replace("make -C rust SHELL=/bin/bash regenerate_generated_parsers","true",1))
PY
arm "RED-5  action stops delegating to the target" FAIL "no longer delegates"
restore

# RED-6 — the `GENERATED-LINT-CORRECTNESS.3` erosion, replayed one surface over.
python3 - "$ROOT/$PARITY" <<'PY'
import sys; p=sys.argv[1]; s=open(p).read()
open(p,"w").write(s.replace('PGEN_CI_WORKFLOW_LOCAL_PREPARE:-true}"','PGEN_CI_WORKFLOW_LOCAL_PREPARE:-false}"',1))
PY
arm "RED-6  PREPARE default flipped off" FAIL "expected 'true'"
restore

# RED-7 — the `.7` incident: a hand-off pointed at a standalone default state dir.
python3 - "$ROOT/rust/scripts/sv_failure_context_contract_gate.sh" <<'PY'
import sys; p=sys.argv[1]; s=open(p).read()
open(p,"w").write(s + '\nEXISTING_PROBE_STATE_DIR="$RUST_DIR/target/sv_syntax_closure_gate"\n')
PY
arm "RED-7  hand-off at a standalone default dir" FAIL "STANDALONE default state dir"
restore

# RED-8 — the `.5` incident: an assertion that requires a defect in order to pass.
printf '\n# probe\nif [ "$x" -lt 1 ]; then echo "error: expected at least one probe excerpt" >&2; fi\n' \
  >> "$ROOT/rust/scripts/sv_failure_context_contract_gate.sh"
arm "RED-8  requires-a-defect assertion re-added" FAIL "requires a FAILURE to exist"
restore

# RED-9 — a NEW hand-off consumer that does not verify what it is handed.
cat > "$ROOT/rust/scripts/zz_probe_handoff_gate.sh" <<'SH'
#!/usr/bin/env bash
EXISTING_ZZ_PROBE_STATE_DIR="${PGEN_ZZ_PROBE_EXISTING_ZZ_PROBE_STATE_DIR:-}"
SH
arm "RED-9  NEW unverified hand-off consumer" FAIL "do not verify what they are handed"
rm -f "$ROOT/rust/scripts/zz_probe_handoff_gate.sh"

# RED-10 — a DEAD ratchet entry: a gate that now verifies but is still registered as not.
python3 - "$ROOT/$REGISTER" <<'PY'
import json,sys; p=sys.argv[1]; d=json.load(open(p))
d["handoff_provenance_ratchet"]["unverified_consumers"].append("sv_parser_family_status_gate")
json.dump(d,open(p,"w"),indent=2,ensure_ascii=False)
PY
arm "RED-10 dead ratchet entry" FAIL "now DO verify"
restore

# CTRL-1 — the register is the SINGLE source: the parity gate must read the same exemption set.
python3 - "$ROOT/$REGISTER" <<'PY'
import json,sys; p=sys.argv[1]; d=json.load(open(p))
d["regeneration_exempt_workflows"].pop(".github/workflows/mdbook-docs-gate.yml")
json.dump(d,open(p,"w"),indent=2,ensure_ascii=False)
PY
out="$(bash "$ROOT/$CHECK" 2>&1)"; doctrine_rc=$?
probe="rust/scripts/.flowprobe.sh"; sed 's/^main "\$@"$//' "$ROOT/$PARITY" > "$ROOT/$probe"
( set +e; . "$ROOT/$probe" >/dev/null 2>&1; audit_workflow_regeneration_surface ) >/dev/null 2>&1
audit_rc=$?; rm -f "$ROOT/$probe"
if [ "$doctrine_rc" -ne 0 ] && [ "$audit_rc" -ne 0 ]; then
  pass=$((pass+1)); printf '✓ %-46s both readers reject (single source holds)\n' "CTRL-1 register drives BOTH readers"
else
  fail=$((fail+1)); printf '✗ %-46s doctrine rc=%d audit rc=%d — they disagree\n' \
    "CTRL-1 register drives BOTH readers" "$doctrine_rc" "$audit_rc"
fi
restore

# CTRL-2 — an unrelated edit must not trip it. A gate that blames the nearest thing it knows about
# is worse than one that stays quiet.
printf '\n# CI-PARITY-GATE-ROT.8 probe: unrelated comment\n' >> "$ROOT/rust/Makefile"
arm "CTRL-2 unrelated Makefile edit" PASS
restore

printf '%s\n' "------------------------------------------------------------------------------"
printf 'arms=%d  PASS=%d  FAIL=%d\n' "$((pass + fail))" "$pass" "$fail"
[ "$fail" -eq 0 ]
