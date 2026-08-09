#!/usr/bin/env python3
"""Per-FILE adjudication-class SET diff across a grammar edit, both SV corpus lanes.

`SV-CORPUS-GRAD.3.19` computed this inline; `.3.20` needs it again — and a tightening
leaf needs it MORE than a widening one, because the failure mode it rules out is the
opposite direction. So it is a script, run the same way on both sides.

⛔ WHY A SET DIFF AND NOT COUNTS (the `.3.4` LAW, restated for the adjudicator).
`unexplained_rejects_valid: 304 -> 304` is compatible with 8 rows healing and 8 NEW rows
appearing. Only a set comparison distinguishes them, and on a tightening leaf the NEW-row
direction is precisely the regression being hunted: taking language away can turn a
correctly-accepted file into a wrongly-rejected one. Every class is therefore compared as
a set of `(suite, relpath)` keys, and the per-class NEW / HEALED lists are printed.

⭐ THE CONTROL THAT MATTERS DEPENDS ON THE DIRECTION OF THE EDIT.
  - a WIDENING edit (`.3.19`) must not grow `unexplained_accepts_invalid`;
  - a TIGHTENING edit (`.3.20`) must not grow `unexplained_rejects_valid`.
Both are printed for both lanes, with NEW listed by name, so the reader never has to
infer which control was the load-bearing one.

Manifest columns: suite <TAB> relpath <TAB> observed <TAB> expected <TAB> adjudication [<TAB> basis]

Usage (paths are repo-root-relative, directive 12):
  python3 docs/tasks/artifacts/sv_corpus_grad/config_use_param_override/analyze_adjudication_setdiff.py \
      --lane sv_2017 rust/target/sv_axis2_baseline/adjudication_manifest.pre_3_20.tsv \
                     stimuli/sv/characterization/adjudication_manifest.tsv \
      --lane verilog_2005 rust/target/sv_axis2_baseline/adjudication_manifest_v2005.pre_3_20.tsv \
                          stimuli/sv/characterization/adjudication_manifest_v2005.tsv
"""

import argparse
from collections import Counter, defaultdict
from pathlib import Path

# The two classes whose SET must be watched, whichever way an edit moves the language.
CONTROL_CLASSES = (
    "divergence:unexplained_rejects_valid",
    "divergence:unexplained_accepts_invalid",
)


def load(path: Path):
    """-> ({class: {(suite, relpath)}}, {(suite, relpath): class})."""
    by_class, of_key = defaultdict(set), {}
    with path.open(encoding="utf-8", errors="replace") as fh:
        next(fh, "")  # header
        for line in fh:
            parts = line.rstrip("\n").split("\t")
            if len(parts) < 5:
                continue
            key = (parts[0], parts[1])
            by_class[parts[4]].add(key)
            of_key[key] = parts[4]
    return by_class, of_key


def report(label: str, before: Path, after: Path, list_limit: int) -> int:
    bc, bk = load(before)
    ac, ak = load(after)
    print(f"===== {label} lane   rows {len(bk)} -> {len(ak)}   "
          f"key set identical: {set(bk) == set(ak)}")
    for cls in sorted(set(bc) | set(ac)):
        b, a = len(bc.get(cls, ())), len(ac.get(cls, ()))
        delta = f"  {a - b:+d}" if a != b else ""
        print(f"  {cls:<46} {b:>6} -> {a:>6}{delta}")

    moved = {k: (bk[k], ak[k]) for k in set(bk) & set(ak) if bk[k] != ak[k]}
    print(f"  rows CHANGING adjudication class: {len(moved)}")
    print(f"  transitions: {Counter(moved.values())}")

    breaches = 0
    for cls in CONTROL_CLASSES:
        b, a = bc.get(cls, set()), ac.get(cls, set())
        new, healed = sorted(a - b), sorted(b - a)
        print(f"  {cls}: {len(b)} -> {len(a)} | NEW (set diff): {len(new)} | healed: {len(healed)}")
        for suite, rel in new[:list_limit]:
            print(f"      NEW     {suite}\t{rel}")
        for suite, rel in healed[:list_limit]:
            print(f"      healed  {suite}\t{rel}")
        breaches += len(new)
    print()
    return breaches


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--lane", nargs=3, action="append", metavar=("LABEL", "BEFORE", "AFTER"),
                    required=True, help="lane label, before manifest, after manifest")
    ap.add_argument("--list-limit", type=int, default=40)
    args = ap.parse_args()

    total_new = 0
    for label, before, after in args.lane:
        total_new += report(label, Path(before), Path(after), args.list_limit)
    print(f"TOTAL new rows across both control classes and every lane: {total_new}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
