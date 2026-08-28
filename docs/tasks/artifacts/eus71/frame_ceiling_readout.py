#!/usr/bin/env python3
"""ENGINE-UNIVERSAL-SERVICES.71 — the SECOND AXIS for "the recursion ceiling is
host-derived", read out of the parser instead of inferred from my own bisection.

`reservation_scaling.py` measures SOURCE LEVELS by binary search on the accept/reject
boundary. That is one quantity, produced by one method. The guard itself reports a
different one in its rejection text — the number of PARSER FRAMES this host's stack could
carry ("this host's parse stack ran out at N parser frames") — and nothing in the
bisection produces or consumes it.

Two independent predictions follow, and a constant ceiling satisfies neither:

  (1) WITHIN a shape, frames must be linear in the reservation, with the same flatness the
      levels table shows. levels/MiB and frames/MiB are then two readings of one bound.
  (2) ACROSS REGIMES, the per-level frame cost must agree with what `.63`(b) measured under
      the OLD 4096-frame ceiling — a different mechanism, a different binary, a different
      slice, so an oracle this slice did not build.

      ⚠ The comparable quantity is the SLOPE of frames against depth, NOT the quotient
      `frames_per_level` this script reports. frames = a + b*depth makes the quotient
      (a + b*depth)/depth drift with depth, so two regimes measured at very different depths
      disagree on the quotient even when b is identical. `ratio_vs_63b` below is therefore a
      screening number, not a verdict: `proportionality_fit.py` does the slope fit and
      attributes the difference. Measured: the quotient differs by +3.55 % on `cast_paren`
      and +0.48 % on `gen_mux_both_arms`, and both are the intercept showing through a ratio.

Usage: python3 docs/tasks/artifacts/eus71/frame_ceiling_readout.py <shape> <MiB> <first_bad_depth> [...]
       (shape/MiB/depth triples; depths come from reservation_scaling.py's `first_bad`)
"""
import subprocess, sys, os, re, json

PROBE = os.environ.get("PGEN_PARSE_PROBE_BIN", "./rust/target/release/parseability_probe")
TMP = "tmp/eus71/frames"
os.makedirs(TMP, exist_ok=True)

# Kept identical to reservation_scaling.py's SHAPES so the two producers cannot drift apart.
SHAPES = {
    "cast_paren": ("systemverilog", "sv", ["--profile", "sv_2017"],
                   lambda d: "module m; assign x = " + "int'("*d + "1" + ")"*d + "; endmodule\n"),
    "gen_mux_both_arms": ("systemverilog", "sv", ["--profile", "sv_2017"],
                   lambda d: "module m; assign x = " + "(s ? a : ("*d + "b" + "))"*d + "; endmodule\n"),
}

# `.63`(b), measured under the OLD fixed ceiling as 4096 / deepest-accepted. NOT re-derived
# here on purpose: an oracle recomputed by this slice would stop being independent of it.
FRAMES_PER_LEVEL_63B = {"cast_paren": 4096 / 249, "gen_mux_both_arms": 4096 / 146}

FRAMES_RE = re.compile(r"ran out at (\d+) parser frames")


def readout(shape, mib, depth):
    fam, ext, extra, gen = SHAPES[shape]
    p = f"{TMP}/{shape}_{mib}_{depth}.{ext}"
    with open(p, "w") as fh:
        fh.write(gen(depth))
    env = dict(os.environ)
    env["PGEN_PARSE_STACK_BYTES"] = str(mib * 1024 * 1024)
    r = subprocess.run([PROBE, "--parse", fam, p] + extra,
                       capture_output=True, timeout=1800, env=env)
    out = (r.stdout + r.stderr).decode(errors="replace")
    m = FRAMES_RE.search(out)
    return {
        "shape": shape, "reservation_MiB": mib, "first_bad_depth": depth,
        # A run that ACCEPTED, or rejected for any other reason, is rejected as evidence:
        # the frame number is only meaningful when the guard is what stopped the parse.
        "ceiling_labelled": "recursion ceiling CONTACTED" in out,
        "frames": int(m.group(1)) if m else None,
    }


def main():
    a = sys.argv[1:]
    if not a or len(a) % 3:
        sys.exit(__doc__)
    rows = []
    for shape, mib, depth in zip(a[0::3], map(int, a[1::3]), map(int, a[2::3])):
        row = readout(shape, mib, depth)
        if row["frames"]:
            row["frames_per_MiB"] = round(row["frames"] / mib, 2)
            row["frames_per_level"] = round(row["frames"] / depth, 3)
            # Screening only — see the module docstring: this quotient carries the
            # intercept, so a deviation here is a prompt to run the slope fit, not a finding.
            row["ratio_vs_63b_screening"] = round(
                row["frames_per_level"] / FRAMES_PER_LEVEL_63B[shape], 4)
        rows.append(row)
        print(json.dumps(row), flush=True)
    print()
    print(f"{'shape':>18} {'MiB':>5} {'depth':>7} {'frames':>8} {'frames/MiB':>11} "
          f"{'frames/level':>13} {'screen63b':>10} {'labelled':>9}")
    for r in rows:
        print(f"{r['shape']:>18} {r['reservation_MiB']:>5} {r['first_bad_depth']:>7} "
              f"{str(r['frames']):>8} {str(r.get('frames_per_MiB','-')):>11} "
              f"{str(r.get('frames_per_level','-')):>13} "
              f"{str(r.get('ratio_vs_63b_screening','-')):>10} {str(r['ceiling_labelled']):>9}")
    with open("tmp/eus71/frame_ceiling_readout.json", "w") as fh:
        json.dump(rows, fh, indent=1)


if __name__ == "__main__":
    main()
