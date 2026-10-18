#!/usr/bin/env python3
"""Triage a certificate's SAMPLE-PARSE FAILURES: extract each one byte-exact and read it three ways.

`ast_pipeline G --report-certificate-coverage …` prints each generated sample the real parser refused under
`SAMPLE-PARSE FAILURES`. A refused sample is EITHER a lexical boundary the grammar never declared for generation (the
parser is right) OR a parser defect (the text is valid) — never discounted (SV-CORPUS-GRAD.13c.2v.16; knowledge card
a-generated-sample-the-parser-refuses-is-a-boundary-or-a-defect). This tool separates them:

  generated  the release probe's verdict (`parseability_probe --parse <grammar-name> <file>`)
  committed  the interpreter on the committed grammar (`ast_pipeline <grammar> --interpret-parse <file>`)
  candidate  the interpreter on a candidate grammar, when given

`generated != committed` is an ENGINE DIVERGENCE. A sample the candidate accepts is a PARSER fix that candidate's
rebuild delivers. One the candidate still refuses is either a boundary GENERATION must not render (the parser is right;
the fix is a declaration or a generator service, measured by re-running the sweep) or a defect the candidate does not
fix — the tool cannot tell those two apart; read each one until it is attributed.

Each sample is cut EXACTLY `N bytes` after its `sample (N bytes): ` header, from a log read with `newline=""`: a
text-mode read folds CR LF to LF and shifts every later offset (the first cut of this tool did exactly that).

usage: triage.py --logs 'DIR/seed*.log' --grammar grammars/G.ebnf [--candidate CAND.ebnf] [--profile P]
                 [--probe rust/target/release/parseability_probe] --out DIR
"""
import argparse, glob, json, os, re, subprocess, sys
from collections import Counter

ap = argparse.ArgumentParser()
ap.add_argument("--logs", required=True)
ap.add_argument("--grammar", required=True)
ap.add_argument("--candidate")
ap.add_argument("--profile")
ap.add_argument("--probe", default="rust/target/release/parseability_probe")
ap.add_argument("--pipeline", default="rust/target/debug/ast_pipeline")
ap.add_argument("--out", required=True)
a = ap.parse_args()
os.makedirs(a.out, exist_ok=True)
name = os.path.basename(a.grammar).removesuffix(".ebnf")
prof_i = ["--grammar-profile", a.profile] if a.profile else []
prof_p = ["--profile", a.profile] if a.profile else []
VERDICT = re.compile(r"accepted=(true|false) furthest_position=(\d+)")

def interp(grammar, path):
    r = subprocess.run([a.pipeline, grammar, *prof_i, "--interpret-parse", path], capture_output=True, text=True)
    m = VERDICT.search(r.stdout + r.stderr)
    return (m.group(1), int(m.group(2))) if m else ("?", -1)

def seed_of(path):
    m = re.search(r"seed(\d+)", path)
    return int(m.group(1)) if m else -1

rows = []
for log in sorted(glob.glob(a.logs), key=seed_of):
    text = open(log, encoding="utf-8", errors="replace", newline="").read()
    for m in re.finditer(r"\[(\d+)\] error: ([^\n]*)\n\s+\[\d+\] sample \((\d+) bytes\): ", text):
        n = int(m.group(3)); body = text[m.end():m.end() + n]
        path = os.path.join(a.out, f"f{len(rows):03d}.sv")
        open(path, "w", newline="").write(body)
        gen = subprocess.run([a.probe, "--parse", name, path, *prof_p], capture_output=True).returncode == 0
        com, com_fp = interp(a.grammar, path)
        can, can_fp = interp(a.candidate, path) if a.candidate else ("-", -1)
        rows.append({"file": path, "seed": seed_of(log), "generated": "true" if gen else "false",
                     "committed": com, "committed_fp": com_fp, "candidate": can, "candidate_fp": can_fp})
json.dump(rows, open(os.path.join(a.out, "index.json"), "w"), indent=0)
divergent = [r for r in rows if r["generated"] != r["committed"]]
fixed = [r for r in rows if r["candidate"] == "true"]
open_rows = [r for r in rows if r["candidate"] != "true"]
print(f"CERT-SAMPLE-TRIAGE: failures={len(rows)} engine_divergent={len(divergent)} "
      f"candidate_accepts={len(fixed) if a.candidate else '-'} still_refused={len(open_rows) if a.candidate else len(rows)} "
      f"(still_refused = a boundary generation must not render, or a defect the candidate does not fix: read each)")
for r in (open_rows if a.candidate else rows)[:40]:
    b = open(r["file"], newline="").read(); fp = r["candidate_fp"] if a.candidate else r["committed_fp"]
    print(f"  {r['file']} seed={r['seed']} at={fp}: {b[max(0, fp - 60):fp]!r} ⟦{b[fp:fp + 24]!r}⟧")
sys.exit(1 if divergent else 0)
