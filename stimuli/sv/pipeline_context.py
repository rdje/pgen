#!/usr/bin/env python3
"""Derive a corpus suite's COMPILATION CONTEXT for the preprocess→parse lane (SVPP-EXPANSION.4).

WHY THIS EXISTS
---------------
The raw corpus lane (`stimuli/run_external_corpus.sh`) parses every file ALONE and cannot judge a
file that `include`s a header it cannot find, so 4 157 UVM + OpenTitan rows carry no verdict at all
(`deferred:chained_only`). The pipeline lane runs the preprocessor first, and a preprocessor needs
an include search path. Nothing in this repository reads a build system's own context (`.core`,
`.f`, `+incdir+` — measured, SVPP-EXPANSION.1), so the context is DERIVED from the corpus itself:

    for every `` `include "NAME" `` in every source file of the suite,
      if `dirname(includer)/NAME` exists      -> LOCAL: the preprocessor already searches the
                                                 including file's own directory first; no dir needed
      else the directories D with `D/NAME`   -> exactly one: D is a SHARED header directory -> emitted
                                              -> several:    AMBIGUOUS-NONLOCAL, reported, not guessed
                                              -> none:       ABSENT, reported

The emitted directories are ordered by how many includers need them (ties by path), which is the
order the preprocessor searches them in. ⛔ An ambiguous name is reported with every candidate and
is NOT resolved by picking one: the per-file context a build system would supply is `.4`(b)'s
question, and a guessed directory would turn an honest `preprocess_failed` into a silent wrong
expansion.

OUTPUT (deterministic; sorted; no timestamps)
    stimuli/sv/pipeline_context.tsv        suite<TAB>kind<TAB>value<TAB>needed_by<TAB>resolves
                                           kind = include_dir | define  (defines are hand-added rows)
    stimuli/sv/pipeline_context_files.tsv  suite<TAB>relpath<TAB>include_dir<TAB>name — the PER-FILE
                                           rows (SVPP-EXPANSION.4b): an ambiguous name resolved for ONE
                                           including file by the NEAREST candidate (the candidate
                                           directory sharing the longest path prefix with the includer —
                                           what a build system's per-top file list does for the
                                           `top_earlgrey` / `top_darjeeling` variants of one `seq_lib`).
                                           A tie stays ambiguous and is reported, never guessed.
    stdout                                 one PIPELINE-CONTEXT: line per suite + the ambiguous /
                                           absent names

USAGE
    python3 stimuli/sv/pipeline_context.py --write            # re-derive every registered suite
    python3 stimuli/sv/pipeline_context.py --check            # re-derive and diff against the tracked TSV (exit 1 on drift)
    python3 stimuli/sv/pipeline_context.py --self-test        # the synthetic tree; exit 7 on a miss
    python3 stimuli/sv/pipeline_context.py --suite opentitan  # report one suite, write nothing
    python3 stimuli/sv/pipeline_context.py --list-suites      # every registered suite, one per line (the lane's roster)
"""
from __future__ import annotations

import argparse
import collections
import os
import pathlib
import re
import sys
import tempfile

ROOT = pathlib.Path(__file__).resolve().parents[2]
if not (ROOT / "grammars" / "systemverilog.ebnf").exists():
    sys.exit(f"pipeline_context: derived repo root {ROOT} holds no grammars/systemverilog.ebnf — refusing")

TSV = ROOT / "stimuli" / "sv" / "pipeline_context.tsv"
SOURCE_SUFFIXES = (".sv", ".svh", ".v", ".vh")
INCLUDE_RE = re.compile(r'^[ \t]*`include[ \t]+"([^"]+)"', re.M)

# suite name -> root directory, repo-root-relative. The names are the raw lane's own (the `sub`
# column of results.tsv: the directory under stimuli/sv/subs/, and `uvm-core` for the UVM tree), so
# the two lanes join on them. DERIVED from the tree (SVPP-EXPANSION.4b): every directory under
# stimuli/sv/subs/ is a suite, and a suite added by vendoring joins the lane without an edit here.
def registered_suites() -> dict[str, str]:
    suites = {"uvm-core": "stimuli/sv/uvm"}
    subs = ROOT / "stimuli" / "sv" / "subs"
    if subs.is_dir():
        for d in sorted(subs.iterdir()):
            if d.is_dir():
                suites[d.name] = f"stimuli/sv/subs/{d.name}"
    return suites


SUITES = registered_suites()
FILES_TSV = ROOT / "stimuli" / "sv" / "pipeline_context_files.tsv"
# SVPP-EXPANSION.4c.1 — per-file DEFINES a suite's own driver passes for the file: verilator's `t_x.py`
# (`+define+NAME[=VALUE]` / `-DNAME[=VALUE]` inside its flag lists) and sv-tests' `:defines:` header.
# Derived, never guessed: only a LITERAL value is emitted (a value the driver computes at run time —
# `"+define+SIM_CYCLES=" + str(test.cycles)`, an f-string — is REPORTED as computed); the SAME-STEM driver
# owns the file, and where only variant drivers (`top_filename = "t/t_x.v"`) target it, a single variant
# is used and several are REPORTED as a tie. ⛔ This is the test's OWN configuration, not a tool-identity
# predefine (`.6b.3` measured and declined those): the driver ran the file with exactly these.
DEFINES_TSV = ROOT / "stimuli" / "sv" / "pipeline_context_defines.tsv"
# Rows the derivation cannot produce: predefined macros a suite is compiled with. Hand-curated,
# each with its source; empty until a suite needs one.
HAND_DEFINES: dict[str, list[tuple[str, str]]] = {}
# Library trees every suite may include from, beyond its own tree. Measured (SVPP-EXPANSION.4):
# 231 OpenTitan files `include "uvm_macros.svh"`, which lives in the UVM suite — a cross-suite
# dependency the derivation resolves like any other shared header, and reports as such.
SHARED_LIBRARIES = ["stimuli/sv/uvm/uvm-core-2020.3.1/src"]


def source_files(base: pathlib.Path) -> list[pathlib.Path]:
    return sorted(p for p in base.rglob("*") if p.is_file() and p.suffix in SOURCE_SUFFIXES)


def derive(base: pathlib.Path):
    """Return (dirs, ambiguous, absent, stats) for the suite rooted at `base`.

    dirs: {dir (relative to ROOT): (needed_by_file_count, resolved_name_set)}
    ambiguous: {name: sorted candidate dirs} — non-local names resolvable in more than one dir
    absent: {name: includer_count}
    """
    files = source_files(base)
    # every file path relative to `base`, so a NAME with a subpath ("macros/x.svh") matches
    # `D/NAME` for every directory D whose subtree holds it; the shared library trees join the
    # candidate set under their own repo-relative prefix
    rel_paths = {p.relative_to(base).as_posix() for p in files}
    lib_paths: dict[str, pathlib.Path] = {}
    for lib in SHARED_LIBRARIES:
        lib_base = ROOT / lib
        if lib_base.exists() and not lib_base.is_relative_to(base):
            for lp in source_files(lib_base):
                lib_paths[lp.relative_to(ROOT).as_posix()] = lp
    dirs: dict[str, set[str]] = collections.defaultdict(set)
    needed_by: collections.Counter = collections.Counter()
    ambiguous: dict[str, list[str]] = {}
    absent: collections.Counter = collections.Counter()
    per_file: list[tuple[str, str, str]] = []   # (includer relpath, include_dir, name) — nearest-candidate rows
    local = 0
    for p in files:
        try:
            text = p.read_text(errors="replace")
        except OSError:
            continue
        for name in INCLUDE_RE.findall(text):
            if (p.parent / name).exists():
                local += 1
                continue
            name_posix = pathlib.PurePosixPath(name).as_posix()
            candidates = sorted(
                {
                    (base.relative_to(ROOT) / rp[: -len(name_posix)]).as_posix().rstrip("/")
                    for rp in rel_paths
                    if rp == name_posix or rp.endswith("/" + name_posix)
                }
                | {
                    lp[: -len(name_posix)].rstrip("/")
                    for lp in lib_paths
                    if lp == name_posix or lp.endswith("/" + name_posix)
                }
            )
            if not candidates:
                absent[name] += 1
            elif len(candidates) == 1:
                dirs[candidates[0]].add(name)
                needed_by[candidates[0]] += 1
            else:
                # SVPP-EXPANSION.4b: the NEAREST candidate for THIS includer — the one sharing the
                # longest path prefix with it — is what a per-top build would hand it; a tie is
                # still ambiguous for this file and stays reported.
                includer = p.relative_to(ROOT).as_posix()
                nearest = nearest_candidate(includer, candidates)
                if nearest is not None:
                    per_file.append((includer, nearest, name))
                    dirs.setdefault("__per_file__", set())
                else:
                    ambiguous[name] = candidates
                if name not in ambiguous:
                    ambiguous.setdefault("__resolved_per_file__", []).append(name)
    dirs.pop("__per_file__", None)
    resolved_per_file = sorted(set(ambiguous.pop("__resolved_per_file__", [])))
    stats = {"files": len(files), "local": local, "per_file_rows": len(per_file),
             "resolved_per_file_names": resolved_per_file}
    return {d: (needed_by[d], dirs[d]) for d in dirs}, ambiguous, absent, stats, per_file


def common_prefix_len(a: str, b: str) -> int:
    """Path components the two repo-relative paths share from the root."""
    n = 0
    for x, y in zip(a.split("/"), b.split("/")):
        if x != y:
            break
        n += 1
    return n


def nearest_candidate(includer: str, candidates: list[str]) -> str | None:
    """The single candidate directory sharing the longest path prefix with the includer, or None on a tie."""
    scored = sorted(((common_prefix_len(includer, c), c) for c in candidates), key=lambda t: (-t[0], t[1]))
    if len(scored) >= 2 and scored[0][0] == scored[1][0]:
        return None
    return scored[0][1]


DEFINE_TOKEN = re.compile(r"""(?:\+define\+|(?<![A-Za-z0-9_])-D)([A-Za-z_][A-Za-z0-9_]*)(=?)([^"'\s\]\),]*)""")
COMPUTED_TAIL = re.compile(r"""^["']\s*\+|\{[^}]*\}""")
# spans whose -D flags reach the C++ compiler or iverilog, never the SV source under verilator:
# make_flags=[…] (CPPFLAGS_ADD=…), CPPFLAGS_*=…, -CFLAGS/--CFLAGS/-LDFLAGS + one quoted argument, iv_flags*=[…]
CXX_SPANS = re.compile(
    r"""make_flags\s*=\s*\[.*?\]|iv_flags2?\s*=\s*\[.*?\]|CPPFLAGS\w*=[^\]\n]*|--?(?:CFLAGS|LDFLAGS)['"]?\s*,?\s*['"][^,\]\n]*""",
    re.S,
)


def driver_defines(py_text: str) -> tuple[list[str], list[str]]:
    """(literal defines as `NAME` or `NAME=VALUE`, computed define names) found in a verilator driver."""
    literal, computed = [], []
    py_text = CXX_SPANS.sub(" ", py_text)
    for m in DEFINE_TOKEN.finditer(py_text):
        name, eq, value = m.group(1), m.group(2), m.group(3)
        tail = py_text[m.end():m.end() + 12]
        if (eq and value == "" and COMPUTED_TAIL.match(tail)) or "{" in value:
            computed.append(name)          # the value is built at run time — not derivable here
            continue
        literal.append(f"{name}={value}" if eq and value else name)
    return sorted(set(literal)), sorted(set(computed))


def derive_defines(suite: str, base: pathlib.Path):
    """Per-file define rows for the suites whose drivers carry them; plus what was REPORTED, not emitted.
    Returns (rows [(suite, relpath, define, source)], computed {relpath: [names]}, ties {relpath: [drivers]})."""
    rows, computed, ties = [], {}, {}
    if suite == "verilator":
        tdir = base / "test_regress" / "t"
        if not tdir.is_dir():
            return rows, computed, ties
        by_target: dict[str, list[pathlib.Path]] = {}
        for py in sorted(tdir.glob("t_*.py")):  # a driver; `*_common.py` helpers are not drivers
            text = py.read_text(errors="replace")
            m = re.search(r'top_filename\s*=\s*"t/([^"]+)"', text)
            target = m.group(1) if m else py.stem + ".v"
            if not (tdir / target).is_file():
                continue
            by_target.setdefault(target, []).append(py)
        for target, drivers in sorted(by_target.items()):
            same = [d for d in drivers if d.stem + ".v" == target]
            chosen = same[0] if same else (drivers[0] if len(drivers) == 1 else None)
            rel = (tdir / target).relative_to(ROOT).as_posix()
            if chosen is None:
                with_defs = [d.name for d in drivers if any(driver_defines(d.read_text(errors="replace")))]
                if with_defs:
                    ties[rel] = with_defs
                continue
            lit, comp = driver_defines(chosen.read_text(errors="replace"))
            for d in lit:
                rows.append((suite, rel, d, chosen.relative_to(ROOT).as_posix()))
            if comp:
                computed[rel] = comp
    elif suite == "sv-tests":
        for f in source_files(base):
            try:
                head = f.read_text(errors="replace")[:4096]
            except OSError:
                continue
            for m in re.finditer(r"^:defines:\s*(.+)$", head, re.M):
                for tok in m.group(1).split():
                    rows.append((suite, f.relative_to(ROOT).as_posix(), tok.strip(), f.relative_to(ROOT).as_posix() + " (:defines: header)"))
    return sorted(set(rows)), computed, ties


def rows_for(suite: str, base: pathlib.Path):
    dirs, ambiguous, absent, stats, per_file = derive(base)
    ordered = sorted(dirs.items(), key=lambda kv: (-kv[1][0], kv[0]))
    rows = [(suite, "include_dir", d, str(n), ",".join(sorted(names))) for d, (n, names) in ordered]
    for name, source in HAND_DEFINES.get(suite, []):
        rows.append((suite, "define", name, "hand", source))
    file_rows = sorted({(suite, includer, d, name) for includer, d, name in per_file})
    define_rows, computed, ties = derive_defines(suite, base)
    stats["define_rows"] = len(define_rows)
    stats["computed_defines"] = computed
    stats["define_ties"] = ties
    return rows, ambiguous, absent, stats, file_rows, define_rows


def report(suite: str, rows, ambiguous, absent, stats, out=sys.stdout):
    n_dirs = sum(1 for r in rows if r[1] == "include_dir")
    print(
        f"PIPELINE-CONTEXT: suite={suite} files={stats['files']} local_includes={stats['local']} "
        f"include_dirs={n_dirs} per_file_rows={stats['per_file_rows']} "
        f"names_resolved_per_file={len(stats['resolved_per_file_names'])} "
        f"ambiguous_nonlocal_names={len(ambiguous)} absent_names={len(absent)} "
        f"define_rows={stats.get('define_rows', 0)} computed_defines={len(stats.get('computed_defines', {}))} "
        f"define_ties={len(stats.get('define_ties', {}))}",
        file=out,
    )
    for name in sorted(ambiguous):
        print(f"  ambiguous {name}: {' '.join(ambiguous[name])}", file=out)
    for name in sorted(absent):
        print(f"  absent    {name} (x{absent[name]})", file=out)
    for rel in sorted(stats.get("computed_defines", {})):
        print(f"  computed  {rel}: {' '.join(stats['computed_defines'][rel])} (the driver builds the value at run time; not derived)", file=out)
    for rel in sorted(stats.get("define_ties", {})):
        print(f"  tie       {rel}: {' '.join(stats['define_ties'][rel])} (several variant drivers carry defines; none applied)", file=out)


def render(all_rows) -> str:
    lines = ["suite\tkind\tvalue\tneeded_by\tresolves"]
    for r in all_rows:
        lines.append("\t".join(r))
    return "\n".join(lines) + "\n"


def render_files(file_rows) -> str:
    lines = ["suite\trelpath\tinclude_dir\tname"]
    for r in file_rows:
        lines.append("\t".join(r))
    return "\n".join(lines) + "\n"


def render_defines(define_rows) -> str:
    lines = ["suite\trelpath\tdefine\tsource"]
    for r in define_rows:
        lines.append("\t".join(r))
    return "\n".join(lines) + "\n"


def derive_all(suites=None):
    all_rows = []
    all_file_rows = []
    all_define_rows = []
    for suite, rel in SUITES.items():
        if suites and suite not in suites:
            continue
        base = ROOT / rel
        if not base.exists():
            print(f"PIPELINE-CONTEXT: suite={suite} NOT EVALUATED (missing {rel})", file=sys.stderr)
            continue
        rows, ambiguous, absent, stats, file_rows, define_rows = rows_for(suite, base)
        report(suite, rows, ambiguous, absent, stats)
        all_rows.extend(rows)
        all_file_rows.extend(file_rows)
        all_define_rows.extend(define_rows)
    return all_rows, all_file_rows, all_define_rows


def self_test() -> int:
    """A synthetic suite: one local include, one shared header, one ambiguous name, one absent."""
    failures = 0
    with tempfile.TemporaryDirectory(dir=ROOT / "rust" / "target") as tmp:
        base = pathlib.Path(tmp)
        (base / "a").mkdir(); (base / "b").mkdir(); (base / "shared").mkdir(); (base / "x").mkdir(); (base / "y").mkdir()
        (base / "a" / "local.svh").write_text("// local\n")
        (base / "shared" / "hdr.svh").write_text("`define H 1\n")
        (base / "x" / "dup.svh").write_text("// x\n"); (base / "y" / "dup.svh").write_text("// y\n")
        (base / "a" / "top.sv").write_text('`include "local.svh"\n`include "hdr.svh"\n`include "dup.svh"\n`include "gone.svh"\n')
        (base / "b" / "other.sv").write_text('`include "hdr.svh"\n')
        # a per-file resolvable name: two candidates, one of them under the includer's own top
        (base / "top_a" / "env" / "seq_lib").mkdir(parents=True); (base / "top_b" / "env" / "seq_lib").mkdir(parents=True)
        (base / "top_a" / "env" / "seq_lib" / "list.sv").write_text("// a\n"); (base / "top_b" / "env" / "seq_lib" / "list.sv").write_text("// b\n")
        (base / "top_a" / "env" / "pkg.sv").write_text('`include "list.sv"\n')
        dirs, ambiguous, absent, stats, per_file = derive(base)
        shared = (base.relative_to(ROOT) / "shared").as_posix()
        top_a_seq = (base.relative_to(ROOT) / "top_a" / "env" / "seq_lib").as_posix()
        checks = [
            ("local include needs no dir", stats["local"] == 1),
            ("the shared header dir is emitted", shared in dirs),
            ("it is needed by both includers", dirs.get(shared, (0, set()))[0] == 2),
            ("a tied ambiguous name is reported with two candidates", len(ambiguous.get("dup.svh", [])) == 2),
            ("the tied ambiguous name emits NO dir", not any(d.endswith("/x") or d.endswith("/y") for d in dirs)),
            ("the absent name is reported", absent.get("gone.svh") == 1),
            ("a name with a NEAREST candidate is resolved per file, to the includer's own top",
             any(d == top_a_seq and n == "list.sv" for _, d, n in per_file)),
            ("and it is not reported as ambiguous", "list.sv" not in ambiguous),
        ]
        for label, ok in checks:
            print(f"  {'PASS' if ok else 'FAIL'} {label}")
            failures += 0 if ok else 1
        # SVPP-EXPANSION.4c.1 — a synthetic verilator tree: a same-stem driver with a literal and a
        # computed define, a single variant driver, and two variant drivers (a tie)
        t = base / "test_regress" / "t"; t.mkdir(parents=True)
        for stem in ("t_a", "t_b", "t_c"):
            (t / f"{stem}.v").write_text("module m; endmodule\n")
        (t / "t_a.py").write_text('test.compile(v_flags2=["+define+LIT=1 -DFLAG", "+define+CYC=" + str(test.cycles)],\n'
                                  '    verilator_flags2=["-CFLAGS", "-DCXX2"], make_flags=["CPPFLAGS_ADD=-DCXX"])\n')
        (t / "t_b_only.py").write_text('test.top_filename = "t/t_b.v"\ntest.compile(v_flags2=["+define+ONE"])\n')
        (t / "b_common.py").write_text('test.top_filename = "t/t_b.v"\ntest.compile(v_flags2=["+define+HELPER"])\n')
        (t / "t_c_x.py").write_text('test.top_filename = "t/t_c.v"\ntest.compile(v_flags2=["+define+X"])\n')
        (t / "t_c_y.py").write_text('test.top_filename = "t/t_c.v"\ntest.compile(v_flags2=["+define+Y"])\n')
        rows, computed, ties = derive_defines("verilator", base)
        defs = {(r[1].rsplit("/", 1)[1], r[2]) for r in rows}
        checks2 = [
            ("a same-stem driver's literal defines are emitted", ("t_a.v", "LIT=1") in defs and ("t_a.v", "FLAG") in defs),
            ("its computed define is reported, not emitted", not any(d == "CYC" or d.startswith("CYC=") for _, d in defs) and any(k.endswith("t_a.v") for k in computed)),
            ("a single variant driver's define is emitted", ("t_b.v", "ONE") in defs),
            ("a C++ define (-CFLAGS, CPPFLAGS_ADD) is neither emitted nor reported", not any(d.startswith("CXX") for _, d in defs) and not any("CXX" in n for ns in computed.values() for n in ns)),
            ("a *_common.py helper is not a driver", ("t_b.v", "HELPER") not in defs and not any(k.endswith("t_b.v") for k in ties)),
            ("two variant drivers are a tie: nothing emitted, the tie reported", not any(f == "t_c.v" for f, _ in defs) and any(k.endswith("t_c.v") for k in ties)),
        ]
        for label, ok in checks2:
            print(f"  {'PASS' if ok else 'FAIL'} {label}")
            failures += 0 if ok else 1
        checks += checks2
    print(f"PIPELINE-CONTEXT-SELF-TEST: checks={len(checks)} failed={failures}")
    return 7 if failures else 0


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--write", action="store_true")
    ap.add_argument("--check", action="store_true")
    ap.add_argument("--self-test", action="store_true")
    ap.add_argument("--suite", action="append")
    ap.add_argument("--list-suites", action="store_true")
    args = ap.parse_args()
    if args.list_suites:
        for suite in SUITES:
            print(suite)
        return 0
    if args.self_test:
        return self_test()
    all_rows, all_file_rows, all_define_rows = derive_all(args.suite)
    if args.write:
        TSV.write_text(render(all_rows))
        FILES_TSV.write_text(render_files(all_file_rows))
        DEFINES_TSV.write_text(render_defines(all_define_rows))
        print(f"PIPELINE-CONTEXT: wrote {TSV.relative_to(ROOT)} rows={len(all_rows)}, {FILES_TSV.relative_to(ROOT)} rows={len(all_file_rows)} and {DEFINES_TSV.relative_to(ROOT)} rows={len(all_define_rows)}")
    if args.check:
        current = TSV.read_text() if TSV.exists() else ""
        current_files = FILES_TSV.read_text() if FILES_TSV.exists() else ""
        current_defines = DEFINES_TSV.read_text() if DEFINES_TSV.exists() else ""
        if current != render(all_rows) or current_files != render_files(all_file_rows) or current_defines != render_defines(all_define_rows):
            print(f"PIPELINE-CONTEXT: DRIFT — the tracked context differs from a fresh derivation; run --write", file=sys.stderr)
            return 1
        print("PIPELINE-CONTEXT: OK — the tracked context equals a fresh derivation")
    return 0


if __name__ == "__main__":
    sys.exit(main())
