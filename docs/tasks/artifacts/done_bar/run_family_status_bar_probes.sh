#!/usr/bin/env bash
# run_family_status_bar_probes.sh — prove the DONE-BAR.2a shared helper and the re-taught
# family-status gates behave as chartered (task-tree leaf `DONE-BAR.2a`).
#
# WHAT IS PROVED, in two layers:
#
#   1. HELPER ARMS — rust/scripts/lib/parser_family_status_bar.sh, driven through its testability
#      seams. Every RED arm asserts BOTH the exit code AND a substring of the message (comparing
#      only pass/fail was measured to hide arms reaching the right verdict for the wrong reason —
#      CI-PARITY-GATE-ROT.4). CTRL-6 is the arm that keeps `Done` HONESTLY reachable: a declared
#      surface that passes all three leg-3 tests scores met=true, so the cap is a criterion, not a
#      hard-coded false.
#
#   2. GATE REPLAYS — the three live *_parser_family_status_gate.sh scripts replayed against the
#      aggregate run-3 artifacts (the same artifact set DONE-BAR.1 audited), via the same
#      EXISTING_*_STATE_DIR hand-off the aggregate itself uses. Post-`.2b` (the rows moved), the
#      replays assert the ALIGNED steady state: vhdl and systemverilog_preprocessor compute
#      `Provisional (corpus pending)` and the tracker agrees (exit 0); regex computes `In Progress`
#      and the tracker agrees; systemverilog stays `Mostly Done`. The `.2a`-era capture
#      (family_status_bar_probes.txt at `PGEN-DONE-BAR-0010`) preserves the transitional state —
#      same computed statuses, exit 1 against the then-stale `Done` rows, with the full summary
#      pair emitted before the exit (the 0-byte-summary fix, CI-PARITY-GATE-ROT.14). The BEFORE of
#      `.2a` is on record in demotion_impact_probe.txt (Provisional in zero gate scripts).
#
#   bash docs/tasks/artifacts/done_bar/run_family_status_bar_probes.sh
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
cd "$ROOT"

LIB="$ROOT/rust/scripts/lib/parser_family_status_bar.sh"
WORK="$ROOT/rust/target/done_bar_audit/status_bar_probe"
rm -rf "$WORK"; mkdir -p "$WORK/scripts" "$WORK/replay"

pass=0
fail=0
arm() {
    local label="$1" want_rc="$2" want_substr="$3" got_rc="$4" got_out="$5"
    if [[ "$got_rc" == "$want_rc" && "$got_out" == *"$want_substr"* ]]; then
        printf '   ✅ %-52s rc=%s, matched %q\n' "$label" "$got_rc" "$want_substr"
        pass=$((pass + 1))
    else
        printf '   ⛔ %-52s want rc=%s + %q; got rc=%s\n' "$label" "$want_rc" "$want_substr" "$got_rc"
        printf '      output: %s\n' "$(head -c 600 <<<"$got_out")"
        fail=$((fail + 1))
    fi
}

# run_leg3 FAMILY — source the lib and evaluate leg 3, printing the four globals; env seams are
# inherited from the caller. Runs in its own bash so a REFUSAL (exit 2) is captured, not fatal.
run_leg3() {
    local family="$1"
    bash -c '
        set -euo pipefail
        source "'"$LIB"'"
        family_done_bar_leg3 "'"$family"'" "'"$WORK"'"
        echo "met=$DONE_BAR_LEG3_MET qual=$DONE_BAR_PROVISIONAL_QUALIFIER gate=$DONE_BAR_LEG3_SURFACE_GATE"
        echo "detail=$DONE_BAR_LEG3_DETAIL"
    ' 2>&1
}

echo "=============================================================================="
echo "FAMILY-STATUS DONE-BAR PROBES — the .2a helper + the re-taught gates"
echo "=============================================================================="
echo
echo "1. HELPER ARMS (lib/parser_family_status_bar.sh through its seams)"
echo

# --- CTRL-1: real register, vhdl => leg 3 unmet, qualifier (corpus pending), no surface ----------
out="$(run_leg3 vhdl)"; rc=$?
arm "CTRL-1 vhdl: unmet + (corpus pending)" 0 "met=false qual=(corpus pending) gate=<none>" "$rc" "$out"

# --- CTRL-2: real register, return_annotation => qualifier (ceiling), a FINISHED row -------------
out="$(run_leg3 return_annotation)"; rc=$?
arm "CTRL-2 return_annotation: (ceiling)" 0 "met=false qual=(ceiling) gate=<none>" "$rc" "$out"

# --- RED-1: family absent from the register => REFUSE (exit 2), never a comfortable score --------
printf '{"families":{}}' >"$WORK/register_empty.json"
out="$(PGEN_FAMILY_STATUS_DONE_BAR_REGISTER="$WORK/register_empty.json" run_leg3 vhdl)"; rc=$?
arm "RED-1 unregistered family: REFUSE" 2 "absent from the done-bar register" "$rc" "$out"

# --- RED-2: UNADJUDICATED owner => REFUSE — a status gate may not guess the qualifier ------------
out="$(run_leg3 rtl_frontend)"; rc=$?
arm "RED-2 unadjudicated owner: REFUSE" 2 "UNADJUDICATED" "$rc" "$out"

# --- Fixtures for the declared-surface arms ------------------------------------------------------
cat >"$WORK/scripts/fake_conformance_gate.sh" <<'EOF'
#!/usr/bin/env bash
# synthetic probe fixture: reads the declared external corpus root fake_corpus_root/
EOF
cat >"$WORK/scripts/fake_nonbacked_gate.sh" <<'EOF'
#!/usr/bin/env bash
# synthetic probe fixture: reads only a repo-authored fixture, no external corpus reference
EOF
cat >"$WORK/scripts/fake_corpus_triage_gate.sh" <<'EOF'
#!/usr/bin/env bash
# synthetic probe fixture: a triage gate that DOES read fake_corpus_root/ — kind must still refuse
EOF
mk_register() { # mk_register GATE -> register json declaring that surface for fakefam
    jq -n --arg gate "$1" '{families: {fakefam: {
        gate_prefixes: ["fakefam_"], corpus_roots: ["fake_corpus_root"],
        language_owner: "external-standard", standard: "FAKE-1234",
        leg3_surface: {gate: $gate,
                       summary_json: "rust/target/done_bar_audit/status_bar_probe/fake_summary.json",
                       pass_query: ".status == \"pass\" and .failures == 0"}}}}'
}
printf '{"status":"pass","cases":100,"failures":0}' >"$WORK/fake_summary.json"
jq -n '{rows: [{target: "fake_conformance_gate", status: "reachable", invokers: ["aggregate"], kind: "gate-named"}]}' >"$WORK/reach_reachable.json"
jq -n '{rows: [{target: "fake_conformance_gate", status: "ORPHAN", invokers: [], kind: "gate-named"}]}' >"$WORK/reach_orphan.json"

# --- RED-3: a declared surface naming a *triage* gate => REFUSE (register error) -----------------
mk_register "fake_corpus_triage_gate" >"$WORK/register_triage.json"
out="$(PGEN_FAMILY_STATUS_DONE_BAR_REGISTER="$WORK/register_triage.json" \
       PGEN_FAMILY_STATUS_SCRIPTS_DIR="$WORK/scripts" \
       PGEN_FAMILY_STATUS_REACHABILITY_JSON="$WORK/reach_reachable.json" run_leg3 fakefam)"; rc=$?
arm "RED-3 triage surface: REFUSE" 2 "TRIAGE gate is not a conformance" "$rc" "$out"

# --- RED-4: a declared surface whose script reads no declared corpus root => REFUSE --------------
mk_register "fake_nonbacked_gate" >"$WORK/register_nonbacked.json"
out="$(PGEN_FAMILY_STATUS_DONE_BAR_REGISTER="$WORK/register_nonbacked.json" \
       PGEN_FAMILY_STATUS_SCRIPTS_DIR="$WORK/scripts" \
       PGEN_FAMILY_STATUS_REACHABILITY_JSON="$WORK/reach_reachable.json" run_leg3 fakefam)"; rc=$?
arm "RED-4 not external-backed: REFUSE" 2 "not external-backed" "$rc" "$out"

# --- CTRL-5: valid surface but ORPHAN per reachability => leg 3 UNMET (verdict, not refusal) -----
mk_register "fake_conformance_gate" >"$WORK/register_valid.json"
out="$(PGEN_FAMILY_STATUS_DONE_BAR_REGISTER="$WORK/register_valid.json" \
       PGEN_FAMILY_STATUS_SCRIPTS_DIR="$WORK/scripts" \
       PGEN_FAMILY_STATUS_REACHABILITY_JSON="$WORK/reach_orphan.json" run_leg3 fakefam)"; rc=$?
arm "CTRL-5 orphan surface: unmet, named" 0 "not invoked by anything that runs" "$rc" "$out"

# --- CTRL-6: valid surface, reachable, artifact passes => leg 3 MET — Done stays reachable -------
out="$(PGEN_FAMILY_STATUS_DONE_BAR_REGISTER="$WORK/register_valid.json" \
       PGEN_FAMILY_STATUS_SCRIPTS_DIR="$WORK/scripts" \
       PGEN_FAMILY_STATUS_REACHABILITY_JSON="$WORK/reach_reachable.json" run_leg3 fakefam)"; rc=$?
arm "CTRL-6 valid surface: leg 3 MET" 0 "met=true qual=(corpus pending) gate=fake_conformance_gate" "$rc" "$out"

# --- CTRL-7: the failing-artifact half of CTRL-6 — a pass_query that does not hold => unmet ------
printf '{"status":"fail","cases":100,"failures":3}' >"$WORK/fake_summary.json"
out="$(PGEN_FAMILY_STATUS_DONE_BAR_REGISTER="$WORK/register_valid.json" \
       PGEN_FAMILY_STATUS_SCRIPTS_DIR="$WORK/scripts" \
       PGEN_FAMILY_STATUS_REACHABILITY_JSON="$WORK/reach_reachable.json" run_leg3 fakefam)"; rc=$?
arm "CTRL-7 failing artifact: unmet, named" 0 "does NOT satisfy its declared pass assertion" "$rc" "$out"
printf '{"status":"pass","cases":100,"failures":0}' >"$WORK/fake_summary.json"

# --- CTRL-8: the ladder cap — Done+unmet => qualified Provisional; below-Done passes through -----
ladder() {
    bash -c '
        set -euo pipefail
        source "'"$LIB"'"
        DONE_BAR_LEG3_MET="'"$1"'"
        DONE_BAR_PROVISIONAL_QUALIFIER="'"$2"'"
        family_apply_done_bar_status "'"$3"'"
    ' 2>&1
}
out="$(ladder false "(corpus pending)" "Done")"; rc=$?
arm "CTRL-8a Done+unmet => Provisional (corpus pending)" 0 "Provisional (corpus pending)" "$rc" "$out"
out="$(ladder false "(ceiling)" "Done")"; rc=$?
arm "CTRL-8b Done+unmet => Provisional (ceiling)" 0 "Provisional (ceiling)" "$rc" "$out"
out="$(ladder true "(corpus pending)" "Done")"; rc=$?
arm "CTRL-8c Done+met => Done" 0 "Done" "$rc" "$out"
out="$(ladder false "(corpus pending)" "Mostly Done")"; rc=$?
arm "CTRL-8d Mostly Done passes through" 0 "Mostly Done" "$rc" "$out"
out="$(ladder false "" "Done")"; rc=$?
arm "CTRL-8e call-order guard: no qualifier => REFUSE" 2 "called before family_done_bar_leg3" "$rc" "$out"

echo
echo "2. GATE REPLAYS against the aggregate run-3 artifacts (the measured AFTER of .2a)"
echo

SOTA="$ROOT/rust/target/sota_exit_gate/work"
missing=0
for d in \
    "$SOTA/regex_parser_family_contract_gate" \
    "$SOTA/regex_parser_family_status_gate/work/regex_formal_exhaustive_closure_gate" \
    "$SOTA/vhdl_parser_family_contract_gate" \
    "$SOTA/vhdl_parser_family_status_gate/work/vhdl_formal_exhaustive_closure_gate" \
    "$SOTA/sv_parser_family_status_gate/work/sv_syntax_closure_gate" \
    "$SOTA/sv_parser_family_status_gate/work/sv_preprocessor_syntax_closure_gate" \
    "$SOTA/sv_parser_aggregate_contract_gate" \
    "$SOTA/sv_stimuli_quality_gate" \
    "$SOTA/sv_preprocessor_aggregate_contract_gate" \
    "$SOTA/sv_preprocessor_reachability_closure_gate" \
    "$SOTA/sv_parser_family_status_gate/work/sv_preprocessor_formal_exhaustive_closure_gate" \
    "$SOTA/sv_preprocessor_quality_gate" \
    "$SOTA/sv_parser_family_status_gate/work/sv_semantic_scope_contract_gate" \
    "$SOTA/sv_parser_family_status_gate/work/sv_formal_exhaustive_closure_gate"; do
    if [[ ! -s "$d/summary.txt" ]]; then
        echo "   ⚠️  replay input missing or empty: $d/summary.txt"
        missing=$((missing + 1))
    fi
done
if [[ "$missing" -gt 0 ]]; then
    echo "   ⚠️  SKIPPING the gate replays: $missing run-3 artifact(s) unavailable on this machine."
    echo "      The helper arms above still verify the .2a logic; re-run after an aggregate run."
else
    # regex — legacy ladder computes In Progress (leg-1 debt); the cap does not apply below Done.
    # Post-.2b the tracker row says In Progress too, so the gate ALIGNS and passes.
    rgx_state="$WORK/replay/regex_parser_family_status_gate"
    out="$(PGEN_REGEX_FAMILY_STATUS_STATE_DIR="$rgx_state" \
           PGEN_REGEX_FAMILY_STATUS_EXISTING_FAMILY_CONTRACT_STATE_DIR="$SOTA/regex_parser_family_contract_gate" \
           PGEN_REGEX_FAMILY_STATUS_EXISTING_FORMAL_EXHAUSTIVE_CLOSURE_STATE_DIR="$SOTA/regex_parser_family_status_gate/work/regex_formal_exhaustive_closure_gate" \
           bash "$ROOT/rust/scripts/regex_parser_family_status_gate.sh" 2>&1)"; rc=$?
    arm "REPLAY regex: ALIGNED, gate passes" 0 "✅ Regex parser-family status gate passed." "$rc" "$out"
    arm "REPLAY regex: computed In Progress recorded" 0 "regex_status: In Progress" 0 "$(cat "$rgx_state/summary.txt" 2>/dev/null)"
    arm "REPLAY regex: leg-3 criterion recorded" 0 "regex_done_bar_leg3_qualifier: (corpus pending)" 0 "$(cat "$rgx_state/summary.txt" 2>/dev/null)"

    # vhdl — THE headline row: the gate computes the qualified Provisional and the post-.2b
    # tracker row agrees, so the gate is GREEN stating the demoted truth.
    vhdl_state="$WORK/replay/vhdl_parser_family_status_gate"
    out="$(PGEN_VHDL_FAMILY_STATUS_STATE_DIR="$vhdl_state" \
           PGEN_VHDL_FAMILY_STATUS_EXISTING_FAMILY_CONTRACT_STATE_DIR="$SOTA/vhdl_parser_family_contract_gate" \
           PGEN_VHDL_FAMILY_STATUS_EXISTING_FORMAL_EXHAUSTIVE_CLOSURE_STATE_DIR="$SOTA/vhdl_parser_family_status_gate/work/vhdl_formal_exhaustive_closure_gate" \
           bash "$ROOT/rust/scripts/vhdl_parser_family_status_gate.sh" 2>&1)"; rc=$?
    arm "REPLAY vhdl: ALIGNED on Provisional, gate passes" 0 "✅ VHDL parser-family status gate passed." "$rc" "$out"
    arm "REPLAY vhdl: computed Provisional recorded" 0 "vhdl_status: Provisional (corpus pending)" 0 "$(cat "$vhdl_state/summary.txt" 2>/dev/null)"
    arm "REPLAY vhdl: json computed_status states the truth" 0 "Provisional (corpus pending)" 0 "$(jq -r '.families[0].computed_status' "$vhdl_state/summary.json" 2>/dev/null)"
    arm "REPLAY vhdl: alignment_ok recorded true" 0 "true" 0 "$(jq -r '.families[0].tracker_alignment_ok' "$vhdl_state/summary.json" 2>/dev/null)"

    # sv — two families: systemverilog stays Mostly Done (the cap only affects Done);
    # systemverilog_preprocessor computes Provisional and the post-.2b tracker row agrees.
    sv_state="$WORK/replay/sv_parser_family_status_gate"
    out="$(PGEN_SV_FAMILY_STATUS_STATE_DIR="$sv_state" \
           PGEN_SV_FAMILY_STATUS_EXISTING_SV_SYNTAX_CLOSURE_STATE_DIR="$SOTA/sv_parser_family_status_gate/work/sv_syntax_closure_gate" \
           PGEN_SV_FAMILY_STATUS_EXISTING_SV_PREPROCESSOR_SYNTAX_CLOSURE_STATE_DIR="$SOTA/sv_parser_family_status_gate/work/sv_preprocessor_syntax_closure_gate" \
           PGEN_SV_FAMILY_STATUS_EXISTING_SV_PARSER_AGGREGATE_STATE_DIR="$SOTA/sv_parser_aggregate_contract_gate" \
           PGEN_SV_FAMILY_STATUS_EXISTING_SV_STIMULI_QUALITY_STATE_DIR="$SOTA/sv_stimuli_quality_gate" \
           PGEN_SV_FAMILY_STATUS_EXISTING_SV_PREPROCESSOR_AGGREGATE_STATE_DIR="$SOTA/sv_preprocessor_aggregate_contract_gate" \
           PGEN_SV_FAMILY_STATUS_EXISTING_SV_PREPROCESSOR_REACHABILITY_STATE_DIR="$SOTA/sv_preprocessor_reachability_closure_gate" \
           PGEN_SV_FAMILY_STATUS_EXISTING_SV_PREPROCESSOR_FORMAL_EXHAUSTIVE_CLOSURE_STATE_DIR="$SOTA/sv_parser_family_status_gate/work/sv_preprocessor_formal_exhaustive_closure_gate" \
           PGEN_SV_FAMILY_STATUS_EXISTING_SV_PREPROCESSOR_QUALITY_STATE_DIR="$SOTA/sv_preprocessor_quality_gate" \
           PGEN_SV_FAMILY_STATUS_EXISTING_SV_SEMANTIC_SCOPE_CONTRACT_STATE_DIR="$SOTA/sv_parser_family_status_gate/work/sv_semantic_scope_contract_gate" \
           PGEN_SV_FAMILY_STATUS_EXISTING_SV_FORMAL_EXHAUSTIVE_CLOSURE_STATE_DIR="$SOTA/sv_parser_family_status_gate/work/sv_formal_exhaustive_closure_gate" \
           bash "$ROOT/rust/scripts/sv_parser_family_status_gate.sh" 2>&1)"; rc=$?
    arm "REPLAY sv: ALIGNED on both families, gate passes" 0 "✅ SV parser-family status gate passed." "$rc" "$out"
    arm "REPLAY sv: sv family stays Mostly Done, aligned" 0 "systemverilog_status: Mostly Done" 0 "$(cat "$sv_state/summary.txt" 2>/dev/null)"
    arm "REPLAY sv: svpp computed Provisional recorded" 0 "systemverilog_preprocessor_status: Provisional (corpus pending)" 0 "$(cat "$sv_state/summary.txt" 2>/dev/null)"
fi

echo
echo "=============================================================================="
echo "VERDICT: $pass arm(s) passed, $fail failed"
echo "=============================================================================="
[[ "$fail" -eq 0 ]] || exit 1
