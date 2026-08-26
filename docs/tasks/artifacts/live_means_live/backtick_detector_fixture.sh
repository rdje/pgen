#!/usr/bin/env bash
# backtick_detector_fixture.sh — why a symptom detector must enumerate its SUCCESS case.
# Owning record: docs/decisions/name-the-success-case-or-it-is-counted-as-the-symptom.md
# Owning leaf:   `LIVE-DOC-CONTAINMENT.9`
#
# Two rendered table cells, exactly as a Markdown renderer emits them:
#   HEALTHY — the author wrote `` `ifdef `` and the renderer produced <code>`ifdef</code>.
#             The backtick is CONTENT. This is the idiom working.
#   BROKEN  — a single-backtick span could not hold its backtick, so the span closed early and
#             the backtick fell into PROSE, outside any <code> element.
#
# A detector that asks "does a backtick survive into the rendered text?" cannot tell them apart and
# reports 2. Stripping <code> first reports 1 — the real defect. On the released-parser bug ledger
# that difference was 21 reported against a true 16, a 31 % over-count, and it sent a leaf to fix
# `SV-0046`, which was entirely healthy.
#
# No renderer required: the fixture IS renderer output. Exits non-zero if the readings change.
set -uo pipefail

out="$(python3 - <<'PY'
import re, html

CELLS = [
    ("HEALTHY", '<td>each of which probes as dying elsewhere (<code>`ifdef</code> x2)</td>'),
    ("BROKEN ", '<td>and a bare <code>MY_MACRO(x)</code> ``, `` include "x.svh" ``.</td>'),
]

def loose(td):
    """Any backtick in the rendered text."""
    return '`' in html.unescape(re.sub(r'<[^>]+>', ' ', td))

def strict(td):
    """A backtick in PROSE — <code> content is the author's escaped text, rendered as intended."""
    prose = re.sub(r'<code[^>]*>.*?</code>', ' ', td, flags=re.S)
    return '`' in html.unescape(re.sub(r'<[^>]+>', ' ', prose))

for name, td in CELLS:
    print(f"{name}  loose={int(loose(td))}  strict={int(strict(td))}")
print(f"TOTALS loose={sum(loose(t) for _, t in CELLS)} strict={sum(strict(t) for _, t in CELLS)}")
PY
)"
printf '%s\n' "$out"
expected='HEALTHY  loose=1  strict=0
BROKEN   loose=1  strict=1
TOTALS loose=2 strict=1'
if [ "$out" = "$expected" ]; then
  echo "backtick-detector: CONTROL HELD — the loose detector counts the idiom WORKING as a defect."
  exit 0
fi
echo "backtick-detector: CONTROL MISSED — expected:" >&2
printf '%s\n' "$expected" >&2
exit 1
