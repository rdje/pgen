#!/usr/bin/env python3
"""ENGINE-UNIVERSAL-SERVICES.65 — FALSIFICATION CONTROL for "the recursion ceiling is
derived from real host stack headroom".

A bigger accept boundary is not evidence that a ceiling became host-derived; it is only
evidence that a number got bigger. The discriminating prediction is PROPORTIONALITY: if
the ceiling is really the stack the thread owns, then halving the reservation must halve
the deepest accepted input, and the ratio (deepest_ok / reservation) must be flat across
reservations. A constant — of any size — cannot produce that.

For each reservation the same shape is bisected the same way, and the run is REJECTED as
evidence unless the boundary rejection is LABELLED as a recursion-ceiling contact (a
timeout or a grammar rejection would otherwise masquerade as a ceiling).

Usage: python3 tmp/eus65/reservation_scaling.py [shape] [reservation_MiB ...]
"""
import subprocess, sys, os, json, time

PROBE = os.environ.get("PGEN_PARSE_PROBE_BIN", "./rust/target/release/parseability_probe")
TMP = "tmp/eus65/scaling"
os.makedirs(TMP, exist_ok=True)

SHAPES = {
    # frames/level ~16.4 before `.65` — deep enough to be ceiling-bound, cheap enough
    # that even the largest reservation's boundary parses in seconds.
    "cast_paren": ("systemverilog", "sv", ["--profile", "sv_2017"],
                   lambda d: "module m; assign x = " + "int'("*d + "1" + ")"*d + "; endmodule\n"),
    "gen_mux_both_arms": ("systemverilog", "sv", ["--profile", "sv_2017"],
                   lambda d: "module m; assign x = " + "(s ? a : ("*d + "b" + "))"*d + "; endmodule\n"),
    "vhdl_paren": ("vhdl", "vhd", [],
                   lambda d: "entity e is end entity;\narchitecture a of e is\nsignal s : bit;\nbegin\n s <= "
                             + "("*d + "'0'" + ")"*d + ";\nend architecture;\n"),
}


def run(fam, name, d, gen, ext, extra, env):
    p = f"{TMP}/{fam}_{name}_{d}.{ext}"
    with open(p, "w") as fh:
        fh.write(gen(d))
    try:
        r = subprocess.run([PROBE, "--parse", fam, p] + extra,
                           capture_output=True, timeout=900, env=env)
    except subprocess.TimeoutExpired:
        return None, False
    out = (r.stdout + r.stderr).decode(errors="replace")
    return r.returncode == 0, ("recursion ceiling CONTACTED" in out)


def boundary(shape, mib):
    fam, ext, extra, gen = SHAPES[shape]
    env = dict(os.environ)
    env["PGEN_PARSE_STACK_BYTES"] = str(mib * 1024 * 1024)
    lo, hi = 1, 2
    while True:
        ok, _ = run(fam, shape, hi, gen, ext, extra, env)
        if ok is not True:
            break
        lo, hi = hi, hi * 2
        if hi > 1 << 20:
            return {"reservation_MiB": mib, "deepest_ok": f">{lo}", "labelled": None}
    while hi - lo > 1:
        mid = (lo + hi) // 2
        ok, _ = run(fam, shape, mid, gen, ext, extra, env)
        if ok is True:
            lo = mid
        else:
            hi = mid
    _, labelled = run(fam, shape, hi, gen, ext, extra, env)
    return {"reservation_MiB": mib, "deepest_ok": lo, "first_bad": hi,
            "labelled": bool(labelled), "levels_per_MiB": round(lo / mib, 3)}


def main():
    shape = sys.argv[1] if len(sys.argv) > 1 else "cast_paren"
    mibs = [int(x) for x in sys.argv[2:]] or [96, 128, 256, 512, 1024]
    print(f"# shape={shape} probe={PROBE}")
    print(f"{'reservation_MiB':>15} {'deepest OK':>10} {'first bad':>10} {'levels/MiB':>11} {'ceiling-labelled':>17}")
    rows = []
    for mib in mibs:
        t0 = time.time()
        row = boundary(shape, mib)
        row["wall_s"] = round(time.time() - t0, 1)
        rows.append(row)
        print(f"{row['reservation_MiB']:>15} {str(row.get('deepest_ok')):>10} "
              f"{str(row.get('first_bad', '-')):>10} {str(row.get('levels_per_MiB', '-')):>11} "
              f"{str(row.get('labelled')):>17}", flush=True)
    with open(f"tmp/eus65/reservation_scaling_{shape}.json", "w") as fh:
        json.dump(rows, fh, indent=1)


if __name__ == "__main__":
    main()
