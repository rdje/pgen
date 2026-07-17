#!/usr/bin/env python3
"""RGX-0078.5.i.14 land gate — geomean-of-best-mins, base (-0115) vs cand (.5.i.14).
Reads roundN_base.txt / roundN_cand.txt (regex_perf_probe output; column 2 = min ns).
Land-iff-faster bar: ratio cand/base < 0.98 (>=2.0% faster)."""
import glob, math, os, re, sys

# repo-root-relative; override with PGEN_SPINE_BENCH_DIR. Run from the repo root
# (or point the env var at the round*_{base,cand}.txt directory).
BENCH = os.environ.get("PGEN_SPINE_BENCH_DIR", "rust/target/generated_logs/spine_bench")

def parse(path):
    out = {}
    for line in open(path):
        m = re.match(r"^([a-z_]+)\s+(\d+)\s+(\d+)\s+(\d+)\s+(\d+)\s+(\d+)\s+(\d+)\s*$", line)
        if m:
            out[m.group(1)] = int(m.group(2))  # min ns
    return out

def geomean(xs):
    return math.exp(sum(math.log(x) for x in xs) / len(xs))

rounds = sorted(int(re.search(r"round(\d+)_base", p).group(1))
                for p in glob.glob(f"{BENCH}/round*_base.txt"))
if not rounds:
    print("no round files"); sys.exit(1)

base = {r: parse(f"{BENCH}/round{r}_base.txt") for r in rounds}
cand = {r: parse(f"{BENCH}/round{r}_cand.txt") for r in rounds}
patterns = sorted(set().union(*[set(base[r]) for r in rounds]))

print(f"rounds={rounds}  patterns={len(patterns)}\n")
# per-round geomean-of-mins ratio
print("per-round geomean-of-mins (base -> cand, ratio):")
for r in rounds:
    b = geomean([base[r][p] for p in patterns])
    c = geomean([cand[r][p] for p in patterns])
    print(f"  round {r}: {b:8.1f} -> {c:8.1f}  ratio={c/b:.4f}  ({100*(c/b-1):+.2f}%)")

# best-min across rounds per pattern
best_b = {p: min(base[r][p] for r in rounds) for p in patterns}
best_c = {p: min(cand[r][p] for r in rounds) for p in patterns}
print("\nper-pattern best-min (base -> cand, ratio):")
for p in patterns:
    print(f"  {p:20s} {best_b[p]:8d} -> {best_c[p]:8d}  ratio={best_c[p]/best_b[p]:.4f}  ({100*(best_c[p]/best_b[p]-1):+.2f}%)")

gb = geomean([best_b[p] for p in patterns])
gc = geomean([best_c[p] for p in patterns])
ratio = gc / gb
print(f"\n=== GEOMEAN-OF-BEST-MINS: base {gb:.1f}ns -> cand {gc:.1f}ns ===")
print(f"=== RATIO cand/base = {ratio:.4f}  =>  {100*(ratio-1):+.2f}% ===")
print(f"=== falsification bar -2.0% (ratio<0.98): {'LAND (faster)' if ratio < 0.98 else 'DO NOT LAND (sub-bar)'} ===")
