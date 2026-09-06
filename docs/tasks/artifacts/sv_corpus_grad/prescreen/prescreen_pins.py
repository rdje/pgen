#!/usr/bin/env python3
"""Grammar-candidate pre-screen, PINS: every reproducer pin x profile under the INTERPRETER, HEAD grammar vs
candidate grammar - verdict AND typed-AST sha per arm, so a candidate is measured before a byte is generated.

Usage:
  python3 docs/tasks/artifacts/sv_corpus_grad/prescreen/prescreen_pins.py \
      --head rust/target/sNNN/g_head.ebnf --new grammars/systemverilog.ebnf \
      --bin rust/target/sNNN/ast_pipeline_interp --out rust/target/sNNN [--jobs 5]

Reads stimuli/sv/adjudication_repros/MANIFEST.tsv (id / profiles / entry columns). Writes <out>/prescreen_pins.tsv
and prints `PRESCREEN-PINS: jobs=N verdict_moved=V ast_moved=A`, then one line per moved arm.

Provenance: SV-CORPUS-GRAD.3.41 (first use, scratch), tracked by .3.38. The `--bin` is a COPY of the debug
ast_pipeline taken before the slice's builds, so the tool under measurement cannot change between its arms
(a-control-that-cannot-fail-is-not-a-control). Paths are repo-root-relative; the repo root is derived from this file.
"""
import argparse, csv, hashlib, os, pathlib, subprocess, sys, time, concurrent.futures as cf

R = pathlib.Path(__file__).resolve().parents[4]

def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--head", required=True); ap.add_argument("--new", required=True)
    ap.add_argument("--bin", required=True); ap.add_argument("--out", required=True)
    ap.add_argument("--jobs", type=int, default=5); ap.add_argument("--timeout", type=int, default=300)
    a = ap.parse_args()
    head, new, bin_, out = (str(R / a.head), str(R / a.new), str(R / a.bin), R / a.out)
    for f in (head, new, bin_):
        if not os.path.exists(f): sys.exit(f"prescreen_pins: missing {f}")
    (out / "pins_ast").mkdir(parents=True, exist_ok=True)
    jobs = []
    with open(R / "stimuli/sv/adjudication_repros/MANIFEST.tsv") as fh:
        for row in csv.DictReader(fh, delimiter="\t"):
            profs = [p.strip() for p in (row.get("profiles") or "sv_2017").split(",") if p.strip()]
            for p in profs: jobs.append((row["id"], row["expect"], p, row.get("arm") or "", row.get("entry") or ""))

    def one(j):
        pid, expect, prof, arm, entry = j
        f = str(R / "stimuli/sv/adjudication_repros" / pid); res = []
        for tag, g in (("head", head), ("new", new)):
            ast = out / "pins_ast" / f"{pid}.{prof}.{tag}.json"
            cmd = [bin_, g, "--grammar-profile", prof, "--interpret-parse", f, "--interpret-parse-ast-json", str(ast)]
            if entry: cmd += ["--interpret-entry-rule", entry]
            try:
                r = subprocess.run(cmd, capture_output=True, text=True, timeout=a.timeout)
                acc = "accepted=true" in r.stdout
            except subprocess.TimeoutExpired:
                acc = None
            sha = hashlib.sha256(ast.read_bytes()).hexdigest()[:16] if ast.exists() else "-"
            res.append((acc, sha))
        return j, res

    t0 = time.time(); rows = []
    with cf.ThreadPoolExecutor(max_workers=a.jobs) as ex:
        for i, (j, res) in enumerate(ex.map(one, jobs), 1):
            rows.append((j, res))
            if i % 100 == 0: print(f"  {i}/{len(jobs)} {time.time()-t0:.0f}s", flush=True)
    vm = am = 0
    with open(out / "prescreen_pins.tsv", "w") as fh:
        fh.write("id\texpect\tprofile\tarm\tentry\thead_accept\tnew_accept\thead_sha\tnew_sha\tverdict_moved\tast_moved\n")
        for (pid, expect, prof, arm, entry), ((ha, hs), (na, ns)) in rows:
            v = ha != na; m = bool(ha and na and hs != ns); vm += v; am += m
            fh.write(f"{pid}\t{expect}\t{prof}\t{arm}\t{entry}\t{ha}\t{na}\t{hs}\t{ns}\t{int(v)}\t{int(m)}\n")
    print(f"PRESCREEN-PINS: jobs={len(jobs)} verdict_moved={vm} ast_moved={am} in {time.time()-t0:.0f}s")
    for (pid, expect, prof, arm, entry), ((ha, hs), (na, ns)) in rows:
        if ha != na: print(f"  VERDICT {pid} {prof}: head={ha} new={na} expect={expect}")
        elif ha and na and hs != ns: print(f"  AST {pid} {prof}: {hs} -> {ns}")
    return 0

if __name__ == "__main__":
    sys.exit(main())
