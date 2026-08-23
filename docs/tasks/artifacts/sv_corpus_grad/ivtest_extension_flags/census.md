# ivtest `-g` extension-flag census — SV-CORPUS-GRAD.13e.3

> DERIVED. Re-run: `python3 docs/tasks/artifacts/sv_corpus_grad/ivtest_extension_flags/census.py`
> The language-affecting classification is IMPORTED from `stimuli/sv/adjudicate_external_corpus.py`, which derives it from the vendored
> compiler and REFUSES when that compiler moves — never re-stated here.

- vvp descriptors scanned: **560**
- descriptors carrying a non-generation `-g` flag: **43**
- distinct source files behind them: **40**
- distinct non-generation `-g` flags: **11**

## Flags, by descriptor count

| flag | descriptors | decides the language? |
|---|---:|---|
| `-gverilog-ams` | 20 | **YES** |
| `-gspecify` | 9 | no |
| `-ginterconnect` | 5 | no |
| `-gno-xtypes` | 3 | **YES** |
| `-gno-strict-declaration` | 3 | no |
| `-gxtypes` | 2 | **YES** |
| `-gno-strict-net-var-declaration` | 2 | no |
| `-gno-specify` | 1 | no |
| `-gno-icarus-misc` | 1 | **YES** |
| `-gno-strict-parameter-declaration` | 1 | no |
| `-gsupported-assertions` | 1 | no |

## Every carrier, with its current adjudication

| descriptor | type | `iverilog-args` | lane | adjudication |
|---|---|---|---|---|
| `analog1` | NI | `-gverilog-ams` | sv | `deferred:verilog_ams_lane` |
| `analog2` | NI | `-gverilog-ams` | sv | `deferred:verilog_ams_lane` |
| `begin_keywords_vams_2_3` | normal | `-g2005-sv -gverilog-ams` | sv | `deferred:verilog_ams_lane` |
| `br_gh1087a1` | CE | `-gno-xtypes` | sv | `match` |
| `br_gh1087a2` | CE | `-gxtypes` | sv | `match` |
| `br_gh1087a3` | CE | `-g2009 -gno-xtypes` | sv | `match` |
| `br_gh1087b` | CE | `-gxtypes` | v2005 | `match` |
| `br_gh1087c` | CE | `-g2009 -gno-xtypes` | sv | `match` |
| `br_gh1184` | EF | `-gspecify` | v2005 | `match` |
| `br_gh1248` | normal | `-Ttyp -ginterconnect -gspecify` | v2005 | `match` |
| `br_gh1258a` | normal | `-gno-specify` | v2005 | `match` |
| `br_gh1258b` | normal | `-gspecify` | v2005 | `match` |
| `br_gh552` | CE | `-gno-icarus-misc` | v2005 | `match` |
| `br_gh99c` | normal | `-gverilog-ams` | sv | `deferred:verilog_ams_lane` |
| `cast_int_ams` | normal | `-gverilog-ams` | sv | `deferred:verilog_ams_lane` |
| `constfunc4_ams` | normal | `-gverilog-ams` | sv | `deferred:verilog_ams_lane` |
| `decl_before_use1_warn` | normal | `-gno-strict-net-var-declaration` | v2005 | `match` |
| `decl_before_use2_warn` | normal | `-gno-strict-declaration` | v2005 | `match` |
| `decl_before_use3_warn` | normal | `-gno-strict-net-var-declaration` | v2005 | `match` |
| `decl_before_use4_warn` | normal | `-gno-strict-declaration -Wno-declaration-after-use` | v2005 | `match` |
| `decl_before_use5_warn` | normal | `-gno-strict-parameter-declaration` | v2005 | `match` |
| `decl_before_use6_warn` | normal | `-gno-strict-declaration -Wno-declaration-after-use` | v2005 | `match` |
| `delayed_sfunc` | normal | `-gspecify` | v2005 | `match` |
| `scaled_real` | normal | `-gverilog-ams` | sv | `deferred:verilog_ams_lane` |
| `sdf_header` | normal | `-gspecify` | v2005 | `match` |
| `sdf_interconnect1` | normal | `-Ttyp -ginterconnect -gspecify` | v2005 | `match` |
| `sdf_interconnect2` | normal | `-Ttyp -ginterconnect -gspecify` | v2005 | `match` |
| `sdf_interconnect3` | normal | `-Ttyp -ginterconnect -gspecify` | v2005 | `match` |
| `sdf_interconnect4` | normal | `-Ttyp -ginterconnect -gspecify` | v2005 | `match` |
| `sv_type_identifier_ams_name_fields` | normal | `-g2005-sv -gverilog-ams` | sv | `deferred:verilog_ams_lane` |
| `sv_type_identifier_assert_item_label` | normal | `-g2012 -gsupported-assertions` | sv | `match` |
| `sv_type_identifier_config_name` | normal | `-g2005-sv -gverilog-ams` | sv | `deferred:verilog_ams_lane` |
| `sv_type_identifier_discipline_name` | normal | `-g2005-sv -gverilog-ams` | sv | `deferred:verilog_ams_lane` |
| `sv_type_identifier_discipline_nature_ref` | normal | `-g2005-sv -gverilog-ams` | sv | `deferred:verilog_ams_lane` |
| `sv_type_identifier_nature_name` | normal | `-g2005-sv -gverilog-ams` | sv | `deferred:verilog_ams_lane` |
| `test_vams_math` | normal | `-gverilog-ams` | sv | `deferred:verilog_ams_lane` |
| `value_range1` | normal | `-gverilog-ams` | sv | `deferred:verilog_ams_lane` |
| `value_range2` | normal | `-gverilog-ams` | sv | `deferred:verilog_ams_lane` |
| `value_range3` | CE | `-gverilog-ams` | sv | `deferred:verilog_ams_lane` |
| `vams_abs1` | normal | `-gverilog-ams` | sv | `deferred:verilog_ams_lane` |
| `vams_abs2` | normal | `-gverilog-ams` | sv | `deferred:verilog_ams_lane` |
| `vams_abs3` | normal | `-gverilog-ams` | sv | `deferred:verilog_ams_lane` |
| `wreal` | normal | `-gverilog-ams` | sv | `deferred:verilog_ams_lane` |

## Roll-up

| lane | adjudication | descriptors |
|---|---|---:|
| sv | `deferred:verilog_ams_lane` | 20 |
| v2005 | `match` | 18 |
| sv | `match` | 5 |

