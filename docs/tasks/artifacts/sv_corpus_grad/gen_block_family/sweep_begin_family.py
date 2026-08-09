#!/usr/bin/env python3
"""Sweep EVERY `unexplained_rejects_valid` row for the bare-generate-block construct
(`SV-CORPUS-GRAD.3.15`).

⛔ WHY A SWEEP AND NOT THE FAMILY MAP. `classify_rejects_valid_families.py` buckets on the
STUCK LINE's surface text, so it can only see a row whose stuck line literally reads
`begin : label`. A row blocked by the same construct but whose stuck line reads `end`, a
declaration, or a continuation is invisible to it. The fix scope must be decided from the
STUCK POSITION over the whole population, never from the coarse bucketer's output — that is
the difference between fixing a construct and fixing whatever the bucketer happened to name.

The predicate is deliberately positional, not textual: a row is in the family iff the parse
is blocked with `begin` as the last token consumed (stuck at its `: label`) or blocked
exactly AT a `begin` keyword.

Ground truth ([[feedback_instrument_needs_ground_truth]]) — both controls run before any
number is printed, and the script REFUSES rather than reporting on a miss:
  positive  repro/B_labelled_begin_in_generate.sv  MUST classify into the family
  negative  Surelog/tests/InsideOp/dut.sv          MUST NOT (it is stuck at `inside`,
                                                    a different defect, and the coarse
                                                    bucketer DID mis-file it here)

Usage (from the repository root, any location on disk):
  python3 docs/tasks/artifacts/sv_corpus_grad/gen_block_family/sweep_begin_family.py
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
TRAILING_WORD_RE = re.compile(rb"[A-Za-z_$][A-Za-z0-9_$]*\Z")


def strip_trailing_trivia(consumed: bytes) -> bytes:
    """Drop the trailing run of whitespace and comments, so the caller sees a real token.

    ⛔ Deliberately a backwards SCAN, not a regex. The obvious spelling —
    `re.sub(rb"(?:\\s+|//[^\\n]*|/\\*.*?\\*/)+\\Z", b"", consumed)` — nests two
    quantifiers under an anchored `+` and backtracks catastrophically: measured
    spinning at 100 % CPU for over three minutes on this corpus with no probe
    subprocess running at all, i.e. the instrument looked BUSY rather than broken.
    This loop is linear and terminates on every input.
    """
    i = len(consumed)
    while i > 0:
        j = i
        while j > 0 and consumed[j - 1:j].isspace():
            j -= 1
        if j != i:
            i = j
            continue
        # `... */`  — a block comment ending here
        if i >= 2 and consumed[i - 2:i] == b"*/":
            start = consumed.rfind(b"/*", 0, i - 2)
            if start == -1:
                break
            i = start
            continue
        # `// ...`  — a line comment is only trailing trivia when nothing but the
        # comment sits between the previous newline and here.
        nl = consumed.rfind(b"\n", 0, i)
        line_start = nl + 1
        slashes = consumed.find(b"//", line_start, i)
        if slashes != -1:
            i = slashes
            continue
        break
    return consumed[:i]


def probe(path: Path, profile: str = "sv_2017"):
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


def _stuck_at_keyword(at: bytes, kw: bytes) -> bool:
    tail = at[len(kw):len(kw) + 1]
    return at.startswith(kw) and not tail.isalnum() and tail != b"_"


def in_family(path: Path, pos: int, family: str = "begin") -> bool:
    """Is the parse blocked at the construct this family names?

    ⛔ PREDICATES, NOT COPIES. `.3.16` needed the same sweep over a different stuck token,
    and the cheap answer — copy the file and edit one string — is the defect this repo has a
    record for (docs/knowledge/a-copied-diagnostic-covers-only-where-it-was-pasted.md). The
    probe, the trivia scanner, the controls and the manifest walk are family-neutral; only
    the predicate is not, so only the predicate is selectable.
    """
    data = path.read_bytes()
    at = data[pos:pos + 64].lstrip()
    if family == "inside":
        # `inside` is stuck-AT only: the expression to its left parsed fine, and it is the
        # keyword itself that has no production in this context.
        return _stuck_at_keyword(at, b"inside")
    if family == "begin":
        consumed = strip_trailing_trivia(data[:pos])
        last_word = TRAILING_WORD_RE.search(consumed)
        # Either `begin` was consumed and the parse died on its `: label`, or it died AT the
        # keyword. Both are the same absent production.
        stuck_after_begin = bool(last_word) and last_word.group(0) == b"begin"
        return stuck_after_begin or _stuck_at_keyword(at, b"begin")
    raise SystemExit(f"REFUSE: unknown family {family!r}")


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    # Parameterized rather than forked, so the v2005 lane is answered by THIS instrument.
    # A second copy would only ever cover the lane it was pasted into
    # (docs/knowledge/a-copied-diagnostic-covers-only-where-it-was-pasted.md).
    ap.add_argument("--manifest", type=Path,
                    default=ROOT / "stimuli/sv/characterization/adjudication_manifest.tsv")
    ap.add_argument("--profile", default="sv_2017")
    ap.add_argument("--family", choices=("begin", "inside"), default="begin",
                    help="which stuck-point predicate to sweep for")
    args = ap.parse_args()

    if not PROBE.is_file():
        raise SystemExit(f"REFUSE: probe not built at {PROBE.relative_to(ROOT)}")

    # ⛔ CONTROLS ARE CONSTRUCTED AND TRACKED, never a corpus row. The first cut pinned the
    # `inside` control to `Surelog/tests/InsideOp/dut.sv`, which is SV-only source: under
    # `verilog_2005` it stops at `package`, not at `inside`, so the whole v2005 sweep REFUSED.
    # That refusal was the instrument behaving correctly — and the fix is to construct the
    # state being observed rather than to relax the assertion
    # ([[feedback_ground_truth_control_must_not_pin_untracked_state]]).
    #
    # The two families are each other's NEGATIVE control: both positives are rejections, so a
    # predicate that answered "yes" to any rejection is caught here rather than reported.
    positives = {
        # The bare generate_block is illegal in BOTH editions and the control text is pure
        # 1364, so one file serves both profiles at the same pinned offset.
        ("begin", "sv_2017"): (REPRO / "B_labelled_begin_in_generate.sv", 30),
        ("begin", "verilog_2005"): (REPRO / "B_labelled_begin_in_generate.sv", 30),
        ("inside", "sv_2017"): (REPRO / "E_inside_in_constant_expression.sv", 56),
        # `inside` does not exist in IEEE 1364-2005 at all, so the v2005 control is pure
        # 1364 text plus the one keyword — it rejects AT `inside` under verilog_2005 and
        # PASSES under sv_2017, which is itself the edition evidence.
        ("inside", "verilog_2005"): (REPRO / "G_inside_v2005_pure.sv", 43),
    }
    key = (args.family, args.profile)
    if key not in positives:
        raise SystemExit(
            f"REFUSE: no constructed control for family {args.family!r} under profile "
            f"{args.profile!r} — add one to repro/ rather than sweeping uncontrolled")
    pos_path, pos_pin = positives[key]
    pos_at = probe(pos_path, args.profile)
    if pos_at != pos_pin:
        raise SystemExit(f"REFUSE: positive control {pos_path.name} rejected at {pos_at}, "
                         f"pinned {pos_pin}")
    if not in_family(pos_path, pos_at, args.family):
        raise SystemExit(f"REFUSE: positive control not classified into `{args.family}`")

    other = "inside" if args.family == "begin" else "begin"
    neg_key = (other, args.profile)
    if neg_key in positives:
        neg_path, neg_pin = positives[neg_key]
        if in_family(neg_path, neg_pin, args.family):
            raise SystemExit(f"REFUSE: `{other}` control misclassified into `{args.family}`")
        neg_note = neg_path.name
    else:
        neg_note = "none available for this profile"
    print(f"controls OK (family {args.family}, profile {args.profile}): "
          f"+{pos_path.name}@{pos_at}, -{neg_note}", file=sys.stderr)

    # A relative --manifest is resolved against the repository root, never the caller's cwd
    # (directive 12): the same command line must mean the same thing from any directory.
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
        return (suite, rel, fp, in_family(path, fp, args.family))

    with ThreadPoolExecutor(max_workers=8) as ex:
        results = list(ex.map(verdict, rows))

    hits = sorted(r for r in results if r[3])
    print(f"`{args.family}` family: {len(hits)} / {len(rows)} rows")
    for suite, rel, fp, _ in hits:
        print(f"  {suite}\t{rel}\tfurthest={fp}")

    odd = sorted((r for r in results if not isinstance(r[2], int)), key=lambda r: r[:2])
    print(f"\nnon-integer probe results: {len(odd)}")
    for suite, rel, fp, _ in odd:
        print(f"  {suite}\t{rel}\t{fp}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
