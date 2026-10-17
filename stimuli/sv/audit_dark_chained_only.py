#!/usr/bin/env python3
"""Which DARK `deferred:chained_only` rows can chaining NOT explain? (SV-CORPUS-GRAD.13c)

WHY THIS EXISTS
---------------
`SV-CORPUS-GRAD.13a` measured the denominator: of 16 336 corpus rows, 4 398 (26.9 %) FAIL
a parse inside a deferral nobody has looked at, and 4 158 of those carry the label
`deferred:chained_only` -- *"this file only parses honestly as part of a multi-file unit"*.

That label is read downstream as the much stronger claim *the parse fails BECAUSE the other
files are missing*.  `.12` established both the asymmetry and the method one bucket up: a
wrong `deferred` verdict SHIPS a parser defect, because nothing looks at the row again.

THE SOUND FALSIFICATION (reused, not re-derived)
-----------------------------------------------
A preprocessor's output is byte-identical to its input up to the first token it can ALTER.
`define lines are deleted, macros expand, `ifdef arms vanish, `include inlines -- all of it
at or after that offset, never before it.  So:

    stuck_offset < first_alterable_tick   (or: the file has no alterable tick at all)
        =>  chaining CANNOT change one byte of the text the parse actually choked on
        =>  the parse fails identically on the fully chained text
        =>  the TEXTUAL half of the `chained_only` deferral is FALSE for that row.

⛔ THE LAYOUT GAP IS RESOLVED FIRST, and this instrument does not re-derive it: it imports
`stuck_offset_of` / `blank_comments_and_strings` / `tick_offsets` from the `.12` audit, whose
own first cut "proved" 504 misclassifications that were entirely a 6-byte whitespace gap
([[a-furthest-position-names-a-region-not-a-token]]).

⛔ AND A REFUTATION NAMES WHAT IS *NOT* THE CAUSE, NEVER WHAT IS
---------------------------------------------------------------
`.13a` probed three text-refuted rows with the real toolbox and all three died on a
user-defined TYPE NAME a sibling file declares:

    wire csrng_req_t   cmd_req;
    🚫 Rule 'checked_type_identifier' rejected by post predicate 'has_fact [type_name, …]'

PGEN's SV parser is fact-gated, so a `typedef` in another file supplies a fact a predicate
requires.  That is a genuine multi-file dependency -- but it is a **cross-file FACT**
dependency, not a text one, and the two are satisfied by different capabilities (parsing a
file list into one fact store, versus running a preprocessor).  So each text-refuted row is
then sorted by whether that second channel can plausibly explain it:

  FACT-GATED-CROSS-FILE  the PARSER demanded a `type_name` fact it did not have, for a name
                       this file does not declare -- a sibling file's `typedef` is the live
                       dependency.  ⚠️ The parser's own testimony that the channel is ACTIVE;
                       whether it is the whole cause is settled by a chained parse (`.13d`).
  ⭐ FACT-GATED-LOCAL-TYPE  every demanded name IS declared in THIS file and the parser still
                       lacked the fact.  No chaining could help ⇒ CANDIDATE DEFECT.
  ⭐ NO-FACT-GATE-AT-FAILURE  neither channel is live: nothing to expand, and no
                       name-shaped dependency at the failure.  ⛔ This is a WORKLIST bucket,
                       NOT a defect count -- the text simply does not explain the failure, and
                       only a probe + `--trace-rules` per row can (`NO cut heuristic is a
                       census`).

⛔ THE TWO CROSS-FILE BUCKETS ARE A SIGNAL, NOT A PROOF, and are counted separately for
exactly that reason.  The two ⭐ buckets are the WORKLIST this leaf exists to produce; each
row on it must still be adjudicated with the toolbox (probe + `--trace-rules`) before it is
called a defect.  ⛔ NO row is relabelled by this instrument -- a row earns a verdict by being
PARSED (`SV-CORPUS-GRAD.13`).

USAGE
    python3 stimuli/sv/audit_dark_chained_only.py

Outputs (repo-root-relative, deterministic -- rows sorted by (verdict, suite, relpath)):
    docs/tasks/artifacts/sv_corpus_grad/dark_chained_only/audit.tsv
    docs/tasks/artifacts/sv_corpus_grad/dark_chained_only/summary.md
    docs/tasks/artifacts/sv_corpus_grad/dark_chained_only/worklist.tsv   (the ⭐ buckets)

    python3 stimuli/sv/audit_dark_chained_only.py --self-test   (the unit rule and the fact channel on literal rows)

⚠️ Pass 2 runs one TRACED parse per text-refuted row (~1-2 s each, 8-way parallel by default).
"""

import argparse
import hashlib
import re
import shlex
import subprocess
import sys
from collections import Counter
from concurrent.futures import ThreadPoolExecutor
from functools import lru_cache
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent

# ⛔ SINGLE SOURCE OF TRUTH for the svpp reach model AND for the layout-gap resolution: both
# are owned by the `.12` audit / the adjudicator, and are imported rather than restated.
sys.path.insert(0, str(Path(__file__).resolve().parent))
from audit_explained_svpp import (                      # noqa: E402
    blank_comments_and_strings,
    line_of,
    read_raw,
    resolve_path,
    snippet_at,
    stuck_offset_of,
    tick_offsets,
)

TEXT_REFUTED = {"NO-ALTERABLE-TICK", "PRE-TICK"}

IDENT_RE = re.compile(r"[A-Za-z_][A-Za-z0-9_$]*")
# A token is either an identifier-ish run or a single non-space character.
TOKEN_RE = re.compile(r"[A-Za-z_][A-Za-z0-9_$]*|::|[^\sA-Za-z0-9_$]")


def sv_reserved_words(grammar: Path) -> frozenset:
    """The parser's OWN reserved-word list, read out of the grammar rather than retyped.
    `reserved_non_keyword_identifier_sv` is the regex the parser uses to keep keywords out of
    identifier positions, so it is exactly the right set for "is this token a name?"."""
    text = grammar.read_text(encoding="utf-8")
    m = re.search(r"reserved_non_keyword_identifier_sv\s*:=\s*trivia\s*/\(\?:([^)]*)\)", text)
    if not m:
        raise SystemExit(
            "⛔ REFUSING: could not read `reserved_non_keyword_identifier_sv` out of "
            f"{grammar}. This instrument will not fall back to a hand-typed keyword list — "
            "a stale copy would silently mis-sort identifiers into keywords.")
    return frozenset(m.group(1).split("|"))


# ⛔ A file that is not SOURCE TEXT cannot testify about a parser. Detected, counted and
# routed rather than filed under "unexplained": a `$readmemh` memory image carrying a `.v`
# extension is a CORPUS COMPOSITION defect, and calling it a parser candidate would waste a
# real investigation (SV-CORPUS-GRAD.13c, measured: 346 rows, every one friscv).
HEXDATA_LINE_RE = re.compile(r"^(?:@[0-9A-Fa-f]+|(?:[0-9A-Fa-f]{2}[ \t]*)+)$")


def looks_like_memory_image(raw: str) -> bool:
    """>=80 % of the first 200 non-blank lines are `@address` / hex-byte rows. Deliberately
    strict on shape and generous on length: RTL never looks like this, and a 3-line file is
    too short to judge."""
    lines = [l.strip() for l in raw.splitlines() if l.strip()][:200]
    if len(lines) < 3:
        return False
    hexish = sum(1 for l in lines if HEXDATA_LINE_RE.match(l))
    return hexish / len(lines) >= 0.8


# ⛔⛔ ONE DECLARATION READER, USED BY BOTH THE IN-FILE TEST AND THE CORPUS INDEX — and the
# regex forms it replaces were each MEASURED WRONG, in opposite directions:
#   * `\btypedef\b[^;]*\b{name}\b` (the in-file test) matched a type USED INSIDE a typedef's
#     body — in `typedef struct packed { sram_key_t key; … } scrmbl_key_init_t;` it called
#     `sram_key_t` locally declared, moving the row into the CANDIDATE-DEFECT bucket. Toolbox-
#     verified: the parser demands `has_fact [type_name, sram_key_t]` there and the name is
#     declared in ANOTHER package.
#   * `\btypedef\b[^;]{0,300}?\b(name)\s*;` (the corpus index) can never reach the name of a
#     STRUCT typedef at all, because every member ends in `;` — so 276 opentitan rows whose types
#     are ordinary `typedef struct`/`typedef enum` names were reported as declared NOWHERE.
# A typedef's declared name is the last identifier before its terminating `;` at brace depth 0,
# so scan for that instead of pattern-matching around it.
TYPEDEF_RE = re.compile(r"\btypedef\b")
NAME_RE = re.compile(r"[A-Za-z_]\w*")
DECL_KEYWORD_RE = re.compile(r"\b(?:class|package)\s+([A-Za-z_]\w*)")
TYPE_PARAM_RE = re.compile(r"\btype\s+([A-Za-z_]\w*)")
NETTYPE_RE = re.compile(r"\bnettype\b[^;]*?\b([A-Za-z_]\w*)\s*;")


def declared_type_names(text: str, scan_limit: int = 8000) -> set:
    """Every name this text gives a TYPE meaning: `typedef … name;` (brace-aware),
    `class`/`package` declarations, `#(type T = …)` type parameters, `nettype … name;`.

    ⚠️ Deliberately over-inclusive on the class/type-parameter arms: a false entry moves a row
    OUT of the defect worklist, which is the conservative direction for a worklist this leaf
    must not inflate."""
    names = {m.group(1) for m in DECL_KEYWORD_RE.finditer(text)}
    names |= {m.group(1) for m in TYPE_PARAM_RE.finditer(text)}
    names |= {m.group(1) for m in NETTYPE_RE.finditer(text)}
    for m in TYPEDEF_RE.finditer(text):
        i, depth, stop = m.end(), 0, None
        limit = min(len(text), m.end() + scan_limit)
        while i < limit:
            c = text[i]
            if c in "{([":
                depth += 1
            elif c in "})]":
                depth -= 1
            elif c == ";" and depth <= 0:
                stop = i
                break
            i += 1
        if stop is None:
            continue
        ids = NAME_RE.findall(text[m.end():stop])
        if ids:
            names.add(ids[-1])
    return names


def declared_in_file(stripped: str, name: str) -> bool:
    """Does THIS file give `name` a type meaning? If it does, no chaining is needed for the
    parser to know it, so a missing `type_name` fact for it is a CANDIDATE DEFECT rather than a
    cross-file dependency."""
    return name in declared_type_names(stripped)


# ⛔⛔ THE SECOND QUESTION IS ANSWERED BY THE PARSER, NOT BY A REGEX OVER THE SOURCE.
#
# The first cut of this instrument classified the fact channel from the source text — which
# identifier sits where, relative to the failure. It was rewritten because it could not be
# calibrated: three successive honest refinements moved the residue bucket 1 902 → 1 933 → 53
# rows, i.e. by more than the whole finding, each time for a defensible reason. A number that
# swings by 1 900 on a one-line edit is a heuristic, and ⛔ *no cut heuristic is a census*.
#
# The parser already answers the question authoritatively and cheaply: at
# `PGEN_TRACE_VERBOSITY=high` every `@predicate` evaluation prints its verdict, so a demanded
# `type_name` fact that was missing is stated in the parser's own words (TOOLBOX §2.4).
# Measured: ~200 MB of trace in ~1 s for a 4.7 KB file, ~400 MB in ~2 s for 79 KB — streamed
# through a grep, never stored.
#
# ⚠️ HONEST NOTE, from TOOLBOX §2.1: enabling a trace routes the parse onto the PROTOCOL graph
# instead of the fused `cascade_*` graph. The two are held byte-identical by the equivalence
# and AST oracles, so the verdict is the same run to run — but the trace we read is not
# literally the untraced run, and saying so is part of the evidence.
# ⛔⛔ THE VOCABULARY IS ENUMERATED FROM A REAL TRACE, NOT GUESSED — and getting it wrong was
# MEASURED, not imagined. The first cut matched only `has_fact`, and the dominant UVM shape
# (`class X extends BASE;`) is gated by `fact_attribute_equals [type_name, BASE,
# declaration_family, class]`. That one missing arm put **1 203 rows** — 63 % of the worklist —
# into the candidate-defect bucket while the parser's own trace named the cross-file base class
# on the line above. A big number in the alarming direction is the tell, not the finding.
FACT_MISS_RE = re.compile(
    r'has_fact\(kind=type_name, name=Identifier\("([^"]+)"\)\) → false'
    r'|fact_attribute_equals\(kind=type_name, name=Identifier\("([^"]+)"\)[^\n]*→ false'
    r'|rejected by post predicate .has_fact \[Identifier\("type_name"\), '
    r'Identifier\("([^"]+)"\)\]'
    r'|rejected by post predicate .fact_attribute_equals \[Identifier\("type_name"\), '
    r'Identifier\("([^"]+)"\)')


def probe_fact_gates(probe_bin: Path, path: Path, timeout_s: int):
    """Run the real parse under a predicate trace and return the set of identifiers for which
    a `type_name` fact was DEMANDED AND MISSING, or None on timeout / probe failure."""
    cmd = (f'PGEN_TRACE_VERBOSITY=high {shlex.quote(str(probe_bin))} --parse systemverilog '
           f'{shlex.quote(str(path))} --profile sv_2017 --trace 2>&1 '
           f'| grep -aE "has_fact|fact_attribute_equals" | head -8000')
    try:
        proc = subprocess.run(["/bin/sh", "-c", cmd], capture_output=True, text=True,
                              timeout=timeout_s)
    except subprocess.TimeoutExpired:
        return None
    names = set()
    for m in FACT_MISS_RE.finditer(proc.stdout):
        names.add(next(g for g in m.groups() if g))
    return names


def region_bounds(stripped: str, stuck: int, back: int = 300) -> int:
    """Start of the syntactic region the failure sits in: the byte after the nearest
    statement/port separator behind it."""
    lo = max(0, stuck - back)
    seps = [stripped.rfind(ch, lo, stuck) for ch in (";", ",", "(", ")", "{", "}")]
    return max(seps) + 1 if max(seps) >= 0 else lo


def failure_region_names(stripped: str, stuck: int) -> set:
    """Identifiers in the region the parse actually died in."""
    if stuck is None:
        return set()
    start = region_bounds(stripped, stuck)
    end = min(len(stripped), stuck + 80)
    return {m.group(0) for m in IDENT_RE.finditer(stripped, start, end)}


# ⛔⛔ THE THIRD LEG: "a sibling file declares it" IS A CHECKABLE CLAIM, SO CHECK IT.
# The failure-region intersection still admits SPECULATION: the parser tries a type reading on
# ordinary tokens, so `static task host();` produced demanded-and-missing names `static` and
# `host` inside the failure region and the row was excused as cross-file. Measured, and wrong —
# nothing anywhere declares `host` as a type. The cross-file channel claims a NAME another file
# declares, so the index below decides it: 11 359 corpus files, 7 301 declared type names,
# 1.5 s. `mem_model_base_test` resolves to `dccm_base_test.sv`; `host` and `static` resolve to
# nothing, and their row stays a candidate.
INDEX_EXTS = {".sv", ".svh", ".v", ".vh"}


def build_declaration_index(suites) -> dict:
    """suite -> {declared type/class/package name: the file that declares it}.

    ⚠️ Deliberately over-inclusive (the `typedef` arm's tail can catch an unrelated trailing
    identifier, and commented-out code is not excluded): a false entry moves a row OUT of the
    defect worklist, which is the conservative direction for a worklist this leaf must not
    inflate. The names it matters for are long project-specific type names, where the noise
    (`q`, `d`, `valid`) cannot collide."""
    index = {}
    for suite in sorted(suites):
        root = (ROOT / "stimuli/sv/uvm" if suite == "uvm-core"
                else ROOT / "stimuli/sv/subs" / suite)
        names = {}
        for path in root.rglob("*"):
            if not path.is_file() or path.suffix.lower() not in INDEX_EXTS:
                continue
            try:
                text = path.read_bytes().decode("latin-1")
            except OSError:
                continue
            for n in declared_type_names(text):
                names.setdefault(n, str(path.relative_to(ROOT)))
        index[suite] = names
    return index


SURELOG_TESTS = ROOT / "stimuli/sv/subs/Surelog/tests"


def unit_dir_of(suite: str, path: Path, tests_root: Path = SURELOG_TESTS):
    """The row's COMPILATION UNIT, where the answer key defines one this audit can read off the tree
    (SV-CORPUS-GRAD.57c). Surelog: *a test unit = a directory under tests/ directly containing >= 1
    `.sl` driver* (`SurelogIndex`, adjudicate_external_corpus.py) — the nearest such directory above
    the file; a file with no driver directory above it has no unit here. Every other suite's unit is
    the library design's (`SVPP-EXPANSION.6d`, the members dump), not read here: None."""
    if suite != "Surelog":
        return None
    d = path.parent
    while tests_root in d.parents:
        if any(x.suffix == ".sl" for x in d.iterdir()):
            return d
        d = d.parent
    return None


@lru_cache(maxsize=None)
def unit_declared_map(unit_dir: Path) -> dict:
    """{declared type/class/package name: the unit file that declares it} over EVERY source file of
    the unit's directory (the same over-inclusive reader as the suite index, the same conservative
    direction: a false in-unit entry keeps a row OUT of the out-of-unit stratum)."""
    names = {}
    for f in sorted(unit_dir.rglob("*")):
        if not f.is_file() or f.suffix.lower() not in INDEX_EXTS:
            continue
        try:
            text = f.read_bytes().decode("latin-1")
        except OSError:
            continue
        for n in declared_type_names(text):
            names.setdefault(n, str(f.relative_to(ROOT)) if f.is_relative_to(ROOT) else str(f))
    return names


def classify_fact_channel(stripped: str, stuck: int, names, declared_elsewhere=None, unit_names=None) -> tuple:
    """Sort the row by what the PARSER demanded AT THE FAILURE, not by what the text looks
    like anywhere.

    ⛔⛔ THE INTERSECTION WITH THE FAILURE REGION IS LOAD-BEARING, AND ITS ABSENCE WAS
    MEASURED. A PEG parser SPECULATES: it tries `checked_type_identifier` on ordinary
    identifiers all the time, so `has_fact(kind=type_name, name="clk") → false` appears in
    almost every trace. Classifying on "the parse demanded SOME missing type fact" would
    therefore excuse nearly every row as cross-file — a classifier that fails in the
    REASSURING direction, which is the one that ships defects. Only a demanded-and-missing
    name that appears WHERE THE PARSE DIED is evidence about this failure."""
    if names is None:
        return ("PROBE-TIMEOUT", "")
    local = sorted(names & failure_region_names(stripped, stuck))
    if not local:
        # The parser never demanded a missing `type_name` fact for anything at the failure,
        # so no sibling file's `typedef` explains it, and no text of it can be expanded.
        return ("NO-FACT-GATE-AT-FAILURE", "")
    for n in local:
        if declared_in_file(stripped, n):
            # ⭐ This file declares it and the parser still lacked the fact.
            return ("FACT-GATED-LOCAL-TYPE", n)
    own, other = declared_elsewhere or ({}, {})
    in_suite = [n for n in local if n in own]
    if in_suite:
        if unit_names is None:
            return ("FACT-GATED-CROSS-FILE", f"{in_suite[0]} @ {own[in_suite[0]]}")
        # SV-CORPUS-GRAD.57c — the suite is not the unit. A name a file of the row's OWN unit declares is
        # the `.6d` declaration barrier (chaining would supply it; the in-unit file is named, and it wins
        # over an out-of-unit one — the conservative direction). A name declared in the suite but in NO
        # file of the row's unit cannot reach the row's compile: the unit's own text is all the parser can
        # ever see, so the sound store gate's refusal is provable on the raw text — a WORKLIST bucket.
        in_unit = [n for n in in_suite if n in unit_names]
        if in_unit:
            return ("FACT-GATED-CROSS-FILE", f"{in_unit[0]} @ {unit_names[in_unit[0]]}")
        return ("FACT-GATED-OUT-OF-UNIT", f"{in_suite[0]} @ {own[in_suite[0]]}")
    # ⚠️ A THIRD, HONEST ANSWER. A test can depend on a library that is vendored in ANOTHER
    # suite (Surelog's tests use UVM, which lives under `stimuli/sv/uvm`). Then the name is
    # neither a parser defect nor in-suite chaining: the compilation unit is incomplete as
    # the corpus holds it. Measured while calibrating — those rows were sitting in
    # UNDECLARED-ANYWHERE and would have been read as parser candidates.
    for n in local:
        if n in other:
            return ("CROSS-LIBRARY", f"{n} @ {other[n]}")
    # A fact was demanded at the failure and NOTHING in the suite declares that name as a
    # type — so chaining cannot supply it either. ⭐ CANDIDATE DEFECT.
    return ("UNDECLARED-ANYWHERE", ",".join(local[:3]))


def read_manifest(path: Path):
    rows = []
    with path.open(encoding="utf-8") as fh:
        header = fh.readline().rstrip("\n").split("\t")
        idx = {n: header.index(n) for n in
               ("suite", "relpath", "observed", "adjudication", "basis")}
        for line in fh:
            if not line.strip():
                continue
            c = line.rstrip("\n").split("\t")
            row = {k: c[i] for k, i in idx.items()}
            if row["adjudication"] == "deferred:chained_only" and row["observed"] == "fail":
                rows.append(row)
    return rows


def read_positions(path: Path):
    out = {}
    with path.open(encoding="utf-8") as fh:
        for line in fh:
            if not line.strip():
                continue
            _suite, full, pos = line.rstrip("\n").split("\t")
            out[full] = int(pos)
    return out


def md_code_cell(text: str) -> str:
    """A source excerpt as ONE GFM table cell that renders as code whatever the excerpt holds.

    ⛔ Two measured ways a raw excerpt breaks a table (TABLE-ARITY-RATCHET, both arms): a `|`
    inside the excerpt (`~|`, `||`) is a CELL SEPARATOR unless escaped, so the row gains cells
    and the renderer silently discards the rightmost ones; and a backtick inside a single-backtick
    span (`` `uvm_object_utils ``) leaves the span UNPAIRED, so the rest of the cell renders as
    code-or-prose swapped. The `.13c.1` re-run moved four new rows into the first-40 view and two
    of them carried backticks — 9 -> 11 unpaired cells, and the ratchet refused. So: escape every
    pipe, use a delimiter run one longer than the longest backtick run in the text, and pad with a
    space when the text starts or ends with a backtick (CommonMark strips one such space)."""
    text = text.replace("|", "\\|")
    longest = max((len(m) for m in re.findall(r"`+", text)), default=0)
    fence = "`" * (longest + 1)
    if text.startswith("`") or text.endswith("`") or not text:
        text = f" {text} "
    return f"{fence}{text}{fence}"


def self_test() -> int:
    """SV-CORPUS-GRAD.57c — the unit rule and the fact channel on literal rows, no probe, no corpus."""
    import tempfile
    checks = []
    with tempfile.TemporaryDirectory(dir=str(ROOT / "rust/target")) as td:   # DATA-LOCALITY: scratch stays on the repo volume
        tests = Path(td) / "tests"
        (tests / "A" / "sub").mkdir(parents=True)
        (tests / "A" / "A.sl").write_text("-parse g.sv sub/f.sv\n")
        (tests / "A" / "g.sv").write_text("class K; endclass\n")
        (tests / "A" / "sub" / "f.sv").write_text("module m; K k; endmodule\n")
        (tests / "B").mkdir()
        (tests / "B" / "B.sl").write_text("-parse dut.sv\n")
        (tests / "B" / "dut.sv").write_text("class S #(type REQ = int, type RSP = REQ); endclass\n")
        (tests / "C").mkdir()
        (tests / "C" / "x.sv").write_text("module x; endmodule\n")
        u = unit_dir_of("Surelog", tests / "A" / "sub" / "f.sv", tests)
        checks.append(("a Surelog file's unit is the nearest directory above it holding a `.sl` driver", u == tests / "A"))
        checks.append(("a file with no driver directory above it has no unit", unit_dir_of("Surelog", tests / "C" / "x.sv", tests) is None))
        checks.append(("a non-Surelog suite has no unit notion here (the library design's)", unit_dir_of("opentitan", tests / "A" / "g.sv", tests) is None))
        umap = unit_declared_map(tests / "A")
        checks.append(("the unit map names the unit file declaring each type", set(umap) == {"K"} and umap["K"].endswith("g.sv")))
        stripped = "module m; REQ t; endmodule"
        stuck = stripped.index("REQ")
        own = {"REQ": "tests/B/dut.sv", "RSP": "tests/B/dut.sv"}
        checks.append(("declared in the suite, in no file of the row's unit → FACT-GATED-OUT-OF-UNIT, the suite file named",
                       classify_fact_channel(stripped, stuck, {"REQ"}, (own, {}), {}) == ("FACT-GATED-OUT-OF-UNIT", "REQ @ tests/B/dut.sv")))
        checks.append(("declared in a file of the row's own unit → FACT-GATED-CROSS-FILE, the unit file named",
                       classify_fact_channel(stripped, stuck, {"REQ"}, (own, {}), {"REQ": "tests/A/g.sv"}) == ("FACT-GATED-CROSS-FILE", "REQ @ tests/A/g.sv")))
        checks.append(("no unit notion → FACT-GATED-CROSS-FILE as before (the suite's first declaring file)",
                       classify_fact_channel(stripped, stuck, {"REQ"}, (own, {}), None) == ("FACT-GATED-CROSS-FILE", "REQ @ tests/B/dut.sv")))
        two = "module m; REQ a; RSP b; endmodule"
        checks.append(("two demanded names, one in the unit → the in-unit one wins (conservative: the .6d barrier)",
                       classify_fact_channel(two, two.index("REQ"), {"REQ", "RSP"}, (own, {}), {"RSP": "tests/A/g.sv"}) == ("FACT-GATED-CROSS-FILE", "RSP @ tests/A/g.sv")))
        checks.append(("declared only in another suite's vendored library → CROSS-LIBRARY, unchanged",
                       classify_fact_channel(stripped, stuck, {"REQ"}, ({}, {"REQ": "stimuli/sv/uvm/x.svh"}), {}) == ("CROSS-LIBRARY", "REQ @ stimuli/sv/uvm/x.svh")))
        checks.append(("declared nowhere → UNDECLARED-ANYWHERE, unchanged",
                       classify_fact_channel(stripped, stuck, {"REQ"}, ({}, {}), {}) == ("UNDECLARED-ANYWHERE", "REQ")))
        checks.append(("declared in this very file → FACT-GATED-LOCAL-TYPE before any unit reading",
                       classify_fact_channel("typedef int REQ; module m; REQ t; endmodule", 25, {"REQ"}, (own, {}), {}) == ("FACT-GATED-LOCAL-TYPE", "REQ")))
    failed = 0
    for label, good in checks:
        print(f"  {'PASS' if good else 'FAIL'} {label}")
        failed += 0 if good else 1
    print(f"DARK-CHAINED-AUDIT-SELF-TEST: checks={len(checks)} failed={failed}")
    return 7 if failed else 0


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--manifest",
                    default=ROOT / "stimuli/sv/characterization/adjudication_manifest.tsv",
                    type=Path)
    ap.add_argument("--positions",
                    default=ROOT / "stimuli/sv/characterization/positions.tsv", type=Path)
    ap.add_argument("--grammar", default=ROOT / "grammars/systemverilog.ebnf", type=Path)
    ap.add_argument("--probe-bin",
                    default=ROOT / "rust/target/release/parseability_probe", type=Path)
    ap.add_argument("--jobs", type=int, default=8,
                    help="parallel traced parses (corpus provenance uses 8)")
    ap.add_argument("--timeout", type=int, default=120,
                    help="per-file traced-parse timeout (a traced parse is ~1-2 s)")
    ap.add_argument("--outdir",
                    default=ROOT / "docs/tasks/artifacts/sv_corpus_grad/dark_chained_only",
                    type=Path)
    ap.add_argument("--self-test", action="store_true",
                    help="the unit rule and the fact channel on literal rows (no probe, no corpus)")
    args = ap.parse_args()

    if args.self_test:
        sys.exit(self_test())

    if not args.probe_bin.is_file():
        raise SystemExit(
            f"⛔ REFUSING: probe binary not found: {args.probe_bin}\n"
            "  (cd rust && cargo build --release --features generated_parsers "
            "--bin parseability_probe)")

    rows = read_manifest(args.manifest)
    positions = read_positions(args.positions)

    out = []
    for row in rows:
        path = resolve_path(row["suite"], row["relpath"])
        rel = str(path.relative_to(ROOT))
        raw = read_raw(path)
        stripped = blank_comments_and_strings(raw)
        first_tick, last_tick = tick_offsets(stripped)
        furthest = positions.get(rel)
        stuck = stuck_offset_of(stripped, furthest) if furthest is not None else None

        if looks_like_memory_image(raw):
            # Not source text at all — a `$readmemh` image wearing a `.v` extension. It can
            # never parse, and it is not evidence about the parser (routed, see summary).
            verdict = "NOT-SV-SOURCE"
        elif furthest is None:
            verdict = "NO-POSITION"
        elif first_tick is None:
            verdict = "NO-ALTERABLE-TICK"
        elif stuck is None:
            # consumed every byte and still failed — nothing left for expansion to move
            verdict = "PRE-TICK" if last_tick < len(stripped) else "IN-WINDOW"
        elif stuck < first_tick:
            verdict = "PRE-TICK"
        elif stuck > last_tick:
            verdict = "PAST-LAST-TICK"
        else:
            verdict = "IN-WINDOW"

        at = stuck if stuck is not None else furthest
        out.append({
            "verdict": verdict, "channel": "", "_path": path, "_stripped": stripped,
            "_stuck": stuck,
            "suite": row["suite"], "relpath": row["relpath"], "path": rel,
            "bytes": len(raw), "furthest": furthest if furthest is not None else "",
            "stuck": stuck if stuck is not None else "",
            "first_tick": first_tick if first_tick is not None else "",
            "fail_line": line_of(raw, at) if at is not None else "",
            "snippet": snippet_at(raw, at) if at is not None else "",
        })

    # --- pass 2: ask the PARSER, only for the rows whose text refutes the deferral ---
    pending = [r for r in out if r["verdict"] in TEXT_REFUTED]
    index_suites = {r["suite"] for r in pending}
    print(f"pass 2: {len(pending):,} text-refuted rows, {args.jobs}-way traced parses ...",
          file=sys.stderr)

    print(f"indexing declared type names across {len(index_suites)} suites ...",
          file=sys.stderr)
    decl_index = build_declaration_index(index_suites | {"uvm-core"})
    global_index = {}
    for suite in sorted(decl_index):
        for name, where in decl_index[suite].items():
            global_index.setdefault(name, where)

    def adjudicate(r):
        names = probe_fact_gates(args.probe_bin, r["_path"], args.timeout)
        unit = unit_dir_of(r["suite"], r["_path"])
        r["channel"], r["fact_names"] = classify_fact_channel(
            r["_stripped"], r["_stuck"], names,
            (decl_index.get(r["suite"], {}), global_index),
            unit_declared_map(unit) if unit is not None else None)
        return r

    with ThreadPoolExecutor(max_workers=args.jobs) as pool:
        list(pool.map(adjudicate, pending))
    for r in out:
        r.setdefault("fact_names", "")
        del r["_path"], r["_stripped"], r["_stuck"]

    out.sort(key=lambda r: (r["verdict"], r["channel"], r["suite"], r["relpath"]))
    args.outdir.mkdir(parents=True, exist_ok=True)

    cols = ["verdict", "channel", "suite", "relpath", "furthest", "stuck", "first_tick",
            "fail_line", "bytes", "fact_names", "snippet"]
    with (args.outdir / "audit.tsv").open("w", encoding="utf-8") as fh:
        fh.write("\t".join(cols) + "\n")
        for r in out:
            fh.write("\t".join(str(r[c]) for c in cols) + "\n")

    worklist = [r for r in out
                if r["channel"] in ("FACT-GATED-LOCAL-TYPE", "NO-FACT-GATE-AT-FAILURE",
                                    "UNDECLARED-ANYWHERE", "FACT-GATED-OUT-OF-UNIT")]
    with (args.outdir / "worklist.tsv").open("w", encoding="utf-8") as fh:
        fh.write("\t".join(cols) + "\n")
        for r in worklist:
            fh.write("\t".join(str(r[c]) for c in cols) + "\n")

    verdicts = Counter(r["verdict"] for r in out)
    channels = Counter(r["channel"] for r in out if r["channel"])
    total = len(out)
    refuted = sum(verdicts[v] for v in TEXT_REFUTED)

    L = [f"# DARK `deferred:chained_only` — what can chaining actually explain? "
         f"(SV-CORPUS-GRAD.13c)\n",
         "> Generated by `stimuli/sv/audit_dark_chained_only.py`. ⛔ No row is relabelled "
         "here: a row earns a verdict by being PARSED. The ⭐ buckets are a WORKLIST for "
         "toolbox adjudication, not a defect count.\n",
         "## Instrument identity\n",
         "| input | repo-root-relative path | sha256 |", "|---|---|---|"]
    for label, p in (("manifest", args.manifest), ("positions", args.positions),
                     ("grammar", args.grammar)):
        L.append(f"| {label} | `{p.relative_to(ROOT)}` | "
                 f"`{hashlib.sha256(p.read_bytes()).hexdigest()}` |")
    L += ["",
          "## The TEXT channel — can expansion move the byte the parse choked on?\n",
          "| verdict | rows | % | reading |", "|---|---:|---:|---|"]
    reading = {
        "NOT-SV-SOURCE": "⛔ **not source text** — a `$readmemh` memory image (`@address` + "
                         "hex bytes) carrying a `.v` extension. It can never parse and is NOT "
                         "evidence about the parser: a CORPUS COMPOSITION defect",
        "NO-ALTERABLE-TICK": "the file has **no alterable directive at all** — expansion is "
                             "a no-op on it. The textual deferral is REFUTED",
        "PRE-TICK": "the parse choked **before** the first byte expansion can reach — it "
                    "would choke identically on the expanded text. REFUTED",
        "IN-WINDOW": "choked between the first and last alterable directive — an earlier "
                     "expansion can change the following token stream. UNDECIDABLE from text",
        "PAST-LAST-TICK": "choked past every alterable directive — suspicious but still "
                          "reachable by an earlier expansion's shift. UNDECIDABLE",
        "NO-POSITION": "no failure position banked for this row (⛔ investigate: every "
                       "failing row should have one)",
    }
    for v, n in verdicts.most_common():
        L.append(f"| `{v}` | {n:,} | {100*n/total:.1f} % | {reading.get(v, '')} |")
    L.append(f"| **TOTAL** | **{total:,}** | **100.0 %** | |")
    L += ["",
          f"⇒ **{refuted:,} of {total:,} rows ({100*refuted/total:.1f} %) carry a "
          "`chained_only` deferral whose TEXTUAL justification their own bytes refute.**",
          ""]
    L += ["## Then the FACT channel — is a cross-file *name* the live explanation?\n",
          "| channel | rows | reading |", "|---|---:|---|"]
    creading = {
        "FACT-GATED-CROSS-FILE": "the parser demanded a type fact AT THE FAILURE that another file of "
                                 "the suite declares (`NAME @ file`) — for a Surelog row, a file of its "
                                 "own test unit ⇒ the `.6d` declaration barrier (the library design's "
                                 "population, `SVPP-EXPANSION.6d`), not the worklist's",
        "FACT-GATED-OUT-OF-UNIT": "⭐ the demanded name is declared in the suite but in **no file of the "
                                  "row's own compilation unit** (Surelog: another test's directory; "
                                  "`SV-CORPUS-GRAD.57c`) — the unit's compile cannot see it, so the "
                                  "sound store gate's refusal is provable on the raw text ⇒ WORKLIST "
                                  "(the `.13c.2` proof runs on it; a must-accept row it proves earns "
                                  "`chained:explained_store_gate` under the pipeline)",
        "CROSS-LIBRARY": "the name lives in a library vendored in ANOTHER suite (Surelog's tests use the "
                         "UVM under `stimuli/sv/uvm`) — the compilation unit is incomplete as the corpus "
                         "holds it; neither a parser defect nor in-suite chaining",
        "UNDECLARED-ANYWHERE": "⭐ a fact was demanded at the failure and NOTHING in the suite declares "
                               "the name as a type ⇒ WORKLIST (a candidate defect until the toolbox "
                               "says otherwise)",
        "NO-FACT-GATE-AT-FAILURE": "⭐ no missing fact at the failure — nothing to expand, no unknown "
                                   "name ⇒ WORKLIST",
        "FACT-GATED-LOCAL-TYPE": "⭐ the demanded name **IS declared in this very file** and the parser "
                                 "still lacked the fact ⇒ WORKLIST",
        "PROBE-TIMEOUT": "the traced parse did not finish inside the timeout — unclassified",
    }
    for c, n in channels.most_common():
        L.append(f"| `{c}` | {n:,} | {creading.get(c, '')} |")
    L += ["",
          f"⇒ **the worklist is {len(worklist):,} rows** "
          f"(`docs/tasks/artifacts/sv_corpus_grad/dark_chained_only/worklist.tsv`) — every "
          "one must be adjudicated with the toolbox (probe + `--trace-rules`) before it is "
          "called a defect.",
          ""]
    L += ["## Worklist by suite\n", "| suite | channel | rows |", "|---|---|---:|"]
    for (s, c), n in Counter((r["suite"], r["channel"]) for r in worklist).most_common():
        L.append(f"| {s} | `{c}` | {n:,} |")
    L += ["", "## First 40 worklist rows (source excerpt at the failure)\n",
          "| suite | file | line | channel | source at the failure |",
          "|---|---|---:|---|---|"]
    for r in worklist[:40]:
        L.append(f"| {r['suite']} | `{r['relpath']}` | {r['fail_line']} | "
                 f"`{r['channel']}` | {md_code_cell(r['snippet'][:90])} |")
    L.append("")

    (args.outdir / "summary.md").write_text("\n".join(L) + "\n", encoding="utf-8")

    for v, n in verdicts.most_common():
        print(f"{v:>18}: {n:6,}")
    print(f"{'TEXT-REFUTED':>18}: {refuted:6,}  ({100*refuted/total:.1f} %)")
    for c, n in channels.most_common():
        print(f"{c:>18}: {n:6,}")
    print(f"{'WORKLIST':>18}: {len(worklist):6,}")
    # Repo-root-relative when it can be (the project's path policy), absolute when the caller
    # pointed --outdir elsewhere — reporting where the file went must not raise (a scratch run
    # with a relative --outdir crashed here AFTER writing every artifact, rc 1 on complete output).
    out_md = args.outdir / "summary.md"
    print(f"wrote {out_md.relative_to(ROOT) if out_md.is_absolute() and out_md.is_relative_to(ROOT) else out_md}")


if __name__ == "__main__":
    main()
