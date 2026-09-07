# Compiler-directive lines the corpus writes — census at release `1.0.216` (`SV-CORPUS-GRAD.3.39`, 2026-09-06)

Population: the **699** SV-lane files (of 16 336; `.sv` / `.svh` / `.v` — six `.vh` hits are not lane rows and were
dropped) whose text carries a line beginning with one of the thirteen directive names IEEE 1800-2017 clause 22 gives a
syntax box — selected with `grep -lE '^[[:space:]]*`(resetall|undef|undefineall|timescale|default_nettype|unconnected_drive|nounconnected_drive|celldefine|endcelldefine|pragma|line|begin_keywords|end_keywords)\b'`
over `stimuli/sv/subs`, checked by name against the ten rows the leaf was opened on and its triage named (all ten
selected); the `verilog_2005` lane is that list intersected with the lane's own file list (**232**). Instrument:
`../ast_field_census/ast_field_census.py` on the **1.0.216 release probe** (`rust/target/release/parseability_probe`
`b15cf946dbc41057…`, generated-parser identity `94a661afd7dccef1…`), field `body` of every `compiler_directive` node —
on 1.0.216 that body is the directive LINE as one string, so the census reads every backtick line the parser tolerated,
typed names and residue alike; rejected files are scanned by the fallback regex for the thirteen names only.

```
AST-FIELD-CENSUS: kind=compiler_directive field=body profile=sv_2017 files=699 accepted=413 rejected=286 timeout=0 nodump=0 nodes=794 distinct=221 fallback_rows=967
AST-FIELD-CENSUS: kind=compiler_directive field=body profile=sv_2023 files=699 accepted=413 rejected=286 timeout=0 nodump=0 nodes=794 distinct=221 fallback_rows=967
AST-FIELD-CENSUS: kind=compiler_directive field=body profile=verilog_2005 files=232 accepted=204 rejected=28 timeout=0 nodump=0 nodes=310 distinct=64 fallback_rows=43
```

Classified against the boxes by `classify.py` (verbatim; the version_specifier list follows each profile's edition —
IEEE 1800-2017 Syntax 22-10, IEEE 1800-2023 Syntax 22-10 which adds `1800-2023`, IEEE 1364-2005 Syntax 19-10):

```
DIRECTIVE-CLASSIFY: rust/target/s339/directive_census_head_sv_2017.tsv profile=sv_2017 rows=1762 parser_rows=794 regex_rows=967 typed=1615 legal=1552 ILLEGAL=51 TRAILING=10 opaque=146 macro_body=2 no_node=1 unclassified=0
  ILLEGAL:begin_keywords                           12
  ILLEGAL:default_nettype                          4
  ILLEGAL:line                                     21
  ILLEGAL:pragma                                   4
  ILLEGAL:timescale                                6
  ILLEGAL:unconnected_drive                        4
  TRAILING:nounconnected_drive                     1
  TRAILING:pragma                                  2
  TRAILING:timescale                               1
  TRAILING:undef                                   6
  legal:begin_keywords                             91
  legal:celldefine                                 23
  legal:default_nettype                            171
  legal:end_keywords                               97
  legal:endcelldefine                              26
  legal:line                                       19
  legal:nounconnected_drive                        8
  legal:pragma                                     126
  legal:resetall                                   95
  legal:timescale                                  411
  legal:unconnected_drive                          8
  legal:undef                                      467
  legal:undefineall                                10
  macro_body:default_nettype                       2
  no_node                                          1
  opaque:BAR                                       1
  opaque:CONCAT                                    1
  opaque:FOO                                       1
  opaque:STRINGIFY                                 1
  opaque:accelerate                                1
  opaque:autoexpand_vectornets                     1
  opaque:default_decay_time                        3
  opaque:default_trireg_strength                   1
  opaque:define                                    20
  opaque:delay_mode_distributed                    1
  opaque:delay_mode_path                           4
  opaque:delay_mode_unit                           1
  opaque:delay_mode_zero                           1
  opaque:disable_portfaults                        11
  opaque:else                                      2
  opaque:elsif                                     3
  opaque:enable_portfaults                         11
  opaque:endif                                     18
  opaque:error                                     1
  opaque:expand_vectornets                         1
  opaque:ifdef                                     12
  opaque:ifndef                                    5
  opaque:include                                   16
  opaque:noaccelerate                              1
  opaque:noexpand_vectornets                       1
  opaque:noremove_gatenames                        1
  opaque:noremove_netnames                         1
  opaque:nosuppress_faults                         11
  opaque:remove_gatenames                          1
  opaque:remove_netnames                           1
  opaque:suppress_faults                           11
  opaque:verilog                                   1
ROWS (every ILLEGAL / TRAILING / macro_body / no_node / unclassified row, file status in brackets):
  ILLEGAL:begin_keywords:unknown_version       [accepted] stimuli/sv/subs/iverilog/ivtest/ivltests/begin_keywords_1800_2023.v :: `begin_keywords "1800-2023"
  ILLEGAL:begin_keywords:unknown_version       [accepted] stimuli/sv/subs/iverilog/ivtest/ivltests/begin_keywords_vams_2_3.v :: `begin_keywords "VAMS-2.3"
  no_node                                      [accepted] stimuli/sv/subs/Surelog/tests/BeginKeywords/top.v :: <no compiler_directive node>
  ILLEGAL:default_nettype:missing_value        [rejected] stimuli/sv/subs/Surelog/tests/DiffSimpleIncludeAndMacros/top_1.v :: `default_nettype
  TRAILING:undef                               [rejected] stimuli/sv/subs/Surelog/tests/DiffSimpleIncludeAndMacros/top_3.v :: `undef multipl tt
  TRAILING:pragma                              [rejected] stimuli/sv/subs/Surelog/tests/PragmaProtect/top.sv :: `pragma protect key_block encoding = (enctype = "base64")
  TRAILING:pragma                              [rejected] stimuli/sv/subs/Surelog/tests/PragmaProtect/top.sv :: `pragma protect data_block encoding = (enctype = "base64", bytes = 1059)
  ILLEGAL:line:bad_number                      [rejected] stimuli/sv/subs/Surelog/tests/PreprocLine/dut.sv :: `line `new_line_num `foo(line) 1
  ILLEGAL:default_nettype:missing_value        [rejected] stimuli/sv/subs/Surelog/tests/SimpleIncludeAndMacros/top_1.v :: `default_nettype
  TRAILING:undef                               [rejected] stimuli/sv/subs/Surelog/tests/SimpleIncludeAndMacros/top_3.v :: `undef multipl tt
  ILLEGAL:pragma:missing_name                  [accepted] stimuli/sv/subs/sv-tests/tests/chapter-22/22.11--pragma-invalid.sv :: `pragma
  ILLEGAL:line:bad_level                       [accepted] stimuli/sv/subs/sv-tests/tests/chapter-22/22.12--line-illegal-1.sv :: `line 1 "somefile" 3
  ILLEGAL:line:bad_filename                    [accepted] stimuli/sv/subs/sv-tests/tests/chapter-22/22.12--line-illegal-2.sv :: `line 1 somefile 2
  ILLEGAL:line:bad_number                      [accepted] stimuli/sv/subs/sv-tests/tests/chapter-22/22.12--line-illegal-3.sv :: `line -12 "somefile" 3
  ILLEGAL:line:bad_level                       [accepted] stimuli/sv/subs/sv-tests/tests/chapter-22/22.12--line-illegal-4.sv :: `line 1 "somefile"
  ILLEGAL:line:bad_filename                    [accepted] stimuli/sv/subs/sv-tests/tests/chapter-22/22.12--line-illegal-5.sv :: `line 1
  ILLEGAL:timescale:bad_time_unit              [accepted] stimuli/sv/subs/sv-tests/tests/chapter-22/22.7--timescale-basic-3.sv :: `timescale 9 ns / 1 ps
  ILLEGAL:unconnected_drive:missing_value      [accepted] stimuli/sv/subs/sv-tests/tests/chapter-22/22.9--unconnected_drive-invalid-1.sv :: `unconnected_drive
  ILLEGAL:unconnected_drive:bad_value          [accepted] stimuli/sv/subs/sv-tests/tests/chapter-22/22.9--unconnected_drive-invalid-2.sv :: `unconnected_drive pull2
  TRAILING:nounconnected_drive                 [accepted] stimuli/sv/subs/sv-tests/tests/chapter-22/22.9--unconnected_drive-invalid-3.sv :: `nounconnected_drive pull0
  macro_body:default_nettype                   [rejected] stimuli/sv/subs/sv2v/test/core/default_nettype.sv :: `default_nettype t \
  macro_body:default_nettype                   [rejected] stimuli/sv/subs/sv2v/test/core/default_nettype.v :: `default_nettype t \
  ILLEGAL:default_nettype:bad_value            [accepted] stimuli/sv/subs/sv2v/test/error/default_nettype_invalid.sv :: `default_nettype invalid
  ILLEGAL:line:bad_level                       [accepted] stimuli/sv/subs/sv2v/test/error/line_char1.sv :: `line 1 "asd" B
  ILLEGAL:line:bad_level                       [accepted] stimuli/sv/subs/sv2v/test/error/line_char2.sv :: `line 1 "asd" 1B
  ILLEGAL:line:bad_number                      [accepted] stimuli/sv/subs/sv2v/test/error/line_eof.sv :: `line
  ILLEGAL:line:bad_number                      [accepted] stimuli/sv/subs/sv2v/test/error/line_level.sv :: `line 0 "asd" 3
  ILLEGAL:line:bad_number                      [rejected] stimuli/sv/subs/sv2v/test/lex/line.sv :: `line `new_line_num `foo(line) 1
  ILLEGAL:line:bad_number                      [rejected] stimuli/sv/subs/sv2v/test/lex/string_macro.sv :: `line
  ILLEGAL:begin_keywords:unknown_version       [accepted] stimuli/sv/subs/verilator/test_regress/t/t_dpi_vams.v :: `begin_keywords "1800+VAMS"
  ILLEGAL:pragma:missing_name                  [rejected] stimuli/sv/subs/verilator/test_regress/t/t_lint_pragma_protected_bad.v :: `pragma
  ILLEGAL:line:bad_number                      [accepted] stimuli/sv/subs/verilator/test_regress/t/t_lint_warn_line_bad.v :: `line `__LINE__ "the_line_file" 1
  ILLEGAL:line:bad_number                      [accepted] stimuli/sv/subs/verilator/test_regress/t/t_lint_warn_line_bad.v :: `line `__LINE__ "the_line_file" 2
  ILLEGAL:default_nettype:bad_value            [accepted] stimuli/sv/subs/verilator/test_regress/t/t_pp_defnettype_bad.v :: `default_nettype bad_none_such
  ILLEGAL:line:bad_number                      [accepted] stimuli/sv/subs/verilator/test_regress/t/t_pp_line_bad.v :: `line
  ILLEGAL:line:bad_filename                    [accepted] stimuli/sv/subs/verilator/test_regress/t/t_pp_line_bad.v :: `line 100
  ILLEGAL:line:bad_filename                    [accepted] stimuli/sv/subs/verilator/test_regress/t/t_pp_line_bad.v :: `line 200 somefile
  ILLEGAL:line:bad_filename                    [accepted] stimuli/sv/subs/verilator/test_regress/t/t_pp_line_bad.v :: `line 300 "somefile 1
  ILLEGAL:line:bad_level                       [accepted] stimuli/sv/subs/verilator/test_regress/t/t_pp_line_bad.v :: `line 400 "some file"
  ILLEGAL:line:bad_level                       [accepted] stimuli/sv/subs/verilator/test_regress/t/t_pp_line_bad.v :: `line 500 "somefile" 3
  ILLEGAL:line:bad_level                       [accepted] stimuli/sv/subs/verilator/test_regress/t/t_pp_line_bad.v :: `line 600 "some file" 3
  ILLEGAL:pragma:missing_name                  [accepted] stimuli/sv/subs/verilator/test_regress/t/t_pp_pragma_bad.v :: `pragma
  ILLEGAL:begin_keywords:unknown_version       [accepted] stimuli/sv/subs/verilator/test_regress/t/t_preproc_ifexpr_bad.v :: `begin_keywords "1800-2023"
  ILLEGAL:begin_keywords:unknown_version       [rejected] stimuli/sv/subs/verilator/test_regress/t/t_preproc_ifexpr.v :: `begin_keywords "1800-2023"
  ILLEGAL:begin_keywords:unknown_version       [rejected] stimuli/sv/subs/verilator/test_regress/t/t_preproc_kwd.v :: `begin_keywords "1800-2023"
  ILLEGAL:begin_keywords:unknown_version       [rejected] stimuli/sv/subs/verilator/test_regress/t/t_preproc_kwd.v :: `begin_keywords "VAMS-2.3"
  ILLEGAL:begin_keywords:unknown_version       [accepted] stimuli/sv/subs/verilator/test_regress/t/t_preproc_preproczero_bad.v :: `begin_keywords "1800-2023"
  ILLEGAL:pragma:bad_expression                [rejected] stimuli/sv/subs/verilator/test_regress/t/t_preproc.v :: `pragma foo = 1
  TRAILING:undef                               [rejected] stimuli/sv/subs/verilator/test_regress/t/t_preproc.v :: `undef CAT `undef ESC
  TRAILING:undef                               [rejected] stimuli/sv/subs/verilator/test_regress/t/t_preproc.v :: `undef CAT `undef ESC
  TRAILING:undef                               [rejected] stimuli/sv/subs/verilator/test_regress/t/t_preproc.v :: `undef FOO `undef ESC
  TRAILING:undef                               [rejected] stimuli/sv/subs/verilator/test_regress/t/t_preproc.v :: `undef A `undef STR
  ILLEGAL:begin_keywords:unknown_version       [rejected] stimuli/sv/subs/verilator/test_regress/t/t_split_var_3_wreal.v :: `begin_keywords "VAMS-2.3"
  ILLEGAL:timescale:bad_time_unit              [rejected] stimuli/sv/subs/verilator/test_regress/t/t_time_vpi.v :: `timescale `time_scale_units / `time_scale_prec
  ILLEGAL:timescale:bad_time_unit              [accepted] stimuli/sv/subs/verilator/test_regress/t/t_timescale_parse_bad.v :: `timescale frump
  ILLEGAL:timescale:bad_time_unit              [accepted] stimuli/sv/subs/verilator/test_regress/t/t_timescale_parse_bad.v :: `timescale 1xs
  ILLEGAL:timescale:bad_time_unit              [accepted] stimuli/sv/subs/verilator/test_regress/t/t_timescale_parse_bad.v :: `timescale 2ps
  ILLEGAL:timescale:bad_time_precision         [accepted] stimuli/sv/subs/verilator/test_regress/t/t_timescale_parse_bad.v :: `timescale 1ns / frump
  TRAILING:timescale                           [accepted] stimuli/sv/subs/verilator/test_regress/t/t_timescale_parse_bad.v :: `timescale 1ns / 1ps /extra
  ILLEGAL:unconnected_drive:missing_value      [accepted] stimuli/sv/subs/verilator/test_regress/t/t_unconnected_bad.v :: `unconnected_drive
  ILLEGAL:unconnected_drive:bad_value          [accepted] stimuli/sv/subs/verilator/test_regress/t/t_unconnected_bad.v :: `unconnected_drive pull2
  ILLEGAL:begin_keywords:unknown_version       [rejected] stimuli/sv/subs/verilator/test_regress/t/t_vams_basic.v :: `begin_keywords "VAMS-2.3"
  ILLEGAL:begin_keywords:unknown_version       [rejected] stimuli/sv/subs/verilator/test_regress/t/t_vams_kwd_bad.v :: `begin_keywords "VAMS-2.3"
  ILLEGAL:begin_keywords:unknown_version       [rejected] stimuli/sv/subs/verilator/test_regress/t/t_vams_wreal.v :: `begin_keywords "VAMS-2.3"
PREDICTION accepted files that must move pass -> fail: 26
  MOVE  stimuli/sv/subs/iverilog/ivtest/ivltests/begin_keywords_1800_2023.v :: ILLEGAL:begin_keywords:unknown_version :: `begin_keywords "1800-2023"
  MOVE  stimuli/sv/subs/iverilog/ivtest/ivltests/begin_keywords_vams_2_3.v :: ILLEGAL:begin_keywords:unknown_version :: `begin_keywords "VAMS-2.3"
  MOVE  stimuli/sv/subs/sv-tests/tests/chapter-22/22.11--pragma-invalid.sv :: ILLEGAL:pragma:missing_name :: `pragma
  MOVE  stimuli/sv/subs/sv-tests/tests/chapter-22/22.12--line-illegal-1.sv :: ILLEGAL:line:bad_level :: `line 1 "somefile" 3
  MOVE  stimuli/sv/subs/sv-tests/tests/chapter-22/22.12--line-illegal-2.sv :: ILLEGAL:line:bad_filename :: `line 1 somefile 2
  MOVE  stimuli/sv/subs/sv-tests/tests/chapter-22/22.12--line-illegal-3.sv :: ILLEGAL:line:bad_number :: `line -12 "somefile" 3
  MOVE  stimuli/sv/subs/sv-tests/tests/chapter-22/22.12--line-illegal-4.sv :: ILLEGAL:line:bad_level :: `line 1 "somefile"
  MOVE  stimuli/sv/subs/sv-tests/tests/chapter-22/22.12--line-illegal-5.sv :: ILLEGAL:line:bad_filename :: `line 1
  MOVE  stimuli/sv/subs/sv-tests/tests/chapter-22/22.7--timescale-basic-3.sv :: ILLEGAL:timescale:bad_time_unit :: `timescale 9 ns / 1 ps
  MOVE  stimuli/sv/subs/sv-tests/tests/chapter-22/22.9--unconnected_drive-invalid-1.sv :: ILLEGAL:unconnected_drive:missing_value :: `unconnected_drive
  MOVE  stimuli/sv/subs/sv-tests/tests/chapter-22/22.9--unconnected_drive-invalid-2.sv :: ILLEGAL:unconnected_drive:bad_value :: `unconnected_drive pull2
  MOVE  stimuli/sv/subs/sv-tests/tests/chapter-22/22.9--unconnected_drive-invalid-3.sv :: TRAILING:nounconnected_drive :: `nounconnected_drive pull0
  MOVE  stimuli/sv/subs/sv2v/test/error/default_nettype_invalid.sv :: ILLEGAL:default_nettype:bad_value :: `default_nettype invalid
  MOVE  stimuli/sv/subs/sv2v/test/error/line_char1.sv :: ILLEGAL:line:bad_level :: `line 1 "asd" B
  MOVE  stimuli/sv/subs/sv2v/test/error/line_char2.sv :: ILLEGAL:line:bad_level :: `line 1 "asd" 1B
  MOVE  stimuli/sv/subs/sv2v/test/error/line_eof.sv :: ILLEGAL:line:bad_number :: `line
  MOVE  stimuli/sv/subs/sv2v/test/error/line_level.sv :: ILLEGAL:line:bad_number :: `line 0 "asd" 3
  MOVE  stimuli/sv/subs/verilator/test_regress/t/t_dpi_vams.v :: ILLEGAL:begin_keywords:unknown_version :: `begin_keywords "1800+VAMS"
  MOVE  stimuli/sv/subs/verilator/test_regress/t/t_lint_warn_line_bad.v :: ILLEGAL:line:bad_number :: `line `__LINE__ "the_line_file" 1
  MOVE  stimuli/sv/subs/verilator/test_regress/t/t_lint_warn_line_bad.v :: ILLEGAL:line:bad_number :: `line `__LINE__ "the_line_file" 2
  MOVE  stimuli/sv/subs/verilator/test_regress/t/t_pp_defnettype_bad.v :: ILLEGAL:default_nettype:bad_value :: `default_nettype bad_none_such
  MOVE  stimuli/sv/subs/verilator/test_regress/t/t_pp_line_bad.v :: ILLEGAL:line:bad_number :: `line
  MOVE  stimuli/sv/subs/verilator/test_regress/t/t_pp_line_bad.v :: ILLEGAL:line:bad_filename :: `line 100
  MOVE  stimuli/sv/subs/verilator/test_regress/t/t_pp_line_bad.v :: ILLEGAL:line:bad_filename :: `line 200 somefile
  MOVE  stimuli/sv/subs/verilator/test_regress/t/t_pp_line_bad.v :: ILLEGAL:line:bad_filename :: `line 300 "somefile 1
  MOVE  stimuli/sv/subs/verilator/test_regress/t/t_pp_line_bad.v :: ILLEGAL:line:bad_level :: `line 400 "some file"
  MOVE  stimuli/sv/subs/verilator/test_regress/t/t_pp_line_bad.v :: ILLEGAL:line:bad_level :: `line 500 "somefile" 3
  MOVE  stimuli/sv/subs/verilator/test_regress/t/t_pp_line_bad.v :: ILLEGAL:line:bad_level :: `line 600 "some file" 3
  MOVE  stimuli/sv/subs/verilator/test_regress/t/t_pp_pragma_bad.v :: ILLEGAL:pragma:missing_name :: `pragma
  MOVE  stimuli/sv/subs/verilator/test_regress/t/t_preproc_ifexpr_bad.v :: ILLEGAL:begin_keywords:unknown_version :: `begin_keywords "1800-2023"
  MOVE  stimuli/sv/subs/verilator/test_regress/t/t_preproc_preproczero_bad.v :: ILLEGAL:begin_keywords:unknown_version :: `begin_keywords "1800-2023"
  MOVE  stimuli/sv/subs/verilator/test_regress/t/t_timescale_parse_bad.v :: ILLEGAL:timescale:bad_time_unit :: `timescale frump
  MOVE  stimuli/sv/subs/verilator/test_regress/t/t_timescale_parse_bad.v :: ILLEGAL:timescale:bad_time_unit :: `timescale 1xs
  MOVE  stimuli/sv/subs/verilator/test_regress/t/t_timescale_parse_bad.v :: ILLEGAL:timescale:bad_time_unit :: `timescale 2ps
  MOVE  stimuli/sv/subs/verilator/test_regress/t/t_timescale_parse_bad.v :: ILLEGAL:timescale:bad_time_precision :: `timescale 1ns / frump
  MOVE  stimuli/sv/subs/verilator/test_regress/t/t_timescale_parse_bad.v :: TRAILING:timescale :: `timescale 1ns / 1ps /extra
  MOVE  stimuli/sv/subs/verilator/test_regress/t/t_unconnected_bad.v :: ILLEGAL:unconnected_drive:missing_value :: `unconnected_drive
  MOVE  stimuli/sv/subs/verilator/test_regress/t/t_unconnected_bad.v :: ILLEGAL:unconnected_drive:bad_value :: `unconnected_drive pull2
PREDICTION rejected files whose failure position may move: 19
  STAYS stimuli/sv/subs/Surelog/tests/DiffSimpleIncludeAndMacros/top_1.v :: ILLEGAL:default_nettype:missing_value :: `default_nettype
  STAYS stimuli/sv/subs/Surelog/tests/DiffSimpleIncludeAndMacros/top_3.v :: TRAILING:undef :: `undef multipl tt
  STAYS stimuli/sv/subs/Surelog/tests/PragmaProtect/top.sv :: TRAILING:pragma :: `pragma protect key_block encoding = (enctype = "base64")
  STAYS stimuli/sv/subs/Surelog/tests/PragmaProtect/top.sv :: TRAILING:pragma :: `pragma protect data_block encoding = (enctype = "base64", bytes = 1059)
  STAYS stimuli/sv/subs/Surelog/tests/PreprocLine/dut.sv :: ILLEGAL:line:bad_number :: `line `new_line_num `foo(line) 1
  STAYS stimuli/sv/subs/Surelog/tests/SimpleIncludeAndMacros/top_1.v :: ILLEGAL:default_nettype:missing_value :: `default_nettype
  STAYS stimuli/sv/subs/Surelog/tests/SimpleIncludeAndMacros/top_3.v :: TRAILING:undef :: `undef multipl tt
  STAYS stimuli/sv/subs/sv2v/test/core/default_nettype.sv :: macro_body:default_nettype :: `default_nettype t \
  STAYS stimuli/sv/subs/sv2v/test/core/default_nettype.v :: macro_body:default_nettype :: `default_nettype t \
  STAYS stimuli/sv/subs/sv2v/test/lex/line.sv :: ILLEGAL:line:bad_number :: `line `new_line_num `foo(line) 1
  STAYS stimuli/sv/subs/sv2v/test/lex/string_macro.sv :: ILLEGAL:line:bad_number :: `line
  STAYS stimuli/sv/subs/verilator/test_regress/t/t_lint_pragma_protected_bad.v :: ILLEGAL:pragma:missing_name :: `pragma
  STAYS stimuli/sv/subs/verilator/test_regress/t/t_preproc.v :: ILLEGAL:pragma:bad_expression :: `pragma foo = 1
  STAYS stimuli/sv/subs/verilator/test_regress/t/t_preproc.v :: TRAILING:undef :: `undef CAT `undef ESC
  STAYS stimuli/sv/subs/verilator/test_regress/t/t_preproc.v :: TRAILING:undef :: `undef CAT `undef ESC
  STAYS stimuli/sv/subs/verilator/test_regress/t/t_preproc.v :: TRAILING:undef :: `undef FOO `undef ESC
  STAYS stimuli/sv/subs/verilator/test_regress/t/t_preproc.v :: TRAILING:undef :: `undef A `undef STR
  STAYS stimuli/sv/subs/verilator/test_regress/t/t_preproc_ifexpr.v :: ILLEGAL:begin_keywords:unknown_version :: `begin_keywords "1800-2023"
  STAYS stimuli/sv/subs/verilator/test_regress/t/t_preproc_kwd.v :: ILLEGAL:begin_keywords:unknown_version :: `begin_keywords "1800-2023"
  STAYS stimuli/sv/subs/verilator/test_regress/t/t_preproc_kwd.v :: ILLEGAL:begin_keywords:unknown_version :: `begin_keywords "VAMS-2.3"
  STAYS stimuli/sv/subs/verilator/test_regress/t/t_split_var_3_wreal.v :: ILLEGAL:begin_keywords:unknown_version :: `begin_keywords "VAMS-2.3"
  STAYS stimuli/sv/subs/verilator/test_regress/t/t_time_vpi.v :: ILLEGAL:timescale:bad_time_unit :: `timescale `time_scale_units / `time_scale_prec
  STAYS stimuli/sv/subs/verilator/test_regress/t/t_vams_basic.v :: ILLEGAL:begin_keywords:unknown_version :: `begin_keywords "VAMS-2.3"
  STAYS stimuli/sv/subs/verilator/test_regress/t/t_vams_kwd_bad.v :: ILLEGAL:begin_keywords:unknown_version :: `begin_keywords "VAMS-2.3"
  STAYS stimuli/sv/subs/verilator/test_regress/t/t_vams_wreal.v :: ILLEGAL:begin_keywords:unknown_version :: `begin_keywords "VAMS-2.3"
```

```
DIRECTIVE-CLASSIFY: rust/target/s339/directive_census_head_sv_2023.tsv profile=sv_2023 rows=1762 parser_rows=794 regex_rows=967 typed=1615 legal=1557 ILLEGAL=46 TRAILING=10 opaque=146 macro_body=2 no_node=1 unclassified=0
  ILLEGAL:begin_keywords                           7
  ILLEGAL:default_nettype                          4
  ILLEGAL:line                                     21
  ILLEGAL:pragma                                   4
  ILLEGAL:timescale                                6
  ILLEGAL:unconnected_drive                        4
  TRAILING:nounconnected_drive                     1
  TRAILING:pragma                                  2
  TRAILING:timescale                               1
  TRAILING:undef                                   6
  legal:begin_keywords                             96
  legal:celldefine                                 23
  legal:default_nettype                            171
  legal:end_keywords                               97
  legal:endcelldefine                              26
  legal:line                                       19
  legal:nounconnected_drive                        8
  legal:pragma                                     126
  legal:resetall                                   95
  legal:timescale                                  411
  legal:unconnected_drive                          8
  legal:undef                                      467
  legal:undefineall                                10
  macro_body:default_nettype                       2
  no_node                                          1
  opaque:BAR                                       1
  opaque:CONCAT                                    1
  opaque:FOO                                       1
  opaque:STRINGIFY                                 1
  opaque:accelerate                                1
  opaque:autoexpand_vectornets                     1
  opaque:default_decay_time                        3
  opaque:default_trireg_strength                   1
  opaque:define                                    20
  opaque:delay_mode_distributed                    1
  opaque:delay_mode_path                           4
  opaque:delay_mode_unit                           1
  opaque:delay_mode_zero                           1
  opaque:disable_portfaults                        11
  opaque:else                                      2
  opaque:elsif                                     3
  opaque:enable_portfaults                         11
  opaque:endif                                     18
  opaque:error                                     1
  opaque:expand_vectornets                         1
  opaque:ifdef                                     12
  opaque:ifndef                                    5
  opaque:include                                   16
  opaque:noaccelerate                              1
  opaque:noexpand_vectornets                       1
  opaque:noremove_gatenames                        1
  opaque:noremove_netnames                         1
  opaque:nosuppress_faults                         11
  opaque:remove_gatenames                          1
  opaque:remove_netnames                           1
  opaque:suppress_faults                           11
  opaque:verilog                                   1
ROWS (every ILLEGAL / TRAILING / macro_body / no_node / unclassified row, file status in brackets):
  ILLEGAL:begin_keywords:unknown_version       [accepted] stimuli/sv/subs/iverilog/ivtest/ivltests/begin_keywords_vams_2_3.v :: `begin_keywords "VAMS-2.3"
  no_node                                      [accepted] stimuli/sv/subs/Surelog/tests/BeginKeywords/top.v :: <no compiler_directive node>
  ILLEGAL:default_nettype:missing_value        [rejected] stimuli/sv/subs/Surelog/tests/DiffSimpleIncludeAndMacros/top_1.v :: `default_nettype
  TRAILING:undef                               [rejected] stimuli/sv/subs/Surelog/tests/DiffSimpleIncludeAndMacros/top_3.v :: `undef multipl tt
  TRAILING:pragma                              [rejected] stimuli/sv/subs/Surelog/tests/PragmaProtect/top.sv :: `pragma protect key_block encoding = (enctype = "base64")
  TRAILING:pragma                              [rejected] stimuli/sv/subs/Surelog/tests/PragmaProtect/top.sv :: `pragma protect data_block encoding = (enctype = "base64", bytes = 1059)
  ILLEGAL:line:bad_number                      [rejected] stimuli/sv/subs/Surelog/tests/PreprocLine/dut.sv :: `line `new_line_num `foo(line) 1
  ILLEGAL:default_nettype:missing_value        [rejected] stimuli/sv/subs/Surelog/tests/SimpleIncludeAndMacros/top_1.v :: `default_nettype
  TRAILING:undef                               [rejected] stimuli/sv/subs/Surelog/tests/SimpleIncludeAndMacros/top_3.v :: `undef multipl tt
  ILLEGAL:pragma:missing_name                  [accepted] stimuli/sv/subs/sv-tests/tests/chapter-22/22.11--pragma-invalid.sv :: `pragma
  ILLEGAL:line:bad_level                       [accepted] stimuli/sv/subs/sv-tests/tests/chapter-22/22.12--line-illegal-1.sv :: `line 1 "somefile" 3
  ILLEGAL:line:bad_filename                    [accepted] stimuli/sv/subs/sv-tests/tests/chapter-22/22.12--line-illegal-2.sv :: `line 1 somefile 2
  ILLEGAL:line:bad_number                      [accepted] stimuli/sv/subs/sv-tests/tests/chapter-22/22.12--line-illegal-3.sv :: `line -12 "somefile" 3
  ILLEGAL:line:bad_level                       [accepted] stimuli/sv/subs/sv-tests/tests/chapter-22/22.12--line-illegal-4.sv :: `line 1 "somefile"
  ILLEGAL:line:bad_filename                    [accepted] stimuli/sv/subs/sv-tests/tests/chapter-22/22.12--line-illegal-5.sv :: `line 1
  ILLEGAL:timescale:bad_time_unit              [accepted] stimuli/sv/subs/sv-tests/tests/chapter-22/22.7--timescale-basic-3.sv :: `timescale 9 ns / 1 ps
  ILLEGAL:unconnected_drive:missing_value      [accepted] stimuli/sv/subs/sv-tests/tests/chapter-22/22.9--unconnected_drive-invalid-1.sv :: `unconnected_drive
  ILLEGAL:unconnected_drive:bad_value          [accepted] stimuli/sv/subs/sv-tests/tests/chapter-22/22.9--unconnected_drive-invalid-2.sv :: `unconnected_drive pull2
  TRAILING:nounconnected_drive                 [accepted] stimuli/sv/subs/sv-tests/tests/chapter-22/22.9--unconnected_drive-invalid-3.sv :: `nounconnected_drive pull0
  macro_body:default_nettype                   [rejected] stimuli/sv/subs/sv2v/test/core/default_nettype.sv :: `default_nettype t \
  macro_body:default_nettype                   [rejected] stimuli/sv/subs/sv2v/test/core/default_nettype.v :: `default_nettype t \
  ILLEGAL:default_nettype:bad_value            [accepted] stimuli/sv/subs/sv2v/test/error/default_nettype_invalid.sv :: `default_nettype invalid
  ILLEGAL:line:bad_level                       [accepted] stimuli/sv/subs/sv2v/test/error/line_char1.sv :: `line 1 "asd" B
  ILLEGAL:line:bad_level                       [accepted] stimuli/sv/subs/sv2v/test/error/line_char2.sv :: `line 1 "asd" 1B
  ILLEGAL:line:bad_number                      [accepted] stimuli/sv/subs/sv2v/test/error/line_eof.sv :: `line
  ILLEGAL:line:bad_number                      [accepted] stimuli/sv/subs/sv2v/test/error/line_level.sv :: `line 0 "asd" 3
  ILLEGAL:line:bad_number                      [rejected] stimuli/sv/subs/sv2v/test/lex/line.sv :: `line `new_line_num `foo(line) 1
  ILLEGAL:line:bad_number                      [rejected] stimuli/sv/subs/sv2v/test/lex/string_macro.sv :: `line
  ILLEGAL:begin_keywords:unknown_version       [accepted] stimuli/sv/subs/verilator/test_regress/t/t_dpi_vams.v :: `begin_keywords "1800+VAMS"
  ILLEGAL:pragma:missing_name                  [rejected] stimuli/sv/subs/verilator/test_regress/t/t_lint_pragma_protected_bad.v :: `pragma
  ILLEGAL:line:bad_number                      [accepted] stimuli/sv/subs/verilator/test_regress/t/t_lint_warn_line_bad.v :: `line `__LINE__ "the_line_file" 1
  ILLEGAL:line:bad_number                      [accepted] stimuli/sv/subs/verilator/test_regress/t/t_lint_warn_line_bad.v :: `line `__LINE__ "the_line_file" 2
  ILLEGAL:default_nettype:bad_value            [accepted] stimuli/sv/subs/verilator/test_regress/t/t_pp_defnettype_bad.v :: `default_nettype bad_none_such
  ILLEGAL:line:bad_number                      [accepted] stimuli/sv/subs/verilator/test_regress/t/t_pp_line_bad.v :: `line
  ILLEGAL:line:bad_filename                    [accepted] stimuli/sv/subs/verilator/test_regress/t/t_pp_line_bad.v :: `line 100
  ILLEGAL:line:bad_filename                    [accepted] stimuli/sv/subs/verilator/test_regress/t/t_pp_line_bad.v :: `line 200 somefile
  ILLEGAL:line:bad_filename                    [accepted] stimuli/sv/subs/verilator/test_regress/t/t_pp_line_bad.v :: `line 300 "somefile 1
  ILLEGAL:line:bad_level                       [accepted] stimuli/sv/subs/verilator/test_regress/t/t_pp_line_bad.v :: `line 400 "some file"
  ILLEGAL:line:bad_level                       [accepted] stimuli/sv/subs/verilator/test_regress/t/t_pp_line_bad.v :: `line 500 "somefile" 3
  ILLEGAL:line:bad_level                       [accepted] stimuli/sv/subs/verilator/test_regress/t/t_pp_line_bad.v :: `line 600 "some file" 3
  ILLEGAL:pragma:missing_name                  [accepted] stimuli/sv/subs/verilator/test_regress/t/t_pp_pragma_bad.v :: `pragma
  ILLEGAL:begin_keywords:unknown_version       [rejected] stimuli/sv/subs/verilator/test_regress/t/t_preproc_kwd.v :: `begin_keywords "VAMS-2.3"
  ILLEGAL:pragma:bad_expression                [rejected] stimuli/sv/subs/verilator/test_regress/t/t_preproc.v :: `pragma foo = 1
  TRAILING:undef                               [rejected] stimuli/sv/subs/verilator/test_regress/t/t_preproc.v :: `undef CAT `undef ESC
  TRAILING:undef                               [rejected] stimuli/sv/subs/verilator/test_regress/t/t_preproc.v :: `undef CAT `undef ESC
  TRAILING:undef                               [rejected] stimuli/sv/subs/verilator/test_regress/t/t_preproc.v :: `undef FOO `undef ESC
  TRAILING:undef                               [rejected] stimuli/sv/subs/verilator/test_regress/t/t_preproc.v :: `undef A `undef STR
  ILLEGAL:begin_keywords:unknown_version       [rejected] stimuli/sv/subs/verilator/test_regress/t/t_split_var_3_wreal.v :: `begin_keywords "VAMS-2.3"
  ILLEGAL:timescale:bad_time_unit              [rejected] stimuli/sv/subs/verilator/test_regress/t/t_time_vpi.v :: `timescale `time_scale_units / `time_scale_prec
  ILLEGAL:timescale:bad_time_unit              [accepted] stimuli/sv/subs/verilator/test_regress/t/t_timescale_parse_bad.v :: `timescale frump
  ILLEGAL:timescale:bad_time_unit              [accepted] stimuli/sv/subs/verilator/test_regress/t/t_timescale_parse_bad.v :: `timescale 1xs
  ILLEGAL:timescale:bad_time_unit              [accepted] stimuli/sv/subs/verilator/test_regress/t/t_timescale_parse_bad.v :: `timescale 2ps
  ILLEGAL:timescale:bad_time_precision         [accepted] stimuli/sv/subs/verilator/test_regress/t/t_timescale_parse_bad.v :: `timescale 1ns / frump
  TRAILING:timescale                           [accepted] stimuli/sv/subs/verilator/test_regress/t/t_timescale_parse_bad.v :: `timescale 1ns / 1ps /extra
  ILLEGAL:unconnected_drive:missing_value      [accepted] stimuli/sv/subs/verilator/test_regress/t/t_unconnected_bad.v :: `unconnected_drive
  ILLEGAL:unconnected_drive:bad_value          [accepted] stimuli/sv/subs/verilator/test_regress/t/t_unconnected_bad.v :: `unconnected_drive pull2
  ILLEGAL:begin_keywords:unknown_version       [rejected] stimuli/sv/subs/verilator/test_regress/t/t_vams_basic.v :: `begin_keywords "VAMS-2.3"
  ILLEGAL:begin_keywords:unknown_version       [rejected] stimuli/sv/subs/verilator/test_regress/t/t_vams_kwd_bad.v :: `begin_keywords "VAMS-2.3"
  ILLEGAL:begin_keywords:unknown_version       [rejected] stimuli/sv/subs/verilator/test_regress/t/t_vams_wreal.v :: `begin_keywords "VAMS-2.3"
PREDICTION accepted files that must move pass -> fail: 23
  MOVE  stimuli/sv/subs/iverilog/ivtest/ivltests/begin_keywords_vams_2_3.v :: ILLEGAL:begin_keywords:unknown_version :: `begin_keywords "VAMS-2.3"
  MOVE  stimuli/sv/subs/sv-tests/tests/chapter-22/22.11--pragma-invalid.sv :: ILLEGAL:pragma:missing_name :: `pragma
  MOVE  stimuli/sv/subs/sv-tests/tests/chapter-22/22.12--line-illegal-1.sv :: ILLEGAL:line:bad_level :: `line 1 "somefile" 3
  MOVE  stimuli/sv/subs/sv-tests/tests/chapter-22/22.12--line-illegal-2.sv :: ILLEGAL:line:bad_filename :: `line 1 somefile 2
  MOVE  stimuli/sv/subs/sv-tests/tests/chapter-22/22.12--line-illegal-3.sv :: ILLEGAL:line:bad_number :: `line -12 "somefile" 3
  MOVE  stimuli/sv/subs/sv-tests/tests/chapter-22/22.12--line-illegal-4.sv :: ILLEGAL:line:bad_level :: `line 1 "somefile"
  MOVE  stimuli/sv/subs/sv-tests/tests/chapter-22/22.12--line-illegal-5.sv :: ILLEGAL:line:bad_filename :: `line 1
  MOVE  stimuli/sv/subs/sv-tests/tests/chapter-22/22.7--timescale-basic-3.sv :: ILLEGAL:timescale:bad_time_unit :: `timescale 9 ns / 1 ps
  MOVE  stimuli/sv/subs/sv-tests/tests/chapter-22/22.9--unconnected_drive-invalid-1.sv :: ILLEGAL:unconnected_drive:missing_value :: `unconnected_drive
  MOVE  stimuli/sv/subs/sv-tests/tests/chapter-22/22.9--unconnected_drive-invalid-2.sv :: ILLEGAL:unconnected_drive:bad_value :: `unconnected_drive pull2
  MOVE  stimuli/sv/subs/sv-tests/tests/chapter-22/22.9--unconnected_drive-invalid-3.sv :: TRAILING:nounconnected_drive :: `nounconnected_drive pull0
  MOVE  stimuli/sv/subs/sv2v/test/error/default_nettype_invalid.sv :: ILLEGAL:default_nettype:bad_value :: `default_nettype invalid
  MOVE  stimuli/sv/subs/sv2v/test/error/line_char1.sv :: ILLEGAL:line:bad_level :: `line 1 "asd" B
  MOVE  stimuli/sv/subs/sv2v/test/error/line_char2.sv :: ILLEGAL:line:bad_level :: `line 1 "asd" 1B
  MOVE  stimuli/sv/subs/sv2v/test/error/line_eof.sv :: ILLEGAL:line:bad_number :: `line
  MOVE  stimuli/sv/subs/sv2v/test/error/line_level.sv :: ILLEGAL:line:bad_number :: `line 0 "asd" 3
  MOVE  stimuli/sv/subs/verilator/test_regress/t/t_dpi_vams.v :: ILLEGAL:begin_keywords:unknown_version :: `begin_keywords "1800+VAMS"
  MOVE  stimuli/sv/subs/verilator/test_regress/t/t_lint_warn_line_bad.v :: ILLEGAL:line:bad_number :: `line `__LINE__ "the_line_file" 1
  MOVE  stimuli/sv/subs/verilator/test_regress/t/t_lint_warn_line_bad.v :: ILLEGAL:line:bad_number :: `line `__LINE__ "the_line_file" 2
  MOVE  stimuli/sv/subs/verilator/test_regress/t/t_pp_defnettype_bad.v :: ILLEGAL:default_nettype:bad_value :: `default_nettype bad_none_such
  MOVE  stimuli/sv/subs/verilator/test_regress/t/t_pp_line_bad.v :: ILLEGAL:line:bad_number :: `line
  MOVE  stimuli/sv/subs/verilator/test_regress/t/t_pp_line_bad.v :: ILLEGAL:line:bad_filename :: `line 100
  MOVE  stimuli/sv/subs/verilator/test_regress/t/t_pp_line_bad.v :: ILLEGAL:line:bad_filename :: `line 200 somefile
  MOVE  stimuli/sv/subs/verilator/test_regress/t/t_pp_line_bad.v :: ILLEGAL:line:bad_filename :: `line 300 "somefile 1
  MOVE  stimuli/sv/subs/verilator/test_regress/t/t_pp_line_bad.v :: ILLEGAL:line:bad_level :: `line 400 "some file"
  MOVE  stimuli/sv/subs/verilator/test_regress/t/t_pp_line_bad.v :: ILLEGAL:line:bad_level :: `line 500 "somefile" 3
  MOVE  stimuli/sv/subs/verilator/test_regress/t/t_pp_line_bad.v :: ILLEGAL:line:bad_level :: `line 600 "some file" 3
  MOVE  stimuli/sv/subs/verilator/test_regress/t/t_pp_pragma_bad.v :: ILLEGAL:pragma:missing_name :: `pragma
  MOVE  stimuli/sv/subs/verilator/test_regress/t/t_timescale_parse_bad.v :: ILLEGAL:timescale:bad_time_unit :: `timescale frump
  MOVE  stimuli/sv/subs/verilator/test_regress/t/t_timescale_parse_bad.v :: ILLEGAL:timescale:bad_time_unit :: `timescale 1xs
  MOVE  stimuli/sv/subs/verilator/test_regress/t/t_timescale_parse_bad.v :: ILLEGAL:timescale:bad_time_unit :: `timescale 2ps
  MOVE  stimuli/sv/subs/verilator/test_regress/t/t_timescale_parse_bad.v :: ILLEGAL:timescale:bad_time_precision :: `timescale 1ns / frump
  MOVE  stimuli/sv/subs/verilator/test_regress/t/t_timescale_parse_bad.v :: TRAILING:timescale :: `timescale 1ns / 1ps /extra
  MOVE  stimuli/sv/subs/verilator/test_regress/t/t_unconnected_bad.v :: ILLEGAL:unconnected_drive:missing_value :: `unconnected_drive
  MOVE  stimuli/sv/subs/verilator/test_regress/t/t_unconnected_bad.v :: ILLEGAL:unconnected_drive:bad_value :: `unconnected_drive pull2
PREDICTION rejected files whose failure position may move: 18
  STAYS stimuli/sv/subs/Surelog/tests/DiffSimpleIncludeAndMacros/top_1.v :: ILLEGAL:default_nettype:missing_value :: `default_nettype
  STAYS stimuli/sv/subs/Surelog/tests/DiffSimpleIncludeAndMacros/top_3.v :: TRAILING:undef :: `undef multipl tt
  STAYS stimuli/sv/subs/Surelog/tests/PragmaProtect/top.sv :: TRAILING:pragma :: `pragma protect key_block encoding = (enctype = "base64")
  STAYS stimuli/sv/subs/Surelog/tests/PragmaProtect/top.sv :: TRAILING:pragma :: `pragma protect data_block encoding = (enctype = "base64", bytes = 1059)
  STAYS stimuli/sv/subs/Surelog/tests/PreprocLine/dut.sv :: ILLEGAL:line:bad_number :: `line `new_line_num `foo(line) 1
  STAYS stimuli/sv/subs/Surelog/tests/SimpleIncludeAndMacros/top_1.v :: ILLEGAL:default_nettype:missing_value :: `default_nettype
  STAYS stimuli/sv/subs/Surelog/tests/SimpleIncludeAndMacros/top_3.v :: TRAILING:undef :: `undef multipl tt
  STAYS stimuli/sv/subs/sv2v/test/core/default_nettype.sv :: macro_body:default_nettype :: `default_nettype t \
  STAYS stimuli/sv/subs/sv2v/test/core/default_nettype.v :: macro_body:default_nettype :: `default_nettype t \
  STAYS stimuli/sv/subs/sv2v/test/lex/line.sv :: ILLEGAL:line:bad_number :: `line `new_line_num `foo(line) 1
  STAYS stimuli/sv/subs/sv2v/test/lex/string_macro.sv :: ILLEGAL:line:bad_number :: `line
  STAYS stimuli/sv/subs/verilator/test_regress/t/t_lint_pragma_protected_bad.v :: ILLEGAL:pragma:missing_name :: `pragma
  STAYS stimuli/sv/subs/verilator/test_regress/t/t_preproc.v :: ILLEGAL:pragma:bad_expression :: `pragma foo = 1
  STAYS stimuli/sv/subs/verilator/test_regress/t/t_preproc.v :: TRAILING:undef :: `undef CAT `undef ESC
  STAYS stimuli/sv/subs/verilator/test_regress/t/t_preproc.v :: TRAILING:undef :: `undef CAT `undef ESC
  STAYS stimuli/sv/subs/verilator/test_regress/t/t_preproc.v :: TRAILING:undef :: `undef FOO `undef ESC
  STAYS stimuli/sv/subs/verilator/test_regress/t/t_preproc.v :: TRAILING:undef :: `undef A `undef STR
  STAYS stimuli/sv/subs/verilator/test_regress/t/t_preproc_kwd.v :: ILLEGAL:begin_keywords:unknown_version :: `begin_keywords "VAMS-2.3"
  STAYS stimuli/sv/subs/verilator/test_regress/t/t_split_var_3_wreal.v :: ILLEGAL:begin_keywords:unknown_version :: `begin_keywords "VAMS-2.3"
  STAYS stimuli/sv/subs/verilator/test_regress/t/t_time_vpi.v :: ILLEGAL:timescale:bad_time_unit :: `timescale `time_scale_units / `time_scale_prec
  STAYS stimuli/sv/subs/verilator/test_regress/t/t_vams_basic.v :: ILLEGAL:begin_keywords:unknown_version :: `begin_keywords "VAMS-2.3"
  STAYS stimuli/sv/subs/verilator/test_regress/t/t_vams_kwd_bad.v :: ILLEGAL:begin_keywords:unknown_version :: `begin_keywords "VAMS-2.3"
  STAYS stimuli/sv/subs/verilator/test_regress/t/t_vams_wreal.v :: ILLEGAL:begin_keywords:unknown_version :: `begin_keywords "VAMS-2.3"
```

```
DIRECTIVE-CLASSIFY: rust/target/s339/directive_census_head_verilog_2005.tsv profile=verilog_2005 rows=353 parser_rows=310 regex_rows=43 typed=348 legal=347 ILLEGAL=0 TRAILING=0 opaque=5 macro_body=1 no_node=0 unclassified=0
  legal:begin_keywords                             53
  legal:celldefine                                 12
  legal:default_nettype                            44
  legal:end_keywords                               53
  legal:endcelldefine                              13
  legal:line                                       1
  legal:nounconnected_drive                        2
  legal:resetall                                   11
  legal:timescale                                  153
  legal:unconnected_drive                          2
  legal:undef                                      3
  macro_body:default_nettype                       1
  opaque:delay_mode_path                           1
  opaque:disable_portfaults                        1
  opaque:enable_portfaults                         1
  opaque:nosuppress_faults                         1
  opaque:suppress_faults                           1
ROWS (every ILLEGAL / TRAILING / macro_body / no_node / unclassified row, file status in brackets):
  macro_body:default_nettype                   [rejected] stimuli/sv/subs/sv2v/test/core/default_nettype.v :: `default_nettype t \
PREDICTION accepted files that must move pass -> fail: 0
PREDICTION rejected files whose failure position may move: 1
  STAYS stimuli/sv/subs/sv2v/test/core/default_nettype.v :: macro_body:default_nettype :: `default_nettype t \
```

## What the census decides

- **1 615 typed directive lines on the SV lanes** (794 read by the parser in 408 accepting files with a node, 967 by the
  regex in the 291 files the parser rejects for other reasons) and **348 on the `verilog_2005` lane**; the `sv_2023`
  reading differs from `sv_2017` in exactly one bucket — the five `"1800-2023"` version specifiers are `legal` there and
  `ILLEGAL:unknown_version` under 2017 — which is the edition contrast the two profiled `keywords_version_specifier`
  tokens spell.
- **The residue is 146 lines under 29 names** (`define` 23, `endif` 19, `include` 16, `ifdef` 12, the Verilog-XL
  `*_portfaults` / `*suppress_faults` families 44, `delay_mode_*` 7, `default_decay_time` 3, four macro uses …): every
  one stays an opaque line at file top level, by construction (`!lrm_directive_name` refuses only the thirteen names).
- **Every `ILLEGAL` row is a form a box refuses**, and the classifier's reason names which clause: 22 `line` rows in
  nine files, 12 (7 under 2023) `begin_keywords` rows in eight files — six of them Verilog-AMS versions, five
  `"1800-2023"` — 6 `timescale`, 4 `default_nettype`, 4 `unconnected_drive`, 4 `pragma`. The 10 `TRAILING` rows
  are seven `` `undef X `undef Y `` doubled directives (legal: the second is the next item), one `nounconnected_drive
  pull0`, one `timescale … /extra` and two Surelog `key_block encoding = (…)` lines (a comma-less spelling IEEE
  1800-2017 §34's own examples do not use: `docs/systemverilog/2017/md/section-34-protected-envelopes.md:130` writes
  `encoding=(…), data_block`).
- **`macro_body` = 2 (+1 on v2005)**: sv2v `test/core/default_nettype.{sv,v}`'s `` `default_nettype t \ `` sits on
  a continuation line of `` `define TEST(t) \ `` — macro text, which is why the adjudicator gained the
  `inside_define_continuation` rule rather than a per-file exception.
- **`no_node` = 1**: Surelog `tests/BeginKeywords/top.v`'s two directives sit inside a block comment.
- **PREDICTION, pre-registered**: on `sv_2017` exactly the **26** `MOVE` files go `pass -> fail` (23 on `sv_2023`:
  the three `"1800-2023"` files stay accepted there), **0** on `verilog_2005`; the `STAYS` files are already rejected
  and only their failure position may move — `t_time_vpi.v` and the two sv2v `default_nettype` rows onto a
  pass-through directive, which the two adjudicator refinements explain from the FILE.

## The grammar-candidate pre-screen (interpreter, HEAD grammar vs candidate, TOOLBOX §1.5c)

```
PRESCREEN-PINS: jobs=649 verdict_moved=0 ast_moved=0 in 583s
PRESCREEN-CORPUS: files=699 arms=1398 in 1666s
```

**49 `PASS->FAIL` arms = the 26 predicted files on `sv_2017` + the 23 on `sv_2023`, file for file; 0 `FAIL->PASS`;
23 `FURTHEST-MOVED` arms** in 12 already-rejected files (both SV profiles): the `default_nettype` continuation
goldens (`70 -> 49`, now stopping ON the macro-formal argument — `inside_define_continuation` explains them),
`t_time_vpi.v` (`697 -> 213`, ON `` `timescale `` whose arguments are macro uses — the argument rule explains it), the
five Verilog-AMS files (`-> 216..224`, ON `` `begin_keywords "VAMS-2.3" `` — the AMS lane owns them), Surelog's
`top_1.v` pair and `PragmaProtect/top.sv` (chained-only rows), `t_preproc_ifexpr.v` (a `t_preproc_*` row) — and ONE
that moved **forward**: iverilog `ivtest/ivltests/br_gh782b.v` `63 -> 360`. That file is iverilog's regression for
GitHub issue 782, a `` `timescale `` whose arguments are spread over SEVERAL LINES with block and line comments
between every token (`/* c */ `timescale /* c */ // c` ⏎ `/* c */ 1 /* c */ // c` ⏎ `/* c */ s …`). Under 1.0.216
the opaque token ended at the first newline and the parse died on the `1` of the next line — a stop svpp cannot
alter, so the row sat on the residual bar as `unexplained_rejects_valid`. Under the candidate a newline between
directive arguments is §5.3 white space, the directive parses, and the file stops at byte 360 ON `` `__FILE__ `` — a
§22.13 macro use the standing svpp rule explains. The §5.3 reading was a design decision; this row is its corpus
witness, keyed `must_accept` by iverilog's own suite (`regress-sv.list type normal`). Eight `STAYS` candidates did not
move because they already fail before their illegal directive line.

**Residual-bar prediction (axis 2, `sv_2017`), pre-registered:** `99 -> 95` — `t_timescale_parse_bad.v` and
`t_unconnected_bad.v` (`accepts_invalid -> match`), `t_vams_basic.v` (`unexplained_rejects_valid -> deferred:verilog_ams_lane`),
`br_gh782b.v` (`unexplained_rejects_valid -> explained_svpp_macro_use`).

## The same census on the release-1.0.217 probe (the final bytes)

Re-run on the complete population — **724** lane files selected on the directive TOKEN (`` `(name|…)\b `` anywhere,
the rule the anchored selector's 19 misses taught; `verilog_2005` lane ∩ = **241**) — with the classifier reading the
typed node the parser now writes (`typed:<kind>`), the `opaque` node's `text` textually, and the regex rows in the files
the parser rejects:

```
AST-FIELD-CENSUS: kind=compiler_directive field=body profile=sv_2017 files=724 accepted=393 rejected=331 timeout=0 nodump=0 nodes=743 distinct=144 fallback_rows=1023
AST-FIELD-CENSUS: kind=compiler_directive field=body profile=sv_2023 files=724 accepted=396 rejected=328 timeout=0 nodump=0 nodes=770 distinct=158 fallback_rows=1018
memory-guard: started pid=91821 pgid=91821 budget=8192MB floor=10% disk_floor=8GB interval=5s timeout=600s free=46% disk=3270GB cmd: python3 docs/tasks/artifacts/sv_corpus_grad/ast_field_census/ast_field_census.py --probe rust/target/release/parseability_probe --profile verilog_2005 --files rust/target/s339/directive_files_final_v2005.txt --kind compiler_directive --field body --out rust/target/s339/directive_census_final_verilog_2005.tsv --jobs 6 --timeout 120 --fallback-regex (?m)^[ \t]*(?P<field>`(?:resetall|undef|undefineall|timescale|default_nettype|unconnected_drive|nounconnected_drive|celldefine|endcelldefine|pragma|line|begin_keywords|end_keywords)\b[^\r\n]*)
```

```
DIRECTIVE-CLASSIFY: rust/target/s339/directive_census_final_sv_2017.tsv profile=sv_2017 rows=1781 parser_rows=743 regex_rows=1023 typed=1642 typed_nodes=619 legal=960 ILLEGAL=51 TRAILING=10 opaque=124 macro_body=2 no_node=2 unclassified=0
  ILLEGAL:begin_keywords                           12
  ILLEGAL:default_nettype                          4
  ILLEGAL:line                                     21
  ILLEGAL:pragma                                   4
  ILLEGAL:timescale                                6
  ILLEGAL:unconnected_drive                        4
  TRAILING:nounconnected_drive                     1
  TRAILING:pragma                                  2
  TRAILING:timescale                               1
  TRAILING:undef                                   6
  legal:begin_keywords                             80
  legal:celldefine                                 2
  legal:default_nettype                            51
  legal:end_keywords                               84
  legal:endcelldefine                              2
  legal:line                                       11
  legal:nounconnected_drive                        2
  legal:pragma                                     108
  legal:resetall                                   32
  legal:timescale                                  122
  legal:unconnected_drive                          1
  legal:undef                                      461
  legal:undefineall                                4
  macro_body:default_nettype                       2
  no_node                                          2
  opaque:BAR                                       1
  opaque:CONCAT                                    1
  opaque:FOO                                       1
  opaque:STRINGIFY                                 1
  opaque:accelerate                                1
  opaque:autoexpand_vectornets                     1
  opaque:default_decay_time                        3
  opaque:define                                    18
  opaque:delay_mode_distributed                    1
  opaque:delay_mode_path                           4
  opaque:delay_mode_unit                           1
  opaque:delay_mode_zero                           1
  opaque:disable_portfaults                        11
  opaque:else                                      2
  opaque:elsif                                     2
  opaque:enable_portfaults                         11
  opaque:endif                                     8
  opaque:error                                     1
  opaque:expand_vectornets                         1
  opaque:ifdef                                     3
  opaque:ifndef                                    5
  opaque:include                                   16
  opaque:noaccelerate                              1
  opaque:noexpand_vectornets                       1
  opaque:non_identifier                            1
  opaque:noremove_gatenames                        1
  opaque:noremove_netnames                         1
  opaque:nosuppress_faults                         11
  opaque:remove_gatenames                          1
  opaque:remove_netnames                           1
  opaque:suppress_faults                           11
  opaque:verilog                                   1
  typed:begin_keywords                             13
  typed:celldefine                                 23
  typed:default_nettype                            127
  typed:end_keywords                               15
  typed:endcelldefine                              26
  typed:line                                       8
  typed:nounconnected_drive                        8
  typed:pragma                                     18
  typed:resetall                                   65
  typed:timescale                                  294
  typed:unconnected_drive                          9
  typed:undef                                      7
  typed:undefineall                                6
  no_node                                      [accepted] stimuli/sv/subs/Surelog/tests/BeginKeywords/top.v :: <no compiler_directive node>
  no_node                                      [accepted] stimuli/sv/subs/iverilog/ivtest/ivltests/pr530a.v :: <no compiler_directive node>
PREDICTION accepted files that must move pass -> fail: 0
```

```
DIRECTIVE-CLASSIFY: rust/target/s339/directive_census_final_sv_2023.tsv profile=sv_2023 rows=1803 parser_rows=770 regex_rows=1018 typed=1642 typed_nodes=624 legal=960 ILLEGAL=46 TRAILING=10 opaque=146 macro_body=2 no_node=2 unclassified=0
  ILLEGAL:begin_keywords                           7
  ILLEGAL:default_nettype                          4
  ILLEGAL:line                                     21
  ILLEGAL:pragma                                   4
  ILLEGAL:timescale                                6
  ILLEGAL:unconnected_drive                        4
  TRAILING:nounconnected_drive                     1
  TRAILING:pragma                                  2
  TRAILING:timescale                               1
  TRAILING:undef                                   6
  legal:begin_keywords                             82
  legal:celldefine                                 2
  legal:default_nettype                            51
  legal:end_keywords                               83
  legal:endcelldefine                              2
  legal:line                                       11
  legal:nounconnected_drive                        2
  legal:pragma                                     108
  legal:resetall                                   32
  legal:timescale                                  122
  legal:unconnected_drive                          1
  legal:undef                                      460
  legal:undefineall                                4
  macro_body:default_nettype                       2
  no_node                                          2
  opaque:BAR                                       1
  opaque:CONCAT                                    1
  opaque:FOO                                       1
  opaque:STRINGIFY                                 1
  opaque:accelerate                                1
  opaque:autoexpand_vectornets                     1
  opaque:default_decay_time                        3
  opaque:define                                    20
  opaque:delay_mode_distributed                    1
  opaque:delay_mode_path                           4
  opaque:delay_mode_unit                           1
  opaque:delay_mode_zero                           1
  opaque:disable_portfaults                        11
  opaque:else                                      2
  opaque:elsif                                     3
  opaque:enable_portfaults                         11
  opaque:endif                                     18
  opaque:error                                     1
  opaque:expand_vectornets                         1
  opaque:ifdef                                     12
  opaque:ifndef                                    5
  opaque:include                                   16
  opaque:noaccelerate                              1
  opaque:noexpand_vectornets                       1
  opaque:non_identifier                            1
  opaque:noremove_gatenames                        1
  opaque:noremove_netnames                         1
  opaque:nosuppress_faults                         11
  opaque:remove_gatenames                          1
  opaque:remove_netnames                           1
  opaque:suppress_faults                           11
  opaque:verilog                                   1
  typed:begin_keywords                             16
  typed:celldefine                                 23
  typed:default_nettype                            127
  typed:end_keywords                               16
  typed:endcelldefine                              26
  typed:line                                       8
  typed:nounconnected_drive                        8
  typed:pragma                                     18
  typed:resetall                                   65
  typed:timescale                                  294
  typed:unconnected_drive                          9
  typed:undef                                      8
  typed:undefineall                                6
  no_node                                      [accepted] stimuli/sv/subs/Surelog/tests/BeginKeywords/top.v :: <no compiler_directive node>
  no_node                                      [accepted] stimuli/sv/subs/iverilog/ivtest/ivltests/pr530a.v :: <no compiler_directive node>
PREDICTION accepted files that must move pass -> fail: 0
```

```
DIRECTIVE-CLASSIFY: rust/target/s339/directive_census_final_verilog_2005.tsv profile=verilog_2005 rows=374 parser_rows=326 regex_rows=43 typed=364 typed_nodes=321 legal=42 ILLEGAL=0 TRAILING=0 opaque=5 macro_body=1 no_node=1 unclassified=0
  legal:begin_keywords                             13
  legal:default_nettype                            5
  legal:end_keywords                               13
  legal:resetall                                   2
  legal:timescale                                  6
  legal:undef                                      3
  macro_body:default_nettype                       1
  no_node                                          1
  opaque:delay_mode_path                           1
  opaque:disable_portfaults                        1
  opaque:enable_portfaults                         1
  opaque:nosuppress_faults                         1
  opaque:suppress_faults                           1
  typed:begin_keywords                             42
  typed:celldefine                                 14
  typed:default_nettype                            41
  typed:end_keywords                               42
  typed:endcelldefine                              15
  typed:line                                       1
  typed:nounconnected_drive                        4
  typed:resetall                                   11
  typed:timescale                                  147
  typed:unconnected_drive                          4
  no_node                                      [accepted] stimuli/sv/subs/iverilog/ivtest/ivltests/pr530a.v :: <no compiler_directive node>
PREDICTION accepted files that must move pass -> fail: 0
```

**What the final reading says.** Every directive node in an ACCEPTING file is a typed node (`sv_2017` 619 across all
thirteen kinds, `sv_2023` 624, `verilog_2005` 321 — where `` `undefineall `` is `opaque`, as designed) and **no accepting
file carries an ILLEGAL / TRAILING / macro_body row** (`PREDICTION accepted files that must move: 0` on every lane): the
51 / 46 / 0 illegal rows all sit in files the parser rejects, read by the regex. No `opaque` node carries a typed name —
the `!lrm_directive_name` guard held on every residue line the corpus writes (the one `opaque` text that is not an
identifier directive at all, sv2v `string_directive.sv`'s `` `"asdf `line`" `` §22.5.1 quote operator, is residue by
definition). The two `no_node` files are Surelog `BeginKeywords/top.v` (directives inside a block comment) and iverilog
`pr530a.v` (a directive token inside a string / comment): selected by text, unread by the parser, unmoved.

## Corpus AST identity sweep

**Whole lane, both SV profiles** (TOOLBOX §1.5e; HEAD probe rebuilt in a detached worktree from `g_head.ebnf` with the 1.0.216 generator snapshot — the regenerated parser is BYTE-IDENTICAL to the shipped 1.0.216 parser, `94a661afd7dccef1…`, and the rebuilt probe reproduces 1.0.216's verdicts on both opened rows — vs the release probe, 16 336 lane files × 2 SV profiles; the 174 `uvm-core` files re-swept at their real `stimuli/sv/uvm/` paths after the manifest-derived list mis-addressed them): `sv_2017` `both_accepted=9752 identical=8962 ast_moved=790 verdict_moved=30`, `sv_2023` `both_accepted=9776 identical=8982 ast_moved=794 verdict_moved=27` — the verdict movers are exactly the lanes' 26 + 4 and 23 + 4. ⭐ **The shape change is by design and its extent is proven TWO-SIDEDLY**: a census of every fresh-pass file for a `compiler_directive` node (`AST-FIELD-CENSUS: … files=9756 … nodes=4055`) finds the 794 files that carry one to be EXACTLY the 790 moved trees ∪ the 4 `fail → pass` rows, and the 8 962 files without one to be EXACTLY the identical trees (0 in either difference) — every tree that moved carries a directive, every tree without one is byte-identical.

## The same census on the FINAL 1.0.217 probe (host guard + residue-first order)

The parse-cost ratchet attributed the first typed spelling's +0.36 % entries rise to the typed alternation being tried at
71 972 item positions holding no directive; every host now guards its directive alternative with `&opaque_directive_line`
and `compiler_directive` is `@branch_policy: ordered` residue-first (the leaf's FIX (4)). The census was re-run on the
final probe (generated parser `07936ecb0b19912f…`, release probe `52d1ff8fecf06519…`, `rust/target/s339/census_final3.sh`)
over the same 724 / 241 files; every headline is IDENTICAL to the section above, per profile — the guard and the order
change no verdict and no node:

```
AST-FIELD-CENSUS: kind=compiler_directive field=body profile=sv_2017 files=724 accepted=393 rejected=331 timeout=0 nodump=0 nodes=743 distinct=144 fallback_rows=1023
DIRECTIVE-CLASSIFY: rust/target/s339/directive_census_final3_sv_2017.tsv profile=sv_2017 rows=1781 parser_rows=743 regex_rows=1023 typed=1642 typed_nodes=619 legal=960 ILLEGAL=51 TRAILING=10 opaque=124 macro_body=2 no_no
AST-FIELD-CENSUS: kind=compiler_directive field=body profile=sv_2023 files=724 accepted=396 rejected=328 timeout=0 nodump=0 nodes=770 distinct=158 fallback_rows=1018
DIRECTIVE-CLASSIFY: rust/target/s339/directive_census_final3_sv_2023.tsv profile=sv_2023 rows=1803 parser_rows=770 regex_rows=1018 typed=1642 typed_nodes=624 legal=960 ILLEGAL=46 TRAILING=10 opaque=146 macro_body=2 no_no
AST-FIELD-CENSUS: kind=compiler_directive field=body profile=verilog_2005 files=241 accepted=209 rejected=32 timeout=0 nodump=0 nodes=326 distinct=47 fallback_rows=43
DIRECTIVE-CLASSIFY: rust/target/s339/directive_census_final3_verilog_2005.tsv profile=verilog_2005 rows=374 parser_rows=326 regex_rows=43 typed=364 typed_nodes=321 legal=42 ILLEGAL=0 TRAILING=0 opaque=5 macro_body=1 no_n
```

The corpus AST identity sweep between the probe built before the guard and the final probe reads
`sv_2017` `both_accepted=9756 identical=9756 ast_moved=0 verdict_moved=0` and `sv_2023` `both_accepted=9780 identical=9780
ast_moved=0 verdict_moved=0` (`rust/target/s339/ast_sweep3_*.log`); the interpreter A/B over the 41 directive pins × 3
profiles reads 29 accepting arms byte-identical and 94 rejecting arms at identical positions, for the guard and again for
the order.
