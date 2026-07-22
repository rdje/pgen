#!/usr/bin/env python3
"""SV external-corpus adjudication manifest builder (SV-CORPUS-GRAD.2 + .8).

Turns the raw parse outcomes of stimuli/run_external_corpus.sh sv
(stimuli/sv/characterization/results.tsv) into the expected-vs-actual
adjudication manifest the graduation campaign burns down from
(docs/tasks/SV-CORPUS-GRAD.md leaf .2; ADD-v1 suites + uvm-core fold = leaf .8;
deep answer-key extraction - Surelog golden logs, sv2v error-pattern stage
classification, ivtest vvp_tests JSON descriptors = leaf .8b.1).

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


IVTEST_TYPES = {"normal", "CE", "CO", "EF", "RE", "NI"}


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
        self.vlg_keys = set()  # (dir, name)
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
                self.vlg_keys.add((dir_field, name))

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
            return ("out_of_scope_with_cause:negative_stage_triage",
                    "ivtest: CE without golden output - failure stage (parse vs "
                    "elaboration) unresolved; per-file stage triage = leaf .8b")
        if key in self.vlg_keys:
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
                return ("out_of_scope_with_cause:negative_stage_triage",
                        f"ivtest: vvp_tests descriptor(s) {names} - CE "
                        "without usable golden output; per-file stage "
                        "triage = leaf .8b")
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
                "sv2v: error/ negative outside the .8b.1 stage-classified "
                "population (no '// pattern:' key or named-ambiguous stage) "
                "- per-file pinned adjudication = leaf .8b.2")
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
    return 0


if __name__ == "__main__":
    sys.exit(main())
