#!/usr/bin/env python3
"""SV external-corpus adjudication manifest builder (SV-CORPUS-GRAD.2 + .8).

Turns the raw parse outcomes of stimuli/run_external_corpus.sh sv
(stimuli/sv/characterization/results.tsv) into the expected-vs-actual
adjudication manifest the graduation campaign burns down from
(docs/tasks/SV-CORPUS-GRAD.md leaf .2; ADD-v1 suites + uvm-core fold = leaf .8;
deep answer-key extraction - Surelog golden logs, sv2v error-pattern stage
classification, ivtest vvp_tests JSON descriptors = leaf .8b.1; per-file
pinned stage adjudication of the ispras NEGATIVE + sv2v residue = leaf .8b.2;
clustered stage pins for the ivtest CE-without-gold population = leaf .8b.3).

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
  out_of_scope_with_cause  owned by another lane (cause named, e.g. svpp).
                           May carry a deferral slug suffix after ':'
                           (e.g. out_of_scope_with_cause:v2005_profile_lane) —
                           the slug names the owning lane in the verdict class;
                           the bare form keeps the historical svpp_owned label.

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
import json
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

# Pinned rulings from in-file upstream statements (leaf .3.0 triage findings).
EXTRA_PINNED = {
    ("verible", "verible/verilog/tools/lint/testdata/bad-id-lex.sv"): (
        "must_reject", "pinned: in-file comment 'lexer should reject invalid "
        "identifier' (module 654foo)"),
    ("verible", "verible/verilog/tools/lint/testdata/module_begin_block.sv"): (
        "must_reject", "pinned: in-file comment marks the bare begin block "
        "'LRM-invalid syntax'"),
    ("slang", "tests/regression/driver/cross-ident-in-binsof.sv"): (
        "must_reject", "pinned: in-file comment 'The LRM disallows "
        "cross_identifier as a bins_expression' - a grammar-level restriction, "
        "and the driver CHECKs for an error"),
}

VERIBLE_SYNTAX_MODE_RE = re.compile(r"//\s*verilog_syntax\s*:")

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


def expect_verible(relpath: str, text: str):
    p = relpath.replace("\\", "/")
    if VERIBLE_SYNTAX_MODE_RE.search(text):
        return ("out_of_scope_with_cause",
                "verible: '// verilog_syntax:' excerpt-mode fixture - a fragment "
                "parsed under a tool-specific mode, never a standalone unit")
    if "/kythe/testdata/" in p:
        tail = p.split("/kythe/testdata/", 1)[1]
        if "/" in tail:
            return ("chained_only", "verible: kythe multi-file/include fixture directory")
        return ("must_accept", "verible: kythe single-file indexing fixture")
    if "/lint/testdata/" in p:
        return ("must_accept", "verible: lint-rule fixture (style findings, valid syntax)")
    return ("must_accept", "verible: fixture (positive-dominant default)")


# --- SV-CORPUS-GRAD.8 ADD-v1 suites -----------------------------------------

ISPRAS_TYPE_RE = re.compile(r"^\s*//\s*!\s*TYPE:\s*([A-Z]+)", re.MULTILINE)

# ispras TYPE:NEGATIVE per-file stage pins (leaf .8b.2): each file read and
# adjudicated against the IEEE 1800 LRM's BNF-vs-prose split (in-repo
# docs/systemverilog/2017/md), clause per the file's own header. must_accept
# = the text is BNF-parseable and the cited invalidity is a prose "shall"
# (semantic/elaboration stage); must_reject = the text violates the Annex A
# BNF (or, where noted, is syntactically broken as committed).
ISPRAS_NEGATIVE_PINNED = {
    "ieee-1800-2012/03/03.12.01_02.sv": ("must_accept",
        "pinned .8b.2: use-before-declaration + $unit:: forward reference "
        "(3.12.1) - name resolution, semantic stage"),
    "ieee-1800-2012/06/06.07.01_02.sv": ("must_accept",
        "pinned .8b.2: 'tri reg r;'/'inout wire reg p;' - A.2.1.3 "
        "net_declaration admits any data_type; the reg-on-net ban is prose "
        "(6.7.1)"),
    "ieee-1800-2012/06/06.21_03.sv": ("must_accept",
        "pinned .8b.2: block-local initializer without explicit "
        "static/automatic (6.21) - a lifetime prose rule; the BNF admits "
        "the declaration"),
    "ieee-1800-2012/06/06.24.03_01.sv": ("must_accept",
        "pinned .8b.2: bit-stream cast size mismatches (6.24.3) - "
        "type/width semantics"),
    "ieee-1800-2012/07/07.09.08_01.sv": ("must_accept",
        "pinned .8b.2: associative-array traversal argument narrowing "
        "(7.9.8) - argument typing semantics"),
    "ieee-1800-2012/08/08.24_02.sv": ("must_accept",
        "pinned .8b.2: out-of-block declaration type-name re-resolution "
        "(8.24) - semantic stage"),
    "ieee-1800-2012/08/08.26.04_01.sv": ("must_accept",
        "pinned .8b.2: 'implements' of a forward-typedef'd interface class "
        "(8.26.4) - declaration-ordering semantics"),
    "ieee-1800-2012/08/08.26.06.01_02.sv": ("must_accept",
        "pinned .8b.2: interface-class method name conflict (8.26.6.1) - "
        "semantic stage"),
    "ieee-1800-2012/09/09.03.02_03.sv": ("must_accept",
        "pinned .8b.2: 'return' inside fork..join_none (9.3.2/12.8-family "
        "placement prose) - jump_statement is an ordinary statement "
        "production"),
    "ieee-1800-2012/10/10.11_05.sv": ("must_accept",
        "pinned .8b.2: overlapping alias operands (10.11) - alias "
        "coherence semantics"),
    "ieee-1800-2012/10/10.11_06.sv": ("must_accept",
        "pinned .8b.2: self-alias (10.11) - semantic; A.6.1 net_alias BNF "
        "admits the multi-'=' form"),
    "ieee-1800-2012/13/13.05.02_02.sv": ("must_reject",
        "pinned .8b.2: 'ref input int a' - A.2.7 tf_port_item admits a "
        "single tf_port_direction; a second direction keyword cannot start "
        "data_type_or_implicit (parse level)"),
    "ieee-1800-2012/16/16.09.04_01.sv": ("must_reject",
        "pinned .8b.2: the committed text is syntactically broken - the a1 "
        "assert property's opening parenthesis is never closed (upstream "
        "typo); parse-level regardless of the intended 16.9.4 "
        "nested-gclk-function semantic rule"),
    "ieee-1800-2012/17/17.07.01_01.sv": ("must_accept",
        "pinned .8b.2: assigning a checker variable from outside (17.7.1) "
        "- semantic stage"),
    "ieee-1800-2012/19/19.08.01_02.sv": ("must_accept",
        "pinned .8b.2: covergroup port list + sample-method override "
        "(19.8.1) - both are independent BNF optionals (A.2.11); the "
        "conflict and option.* rules are prose"),
    "ieee-1800-2012/22/22.14.01_02.sv": ("must_reject",
        "pinned .8b.2: 'reg [63:0] logic;' - 'logic' is reserved both "
        "under the `begin_keywords \"1800-2005\" set (22.14) and under "
        "bare sv_2017 lexing; parse level either way"),
    "ieee-1800-2012/22/22.14.01_04.sv": ("must_reject",
        "pinned .8b.2: under `begin_keywords \"1364-2005\" (22.14) "
        "'interface'/'endinterface' are ordinary identifiers and the items "
        "match no IEEE 1364-2005 production (an instantiation requires "
        "parentheses) - parse level under the directive-aware reading (the "
        "F5 in-scope-directives family)"),
    "ieee-1800-2012/23/23.03.02.02_02.sv": ("must_accept",
        "pinned .8b.2: duplicate named port connections (23.3.2.2) - the "
        "BNF repeats named_port_connection freely; uniqueness is prose"),
    "ieee-1800-2012/23/23.10.01_01.sv": ("must_accept",
        "pinned .8b.2: defparam into another generate-loop instance "
        "(23.10.1) - elaboration semantics; the hierarchical name is "
        "BNF-parseable"),
    "ieee-1800-2012/30/30.04.04.03_04.sv": ("must_accept",
        "pinned .8b.2: overlapping edge-sensitive state-dependent paths "
        "(30.4.4.3) - path coherence semantics; each specify item is "
        "BNF-parseable"),
    "ieee-1800-2012/33/33.04.02_01.sv": ("must_accept",
        "pinned .8b.2: config liblist override through a config'd instance "
        "(33.4.2) - configuration resolution semantics"),
}


def expect_ispras(relpath: str, text: str):
    """ispras/sv-tests: per-file `// ! TYPE: POSITIVE|NEGATIVE|VARYING` keys;
    filenames encode the LRM clause. The ieee-1364-2005/ half is keyed for the
    verilog_2005 profile, not this sv_2017 bulk run."""
    p = relpath.replace("\\", "/")
    m = ISPRAS_TYPE_RE.search(text)
    ttype = m.group(1) if m else ""
    if p.startswith("ieee-1364-2005/"):
        return ("out_of_scope_with_cause:v2005_profile_lane",
                f"ispras: IEEE 1364-2005 half (TYPE: {ttype or 'none'}) - keyed "
                "for the verilog_2005 profile lane (incl. the KNOWN_TEXT_BUGS "
                "list), not the sv_2017 bulk run")
    if ttype == "POSITIVE":
        return ("must_accept",
                "ispras: '// ! TYPE: POSITIVE' clause-keyed valid example")
    if ttype == "NEGATIVE":
        pinned = ISPRAS_NEGATIVE_PINNED.get(p)
        if pinned:
            return pinned
        return ("out_of_scope_with_cause:negative_stage_triage",
                "ispras: TYPE NEGATIVE - invalid per the cited clause but the "
                "failure STAGE (parse vs elaboration) is not encoded; per-file "
                "stage triage = leaf .8b")
    if ttype == "VARYING":
        return ("out_of_scope_with_cause:impl_varying",
                "ispras: TYPE VARYING - implementation-dependent verdict by "
                "suite contract (profile-boundary probe)")
    return ("out_of_scope_with_cause:no_sv_key",
            "ispras: no '// ! TYPE:' key found in the file header")


# --- SV-CORPUS-GRAD.8c: the verilog_2005 profile lane -----------------------
#
# The v2005-lane rows (ispras ieee-1364-2005/, ivtest regress-vlg.list +
# explicit plain-Verilog vvp generations, sv2v .v conversion goldens) are
# re-parsed under `--profile verilog_2005` by the runner's sv2005 mode and
# adjudicated here against IEEE 1364-2005 answer keys. Edition law: verdicts
# in this lane follow the 1364-2005 BNF, which differs from 1800 (e.g. the
# charge/drive-strength placement rules ARE grammar there — the .8b.1
# 1364->1800 relaxations read in the opposite direction).

# ispras ieee-1364-2005/ TYPE:NEGATIVE per-file stage pins (leaf .8c.1): only
# two exist; both files read and adjudicated against the 1364-2005 BNF.
ISPRAS_1364_NEGATIVE_PINNED = {
    "ieee-1364-2005/test_12_03_03_2.v": ("must_accept",
        "pinned .8c.1: 12.3.3 port/net signedness-inheritance and "
        "concat/part-select/named header ports - every module header form "
        "is 1364-2005 A.1.4 port/port_expression/port_reference BNF and the "
        "redeclaration pairs are legal declaration sequences; the cited "
        "invalidities (signedness coherence, named-connection usability) "
        "are semantic stage"),
    "ieee-1364-2005/test_12_08_02_1.v": ("must_accept",
        "pinned .8c.1: 12.8.2 early defparam hierarchical-name resolution "
        "ambiguity - pure elaboration semantics; defparam and the generate "
        "block are ordinary 1364-2005 BNF"),
}

# ispras ieee-1364-2005/ TYPE:POSITIVE per-file re-adjudication (leaf .8c.2):
# every POSITIVE row failing at the .8c.1 baseline was read and re-adjudicated
# against the suite's KNOWN_TEXT_BUGS errata (committed-text-over-intent) and
# the IEEE 1364-2005 text (in-repo docs/verilog/2005/md: Annex A + clause
# prose). Outcome: ZERO errata flips - no committed text embodies an LRM typo
# that renders it BNF-invalid (the one errata candidate, 'PATHPULSE$ = 3', is
# parseable via the specparam_identifier alternative: Annex B does not reserve
# PATHPULSE$ and simple_identifier admits '$'), so all 17 stay must_accept =
# measured v2005 defect signal. Clusters keep the shared LRM ground in one
# place; the flatten below audits count + duplicates.
ISPRAS_1364_POSITIVE_CLUSTERS = [
    ("3.5.1/5.1 spaced + signed based literals - 3.5.1 verbatim: the size, "
     "base and value are separate tokens (A.8.7), white space is allowed "
     "between size and base, and the unsigned number token 'shall "
     "immediately follow the base format, optionally preceded by white "
     "space'; the s designator and unary minus on based primaries are "
     "ordinary A.8.3/A.8.6 syntax; KNOWN_TEXT_BUGS covers only Example 3 "
     "(16'sd?) - none of these texts",
     ["test_03_05_01_2", "test_03_05_01_4", "test_03_05_01_5",
      "test_05_01_03_1", "test_05_01_05_2"]),
    ("parenthesized mintypmax as a parameter value - A.2.4 param_assignment "
     "RHS is constant_mintypmax_expression and A.8.4 constant_primary "
     "admits ( constant_mintypmax_expression )",
     ["test_05_03_00_1"]),
    ("1364 strength grammar (the .8b.1 1364->1800 edition law's origin - "
     "these placements ARE grammar in 1364-2005): A.2.1.3 net_declaration "
     "[ drive_strength ] with a decl assignment, A.6.1 continuous_assign "
     "[ drive_strength ], A.3.1 gate [ drive_strength ] with A.2.2.2 "
     "( highz1 , strength0 ) a listed alternative, A.3.2 pullup_strength "
     "( strength1 )",
     ["test_06_01_01_1", "test_06_01_02_1", "test_07_01_02_1",
      "test_07_08_00_1"]),
    ("scalared/vectored net declaration - A.2.1.3 net_declaration "
     "'net_type [ vectored | scalared ] [ signed ] range' alternative",
     ["test_04_03_02_1"]),
    ("UDP bodies and instances - A.5.3 edge_indicator '( level level )' and "
     "edge_symbol '*' inside edge_input_list, level_symbol 'b', next_state "
     "'-'; A.5.4 udp_instantiation [ delay2 ] with '# identifier' a legal "
     "delay_value (and the instance is module-instantiation-shaped either "
     "way)",
     ["test_08_04_00_1", "test_08_06_00_1", "test_08_07_00_1"]),
    ("PATHPULSE$ - the KNOWN_TEXT_BUGS candidate ruled STILL PARSEABLE: "
     "'PATHPULSE$ = 3;' matches A.2.4 specparam_assignment's "
     "'specparam_identifier = constant_mintypmax_expression' alternative "
     "because A.9.3 simple_identifier admits '$' and Annex B does not "
     "reserve PATHPULSE$ (the identifier escape-hatch law, the A.8.2 "
     "system_tf_call mirror); the (2,9) rows match pulse_control_specparam; "
     "(clr, pre *> q) is an A.7.2 full_path_description",
     ["test_14_06_01_1"]),
    ("timing checks - A.7.5.1 $width ( controlled_reference_event , "
     "timing_check_limit ) with threshold/notifier optional; A.7.5.3 "
     "edge_control_specifier 'edge [ 01 , 0x , x1 ]' via edge_descriptor "
     "01 | z_or_x zero_or_one | zero_or_one z_or_x",
     ["test_15_04_00_1"]),
    ("`begin_keywords \"1364-2001\" - 19.11 selects the 1364-2001 keyword "
     "set so 'uwire' is an ordinary identifier in scope; section 19: "
     "directives may appear anywhere in the source description (the "
     "sv-lane 22.14 begin_keywords mirror - directive-aware keyword "
     "selection is parser duty)",
     ["test_19_11_00_1"]),
]

ISPRAS_1364_POSITIVE_PINNED = {}
for _basis, _names in ISPRAS_1364_POSITIVE_CLUSTERS:
    for _n in _names:
        _key = f"ieee-1364-2005/{_n}.v"
        if _key in ISPRAS_1364_POSITIVE_PINNED:
            raise SystemExit(f"duplicate .8c.2 ispras POSITIVE pin: {_key}")
        ISPRAS_1364_POSITIVE_PINNED[_key] = (
            "must_accept", "ispras-1364 POSITIVE pinned .8c.2 "
            "(KNOWN_TEXT_BUGS re-adjudicated, no errata flip): " + _basis)
if len(ISPRAS_1364_POSITIVE_PINNED) != 17:
    raise SystemExit(
        f".8c.2 ispras POSITIVE pin table holds "
        f"{len(ISPRAS_1364_POSITIVE_PINNED)} entries, expected exactly the "
        "17-row failing-POSITIVE population")


def expect_ispras_v2005(relpath: str, text: str):
    """ispras ieee-1364-2005/ half under the verilog_2005 profile."""
    p = relpath.replace("\\", "/")
    m = ISPRAS_TYPE_RE.search(text)
    ttype = m.group(1) if m else ""
    if ttype == "POSITIVE":
        pinned = ISPRAS_1364_POSITIVE_PINNED.get(p)
        if pinned:
            return pinned
        return ("must_accept",
                "ispras-1364: '// ! TYPE: POSITIVE' clause-keyed valid "
                "example (a failing residue re-adjudicates against the "
                "suite's KNOWN_TEXT_BUGS errata in leaf .8c.2 - "
                "committed-text-over-intent)")
    if ttype == "NEGATIVE":
        pinned = ISPRAS_1364_NEGATIVE_PINNED.get(p)
        if pinned:
            return pinned
        return ("out_of_scope_with_cause:negative_stage_triage_v2005",
                "ispras-1364: TYPE NEGATIVE outside the .8c.1 pinned pair "
                "(added upstream after the vendored pin?)")
    if ttype == "VARYING":
        return ("out_of_scope_with_cause:impl_varying_v2005",
                "ispras-1364: TYPE VARYING - implementation-dependent "
                "verdict by suite contract")
    return ("chained_only",
            "ispras-1364: no '// ! TYPE:' key (multi-file companion under "
            "parts/) - not a standalone keyed unit")


def expect_v2005(suite: str, rel: str, text: str, ividx):
    """Expected verdict for one v2005-lane row (leaf .8c.1) - suite metadata
    / upstream driver conventions only, judged against IEEE 1364-2005."""
    if suite == "ispras-sv-tests":
        return expect_ispras_v2005(rel, text)
    if suite == "sv2v":
        return ("must_accept",
                "sv2v: committed conversion GOLDEN - valid Verilog-2005 "
                "output by suite contract (the tool's own emitted text)")
    if suite == "iverilog":
        return ividx.expect_v2005(rel)
    raise ValueError(f"suite {suite!r} has no v2005-lane key source")


# ivtest vlg CE-without-gold stage pins (leaf .8c.2): the 176 v2005-lane rows
# whose upstream key (regress-vlg.list CE entry or vvp_tests descriptor)
# encodes the compile-error INTENT but not its stage. Every file was read and
# adjudicated against the IEEE 1364-2005 Annex A BNF (in-repo
# docs/verilog/2005/md, the full Annex A dump re-verified verbatim before
# pinning) plus the section 19 directive rules. must_accept = the committed
# text is BNF-parseable and the intended invalidity is a prose "shall"
# (semantic/elaboration stage, or an iverilog-specific check); must_reject =
# the text violates the 1364-2005 BNF (several are the edition law's v2005
# face: 1800-only syntax judged in this verilog_2005 lane); the
# out_of_scope_with_cause:svpp_owned_v2005 pair are DIRECTIVE-stage
# invalidities (section 19 preprocessor grammar/value rules - the
# preprocessor lane owns them, not the Annex A parser). Clusters keep the
# shared LRM ground in one place; the flatten below audits count+duplicates.
IVTEST_VLG_CE_STAGE_CLUSTERS = [
    # --- parse-level rejects (Annex A BNF; edition-law faces named) --------
    ("must_reject",
     "empty tf-call parentheses - A.8.2 function_call and A.6.9 task_enable "
     "both require '( expression { , expression } )' with at least one "
     "expression when parentheses are present (1800's empty "
     "list_of_arguments is the edition contrast); task_in_expr_fail also "
     "declares 'int x;' in an unnamed begin (no 1364 production)",
     ["function4", "task_nonansi_fail5", "task_nonansi_fail8",
      "task_in_expr_fail"]),
    ("must_reject",
     "declarations in an unnamed block - A.6.3 seq_block/par_block admit "
     "block_item_declarations only after ': block_identifier' "
     "(begin [ : block_identifier { block_item_declaration } ] "
     "{ statement } end; 1800 unnamed-block declarations are the edition "
     "contrast)",
     ["unnamed_block_var_decl", "unnamed_fork_var_decl"]),
    ("must_reject",
     "more than one statement in a task body - A.2.7 task_declaration is "
     "'{ task_item_declaration } statement_or_null endtask' (exactly one "
     "statement; 1800's { statement_or_null } body is the edition "
     "contrast): 'y = x; $display(...);' needs a begin/end",
     ["task_port_range_mismatch"]),
    ("must_reject",
     "default values in an input/inout ANSI port list - A.2.1.2 input/inout "
     "declarations end in A.2.3 list_of_port_identifiers (bare identifiers, "
     "no '= constant_expression'; only list_of_variable_port_identifiers "
     "under 'output reg'/output_variable_type has defaults - and port "
     "defaults per se are 1800-2009+ syntax)",
     ["module_input_port_list_def", "module_inout_port_list_def"]),
    ("must_reject",
     "variable-typed input/inout module ports - A.2.1.2 input_declaration/"
     "inout_declaration admit only '[ net_type ] [ signed ] [ range ]' "
     "(reg/time/integer match no alternative; the 1800 variable-port form "
     "is the edition contrast)",
     ["module_input_port_type", "module_inout_port_type"]),
    ("must_reject",
     "port_declaration inside a portless module - A.1.2 module_declaration: "
     "the only form whose body admits port_declaration (module_item) "
     "requires a parenthesized list_of_ports; 'module test;' takes the "
     "second form whose body is { non_port_module_item }",
     ["module_port_range_mismatch"]),
    ("must_reject",
     "non-generate items inside a generate block - A.4.2 generate_block "
     "holds { module_or_generate_item } and A.1.4 places "
     "parameter_declaration, specparam_declaration and specify_block in "
     "non_port_module_item ONLY (localparam is the admitted generate-scope "
     "form; 1800's generate-scope parameter is the edition contrast)",
     ["generate_specify", "generate_specparam", "parameter_in_generate1"]),
    ("must_reject",
     "gate-instance terminal arity/emptiness - A.3.1 "
     "n_output_gate_instance requires '( output_terminal { , "
     "output_terminal } , input_terminal )' (>= 2 terminals for buf/not) "
     "and A.3.3 terminals are net_lvalues/expressions which are never "
     "empty; br_gh152 additionally has an unclosed instance parenthesis "
     "with a keyword in terminal position",
     ["pr2395378a", "pr2395378b", "pr2395378c", "pr1763333", "br_gh152"]),
    ("must_reject",
     "malformed tf port declarations - A.2.7/A.2.6 tf_input_declaration "
     "ends in list_of_port_identifiers (bare identifiers): 'input "
     "make_me_crash i;' juxtaposes two identifiers with no comma and "
     "'input bit_array[3:0]' carries a per-identifier dimension no 1364 "
     "production admits; br1015a additionally uses '^=' which exists in "
     "neither A.6.2 blocking_assignment nor A.8.6 (1800 compound "
     "assignment - edition contrast)",
     ["br_gh163", "br1015a"]),
    ("must_reject",
     "source-level task/function declarations - A.1.2 description ::= "
     "module_declaration | udp_declaration | config_declaration (the 1800 "
     "$unit compilation-unit scope is the edition contrast; the files "
     "state the 1364-2005 CE intent verbatim)",
     ["br_gh25a", "br_gh25b"]),
    ("must_reject",
     "zero-sized literal - A.8.7 size ::= non_zero_unsigned_number, so "
     "'0'b0' cannot lex as one sized literal and the token split "
     "'0' + ''b0' juxtaposes two primaries with no operator (no "
     "production)",
     ["br_gh60a"]),
    ("must_reject",
     "hierarchical name in a constant context - A.8.4 constant_primary has "
     "no hierarchical alternative (parameter_identifier ::= identifier, "
     "one undotted name), so 'parameter WIDTH = dut.WIDTH;' matches no "
     "production",
     ["pr2792883"]),
    ("must_reject",
     "'reg real [1:0] a;' - A.2.1.3 reg_declaration is 'reg [ signed ] "
     "[ range ] list_of_variable_identifiers' and 'real' is a keyword, "
     "not a variable_identifier",
     ["pr3112073a"]),
    ("must_reject",
     "mixed ordered and named port connections - A.4.1 "
     "list_of_port_connections is homogeneous (all ordered_port_connection "
     "or all named_port_connection; the tf mirror of the .8b.3 A.8.2 rule)",
     ["pr2051975"]),
    # --- directive-stage invalidities (section 19 - preprocessor lane) -----
    ("out_of_scope_with_cause:svpp_owned_v2005",
     "malformed conditional directives - 19.4 Syntax 19-4 requires "
     "`ifdef/`ifndef/`elsif to carry a text_macro_identifier; the bare "
     "forms violate the DIRECTIVE grammar, a preprocessor-stage error the "
     "svpp lane owns (Annex A has no directive productions)",
     ["ifdef_fail"]),
    ("out_of_scope_with_cause:svpp_owned_v2005",
     "directive value rule - 19.8: 'The time_precision argument shall be "
     "at least as precise as the time_unit argument'; '`timescale 1ns/"
     "10ns' violates a DIRECTIVE-value shall, a preprocessor-stage error "
     "the svpp lane owns",
     ["timescale3"]),
    # --- parse-level accepts (BNF-parseable; invalidity is prose/semantic
    #     stage or an iverilog-specific check) ------------------------------
    ("must_accept",
     "procedural assignment/timing forms - A.6.2 blocking/nonblocking "
     "assignment with [ delay_or_event_control ] (A.6.5 '# delay_value', "
     "'# ( mintypmax_expression )', '@ hierarchical_event_identifier', "
     "'@ ( event_expression )'), A.6.2 procedural_continuous_assignments "
     "(assign/force on variable_assignment/net_assignment, deassign/"
     "release on lvalues), A.6.5 disable_statement and event_trigger; the "
     "iverilog CE is its always-without-delay/infinite-loop or "
     "unsupported-construct check, not a 1364 parse rule (always3.1.2I's "
     "spaced '5'h 0' is 3.5.1-legal - the ispras 3.5.1 mirror)",
     ["always3.1.10A", "always3.1.1A", "always3.1.1B", "always3.1.2A",
      "always3.1.2B", "always3.1.2C", "always3.1.2D", "always3.1.2E",
      "always3.1.2F", "always3.1.2G", "always3.1.2H", "always3.1.2I",
      "always3.1.3A", "always3.1.3B", "always3.1.3C", "always3.1.3D",
      "always3.1.3E", "always3.1.3F", "always3.1.3G", "always3.1.3H",
      "always3.1.3J", "always3.1.9A", "always3.1.9B"]),
    ("must_accept",
     "event arrays and array-word assignment targets - A.2.3 "
     "list_of_event_identifiers carries { dimension }, A.6.5 event_trigger "
     "is '-> hierarchical_event_identifier { [ expression ] } ;' and "
     "A.8.5 variable_lvalue admits '{ [ expression ] }' word selects with "
     "intra-assignment delay",
     ["event_array", "pr2597278b"]),
    ("must_accept",
     "denotation-dependent constancy (the .8b.3 fn-15 mirror) - "
     "constant_expression/genvar contexts reach A.8.4 constant_primary's "
     "parameter_identifier ::= identifier, which is denotation-blind: "
     "whether the name denotes a variable, an undeclared symbol, a "
     "self/circular parameter or a genvar is semantic/elaboration stage "
     "(A.4.2 genvar_initialization/iteration and A.9.1 attr_spec "
     "likewise)",
     ["check_constant_1", "check_constant_2", "check_constant_3",
      "check_constant_4", "check_constant_5", "check_constant_6",
      "check_constant_7", "check_constant_8", "check_constant_9",
      "check_constant_10", "check_constant_11", "check_constant_12",
      "check_constant_13", "check_constant_14", "check_constant_15",
      "check_constant_16", "check_constant_17", "check_constant_18",
      "check_constant_19", "check_constant_20", "br_gh142", "pr2039632",
      "pr3061015a", "pr3061015b", "pr3061015c"]),
    ("must_accept",
     "replication/width value rules are prose - A.8.1 "
     "multiple_concatenation ::= { constant_expression concatenation } is "
     "width-value-blind; zero or x replication constants and zero-width "
     "indexed part-selects (A.8.3 range_expression '+:' form) are 5.1.14/"
     "value-rule semantics",
     ["concat_zero_wid_fail", "repl_zero_wid_fail", "zero_repl_fail",
      "pr1925363a", "pr1925363b", "pr1971662a", "pr1971662b",
      "pr3549328"]),
    ("must_accept",
     "identifier-resolution/arity semantics - undeclared names, missing "
     "functions/modules, wrong task/function argument counts and "
     "upward-scope misses are elaboration stage; every call carries >= 1 "
     "expression so A.8.2/A.6.9 are satisfied",
     ["br924", "br961a", "br982", "br982a", "br982b", "br_gh26",
      "br_ml20150321", "hier_ref_error", "pr2051694", "pr2528915",
      "pr3270320", "scope2b"]),
    ("must_accept",
     "duplicate-name/link semantics - duplicate identifiers in one "
     "declaration list, duplicate module definitions, duplicate scope "
     "names and port-vs-variable redeclaration pairs are each BNF-valid "
     "declaration sequences; the clash is semantic",
     ["redef_net_error", "redef_reg_error", "pr1938138", "pr1833754",
      "pr1704013"]),
    ("must_accept",
     "specify path width coherence is prose - the parallel-connection "
     "width rule is a semantic check; '(in => out) = 2;' is an A.7.2 "
     "parallel_path_description",
     ["par_mismatch"]),
    ("must_accept",
     "lvalue-select stage rules (the .8c.1 12.3.3 semantic-stage mirror) - "
     "A.8.5 variable_lvalue/net_lvalue admit bit/word/part selects "
     "(range_expression msb:lsb and +:/-: forms; identifiers in "
     "constant_expression slots are denotation-blind), and A.8.4 primary "
     "admits 'identifier [ range_expression ]'; the procedural-continuous/"
     "force lvalue-class and memory-select bans are 9.3 prose",
     ["array5", "array_lval_select3b", "array_lval_select4b",
      "undef_lval_select3b", "undef_lval_select3c", "undef_lval_select4b",
      "undef_lval_select4c", "sv_lval_idx_part_invalid_base_down_fail",
      "pr1735724", "readmemh5"]),
    ("must_accept",
     "module non-ANSI port/variable redeclaration coherence - each "
     "declaration is individually A.2.1.2/A.2.1.3 BNF; duplicate or "
     "type/range-conflicting redeclaration of a port signal is 12.3.3 "
     "semantics (the ispras test_12_03_03_2 .8c.1 mirror)",
     ["module_nonansi_fail1", "module_nonansi_fail2", "module_nonansi_fail3",
      "module_nonansi_fail4", "module_nonansi_fail5", "module_nonansi_fail6",
      "module_nonansi_fail7", "module_nonansi_fail8", "module_nonansi_fail9",
      "module_nonansi_fail10", "module_nonansi_fail11",
      "module_nonansi_fail12", "module_nonansi_fail13",
      "module_nonansi_integer_fail", "module_nonansi_time_fail",
      "module_nonansi_vec_fail1", "module_nonansi_vec_fail2",
      "module_nonansi_vec_fail3"]),
    ("must_accept",
     "trailing null port + direction semantics - A.1.3 port ::= "
     "[ port_expression ] may be EMPTY, so '(a, b, )' is a legal "
     "list_of_ports ending in a null port; duplicate port declarations "
     "and wrong directions are semantic",
     ["port-test3", "port-test4a", "port-test4b"]),
    ("must_accept",
     "task non-ANSI port/variable redeclaration coherence - each "
     "task_item_declaration is A.2.7 BNF (single trailing statement "
     "each); duplicate/conflicting port-variable redeclaration inside a "
     "task is semantics (the module_nonansi mirror)",
     ["task_nonansi_fail1", "task_nonansi_fail2", "task_nonansi_fail3",
      "task_nonansi_fail4", "task_nonansi_fail6", "task_nonansi_fail7",
      "task_nonansi_fail9", "task_nonansi_fail10", "task_nonansi_fail11",
      "task_nonansi_integer_fail", "task_nonansi_real_fail",
      "task_nonansi_time_fail", "task_nonansi_vec_fail1",
      "task_nonansi_vec_fail2", "task_nonansi_vec_fail3"]),
    ("must_accept",
     "null-statement/empty-connection forms - A.6.4 statement_or_null "
     "admits ';' as an if-body and A.4.1 named_port_connection is "
     "'. port_identifier ( [ expression ] )' with the expression "
     "omissible",
     ["no_if_statement", "contrib8.3"]),
    ("must_accept",
     "real operands in a concatenation are a value rule - A.8.1 "
     "concatenation admits any expression (real_number is an A.8.4 "
     "primary); the real-operand ban is 5.1.14 prose",
     ["real_concat_invalid2"]),
    ("must_accept",
     "automatic-task access rules are prose - hierarchical references "
     "into automatic tasks, nonblocking/procedural-continuous "
     "assignments on automatic variables (9.2.2/10.2.2 'shall not') are "
     "semantic; every construct is A.2.7/A.6.2 BNF (automatic_error3's "
     "`begin_keywords \"1364-2005\" is a legal 19.11 directive)",
     ["automatic_error1", "automatic_error2", "automatic_error3",
      "automatic_error5", "automatic_error6", "automatic_error7",
      "automatic_error8", "automatic_error9", "automatic_error10"]),
    ("must_accept",
     "generate parse-valid forms - A.4.2 loop_generate_construct nests as "
     "its own generate_block and A.1.4 admits initial_construct in "
     "generate scope (genvar reuse in nested loops is 12.4 semantics); "
     "A.9.3 hierarchical_identifier admits '[ constant_expression ]' on "
     "path segments (generate-scope indexed references)",
     ["br_gh533", "pr1988302b"]),
    ("must_accept",
     "parameter-override resolution semantics - overriding a nonexistent/"
     "local/body parameter via #(.) or defparam parses (A.4.1 "
     "named_parameter_assignment, A.2.4 defparam_assignment); the "
     "override legality is elaboration",
     ["parameter_override_invalid1", "parameter_override_invalid2",
      "parameter_override_invalid3", "parameter_override_invalid4",
      "parameter_override_invalid5", "parameter_override_invalid6"]),
    ("must_accept",
     "link-stage instance discrimination - '#(10, 20, 30)' after an "
     "identifier parses as A.4.1 parameter_value_assignment (the BNF "
     "cannot know mux2 names a UDP - the udp-vs-module split is link "
     "stage), and recursive module instantiation depth is elaboration; "
     "$clog2 in a parameter is A.8.4 constant_system_function_call "
     "(1364-2005-added)",
     ["udp_delay_fail", "pr2728812b", "pr2728812c"]),
    ("must_accept",
     "directive placement is unrestricted - section 19: 'These directives "
     "may appear anywhere in the source description'; 19.8 defines "
     "`timescale meaning for the modules that follow and states NO "
     "placement shall, so an in-module-body `timescale is not a 1364 "
     "error (iverilog strictness) - the parser must tolerate it",
     ["no_timescale_in_module"]),
]

IVTEST_VLG_CE_STAGE_PINNED = {}
for _cls, _basis, _names in IVTEST_VLG_CE_STAGE_CLUSTERS:
    for _n in _names:
        _key = f"ivltests/{_n}.v"
        if _key in IVTEST_VLG_CE_STAGE_PINNED:
            raise SystemExit(f"duplicate .8c.2 vlg stage pin: {_key}")
        IVTEST_VLG_CE_STAGE_PINNED[_key] = (
            _cls, "ivtest vlg CE pinned .8c.2: " + _basis)
if len(IVTEST_VLG_CE_STAGE_PINNED) != 176:
    raise SystemExit(
        f".8c.2 vlg stage-pin table holds {len(IVTEST_VLG_CE_STAGE_PINNED)} "
        "entries, expected exactly the 176-row CE-without-gold population")


IVTEST_TYPES = {"normal", "CE", "CO", "EF", "RE", "NI"}

# ivtest CE-without-gold stage pins (leaf .8b.3): the 283 CE rows whose upstream
# key (regress-sv.list entry or vvp_tests descriptor) encodes the compile-error
# INTENT but not its stage. Every file was read and adjudicated against the
# IEEE 1800-2017 LRM's BNF-vs-prose split (in-repo docs/systemverilog/2017/md,
# Annex A dump in section-41-data-read-api.md, incl. the normative Annex A
# footnotes per the banked .8b.2 footnote law). must_accept = the committed
# text is BNF-parseable and the intended invalidity is a prose "shall"
# (semantic/elaboration stage, or an iverilog-specific check); must_reject =
# the text violates the Annex A BNF, a normative footnote, or (edition law)
# uses 1800-2023-only syntax judged in this sv_2017 lane. Clusters keep the
# shared LRM ground in one place; the flatten below audits count + duplicates.
IVTEST_CE_STAGE_CLUSTERS = [
    # --- parse-level rejects (Annex A BNF / normative footnotes) -------------
    ("must_reject",
     "single-expression / '$' / unsized packed dimensions - A.2.5 "
     "packed_dimension ::= [constant_range] | unsized_dimension; a lone "
     "expression is not a constant_range, '$' is not a constant_primary "
     "(A.8.4), and footnote 20 permits unsized '[]' only as the sole packed "
     "dimension of a DPI import declaration",
     ["br_ml20181012a", "br_ml20181012b", "br_ml20181012c",
      "packed_dims_invalid_class", "packed_dims_invalid_module",
      "enum_dims_invalid"]),
    ("must_reject",
     "enum base type outside A.2.2.1 enum_base_type - 'real'/'string' "
     "keywords match no alternative, and integer_vector_type admits at most "
     "ONE optional packed_dimension (two-dimension base has no production)",
     ["enum_base_fail_real1", "enum_base_fail_string1",
      "enum_base_fail_range3"]),
    ("must_reject",
     "declaration in a structurally illegal context - the A.4.2 "
     "generate_item -> module_or_generate_item -> module_common_item chain "
     "and A.1.11 package_or_generate_item_declaration admit neither "
     "module_declaration nor timeunits_declaration inside generate blocks; "
     "A.1.7 non_port_program_item admits neither module_declaration nor "
     "always_construct inside program blocks",
     ["generate_module", "generate_timeunit", "program5a", "program_hello2"]),
    ("must_reject",
     "net declaration dimensions - A.2.4 net_decl_assignment admits only "
     "unpacked_dimension ([constant_range] | [constant_expression]); '[]' is "
     "unsized_dimension (not admitted) and '$' is not a constant_primary, so "
     "'wire x[];' / 'wire x[$];' match no production",
     ["net_darray_fail", "net_queue_fail"]),
    ("must_reject",
     "normative Annex A footnote violations (the .8b.2 footnote law) - "
     "fn 10: 'automatic' is illegal in a data_declaration outside a "
     "procedural context, and omitting the explicit data_type before a "
     "list_of_variable_decl_assignments is illegal unless 'var' is used; "
     "fn 18: a defaultless parameter is legal only within a "
     "parameter_port_list",
     ["parameter_no_default_fail2", "sv_package_implicit_var1",
      "sv_package_implicit_var2", "sv_package_lifetime_fail"]),
    ("must_reject",
     "positional argument after a named argument - A.8.2 list_of_arguments: "
     "once a '. identifier ( ... )' member appears, only further named "
     "members may follow (the positional-then-named order IS legal, the "
     "reverse is not)",
     ["sv_named_arg_base_fail4", "sv_named_arg_chained_fail4",
      "sv_named_arg_func_fail4", "sv_named_arg_new_fail4",
      "sv_named_arg_task_fail4"]),
    ("must_reject",
     "1800-2023-only syntax judged in the sv_2017 lane (edition law, the "
     "opposite face of the .8b.1 1364->1800 rule) - 2017 A.2.2.1 "
     "struct_union ::= struct | union [tagged] (no 'soft'), and 2017 "
     "parameter_declaration admits 'parameter type list_of_type_assignments' "
     "with no enum/struct/union/class restriction keyword",
     ["sv_soft_packed_union_fail1",
      "sv_type_param_restrict_class_fail1", "sv_type_param_restrict_class_fail2",
      "sv_type_param_restrict_enum_fail1", "sv_type_param_restrict_enum_fail2",
      "sv_type_param_restrict_struct_fail1", "sv_type_param_restrict_struct_fail2",
      "sv_type_param_restrict_union_fail1", "sv_type_param_restrict_union_fail2"]),
    ("must_reject",
     "assorted single-production BNF violations - 'real [1:0]' (A.2.2.1 "
     "data_type gives non_integer_type NO packed_dimension); non-ANSI "
     "'input x;' inside 'function new' (A.1.9 class_constructor_declaration "
     "body admits only block_item_declaration, no tf_port_declaration); a "
     "block_item_declaration after a null statement (A.6.3 seq_block puts "
     "all declarations before statements); 'defparam m.T = real;' (A.2.4 "
     "defparam_assignment RHS is a constant_mintypmax_expression - the "
     "keyword 'real' matches no expression production); 'for (var [7:0] i "
     "= 0;...)' (A.6.8 for_variable_declaration requires an explicit "
     "data_type); '#( inout var x )' (A.1.3 parameter_port_declaration "
     "admits no port direction); non-ANSI 'inout var x;' (A.2.1.2 "
     "inout_declaration takes a net_port_type, which has no var form)",
     ["sv_array_cassign_fail5", "sv_class_constructor_fail",
      "sv_declaration_after_null_statement_fail", "sv_type_param_fail2",
      "sv_var_for_fail", "sv_var_module_inout1", "sv_var_module_inout2"]),
    # --- parse-level accepts (prose "shall" rules / semantics / tool checks) -
    ("must_accept",
     "always_comb/always_ff/always_latch content and sensitivity rules - "
     "A.6.2 always_construct ::= always_keyword statement; the bans on "
     "blocking timing controls, event controls, fork/join and the "
     "exactly-one-event-control requirement are 9.2.2.2-9.2.2.4 prose",
     ["always_comb_fail", "always_comb_fail3", "always_comb_fail4",
      "always_ff_fail", "always_ff_fail2", "always_ff_fail3",
      "always_ff_fail4", "always_ff_no_sens", "always_latch_fail",
      "always_latch_fail3", "always_latch_fail4", "always_latch_no_sens"]),
    ("must_accept",
     "'always fork ... join_any/join_none' zero-time loop - ordinary A.6.2/"
     "A.6.3 statement BNF; the CE is iverilog's always-does-not-advance-time "
     "diagnostic (tool/semantic stage)",
     ["always4A", "always4B"]),
    ("must_accept",
     "assignments touching automatic-lifetime variables (NBA to automatic "
     "struct field / class handle, procedural assign/force referencing an "
     "automatic) - 6.21/10.6.1 prose; the statements are ordinary BNF",
     ["automatic_error14", "automatic_error15", "automatic_error16",
      "automatic_error17", "automatic_error18",
      "sv_assign_pattern_auto_force_fail"]),
    ("must_accept",
     "intended elaboration-time $fatal - A.1.4 module_common_item admits "
     "elaboration_system_task; the compile error IS the intended semantics",
     ["br_gh1029"]),
    ("must_accept",
     "enum value/compatibility semantics (implicit casts to enum, duplicate "
     "or X/size-mismatched enum constants, $time as an enum value) - 6.19.x "
     "prose value rules; declarations and assignments are ordinary BNF",
     ["br_gh130a", "br_gh386c", "enum_compatibility_fail1",
      "enum_compatibility_fail2", "enum_compatibility_fail3",
      "enum_compatibility_fail4", "enum_compatibility_fail5",
      "enum_compatibility_fail6", "enum_compatibility_fail7",
      "enum_compatibility_fail8", "enum_test3", "enum_test5", "enum_test6",
      "enum_test7", "pr3366217g"]),
    ("must_accept",
     "super.new not the first constructor statement - 8.15 prose ordering "
     "rule; every statement is ordinary BNF",
     ["br_gh390a"]),
    ("must_accept",
     "name/function resolution semantics (undefined function in a constant "
     "expression, hierarchical access to imported identifiers, "
     "package-scoped lookup crossing the package boundary, calling a "
     "variable/task as a function, ambiguous wildcard imports) - 26.3/26.4 "
     "and 13.x resolution rules, all post-parse",
     ["br_gh699", "sv_import_hier_fail1", "sv_import_hier_fail2",
      "sv_import_hier_fail3", "sv_ps_function_fail1", "sv_ps_function_fail2",
      "sv_ps_function_fail3", "sv_ps_hier_fail1", "sv_ps_hier_fail2",
      "sv_wildcard_import5"]),
    ("must_accept",
     "member select on a type without that member - member resolution "
     "semantics; hierarchical/member lvalues are A.8.5 BNF",
     ["br_gh823a", "br_gh823b", "sv_bad_member_lval_proc_fail"]),
    ("must_accept",
     "generate-loop index out of the target's declared range - elaboration "
     "arithmetic; the loop generate construct is A.4.2 BNF",
     ["br_gh840a", "br_gh840b"]),
    ("must_accept",
     "assignment type-compatibility semantics (string to bit-vector, scalar "
     "to array, element-type / dimension-count / size mismatches across "
     "unpacked array, dynamic array and queue assignments) - 6.22/7.6/10.7 "
     "prose compatibility rules; every assignment is ordinary BNF",
     ["br_ml20180227", "sv_array_assign_fail1", "sv_array_assign_fail2",
      "sv_array_assign_single_fail1", "sv_array_cassign_fail1",
      "sv_array_cassign_fail2", "sv_array_cassign_fail3",
      "sv_array_cassign_fail4", "sv_array_cassign_fail6",
      "sv_array_cassign_fail7", "sv_array_cassign_fail8",
      "sv_array_cassign_fail9", "sv_array_cassign_fail10",
      "sv_array_cassign_fail11", "sv_array_cassign_single_fail1",
      "sv_darray_assign_fail1", "sv_darray_assign_fail2",
      "sv_darray_assign_fail3", "sv_darray_assign_fail4",
      "sv_darray_assign_fail5", "sv_darray_assign_fail6",
      "sv_queue_assign_fail1", "sv_queue_assign_fail2",
      "sv_queue_assign_fail3", "sv_queue_assign_fail4",
      "sv_queue_assign_fail5", "sv_queue_assign_fail6"]),
    ("must_accept",
     "'reg illegal[0];' - A.2.5 unpacked_dimension admits a single "
     "[constant_expression]; a zero-size array is a semantic error",
     ["br_ml20181012d"]),
    ("must_accept",
     "cast operand/size-value semantics (string/array/queue/darray to real, "
     "zero/negative/undefined size casts, void'() operand rules) - 6.24.1/"
     "13.4.1 prose; casting_type and the cast forms are A.8.4 BNF",
     ["cast_real_invalid1", "cast_real_invalid2", "cast_real_invalid3",
      "cast_real_invalid4", "size_cast_fail1", "size_cast_fail2",
      "size_cast_fail3", "sv_void_cast_fail1", "sv_void_cast_fail2",
      "sv_void_cast_fail3"]),
    ("must_accept",
     "enum base type via type_identifier - A.2.2.1 enum_base_type admits "
     "type_identifier [packed_dimension]; footnote 15's legality condition "
     "depends on what the type_identifier DENOTES (integer atom/vector vs "
     "array/enum/real/string/struct), i.e. resolution-dependent = semantic "
     "stage",
     ["enum_base_fail_array", "enum_base_fail_darray", "enum_base_fail_enum",
      "enum_base_fail_queue", "enum_base_fail_real2",
      "enum_base_fail_string2", "enum_base_fail_struct",
      "enum_base_fail_range1", "enum_base_fail_range2"]),
    ("must_accept",
     "enum member name colliding with another class-scope symbol - scope "
     "population semantics",
     ["enum_in_class_name_coll"]),
    ("must_accept",
     "final-block content restrictions (task enable, non-blocking "
     "assignment) - 9.2.3 prose gives final blocks function-like statement "
     "restrictions; A.6.2 final_construct ::= final function_statement and "
     "A.2.6 function_statement ::= statement",
     ["final_nested_block_task_fail", "program3b"]),
    ("must_accept",
     "tf argument-binding semantics (too many/empty arguments, nonexistent "
     "or duplicate named arguments, name+positional double binding, empty "
     "actual without a default, built-in queue/string method arity) - "
     "13.5.3/13.5.4 prose; A.8.2 list_of_arguments admits empty slots, "
     "all-named calls and the positional-then-named order",
     ["func_empty_arg_fail1", "func_empty_arg_fail2", "func_empty_arg_fail3",
      "func_empty_arg_fail4",
      "sv_named_arg_base_fail1", "sv_named_arg_base_fail2",
      "sv_named_arg_base_fail3", "sv_named_arg_base_fail5",
      "sv_named_arg_chained_fail1", "sv_named_arg_chained_fail2",
      "sv_named_arg_chained_fail3", "sv_named_arg_chained_fail5",
      "sv_named_arg_func_fail1", "sv_named_arg_func_fail2",
      "sv_named_arg_func_fail3", "sv_named_arg_func_fail5",
      "sv_named_arg_new_fail1", "sv_named_arg_new_fail2",
      "sv_named_arg_new_fail3", "sv_named_arg_new_fail5",
      "sv_named_arg_task_fail1", "sv_named_arg_task_fail2",
      "sv_named_arg_task_fail3", "sv_named_arg_task_fail5",
      "sv_queue_method_insert_too_few_arg_fail",
      "sv_queue_method_insert_too_many_arg_fail",
      "sv_queue_method_push_back_too_few_arg_fail",
      "sv_queue_method_push_back_too_many_arg_fail",
      "sv_queue_method_push_front_too_few_arg_fail",
      "sv_queue_method_push_front_too_many_arg_fail",
      "sv_string_method_substr_too_few_arg_fail"]),
    ("must_accept",
     "void function/task in an expression, return-with-value in a void "
     "function - 13.3/13.4.1 prose; calls and jump_statement are ordinary "
     "BNF",
     ["func_void_in_expr_fail", "function11", "sv_class_task_expr_fail"]),
    ("must_accept",
     "port-connection resolution/assignability semantics (implicit named "
     "connection to a nonexistent signal or port, non-assignable output "
     "actuals) - 23.3.2/23.3.3 elaboration rules; connections are A.4.1.1 "
     "BNF",
     ["implicit-port2", "implicit-port3", "implicit-port6",
      "module_port_array_fail1", "sv_byte_array_string_fail3"]),
    ("must_accept",
     "parameter/localparam override semantics (overriding a localparam by "
     "name or defparam, defparam into a generate block, defaultless "
     "port-list parameter left unoverridden - fn 18 makes the port-list "
     "omission itself legal) - 6.20.4/23.10.x elaboration rules",
     ["localparam_implicit2", "localparam_implicit3",
      "parameter_override_invalid7", "parameter_override_invalid8",
      "parameter_in_generate2", "parameter_no_default_fail1"]),
    ("must_accept",
     "non-ANSI port / tf-port redeclaration coherence (implicit packed "
     "dimensions later redeclared as atom2/enum/packed-array/real/struct "
     "typed variables) - 23.2.2.2/13.3 prose completeness rules; each "
     "declaration is ordinary BNF",
     ["module_nonansi_atom2_fail", "module_nonansi_enum_fail",
      "module_nonansi_parray_fail", "module_nonansi_real_fail",
      "module_nonansi_struct_fail", "task_nonansi_atom2_fail",
      "task_nonansi_enum_fail", "task_nonansi_parray_fail",
      "task_nonansi_struct_fail"]),
    ("must_accept",
     "block/fork end-label mismatch - the A.6.3 end label is any "
     "block_identifier; the must-match rule is 9.3.4 prose (the .8b.1 "
     "end-label law)",
     ["named_begin_fail", "named_fork_fail"]),
    ("must_accept",
     "statement label combined with a block name - statement ::= "
     "[block_identifier :] statement_item and the seq/par block's own "
     "[: block_identifier] are independent BNF optionals; the "
     "one-or-the-other rule is 9.3.5 prose",
     ["sv_block_prefix_name_diff_fail", "sv_block_prefix_name_same_fail",
      "sv_fork_prefix_name_diff_fail", "sv_fork_prefix_name_same_fail"]),
    ("must_accept",
     "'@(edge e)' on a named event - A.6.5 event_expression admits "
     "[edge_identifier] expression; the named-event restriction is prose",
     ["named_event_edge_fail"]),
    ("must_accept",
     "net of class/string type - A.2.1.3 net_declaration admits any "
     "data_type; the 4-state-integral-only net rule is 6.7.1 prose (the "
     ".8b.2 ispras 06.07.01_02 mirror)",
     ["net_class_fail", "net_string_fail"]),
    ("must_accept",
     "module instantiation inside a program block - BNF-parseable as an "
     "A.6.10 checker_instantiation (concurrent_assertion_item is a "
     "non_port_program_item); whether the name denotes a module or a "
     "checker is resolution semantics (the generic-production escape-hatch "
     "law)",
     ["program5b"]),
    ("must_accept",
     "packed struct/union member restrictions (dynamic array / queue / "
     "unpacked array members, member default values) - 7.2.x/7.3.x prose; "
     "A.2.2.1 struct_union_member admits variable_dimensions and "
     "initializers",
     ["struct_packed_darray_fail", "struct_packed_queue_fail",
      "struct_packed_uarray_fail", "struct_packed_member_def",
      "union_packed_darray_fail", "union_packed_queue_fail",
      "union_packed_uarray_fail"]),
    ("must_accept",
     "assignment-pattern arity/shape semantics (too few/many elements, "
     "pattern on a scalar) - 10.9 prose; assignment_pattern_expression is "
     "A.8.4 BNF",
     ["sv_ap_parray_fail1", "sv_ap_parray_fail2", "sv_ap_parray_fail3",
      "sv_ap_struct_fail1", "sv_ap_struct_fail2", "sv_ap_uarray_fail1",
      "sv_ap_uarray_fail2"]),
    ("must_accept",
     "string literal assigned to unpacked arrays (4-state/narrow/wide "
     "element types, multi-dimensional targets) - assignment compatibility "
     "prose; the initializers are ordinary BNF",
     ["sv_byte_array_string_fail1", "sv_byte_array_string_fail2",
      "sv_byte_array_string_fail4", "sv_byte_array_string_fail5"]),
    ("must_accept",
     "class typing/hierarchy semantics (unrelated/base-to-derived "
     "assignment, new on a non-class variable, typed constructor "
     "relatedness, instantiating a virtual class, super without a parent) - "
     "8.x prose; 'x = new'/'x = T::new' are A.2.4 class_new BNF",
     ["sv_class_compat_fail1", "sv_class_compat_fail2",
      "sv_class_compat_fail3", "sv_class_new_fail1", "sv_class_new_fail2",
      "sv_class_new_typed_fail1", "sv_class_new_typed_fail2",
      "sv_class_new_typed_fail3", "sv_class_new_typed_fail4",
      "sv_class_virt_new_fail", "sv_super_member_fail"]),
    ("must_accept",
     "class method declared with static lifetime - A.1.9 class_method -> "
     "task/function_declaration admits [lifetime]; the automatic-only rule "
     "is 8.6 prose",
     ["sv_class_method_lt_static1", "sv_class_method_lt_static2"]),
    ("must_accept",
     "writes to a const variable (continuous/blocking/non-blocking/force/"
     "procedural-assign, output/inout port binding) - 6.20.6 prose; every "
     "form is ordinary BNF",
     ["sv_const_fail1", "sv_const_fail2", "sv_const_fail3", "sv_const_fail4",
      "sv_const_fail5", "sv_const_fail6", "sv_const_fail7", "sv_const_fail8",
      "sv_const_fail9"]),
    ("must_accept",
     "package export eligibility (not-imported, wildcard-conflict, "
     "declared-outside-package) - 26.6 prose; export declarations are "
     "A.1.11 BNF (the .8b.2 export-pin mirror)",
     ["sv_export_fail1", "sv_export_fail2", "sv_export_fail3",
      "sv_export_fail4", "sv_export_fail5", "sv_export_fail6"]),
    ("must_accept",
     "foreach loop-variable count exceeding the array dimensions - 12.7.3 "
     "semantics; foreach is ordinary BNF",
     ["sv_foreach_fail1"]),
    ("must_accept",
     "class/dynamic-array/queue/string/unpacked-array operands inside "
     "lvalue concatenations - A.8.5 variable_lvalue admits the concat of "
     "variable_lvalues; the integral-only member rule is 10.10/11.4.12 "
     "prose",
     ["sv_lval_concat_class_fail1", "sv_lval_concat_class_fail2",
      "sv_lval_concat_class_fail3", "sv_lval_concat_class_fail4",
      "sv_lval_concat_darray_fail1", "sv_lval_concat_darray_fail2",
      "sv_lval_concat_darray_fail3", "sv_lval_concat_darray_fail4",
      "sv_lval_concat_queue_fail1", "sv_lval_concat_queue_fail2",
      "sv_lval_concat_queue_fail3", "sv_lval_concat_queue_fail4",
      "sv_lval_concat_string_fail1", "sv_lval_concat_string_fail2",
      "sv_lval_concat_string_fail3", "sv_lval_concat_string_fail4",
      "sv_lval_concat_uarray_fail1", "sv_lval_concat_uarray_fail2",
      "sv_lval_concat_uarray_fail3", "sv_lval_concat_uarray_fail4"]),
    ("must_accept",
     "tf port default-value semantics (non-constant default used in a "
     "constant context; defaults on output ports) - 13.5.3 rules and tool "
     "support; A.2.7 tf_port_item's [= expression] is direction-agnostic "
     "BNF",
     ["sv_port_default13", "sv_port_default14"]),
    ("must_accept",
     "value override of a type parameter - A.4.1.1 "
     "named_parameter_assignment's param_expression admits plain "
     "expressions; the kind check is 6.20.3 elaboration",
     ["sv_type_param_fail1"]),
    ("must_accept",
     "forward-typedef circularity / self-reference (circular chains, enum/"
     "struct/union using the forwarded name inside its own definition) - "
     "6.18 resolution semantics; forward typedefs and type_identifier uses "
     "are ordinary BNF",
     ["sv_typedef_circular1", "sv_typedef_circular2",
      "sv_typedef_fwd_enum_fail", "sv_typedef_fwd_struct_fail",
      "sv_typedef_fwd_union_fail"]),
    ("must_accept",
     "return-with-value in a task / return inside fork..join - A.6.4 "
     "jump_statement 'return [expression];' is an ordinary statement; the "
     "task-value and parallel-block placement bans are 13.3/9.3.2 prose "
     "(the .8b.2 ispras 09.03.02_03 mirror)",
     ["task_return_fail1", "task_return_fail2"]),
]

IVTEST_CE_STAGE_PINNED = {}
for _cls, _basis, _names in IVTEST_CE_STAGE_CLUSTERS:
    for _n in _names:
        _key = f"ivltests/{_n}.v"
        if _key in IVTEST_CE_STAGE_PINNED:
            raise SystemExit(f"duplicate .8b.3 stage pin: {_key}")
        IVTEST_CE_STAGE_PINNED[_key] = (
            _cls, "ivtest CE pinned .8b.3: " + _basis)
if len(IVTEST_CE_STAGE_PINNED) != 283:
    raise SystemExit(
        f".8b.3 stage-pin table holds {len(IVTEST_CE_STAGE_PINNED)} entries, "
        "expected exactly the 283-row CE-without-gold population")


class IvtestIndex:
    """Answer key from ivtest regress-sv.list / regress-vlg.list entries
    (SV-CORPUS-GRAD.8) + the vvp_tests/*.json per-test descriptors as the
    secondary key source for unlisted files (leaf .8b.1). Logical list entry
    (backslash-continued physical lines joined first):
    `<name> <type>[,<flags>...] <dir> [gold=<file>]`.
    The CE parse-vs-elaboration stage split mirrors the verilator convention:
    a golden output reporting a syntax error = parse-level invalid; a golden
    output with only post-parse errors = syntax itself valid.
    vvp_tests descriptors (consumed by upstream vvp_reg.py) carry
    `type` / `source` (always under ivltests/) / `iverilog-args` (the dialect
    generation) / `gold` (a stem resolved as gold/<gold>-iverilog-<chan>.gold);
    only descriptors with an EXPLICIT SystemVerilog generation flag key the
    sv_2017 bulk lane."""

    SV_GENS = ("-g2005-sv", "-g2009", "-g2012", "-g2017", "-g2023")
    V2005_GENS = ("-g1995", "-g2001", "-g2001-noconfig", "-g2005")

    def __init__(self, ivtest_dir: Path):
        self.sv_entries = {}   # (dir, name) -> (type, gold-or-None)
        self.vlg_entries = {}  # (dir, name) -> (type, gold-or-None)
        self.gold_dir = ivtest_dir / "gold"
        self.vvp_desc = defaultdict(list)  # source stem -> [descriptor dict]
        self._load(ivtest_dir / "regress-sv.list", sv=True)
        self._load(ivtest_dir / "regress-vlg.list", sv=False)
        self._load_vvp(ivtest_dir / "vvp_tests")

    def _load_vvp(self, vvp_dir: Path):
        if not vvp_dir.is_dir():
            return
        for jf in sorted(vvp_dir.glob("*.json")):
            try:
                desc = json.loads(read_text(jf))
            except ValueError:
                continue
            src = desc.get("source")
            ttype = desc.get("type")
            if not src or not ttype:
                continue
            desc["_key"] = jf.stem
            self.vvp_desc[Path(src).stem].append(desc)

    def _gold_has_syntax_error(self, gold_stem: str) -> bool:
        text = ""
        for chan in ("iverilog-stderr", "iverilog-stdout"):
            text += read_text(self.gold_dir / f"{gold_stem}-{chan}.gold")
        return bool(SYNTAX_ERR_RE.search(text))

    def _vvp_implied(self, desc):
        """Map one SV-dialect descriptor to an implied parse-level verdict
        tag: 'accept' / 'reject' / 'triage' (CE without usable golden) /
        'ni' (Not Implemented - upstream runner skips, no testimony)."""
        ttype = desc.get("type")
        if ttype == "NI":
            return "ni"
        if ttype != "CE":
            # normal/EF (and any run-to-completion type): iverilog compiles
            # the file under the declared SV generation - parse-level valid.
            return "accept"
        gold = desc.get("gold")
        if gold:
            return "reject" if self._gold_has_syntax_error(gold) else "accept"
        return "triage"

    def _load(self, list_path: Path, sv: bool):
        text = read_text(list_path)
        if not text:
            return
        logical, buf = [], ""
        for line in text.splitlines():
            if line.rstrip().endswith("\\"):
                buf += line.rstrip()[:-1] + " "
                continue
            logical.append(buf + line)
            buf = ""
        if buf:
            logical.append(buf)
        for line in logical:
            line = line.split("#", 1)[0].strip()
            if not line:
                continue
            fields = line.split()
            if len(fields) < 3:
                continue
            ttype = fields[1].split(",")[0]
            if ttype not in IVTEST_TYPES:
                continue
            name, gold, dir_field = fields[0], None, None
            for f in fields[2:]:
                if f.startswith("gold="):
                    gold = f[len("gold="):]
                elif f.startswith(("./", "-")):
                    continue  # continuation source path / stray flag
                elif dir_field is None:
                    dir_field = f
            if dir_field is None:
                continue
            if sv:
                self.sv_entries[(dir_field, name)] = (ttype, gold)
            else:
                self.vlg_entries[(dir_field, name)] = (ttype, gold)

    def expect(self, relpath: str):
        p = Path(relpath.replace("\\", "/"))
        if len(p.parts) < 3 or p.parts[0] != "ivtest":
            return ("out_of_scope_with_cause:no_sv_key",
                    "ivtest: outside the ivtest test tree")
        key = (p.parts[1], p.stem)
        if key in self.sv_entries:
            ttype, gold = self.sv_entries[key]
            if ttype != "CE":
                # normal/CO require compilation to succeed; EF/RE fail only at
                # run time; all imply parse-level validity.
                return ("must_accept",
                        f"ivtest: regress-sv.list type {ttype} - compiles under "
                        "the iverilog SV dialect, parse-level valid")
            if gold:
                gtext = read_text(self.gold_dir / gold)
                if SYNTAX_ERR_RE.search(gtext):
                    return ("must_reject",
                            f"ivtest: CE with golden {gold} reporting a syntax "
                            "error - parse-level invalid")
                return ("must_accept",
                        f"ivtest: CE but golden {gold} shows only post-parse "
                        "errors - syntax itself valid")
            pinned = IVTEST_CE_STAGE_PINNED.get("/".join(p.parts[1:]))
            if pinned:
                return pinned
            return ("out_of_scope_with_cause:negative_stage_triage",
                    "ivtest: CE without golden output - failure stage (parse vs "
                    "elaboration) unresolved and outside the .8b.3 pinned "
                    "population (added upstream after the vendored pin?)")
        if key in self.vlg_entries:
            return ("out_of_scope_with_cause:v2005_profile_lane",
                    "ivtest: regress-vlg.list entry - keyed for the "
                    "verilog_2005 profile lane, not the sv_2017 bulk run")
        vvp = self._expect_vvp(p)
        if vvp is not None:
            return vvp
        return ("out_of_scope_with_cause:no_sv_key",
                "ivtest: no regress-sv.list entry (other-target list / "
                "multi-file companion / unlisted)")

    def _expect_vvp(self, p: Path):
        """Secondary key: vvp_tests JSON descriptors (leaf .8b.1). Sources
        live under ivltests/ by upstream convention (vvp_reg.py). Multiple
        descriptors may share one source (dialect variants) - keyable only
        when every SV-dialect descriptor implies the same verdict."""
        if p.parts[1] != "ivltests":
            return None
        descs = self.vvp_desc.get(p.stem)
        if not descs:
            return None
        sv_descs, v2005 = [], False
        for d in descs:
            args = d.get("iverilog-args", [])
            if any("verilog-ams" in a for a in args):
                continue  # AMS-flavored run - never an SV/plain-Verilog key
            if any(a in self.SV_GENS for a in args):
                sv_descs.append(d)
            elif any(a in self.V2005_GENS for a in args):
                v2005 = True
        if sv_descs:
            implied = {self._vvp_implied(d) for d in sv_descs}
            names = ", ".join(d["_key"] + ".json" for d in sv_descs)
            if implied == {"accept"}:
                return ("must_accept",
                        f"ivtest: vvp_tests descriptor(s) {names} - runs "
                        "under an explicit SV generation with no "
                        "syntax-error golden, parse-level valid")
            if implied == {"reject"}:
                return ("must_reject",
                        f"ivtest: vvp_tests descriptor(s) {names} - CE with "
                        "golden iverilog output reporting a syntax error, "
                        "parse-level invalid")
            if implied == {"triage"}:
                pinned = IVTEST_CE_STAGE_PINNED.get("/".join(p.parts[1:]))
                if pinned:
                    return pinned
                return ("out_of_scope_with_cause:negative_stage_triage",
                        f"ivtest: vvp_tests descriptor(s) {names} - CE "
                        "without usable golden output and outside the .8b.3 "
                        "pinned population (added upstream after the "
                        "vendored pin?)")
            if implied == {"ni"}:
                return ("out_of_scope_with_cause:ni_unimplemented",
                        f"ivtest: vvp_tests descriptor(s) {names} - type NI "
                        "(Not Implemented): the upstream runner skips the "
                        "test, so it carries no validity testimony")
            return ("out_of_scope_with_cause:descriptor_conflict",
                    f"ivtest: vvp_tests descriptors {names} imply "
                    f"conflicting verdicts ({', '.join(sorted(implied))}) - "
                    "unkeyable without per-file adjudication")
        if any("verilog-ams" in a for d in descs
               for a in d.get("iverilog-args", [])):
            return ("out_of_scope_with_cause:verilog_ams_lane",
                    "ivtest: vvp_tests descriptor runs under -gverilog-ams - "
                    "Verilog-AMS surface, outside the IEEE 1800 scope "
                    "(VERILOG-AMS tree parked)")
        if v2005:
            return ("out_of_scope_with_cause:v2005_profile_lane",
                    "ivtest: vvp_tests descriptor runs under an explicit "
                    "plain-Verilog generation - keyed for the verilog_2005 "
                    "profile lane")
        return ("out_of_scope_with_cause:no_sv_key",
                "ivtest: vvp_tests descriptor(s) without an explicit "
                "generation flag - dialect unresolved (the upstream default "
                "generation is not encoded in the descriptor)")

    def expect_v2005(self, relpath: str):
        """v2005-lane key for one ivtest row (leaf .8c.1): regress-vlg.list
        entries first (the .8a CE stage split mirrored: golden `syntax
        error` = parse-level invalid under the iverilog plain-Verilog
        dialect), then explicit plain-Verilog vvp_tests generations."""
        p = Path(relpath.replace("\\", "/"))
        key = (p.parts[1], p.stem) if len(p.parts) >= 3 else None
        if key in self.vlg_entries:
            ttype, gold = self.vlg_entries[key]
            if ttype != "CE":
                return ("must_accept",
                        f"ivtest: regress-vlg.list type {ttype} - compiles "
                        "under the iverilog plain-Verilog dialect, "
                        "parse-level valid 1364-2005")
            if gold:
                gtext = read_text(self.gold_dir / gold)
                if SYNTAX_ERR_RE.search(gtext):
                    return ("must_reject",
                            f"ivtest: vlg CE with golden {gold} reporting a "
                            "syntax error - parse-level invalid 1364-2005")
                return ("must_accept",
                        f"ivtest: vlg CE but golden {gold} shows only "
                        "post-parse errors - syntax itself valid")
            pinned = IVTEST_VLG_CE_STAGE_PINNED.get("/".join(p.parts[1:]))
            if pinned:
                return pinned
            return ("out_of_scope_with_cause:negative_stage_triage_v2005",
                    "ivtest: vlg CE without golden output - failure stage "
                    "(parse vs elaboration) unresolved and outside the "
                    ".8c.2 pinned population (added upstream after the "
                    "vendored pin?)")
        if len(p.parts) >= 3 and p.parts[1] == "ivltests":
            descs = self.vvp_desc.get(p.stem, [])
            v_descs = [d for d in descs
                       if not any("verilog-ams" in a
                                  for a in d.get("iverilog-args", []))
                       and any(a in self.V2005_GENS
                               for a in d.get("iverilog-args", []))]
            if v_descs:
                implied = {self._vvp_implied(d) for d in v_descs}
                names = ", ".join(d["_key"] + ".json" for d in v_descs)
                if implied == {"accept"}:
                    return ("must_accept",
                            f"ivtest: vvp_tests descriptor(s) {names} - "
                            "runs under an explicit plain-Verilog "
                            "generation with no syntax-error golden")
                if implied == {"reject"}:
                    return ("must_reject",
                            f"ivtest: vvp_tests descriptor(s) {names} - CE "
                            "with golden iverilog output reporting a "
                            "syntax error")
                if implied == {"triage"}:
                    pinned = IVTEST_VLG_CE_STAGE_PINNED.get(
                        "/".join(p.parts[1:]))
                    if pinned:
                        return pinned
                    return (
                        "out_of_scope_with_cause:negative_stage_triage_v2005",
                        f"ivtest: vvp_tests descriptor(s) {names} - CE "
                        "without usable golden output and outside the "
                        ".8c.2 pinned population (added upstream after "
                        "the vendored pin?)")
                if implied == {"ni"}:
                    return ("out_of_scope_with_cause:ni_unimplemented",
                            f"ivtest: vvp_tests descriptor(s) {names} - "
                            "type NI (upstream runner skips, no testimony)")
                return ("out_of_scope_with_cause:descriptor_conflict",
                        f"ivtest: vvp_tests descriptors {names} imply "
                        f"conflicting verdicts "
                        f"({', '.join(sorted(implied))})")
        return ("out_of_scope_with_cause:no_v2005_key",
                "ivtest: v2005-lane row without a usable vlg-list or "
                "vvp_tests key")


# sv2v test/error/ stage classification (leaf .8b.1). Every error/ file is an
# INTENDED sv2v failure whose `// pattern:` header keys the upstream message;
# the STAGE of the invalidity is adjudicated per the IEEE 1800-2017 LRM
# (in-repo docs/systemverilog/2017/md), never per what any parser does:
#   reject  - the text violates the Annex A BNF itself (parse/lexical level)
#   accept  - the text is BNF-parseable; the invalidity is a prose "shall"
#             (semantic/elaboration stage), so parse-level valid
#   preproc - the intended failure is at the preprocessing stage (svpp lane)
# Filenames absent from the table (incl. the 9 files with no `// pattern:`
# key and the named ambiguous families) stay in the error_pretriage residue
# for per-file pinned adjudication (leaf .8b.2).
SV2V_REJECT_GROUPS = [
    ({"assert_deferred_nonzero"},
     "IEEE 1800-2017 A.6.10: a deferred immediate assertion takes the "
     "literal '#0' ('#1' has no production)"),
    ({"auto_dim_int", "const_const", "var_var", "decl_bare", "decl_const_wire",
      "decl_binop_asgn", "decl_missing_comma", "decl_non_blocking_asgn",
      "decl_ranged_implicit", "decl_signed_implicit", "decl_trailing_comma",
      "decl_wire_var", "decl_delay_asgn", "decl_delay_asgn_init",
      "decl_delay_asgn_package", "decl_delay_asgn_port"},
     "declaration grammar violation - no IEEE 1800-2017 Annex A "
     "data/net-declaration production admits this form (sv2v keys it at its "
     "parse stage)"),
    ({"block_start_1", "block_start_2", "block_start_3", "block_start_4",
      "run_on_decl_item", "run_on_decl_package", "run_on_decl_stmt",
      "port_list_incomplete", "elab_task_stray_after_args",
      "elab_task_stray_before_args", "elab_task_stray_no_args"},
     "statement/item grammar violation - no IEEE 1800-2017 Annex A "
     "production admits this token sequence (sv2v keys it at its parse "
     "stage)"),
    ({"for_loop_decl_no_init", "for_loop_init_bare", "for_loop_init_delay",
      "for_loop_init_nblk", "for_loop_init_stray"},
     "IEEE 1800-2017 A.6.8 for_initialization/for_variable_declaration "
     "requires '= expression' and admits no timing/non-blocking form"),
    ({"instantiation_extra_comma", "instantiation_missing_ports",
      "instantiation_no_label", "instantiation_no_module",
      "instantiation_not_ports", "instantiation_not_range",
      "instantiation_trailing_comma"},
     "IEEE 1800-2017 A.4.1.1 module_instantiation grammar violation"),
    ({"missing_end", "missing_endfunction", "missing_endgenerate",
      "missing_endinterface_1", "missing_endinterface_2",
      "missing_endmodule_1", "missing_endmodule_2", "missing_endpackage",
      "missing_endtask", "missing_join"},
     "EOF truncation - the construct's mandatory closing keyword production "
     "is unsatisfied (parse level)"),
    ({"block_comment_eof", "string_literal_eof"},
     "lexical level - unterminated block comment / string literal"),
    ({"highz0_highz1"},
     "IEEE 1800-2017 A.2.2.2 drive_strength pairs a 0-strength with a "
     "1-strength - (highz0, highz1) has no production"),
    ({"casex_inside", "casez_inside"},
     "IEEE 1800-2017 A.6.7: the 'inside' case variant is the literal 'case' "
     "keyword only - casex/casez ... inside has no production"),
    ({"decl_after_stmt"},
     "IEEE 1800-2017 A.6.3 seq_block: { block_item_declaration } strictly "
     "precedes { statement_or_null } - a declaration after a statement has "
     "no production"),
    ({"string_packed", "string_signed", "byte_packed", "enum_post_signed",
      "typeof_packed", "typeof_signed"},
     "IEEE 1800-2017 A.2.2.1 data_type: string/type_reference admit no "
     "signing or packed dimension; integer_atom_type admits no packed "
     "dimension; enum admits only trailing packed dimensions"),
    ({"binding_mix_param", "binding_mix_port", "binding_mix_port_trail"},
     "IEEE 1800-2017 A.4.1.1 list_of_port_connections / "
     "list_of_parameter_assignments: all-ordered or all-named - mixing has "
     "no production"),
]
SV2V_ACCEPT_GROUPS = [
    ({"end_label_block_only", "end_label_block_wrong", "end_label_class_wrong",
      "end_label_function_wrong", "end_label_gen_block_only",
      "end_label_gen_block_wrong", "end_label_interface_wrong",
      "end_label_module_wrong", "end_label_package_wrong",
      "end_label_task_wrong"},
     "end-label matching is prose (IEEE 1800-2017 9.3.4/23.2.1 'shall be an "
     "error if the name at the end is different') - the BNF end label is an "
     "unconstrained identifier"),
    ({"class_missing_item", "class_not_specialized",
      "class_parameter_missing_1", "class_parameter_missing_2",
      "class_parameter_not_expr", "class_parameter_not_type", "missing_class",
      "module_import_missing_package", "module_import_missing_package_item",
      "package_export_export_1", "package_export_export_2",
      "package_export_missing", "package_export_wrong_1",
      "package_export_wrong_2", "package_import_missing_package",
      "package_import_missing_package_item", "package_loop_1",
      "package_loop_2", "package_scope_conflict_1", "package_scope_conflict_2",
      "package_scope_conflict_3", "package_scope_conflict_4",
      "package_scope_conflict_5", "package_scope_conflict_6",
      "package_scope_conflict_7", "package_scope_conflict_8",
      "package_self_export", "package_self_import",
      "package_self_reference_early", "package_self_reference_loop",
      "typedef_missing", "typedef_not_type_localparam", "typedef_not_type_net",
      "typedef_not_type_var", "typedef_ref_not_type"},
     "name/package/class/type resolution failure - post-parse semantics "
     "(IEEE 1800-2017 clauses 6/8/26 prose); the text is BNF-parseable"),
    ({"binding_not_found_class", "binding_not_found_overflow",
      "binding_not_found_param", "binding_not_found_port",
      "binding_overflow_class", "binding_overflow_param",
      "binding_overflow_port", "module_param_mismatch_expr",
      "module_param_mismatch_type", "parameter_no_default_1",
      "parameter_no_default_2", "parameter_no_default_3",
      "interface_param_mismatch_expr", "interface_param_mismatch_type"},
     "instantiation binding/parameter resolution - elaboration semantics "
     "(IEEE 1800-2017 23.10/23.3.2 prose); the text is BNF-parseable"),
    ({"interface_bad_expr", "interface_bad_expr_arr",
      "interface_bad_expr_genvar", "interface_bad_expr_module",
      "interface_mismatch_1", "interface_mismatch_2", "interface_mismatch_3",
      "interface_mismatch_4", "interface_mismatch_5", "interface_mismatch_6",
      "interface_modport_missing", "interface_modport_unlisted",
      "interface_name_func", "interface_name_var", "interface_unbound_modport",
      "interface_unbound_modports", "interface_unknown"},
     "interface/modport binding semantics (IEEE 1800-2017 clause 25 prose); "
     "the text is BNF-parseable"),
    ({"struct_extra_named_field", "struct_extra_unnamed_field",
      "struct_invalid_key", "struct_logic_bit", "struct_logic_part_range",
      "struct_logic_part_select", "struct_missing_field", "struct_non_integer",
      "struct_out_of_bounds", "struct_out_of_bounds_neg",
      "struct_unknown_field", "typeof_atom_bit", "typeof_atom_range"},
     "assignment-pattern/type-index semantics (IEEE 1800-2017 10.9/7.2 "
     "prose); the text is BNF-parseable"),
    ({"size_cast_neg_lit_1", "size_cast_neg_lit_2", "size_cast_neg_var_1",
      "size_cast_neg_var_2", "size_cast_x_lit", "size_cast_x_var",
      "size_cast_xpr_lit", "size_cast_xpr_var", "size_cast_zero_lit",
      "size_cast_zero_var", "enum_range_neg", "enum_range_x",
      "enum_range_zero", "enum_conflict"},
     "constant-value legality (cast width / enum range values) - semantic "
     "(IEEE 1800-2017 6.24.1/6.19 prose); the text is BNF-parseable"),
    ({"break_inside_fork", "break_outside_loop", "continue_inside_fork",
      "continue_outside_loop", "return_inside_fork", "return_outside_tf",
      "return_task", "return_void_func"},
     "jump-statement placement and return typing are prose rules (IEEE "
     "1800-2017 12.8/13.4.1) - jump_statement is an ordinary statement "
     "production"),
    ({"case_multiple_defaults", "generate_case_multiple_defaults"},
     "IEEE 1800-2017 12.5 'use of multiple default statements in one case "
     "statement shall be illegal' is prose - the BNF admits repeated "
     "default case_items"),
    ({"port_init_early", "port_not_in_header", "port_packed_first",
      "port_packed_second", "port_redeclare", "port_unpacked_first",
      "port_unpacked_second"},
     "port/declaration coherence rules (IEEE 1800-2017 23.2.2 prose); each "
     "declaration is BNF-parseable"),
    ({"default_nettype_none"},
     "implicit-net legality under `default_nettype none is semantic (IEEE "
     "1800-2017 6.10/22.8); the text is BNF-parseable"),
    ({"charge_strength_non_trireg", "drive_strength_uninit"},
     "IEEE 1800-2017 A.2.1.3 net_declaration admits [drive_strength | "
     "charge_strength] on any net_type and net_decl_assignment's "
     "'= expression' is optional - the trireg-only/initializer restrictions "
     "of 1364 became prose in 1800"),
]
SV2V_PREPROC = {
    "include_loop_1", "include_loop_2", "missing_include",
    "include_filename_eof", "unmatched_else", "unmatched_else_end",
    "unmatched_elsif", "unmatched_elsif_end", "unmatched_endif",
    "unmatched_ifdef", "unmatched_ifndef", "undefined_macro",
    "macro_overapplied", "macro_underapplied", "macro_unapplied",
    "macro_unapplied_eof", "macro_args_empty", "macro_arg_bad_eq",
    "macro_arg_bad_name", "macro_illegal_name", "string_directive",
    "double_backtick", "stray_escaped_vendor_comment",
    "string_literal_backtick_eof", "default_nettype_invalid",
}
# The .8b.1 named-ambiguous residue, per-file pinned (leaf .8b.2): each file
# read and adjudicated against the LRM BNF-vs-prose split.
SV2V_PINNED = {
    "dangling_stmt": ("must_reject",
        "pinned .8b.2: a bare statement 'y = 1;' at module level - no A.1.4 "
        "module-item production admits a naked assignment (continuous "
        "assignment requires the assign keyword, A.6.1)"),
    "asgn_expr_non_lhs": ("must_reject",
        "pinned .8b.2: 'x = (1 = x);' - A.8.3 admits '( operator_assignment "
        ")' as an expression but operator_assignment requires a "
        "variable_lvalue and the literal 1 is not one (11.3.6 family)"),
    "parameter_list_not_type": ("must_reject",
        "pinned .8b.2: '#(parameter type X = 1)' - A.2.4 type_assignment "
        "requires a data_type after '='; an integer literal matches no "
        "data_type production"),
    "localparam_no_default": ("must_reject",
        "pinned .8b.2: 'localparam X;' in a module body - A.10 footnote 18: "
        "omitting the constant_param_expression is legal ONLY within a "
        "parameter_port_list (and never for localparam)"),
    "localparam_type_no_default": ("must_reject",
        "pinned .8b.2: 'localparam type X;' - A.10 footnote 18: omitting "
        "the data_type from a type_assignment is legal ONLY within a "
        "parameter_port_list (and never for localparam)"),
    "severity_task_token": ("must_reject",
        "pinned .8b.2: '$fatal x;' - no production admits an identifier "
        "between a system-tf call and ';' (A.6.9/A.8.2)"),
    "export_outside_package_2": ("must_reject",
        "pinned .8b.2: 'export Pkg::Foo;' INSIDE a module - "
        "package_export_declaration is a package_item only (A.1.11); no "
        "A.1.4 module-context production admits it"),
    "decl_const_var_uninit": ("must_accept",
        "pinned .8b.2: 'const var x;' - A.2.1.3 [const][var] with A.2.4's "
        "optional '= expression' is BNF-parseable; 6.20.6 only says what a "
        "const CAN be set to (no parse-level initializer requirement)"),
    "interface_excess_ports": ("must_accept",
        "pinned .8b.2: 3 actuals for a 2-port interface - connection arity "
        "is elaboration semantics (23.3.3)"),
    "interface_missing_direction": ("must_accept",
        "pinned .8b.2: a listed non-ANSI port lacking a direction "
        "declaration in the body - a 23.2.2.1 prose completeness rule"),
    "interface_non_lhs": ("must_accept",
        "pinned .8b.2: expression actual 'b + 1' on an output port - "
        "list_of_port_connections actuals are plain expressions in the BNF; "
        "output-actual lvalue-ness is semantic (23.3.3)"),
    "lhs_expr": ("must_accept",
        "pinned .8b.2: streaming concatenation as continuous-assign LHS - "
        "parses via A.6.1's 'assign list_of_variable_assignments' whose "
        "variable_lvalue admits streaming_concatenation (A.8.5); the "
        "literal-inside-stream-lvalue ban is 11.4.14 prose"),
    "lhs_pattern": ("must_accept",
        "pinned .8b.2: assignment pattern inside a stream LHS - "
        "stream_expression is a plain expression and "
        "assignment_pattern_expression is a primary (A.8.4); inner "
        "lvalue-ness is 11.4.14 prose"),
    "severity_task_arg": ("must_accept",
        "pinned .8b.2: '$fatal(.x(\"x\"));' - BNF-parseable as an A.8.2 "
        "system_tf_call with a named list_of_arguments member; the 20.10 "
        "severity-task argument shape is enforceable only semantically"),
    "export_outside_package_1": ("must_accept",
        "pinned .8b.2: top-level 'export Pkg::Foo;' - "
        "package_export_declaration is a package_item (A.1.11) and A.1.2 "
        "description admits package_item at $unit level; the "
        "outside-package restriction is prose (26.6)"),
    "export_outside_package_3": ("must_accept",
        "pinned .8b.2: top-level 'export *::*;' - same A.1.2/A.1.11 route "
        "as export_outside_package_1; prose restriction only"),
    "include_apos": ("preproc",
        "pinned .8b.2: `include with apostrophe-quoted filename - 22.5 "
        "requires \"...\" or <...>; preprocessing-stage invalidity"),
    "line_char1": ("preproc",
        "pinned .8b.2: `line with a non-numeric level argument - 22.12 "
        "directive-argument validity, preprocessing stage"),
    "line_char2": ("preproc",
        "pinned .8b.2: `line with a malformed level argument '1B' - 22.12 "
        "directive-argument validity, preprocessing stage"),
    "line_eof": ("preproc",
        "pinned .8b.2: bare `line at EOF missing all arguments - 22.12 "
        "directive-argument validity, preprocessing stage"),
    "line_level": ("preproc",
        "pinned .8b.2: `line level 3 (valid levels 0/1/2) - 22.12 "
        "directive-argument validity, preprocessing stage"),
}

SV2V_ERROR_KEY = {}
for _names, _basis in SV2V_REJECT_GROUPS:
    for _n in _names:
        SV2V_ERROR_KEY[_n] = ("must_reject", "sv2v error-suite key: " + _basis)
for _names, _basis in SV2V_ACCEPT_GROUPS:
    for _n in _names:
        SV2V_ERROR_KEY[_n] = ("must_accept", "sv2v error-suite key: " + _basis)
for _n in SV2V_PREPROC:
    SV2V_ERROR_KEY[_n] = (
        "out_of_scope_with_cause",
        "sv2v error-suite key: the intended failure is at the preprocessing "
        "stage (`include/`ifdef/macro machinery, stray backtick, directive "
        "arguments) - svpp-owned conformance")
for _n, (_cls, _basis) in SV2V_PINNED.items():
    if _cls == "preproc":
        SV2V_ERROR_KEY[_n] = ("out_of_scope_with_cause", "sv2v error-suite " + _basis)
    else:
        SV2V_ERROR_KEY[_n] = (_cls, "sv2v error-suite " + _basis)


def expect_sv2v(relpath: str):
    """sv2v test suite: `.sv` files are conversion INPUTS (valid SV by suite
    contract); `.v` files are conversion GOLDENS (Verilog-2005 by contract);
    error/ negatives are stage-classified from their `// pattern:` upstream
    keys against the LRM BNF (leaf .8b.1); the ambiguous residue stays in
    error_pretriage for per-file pinning (leaf .8b.2)."""
    p = relpath.replace("\\", "/")
    if p.endswith(".v"):
        return ("out_of_scope_with_cause:v2005_profile_lane",
                "sv2v: golden Verilog-2005 conversion output - keyed for the "
                "verilog_2005 profile lane")
    if p.startswith("test/error/"):
        key = SV2V_ERROR_KEY.get(Path(p).stem)
        if key:
            return key
        return ("out_of_scope_with_cause:error_pretriage",
                "sv2v: error/ negative outside the .8b.1/.8b.2 classified "
                "population (added upstream after the vendored pin?) - "
                "needs a stage classification before it can key")
    return ("must_accept",
            "sv2v: conversion-input .sv (valid SV by suite contract)")


class SurelogIndex:
    """Answer key from Surelog per-test drivers + committed golden logs
    (leaf .8b.1). A test unit = a directory under tests/ directly containing
    >= 1 `.sl` driver. A unit is keyable only when it is single-source
    (exactly one .sv/.v/.svh under the unit dir, recursively) AND no driver
    references out-of-unit sources/libraries/file-lists; then the committed
    golden log(s) carry the upstream parse testimony:
      [SNT:...] syntax-error codes -> parse-level reject intent;
      a completed log with no [SNT:]/[FTL:] -> the exact file text parses
      under Surelog's IEEE 1800-2017 grammar (accept);
      [FTL:...] fatal -> the run aborted, no usable testimony."""

    UNKEYABLE_FLAGS = {"-y", "-v", "-f", "-map", "-cfg", "-batch"}
    SRC_SUFFIXES = (".sv", ".v", ".svh")

    def __init__(self, tests_root: Path):
        self.tests_root = tests_root
        self.units = {}  # dir path relative to tests_root -> unit record
        if not tests_root.is_dir():
            return
        for sl in sorted(tests_root.rglob("*.sl")):
            rel_dir = sl.parent.relative_to(tests_root)
            if rel_dir in self.units:
                continue
            srcs = sorted(p for p in sl.parent.rglob("*")
                          if p.suffix in self.SRC_SUFFIXES)
            logs = sorted(sl.parent.glob("*.log"))
            external = False
            for drv in sorted(sl.parent.glob("*.sl")):
                for tok in read_text(drv).split():
                    if tok in self.UNKEYABLE_FLAGS or ".." in tok:
                        external = True
            log_text = "".join(read_text(l) for l in logs)
            self.units[rel_dir] = {
                "n_srcs": len(srcs),
                "external": external,
                "log_names": ", ".join(l.name for l in logs),
                "has_logs": bool(logs),
                "snt": "[SNT:" in log_text,
                "ftl": "[FTL:" in log_text,
            }

    def expect(self, relpath: str):
        p = Path(relpath.replace("\\", "/"))
        unit = None
        if p.parts and p.parts[0] == "tests":
            anc = p.parent
            while len(anc.parts) > 1:
                unit = self.units.get(Path(*anc.parts[1:]))
                if unit is not None:
                    break
                anc = anc.parent
        if unit is None:
            return ("chained_only",
                    "Surelog: no per-test .sl driver unit owns this file's "
                    "directory - unkeyed (chain-level only)")
        if unit["n_srcs"] != 1:
            return ("chained_only",
                    f"Surelog: multi-file test unit ({unit['n_srcs']} "
                    "sources) - dir-level chained adjudication (leaf .4)")
        if unit["external"]:
            return ("chained_only",
                    "Surelog: driver references out-of-unit sources/"
                    "libraries/file-lists (-y/-v/-f/-map/-cfg/-batch or "
                    "../ paths) - chain-level only")
        if not unit["has_logs"]:
            return ("chained_only",
                    "Surelog: no committed golden log for the unit - no "
                    "upstream parse testimony")
        if unit["ftl"]:
            return ("chained_only",
                    f"Surelog: golden log {unit['log_names']} aborted with "
                    "[FTL:] - no usable parse testimony")
        if unit["snt"]:
            return ("must_reject",
                    f"Surelog: golden log {unit['log_names']} reports "
                    "[SNT:] syntax errors - upstream keys this single-source "
                    "unit parse-level invalid")
        return ("must_accept",
                f"Surelog: golden log {unit['log_names']} completes with no "
                "[SNT:]/[FTL:] - upstream testimony that the unit's single "
                "source parses under Surelog's IEEE 1800-2017 grammar")


# opentitan / black-parrot / uvm-core join the design-corpus lane: real-design
# trees whose files need include/define chaining to adjudicate honestly.
DESIGN_SUITES = {"Cores-VeeR-EL2", "friscv", "scr1",
                 "opentitan", "black-parrot", "uvm-core"}


def adjudicate(expected, observed, dep_flag):
    if observed == "crash":
        # Probe signal-death (e.g. debug-build stack overflow) is a defect
        # REGARDLESS of the expected verdict - never a graceful reject and
        # never explained away (leaf .8c.1 finding: br_gh330.v).
        return "divergence:unexplained_crash"
    if observed == "timeout":
        # Known-pathological tracking outranks deferral: a hang is surfaced
        # no matter which lane owns the file.
        return "divergence:explained_timeout"
    if expected == "chained_only":
        return "deferred:chained_only"
    if expected.startswith("out_of_scope_with_cause"):
        # A ':<slug>' suffix names the owning lane (SV-CORPUS-GRAD.8); the
        # bare form keeps the historical svpp_owned label byte-stable.
        if ":" in expected:
            return "deferred:" + expected.split(":", 1)[1]
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
    # --- the verilog_2005 profile lane (leaf .8c) ---
    ap.add_argument("--out-lane-list",
                    default=root / "stimuli/sv/characterization/v2005_lane_files.tsv",
                    type=Path,
                    help="emitted list of v2005-lane files (runner sv2005 input)")
    ap.add_argument("--results-v2005",
                    default=root / "stimuli/sv/characterization/results_v2005.tsv",
                    type=Path,
                    help="raw outcomes of the runner's sv2005 mode; the v2005 "
                         "manifest is built only when this file exists")
    ap.add_argument("--out-manifest-v2005",
                    default=root / "stimuli/sv/characterization/adjudication_manifest_v2005.tsv",
                    type=Path)
    ap.add_argument("--out-summary-v2005",
                    default=root / "stimuli/sv/characterization/adjudication_summary_v2005.md",
                    type=Path)
    args = ap.parse_args()

    rows = []
    for line in args.results.read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        suite, observed, path = line.split("\t")
        if f"/subs/{suite}/" in path:
            rel = path.split(f"/subs/{suite}/", 1)[1]
        elif "/stimuli/sv/uvm/" in path:
            # the uvm-core fold (leaf .8): plain tracked files, not a submodule
            rel = path.split("/stimuli/sv/uvm/", 1)[1]
        else:
            raise SystemExit(f"unrecognized results path shape: {path!r}")
        rows.append((suite, rel, observed))
    rows.sort()

    vidx = VerilatorIndex(args.subs_root / "verilator/test_regress/t")
    ividx = IvtestIndex(args.subs_root / "iverilog/ivtest")
    sidx = SurelogIndex(args.subs_root / "Surelog/tests")

    manifest = []
    for suite, rel, observed in rows:
        if suite == "uvm-core":
            fpath = args.subs_root.parent / "uvm" / rel
        else:
            fpath = args.subs_root / suite / rel
        text = read_text(fpath)
        if (suite, rel) in EXTRA_PINNED:
            expected, basis = EXTRA_PINNED[(suite, rel)]
        elif rel.endswith(".svh") and suite not in DESIGN_SUITES:
            # Generic across the test suites: .svh files are `include payloads,
            # not standalone compilation units (may be bare fragments).
            expected, basis = ("chained_only",
                               f"{suite}: .svh include payload, not a standalone unit")
        elif suite in DESIGN_SUITES:
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
            expected, basis = expect_verible(rel, text)
        elif suite == "ispras-sv-tests":
            expected, basis = expect_ispras(rel, text)
        elif suite == "iverilog":
            expected, basis = ividx.expect(rel)
        elif suite == "sv2v":
            expected, basis = expect_sv2v(rel)
        elif suite == "Surelog":
            expected, basis = sidx.expect(rel)
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
        elif expected == "must_reject" and dep_flag in ("macro_use", "conditional"):
            # Mirror of the include rule (leaf .8b.1): the intended syntax
            # error may only materialize after macro expansion / conditional
            # resolution, which isolation parse of the raw text never
            # performs - a raw-text reject would testify for the wrong
            # reason, so the reject expectation is svpp-lane.
            expected = "out_of_scope_with_cause"
            basis += (" - but macro/conditional-dependent: the reject "
                      "expectation is only meaningful post-preprocessing "
                      "(svpp lane)")
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

    # --- the verilog_2005 profile lane (leaf .8c) ------------------------
    lane = [(suite, rel) for suite, rel, _obs, _exp, verdict, _basis in manifest
            if verdict == "deferred:v2005_profile_lane"]
    with args.out_lane_list.open("w", encoding="utf-8") as fh:
        fh.write("suite\trelpath\trepo_path\n")
        for suite, rel in lane:
            fh.write(f"{suite}\t{rel}\tstimuli/sv/subs/{suite}/{rel}\n")
    print(f"v2005 lane list: {args.out_lane_list} ({len(lane)} rows)")

    if not args.results_v2005.is_file():
        print("v2005 results absent - run `stimuli/run_external_corpus.sh "
              "sv2005` to produce them; v2005 manifest skipped")
        return 0

    lane_set = set(lane)
    v_rows = []
    for line in args.results_v2005.read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        suite, observed, path = line.split("\t")
        if f"/subs/{suite}/" not in path:
            raise SystemExit(f"unrecognized v2005 results path: {path!r}")
        rel = path.split(f"/subs/{suite}/", 1)[1]
        if (suite, rel) not in lane_set:
            raise SystemExit(
                f"v2005 results row ({suite}, {rel}) is not a lane member - "
                "stale results_v2005.tsv vs the current lane derivation")
        v_rows.append((suite, rel, observed))
    v_rows.sort()
    missing = lane_set - {(s, r) for s, r, _o in v_rows}
    if missing:
        raise SystemExit(
            f"{len(missing)} lane rows missing from results_v2005.tsv "
            f"(stale/partial sv2005 run), e.g. {sorted(missing)[:3]}")

    v_manifest = []
    for suite, rel, observed in v_rows:
        text = read_text(args.subs_root / suite / rel)
        expected, basis = expect_v2005(suite, rel, text, ividx)
        dep_flag = ""
        if expected in ("must_accept", "must_reject"):
            dep_flag = preproc_dependency(text)
        if expected == "must_reject" and dep_flag == "include":
            expected = "chained_only"
            basis += " - but `include-dependent: reject expectation is chain-level"
        elif expected == "must_reject" and dep_flag in ("macro_use", "conditional"):
            expected = "out_of_scope_with_cause"
            basis += (" - but macro/conditional-dependent: the reject "
                      "expectation is only meaningful post-preprocessing "
                      "(svpp lane)")
        if expected != "must_accept":
            dep_flag = ""
        verdict = adjudicate(expected, observed, dep_flag)
        v_manifest.append((suite, rel, observed, expected, verdict, basis))

    with args.out_manifest_v2005.open("w", encoding="utf-8") as fh:
        fh.write("suite\trelpath\tobserved\texpected\tadjudication\tbasis\n")
        for row in v_manifest:
            fh.write("\t".join(row) + "\n")

    v_per_suite = defaultdict(Counter)
    v_total = Counter()
    for suite, _rel, _obs, _exp, verdict, _basis in v_manifest:
        v_per_suite[suite][verdict] += 1
        v_total[verdict] += 1
    lines = []
    lines.append("# SV external-corpus adjudication - the verilog_2005 "
                 "profile lane (leaf SV-CORPUS-GRAD.8c)")
    lines.append("")
    lines.append(f"Input: `results_v2005.tsv` ({len(v_manifest)} rows parsed "
                 "under `--profile verilog_2005`); expected verdicts per "
                 "IEEE 1364-2005 answer keys (ispras TYPE headers + "
                 "KNOWN_TEXT_BUGS deferral note, ivtest regress-vlg.list / "
                 "plain-Verilog vvp_tests descriptors with the CE golden "
                 "syntax-error split, sv2v conversion-golden contract).")
    lines.append("")
    lines.append("| suite | rows | match | UNEXPLAINED div | explained div | deferred |")
    lines.append("|---|---|---|---|---|---|")
    for suite in sorted(v_per_suite):
        c = v_per_suite[suite]
        m2, u2, e2, d2 = bucket(c)
        lines.append(f"| {suite} | {sum(c.values())} | {m2} | {u2} | {e2} | {d2} |")
    m2, u2, e2, d2 = bucket(v_total)
    lines.append(f"| **total** | **{len(v_manifest)}** | **{m2}** | **{u2}** "
                 f"| **{e2}** | **{d2}** |")
    lines.append("")
    lines.append("## Verdict-class detail")
    lines.append("")
    lines.append("| class | count |")
    lines.append("|---|---|")
    for cls in sorted(v_total):
        lines.append(f"| {cls} | {v_total[cls]} |")
    lines.append("")
    lines.append("**The v2005 arm's burn-down baseline = the UNEXPLAINED "
                 f"divergence count ({u2}: rejects-valid "
                 f"{v_total.get('divergence:unexplained_rejects_valid', 0)}, "
                 f"accepts-invalid "
                 f"{v_total.get('divergence:unexplained_accepts_invalid', 0)}, "
                 f"crash "
                 f"{v_total.get('divergence:unexplained_crash', 0)})** "
                 "- a separate arm from the sv_2017 baseline; the `.5` "
                 "graduation gate asserts both.")
    lines.append("")
    args.out_summary_v2005.write_text("\n".join(lines) + "\n", encoding="utf-8")

    print(f"v2005 manifest: {args.out_manifest_v2005} ({len(v_manifest)} rows)")
    print(f"v2005 summary:  {args.out_summary_v2005}")
    print(f"v2005: match={m2} unexplained={u2} explained={e2} deferred={d2}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
