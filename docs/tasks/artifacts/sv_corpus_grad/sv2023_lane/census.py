#!/usr/bin/env python3
"""SV-CORPUS-GRAD.14a — the sv_2017 × sv_2023 VERDICT CENSUS over the same corpus population.

The `sv2023` lane (`stimuli/run_external_corpus.sh sv2023 60 8 0`) re-parses every file the `sv`
lane parses, under `--profile sv_2023`. This census joins the two lanes' raw outcomes per file and
publishes the 2×2 matrix (pass/non-pass under each edition) plus EVERY row whose verdict differs —
the rows that make the 2023 lane worth having:

  * `2023_only_pass`  — parses under sv_2023 and NOT under sv_2017: the IEEE 1800-2023 widening
                        the corpus actually exercises (the `.13x.d` edition contrasts were known
                        from two pins only); the 2023-lane answer key (`.14b`) keys these `must_accept`.
  * `2017_only_pass`  — parses under sv_2017 and NOT under sv_2023: a 2023-profile REGRESSION
                        candidate (1800-2023 is a superset edition; a rule the profile gates OUT
                        of sv_2023 would show here), each a parser question until adjudicated.

⛔ IDENTITY IS VERIFIED, NEVER ASSUMED. Both lanes' reports must name the SAME parse binary and the
same semantic grammar digest (a census across two binaries describes nothing), the binary on disk
must be that one, and each lane's raw dumps must re-hash to the OUTPUT rows its report carries
(`SV-CORPUS-GRAD.13e.6`: the dumps are untracked, so the report is the only thing that says which
run produced them). Any mismatch REFUSES (exit 2).

Usage:
  python3 docs/tasks/artifacts/sv_corpus_grad/sv2023_lane/census.py              # derive + write census.md/.tsv
  python3 docs/tasks/artifacts/sv_corpus_grad/sv2023_lane/census.py --check      # re-derive, compare bytes (rc 1 on drift)
  python3 docs/tasks/artifacts/sv_corpus_grad/sv2023_lane/census.py --self-test  # the arithmetic + the refusals, no parser
"""
from __future__ import annotations

import argparse
import hashlib
import re
import sys
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]
HERE = Path(__file__).resolve().parent
CH = ROOT / "stimuli/sv/characterization"
LANES = {
    "sv_2017": {"results": CH / "results.tsv", "positions": CH / "positions.tsv", "report": CH / "characterization.md"},
    "sv_2023": {"results": CH / "results_sv2023.tsv", "positions": CH / "positions_sv2023.tsv", "report": CH / "characterization_sv2023.md"},
}
MANIFEST = CH / "adjudication_manifest.tsv"
PROBE = ROOT / "rust/target/release/parseability_probe"
OUT_MD = HERE / "census.md"
OUT_TSV = HERE / "census.tsv"
LEAF = "SV-CORPUS-GRAD.14a"
ROW_RE = re.compile(r"^\| ([^|]+?) \| `([^`]+)` \| `([0-9a-f]{64})` \|")


def sha256_of(path: Path) -> str:
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def report_rows(text: str) -> dict:
    """label -> (repo-relative path, sha256) for every identity row of a characterization report."""
    rows = {}
    for line in text.splitlines():
        m = ROW_RE.match(line)
        if m and m.group(1).strip() not in rows:
            rows[m.group(1).strip()] = (m.group(2), m.group(3))
    return rows


def manifest_relpath(suite: str, path: str) -> str:
    """The adjudication manifest keys a file RELATIVE TO ITS SUITE (`stimuli/sv/subs/<suite>/` and,
    for the folded `uvm-core` tree, `stimuli/sv/uvm/` are stripped); the raw dumps carry the
    repo-relative path. One convention for the join, the manifest's."""
    for prefix in (f"stimuli/sv/subs/{suite}/", "stimuli/sv/uvm/"):
        if path.startswith(prefix):
            return path[len(prefix):]
    return path


def load_results(text: str) -> dict:
    out = {}
    for line in text.splitlines():
        if not line.strip():
            continue
        suite, status, rel = line.split("\t")[:3]
        out[(suite, manifest_relpath(suite, rel))] = status
    return out


def load_positions(text: str) -> dict:
    out = {}
    for line in text.splitlines():
        parts = line.split("\t")
        if len(parts) >= 3 and parts[2].isdigit():
            out[(parts[0], manifest_relpath(parts[0], parts[1]))] = int(parts[2])
    return out


def load_manifest(text: str) -> dict:
    out = {}
    lines = text.splitlines()
    header = lines[0].split("\t")
    for line in lines[1:]:
        if not line.strip():
            continue
        r = dict(zip(header, line.split("\t")))
        out[(r["suite"], r["relpath"])] = r
    return out


def verify_identity(rows_a: dict, rows_b: dict, on_disk: dict) -> list:
    """Refusals (empty = sound). `on_disk` maps a report row label to the sha the tree has NOW for
    that row's file ('parse binary', 'results (output)', 'positions (output)' per lane)."""
    refusals = []
    for label in ("parse binary", "grammar raw ast"):
        a, b = rows_a.get(label), rows_b.get(label)
        if not a or not b:
            refusals.append(f"a lane's report carries no `{label}` row")
        elif a[1] != b[1]:
            refusals.append(f"the two lanes name different `{label}` digests ({a[1][:12]}… vs {b[1][:12]}…) — a census across two instruments describes nothing")
    pb = rows_a.get("parse binary")
    if pb and on_disk.get("parse binary") and on_disk["parse binary"] != pb[1]:
        refusals.append(f"the binary on disk ({on_disk['parse binary'][:12]}…) is not the one the reports name ({pb[1][:12]}…)")
    for lane, rows in (("sv_2017", rows_a), ("sv_2023", rows_b)):
        for label in ("results (output)", "positions (output)"):
            want = rows.get(label)
            have = on_disk.get(f"{lane}:{label}")
            if want and have and have != want[1]:
                refusals.append(f"{lane}: `{label}` on disk does not re-hash to the row its report carries — a dump from another run")
    return refusals


def matrix(res17: dict, res23: dict) -> dict:
    """The join. Every key must be in both lanes (same population); a key in one lane only is a
    population breach and is counted, never silently dropped."""
    keys = set(res17) | set(res23)
    only17 = sorted(k for k in keys if k not in res23)
    only23 = sorted(k for k in keys if k not in res17)
    both = sorted(k for k in keys if k in res17 and k in res23)
    cells = Counter()
    pairs = Counter()
    moved = []
    for k in both:
        a, b = res17[k], res23[k]
        pairs[(a, b)] += 1
        pa, pb = a == "pass", b == "pass"
        cell = ("both_pass" if pa and pb else "both_nonpass" if not pa and not pb else
                "2017_only_pass" if pa else "2023_only_pass")
        cells[cell] += 1
        if cell in ("2017_only_pass", "2023_only_pass"):
            moved.append((cell, k, a, b))
    return {"rows": len(both), "only_in_sv_2017": only17, "only_in_sv_2023": only23,
            "cells": dict(cells), "status_pairs": dict(pairs), "moved": moved}


def derive() -> dict:
    texts = {lane: {k: p.read_text(encoding="utf-8", errors="replace") for k, p in spec.items()}
             for lane, spec in LANES.items()}
    rows17, rows23 = report_rows(texts["sv_2017"]["report"]), report_rows(texts["sv_2023"]["report"])
    on_disk = {"parse binary": sha256_of(PROBE) if PROBE.is_file() else ""}
    for lane, spec in LANES.items():
        on_disk[f"{lane}:results (output)"] = sha256_of(spec["results"])
        on_disk[f"{lane}:positions (output)"] = sha256_of(spec["positions"])
    refusals = verify_identity(rows17, rows23, on_disk)
    if refusals:
        for r in refusals:
            print(f"sv2023-lane-census: REFUSED — {r}", file=sys.stderr)
        raise SystemExit(2)
    res17, res23 = load_results(texts["sv_2017"]["results"]), load_results(texts["sv_2023"]["results"])
    pos17, pos23 = load_positions(texts["sv_2017"]["positions"]), load_positions(texts["sv_2023"]["positions"])
    man = load_manifest(MANIFEST.read_text(encoding="utf-8"))
    m = matrix(res17, res23)
    missing = [k for k in res17 if k not in man]
    if missing:
        print(f"sv2023-lane-census: REFUSED — {len(missing)} joined row(s) are not in the 2017 manifest (first: {missing[0]}); "
              "the join convention or the manifest moved", file=sys.stderr)
        raise SystemExit(2)
    detail = []
    for cell, k, a, b in m["moved"]:
        r = man.get(k, {})
        detail.append({"cell": cell, "suite": k[0], "relpath": k[1], "sv_2017": a, "sv_2023": b,
                       "furthest_sv_2017": pos17.get(k, "-"), "furthest_sv_2023": pos23.get(k, "-"),
                       "adjudication_2017_lane": r.get("adjudication", "(not in manifest)"),
                       "expected_2017_lane": r.get("expected", "")})
    detail.sort(key=lambda d: (d["cell"], d["suite"], d["relpath"]))
    identity = [("parse binary", rows17["parse binary"][0], rows17["parse binary"][1]),
                ("grammar raw ast", rows17["grammar raw ast"][0], rows17["grammar raw ast"][1]),
                ("sv_2017 results (output)", "stimuli/sv/characterization/results.tsv", on_disk["sv_2017:results (output)"]),
                ("sv_2023 results (output)", "stimuli/sv/characterization/results_sv2023.tsv", on_disk["sv_2023:results (output)"]),
                ("sv_2017 manifest", "stimuli/sv/characterization/adjudication_manifest.tsv", sha256_of(MANIFEST))]
    return {"matrix": m, "detail": detail, "identity": identity}


def render_tsv(d: dict) -> str:
    cols = ["cell", "suite", "relpath", "sv_2017", "sv_2023", "furthest_sv_2017", "furthest_sv_2023",
            "adjudication_2017_lane", "expected_2017_lane"]
    lines = ["\t".join(cols)]
    for r in d["detail"]:
        lines.append("\t".join(str(r[c]) for c in cols))
    return "\n".join(lines) + "\n"


def render_md(d: dict) -> str:
    m, cells = d["matrix"], d["matrix"]["cells"]
    by_cell = Counter(r["cell"] for r in d["detail"])
    adj = Counter((r["cell"], r["adjudication_2017_lane"]) for r in d["detail"])
    out = []
    out.append(f"# The sv_2017 × sv_2023 verdict census ({LEAF})\n")
    out.append("> DERIVED. Re-run: `python3 docs/tasks/artifacts/sv_corpus_grad/sv2023_lane/census.py`; `--check` re-derives and diffs.\n")
    out.append("Both lanes parse the SAME population — every `.sv` / `.svh` / `.v` file under `stimuli/sv/subs` plus the folded `uvm-core` tree — "
               "with the same binary; only `--profile` differs. A row is `pass` or not (`fail` / `timeout` / `crash` are all not-pass; the status pairs are below).\n")
    out.append("## Instrument identity\n")
    out.append("> Re-hash the rows; if any differs, this census no longer describes your tree — re-run it, never quote it.\n")
    out.append("| input | repo-root-relative path | sha256 |\n|---|---|---|")
    for label, path, sha in d["identity"]:
        out.append(f"| {label} | `{path}` | `{sha}` |")
    out.append("")
    out.append("## The matrix\n")
    out.append(f"Rows joined: **{m['rows']}**; only in the sv_2017 lane: **{len(m['only_in_sv_2017'])}**; only in the sv_2023 lane: **{len(m['only_in_sv_2023'])}** (a non-zero here is a population breach, not a verdict).\n")
    out.append("| cell | rows | meaning |\n|---|---|---|")
    out.append(f"| `both_pass` | {cells.get('both_pass', 0)} | parses under both editions |")
    out.append(f"| `both_nonpass` | {cells.get('both_nonpass', 0)} | parses under neither |")
    out.append(f"| `2023_only_pass` | {cells.get('2023_only_pass', 0)} | the 1800-2023 widening the corpus exercises — `must_accept` on a 2023 lane (`.14b`) |")
    out.append(f"| `2017_only_pass` | {cells.get('2017_only_pass', 0)} | a 2023-profile REGRESSION candidate — a parser question until adjudicated |")
    out.append("")
    out.append("### Status pairs (sv_2017, sv_2023)\n")
    out.append("| sv_2017 | sv_2023 | rows |\n|---|---|---|")
    for (a, b), n in sorted(m["status_pairs"].items(), key=lambda kv: (-kv[1], kv[0])):
        out.append(f"| `{a}` | `{b}` | {n} |")
    out.append("")
    out.append("## Every row whose verdict differs\n")
    if not d["detail"]:
        out.append("(none — the two editions agree on every row)\n")
    else:
        out.append("The 2017-lane adjudication class is carried beside each row so the reader sees whether the row is already "
                   "ruled (an edition contrast keyed `must_reject` on the 2017 lane is exactly a `2023_only_pass` row).\n")
        out.append("| cell | 2017-lane adjudication | rows |\n|---|---|---|")
        for (cell, a), n in sorted(adj.items()):
            out.append(f"| `{cell}` | `{a}` | {n} |")
        out.append("")
        out.append("| cell | suite | relpath | sv_2017 (furthest) | sv_2023 (furthest) | 2017-lane adjudication | expected |\n|---|---|---|---|---|---|---|")
        for r in d["detail"]:
            out.append(f"| `{r['cell']}` | `{r['suite']}` | `{r['relpath']}` | {r['sv_2017']} ({r['furthest_sv_2017']}) | {r['sv_2023']} ({r['furthest_sv_2023']}) | `{r['adjudication_2017_lane']}` | `{r['expected_2017_lane']}` |")
        out.append("")
    if m["only_in_sv_2017"] or m["only_in_sv_2023"]:
        out.append("## Population breaches\n")
        for k in m["only_in_sv_2017"][:50]:
            out.append(f"- only in sv_2017: `{k[0]}` `{k[1]}`")
        for k in m["only_in_sv_2023"][:50]:
            out.append(f"- only in sv_2023: `{k[0]}` `{k[1]}`")
        out.append("")
    return "\n".join(out)


def headline(d: dict) -> str:
    c = d["matrix"]["cells"]
    return (f"SV2023-LANE-CENSUS: rows={d['matrix']['rows']} both_pass={c.get('both_pass', 0)} both_nonpass={c.get('both_nonpass', 0)} "
            f"2023_only_pass={c.get('2023_only_pass', 0)} 2017_only_pass={c.get('2017_only_pass', 0)} "
            f"population_breaches={len(d['matrix']['only_in_sv_2017']) + len(d['matrix']['only_in_sv_2023'])}")


def self_test() -> int:
    fails = []

    def check(name, cond):
        print(f"  {'PASS' if cond else 'FAIL'}  {name}")
        if not cond:
            fails.append(name)

    a = {("s", "x.sv"): "pass", ("s", "y.sv"): "fail", ("s", "z.sv"): "pass", ("s", "w.sv"): "timeout", ("s", "q.sv"): "fail"}
    b = {("s", "x.sv"): "pass", ("s", "y.sv"): "pass", ("s", "z.sv"): "fail", ("s", "w.sv"): "timeout", ("s", "q.sv"): "crash"}
    m = matrix(a, b)
    check("the four cells partition the joined rows", sum(m["cells"].values()) == m["rows"] == 5)
    check("a 2023-only pass is classified and named", m["cells"].get("2023_only_pass") == 1 and ("2023_only_pass", ("s", "y.sv"), "fail", "pass") in m["moved"])
    check("a 2017-only pass is classified and named", m["cells"].get("2017_only_pass") == 1 and ("2017_only_pass", ("s", "z.sv"), "pass", "fail") in m["moved"])
    check("timeout and crash are not-pass, and their pair is counted", m["cells"].get("both_nonpass") == 2 and m["status_pairs"].get(("fail", "crash")) == 1)
    m2 = matrix(a, {**b, ("s", "extra.sv"): "pass"})
    check("a key in one lane only is a POPULATION BREACH, counted, never dropped", m2["only_in_sv_2023"] == [("s", "extra.sv")] and m2["rows"] == 5)
    sha = "a" * 64
    ra = {"parse binary": ("rust/target/release/parseability_probe", sha), "grammar raw ast": ("grammars/systemverilog.ebnf", "b" * 64), "results (output)": ("r", "c" * 64), "positions (output)": ("p", "d" * 64)}
    rb = dict(ra)
    ok = verify_identity(ra, rb, {"parse binary": sha, "sv_2017:results (output)": "c" * 64, "sv_2017:positions (output)": "d" * 64, "sv_2023:results (output)": "c" * 64, "sv_2023:positions (output)": "d" * 64})
    check("matching identities are sound", ok == [])
    rb2 = dict(ra); rb2["parse binary"] = ("rust/target/release/parseability_probe", "e" * 64)
    check("two lanes naming different binaries REFUSE", any("different `parse binary`" in r for r in verify_identity(ra, rb2, {"parse binary": sha})))
    check("a binary on disk that is not the reports' REFUSES", any("on disk" in r for r in verify_identity(ra, rb, {"parse binary": "f" * 64})))
    check("a raw dump that does not re-hash to its report's OUTPUT row REFUSES",
          any("does not re-hash" in r for r in verify_identity(ra, rb, {"parse binary": sha, "sv_2023:results (output)": "0" * 64})))
    check("a report without an identity row REFUSES", any("no `grammar raw ast` row" in r for r in verify_identity(ra, {"parse binary": ra["parse binary"]}, {})))
    rows = report_rows("| parse binary | `rust/target/release/parseability_probe` | `" + sha + "` |\n| parse binary | `x` | `" + "9" * 64 + "` |\n")
    check("the FIRST identity row of a label wins (the parameters table repeats the label without a sha)", rows["parse binary"][1] == sha)
    check("the join uses the manifest's suite-relative convention (subs and the folded uvm tree)",
          manifest_relpath("verilator", "stimuli/sv/subs/verilator/test_regress/t/a.v") == "test_regress/t/a.v"
          and manifest_relpath("uvm-core", "stimuli/sv/uvm/src/base/x.svh") == "src/base/x.svh"
          and manifest_relpath("s", "already/relative.sv") == "already/relative.sv")
    print(f"SV2023-LANE-CENSUS-SELFTEST: arms=12 failed={len(fails)}")
    return 1 if fails else 0


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--check", action="store_true")
    ap.add_argument("--self-test", action="store_true")
    a = ap.parse_args()
    if a.self_test:
        return self_test()
    d = derive()
    md, tsv = render_md(d), render_tsv(d)
    if a.check:
        cur_md = OUT_MD.read_text(encoding="utf-8") if OUT_MD.is_file() else ""
        cur_tsv = OUT_TSV.read_text(encoding="utf-8") if OUT_TSV.is_file() else ""
        if cur_md != md or cur_tsv != tsv:
            print(f"sv2023-lane-census: STALE — {OUT_MD.relative_to(ROOT)} / .tsv differ from a fresh derivation; re-run without --check", file=sys.stderr)
            print(headline(d))
            return 1
        print(headline(d) + " (current)")
        return 0
    OUT_MD.write_text(md, encoding="utf-8"); OUT_TSV.write_text(tsv, encoding="utf-8")
    print(headline(d))
    print(f"wrote {OUT_MD.relative_to(ROOT)} and {OUT_TSV.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
