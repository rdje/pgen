#!/usr/bin/env bash
# CI-PARITY-GATE-ROT.11 — probes for the stage-telemetry reader in rust/scripts/sota_exit_gate.sh
#
# WHAT IS UNDER TEST
#   The aggregate used to read the SV and VHDL stimuli telemetry out of each stage's PROSE LOG
#   (`summary_value_from_log`, 24 SV sites; `summary_value_from_log_or_txt`, log-first, 28 VHDL
#   sites). Each stimuli gate ends by `cat`-ing its own `summary.txt`, so the log carries that block
#   PLUS everything else the gate printed — `^key: value$` is ambiguous there and `tail -n 1` is
#   correct only while the summary stays the last thing printed. `.9` is that coupling breaking:
#   an appended pass emitted a differently-worded line and the gate published a superseded number
#   for two months.
#
#   The fix reads the stage's STRUCTURED `summary.txt` first (`summary_value_from_stage`) and adds a
#   statement-level tripwire (`assert_stage_summary_matches_log`) so a log that has drifted away from
#   the artifact is REPORTED rather than silently out-voted by a selector.
#
# HOW IT IS TESTED
#   The functions are EXTRACTED FROM THE REAL SCRIPT — both from the working tree (AFTER) and from
#   `git show HEAD:` (BEFORE) — so the probe never tests a hand-copied lookalike.
#   The fixture is a COPY of the real run-4 stage artifacts, doctored in the copy only: the real logs
#   are evidence and a diagnostic re-run must not overwrite them.
#
# CALIBRATION
#   The probe first re-derives two facts the leaf already measured (20 multi-match keys per stage,
#   0 torn keys). If they do not reproduce it prints MISCALIBRATED and refuses with no verdict —
#   a clean sweep and a blind sweep are indistinguishable without a positive control.
set -uo pipefail

# docs/tasks/artifacts/ci_parity_gate_rot/ is FOUR levels below the repo root.
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
GATE_REL="rust/scripts/sota_exit_gate.sh"
GATE="$ROOT/$GATE_REL"

SV_SUMMARY="$ROOT/rust/target/sota_exit_gate/work/sv_stimuli_quality_gate/summary.txt"
SV_LOG="$ROOT/rust/target/sota_exit_gate/logs/sv_stimuli_quality_gate.log"
VHDL_SUMMARY="$ROOT/rust/target/sota_exit_gate/work/vhdl_stimuli_quality_gate/summary.txt"
VHDL_LOG="$ROOT/rust/target/sota_exit_gate/logs/vhdl_stimuli_quality_gate.log"

pass=0
fail=0

say() { printf '%s\n' "$*"; }
ok() { pass=$((pass + 1)); printf '  ✅ %s\n' "$1"; }
no() { fail=$((fail + 1)); printf '  ❌ %s\n' "$1"; }

refuse() {
    printf 'REFUSED: %s\n' "$1" >&2
    printf 'No verdict is being offered.\n' >&2
    exit 2
}

for f in "$GATE" "$SV_SUMMARY" "$SV_LOG" "$VHDL_SUMMARY" "$VHDL_LOG"; do
    [[ -f "$f" ]] || refuse "required input is absent: ${f#"$ROOT"/} (run 4 artifacts are the fixture source; re-run the aggregate or point this probe at a preserved copy)"
done

WORK="$(mktemp -d "${TMPDIR:-/tmp}/ci-parity-11-probes.XXXXXX")"
trap 'rm -rf "$WORK"' EXIT

# Extract the named top-level shell functions from a script into a sourceable file.
# Functions in this file are declared at column 0 as `name() {` and closed by `}` at column 0.
extract_funcs() {
    local src="$1"
    shift
    local out="$1"
    shift
    : >"$out"
    local name
    for name in "$@"; do
        awk -v want="$name" '
            $0 == want "() {" { inside = 1 }
            inside { print }
            inside && $0 == "}" { inside = 0 }
        ' "$src" >>"$out"
    done
}

git -C "$ROOT" show "HEAD:$GATE_REL" >"$WORK/gate_before.sh" 2>/dev/null \
    || refuse "cannot read HEAD:$GATE_REL"

extract_funcs "$WORK/gate_before.sh" "$WORK/before.sh" \
    summary_value_from_log summary_value_from_txt summary_value_from_log_or_txt
extract_funcs "$GATE" "$WORK/after.sh" \
    summary_value_from_log summary_value_from_txt summary_value_from_stage assert_stage_summary_matches_log

for want in summary_value_from_log summary_value_from_log_or_txt; do
    grep -q "^${want}() {" "$WORK/before.sh" || refuse "BEFORE extraction missed ${want} — the probe is not exercising the retired reader"
done
for want in summary_value_from_stage assert_stage_summary_matches_log; do
    grep -q "^${want}() {" "$WORK/after.sh" || refuse "AFTER extraction missed ${want} — the probe is not exercising the shipped reader"
done

# --------------------------------------------------------------------------------------------------
# Calibration — re-derive the two facts the leaf measured. A detector with no positive control
# cannot tell "the hazard is absent" from "I am looking at the wrong tree".
# --------------------------------------------------------------------------------------------------
say "=== calibration ==="
miscalibrated=0

count_multi_and_torn() {
    local summary="$1" log="$2"
    local multi=0 torn=0 key sval lval n
    while IFS= read -r key; do
        n="$(sed -nE "s/^${key}: (.*)$/\1/p" "$log" | wc -l | tr -d ' ')"
        sval="$(sed -nE "s/^${key}: (.*)$/\1/p" "$summary" | tail -n 1)"
        lval="$(sed -nE "s/^${key}: (.*)$/\1/p" "$log" | tail -n 1)"
        [[ "$n" -gt 1 ]] && multi=$((multi + 1))
        [[ -n "$lval" && "$lval" != "$sval" ]] && torn=$((torn + 1))
    done < <(sed -nE 's/^([a-z0-9_]+): .*$/\1/p' "$summary" | sort -u)
    printf '%s %s\n' "$multi" "$torn"
}

read -r sv_multi sv_torn < <(count_multi_and_torn "$SV_SUMMARY" "$SV_LOG")
read -r vhdl_multi vhdl_torn < <(count_multi_and_torn "$VHDL_SUMMARY" "$VHDL_LOG")

check_cal() {
    local label="$1" got="$2" want="$3"
    if [[ "$got" == "$want" ]]; then
        printf '  CAL %-46s %s (expected %s) ✓\n' "$label" "$got" "$want"
    else
        printf '  CAL %-46s %s (expected %s) ✗\n' "$label" "$got" "$want"
        miscalibrated=$((miscalibrated + 1))
    fi
}

check_cal "sv keys matching >1 log line" "$sv_multi" 20
check_cal "vhdl keys matching >1 log line" "$vhdl_multi" 20
check_cal "sv torn keys (artifact vs log)" "$sv_torn" 0
check_cal "vhdl torn keys (artifact vs log)" "$vhdl_torn" 0

if [[ "$miscalibrated" -gt 0 ]]; then
    say ""
    say "MISCALIBRATED — ${miscalibrated} pinned fact(s) did not reproduce."
    refuse "the fixture no longer matches the measurements this leaf was decided on"
fi
say "  calibration OK — the ambiguity is real (20 keys per stage) and today's logs are not torn."
say ""

# --------------------------------------------------------------------------------------------------
# Fixture — a COPY of the real artifacts, with an appended stage doctored into the log copy only.
# This is `.9`'s defect shape: a later pass prints the same key with a newer, different value.
# --------------------------------------------------------------------------------------------------
FIX_SUMMARY="$WORK/summary.txt"
FIX_LOG="$WORK/stage.log"
cp "$SV_SUMMARY" "$FIX_SUMMARY"
cp "$SV_LOG" "$FIX_LOG"

TORN_KEY="closed_loop_initial_targets_total"
TRUE_VALUE="$(sed -nE "s/^${TORN_KEY}: (.*)$/\1/p" "$FIX_SUMMARY" | tail -n 1)"
[[ -n "$TRUE_VALUE" ]] || refuse "fixture key ${TORN_KEY} carries no value in the summary artifact"
APPENDED_VALUE="$((TRUE_VALUE + 4242))"
{
    say "==> appended post-summary stage (probe fixture)"
    say "${TORN_KEY}: ${APPENDED_VALUE}"
} >>"$FIX_LOG"

say "=== fixture ==="
say "  key            : ${TORN_KEY}"
say "  artifact value : ${TRUE_VALUE}   (the truth: what the stage actually resolved)"
say "  appended value : ${APPENDED_VALUE}   (what a stage appended after the summary would print)"
say ""

# --------------------------------------------------------------------------------------------------
say "=== BEFORE (the retired readers, extracted from HEAD) ==="
before_log_value="$(bash -c "source '$WORK/before.sh'; summary_value_from_log '$TORN_KEY' '$FIX_LOG'")"
if [[ "$before_log_value" == "$APPENDED_VALUE" ]]; then
    ok "BEFORE-1  summary_value_from_log returns the APPENDED value ${before_log_value} — the defect reproduces"
else
    no "BEFORE-1  expected the retired reader to return the appended ${APPENDED_VALUE}, got '${before_log_value}' — the fixture does not reproduce the defect"
fi

before_or_txt_value="$(bash -c "source '$WORK/before.sh'; summary_value_from_log_or_txt '$TORN_KEY' '$FIX_LOG' '$FIX_SUMMARY'")"
if [[ "$before_or_txt_value" == "$APPENDED_VALUE" ]]; then
    ok "BEFORE-2  the VHDL reader had the artifact AND still returned the log's ${before_or_txt_value} — log-first precedence was the bug"
else
    no "BEFORE-2  expected log-first precedence to return ${APPENDED_VALUE}, got '${before_or_txt_value}'"
fi
say ""

# --------------------------------------------------------------------------------------------------
say "=== AFTER (the shipped reader, extracted from the working tree) ==="
after_value="$(bash -c "source '$WORK/after.sh'; summary_value_from_stage '$TORN_KEY' '$FIX_SUMMARY' '$FIX_LOG'")"
if [[ "$after_value" == "$TRUE_VALUE" ]]; then
    ok "AFTER-1   summary_value_from_stage returns the ARTIFACT value ${after_value} — the appended line cannot corrupt it"
else
    no "AFTER-1   expected the artifact value ${TRUE_VALUE}, got '${after_value}'"
fi

# RED — the tripwire must FIRE on the doctored log and NAME the key.
red_out="$(bash -c "source '$WORK/after.sh'; assert_stage_summary_matches_log 'probe_stage' '$FIX_SUMMARY' '$FIX_LOG'" 2>&1)"
red_rc=$?
if [[ "$red_rc" -ne 0 ]] && printf '%s' "$red_out" | grep -q "$TORN_KEY"; then
    ok "RED-1     the tripwire exits ${red_rc} and names '${TORN_KEY}' — drift is reported, not out-voted"
else
    no "RED-1     expected a nonzero exit naming ${TORN_KEY}; rc=${red_rc} out=$(printf '%s' "$red_out" | head -n 2 | tr '\n' ' ')"
fi
if printf '%s' "$red_out" | grep -q "1 key(s) disagree"; then
    ok "RED-1b    it reports exactly 1 disagreeing key — it counts offenders, it does not stop at the first"
else
    no "RED-1b    expected a '1 key(s) disagree' tally; got: $(printf '%s' "$red_out" | tail -n 3 | tr '\n' ' ')"
fi

# GREEN — the tripwire must PASS on the real, undoctored artifacts of both stages. Without this arm
# RED-1 would be satisfied by a check that fails on everything.
for stage in "sv_stimuli_quality_gate:$SV_SUMMARY:$SV_LOG" "vhdl_stimuli_quality_gate:$VHDL_SUMMARY:$VHDL_LOG"; do
    name="${stage%%:*}"
    rest="${stage#*:}"
    sfile="${rest%%:*}"
    lfile="${rest#*:}"
    out="$(bash -c "source '$WORK/after.sh'; assert_stage_summary_matches_log '$name' '$sfile' '$lfile'" 2>&1)"
    rc=$?
    if [[ "$rc" -eq 0 && -z "$out" ]]; then
        ok "GREEN-1   ${name}: the tripwire passes silently on the real run-4 artifacts"
    else
        no "GREEN-1   ${name}: expected a silent pass, rc=${rc} out=$(printf '%s' "$out" | head -n 2 | tr '\n' ' ')"
    fi
done
say ""

# --------------------------------------------------------------------------------------------------
say "=== controls — the failure paths must behave exactly as they did before ==="

# CTRL-1: the artifact is absent (the stage died before writing it) ⇒ the log fallback must survive,
# otherwise this change would blind the aggregate's failure reporting.
ctrl1="$(bash -c "source '$WORK/after.sh'; summary_value_from_stage '$TORN_KEY' '$WORK/nonexistent-summary.txt' '$FIX_LOG'")"
if [[ "$ctrl1" == "$APPENDED_VALUE" ]]; then
    ok "CTRL-1    with no artifact the reader still returns the log value (${ctrl1}) — failure-path reads unchanged"
else
    no "CTRL-1    expected the log fallback to return ${APPENDED_VALUE}, got '${ctrl1}'"
fi

# CTRL-2: a key in neither source yields empty, so the caller's `${x:-unknown}` default still fires.
ctrl2="$(bash -c "source '$WORK/after.sh'; summary_value_from_stage 'a_key_no_stage_emits' '$FIX_SUMMARY' '$FIX_LOG'")"
if [[ -z "$ctrl2" ]]; then
    ok "CTRL-2    an absent key returns empty — the callers' ':-unknown' defaults still apply"
else
    no "CTRL-2    expected empty for an absent key, got '${ctrl2}'"
fi

# CTRL-3: the tripwire must be a NO-OP when a file is missing. A stage that died is already reported
# by run_check; turning that into a torn-read failure would misattribute the cause (.14's lesson).
ctrl3_out="$(bash -c "source '$WORK/after.sh'; assert_stage_summary_matches_log 'probe_stage' '$WORK/nonexistent-summary.txt' '$FIX_LOG'" 2>&1)"
ctrl3_rc=$?
if [[ "$ctrl3_rc" -eq 0 && -z "$ctrl3_out" ]]; then
    ok "CTRL-3    with a missing artifact the tripwire is a silent no-op — it cannot mask a dead stage"
else
    no "CTRL-3    expected a silent no-op, rc=${ctrl3_rc} out=$(printf '%s' "$ctrl3_out" | head -n 1)"
fi

# CTRL-4: the artifact wins even when the log agrees — proves AFTER-1 is not passing by coincidence
# because the two happened to match.
clean_log="$WORK/clean.log"
cp "$SV_LOG" "$clean_log"
ctrl4="$(bash -c "source '$WORK/after.sh'; summary_value_from_stage '$TORN_KEY' '$FIX_SUMMARY' '$clean_log'")"
if [[ "$ctrl4" == "$TRUE_VALUE" ]]; then
    ok "CTRL-4    on an undoctored log the reader returns the same ${ctrl4} — the fix is not a behaviour change on a healthy run"
else
    no "CTRL-4    expected ${TRUE_VALUE} on a clean log, got '${ctrl4}'"
fi

# CTRL-5: the retired log-first VHDL reader is gone from the script, not merely unused.
if grep -q 'summary_value_from_log_or_txt' "$GATE"; then
    no "CTRL-5    summary_value_from_log_or_txt still exists in ${GATE_REL} — a log-first reader left in the file is an invitation to reuse it"
else
    ok "CTRL-5    summary_value_from_log_or_txt is removed from ${GATE_REL}, not just unreferenced"
fi

# CTRL-6: no stage read scrapes the prose log directly any more; the only two remaining uses of the
# raw log reader are inside the artifact-first reader and the tripwire.
raw_uses="$(grep -c 'summary_value_from_log "' "$GATE")"
if [[ "$raw_uses" -eq 2 ]]; then
    ok "CTRL-6    exactly 2 raw-log reads remain (the fallback inside the reader + the tripwire's comparison)"
else
    no "CTRL-6    expected 2 raw-log reads, found ${raw_uses} — a call site was missed or a new one appeared"
fi

say ""
say "=== REUSE mode — the live defect, not a hypothetical one ==="

# ⭐ This is what makes the leaf a repair rather than hardening. When the aggregate is pointed at an
# existing stage state dir (`PGEN_SOTA_EXISTING_SV_STIMULI_QUALITY_STATE_DIR`), it does not re-run the
# stage — it calls `run_check` with a `bash -lc "test -s .../summary.txt"` probe, and `run_check`
# redirects that probe's output over `logs/sv_stimuli_quality_gate.log`. The stage log the reader was
# aimed at therefore becomes a 33-byte login banner, while the reused state dir carries every value in
# its `summary.txt`. Measured end-to-end: 22 SV telemetry values published as `unknown`.
REUSE_LOG="$WORK/reuse.log"
printf 'Bash profile loaded successfully\n' >"$REUSE_LOG"

reuse_before="$(bash -c "source '$WORK/before.sh'; summary_value_from_log '$TORN_KEY' '$REUSE_LOG'")"
if [[ -z "$reuse_before" ]]; then
    ok "REUSE-1   the retired reader finds nothing in the reuse-branch log ⇒ the aggregate published 'unknown'"
else
    no "REUSE-1   expected the retired reader to come up empty, got '${reuse_before}'"
fi

reuse_after="$(bash -c "source '$WORK/after.sh'; summary_value_from_stage '$TORN_KEY' '$SV_SUMMARY' '$REUSE_LOG'")"
if [[ "$reuse_after" == "$TRUE_VALUE" ]]; then
    ok "REUSE-2   the shipped reader returns ${reuse_after} from the reused state dir's own artifact"
else
    no "REUSE-2   expected ${TRUE_VALUE} from the reused artifact, got '${reuse_after}'"
fi

# The VHDL reader was accidentally immune — it had the artifact as a FALLBACK, so an empty log fell
# through to it. That asymmetry between two stages of the same aggregate IS the defect this leaf closes:
# one family's telemetry was right by luck, the other's was silently `unknown`.
reuse_vhdl_before="$(bash -c "source '$WORK/before.sh'; summary_value_from_log_or_txt '$TORN_KEY' '$REUSE_LOG' '$SV_SUMMARY'")"
if [[ "$reuse_vhdl_before" == "$TRUE_VALUE" ]]; then
    ok "REUSE-3   the retired VHDL reader survived reuse mode via its artifact fallback — SV had none, hence the asymmetry"
else
    no "REUSE-3   expected the log-or-txt fallback to yield ${TRUE_VALUE}, got '${reuse_vhdl_before}'"
fi

say ""
say "=== no-regression — the aggregate must publish the SAME numbers on real evidence ==="

# The fix is only safe if it does not silently restate run 4's telemetry. For every key the aggregate
# actually reads (DERIVED from the shipped call sites, not hand-listed, so a missed site shows up as a
# count change rather than a silent omission), the retired reader and the shipped reader must agree on
# the real run-4 artifacts.
noreg_keys_sv="$(grep -oE 'summary_value_from_stage "[a-z0-9_]+" "\$SV_STIMULI_QUALITY_STAGE_SUMMARY_TXT"' "$GATE" | sed -E 's/.*"([a-z0-9_]+)".*/\1/' | sort -u)"
noreg_keys_vhdl="$(grep -oE 'summary_value_from_stage "[a-z0-9_]+" "\$VHDL_STIMULI_QUALITY_STAGE_SUMMARY_TXT"' "$GATE" | sed -E 's/.*"([a-z0-9_]+)".*/\1/' | sort -u)"

compare_readers() {
    local label="$1" summary="$2" log="$3" keys="$4" expect_count="$5"
    local n=0 differ=0 key b a
    while IFS= read -r key; do
        [[ -n "$key" ]] || continue
        n=$((n + 1))
        b="$(bash -c "source '$WORK/before.sh'; summary_value_from_log '$key' '$log'")"
        a="$(bash -c "source '$WORK/after.sh'; summary_value_from_stage '$key' '$summary' '$log'")"
        if [[ "$b" != "$a" ]]; then
            differ=$((differ + 1))
            printf '     %-64s before=%s after=%s\n' "$key" "$b" "$a"
        fi
    done <<<"$keys"
    if [[ "$n" -ne "$expect_count" ]]; then
        no "NOREG     ${label}: derived ${n} read keys, expected ${expect_count} — a call site was added or lost"
        return
    fi
    if [[ "$differ" -eq 0 ]]; then
        ok "NOREG     ${label}: all ${n} published values identical before→after on the run-4 artifacts"
    else
        no "NOREG     ${label}: ${differ} of ${n} published values changed"
    fi
}

# SV expects 26, not the 24 helper sites the census counted: the leaf also converted the two dedicated
# METRIC `sed` sites (`parse_full_pass_ratio_percent`, `diff_mismatch_count`), which are the reads that
# feed the aggregate's published SV parse-full ratio and differential mismatch count when the stage's
# JSON reports are absent. The count is pinned so a future edit cannot quietly drop a read site.
compare_readers "sv_stimuli_quality_gate" "$SV_SUMMARY" "$SV_LOG" "$noreg_keys_sv" 26
compare_readers "vhdl_stimuli_quality_gate" "$VHDL_SUMMARY" "$VHDL_LOG" "$noreg_keys_vhdl" 28

say ""
say "=== probes: ${pass} pass / ${fail} fail ==="
[[ "$fail" -eq 0 ]] || exit 1
exit 0
