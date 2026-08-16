#!/usr/bin/env bash
# docs/tasks/artifacts/engine_universal_services/probe_fingerprint_gate/wrong_probe_cost.sh
#
# ENGINE-UNIVERSAL-SERVICES.24 — HOW WRONG A WRONG PROBE IS, and in which DIRECTION.
#
# The leaf's headline number — that the `.20` slice-4 arm reports ~15× FEWER rule entries than the
# shipped probe on the same files — is the reason the fingerprint arm is a REFUSAL and not a note
# in a document. This script is its PRODUCER, tracked, so the number can be re-derived rather than
# quoted (`ENGINE-UNIVERSAL-SERVICES.21` acceptance (e): a measurement whose instrument lives in an
# untracked scratch directory is a measurement nobody can reproduce).
#
# ⭐ THE DIRECTION IS THE POINT. `PARSE-COST-RATCHET` breaches on a RISE. A FALL is a note reading
# *"an improvement — promote it deliberately so the ratchet tightens"*. So a probe built from a
# smaller parser does not trip the gate; it invites a rebaseline that lowers the ratchet forever to
# a number no real parser produces. A gap that fails in the passing direction cannot be left to
# discipline.
#
# ⛔ THE SAMPLE IS DERIVED FROM THE TRACKED BASELINE, NEVER TYPED. Four files — the first three
# ACCEPTED `breadth` rows and the first ACCEPTED `hot` row of `entries.tsv`, in file order. Accepted
# only, because a rejected parse commits nothing and its entry count is all speculation; four,
# because this is an illustration of a ratio, not a baseline.
#
# ⛔ It runs the wrong-probe arm under `PGEN_PARSE_COST_ALLOW_PROBE_MISMATCH=1` — which is the
# whole point of that switch existing — and then ASSERTS that the resulting `advisory.json` carries
# the mismatch stamp. A measurement taken with a mismatched probe that did not say so would be the
# defect one layer down.
#
# Usage:  bash docs/tasks/artifacts/engine_universal_services/probe_fingerprint_gate/wrong_probe_cost.sh
# Output: docs/tasks/artifacts/engine_universal_services/probe_fingerprint_gate/wrong_probe_cost.txt
set -uo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../../../.." && pwd)"; cd "$ROOT" || exit 2
OUT="$HERE/wrong_probe_cost.txt"

INSTRUMENT="stimuli/sv/corpus_parse_cost.py"
BASELINE="docs/tasks/artifacts/engine_universal_services/parse_cost_ratchet/entries.tsv"
REAL_PROBE="rust/target/release/parseability_probe"
ARM3="rust/target/lr_ab_arms/probe_arm3"
WORK="rust/target/probe_fingerprint_gate/wrong_probe_cost"
rm -rf "$WORK"; mkdir -p "$WORK"

exec > >(tee "$OUT") 2>&1

printf '===========================================================================\n'
printf 'ENGINE-UNIVERSAL-SERVICES.24 — the SIZE and DIRECTION of a wrong-probe error\n'
printf '===========================================================================\n'

if [ ! -x "$ARM3" ]; then
  printf '⚠️  SKIPPED — %s is absent.\n' "$ARM3"
  printf '   It is the .20 slice-4 experimental arm, and it lives in the untracked rust/target/\n'
  printf '   tree. Rebuild an equivalent (guard emission suppressed) or run this on a tree that\n'
  printf '   still has it. ⛔ Reported, never silently skipped.\n'
  exec 1>&- 2>&-; wait; exit 3
fi
if [ ! -x "$REAL_PROBE" ]; then
  printf '⚠️  SKIPPED — %s is absent; build it first.\n' "$REAL_PROBE"
  exec 1>&- 2>&-; wait; exit 3
fi

# the derived four-file manifest
{
  awk -F'\t' 'NR>1 && $4=="yes" && $2=="breadth" {print $2"\t"$3; n++} n==3{exit}' "$BASELINE"
  awk -F'\t' 'NR>1 && $4=="yes" && $2=="hot"     {print $2"\t"$3; exit}'            "$BASELINE"
} > "$WORK/sample.tsv"
printf 'sample (derived from %s):\n' "$BASELINE"
sed 's/^/  /' "$WORK/sample.tsv"
printf '\n'

# ⛔ The narration goes to STDERR and only the number to STDOUT. Command substitution captures
# stdout alone, so a run() that printed both would swallow its own narration — and the script's
# own `exec 2>&1` still routes that stderr into the tee, so nothing is lost from the record.
run() {  # $1 label · $2 probe · $3 outdir · $4 extra env ("" = none)
  local label="$1" probe="$2" outdir="$3" extra="$4" line
  line="$(env PGEN_PARSE_COST_PROBE="$probe" ${extra:+$extra} \
          python3 "$INSTRUMENT" --manifest "$WORK/sample.tsv" --outdir "$outdir" --repeats 1 \
          2>&1 | tail -1)"
  printf '%-28s %s\n' "$label" "$line" >&2
  python3 - "$outdir/entries.tsv" <<'PY'
import sys
tot = 0
with open(sys.argv[1], encoding="utf-8") as fh:
    idx = {n: i for i, n in enumerate(fh.readline().rstrip("\n").split("\t"))}
    for line in fh:
        if line.strip():
            tot += int(line.rstrip("\n").split("\t")[idx["entries"]])
print(tot)
PY
}

printf -- '-- the SHIPPED probe (embeds the parser on disk) ---------------------------\n'
SHIPPED="$(run "shipped:" "$REAL_PROBE" "$WORK/shipped" "")"

printf -- '\n-- the .20 slice-4 ARM (guard emission suppressed) -------------------------\n'
printf '⛔ needs PGEN_PARSE_COST_ALLOW_PROBE_MISMATCH=1 — the guard refuses it otherwise, which\n'
printf '   is the property this whole leaf ships.\n'
WRONG="$(run "arm3:" "$ARM3" "$WORK/arm3" "PGEN_PARSE_COST_ALLOW_PROBE_MISMATCH=1")"

printf '\n'
printf 'rule entries, shipped probe : %s\n' "$SHIPPED"
printf 'rule entries, arm3 probe    : %s\n' "$WRONG"
python3 - "$SHIPPED" "$WRONG" <<'PY'
import sys
s, w = int(sys.argv[1]), int(sys.argv[2])
print(f"ratio shipped/arm3          : {s / w:.1f}×")
print(f"direction                   : the wrong probe reads "
      f"{'LOWER' if w < s else 'HIGHER'} — "
      f"{'a FALL, which this ratchet reports as an IMPROVEMENT worth promoting' if w < s else 'a RISE, which the ratchet would have caught'}")
PY

printf '\n-- the STAMP: a mismatched measurement must say so in its own artifact -----\n'
python3 - "$WORK/arm3/advisory.json" "$WORK/shipped/advisory.json" <<'PY'
import json, sys
bad = json.load(open(sys.argv[1], encoding="utf-8"))
good = json.load(open(sys.argv[2], encoding="utf-8"))
ok = True
for label, d, want in (("arm3", bad, False), ("shipped", good, True)):
    got = d.get("probe_parser_matches_generated")
    mark = "✓" if got is want else "✗"
    ok &= got is want
    print(f"  {mark} {label:8s} advisory.json probe_parser_matches_generated = {got!r} "
          f"(expected {want!r}); probe_parser_sha256 = "
          f"{(d.get('probe_parser_sha256') or 'null')[:16]}…")
sys.exit(0 if ok else 1)
PY
rc=$?
printf '\n'
[ "$rc" -eq 0 ] && printf '⭐ both artifacts declare which parser their executable carried.\n' \
                || printf '⛔ an artifact did not record its probe fingerprint.\n'

exec 1>&- 2>&-
wait
exit "$rc"
