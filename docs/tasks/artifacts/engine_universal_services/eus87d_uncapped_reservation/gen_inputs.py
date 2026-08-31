#!/usr/bin/env python3
"""`ENGINE-UNIVERSAL-SERVICES.87`(d) — regenerate the cast inputs used to prove
the UPWARD half of the capability mandate.

`.69`(d)'s generator stops at 5 200 nested casts because that was past the
boundary at the 256 MiB reference. Lifting `.69`(a)'s upper clamp moves the
boundary on any host above 16 GiB, so the ladder needs rungs ABOVE it: this
host (24 GiB) derives 384 MiB, and 384 MiB x 18.82 levels/MiB predicts a
boundary near 7 220.

The inputs are NOT tracked (mechanical, up to ~90 KB each); this generator is,
so every measurement in the leaf is reproducible from the repository alone.
Paths are relative to the repository root by directive 12.

  python3 docs/tasks/artifacts/engine_universal_services/eus87d_uncapped_reservation/gen_inputs.py <outdir>
"""
import os
import sys

# Nested `int'(...)` casts — the shape `.61`(a)'s shipped ceiling label was
# captured on, and the one `.69`(d) established REACHES the ceiling at a
# practical cost. 4814 is `.69`(d)'s own rung, reproduced here so the two
# measurements share an input; the rest bracket the predicted 384 MiB boundary.
CASTS = (4814, 6000, 7000, 7600, 8400)


def main() -> int:
    out = sys.argv[1] if len(sys.argv) > 1 else "preserved_probes/eus87d"
    os.makedirs(out, exist_ok=True)
    for n in CASTS:
        body = "int'(" * n + "1" + ")" * n
        with open(os.path.join(out, f"cast_{n}.sv"), "w") as fh:
            fh.write(f"module m;\n  assign x = {body};\nendmodule\n")
    print(f"wrote {len(CASTS)} cast inputs to {out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
