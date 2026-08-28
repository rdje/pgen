#!/usr/bin/env python3
"""ENGINE-UNIVERSAL-SERVICES.65 — WHAT DOES THE NEW CAPACITY COST?

Raising the ceiling makes deeper input ACCEPTED; it does not make it cheap. This harness
times the SAME shape at a ladder of depths and reports wall time and peak RSS per depth,
so the leaf can publish what a consumer actually pays instead of asserting the capability
and stopping there.

Two ladders per shape:
  * ACCEPT ladder  — depths below the boundary; the cost of the capability itself.
  * REJECT probe   — one depth just past the boundary; the cost of refusing, which is the
                     term `.44`(d) measured as pathological and which now sits at a much
                     deeper place than it used to.

Usage: python3 tmp/eus65/depth_cost_curve.py <shape> <depth> [<depth> ...]
"""
import subprocess, sys, os, time, re, json

PROBE = os.environ.get("PGEN_PARSE_PROBE_BIN", "./rust/target/release/parseability_probe")
TMP = "tmp/eus65/cost"
os.makedirs(TMP, exist_ok=True)

SHAPES = {
    "paren_expr": ("systemverilog", "sv", ["--profile", "sv_2017"],
                   lambda d: "module m; assign x = " + "("*d + "1" + ")"*d + "; endmodule\n"),
    "cast_paren": ("systemverilog", "sv", ["--profile", "sv_2017"],
                   lambda d: "module m; assign x = " + "int'("*d + "1" + ")"*d + "; endmodule\n"),
    "gen_ternary_chain": ("systemverilog", "sv", ["--profile", "sv_2017"],
                   lambda d: "module m; assign x = " + "".join(f"c{i} ? v{i} : " for i in range(d)) + "vd; endmodule\n"),
}


def measure(shape, d, cap=1800):
    fam, ext, extra, gen = SHAPES[shape]
    p = f"{TMP}/{shape}_{d}.{ext}"
    src = gen(d)
    with open(p, "w") as fh:
        fh.write(src)
    t0 = time.time()
    try:
        r = subprocess.run(["/usr/bin/time", "-l", PROBE, "--parse", fam, p] + extra,
                           capture_output=True, timeout=cap)
        rc, err = r.returncode, r.stderr.decode(errors="replace")
    except subprocess.TimeoutExpired:
        return {"depth": d, "bytes": len(src), "verdict": f"NO-RESULT(>{cap}s)",
                "wall_s": cap, "peak_MB": None}
    wall = time.time() - t0
    m = re.search(r"(\d+)\s+maximum resident set size", err)
    peak = round(int(m.group(1)) / (1024 * 1024), 1) if m else None
    labelled = "recursion ceiling CONTACTED" in err
    return {"depth": d, "bytes": len(src),
            "verdict": "accepted" if rc == 0 else ("rejected-ceiling" if labelled else "rejected"),
            "wall_s": round(wall, 2), "peak_MB": peak}


def main():
    shape = sys.argv[1]
    depths = [int(x) for x in sys.argv[2:]]
    print(f"# shape={shape} probe={PROBE}")
    print(f"{'depth':>8} {'input B':>10} {'wall s':>9} {'peak MB':>9}  verdict")
    rows = []
    for d in depths:
        row = measure(shape, d)
        rows.append(row)
        print(f"{row['depth']:>8} {row['bytes']:>10} {str(row['wall_s']):>9} "
              f"{str(row['peak_MB']):>9}  {row['verdict']}", flush=True)
    with open(f"tmp/eus65/depth_cost_{shape}.json", "w") as fh:
        json.dump(rows, fh, indent=1)


if __name__ == "__main__":
    main()
