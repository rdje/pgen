#!/usr/bin/env python3
"""`SV-CORPUS-GRAD.13c.2b` — is this defect the SOLE blocker of its two corpus rows?

`.13c.2` attributed 2 DARK corpus rows to the constant size cast. An attribution is not a
measurement: a file can be blocked by several constructs at once, and "rows unblocked: 2" is then
an over-claim that only shows up when the fix lands and the number does not move.

This bisect answers it now, without a fix. It removes ONLY the numeric size-cast prefix (`N'`
before a `(`) from each row, changing nothing else, and re-probes:

  * before — REJECT (both rows)
  * after  — parse_full passed (both rows)

⇒ the numeric size cast is the sole remaining blocker of both files, so the fix flips exactly 2
rows and no fewer. The transformation is deliberately the SMALLEST one that removes the construct:
`512'({ … })` becomes `({ … })`, a legal SystemVerilog parenthesised concatenation, so the rest of
each file is still exercised at full strength.

  python3 docs/tasks/artifacts/sv_corpus_grad/constant_size_cast/corpus_row_cast_bisect.py
"""
from __future__ import annotations

import re
import subprocess
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from _repo_root import repo_root  # noqa: E402  (path is set immediately above)

ROWS = (
    "stimuli/sv/subs/opentitan/hw/top_darjeeling/rtl/autogen/testing/top_darjeeling_rnd_cnst_pkg.sv",
    "stimuli/sv/subs/opentitan/hw/top_earlgrey/rtl/autogen/testing/top_earlgrey_rnd_cnst_pkg.sv",
)
SIZE_CAST_PREFIX = re.compile(r"\b\d+'(?=\()")


def probe(root: Path, path: Path) -> str:
    binary = root / "rust/target/release/parseability_probe"
    if not binary.is_file():
        raise SystemExit(
            f"REFUSE: {binary} is missing — build it with:\n"
            "  (cd rust && cargo build --release --features generated_parsers --bin parseability_probe)"
        )
    result = subprocess.run(
        [str(binary), "--parse", "systemverilog", str(path), "--profile", "sv_2017"],
        capture_output=True,
        text=True,
    )
    tail = (result.stdout + result.stderr).strip().splitlines()[-1]
    return "ACCEPT" if tail.startswith("parse_full passed") else f"REJECT {tail.split('[')[-1].rstrip(']')}"


def main() -> int:
    root = repo_root()
    status = 0
    with tempfile.TemporaryDirectory(dir=root / "rust/target") as work:
        for relative in ROWS:
            source = root / relative
            if not source.is_file():
                raise SystemExit(f"REFUSE: corpus row {relative} is not present")
            text = source.read_text()
            stripped, removed = SIZE_CAST_PREFIX.subn("", text)
            if removed == 0:
                raise SystemExit(f"REFUSE: {relative} carries no numeric size cast — the row moved")
            after_path = Path(work) / source.name
            after_path.write_text(stripped)

            before = probe(root, source)
            after = probe(root, after_path)
            print(f"{source.name}")
            print(f"    size casts removed : {removed}")
            print(f"    before             : {before}")
            print(f"    after              : {after}")
            if not before.startswith("REJECT") or after != "ACCEPT":
                status = 1
                print("    ⛔ the bisect no longer holds — re-adjudicate this row")
    print()
    print(
        "VERDICT: the numeric size cast is the SOLE blocker of both rows"
        if status == 0
        else "VERDICT: a row moved — the 'unblocks exactly 2 rows' claim must be re-measured"
    )
    return status


if __name__ == "__main__":
    raise SystemExit(main())
