#!/usr/bin/env bash
# docs/tasks/artifacts/engine_universal_services/probe_fingerprint_gate/probe.sh
#
# ENGINE-UNIVERSAL-SERVICES.24 acceptance (b) — THE RED PROBES FOR THE PROBE-FINGERPRINT ARM.
#
#   A control never observed failing is not known to work (`docs/CLAIM_VERIFICATION.md` §6).
#
# ⛔ WHAT THIS GUARDS. `PARSE-COST-RATCHET`'s baseline pins four inputs — grammar, generated
# parser, instrument, sampled corpus files — and every one of them is a SOURCE. The numbers are
# produced by `rust/target/release/parseability_probe`, an UNTRACKED build artifact that nothing
# hashed and nothing tied to the parser it was compiled from. `.20` slice 4 built one from an
# experimental arm with left-recursion guard emission suppressed, and with that binary on disk the
# gate printed *"the measurement cannot have moved"* while `nm … | grep -c _lr_guard` read 0
# against a pinned parser declaring 6.
#
# ⭐ AND THE GAP FAILED IN THE PASSING DIRECTION. On four sampled files that binary reports
# 762,345 rule entries where the shipped one reports 11,240,430 — 14.7× smaller — and this ratchet
# breaches on a RISE. A FALL is a note reading *"an improvement — promote it deliberately so the
# ratchet tightens"*. So the wrong binary did not merely mislead a reader; it invited a rebaseline
# that would have lowered the ratchet permanently to a number no real parser produces.
#
# ⛔⛔ TWO KINDS OF ARM, AND THE DISTINCTION IS THE POINT.
#
#   REAL-BINARY arms use `rust/target/lr_ab_arms/probe_arm3` — the actual incident fixture, the
#   guard-suppressed binary from `.20` slice 4 — and the real release probe. They prove the check
#   fires on the thing that actually happened. ⚠️ Their honest limit: arm3 PREDATES
#   `--parser-fingerprint`, so it exercises the "this binary cannot say what it embeds" path, NOT
#   the digest COMPARISON.
#
#   STUB arms use a tiny executable script that prints a chosen fingerprint payload. That is a
#   legitimate probe for this check — the check's entire input is the payload — and it is the ONLY
#   way to drive the digest comparison and every malformed-payload path without a 22-minute
#   release build per arm. ⭐ STUB GREEN (arm 2) is what keeps the stubs honest: a stub printing
#   the CORRECT digest must PASS, so the RED stubs are failing on their payload and not merely on
#   being stubs.
#
# ⛔ NOTHING HERE MOVES A BINARY OVER THE DEFAULT PATH. `PGEN_PARSE_COST_PROBE` selects the probe,
# so the incident replays against the real fixture without a mv/restore dance whose failure mode
# is a corrupted tree. A control whose own setup can break the repository is a control people stop
# running.
#
# Usage:  bash docs/tasks/artifacts/engine_universal_services/probe_fingerprint_gate/probe.sh
# Output: docs/tasks/artifacts/engine_universal_services/probe_fingerprint_gate/probe.txt
set -uo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../../../.." && pwd)"; cd "$ROOT" || exit 2
OUT="$HERE/probe.txt"

INSTRUMENT="stimuli/sv/corpus_parse_cost.py"
GATE="scripts/check_parse_cost_ratchet.sh"
GENERATED_PARSER="generated/systemverilog_parser.rs"
REAL_PROBE="rust/target/release/parseability_probe"
ARM3="rust/target/lr_ab_arms/probe_arm3"
WORK="rust/target/probe_fingerprint_gate"
rm -rf "$WORK"; mkdir -p "$WORK"

pass=0; fail=0

# The digest the tree actually carries — computed here, never typed, so an arm that must MATCH
# cannot drift into being an arm that must differ.
LIVE_SHA="$(shasum -a 256 "$GENERATED_PARSER" 2>/dev/null | cut -d' ' -f1)"

# $1 stub name · $2 exit code the stub returns · $3 stdout payload
stub() {
  local path="$WORK/$1"
  {
    printf '#!/usr/bin/env bash\n'
    printf 'cat <<'"'"'PAYLOAD'"'"'\n%s\nPAYLOAD\n' "$3"
    printf 'exit %s\n' "$2"
  } > "$path"
  chmod +x "$path"
  printf '%s' "$path"
}

# $1 arm name · $2 expected exit code · $3 probe path ("" = none) · $4 what it proves
arm() {
  local name="$1" want="$2" probe="$3" proves="$4"
  local out rc
  if [ -n "$probe" ]; then
    out="$(PGEN_PARSE_COST_PROBE="$probe" python3 "$INSTRUMENT" --verify-probe-fingerprint 2>&1)"
  else
    # ⛔ A TREE WITH NO PROBE AT ALL. There is no way to spell "no probe" through the CLI on a
    # machine that HAS one, so this arm imports the instrument and calls the mode's own entry
    # point with `None` — the exact value `find_probe` returns when neither build exists. That is
    # a narrower arm than the others by construction, and it is named as such rather than dressed
    # up: it proves the NOT-EVALUATED branch, not the search that reaches it.
    out="$(python3 -c "
import importlib.util, sys
spec = importlib.util.spec_from_file_location('cpc', '$INSTRUMENT')
m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m)
sys.exit(m.run_verify_probe_fingerprint(None))
" 2>&1)"
  fi
  rc=$?
  if [ "$rc" -eq "$want" ]; then
    printf '  ✓ %-36s exit %d (expected %d) — %s\n' "$name" "$rc" "$want" "$proves"
    pass=$((pass + 1))
  else
    printf '  ✗ %-36s exit %d (expected %d) — %s\n' "$name" "$rc" "$want" "$proves"
    printf '%s\n' "$out" | sed 's/^/        /'
    fail=$((fail + 1))
  fi
  printf '### %s\n' "$name" >> "$WORK/detail.txt"
  printf '%s\n' "$out" | sed 's/^/        | /' >> "$WORK/detail.txt"
}

# $1 arm name · $2 expected exit code · $3 probe path · $4 extra env assignment ("" = none)
#                                                       · $5 what it proves
# ⛔ Drives the GATE, not the instrument — tier 1 only (no PGEN_PARSE_COST_REMEASURE), which is
# seconds, plus one tier-2 arm below that must refuse BEFORE measuring and so is also seconds.
gate_arm() {
  local name="$1" want="$2" probe="$3" extra="$4" proves="$5"
  local out rc
  out="$(env PGEN_PARSE_COST_PROBE="$probe" ${extra:+$extra} bash "$GATE" 2>&1)"
  rc=$?
  if [ "$rc" -eq "$want" ]; then
    printf '  ✓ %-36s exit %d (expected %d) — %s\n' "$name" "$rc" "$want" "$proves"
    pass=$((pass + 1))
  else
    printf '  ✗ %-36s exit %d (expected %d) — %s\n' "$name" "$rc" "$want" "$proves"
    printf '%s\n' "$out" | sed 's/^/        /'
    fail=$((fail + 1))
  fi
  printf '### %s\n' "$name" >> "$WORK/detail.txt"
  printf '%s\n' "$out" | sed 's/^/        | /' >> "$WORK/detail.txt"
}

# ⛔ NOT `{ … } | tee`. A pipeline runs its left side in a SUBSHELL, so the pass/fail counters
# would be discarded and this script would exit 0 while printing "8 failed" — the exact
# report-says-one-thing / exit-code-says-another shape `.20` slice 4's aggregator shipped with.
exec > >(tee "$OUT") 2>&1

printf '===========================================================================\n'
printf 'ENGINE-UNIVERSAL-SERVICES.24 (b) — probe-fingerprint arm, RED probes\n'
printf '===========================================================================\n'
printf 'instrument:   %s\n' "$INSTRUMENT"
printf 'gate:         %s\n' "$GATE"
printf 'parser:       %s  sha256 %s\n' "$GENERATED_PARSER" "${LIVE_SHA:0:16}…"
printf 'real probe:   %s\n' "$REAL_PROBE"
printf 'arm3 fixture: %s\n\n' "$ARM3"

printf -- '-- REAL-BINARY arms (the incident, replayed) -------------------------------\n'

arm "GREEN real release probe" 0 "$REAL_PROBE" \
    "the shipped probe embeds the parser on disk, so the REDs are not vacuous"

if [ -x "$ARM3" ]; then
  arm "RED 1 the .20 slice-4 arm3 binary" 1 "$ARM3" \
      "THE INCIDENT: the guard-suppressed binary is refused, not measured"
else
  printf '  ⚠ %-36s SKIPPED — %s is absent (untracked build tree)\n' \
         "RED 1 the .20 slice-4 arm3 binary" "$ARM3"
fi

printf -- '\n-- STUB arms (the payload paths a real rebuild cannot reach cheaply) -------\n'

a="$(stub green_match 0 "{\"pgen_parser_fingerprint_version\":1,\"parsers\":{\"systemverilog\":\"$LIVE_SHA\"},\"absent\":[]}")"
arm "GREEN 2 stub with the LIVE digest" 0 "$a" \
    "a stub printing the right digest PASSES — the REDs below fail on payload, not on being stubs"

a="$(stub red_wrong 0 "{\"pgen_parser_fingerprint_version\":1,\"parsers\":{\"systemverilog\":\"$(printf '%064d' 0)\"},\"absent\":[]}")"
arm "RED 2 a DIFFERENT parser digest" 1 "$a" \
    "the comparison itself fires — the path arm3 cannot reach, because it predates the flag"

a="$(stub red_absent 0 '{"pgen_parser_fingerprint_version":1,"parsers":{"vhdl":"'"$LIVE_SHA"'"},"absent":["systemverilog"]}')"
arm "RED 3 built with NO SV parser" 1 "$a" \
    "'not measured' is its own verdict and never reads as a match"

a="$(stub red_version 0 '{"pgen_parser_fingerprint_version":99,"parsers":{"systemverilog":"'"$LIVE_SHA"'"}}')"
arm "RED 4 payload schema moved" 1 "$a" \
    "a reader that outlived its producer refuses instead of unpacking it anyway"

a="$(stub red_placeholder 0 '{"pgen_parser_fingerprint_version":1,"parsers":{"systemverilog":"unknown"}}')"
arm "RED 5 a placeholder, not a digest" 1 "$a" \
    "a stand-in is REFUSED, never compared — else it would read as 'wrong parser'"

a="$(stub red_notjson 0 'Usage:
  parseability_probe --supports <grammar_name>')"
arm "RED 6 a probe predating the flag" 1 "$a" \
    "usage-on-stdout is diagnosed as an old binary, with 'rebuild it' as the act"

a="$(stub red_exit 2 '')"
arm "RED 7 the probe exits non-zero" 1 "$a" \
    "a refusing probe is a refusing check, never a silent pass"

arm "RED 8 no probe on disk at all" 3 "" \
    "NOT EVALUATED is its own exit code — a tree with no probe is not a clean tree"

printf -- '\n-- GATE arms (the property acceptance (b) actually names) ------------------\n'

a="$(stub gate_wrong 0 "{\"pgen_parser_fingerprint_version\":1,\"parsers\":{\"systemverilog\":\"$(printf '%064d' 0)\"},\"absent\":[]}")"

gate_arm "GREEN 3 gate tier 1, right probe" 0 "$REAL_PROBE" "" \
         "the gate passes on the real tree with the real probe"

gate_arm "RED 9 gate tier 1 NOTES a mismatch" 0 "$a" "" \
         "tier 1 measures nothing, so it warns rather than blocking every commit"

gate_arm "RED 10 gate tier 2 REFUSES" 1 "$a" "PGEN_PARSE_COST_REMEASURE=1" \
         "ACCEPTANCE (b): the re-measure refuses BEFORE it measures one file"

gate_arm "RED 11 escape hatch cannot reach it" 1 "$a" \
         "PGEN_PARSE_COST_REMEASURE=1 PGEN_PARSE_COST_ALLOW_PROBE_MISMATCH=1" \
         "the operator override is stripped by the gate — the tracked-reference path has no hatch"

printf '\n%d passed, %d failed.\n' "$pass" "$fail"
if [ "$fail" -eq 0 ]; then
  printf '⭐ every refusal in the probe-fingerprint arm has now been OBSERVED firing.\n'
else
  printf '⛔ an arm did not behave as declared — the arm is not proven.\n'
fi

# let the tee-writer drain before the shell exits, or the tail of the report is lost
exec 1>&- 2>&-
wait
exit "$fail"
