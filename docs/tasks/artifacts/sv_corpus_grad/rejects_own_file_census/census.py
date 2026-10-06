#!/usr/bin/env python3
"""SVPP-EXPANSION.6e — the census of the rows the parser REJECTS IN THEIR OWN FILE after expansion.

The re-adjudication (`stimuli/sv/pipeline_readjudication.py`) files a deferred row that preprocesses and
then fails to parse as `rejects_own_file` (`chained:` / `svpp_owned:`), with the construct of the SOURCE
line at the mapped origin. That column reads the source, not the EXPANSION, and the expansion is what the
parser saw — so the class carried a mixture: a parser rejection, an undefined macro left as a raw backtick
(a CONTEXT gap that happened at the parser instead of the preprocessor), a file that is only ever read
through its includer, a spelling the 2023 text licenses and the 2017 text does not, an error test. This
census re-expands every such row under the lane's own derived context and classes it MECHANICALLY, in
this order (first match wins):

  raw_backtick        the token at `furthest_position` in the EXPANDED text carries a backtick (`` `X ``, or
                      `` 32'h`X `` glued) — an undefined macro reached the parser; the macro's name is recorded
                      (the card `a-raw-backtick-at-the-furthest-position-is-a-context-defect`)
  base_class          the origin line's construct is a class header with a base class — the cross-file
                      declaration barrier the library flags exist for (the pipeline chapter)
  edition_2023        the SAME row passes under the `sv_2023` reading — the 2017 rejection is the older text's
                      (the sv_2017 × sv_2023 census carries it as `2023_only_pass`)
  tolerated_spelling  the SAME row passes under `sv_2023_tolerant` only — one of the opt-in profile's closed
                      list of spellings (the tolerant census carries it as `tolerant_only_pass`)
  edition_2023_moves  the `sv_2023` reading of the same row stops FURTHER than the 2017 one: the construct at
                      THIS stop is licensed by the 2023 text (a parameter's unsized `[]`, a `begin_keywords
                      "1800-2023"`), and a later barrier keeps the row failing under 2023
  tolerated_moves     the `sv_2023_tolerant` reading stops further than the `sv_2023` one: the construct at this
                      stop is a tolerated spelling, and a later barrier keeps the row failing
  error_test          the path names an intended-invalid test (`_bad`, `/error/`, `should_fail`, `_fail`) — the
                      rejection is the expected verdict; the row was never keyed because its suite is deferred
  preprocess_only     a verilator test whose own driver (`t/<stem>.py`) runs the tool with `-E`: the file is the
                      PREPROCESSOR's input and its output is compared as text; it is not SystemVerilog to parse
  declared_fragment   the file declares itself a fragment (verible's `// verilog_syntax: parse-as-module-body` and
                      kin): a module body with no module around it, by the suite's own convention
  fragment            the file is the `include` target of another corpus source file — its standalone
                      reading is not a claim about the parser (the includer's reading is the one that counts)
  undeclared_type     the stop is an identifier that follows another identifier (optionally `pkg::`-scoped, with
                      a `#(…)`) and a declaration qualifier — `mubi4_t sel_mux;`, `uvm_component parent)`: a
                      declaration whose TYPE the parser has not seen declared — the same cross-file barrier as
                      `base_class`, in the data-declaration position
  residue             none of the above: listed one by one with the token at the stop, for the leaf to adjudicate

⛔ IDENTITY IS RECORDED, AND `--check` VERIFIES IT: the tracked census names the sha256 of the re-adjudication
table, of the two dumps it read (sv_2017, sv_2023) and of the expansion binary; `--check` re-hashes them
(an absent dump is NOT EVALUATED, rc 0; a moved input is DRIFT, rc 1) without re-expanding, so it is cheap
enough for the PIPELINE-LANE-CURRENCY arm; `--check --full` re-derives and compares bytes.

Usage:
  python3 docs/tasks/artifacts/sv_corpus_grad/rejects_own_file_census/census.py              # derive + write census.md / census.tsv
  python3 docs/tasks/artifacts/sv_corpus_grad/rejects_own_file_census/census.py --check      # identity only (cheap)
  python3 docs/tasks/artifacts/sv_corpus_grad/rejects_own_file_census/census.py --check --full
  python3 docs/tasks/artifacts/sv_corpus_grad/rejects_own_file_census/census.py --self-test  # the classifier, no parser
"""
from __future__ import annotations

import argparse
import hashlib
import os
import pathlib
import re
import subprocess
import sys
from collections import Counter, defaultdict

ROOT = pathlib.Path(__file__).resolve().parents[5]
HERE = pathlib.Path(__file__).resolve().parent
CH = ROOT / "stimuli/sv/characterization"
READJ = CH / "pipeline_readjudication.tsv"
DUMP_2017 = CH / "pipeline_results.tsv"
DUMP_2023 = CH / "pipeline_results_sv2023.tsv"
DUMP_TOL = CH / "pipeline_results_sv2023_tolerant.tsv"
EXPANDER = ROOT / "rust/target/debug/ast_pipeline"
CTX_ARGS = ROOT / "stimuli/sv/pipeline_context_args.sh"
OUT_MD = HERE / "census.md"
OUT_TSV = HERE / "census.tsv"
SCRATCH = ROOT / "rust/target/rejects_own_file_census"
INCLUDE_RE = re.compile(r'^[ \t]*`include[ \t]+"([^"]+)"', re.M)
INTENDED_INVALID_RE = re.compile(r"_bad|/error/|should_fail|_fail")
BASE_CLASS = "class header with a base class (`extends`)"
SOURCE_SUFFIXES = (".sv", ".svh", ".v", ".vh")
LRM_DIRECTIVES = set("begin_keywords celldefine default_nettype define else elsif end_keywords endcelldefine endif ifdef ifndef include line nounconnected_drive pragma resetall timescale unconnected_drive undef undefineall __FILE__ __LINE__".split())
CLASSES = ["raw_backtick", "base_class", "edition_2023", "tolerated_spelling", "edition_2023_moves", "tolerated_moves",
           "error_test", "preprocess_only", "declared_fragment", "fragment", "undeclared_type", "residue"]
DECLARED_FRAGMENT_RE = re.compile(r"verilog_syntax:\s*parse-as-")
PREPROCESS_ONLY_RE = re.compile(r"""(?<![\w-])["']?-E["']?(?![\w-])""")
SV_KEYWORDS = set("""accept_on alias always always_comb always_ff always_latch and assert assign assume automatic before begin bind bins binsof bit break buf bufif0 bufif1 byte case casex casez cell chandle checker class clocking cmos config const constraint context continue cover covergroup coverpoint cross deassign default defparam design disable dist do edge else end endcase endchecker endclass endclocking endconfig endfunction endgenerate endgroup endinterface endmodule endpackage endprimitive endprogram endproperty endspecify endsequence endtable endtask enum event eventually expect export extends extern final first_match for force foreach forever fork forkjoin function generate genvar global highz0 highz1 if iff ifnone ignore_bins illegal_bins implements implies import incdir include initial inout input inside instance int integer interconnect interface intersect join join_any join_none large let liblist library local localparam logic longint macromodule matches medium modport module nand negedge nettype new nexttime nmos nor noshowcancelled not notif0 notif1 null or output package packed parameter pmos posedge primitive priority program property protected pull0 pull1 pulldown pullup pulsestyle_ondetect pulsestyle_onevent pure rand randc randcase randsequence rcmos real realtime ref reg reject_on release repeat restrict return rnmos rpmos rtran rtranif0 rtranif1 s_always s_eventually s_nexttime s_until s_until_with scalared sequence shortint shortreal showcancelled signed small soft solve specify specparam static string strong strong0 strong1 struct super supply0 supply1 sync_accept_on sync_reject_on table tagged task this throughout time timeprecision timeunit tran tranif0 tranif1 tri tri0 tri1 triand trior trireg type typedef union unique unique0 unsigned until until_with untyped use uwire var vectored virtual void wait wait_order wand weak weak0 weak1 while wildcard wire with within wor xnor xor""".split())
QUALIFIERS = r"(?:(?:static|automatic|const|var|input|output|inout|ref|localparam|parameter|rand|randc|local|protected|virtual)\s+)*"
UNDECLARED_TYPE_RE = re.compile(r"(?:^|[\s(,])" + QUALIFIERS + r"([A-Za-z_]\w*)(?:::[A-Za-z_]\w*)*(?:\s*#\s*\([^()]*\))?\s+$")
IDENT_RE = re.compile(r"[A-Za-z_]\w*$")
TOKEN_RE = re.compile(r"`[A-Za-z_][A-Za-z0-9_]*|[A-Za-z_$][A-Za-z0-9_$]*|\S")


def sha256_of(p: pathlib.Path) -> str:
    return hashlib.sha256(p.read_bytes()).hexdigest()


def token_at(text: str) -> str:
    """The first token at or after the stop, white space INCLUDING newlines skipped: the furthest position
    the parser reports can sit at the end of the last token it accepted, before the trivia that precedes
    the token it could not read (measured: nine rows whose expansion reads `\n` and then the raw backtick)."""
    t = text.lstrip()
    if not t:
        return "<end of input>"
    m = TOKEN_RE.match(t)
    return m.group(0) if m else "<end of input>"


def undeclared_type_shape(before: str, tok: str) -> bool:
    """`before` is the source line up to the stop; `tok` the token at the stop: an identifier preceded by an
    identifier (not a keyword) is a declaration whose type the parser never saw declared."""
    if not IDENT_RE.match(tok) or tok in SV_KEYWORDS:
        return False
    m = UNDECLARED_TYPE_RE.search(before + " ")
    return bool(m) and m.group(1) not in SV_KEYWORDS


def classify(at: str, construct: str, passes_2023: bool, relpath: str, include_target: int,
             passes_tolerant: bool = False, preprocess_only: bool = False, declared_fragment: bool = False,
             before: str = "", fp_2017: int = -1, fp_2023: int = -1, fp_tol: int = -1) -> str:
    """The mechanical class of one row — the order is the docstring's."""
    head = at.lstrip()
    m = re.match(r"\S*`([A-Za-z_][A-Za-z0-9_]*)?", head)
    if m and (m.group(1) or "") not in LRM_DIRECTIVES:
        # `.6f`: a backtick the stage let through is an LRM directive the parser reads (`` `resetall `` inside a
        # module, refused by the grammar's own position rule) — not an unresolved usage; every other backtick
        # at the stop is one (and since `.6f` the stage refuses it before the parser can meet it)
        return "raw_backtick"
    if construct == BASE_CLASS:
        return "base_class"
    if passes_2023:
        return "edition_2023"
    if passes_tolerant:
        return "tolerated_spelling"
    if fp_2017 >= 0 and fp_2023 > fp_2017:
        return "edition_2023_moves"
    if fp_2023 >= 0 and fp_tol > fp_2023:
        return "tolerated_moves"
    if INTENDED_INVALID_RE.search(relpath):
        return "error_test"
    if preprocess_only:
        return "preprocess_only"
    if declared_fragment:
        return "declared_fragment"
    if include_target > 0:
        return "fragment"
    if undeclared_type_shape(before, token_at(at)):
        return "undeclared_type"
    return "residue"


def driver_preprocess_only(suite: str, repo_rel: str) -> bool:
    """verilator: the same-stem driver `t/<stem>.py` passes `-E` (preprocess only)."""
    if suite != "verilator":
        return False
    p = ROOT / repo_rel
    drv = p.with_suffix(".py")
    try:
        return bool(PREPROCESS_ONLY_RE.search(drv.read_text(errors="replace")))
    except OSError:
        return False


def declares_fragment(repo_rel: str) -> bool:
    try:
        head = (ROOT / repo_rel).read_text(errors="replace")[:600]
    except OSError:
        return False
    return bool(DECLARED_FRAGMENT_RE.search(head))


def source_before(origin: str) -> str:
    """The origin line's text up to the stop column (the source side of the mapped position)."""
    if origin.count(":") < 2:
        return ""
    f, ln, col = origin.rsplit(":", 2)
    try:
        line = (ROOT / f if not f.startswith("/") else pathlib.Path(f)).read_text(errors="replace").split("\n")[int(ln) - 1]
        return line[: max(0, int(col) - 1)]
    except (OSError, ValueError, IndexError):
        return ""


def read_readj():
    rows = []
    with open(READJ, encoding="utf-8") as f:
        header = f.readline().rstrip("\n").split("\t")
        for line in f:
            c = line.rstrip("\n").split("\t")
            if len(c) < len(header):
                c += [""] * (len(header) - len(c))
            rows.append(dict(zip(header, c)))
    return [r for r in rows if r["verdict"].endswith("rejects_own_file")]


def read_dump(p: pathlib.Path) -> dict:
    d = {}
    for line in open(p, encoding="utf-8", errors="replace"):
        c = line.rstrip("\n").split("\t")
        if len(c) >= 3 and c[0] != "suite":
            c += [""] * (7 - len(c))
            d[(c[0], c[2])] = c
    return d


def lane_key(dump: dict, suite: str, relpath: str):
    for (s, p), c in dump.items():
        if s == suite and (p.endswith("/" + relpath) or p == relpath):
            return (s, p), c
    return None, None


def include_targets() -> Counter:
    """basename -> number of `include lines naming it, over every corpus source file."""
    t: Counter = Counter()
    for base in (ROOT / "stimuli/sv/subs", ROOT / "stimuli/sv/uvm"):
        if not base.exists():
            continue
        for p in base.rglob("*"):
            if p.is_file() and p.suffix in SOURCE_SUFFIXES:
                try:
                    text = p.read_text(errors="replace")
                except OSError:
                    continue
                for n in INCLUDE_RE.findall(text):
                    t[pathlib.PurePosixPath(n).name] += 1
    return t


def context_flags(suite: str, repo_rel: str) -> list[str]:
    out = subprocess.run(["bash", "-c", f"source {CTX_ARGS}; context_args '{suite}' '{repo_rel}'"],
                         capture_output=True, text=True, cwd=ROOT).stdout
    flags = []
    for line in out.splitlines():
        if "\t" not in line:
            continue
        f, v = line.split("\t", 1)
        if f in ("--include-dir", "--symbol", "--unit-file"):
            flags += [f, v]
    return flags


def expand(suite: str, repo_rel: str, idx: int) -> str | None:
    SCRATCH.mkdir(parents=True, exist_ok=True)
    outp = SCRATCH / f"exp_{idx}.txt"
    r = subprocess.run([str(EXPANDER), repo_rel, "--preprocess", "systemverilog", "--output", str(outp)]
                       + context_flags(suite, repo_rel), capture_output=True, text=True, cwd=ROOT, timeout=600)
    if r.returncode != 0 or not outp.exists():
        return None
    return outp.read_text(encoding="utf-8", errors="replace")


def derive():
    rj = read_readj()
    d17 = read_dump(DUMP_2017)
    d23 = read_dump(DUMP_2023) if DUMP_2023.exists() else {}
    dtol = read_dump(DUMP_TOL) if DUMP_TOL.exists() else {}
    targets = include_targets()
    rows = []
    for i, r in enumerate(rj):
        suite, rel = r["suite"], r["relpath"]
        key, c17 = lane_key(d17, suite, rel)
        if key is None:
            rows.append({"suite": suite, "relpath": rel, "class": "residue", "fp": "", "origin": r.get("origin", ""),
                         "token": "<no lane row>", "construct": r["construct"], "sv2023": "?", "sv2023_tolerant": "?", "include_target": 0, "verdict": r["verdict"]})
            continue
        repo_rel = key[1]
        fp = c17[5]
        text = expand(suite, repo_rel, i)
        at = ""
        if text is not None and fp:
            p = int(fp)
            at = text[p:p + 60]
        c23 = d23.get(key)
        ctol = dtol.get(key)
        passes_2023 = bool(c23) and c23[1] == "pass"
        passes_tol = bool(ctol) and ctol[1] == "pass"
        inc = targets.get(pathlib.PurePosixPath(rel).name, 0)
        origin = r.get("origin", "")
        def fp_of(c):
            try:
                return int(c[5]) if c and c[1] == "fail" and c[5] else -1
            except ValueError:
                return -1
        cls = classify(at if text is not None else "", r["construct"], passes_2023, rel, inc, passes_tol,
                       driver_preprocess_only(suite, repo_rel), declares_fragment(repo_rel), source_before(origin),
                       fp_of(c17), fp_of(c23), fp_of(ctol))
        tok = token_at(at) if text is not None else "<expansion failed>"
        rows.append({"suite": suite, "relpath": rel, "class": cls, "fp": fp, "origin": origin,
                     "token": tok, "construct": r["construct"], "sv2023": (c23[1] if c23 else "-"),
                     "sv2023_tolerant": (ctol[1] if ctol else "-"), "include_target": inc, "verdict": r["verdict"]})
    identity = [("re-adjudication table", str(READJ.relative_to(ROOT)), sha256_of(READJ)),
                ("sv_2017 dump", str(DUMP_2017.relative_to(ROOT)), sha256_of(DUMP_2017)),
                ("sv_2023 dump", str(DUMP_2023.relative_to(ROOT)), sha256_of(DUMP_2023) if DUMP_2023.exists() else "absent"),
                ("sv_2023_tolerant dump", str(DUMP_TOL.relative_to(ROOT)), sha256_of(DUMP_TOL) if DUMP_TOL.exists() else "absent"),
                ("expansion binary", str(EXPANDER.relative_to(ROOT)), sha256_of(EXPANDER) if EXPANDER.exists() else "absent")]
    return rows, identity


def render_tsv(rows) -> str:
    L = ["suite\trelpath\tclass\tfurthest_position\torigin\ttoken_at_stop\tconstruct\tsv_2023\tsv_2023_tolerant\tinclude_target\tverdict"]
    for r in sorted(rows, key=lambda r: (CLASSES.index(r["class"]), r["suite"], r["relpath"])):
        L.append("\t".join(str(r[k]) for k in ("suite", "relpath", "class", "fp", "origin", "token", "construct", "sv2023", "sv2023_tolerant", "include_target", "verdict")))
    return "\n".join(L) + "\n"


def code(text: str) -> str:
    longest = max((len(m) for m in re.findall(r"`+", text)), default=0)
    fence = "`" * (longest + 1)
    pad = " " if text.startswith("`") or text.endswith("`") else ""
    return f"{fence}{pad}{text}{pad}{fence}"


def render_md(rows, identity) -> str:
    n = len(rows)
    by = Counter(r["class"] for r in rows)
    L = ["# The rows the parser rejects in their own file, classed by what it actually saw (SVPP-EXPANSION.6e)", "",
         "Derived by `census.py` from the tracked re-adjudication table, the two lane dumps and a re-expansion of every row",
         "under the lane's own derived context — **never edited by hand**. `--check` re-hashes the identity below.", "",
         "## Identity", "", "| input | path | sha256 |", "|---|---|---|"]
    for label, path, sha in identity:
        L.append(f"| {label} | `{path}` | `{sha}` |")
    L += ["", f"## The classes — {n} rows", "", "| class | rows | what it means |", "|---|---:|---|"]
    meaning = {
        "raw_backtick": "the expanded text at the stop begins with a backtick: an undefined macro reached the parser — CONTEXT, not syntax",
        "base_class": "a class header with a base class the parser has not seen declared — the cross-file declaration barrier",
        "edition_2023": "the same row passes under the `sv_2023` reading — the 2017 rejection is the older text's",
        "tolerated_spelling": "the same row passes under `sv_2023_tolerant` only — a spelling of the opt-in profile's closed list",
        "edition_2023_moves": "the 2023 reading stops further: the construct at this stop is the 2023 text's, a later barrier remains",
        "tolerated_moves": "the tolerant reading stops further than the 2023 one: a tolerated spelling here, a later barrier remains",
        "error_test": "an intended-invalid test by name — the rejection is the expected verdict",
        "preprocess_only": "a verilator test its own driver runs with `-E` — the preprocessor's input, not SystemVerilog to parse",
        "declared_fragment": "the file declares itself a module body (`verilog_syntax: parse-as-…`) — the suite's own convention",
        "fragment": "the file is the `include` target of another corpus file — its standalone reading is not the parser's claim",
        "undeclared_type": "a declaration whose type is an identifier the parser has not seen declared — the cross-file barrier in data position",
        "residue": "none of the above — adjudicated one by one below",
    }
    for cls in CLASSES:
        L.append(f"| `{cls}` | {by.get(cls, 0)} | {meaning[cls]} |")
    # raw backtick: by macro
    rb = [r for r in rows if r["class"] == "raw_backtick"]
    L += ["", f"## `raw_backtick` — {len(rb)} rows, by the macro the context did not define", "",
          "| macro | rows | files |", "|---|---:|---|"]
    bym = defaultdict(list)
    for r in rb:
        bym[r["token"]].append(r)
    for tok, rs in sorted(bym.items(), key=lambda kv: (-len(kv[1]), kv[0])):
        files = ", ".join(sorted(f"{r['suite']}:{r['relpath'].rsplit('/', 1)[-1]}" for r in rs))
        L.append(f"| {code(tok)} | {len(rs)} | {files} |")
    # fragments by token
    fr = [r for r in rows if r["class"] == "fragment"]
    L += ["", f"## `fragment` — {len(fr)} rows, by the token at the stop", "", "| token at the stop | rows | one example |", "|---|---:|---|"]
    byt = defaultdict(list)
    for r in fr:
        byt[r["token"]].append(r)
    for tok, rs in sorted(byt.items(), key=lambda kv: (-len(kv[1]), kv[0])):
        ex = rs[0]
        L.append(f"| {code(tok)} | {len(rs)} | `{ex['suite']}` `{ex['relpath']}` ({ex['origin'].rsplit(':', 2)[-2] if ex['origin'].count(':') >= 2 else ''}) |")
    # base class / edition / error tests: counts by suite
    ut = [r for r in rows if r["class"] == "undeclared_type"]
    L += ["", f"## `undeclared_type` — {len(ut)} rows, by the type the parser never saw", "", "| type | rows | files |", "|---|---:|---|"]
    byu = defaultdict(list)
    for r in ut:
        m = UNDECLARED_TYPE_RE.search(source_before(r["origin"]) + " ")
        byu[m.group(1) if m else "?"].append(r)
    for t, rs in sorted(byu.items(), key=lambda kv: (-len(kv[1]), kv[0])):
        L.append(f"| `{t}` | {len(rs)} | {', '.join(sorted(f"{r['suite']}:{r['relpath'].rsplit('/', 1)[-1]}" for r in rs))} |")
    for cls in ("base_class", "edition_2023", "tolerated_spelling", "edition_2023_moves", "tolerated_moves", "error_test", "preprocess_only", "declared_fragment"):
        rs = [r for r in rows if r["class"] == cls]
        bys = Counter(r["suite"] for r in rs)
        L += ["", f"## `{cls}` — {len(rs)} rows, by suite", "", "| suite | rows |", "|---|---:|"]
        for s, k in bys.most_common():
            L.append(f"| `{s}` | {k} |")
    # residue one by one
    rs = [r for r in rows if r["class"] == "residue"]
    L += ["", f"## `residue` — {len(rs)} rows, one by one", "", "| suite | file | origin | token at the stop | source construct | sv_2023 | tolerant |", "|---|---|---|---|---|---|---|"]
    for r in sorted(rs, key=lambda r: (r["suite"], r["relpath"])):
        L.append(f"| `{r['suite']}` | `{r['relpath']}` | `{r['origin'].split('/subs/')[-1].split('/uvm/')[-1]}` | {code(r['token'])} | {r['construct']} | `{r['sv2023']}` | `{r['sv2023_tolerant']}` |")
    return "\n".join(L) + "\n"


def self_test() -> int:
    checks = [
        ("a backtick at the stop is raw_backtick before anything else", classify("`EN_MASKING == 0", BASE_CLASS, True, "x/error/y_bad.sv", 3) == "raw_backtick"),
        ("a backtick after a newline at the stop is raw_backtick too", classify("\n`MEM_BKDR_UTIL_FILE_OP(", "a `::`-scoped name", False, "a.sv", 0) == "raw_backtick"),
        ("a base-class construct wins over the 2023 reading", classify("foo", BASE_CLASS, True, "a.sv", 0) == "base_class"),
        ("a row that passes under sv_2023 is edition_2023", classify("] = {", "other", True, "a.sv", 0) == "edition_2023"),
        ("an intended-invalid name is error_test", classify("x", "other", False, "t/t_x_bad.v", 0) == "error_test"),
        ("an include target is a fragment", classify("x", "other", False, "a.sv", 2) == "fragment"),
        ("otherwise residue", classify("x", "other", False, "a.sv", 0) == "residue"),
        ("an LRM directive at the stop (`resetall inside a module) is NOT raw_backtick", classify("`resetall\nmodule t;", "other", False, "t/t_pp_resetall_bad.v", 0) == "error_test"),
        ("a backtick glued after a based prefix is raw_backtick (32'h`DM_ADDR)", classify("h`DM_ADDR;", "other", False, "a.sv", 0) == "raw_backtick"),
        ("a row that passes under the tolerant profile only is tolerated_spelling", classify(".rlb iff", "other", False, "a.sv", 0, passes_tolerant=True) == "tolerated_spelling"),
        ("a -E driver makes preprocess_only, before the fragment test", classify("Hello in x", "other", False, "t/t_x.v", 1, preprocess_only=True) == "preprocess_only"),
        ("a parse-as-module-body declaration makes declared_fragment", classify("generate", "other", False, "a.sv", 0, declared_fragment=True) == "declared_fragment"),
        ("identifier after identifier is undeclared_type", classify("sel_mux;", "other", False, "a.sv", 0, before="  mubi4_t ") == "undeclared_type"),
        ("…also with a qualifier and a package scope", classify("parent);", "a function / task header", False, "a.sv", 0, before="extern function new (string name, uvm_pkg::uvm_component ") == "undeclared_type"),
        ("…but not after a keyword", classify("x = f;", "other", False, "a.sv", 0, before="  assign ") == "residue"),
        ("…and not when the stop token is a keyword", classify("task automatic w();", "other", False, "a.sv", 0, before="  static ") == "residue"),
        ("a 2023 stop further along than the 2017 one is edition_2023_moves", classify("] = {", "other", False, "a.sv", 0, fp_2017=100, fp_2023=5000) == "edition_2023_moves"),
        ("a tolerant stop further along than the 2023 one is tolerated_moves", classify(".rlb", "other", False, "a.sv", 0, fp_2017=100, fp_2023=100, fp_tol=900) == "tolerated_moves"),
        ("the same stop under every reading moves nothing", classify("x", "other", False, "a.sv", 0, fp_2017=100, fp_2023=100, fp_tol=100) == "residue"),
        ("the -E flag is read from the driver text, quoted or bare", bool(PREPROCESS_ONLY_RE.search('verilator_flags2=["-E"]')) and bool(PREPROCESS_ONLY_RE.search("flags: -E -P")) and not PREPROCESS_ONLY_RE.search("--E2BIG -Ex")),
        ("the token reader skips newlines too", token_at("\n  foo") == "foo"),
        ("the token reader: nothing after the stop", token_at("  \n") == "<end of input>"),
        ("the token reader: a macro usage", token_at("  `RNG_BUS_WIDTH;") == "`RNG_BUS_WIDTH"),
        ("the token reader: punctuation", token_at("] = {") == "]"),
    ]
    bad = 0
    for label, ok in checks:
        print(f"  {'PASS' if ok else 'FAIL'} {label}")
        bad += 0 if ok else 1
    print(f"REJECTS-OWN-FILE-CENSUS-SELF-TEST: checks={len(checks)} failed={bad}")
    return 7 if bad else 0


def check(full: bool) -> int:
    if not OUT_MD.exists():
        print("REJECTS-OWN-FILE-CENSUS: no tracked census.md — run the census", file=sys.stderr)
        return 1
    if not DUMP_2017.exists():
        print("REJECTS-OWN-FILE-CENSUS: NOT EVALUATED — the sv_2017 dump is absent on this tree (run the lane)")
        return 0
    text = OUT_MD.read_text(encoding="utf-8")
    rows = re.findall(r"^\| (.+?) \| `(.+?)` \| `([0-9a-f]{64}|absent)` \|$", text, re.M)
    wants = {label: (path, sha) for label, path, sha in rows}
    for label in ("re-adjudication table", "sv_2017 dump", "sv_2023 dump"):
        if label not in wants:
            print(f"REJECTS-OWN-FILE-CENSUS: DRIFT — the identity block lacks '{label}'", file=sys.stderr)
            return 1
        path, sha = wants[label]
        p = ROOT / path
        if not p.exists():
            if label == "sv_2023 dump":
                print(f"REJECTS-OWN-FILE-CENSUS: NOT EVALUATED for {label} (absent on this tree)")
                continue
            print(f"REJECTS-OWN-FILE-CENSUS: DRIFT — {path} is absent", file=sys.stderr)
            return 1
        if sha256_of(p) != sha:
            print(f"REJECTS-OWN-FILE-CENSUS: DRIFT — {label} ({path}) no longer hashes to the census's identity; re-run the census", file=sys.stderr)
            return 1
    if full:
        rows2, identity = derive()
        if render_md(rows2, identity) != text or render_tsv(rows2) != OUT_TSV.read_text(encoding="utf-8"):
            print("REJECTS-OWN-FILE-CENSUS: DRIFT — a fresh derivation differs from the tracked census", file=sys.stderr)
            return 1
    print(f"REJECTS-OWN-FILE-CENSUS: OK — identity holds ({'full re-derivation' if full else 'inputs re-hashed'})")
    return 0


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--check", action="store_true")
    ap.add_argument("--full", action="store_true")
    ap.add_argument("--self-test", action="store_true")
    a = ap.parse_args()
    if a.self_test:
        return self_test()
    if a.check:
        return check(a.full)
    if not DUMP_2017.exists() or not READJ.exists():
        print("REJECTS-OWN-FILE-CENSUS: REFUSED — the sv_2017 dump or the re-adjudication table is missing", file=sys.stderr)
        return 2
    if not EXPANDER.exists():
        print(f"REJECTS-OWN-FILE-CENSUS: REFUSED — {EXPANDER.relative_to(ROOT)} is missing (build the debug ast_pipeline)", file=sys.stderr)
        return 2
    rows, identity = derive()
    OUT_TSV.write_text(render_tsv(rows), encoding="utf-8")
    OUT_MD.write_text(render_md(rows, identity), encoding="utf-8")
    by = Counter(r["class"] for r in rows)
    print("REJECTS-OWN-FILE-CENSUS: rows=%d %s → %s, %s" % (len(rows), " ".join(f"{c}={by.get(c, 0)}" for c in CLASSES),
                                                              OUT_MD.relative_to(ROOT), OUT_TSV.relative_to(ROOT)))
    return 0


if __name__ == "__main__":
    sys.exit(main())
