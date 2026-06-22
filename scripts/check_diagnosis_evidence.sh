#!/usr/bin/env bash
# scripts/check_diagnosis_evidence.sh
# DIAG-TOOLBOX-ENFORCE (PGEN-DIAG-TOOLBOX-0002): make "use the debug toolbox" ENFORCEABLE,
# not prose. Per the director directive (2026-06-22) + MEMORY_ARCHITECTURE.md §9 (defense in
# depth: discovery → self-check → git hook → CI), a code change may not land unless the staged
# owning task-tree leaf carries REAL, re-checkable tool-output evidence of the WHY+WHERE
# diagnosis (the debug toolbox, TOOLBOX.md). Exits NONZERO on a breach.
#
# What it enforces: a commit that touches CODE (grammars/*.ebnf, rust/src/**, generated/**,
# the ast_shape_contract manifests) MUST stage at least one docs/tasks/*.md whose content
# carries >=1 recognized tool-output / tool-invocation signature. Pure-docs commits are exempt.
#
# Why this is "not trust-me-bro": the signatures below are strings the real tools emit (cert
# report header, plannable-probe lines, the predicate-rejection trace, furthest_position), and
# the project's DETERMINISTIC gates (certificate-coverage, ast_shape_contract, syntax-closure)
# re-RUN those tools at fixed seeds in CI — so a fabricated FINAL STATE is caught independently
# of this hook. This gate enforces "show your work in the tracked leaf"; the deterministic gates
# enforce "the work is real". Honest limit: a hook cannot prove the agent reasoned from the
# evidence — but it makes landing a code change with NO pasted, reproducible tool output
# impossible locally and un-mergeable in CI.
#
# Called by .githooks/pre-commit (E3) and intended for CI (E4). Override for a genuine
# exception (rare; recorded, not silent): PGEN_DIAG_EVIDENCE_WAIVER="<reason>" — it prints the
# waived reason loudly so the exception is visible in the commit run, never a quiet bypass.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"; cd "$ROOT"

# Files staged for THIS commit (added/copied/modified/renamed). Fall back to a diff vs HEAD
# when run outside a commit (CI on a range can set PGEN_DIAG_EVIDENCE_RANGE).
if [ -n "${PGEN_DIAG_EVIDENCE_RANGE:-}" ]; then
  mapfile -t staged < <(git diff --name-only --diff-filter=ACMR "$PGEN_DIAG_EVIDENCE_RANGE")
else
  mapfile -t staged < <(git diff --cached --name-only --diff-filter=ACMR)
fi

# Is this a CODE change? (the doctrine's definition of "code": grammars, rust sources, generated
# parsers, and the AST shape-contract manifests). Pure docs/tooling commits are exempt.
code_changed=0
for f in "${staged[@]:-}"; do
  case "$f" in
    grammars/*.ebnf|rust/src/*|generated/*|rust/test_data/ast_shape_contract/*.json)
      code_changed=1 ;;
  esac
done

if [ "$code_changed" -eq 0 ]; then
  echo "diag-evidence: OK (no code change staged; toolbox-evidence not required)"
  exit 0
fi

# Genuine, recorded exception (loud, never silent).
if [ -n "${PGEN_DIAG_EVIDENCE_WAIVER:-}" ]; then
  printf 'diag-evidence: ⚠️ WAIVED by PGEN_DIAG_EVIDENCE_WAIVER="%s" — exception recorded; CI still re-checks.\n' \
    "$PGEN_DIAG_EVIDENCE_WAIVER" >&2
  exit 0
fi

# A code change must be owned by a staged task-tree leaf (COMMIT.md / Code-Change Doctrine).
mapfile -t staged_tasks < <(printf '%s\n' "${staged[@]:-}" | grep -E '^docs/tasks/.*\.md$' || true)
if [ "${#staged_tasks[@]}" -eq 0 ]; then
  cat >&2 <<'MSG'
diag-evidence: ✗ a CODE change is staged but NO owning task-tree leaf (docs/tasks/*.md) is staged.
  The Code-Change Doctrine requires the change be owned by a task leaf, and that leaf must carry
  the tool-backed WHY+WHERE diagnosis. Stage the owning docs/tasks/<TREE>.md with a tool-evidence
  block. See TOOLBOX.md (the debug toolbox + the 3-step UNKNOWN protocol).
MSG
  exit 1
fi

# Enforce the ROOT-CAUSE PROCEDURE, not merely "some evidence". The leaf must show BOTH:
#   (1) DIAGNOSIS — the WHY+WHERE was established with a debug tool (the tool that located and
#       explained the cause: cert report / forced-probe verdict / predicate-rejection trace /
#       furthest_position / an explicit toolbox invocation); AND
#   (2) VERIFICATION — the fix's effect was MEASURED before->after, deterministically (a metric
#       delta, REJECT->PASS, seeds 0/7/42, byte-identical, spf=0).
# Together these are the mechanizable proxy for "followed a procedure + reasoned from the
# evidence": a tool-located cause AND a measured, reproducible effect. The DETERMINISTIC gates
# (cert-coverage / ast_shape_contract / syntax-closure), re-run by the make gates and CI, then
# independently RE-EXECUTE the cited oracle at fixed seeds — so a cause->fix->effect chain that
# does not reproduce FAILS. (A reproducible chain is, operationally, a correct diagnosis.)
DIAGNOSIS_SIG='CERTIFICATE-COVERAGE:|\[plannable-probe\]|rejected by post predicate|furthest_position=|sample_parse_failures=|witnessed_target=(true|false)|PGEN_CERT_COVERAGE_(DUMP_ALL|DEBUG_PROBES)|PGEN_REACH_PATH_DUMP|--report-certificate-coverage|--trace-rules|--dump-rule-call-counts|--lint-grammar|--parse-dump-ast'
VERIFY_SIG='[0-9]+ *(->|→) *[0-9]+|REJECT[^a-z]*(->|→|to)[^a-z]*PASS|seeds? *0/7/42|byte-identical|spf=0|sample_parse_failures=0|UNKNOWN=[0-9]'

diag_found=0; verify_found=0
for t in "${staged_tasks[@]}"; do
  [ -f "$t" ] || continue
  grep -Eq "$DIAGNOSIS_SIG" "$t" && diag_found=1
  grep -Eq "$VERIFY_SIG"    "$t" && verify_found=1
done

if [ "$diag_found" -eq 0 ] || [ "$verify_found" -eq 0 ]; then
  cat >&2 <<'MSG'
diag-evidence: ✗ code change staged, but the staged task leaf does not show the tool-backed
  ROOT-CAUSE PROCEDURE. A fix must be provably diagnosed, not "trust me". The owning
  docs/tasks/<TREE>.md must contain BOTH:
    (1) DIAGNOSIS (WHY+WHERE) — pasted output from a debug tool that located + explained the
        cause: a CERTIFICATE-COVERAGE: line, a [plannable-probe] verdict, a "rejected by post
        predicate" trace, a furthest_position= error, or the exact toolbox command run; AND
    (2) VERIFICATION — the measured before->after effect: a metric delta (e.g. "UNKNOWN 56->55"),
        REJECT->PASS, determinism at seeds 0/7/42, byte-identical, or spf=0.
  Procedure (TOOLBOX.md): run the toolbox FIRST (DUMP_ALL -> DEBUG_PROBES -> scoped trace) to get
  (1); apply the minimal fix; re-run the deterministic oracle for (2); paste both into the leaf;
  then commit. CI re-runs the deterministic gates, so the numbers you cite are re-verified.
MSG
  [ "$diag_found"   -eq 0 ] && printf '  -> missing: DIAGNOSIS signature (the WHY+WHERE tool output).\n' >&2
  [ "$verify_found" -eq 0 ] && printf '  -> missing: VERIFICATION signature (the measured before->after effect).\n' >&2
  exit 1
fi

echo "diag-evidence: OK (code change shows tool-backed DIAGNOSIS + measured VERIFICATION)"
exit 0
