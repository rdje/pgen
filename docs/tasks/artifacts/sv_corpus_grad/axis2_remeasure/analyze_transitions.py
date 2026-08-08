#!/usr/bin/env python3
"""Per-file transition census between two `run_external_corpus.sh` results.tsv files.

SV-CORPUS-GRAD.10 — the axis-2 RE-MEASURE. A pass-COUNT delta is the least
trustworthy signal in a corpus run: it nets a heal against a regression and
reports zero (the lesson banked as
docs/knowledge/a-rising-pass-rate-is-not-evidence-of-correctness.md). What
answers "did anything break?" is the per-FILE transition census, which this
script computes.

Both inputs are `<subcorpus>\t<status>\t<path>` TSVs. Column 3 is repo-root-
relative since CORPUS-GRAD-ALL.2.1, but older artifacts carry ABSOLUTE paths,
so paths are normalized to a repo-root-relative key before joining — otherwise
every row of an older baseline looks "removed" and every new row "added".

Usage:
  python3 docs/tasks/artifacts/sv_corpus_grad/axis2_remeasure/analyze_transitions.py \
      --before rust/target/sv_axis2_baseline/results.before.tsv \
      --after  stimuli/sv/characterization/results.tsv \
      [--out   docs/tasks/artifacts/sv_corpus_grad/axis2_remeasure/transitions.txt]
"""

import argparse
import sys
from collections import Counter, defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]
# The stable infix every corpus path contains, used only when an absolute path in an
# older artifact points at a DIFFERENT checkout than this one (the exact situation
# CORPUS-GRAD-ALL.2.1 made impossible for new artifacts).
CORPUS_MARKER = "/stimuli/"


def normalize(path: str) -> str:
    """Repo-root-relative key, whatever spelling the artifact used."""
    p = path.strip()
    if not p.startswith("/"):
        return p
    try:
        return str(Path(p).relative_to(ROOT))
    except ValueError:
        idx = p.rfind(CORPUS_MARKER)
        return p[idx + 1:] if idx != -1 else p


def load(path: Path) -> dict:
    rows = {}
    with path.open(encoding="utf-8", errors="replace") as fh:
        for line in fh:
            parts = line.rstrip("\n").split("\t")
            if len(parts) != 3:
                continue
            sub, status, fpath = parts
            rows[normalize(fpath)] = (sub, status)
    return rows


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--before", required=True)
    ap.add_argument("--after", required=True)
    ap.add_argument("--out", default=None)
    ap.add_argument("--list-limit", type=int, default=200,
                    help="max rows to list per transition class (0 = unlimited)")
    args = ap.parse_args()

    before = load(Path(args.before))
    after = load(Path(args.after))

    only_before = sorted(set(before) - set(after))
    only_after = sorted(set(after) - set(before))
    common = sorted(set(before) & set(after))

    transitions = Counter()
    per_sub = defaultdict(Counter)
    changed = defaultdict(list)
    for key in common:
        b_status = before[key][1]
        a_sub, a_status = after[key]
        transitions[(b_status, a_status)] += 1
        per_sub[a_sub][(b_status, a_status)] += 1
        if b_status != a_status:
            changed[(b_status, a_status)].append((a_sub, key))

    out = []
    w = out.append
    w("# SV axis-2 per-file transition census")
    w("")
    w(f"before : {args.before}  ({len(before)} rows)")
    w(f"after  : {args.after}  ({len(after)} rows)")
    w(f"common : {len(common)}   only-before: {len(only_before)}   only-after: {len(only_after)}")
    w("")
    b_pass = sum(1 for v in before.values() if v[1] == "pass")
    a_pass = sum(1 for v in after.values() if v[1] == "pass")
    w(f"pass count : {b_pass} -> {a_pass}  (delta {a_pass - b_pass:+d})")
    w("")
    w("## Transition matrix (common files only)")
    w("")
    w("| before | after | files |")
    w("|---|---|---|")
    for (b_status, a_status), n in sorted(transitions.items(), key=lambda kv: (-kv[1], kv[0])):
        mark = "" if b_status == a_status else "   <-- CHANGED"
        w(f"| {b_status} | {a_status} | {n} |{mark}")
    w("")
    w("## Per sub-corpus changed transitions")
    w("")
    w("| sub-corpus | transition | files |")
    w("|---|---|---|")
    for sub in sorted(per_sub):
        for (b_status, a_status), n in sorted(per_sub[sub].items()):
            if b_status != a_status:
                w(f"| {sub} | {b_status} -> {a_status} | {n} |")
    if not any(b != a for sub in per_sub for (b, a) in per_sub[sub]):
        w("| _(none)_ | | |")
    w("")
    for (b_status, a_status), rows in sorted(changed.items()):
        w(f"## {b_status} -> {a_status}  ({len(rows)} files)")
        w("")
        shown = rows if args.list_limit == 0 else rows[: args.list_limit]
        for sub, key in shown:
            w(f"  [{sub}] {key}")
        if len(shown) < len(rows):
            w(f"  ... {len(rows) - len(shown)} more (re-run with --list-limit 0)")
        w("")
    if only_before:
        w(f"## files present in BEFORE only ({len(only_before)})")
        w("")
        for key in only_before[: args.list_limit or len(only_before)]:
            w(f"  {key}")
        w("")
    if only_after:
        w(f"## files present in AFTER only ({len(only_after)})")
        w("")
        for key in only_after[: args.list_limit or len(only_after)]:
            w(f"  {key}")
        w("")

    text = "\n".join(out) + "\n"
    if args.out:
        Path(args.out).parent.mkdir(parents=True, exist_ok=True)
        Path(args.out).write_text(text, encoding="utf-8")
        print(f"wrote {args.out}", file=sys.stderr)
    print(text)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
