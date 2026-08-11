#!/usr/bin/env python3
"""Adjudicate the DARK `chained_only` candidate worklist ROW BY ROW (SV-CORPUS-GRAD.13c.2)

WHY THIS EXISTS
---------------
`.13c` reduced 4 158 DARK `deferred:chained_only` rows to a **57-row candidate worklist** and
said, in its own words, ⛔ *"a worklist is not a defect count"*: each row still had to be
adjudicated with the toolbox before anyone could call it a parser defect.  This instrument is
that adjudication, and it is mechanical rather than editorial.

⛔⛔ THE ONE RULE THIS INSTRUMENT OBEYS: **A ROW EARNS ITS VERDICT BY BEING PARSED.**
Nothing here is relabelled by argument.  Every verdict below is the outcome of a REAL parse of
a NAMED, reproducible transformation of the row's own bytes, and the transformation is
published with the verdict so anyone can re-run it:

    CROSS-FILE-FACTS   declaring the names the parser demanded and lacked makes the file PARSE.
                       ⇒ the row is blocked by cross-file DECLARATIONS, exactly the `.4`
                       unit-level-fact-continuity capability, and is NOT a parser defect.  The
                       MINIMAL name set AND the minimal declaration FORM are published per row.
    FRAGMENT-<KIND>    the file does not parse standalone but DOES parse wrapped in
                       `module`/`package`/`class`/`interface` ⇒ it is an include payload, not a
                       legal standalone compilation unit.  Not a parser defect; a UNIT-SHAPE
                       fact about the corpus (`.13g`'s question, answered per row).
    MACRO-BLOCKED      after those two are removed, the parse dies ON an undefined macro
                       invocation.  ⭐ The row's FIRST failure was text-refuted (`.13c`), and
                       its RESIDUAL failure is not: expansion owns what is left.  Not a parser
                       defect.  ⛔ Nothing is expanded here — the verdict is the position of the
                       choking token, read off the parse.
    RESIDUAL           no transformation makes it parse.  ⇒ the row survives every cheap
                       explanation and is escalated to a per-row structural diagnosis
                       (minimal reproducer + scoped `--trace-rules`), which decides DEFECT vs
                       genuinely-invalid SV.  ⛔ RESIDUAL is a WORKLIST, not a defect count —
                       the same discipline `.13c` imposed one level up.

⛔ WHY THE `.13c` LABELS COULD NOT BE TRUSTED AS-IS (measured, not suspected).  `.13c`'s fact
channel intersects the demanded-and-missing `type_name` names with the identifiers in the
failure REGION, then returns `FACT-GATED-LOCAL-TYPE` as soon as ANY of them is declared in the
file.  On `uvm_policies.svh` the region holds both `T` (a local `#(type T=int)` parameter) and
`uvm_object` (the cross-file return type that actually blocks the parse), and the LOCAL name
wins the label.  The parser says otherwise in its own words:

    ↪ NEGATIVE: 15 facts of kind 'type_name' exist (none matched name Identifier("uvm_object"))

and prepending one `class uvm_object; endclass` makes the whole file parse.  A text-region
intersection cannot see that; a parse can.  ⇒ this instrument never reads a label, only bytes.

HOW A ROW IS DRIVEN
-------------------
For each wrapper in (none, module, package, class, interface, program), a fixed-point loop:

  1. parse the current text; ACCEPT ⇒ done, verdict for this wrapper;
  2. otherwise run the parse again under `PGEN_TRACE_VERBOSITY=high --trace` and harvest every
     identifier for which a `type_name` fact was DEMANDED AND MISSING (TOOLBOX §2.4), keeping
     only those in the failure REGION -- a PEG parser speculates a type reading on ordinary
     tokens constantly, so an unscoped harvest would excuse every row (`.13c`'s finding, kept);
  3. add the new ones to the prelude and go to 1; stop after `MAX_ROUNDS` or when a round adds
     nothing new.

On ACCEPT the prelude is then MINIMIZED (drop each name, keep it only if its removal flips the
parse back to REJECT), so the published set is the smallest declaration set that explains the
row -- not whatever the loop happened to accumulate.  ⛔ Without minimization a bogus prelude
could excuse a genuine defect, which is the one direction this leaf must not fail in.

⚠️ HONEST BOUNDS, carried in the artifact itself:
  * the prelude declares each name in the SMALLEST of four escalating forms that works --
    `class` (satisfies `has_fact(type_name, N)` and `declaration_family, class`), `module`,
    then both, then all four (`class`/`module`/`package`/`interface`).  A name whose use site
    demands a family none of those supply (`nettype`, `covergroup`) will NOT be satisfied and
    its row stays RESIDUAL -- the failure direction is conservative (toward more work, not
    less).  The form actually used is published per row, so nothing is excused invisibly.
  * tracing routes the parse onto the PROTOCOL graph rather than the fused `cascade_*` graph
    (TOOLBOX §2.1).  The two are held byte-identical by the equivalence/AST oracles, so the
    verdict is stable -- but the harvested trace is not literally the untraced run, and saying
    so is part of the evidence.
  * a wrapper answers "is this a legal STANDALONE unit?", never "is this construct legal SV".
    A FRAGMENT verdict is a statement about the corpus row, not about the parser.

Outputs (repo-root-relative, deterministic -- rows sorted by (verdict, suite, relpath)):
    docs/tasks/artifacts/sv_corpus_grad/dark_chained_only/adjudication.tsv
    docs/tasks/artifacts/sv_corpus_grad/dark_chained_only/adjudication.md

Run (under the memory guard, as every heavy job here is):
    scripts/run_with_memory_guard.sh --budget-mb 8192 --timeout-s 3600 -- \
        python3 stimuli/sv/adjudicate_dark_worklist.py
"""
from __future__ import annotations

import argparse
import csv
import hashlib
import os
import re
import shlex
import subprocess
import sys
import tempfile
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
from threading import Lock

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "stimuli/sv"))

from audit_dark_chained_only import (  # noqa: E402  (path set above, by design)
    FACT_MISS_RE,
    declared_type_names,
    failure_region_names,
    sv_reserved_words,
)
from audit_explained_svpp import (  # noqa: E402
    FURTHEST_RE,
    SURFACE_RE,
    SVPP_PASSTHROUGH,
    TICK_NAME_RE,
    resolve_path,
    stuck_offset_of,
)
from adjudicate_external_corpus import blank_comments_and_strings  # noqa: E402

WORKLIST = ROOT / "docs/tasks/artifacts/sv_corpus_grad/dark_chained_only/worklist.tsv"
RESIDUAL_ROWS = ROOT / "stimuli/sv/adjudication_repros/RESIDUAL_ROWS.tsv"
OUT_TSV = ROOT / "docs/tasks/artifacts/sv_corpus_grad/dark_chained_only/adjudication.tsv"
OUT_MD = ROOT / "docs/tasks/artifacts/sv_corpus_grad/dark_chained_only/adjudication.md"
GRAMMAR = ROOT / "grammars/systemverilog.ebnf"
PROBE = ROOT / "rust/target/release/parseability_probe"

# (name, prefix, suffix) -- `none` MUST come first: a row that parses standalone needs no story.
WRAPPERS = [
    ("none", "", ""),
    ("module", "module pgen_adj_wrap;\n", "\nendmodule\n"),
    ("package", "package pgen_adj_wrap;\n", "\nendpackage\n"),
    ("class", "class pgen_adj_wrap;\n", "\nendclass\n"),
    ("interface", "interface pgen_adj_wrap;\n", "\nendinterface\n"),
    ("program", "program pgen_adj_wrap;\n", "\nendprogram\n"),
]
MAX_ROUNDS = 14
TIMEOUT_S = 120

# Escalating declaration FORMS. The search takes the first that works, so a row is never
# excused by a more permissive declaration than its own bytes actually need -- and the winning
# form is published beside the verdict.
FORM_TEXT = {
    "class": "class {n}; endclass\n",
    "module": "module {n}; endmodule\n",
    "package": "package {n}; endpackage\n",
    "interface": "interface {n}; endinterface\n",
}
FORM_LADDER = [("class",), ("module",), ("class", "module"),
               ("class", "module", "package", "interface")]


def prelude_for(names, forms) -> str:
    return "".join(FORM_TEXT[f].format(n=n) for n in sorted(names) for f in forms)


def compose(raw: str, wrapper, names, forms) -> str:
    _, pre, post = wrapper
    return prelude_for(names, forms) + pre + raw + post


def probe_text(text: str, trace: bool):
    """Parse `text` as a standalone file. Returns (accepted, surface, furthest, trace_stdout)."""
    with tempfile.NamedTemporaryFile("w", suffix=".sv", dir=str(ROOT / "tmp"),
                                     delete=False, encoding="latin-1") as fh:
        fh.write(text)
        tmp = Path(fh.name)
    try:
        if trace:
            cmd = (f'PGEN_TRACE_VERBOSITY=high {shlex.quote(str(PROBE))} --parse systemverilog '
                   f'{shlex.quote(str(tmp))} --profile sv_2017 --trace 2>&1 '
                   f'| grep -aE "has_fact|fact_attribute_equals"')
            proc = subprocess.run(["/bin/sh", "-c", cmd], capture_output=True, text=True,
                                  timeout=TIMEOUT_S)
            return (None, None, None, proc.stdout)
        proc = subprocess.run(
            [str(PROBE), "--parse", "systemverilog", str(tmp), "--profile", "sv_2017"],
            capture_output=True, text=True, timeout=TIMEOUT_S)
        out = (proc.stderr or "") + (proc.stdout or "")
        fm, sm = FURTHEST_RE.search(out), SURFACE_RE.search(out)
        return (proc.returncode == 0,
                int(sm.group(1)) if sm else None,
                int(fm.group(1)) if fm else None,
                out)
    except subprocess.TimeoutExpired:
        return (False, None, None, "TIMEOUT")
    finally:
        try:
            os.unlink(tmp)
        except OSError:
            pass


_DECLARABLE: dict = {}
_DECLARABLE_LOCK = Lock()


def declarable(name: str) -> bool:
    """Can this identifier legally BE declared? ⛔ Answered by the parser, not by a hand-typed
    keyword list. `reserved_non_keyword_identifier_sv` (the audit's source of truth for "is this
    a name?") holds 173 spellings and does NOT include `covergroup`, `endgroup`, `virtual`,
    `bind` or `new` — all of which a speculating PEG parser really does report as
    demanded-and-missing `type_name`s. A prelude containing `class covergroup; endclass` is a
    SYNTAX ERROR, and it would poison every later probe of that row into a false RESIDUAL. So
    each candidate's own declaration block is parsed before it is allowed into a prelude."""
    with _DECLARABLE_LOCK:
        hit = _DECLARABLE.get(name)
    if hit is None:
        ok, _, _, _ = probe_text(prelude_for({name}, FORM_LADDER[-1]), trace=False)
        hit = bool(ok)
        with _DECLARABLE_LOCK:
            _DECLARABLE[name] = hit
    return hit


# ⛔ WIDER THAN `.13c`'s HARVEST, ON PURPOSE — and the widening is what a MEASUREMENT forced.
# `.13c` matched `type_name` only, which is right for its question (is a sibling `typedef` the
# live dependency?) and wrong for this one. `cip_lc_tx_cov_if.sv` opens `import uvm_pkg::*;`:
# the missing declaration is a PACKAGE, no `type_name` fact is ever demanded for it, and the
# row sat in RESIDUAL — i.e. as a candidate parser defect — until the harvest could see it.
# Reconstructed by hand the file parses (`repro L6`), so the row was never a defect at all.
ANY_FACT_MISS_RE = re.compile(
    r'has_fact\(kind=\w+, name=Identifier\("([^"]+)"\)\) → false'
    r'|rejected by post predicate .has_fact \[Identifier\("\w+"\), Identifier\("([^"]+)"\)\]'
    r'|fact_attribute_equals\(kind=\w+, name=Identifier\("([^"]+)"\)[^\n]*→ false'
    r'|rejected by post predicate .fact_attribute_equals \[Identifier\("\w+"\), '
    r'Identifier\("([^"]+)"\)')


def demanded_missing(text: str, reserved) -> set:
    """Identifiers for which the parser DEMANDED a declaration fact it did not have."""
    _, _, _, out = probe_text(text, trace=True)
    names = set()
    for rx in (FACT_MISS_RE, ANY_FACT_MISS_RE):
        for m in rx.finditer(out):
            n = next(g for g in m.groups() if g)
            if n not in reserved and n.isidentifier() and declarable(n):
                names.add(n)
    return names


def reach(raw, wrapper, names, forms) -> int:
    """How far into the composed text the parse gets. The search's only objective function."""
    ok, surface, furthest, _ = probe_text(compose(raw, wrapper, names, forms), trace=False)
    if ok:
        return 1 << 30
    return max(furthest or 0, surface or 0) - len(prelude_for(names, forms))


def drive(raw: str, wrapper, forms, reserved, seed_names=frozenset()):
    """Fixed-point: parse, harvest the missing declaration facts AT THE FAILURE, declare, repeat.

    ⛔⛔ EVERY CANDIDATE IS ADMITTED BY MEASUREMENT, NEVER BY PLAUSIBILITY, and the first cut of
    this loop had to be rewritten because it was not. A PEG parser speculates a declaration
    reading on ordinary tokens, so the harvest returns real names AND noise: on
    `cip_lc_tx_cov_if.sv` it returned the interface's own PORT `rst_ni`, declaring it shadowed
    the port, the parse went BACKWARDS, and the row was reported RESIDUAL — a candidate parser
    defect — for a file that parses perfectly once its two missing packages are declared. So a
    name joins the prelude only if adding it does not REDUCE how far the parse reaches."""
    names = set(seed_names)
    furthest = None
    for rnd in range(1, MAX_ROUNDS + 1):
        text = compose(raw, wrapper, names, forms)
        ok, surface, furthest, _ = probe_text(text, trace=False)
        if ok:
            return True, names, rnd, furthest
        missing = demanded_missing(text, reserved)
        if not missing:
            break
        stripped = blank_comments_and_strings(text)
        stuck = furthest if furthest is not None else surface
        region = failure_region_names(stripped, stuck) if stuck is not None else set()
        declared = declared_type_names(stripped)
        fresh = (missing & region) - names - declared
        if not fresh:
            # ⚠️ WIDE fallback: nothing in the failure region, so ask whether ANY
            # demanded-and-missing name explains it. Noisier, and admission prunes it.
            fresh = missing - names - declared
            if not fresh:
                break
        base = reach(raw, wrapper, names, forms)
        admitted = {n for n in fresh if reach(raw, wrapper, names | {n}, forms) >= base}
        if not admitted:
            break
        names |= admitted
    return False, names, MAX_ROUNDS, furthest


def minimize(raw: str, wrapper, names, forms) -> tuple:
    """Smallest (name set, form set) that still parses. ⛔ Without this a bogus prelude could
    excuse a real defect -- the one direction this leaf must not fail in."""
    keep = set(names)
    for n in sorted(names):
        trial = keep - {n}
        ok, _, _, _ = probe_text(compose(raw, wrapper, trial, forms), trace=False)
        if ok:
            keep = trial
    for cand in FORM_LADDER:
        if len(cand) >= len(forms):
            break
        ok, _, _, _ = probe_text(compose(raw, wrapper, keep, cand), trace=False)
        if ok:
            return keep, cand
    return keep, forms


def macro_block_reason(raw: str, wrapper, names, forms) -> str:
    """Is the RESIDUAL failure sitting ON an undefined macro invocation? ⛔ Read off the parse,
    never assumed: the choking token is `stuck_offset_of(furthest)` and the macro name must be
    neither an SVPP pass-through directive nor `define`d in this very file."""
    text = compose(raw, wrapper, names, forms)
    ok, surface, furthest, _ = probe_text(text, trace=False)
    if ok:
        return ""
    stripped = blank_comments_and_strings(text)
    stuck = stuck_offset_of(stripped, furthest if furthest is not None else surface)
    if stuck is None or stuck >= len(stripped) or stripped[stuck] != "`":
        return ""
    m = TICK_NAME_RE.match(stripped, stuck)
    if not m:
        return ""
    name = m.group(1)
    if name in SVPP_PASSTHROUGH or f"`define {name}" in stripped:
        return ""
    return name


def minimize_macro(raw: str, wrapper, names, forms) -> tuple:
    """Same discipline as `minimize`, for the MACRO-BLOCKED verdict: keep only the declarations
    whose removal costs the row its macro block. ⛔ Otherwise an over-wide prelude could carry a
    parse PAST a genuine defect and land it on a macro, which reads as an excuse."""
    keep, macro = set(names), macro_block_reason(raw, wrapper, names, forms)
    for n in sorted(names):
        trial = keep - {n}
        if macro_block_reason(raw, wrapper, trial, forms) == macro:
            keep = trial
    for cand in FORM_LADDER:
        if len(cand) >= len(forms):
            break
        if macro_block_reason(raw, wrapper, keep, cand) == macro:
            return keep, cand, macro
    return keep, forms, macro


def adjudicate(row):
    """⛔ The wrapper ladder is ordered so a row is never given a story it does not need:
    `none` first (a legal standalone unit needs no wrapper), and the declaration form is
    MINIMIZED after the fact rather than chosen permissively up front."""
    reserved = sv_reserved_words(GRAMMAR)
    path = resolve_path(row["suite"], row["relpath"])
    raw = path.read_bytes().decode("latin-1")
    widest = FORM_LADDER[-1]
    seed, furthest, harvested = frozenset(), None, {}
    for wrapper in WRAPPERS:
        ok, names, rounds, f = drive(raw, wrapper, widest, reserved, seed)
        harvested[wrapper[0]] = frozenset(names)
        if wrapper[0] == "none":
            furthest, seed = f, frozenset(names)
        if not ok:
            continue
        minimal, min_forms = minimize(raw, wrapper, names, widest)
        verdict = ("CROSS-FILE-FACTS" if wrapper[0] == "none"
                   else f"FRAGMENT-{wrapper[0].upper()}")
        if wrapper[0] == "none" and not minimal:
            verdict = "PARSES-BARE"              # would contradict the corpus row; flagged
        return {"verdict": verdict, "wrapper": wrapper[0], "form": "+".join(min_forms),
                "decl_count": len(minimal), "declarations": ",".join(sorted(minimal)),
                "rounds": rounds, "residual_furthest": "", "macro": ""}

    for wrapper in WRAPPERS:
        if not macro_block_reason(raw, wrapper, harvested[wrapper[0]], widest):
            continue
        minimal, min_forms, macro = minimize_macro(raw, wrapper, harvested[wrapper[0]], widest)
        return {"verdict": "MACRO-BLOCKED", "wrapper": wrapper[0],
                "form": "+".join(min_forms), "decl_count": len(minimal),
                "declarations": ",".join(sorted(minimal)), "rounds": MAX_ROUNDS,
                "residual_furthest": str(furthest or ""), "macro": macro}
    return {"verdict": "RESIDUAL", "wrapper": "", "form": "+".join(widest),
            "decl_count": len(seed), "declarations": ",".join(sorted(seed)),
            "rounds": MAX_ROUNDS, "residual_furthest": str(furthest or ""), "macro": ""}


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--jobs", type=int, default=6)
    ap.add_argument("--only", type=int, default=0, help="1-based worklist row, for one-row runs")
    args = ap.parse_args()

    if not PROBE.exists():
        raise SystemExit(f"⛔ REFUSING: {PROBE} is missing. Build it first:\n"
                         "   (cd rust && cargo build --release --features generated_parsers "
                         "--bin parseability_probe)")
    (ROOT / "tmp").mkdir(exist_ok=True)

    with WORKLIST.open(encoding="utf-8") as fh:
        rows = list(csv.DictReader(fh, delimiter="\t"))
    if args.only:
        rows = [rows[args.only - 1]]

    with ThreadPoolExecutor(max_workers=args.jobs) as ex:
        results = list(ex.map(adjudicate, rows))

    # ⛔ THE STRUCTURAL RESIDUE IS ADJUDICATED PER ROW, AND THE MAPPING IS TWO-SIDED.
    # A RESIDUAL row with no entry in RESIDUAL_ROWS.tsv is a HARD ERROR (a new unexplained row
    # must not slip through as "just residue"), and an entry for a row that is no longer
    # RESIDUAL is a hard error too (the adjudication has been overtaken and must be re-read).
    with RESIDUAL_ROWS.open(encoding="utf-8") as fh:
        residual = {(r["suite"], r["relpath"]): r for r in csv.DictReader(fh, delimiter="\t")}
    got = {(row["suite"], row["relpath"]) for row, res in zip(rows, results)
           if res["verdict"] == "RESIDUAL"}
    if got != set(residual):
        for k in sorted(got - set(residual)):
            print(f"⛔ RESIDUAL with no adjudication in {RESIDUAL_ROWS.relative_to(ROOT)}: {k}")
        for k in sorted(set(residual) - got):
            print(f"⛔ adjudicated in {RESIDUAL_ROWS.relative_to(ROOT)} but no longer "
                  f"RESIDUAL — re-read it: {k}")
        return 1

    out = []
    for row, res in zip(rows, results):
        adj = residual.get((row["suite"], row["relpath"]), {})
        res["final"] = adj.get("final_verdict", res["verdict"])
        res["repro"] = adj.get("repro", "")
        res["note"] = adj.get("note", "")
        out.append({"verdict": res["verdict"], "channel": row["channel"], "suite": row["suite"],
                    "relpath": row["relpath"], "fail_line": row["fail_line"],
                    "final_verdict": res["final"], "repro": res["repro"],
                    "note": res["note"],
                    "wrapper": res["wrapper"], "form": res["form"],
                    "decl_count": res["decl_count"], "declarations": res["declarations"],
                    "macro": res["macro"], "rounds": res["rounds"],
                    "residual_furthest": res["residual_furthest"], "snippet": row["snippet"]})
    out.sort(key=lambda r: (r["final_verdict"], r["suite"], r["relpath"]))

    cols = ["final_verdict", "repro", "note", "verdict", "channel", "suite", "relpath",
            "fail_line", "wrapper", "form", "decl_count", "declarations", "macro", "rounds",
            "residual_furthest", "snippet"]
    OUT_TSV.parent.mkdir(parents=True, exist_ok=True)
    with OUT_TSV.open("w", encoding="utf-8", newline="") as fh:
        w = csv.DictWriter(fh, fieldnames=cols, delimiter="\t", lineterminator="\n")
        w.writeheader()
        w.writerows(out)

    tally = {}
    for r in out:
        tally[r["final_verdict"]] = tally.get(r["final_verdict"], 0) + 1

    L = ["# The 57-row candidate worklist, adjudicated by PARSING (SV-CORPUS-GRAD.13c.2)", "",
         "> Generated by `stimuli/sv/adjudicate_dark_worklist.py`. ⛔ No row is relabelled: every",
         "> verdict is the outcome of a real parse of a NAMED transformation of the row's own bytes.",
         "", "## Instrument identity", "",
         "| input | repo-root-relative path | sha256 |", "|---|---|---|",
         f"| worklist | `{WORKLIST.relative_to(ROOT)}` | `{sha256(WORKLIST)}` |",
         f"| grammar | `{GRAMMAR.relative_to(ROOT)}` | `{sha256(GRAMMAR)}` |",
         f"| residual adjudication | `{RESIDUAL_ROWS.relative_to(ROOT)}` | "
         f"`{sha256(RESIDUAL_ROWS)}` |", "",
         "## Verdicts", "", "| verdict | rows | what it means |", "|---|---:|---|"]
    meaning = {
        "DEFECT": "⭐ valid SV that PGEN REJECTS — pinned by a minimal reproducer under "
                  "`stimuli/sv/adjudication_repros/`, re-run by "
                  "`stimuli/sv/run_adjudication_repros.py`",
        "INVALID-SV": "the text is NOT legal SV; the rejection is CORRECT and accepting it "
                      "would be an over-acceptance defect (pinned as a negative reproducer)",
        "NOT-SV-SOURCE": "not source text at all (a command line, a plain-text fixture) — a "
                         "CORPUS COMPOSITION defect, routed to `.13c.1`",
        "FRAGMENT-ENUM-BODY": "an enum-member list included INSIDE an `enum { … }` — no "
                              "compilation-unit wrapper can shape it",
        "CROSS-FILE-FACTS": "declaring the cross-file names the parser demanded makes the file "
                            "PARSE ⇒ blocked by cross-file FACT continuity (`.4`), NOT a parser "
                            "defect",
        "MACRO-BLOCKED": "the RESIDUAL failure sits on an undefined macro invocation ⇒ expansion "
                         "owns what is left (`.13d`), NOT a parser defect",
        "RESIDUAL": "no transformation makes it parse ⇒ escalated to a per-row structural "
                    "diagnosis (minimal reproducer + `--trace-rules`)",
        "PARSES-BARE": "⚠️ parses with NO transformation at all — contradicts the corpus row",
    }
    for v, n in sorted(tally.items(), key=lambda kv: (-kv[1], kv[0])):
        m = meaning.get(v)
        if m is None and v.startswith("FRAGMENT-"):
            m = (f"parses only inside a `{v.split('-', 1)[1].lower()}` ⇒ an include payload, not a "
                 "legal standalone compilation unit")
        L += [f"| `{v}` | {n} | {m or ''} |"]
    L += [f"| **TOTAL** | **{len(out)}** | |", "", "## Per row", "",
          "⛔ `wrapper`/`form`/`declarations` together ARE the transformation that was parsed —",
          "re-run it and you get the same verdict, or the row is wrong. A `DEFECT` /",
          "`INVALID-SV` row names the minimal reproducer that pins its claim.", "",
          "| verdict | suite | file | line | wrapper | what unblocks it / repro |",
          "|---|---|---|---:|---|---|"]
    for r in out:
        what = f"`{r['declarations']}`" if r["declarations"] else ""
        if r["macro"]:
            what = f"stuck on `` `{r['macro']} `` (undefined here)"
        if r["repro"]:
            what = f"`{r['repro']}`"
        elif r["note"]:
            what = r["note"]
        L += [f"| `{r['final_verdict']}` | {r['suite']} | `{r['relpath']}` | {r['fail_line']} | "
              f"{r['wrapper'] or '—'} | {what} |"]
    OUT_MD.write_text("\n".join(L) + "\n", encoding="utf-8")

    for v, n in sorted(tally.items(), key=lambda kv: (-kv[1], kv[0])):
        print(f"{v:22s} {n:3d}")
    print(f"{'TOTAL':22s} {len(out):3d}")
    print(f"wrote {OUT_TSV.relative_to(ROOT)} and {OUT_MD.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
