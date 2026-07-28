#!/usr/bin/env bash
# docs/tasks/artifacts/ci_parity_gate_rot/run_earned_zero_probes.sh
#
# CI-PARITY-GATE-ROT.5 — RED / GREEN / CONTROL arms for the "the zero must be EARNED" replacement
# in `rust/scripts/sv_failure_context_contract_gate.sh`.
#
# ⭐ WHAT IS BEING PROVED. The three assertions this replaces read
#   `(.by_failure_context_excerpt | length) < 1` → `error: expected at least one … excerpt`
# and therefore **passed only when the SV parser rejected its own generated sample.** The claim to
# prove is that the replacement is a CORRECTION and not a relaxation: it must
#   (a) STOP failing on a healthy zero            → GREEN-1, and CTRL-1 shows the old form failed there;
#   (b) START failing on things the old form could not see → RED-A (vacuous run) and RED-B
#       (rejections with no excerpt), neither of which the old `length < 1` test could distinguish
#       from a healthy run;
#   (c) KEEP failing everywhere the old form was right → RED-C, RED-D, and CTRL-2 (a real
#       counterexample still passes both forms).
#
# ⛔ THE FUNCTION IS SOURCED FROM THE LIVE GATE, NEVER RE-TYPED. `GENERATED-LINT-CORRECTNESS.4`
# found a probe driver that had hand-copied the rule it was verifying, so correcting the rule left
# the driver measuring the stale one. Here the helper is extracted from the gate file at run time,
# so this driver cannot test a rule the gate does not apply.
#
# Usage: bash docs/tasks/artifacts/ci_parity_gate_rot/run_earned_zero_probes.sh
# Exit 0 iff every arm reaches its expected verdict.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"; cd "$ROOT"

GATE="rust/scripts/sv_failure_context_contract_gate.sh"
[ -f "$GATE" ] || { echo "probes: ✗ gate not found: $GATE" >&2; exit 1; }

WORK="rust/target/ci_parity_gate_rot_probe/earned_zero"
rm -rf "$WORK"; mkdir -p "$WORK"

# --- extract the LIVE helper + its two dependencies, rather than re-typing them ----------------
HELPER="$WORK/helper.sh"
{
  echo '#!/usr/bin/env bash'
  echo 'set -uo pipefail'
  sed -n '/^extract_json_number() {/,/^}/p' "$GATE"
  sed -n '/^extract_json_string() {/,/^}/p' "$GATE"
  sed -n '/^assert_failure_context_zero_is_earned() {/,/^}/p' "$GATE"
} > "$HELPER"
for fn in extract_json_number extract_json_string assert_failure_context_zero_is_earned; do
  grep -q "^${fn}() {" "$HELPER" || { echo "probes: ✗ could not extract $fn from $GATE" >&2; exit 1; }
done

# --- the OLD form, extracted from HEAD so the before→after comparison is against the real thing --
OLD="$WORK/old_form.sh"
{
  echo '#!/usr/bin/env bash'
  echo 'set -uo pipefail'
  git show "HEAD:$GATE" | sed -n '/^extract_json_number() {/,/^}/p'
  cat <<'SH'
# Verbatim shape of the retired assertion (HEAD): a counterexample must EXIST.
old_assert() {
  local triage_json="$1"
  local n
  n="$(extract_json_number "$triage_json" '(.by_failure_context_excerpt | length)')"
  if [[ "$n" -lt 1 ]]; then
    echo "error: expected at least one generation failure-context excerpt" >&2
    exit 1
  fi
}
SH
} > "$OLD"
grep -q 'expected at least one generation failure-context excerpt' "$OLD" || {
  echo "probes: ✗ HEAD does not carry the old assertion — before→after comparison would be fake" >&2; exit 1; }

triage() {  # triage <file> <total_counterexamples> <excerpt_kinds_json> <previews_json>
  cat > "$WORK/$1" <<JSON
{
  "aggregate_surface": "generation",
  "grammar_name": "systemverilog",
  "total_counterexamples": $2,
  "by_failure_context_excerpt": $3,
  "sample_previews": $4
}
JSON
}

triage healthy_zero.json      0 '{}'                 '[]'
triage rejections_no_excerpt.json 0 '{}'             '[]'
triage cx_no_excerpt.json     3 '{}'                 '[{"failure_context_excerpt":"module m;"}]'
triage cx_empty_preview.json  3 '{"module m;": 3}'   '[{"failure_context_excerpt":""}]'
triage real_counterexample.json 3 '{"module m;": 3}' '[{"failure_context_excerpt":"module m;\n  wire w"}]'

pass=0; fail=0
arm() {  # arm <label> <expected PASS|FAIL> <triage> <attempts> <rejections> [needle]
  local label="$1" want="$2" t="$3" att="$4" rej="$5" needle="${6:-}"
  local out rc got
  out="$( ( set +e; . "$HELPER"; assert_failure_context_zero_is_earned \
              "probe" "$WORK/$t" "$att" "$rej" "synthetic fixture $t" ) 2>&1 )"; rc=$?
  [ "$rc" -eq 0 ] && got=PASS || got=FAIL
  if [ "$got" != "$want" ]; then
    fail=$((fail + 1)); printf '✗ %-38s expected %s, got %s\n' "$label" "$want" "$got"
    printf '%s\n' "$out" | sed 's/^/      /'; return 0
  fi
  if [ -n "$needle" ] && ! printf '%s\n' "$out" | grep -qF -- "$needle"; then
    fail=$((fail + 1)); printf '✗ %-38s %s as expected, but never mentions %s\n' "$label" "$got" "$needle"
    printf '%s\n' "$out" | sed 's/^/      /'; return 0
  fi
  pass=$((pass + 1)); printf '✓ %-38s %s\n' "$label" "$got"
}

old_arm() {  # old_arm <label> <expected PASS|FAIL> <triage>
  local label="$1" want="$2" t="$3"
  local rc got
  ( set +e; . "$OLD"; old_assert "$WORK/$t" ) >/dev/null 2>&1; rc=$?
  [ "$rc" -eq 0 ] && got=PASS || got=FAIL
  if [ "$got" != "$want" ]; then
    fail=$((fail + 1)); printf '✗ %-38s expected %s, got %s\n' "$label" "$want" "$got"; return 0
  fi
  pass=$((pass + 1)); printf '✓ %-38s %s\n' "$label" "$got"
}

printf '%s\n' "=============================================================================="
printf 'CI-PARITY-GATE-ROT.5 — "the zero must be EARNED" probes\n'
printf '%s\n' "=============================================================================="

# ---- (a) the new form STOPS failing on a healthy zero, and the old form DID fail there ----------
# These are the measured live numbers: one sample requested, accepted first try, zero rejections.
arm     "GREEN-1 healthy zero (attempts=1)" PASS healthy_zero.json      1 0 "EARNED"
old_arm "CTRL-1  same input, OLD assertion" FAIL healthy_zero.json

# ---- (b) the new form STARTS failing on what the old form could not see -------------------------
# ⭐ RED-A and RED-B both read `by_failure_context_excerpt | length == 0`, exactly like GREEN-1.
#    The old assertion CANNOT tell the three apart — it fails all of them, for the same wrong
#    reason. The new one separates a healthy zero from a vacuous run and from a broken capture path.
arm     "RED-A   vacuous run (attempts=0)"  FAIL healthy_zero.json      0 0 "never exercised"
arm     "RED-B   rejections, zero excerpts" FAIL rejections_no_excerpt.json 20 4 "left no excerpt"
old_arm "CTRL-3  RED-B input, OLD assertion (fails for the WRONG reason)" FAIL rejections_no_excerpt.json

# ---- (c) the new form KEEPS failing everywhere the old form was right ---------------------------
arm     "RED-C   counterexamples, no excerpt" FAIL cx_no_excerpt.json    20 3 "no failure-context excerpt"
arm     "RED-D   excerpt present but EMPTY"   FAIL cx_empty_preview.json 20 3 "missing or empty"
arm     "GREEN-2 a real counterexample"       PASS real_counterexample.json 20 3 "3 counterexample(s)"
old_arm "CTRL-2  real counterexample, OLD assertion" PASS real_counterexample.json

# ---- the class is closed: no "a failure must exist" assertion survives --------------------------
survivors="$(grep -rn 'expected at least one' rust/scripts/*.sh | grep -v '^\S*:[0-9]*:#' | wc -l | tr -d ' ')"
if [ "$survivors" = "0" ]; then
  pass=$((pass + 1)); printf '✓ %-38s 0 surviving "a failure must exist" assertions\n' "SWEEP   class closed"
else
  fail=$((fail + 1)); printf '✗ %-38s %s survivor(s):\n' "SWEEP   class closed" "$survivors"
  grep -rn 'expected at least one' rust/scripts/*.sh | grep -v '^\S*:[0-9]*:#' | sed 's/^/      /'
fi

printf '%s\n' "------------------------------------------------------------------------------"
printf 'arms=%d  PASS=%d  FAIL=%d\n' "$((pass + fail))" "$pass" "$fail"
[ "$fail" -eq 0 ]
