#!/usr/bin/env python3
"""Compare the preprocess→parse lane with the raw lane and publish the report (SVPP-EXPANSION.4).

Reads three dumps and one tracked table, joins them per corpus row, and writes
`stimuli/sv/characterization/pipeline_characterization.md` — the TRACKED claim of what the
pipeline recognises that the raw lane could not (and the reverse):

    stimuli/sv/characterization/pipeline_results.tsv   the pipeline lane (run_pipeline_lane.sh)
    stimuli/sv/characterization/results.tsv            the raw lane (run_external_corpus.sh sv)
    stimuli/sv/characterization/adjudication_manifest.tsv   the answer key's verdict per row
    stimuli/sv/pipeline_context.tsv                    the context the lane ran under
    stimuli/sv/pipeline_context_defines.tsv            the per-file defines the files' own drivers pass (SVPP-EXPANSION.4c.1)
    stimuli/sv/pipeline_context_units.tsv              the per-file unit predecessors the files' own harness compiles first (SVPP-EXPANSION.4c.2)

⛔ Every number here is a JOIN, never a copy: a row is counted only where the pipeline dump holds
it, the raw status comes from the raw dump for the SAME path, and the adjudication class from the
manifest for the same (suite, relpath). Rows the pipeline lane did not run are not in the report,
and the report says which suites it covers.

⛔ The identity block re-hashes every input the numbers depend on — the probe (and whether it is a
debug or a release build), the grammar (bytes and the frontend's semantic envelope), the generated
parser, the context, both dumps — so a reader re-hashes before quoting, exactly as the raw lane's
report demands.

USAGE
    python3 stimuli/sv/pipeline_vs_raw.py [--pipeline TSV] [--raw TSV] [--out MD] [--probe BIN] [--top N]
    python3 stimuli/sv/pipeline_vs_raw.py --check      # SVPP-EXPANSION.4b: re-render to a scratch path and
                                                       # DIFF against the tracked report (exit 1 on drift);
                                                       # a dump absent here is NOT EVALUATED, loudly, exit 0
"""
from __future__ import annotations

import argparse
import collections
import hashlib
import pathlib
import re
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
if not (ROOT / "grammars" / "systemverilog.ebnf").exists():
    sys.exit(f"pipeline_vs_raw: derived repo root {ROOT} holds no grammars/systemverilog.ebnf — refusing")

CHAR = ROOT / "stimuli" / "sv" / "characterization"
STATUSES = ("pass", "fail", "pp_fail", "timeout", "crash")


def sha256(path: pathlib.Path) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def semantic_digest(grammar: pathlib.Path) -> str:
    try:
        out = subprocess.run(
            ["bash", str(ROOT / "scripts" / "check_baseline_identity.sh"), "--digest", "ebnf_raw_ast", str(grammar)],
            capture_output=True, text=True, check=True, cwd=ROOT,
        )
        return out.stdout.strip().splitlines()[-1]
    except Exception as err:  # noqa: BLE001 — a missing digester is reported, never faked
        return f"NOT EVALUATED ({type(err).__name__})"


def head_commit() -> str:
    try:
        h = subprocess.run(["git", "rev-parse", "--short", "HEAD"], capture_output=True, text=True, check=True, cwd=ROOT).stdout.strip()
        dirty = subprocess.run(["git", "status", "--porcelain"], capture_output=True, text=True, check=True, cwd=ROOT).stdout.strip()
        return h + ("+dirty" if dirty else "")
    except Exception:  # noqa: BLE001
        return "NOT EVALUATED"


def read_tsv(path: pathlib.Path, ncols: int):
    rows = []
    with open(path, encoding="utf-8", errors="replace") as f:
        for line in f:
            line = line.rstrip("\n")
            if not line:
                continue
            cols = line.split("\t")
            cols += [""] * (ncols - len(cols))
            rows.append(cols[:ncols])
    return rows


def suite_relpath(suite: str, repo_rel: str) -> str:
    prefix = "stimuli/sv/uvm/" if suite == "uvm-core" else f"stimuli/sv/subs/{suite}/"
    return repo_rel[len(prefix):] if repo_rel.startswith(prefix) else repo_rel


ORIGIN_BUCKETS = [
    ("class header with a base class (`extends`)", re.compile(r"\bclass\b.*\bextends\b")),
    ("`import` of a package", re.compile(r"^\s*import\b")),
    ("a `typedef`", re.compile(r"^\s*typedef\b")),
    ("a `new(…)` initializer / construction", re.compile(r"=\s*new\b")),
    ("a `::`-scoped name", re.compile(r"::")),
    ("a function / task header", re.compile(r"^\s*(virtual\s+|static\s+|protected\s+|local\s+|extern\s+|pure\s+)*(function|task)\b")),
    ("a parameter port list `#(`", re.compile(r"#\s*\(")),
    ("a `uvm_*` name (typed, not a macro)", re.compile(r"\buvm_[A-Za-z_]+")),
    ("a `constraint` / `rand`", re.compile(r"\b(constraint|rand|randc)\b")),
]


def origin_bucket(line: str) -> str:
    for label, rx in ORIGIN_BUCKETS:
        if rx.search(line):
            return label
    return "other"


_line_cache: dict[str, list[str]] = {}


def origin_line_text(origin: str) -> str | None:
    """The source line an origin names (`file:line:col`), read from the tree; None if unreadable."""
    if origin.count(":") < 2:
        return None
    file_part, line_s, _col = origin.rsplit(":", 2)
    try:
        lines = _line_cache.get(file_part)
        if lines is None:
            lines = (ROOT / file_part).read_text(errors="replace").splitlines()
            _line_cache[file_part] = lines
        return lines[int(line_s) - 1]
    except (OSError, ValueError, IndexError):
        return None


def code(text: str) -> str:
    """A GFM code span that can hold its own content: the delimiter is one backtick longer than
    the longest backtick run inside, padded with a space when the content starts or ends with one
    (TABLE-ARITY-RATCHET's unpaired-backtick arm — a `` `define `` inside a single-backtick span
    renders as a stray backtick)."""
    longest = max((len(m) for m in re.findall(r"`+", text)), default=0)
    fence = "`" * (longest + 1)
    pad = " " if text.startswith("`") or text.endswith("`") else ""
    return f"{fence}{pad}{text}{pad}{fence}"


def normalize_reason(reason: str) -> str:
    r = re.sub(r"'[^']*'", "'…'", reason)
    r = re.sub(r"\d+", "N", r)
    return r.strip()[:120]


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--pipeline", default=str(CHAR / "pipeline_results.tsv"))
    ap.add_argument("--raw", default=str(CHAR / "results.tsv"))
    ap.add_argument("--manifest", default=str(CHAR / "adjudication_manifest.tsv"))
    ap.add_argument("--context", default=str(ROOT / "stimuli" / "sv" / "pipeline_context.tsv"))
    ap.add_argument("--context-defines", default=str(ROOT / "stimuli" / "sv" / "pipeline_context_defines.tsv"))
    ap.add_argument("--context-units", default=str(ROOT / "stimuli" / "sv" / "pipeline_context_units.tsv"))
    ap.add_argument("--probe", default=str(ROOT / "rust" / "target" / "release" / "parseability_probe"))
    ap.add_argument("--out", default=str(CHAR / "pipeline_characterization.md"))
    ap.add_argument("--top", type=int, default=25)
    ap.add_argument("--check", action="store_true",
                    help="re-render into rust/target/ and diff against --out; drift exits 1, an absent dump is NOT EVALUATED")
    args = ap.parse_args()
    if args.check:
        tracked = pathlib.Path(args.out)
        for needed in (pathlib.Path(args.pipeline), pathlib.Path(args.raw)):
            if not needed.exists():
                print(f"PIPELINE-VS-RAW: NOT EVALUATED — {needed.relative_to(ROOT) if needed.is_relative_to(ROOT) else needed} is absent here (the dumps are .gitignored; run the lanes to re-derive)")
                return 0
        scratch_dir = ROOT / "rust" / "target" / "pipeline_vs_raw_check"
        scratch_dir.mkdir(parents=True, exist_ok=True)
        scratch = scratch_dir / tracked.name
        args.out = str(scratch)

    pipeline = read_tsv(pathlib.Path(args.pipeline), 7)          # suite status relpath secs rc furthest origin
    if not pipeline:
        # SVPP-EXPANSION.5(g)(h), 2026-09-09: a lane run the memory guard killed left an EMPTY dump, and the
        # render that followed published the tracked report from it (rows=0). An empty dump is not a
        # measurement; refuse to render (or check) from it, and write nothing.
        dump = pathlib.Path(args.pipeline)
        shown = dump.relative_to(ROOT) if dump.is_absolute() and dump.is_relative_to(ROOT) else dump
        print(f"PIPELINE-VS-RAW: REFUSED — {shown} has no rows (a killed or "
              f"unfinished lane run?); nothing rendered — re-run the lane, or restore the previous dump",
              file=sys.stderr)
        return 2
    raw = {r[2]: r[1] for r in read_tsv(pathlib.Path(args.raw), 3)}  # repo-rel path -> status
    manifest = {}
    mrows = read_tsv(pathlib.Path(args.manifest), 6)
    for r in mrows[1:]:                                             # suite relpath observed expected adjudication basis
        manifest[(r[0], r[1])] = r[4]
    suites = sorted({r[0] for r in pipeline})

    # ── per suite ──────────────────────────────────────────────────────────────────────────────
    per_suite = {s: collections.Counter() for s in suites}
    raw_per_suite = {s: collections.Counter() for s in suites}
    transitions = {s: collections.Counter() for s in suites}
    by_class = collections.defaultdict(collections.Counter)
    origin_files = collections.Counter()
    origin_lines = collections.Counter()
    pp_reasons = collections.Counter()
    origin_buckets = collections.Counter()
    origin_samples: dict[str, tuple[str, str]] = {}
    missing_raw = 0
    hollow = {s: 0 for s in suites}        # raw pass → pipeline fail whose origin lies in ANOTHER file
    own_file_regress = {s: 0 for s in suites}  # raw pass → pipeline fail whose origin is the row's own file
    for suite, status, rel, _secs, _rc, _fp, origin in pipeline:
        per_suite[suite][status] += 1
        raw_status = raw.get(rel)
        if raw_status is None:
            missing_raw += 1
            raw_status = "absent"
        raw_per_suite[suite][raw_status] += 1
        transitions[suite][(raw_status, status)] += 1
        if raw_status == "pass" and status == "fail":
            origin_file = origin.rsplit(":", 2)[0] if origin.count(":") >= 2 else origin
            if origin_file and origin_file != rel:
                hollow[suite] += 1
            else:
                own_file_regress[suite] += 1
        cls = manifest.get((suite, suite_relpath(suite, rel)), "not_in_manifest")
        by_class[cls][status] += 1
        if status == "fail" and origin:
            file_part = origin.rsplit(":", 2)[0] if origin.count(":") >= 2 else origin
            origin_files[file_part] += 1
            origin_lines[origin.rsplit(":", 1)[0] if origin.count(":") >= 2 else origin] += 1
            text = origin_line_text(origin)
            bucket = origin_bucket(text) if text is not None else "unreadable origin"
            origin_buckets[bucket] += 1
            origin_samples.setdefault(bucket, (origin, (text or "").strip()[:110]))
        if status == "pp_fail":
            pp_reasons[normalize_reason(origin)] += 1

    total = collections.Counter()
    for s in suites:
        total.update(per_suite[s])
    raw_total = collections.Counter()
    for s in suites:
        raw_total.update(raw_per_suite[s])

    probe = pathlib.Path(args.probe)
    grammar = ROOT / "grammars" / "systemverilog.ebnf"
    parser = ROOT / "generated" / "systemverilog_parser.rs"
    build_mode = "release" if "/release/" in str(probe) else ("debug" if "/debug/" in str(probe) else "unknown")

    def rel(p: pathlib.Path) -> str:
        try:
            return p.resolve().relative_to(ROOT).as_posix()
        except ValueError:
            return str(p)

    L = []
    L.append("# The Preprocess→Parse Lane vs the Raw Lane — `sv_2017` (SVPP-EXPANSION.4)\n")
    L.append("Generated by `stimuli/sv/pipeline_vs_raw.py` from the dumps named below. Per-file parse via\n"
             "`parseability_probe --parse systemverilog <file> --profile sv_2017 --preprocess <the suite's context>`\n"
             "against the raw lane's `--parse systemverilog <file> --profile sv_2017` on the SAME rows.\n")
    L.append("> **Characterize, don't game.** Raw outcomes only. A raw-lane `pass` on a file that only `include`s\n"
             "> others is HOLLOW (the directive is opaque to the raw parse); the pipeline pulls the included text in\n"
             "> and judges it — so a `pass → fail` here can be the honest verdict arriving, not a regression. Which it\n"
             "> is, per row, is the origin column's question.\n")
    L.append(f"Suites covered by the pipeline dump: **{', '.join(suites)}** ({sum(total.values())} rows). "
             f"Rows with no raw-lane twin: {missing_raw}.\n")
    L.append("## Instrument identity (what produced this number)\n")
    L.append("> Re-hash the rows; if any differs, this report no longer describes your tree — re-measure, do not quote.\n")
    L.append("| input | repo-root-relative path | sha256 |\n|---|---|---|")
    L.append(f"| parse binary | `{rel(probe)}` | `{sha256(probe) if probe.exists() else 'NOT EVALUATED (absent)'}` |")
    L.append(f"| grammar | `grammars/systemverilog.ebnf` | `{sha256(grammar)}` |")
    L.append(f"| grammar raw ast | `grammars/systemverilog.ebnf` | `{semantic_digest(grammar)}` |")
    L.append(f"| generated parser | `generated/systemverilog_parser.rs` | `{sha256(parser) if parser.exists() else 'NOT EVALUATED (absent)'}` |")
    L.append(f"| context | `{rel(pathlib.Path(args.context))}` | `{sha256(pathlib.Path(args.context))}` |")
    ctx_defines = pathlib.Path(args.context_defines)
    if ctx_defines.exists():
        L.append(f"| context (per-file defines) | `{rel(ctx_defines)}` | `{sha256(ctx_defines)}` |")
    ctx_units = pathlib.Path(args.context_units)
    if ctx_units.exists():
        L.append(f"| context (per-file unit predecessors) | `{rel(ctx_units)}` | `{sha256(ctx_units)}` |")
    L.append(f"| pipeline results (output) | `{rel(pathlib.Path(args.pipeline))}` | `{sha256(pathlib.Path(args.pipeline))}` |")
    L.append(f"| raw results (input) | `{rel(pathlib.Path(args.raw))}` | `{sha256(pathlib.Path(args.raw))}` |")
    L.append(f"\nMeasured at `HEAD` = `{head_commit()}`; the parse binary is a **{build_mode}** build "
             f"(the label above is the identity-carriers doctrine's, which treats the shared probe's digest as provenance, not staleness).\n")

    L.append("## Per suite — raw lane vs pipeline lane\n")
    L.append("| suite | rows | raw pass | raw fail | raw timeout | pipeline pass | pipeline fail | pp_fail | timeout | crash | Δ pass |")
    L.append("|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|")
    for s in suites + ["**total**"]:
        c = total if s == "**total**" else per_suite[s]
        r = raw_total if s == "**total**" else raw_per_suite[s]
        n = sum(c.values())
        L.append(f"| {s} | {n} | {r['pass']} | {r['fail']} | {r['timeout']} | {c['pass']} | {c['fail']} | {c['pp_fail']} | {c['timeout']} | {c['crash']} | {c['pass'] - r['pass']:+d} |")

    L.append("\n## Transitions — raw status → pipeline status\n")
    L.append("| suite | raw → pipeline | rows |\n|---|---|---:|")
    for s in suites:
        for (a, b), n in sorted(transitions[s].items(), key=lambda kv: (-kv[1], kv[0])):
            L.append(f"| {s} | `{a}` → `{b}` | {n} |")
    L.append("\n**`pass → fail`, split by where the rejection lands.** A raw `pass` on a file whose rejection now"
             " lands in ANOTHER file was HOLLOW — the raw parse never saw the included text; a rejection landing in"
             " the row's OWN file is the case to read as a candidate regression.\n")
    L.append("| suite | `pass → fail` landing in an included file (hollow raw pass) | landing in the row's own file |\n|---|---:|---:|")
    for s in suites:
        L.append(f"| {s} | {hollow[s]} | {own_file_regress[s]} |")

    L.append("\n## By adjudication class (the answer key's verdict for the row, raw lane)\n")
    L.append("| adjudication class | rows | pipeline pass | pipeline fail | pp_fail | timeout | crash |\n|---|---:|---:|---:|---:|---:|---:|")
    for cls, c in sorted(by_class.items(), key=lambda kv: (-sum(kv[1].values()), kv[0])):
        L.append(f"| `{cls}` | {sum(c.values())} | {c['pass']} | {c['fail']} | {c['pp_fail']} | {c['timeout']} | {c['crash']} |")

    L.append("\n## What sits at the origin — the construct on the rejecting line, bucketed\n")
    L.append("> The line each `fail` row's origin names, read from the tree and matched against a short ordered\n"
             "> regex list (first match wins). This says what kind of barrier the pipeline uncovered once the\n"
             "> macro barrier was gone; `other` is the residue the bucketing does not name.\n")
    L.append("| construct at the origin line | rows | one example (origin — line) |\n|---|---:|---|")
    for b, n in origin_buckets.most_common():
        o, t = origin_samples.get(b, ("", ""))
        L.append(f"| {b} | {n} | {code(o)} — {code(t)} |")
    L.append(f"\n## Where the pipeline's rejections land — top {args.top} ORIGIN FILES\n")
    L.append("> The origin is the source file and line the deepest-reaching byte came from, mapped through the\n"
             "> preprocessor's source map: a header that many files `include` shows up once per includer.\n")
    L.append("| origin file | rejections landing in it |\n|---|---:|")
    for f_, n in origin_files.most_common(args.top):
        L.append(f"| {code(f_)} | {n} |")
    L.append(f"\n## Top {args.top} ORIGIN LINES\n")
    L.append("| origin file:line | rejections |\n|---|---:|")
    for l_, n in origin_lines.most_common(args.top):
        L.append(f"| {code(l_)} | {n} |")

    L.append("\n## Preprocessor refusals (`pp_fail`) by reason\n")
    L.append("| reason (paths and numbers normalised) | rows |\n|---|---:|")
    for r_, n in pp_reasons.most_common(args.top):
        L.append(f"| {code(r_)} | {n} |")
    if not pp_reasons:
        L.append("| (none) | 0 |")

    L.append("\n## The context the lane ran under\n")
    L.append("| suite | kind | value | needed by |\n|---|---|---|---:|")
    for r in read_tsv(pathlib.Path(args.context), 5)[1:]:
        if r[0] in suites:
            L.append(f"| {r[0]} | {r[1]} | {code(r[2])} | {r[3]} |")
    # SVPP-EXPANSION.4c.1 — the defines a file's OWN driver passes for it (verilator `t_x.py`, sv-tests
    # `:defines:`), applied per file as `--sv-define`; derived with a literal value only, ties and
    # computed values reported by the deriver, never guessed
    if ctx_defines.exists():
        drows = [r for r in read_tsv(ctx_defines, 4)[1:] if r[0] in suites]
        L.append("\n### Per-file defines (the file's own driver)\n")
        L.append("| suite | files | defines | e.g. |\n|---|---:|---:|---|")
        for suite in sorted({r[0] for r in drows}):
            mine = [r for r in drows if r[0] == suite]
            first = mine[0]
            L.append(f"| {suite} | {len({r[1] for r in mine})} | {len(mine)} | {code(first[1].rsplit('/', 1)[-1])} ← {code('--sv-define ' + first[2])} |")
        if not drows:
            L.append("| (none) | 0 | 0 | |")
    # SVPP-EXPANSION.4c.2 — the source files a file's OWN harness compiles before it in the same unit
    # (verilator `t_x.py` source arguments, slang `data/*.f`, sv2v `run.sh` explicit lists), applied per
    # file as `--sv-unit-file` in order; a -f list, a glob, a --siloed run and an ordering conflict are
    # reported by the deriver, never guessed
    if ctx_units.exists():
        urows = [r for r in read_tsv(ctx_units, 5)[1:] if r[0] in suites]
        L.append("\n### Per-file unit predecessors (the file's own harness)\n")
        L.append("| suite | files | predecessor rows | e.g. |\n|---|---:|---:|---|")
        for suite in sorted({r[0] for r in urows}):
            mine = [r for r in urows if r[0] == suite]
            first = mine[0]
            L.append(f"| {suite} | {len({r[1] for r in mine})} | {len(mine)} | {code(first[1].rsplit('/', 1)[-1])} ← {code(first[3].rsplit('/', 1)[-1])} |")
        if not urows:
            L.append("| (none) | 0 | 0 | |")

    out = pathlib.Path(args.out)
    out.write_text("\n".join(L) + "\n")
    print(f"PIPELINE-VS-RAW: suites={','.join(suites)} rows={sum(total.values())} raw_pass={raw_total['pass']} pipeline_pass={total['pass']} "
          f"pipeline_fail={total['fail']} pp_fail={total['pp_fail']} timeout={total['timeout']} crash={total['crash']} missing_raw={missing_raw} → {rel(out)}")
    if args.check:
        tracked = CHAR / "pipeline_characterization.md"
        if not tracked.exists():
            print(f"PIPELINE-VS-RAW: DRIFT — the tracked {rel(tracked)} does not exist; run without --check to publish it", file=sys.stderr)
            return 1
        # the identity block names the PROBE by digest; a rebuilt probe changes that row without any
        # number moving, and that row is provenance (IDENTITY-CARRIERS treats `parse binary` as a note),
        # so the comparison masks it and holds every other line equal. The `Measured at HEAD` line is
        # provenance of the same kind and is masked too (SVPP-EXPANSION.5(g)(h), 2026-09-09): it names
        # the commit the RENDER ran on, so the tracked report was DRIFT on every commit after the one
        # that published it — the doctrine asked for a re-render per commit of the repository, which is
        # a treadmill, not currency. The numbers, the tables and the context are what must hold equal.
        PROVENANCE_PREFIXES = ("| parse binary |", "Measured at `HEAD` = ")
        def masked(text: str) -> str:
            return "\n".join(
                (next((pfx for pfx in PROVENANCE_PREFIXES if line.startswith(pfx)), line))
                for line in text.splitlines()
            )
        if masked(tracked.read_text()) != masked(out.read_text()):
            print(f"PIPELINE-VS-RAW: DRIFT — {rel(tracked)} differs from a fresh render of the dumps on this tree (scratch: {rel(out)}); re-run the lane and publish", file=sys.stderr)
            return 1
        print(f"PIPELINE-VS-RAW: OK — {rel(tracked)} equals a fresh render of the dumps (the probe digest row and the HEAD line masked as provenance)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
