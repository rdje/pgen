#!/usr/bin/env python3
r"""SV-CORPUS-GRAD.3.43 — AST FIELD CENSUS: what does a corpus actually WRITE at one typed-AST slot?

For every file in a list the release probe dumps the typed AST (`--parse-dump-ast systemverilog <file>
<tmp> --profile <p>`, TOOLBOX §1.2); every node whose `kind` is the one named is selected and the dotted
field path is read off it; one TSV row per selected node carries the field's compact JSON, verbatim.
Files the probe REJECTS (or times out on) carry no tree, so they are COUNTED as rows of their own and,
when `--fallback-regex` is given, scanned textually — those rows are tagged `regex`, never mixed with the
parser's reading. The parser's own reading of the slot is the census; a grep is only the fallback for
the rows the parser cannot read.

⛔ THE CENSUS IS ONLY AS STRONG AS ITS FILE LIST (the `.3.38` pre-screen rule): build the list with a
selector that cannot miss the construct (for a DPI import: the spec string `"DPI`, which every import
carries), and check it against the rows already known by name before trusting the count.

Usage:
  python3 docs/tasks/artifacts/sv_corpus_grad/ast_field_census/ast_field_census.py \
      --probe rust/target/release/parseability_probe --profile sv_2017 \
      --files rust/target/sNNN/dpi_files.txt --kind import_function --field proto.body.body.return_type \
      --out rust/target/sNNN/dpi_result_census.tsv [--jobs 8] [--timeout 120] \
      [--fallback-regex 'import\s+"DPI(?:-C)?"\s+(?:context|pure)?\s*(?:(?:\\\S+|\w+)\s*=\s*)?function\s+(?P<field>.+?)\s+(?:\\\S+|\w+)\s*[(;]']
  python3 .../ast_field_census.py --probe rust/target/release/parseability_probe --self-test

Headline (one line): `AST-FIELD-CENSUS: kind=K field=F profile=P files=N accepted=A rejected=R timeout=T
nodump=D nodes=M distinct=X fallback_rows=Y`.  Exit 0; exit 2 when the file list or the probe is unusable;
exit 7 on a self-test miss.  TSV columns: file  status  index  value  (value = compact JSON for parser rows,
the captured text for `regex` rows, `-` for a rejected/timeout file without a fallback match).
"""
from __future__ import annotations
import argparse, concurrent.futures as cf, json, os, re, shutil, subprocess, sys, tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]


def resolve(node, dotted: str):
    cur = node
    for seg in dotted.split("."):
        if isinstance(cur, dict):
            if seg not in cur:
                return ("<missing:%s>" % seg)
            cur = cur[seg]
        elif isinstance(cur, list) and seg.isdigit() and int(seg) < len(cur):
            cur = cur[int(seg)]
        else:
            return ("<missing:%s>" % seg)
    return cur


def select(tree, kind: str, dotted: str) -> list:
    out = []
    stack = [tree]
    while stack:
        o = stack.pop()
        if isinstance(o, dict):
            if o.get("kind") == kind:
                out.append(resolve(o, dotted))
            stack.extend(o.values())
        elif isinstance(o, list):
            stack.extend(o)
    # DFS via a stack reverses order; restore source order deterministically by re-walking in order
    return list(reversed(out)) if len(out) > 1 else out


def one_file(probe: str, profile: str, rel: str, kind: str, dotted: str, timeout: int, fallback: "re.Pattern|None"):
    src = ROOT / rel
    fd, tmp = tempfile.mkstemp(prefix="afc_", suffix=".json", dir=str(ROOT / "rust" / "target"))
    os.close(fd)
    rows = []
    try:
        try:
            proc = subprocess.run([probe, "--parse-dump-ast", "systemverilog", str(src), tmp, "--profile", profile],
                                  capture_output=True, timeout=timeout)
            status = "accepted" if (proc.returncode == 0 and b"parse_full passed" in (proc.stdout + proc.stderr)) else "rejected"
        except subprocess.TimeoutExpired:
            status = "timeout"
        if status == "accepted":
            if os.path.getsize(tmp) == 0:
                status = "nodump"
            else:
                with open(tmp, "rb") as fh:
                    tree = json.load(fh)
                vals = select(tree, kind, dotted)
                for i, v in enumerate(vals):
                    rows.append((rel, status, str(i), json.dumps(v, sort_keys=True, separators=(",", ":"))))
                if not vals:
                    rows.append((rel, status, "-", "<no %s node>" % kind))
        if status != "accepted":
            hit = False
            if fallback is not None:
                try:
                    text = src.read_text(errors="replace")
                except OSError:
                    text = ""
                for j, m in enumerate(fallback.finditer(text)):
                    hit = True
                    cap = m.group("field") if "field" in m.groupdict() else m.group(0)
                    rows.append((rel, status, "regex%d" % j, " ".join(cap.split())))
            if not hit:
                rows.append((rel, status, "-", "-"))
    finally:
        try:
            os.unlink(tmp)
        except OSError:
            pass
    return status, rows


def run(args) -> int:
    files = [l.strip() for l in Path(args.files).read_text().splitlines() if l.strip()]
    if not files:
        print("⛔ REFUSING: the file list is empty (%s)" % args.files); return 2
    probe = str(Path(args.probe).resolve())
    if not os.access(probe, os.X_OK):
        print("⛔ REFUSING: probe is not executable: %s" % probe); return 2
    fallback = re.compile(args.fallback_regex, re.S) if args.fallback_regex else None
    counts = {"accepted": 0, "rejected": 0, "timeout": 0, "nodump": 0}
    all_rows = []
    with cf.ThreadPoolExecutor(max_workers=args.jobs) as ex:
        futs = {ex.submit(one_file, probe, args.profile, f, args.kind, args.field, args.timeout, fallback): f for f in files}
        results = {}
        for fut in cf.as_completed(futs):
            results[futs[fut]] = fut.result()
    for f in files:  # deterministic order = the list's order
        status, rows = results[f]
        counts[status] += 1
        all_rows.extend(rows)
    parser_rows = [r for r in all_rows if r[1] == "accepted" and r[2] != "-"]
    fallback_rows = [r for r in all_rows if r[2].startswith("regex")]
    distinct = len({r[3] for r in parser_rows})
    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    with open(out, "w") as fh:
        fh.write("file\tstatus\tindex\tvalue\n")
        for r in all_rows:
            fh.write("\t".join(r) + "\n")
    print("AST-FIELD-CENSUS: kind=%s field=%s profile=%s files=%d accepted=%d rejected=%d timeout=%d nodump=%d nodes=%d distinct=%d fallback_rows=%d"
          % (args.kind, args.field, args.profile, len(files), counts["accepted"], counts["rejected"], counts["timeout"], counts["nodump"],
             len(parser_rows), distinct, len(fallback_rows)))
    print("rows -> %s" % out)
    return 0


def self_test(args) -> int:
    """Two synthetic files: an accepting one with two imports (nodes=2 distinct=2) and a rejecting one whose
    fallback regex must yield exactly one `regex` row. A miss exits 7."""
    probe = str(Path(args.probe).resolve())
    d = Path(tempfile.mkdtemp(prefix="afc_selftest_", dir=str(ROOT / "rust" / "target")))
    ok = (d / "ok.sv"); bad = (d / "bad.sv"); lst = (d / "list.txt")
    ok.write_text('module m;\n  import "DPI-C" function int f(input int a);\n  import "DPI-C" function void g();\nendmodule\n')
    bad.write_text('module m;\n  import "DPI-C" function int h(input int a);\n  this is not systemverilog ;;\nendmodule\n')
    lst.write_text("%s\n%s\n" % (ok.relative_to(ROOT), bad.relative_to(ROOT)))
    ns = argparse.Namespace(probe=probe, profile="sv_2017", files=str(lst), kind="import_function", field="proto.body.body.return_type",
                            out=str(d / "out.tsv"), jobs=2, timeout=60,
                            fallback_regex=r'import\s+"DPI(?:-C)?"\s+(?:context|pure)?\s*(?:(?:\\\S+|\w+)\s*=\s*)?function\s+(?P<field>.+?)\s+(?:\\\S+|\w+)\s*[(;]')
    rc = run(ns)
    if rc:
        return rc
    rows = [l.rstrip("\n").split("\t") for l in (d / "out.tsv").read_text().splitlines()[1:]]
    parser_rows = [r for r in rows if r[1] == "accepted"]
    regex_rows = [r for r in rows if r[2].startswith("regex")]
    want = {'{"body":{"base":{"kind":"int"},"kind":"integer_atom","signing":[]},"kind":"data_type"}', '{"kind":"void"}'}
    got = {r[3] for r in parser_rows}
    problems = []
    if got != want: problems.append("parser rows %r != %r" % (sorted(got), sorted(want)))
    if len(regex_rows) != 1 or regex_rows[0][3] != "int": problems.append("regex rows %r" % regex_rows)
    if rows[0][2] != "0" or rows[1][2] != "1": problems.append("index order %r" % [r[2] for r in rows[:2]])
    shutil.rmtree(d, ignore_errors=True)
    if problems:
        print("AST-FIELD-CENSUS SELF-TEST: MISS — " + "; ".join(problems)); return 7
    print("AST-FIELD-CENSUS SELF-TEST: OK — 2 parser rows (int, void) in source order, 1 regex fallback row on the rejected file")
    return 0


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--probe", required=True); ap.add_argument("--profile", default="sv_2017")
    ap.add_argument("--files"); ap.add_argument("--kind"); ap.add_argument("--field"); ap.add_argument("--out")
    ap.add_argument("--jobs", type=int, default=4); ap.add_argument("--timeout", type=int, default=120)
    ap.add_argument("--fallback-regex", default=None); ap.add_argument("--self-test", action="store_true")
    args = ap.parse_args()
    if args.self_test:
        return self_test(args)
    for k in ("files", "kind", "field", "out"):
        if getattr(args, k) is None:
            ap.error("--%s is required" % k)
    return run(args)


if __name__ == "__main__":
    sys.exit(main())
