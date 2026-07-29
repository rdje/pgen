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

# --- CTRL-9 / RED-L: the .5b ledger criterion (family_open_ledger_entries) ----------------------
run_ledger() {
    bash -c '
        set -euo pipefail
        source "'"$LIB"'"
        family_open_ledger_entries "'"$1"'"
        echo "rows=$DONE_BAR_LEDGER_ROWS open=$DONE_BAR_LEDGER_OPEN_COUNT ids=$DONE_BAR_LEDGER_OPEN_IDS"
    ' 2>&1
}
out="$(run_ledger vhdl)"; rc=$?
arm "CTRL-9 real ledger: vhdl 2 rows, 0 open" 0 "rows=2 open=0 ids=-" "$rc" "$out"

# a synthetic ledger with one OPEN row for fakefam — the criterion must count and NAME it
cat >"$WORK/ledger_open.md" <<'EOF'
# synthetic ledger fixture
## State Meanings
- `Reported`
  - received
- `Released`
  - closed out
- `Rejected`
  - closed out
## Rows
| ID | family | a | b | c | d | State | e | f | notes |
|---|---|---|---|---|---|---|---|---|---|
| `FAKE-0001` | `fakefam` / `fake_default` | `1.0` | `1.0` | `X` | `2026-01-01` | `Reported` | `-` | `-` | an open one |
| `FAKE-0002` | `fakefam` / `fake_default` | `1.0` | `1.0` | `X` | `2026-01-01` | `Released` | `-` | `-` | a closed one |
EOF
out="$(PGEN_FAMILY_STATUS_LEDGER="$WORK/ledger_open.md" run_ledger fakefam)"; rc=$?
arm "RED-L1 open ledger row counted and NAMED" 0 "rows=2 open=1 ids=FAKE-0001" "$rc" "$out"

# a ledger with no State Meanings section — classification against a guessed vocabulary REFUSES
printf '# a ledger with no vocabulary\n| `X-1` | `fakefam` | `Reported` |\n' >"$WORK/ledger_no_vocab.md"
out="$(PGEN_FAMILY_STATUS_LEDGER="$WORK/ledger_no_vocab.md" run_ledger fakefam)"; rc=$?
arm "RED-L2 no State Meanings: REFUSE" 2 "state vocabulary cannot be derived" "$rc" "$out"

# --- CTRL-S / RED-S: the .5e silent-success criterion (family_reachable_silent_success) -----------
#
# Driven through PGEN_FAMILY_STATUS_SENTINEL_SUMMARY_JSON so each arm costs milliseconds instead of
# a ~40 s sweep. The PRODUCE-IT-YOURSELF path (no seam) is exercised for real by the gate replays
# below, which is where it has to hold.
run_sentinel() {
    bash -c '
        set -euo pipefail
        source "'"$LIB"'"
        family_reachable_silent_success "'"$1"'" "'"$WORK"'"
        echo "samples=$DONE_BAR_SENTINEL_SAMPLES reached=$DONE_BAR_SENTINEL_REACHED_SAMPLES sentinels=$DONE_BAR_SENTINEL_REACHED_TOTAL codegen=$DONE_BAR_SENTINEL_CODEGEN_VIOLATIONS"
        echo "detail=$DONE_BAR_SENTINEL_DETAIL"
    ' 2>&1
}
# mk_sentinel — a synthetic sentinel artifact. $1 = jq expression applied to a clean 2-family base.
mk_sentinel() {
    jq -n '{
        gate: "silent_success_sentinel_gate", status: "pass", exit: 0,
        report: {
          static: {status: "OK", parsers_scanned: 11, codegen_placeholder_violations: {},
                   runtime_fallback_arms_present: {"<invalid_sequence_access>": 3016}},
          families: [
            {grammar: "vhdl", status: "OK", samples: 25, parsed_ok: 25, sentinel_totals: {}, sentinel_samples: 0},
            {grammar: "systemverilog", status: "OK", samples: 25, parsed_ok: 25, sentinel_totals: {}, sentinel_samples: 0},
            {grammar: "systemverilog_preprocessor", status: "OK", samples: 25, parsed_ok: 25, sentinel_totals: {}, sentinel_samples: 0}
          ]
        }}' | jq "$1"
}

# CTRL-S1 — the REAL shipped artifact, if this machine has one: the criterion must reproduce the
# gate's own published per-family numbers. Ground truth for the derivation itself.
real_sentinel="$ROOT/rust/target/silent_success_sentinel_gate/summary.json"
if [[ -s "$real_sentinel" ]] && [[ "$(jq -r '.report | type' "$real_sentinel")" == "object" ]]; then
    want="samples=$(jq -r '[.report.families[]|select(.grammar=="vhdl")][0].samples' "$real_sentinel") reached=$(jq -r '[.report.families[]|select(.grammar=="vhdl")][0].sentinel_samples' "$real_sentinel")"
    out="$(PGEN_FAMILY_STATUS_SENTINEL_SUMMARY_JSON="$real_sentinel" run_sentinel vhdl)"; rc=$?
    arm "CTRL-S1 real artifact: vhdl numbers reproduce" 0 "$want" "$rc" "$out"
else
    echo "   ⚠️  CTRL-S1 skipped: no shipped silent_success_sentinel_gate/summary.json on this machine"
fi

# RED-S1 — a REACHED sentinel must be counted AND named. This is the arm that matters: it is the
# whole point of the criterion, and a 0 here would mean the tier can never fall for this defect.
mk_sentinel '.report.families[0].sentinel_samples = 2
             | .report.families[0].sentinel_totals = {"<invalid_sequence_access>": 3}' >"$WORK/sentinel_reached.json"
out="$(PGEN_FAMILY_STATUS_SENTINEL_SUMMARY_JSON="$WORK/sentinel_reached.json" run_sentinel vhdl)"; rc=$?
arm "RED-S1 reached sentinel counted" 0 "samples=25 reached=2 sentinels=3 codegen=0" "$rc" "$out"
arm "RED-S1 reached sentinel NAMED in the detail" 0 "REACHABLE on this family's own proof surface" 0 "$out"

# CTRL-S2 — THE ATTRIBUTION ARM. A codegen placeholder in systemverilog_preprocessor_parser.rs must
# be charged to svpp and NOT to systemverilog: a substring match would hand it to both, demoting a
# family for another family's defect.
mk_sentinel '.report.static.codegen_placeholder_violations = {"<array_access>": {"systemverilog_preprocessor_parser.rs": 4}}' \
    >"$WORK/sentinel_codegen.json"
out="$(PGEN_FAMILY_STATUS_SENTINEL_SUMMARY_JSON="$WORK/sentinel_codegen.json" run_sentinel systemverilog_preprocessor)"; rc=$?
arm "CTRL-S2a codegen violation charged to svpp" 0 "reached=0 sentinels=0 codegen=4" "$rc" "$out"
out="$(PGEN_FAMILY_STATUS_SENTINEL_SUMMARY_JSON="$WORK/sentinel_codegen.json" run_sentinel systemverilog)"; rc=$?
arm "CTRL-S2b sv NOT charged for svpp's violation" 0 "reached=0 sentinels=0 codegen=0" "$rc" "$out"

# RED-S2 — a family ABSENT from the swept roster REFUSES. Dropping vhdl from the sentinel contract
# must never read as "0 sentinels reached" — the vacuity trap, one level up.
mk_sentinel 'del(.report.families[0])' >"$WORK/sentinel_no_family.json"
out="$(PGEN_FAMILY_STATUS_SENTINEL_SUMMARY_JSON="$WORK/sentinel_no_family.json" run_sentinel vhdl)"; rc=$?
arm "RED-S2 family absent from roster: REFUSE" 2 "ABSENT from the silent-success contract roster" "$rc" "$out"

# RED-S3 — a REFUSAL/MISCALIBRATION summary carries no report and cannot answer a per-family question.
jq -n '{gate:"silent_success_sentinel_gate",status:"miscalibrated",exit:2}' >"$WORK/sentinel_refuse.json"
out="$(PGEN_FAMILY_STATUS_SENTINEL_SUMMARY_JSON="$WORK/sentinel_refuse.json" run_sentinel vhdl)"; rc=$?
arm "RED-S3 miscalibrated artifact: REFUSE" 2 "carries no sweep report" "$rc" "$out"

# RED-S4 — a family row the sweep itself could not measure.
mk_sentinel '.report.families[0].status = "REFUSE_ZERO_SAMPLES"' >"$WORK/sentinel_row_refuse.json"
out="$(PGEN_FAMILY_STATUS_SENTINEL_SUMMARY_JSON="$WORK/sentinel_row_refuse.json" run_sentinel vhdl)"; rc=$?
arm "RED-S4 unmeasured family row: REFUSE" 2 "could not measure family vhdl" "$rc" "$out"

# RED-S5 — a green ZERO over ZERO samples is never a pass.
mk_sentinel '.report.families[0].samples = 0' >"$WORK/sentinel_zero_samples.json"
out="$(PGEN_FAMILY_STATUS_SENTINEL_SUMMARY_JSON="$WORK/sentinel_zero_samples.json" run_sentinel vhdl)"; rc=$?
arm "RED-S5 zero samples swept: REFUSE" 2 "swept with ZERO samples" "$rc" "$out"

# RED-S6 — a missing artifact refuses rather than scoring the comfortable answer.
out="$(PGEN_FAMILY_STATUS_SENTINEL_SUMMARY_JSON="$WORK/does_not_exist.json" run_sentinel vhdl)"; rc=$?
arm "RED-S6 missing artifact: REFUSE" 2 "missing or empty" "$rc" "$out"

# CTRL-S3 — the honest bound must travel with the clean verdict, not be implied away.
mk_sentinel '.' >"$WORK/sentinel_clean.json"
out="$(PGEN_FAMILY_STATUS_SENTINEL_SUMMARY_JSON="$WORK/sentinel_clean.json" run_sentinel vhdl)"; rc=$?
arm "CTRL-S3 clean verdict carries the honest bound" 0 "NOT unreachability" "$rc" "$out"

# CTRL-S4 / RED-S7 — the PRODUCE-IT-YOURSELF path, driven through a FAKE ROOT whose sentinel gate is
# a STUB that records every invocation. The lib derives its root from ${BASH_SOURCE[0]} (correct —
# the repo-root relative-path policy), so a fake root is the only way to observe the invocation
# count without a 40 s real sweep. CTRL-S4 is the CACHE arm: the sv gate computes TWO families and
# must sweep ONCE, or the aggregate pays for the same evidence twice.
FAKE="$WORK/fake_root"
mkdir -p "$FAKE/rust/scripts/lib"
cp "$LIB" "$FAKE/rust/scripts/lib/parser_family_status_bar.sh"
mk_stub_gate() { # mk_stub_gate EXIT_CODE — writes a stub that logs each run and exits EXIT_CODE
    cat >"$FAKE/rust/scripts/silent_success_sentinel_gate.sh" <<STUB
#!/usr/bin/env bash
# stub sentinel gate: records the invocation, emits a well-formed artifact, exits $1
echo run >>"$FAKE/invocations.txt"
mkdir -p "\$PGEN_SILENT_SUCCESS_STATE_DIR"
cat >"\$PGEN_SILENT_SUCCESS_STATE_DIR/summary.json" <<'JSON'
$(cat "$WORK/sentinel_clean.json")
JSON
exit $1
STUB
}
two_families() {
    bash -c '
        set -euo pipefail
        source "'"$FAKE"'/rust/scripts/lib/parser_family_status_bar.sh"
        family_reachable_silent_success "systemverilog" "'"$WORK"'/twofam"
        echo "first=$DONE_BAR_SENTINEL_SAMPLES"
        family_reachable_silent_success "systemverilog_preprocessor" "'"$WORK"'/twofam"
        echo "second=$DONE_BAR_SENTINEL_SAMPLES"
    ' 2>&1
}
mkdir -p "$WORK/twofam"
: >"$FAKE/invocations.txt"
mk_stub_gate 0
out="$(two_families)"; rc=$?
arm "CTRL-S4a two families, both answered" 0 "first=25" "$rc" "$out"
arm "CTRL-S4b ONE sweep for both (per-process cache)" 0 "1" 0 "$(wc -l <"$FAKE/invocations.txt" | tr -d ' ')"

# RED-S7 — the gate REFUSING (exit 2) makes the criterion unjudgeable ⇒ the status gate REFUSES too.
# ⛔ Deliberately distinguished from exit 1 (RED-S8): exit 1 is a real VERDICT this criterion reads
# per family, so a defect in one family must not make another family unjudgeable.
: >"$FAKE/invocations.txt"
mk_stub_gate 2
out="$(two_families)"; rc=$?
arm "RED-S7 gate REFUSES => criterion REFUSES" 2 "REFUSE/MISCALIBRATED" "$rc" "$out"

# RED-S8 — the gate's exit 1 is a VERDICT, not a refusal: the per-family rows are still read.
: >"$FAKE/invocations.txt"
mk_stub_gate 1
out="$(two_families)"; rc=$?
arm "RED-S8 gate exit 1 is a verdict, still judged" 0 "second=25" "$rc" "$out"

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
# DONE-BAR.5e: the replays now RUN the silent-success sweep (the produce-it-yourself path), which
# needs the two debug binaries and the untracked generated/ tree. Say so and skip rather than
# reporting a red the machine caused.
for b in "$ROOT/rust/target/debug/ast_pipeline" "$ROOT/rust/target/debug/parseability_probe"; do
    if [[ ! -x "$b" ]]; then
        echo "   ⚠️  replay input missing: $b (needed by the .5e sentinel sweep)"
        missing=$((missing + 1))
    fi
done
if ! compgen -G "$ROOT/generated/*.rs" >/dev/null; then
    echo "   ⚠️  replay input missing: generated/*.rs (needed by the .5e sentinel sweep)"
    missing=$((missing + 1))
fi
if [[ "$missing" -gt 0 ]]; then
    echo "   ⚠️  SKIPPING the gate replays: $missing run-3 artifact(s)/input(s) unavailable on this machine."
    echo "      The helper arms above still verify the .2a/.5b/.5e logic; re-run after an aggregate run."
else
    echo "   (each replay runs a real ~40 s silent-success sweep — the .5e produce-it-yourself path)"
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
    arm "REPLAY vhdl: ledger criterion recorded (0 open)" 0 "vhdl_ledger_open_entries_zero: true" 0 "$(cat "$vhdl_state/summary.txt" 2>/dev/null)"
    arm "REPLAY vhdl: .5e criterion recorded (self-produced)" 0 "vhdl_no_reachable_silent_success: true" 0 "$(cat "$vhdl_state/summary.txt" 2>/dev/null)"
    arm "REPLAY vhdl: .5e evidence is FRESH, not ambient" 0 "true" 0 "$([[ -s "$vhdl_state/silent_success_sentinel_gate/summary.json" ]] && echo true)"
    arm "REPLAY vhdl: criteria total moved 12 -> 13" 0 "13" 0 "$(jq -r '.families[0].closure_criteria_total_count' "$vhdl_state/summary.json" 2>/dev/null)"

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
    arm "REPLAY sv: ledger criterion recorded (0 open)" 0 "systemverilog_preprocessor_ledger_open_entries_zero: true" 0 "$(cat "$sv_state/summary.txt" 2>/dev/null)"
    arm "REPLAY sv: .5e criterion recorded for sv" 0 "systemverilog_no_reachable_silent_success: true" 0 "$(cat "$sv_state/summary.txt" 2>/dev/null)"
    arm "REPLAY sv: .5e criterion recorded for svpp" 0 "systemverilog_preprocessor_no_reachable_silent_success: true" 0 "$(cat "$sv_state/summary.txt" 2>/dev/null)"
    # The cache itself is proved decisively by CTRL-S4b above (a stub gate counting invocations);
    # here we only confirm the real gate produced its own evidence inside this run's state dir.
    arm "REPLAY sv: .5e evidence is FRESH, not ambient" 0 "true" 0 "$([[ -s "$sv_state/silent_success_sentinel_gate/summary.json" ]] && echo true)"
    arm "REPLAY regex: criteria total moved 10 -> 11" 0 "11" 0 "$(jq -r '.families[0].closure_criteria_total_count' "$rgx_state/summary.json" 2>/dev/null)"
    arm "REPLAY sv: criteria totals moved 9 -> 10 / 14 -> 15" 0 "10 15" 0 "$(jq -r '[.families[].closure_criteria_total_count] | join(" ")' "$sv_state/summary.json" 2>/dev/null)"
fi

echo
echo "=============================================================================="
echo "VERDICT: $pass arm(s) passed, $fail failed"
echo "=============================================================================="
[[ "$fail" -eq 0 ]] || exit 1
