#!/usr/bin/env python3
"""ENGINE-UNIVERSAL-SERVICES.63(c) — price the alternatives with measurement, not assertion.

Two questions the ruling cannot be made without:
  1. STACK. `SV-CORPUS-GRAD.8c.3` measured ~2 KB/frame release / ~17 KB/frame debug and
     sized the dedicated thread at 256 MiB on that. Re-derive it here (peak RSS as a
     function of parse depth) rather than carrying the number.
  2. TIME. A ceiling that could be raised safely is still not raisable if the parse at the
     new depth takes minutes. Measure wall time as a function of depth on the accept path.
"""
import subprocess, os, re, sys, time, json

PROBE = "./rust/target/release/parseability_probe"
TMP = "tmp/eus63/shapes"
os.makedirs(TMP, exist_ok=True)

SHAPES = {
    "paren":        (lambda d: "module m; assign x = " + "("*d + "1" + ")"*d + "; endmodule\n"),
    "mux_paren":    (lambda d: "module m; assign x = " + "s ? a : ("*d + "1" + ")"*d + "; endmodule\n"),
    "sv2v_mux":     (lambda d: "module m; assign x = " + "$signed({1'b0, (s ? a : ("*d + "1" + "))})"*d + "; endmodule\n"),
    "bare_chain":   (lambda d: "module m; assign x = " + "".join(f"c{i} ? v{i} : " for i in range(d)) + "vd; endmodule\n"),
}

RSS = re.compile(r"^\s*(\d+)\s+maximum resident set size", re.M)


def measure(name, gen, d):
    p = f"{TMP}/pr_{name}_{d}.sv"
    with open(p, "w") as fh:
        fh.write(gen(d))
    t0 = time.time()
    r = subprocess.run(["/usr/bin/time", "-l", PROBE, "--parse", "systemverilog", p,
                        "--profile", "sv_2017"], capture_output=True, timeout=900)
    wall = time.time() - t0
    err = r.stderr.decode(errors="replace")
    m = RSS.search(err)
    return r.returncode == 0, (int(m.group(1)) if m else None), wall


print(f"{'shape':12} {'depth':>7} {'accept':>7} {'peakRSS_MB':>11} {'wall_s':>8} {'ΔRSS/frame_B':>13}")
out = {}
for name, gen in SHAPES.items():
    rows, base = [], None
    depths = {"paren": [10, 80, 160, 240, 322],
              "mux_paren": [10, 70, 140, 210, 272],
              "sv2v_mux": [5, 15, 30, 45, 61],
              "bare_chain": [10, 500, 1000, 1500, 2028]}[name]
    for d in depths:
        acc, rss, wall = measure(name, gen, d)
        if base is None:
            base = (d, rss)
        per = ""
        if rss and base[1] and d != base[0]:
            # frames = depth * frames_per_level; report per-LEVEL then convert below
            per = f"{(rss - base[1]) / (d - base[0]):.0f}"
        rows.append({"depth": d, "accept": acc, "rss_bytes": rss, "wall_s": round(wall, 3)})
        print(f"{name:12} {d:>7} {str(acc):>7} {(rss or 0)/1048576:>11.1f} {wall:>8.3f} {per:>13}")
    out[name] = rows
with open("tmp/eus63/ceiling_pricing.json", "w") as fh:
    json.dump(out, fh, indent=1)
