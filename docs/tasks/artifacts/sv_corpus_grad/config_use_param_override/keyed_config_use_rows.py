#!/usr/bin/env python3
"""Which `unexplained_rejects_valid` rows are blocked at a config `use #( … )`? (`SV-CORPUS-GRAD.3.19`)

This is the leaf's TARGETING instrument in the sense `.3.8`/`.3.18` established: the set it
prints BEFORE the fix is the set the fix is allowed to move, so "flipped-but-not-keyed = 0"
and "keyed-but-not-flipped = 0" are checkable rather than asserted.

⛔ WHY A POSITIONAL SCAN RATHER THAN THE FAMILY BUCKETER. The coarse family classifier files
these rows under `config/library (ch33)` together with library-map and `liblist` rows, and the
3-token cluster table splits them across TWO signatures (`# ( .` and `# ( )`) purely on whether
the override list is empty. Neither view is the construct. The predicate below is positional
and construct-shaped, so it names exactly one mechanism.

The predicate: a row is keyed iff its stuck position lies inside a `config … endconfig` span
AND the nearest word-bounded `use` keyword at or before that position is followed — modulo
white space — by a `#`. A row that merely contains the word `config`, or that dies at a `#(`
belonging to an ordinary module instantiation, is NOT keyed.

⚠️ HONEST BOUND, stated rather than papered over: the span scan is lexical, so the words
`config`/`endconfig`/`use` inside a string literal or a comment would mis-nest it. The three
controls below are what stop that from being a silent assumption — and the binding
no-regression proof for this leaf is the GLOBAL per-file pass-set diff (the `.3.4` LAW), not
this scan.

⭐ Unlike `.3.8`'s `sweep_begin_family.py`, this instrument's controls are pinned to
CONSTRUCTED repro files and assert the PREDICATE, not the parser, so it stays re-runnable
after the fix lands ([[feedback_instrument_needs_ground_truth]]).

Usage (from anywhere; relative paths resolve against the repository root, directive 12):
  python3 docs/tasks/artifacts/sv_corpus_grad/config_use_param_override/keyed_config_use_rows.py
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

CONFIG_RE = re.compile(rb"\bconfig\b")
ENDCONFIG_RE = re.compile(rb"\bendconfig\b")
USE_RE = re.compile(rb"\buse\b")


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


def inside_config_block(data: bytes, pos: int) -> bool:
    """Is `pos` inside some `config … endconfig` span?

    `endconfig` also matches `\\bconfig\\b`? No — the word boundary at the LEFT of `config`
    fails inside `endconfig`, so the two scans do not overlap and a naive nearest-opener
    search is sound.
    """
    for m in CONFIG_RE.finditer(data):
        e = ENDCONFIG_RE.search(data, m.start())
        if e and m.start() <= pos <= e.end():
            return True
    return False


def use_hash_before(data: bytes, pos: int) -> bool:
    """Is the nearest `use` keyword at/before `pos` followed (modulo space) by a `#`?"""
    last = None
    for m in USE_RE.finditer(data, 0, pos + 1):
        last = m
    if last is None:
        return False
    j = last.end()
    while j < len(data) and data[j:j + 1].isspace():
        j += 1
    return data[j:j + 1] == b"#"


def keyed(data: bytes, pos: int) -> bool:
    return inside_config_block(data, pos) and use_hash_before(data, pos)


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--manifest", type=Path,
                    default=ROOT / "stimuli/sv/characterization/adjudication_manifest.tsv")
    ap.add_argument("--profile", default="sv_2017")
    args = ap.parse_args()

    if not PROBE.is_file():
        raise SystemExit(f"REFUSE: probe not built at {PROBE.relative_to(ROOT)}")

    # Ground truth, BOTH directions, from CONSTRUCTED files — never a corpus row
    # ([[feedback_ground_truth_control_must_not_pin_untracked_state]]). These assert the
    # PREDICATE against known offsets, not the parser, so they hold on both sides of the fix.
    pos_src = (REPRO / "h1_hash_named.sv").read_bytes()
    pos_at = pos_src.index(b"#(.WIDTH")
    if not keyed(pos_src, pos_at):
        raise SystemExit("REFUSE: positive control not keyed (config `use #(`)")

    neg1_src = (REPRO / "a3_use_lib_cell_config.sv").read_bytes()
    neg1_at = neg1_src.index(b":config")
    if keyed(neg1_src, neg1_at):
        raise SystemExit("REFUSE: a brace-less `use lib.cell:config` was keyed")

    neg2_src = (REPRO / "c3_module_param_inst.sv").read_bytes()
    neg2_at = neg2_src.index(b"#(8, 16)")
    if keyed(neg2_src, neg2_at):
        raise SystemExit("REFUSE: an ordinary module `#()` instantiation was keyed")

    print(f"controls OK: +h1@{pos_at}, -a3@{neg1_at}, -c3@{neg2_at}", file=sys.stderr)

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
        return (suite, rel, fp, keyed(data, fp))

    with ThreadPoolExecutor(max_workers=8) as ex:
        results = list(ex.map(verdict, rows))

    hits = sorted(r for r in results if r[3])
    print(f"blocked at a config `use #( … )`: {len(hits)} / {len(rows)} "
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
