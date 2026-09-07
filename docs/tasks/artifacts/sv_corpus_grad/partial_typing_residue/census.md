# `RAW-ENVELOPE-CENSUS:` — the partial-typing residue over the corpus (`SV-CORPUS-GRAD.3.45`)

Instrument: `raw_envelope_census.py` (this directory; TOOLBOX §1.5h). Population: EVERY lane row — the 16 336 files of
`stimuli/sv/characterization/results.tsv` under `sv_2017` and `sv_2023`, the 2 606 files of `results_v2005.tsv` under
`verilog_2005` — read with the release probe named in each block, no token pre-selection, so the census is of the corpus
rather than of a candidate list. Buckets and signatures: `README.md`.

## HEAD — release `1.0.218` (probe `00d6fdfc9724c12f…`, generated parser `69f5341307f2969d…`)

```
RAW-ENVELOPE-CENSUS: profile=sv_2017 files=16336 accepted=9756 rejected=6580 timeout=0 nodump=0 rows=5362 typed=2201 RAW=2325 passthrough=836 unclassified=0 alias_shadowed=2
RAW-ENVELOPE-CENSUS: profile=sv_2023 files=16336 accepted=9780 rejected=6556 timeout=0 nodump=0 rows=8439 typed=5815 RAW=1779 passthrough=845 unclassified=0 alias_shadowed=2
RAW-ENVELOPE-CENSUS: profile=verilog_2005 files=2606 accepted=2310 rejected=296 timeout=0 nodump=0 rows=528 typed=292 RAW=217 passthrough=19 unclassified=0 alias_shadowed=0
```

| rule : alternative | `sv_2017` | `sv_2023` | `verilog_2005` |
|---|---:|---:|---:|
| `assignment_pattern` : 1 positional (typed) | 554 | 555 | — |
| `assignment_pattern` : 2 structure-keyed **RAW** | 524 | 524 | — |
| `assignment_pattern` : 3 array-keyed **RAW** | 56 | 56 | — |
| `assignment_pattern` : 4 replication **RAW** | 60 | 60 | — |
| `assignment_pattern` : 5 empty (typed) | 7 | 7 | — |
| `block_data_type` : 1 integer vector (typed) | 712 | 712 | 130 |
| `block_data_type` : 2 integer atom **RAW** | 873 | 875 | 102 |
| `block_data_type` : 4 struct / union **RAW** | 1 | 1 | — |
| `block_data_type` : 5 enum **RAW** | 0 | 0 | — |
| `block_data_type` : 8 virtual interface **RAW** | 1 | 1 | — |
| `block_data_type` : the thirteen passthrough alternatives | 836 | 845 | 19 |
| `case_statement` : 1 plain (typed) | 646 | 646 | 97 |
| `case_statement` : 2 `matches` **RAW** | 16 | 16 | — |
| `case_statement` : 3 `inside` **RAW** | 15 | 15 | — |
| `clocking_declaration` : 1 (typed) | 143 | 143 | — |
| `clocking_declaration` : 2 `global` **RAW** | 27 | 27 | — |
| `clocking_event_sv_2017` : 1 `@id` (typed) | 32 | (twin) | — |
| `clocking_event_sv_2017` : 2 `@( … )` **RAW** | 548 | (twin) | — |
| `clocking_event_sv_2023` : ps / hierarchical / expression (typed, for parity) | — | 176 / 4 / 3465 | — |
| `list_of_path_delay_expressions` : 1 one value (typed) | 90 | 90 | 65 |
| `list_of_path_delay_expressions` : 2 two values **RAW** | 193 | 193 | 109 |
| `list_of_path_delay_expressions` : 3 three values **RAW** | 4 | 4 | 2 |
| `list_of_path_delay_expressions` : 4 six values **RAW** | 5 | 5 | 3 |
| `list_of_path_delay_expressions` : 5 twelve values **RAW** | 2 | 2 | 1 |
| `net_type_declaration_sv_2017` / `nettype_declaration_sv_2023` : data-type form (typed) | 17 | 17 | — |
| … of which `alias_shadowed` (the §6.6.7 alias form parsed down the data-type alternative) | 2 | 2 | — |
| `net_type_declaration_sv_2017` : 2 alias form **RAW** | 0 | (typed twin: 0) | — |
| `property_case_item` : 1 / 2 | 0 | 0 | — |

Two readings the census corrected in the leaf that opened it: the nettype alias form is SHADOWED rather than raw (Protocol D
on IEEE 1800-2017 §6.6.7's own example: `🏁 Rule 'net_type_declaration_sv_2017' selected branch 1/2` after the alias
alternative's parts had succeeded; the same for the fully typed `sv_2023` twin), and `list_of_path_delay_expressions`
alternatives 6 and 7 can never fire (a duplicate six-value spelling that loses the tie to alternative 4 — `selected branch
4/7` on six values — and a twelve-value spelling whose middle four positions are the literal text
`tx_path_delay_expression`, 0 corpus files). The corpus writes no `property_case_item` and no block-scoped anonymous `enum`
on any lane; the pins and the shape-contract samples are the only witnesses of those shapes.

```
PREDICTION sv_2017:      raw_files=1092 alias_shadowed_files=2 movers=1094
PREDICTION sv_2023:      raw_files=819  alias_shadowed_files=2 movers=821
PREDICTION verilog_2005: raw_files=120  alias_shadowed_files=0 movers=120
```

The two alias declarations per SV lane: `stimuli/sv/subs/verilator/test_regress/t/t_nettype.v` and
`stimuli/sv/subs/ispras-sv-tests/ieee-1800-2012/06/06.06.07_01.sv`. The movers lists (`movers_head_<profile>.txt`) are the
sets the release's corpus AST identity sweep is read against two-sidedly.

⚠️ The census's first run predicted 1 088 / 819 / 120 raw files with `RAW=2319` on `sv_2017`; the pin pre-screen read one
moved arm the census had not predicted (`defect_systf_clocking_event.sv`: the clocking event inside the optional group's
LIST under the `clocking` key of `$rose(a, @(posedge clk))`), the context key is now carried through lists, and the census
above is the re-run — six more system-call clocking events on `sv_2017` (2 325), the same 1 779 on `sv_2023` (its event
twin was already typed), 217 unchanged on the 1364-2005 lane.

## FINAL — release `1.0.219`

Release probe `011dbe9f8f1a2dd5…`, generated parser `6578eda69d2a1695…` (the third spelling — see the leaf for the two refuted before it).

```
RAW-ENVELOPE-CENSUS: profile=sv_2017 files=16336 accepted=9756 rejected=6580 timeout=0 nodump=0 rows=5362 typed=4526 RAW=0 passthrough=836 unclassified=0 alias_shadowed=0
RAW-ENVELOPE-CENSUS: profile=sv_2023 files=16336 accepted=9780 rejected=6556 timeout=0 nodump=0 rows=8439 typed=7594 RAW=0 passthrough=845 unclassified=0 alias_shadowed=0
RAW-ENVELOPE-CENSUS: profile=verilog_2005 files=2606 accepted=2310 rejected=296 timeout=0 nodump=0 rows=528 typed=509 RAW=0 passthrough=19 unclassified=0 alias_shadowed=0
```

`RAW=0 unclassified=0 alias_shadowed=0` on every lane; rows conserved (5 362 / 8 439 / 528); every former RAW row and every alias declaration reads `typed`.

The corpus AST identity sweep (the 1.0.218 probe vs 1.0.219, every lane row, TOOLBOX §1.5e):

```
AST-IDENTITY-SWEEP: profile=sv_2017 files=16336 both_accepted=9756 identical=8662 ast_moved=1094 verdict_moved=0 both_nonaccepting=6580
AST-IDENTITY-SWEEP: profile=sv_2023 files=16336 both_accepted=9780 identical=8959 ast_moved=821 verdict_moved=0 both_nonaccepting=6556
AST-IDENTITY-SWEEP: profile=verilog_2005 files=2606 both_accepted=2310 identical=2190 ast_moved=120 verdict_moved=0 both_nonaccepting=296
```

TWO-SIDED against the HEAD movers above: `sv_2017`: predicted 1094, moved 1094, predicted-but-identical 0, moved-but-not-predicted 0; `sv_2023`: predicted 821, moved 821, predicted-but-identical 0, moved-but-not-predicted 0; `verilog_2005`: predicted 120, moved 120, predicted-but-identical 0, moved-but-not-predicted 0.

