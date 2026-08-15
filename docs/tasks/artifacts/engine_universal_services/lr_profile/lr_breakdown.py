#!/usr/bin/env python3
"""Break LR self/inclusive time down by which FAMILY DEFINITION is used — so the published
percentages carry the predicate that defines the set they describe.

`ENGINE-UNIVERSAL-SERVICES.20` slice 4 / `.21` acceptance (e).

`.20` slice 2 published `self 1.9 % / inclusive 27.3 %` for *"the LR machinery"* without recording
the symbol predicate that defines that set — and at the time the tracked instrument used a NARROWER
predicate than the profile did. This prices every definition on the same reports, so no surface can
quote a percentage whose population is unstated.

⛔⛔ THE LABELS IN THE SCRATCH VERSION WERE STALE AND WOULD HAVE BEEN PROMOTED AS FACT. It called
`_lr_base|_lr_suffix` *"(tracked)"*. That stopped being true when `.21` slice 1 replaced the tracked
predicate with the emitter-derived `_lr_(base|suffix|seed|guard|alt)(?![a-z])`. Promoting a file that
names the wrong definition as the live one is how a corrected number goes stale again, so the
historical definitions are now labelled HISTORICAL and the live one is IMPORTED from its single home
(`stimuli/sv/corpus_parse_cost.py`) rather than re-typed here.

Usage: lr_breakdown.py <sample-report.txt> [...]
"""
import sys
import os

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from lr_attribute import (LR, HISTORICAL_SCRATCH_RE, HISTORICAL_TRACKED_RE,  # noqa: E402
                          parse, selves, check_oracle, STEP)

DEFS = {
    "LIVE     base|suffix|seed|guard|alt (imported)": LR,
    "HIST-A   base|suffix|seed|guard  (scratch)": HISTORICAL_SCRATCH_RE,
    "HIST-B   base|suffix END-anchored (pre-.21)": HISTORICAL_TRACKED_RE,
}

# ⛔ HIST-B MEASURES 0 EVERYWHERE, AND THAT IS NOT "THE LR FAMILY COSTS NOTHING" — READ THE ROW
# CAREFULLY. `_lr_base$|_lr_suffix(_r\d+)?$` is END-ANCHORED and was written for RULE NAMES, which is
# what the corpus instrument classifies. A `sample` report carries SYMBOL strings
# (`…::rule_expression_lr_base  (in parseability_probe)`), so nothing can end at `_lr_base` and the
# predicate matches zero rows by construction.
# ⭐ This retires `.21` GAP 4 ("the classifier is SHARED") with a sharper statement: the two sides
# NEVER shared a classifier — the profile side could not have used the tracked one at all. So the
# 75.1 % / 76 % agreement between them bounds even less than GAP 4 conceded, and the fix is not "make
# them share" but "make the one they DO share — the live predicate — the only one either can use",
# which is what the promotion did.


def rows_for(path):
    worker_root, rows = parse(path)
    if not worker_root:
        sys.exit(f"REFUSE: {path}: no worker thread")
    sc = selves(path, rows)
    check_oracle(path, rows, sc)
    return worker_root, rows, sc


def measure(rows, sc, family_re):
    widx = [i for i in range(len(rows)) if rows[i][3]]
    idx = [i for i in widx if family_re.search(rows[i][2])]
    self_c = sum(sc[i] for i in idx)
    incl, stack = 0, []
    for i in widx:
        d, c, s = rows[i][0], rows[i][1], rows[i][2]
        while stack and stack[-1][0] >= d:
            stack.pop()
        under = bool(stack) and stack[-1][1]
        is_lr = bool(family_re.search(s))
        if is_lr and not under:
            incl += c
        stack.append((d, is_lr or under))
    return len(idx), self_c, incl


agree = True
for path in sys.argv[1:]:
    root, rows, sc = rows_for(path)
    print(f"\n=== {path.rsplit('/', 1)[-1]}   worker root={root}")
    seen = {}
    for label, rx in DEFS.items():
        n, s, inc = measure(rows, sc, rx)
        seen[label] = (n, s, inc)
        print(f"  {label:<48} nodes={n:<5} self={s:>5} ({100.0 * s / root:5.2f}%) "
              f"incl={inc:>5} ({100.0 * inc / root:5.2f}%)")
    live = seen["LIVE     base|suffix|seed|guard|alt (imported)"]
    hist = seen["HIST-A   base|suffix|seed|guard  (scratch)"]
    if live != hist:
        agree = False
        print("  ⇒ ⛔ LIVE and HIST-A DISAGREE on this report — the published `.20` slice 4 "
              "interval was computed under HIST-A and must be re-derived.")

print("\n" + "-" * 78)
if agree:
    print("⭐ LIVE == HIST-A on every report: the predicate correction `.21` slice 1 made to the")
    print("   CORPUS instrument does NOT move the PROFILE numbers, so `.20` slice 4's published")
    print("   1.83-3.60 % / 22.38-26.75 % / 6.9-13.6x intervals survive it unchanged. The two")
    print("   differ only by `_lr_alt` (0 rules in SV) and the `(?![a-z])` anchor (no near-miss")
    print("   symbol in these reports) — stated as a MEASUREMENT here, not as an expectation.")
else:
    print("⛔ at least one report disagrees — see above.")
sys.exit(0 if agree else 1)
