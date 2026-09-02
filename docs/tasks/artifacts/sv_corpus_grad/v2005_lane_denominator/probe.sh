#!/usr/bin/env bash
# docs/tasks/artifacts/sv_corpus_grad/v2005_lane_denominator/probe.sh
#
# SV-CORPUS-GRAD.13m — the RED/GREEN bank for `CORPUS-CENSUS-CURRENCY`'s DECLARED denominator
# source, driven against the REAL tracked v2005 report.
#
# ⛔ WHY THE SEAM AND NOT A MUTATION. Every RED arm needs a lane list that disagrees with the
# report, and the lane list is a TRACKED file. Mutating it in place and reverting later is the
# shape that cost `ENGINE-UNIVERSAL-SERVICES.87` a whole slice — a deliberate mutation must never
# outlive the tool call that applied it. So every arm CONSTRUCTS a declaration in a scratch schema
# (`PGEN_FAMILY_STATUS_SCHEMA`, the gate's own seam) pointing at a COPY of the lane list under
# `rust/target/`, and nothing tracked is written at any point.
#
# ⭐ ARM 0 IS THE FOUNDING MEASUREMENT, RE-DERIVED: the same report under the FILESYSTEM source
# breaches on 3 of 3 suites. Without it, the green arms below are equally consistent with "the
# lane_list source is right" and "arm (4) stopped looking", which is the failure mode this
# repository names most often.
#
# Deterministic, ~2 s, mutates no tracked file. rc 0 = every arm as declared.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"; cd "$ROOT" || exit 2

FAMILY=systemverilog_v2005_raw_parse
SCHEMA=rust/test_data/grammar_quality/family_status_block_v0.json
LANE=stimuli/sv/characterization/v2005_lane_files.tsv
WORK=rust/target/v2005_lane_denominator_probe
rm -rf "$WORK"; mkdir -p "$WORK"
trap 'rm -rf "$ROOT/$WORK"' EXIT INT TERM HUP

pass=0; fail=0
arm() { # arm <label> <expect:green|red> <schema-path> [<grep-token>]
  local label="$1" expect="$2" sch="$3" token="${4-}"
  local out rc
  out="$(PGEN_FAMILY_STATUS_SCHEMA="$sch" bash scripts/check_corpus_census.sh "$FAMILY" 2>&1)"; rc=$?
  local ok=1
  case "$expect" in
    green) [ "$rc" -eq 0 ] || ok=0 ;;
    red)   [ "$rc" -ne 0 ] || ok=0 ;;
  esac
  if [ -n "$token" ] && ! printf '%s' "$out" | grep -qF -- "$token"; then ok=0; fi
  if [ "$ok" -eq 1 ]; then
    pass=$((pass+1)); printf 'PASS  %-58s rc=%d\n' "$label" "$rc"
  else
    fail=$((fail+1)); printf 'FAIL  %-58s rc=%d\n' "$label" "$rc"
    printf '%s\n' "$out" | sed 's/^/        /'
  fi
}

# A declaration built from the REAL one, with the named keys overridden.
# usage: decl <out.json> <python-dict-of-overrides>
decl() {
  python3 - "$1" "$2" <<'PY'
import json, sys
out, overrides = sys.argv[1], json.loads(sys.argv[2])
d = json.load(open("rust/test_data/grammar_quality/family_status_block_v0.json"))
e = dict(d["corpus_census"]["systemverilog_v2005_raw_parse"])
for k, v in overrides.items():
    if v is None:
        e.pop(k, None)
    else:
        e[k] = v
json.dump({"corpus_census": {"systemverilog_v2005_raw_parse": e}}, open(out, "w"), indent=2)
PY
}

# ---- ARM 0 — the founding measurement: the FILESYSTEM source breaches this report -----------------
decl "$WORK/filesystem.json" '{"denominator_source": "filesystem", "corpus_root": "stimuli/sv/subs", "file_extensions": [".v"]}'
arm "0  filesystem source on the v2005 report BREACHES" red "$WORK/filesystem.json" "DENOMINATOR moved"

# ---- ARM 1 — GREEN, the shipped declaration ------------------------------------------------------
arm "1  lane_list source on the tracked tree is GREEN" green "$SCHEMA" \
    "corpus-census[$FAMILY]: OK (tuple 2606/2300/306/0/0"

# ---- ARM 2 — RED: one ADDED row (the experiment `.13m`(c) asks for) -------------------------------
cp "$LANE" "$WORK/added.tsv"
tail -n1 "$LANE" >> "$WORK/added.tsv"
decl "$WORK/added.json" "{\"lane_list\": \"$WORK/added.tsv\"}"
arm "2  RED: one row ADDED to the lane list" red "$WORK/added.json" "holds 342 row(s) for it"

# ---- ARM 3 — RED: one REMOVED row ---------------------------------------------------------------
head -n -1 "$LANE" > "$WORK/removed.tsv"
decl "$WORK/removed.json" "{\"lane_list\": \"$WORK/removed.tsv\"}"
arm "3  RED: one row REMOVED from the lane list" red "$WORK/removed.json" "holds 340 row(s) for it"

# ---- ARM 4 — RED: a suite the report does not publish --------------------------------------------
cp "$LANE" "$WORK/newsuite.tsv"
printf 'brand-new-suite\tx.v\t%s\n' "$(tail -n1 "$LANE" | cut -f3)" >> "$WORK/newsuite.tsv"
decl "$WORK/newsuite.json" "{\"lane_list\": \"$WORK/newsuite.tsv\"}"
arm "4  RED: a suite the report does not publish" red "$WORK/newsuite.json" "suite \`brand-new-suite\`"

# ---- ARM 5 — RED: a listed path that is not a regular file ---------------------------------------
awk -F'\t' 'BEGIN{OFS="\t"} NR==2{$3=$3 ".gone"} {print}' "$LANE" > "$WORK/gone.tsv"
decl "$WORK/gone.json" "{\"lane_list\": \"$WORK/gone.tsv\"}"
arm "5  RED: a listed path that is not a regular file" red "$WORK/gone.json" "are not regular files"

# ---- ARM 6 — RED: the declared lane list is absent (a BREACH, never a skip) -----------------------
decl "$WORK/absent.json" "{\"lane_list\": \"$WORK/no-such-lane.tsv\"}"
arm "6  RED: the declared lane list is absent" red "$WORK/absent.json" "does not exist in this tree"

# ---- ARM 7 — REFUSE: an unimplemented source may not fall through to the walk ---------------------
decl "$WORK/bogus.json" '{"denominator_source": "guess"}'
arm "7  REFUSE: an unimplemented denominator_source" red "$WORK/bogus.json" "which this gate does not"

# ---- ARM 8 — REFUSE: `lane_list` declared with no path -------------------------------------------
decl "$WORK/nopath.json" '{"lane_list": null}'
arm "8  REFUSE: lane_list source with no path" red "$WORK/nopath.json" "and no \`lane_list\` path"

printf 'V2005-LANE-DENOMINATOR: %d/%d as declared\n' "$pass" "$((pass + fail))"
[ "$fail" -eq 0 ]
