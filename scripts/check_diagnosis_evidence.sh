#!/usr/bin/env bash
# scripts/check_diagnosis_evidence.sh
# DIAG-TOOLBOX-ENFORCE (PGEN-DIAG-TOOLBOX-0002/0003): make a fix PROVABLY follow the
# task-acceptance procedure from start to finish — not "trust me". Per the director directive
# (2026-06-22): "every task tree shall check some boxes, [go] through certain steps to analyse
# an issue, and made sure the issue it wanted to address was clearly addressed with no
# regression." This gate enforces a REQUIRED ACCEPTANCE CHECKLIST in the owning task leaf for any
# code change: each required box must be TICKED ([x]) and backed by real tool-output evidence; an
# unticked or missing required box BLOCKS the commit. Exits NONZERO on any breach.
#
# The required checklist (label keywords are flexible; the [x] and the keyword are what matter):
#   - [x] ROOT CAUSE (WHY + WHERE) ........ backed by a DIAGNOSIS tool signature (correctness:
#                                           cert/probe/trace/reach/lint; performance/SPEED phase:
#                                           a profiler — `sample`/`flamegraph` self-time+call-graph)
#   - [x] ADDRESSED (verified)  ........... the issue is resolved (before->after on the symptom)
#   - [x] NO REGRESSION ................... backed by a global-gate signature (seeds 0/7/42, etc.)
# (REPRODUCE/FIX/LOCKSTEP boxes are recommended by the template but not hard-required here, to
# avoid false-blocking; the three above are the director's named steps and ARE required.)
#
# Why "not trust-me-bro": (1) a ticked box must co-occur with the real tool-output signature that
# only the debug tools / deterministic gates emit; (2) the project's DETERMINISTIC gates
# (cert-coverage at seeds 0/7/42, ast_shape_contract, syntax-closure, external-corpus) re-RUN the
# real tools in the make gates / CI, so a fabricated "NO REGRESSION" claim does not reproduce and
# fails there. Honest limit: a hook cannot prove the author reasoned — only that the boxes are
# ticked, the evidence is present, and (via the oracle gates) the numbers reproduce.
#
# Called by scripts/check_doctrines.sh (the general enforcer) via .githooks/pre-commit (E3) + CI
# (E4). Recorded exception (loud, never silent): PGEN_DIAG_EVIDENCE_WAIVER="<reason>".
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"; cd "$ROOT"

if [ -n "${PGEN_DIAG_EVIDENCE_RANGE:-}" ]; then
  mapfile -t staged < <(git diff --name-only --diff-filter=ACMR "$PGEN_DIAG_EVIDENCE_RANGE")
else
  mapfile -t staged < <(git diff --cached --name-only --diff-filter=ACMR)
fi

# Is this a CODE change? (grammars, rust sources, generated parsers, AST shape-contract manifests)
code_changed=0
for f in "${staged[@]:-}"; do
  case "$f" in
    grammars/*.ebnf|rust/src/*|generated/*|rust/test_data/ast_shape_contract/*.json) code_changed=1 ;;
  esac
done

if [ "$code_changed" -eq 0 ]; then
  echo "diag-evidence: OK (no code change staged; task-acceptance checklist not required)"
  exit 0
fi

if [ -n "${PGEN_DIAG_EVIDENCE_WAIVER:-}" ]; then
  printf 'diag-evidence: ⚠️ WAIVED by PGEN_DIAG_EVIDENCE_WAIVER="%s" — exception recorded; CI still re-checks.\n' \
    "$PGEN_DIAG_EVIDENCE_WAIVER" >&2
  exit 0
fi

# A code change must be owned by a staged task-tree leaf carrying the acceptance checklist.
mapfile -t staged_tasks < <(printf '%s\n' "${staged[@]:-}" | grep -E '^docs/tasks/.*\.md$' || true)
if [ "${#staged_tasks[@]}" -eq 0 ]; then
  cat >&2 <<'MSG'
diag-evidence: ✗ a CODE change is staged but NO owning task-tree leaf (docs/tasks/*.md) is staged.
  Stage the owning docs/tasks/<TREE>.md carrying the ACCEPTANCE CHECKLIST (TOOLBOX.md template).
MSG
  exit 1
fi

# Scan the staged leaves DIRECTLY (grep reads the files itself). Do NOT do
# `printf '%s\n' "$leaf_text" | grep -q` — under `set -o pipefail`, `grep -q` exits on the first
# match and closes the pipe, the upstream `printf` then takes SIGPIPE (rc 141), and pipefail
# propagates that as the pipeline's status, so a genuinely-present box is reported as MISSING once
# the concatenated leaf text exceeds the pipe buffer (~64 KB) and the match is early. Grepping the
# files directly has no upstream writer to kill, so the match is deterministic at any file size.
# `grep -q` over multiple files returns 0 iff ANY line in ANY file matches — identical to scanning
# the concatenation. (STORE-AWARE-GEN.4b.12: this race began false-failing once the owning task
# leaf grew past ~64 KB.)

# A checked / unchecked checklist box mentioning a category keyword.
checked()   { grep -Eiq "^[[:space:]]*[-*][[:space:]]*\[[xX]\][[:space:]].*($1)" "${staged_tasks[@]}"; }
unchecked() { grep -Eiq "^[[:space:]]*[-*][[:space:]]*\[[[:space:]]\][[:space:]].*($1)" "${staged_tasks[@]}"; }

# Evidence signatures that must BACK the ticked boxes.
# The first group is the CORRECTNESS-defect diagnosis toolbox (cert/probe/trace/reach/lint — "why
# does this parse WRONG / where"). The second group (added for the SPEED phase, director 2026-07-11
# — speed is now a first-class, continuously-tracked deliverable) is the PERFORMANCE-defect diagnosis
# toolbox: a slowness defect is legitimately root-caused by a PROFILER (macOS `sample`, `cargo
# flamegraph`) via self-time / call-graph attribution, NOT by a correctness tool. Tokens are kept
# tight so they cannot match unrelated text (e.g. `self-time`/`call-graph`/`flamegraph` never appear
# inside `sample_parse_failures`). Rationale + the case that motivated it (RGX-0078.3):
# docs/decisions/project_speed_phase_profiler_root_cause_signature.md.
DIAGNOSIS_SIG='CERTIFICATE-COVERAGE:|\[plannable-probe\]|rejected by post predicate|furthest_position=|witnessed_target=(true|false)|PGEN_CERT_COVERAGE_(DUMP_ALL|DEBUG_PROBES)|PGEN_REACH_PATH_DUMP|--report-certificate-coverage|--trace-rules|--dump-rule-call-counts|--lint-grammar|--parse-dump-ast|self-time|call-graph attribution|call-graph samples|cargo flamegraph|flamegraph'
NOREGRESS_SIG='seeds? *0/7/42|byte-identical|external corpus *1[0-9]/1[0-9]|corpus *1[0-9]/1[0-9]|shape.?contract|spf=0|sample_parse_failures=0|fully_certified|clippy'

fails=()

# Required box 1 — ROOT CAUSE (WHY + WHERE) ticked + a diagnosis tool signature present.
if   unchecked 'root cause|why ?\+ ?where|\bwhy\b'; then fails+=("ROOT CAUSE box is present but UNTICKED ([ ]) — the cause is not yet established.")
elif ! checked 'root cause|why ?\+ ?where|\bwhy\b'; then fails+=("ROOT CAUSE (WHY+WHERE) box is MISSING/unticked from the acceptance checklist.")
elif ! grep -Eq "$DIAGNOSIS_SIG" "${staged_tasks[@]}"; then
  fails+=("ROOT CAUSE box is ticked but NOT backed by a debug-tool signature (cert/probe/trace/furthest_position).")
fi

# Required box 2 — ADDRESSED (verified) ticked.
if   unchecked 'addressed|verified|resolved|before.{0,5}after|reject.{0,6}pass'; then fails+=("ADDRESSED/VERIFIED box is present but UNTICKED — the fix is not yet confirmed to resolve the issue.")
elif ! checked 'addressed|verified|resolved|before.{0,5}after|reject.{0,6}pass'; then fails+=("ADDRESSED (verified the issue is resolved) box is MISSING/unticked.")
fi

# Required box 3 — NO REGRESSION ticked + a global-gate signature present.
if   unchecked 'no.?regress|regression'; then fails+=("NO REGRESSION box is present but UNTICKED — regressions are not yet cleared.")
elif ! checked 'no.?regress|regression'; then fails+=("NO REGRESSION box is MISSING/unticked from the acceptance checklist.")
elif ! grep -Eiq "$NOREGRESS_SIG" "${staged_tasks[@]}"; then
  fails+=("NO REGRESSION box is ticked but NOT backed by a global-gate signature (seeds 0/7/42, byte-identical, external corpus, shape-contract, spf=0).")
fi

if [ "${#fails[@]}" -gt 0 ]; then
  cat >&2 <<'MSG'
diag-evidence: ✗ the staged task leaf does NOT pass the required ACCEPTANCE CHECKLIST for a code
  change. A fix must be PROVABLY taken through the procedure (analyse -> root cause -> fix ->
  addressed -> no regression), not "trust me". Add/complete the checklist (template in TOOLBOX.md):
    - [x] ROOT CAUSE (WHY + WHERE)  — backed by a debug-tool signature (cert / [plannable-probe] / predicate-rejection trace / furthest_position=)
    - [x] ADDRESSED (verified)      — the issue is resolved (before->after on the symptom)
    - [x] NO REGRESSION             — global metrics (cert seeds 0/7/42 spf=0, 6 grammars byte-identical, external corpus 14/14, ast_shape_contract GREEN, clippy)
  An UNTICKED required box means the task is not done — finish the step, do not bypass. The
  deterministic gates re-run the oracles in CI, so the NO-REGRESSION numbers you cite are re-verified.
  Breaches:
MSG
  for m in "${fails[@]}"; do printf '  - %s\n' "$m" >&2; done
  exit 1
fi

echo "diag-evidence: OK (task leaf passes the acceptance checklist: ROOT CAUSE + ADDRESSED + NO REGRESSION, evidence-backed)"
exit 0
