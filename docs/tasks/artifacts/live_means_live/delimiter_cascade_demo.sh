#!/usr/bin/env bash
# delimiter_cascade_demo.sh — the two-line proof that a delimiter imbalance is NOT a local defect.
# Owning record: docs/decisions/a-delimiter-imbalance-is-not-a-local-defect.md
# Owning leaf:   `LIVE-DOC-CONTAINMENT.8`
#
# CommonMark pairs a backtick run with the NEXT run of the same length, scanning left to right over
# the whole record. So ONE unmatched backtick re-pairs everything after it, and a scan looking for
# "pipes inside a code span" starts reporting REAL table-cell separators as findings.
#
# Two inputs, identical except for one backtick at the FRONT. Expected output:
#     balanced            -> 0
#     one stray up front  -> 1
# The second line is the whole lesson: nothing changed at the pipe, and the pipe changed meaning.
#
# Exits 0 when both readings are as expected, 1 otherwise — so it is a CONTROL, not a printout.
set -uo pipefail

out="$(python3 - <<'PY'
import re

def spans(s):
    """Code spans per CommonMark: a run of N backticks closed by the next run of exactly N."""
    runs = [m.span() for m in re.finditer(r'`+', s)]
    out, k = [], 0
    while k < len(runs):
        a, b = runs[k]
        want = b - a
        m = k + 1
        while m < len(runs) and (runs[m][1] - runs[m][0]) != want:
            m += 1
        if m < len(runs):
            out.append((b, runs[m][0]))
            k = m + 1
        else:
            k += 1
    return out

for label, s in [("balanced", "A `p` | `q` | B"),
                 ("one stray up front", "`A `p` | `q` | B")]:
    inside = sum(1 for a, b in spans(s) if '|' in s[a:b])
    print(f"{label} -> {inside}")
PY
)"
printf '%s\n' "$out"
expected='balanced -> 0
one stray up front -> 1'
if [ "$out" = "$expected" ]; then
  echo "delimiter-cascade: CONTROL HELD — one added delimiter re-paired everything downstream."
  exit 0
fi
echo "delimiter-cascade: CONTROL MISSED — expected:" >&2
printf '%s\n' "$expected" >&2
exit 1
