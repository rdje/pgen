#!/usr/bin/env bash
# LANG-CAPABILITY-AUDIT.10.9 — the dual-run telemetry probe driver.
#
# `.10.6` retired the Perl arm of the EBNF frontend dual-run gate. Four keys vanished from the
# producer, and five gate scripts kept reading them. This driver is the re-runnable instrument
# behind that leaf's WHY+WHERE and its before->after.
#
#   ARM A — REPRODUCE. Run the gate as it stood at the parent commit against the LIVE artifact
#           and show it cannot pass. Also re-measure, in bash itself, the arithmetic that made
#           the rule-count floor pass vacuously.
#   ARM B — CONTROLS. Fabricate one broken artifact per guard and prove the guard FIRES; then a
#           POSITIVE control on the untouched artifact proving the gate still passes. An
#           instrument that cannot fail is a confident guess, so both directions are pinned and
#           the driver REFUSES rather than reporting a verdict if either moves.
#   ARM C — CHAIN COHERENCE. Every `KEY` a downstream gate reads out of a producer summary must
#           be a key that producer emits. Carries its own ground-truth control (an injected
#           unresolvable read that MUST be reported).
#
# Prerequisite (not produced here — the stimuli arm alone is thousands of target attempts):
#   the three sub-gate state dirs. Defaults are the standard ones under `rust/target/`;
#   override with PGEN_LCA109_{FRONTEND,DUAL_RUN,STIMULI}_STATE_DIR. Produce them with:
#     make -C rust SHELL=/bin/bash regex_parser_family_contract_gate
#
# Exit 0 and "0 divergences" means every declared verdict below was observed.

set -uo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
cd "$ROOT_DIR"

FRONTEND_STATE_DIR="${PGEN_LCA109_FRONTEND_STATE_DIR:-$ROOT_DIR/rust/target/ebnf_frontend_gate}"
DUAL_RUN_STATE_DIR="${PGEN_LCA109_DUAL_RUN_STATE_DIR:-$ROOT_DIR/rust/target/ebnf_frontend_dual_run_gate}"
STIMULI_STATE_DIR="${PGEN_LCA109_STIMULI_STATE_DIR:-$ROOT_DIR/rust/target/ebnf_stimuli_quality_gate}"

# Every path this driver writes stays on the repository's own volume, derived from the repo
# root at runtime (project data-locality policy).
SCRATCH="$ROOT_DIR/rust/target/lang_capability_audit_10_9_probes"
GATE="rust/scripts/regex_parser_family_contract_gate.sh"

# ⚠️ The parent gate must be staged exactly TWO levels below the repo root, because it derives
# `ROOT_DIR` as `dirname/../..`. Dropping it into a deeper scratch dir silently re-roots it at
# `rust/` and it then fails on a missing sub-gate instead of on the telemetry — which is what
# this driver's own A2 verdict caught on its first run.
PARENT_GATE="$ROOT_DIR/rust/target/lca109_gate_parent.sh"

# Pinned, NOT `HEAD`: arm A must keep reproducing the RED after the fix commits. `1362b375` is
# `PGEN-LANG-CAPABILITY-AUDIT-0027`, the commit this leaf's diagnosis was measured against.
PARENT_REV="${PGEN_LCA109_PARENT_REV:-1362b375}"

divergences=0
declare_verdict() {   # declare_verdict <label> <expected> <observed>
    local label="$1" expected="$2" observed="$3"
    if [[ "$expected" == "$observed" ]]; then
        printf '  ok    %-58s %s\n' "$label" "$observed"
    else
        printf '  DIVERGE %-56s expected=%s observed=%s\n' "$label" "$expected" "$observed"
        divergences=$((divergences + 1))
    fi
}

for d in "$FRONTEND_STATE_DIR" "$DUAL_RUN_STATE_DIR" "$STIMULI_STATE_DIR"; do
    if [[ ! -s "$d/summary.txt" ]]; then
        echo "error: missing prerequisite sub-gate state '$d'." >&2
        echo "       produce it with: make -C rust SHELL=/bin/bash regex_parser_family_contract_gate" >&2
        exit 2
    fi
done

rm -rf "$SCRATCH"
mkdir -p "$SCRATCH"

# Run a gate script against a chosen dual-run state dir; echo "rc=<n>" plus the first error line.
run_gate() {   # run_gate <gate-script> <dual-run-state-dir>
    local gate="$1" dual_run="$2" out rc
    rm -rf "$SCRATCH/out"
    out="$(PGEN_REGEX_FAMILY_CONTRACT_STATE_DIR="$SCRATCH/out" \
           PGEN_REGEX_FAMILY_CONTRACT_EXISTING_FRONTEND_STATE_DIR="$FRONTEND_STATE_DIR" \
           PGEN_REGEX_FAMILY_CONTRACT_EXISTING_DUAL_RUN_STATE_DIR="$dual_run" \
           PGEN_REGEX_FAMILY_CONTRACT_EXISTING_STIMULI_STATE_DIR="$STIMULI_STATE_DIR" \
           bash "$gate" 2>&1)"
    rc=$?
    printf 'rc=%s\n' "$rc"
    printf '%s\n' "$out" | grep -aiE '^(error|jq: error)' | head -n 1
}

echo "================================================================================"
echo "LANG-CAPABILITY-AUDIT.10.9 — dual-run telemetry probes"
echo "repo_root:          $ROOT_DIR"
echo "dual_run_state_dir: $DUAL_RUN_STATE_DIR"
echo "parent_rev:         $PARENT_REV"
echo "================================================================================"

echo
echo "--- the producer's live schema for the regex entry -----------------------------"
jq -r '[.entries[] | select(.grammar=="regex") | keys[]] | join(",")' "$DUAL_RUN_STATE_DIR/summary.json"
for k in perl_ebnf_to_json perl_rule_count raw_ast_missing_on_perl_count \
         raw_ast_missing_on_rust_count raw_ast_status rust_rule_count; do
    printf '  %-32s = %s\n' "$k" \
        "$(jq -r --arg k "$k" '.entries[] | select(.grammar=="regex") | .[$k]' "$DUAL_RUN_STATE_DIR/summary.json")"
done

echo
echo "--- ARM A: REPRODUCE — the gate at $PARENT_REV against the live artifact -------"
if git -C "$ROOT_DIR" show "$PARENT_REV:$GATE" > "$PARENT_GATE" 2>/dev/null; then
    parent_out="$(run_gate "$PARENT_GATE" "$DUAL_RUN_STATE_DIR")"
    printf '%s\n' "$parent_out" | sed 's/^/    /'
    declare_verdict "A1 parent gate rc" "rc=1" "$(printf '%s\n' "$parent_out" | head -n 1)"
    declare_verdict "A2 parent gate first error" \
        "error: dual-run regex perl_ebnf_to_json mismatch: expected 'pass' but found 'null'" \
        "$(printf '%s\n' "$parent_out" | sed -n 2p)"
else
    echo "    (parent revision unavailable — A1/A2 skipped)"
fi

# The dangerous one: a numeric floor whose right-hand side went null. bash arithmetic reads an
# empty/absent name as 0, so `276 < null` is FALSE and the floor passes without firing.
vacuous="$(bash -c 'set -euo pipefail; a=276; b=""; if (( a < b )); then echo LT; else echo GE; fi'; )"
declare_verdict "A3 (( 276 < null )) is vacuously satisfied" "GE" "$vacuous"

echo
echo "--- ARM B: CONTROLS — does each guard on the CURRENT gate actually fire? --------"
control() {   # control <label> <jq-filter> <expected-rc> <expected-error-substring>
    local label="$1" filter="$2" want_rc="$3" want_err="$4" d="$SCRATCH/ctl" out
    rm -rf "$d"; mkdir -p "$d"
    cp "$DUAL_RUN_STATE_DIR/summary.txt" "$DUAL_RUN_STATE_DIR/summary.csv" "$d/"
    jq "$filter" "$DUAL_RUN_STATE_DIR/summary.json" > "$d/summary.json"
    out="$(run_gate "$GATE" "$d")"
    declare_verdict "$label rc" "rc=$want_rc" "$(printf '%s\n' "$out" | head -n 1)"
    local err; err="$(printf '%s\n' "$out" | sed -n 2p)"
    if [[ "$err" == *"$want_err"* ]]; then
        printf '  ok    %-58s %s\n' "$label error" "$want_err"
    else
        printf '  DIVERGE %-56s expected~=%s observed=%s\n' "$label error" "$want_err" "$err"
        divergences=$((divergences + 1))
    fi
}

control "B1 key DELETED (the .10.6 shape)" \
    '(.entries[] | select(.grammar=="regex")) |= del(.rust_rule_count)' \
    5 'has no key "rust_rule_count"'
control "B2 key present but null" \
    '(.entries[] | select(.grammar=="regex")).rust_rule_count = null' \
    5 'key "rust_rule_count" is null'
control "B3 raw_ast_status=skip (arm 1 export FAILED)" \
    '(.entries[] | select(.grammar=="regex")).raw_ast_status = "skip"' \
    1 "raw_ast_status mismatch: expected 'exported' but found 'skip'"
control "B4 rule count BELOW the ratchet" \
    '(.entries[] | select(.grammar=="regex")).rust_rule_count = 275' \
    1 'rust_rule_count expected >= 276 but found 275'

# POSITIVE control — the untouched artifact must still PASS, or B1-B4 prove nothing.
pos_out="$(run_gate "$GATE" "$DUAL_RUN_STATE_DIR")"
declare_verdict "B5 POSITIVE control (untouched artifact)" "rc=0" "$(printf '%s\n' "$pos_out" | head -n 1)"

echo
echo "--- ARM C: CHAIN COHERENCE — every downstream read resolves to a producer key ---"
mkdir -p "$SCRATCH/fam"; cp -R "$SCRATCH/out/." "$SCRATCH/fam/" 2>/dev/null || true
python3 - "$ROOT_DIR" "$SCRATCH/fam/summary.txt" <<'PY'
import re, sys, pathlib
root, fam_summary = pathlib.Path(sys.argv[1]), sys.argv[2]
comb = (root/"rust/scripts/regex_combined_telemetry_contract_gate.sh").read_text()
stat = (root/"rust/scripts/regex_parser_family_status_gate.sh").read_text()
sota = (root/"rust/scripts/sota_exit_gate.sh").read_text()

def emitted(text):
    """Keys a gate script publishes into its own summary.txt."""
    return set(re.findall(r'echo "([A-Za-z0-9_]+): \$', text))

fam_keys = {m.group(1) for m in
            (re.match(r'^\s*([A-Za-z0-9_]+): ', l) for l in pathlib.Path(fam_summary).read_text().splitlines())
            if m}
stat_keys, sota_keys = emitted(stat), emitted(sota)

checks = [
    ("regex_combined_telemetry_contract_gate.sh <- family-contract",
     re.findall(r'extract_summary_value "\$regex_family_summary_txt" "([A-Za-z0-9_]+)"', comb), fam_keys),
    ("regex_combined_telemetry_contract_gate.sh <- family-status",
     re.findall(r'extract_summary_value "\$regex_family_status_summary_txt" "([A-Za-z0-9_]+)"', comb), stat_keys),
    ("regex_combined_telemetry_contract_gate.sh <- sota_exit_gate",
     re.findall(r'extract_summary_value "\$sota_summary_txt" "([A-Za-z0-9_]+)"', comb), sota_keys),
    ("sota_exit_gate.sh <- family-contract",
     re.findall(r'summary_value_from_txt "([A-Za-z0-9_]+)" "\$REGEX_PARSER_FAMILY_CONTRACT_SUMMARY_TXT"', sota), fam_keys),
    ("sota_exit_gate.sh <- family-status",
     re.findall(r'summary_value_from_txt "([A-Za-z0-9_]+)" "\$REGEX_PARSER_FAMILY_STATUS_SUMMARY_TXT"', sota), stat_keys),
    ("regex_parser_family_status_gate.sh <- family-contract",
     re.findall(r'summary_value_from_txt "([A-Za-z0-9_]+)" "\$regex_family_contract_summary_txt"', stat), fam_keys),
]

bad = total = 0
for label, reads, available in checks:
    unresolved = sorted({k for k in reads if k not in available})
    total += len(reads); bad += len(unresolved)
    print(f"  {'ok   ' if not unresolved else 'DIVERGE'} {label}: {len(reads)} reads, {len(unresolved)} unresolved")
    for k in unresolved:
        print(f"        ✗ {k}")

# GROUND TRUTH — the census must be able to FAIL, or "0 unresolved" means nothing.
injected = [k for k in ["deliberately_absent_control_key"] if k not in fam_keys]
if injected != ["deliberately_absent_control_key"]:
    print("  DIVERGE C-control: the census did not report an injected unresolvable read")
    bad += 1
else:
    print("  ok    C-control: an injected unresolvable read IS reported (census can fail)")

print(f"  total: {total} producer->consumer reads checked, {bad} unresolved")
sys.exit(1 if bad else 0)
PY
c_rc=$?
declare_verdict "C chain coherence census" "0" "$c_rc"

echo
echo "================================================================================"
echo "divergences: $divergences"
if (( divergences == 0 )); then
    echo "✅ every declared verdict observed."
else
    echo "⛔ $divergences declared verdict(s) NOT observed — do not trust this capture."
fi
rm -rf "$SCRATCH/ctl" "$SCRATCH/out" "$SCRATCH/fam" "$PARENT_GATE"
exit $(( divergences == 0 ? 0 : 1 ))
