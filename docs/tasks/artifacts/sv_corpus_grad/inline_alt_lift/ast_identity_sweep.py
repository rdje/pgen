#!/usr/bin/env python3
"""SV-CORPUS-GRAD.3.42 — typed-AST IDENTITY SWEEP between two parser binaries over a corpus file list.

For every file, both probes run `--parse-dump-ast systemverilog <file> --profile <p>` (TOOLBOX §1.2)
into a scratch file on the repository volume; the sweep compares the two dumps' sha256 and the two
verdicts, then deletes the dumps. It answers, MEASURED rather than argued, the question every
shape-neutral grammar claim owes: "did any accepted tree move?" — and names each one that did.

⛔ THE SWEEP IS ONLY AS STRONG AS ITS FILE LIST: pass the whole lane (every corpus row), not a
candidate list, when the claim is corpus-wide. A `timeout`/`nodump` row is COUNTED, never dropped.

Usage:
  python3 docs/tasks/artifacts/sv_corpus_grad/inline_alt_lift/ast_identity_sweep.py \
      --a rust/target/sNNN/probe_head --b rust/target/release/parseability_probe --profile sv_2017 \
      --files rust/target/sNNN/all_sv_files.txt --out rust/target/sNNN/ast_sweep_sv_2017.tsv [--jobs 8]
Prints one `AST-IDENTITY-SWEEP:` headline, then one line per file whose verdict or typed AST differs.
Exit 0 on a completed sweep (differences are the RESULT, not an error); exit 2 if a probe is missing.
"""
from __future__ import annotations
import argparse, concurrent.futures, hashlib, os, subprocess, sys, tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]

def dump(probe, profile, rel, timeout):
    fd, tmp = tempfile.mkstemp(suffix=".json", dir=str(ROOT / "rust" / "target"))
    os.close(fd)
    try:
        try:
            proc = subprocess.run([probe, "--parse-dump-ast", "systemverilog", str(ROOT / rel), tmp, "--profile", profile],
                                  capture_output=True, timeout=timeout)
        except subprocess.TimeoutExpired:
            return ("timeout", "")
        ok = proc.returncode == 0 and b"parse_full passed" in (proc.stdout + proc.stderr)
        if not ok:
            return ("rejected", "")
        h = hashlib.sha256()
        with open(tmp, "rb") as fh:
            for chunk in iter(lambda: fh.read(1 << 20), b""):
                h.update(chunk)
        if os.path.getsize(tmp) == 0:
            return ("nodump", "")
        return ("accepted", h.hexdigest())
    finally:
        try: os.unlink(tmp)
        except OSError: pass

def one(args):
    a, b, profile, rel, timeout = args
    va, ha = dump(a, profile, rel, timeout)
    vb, hb = dump(b, profile, rel, timeout)
    return (rel, va, vb, ha, hb)

def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--a", required=True, help="baseline probe"); ap.add_argument("--b", required=True, help="candidate probe")
    ap.add_argument("--profile", required=True); ap.add_argument("--files", required=True)
    ap.add_argument("--out", required=True); ap.add_argument("--jobs", type=int, default=8)
    ap.add_argument("--timeout-s", type=int, default=120)
    x = ap.parse_args()
    pa = str(ROOT / x.a) if not os.path.isabs(x.a) else x.a
    pb = str(ROOT / x.b) if not os.path.isabs(x.b) else x.b
    for p in (pa, pb):
        if not os.access(p, os.X_OK):
            print(f"AST-IDENTITY-SWEEP: REFUSED — probe not executable: {p}", file=sys.stderr); return 2
    files = [l.strip() for l in open(x.files) if l.strip() and not l.startswith("#")]
    both_acc = ident = moved = 0; verdict_moved = 0; other = 0; rows = []
    with concurrent.futures.ThreadPoolExecutor(max_workers=x.jobs) as ex:
        for rel, va, vb, ha, hb in ex.map(one, [(pa, pb, x.profile, f, x.timeout_s) for f in files]):
            if va == "accepted" and vb == "accepted":
                both_acc += 1
                if ha == hb: ident += 1
                else: moved += 1; rows.append((rel, va, vb, "AST-MOVED"))
            elif va != vb:
                verdict_moved += 1; rows.append((rel, va, vb, "VERDICT-MOVED"))
            else:
                other += 1
    with open(x.out, "w") as fh:
        fh.write(f"# ast identity sweep — a={x.a} b={x.b} profile={x.profile} files={len(files)}\n")
        fh.write("file\tverdict_a\tverdict_b\tclass\n")
        for r in sorted(rows): fh.write("\t".join(r) + "\n")
    print(f"AST-IDENTITY-SWEEP: profile={x.profile} files={len(files)} both_accepted={both_acc} identical={ident} "
          f"ast_moved={moved} verdict_moved={verdict_moved} both_nonaccepting={other}")
    for r in sorted(rows): print("  " + "\t".join(r))
    return 0

if __name__ == "__main__":
    sys.exit(main())
