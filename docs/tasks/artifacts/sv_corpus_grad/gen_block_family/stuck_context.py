#!/usr/bin/env python3
"""Per-row stuck-point CONTEXT dump for the generate-block adjudication
(`SV-CORPUS-GRAD.3.15`).

`sweep_begin_family.py` answers *which* rows are in the family. This answers *what the
parser was looking at* when it stopped, which is what an LRM adjudication actually needs:
a row is only `must_reject` if the construct AT the stuck point is the illegal one, and
that cannot be read off a row count.

Ground truth ([[feedback_instrument_needs_ground_truth]]) — REFUSES rather than reporting
on a miss:
  negative  repro/C_labelled_begin_in_for.sv        MUST parse clean (a legal generate_block)
  positive  repro/B_labelled_begin_in_generate.sv   MUST reject at byte 30

Usage — reads `suite<TAB>relpath` rows on stdin:
  cut -f1,2 docs/tasks/artifacts/sv_corpus_grad/gen_block_family/family_rows.tsv \\
    | python3 docs/tasks/artifacts/sv_corpus_grad/gen_block_family/stuck_context.py
"""

import re
import subprocess
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from _repo_root import repo_root  # noqa: E402

ROOT = repo_root()
PROBE = ROOT / "rust/target/release/parseability_probe"
SUBS = ROOT / "stimuli/sv/subs"
REPRO = Path(__file__).resolve().parent / "repro"
POS_RE = re.compile(r"furthest_position=(\d+)")


def probe(path: Path):
    r = subprocess.run(
        [str(PROBE), "--parse", "systemverilog", str(path), "--profile", "sv_2017"],
        capture_output=True, text=True)
    out = r.stdout + r.stderr
    if "parse_full passed" in out:
        return None
    m = POS_RE.search(out)
    if not m:
        raise SystemExit(f"REFUSE: unparseable probe output for {path}:\n{out[:400]}")
    return int(m.group(1))


def context(path: Path, pos: int, before: int = 80, after: int = 40):
    data = path.read_bytes()
    line = data[:pos].count(b"\n") + 1
    return (line,
            data[max(0, pos - before):pos].decode("utf8", "replace"),
            data[pos:pos + after].decode("utf8", "replace"))


def main() -> int:
    if probe(REPRO / "C_labelled_begin_in_for.sv") is not None:
        raise SystemExit("REFUSE: negative control (legal for-generate block) did not pass")
    if probe(REPRO / "B_labelled_begin_in_generate.sv") != 30:
        raise SystemExit("REFUSE: positive control did not reject at the pinned position 30")
    print("controls OK: negative passes; positive rejects at 30\n")

    for raw in sys.stdin:
        raw = raw.rstrip("\n")
        if not raw:
            continue
        suite, rel = raw.split("\t")[:2]
        path = SUBS / suite / rel
        fp = probe(path)
        if fp is None:
            print(f"{suite}/{rel}\n  *** NOW PASSES ***\n")
            continue
        line, pre, post = context(path, fp)
        print(f"{suite}/{rel}  furthest={fp} line={line}")
        print(f"  consumed ...{pre!r}")
        print(f"  stuck at >>>{post!r}\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
