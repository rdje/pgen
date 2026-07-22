#!/usr/bin/env python3
"""SV external-corpus adjudication manifest builder (SV-CORPUS-GRAD.2).

Turns the raw parse outcomes of stimuli/run_external_corpus.sh sv
(stimuli/sv/characterization/results.tsv) into the expected-vs-actual
adjudication manifest the graduation campaign burns down from
(docs/tasks/SV-CORPUS-GRAD.md leaf .2).

Doctrine (corpus-expected-from-SPEC, never from the fix):
  Every expected verdict is derived ONLY from suite metadata / upstream driver
  conventions / LRM-grounded pinned rulings — never from what the parser
  currently does. A raw corpus fail is NOT automatically a parser bug; this
  manifest is what separates the defect signal from the intended-fail /
  harness-dependency population.

Expected-verdict taxonomy (leaf .1 design):
  must_accept              valid SV at parse level (post-parse should-fails included)
  must_reject              parse/lexical-level intentional invalidity
  chained_only             adjudicable only with include/library chaining (leaf .4)
  out_of_scope_with_cause  owned by another lane (cause named, e.g. svpp)

Adjudication classes:
  match                                  observed == expected
  divergence:unexplained_rejects_valid   expected accept, parser rejected  <- defect signal
  divergence:unexplained_accepts_invalid expected reject, parser accepted  <- defect signal
  divergence:explained_svpp_include      expected accept, rejected, file needs `include
  divergence:explained_svpp_macro_use    ... file expands non-standard macros
  divergence:explained_svpp_conditional  ... file relies on `ifdef conditionals
  divergence:explained_timeout           the tracked pathological-input population
  deferred:<cause>                       chained_only / out_of_scope rows (no verdict here)

Deterministic: output ordering is (suite, relpath); content-derived only.

Usage:
  python3 stimuli/sv/adjudicate_external_corpus.py \
      [--results stimuli/sv/characterization/results.tsv] \
      [--subs-root stimuli/sv/subs] \
      [--out-manifest stimuli/sv/characterization/adjudication_manifest.tsv] \
      [--out-summary stimuli/sv/characterization/adjudication_summary.md]
"""

import argparse
import re
import sys
from collections import Counter, defaultdict
from pathlib import Path

# --- pinned per-file rulings (LRM-grounded; sv-tests files whose headers lack a
# `:type:` stage list so the stage of the intended failure must be adjudicated
# from the stated reason). Reasons quoted from the files' :should_fail_because:.
SVTESTS_PINNED = {
    "tests/sanity.sv": (
        "must_reject", "pinned: header says explicitly 'syntax error, fails during parsing'"),
    "tests/chapter-11/11.3.6--assign_in_expr_inv.sv": (
        "must_reject", "pinned: LRM 11.3.6 requires parentheses around operator "
        "assignment in expressions - 'a = b = c = 5;' is a grammar-level violation"),
    "tests/chapter-6/6.9.2--vector_vectored_inv.sv": (
        "must_accept", "pinned: bit-select on a vectored net is a semantic "
        "(elaboration) restriction; the syntax itself parses"),
    "tests/chapter-5/5.7.2-real-constants-illegal.sv": (
        "must_reject", "pinned: LRM 5.7.2 lexical rule - real literals need a "
        "digit on each side of the decimal point ('.12', '9.', '4.E3')"),
    "tests/chapter-5/5.6--wrong-identifiers.sv": (
        "must_reject", "pinned: LRM 5.6 lexical rule - identifier may not start "
        "with a digit or $ ('reg 0number;')"),
    "tests/chapter-5/5.7.1--integers-unsized-illegal.sv": (
        "must_reject", "pinned: LRM 5.7.1 lexical rule - '4af' needs 'h"),
    "tests/chapter-5/5.7.1--integers-signed-illegal.sv": (
        "must_reject", "pinned: LRM 5.7.1 - \"8'd-6\" illegal negative literal syntax"),
}

# Standard preprocessor/compiler directives that are NOT user-macro expansion.
KNOWN_DIRECTIVES = {
    "define", "include", "ifdef", "ifndef", "else", "elsif", "endif", "undef",
    "undefineall", "timescale", "default_nettype", "celldefine", "endcelldefine",
    "resetall", "line", "pragma", "begin_keywords", "end_keywords",
    "unconnected_drive", "nounconnected_drive", "__FILE__", "__LINE__",
}

META_RE = re.compile(r"^:([a-z_]+):\s*(.*)$")
TICK_RE = re.compile(r"`([A-Za-z_][A-Za-z0-9_$]*)")
INCLUDE_RE = re.compile(r"^\s*`include", re.MULTILINE)
COND_RE = re.compile(r"^\s*`(ifdef|ifndef|elsif)\b", re.MULTILINE)
FAILS_TRUE_RE = re.compile(r"fails\s*=\s*(True|test\.vlt_all)\b")
TOP_FILENAME_RE = re.compile(r"top_filename\s*=\s*[\"']([^\"']+)[\"']")
SYNTAX_ERR_RE = re.compile(r"syntax error", re.IGNORECASE)


def read_text(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8", errors="replace")
    except OSError:
        return ""


def sv_tests_metadata(text: str):
    fields = {}
    for line in text.splitlines()[:80]:
        m = META_RE.match(line)
        if m:
            fields.setdefault(m.group(1), m.group(2).strip())
    return fields


COMMENT_STRING_RE = re.compile(
    r"//[^\n]*|/\*.*?\*/|\"(?:[^\"\\\n]|\\.)*\"", re.DOTALL)


def strip_comments_and_strings(text: str) -> str:
    """Remove comments and string literals so directives inside them do not
    count as preprocessing dependencies (the lexer never sees them)."""
    return COMMENT_STRING_RE.sub(" ", text)


def preproc_dependency(raw_text: str):
    """Return the strongest svpp dependency flag for a file, or ''."""
    text = strip_comments_and_strings(raw_text)
    if INCLUDE_RE.search(text):
        return "include"
    for name in TICK_RE.findall(text):
        if name not in KNOWN_DIRECTIVES:
            return "macro_use"
    if COND_RE.search(text):
        return "conditional"
    return ""


def expect_sv_tests(relpath: str, text: str):
    if relpath in SVTESTS_PINNED:
        return SVTESTS_PINNED[relpath]
    if relpath.endswith(".svh"):
        return ("chained_only", "sv-tests: .svh include payload, not a standalone test")
    fields = sv_tests_metadata(text)
    if not fields:
        return ("must_accept", "sv-tests: no metadata header (positive-dominant default)")
    types = set(fields.get("type", "").split())
    should_fail = "should_fail_because" in fields
    if "preprocessing" in types:
        return ("out_of_scope_with_cause",
                "sv-tests: :type: includes preprocessing - svpp-owned conformance "
                "(SVPP-EXPANSION lane)")
    if should_fail:
        if "parsing" in types:
            return ("must_reject", "sv-tests: :should_fail_because: with :type: parsing")
        return ("must_accept",
                "sv-tests: :should_fail_because: at post-parse stage(s) "
                f"({' '.join(sorted(types)) or 'untyped'}) - syntax itself is valid")
    return ("must_accept", "sv-tests: positive test")


class VerilatorIndex:
    """Answer key from t/*.py driver conventions + t/*.out goldens."""

    def __init__(self, t_dir: Path):
        self.fail_stems = set()
        self.pass_stems = set()
        self.syntax_out_stems = set()
        self.top_ref = {}  # referenced .v stem -> driver stem
        if not t_dir.is_dir():
            return
        for py in sorted(t_dir.glob("*.py")):
            text = read_text(py)
            stem = py.stem
            if FAILS_TRUE_RE.search(text):
                self.fail_stems.add(stem)
            else:
                self.pass_stems.add(stem)
            for ref in TOP_FILENAME_RE.findall(text):
                self.top_ref.setdefault(Path(ref).stem, stem)
        for out in sorted(t_dir.glob("*.out")):
            if SYNTAX_ERR_RE.search(read_text(out)):
                self.syntax_out_stems.add(out.stem)

    def expect(self, relpath: str):
        # relpath is relative to the verilator submodule root.
        parts = Path(relpath).parts
        if len(parts) != 3 or parts[0] != "test_regress" or parts[1] != "t":
            return ("chained_only",
                    "verilator: multi-file test payload outside flat test_regress/t/")
        stem = Path(parts[2]).stem
        if stem.startswith(("t_pp_", "t_preproc_")):
            # The test's target is the PREPROCESSOR (mirrors the sv-tests
            # ':type: preprocessing' rule): its verdicts belong to the svpp
            # lane, not to isolation parse of the raw text.
            return ("out_of_scope_with_cause",
                    "verilator: preprocessor-target test (t_pp_*/t_preproc_*) - "
                    "svpp-owned conformance")
        driver = None
        if stem in self.fail_stems or stem in self.pass_stems:
            driver = stem
        elif stem in self.top_ref:
            driver = self.top_ref[stem]
        if driver is None:
            return ("chained_only",
                    "verilator: no same-stem driver and never a top_filename - "
                    "include/companion fragment")
        if driver in self.fail_stems:
            if driver in self.syntax_out_stems or stem in self.syntax_out_stems:
                return ("must_reject",
                        f"verilator: driver {driver}.py fails=True and golden "
                        f".out reports a syntax error")
            return ("must_accept",
                    f"verilator: driver {driver}.py fails=True but golden .out is "
                    "a post-parse tool error (lint/elab/unsupported) - syntax valid")
        return ("must_accept", f"verilator: driver {driver}.py expects success")


def expect_slang(relpath: str):
    p = relpath.replace("\\", "/")
    if p.startswith("tests/unittests/data/"):
        sub = p[len("tests/unittests/data/"):]
        if "/" in sub:
            return ("chained_only",
                    "slang: unittest library/multi-file fixture directory")
        return ("must_accept", "slang: unittest data fixture (positive-dominant)")
    if p.startswith("tests/regression/"):
        return ("must_accept",
                "slang: regression/driver fixture - strict-mode errors are "
                "semantic, syntax is valid")
    return ("must_accept", "slang: fixture (positive-dominant default)")


def expect_verible(relpath: str):
    p = relpath.replace("\\", "/")
    if "/kythe/testdata/" in p:
        tail = p.split("/kythe/testdata/", 1)[1]
        if "/" in tail:
            return ("chained_only", "verible: kythe multi-file/include fixture directory")
        return ("must_accept", "verible: kythe single-file indexing fixture")
    if "/lint/testdata/" in p:
        return ("must_accept", "verible: lint-rule fixture (style findings, valid syntax)")
    return ("must_accept", "verible: fixture (positive-dominant default)")


DESIGN_SUITES = {"Cores-VeeR-EL2", "friscv", "scr1"}


def adjudicate(expected, observed, dep_flag):
    if observed == "timeout":
        # Known-pathological tracking outranks deferral: a hang is surfaced
        # no matter which lane owns the file.
        return "divergence:explained_timeout"
    if expected == "chained_only":
        return "deferred:chained_only"
    if expected == "out_of_scope_with_cause":
        return "deferred:svpp_owned"
    if expected == "must_accept":
        if observed == "pass":
            return "match"
        if dep_flag:
            return f"divergence:explained_svpp_{dep_flag}"
        return "divergence:unexplained_rejects_valid"
    if expected == "must_reject":
        if observed == "fail":
            return "match"
        return "divergence:unexplained_accepts_invalid"
    raise ValueError(f"unknown expected verdict {expected!r}")


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    root = Path(__file__).resolve().parent.parent.parent  # repo root
    ap.add_argument("--results", default=root / "stimuli/sv/characterization/results.tsv",
                    type=Path)
    ap.add_argument("--subs-root", default=root / "stimuli/sv/subs", type=Path)
    ap.add_argument("--out-manifest",
                    default=root / "stimuli/sv/characterization/adjudication_manifest.tsv",
                    type=Path)
    ap.add_argument("--out-summary",
                    default=root / "stimuli/sv/characterization/adjudication_summary.md",
                    type=Path)
    args = ap.parse_args()

    rows = []
    for line in args.results.read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        suite, observed, path = line.split("\t")
        rel = path.split(f"/subs/{suite}/", 1)[1]
        rows.append((suite, rel, observed))
    rows.sort()

    vidx = VerilatorIndex(args.subs_root / "verilator/test_regress/t")

    manifest = []
    for suite, rel, observed in rows:
        fpath = args.subs_root / suite / rel
        text = read_text(fpath)
        if suite in DESIGN_SUITES:
            expected, basis = ("chained_only",
                               "design corpus parsed in isolation - honest adjudication "
                               "needs include/define chaining (leaf .4)")
        elif suite == "sv-tests":
            expected, basis = expect_sv_tests(rel, text)
        elif suite == "verilator":
            expected, basis = vidx.expect(rel)
        elif suite == "slang":
            expected, basis = expect_slang(rel)
        elif suite == "verible":
            expected, basis = expect_verible(rel)
        else:
            raise SystemExit(f"unknown suite {suite!r} in results.tsv")
        dep_flag = ""
        if expected in ("must_accept", "must_reject"):
            dep_flag = preproc_dependency(text)
        if expected == "must_reject" and dep_flag == "include":
            # The intended-bad content may live in the `include'd file, which
            # isolation parse never sees - the reject expectation is only
            # meaningful at chain level (leaf .4).
            expected = "chained_only"
            basis += " - but `include-dependent: reject expectation is chain-level"
        if expected != "must_accept":
            dep_flag = ""
        verdict = adjudicate(expected, observed, dep_flag)
        manifest.append((suite, rel, observed, expected, verdict, basis))

    args.out_manifest.parent.mkdir(parents=True, exist_ok=True)
    with args.out_manifest.open("w", encoding="utf-8") as fh:
        fh.write("suite\trelpath\tobserved\texpected\tadjudication\tbasis\n")
        for row in manifest:
            fh.write("\t".join(row) + "\n")

    # --- summary
    per_suite = defaultdict(Counter)
    total = Counter()
    for suite, _rel, _obs, _exp, verdict, _basis in manifest:
        per_suite[suite][verdict] += 1
        total[verdict] += 1

    def bucket(counter):
        match = counter.get("match", 0)
        unexplained = sum(v for k, v in counter.items()
                          if k.startswith("divergence:unexplained"))
        explained = sum(v for k, v in counter.items()
                        if k.startswith("divergence:explained"))
        deferred = sum(v for k, v in counter.items() if k.startswith("deferred:"))
        return match, unexplained, explained, deferred

    lines = []
    lines.append("# SV external-corpus adjudication summary (leaf SV-CORPUS-GRAD.2)")
    lines.append("")
    lines.append(f"Input: `results.tsv` ({len(manifest)} rows); generator: "
                 "`stimuli/sv/adjudicate_external_corpus.py` (deterministic).")
    lines.append("")
    lines.append("| suite | rows | match | UNEXPLAINED div | explained div | deferred |")
    lines.append("|---|---|---|---|---|---|")
    for suite in sorted(per_suite):
        c = per_suite[suite]
        m, u, e, d = bucket(c)
        lines.append(f"| {suite} | {sum(c.values())} | {m} | {u} | {e} | {d} |")
    m, u, e, d = bucket(total)
    lines.append(f"| **total** | **{len(manifest)}** | **{m}** | **{u}** | **{e}** | **{d}** |")
    lines.append("")
    lines.append("## Verdict-class detail")
    lines.append("")
    lines.append("| class | count |")
    lines.append("|---|---|")
    for cls in sorted(total):
        lines.append(f"| {cls} | {total[cls]} |")
    lines.append("")
    lines.append("**The graduation burn-down baseline = the UNEXPLAINED divergence count** "
                 f"(**{u}**: rejects-valid "
                 f"{total.get('divergence:unexplained_rejects_valid', 0)}, "
                 f"accepts-invalid "
                 f"{total.get('divergence:unexplained_accepts_invalid', 0)}). "
                 "Explained divergences are svpp/chaining/timeout-owned with named "
                 "causes; deferred rows adjudicate in their owning lanes "
                 "(leaf .4 chaining, SVPP lane).")
    lines.append("")
    args.out_summary.write_text("\n".join(lines) + "\n", encoding="utf-8")

    print(f"manifest: {args.out_manifest} ({len(manifest)} rows)")
    print(f"summary:  {args.out_summary}")
    m, u, e, d = bucket(total)
    print(f"match={m} unexplained={u} explained={e} deferred={d}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
