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

Selector (exactly one): `--kind K` selects every object whose `kind` is K; `--select-keys a,b,c`
(SV-CORPUS-GRAD.3.44) selects every object whose key set is EXACTLY {a, b, c} — for a slot whose node
carries no `kind` discriminator, such as the `{lifetime, body}` node `function_declaration_sv_2017` /
`task_declaration_sv_2017` return (`--select-keys lifetime,body --field body` reads every function and
task body the parser wrote, typed object or raw envelope alike).

Headline (one line): `AST-FIELD-CENSUS: kind=K field=F profile=P files=N accepted=A rejected=R timeout=T
nodump=D nodes=M distinct=X fallback_rows=Y` (`keys=a,b,c` in place of `kind=K` under `--select-keys`).
Exit 0; exit 2 when the file list or the probe is unusable; exit 7 on a self-test miss.  TSV columns:
file  status  index  value  (value = compact JSON for parser rows, the captured text for `regex` rows,
`-` for a rejected/timeout file without a fallback match).
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


def select(tree, kind: "str|None", dotted: str, keys: "frozenset|None" = None) -> list:
    """Every object whose `kind` is `kind`, or — under `--select-keys` — whose key set equals `keys` exactly."""
    out = []
    stack = [tree]
    while stack:
        o = stack.pop()
        if isinstance(o, dict):
            if (keys is not None and frozenset(o.keys()) == keys) or (keys is None and o.get("kind") == kind):
                out.append(resolve(o, dotted))
            stack.extend(o.values())
        elif isinstance(o, list):
            stack.extend(o)
    # DFS via a stack reverses order; restore source order deterministically by re-walking in order
    return list(reversed(out)) if len(out) > 1 else out


def dump_tree(probe: str, profile: str, rel: str, timeout: int):
    """Dump one lane file's typed AST with the release probe (TOOLBOX §1.2) into a scratch file on the repository
    volume, read it, delete it. Returns `(status, tree)`: status is `accepted` (tree is the parsed JSON), `rejected`,
    `timeout` or `nodump` (the probe accepted but wrote no tree) — the tree is `None` for the last three. Shared by
    this census and `partial_typing_residue/raw_envelope_census.py` (SV-CORPUS-GRAD.3.45), so both read a file the
    same way."""
    src = ROOT / rel
    fd, tmp = tempfile.mkstemp(prefix="afc_", suffix=".json", dir=str(ROOT / "rust" / "target"))
    os.close(fd)
    tree = None
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
    finally:
        try:
            os.unlink(tmp)
        except OSError:
            pass
    return status, tree


def one_file(probe: str, profile: str, rel: str, kind: "str|None", dotted: str, timeout: int, fallback: "re.Pattern|None",
             keys: "frozenset|None" = None):
    src = ROOT / rel
    selector_name = kind if keys is None else "keys{%s}" % ",".join(sorted(keys))
    rows = []
    status, tree = dump_tree(probe, profile, rel, timeout)
    if status == "accepted":
        vals = select(tree, kind, dotted, keys)
        for i, v in enumerate(vals):
            rows.append((rel, status, str(i), json.dumps(v, sort_keys=True, separators=(",", ":"))))
        if not vals:
            rows.append((rel, status, "-", "<no %s node>" % selector_name))
    else:
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
    return status, rows


def run(args) -> int:
    files = [l.strip() for l in Path(args.files).read_text().splitlines() if l.strip()]
    if not files:
        print("⛔ REFUSING: the file list is empty (%s)" % args.files); return 2
    probe = str(Path(args.probe).resolve())
    if not os.access(probe, os.X_OK):
        print("⛔ REFUSING: probe is not executable: %s" % probe); return 2
    fallback = re.compile(args.fallback_regex, re.S) if args.fallback_regex else None
    keys = frozenset(k.strip() for k in args.select_keys.split(",") if k.strip()) if getattr(args, "select_keys", None) else None
    if keys is not None and not keys:
        print("⛔ REFUSING: --select-keys names no key"); return 2
    counts = {"accepted": 0, "rejected": 0, "timeout": 0, "nodump": 0}
    all_rows = []
    with cf.ThreadPoolExecutor(max_workers=args.jobs) as ex:
        futs = {ex.submit(one_file, probe, args.profile, f, args.kind, args.field, args.timeout, fallback, keys): f for f in files}
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
    selector = ("kind=%s" % args.kind) if keys is None else ("keys=%s" % ",".join(sorted(keys)))
    print("AST-FIELD-CENSUS: %s field=%s profile=%s files=%d accepted=%d rejected=%d timeout=%d nodump=%d nodes=%d distinct=%d fallback_rows=%d"
          % (selector, args.field, args.profile, len(files), counts["accepted"], counts["rejected"], counts["timeout"], counts["nodump"],
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
    # Third arm (SV-CORPUS-GRAD.3.44): the key-set selector on a slot with no `kind` — one non-ported function
    # (alternative 1, a typed object carrying `return_type`) and one ported task (a raw envelope until 1.0.218,
    # a typed object carrying `ports` from 1.0.218 on). Both readings are admitted; a MISSING row, an extra row, or
    # a body that is neither a list nor an object carrying `name` is a miss — the arm holds the SELECTOR, not the
    # release's shape (the shape is the classifier's business, `tf_body_census/classify.py`).
    ks = (d / "keys.sv"); klst = (d / "klist.txt")
    ks.write_text("module m;\n  function int f;\n    return 1;\n  endfunction\n  task t(input int a);\n    #1;\n  endtask\nendmodule\n")
    klst.write_text("%s\n" % ks.relative_to(ROOT))
    ns2 = argparse.Namespace(probe=probe, profile="sv_2017", files=str(klst), kind=None, select_keys="lifetime,body", field="body",
                             out=str(d / "out2.tsv"), jobs=1, timeout=60, fallback_regex=None)
    rc = run(ns2)
    if rc:
        shutil.rmtree(d, ignore_errors=True); return rc
    rows2 = [l.rstrip("\n").split("\t") for l in (d / "out2.tsv").read_text().splitlines()[1:]]
    bodies = [json.loads(r[3]) for r in rows2 if r[1] == "accepted" and r[2] != "-"]
    if len(bodies) != 2:
        problems.append("select-keys rows %d != 2 (%r)" % (len(bodies), [r[3][:60] for r in rows2]))
    else:
        f_body, t_body = bodies
        if not (isinstance(f_body, dict) and "return_type" in f_body and f_body.get("name") == {"body": "f"}):
            problems.append("select-keys function body %s" % json.dumps(f_body)[:120])
        if not (isinstance(t_body, list) or (isinstance(t_body, dict) and t_body.get("name") == {"body": "t"} and "ports" in t_body)):
            problems.append("select-keys task body %s" % json.dumps(t_body)[:120])
    shutil.rmtree(d, ignore_errors=True)
    if problems:
        print("AST-FIELD-CENSUS SELF-TEST: MISS — " + "; ".join(problems)); return 7
    print("AST-FIELD-CENSUS SELF-TEST: OK — 2 parser rows (int, void) in source order, 1 regex fallback row on the rejected file; "
          "--select-keys lifetime,body reads the two {lifetime, body} nodes (function alternative 1 typed, task ported: %s)"
          % ("raw envelope" if isinstance(bodies[1], list) else "typed"))
    return 0


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--probe", required=True); ap.add_argument("--profile", default="sv_2017")
    ap.add_argument("--files"); ap.add_argument("--kind"); ap.add_argument("--field"); ap.add_argument("--out")
    ap.add_argument("--select-keys", default=None,
                    help="select every object whose key set is EXACTLY this comma-separated set (instead of --kind)")
    ap.add_argument("--jobs", type=int, default=4); ap.add_argument("--timeout", type=int, default=120)
    ap.add_argument("--fallback-regex", default=None); ap.add_argument("--self-test", action="store_true")
    args = ap.parse_args()
    if args.self_test:
        return self_test(args)
    for k in ("files", "field", "out"):
        if getattr(args, k) is None:
            ap.error("--%s is required" % k)
    if (args.kind is None) == (args.select_keys is None):
        ap.error("exactly one of --kind / --select-keys is required")
    return run(args)


if __name__ == "__main__":
    sys.exit(main())
