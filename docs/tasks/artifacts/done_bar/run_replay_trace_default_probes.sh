#!/usr/bin/env bash
# run_replay_trace_default_probes.sh — prove the `DONE-BAR.5f` replay-trace default actually moved,
# that the override still works, and that the new summary key cannot collide with the log scrapers.
#
# ⭐ The default-resolution line and the note text are EXTRACTED FROM THE LIVE GATE, never re-typed,
# so this driver cannot test a rule the gate does not apply (the `.5e` / `CI-PARITY-GATE-ROT.7`
# convention). The BEFORE arm replays the RETIRED default out of `git show HEAD:` — the before→after
# is replayed, not described.
#
# Every arm asserts BOTH the observable value AND the message shape (CI-PARITY-GATE-ROT.4's rule:
# pass/fail alone hides arms that reach the right verdict for the wrong reason).
#
#   bash docs/tasks/artifacts/done_bar/run_replay_trace_default_probes.sh
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
cd "$ROOT"

GATE="rust/scripts/sv_stimuli_quality_gate.sh"
WORK="$ROOT/rust/target/done_bar_5f/probe"
rm -rf "$WORK"; mkdir -p "$WORK"

pass=0
fail=0

check() {
    local label="$1" got="$2" want="$3"
    if [[ "$got" == "$want" ]]; then
        printf '  ✅ %-58s %s\n' "$label" "$got"
        pass=$((pass + 1))
    else
        printf '  ⛔ %-58s got=%q want=%q\n' "$label" "$got" "$want"
        fail=$((fail + 1))
    fi
}

check_substr() {
    local label="$1" got="$2" want="$3"
    if [[ "$got" == *"$want"* ]]; then
        printf '  ✅ %-58s contains %q\n' "$label" "$want"
        pass=$((pass + 1))
    else
        printf '  ⛔ %-58s missing %q\n' "$label" "$want"
        printf '     got: %s\n' "$got"
        fail=$((fail + 1))
    fi
}

# Extract the LIVE gate's default-resolution + note block (the assignment plus the if/else that
# derives the note) into a sourceable fragment. Extracting rather than re-typing is the point.
extract_resolution() {
    local src="$1" dest="$2"
    # Stop at the first blank line OR the block's closing `fi`, whichever comes first: the LIVE
    # form is assignment + if/else/fi with no blank line inside, while the RETIRED form is the bare
    # assignment followed by a blank line. Anchoring on `fi` alone would run the retired extraction
    # off the end of the file and into unrelated `RUST_DIR` assignments.
    awk '
        /^REPLAY_TRACE_VERBOSITY=/ { emit = 1 }
        emit && /^[[:space:]]*$/ { exit }
        emit { print }
        emit && /^fi$/ { exit }
    ' "$src" > "$dest"
    [[ -s "$dest" ]]
}

resolve() {
    # $1 = fragment path; remaining args = env assignments
    local frag="$1"; shift
    env "$@" bash -c '
        set -u
        REPLAY_TRACE_VERBOSITY_NOTE=""
        . "$1"
        printf "%s\n" "$REPLAY_TRACE_VERBOSITY"
        printf "%s\n" "${REPLAY_TRACE_VERBOSITY_NOTE:-<none>}"
    ' _ "$frag"
}

echo "=============================================================================="
echo "DONE-BAR.5f — replay-trace default probe arms ($GATE)"
echo "=============================================================================="
echo

LIVE_FRAG="$WORK/live_resolution.sh"
extract_resolution "$GATE" "$LIVE_FRAG" \
    || { echo "⛔ REFUSED: could not extract the resolution block from the live gate" >&2; exit 2; }
echo "extracted from the LIVE gate:"
sed 's/^/  | /' "$LIVE_FRAG" | grep -E 'REPLAY_TRACE_VERBOSITY=|^  \| (if|else|fi)' || true
echo

# --- GREEN-1: the shipped default is quiet -----------------------------------------------------
mapfile -t got < <(resolve "$LIVE_FRAG" PGEN_SV_STIMULI_QUALITY_REPLAY_TRACE_VERBOSITY=)
check       "GREEN-1 default (unset override) resolves to"        "${got[0]}" "none"
check_substr "GREEN-1 note names the opt-in env var"              "${got[1]}" "PGEN_SV_STIMULI_QUALITY_REPLAY_TRACE_VERBOSITY=low"
check_substr "GREEN-1 note states the cost of turning it on"      "${got[1]}" "50-60 MB/s of log"

# --- CTRL-1: the override still works (the capability is opt-in, not deleted) -------------------
mapfile -t got < <(resolve "$LIVE_FRAG" PGEN_SV_STIMULI_QUALITY_REPLAY_TRACE_VERBOSITY=low)
check       "CTRL-1 override=low still resolves to"               "${got[0]}" "low"
check_substr "CTRL-1 note says tracing is ENABLED by override"    "${got[1]}" "ENABLED by override"
check_substr "CTRL-1 note names how to restore the default"      "${got[1]}" "unset PGEN_SV_STIMULI_QUALITY_REPLAY_TRACE_VERBOSITY"

# --- CTRL-2: an arbitrary override level is passed through unchanged ---------------------------
mapfile -t got < <(resolve "$LIVE_FRAG" PGEN_SV_STIMULI_QUALITY_REPLAY_TRACE_VERBOSITY=debug)
check       "CTRL-2 override=debug passes through"                "${got[0]}" "debug"

# --- BEFORE-1: the RETIRED default replayed from git, not described ----------------------------
PREV_GATE="$WORK/prev_gate.sh"
if git show "HEAD:$GATE" > "$PREV_GATE" 2>/dev/null; then
    PREV_FRAG="$WORK/prev_resolution.sh"
    extract_resolution "$PREV_GATE" "$PREV_FRAG" \
        || { echo "⛔ REFUSED: could not extract the resolution block from HEAD's gate" >&2; exit 2; }
    mapfile -t got < <(resolve "$PREV_FRAG" PGEN_SV_STIMULI_QUALITY_REPLAY_TRACE_VERBOSITY=)
    check       "BEFORE-1 HEAD's retired default resolved to"      "${got[0]}" "low"
    check       "BEFORE-1 HEAD had no discoverable note"           "${got[1]}" "<none>"
else
    echo "  ⚠️  BEFORE-1 skipped: git show HEAD:$GATE unavailable"
fi

# --- RED-1: the new summary key must not shadow the scraped key -------------------------------
# `sota_exit_gate.sh`'s scraper is `sed -nE "s/^KEY: (.*)$/\1/p" | tail -n 1`. A new key that is a
# prefix-extension of a scraped key would silently win the `tail -n 1`. Replay the REAL matcher.
SUMMARY_FIXTURE="$WORK/summary_fixture.txt"
{
    echo "closed_loop_replay_trace_verbosity: none"
    echo "closed_loop_replay_trace_verbosity_note: quiet by default; set X=low for live progress"
} > "$SUMMARY_FIXTURE"
scraped="$(sed -nE "s/^closed_loop_replay_trace_verbosity: (.*)$/\1/p" "$SUMMARY_FIXTURE" | tail -n 1)"
check "RED-1 anchored scraper reads the value, not the note"      "$scraped" "none"
scraped_grepf="$(grep -F "closed_loop_replay_trace_verbosity: " "$SUMMARY_FIXTURE" | tail -n 1)"
check "RED-1b grep -F matcher also reads the value line"          "$scraped_grepf" "closed_loop_replay_trace_verbosity: none"

# --- RED-2: nothing may start READING these stage logs ----------------------------------------
# The whole disposition rests on no consumer of the replay stage logs. Re-derive it, never assume.
log_consumers="$(grep -rn 'closed_loop_replay' rust/scripts/*.sh scripts/*.sh 2>/dev/null \
    | grep -iE '\.log|logs/' | grep -vc '^$' || true)"
check "RED-2 consumers of the replay stage LOGS (must be 0)"      "${log_consumers:-0}" "0"

# --- CTRL-3: the gate still parses -------------------------------------------------------------
if bash -n "$GATE" 2>"$WORK/syntax.err"; then
    check "CTRL-3 gate passes bash -n"                            "ok" "ok"
else
    check "CTRL-3 gate passes bash -n"                            "$(cat "$WORK/syntax.err")" "ok"
fi

echo
echo "=============================================================================="
printf 'replay-trace default probes: %d passed, %d failed\n' "$pass" "$fail"
echo "=============================================================================="
[[ "$fail" -eq 0 ]]
