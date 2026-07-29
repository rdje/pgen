#!/usr/bin/env bash
# run_done_bar_probes.sh — probe arms for scripts/audit_done_bar.sh (task-tree leaf `DONE-BAR.1`).
#
# WHY THIS EXISTS. `audit_done_bar.sh` reports that 5 of 5 `Done` rows do not meet the bar. That is
# an uncomfortable number, and an uncomfortable number from an instrument nobody has tried to break
# is a claim, not a measurement. These arms break it deliberately, one input at a time:
#
#   RED-*   a defect is INJECTED; the audit must REFUSE or report MISCALIBRATED. An arm that does
#           not flip means the corresponding control is decorative.
#   CTRL-*  a HEALTHY state; the audit must behave identically to the untouched tree. These are the
#           false-positive guards — the arms that stop the instrument from failing everything.
#
# Every arm asserts the EXIT CODE **and** a substring of the message it expects. Comparing only
# pass/fail was measured to hide 8 arms reaching the right verdict for the wrong reason
# (CI-PARITY-GATE-ROT.4), so an arm here is green only when it fails the way it intended to.
#
# Mutations are made on COPIES under rust/target/ (the repository volume, derived from the repo root
# at runtime per the project data-locality policy) and reach the audit through its default-safe
# testability seams. The tracked tree is never modified.
#
#   bash docs/tasks/artifacts/done_bar/run_done_bar_probes.sh
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
cd "$ROOT"

AUDIT="scripts/audit_done_bar.sh"
WORK="$ROOT/rust/target/done_bar_audit/probes"
rm -rf "$WORK"
mkdir -p "$WORK"

TRACKER_SRC="$ROOT/LIVE_ACHIEVEMENT_STATUS.md"
REGISTER_SRC="$ROOT/rust/test_data/grammar_quality/done_bar_family_register_v0.json"

pass=0
fail=0

# arm <label> <expected-exit> <expected-substring> -- <env assignments...>
arm() {
    local label="$1" want_exit="$2" want_text="$3"
    shift 3
    [[ "${1:-}" == "--" ]] && shift
    local out rc
    out="$(env "$@" bash "$AUDIT" 2>&1)"
    rc=$?
    local ok_exit="no" ok_text="no"
    [[ "$rc" == "$want_exit" ]] && ok_exit="yes"
    grep -qF -- "$want_text" <<<"$out" && ok_text="yes"
    if [[ "$ok_exit" == yes && "$ok_text" == yes ]]; then
        printf '  ✅ %-58s exit=%s, message matched\n' "$label" "$rc"
        pass=$((pass + 1))
    else
        printf '  ❌ %-58s exit=%s (want %s), message matched=%s\n' "$label" "$rc" "$want_exit" "$ok_text"
        printf '     wanted substring: %s\n' "$want_text"
        printf '%s\n' "$out" | tail -n 12 | sed 's/^/     | /'
        fail=$((fail + 1))
    fi
}

echo "=============================================================================="
echo "DONE-BAR.1 probe arms — scripts/audit_done_bar.sh"
echo "=============================================================================="

# ---------------------------------------------------------------------------
# CTRL-1 — the untouched tree. Until DONE-BAR.2b this arm pinned "5 of 5 `Done` rows DO NOT meet
#          the bar" (exit 1). The `.2b` demotion legitimately moved that ground truth: the tracker
#          now claims ZERO `Done` rows, and the audit must state that VACUOUS green explicitly
#          (exit 0) rather than pretending to have judged something. CTRL-1b keeps the old arm's
#          essence alive: a re-promoted `Done` row that does not meet the bar must still FAIL.
# ---------------------------------------------------------------------------
arm "CTRL-1 untouched tree states the vacuous zero-Done green" 0 "0 \`Done\` rows are claimed" --

cp "$TRACKER_SRC" "$WORK/tracker_repromoted.md"
python3 - "$WORK/tracker_repromoted.md" <<'PY'
import sys
p = sys.argv[1]
t = open(p, encoding="utf-8").read()
old = "| `vhdl` parser family | Provisional (corpus pending) |"
if old not in t:
    raise SystemExit("probe fixture: vhdl Provisional row not found — tracker moved again?")
t = t.replace(old, "| `vhdl` parser family | Done |", 1)
open(p, "w", encoding="utf-8").write(t)
PY
arm "CTRL-1b a re-promoted unproven Done row still fails" 1 "1 of 1 \`Done\` rows DO NOT meet the bar" \
    -- "PGEN_DONE_BAR_TRACKER=$WORK/tracker_repromoted.md"

# ---------------------------------------------------------------------------
# CTRL-2 — FALSE-POSITIVE GUARD. A row that is NOT a `Done` claim must be reported for context and
#          must NOT be counted as a failing `Done` row. Without this, "everything fails" would be
#          indistinguishable from an instrument that fails everything.
# ---------------------------------------------------------------------------
arm "CTRL-2 Mostly Done row is context, not a failure" 0 "not a \`Done\` claim — reported for context" --

# ---------------------------------------------------------------------------
# CTRL-3 — the `gate-level` trap, replayed. A tracker row whose Area cell carries a backticked token
#          that is NOT a tracked grammar must not become a family. A naive "has a backtick" reader
#          admits "Later auxiliary readers (`gate-level` netlist reader…)" and reports 8 families.
# ---------------------------------------------------------------------------
cp "$TRACKER_SRC" "$WORK/tracker_backtick_trap.md"
cat >>"$WORK/tracker_backtick_trap.md" <<'EOF'

| Area | Status | Evidence | Left To Close |
|---|---|---|---|
| `not-a-grammar-at-all` synthetic probe row | Done | probe | probe |
EOF
arm "CTRL-3 non-grammar backtick is not a family" 0 "grammars/*.ebnf: 7" \
    -- "PGEN_DONE_BAR_TRACKER=$WORK/tracker_backtick_trap.md"

# ---------------------------------------------------------------------------
# RED-1 — a family on the tracker with NO register entry must REFUSE (exit 2). The safe polarity:
#         an unregistered family blocks the audit rather than being silently skipped, because a skip
#         would let a brand-new `Done` row score well by being invisible.
# ---------------------------------------------------------------------------
cp "$TRACKER_SRC" "$WORK/tracker_unregistered.md"
cat >>"$WORK/tracker_unregistered.md" <<'EOF'

| Area | Status | Evidence | Left To Close |
|---|---|---|---|
| `json` parser family (synthetic probe row) | Done | probe | probe |
EOF
arm "RED-1 unregistered family refuses" 2 "absent from" \
    -- "PGEN_DONE_BAR_TRACKER=$WORK/tracker_unregistered.md"

# ---------------------------------------------------------------------------
# RED-2 — an EMPTY derived roster must REFUSE, never exit 0. A derivation that finds nothing and
#         reports success is the vacuous green this repository has shipped before
#         (CI-PARITY-GATE-ROT.3: a mistyped filter replayed zero workflows and printed ✅).
# ---------------------------------------------------------------------------
printf '# Live Achievement Status\n\nNo tables here.\n' >"$WORK/tracker_empty.md"
arm "RED-2 empty roster refuses" 2 "derived zero parser families" \
    -- "PGEN_DONE_BAR_TRACKER=$WORK/tracker_empty.md"

# ---------------------------------------------------------------------------
# RED-3 — break longest-prefix attribution: drop the `sv_preprocessor_` prefix so `sv_` wins and the
#         preprocessor's own closure gate is credited to `systemverilog`. Control C4 must fire.
# ---------------------------------------------------------------------------
python3 - "$REGISTER_SRC" "$WORK/register_no_pp_prefix.json" <<'PY'
import json, sys
d = json.load(open(sys.argv[1]))
d["families"]["systemverilog_preprocessor"]["gate_prefixes"] = ["systemverilog_preprocessor_"]
json.dump(d, open(sys.argv[2], "w"), indent=2)
PY
arm "RED-3 broken prefix attribution is MISCALIBRATED" 3 "C4 sv_preprocessor_formal_exhaustive_closure_gate owner" \
    -- "PGEN_DONE_BAR_REGISTER=$WORK/register_no_pp_prefix.json"

# ---------------------------------------------------------------------------
# RED-4 — THE DEFECT THIS INSTRUMENT ACTUALLY MADE. Attributing family-status gates by NAME prefix
#         left `systemverilog_preprocessor` with no status gate at all and reported it worse than it
#         measures. Remove the preprocessor's emitted status key and control C9 must fire.
# ---------------------------------------------------------------------------
mkdir -p "$WORK/scripts_no_pp_status"
cp "$ROOT"/rust/scripts/*.sh "$WORK/scripts_no_pp_status/"
sed -i.bak 's/echo "systemverilog_preprocessor_status:/echo "SUPPRESSED_preprocessor_status:/' \
    "$WORK/scripts_no_pp_status/sv_parser_family_status_gate.sh"
rm -f "$WORK/scripts_no_pp_status/sv_parser_family_status_gate.sh.bak"
arm "RED-4 status-gate coverage lost is MISCALIBRATED" 3 "C9 sv_parser_family_status_gate covers both SV families" \
    -- "PGEN_DONE_BAR_SCRIPTS_DIR=$WORK/scripts_no_pp_status"

# ---------------------------------------------------------------------------
# RED-5 — MENTION COUNTED AS INVOCATION, the first defect the gate-reachability instrument had to
#         fix. Disable the ci_workflow_local_gate.sh caller exclusion and a gate referenced only by
#         that gate's assert_tracked lines starts reading as "something runs it". Control C7 fires.
# ---------------------------------------------------------------------------
arm "RED-5 mention-as-invocation is MISCALIBRATED" 3 "C7 regex_corpus_bundle_contract_gate runs" \
    -- "PGEN_DONE_BAR_NO_CALLER_EXCLUSIONS=1"

# ---------------------------------------------------------------------------
# RED-6 — the register must be CHECKABLE, not trusted. Declare that `vhdl` has no external corpus
#         while its corpus-facing gate is still attributed to it; the cross-check must fire rather
#         than accepting a hand-written claim that flatters the family.
# ---------------------------------------------------------------------------
python3 - "$REGISTER_SRC" "$WORK/register_vhdl_no_corpus.json" <<'PY'
import json, sys
d = json.load(open(sys.argv[1]))
d["families"]["vhdl"]["corpus_roots"] = []
json.dump(d, open(sys.argv[2], "w"), indent=2)
PY
arm "RED-6 register contradicted by gate set is MISCALIBRATED" 3 "register says vhdl has no corpus roots" \
    -- "PGEN_DONE_BAR_REGISTER=$WORK/register_vhdl_no_corpus.json"

# ---------------------------------------------------------------------------
# CTRL-4 — GROUND TRUTH FOR THE INSTRUMENT ITSELF. The two facts below were measured independently
#          of this script and must be reproduced verbatim, or the report is not evidence:
#            (a) regex's status gate RAN and FAILED with a named cause in aggregate run 3;
#            (b) VHDL's only external-corpus lane is a TRIAGE gate.
# ---------------------------------------------------------------------------
arm "CTRL-4a regex failure recovered verbatim" 0 "computed 'In Progress' but tracker says 'Done'" --
arm "CTRL-4b vhdl corpus lane is TRIAGE" 0 "a TRIAGE gate is not a conformance gate" --

echo "------------------------------------------------------------------------------"
printf 'done-bar probes: %d/%d passed\n' "$pass" "$((pass + fail))"
[[ "$fail" -eq 0 ]] || exit 1
