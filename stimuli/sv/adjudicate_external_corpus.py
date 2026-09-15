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
  must_reject              the text has no derivation in the LRM's grammar (Annex A /
                           the clause-5 lexical rules). Covers BOTH the suite's own
                           intentional invalidity AND text a vendor tolerates that the
                           standard cannot derive - the pin tables below carry ~30 of the
                           latter (leaves .3.14a/.3.15/.3.16/.3.23), where the spec
                           outranks the upstream tool. "Intentional" is the common case,
                           never the test: the test is the missing derivation.
  chained_only             adjudicable only with include/library chaining (leaf .4)
  not_source_text          the file's BYTES are not SystemVerilog source text at all - a
                           $readmemh memory image (IEEE 1800-2017 §21.4), a bare string
                           literal, a tool command line. Decided by `not_source_text()`
                           from the bytes, never from a list or an extension
                           (SV-CORPUS-GRAD.13c.1). A must_reject key OUTRANKS the shape
                           (rejecting non-source text is a verdict); a must_accept key on
                           such a file is a broken key and the run REFUSES.
  out_of_scope_with_cause  owned by another lane (cause named, e.g. svpp).
                           May carry a deferral slug suffix after ':'
                           (e.g. out_of_scope_with_cause:v2005_profile_lane) —
                           the slug names the owning lane in the verdict class;
                           the bare form keeps the historical svpp_owned label.

Adjudication classes:
  match                                  observed == expected
  divergence:unexplained_rejects_valid   expected accept, parser rejected  <- defect signal
  divergence:unexplained_accepts_invalid expected reject, parser accepted  <- defect signal
  divergence:explained_svpp_protected_envelope
                                         ... file carries a §34 protected envelope
                                             whose key/data block is encoded text
  divergence:explained_svpp_include      expected accept, rejected, file needs `include
  divergence:explained_svpp_macro_use    ... file expands macros (user-defined, or the
                                             §22.13 predefined `__FILE__/`__LINE__)
  divergence:explained_svpp_conditional  ... file relies on `ifdef conditionals
  divergence:explained_timeout           the tracked pathological-input population
  deferred:<cause>                       chained_only / out_of_scope rows (no verdict here)
  deferred:not_source_text               the row is a fact about the CORPUS, not the parser
                                         (PERMANENT and ceiling-bounded in the instrument)

Deterministic: output ordering is (suite, relpath); content-derived only.

Usage:
  python3 stimuli/sv/adjudicate_external_corpus.py \
      [--results stimuli/sv/characterization/results.tsv] \
      [--subs-root stimuli/sv/subs] \
      [--out-manifest stimuli/sv/characterization/adjudication_manifest.tsv] \
      [--out-summary stimuli/sv/characterization/adjudication_summary.md]
"""

import argparse
import hashlib
import json
import re
import sys
from collections import Counter, defaultdict
from pathlib import Path

# The path infix identifying the uvm-core fold in a results.tsv row (leaf .8). No leading
# slash: it must match an absolute row and a repo-root-relative one alike (see main()).
UVM_FOLD_MARKER = "stimuli/sv/uvm/"

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
        "must_reject", 'pinned .3.51 (re-ruling the earlier pin "bit-select on a vectored net is a semantic restriction; the syntax itself parses"): line 17 `logic vectored [15:0] a = 0;` - `vectored` on a VARIABLE declaration. IEEE 1800-2017 A.2.1.3 net_declaration ::= net_type [ drive_strength | charge_strength ] [ vectored | scalared ] data_type_or_implicit [ delay3 ] list_of_net_decl_assignments ; (docs/systemverilog/2017/md/section-41-data-read-api.md:805-812) - the keyword follows a net_type, and no data_declaration carries it; IEEE 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:785-789). The suite\'s intended failure (:should_fail_because: bit selects are not permitted on vectored vector nets, 6.9.2) is semantic, but the declaration line itself is not derivable, which is where the parser stops: REJECTS at furthest_position=432 on sv_2017 and sv_2023 (the `vectored`). Controls (rust/target/s351/rep, release probe 1.0.223 a2ca4c477d54c216…): `wire vectored [15:0] a;` ACCEPTS on every profile. Pins invalid_vectored_on_variable.sv / control_vectored_on_net.sv'),
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
    # SV-CORPUS-GRAD.3.32 (PGEN-SV-CORPUS-GRAD-0375): the chapter-6.23 test writes type-reference comparisons as
    # PROCEDURAL case / if conditions; Annex A of both editions derives them only inside a constant_expression.
    "tests/chapter-6/6.23--type_op_compare.sv": ("must_reject",
        "pinned .3.32: line 18 `case (type(T))` inside `initial begin` with type_reference case items (19), then "
        "`if (type(T) == type(logic[12:0])) $stop;` and its !=, ===, !== twins (22-25) - type-reference comparisons "
        "as PROCEDURAL case / if conditions. IEEE 1800-2017 A.6.7 case_statement ::= [ unique_priority ] case_keyword "
        "( case_expression ) case_item { case_item } endcase (docs/systemverilog/2017/md/section-41-data-read-api.md:2228-2233), "
        "case_expression ::= expression (docs/systemverilog/2017/md/section-41-data-read-api.md:2239), case_item_expression ::= "
        "expression (docs/systemverilog/2017/md/section-41-data-read-api.md:2258); A.6.6 conditional_statement "
        "(docs/systemverilog/2017/md/section-41-data-read-api.md:2205-2212) likewise takes an expression; IEEE 1800-2017 A.8.4 primary (docs/systemverilog/2017/md/section-41-data-read-api.md:3122-3140) has no type_reference arm - type_reference (docs/systemverilog/2017/md/section-41-data-read-api.md:918-920) is a constant_primary (docs/systemverilog/2017/md/section-41-data-read-api.md:3109) under footnote 40 (docs/systemverilog/2017/md/section-41-data-read-api.md:3560-3562: legal as a casting_type and with the four equality operators - i.e. inside a constant_expression) and a data_type (docs/systemverilog/2017/md/section-41-data-read-api.md:859); 6.23 (docs/systemverilog/2017/md/section-65536-216-bits.md:1610-1614) says such comparisons 'are considered to be constant expressions (see 11.2.1)' and its only example is the generate case (docs/systemverilog/2017/md/section-65536-216-bits.md:1626-1631); 11.2.1 (docs/systemverilog/2017/md/section-11-operators-and-expressions.md:87-90): 'Some statement constructs require an expression to be a constant expression' - a procedural if / case takes an expression and is not one of them. "
        "REJECTS at furthest_position=412 on sv_2017 and sv_2023: the byte after the first `type(...)`, where the ONE derivation that admits a type reference inside an expression - a static cast, primary ::= ... | cast, cast ::= casting_type ' ( expression ) (docs/systemverilog/2017/md/section-41-data-read-api.md:3186), casting_type ::= ... | constant_primary (docs/systemverilog/2017/md/section-41-data-read-api.md:841) - requires its ' and meets the comparison operator. "
        "Two-sided: `module top #( parameter type T = type(logic[11:0]) ) ();` ACCEPTS (control param_type_ref.sv, both "
        "profiles) and the standard's own generate form ACCEPTS (control_generate_case_type_reference.sv). Pins invalid_procedural_type_reference_comparison.sv / control_constant_context_type_reference_comparison.sv (the same comparisons as a module-level if / case = conditional_generate_construct / case_generate_construct on a constant_expression, docs/systemverilog/2017/md/section-41-data-read-api.md:1887-1896, in a localparam initializer, and the cast `type(a)'(y)` in procedural code - all ACCEPT on both profiles). Spec outranks the suite: verilator, sv-tests and slang's regression file all write the procedural form and no LRM text or example does - IEEE 1800-2023 kept type_reference in constant_primary alone (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:3066, footnote 45 :3525-3527) and primary without it (:3079-3094) while revising 6.23 (docs/systemverilog/2023/md/section-6-data-types.md:2936-2975). "
        "The test's `:tags: 6.23` / `:description: type comparison tests` claim the clause, and the clause puts the "
        "comparison in a constant_expression context; the suite's positive-test convention is what it outranks"),
}

# Pinned rulings from in-file upstream statements (leaf .3.0 triage findings).
EXTRA_PINNED = {
    # ---- SV-CORPUS-GRAD.3.39 (PGEN-SV-CORPUS-GRAD-0372) ------------------------------------------
    # IEEE 1800-2017 clause 22 gives thirteen compiler directives a syntax box (every one captioned
    # "not in Annex A"), and release 1.0.217 transcribes each box as a production: the parser now
    # REFUSES `timescale frump, `unconnected_drive pull2, `line 1 "f" 3, a bare `pragma, an unknown
    # `default_nettype value and a version_specifier the edition does not list, where 1.0.0-1.0.216
    # swallowed any backtick line whole. The rows below are the corpus lines that census found
    # (docs/tasks/artifacts/sv_corpus_grad/directive_census/census.md: 1 615 typed directive lines in
    # 699 files, 51 illegal by the boxes on sv_2017) and that ACCEPTED on 1.0.216; each is keyed
    # must_reject WITH THE CLAUSE so its pass -> fail lands as a match. Three keys it corrects were
    # right about the FORM of the defect and wrong about the STAGE ("preprocessing" / "elaboration"):
    # the boxes are the directive's syntax, the parser is the stage that reads it since 1.0.217, and
    # the spec outranks a suite's stage tag (.3.14a). NOT re-keyed, explained by the standing svpp
    # rule instead: verilator t_lint_warn_line_bad.v (`line `__LINE__ ... - a text macro as the
    # number argument, a preprocessing dependency the isolation parse cannot resolve) and every file
    # whose only illegal directive sits inside a `define continuation (macro text, .3.39's
    # inside_define_continuation rule).
    ('sv-tests', 'tests/chapter-22/22.9--unconnected_drive-invalid-1.sv'): ('must_reject',
        "pinned .3.39: line 18 `unconnected_drive with NO argument - IEEE 1800-2017 22.9 'The directive `unconnected_drive takes one of two arguments-pull1 or pull0' (docs/systemverilog/2017/md/section-22-compiler-directives.md:698-699); the suite's ':type: simulation elaboration' is tool testimony about the STAGE and the spec outranks it (.3.14a): the argument list is the directive's syntax. Pin invalid_directive_unconnected_drive_missing.sv"),
    ('sv-tests', 'tests/chapter-22/22.9--unconnected_drive-invalid-2.sv'): ('must_reject',
        "pinned .3.39: line 18 `unconnected_drive pull2 - IEEE 1800-2017 22.9 admits pull1 or pull0 only (docs/systemverilog/2017/md/section-22-compiler-directives.md:698-699); spec outranks the suite's stage tag (.3.14a). Pin invalid_directive_unconnected_drive_bad_value.sv"),
    ('sv-tests', 'tests/chapter-22/22.9--unconnected_drive-invalid-3.sv'): ('must_reject',
        "pinned .3.39: line 19 `nounconnected_drive pull0 - IEEE 1800-2017 22.9 gives `nounconnected_drive no argument (the argument belongs to `unconnected_drive alone, docs/systemverilog/2017/md/section-22-compiler-directives.md:698-711; 22.1's directive list spells the bare form :51); the stray token is source text no item derives. Pin invalid_directive_nounconnected_drive_argument.sv"),
    ('sv-tests', 'tests/chapter-22/22.7--timescale-basic-3.sv'): ('must_reject',
        "pinned .3.39: line 18 `timescale 9 ns / 1 ps - IEEE 1800-2017 22.7 'The integers in these arguments specify an order of magnitude ... the valid integers are 1, 10, and 100' (docs/systemverilog/2017/md/section-22-compiler-directives.md:627-629), the directive's ONLY definition of time_unit (no Annex A production exists for the directive, unlike `timeunit`); the suite's ':type: simulation elaboration' is a stage claim the spec outranks (.3.14a). Pin invalid_directive_timescale_magnitude.sv"),
    ('sv-tests', 'tests/chapter-22/22.11--pragma-invalid.sv'): ('must_reject',
        "pinned .3.39: line 17 a bare `pragma - IEEE 1800-2017 22.11 Syntax 22-8 'pragma ::= `pragma pragma_name [ pragma_expression ... ]' makes pragma_name mandatory (docs/systemverilog/2017/md/section-22-compiler-directives.md:734-739). Pin invalid_directive_pragma_bare.sv"),
    ('sv-tests', 'tests/chapter-22/22.12--line-illegal-1.sv'): ('must_reject',
        'pinned .3.39: line 17 `line 1 "somefile" 3 - level 3 - IEEE 1800-2017 22.12 Syntax 22-9 \'`line number " filename " level\' with \'The number parameter shall be a positive integer ... The filename parameter shall be a string literal ... The level parameter shall be 0, 1, or 2 ... All parameters in the `line directive are required\' (docs/systemverilog/2017/md/section-22-compiler-directives.md:804-816). The suite\'s \':type: preprocessing\' names the stage a tool checks it at; the syntax is the parser\'s since 1.0.217. Pin invalid_directive_line_level.sv'),
    ('sv-tests', 'tests/chapter-22/22.12--line-illegal-2.sv'): ('must_reject',
        'pinned .3.39: line 17 `line 1 somefile 2 - an unquoted filename - IEEE 1800-2017 22.12 Syntax 22-9 \'`line number " filename " level\' with \'The number parameter shall be a positive integer ... The filename parameter shall be a string literal ... The level parameter shall be 0, 1, or 2 ... All parameters in the `line directive are required\' (docs/systemverilog/2017/md/section-22-compiler-directives.md:804-816). The suite\'s \':type: preprocessing\' names the stage a tool checks it at; the syntax is the parser\'s since 1.0.217. Pin invalid_directive_line_unquoted_filename.sv'),
    ('sv-tests', 'tests/chapter-22/22.12--line-illegal-3.sv'): ('must_reject',
        'pinned .3.39: line 17 `line -12 "somefile" 3 - a negative number - IEEE 1800-2017 22.12 Syntax 22-9 \'`line number " filename " level\' with \'The number parameter shall be a positive integer ... The filename parameter shall be a string literal ... The level parameter shall be 0, 1, or 2 ... All parameters in the `line directive are required\' (docs/systemverilog/2017/md/section-22-compiler-directives.md:804-816). The suite\'s \':type: preprocessing\' names the stage a tool checks it at; the syntax is the parser\'s since 1.0.217. Pin invalid_directive_line_zero_number.sv'),
    ('sv-tests', 'tests/chapter-22/22.12--line-illegal-4.sv'): ('must_reject',
        'pinned .3.39: line 17 `line 1 "somefile" - no level - IEEE 1800-2017 22.12 Syntax 22-9 \'`line number " filename " level\' with \'The number parameter shall be a positive integer ... The filename parameter shall be a string literal ... The level parameter shall be 0, 1, or 2 ... All parameters in the `line directive are required\' (docs/systemverilog/2017/md/section-22-compiler-directives.md:804-816). The suite\'s \':type: preprocessing\' names the stage a tool checks it at; the syntax is the parser\'s since 1.0.217. Pin invalid_directive_line_missing_level.sv'),
    ('sv-tests', 'tests/chapter-22/22.12--line-illegal-5.sv'): ('must_reject',
        'pinned .3.39: line 17 `line 1 - no filename and no level - IEEE 1800-2017 22.12 Syntax 22-9 \'`line number " filename " level\' with \'The number parameter shall be a positive integer ... The filename parameter shall be a string literal ... The level parameter shall be 0, 1, or 2 ... All parameters in the `line directive are required\' (docs/systemverilog/2017/md/section-22-compiler-directives.md:804-816). The suite\'s \':type: preprocessing\' names the stage a tool checks it at; the syntax is the parser\'s since 1.0.217. Pin invalid_directive_line_missing_level.sv'),
    ('sv2v', 'test/error/default_nettype_invalid.sv'): ('must_reject',
        'pinned .3.39: line 1 `default_nettype invalid - IEEE 1800-2017 22.8 Syntax 22-7 default_nettype_value ::= wire | tri | tri0 | tri1 | wand | triand | wor | trior | trireg | uwire | none (docs/systemverilog/2017/md/section-22-compiler-directives.md:681-686). Was SV2V_PREPROC (preprocessing-stage); the parser judges the argument since 1.0.217. Pin invalid_directive_default_nettype_unknown.sv'),
    ('sv2v', 'test/error/line_char1.sv'): ('must_reject',
        'pinned .3.39: `line 1 "asd" B - a non-numeric level; IEEE 1800-2017 22.12 \'The level parameter shall be 0, 1, or 2\' (docs/systemverilog/2017/md/section-22-compiler-directives.md:812-816). Was pinned .8b.2 preprocessing-stage; the parser judges the argument since 1.0.217. Pin invalid_directive_line_level.sv'),
    ('sv2v', 'test/error/line_char2.sv'): ('must_reject',
        'pinned .3.39: `line 1 "asd" 1B - a malformed level; IEEE 1800-2017 22.12 (docs/systemverilog/2017/md/section-22-compiler-directives.md:812-816). Was pinned .8b.2 preprocessing-stage. Pin invalid_directive_line_level.sv'),
    ('sv2v', 'test/error/line_eof.sv'): ('must_reject',
        "pinned .3.39: a bare `line at EOF - IEEE 1800-2017 22.12 'All parameters in the `line directive are required' (docs/systemverilog/2017/md/section-22-compiler-directives.md:804-816). Was pinned .8b.2 preprocessing-stage. Pin invalid_directive_line_bare.sv"),
    ('sv2v', 'test/error/line_level.sv'): ('must_reject',
        'pinned .3.39: `line 0 "asd" 3 - number 0 is not \'a positive integer\' and level 3 is not \'0, 1, or 2\'; IEEE 1800-2017 22.12 (docs/systemverilog/2017/md/section-22-compiler-directives.md:812-816). Was pinned .8b.2 preprocessing-stage. Pin invalid_directive_line_zero_number.sv'),
    ('verilator', 'test_regress/t/t_timescale_parse_bad.v'): ('must_reject',
        "pinned .3.39: lines 11-15 `timescale frump / 1xs / 2ps / 1ns / frump / 1ns / 1ps /extra - IEEE 1800-2017 22.7 Syntax 22-6 '`timescale time_unit / time_precision' with the valid integers 1, 10, 100 and units s, ms, us, ns, ps, fs (docs/systemverilog/2017/md/section-22-compiler-directives.md:615-629). The driver's golden .out reads 'syntax error' on each and the clause agrees; line 8's 1ps/1ns ORDERING is a value rule (22.7 :623-625) the parser does not judge, as at the v2005 lane's timescale3. Pins invalid_directive_timescale_*.sv"),
    ('verilator', 'test_regress/t/t_unconnected_bad.v'): ('must_reject',
        "pinned .3.39: lines 7 and 9 `unconnected_drive (bare) and `unconnected_drive pull2 - IEEE 1800-2017 22.9 'takes one of two arguments-pull1 or pull0' (docs/systemverilog/2017/md/section-22-compiler-directives.md:698-699); the driver's golden .out reads 'Bad `unconnected_drive syntax' and the clause agrees. Pins invalid_directive_unconnected_drive_missing.sv / _bad_value.sv"),
    ('verilator', 'test_regress/t/t_pp_defnettype_bad.v'): ('must_reject',
        "pinned .3.39: line 7 `default_nettype bad_none_such - IEEE 1800-2017 22.8 Syntax 22-7's eleven values (docs/systemverilog/2017/md/section-22-compiler-directives.md:681-686); line 8's `default_trireg_strength is a vendor directive the parser still tolerates as an opaque line. The t_pp_* preprocessor-target rule is overridden: the parser judges this argument since 1.0.217. Pin invalid_directive_default_nettype_unknown.sv"),
    ('verilator', 'test_regress/t/t_pp_line_bad.v'): ('must_reject',
        "pinned .3.39: lines 7-13 `line with no arguments, `line 100, `line 200 somefile, an unterminated filename, a missing level and level 3 - IEEE 1800-2017 22.12 Syntax 22-9 and its 'All parameters ... are required' (docs/systemverilog/2017/md/section-22-compiler-directives.md:804-816). The t_pp_* rule is overridden: the parser judges these arguments since 1.0.217. Pins invalid_directive_line_*.sv"),
    ('verilator', 'test_regress/t/t_pp_pragma_bad.v'): ('must_reject',
        'pinned .3.39: a bare `pragma - IEEE 1800-2017 22.11 Syntax 22-8 makes pragma_name mandatory (docs/systemverilog/2017/md/section-22-compiler-directives.md:734-739). The t_pp_* rule is overridden: the parser judges the argument since 1.0.217. Pin invalid_directive_pragma_bare.sv'),
    ('iverilog', 'ivtest/ivltests/begin_keywords_1800_2023.v'): ('must_reject',
        'pinned .3.39: line 1 `begin_keywords "1800-2023" - an EDITION CONTRAST: IEEE 1800-2017 22.14 Syntax 22-10 lists eight version_specifiers (docs/systemverilog/2017/md/section-22-compiler-directives.md:851-862), \'1800-2023\' is not among them and \'It shall be an error if an implementation does not recognize the version_specifier used\' (:869-870); IEEE 1800-2023 Syntax 22-10 adds it (docs/systemverilog/2023/md/section-22-compiler-directives.md:922-935), so the sv_2023 lane keys this row must_accept. The descriptor\'s \'explicit SV generation\' is iverilog\'s, whose keyword tables post-date the 2017 edition. Pins invalid_directive_begin_keywords_2023_under_2017.sv / control_directive_begin_keywords_2023.sv'),
    # ---- SV-CORPUS-GRAD.3.43 (PGEN-SV-CORPUS-GRAD-0371) ------------------------------------------
    # IEEE 1800-2017 35.5.5 (docs/systemverilog/2017/md/section-35-direct-programming-interface.md:485-493) restricts a
    # DPI import function's RESULT type to void, byte, shortint, int, longint, real, shortreal, chandle, string and
    # SCALAR bit / logic, and Annex A footnote 21 (docs/systemverilog/2017/md/section-41-data-read-api.md:3524,
    # "dpi_function_proto return types are restricted to small values, per 35.5.5") makes that a GRAMMAR restriction;
    # 1800-2023 reads the same (35.5.5 :493-501; footnote 26 section-Annex_A-normative-formal-syntax.md:3490). Three
    # rows carry a packed bit / logic vector (or `integer`) result UNCONDITIONALLY and are keyed must_reject WITH THE
    # PRODUCTION (the third, t_dpi_import.v, stops at byte 551 inside its `ifdef preamble, so the .13s rule routes it
    # deferred:svpp_owned / out_of_scope_with_cause until the svpp lane resolves the conditional - the citation waits
    # on the row) - spec outranks tool, the .3.38 UnsizedPacked class: sv2v converts without checking DPI result types;
    # verilator accepts 2-state vector results of <= 32 bits as an extension (svBitVecVal) and its own
    # t_dpi_result_type_bad.v rejects the > 32-bit and 4-state ones at elaboration, which is the golden .out the earlier
    # basis read as "post-parse tool error - syntax valid". Measured by the AST-field census
    # (docs/tasks/artifacts/sv_corpus_grad/dpi_result_census/census.md): 1 298 result slots over the 172 corpus files
    # that carry a DPI spec string, 24 of them illegal, in exactly these four files.
    # NOT re-keyed: verilator test_regress/t/t_dpi_result_type.v (fail, divergence:explained_svpp_macro_use) - its two
    # packed results (:114-115 `bit [ 0:0]` / `bit [31:0]`) sit under `ifndef NO_ARRAY`, so the ruling depends on the
    # preprocessing configuration the svpp resolution will fix; when that row resolves, those two lines are the next
    # 35.5.5 stop unless NO_ARRAY is defined. Owner: the svpp-divergence resolution of that row.
    # ---- SV-CORPUS-GRAD.3.48 (PGEN-SV-CORPUS-GRAD-0378) - the `[N]` PACKED dimension: a packed dimension is a RANGE in every
    # edition (A.2.5); the size form `[N]` is the UNPACKED dimension's alone. Four rows keyed must_accept for text no
    # Annex A production derives; the parser stops at the byte after the `[` on every one.
    ("sv2v", "test/core/multipack_struct_cast.sv"): ("must_reject",
        "pinned .3.48: line 4 `logic [2][3] x;` inside `struct packed { … }` - two PACKED dimensions in the size form. "
        "IEEE 1800-2017 A.2.5 packed_dimension ::= [ constant_range ] | unsized_dimension (docs/systemverilog/2017/md/section-41-data-read-api.md:1058-1062; footnote 20 docs/systemverilog/2017/md/section-41-data-read-api.md:3522 confines the unsized form to a DPI import formal) - a packed dimension is a RANGE; the size form [ constant_expression ] belongs to unpacked_dimension alone (docs/systemverilog/2017/md/section-41-data-read-api.md:1046-1050); IEEE 1800-2023 A.2.5 the same two alternatives (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1085-1089); IEEE 1364-2005 A.2.5 range ::= [ msb_constant_expression : lsb_constant_expression ] (docs/verilog/2005/md/section-Annex_A-normative-formal-syntax-definition.md:310). "
        "REJECTS at furthest_position=56 on sv_2017 and sv_2023 (the `2` after the `[`). Controls (rust/target/s348, release probe 1.0.221): `reg [1:0] b; reg [1:0][2:0] x;` ACCEPTS on all three profiles; `logic x [2]; logic y [2][3];` (the UNPACKED size form) ACCEPTS on sv_2017 / sv_2023; every packed `[N]` spelling REJECTS at the byte after the `[` on all three profiles (`logic [2] b;` 20, `logic [2][3] x;` 20, `input [4][5] inp;` 17, `word [2] array;` 45 with `word` a typedef). Pins invalid_packed_dimension_size.sv / control_packed_dimension_range.sv / control_unpacked_dimension_size.sv / accepts_invalid_v2005_unpacked_dimension_size.sv. Spec outranks the suite: sv2v, iverilog and verible all read a packed `[N]` as `[N-1:0]`, a tool extension no edition of the standard derives"),
    ("sv2v", "test/core/struct_param.sv"): ("must_reject",
        "pinned .3.48: line 23 `logic [2] b;` inside `struct packed { … }` - a PACKED dimension in the size form. "
        "IEEE 1800-2017 A.2.5 packed_dimension ::= [ constant_range ] | unsized_dimension (docs/systemverilog/2017/md/section-41-data-read-api.md:1058-1062; footnote 20 docs/systemverilog/2017/md/section-41-data-read-api.md:3522 confines the unsized form to a DPI import formal) - a packed dimension is a RANGE; the size form [ constant_expression ] belongs to unpacked_dimension alone (docs/systemverilog/2017/md/section-41-data-read-api.md:1046-1050); IEEE 1800-2023 A.2.5 the same two alternatives (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1085-1089); IEEE 1364-2005 A.2.5 range ::= [ msb_constant_expression : lsb_constant_expression ] (docs/verilog/2005/md/section-Annex_A-normative-formal-syntax-definition.md:310). "
        "REJECTS at furthest_position=522 on sv_2017 and sv_2023 (the `2` after the `[`). Controls (rust/target/s348, release probe 1.0.221): `reg [1:0] b; reg [1:0][2:0] x;` ACCEPTS on all three profiles; `logic x [2]; logic y [2][3];` (the UNPACKED size form) ACCEPTS on sv_2017 / sv_2023; every packed `[N]` spelling REJECTS at the byte after the `[` on all three profiles (`logic [2] b;` 20, `logic [2][3] x;` 20, `input [4][5] inp;` 17, `word [2] array;` 45 with `word` a typedef). Pins invalid_packed_dimension_size.sv / control_packed_dimension_range.sv / control_unpacked_dimension_size.sv / accepts_invalid_v2005_unpacked_dimension_size.sv. Spec outranks the suite: sv2v, iverilog and verible all read a packed `[N]` as `[N-1:0]`, a tool extension no edition of the standard derives"),
    ("sv2v", "test/core/unbased_unsized_default.sv"): ("must_reject",
        "pinned .3.48: line 2 `input [4][5] inp;` - a non-ANSI port declaration whose implicit net type carries two PACKED dimensions in the size form "
        "(A.2.1.2 input_declaration ::= input net_port_type list_of_port_identifiers, net_port_type ::= [ net_type ] data_type_or_implicit, "
        "implicit_data_type ::= [ signing ] { packed_dimension } docs/systemverilog/2017/md/section-41-data-read-api.md:867). "
        "IEEE 1800-2017 A.2.5 packed_dimension ::= [ constant_range ] | unsized_dimension (docs/systemverilog/2017/md/section-41-data-read-api.md:1058-1062; footnote 20 docs/systemverilog/2017/md/section-41-data-read-api.md:3522 confines the unsized form to a DPI import formal) - a packed dimension is a RANGE; the size form [ constant_expression ] belongs to unpacked_dimension alone (docs/systemverilog/2017/md/section-41-data-read-api.md:1046-1050); IEEE 1800-2023 A.2.5 the same two alternatives (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1085-1089); IEEE 1364-2005 A.2.5 range ::= [ msb_constant_expression : lsb_constant_expression ] (docs/verilog/2005/md/section-Annex_A-normative-formal-syntax-definition.md:310). "
        "REJECTS at furthest_position=33 on sv_2017 and sv_2023 (the `4` after the `[`). Controls (rust/target/s348, release probe 1.0.221): `reg [1:0] b; reg [1:0][2:0] x;` ACCEPTS on all three profiles; `logic x [2]; logic y [2][3];` (the UNPACKED size form) ACCEPTS on sv_2017 / sv_2023; every packed `[N]` spelling REJECTS at the byte after the `[` on all three profiles (`logic [2] b;` 20, `logic [2][3] x;` 20, `input [4][5] inp;` 17, `word [2] array;` 45 with `word` a typedef). Pins invalid_packed_dimension_size.sv / control_packed_dimension_range.sv / control_unpacked_dimension_size.sv / accepts_invalid_v2005_unpacked_dimension_size.sv. Spec outranks the suite: sv2v, iverilog and verible all read a packed `[N]` as `[N-1:0]`, a tool extension no edition of the standard derives"),
    ("iverilog", "ivtest/ivltests/display_bug.v"): ("must_reject",
        "pinned .3.48: line 9 `word [2] array;` with `typedef struct packed { … } word;` - a typedef'd data type followed by a PACKED dimension in the "
        "size form (A.2.2.1 data_type ::= … | [ class_scope | package_scope ] type_identifier { packed_dimension } docs/systemverilog/2017/md/section-41-data-read-api.md:855). "
        "IEEE 1800-2017 A.2.5 packed_dimension ::= [ constant_range ] | unsized_dimension (docs/systemverilog/2017/md/section-41-data-read-api.md:1058-1062; footnote 20 docs/systemverilog/2017/md/section-41-data-read-api.md:3522 confines the unsized form to a DPI import formal) - a packed dimension is a RANGE; the size form [ constant_expression ] belongs to unpacked_dimension alone (docs/systemverilog/2017/md/section-41-data-read-api.md:1046-1050); IEEE 1800-2023 A.2.5 the same two alternatives (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1085-1089); IEEE 1364-2005 A.2.5 range ::= [ msb_constant_expression : lsb_constant_expression ] (docs/verilog/2005/md/section-Annex_A-normative-formal-syntax-definition.md:310). "
        "REJECTS at furthest_position=110 on sv_2017 and sv_2023 (the `2` after the `[`); the ivtest key reads the iverilog SV dialect, which accepts it. Controls (rust/target/s348, release probe 1.0.221): `reg [1:0] b; reg [1:0][2:0] x;` ACCEPTS on all three profiles; `logic x [2]; logic y [2][3];` (the UNPACKED size form) ACCEPTS on sv_2017 / sv_2023; every packed `[N]` spelling REJECTS at the byte after the `[` on all three profiles (`logic [2] b;` 20, `logic [2][3] x;` 20, `input [4][5] inp;` 17, `word [2] array;` 45 with `word` a typedef). Pins invalid_packed_dimension_size.sv / control_packed_dimension_range.sv / control_unpacked_dimension_size.sv / accepts_invalid_v2005_unpacked_dimension_size.sv. Spec outranks the suite: sv2v, iverilog and verible all read a packed `[N]` as `[N-1:0]`, a tool extension no edition of the standard derives"),
    ("sv2v", "test/nosim/dpi.sv"): ("must_reject",
        "pinned .3.43: line 35 `import \"DPI-C\" function bit [15:0] getStimulus(input integer x);` - a PACKED vector result; IEEE 1800-2017 "
        "§35.5.5 admits scalar bit / logic only (docs/systemverilog/2017/md/section-35-direct-programming-interface.md:485-493), Annex A footnote 21 "
        "(docs/systemverilog/2017/md/section-41-data-read-api.md:3524). The line is the LRM's OWN informative example at 35.5.4 (:476; 1800-2023 :489 "
        "repeats it) transcribed - the normative subclause one paragraph later and the Annex A footnote outrank it. Every other import in the file "
        "(void, real, chandle) is admitted. The pin invalid_dpi_result_packed_bit.sv guards the form"),
    ("verilator", "test_regress/t/t_dpi_result_type_bad.v"): ("must_reject",
        "pinned .3.43: lines 80-83 `function bit [ 32:0] ... [127:0]`, 104 `function integer`, 107-117 `function logic [  0:0] ... [127:0]` - "
        "sixteen DPI import results IEEE 1800-2017 §35.5.5 does not admit (docs/systemverilog/2017/md/section-35-direct-programming-interface.md:485-493: "
        "packed vectors, integer); Annex A footnote 21 (docs/systemverilog/2017/md/section-41-data-read-api.md:3524) makes the restriction grammar, so the "
        "driver's fails=True IS a syntax verdict, not the post-parse tool error the earlier basis read. The typedef'd struct / union / array "
        "results on lines 86-101 are admitted by the grammar (a typedef is not seen through); the file rejects at line 80 regardless. "
        "The pins invalid_dpi_result_packed_bit.sv / _packed_logic.sv / _integer.sv guard the three forms"),
    ("verilator", "test_regress/t/t_dpi_import.v"): ("must_reject",
        "pinned .3.43: lines 37-41 `pure function bit [8-1:0] dpii_f_bit8 ... bit [32-1:0] dpii_f_bit32` - five UNCONDITIONAL packed-vector "
        "results (IEEE 1800-2017 §35.5.5 admits scalar bit only, docs/systemverilog/2017/md/section-35-direct-programming-interface.md:485-493; Annex A "
        "footnote 21 docs/systemverilog/2017/md/section-41-data-read-api.md:3524). Today the parser stops earlier, at byte 551 inside the file's "
        "`ifdef VCS / NC / VERILATOR` preamble (the row's standing svpp explanation); once that resolves, line 37 is the next stop. "
        "Verilator accepts <= 32-bit 2-state vector results as an extension (svBitVecVal)"),
    # ---- SV-CORPUS-GRAD.13z (PGEN-SV-CORPUS-GRAD-0366) ------------------------------------------
    # Four accepts-invalid rows whose text IEEE 1800-2017 Annex A DERIVES; the rejecting tool
    # elaborates (or names its own unsupported feature). Spec outranks tool, the .3.14a/.3.15/.3.16
    # class: must_accept WITH THE PRODUCTION, so today's PASS is a match CI re-verifies. Each has a
    # control pin (stimuli/sv/adjudication_repros/control_*.sv, .13z) that must keep ACCEPTING.
    ("sv2v", "test/error/auto_dim_int.sv"): ("must_accept",
        "pinned .13z: `integer x [] = 1;` - A.2.4 variable_decl_assignment ::= variable_identifier { variable_dimension } [ = expression ] "
        "(docs/systemverilog/2017/md/section-41-data-read-api.md:1028-1034) with A.2.5 variable_dimension ::= unsized_dimension | ... (:1070; unsized_dimension ::= [ ] :1079); "
        "sv2v rejects the dynamic array's scalar initialiser at elaboration"),
    ("sv2v", "test/error/block_start_3.sv"): ("must_accept",
        "pinned .13z: `P::Q = 1;` in an initial block - A.8.5 variable_lvalue ::= [ implicit_class_handle . | package_scope ] "
        "hierarchical_variable_identifier select (docs/systemverilog/2017/md/section-41-data-read-api.md:3199-3202); sv2v rejects the undeclared package P at elaboration"),
    ("verilator", "test_regress/t/t_concat_link_bad.v"): ("must_accept",
        "pinned .13z: `{foo_s, foo_s}.f1` - A.8.2 method_call_root ::= primary | implicit_class_handle (docs/systemverilog/2017/md/section-41-data-read-api.md:2968) with "
        "method_call_body ::= method_identifier { attribute_instance } [ ( list_of_arguments ) ] (:2942-2944) and A.8.4 primary ::= ... | "
        "concatenation [ [ range_expression ] ]; verilator's error is a LINK error (no member f1), `bar_s` is an implicit net (6.10)"),
    ("verilator", "test_regress/t/t_wire_trireg_unsup.v"): ("must_accept",
        "pinned .13z: `trireg (small) unsup_s;` - A.2.1.3 net_declaration ::= net_type [ drive_strength | charge_strength ] [ vectored | scalared ] "
        "data_type_or_implicit [ delay3 ] list_of_net_decl_assignments ; (docs/systemverilog/2017/md/section-41-data-read-api.md:805) with charge_strength ::= ( small ) | ( medium ) | ( large ) (:937); "
        "`unsup` names verilator's own unsupported feature, not the LRM's"),
    # ---- SV-CORPUS-GRAD.3.37 (PGEN-SV-CORPUS-GRAD-0366): footnote 18's `parameter` half ----------
    # Body parameters without a default, keyed must_accept because the suite's tool tolerates them
    # (sv2v converts, verilator warns, Surelog elaborates). Each read at its stop line; the grammar now
    # refuses them (1.0.211). Pins: invalid_body_parameter_without_default.sv, invalid_interface_parameter_without_default.sv,
    # invalid_class_type_parameter_without_default.sv; control_body_parameters_with_defaults.sv.
    ("sv2v", "test/core/interface_genvar_shadow.sv"): ("must_reject",
        "pinned .3.37: line 2 `parameter P;` in an INTERFACE body - IEEE 1800-2017 Annex A footnote 18 (docs/systemverilog/2017/md/section-41-data-read-api.md:3518-3520): the constant_param_expression / data_type may be omitted from a param_assignment / type_assignment ONLY within a parameter_port_list; sv2v converts it, the LRM does not derive it"),
    ("sv2v", "test/core/interface_similar.sv"): ("must_reject",
        "pinned .3.37: line 2 `parameter P;` in an INTERFACE body - IEEE 1800-2017 Annex A footnote 18 (docs/systemverilog/2017/md/section-41-data-read-api.md:3518-3520): the constant_param_expression / data_type may be omitted from a param_assignment / type_assignment ONLY within a parameter_port_list; sv2v converts it, the LRM does not derive it"),
    ("sv2v", "test/top/main.sv"): ("must_reject",
        "pinned .3.37: lines 20-21 `parameter type T; parameter U;` in the body of module top_c - IEEE 1800-2017 Annex A footnote 18 (docs/systemverilog/2017/md/section-41-data-read-api.md:3518-3520): the constant_param_expression / data_type may be omitted from a param_assignment / type_assignment ONLY within a parameter_port_list; sv2v's top-level test relies on defparam-style overrides the LRM does not derive"),
    ("verilator", "test_regress/t/t_lint_paramnodefault.v"): ("must_reject",
        "pinned .3.37: line 8 `parameter NODEF;  //<--- Warning` in a module body - IEEE 1800-2017 Annex A footnote 18 (docs/systemverilog/2017/md/section-41-data-read-api.md:3518-3520): the constant_param_expression / data_type may be omitted from a param_assignment / type_assignment ONLY within a parameter_port_list; verilator itself flags it (a lint warning), the LRM forbids it"),
    ("Surelog", "tests/MaskNeg/dut.sv"): ("must_reject",
        "pinned .3.37: line 2 `parameter logic [1:0] RESVAL;` in a module body - IEEE 1800-2017 Annex A footnote 18 (docs/systemverilog/2017/md/section-41-data-read-api.md:3518-3520): the constant_param_expression / data_type may be omitted from a param_assignment / type_assignment ONLY within a parameter_port_list; Surelog elaborates with an override, the LRM does not derive the declaration"),
    ("Surelog", "tests/InvalidTypeParam/dut.sv"): ("must_reject",
        "pinned .3.37: line 2 `parameter type P;` in an INTERFACE body - IEEE 1800-2017 Annex A footnote 18 (docs/systemverilog/2017/md/section-41-data-read-api.md:3518-3520): the constant_param_expression / data_type may be omitted from a param_assignment / type_assignment ONLY within a parameter_port_list (the test's own name says invalid)"),
    # ---- SV-CORPUS-GRAD.3.37: rows whose earlier ruling read the text as BNF-parseable when it is not ----
    # Each was `match` (pass, keyed must_accept) under 1.0.210 because the keyword hole or the missing
    # footnote made the parse succeed; on 1.0.211 they REJECT and the key, not the parser, was wrong.
    ("Surelog", "tests/ParamNoDefault/dut.sv"): ("must_reject",
        "pinned .3.37: line 2 `parameter logic [4:0] A;` in a module body - IEEE 1800-2017 Annex A footnote 18 (docs/systemverilog/2017/md/section-41-data-read-api.md:3518-3520): the default may be omitted from a param_assignment / type_assignment ONLY within a parameter_port_list; Surelog's golden log completes because it elaborates with an override"),
    ("iverilog", "ivtest/ivltests/br_gh390a.v"): ("must_reject",
        "pinned .3.37 (CORRECTS the .8b.3 cluster reading '8.15 prose ordering rule; every statement is ordinary BNF'): line 25 `super.new (name);` "
        "AFTER a `$display` statement in a constructor - IEEE 1800-2017 A.2.7 class_constructor_declaration ::= function [ class_scope ] new [ ( [ tf_port_list ] ) ] ; { block_item_declaration } [ super . new [ ( list_of_arguments ) ] ; ] { function_statement_or_null } endfunction (docs/systemverilog/2017/md/section-41-data-read-api.md:622-626): the super.new call is a fixed position BEFORE the statements, and with `new` reserved (Annex B) a later `super.new(...)` has no derivation as a statement (it is not a method_call_body). Under 1.0.210 it parsed only because `new` was an admissible identifier (a method named new); ivtest's own CE key was right"),
    ("iverilog", "ivtest/ivltests/sv_new_array_error.v"): ("must_reject",
        "pinned .3.37 (CORRECTS the earlier 'syntax itself valid' reading): line 3 `logic [1:0] array = new[4];` - IEEE 1800-2017 A.2.4 variable_decl_assignment "
        "(docs/systemverilog/2017/md/section-41-data-read-api.md:1028-1034) admits `= dynamic_array_new` ONLY after `dynamic_array_variable_identifier unsized_dimension { variable_dimension }`, and `new[4]` is not an expression (A.8.4 primary has no `new`); "
        "under 1.0.210 `new` was an identifier and `new[4]` a bit-select of it. ivtest's CE key was right"),
    ("verilator", "test_regress/t/t_class_super_new_bad_nfirst.v"): ("must_reject",
        "pinned .3.37: line 18 `super.new(imemberc);  // BAD not first` after `imemberc = 10;` - IEEE 1800-2017 A.2.7 class_constructor_declaration ::= function [ class_scope ] new [ ( [ tf_port_list ] ) ] ; { block_item_declaration } [ super . new [ ( list_of_arguments ) ] ; ] { function_statement_or_null } endfunction (docs/systemverilog/2017/md/section-41-data-read-api.md:622-626): the super.new call is a fixed position BEFORE the statements, and with `new` reserved (Annex B) a later `super.new(...)` has no derivation as a statement (it is not a method_call_body); verilator's driver expects failure and its golden is the same rule"),
    ("verilator", "test_regress/t/t_var_lifetime_module_bad.v"): ("must_reject",
        "pinned .3.37: line 8 `automatic int i;` at module level - IEEE 1800-2017 Annex A footnote 10 (docs/systemverilog/2017/md/section-41-data-read-api.md:3503-3505): no `automatic` on a data_declaration outside a procedural context; verilator's driver expects failure"),
    ("sv2v", "test/error/interface_param_mismatch_type.sv"): ("must_reject",
        "pinned .3.37 (removed from the 'instantiation binding ... the text is BNF-parseable' set): line 3 `parameter type P;` in an INTERFACE body - IEEE 1800-2017 Annex A footnote 18 (docs/systemverilog/2017/md/section-41-data-read-api.md:3518-3520): the default may be omitted from a param_assignment / type_assignment ONLY within a parameter_port_list"),
    ("sv2v", "test/error/module_param_mismatch_type.sv"): ("must_reject",
        "pinned .3.37 (removed from the 'instantiation binding ... the text is BNF-parseable' set): line 3 `parameter type P;` in a module body - IEEE 1800-2017 Annex A footnote 18 (docs/systemverilog/2017/md/section-41-data-read-api.md:3518-3520): the default may be omitted from a param_assignment / type_assignment ONLY within a parameter_port_list"),
    ("sv2v", "test/error/parameter_no_default_1.sv"): ("must_reject",
        "pinned .3.37 (removed from the 'instantiation binding ... the text is BNF-parseable' set): line 3 `parameter type X;` in a module body - IEEE 1800-2017 Annex A footnote 18 (docs/systemverilog/2017/md/section-41-data-read-api.md:3518-3520): the default may be omitted from a param_assignment / type_assignment ONLY within a parameter_port_list; the file's own name says it"),
    ("sv2v", "test/error/parameter_no_default_2.sv"): ("must_reject",
        "pinned .3.37 (removed from the 'instantiation binding ... the text is BNF-parseable' set): line 4 `parameter Y;` in a module body - IEEE 1800-2017 Annex A footnote 18 (docs/systemverilog/2017/md/section-41-data-read-api.md:3518-3520): the default may be omitted from a param_assignment / type_assignment ONLY within a parameter_port_list"),
    ("sv2v", "test/error/parameter_no_default_3.sv"): ("must_reject",
        "pinned .3.37 (removed from the 'instantiation binding ... the text is BNF-parseable' set): line 4 `parameter Y;` in a module body - IEEE 1800-2017 Annex A footnote 18 (docs/systemverilog/2017/md/section-41-data-read-api.md:3518-3520): the default may be omitted from a param_assignment / type_assignment ONLY within a parameter_port_list"),
    ("verilator", "test_regress/t/t_var_rsvd.v"): ("must_reject",
        "pinned .3.37: line 25 `reg  global;` - `global` is a reserved word (IEEE 1800-2017 Annex B Table B.1, docs/systemverilog/2017/md/section-41-data-read-api.md:3599-3858; 1800-2009+), "
        "bound here as a variable name; the file is SystemVerilog (`typedef struct packed`), so the edition-keyword pass cannot certify it as "
        "1364-2005 text and the ruling is by hand. Verilator's own comment says it tolerates reserved names 'when possible'; the LRM does not"),
    # ---- SV-CORPUS-GRAD.3.38 (PGEN-SV-CORPUS-GRAD-0369) ------------------------------------------
    # The Surelog golden log keys this unit must_accept because Surelog's parser takes the unsized arm of
    # packed_dimension at every site (its AST log: Packed_dimension > Unsized_dimension at l<5:5>) and its
    # elaboration never checks footnote 20; the unit itself names the variable `illegal`. Spec outranks
    # the golden. The iverilog twin (ivltests/br_ml20181012b.v, the same `reg [] illegal;`) is already
    # keyed must_reject by IVTEST_CE_STAGE_CLUSTERS on the same footnote; this row makes the two agree.
    ("Surelog", "tests/UnsizedPacked/dut.sv"): ("must_reject",
        "pinned .3.38: line 5 `reg [] illegal;` at module level - IEEE 1800-2017 A.2.5 packed_dimension ::= [ constant_range ] | unsized_dimension "
        "(docs/systemverilog/2017/md/section-41-data-read-api.md:1058-1062) carries Annex A footnote 20 (:3522-3523): unsized_dimension is permitted "
        "only as the sole packed dimension in a DPI import declaration; IEEE 1800-2023 keeps it verbatim as footnote 24 "
        "(docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:3487). Line 3 `reg legal [];` is the UNPACKED form (A.2.5 "
        "variable_dimension ::= unsized_dimension | ..., :1070-1079) and stays legal. Surelog's golden log completes because its parser takes "
        "the unsized arm at every site and its elaboration never checks the footnote; the pin invalid_packed_unsized_outside_dpi_import.sv guards it"),
    # ---- SV-CORPUS-GRAD.13y (PGEN-SV-CORPUS-GRAD-0364) ------------------------------------------
    # .13y (b): three rows that PASSED only through the footnote-6 mis-derivation the .3.35 grammar
    # closes (a leading statement read as an implicit-type declaration) and now stop where Annex A stops.
    ("verible", "verible/verilog/tools/kythe/testdata/array.sv"): ("must_reject",
        "pinned .13y: line 51 `int var1 = 0, var2 = 0;` AFTER two statements in an `initial begin` block. IEEE 1800-2017 A.6.3 seq_block ::= begin "
        "[ : block_identifier ] { block_item_declaration } { statement_or_null } end (docs/systemverilog/2017/md/section-41-data-read-api.md:2064-2066) - declarations precede statements, and no production "
        "admits a block_item_declaration after a statement. It parsed before 1.0.210 only because `real_arr[80] = 81;` derived as an implicit-type "
        "data declaration (Annex A footnote 6, docs/systemverilog/2017/md/section-41-data-read-api.md:3504-3505), so the later declaration looked in order; with the .3.35 grammar the statements are "
        "statements and the parse REJECTS at furthest_position=1189 (the declaration). A kythe indexing fixture keyed must_accept by verible's suite "
        "convention (a lint fixture is not a conformance claim). Spec outranks the fixture"),
    ("verible", "verible/verilog/tools/lint/testdata/shadow_parameter.sv"): ("must_reject",
        "pinned .13y: line 6 `int jumbo_packet = 0;` AFTER the statement `payload = new[ ... ];` in a function body. IEEE 1800-2017 A.2.6 "
        "function_body_declaration ::= ... { tf_item_declaration } { function_statement_or_null } ... (docs/systemverilog/2017/md/section-41-data-read-api.md:1091-1099) - declarations precede statements. "
        "It parsed before 1.0.210 only because `payload = new[...]` derived as an implicit-type declaration (footnote 6); now REJECTS at "
        "furthest_position=202. verible's own comment on the line reads `// shadowing the class name, error` - an intentionally erroneous lint fixture "
        "keyed must_accept by the suite convention. Spec outranks the fixture"),
    ("verilator", "test_regress/t/t_final.v"): ("must_reject",
        "pinned .13y: line 16 `final ;` (`// Empty test`). IEEE 1800-2017 A.6.2 final_construct ::= final function_statement (docs/systemverilog/2017/md/section-41-data-read-api.md:2022), "
        "function_statement ::= statement (docs/systemverilog/2017/md/section-41-data-read-api.md:2117); a bare `;` is statement_or_null ::= statement | { attribute_instance } ; (docs/systemverilog/2017/md/section-41-data-read-api.md:2081-2083), never a "
        "statement - `final ;` has no derivation. It parsed before 1.0.210 as an implicit-type DECLARATION of a variable named `final` "
        "(data_declaration > variable_decl > implicit, measured on the 1.0.209 grammar with the interpreter) - footnote 6 closes that arm; now REJECTS at "
        "furthest_position=383. verilator tolerates the empty final block; spec outranks the driver key"),
    # The residual-bar triage by construct: ten rows whose text IEEE 1800-2017 derives nothing for,
    # keyed must_accept by a tool that tolerates it (verilator, verible's lint fixture, Surelog).
    ("verilator", "test_regress/t/t_clk_concat5.v"): ("must_reject",
        "pinned .13y: a MIXED port list `module t ( clk /*verilator clocker*/, input clk0 ... );` - the first port omits direction, kind and type. IEEE 1800-2017 23.2.2.3 "
        "(docs/systemverilog/2017/md/section-23-modules-and-hierarchy.md:472-474): 'If the direction, port kind, and data type are all omitted for the first port in the port list, then all ports shall be assumed to be "
        "non-ANSI style ... Otherwise, all ports shall be assumed to be ANSI style' - so the list is NON-ANSI, and a non-ANSI list_of_ports (docs/systemverilog/2017/md/section-41-data-read-api.md:246) takes "
        "`port ::= [ port_expression ] | . port_identifier ( [ port_expression ] )` (docs/systemverilog/2017/md/section-41-data-read-api.md:266) only: the typed items after it have no derivation. The parser "
        "instead admits the header as ANSI (the letter of A.1.3 ansi_port_declaration makes every header optional) and REJECTS at furthest_position=1384 on the BODY "
        "declaration `input clk;`, which under an ANSI header is illegal by A.1.2 (module_ansi_header takes non_port_module_item, docs/systemverilog/2017/md/section-41-data-read-api.md:108-113) - the .13x.b "
        "t_assert_imm_nz_bad.v ruling. Either reading rejects the file; the tool that keyed it must_accept tolerates the mixture. Spec outranks the driver key"),
    ("verilator", "test_regress/t/t_clk_concat6.v"): ("must_reject",
        "pinned .13y: a MIXED port list `module t ( clk /*verilator clocker*/ /*verilator public_flat*/, input clk0 ... );` - the first port omits direction, kind and type. IEEE 1800-2017 23.2.2.3 "
        "(docs/systemverilog/2017/md/section-23-modules-and-hierarchy.md:472-474): 'If the direction, port kind, and data type are all omitted for the first port in the port list, then all ports shall be assumed to be "
        "non-ANSI style ... Otherwise, all ports shall be assumed to be ANSI style' - so the list is NON-ANSI, and a non-ANSI list_of_ports (docs/systemverilog/2017/md/section-41-data-read-api.md:246) takes "
        "`port ::= [ port_expression ] | . port_identifier ( [ port_expression ] )` (docs/systemverilog/2017/md/section-41-data-read-api.md:266) only: the typed items after it have no derivation. The parser "
        "instead admits the header as ANSI (the letter of A.1.3 ansi_port_declaration makes every header optional) and REJECTS at furthest_position=1568 on the BODY "
        "declaration `input clk;`, which under an ANSI header is illegal by A.1.2 (module_ansi_header takes non_port_module_item, docs/systemverilog/2017/md/section-41-data-read-api.md:108-113) - the .13x.b "
        "t_assert_imm_nz_bad.v ruling. Either reading rejects the file; the tool that keyed it must_accept tolerates the mixture. Spec outranks the driver key"),
    ("verilator", "test_regress/t/t_comb_do_not_convert_to.v"): ("must_reject",
        "pinned .13y: a MIXED port list `module t ( clk, input wire i, output reg o_0 ... );` - the first port omits direction, kind and type. IEEE 1800-2017 23.2.2.3 "
        "(docs/systemverilog/2017/md/section-23-modules-and-hierarchy.md:472-474): 'If the direction, port kind, and data type are all omitted for the first port in the port list, then all ports shall be assumed to be "
        "non-ANSI style ... Otherwise, all ports shall be assumed to be ANSI style' - so the list is NON-ANSI, and a non-ANSI list_of_ports (docs/systemverilog/2017/md/section-41-data-read-api.md:246) takes "
        "`port ::= [ port_expression ] | . port_identifier ( [ port_expression ] )` (docs/systemverilog/2017/md/section-41-data-read-api.md:266) only: the typed items after it have no derivation. The parser "
        "instead admits the header as ANSI (the letter of A.1.3 ansi_port_declaration makes every header optional) and REJECTS at furthest_position=358 on the BODY "
        "declaration `input clk;`, which under an ANSI header is illegal by A.1.2 (module_ansi_header takes non_port_module_item, docs/systemverilog/2017/md/section-41-data-read-api.md:108-113) - the .13x.b "
        "t_assert_imm_nz_bad.v ruling. Either reading rejects the file; the tool that keyed it must_accept tolerates the mixture. Spec outranks the driver key"),
    ("verible", "verible/verilog/tools/kythe/testdata/module_port_type_forwarding.sv"): ("must_reject",
        "pinned .13y: a MIXED port list `module ( n1, bit b1, b2, ... );` - the first port omits direction, kind and type. IEEE 1800-2017 23.2.2.3 "
        "(docs/systemverilog/2017/md/section-23-modules-and-hierarchy.md:472-474): 'If the direction, port kind, and data type are all omitted for the first port in the port list, then all ports shall be assumed to be "
        "non-ANSI style ... Otherwise, all ports shall be assumed to be ANSI style' - so the list is NON-ANSI, and a non-ANSI list_of_ports (docs/systemverilog/2017/md/section-41-data-read-api.md:246) takes "
        "`port ::= [ port_expression ] | . port_identifier ( [ port_expression ] )` (docs/systemverilog/2017/md/section-41-data-read-api.md:266) only: the typed items after it have no derivation. The parser "
        "instead admits the header as ANSI (the letter of A.1.3 ansi_port_declaration makes every header optional) and REJECTS at furthest_position=1184 on the BODY "
        "declaration `input n1;`, which under an ANSI header is illegal by A.1.2 (module_ansi_header takes non_port_module_item, docs/systemverilog/2017/md/section-41-data-read-api.md:108-113) - the .13x.b "
        "t_assert_imm_nz_bad.v ruling. Either reading rejects the file; the tool that keyed it must_accept tolerates the mixture. Spec outranks the driver key"),
    ("verilator", "test_regress/t/t_lparam_assign_iface_typedef2.v"): ("must_reject",
        "pinned .13y: `localparam type p0_t = if0.rq_t;` - a type named THROUGH AN INTERFACE INSTANCE. IEEE 1800-2017 A.2.2.1 data_type (docs/systemverilog/2017/md/section-41-data-read-api.md:842) admits a qualified type identifier "
        "only as `[ class_scope | package_scope ] type_identifier { packed_dimension }` - an instance name is neither a class scope nor a package scope, and no arm "
        "of data_type, type_assignment (A.2.4) or type_reference reaches a typedef through a hierarchical name; IEEE 1800-2023 A.2.2.1 data_type (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:825) is "
        "the same on this point. REJECTS at furthest_position=407 on sv_2017 AND sv_2023 (the .14a census: not a 2023_only_pass row) - a verilator extension "
        "its driver keys must_accept. Spec outranks the driver key"),
    ("verilator", "test_regress/t/t_lparam_assign_iface_typedef_nested_mod1.v"): ("must_reject",
        "pinned .13y: `localparam type bus_rq_t = bus_io.rq_t;` - a type named THROUGH AN INTERFACE INSTANCE. IEEE 1800-2017 A.2.2.1 data_type (docs/systemverilog/2017/md/section-41-data-read-api.md:842) admits a qualified type identifier "
        "only as `[ class_scope | package_scope ] type_identifier { packed_dimension }` - an instance name is neither a class scope nor a package scope, and no arm "
        "of data_type, type_assignment (A.2.4) or type_reference reaches a typedef through a hierarchical name; IEEE 1800-2023 A.2.2.1 data_type (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:825) is "
        "the same on this point. REJECTS at furthest_position=521 on sv_2017 AND sv_2023 (the .14a census: not a 2023_only_pass row) - a verilator extension "
        "its driver keys must_accept. Spec outranks the driver key"),
    ("verilator", "test_regress/t/t_lparam_assign_iface_typedef_nested_mpkg4.v"): ("must_reject",
        "pinned .13y: `localparam type cc_index_t = tb_io.cc_index_t;` - a type named THROUGH AN INTERFACE INSTANCE. IEEE 1800-2017 A.2.2.1 data_type (docs/systemverilog/2017/md/section-41-data-read-api.md:842) admits a qualified type identifier "
        "only as `[ class_scope | package_scope ] type_identifier { packed_dimension }` - an instance name is neither a class scope nor a package scope, and no arm "
        "of data_type, type_assignment (A.2.4) or type_reference reaches a typedef through a hierarchical name; IEEE 1800-2023 A.2.2.1 data_type (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:825) is "
        "the same on this point. REJECTS at furthest_position=789 on sv_2017 AND sv_2023 (the .14a census: not a 2023_only_pass row) - a verilator extension "
        "its driver keys must_accept. Spec outranks the driver key"),
    ("verilator", "test_regress/t/t_lparam_assign_iface_typedef_nested_mpkg5.v"): ("must_reject",
        "pinned .13y: `localparam type cc_index_t = tb_io.cc_index_t;` - a type named THROUGH AN INTERFACE INSTANCE. IEEE 1800-2017 A.2.2.1 data_type (docs/systemverilog/2017/md/section-41-data-read-api.md:842) admits a qualified type identifier "
        "only as `[ class_scope | package_scope ] type_identifier { packed_dimension }` - an instance name is neither a class scope nor a package scope, and no arm "
        "of data_type, type_assignment (A.2.4) or type_reference reaches a typedef through a hierarchical name; IEEE 1800-2023 A.2.2.1 data_type (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:825) is "
        "the same on this point. REJECTS at furthest_position=754 on sv_2017 AND sv_2023 (the .14a census: not a 2023_only_pass row) - a verilator extension "
        "its driver keys must_accept. Spec outranks the driver key"),
    # ---- SVPP-EXPANSION.6b.1 (PGEN-SVPP-EXPANSION-0010) --------------------------------------------
    # The pipeline lane (`.6a`'s re-adjudication) reached 34 verilator rows the raw parse never did -
    # each explained away as `explained_svpp_macro_use` because the raw parse stopped on a macro use
    # first - and every one rejects, under the derived context, on a construct no edition derives:
    # a type reached through two or more hierarchical steps (17), a type named through an interface
    # instance in a type assignment (16, the .13y ruling) and an enum with a range but no base type
    # (1). Same taxonomy as the .13y block below: text a vendor tolerates that the standard cannot
    # derive, where the spec outranks the driver key. ⛔ The RAW lane cannot cash these pins (its
    # parse stops on the macro token, above the construct), so the reject-arm demotion keeps them
    # out of the raw `match` column; the pipeline re-adjudication (`pipeline_readjudication.py`,
    # which reads the EXPECTED column) is where a must_reject row rejecting at the pinned line
    # becomes a match under expansion.
    ("verilator", "test_regress/t/t_enum_type_methods.v"): ("must_reject",
        "pinned SVPP-EXPANSION.6b.1: line 17 `typedef enum [3:0] {` - an enum with a packed range and NO base type. IEEE 1800-2017 A.2.2.1 data_type (docs/systemverilog/2017/md/section-41-data-read-api.md:850) admits `enum [ enum_base_type ] { ... } { packed_dimension }` and enum_base_type (docs/systemverilog/2017/md/section-41-data-read-api.md:868-873) is `integer_atom_type [ signing ] | integer_vector_type [ signing ] [ packed_dimension ] | type_identifier [ packed_dimension ]` - a bare `[3:0]` is none of them (the dimension belongs to a keyword type, `enum logic [3:0]`); IEEE 1800-2023 A.2.2.1 is the same on this point. The raw parse stops on the file's macro use first; under the pipeline (`--preprocess`, the derived verilator context) REJECTS at line 17 on sv_2017 AND sv_2023 (measured on the corpus file and on the minimal reproducer `typedef enum [3:0] { A = 1, B = 3 } e;` - furthest_position=22 on both, while the control `typedef enum logic [3:0] { A = 1, B = 3 } e;` ACCEPTS on both). A verilator extension its driver keys must_accept. Spec outranks the driver key"),
    ("verilator", "test_regress/t/t_lparam_assign_iface_array_typedef.v"): ("must_reject",
        "pinned SVPP-EXPANSION.6b.1: line 29 `localparam type tgt_id_t = bus_tgt_io.id_t;` - a type named THROUGH AN INTERFACE INSTANCE, the .13y ruling extended to the rows the raw lane never reached. IEEE 1800-2017 A.2.2.1 data_type (docs/systemverilog/2017/md/section-41-data-read-api.md:842) admits a qualified type identifier only as `[ class_scope | package_scope ] type_identifier { packed_dimension }` and A.2.4 type_assignment (docs/systemverilog/2017/md/section-41-data-read-api.md:1013) takes a data_type - an instance name is neither a class scope nor a package scope; IEEE 1800-2023 6.18 says it in prose (docs/systemverilog/2023/md/section-6-data-types.md:1839: 'hierarchical references to type identifiers shall not be allowed'). The raw parse stops on the file's macro use first; under the pipeline (`--preprocess`, the derived verilator context) REJECTS at that line on sv_2017 AND sv_2023 (the minimal reproducer `localparam type u = i.t;` with `i` an interface instance: furthest_position=87 on both; the control `localparam type u = p::t;` ACCEPTS on both). A verilator extension its driver keys must_accept. Spec outranks the driver key"),
    ("verilator", "test_regress/t/t_lparam_assign_iface_array_typedef2.v"): ("must_reject",
        "pinned SVPP-EXPANSION.6b.1: line 30 `localparam type tgt_id_t = bus_tgt_io_a[0].id_t;` - a type named THROUGH AN INTERFACE INSTANCE, the .13y ruling extended to the rows the raw lane never reached. IEEE 1800-2017 A.2.2.1 data_type (docs/systemverilog/2017/md/section-41-data-read-api.md:842) admits a qualified type identifier only as `[ class_scope | package_scope ] type_identifier { packed_dimension }` and A.2.4 type_assignment (docs/systemverilog/2017/md/section-41-data-read-api.md:1013) takes a data_type - an instance name is neither a class scope nor a package scope; IEEE 1800-2023 6.18 says it in prose (docs/systemverilog/2023/md/section-6-data-types.md:1839: 'hierarchical references to type identifiers shall not be allowed'). The raw parse stops on the file's macro use first; under the pipeline (`--preprocess`, the derived verilator context) REJECTS at that line on sv_2017 AND sv_2023 (the minimal reproducer `localparam type u = i.t;` with `i` an interface instance: furthest_position=87 on both; the control `localparam type u = p::t;` ACCEPTS on both). A verilator extension its driver keys must_accept. Spec outranks the driver key"),
    ("verilator", "test_regress/t/t_lparam_assign_iface_typedef.v"): ("must_reject",
        "pinned SVPP-EXPANSION.6b.1: line 34 `localparam type p0_rq_t = if0.rq_t;` - a type named THROUGH AN INTERFACE INSTANCE, the .13y ruling extended to the rows the raw lane never reached. IEEE 1800-2017 A.2.2.1 data_type (docs/systemverilog/2017/md/section-41-data-read-api.md:842) admits a qualified type identifier only as `[ class_scope | package_scope ] type_identifier { packed_dimension }` and A.2.4 type_assignment (docs/systemverilog/2017/md/section-41-data-read-api.md:1013) takes a data_type - an instance name is neither a class scope nor a package scope; IEEE 1800-2023 6.18 says it in prose (docs/systemverilog/2023/md/section-6-data-types.md:1839: 'hierarchical references to type identifiers shall not be allowed'). The raw parse stops on the file's macro use first; under the pipeline (`--preprocess`, the derived verilator context) REJECTS at that line on sv_2017 AND sv_2023 (the minimal reproducer `localparam type u = i.t;` with `i` an interface instance: furthest_position=87 on both; the control `localparam type u = p::t;` ACCEPTS on both). A verilator extension its driver keys must_accept. Spec outranks the driver key"),
    ("verilator", "test_regress/t/t_lparam_assign_iface_typedef3.v"): ("must_reject",
        "pinned SVPP-EXPANSION.6b.1: line 32 `localparam type p0_rq_t = if0[0].rq_t;` - a type named THROUGH AN INTERFACE INSTANCE, the .13y ruling extended to the rows the raw lane never reached. IEEE 1800-2017 A.2.2.1 data_type (docs/systemverilog/2017/md/section-41-data-read-api.md:842) admits a qualified type identifier only as `[ class_scope | package_scope ] type_identifier { packed_dimension }` and A.2.4 type_assignment (docs/systemverilog/2017/md/section-41-data-read-api.md:1013) takes a data_type - an instance name is neither a class scope nor a package scope; IEEE 1800-2023 6.18 says it in prose (docs/systemverilog/2023/md/section-6-data-types.md:1839: 'hierarchical references to type identifiers shall not be allowed'). The raw parse stops on the file's macro use first; under the pipeline (`--preprocess`, the derived verilator context) REJECTS at that line on sv_2017 AND sv_2023 (the minimal reproducer `localparam type u = i.t;` with `i` an interface instance: furthest_position=87 on both; the control `localparam type u = p::t;` ACCEPTS on both). A verilator extension its driver keys must_accept. Spec outranks the driver key"),
    ("verilator", "test_regress/t/t_lparam_assign_iface_typedef_nested2.v"): ("must_reject",
        "pinned SVPP-EXPANSION.6b.1: line 42 `localparam type p0_rq2_t = if0.y_if0.rq2_t;` - a type named THROUGH AN INTERFACE INSTANCE, the .13y ruling extended to the rows the raw lane never reached. IEEE 1800-2017 A.2.2.1 data_type (docs/systemverilog/2017/md/section-41-data-read-api.md:842) admits a qualified type identifier only as `[ class_scope | package_scope ] type_identifier { packed_dimension }` and A.2.4 type_assignment (docs/systemverilog/2017/md/section-41-data-read-api.md:1013) takes a data_type - an instance name is neither a class scope nor a package scope; IEEE 1800-2023 6.18 says it in prose (docs/systemverilog/2023/md/section-6-data-types.md:1839: 'hierarchical references to type identifiers shall not be allowed'). The raw parse stops on the file's macro use first; under the pipeline (`--preprocess`, the derived verilator context) REJECTS at that line on sv_2017 AND sv_2023 (the minimal reproducer `localparam type u = i.t;` with `i` an interface instance: furthest_position=87 on both; the control `localparam type u = p::t;` ACCEPTS on both). A verilator extension its driver keys must_accept. Spec outranks the driver key"),
    ("verilator", "test_regress/t/t_lparam_assign_iface_typedef_nested3.v"): ("must_reject",
        "pinned SVPP-EXPANSION.6b.1: line 43 `localparam type p0_rq2_t = if0.y_if0.rq2_t;` - a type named THROUGH AN INTERFACE INSTANCE, the .13y ruling extended to the rows the raw lane never reached. IEEE 1800-2017 A.2.2.1 data_type (docs/systemverilog/2017/md/section-41-data-read-api.md:842) admits a qualified type identifier only as `[ class_scope | package_scope ] type_identifier { packed_dimension }` and A.2.4 type_assignment (docs/systemverilog/2017/md/section-41-data-read-api.md:1013) takes a data_type - an instance name is neither a class scope nor a package scope; IEEE 1800-2023 6.18 says it in prose (docs/systemverilog/2023/md/section-6-data-types.md:1839: 'hierarchical references to type identifiers shall not be allowed'). The raw parse stops on the file's macro use first; under the pipeline (`--preprocess`, the derived verilator context) REJECTS at that line on sv_2017 AND sv_2023 (the minimal reproducer `localparam type u = i.t;` with `i` an interface instance: furthest_position=87 on both; the control `localparam type u = p::t;` ACCEPTS on both). A verilator extension its driver keys must_accept. Spec outranks the driver key"),
    ("verilator", "test_regress/t/t_lparam_assign_iface_typedef_nested4.v"): ("must_reject",
        "pinned SVPP-EXPANSION.6b.1: line 39 `localparam type p0_rq2_t = if0.y_if0.rq2_t;` - a type named THROUGH AN INTERFACE INSTANCE, the .13y ruling extended to the rows the raw lane never reached. IEEE 1800-2017 A.2.2.1 data_type (docs/systemverilog/2017/md/section-41-data-read-api.md:842) admits a qualified type identifier only as `[ class_scope | package_scope ] type_identifier { packed_dimension }` and A.2.4 type_assignment (docs/systemverilog/2017/md/section-41-data-read-api.md:1013) takes a data_type - an instance name is neither a class scope nor a package scope; IEEE 1800-2023 6.18 says it in prose (docs/systemverilog/2023/md/section-6-data-types.md:1839: 'hierarchical references to type identifiers shall not be allowed'). The raw parse stops on the file's macro use first; under the pipeline (`--preprocess`, the derived verilator context) REJECTS at that line on sv_2017 AND sv_2023 (the minimal reproducer `localparam type u = i.t;` with `i` an interface instance: furthest_position=87 on both; the control `localparam type u = p::t;` ACCEPTS on both). A verilator extension its driver keys must_accept. Spec outranks the driver key"),
    ("verilator", "test_regress/t/t_lparam_assign_iface_typedef_nested5.v"): ("must_reject",
        "pinned SVPP-EXPANSION.6b.1: line 49 `localparam type p0_rq2_t = if0.y_if0.rq2_t;` - a type named THROUGH AN INTERFACE INSTANCE, the .13y ruling extended to the rows the raw lane never reached. IEEE 1800-2017 A.2.2.1 data_type (docs/systemverilog/2017/md/section-41-data-read-api.md:842) admits a qualified type identifier only as `[ class_scope | package_scope ] type_identifier { packed_dimension }` and A.2.4 type_assignment (docs/systemverilog/2017/md/section-41-data-read-api.md:1013) takes a data_type - an instance name is neither a class scope nor a package scope; IEEE 1800-2023 6.18 says it in prose (docs/systemverilog/2023/md/section-6-data-types.md:1839: 'hierarchical references to type identifiers shall not be allowed'). The raw parse stops on the file's macro use first; under the pipeline (`--preprocess`, the derived verilator context) REJECTS at that line on sv_2017 AND sv_2023 (the minimal reproducer `localparam type u = i.t;` with `i` an interface instance: furthest_position=87 on both; the control `localparam type u = p::t;` ACCEPTS on both). A verilator extension its driver keys must_accept. Spec outranks the driver key"),
    ("verilator", "test_regress/t/t_lparam_assign_iface_typedef_nested_mod2.v"): ("must_reject",
        "pinned SVPP-EXPANSION.6b.1: line 31 `localparam type bus_rq_t = bus_io.rq_t;` - a type named THROUGH AN INTERFACE INSTANCE, the .13y ruling extended to the rows the raw lane never reached. IEEE 1800-2017 A.2.2.1 data_type (docs/systemverilog/2017/md/section-41-data-read-api.md:842) admits a qualified type identifier only as `[ class_scope | package_scope ] type_identifier { packed_dimension }` and A.2.4 type_assignment (docs/systemverilog/2017/md/section-41-data-read-api.md:1013) takes a data_type - an instance name is neither a class scope nor a package scope; IEEE 1800-2023 6.18 says it in prose (docs/systemverilog/2023/md/section-6-data-types.md:1839: 'hierarchical references to type identifiers shall not be allowed'). The raw parse stops on the file's macro use first; under the pipeline (`--preprocess`, the derived verilator context) REJECTS at that line on sv_2017 AND sv_2023 (the minimal reproducer `localparam type u = i.t;` with `i` an interface instance: furthest_position=87 on both; the control `localparam type u = p::t;` ACCEPTS on both). A verilator extension its driver keys must_accept. Spec outranks the driver key"),
    ("verilator", "test_regress/t/t_lparam_assign_iface_typedef_nested_mod3.v"): ("must_reject",
        "pinned SVPP-EXPANSION.6b.1: line 39 `localparam type p0_rq2_t = if0.y_if0.rq2_t;` - a type named THROUGH AN INTERFACE INSTANCE, the .13y ruling extended to the rows the raw lane never reached. IEEE 1800-2017 A.2.2.1 data_type (docs/systemverilog/2017/md/section-41-data-read-api.md:842) admits a qualified type identifier only as `[ class_scope | package_scope ] type_identifier { packed_dimension }` and A.2.4 type_assignment (docs/systemverilog/2017/md/section-41-data-read-api.md:1013) takes a data_type - an instance name is neither a class scope nor a package scope; IEEE 1800-2023 6.18 says it in prose (docs/systemverilog/2023/md/section-6-data-types.md:1839: 'hierarchical references to type identifiers shall not be allowed'). The raw parse stops on the file's macro use first; under the pipeline (`--preprocess`, the derived verilator context) REJECTS at that line on sv_2017 AND sv_2023 (the minimal reproducer `localparam type u = i.t;` with `i` an interface instance: furthest_position=87 on both; the control `localparam type u = p::t;` ACCEPTS on both). A verilator extension its driver keys must_accept. Spec outranks the driver key"),
    ("verilator", "test_regress/t/t_lparam_assign_iface_typedef_nested_mpkg1.v"): ("must_reject",
        "pinned SVPP-EXPANSION.6b.1: line 40 `localparam type bus_rq_t = bus_io.rq_t;` - a type named THROUGH AN INTERFACE INSTANCE, the .13y ruling extended to the rows the raw lane never reached. IEEE 1800-2017 A.2.2.1 data_type (docs/systemverilog/2017/md/section-41-data-read-api.md:842) admits a qualified type identifier only as `[ class_scope | package_scope ] type_identifier { packed_dimension }` and A.2.4 type_assignment (docs/systemverilog/2017/md/section-41-data-read-api.md:1013) takes a data_type - an instance name is neither a class scope nor a package scope; IEEE 1800-2023 6.18 says it in prose (docs/systemverilog/2023/md/section-6-data-types.md:1839: 'hierarchical references to type identifiers shall not be allowed'). The raw parse stops on the file's macro use first; under the pipeline (`--preprocess`, the derived verilator context) REJECTS at that line on sv_2017 AND sv_2023 (the minimal reproducer `localparam type u = i.t;` with `i` an interface instance: furthest_position=87 on both; the control `localparam type u = p::t;` ACCEPTS on both). A verilator extension its driver keys must_accept. Spec outranks the driver key"),
    ("verilator", "test_regress/t/t_lparam_assign_iface_typedef_nested_pkg.v"): ("must_reject",
        "pinned SVPP-EXPANSION.6b.1: line 57 `localparam type p0_rq2_t = if0.y_if0.rq2_t;` - a type named THROUGH AN INTERFACE INSTANCE, the .13y ruling extended to the rows the raw lane never reached. IEEE 1800-2017 A.2.2.1 data_type (docs/systemverilog/2017/md/section-41-data-read-api.md:842) admits a qualified type identifier only as `[ class_scope | package_scope ] type_identifier { packed_dimension }` and A.2.4 type_assignment (docs/systemverilog/2017/md/section-41-data-read-api.md:1013) takes a data_type - an instance name is neither a class scope nor a package scope; IEEE 1800-2023 6.18 says it in prose (docs/systemverilog/2023/md/section-6-data-types.md:1839: 'hierarchical references to type identifiers shall not be allowed'). The raw parse stops on the file's macro use first; under the pipeline (`--preprocess`, the derived verilator context) REJECTS at that line on sv_2017 AND sv_2023 (the minimal reproducer `localparam type u = i.t;` with `i` an interface instance: furthest_position=87 on both; the control `localparam type u = p::t;` ACCEPTS on both). A verilator extension its driver keys must_accept. Spec outranks the driver key"),
    ("verilator", "test_regress/t/t_lparam_assign_iface_typedef_nested_pkg2.v"): ("must_reject",
        "pinned SVPP-EXPANSION.6b.1: line 57 `localparam type p0_rq2_t = if0.y_if0.rq2_t;` - a type named THROUGH AN INTERFACE INSTANCE, the .13y ruling extended to the rows the raw lane never reached. IEEE 1800-2017 A.2.2.1 data_type (docs/systemverilog/2017/md/section-41-data-read-api.md:842) admits a qualified type identifier only as `[ class_scope | package_scope ] type_identifier { packed_dimension }` and A.2.4 type_assignment (docs/systemverilog/2017/md/section-41-data-read-api.md:1013) takes a data_type - an instance name is neither a class scope nor a package scope; IEEE 1800-2023 6.18 says it in prose (docs/systemverilog/2023/md/section-6-data-types.md:1839: 'hierarchical references to type identifiers shall not be allowed'). The raw parse stops on the file's macro use first; under the pipeline (`--preprocess`, the derived verilator context) REJECTS at that line on sv_2017 AND sv_2023 (the minimal reproducer `localparam type u = i.t;` with `i` an interface instance: furthest_position=87 on both; the control `localparam type u = p::t;` ACCEPTS on both). A verilator extension its driver keys must_accept. Spec outranks the driver key"),
    ("verilator", "test_regress/t/t_lparam_assign_iface_typedef_nested_pkg3.v"): ("must_reject",
        "pinned SVPP-EXPANSION.6b.1: line 30 `localparam type cfg_id_t = bus_io.id_t;` - a type named THROUGH AN INTERFACE INSTANCE, the .13y ruling extended to the rows the raw lane never reached. IEEE 1800-2017 A.2.2.1 data_type (docs/systemverilog/2017/md/section-41-data-read-api.md:842) admits a qualified type identifier only as `[ class_scope | package_scope ] type_identifier { packed_dimension }` and A.2.4 type_assignment (docs/systemverilog/2017/md/section-41-data-read-api.md:1013) takes a data_type - an instance name is neither a class scope nor a package scope; IEEE 1800-2023 6.18 says it in prose (docs/systemverilog/2023/md/section-6-data-types.md:1839: 'hierarchical references to type identifiers shall not be allowed'). The raw parse stops on the file's macro use first; under the pipeline (`--preprocess`, the derived verilator context) REJECTS at that line on sv_2017 AND sv_2023 (the minimal reproducer `localparam type u = i.t;` with `i` an interface instance: furthest_position=87 on both; the control `localparam type u = p::t;` ACCEPTS on both). A verilator extension its driver keys must_accept. Spec outranks the driver key"),
    ("verilator", "test_regress/t/t_lparam_dep_iface10.v"): ("must_reject",
        "pinned SVPP-EXPANSION.6b.1: line 44 `typedef io.types_mul.a_t a_mul_t;` - a typedef reaching a type through TWO OR MORE hierarchical steps. IEEE 1800-2017 A.2.1.3 type_declaration (docs/systemverilog/2017/md/section-41-data-read-api.md:825) admits exactly ONE `interface_instance_identifier constant_bit_select . type_identifier` before the new name - one instance, one dot - and IEEE 1800-2023 A.2.1.3 (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:800) narrows even that to an interface PORT (6.18, docs/systemverilog/2023/md/section-6-data-types.md:1839-1840: hierarchical references to type identifiers are not allowed; references through ports are not hierarchical). The raw parse stops on the file's macro use first; under the pipeline (`--preprocess`, the derived verilator context) REJECTS at that line on sv_2017 AND sv_2023, exactly at the second dot (the one-step control `typedef i.t u;` ACCEPTS on both - the letter of 2017 A.2.1.3). A verilator extension its driver keys must_accept. Spec outranks the driver key"),
    ("verilator", "test_regress/t/t_lparam_dep_iface14.v"): ("must_reject",
        "pinned SVPP-EXPANSION.6b.1: line 79 `typedef types.a_if0.a_t a_t;` - a typedef reaching a type through TWO OR MORE hierarchical steps. IEEE 1800-2017 A.2.1.3 type_declaration (docs/systemverilog/2017/md/section-41-data-read-api.md:825) admits exactly ONE `interface_instance_identifier constant_bit_select . type_identifier` before the new name - one instance, one dot - and IEEE 1800-2023 A.2.1.3 (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:800) narrows even that to an interface PORT (6.18, docs/systemverilog/2023/md/section-6-data-types.md:1839-1840: hierarchical references to type identifiers are not allowed; references through ports are not hierarchical). The raw parse stops on the file's macro use first; under the pipeline (`--preprocess`, the derived verilator context) REJECTS at that line on sv_2017 AND sv_2023, exactly at the second dot (the one-step control `typedef i.t u;` ACCEPTS on both - the letter of 2017 A.2.1.3). A verilator extension its driver keys must_accept. Spec outranks the driver key"),
    ("verilator", "test_regress/t/t_lparam_dep_iface15.v"): ("must_reject",
        "pinned SVPP-EXPANSION.6b.1: line 99 `typedef sc_io.types.a_if0.a_t inner_t;` - a typedef reaching a type through TWO OR MORE hierarchical steps. IEEE 1800-2017 A.2.1.3 type_declaration (docs/systemverilog/2017/md/section-41-data-read-api.md:825) admits exactly ONE `interface_instance_identifier constant_bit_select . type_identifier` before the new name - one instance, one dot - and IEEE 1800-2023 A.2.1.3 (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:800) narrows even that to an interface PORT (6.18, docs/systemverilog/2023/md/section-6-data-types.md:1839-1840: hierarchical references to type identifiers are not allowed; references through ports are not hierarchical). The raw parse stops on the file's macro use first; under the pipeline (`--preprocess`, the derived verilator context) REJECTS at that line on sv_2017 AND sv_2023, exactly at the second dot (the one-step control `typedef i.t u;` ACCEPTS on both - the letter of 2017 A.2.1.3). A verilator extension its driver keys must_accept. Spec outranks the driver key"),
    ("verilator", "test_regress/t/t_lparam_dep_iface16.v"): ("must_reject",
        "pinned SVPP-EXPANSION.6b.1: line 96 `typedef sc_io.types.a_if0.a_t inner_t;` - a typedef reaching a type through TWO OR MORE hierarchical steps. IEEE 1800-2017 A.2.1.3 type_declaration (docs/systemverilog/2017/md/section-41-data-read-api.md:825) admits exactly ONE `interface_instance_identifier constant_bit_select . type_identifier` before the new name - one instance, one dot - and IEEE 1800-2023 A.2.1.3 (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:800) narrows even that to an interface PORT (6.18, docs/systemverilog/2023/md/section-6-data-types.md:1839-1840: hierarchical references to type identifiers are not allowed; references through ports are not hierarchical). The raw parse stops on the file's macro use first; under the pipeline (`--preprocess`, the derived verilator context) REJECTS at that line on sv_2017 AND sv_2023, exactly at the second dot (the one-step control `typedef i.t u;` ACCEPTS on both - the letter of 2017 A.2.1.3). A verilator extension its driver keys must_accept. Spec outranks the driver key"),
    ("verilator", "test_regress/t/t_lparam_dep_iface2.v"): ("must_reject",
        "pinned SVPP-EXPANSION.6b.1: line 58 `typedef io.types.a_t a_t;` - a typedef reaching a type through TWO OR MORE hierarchical steps. IEEE 1800-2017 A.2.1.3 type_declaration (docs/systemverilog/2017/md/section-41-data-read-api.md:825) admits exactly ONE `interface_instance_identifier constant_bit_select . type_identifier` before the new name - one instance, one dot - and IEEE 1800-2023 A.2.1.3 (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:800) narrows even that to an interface PORT (6.18, docs/systemverilog/2023/md/section-6-data-types.md:1839-1840: hierarchical references to type identifiers are not allowed; references through ports are not hierarchical). The raw parse stops on the file's macro use first; under the pipeline (`--preprocess`, the derived verilator context) REJECTS at that line on sv_2017 AND sv_2023, exactly at the second dot (the one-step control `typedef i.t u;` ACCEPTS on both - the letter of 2017 A.2.1.3). A verilator extension its driver keys must_accept. Spec outranks the driver key"),
    ("verilator", "test_regress/t/t_lparam_dep_iface3.v"): ("must_reject",
        "pinned SVPP-EXPANSION.6b.1: line 92 `typedef io.types.addr_t addr_t;` - a typedef reaching a type through TWO OR MORE hierarchical steps. IEEE 1800-2017 A.2.1.3 type_declaration (docs/systemverilog/2017/md/section-41-data-read-api.md:825) admits exactly ONE `interface_instance_identifier constant_bit_select . type_identifier` before the new name - one instance, one dot - and IEEE 1800-2023 A.2.1.3 (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:800) narrows even that to an interface PORT (6.18, docs/systemverilog/2023/md/section-6-data-types.md:1839-1840: hierarchical references to type identifiers are not allowed; references through ports are not hierarchical). The raw parse stops on the file's macro use first; under the pipeline (`--preprocess`, the derived verilator context) REJECTS at that line on sv_2017 AND sv_2023, exactly at the second dot (the one-step control `typedef i.t u;` ACCEPTS on both - the letter of 2017 A.2.1.3). A verilator extension its driver keys must_accept. Spec outranks the driver key"),
    ("verilator", "test_regress/t/t_lparam_dep_iface4.v"): ("must_reject",
        "pinned SVPP-EXPANSION.6b.1: line 48 `typedef io.types.sc_num_lines_t sc_num_lines_t;` - a typedef reaching a type through TWO OR MORE hierarchical steps. IEEE 1800-2017 A.2.1.3 type_declaration (docs/systemverilog/2017/md/section-41-data-read-api.md:825) admits exactly ONE `interface_instance_identifier constant_bit_select . type_identifier` before the new name - one instance, one dot - and IEEE 1800-2023 A.2.1.3 (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:800) narrows even that to an interface PORT (6.18, docs/systemverilog/2023/md/section-6-data-types.md:1839-1840: hierarchical references to type identifiers are not allowed; references through ports are not hierarchical). The raw parse stops on the file's macro use first; under the pipeline (`--preprocess`, the derived verilator context) REJECTS at that line on sv_2017 AND sv_2023, exactly at the second dot (the one-step control `typedef i.t u;` ACCEPTS on both - the letter of 2017 A.2.1.3). A verilator extension its driver keys must_accept. Spec outranks the driver key"),
    ("verilator", "test_regress/t/t_lparam_dep_iface5.v"): ("must_reject",
        "pinned SVPP-EXPANSION.6b.1: line 50 `typedef io.types.tag_t tag_t;` - a typedef reaching a type through TWO OR MORE hierarchical steps. IEEE 1800-2017 A.2.1.3 type_declaration (docs/systemverilog/2017/md/section-41-data-read-api.md:825) admits exactly ONE `interface_instance_identifier constant_bit_select . type_identifier` before the new name - one instance, one dot - and IEEE 1800-2023 A.2.1.3 (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:800) narrows even that to an interface PORT (6.18, docs/systemverilog/2023/md/section-6-data-types.md:1839-1840: hierarchical references to type identifiers are not allowed; references through ports are not hierarchical). The raw parse stops on the file's macro use first; under the pipeline (`--preprocess`, the derived verilator context) REJECTS at that line on sv_2017 AND sv_2023, exactly at the second dot (the one-step control `typedef i.t u;` ACCEPTS on both - the letter of 2017 A.2.1.3). A verilator extension its driver keys must_accept. Spec outranks the driver key"),
    ("verilator", "test_regress/t/t_lparam_dep_iface7.v"): ("must_reject",
        "pinned SVPP-EXPANSION.6b.1: line 42 `typedef io.types.a_t a_t;` - a typedef reaching a type through TWO OR MORE hierarchical steps. IEEE 1800-2017 A.2.1.3 type_declaration (docs/systemverilog/2017/md/section-41-data-read-api.md:825) admits exactly ONE `interface_instance_identifier constant_bit_select . type_identifier` before the new name - one instance, one dot - and IEEE 1800-2023 A.2.1.3 (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:800) narrows even that to an interface PORT (6.18, docs/systemverilog/2023/md/section-6-data-types.md:1839-1840: hierarchical references to type identifiers are not allowed; references through ports are not hierarchical). The raw parse stops on the file's macro use first; under the pipeline (`--preprocess`, the derived verilator context) REJECTS at that line on sv_2017 AND sv_2023, exactly at the second dot (the one-step control `typedef i.t u;` ACCEPTS on both - the letter of 2017 A.2.1.3). A verilator extension its driver keys must_accept. Spec outranks the driver key"),
    ("verilator", "test_regress/t/t_lparam_dep_iface9.v"): ("must_reject",
        "pinned SVPP-EXPANSION.6b.1: line 44 `typedef io.types.a_t a_t;` - a typedef reaching a type through TWO OR MORE hierarchical steps. IEEE 1800-2017 A.2.1.3 type_declaration (docs/systemverilog/2017/md/section-41-data-read-api.md:825) admits exactly ONE `interface_instance_identifier constant_bit_select . type_identifier` before the new name - one instance, one dot - and IEEE 1800-2023 A.2.1.3 (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:800) narrows even that to an interface PORT (6.18, docs/systemverilog/2023/md/section-6-data-types.md:1839-1840: hierarchical references to type identifiers are not allowed; references through ports are not hierarchical). The raw parse stops on the file's macro use first; under the pipeline (`--preprocess`, the derived verilator context) REJECTS at that line on sv_2017 AND sv_2023, exactly at the second dot (the one-step control `typedef i.t u;` ACCEPTS on both - the letter of 2017 A.2.1.3). A verilator extension its driver keys must_accept. Spec outranks the driver key"),
    ("verilator", "test_regress/t/t_paramgraph_comined_iface.v"): ("must_reject",
        "pinned SVPP-EXPANSION.6b.1: line 87 `typedef cca_io.tlb_io_a.r_chan_t m_r_chan_a_t;` - a typedef reaching a type through TWO OR MORE hierarchical steps. IEEE 1800-2017 A.2.1.3 type_declaration (docs/systemverilog/2017/md/section-41-data-read-api.md:825) admits exactly ONE `interface_instance_identifier constant_bit_select . type_identifier` before the new name - one instance, one dot - and IEEE 1800-2023 A.2.1.3 (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:800) narrows even that to an interface PORT (6.18, docs/systemverilog/2023/md/section-6-data-types.md:1839-1840: hierarchical references to type identifiers are not allowed; references through ports are not hierarchical). The raw parse stops on the file's macro use first; under the pipeline (`--preprocess`, the derived verilator context) REJECTS at that line on sv_2017 AND sv_2023, exactly at the second dot (the one-step control `typedef i.t u;` ACCEPTS on both - the letter of 2017 A.2.1.3). A verilator extension its driver keys must_accept. Spec outranks the driver key"),
    ("verilator", "test_regress/t/t_paramgraph_iface_array_ports.v"): ("must_reject",
        "pinned SVPP-EXPANSION.6b.1: line 44 `localparam type req_t = tgt_ports[0].req_t;` - a type named THROUGH AN INTERFACE INSTANCE, the .13y ruling extended to the rows the raw lane never reached. IEEE 1800-2017 A.2.2.1 data_type (docs/systemverilog/2017/md/section-41-data-read-api.md:842) admits a qualified type identifier only as `[ class_scope | package_scope ] type_identifier { packed_dimension }` and A.2.4 type_assignment (docs/systemverilog/2017/md/section-41-data-read-api.md:1013) takes a data_type - an instance name is neither a class scope nor a package scope; IEEE 1800-2023 6.18 says it in prose (docs/systemverilog/2023/md/section-6-data-types.md:1839: 'hierarchical references to type identifiers shall not be allowed'). The raw parse stops on the file's macro use first; under the pipeline (`--preprocess`, the derived verilator context) REJECTS at that line on sv_2017 AND sv_2023 (the minimal reproducer `localparam type u = i.t;` with `i` an interface instance: furthest_position=87 on both; the control `localparam type u = p::t;` ACCEPTS on both). A verilator extension its driver keys must_accept. Spec outranks the driver key"),
    ("verilator", "test_regress/t/t_paramgraph_iface_cfg_zero.v"): ("must_reject",
        "pinned SVPP-EXPANSION.6b.1: line 35 `localparam type cc_index_t = types.cc_index_t;` - a type named THROUGH AN INTERFACE INSTANCE, the .13y ruling extended to the rows the raw lane never reached. IEEE 1800-2017 A.2.2.1 data_type (docs/systemverilog/2017/md/section-41-data-read-api.md:842) admits a qualified type identifier only as `[ class_scope | package_scope ] type_identifier { packed_dimension }` and A.2.4 type_assignment (docs/systemverilog/2017/md/section-41-data-read-api.md:1013) takes a data_type - an instance name is neither a class scope nor a package scope; IEEE 1800-2023 6.18 says it in prose (docs/systemverilog/2023/md/section-6-data-types.md:1839: 'hierarchical references to type identifiers shall not be allowed'). The raw parse stops on the file's macro use first; under the pipeline (`--preprocess`, the derived verilator context) REJECTS at that line on sv_2017 AND sv_2023 (the minimal reproducer `localparam type u = i.t;` with `i` an interface instance: furthest_position=87 on both; the control `localparam type u = p::t;` ACCEPTS on both). A verilator extension its driver keys must_accept. Spec outranks the driver key"),
    ("verilator", "test_regress/t/t_paramgraph_minimal_sibling.v"): ("must_reject",
        "pinned SVPP-EXPANSION.6b.1: line 46 `typedef wif.a_inst.data_t local_a_t;` - a typedef reaching a type through TWO OR MORE hierarchical steps. IEEE 1800-2017 A.2.1.3 type_declaration (docs/systemverilog/2017/md/section-41-data-read-api.md:825) admits exactly ONE `interface_instance_identifier constant_bit_select . type_identifier` before the new name - one instance, one dot - and IEEE 1800-2023 A.2.1.3 (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:800) narrows even that to an interface PORT (6.18, docs/systemverilog/2023/md/section-6-data-types.md:1839-1840: hierarchical references to type identifiers are not allowed; references through ports are not hierarchical). The raw parse stops on the file's macro use first; under the pipeline (`--preprocess`, the derived verilator context) REJECTS at that line on sv_2017 AND sv_2023, exactly at the second dot (the one-step control `typedef i.t u;` ACCEPTS on both - the letter of 2017 A.2.1.3). A verilator extension its driver keys must_accept. Spec outranks the driver key"),
    ("verilator", "test_regress/t/t_paramgraph_nested_iface_typedef.v"): ("must_reject",
        "pinned SVPP-EXPANSION.6b.1: line 84 `typedef cca_io.tlb_io.r_chan_t m_r_chan_t;` - a typedef reaching a type through TWO OR MORE hierarchical steps. IEEE 1800-2017 A.2.1.3 type_declaration (docs/systemverilog/2017/md/section-41-data-read-api.md:825) admits exactly ONE `interface_instance_identifier constant_bit_select . type_identifier` before the new name - one instance, one dot - and IEEE 1800-2023 A.2.1.3 (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:800) narrows even that to an interface PORT (6.18, docs/systemverilog/2023/md/section-6-data-types.md:1839-1840: hierarchical references to type identifiers are not allowed; references through ports are not hierarchical). The raw parse stops on the file's macro use first; under the pipeline (`--preprocess`, the derived verilator context) REJECTS at that line on sv_2017 AND sv_2023, exactly at the second dot (the one-step control `typedef i.t u;` ACCEPTS on both - the letter of 2017 A.2.1.3). A verilator extension its driver keys must_accept. Spec outranks the driver key"),
    ("verilator", "test_regress/t/t_typedef_iface_typedef3.v"): ("must_reject",
        "pinned SVPP-EXPANSION.6b.1: line 42 `typedef y_if0.x_if_a[0].rq_t rq_t;` - a typedef reaching a type through TWO OR MORE hierarchical steps. IEEE 1800-2017 A.2.1.3 type_declaration (docs/systemverilog/2017/md/section-41-data-read-api.md:825) admits exactly ONE `interface_instance_identifier constant_bit_select . type_identifier` before the new name - one instance, one dot - and IEEE 1800-2023 A.2.1.3 (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:800) narrows even that to an interface PORT (6.18, docs/systemverilog/2023/md/section-6-data-types.md:1839-1840: hierarchical references to type identifiers are not allowed; references through ports are not hierarchical). The raw parse stops on the file's macro use first; under the pipeline (`--preprocess`, the derived verilator context) REJECTS at that line on sv_2017 AND sv_2023, exactly at the second dot (the one-step control `typedef i.t u;` ACCEPTS on both - the letter of 2017 A.2.1.3). A verilator extension its driver keys must_accept. Spec outranks the driver key"),
    ("verilator", "test_regress/t/t_typedef_iface_typedef4.v"): ("must_reject",
        "pinned SVPP-EXPANSION.6b.1: line 38 `typedef z_if0.x_if0.rq_t rq_t;` - a typedef reaching a type through TWO OR MORE hierarchical steps. IEEE 1800-2017 A.2.1.3 type_declaration (docs/systemverilog/2017/md/section-41-data-read-api.md:825) admits exactly ONE `interface_instance_identifier constant_bit_select . type_identifier` before the new name - one instance, one dot - and IEEE 1800-2023 A.2.1.3 (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:800) narrows even that to an interface PORT (6.18, docs/systemverilog/2023/md/section-6-data-types.md:1839-1840: hierarchical references to type identifiers are not allowed; references through ports are not hierarchical). The raw parse stops on the file's macro use first; under the pipeline (`--preprocess`, the derived verilator context) REJECTS at that line on sv_2017 AND sv_2023, exactly at the second dot (the one-step control `typedef i.t u;` ACCEPTS on both - the letter of 2017 A.2.1.3). A verilator extension its driver keys must_accept. Spec outranks the driver key"),
    ("verilator", "test_regress/t/t_typedef_iface_typedef5.v"): ("must_reject",
        "pinned SVPP-EXPANSION.6b.1: line 41 `typedef if0.x_if0.rq_t rq_t;` - a typedef reaching a type through TWO OR MORE hierarchical steps. IEEE 1800-2017 A.2.1.3 type_declaration (docs/systemverilog/2017/md/section-41-data-read-api.md:825) admits exactly ONE `interface_instance_identifier constant_bit_select . type_identifier` before the new name - one instance, one dot - and IEEE 1800-2023 A.2.1.3 (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:800) narrows even that to an interface PORT (6.18, docs/systemverilog/2023/md/section-6-data-types.md:1839-1840: hierarchical references to type identifiers are not allowed; references through ports are not hierarchical). The raw parse stops on the file's macro use first; under the pipeline (`--preprocess`, the derived verilator context) REJECTS at that line on sv_2017 AND sv_2023, exactly at the second dot (the one-step control `typedef i.t u;` ACCEPTS on both - the letter of 2017 A.2.1.3). A verilator extension its driver keys must_accept. Spec outranks the driver key"),
    ("verilator", "test_regress/t/t_typedef_iface_typedef6.v"): ("must_reject",
        "pinned SVPP-EXPANSION.6b.1: line 41 `typedef if0[0].x_if0.rq_t rq_t;` - a typedef reaching a type through TWO OR MORE hierarchical steps. IEEE 1800-2017 A.2.1.3 type_declaration (docs/systemverilog/2017/md/section-41-data-read-api.md:825) admits exactly ONE `interface_instance_identifier constant_bit_select . type_identifier` before the new name - one instance, one dot - and IEEE 1800-2023 A.2.1.3 (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:800) narrows even that to an interface PORT (6.18, docs/systemverilog/2023/md/section-6-data-types.md:1839-1840: hierarchical references to type identifiers are not allowed; references through ports are not hierarchical). The raw parse stops on the file's macro use first; under the pipeline (`--preprocess`, the derived verilator context) REJECTS at that line on sv_2017 AND sv_2023, exactly at the second dot (the one-step control `typedef i.t u;` ACCEPTS on both - the letter of 2017 A.2.1.3). A verilator extension its driver keys must_accept. Spec outranks the driver key"),
    # ---- SVPP-EXPANSION.6c.2 (PGEN-SVPP-EXPANSION-0016) --------------------------------------------
    # The 28 `key_question` rows of the .6b disposition table: rows the raw lane explained away (a macro
    # use, a conditional or an `include stood between the parse and the construct) and the pipeline
    # reached — each REJECTS in its own text once preprocessed, on sv_2017 AND sv_2023, on text no
    # edition derives (or, twice, text 1800-2023 derives and 1800-2017 does not: an edition contrast, the
    # .13x.d shape). Each pin below carries the clause and the measurement — a minimal reproducer that
    # rejects and a control that accepts on both profiles, or the earlier ruling whose reproducer stands.
    # Same taxonomy as the blocks above: text a vendor tolerates that the standard cannot derive, where
    # the spec outranks the driver key. The raw lane cannot cash these pins (its parse stops before the
    # construct) and demotes them to `deferred:svpp_owned`; the pipeline re-adjudication reads the pin
    # from this table and records `must_reject:match_under_expansion`.
    ("Surelog", "tests/PoundParam/dut.v"): ("must_reject",
        "pinned SVPP-EXPANSION.6c.2: line 39 `dffr #4 park_reg(.din (next_pv[3:0]), ...)` - a parameter value assignment WITHOUT parentheses: the SV-CORPUS-GRAD.3.51 ruling (IEEE 1800-2017 A.4.1.1 parameter_value_assignment ::= # ( [ list_of_parameter_assignments ] )); Surelog's golden log tolerates it. MEASURED under the pipeline: the corpus row REJECTS at line 39 on sv_2017 AND sv_2023 (the raw parse stops on the file's macro use first). The .3.51 ruling's reproducer and control stand for the construct."),
    ("slang", "tests/regression/driver/dep-files.sv"): ("must_reject",
        "pinned SVPP-EXPANSION.6c.2: origin line 35 `string s = `FOO;` expands to a legal string literal (`FOO -> `BAR -> `__FILE__, data/file_defn.svh:1-2); the parse actually stops on the NEXT item, a bare `begin end` at module-item level - not derivable (IEEE 1800-2017 A.1.4 module_item; the SV-CORPUS-GRAD.3.56 ruling: bare begin/end blocks at module level are must_reject). A slang regression-driver fixture; the suite rule keyed it must_accept. MEASURED under the pipeline: REJECTS on sv_2017 AND sv_2023 at the pinned line (the raw parse stops on the file's `include first). The .3.56 ruling's own reproducer and control stand for the construct."),
    ("slang", "tests/regression/driver/disable-local-includes.sv"): ("must_reject",
        "pinned SVPP-EXPANSION.6c.2: the same body as dep-files.sv - a bare `begin end` at module-item level after `string s = `FOO;` (A.1.4; the .3.56 ruling); a slang driver fixture run with `|| true`. MEASURED under the pipeline: REJECTS on sv_2017 AND sv_2023 at the pinned line (the raw parse stops on the file's `include first). The .3.56 ruling's own reproducer and control stand for the construct."),
    ("slang", "tests/regression/driver/module-basics.sv"): ("must_reject",
        "pinned SVPP-EXPANSION.6c.2: the same body as dep-files.sv - a bare `begin end` at module-item level (A.1.4; the .3.56 ruling). MEASURED under the pipeline: REJECTS on sv_2017 AND sv_2023 at the pinned line (the raw parse stops on the file's `include first). The .3.56 ruling's own reproducer and control stand for the construct."),
    ("slang", "tests/regression/driver/nonstandard-constraint-block.sv"): ("must_reject",
        "pinned SVPP-EXPANSION.6c.2: line 21 ``CREATE_VSEQ(my_vseq, {` - the file's own RUN line is `%slang %s 2>&1 || true` then `--compat=all`: a fixture for a NON-STANDARD constraint-block form slang accepts only in compat mode; the standard cannot derive it and the suite rule keyed it must_accept. MEASURED: the corpus row rejects at line 21 on sv_2017 AND sv_2023 under the pipeline; the file's own RUN line runs slang with `|| true` and then `--compat=all`, naming the form non-standard."),
    ("slang", "tests/regression/driver/print-macros.sv"): ("must_reject",
        "pinned SVPP-EXPANSION.6c.2: the same body as dep-files.sv - a bare `begin end` at module-item level (A.1.4; the .3.56 ruling); a `-E --preprocess-source` fixture. MEASURED under the pipeline: REJECTS on sv_2017 AND sv_2023 at the pinned line (the raw parse stops on the file's `include first). The .3.56 ruling's own reproducer and control stand for the construct."),
    ("slang", "tests/unittests/data/test.sv"): ("must_reject",
        "pinned SVPP-EXPANSION.6c.2: the same body - a bare `begin end` at module-item level after `string s = `FOO;` (A.1.4; the .3.56 ruling); a FileTests.cpp unit-test data file. MEASURED under the pipeline: REJECTS on sv_2017 AND sv_2023 at the pinned line (the raw parse stops on the file's `include first). The .3.56 ruling's own reproducer and control stand for the construct."),
    ("slang", "tests/unittests/data/verilog.v"): ("must_reject",
        "pinned SVPP-EXPANSION.6c.2: line 9 `wire do;` - `do` is a reserved keyword in every SystemVerilog edition (IEEE 1800-2017 Table B.1); the file is Verilog by extension and by intent and belongs to the verilog_2005 lane's judgement, not the sv_2017 lane's must_accept. MEASURED: `module m; wire do; endmodule` REJECTS at furthest_position=14 on sv_2017 AND sv_2023; the control `wire done;` ACCEPTS on both; the corpus row rejects at line 9 on both profiles under the pipeline."),
    ("sv2v", "test/core/constexpr.sv"): ("must_reject",
        "pinned SVPP-EXPANSION.6c.2: line 48 ``ASSERT_TRUE(5'{4'hF, 3'{1'b1, 1'b1}} == 27)` - a SIZED assignment-pattern-like `5'{...}`; IEEE 1800-2017 A.8.1 assignment_pattern is `'{` only and A.8.1 multiple_concatenation is `{ expression concatenation }` - `N'{...}` is not derivable in 2017 (sv2v's extension). The suite contract keyed it must_accept. MEASURED: `v = 5'{4'hF, 1'b1};` REJECTS at furthest_position=39 on sv_2017 AND sv_2023; the control `v = {4'hF, 1'b1};` ACCEPTS on both; the corpus row rejects at line 48 on both profiles under the pipeline."),
    ("sv2v", "test/core/no_default_param.sv"): ("must_reject",
        'pinned SVPP-EXPANSION.6c.2: line 9 ``DUMP(InterfaceA)` with `define DUMP(name) initial $display(`"name W=%b ... %0d", ...)` - a `" macro string CLOSED by a plain `"` (IEEE 1800-2017 §22.5.3.2: the enclosed text is delimited by `" on both sides); sv2v tolerates the mixed form. The suite contract keyed it must_accept. MEASURED: a `" macro string closed by a plain `"` REJECTS at furthest_position=32 on sv_2017 AND sv_2023 (the reproducer `define DUMP(name) initial $display(`"name W=%b", 1\'b1);); the control closed by `" ACCEPTS on both; the corpus row rejects at its line on both profiles under the pipeline.'),
    ("sv2v", "test/lex/macro.sv"): ("must_reject",
        'pinned SVPP-EXPANSION.6c.2: line 142 ``MACRO_E(display);` with `define MACRO_E(s) $display("s", `"s", `"ss"); - two `" macro strings each CLOSED by a plain `"` (IEEE 1800-2017 §22.5.3.2: the enclosed text is delimited by `" on both sides, so the standard reads ONE macro string from the first `" to the second); sv2v\'s lexer tolerates the plain close. The `$display(display)` item of line 137 is fixed (.6b.5); the row\'s next barrier is this form, the no_default_param.sv ruling. The suite contract keyed it must_accept. MEASURED: a `" macro string closed by a plain `"` REJECTS at furthest_position=32 on sv_2017 AND sv_2023 (the reproducer `define DUMP(name) initial $display(`"name W=%b", 1\'b1);); the control closed by `" ACCEPTS on both; the corpus row rejects at its line on both profiles under the pipeline.'),
    ("sv2v", "test/lex/string_macro.sv"): ("must_reject",
        'pinned SVPP-EXPANSION.6c.2: line 9 `$display(`"`FOO`");` - the `" macro-string form OUTSIDE macro text; IEEE 1800-2017 §22.5.3.2 defines `" and `\\`" within the text of a macro only; sv2v\'s lexer tolerates it in source. The suite contract keyed it must_accept. MEASURED: `module m; initial $display(`"x`"); endmodule` REJECTS at furthest_position=27 on sv_2017 AND sv_2023; the control (the same `"x`" inside a macro\'s text) ACCEPTS on both; the corpus row rejects at line 9 on both profiles under the pipeline.'),
    ("verible", "verible/verilog/tools/lint/testdata/uvm_macro_semicolon.sv"): ("must_reject",
        'pinned SVPP-EXPANSION.6c.2: line 3 `function f();` followed by ``uvm_info("msg_id", "message", UVM_LOW);` - the file includes nothing and defines nothing: `uvm_info is undefined in the unit (verible lints the raw text without preprocessing); as a compilation unit the text is not derivable. The suite rule keyed a lint fixture must_accept. MEASURED: `class foo; task bar; `uvm_warning("x") endtask endclass` (no include, no define) REJECTS at furthest_position=20 on sv_2017 AND sv_2023; the control with a `$display` in its place ACCEPTS on both; the corpus row rejects at its line on both profiles under the pipeline. A `identifier naming no defined macro has no text to substitute (IEEE 1800-2017 22.5.1) and is no token of Annex A.'),
    ("verible", "verible/verilog/tools/lint/testdata/uvm_warning.sv"): ("must_reject",
        'pinned SVPP-EXPANSION.6c.2: line 2 `task bar;` followed by ``uvm_warning(...)` - the same: a UVM macro used with no definition in the unit (a lint fixture verible never preprocesses). MEASURED: `class foo; task bar; `uvm_warning("x") endtask endclass` (no include, no define) REJECTS at furthest_position=20 on sv_2017 AND sv_2023; the control with a `$display` in its place ACCEPTS on both; the corpus row rejects at its line on both profiles under the pipeline. A `identifier naming no defined macro has no text to substitute (IEEE 1800-2017 22.5.1) and is no token of Annex A.'),
    ("verilator", "test_regress/t/t_assert_opt_check.v"): ("must_reject",
        'pinned SVPP-EXPANSION.6c.2: line 65 `assert(cntB - 100 == cntA); else assert(cntB == cntA + 100);` - a `;` between the assertion and its `else`: IEEE 1800-2017 A.6.10 action_block ::= statement_or_null | [ statement ] else statement_or_null - the `;` IS the statement_or_null, and the `else` that follows has no derivation. The driver key tolerates it. MEASURED: `assert(a == b); else $display("x");` REJECTS at furthest_position=49 on sv_2017 AND sv_2023; the control without the `;` ACCEPTS on both; the corpus row rejects at line 65 on both profiles under the pipeline.'),
    ("verilator", "test_regress/t/t_bitsel_concat.v"): ("must_reject",
        "pinned SVPP-EXPANSION.6c.2: line 62 `s1 = valf0ed()[4];` - a bit-select applied to a FUNCTION CALL's result; IEEE 1800-2017 A.8.4 primary: a select attaches to a hierarchical_identifier (or a concatenation / streaming), not to a subroutine_call (A.8.2 tf_call carries no select). The driver key tolerates it. MEASURED: EDITION CONTRAST, the .13x.d shape - `s = f()[4];` REJECTS at furthest_position=86 on sv_2017 and ACCEPTS on sv_2023 (IEEE 1800-2023 A.8.4 primary ::= ... | function_subroutine_call [ [ range_expression ] ] admits ONE select, docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:3087; 1800-2017 A.8.4 has `function_subroutine_call` alone); the control through a variable ACCEPTS on both; the corpus row rejects at line 62 on sv_2017 and, on sv_2023, at line 97 `valarr()[1][2]` (TWO selects - beyond 2023's one). On this sv_2017 lane the row is must_reject."),
    ("verilator", "test_regress/t/t_clk_concat2.v"): ("must_reject",
        "pinned SVPP-EXPANSION.6c.2: line 76 `);` closes a MIXED port list (line 71 `clk /*verilator clocker*/,` omits direction, kind and type; typed `input clk0 ...` items follow): the SV-CORPUS-GRAD.13y ruling (IEEE 1800-2017 23.2.2.3, section-23-modules-and-hierarchy.md:472-474 - the list is non-ANSI and the typed items have no derivation). The driver key tolerates it. MEASURED under the pipeline: the corpus row REJECTS at line 76 on sv_2017 AND sv_2023. The .13y ruling's reading stands for the construct."),
    ("verilator", "test_regress/t/t_covergroup_auto_bins.v"): ("must_reject",
        'pinned SVPP-EXPANSION.6c.2: line 22 `bins auto[4];` - a bins declaration with NO `= { ... }` / `= default` right-hand side; IEEE 1800-2017 A.2.11 bins_or_options ::= ... [ wildcard ] bins_keyword bin_identifier [ [ [ covergroup_expression ] ] ] = { covergroup_range_list } [ with ... ] [ iff ... ] | ... = default ... - the right-hand side is required. The driver key tolerates it. MEASURED: `bins auto[4];` REJECTS at furthest_position=77 on sv_2017 AND sv_2023; the control `bins auto[4] = {[0:255]};` ACCEPTS on both; the corpus row rejects at its line on both profiles under the pipeline.'),
    ("verilator", "test_regress/t/t_covergroup_default_bins.v"): ("must_reject",
        'pinned SVPP-EXPANSION.6c.2: line 71 `cp_data64: coverpoint data64 { bins auto[4]; }` - a bins declaration with no right-hand side (A.2.11, as t_covergroup_auto_bins.v). The driver key tolerates it. MEASURED: `bins auto[4];` REJECTS at furthest_position=77 on sv_2017 AND sv_2023; the control `bins auto[4] = {[0:255]};` ACCEPTS on both; the corpus row rejects at its line on both profiles under the pipeline.'),
    ("verilator", "test_regress/t/t_default_disable_iff_scope.v"): ("must_reject",
        "pinned SVPP-EXPANSION.6c.2: line 31 `generate` followed by `begin` - a bare begin/end block directly inside a generate region (the .3.56 ruling, A.4.2 / A.1.4). MEASURED under the pipeline: the corpus row REJECTS at the `generate` line on sv_2017 AND sv_2023 (the raw parse stops on a macro use first). The .3.56 ruling's reproducer and control stand for the construct."),
    ("verilator", "test_regress/t/t_gen_genblk.v"): ("must_reject",
        "pinned SVPP-EXPANSION.6c.2: line 20 `generate` followed by `begin : direct_ignored` - a bare (labelled) begin/end block directly inside a generate region: the SV-CORPUS-GRAD.3.56 ruling (IEEE 1800-2017 A.4.2 generate_region / A.1.4: a begin/end is a generate_block only under a conditional or loop construct); the driver's expect_filename is a lint golden, not an error. MEASURED under the pipeline: the corpus row REJECTS at the `generate` line on sv_2017 AND sv_2023 (the raw parse stops on a macro use first). The .3.56 ruling's reproducer and control stand for the construct."),
    ("verilator", "test_regress/t/t_hier_block.v"): ("must_reject",
        "pinned SVPP-EXPANSION.6c.2: line 81 `delay #(.N(2), 8) i_delay0(clk, out3, out5);` - NAMED and ORDERED parameter assignments mixed in one list; IEEE 1800-2017 A.4.1.1 list_of_parameter_assignments ::= ordered_parameter_assignment { , ... } | named_parameter_assignment { , ... } - one kind per list. The driver key tolerates it. MEASURED: `d #(.N(2), 8) i0(1'b0);` REJECTS at furthest_position=81 on sv_2017 AND sv_2023; the control `d #(.N(2), .M(8)) i0(1'b0);` ACCEPTS on both; the corpus row rejects at line 81 on both profiles under the pipeline."),
    ("verilator", "test_regress/t/t_inside_tolerance_unsup.v"): ("must_reject",
        "pinned SVPP-EXPANSION.6c.2: line 18 ``checkh(r inside {[1492 +/- 2]}, 1'b1);` - the `+/-` tolerance range of an inside expression is IEEE 1800-2023 (11.4.13 value ranges with +/- and +%-); 1800-2017 A.8.3 value_range ::= expression | [ expression : expression ] has no such form - an edition contrast, the .13x.d shape (must_reject on sv_2017; a 2023 lane would judge it). The driver's fails=True names Verilator's UNSUPPORTED. MEASURED: EDITION CONTRAST, the .13x.d shape - the corpus row REJECTS at line 18 (`[1492 +/- 2]`) on sv_2017 and ACCEPTS that line on sv_2023 (IEEE 1800-2023 11.4.13 admits `+/-`), where it then rejects at line 20 on `[1490 +%- 10]` - the `+%-` form 1800-2023 also admits: a gap of the sv_2023 profile's grammar, routed to SV-CORPUS-GRAD's 2023 lane (.14). On this sv_2017 lane the row is must_reject. ⭐ SV-CORPUS-GRAD.3.59 (release 1.0.229): the sv_2023 profile now derives `+%-` too (11.4.13, A.8.3 value_range), so under sv_2023 the row ACCEPTS entirely; the sv_2017 ruling stands (an edition contrast either way)"),
    ("verilator", "test_regress/t/t_mod_uselib.v"): ("must_reject",
        'pinned SVPP-EXPANSION.6c.2: line 9 ``uselib libext=.v` - `uselib is not a compiler directive of IEEE 1800-2017 clause 22 (nor of 1364-2005 clause 19; it is a Verilog-XL library directive); the .3.39 rule refuses a directive no edition lists. The driver key tolerates it. MEASURED: `module m; `uselib libext=.v endmodule` REJECTS at furthest_position=9 on sv_2017 AND sv_2023 - exactly as `resetall or any unknown directive inside a module does (no directive but the .3.14b whitelist is a module_item, A.1.4); at UNIT level an unknown directive passes as opaque text (a documented tolerance, not a ruling on the name); the corpus row rejects at line 9 on both profiles under the pipeline.'),
    ("verilator", "test_regress/t/t_pgo_profoutofdate_bad.v"): ("must_reject",
        'pinned SVPP-EXPANSION.6c.2: line 27 `profile_data -model "x" -mtask "h7baded98__0" -cost 64\'d1234` - Verilator profile-guided-optimisation configuration text inside the .v (the driver\'s fails=True and PROFOUTOFDATE golden name it); not SystemVerilog source under any edition. MEASURED under the pipeline: the corpus row REJECTS at line 27 on sv_2017 AND sv_2023 - `profile_data -model "x" -mtask ... -cost 64\'d1234` is Verilator\'s profile-data configuration language, not SystemVerilog text under any edition (its driver\'s fails=True and PROFOUTOFDATE golden name it).'),
    ("verilator", "test_regress/t/t_randomize_array.v"): ("must_reject",
        'pinned SVPP-EXPANSION.6c.2: line 51 `foreach (unpacked_array[i][j]) begin` - loop variables in SEPARATE bracket pairs: the SV-CORPUS-GRAD.13y ruling on Surelog ForeachSquare (IEEE 1800-2017 A.6.8 loop_variables ::= [ index_variable_identifier ] { , [ index_variable_identifier ] } - one bracket pair, comma-separated). The driver key tolerates it. MEASURED under the pipeline: the corpus row REJECTS at line 51 on sv_2017 AND sv_2023. The .13y ForeachSquare ruling stands for the construct.'),
    ("verilator", "test_regress/t/t_struct_init.v"): ("must_reject",
        "pinned SVPP-EXPANSION.6c.2: line 90 `q = '{3{1'b1}, 1'b0};` - a REPLICATION inside an assignment pattern's expression list; IEEE 1800-2017 A.8.1 assignment_pattern ::= '{ expression { , expression } } | '{ ... : ... } | '{ constant_expression { expression { , expression } } } - `3{1'b1}` is a multiple_concatenation only inside braces of its own, never an expression. The driver key tolerates it. MEASURED: `q = '{3{1'b1}, 1'b0};` REJECTS at furthest_position=65 on sv_2017 AND sv_2023; the control `q = {{3{1'b1}}, 1'b0};` ACCEPTS on both; the corpus row rejects at line 90 on both profiles under the pipeline."),
    ("verilator", "test_regress/t/t_var_dotted2.v"): ("must_reject",
        "pinned SVPP-EXPANSION.6c.2: line 94 `generate` followed by `begin` - a bare begin/end block directly inside a generate region (the .3.56 ruling, A.4.2 / A.1.4); measured: fails identically with the driver's `+define+NOUSE_INLINE`. MEASURED under the pipeline: the corpus row REJECTS at the `generate` line on sv_2017 AND sv_2023 (the raw parse stops on a macro use first). The .3.56 ruling's reproducer and control stand for the construct."),
    # ---- SVPP-EXPANSION.4c.1 (PGEN-SVPP-EXPANSION-0017) --------------------------------------------
    # The lane now runs every file under the defines its OWN driver passes for it (verilator `t_x.py`
    # flags, sv-tests `:defines:`; `stimuli/sv/pipeline_context_defines.tsv`, derived). Three rows the
    # driver's define reaches REJECT under the pipeline on sv_2017 AND sv_2023, on text no edition
    # derives — the same taxonomy as the .6c.2 block (text a vendor tolerates that the standard cannot
    # derive; the spec outranks the driver key), each with a reproducer/control measured on both
    # profiles (rust/target/svpp_e1/s4c1/*.sv). The raw lane stops before the construct (the
    # driver's conditional) and demotes them to `deferred:svpp_owned`; the pipeline cashes the pin.
    ("verilator", "test_regress/t/t_gen_missing.v"): ("must_reject",
        "pinned SVPP-EXPANSION.4c.1: under the driver's own `+define+T_GEN_MISSING` (t_gen_missing.py:14) the parse reaches module foobar and stops at line 30 `generate` followed by `begin: g` - a bare begin/end block directly inside a generate region: the SV-CORPUS-GRAD.3.56 ruling (IEEE 1800-2017 A.4.2 generate_region ::= generate { generate_item } endgenerate; a generate_block is reached only through a conditional or loop generate construct). The driver key tolerates it. MEASURED: rust/target/svpp_e1/s4c1/gen_bare_begin_repro.sv (the file's generate region, wrapper kept) fails at byte 144 (:3:11, after `generate`) on sv_2017 AND sv_2023; gen_bare_begin_control.sv (the wrapper removed, the `;` after `begin: foo_inst` kept - a legal empty package_or_generate_item_declaration) accepts on both. Under the pipeline the corpus row rejects at [origin=t_gen_missing.v:30:11] on both profiles; without the define the raw lane and the pipeline stop at line 14 on the file's `error stub (the .6b.3 driver_define disposition, now superseded by the derived define)."),
    ("verilator", "test_regress/t/t_vpi_module.v"): ("must_reject",
        "pinned SVPP-EXPANSION.4c.1: under the driver's own `+define+USE_DOLLAR_C32` (t_vpi_module.py:19) the selected branch is lines 22-25 `systemc_header / extern \"C\" int mon_check(); / `verilog - Verilator's extension directives (neither is among the compiler directives IEEE 1800-2017 clause 22 lists; 1800-2023 clause 22 likewise) enclosing C++ text no production of Annex A derives; the driver key (Verilator, --exe --vpi) tolerates it. MEASURED: rust/target/svpp_e1/s4c1/vendor_directive_repro.sv (`module t; `systemc_header / extern \"C\" int mon_check(); / `verilog / endmodule`) fails at byte 9 (:1:10, the directive inside the module) on sv_2017 AND sv_2023; vendor_directive_control.sv (the three lines removed) accepts on both; at unit level the directive line passes as opaque text and the C++ line is refused (:2:7). Under the pipeline the corpus row rejects at [origin=t_vpi_module.v:20:5] (the module header's end, the item after it underivable) on both profiles; without the define the pipeline accepted the file (the branch skipped) and the raw lane stops in the branch's C++ text (explained_svpp_macro_use)."),
    ("verilator", "test_regress/t/t_vpi_public_depth.v"): ("must_reject",
        "pinned SVPP-EXPANSION.4c.1: under the driver's own `+define+USE_DOLLAR_C32` (t_vpi_public_depth.py:19) the selected branch is lines 22-25 `systemc_header / extern \"C\" int mon_check(); / `verilog - Verilator's extension directives (not among IEEE 1800-2017 clause 22's compiler directives) enclosing C++ text no production of Annex A derives; the driver key tolerates it. MEASURED: the same reproducer/control pair as t_vpi_module.v (rust/target/svpp_e1/s4c1/vendor_directive_{repro,control}.sv: fail@9 / accept on sv_2017 AND sv_2023). Under the pipeline the corpus row rejects at [origin=t_vpi_public_depth.v:20:5] on both profiles; without the define the pipeline accepted the file and the raw lane stops in the branch's C++ text (explained_svpp_macro_use)."),
    # ---- SV-CORPUS-GRAD.12b.1 (PGEN-SV-CORPUS-GRAD-0390) ---------------------------------------------
    # The 21 rows the pipeline re-adjudication routed here as parser questions (`SVPP-EXPANSION.6b.4`,
    # `refuted:parser_question`), read one by one with the toolbox on the EXPANDED text at the probe's furthest
    # TOKEN (not the mapped origin line): eleven are the standard's own refusals — a minimal reproducer that
    # REJECTS on sv_2017 AND sv_2023 (one an EDITION CONTRAST: 1800-2023 admits it) and a control that ACCEPTS on
    # both (rust/target/svpp_e1/s12b1/*.sv, the debug probe 1.0.227) — pinned must_reject with the clause; the raw lane cannot reach
    # them (`explained_svpp_macro_use`) and demotes them to deferred:svpp_owned; the pipeline cashes the pin.
    ("verilator", "test_regress/t/t_foreach.v"): ("must_reject",
        "pinned SV-CORPUS-GRAD.12b.1: line 152 `foreach (twod[i, j]);` - a foreach whose body is the NULL statement; IEEE 1800-2017 A.6.8 loop_statement ::= foreach ( ps_or_hierarchical_array_identifier [ loop_variables ] ) statement - `statement`, not statement_or_null (A.6.4 statement_item has no bare `;`; the for / while / repeat / forever forms take statement_or_null, foreach alone does not); 1800-2023 A.6.8 identical. The driver key tolerates it. MEASURED: rust/target/svpp_e1/s12b1/r8.sv (`foreach (twod[i, j]);`) fails at byte 60 on sv_2017 AND sv_2023; r8b.sv (`foreach (twod[i, j]) begin end`) accepts on both. Under the pipeline the corpus row rejects at [origin=t_foreach.v:152:25] on both profiles."),
    ("verilator", "test_regress/t/t_foreach_const.v"): ("must_reject",
        "pinned SV-CORPUS-GRAD.12b.1: line 170 `foreach (twod[i, j]);` - the same null-body foreach as t_foreach.v (IEEE 1800-2017 A.6.8 foreach (...) statement; A.6.4). MEASURED: the same reproducer/control pair (rust/target/svpp_e1/s12b1/r8.sv fail@60 / r8b.sv accept, sv_2017 AND sv_2023); the corpus row rejects at [origin=t_foreach_const.v:170:25] on both profiles under the pipeline."),
    ("verilator", "test_regress/t/t_event_array_fire.v"): ("must_reject",
        "pinned SV-CORPUS-GRAD.12b.1: line 39 `->e_all_xfers_completed[id];` - an event trigger on an ELEMENT of an event array: an EDITION CONTRAST. IEEE 1800-2017 A.6.5 event_trigger ::= -> hierarchical_event_identifier ; with A.9.3 hierarchical_identifier ::= [ $root . ] { identifier constant_bit_select . } identifier - the FINAL identifier carries no select, so `[id]` is underivable in 2017; IEEE 1800-2023 A.6.5 adds it: event_trigger ::= -> hierarchical_event_identifier nonrange_select ; (section-Annex_A-normative-formal-syntax.md:2192). The raw lane and the pipeline are sv_2017. MEASURED: rust/target/svpp_e1/s12b1/r7.sv (`-> ev[i];`) fails at byte 52 on sv_2017 and ACCEPTS on sv_2023; r7b.sv (`-> ev;`) accepts on both. Under the pipeline the corpus row rejects at [origin=t_event_array_fire.v:39] on sv_2017 and ACCEPTS on sv_2023 - the 2023 lane's inherited ruling is overridden by the contrast, as .13x.d's."),
    ("verilator", "test_regress/t/t_fork_join_none_wait_ev.v"): ("must_reject",
        "pinned SV-CORPUS-GRAD.12b.1: line 32 `->ev[i];` - an event trigger on an event-array element, as t_event_array_fire.v: an EDITION CONTRAST (IEEE 1800-2017 A.6.5 / A.9.3 derive no select on the final identifier; 1800-2023 A.6.5 event_trigger ::= -> hierarchical_event_identifier nonrange_select ;). MEASURED: rust/target/svpp_e1/s12b1/r7.sv fail@52 on sv_2017 / accept on sv_2023, r7b.sv accept on both; the corpus row rejects on sv_2017 and accepts on sv_2023 under the pipeline."),
    ("verilator", "test_regress/t/t_property_sexpr_unsup.v"): ("must_reject",
        "pinned SV-CORPUS-GRAD.12b.1: line 45 `assert property (@(posedge clk) ##1 (val until val))` - a PROPERTY operator inside the parenthesised term after a cycle delay; IEEE 1800-2017 A.2.10 sequence_expr ::= cycle_delay_range sequence_expr { ... } | ... | ( sequence_expr { , sequence_match_item } ) [ sequence_abbrev ] - what follows `##1` is a sequence_expr, and `val until val` is a property_expr (until is a property operator, A.2.10 property_expr ::= ... | property_expr until property_expr), never a sequence_expr; 1800-2023 A.2.10 identical; Verilator's own driver names it UNSUPPORTED. MEASURED: rust/target/svpp_e1/s12b1/p1.sv (`##1 (val until val)`) fails at byte 66 on sv_2017 AND sv_2023; p1c.sv (`##1 (val)`) accepts on both. Under the pipeline the corpus row rejects at [origin=t_property_sexpr_unsup.v:45:43] on both profiles."),
    ("verilator", "test_regress/t/t_randomize_solve_before_foreach.v"): ("must_reject",
        "pinned SV-CORPUS-GRAD.12b.1: line 40-41 `foreach (data[i]) { solve mode before data[i]; ... }` - a `solve ... before` INSIDE a foreach constraint_set; IEEE 1800-2017 A.1.10 constraint_block_item ::= solve solve_before_list before solve_before_list ; | constraint_expression - `solve` is a constraint BLOCK item only, while foreach ( ... ) constraint_set with constraint_set ::= constraint_expression | { { constraint_expression } } admits constraint_expressions alone (18.5.10 / 18.5.8); 1800-2023 A.1.10 identical. The .6b.4 reading had bucketed the foreach (legal); the probe's furthest TOKEN is the `solve` on the next line. MEASURED: rust/target/svpp_e1/s12b1/p2.sv (solve inside the foreach set) fails at byte 76 on sv_2017 AND sv_2023; p2c.sv (solve at block level, the foreach beside it) accepts on both. Under the pipeline the corpus row rejects at [origin=t_randomize_solve_before_foreach.v:40:26] on both profiles."),
    ("verilator", "test_regress/t/t_std_randomize.v"): ("must_reject",
        "pinned SV-CORPUS-GRAD.12b.1: line 110 `std::randomize(limit_63bits[i]) with { ... }` - an INDEXED variable as a randomize argument; IEEE 1800-2017 A.8.2 randomize_call ::= randomize { attribute_instance } [ ( [ variable_identifier_list | null ] ) ] [ with ... constraint_block ] with A.9.3 variable_identifier_list ::= variable_identifier { , variable_identifier } - identifiers, no select (18.12 scope randomize: `std::randomize( variable_identifier_list )`); 1800-2023 A.8.2 identical. The .6b.4 reading had bucketed the `with` clause (legal); the furthest TOKEN is the `with` after the indexed argument. MEASURED: rust/target/svpp_e1/s12b1/p3.sv (`std::randomize(a[i]) with { ... }`) fails at byte 65 on sv_2017 AND sv_2023; p3c.sv (`std::randomize(a) with { ... }`) accepts on both. Under the pipeline the corpus row rejects at [origin=t_std_randomize.v:110:46] on both profiles."),
    ("verilator", "test_regress/t/t_prop_always.v"): ("must_reject",
        "pinned SV-CORPUS-GRAD.12b.1: line 50-51 `assert property (@(posedge clk) always [0:3] a_low)` newline `;` newline `else ...` - a `;` before an assertion's else: the SVPP-EXPANSION.6c.2 ruling (IEEE 1800-2017 A.6.10 concurrent_assertion_statement ::= assert property ( property_spec ) action_block, A.6.3 action_block ::= statement_or_null | [ statement ] else statement_or_null - the item before `else` is a statement, and a bare `;` is not one); the `always [0:3]` operator itself is legal (A.2.10) and the .6b.4 reading had bucketed it; the furthest TOKEN is the `else` after the `;`. MEASURED: rust/target/svpp_e1/s12b1/p4.sv (`always [0:3] a)` `;` `else`) fails at byte 75 on sv_2017 AND sv_2023; p4c.sv (no `;` before the else) accepts on both. Under the pipeline the corpus row rejects at [origin=t_prop_always.v:50:6] on both profiles."),
    ("verilator", "test_regress/t/t_dpi_var.v"): ("must_reject",
        "pinned SV-CORPUS-GRAD.12b.1: line 84-87, the item after the non-ANSI port list of module sub: `systemc_imp_header / void mon_class_name(const char* namep); ... / `verilog - Verilator's extension directives (not among the compiler directives IEEE 1800-2017 clause 22 lists; 1800-2023 likewise) enclosing C++ text no production of Annex A derives, INSIDE a module; the .6b.4 reading had looked at the port list (legal, the parse consumed it); the furthest TOKEN is the directive. The driver key tolerates it. MEASURED: the SVPP-EXPANSION.4c.1 reproducer rust/target/svpp_e1/s12b1/vendor_directive_repro.sv (`module t; `systemc_header ... `verilog endmodule`) fails at byte 9 on sv_2017 AND sv_2023; vendor_directive_control.sv accepts on both. Under the pipeline the corpus row rejects at [origin=t_dpi_var.v:82:5] (the port list's `);`, the item after it underivable) on both profiles."),
    ("verilator", "test_regress/t/t_extend_c_class.v"): ("must_reject",
        "pinned SV-CORPUS-GRAD.12b.1: line 47-49, the item after the always block: `systemc_header / #include \"t_extend_c_class_c.h\" / ... - Verilator's extension directive enclosing C++ text inside the module (IEEE 1800-2017 clause 22 lists no such directive; Annex A derives no `#include`); the `$c(...)` call the .6b.4 reading had bucketed is derivable (A.9.3 system_tf_identifier; measured: `out = $c(\"x(\", in, \")\");` accepts on both profiles); the furthest TOKEN is the directive after `end`. MEASURED: rust/target/svpp_e1/s12b1/vendor_directive_repro.sv fails at byte 9 on sv_2017 AND sv_2023; vendor_directive_control.sv accepts on both. Under the pipeline the corpus row rejects at [origin=t_extend_c_class.v:45:6] (the always block's end, the item after it underivable) on both profiles."),
    ("verilator", "test_regress/t/t_param_hier_bad.v"): ("must_reject",
        "pinned SV-CORPUS-GRAD.12b.1: line 46 `begin : block` at MODULE level (after line 44 `sub #(.X(block.block_func())) u_sub2 ();`) - a bare begin/end block as a module item: the SV-CORPUS-GRAD.3.56 ruling (IEEE 1800-2017 A.1.4 module_item / A.1.9 - no seq_block among the non-port module items); the driver's fails=True names Verilator's post-parse HIERPARAM error, but the syntax barrier is the bare block; the .6b.4 reading (`origin line 45 is BLANK`) had not read the furthest TOKEN. MEASURED: the .3.56 reproducer stands (a bare `begin ... end` at module level rejects on sv_2017 AND sv_2023; the same block inside an initial accepts); rust/target/svpp_e1/s12b1/tok.sh shows the furthest token at line 46 col 1. Under the pipeline the corpus row rejects at [origin=t_param_hier_bad.v:45:1] on both profiles."),
    # ---- SV-CORPUS-GRAD.3.59 (PGEN-SV-CORPUS-GRAD-0392) ---------------------------------------------
    # One of the .12b.1 grammar families read at the furthest TOKEN turned out to be the standard's own refusal.
    ("verilator", "test_regress/t/t_stream_unpack.v"): ("must_reject",
        "pinned SV-CORPUS-GRAD.3.59: line 89 `{ >> bit[1:0] {arr2}} = bit6;` - a streaming concatenation whose slice_size is a type WITH A PACKED DIMENSION; IEEE 1800-2017 A.8.1 slice_size ::= simple_type | constant_expression with A.2.2.1 simple_type ::= integer_type | non_integer_type | ps_type_identifier | ps_parameter_identifier (docs/systemverilog/2017/md/section-41-data-read-api.md:2896, :910) - `bit` is an integer_type, `bit[1:0]` is not a simple_type and not a constant_expression; 1800-2023 A.8.1 identical. The .6b.4 reading had bucketed the streaming lvalue (A.6.2, legal - measured: `{>> {arr2}} = b6;`, `{<< 2 {arr2}} = b6;`, `{ >> bit {arr2}} = b6;`, `{ >> byte {arr2}} = b6;` all ACCEPT on both profiles); the furthest TOKEN is the `[` of the packed dimension. The driver key tolerates it. MEASURED: rust/target/s359/rep/stream_packed_slice.sv (`{ >> bit[1:0] {arr2}} = b6;`) fails at byte 141 on sv_2017 AND sv_2023 (`logic[1:0]` at 143 likewise); ctrl_stream_bit_slice.sv (`{ >> bit {arr2}} = b6;`) accepts on both. Under the pipeline the corpus row rejects at [origin=t_stream_unpack.v:89:13] on both profiles."),
    ("Surelog", "tests/ForeachSquare/dut.sv"): ("must_reject",
        "pinned .13y: `foreach(array[i][j])` - loop variables in SEPARATE bracket pairs. IEEE 1800-2017 A.6.8 loop_statement ::= ... | foreach ( "
        "ps_or_hierarchical_array_identifier [ loop_variables ] ) statement (docs/systemverilog/2017/md/section-41-data-read-api.md:915), loop_variables ::= [ index_variable_identifier ] { , [ index_variable_identifier ] } "
        "(docs/systemverilog/2017/md/section-41-data-read-api.md:2360) - ONE bracket pair, comma-separated; 12.7.3 (docs/systemverilog/2017/md/section-12-procedural-programming-statements.md:1045, its own examples `foreach( A [ i, j, k ] )` and `foreach( B [ q, r, , s ] )` at :1082-1083). "
        "REJECTS at furthest_position=112 on sv_2017 and sv_2023 (not a .14a differential row) - a Surelog tolerance its golden log keys must_accept. Spec outranks the golden"),
    ("Surelog", "tests/SynthForeach/dut.sv"): ("must_reject",
        "pinned .13y: `foreach(array[i][j])` - loop variables in SEPARATE bracket pairs. IEEE 1800-2017 A.6.8 loop_statement ::= ... | foreach ( "
        "ps_or_hierarchical_array_identifier [ loop_variables ] ) statement (docs/systemverilog/2017/md/section-41-data-read-api.md:915), loop_variables ::= [ index_variable_identifier ] { , [ index_variable_identifier ] } "
        "(docs/systemverilog/2017/md/section-41-data-read-api.md:2360) - ONE bracket pair, comma-separated; 12.7.3 (docs/systemverilog/2017/md/section-12-procedural-programming-statements.md:1045, its own examples `foreach( A [ i, j, k ] )` and `foreach( B [ q, r, , s ] )` at :1082-1083). "
        "REJECTS at furthest_position=142 on sv_2017 and sv_2023 (not a .14a differential row) - a Surelog tolerance its golden log keys must_accept. Spec outranks the golden"),
    # ---- SV-CORPUS-GRAD.14b.2 (PGEN-SV-CORPUS-GRAD-0361) ------------------------------------------
    # Found by the sv_2023 LANE, not by the differential: these nine ivtest rows fail under BOTH profiles,
    # and the 2023 lane's .12a machinery explains their 2023 failure as a macro use (the parse reaches
    # a later, svpp-alterable point), which leaves their 2017 failure as the edition token itself -
    # the passing twins of the .8b.3 `_fail` cluster, keyed must_accept by ivtest's normal descriptor.
    # The same edition law: must_reject on the 2017 lane, must_accept on the sv_2023 lane (key_2023).
    ("iverilog", "ivtest/ivltests/sv_type_param_restrict_class1.v"): ("must_reject",
        "pinned .14b.2: line 8 `parameter type class T = C0` - a type parameter with a forward-type restriction. IEEE 1800-2023 A.2.1.1 "
        "type_parameter_declaration ::= type [ forward_type ] list_of_type_assignments (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:721), forward_type (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:803); "
        "IEEE 1800-2017 parameter type list_of_type_assignments (docs/systemverilog/2017/md/section-41-data-read-api.md:739-741) admits no forward_type - the construct does not exist "
        "in the edition this lane judges. EDITION CONTRAST (the .8b.3 law its `_fail` twin already carries): REJECTS at "
        "furthest_position=138 on sv_2017 (the forward-type token); under sv_2023 the parse passes that token and stops later at a "
        "point svpp can alter (the file uses macros), so the sv_2023 lane files it explained_svpp_macro_use - measured by the "
        "sv2023 lane (.14b.2). Pins: the .13x.d invalid_type_parameter_forward_type_sv2017.sv / control_type_parameter_forward_type_sv2023.sv. "
        "On the sv_2023 lane this row is must_accept (key_2023). Spec outranks the ivtest descriptor, which records what iverilog "
        "(with its 2023 extensions on) tolerates"),
    ("iverilog", "ivtest/ivltests/sv_type_param_restrict_class2.v"): ("must_reject",
        "pinned .14b.2: line 12 `parameter type class T = C0` - a type parameter with a forward-type restriction. IEEE 1800-2023 A.2.1.1 "
        "type_parameter_declaration ::= type [ forward_type ] list_of_type_assignments (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:721), forward_type (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:803); "
        "IEEE 1800-2017 parameter type list_of_type_assignments (docs/systemverilog/2017/md/section-41-data-read-api.md:739-741) admits no forward_type - the construct does not exist "
        "in the edition this lane judges. EDITION CONTRAST (the .8b.3 law its `_fail` twin already carries): REJECTS at "
        "furthest_position=179 on sv_2017 (the forward-type token); under sv_2023 the parse passes that token and stops later at a "
        "point svpp can alter (the file uses macros), so the sv_2023 lane files it explained_svpp_macro_use - measured by the "
        "sv2023 lane (.14b.2). Pins: the .13x.d invalid_type_parameter_forward_type_sv2017.sv / control_type_parameter_forward_type_sv2023.sv. "
        "On the sv_2023 lane this row is must_accept (key_2023). Spec outranks the ivtest descriptor, which records what iverilog "
        "(with its 2023 extensions on) tolerates"),
    ("iverilog", "ivtest/ivltests/sv_type_param_restrict_enum1.v"): ("must_reject",
        "pinned .14b.2: line 8 `parameter type enum T = T0` - a type parameter with a forward-type restriction. IEEE 1800-2023 A.2.1.1 "
        "type_parameter_declaration ::= type [ forward_type ] list_of_type_assignments (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:721), forward_type (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:803); "
        "IEEE 1800-2017 parameter type list_of_type_assignments (docs/systemverilog/2017/md/section-41-data-read-api.md:739-741) admits no forward_type - the construct does not exist "
        "in the edition this lane judges. EDITION CONTRAST (the .8b.3 law its `_fail` twin already carries): REJECTS at "
        "furthest_position=139 on sv_2017 (the forward-type token); under sv_2023 the parse passes that token and stops later at a "
        "point svpp can alter (the file uses macros), so the sv_2023 lane files it explained_svpp_macro_use - measured by the "
        "sv2023 lane (.14b.2). Pins: the .13x.d invalid_type_parameter_forward_type_sv2017.sv / control_type_parameter_forward_type_sv2023.sv. "
        "On the sv_2023 lane this row is must_accept (key_2023). Spec outranks the ivtest descriptor, which records what iverilog "
        "(with its 2023 extensions on) tolerates"),
    ("iverilog", "ivtest/ivltests/sv_type_param_restrict_enum2.v"): ("must_reject",
        "pinned .14b.2: line 12 `parameter type enum T = T0` - a type parameter with a forward-type restriction. IEEE 1800-2023 A.2.1.1 "
        "type_parameter_declaration ::= type [ forward_type ] list_of_type_assignments (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:721), forward_type (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:803); "
        "IEEE 1800-2017 parameter type list_of_type_assignments (docs/systemverilog/2017/md/section-41-data-read-api.md:739-741) admits no forward_type - the construct does not exist "
        "in the edition this lane judges. EDITION CONTRAST (the .8b.3 law its `_fail` twin already carries): REJECTS at "
        "furthest_position=181 on sv_2017 (the forward-type token); under sv_2023 the parse passes that token and stops later at a "
        "point svpp can alter (the file uses macros), so the sv_2023 lane files it explained_svpp_macro_use - measured by the "
        "sv2023 lane (.14b.2). Pins: the .13x.d invalid_type_parameter_forward_type_sv2017.sv / control_type_parameter_forward_type_sv2023.sv. "
        "On the sv_2023 lane this row is must_accept (key_2023). Spec outranks the ivtest descriptor, which records what iverilog "
        "(with its 2023 extensions on) tolerates"),
    ("iverilog", "ivtest/ivltests/sv_type_param_restrict_struct1.v"): ("must_reject",
        "pinned .14b.2: line 8 `parameter type struct T = T0` - a type parameter with a forward-type restriction. IEEE 1800-2023 A.2.1.1 "
        "type_parameter_declaration ::= type [ forward_type ] list_of_type_assignments (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:721), forward_type (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:803); "
        "IEEE 1800-2017 parameter type list_of_type_assignments (docs/systemverilog/2017/md/section-41-data-read-api.md:739-741) admits no forward_type - the construct does not exist "
        "in the edition this lane judges. EDITION CONTRAST (the .8b.3 law its `_fail` twin already carries): REJECTS at "
        "furthest_position=148 on sv_2017 (the forward-type token); under sv_2023 the parse passes that token and stops later at a "
        "point svpp can alter (the file uses macros), so the sv_2023 lane files it explained_svpp_macro_use - measured by the "
        "sv2023 lane (.14b.2). Pins: the .13x.d invalid_type_parameter_forward_type_sv2017.sv / control_type_parameter_forward_type_sv2023.sv. "
        "On the sv_2023 lane this row is must_accept (key_2023). Spec outranks the ivtest descriptor, which records what iverilog "
        "(with its 2023 extensions on) tolerates"),
    ("iverilog", "ivtest/ivltests/sv_type_param_restrict_struct2.v"): ("must_reject",
        "pinned .14b.2: line 12 `parameter type struct T = T0` - a type parameter with a forward-type restriction. IEEE 1800-2023 A.2.1.1 "
        "type_parameter_declaration ::= type [ forward_type ] list_of_type_assignments (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:721), forward_type (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:803); "
        "IEEE 1800-2017 parameter type list_of_type_assignments (docs/systemverilog/2017/md/section-41-data-read-api.md:739-741) admits no forward_type - the construct does not exist "
        "in the edition this lane judges. EDITION CONTRAST (the .8b.3 law its `_fail` twin already carries): REJECTS at "
        "furthest_position=197 on sv_2017 (the forward-type token); under sv_2023 the parse passes that token and stops later at a "
        "point svpp can alter (the file uses macros), so the sv_2023 lane files it explained_svpp_macro_use - measured by the "
        "sv2023 lane (.14b.2). Pins: the .13x.d invalid_type_parameter_forward_type_sv2017.sv / control_type_parameter_forward_type_sv2023.sv. "
        "On the sv_2023 lane this row is must_accept (key_2023). Spec outranks the ivtest descriptor, which records what iverilog "
        "(with its 2023 extensions on) tolerates"),
    ("iverilog", "ivtest/ivltests/sv_type_param_restrict_union1.v"): ("must_reject",
        "pinned .14b.2: line 9 `parameter type union T = T0` - a type parameter with a forward-type restriction. IEEE 1800-2023 A.2.1.1 "
        "type_parameter_declaration ::= type [ forward_type ] list_of_type_assignments (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:721), forward_type (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:803); "
        "IEEE 1800-2017 parameter type list_of_type_assignments (docs/systemverilog/2017/md/section-41-data-read-api.md:739-741) admits no forward_type - the construct does not exist "
        "in the edition this lane judges. EDITION CONTRAST (the .8b.3 law its `_fail` twin already carries): REJECTS at "
        "furthest_position=163 on sv_2017 (the forward-type token); under sv_2023 the parse passes that token and stops later at a "
        "point svpp can alter (the file uses macros), so the sv_2023 lane files it explained_svpp_macro_use - measured by the "
        "sv2023 lane (.14b.2). Pins: the .13x.d invalid_type_parameter_forward_type_sv2017.sv / control_type_parameter_forward_type_sv2023.sv. "
        "On the sv_2023 lane this row is must_accept (key_2023). Spec outranks the ivtest descriptor, which records what iverilog "
        "(with its 2023 extensions on) tolerates"),
    ("iverilog", "ivtest/ivltests/sv_type_param_restrict_union2.v"): ("must_reject",
        "pinned .14b.2: line 14 `parameter type union T = T0` - a type parameter with a forward-type restriction. IEEE 1800-2023 A.2.1.1 "
        "type_parameter_declaration ::= type [ forward_type ] list_of_type_assignments (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:721), forward_type (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:803); "
        "IEEE 1800-2017 parameter type list_of_type_assignments (docs/systemverilog/2017/md/section-41-data-read-api.md:739-741) admits no forward_type - the construct does not exist "
        "in the edition this lane judges. EDITION CONTRAST (the .8b.3 law its `_fail` twin already carries): REJECTS at "
        "furthest_position=228 on sv_2017 (the forward-type token); under sv_2023 the parse passes that token and stops later at a "
        "point svpp can alter (the file uses macros), so the sv_2023 lane files it explained_svpp_macro_use - measured by the "
        "sv2023 lane (.14b.2). Pins: the .13x.d invalid_type_parameter_forward_type_sv2017.sv / control_type_parameter_forward_type_sv2023.sv. "
        "On the sv_2023 lane this row is must_accept (key_2023). Spec outranks the ivtest descriptor, which records what iverilog "
        "(with its 2023 extensions on) tolerates"),
    ("iverilog", "ivtest/ivltests/sv_soft_packed_union.v"): ("must_reject",
        "pinned .14b.2: line 7 `typedef union soft packed {` - a soft union. IEEE 1800-2023 A.2.2.1 struct_union ::= struct | "
        "union [ soft | tagged ] (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:916-920); IEEE 1800-2017 A.2.2.1 struct_union ::= struct | union [ tagged ] (docs/systemverilog/2017/md/section-41-data-read-api.md:917) admits "
        "no `soft` - the construct does not exist in the edition this lane judges. EDITION CONTRAST (the .8b.3 law its `_fail1` twin "
        "already carries): REJECTS at furthest_position=130 on sv_2017 (the `soft` token); under sv_2023 the parse passes it and stops "
        "at line 28 `nested_t nested;`, a point svpp can alter (the file uses macros), so the sv_2023 lane files it "
        "explained_svpp_macro_use - measured by the sv2023 lane (.14b.2). Pins: the .14b.1 union_soft_packed triple. On the sv_2023 "
        "lane this row is must_accept (key_2023). Spec outranks the ivtest descriptor"),
    ("verible", "verible/verilog/tools/lint/testdata/bad-id-lex.sv"): (
        "must_reject", "pinned: in-file comment 'lexer should reject invalid "
        "identifier' (module 654foo)"),
    ("verible", "verible/verilog/tools/lint/testdata/module_begin_block.sv"): (
        "must_reject", "pinned: in-file comment marks the bare begin block "
        "'LRM-invalid syntax'"),
    # ---- GRAMMAR-WELLFORMED.A2.5 -- pin CORRECTED from must_reject to must_accept. --------
    #
    # ⛔ THE PIN WAS WRONG, AND IT LOOKED RIGHT ONLY BECAUSE A SEPARATE DEFECT MASKED IT.
    # It read: "'The LRM disallows cross_identifier as a bins_expression' - a grammar-level
    # restriction, and the driver CHECKs for an error". The restriction is REAL and the driver
    # does CHECK for an error, but it is NOT grammar-level, and no parser can enforce it:
    #
    #   bins_expression ::= variable_identifier | cover_point_identifier [ . bin_identifier ]
    #     -- IEEE 1800-2017 A.2.11, docs/systemverilog/2017/md/section-19-functional-coverage.md:1032
    #
    # and in Annex A *every* one of those identifier classes is the same terminal --
    # `cover_point_identifier := identifier` (grammars/systemverilog.ebnf:1742) and
    # `cross_identifier := identifier` (:1845). Telling `binsof(cp_a_cross_b)` from
    # `binsof(cp_a)` therefore requires knowing what the name was DECLARED as, i.e. name
    # resolution -- semantic, not syntactic. slang enforces it because slang elaborates; the
    # file's own header concedes the split ("Some tools accept this; --compat=all keeps the
    # diagnostic as a warning"). PGEN transcribes A.2.11 verbatim and is RIGHT to accept.
    #
    # ⭐ WHY IT SAT GREEN: PGEN did reject this file -- at its FIRST bins, `binsof(cp_a.one) &&
    # binsof(cp_b.one)`, because `select_expression`'s `&&` continuation was dead code
    # (SV-CORPUS-GRAD.13c.2a.2). The row never exercised the cross-identifier question at all.
    # A NEGATIVE TEST THAT PASSES FOR THE WRONG REASON is worth no more than a positive one that
    # does -- the same lesson `arm` was added to MANIFEST.tsv for, one level up, in the corpus
    # adjudication. A2.5 fixed the `&&` defect and the accidental agreement evaporated.
    # → [[a-catch-all-alternative-makes-an-accept-meaningless]], the `.3.15` "the parser is
    # correct and the manifest was wrong" precedent, and the corpus doctrine's LRM-above-tool rank.
    #
    # ⚠️ ROUTED, NOT SILENT: PGEN owns a semantic store (`@emit_fact`/`@predicate`) that could in
    # principle carry declaration-kind facts and enforce this class. Whether a PARSER should is a
    # scope question for the director, not a defect in this fix -- LANG-CAPABILITY-AUDIT.11.
    ("slang", "tests/regression/driver/cross-ident-in-binsof.sv"): (
        "must_accept", "pinned (A2.5, CORRECTED): the cross-identifier restriction is "
        "SEMANTIC, not grammar-level - A.2.11's bins_expression admits any identifier and "
        "cover_point_identifier/cross_identifier are the same terminal, so only name "
        "resolution can separate them; slang errors because slang elaborates"),

    # ---- SV-CORPUS-GRAD.3.15 -- the bare generate_block family. ----------------
    #
    # ⛔ THE PARSER IS CORRECT ON ALL 19 AND THE MANIFEST WAS WRONG. Each file uses a
    # begin/end block where IEEE 1800-2017 Annex A has no `generate_block` at all:
    #
    #   generate_region ::= generate { generate_item } endgenerate
    #   generate_item   ::= module_or_generate_item | interface_or_generate_item
    #                     | checker_or_generate_item
    #   generate_block  ::= generate_item
    #                     | [ id : ] begin [ : id ] { generate_item } end [ : id ]
    #     -- A.4.2, mirrored verbatim by grammars/systemverilog.ebnf:2517-2521
    #
    # `generate_block` is reachable ONLY from `loop_generate_construct`,
    # `if_generate_construct` and `case_generate_item`. So `for (...) begin : A ... end`
    # and `if (P) begin : A ... end` are legal and DO parse (repro/C_*.sv, repro/D_*.sv
    # both pass), while the same block sitting directly in a generate region or a module
    # body does not -- it is the pre-2005 "legacy generate region", which verible's own
    # lint rule NAMES as legacy in one of the very fixtures pinned below.
    #
    # ⭐⭐ THE REPO HAD ALREADY RULED THIS WAY, on the same construct, from the same suite:
    # `module_begin_block.sv` two entries above is pinned `must_reject` because verible's
    # in-file comment calls a bare begin block "LRM-invalid syntax". Its sibling fixture
    # `generate_begin_module.sv` carried the OPPOSITE verdict purely because no comment
    # happened to say so -- the two differ by an upstream author's annotation, not by
    # anything in the LRM. That inconsistency IS the adjudicator hole this leaf closes.
    #
    # ⛔ This is an ADJUDICATION CORRECTION, never burn-down yield: no parser byte changes
    # and the construct stays REJECTED. Tolerating it is a FUTURE, ADDITIVE, EBNF-declared
    # dialect layer if it is ever wanted (feedback_sv_strict_lrm_compliance_default) --
    # making it parse today would convert a correct reject into an over-acceptance defect.
    ("Surelog", "tests/DefParamIndex/dut.sv"): (
        "must_reject", "pinned .3.15: bare begin/end generate_block at line 5 - "
        "IEEE 1800-2017 A.4.2 admits a generate_block only under "
        "loop_generate_construct / if_generate_construct / case_generate_item; "
        "neither generate_item nor module_or_generate_item has that alternative; "
        "spec outranks the Surelog golden-log heuristic"),
    ("Surelog", "tests/GenerateBlock/dut.sv"): (
        "must_reject", "pinned .3.15: bare begin/end generate_block at line 4 - "
        "IEEE 1800-2017 A.4.2 admits a generate_block only under "
        "loop_generate_construct / if_generate_construct / case_generate_item; "
        "neither generate_item nor module_or_generate_item has that alternative; "
        "spec outranks the Surelog golden-log heuristic"),
    ("Surelog", "tests/GenerateRegion/dut.sv"): (
        "must_reject", "pinned .3.15: bare begin/end generate_block at line 49 - "
        "IEEE 1800-2017 A.4.2 admits a generate_block only under "
        "loop_generate_construct / if_generate_construct / case_generate_item; "
        "neither generate_item nor module_or_generate_item has that alternative; "
        "spec outranks the Surelog golden-log heuristic"),
    ("Surelog", "tests/ImplicitGenBlock/dut.sv"): (
        "must_reject", "pinned .3.15: bare begin/end generate_block at line 37 - "
        "IEEE 1800-2017 A.4.2 admits a generate_block only under "
        "loop_generate_construct / if_generate_construct / case_generate_item; "
        "neither generate_item nor module_or_generate_item has that alternative; "
        "spec outranks the Surelog golden-log heuristic"),
    ("sv2v", "test/core/array.sv"): (
        "must_reject", "pinned .3.15: bare begin/end generate_block at line 32 - "
        "IEEE 1800-2017 A.4.2 admits a generate_block only under "
        "loop_generate_construct / if_generate_construct / case_generate_item; "
        "neither generate_item nor module_or_generate_item has that alternative; "
        "spec outranks the sv2v suite-contract heuristic"),
    ("sv2v", "test/core/cast.sv"): (
        "must_reject", "pinned .3.15: bare begin/end generate_block at line 5 - "
        "IEEE 1800-2017 A.4.2 admits a generate_block only under "
        "loop_generate_construct / if_generate_construct / case_generate_item; "
        "neither generate_item nor module_or_generate_item has that alternative; "
        "spec outranks the sv2v suite-contract heuristic"),
    ("sv2v", "test/core/decl_scope.sv"): (
        "must_reject", "pinned .3.15: bare begin/end generate_block at line 6 - "
        "IEEE 1800-2017 A.4.2 admits a generate_block only under "
        "loop_generate_construct / if_generate_construct / case_generate_item; "
        "neither generate_item nor module_or_generate_item has that alternative; "
        "spec outranks the sv2v suite-contract heuristic"),
    ("sv2v", "test/core/inside_expr.sv"): (
        "must_reject", "pinned .3.15: bare begin/end generate_block at line 27 - "
        "IEEE 1800-2017 A.4.2 admits a generate_block only under "
        "loop_generate_construct / if_generate_construct / case_generate_item; "
        "neither generate_item nor module_or_generate_item has that alternative; "
        "spec outranks the sv2v suite-contract heuristic"),
    ("sv2v", "test/core/logic_tf.sv"): (
        "must_reject", "pinned .3.15: bare begin/end generate_block at line 18 - "
        "IEEE 1800-2017 A.4.2 admits a generate_block only under "
        "loop_generate_construct / if_generate_construct / case_generate_item; "
        "neither generate_item nor module_or_generate_item has that alternative; "
        "spec outranks the sv2v suite-contract heuristic"),
    ("verible", "verible/verilog/tools/lint/testdata/generate_begin_module.sv"): (
        "must_reject", "pinned .3.15: bare begin/end generate_block at line 4 - "
        "IEEE 1800-2017 A.4.2 admits a generate_block only under "
        "loop_generate_construct / if_generate_construct / case_generate_item; "
        "neither generate_item nor module_or_generate_item has that alternative; "
        "spec outranks the verible fixture heuristic"),
    ("verilator", "test_regress/t/t_default_disable_iff_gen_multi_bad.v"): (
        "must_reject", "pinned .3.15: bare begin/end generate_block at line 13 - "
        "IEEE 1800-2017 A.4.2 admits a generate_block only under "
        "loop_generate_construct / if_generate_construct / case_generate_item; "
        "neither generate_item nor module_or_generate_item has that alternative; "
        "spec outranks the verilator driver heuristic"),
    ("verilator", "test_regress/t/t_dpi_string.v"): (
        "must_reject", "pinned .3.15: bare begin/end generate_block at line 14 - "
        "IEEE 1800-2017 A.4.2 admits a generate_block only under "
        "loop_generate_construct / if_generate_construct / case_generate_item; "
        "neither generate_item nor module_or_generate_item has that alternative; "
        "spec outranks the verilator driver heuristic"),
    ("verilator", "test_regress/t/t_gen_duplicated_blocks_bad.v"): (
        "must_reject", "pinned .3.15: bare begin/end generate_block at line 9 - "
        "IEEE 1800-2017 A.4.2 admits a generate_block only under "
        "loop_generate_construct / if_generate_construct / case_generate_item; "
        "neither generate_item nor module_or_generate_item has that alternative; "
        "spec outranks the verilator driver heuristic"),
    ("verilator", "test_regress/t/t_gen_if.v"): (
        "must_reject", "pinned .3.15: bare begin/end generate_block at line 23 - "
        "IEEE 1800-2017 A.4.2 admits a generate_block only under "
        "loop_generate_construct / if_generate_construct / case_generate_item; "
        "neither generate_item nor module_or_generate_item has that alternative; "
        "spec outranks the verilator driver heuristic"),
    ("verilator", "test_regress/t/t_gen_local.v"): (
        "must_reject", "pinned .3.15: bare begin/end generate_block at line 19 - "
        "IEEE 1800-2017 A.4.2 admits a generate_block only under "
        "loop_generate_construct / if_generate_construct / case_generate_item; "
        "neither generate_item nor module_or_generate_item has that alternative; "
        "spec outranks the verilator driver heuristic"),
    ("verilator", "test_regress/t/t_gen_self_return.v"): (
        "must_reject", "pinned .3.15: bare begin/end generate_block at line 31 - "
        "IEEE 1800-2017 A.4.2 admits a generate_block only under "
        "loop_generate_construct / if_generate_construct / case_generate_item; "
        "neither generate_item nor module_or_generate_item has that alternative; "
        "spec outranks the verilator driver heuristic"),
    ("verilator", "test_regress/t/t_interface_gen10.v"): (
        "must_reject", "pinned .3.15: bare begin/end generate_block at line 28 - "
        "IEEE 1800-2017 A.4.2 admits a generate_block only under "
        "loop_generate_construct / if_generate_construct / case_generate_item; "
        "neither generate_item nor module_or_generate_item has that alternative; "
        "spec outranks the verilator driver heuristic"),
    ("verilator", "test_regress/t/t_interface_gen5.v"): (
        "must_reject", "pinned .3.15: bare begin/end generate_block at line 28 - "
        "IEEE 1800-2017 A.4.2 admits a generate_block only under "
        "loop_generate_construct / if_generate_construct / case_generate_item; "
        "neither generate_item nor module_or_generate_item has that alternative; "
        "spec outranks the verilator driver heuristic"),
    ("verilator", "test_regress/t/t_interface_gen7.v"): (
        "must_reject", "pinned .3.15: bare begin/end generate_block at line 34 - "
        "IEEE 1800-2017 A.4.2 admits a generate_block only under "
        "loop_generate_construct / if_generate_construct / case_generate_item; "
        "neither generate_item nor module_or_generate_item has that alternative; "
        "spec outranks the verilator driver heuristic"),

    # ---- SV-CORPUS-GRAD.3.16 -- `inside` where a CONSTANT expression is required. ---------
    #
    # ⛔ THE PARSER IS CORRECT ON BOTH. IEEE 1800-2017 A.8.3, verbatim:
    #
    #   constant_expression ::=
    #       constant_primary
    #     | unary_operator { attribute_instance } constant_primary
    #     | constant_expression binary_operator { attribute_instance } constant_expression
    #     | constant_expression ? { attribute_instance } constant_expression
    #                             : constant_expression
    #
    # There is NO `inside` alternative, and `inside` is not a `binary_operator` either --
    # A.8.6 lists them exhaustively (+ - * / % == != === !== ==? !=? && || ** < <= > >= &
    # | ^ ^~ ~^ >> << >>> <<< -> <->) and `inside` is not among them. The set-membership
    # operator has its own production, an alternative of `expression` ONLY:
    #
    #   inside_expression ::= expression inside { open_range_list }
    #     -- A.8.3, quoted again as Syntax 11-3 in section 11.4.13
    #
    # So `inside` is legal in an expression and illegal wherever the grammar demands a
    # constant_expression: an if-generate condition (A.4.2 `if ( constant_expression )`) and
    # a localparam initializer (A.2.3 param_assignment -> constant_param_expression ->
    # constant_mintypmax_expression -> constant_expression) are the two shapes measured here.
    #
    # ⭐ The discriminating pair is TRACKED, and it shows the parser mirrors Annex A exactly:
    # gen_block_family/repro/F_inside_in_expression.sv (`initial b = a inside {1, 2};`)
    # PASSES, while E_inside_in_constant_expression.sv (`localparam int B = A inside {1, 2};`)
    # rejects AT the keyword. Same operator, same file shape, different required nonterminal.
    ("Surelog", "tests/InsideOp/dut.sv"): (
        "must_reject", "pinned .3.16: `inside` in an if-generate CONSTANT condition at "
        "line 25 - IEEE 1800-2017 A.8.3 constant_expression has no inside alternative and "
        "A.8.6 binary_operator does not list `inside`; A.4.2 requires "
        "if ( constant_expression ); spec outranks the Surelog golden-log heuristic"),
    ("verilator", "test_regress/t/t_inside_unpacked_param.v"): (
        "must_reject", "pinned .3.16: `inside` in a localparam initializer at line 13 - "
        "IEEE 1800-2017 A.2.3 param_assignment takes a constant_param_expression, and "
        "A.8.3 constant_expression has no inside alternative (inside_expression is an "
        "alternative of expression only); spec outranks the verilator driver heuristic"),

    # ---- SV-CORPUS-GRAD.3.23 -- `enum [N:M] { … }`: a packed dimension with NO base type. ----
    #
    # ⛔ THE PARSER IS CORRECT ON ALL 9 AND THE MANIFEST WAS WRONG. IEEE 1800-2017 A.2.2.1:
    #
    #   enum_base_type ::= integer_atom_type [ signing ]
    #                    | integer_vector_type [ signing ] [ packed_dimension ]
    #                    | type_identifier [ packed_dimension ]
    #     -- mirrored by grammars/systemverilog.ebnf's enum_base_type
    #
    # A `packed_dimension` is reachable ONLY behind a type, so a bare `enum [2:0] {…}` has no
    # derivation. Clause 6.19 says the same thing in prose, independently: "In the absence of
    # a data type declaration, the default data type shall be int. Any other data type used
    # with enumerated types shall require an explicit data type declaration." A `[2:0]`
    # dimension IS another data type (a 3-bit vector, not int), so it REQUIRES the explicit
    # declaration it is missing -- and every dimension-bearing 6.19 example writes one
    # (`enum bit [1:0] {IDLE, …}`). Measured, not recalled: the 1800-2017 and 1800-2023 texts
    # contain ZERO literal base-less `enum [` occurrences.
    #
    # ⭐ The BOUNDARY IS PRECISE, not a hole in enum support (matrix in
    # docs/tasks/artifacts/sv_corpus_grad/enum_base_range/sweep.txt): `enum {…}`,
    # `enum int {…}`, `enum logic [2:0] {…}`, `enum bit signed [2:0] {…}` and
    # `enum my_t [2:0] {…}` ALL parse. Exactly one shape rejects, and it is exactly the one
    # Annex A cannot derive.
    #
    # ⛔ WHY must_reject AND NOT a `vendor_extension` DEFERRAL. `out_of_scope_with_cause`
    # means "owned by ANOTHER LANE" and its rows adjudicate to `deferred:` -- no verdict, no
    # claim. There is no vendor-dialect lane to own these, so a deferral would remove the
    # rows from the proof surface while asserting nothing. `must_reject` makes today's REJECT
    # a `match`: the parser is claimed CORRECT WITH A CITE, and CI re-verifies it. Same
    # ruling, same reasoning and the same class as .3.14a/.3.15/.3.16 -- spec outranks tool.
    #
    # ⛔ PER-FILE, NOT BY FAMILY MEMBERSHIP. A file can reject for more than one reason, and
    # pinning it on the enum ground would then MASK that other defect. Each of the 9 is
    # proven stuck INSIDE its first base-less enum header and nowhere earlier, by
    # verify_pins.py (same artifact dir; controls: a positive reproducer, a planted
    # earlier-error negative, and a legal typed-base negative -- it REFUSES on a miss).
    # Honest bound carried by that instrument: a pinned row can no longer testify about
    # anything AFTER its enum header. Inherent (the text is LRM-underivable, so the file can
    # never be must_accept), but a real loss of reach rather than a free win.
    ("sv2v", "test/core/enum_scope.sv"): (
        "must_reject", "pinned .3.23: `enum [5:0] {` at line 37 - a packed dimension with "
        "no base type; IEEE 1800-2017 A.2.2.1 admits packed_dimension only behind an "
        "integer_vector_type or a type_identifier, and 6.19 requires an explicit data type "
        "declaration for any non-int enum base"),
    ("verilator", "test_regress/t/t_cast.v"): (
        "must_reject", "pinned .3.23: `enum [15:0] {` at line 27 - base-less packed "
        "dimension, no A.2.2.1 derivation (6.19 requires the explicit declaration); spec "
        "outranks the verilator driver heuristic"),
    ("verilator", "test_regress/t/t_debug_emitv.v"): (
        "must_reject", "pinned .3.23: `enum [2:0] {` at line 57 - base-less packed "
        "dimension, no A.2.2.1 derivation (6.19 requires the explicit declaration); spec "
        "outranks the verilator driver heuristic"),
    ("verilator", "test_regress/t/t_enum.v"): (
        "must_reject", "pinned .3.23: `enum [2:0] {` at line 39 - base-less packed "
        "dimension, no A.2.2.1 derivation (6.19 requires the explicit declaration); spec "
        "outranks the verilator driver heuristic"),
    ("verilator", "test_regress/t/t_enum_bad_value.v"): (
        "must_reject", "pinned .3.23: `enum [2:0] {` at line 12 - base-less packed "
        "dimension, no A.2.2.1 derivation. ⚠️ The file's own `_bad` label names an "
        "out-of-range enum VALUE (an elaboration error) - a DIFFERENT defect from the one "
        "the parser hits; the name is not the oracle here"),
    ("verilator", "test_regress/t/t_enum_bad_wrap.v"): (
        "must_reject", "pinned .3.23: `enum [1:0] {` at line 9 - base-less packed "
        "dimension, no A.2.2.1 derivation. ⚠️ Same as t_enum_bad_value.v: the `_bad` label "
        "names a value-wrap elaboration error, not this syntax"),
    ("verilator", "test_regress/t/t_enum_const_methods.v"): (
        "must_reject", "pinned .3.23: `enum [1:0] {` at line 9 - base-less packed "
        "dimension, no A.2.2.1 derivation (6.19 requires the explicit declaration); spec "
        "outranks the verilator driver heuristic"),
    ("verilator", "test_regress/t/t_enum_type_methods_bad.v"): (
        "must_reject", "pinned .3.23: `enum [3:0] {` at line 14 - base-less packed "
        "dimension, no A.2.2.1 derivation; the `_bad` label names a post-parse method-call "
        "error, so this reject is on the syntax ground stated here"),
    ("verilator", "test_regress/t/t_enum_type_nomethod_bad.v"): (
        "must_reject", "pinned .3.23: `enum [3:0] {` at line 8 - base-less packed "
        "dimension, no A.2.2.1 derivation; the `_bad` label names a post-parse method-call "
        "error, so this reject is on the syntax ground stated here"),
    # ---- SV-CORPUS-GRAD.3.51 (PGEN-SV-CORPUS-GRAD-0381): the residual bar triaged by production ----
    # Twenty-six rows whose text no edition derives, each with the production, the measured stop and a pin pair.
    ('sv2v', 'test/core/trailing_commas.sv'): ('must_reject',
        "pinned .3.51: line 1 `module ModuleA #(P=1,) (inp,);` - TRAILING COMMAS in a parameter port list and a port list. IEEE 1800-2017 A.1.3 parameter_port_list ::= # ( list_of_param_assignments { , parameter_port_declaration } ) | # ( parameter_port_declaration { , parameter_port_declaration } ) | #( ) (docs/systemverilog/2017/md/section-41-data-read-api.md:231-236) and list_of_ports ::= ( port { , port } ) (docs/systemverilog/2017/md/section-41-data-read-api.md:246) - every comma is followed by an item; IEEE 1800-2023 A.1.3 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:208-213, :223). REJECTS at furthest_position=21 on sv_2017 and sv_2023 (the `)` after the first trailing comma). Controls (rust/target/s351/rep, release probe 1.0.223 a2ca4c477d54c216…): `module ModuleA #(P=1) (inp);` ACCEPTS on every profile. Pins invalid_trailing_comma_in_port_lists.sv / control_port_lists_without_trailing_comma.sv. the file is sv2v's own test of the tolerance its converter offers; no edition derives a trailing comma"),
    ('sv2v', 'test/core/for_loop_inits.sv'): ('must_reject',
        "pinned .3.51: line 6 `for (x += 1; x < y; x++)` - an OPERATOR assignment as the for-loop initialization. IEEE 1800-2017 A.6.8 for_initialization ::= list_of_variable_assignments | for_variable_declaration { , for_variable_declaration } (docs/systemverilog/2017/md/section-41-data-read-api.md:2336-2340) with A.6.2 variable_assignment ::= variable_lvalue = expression (docs/systemverilog/2017/md/section-41-data-read-api.md:2053) - the initialization is a plain `=` assignment; the operator forms are for_step's; IEEE 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:2321-2325, :2055). REJECTS at furthest_position=96 on sv_2017 and sv_2023 (the `+`). Controls (rust/target/s351/rep, release probe 1.0.223 a2ca4c477d54c216…): `for (x = 1; x < y; x++) ;` ACCEPTS on the SV profiles. Pins invalid_for_initialization_operator_assignment.sv / control_for_initialization_assignment.sv. sv2v's conversion input; no edition derives an operator assignment in for_initialization"),
    ('iverilog', 'ivtest/ivltests/pr3366217d.v'): ('must_reject',
        'pinned .3.51: line 17 `enum {ndef[-1]} ndef1;` - a NEGATIVE enum name index. IEEE 1800-2017 A.2.2.1 enum_name_declaration ::= enum_identifier [ [ integral_number [ : integral_number ] ] ] [ = constant_expression ] (docs/systemverilog/2017/md/section-41-data-read-api.md:875-878) with A.8.7 integral_number ::= decimal_number | octal_number | binary_number | hex_number (docs/systemverilog/2017/md/section-41-data-read-api.md:3240-3244) - an unsigned literal, no sign; IEEE 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:858-861, :3180-3184). REJECTS at furthest_position=465 on sv_2017 and sv_2023 (the `-`). Controls (rust/target/s351/rep, release probe 1.0.223 a2ca4c477d54c216…): `enum {{ndef[1]}} ndef1;` ACCEPTS on the SV profiles. Pins invalid_enum_name_negative_index.sv / control_enum_name_index.sv. the ivtest golden records a post-parse error, but the index is not derivable: the parser is right to stop at the sign'),
    ('iverilog', 'ivtest/ivltests/pr3515542.v'): ('must_reject',
        "pinned .3.51: line 5 `a = 15'1;` - a SIZE on an unbased unsized literal. IEEE 1800-2017 A.8.7 unbased_unsized_literal ::= '0 | '1 | 'z_or_x (docs/systemverilog/2017/md/section-41-data-read-api.md:3290) takes no size, and decimal_number ::= unsigned_number | [ size ] decimal_base unsigned_number | ... (docs/systemverilog/2017/md/section-41-data-read-api.md:3253-3258) needs a base after a size - `15'1` is neither; IEEE 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:3231, :3188). REJECTS at furthest_position=53 on sv_2017 and sv_2023 (the `'` after `15`). Controls (rust/target/s351/rep, release probe 1.0.223 a2ca4c477d54c216…): `a = '1;` ACCEPTS on the SV profiles. Pins invalid_sized_unbased_unsized_literal.sv / control_unbased_unsized_literal.sv. the ivtest golden records a post-parse error, but the token is not derivable"),
    ('Surelog', 'tests/HierPathModule/dut.sv'): ('must_reject',
        "pinned .3.51: line 3 `assign o = medium.c;` - the reserved word `medium` bound as a module name (`module medium();`) and used in a hierarchical path. IEEE 1800-2017 Annex B Table B.1 reserves `medium` (docs/systemverilog/2017/md/section-41-data-read-api.md:3722; the table docs/systemverilog/2017/md/section-41-data-read-api.md:3599-3858; a charge strength, A.2.2.2) and so does IEEE 1364-2005 Annex B (docs/verilog/2005/md/section-Annex_B-normative-list-of-keywords.md:79); IEEE 1800-2023 Annex B the same reserved set (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:3599). REJECTS at furthest_position=30 on sv_2017 and sv_2023 (the `medium`). Controls (rust/target/s351/rep, release probe 1.0.223 a2ca4c477d54c216…): the same unit with the module named `med` ACCEPTS on every profile. Pins invalid_reserved_word_medium_as_module_name.sv / control_module_named_non_keyword.sv. Surelog's golden log completes because Surelog tolerates a reserved word as a module name; a reserved word is not an identifier in either standard (the .3.37 Annex B ruling)"),
    ('slang', 'tests/regression/all.sv'): ('must_reject',
        "pinned .3.51: line 255 `default disable iff 1 dist { [1:2] :/ 3, 2, default :/ 4 };` - `default` as a dist item - an EDITION CONTRAST. IEEE 1800-2017 A.1.10 dist_item ::= value_range [ dist_weight ] (docs/systemverilog/2017/md/section-41-data-read-api.md:669) with value_range ::= expression | [ expression : expression ] (docs/systemverilog/2017/md/section-41-data-read-api.md:3044-3047) has no default item, and the item `default disable iff expression_or_dist ;` takes no parentheses (docs/systemverilog/2017/md/section-41-data-read-api.md:354); IEEE 1800-2023 A.1.10 dist_item ::= value_range [ dist_weight ] | default :/ expression (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:636-639; 18.5.3 docs/systemverilog/2023/md/section-18-constrained-random-value-generation.md:652-656) ADDS it - legal in the edition the 2023 lane judges (the 2023 probe parses past it to furthest_position=14074). REJECTS at furthest_position=5694 on sv_2017 and sv_2023 (the `default`). Controls (rust/target/s351/rep, release probe 1.0.223 a2ca4c477d54c216…): `default disable iff 1 dist {{ [1:2] :/ 3, 2 }};` ACCEPTS on both SV profiles; the default item ACCEPTS under sv_2023 alone. Pins invalid_dist_default_under_2017.sv / control_dist_default_sv2023.sv / control_dist_without_default.sv. slang's regression fixture mixes editions; the row is legal 1800-2023 text and not 1800-2017 text"),
    ('iverilog', 'ivtest/ivltests/sv_wildcard_import2.v'): ('must_reject',
        "pinned .3.51: line 39 `#1 ->my_package::e;` - an event trigger on a PACKAGE-SCOPED event. IEEE 1800-2017 A.6.5 event_trigger ::= -> hierarchical_event_identifier ; | ->> [ delay_or_event_control ] hierarchical_event_identifier ; (docs/systemverilog/2017/md/section-41-data-read-api.md:2191-2194) with A.9.3 hierarchical_identifier ::= [ $root . ] { identifier constant_bit_select . } identifier (docs/systemverilog/2017/md/section-41-data-read-api.md:3361) - no package scope; IEEE 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:2189-2192, :3310). REJECTS at furthest_position=426 on sv_2017 and sv_2023 (the `::`). Controls (rust/target/s351/rep, release probe 1.0.223 a2ca4c477d54c216…): `->e;` on a local event ACCEPTS on every profile. Pins invalid_package_scoped_event_trigger.sv / control_event_trigger_local.sv. iverilog's SV dialect triggers a package event; hierarchical_identifier has no package scope"),
    ('iverilog', 'ivtest/ivltests/sv_wildcard_import3.v'): ('must_reject',
        "pinned .3.51: line 37 `#1 ->my_package::e;` - an event trigger on a PACKAGE-SCOPED event. IEEE 1800-2017 A.6.5 event_trigger ::= -> hierarchical_event_identifier ; | ->> [ delay_or_event_control ] hierarchical_event_identifier ; (docs/systemverilog/2017/md/section-41-data-read-api.md:2191-2194) with A.9.3 hierarchical_identifier ::= [ $root . ] { identifier constant_bit_select . } identifier (docs/systemverilog/2017/md/section-41-data-read-api.md:3361) - no package scope; IEEE 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:2189-2192, :3310). REJECTS at furthest_position=385 on sv_2017 and sv_2023 (the `::`). Controls (rust/target/s351/rep, release probe 1.0.223 a2ca4c477d54c216…): `->e;` on a local event ACCEPTS on every profile. Pins invalid_package_scoped_event_trigger.sv / control_event_trigger_local.sv. iverilog's SV dialect triggers a package event; hierarchical_identifier has no package scope"),
    ('sv2v', 'test/nosim/assert.sv'): ('must_reject',
        "pinned .3.51: line 18 `assert #0_0 (1);` - the deferred assertion's `#0` spelled `#0_0`. IEEE 1800-2017 A.6.10 deferred_immediate_assert_statement ::= assert #0 ( expression ) action_block | assert final ( expression ) action_block (docs/systemverilog/2017/md/section-41-data-read-api.md:2421-2424) - the `#0` is the production's own token, not a delay value; IEEE 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:2406-2409). REJECTS at furthest_position=401 on sv_2017 and sv_2023 (the `0_0`). Controls (rust/target/s351/rep, release probe 1.0.223 a2ca4c477d54c216…): `assert #0 (1);` ACCEPTS on the SV profiles (the file's own earlier lines). Pins invalid_deferred_assert_delay_spelled_0_0.sv / control_deferred_assert_zero.sv. sv2v's own tolerance test; the production spells #0 literally"),
    ('sv2v', 'test/core/delay.sv'): ('must_reject',
        "pinned .3.51: line 2 `logic [4] a, b;` - a PACKED dimension in the size form (the .3.48 ruling). IEEE 1800-2017 A.2.5 packed_dimension ::= [ constant_range ] | unsized_dimension (docs/systemverilog/2017/md/section-41-data-read-api.md:1058-1062) - a packed dimension is a range; the size form is unpacked_dimension's (docs/systemverilog/2017/md/section-41-data-read-api.md:1046-1050); IEEE 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1085-1089). REJECTS at furthest_position=24 on sv_2017 and sv_2023 (the `4` after the `[`). Controls (rust/target/s351/rep, release probe 1.0.223 a2ca4c477d54c216…): `logic [3:0] a, b;` ACCEPTS on every profile (the .3.48 control_packed_dimension_range.sv). Pins invalid_packed_dimension_size.sv (worklist_rows 4 -> 5). the .3.48 ruling; sv2v reads a packed `[N]` as `[N-1:0]`"),
    ('sv2v', 'test/core/struct_unit_array.sv'): ('must_reject',
        "pinned .3.51: line 11 `S [WIDTH] s = '{` - the PACKED size form on a typedef'd type. IEEE 1800-2017 A.2.2.1 data_type ::= ... | [ class_scope | package_scope ] type_identifier { packed_dimension } (docs/systemverilog/2017/md/section-41-data-read-api.md:855) with A.2.5 packed_dimension ::= [ constant_range ] | unsized_dimension (docs/systemverilog/2017/md/section-41-data-read-api.md:1058-1062); IEEE 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:838, :1085-1089). REJECTS at furthest_position=175 on sv_2017 and sv_2023 (the `W` after the `[`). Controls (rust/target/s351/rep, release probe 1.0.223 a2ca4c477d54c216…): `S [1:0] s;` ACCEPTS on the SV profiles. Pins invalid_packed_dimension_size_on_type_identifier.sv / control_packed_dimension_range_on_type_identifier.sv. the .3.48 ruling on a type_identifier"),
    ('sv2v', 'test/core/enum.sv'): ('must_reject',
        "pinned .3.51: line 25 `typedef enum [0:0] {` - a packed dimension with NO base type on an enum. IEEE 1800-2017 A.2.2.1 enum_base_type ::= integer_atom_type [ signing ] | integer_vector_type [ signing ] [ packed_dimension ] | type_identifier [ packed_dimension ] (docs/systemverilog/2017/md/section-41-data-read-api.md:868-873) - the dimension follows a vector type or a type identifier, never stands alone; IEEE 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:851-856). REJECTS at furthest_position=333 on sv_2017 and sv_2023 (the `[`). Controls (rust/target/s351/rep, release probe 1.0.223 a2ca4c477d54c216…): `typedef enum logic [0:0] {{ G_1, G_2 }} EnumG;` ACCEPTS on the SV profiles. Pins invalid_enum_base_dimension_without_type.sv / control_enum_base_type_vector.sv. sv2v's conversion input; no edition derives a bare dimension"),
    ('iverilog', 'ivtest/ivltests/sv_type_identifier_attribute_target.v'): ('must_reject',
        'pinned .3.51: line 13 `$attribute(ATTR_TYPE, "test", "true")` - iverilog\'s `$attribute(...)` as a top-level item. IEEE 1800-2017 A.1.2 description ::= module_declaration | udp_declaration | interface_declaration | program_declaration | package_declaration | { attribute_instance } package_item | { attribute_instance } bind_directive | config_declaration (docs/systemverilog/2017/md/section-41-data-read-api.md:79-90) - no system task call; IEEE 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:64-75). REJECTS at furthest_position=205 on sv_2017 and sv_2023 (the `$`). Controls (rust/target/s351/rep, release probe 1.0.223 a2ca4c477d54c216…): the same unit without the line ACCEPTS on the SV profiles. Pins invalid_attribute_system_call_as_description.sv / control_typedef_description_item.sv. iverilog\'s attribute pragma; no edition derives it'),
    ('verible', 'verible/verilog/tools/kythe/testdata/foreach.sv'): ('must_reject',
        "pinned .3.51: line 9 `foreach (y) begin` - a foreach with NO loop-variable brackets. IEEE 1800-2017 A.6.8 loop_statement ::= ... | foreach ( ps_or_hierarchical_array_identifier [ loop_variables ] ) statement (docs/systemverilog/2017/md/section-41-data-read-api.md:2334) - the brackets are the production's own tokens (loop_variables may be empty, the brackets may not); IEEE 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:2319). REJECTS at furthest_position=181 on sv_2017 and sv_2023 (the `)`). Controls (rust/target/s351/rep, release probe 1.0.223 a2ca4c477d54c216…): `foreach (y[i]) y[i] = 0;` ACCEPTS on the SV profiles. Pins invalid_foreach_without_loop_variables.sv / control_foreach_loop_variables.sv. verible's indexing fixture; the brackets are literal"),
    ('iverilog', 'ivtest/ivltests/sf1289.v'): ('must_reject',
        "pinned .3.51: line 18 `foreach (a[i]) ;` - a foreach whose body is the NULL statement. IEEE 1800-2017 A.6.8 foreach ( ps_or_hierarchical_array_identifier [ loop_variables ] ) statement (docs/systemverilog/2017/md/section-41-data-read-api.md:2334) - `statement`, where every other loop takes statement_or_null (docs/systemverilog/2017/md/section-41-data-read-api.md:2326-2333); statement ::= [ block_identifier : ] { attribute_instance } statement_item (docs/systemverilog/2017/md/section-41-data-read-api.md:2087) and the bare `;` is statement_or_null's own alternative (docs/systemverilog/2017/md/section-41-data-read-api.md:2081-2084); IEEE 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:2311-2319). REJECTS at furthest_position=270 on sv_2017 and sv_2023 (the `;`). Controls (rust/target/s351/rep, release probe 1.0.223 a2ca4c477d54c216…): `for (int i = 0; i < 3; i++) ;` (statement_or_null) and `foreach (y[i]) begin end` ACCEPT on the SV profiles. Pins invalid_foreach_null_statement.sv / control_for_null_statement.sv / control_foreach_loop_variables.sv. the one loop whose body is statement, not statement_or_null; iverilog tolerates the null body"),
    # ---- SV-CORPUS-GRAD.3.56 (PGEN-SV-CORPUS-GRAD-0386): the remaining bar rows bisected ----
    ('Surelog', 'tests/IndexedSelect/dut.sv'): ('must_reject',
        'pinned .3.56: lines 11-13 `for (i = 0; …) begin : genfoo $display("test"); end` - a STATEMENT as a generate item. IEEE 1800-2017 A.4.2 generate_item ::= module_or_generate_item | interface_or_generate_item | checker_or_generate_item (docs/systemverilog/2017/md/section-41-data-read-api.md:1911-1915), A.1.4 module_or_generate_item (docs/systemverilog/2017/md/section-41-data-read-api.md:313-319) - module items, no statement; `initial $display("test");` is the legal spelling. Isolated: rejects @78 at `$display`, the initial form accepts (pins)'),
    ('Surelog', 'tests/TranslateOff/dut.sv'): ('must_reject',
        'pinned .3.56: lines 14-18 `//pragma translate_off` / `ggggll` / `//pragma translate_on` - free text between two comments. IEEE 1800-2017 5.4 (docs/systemverilog/2017/md/section-5-lexical-conventions.md:56-60): a one-line comment ends at its newline, so `ggggll` is source, and A.1.4 module_item (docs/systemverilog/2017/md/section-41-data-read-api.md:327-330) derives no such item; the translate_off pragma is a synthesis-tool comment convention, not a directive. Isolated: rejects @99 at `ggggll` (pins)'),
    # ---- SV-CORPUS-GRAD.3.55 (PGEN-SV-CORPUS-GRAD-0385): the port / generate group bisected to its one variable ----
    ('sv2v', 'test/core/interface_generate.sv'): ('must_reject',
        'pinned .3.55: lines 3-5 `for (genvar i = 0; i < num_clients; ++i) begin : mps modport client_mp (output .client_req(req[i])); end` - a MODPORT inside a generate block. IEEE 1800-2017 A.1.4 interface_or_generate_item ::= { attribute_instance } module_common_item | { attribute_instance } extern_tf_declaration (docs/systemverilog/2017/md/section-41-data-read-api.md:437-441) names no modport_declaration; modport_declaration is a non_port_interface_item (docs/systemverilog/2017/md/section-41-data-read-api.md:455-462), an interface item, never a generate item. Isolated: rejects @85 at `modport`; the modport expression at interface level accepts (pins)'),
    ('sv2v', 'test/core/port_default.sv'): ('must_reject',
        'pinned .3.55: line 8 `input wire integer x = F(P);` - a DEFAULT on a NON-ANSI input port (the file\'s own comment: "defining a default here is non-standard"). IEEE 1800-2017 A.2.1.2 input_declaration ::= input net_port_type list_of_port_identifiers | input variable_port_type list_of_variable_identifiers (docs/systemverilog/2017/md/section-41-data-read-api.md:756-760); A.2.3 list_of_port_identifiers (docs/systemverilog/2017/md/section-41-data-read-api.md:975-978) and list_of_variable_identifiers (docs/systemverilog/2017/md/section-41-data-read-api.md:989-992) carry no initializer - the default value belongs to the ANSI form (A.1.3). Isolated: `module M(x); input wire integer x = 1; endmodule` rejects @35, without the default accepts (pins)'),
    ('sv2v', 'test/core/struct_scope.sv'): ('must_reject',
        'pinned .3.55: lines 20-21 `generate begin : foo … end endgenerate` - a BARE begin/end block directly inside a generate region, the Verilog-2001 form IEEE 1364-2005 removed. IEEE 1800-2017 A.4.2 generate_region ::= generate { generate_item } endgenerate (docs/systemverilog/2017/md/section-41-data-read-api.md:1853-1855), generate_item ::= module_or_generate_item | interface_or_generate_item | checker_or_generate_item (docs/systemverilog/2017/md/section-41-data-read-api.md:1911-1915); a generate_block (docs/systemverilog/2017/md/section-41-data-read-api.md:1903-1908) belongs only to a conditional or loop generate construct. Isolated: `generate begin : foo wire w; end endgenerate` rejects @20, `generate if (1) begin : foo … end endgenerate` accepts (pins)'),
    ('iverilog', 'ivtest/ivltests/module_output_port_sv_var2.v'): ('must_reject',
        'pinned .3.55: lines 8-9 `module test1; output reg a;` - a PORT DECLARATION in the body of a module whose header lists no ports. IEEE 1800-2017 A.1.2 module_declaration ::= module_nonansi_header { module_item } endmodule | module_ansi_header { non_port_module_item } endmodule (docs/systemverilog/2017/md/section-41-data-read-api.md:108-113); module_nonansi_header REQUIRES list_of_ports (docs/systemverilog/2017/md/section-41-data-read-api.md:96-99), module_ansi_header takes [ list_of_port_declarations ] (docs/systemverilog/2017/md/section-41-data-read-api.md:102-105), so `module test1;` is an ANSI header and its body admits non_port_module_item only (docs/systemverilog/2017/md/section-41-data-read-api.md:356-362), which has no port_declaration (module_item does, docs/systemverilog/2017/md/section-41-data-read-api.md:327-330). Isolated: `module test1; output reg a; endmodule` rejects @13, `module test1(a); output reg a; endmodule` accepts (pins)'),
    # ---- SV-CORPUS-GRAD.3.54 (PGEN-SV-CORPUS-GRAD-0384): the class / scope group bisected to its one variable ----
    ('verible', 'verible/verilog/tools/kythe/testdata/program.sv'): ('must_reject',
        "pinned .3.54: lines 36-37 `.clk(intf.clk),` / `);` - a TRAILING COMMA in a named port-connection list. IEEE 1800-2017 A.4.1.1 list_of_port_connections ::= ordered_port_connection { , ordered_port_connection } | named_port_connection { , named_port_connection } (docs/systemverilog/2017/md/section-41-data-read-api.md:1811-1815), named_port_connection ::= { attribute_instance } . port_identifier [ ( [ expression ] ) ] (docs/systemverilog/2017/md/section-41-data-read-api.md:1818-1821) - no empty connection after a comma. "
        "Bisected: the parser stops at 361 (`environment env;`) because the class `environment` is declared in no file of the fixture (the kythe testdata is indexed as one unit per file); with `class environment` planted the parse advances to the comma (rust/target/s354/rep/h2e_program_with_env_planted.sv @921), the row's ONE variable no declaration can supply. Isolated: `memory dut( .clk(clk), );` rejects @102, without the comma accepts (pins)"),
    # ---- SV-CORPUS-GRAD.3.52 (PGEN-SV-CORPUS-GRAD-0382): the residual bar triaged by production, batch two ----
    ('sv2v', 'test/core/interface_array.sv'): ('must_reject',
        "pinned .3.52: line 114 `ModuleASet #(1, 1, 0) xs(intfX[2:0].ModportB);` - a RANGE select inside a hierarchical path. IEEE 1800-2017 A.9.3 hierarchical_identifier ::= [ $root . ] { identifier constant_bit_select . } identifier (docs/systemverilog/2017/md/section-41-data-read-api.md:3361) with A.8.4 constant_bit_select ::= { [ constant_expression ] } (docs/systemverilog/2017/md/section-41-data-read-api.md:3173) - a bit-select, never a range; IEEE 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:3310, :3120). REJECTS at furthest_position=2985 on sv_2017 and sv_2023 (the `:` inside `[2:0]`). Controls (rust/target/s352/rep, release probe 1.0.223 a2ca4c477d54c216…): `xs(intfX[1].ModportB)` ACCEPTS on the SV profiles. Pins invalid_range_select_in_hierarchical_path.sv / control_bit_select_in_hierarchical_path.sv. sv2v's conversion input; a path element takes bit-selects only"),
    ('verible', 'verible/verilog/tools/kythe/testdata/event_control.sv'): ('must_reject',
        "pinned .3.52: line 8 `@x[y];` - an event control on an INDEXED identifier. IEEE 1800-2017 A.6.5 event_control ::= @ hierarchical_event_identifier | @ ( event_expression ) | @* | @ (*) | @ ps_or_hierarchical_sequence_identifier (docs/systemverilog/2017/md/section-41-data-read-api.md:2147-2154) with A.9.3 hierarchical_identifier (docs/systemverilog/2017/md/section-41-data-read-api.md:3361) - a select sits only on a non-final path element; `@(x[y])` is the derivable spelling; IEEE 1800-2023 event_control ::= clocking_event | @ * | @ ( * ) and clocking_event ::= @ ps_identifier | @ hierarchical_identifier | @ ( event_expression ) (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:2140-2151) - the same. REJECTS at furthest_position=154 on sv_2017 and sv_2023 (the `[`). Controls (rust/target/s352/rep, release probe 1.0.223 a2ca4c477d54c216…): `@(x[y]);` ACCEPTS on the SV profiles. Pins invalid_event_control_indexed_identifier.sv / control_event_control_parenthesised_index.sv. verible's indexing fixture; the unparenthesised form takes a hierarchical identifier"),
    ('verible', 'verible/verilog/tools/kythe/testdata/function_port_dimensions.sv'): ('must_reject',
        "pinned .3.52: line 13 `function f4(int [var1:var2] t1 [var3:var4]);` - a PACKED dimension on an integer atom type. IEEE 1800-2017 A.2.2.1 data_type ::= integer_vector_type [ signing ] { packed_dimension } | integer_atom_type [ signing ] | … (docs/systemverilog/2017/md/section-41-data-read-api.md:842-846) with integer_atom_type ::= byte | shortint | int | longint | integer | time (docs/systemverilog/2017/md/section-41-data-read-api.md:888) - the atom types take a signing and nothing else; IEEE 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:825-829, :885). REJECTS at furthest_position=355 on sv_2017 and sv_2023 (the `[` after `int`). Controls (rust/target/s352/rep, release probe 1.0.223 a2ca4c477d54c216…): `function f4(logic [1:0] t1 [3:0]);` and `function f5(int t1 [3:0]);` ACCEPT on the SV profiles. Pins invalid_packed_dimension_on_integer_atom_type.sv / control_packed_dimension_on_vector_type.sv. verible's indexing fixture; an integer atom type takes no packed dimension"),
    ('verible', 'verible/verilog/tools/kythe/testdata/primitive_type_with_dimensions.sv'): ('must_reject',
        "pinned .3.52: line 22 `int [foo_pkg::DATA_WIDTH-1:foo_pkg::DATA_LENGTH] var5 [foo_pkg::DATA_WIDTH-1:foo_pkg::DATA_LENGTH];` - a PACKED dimension on an integer atom type. IEEE 1800-2017 A.2.2.1 data_type ::= integer_vector_type [ signing ] { packed_dimension } | integer_atom_type [ signing ] | … (docs/systemverilog/2017/md/section-41-data-read-api.md:842-846) with integer_atom_type ::= byte | shortint | int | longint | integer | time (docs/systemverilog/2017/md/section-41-data-read-api.md:888) - the atom types take a signing and nothing else; IEEE 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:825-829, :885). REJECTS at furthest_position=622 on sv_2017 and sv_2023 (the `[` after `int`). Controls (rust/target/s352/rep, release probe 1.0.223 a2ca4c477d54c216…): `logic [p::W-1:0] v5 [p::W-1:0];` ACCEPTS on the SV profiles. Pins invalid_packed_dimension_on_integer_atom_type.sv / control_packed_dimension_on_vector_type.sv. verible's indexing fixture; the same spelling as function_port_dimensions.sv"),
    ('Surelog', 'tests/EnumParenType/dut.sv'): ('must_reject',
        'pinned .3.52: line 8 `(states_t) state1;` - a PARENTHESISED type in a variable declaration (the file\'s own comment: "Test parenthesized type variable declaration: (type) varname;"). IEEE 1800-2017 A.2.1.3 data_declaration ::= [ const ] [ var ] [ lifetime ] data_type_or_implicit list_of_variable_decl_assignments ; | … (docs/systemverilog/2017/md/section-41-data-read-api.md:779) with A.2.2.1 data_type (docs/systemverilog/2017/md/section-41-data-read-api.md:842-866) - no alternative parenthesises a type; IEEE 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:756, :825-849). REJECTS at furthest_position=196 on sv_2017 and sv_2023 (the end of the preceding declaration `states_t state;` on line 8 - the `(` of line 9 is the first byte no item derives). Controls (rust/target/s352/rep, release probe 1.0.223 a2ca4c477d54c216…): `states_t state1;` ACCEPTS on the SV profiles. Pins invalid_parenthesised_type_declaration.sv / control_typedef_name_declaration.sv. a Surelog extension by the file\'s own description; no edition derives it'),
    ('Surelog', 'tests/ClassTypeParamAlias/dut.sv'): ('must_reject',
        'pinned .3.52: line 4 `parameter type AT;` - a class-body TYPE parameter without a default. IEEE 1800-2017 Annex A footnote 18: "It shall be legal to omit the constant_param_expression from a param_assignment or the data_type from a type_assignment only within a parameter_port_list" (docs/systemverilog/2017/md/section-41-data-read-api.md:3518-3519) - the .3.37 ruling (footnote 18\'s parameter half) on a type_assignment; IEEE 1800-2023 the same footnote (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:3483-3484). REJECTS at furthest_position=54 on sv_2017 and sv_2023 (the `;`). Controls (rust/target/s352/rep, release probe 1.0.223 a2ca4c477d54c216…): `parameter type AT = logic;` in a class body and `class D #(parameter type BT);` ACCEPT on the SV profiles. Pins invalid_class_body_type_parameter_without_default.sv / control_class_body_type_parameter_default.sv. Surelog tolerates the missing default; footnote 18 confines the omission to a parameter_port_list'),
}

VERIBLE_SYNTAX_MODE_RE = re.compile(r"//\s*verilog_syntax\s*:")

# Standard preprocessor/compiler DIRECTIVES that are NOT text expansion: each one
# steers the compilation (IEEE 1800-2017 clause 22) and leaves the surrounding
# source text parseable exactly as written.
#
# ⛔ `__FILE__ / `__LINE__ ARE NOT IN THIS SET, and their removal is leaf .3.13's
# whole correction. Clause 22 separates two things this allowlist used to conflate:
# a compiler directive steers the compiler, whereas §22.13 defines `__FILE__ and
# `__LINE__ as predefined text MACROS that EXPAND - "`__FILE__ expands to the name
# of the current input file, in the form of a string literal" / "`__LINE__ expands
# to the current input line number, in the form of a simple decimal number"
# (verbatim). The expression around such a name is not parseable until expansion,
# so a file using one carries a genuine preprocessing dependency exactly like a
# user macro - which is what preproc_dependency() must report.
KNOWN_DIRECTIVES = {
    "define", "include", "ifdef", "ifndef", "else", "elsif", "endif", "undef",
    "undefineall", "timescale", "default_nettype", "celldefine", "endcelldefine",
    "resetall", "line", "pragma", "begin_keywords", "end_keywords",
    "unconnected_drive", "nounconnected_drive",
}

META_RE = re.compile(r"^:([a-z_]+):\s*(.*)$")
TICK_RE = re.compile(r"`([A-Za-z_][A-Za-z0-9_$]*)")
INCLUDE_RE = re.compile(r"^\s*`include", re.MULTILINE)
COND_RE = re.compile(r"^\s*`(ifdef|ifndef|elsif)\b", re.MULTILINE)
# IEEE 1800-2017 §34.5: "The data_block and key_block pragma expressions introduce
# the encrypted data or keys" - the lines that FOLLOW such a pragma inside a
# `begin_protected`-`end_protected` envelope are encoded bytes, not source text, and
# §34.3 has the decrypting tool replace "each decryption envelope with the decrypted
# source text from the data_block" before compilation. So a file carrying one is
# opaque to a parser until that stage runs - a preprocessing dependency of the same
# kind as `include, and strictly stronger (it hides arbitrary text, including the
# other dependency markers), which is why preproc_dependency() tests it first.
# `digest_block joins them: §34.5 encodes it identically.
PROTECTED_ENVELOPE_RE = re.compile(
    r"`pragma\s+protect\b[^\r\n]*\b(?:key_block|data_block|digest_block)\b")
FAILS_TRUE_RE = re.compile(r"fails\s*=\s*(True|test\.vlt_all)\b")
TOP_FILENAME_RE = re.compile(r"top_filename\s*=\s*[\"']([^\"']+)[\"']")
SYNTAX_ERR_RE = re.compile(r"syntax error", re.IGNORECASE)

# SV-CORPUS-GRAD.3.24 — verilator's PARSE-STAGE error vocabulary, enumerated.
#
# ⛔ THE HOLE: `SYNTAX_ERR_RE` above is the ONLY thing `VerilatorIndex` used to decide whether a
# `fails=True` test intends a parse-level failure, and verilator does NOT spell every such failure
# "syntax error" — its LEXER has its own message set. Measured across all 1 427 tracked goldens,
# and it split one construct family straight down the middle:
#
#   t_parse_eof_str_bad.v   unterminated "string   golden ALSO says "syntax error"  -> must_reject ✔
#   t_parse_eof_qqq_bad.v   unterminated \"\"\"string  golden says only `EOF in unterminated`  -> must_accept ✘
#   t_parse_eof_attr_bad.v  unterminated `(*`      golden says only `EOF in (*`      -> must_accept ✘
#   t_fuzz_eof_bad.v        both at once           golden ALSO says "syntax error"  -> must_reject ✔
#
# Four files, one lexical class, two verdicts — separated by nothing but whether the upstream
# message happened to contain two particular words. That is the `.3.15` "the pin table already
# contradicts itself" shape, one level down: in the HEURISTIC rather than in the pins.
#
# ⛔⛔ AND A PREFIX RULE WOULD HAVE BEEN WRONG — the `EOF in …` family is NOT uniformly parse-stage.
# The full enumerated vocabulary contains four spellings that belong to OTHER lanes entirely:
#   `EOF in define argument list`, `Unterminated ( in define formal arguments.`,
#   `EOF in unterminated preprocessor expression`   -> the PREPROCESSOR (svpp lane), and
#   `Unterminated /* comment inside -f file.`       -> a `-f` COMMAND FILE, not source text at all.
# Matching `EOF in ` would have swept all four into the parse verdict. So this is an enumerated
# allowlist, exactly as `.3.14b`'s directive whitelist had to be, and for the same reason.
VERILATOR_PARSE_STAGE_RE = re.compile(
    r"syntax error"                                   # the parser proper
    r"|EOF in unterminated (?:\"\"\" )?string"         # 5.9: a string literal running to EOF
    r"|Unterminated string"                            # 5.9, same class, reported at the quote
    r"|EOF in \(\*"                                    # A.9.1: attribute_instance with no `*)`
    r"|EOF in '[^']*' block comment"                   # 5.4: /* … with no */
    r"|Version control conflict marker in file",       # `<<<<<<<` has no lexical derivation at all
    re.IGNORECASE)


# SV-CORPUS-GRAD.13e.3(b) — ivtest's PARSE-STAGE error vocabulary, enumerated, and SPLIT BY
# EDITION because it has to be.
#
# ⛔ THE HOLE: for an ivtest `type: "CE"` row, `SYNTAX_ERR_RE` alone decided whether the failure
# was parse-level, and iverilog spells most parse refusals some other way. Measured over the 394
# ivtest CE rows: 84 carry a usable golden, and only **6** of those say "syntax error".
#
# ⛔⛔ AND A FILE-BASED CLASSIFIER IS NOT ENOUGH EITHER. Mapping each golden message back to the
# source that emits it looks decisive until you read the result: 97 of the messages resolve to a
# "parse-stage" file and **90 of them are `'X' has already been declared in this scope.` out of
# `pform.cc`** — a duplicate declaration, which every edition derives perfectly well. `pform.cc`
# is the parse-time FORM BUILDER and does semantic checks too, so file membership over-admits.
# Only `parse.y` / `lexor.lex` are reliable, and even they need reading one message at a time.
#
# ⭐⭐ THE SPLIT IS THE FINDING. `.3.24` could give verilator ONE flat list because every entry
# there is a LEXICAL fact true in all editions (unterminated string, `EOF in (*`, block comment).
# Several ivtest parse refusals are DIALECT GATES instead — true under the generation iverilog
# was run with, and FALSE under the edition the lane adjudicates. Measured: an unsplit list
# containing `requires SystemVerilog` would have manufactured FOUR false over-acceptance defects
# (`br_gh1143e/f/g/h`, run at `-g2009`, whose null for-loop initialization IEEE 1800-2017 A.6.8
# makes optional) and one more on `br_gh1087a` (`Net data type requires SystemVerilog or
# -gxtypes` — and IEEE 1800 net declarations DO take a data type). ⇒ that whole family is
# EXCLUDED, not merely re-worded: iverilog is reporting the distance to ITS configured
# generation, which is not a statement about the edition PGEN is being judged against.
IVTEST_PARSE_STAGE_RE = re.compile(
    r"syntax error"                                    # the parser proper
    r"|operator is an Icarus Verilog extension"        # parse.y:4162/4174 — binary ~& / ~|,
                                                       #   in NO IEEE edition (A.8.6)
    r"|Empty UDP table\."                              # parse.y — A.5.3 makes the 1st entry
    r"|Invalid table for UDP primitive"                #   mandatory in both editions
    r"|generate/endgenerate regions cannot nest\."     # A.4.2 module_or_generate_item has no
                                                       #   generate_region alternative; only
                                                       #   non_port_module_item does
    r"|is not a valid expression\. Please use operator",  # parse.y — `~ |x`: A.8.3's unary form
                                                       #   is `unary_operator primary`, and `|x`
                                                       #   is an expression, not a primary
    re.IGNORECASE)

# Parse refusals that hold under IEEE 1364-2005 and are RELAXED by IEEE 1800 — usable only when
# the row is being judged as plain Verilog. Keeping these in the shared list is exactly the
# mistake the `requires SystemVerilog` family demonstrates.
IVTEST_PARSE_STAGE_V2005_ONLY_RE = re.compile(
    r"Missing task/function port direction\.",         # 1364-2005 A.2.7: task_port_item is one
                                                       #   of three tf_*_declarations and each
                                                       #   BEGINS with its direction keyword;
                                                       #   IEEE 1800 A.2.7 makes it optional
    re.IGNORECASE)


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
    """Return the strongest svpp dependency flag for a file, or ''.

    ⛔ This is a WHOLE-FILE existence test and that is correct for what it is asked
    below on the `must_reject` path: "could the intended syntax error be hidden until
    after preprocessing?" is genuinely a question about the whole file, and such a row
    may not even have a failure position (it can be the one that wrongly ACCEPTS).

    It is NOT sufficient on the `must_accept` path, where the answer is read as the
    causal claim "this file fails BECAUSE it needs svpp". SV-CORPUS-GRAD.12 measured
    the gap over all 1 459 labelled rows: the two statements agree 80.2 % of the time
    and disagree provably for 26. `svpp_can_explain_failure()` below is the positional
    gate that closes it; see SV-CORPUS-GRAD.12a.
    """
    text = strip_comments_and_strings(raw_text)
    if PROTECTED_ENVELOPE_RE.search(text):
        return "protected_envelope"
    if INCLUDE_RE.search(text):
        return "include"
    for name in TICK_RE.findall(text):
        if name not in KNOWN_DIRECTIVES:
            return "macro_use"
    if COND_RE.search(text):
        return "conditional"
    return ""


# ---------------------------------------------------------------------------
# SV-CORPUS-GRAD.12a — the positional gate on `explained_svpp_*`
# ---------------------------------------------------------------------------
# Compiler directives svpp hands through to the parser UNCHANGED.  SVPP-EXPANSION is
# scoped to three operations - macro substitution, conditional resolution, `include
# inlining (docs/tasks/SVPP-EXPANSION.md) - so these move no byte, and a parse stopped
# ON one of them is not waiting for the preprocessor lane at all.  `line and `pragma are
# deliberately absent: `line is preprocessor-adjacent but not one of the three scoped
# operations, `pragma protect belongs to the §34 envelope lane, and treating an unclear
# case as ALTERABLE is the choice that cannot manufacture a false reclassification.
SVPP_PASSTHROUGH = {
    "timescale", "default_nettype", "celldefine", "endcelldefine", "resetall",
    "begin_keywords", "end_keywords", "unconnected_drive", "nounconnected_drive",
}
ANY_TICK_RE = re.compile(r"`[A-Za-z_][A-Za-z0-9_$]*")


def blank_comments_and_strings(text: str) -> str:
    """Blank comments/strings IN PLACE - length- and newline-preserving.

    ⛔ Deliberately NOT `strip_comments_and_strings()`, which collapses each match to a
    single space.  Collapsing is fine for "is there one?" and silently wrong for
    "where is it?": every offset after the first comment shifts.
    """
    def repl(m):
        return "".join("\n" if ch == "\n" else " " for ch in m.group(0))
    return COMMENT_STRING_RE.sub(repl, text)


DEFINE_LINE_RE = re.compile(r"^[ \t]*`define\b")


def inside_define_continuation(stripped: str, offset: int) -> bool:
    """SV-CORPUS-GRAD.3.39 — does `offset` sit on a CONTINUATION line of a `define?

    IEEE 1800-2017 22.5.1: "If more than one line is necessary to specify the text, the
    newline shall be preceded by a backslash". Those lines are macro TEXT; an isolation
    parse that stops on one is stopped inside the preprocessor's business no matter which
    token it stopped on — since 1.0.217 the parser refuses `default_nettype t \\ there
    (the directive's argument is a macro FORMAL), which is correct and still svpp's.
    `stripped` is the length-preserving blanked text, so offsets are the parser's own.
    """
    pos = 0
    in_span = False
    for line in stripped.split("\n"):
        end = pos + len(line)
        if in_span:
            if pos <= offset <= end:
                return True
            in_span = line.rstrip().endswith("\\")
        elif DEFINE_LINE_RE.match(line) and line.rstrip().endswith("\\"):
            in_span = True
        pos = end + 1
    return False


def first_alterable_tick(stripped: str):
    """Byte offset of the first backtick token svpp can ALTER, or None.

    Below this offset a preprocessor's output is byte-identical to its input, so a
    parse that stops there stops for a reason expansion cannot reach.
    """
    for m in ANY_TICK_RE.finditer(stripped):
        if TICK_RE.match(stripped, m.start()).group(1) not in SVPP_PASSTHROUGH:
            return m.start()
    return None


REPO_ROOT = Path(__file__).resolve().parent.parent.parent

# The instrument-identity table `stimuli/run_external_corpus.sh` writes into every characterization
# report: `| <label> | ` + "`" + `<repo-root-relative path>` + "`" + ` | ` + "`" + `<sha256>` + "`" + ` |`.
IDENTITY_ROW_RE = re.compile(r"^\|\s*([^|]+?)\s*\|\s*`([^`]+)`\s*\|\s*`([0-9a-f]{64})`\s*\|\s*$")
# A row whose LABEL says it is a SEMANTIC digest (SV-CORPUS-GRAD.13r's `grammar raw ast`) is a
# digest of the frontend's envelope, not of the file's bytes — it can never vouch for a dump and
# must not be confused with the byte row that shares its path.
SEMANTIC_LABEL_RE = re.compile(r"\braw[ _]ast\b", re.I)


def sha256_of(path: Path) -> str:
    """The digest of a file, streamed — these dumps run to 1.4 MB and there is no reason to slurp."""
    h = hashlib.sha256()
    with path.open("rb") as fh:
        for chunk in iter(lambda: fh.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def verify_outcome_identity(report: Path, dumps, *, rerun: str) -> None:
    """REFUSE when a raw dump is not the one the TRACKED report describes (SV-CORPUS-GRAD.13e.6).

    ⛔ WHY THIS EXISTS, MEASURED. `results.tsv` / `positions.tsv` are `.gitignore`d — deliberately,
    they are 1.4 MB of regenerable intermediate — so they carry NEITHER an identity block NOR a
    history: nothing on disk can say when a copy was produced, against which parser, or that it has
    drifted. And they are the INPUT to every number this campaign publishes. `.13e.5` measured the
    consequence: one SV-lane row (`t_property_unsup.v`) had been `fail` for THREE DAYS after
    `SV-0066` fixed the construct, so the axis-2 bar was counting a defect that no longer existed
    (275 -> 274). A stale dump that hides a FIX inflates the bar and the campaign works on phantoms;
    one that hides a REGRESSION deflates it and the campaign ships. Both are silent.

    ⭐ THE IDENTITY TRANSFERS THROUGH THE TRACKED FILE. An identity block on a DERIVATION cannot say
    that its own input was measured against yesterday's parser — so the runner now digests the dumps
    it just wrote into the report's own identity table, and this refuses when the copy on disk is a
    different one. The report says WHICH parser; the digest says the dump is that parser's.

    ⛔ IT IS EXACT, NOT A RECONCILIATION. Holding the report's per-sub-corpus verdict COUNTS equal to
    the dump would look equivalent and is not: two files swapping verdicts inside one sub-corpus
    CANCEL, which is the multiset trap `PGEN-ENGINE-UNIVERSAL-SERVICES-0160` paid for and re-keyed
    onto the PATH. A digest has no such hole.

    A report with no output row is reported NOT EVALUATED, loudly, and never as a pass: an older
    tree adjudicates exactly as it did before, and says so.
    """
    if not report.is_file():
        print(f"outcome-identity: NOT EVALUATED - {_rel(report)} is absent, so nothing names the "
              f"runs these dumps came from", file=sys.stderr)
        return
    recorded = {}
    for line in report.read_text(encoding="utf-8").splitlines():
        m = IDENTITY_ROW_RE.match(line)
        if not m:
            continue
        label, path, digest = m.group(1), m.group(2), m.group(3)
        # ⛔ ONLY BYTE-KIND ROWS ARE CANDIDATES. `.13r` added a `grammar raw ast` row that shares its
        # PATH with the byte `grammar` row and legitimately carries a different digest (the envelope,
        # not the bytes). Keying by path alone made the duplicate-row refusal below fire on the first
        # report that carried both — measured on the `-0340` re-stamp: the adjudicator refused a
        # correct report, so the two slices were each right alone and wrong together. A semantic row
        # can never vouch for a dump, so it is not read here at all.
        if SEMANTIC_LABEL_RE.search(label):
            continue
        # ⛔ A DUPLICATE ROW REFUSES RATHER THAN LETTING THE LAST ONE WIN, and this is not a
        # hypothetical: the bank's end-to-end arm built its fixture by APPENDING a wrong-digest row,
        # which passed silently the moment the real report grew a row of its own — a dict assignment
        # took the second one and the arm went green on a stale dump. A table carrying two BYTE digests
        # for one file cannot vouch for anything, so it is an error, not a precedence question.
        if path in recorded and recorded[path] != digest:
            raise SystemExit(
                f"outcome-identity: {_rel(report)} records TWO different digests for `{path}` "
                f"({recorded[path][:12]}... and {digest[:12]}...). An identity table that "
                f"disagrees with itself cannot vouch for anything - fix the report.")
        recorded[path] = digest
    checked = []
    for dump in dumps:
        rel = _rel(dump)
        want = recorded.get(rel)
        if want is None:
            print(f"outcome-identity: NOT EVALUATED - {_rel(report)} records no digest for {rel}, "
                  f"so nothing can say this dump came from the run that report describes "
                  f"(SV-CORPUS-GRAD.13e.6; re-run `{rerun}` to stamp one)", file=sys.stderr)
            continue
        if not dump.is_file():
            # An absent dump is the caller's own business - `--positions` is optional by design and
            # `--results-v2005` gates a whole lane. Only a PRESENT dump can be the WRONG dump.
            continue
        got = sha256_of(dump)
        if got != want:
            raise SystemExit(
                f"outcome-identity: {rel} is NOT the dump {_rel(report)} describes "
                f"(report {want[:12]}..., disk {got[:12]}...). Every number this adjudication "
                f"publishes would be about a different parse than the tracked report claims. "
                f"Re-run the lane: `{rerun}`")
        checked.append(rel)
    if checked:
        print(f"outcome-identity: OK - {', '.join(checked)} re-hash equal to {_rel(report)}",
              file=sys.stderr)


def _rel(path: Path) -> str:
    """Repo-root-relative, whatever the caller passed (the repo-relative-paths policy)."""
    try:
        return str(Path(path).resolve().relative_to(REPO_ROOT))
    except ValueError:
        return str(path)


def load_positions(path: Path, label: str):
    """results.tsv column-3 path -> `furthest_position`, from the runner's sidecar.

    Keyed by the RAW column-3 spelling on purpose: the caller re-keys it through the
    same path->suite-relative mapping it already performs for the results rows, so the
    two can never disagree about a spelling (the failure mode CORPUS-GRAD-ALL.2.1's
    absolute->relative switch actually produced).

    Absent file => empty map => `svpp_can_explain_failure()` answers True everywhere
    and the adjudication is byte-identical to the pre-.12a behaviour.
    """
    out = {}
    if not path.is_file():
        print(f"adjudicate: no {label} sidecar at {path} - the svpp positional gate "
              "(SV-CORPUS-GRAD.12a) is INACTIVE for this run; re-run "
              "stimuli/run_external_corpus.sh to emit it", file=sys.stderr)
        return out
    for line in path.read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        _suite, relpath, pos = line.split("\t")
        out[relpath] = int(pos)
    return out


def svpp_can_explain_failure(raw_text: str, furthest):
    """Could running svpp plausibly change THIS parse failure?

    `furthest` is the probe's `furthest_position` - the deepest byte any branch
    CONSUMED, not the offending token's offset, so the layout gap between the two must
    be skipped before anything is compared.  Getting this backwards is not theoretical:
    the first cut of the SV-CORPUS-GRAD.12 audit compared them directly and reported
    504 of 1 459 rows misclassified, of which the true answer was 26.  See
    docs/knowledge/a-furthest-position-names-a-region-not-a-token.md.

    Answers True whenever the evidence is absent or undecidable - the gate only ever
    REMOVES a label it can disprove.
    """
    if furthest is None:
        return True                       # no position banked - keep today's answer
    stripped = blank_comments_and_strings(raw_text)
    stuck = None
    for i in range(min(furthest, len(stripped)), len(stripped)):
        if not stripped[i].isspace():
            stuck = i
            break
    if stuck is None:
        return True                       # consumed everything; a resolved `ifdef arm
                                          # could still supply the missing tail
    if inside_define_continuation(stripped, stuck):
        # SV-CORPUS-GRAD.3.39: the stop is on a CONTINUATION line of a `define — macro TEXT the
        # isolation parse reads as source (22.5.1: a macro text spanning lines ends each line but
        # the last with a backslash). Whatever token sits there, the preprocessor owns it.
        # Measured: sv2v test/core/default_nettype.sv's `default_nettype t \ inside `define TEST(t).
        return True
    m = TICK_RE.match(stripped, stuck)
    if m:
        # Stopped exactly ON a directive: svpp explains it unless it is one svpp
        # hands straight back.
        if m.group(1) not in SVPP_PASSTHROUGH:
            return True
        # SV-CORPUS-GRAD.3.39: since release 1.0.217 the parser reads a pass-through directive's
        # ARGUMENTS (`timescale time_unit / time_precision, ...), so a stop ON the directive can be
        # a stop on an argument — and an argument that is a text-macro use is text svpp alters.
        # Measured: verilator t_time_vpi.v `timescale `time_scale_units / `time_scale_prec.
        line_end = stripped.find("\n", stuck)
        if line_end < 0:
            line_end = len(stripped)
        return any(t.group(1) not in KNOWN_DIRECTIVES
                   for t in TICK_RE.finditer(stripped, m.end(), line_end))
    first = first_alterable_tick(stripped)
    if first is None:
        return False                      # label claims a dependency the file lacks
    return stuck >= first


def reject_arm_demotion(dep_flag: str, *, position_refutes_dependency: bool = False):
    """The `must_reject` arm's use of the preprocessor dependency — a WHOLE-FILE question,
    refutable by the CALLER on one narrow, measured ground.

    ⛔ SV-CORPUS-GRAD.12c.3 (finding F3).  `preproc_dependency()` feeds TWO callers that ask
    DIFFERENT questions, and `.12a` had to keep them apart BY HAND:

      must_accept + observed fail -> "did this file fail BECAUSE it needs the preprocessor?"
                                     a question about WHERE the parse stopped => POSITIONAL,
                                     answered by `svpp_can_explain_failure()`.
      must_reject                 -> "could the intended syntax error live behind an `include
                                     or a macro, so that isolation parse cannot testify?"
                                     a question about the FILE => an EXISTENCE test, and there
                                     may be no failure position at all (the row can be the one
                                     that wrongly ACCEPTS).

    Nothing mechanical held that apart.  A future edit making the predicate positional wholesale
    would silently corrupt this arm — every `must_reject` row with no banked position would stop
    being demoted and would re-enter the burn-down as a defect it is not.

    ⭐ This function IS the guard, and it is structural rather than a test: **it can obtain no
    position of its own.**  Making this arm positional requires changing a signature, which is
    visible in review; before, it required changing one shared helper, which was not.  The runtime
    controls in `_self_check_arm_split()` cover the rest.

    ⛔⛔ SV-CORPUS-GRAD.13s — THE ONE NARROW REFUTATION, and why it does not reopen what F3 closed.
    `position_refutes_dependency` is a DECIDED BOOLEAN the caller computes as
    `observed == "fail" and not svpp_can_explain_failure(text, banked_position)`; this function is
    still unable to derive it, mis-derive it, or obtain a position.  F3's feared case is the row
    with NO banked position, and it is structurally impossible here: `svpp_can_explain_failure`
    answers True whenever the evidence is absent or undecidable, so an unbanked row yields False and
    is demoted exactly as before.  What the refutation removes is one measured contradiction —
    `.13k`'s `verilator/test_regress/t/t_gate_basic.v` stops at byte 530 (line 25, `nand #(2,3,4)`)
    while the file's first backtick-ifdef is at line 41, so the demoted basis published the sentence
    *"the parse stops at line 25, above the file's first ifdef — BUT macro/conditional-dependent:
    the reject expectation is only meaningful post-preprocessing"*.  Population measured before the
    change: 2 rows demoted on `macro_use`/`conditional` in the main lane, exactly 1 refutable.

    ⛔ THE `include` ARM IS DELIBERATELY NOT REFUTABLE.  Its argument is not positional: the
    intended-bad content may live in the INCLUDED file, so the unit can be valid standalone and
    invalid only at chain level (leaf .4).  A failure position above the first backtick says nothing
    about that, and treating an unclear case as chain-level is the choice that cannot manufacture a
    false reclassification — the same asymmetry `SVPP_PASSTHROUGH` is built on.

    Returns `(new_expected, basis_suffix)` or `None` when the row keeps its `must_reject`.
    """
    if dep_flag == "include":
        # The intended-bad content may live in the `include'd file, which isolation parse never
        # sees - the reject expectation is only meaningful at chain level (leaf .4).
        return ("chained_only",
                " - but `include-dependent: reject expectation is chain-level")
    if dep_flag in ("macro_use", "conditional"):
        if position_refutes_dependency:
            # SV-CORPUS-GRAD.13s: the file DOES carry the construct and the parse provably
            # stopped where expansion cannot reach, so the dependency is real and irrelevant to
            # THIS reject. Keeping `must_reject` states the parser is correct WITH A CITE
            # instead of deferring a reject nothing in the svpp lane will ever explain.
            return None
        # Mirror of the include rule (leaf .8b.1): the intended syntax error may only
        # materialize after macro expansion / conditional resolution, which isolation parse of
        # the raw text never performs - a raw-text reject would testify for the wrong reason.
        return ("out_of_scope_with_cause",
                " - but macro/conditional-dependent: the reject "
                "expectation is only meaningful post-preprocessing "
                "(svpp lane)")
    return None


# --- SV-CORPUS-GRAD.13c.1: rows that are not SystemVerilog source text at all -----------------
# ⛔ Membership is a property of the BYTES. A list of paths would be wrong the day a corpus adds a
# 347th memory image; an extension test is exactly what let 346 `.v` files into a parse census.
# Each shape is stated so a reader can refute it: what the file contains, and why that has no
# derivation as `source_text` (IEEE 1800-2017 §A.1.2).
#   readmemh_image  — §21.4 (docs/systemverilog/2017/md/section-21-input-output-system-tasks-and-
#                     system-functions.md:1305-1322): "The text file to be read shall contain only
#                     ... White space ... Comments ... Binary or hexadecimal numbers", with `x/X`,
#                     `z/Z`, `_` allowed in a number and addresses written `@hh...h`.
#   string_literal_only — nothing outside string literals and comments; no `description` derives it.
#   tool_command_line   — every non-blank line is a run of `-`/`+` options (a Surelog `.sl` body
#                         saved with a `.sv` extension).
# CONSERVATIVE BY DESIGN: a shape the detector cannot decide is left as it was (visible under its old
# class), never guessed into this one — an empty or comment-only file IS a compilation unit with zero
# descriptions and is source text.
_NS_COMMENT_RE = re.compile(r"/\*.*?\*/|//[^\n]*", re.S)
_NS_READMEM_TOKEN_RE = re.compile(r"^(@[0-9a-fA-F]+|[0-9a-fA-F_xXzZ]+)$")
_NS_STRING_LITERAL_RE = re.compile(r'"(?:[^"\\\n]|\\.)*"')
NOT_SOURCE_SHAPES = {
    "readmemh_image":
        "a $readmemh memory image - only white space, comments and hexadecimal numbers, with "
        "optional @hh...h address lines (IEEE 1800-2017 §21.4)",
    "string_literal_only":
        "a bare string literal - nothing outside string literals and comments, so no "
        "description (IEEE 1800-2017 §A.1.2 source_text) can derive it",
    "tool_command_line":
        "a tool command line - every non-blank line is a run of -/+ options, not "
        "SystemVerilog",
}


def not_source_text(text: str):
    """The shape slug when `text` is decidably not SystemVerilog source text, else None."""
    body = _NS_COMMENT_RE.sub(" ", text)
    if not body.strip():
        return None
    toks = body.split()
    if all(_NS_READMEM_TOKEN_RE.match(t) for t in toks) and (
            any(t.startswith("@") for t in toks)
            or all(any(ch.isdigit() for ch in t) for t in toks)):
        return "readmemh_image"
    if _NS_STRING_LITERAL_RE.search(body) and not _NS_STRING_LITERAL_RE.sub(" ", body).strip():
        return "string_literal_only"
    lines = [ln for ln in body.splitlines() if ln.strip()]
    if lines and not re.search(r"[;(){}]", body) and all(ln.lstrip()[0] in "-+" for ln in lines):
        return "tool_command_line"
    return None


def apply_not_source_text(suite: str, rel: str, text: str, expected: str, basis: str):
    """The ONE decision both lanes make after the answer key has spoken.

    A key OUTRANKS a shape in the direction that is a verdict: `must_reject` on non-source text is
    a correct expectation and stays. `must_accept` on non-source text is a contradiction nobody can
    adjudicate around, so the run REFUSES and names the row - fix the key. Every unkeyed row
    (chained_only / out_of_scope) whose bytes are not source text becomes `not_source_text`.
    """
    shape = not_source_text(text)
    if shape is None or expected == "must_reject":
        return expected, basis
    if expected == "must_accept":
        raise SystemExit(
            f"⛔ REFUSING: {suite}/{rel} is keyed must_accept but its bytes are not SystemVerilog "
            f"source text ({shape}: {NOT_SOURCE_SHAPES[shape]}). A key that expects that to parse is "
            "a broken key - fix the KEY, not the detector (SV-CORPUS-GRAD.13c.1)")
    return ("not_source_text",
            f"not SystemVerilog source text: {NOT_SOURCE_SHAPES[shape]} (SV-CORPUS-GRAD.13c.1)")


def _self_check_not_source_text():
    """Ground truth for the not-source decision, re-run on every invocation (microseconds).

    Both directions: every shape has positives that MUST classify, and a negative list of things
    that look degenerate but ARE source text (empty, comment-only, a `define pair, an include
    fragment, an enum-member list) or that a lazy detector would swallow (`abc`, `deadbeef cafe`,
    an unterminated literal, `-1;`). Then the decision itself, all four arms.
    """
    positives = {
        "readmemh_image": ["@00010000 93 00 00 00\n13 01 00 00\n", "00000000\n00000000\n",
                           "// boot image\n@0 ff_ff xx zz 1\n/* end */\n"],
        "string_literal_only": ['// Just a test string\n"test string"\n', '"system stuff!"',
                                '"a" "b"\n/* c */\n'],
        "tool_command_line": ["-parse -d uhdm -d coveruhdm -elabuhdm -d ast dut.sv -nobuiltin\n",
                              "+incdir+../rtl\n-f files.f\n"],
    }
    negatives = ["", "\n\n", "// only a comment\n", "/* block */\n",
                 "`define BAR `__FILE__\n`define FOO `BAR\n", '`include "macro.svh"\n',
                 "module m; endmodule\n", "CUSTOM_1,\n", "localparam X = 8'hFF;\n", "abc\n",
                 "deadbeef cafe\n", '"unterminated\n', "-1;\n", "wire w;\n", "timeunit 1ns;\n"]
    for shape, texts in positives.items():
        for t in texts:
            got = not_source_text(t)
            if got != shape:
                raise SystemExit(f"adjudicator MISCALIBRATED: not_source_text({t!r}) = {got!r}, "
                                 f"want {shape!r} (SV-CORPUS-GRAD.13c.1)")
    for t in negatives:
        got = not_source_text(t)
        if got is not None:
            raise SystemExit(f"adjudicator MISCALIBRATED: not_source_text({t!r}) = {got!r} on "
                             "SOURCE TEXT - the detector must never widen past its stated shapes "
                             "(SV-CORPUS-GRAD.13c.1)")
    img = "@0 ff 00\n"
    refused = None
    try:
        apply_not_source_text("s", "r.v", img, "must_accept", "k")
    except SystemExit as e:
        refused = str(e)
    if not refused or "broken key" not in refused:
        raise SystemExit("adjudicator MISCALIBRATED: must_accept on a memory image did not REFUSE "
                         "with the broken-key text (SV-CORPUS-GRAD.13c.1)")
    if apply_not_source_text("s", "r.v", img, "must_reject", "k") != ("must_reject", "k"):
        raise SystemExit("adjudicator MISCALIBRATED: a must_reject key must OUTRANK the shape")
    if apply_not_source_text("s", "r.v", img, "chained_only", "k")[0] != "not_source_text":
        raise SystemExit("adjudicator MISCALIBRATED: an unkeyed memory image must become "
                         "not_source_text")
    if apply_not_source_text("s", "r.v", "module m; endmodule\n", "chained_only", "k") != (
            "chained_only", "k"):
        raise SystemExit("adjudicator MISCALIBRATED: source text must pass through untouched")


def _self_check_arm_split():
    """Ground truth for the two-caller split, re-run on every invocation (microseconds).

    An instrument with no ground truth is a confident guess
    (docs/decisions/feedback_instrument_needs_ground_truth.md).  These controls assert the
    property `.12a` established by hand and `.12c.3` makes mechanical, in BOTH directions.
    """
    import inspect

    # (1) STRUCTURAL: the reject arm still cannot OBTAIN a position. `.13s` gave it one
    #     DECIDED BOOLEAN and nothing else, exactly as F3 prescribed — a changed signature,
    #     visible in review. Pinning the parameter list keeps the next widening equally visible:
    #     a raw position, a text argument or a positions map appearing here fails this control.
    reject_params = list(inspect.signature(reject_arm_demotion).parameters)
    if reject_params != ["dep_flag", "position_refutes_dependency"]:
        raise SystemExit(
            "adjudicator MISCALIBRATED: reject_arm_demotion() takes "
            f"{reject_params!r}; the must_reject arm is a WHOLE-FILE existence test that may "
            "receive only the caller's DECIDED refutation boolean, never a failure position or "
            "the text to derive one from (SV-CORPUS-GRAD.12c.3 F3, narrowed by .13s)")

    # (2) BEHAVIOURAL: a must_reject row is demoted on the DEPENDENCY ALONE - no position exists
    #     for it and none is needed. ⛔ This is F3's own case and it must never move: an
    #     unrefuted row (which includes EVERY row with no banked position, because
    #     `svpp_can_explain_failure` answers True when the evidence is absent) demotes as before.
    for flag, want in (("include", "chained_only"),
                       ("macro_use", "out_of_scope_with_cause"),
                       ("conditional", "out_of_scope_with_cause"),
                       ("", None),
                       ("protected", None)):
        got = reject_arm_demotion(flag)
        got_class = got[0] if got else None
        if got_class != want:
            raise SystemExit(
                f"adjudicator MISCALIBRATED: reject_arm_demotion({flag!r}) -> {got_class!r}, "
                f"expected {want!r} (SV-CORPUS-GRAD.12c.3 F3)")

    # (2b) SV-CORPUS-GRAD.13s - THE REFUTATION, TWO-SIDED. `.13k`'s fourth row is the positive
    #      side and the `include` arm is the negative one: an include demotion is NOT positional
    #      (the intended-bad content may live in the included FILE), so it must survive a
    #      refutation that a macro/conditional demotion yields to.
    for flag, refuted, want in (("macro_use", True, None),
                                ("conditional", True, None),
                                ("include", True, "chained_only"),
                                ("", True, None),
                                ("protected", True, None),
                                ("macro_use", False, "out_of_scope_with_cause"),
                                ("conditional", False, "out_of_scope_with_cause")):
        got = reject_arm_demotion(flag, position_refutes_dependency=refuted)
        got_class = got[0] if got else None
        if got_class != want:
            raise SystemExit(
                f"adjudicator MISCALIBRATED: reject_arm_demotion({flag!r}, "
                f"position_refutes_dependency={refuted!r}) -> {got_class!r}, expected {want!r} "
                "(SV-CORPUS-GRAD.13s)")

    # (3) THE OTHER ARM REALLY IS POSITIONAL - the control that fails if someone "unifies" the
    #     two by making the accept arm whole-file again.  ONE text, TWO positions, opposite
    #     answers: a whole-file predicate cannot produce them.
    #
    #     ⛔ The tick is deliberately placed LAST.  The first draft of this control put the
    #     `define first and probed the end of the text - which lands on a trailing newline, so
    #     `stuck is None` ("consumed everything") returned True for the wrong reason and the
    #     control fired against correct code.  A control that fails for a reason other than the
    #     one it names is worse than no control; the probe is built so the two answers can only
    #     differ positionally.  With the tick last, `stuck < first_alterable_tick` is reachable.
    probe = "module m;\n  BADTOKEN\nendmodule\n`define W 8\n"
    tick_at = probe.index("`define")
    if svpp_can_explain_failure(probe, 0):
        raise SystemExit("adjudicator MISCALIBRATED: a parse stopping BEFORE every alterable "
                         "tick is not svpp-explainable (SV-CORPUS-GRAD.12c.3 F3)")
    if not svpp_can_explain_failure(probe, tick_at):
        raise SystemExit("adjudicator MISCALIBRATED: a parse stopping ON a `define must be "
                         "svpp-explainable (SV-CORPUS-GRAD.12c.3 F3)")

    # (3b) SV-CORPUS-GRAD.3.39 — the two refinements, each in BOTH directions. A stop on a
    #      `define CONTINUATION line is svpp-explainable whatever token sits there; a stop ON a
    #      pass-through directive is explainable exactly when one of its ARGUMENTS is a text-macro
    #      use, and stays unexplained on plain arguments (the 1.0.217 parser refuses those itself).
    probe2 = "`define T(t) \\\n    `default_nettype t \\\n    module m; endmodule\n`default_nettype none\n"
    if not svpp_can_explain_failure(probe2, probe2.index("`default_nettype t")):
        raise SystemExit("adjudicator MISCALIBRATED: a parse stopping on a `define CONTINUATION "
                         "line must be svpp-explainable (SV-CORPUS-GRAD.3.39)")
    if svpp_can_explain_failure(probe2, probe2.index("`default_nettype none")):
        raise SystemExit("adjudicator MISCALIBRATED: a parse stopping ON a pass-through directive "
                         "with plain arguments is NOT svpp-explainable (SV-CORPUS-GRAD.3.39)")
    probe3 = "`timescale `unit / `prec\nmodule m; endmodule\n"
    if not svpp_can_explain_failure(probe3, 0):
        raise SystemExit("adjudicator MISCALIBRATED: a pass-through directive whose ARGUMENT is a "
                         "text-macro use must be svpp-explainable (SV-CORPUS-GRAD.3.39)")
    if not verilog_ams_keyword_version_only("`begin_keywords \"VAMS-2.3\"\nmodule m; endmodule\n`end_keywords\n"):
        raise SystemExit("adjudicator MISCALIBRATED: a file whose every `begin_keywords names a "
                         "Verilog-AMS version is Verilog-AMS input (SV-CORPUS-GRAD.3.39)")
    if verilog_ams_keyword_version_only("`begin_keywords \"1364-1995\"\n`end_keywords\n`begin_keywords \"1800+VAMS\"\n`end_keywords\n"):
        raise SystemExit("adjudicator MISCALIBRATED: a keyword-region test mixing IEEE and AMS "
                         "versions is NOT Verilog-AMS input (SV-CORPUS-GRAD.3.39)")
    if verilog_ams_keyword_version_only("// `begin_keywords \"VAMS-2.3\"\nmodule m; endmodule\n"):
        raise SystemExit("adjudicator MISCALIBRATED: a commented-out AMS version directive is not "
                         "a directive (SV-CORPUS-GRAD.3.39)")

    # (4) AND IT STILL DEFAULTS TO KEEPING THE LABEL when there is no position at all - the
    #     gate only ever REMOVES a label it can disprove.  Note this is the SAME text that
    #     answers False at position 0: without a position the answer flips to True, which is
    #     exactly the asymmetry the `must_reject` arm relies on.
    if not svpp_can_explain_failure(probe, None):
        raise SystemExit("adjudicator MISCALIBRATED: with no banked position the accept arm must "
                         "keep today's answer (SV-CORPUS-GRAD.12c.3 F3)")


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


# SV-CORPUS-GRAD.3.14a/.3.14b — the SPEC outranks the TOOL.
#
# `VerilatorIndex.expect()` below derives its answer key from verilator's own driver
# conventions: `fails=True` + no "syntax error" in the golden `.out` => the syntax is
# valid => `must_accept`. That heuristic is upstream TOOL testimony, and for these three
# files it is wrong as a CONFORMANCE statement: each places `` `default_nettype `` INSIDE
# a module body, which both normative clauses forbid outright —
#
#   "It can be used only outside design elements."
#     — IEEE 1800-2017 §22.8  (docs/systemverilog/2017/md/section-22-compiler-directives.md)
#   "It can be used only outside of module definitions."
#     — IEEE 1364-2005 §19.2  (docs/verilog/2005/md/section-19-compiler-directives.md)
#
# Verilator TOLERATING the construct is a real-world datum, not a conformance argument.
# The corpus doctrine ranks the LRM above the tool, and these pin tables
# (`ISPRAS_NEGATIVE_PINNED`, `IVTEST_CE_STAGE_PINNED`, …) are the standing mechanism for
# exactly that. Pinning them `must_reject` makes today's REJECT a `match` — the parser is
# claimed CORRECT WITH A CITE rather than the rows being quietly deferred.
#
# ⛔ These three are the reason `.3.14b`'s grammar fix is a NAME WHITELIST and not a
# blanket directive-tolerance rule: a blanket rule would have made them PASS, converting
# a correct reject into an over-acceptance defect
# (feedback_sv_strict_lrm_compliance_default).
VERILATOR_PINNED = {
    # ---- SV-CORPUS-GRAD.13k(a)+(c), landed by .13s (PGEN-SV-CORPUS-GRAD-0328) --------------
    # The first MAIN-lane member of `.13k`'s answer-key class, routed here by `.13p`
    # (PGEN-SV-CORPUS-GRAD-0322) when the delay2 split made the parser stop on the construct.
    # Same shape as the three v2005 compound-assignment rows: a `must_accept` resting on a
    # DRIVER's expectation, refuted by the standard. Ruled `must_reject` on the taxonomy at the
    # head of this module - text a vendor tolerates that no edition can derive, where the spec
    # outranks the upstream tool.
    #
    # ⛔ THIS PIN WAS APPLIED, MEASURED AND REVERTED ONCE (`-0327`) BEFORE IT COULD LAND. The
    # whole-file `reject_arm_demotion` sent it to `deferred:svpp_owned` on the file's line-41
    # backtick-ifdef, composing a basis that contradicted itself in one string. `.13s` gave that
    # arm the caller's decided refutation boolean, so a reject provably above every alterable
    # tick now keeps its `must_reject`.
    "test_regress/t/t_gate_basic.v": ("must_reject",
        "pinned .13k(a): line 25 `nand #(2,3,4) ND0 (nd0, a[0], b[0], b[1]);` puts THREE delay "
        "values on an n-input gate. IEEE 1800-2017 28.4 "
        "(docs/systemverilog/2017/md/section-28-gate-level-and-switch-level-modeling.md:396) and "
        "IEEE 1364-2005 7.2 "
        "(docs/verilog/2005/md/section-7-gate-and-switch-level-modeling.md:376): 'The delay "
        "specification shall be zero, one, or two delays' for and/nand/nor/or/xor/xnor; Annex A "
        "A.3.1 spells `n_input_gatetype [ drive_strength ] [ delay2 ]`, which carries at most two "
        "mintypmax_expressions. Three delays belong to the ENABLE gates and switches (1800-2017 "
        "28.6, 1364-2005 7.4, Annex A `[ delay3 ]`). REJECTS on ALL THREE profiles since release "
        "1.0.204 / ledger SV-0077, at furthest_position=530 - exactly the second comma. Controls: "
        "`nand #(2,3) ND0 (...)` ACCEPTS on all three profiles (the narrowing is the THIRD value "
        "alone) and `bufif0 #(1,2,3)` ACCEPTS (an enable gate takes three). Spec outranks the "
        "verilator driver key 't_gate_basic.py expects success', which records what verilator "
        "tolerates. The preprocessor does not explain this reject and .13s now proves it "
        "mechanically: the parse stops at byte 530, line 25, while the file's first alterable "
        "backtick token is the line-41 ifdef"),
    "test_regress/t/t_lint_implicit_def_bad.v": ("must_reject",
        "pinned .3.14a: `default_nettype none/wire inside module t (lines 17, 20) - "
        "IEEE 1800-2017 22.8 / IEEE 1364-2005 19.2 permit it only OUTSIDE design "
        "elements; spec outranks the verilator driver heuristic"),
    "test_regress/t/t_lint_implicit_func_bad.v": ("must_reject",
        "pinned .3.14a: `default_nettype wire inside module t (line 11) - "
        "IEEE 1800-2017 22.8 / IEEE 1364-2005 19.2 permit it only OUTSIDE design "
        "elements; spec outranks the verilator driver heuristic"),
    "test_regress/t/t_lint_implicit_type_bad.v": ("must_reject",
        "pinned .3.14a: `default_nettype wire inside module t (line 14) - "
        "IEEE 1800-2017 22.8 / IEEE 1364-2005 19.2 permit it only OUTSIDE design "
        "elements; spec outranks the verilator driver heuristic"),
    # ---- SV-CORPUS-GRAD.13x.d (PGEN-SV-CORPUS-GRAD-0353) -----------------------------------------
    # Two IEEE 1800-2023 constructs judged on the 1800-2017 lane: the same edition-contrast ruling
    # .13k made for compound assignment on the verilog_2005 lane. The parser accepts BOTH under
    # `--profile sv_2023` (measured), so these pins record a fact about the LANE, not the parser;
    # pinned class=invalid on sv_2017 and class=control on sv_2023 under adjudication_repros/.
    "test_regress/t/t_class_override_bad.v": ("must_reject",
        "pinned .13x.d: line 23 `function :extends int get_e; return 1; endfunction` - a dynamic-override "
        "specifier. IEEE 1800-2023 A.1.9 dynamic_override_specifiers ::= [ initial_or_extends_specifier ] "
        "[ final_specifier ], initial_or_extends_specifier ::= : initial | : extends, final_specifier ::= "
        ": final (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1210-1217); IEEE "
        "1800-2017 class_method (docs/systemverilog/2017/md/section-41-data-read-api.md:210-215) and "
        "function_declaration carry NO such production - the construct does not exist in the edition this "
        "lane judges. EDITION CONTRAST, the .13k `|=`-on-verilog_2005 shape: REJECTS at "
        "furthest_position=819 on sv_2017 and ACCEPTS on sv_2023 (measured, both on the corpus file and "
        "on the minimal reproducer `class C; function :extends int get_e; return 1; endfunction "
        "endclass`); the 2017 spelling `function int get_e;` ACCEPTS on sv_2017 (arms c1/c1c). On an "
        "sv_2023 lane this row would be must_accept (its golden names 8.20 prose: no base method to "
        "extend); no such lane exists yet (SV-CORPUS-GRAD.14). Spec outranks the verilator driver key, "
        "which records what verilator (a 2023 tool) tolerates"),
    "test_regress/t/t_param_type_fwd_bad.v": ("must_reject",
        "pinned .13x.d: line 10 `parameter type enum E_t;` - a type parameter with a forward type "
        "restriction. IEEE 1800-2023 A.2.1.1 type_parameter_declaration ::= type [ forward_type ] "
        "list_of_type_assignments "
        "(docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:721), forward_type ::= "
        "enum | struct | union | class | interface class "
        "(docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:803); IEEE 1800-2017 "
        "parameter_declaration ::= ... | parameter type list_of_type_assignments "
        "(docs/systemverilog/2017/md/section-41-data-read-api.md:739-741) admits no forward_type. edition "
        "contrast (WITHDRAWN by .3.37, see the closing sentence): REJECTS at furthest_position=251 on sv_2017 and ACCEPTS on sv_2023 (measured on the "
        "corpus file and on `module sub; parameter type enum E_t; endmodule`); the 2017 spelling "
        "`parameter type E_t = int;` ACCEPTS on sv_2017 (arms c2/c2c). On an sv_2023 lane this row would "
        "be must_accept (its golden is 6.20.3 prose about the override's type); no such lane exists yet "
        "(SV-CORPUS-GRAD.14). Spec outranks the verilator driver key"
        " ⛔ WITHDRAWN by SV-CORPUS-GRAD.3.37: the text omits the default in a BODY, which IEEE 1800-2023 Annex A footnote 22 (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:3483-3484, the twin of 2017's footnote 18): the data_type may be omitted from a type_assignment ONLY within a parameter_port_list forbids - the row is must_reject in BOTH editions; the .13x.d control pin was corrected to carry a default"),
    # ---- SV-CORPUS-GRAD.14b (PGEN-SV-CORPUS-GRAD-0360) ------------------------------------------
    # The .14a sv_2017 x sv_2023 census: ten verilator rows the 2023 profile parses and the 2017
    # profile refuses, each an IEEE 1800-2023 construct with NO derivation in 1800-2017 Annex A -
    # the .13x.d ruling at corpus scale. verilator is a 2023 tool; its driver keys them must_accept.
    # On the 2017 lane they are must_reject EDITION CONTRASTS; on the sv_2023 lane (.14b.2) must_accept.
    "test_regress/t/t_class_extends_default.v": ("must_reject",
        "pinned .14b: line 14 `class Cls1 extends Base1(default);`. IEEE 1800-2023 A.1.9 class_declaration ::= ... [ extends class_type [ ( [ list_of_arguments | default ] ) ] ] (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:179); IEEE 1800-2017 A.1.9 [ extends class_type [ ( list_of_arguments ) ] ] (docs/systemverilog/2017/md/section-41-data-read-api.md:187) admits no `default` - the construct does not exist in the edition this lane judges. "
        "EDITION CONTRAST (the .13k / .13x.d shape): REJECTS at furthest_position=320 on sv_2017 and ACCEPTS on sv_2023 - measured on the corpus file by the "
        "sv2023 lane itself (SV-CORPUS-GRAD.14a census, a 2023_only_pass row) and on the pins invalid_class_extends_default_sv2017.sv / control_class_extends_default_sv2023.sv / control_class_extends_default_sv2017.sv. On the sv_2023 lane this row is must_accept "
        "(its golden is empty: a passing test; SV-CORPUS-GRAD.14b.2). Spec outranks the verilator driver key, which records what verilator (a 2023 tool) tolerates"),
    "test_regress/t/t_class_new_default.v": ("must_reject",
        "pinned .14b: line 10 `function new(default);`. IEEE 1800-2023 A.1.9 class_constructor_declaration ::= function [ class_scope ] new [ ( [ class_constructor_arg_list ] ) ] ; (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:546-549), class_constructor_arg ::= tf_port_item | default (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:557); IEEE 1800-2017 A.1.9 new [ ( [ tf_port_list ] ) ] (docs/systemverilog/2017/md/section-41-data-read-api.md:622-625), tf_port_item admits no `default` - the construct does not exist in the edition this lane judges. "
        "EDITION CONTRAST (the .13k / .13x.d shape): REJECTS at furthest_position=249 on sv_2017 and ACCEPTS on sv_2023 - measured on the corpus file by the "
        "sv2023 lane itself (SV-CORPUS-GRAD.14a census, a 2023_only_pass row) and on the pins invalid_class_new_default_sv2017.sv / control_class_new_default_sv2023.sv / control_class_new_default_sv2017.sv. On the sv_2023 lane this row is must_accept "
        "(a passing test; SV-CORPUS-GRAD.14b.2). Spec outranks the verilator driver key, which records what verilator (a 2023 tool) tolerates"),
    "test_regress/t/t_class_override.v": ("must_reject",
        "pinned .14b: line 26 `function :initial int get_i; return 1; endfunction`. IEEE 1800-2023 A.1.9 dynamic_override_specifiers ::= [ initial_or_extends_specifier ] [ final_specifier ], initial_or_extends_specifier ::= : initial | : extends (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1210-1217); IEEE 1800-2017 class_method (docs/systemverilog/2017/md/section-41-data-read-api.md:210-215) and function_declaration carry no such production - the construct does not exist in the edition this lane judges. "
        "EDITION CONTRAST (the .13k / .13x.d shape): REJECTS at furthest_position=978 on sv_2017 and ACCEPTS on sv_2023 - measured on the corpus file by the "
        "sv2023 lane itself (SV-CORPUS-GRAD.14a census, a 2023_only_pass row) and on the pins invalid_override_specifier_initial_sv2017.sv / control_override_specifier_initial_sv2023.sv (+ the .13x.d control_plain_method_sv2017.sv). On the sv_2023 lane this row is must_accept "
        "(a passing test - the _bad twin is the .13x.d row; SV-CORPUS-GRAD.14b.2). Spec outranks the verilator driver key, which records what verilator (a 2023 tool) tolerates"),
    "test_regress/t/t_export_packed_struct.v": ("must_reject",
        "pinned .14b: line 9 `typedef union soft packed {`. IEEE 1800-2023 A.2.2.1 struct_union ::= struct | union [ soft | tagged ] (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:916-920); IEEE 1800-2017 A.2.2.1 struct_union ::= struct | union [ tagged ] (docs/systemverilog/2017/md/section-41-data-read-api.md:917) admits no `soft` - the construct does not exist in the edition this lane judges. "
        "EDITION CONTRAST (the .13k / .13x.d shape): REJECTS at furthest_position=248 on sv_2017 and ACCEPTS on sv_2023 - measured on the corpus file by the "
        "sv2023 lane itself (SV-CORPUS-GRAD.14a census, a 2023_only_pass row) and on the pins invalid_union_soft_packed_sv2017.sv / control_union_soft_packed_sv2023.sv / control_union_soft_packed_sv2017.sv. On the sv_2023 lane this row is must_accept "
        "(a passing test; SV-CORPUS-GRAD.14b.2). Spec outranks the verilator driver key, which records what verilator (a 2023 tool) tolerates"),
    "test_regress/t/t_export_packed_struct2.v": ("must_reject",
        "pinned .14b: line 9 `typedef union soft packed {`. IEEE 1800-2023 A.2.2.1 struct_union ::= struct | union [ soft | tagged ] (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:916-920); IEEE 1800-2017 A.2.2.1 struct_union ::= struct | union [ tagged ] (docs/systemverilog/2017/md/section-41-data-read-api.md:917) admits no `soft` - the construct does not exist in the edition this lane judges. "
        "EDITION CONTRAST (the .13k / .13x.d shape): REJECTS at furthest_position=267 on sv_2017 and ACCEPTS on sv_2023 - measured on the corpus file by the "
        "sv2023 lane itself (SV-CORPUS-GRAD.14a census, a 2023_only_pass row) and on the pins the union_soft_packed pins (shared with t_export_packed_struct.v). On the sv_2023 lane this row is must_accept "
        "(a passing test; SV-CORPUS-GRAD.14b.2). Spec outranks the verilator driver key, which records what verilator (a 2023 tool) tolerates"),
    "test_regress/t/t_param_type_fwd.v": ("must_reject",
        "pinned .14b: line 20 `parameter type enum E_t;`. IEEE 1800-2023 A.2.1.1 type_parameter_declaration ::= type [ forward_type ] list_of_type_assignments (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:721), forward_type (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:803); IEEE 1800-2017 parameter type list_of_type_assignments (docs/systemverilog/2017/md/section-41-data-read-api.md:739-741) admits no forward_type - the construct does not exist in the edition this lane judges. "
        "edition contrast (WITHDRAWN by .3.37, see the closing sentence) (the .13k / .13x.d shape): REJECTS at furthest_position=378 on sv_2017 and ACCEPTS on sv_2023 - measured on the corpus file by the "
        "sv2023 lane itself (SV-CORPUS-GRAD.14a census, a 2023_only_pass row) and on the pins the .13x.d pins invalid_type_parameter_forward_type_sv2017.sv / control_type_parameter_forward_type_sv2023.sv (their worklist_rows is now 2: this row is the _bad twin's passing sibling). On the sv_2023 lane this row is must_accept "
        "(a passing test; SV-CORPUS-GRAD.14b.2). Spec outranks the verilator driver key, which records what verilator (a 2023 tool) tolerates"
        " ⛔ WITHDRAWN by SV-CORPUS-GRAD.3.37: the text omits the default in a BODY, which IEEE 1800-2023 Annex A footnote 22 (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:3483-3484, the twin of 2017's footnote 18): the data_type may be omitted from a type_assignment ONLY within a parameter_port_list forbids - the row is must_reject in BOTH editions; the .13x.d control pin was corrected to carry a default"),
    "test_regress/t/t_string_sel.v": ("must_reject",
        "pinned .14b: line 35 `if (get_str()[1] != \"a\") $stop;`. IEEE 1800-2023 A.8.4 primary ::= ... | function_subroutine_call [ [ range_expression ] ] (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:3079-3085); IEEE 1800-2017 A.8.4 primary ::= ... | function_subroutine_call (docs/systemverilog/2017/md/section-41-data-read-api.md:3122-3128) admits no range after a call - the construct does not exist in the edition this lane judges. "
        "EDITION CONTRAST (the .13k / .13x.d shape): REJECTS at furthest_position=651 on sv_2017 and ACCEPTS on sv_2023 - measured on the corpus file by the "
        "sv2023 lane itself (SV-CORPUS-GRAD.14a census, a 2023_only_pass row) and on the pins invalid_select_on_call_result_sv2017.sv / control_select_on_call_result_sv2023.sv / control_select_on_call_result_sv2017.sv. On the sv_2023 lane this row is must_accept "
        "(a passing test; SV-CORPUS-GRAD.14b.2). Spec outranks the verilator driver key, which records what verilator (a 2023 tool) tolerates"),
    "test_regress/t/t_union_soft.v": ("must_reject",
        "pinned .14b: line 9 `union soft {`. IEEE 1800-2023 A.2.2.1 struct_union ::= struct | union [ soft | tagged ] (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:916-920); IEEE 1800-2017 A.2.2.1 struct_union ::= struct | union [ tagged ] (docs/systemverilog/2017/md/section-41-data-read-api.md:917) admits no `soft` - the construct does not exist in the edition this lane judges. "
        "EDITION CONTRAST (the .13k / .13x.d shape): REJECTS at furthest_position=221 on sv_2017 and ACCEPTS on sv_2023 - measured on the corpus file by the "
        "sv2023 lane itself (SV-CORPUS-GRAD.14a census, a 2023_only_pass row) and on the pins invalid_union_soft_sv2017.sv / control_union_soft_sv2023.sv / control_union_soft_sv2017.sv. On the sv_2023 lane this row is must_accept "
        "(a passing test; SV-CORPUS-GRAD.14b.2). Spec outranks the verilator driver key, which records what verilator (a 2023 tool) tolerates"),
    "test_regress/t/t_var_ref_static.v": ("must_reject",
        "pinned .14b: line 12 `function void crs(const ref static i);`. IEEE 1800-2023 A.2.7 tf_port_direction ::= port_direction | [ const ] ref [ static ] (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1198-1202); IEEE 1800-2017 A.2.7 tf_port_direction ::= port_direction | const ref (docs/systemverilog/2017/md/section-41-data-read-api.md:1159) admits no `static` - the construct does not exist in the edition this lane judges. "
        "EDITION CONTRAST (the .13k / .13x.d shape): REJECTS at furthest_position=357 on sv_2017 and ACCEPTS on sv_2023 - measured on the corpus file by the "
        "sv2023 lane itself (SV-CORPUS-GRAD.14a census, a 2023_only_pass row) and on the pins invalid_ref_static_port_sv2017.sv / control_ref_static_port_sv2023.sv / control_ref_static_port_sv2017.sv. On the sv_2023 lane this row is must_accept "
        "(a passing test; SV-CORPUS-GRAD.14b.2). Spec outranks the verilator driver key, which records what verilator (a 2023 tool) tolerates"),
    "test_regress/t/t_array_event.v": ("must_reject",
        "pinned .14b: line 15 `->write_events[index];`. IEEE 1800-2023 A.6.5 event_trigger ::= -> hierarchical_event_identifier nonrange_select ; (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:2189-2192; nonrange_select docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:3115); IEEE 1800-2017 A.6.5 event_trigger ::= -> hierarchical_event_identifier ; (docs/systemverilog/2017/md/section-41-data-read-api.md:2191-2195) with hierarchical_event_identifier ::= hierarchical_identifier (docs/systemverilog/2017/md/section-41-data-read-api.md:3360) whose FINAL identifier carries no select (docs/systemverilog/2017/md/section-41-data-read-api.md:3361) - --trace-rules shows the 2017 path parsing `[index]` inside hierarchical_identifier's loop and failing at `;` where a `.` is required - the construct does not exist in the edition this lane judges. "
        "EDITION CONTRAST (the .13k / .13x.d shape): REJECTS at furthest_position=393 on sv_2017 and ACCEPTS on sv_2023 - measured on the corpus file by the "
        "sv2023 lane itself (SV-CORPUS-GRAD.14a census, a 2023_only_pass row) and on the pins invalid_event_trigger_select_sv2017.sv / control_event_trigger_select_sv2023.sv / control_event_trigger_select_sv2017.sv. On the sv_2023 lane this row is must_accept "
        "(a passing test; SV-CORPUS-GRAD.14b.2). Spec outranks the verilator driver key, which records what verilator (a 2023 tool) tolerates"),
    # ---- SV-CORPUS-GRAD.13x.b (PGEN-SV-CORPUS-GRAD-0351) -----------------------------------------
    # The suite-intended-invalid `must_accept` class, censused by .13x.a (docs/tasks/artifacts/
    # sv_corpus_grad/answer_key_intended_invalid/census.md): 34 verilator rows whose text has NO
    # derivation in IEEE 1800-2017 Annex A. The driver heuristic below keys `fails=True` + a
    # post-parse golden as must_accept - tool testimony about the stage verilator's TOLERANT parser
    # reached, not about the standard; on 9 of these the golden names a SEMANTIC error on a line
    # the parser passes, while the syntax error it stops on is elsewhere. Every pin carries the
    # production, the md line, the measured furthest_position and a two-sided control (the arm
    # names are the .13x.a probe bank, rust/target/s331, recorded in the leaf); every construct is
    # ALSO pinned as class=invalid + class=control under stimuli/sv/adjudication_repros/.
    "test_regress/t/t_assert_imm_nz_bad.v": ("must_reject",
        "pinned .13x.b: an ANSI header `module t ( input clk );` followed by a BODY port declaration "
        "`input clk;` (line 11), and `assert #1 (clk);` (line 14). IEEE 1800-2017 A.1.2 "
        "module_declaration: the module_ansi_header arm takes { non_port_module_item } "
        "(docs/systemverilog/2017/md/section-41-data-read-api.md:108-113) while `port_declaration ;` is a "
        "module_item of the NON-ANSI arm only "
        "(docs/systemverilog/2017/md/section-41-data-read-api.md:327); A.6.10 "
        "deferred_immediate_assert_statement ::= assert #0 ( expression ) action_block "
        "(docs/systemverilog/2017/md/section-41-data-read-api.md:2421-2424) admits no other delay. "
        "REJECTS at furthest_position=229 (the header's `;`) on sv_2017 and sv_2023. Controls: `module t "
        "(input clk); endmodule` ACCEPTS; `assert #0 (clk);` ACCEPTS, `assert #1 (clk);` alone REJECTS "
        "(arms d3/d3c/h1/h1c). Spec outranks the verilator driver key (fails=True + a post-parse golden "
        "reads as must_accept): the golden records the SEMANTIC error the test was written for, and the "
        "parser stops on text no Annex A production derives"),
    "test_regress/t/t_case_inside_bad.v": ("must_reject",
        "pinned .13x.b: line 9 `casex (1'bx) inside`. IEEE 1800-2017 A.6.7 case_statement: the `inside` "
        "arm is the LITERAL keyword `case` - `[ unique_priority ] case ( case_expression ) inside "
        "case_inside_item ...` (docs/systemverilog/2017/md/section-41-data-read-api.md:2228-2238); "
        "case_keyword ::= case | casez | casex belongs to the other two arms only. REJECTS at "
        "furthest_position=252 on sv_2017 and sv_2023. Control: `case (1'bx) inside default: ...` ACCEPTS "
        "(arms a1/a1c). sv2v's error suite keys the same construct `casex_inside` must_reject. Spec "
        "outranks the verilator driver key (fails=True + a post-parse golden reads as must_accept): the "
        "golden records the SEMANTIC error the test was written for, and the parser stops on text no "
        "Annex A production derives"),
    "test_regress/t/t_class_extends_bad.v": ("must_reject",
        "pinned .13x.b: line 13 `class Cls extends Base1, Base2;`. IEEE 1800-2017 A.1.2 "
        "class_declaration: `[ extends class_type [ ( list_of_arguments ) ] ]` - ONE class_type "
        "(docs/systemverilog/2017/md/section-41-data-read-api.md:183-190); a class list is admitted only "
        "after `implements` (interface classes). REJECTS at furthest_position=272 on sv_2017 and sv_2023. "
        "Control: `class Cls extends Base1;` ACCEPTS (arms a31/a31c). Spec outranks the verilator driver "
        "key (fails=True + a post-parse golden reads as must_accept): the golden records the SEMANTIC "
        "error the test was written for, and the parser stops on text no Annex A production derives"),
    "test_regress/t/t_covergroup_autobins_bad.v": ("must_reject",
        "pinned .13x.b: line 17 `bins auto[size_var];` - a bins declaration with NO `=`. IEEE 1800-2017 "
        "A.2.11 bins_or_options: every `bins_keyword bin_identifier [ [ ... ] ]` arm continues with `= "
        "...` (docs/systemverilog/2017/md/section-41-data-read-api.md:1553-1566). REJECTS at "
        "furthest_position=471 (the `;`) on sv_2017 and sv_2023. Control: `bins auto[size_var] = "
        "{[0:3]};` ACCEPTS (arms a3/a3c); the golden's `size must be a constant` is the SEMANTIC error, "
        "on a line the parser never reaches as valid syntax. Spec outranks the verilator driver key "
        "(fails=True + a post-parse golden reads as must_accept): the golden records the SEMANTIC error "
        "the test was written for, and the parser stops on text no Annex A production derives"),
    "test_regress/t/t_covergroup_with_function_foo_bad.v": ("must_reject",
        "pinned .13x.b: line 8 `covergroup cg_bad with function foo(int x);`. IEEE 1800-2017 A.2.11 "
        "coverage_event ::= clocking_event | with function sample ( [ tf_port_list ] ) | @@( ... ) "
        "(docs/systemverilog/2017/md/section-41-data-read-api.md:1520-1523) - `sample` is a literal token "
        "of the production. REJECTS at furthest_position=240 on sv_2017 and sv_2023. Control: `covergroup "
        "cg with function sample(int x);` ACCEPTS (arms a4/a4c). Spec outranks the verilator driver key "
        "(fails=True + a post-parse golden reads as must_accept): the golden records the SEMANTIC error "
        "the test was written for, and the parser stops on text no Annex A production derives"),
    "test_regress/t/t_dpi_type_bad.v": ("must_reject",
        "pinned .13x.b: line 11 `import \"DPI-BAD\" task dpix_twice;`. IEEE 1800-2017 A.2.6 "
        "dpi_spec_string ::= \"DPI-C\" | \"DPI\" "
        "(docs/systemverilog/2017/md/section-41-data-read-api.md:1114) - two literal strings. REJECTS at "
        "furthest_position=371 on sv_2017 and sv_2023. Control: `import \"DPI-C\" task dpix_twice;` "
        "ACCEPTS (arms a5/a5c). Spec outranks the verilator driver key (fails=True + a post-parse golden "
        "reads as must_accept): the golden records the SEMANTIC error the test was written for, and the "
        "parser stops on text no Annex A production derives"),
    "test_regress/t/t_enum_base_bad.v": ("must_reject",
        "pinned .13x.b: line 25 `enum logic [1:0][1:0] {` - TWO packed dimensions on the enum base type. "
        "IEEE 1800-2017 A.2.2.1 enum_base_type ::= integer_atom_type [ signing ] | integer_vector_type [ "
        "signing ] [ packed_dimension ] | type_identifier [ packed_dimension ] "
        "(docs/systemverilog/2017/md/section-41-data-read-api.md:868-871) - at most ONE. REJECTS at "
        "furthest_position=498 on sv_2017 and sv_2023. Control: `enum logic [1:0] { A, B }` ACCEPTS (arms "
        "a6/a6c). The golden's error is line 11 (`enum s_t {`, a struct as base type - 6.19 prose) and "
        "the parser passes that line: the file is parse-invalid for a reason the golden never mentions. "
        "Spec outranks the verilator driver key (fails=True + a post-parse golden reads as must_accept): "
        "the golden records the SEMANTIC error the test was written for, and the parser stops on text no "
        "Annex A production derives"),
    "test_regress/t/t_foreach_bad.v": ("must_reject",
        "pinned .13x.b: line 14 `foreach (array);` - no bracketed loop-variable list. IEEE 1800-2017 "
        "A.6.8 `foreach ( ps_or_hierarchical_array_identifier [ loop_variables ] ) statement` "
        "(docs/systemverilog/2017/md/section-41-data-read-api.md:2334) where the brackets are the "
        "construct's own tokens (12.7.3 "
        "`docs/systemverilog/2017/md/section-12-procedural-programming-statements.md:1045`: 'a "
        "comma-separated list of loop variables enclosed in square brackets'; the OPTIONAL markers sit "
        "inside loop_variables, :2360), and the body is `statement`, never a bare `;`. REJECTS at "
        "furthest_position=290 on sv_2017 and sv_2023. Controls: `foreach (array[i]) $display(i);` "
        "ACCEPTS; `foreach (array) $display(1);` REJECTS; `foreach (array[i]);` REJECTS (arms "
        "a2/a2c/a2d). Spec outranks the verilator driver key (fails=True + a post-parse golden reads as "
        "must_accept): the golden records the SEMANTIC error the test was written for, and the parser "
        "stops on text no Annex A production derives"),
    "test_regress/t/t_foreach_nindex_bad.v": ("must_reject",
        "pinned .13x.b: line 12 `foreach (array[i, j, badk, badl]);` - a bare `;` as the loop body. IEEE "
        "1800-2017 A.6.8 loop_statement "
        "(docs/systemverilog/2017/md/section-41-data-read-api.md:2325-2334): forever/repeat/while/for/do "
        "take statement_or_null, `foreach ... ) statement` is the ONE arm that does not, and "
        "statement_or_null ::= statement | { attribute_instance } ; "
        "(docs/systemverilog/2017/md/section-41-data-read-api.md:2081) is where a bare `;` lives. REJECTS "
        "at furthest_position=287 (the `;`) on sv_2017 and sv_2023. Control: `foreach (array[i]) "
        "$display(i);` ACCEPTS (arms a2/a2c). The golden's `loop variables exceed indices` is 12.7.3 "
        "prose on a line whose SYNTAX is already underivable. Spec outranks the verilator driver key "
        "(fails=True + a post-parse golden reads as must_accept): the golden records the SEMANTIC error "
        "the test was written for, and the parser stops on text no Annex A production derives"),
    "test_regress/t/t_foreach_type_bad.v": ("must_reject",
        "pinned .13x.b: line 19 `foreach (c[i]);` - a bare `;` as the loop body, the "
        "t_foreach_nindex_bad.v shape (A.6.8 docs/systemverilog/2017/md/section-41-data-read-api.md:2334 "
        "takes `statement`; A.6.4 docs/systemverilog/2017/md/section-41-data-read-api.md:2081). REJECTS "
        "at furthest_position=303 on sv_2017 and sv_2023. Control: `foreach (array[i]) $display(i);` "
        "ACCEPTS (arms a2/a2c). Spec outranks the verilator driver key (fails=True + a post-parse golden "
        "reads as must_accept): the golden records the SEMANTIC error the test was written for, and the "
        "parser stops on text no Annex A production derives"),
    "test_regress/t/t_fuzz_genintf_bad.v": ("must_reject",
        "pinned .13x.b: lines 22-25 `m1 m0 ( j.e(0), .b(ifs) );` - an ORDERED connection followed by a "
        "NAMED one. IEEE 1800-2017 A.4.1.1 list_of_port_connections ::= ordered_port_connection { , "
        "ordered_port_connection } | named_port_connection { , named_port_connection } "
        "(docs/systemverilog/2017/md/section-41-data-read-api.md:1811-1820) - all of one kind. REJECTS at "
        "furthest_position=398 on sv_2017 and sv_2023. Control: `m1 m0 (.a(j), .b(j));` ACCEPTS (arms "
        "a7/a7c). sv2v keys the same class `binding_mix_port` must_reject. Spec outranks the verilator "
        "driver key (fails=True + a post-parse golden reads as must_accept): the golden records the "
        "SEMANTIC error the test was written for, and the parser stops on text no Annex A production "
        "derives"),
    "test_regress/t/t_implements_contents_bad.v": ("must_reject",
        "pinned .13x.b: line 8 `int badi;` inside `interface class Icls;`. IEEE 1800-2017 A.1.2 "
        "interface_class_item ::= type_declaration | { attribute_instance } interface_class_method | "
        "local_parameter_declaration ; | parameter_declaration ; | ; "
        "(docs/systemverilog/2017/md/section-41-data-read-api.md:201-206) - no data declaration, no task "
        "body. REJECTS at furthest_position=224 on sv_2017 and sv_2023. Control: `interface class Icls; "
        "pure virtual task t(); endclass` ACCEPTS (arms a29/a29c). 8.26 prose says the same thing the BNF "
        "already refuses. Spec outranks the verilator driver key (fails=True + a post-parse golden reads "
        "as must_accept): the golden records the SEMANTIC error the test was written for, and the parser "
        "stops on text no Annex A production derives"),
    "test_regress/t/t_implements_nested_bad.v": ("must_reject",
        "pinned .13x.b: line 9 `interface class bad_cannot_nest;` nested inside an interface class. IEEE "
        "1800-2017 A.1.2 interface_class_item "
        "(docs/systemverilog/2017/md/section-41-data-read-api.md:201-206) has no class-declaration arm. "
        "REJECTS at furthest_position=237 on sv_2017 and sv_2023. Control: an ordinary class nested in an "
        "ordinary class ACCEPTS (arms a30/a30c). Spec outranks the verilator driver key (fails=True + a "
        "post-parse golden reads as must_accept): the golden records the SEMANTIC error the test was "
        "written for, and the parser stops on text no Annex A production derives"),
    "test_regress/t/t_inst_2star_bad.v": ("must_reject",
        "pinned .13x.b: line 14 `sub sub (foo, .*);` - an ORDERED connection then `.*`, which is a "
        "named_port_connection (docs/systemverilog/2017/md/section-41-data-read-api.md:1818-1820); "
        "A.4.1.1 list_of_port_connections admits one kind per list "
        "(docs/systemverilog/2017/md/section-41-data-read-api.md:1811). REJECTS at furthest_position=275 "
        "on sv_2017 and sv_2023. Controls: `sub sub (.*, .*);` (the golden's line 12, 23.3.2 prose) "
        "ACCEPTS; `sub sub (.a(x));` ACCEPTS (arms a8/a8c). Spec outranks the verilator driver key "
        "(fails=True + a post-parse golden reads as must_accept): the golden records the SEMANTIC error "
        "the test was written for, and the parser stops on text no Annex A production derives"),
    "test_regress/t/t_inst_noname_bad.v": ("must_reject",
        "pinned .13x.b: line 8 `m();` - a module instance with no instance name. IEEE 1800-2017 A.4.1.1 "
        "hierarchical_instance ::= name_of_instance ( [ list_of_port_connections ] ), name_of_instance "
        "::= instance_identifier { unpacked_dimension } "
        "(docs/systemverilog/2017/md/section-41-data-read-api.md:1809-1810); IEEE 1364-2005 A.4.1 says "
        "the same. REJECTS at furthest_position=218 on sv_2017 and sv_2023. Control: `m u();` ACCEPTS "
        "(arms a9/a9c). Spec outranks the verilator driver key (fails=True + a post-parse golden reads as "
        "must_accept): the golden records the SEMANTIC error the test was written for, and the parser "
        "stops on text no Annex A production derives"),
    "test_regress/t/t_lint_colonplus_bad.v": ("must_reject",
        "pinned .13x.b: line 13 `output [2:1] z = r[2 :+ 1];` in a NON-ANSI body - an initializer on an "
        "IMPLICIT-TYPE net port. IEEE 1800-2017 A.2.1.2 output_declaration ::= output net_port_type "
        "list_of_port_identifiers | output variable_port_type list_of_variable_port_identifiers "
        "(docs/systemverilog/2017/md/section-41-data-read-api.md:762-765); only "
        "list_of_variable_port_identifiers carries `[ = constant_expression ]` "
        "(docs/systemverilog/2017/md/section-41-data-read-api.md:994 vs :975) and variable_port_type ::= "
        "var_data_type ::= data_type | var data_type_or_implicit "
        "(docs/systemverilog/2017/md/section-41-data-read-api.md:907-908) needs a type keyword or `var`. "
        "REJECTS at furthest_position=289 on sv_2017 and sv_2023. Controls: `output [2:1] z;` ACCEPTS; "
        "`output reg [2:1] z = 2'b10;` ACCEPTS; `wire [1:0] z = r[2 :+ 1];` ACCEPTS - the `:+` the lint "
        "warns about is `2 : +1`, legal syntax (arms a10/a10b/a10c/a10d). Spec outranks the verilator "
        "driver key (fails=True + a post-parse golden reads as must_accept): the golden records the "
        "SEMANTIC error the test was written for, and the parser stops on text no Annex A production "
        "derives"),
    "test_regress/t/t_lint_notredop_bad.v": ("must_reject",
        "pinned .13x.b: line 12 `assign y[0] = !|v;`. IEEE 1800-2017 A.8.3 expression ::= ... | "
        "unary_operator { attribute_instance } primary "
        "(docs/systemverilog/2017/md/section-41-data-read-api.md:272-280): the operand of `!` must be a "
        "PRIMARY, and `|v` is itself `unary_operator primary`, an expression. REJECTS at "
        "furthest_position=278 on sv_2017 and sv_2023. Control: `!(|v)` ACCEPTS (arms a11/a11c). "
        "iverilog's parse-stage vocabulary carries this exact class (`is not a valid expression. Please "
        "use operator`). Spec outranks the verilator driver key (fails=True + a post-parse golden reads "
        "as must_accept): the golden records the SEMANTIC error the test was written for, and the parser "
        "stops on text no Annex A production derives"),
    "test_regress/t/t_mem_multi_ref_bad.v": ("must_reject",
        "pinned .13x.b: line 19 `dim2[0+:1][1] = 0;` - a PART-select followed by a BIT-select. IEEE "
        "1800-2017 A.8.4 select ::= [ { . member_identifier bit_select } . member_identifier ] bit_select "
        "[ [ part_select_range ] ] (docs/systemverilog/2017/md/section-41-data-read-api.md:3157-3159): "
        "the range is the LAST select. REJECTS at furthest_position=513 on sv_2017 and sv_2023. Control: "
        "`dim2[1][0+:1] = 0;` ACCEPTS (arms a12/a12c). The golden's line 15 is 7.4 prose the parser "
        "passes. Spec outranks the verilator driver key (fails=True + a post-parse golden reads as "
        "must_accept): the golden records the SEMANTIC error the test was written for, and the parser "
        "stops on text no Annex A production derives"),
    "test_regress/t/t_mem_slice_bad.v": ("must_reject",
        "pinned .13x.b: line 36 `active_command3[1:0][2:0][3:0]` - three ranges. IEEE 1800-2017 A.8.4 "
        "constant_select and select carry ONE `[ part_select_range ]`, after the bit-selects "
        "(docs/systemverilog/2017/md/section-41-data-read-api.md:3157, :3174-3176). REJECTS at "
        "furthest_position=969 on sv_2017 and sv_2023. Control: `a[1][2][3:0] = b[1][2][3:0];` ACCEPTS "
        "(arms a13/a13c). Spec outranks the verilator driver key (fails=True + a post-parse golden reads "
        "as must_accept): the golden records the SEMANTIC error the test was written for, and the parser "
        "stops on text no Annex A production derives"),
    "test_regress/t/t_param_implicit_bad.v": ("must_reject",
        "pinned .13x.b: line 9 `module sub1 #([7:0] PAR1 = 1);`. IEEE 1800-2017 A.1.3 "
        "parameter_port_declaration ::= parameter_declaration | local_parameter_declaration | data_type "
        "list_of_param_assignments | type list_of_type_assignments "
        "(docs/systemverilog/2017/md/section-41-data-read-api.md:238-243) - `[7:0]` is a packed "
        "dimension, not a data_type, and the keyword-less arms need one. REJECTS at furthest_position=297 "
        "on sv_2017 and sv_2023. Control: `#(parameter [7:0] PAR1 = 1)` ACCEPTS (arms a14/a14c); "
        "verilator's own message cites 6.20.1/A.2.1.1. Spec outranks the verilator driver key (fails=True "
        "+ a post-parse golden reads as must_accept): the golden records the SEMANTIC error the test was "
        "written for, and the parser stops on text no Annex A production derives"),
    "test_regress/t/t_param_type_bad.v": ("must_reject",
        "pinned .13x.b: line 9 `localparam type bad2 = 2;`. IEEE 1800-2017 A.2.1.1 "
        "local_parameter_declaration ::= ... | localparam type list_of_type_assignments "
        "(docs/systemverilog/2017/md/section-41-data-read-api.md:733-737), A.2.4 type_assignment ::= "
        "type_identifier [ = data_type ] (docs/systemverilog/2017/md/section-41-data-read-api.md:1013) - "
        "`2` is no data_type. REJECTS at furthest_position=275 on sv_2017 and sv_2023. Control: "
        "`localparam type ok = int;` ACCEPTS (arms a15/a15c). Spec outranks the verilator driver key "
        "(fails=True + a post-parse golden reads as must_accept): the golden records the SEMANTIC error "
        "the test was written for, and the parser stops on text no Annex A production derives"),
    "test_regress/t/t_prop_always_bad.v": ("must_reject",
        "pinned .13x.b: line 17 `assert property (s_always a);` - `s_always` with no range. IEEE "
        "1800-2017 A.2.10 property_expr "
        "(docs/systemverilog/2017/md/section-41-data-read-api.md:1344-1346): `| always property_expr` AND "
        "`| always [ cycle_delay_const_range_expression ] property_expr` are two arms, so the brackets of "
        "the second are the construct's own tokens, and `| s_always [ constant_range ] property_expr` has "
        "NO range-free twin. REJECTS at furthest_position=466 on sv_2017 and sv_2023. Controls: `s_always "
        "[1:3] a` ACCEPTS; `always a` ACCEPTS (arms a16/a16c/a16d). The golden's line 13 (a negative "
        "bound, 16.12.11 prose) is passed by the parser. Spec outranks the verilator driver key "
        "(fails=True + a post-parse golden reads as must_accept): the golden records the SEMANTIC error "
        "the test was written for, and the parser stops on text no Annex A production derives"),
    "test_regress/t/t_property_case_bad.v": ("must_reject",
        "pinned .13x.b: line 18 `case (sel) endcase` inside property p2 - no item. IEEE 1800-2017 A.2.10 "
        "`| case ( expression_or_dist ) property_case_item { property_case_item } endcase` "
        "(docs/systemverilog/2017/md/section-41-data-read-api.md:1337) - at least one. REJECTS at "
        "furthest_position=356 on sv_2017 and sv_2023. Control: p1's `case (sel) default: 1'b1; default: "
        "1'b0; endcase` (the golden's duplicate default, 12.5 prose) ACCEPTS (arms a17/a17c). Spec "
        "outranks the verilator driver key (fails=True + a post-parse golden reads as must_accept): the "
        "golden records the SEMANTIC error the test was written for, and the parser stops on text no "
        "Annex A production derives"),
    "test_regress/t/t_property_sexpr_bad.v": ("must_reject",
        "pinned .13x.b: line 20 `##1 not val`. IEEE 1800-2017 A.2.10 sequence_expr ::= cycle_delay_range "
        "sequence_expr ... (docs/systemverilog/2017/md/section-41-data-read-api.md:1404-1414) - after "
        "`##1` a SEQUENCE expression is required, and `not property_expr` is an arm of property_expr only "
        "(docs/systemverilog/2017/md/section-41-data-read-api.md:1324). REJECTS at furthest_position=398 "
        "on sv_2017 and sv_2023. Control: `assert property (@(posedge clk) not val);` ACCEPTS (arms "
        "a18/a18c). Spec outranks the verilator driver key (fails=True + a post-parse golden reads as "
        "must_accept): the golden records the SEMANTIC error the test was written for, and the parser "
        "stops on text no Annex A production derives"),
    "test_regress/t/t_randomize_with_idlist_method_bad.v": ("must_reject",
        "pinned .13x.b: line 13 `q.sum() with (item) { item > 0; }`. IEEE 1800-2017 A.8.2 "
        "array_manipulation_call ::= array_method_name { attribute_instance } [ ( list_of_arguments ) ] [ "
        "with ( expression ) ] (docs/systemverilog/2017/md/section-41-data-read-api.md:2954-2957) - "
        "`with` takes a parenthesised EXPRESSION; the `with [ ( identifier_list ) ] constraint_block` "
        "form belongs to randomize_call alone "
        "(docs/systemverilog/2017/md/section-41-data-read-api.md:2961-2964). REJECTS at "
        "furthest_position=322 on sv_2017 and sv_2023. Control: `q.sum() with (item > 0)` ACCEPTS (arms "
        "a19/a19c). Spec outranks the verilator driver key (fails=True + a post-parse golden reads as "
        "must_accept): the golden records the SEMANTIC error the test was written for, and the parser "
        "stops on text no Annex A production derives"),
    "test_regress/t/t_timescale_udp.v": ("must_reject",
        "pinned .13x.b (the row .3.30 named): line 39 `?     :  ?;` - `?` in a COMBINATIONAL output "
        "field. IEEE 1800-2017 A.5.3 combinational_entry ::= level_input_list : output_symbol ; and "
        "output_symbol ::= 0 | 1 | x | X "
        "(docs/systemverilog/2017/md/section-29-user-defined-primitives.md:119, :130; "
        "docs/systemverilog/2017/md/section-41-data-read-api.md:1975, :1986); `?` is a level_symbol "
        "(inputs only, docs/systemverilog/2017/md/section-29-user-defined-primitives.md:131). IEEE "
        "1364-2005 A.5.3 identical. REJECTS at furthest_position=605 on sv_2017 and sv_2023. Control: `0 "
        ": 1;` ACCEPTS (arms h2/h2c); the rest of the file's tables parse since release 1.0.207. The "
        "driver key `t_timescale_udp.py expects success` records what verilator tolerates. Spec outranks "
        "the verilator driver key (fails=True + a post-parse golden reads as must_accept): the golden "
        "records the SEMANTIC error the test was written for, and the parser stops on text no Annex A "
        "production derives"),
    "test_regress/t/t_type_compare_bad.v": ("must_reject",
        "pinned .13x.b: line 11 `case (type(real))` in an `initial` block - a type reference as a "
        "PROCEDURAL case_expression. IEEE 1800-2017 A.6.7 case_expression ::= expression "
        "(docs/systemverilog/2017/md/section-41-data-read-api.md:2239); A.8.4 primary "
        "(docs/systemverilog/2017/md/section-41-data-read-api.md:3122-3140; the .13x.b basis cited :641-656, the "
        "constraint_primary block - corrected by .3.32) has no type_reference arm - "
        "type_reference is a constant_primary "
        "(docs/systemverilog/2017/md/section-41-data-read-api.md:3109, footnote 40 "
        "docs/systemverilog/2017/md/section-41-data-read-api.md:3560) and a data_type; 6.23 "
        "(`docs/systemverilog/2017/md/section-65536-216-bits.md:1610-1614`) makes a type-reference "
        "comparison 'a constant expression' and writes its own `case (type(bus_t))` inside `generate` "
        "(:1626-1631). REJECTS at furthest_position=325 on sv_2017 and sv_2023. Controls: `generate case "
        "(type(bus_t)) type(bit[12:0]): ... endcase endgenerate` ACCEPTS; `case (a) 1: ; default: ; "
        "endcase` ACCEPTS (arms d1/d1c/d1d). RULED by SV-CORPUS-GRAD.3.32 (2026-09-07, PGEN-SV-CORPUS-GRAD-0375): "
        "the letter stands in both editions (1800-2023 primary docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:3079-3094 "
        "carries no type_reference either), so this pin is a STANDING invalid pin and the three positive-test rows that "
        "write the same form (t_type_compare.v, t_type_expression_compare.v, sv-tests 6.23--type_op_compare.sv) are keyed "
        "must_reject beside it. Spec outranks the verilator driver key (fails=True + a post-parse golden "
        "reads as must_accept): the golden records the SEMANTIC error the test was written for, and the "
        "parser stops on text no Annex A production derives"),
    # ---- SV-CORPUS-GRAD.3.32 (PGEN-SV-CORPUS-GRAD-0375) - the partition question RULED: a type reference is a
    # constant_primary in BOTH editions and no primary derives it, so a type-reference comparison is legal exactly
    # where a constant_expression is required (generate if / case, a parameter initializer, a static cast's
    # casting_type) and has no derivation as a procedural if / case / assert condition. Three positive tests write
    # the procedural form; the parser stops on every one at the byte after the first `type(...)`.
    "test_regress/t/t_type_compare.v": ("must_reject",
        "pinned .3.32: line 32 `if (type(real) == type(logic[12:0])) $stop;` inside `initial begin` - a type-reference "
        "comparison as a PROCEDURAL if condition (the same form on lines 32-40, and procedural `case (type(real))` with "
        "type_reference items at 43, 52 and 59). IEEE 1800-2017 A.6.6 conditional_statement ::= [ unique_priority ] if "
        "( cond_predicate ) statement_or_null ... (docs/systemverilog/2017/md/section-41-data-read-api.md:2205-2212), "
        "cond_predicate ::= expression_or_cond_pattern { &&& expression_or_cond_pattern } "
        "(docs/systemverilog/2017/md/section-41-data-read-api.md:2213), expression_or_cond_pattern ::= expression | "
        "expression matches pattern (docs/systemverilog/2017/md/section-41-data-read-api.md:2218-2219); "
        "IEEE 1800-2017 A.8.4 primary (docs/systemverilog/2017/md/section-41-data-read-api.md:3122-3140) has no type_reference arm - type_reference (docs/systemverilog/2017/md/section-41-data-read-api.md:918-920) is a constant_primary (docs/systemverilog/2017/md/section-41-data-read-api.md:3109) under footnote 40 (docs/systemverilog/2017/md/section-41-data-read-api.md:3560-3562: legal as a casting_type and with the four equality operators - i.e. inside a constant_expression) and a data_type (docs/systemverilog/2017/md/section-41-data-read-api.md:859); 6.23 (docs/systemverilog/2017/md/section-65536-216-bits.md:1610-1614) says such comparisons 'are considered to be constant expressions (see 11.2.1)' and its only example is the generate case (docs/systemverilog/2017/md/section-65536-216-bits.md:1626-1631); 11.2.1 (docs/systemverilog/2017/md/section-11-operators-and-expressions.md:87-90): 'Some statement constructs require an expression to be a constant expression' - a procedural if / case takes an expression and is not one of them. "
        "REJECTS at furthest_position=830 on sv_2017 and sv_2023: the byte after the first `type(...)`, where the ONE derivation that admits a type reference inside an expression - a static cast, primary ::= ... | cast, cast ::= casting_type ' ( expression ) (docs/systemverilog/2017/md/section-41-data-read-api.md:3186), casting_type ::= ... | constant_primary (docs/systemverilog/2017/md/section-41-data-read-api.md:841) - requires its ' and meets the comparison operator. "
        "Two-sided: lines 7-29 of this file PARSE - the module-level `if` / `case` at 19-29 are generate constructs "
        "on a constant_expression, `Sub #(.T(int)) sub();` (14) and `parameter type T = type(logic[11:0])` (7) ACCEPT "
        "(controls gen_level_ifs.sv / sub_type_override.sv / param_type_ref.sv, both profiles), and `if (type(a)'(y) == 3)` "
        "in procedural code ACCEPTS. Pins invalid_procedural_type_reference_comparison.sv / control_constant_context_type_reference_comparison.sv (the same comparisons as a module-level if / case = conditional_generate_construct / case_generate_construct on a constant_expression, docs/systemverilog/2017/md/section-41-data-read-api.md:1887-1896, in a localparam initializer, and the cast `type(a)'(y)` in procedural code - all ACCEPT on both profiles). Spec outranks the suite: verilator, sv-tests and slang's regression file all write the procedural form and no LRM text or example does - IEEE 1800-2023 kept type_reference in constant_primary alone (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:3066, footnote 45 :3525-3527) and primary without it (:3079-3094) while revising 6.23 (docs/systemverilog/2023/md/section-6-data-types.md:2936-2975). The verilator driver key reads a passing test as must_accept"),
    "test_regress/t/t_type_expression_compare.v": ("must_reject",
        "pinned .3.32: line 14 `if (type(value1) != type(value2)) $stop;` inside `initial begin` - a type-reference "
        "comparison as a PROCEDURAL if condition (lines 14, 15 and 29; procedural `case (type(value1))` / "
        "`case (type(value1 + value2 + 13))` with type_reference items at 16 and 23). IEEE 1800-2017 A.6.6 "
        "conditional_statement (docs/systemverilog/2017/md/section-41-data-read-api.md:2205-2212) takes a cond_predicate "
        "(docs/systemverilog/2017/md/section-41-data-read-api.md:2213) whose operand is an expression "
        "(docs/systemverilog/2017/md/section-41-data-read-api.md:2218-2219); IEEE 1800-2017 A.8.4 primary (docs/systemverilog/2017/md/section-41-data-read-api.md:3122-3140) has no type_reference arm - type_reference (docs/systemverilog/2017/md/section-41-data-read-api.md:918-920) is a constant_primary (docs/systemverilog/2017/md/section-41-data-read-api.md:3109) under footnote 40 (docs/systemverilog/2017/md/section-41-data-read-api.md:3560-3562: legal as a casting_type and with the four equality operators - i.e. inside a constant_expression) and a data_type (docs/systemverilog/2017/md/section-41-data-read-api.md:859); 6.23 (docs/systemverilog/2017/md/section-65536-216-bits.md:1610-1614) says such comparisons 'are considered to be constant expressions (see 11.2.1)' and its only example is the generate case (docs/systemverilog/2017/md/section-65536-216-bits.md:1626-1631); 11.2.1 (docs/systemverilog/2017/md/section-11-operators-and-expressions.md:87-90): 'Some statement constructs require an expression to be a constant expression' - a procedural if / case takes an expression and is not one of them. "
        "REJECTS at furthest_position=338 on sv_2017 and sv_2023: the byte after the first `type(...)`, where the ONE derivation that admits a type reference inside an expression - a static cast, primary ::= ... | cast, cast ::= casting_type ' ( expression ) (docs/systemverilog/2017/md/section-41-data-read-api.md:3186), casting_type ::= ... | constant_primary (docs/systemverilog/2017/md/section-41-data-read-api.md:841) - requires its ' and meets the comparison operator. "
        "Two-sided: `automatic Int_T value1 = 7;` / `automatic int value2 = 13;` / `real r;` in the same block ACCEPT "
        "(control automatic_decl.sv, both profiles); `type(type(value2 + 18 - 40))` on line 24 is 6.23's type reference "
        "applied to a data type and sits past the stop. Pins invalid_procedural_type_reference_comparison.sv / control_constant_context_type_reference_comparison.sv (the same comparisons as a module-level if / case = conditional_generate_construct / case_generate_construct on a constant_expression, docs/systemverilog/2017/md/section-41-data-read-api.md:1887-1896, in a localparam initializer, and the cast `type(a)'(y)` in procedural code - all ACCEPT on both profiles). Spec outranks the suite: verilator, sv-tests and slang's regression file all write the procedural form and no LRM text or example does - IEEE 1800-2023 kept type_reference in constant_primary alone (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:3066, footnote 45 :3525-3527) and primary without it (:3079-3094) while revising 6.23 (docs/systemverilog/2023/md/section-6-data-types.md:2936-2975). The verilator driver key reads a passing test "
        "(2025, Antmicro) as must_accept"),
    # ---- SV-CORPUS-GRAD.3.47 (PGEN-SV-CORPUS-GRAD-0377) - the IEEE 1800-2023 TRIPLE-QUOTED string is no 1800-2017 form.
    "test_regress/t/t_display_qqq.v": ("must_reject",
        "pinned .3.47: line 10 `$display(\"\"\"First \"quoted\"\\nsecond\\` … `fourth\"\"\");` - a TRIPLE-QUOTED string literal. "
        "IEEE 1800-2023 5.9 (docs/systemverilog/2023/md/section-5-lexical-conventions.md:436-452): `string_literal ::= "
        "quoted_string | triple_quoted_string`, `triple_quoted_string ::= \"\"\" { triple_quoted_string_item | "
        "string_escape_seq } \"\"\"`; IEEE 1800-2017 5.9 (docs/systemverilog/2017/md/section-32-h-12ab-f001.md:99-102) "
        "defines the double-quoted form ALONE, so under 1800-2017 `\"\"` is an empty string and the third `\"` opens a "
        "string that runs to the next quote - the text has no derivation in the edition this lane judges. EDITION CONTRAST, "
        "the .13x.d shape: REJECTS on sv_2017 (furthest_position at the third quote of line 10) and, since 1.0.221, ACCEPTS on "
        "sv_2023, where the string terminal's 2023 twin carries the triple-quoted alternative ({kind: \"triple_quoted\", body}). "
        "Pins control_string_triple_quoted_sv2023.sv / invalid_string_triple_quoted_under_2017.sv. Spec outranks the verilator "
        "driver key (a passing test = must_accept): verilator is a 2023 tool"),
    "test_regress/t/t_udp_bad.v": ("must_reject",
        "pinned .13x.b: line 15 `tri  a_bad;` inside a primitive. IEEE 1800-2017 A.5.2 "
        "udp_port_declaration ::= udp_output_declaration ; | udp_input_declaration ; | "
        "udp_reg_declaration ; (docs/systemverilog/2017/md/section-41-data-read-api.md:1949-1953; "
        "docs/systemverilog/2017/md/section-29-user-defined-primitives.md:98-101) - no net declaration of "
        "any kind; IEEE 1364-2005 A.5.2 identical. REJECTS at furthest_position=291 (the end of the "
        "header line) on sv_2017 and sv_2023. Control: `output a_bad;` in its place ACCEPTS (arm a23 vs "
        "the release's passing UDP corpus). The golden's PINMISSING warning is about the instance on line "
        "10. Spec outranks the verilator driver key (fails=True + a post-parse golden reads as "
        "must_accept): the golden records the SEMANTIC error the test was written for, and the parser "
        "stops on text no Annex A production derives"),
    "test_regress/t/t_udp_bad_comb_trigger.v": ("must_reject",
        "pinned .13x.b: line 14 `(01) 1 0  :   0;` - an EDGE indicator in a ONE-colon table row. IEEE "
        "1800-2017 A.5.3 combinational_entry ::= level_input_list : output_symbol ; "
        "(docs/systemverilog/2017/md/section-29-user-defined-primitives.md:119) admits level symbols "
        "only, and an edge belongs to sequential_entry ::= seq_input_list : current_state : next_state ; "
        "with TWO colons (docs/systemverilog/2017/md/section-29-user-defined-primitives.md:123-126; "
        "docs/systemverilog/2017/md/section-41-data-read-api.md:1975-1982); IEEE 1364-2005 A.5.3 "
        "identical. REJECTS at furthest_position=312 (the end of line 13, i.e. the start of line 14) on "
        "sv_2017 and sv_2023. Control: `1 1 0 : 0;` in its place ACCEPTS (arms a20/a20c). Spec outranks "
        "the verilator driver key (fails=True + a post-parse golden reads as must_accept): the golden "
        "records the SEMANTIC error the test was written for, and the parser stops on text no Annex A "
        "production derives"),
    "test_regress/t/t_udp_bad_illegal_output.v": ("must_reject",
        "pinned .13x.b: line 16 `1 1 ?  :  ?: *;` - `*` as the next state. IEEE 1800-2017 A.5.3 "
        "next_state ::= output_symbol | - and output_symbol ::= 0 | 1 | x | X "
        "(docs/systemverilog/2017/md/section-29-user-defined-primitives.md:129-130; "
        "docs/systemverilog/2017/md/section-41-data-read-api.md:1985-1986); IEEE 1364-2005 A.5.3 "
        "identical. REJECTS at furthest_position=378 on sv_2017 and sv_2023. Control: `1 1 : ? : -;` "
        "ACCEPTS (arms a21/a21c). The golden's `output must be of reg type` is 29.4 prose on line 9. Spec "
        "outranks the verilator driver key (fails=True + a post-parse golden reads as must_accept): the "
        "golden records the SEMANTIC error the test was written for, and the parser stops on text no "
        "Annex A production derives"),
    "test_regress/t/t_udp_bad_multi_output.v": ("must_reject",
        "pinned .13x.b: line 8 `output dout1, dout2;` inside a primitive. IEEE 1800-2017 A.5.2 "
        "udp_output_declaration ::= { attribute_instance } output port_identifier "
        "(docs/systemverilog/2017/md/section-29-user-defined-primitives.md:105-107; "
        "docs/systemverilog/2017/md/section-41-data-read-api.md:1956-1958) - ONE identifier, no list; "
        "IEEE 1364-2005 A.5.2 identical. REJECTS at furthest_position=258 (the comma) on sv_2017 and "
        "sv_2023. Control: `output dout1;` ACCEPTS (arm a22 vs a20c). Spec outranks the verilator driver "
        "key (fails=True + a post-parse golden reads as must_accept): the golden records the SEMANTIC "
        "error the test was written for, and the parser stops on text no Annex A production derives"),
    "test_regress/t/t_udp_tableeof_bad.v": ("must_reject",
        "pinned .13x.b: the file ends inside `table` (14 lines, no `endtable`, no `endprimitive`). IEEE "
        "1800-2017 A.5.3 combinational_body ::= table combinational_entry { combinational_entry } "
        "endtable (docs/systemverilog/2017/md/section-29-user-defined-primitives.md:118; "
        "docs/systemverilog/2017/md/section-41-data-read-api.md:1974) and A.5.1 udp_declaration ... "
        "endprimitive (docs/systemverilog/2017/md/section-41-data-read-api.md:1930-1934). REJECTS at "
        "furthest_position=237 on sv_2017 and sv_2023. Control: the same header and table closed by "
        "`endtable endprimitive` ACCEPTS (arm a24 vs a23's control). The intent is in the file name; the "
        "derivation is simply unfinished. Spec outranks the verilator driver key (fails=True + a "
        "post-parse golden reads as must_accept): the golden records the SEMANTIC error the test was "
        "written for, and the parser stops on text no Annex A production derives"),
    "test_regress/t/t_var_dotted_dup_bad.v": ("must_reject",
        "pinned .13x.b: lines 9-16 `generate begin ... end begin ... end endgenerate` - bare blocks "
        "directly in a generate region. IEEE 1800-2017 A.4.2 generate_region ::= generate { generate_item "
        "} endgenerate (docs/systemverilog/2017/md/section-41-data-read-api.md:1853), generate_item is a "
        "module_or_generate_item (docs/systemverilog/2017/md/section-41-data-read-api.md:338-345) with no "
        "bare-block arm; a generate_block is reached only through loop_generate_construct / "
        "conditional_generate_construct (docs/systemverilog/2017/md/section-41-data-read-api.md:1903); "
        "IEEE 1364-2005 A.4.2 identical (the Verilog-2001 bare-block form was removed). REJECTS at "
        "furthest_position=224 on sv_2017 and sv_2023. Control: `generate if (1) begin ... end "
        "endgenerate` ACCEPTS (arms a25/a25c). The golden's `duplicate declaration` is line 14 prose. "
        "Spec outranks the verilator driver key (fails=True + a post-parse golden reads as must_accept): "
        "the golden records the SEMANTIC error the test was written for, and the parser stops on text no "
        "Annex A production derives"),
    "test_regress/t/t_var_dup_bad.v": ("must_reject",
        "pinned .13x.b: lines 59-62 `module sub2 ( output bad_reout_port ); output bad_reout_port; "
        "endmodule` - an ANSI header followed by a BODY port declaration. IEEE 1800-2017 A.1.2 "
        "module_declaration: the ANSI arm's body is { non_port_module_item } "
        "(docs/systemverilog/2017/md/section-41-data-read-api.md:108-113) and `port_declaration ;` is a "
        "module_item of the non-ANSI arm only "
        "(docs/systemverilog/2017/md/section-41-data-read-api.md:327); 23.2.2.2 prose "
        "(`docs/systemverilog/2017/md/section-23-modules-and-hierarchy.md:407`) says the same. REJECTS at "
        "furthest_position=770 on sv_2017 and sv_2023. Controls: `module sub3 (p); output p; endmodule` "
        "ACCEPTS; `module sub1 (bad_mixport, output bad_mixport);` (lines 53-56) PARSES - "
        "ansi_port_declaration's header is optional and the first-port defaults are 23.2.2.3 prose "
        "(:477), so that module is not this file's syntax error (arms d3/d4/d4b/d4c). Spec outranks the "
        "verilator driver key (fails=True + a post-parse golden reads as must_accept): the golden records "
        "the SEMANTIC error the test was written for, and the parser stops on text no Annex A production "
        "derives"),
    # ---- SV-CORPUS-GRAD.3.51 (PGEN-SV-CORPUS-GRAD-0381): the residual bar triaged by production ----
    # Twenty-six rows whose text no edition derives, each with the production, the measured stop and a pin pair.
    'test_regress/t/t_param_no_parentheses.v': ('must_reject',
        "pinned .3.51: line 16 `m1 #PAR m1();` - a parameter value assignment WITHOUT parentheses. IEEE 1800-2017 A.4.1.1 parameter_value_assignment ::= # ( [ list_of_parameter_assignments ] ) (docs/systemverilog/2017/md/section-41-data-read-api.md:1795) - the parentheses are the production, a bare `#PAR` is a delay control no instantiation admits; IEEE 1800-2023 A.4.1.1 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1804). REJECTS at furthest_position=379 on sv_2017 and sv_2023 (the `P` after `#`). Controls (rust/target/s351/rep, release probe 1.0.223 a2ca4c477d54c216…): `m1 #(PAR) u1();` ACCEPTS on every profile, `m1 #PAR u1();` REJECTS at the byte after `#`. Pins invalid_parameter_value_without_parentheses.sv / control_parameter_value_parenthesised.sv. Verilator's driver expects success because Verilator accepts a single unparenthesised value; no edition derives it"),
    'test_regress/t/t_param.v': ('must_reject',
        'pinned .3.51: line 16 `mreal #1.2 mr ();` - a parameter value assignment WITHOUT parentheses (a real value). IEEE 1800-2017 A.4.1.1 parameter_value_assignment ::= # ( [ list_of_parameter_assignments ] ) (docs/systemverilog/2017/md/section-41-data-read-api.md:1795); IEEE 1800-2023 A.4.1.1 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1804). REJECTS at furthest_position=337 on sv_2017 and sv_2023 (the `1` after `#`). Controls (rust/target/s351/rep, release probe 1.0.223 a2ca4c477d54c216…): `mreal #(1.2) mr ();` ACCEPTS on every profile. Pins invalid_parameter_value_without_parentheses.sv / control_parameter_value_parenthesised.sv. the same Verilator tolerance as t_param_no_parentheses.v'),
    'test_regress/t/t_property_named.v': ('must_reject',
        "pinned .3.51: line 26 `property check_if_1(var int cyc_mod_2);` - `var` on a property formal. IEEE 1800-2017 A.2.10 property_port_item ::= { attribute_instance } [ local [ property_lvar_port_direction ] ] property_formal_type formal_port_identifier { variable_dimension } [ = property_actual_arg ] (docs/systemverilog/2017/md/section-41-data-read-api.md:1301-1304) with sequence_formal_type ::= data_type_or_implicit | sequence | untyped (docs/systemverilog/2017/md/section-41-data-read-api.md:1397-1401) - `var` is not a data_type; IEEE 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1340-1343, :1420-1424). REJECTS at furthest_position=550 on sv_2017 and sv_2023 (the `var`). Controls (rust/target/s351/rep, release probe 1.0.223 a2ca4c477d54c216…): `property check_if_1(int cyc_mod_2);` ACCEPTS on the SV profiles. Pins invalid_property_port_var_qualifier.sv / control_property_port_typed.sv. Verilator's driver expects success; the production has no var"),
    'test_regress/t/t_class_static_member_sel.v': ('must_reject',
        "pinned .3.51: line 74 `baz.get_bar().f.x = 4;` - a METHOD CALL inside an assignment target. IEEE 1800-2017 A.8.5 variable_lvalue ::= [ implicit_class_handle . | package_scope ] hierarchical_variable_identifier select | { variable_lvalue { , variable_lvalue } } | [ assignment_pattern_expression_type ] assignment_pattern_variable_lvalue | streaming_concatenation (docs/systemverilog/2017/md/section-41-data-read-api.md:3199-3204) - a hierarchical identifier with selects, never a call; IEEE 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:3146-3151). REJECTS at furthest_position=1229 on sv_2017 and sv_2023 (the `(` of the call). Controls (rust/target/s351/rep, release probe 1.0.223 a2ca4c477d54c216…): `c.x = 4;` ACCEPTS on the SV profiles. Pins invalid_method_call_in_lvalue.sv / control_member_lvalue.sv. Verilator's driver expects success; the production has no call"),
    'test_regress/t/t_delay.v': ('must_reject',
        "pinned .3.51: line 26 `assign #sub.delay dly4 = dly1 + 1;` - a HIERARCHICAL name as a delay value. IEEE 1800-2017 A.6.1 delay_value ::= unsigned_number | real_number | ps_identifier | time_literal | 1step (docs/systemverilog/2017/md/section-41-data-read-api.md:948-954) with A.9.3 ps_identifier ::= [ package_scope ] identifier (docs/systemverilog/2017/md/section-41-data-read-api.md:3404) - package-scoped at most, never hierarchical; IEEE 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:989-995, :3355). REJECTS at furthest_position=549 on sv_2017 and sv_2023 (the `.` after `sub`). Controls (rust/target/s351/rep, release probe 1.0.223 a2ca4c477d54c216…): `assign #pkg::delay a = b;` ACCEPTS on the SV profiles. Pins invalid_hierarchical_delay_value.sv / control_package_scoped_delay_value.sv. Verilator's driver expects success; a delay value is a ps_identifier"),
    'test_regress/t/t_lparam_assign_iface_typedef4.v': ('must_reject',
        "pinned .3.51: line 18 `localparam type p0_t = if0.rq_t[1:0];` - a type named THROUGH an interface instance. IEEE 1800-2017 A.2.2.1 data_type ::= ... | [ class_scope | package_scope ] type_identifier { packed_dimension } | ... (docs/systemverilog/2017/md/section-41-data-read-api.md:855; the rule docs/systemverilog/2017/md/section-41-data-read-api.md:842-866) - a type is reached through a class scope or a package scope, never a hierarchical path; IEEE 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:838; the rule docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:825-849). REJECTS at furthest_position=407 on sv_2017 and sv_2023 (the `.` after the instance name). Controls (rust/target/s351/rep, release probe 1.0.223 a2ca4c477d54c216…): `localparam type t = pkg::rq_t;` ACCEPTS on the SV profiles. Pins invalid_hierarchical_type_reference.sv / control_package_scoped_type_reference.sv. Verilator's extension (a type through an interface instance); no edition derives it"),
    'test_regress/t/t_lparam_assign_iface_typedef_nested.v': ('must_reject',
        "pinned .3.51: line 40 `localparam type p0_rq2_t = if0.y_if0.rq2_t;` - a type named THROUGH an interface instance. IEEE 1800-2017 A.2.2.1 data_type ::= ... | [ class_scope | package_scope ] type_identifier { packed_dimension } | ... (docs/systemverilog/2017/md/section-41-data-read-api.md:855; the rule docs/systemverilog/2017/md/section-41-data-read-api.md:842-866) - a type is reached through a class scope or a package scope, never a hierarchical path; IEEE 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:838; the rule docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:825-849). REJECTS at furthest_position=803 on sv_2017 and sv_2023 (the `.` after the instance name). Controls (rust/target/s351/rep, release probe 1.0.223 a2ca4c477d54c216…): `localparam type t = pkg::rq_t;` ACCEPTS on the SV profiles. Pins invalid_hierarchical_type_reference.sv / control_package_scoped_type_reference.sv. Verilator's extension (a type through an interface instance); no edition derives it"),
    'test_regress/t/t_lparam_assign_iface_typedef_nested6.v': ('must_reject',
        "pinned .3.51: line 38 `localparam type rq_t = if0.x_if0.rq_t;` - a type named THROUGH an interface instance. IEEE 1800-2017 A.2.2.1 data_type ::= ... | [ class_scope | package_scope ] type_identifier { packed_dimension } | ... (docs/systemverilog/2017/md/section-41-data-read-api.md:855; the rule docs/systemverilog/2017/md/section-41-data-read-api.md:842-866) - a type is reached through a class scope or a package scope, never a hierarchical path; IEEE 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:838; the rule docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:825-849). REJECTS at furthest_position=769 on sv_2017 and sv_2023 (the `.` after the instance name). Controls (rust/target/s351/rep, release probe 1.0.223 a2ca4c477d54c216…): `localparam type t = pkg::rq_t;` ACCEPTS on the SV profiles. Pins invalid_hierarchical_type_reference.sv / control_package_scoped_type_reference.sv. Verilator's extension (a type through an interface instance); no edition derives it"),
    'test_regress/t/t_param_type_from_iface_struct.v': ('must_reject',
        "pinned .3.51: line 72 `typedef ports[0].inner.req_t local_req_t;` - a type named THROUGH an interface instance. IEEE 1800-2017 A.2.2.1 data_type ::= ... | [ class_scope | package_scope ] type_identifier { packed_dimension } | ... (docs/systemverilog/2017/md/section-41-data-read-api.md:855; the rule docs/systemverilog/2017/md/section-41-data-read-api.md:842-866) - a type is reached through a class scope or a package scope, never a hierarchical path; IEEE 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:838; the rule docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:825-849). REJECTS at furthest_position=2160 on sv_2017 and sv_2023 (the `.` after the instance name). Controls (rust/target/s351/rep, release probe 1.0.223 a2ca4c477d54c216…): `localparam type t = pkg::rq_t;` ACCEPTS on the SV profiles. Pins invalid_hierarchical_type_reference.sv / control_package_scoped_type_reference.sv. Verilator's extension (a type through an interface instance); no edition derives it"),
    'test_regress/t/t_typedef_package.v': ('must_reject',
        "pinned .3.51: line 12 `typedef enum pkg1::uint8_t {` - a PACKAGE-SCOPED enum base type. IEEE 1800-2017 A.2.2.1 enum_base_type ::= integer_atom_type [ signing ] | integer_vector_type [ signing ] [ packed_dimension ] | type_identifier [ packed_dimension ] (docs/systemverilog/2017/md/section-41-data-read-api.md:868-873) - a bare type_identifier, where data_type spells [ class_scope | package_scope ] type_identifier (docs/systemverilog/2017/md/section-41-data-read-api.md:855); IEEE 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:851-856). REJECTS at furthest_position=293 on sv_2017 and sv_2023 (the `::`). Controls (rust/target/s351/rep, release probe 1.0.223 a2ca4c477d54c216…): `typedef enum logic [7:0] {{ a = 1 }} e_t;` ACCEPTS on the SV profiles. Pins invalid_enum_base_type_package_scoped.sv / control_enum_base_type_vector.sv. Verilator's extension; enum_base_type names a bare type_identifier in both editions"),
    # ---- SV-CORPUS-GRAD.3.56 (PGEN-SV-CORPUS-GRAD-0386): the remaining bar rows bisected ----
    'test_regress/t/t_cover_fsm_sel_assign.v': ('must_reject',
        'pinned .3.56: lines 29-34 `begin logic [D-1:0][W-1:0] s; begin always_ff … end end` - bare begin/end blocks as MODULE items. IEEE 1800-2017 A.1.4 module_item (docs/systemverilog/2017/md/section-41-data-read-api.md:327-330), non_port_module_item (docs/systemverilog/2017/md/section-41-data-read-api.md:356-362), module_or_generate_item (docs/systemverilog/2017/md/section-41-data-read-api.md:313-319) name no begin/end block (the Verilog-2001 unconditional generate block IEEE 1364-2005 removed). Isolated: `module m; begin logic s; … end endmodule` rejects @20 (pins)'),
    'test_regress/t/t_simulate_array.v': ('must_reject',
        'pinned .3.56: line 16 `module test (); begin localparam something = fun(); initial … end` - a bare begin/end block as a MODULE item. IEEE 1800-2017 A.1.4 module_item (docs/systemverilog/2017/md/section-41-data-read-api.md:327-330), non_port_module_item (docs/systemverilog/2017/md/section-41-data-read-api.md:356-362) name no begin/end block. Isolated: rejects @20 at `begin` (pins)'),
    'test_regress/t/t_gen_for.v': ('must_reject',
        'pinned .3.56: lines 65-66 `generate begin case (MODE) …` - a BARE begin/end block directly inside a generate region, the Verilog-2001 form IEEE 1364-2005 removed. IEEE 1800-2017 A.4.2 generate_region ::= generate { generate_item } endgenerate (docs/systemverilog/2017/md/section-41-data-read-api.md:1853-1855), generate_item (docs/systemverilog/2017/md/section-41-data-read-api.md:1911-1915); a generate_block (docs/systemverilog/2017/md/section-41-data-read-api.md:1903-1908) belongs only to a conditional or loop generate construct. Isolated (the .3.55 pin `invalid_bare_generate_block.sv`): rejects @20, the conditional form accepts'),
    'test_regress/t/t_interface_gen11.v': ('must_reject',
        'pinned .3.56: lines 29-33 `generate begin t1 t (.mod_intf(mod_intfs[0])); end endgenerate` - a BARE begin/end block inside a generate region. IEEE 1800-2017 A.4.2 generate_region (docs/systemverilog/2017/md/section-41-data-read-api.md:1853-1855), generate_item (docs/systemverilog/2017/md/section-41-data-read-api.md:1911-1915), generate_block (docs/systemverilog/2017/md/section-41-data-read-api.md:1903-1908). The .3.55 pin `invalid_bare_generate_block.sv` is the reproducer'),
    'test_regress/t/t_interface_gen13.v': ('must_reject',
        'pinned .3.56: lines 19-23 `generate begin initial p2 = 1; end endgenerate` - a BARE begin/end block inside a generate region. IEEE 1800-2017 A.4.2 generate_region (docs/systemverilog/2017/md/section-41-data-read-api.md:1853-1855), generate_item (docs/systemverilog/2017/md/section-41-data-read-api.md:1911-1915), generate_block (docs/systemverilog/2017/md/section-41-data-read-api.md:1903-1908). The .3.55 pin `invalid_bare_generate_block.sv` is the reproducer'),
    'test_regress/t/t_interface_gen6.v': ('must_reject',
        'pinned .3.56: line 28 `generate begin …` - a BARE begin/end block inside a generate region. IEEE 1800-2017 A.4.2 generate_region (docs/systemverilog/2017/md/section-41-data-read-api.md:1853-1855), generate_item (docs/systemverilog/2017/md/section-41-data-read-api.md:1911-1915), generate_block (docs/systemverilog/2017/md/section-41-data-read-api.md:1903-1908). The .3.55 pin `invalid_bare_generate_block.sv` is the reproducer'),
    'test_regress/t/t_covergroup_unsup.v': ('must_reject',
        'pinned .3.56: lines 66-68 `covergroup cg_bracket; {} endgroup` - a bare brace pair as a covergroup item (the file is Verilator\'s unsupported-construct bank). IEEE 1800-2017 A.2.11 covergroup_declaration (docs/systemverilog/2017/md/section-41-data-read-api.md:1495-1500), coverage_spec_or_option ::= { attribute_instance } coverage_spec | { attribute_instance } coverage_option ; (docs/systemverilog/2017/md/section-41-data-read-api.md:1502-1506). Isolated: rejects @26 at `{`, `coverpoint x;` accepts (pins)'),
    'test_regress/t/t_udp_nonsequential.v': ('must_reject',
        'pinned .3.56: lines 66-67 `0   0  x \\` newline `: 0 ;` - a backslash-newline inside a UDP table row (the file\'s own comment: intentional for parser). IEEE 1800-2017 A.5.4 sequential_entry ::= seq_input_list : current_state : next_state ; (docs/systemverilog/2017/md/section-41-data-read-api.md:1979), level_input_list (docs/systemverilog/2017/md/section-41-data-read-api.md:1981), combinational_entry (docs/systemverilog/2017/md/section-41-data-read-api.md:1975) - no continuation; 22.5.1\'s backslash-newline (docs/systemverilog/2017/md/section-22-compiler-directives.md:171-174) is macro text only. Isolated: rejects @81, the plain rows accept (pins)'),
    'test_regress/t/t_pipe_filter.v': ('must_reject',
        'pinned .3.56: lines 11-12 `example line 10;` `example line 11;` at the top level - placeholder text the driver\'s +pipe-filter rewrites before compilation. IEEE 1800-2017 A.1.2 description (docs/systemverilog/2017/md/section-41-data-read-api.md:48-56) derives no such item. Isolated: rejects @7 (pins)'),
    'test_regress/t/t_gate_primitives_implicit_net.v': ('must_reject',
        'pinned .3.56: line 53 `` `default_nettype none `` inside `module t` - the directive inside a design element. IEEE 1800-2017 22.8 (docs/systemverilog/2017/md/section-22-compiler-directives.md:672): `default_nettype can be used only outside design elements. Isolated: `module m; wire a; `default_nettype none …` rejects @19, the same directive before the module accepts (pins)'),
    # ---- SV-CORPUS-GRAD.3.55 (PGEN-SV-CORPUS-GRAD-0385): the port / generate group bisected to its one variable ----
    'test_regress/t/t_math_pow6.v': ('must_reject',
        'pinned .3.55: line 21 `output [3:0] i65 = 65\'d3 ** a;` - an INITIALIZER on a NON-ANSI NET output port (the header `module t(i65, …, a, …);` is non-ANSI). IEEE 1800-2017 A.2.1.2 output_declaration ::= output net_port_type list_of_port_identifiers | output variable_port_type list_of_variable_port_identifiers (docs/systemverilog/2017/md/section-41-data-read-api.md:762-766); A.2.3 list_of_port_identifiers (docs/systemverilog/2017/md/section-41-data-read-api.md:975-978) has no initializer - only list_of_variable_port_identifiers (docs/systemverilog/2017/md/section-41-data-read-api.md:994-997) does, and `output [3:0]` with no data type is a net port (23.2.2.3: it shall be illegal to initialize a port that is not a variable output port). Isolated: `output [3:0] i65 = 4\'d3;` rejects @53, `output logic [3:0] i65 = 4\'d3;` and `output var [3:0] i65 = 4\'d3;` accept (pins)'),
    # ---- SV-CORPUS-GRAD.3.54 (PGEN-SV-CORPUS-GRAD-0384): the class / scope group bisected to its one variable ----
    'test_regress/t/t_std_randomize_with.v': ('must_reject',
        'pinned .3.54: line 41 `std::randomize( a, obj.x ) with { ... }` - a MEMBER SELECT as a scope-randomize argument. IEEE 1800-2017 18.12 Syntax 18-11 scope_randomize ::= [ std :: ] randomize ( [ variable_identifier_list ] ) [ with constraint_block ] (docs/systemverilog/2017/md/section-18-constrained-random-value-generation.md:1949-1952); A.9.3 variable_identifier_list ::= variable_identifier { , variable_identifier } (docs/systemverilog/2017/md/section-41-data-read-api.md:2124), variable_identifier ::= identifier (docs/systemverilog/2017/md/section-41-data-read-api.md:3452) - an argument is an IDENTIFIER, never a select. Test 1 (`a, b`, line 28) parses; the stop is Test 2. Isolated: `std::randomize(a, obj.x) with { ... }` rejects @110, `std::randomize(a, b) with { ... }` accepts (pins)'),
    'test_regress/t/t_randomize_arraysel_membersel.v': ('must_reject',
        'pinned .3.54: line 32 `std::randomize(example.member[1]) with { ... }` - an INDEXED member select as a scope-randomize argument. IEEE 1800-2017 18.12 Syntax 18-11 (docs/systemverilog/2017/md/section-18-constrained-random-value-generation.md:1949-1952); A.9.3 variable_identifier_list (docs/systemverilog/2017/md/section-41-data-read-api.md:2124), variable_identifier ::= identifier (docs/systemverilog/2017/md/section-41-data-read-api.md:3452). Isolated: rejects @128 at the `.` (pins)'),
    'test_regress/t/t_class_extern.v': ('must_reject',
        'pinned .3.54: line 12 `extern function ext_f_imp(in);` - a function PROTOTYPE with no return type. IEEE 1800-2017 A.2.6 function_prototype ::= function data_type_or_void function_identifier [ ( [ tf_port_list ] ) ] (docs/systemverilog/2017/md/section-41-data-read-api.md:1105) - the data_type_or_void is not optional (the implicit-type form belongs to function_body_declaration, A.2.6, not to a prototype); A.1.9 class_method ::= ... | extern { method_qualifier } method_prototype ; (docs/systemverilog/2017/md/section-41-data-read-api.md:570-577), method_prototype ::= task_prototype | function_prototype (docs/systemverilog/2017/md/section-41-data-read-api.md:616-619). Lines 9-11 (`extern function int ...`) parse. Isolated: `class Cls; extern function ext_f_imp(in); endclass` rejects @38, the typed prototypes accept (pins)'),
    'test_regress/t/t_interface_modport_export.v': ('must_reject',
        'pinned .3.54: line 12 `extern function myfunc (input logic val);` in an interface - an extern_tf_declaration whose function_prototype has no return type. IEEE 1800-2017 A.2.6 extern_tf_declaration ::= extern method_prototype ; | extern forkjoin task_prototype ; (docs/systemverilog/2017/md/section-41-data-read-api.md:443-447), function_prototype ::= function data_type_or_void function_identifier [ ( [ tf_port_list ] ) ] (docs/systemverilog/2017/md/section-41-data-read-api.md:1105). Isolated: rejects @43 at the `(`, `extern function logic myfunc (input logic val);` accepts (pins)'),
    # ---- SV-CORPUS-GRAD.3.52 (PGEN-SV-CORPUS-GRAD-0382): the residual bar triaged by production, batch two ----
    'test_regress/t/t_foreach_array.v': ('must_reject',
        'pinned .3.52: line 64 `foreach (queue[i]) foreach (queue[i][j]) exp_count_que++;` - a foreach whose loop variables are spelled ONE PER BRACKET PAIR. IEEE 1800-2017 A.6.8 foreach ( ps_or_hierarchical_array_identifier [ loop_variables ] ) statement (docs/systemverilog/2017/md/section-41-data-read-api.md:2334) with loop_variables ::= [ index_variable_identifier ] { , [ index_variable_identifier ] } (docs/systemverilog/2017/md/section-41-data-read-api.md:2360) - ONE bracket pair holding a comma-separated list, as 12.7.3 says ("a comma-separated list of loop variables enclosed in square brackets", docs/systemverilog/2017/md/section-12-procedural-programming-statements.md:1044-1045; examples `foreach( prod[ k, m ] )` docs/systemverilog/2017/md/section-12-procedural-programming-statements.md:1058, `foreach( A [ i, j, k ] )` docs/systemverilog/2017/md/section-12-procedural-programming-statements.md:1082); IEEE 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:2319, :2345; 12.7.3 docs/systemverilog/2023/md/section-12-procedural-programming-statements.md:1037-1043, :1054, :1078). REJECTS at furthest_position=2532 on sv_2017 and sv_2023 (the second `[`). Controls (rust/target/s352/rep, release probe 1.0.223 a2ca4c477d54c216…): `foreach (q[i, j]) n++;` ACCEPTS on the SV profiles (the file\'s own line 62 writes `foreach (queue[i, j])`). Pins invalid_foreach_bracket_per_variable.sv / control_foreach_comma_list.sv. Verilator accepts the bracket-per-variable spelling; the production holds one bracket pair'),
    'test_regress/t/t_foreach_nested.v': ('must_reject',
        'pinned .3.52: line 14 `foreach(this.foo.x[i][j])` - a foreach whose loop variables are spelled ONE PER BRACKET PAIR (with an implicit class handle, which A.9.3 ps_or_hierarchical_array_identifier admits). IEEE 1800-2017 A.6.8 foreach ( ps_or_hierarchical_array_identifier [ loop_variables ] ) statement (docs/systemverilog/2017/md/section-41-data-read-api.md:2334) with loop_variables ::= [ index_variable_identifier ] { , [ index_variable_identifier ] } (docs/systemverilog/2017/md/section-41-data-read-api.md:2360) - ONE bracket pair holding a comma-separated list, as 12.7.3 says ("a comma-separated list of loop variables enclosed in square brackets", docs/systemverilog/2017/md/section-12-procedural-programming-statements.md:1044-1045; examples `foreach( prod[ k, m ] )` docs/systemverilog/2017/md/section-12-procedural-programming-statements.md:1058, `foreach( A [ i, j, k ] )` docs/systemverilog/2017/md/section-12-procedural-programming-statements.md:1082); IEEE 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:2319, :2345; 12.7.3 docs/systemverilog/2023/md/section-12-procedural-programming-statements.md:1037-1043, :1054, :1078). REJECTS at furthest_position=348 on sv_2017 and sv_2023 (the second `[`). Controls (rust/target/s352/rep, release probe 1.0.223 a2ca4c477d54c216…): `foreach (this.x[i, j]) …` ACCEPTS on the SV profiles. Pins invalid_foreach_bracket_per_variable.sv / control_foreach_comma_list.sv. the same Verilator spelling as t_foreach_array.v'),
    'test_regress/t/t_constraint_operators.v': ('must_reject',
        "pinned .3.52: line 52 `constraint unary { !(-~c == 'h22); }` - a unary operator applied to a UNARY EXPRESSION (`-~c`). IEEE 1800-2017 A.8.3 expression ::= primary | unary_operator { attribute_instance } primary | inc_or_dec_expression | ( operator_assignment ) | expression binary_operator { attribute_instance } expression | … (docs/systemverilog/2017/md/section-41-data-read-api.md:3026-3034) - a unary operator takes a PRIMARY and `~c` is an expression, not a primary (A.8.4 primary docs/systemverilog/2017/md/section-41-data-read-api.md:3122); `-(~c)` is the derivable spelling; IEEE 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:2988-2996, docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:3079). REJECTS at furthest_position=1423 on sv_2017 and sv_2023 (the `~`). Controls (rust/target/s352/rep, release probe 1.0.223 a2ca4c477d54c216…): `!(-(~c) == 'h22)` ACCEPTS on the SV profiles. Pins invalid_unary_on_unary_expression.sv / control_unary_on_parenthesised.sv. Verilator's driver expects success; the production takes a primary"),
    'test_regress/t/t_inst_port_complex_unsup.v': ('must_reject',
        'pinned .3.52: line 10 `output wire .ao_rename(ao),  .bo_rename(bo)` - a NET TYPE on an ANSI port-expression declaration. IEEE 1800-2017 A.1.3 ansi_port_declaration ::= … | [ port_direction ] . port_identifier ( [ expression ] ) (docs/systemverilog/2017/md/section-41-data-read-api.md:292-298) - the port-expression alternative takes a direction only, never a net type; IEEE 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:265-271). REJECTS at furthest_position=319 on sv_2017 and sv_2023 (the `.` after `wire`). Controls (rust/target/s352/rep, release probe 1.0.223 a2ca4c477d54c216…): `module m(output .ao_rename(ao));` ACCEPTS on the SV profiles. Pins invalid_ansi_port_expression_with_net_type.sv / control_ansi_port_expression.sv. the file is `_unsup` for Verilator itself; the third alternative has no net_port_header'),
    'test_regress/t/t_setuphold.v': ('must_reject',
        "pinned .3.52: line 46 `$setuphold (posedge clk, negedge d, 0:0:0, 0:0:0);` - a min:typ:max triple as a TIMING CHECK LIMIT. IEEE 1800-2017 A.7.5.1 $setuphold_timing_check ::= $setuphold ( reference_event , data_event , timing_check_limit , timing_check_limit [ , … ] ) ; (docs/systemverilog/2017/md/section-41-data-read-api.md:2732-2735) with A.7.5.3 timing_check_limit ::= expression (docs/systemverilog/2017/md/section-41-data-read-api.md:2817) - an expression, and a mintypmax_expression is not one (A.8.3 docs/systemverilog/2017/md/section-41-data-read-api.md:3026); IEEE 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:2709-2712, :2794). REJECTS at furthest_position=1237 on sv_2017 and sv_2023 (the first `:` of `0:0:0`). Controls (rust/target/s352/rep, release probe 1.0.223 a2ca4c477d54c216…): `$setuphold (posedge clk, negedge d, 0, 0);` ACCEPTS on every profile. Pins invalid_timing_check_limit_mintypmax.sv / control_timing_check_limit_expression.sv. Verilator's driver expects success; a timing_check_limit is an expression"),
    'test_regress/t/t_event_control_pass.v': ('must_reject',
        "pinned .3.52: line 21 `->b.get_event();` - a method CALL as an event trigger target. IEEE 1800-2017 A.6.5 event_trigger ::= -> hierarchical_event_identifier ; | ->> [ delay_or_event_control ] hierarchical_event_identifier ; (docs/systemverilog/2017/md/section-41-data-read-api.md:2191-2194) with A.9.3 hierarchical_identifier (docs/systemverilog/2017/md/section-41-data-read-api.md:3361) - no call; IEEE 1800-2023 the same with a trailing nonrange_select (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:2189-2192), still no call. REJECTS at furthest_position=427 on sv_2017 and sv_2023 (the `(`). Controls (rust/target/s352/rep, release probe 1.0.223 a2ca4c477d54c216…): `->b.e;` ACCEPTS on the SV profiles. Pins invalid_event_trigger_on_method_call.sv / control_event_trigger_hierarchical.sv. Verilator's driver expects success; the target is a hierarchical event identifier"),
}


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
            # .3.24: the PARSE-STAGE vocabulary, not just the literal "syntax error" —
            # verilator's lexer errors never carry those two words. See
            # VERILATOR_PARSE_STAGE_RE for the enumeration and for the four spellings
            # deliberately left out (three preprocessor, one `-f` command file).
            if VERILATOR_PARSE_STAGE_RE.search(read_text(out)):
                self.syntax_out_stems.add(out.stem)

    def expect(self, relpath: str):
        # relpath is relative to the verilator submodule root.
        # SV-CORPUS-GRAD.3.14a: the LRM pins run FIRST - they override the driver
        # heuristic below, which is tool testimony rather than a conformance claim.
        pinned = VERILATOR_PINNED.get(relpath.replace("\\", "/"))
        if pinned:
            return pinned
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
                        f".out reports a PARSE-STAGE error (.3.24: the parser's "
                        f"'syntax error', or a LEXER message from the enumerated "
                        f"vocabulary - unterminated string/attribute/block comment, "
                        f"version-control conflict marker)")
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
    "ieee-1800-2012/06/06.07.01_02.sv": ("must_reject",
        "pinned .3.55: lines 8-10 `module top; tri reg r; inout wire reg p; endmodule` - a PORT DECLARATION in the body of a module whose header lists no ports. IEEE 1800-2017 A.1.2 module_declaration ::= module_nonansi_header { module_item } endmodule | module_ansi_header { non_port_module_item } endmodule (docs/systemverilog/2017/md/section-41-data-read-api.md:108-113); module_nonansi_header REQUIRES list_of_ports (docs/systemverilog/2017/md/section-41-data-read-api.md:96-99) and module_ansi_header takes [ list_of_port_declarations ] (docs/systemverilog/2017/md/section-41-data-read-api.md:102-105), so `module top;` is an ANSI header whose body admits non_port_module_item only (docs/systemverilog/2017/md/section-41-data-read-api.md:356-362) - the parser stops at 168, the `inout`. "
        "The .8b.2 reading (A.2.1.3 net_declaration admits any data_type; the reg-on-net ban of 6.7.1 is prose) keyed the row must_accept on line 9 alone; the file is `// ! TYPE: NEGATIVE`. Isolated: `module top; inout wire reg p; endmodule` rejects @11, `module top(p); inout wire reg p; endmodule` accepts (pins)"),
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
    "ieee-1800-2012/08/08.26.04_01.sv": ("must_reject",
        "pinned .3.54: line 13 `virtual function bit[1:0] funcD();` inside `class ClassB` - a VIRTUAL method with NO body and neither `pure` nor `extern`. "
        "IEEE 1800-2017 A.1.9 class_method ::= { method_qualifier } function_declaration | pure virtual { class_item_qualifier } method_prototype ; | extern { method_qualifier } method_prototype ; (docs/systemverilog/2017/md/section-41-data-read-api.md:570-577) - "
        "a function_declaration carries a body and ends with endfunction (A.2.6), a bare prototype is admitted only behind `pure virtual` or `extern`; the file is `// ! TYPE: NEGATIVE`. "
        "The .8b.2 reading (8.26.4 `implements` of a forward-typedef'd interface class, a declaration-order matter) keyed the row must_accept without reading line 13; the parser stops at 263, the `endclass` where the body should start. "
        "Isolated: `class ClassB; virtual function bit[1:0] funcD(); endclass` rejects @66, the `pure virtual` interface-class form and the bodied form accept (pins)"),
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


# SV-CORPUS-GRAD.3.15 -- the bare generate_block family, verilog_2005 lane.
#
# The SAME construct as the EXTRA_PINNED block above, judged against the OTHER edition.
# IEEE 1364-2005 A.4.2 as extracted in grammars/verilog_2005_lrm_extracted.ebnf:452-462:
#
#   generate_region ::= generate { module_or_generate_item } endgenerate
#   generate_block  ::= module_or_generate_item
#                     | begin [ : generate_block_identifier ]
#                         { module_or_generate_item } end
#
# `generate_block` is reachable only from loop_generate_construct (:714),
# if_generate_construct (:534) and case_generate_item (:74) -- `module_or_generate_item`
# (:749) has no bare-block alternative, so the verdict does NOT change with the edition.
#
# ⛔ THESE SIX WERE FOUND ONLY BECAUSE THE SWEEP WAS PARAMETERIZED BY LANE. The leaf was
# cut from the sv_2017 axis-2 worklist, none of that lane's 19 files appear here, and
# stopping at the lane the task named would have shipped a half fix that looked complete.
# The tracked IEEE 1364-2005 Annex A extraction every v2005 pin below cites by line. Named
# once so a re-extraction that moves the file has ONE place to update, not seven.
V2005_ANNEX_A = "docs/verilog/2005/txt/section-Annex_A-normative-formal-syntax-definition.txt"

V2005_LRM_PINNED = {
    # ---- SV-CORPUS-GRAD.3.47 (PGEN-SV-CORPUS-GRAD-0377) ------------------------------------------
    # A string literal continued across a line with a trailing backslash is IEEE 1800 §5.9; IEEE 1364-2005
    # §3.6 says "contained on a single line" and A.8.8 spells the string as any characters EXCEPT a new line.
    "ivtest/ivltests/string12.v": (
        "must_reject", "pinned .3.47: line 4 `if (\"this matches\" == \"this\\` + newline + ` matches\")` - a string literal "
        "continued across a line break. IEEE 1364-2005 3.6 (docs/verilog/2005/md/section-3-lexical-conventions.md:299): "
        "'A string is a sequence of characters enclosed by double quotes and contained on a single line'; A.8.8 "
        "(docs/verilog/2005/md/section-Annex_A-normative-formal-syntax-definition.md:1218) `string ::= \" { "
        "Any_ASCII_Characters_except_new_line } \"` - no backslash-newline continuation exists in 1364-2005; it is IEEE "
        "1800-2017 5.9 (docs/systemverilog/2017/md/section-32-h-12ab-f001.md:101-102), an EDITION contrast between the "
        "Verilog and the SystemVerilog lanes. REJECTS under verilog_2005 before and after 1.0.221 (the 1364-2005 twin of "
        "the string terminal admits `\\` + any character except a newline), ACCEPTS under sv_2017 / sv_2023 since 1.0.221. "
        "Control: control_string_line_continuation.sv under the SV profiles; the pin invalid_string_line_continuation_v2005.sv. "
        "Spec outranks the ivtest regress-vlg.list 'normal' key, a claim about the iverilog plain-Verilog DIALECT"),
    # ---- SV-CORPUS-GRAD.13k(a)+(c) (PGEN-SV-CORPUS-GRAD-0327) ------------------------------
    # The COMPOUND ASSIGNMENT class. `.13k` opened because this manifest adjudicated ONE construct
    # class both ways: `br1015a.v` is `must_reject` on the ground that `^=` "exists in neither
    # A.6.2 blocking_assignment nor A.8.6", while `|=`, `+=` and `-=` carried `must_accept` on a
    # SUITE's word — "compiles under the iverilog plain-Verilog dialect" and "sv2v GOLDEN, valid
    # Verilog-2005 output by suite contract". A suite's contract is evidence about the SUITE.
    #
    # ⛔ THE RULING (c): `must_reject`, on this module's own taxonomy — "the text has no derivation
    # in the LRM's grammar … covers BOTH the suite's own intentional invalidity AND text a vendor
    # TOLERATES that the standard cannot derive … where the spec outranks the upstream tool". Not
    # `divergence:explained_*`, whose whole vocabulary is svpp/timeout CAUSES, none of which apply:
    # nothing about preprocessing or a deadline explains a token 1364-2005 does not have. Making
    # them `match` states the parser is CORRECT WITH A CITE rather than leaving a correct reject
    # sitting in the defect column.
    #
    # ⛔ RE-DERIVED IN-REPO, FOUR WAYS, NEVER FROM WHAT PGEN DOES:
    #   (1) IEEE 1364-2005 Annex A carries NO compound assignment operator AT ALL — 0 hits for
    #       `+= -= *= /= %= &= |= ^= <<= >>=` over the whole tracked extract
    #       (docs/verilog/2005/txt/section-Annex_A-…txt, docs/verilog/2005/grammar_clean.ebnf and
    #       grammars/verilog_2005_lrm_extracted.ebnf all agree at 0). The only assignment forms are
    #       `blocking_assignment ::= variable_lvalue = [ delay_or_event_control ] expression`
    #       (:518), `variable_assignment ::= variable_lvalue = expression` (:527), and the for-step,
    #       which IS a `variable_assignment` (`loop_statement`, :617-622) — so `i -= 1` in a `for`
    #       header is refused by the same clause as `i |= 1` in a body.
    #   (2) IEEE 1364-2005 PROSE carries none either: 0 hits across all 37 docs/verilog/2005/md
    #       sections. The only `+=` / `-=` bytes in that standard are the clause-14 specify-path
    #       POLARITY operators `+=>` / `-=>` (section-14-specify-blocks.md:638, :652) — a different
    #       token in a different role.
    #   (3) IEEE 1800 DOES define them, which is what makes this an EDITION contrast rather than a
    #       gap: `operator_assignment ::= variable_lvalue assignment_operator expression` and
    #       `assignment_operator ::= = | += | -= | *= | /= | %= | &= | |= | ^= | <<= | >>= | <<<= |
    #       >>>=` — 1800-2017 §10.4.1 (docs/systemverilog/2017/txt/section-10-assignment-statements.txt:288)
    #       and §11.4.1 (section-11-operators-and-expressions.txt:100, tagged "from A.6.2");
    #       1800-2023 in grammars/systemverilog_2023_lrm_extracted.ebnf:58.
    #   (4) THE SHIPPED PARSER ALREADY RULED THIS, UNDER RELEASE 1.0.197 / LEDGER `SV-0071`:
    #       grammars/systemverilog.ebnf:6938 gates all twelve compound-assignment terminals out of
    #       `verilog_2005` with that same A.6.2 citation, two-sided adjudicated, and its comment
    #       independently records the `+=>` / `-=>` caveat found again at (2). So these rows are an
    #       answer key contradicting a SHIPPED, LEDGERED conformance decision — not a parser gap.
    # MEASURED per row (release probe): each file REJECTS under `--profile verilog_2005` with
    # `furthest_position` landing exactly on the operator, and the minimal reproducer ACCEPTS under
    # `sv_2017` AND `sv_2023` — the edition contrast, measured. TWO-SIDED CONTROL per row: the
    # 1364-2005 spelling of the same intent (`x = x | y`, `x = x + 1`, `i = i - 1`) ACCEPTS on all
    # three profiles, isolating exactly one difference.
    "ivtest/ivltests/br_gh531.v": (
        "must_reject", "pinned .13k(a): lines 39 and 48 `tmp[3:0] |= i[3:0];` - IEEE 1364-2005 has "
        "NO compound assignment operator: Annex A "
        "(" + V2005_ANNEX_A + ":518 blocking_assignment, :527 variable_assignment) admits `=` "
        "alone, and 0 of `+= -= *= /= %= &= |= ^= <<= >>=` occur anywhere in the Annex A extract or "
        "in the 37 clause files. `|=` is IEEE 1800 A.6.2 assignment_operator (1800-2017 10.4.1, "
        "docs/systemverilog/2017/txt/section-10-assignment-statements.txt:289) - an EDITION "
        "contrast, and the shipped grammar already gates the twelve compound operators out of "
        "verilog_2005 under ledger SV-0071. Parse stops at furthest_position=635, exactly the `|=`. "
        "Control: `tmp[3:0] = tmp[3:0] | i[3:0]` ACCEPTS on all three profiles. Spec outranks the "
        "ivtest regress-vlg.list 'normal' key, which is a claim about the iverilog plain-Verilog "
        "DIALECT and not about 1364-2005"),
    "test/core/interface_generate.v": (
        "must_reject", "pinned .13k(a): line 19 `counter += 1;` - IEEE 1364-2005 has NO compound "
        "assignment operator (" + V2005_ANNEX_A + ":518/:527 admit `=` alone; 0 occurrences of the "
        "class in Annex A or the clause files - the only `+=` in that standard is the clause-14 "
        "specify-path polarity operator `+=>`, a different token). `+=` is IEEE 1800 A.6.2 "
        "assignment_operator; the shipped grammar gates it out of verilog_2005 under SV-0071. Parse "
        "stops at furthest_position=403, exactly the `+=`. Control: `counter = counter + 1` ACCEPTS "
        "on all three profiles. A sv2v GOLDEN file is what sv2v EMITS, which is testimony about the "
        "converter and not about the standard - the file is genuinely not 1364-2005"),
    "test/core/unpacked_localparam.v": (
        "must_reject", "pinned .13k(a): lines 5 and 7 `for (i = 3; i >= 0; i -= 1)` and "
        "`for (j = 0; j < 8; j += 1)` - IEEE 1364-2005 A.6.8 loop_statement "
        "(" + V2005_ANNEX_A + ":617-622) spells the for-STEP as a `variable_assignment`, which is "
        "`variable_lvalue = expression` (:527) - `=` alone, so a compound operator has no "
        "derivation in the header either. IEEE 1800 A.6.8 for_step_assignment admits "
        "operator_assignment; edition contrast, gated under SV-0071. Parse stops at "
        "furthest_position=149, exactly the `-=`. Control: `for (i = 3; i >= 0; i = i - 1)` ACCEPTS "
        "on all three profiles. sv2v GOLDEN = the converter's own output, not a conformance claim"),
    "ivtest/ivltests/br988.v": (
        "must_reject", "pinned .3.15: bare begin/end generate_block at line 7 - "
        "IEEE 1364-2005 A.4.2 reaches a generate_block only from "
        "loop/if/case generate constructs; generate_region and module_or_generate_item "
        "have no such alternative; spec outranks the ivtest driver key"),
    "ivtest/ivltests/pr1704726a.v": (
        "must_reject", "pinned .3.15: bare begin/end generate_block at line 39 - "
        "IEEE 1364-2005 A.4.2 reaches a generate_block only from "
        "loop/if/case generate constructs; generate_region and module_or_generate_item "
        "have no such alternative; spec outranks the ivtest driver key"),
    "ivtest/ivltests/pr1704726c.v": (
        "must_reject", "pinned .3.15: bare begin/end generate_block at line 517 - "
        "IEEE 1364-2005 A.4.2 reaches a generate_block only from "
        "loop/if/case generate constructs; generate_region and module_or_generate_item "
        "have no such alternative; spec outranks the ivtest driver key"),
    "ivtest/ivltests/pr1704726d.v": (
        "must_reject", "pinned .3.15: bare begin/end generate_block at line 82 - "
        "IEEE 1364-2005 A.4.2 reaches a generate_block only from "
        "loop/if/case generate constructs; generate_region and module_or_generate_item "
        "have no such alternative; spec outranks the ivtest driver key"),
    "ivtest/ivltests/pr2257003.v": (
        "must_reject", "pinned .3.15: bare begin/end generate_block at line 10 - "
        "IEEE 1364-2005 A.4.2 reaches a generate_block only from "
        "loop/if/case generate constructs; generate_region and module_or_generate_item "
        "have no such alternative; spec outranks the ivtest driver key"),
    "ivtest/ivltests/pr2257003b.v": (
        "must_reject", "pinned .3.15: bare begin/end generate_block at line 4 - "
        "IEEE 1364-2005 A.4.2 reaches a generate_block only from "
        "loop/if/case generate constructs; generate_region and module_or_generate_item "
        "have no such alternative; spec outranks the ivtest driver key"),
    # ---- SV-CORPUS-GRAD.13c.2k (PGEN-SV-CORPUS-GRAD-0237) ------------------------------------
    # Three rows that PGEN accepted only through the raw-`identifier` hole. Closing it turned
    # each into a REJECT, and each rejection is CORRECT under IEEE 1364-2005 — verified by
    # reading the tracked LRM, not from the ivtest driver key, which is upstream TOOL testimony
    # and is outranked by a clause cite (.3.14a/.3.14b/.3.15).
    "ivtest/ivltests/pr1758122.v": (
        "must_reject", "pinned .13c.2k: line 18 `adder #(W) instance(A[j], B[j], Y[j]);` names "
        "a module instance `instance`, which IEEE 1364-2005 Annex B:59 lists as a KEYWORD - and "
        "that annex's own preamble says keywords are predefined NONESCAPED identifiers and only "
        "an ESCAPED identifier is not treated as one. A.4.1.1 name_of_instance therefore has no "
        "derivation for it. Reserved in IEEE 1800-2023 Annex B:113 too, so the verdict does not "
        "change with the edition. PGEN accepted this until the raw-`identifier` hole closed"),
    "ivtest/ivltests/pr1787423.v": (
        "must_reject", "pinned .13c.2k: line 15 `pulldown (pd1, pd2);` gives a pull gate TWO "
        "output terminals. IEEE 1364-2005 A.3.1 "
        "(docs/verilog/2005/txt/section-Annex_A-normative-formal-syntax-definition.txt:362) is "
        "`pull_gate_instance ::= [ name_of_gate_instance ] ( output_terminal )` - exactly ONE. "
        "The legal spelling of the same intent is `pulldown (pd1), (pd2);`. PGEN reached "
        "udp_instantiation instead until the raw-`identifier` hole closed, because `pulldown` "
        "could stand in for a UDP type name. Control: pr1787423c.v carries no pull gate at all "
        "and did NOT move"),
    "ivtest/ivltests/pr1787423b.v": (
        "must_reject", "pinned .13c.2k: line 68 `pulldown (pd1, pd2);` - the same construct as "
        "pr1787423.v, same clause (IEEE 1364-2005 A.3.1:362, ONE output_terminal), same ruling"),
    # ---- SV-CORPUS-GRAD.13e.2 (PGEN-SV-CORPUS-GRAD-0284) ------------------------------------
    # The seven v2005-lane `unexplained_rejects_valid` rows that leaf .13e.1 SURFACED when 147
    # rows stopped being silent. Each was adjudicated against the tracked IEEE 1364-2005 LRM
    # (Annex A + the clause bodies), never against what PGEN does, and each carries a MEASURED
    # accepting control isolating exactly one difference (Protocol B step 4).
    #
    # ⛔ THE FILE NAMES SUGGESTED THREE CLUSTERS AND MEASUREMENT REFUTED TWO OF THEM. The real
    # population is FIVE constructs, and the `partsel_outside_*` pair is not about part-selects
    # at all: `arr[-'d1 +: 2]` PARSES (control g_partsel), and both files stop on an unbased
    # unsized `'x` in their self-checking `if`. Naming a cluster from a filename is a guess.
    #
    # ⭐ THREE of the seven are keyed `type: "CE"` upstream — the file is EXPECTED not to
    # compile.
    # They reached this lane because `_gold_has_syntax_error()` searches the golden for the
    # literal words "syntax error" and iverilog says something else ("is an Icarus Verilog
    # extension", "Empty UDP table."). Two of those three descriptors ALSO carry an explicit
    # non-standard extension flag in their own `iverilog-args` (`-gxtypes`, `-gno-icarus-misc`)
    # that `effective_gen()`/`gen_source()` do not read, so their basis line says "the compiler's
    # own default" about a row the descriptor configured by hand — routed to .13e.3.
    "ivtest/ivltests/br_gh1087b.v": (
        "must_reject", "pinned .13e.2: line 3 `wire bool [7:0] b;` gives a NET a data type. "
        "IEEE 1364-2005 A.2.1.3 net_declaration (" + V2005_ANNEX_A + ":153) admits only "
        "`net_type [signed] [delay3] list_of_net_identifiers ;` or a form whose `range` follows "
        "the net_type directly - there is no net data type, so `bool` can only fill "
        "net_identifier and `[7:0]` its dimension (A.2.3:221), leaving `b` with no derivation, "
        "which is exactly where PGEN stops. `bool` is a CADENCE EXTENDED TYPE the descriptor "
        "enables ITSELF (`iverilog-args: [\"-gxtypes\"]`), and the golden is an ELABORATION "
        "error - upstream's parser accepted it only with a non-standard extension switched on. "
        "Control: `wire [7:0] b;` parses"),
    "ivtest/ivltests/br_gh552.v": (
        "must_reject", "pinned .13e.2: lines 8-9 use `~&` and `~|` as BINARY operators. "
        "IEEE 1364-2005 A.8.6 binary_operator (" + V2005_ANNEX_A + ":954) lists neither; both "
        "appear only "
        "under unary_operator (:952) and unary_module_path_operator (:957). Upstream says the "
        "same in its own words - the descriptor passes `-gno-icarus-misc` and the golden reads "
        "`The binary NAND operator is an Icarus Verilog extension. Use -gicarus-misc to enable "
        "it.` Control: `a & b` parses"),
    "ivtest/ivltests/partsel_outside_const.v": (
        "must_reject", "pinned .13e.2: line 15 `outside_const !== 'x` is an UNBASED UNSIZED "
        "literal. IEEE 1364-2005 A.8.7 decimal_number (" + V2005_ANNEX_A + ":978) admits "
        "`x_digit` only "
        "after a `decimal_base`, and binary_/octal_/hex_number (:983-985) each require their own "
        "base, so bare `'x` has no derivation - it is SystemVerilog's unbased_unsized_literal. "
        "The vendored compiler agrees against itself: stimuli/sv/subs/iverilog/lexor.lex:516 "
        "warns `Using SystemVerilog 'N bit vector. Use at least -g2005-sv` and TEMPORARILY sets "
        "`generation_flag = GN_VER2005_SV` (:522) to lex it. NOT the construct the file is named "
        "for - `arr[-'d1 +: 2]` (line 12) parses, measured. Control: `r !== 2'bxx` parses"),
    "ivtest/ivltests/partsel_outside_expr.v": (
        "must_reject", "pinned .13e.2: line 16 `outside_expr !== 'x` - the same unbased unsized "
        "literal as partsel_outside_const.v, same clause (IEEE 1364-2005 A.8.7 decimal_number, "
        + V2005_ANNEX_A + ":978), same ruling, same lexor.lex:516/:522 self-testimony. The "
        "named construct "
        "`arr[uoffset +: 2]` (line 13) parses"),
    "ivtest/ivltests/timing_check_syntax.v": (
        "must_reject", "pinned .13e.2: line 12 `$setup(posedge sig1 , negedge sig2 , 0:0:0);` "
        "passes a min:typ:max triple as the timing_check_limit. IEEE 1364-2005 A.7.5.2 "
        "(" + V2005_ANNEX_A + ":800) AND clause 15.5.2 "
        "(docs/verilog/2005/txt/section-15-timing-checks.txt:92) both define "
        "`timing_check_limit ::= expression` - not mintypmax_expression, which the SAME argument "
        "box spends on end_edge_offset (:84), stamptime_condition (:89) and start_edge_offset "
        "(:90), so the distinction is deliberate rather than an omission. iverilog is a superset "
        "by its own grammar: stimuli/sv/subs/iverilog/parse.y:3746 "
        "`delay_value : expression | expression ':' expression ':' expression`. "
        "Control: `$setup(posedge s1, negedge s2, 0);` parses"),
    "ivtest/ivltests/timing_check_delayed_signals.v": (
        "must_reject", "pinned .13e.2: line 13 `$setuphold(posedge sig1, negedge sig2 , 0:0:0 , "
        "0:0:0 , ...)` - the same min:typ:max timing_check_limit as timing_check_syntax.v, same "
        "clauses (IEEE 1364-2005 A.7.5.2 " + V2005_ANNEX_A + ":800 and clause 15.5.2 "
        "docs/verilog/2005/txt/section-15-timing-checks.txt:92), same ruling"),
    # ---- SV-CORPUS-GRAD.13e.3 (PGEN-SV-CORPUS-GRAD-0285) ------------------------------------
    # The row that PROVES the extension hole invents defects, found by scanning the 53
    # remaining v2005 rejects-valid rows for the two DEFAULT-ON extensions' constructs.
    "ivtest/ivltests/real_invalid_ops.v": (
        "must_reject", "pinned .13e.3: line 17 `wire r_b_na = var1 ~& var2;` (and :18 "
        "`~|`) use the reduction operators as BINARY ones - IEEE 1364-2005 A.8.6 "
        "binary_operator (" + V2005_ANNEX_A + ":954) lists neither. ⛔ THE SAME "
        "CONSTRUCT AS br_gh552.v, KEYED THE OPPOSITE WAY, and the difference is a flag "
        "the answer key did not read: br_gh552 passes `-gno-icarus-misc` and gets the "
        "golden `The binary NAND operator is an Icarus Verilog extension`, while this "
        "row inherits `gn_icarus_misc_flag = true` (main.cc:109) and so PARSES for "
        "iverilog - its golden is elaboration-only (`~& operator may not have a REAL "
        "operand`), which is exactly why `_gold_has_syntax_error()` read it as "
        "`must_accept`. The elaboration errors are real and orthogonal: the operands "
        "are `real`. Control: control_v2005_binary_and.sv parses"),
    "ivtest/ivltests/udp_empty_table_fail.v": (
        "must_reject", "pinned .13e.2: lines 7-8 are an EMPTY `table ... endtable`. IEEE "
        "1364-2005 A.5.3 (" + V2005_ANNEX_A + ":483/:485) is "
        "`combinational_body ::= table combinational_entry { combinational_entry } endtable` and "
        "`sequential_body ::= [ udp_initial_statement ] table sequential_entry "
        "{ sequential_entry } endtable` - the FIRST entry is mandatory in both, so an empty "
        "table has no derivation and PGEN stops exactly after `table`. The file's own header "
        "says `Check that an empty old-style UDP table generates an error` and the golden is "
        "`error: Empty UDP table.` Control: the same primitive with two entries parses"),
}


def expect_v2005(suite: str, rel: str, text: str, ividx):
    """Expected verdict for one v2005-lane row (leaf .8c.1) - suite metadata
    / upstream driver conventions only, judged against IEEE 1364-2005."""
    # The LRM pins run FIRST: everything below is upstream TOOL testimony, and a
    # conformance ruling with a clause cite outranks it (.3.14a/.3.14b/.3.15).
    pinned = V2005_LRM_PINNED.get(rel.replace("\\", "/"))
    if pinned:
        return pinned
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
      "size_cast_fail3", "sv_void_cast_fail1", "sv_void_cast_fail2"]),
    ("must_reject",
     "re-adjudicated by SV-CORPUS-GRAD.13x.c: void'(1+2); - IEEE 1800-2017 A.6.9 "
     "subroutine_call_statement ::= subroutine_call ; | void ' ( function_subroutine_call ) ; "
     "(docs/systemverilog/2017/md/section-41-data-read-api.md:2365-2367) admits only a subroutine CALL inside void'( ), and 1+2 is an "
     "expression; sv_void_cast_fail1/2 (void'(f(10)), void'(t(10))) stay must_accept because a "
     "call is derivable whatever the callee turns out to be. Release probe: REJECTS at "
     "furthest_position=103 on sv_2017 and sv_2023; control void'(f()) ACCEPTS",
     ["sv_void_cast_fail3"]),
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
    a descriptor keys the sv_2017 bulk lane when the generation it actually
    compiles under is a SystemVerilog one - its own flag when it carries one,
    otherwise the vendored compiler's GN_DEFAULT (leaf .13e.1). Before that
    leaf this read "an EXPLICIT SystemVerilog generation flag", and 147 rows
    sat in NO VERDICT because the descriptor was asked a question only the
    compiler could answer."""

    SV_GENS = ("-g2005-sv", "-g2009", "-g2012", "-g2017", "-g2023")
    V2005_GENS = ("-g1995", "-g2001", "-g2001-noconfig", "-g2005")

    # `enum generation_t` member -> the -g flag that selects it. Only the members
    # GN_DEFAULT can alias need an entry; a value outside this map REFUSES.
    GEN_ENUM_TO_FLAG = {
        "GN_VER1995": "-g1995",
        "GN_VER2001_NOCONFIG": "-g2001-noconfig",
        "GN_VER2001": "-g2001",
        "GN_VER2005": "-g2005",
        "GN_VER2005_SV": "-g2005-sv",
        "GN_VER2009": "-g2009",
        "GN_VER2012": "-g2012",
        "GN_VER2017": "-g2017",
        "GN_VER2023": "-g2023",
    }

    def __init__(self, ivtest_dir: Path):
        self.sv_entries = {}   # (dir, name) -> (type, gold-or-None)
        self.vlg_entries = {}  # (dir, name) -> (type, gold-or-None)
        self.gold_dir = ivtest_dir / "gold"
        self.vvp_desc = defaultdict(list)  # source stem -> [descriptor dict]
        self.default_gen = self._resolve_default_generation(ivtest_dir)
        # .13e.3: the DEFAULT ON/OFF state of every language-affecting extension.
        self.ext_defaults = self._resolve_extension_defaults(ivtest_dir)
        self._load(ivtest_dir / "regress-sv.list", sv=True)
        self._load(ivtest_dir / "regress-vlg.list", sv=False)
        self._load_vvp(ivtest_dir / "vvp_tests")

    # ---- SV-CORPUS-GRAD.13e.3 --------------------------------------------
    # ⛔ `-g` IS TWO FLAG FAMILIES WEARING ONE PREFIX, AND ONLY ONE OF THEM DECIDES
    # WHAT THE LANGUAGE IS. `.13e.1` taught this index to read the GENERATION out of
    # `iverilog-args`; it read nothing else, so a descriptor that switches a
    # NON-STANDARD EXTENSION on or off was still keyed *"compiles as -g2005 (the
    # compiler's own default)"* — a sentence that is false in its own terms about a
    # row whose descriptor names an explicit `-g` flag. Measured over all 560 vvp
    # descriptors: 43 carry a non-generation `-g` flag, spanning 11 distinct flags
    # over 40 source files.
    #
    # ⛔⛔ AND THE OBVIOUS CLASSIFIER IS WRONG IN BOTH DIRECTIONS. "Does the flag
    # reach parse.y?" admits `-gspecify` (21 sites) and `-gsupported-assertions`
    # (3 sites) — both of which sit inside the ACTION BODY of a rule that has
    # already reduced, so the grammar accepts the text either way — and it REJECTS
    # `-gxtypes`, which never appears in parse.y at all and is the actual gate
    # (`pform.cc:3580`, *"Net data type requires SystemVerilog or -gxtypes."*).
    # ⇒ the classification below is derived from WHAT EACH FLAG'S VARIABLE GUARDS,
    # and each entry names the site, so a reader can re-check it in one grep.
    #
    #   flag family              variable                       decides the language?
    #   -gxtypes/-gno-xtypes     gn_cadence_types_flag          YES  pform.cc:3580 gates a NET DATA TYPE
    #   -gicarus-misc/-gno-      gn_icarus_misc_flag            YES  parse.y:4162/4174 gate binary ~& / ~|
    #   -gverilog-ams/-gno-      gn_verilog_ams_flag            YES  lexor.lex (a different LANGUAGE; own lane)
    #   -gspecify/-gno-specify   gn_specify_blocks_flag         no   parse.y action bodies only (warnings)
    #   -gsupported-assertions   gn_supported_assertions_flag   no   parse.y action bodies only
    #   -ginterconnect/-gno-     gn_interconnect_flag           no   elaborate.cc only
    #   -gno-strict-*declaration gn_strict_*_declaration        no   symbol_search.cc (name resolution)
    #
    # ⭐⭐ THE TWO THAT MATTER MOST ARE DEFAULT-**ON** (`main.cc:109-110`), so they
    # are invisible in every descriptor that does not mention them: EVERY unflagged
    # ivtest row compiles with Cadence extended types AND the Icarus misc extensions
    # enabled. That is why `br_gh552.v` (extension explicitly OFF, keyed CE) and
    # `real_invalid_ops.v` (extension implicitly ON, keyed must_accept) carry
    # OPPOSITE expectations for the SAME construct — the answer key's verdict on
    # binary `~&` depended on a flag it did not read.
    EXT_LANGUAGE_AFFECTING = {
        "xtypes": ("gn_cadence_types_flag", True, "pform.cc"),
        "no-xtypes": ("gn_cadence_types_flag", False, "pform.cc"),
        "icarus-misc": ("gn_icarus_misc_flag", True, "parse.y"),
        "no-icarus-misc": ("gn_icarus_misc_flag", False, "parse.y"),
        "verilog-ams": ("gn_verilog_ams_flag", True, "lexor.lex"),
        "no-verilog-ams": ("gn_verilog_ams_flag", False, "lexor.lex"),
    }

    @classmethod
    def _resolve_extension_defaults(cls, ivtest_dir: Path) -> dict:
        """The default ON/OFF state of every language-affecting extension, READ from
        the vendored compiler rather than assumed — and re-checked against the two
        facts the classification rests on, so a re-vendoring that moves either one
        REFUSES instead of keying rows off a stale reading."""
        ivl = ivtest_dir.parent
        main_cc = ivl / "main.cc"
        if not main_cc.is_file():
            raise SystemExit(f"REFUSE: {main_cc} is missing - the extension posture "
                             "cannot be resolved, and guessing it is the defect .13e.3 fixed")
        mtext = main_cc.read_text(errors="replace")
        defaults, seen_site = {}, {}
        for flag, (var, _on, site) in sorted(cls.EXT_LANGUAGE_AFFECTING.items()):
            # (1) the flag still sets the variable it is classified by
            if not re.search(r'strcmp\(gen,\s*"' + re.escape(flag) + r'"\)\s*==\s*0', mtext):
                raise SystemExit(
                    f"REFUSE: main.cc no longer recognises -g{flag} - the .13e.3 extension "
                    "classification is keyed to a flag this compiler does not have")
            # (2) the variable still has a compile-time default here
            m = re.search(r"^bool\s+" + re.escape(var) + r"\s*=\s*(true|false)\s*;", mtext, re.M)
            if not m:
                raise SystemExit(
                    f"REFUSE: main.cc no longer initialises `{var}` - the DEFAULT posture of "
                    f"-g{flag} is what makes an unflagged row keyable at all")
            defaults[var] = (m.group(1) == "true")
            # (3) the variable is still consumed at a PARSE-STAGE site, which is the
            #     whole reason it is classified language-affecting
            src = ivl / site
            if var not in seen_site:
                if not src.is_file() or var not in src.read_text(errors="replace"):
                    raise SystemExit(
                        f"REFUSE: `{var}` no longer appears in {site} - it was classified "
                        "language-affecting BECAUSE it is consumed there; re-derive .13e.3's "
                        "table rather than trusting it")
                seen_site[var] = True
        return defaults

    def extension_posture(self, desc: dict):
        """(explicitly-set language-affecting flags, the default-ON ones that apply anyway).

        The first element is what makes a descriptor's own compile testimony testimony
        about a SUPERSET language; the second is the standing posture every ivtest row
        inherits whether or not it says so."""
        explicit = []
        for a in desc.get("iverilog-args", []):
            if a.startswith("-g") and a[2:] in self.EXT_LANGUAGE_AFFECTING:
                explicit.append(a)
        default_on = sorted({
            var for _f, (var, _on, _s) in self.EXT_LANGUAGE_AFFECTING.items()
            if self.ext_defaults.get(var)})
        return explicit, default_on

    def extension_note(self, desc: dict) -> str:
        """The clause a basis string owes the reader about WHICH language compiled."""
        explicit, default_on = self.extension_posture(desc)
        parts = []
        if explicit:
            parts.append("descriptor sets " + " ".join(explicit))
        if default_on:
            parts.append("compiler defaults ON: " + ", ".join(default_on))
        return "; ".join(parts)

    def enables_nonstandard(self, desc: dict) -> list:
        """The language-affecting extensions this descriptor turns ON by hand. A row
        whose compile testimony rests on one of these is testimony about a SUPERSET of
        IEEE 1364-2005 and cannot be keyed `must_accept` from it."""
        out = []
        for a in desc.get("iverilog-args", []):
            spec = self.EXT_LANGUAGE_AFFECTING.get(a[2:] if a.startswith("-g") else "")
            if spec and spec[1]:
                out.append(a)
        return out

    # ---- SV-CORPUS-GRAD.13e.1 --------------------------------------------
    # A vvp_tests descriptor that carries no -g flag is NOT dialect-unresolved.
    # The default generation is a property of the COMPILER, and that compiler is
    # vendored beside this test tree. Reading it here is what stopped 147 rows
    # from sitting in NO VERDICT on a premise the corpus itself refutes
    # (leaf .13e). Derived at run time and never hard-coded: if upstream moves
    # GN_DEFAULT to a SystemVerilog generation, those rows must move with it.
    @classmethod
    def _resolve_default_generation(cls, ivtest_dir: Path) -> str:
        ivl = ivtest_dir.parent
        header, main_cc = ivl / "compiler.h", ivl / "main.cc"
        runner, makefile = ivtest_dir / "vvp_reg.py", ivtest_dir / "Makefile.in"
        for path in (header, main_cc, runner, makefile):
            if not path.is_file():
                raise SystemExit(
                    f"REFUSE: {path} is missing - the ivtest default generation cannot be "
                    "resolved, and guessing it is the defect leaf .13e fixed")

        block = re.search(r"enum\s+generation_t\s*\{(.*?)\}", header.read_text(errors="replace"), re.S)
        if not block:
            raise SystemExit(f"REFUSE: no `enum generation_t` in {header}")
        members = {m.group(1): int(m.group(2))
                   for m in re.finditer(r"(GN_[A-Z0-9_]+)\s*=\s*(\d+)", block.group(1))}
        if "GN_DEFAULT" not in members:
            raise SystemExit(f"REFUSE: `GN_DEFAULT` is not a member of `enum generation_t` in {header}")
        named = sorted(k for k, v in members.items()
                       if v == members["GN_DEFAULT"] and k != "GN_DEFAULT")
        if len(named) != 1 or named[0] not in cls.GEN_ENUM_TO_FLAG:
            raise SystemExit(
                f"REFUSE: GN_DEFAULT = {members['GN_DEFAULT']} resolves to {named or 'nothing'} - "
                "not exactly one generation this adjudicator knows a -g flag for")
        # A default nothing reads decides nothing.
        if not re.search(r"generation_flag\s*=\s*GN_DEFAULT\s*;", main_cc.read_text(errors="replace")):
            raise SystemExit(
                f"REFUSE: {main_cc} no longer initialises `generation_flag = GN_DEFAULT` - "
                "the enum value may be inert")

        # The ONE mechanism that would override it. vvp_reg.py's force_gen() strips any
        # generation and inserts -g2023; if it ran by default, an unflagged descriptor
        # would be SystemVerilog and every verdict below would be inverted.
        rtext = runner.read_text(errors="replace")
        if not re.search(r"--force-sv['\"],\s*action=['\"]store_true", rtext):
            raise SystemExit(
                f"REFUSE: {runner}: `--force-sv` is no longer an off-by-default store_true flag - "
                "an unflagged descriptor may now be forced to SystemVerilog")
        if not re.search(r"if\s+cfg\[['\"]force-sv['\"]\]:\s*\n\s*force_gen\(", rtext):
            raise SystemExit(
                f"REFUSE: {runner}: force_gen() is no longer guarded by cfg['force-sv'] - "
                "it may now run unconditionally")
        mtext = makefile.read_text(errors="replace")
        invocations = [l for l in mtext.splitlines() if "vvp_reg.py" in l and "echo" not in l]
        if not invocations:
            raise SystemExit(f"REFUSE: {makefile} no longer invokes vvp_reg.py - the harness moved")
        if any("--force-sv" in l for l in invocations):
            raise SystemExit(
                f"REFUSE: {makefile} now passes --force-sv - the upstream default run IS "
                "SystemVerilog and the unflagged-descriptor verdicts are inverted")

        return cls.GEN_ENUM_TO_FLAG[named[0]]

    def effective_gen(self, desc: dict) -> str:
        """The generation a descriptor actually compiles under: its own -g flag when it
        carries one, otherwise the compiler's own default (leaf .13e.1)."""
        for a in desc.get("iverilog-args", []):
            if a in self.SV_GENS or a in self.V2005_GENS:
                return a
        return self.default_gen

    def gen_source(self, desc: dict) -> str:
        """WHERE that generation came from, AND under which extension posture. Carried
        into the basis string because the sentence this leaf replaced was precise about
        the descriptor and wrong about the question - a basis that cannot say which
        artifact decided is how that happens (leaf .13e). ⛔ .13e.3 extends it in exactly
        the same way for the SECOND thing the descriptor decides: naming the generation
        while staying silent about `-gxtypes` / the default-ON extensions is the same
        defect one field over."""
        for a in desc.get("iverilog-args", []):
            if a in self.SV_GENS or a in self.V2005_GENS:
                base = "explicit descriptor flag"
                break
        else:
            base = "the compiler's own default, compiler.h GN_DEFAULT"
        note = self.extension_note(desc)
        return f"{base}; {note}" if note else base

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

    def _gold_text(self, gold_stem: str) -> str:
        text = ""
        for chan in ("iverilog-stderr", "iverilog-stdout"):
            text += read_text(self.gold_dir / f"{gold_stem}-{chan}.gold")
        return text

    def _gold_has_parse_error(self, gold_stem: str, v2005: bool = False) -> bool:
        """Did the golden report a PARSE-stage refusal? (.13e.3(b))

        ⛔ `v2005` is not a convenience: the second vocabulary holds only for IEEE 1364-2005 and
        an unsplit list was measured manufacturing five false defects."""
        text = self._gold_text(gold_stem)
        if IVTEST_PARSE_STAGE_RE.search(text):
            return True
        return bool(v2005 and IVTEST_PARSE_STAGE_V2005_ONLY_RE.search(text))

    def _vvp_implied(self, desc, v2005: bool = False):
        """Map one descriptor to an implied parse-level verdict tag: 'accept' / 'reject' /
        'triage' (CE without usable golden) / 'ni' (upstream runner skips, no testimony).

        `v2005` selects the edition the caller is judging against (.13e.3(b))."""
        ttype = desc.get("type")
        if ttype == "NI":
            return "ni"
        if ttype != "CE":
            # normal/EF (and any run-to-completion type): iverilog compiles
            # the file under the declared SV generation - parse-level valid.
            return "accept"
        gold = desc.get("gold")
        if gold:
            return "reject" if self._gold_has_parse_error(gold, v2005) else "accept"
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
                # .13e.3(b): the SV lane gets ONLY the edition-invariant vocabulary. The
                # v2005-only entries are relaxations IEEE 1800 makes, so applying them here
                # is precisely the error that would have manufactured five false defects.
                hit = IVTEST_PARSE_STAGE_RE.search(gtext)
                if hit:
                    return ("must_reject",
                            f"ivtest: CE with golden {gold} reporting a "
                            f"PARSE-stage refusal ({hit.group(0)!r}) - "
                            "parse-level invalid (.13e.3)")
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
        sv_descs, v2005_gens = [], set()
        for d in descs:
            args = d.get("iverilog-args", [])
            if any("verilog-ams" in a for a in args):
                continue  # AMS-flavored run - never an SV/plain-Verilog key
            # .13e.1: a descriptor with no -g flag is not dialect-unresolved; it
            # compiles under the vendored compiler's own default generation.
            gen = self.effective_gen(d)
            if gen in self.SV_GENS:
                sv_descs.append(d)
            elif gen in self.V2005_GENS:
                v2005_gens.add((gen, self.gen_source(d)))
        v2005 = bool(v2005_gens)
        if sv_descs:
            implied = {self._vvp_implied(d) for d in sv_descs}
            names = ", ".join(d["_key"] + ".json" for d in sv_descs)
            if implied == {"accept"}:
                ext_on = sorted({e for d in sv_descs for e in self.enables_nonstandard(d)})
                if ext_on:
                    # .13e.3: the compile testimony is about a SUPERSET language. A row
                    # whose own descriptor switches a non-standard extension ON cannot be
                    # keyed `must_accept` from "it compiled" - that is testimony about
                    # iverilog-plus-extension, not about the standard.
                    return ("out_of_scope_with_cause:vendor_extension_enabled",
                            f"ivtest: vvp_tests descriptor(s) {names} compile only with "
                            f"{' '.join(ext_on)} - a NON-STANDARD extension the descriptor "
                            "enables itself, so the compile testimony is about a superset "
                            "language and cannot key a conformance verdict (.13e.3)")
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
            how = ", ".join(
                f"{gen} ({src})" for gen, src in sorted(v2005_gens))
            return ("out_of_scope_with_cause:v2005_profile_lane",
                    f"ivtest: vvp_tests descriptor compiles as {how} - keyed "
                    "for the verilog_2005 profile lane")
        return ("out_of_scope_with_cause:no_sv_key",
                "ivtest: vvp_tests descriptor(s) without an explicit "
                "generation flag - dialect unresolved (the upstream default "
                "generation is not encoded in the descriptor)")

    def expect_v2005(self, relpath: str):
        """v2005-lane key for one ivtest row (leaf .8c.1): regress-vlg.list
        entries first (the .8a CE stage split mirrored: golden `syntax
        error` = parse-level invalid under the iverilog plain-Verilog
        dialect), then plain-Verilog vvp_tests generations - explicit, or the
        vendored compiler's own GN_DEFAULT when the descriptor names none
        (leaf .13e.1; the two lanes share `effective_gen`, or a row routed
        here would land in a lane that cannot answer it)."""
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
                # .13e.3(b): the vlg lane IS the plain-Verilog lane, so both vocabularies
                # apply. `SYNTAX_ERR_RE` alone read 78 of 84 usable goldens as post-parse.
                hit = (IVTEST_PARSE_STAGE_RE.search(gtext)
                       or IVTEST_PARSE_STAGE_V2005_ONLY_RE.search(gtext))
                if hit:
                    return ("must_reject",
                            f"ivtest: vlg CE with golden {gold} reporting a "
                            f"PARSE-stage refusal ({hit.group(0)!r}) - "
                            "parse-level invalid 1364-2005 (.13e.3)")
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
            # .13e.1: same rule as _expect_vvp, from the SAME definition - a
            # descriptor with no -g flag compiles under the compiler's default.
            # The two sides MUST agree, or a row routed here would land in a
            # lane that cannot answer it.
            v_descs = [d for d in descs
                       if not any("verilog-ams" in a
                                  for a in d.get("iverilog-args", []))
                       and self.effective_gen(d) in self.V2005_GENS]
            gens = ", ".join(sorted({
                f"{self.effective_gen(d)} ({self.gen_source(d)})"
                for d in v_descs}))
            if v_descs:
                implied = {self._vvp_implied(d, v2005=True) for d in v_descs}
                names = ", ".join(d["_key"] + ".json" for d in v_descs)
                if implied == {"accept"}:
                    ext_on = sorted({e for d in v_descs
                                     for e in self.enables_nonstandard(d)})
                    if ext_on:
                        # .13e.3: see _expect_vvp - the compile testimony is about a
                        # SUPERSET language, so it cannot key a conformance verdict.
                        return ("out_of_scope_with_cause:vendor_extension_enabled",
                                f"ivtest: vvp_tests descriptor(s) {names} compile only "
                                f"with {' '.join(ext_on)} - a NON-STANDARD extension the "
                                "descriptor enables itself, so the compile testimony is "
                                "about a superset language and cannot key a conformance "
                                "verdict (.13e.3)")
                    return ("must_accept",
                            f"ivtest: vvp_tests descriptor(s) {names} - "
                            f"compiles as {gens} with no syntax-error golden")
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
    # ---- SV-CORPUS-GRAD.13x.c (PGEN-SV-CORPUS-GRAD-0352): two rows the .8b.1 ACCEPT groups
    # keyed on a "BNF-parseable" claim the productions refute. Re-derived in-repo from the
    # Annex A text at the cited lines; each also pinned class=invalid + class=control under
    # stimuli/sv/adjudication_repros/. The release probe REJECTS both at exactly the token
    # named (enum_range_x furthest_position=87, port_init_early furthest_position=148) and
    # ACCEPTS the one-token-legal twins (A[1'sd1] / A[2]; `output a;` / `output reg a = 1;`).
    ({"enum_range_x"},
     "IEEE 1800-2017 A.2.2.1 enum_name_declaration ::= enum_identifier "
     "[ [ integral_number [ : integral_number ] ] ] [ = constant_expression ] "
     "(docs/systemverilog/2017/md/section-41-data-read-api.md:875-876): the range takes an integral_number (docs/systemverilog/2017/md/section-41-data-read-api.md:3240-3243, "
     "a decimal/octal/binary/hex number) and 'x is an unbased_unsized_literal "
     "(docs/systemverilog/2017/md/section-41-data-read-api.md:3290) - no production admits it there (SV-CORPUS-GRAD.13x.c)"),
    ({"port_init_early"},
     "IEEE 1800-2017 A.2.1.2 output_declaration ::= output net_port_type "
     "list_of_port_identifiers | output variable_port_type "
     "list_of_variable_port_identifiers (docs/systemverilog/2017/md/section-41-data-read-api.md:762-765): only the variable "
     "form carries [ = constant_expression ] (docs/systemverilog/2017/md/section-41-data-read-api.md:994 vs :975) and "
     f"variable_port_type ::= var_data_type ::= data_type | var data_type_or_implicit "
     "(docs/systemverilog/2017/md/section-41-data-read-api.md:907-908) needs a type keyword or var, so `output a = 1;` with an "
     "implicit net type has no derivation (SV-CORPUS-GRAD.13x.c)"),
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
      # .3.37: module_param_mismatch_type, interface_param_mismatch_type and parameter_no_default_1/2/3
      # LEFT this set - their text omits a body parameter's default, which Annex A footnote 18 forbids,
      # so it is NOT BNF-parseable; they are pinned must_reject in EXTRA_PINNED with the cite.
      "interface_param_mismatch_expr"},
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
      "size_cast_zero_var", "enum_range_neg",
      "enum_range_zero", "enum_conflict"},
     "constant-value legality (cast width / enum range values) - semantic "
     "(IEEE 1800-2017 6.24.1/6.19 prose); the text is BNF-parseable "
     "(enum_range_x left this group in SV-CORPUS-GRAD.13x.c: 'x is no "
     "integral_number, so its text is NOT BNF-parseable; A[1'sd1] and A[0] are)"),
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
    ({"port_not_in_header", "port_packed_first",
      "port_packed_second", "port_redeclare", "port_unpacked_first",
      "port_unpacked_second"},
     "port/declaration coherence rules (IEEE 1800-2017 23.2.2 prose); each "
     "declaration is BNF-parseable (port_init_early left this group in "
     "SV-CORPUS-GRAD.13x.c: an initializer on an implicit-type net port has "
     "no A.2.1.2 derivation; the six that stay were each re-read and derive)"),
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
    "string_literal_backtick_eof",
    # "default_nettype_invalid" moved to the EXTRA_PINNED .3.39 block: since 1.0.217 the PARSER
    # judges `default_nettype's argument (22.8), so the row is must_reject, not preprocessor-owned.
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
    "severity_task_arg": ("must_reject",
        "pinned .13x.c (re-adjudicating the .8b.2 pin): '$fatal(.x(\"x\"));' sits at "
        "MODULE level, where IEEE 1800-2017 A.1.4 module_common_item admits it only as an "
        "elaboration_system_task ::= $fatal [ ( finish_number [ , list_of_arguments ] ) ] ; "
        "(docs/systemverilog/2017/md/section-41-data-read-api.md:301-309), finish_number ::= 0 | 1 | 2 - a named argument cannot be the "
        "first token. The .8b.2 reading (an A.8.2 system_tf_call with a named "
        "list_of_arguments member) holds only INSIDE procedural code, which this file is not. "
        "Release probe: REJECTS at furthest_position=102 on sv_2017 and sv_2023; controls "
        "`$fatal(1, \"x\");` and `$error(.x(\"x\"));` at module level ACCEPT (the $error "
        "arm takes a bare list_of_arguments, so the named form IS legal there)"),
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
    # "line_char1", "line_char2", "line_eof", "line_level" moved to the EXTRA_PINNED .3.39 block:
    # since 1.0.217 the PARSER judges `line's three arguments (22.12), so the four rows are
    # must_reject with the clause, not preprocessor-owned. Their .8b.2 readings were right about
    # the FORM of each defect and wrong only about the stage.
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


_SG_MODULE = None


def _sg():
    """`stimuli/sv/store_gate_explanations.py`, imported by path (SV-CORPUS-GRAD.13w)."""
    global _SG_MODULE
    if _SG_MODULE is None:
        import importlib.util
        spec = importlib.util.spec_from_file_location(
            "pgen_store_gate_explanations", Path(__file__).resolve().parent / "store_gate_explanations.py")
        mod = importlib.util.module_from_spec(spec)
        sys.modules["pgen_store_gate_explanations"] = mod
        spec.loader.exec_module(mod)
        _SG_MODULE = mod
    return _SG_MODULE


def report_parse_binary_sha(report: Path) -> str:
    """The `parse binary` digest the TRACKED lane report names; "" when the report is absent."""
    if not report.is_file():
        return ""
    for line in report.read_text(encoding="utf-8").splitlines():
        m = IDENTITY_ROW_RE.match(line)
        if m and m.group(1).strip() == "parse binary":
            return m.group(3)
    return ""


def resolve_corpus_path(args):
    def resolve(suite, rel):
        if suite == "uvm-core":
            return args.subs_root.parent / "uvm" / rel
        return args.subs_root / suite / rel
    return resolve


def store_gate_pass(lane, manifest, resolve, report, cache, jobs, *, measurer=None):
    """SV-CORPUS-GRAD.13w. Every `divergence:unexplained_rejects_valid` row is a candidate: the
    store-gate search decides, BY PARSING, whether the parser's sound gate on a provably-undeclared
    type identifier is the only obstacle (=> `divergence:explained_store_gate`, with the minimal
    prelude in the basis) or the row stays in the bar (with the search's residual reason in the
    basis, so the next defect slice starts from a triage instead of a list)."""
    sg = _sg()
    candidates = [(suite, rel, resolve(suite, rel)) for suite, rel, _obs, _exp, verdict, _b in manifest
                  if verdict == "divergence:unexplained_rejects_valid"]
    decided = sg.explain_rows(lane, candidates, report_parse_binary_sha(report),
                              Path(__file__).resolve().parent.parent.parent / "rust/target/release/parseability_probe",
                              cache, jobs=jobs, measurer=measurer,
                              log=lambda msg: print(msg, file=sys.stderr))
    out = []
    for suite, rel, observed, expected, verdict, basis in manifest:
        row = decided.get((suite, rel)) if verdict == "divergence:unexplained_rejects_valid" else None
        if row is not None:
            if row["verdict"] == sg.EXPLAINED:
                verdict = sg.CLASS
            basis += sg.basis_suffix(row)
        out.append((suite, rel, observed, expected, verdict, basis))
    return out



# ---- SV-CORPUS-GRAD.3.37: the EDITION-KEYWORD pass ---------------------------------------------
# A Verilog-2005 test that names a variable `var`, `global`, `local` or `new` is legal 1364-2005 and
# NOT SystemVerilog: IEEE 1800-2017 Annex B reserves every one of those words (Table B.1, 248 words;
# 1800-2023 the same), IEEE 1364-2005 Annex B (124) does not. Until release 1.0.211 the grammar's
# reserved set held 173 of the 248, so such files PASSED on the sv lanes and their must_accept keys
# (a Verilog tool's testimony) read as confirmed. With the set derived from Annex B they REJECT, and
# the key is what was wrong. This pass corrects it PER ROW, BY PARSING: the sv parse must stop ON the
# keyword (comments and strings blanked, layout skipped), and the verilog_2005 profile must ACCEPT the
# same bytes. The second condition is the guard against a grammar hole hiding here — `d = null;`
# stops on `null` too, and no 1364-2005 parse accepts a class handle.
_KW_MODULE = None
EDITION_BASIS_TAG = "EDITION-KEYWORD (SV-CORPUS-GRAD.3.37)"


def _kw():
    """`stimuli/sv/check_sv_reserved_keywords.py`, imported by path — the Annex B tables are DERIVED
    from the in-repo LRM text there, never copied here."""
    global _KW_MODULE
    if _KW_MODULE is None:
        import importlib.util
        spec = importlib.util.spec_from_file_location(
            "pgen_check_sv_reserved_keywords", Path(__file__).resolve().parent / "check_sv_reserved_keywords.py")
        mod = importlib.util.module_from_spec(spec)
        sys.modules["pgen_check_sv_reserved_keywords"] = mod
        spec.loader.exec_module(mod)
        _KW_MODULE = mod
    return _KW_MODULE


def keywords_1800_only():
    kw = _kw()
    k17 = set(kw.table("1800-2017"))
    return k17 - set(kw.table("1364-2005", reference=k17))


_IDENT_RE = re.compile(r"[A-Za-z_$][A-Za-z0-9_$]*")


def stuck_token(text: str, furthest):
    """The identifier-shaped token the parse stopped ON, and its byte offset — comments and strings
    blanked, the layout gap after `furthest` skipped (a-furthest-position-names-a-region-not-a-token)."""
    if furthest is None:
        return None, None
    stripped = blank_comments_and_strings(text)
    i = min(furthest, len(stripped))
    while i < len(stripped) and stripped[i].isspace():
        i += 1
    m = _IDENT_RE.match(stripped, i)
    return (m.group(0), i) if m else (None, i)


def edition_keyword_pass(lane, manifest, resolve, positions, *, measurer=None):
    """SV-CORPUS-GRAD.3.37. Every `divergence:unexplained_rejects_valid` row keyed must_accept is a
    candidate; the row is re-keyed must_reject (=> `match`) iff the sv parse stops ON an 1800-only
    keyword AND the verilog_2005 profile accepts the bytes (measured here). Every other row is
    byte-identical on the way out; the basis carries the byte, the line, the word and both cites."""
    only = keywords_1800_only()
    if measurer is None:
        def measurer(raw):
            sg = _sg()
            m = sg.machinery()
            saved = m.PROFILE
            m.PROFILE = "verilog_2005"
            try:
                ok, _surface, _furthest, _trace = m.probe_text(raw, trace=False)
            finally:
                m.PROFILE = saved
            return bool(ok)
    out = []
    ruled = 0
    for suite, rel, observed, expected, verdict, basis in manifest:
        if verdict == "divergence:unexplained_rejects_valid" and expected == "must_accept":
            text = read_text(resolve(suite, rel))
            tok, at = stuck_token(text, positions.get((suite, rel)))
            # A keyword-version directive (22.14) makes the reserved set DIRECTIVE-CONTROLLED: with
            # `begin_keywords "1364-2005"` honoured the text IS legal SystemVerilog, and the parser's
            # inability to honour it is its own feature gap (the parked ispras 22.14.01_04 row), not a
            # wrong key - such a row stays in the bar. Measured 2026-09-04: 3 of the first 5 rulings were this.
            # ⛔ ONE condition, no `continue`: every row must reach `out.append` below (the self-test's
            # byte-identical arms are what caught a `continue` that dropped non-candidates from the manifest).
            if (tok in only and "`begin_keywords" not in blank_comments_and_strings(text)
                    and measurer(text)):
                line = text.count("\n", 0, at) + 1
                expected, verdict, ruled = "must_reject", "match", ruled + 1
                basis = (f"{EDITION_BASIS_TAG}: the {lane} parse stops at byte {at} (line {line}) ON `{tok}`, a word "
                         "IEEE 1800-2017 Annex B Table B.1 reserves (docs/systemverilog/2017/md/section-41-data-read-api.md:3599-3858; "
                         "1800-2023 Annex B the same 248) and IEEE 1364-2005 Annex B does not "
                         "(docs/verilog/2005/md/section-Annex_B-normative-list-of-keywords.md) - the text binds it as an identifier, "
                         "so it is not SystemVerilog; the verilog_2005 profile ACCEPTS these bytes (measured here, by parsing), which is "
                         "the reading the suite's key records - a Verilog tool's testimony. Spec outranks tool. Original key: must_accept - "
                         + basis)
        out.append((suite, rel, observed, expected, verdict, basis))
    print(f"edition-keyword pass [{lane}]: {ruled} row(s) re-keyed must_reject (parse stopped ON an 1800-only keyword; verilog_2005 accepts)",
          file=sys.stderr)
    return out


def _self_test_edition_keyword() -> int:
    """SV-CORPUS-GRAD.3.37 arms: the pass re-keys ONLY a row that stops ON an 1800-only keyword AND
    that the fake verilog_2005 measurer accepts; a keyword stop the measurer refuses (`null` in a
    class file), a non-keyword stop, and a non-candidate verdict all pass through byte-identical."""
    import tempfile
    fails = []
    calls = []

    def check(name, cond):
        print(f"  {'PASS' if cond else 'FAIL'}  edition-keyword: {name}")
        if not cond:
            fails.append(name)

    only = keywords_1800_only()
    check("derived set holds var/global/local/new/null and not always/reg/automatic",
          {"var", "global", "local", "new", "null"} <= only and not ({"always", "reg", "automatic"} & only))
    with tempfile.TemporaryDirectory(dir=str(Path(__file__).resolve().parent.parent.parent / "rust/target")) as td:
        d = Path(td)
        (d / "s").mkdir()
        bodies = {"a.v": "module a;\nreg var; // var is a name here\nendmodule\n",
                  "b.sv": "class C; endclass\nmodule b; C d; initial d = null; endmodule\n",
                  "c.v": "module c;\nreg x;\nendmodule\n",
                  "e.v": "module e;\nreg global;\nendmodule\n",
                  "f.v": "module f; endmodule\n"}
        for name, body in bodies.items():
            (d / "s" / name).write_text(body)
        positions = {("s", "a.v"): bodies["a.v"].index("var"), ("s", "b.sv"): bodies["b.sv"].index("null"),
                     ("s", "c.v"): bodies["c.v"].index("x;"), ("s", "e.v"): bodies["e.v"].index("global")}
        bodies["g.v"] = "`begin_keywords \"1364-2005\"\nmodule g;\nreg var;\nendmodule\n`end_keywords\n"
        (d / "s" / "g.v").write_text(bodies["g.v"]); positions[("s", "g.v")] = bodies["g.v"].index("var;")
        manifest = [
            ("s", "a.v", "fail", "must_accept", "divergence:unexplained_rejects_valid", "key a"),
            ("s", "b.sv", "fail", "must_accept", "divergence:unexplained_rejects_valid", "key b"),
            ("s", "c.v", "fail", "must_accept", "divergence:unexplained_rejects_valid", "key c"),
            ("s", "e.v", "fail", "must_accept", "divergence:unexplained_rejects_valid", "key e"),
            ("s", "f.v", "pass", "must_accept", "match", "key f"),
            ("s", "g.v", "fail", "must_accept", "divergence:unexplained_rejects_valid", "key g"),
        ]

        def measurer(raw):
            calls.append(raw)
            return "module a;" in raw or "module g;" in raw   # a.v and g.v are "legal 1364-2005" to the fake

        out = edition_keyword_pass("sv_2017", manifest, lambda s_, r_: d / s_ / r_, positions, measurer=measurer)
        by = {(r[0], r[1]): r for r in out}
        a = by[("s", "a.v")]
        check("a.v (stops ON var, measurer accepts) is re-keyed must_reject and matches",
              a[3] == "must_reject" and a[4] == "match" and a[5].startswith(EDITION_BASIS_TAG)
              and "`var`" in a[5] and "line 2" in a[5] and a[5].endswith("key a"))
        check("b.sv (stops ON null, measurer refuses) is byte-identical", by[("s", "b.sv")] == manifest[1])
        check("c.v (stops on a non-keyword) is byte-identical and never measured",
              by[("s", "c.v")] == manifest[2] and not any("module c;" in c for c in calls))
        check("e.v (stops ON global, measurer refuses) is byte-identical", by[("s", "e.v")] == manifest[3])
        check("f.v (a match) is byte-identical and never measured",
              by[("s", "f.v")] == manifest[4] and not any("module f;" in c for c in calls))
        check("g.v (stops ON var but carries `begin_keywords) is byte-identical and never measured",
              by[("s", "g.v")] == manifest[5] and not any("module g;" in c for c in calls))
        check("the measurer ran exactly for the keyword stops without a keyword-version directive", len(calls) == 3)
    print(f"edition-keyword self-test: {'PASS' if not fails else 'FAIL'} ({len(fails)} failing)")
    return 1 if fails else 0

def _self_test_store_gate() -> int:
    """SV-CORPUS-GRAD.13w arms: the post-pass reclassifies ONLY the rows the search explains,
    leaves every other verdict byte-identical, and writes the proof into the basis. A fake
    measurer drives it, so the arms need no parser and cannot be excused by one."""
    import tempfile
    sg = _sg()
    fails = []

    def check(name, cond):
        print(f"  {'PASS' if cond else 'FAIL'}  store-gate: {name}")
        if not cond:
            fails.append(name)

    with tempfile.TemporaryDirectory(dir=str(Path(__file__).resolve().parent.parent.parent / "rust/target")) as td:
        d = Path(td)
        (d / "s").mkdir()
        for name, body in (("a.sv", "module a; T x; endmodule\n"), ("b.sv", "module b; U y; endmodule\n"),
                           ("c.sv", "module c; endmodule\n"), ("r.sv", "module r; U y; endmodule\n")):
            (d / "s" / name).write_text(body)
        manifest = [
            ("s", "a.sv", "fail", "must_accept", "divergence:unexplained_rejects_valid", "key"),
            ("s", "b.sv", "fail", "must_accept", "divergence:unexplained_rejects_valid", "key"),
            ("s", "c.sv", "pass", "must_accept", "match", "key"),
            ("s", "r.sv", "pass", "must_reject", "divergence:unexplained_accepts_invalid", "key"),
        ]
        seen = []

        def fake(raw, profile):
            seen.append(raw)
            return (sg.EXPLAINED, ("T",), ("class",)) if "T x" in raw else ("residual:not_parsed_with_declarations", ("U",), ())
        cache = {}
        # the lane report is what names the parser the memo is keyed on - a synthetic one here
        report = d / "characterization.md"
        report.write_text("| parse binary | `rust/target/release/parseability_probe` | `" + "a" * 64 + "` |\n")
        check("the report's `parse binary` digest is read", report_parse_binary_sha(report) == "a" * 64)
        try:
            store_gate_pass("sv_2017", manifest, lambda s, r: d / s / r, d / "absent.md", cache, 2, measurer=fake)
            check("a lane with NO report digest refuses to measure", False)
        except SystemExit as e:
            check("a lane with NO report digest refuses to measure", "names no `parse binary` digest" in str(e))
        out = store_gate_pass("sv_2017", manifest, lambda s, r: d / s / r, report, cache, 2, measurer=fake)
        check("only rejects-valid rows are candidates (2 of 4 measured)", len(seen) == 2)
        by = {(r[0], r[1]): r for r in out}
        check("an explained row becomes divergence:explained_store_gate",
              by[("s", "a.sv")][4] == sg.CLASS and "parses once `T` is declared (class)" in by[("s", "a.sv")][5])
        check("a residual row stays in the bar and names what it demanded",
              by[("s", "b.sv")][4] == "divergence:unexplained_rejects_valid" and "residual:not_parsed_with_declarations" in by[("s", "b.sv")][5] and "`U`" in by[("s", "b.sv")][5])
        check("match and accepts-invalid rows are byte-identical", by[("s", "c.sv")] == manifest[2] and by[("s", "r.sv")] == manifest[3])
        check("the cache holds exactly the candidate population", sorted(k[2] for k in cache) == ["a.sv", "b.sv"])
    print(f"store-gate self-test: {'PASS' if not fails else 'FAIL'} ({len(fails)} failing)")
    return 1 if fails else 0


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
    if expected == "not_source_text":
        return "deferred:not_source_text"
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


def _self_test_extension_guard(ivtest_dir: Path) -> int:
    """SV-CORPUS-GRAD.13e.3 RED CONTROL. The vendor-extension guard fires on ZERO rows
    at HEAD, and a guard never observed firing is not known to work
    ([[feedback_an_instrument_that_can_only_return_one_reading_is_not_a_measurement]]).
    These arms drive it with SYNTHETIC descriptors and assert both directions, plus the
    three refusals that keep the classification tied to the vendored compiler."""
    ok, bad = 0, []

    def check(name, cond):
        nonlocal ok
        if cond:
            ok += 1
            print(f"  ok   {name}")
        else:
            bad.append(name)
            print(f"  FAIL {name}")

    idx = IvtestIndex(ivtest_dir)

    # --- the classifier, both directions -------------------------------------------------
    check("an ENABLER is language-affecting",
          idx.enables_nonstandard({"iverilog-args": ["-gxtypes"]}) == ["-gxtypes"])
    check("a DISABLER enables nothing",
          idx.enables_nonstandard({"iverilog-args": ["-gno-xtypes"]}) == [])
    check("-gspecify is NOT language-affecting (parse.y action bodies only)",
          idx.enables_nonstandard({"iverilog-args": ["-gspecify"]}) == [])
    check("-gsupported-assertions is NOT language-affecting",
          idx.enables_nonstandard({"iverilog-args": ["-gsupported-assertions"]}) == [])
    check("-ginterconnect is NOT language-affecting (elaborate.cc only)",
          idx.enables_nonstandard({"iverilog-args": ["-ginterconnect"]}) == [])
    check("a bare generation flag enables nothing",
          idx.enables_nonstandard({"iverilog-args": ["-g2005"]}) == [])

    # --- the DEFAULT posture is READ, not assumed ----------------------------------------
    check("gn_icarus_misc_flag is default ON in the vendored compiler",
          idx.ext_defaults.get("gn_icarus_misc_flag") is True)
    check("gn_cadence_types_flag is default ON in the vendored compiler",
          idx.ext_defaults.get("gn_cadence_types_flag") is True)
    check("gn_verilog_ams_flag is default OFF in the vendored compiler",
          idx.ext_defaults.get("gn_verilog_ams_flag") is False)

    # --- the basis string SAYS so ---------------------------------------------------------
    note = idx.gen_source({"iverilog-args": ["-gxtypes"]})
    check("gen_source names the explicit extension", "-gxtypes" in note)
    check("gen_source names the default-ON posture",
          "gn_icarus_misc_flag" in idx.gen_source({}))

    # --- THE GUARD ITSELF, driven RED on a synthetic row ---------------------------------
    # A descriptor that would otherwise key `must_accept` by compile testimony, with an
    # extension it enables ITSELF. Injected into a copy of the index so no tracked input moves.
    probe = dict(_key="synthetic_ext_probe", source="synthetic_ext_probe.v",
                 type="normal", **{"iverilog-args": ["-g2005", "-gxtypes"]})
    idx.vvp_desc["synthetic_ext_probe"] = [probe]
    verdict, basis = idx.expect_v2005("ivtest/ivltests/synthetic_ext_probe.v")
    check("GUARD FIRES: an extension-enabling accept row is refused a conformance verdict",
          verdict == "out_of_scope_with_cause:vendor_extension_enabled")
    check("GUARD basis names the flag", "-gxtypes" in basis)

    # The SAME row without the extension must still key must_accept - otherwise the guard
    # is not discriminating, it is just refusing everything.
    probe2 = dict(_key="synthetic_plain_probe", source="synthetic_plain_probe.v",
                  type="normal", **{"iverilog-args": ["-g2005"]})
    idx.vvp_desc["synthetic_plain_probe"] = [probe2]
    verdict2, _ = idx.expect_v2005("ivtest/ivltests/synthetic_plain_probe.v")
    check("GUARD DISCRIMINATES: the same row without the extension still keys must_accept",
          verdict2 == "must_accept")

    # --- the three REFUSALS that keep the table tied to the compiler ----------------------
    import tempfile, shutil
    for label, mutate in (
        ("REFUSAL: main.cc stops recognising -gxtypes",
         lambda t: t.replace('strcmp(gen,"xtypes")', 'strcmp(gen,"xtypes_RENAMED")')),
        ("REFUSAL: main.cc stops initialising gn_icarus_misc_flag",
         lambda t: t.replace("bool gn_icarus_misc_flag = true;", "// removed")),
    ):
        # `DATA-LOCALITY.2`: the scratch tree is derived from the repository root at run time,
        # never `$TMPDIR` — a copytree of the iverilog checkout is project-owned data.
        _dl_scratch = REPO_ROOT / "rust" / "target" / "adjudicator_refusals"
        _dl_scratch.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(dir=str(_dl_scratch)) as td:
            fake_ivl = Path(td) / "iverilog"
            shutil.copytree(ivtest_dir.parent, fake_ivl, symlinks=True,
                            ignore=shutil.ignore_patterns("ivtest"))
            shutil.copytree(ivtest_dir, fake_ivl / "ivtest", symlinks=True)
            mc = fake_ivl / "main.cc"
            mc.write_text(mutate(mc.read_text(errors="replace")), errors="replace")
            try:
                IvtestIndex._resolve_extension_defaults(fake_ivl / "ivtest")
                check(label, False)
            except SystemExit:
                check(label, True)

    print(f"\nEXTENSION-GUARD SELF-TEST: {ok} passed, {len(bad)} failed")
    if bad:
        print("  failed arms: " + ", ".join(bad))
    return 1 if bad else 0

BEGIN_KEYWORDS_RE = re.compile(r'`begin_keywords\s+"([^"\n]*)"')
VAMS_VERSION_RE = re.compile(r"^(?:VAMS-[0-9.]+|1800\+VAMS)$")


def verilog_ams_keyword_version_only(text: str) -> bool:
    """SV-CORPUS-GRAD.3.39 — is EVERY `begin_keywords in the file a Verilog-AMS version?

    IEEE 1800-2017 22.14 Syntax 22-10 (docs/systemverilog/2017/md/section-22-compiler-directives.md
    :851-862) lists eight version_specifiers and "It shall be an error if an implementation does not
    recognize the version_specifier used" (:869-870); "VAMS-2.3" / "1800+VAMS" are Accellera
    Verilog-AMS's, so the file is Verilog-AMS input — the class the ivtest descriptor rule already
    routes to the parked VERILOG-AMS tree, now decided from the FILE for every suite. A file that
    switches between IEEE and AMS versions is a keyword-REGION test and is deliberately not taken.
    Comments and strings are blanked first (a commented-out directive is not one).
    """
    versions = BEGIN_KEYWORDS_RE.findall(blank_comments_only(text))
    return bool(versions) and all(VAMS_VERSION_RE.match(v) for v in versions)


def blank_comments_only(text: str) -> str:
    """Blank comments in place (length- and newline-preserving) but KEEP string literals — the
    `begin_keywords version lives inside one."""
    def repl(m):
        tok = m.group(0)
        if tok.startswith('"'):
            return tok
        return "".join("\n" if ch == "\n" else " " for ch in tok)
    return COMMENT_STRING_RE.sub(repl, text)


def key_2017(suite, rel, text, vidx, ividx, sidx):
    """The sv_2017 lane's answer key for ONE row: the pinned overrides first, then the suite
    conventions, then each suite's own oracle. Factored out of main() by SV-CORPUS-GRAD.14b.2 so
    the sv_2023 lane keys the same rows through the same code and differs ONLY by edition."""
    if (suite, rel) in EXTRA_PINNED:
        return EXTRA_PINNED[(suite, rel)]
    if verilog_ams_keyword_version_only(text):
        return ("out_of_scope_with_cause:verilog_ams_lane",
                "SV-CORPUS-GRAD.3.39: every `begin_keywords in the file names a Verilog-AMS version_specifier "
                "(\"VAMS-2.3\" / \"1800+VAMS\") that IEEE 1800-2017 22.14 Syntax 22-10 does not list "
                "(docs/systemverilog/2017/md/section-22-compiler-directives.md:851-862; 'It shall be an error if an "
                "implementation does not recognize the version_specifier', :869-870) - Verilog-AMS surface, outside "
                "the IEEE 1800 scope (VERILOG-AMS tree parked); since 1.0.217 the parser refuses the directive itself")
    if rel.endswith(".svh") and suite not in DESIGN_SUITES:
        # Generic across the test suites: .svh files are `include payloads,
        # not standalone compilation units (may be bare fragments).
        return ("chained_only", f"{suite}: .svh include payload, not a standalone unit")
    if suite in DESIGN_SUITES:
        return ("chained_only",
                "design corpus parsed in isolation - honest adjudication "
                "needs include/define chaining (leaf .4)")
    if suite == "sv-tests":
        return expect_sv_tests(rel, text)
    if suite == "verilator":
        return vidx.expect(rel)
    if suite == "slang":
        return expect_slang(rel)
    if suite == "verible":
        return expect_verible(rel, text)
    if suite == "ispras-sv-tests":
        return expect_ispras(rel, text)
    if suite == "iverilog":
        return ividx.expect(rel)
    if suite == "sv2v":
        return expect_sv2v(rel)
    if suite == "Surelog":
        return sidx.expect(rel)
    raise SystemExit(f"unknown suite {suite!r} in results.tsv")


def finish_row(suite, rel, observed, text, expected, basis, positions):
    """The edition-INDEPENDENT tail every lane runs after its key: the not-source-text guard, the
    preprocessor-dependency flag with its .12a refutation, the .13s reject-arm demotion, and the
    verdict. One body for three lanes (SV-CORPUS-GRAD.14b.2); byte-identical to the two inline
    copies it replaced (the tracked manifests re-derive unchanged)."""
    expected, basis = apply_not_source_text(suite, rel, text, expected, basis)
    dep_flag = ""
    if expected in ("must_accept", "must_reject"):
        dep_flag = preproc_dependency(text)
    if expected == "must_reject":
        # SV-CORPUS-GRAD.12c.3 F3 - the WHOLE-FILE arm, structurally unable to OBTAIN a
        # failure position; .13s hands it one DECIDED boolean and nothing else, computed
        # here by the same predicate the must_accept arm below uses (see the docstring).
        demotion = reject_arm_demotion(
            dep_flag,
            position_refutes_dependency=bool(
                dep_flag and observed == "fail"
                and not svpp_can_explain_failure(text, positions.get((suite, rel)))))
        if demotion is not None:
            expected, basis_suffix = demotion
            basis += basis_suffix
    if expected != "must_accept":
        dep_flag = ""
    elif (dep_flag and observed == "fail"
            and not svpp_can_explain_failure(text, positions.get((suite, rel)))):
        # SV-CORPUS-GRAD.12a - the file DOES carry a preprocessor construct, but
        # the parse stops at a point svpp provably cannot reach or alter, so the
        # dependency is real and irrelevant to THIS failure. Dropping the flag
        # returns the row to `unexplained_rejects_valid`, where it always belonged.
        dep_flag = ""
        basis += (" - .12a: parse stops where svpp cannot alter the text, so the "
                  "preprocessor dependency does not explain THIS failure")
    verdict = adjudicate(expected, observed, dep_flag)
    return (suite, rel, observed, expected, verdict, basis)


# ---- the sv_2023 lane's key (SV-CORPUS-GRAD.14b.2) ---------------------------------------------
# The lane judges IEEE 1800-2023 over the SAME rows as the sv_2017 lane. Its key is the 2017 key
# read edition-relatively: (a) a row the 2017 key rules an EDITION CONTRAST (its basis cites the
# 1800-2023 production that derives the text) is must_accept here; (b) the eleven rows below are
# must_reject on 2017 because 1800-2017 has NO production for the text, while 1800-2023 derives
# it and the upstream `_fail` golden names a SEMANTIC violation (6.20.3, 7.3, 8.20) that no parse
# can see - must_accept here, the .13x.a principle; (c) every other row inherits its 2017 ruling,
# and a must_reject inherited on a 1800-2017 production carries an explicit RE-READ-OWED marker
# (.14b.3) so the summary can count what this lane has NOT yet judged for itself.
_A23 = "docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md"
_TP23 = (f"IEEE 1800-2023 A.2.1.1 type_parameter_declaration ::= type [ forward_type ] list_of_type_assignments "
         f"({_A23}:721), forward_type ::= enum | struct | union | class | interface class ({_A23}:803), "
         f"type_assignment ::= type_identifier [ = data_type_or_incomplete_class_scoped_type ] ({_A23}:1059)")
SV2023_KEY_OVERRIDES = {}
for _n in ("class", "enum", "struct", "union"):
    for _k in ("1", "2"):
        SV2023_KEY_OVERRIDES[("iverilog", f"ivtest/ivltests/sv_type_param_restrict_{_n}_fail{_k}.v")] = (
            "must_accept",
            f"sv_2023 lane (.14b.2): `parameter type {_n} T = ...` IS derivable in the edition this lane judges - {_TP23}; "
            "the ivtest golden fails on 6.20.3's restriction of the DEFAULT's kind (a struct default for an enum "
            "restriction, ...) - SEMANTIC, after the parse. ACCEPTS under sv_2023 (the .14a census); on the 2017 lane the "
            "same text has no derivation and is must_reject there.")
SV2023_KEY_OVERRIDES[("iverilog", "ivtest/ivltests/sv_soft_packed_union_fail1.v")] = (
    "must_accept",
    f"sv_2023 lane (.14b.2): `typedef union soft {{ logic [31:0] w = 32'h1; ... }}` IS derivable in the edition this lane judges - "
    f"IEEE 1800-2023 A.2.2.1 struct_union ::= struct | union [ soft | tagged ] ({_A23}:916-920), struct_union_member ::= "
    f"{{ attribute_instance }} [ random_qualifier ] data_type_or_void list_of_variable_decl_assignments ; ({_A23}:922-925) admits "
    "the initializer syntactically; the ivtest golden fails on the SEMANTIC rule about a packed/soft union member's initial value "
    "(7.3). ACCEPTS under sv_2023 (the .14a census); must_reject on the 2017 lane (no `soft` in 1800-2017).")
SV2023_KEY_OVERRIDES[("verilator", "test_regress/t/t_class_override_bad.v")] = (
    "must_accept",
    f"sv_2023 lane (.14b.2): `function :extends int get_e;` IS derivable in the edition this lane judges - IEEE 1800-2023 A.1.9 "
    f"dynamic_override_specifiers ({_A23}:1210-1217); its golden names 8.20 prose (no base method to extend) - SEMANTIC. The .13x.d "
    "entry recorded this: on an sv_2023 lane this row is must_accept. ACCEPTS under sv_2023 (the .14a census).")
SV2023_KEY_OVERRIDES[("verilator", "test_regress/t/t_param_type_fwd_bad.v")] = (
    "must_reject",
    f"sv_2023 lane (.3.37, CORRECTING .14b.2): `parameter type enum E_t;` in a module body - the forward type IS 1800-2023 syntax ({_TP23}) but the default is omitted outside a parameter_port_list, which IEEE 1800-2023 Annex A footnote 22 (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:3483-3484, the twin of 2017's footnote 18): the data_type may be omitted from a type_assignment ONLY within a parameter_port_list forbids. REJECTS under sv_2023 on 1.0.211 (the .14a census).")
SV2023_KEY_OVERRIDES[("verilator", "test_regress/t/t_param_type_fwd.v")] = (
    "must_reject",
    f"sv_2023 lane (.3.37, CORRECTING .14b): line 20 `parameter type enum E_t;` in a module body - the forward type IS 1800-2023 syntax ({_TP23}) but the default is omitted outside a parameter_port_list, which IEEE 1800-2023 Annex A footnote 22 (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:3483-3484, the twin of 2017's footnote 18): the data_type may be omitted from a type_assignment ONLY within a parameter_port_list forbids. REJECTS under sv_2023 on 1.0.211 (the .14a census).")
REREAD_OWED = "its 1800-2023 re-read is owed (.14b.3)"
PARITY_TSV = Path(__file__).resolve().parent.parent.parent / "docs/tasks/artifacts/sv_corpus_grad/sv2023_lane/parity.tsv"
_PARITY = None


def parity_verdicts() -> dict:
    """(suite, relpath) -> (verdict, productions) from the .14b.3 production-parity artifact — empty when absent."""
    global _PARITY
    if _PARITY is None:
        _PARITY = {}
        if PARITY_TSV.is_file():
            for line in PARITY_TSV.read_text(encoding="utf-8").splitlines()[1:]:
                s_, rel, verdict, prods = (line.split("\t") + ["", ""])[:4]
                _PARITY[(s_, rel)] = (verdict, prods)
    return _PARITY


# .14b.3, the FIVE rows whose cited 1800-2017 production reads differently in 1800-2023 (or was renamed), each
# re-read by hand against the 2023 text: in every case the 2023 change does not touch the reason the text
# has no derivation, so the 2017 ruling CARRIES OVER with the 2023 production cited.
SV2023_REREAD_BY_HAND = {
    ("sv2v", "test/error/severity_task_arg.sv"):
        f"1800-2023 renames the production: elaboration_severity_system_task ::= severity_system_task (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:283), severity_system_task ::= "
        f"$fatal [ ( finish_number [ , list_of_arguments ] ) ] ; | ... (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:274) - the same shape as 2017's elaboration_system_task; "
        "`$fatal(.x(\"x\"))` at module level still needs a finish_number (0 | 1 | 2) first, and a named argument is not one",
    ("verilator", "test_regress/t/t_class_extends_bad.v"):
        f"1800-2023 class_declaration ::= [ virtual ] class [ final_specifier ] class_identifier [ parameter_port_list ] [ extends class_type "
        f"[ ( [ list_of_arguments | default ] ) ] ] ... (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:175) - still ONE class_type after `extends`; `extends Base1, Base2` has no derivation",
    ("verilator", "test_regress/t/t_param_implicit_bad.v"):
        f"1800-2023 parameter_port_declaration ::= parameter_declaration | local_parameter_declaration | data_type list_of_param_assignments | "
        f"type_parameter_declaration (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:215) - `[7:0] PAR1 = 1` is a bare packed dimension, neither a data_type nor a type parameter",
    ("verilator", "test_regress/t/t_param_type_bad.v"):
        f"1800-2023 local_parameter_declaration ::= ... | localparam type_parameter_declaration (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:709), type_parameter_declaration ::= type "
        f"[ forward_type ] list_of_type_assignments (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:721), type_assignment ::= type_identifier [ = data_type_or_incomplete_class_scoped_type ] "
        f"(docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1059) - `localparam type bad2 = 2;` assigns a NUMBER where a data type or an incomplete class-scoped type is required",
    ("verilator", "test_regress/t/t_type_compare_bad.v"):
        f"1800-2023 case_expression ::= expression (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:2234) and primary (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:3079-3094) still carry no type_reference - it lives in "
        f"constant_primary alone, as in 2017; a procedural `case (type(real))` has no derivation in either edition (RULED "
        "edition-independent by .3.32, PGEN-SV-CORPUS-GRAD-0375)",
    # SV-CORPUS-GRAD.3.48 (PGEN-SV-CORPUS-GRAD-0378): the four `[N]` packed-dimension rows read under 1800-2023 - A.2.5 unchanged.
    ("sv2v", "test/core/multipack_struct_cast.sv"):
        f"1800-2023 A.2.5 packed_dimension ::= [ constant_range ] | unsized_dimension (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1085-1089) and "
        f"unpacked_dimension ::= [ constant_range ] | [ constant_expression ] (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1079-1083) - the same two productions as 2017: "
        "a packed dimension is a range in both editions and the size form is unpacked's alone; the `range ::=` in the 2017 basis names IEEE 1364-2005 A.2.5 (a 2005 production, not a 2017 one), which is why parity could not read it",
    ("sv2v", "test/core/struct_param.sv"):
        f"1800-2023 A.2.5 packed_dimension ::= [ constant_range ] | unsized_dimension (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1085-1089) and "
        f"unpacked_dimension ::= [ constant_range ] | [ constant_expression ] (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1079-1083) - the same two productions as 2017: "
        "a packed dimension is a range in both editions and the size form is unpacked's alone; the `range ::=` in the 2017 basis names IEEE 1364-2005 A.2.5 (a 2005 production, not a 2017 one), which is why parity could not read it",
    ("sv2v", "test/core/unbased_unsized_default.sv"):
        f"1800-2023 A.2.5 packed_dimension ::= [ constant_range ] | unsized_dimension (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1085-1089) and "
        f"unpacked_dimension ::= [ constant_range ] | [ constant_expression ] (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1079-1083) - the same two productions as 2017: "
        "a packed dimension is a range in both editions and the size form is unpacked's alone; the `range ::=` in the 2017 basis names IEEE 1364-2005 A.2.5 (a 2005 production, not a 2017 one), which is why parity could not read it" + f"; 1800-2023 net_port_type ::= [ net_type ] data_type_or_implicit | nettype_identifier (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:893-897) adds the nettype arm and keeps implicit_data_type ::= [ signing ] {{ packed_dimension }} (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:850) - the port still carries a packed range, never a size",
    ("iverilog", "ivtest/ivltests/display_bug.v"):
        f"1800-2023 A.2.5 packed_dimension ::= [ constant_range ] | unsized_dimension (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1085-1089) and "
        f"unpacked_dimension ::= [ constant_range ] | [ constant_expression ] (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1079-1083) - the same two productions as 2017: "
        "a packed dimension is a range in both editions and the size form is unpacked's alone; the `range ::=` in the 2017 basis names IEEE 1364-2005 A.2.5 (a 2005 production, not a 2017 one), which is why parity could not read it",
    ("verilator", "test_regress/t/t_type_compare.v"):
        f"1800-2023 conditional_statement ::= [ unique_priority ] if ( cond_predicate ) statement_or_null ... (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:2203-2210), "
        f"cond_predicate ::= expression_or_cond_pattern {{ &&& expression_or_cond_pattern }} (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:2211), "
        f"case_expression ::= expression (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:2234) and primary "
        f"(docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:3079-3094) still carry no type_reference - it lives in constant_primary alone "
        f"(docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:3066, footnote 45 :3525-3527), and 6.23 "
        f"(docs/systemverilog/2023/md/section-6-data-types.md:2936-2975) keeps the constant-expression sentence and the generate example unchanged "
        "while adding type(this) - the .3.32 ruling is edition-independent",
    ("verilator", "test_regress/t/t_type_expression_compare.v"):
        f"1800-2023 conditional_statement ::= [ unique_priority ] if ( cond_predicate ) statement_or_null ... (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:2203-2210), "
        f"cond_predicate ::= expression_or_cond_pattern {{ &&& expression_or_cond_pattern }} (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:2211), "
        f"case_expression ::= expression (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:2234) and primary "
        f"(docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:3079-3094) still carry no type_reference - it lives in constant_primary alone "
        f"(docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:3066, footnote 45 :3525-3527), and 6.23 "
        f"(docs/systemverilog/2023/md/section-6-data-types.md:2936-2975) keeps the constant-expression sentence and the generate example unchanged "
        "while adding type(this) - the .3.32 ruling is edition-independent",
    ("sv-tests", "tests/chapter-6/6.23--type_op_compare.sv"):
        f"1800-2023 conditional_statement ::= [ unique_priority ] if ( cond_predicate ) statement_or_null ... (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:2203-2210), "
        f"cond_predicate ::= expression_or_cond_pattern {{ &&& expression_or_cond_pattern }} (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:2211), "
        f"case_expression ::= expression (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:2234) and primary "
        f"(docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:3079-3094) still carry no type_reference - it lives in constant_primary alone "
        f"(docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:3066, footnote 45 :3525-3527), and 6.23 "
        f"(docs/systemverilog/2023/md/section-6-data-types.md:2936-2975) keeps the constant-expression sentence and the generate example unchanged "
        "while adding type(this) - the .3.32 ruling is edition-independent",
    # ---- SV-CORPUS-GRAD.3.51: the 2023 re-readings of the triage rulings ----
    ('verilator', 'test_regress/t/t_param_no_parentheses.v'):
        'IEEE 1800-2023 A.4.1.1 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1804) - the same production as 2017; the .3.51 basis cites the 2017 and 2023 Annex A lines',
    ('verilator', 'test_regress/t/t_param.v'):
        'IEEE 1800-2023 A.4.1.1 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1804) - the same production as 2017; the .3.51 basis cites the 2017 and 2023 Annex A lines',
    ('sv2v', 'test/core/trailing_commas.sv'):
        'IEEE 1800-2023 A.1.3 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:208-213, :223) - the same production as 2017; the .3.51 basis cites the 2017 and 2023 Annex A lines',
    ('sv2v', 'test/core/for_loop_inits.sv'):
        'IEEE 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:2321-2325, :2055) - the same production as 2017; the .3.51 basis cites the 2017 and 2023 Annex A lines',
    ('iverilog', 'ivtest/ivltests/pr3366217d.v'):
        'IEEE 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:858-861, :3180-3184) - the same production as 2017; the .3.51 basis cites the 2017 and 2023 Annex A lines',
    ('iverilog', 'ivtest/ivltests/pr3515542.v'):
        'IEEE 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:3231, :3188) - the same production as 2017; the .3.51 basis cites the 2017 and 2023 Annex A lines',
    ('Surelog', 'tests/HierPathModule/dut.sv'):
        'IEEE 1800-2023 Annex B the same reserved set (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:3599) - the same production as 2017; the .3.51 basis cites the 2017 and 2023 Annex A lines',
    ('verilator', 'test_regress/t/t_property_named.v'):
        'IEEE 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1340-1343, :1420-1424) - the same production as 2017; the .3.51 basis cites the 2017 and 2023 Annex A lines',
    ('verilator', 'test_regress/t/t_class_static_member_sel.v'):
        'IEEE 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:3146-3151) - the same production as 2017; the .3.51 basis cites the 2017 and 2023 Annex A lines',
    ('verilator', 'test_regress/t/t_delay.v'):
        'IEEE 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:989-995, :3355) - the same production as 2017; the .3.51 basis cites the 2017 and 2023 Annex A lines',
    ('verilator', 'test_regress/t/t_lparam_assign_iface_typedef4.v'):
        'IEEE 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:838; the rule docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:825-849) - the same production as 2017; the .3.51 basis cites the 2017 and 2023 Annex A lines',
    ('verilator', 'test_regress/t/t_lparam_assign_iface_typedef_nested.v'):
        'IEEE 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:838; the rule docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:825-849) - the same production as 2017; the .3.51 basis cites the 2017 and 2023 Annex A lines',
    ('verilator', 'test_regress/t/t_lparam_assign_iface_typedef_nested6.v'):
        'IEEE 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:838; the rule docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:825-849) - the same production as 2017; the .3.51 basis cites the 2017 and 2023 Annex A lines',
    ('verilator', 'test_regress/t/t_param_type_from_iface_struct.v'):
        'IEEE 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:838; the rule docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:825-849) - the same production as 2017; the .3.51 basis cites the 2017 and 2023 Annex A lines',
    ('iverilog', 'ivtest/ivltests/sv_wildcard_import2.v'):
        'IEEE 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:2189-2192, :3310) - the same production as 2017; the .3.51 basis cites the 2017 and 2023 Annex A lines',
    ('iverilog', 'ivtest/ivltests/sv_wildcard_import3.v'):
        'IEEE 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:2189-2192, :3310) - the same production as 2017; the .3.51 basis cites the 2017 and 2023 Annex A lines',
    ('verilator', 'test_regress/t/t_typedef_package.v'):
        'IEEE 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:851-856) - the same production as 2017; the .3.51 basis cites the 2017 and 2023 Annex A lines',
    ('sv2v', 'test/nosim/assert.sv'):
        'IEEE 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:2406-2409) - the same production as 2017; the .3.51 basis cites the 2017 and 2023 Annex A lines',
    ('sv2v', 'test/core/delay.sv'):
        'IEEE 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1085-1089) - the same production as 2017; the .3.51 basis cites the 2017 and 2023 Annex A lines',
    ('sv2v', 'test/core/struct_unit_array.sv'):
        'IEEE 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:838, :1085-1089) - the same production as 2017; the .3.51 basis cites the 2017 and 2023 Annex A lines',
    ('sv2v', 'test/core/enum.sv'):
        'IEEE 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:851-856) - the same production as 2017; the .3.51 basis cites the 2017 and 2023 Annex A lines',
    ('iverilog', 'ivtest/ivltests/sv_type_identifier_attribute_target.v'):
        'IEEE 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:64-75) - the same production as 2017; the .3.51 basis cites the 2017 and 2023 Annex A lines',
    ('verible', 'verible/verilog/tools/kythe/testdata/foreach.sv'):
        'IEEE 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:2319) - the same production as 2017; the .3.51 basis cites the 2017 and 2023 Annex A lines',
    ('iverilog', 'ivtest/ivltests/sf1289.v'):
        'IEEE 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:2311-2319) - the same production as 2017; the .3.51 basis cites the 2017 and 2023 Annex A lines',
    ('sv-tests', 'tests/chapter-6/6.9.2--vector_vectored_inv.sv'):
        'IEEE 1800-2023 A.2.1.3 net_declaration (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:785-789) - the same production as 2017; `vectored` follows a net_type',
    # ---- SV-CORPUS-GRAD.3.56: the 2023 re-readings ----
    ('Surelog', 'tests/IndexedSelect/dut.sv'):
        'IEEE 1800-2023 the same (generate_item docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1910-1914, module_or_generate_item docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:287) - the same production as 2017; the .3.56 basis cites the 2017 and 2023 lines',
    ('Surelog', 'tests/TranslateOff/dut.sv'):
        'IEEE 1800-2023 the same (module_item docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:301) - the same production as 2017; the .3.56 basis cites the 2017 and 2023 lines',
    ('verilator', 'test_regress/t/t_cover_fsm_sel_assign.v'):
        'IEEE 1800-2023 the same (module_item docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:301, non_port_module_item docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:330, module_or_generate_item docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:287) - the same production as 2017; the .3.56 basis cites the 2017 and 2023 lines',
    ('verilator', 'test_regress/t/t_simulate_array.v'):
        'IEEE 1800-2023 the same (module_item docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:301, non_port_module_item docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:330) - the same production as 2017; the .3.56 basis cites the 2017 and 2023 lines',
    ('verilator', 'test_regress/t/t_gen_for.v'):
        'IEEE 1800-2023 the same (generate_region docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1865, generate_block docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1902-1908, generate_item docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1910-1914) - the same production as 2017; the .3.56 basis cites the 2017 and 2023 lines',
    ('verilator', 'test_regress/t/t_interface_gen11.v'):
        'IEEE 1800-2023 the same (generate_region docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1865, generate_block docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1902-1908, generate_item docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1910-1914) - the same production as 2017; the .3.56 basis cites the 2017 and 2023 lines',
    ('verilator', 'test_regress/t/t_interface_gen13.v'):
        'IEEE 1800-2023 the same (generate_region docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1865, generate_block docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1902-1908, generate_item docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1910-1914) - the same production as 2017; the .3.56 basis cites the 2017 and 2023 lines',
    ('verilator', 'test_regress/t/t_interface_gen6.v'):
        'IEEE 1800-2023 the same (generate_region docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1865, generate_block docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1902-1908, generate_item docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1910-1914) - the same production as 2017; the .3.56 basis cites the 2017 and 2023 lines',
    ('verilator', 'test_regress/t/t_covergroup_unsup.v'):
        'IEEE 1800-2023 the same (covergroup_declaration docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1517, coverage_spec_or_option docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1527) - the same production as 2017; the .3.56 basis cites the 2017 and 2023 lines',
    ('verilator', 'test_regress/t/t_udp_nonsequential.v'):
        'IEEE 1800-2023 the same (combinational_entry docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1971, sequential_entry docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1975, level_input_list docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1979; 22.5.1 docs/systemverilog/2023/md/section-22-compiler-directives.md:173) - the same production as 2017; the .3.56 basis cites the 2017 and 2023 lines',
    ('verilator', 'test_regress/t/t_pipe_filter.v'):
        'IEEE 1800-2023 the same (description docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:37) - the same production as 2017; the .3.56 basis cites the 2017 and 2023 lines',
    ('verilator', 'test_regress/t/t_gate_primitives_implicit_net.v'):
        'IEEE 1800-2023 the same (22.8 docs/systemverilog/2023/md/section-22-compiler-directives.md:750) - the same production as 2017; the .3.56 basis cites the 2017 and 2023 lines',
    # ---- SV-CORPUS-GRAD.3.55: the 2023 re-readings ----
    ('sv2v', 'test/core/interface_generate.sv'):
        'IEEE 1800-2023 the same (interface_or_generate_item docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:407-411, non_port_interface_item docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:425-432) - the same production as 2017; the .3.55 basis cites the 2017 and 2023 lines',
    ('sv2v', 'test/core/port_default.sv'):
        'IEEE 1800-2023 the same (input_declaration docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:732-736; list_of_port_identifiers docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1008, list_of_variable_identifiers docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1023) - the same production as 2017; the .3.55 basis cites the 2017 and 2023 lines',
    ('sv2v', 'test/core/struct_scope.sv'):
        'IEEE 1800-2023 the same (generate_region docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1865, generate_block docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1902-1908, generate_item docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1910-1914) - the same production as 2017; the .3.55 basis cites the 2017 and 2023 lines',
    ('iverilog', 'ivtest/ivltests/module_output_port_sv_var2.v'):
        'IEEE 1800-2023 the same (module_declaration docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:93-99, module_nonansi_header docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:81-84, module_ansi_header docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:87-90, non_port_module_item docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:330) - the same production as 2017; the .3.55 basis cites the 2017 and 2023 lines',
    ('ispras-sv-tests', 'ieee-1800-2012/06/06.07.01_02.sv'):
        'IEEE 1800-2023 the same (module_declaration docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:93-99, module_ansi_header docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:87-90, non_port_module_item docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:330) - the same production as 2017; the .3.55 basis cites the 2017 and 2023 lines',
    ('verilator', 'test_regress/t/t_math_pow6.v'):
        'IEEE 1800-2023 the same (output_declaration docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:738-742, list_of_port_identifiers docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1008, list_of_variable_port_identifiers docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1028) - the same production as 2017; the .3.55 basis cites the 2017 and 2023 lines',
    # ---- SV-CORPUS-GRAD.3.54: the 2023 re-readings ----
    ('verilator', 'test_regress/t/t_std_randomize_with.v'):
        'IEEE 1800-2023 the same (18.12 Syntax 18-11 docs/systemverilog/2023/md/section-18-constrained-random-value-generation.md:2105-2108; docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:2940, :2933-2935) - the same production as 2017; the .3.54 basis cites the 2017 and 2023 lines',
    ('verilator', 'test_regress/t/t_randomize_arraysel_membersel.v'):
        'IEEE 1800-2023 the same (18.12 Syntax 18-11 docs/systemverilog/2023/md/section-18-constrained-random-value-generation.md:2105-2108; docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:2940) - the same production as 2017; the .3.54 basis cites the 2017 and 2023 lines',
    ('verilator', 'test_regress/t/t_class_extern.v'):
        'IEEE 1800-2023 function_prototype ::= function [ dynamic_override_specifiers ] data_type_or_void function_identifier ... (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1141-1143) - the return type still mandatory; class_method docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:531-540, method_prototype docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:588-591 - the same production as 2017',
    ('verilator', 'test_regress/t/t_interface_modport_export.v'):
        'IEEE 1800-2023 the same (extern_tf_declaration docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:413-417; function_prototype docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1141-1143) - the same production as 2017; the .3.54 basis cites the 2017 and 2023 lines',
    ('verible', 'verible/verilog/tools/kythe/testdata/program.sv'):
        'IEEE 1800-2023 the same (list_of_port_connections docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1823-1827, named_port_connection docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:1830-1833) - the same production as 2017; the .3.54 basis cites the 2017 and 2023 lines',
    ('ispras-sv-tests', 'ieee-1800-2012/08/08.26.04_01.sv'):
        'IEEE 1800-2023 the same (class_method docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:531-540) - a bodiless virtual method still needs pure or extern; the same production as 2017',
    # ---- SV-CORPUS-GRAD.3.52: the 2023 re-readings ----
    ('verilator', 'test_regress/t/t_foreach_array.v'):
        'IEEE 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:2319, :2345; 12.7.3 docs/systemverilog/2023/md/section-12-procedural-programming-statements.md:1037-1043, :1054, :1078) - the same production as 2017; the .3.52 basis cites the 2017 and 2023 lines',
    ('verilator', 'test_regress/t/t_foreach_nested.v'):
        'IEEE 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:2319, :2345; 12.7.3 docs/systemverilog/2023/md/section-12-procedural-programming-statements.md:1037-1043, :1054, :1078) - the same production as 2017; the .3.52 basis cites the 2017 and 2023 lines',
    ('verilator', 'test_regress/t/t_constraint_operators.v'):
        'IEEE 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:2988-2996, docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:3079) - the same production as 2017; the .3.52 basis cites the 2017 and 2023 lines',
    ('verilator', 'test_regress/t/t_inst_port_complex_unsup.v'):
        'IEEE 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:265-271) - the same production as 2017; the .3.52 basis cites the 2017 and 2023 lines',
    ('sv2v', 'test/core/interface_array.sv'):
        'IEEE 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:3310, :3120) - the same production as 2017; the .3.52 basis cites the 2017 and 2023 lines',
    ('verilator', 'test_regress/t/t_setuphold.v'):
        'IEEE 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:2709-2712, :2794) - the same production as 2017; the .3.52 basis cites the 2017 and 2023 lines',
    ('verible', 'verible/verilog/tools/kythe/testdata/event_control.sv'):
        'IEEE 1800-2023 event_control ::= clocking_event | @ * | @ ( * ) and clocking_event ::= @ ps_identifier | @ hierarchical_identifier | @ ( event_expression ) (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:2140-2151) - the same - the same production as 2017; the .3.52 basis cites the 2017 and 2023 lines',
    ('verilator', 'test_regress/t/t_event_control_pass.v'):
        'IEEE 1800-2023 the same with a trailing nonrange_select (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:2189-2192), still no call - the same production as 2017; the .3.52 basis cites the 2017 and 2023 lines',
    ('verible', 'verible/verilog/tools/kythe/testdata/function_port_dimensions.sv'):
        'IEEE 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:825-829, :885) - the same production as 2017; the .3.52 basis cites the 2017 and 2023 lines',
    ('verible', 'verible/verilog/tools/kythe/testdata/primitive_type_with_dimensions.sv'):
        'IEEE 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:825-829, :885) - the same production as 2017; the .3.52 basis cites the 2017 and 2023 lines',
    ('Surelog', 'tests/EnumParenType/dut.sv'):
        'IEEE 1800-2023 the same (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:756, :825-849) - the same production as 2017; the .3.52 basis cites the 2017 and 2023 lines',
    ('Surelog', 'tests/ClassTypeParamAlias/dut.sv'):
        'IEEE 1800-2023 the same footnote (docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:3483-3484) - the same production as 2017; the .3.52 basis cites the 2017 and 2023 lines',
}


def key_2023(suite, rel, expected17, basis17):
    """(expected, basis) for the sv_2023 lane from the row's sv_2017 ruling - see the block comment above."""
    if (suite, rel) in SV2023_KEY_OVERRIDES:
        e, b = SV2023_KEY_OVERRIDES[(suite, rel)]
        return (e, b + " 2017 basis: " + basis17)
    if "EDITION CONTRAST" in basis17:
        return ("must_accept",
                "sv_2023 lane (.14b.2): the 2017 key rules this row an EDITION CONTRAST and cites the IEEE 1800-2023 production "
                "that derives the text - legal in the edition this lane judges. 2017 basis: " + basis17)
    if expected17 == "must_reject" and "section-41-data-read-api.md" in basis17:
        if (suite, rel) in SV2023_REREAD_BY_HAND:
            return (expected17,
                    "sv_2023 lane (.14b.3): the 2017 ruling CARRIES OVER, re-read by hand against IEEE 1800-2023 Annex A because a "
                    "cited 2017 production reads differently there - " + SV2023_REREAD_BY_HAND[(suite, rel)] + ". 2017 basis: " + basis17)
        verdict, prods = parity_verdicts().get((suite, rel), ("", ""))
        if verdict == "IDENTICAL":
            return (expected17,
                    "sv_2023 lane (.14b.3): the 2017 ruling CARRIES OVER mechanically - every 1800-2017 production the 2017 basis cites "
                    "reads IDENTICALLY in 1800-2023 Annex A (production_parity.py: " + prods + "). 2017 basis: " + basis17)
        return (expected17,
                "sv_2023 lane (.14b.2): INHERITED from the 2017 key - a must_reject on an IEEE 1800-2017 production; "
                + REREAD_OWED + ", and until then this row's verdict is the 2017 lane's. 2017 basis: " + basis17)
    if expected17 == "must_reject":
        return (expected17,
                "sv_2023 lane (.14b.2): INHERITED from the 2017 key - a must_reject on a suite's own oracle (a 2017-era tool or "
                "golden); " + REREAD_OWED + ". 2017 basis: " + basis17)
    return (expected17, "sv_2023 lane (.14b.2): inherited from the 2017 key. 2017 basis: " + basis17)


def main():
    # SV-CORPUS-GRAD.12c.3 F3 - the two-caller split is asserted BEFORE any row is adjudicated,
    # so a broken split refuses to produce a manifest rather than producing a wrong one.
    _self_check_arm_split()
    _self_check_not_source_text()
    ap = argparse.ArgumentParser(description=__doc__)
    root = Path(__file__).resolve().parent.parent.parent  # repo root
    ap.add_argument("--self-test", action="store_true",
                    help="SV-CORPUS-GRAD.13e.3: drive the vendor-extension guard RED on "
                         "synthetic descriptors and exit (a guard fires on 0 tracked rows "
                         "at HEAD, so this is the only evidence it works)")
    ap.add_argument("--results", default=root / "stimuli/sv/characterization/results.tsv",
                    type=Path)
    ap.add_argument("--subs-root", default=root / "stimuli/sv/subs", type=Path)
    ap.add_argument("--out-manifest",
                    default=root / "stimuli/sv/characterization/adjudication_manifest.tsv",
                    type=Path)
    ap.add_argument("--out-summary",
                    default=root / "stimuli/sv/characterization/adjudication_summary.md",
                    type=Path)
    # SV-CORPUS-GRAD.12a - the per-file `furthest_position` sidecar the runner emits.
    # OPTIONAL by design: an older tree without it adjudicates exactly as before, and
    # says so on stderr rather than silently reverting to the whole-file test.
    ap.add_argument("--positions",
                    default=root / "stimuli/sv/characterization/positions.tsv",
                    type=Path)
    ap.add_argument("--positions-v2005",
                    default=root / "stimuli/sv/characterization/positions_v2005.tsv",
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
    ap.add_argument("--report",
                    default=root / "stimuli/sv/characterization/characterization.md",
                    type=Path,
                    help="the TRACKED sv-lane report whose instrument identity names the raw "
                         "dumps this run may read (SV-CORPUS-GRAD.13e.6)")
    ap.add_argument("--store-gate-cache", type=Path,
                    default=root / "stimuli/sv/characterization/store_gate_cache.tsv",
                    help="SV-CORPUS-GRAD.13w: the tracked memo of store-gate measurements "
                         "(re-measured per row when its bytes or the parser change)")
    ap.add_argument("--store-gate-jobs", type=int, default=8)
    ap.add_argument("--report-v2005",
                    default=root / "stimuli/sv/characterization/characterization_v2005.md",
                    type=Path,
                    help="the TRACKED sv2005-lane report, same role as --report")
    # --- the sv_2023 profile lane (leaf .14b.2) ---
    ap.add_argument("--results-sv2023",
                    default=root / "stimuli/sv/characterization/results_sv2023.tsv", type=Path,
                    help="raw outcomes of the runner's sv2023 mode (the SAME population as results.tsv "
                         "under --profile sv_2023); the sv2023 manifest is built only when this file exists")
    ap.add_argument("--positions-sv2023",
                    default=root / "stimuli/sv/characterization/positions_sv2023.tsv", type=Path)
    ap.add_argument("--report-sv2023",
                    default=root / "stimuli/sv/characterization/characterization_sv2023.md", type=Path,
                    help="the TRACKED sv2023-lane report, same role as --report")
    ap.add_argument("--out-manifest-sv2023",
                    default=root / "stimuli/sv/characterization/adjudication_manifest_sv2023.tsv", type=Path)
    ap.add_argument("--out-summary-sv2023",
                    default=root / "stimuli/sv/characterization/adjudication_summary_sv2023.md", type=Path)
    args = ap.parse_args()

    if args.self_test:
        return (_self_test_extension_guard(args.subs_root / "iverilog" / "ivtest")
                | _self_test_store_gate()
                | _self_test_edition_keyword())

    verify_outcome_identity(args.report, [args.results, args.positions],
                            rerun="stimuli/run_external_corpus.sh sv")

    raw_positions = load_positions(args.positions, "positions.tsv")
    positions = {}
    rows = []
    for line in args.results.read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        suite, observed, path = line.split("\t")
        # ⛔ Both markers must be LEADING-SLASH-FREE so they match column 3 in BOTH
        # spellings: the absolute paths older artifacts carry AND the repo-root-relative
        # ones the runner has emitted since CORPUS-GRAD-ALL.2.1. The uvm marker used to be
        # written "/stimuli/sv/uvm/", which a relative row ("stimuli/sv/uvm/…") cannot
        # contain, so re-adjudicating a post-.2.1 run died on its first uvm-core row
        # (`unrecognized results path shape`) — measured in SV-CORPUS-GRAD.10. The
        # "backward-compatible by construction" claim in run_external_corpus.sh held for
        # the `/subs/<suite>/` arm (a relative row still contains it) and was never true
        # for this one; a shared infix is only compatible with a spelling that has it.
        if f"/subs/{suite}/" in path:
            rel = path.split(f"/subs/{suite}/", 1)[1]
        elif UVM_FOLD_MARKER in path:
            # the uvm-core fold (leaf .8): plain tracked files, not a submodule
            rel = path.split(UVM_FOLD_MARKER, 1)[1]
        else:
            raise SystemExit(f"unrecognized results path shape: {path!r}")
        rows.append((suite, rel, observed))
        # .12a: re-key the sidecar through the SAME path->rel mapping this loop just
        # performed, so the two spellings cannot drift apart the way the uvm marker did.
        if path in raw_positions:
            positions[(suite, rel)] = raw_positions[path]
    rows.sort()

    vidx = VerilatorIndex(args.subs_root / "verilator/test_regress/t")
    ividx = IvtestIndex(args.subs_root / "iverilog/ivtest")
    sidx = SurelogIndex(args.subs_root / "Surelog/tests")

    resolve = resolve_corpus_path(args)
    manifest = []
    for suite, rel, observed in rows:
        text = read_text(resolve(suite, rel))
        expected, basis = key_2017(suite, rel, text, vidx, ividx, sidx)
        manifest.append(finish_row(suite, rel, observed, text, expected, basis, positions))
    manifest = edition_keyword_pass("sv_2017", manifest, resolve, positions)

    store_gate_cache = _sg().load_cache(args.store_gate_cache)
    manifest = store_gate_pass("sv_2017", manifest, resolve_corpus_path(args), args.report,
                               store_gate_cache, args.store_gate_jobs)
    _sg().save_cache(args.store_gate_cache, store_gate_cache)

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
                 "Explained divergences are svpp/chaining/timeout-owned or store-gate-owned "
                 "(`.13w`: the row parses once its undeclared type is declared) with named "
                 "causes; deferred rows adjudicate in their owning lanes "
                 "(leaf .4 chaining, SVPP lane).")
    lines.append("")
    args.out_summary.write_text("\n".join(lines) + "\n", encoding="utf-8")

    print(f"manifest: {args.out_manifest} ({len(manifest)} rows)")
    print(f"summary:  {args.out_summary}")
    m, u, e, d = bucket(total)
    print(f"match={m} unexplained={u} explained={e} deferred={d}")

    # --- the sv_2023 profile lane (leaf .14b.2) ----------------------------
    if args.results_sv2023.is_file():
        verify_outcome_identity(args.report_sv2023, [args.results_sv2023, args.positions_sv2023],
                                rerun="stimuli/run_external_corpus.sh sv2023")
        raw_positions_sv2023 = load_positions(args.positions_sv2023, "positions_sv2023.tsv")
        positions23 = {}
        rows23 = []
        for line in args.results_sv2023.read_text(encoding="utf-8").splitlines():
            if not line.strip():
                continue
            suite, observed, path = line.split("\t")
            if f"/subs/{suite}/" in path:
                rel = path.split(f"/subs/{suite}/", 1)[1]
            elif UVM_FOLD_MARKER in path:
                rel = path.split(UVM_FOLD_MARKER, 1)[1]
            else:
                raise SystemExit(f"unrecognized sv2023 results path shape: {path!r}")
            rows23.append((suite, rel, observed))
            if path in raw_positions_sv2023:
                positions23[(suite, rel)] = raw_positions_sv2023[path]
        rows23.sort()
        pop17 = {(s, r) for s, r, _o in rows}
        pop23 = {(s, r) for s, r, _o in rows23}
        if pop17 != pop23:
            raise SystemExit(f"the sv2023 lane's population ({len(pop23)}) is not the sv lane's ({len(pop17)}) - "
                             "stale results_sv2023.tsv vs results.tsv; re-run both lanes on one tree")
        manifest23 = []
        for suite, rel, observed in rows23:
            text = read_text(resolve(suite, rel))
            e17, b17 = key_2017(suite, rel, text, vidx, ividx, sidx)
            e23, b23 = key_2023(suite, rel, e17, b17)
            manifest23.append(finish_row(suite, rel, observed, text, e23, b23, positions23))
        manifest23 = edition_keyword_pass("sv_2023", manifest23, resolve, positions23)
        manifest23 = store_gate_pass("sv_2023", manifest23, resolve, args.report_sv2023,
                                     store_gate_cache, args.store_gate_jobs)
        _sg().save_cache(args.store_gate_cache, store_gate_cache)
        with args.out_manifest_sv2023.open("w", encoding="utf-8") as fh:
            fh.write("suite\trelpath\tobserved\texpected\tadjudication\tbasis\n")
            for row in manifest23:
                fh.write("\t".join(row) + "\n")
        per23 = defaultdict(Counter)
        tot23 = Counter()
        for suite, _rel, _obs, _exp, verdict, _basis in manifest23:
            per23[suite][verdict] += 1
            tot23[verdict] += 1
        n_contrast = sum(1 for r in manifest23 if "rules this row an EDITION CONTRAST" in r[5])
        n_override = sum(1 for r in manifest23 if (r[0], r[1]) in SV2023_KEY_OVERRIDES)
        n_owed_prod = sum(1 for r in manifest23 if REREAD_OWED in r[5] and "on an IEEE 1800-2017 production" in r[5])
        n_carried_mech = sum(1 for r in manifest23 if "CARRIES OVER mechanically" in r[5])
        n_carried_hand = sum(1 for r in manifest23 if "CARRIES OVER, re-read by hand" in r[5])
        n_owed_suite = sum(1 for r in manifest23 if REREAD_OWED in r[5] and "suite's own oracle" in r[5])
        lines = []
        lines.append("# SV external-corpus adjudication - the sv_2023 profile lane (leaf SV-CORPUS-GRAD.14b.2)")
        lines.append("")
        lines.append(f"Input: `results_sv2023.tsv` ({len(manifest23)} rows - the SAME population as `results.tsv`, parsed "
                     "under `--profile sv_2023`); expected verdicts = the sv_2017 key read EDITION-RELATIVELY: "
                     f"{n_contrast} rows the 2017 key rules edition contrasts are `must_accept` here (the 1800-2023 "
                     f"production is cited in their basis); {n_override} rows are `must_accept` by an explicit 1800-2023 "
                     "reading (text 1800-2023 derives whose upstream failure is semantic); every other row inherits its "
                     "2017 ruling.")
        lines.append("")
        lines.append("| suite | rows | match | UNEXPLAINED div | explained div | deferred |")
        lines.append("|---|---|---|---|---|---|")
        for suite in sorted(per23):
            c = per23[suite]
            m3, u3, e3, d3 = bucket(c)
            lines.append(f"| {suite} | {sum(c.values())} | {m3} | {u3} | {e3} | {d3} |")
        m3, u3, e3, d3 = bucket(tot23)
        lines.append(f"| **total** | **{len(manifest23)}** | **{m3}** | **{u3}** | **{e3}** | **{d3}** |")
        lines.append("")
        lines.append("## Verdict-class detail")
        lines.append("")
        lines.append("| class | count |")
        lines.append("|---|---|")
        for cls in sorted(tot23):
            lines.append(f"| {cls} | {tot23[cls]} |")
        lines.append("")
        lines.append("**The sv_2023 arm's burn-down baseline = the UNEXPLAINED divergence count "
                     f"({u3}: rejects-valid {tot23.get('divergence:unexplained_rejects_valid', 0)}, "
                     f"accepts-invalid {tot23.get('divergence:unexplained_accepts_invalid', 0)}, "
                     f"crash {tot23.get('divergence:unexplained_crash', 0)})** - a separate arm from the sv_2017 "
                     "baseline over the same files.")
        lines.append("")
        lines.append(f"✅ **Re-read under 1800-2023 (.14b.3):** {n_carried_mech} inherited `must_reject` rulings carry over MECHANICALLY - every "
                     "1800-2017 production their 2017 basis cites reads identically in 1800-2023 Annex A (`production_parity.py`, its artifact "
                     f"`docs/tasks/artifacts/sv_corpus_grad/sv2023_lane/parity.md`) - and {n_carried_hand} carry over BY HAND with the changed 2023 "
                     "production cited in the basis.")
        lines.append("")
        lines.append(f"⚠️ **What this lane has NOT judged for itself, stated rather than hidden:** {n_owed_prod} `must_reject` "
                     "rows are INHERITED from a 1800-2017 production the 2017 key cites, and "
                     f"{n_owed_suite} from a suite's own 2017-era oracle; each carries the marker `{REREAD_OWED}` in its basis. "
                     "Their `match` here means the parser REJECTS them under sv_2023 exactly as under sv_2017 (the .14a census "
                     "names every file whose verdict differs between the profiles, and these are not among them); whether "
                     "1800-2023 derives any of them is `.14b.3`'s per-row question, and a row it turns must_accept becomes "
                     "an `unexplained_rejects_valid` of THIS lane.")
        lines.append("")
        args.out_summary_sv2023.write_text("\n".join(lines) + "\n", encoding="utf-8")
        print(f"sv2023 manifest: {args.out_manifest_sv2023} ({len(manifest23)} rows)")
        print(f"sv2023 summary:  {args.out_summary_sv2023}")
        print(f"sv2023: match={m3} unexplained={u3} explained={e3} deferred={d3} "
              f"edition_contrasts={n_contrast} explicit_2023_readings={n_override} "
              f"inherited_reject_reread_owed={n_owed_prod}+{n_owed_suite} carried_over={n_carried_mech}+{n_carried_hand}")
    else:
        print("sv2023 results absent - run `stimuli/run_external_corpus.sh sv2023` to produce them; "
              "sv2023 manifest skipped")

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

    verify_outcome_identity(args.report_v2005, [args.results_v2005, args.positions_v2005],
                            rerun="stimuli/run_external_corpus.sh sv2005")

    lane_set = set(lane)
    raw_positions_v2005 = load_positions(args.positions_v2005, "positions_v2005.tsv")
    positions = {}
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
        if path in raw_positions_v2005:
            positions[(suite, rel)] = raw_positions_v2005[path]
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
        v_manifest.append(finish_row(suite, rel, observed, text, expected, basis, positions))

    v_manifest = store_gate_pass("verilog_2005", v_manifest, resolve_corpus_path(args), args.report_v2005,
                                 store_gate_cache, args.store_gate_jobs)
    _sg().save_cache(args.store_gate_cache, store_gate_cache)

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
