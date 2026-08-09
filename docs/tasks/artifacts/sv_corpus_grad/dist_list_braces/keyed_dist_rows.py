#!/usr/bin/env python3
"""Which `unexplained_rejects_valid` rows are blocked INSIDE a `dist { … }`? (`SV-CORPUS-GRAD.3.18`)

This is the leaf's TARGETING instrument, in the sense `.3.8` established: the set it prints
before the fix is the set the fix is allowed to move, so "flipped-but-not-keyed = 0" is
checkable rather than asserted.

⛔ WHY NOT `sweep_begin_family.py`. That instrument's contract is a construct that stays
REJECTED — its positive control is pinned to a rejection offset, which is exactly what a
grammar fix invalidates. `dist` is the opposite case: the whole point is that the keyed rows
start parsing. A "positive control" for this family therefore cannot survive its own leaf, so
forcing `dist` into that instrument would have meant either a control that breaks on landing
or an assertion quietly relaxed. Two instruments with honest contracts beat one with a
loophole ([[feedback_instrument_needs_ground_truth]]).

The predicate is positional, never textual: a row is keyed iff the nearest UNCLOSED `{`
before the stuck position is immediately preceded — modulo white space — by the word-bounded
keyword `dist`. A row that merely mentions `dist` somewhere, or that dies before reaching it,
is not keyed.

⚠️ HONEST BOUND, stated rather than papered over: the brace scan is lexical, so a `{` inside
a string literal or a comment between the `dist` and the stuck position would mis-nest it.
The two controls below are what stop that from being a silent assumption — and the binding
no-regression proof for this leaf is the GLOBAL per-file pass-set diff (the `.3.4` LAW), not
this scan.

Usage (from anywhere; relative paths resolve against the repository root, directive 12):
  python3 docs/tasks/artifacts/sv_corpus_grad/dist_list_braces/keyed_dist_rows.py
"""

import argparse
import re
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from _repo_root import repo_root  # noqa: E402

ROOT = repo_root()
PROBE = ROOT / "rust/target/release/parseability_probe"
SUBS = ROOT / "stimuli/sv/subs"
REPRO = Path(__file__).resolve().parent / "repro"
POS_RE = re.compile(r"furthest_position=(\d+)")

IDENT_BYTE = re.compile(rb"[A-Za-z0-9_$]")


def probe(path: Path, profile: str):
    """-> int furthest_position | None (parsed clean) | 'timeout' | 'unparseable'."""
    try:
        r = subprocess.run(
            [str(PROBE), "--parse", "systemverilog", str(path), "--profile", profile],
            capture_output=True, text=True, timeout=90)
    except subprocess.TimeoutExpired:
        return "timeout"
    out = r.stdout + r.stderr
    if "parse_full passed" in out:
        return None
    m = POS_RE.search(out)
    return int(m.group(1)) if m else "unparseable"


def blocked_inside_dist_braces(data: bytes, pos: int) -> bool:
    """Is the nearest unclosed `{` before `pos` the one opened by a `dist`?"""
    depth = 0
    i = pos - 1
    while i >= 0:
        c = data[i:i + 1]
        if c == b"}":
            depth += 1
        elif c == b"{":
            if depth == 0:
                j = i
                while j > 0 and data[j - 1:j].isspace():
                    j -= 1
                if data[max(0, j - 4):j] != b"dist":
                    return False
                before = data[j - 5:j - 4] if j >= 5 else b""
                return not IDENT_BYTE.match(before) if before else True
            depth -= 1
        i -= 1
    return False


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--manifest", type=Path,
                    default=ROOT / "stimuli/sv/characterization/adjudication_manifest.tsv")
    ap.add_argument("--profile", default="sv_2017")
    args = ap.parse_args()

    if not PROBE.is_file():
        raise SystemExit(f"REFUSE: probe not built at {PROBE.relative_to(ROOT)}")

    # Ground truth, both directions, from CONSTRUCTED files — never a corpus row
    # ([[feedback_ground_truth_control_must_not_pin_untracked_state]]). These assert the
    # PREDICATE against known positions, not the parser, so they hold on both sides of the
    # fix and this instrument stays re-runnable after landing.
    pos_src = (REPRO / "d1_dist_weighted.sv").read_bytes()
    pos_at = pos_src.index(b":= 1")
    if not blocked_inside_dist_braces(pos_src, pos_at):
        raise SystemExit("REFUSE: positive control not keyed (inside `dist {`)")
    neg_src = (REPRO / "c2_concat_expression.sv").read_bytes()
    neg_at = neg_src.index(b"4'd8")
    if blocked_inside_dist_braces(neg_src, neg_at):
        raise SystemExit("REFUSE: a plain concatenation was keyed as a dist list")
    print(f"controls OK: +d1_dist_weighted@{pos_at}, -c2_concat_expression@{neg_at}",
          file=sys.stderr)

    manifest = args.manifest if args.manifest.is_absolute() else ROOT / args.manifest
    rows = []
    with manifest.open() as fh:
        next(fh)
        for line in fh:
            f = line.rstrip("\n").split("\t")
            if len(f) >= 5 and f[4] == "divergence:unexplained_rejects_valid":
                rows.append((f[0], f[1]))
    print(f"sweeping {len(rows)} unexplained_rejects_valid rows "
          f"from {manifest.relative_to(ROOT)}", file=sys.stderr)

    def verdict(row):
        suite, rel = row
        path = SUBS / suite / rel
        fp = probe(path, args.profile)
        if not isinstance(fp, int):
            return (suite, rel, fp, False)
        try:
            data = path.read_bytes()
        except OSError as exc:
            return (suite, rel, f"unreadable: {exc}", False)
        return (suite, rel, fp, blocked_inside_dist_braces(data, fp))

    with ThreadPoolExecutor(max_workers=8) as ex:
        results = list(ex.map(verdict, rows))

    hits = sorted(r for r in results if r[3])
    print(f"blocked inside `dist {{ … }}`: {len(hits)} / {len(rows)} "
          f"(profile {args.profile})")
    for suite, rel, fp, _ in hits:
        print(f"  {suite}\t{rel}\tfurthest={fp}")

    odd = sorted((r for r in results if not isinstance(r[2], int)), key=lambda r: r[:2])
    print(f"\nnon-integer probe results: {len(odd)}")
    for suite, rel, fp, _ in odd:
        print(f"  {suite}\t{rel}\t{fp}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
