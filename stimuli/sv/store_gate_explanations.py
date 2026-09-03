#!/usr/bin/env python3
"""Store-gate explanations for `must_accept` rows the parser rejects (SV-CORPUS-GRAD.13w).

THE QUESTION. `SV-PARSE-STRICT.2` shipped the director-confirmed parser/elaborator boundary:
the parser REJECTS a provably-undeclared type/nettype identifier at parse time, and ACCEPTS when
it cannot prove (an unresolved `import pkg::*` is in scope). An answer key derived from Annex A
cannot see that boundary — the BNF has no notion of "declared" — so a single-file suite test that
uses a type it never declares (`T x;`, `extends uvm_object`, `intf intf_i;` with no instance
parentheses) is keyed `must_accept` and refused BY DESIGN. Such a row is a divergence EXPLAINED
by a design decision, exactly as `explained_svpp_*` is explained by the preprocessor lane, and it
has been sitting inside the axis-2 bar as if it were a defect.

THE PROOF, PER ROW, BY PARSING. Nothing here is relabelled by argument. A row earns the class
`divergence:explained_store_gate` only when ALL of these hold, each decided mechanically:

  1. the file's own bytes, with NOTHING but a minimal prelude of declarations prepended, PARSE —
     through the same fixed-point search `adjudicate_dark_worklist.py` uses (`drive`: harvest the
     names the parser DEMANDED AND LACKED at the failure, declare them, repeat; `minimize`:
     drop every declaration the parse does not need). The winning names and forms are published
     beside the verdict, so a reader can re-run the proof;
  2. no wildcard import in the file names a package the file does not declare — there the
     boundary says ACCEPT, so a rejection would be the gate being UNSOUND, a defect for
     `SV-PARSE-STRICT`, and the row stays in the bar with that reason;
  3. no package `export` — visibility the file cannot settle on its own;
  4. none of the demanded names is a type the LRM PREDEFINES (`CrossValType`/`CrossQueueType`
     inside a `cross`, IEEE 1800-2017 19.6.1.4): the file owes no declaration for those, so the
     demand itself is a grammar defect and the row stays in the bar.

THE MEMO. Measuring costs a real parse per round per row (~3 min for 250 rows), so the verdicts
are memoised in a TRACKED cache keyed by (lane, suite, relpath, file sha256, parser binary
sha256). A row is re-measured only when its bytes or the parser changed; a row that is no longer
a candidate is dropped, so the cache is always exactly the candidate population and doubles as
the triage report. The binary sha in the key is the one the lane's TRACKED report names, and a
measurement is REFUSED when the binary on disk is not that one: a verdict measured against a
different parser than the results would be a number about nothing.

CLI:
    python3 stimuli/sv/store_gate_explanations.py --self-test
    python3 stimuli/sv/store_gate_explanations.py --report          # the cache, summarised
"""
from __future__ import annotations

import argparse
import csv
import hashlib
import importlib.util
import re
import sys
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent
CLASS = "divergence:explained_store_gate"
LEAF = "SV-CORPUS-GRAD.13w"
DEFAULT_CACHE = ROOT / "stimuli/sv/characterization/store_gate_cache.tsv"
CACHE_COLUMNS = ["lane", "suite", "relpath", "file_sha256", "binary_sha256", "verdict", "names", "forms"]
PROFILES = {"sv_2017": "sv_2017", "verilog_2005": "verilog_2005"}

# Types the LRM predefines: the file owes no declaration, so a parser that demands one is wrong.
PREDEFINED_TYPE_NAMES = {
    "CrossValType": "IEEE 1800-2017 19.6.1.4 (predefined inside a `cross`)",
    "CrossQueueType": "IEEE 1800-2017 19.6.1.4 (predefined inside a `cross`)",
}
WILDCARD_IMPORT_RE = re.compile(r"\bimport\s+([A-Za-z_]\w*)\s*::\s*\*")
PACKAGE_DECL_RE = re.compile(r"\bpackage\b\s+(?:automatic\s+|static\s+)?([A-Za-z_]\w*)")
EXPORT_RE = re.compile(r"\bexport\s+(?:\*\s*::\s*\*|[A-Za-z_]\w*\s*::)")

EXPLAINED = "explained"
RESIDUALS = ("residual:parses_bare", "residual:not_parsed_with_declarations",
             "residual:unresolved_wildcard_import", "residual:package_export",
             "residual:predefined_type_name")

_MACHINERY = None


def machinery():
    """The dark-worklist search, imported by path — never copied."""
    global _MACHINERY
    if _MACHINERY is None:
        spec = importlib.util.spec_from_file_location(
            "pgen_adjudicate_dark_worklist", ROOT / "stimuli/sv/adjudicate_dark_worklist.py")
        m = importlib.util.module_from_spec(spec)
        sys.modules["pgen_adjudicate_dark_worklist"] = m
        spec.loader.exec_module(m)
        _MACHINERY = m
    return _MACHINERY


def soundness_reason(stripped: str, names) -> str | None:
    """Why a rejection may NOT be the sound gate — conditions 2..4 of the module docstring.
    `stripped` has comments and strings blanked, so a mention inside a comment is not a fact."""
    declared_packages = set(PACKAGE_DECL_RE.findall(stripped))
    for pkg in WILDCARD_IMPORT_RE.findall(stripped):
        if pkg not in declared_packages:
            return "residual:unresolved_wildcard_import"
    if EXPORT_RE.search(stripped):
        return "residual:package_export"
    if any(n in PREDEFINED_TYPE_NAMES for n in names):
        return "residual:predefined_type_name"
    return None


def measure(raw: str, profile: str) -> tuple:
    """(verdict, names, forms) for one file's bytes — condition 1 by parsing, then 2..4."""
    m = machinery()
    m.PROFILE = PROFILES[profile]
    ok0, _, _, _ = m.probe_text(raw, trace=False)
    if ok0:
        return ("residual:parses_bare", (), ())
    reserved = m.sv_reserved_words(m.GRAMMAR)
    none_wrapper = next(w for w in m.WRAPPERS if w[0] == "none")
    widest = m.FORM_LADDER[-1]
    ok, names, _rounds, _furthest = m.drive(raw, none_wrapper, widest, reserved)
    if not ok:
        return ("residual:not_parsed_with_declarations", tuple(sorted(names)), ())
    keep, forms = m.minimize(raw, none_wrapper, names, widest)
    reason = soundness_reason(m.blank_comments_and_strings(raw), keep)
    return (reason or EXPLAINED, tuple(sorted(keep)), tuple(forms))


def sha256_bytes(b: bytes) -> str:
    return hashlib.sha256(b).hexdigest()


def load_cache(path: Path) -> dict:
    out = {}
    if not path.is_file():
        return out
    with path.open(encoding="utf-8") as fh:
        rows = [l for l in fh if not l.startswith("#")]
    for r in csv.DictReader(rows, delimiter="\t"):
        out[(r["lane"], r["suite"], r["relpath"])] = r
    return out


def save_cache(path: Path, cache: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as fh:
        fh.write(f"# store-gate explanation memo ({LEAF}) - DERIVED by stimuli/sv/adjudicate_external_corpus.py;\n"
                 "# a row is re-measured when its file_sha256 or binary_sha256 no longer matches. See\n"
                 "# stimuli/sv/store_gate_explanations.py for what each verdict means.\n")
        fh.write("\t".join(CACHE_COLUMNS) + "\n")
        for key in sorted(cache):
            r = cache[key]
            fh.write("\t".join(str(r[c]) for c in CACHE_COLUMNS) + "\n")


def explain_rows(lane: str, candidates, report_binary_sha: str, binary_path: Path, cache: dict,
                 *, jobs: int = 8, measurer=None, log=print) -> dict:
    """Decide every candidate (suite, relpath, file_path) for `lane`; update `cache` in place so it
    holds exactly the candidate population for the lane. Returns {(suite, relpath): cache row}.

    `measurer(raw, profile) -> (verdict, names, forms)` is injectable so the memo logic is testable
    without a parser; the default is `measure` (a real parse). A measurement is REFUSED when the
    binary on disk is not the one the lane's report names."""
    measurer = measurer or measure
    profile = PROFILES[lane]
    wanted = {}
    todo = []
    for suite, rel, fpath in candidates:
        raw_bytes = Path(fpath).read_bytes()
        fsha = sha256_bytes(raw_bytes)
        key = (lane, suite, rel)
        hit = cache.get(key)
        if hit and hit["file_sha256"] == fsha and hit["binary_sha256"] == report_binary_sha:
            wanted[(suite, rel)] = hit
            continue
        todo.append((key, fsha, raw_bytes.decode("latin-1")))
    if todo:
        if not report_binary_sha:
            raise SystemExit(f"store-gate: {len(todo)} {lane} row(s) need measuring but the lane's report "
                             "names no `parse binary` digest - re-run the corpus lane first")
        if measurer is measure:
            if not Path(binary_path).is_file():
                raise SystemExit(f"store-gate: {len(todo)} {lane} row(s) need measuring and "
                                 f"{binary_path} is absent - build the release probe first")
            on_disk = sha256_bytes(Path(binary_path).read_bytes())
            if on_disk != report_binary_sha:
                raise SystemExit(f"store-gate: REFUSED - {binary_path} ({on_disk[:12]}...) is not the binary the "
                                 f"{lane} report names ({report_binary_sha[:12]}...); a verdict measured "
                                 "against a different parser than the results would describe nothing. "
                                 "Re-run the corpus lane against the binary on disk.")
        log(f"store-gate: measuring {len(todo)} {lane} row(s) (cache hits: {len(wanted)})")

        def one(item):
            key, fsha, raw = item
            verdict, names, forms = measurer(raw, profile)
            return key, {"lane": key[0], "suite": key[1], "relpath": key[2], "file_sha256": fsha,
                         "binary_sha256": report_binary_sha, "verdict": verdict,
                         "names": ",".join(names), "forms": "+".join(forms)}
        with ThreadPoolExecutor(max_workers=max(1, jobs)) as ex:
            for key, row in ex.map(one, todo):
                cache[key] = row
                wanted[(key[1], key[2])] = row
    # The cache is exactly the candidate population: rows that stopped being candidates go.
    live = {(lane, s, r) for s, r in wanted}
    for key in [k for k in cache if k[0] == lane and k not in live]:
        del cache[key]
    return wanted


def basis_suffix(row: dict) -> str:
    if row["verdict"] == EXPLAINED:
        return (f" - .13w: parses once `{row['names']}` is declared ({row['forms']}); the sound store gate "
                "(SV-PARSE-STRICT boundary) is the only obstacle")
    return f" - .13w: store-gate search {row['verdict']}" + (f" (demanded: `{row['names']}`)" if row["names"] else "")


def report(cache_path: Path) -> str:
    from collections import Counter
    cache = load_cache(cache_path)
    lines = []
    for lane in PROFILES:
        rows = [r for k, r in cache.items() if k[0] == lane]
        c = Counter(r["verdict"] for r in rows)
        lines.append(f"{lane}: candidates={len(rows)} " + " ".join(f"{k}={v}" for k, v in sorted(c.items())))
    return "\n".join(lines)


def _self_test() -> int:
    """Arms that need no parser (a fake measurer drives the memo logic), then one live arm."""
    import tempfile
    fails = []

    def check(name, cond):
        print(f"  {'PASS' if cond else 'FAIL'}  {name}")
        if not cond:
            fails.append(name)

    m = machinery()
    strip = m.blank_comments_and_strings
    # conditions 2..4
    check("unresolved wildcard import is a residual, not an explanation",
          soundness_reason(strip("import ext_pkg::*;\nmodule t; T x; endmodule\n"), ("T",)) == "residual:unresolved_wildcard_import")
    check("a wildcard import of a package the file declares is resolved",
          soundness_reason(strip("package p; endpackage\nimport p::*;\nmodule t; T x; endmodule\n"), ("T",)) is None)
    check("an import mentioned only in a comment is not an import",
          soundness_reason(strip("// import ext_pkg::*;\nmodule t; T x; endmodule\n"), ("T",)) is None)
    check("a package export is a residual",
          soundness_reason(strip("package p; export q::*; endpackage\nmodule t; T x; endmodule\n"), ("T",)) == "residual:package_export")
    check("an LRM-predefined type name is a residual (grammar defect, not a missing declaration)",
          soundness_reason(strip("module t; endmodule\n"), ("CrossQueueType",)) == "residual:predefined_type_name")
    # the memo
    with tempfile.TemporaryDirectory(dir=str(ROOT / "rust/target")) as td:
        d = Path(td)
        f1 = d / "a.sv"; f1.write_text("module a; T x; endmodule\n")
        f2 = d / "b.sv"; f2.write_text("module b; U y; endmodule\n")
        calls = []

        def fake(raw, profile):
            calls.append(raw)
            return (EXPLAINED, ("T",), ("class",)) if "T x" in raw else ("residual:not_parsed_with_declarations", ("U",), ())
        cache = {}
        got = explain_rows("sv_2017", [("s", "a.sv", f1), ("s", "b.sv", f2)], "b" * 64, d / "nobinary", cache, measurer=fake, log=lambda *_: None)
        check("first pass measures every candidate", len(calls) == 2)
        check("an explained row carries its minimal prelude", got[("s", "a.sv")]["verdict"] == EXPLAINED and got[("s", "a.sv")]["names"] == "T" and got[("s", "a.sv")]["forms"] == "class")
        check("a residual row stays a residual with the names it demanded", got[("s", "b.sv")]["verdict"].startswith("residual:") and got[("s", "b.sv")]["names"] == "U")
        calls.clear()
        explain_rows("sv_2017", [("s", "a.sv", f1), ("s", "b.sv", f2)], "b" * 64, d / "nobinary", cache, measurer=fake, log=lambda *_: None)
        check("an unchanged (file, binary) key is NOT re-measured", calls == [])
        calls.clear()
        explain_rows("sv_2017", [("s", "a.sv", f1), ("s", "b.sv", f2)], "c" * 64, d / "nobinary", cache, measurer=fake, log=lambda *_: None)
        check("a new parser binary re-measures every row", len(calls) == 2)
        calls.clear()
        f1.write_text("module a; T x; T z; endmodule\n")
        explain_rows("sv_2017", [("s", "a.sv", f1), ("s", "b.sv", f2)], "c" * 64, d / "nobinary", cache, measurer=fake, log=lambda *_: None)
        check("changed file bytes re-measure that row only", len(calls) == 1 and "T z" in calls[0])
        explain_rows("sv_2017", [("s", "a.sv", f1)], "c" * 64, d / "nobinary", cache, measurer=fake, log=lambda *_: None)
        check("a row that stopped being a candidate leaves the cache", ("sv_2017", "s", "b.sv") not in cache and ("sv_2017", "s", "a.sv") in cache)
        cp = d / "cache.tsv"; save_cache(cp, cache); back = load_cache(cp)
        check("the cache round-trips through its TSV", back == {k: {c: str(v) for c, v in r.items()} for k, r in cache.items()})
        try:
            explain_rows("sv_2017", [("s", "b.sv", f2)], "d" * 64, d / "nobinary", {}, log=lambda *_: None)
            check("a real measurement without the binary REFUSES", False)
        except SystemExit as e:
            check("a real measurement without the binary REFUSES", "absent" in str(e))
        wrong = d / "probe"; wrong.write_bytes(b"not the parser")
        try:
            explain_rows("sv_2017", [("s", "b.sv", f2)], "d" * 64, wrong, {}, log=lambda *_: None)
            check("a binary that is not the report's REFUSES to measure", False)
        except SystemExit as e:
            check("a binary that is not the report's REFUSES to measure", "REFUSED" in str(e))
    # live: the parser itself, if built
    if m.PROBE.is_file():
        v = measure("module t;\n  T x;\nendmodule\n", "sv_2017")
        check("LIVE: `T x;` in a module is explained by declaring `T` as a class", v == (EXPLAINED, ("T",), ("class",)))
        v = measure("module t;\n  int x;\nendmodule\n", "sv_2017")
        check("LIVE: a file that parses bare is `residual:parses_bare`", v[0] == "residual:parses_bare")
    else:
        check("LIVE arms need rust/target/release/parseability_probe (absent)", False)
    print(f"store_gate_explanations self-test: {'PASS' if not fails else 'FAIL'} ({len(fails)} failing)")
    return 1 if fails else 0


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--self-test", action="store_true")
    ap.add_argument("--report", action="store_true", help="summarise the tracked cache")
    ap.add_argument("--cache", type=Path, default=DEFAULT_CACHE)
    a = ap.parse_args()
    if a.self_test:
        raise SystemExit(_self_test())
    if a.report:
        print(report(a.cache))
        return
    ap.print_help()


if __name__ == "__main__":
    main()
