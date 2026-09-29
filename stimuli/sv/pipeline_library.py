#!/usr/bin/env python3
"""SVPP-EXPANSION.6d.1 — the LIBRARY PASS of the preprocess→parse lane.

A design suite's file is one member of a compiled design: its `import X::*` names typedefs, classes and
parameters that live in a PACKAGE declared by another file, and the parser's fact-gated productions
(`known_unscoped_data_type_identifier`, `known_unscoped_base_class_type_identifier`, …) only admit such a
name once the package's facts are in scope. The grammar exports them (`@export_to_library` on
`package_declaration`, `--lib-out DIR` → `DIR/package/<name>.facts.json`) and imports them
(`@import_from_library` on `package_import_item`, `--lib-in DIR`). This pass builds that library for the
design suites BEFORE the per-file lane runs:

  1. inventory — every `*.sv` / `*.svh` under a design suite whose text declares `package NAME;`
  2. edges — the packages each file IMPORTS, read from its PREPROCESSED text (the unit the engine
     parses: `` `include ``d class files carry their own imports), to every file declaring that package
  3. order — a topological order (Kahn; ties by path); a cycle is reported and its members parsed last
  4. parse — each file through the probe with its own derived context (`pipeline_context_args.sh`),
     `--lib-in LIB` (what its predecessors exported) and `--lib-out` into a private directory; a PASSING
     file's exports are PUBLISHED into LIB — at once when the package is declared by one file, and for a
     package several files declare (`prim_pkg` ×4: one per vendor) only once EVERY declaring file has
     passed and their fact sets are byte-identical after normalisation (else `variants_disagree` /
     `variants_incomplete`: reported, nothing published — the build declaration picks, we do not guess)
  5. a REJECTING or REFUSED file exports nothing (the engine writes no facts for a rejected parse —
     SVPP-EXPANSION.5 pre-scoping (v)); the packages that import it stay dark on its names, measured

The per-package outcome goes to `stimuli/sv/characterization/pipeline_library.tsv` (a dump, like the
lane's; `.gitignore`d; the report renders its summary):
    suite  status  relpath  packages  secs  rc  furthest  origin  exported  note
`status` ∈ pass | fail | pp_fail | timeout | crash; `exported` = the packages published from this file.

SVPP-EXPANSION.6d.2 — the UNIT MEMBERSHIP: the files a package unit's preprocessed text came from (the
preprocessor's source map, `--source-map-json`), one row per (package file, member file) to
`stimuli/sv/characterization/pipeline_library_members.tsv` (`suite  package_relpath  member_relpath`; a
dump too). A member has no `import` of its own; its verdict under the pipeline is its unit's at that
file's origin — the re-adjudication instrument reads this table with the dump above.
"""
import argparse
import collections
import json
import os
import pathlib
import re
import shutil
import subprocess
import sys
import time

ROOT = pathlib.Path(__file__).resolve().parents[2]
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from pipeline_context import DESIGN_SUITES, SUITES  # noqa: E402

PKG_DECL = re.compile(r"^\s*package\s+(?:automatic\s+|static\s+)?([A-Za-z_]\w*)\s*;", re.M)
IMPORT = re.compile(r"\bimport\s+([A-Za-z_]\w*)\s*::")
CTX_SCRIPT = "stimuli/sv/pipeline_context_args.sh"


def context_flags(suite: str, rel: str) -> list[str]:
    """The file's derived context as probe flags — the lane's own source of truth, sourced, minus any --lib-in."""
    env = dict(os.environ)
    env["PGEN_PIPELINE_LIB_DIR"] = ""
    r = subprocess.run(["bash", "-c", f'source {CTX_SCRIPT}; context_args "$@"', "_", suite, rel],
                       capture_output=True, text=True, cwd=ROOT, env=env, check=True)
    flags: list[str] = []
    for line in r.stdout.splitlines():
        if "\t" in line:
            flag, value = line.split("\t", 1)
            flags += [flag, value]
    return flags


def expansion(ast_pipeline: pathlib.Path, rel: str, flags: list[str], timeout_s: int,
              source_map: pathlib.Path | None = None) -> tuple[str | None, list[str]]:
    """(the preprocessed text — the unit the parser sees — or None when the preprocessor refuses,
    the distinct repo-relative files the text came from, in first-appearance order)."""
    cmd = [str(ast_pipeline), rel, "--preprocess", "systemverilog", *flags]
    if source_map is not None:
        cmd += ["--source-map-json", str(source_map)]
    try:
        r = subprocess.run(cmd, capture_output=True, text=True, cwd=ROOT, timeout=timeout_s, errors="replace")
    except subprocess.TimeoutExpired:
        return None, []
    if r.returncode != 0:
        return None, []
    members: list[str] = []
    if source_map is not None and source_map.exists():
        try:
            for seg in json.loads(source_map.read_text()):
                f = seg.get("source", {}).get("file", "")
                try:
                    m = pathlib.Path(f).resolve().relative_to(ROOT).as_posix()
                except ValueError:
                    m = f
                if m and m not in members:
                    members.append(m)
        except (json.JSONDecodeError, OSError):
            pass
    return r.stdout, members


def inventory(suites: list[str]) -> dict[str, tuple[str, list[str]]]:
    """relpath → (suite, packages declared in the RAW text) for every package file of the suites."""
    files: dict[str, tuple[str, list[str]]] = {}
    for suite in suites:
        base = ROOT / SUITES[suite]
        if not base.is_dir():
            continue
        for f in sorted(list(base.rglob("*.sv")) + list(base.rglob("*.svh"))):
            try:
                text = f.read_text(errors="replace")
            except OSError:
                continue
            names = PKG_DECL.findall(text)
            if names:
                files[f.relative_to(ROOT).as_posix()] = (suite, list(dict.fromkeys(names)))
    return files


def topological(files: dict[str, set[str]]) -> tuple[list[str], list[str]]:
    """(order, cyclic) — Kahn's algorithm over file → files it depends on; ties by path."""
    indeg = {f: len(deps) for f, deps in files.items()}
    users: dict[str, list[str]] = collections.defaultdict(list)
    for f, deps in files.items():
        for d in deps:
            users[d].append(f)
    ready = sorted(f for f, n in indeg.items() if n == 0)
    order: list[str] = []
    while ready:
        f = ready.pop(0)
        order.append(f)
        for g in users.get(f, []):
            indeg[g] -= 1
            if indeg[g] == 0:
                ready.append(g)
                ready.sort()
    cyclic = sorted(f for f in files if f not in set(order))
    return order, cyclic


def normalised_facts(path: pathlib.Path) -> str:
    d = json.loads(path.read_text())
    facts = d.get("facts", [])
    return json.dumps(sorted(json.dumps(x, sort_keys=True) for x in facts))


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--probe", default="rust/target/release/parseability_probe")
    ap.add_argument("--ast-pipeline", default=None, help="the expansion tool (default: next to the probe)")
    ap.add_argument("--lib-dir", default="stimuli/sv/characterization/pipeline_library")
    ap.add_argument("--out", default="stimuli/sv/characterization/pipeline_library.tsv")
    ap.add_argument("--out-members", default="stimuli/sv/characterization/pipeline_library_members.tsv")
    ap.add_argument("--timeout-s", type=int, default=60)
    ap.add_argument("--suites", nargs="*", default=sorted(DESIGN_SUITES))
    ap.add_argument("--max-files", type=int, default=0, help="smoke tests: parse only the first N of the order")
    args = ap.parse_args()
    probe = pathlib.Path(args.probe)
    ast_pipeline = pathlib.Path(args.ast_pipeline) if args.ast_pipeline else probe.parent / "ast_pipeline"
    if not probe.is_file() or not ast_pipeline.is_file():
        print(f"pipeline-library: missing {probe} or {ast_pipeline}", file=sys.stderr)
        return 2
    lib = pathlib.Path(args.lib_dir)
    (lib / "package").mkdir(parents=True, exist_ok=True)
    work = lib / ".work"
    shutil.rmtree(work, ignore_errors=True)
    work.mkdir()

    files = inventory(args.suites)
    declared: dict[str, list[str]] = collections.defaultdict(list)
    for rel, (_, names) in files.items():
        for n in names:
            declared[n].append(rel)
    # 2. edges from the preprocessed text
    imports: dict[str, set[str]] = {}
    pp_refused: set[str] = set()
    flags_of: dict[str, list[str]] = {}
    members_of: dict[str, list[str]] = {}
    for rel, (suite, names) in files.items():
        flags_of[rel] = context_flags(suite, rel)
        text, members = expansion(ast_pipeline, rel, flags_of[rel], args.timeout_s, work / "srcmap.json")
        if text is None:
            pp_refused.add(rel)
            text = (ROOT / rel).read_text(errors="replace")
        imports[rel] = set(IMPORT.findall(text)) - set(names)
        members_of[rel] = [m for m in members if m != rel]
    deps = {rel: {d for p in imports[rel] for d in declared.get(p, []) if d != rel} for rel in files}
    order, cyclic = topological(deps)
    order += cyclic
    unresolved = collections.Counter(p for rel in files for p in imports[rel] if p not in declared)

    # 4. parse in order, publish
    rows: list[list[str]] = []
    variant_exports: dict[str, dict[str, pathlib.Path | None]] = collections.defaultdict(dict)  # pkg → {rel: facts or None}
    published: set[str] = set()
    notes_of: dict[str, list[str]] = collections.defaultdict(list)
    for k, rel in enumerate(order):
        if args.max_files and k >= args.max_files:
            break
        suite, names = files[rel]
        out_dir = work / f"{k:04d}"
        out_dir.mkdir()
        cmd = [str(probe), "--parse", "systemverilog", rel, "--profile", "sv_2017", "--preprocess", *flags_of[rel],
               "--lib-in", str(lib), "--lib-out", str(out_dir)]
        t0 = time.monotonic()
        try:
            r = subprocess.run(cmd, capture_output=True, text=True, cwd=ROOT, timeout=args.timeout_s, errors="replace")
            rc = r.returncode
            err = r.stderr + r.stdout
        except subprocess.TimeoutExpired:
            rc, err = 124, ""
        secs = time.monotonic() - t0
        status = {0: "pass", 1: "fail", 3: "pp_fail", 124: "timeout"}.get(rc, "crash" if rc >= 128 else "fail")
        m = re.search(r"furthest_position=(\d+)", err)
        furthest = m.group(1) if m else ""
        m = re.search(r"\[origin=([^\]]*)\]", err)
        origin = m.group(1) if m else ""
        if status == "pp_fail":
            m = re.search(r"preprocess_failed[^:]*: ([^\n]*)", err)
            origin = (m.group(1) if m else "")[:200]
        exported: list[str] = []
        notes = notes_of[rel]
        if rel in cyclic:
            notes.append("cycle")
        if rel in pp_refused:
            notes.append("expansion_refused_edges_from_raw_text")
        for facts in sorted((out_dir / "package").glob("*.facts.json")) if status == "pass" else []:
            pkg = facts.name[: -len(".facts.json")]
            if len(declared.get(pkg, [])) <= 1:
                shutil.copyfile(facts, lib / "package" / facts.name)
                published.add(pkg)
                exported.append(pkg)
            else:
                variant_exports[pkg][rel] = facts
        if status != "pass":
            for pkg in names:
                if len(declared.get(pkg, [])) > 1:
                    variant_exports[pkg][rel] = None
        # a multiply-declared package resolves once every declaring file has been parsed
        for pkg in names:
            decl = declared.get(pkg, [])
            if len(decl) > 1 and all(d in variant_exports[pkg] for d in decl) and pkg not in published:
                got = variant_exports[pkg]
                if any(v is None for v in got.values()):
                    for d in decl:
                        notes_of[d].append(f"variants_incomplete:{pkg}")
                elif len({normalised_facts(v) for v in got.values()}) == 1:
                    src = got[sorted(got)[0]]
                    shutil.copyfile(src, lib / "package" / f"{pkg}.facts.json")
                    published.add(pkg)
                    for d in decl:
                        notes_of[d].append(f"variants_agree:{pkg}")
                else:
                    for d in decl:
                        notes_of[d].append(f"variants_disagree:{pkg}")
        rows.append([suite, status, rel, " ".join(names), f"{secs:.2f}", str(rc), furthest, origin, " ".join(exported), ""])
    # notes are complete only after the loop (a variant resolves on its last declaring file)
    for row in rows:
        row[9] = ";".join(notes_of[row[2]])
    shutil.rmtree(work, ignore_errors=True)
    out_members = pathlib.Path(args.out_members)
    tmp_m = out_members.with_suffix(out_members.suffix + f".tmp.{os.getpid()}")
    tmp_m.write_text("".join(f"{files[rel][0]}\t{rel}\t{m}\n" for rel in order for m in members_of.get(rel, [])))
    tmp_m.replace(out_members)
    out = pathlib.Path(args.out)
    tmp = out.with_suffix(out.suffix + f".tmp.{os.getpid()}")
    tmp.write_text("".join("\t".join(r) + "\n" for r in rows))
    tmp.replace(out)
    by = collections.Counter((r[0], r[1]) for r in rows)
    suites = sorted({r[0] for r in rows})
    print("PIPELINE-LIBRARY: " + " ".join(
        f"{s}={by[(s, 'pass')]}/{sum(n for (ss, _), n in by.items() if ss == s)}" for s in suites)
        + f" published_packages={len(published)} unresolved_imports={len(unresolved)} cyclic_files={len(cyclic)} "
          f"member_rows={sum(len(v) for v in members_of.values())} → {out}, {out_members}")
    if unresolved:
        print("  unresolved (imported, declared by no file of these suites): " + " ".join(f"{p}×{n}" for p, n in unresolved.most_common(12)))
    return 0


if __name__ == "__main__":
    sys.exit(main())
