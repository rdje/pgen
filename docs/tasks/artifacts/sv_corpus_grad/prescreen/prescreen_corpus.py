#!/usr/bin/env python3
"""Grammar-candidate pre-screen, CORPUS: a candidate list of corpus files under the INTERPRETER, HEAD grammar vs
candidate grammar, on the SV profiles - verdict + furthest position per arm, so the lanes' movement is predicted
before the parser is built.

Usage:
  python3 docs/tasks/artifacts/sv_corpus_grad/prescreen/prescreen_corpus.py \
      --head rust/target/sNNN/g_head.ebnf --new grammars/systemverilog.ebnf \
      --bin rust/target/sNNN/ast_pipeline_interp --out rust/target/sNNN \
      --files rust/target/sNNN/corpus_candidates.txt [--profiles sv_2017,sv_2023] [--jobs 4]

`--files` is one repo-root-relative path per line. ⛔ SEED IT WITH THE ROWS THE SLICE WAS OPENED ON, BY NAME, and
only then add the textual candidates (a grep) and the construct's positive population (e.g. every file with a DPI
import): SV-CORPUS-GRAD.3.38 measured a broad ugrep pre-screen that returned 33 files and MISSED 5 of the 6 rows
the slice existed for (a `\\\\` alternative inside a group made the regex fail on `reg [] illegal;` itself), so
"zero movers" was an instrument blind spot until the known rows were run. A candidate list that does not contain
the known positive has not been tested.

Writes <out>/prescreen_corpus.tsv and prints `PRESCREEN-CORPUS: files=N arms=M`, the (profile, head, new)
verdict histogram, then every PASS->FAIL / FAIL->PASS / FURTHEST-MOVED arm. Provenance: SV-CORPUS-GRAD.3.41
(scratch), tracked by .3.38.
"""
import argparse, os, pathlib, re, subprocess, sys, time, concurrent.futures as cf
from collections import Counter

# The repo root is FIVE parents up (prescreen/ -> sv_corpus_grad/ -> artifacts/ -> tasks/ -> docs/ -> root).
# SV-CORPUS-GRAD.3.42: this read parents[4] (= docs/) since the scripts were tracked by .3.38, so the tracked
# form could never run from its own location -- .3.38 had measured with the scratch copies under rust/target/.
# REFUSE rather than derive against a wrong root: a pre-screen that cannot find the corpus reports nothing.
R = pathlib.Path(__file__).resolve().parents[5]
if not (R / "grammars" / "systemverilog.ebnf").is_file():
    sys.exit(f"{pathlib.Path(__file__).name}: REFUSED -- {R} is not the repository root (no grammars/systemverilog.ebnf)")

def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--head", required=True); ap.add_argument("--new", required=True)
    ap.add_argument("--bin", required=True); ap.add_argument("--out", required=True)
    ap.add_argument("--files", required=True); ap.add_argument("--profiles", default="sv_2017,sv_2023")
    ap.add_argument("--jobs", type=int, default=4); ap.add_argument("--timeout", type=int, default=600)
    a = ap.parse_args()
    head, new, bin_, out = (str(R / a.head), str(R / a.new), str(R / a.bin), R / a.out)
    for f in (head, new, bin_):
        if not os.path.exists(f): sys.exit(f"prescreen_corpus: missing {f}")
    out.mkdir(parents=True, exist_ok=True)
    files = [l.strip() for l in open(R / a.files) if l.strip() and not l.startswith("#")]
    missing = [f for f in files if not (R / f).exists()]
    if missing: sys.exit(f"prescreen_corpus: {len(missing)} listed file(s) do not exist, e.g. {missing[:3]}")
    profiles = [p for p in a.profiles.split(",") if p]
    jobs = [(f, p) for f in files for p in profiles]

    def one(j):
        rel, prof = j; res = []
        for g in (head, new):
            try:
                r = subprocess.run([bin_, g, "--grammar-profile", prof, "--interpret-parse", str(R / rel)],
                                   capture_output=True, text=True, timeout=a.timeout)
                m = re.search(r"accepted=(\w+) furthest_position=(\d+)", r.stdout)
                res.append((m.group(1), int(m.group(2))) if m else ("err", -1))
            except subprocess.TimeoutExpired:
                res.append(("timeout", -1))
        return j, res

    t0 = time.time(); rows = []
    with cf.ThreadPoolExecutor(max_workers=a.jobs) as ex:
        for i, (j, res) in enumerate(ex.map(one, jobs), 1):
            rows.append((j, res))
            if i % 25 == 0: print(f"  {i}/{len(jobs)} {time.time()-t0:.0f}s", flush=True)
    with open(out / "prescreen_corpus.tsv", "w") as fh:
        fh.write("path\tprofile\thead_accept\thead_furthest\tnew_accept\tnew_furthest\n")
        for (rel, prof), ((ha, hf), (na, nf)) in rows: fh.write(f"{rel}\t{prof}\t{ha}\t{hf}\t{na}\t{nf}\n")
    hist = Counter((prof, ha, na) for (rel, prof), ((ha, hf), (na, nf)) in rows)
    print(f"PRESCREEN-CORPUS: files={len(files)} arms={len(rows)} in {time.time()-t0:.0f}s")
    for k, v in sorted(hist.items()): print("  ", k, v)
    for (rel, prof), ((ha, hf), (na, nf)) in rows:
        if ha == "true" and na != "true": print("  PASS->FAIL", prof, rel, hf, "->", nf)
        elif ha != "true" and na == "true": print("  FAIL->PASS", prof, rel)
        elif ha != "true" and na != "true" and nf != hf: print("  FURTHEST-MOVED", prof, rel, hf, "->", nf)
    return 0

if __name__ == "__main__":
    sys.exit(main())
