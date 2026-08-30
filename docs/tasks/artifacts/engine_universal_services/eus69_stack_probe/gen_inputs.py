#!/usr/bin/env python3
"""`ENGINE-UNIVERSAL-SERVICES.69`(d) — regenerate the ladder inputs.

The inputs themselves are NOT tracked (they are mechanical and up to 64 KB
each); this generator is, so every measurement in the leaf is reproducible from
the repository alone. Paths are relative to the repository root by directive 12.

  python3 docs/tasks/artifacts/engine_universal_services/eus69_stack_probe/gen_inputs.py <outdir>
"""
import os
import sys

PARENS = (100, 200, 500, 1000, 1500, 2000, 2100, 2200, 2400, 2600, 2800, 2900,
          3000, 3500, 4000, 8000, 16000, 32000)
# `int'(...)` nested casts — the shape `.61`(a)'s shipped ceiling label was
# captured on, and the only one of the two that REACHES the ceiling at the
# 256 MiB default within a practical time budget.
CASTS = (4000, 4814, 5200)


def main() -> int:
    out = sys.argv[1] if len(sys.argv) > 1 else "preserved_probes/eus69"
    os.makedirs(out, exist_ok=True)
    for n in PARENS:
        body = "(" * n + "1" + ")" * n
        with open(os.path.join(out, f"deep_{n}.sv"), "w") as fh:
            fh.write(f"module m;\n  localparam int p = {body};\nendmodule\n")
    for n in CASTS:
        body = "int'(" * n + "1" + ")" * n
        with open(os.path.join(out, f"cast_{n}.sv"), "w") as fh:
            fh.write(f"module m;\n  assign x = {body};\nendmodule\n")
    print(f"wrote {len(PARENS)} paren + {len(CASTS)} cast inputs to {out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
