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
         .github/workflows/memory-architecture-gate.yml \
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

# ---------------------------------------------------------------- invariant (8), CI-PARITY-GATE-ROT.15
AUTO_WF=".github/workflows/memory-architecture-gate.yml"

# RED-11 — THE VERBATIM `.15` INCIDENT: the auto workflow names enforcers instead of the driver.
# This is the exact shape measured on 2026-07-29 — five by name, 8 of 13 doctrines with no
# automatic lane, and every doctrine registered afterwards inheriting none.
python3 - "$ROOT/$AUTO_WF" <<'PY'
import sys; p=sys.argv[1]; s=open(p).read()
open(p,"w").write(s.replace("run: bash scripts/check_doctrines.sh",
                            "run: bash scripts/check_memory_architecture.sh",1))
PY
arm "RED-11 auto workflow names an enforcer" FAIL "BY NAME"
restore

# RED-12 — the roster loses its automatic lane entirely: the driver step is still there but the
# workflow no longer runs on push/pull_request. ⭐ A DIFFERENT DEFECT FROM RED-11 and the one that
# would otherwise be silent — nothing else in the repo would notice E4 had gone away.
python3 - "$ROOT/$AUTO_WF" <<'PY'
import sys; p=sys.argv[1]; s=open(p).read()
open(p,"w").write(s.replace("on:\n  workflow_dispatch:\n  push:\n  pull_request:\n",
                            "on:\n  workflow_dispatch:\n",1))
PY
arm "RED-12 doctrine roster loses its auto lane" FAIL "NO automatically-triggered workflow invokes"
restore

# CTRL-3 — ⛔ THE FALSE-POSITIVE THAT WOULD MAKE INVARIANT (8) UNUSABLE. A `workflow_dispatch`-only
# workflow naming an enforcer is NOT the defect: the rule is about the AUTOMATIC lane, and 14 of the
# 15 tracked workflows are manual. If this arm fails, the trigger classification is broken and the
# invariant is blaming files it has no business inspecting.
printf '      - name: probe\n        run: bash scripts/check_memory_architecture.sh\n' \
  >> "$ROOT/.github/workflows/performance-gate.yml"
arm "CTRL-3 manual workflow names an enforcer" PASS
restore

# RED-13 — the driver invoked only from a COMMENT must not count as a lane. That is the shape a
# half-finished revert leaves behind, and this repository has twice shipped an audit that fired on
# its own comment; the converse — a comment read as a live invocation — is the same defect inverted.
# ⛔ EVERY invocation is commented, not just the first: leaving one live would test nothing.
python3 - "$ROOT/$AUTO_WF" <<'PY'
import sys; p=sys.argv[1]
out=[]
for l in open(p):
    out.append(("#" + l) if "scripts/check_doctrines.sh" in l and not l.lstrip().startswith("#") else l)
open(p,"w").writelines(out)
PY
arm "RED-13 commented-out driver is not a lane" FAIL "NO automatically-triggered workflow invokes"
restore

# ---------------------------------------------------------------- invariant (9), CI-PARITY-GATE-ROT.14
# RED-14 — the verbatim `.14` incident: a block guarded on summary.txt whose else-branch jq-reads
# summary.json. The mutation is deliberately placed inside a NESTED if/else at the same indentation,
# because that nesting is exactly what made this check's own first cut miss 1 of the 6 real sites.
cat > "$ROOT/rust/scripts/zz_probe_guard_gate.sh" <<'SH'
#!/usr/bin/env bash
if [[ "$mode" == "a" ]]; then
    if [[ ! -f "$ZZ_PROBE_SUMMARY_TXT" ]]; then
        ZZ_PROBE_GATE="<missing>"
        if [[ "$inner" == "1" ]]; then
            ZZ_PROBE_INNER="x"
        else
            ZZ_PROBE_INNER="y"
        fi
    else
        ZZ_PROBE_GATE="$(jq -r '.gate' "$ZZ_PROBE_SUMMARY_JSON")"
    fi
fi
SH
arm "RED-14 guard tests summary.txt but reads .json" FAIL "guards a block by testing summary.txt"
rm -f "$ROOT/rust/scripts/zz_probe_guard_gate.sh"

# CTRL-5 — the CORRECT form must not trip it. Without this arm the invariant could be satisfied by
# a check that flags every guard it sees, which would make the fix itself unlandable.
cat > "$ROOT/rust/scripts/zz_probe_guard_gate.sh" <<'SH'
#!/usr/bin/env bash
if [[ ! -s "$ZZ_PROBE_SUMMARY_TXT" || ! -s "$ZZ_PROBE_SUMMARY_JSON" ]]; then
    ZZ_PROBE_GATE="<missing>"
else
    ZZ_PROBE_GATE="$(jq -r '.gate' "$ZZ_PROBE_SUMMARY_JSON")"
fi
SH
arm "CTRL-5 the corrected guard form passes" PASS
rm -f "$ROOT/rust/scripts/zz_probe_guard_gate.sh"

# ---------------------------------------------------------------- invariant (10), CI-PARITY-GATE-ROT.31
# ⭐ RED-15 IS THE VERBATIM INCIDENT, REPLAYED FROM THE COMMIT THAT HAD IT — not a hand-written
# imitation of it. `0994c3c0` is the last commit whose `rust/Makefile` defined
# `RUST_GENERATOR = … --generate-parser --debug --trace …`; the sha is PINNED rather than symbolic,
# because `HEAD` silently re-points the moment the fix lands and the arm would then test nothing.
NOISY_MAKEFILE_SHA="0994c3c0"
if git -C "$ROOT" cat-file -e "$NOISY_MAKEFILE_SHA:rust/Makefile" 2>/dev/null; then
  git -C "$ROOT" show "$NOISY_MAKEFILE_SHA:rust/Makefile" > "$ROOT/rust/Makefile"
  arm "RED-15 the pre-fix Makefile (0994c3c0)" FAIL "on the SHIPPING path"
  restore
else
  fail=$((fail+1))
  printf '✗ %-46s %s:rust/Makefile is unreachable — the arm tested NOTHING\n' \
    "RED-15 the pre-fix Makefile" "$NOISY_MAKEFILE_SHA"
fi

# RED-16 — only ONE of the two flags comes back. `--debug` alone is the shape
# `RUST_GENERATOR_BOOTSTRAP` actually carried, and it is the half a partial revert restores.
printf '\nZZ_PROBE_GEN = $(RUST_AST_PIPELINE) --generate-parser --debug -o /dev/null\n' \
  >> "$ROOT/rust/Makefile"
arm "RED-16 --debug alone re-added" FAIL "invokes the generator with --debug"
restore

# CTRL-6 — ⛔ THE FALSE POSITIVE THAT WOULD MAKE INVARIANT (10) UNUSABLE. `--trace-rules` is a
# LIVE debug tool this repository documents in TOOLBOX.md §2.2; a substring match would condemn it
# and teach the next author to waive the gate rather than fix anything.
printf '\nZZ_PROBE_GEN = $(RUST_AST_PIPELINE) --generate-parser --trace-rules foo -o /dev/null\n' \
  >> "$ROOT/rust/Makefile"
arm "CTRL-6 --trace-rules is not --trace" PASS
restore

# CTRL-7 — the OPT-IN shape must stay legal: a variable may HOLD the flags, because the rule is
# about a shipping INVOCATION, not about the tokens existing. Without this arm the invariant would
# forbid the very escape hatch the director's ruling requires be kept.
printf '\nZZ_PROBE_TRACE_FLAGS = --debug --trace\n' >> "$ROOT/rust/Makefile"
arm "CTRL-7 a variable may hold the flags" PASS
restore

# CTRL-8 — prose must be untouched. The fix's own comment block quotes the removed flags next to
# the words "--generate-parser"; a check that fired on its own explanation would be unlandable.
printf '\n# probe: RUST_GENERATOR used to run --generate-parser --debug --trace here\n' \
  >> "$ROOT/rust/Makefile"
arm "CTRL-8 a comment quoting the flags" PASS
restore

printf '%s\n' "------------------------------------------------------------------------------"
printf 'arms=%d  PASS=%d  FAIL=%d\n' "$((pass + fail))" "$pass" "$fail"
[ "$fail" -eq 0 ]
