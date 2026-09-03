#!/usr/bin/env python3
"""SV-CORPUS-GRAD.13x — the suite-intended-invalid `must_accept` census (MEASUREMENT, never a ruling).

WHAT IT ANSWERS
  Which rows of the axis-2 bar sit on a path the OWNING SUITE marks as intended-invalid (`_bad`,
  `test/error/`, `should_fail`, `_fail`) while the answer key says `must_accept` — and, for each,
  WHERE the shipped parser stops, WHAT the source says there, and WHAT the upstream tool said. The
  RULING for each row lives in `stimuli/sv/adjudicate_external_corpus.py`'s pin tables (the answer
  key), never here: this census re-measures the population and joins the manifest's verdict back
  onto it, so the same table shows the rows BEFORE and AFTER each ruling slice.

WHY THE POPULATION IS DERIVED, NOT LISTED
  `.13k` ruled that an answer key resting on a SUITE's word is evidence about the suite; a path the
  suite itself names `_bad` is the cheapest such signal there is. The regex is the whole selection
  rule and is printed in the report, so a reader can widen or narrow it and re-derive. One EXTRA
  row is admitted by name — `t_timescale_udp.v` — because `.3.30` found its text LRM-illegal
  (`?` in a combinational output field) without a `_bad` path to flag it; the reason is carried
  beside the row and the row is counted separately.

WHAT EACH ROW CARRIES (all measured by THIS run, on THIS tree)
  * the release probe's verdict and `furthest_position` under `sv_2017` (the lane) AND `sv_2023`
    (so an edition contrast is visible in the same row);
  * the 1-based line / 0-based column of the furthest byte and the source line there;
  * agreement with the lane's tracked `positions.tsv` (a disagreement means the tracked report and
    the probe on disk describe different parsers — refuse to read further, re-measure the lane);
  * the upstream testimony the key rests on: verilator's first `%…` golden line, sv2v's
    `// pattern:` header, ivtest's basis text;
  * the manifest's `expected` / `adjudication` / `basis` head — the ruling state.

Usage:
  python3 docs/tasks/artifacts/sv_corpus_grad/answer_key_intended_invalid/census.py            # write after/census.tsv + after/census.md (frozen at .13x.f)
  python3 docs/tasks/artifacts/sv_corpus_grad/answer_key_intended_invalid/census.py --check    # re-derive and diff (rc 1 on drift)
  python3 docs/tasks/artifacts/sv_corpus_grad/answer_key_intended_invalid/census.py --self-test
"""
from __future__ import annotations

import argparse
import csv
import hashlib
import io
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]
HERE = Path(__file__).resolve().parent
MANIFEST = ROOT / "stimuli/sv/characterization/adjudication_manifest.tsv"
POSITIONS = ROOT / "stimuli/sv/characterization/positions.tsv"
PROBE = ROOT / "rust/target/release/parseability_probe"
SUBS = ROOT / "stimuli/sv/subs"
# SV-CORPUS-GRAD.13x.f: the leaf closed with 47 of its 48 rows adjudicated (the 48th is .3.31's defect), so the
# record is FROZEN under after/ - a path the identity-carrier doctrine reports and never judges. Re-running
# reproduces the closing state as long as the manifest still carries those rows; a later manifest makes
# --check report drift, which is the honest reading of a frozen snapshot.
OUT_TSV = HERE / "after" / "census.tsv"
OUT_MD = HERE / "after" / "census.md"

# The selection rule. A suite's own convention for "this file is meant to fail": verilator's
# `_bad` stem infix, sv2v's `test/error/` directory, ivtest's `_fail` stem infix, sv-tests'
# `should_fail`. Widening it is a one-line edit whose effect this report re-derives.
INTENDED_INVALID_RE = re.compile(r"_bad|/error/|should_fail|_fail")
BAR_CLASS = "divergence:unexplained_rejects_valid"
KEY = "must_accept"
# Rows admitted by NAME, with the leaf that found the text LRM-illegal and the reason.
EXTRA = {
    ("verilator", "test_regress/t/t_timescale_udp.v"):
        "SV-CORPUS-GRAD.3.30: line 39 `? : ?;` puts `?` in a combinational output field; "
        "A.5.3 output_symbol ::= 0 | 1 | x | X derives no `?`",
}
COLUMNS = ["suite", "relpath", "member", "line", "col", "sv_2017", "furthest_sv_2017", "sv_2023",
           "furthest_sv_2023", "tracked_pos", "tracked_agrees", "source_line", "upstream",
           "expected", "adjudication", "basis_head"]


def read_manifest():
    with MANIFEST.open(encoding="utf-8") as fh:
        return list(csv.DictReader(fh, delimiter="\t"))


def read_positions():
    out = {}
    with POSITIONS.open(encoding="utf-8") as fh:
        for line in fh:
            parts = line.rstrip("\n").split("\t")
            if len(parts) >= 3:
                out[parts[1]] = int(parts[2])
    return out


def population(rows):
    """(suite, relpath, member) for every bar row on an intended-invalid path, plus EXTRA."""
    pop = []
    for r in rows:
        key = (r["suite"], r["relpath"])
        if r["adjudication"] == BAR_CLASS and r["expected"] == KEY and INTENDED_INVALID_RE.search(r["relpath"]):
            pop.append((r["suite"], r["relpath"], "regex"))
        elif key in EXTRA and r["adjudication"] == BAR_CLASS and r["expected"] == KEY:
            pop.append((r["suite"], r["relpath"], "extra"))
    return sorted(pop)


def probe(path: Path, profile: str):
    proc = subprocess.run([str(PROBE), "--parse", "systemverilog", str(path), "--profile", profile],
                          capture_output=True, text=True)
    text = (proc.stdout or "") + (proc.stderr or "")
    if "parse_full passed" in text:
        return "pass", -1
    m = re.search(r"furthest_position=(\d+)", text)
    return "fail", (int(m.group(1)) if m else -1)


def line_col(text: str, pos: int):
    if pos < 0:
        return -1, -1, ""
    pre = text[:pos]
    line = pre.count("\n") + 1
    col = pos - (pre.rfind("\n") + 1)
    lines = text.split("\n")
    src = lines[line - 1].strip() if 0 < line <= len(lines) else ""
    return line, col, src


def upstream(suite: str, fp: Path, text: str) -> str:
    if suite == "verilator":
        g = fp.with_suffix(".out")
        if g.exists():
            for gl in g.read_text(errors="replace").splitlines():
                if gl.startswith("%"):
                    return gl.strip()
        return "(no golden .out)"
    if suite == "sv2v":
        for l in text.splitlines()[:3]:
            if l.startswith("// pattern:"):
                return l[len("// pattern:"):].strip()
        return "(no // pattern: header)"
    return ""


def measure(rows, positions):
    by_key = {(r["suite"], r["relpath"]): r for r in rows}
    out = []
    for suite, rel, member in population(rows):
        fp = SUBS / suite / rel
        text = fp.read_bytes().decode("utf-8", "replace")
        v17, f17 = probe(fp, "sv_2017")
        v23, f23 = probe(fp, "sv_2023")
        line, col, src = line_col(text, f17)
        tracked = positions.get(f"stimuli/sv/subs/{suite}/{rel}", -1)
        r = by_key[(suite, rel)]
        out.append({
            "suite": suite, "relpath": rel, "member": member, "line": line, "col": col,
            "sv_2017": v17, "furthest_sv_2017": f17, "sv_2023": v23, "furthest_sv_2023": f23,
            "tracked_pos": tracked, "tracked_agrees": "yes" if tracked == f17 else "NO",
            "source_line": src, "upstream": upstream(suite, fp, text),
            "expected": r["expected"], "adjudication": r["adjudication"],
            "basis_head": r["basis"][:110],
        })
    return out


def render_tsv(measured) -> str:
    buf = io.StringIO()
    w = csv.DictWriter(buf, fieldnames=COLUMNS, delimiter="\t", lineterminator="\n")
    w.writeheader()
    for m in measured:
        w.writerow(m)
    return buf.getvalue()


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def render_md(measured) -> str:
    by_suite, by_adj, by_member = {}, {}, {}
    for m in measured:
        by_suite[m["suite"]] = by_suite.get(m["suite"], 0) + 1
        by_adj[m["adjudication"]] = by_adj.get(m["adjudication"], 0) + 1
        by_member[m["member"]] = by_member.get(m["member"], 0) + 1
    disagree = [m for m in measured if m["tracked_agrees"] != "yes"]
    contrast = [m for m in measured if m["sv_2017"] == "fail" and m["sv_2023"] == "pass"]
    L = []
    L.append("# Suite-intended-invalid `must_accept` census — SV-CORPUS-GRAD.13x")
    L.append("")
    L.append("> DERIVED. Re-run: `python3 docs/tasks/artifacts/sv_corpus_grad/answer_key_intended_invalid/census.py`")
    L.append(">")
    L.append("> ⛔ **A MEASUREMENT, NOT A RULING.** Every row's `expected` / `adjudication` is read back from the")
    L.append("> tracked manifest; the rulings live in `stimuli/sv/adjudicate_external_corpus.py`'s pin tables and")
    L.append("> move only on an LRM citation (`.13k`'s discipline). This table shows the population before and")
    L.append("> after each ruling slice.")
    L.append("")
    L.append(f"**Population: {len(measured)} rows** = `{INTENDED_INVALID_RE.pattern}` on a `{BAR_CLASS}` + "
             f"`{KEY}` row ({by_member.get('regex', 0)}) + named EXTRA ({by_member.get('extra', 0)}).")
    L.append("")
    L.append("| suite | rows |\n|---|---:|")
    for s, n in sorted(by_suite.items(), key=lambda kv: -kv[1]):
        L.append(f"| `{s}` | {n} |")
    L.append("")
    L.append("| adjudication (manifest, as of this run) | rows |\n|---|---:|")
    for a, n in sorted(by_adj.items(), key=lambda kv: -kv[1]):
        L.append(f"| `{a}` | {n} |")
    L.append("")
    L.append(f"Tracked `positions.tsv` agrees with the live probe on **{len(measured) - len(disagree)} of {len(measured)}** rows"
             + ("." if not disagree else f" — ⛔ **{len(disagree)} DISAGREE**: the tracked lane describes a different parser; re-measure before reading on."))
    L.append(f"Edition contrast (`sv_2017` fail, `sv_2023` pass): **{len(contrast)}** rows"
             + (" — " + ", ".join(f"`{m['relpath']}`" for m in contrast) if contrast else "."))
    L.append("")
    L.append("## Rows")
    L.append("")
    L.append("| # | suite | relpath | line:col | sv_2017 | sv_2023 | source at furthest | upstream testimony | expected | adjudication |")
    L.append("|---:|---|---|---|---|---|---|---|---|---|")
    for i, m in enumerate(measured, 1):
        def cell(s):
            return str(s).replace("|", "\\|").replace("`", "'")
        L.append(f"| {i} | `{m['suite']}` | `{m['relpath']}`{' (extra)' if m['member'] == 'extra' else ''} | "
                 f"{m['line']}:{m['col']} | {m['sv_2017']}@{m['furthest_sv_2017']} | {m['sv_2023']}@{m['furthest_sv_2023']} | "
                 f"`{cell(m['source_line'])[:70]}` | {cell(m['upstream'])[:110]} | `{m['expected']}` | `{m['adjudication']}` |")
    L.append("")
    L.append("## Instrument identity")
    L.append("")
    L.append("Re-hash these inputs; if any hash differs from the row below, **this report no longer")
    L.append("describes your tree** and the honest act is to re-measure, not to quote.")
    L.append("")
    L.append("| input | sha256 |\n| --- | --- |")
    L.append(f"| `stimuli/sv/characterization/adjudication_manifest.tsv` | `{sha(MANIFEST)}` |")
    L.append(f"| `stimuli/sv/characterization/positions.tsv` | `{sha(POSITIONS)}` |")
    L.append(f"| `docs/tasks/artifacts/sv_corpus_grad/answer_key_intended_invalid/census.py` | `{sha(Path(__file__).resolve())}` |")
    L.append(f"| `rust/target/release/parseability_probe` (parse binary) | `{sha(PROBE)}` |")
    L.append("")
    return "\n".join(L) + "\n"


def self_test() -> int:
    fails = 0

    def arm(name, ok):
        nonlocal fails
        print(f"  [{'ok' if ok else 'FAIL'}] {name}")
        fails += 0 if ok else 1
    arm("regex picks a verilator _bad stem", bool(INTENDED_INVALID_RE.search("test_regress/t/t_x_bad.v")))
    arm("regex picks an sv2v error dir", bool(INTENDED_INVALID_RE.search("test/error/x.sv")))
    arm("regex picks an ivtest _fail stem", bool(INTENDED_INVALID_RE.search("ivtest/ivltests/sv_x_fail3.v")))
    arm("regex leaves a plain path alone", not INTENDED_INVALID_RE.search("test_regress/t/t_udp_sequential.v"))
    rows = [
        {"suite": "verilator", "relpath": "test_regress/t/a_bad.v", "expected": KEY, "adjudication": BAR_CLASS, "basis": ""},
        {"suite": "verilator", "relpath": "test_regress/t/a_bad.v", "expected": "must_reject", "adjudication": "match", "basis": ""},
        {"suite": "verilator", "relpath": "test_regress/t/plain.v", "expected": KEY, "adjudication": BAR_CLASS, "basis": ""},
        {"suite": "verilator", "relpath": "test_regress/t/t_timescale_udp.v", "expected": KEY, "adjudication": BAR_CLASS, "basis": ""},
    ]
    pop = population(rows)
    arm("population = the regex row + the EXTRA row, never a match or a plain path",
        pop == [("verilator", "test_regress/t/a_bad.v", "regex"), ("verilator", "test_regress/t/t_timescale_udp.v", "extra")])
    arm("line/col mapping: byte 7 of 'ab\\ncdef\\n' is line 2 col 4", line_col("ab\ncdef\n", 7) == (2, 4, "cdef"))
    arm("line/col mapping: a pass (pos -1) maps to no line", line_col("x", -1) == (-1, -1, ""))
    arm("tracked manifest is present", MANIFEST.is_file() and POSITIONS.is_file())
    if MANIFEST.is_file():
        arm("live population is non-empty (a census over zero rows is not a census)", len(population(read_manifest())) > 0)
    print(f"ANSWER-KEY-INTENDED-INVALID-CENSUS: self-test arms={9 if MANIFEST.is_file() else 8} failed={fails}")
    return 1 if fails else 0


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--check", action="store_true", help="re-derive and diff against the tracked artifacts")
    ap.add_argument("--self-test", action="store_true")
    a = ap.parse_args()
    if a.self_test:
        return self_test()
    if not PROBE.is_file():
        print(f"REFUSING: release probe missing at {PROBE.relative_to(ROOT)} — build it before measuring", file=sys.stderr)
        return 2
    if not SUBS.is_dir() or not any(SUBS.iterdir()):
        print("NOT EVALUATED: vendored corpora (git submodules) are absent", file=sys.stderr)
        return 2
    measured = measure(read_manifest(), read_positions())
    tsv, md = render_tsv(measured), render_md(measured)
    if a.check:
        drift = []
        if not OUT_TSV.is_file() or OUT_TSV.read_text() != tsv:
            drift.append(str(OUT_TSV.relative_to(ROOT)))
        if not OUT_MD.is_file() or OUT_MD.read_text() != md:
            drift.append(str(OUT_MD.relative_to(ROOT)))
        summary = summarize(measured)
        if drift:
            print(f"ANSWER-KEY-INTENDED-INVALID-CENSUS: {summary} DRIFT in {', '.join(drift)} — re-run without --check")
            return 1
        print(f"ANSWER-KEY-INTENDED-INVALID-CENSUS: {summary} current")
        return 0
    OUT_TSV.write_text(tsv)
    OUT_MD.write_text(md)
    print(f"ANSWER-KEY-INTENDED-INVALID-CENSUS: {summarize(measured)} written")
    return 0


def summarize(measured) -> str:
    adj = {}
    for m in measured:
        adj[m["adjudication"]] = adj.get(m["adjudication"], 0) + 1
    bar = adj.get(BAR_CLASS, 0)
    return (f"rows={len(measured)} still_in_bar={bar} " +
            " ".join(f"{k.split(':')[-1]}={v}" for k, v in sorted(adj.items())) +
            f" disagree={sum(1 for m in measured if m['tracked_agrees'] != 'yes')}")


if __name__ == "__main__":
    sys.exit(main())
