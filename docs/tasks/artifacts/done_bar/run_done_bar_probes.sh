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

REGISTER_SRC="$ROOT/rust/test_data/grammar_quality/done_bar_family_register_v0.json"
GRAMMARS_SRC="$ROOT/grammars"

# A grammars/ mirror the RED/CTRL roster arms mutate. Symlinks, so adding or removing one candidate
# costs nothing and the tracked tree is never touched.
mirror_grammars() {   # mirror_grammars <dest>
    local dest="$1"
    rm -rf "$dest"; mkdir -p "$dest"
    local f
    for f in "$GRAMMARS_SRC"/*.ebnf; do ln -s "$f" "$dest/$(basename "$f")"; done
}

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
#          the bar" (exit 1). The `.2b` demotion legitimately moved that ground truth: the register
#          now claims ZERO `Done` rows, and the audit must state that VACUOUS green explicitly
#          (exit 0) rather than pretending to have judged something. CTRL-1b keeps the old arm's
#          essence alive: a re-promoted `Done` row that does not meet the bar must still FAIL.
# ---------------------------------------------------------------------------
arm "CTRL-1 untouched tree states the vacuous zero-Done green" 0 "0 \`Done\` rows are claimed" --

# The claim now lives in the REGISTER (LIVE-MEANS-LIVE.1a), so the re-promotion is a register edit.
python3 - "$REGISTER_SRC" "$WORK/register_repromoted.json" <<'PY'
import json, sys
d = json.load(open(sys.argv[1]))
if d["families"]["vhdl"]["claimed_status"] != "Provisional (corpus pending)":
    raise SystemExit("probe fixture: vhdl claim is not `Provisional (corpus pending)` — register moved?")
d["families"]["vhdl"]["claimed_status"] = "Done"
json.dump(d, open(sys.argv[2], "w"), indent=2)
PY
arm "CTRL-1b a re-promoted unproven Done claim still fails" 1 "1 of 1 \`Done\` rows DO NOT meet the bar" \
    -- "PGEN_DONE_BAR_REGISTER=$WORK/register_repromoted.json"

# ---------------------------------------------------------------------------
# CTRL-2 — FALSE-POSITIVE GUARD. A row that is NOT a `Done` claim must be reported for context and
#          must NOT be counted as a failing `Done` row. Without this, "everything fails" would be
#          indistinguishable from an instrument that fails everything.
# ---------------------------------------------------------------------------
arm "CTRL-2 Mostly Done row is context, not a failure" 0 "not a \`Done\` claim — reported for context" --

# ---------------------------------------------------------------------------
# CTRL-3 — FALSE-POSITIVE GUARD for the NEW refusal (LIVE-MEANS-LIVE.1b). The roster now derives
#          from grammars/*.ebnf, so every tracked grammar must be adjudicated — and 7 of the 17 are
#          adjudicated as NON-families (bootstrap contracts, LRM extraction inputs, derived
#          artifacts). Those must stay out of the roster. Without this arm, "refuse on anything
#          unadjudicated" could not be distinguished from "admit everything".
# ---------------------------------------------------------------------------
arm "CTRL-3 dispositioned grammars are not admitted as families" 0 \
    "17 tracked grammars = 10 families + 7 recorded non-families" --

# ---------------------------------------------------------------------------
# RED-1 — ⭐ THE ARM `.1b` EXISTS FOR. A tracked grammar adjudicated NEITHER as a family NOR with a
#         recorded disposition must REFUSE (exit 2). Under the OLD tracker-joined derivation this
#         grammar would simply not have been a family — silently, exit 0 — which is how `ebnf`,
#         `json` and `semantic_annotation` stayed invisible while shipping registered parsers.
# ---------------------------------------------------------------------------
mirror_grammars "$WORK/grammars_undisposed"
printf 'probe_undisposed := "x"\n' >"$WORK/grammars_undisposed/probe_undisposed.ebnf"
arm "RED-1 a grammar with no entry and no disposition refuses" 2 \
    "no register entry and no recorded disposition" \
    -- "PGEN_DONE_BAR_GRAMMARS_DIR=$WORK/grammars_undisposed"

# RED-1b — the contradiction arm: a grammar claimed BOTH as a family and as a non-family. A register
#          that says both cannot be audited, and picking one silently would be the audit choosing
#          the answer.
python3 - "$REGISTER_SRC" "$WORK/register_both_ways.json" <<'PY'
import json, sys
d = json.load(open(sys.argv[1]))
d["grammar_dispositions"]["json"] = {"disposition": "derived_artifact", "reason": "probe contradiction"}
json.dump(d, open(sys.argv[2], "w"), indent=2)
PY
arm "RED-1b a grammar adjudicated BOTH ways refuses" 2 "appear in BOTH" \
    -- "PGEN_DONE_BAR_REGISTER=$WORK/register_both_ways.json"

# RED-1c — the CONVERSE arm, and the reason the check is two-sided: an entry naming a grammar that
#          does not exist is a STALE adjudication describing a tree that is gone. Without this, the
#          register could quietly keep covering a deleted grammar forever.
python3 - "$REGISTER_SRC" "$WORK/register_stale_entry.json" <<'PY'
import json, sys
d = json.load(open(sys.argv[1]))
d["grammar_dispositions"]["grammar_that_was_deleted"] = {"disposition": "derived_artifact", "reason": "probe stale"}
json.dump(d, open(sys.argv[2], "w"), indent=2)
PY
arm "RED-1c a stale entry naming no grammar refuses" 2 "name no tracked grammars/*.ebnf file" \
    -- "PGEN_DONE_BAR_REGISTER=$WORK/register_stale_entry.json"

# RED-1d — the claim is never DEFAULTED. A family whose `claimed_status` is absent must refuse, not
#          be audited against nothing. Mirrors the same refusal `claimed_status_for_family` makes in
#          rust/scripts/lib/parser_family_status_bar.sh (LIVE-MEANS-LIVE.1a).
python3 - "$REGISTER_SRC" "$WORK/register_no_claim.json" <<'PY'
import json, sys
d = json.load(open(sys.argv[1]))
del d["families"]["vhdl"]["claimed_status"]
json.dump(d, open(sys.argv[2], "w"), indent=2)
PY
arm "RED-1d a family with no claimed_status refuses" 2 "is never defaulted" \
    -- "PGEN_DONE_BAR_REGISTER=$WORK/register_no_claim.json"

# ---------------------------------------------------------------------------
# RED-2 — an EMPTY derived roster must REFUSE, never exit 0. A derivation that finds nothing and
#         reports success is the vacuous green this repository has shipped before
#         (CI-PARITY-GATE-ROT.3: a mistyped filter replayed zero workflows and printed ✅).
# ---------------------------------------------------------------------------
mkdir -p "$WORK/grammars_empty"
arm "RED-2 empty roster refuses" 2 "no grammars/*.ebnf found" \
    -- "PGEN_DONE_BAR_GRAMMARS_DIR=$WORK/grammars_empty"

# ---------------------------------------------------------------------------
# RED-2b — ⭐ THE ADMISSION RULE (control C8). The convenient way to silence RED-1 is to give a real
#          shipping parser a disposition instead of a register entry. C8 pins the rule against
#          rust/src/parser_registry.rs — an INDEPENDENT tracked source the register cannot edit —
#          so demoting a family that ships a registered generated parser is MISCALIBRATED, not a
#          quiet re-classification.
# ---------------------------------------------------------------------------
python3 - "$REGISTER_SRC" "$WORK/register_family_demoted.json" <<'PY'
import json, sys
d = json.load(open(sys.argv[1]))
del d["families"]["json"]          # json ships a registered generated parser
d["grammar_dispositions"]["json"] = {"disposition": "derived_artifact", "reason": "probe demotion"}
json.dump(d, open(sys.argv[2], "w"), indent=2)
PY
arm "RED-2b demoting a registered parser to a disposition is MISCALIBRATED" 3 \
    "C8a every registered parser is a family" \
    -- "PGEN_DONE_BAR_REGISTER=$WORK/register_family_demoted.json"

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
# CTRL-4 — GROUND TRUTH FOR THE INSTRUMENT ITSELF: facts measured independently of this script that
#          must be reproduced, or the report is not evidence.
#
# ⚠️⚠️ CTRL-4a WAS REWRITTEN BY LIVE-MEANS-LIVE.1b, AND WHY IS THE LESSON. It used to pin a failure
#      recorded in rust/target/ during aggregate run 3 — `computed 'In Progress' but tracker says
#      'Done'` — and it was found RED at commit ce1df2b0, for a reason having nothing to do with
#      the instrument it guards:
#
#        $ wc -c rust/target/regex_parser_family_status_gate/summary.txt          # 7377 (non-empty)
#        $ grep -c '^error:' rust/target/sota_exit_gate/logs/regex_…_gate.log     # 0
#
#      The gate had since been re-run and PASSED, so `find_artifact()` now finds a summary and the
#      `gate_ran_and_failed()` recovery path — the only place that string can come from — is never
#      taken. ⇒ **A GROUND-TRUTH CONTROL PINNED TO UNTRACKED STATE DECAYS SILENTLY.** rust/target/
#      is regenerable build output; the arm was green only until someone re-ran that gate. Its
#      sibling CTRL-4b pins a fact derived from TRACKED script text and is still green.
#
#      The rule this bought: **a control must pin a TRACKED fact, or CONSTRUCT the state it
#      observes.** CTRL-4a now constructs it — a synthetic target dir with exactly the shape
#      DONE-BAR.1a was written for (a 0-byte summary.txt beside a log naming the real cause), so it
#      reproduces on a fresh clone with an empty rust/target/ and can never decay again.
# ---------------------------------------------------------------------------
FAKE_TARGET="$WORK/target_gate_ran_and_failed"
mkdir -p "$FAKE_TARGET/sota_exit_gate/logs" "$FAKE_TARGET/regex_parser_family_status_gate"
: >"$FAKE_TARGET/regex_parser_family_status_gate/summary.txt"        # 0 bytes = died mid-run
cat >"$FAKE_TARGET/sota_exit_gate/logs/regex_parser_family_status_gate.log" <<'EOF'
regex_parser_family_status_gate: starting
error: regex tracker alignment mismatch: computed 'In Progress' but register says 'Done'
EOF
arm "CTRL-4a a gate that RAN AND DIED is recovered from its log" 0 \
    "computed 'In Progress' but register says 'Done'" \
    -- "PGEN_DONE_BAR_TARGET_DIR=$FAKE_TARGET"
arm "CTRL-4b vhdl corpus lane is TRIAGE" 0 "a TRIAGE gate is not a conformance gate" --

echo "------------------------------------------------------------------------------"
printf 'done-bar probes: %d/%d passed\n' "$pass" "$((pass + fail))"
[[ "$fail" -eq 0 ]] || exit 1
