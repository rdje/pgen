#!/usr/bin/env python3
"""Attribute LR-machinery self-time and inclusive time from a `/usr/bin/sample` report.

`ENGINE-UNIVERSAL-SERVICES.20` slice 4 / `.21` acceptance (e).

⛔⛔ PROMOTED OUT OF `rust/target/audit_scratch/` (gitignored) BY `.21` ACCEPTANCE (e), AND THAT IS
THE POINT OF THE FILE. `.20` slice 4 published `self 1.83-3.60 % / inclusive 22.38-26.75 % /
ratio 6.9-13.6x` from an instrument nobody could re-run — not even its author after a `cargo clean`.
`GATE-REACHABILITY`'s founding sentence applies verbatim: a check nothing invokes is
indistinguishable from a check that does not exist, and a measured number whose producer is
untracked is a *"trust me"*.

⛔ THE FAMILY PREDICATE IS IMPORTED, NOT COPIED — and promoting it verbatim would have shipped a
SECOND, ALREADY-DIVERGENT home. The scratch copy carried `_lr_(base|suffix|seed|guard)`; the tracked
predicate `.21` slice 1 derived from the two emission sites is
`_lr_(base|suffix|seed|guard|alt)(?![a-z])`. They had drifted apart the moment slice 1 fixed one of
them, which is the exact shape of the defect this leaf exists for. There is now one home —
`stimuli/sv/corpus_parse_cost.py:LR_FAMILY_RE` — and this file refuses if it cannot reach it.

⚠️ `.21` GAP 4 SHARPENED BY THIS PROMOTION. The self-assessment said *"the QUANTITIES are
independent, the classifier is SHARED"*. Measured while promoting: the classifiers were not shared,
they were DUPLICATED and had diverged — which is weaker still, because two copies can agree by
accident. Now they cannot differ.

Sampling traps this obeys, all four measured (TOOLBOX 3.8):
  TRAP 1  denominator = the WORKER thread's root count, never the process total.
  TRAP 2  inclusive sums count OUTERMOST family nodes only, or nested frames double-count.
  TRAP 3  per-family symbol names can be folded by the LINKER; checked separately (bl_callers.py).
  TRAP 4  ⛔ found by this script's own first version being WRONG — a `sample` report has FOUR
          sections and only the first is a call graph:

              Call graph:
              Total number in stack (recursive counted multiple, when >=5):   <- NOT a call graph
              Sort by top of stack, same collapsed (when >= 5):
              Binary Images:

          Ingesting section 2 as call-graph rows re-parents ~14 000 samples onto the last row of
          section 1 and produces NEGATIVE self-time — an 8x error (18.12 % vs 2.27 %). ⭐⭐ The
          conservation control `sum(self) == root` did NOT catch it, because a MISASSIGNMENT
          conserves the total. That is why the binding control below is an EXTERNAL oracle rather
          than an internal identity.

GROUND-TRUTH CONTROLS — the script REFUSES (exit 2) rather than publishing when one fails:
  C1  every parsed depth is an even offset from the section base (well-formed indent).
  C2  sum(self) over all rows == the sum of the thread root counts (conservation).
  C3  no row has negative self-time (catches re-parenting, which C2 cannot).
  C4  ⭐ THE ORACLE: for EVERY symbol `sample` itself lists in `Sort by top of stack`, this
      script's independently-derived per-symbol self-time must MATCH IT EXACTLY. ~195 symbols on a
      typical report. That is what makes the long tail below sample's >=5 threshold trustworthy —
      the part overlapping sample's own view is proven identical first.
  C5  at least one LR symbol is present (otherwise the family regex has gone stale).
  C6  NEW with the promotion: the predicate really came from the single tracked home.

Usage: lr_attribute.py <sample-report.txt> [<sample-report.txt> ...]
Reports are produced by `prof_run.sh` beside this file. ⚠️ They are 3.5-6 MB each and are NOT
tracked; the RUNNER is, so they are regenerable. Only the derived table is committed.
"""
import os
import re
import sys

# lr_profile → engine_universal_services → artifacts → tasks → docs → repository root (5 levels).
ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), *([".."] * 5)))

# ⛔ ONE HOME FOR THE FAMILY PREDICATE (C6). Import, never re-type.
sys.path.insert(0, os.path.join(ROOT, "stimuli", "sv"))
try:
    from corpus_parse_cost import LR_FAMILY_RE as LR
except Exception as exc:                                                  # pragma: no cover
    sys.exit(f"REFUSE: cannot import LR_FAMILY_RE from stimuli/sv/corpus_parse_cost.py: {exc}\n"
             "  This instrument must classify with the SAME predicate the ratchet reports, or the\n"
             "  two surfaces can disagree silently — which is the defect `.21` was opened for.")

# The predicate this instrument carried while it lived in `rust/target/audit_scratch/`. Kept ONLY so
# `lr_breakdown.py` can price the historical definitions against the live one; never used to classify.
HISTORICAL_SCRATCH_RE = re.compile(r"_lr_(base|suffix|seed|guard)")
HISTORICAL_TRACKED_RE = re.compile(r"_lr_base$|_lr_suffix(_r\d+)?$")

LINE = re.compile(r"^([ +!:|]*)(\d+) (.*)$")
STEP = 2
END_OF_CALL_GRAPH = ("Total number in stack", "Sort by top of stack", "Binary Images:")


def parse(path):
    """-> (worker_root, [(depth, count, symbol, is_worker)]) over the CALL GRAPH only."""
    rows, worker_root, in_graph, is_worker = [], None, False, False
    for raw in open(path, encoding="utf-8", errors="replace"):
        ln = raw.rstrip("\n")
        if ln.startswith("Call graph:"):
            in_graph = True
            continue
        if any(ln.startswith(p) for p in END_OF_CALL_GRAPH):
            in_graph = False
            continue
        if not in_graph:
            continue
        m = re.match(r"^ {4}(\d+) Thread_\d+(.*)$", ln)
        if m:
            is_worker = "com.apple.main-thread" not in m.group(2)
            if is_worker:
                worker_root = int(m.group(1))
            continue
        m = LINE.match(ln)
        if m:
            rows.append((len(m.group(1)), int(m.group(2)), m.group(3), is_worker))
    return worker_root, rows


def collapsed_view(path):
    """sample's OWN per-symbol self-time table (its `>= 5` collapse threshold applies)."""
    out, sec = {}, False
    for ln in open(path, encoding="utf-8", errors="replace"):
        if ln.startswith("Sort by top of stack"):
            sec = True
            continue
        if ln.startswith("Binary Images"):
            break
        if sec and ln.strip():
            p = ln.rsplit(None, 1)
            if len(p) == 2 and p[1].isdigit():
                out[p[0].strip().split("  (in ")[0]] = int(p[1])
    return out


def refuse(msg):
    sys.exit(f"REFUSE: {msg}")


def selves(path, rows):
    n = len(rows)
    if not n:
        refuse(f"{path}: no call-graph rows")
    base = rows[0][0]
    for d, _c, s, _w in rows:                                             # C1
        if (d - base) % STEP:
            refuse(f"{path}: ragged indent {d} (base {base}) at {s[:60]!r}")
    sc = [0] * n
    for i in range(n):
        d, c = rows[i][0], rows[i][1]
        kids, j = 0, i + 1
        while j < n and rows[j][0] > d:
            if rows[j][0] == d + STEP:
                kids += rows[j][1]
            j += 1
        sc[i] = c - kids
        if sc[i] < 0:                                                     # C3
            refuse(f"{path}: negative self {sc[i]} at depth {d} for {rows[i][2][:60]!r} "
                   f"— rows have been re-parented across a section boundary")
    return sc


def check_oracle(path, rows, sc):                                         # C4
    mine = {}
    for i, (_d, _c, s, _w) in enumerate(rows):
        sym = s.split("  (in ")[0]
        mine[sym] = mine.get(sym, 0) + sc[i]
    want = collapsed_view(path)
    bad = [(k, v, mine.get(k, 0)) for k, v in want.items() if mine.get(k, 0) != v]
    if bad:
        for k, v, m in bad[:5]:
            print(f"  sample={v} mine={m}  {k[-70:]}", file=sys.stderr)
        refuse(f"{path}: {len(bad)} of {len(want)} symbols disagree with sample's own "
               f"`Sort by top of stack` table")
    return len(want)


def attribute(path, family_re=LR):
    """-> dict of the published quantities for ONE report, every control enforced."""
    worker_root, rows = parse(path)
    if not worker_root:
        refuse(f"{path}: no worker thread found")
    sc = selves(path, rows)
    all_roots = sum(c for i, (d, c, _s, _w) in enumerate(rows) if d == rows[0][0])
    if sum(sc) != all_roots:                                              # C2
        refuse(f"{path}: self sums to {sum(sc)}, root rows sum to {all_roots}")
    n_oracle = check_oracle(path, rows, sc)

    widx = [i for i in range(len(rows)) if rows[i][3]]
    lr_idx = [i for i in widx if family_re.search(rows[i][2])]
    if not lr_idx:                                                        # C5
        refuse(f"{path}: no LR symbol matched — the family regex has gone stale")
    lr_self = sum(sc[i] for i in lr_idx)

    lr_incl, stack = 0, []
    for i in widx:
        d, c, s = rows[i][0], rows[i][1], rows[i][2]
        while stack and stack[-1][0] >= d:
            stack.pop()
        under = bool(stack) and stack[-1][1]
        is_lr = bool(family_re.search(s))
        if is_lr and not under:
            lr_incl += c
        stack.append((d, is_lr or under))

    return {"report": path.rsplit("/", 1)[-1], "root": worker_root, "oracle_syms": n_oracle,
            "lr_nodes": len(lr_idx), "self": lr_self, "incl": lr_incl,
            "self_pct": 100.0 * lr_self / worker_root, "incl_pct": 100.0 * lr_incl / worker_root,
            "ratio": (lr_incl / lr_self) if lr_self else float("nan")}


def main(argv):
    if not argv:
        sys.exit(__doc__)
    print(f"# predicate (imported, single home): {LR.pattern}")
    for path in argv:
        r = attribute(path)
        print(f"{r['report']:<10} root={r['root']:<6} oracle={r['oracle_syms']} syms OK"
              f"  LRnodes={r['lr_nodes']:<5}"
              f" self={r['self']:>5} ({r['self_pct']:5.2f}%)"
              f" incl={r['incl']:>5} ({r['incl_pct']:5.2f}%)"
              f" ratio={r['ratio']:.1f}x")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
