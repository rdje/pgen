#!/usr/bin/env python3
"""SV-CORPUS-GRAD.13c.2v.2 — the GENERATED `systemverilog_preprocessor` parser over the SystemVerilog corpus.

The svpp family publishes its corpus leg as NOT MEASURED (no corpus of directive files is wired). Every file
the SystemVerilog lanes read IS a preprocessor input, though: a directive-structure parser that refuses a
file a SystemVerilog tool compiles has refused valid clause-22 text, unless the file itself is invalid. This
census asks that question of the parser a downstream consumer receives — the generated one inside the
release `parseability_probe` (`--parse systemverilog_preprocessor`) — never of the interpreter.

For each corpus file (the path column of `stimuli/sv/characterization/results.tsv`, the SV raw lane's own
population) it records the verdict and, for a rejection, the line holding the furthest position, bucketed
by the backtick word that opens it (`` `resetall ``, a macro usage, …) or OTHER.

With `--before BIN` it also reads every file under a second binary and publishes the TRANSITIONS
(`fail -> pass`, `pass -> fail`, …) and the AST IDENTITY of every file both accept (`--parse-dump-ast`
under each binary, compared byte for byte): a previously accepted file whose tree moved is a shape change
a consumer sees, and each one must be explained.

Every rejection the census reports is looked up in `residual_dispositions.tsv` (path -> disposition ->
the LRM text or the suite's own verdict that makes it right). `--check-dispositions` exits 1 when a
rejection carries no disposition, or a disposition names a file the parser now accepts.

`--bank` runs the clause-22 reproducer bank (`bank/MANIFEST.tsv`: one file per case, ACCEPT or REJECT, the
clause that decides it) through the same generated parser; exit 1 on any case that does not behave as keyed.
It needs no corpus, so it runs on a fresh clone.

⛔ The population is a lane DUMP (`results.tsv` is .gitignored): without it the census REFUSES (exit 2) rather
than measuring an empty corpus; the report records a digest of the sorted path list it read.

Usage (repo-root-relative paths; writes only inside this directory):
  python3 docs/tasks/artifacts/sv_corpus_grad/svpp_corpus_census/census.py --probe rust/target/release/parseability_probe
  python3 … --probe NEW --before OLD          # + transitions and AST identity
  python3 … --check-dispositions              # rc 1 on an unadjudicated rejection (reads rejections.tsv)
  python3 … --bank --probe BIN                # the clause-22 reproducer bank, rc 1 on any miss
  python3 … --self-test                       # the classifier and the disposition join, no parser
"""
from __future__ import annotations

import argparse
import concurrent.futures as cf
import hashlib
import os
import re
import subprocess
import sys
import tempfile
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]
HERE = Path(__file__).resolve().parent
POPULATION = "stimuli/sv/characterization/results.tsv"
GRAMMAR = "grammars/systemverilog_preprocessor.ebnf"
DIGESTER = "scripts/check_baseline_identity.sh"
DISPOSITIONS = HERE / "residual_dispositions.tsv"
BANK = HERE / "bank"
REJECTIONS = HERE / "rejections.tsv"
MOVERS = HERE / "movers.tsv"
REPORT = HERE / "census.md"
FAMILY = "systemverilog_preprocessor"
TAG = "SVPP-CORPUS-CENSUS"
TOP_WORDS = 40   # the per-word table's length; the remainder is one summed row
FURTHEST = re.compile(r"furthest_position=(\d+)")
# the backtick word opening a line, after any run of block comments and blanks
OPENING_WORD = re.compile(rb"^[ \t]*(?:/\*.*?\*/[ \t]*)*`([A-Za-z_][A-Za-z0-9_$]*)")
# IEEE 1800-2017 / 1800-2023 §22.1's directive list, minus `__FILE__` / `__LINE__` (§22.13: predefined MACROS)
CLAUSE_22_DIRECTIVES = frozenset("""begin_keywords celldefine default_nettype define else elsif end_keywords
    endcelldefine endif ifdef ifndef include line nounconnected_drive pragma resetall timescale unconnected_drive
    undef undefineall""".split())


def shape_class(shape: str) -> str:
    if shape == "OTHER":
        return "no backtick word opens the stopping line"
    return "a clause-22 directive" if shape[1:] in CLAUSE_22_DIRECTIVES else "a text macro usage (§22.5.1)"


def code(text: str) -> str:
    """A markdown code span that survives a backtick inside it."""
    return f"`` {text} ``" if "`" in text else f"`{text}`"


def rel(path: Path) -> str:
    return os.path.relpath(path, ROOT)


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def population() -> list[str] | None:
    """The SV raw lane's file list, or None when the (gitignored) dump is absent."""
    if not (ROOT / POPULATION).is_file():
        return None
    rows = (ROOT / POPULATION).read_text(encoding="utf-8").splitlines()
    return sorted({r.split("\t")[2] for r in rows if r.count("\t") >= 2})


def population_digest(paths: list[str]) -> str:
    return hashlib.sha256("\n".join(paths).encode()).hexdigest()


def classify(data: bytes, pos: int) -> tuple[str, int, str]:
    """(shape, 1-based line, the line's first 100 characters) for the line holding byte `pos`."""
    pos = max(0, min(pos, len(data)))
    start = data.rfind(b"\n", 0, pos) + 1
    end = data.find(b"\n", pos)
    line = data[start:end if end >= 0 else len(data)]
    m = OPENING_WORD.match(line)
    shape = ("`" + m.group(1).decode()) if m else "OTHER"
    text = line[:100].decode("utf-8", "replace").replace("\t", " ").rstrip("\r")
    return shape, data.count(b"\n", 0, start) + 1, text


def parse(probe: str, path: str) -> tuple[str, int]:
    proc = subprocess.run([probe, "--parse", FAMILY, path], capture_output=True, cwd=ROOT, timeout=600)
    if proc.returncode == 0:
        return "pass", -1
    m = FURTHEST.search(proc.stderr.decode("utf-8", "replace"))
    return ("fail", int(m.group(1))) if m else ("error", -1)


def tree_digest(probe: str, path: str, scratch: str, tag: str) -> str:
    out = os.path.join(scratch, f"{hashlib.sha1(path.encode()).hexdigest()}.{tag}.json")
    proc = subprocess.run([probe, "--parse-dump-ast", FAMILY, path, out], capture_output=True, cwd=ROOT,
                          timeout=600)
    if proc.returncode != 0 or not os.path.exists(out):
        return "unavailable"
    digest = hashlib.sha256(Path(out).read_bytes()).hexdigest()
    os.remove(out)
    return digest


def load_dispositions() -> dict[str, tuple[str, str]]:
    if not DISPOSITIONS.exists():
        return {}
    table = {}
    for line in DISPOSITIONS.read_text(encoding="utf-8").splitlines():
        if not line or line.startswith("#"):
            continue
        path, disposition, cite = line.split("\t", 2)
        table[path] = (disposition, cite)
    return table


def judge_dispositions(rejected: list[str], table: dict[str, tuple[str, str]]) -> tuple[list[str], list[str]]:
    """(rejections with no disposition, dispositions naming a file that is no longer rejected)."""
    rejected_set = set(rejected)
    return sorted(p for p in rejected_set if p not in table), sorted(p for p in table if p not in rejected_set)


def measure(args) -> int:
    probe = args.probe
    before = args.before
    for binary in [probe] + ([before] if before else []):
        if not (ROOT / binary).is_file():
            print(f"{TAG}: REFUSE — {binary} is not a file", file=sys.stderr)
            return 2
    paths = population()
    if paths is None:
        print(f"{TAG}: REFUSE — {POPULATION} is absent (a gitignored lane dump: run "
              f"`stimuli/run_external_corpus.sh sv 60 8 0` first)", file=sys.stderr)
        return 2
    scratch = tempfile.mkdtemp(prefix="svpp_census_", dir=ROOT / "rust/target")

    def one(path: str):
        verdict, pos = parse(probe, path)
        shape = line = text = ""
        if verdict == "fail":
            shape, line, text = classify((ROOT / path).read_bytes(), pos)
        old = tree = ""
        if before:
            old, _ = parse(before, path)
            if old == "pass" and verdict == "pass":
                same = tree_digest(probe, path, scratch, "new") == tree_digest(before, path, scratch, "old")
                tree = "same" if same else "MOVED"
        return path, verdict, shape, line, text, old, tree

    with cf.ThreadPoolExecutor(max_workers=args.jobs) as pool:
        rows = list(pool.map(one, paths))
    os.rmdir(scratch)

    rejected = [r for r in rows if r[1] != "pass"]
    with REJECTIONS.open("w", encoding="utf-8") as out:
        out.write("# path\tverdict\tshape\tline\ttext (the line holding the furthest position)\n")
        for path, verdict, shape, line, text, _, _ in rejected:
            out.write(f"{path}\t{verdict}\t{shape}\t{line}\t{text}\n")
    if before:
        with MOVERS.open("w", encoding="utf-8") as out:
            out.write("# path\tbefore\tafter\ttree (both accept: same / MOVED)\n")
            for path, verdict, _, _, _, old, tree in rows:
                if old != verdict or tree == "MOVED":
                    out.write(f"{path}\t{old}\t{verdict}\t{tree}\n")

    table = load_dispositions()
    missing, stale = judge_dispositions([r[0] for r in rejected], table)
    digest = subprocess.run(["bash", DIGESTER, "--digest", "ebnf_raw_ast", GRAMMAR], capture_output=True,
                            text=True, cwd=ROOT).stdout.strip()
    write_report(rows, probe, before, digest, table, missing, stale)
    verdicts = Counter(r[1] for r in rows)
    print(f"{TAG}: files={len(rows)} " + " ".join(f"{k}={v}" for k, v in sorted(verdicts.items()))
          + f" unadjudicated={len(missing)} stale_dispositions={len(stale)}")
    return 0


def write_report(rows, probe, before, digest, table, missing, stale) -> None:
    verdicts = Counter(r[1] for r in rows)
    rejected = [r for r in rows if r[1] != "pass"]
    shapes = Counter(r[2] for r in rejected)
    lines = [
        "# The generated `systemverilog_preprocessor` parser over the SystemVerilog corpus (SV-CORPUS-GRAD.13c.2v.2)",
        "",
        "> Generated by `docs/tasks/artifacts/sv_corpus_grad/svpp_corpus_census/census.py`. Do not hand-edit:",
        "> re-run it with the release `parseability_probe` after every `systemverilog_preprocessor` release.",
        "> The parser measured is the GENERATED one a downstream consumer receives, never the interpreter.",
        "",
        "## Instrument identity",
        "",
        "| input | repo-root-relative path | sha256 |",
        "|---|---|---|",
        f"| systemverilog_preprocessor raw ast | `{GRAMMAR}` | `{digest}` |",
        f"| parse binary | `{probe}` | `{sha256(ROOT / probe)}` |",
    ]
    if before:
        lines.append(f"| parse binary (before) | `{before}` | `{sha256(ROOT / before)}` |")
    lines += [
        "",
        f"Population: every path in the SV raw lane's `{POPULATION}` (a gitignored lane dump) — **{len(rows):,} files**,",
        "the corpus the SystemVerilog lanes read (each is a preprocessor input whatever its SystemVerilog verdict);",
        f"the sorted path list's sha256 is `{population_digest([r[0] for r in rows])}`.",
        "",
        "## The headline",
        "",
        f"- **accepted: {verdicts['pass']:,}** · **rejected: {verdicts['fail']:,}**"
        + (f" · **no verdict (the probe printed no position): {verdicts['error']:,}**" if verdicts["error"] else ""),
        f"- every rejection adjudicated: **{'yes' if not missing else 'NO — ' + str(len(missing)) + ' unadjudicated'}**"
        f" (`residual_dispositions.tsv`, {len(table)} rows; stale rows: {len(stale)})",
        "",
    ]
    if before:
        trans = Counter(f"{r[5]} → {r[1]}" for r in rows)
        trees = Counter(r[6] for r in rows if r[6])
        lines += ["## Transitions against the `before` binary", "", "| before → after | files |", "|---|---:|"]
        lines += [f"| `{k}` | {v:,} |" for k, v in sorted(trans.items())]
        lines += ["", f"AST identity over the {sum(trees.values()):,} files both binaries accept: "
                  f"**{trees['same']:,} byte-identical**, **{trees['MOVED']:,} moved** (each listed in `movers.tsv`).", ""]
    classes = Counter(shape_class(r[2]) for r in rejected)
    lines += ["## Rejections by the line that stops the parse", "",
              "| what opens the stopping line | files |", "|---|---:|"]
    lines += [f"| {c} | {n:,} |" for c, n in sorted(classes.items(), key=lambda kv: (-kv[1], kv[0]))]
    lines += ["", "| opening word of the stopping line | class | files |", "|---|---|---:|"]
    ranked = sorted(shapes.items(), key=lambda kv: (-kv[1], kv[0]))
    lines += [f"| {code(s) if s != 'OTHER' else 'OTHER'} | {shape_class(s)} | {n:,} |" for s, n in ranked[:TOP_WORDS]]
    if len(ranked) > TOP_WORDS:
        rest = ranked[TOP_WORDS:]
        lines.append(f"| … {len(rest):,} more words | (every row is in `rejections.tsv`) | {sum(n for _, n in rest):,} |")
    if table:
        by_disposition = Counter(table[r[0]][0] for r in rejected if r[0] in table)
        lines += ["", "## Every rejection, by disposition", "", "| disposition | files |", "|---|---:|"]
        lines += [f"| `{d}` | {n:,} |" for d, n in sorted(by_disposition.items(), key=lambda kv: (-kv[1], kv[0]))]
    if missing:
        lines += ["", "## ⛔ Unadjudicated rejections", ""] + [f"- `{p}`" for p in missing[:TOP_WORDS]]
        if len(missing) > TOP_WORDS:
            lines.append(f"- … and {len(missing) - TOP_WORDS:,} more (every rejection is a row of `rejections.tsv`)")
    lines.append("")
    REPORT.write_text("\n".join(lines), encoding="utf-8")


def run_bank(probe: str) -> int:
    manifest = BANK / "MANIFEST.tsv"
    if not manifest.is_file() or not (ROOT / probe).is_file():
        print(f"{TAG}-BANK: REFUSE — {rel(manifest)} or {probe} is missing", file=sys.stderr)
        return 2
    cases = [l.split("\t") for l in manifest.read_text(encoding="utf-8").splitlines() if l and not l.startswith("#")]
    misses = []
    for case in cases:
        name, expect = case[0], case[1]
        verdict, _ = parse(probe, rel(BANK / name))
        got = "ACCEPT" if verdict == "pass" else "REJECT"
        if got != expect:
            misses.append(f"{name}: keyed {expect}, the parser says {got}")
    for m in misses:
        print(f"{TAG}-BANK: ✗ {m}", file=sys.stderr)
    counts = Counter(c[1] for c in cases)
    print(f"{TAG}-BANK: cases={len(cases)} accept={counts['ACCEPT']} reject={counts['REJECT']} failures={len(misses)}")
    return 1 if misses else 0


def check_dispositions() -> int:
    if not REJECTIONS.exists():
        print(f"{TAG}: NOT EVALUATED — {rel(REJECTIONS)} is absent (run the census first)")
        return 0
    rejected = [l.split("\t", 1)[0] for l in REJECTIONS.read_text(encoding="utf-8").splitlines()
                if l and not l.startswith("#")]
    missing, stale = judge_dispositions(rejected, load_dispositions())
    for p in missing:
        print(f"{TAG}: ✗ unadjudicated rejection {p}", file=sys.stderr)
    for p in stale:
        print(f"{TAG}: ✗ disposition names a file the parser no longer rejects: {p}", file=sys.stderr)
    if missing or stale:
        return 1
    print(f"{TAG}: OK — {len(rejected)} rejections, every one adjudicated")
    return 0


def self_test() -> int:
    failures = 0

    def expect(label, got, want):
        nonlocal failures
        ok = got == want
        failures += not ok
        print(f"  {'ok ' if ok else 'FAIL'} {label}: got {got!r}" + ("" if ok else f", want {want!r}"))

    data = b"module m;\n  `uvm_info(\"a\")\n/* c */ `resetall x\nplain\n"
    expect("macro usage line", classify(data, data.index(b"`uvm"))[:2], ("`uvm_info", 2))
    expect("comment-led directive", classify(data, data.index(b"`reset"))[:2], ("`resetall", 3))
    expect("plain text line", classify(data, data.index(b"plain"))[:2], ("OTHER", 4))
    expect("position past the end", classify(data, len(data) + 9)[1], 5)
    table = {"a.sv": ("must_reject", "x"), "gone.sv": ("must_reject", "y")}
    expect("disposition join", judge_dispositions(["a.sv", "b.sv"], table), (["b.sv"], ["gone.sv"]))
    expect("directive class", shape_class("`resetall"), "a clause-22 directive")
    expect("predefined macro class", shape_class("`__FILE__"), "a text macro usage (§22.5.1)")
    expect("code span with a backtick", code("`x"), "`` `x ``")
    print(f"{TAG}-SELF-TEST: {'pass' if not failures else 'FAIL'} ({8 - failures}/8)")
    return 1 if failures else 0


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--probe", help="the parseability_probe to measure (repo-root-relative)")
    ap.add_argument("--before", help="a second parseability_probe: transitions + AST identity")
    ap.add_argument("--jobs", type=int, default=8)
    ap.add_argument("--check-dispositions", action="store_true")
    ap.add_argument("--bank", action="store_true", help="run the clause-22 reproducer bank through --probe")
    ap.add_argument("--self-test", action="store_true")
    args = ap.parse_args()
    if args.self_test:
        return self_test()
    if args.check_dispositions:
        return check_dispositions()
    if args.bank:
        if not args.probe:
            ap.error("--bank needs --probe")
        return run_bank(args.probe)
    if not args.probe:
        ap.error("--probe is required unless --check-dispositions or --self-test")
    return measure(args)


if __name__ == "__main__":
    sys.exit(main())
