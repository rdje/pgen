#!/usr/bin/env python3
"""`SV-CORPUS-GRAD.13c.2b` — do the two attributed corpus rows still parse, and if not, WHY?

⭐ RE-PURPOSED 2026-08-23 (`PGEN-SV-CORPUS-GRAD-0279`), because its original question is answered
and can never be asked again. It opened as a PRICING bisect: `.13c.2` had *attributed* 2 DARK
corpus rows to the constant size cast, and an attribution is not a measurement — a file can be
blocked by several constructs at once, so "rows unblocked: 2" would only be caught being wrong when
the fix landed and the number did not move. It removed only the numeric size-cast prefix (`N'`
before a `(`), changing nothing else, and measured `before REJECT -> after ACCEPT` on both rows:
the cast was the SOLE remaining blocker, so the fix would flip exactly 2 rows and no fewer.

⛔ THE FIX LANDED (`ENGINE-UNIVERSAL-SERVICES.17` slice 9,
`PGEN-ENGINE-UNIVERSAL-SERVICES-0032`) AND THE PREMISE IS VOID: both rows now parse WITH their
casts intact, so "before" can never be a rejection again and the script asserting it was RED on a
correct tree. A pricing instrument outlives its question; a REGRESSION instrument does not.

So it now measures the same two arms and reads them as a DIFFERENTIAL, which is strictly more than
the boolean it replaced:

  | as shipped (casts intact) | casts stripped | verdict                                          |
  |---------------------------|----------------|--------------------------------------------------|
  | ACCEPT                    | ACCEPT         | ✅ the construct parses — the post-fix baseline   |
  | REJECT                    | ACCEPT         | ⛔ the SIZE CAST regressed — removing it recovers |
  | REJECT                    | REJECT         | ⛔ something ELSE in the row broke — not the cast |

⇒ a future red run does not merely say "a row moved", it says whether the numeric size cast is the
thing that moved. The stripped arm is the control that separates those two failures, which is why
it is kept now that it is no longer the measurement.

⛔ AND IT STILL REFUSES IF A ROW LOSES ITS CASTS. Both rows are VENDORED OpenTitan sources; a
re-vendor that dropped the construct would leave two arms passing for a reason that has nothing to
do with this defect — a test that cannot fail, reported as a test that passed. The `removed == 0`
refusal is what keeps the ACCEPT meaningful.

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
            stripped_text, removed = SIZE_CAST_PREFIX.subn("", text)
            if removed == 0:
                raise SystemExit(f"REFUSE: {relative} carries no numeric size cast — the row moved")
            after_path = Path(work) / source.name
            after_path.write_text(stripped_text)

            shipped = probe(root, source)
            stripped = probe(root, after_path)
            print(f"{source.name}")
            print(f"    numeric size casts : {removed}")
            print(f"    as shipped         : {shipped}")
            print(f"    casts stripped     : {stripped}")
            if shipped == "ACCEPT" and stripped == "ACCEPT":
                continue
            status = 1
            if stripped == "ACCEPT":
                print(
                    "    ⛔ REGRESSION IN THE SIZE CAST: the row parses once the N' prefixes are"
                    " removed, so the construct ENGINE-UNIVERSAL-SERVICES.17 slice 9 admitted is"
                    " what stopped parsing. Check left_recursion_unhandled=0 on"
                    " grammars/systemverilog.ebnf and that no build passes"
                    " --indirect-lr-admit-starvation-safe-only."
                )
            else:
                print(
                    "    ⛔ NOT THE CAST: the row fails with the casts removed too, so the blocker"
                    " is some OTHER construct in this file. Re-probe it with"
                    " parseability_probe --parse systemverilog <row> --profile sv_2017 and read"
                    " furthest_position — this leaf is not the owner."
                )
    print()
    print(
        "VERDICT: both rows parse as shipped, casts intact — the post-fix baseline"
        " (PGEN-ENGINE-UNIVERSAL-SERVICES-0032)"
        if status == 0
        else "VERDICT: a row moved — read the per-row line above; it says WHETHER the cast is the cause"
    )
    return status


if __name__ == "__main__":
    raise SystemExit(main())
