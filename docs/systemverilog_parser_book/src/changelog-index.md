# Changelog Index

This chapter is an index — pointers into other docs that carry the full changelog detail. Use it to find what changed in a given release.

## Where the canonical changelogs live

| Source | Granularity | Purpose |
|---|---|---|
| `docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md` | Per-release shape change | The authoritative contract. Each release's section lists the AST shape changes consumers care about. |
| `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md` | Per-bug | When a bug is fixed in a release, the ledger entry records the input/output shape change. |
| `CHANGES.md` (root) | Per-release | Human-readable summary of all changes. |
| Git tags + commit log | Commit-by-commit | The most granular source. |

When investigating "what changed and why," start with the contract document, drop down to the ledger for specific bugs, fall back to git for diffs.

## Releases relevant to this book

> ✅ **THE GAP IS CLOSED — 2026-08-21, `SV-CORPUS-GRAD.13c.2z`.** This index jumped from `1.0.183`
> to `1.0.193`, omitting **`1.0.184`–`1.0.192`** (ledger `SV-0054`…`SV-0065`, nine releases, four of
> which REPLACE an AST shape a consumer was already reading). All nine are now published below,
> authored from `docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md` and
> `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md` — the authoritative per-release and per-bug
> records — rather than paraphrased. `schema-versioning.md`'s missing rows **21**–**25** are
> back-filled in the same change.
>
> ⛔ **The back-fill is the smaller half.** The same table silently skipped schemas `17`–`19` once
> before and was hand-reconstructed then too; a hand-fix plus a prose warning bought exactly one
> cycle. Nothing in the repository compared a family book's newest published release against the
> family's actual release, which is why the gap could grow from three releases to nine unseen. That
> check now exists — `scripts/check_parser_book_currency.sh`, doctrine `PARSER-BOOK-CURRENCY` — and
> it fails in both directions for every family that has a book.

This book is **live** and tracks current main HEAD. Versioning summary:

- The most recent **published** parser-release section in the contract is **1.0.0 / Contract 1.0.0** (foundation baseline).

### 1.0.253 / Contract 1.0.253 — SV-CORPUS-GRAD.50 (`PGEN-SV-CORPUS-GRAD-0442`, 2026-10-16), ledger `SV-0138` (`Released`), **schema 34 unchanged**:
**THE INSTANCE'S PARAMETER VALUE ASSIGNMENT AND THE PACKED-DIMENSION LIST ARE SPLIT BY EDITION — under `verilog_2005` `mod #() m1(…)` and a SECOND packed range on a reg, a named-block reg, a function range, a port or a task port are refused; `sv_2017` / `sv_2023` / `sv_2023_tolerant` unchanged.** IEEE 1364-2005 A.4.1.1 `parameter_value_assignment ::= # ( list_of_parameter_assignments )` with a non-empty list (`docs/verilog/2005/txt/section-Annex_A-normative-formal-syntax-definition.txt:399-402`) and the ONE optional `range` of A.2.1.3 `reg_declaration` (`docs/verilog/2005/txt/section-Annex_A-normative-formal-syntax-definition.txt:172`), A.2.1.2 `input/inout/output_declaration` (`:133-139`), A.2.6 `function_range_or_type` (`:268`) and A.2.7 `tf_*_declaration` (`:296-305`), against IEEE 1800-2017 A.4.1.1 `# ( [ list ] )` (`docs/systemverilog/2017/md/section-41-data-read-api.md:1795`) and A.2.2.1 `{ packed_dimension }` (`docs/systemverilog/2017/md/section-41-data-read-api.md:842`, `:867`). Through `1.0.252` the 1800 shapes served the `verilog_2005` profile at these rules (the net site had been split by `.3.66` in `1.0.236`). The twins: `parameter_value_assignment_v2005 := hash lparen list_of_parameter_assignments rparen -> {params: $3}` under the dispatcher's `kind: "sv_2017"` (the shape's name), and `packed_dimension_list := packed_dimension_list_sv_only -> $1 | packed_dimension_list_v2005 -> $1` with `packed_dimension_list_sv_only := packed_dimension* -> $1` (the SV profiles) and `packed_dimension_list_v2005 := ( packed_dimension )? -> $1` at `data_type` alternative 1, `block_data_type` alternative 1 and `implicit_data_type` — the spelling measured byte-identical to the inline repetition on a synthetic carrier before the grammar was touched. Rule inventory 1749 → 1753. The adjudicator's EDITION-PACKED-DIMENSION pass re-keys the ten verilog_2005-lane corpus rows iverilog `-g2005` had keyed `must_accept` (a tool extension; the clause decides): `edition-packed-dimension pass [verilog_2005]: 10 row(s) re-keyed must_reject (the stop is ON a second packed range in a reg / port / function / task declaration or ON an empty #(); sv_2017 accepts)`.

**Measured before a byte was generated.** `--lint-grammar` rc 0 (`1753` rules); `SAMPLE-PROFILE-PARITY: … profiles=4 samples=60 checks=145 defects=0`; the targeted A/B on the interpreter `AB-TARGETED: jobs=34 unexpected=1` (every flipped pin and site probe `true → false` under `verilog_2005` only, every control `true → true` with an identical typed-AST sha); the eight new pins `PRESCREEN-NEW-PINS: jobs=12 verdict_moved=5 ast_moved=0`; `PRESCREEN-PINS: jobs=2021 verdict_moved=8 ast_moved=0 in 2202s`; `PRESCREEN-CORPUS: files=201 arms=603 in 791s` — `PASS->FAIL verilog_2005 stimuli/sv/subs/iverilog/ivtest/ivltests/array_packed_2d.v 2355 -> 910`; `PASS->FAIL verilog_2005 stimuli/sv/subs/iverilog/ivtest/ivltests/br_gh497c.v 876 -> 23`; `PASS->FAIL verilog_2005 stimuli/sv/subs/iverilog/ivtest/ivltests/br_gh497d.v 1329 -> 163`; `PASS->FAIL verilog_2005 stimuli/sv/subs/iverilog/ivtest/ivltests/br_gh497e.v 822 -> 23`; `PASS->FAIL verilog_2005 stimuli/sv/subs/iverilog/ivtest/ivltests/br_gh497f.v 1826 -> 163`; `PASS->FAIL verilog_2005 stimuli/sv/subs/iverilog/ivtest/ivltests/partsel_reversed_idx2.v 267 -> 138`; `PASS->FAIL verilog_2005 stimuli/sv/subs/iverilog/ivtest/ivltests/partsel_reversed_idx4.v 262 -> 138`; `PASS->FAIL verilog_2005 stimuli/sv/subs/iverilog/ivtest/ivltests/partsel_reversed_idx6.v 267 -> 132`; `FURTHEST-MOVED verilog_2005 stimuli/sv/subs/sv2v/test/core/string_byte_order.v 168 -> 87`; `PASS->FAIL verilog_2005 stimuli/sv/subs/sv2v/test/core/tf_unpacked_input.v 764 -> 71`; the envelope differential `divergence_total=203` on both grammars.

**Measured on the final bytes.** `ADJUDICATION-REPROS: checked=2020 armed=236 listed=985 multi_profile_rows=759 preprocess_rows=41 tolerated_rows=8 superset_rows=760 failures=0`; the three raw lanes `LANE-MOVERS lane=sv_2017 rows_before=16336 rows_after=16336 added=0 lost=0 pass_to_fail=0 fail_to_pass=0 other=0`; `LANE-MOVERS lane=sv_2023 rows_before=16336 rows_after=16336 added=0 lost=0 pass_to_fail=0 fail_to_pass=0 other=0`; `LANE-MOVERS lane=verilog_2005 rows_before=2943 rows_after=2943 added=0 lost=0 pass_to_fail=10 fail_to_pass=0 other=0` (`MOVED verilog_2005 ('iverilog', 'stimuli/sv/subs/iverilog/ivtest/ivltests/array_packed_2d.v') pass -> fail`; `MOVED verilog_2005 ('iverilog', 'stimuli/sv/subs/iverilog/ivtest/ivltests/br_gh497c.v') pass -> fail`; `MOVED verilog_2005 ('iverilog', 'stimuli/sv/subs/iverilog/ivtest/ivltests/br_gh497d.v') pass -> fail`; `MOVED verilog_2005 ('iverilog', 'stimuli/sv/subs/iverilog/ivtest/ivltests/br_gh497e.v') pass -> fail`; `MOVED verilog_2005 ('iverilog', 'stimuli/sv/subs/iverilog/ivtest/ivltests/br_gh497f.v') pass -> fail`; `MOVED verilog_2005 ('iverilog', 'stimuli/sv/subs/iverilog/ivtest/ivltests/partsel_reversed_idx2.v') pass -> fail`; `MOVED verilog_2005 ('iverilog', 'stimuli/sv/subs/iverilog/ivtest/ivltests/partsel_reversed_idx4.v') pass -> fail`; `MOVED verilog_2005 ('iverilog', 'stimuli/sv/subs/iverilog/ivtest/ivltests/partsel_reversed_idx6.v') pass -> fail`; `MOVED verilog_2005 ('sv2v', 'stimuli/sv/subs/sv2v/test/core/tf_unpacked_input.v') pass -> fail`; `MOVED verilog_2005 ('sv2v', 'stimuli/sv/subs/sv2v/test/core/unbased_unsized_function.v') pass -> fail`); AST identity `AST-IDENTITY-SWEEP: profile=sv_2017 files=16336 both_accepted=10022 identical=10022 ast_moved=0 verdict_moved=0 both_nonaccepting=6314`, `AST-IDENTITY-SWEEP: profile=sv_2023 files=16336 both_accepted=10048 identical=10048 ast_moved=0 verdict_moved=0 both_nonaccepting=6288`, `AST-IDENTITY-SWEEP: profile=verilog_2005 files=2943 both_accepted=2548 identical=2548 ast_moved=0 verdict_moved=10 both_nonaccepting=385`; the adjudicator `match=6310 unexplained=0 explained=1420 deferred=8606`, `v2005: match=2691 unexplained=0 explained=205 deferred=47`; the second pass `PASS2: the sv_2023 manifest unchanged by the second pass`; `✅ SV recognized cert-coverage union gate passed (canonical UNKNOWN=14, union UNKNOWN=0, residual=[], deterministic across seeds [0,7,42]).`; `✅ verilog_2005 conformance gate passed (lint orphans=0, corpus 369 checks/0 mismatches, aliases 2, cert deterministic across seeds [0,7,42]).`; `unreachable_rules: 11`; `[ast_shape_contract][systemverilog] samples=60 aligned=60 drift=0 regression_lock_failures=0 aligned_assertion_failures=0 samples_per_status={"aligned": 60}`; the pipeline `PIPELINE-MOVERS[sv_2017] rows_before=16336 rows_after=16336 moved=0 added=0 lost=0`, `PIPELINE-MOVERS[sv_2023] rows_before=16336 rows_after=16336 moved=0 added=0 lost=0`, `PIPELINE-MOVERS[sv_2023_tolerant] rows_before=16336 rows_after=16336 moved=0 added=0 lost=0`; all eleven family umbrellas green; `RULE-FIRE-PARTITION: profile=sv_2017 gap_rules=91 (cert total=1533 witness=1506 proof=24 UNKNOWN=3)`; `sv-rule-fire-partition: OK (coverage_gap=52, lr_eliminated_original=2, lr_synthetic=14, unreachable_from_entry=23; 91 never-fired rules, 0 unadjudicated; census rule set = grammar inventory, 1753 rules)`; `SV-CEREMONY-BINARIES: OK verify copies=3 published=3 reports=5 memo=OK recorded=2 notes=0`.

**Consumers.** Nothing to change under `sv_2017`, `sv_2023` or `sv_2023_tolerant`. Under `verilog_2005` an instance's `#()` and a second packed range on a reg / named-block reg / function range / port / task port are refused from `1.0.253`; the node of a legal instance or declaration is the one you already had — `{params}` under `kind: "sv_2017"`, the same `dims` list.

### 1.0.252 / Contract 1.0.252 — SV-CORPUS-GRAD.48 (`PGEN-SV-CORPUS-GRAD-0436`, 2026-10-09), ledger `SV-0137` (`Released`), **schema 34 unchanged**:
**THE PARAMETER PORT LIST IS SPLIT BY EDITION — under `verilog_2005` a module's `#(…)` list is `parameter`-led declarations, at least one, and nothing else; `sv_2017` / `sv_2023` / `sv_2023_tolerant` unchanged.** IEEE 1364-2005 A.1.3 `module_parameter_port_list ::= # ( parameter_declaration { , parameter_declaration } )` (`docs/verilog/2005/txt/section-Annex_A-normative-formal-syntax-definition.txt:38`) and A.2.1.1 `parameter_declaration ::= parameter [ signed ] [ range ] list_of_param_assignments | parameter parameter_type list_of_param_assignments` (`:126-128`), against IEEE 1800-2017 A.1.3 `parameter_port_list ::= # ( list_of_param_assignments { , parameter_port_declaration } ) | # ( parameter_port_declaration { , … } ) | #( )` with `parameter_port_declaration ::= … | local_parameter_declaration | data_type list_of_param_assignments` (`docs/systemverilog/2017/md/section-41-data-read-api.md:231-243`). Through `1.0.251` ONE `parameter_port_list` rule served every profile with the 1800 production, so `module a #(A = 1);`, `#(integer A = 1)`, `#(parameter real A = 1.0, integer B = 2)` (ivtest `parameter_omit1/2/3.v`, each headed *"In Verilog this is not allowed and should result in an error"*), `#()` and `#(parameter X = 1, localparam Y = 2)` parsed under `verilog_2005`. `1.0.252` gives the production a pass-through dispatcher over the 1800 rule (`parameter_port_list_sv_only`, verbatim, for the SV profiles) and a `verilog_2005` twin (`parameter_port_list_v2005`: each declaration admitted behind a `parameter` lookahead, the shared `parameter_port_declaration` node) — the `declarations` kind and the same items, so every tree the profile still accepts is byte-identical, by construction and measured. The sibling `parameter_value_assignment` (`mod #() m1(…)`, 1364-2005 A.4.1.1) stays admitted and is routed (`SV-CORPUS-GRAD.50`, watched by a pin).

**Measured before a byte was generated.** `--lint-grammar` rc 0 (`1749` rules, 1746 → 1749); `SAMPLE-PROFILE-PARITY: … profiles=4 samples=60 checks=145 defects=0`; the interpreter on the `1.0.251` grammar against the candidate — the three over-acceptance pins `accepted=true → false` under `verilog_2005` only, every control `true → true` with the control's typed AST byte-identical; `PRESCREEN-PINS: jobs=1985 verdict_moved=4 ast_moved=0 in 2161s` (`VERDICT control_port_lists_without_trailing_comma.sv verilog_2005: head=True new=False expect=ACCEPT`; `VERDICT accepts_invalid_v2005_parameter_port_no_keyword.sv verilog_2005: head=True new=False expect=ACCEPT`; `VERDICT accepts_invalid_v2005_parameter_port_type_no_keyword.sv verilog_2005: head=True new=False expect=ACCEPT`; `VERDICT accepts_invalid_v2005_parameter_port_second_decl_no_keyword.sv verilog_2005: head=True new=False expect=ACCEPT` — the three pins and one `.3.51` control mislisted on `verilog_2005`, narrowed to the IEEE 1800 profiles); `PRESCREEN-NEW-PINS: jobs=19 verdict_moved=3 ast_moved=0` (the seven pins listed after that run); `PRESCREEN-CORPUS: files=184 arms=552 in 716s` over the 184 verilog_2005-lane files that spell `#(`, three profiles — `PASS->FAIL verilog_2005 stimuli/sv/subs/iverilog/ivtest/ivltests/parameter_omit1.v 375 -> 189`; `PASS->FAIL verilog_2005 stimuli/sv/subs/iverilog/ivtest/ivltests/parameter_omit2.v 385 -> 189`; `PASS->FAIL verilog_2005 stimuli/sv/subs/iverilog/ivtest/ivltests/parameter_omit3.v 457 -> 239`.

**Measured on the final bytes.** `ADJUDICATION-REPROS: checked=1997 armed=236 listed=972 multi_profile_rows=752 preprocess_rows=41 tolerated_rows=8 superset_rows=753 failures=0`; the three raw lanes `LANE-MOVERS lane=sv_2017 rows_before=16336 rows_after=16336 added=0 lost=0 pass_to_fail=0 fail_to_pass=0 other=0`; `LANE-MOVERS lane=sv_2023 rows_before=16336 rows_after=16336 added=0 lost=0 pass_to_fail=0 fail_to_pass=0 other=0`; `LANE-MOVERS lane=verilog_2005 rows_before=2943 rows_after=2943 added=0 lost=0 pass_to_fail=3 fail_to_pass=0 other=0` (`MOVED verilog_2005 ('iverilog', 'stimuli/sv/subs/iverilog/ivtest/ivltests/parameter_omit1.v') pass -> fail`; `MOVED verilog_2005 ('iverilog', 'stimuli/sv/subs/iverilog/ivtest/ivltests/parameter_omit2.v') pass -> fail`; `MOVED verilog_2005 ('iverilog', 'stimuli/sv/subs/iverilog/ivtest/ivltests/parameter_omit3.v') pass -> fail`); AST identity `AST-IDENTITY-SWEEP: profile=sv_2017 files=16336 both_accepted=10022 identical=10022 ast_moved=0 verdict_moved=0 both_nonaccepting=6314`, `AST-IDENTITY-SWEEP: profile=sv_2023 files=16336 both_accepted=10048 identical=10048 ast_moved=0 verdict_moved=0 both_nonaccepting=6288`, `AST-IDENTITY-SWEEP: profile=verilog_2005 files=2943 both_accepted=2558 identical=2558 ast_moved=0 verdict_moved=3 both_nonaccepting=382` (controls green 0, red 1); the adjudicator `match=6266 unexplained=0 explained=1418 deferred=8652`, `v2005: match=2691 unexplained=0 explained=206 deferred=46` (the three rows `→ match`, the verilog_2005 bar 3 → 0); `✅ SV recognized cert-coverage union gate passed (canonical UNKNOWN=14, union UNKNOWN=0, residual=[], deterministic across seeds [0,7,42]).`; `✅ verilog_2005 conformance gate passed (lint orphans=0, corpus 369 checks/0 mismatches, aliases 2, cert deterministic across seeds [0,7,42]).`; syntax closure `unreachable_rules: 11`; `[ast_shape_contract][systemverilog] samples=60 aligned=60 drift=0 regression_lock_failures=0 aligned_assertion_failures=0 samples_per_status={"aligned": 60}`; every generated artifact re-derives byte-identically; parse cost: `BINDING metric `entries` ROSE 408,220,054 -> 408,220,495 (+0.00 %)`; `BINDING metric `committed` ROSE 7,090,581 -> 7,090,604 (+0.00 %)` — the dispatcher's arrival, accepted under `profile_split_respelling` (the `.3.66` precedent), then `parse-cost-ratchet: OK (identity fresh for: generated parser, grammar raw ast, instrument, sample inputs; 192 pinned sample files)`; the pipeline lane `PIPELINE-MOVERS[sv_2017] rows_before=16336 rows_after=16336 moved=0 added=0 lost=0`, `PIPELINE-MOVERS[sv_2023] rows_before=16336 rows_after=16336 moved=0 added=0 lost=0`, `PIPELINE-MOVERS[sv_2023_tolerant] rows_before=16336 rows_after=16336 moved=0 added=0 lost=0`; all eleven family umbrellas green; `RULE-FIRE-PARTITION: profile=sv_2017 gap_rules=91 (cert total=1531 witness=1504 proof=24 UNKNOWN=3)`; `sv-rule-fire-partition: OK (coverage_gap=52, lr_eliminated_original=2, lr_synthetic=14, unreachable_from_entry=23; 91 never-fired rules, 0 unadjudicated; census rule set = grammar inventory, 1749 rules)`.

**Consumers.** Nothing to change under `sv_2017`, `sv_2023` or `sv_2023_tolerant`. Under `verilog_2005` a parameter port list that IEEE 1800 admits and 1364-2005 does not — a declaration without `parameter`, a `localparam` in the list, an empty `#()` — is refused from `1.0.252`; the node of a legal list is the one you already handle (`{kind: "declarations", items: [...]}`).

### 1.0.251 / Contract 1.0.251 — SV-CORPUS-GRAD.47 (`PGEN-SV-CORPUS-GRAD-0432`, 2026-10-07), ledger `SV-0136` (`Released`), **schema 34 unchanged**:
**THE SIZE TOKEN IS SPLIT BY EDITION — a sized literal whose size begins with `0` parses under `sv_2023`; `sv_2017` and `verilog_2005` keep rejecting it.** IEEE 1800-2023 §5.7 Syntax 5-2 and A.8.7 read `size ::= unsigned_number` (`docs/systemverilog/2023/md/section-5-lexical-conventions.md:213`, `docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:3200`) and the edition defines no `non_zero_unsigned_number`; 1800-2017 §5.7 / A.8.7 and 1364-2005 §3.5 / A.8.7 read `size ::= non_zero_unsigned_number`. Through `1.0.250` one size terminal served every profile with the older production, so `sv_2023` rejected text its own edition derives (`01'b0`; friscv `rtl/friscv_alu.sv`, 27 sites) — a REJECTS-VALID defect since `1.0.235`, whose record had read 1800-2023 as agreeing with 2017 (`SV-0120`, corrected forward). The 2023 prose keeps the size *"a nonzero unsigned decimal number"*, so a ZERO size (`0'b1`) stays rejected on every profile. GRAMMAR: ONE terminal twin, `integral_number_leading_zero_size_sv_2023`, gated to `sv_2023` (and the tolerant profile, which inherits every `sv_2023` rule), returning the strict shape — the literal arrives as `{kind: "integral", body}` with the source text as `body`, no new kind. ⛔ **Not a tolerance.** The 2026-10-06 session had designed this spelling as the opt-in profile's sixth member on an ecosystem reading (`SV-CORPUS-GRAD.46`: every tool on this host accepts it); the pins' citations, checked against the 2023 text, refuted the design during the recovery from a host crash, and the release was re-run as an edition split. The opt-in list is unchanged at five.

**Measured before a byte was generated.** `--lint-grammar` rc 0 (`1746` rules); `SAMPLE-PROFILE-PARITY: … profiles=4 samples=60 checks=145 defects=0`; `sv-tolerant-profile: OK` (394 of 580 lists; tolerant-only rules=11); the interpreter on every pin, the 1.0.250 grammar against the candidate: `PRESCREEN-PINS: jobs=1938 verdict_moved=2 ast_moved=0` — the two 2023 controls under `sv_2023` and nothing else; `PRESCREEN-CORPUS: files=30 arms=90`, no verdict and no stop moved under any profile (the raw reading; the effect is the pipeline's); friscv's lane expansion `accepted=false` under HEAD on all three SV profiles, `accepted=true` under `sv_2023` and the tolerant profile on the new grammar, `sv_2017` unchanged.

**Measured on the final bytes.** `ADJUDICATION-REPROS: checked=1938 armed=236 listed=946 multi_profile_rows=733 preprocess_rows=38 tolerated_rows=8 superset_rows=734 failures=0`; the three raw lanes `LANE-MOVERS lane=sv_2017 rows_before=16336 rows_after=16336 added=0 lost=0 pass_to_fail=0 fail_to_pass=0 other=0`; `LANE-MOVERS lane=sv_2023 rows_before=16336 rows_after=16336 added=0 lost=0 pass_to_fail=0 fail_to_pass=0 other=0`; `LANE-MOVERS lane=verilog_2005 rows_before=2606 rows_after=2606 added=0 lost=0 pass_to_fail=0 fail_to_pass=0 other=0`; AST identity `AST-IDENTITY-SWEEP: profile=sv_2017 files=16336 both_accepted=10022 identical=10022 ast_moved=0 verdict_moved=0 both_nonaccepting=6314`, `AST-IDENTITY-SWEEP: profile=sv_2023 files=16336 both_accepted=10048 identical=10048 ast_moved=0 verdict_moved=0 both_nonaccepting=6288`, `AST-IDENTITY-SWEEP: profile=verilog_2005 files=2606 both_accepted=2283 identical=2283 ast_moved=0 verdict_moved=0 both_nonaccepting=323` (controls green 0, red 1); the adjudicator `match=6069 unexplained=0 explained=1416 deferred=8851`, `v2005: match=2345 unexplained=0 explained=190 deferred=71`; `✅ SV recognized cert-coverage union gate passed (canonical UNKNOWN=14, union UNKNOWN=0, residual=[], deterministic across seeds [0,7,42]).`; `✅ verilog_2005 conformance gate passed (lint orphans=0, corpus 369 checks/0 mismatches, aliases 2, cert deterministic across seeds [0,7,42]).`; syntax closure `unreachable_rules: 11`; `[ast_shape_contract][systemverilog] samples=60 aligned=60 drift=0 regression_lock_failures=0 aligned_assertion_failures=0 samples_per_status={"aligned": 60}`; every generated artifact re-derives byte-identically; parse cost: `parse-cost-ratchet: OK (identity fresh for: generated parser, grammar raw ast, instrument, sample inputs; 192 pinned sample files)`; the library suite `test result: ok. 1335 passed; 0 failed; 28 ignored`. The pipeline lane: `PIPELINE-LANE: files=16336 pass=12704 fail=2997 pp_fail=635 timeout=0 crash=0 wall_s=896 probe=rust/target/debug/parseability_probe timeout_s=600 jobs=6 profile=sv_2017`, `PIPELINE-MOVERS[sv_2017] rows_before=16336 rows_after=16336 moved=0 added=0 lost=0`; `PIPELINE-LANE: files=16336 pass=12745 fail=2956 pp_fail=635 timeout=0 crash=0 wall_s=1010 probe=rust/target/debug/parseability_probe timeout_s=600 jobs=6 profile=sv_2023`, `PIPELINE-MOVERS[sv_2023] rows_before=16336 rows_after=16336 moved=1 added=0 lost=0 MOVED (('friscv', 'stimuli/sv/subs/friscv/rtl/friscv_alu.sv'), 'fail', 'pass')` — friscv `rtl/friscv_alu.sv`, the one file named before the measurement; `PIPELINE-LANE: files=16336 pass=12763 fail=2938 pp_fail=635 timeout=0 crash=0 wall_s=1095 probe=rust/target/debug/parseability_probe timeout_s=600 jobs=6 profile=sv_2023_tolerant`, `PIPELINE-MOVERS[sv_2023_tolerant] rows_before=16336 rows_after=16336 moved=1 added=0 lost=0 MOVED (('friscv', 'stimuli/sv/subs/friscv/rtl/friscv_alu.sv'), 'fail', 'pass')`; `PIPELINE-SV2023-CENSUS: rows=16336 both_pass=12704 both_nonpass=3591 2023_only_pass=41 2017_only_pass=0 population_breaches=0 | library: files=407 2023_only_pass=3 2017_only_pass=0 published_2017=277 published_2023=278 | members: unreached_2017=337 reached_2023=51 lost_2023=0`; `PIPELINE-TOLERANT-CENSUS: rows=16336 both_pass=12745 both_nonpass=3573 tolerant_only_pass=18 2023_only_pass=0 population_breaches=0 | library: files=407 tolerant_only_pass=7 2023_only_pass=0 published_2023=278 published_tolerant=283 | members: unreached_2017=337 reached_tolerant=337 lost_tolerant=0`. UMBRELLAS NOT ALL GREEN: sv_formal_exhaustive_closure_gate rc 2, sv_stimuli_quality_gate rc 2; `sv-rule-fire-partition: OK (coverage_gap=52, lr_eliminated_original=2, lr_synthetic=14, unreachable_from_entry=23; 91 never-fired rules, 0 unadjudicated; census rule set = grammar inventory, 1746 rules)`.

**Consumers.** Nothing to change under `sv_2017` or `verilog_2005`. Under `sv_2023` and `sv_2023_tolerant` a file carrying a leading-zero size now parses; the literal's node is the one you already handle, `body` the source text with its leading zeros (the size is the same number).

### 1.0.250 / Contract 1.0.250 — SV-CORPUS-GRAD.45 (`PGEN-SV-CORPUS-GRAD-0431`, 2026-10-05), ledger `SV-0135` (`Released`), **schema 34 unchanged**:
**THE OPT-IN PROFILE `sv_2023_tolerant` GAINS ITS FIFTH SPELLING — a hierarchical name of three or more parts inside `binsof(…)`; the edition profiles unmoved.** `binsof(cfg.vif.sig) intersect {1}` is the reference form of the dotted `cross` item the profile has admitted since `1.0.249`: a select expression has no other way to name an implicitly covered cross item, and every OpenTitan statement that writes it — all eight — names an item of its own cross. The boundary `1.0.249` left was an accident of token shape (a two-part name is `cover_point . bin` by syntax and parsed everywhere; a three-part name did not), written on adjacent lines of one select expression in `core_ibex_pmp_fcov_if.sv`. The standard's text is against the spelling (`SV-CORPUS-GRAD.44`: §19.6.1, *"either a coverage point … or a coverage point bin"*), which is what a member of this list is; the admission is a call the director delegated (2026-10-05, *"yours to make but better be sota, signoff and production-grade"*) and is recorded with its reasons in the decision record. GRAMMAR: ONE rule gated to the tolerant profile, `tolerated_bins_expression_hierarchical`, as `bins_expression`'s third alternative — a two-dot lookahead, then a standard `hierarchical_identifier`; kind `tolerated_hierarchical`, shape `{body: {path}}`, the cross item's own. A two-part name keeps the standard `cover_point` reading under every profile; an expression, a trailing select and a call inside `binsof` stay rejected.

**Measured before a byte was generated.** `--lint-grammar` rc 0 (`1745` rules); `SAMPLE-PROFILE-PARITY: … profiles=4 samples=60 checks=145 defects=0`; `sv-tolerant-profile: OK` (393 of 579 lists; tolerant-only rules=11); the interpreter on every pin, the 1.0.249 grammar against the candidate: `PRESCREEN-PINS: jobs=1881 verdict_moved=2 ast_moved=0` — the two new tolerated pins and nothing else; on the 35 corpus files that use `binsof`, three profiles: `PRESCREEN-CORPUS: files=35 arms=105`, no verdict and no stop position moved (the raw reading; the effect is the pipeline's).

**Measured on the final bytes.** `ADJUDICATION-REPROS: checked=1881 armed=236 listed=919 multi_profile_rows=714 preprocess_rows=16 tolerated_rows=8 superset_rows=713 failures=0`; the three raw lanes `pass_to_fail=0 fail_to_pass=0` (10 022 / 10 048 / 2 283 pass); every file accepted before and after yields a byte-identical typed AST (`identical=10022` / `10048` / `2283`, `ast_moved=0`, `verdict_moved=0`; controls green 0, red 1); the adjudicator `match=6069 unexplained=0`, Verilog-2005 `match=2345 unexplained=0`; the recognized-certificate union passed unchanged (canonical `UNKNOWN=14`, union `UNKNOWN=0`, seeds 0 / 7 / 42) and the Verilog-2005 conformance gate passed; syntax closure `unreachable_rules: 11`; the shape contract `samples=60 aligned=60`; every generated artifact re-derives byte-identically; the binding parse-cost counters are BYTE-IDENTICAL (the rebaseline changed the two identity rows and nothing else; family share 2.634 %); the library suite `1308 passed; 0 failed`. The pipeline lane: `sv_2017` `pass=12743` and `sv_2023` `pass=12783`, `moved=0` under each; `sv_2023_tolerant` `pass=12801` (was 12798), `moved=3` — `sysrst_ctrl_env_pkg.sv`, `core_ibex_fcov_if.sv`, `core_ibex_pmp_fcov_if.sv`, the three files named before the measurement — `PIPELINE-TOLERANT-CENSUS: rows=16336 both_pass=12783 both_nonpass=3535 tolerant_only_pass=18 2023_only_pass=0 population_breaches=0 | library: files=407 tolerant_only_pass=7 2023_only_pass=0 published_2023=278 published_tolerant=283 | members: unreached_2017=498 reached_tolerant=337 lost_tolerant=0`. All eleven family umbrellas green — the regex umbrella after the SV envelope ceiling moved 200 → 201, attributed by the envelope differential to ONE new row, the new rule's rule-head sample (`envelope_div 201 envelope_ceiling 201`); the rule-fire partition re-derived at 1745 rules with 0 unadjudicated.

**What this release does not have.** The tolerant profile's own certificate reads `total=1564 proof=23 witness=1532 UNKNOWN=9`: the new rule is not witnessed by generation — the witness planner renders a one-part name, the rule's two-dot guard routes it to the standard alternative — and neither are the three `solve` rules, for the same reason (`ENGINE-UNIVERSAL-SERVICES.120`, parked). `1.0.249` published a different reason for those three; it was not what the certificate's debug probe shows and is corrected in this release's Json Carrier chapter.

**Consumers.** Nothing to change under `sv_2017`, `sv_2023` or `verilog_2005`. Under `sv_2023_tolerant`, a `bins_expression` may now carry kind `tolerated_hierarchical` — handle it as the `cross_item` of the same kind is handled; the `path` node is the same.

### 1.0.249 / Contract 1.0.249 — SV-CORPUS-GRAD.43 (`PGEN-SV-CORPUS-GRAD-0428`, 2026-10-03), ledger `SV-0134` (`Released`), **schema 34 unchanged**:
**A NEW OPT-IN PROFILE, `sv_2023_tolerant`, BY DIRECTOR DECISION — `sv_2023` plus a closed list of four simulator-tolerated spellings; the edition profiles unmoved.** The four (`foreach (a[i][j])`, `solve … before` directly inside a `foreach` constraint, a dotted name as a `cross` item, a lifetime in a class method prototype) had been ruled tool tolerances by `SV-CORPUS-GRAD.41` — no derivation in either edition, the clause text against each — and pinned `REJECT`. The director ordered the opt-in layer the strict-LRM policy had foreseen. GRAMMAR: the profile rides the existing `@profiles` directive (every list naming `sv_2023` names it; 393 lists), ten rules gated to it alone (`tolerated_*`), each host alternative ONE gated rule so the strict profiles ask nothing for them (parse cost `Δentries 0 Δcommitted 0 Δmemo_hits 0` — a first cut that began each alternative with its keyword had cost `+0.02 %` and was rebuilt). FAMILY COMPONENT: the profile maps to `sv_2023`'s reserved words and standard-library bundle (a gate that could not see a new profile, `SV-KEYWORD-VERSIONS`, now refuses one it cannot map). PINS: six `tolerated` twins (ACCEPT under the tolerant profile alone, through the arm named), four boundary pins, the runner's SUPERSET SWEEP (`superset_rows=711`): `ADJUDICATION-REPROS: checked=1856 armed=231 listed=908 multi_profile_rows=705 preprocess_rows=13 tolerated_rows=6 superset_rows=711 failures=0`. DOCTRINE 46 `SV-TOLERANT-PROFILE-SUPERSET`. MEASURED: three raw lanes `pass_to_fail=0 fail_to_pass=0`; AST identity `identical=10022 / 10048 / 2283, ast_moved=0`; shape 60 / 60; closure 11; both certificates green; the tolerant READING of the pipeline lane `PIPELINE-LANE: files=16336 pass=12797 fail=3268 pp_fail=271 … profile=sv_2023_tolerant` (sv_2023: `pass=12782`), its census `PIPELINE-TOLERANT-CENSUS: rows=16336 both_pass=12782 both_nonpass=3539 tolerant_only_pass=15 2023_only_pass=0 population_breaches=0 | library: files=407 tolerant_only_pass=6 2023_only_pass=0 published_2023=278 published_tolerant=282 | members: unreached_2017=498 reached_tolerant=315 lost_tolerant=0`. Each tolerated spelling has its own AST `kind`; the list is closed; still rejected: `solve` under an `if`, a lifetime in a modport or interface-class prototype, an expression as a cross item, a dotted name inside `binsof(…)` (`SV-CORPUS-GRAD.44`, the next ruling). See *The Json Carrier* → *The Opt-In Tolerance Profile*.

### 1.0.248 / Contract 1.0.248 — SV-CORPUS-GRAD.39 (`PGEN-SV-CORPUS-GRAD-0425`, 2026-10-02), ledger `SV-0133` (`Released`), **schema 34 unchanged**:

⭐ **A `local::`-qualified name can be a method call's receiver.** `grammars/systemverilog.ebnf`: `split_callable_receiver_scope` — the rendering of A.8.4's scope prefix a callable receiver reaches — gains the `[ local :: ]` half of `class_qualifier`. IEEE 1800-2017 §18.7.1's `local::` inside `randomize() with { … }` followed by a method call (`q.size() == local::q.size()`, OpenTitan's spi and i2c agent sequences) was rejected at the `(` through `1.0.247` while `local::x`, `local::q[i]`, `local::q.size` and `local::g()` all parsed; A.8.2 `method_call_root ::= primary` and A.8.4 `primary ::= [ class_qualifier \| package_scope ] hierarchical_identifier select` derive it in both editions — no deviation. Typed shape: the receiver's `scope` carries `{kind: "local_qualifier", scope: …}` where a package or class scope already sat. Housekeeping in the same revision: the fifteen declarative separator `@sample`s `ENGINE-UNIVERSAL-SERVICES.98` made redundant are removed (stimuli only; the generated parser moves because an annotated token is emitted with its scaffolding, the language does not).

**Measured on the final bytes.** `ADJUDICATION-REPROS: checked=1820 armed=225 listed=891 multi_profile_rows=694 preprocess_rows=13 failures=0` (three pins filed); the three raw lanes `pass_to_fail=0 fail_to_pass=0`; every file accepted before and after yields a byte-identical typed AST except ONE — verilator `t_randomize_method_with_scoping.v`, whose `local::e.c2_f` member access now renders through the split callable receiver exactly as a package-qualified `pkg::e.c2_f` already did (`identical=10021` / `10047` / `2283`, `ast_moved=1` / `1` / `0`, `verdict_moved=0`); under the preprocess-then-parse pipeline one package moves `fail` → `pass` (`i2c_agent_pkg.sv`) and nothing else; the union certificate reads `sample_parse_failures=0`, union `UNKNOWN=0` with the fifteen samples gone; all eleven family umbrellas green; all 11 generated artifacts re-derive byte-identically; the library suite `1302 passed; 0 failed`. **Typed-shape note for consumers of `local::` names:** a `local::`-qualified dotted name (`local::a.b`, with or without a call) is now a `split_direct_callable_method` node — receiver `{scope: {kind: "local_qualifier"}, name: a}`, body `b` — as a package-qualified dotted name already was on `1.0.247` (the control measured on both parsers); a bare `local::x` and an indexed `local::q[i]` are unchanged.

**Consumers.** Nothing to change: an input that parsed before parses identically (the controls' typed ASTs are byte-identical across the change); the newly accepted form yields the receiver shape the split callable rule always declared, with a `local_qualifier` scope object.

### 1.0.247 / Contract 1.0.247 — SV-CORPUS-GRAD.37 (`PGEN-SV-CORPUS-GRAD-0423`, 2026-10-01), ledger `SV-0132` (`Released`), **schema 34 unchanged**:

⭐ **UVM's `const local T x = new(…)` is admitted as a documented tool extension.** `grammars/systemverilog.ebnf`: `class_property`'s `const`-first arm reads `( assign ( class_new | constant_expression ) )?`, `class_new` first. Annex A A.1.9 gives that arm only a `constant_expression`, and `new(…)` is none — yet the reference library writes it (`uvm_transaction.svh:443`), every simulator accepts it, and §8.19's prose calls it a constant with an initial value; PGEN accepted it through `1.0.218` only because `local` lexed as an identifier, and rejected it from the Annex B reserved-word fix (`1.0.219`, 2026-09-04) until now — `uvm_pkg.sv`, and everything built on UVM, with it. The director's 2026-08-10 bounding ruling (the operative test first: a change must not make the parser reject what the ecosystem writes) decides the open `SVPP-EXPANSION.5`(d) in favour of the arm. Typed shape unchanged: `{kind: "const", qualifiers, data_type, name, init}` with `init` the `class_new` object; the `local const …` order already parsed through `data_declaration`.

**Measured on the final bytes.** `ADJUDICATION-REPROS: checked=1814 armed=225 listed=888 multi_profile_rows=691 preprocess_rows=13 failures=0` (three pins filed: the UVM spelling ACCEPT, the `local const` order ACCEPT as before, an empty initializer REJECT); the three raw lanes `pass_to_fail=0 fail_to_pass=0`, and every file accepted before and after yields a byte-identical typed AST (`identical=10022` / `10048` / `2283`, `ast_moved=0 verdict_moved=0`); under the preprocess-then-parse pipeline five UVM units move `fail` → `pass` (`uvm.sv`, `uvm_pkg.sv` and three files that include them) and nothing else moves; the external-corpus proof surface reads `parse_pass_total: 14` / `parse_fail_total: 0`; both certificates and the syntax closure green; all 11 generated artifacts re-derive byte-identically; the library suite `1301 passed; 0 failed`.

**Consumers.** Nothing to change: an input that parsed before parses identically (the controls' typed ASTs are byte-identical across the change); the newly accepted form yields the shape the `const` arm always declared.

### 1.0.246 / Contract 1.0.246 — ENGINE-UNIVERSAL-SERVICES.111 + SV-CORPUS-GRAD.33 (`PGEN-ENGINE-UNIVERSAL-SERVICES-0163`, 2026-09-28), ledger `SV-0131` (`Released`), **schema 34 unchanged**:

⭐ **The store write epoch names CONTENT, not time — and the wildcard escapes reach module scope.** `rust/src/ast_pipeline/semantic_runtime.rs`: every store mutation takes a fresh epoch from a monotone allocator; a discarding rollback RESTORES the checkpoint's epoch, because facts, arena and chain are the checkpoint's again — so a store-tainted memoized success stamped before a speculative branch is answered again after the branch rolls back (through `1.0.245` the rollback bumped the counter and evicted it; `.28`'s first candidate re-parsed an instantiation head's parameter list on the second route, entries +5.18 %). Measured alone on the pinned 192-file sample: entries −0.05 %, memo_hits −0.07 %, committed flat; the generated parser byte-identical; the semantic suite 387 / 387 with every `sem_memo_*` anchor intact (a mutation still evicts). The grammar then admits the `.28` escape at every `class_type` (the `in_scope_kind(class_body)` gate is gone) and the `.31` escape at the type-position prefixes of the data-type, DPI-result, nettype and ps_type sites (the block site keeps `class_scope_plain`: its own `.30` arm already renders the shape, and the shared prefix moved a block-local tree in the first candidate): `import q::*; module m; uvm_tlm_fifo#(int) fifo; C#(int)::width_t x; endmodule` parses on both SV profiles — the two pinned residuals flip to `fixed`. `SV-CORPUS-GRAD.33` rides along: `method_call_receiver_qualifier` and its arm in both `method_call_receiver` twins are deleted (dominated by the split receiver route at every measured site: 0 commits in 10 022 accepted files and 20 shapes; `qualifier: null` keeps the shape). Rules `1734 → 1734` (one prefix twin added, the qualifier rule deleted). Corpus: ZERO rows moved on any lane (sv_2017 10 022 / sv_2023 10 048 / verilog_2005 2 283 pass, unchanged) — the module-scope shapes the release makes legal occur in no corpus file that reaches them; the fix is proven by the two pins; sweeps the corpus AST identity sweeps read ast_moved=0 on every lane — 10 022 / 10 022 / 2 283 accepted trees byte-identical against the 1.0.245 probe, verdict_moved=0; cost on these bytes entries −0.57 % / memo_hits −0.67 % / committed unchanged (7,090,581), both falls promoted by rebaseline — the deleted qualifier probe alone −1.24 M entries; pins the two defect pins flipped to `fixed` (885 rows, `failures=0`).

### 1.0.245 / Contract 1.0.245 — SV-CORPUS-GRAD.32 (`PGEN-SV-CORPUS-GRAD-0414`, 2026-09-23), ledger `SV-0130` (`Released`), **schema 34 unchanged**:

⭐ **The block site's permissive class-type arm is deleted.** IEEE 1800-2017 A.2.2.1 `data_type ::= … | [ class_scope | package_scope ] type_identifier { packed_dimension } | class_type`, A.2.8 `block_item_declaration`, §8.25, §8.23 / §26.3, §6.18 (docs/systemverilog/2017/md/section-41-data-read-api.md). `block_data_type` — the type of a function-body, task-body, constructor or `begin`-`end` local — carried, beside the typedef-excluded `known_unscoped_block_class_type`, a permissive twin `provisional_unscoped_block_class_type` gated on `has_fact(type_name)` alone, kept there since 2026-05-30 "for class-internal scoped types" when the data_type site took the stricter arm (SV-EXH-PROOF.5.2.4.1). Measured on 1.0.244 with `--trace-rules block_data_type`: the class-internal scoped type `C::T x;` and the nested class `outer::inner h;` select branch 9 (`scoped_block_type_identifier`), a declared parameterized class branch 12 (the known arm), a plain typedef branch 10 — and the permissive arm wins only `typedef int T; … begin T #(int) x; end` and `T::P x;`, both illegal for a typedef'd `int` and both refused at the data_type site. The corpus liveness census over the whole `sv_2017` lane read the arm committing in 357 files, in 72 of them without its sibling, every one a bare typedef'd type that branch 10 wins as the earlier arm. From 1.0.245 the rule and its arm are gone (`1735 → 1734` rules; `block_data_type` 18 → 17 branches). Schema 34 is unchanged: the deleted arm passed its child through unannotated, the same `{head, params, scope_chain}` object its sibling renders, and no accepted tree ever passed through it (every sweep `ast_moved=0`). The routed `accepts_invalid` pin flips to `invalid`; the `T::P` shape is pinned invalid; the two shapes the arm was kept for are pinned as identity controls.

**Measured before a byte was generated.** `--lint-grammar` rc 0; `SAMPLE-PROFILE-PARITY: … defects=0`; the liveness census `LIVENESS-CENSUS: profile=sv_2017 files=16336 accepted=10022 rejected=6314 timeout=0 nodump=0 malformed=0 | provisional_unscoped_block_class_type:entered_files=8233,committed_files=357 known_unscoped_block_class_type:entered_files=16336,committed_files=865 scoped_block_class_type:entered_files=8233,committed_files=19 wildcard_escape_block_class_type:entered_files=8233,committed_files=4` (the §1.5e liveness census on the `1.0.244` release probe over the whole `sv_2017` lane, 75 s): the provisional arm committed in 357 files, in 72 of them WITHOUT the typedef-excluded arm (`rust/target/s377/pop_files_32.txt`) — a bare typedef'd type as a block local, where the arm succeeds as a successful-but-losing branch (C3-B) and branch 10 wins the tie; the A/B the 18-file interpreter A/B (HEAD `1.0.244` / the candidate, all three profiles; the routed pin, the three new pins, the `.30` block-local pins and controls, the `.28` member control and five probes; `rust/target/s377/ab_1.txt`): 54 rows — 48 identity (the class-internal scoped type `C::T x;`, the nested class `outer::inner h;`, the declared parameterized class `C #(int) x;`, the plain typedef `T x;`, every `.30` block-local pin, every tree byte-identical), 6 LOSS accept → reject (EXACTLY the routed pin `accepts_invalid_block_typedef_parameterized`, the new `invalid_block_typedef_scope_chain` and its probe twin, × two SV profiles), ZERO movers, ZERO `AST-MOVED`; the pins pre-screen `PRESCREEN-PINS: jobs=1770 verdict_moved=4 ast_moved=0 in 4597s` (the 865-row manifest, two workers) — EXACTLY the routed `accepts_invalid_block_typedef_parameterized` and the new `invalid_block_typedef_scope_chain` accept → reject under both SV profiles (`head=True new=False` × 4), ZERO trees moved across every pin — every block-local, class-scope, wildcard-escape and typedef pin included; as pre-registered; the population pre-screen `PRESCREEN-CORPUS: files=72 arms=144 in 561s` (the 72 files where the deleted arm committed without its sibling — two workers, both SV profiles): ZERO verdict movers either way (`sv_2017`: 71 both-accept, 1 both-reject; `sv_2023`: 71 / 1), no furthest-moved file — as pre-registered: branch 10 had already won every tree the arm committed in.

**Measured on the final bytes.** `ADJUDICATION-REPROS: checked=1770 armed=225 listed=865 multi_profile_rows=672 preprocess_rows=13 failures=0` (865 rows: the routed `accepts_invalid_block_typedef_parameterized.sv` flipped to `invalid` / REJECT — the RED run before the flip read EXACTLY its two arms `expect=ACCEPT got=REJECT` (`checked=1770 … failures=2`) and nothing else — `invalid_block_typedef_scope_chain.sv` REJECT, the two identity controls ACCEPT, every other pin as keyed); `LANE-MOVERS lane=sv_2017 rows_before=16336 rows_after=16336 added=0 lost=0 pass_to_fail=0 fail_to_pass=0 other=0`; `LANE-MOVERS lane=sv_2023 rows_before=16336 rows_after=16336 added=0 lost=0 pass_to_fail=0 fail_to_pass=0 other=0`; `LANE-MOVERS lane=verilog_2005 rows_before=2606 rows_after=2606 added=0 lost=0 pass_to_fail=0 fail_to_pass=0 other=0` — ZERO rows moved on any lane (`LANE-TOTALS: lane=sv_2017 rows=16336 pass=10022 fail=6314 timeout=0`; `LANE-TOTALS: lane=sv_2023 rows=16336 pass=10048 fail=6288 timeout=0`; `LANE-TOTALS: lane=verilog_2005 rows=2606 pass=2283 fail=323 timeout=0` — unchanged from `1.0.244`), as pre-registered: the 72 files where the deleted arm committed alone were bare typedef'd types branch 10 already won; `timeout=0` on every lane; `AST-IDENTITY-SWEEP: profile=sv_2017 files=16336 both_accepted=10022 identical=10022 ast_moved=0 verdict_moved=0 both_nonaccepting=6314`; `AST-IDENTITY-SWEEP: profile=sv_2023 files=16336 both_accepted=10048 identical=10048 ast_moved=0 verdict_moved=0 both_nonaccepting=6288`; `AST-IDENTITY-SWEEP: profile=verilog_2005 files=2606 both_accepted=2283 identical=2283 ast_moved=0 verdict_moved=0 both_nonaccepting=323` — every tree both binaries accept byte-identical, no verdict mover anywhere (GREEN control `GREEN control: AST-IDENTITY-SWEEP: profile=sv_2017 files=36 both_accepted=25 identical=25 ast_moved=0 verdict_moved=0 both_nonaccepting=11`; RED control `RED control: AST-IDENTITY-SWEEP: profile=sv_2017 files=1 both_accepted=1 identical=0 ast_moved=1 verdict_moved=0 both_nonaccepting=0`); `TWO-SIDED sv_2017: ast_moved=0 verdict_moved=0 predicted_verdict_movers=0 predicted_but_unmoved=0 moved_but_not_predicted=0`; `TWO-SIDED sv_2023: ast_moved=0 verdict_moved=0 predicted_verdict_movers=0 predicted_but_unmoved=0 moved_but_not_predicted=0`; `TWO-SIDED verilog_2005: ast_moved=0 verdict_moved=0 predicted_verdict_movers=0 predicted_but_unmoved=0 moved_but_not_predicted=0` — predicted = moved = 0 on every lane, nothing unpredicted; `match=6069 unexplained=0 explained=1416 deferred=8851`; `sv2023: match=6061 unexplained=0 explained=1428 deferred=8847 edition_contrasts=25 explicit_2023_readings=12 inherited_reject_reread_owed=54+240 carried_over=49+72`; `v2005: match=2345 unexplained=0 explained=190 deferred=71` — every count unchanged from `1.0.244` (no corpus row moved); the `sv_2017` × `sv_2023` census `SV2023-LANE-CENSUS: rows=16336 both_pass=10022 both_nonpass=6288 2023_only_pass=26 2017_only_pass=0 population_breaches=0` unchanged; production parity rc 0; the dark worklist rc 0 — `RESIDUAL_ROWS.tsv` unchanged (15 rows; no row newly RESIDUAL, none re-keyed); `MACRO-BLOCKED 28 / INVALID-SV 14 / CROSS-FILE-FACTS 5 / FRAGMENT-MODULE 2 / FRAGMENT-ENUM-BODY 1, TOTAL 50` — `PARSER-DEFECT 0`, every count unchanged from `1.0.244`; verdict coverage `7485/2606/6245/3864/0`, under the pipeline `10296/2606/3434/2150/0`, unchanged; closure rc 0 (`unreachable_rules: 1`, `unreachable_branches: 2` — the pre-existing set, unchanged; the contract re-stamped on the new digest with `defined_rule_count=1702 reachable_rules=1733` — the one deleted rule gone from both counts, nothing newly unreachable; the five family umbrellas read EXACTLY `1.0.244`'s pre-existing `.24` conditions and nothing new — (a) the EBNF dual-run divergence `systemverilog` 211 against the ceiling 155, unchanged (its row's rule count 1703 → 1702); (b) the preprocessor syntax-closure contract carrying no identity block, refusing `sv_formal_exhaustive_closure_gate`, `sv_preprocessor_formal_exhaustive_closure_gate` and `sv_preprocessor_syntax_closure_gate` alike; (c) the VHDL realistic corpus' observed pass total 14 against the contract's 13 — while the annotation-differential, return and stimuli-module parity gates, the `rtl_const_expr` certificate gate, the parse-harness equivalence and semantic gates and the preprocessor reachability closure are GREEN); union canonical total=1530 proof=13 witness=1502 unknown=15 / union witness=1517 unknown=0, residual `[]` (was 1531 / 13 / 1503 / 15 and 1518 / 0: −1 = the deleted rule, witnessed; UNKNOWN unchanged, deterministic across seeds 0/7/42); `verilog_2005` cert 1245 / 397 / 833 / 15 (was 1246 / 398 / 833 / 15: −1 = the deleted rule, declared under every profile and counted as a proof rule there — the declared-set reading the pre-registration used after `.13c.2m`; UNKNOWN unchanged) — both contracts adopted from the gates' printed lines (the first runs RED on `canonical total=1530 (expected 1531)` and `cert total=1245 (expected 1246)`) and the gates re-run GREEN (`✅ SV recognized cert-coverage union gate passed (canonical UNKNOWN=15, union UNKNOWN=0, residual=[], deterministic across seeds [0,7,42]).`; `✅ verilog_2005 conformance gate passed (lint orphans=0, corpus 369 checks/0 mismatches, aliases 2, cert deterministic across seeds [0,7,42]).`); AST shape contract gate `test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 1217 filtered out; finished in 0.20s` (`[ast_shape_contract][systemverilog] samples=57 aligned=57 drift=0 regression_lock_failures=0 aligned_assertion_failures=0`; schema 34 unchanged); reproducibility ✓; `✅ parse-cost ratchet holds` after `REBASELINED` — a FALL on every binding counter, promoted deliberately: `entries 409,391,350 → 409,344,527 (−0.01 %)`, `committed 7,090,713 → 7,090,581 (−132)`, `memo_hits 175,748,478 → 175,726,222 (−0.01 %)` (the pre-read `cost_ratchet_pre` read each as `an improvement — promote it deliberately`, then the rebaseline: `parse-cost-ratchet: OK (identity fresh for: generated parser, grammar raw ast, instrument, sample inputs; 192 pinned sample files)` / `✅ parse-cost baseline promoted — review the diff before committing.` (the promoted baseline `409,344,527` / `7,090,581` / `175,726,222`), and the final check `✅ parse-cost ratchet holds.` on it); per rule, EXACTLY six rules fell and none rose: `provisional_unscoped_block_class_type` −8,516 entries / −22 commits (GONE — the arm derived at every block-item type under longest_match, its 22 successful-but-losing commits on the sample), its identifier chain `class_identifier` / `declaration_identifier` / `non_keyword_identifier` −8,189 each (−22 commits each) and its memo-served `parameter_value_assignment` / `scope_resolution` asks −6,870 each; the family share `2.688 %` reproduces exactly (26 911 385 of 1 001 092 597 entries over 16 336 files — the corpus total fell by the arm's asks, the rounded share unchanged), no anchor moved; the pipeline lane `PIPELINE-LANE: files=16336 pass=12486 fail=3573 pp_fail=277 timeout=0 crash=0 wall_s=808 probe=rust/target/debug/parseability_probe timeout_s=600 jobs=6` — pass 12 486, fail 3 573, `pp_fail` 277, all unchanged from `1.0.244`, no timeout; `PIPELINE-MOVERS rows_before=16336 rows_after=16336 moved=0 added=0 lost=0`; union `1530 13 1502 1517` (UNKNOWN 15 / 0), `verilog_2005` `1245 397 833 15` — adopted with `adopt_certs.py`; the book marker block regenerated; `✅ verilog_2005 reaches 13 IEEE-1800-only keywords rc=0` (the `.3.67` tripwire unchanged since `1.0.244`; `V2005-KEYWORD-FAITHFULNESS-PROBE: pass=18 fail=0`); `V2005-KEYWORD-FAITHFULNESS: reachable_rules=875 reachable_keywords=139 candidates=13 over_acceptances=0 gated=24 identifier_consumed=6 directive_argument=5 (written)` (`reachable_rules` 876 → 875: the deleted rule, reachable under every profile through the un-gated `block_data_type`; every keyword count unchanged); `LRM-KEYWORD-ENUMERATION: editions=3 productions=109 matched=100 unmatched=9 keyword_members=319 present=319 missing=0 missing_no_terminal=0 incomplete_productions=0 extra_members=8 literal_members=3` unchanged; the release distance `SV-RELEASE-DISTANCE: items=21 (legs_unmet=3 bar_rows=0 [sv_2017=0 sv_2023=0 verilog_2005=0] open_leaves=18 [sv=5 engine=13]) corpus_pass_pct=37.2 known_defects=0 | trailing rate=0.25 items/release over 5 rows, ETA≈84.0 releases ≈ 27.5 days (ESTIMATE)`.

**Consumers.** Nothing to change: a block local whose type is a declared class keeps `known_unscoped_block_class_type`'s node; a typedef'd name with a parameter list or a scope chain, which no legal text carries, is now refused. Bars 0 / 0 / 0 (`divergence:unexplained` on all three adjudication manifests: every bar stays EMPTY).

### 1.0.244 / Contract 1.0.244 — SV-CORPUS-GRAD.13c.2m (`PGEN-SV-CORPUS-GRAD-0413`, 2026-09-23), ledger `SV-0129` (`Released`), **schema 33 → 34**:

⭐ **The LRM-extraction pseudo-keywords are deleted — `class_qualifier` and the footnote superscripts `43` / `48` — and the LRM's `local ::` before a type name is rendered.** IEEE 1800-2017 A.8.4 `primary ::= [ class_qualifier | package_scope ] hierarchical_identifier select`, `class_qualifier ::= [ local :: ] [ implicit_class_handle . | class_scope ]`, A.9.3 `ps_type_identifier ::= [ local :: | package_scope | class_scope ] type_identifier` (footnote 43; 1800-2023 footnote 48), §18.7.1, Annex B (docs/systemverilog/2017/md/section-41-data-read-api.md, docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:3097). `class_qualifier` is an Annex A NONTERMINAL, and the markdown the grammar extractor read prints its production head with ONE colon — `class_qualifier := [ local ::43 ] …` where every other head prints `::=` (one such head per edition, 1 against 717 in 2017 and 1 against 734 in 2023) — so the extractor appended the whole line to `primary`'s `null` alternative, tokenised the footnote superscript, and emitted a literal keyword `class_qualifier` that three scope-prefix copies then carried as their first arm. Through 1.0.243 the parser therefore ACCEPTED `y = class_qualifier m;` (both SV profiles; on an `integer` carrier under `verilog_2005` too), `y = null class_qualifier:=;` and the LRM's own production text `y = null class_qualifier:=local::43;` (under `sv_2017` alone) / `…::48;` (under `sv_2023` alone) — nine pinned arms, none derivable from Annex A — while `local::T'(1)` inside `randomize() with { … }` was REJECTED because the type-scope twins demanded the superscript after `local ::`. From 1.0.244 the three token rules, the four welded `null class_qualifier := …` alternatives and the literal arm of `primary_hier_scope_prefix` / `split_callable_receiver_scope` / `method_call_receiver_qualifier` are gone (`1738 → 1735` rules), `primary_hier_scope_prefix` is SV-gated (the literal was its only `verilog_2005`-satisfiable arm; its hosts read an empty optional there, as on every legal 1364-2005 text), and `ps_type_identifier_scope_sv_2017` / `_sv_2023` render `local ::` as `{kind: "local_qualifier"}` with the package / class scopes passed through byte-identically. ⛔ Three kinds that only the LRM's own production text could produce leave the schema — `"null_class"` (the receiver twins), `"null_class_assign"` (the primary twins), `"class_qualifier"` (the prefix) — a branch removed is a schema bump by this book's rule, although no legal text ever produced any of them; `"local_qualifier"` gains a second producer (the type position). Every previously accepted tree is byte-identical: no corpus file carries the literal or a footnote digit after `::`, and `class_qualifier` / `randomize` as ordinary identifiers parse before and after (pinned). The leaf's two owed measurements land with it: (b) the over-rejection half of `local::` was closed by 1.0.211 at the primary and closes here at the type name; (c) a census of every `kw_<name>` token against the three keyword annexes reads 318 = 248 Annex B keywords + 68 LRM literals + two residual names — this weld and `randomize` (A.8.4's own literal, not reserved, steals no identifier).

**Measured before a byte was generated.** `--lint-grammar` rc 0; `SAMPLE-PROFILE-PARITY: … defects=0`; the A/B the 16-file interpreter A/B (HEAD `1.0.243` / the candidate, all three profiles; the five `accepts_invalid_*` pins, the two one-identifier `invalid` controls, the two `.3.37` `local::` pins, the four new controls and three identifier probes; `rust/target/s376/ab_1.txt`): 48 rows — 35 identity (every control, identifier probe and `.3.37` pin, the trees byte-identical), 9 LOSS accept → reject (EXACTLY the five `accepts_invalid_*` pins on their accepting profiles: `_class_qualifier_as_prefix` × 2, `_lrm_production_text_2017_footnote` under `sv_2017`, `_2023_footnote` under `sv_2023`, `_null_class_qualifier_assign` × 2, `_v2005_class_qualifier` × 3), 4 MOVER reject → accept (the LRM's `local ::` before a TYPE name — `local::T'(1)` and `local::T'{1, 2}` inside a randomize-with constraint — under both SV profiles), ZERO `AST-MOVED`; the two annotated type-scope twins were measured against the unannotated spelling row for row (every non-probe row identical; the cast renders `scope: {kind: "local_qualifier"}`); the pins pre-screen `PRESCREEN-PINS: jobs=1764 verdict_moved=11 ast_moved=0 in 4246s` (the 862-row manifest, two workers) — EXACTLY the seven `accepts_invalid_*` arms accept → reject on the profiles the manifest keys them under (`_class_qualifier_as_prefix` × 2, `_lrm_production_text_2017_footnote` under `sv_2017`, `_2023_footnote` under `sv_2023`, `_null_class_qualifier_assign` × 2, `_v2005_class_qualifier` under `verilog_2005`; `head=True new=False` × 7) and the four arms of the two new `local::T` type controls reject → accept (`head=False new=True` × 4), ZERO trees moved across every pin — every wildcard, class-scope, `local::`, `this.` / `super.` and identifier pin included; as pre-registered (the pre-registration counted 13 by running the `verilog_2005` pin under all three profiles in the A/B; the manifest keys it under one); the population pre-screen `PRESCREEN-CORPUS: files=40 arms=80 in 201s` (the 40 corpus files carrying a `local ::` token — the widened type-scope arm is the only arm that can move a verdict, and no corpus file carries the deleted literal or a footnote digit after `::` — two workers, both SV profiles): ZERO verdict movers either way (`sv_2017`: 33 both-reject, 7 both-accept; `sv_2023`: 32 / 8), no furthest-moved file — as pre-registered.

**Measured on the final bytes.** `ADJUDICATION-REPROS: checked=1764 armed=225 listed=862 multi_profile_rows=669 preprocess_rows=13 failures=0` (862 rows: the five `accepts_invalid_*` pins flipped to `invalid` / REJECT — the RED run before the flip read EXACTLY their seven arms `expect=ACCEPT got=REJECT` (`checked=1764 … failures=7`) and nothing else — the two `local::T` type controls ACCEPT from `1.0.244`, the two identifier controls ACCEPT, every other pin as keyed); `LANE-MOVERS lane=sv_2017 rows_before=16336 rows_after=16336 added=0 lost=0 pass_to_fail=0 fail_to_pass=0 other=0`; `LANE-MOVERS lane=sv_2023 rows_before=16336 rows_after=16336 added=0 lost=0 pass_to_fail=0 fail_to_pass=0 other=0`; `LANE-MOVERS lane=verilog_2005 rows_before=2606 rows_after=2606 added=0 lost=0 pass_to_fail=0 fail_to_pass=0 other=0` — ZERO rows moved on any lane (`LANE-TOTALS: lane=sv_2017 rows=16336 pass=10022 fail=6314 timeout=0`; `LANE-TOTALS: lane=sv_2023 rows=16336 pass=10048 fail=6288 timeout=0`; `LANE-TOTALS: lane=verilog_2005 rows=2606 pass=2283 fail=323 timeout=0` — unchanged from `1.0.243`), as pre-registered: no corpus file carries the deleted literal or a footnote digit after `::`, and none of the 40 `local ::` carriers uses the qualifier before a type name; `timeout=0` on every lane; `AST-IDENTITY-SWEEP: profile=sv_2017 files=16336 both_accepted=10022 identical=10022 ast_moved=0 verdict_moved=0 both_nonaccepting=6314`; `AST-IDENTITY-SWEEP: profile=sv_2023 files=16336 both_accepted=10048 identical=10048 ast_moved=0 verdict_moved=0 both_nonaccepting=6288`; `AST-IDENTITY-SWEEP: profile=verilog_2005 files=2606 both_accepted=2283 identical=2283 ast_moved=0 verdict_moved=0 both_nonaccepting=323` — every tree both binaries accept byte-identical, no verdict mover anywhere (GREEN control `GREEN control: AST-IDENTITY-SWEEP: profile=sv_2017 files=36 both_accepted=25 identical=25 ast_moved=0 verdict_moved=0 both_nonaccepting=11`; RED control `RED control: AST-IDENTITY-SWEEP: profile=sv_2017 files=1 both_accepted=1 identical=0 ast_moved=1 verdict_moved=0 both_nonaccepting=0`); `TWO-SIDED sv_2017: ast_moved=0 verdict_moved=0 predicted_verdict_movers=0 predicted_but_unmoved=0 moved_but_not_predicted=0`; `TWO-SIDED sv_2023: ast_moved=0 verdict_moved=0 predicted_verdict_movers=0 predicted_but_unmoved=0 moved_but_not_predicted=0`; `TWO-SIDED verilog_2005: ast_moved=0 verdict_moved=0 predicted_verdict_movers=0 predicted_but_unmoved=0 moved_but_not_predicted=0` — predicted = moved = 0 on every lane, nothing unpredicted; `match=6069 unexplained=0 explained=1416 deferred=8851`; `sv2023: match=6061 unexplained=0 explained=1428 deferred=8847 edition_contrasts=25 explicit_2023_readings=12 inherited_reject_reread_owed=54+240 carried_over=49+72`; `v2005: match=2345 unexplained=0 explained=190 deferred=71` — every count unchanged from `1.0.243` (no corpus row moved); the `sv_2017` × `sv_2023` census `SV2023-LANE-CENSUS: rows=16336 both_pass=10022 both_nonpass=6288 2023_only_pass=26 2017_only_pass=0 population_breaches=0` unchanged; the dark worklist rc 0 — `RESIDUAL_ROWS.tsv` unchanged (15 rows; no row newly RESIDUAL, none re-keyed); `MACRO-BLOCKED 28 / INVALID-SV 14 / CROSS-FILE-FACTS 5 / FRAGMENT-MODULE 2 / FRAGMENT-ENUM-BODY 1, TOTAL 50` — `PARSER-DEFECT 0`, every count unchanged from `1.0.243` (the dark set moves only when a verdict does, and none did); verdict coverage `7485/2606/6245/3864/0`, under the pipeline `10296/2606/3434/2150/0`, unchanged; closure rc 0 (`unreachable_rules: 1`, `unreachable_branches: 2` — the pre-existing set, unchanged; the contract re-stamped on the new digest with `defined_rule_count=1703 reachable_rules=1734` — the three deleted tokens gone from both counts, nothing newly unreachable; the five family umbrellas read EXACTLY `1.0.243`'s pre-existing `.24` conditions and nothing new — (a) the EBNF dual-run divergence `systemverilog` against the ceiling 155 — 214 → 211, three fewer divergent rows, the three deleted token rules, still over the ceiling; (b) the preprocessor syntax-closure contract carrying no identity block, refusing `sv_formal_exhaustive_closure_gate`, `sv_preprocessor_formal_exhaustive_closure_gate` and `sv_preprocessor_syntax_closure_gate` alike; (c) the VHDL realistic corpus' observed pass total 14 against the contract's 13 — while the annotation-differential, return and stimuli-module parity gates, the `rtl_const_expr` certificate gate, the parse-harness equivalence and semantic gates and the preprocessor reachability closure are GREEN); union canonical total=1531 proof=13 witness=1503 unknown=15 / union witness=1518 unknown=0, residual `[]` (was 1534 / 14 / 1505 / 15 and 1520 / 0: −3 = the three deleted tokens — one had counted as a proof rule, two as witnessed; UNKNOWN unchanged, deterministic across seeds 0/7/42); `verilog_2005` cert 1246 / 398 / 833 / 15 (was 1250 / 399 / 836 / 15: −4 = the literal token, the `43` token reachable through `primary_sv_2017`'s welded arm, `primary_hier_scope_prefix` — SV-gated from `1.0.244` — and the `48` token, an un-gated token rule declared under every profile although only 1800-2023 text ever reached it — the certificate population is the set DECLARED under the profile, which the pre-registration had read as the REACHABLE set (1247 predicted; the declared-set delta of the two grammars is exactly these four); UNKNOWN unchanged) — both contracts adopted from the gates' printed lines (the first runs RED on `canonical total=1531 (expected 1534)` and `cert total=1246 (expected 1250)`) and the gates re-run GREEN (`✅ SV recognized cert-coverage union gate passed (canonical UNKNOWN=15, union UNKNOWN=0, residual=[], deterministic across seeds [0,7,42]).`; `✅ verilog_2005 conformance gate passed (lint orphans=0, corpus 369 checks/0 mismatches, aliases 2, cert deterministic across seeds [0,7,42]).`); AST shape contract gate `test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 1217 filtered out; finished in 0.16s` (`[ast_shape_contract][systemverilog] samples=57 aligned=57 drift=0 regression_lock_failures=0 aligned_assertion_failures=0`; the 57 samples aligned across the bump — none produced a removed kind; schema 33 → 34); reproducibility ✓; `✅ parse-cost ratchet holds` after `REBASELINED` — a FALL, promoted deliberately: `entries 416,213,348 → 409,391,350 (−1.64 %)`, `committed 7,090,713 → 7,090,713` (unchanged: the deleted arms never committed on the sample), `memo_hits 179,159,477 → 175,748,478 (−1.90 %)` (the pre-read `cost_ratchet_pre` read the fall as `an improvement — promote it deliberately`, then the rebaseline: `parse-cost-ratchet: OK (identity fresh for: generated parser, grammar raw ast, instrument, sample inputs; 192 pinned sample files)` / `✅ parse-cost baseline promoted — review the diff before committing.`, and the final check `✅ parse-cost ratchet holds.` on the promoted baseline); per rule, EXACTLY three rules fell and none rose: `kw_class_qualifier_fa08937d` −1,681,044 entries (GONE — the literal regex had been asked at every scope-prefix entry), `kw_null_2be88ca4` −1,729,955 (the welded `null class_qualifier := …` alternatives asked `null` at every primary and receiver evaluation) and `trivia` −3,410,999 entries / −3,410,999 memo hits (the layout skip before each of those asks); the family share `2.648 → 2.688 %` (26 911 385 of 1 001 345 178 entries over 16 336 files: the LR family's own entries unchanged, the corpus total fell), anchors moved; the pipeline lane `PIPELINE-LANE: files=16336 pass=12486 fail=3573 pp_fail=277 timeout=0 crash=0 wall_s=597 probe=rust/target/debug/parseability_probe timeout_s=600 jobs=6` — pass 12 486, fail 3 573, `pp_fail` 277, all unchanged from `1.0.243`, no timeout; `PIPELINE-MOVERS rows_before=16336 rows_after=16336 moved=0 added=0 lost=0`; union `1531 13 1503 1518` (UNKNOWN 15 / 0), `verilog_2005` `1246 398 833 15` — adopted with `adopt_certs.py`; the book marker block regenerated; `V2005-KEYWORD-FAITHFULNESS: reachable_rules=876 reachable_keywords=139 candidates=13 over_acceptances=0 gated=24 identifier_consumed=6 directive_argument=5 (written)` — ⛔ the `.3.67` tripwire MOVED, 14 → 13, and this leaf owes its reason: `class_qualifier` had been counted as an IEEE-1800-only keyword reachable under `verilog_2005` (reached through `primary_hier_scope_prefix` and the welded `null class_qualifier := …` alternative of `primary_sv_2017`, which serves `verilog_2005`) although Annex B never reserved it; with the weld deleted it leaves the roster (the 13: `clocking`, `fs`, `ms`, `new`, `ns`, `null`, `ps`, `randomize`, `static`, `super`, `this`, `unique`, `us`), the `sv_2017` control reads 134 (was 135), its manifest row and witness are retired, and the probe's two arms that used it as their sample keyword read `this` (`V2005-KEYWORD-FAITHFULNESS-PROBE: pass=18 fail=0`; the first run on the final bytes read `pass=14 fail=4` — the tripwire doing its job); `LRM-KEYWORD-ENUMERATION: editions=3 productions=109 matched=100 unmatched=9 keyword_members=319 present=319 missing=0 missing_no_terminal=0 incomplete_productions=0 extra_members=8 literal_members=3` unchanged; the release distance `SV-RELEASE-DISTANCE: items=22 (legs_unmet=3 bar_rows=0 [sv_2017=0 sv_2023=0 verilog_2005=0] open_leaves=19 [sv=6 engine=13]) corpus_pass_pct=37.2 known_defects=0 | trailing rate=0.00 items/release over 5 rows, ETA≈n/a (no net closure over the window) releases ≈ n/a days (ESTIMATE)`.

**Consumers.** A consumer that matched `kind: "null_class"`, `"null_class_assign"` or `"class_qualifier"` can delete the arm: no SystemVerilog text ever produced them. A `ps_type` node whose scope is the LRM's `local ::` now carries `scope: {kind: "local_qualifier"}` (see *The Json Carrier* → `ps_type_identifier_scope_sv_2017` / `_sv_2023`); a package- or class-scoped type keeps its tree. Bars 0 / 0 / 0 (`divergence:unexplained` on all three adjudication manifests: every bar stays EMPTY).

### 1.0.243 / Contract 1.0.243 — SV-CORPUS-GRAD.31 (`PGEN-SV-CORPUS-GRAD-0412`, 2026-09-23), ledger `SV-0128` (`Released`), **schema 33 unchanged**:

⭐ **The wildcard escape at the class scope — the UVM `uvm_config_db#(T)::set(…)` idiom parses under an open wildcard import.** IEEE 1800-2017 A.1.9 `class_scope ::= class_type ::`, A.8.4 `primary ::= [ class_qualifier | package_scope ] hierarchical_identifier select`, A.8.5 `variable_lvalue`, A.8.6 `tf_call`, A.9.3 `ps_or_hierarchical_tf_identifier ::= [ package_scope | class_scope ] tf_identifier` / `ps_parameter_identifier`, §8.24 out-of-block declarations, §18.5.1 extern constraint blocks (docs/systemverilog/2017/md/section-41-data-read-api.md); §26.3 / §26.4 make every declaration of a wildcard-imported package a candidate for the importing scope. 1.0.239 admitted an unresolved name at type positions and 1.0.240 / 1.0.242 an unresolved parameterized class as a type, but every class-scope head — `class_scope_type_head`, reached by `class_scope` from twenty-three sites, and `class_scoped_call_prefix_head` — is fact-gated on a declared class (`🚫 Rule 'known_unscoped_class_scoped_call_class_identifier' rejected by post predicate 'fact_attribute_equals [type_name, uvm_config_db, declaration_family, class]'`), so a static call, a static member, a class-scoped parameter, an enum id, an array through the scope, a scoped `new`, a method-call receiver, and the out-of-block method / constructor / extern constraint of an unresolved parameterized class were refused on both SV profiles through 1.0.242 while the same text with the class declared parsed (found by 1.0.242's design probes; the dark worklist's `CROSS-FILE-FACTS` rows demanding `uvm_config_db` are the corpus carriers). From 1.0.243 `wildcard_escape_class_scope_type` re-spells `class_scope_type` with the 1.0.240 head (import-gated, typedef-excluded, the parameter list CONSUMED — a bare `ident ::` never takes it) and is the LAST alternative of `class_scope` itself under an `ordered` policy — tried only where the known arm fails, which a declared class never lets happen on the same span, so the choice is provably the tie `longest_match` gave the first arm — and `class_scoped_call_prefix` gains an ordered last arm with the same head (`1736 → 1738` rules). ⛔ The first spelling — a passthrough twin `class_scope_escapable` at nineteen sites under `longest_match` — read the lanes clean and was refused by the parse-cost ratchet: entries +1.43 % (the dispatcher entered 3.6 M times on the pinned sample, the escape's asks paid at every class-scope evaluation, including where the known arm had won); costs are rejected, not traded, and the ordered in-rule alternative pays the escape only where the known arm fails. ⛔ The four type-position prefixes (`scoped_type_scope_prefix`, `nettype_scope_prefix`, the two `ps_type_identifier_scope` twins) take `class_scope_plain`, the first alternative alone: at a module item's start `ident #( … )` is also an instantiation head, and an escape that consumes the list there re-parses it on the instantiation route after the rollback (1.0.240's first candidate, entries +5.18 %) — the module-scope `C#(int)::t x;` stays refused, pinned and routed to `ENGINE-UNIVERSAL-SERVICES.111`; inside class bodies and blocks the same type already parses. Under longest_match a declared class ties the escape on the same span at every site and wins by declaration order, so every accepted tree is byte-identical (measured: an escaped and a declared `C#(int)::count` render the same `kind: "class_scope"` node). verilog_2005 untouched by construction.

**Measured before a byte was generated.** `--lint-grammar` rc 0; `SAMPLE-PROFILE-PARITY: … defects=0`; the subtree identity (`rust/target/s375/shape/`); the A/B the 86-file interpreter A/B (HEAD `1.0.242` / the candidate, both SV profiles, `verilog_2005` on the block-local and package files; every wildcard-escape, class-scope and block-typedef pin and 32 probes; `rust/target/s375/ab_1.txt`): 181 rows — 145 identity (116 trees accepted on both sides byte-identical, 58 per SV profile: the declared identity control `control_class_scope_parameterized_declared_identity`, the bare package-scope identity control, every declared `c_*` probe, the `e_type_*` probes the `.28` / `.30` escapes already admit, the instantiation control and every prior wildcard / class-scope pin; 29 refused on both sides: the module-scope type residual and the `.28` module-scope member, every `invalid_*`, the no-import call, the block-local files under `verilog_2005`), 36 movers reject → accept (the three `.31` defect pins, the two shapes controls and the thirteen escaped probes — a call in a continuous assign, a call statement, the factory chain, an lvalue, a primary, a parameter, an enum id, an array scope, the out-of-block method and constructor, an extern constraint, a scoped `new`, a receiver — each under both SV profiles), ZERO accept → reject, ZERO `AST-MOVED`; the pins pre-screen `PRESCREEN-PINS: jobs=1756 verdict_moved=10 ast_moved=0 in 4294s` (the 858-row manifest, two workers, on the ordered in-rule spelling; the twin spelling had read the identical ten arms, `in 4476s`) — EXACTLY the three `.31` defect pins (`defect_wildcard_escape_class_scope_static_call_tf`, `_static_call_block`, `_static_member`) and the two shapes controls (`control_wildcard_escape_class_scope_shapes`, `_out_of_block`) reject → accept under both SV profiles (`head=False new=True` × 10), ZERO trees moved across every pin — the declared identity control, the bare package-scope identity control, every wildcard, nettype, interface-port, `this.` / `super.` and class-scope pin included; as pre-registered; the population pre-screen over the token population `PRESCREEN-CORPUS: files=112 arms=224 in 692s` (the wildcard-import files carrying BOTH a `#(` token and a `)` followed by `::` — every `.31` arm consumes a parameter list and then a scope resolution — two workers, both SV profiles; run on BOTH spellings — the twin at 3 769 MB peak, the ordered in-rule alternative at 4 286 MB — with an IDENTICAL mover set): per SV profile EXACTLY SIX `fail → pass` — Surelog `tests/SplitFile/svaunit_pkg.sv` and opentitan `hw/ip/aes/dv/err_injection_if/fi_cipher_fsm_wrapper.sv`, `fi_control_fsm_wrapper.sv`, `fi_core_wrapper.sv`, `fi_ctr_fsm_wrapper.sv`, `fi_ghash_wrapper.sv` (the three `uvm_config_db` carriers the dark worklist named among them; `sv_2017`: 103 both-reject, 3 both-accept; `sv_2023`: 103 / 3) — ZERO `pass → fail` on either profile, and ONE `FURTHEST-MOVED`: opentitan `err_injection_if/signal_force.sv` `719 -> 2011` on both profiles.

**Measured on the final bytes.** `ADJUDICATION-REPROS: checked=1756 armed=225 listed=858 multi_profile_rows=665 preprocess_rows=13 failures=0` (858 rows: the three `.31` defect pins flipped to `fixed` / ACCEPT on both SV profiles — the RED run on the twin bytes had read exactly those six arms `expect=REJECT got=ACCEPT` — the two shapes controls and two identity controls, one invalid, and the module-scope type residual, REJECT, routed `.111`); `LANE-MOVERS lane=sv_2017 rows_before=16336 rows_after=16336 added=0 lost=0 pass_to_fail=0 fail_to_pass=6 other=0`; `LANE-MOVERS lane=sv_2023 rows_before=16336 rows_after=16336 added=0 lost=0 pass_to_fail=0 fail_to_pass=6 other=0`; `LANE-MOVERS lane=verilog_2005 rows_before=2606 rows_after=2606 added=0 lost=0 pass_to_fail=0 fail_to_pass=0 other=0` — EXACTLY the six pre-registered rows `fail → pass` on `sv_2017` (10 016 → 10 022) and the same six on `sv_2023` (10 042 → 10 048) — Surelog `tests/SplitFile/svaunit_pkg.sv` and opentitan `hw/ip/aes/dv/err_injection_if/fi_cipher_fsm_wrapper.sv`, `fi_control_fsm_wrapper.sv`, `fi_core_wrapper.sv`, `fi_ctr_fsm_wrapper.sv`, `fi_ghash_wrapper.sv` — ZERO the other way, ZERO on `verilog_2005` (2 283), `timeout=0` on every lane; `AST-IDENTITY-SWEEP: profile=sv_2017 files=16336 both_accepted=10016 identical=10016 ast_moved=0 verdict_moved=6 both_nonaccepting=6314`; `AST-IDENTITY-SWEEP: profile=sv_2023 files=16336 both_accepted=10042 identical=10042 ast_moved=0 verdict_moved=6 both_nonaccepting=6288`; `AST-IDENTITY-SWEEP: profile=verilog_2005 files=2606 both_accepted=2283 identical=2283 ast_moved=0 verdict_moved=0 both_nonaccepting=323` — every tree both binaries accept byte-identical, the six verdict movers per SV profile the pre-registered rows (GREEN control `GREEN control: AST-IDENTITY-SWEEP: profile=sv_2017 files=36 both_accepted=25 identical=25 ast_moved=0 verdict_moved=0 both_nonaccepting=11`; RED control `RED control: AST-IDENTITY-SWEEP: profile=sv_2017 files=1 both_accepted=1 identical=0 ast_moved=1 verdict_moved=0 both_nonaccepting=0`); `TWO-SIDED sv_2017: ast_moved=0 verdict_moved=6 predicted_verdict_movers=6 predicted_but_unmoved=0 moved_but_not_predicted=0`; `TWO-SIDED sv_2023: ast_moved=0 verdict_moved=6 predicted_verdict_movers=6 predicted_but_unmoved=0 moved_but_not_predicted=0`; `TWO-SIDED verilog_2005: ast_moved=0 verdict_moved=0 predicted_verdict_movers=0 predicted_but_unmoved=0 moved_but_not_predicted=0` — predicted = moved = 6 on each SV lane, 0 on `verilog_2005`, nothing unpredicted; `match=6069 unexplained=0 explained=1416 deferred=8851`; `sv2023: match=6061 unexplained=0 explained=1428 deferred=8847 edition_contrasts=25 explicit_2023_readings=12 inherited_reject_reread_owed=54+240 carried_over=49+72`; `v2005: match=2345 unexplained=0 explained=190 deferred=71` — every count unchanged from `1.0.242` (the six movers are dark `deferred` rows, so their pass moves no adjudicated count); the dark worklist rc 0 — `RESIDUAL_ROWS.tsv` unchanged (15 rows: none of the movers sat in it, no row newly RESIDUAL, none re-keyed); `MACRO-BLOCKED 28 / INVALID-SV 14 / CROSS-FILE-FACTS 5 / FRAGMENT-MODULE 2 / FRAGMENT-ENUM-BODY 1, TOTAL 50` (was 29 / 14 / 11 / 2 / 1, TOTAL 57) — `PARSER-DEFECT 0`; seven rows left the dark worklist: the six raw-lane movers — the six `CROSS-FILE-FACTS` rows whose ONE demanded name was the class `uvm_config_db` (`NO-ALTERABLE-TICK`: the svaunit package and the five AES error-injection wrappers), each now parsing in full — and opentitan `err_injection_if/signal_force.sv` (the pre-screen's furthest-mover, `MACRO-BLOCKED` on the same demanded class: its stop moved `719 → 2019`, from the class-scope shape to a `` `uvm_fatal `` undefined in the raw text — `PRE-TICK` → `IN-WINDOW`, undecidable from text, off the worklist); the audit `NO-ALTERABLE-TICK` 79 → 73, `PRE-TICK` 2 005 → 2 004, `IN-WINDOW` 1 483 → 1 484, TOTAL 3 576 → 3 570; closure rc 0 (`unreachable_rules: 1`, `unreachable_branches: 2` — the pre-existing set, unchanged; the contract re-stamped on the new digest with `defined_rule_count=1706 reachable_rules=1737` — the two introduced rules reachable; the five family umbrellas read EXACTLY `1.0.242`'s pre-existing `.24` conditions and nothing new — (a) the EBNF dual-run divergence `systemverilog` 214 against the ceiling 155, unchanged (its row's rule count 1704 → 1706); (b) the preprocessor syntax-closure contract carrying no identity block, refusing `sv_formal_exhaustive_closure_gate`, `sv_preprocessor_formal_exhaustive_closure_gate` and `sv_preprocessor_syntax_closure_gate` alike; (c) the VHDL realistic corpus' observed pass total 14 against the contract's 13 — while the annotation-differential, return and stimuli-module parity gates, the `rtl_const_expr` certificate gate, the parse-harness equivalence and semantic gates and the preprocessor reachability closure are GREEN); union canonical total=1534 proof=14 witness=1505 unknown=15 / union witness=1520 unknown=0, residual `[]` (was 1532 / 14 / 1503 / 15 and 1518 / 0: +2 = `wildcard_escape_class_scope_type` and `class_scope_plain` witnessed under both configurations; UNKNOWN unchanged, deterministic across seeds 0/7/42); `verilog_2005` cert unchanged (1250 / 399 / 836 / 15), gate GREEN (`✅ verilog_2005 conformance gate passed (lint orphans=0, corpus 369 checks/0 mismatches, aliases 2, cert deterministic across seeds [0,7,42]).`); the union contract adopted from the gate's printed lines (the first run RED on `canonical total=1534 (expected 1532)`) and the gate re-run GREEN (`✅ SV recognized cert-coverage union gate passed (canonical UNKNOWN=15, union UNKNOWN=0, residual=[], deterministic across seeds [0,7,42]).`); AST shape contract gate `test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 1217 filtered out; finished in 0.17s` (`[ast_shape_contract][systemverilog] samples=57 aligned=57 drift=0 regression_lock_failures=0 aligned_assertion_failures=0`; schema 33 unchanged); reproducibility ✓; `✅ parse-cost ratchet holds` after `REBASELINED`: `entries 410,989,639 → 416,213,348 (+1.27 %)`, `committed +0` (`class_scope_plain` +2, `class_scope` −2: the four type-position sites' commits relabeled), `memo_hits 177,473,031 → 179,159,477 (+0.95 %)`, each `ACCEPTED as attributed by accepted_rises.tsv under reached_files_and_contained_guards (SV-CORPUS-GRAD.31)`, scoped to `wildcard_escape_class_scope_type class_scope_plain class_scope class_scoped_call_prefix` with the `.27` reach marker `package_import_item` — the law RE-DERIVED on the run: `74 of 192 sampled files reached (0 advanced) carry every move outside the introduced sub-graph; on the other 118 the 724 rule(s) not reachable from the 4 introduced name(s) do no more work — entries and committed byte-identical, memo hits not fallen (20 in-scope cells moved).` (the reached subset `parse_cost_ratchet/reached_subsets/SV-CORPUS-GRAD.31/`: 74 of 192 sampled files, the import files, 0 advanced, measured under both probes); per rule: the escape is tried at every class scope the known arm refuses (641,109 on the sample) — its head +1,259,495 entries (702,723 memo hits), the identifier chain +556,772 × 3 (every `non_keyword_identifier` ask a memo hit), the escape rule +641,109, and where an `ident #(` follows in an import file the memo-served parameter list (+417,566, 217,600 memo hits; `hash` / `trivia` / `parameter_value_assignment_sv_2017` +199,966 each); `class_scope_plain` +789,741 and `class_scope_type` +625,596 (all memo hits) are the four type-position sites' attempts relabeled from `class_scope`, which fell by 789,413. ⛔ TWO CHEAPER-LOOKING SPELLINGS WERE MEASURED AND REFUSED: the passthrough twin at nineteen sites under `longest_match` read entries +1.43 % (the dispatcher entered 3.6 M times, the escape's asks paid where the known arm had already won) and the import-gate-first arm read +1.82 % (the gate and its `&( trivia )` lookahead entered 1.3 M / 0.9 M times on an import-heavy sample, and a store-first arm re-asked the call prefix's known heads); the family share `2.678 → 2.648 %` (26 911 288 of 1 016 474 674 entries over 16 336 files), anchors moved; the pipeline lane `PIPELINE-LANE: files=16336 pass=12486 fail=3573 pp_fail=277 timeout=0 crash=0 wall_s=745 probe=rust/target/debug/parseability_probe timeout_s=600 jobs=6` — pass 12 381 → 12 486 (+105), fail 3 678 → 3 573, `pp_fail` 277 unchanged, no timeout; `PIPELINE-MOVERS rows_before=16336 rows_after=16336 moved=105 added=0 lost=0` — EVERY mover `fail → pass` (opentitan 69, sv-tests 35, Surelog 1), ZERO the other way; the movers' old stops, read from their origin lines: 86 stopped ON an `ident#( … )::` class scope in their own text (`uvm_config_db#(T)::get` / `::set`, `push_pull_host_seq#(…)::type_id::create`, `uvm_enum_wrapper#(T)::from_name`), 19 ON a macro invocation whose body carries one — `` `uvm_field_enum `` ×12 (through `` `uvm_resource_enum_read `` to `uvm_enum_wrapper#(TYPE)::from_name`, `uvm_resource_defines.svh:125`), `` `DV_GET_ENUM_PLUSARG `` ×3 (`uvm_enum_wrapper#(ENUM_)::from_name`), `` `DV_ALERT_IF_CONNECT `` ×3 (`uvm_config_db#(virtual alert_esc_if)::set`), `` `DV_ASSERT_CTRL_REQ `` ×1 (`uvm_config_db#(bit)::set`); the six raw-lane movers are among them and the other 99 stay `fail` on the raw lanes — 64 `deferred:chained_only` and 35 `divergence:explained_svpp_include` rows of the `sv_2017` manifest, stopped by an include or a macro chain before the scope the pipeline expands to; union `1534 14 1505 1520` (UNKNOWN 15 / 0), `verilog_2005` unchanged — adopted with `adopt_certs.py`; the book marker block regenerated; `✅ verilog_2005 reaches 14 IEEE-1800-only keywords rc=0` (the `.3.67` tripwire unchanged; `V2005-KEYWORD-FAITHFULNESS-PROBE: pass=18 fail=0`); `V2005-KEYWORD-FAITHFULNESS: reachable_rules=879 reachable_keywords=140 candidates=14 over_acceptances=0 gated=25 identifier_consumed=6 directive_argument=5 (written)`; `LRM-KEYWORD-ENUMERATION: editions=3 productions=109 matched=100 unmatched=9 keyword_members=319 present=319 missing=0 missing_no_terminal=0 incomplete_productions=0 extra_members=8 literal_members=3` — both censuses unchanged from `1.0.242`; the release distance `SV-RELEASE-DISTANCE: items=22 (legs_unmet=3 bar_rows=0 [sv_2017=0 sv_2023=0 verilog_2005=0] open_leaves=19 [sv=6 engine=13]) corpus_pass_pct=37.2 known_defects=0 | trailing rate=-0.25 items/release over 5 rows, ETA≈n/a (no net closure over the window) releases ≈ n/a days (ESTIMATE)`.

**Consumers.** Nothing to change: a `kind: "class_scope"` node whose class is unresolved under an open import is the same `{body: {head: {body}, params, scope_chain}}` object a declared class produces (see *The Json Carrier* → `class_scope_type`, `class_scoped_call_prefix`), and elaboration resolves the name; a declared class keeps its tree. Bars 0 / 0 / 0 (`divergence:unexplained` on all three adjudication manifests: every bar stays EMPTY).

### 1.0.242 / Contract 1.0.242 — SV-CORPUS-GRAD.30 (`PGEN-SV-CORPUS-GRAD-0411`, 2026-09-23), ledger `SV-0127` (`Released`), **schema 33 unchanged**:

⭐ **The parameterized-class wildcard escape at the block item's type — and the escape head stops admitting a name the unit declares as a typedef.** IEEE 1800-2017 A.2.8 `block_item_declaration ::= { attribute_instance } data_declaration`, A.1.9 `class_type ::= ps_class_identifier [ parameter_value_assignment ] { :: class_identifier [ parameter_value_assignment ] }` (docs/systemverilog/2017/md/section-41-data-read-api.md); §26.3 / §26.4 make every declaration of a wildcard-imported package a candidate for the importing scope. 1.0.240 put the escape on `class_type`, which `data_type` reaches (class members, type arguments, class-alias typedefs); a block item's type — a function-body, task-body, constructor, `begin`-`end` or `fork`-`join` local — is read by `block_data_type`, whose class-type route (`scoped_block_class_type` / `known_unscoped_block_class_type` / `provisional_unscoped_block_class_type`) carried no escape: `uvm_tlm_b_target_socket_base #(int) bt;` in a static function under an open `import pkg::*` was refused on both SV profiles through 1.0.241 (Surelog `amiq_eth_ve_top.v` line 10592, the dark row 1.0.241 left RESIDUAL and adjudicated PARSER-DEFECT) — the two unscoped class arms parse the parameter list and are `rejected by post predicate 'has_fact [type_name, …]'`, the bare-name escape of 1.0.239 reads the name alone, and the declarator meets `#(`. From 1.0.242 `wildcard_escape_block_class_type` is the LAST arm of `block_data_type` (`1735 → 1736` rules): the parameter list CONSUMED (a bare unresolved name keeps its 1.0.239 tree), no scope gate — `block_data_type` is reached only through `block_item_declaration`, whose reach (seq / par blocks, tf and constructor bodies) admits no module instantiation, so `ident #(` there is never an instantiation head — and rendering `known_unscoped_block_class_type`'s exact `{head, params, scope_chain}` (measured byte-identical under the interpreter), so a declared parameterized class keeps the known arm by tie order and its tree. ⛔ And a design probe read `import q::*; typedef int T; class c; T #(int) x; endclass` ACCEPTED by 1.0.240 / 1.0.241 through the escape (`🏁 Rule 'class_type' selected branch 2/2`) while the same text without the import is refused: §26.3 gives a local declaration precedence over a wildcard import, so `T` is provably the typedef and `T #(int)` illegal (§8.25). The shared escape head `wildcard_escape_class_type_identifier` now carries `lacks_fact_attribute_equals(type_name, $body, declaration_family, typedef)` — exactly the exclusion `known_unscoped_block_class_type` carries — so every escape site refuses it; a type parameter (which may be a class) is untouched. verilog_2005 untouched by construction (sv-gated; no packages).

**Measured before a byte was generated.** `--lint-grammar` rc 0; `SAMPLE-PROFILE-PARITY: … defects=0`; the subtree identity (`rust/target/s374/shape/`, an escaped block local and a declared one render the same `{head: {body: "C"}, params, scope_chain: []}` node); the A/B the 51-file interpreter A/B (HEAD `1.0.241` / the candidate, both SV profiles, `verilog_2005` on the block-local files; every wildcard-escape, parameterized-class-scope and block-typedef pin and 18 probes; `rust/target/s374/ab_1.txt`): 119 rows — 97 identity (64 trees accepted on both sides byte-identical, 32 per SV profile — the declared-parameterized-class block-local identity control and the type-parameter member identity control among them; 33 refused on both sides: the three `.31` static-call / static-member defect pins, the `.28` module-scope member pin, every `invalid_*` wildcard pin, the no-import block-local probe and pin, the no-import typedef probe, and the block-local shapes under `verilog_2005`, where an unnamed `begin`-`end` carries no declarations), 18 movers reject → accept (the two `.30` defect pins, the block-local shapes control, and the constructor / fork-join / named-parameter / scope-chain / task / two-declarator probes, each under both SV profiles), 4 accept → reject (the declared-typedef invalid pin `invalid_wildcard_escape_parameterized_declared_typedef` and its probe twin, both profiles — the ONE pre-registered narrowing), ZERO `AST-MOVED`; the pins pre-screen `PRESCREEN-PINS: jobs=1744 verdict_moved=8 ast_moved=0 in 4689s` (the 852-row manifest, two workers) — EXACTLY the two `.30` defect pins (`defect_wildcard_escape_parameterized_tf_local`, `_block_local`) and the block-local shapes control reject → accept and the declared-typedef invalid pin accept → reject, each under both SV profiles (`head=False new=True` × 6, `head=True new=False` × 2), ZERO trees moved across every pin — every wildcard, nettype, interface-port, `this.` / `super.` and class-scope pin included; the pre-registration counted 6 and had not counted the shapes control, which the A/B had already read as a mover; the population pre-screen over the token population `PRESCREEN-CORPUS: files=1025 arms=2050 in 9865s` (the wildcard-import token population narrowed to the files carrying a `#(` token — neither edit can fire without one — two workers, both SV profiles): per SV profile EXACTLY ONE `fail → pass` — Surelog `tests/SplitFile/amiq_eth_ve_top.v`, which now parses IN FULL under both profiles (`sv_2017`: 782 both-reject, 242 both-accept; `sv_2023`: 781 / 243) — and ZERO `pass → fail` on either profile: the declared-typedef narrowing is absent from the corpus, as pre-registered.

**Measured on the final bytes.** `ADJUDICATION-REPROS: checked=1744 armed=225 listed=852 multi_profile_rows=659 preprocess_rows=13 failures=0` (852 rows: the two `.30` defect pins flipped to `fixed` / ACCEPT on both SV profiles — the RED run before the flip read exactly those four arms `expect=REJECT got=ACCEPT` — the shapes control and two identity controls, two invalids, one `accepts_invalid` (routed `.32`), and the three routed `.31` defect pins — REJECT — with their declared control); `LANE-MOVERS lane=sv_2017 rows_before=16336 rows_after=16336 added=0 lost=0 pass_to_fail=0 fail_to_pass=1 other=0`; `LANE-MOVERS lane=sv_2023 rows_before=16336 rows_after=16336 added=0 lost=0 pass_to_fail=0 fail_to_pass=1 other=0`; `LANE-MOVERS lane=verilog_2005 rows_before=2606 rows_after=2606 added=0 lost=0 pass_to_fail=0 fail_to_pass=0 other=0` — EXACTLY the one pre-registered row `fail → pass` on `sv_2017` (10 015 → 10 016) and the same row on `sv_2023` (10 041 → 10 042) — Surelog `tests/SplitFile/amiq_eth_ve_top.v`, the RESIDUAL dark row, now parsing in full — ZERO the other way, ZERO on `verilog_2005` (2 283), `timeout=0` on every lane; `AST-IDENTITY-SWEEP: profile=sv_2017 files=16336 both_accepted=10015 identical=10015 ast_moved=0 verdict_moved=1 both_nonaccepting=6320`; `AST-IDENTITY-SWEEP: profile=sv_2023 files=16336 both_accepted=10041 identical=10041 ast_moved=0 verdict_moved=1 both_nonaccepting=6294`; `AST-IDENTITY-SWEEP: profile=verilog_2005 files=2606 both_accepted=2283 identical=2283 ast_moved=0 verdict_moved=0 both_nonaccepting=323` — every tree both binaries accept byte-identical, the ONE verdict mover per SV profile the amiq row (GREEN control `GREEN control: AST-IDENTITY-SWEEP: profile=sv_2017 files=36 both_accepted=25 identical=25 ast_moved=0 verdict_moved=0 both_nonaccepting=11`; RED control `RED control: AST-IDENTITY-SWEEP: profile=sv_2017 files=1 both_accepted=1 identical=0 ast_moved=1 verdict_moved=0 both_nonaccepting=0`); `TWO-SIDED sv_2017: ast_moved=0 verdict_moved=1 predicted_verdict_movers=1 predicted_but_unmoved=0 moved_but_not_predicted=0`; `TWO-SIDED sv_2023: ast_moved=0 verdict_moved=1 predicted_verdict_movers=1 predicted_but_unmoved=0 moved_but_not_predicted=0`; `TWO-SIDED verilog_2005: ast_moved=0 verdict_moved=0 predicted_verdict_movers=0 predicted_but_unmoved=0 moved_but_not_predicted=0` — predicted = moved = 1 (the amiq row) on each SV lane, 0 on `verilog_2005`, nothing unpredicted; `match=6069 unexplained=0 explained=1416 deferred=8851`; `sv2023: match=6061 unexplained=0 explained=1428 deferred=8847 edition_contrasts=25 explicit_2023_readings=12 inherited_reject_reread_owed=54+240 carried_over=49+72`; `v2005: match=2345 unexplained=0 explained=190 deferred=71` — every count unchanged from `1.0.241` (the amiq row is a dark `deferred` row, so its pass moves no adjudicated count); the dark worklist rc 0 on the residual table the amiq row LEFT (`RESIDUAL_ROWS.tsv` 16 → 15 rows: the first run on the final bytes read `⛔ adjudicated in RESIDUAL_ROWS.tsv but no longer RESIDUAL — re-read it: ('Surelog', 'tests/SplitFile/amiq_eth_ve_top.v')`, the two-sided reconciliation doing its job; the row removed, the worklist re-run): `MACRO-BLOCKED 29 / INVALID-SV 14 / CROSS-FILE-FACTS 11 / FRAGMENT-MODULE 2 / FRAGMENT-ENUM-BODY 1, TOTAL 57` — `PARSER-DEFECT 0` (was 1), no row newly RESIDUAL; the `CROSS-FILE-FACTS` rows demanding `uvm_config_db` (Surelog `svaunit_pkg.sv`, opentitan `fi_cipher_fsm_wrapper.sv` / `fi_control_fsm_wrapper.sv`) are the `.31` shape's corpus carriers; closure rc 0 (`unreachable_rules: 1`, `unreachable_branches: 2` — the pre-existing set, unchanged; the contract re-stamped on the new digest with `defined_rule_count=1704 reachable_rules=1735` — the one introduced rule reachable; the family umbrellas' pre-existing RED conditions stay routed to `.24`; no new condition); union canonical total=1532 proof=14 witness=1503 unknown=15 / union witness=1518 unknown=0, residual `[]` (was 1531 / 14 / 1502 / 15 and 1517 / 0: +1 = `wildcard_escape_block_class_type` witnessed under both configurations; UNKNOWN unchanged, deterministic across seeds 0/7/42); `verilog_2005` cert unchanged (1250 / 399 / 836 / 15), gate GREEN (`✅ verilog_2005 conformance gate passed (lint orphans=0, corpus 369 checks/0 mismatches, aliases 2, cert deterministic across seeds [0,7,42]).`); the union contract adopted from the gate's printed lines (the first run RED on `canonical total=1532 (expected 1531)`) and the gate re-run GREEN (`✅ SV recognized cert-coverage union gate passed (canonical UNKNOWN=15, union UNKNOWN=0, residual=[], deterministic across seeds [0,7,42]).`); AST shape contract gate `test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 1217 filtered out; finished in 0.16s` (`[ast_shape_contract][systemverilog] samples=57 aligned=57 drift=0 regression_lock_failures=0 aligned_assertion_failures=0`; schema 33 unchanged); reproducibility ✓; `✅ parse-cost ratchet holds` after `REBASELINED`: `entries 410,942,045 → 410,989,639 (+0.01 %)`, `committed +0`, `memo_hits 177,458,513 → 177,473,031 (+0.01 %)`, each `ACCEPTED as attributed by accepted_rises.tsv under contained_in_introduced_subgraph (SV-CORPUS-GRAD.30), scoped to wildcard_escape_block_class_type` — the invariant RE-DERIVED on the run: `6 rule(s) rose (+47,594), 0 fell, 0 escaped the 521-rule sub-graph reachable from wildcard_escape_block_class_type, committed +0` (per rule: the new arm +8,516 entries / 0 commits in the pinned sample, its head `wildcard_escape_class_type_identifier` +8,515, the head's identifier chain `class_identifier` / `declaration_identifier` / `non_keyword_identifier` +8,187 each — every `non_keyword_identifier` ask a memo hit — and `parameter_value_assignment` +6,002, every one a memo replay of the list the two known block class arms had already parsed); the family share `2.679 → 2.678 %` (26 834 684 of 1 001 956 854 entries over 16 336 files), anchors moved; the pipeline lane `PIPELINE-LANE: files=16336 pass=12381 fail=3678 pp_fail=277 timeout=0 crash=0 wall_s=664 probe=rust/target/debug/parseability_probe timeout_s=600 jobs=6` — pass 12 380 → 12 381 (+1: the amiq row), fail 3 679 → 3 678, `pp_fail` 277 unchanged, no timeout; union `1532 14 1503 1518` (UNKNOWN 15 / 0), `verilog_2005` unchanged — adopted with `adopt_certs.py`; the book marker block regenerated; `✅ verilog_2005 reaches 14 IEEE-1800-only keywords rc=0` (the `.3.67` tripwire unchanged); `V2005-KEYWORD-FAITHFULNESS: reachable_rules=879 reachable_keywords=140 candidates=14 over_acceptances=0 gated=25 identifier_consumed=6 directive_argument=5`; `LRM-KEYWORD-ENUMERATION: editions=3 productions=109 matched=100 unmatched=9 keyword_members=319 present=319 missing=0 missing_no_terminal=0 incomplete_productions=0 extra_members=8 literal_members=3` — both unchanged; the release distance `SV-RELEASE-DISTANCE: items=23 (legs_unmet=3 bar_rows=0 [sv_2017=0 sv_2023=0 verilog_2005=0] open_leaves=20 [sv=7 engine=13]) corpus_pass_pct=37.2 known_defects=0 | trailing rate=-0.25 items/release over 5 rows, ETA≈n/a (no net closure over the window) releases ≈ n/a days (ESTIMATE)`.

**Consumers.** Nothing to change: a block-local `block_data` body whose parameterized class is unresolved under an open import is the same bare `{head: {body}, params, scope_chain}` object a declared class produces (see *The Json Carrier* → `block_data_type`, 18 branches), and elaboration resolves the name; a declared class keeps its tree. Bars 0 / 0 / 0 (`divergence:unexplained` on all three adjudication manifests: every bar stays EMPTY); the amiq row PARSES IN FULL on both SV profiles from `1.0.242` — the residual dark row `.27` re-keyed `PARSER-DEFECT` at line 907, `.28` at line 8925 and `.29` at line 10592 leaves `RESIDUAL_ROWS.tsv` (16 → 15 rows) and the dark worklist (`PARSER-DEFECT 0`, TOTAL 57); the lanes read it `fail → pass` on `sv_2017` and `sv_2023`, the pipeline lane too. ⛔ ROUTED + OWNED, both found by this release's design probes: `.31` — the CLASS SCOPE of a static call / static member under an open wildcard import (`uvm_config_db#(int)::set(…)`, the UVM idiom; `🚫 Rule 'known_unscoped_class_scoped_call_class_identifier' rejected by post predicate 'fact_attribute_equals [type_name, uvm_config_db, declaration_family, class]'`) — three defect pins, sequenced NEXT; `.32` — the block site's permissive `provisional_unscoped_block_class_type` admits `typedef int T; … begin T #(int) x; end` with NO import (`accepts_invalid_block_typedef_parameterized.sv`).

### 1.0.241 / Contract 1.0.241 — SV-CORPUS-GRAD.29 (`PGEN-SV-CORPUS-GRAD-0410`, 2026-09-22), ledger `SV-0126` (`Released`), **schema 33 unchanged**:

⭐ **The `implicit_class_handle .` class_qualifier at the primary.** IEEE 1800-2017 A.8.4 `primary ::= [ class_qualifier | package_scope ] hierarchical_identifier select` with `class_qualifier ::= [ local :: ] [ implicit_class_handle . | class_scope ]` (docs/systemverilog/2017/md/section-41-data-read-api.md:3126 / :3140 = IEEE 1800-2023 docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:3083 / :3097): `id = this.arr[0];`, `id = this.q[i];`, `id = super.arr[0];`, `id = this.arr[1:0];`, `id = this.obj.z[0];` are one primary each. Through 1.0.240 `primary_hier_scope_prefix` carried `class_qualifier`, `package_scope`, `class_scope` and `local::` but NOT `implicit_class_handle .`, so a `this.`-rooted expression was read only by `call_primary` — `this.arr` as a paren-less method call — and the sv_2017 primary twin, which carries no trailing select after a call, refused the `[0]` (Surelog `amiq_eth_ve_top.v` line 8925 `m_id = this.x_port_names[m_port_name];`, the dark row 1.0.240 left RESIDUAL and adjudicated PARSER-DEFECT). ⛔ The twins' difference there is NOT a twin defect: 1800-2023 A.8.4 spells `function_subroutine_call [ [ range_expression ] ]` (:3087) where 1800-2017 spells it bare (:3130), which is why the sv_2023 twin accepted the same bytes; copying that select into the sv_2017 twin — the first fix `.28` had offered — would have over-accepted `f(x)[0]` under sv_2017 and was refused. From 1.0.241 the prefix's LAST arm is `implicit_class_handle_prefix_sv_only := implicit_class_handle dot -> {kind: "instance", handle: $1}` (sv-only), the node `instance_or_class_scope` / `variable_lvalue_scope` already render for the same bytes; under longest_match it LOSES every tie to the earlier `call` arm by declaration order — `this.x`, `this.a.b`, `this.a[0].b`, `this.q.size()` keep their `call` trees byte-for-byte — and WINS only where the call arm reads SHORTER, a trailing select. verilog_2005 untouched by construction (`this` is an ordinary identifier there; the helper is sv-gated). One rule (`1734 → 1735`).

**Measured before a byte was generated.** `--lint-grammar` rc 0; `SAMPLE-PROFILE-PARITY: … defects=0`; the A/B the 50-file interpreter A/B (HEAD `1.0.240` / the candidate, both SV profiles, `verilog_2005` on six; every `this.` / `super.` pin — the seven tracked ones and the four this leaf adds — and 36 probes; `rust/target/s373/ab_1.txt`, `ab_2.txt`): 104 rows — 78 identity (66 trees accepted on both sides byte-identical: 44 under `sv_2023`, 21 under `sv_2017`, 1 under `verilog_2005`; 12 refused on both sides: the class probes under `verilog_2005`, `this[0]` on both SV profiles as a probe and as the pin `invalid_this_bare_select`, two older invalid pins), 26 movers reject → accept — every select shape under `sv_2017` (bit, queue index, associative index, part-select, member-then-select, select-member-select, `super.` / `this.super.` roots, the primary as a binary / ternary / condition / argument / concatenation / cast / parenthesised operand, in a constraint block and at module scope; the `.28` defect pin and `control_this_member_select_shapes`) and the two-dimensional select on BOTH profiles (the probe and `control_this_member_two_dim_select`) — ZERO `pass → fail`, ZERO accepted tree moved (`control_this_member_chain_trees`, the tie shapes, byte-identical on both profiles); the pins pre-screen `PRESCREEN-PINS: jobs=1711 verdict_moved=1 ast_moved=0 in 3040s` (the 836-row manifest the job read at its start; the four `.29` pins were measured by the A/B) — EXACTLY the `.28` defect pin `defect_this_member_select_rvalue.sv` under `sv_2017` reject → accept (`head=False new=True`), ZERO trees moved across every pin — the seven tracked `this.` / `super.` pins included — every other arm the same verdict and the same tree; the population pre-screen over the token population `PRESCREEN-CORPUS: files=1602 arms=3204 in 4686s` (every corpus file carrying `this .` / `super .` outside comments and strings, both SV profiles): ZERO `fail → pass` and ZERO `pass → fail` on either profile (`sv_2017`: 1 500 both-reject, 102 both-accept; `sv_2023`: 1 498 / 104) — none of the 22 failing select-shape files has the select as its ONLY stop — and ONE `FURTHEST-MOVED`: Surelog `tests/SplitFile/amiq_eth_ve_top.v` under `sv_2017` `246989 -> 296703`, from line 8925 (`m_id = this.x_port_names[m_port_name];`, now parsed) to line 10592 (`uvm_tlm_b_target_socket_base #(T) bt;`, a parameterized-class-typed LOCAL of a function body — the byte `sv_2023` already stopped at).

**Measured on the final bytes.** `ADJUDICATION-REPROS: checked=1724 armed=225 listed=842 multi_profile_rows=649 preprocess_rows=13 failures=0` (842 rows: the `.28` defect pin flipped to `fixed` / ACCEPT on both SV profiles, the three `.29` controls and one invalid, and the two `.30` defect pins — REJECT — routed); `LANE-MOVERS lane=sv_2017 rows_before=16336 rows_after=16336 added=0 lost=0 pass_to_fail=0 fail_to_pass=0 other=0`; `LANE-MOVERS lane=sv_2023 rows_before=16336 rows_after=16336 added=0 lost=0 pass_to_fail=0 fail_to_pass=0 other=0`; `LANE-MOVERS lane=verilog_2005 rows_before=2606 rows_after=2606 added=0 lost=0 pass_to_fail=0 fail_to_pass=0 other=0` — ZERO rows moved on any lane, as pre-registered by the population pre-screen (`sv_2017` 10 015 pass, `sv_2023` 10 041, `verilog_2005` 2 283 — unchanged; none of the 22 failing select-shape files has the select as its ONLY stop, and the amiq row's stop moved 50 KB further); `AST-IDENTITY-SWEEP: profile=sv_2017 files=16336 both_accepted=10015 identical=10015 ast_moved=0 verdict_moved=0 both_nonaccepting=6321`; `AST-IDENTITY-SWEEP: profile=sv_2023 files=16336 both_accepted=10041 identical=10041 ast_moved=0 verdict_moved=0 both_nonaccepting=6295`; `AST-IDENTITY-SWEEP: profile=verilog_2005 files=2606 both_accepted=2283 identical=2283 ast_moved=0 verdict_moved=0 both_nonaccepting=323` — every tree both binaries accept byte-identical (GREEN control `GREEN control: AST-IDENTITY-SWEEP: profile=sv_2017 files=36 both_accepted=25 identical=25 ast_moved=0 verdict_moved=0 both_nonaccepting=11`; RED control `RED control: AST-IDENTITY-SWEEP: profile=sv_2017 files=1 both_accepted=1 identical=0 ast_moved=1 verdict_moved=0 both_nonaccepting=0`); `TWO-SIDED sv_2017: ast_moved=0 verdict_moved=0 predicted_verdict_movers=0 predicted_but_unmoved=0 moved_but_not_predicted=0`; `TWO-SIDED sv_2023: ast_moved=0 verdict_moved=0 predicted_verdict_movers=0 predicted_but_unmoved=0 moved_but_not_predicted=0`; `TWO-SIDED verilog_2005: ast_moved=0 verdict_moved=0 predicted_verdict_movers=0 predicted_but_unmoved=0 moved_but_not_predicted=0` — predicted = moved = 0, nothing unpredicted, on every lane; `match=6069 unexplained=0 explained=1416 deferred=8851`; `sv2023: match=6061 unexplained=0 explained=1428 deferred=8847 edition_contrasts=25 explicit_2023_readings=12 inherited_reject_reread_owed=54+240 carried_over=49+72`; `v2005: match=2345 unexplained=0 explained=190 deferred=71` — every count unchanged from `1.0.240`; the dark worklist rc 0 on the re-keyed residual table: the amiq row stays `PARSER-DEFECT` (`PARSER-DEFECT 1` in the worklist's summary), now the `.30` finding — its stop moved from line 8925 (`m_id = this.x_port_names[m_port_name];`, parsed) to line 10592 (`uvm_tlm_b_target_socket_base #(T) bt;`, a static function's parameterized-class-typed local); no row newly RESIDUAL; closure rc 0 (`unreachable_rules: 1`, `unreachable_branches: 2` — the pre-existing set, unchanged; the contract re-stamped on the new digest with `defined_rule_count=1703 reachable_rules=1734`; the family umbrellas' pre-existing RED conditions stay routed to `.24`; no new condition); union canonical total=1531 proof=14 witness=1502 unknown=15 / union witness=1517 unknown=0, residual `[]` (was 1530 / 14 / 1501 / 15 and 1516 / 0: +1 = `implicit_class_handle_prefix_sv_only` witnessed under both configurations; UNKNOWN unchanged, deterministic across seeds 0/7/42 — `CERTIFICATE-COVERAGE: grammar='systemverilog' entry='systemverilog_file' samples=40 total=1531 proof=14 witness=1502 UNKNOWN=15 fully_certified=false (sample_parse_failures=0, proof_reverify_failures=0)` on every seed); `verilog_2005` cert unchanged (1250 / 399 / 836 / 15), gate GREEN (`✅ verilog_2005 conformance gate passed (lint orphans=0, corpus 369 checks/0 mismatches, aliases 2, cert deterministic across seeds [0,7,42])`); the union contract adopted from the gate's printed lines and the gate re-run GREEN (`✅ SV recognized cert-coverage union gate passed (canonical UNKNOWN=15, union UNKNOWN=0, residual=[], deterministic across seeds [0,7,42])`); AST shape contract gate `test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 1217 filtered out; finished in 0.31s` (`[ast_shape_contract][systemverilog] samples=57 aligned=57 drift=0 regression_lock_failures=0 aligned_assertion_failures=0`; schema 33 unchanged); reproducibility ✓; `✅ parse-cost ratchet holds` after `REBASELINED`: `entries 409,991,237 → 410,942,045 (+0.23 %)`, `committed +0`, `memo_hits 176,983,107 → 177,458,513 (+0.27 %)`, each `ACCEPTED as attributed by accepted_rises.tsv under contained_in_introduced_subgraph (SV-CORPUS-GRAD.29), scoped to implicit_class_handle_prefix_sv_only` — the invariant RE-DERIVED on the run: `3 rule(s) rose (+950,808), 0 fell, 0 escaped the 8-rule sub-graph reachable from implicit_class_handle_prefix_sv_only, committed +0` (the new rule 475,402 entries / 0 commits in the pinned sample; `implicit_class_handle` +475,402 entries, every one a memo hit; `dot` +4); the family share `2.684 → 2.679 %` (26 785 300 of 999 726 932 entries over 16 336 files), anchors moved. ⛔ The FIRST final-ratchet run REFUSED on three 120 s probe timeouts (the three `pinmux_reg_top.sv` files, the sample's heaviest) while another project's Rust builds held the host at load 40–60 — a busy-host artifact (`never measure on a busy machine`), re-run on the quiet host: holds; the pipeline lane `PIPELINE-LANE: files=16336 pass=12380 fail=3679 pp_fail=277 timeout=0 crash=0 wall_s=590 probe=rust/target/debug/parseability_probe timeout_s=600 jobs=6` — pass 12 380 unchanged, fail 3 679 unchanged, `pp_fail` 277 unchanged, no timeout (⛔ the first run, on the busy host, read `timeout=2` — the two `pinmux_reg_top.sv` files at 600 s — and was RE-RUN on the quiet host); union `1531 14 1502 1517` (UNKNOWN 15 / 0), `verilog_2005` unchanged — adopted with `adopt_certs.py`; the book marker block regenerated; `✅ verilog_2005 reaches 14 IEEE-1800-only keywords rc=0` (the `.3.67` tripwire unchanged; `sv_2017 reaches 135 — the same code, a different reading`); `V2005-KEYWORD-FAITHFULNESS: reachable_rules=879 reachable_keywords=140 candidates=14 over_acceptances=0 gated=25 identifier_consumed=6 directive_argument=5`; `LRM-KEYWORD-ENUMERATION: editions=3 productions=109 matched=100 unmatched=9 keyword_members=319 present=319 missing=0 missing_no_terminal=0 incomplete_productions=0 extra_members=8 literal_members=3` — both unchanged; the release distance `SV-RELEASE-DISTANCE: items=22 (legs_unmet=3 bar_rows=0 [sv_2017=0 sv_2023=0 verilog_2005=0] open_leaves=19 [sv=6 engine=13]) corpus_pass_pct=37.2 known_defects=0 | trailing rate=0.25 items/release over 5 rows, ETA≈88.0 releases ≈ 115.0 days (ESTIMATE)`.

**Consumers.** A `this.`-rooted member chain ending in a select is a `"hierarchical"` primary `{scope: {kind: "instance", handle}, name, select}` under `sv_2017` (no tree existed before); under `sv_2023` the same bytes keep the `"call"` tree with its `select` field, except a multi-dimensional select (`this.m[0][1]`), new on both profiles as the hierarchical primary (the sv_2023 trailing select takes one range). See *The Json Carrier* → `primary_hier_scope_prefix` for the five prefix kinds. Bars 0 / 0 / 0 (`divergence:unexplained` on all three adjudication manifests: every bar stays EMPTY); the amiq row stays RESIDUAL — its stop moved from line 8925 (the `this.member[select]` rvalue, now parsed) to line 10592, `uvm_tlm_b_target_socket_base #(T) bt;`, a parameterized-class-typed LOCAL of a static function under the open wildcard import (the `.28` escape sits on `class_type`, which `data_type` reaches; a block item's type is read by `block_data_type`, whose class-type twins carry no escape): `RESIDUAL_ROWS.tsv` re-keyed `PARSER-DEFECT` → `defect_wildcard_escape_parameterized_tf_local.sv`, routed `.30`; the dark worklist rc 0.

### 1.0.240 / Contract 1.0.240 — SV-CORPUS-GRAD.28 (`PGEN-SV-CORPUS-GRAD-0409`, 2026-09-22), ledger `SV-0125` (`Released`), **schema 33 unchanged**:

⭐ **The wildcard escape for a parameterized class type.** IEEE 1800-2017 A.1.9 `class_type ::= ps_class_identifier [ parameter_value_assignment ] { :: class_identifier [ parameter_value_assignment ] }` (docs/systemverilog/2017/md/section-41-data-read-api.md) is syntactically ANY identifier; §26.3 / §26.4 make every declaration of a wildcard-imported package a candidate for the importing scope, and resolution is elaboration's. `.27` admitted an undeclared bare identifier at every gated type position but not a `class_type` with a `#( … )`, so `typedef uvm_object_registry#(…) type_id;` — the `uvm_object_utils` expansion — and `uvm_tlm_fifo#(int) fifo;` were refused through 1.0.239 (Surelog `amiq_eth_ve_top.v` line 907, the dark row 1.0.239 adjudicated PARSER-DEFECT). An sv-only escape gated on the same `wildcard_import_open` fact now CONSUMES the parameter list at exactly the two sites where `ident #(` can never be a module instantiation — a `class_type` arm inside a class body (a new `class_body` scope the class declaration's `;` opens and `endclass` closes, read by an inline branch predicate), a `param_expression` arm for type arguments (`extends uvm_port_base #(uvm_tlm_if_base #(T,T))`, the UVM idiom; inside `#( … )` there is no instantiation) and the class-alias `typedef` (a passthrough helper — a second `kw_typedef` arm had re-asked the keyword at every typedef in every file, +13 164 entries, caught by the reached law) — so a bare name never takes it and every previously accepted tree is byte-identical (the corpus AST identity sweeps read `ast_moved=0` on every lane (10 013 / 10 039 / 2 283 files both binaries accept, every one identical)). **Why not everywhere.** The first candidate, a lookahead arm at every `class_type`, went through the lanes clean and was refused twice: the union certificate read it UNKNOWN (a positive lookahead is not witnessable — union UNKNOWN would have gone 0 → 1), and the parse-cost ratchet read entries +5.18 % / memo hits +7.67 %, which the per-file per-rule diff placed at instantiation heads under an open import: `m #( … ) u ( … )` parsed its parameter list on the class route, failed at `u (`, rolled back and re-parsed it on the instantiation route — the memo of a branch that rolled back does not survive. A module-scope class-typed member therefore stays refused, pinned and routed `ENGINE-UNIVERSAL-SERVICES.111`. A third candidate put scope markers on every parameter-value list and was refused by the ratchet (entries +16.29 %, memo hits +24.44 %: every scope open advances the store epoch every store-touching memo entry is stamped with, and a parameter list is entered 43 136 times in the pinned sample), by one file going fail → timeout, and by the certificate (a zero-width scope-gated gate rule unknown) — which is why the type-argument site is a static arm and the class-body gate an inline predicate.

**Measured before a byte was generated.** The class-member and typedef shapes reject → accept under the interpreter, the module-member and no-import shapes refused before and after, the scoped / bare identity, all 28 tracked yield probes and every related pin byte-identical; the pins pre-screen `PRESCREEN-PINS: jobs=1711 verdict_moved=4 ast_moved=0 in 2498s` (the shipped spelling, 837 rows) — EXACTLY the two accepting `.28` pins (`defect_wildcard_escape_parameterized_class_type` flipping to `fixed`, `control_wildcard_escape_parameterized_class_member`) × the two SV profiles reject → accept, ZERO trees moved (the class-body scope markers on every class pin included), every other arm the same verdict and the same tree; the first candidate's read `jobs=1706 verdict_moved=6 ast_moved=0` with the module-scope member among its movers; the population pre-screen over the token population `PRESCREEN-CORPUS: files=1631 arms=3262 in 6873s` (the shipped spelling): per SV profile EXACTLY 2 fail → pass — Surelog `tests/SimpleClass1/top.v` (a class declared inside a module with parameterized-class members, then a class header `extends uvm_port_base #(uvm_tlm_if_base #(T,T))`), opentitan `hw/dv/sv/cip_lib/cip_lc_tx_cov_if.sv` (an interface-level `typedef mubi_cov #(.Width(4), …)`) — the same two on both profiles — and ZERO pass → fail; the class-body-only candidate had read ONE (the typedef), which is how the type-argument site was found.

**Measured on the final bytes.** `ADJUDICATION-REPROS: checked=1711 armed=225 listed=836 multi_profile_rows=642 preprocess_rows=13 failures=0` (837 rows: the four `.28` pins — the `.27` defect pin flipped to `fixed`, one control, one invalid, one routed defect — and the three `.29` pins); `LANE-MOVERS lane=sv_2017 rows_before=16336 rows_after=16336 added=0 lost=0 pass_to_fail=0 fail_to_pass=2 other=0`; `LANE-MOVERS lane=sv_2023 rows_before=16336 rows_after=16336 added=0 lost=0 pass_to_fail=0 fail_to_pass=2 other=0`; `LANE-MOVERS lane=verilog_2005 rows_before=2606 rows_after=2606 added=0 lost=0 pass_to_fail=0 fail_to_pass=0 other=0` — EXACTLY the two pre-registered rows `fail → pass` on `sv_2017` (10 013 → 10 015) and the same two on `sv_2023` (10 039 → 10 041) — Surelog `tests/SimpleClass1/top.v`, opentitan `hw/dv/sv/cip_lib/cip_lc_tx_cov_if.sv` — ZERO the other way, ZERO on `verilog_2005`; `AST-IDENTITY-SWEEP: profile=sv_2017 files=36 both_accepted=25 identical=25 ast_moved=0 verdict_moved=0 both_nonaccepting=11`; `AST-IDENTITY-SWEEP: profile=sv_2023 files=16336 both_accepted=10039 identical=10039 ast_moved=0 verdict_moved=2 both_nonaccepting=6295`; `AST-IDENTITY-SWEEP: profile=verilog_2005 files=2606 both_accepted=2283 identical=2283 ast_moved=0 verdict_moved=0 both_nonaccepting=323` — every tree both binaries accept byte-identical (GREEN control `GREEN control: AST-IDENTITY-SWEEP: profile=sv_2017 files=36 both_accepted=25 identical=25 ast_moved=0 verdict_moved=0 both_nonaccepting=11`; RED control `RED control: AST-IDENTITY-SWEEP: profile=sv_2017 files=1 both_accepted=1 identical=0 ast_moved=1 verdict_moved=0 both_nonaccepting=0`); `TWO-SIDED sv_2017: ast_moved=0 verdict_moved=2 predicted_verdict_movers=2 predicted_but_unmoved=0 moved_but_not_predicted=0`; `TWO-SIDED sv_2023: ast_moved=0 verdict_moved=2 predicted_verdict_movers=2 predicted_but_unmoved=0 moved_but_not_predicted=0`; `TWO-SIDED verilog_2005: ast_moved=0 verdict_moved=0 predicted_verdict_movers=0 predicted_but_unmoved=0 moved_but_not_predicted=0` — predicted = moved, nothing unpredicted, on every lane; `match=6069 unexplained=0 explained=1416 deferred=8851`; `sv2023: match=6061 unexplained=0 explained=1428 deferred=8847 edition_contrasts=25 explicit_2023_readings=12 inherited_reject_reread_owed=54+240 carried_over=49+72`; `v2005: match=2345 unexplained=0 explained=190 deferred=71`; the dark worklist rc 0 on the re-keyed residual table (16 rows): the amiq row stays `PARSER-DEFECT`, now the `.29` finding; no row newly RESIDUAL; closure rc 0 (the family umbrellas' pre-existing RED conditions stay routed to `.24`; no new condition); union canonical total=1530 proof=14 witness=1501 unknown=15 / union witness=1516 unknown=0, residual `[]` (was 1529 / 14 / 1500 / 15 and 1515 / 0: +1 = `typedef_class_type` witnessed — the five other introduced rules were witnessed on the previous candidate, 1524 → 1529; UNKNOWN unchanged, deterministic across seeds 0/7/42, sample_parse_failures=0); `verilog_2005` cert unchanged (1250 / 399 / 836 / 15), gate GREEN; both adopted from the gates' printed lines; AST shape contract gate `test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 1217 filtered out; finished in 0.16s` (schema 33 unchanged); reproducibility ✓; `✅ parse-cost ratchet holds` after `REBASELINED`: `entries 408,966,609 → 409,991,237 (+0.25 %)`, `committed 7,090,693 → 7,090,713 (+20: the ten class bodies' two markers)`, `memo_hits 176,742,039 → 176,983,107 (+0.14 %)`, each `ACCEPTED as attributed by accepted_rises.tsv under reached_files_and_contained_guards (SV-CORPUS-GRAD.28)` — the invariant HOLDS: 81 of 192 sampled files reached (0 advanced) carry every move outside the introduced sub-graph; on the other 111 the 719 rule(s) not reachable from the 6 introduced name(s) do no more work — entries and committed byte-id; the family share `2.693 → 2.684 %`, anchors moved; the pipeline lane `PIPELINE-LANE: files=16336 pass=12380 fail=3679 pp_fail=277 timeout=0 crash=0 wall_s=507 probe=rust/target/debug/parseability_probe timeout_s=600 jobs=6` — pass 12 324 → 12 380 (+56: the `uvm_object_utils` expansion parses under the expansion lane wherever a class's registry typedef was its first stop), fail 3 735 → 3 679, `pp_fail` 277 unchanged, no timeout; union `1530 14 1501 1516` (UNKNOWN 15 / 0), `verilog_2005` unchanged — adopted with `adopt_certs.py`; the book marker block regenerated; the share anchors `2.693 → 2.684` moved by their script; `✅ verilog_2005 reaches 14 IEEE-1800-only keywords rc=0` (the `.3.67` tripwire unchanged; `sv_2017 reaches 135 — the same code, a different reading`); the release distance `SV-RELEASE-DISTANCE: items=22 (legs_unmet=3 bar_rows=0 [sv_2017=0 sv_2023=0 verilog_2005=0] open_leaves=19 [sv=6 engine=13]) corpus_pass_pct=37.2 known_defects=0 | trailing rate=0.25 items/release over 5 rows, ETA≈88.0 releases ≈ 104.9 days (ESTIMATE)`.

**Consumers.** Nothing to change. Bars 0 / 0 / 0 (`divergence:unexplained` on all three adjudication manifests: every bar stays EMPTY); the amiq row stays RESIDUAL — its stop moved from line 907 (the parameterized class type, now parsed) to line 8925, `m_id = this.x_port_names[m_port_name];`, a `this.member[select]` RVALUE refused under `sv_2017` only: `RESIDUAL_ROWS.tsv` re-keyed `PARSER-DEFECT` → `defect_this_member_select_rvalue.sv`, routed `.29`; the dark worklist rc 0.

### 1.0.239 / Contract 1.0.239 — SV-CORPUS-GRAD.27 (`PGEN-SV-CORPUS-GRAD-0408`, 2026-09-18; landing `ENGINE-UNIVERSAL-SERVICES.110`), ledger `SV-0124` (`Released`), **schema 33 unchanged**:

⭐ **The wildcard escape reaches type positions.** IEEE 1800-2017 A.1.9 `class_type ::= ps_class_identifier …` and A.2.2.1 `data_type ::= … | [ class_scope | package_scope ] type_identifier { packed_dimension }` are syntactically ANY identifier; §26.3 / §26.4 make every declaration of a wildcard-imported package a candidate for the importing scope, and resolution is elaboration's — the parser cannot prove an identifier undeclared while an unresolved `pkg::*` import is open (SV-PARSE-STRICT.2, sound-not-complete, the nettype precedent). The escape SV-PARSE-STRICT.2 introduced covered nettype identifiers alone; two sv-only escapes gated on the same `wildcard_import_open` fact now sit last in `base_class_type_head` and in the two unscoped type use-sites, each rendering what the known rule renders, so `class test extends uvm_test;`, `uvm_component parent` and a class item's unknown type parse under an open `import uvm_pkg::*` — and nowhere else. No file loses acceptance; every previously accepted tree is byte-identical (the corpus AST identity sweeps read `ast_moved=0` on every lane (9 843 / 9 869 / 2 283 files both binaries accept, every one identical)). **The escape yields where a resolution-free reading already stood:** the first candidate moved two accepted trees and the pins pre-screen caught it before any lane ran (`ast_moved=2`) — `nettype unknown_t x;` under an open import (a nettype alias through 1.0.238, A.2.1.3's second production; a nettype over a data type once the base could be read as a type — the arms tie under `longest_match`) and a direction-less ANSI port `foo_t a` (an interface port through 1.0.238; found by probing every `ident ident` shape the released grammar accepts under an open import, 419 population files carry one). The type route now yields at both sites (`nettype_alias_shape_ahead`, `wildcard_escape_interface_port_shape`: an unresolved bare name followed by an identifier; a declared type is not the shape, a direction leaves no interface reading, `with` is not the alias form), measured back to the released bytes under the interpreter and re-pre-screened (`PRESCREEN-PINS: jobs=1666 verdict_moved=6 ast_moved=0 in 2475s` (the FINAL candidate) — the same three accepting pins × two profiles reject → accept and NOTHING else: the two alias-declaration trees back to HEAD's bytes, every other arm the same verdict and the same tree). Five more pins hold the two sites. **And the escape never steals a name:** the population pre-screen of that candidate read 3 files LOSING acceptance per profile (opentitan `flash_ctrl_phy_cov_if.sv` × 3 copies, a coverpoint label `read_pat :`) — at a `data_type_or_implicit` site the escape had read a LONE unresolved identifier as the type where the implicit reading served it as the NAME, and the engine does not re-open a matched optional or a committed choice; coverpoint labels and `property p(a, b)` lost their verdict, untyped named tf ports (`function f(input a)`) kept it with a moved tree (the name gone) — a class no pin and no verdict count could see. The guard is the grammar's own (`&( declaration_identifier | hash )`, as `data_type_or_implicit_declared` and the function return type already carry), restricted to the escaped reading (`wildcard_escape_lone_name_shape` in `data_type_or_implicit`); a declared or keyword type without a name keeps its reading. Seven probes and the corpus file byte-identical to the released bytes; five more pins (19 in all). The guard's first cut left `::` out of the follower set and refused scoped types at those sites (`PRESCREEN-CORPUS: files=1454 arms=2908 in 5656s` (the SECOND candidate, the lone-name guard without its `::` exclusion): per SV profile 112 fail → pass and 7 pass → fail — every loss and every no-longer-gained file (12) a SCOPED type at an implicit-capable site (`localparam pinmux_pkg::target_cfg_t X = …`, `ref tlul_pkg::tl_h2d_t h2d`), the package name read as the lone-name shape); excluded, gated on the 19 files under the interpreter, and both pre-screens run a third time (`PRESCREEN-PINS: jobs=1686 verdict_moved=10 ast_moved=0 in 2368s` (the FINAL bytes, 824 rows) — EXACTLY the five accepting mover pins × the two SV profiles reject → accept, ZERO trees moved, all 38 `*_wildcard_escape_*` arms as keyed (16 controls accepted, 3 invalids refused on both profiles); `PRESCREEN-CORPUS: files=1454 arms=2908 in 5665s` (the THIRD candidate, over the line-anchored population): per SV profile 124 fail → pass, 0 pass → fail). **What only the lanes could read.** That third candidate was built and run through post chain A: `LANE-MOVERS lane=sv_2017 … pass_to_fail=0 fail_to_pass=170`, `lane=sv_2023 … pass_to_fail=1 fail_to_pass=170`, `lane=verilog_2005 … 0 / 0`; `AST-IDENTITY-SWEEP` `ast_moved=0` on every lane; `TWO-SIDED sv_2017: … predicted_verdict_movers=124 predicted_but_unmoved=0 moved_but_not_predicted=46`, `sv_2023: … moved_but_not_predicted=47` — 46 opentitan movers carry the header-form import `module x import pkg::*;`, which the line-anchored population selector had never seen (re-selected on the construct's token: 1 631 files), and slang `all.sv`, the 1.0.238 row, was refused under sv_2023 at `randsequence(main) main : first second;` — a randsequence production's optional return type, the coverpoint label's theft site under another name. `production_return_type_guarded` guards both randsequence twins with byte-identical rendering, a zero-width gate on the escape's own fact heads the three shapes (their identifier chains had cost +2.87 % committed at every `data_type_or_implicit` entry), two more pins (21), and both pre-screens ran a fourth time over the token population: `PRESCREEN-PINS: jobs=1690 verdict_moved=12 ast_moved=0 in 2380s` (the FINAL bytes, 826 rows) — EXACTLY the six accepting mover pins × the two SV profiles reject → accept (`control_wildcard_escape_extends` / `_class_member` / `_tf_port` / `_directed_port` / `_nettype_with_unresolved` / `_randsequence_typed_production`), ZERO trees moved, all 42 `*_wildcard_escape_*` arms as keyed; `PRESCREEN-CORPUS: files=1631 arms=3262 in 6078s` (the FINAL bytes, over the TOKEN population): per SV profile EXACTLY 170 fail → pass — the same 170 files on both profiles and exactly the lanes' own movers (opentitan 164, Surelog 4, black-parrot 1, verible 1) — and ZERO pass → fail; slang `all.sv` accepted before and after under `sv_2023`. That candidate's lanes read exactly the prediction; the parse-cost ratchet's new invariant (below) then caught its `net_port_header` split running a second arm at every directed port, made `ordered` — provably the same choice — and then a shape that read the import fact before looking for an identifier, tainting the memo of keyword-typed ports (the shapes now open with `&( declaration_identifier )`, provably the same choice); pins pre-screens `PRESCREEN-PINS: jobs=1690 verdict_moved=12 ast_moved=0 in 2144s` (the FINAL bytes after `net_port_header` was made `ordered` — a provably identical choice — 826 rows) — the same six accepting mover pins × the two SV profiles, ZERO trees moved and `PRESCREEN-PINS: jobs=1690 verdict_moved=12 ast_moved=0 in 2214s` (the FINAL bytes — the three shapes opened with `&( declaration_identifier )`, a provably identical choice — 826 rows) — the same six accepting mover pins × the two SV profiles, ZERO trees moved, and the whole post chain re-run on the final bytes.

**Measured before a byte was generated.** 9 probes + the UVM file × 3 profiles under the interpreter: the three escape shapes accept on both SV profiles only, the three no-import shapes stay refused, the two identity controls hold, verilog_2005 untouched; `t_uvm_hello.v`'s stop moves from `uvm_test` onto the class body's `` `uvm_component_utils(test) `` macro use; the whole wildcard-import population (1 454 files × 2 SV profiles): `PRESCREEN-CORPUS: files=1631 arms=3262 in 6078s` (the FINAL bytes, over the TOKEN population): per SV profile EXACTLY 170 fail → pass — the same 170 files on both profiles and exactly the lanes' own movers (opentitan 164, Surelog 4, black-parrot 1, verible 1) — and ZERO pass → fail; slang `all.sv` accepted before and after under `sv_2023`; the pins pre-screen on the first candidate `PRESCREEN-PINS: jobs=1666 verdict_moved=6 ast_moved=2 in 2908s` (the FIRST candidate) — EXACTLY the three new accepting `control_wildcard_escape_extends` / `_class_member` / `_tf_port` pins × the two SV profiles reject → accept; `ast_moved=2` = `control_nettype_alias_wildcard_import_sv2017` / `_sv2023` (the alias declaration rendered through the type arm — the first yield site, fixed before any lane ran), after the yield fix `PRESCREEN-PINS: jobs=1666 verdict_moved=6 ast_moved=0 in 2475s` (the FINAL candidate) — the same three accepting pins × two profiles reject → accept and NOTHING else: the two alias-declaration trees back to HEAD's bytes, every other arm the same verdict and the same tree, after the lone-name guard `PRESCREEN-PINS: jobs=1684 verdict_moved=10 ast_moved=0 in 2464s` (the FINAL candidate, 823 rows) — EXACTLY the five accepting mover pins × the two SV profiles reject → accept (`control_wildcard_escape_extends` / `_class_member` / `_tf_port` / `_directed_port` / `_nettype_with_unresolved`), ZERO trees moved, every one of the 18 new-pin arms accepted (the addendum had predicted 6: it had not counted the two mover pins added with the yield fix), after its `::` exclusion `PRESCREEN-PINS: jobs=1686 verdict_moved=10 ast_moved=0 in 2368s` (the FINAL bytes, 824 rows) — EXACTLY the five accepting mover pins × the two SV profiles reject → accept, ZERO trees moved, all 38 `*_wildcard_escape_*` arms as keyed (16 controls accepted, 3 invalids refused on both profiles), on the final `PRESCREEN-PINS: jobs=1690 verdict_moved=12 ast_moved=0 in 2380s` (the FINAL bytes, 826 rows) — EXACTLY the six accepting mover pins × the two SV profiles reject → accept (`control_wildcard_escape_extends` / `_class_member` / `_tf_port` / `_directed_port` / `_nettype_with_unresolved` / `_randsequence_typed_production`), ZERO trees moved, all 42 `*_wildcard_escape_*` arms as keyed; the population pre-screen of the first candidate `PRESCREEN-CORPUS: files=1454 arms=2908 in 5468s` (the FIRST candidate, before the lone-name guard): per SV profile 124 fail → pass (the same 124 files on both) and 3 pass → fail — opentitan `flash_ctrl_phy_cov_if.sv` × 3 copies, refused at byte 2084 on a coverpoint label `read_pat :`, the lone-name class, of the second `PRESCREEN-CORPUS: files=1454 arms=2908 in 5656s` (the SECOND candidate, the lone-name guard without its `::` exclusion): per SV profile 112 fail → pass and 7 pass → fail — every loss and every no-longer-gained file (12) a SCOPED type at an implicit-capable site (`localparam pinmux_pkg::target_cfg_t X = …`, `ref tlul_pkg::tl_h2d_t h2d`), the package name read as the lone-name shape, of the third `PRESCREEN-CORPUS: files=1454 arms=2908 in 5665s` (the THIRD candidate, over the line-anchored population): per SV profile 124 fail → pass, 0 pass → fail.

**Measured on the final bytes.** `ADJUDICATION-REPROS: checked=1700 armed=225 listed=830 multi_profile_rows=637 preprocess_rows=13 failures=0` (830 rows: the 21 `.27` wildcard-escape pins and the four RESIDUAL-adjudication pins); `LANE-MOVERS lane=sv_2017 rows_before=16336 rows_after=16336 added=0 lost=0 pass_to_fail=0 fail_to_pass=170 other=0`; `LANE-MOVERS lane=sv_2023 rows_before=16336 rows_after=16336 added=0 lost=0 pass_to_fail=0 fail_to_pass=170 other=0`; `LANE-MOVERS lane=verilog_2005 rows_before=2606 rows_after=2606 added=0 lost=0 pass_to_fail=0 fail_to_pass=0 other=0` — EXACTLY the 170 pre-registered rows `fail → pass` on `sv_2017` and the same 170 on `sv_2023` (9 843 → 10 013 and 9 869 → 10 039 passes), ZERO the other way, ZERO on `verilog_2005` — the token population's own prediction, one-to-one; `AST-IDENTITY-SWEEP: profile=sv_2017 files=36 both_accepted=25 identical=25 ast_moved=0 verdict_moved=0`; `AST-IDENTITY-SWEEP: profile=sv_2023 files=16336 both_accepted=9869 identical=9869 ast_moved=0 verdict_moved=170`; `AST-IDENTITY-SWEEP: profile=verilog_2005 files=2606 both_accepted=2283 identical=2283 ast_moved=0 verdict_moved=0` — every tree both binaries accept byte-identical; `TWO-SIDED sv_2017: ast_moved=0 verdict_moved=170 predicted_verdict_movers=170 predicted_but_unmoved=0 moved_but_not_predicted=0`; `TWO-SIDED sv_2023: ast_moved=0 verdict_moved=170 predicted_verdict_movers=170 predicted_but_unmoved=0 moved_but_not_predicted=0`; `TWO-SIDED verilog_2005: ast_moved=0 verdict_moved=0 predicted_verdict_movers=0 predicted_but_unmoved=0 moved_but_not_predicted=0` — predicted = moved, nothing unpredicted, on every lane; `match=6069 unexplained=0 explained=1416 deferred=8851`; `sv2023: match=6061 unexplained=0 explained=1428 deferred=8847 edition_contrasts=25 explicit_2023_readings=12 inherited_reject_reread_owed=54+240 carried_over=49+72`; `v2005: match=2345 unexplained=0 explained=190 deferred=71`; closure rc 0 (the family umbrellas' three pre-existing RED conditions stay routed to `.24`; no new condition); union canonical total=1524 proof=14 witness=1495 unknown=15 / union witness=1510 unknown=0 (was 1517 / 10 / 1492 / 15 and 1507 / 0: +7 = the seven introduced rules, +4 proofs, +3 witnesses, UNKNOWN unchanged — deterministic across seeds 0/7/42, sample_parse_failures=0); `verilog_2005` cert total=1250 proof=399 witness=836 unknown=15 (was 1248 / 397: the two profile-agnostic helpers); both gates GREEN on the adopted tuples; AST shape contract gate `test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 1217 filtered out; finished in 0.16s` (schema 33 unchanged); reproducibility ✓; `✅ parse-cost ratchet holds` after `parse-cost-ratchet: REBASELINED`: `entries 405,715,674 → 408,966,609 (+0.80 %)`, `committed 6,895,857 → 7,090,693 (+2.83 %)`, `memo_hits 175,316,486 → 176,742,039 (+0.81 %)`, each `ACCEPTED as attributed by accepted_rises.tsv under reached_files_and_contained_guards (SV-CORPUS-GRAD.27)` — the invariant HOLDS: 74 of 192 sampled files reached (8 advanced) carry every move outside the introduced sub-graph, the other 118 files' 714 out-of-scope rules do no more work; the family share `2.643 → 2.693 %` (26,455,800 of 982,429,740 entries over 16 336 files), anchors moved; `SV_DECLARED_LR_RULES` 32 unchanged; the pipeline lane `PIPELINE-LANE: files=16336 pass=12324 fail=3735 pp_fail=277 timeout=0 crash=0 wall_s=538 probe=rust/target/debug/parseability_probe timeout_s=600 jobs=6` — pass 12 200 → 12 324 (+124), fail 3 859 → 3 735, `pp_fail` 277 unchanged, the four `.3.67` leading-zero rows still failing (friscv `friscv_alu.sv` among them, the standing [DBINP] candidate); union `1524 14 1495 1510`, `verilog_2005` `1250 399 836 15` — adopted from the gates' printed lines with `adopt_certs.py`; the book marker block and the share anchors (`2.643 → 2.693`) moved by their scripts; `✅ verilog_2005 reaches 14 IEEE-1800-only keywords rc=0` (the `.3.67` tripwire unchanged; `sv_2017 reaches 135 — the same code, a different reading`); the release distance `SV-RELEASE-DISTANCE: items=21 (legs_unmet=3 bar_rows=0 [sv_2017=0 sv_2023=0 verilog_2005=0] open_leaves=18 [sv=6 engine=12]) corpus_pass_pct=37.2 known_defects=0 | trailing rate=0.75 items/release over 5 rows, ETA≈28.0 releases ≈ 10.4 days (ESTIMATE)`.

**Consumers.** Nothing to change. `t_uvm_hello.v` leaves `unexplained_rejects_valid` for `divergence:explained_svpp_macro_use` on both SV manifests — refused ON `` `uvm_component_utils(test) ``, text the preprocessor alters, as pre-registered; the pipeline lane is where its pass is measured; the SV axis-2 bar 2 → 0 rows (`divergence:unexplained` on `adjudication_manifest.tsv`: 0 — the SV axis-2 bar is EMPTY), the `sv_2023` bar 1 → 0 rows (`divergence:unexplained` on `adjudication_manifest_sv2023.tsv`: 0 — EMPTY; `verilog_2005` stays 0).

### 1.0.238 / Contract 1.0.238 — SV-CORPUS-GRAD.26 (`PGEN-SV-CORPUS-GRAD-0407`, 2026-09-17), ledger `SV-0123` (`Released`), **schema 33 unchanged** (an additive `primary` kind):

⭐ **A type comparison is an expression operand on the SV profiles.** IEEE 1800-2017 §6.23 (`docs/systemverilog/2017/md/section-65536-216-bits.md:1612-1613`) = IEEE 1800-2023 §6.23 (`docs/systemverilog/2023/md/section-6-data-types.md:2938-2939`): a type reference "can also be used in equality/inequality and case equality/inequality comparisons with other type references, and such comparisons are considered to be constant expressions"; Annex A lists `type_reference` in `constant_primary` ONLY (footnote 40 / 45 — "illegal … with any operators except the equality/inequality and case equality/inequality operators", `docs/systemverilog/2017/md/section-41-data-read-api.md:3560-3562`, `docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md:3525-3527`) and never in `primary`. Through `1.0.237` `if (type(b) == type(a) && …)` had no derivation on either SV profile while the constant route had one. `primary` now carries a last arm `{kind: "type_comparison", body: {left, op, right}}` — two type references joined by `===`, `!==`, `==` or `!=` — on `sv_2017` / `sv_2023`; `verilog_2005` untouched (`type` is an identifier there). No input an earlier arm accepted reaches the new one, so every accepted tree is byte-identical (the corpus AST identity sweeps read `ast_moved=0` on every lane (9 843 / 9 868 / 2 283 files both binaries accept, every one identical)).

**Measured before a byte was generated.** 40 SV-lane files carry `type (`; with 12 probes, 52 files × 3 profiles under the interpreter: EXACTLY one corpus mover — slang `tests/regression/all.sv` on `sv_2023`, refused at byte 14074 (line 633) on `1.0.237` and accepted on the candidate; the five legal probes accept on both SV profiles, the three illegal ones stay refused, the constant route and the `verilog_2005` call reading render byte-identical trees; verilator `t_type_compare.v` / `t_type_expression_compare.v` (keyed `must_reject`) stay refused with their stop now on `case (type(real))`; the pins pre-screen `PRESCREEN-PINS: jobs=1648 verdict_moved=10 ast_moved=0 in 2528s` — EXACTLY the five new accepting `control_type_comparison_*` pins × the two SV profiles reject → accept, every other arm (the three new invalid pins and the `verilog_2005` call control among them) the same verdict and the same tree.

**Measured on the final bytes.** ADJUDICATION-REPROS: checked=1648 armed=225 listed=805 multi_profile_rows=612 preprocess_rows=13 failures=0 (the first run read failures=1: the `verilog_2005` control was written as `.v` and the runner lists pins by a `*.sv` glob — renamed, a runner limitation worth knowing: a pin's extension is `.sv` whatever its profile); `LANE-TOTALS: lane=sv_2017 rows=16336 pass=9843 fail=6493 timeout=0` (unchanged), `LANE-TOTALS: lane=sv_2023 rows=16336 pass=9869 fail=6467 timeout=0` (was 9868 / 6468), `LANE-TOTALS: lane=verilog_2005 rows=2606 pass=2283 fail=323 timeout=0` (unchanged) — `LANE-MOVERS lane=sv_2017 … pass_to_fail=0 fail_to_pass=0`, `LANE-MOVERS lane=sv_2023 … pass_to_fail=0 fail_to_pass=1` — EXACTLY the pre-registered row, slang `tests/regression/all.sv` (`fail → pass`), `LANE-MOVERS lane=verilog_2005 … pass_to_fail=0 fail_to_pass=0`; `AST-IDENTITY-SWEEP: profile=sv_2017 files=16336 both_accepted=9843 identical=9843 ast_moved=0 verdict_moved=0`, `profile=sv_2023 … both_accepted=9868 identical=9868 ast_moved=0 verdict_moved=1`, `profile=verilog_2005 files=2606 both_accepted=2283 identical=2283 ast_moved=0 verdict_moved=0` — every file both binaries accept renders byte-identically; `TWO-SIDED sv_2017: ast_moved=0 verdict_moved=0 … moved_but_not_predicted=0`, `TWO-SIDED sv_2023: ast_moved=0 verdict_moved=1 predicted_verdict_movers=7 predicted_but_unmoved=6 moved_but_not_predicted=0` (the six predicted-but-unmoved are the six probe files in the prediction table, which are not lane rows; the one lane row predicted moved), `TWO-SIDED verilog_2005: ast_moved=0 verdict_moved=0`; `match=6065 unexplained=1 explained=1419 deferred=8851` (the `sv_2017` lane unchanged); the `sv_2023` lane's `divergence:unexplained` rows `2 → 1` (slang `all.sv` now `match`; `t_uvm_hello.v` remains, `.27`'s); `SV2023-LANE-CENSUS: rows=16336 both_pass=9843 both_nonpass=6467 2023_only_pass=26 2017_only_pass=0 population_breaches=0` (25 → 26); `sv_syntax_closure_gate` GREEN on the allow-list of ONE — `unreachable_rules: 1` (`casting_type`), the baseline STALE (two rules more) and every constraint holding, re-stamped; union GREEN on the gate's own counts `1517/10/1492/15` + `1507/0` at seeds 0 / 7 / 42 (`✅ SV recognized cert-coverage union gate passed (canonical UNKNOWN=15, union UNKNOWN=0, residual=[], deterministic across seeds [0,7,42])` — total `1515 → 1517` and witness `1490 → 1492`, `1505 → 1507`: the two sv-only rules, both witnessed; proof 10, unknown 15 / 0 and the residual `[]` unchanged, `sample_parse_failures=0`); `verilog_2005` conformance GREEN and UNCHANGED at `1248/397/836/15` (the rules are gated out of that profile); `[ast_shape_contract][systemverilog] samples=57 aligned=57 drift=0 regression_lock_failures=0 aligned_assertion_failures=0`; reproducibility ✓; `sv_parse_cost_family_share` `parse-cost: derived family share 2.643 % (23,556,358 of 891,200,417 entries over 16336 files, 0 no-dump)` — the corpus total +1 689 631 (+0.19 %) and the family +1 323, so the share falls 2.648 → 2.643 and every anchor (the gated constant, `cost.md`'s two lines, TOOLBOX, the knowledge card, the doctrine text's two mirrors) is adopted from the artifact; `sv_parse_cost_ratchet` on the pinned sample: `BINDING metric entries ROSE 404,757,886 -> 405,715,674 (+0.24 %)` and `memo_hits ROSE 174,832,885 -> 175,316,486 (+0.28 %)` — attributed per rule against the tracked `rule_costs.tsv`: EXACTLY two rules rose, the introduced `type_comparison_sv_only` (+483,601 entries: one per `primary` that fails its earlier arms) and `type_reference` (+474,187 entries and +474,187 memo hits: entered by the new arm, failing on its first token, memoized), `committed` unchanged, no rule lost entries — ACCEPTED under `contained_in_introduced_subgraph` (the SV-0065 precedent, a `randomize_call` alternative in `primary`, is the same shape) with the invariant re-derived on the run; irreducible under a PEG last arm (a keyword lookahead moves the same count onto the keyword terminal); then `sv_parse_cost_rebaseline` `REBASELINED` and `parse-cost-ratchet: OK (identity fresh for: generated parser, grammar raw ast, instrument, sample inputs; 192 pinned sample files)`; the pipeline lane 16 336 rows on the new debug probe: `PIPELINE-LANE: files=16336 pass=12200 fail=3859 pp_fail=277 timeout=0 crash=0 wall_s=497 …` — every total identical to `1.0.237`'s and `pipeline_results.tsv` BYTE-IDENTICAL to HEAD's (`git diff --quiet`): the pipeline lane parses under `sv_2017`, where nothing moved; `PIPELINE-LANE-CURRENCY: OK`; `sv-corpus-denominator: OK (tuple 7485/2606/6245/4039/1 = adjudicated/routed/no-verdict/dark/bar; under the pipeline 9139/2606/4591/3311/1; …)` — both UNCHANGED (the mover is an `sv_2023`-lane row; the tuples read the `sv_2017` lane); `IDENTITY-CARRIERS: OK (live=14 frozen=7 rows_equal=65 notes=8 not_evaluated=0)`; `corpus-key-integrity: OK`; the `verilog_2005` keyword-faithfulness census `V2005-KEYWORD-FAITHFULNESS: reachable_rules=879 reachable_keywords=140 candidates=14 over_acceptances=0 …` unchanged and the probe `V2005-KEYWORD-FAITHFULNESS-PROBE: pass=18 fail=0`; the release distance `SV-RELEASE-DISTANCE: items=22 (legs_unmet=3 bar_rows=2 [sv_2017=1 sv_2023=1 verilog_2005=0] open_leaves=17 [sv=5 engine=12]) corpus_pass_pct=37.1 known_defects=1 | trailing rate=1.25 items/release over 5 rows, ETA≈17.6 releases ≈ 4.0 days (ESTIMATE)` (the `1.0.238` row recorded; bar rows 3 → 2).

**Consumers.** One additive kind in `primary`. The `sv_2023` bar 2 → 1 (`unexplained_rejects_valid` 2 → 1: slang `all.sv` now matches; `t_uvm_hello.v` remains — the same file as the SV axis-2 bar's one row, `.27`); its other row (`t_uvm_hello.v`, a base class from an unresolved wildcard import) is `.27`'s.

### 1.0.237 / Contract 1.0.237 — SV-CORPUS-GRAD.3.69 + ENGINE-UNIVERSAL-SERVICES.108 (`PGEN-SV-CORPUS-GRAD-0405`, 2026-09-17), ledger `SV-0122` (`Released`), **schema 33 unchanged**:

⭐ **The header-less ANSI default under `verilog_2005` is gated on the latest governing header — and the engine learned to ask which fact came last.** IEEE 1364-2005 A.2.1.2 admits `= expr` only in `output reg` / `output integer|time` lists, and a header-less item is one more identifier of the previous declaration. That is state across siblings, which no lookahead reads and no scope expresses, and the store's existence queries cannot say which header came last; `.3.66` stated it as its residual. The shared runtime now provides `latest_fact_attribute_equals(kind, attribute, value)` — true iff the most recently emitted, still-standing fact of `kind` at any depth up to the current one carries `attribute = value` (37th construct of the semantic-directive suite, interpreter and generated parser byte-identical; a runtime unit test on latest-wins and rollback); the `verilog_2005` header arms re-emit `ansi_port_header{defaults}` at branch start, the list opener resets it, the header-less arm's default is gated on it. Every accepted tree is byte-identical (the corpus AST identity sweeps read `ast_moved=0` on every lane (9 843 / 9 868 / 2 283 files both binaries accept, every one identical)); the SV profiles' store is untouched (every emit gated).

**Measured before a byte was generated.** 17 port probes × 3 profiles under the interpreter: every legal shape holds, the six illegal ones flip under `verilog_2005` only (a default behind `input` / `inout` / a net `output`, behind the LATEST header, on a header-less first item after another module), 11 legal probes render byte-identical trees (11 legal port probes × `verilog_2005` / `sv_2017`: 22/22 arms `ast SAME` against `1.0.236` under the interpreter (`--interpret-parse-ast-json` + `cmp`), 0 DIFF, 0 refused; the 17-probe A/B `PRESCREEN-CORPUS: files=17 arms=51`: 45 arms hold, 6 flip — the six illegal shapes under `verilog_2005` only); the whole `verilog_2005` lane pre-screened (`PRESCREEN-CORPUS: files=2606 arms=2606 in 3621s` — `(true, true)` 2283, `(false, false)` 322, `(true, false)` 1: EXACTLY the pre-registered row (iverilog `ivltests/module_inout_port_list_def.v`), no other file's verdict or stop position moved); the pins pre-screen `PRESCREEN-PINS: jobs=1629 verdict_moved=5 ast_moved=0 in 2669s` — EXACTLY the re-keyed residual pin and the four new `invalid_v2005_ansi_headerless_*` pins under `verilog_2005` accept → reject, every other arm (the `.3.66` port pins among them) the same verdict and the same tree; `make regenerate_generated_parsers` on the new runtime: of the ten generated parsers only `generated/systemverilog_parser.rs` changed (`44dc76eb… → 697f363e…`, the consumer grammar); `ebnf`, `json`, `regex`, `return_annotation`, `rtl_const_expr`, `rtl_frontend`, `scratch`, `semantic_annotation` and the rest byte-identical (`shasum` before / after) — the primitive is runtime-only, no codegen changed.

**Measured on the final bytes.** ADJUDICATION-REPROS: checked=1629 armed=225 listed=795 multi_profile_rows=603 preprocess_rows=13 failures=0; `LANE-TOTALS: lane=sv_2017 rows=16336 pass=9843 fail=6493 timeout=0` (unchanged), `LANE-TOTALS: lane=sv_2023 rows=16336 pass=9868 fail=6468 timeout=0` (unchanged), `LANE-TOTALS: lane=verilog_2005 rows=2606 pass=2283 fail=323 timeout=0` (was 2284 / 322) — `LANE-MOVERS lane=sv_2017 … pass_to_fail=0 fail_to_pass=0`, `sv_2023` the same, `LANE-MOVERS lane=verilog_2005 … pass_to_fail=1 fail_to_pass=0` — EXACTLY the pre-registered row, iverilog `ivltests/module_inout_port_list_def.v` (`inout [31:0] x, y = 1`, listed CE, keyed `must_reject` all along), 0 the other way, 0 on the SV lanes; `AST-IDENTITY-SWEEP: profile=sv_2017 files=16336 both_accepted=9843 identical=9843 ast_moved=0 verdict_moved=0`, `profile=sv_2023 … both_accepted=9868 identical=9868 ast_moved=0 verdict_moved=0`, `profile=verilog_2005 files=2606 both_accepted=2283 identical=2283 ast_moved=0 verdict_moved=1 both_nonaccepting=322` — every file both binaries accept renders byte-identically, the shared `list_of_port_declarations` included; `TWO-SIDED sv_2017: ast_moved=0 verdict_moved=0 predicted_verdict_movers=0 …`, `sv_2023` the same, `TWO-SIDED verilog_2005: ast_moved=0 verdict_moved=1 predicted_verdict_movers=1 predicted_but_unmoved=0 moved_but_not_predicted=0` — the prediction was the whole-lane interpreter pre-screen; `match=6065 unexplained=1 explained=1419 deferred=8851` (the SV lanes unchanged) and `v2005: match=2345 unexplained=0 explained=190 deferred=71` (was 2344 / 1: the `.8c.2` row now matches; the edition-net-declaration pass re-keys its 11 as before); `sv_syntax_closure_gate` GREEN on the allow-list of ONE — `unreachable_rules: 1` (`casting_type`), the baseline STALE (two rules more) and every constraint holding, re-stamped; union GREEN and UNCHANGED on the gate's own counts `1515/10/1490/15` + `1505/0` at seeds 0 / 7 / 42 (`✅ SV recognized cert-coverage union gate passed (canonical UNKNOWN=15, union UNKNOWN=0, residual=[], deterministic across seeds [0,7,42])` — both new rules are gated `verilog_2005`, outside this population; the shared `list_of_port_declarations` renders the same child; `sample_parse_failures=0`); `verilog_2005` conformance on the gate's own counts `1248/397/836/15` (total `1246 → 1248`: the two new rules; witness +1 `ansi_port_list_reset_v2005`; UNKNOWN `14 → 15`: `ansi_port_default_governed_v2005`, UNKNOWN BY CONSTRUCTION — its `latest_fact_attribute_equals` gate on `defaults` is store-entry-blocked for the witness planner, which reads a positive gate on `declaration_family` only (`ENGINE-UNIVERSAL-SERVICES.109`, routed with the measured need); the rule is exercised on the release binary by four `control_v2005_ansi_headerless_*` pins and refuses the four `invalid_v2005_ansi_headerless_*` pins and the corpus row) — adopted, `✅ verilog_2005 conformance gate passed` on the re-run; `rtl_const_expr_cert_gate` ✅, `annotation_differential_parity_gate` ✅, `return_parity_gate` ✅, `stimuli_module_parity_gate` ✅, `sv_preprocessor_reachability_closure_gate` ✅; the four formal-exhaustive closure umbrellas (`regex`, `vhdl`, `sv`, `sv_preprocessor`) and `sv_preprocessor_syntax_closure_gate` RED on three conditions that PREDATE this slice, measured as such — the EBNF dual-run `systemverilog` flow at 198 envelope divergences on the `1.0.236` grammar against a ceiling of 155 (201 with this slice's text; the `return_annotation` flow also over), the preprocessor syntax-closure contract without an identity block (untouched since 2026-05-18), the VHDL realistic contract expecting `process_wait_statement.vhd` to fail while both the `1.0.236` and `1.0.237` probes accept it — routed `SV-CORPUS-GRAD.24` with the numbers; `[ast_shape_contract][systemverilog] samples=57 aligned=57 drift=0 regression_lock_failures=0 aligned_assertion_failures=0`; the gate's first run refused on `systemverilog_fact_kind_registry_is_populated_in_generated_parser`, which pinned the generated parser's fact-kind registry at four — re-pinned at five with `ansi_port_header` asserted non-exportable (the exportable set stays `{type_name}`), GREEN on the re-run; reproducibility ✓; `sv_parse_cost_family_share` `parse-cost: derived family share 2.648 % (23,555,035 of 889,510,786 entries over 16336 files, 0 no-dump)` — byte-identical to `1.0.236`'s derivation (every emit and both new rules are `verilog_2005`-gated; the SV lanes' entries did not move), every anchor unchanged; `sv_parse_cost_ratchet` on the pinned sample: no BINDING metric rose (the first run refused only on the stale baseline), `sv_parse_cost_rebaseline` `REBASELINED`, then `parse-cost-ratchet: OK (identity fresh for: generated parser, grammar raw ast, instrument, sample inputs; 192 pinned sample files)`; the pipeline lane 16 336 rows on the new debug probe: `PIPELINE-LANE: files=16336 pass=12200 fail=3859 pp_fail=277 timeout=0 crash=0 wall_s=554 probe=rust/target/debug/parseability_probe timeout_s=600 jobs=6` — every total identical to `1.0.236`'s and `pipeline_results.tsv` BYTE-IDENTICAL to HEAD's (`git diff --quiet`), 0 movers: the pipeline lane parses under `sv_2017`, where nothing moved; `PIPELINE-LANE-CURRENCY: OK`; `sv-corpus-denominator: OK (tuple 7485/2606/6245/4039/1 = adjudicated/routed/no-verdict/dark/bar; under the pipeline 9139/2606/4591/3311/1; …)` — both UNCHANGED (the mover is a `verilog_2005` row keyed `must_reject`, a match before and after); `IDENTITY-CARRIERS: OK (live=14 frozen=7 rows_equal=65 notes=8 not_evaluated=0)`; `ANSWER-KEY-INTENDED-INVALID-CENSUS: rows=0 still_in_bar=0 disagree=0`; `corpus-key-integrity: OK`; the `verilog_2005` keyword-faithfulness census `V2005-KEYWORD-FAITHFULNESS: reachable_rules=879 reachable_keywords=140 candidates=14 over_acceptances=0 gated=25 identifier_consumed=6 directive_argument=5` (reachable rules 877 → 879, the two new rules; the rest unchanged) and the probe `V2005-KEYWORD-FAITHFULNESS-PROBE: pass=18 fail=0`; `cargo test --features "generated_parsers ebnf_dual_run" --lib`: `test result: ok. 3 passed; 0 failed` — `ast_pipeline::semantic_runtime::tests::latest_fact_attribute_equals_reads_the_nearest_standing_fact`, `parse_harness_semantic_suite::gate::every_semantic_construct_is_byte_identical` (37/37, the new `sem_latest_attr_gate` among them), `parse_harness_semantic_suite::gate::semantic_construct_coverage_is_complete`; `parse_harness_equivalence_gate`: `test result: ok. 4 passed` — every registered grammar CERTIFIED byte-identical between the interpreter and its generated parser on the stimuli corpus at seeds 0/7/42, the deferred ratchet and completeness held; `parse_harness_semantic_gate`: `test result: ok. 2 passed` (37/37 constructs byte-identical, coverage complete).

**Consumers.** The predicate roster is fifteen (`docs/book/src/semantic-store.md`); nothing to change unless a consumer feeds `verilog_2005` a header-less default behind a header that admits none. The `verilog_2005` bar 1 → 0 (`accepts_invalid` 1 → 0: `module_inout_port_list_def.v` matches; the `.8c.2` family is closed and the `verilog_2005` bar is EMPTY).

### 1.0.236 / Contract 1.0.236 — SV-CORPUS-GRAD.3.68 (`PGEN-SV-CORPUS-GRAD-0404`, 2026-09-17), ledger `SV-0121` (`Released`), **schema 33 unchanged**:

⭐ **A net declaration under `verilog_2005` follows IEEE 1364-2005 A.2.1.3: no data type, one range.** Through `1.0.235` the sv_2017 twin served both profiles and read 1800's `data_type_or_implicit`, so `wire integer x;`, `wire real r;` and `wire [3:0][3:0] a;` parsed under `verilog_2005`. A `verilog_2005` twin behind the `net_declaration` dispatcher now renders the sv twin's six fields (the implicit carrier over `( signed )? ( range )?`, one range as the one-element list), so every accepted tree is byte-identical (the corpus AST identity sweeps read `ast_moved=0` on every lane (9 843 / 9 868 / 2 284 files both binaries accept, every one identical)); the sv_2017 twin's gate narrows to `sv_2017` and the SV profiles do not move.

**Measured before a byte was generated.** 28 probes × 3 profiles under the interpreter: every legal shape holds on every profile, the six illegal ones flip under `verilog_2005` only, 21 legal probes render byte-identical trees (21 legal probes × `verilog_2005` all `ast SAME` against `1.0.235` (`--interpret-parse-ast-json` + `cmp`; 12 of them read DIFF on the first cut, whose helper lacked the sv rule's `-> {signing: $1, dims: $2}` template), 19 × `sv_2017` SAME (`wire bit;` / `wire logic;` refused under sv by both grammars, the word being a keyword there)); the whole `verilog_2005` lane pre-screened (`PRESCREEN-CORPUS: files=2606 arms=2606 in 3684s` — `(true, true)` 2284, `(false, false)` 311, `(true, false)` 11: EXACTLY the eleven pre-registered rows (`br_gh156.v`, `ca_time_real.v`, `delayed_sfunc.v`, `pr1864110a.v`, `pr1864110b.v`, `pr1864115.v`, `module_nonansi_fail12.v`, `br_gh497a.v`, `interface_array_multi.v`, `interface_delay_1.v`, `interface_delay_2.v`), none unpredicted, none predicted-but-unmoved; `module_nonansi_fail5.v` refused before and after with its stop moved earlier (226 → 207, onto `wire integer`)); the pins pre-screen `PRESCREEN-PINS: jobs=1607 verdict_moved=6 ast_moved=0 in 2719s` — EXACTLY the six `invalid_v2005_net_*` pins under `verilog_2005` accept → reject (`signed_integer` refused on every profile before and after), every other arm the same verdict and the same tree; predictions pre-registered before any lane ran.

**Measured on the final bytes.** ADJUDICATION-REPROS: checked=1605 armed=225 listed=783 multi_profile_rows=595 preprocess_rows=13 failures=0 (the first run read failures=2: `control_v2005_net_bit_name.sv` declared on every profile while `bit` is a keyword under the 1800 profiles — the manifest row corrected to `verilog_2005` alone, the edition contrast it pins); `LANE-TOTALS: lane=sv_2017 rows=16336 pass=9843 fail=6493 timeout=0` (unchanged), `LANE-TOTALS: lane=sv_2023 rows=16336 pass=9868 fail=6468 timeout=0` (unchanged), `LANE-TOTALS: lane=verilog_2005 rows=2606 pass=2284 fail=322 timeout=0` (was 2295 / 311) — `LANE-MOVERS lane=sv_2017 … pass_to_fail=0 fail_to_pass=0`, `sv_2023` the same, `LANE-MOVERS lane=verilog_2005 … pass_to_fail=11 fail_to_pass=0` — EXACTLY the eleven pre-registered rows (iverilog `br_gh156.v`, `ca_time_real.v`, `delayed_sfunc.v`, `pr1864110a.v`, `pr1864110b.v`, `pr1864115.v`, `module_nonansi_fail12.v`, `br_gh497a.v`; sv2v `interface_array_multi.v`, `interface_delay_1.v`, `interface_delay_2.v`), 0 the other way, 0 on the SV lanes; `AST-IDENTITY-SWEEP: profile=sv_2017 files=16336 both_accepted=9843 identical=9843 ast_moved=0 verdict_moved=0`, `profile=sv_2023 … both_accepted=9868 identical=9868 ast_moved=0 verdict_moved=0`, `profile=verilog_2005 files=2606 both_accepted=2284 identical=2284 ast_moved=0 verdict_moved=11 both_nonaccepting=311` — every file both binaries accept renders byte-identically; `TWO-SIDED sv_2017: ast_moved=0 verdict_moved=0 predicted_verdict_movers=0 …`, `sv_2023` the same, `TWO-SIDED verilog_2005: ast_moved=0 verdict_moved=11 predicted_verdict_movers=11 predicted_but_unmoved=0 moved_but_not_predicted=0` — the prediction was the whole-lane interpreter pre-screen; `edition-net-declaration pass [verilog_2005]: 11 row(s) re-keyed must_reject`; `match=6065 unexplained=1 explained=1419 deferred=8851` (the SV lanes unchanged) and `v2005: match=2344 unexplained=1 explained=190 deferred=71` — the count unchanged because the eleven were `match` as `must_accept` / pass and are `match` as `must_reject` / fail; unexplained 1 = `module_inout_port_list_def.v`; `sv_syntax_closure_gate` GREEN on the allow-list of ONE — `unreachable_rules: 1` (`casting_type`), the baseline STALE (four rules more) and every constraint holding, re-stamped; union GREEN and UNCHANGED on the gate's own counts `1515/10/1490/15` + `1505/0` at seeds 0 / 7 / 42 (`✅ SV recognized cert-coverage union gate passed (canonical UNKNOWN=15, union UNKNOWN=0, residual=[], deterministic across seeds [0,7,42])` — the four new rules are gated `verilog_2005` and outside this population, the sv_2017 twin stays inside it; `sample_parse_failures=0`); `verilog_2005` conformance GREEN on the gate's own counts `1246/397/835/14` (`✅ verilog_2005 conformance gate passed (lint orphans=0, corpus 369 checks/0 mismatches, aliases 2, cert deterministic across seeds [0,7,42])` — total `1243 → 1246`: +4 new rules, −1 the sv_2017 twin now gated out of this profile; witness +3, proof and unknown unchanged); `[ast_shape_contract][systemverilog] samples=57 aligned=57 drift=0 regression_lock_failures=0 aligned_assertion_failures=0`; reproducibility ✓; `sv_parse_cost_family_share` `parse-cost: derived family share 2.648 % (23,555,035 of 889,510,786 entries over 16336 files, 0 no-dump)` — byte-identical to `1.0.235`'s derivation (the family share is measured over the SV lanes, which did not move), every anchor unchanged; `sv_parse_cost_ratchet` on the pinned sample: no BINDING metric rose (the first run refused only on the stale baseline), `sv_parse_cost_rebaseline` `REBASELINED`, then `parse-cost-ratchet: OK (identity fresh for: generated parser, grammar raw ast, instrument, sample inputs; 192 pinned sample files)`; the pipeline lane 16 336 rows on the new debug probe: `PIPELINE-LANE: files=16336 pass=12200 fail=3859 pp_fail=277 timeout=0 crash=0 wall_s=506 probe=rust/target/debug/parseability_probe timeout_s=600 jobs=6` — every total identical to `1.0.235`'s and `pipeline_results.tsv` BYTE-IDENTICAL to HEAD's (`git diff --quiet`), 0 movers: the pipeline lane parses under `sv_2017`, where nothing moved; `PIPELINE-LANE-CURRENCY: OK`; `sv-corpus-denominator: OK (tuple 7485/2606/6245/4039/1 = adjudicated/routed/no-verdict/dark/bar; under the pipeline 9139/2606/4591/3311/1; …)` — both UNCHANGED (the eleven movers are `verilog_2005` rows re-keyed `must_reject`, matches before and after); `IDENTITY-CARRIERS: OK (live=14 frozen=7 rows_equal=67 notes=6 not_evaluated=0)`; `ANSWER-KEY-INTENDED-INVALID-CENSUS: rows=0 still_in_bar=0 disagree=0`; the `verilog_2005` keyword-faithfulness census `V2005-KEYWORD-FAITHFULNESS: reachable_rules=877 reachable_keywords=140 candidates=14 over_acceptances=0 gated=25 identifier_consumed=6 directive_argument=5` (reachable rules 876 → 877, the rest unchanged) and the probe `V2005-KEYWORD-FAITHFULNESS-PROBE: pass=18 fail=0`.

**Consumers.** Nothing to change unless a consumer feeds `verilog_2005` a net with a data type or a second packed range. Answer key: the EDITION-NET-DECLARATION pass (token-exact, 12 self-test arms) re-keys the 11 rows `must_reject` — six Icarus real-net extension tests its own lists mark `normal`, `module_nonansi_fail12.v` (listed CE), and four multi-range nets, three of them sv2v goldens the suite's contract calls valid Verilog-2005 (the third sv2v conformance gap after `++` and the empty user call).

### 1.0.235 / Contract 1.0.235 — SV-CORPUS-GRAD.3.67 (`PGEN-SV-CORPUS-GRAD-0403`, 2026-09-17), ledger `SV-0120` (`Released`), **schema 33 unchanged**:

⭐ **The based literal's lexical strictness, on every profile.** IEEE 1364-2005 A.8.7 = IEEE 1800-2017 A.8.7 = IEEE 1800-2023 A.8.7 agree: a size is a non-zero unsigned number, each base has its own digit set and a value starts with a digit, a decimal value is digits or ONE x/z digit followed by underscores. Through `1.0.234` the `integral_number` terminal took one value charset for every base, so `0'b0`, `04'b1`, `4'b_1`, `4'b2`, `4'o8`, `4'b1a` and `32'dx_1` parsed as numbers. The same terminal slot now reads one alternative per base with the `.3.56` white-space-or-comment gap kept verbatim; the `number` carrier is untouched, so every accepted tree is byte-identical (8/8 legal-probe arms SAME under the interpreter, sweeps the corpus AST identity sweeps read `ast_moved=0` on every lane (9 843 / 9 868 / 2 295 files both binaries accept, every one identical)).

**Measured before a byte was generated.** The new language is a subset of the old, so only a file carrying a refused literal can move: the corpus scanned with comments and strings blanked holds 7 such files of 18 942; under the interpreter on all three profiles EXACTLY 3 move, accept → reject, the other 4 already failing at the same furthest position; 14 illegal probes flip on every profile, 14 legal probes hold; the pins pre-screen `PRESCREEN-PINS: jobs=1547 verdict_moved=42 ast_moved=0 in 1645s` — EXACTLY the 14 `invalid_lit_*` pins × 3 profiles accept → reject, no other arm; on the first cut (the `.3.66`-prescribed re-gate) the same pre-screen read `verdict_moved=2` — the two member-chain pins re-admitted — which is how slice 0 was corrected before any lane ran.

**Measured on the final bytes.** ADJUDICATION-REPROS: checked=1547 armed=225 listed=757 multi_profile_rows=576 preprocess_rows=13 failures=0; `LANE-TOTALS: lane=sv_2017 rows=16336 pass=9843 fail=6493 timeout=0` (was 9846 / 6490), `LANE-TOTALS: lane=sv_2023 rows=16336 pass=9868 fail=6468 timeout=0` (was 9871 / 6465), `LANE-TOTALS: lane=verilog_2005 rows=2606 pass=2295 fail=311 timeout=0` (was 2297 / 309) — `LANE-MOVERS lane=sv_2017 … pass_to_fail=3 fail_to_pass=0` (iverilog `br_gh60a.v`, `br_gh79.v`, verilator `t_const_dec_mixed_bad.v`), `sv_2023` the same three, `LANE-MOVERS lane=verilog_2005 … pass_to_fail=2 fail_to_pass=0` (the two ivtest rows) — EXACTLY the pre-registered rows, 0 the other way on every lane; `AST-IDENTITY-SWEEP: profile=sv_2017 files=16336 both_accepted=9843 identical=9843 ast_moved=0 verdict_moved=3 both_nonaccepting=6490`, `profile=sv_2023 … both_accepted=9868 identical=9868 ast_moved=0 verdict_moved=3`, `profile=verilog_2005 files=2606 both_accepted=2295 identical=2295 ast_moved=0 verdict_moved=2` — every file both binaries accept renders byte-identically; `TWO-SIDED sv_2017: ast_moved=0 verdict_moved=3 predicted_verdict_movers=3 predicted_but_unmoved=0 moved_but_not_predicted=0`, `sv_2023` the same, `TWO-SIDED verilog_2005: ast_moved=0 verdict_moved=2 predicted_verdict_movers=3 predicted_but_unmoved=1 moved_but_not_predicted=0` (the one predicted-but-unmoved is `t_const_dec_mixed_bad.v`, which the population pre-screen read on `verilog_2005` and which is not a row of that lane) — the prediction was the 7-file population's interpreter pre-screen; `match=6065 unexplained=1 explained=1419 deferred=8851` (the SV lanes' counts unchanged: `t_const_dec_mixed_bad.v` moved `must_accept`/pass → `must_reject`/fail, a match either way once re-keyed by A.8.7) and `v2005: match=2344 unexplained=1 explained=190 deferred=71` (was 2342 / 3: the two `.8c.2` rows now match); `sv_syntax_closure_gate` GREEN on the allow-list of ONE — `unreachable_rules: 1` (`casting_type`), the baseline STALE (two rules fewer) and every constraint still holding, re-stamped; union GREEN on the gate's own counts `1515/10/1490/15` + `1505/0` at seeds 0 / 7 / 42 (`✅ SV recognized cert-coverage union gate passed (canonical UNKNOWN=15, union UNKNOWN=0, residual=[], deterministic across seeds [0,7,42])`) — total `1517 → 1515` and proof `12 → 10`: the two profile-dead helpers deleted, nothing else in the population moved (a terminal re-spelling has no rule-level footprint); witness 1490, unknown 15 / 0 and the residual `[]` unchanged, `sample_parse_failures=0`; `verilog_2005` conformance GREEN and UNCHANGED at `1243/397/832/14` (`✅ verilog_2005 conformance gate passed (lint orphans=0, corpus 369 checks/0 mismatches, aliases 2, cert deterministic across seeds [0,7,42])` — the two helpers were gated out of that population all along); `[ast_shape_contract][systemverilog] samples=57 aligned=57 drift=0 regression_lock_failures=0 aligned_assertion_failures=0`; reproducibility ✓; `sv_parse_cost_family_share` `parse-cost: derived family share 2.648 % (23,555,035 of 889,510,786 entries over 16336 files, 0 no-dump)` — the share unchanged at three decimals while both counts FALL (the family −563 and the corpus −19 811 entries: the three refused files stop early), every anchor still `2.648`, `cost.md`'s counts re-derived; `sv_parse_cost_ratchet` on the pinned sample: no BINDING metric rose (the first run refused only on the stale baseline), `sv_parse_cost_rebaseline` `REBASELINED`, then `parse-cost-ratchet: OK (identity fresh for: generated parser, grammar raw ast, instrument, sample inputs; 192 pinned sample files)`; `--verify-families` 32 declared LR names, matching the pin; the pipeline lane 16 336 rows on the new debug probe (`PIPELINE-LANE: files=16336 pass=12200 fail=3859 pp_fail=277 timeout=0 crash=0 wall_s=523 probe=rust/target/debug/parseability_probe timeout_s=600 jobs=6`; `PIPELINE-LIBRARY: Cores-VeeR-EL2=2/2 black-parrot=3/7 opentitan=225/396 uvm-core=0/2 published_packages=166 unresolved_imports=44 cyclic_files=0 member_rows=5074`): `PIPELINE-VS-RAW: … rows=16336 raw_pass=9843 pipeline_pass=12200 pipeline_fail=3859 pp_fail=277 timeout=0 crash=0 missing_raw=0` — pass `12 204 → 12 200`, EXACTLY the four pre-registered rows (friscv `friscv_alu.sv`, the two ivtest rows, verilator `t_const_dec_mixed_bad.v`), nothing else; `PIPELINE-LANE-CURRENCY: OK`; `sv-corpus-denominator: OK (tuple 7485/2606/6245/4039/1 = adjudicated/routed/no-verdict/dark/bar; under the pipeline 9139/2606/4591/3311/1; …)` — the raw tuple UNCHANGED (the movers are deferred or routed rows), the pipeline reading `9140/2606/4590/3310/1 → 9139/2606/4591/3311/1` (friscv's row leaves the adjudicated set under expansion) and the platform book's anchor moved with it; `corpus-key-integrity: OK`; `IDENTITY-CARRIERS: OK (live=14 frozen=7 rows_equal=67 notes=6 not_evaluated=0)`; `ANSWER-KEY-INTENDED-INVALID-CENSUS: rows=0 still_in_bar=0 disagree=0`; the `verilog_2005` keyword-faithfulness census `V2005-KEYWORD-FAITHFULNESS: reachable_rules=876 reachable_keywords=140 candidates=14 over_acceptances=0 gated=25 identifier_consumed=6 directive_argument=5` unchanged and the probe `V2005-KEYWORD-FAITHFULNESS-PROBE: pass=18 fail=0`.

**Consumers.** Nothing to change unless a consumer feeds a literal Annex A does not derive. ⛔ Surfaced for the director: friscv `rtl/friscv_alu.sv` carries 27 leading-zero sizes (`{01'b0, …}`), lexed leniently by every tool and derived by no edition — one real design now fails under the pipeline lane; a leniency would be a deviation from the LRM. Slice 0: the two profile-dead helpers only the `verilog_2005` twins referenced (gated sv-only, `[]`-rendering optionals) are deleted and the fields spelled literally `[]` (behaviour-neutral, `1715 → 1713` rules, union proof `12 → 10`); the `1.0.234` entry's reading "ungated, misnamed" was false — a first re-gate re-admitted `a.b` under `verilog_2005` and the pins pre-screen caught it before any lane ran. Answer key: verilator `t_const_dec_mixed_bad.v` pinned `must_reject` by A.8.7 — its own driver refuses it.

### 1.0.234 / Contract 1.0.234 — SV-CORPUS-GRAD.3.66 (`PGEN-SV-CORPUS-GRAD-0402`, 2026-09-16), ledger `SV-0115` … `SV-0119` (`Released`), **schema 33 unchanged**:

⭐ **The declaration tier of the IEEE 1364-2005 bar — eight productions stop carrying 1800's shape on the
`verilog_2005` profile.** Port declarations, ANSI and non-ANSI, follow A.2.1.2 (no variable type, default or dimension on
`input` / `inout`; `output reg` / `integer` / `time` alone take a default; no `real` port); a `parameter` cannot sit inside a
generate scope (A.4.2 / A.1.4); a dotted name in a constant is neither a member chain, a generate-scoped parameter nor a method call (A.8.4 / A.9.3 / A.8.2); a system function call carries at least
one expression while a system task enable may be empty (A.8.2 / A.6.9, a context split); a function's items and ports are
inputs only (A.2.6). `1.0.234` gives each production a pass-through dispatcher over the 1800 rule (verbatim, for
`sv_2017` / `sv_2023`) and a `verilog_2005` twin with the same template fields — every accepted tree byte-identical, by
construction and measured. Under `verilog_2005`, `input reg a`, `inout integer c`, `inout [31:0] x, y = 1`, `input a = 1`,
`output a = 1`, `input real r`, `input a [0:3]`, `input reg a;`, a generate-scope `parameter`, `dut.WIDTH`, `a.b`, `$time()`,
`function f; output o; …` and `function f(output o)` now REJECT; `output reg [3:0] q = 0`, `output integer c = 1`,
`input a, b`, `input a; reg a;`, `localparam` in generate, `P[3:0]`, `$clog2(8)`, `$time`, `$display()`,
`function f(input a, b)` and `task t(input a, output b)` keep parsing. Measured on the final bytes: three lanes with EXACTLY the nine pre-registered `verilog_2005` rows moved (`pass 2 306 → 2 297`, all `pass → fail`, 0 the other way) and ZERO rows moved on `sv_2017` / `sv_2023`; the corpus AST identity sweeps `ast_moved=0` on every lane; the pins `checked=1469 failures=0`; the closure gate `unreachable_rules: 1`; both certificates GREEN on their gates' own counts; the `verilog_2005` bar 7 → 3.
⛔ Consumers on `verilog_2005`: Verilog text that only a tool's extension admits is refused from `1.0.234`; the SV
profiles are unchanged. One residual is stated rather than hidden and routed: a header-less ANSI item with a default (`inout [31:0] x, y = 1`) stays
admitted, because whether it is legal depends on the governing header — state across siblings the semantic store cannot yet
express (`ENGINE-UNIVERSAL-SERVICES.108`); a later empty slot in a system function call (`$fread(mem, fd, , count)`, the
clause's own example) keeps parsing. Detail: the contract's current-state note; the
ledger rows; `docs/tasks/SV-CORPUS-GRAD.md` `.3.66`.

### 1.0.233 / Contract 1.0.233 — SV-CORPUS-GRAD.3.65 (`PGEN-SV-CORPUS-GRAD-0401`, 2026-09-16), ledger `SV-0112` / `SV-0113` / `SV-0114` (`Released`), **schema 33 unchanged**:

⭐ **The statement tier of the IEEE 1364-2005 bar — three productions stop carrying 1800's shape on the
`verilog_2005` profile.** A user tf-call needs at least one expression inside its parentheses and has no named
argument (A.8.2 / A.6.9); a task body holds exactly one statement and a function body at least one item declaration
or a non-empty port list and exactly one statement, neither with an end label (A.2.7 / A.2.6); a block admits
declarations only behind a label, never a null statement, and has no end label (A.6.3). `1.0.233` gives each production
a pass-through dispatcher over the 1800 rule (verbatim, for `sv_2017` / `sv_2023`) and a `verilog_2005` twin carrying
the 1364-2005 production with the same template fields — every accepted tree byte-identical, by construction and
measured. Under `verilog_2005`, `f()`, `t();`, `t(.x(1))`, a two-statement or empty task body, `function f();`,
`function f; f = 1; endfunction`, `endtask : t`, `endfunction : f`, `begin integer x; … end`, `fork integer x; … join`,
`begin ; end` and `end : b` now REJECT; `$display()`, `t;`, `f(1)`, `task t; input x; ; endtask`, `begin : b integer x; … end`
keep parsing. Measured on the final bytes: three lanes with EXACTLY the ten pre-registered `verilog_2005` rows moved (`pass 2 316 → 2 306`, all `pass → fail`, 0 the other way) and ZERO rows moved on `sv_2017` / `sv_2023`; the corpus AST identity sweeps `ast_moved=0` on every lane; the pins `checked=1343 failures=0`; the closure gate `unreachable_rules: 1`; both certificates GREEN on their gates' own counts; the `verilog_2005` bar 13 → 7. Answer key: the EDITION-EMPTY-CALL pass rules four rows
whose key was a tool's acceptance of `t()`; eight controls whose `verilog_2005` claim rested on 1800-only text are
corrected. ⛔ Consumers on `verilog_2005`: Verilog text that only a tool's extension admits is refused from `1.0.233`;
the SV profiles are unchanged. Detail: the contract's current-state note; the ledger rows; `docs/tasks/SV-CORPUS-GRAD.md` `.3.65`.

### 1.0.232 / Contract 1.0.232 — SV-CORPUS-GRAD.3.64 (`PGEN-SV-CORPUS-GRAD-0400`, 2026-09-16), ledger `SV-0111` (`Released`), **schema 33 unchanged**:

⭐ **Deletions only — the four general terminals `1.0.231` orphaned are gone.** `1.0.231` moved the z/x edge
descriptors onto dedicated boundary-free `edge_sym_*` terminals and left the general single-letter keyword terminals
`kw_x_11f6ad8e` / `kw_X_c032adc1` / `kw_z_395df8f7` / `kw_Z_909f99a7` with no reader: definition-only in the grammar,
unreachable from the entry (the syntax-closure gate listed them at the measured count), PROOF in both certificates,
and still compiled into the generated parser. `1.0.232` deletes the four definitions (`1656 → 1652` rules); the
syntax-closure allow-list returns to `casting_type` alone. The accept set and every typed AST are unchanged — by
construction (no rule referenced them) and measured on the final bytes: three lanes with ZERO rows moved in either direction (totals identical to `1.0.231`'s), the corpus AST identity sweeps `ast_moved=0 verdict_moved=0` on every lane, the pins `checked=1265 failures=0`, the closure gate `unreachable_rules: 1` (was 5), both certificates GREEN at −4 total / −4 proof with witness and unknown unchanged. Consumers of `1.0.231`
need do nothing. Detail: the contract's current-state note; ledger `SV-0111`; `docs/tasks/SV-CORPUS-GRAD.md` `.3.64`.

### 1.0.231 / Contract 1.0.231 — SV-CORPUS-GRAD.3.63 (`PGEN-SV-CORPUS-GRAD-0399`, 2026-09-15), ledger `SV-0108` / `SV-0109` / `SV-0110` (`Released`), **schema 33 unchanged**:

⭐ **Annex A's bold brackets are tokens.** Three timing-check productions had been transcribed with IEEE's literal
brackets read as EBNF optionality — the PDF-to-text extraction drops the bold that marks them — so every corpus spelling
of an edge control specifier (`edge[01, 0x, x1] clr`, the clause's own example) stopped at the `[`, a select on a delayed
terminal (`$setuphold(…, D, BL_X[0])`) stopped at its `[`, and a bracket-less `edge 01 CL` no edition derives was
accepted. `1.0.231` gives `edge_control_specifier` and the two delayed-terminal rules their `lbrack` / `rbrack` tokens
(IEEE 1364-2005 A.7.5.2 / A.7.5.3, IEEE 1800-2017 the same), puts the bracketed `delayed_data` / `delayed_reference`
alternative — the `with_expr` kind published since `1.0.64` but unreachable behind the bare identifier — first, and
moves the two-character descriptors `0x` / `x1` / `1x` onto six boundary-free terminals `edge_sym_*` (the general
`x\b` terminal could never match a half followed by a word character: the UDP-symbol class `.3.30` measured, the
same remedy). And one edition contrast: `$width` needs no threshold under IEEE 1364-2005 A.7.5.1 (§15.4's own example
`$width(posedge clr, tWidth);`) while 1800-2017 / 2023 require it, so a `@profiles: ["verilog_2005"]` sub-rule admits the
two-argument form on the 1364-2005 profile only, wrapped as `sv_dollar_width_timing_check -> {kind: "no_threshold", body}` —
the three-argument form keeps its shape byte-for-byte, and `sv_2017` / `sv_2023` still refuse the short one. Nine pins
(five `fixed`, two `invalid`, two `control`); `1649 → 1656` rules. Measured on the final bytes: `sv_2017` pass 9 844 → 9 846 / fail 6 490, `sv_2023` 9 869 → 9 871 / 6 465, `verilog_2005` 2 313 → 2 316 / 290 — exactly the three predicted rows (`pr1520314.v`, `pr1867332.v` on every lane; `test_15_04_00_1.v` on `verilog_2005`), every one `fail → pass`, 0 the other way;
the corpus AST identity sweeps read `ast_moved=0` on every lane (every file both binaries accept has a byte-identical typed AST) with `verdict_moved` 2 / 2 / 3, the predicted rows; the pipeline lane exactly 2 rows `fail → pass` (iverilog `pr1520314.v`, `pr1867332.v`), 0 the other way, `pass 12 202 → 12 204`; the `verilog_2005` bar 16 → 13 (`rejects_valid` 3 → 0 — the three rows this leaf was opened on; the thirteen `.8c.2` accepts-invalid pins are all that remain). ⛔ Consumers on `verilog_2005`:
a `width` timing check's body may now be the `no_threshold` wrapper with no `threshold` field — read `body.kind` first.
Detail: the contract's current-state note; the ledger rows; `docs/tasks/SV-CORPUS-GRAD.md` `.3.63`.

### 1.0.230 / Contract 1.0.230 — SV-CORPUS-GRAD.3.60 + .3.61 (`PGEN-SV-CORPUS-GRAD-0393`, 2026-09-14), ledger `SV-0103` / `SV-0104` / `SV-0105` / `SV-0106` (`Released`), **schema 32 → 33**:

⭐ **The compilation-unit scope and the module are different name spaces — and a lookahead is a child.** IEEE
1800-2017 3.13 puts a `$unit` typedef in the compilation-unit scope name space and a same-named net, function, port
or parameter in the module's, so `wire T = 1'b1;`, `function U;`, `module m(input T);`, the non-ANSI `input T;`, a tf port `input T;` and
`parameter int P = 5, T typed_value = 8'ha5, p::PT pkg_value = 6'h2a` each declare a NEW name under an IMPLICIT type
(A.2.1.3, A.2.6, A.1.3, A.2.1.2, A.2.7, A.2.4 footnote 18). Through `1.0.229` each read only `data_type_or_implicit`, whose `data_type` arm consumed
the typedef name and found no declarator (the five iverilog `sv_type_identifier_*.v` rows). `1.0.230` admits the typed
reading only where a declarator follows — `data_type_or_implicit_declared := data_type &( declaration_identifier | hash ) |
implicit_data_type`, one new rule (`1648 → 1649`) read by the net, port and tf-port declarations, the same guard on
`var_data_type` and the function return type — so the typedef name before `=` `;` `,` `)` is the declarator under the
implicit reading (a first candidate spelled implicit-twin ALTERNATIVES; the parse-cost ratchet refused it, because the
engine derives every alternative and a twin re-derived each typed declaration it also matched: `committed +3.41 %`);
and it guards the default-less parameter arm with `&( comma | rparen | semi )` — complete only there — and the type
lists with the same reading. The first of those guards reproduced a shape on a CONTROL — `[$1, $2::2*]` over a
guard-opened group returned the comma — and the question "how does the `.3.35` precedent survive its lookahead?" was
answered by dumping its tree: it did not. In the engine a lookahead is a positional child that binds `[]`; ten
templates in the shipped grammar had not counted it, and every gate had stayed green because the accept set never
moved and an identity sweep compares a release with the one before. **Schema 33**: `type_assignment`'s four twins read
`{name: $2, value: $3}` — every `parameter type T = int` since `1.0.79` had read `name: []`, `value: T` and carried no
default type (`SV-0104`); the `.3.35` type lists read `[$1, $2::3*]` — `type T = int, U = real` had dropped `U` for a
`comma` node since `1.0.210` (`SV-0105`); the `.13c.2s` argument lists read `ordered_tail: [$2::3*]` — a property or
sequence instance's further positionals had been `[]` since `1.0.191` (`SV-0106`); the mixed string parameter port
list's `type_tail` / `decl_tail` are clean arrays. Recurrence guard: the doctrine `LOOKAHEAD-SLOT-TEMPLATES`
(`scripts/lookahead_slot_audit.py`, self-tested, every commit); the engine-side lint is routed. Measured on the final
bytes: the raw lanes byte-inert — `sv_2017` pass 9 847 / fail 6 489, `sv_2023` 9 872 / 6 464, `verilog_2005` 2 313 / 293, every count identical to 1.0.229's, 0 rows moved (0 raw rows moved on any lane (the five iverilog rows sit behind a multi-line `` `define ``: the raw lane stops at the macro on every release, the shapes come after it)); the pipeline lane exactly 5 rows `fail → pass` (the five iverilog `sv_type_identifier_*.v` rows), 0 the other way; the corpus AST identity sweeps read `ast_moved=196 verdict_moved=0` on `sv_2017`, `204 / 0` on `sv_2023` and `0 / 0` on `verilog_2005` — every mover a `.3.61` shape (type assignments, type lists, ordered tails), the other 9 651 / 9 668 / 2 313 accepted trees byte-identical — the movers are 196 accepted files on `sv_2017` and 204 on `sv_2023`, every one a `.3.61` shape and nothing else (`rust/target/s360/classify_movers.py`, every mover's two trees diffed node by node: 193 / 201 files carry a `type_assignment` whose `name` and `value` are now bound — 822 / 838 nodes — 9 / 9 carry a type list whose second and later assignments replace a `comma` node — 26 / 26 elements — and 3 / 3 carry a property or sequence instance's `ordered_tail` — 19 / 19 arguments; 0 unclassified), the other 9 651 / 9 668 accepted trees byte-identical, `verilog_2005` untouched (2 313 identical).

### 1.0.229 / Contract 1.0.229 — SV-CORPUS-GRAD.3.59 (`PGEN-SV-CORPUS-GRAD-0392`, 2026-09-10), ledger `SV-0102` (`Released`), **schema 32 unchanged**:

⭐ **A select on a receiver is an expression, and `$` is a primary.** IEEE 1800-2017 A.8.4 derives a method
call's receiver as `hierarchical_identifier select` with `bit_select ::= { [ expression ] }`, and `$` — the last
element of a queue (7.10.4) — is a primary. Through `1.0.228` the chains' index operand carried only
`constant_primary`, so `q[$-1].delete` and `q[$].delete()` rejected at the `$` while `q[0].delete` and `x = q[$-1]`
parsed (verilator `t_dynarray_multid.v`). `1.0.229` adds `kw_dollar -> {kind: "dollar"}` as that operand's last
alternative — every constant-index receiver keeps its tree. And IEEE 1800-2023 11.4.13 defines the relative
tolerance range `[A +%- B]` beside the absolute `[A +/- B]`: `1.0.229` adds it to `value_range_sv_2023` and to the
covergroup twin as `{kind: "relative_tolerance"}` (the 2017 rules keep no tolerance range, pinned). Two additive
kinds; 1648 rules unchanged; no engine, no codegen, no Rust. The answer key pins verilator `t_stream_unpack.v`'s
`{ >> bit[1:0] {arr2}} = bit6;` `must_reject` — A.8.1's `slice_size` is a `simple_type` or a constant expression,
and a type with a packed dimension is neither. Measured on the final bytes: the raw lanes the three raw lanes byte-inert — `sv_2017` pass 9 847 / fail 6 489, `sv_2023` 9 872 / 6 464, `verilog_2005` 2 313 / 293, every count identical to 1.0.228's, 0 rows moved
(both shapes sit behind macros in the raw corpus); the pipeline lane 1 row `fail → pass` (verilator `t_dynarray_multid.v`), 0 the other way; the corpus AST identity sweeps read `ast_moved=0 verdict_moved=0` on all three lanes (9 847 / 9 872 / 2 313 accepted trees byte-identical).

### 1.0.228 / Contract 1.0.228 — SV-CORPUS-GRAD.3.58 (`PGEN-SV-CORPUS-GRAD-0391`, 2026-09-09), ledger `SV-0101` (`Released`), **schema 32 unchanged**:

⭐ **The standard's notation separates tokens by white space.** IEEE 1800-2017 A.6.5 writes the parenthesised
implicit event expression as `@ (*)`, and IEEE 1800-2023 A.6.5 spells it out as three tokens, `@ ( * )`. Through
`1.0.227` the `sv_2017` / `verilog_2005` rule read `(*` as the attribute opener and derived `@*`, `@(*)` and `@ (* )`
only, so a space after the `(` — `@( * )`, `@ ( * )`, `@ ( *)` — rejected at the `*` (sv2v's `sense_star.sv`, which asserts
every spelling equivalent, and eight OpenTitan `*_pgd.sv` files whose `always @( * ) begin` the pipeline lane reached).
`1.0.228` adds `lparen star rparen -> {kind: "wildcard_paren"}` — the kind the `sv_2023` rule already emitted — and
factors the `@` out of the six alternatives (`event_control_sv_2017 := at_sign event_control_after_at_sv_2017 -> $2`,
a pass-through): the un-factored sixth alternative cost one `at_sign` attempt per failing position (+78,986 entries
over the corpus, refused by the parse-cost ratchet); factored, the whole rule costs one, and the corpus cost falls
(`entries` −0.16 % / `memo_hits` −0.18 % promoted, `committed` +88 accepted as the wrapper's frames (`introduced_wrapper_frames`), the LR-family share `2.659 → 2.666`). An additive kind on one node, every other node byte-identical; 1647 → 1648 rules; no engine, no codegen,
no Rust.
Measured on the final bytes: the raw lanes byte-inert by construction (the form sits behind macros in the raw corpus;
the interpreter pre-screen read zero movers over 1 121 pin arms and 20 corpus arms), 9 rows `fail → pass` (sv2v `sense_star.sv`, eight OpenTitan `*_pgd.sv`), 0 the other way under the pipeline;
the corpus AST identity sweeps read `ast_moved=0 verdict_moved=0` on all three lanes (9 847 / 9 872 / 2 313 accepted trees byte-identical).

### 1.0.227 / Contract 1.0.227 — SV-CORPUS-GRAD.3.56 (`PGEN-SV-CORPUS-GRAD-0386`, 2026-09-09), ledger `SV-0100` (`Released`), **schema 32 unchanged**:

⭐ **A lexical fact lives in the terminal's regex.** Two were missing. IEEE 1800-2017 §22.5.1: a macro's text
continues over a newline preceded by a backslash and ends at the first newline that is not — through `1.0.226`
the opaque directive line ended at its first newline, so a multi-line `` `define ``'s continuation lines were parsed
as items (500 failing lane files carry the shape). §5.7.1: a based literal is three tokens — size, apostrophe and
base, digits — and the digits "optionally preceded by white space", where §5.3's white space is spaces, tabs,
newlines and formfeeds, and a comment separates tokens as it does everywhere; through `1.0.226` only spaces and
tabs were admitted between the tokens. `1.0.227` spells both in the terminals; the apostrophe and the base stay
inseparable (§5.7.1), pinned. And, exposed by that widening — the recognized-union certificate's witness generator had composed based-literal
time literals the glued unit must refuse — a third, LRM-faithful fix: a time literal's magnitude is `unsigned_number`
or a fixed-point `real_number`, never a based literal (IEEE 1800-2017 A.8.7). The value node is byte-identical for
every real time literal; only the based-literal magnitude (0 corpus rows) is dropped. 1647 rules unchanged; no engine, no codegen, no Rust.

**Accept-set impact.** Every profile: WIDEN by the two lexical shapes — the sweep reads `AST-IDENTITY-SWEEP: profile=sv_2017 files=16336 both_accepted=9771 identical=9768 ast_moved=3 verdict_moved=76 both_nonaccepting=6489`; `AST-IDENTITY-SWEEP: profile=sv_2023 files=16336 both_accepted=9796 identical=9793 ast_moved=3 verdict_moved=76 both_nonaccepting=6464`; `AST-IDENTITY-SWEEP: profile=verilog_2005 files=2606 both_accepted=2310 identical=2310 ast_moved=0 verdict_moved=3 both_nonaccepting=293` — on each SV lane exactly 3 accepted trees move (the three files whose multi-line `` `define `` bodies fold into the directive's opaque text) and 76 files newly parse; 0 accepted-tree moves on `verilog_2005`
(76 corpus files `fail → pass` on each SV lane (macro-heavy headers and number-whitespace tests across opentitan, black-parrot, friscv, sv2v, Surelog, verilator) and 3 on `verilog_2005` (sv2v `default_nettype.v`, `pattern_revised.v`, `typename_deep.v`), 0 the other way; the three AST movers are the already-accepted define files friscv `friscv_h.sv`, sv-tests `preproc_test_8.sv`, uvm-core `uvm_sequence_defines.svh`), the bar `23 → 9` on `sv_2017` (rejects-valid 21 → 7, accepts-invalid 2), `24 → 10` on `sv_2023`, `verilog_2005` unmoved at 43. **AST impact.** A `{kind: "opaque", text}` directive node now carries a
multi-line macro's whole text; the three accepted files carrying the shape are the only AST movers, pre-registered
(friscv `rtl/friscv_h.sv`, sv-tests `tests/generic/preproc/preproc_test_8.sv`, uvm-core `src/deprecated/macros/uvm_sequence_defines.svh`). Every other accepted tree is byte-identical.

**The answer key.** Twelve bar rows re-keyed `must_reject` by production: bare `begin … end` blocks at module level
and inside generate regions (A.1.4 `module_item`, A.4.2 `generate_item` — the Verilog-2001 form IEEE 1364-2005
removed), a statement as a generate item, a bare brace pair in a covergroup (A.2.11), a backslash-newline in a UDP
table row (A.5.4), free text between pragma comments (§5.4), a pipe-filter placeholder (A.1.2), and
`` `default_nettype `` inside a module (§22.8). Sixteen pins in all.

**Consumers.** A directive node's `text` may now span lines. Pins `ADJUDICATION-REPROS: checked=1121 armed=219 listed=564 multi_profile_rows=422 failures=0` (was 1078 / 219 / 548 / 406; the sixteen new pins green on the final bytes); union `sv_cert_recognized_union_gate` GREEN — canonical `1500/10/1475/15` UNCHANGED from `1.0.226` (no rule added), union witness `1490`, union UNKNOWN `0`, residual `[]`, `sample_parse_failures=0`, deterministic across seeds 0 / 7 / 42; identity re-stamped (the baseline was stale on the grammar digest and every expectation held). ⛔ The FIRST candidate (before the `time_literal` restriction) REFUSED with `sample_parse_failures` non-deterministic across seeds (3 / 2 / 0): the widened `integral_number` let the witness generator compose a based-literal time literal (`timeprecision'D 9F? fs`) whose interior layout the glued unit must refuse; restricting `time_literal` to `unsigned_number` / `real_number` (IEEE 1800-2017 A.8.7) cleared it deterministically; `verilog_2005_conformance_gate` GREEN — cert `1189/372/803/14` UNCHANGED, `corpus matrix: 369 file×profile checks, 0 mismatches`; identity re-stamped; `sv_syntax_closure_gate` GREEN — `reachable_rules: 1646 unreachable_rules: 1 unreachable_branches: 2`, the allow-list unchanged at ONE (`casting_type`), identity re-stamped; `ast_shape_contract_gate` rc 0 (`18 passed`); `[ast_shape_contract][systemverilog] samples=57 aligned=57 drift=0 regression_lock_failures=0`; NO re-anchor — the `time_literal` value node is byte-identical for every real time literal and the H.11.5 locks use plain-number glued time literals;
`check_generated_reproducibility.sh --rebaseline` — TIER 2, every family re-derives byte-identically from HEAD (`systemverilog` 0 sites, every other family 0 sites); `generated_reproducibility_v0.json` re-stamped: `systemverilog` `parser_sha` `→ 423042d2…`, `input_sha` `→ 968033d4…`; `sv_parse_cost_family_share` derived `2.659 %` (`23,552,489 of 885,794,193` entries over 16 336 files) against the carried `2.660` — ADOPTED (`CORPUS_FAMILY_SHARE_PCT` + the four prose anchors; `2.659` reproduces exactly on re-run); `sv_parse_cost_rebaseline` PROMOTED — `entries 403,243,744 → 402,647,195 (−0.15 %)` an improvement (the `time_literal → unsigned_number` re-spelling makes 149 sampled files cheaper), `committed 6,865,447 → 6,865,879 (+432)` and `memo_hits 175,043,676 → 175,049,941 (+6,265)` ACCEPTED as attributed (`accepted_rises.tsv`, invariant `advanced_files_only`): the rise is confined to the three sampled files the opaque-directive-line widening let parse further (`default_nettype.v` and `uvm_recorder_defines.svh` flip no→yes, `alert_handler_base_vseq.sv` advances furthest 334→1475), no non-advanced file rose; `sv_parse_cost_ratchet` GREEN on the promoted baseline (wall-clock advisory `+8.7 %` within the ±50 % band, machine-dependent). ⛔ `advanced_files_only` was EXTENDED this slice (`ENGINE-UNIVERSAL-SERVICES.104`) to admit a non-scoped PURE IMPROVEMENT — a file that only got cheaper cannot hide a rise — with a self-test arm (12/12).

### 1.0.226 / Contract 1.0.226 — SV-CORPUS-GRAD.3.55 (`PGEN-SV-CORPUS-GRAD-0385`, 2026-09-08), ledger `SV-0099` (`Released`), **schema 32 unchanged**:

⭐ **A time unit after white space is an identifier.** IEEE 1800-2017 A.8.7 `time_literal ::= unsigned_number
time_unit | fixed_point_number time_unit`, and Annex A footnote 44 says the number "shall not be followed by
white_space". Through `1.0.225` the unit followed the number through the engine's layout skip, so `1 s` was the
time literal `1s`: `#1 s = 0;` lost its variable and `trans ##1 s` its sequence instance — §16.8 and §16.9.11's
own examples rejected. A whitespace-insensitive grammar had no way to say "no layout between these two
tokens": the engine skips layout before every terminal, a template cannot slice a regex's capture groups, and
the `regex` crate has no lookaround, so the only grammar spelling would have folded number and unit into one
opaque regex and moved every time literal's typed node. `1.0.226` adds the rule-level `@glued: true` directive
(`ENGINE-UNIVERSAL-SERVICES.103`: the rule's regex terminals accept no leading layout; read in the one shared
layout decision the generated parser and the parse-harness interpreter both query) and spells
`time_literal := number time_unit_glued` with `@glued: true` on `time_unit_glued := /s\b/ | /ms\b/ | /us\b/ |
/ns\b/ | /ps\b/ | /fs\b/`; the `` `timescale `` directive keeps the layout-skipping `time_unit` (§22.7 allows
`1 ns / 1 ps`). `1646 → 1647` rules.

**Accept-set impact.** `sv_2017` / `sv_2023`: WIDEN by exactly the spaced shape — a number, white space and a
unit-spelled identifier after `#` or `##` (the corpus census by token: 5 lane files, 0 accepted); the sweep reads
`AST-IDENTITY-SWEEP: profile=sv_2017 files=16336 both_accepted=9767 identical=9767 ast_moved=0 verdict_moved=4 both_nonaccepting=6565`; `AST-IDENTITY-SWEEP: profile=sv_2023 files=16336 both_accepted=9792 identical=9792 ast_moved=0 verdict_moved=4 both_nonaccepting=6540`; `AST-IDENTITY-SWEEP: profile=verilog_2005 files=2606 both_accepted=2310 identical=2310 ast_moved=0 verdict_moved=0 both_nonaccepting=296` — every tree both probes accept is byte-identical on every lane, the 240 accepted files with glued time literals among them (ispras `16.08_04.sv` and `16.09.11_01.sv`, Surelog `FSMBsp13/top.v`, iverilog `pr2785294.v` — `fail → pass` on `sv_2017` and `sv_2023`, 0 the other way; `verdict_moved=0` on `verilog_2005`), the bar `31 → 23` on `sv_2017` (rejects-valid 29 → 21, accepts-invalid 2), `32 → 24` on `sv_2023`, `verilog_2005` unmoved at 43 (`match 6037 → 6045` / `6028 → 6036`). NARROW by the spaced time literal no edition admits
(`timeprecision 1 ps;`, 0 corpus rows). `verilog_2005`: untouched (time literals are IEEE 1800's);
`AST-IDENTITY-SWEEP: profile=verilog_2005 files=2606 both_accepted=2310 identical=2310 ast_moved=0 verdict_moved=0 both_nonaccepting=296`. **AST impact — none for any file the parser accepted before.** A time literal keeps its
`{value: {kind, body}, unit: {kind}}` node.

**The answer key.** Six bar rows re-keyed `must_reject` by production, each with an isolation reproducer and a
control pin: an initializer on a non-ANSI net output port (A.2.1.2 `list_of_port_identifiers` carries none), a
default on a non-ANSI input port (A.2.1.2), a modport inside a generate loop (A.1.4
`interface_or_generate_item`), a bare `begin … end` inside `generate` (A.4.2 `generate_item`), and a port
declaration in the body of a module whose header lists no ports (A.1.2: the header is ANSI, the body admits
`non_port_module_item` only). Fourteen pins in all.

**Consumers.** Nothing to change. Pins `ADJUDICATION-REPROS: checked=1078 armed=219 listed=548 multi_profile_rows=406 failures=0` (was 1050 / 219 / 534 / 392; the fourteen new rows green on the final bytes); union `sv_cert_recognized_union_gate` — canonical `1499/10/1474/15 → 1500/10/1475/15` (+1 total, +1 witness: `time_unit_glued`, witnessed by the canonical config's own time-literal samples; proof 10 and the canonical UNKNOWN 15-rule set unchanged, identical), union witness `1489 → 1490`, union UNKNOWN 0, residual `[]`, deterministic across seeds 0 / 7 / 42. The FIRST run on the final bytes refused twice over: the totals (`seed=0 canonical total=1500 (expected 1499)`) and `sample_parse_failures=10 / 10 / 8` on seeds 0 / 7 / 42 — the witness generator's sequence join wrote `4 us`, `6 ms`, `1 ps`, which the glued unit rightly refuses; the generator's join now appends a glued rule reference with no separator (`append_sequence_segment`, test `a_glued_rule_reference_renders_without_a_separator`), the expectations were stamped from the per-seed `summary.json` with a `.3.55` baseline note, and the second run confirmed them with `sample_parse_failures=0` and re-stamped the identity; `verilog_2005_conformance_gate` — cert `1188/371/803/14 → 1189/372/803/14` (`CERTIFICATE-COVERAGE: grammar='systemverilog' entry='systemverilog_file' samples=40 total=1189 proof=372 witness=803 UNKNOWN=14 fully_certified=false (sample_parse_failures=0, proof_reverify_failures=0)`: +1 total and +1 proof on `time_unit_glued`, reached only through the SV-only `time_literal` and so provably unreachable under `verilog_2005` exactly as `time_literal` is; witness and UNKNOWN unchanged, deterministic across seeds 0 / 7 / 42), `corpus matrix: 369 file×profile checks, 0 mismatches` unchanged; the gate's first run refused (`seed=0 cert total=1189 (expected 1188)` + STALE), stamped from the three-seed census with a `.3.55` note and confirmed by a second GREEN run that re-stamped the identity; `sv_syntax_closure_gate` GREEN — `reachable_rules: 1645 → 1646 unreachable_rules: 1 unreachable_branches: 2` (the glued unit rule), the allow-list unchanged at ONE (`casting_type`), identity re-stamped; `ast_shape_contract_gate` rc 0; `[ast_shape_contract][systemverilog] samples=57 aligned=57 drift=0 regression_lock_failures=0 aligned_assertion_failures=0 samples_per_status={"aligned": 57}` (no new sample: no template changed — a time literal keeps its node); the gate's FIRST run on the final bytes failed ONE regression lock, `systemverilog_hash_token_is_not_stolen_by_comment_arms_during_speculation` (GRAMMAR-WELLFORMED.H.11.5), whose four samples spelled `timeunit 09 ns` / `/633 s` — spaced time literals the pre-1.0.226 grammar accepted against footnote 44; the samples are respelled `09ns` / `633s` (the lock's subject, the `#` arm during speculation over the comment tail, is unchanged) and the gate is GREEN on the re-anchored lock; `check_generated_reproducibility.sh --rebaseline` — TIER 2, every family re-derives byte-identically from HEAD (`systemverilog` 0 sites and every other family 0 sites), the regenerated parser carrying the glued unit's no-skip atoms; `generated_reproducibility_v0.json` re-stamped: `systemverilog` `parser_sha` `bde02432… → e1e5a9d8…`, `input_sha` `be0d073b… → b49eb8d7…`, `verified_at_commit` `89fb21d3 → ac575805`; `sv_parse_cost_family_share` derived `2.660 %` (`23,503,030 of 883,497,827` entries over 16 336 files; the corpus lost 4.9 M entries) against the carried `2.645` — ADOPTED (`CORPUS_FAMILY_SHARE_PCT` and the four prose anchors; `--verify-family-share` fresh); the rebaseline read every binding metric FALLING — `entries 405,659,946 → 403,243,744 (−0.60 %)`, `committed 6,865,461 → 6,865,447 (−14)`, `memo_hits 176,251,757 → 175,043,676 (−0.69 %)` — attributed per rule (`rule_costs.tsv` HEAD vs promoted, 15 rows moved): each of the 201,348 time-literal attempts on the pinned sample used to enter `time_unit` and then SIX keyword-rule frames (`kw_s_a0f1490a` … `kw_fs_3f4bb586`, −201,348 entries each) with their six `trivia` frames (`trivia` −1,208,088 entries, −1,208,081 memo hits); it now enters ONE `time_unit_glued` frame (+201,348 entries, +7 commits — the seven time literals the sample commits, whose unit / keyword / trivia commits are the −14); promoted on the adopted share (`cost.md` re-derived on the `.3.55` digest, share `2.660`); `sv_parse_cost_ratchet` GREEN on the promoted baseline; the wall clock advisory `46674 ms → 45551 ms (−2.4 %)` within the ±50 % band, machine-dependent;
`parse_harness_semantic_gate` — `2 passed` (`✅ Parse-harness semantic-directive orchestration suite passed`, every suite pin `CLEAN … diverge=0 anchor_miss=0`); `parse_harness_equivalence_gate` — `4 passed` (`✅ Parse-harness differential-equivalence gate passed`: the generated parser and the parse-harness interpreter byte-identical on every CERTIFIED family over the stimuli corpus at seeds 0 / 7 / 42, the regenerated SystemVerilog parser included — the `@glued` decision is read by both through one function); `parse_harness_combinator_gate` — `2 passed` (`✅ Parse-harness structural combinator suite passed`, every combinator pin `CLEAN`).

### 1.0.225 / Contract 1.0.225 — SV-CORPUS-GRAD.3.54 (`PGEN-SV-CORPUS-GRAD-0384`, 2026-09-08), ledger `SV-0098` (`Released`), **schema 32 unchanged**:

⭐ **A package name lives in its own name space.** IEEE 1800-2017 §3.13 unifies package identifiers in a name
space of their own, and A.9.3 `package_scope ::= package_identifier ::` makes `T::VALUE` a package scope whenever
`package T` is declared — a typedef named `T`, wildcard-imported or unit-level, is another name space's business.
Through `1.0.224` the package arm every `pkg::` site reaches carried one gate, "the name is no typedef", so a typedef
of the name refused the scope outright and every class-scope arm refused too (`T` is no class). `1.0.225` makes the
exclusion branch-local and adds a second alternative admitting a DECLARED package by its `package_name` fact under
`@branch_policy: ordered` — tried only when the first alternative fails, its `package_scope` answered from the memo; both alternatives carry the
same template. 1646 rules unchanged; no parser, codegen or engine bytes (the certificate planner's name-gate derivation
is made branch-local beside it, `BRANCH-PREDICATE-LOCALITY.3`).

**Accept-set impact.** `sv_2017` / `sv_2023`: WIDEN by exactly the declared-package-beside-typedef shape — the
corpus census by token finds ONE lane file carrying it (iverilog `sv_type_identifier_package_name.v`), which advances
from the `VALUE` after `T::` to a backtick macro inside `$display` and stays a non-accept; the sweep reads `AST-IDENTITY-SWEEP: profile=sv_2017 files=16336 both_accepted=9767 identical=9767 ast_moved=0 verdict_moved=0 both_nonaccepting=6569`; `AST-IDENTITY-SWEEP: profile=sv_2023 files=16336 both_accepted=9792 identical=9792 ast_moved=0 verdict_moved=0 both_nonaccepting=6544`; `AST-IDENTITY-SWEEP: profile=verilog_2005 files=2606 both_accepted=2310 identical=2310 ast_moved=0 verdict_moved=0 both_nonaccepting=296` — every tree both probes accept is byte-identical and no verdict moved on any lane
(`verdict_moved=0` on `sv_2017`, `sv_2023` and `verilog_2005`; the iverilog row `sv_type_identifier_package_name.v` advances `247 → 331` on both SV lanes and stays a non-accept), the bar `38 → 31` on `sv_2017` (rejects-valid 36 → 29, accepts-invalid 2), `39 → 32` on `sv_2023`, `verilog_2005` unmoved at 43 (`match 6031 → 6037` / `6019 → 6028`). `verilog_2005`: untouched (the rule is SV-only); `AST-IDENTITY-SWEEP: profile=verilog_2005 files=2606 both_accepted=2310 identical=2310 ast_moved=0 verdict_moved=0 both_nonaccepting=296`.
**AST impact — none for any file the parser accepted before.** The package scope keeps its `{kind: "package", name}`
node in every slot.

**The answer key.** Six bar rows re-keyed `must_reject` by production, each with an isolation reproducer and a
control pin: a member select or an indexed member as a `std::randomize` argument (18.12 Syntax 18-11; A.9.3
`variable_identifier_list ::= variable_identifier { , variable_identifier }`), an `extern function` prototype with no
return type in a class and in an interface (A.2.6 `function_prototype ::= function data_type_or_void
function_identifier …`), a bodiless `virtual` method with neither `pure` nor `extern` (A.1.9 `class_method`), and a
trailing comma in a named port-connection list (A.4.1.1). Fourteen pins in all.

**Consumers.** Nothing to change. A `pkg::` scope you never received before arrives as the node the rule has always
carried. Pins `ADJUDICATION-REPROS: checked=1050 armed=219 listed=534 multi_profile_rows=392 failures=0` (was 1022 / 219 / 520 / 378; the fourteen new rows green on the final bytes); union `sv_cert_recognized_union_gate` — the FIRST run on the final bytes REFUSED with the planner flattening the new branch gate: canonical `1499/10/1461/28`, union witness `1482`, union UNKNOWN **7** on every seed (`class_scoped_tf_call`, `extern_constraint_declaration`, `extern_constraint_declaration_sv_2017`, `kw_implements_e133e2cb`, `method_call_receiver_qualifier`, `scoped_base_class_type_identifier`, `scoped_interface_class_type_identifier` — the `pkg::`-witnessed rules); on the branch-local planner (`BRANCH-PREDICATE-LOCALITY.3`) the second run read canonical `1499/10/1474/15` (the IDENTICAL 15-rule UNKNOWN set), union witness `1489`, union UNKNOWN 0, residual `[]`, deterministic across seeds 0 / 7 / 42 — every number 1.0.224's, no rule added or removed — the baseline STALE on `stimuli_generator.rs` and every expectation holding, so the gate re-stamped the identity; `verilog_2005_conformance_gate` — cert `1188/371/803/14` UNCHANGED (`CERTIFICATE-COVERAGE: grammar='systemverilog' entry='systemverilog_file' samples=40 total=1188 proof=371 witness=803 UNKNOWN=14 fully_certified=false (sample_parse_failures=0, proof_reverify_failures=0)`, deterministic across seeds 0 / 7 / 42; the rule is SV-only, so nothing enters or leaves that profile's universe), `corpus matrix: 369 file×profile checks, 0 mismatches`, aliases 2 — GREEN on the first run of the final bytes and again on the branch-local planner, each time STALE on a declared input and every criterion holding, so the gate re-stamped the identity; `sv_syntax_closure_gate` GREEN — `reachable_rules: 1645 unreachable_rules: 1 unreachable_branches: 2` unchanged (1646 rules, no rule added), the allow-list unchanged at ONE (`casting_type`), identity re-stamped twice (on the final bytes, and again STALE on `stimuli_generator.rs` after the planner fix); `ast_shape_contract_gate` rc 0; `[ast_shape_contract][systemverilog] samples=57 aligned=57 drift=0 regression_lock_failures=0 aligned_assertion_failures=0 samples_per_status={"aligned": 57}` (no new sample: no template changed — the package scope keeps its `{kind: "package", name}` node in every slot); `check_generated_reproducibility.sh --rebaseline` — TIER 2, every family re-derives byte-identically from HEAD (`systemverilog` 0 sites and every other family 0 sites), run twice (on the final bytes, and again after the grammar's comment edit moved the input hash); `generated_reproducibility_v0.json` re-stamped: `systemverilog` `parser_sha` `fc9962e0… → bde02432…`, `input_sha` `de4c7c04… → be0d073b…`, `verified_at_commit` `85e2bdbd → 89fb21d3`; `sv_parse_cost_family_share` derived `2.645 %` (`23,500,802 of 888,378,865` entries over 16 336 files) against the carried `2.650` — ADOPTED (`CORPUS_FAMILY_SHARE_PCT` and the four prose anchors in `TOOLBOX.md`, `DOCTRINE_ENFORCEMENT.md`, `docs/book/src/the-enforced-doctrines.md`, `docs/knowledge/a-deterministic-counter-cannot-see-a-per-entry-cost-rise.md`; `--verify-family-share` fresh); the first rebaseline REFUSED two binding rises — `entries 404,943,513 → 405,659,946 (+0.18 %)`, `memo_hits 175,533,047 → 176,251,757 (+0.41 %)`, `committed` conserved exactly — attributed per rule (`rule_costs.tsv` HEAD vs fresh, 44 rows moved): ONE riser, `package_scope` `+721,482` entries = `+721,482` memo hits, committed +0 — wherever alternative 1 FAILS for lack of an `identifier ::` (721,482 of the rule's 731,273 fresh evaluations on the pinned sample) the ordered choice tries alternative 2 and the engine answers its `package_scope` from the memo — and forty-three small fallers (`non_typedef_package_scope` −873 / −1,528 memo, `non_keyword_identifier` −781, `declaration_identifier` −465, `trivia` −395, `class_scope` −299 …) from the memo answering what it used to re-ask; no rule introduced, no sampled file moved its verdict or furthest (the one corpus file the change touches is not in the sample); two rows recorded in `accepted_rises.tsv` under `lookahead_replay_occurrences` (introduced `!package_scope ~non_typedef_package_scope`) with the WHY — a dispatcher helper would enter one frame per attempt (7,232,230 on this sample, ten times this rise), a terminal lookahead would trade one replay for another, the predicate language has no disjunction — and the predicate's docstring, self-test (`arms=16 → 18`) and the book name the SECOND SHAPE: an ordered choice whose alternatives share a prefix replays that prefix from the memo, the same clauses reading the same artefact without a lookahead in the spelling (the `.100` record carries the addendum); the second rebaseline read both rises `ACCEPTED as attributed … under lookahead_replay_occurrences (SV-CORPUS-GRAD.3.54)` with the invariant RE-DERIVED on the run (`` `package_scope` entries +721,482 = memo_hits +721,482, committed +0; every sampled file keeps its verdict and furthest; 0 rule(s) committed more (+0) …; 1 rule(s) rose in entries (+721,482), all reachable from the introduced / un-gated rules; 30 fell (-5,049) — an occurrence-count artefact, not re-done work ``) and PROMOTED (`cost.md` re-derived on the `.3.54` digest, share `2.645`); `sv_parse_cost_ratchet` GREEN on the promoted baseline; the wall clock advisory `44569 ms → 46674 ms (+4.7 %)` within the ±50 % band, machine-dependent.

### 1.0.224 / Contract 1.0.224 — SV-CORPUS-GRAD.3.53 (`PGEN-SV-CORPUS-GRAD-0383`, 2026-09-08), ledger `SV-0097` (`Released`), **schema 32 unchanged**:

⭐ **The parenthesised operator assignment is an expression, hence an operand.** IEEE 1800-2017 A.8.3 makes
`( operator_assignment )` an `expression`, so it stands wherever `expression binary_operator expression` puts an
operand — §11.4.2's own example is `j = i++ + (i = i - 1);`. Through `1.0.223` the grammar admitted the form only at
the top of an expression (`expression_base`), never as a chain operand, so that example rejected at the `(` after
`+` (three bar rows); and the alternative was unprofiled, so `verilog_2005` — whose IEEE 1364-2005 A.8.3 `expression`
has no assignment inside an expression — accepted `j = (i = i - 1);`. Grammar tier: an SV-only
`paren_operator_assignment_operand` reached from every chain step through an ordered pass-through
`expression_chain_operand`, as a chain head with `expression_chain_step+`, and from the index rule; `1644 → 1646`
rules; no engine, no codegen, no Rust.

**Accept-set impact.** `sv_2017` / `sv_2023`: WIDEN by exactly the operand form — the sweep's verdict movers are
the pre-registered rows (ispras `11.04.02_01.sv`, sv-tests `11.3.6--two_assign_in_expr.sv` and `-sim`, `fail → pass` on `sv_2017` and `sv_2023`, `verdict_moved=0` on `verilog_2005`), the bar `41 → 38` on `sv_2017` (rejects-valid 39 → 36, accepts-invalid 2), `42 → 39` on `sv_2023`, `verilog_2005` unmoved at 43 (`match 6028 → 6031` / `6019 → 6022`). `verilog_2005`: NARROWS by the top-level form;
the lane's 2 606 rows carried no such text (`AST-IDENTITY-SWEEP: profile=verilog_2005 files=2606 both_accepted=2310 identical=2310 ast_moved=0 verdict_moved=0 both_nonaccepting=296`).

**AST impact — none for any file the parser accepted before.** The top-level form keeps its `{kind:
"paren_op_assign", body}` node at the base of the expression (the alternative passes the same node through), and
every chain step's triple carries the node it did; `ast_moved=0` on every lane. A newly accepted operand arrives as
`{kind: "paren_op_assign", body}` in an `operand_chain`'s `first` slot or as a step triple's third element. Pins
`control_paren_operator_assignment_operand.sv` / `control_paren_operator_assignment_top_level.sv` /
`invalid_paren_operator_assignment_v2005.sv`.

### 1.0.223 / Contract 1.0.223 — SV-CORPUS-GRAD.3.49 (`PGEN-SV-CORPUS-GRAD-0380`, 2026-09-08), ledger `SV-0096` (`Released`), **schema 32 unchanged**:

⭐ **A store gate that decides no parse is a resolution check Annex A never asked for.** Through `1.0.222` the unscoped
checker name carried a `has_fact(checker_name) phase: post` gate — the 2026-03-21 semantic-fact pilot — so a checker
instantiated BEFORE its declaration (the declaration after the module, as Surelog `CheckerInst/dut.sv` and ispras
`14.14_05.sv` are written) or declared in another file REJECTED at its second actual whenever the actuals were sequence /
property syntax, and `test dut(foo);` inside a `program` (iverilog `program5b.v`) rejected at `test`. IEEE 1800-2017 §3.2
makes a checker a design element, §23.3.2 lets an instantiated name be *"previously declared or one declared later"*, and
A.4.1.4 / A.9.3 derive `checker_instantiation` from a bare identifier. The gate never disambiguated anything: with plain
actuals the tie goes to `module_instantiation` by order, declared checker or not. Grammar tier: the annotation is deleted and, where an
instantiation-shaped alternative competes, a beside-module twin behind `!interface_instantiation` keeps the cost flat (the
ungated first spelling re-read every plain-actual instantiation as property actuals, `committed +33.56 %` on the pinned
sample, refused by the parse-cost ratchet); `1641 → 1644` rules; no engine, no codegen, no Rust.

**Accept-set impact.** `sv_2017` / `sv_2023`: WIDENS by exactly the forward / cross-file checker instantiation — the
sweep's verdict movers are the pre-registered rows (Surelog `tests/CheckerInst/dut.sv`, ispras `ieee-1800-2012/14/14.14_05.sv`, iverilog `ivltests/program5b.v` — `fail → pass` on `sv_2017` and `sv_2023`, 3 / 3 / 0 movers, 0 `pass → fail` anywhere), the bar `82 → 79` on both SV lanes (`unexplained_rejects_valid 80 → 77`, accepts-invalid 2). `verilog_2005`: none (no
checker instantiation in IEEE 1364-2005; pin `invalid_checker_instantiation_v2005.sv` REJECTS).

**AST impact — none for any file the parser accepted before.** `ast_moved=0` on every lane; a newly accepted checker
instantiation arrives as the `{kind: "checker_instantiation", body: {name, instance, connections}}` branch documented
since schema 26. Pins `control_checker_forward_instantiation.sv`, `control_checker_instantiation_undeclared.sv`,
`control_checker_instantiation_in_program.sv`, `invalid_checker_instantiation_v2005.sv`.

### 1.0.222 / Contract 1.0.222 — SV-CORPUS-GRAD.3.50 (`PGEN-SV-CORPUS-GRAD-0379`, 2026-09-08), ledger `SV-0095` (`Released`), **schema 32 unchanged**:

⭐ **A profile-blind arm is an over-acceptance on every profile whose standard lacks it.** Through `1.0.221`
`unpacked_dimension := [ constant_range ] | [ constant_expression ]` carried no `@profiles` on its size arm, so the IEEE
1800 unpacked SIZE form `reg x [2];` PARSED under `verilog_2005` — IEEE 1364-2005 A.2.5 `dimension` / `range` admit
`[ a : b ]` alone (the size form is IEEE 1800-2017 A.2.5 / §7.4.2). Found by `SV-CORPUS-GRAD.3.48`'s own two-sided
control: the 1364-2005 twin of `control_unpacked_dimension_size.sv` was written expecting REJECT and ACCEPTED. Grammar
tier: the size arm lifted into `unpacked_dimension_size_sv_only` under `@profiles: ["sv_2017", "sv_2023"]` with a
pass-through dispatcher alternative (`-> $1`), `1640 → 1641` rules; no engine, no codegen, no Rust.

**Accept-set impact.** `sv_2017` / `sv_2023`: none — `verdict_moved=0` on both lanes. `verilog_2005`: NARROWS by the
size form alone; the lane's 2 606 rows carried no such declaration (the census by token read 0 rows, the exhaustive
sweep on the final bytes confirmed it: `AST-IDENTITY-SWEEP: profile=verilog_2005 files=2606 both_accepted=2310 identical=2310 ast_moved=0 verdict_moved=0 both_nonaccepting=296`).

**AST impact — none.** The size form keeps its `{kind: "expression", body}` node and the range form its
`{kind: "range", body}` node; the corpus AST identity sweep reads `ast_moved=0` on every lane. Pin
`accepts_invalid_v2005_unpacked_dimension_size.sv` flipped `ACCEPT` / `accepts_invalid` → `REJECT` / `invalid` (the
manifest's round trip); `control_unpacked_dimension_size.sv` and `control_packed_dimension_range.sv` unchanged.

### 1.0.221 / Contract 1.0.221 — SV-CORPUS-GRAD.3.47 (`PGEN-SV-CORPUS-GRAD-0377`, 2026-09-08), ledger `SV-0094` (`Released`), **schema 32 unchanged**:

⭐ **The string literal is split by edition, and measured in both directions.** One terminal, `/"([^"\\]|\\.)*"/`, had
served every profile since `1.0.0`, and it was wrong for each standard in a different direction. IEEE 1800-2017 /
1800-2023 §5.9's backslash-newline CONTINUATION — `$display("Humpty Dumpty sat on a wall. \` + newline + `…");`, the
standard's own example — REJECTED at the backslash on every profile, because the regex crate's `.` in `\\.` does not
match a newline (five bar rows: Surelog `AssumeProp`, ispras `05.09_01`, sv-tests `5.9-string-broken-line`, verible
`forbid_line_continuations`, verilator `t_display`); a RAW newline inside a string, which both 1800 editions and IEEE
1364-2005 §3.6 / A.8.8 forbid, parsed on every profile (`[^"\\]` admits it; 0 accepting corpus rows carried one). Grammar
tier: three edition twins under `@profiles`, `1637 → 1640` rules; no engine, no codegen, no Rust.

**Accept-set impact.** `sv_2017` / `sv_2023`: the continuation is admitted; a bare newline inside a string is refused (the
one narrowing — it moves no accepted row). `sv_2023` additionally admits IEEE 1800-2023 §5.9's TRIPLE-QUOTED string
`"""…"""` (any character except `\` inside; escapes as `\` + any character; no look-around in the regex) as
`{kind: "triple_quoted", body}` — verilator `t_display_qqq.v`, an EDITION CONTRAST keyed `must_reject` on the 2017 lane and
`must_accept` on the 2023 lane. `verilog_2005`: unchanged — the 1364-2005 twin refuses both the continuation and the raw
newline, and iverilog `string12.v` (a continuation) is re-keyed `must_reject` there on §3.6 / A.8.8.

**AST impact — none for any string the parser accepted before.** Every double-quoted twin carries the shipped
`{kind: "double_quoted", body}` template, `body` the source text verbatim (quotes excluded; a continuation stays in the
text — §5.9's *ignored* is the consumer's reading), so the corpus AST identity sweep reads `ast_moved=0` on every lane.
Pins `control_string_line_continuation.sv` / `invalid_string_line_continuation_v2005.sv` / `invalid_string_raw_newline.sv` /
`control_string_escapes.sv` / `control_string_triple_quoted_sv2023.sv` / `invalid_string_triple_quoted_under_2017.sv`.

### 1.0.220 / Contract 1.0.220 — SV-CORPUS-GRAD.3.46 (`PGEN-SV-CORPUS-GRAD-0376`, 2026-09-07), ledger `SV-0093` (`Released`), **schema 31 → 32**:

⭐ **The last integer-vector shape without a `kind`, and the nettype alias-head guard's blind spot.** `.3.45` typed the
partial-typing residue and measured two items on its way out: `block_data_type` alternative 1 — the block-scoped integer
VECTOR `reg [7:0] v;` / `logic [3:0] t;` — had been typed by SV-Slice-96 with the field set `{base, signing, dims}` and no
`kind`, the only integer-vector shape in the grammar without one once its four siblings took `data_type`'s kinds in
`1.0.219`; and `nettype wT x with f;` — IEEE 1800-2017 §6.6.7's ALIAS form (a declared-nettype base) carrying a resolution
function, a form A.2.1.3 gives NO production — still parsed down the data-type alternative of both edition twins, because
the `.3.45` alias-head guard required the `;` after `[ scope ] <declared nettype> <identifier>` and `with` is not one.
Grammar tier: one template, one lookahead narrowed; no engine, no codegen, no Rust; `1637` rules unchanged.

**AST impact — schema 31 → 32 (a REPLACEMENT).** The `block_data` body of an integer-vector declaration is
`{kind: "integer_vector", base, signing, dims}` — `data_type`'s own template — where `1.0.219` emitted the same fields
without `kind`. Every other alternative is byte-identical. **Migration:** a consumer that recognised the vector form by
the absence of `kind` reads `kind == "integer_vector"`; a `data_type` reader reads the body unchanged. Pre-registered by
the `.3.45` raw-envelope census on the `1.0.219` probe (**712 nodes in 259 accepting files** on `sv_2017` and `sv_2023`,
**130 in 73** on the 1364-2005 lane) and measured two-sided by the corpus AST identity sweep: every predicted file moved,
no other accepted tree moved a byte, no verdict moved. One AST-shape contract sample (`block_data_type_integer_vector`)
locks the kind; the vector pin's arm is `block_data>integer_vector`.

**Accept-set impact — one LRM-illegal form stops parsing.** `nettype <declared nettype> <name> with <fn>;` REJECTS on both
SV profiles at the `with` (the guard now refuses the alias head whatever follows; the alias alternative meets `with` where
it needs `;`). Every legal form is unchanged and byte-identical: the data-type form with `with` on a typedef base
(`nettype T wT with f;`, scoped `nettype p::T x with p::f;`), the alias form (`nettype wT x;`, scoped), the wildcard-import
alias. 0 corpus rows on any lane (14 `with` declarations in the corpus, every base a typedef). Pins
`invalid_nettype_alias_with_resolution_function.sv` / `…_with_scoped_base_resolution_function.sv` (REJECT) with
`control_nettype_data_type_scoped_with.sv` / `_sv2023.sv` (ACCEPT, the one-token twins).

### 1.0.219 / Contract 1.0.219 — SV-CORPUS-GRAD.3.45 (`PGEN-SV-CORPUS-GRAD-0374`, 2026-09-07), ledger `SV-0092` (`Released`), **schema 30 → 31**:

⭐ **The partial-typing residue is typed.** `1.0.218` closed with the class measured — `PARTIAL-TYPING-CENSUS:` read 38 rules
typed on some alternatives and not others, 19 of the untyped alternatives raw `Sequence` envelopes, in eight rules — and the
corpus census on the `1.0.218` probe (`RAW-ENVELOPE-CENSUS:`, TOOLBOX §1.5h, every lane row) read **2 319 raw envelopes in
1 088 accepting files on `sv_2017`** (1 779 / 819 on `sv_2023`, 217 / 120 on the 1364-2005 lane): 873 of them the block-scoped
integer atom `integer i;` / `int unsigned x;`, 542 the parenthesised clocking event `@(posedge clk)`, 524 the structure-keyed
assignment pattern `'{a: 1}`, 193 the two-value path delay, and so on down to the `case … matches` / `inside` forms and the
`global clocking` block. Grammar tier: seventeen return templates, no engine, no codegen; seven test-only Rust dispatch arms so
the rules take AST-shape contract samples.

**AST impact — schema 30 → 31.** Every newly typed alternative carries a `kind`; every typed sibling keeps its shipped
shape byte-identically, so **`kind` absent is the shipped form**. `assignment_pattern` `{kind: "structure" | "array", keys,
values}` (`keys[i]` pairs with `values[i]`) and `{kind: "replication", count, exprs}`; `block_data_type` takes `data_type`'s
own `{kind: "integer_atom" | "struct_union" | "enum" | "virtual_interface", …}`; `case_statement` `{kind: "matches" |
"inside", unique_priority, keyword, expr, items}` (the `inside` form's `keyword` is the literal `{kind: "case"}`);
`clocking_declaration` `{kind: "global", name, event, end_label}`; `clocking_event_sv_2017` `{kind: "expression", body}` (the
`sv_2023` twin's shape); `list_of_path_delay_expressions` `{kind: "two" | "three" | "six" | "twelve", …}` with the LRM's
position names (`trise`, `tfall`, `tz`, `t01` … `tzx`); `property_case_item` `{kind: "default", body}`;
`net_type_declaration_sv_2017` `{kind: "nettype", scope, base, name}` — IEEE 1800-2017 §6.6.7's ALIAS form, which was not raw
but **shadowed**: both alternatives succeed on the LRM's own `nettype wTsum nettypeid2;` and the longest-match tie went to the
data-type form on both SV profiles. `1.0.219` breaks the tie in the LRM's favour by a negative lookahead on the data-type
alternative — it refuses an alias head `[ scope ] <declared nettype> <identifier> ;` — with the order left as shipped, and
routes the alias base, a bare `declaration_identifier` through `1.0.218`, to `checked_nettype_identifier |
wildcard_escape_nettype_identifier` (the SV-0037 / SV-PARSE-STRICT.2 precedent). Two spellings were refuted first:
reordering the ungated alternative misread `nettype T wT;` (T a typedef) as an alias — the two-sided corpus sweep caught it
on one file — and reordering the gated one broke the certificate's own nettype prelude (the generator renders an unplanned
declaration with the first of two equally cheap alternatives; union `UNKNOWN=2`). The winner moves exactly on alias declarations (two per SV lane in the
corpus); the one narrowing is the LRM-illegal `nettype undeclared_t x;` with no wildcard import, accepted through `1.0.218`
and now rejected (0 accepting corpus rows, pinned).

**Two non-LRM alternatives removed.** `list_of_path_delay_expressions` carried seven alternatives since the grammar's first
promotion where every edition gives five: a second six-value spelling that lost the tie to the LRM's on every input
(Protocol D: `selected branch 4/7`) and a twelve-value spelling whose middle positions were the literal text
`tx_path_delay_expression`; both gone with their keyword terminal, and the three ungated nettype identifier twins with them: `1641 → 1637` rules.

**Verification.** 30 pins (`control_assignment_pattern_*`, `control_block_data_type_*`, `control_case_*`, `control_clocking_*`,
`control_path_delay_*`, `control_nettype_*` incl. the two wildcard-import alias controls, `control_property_case_*` — one per
alternative, the typed siblings as the controls that cannot move, arms on the new `kind`s — plus the `must_reject`
`invalid_nettype_alias_undeclared_base.sv`); 16 AST-shape contract samples (40 → 56); the interpreter A/B on the
nineteen reproducers and a seeded corpus sample (87 arms, two-sided clean); `PRESCREEN-PINS: jobs=781 verdict_moved=0
ast_moved=38` two-sided; the three lanes' verdicts and the corpus AST identity sweep against the census's movers are in the
integration contract's `1.0.219` note. See [Typed Partial Alternatives](json-carrier.md#typed-partial-alternatives-schema-31).

### 1.0.218 / Contract 1.0.218 — SV-CORPUS-GRAD.3.44 (`PGEN-SV-CORPUS-GRAD-0373`, 2026-09-07), ledger `SV-0091` (`Released`), **schema 29 → 30**:

⭐ **`function_body_declaration` and `task_body_declaration` are typed on EVERY alternative.** IEEE 1800-2017 A.2.6 / A.2.7
give a function or task body two forms — the non-ported `type f; …` whose ports are `tf_item_declaration` items inside the
body, and the PORTED `type f(ports); …` with `block_item_declaration` items — each with an optional out-of-block prefix
(`I.f` / `C::f`), four alternatives in the grammar. Through `1.0.217` only alternative 1 carried a return template (since
SV-Slice-25, release `1.0.25`, and every register that counts annotations counted the rule as typed from that day); the
other three arrived as the raw positional `Sequence` envelope — 10 / 8 / 11 elements on a function, 9 / 7 / 10 on a task.
The corpus census on the 1.0.217 probe (`AST-FIELD-CENSUS:` with the new `--select-keys lifetime,body` selector +
`TF-BODY-CLASSIFY:`) read **2 211 of the 3 411 function / task bodies the parser wrote on `sv_2017` as that envelope** —
1 646 ported functions, 513 ported tasks, 30 out-of-block functions, 22 out-of-block tasks — in 1 060 of the 1 958
accepting files that declare a function or task at all. The ported form is the most common function in any corpus, and
the typed AST could not read it. Grammar tier: six return templates, `1641` rules unchanged, no engine, no codegen; two
test-only Rust dispatch arms so the body rules take AST-shape contract samples.

**AST impact — schema 29 → 30.** Every alternative returns the typed object, mirroring alternative 1's field set:
`{return_type, name, ports, items, statements, end_label}` (ported), `{return_type, scope, name, items, statements,
end_label}` (out-of-block), `{return_type, scope, name, ports, items, statements, end_label}` (out-of-block ported); the
task twins without `return_type`. There is no `kind` — the discriminator is key presence, which is the standard's own
distinction: `ports` is present exactly on the ported alternatives (the typed `tf_port_list` array, `[]` for `f()`),
`scope` exactly on the out-of-block ones (the `out_of_block_tf_scope` node — the pair `[<interface_identifier>, {kind:
"dot"}]` or the `class_scope` node). `items` holds `tf_item_declaration` nodes on the non-ported forms and bare
`block_item_declaration` nodes on the ported ones, as A.2.6's two productions read. **Migration:** a consumer that indexed
`body[N]` reads the named fields — the same typed nodes in the same order, the `(` `)` `;` `endfunction` tokens dropped
as on alternative 1. Alternative 1 is byte-identical (the two in-block pins on all three profiles; every all-typed
corpus file). A REPLACEMENT of a raw envelope by a typed node — the `SV-0014`…`SV-0020` precedent — hence the bump.

**Accept set — unchanged, measured.** `sv_2017` `transitions: NONE` (pass 9756 / fail 6580 of 16336 rows, 0 timeouts); `sv_2023` `transitions: NONE` (pass 9780 / fail 6556 of 16336 rows, 0 timeouts); `verilog_2005` `transitions: NONE` (pass 2310 / fail 296 of 2606 rows, 0 timeouts). **Typed AST — moved exactly where predicted.** The corpus AST identity sweep (the 1.0.217 probe vs this release, TOOLBOX §1.5e) reads `AST-IDENTITY-SWEEP: profile=sv_2017 files=16336 both_accepted=9756 identical=8696 ast_moved=1060 verdict_moved=0 both_nonaccepting=6580` and `AST-IDENTITY-SWEEP: profile=sv_2023 files=16336 both_accepted=9780 identical=8716 ast_moved=1064 verdict_moved=0 both_nonaccepting=6556`, and over the `verilog_2005` lane `AST-IDENTITY-SWEEP: profile=verilog_2005 files=2606 both_accepted=2310 identical=2270 ast_moved=40 verdict_moved=0 both_nonaccepting=296` — two-sided against the census's prediction (`sv_2017`: predicted 1060, moved 1060, predicted-but-identical 0, moved-but-not-predicted 0; `sv_2023`: predicted 1064, moved 1064, predicted-but-identical 0, moved-but-not-predicted 0; `verilog_2005`: predicted 40, moved 40, predicted-but-identical 0, moved-but-not-predicted 0). The
final-probe census re-reads every body typed (`sv_2017` nodes 3411 → 3411, typed 1200 → 3411, RAW 2211 → 0, unclassified 0; `sv_2023` nodes 3521 → 3521, typed 1300 → 3521, RAW 2221 → 0, unclassified 0; `verilog_2005` nodes 470 → 470, typed 379 → 470, RAW 91 → 0, unclassified 0). Pins `ADJUDICATION-REPROS: checked=781 armed=149 listed=407 multi_profile_rows=279 failures=0` (eight new `control_tf_body_*.sv`,
one per alternative of each rule, the two in-block ones the controls that cannot move, arms `tf_port` / `!tf_port`
pinning WHERE the ports live); six AST-shape contract samples parsed AS the body rules; union certificate and 1364-2005
conformance union `1494/10/1469/15 + 1484/0` and 1364-2005 `1186/370/802/14` both UNCHANGED and GREEN, deterministic across seeds 0/7/42, identities re-stamped.

⭐ **The class is censused and owned.** A rule is typed when EVERY alternative carries a template, and nothing censused the
alternatives: the new `PARTIAL-TYPING-CENSUS:` (TOOLBOX §1.5g) reads every multi-alternative rule typed on some alternatives
and not others — 40 rules with 25 raw-envelope alternatives on 1.0.217, 38 / 19 after this release. The nineteen
(`assignment_pattern`, `block_data_type`, `case_statement`'s `matches` / `inside` forms, `clocking_declaration`,
`clocking_event_sv_2017`, `list_of_path_delay_expressions`, `net_type_declaration_sv_2017`, `property_case_item`) are
`SV-CORPUS-GRAD.3.45`. See the schema-30 row of [Schema Versioning](schema-versioning.md#schema-version-timeline) and the
json-carrier's [Typed Function and Task Bodies](json-carrier.md#typed-function-and-task-bodies-schema-30).

### 1.0.217 / Contract 1.0.217 — SV-CORPUS-GRAD.3.39 (`PGEN-SV-CORPUS-GRAD-0372`, 2026-09-07), ledger `SV-0090` (`Released`), **schema 28 → 29**:

⭐ **The thirteen compiler directives IEEE 1800-2017 clause 22 gives a syntax box are TYPED productions, and the parser refuses what
the boxes refuse.** Through 1.0.216 a tolerated directive line — at file top level and, since 1.0.178, at the in-scope item positions
— was ONE opaque token, so `compiler_directive.body` was the line as a string (`"`timescale 1ns / 1ps"`) and `` `timescale frump ``,
`` `unconnected_drive pull2 ``, `` `line 1 "f" 3 ``, a bare `` `pragma ``, `` `default_nettype supply0 `` and `` `begin_keywords "VAMS-2.3" ``
all parsed. Every clause-22 box is captioned *"not in Annex A"*, which is why no Annex-A-driven release ever transcribed one; this
release transcribes each — `` `resetall `` (§22.3), `` `undef `` / `` `undefineall `` (§22.5.2–3), `` `timescale `` (§22.7),
`` `default_nettype `` (§22.8), `` `unconnected_drive `` / `` `nounconnected_drive `` (§22.9), `` `celldefine `` / `` `endcelldefine ``
(§22.10), `` `pragma `` (§22.11), `` `line `` (§22.12), `` `begin_keywords `` / `` `end_keywords `` (§22.14) — box for box (IEEE
1364-2005 clause 19 reads the same; IEEE 1800-2023 Syntax 22-10 adds `"1800-2023"`, one profiled version token per edition).
`1593 → 1641` rules; grammar tier only, no engine, no codegen.

**AST impact — schema 28 → 29.** `compiler_directive.body` is a typed node whose `kind` is the directive's name —
`{kind: "timescale", unit: {value, unit: {kind}}, precision: {…}}`, `{kind: "default_nettype", value: {kind}}`,
`{kind: "unconnected_drive", value: {kind: "pull0" | "pull1"}}`, `{kind: "undef", name}`, `{kind: "pragma", name, expressions}`,
`{kind: "line", number, filename, level}`, `{kind: "begin_keywords", version}`, and the six bare kinds — at both containers alike
(`source_text_item` and the `non_port_module_item` / `class_item` wrappers); every other backtick line the parser tolerates at file
top level is `{kind: "opaque", text}` carrying the old string. **Migration:** switch on `body.kind`; read `body.text` for `"opaque"`.

**Accept set.** NARROWS on exactly the illegal forms — measured over the corpus by the parser itself before a byte was generated
(`AST-FIELD-CENSUS:` + the new `DIRECTIVE-CLASSIFY:`): 26 rows `pass → fail` on `sv_2017`, 23 on `sv_2023`, 0 on `verilog_2005`,
each pre-registered, keyed `must_reject` with the clause or routed to the Verilog-AMS lane. WIDENS on one form: white space —
newlines and comments — between a directive's arguments is §5.3 white space (only §22.12 binds `` `line `` to its line), so
iverilog's GitHub-782 family (`br_gh782c/d/e/f.v`: `` `unconnected_drive /* … */ `` ⏎ `pull1`) goes `fail → pass` on all three
lanes. Three readings a consumer should know: a bare-name `` `pragma `` is bounded to its line (`@branch_policy: ordered` — the
grammar's default longest-match tournament would let the expression arm eat the next line's first identifier); the magnitude of a
`` `timescale `` argument is syntax (`1`, `10`, `100`) while `timeunit 2ps;` stays derivable through Annex A and the precision
ordering is a value rule not judged; `` `begin_keywords "1800-2023" `` parses under `sv_2023` only, and under `verilog_2005`
`` `undefineall `` is an `opaque` text-macro use. Forty-one pins (`invalid_directive_*.sv`, `control_directive_*.sv`). See the
schema-29 row of [Schema Versioning](schema-versioning.md#schema-version-timeline) and the json-carrier's
[Typed Compiler Directives](json-carrier.md#typed-compiler-directives-schema-29).

### 1.0.216 / Contract 1.0.216 — SV-CORPUS-GRAD.3.43 (`PGEN-SV-CORPUS-GRAD-0371`, 2026-09-06), ledger `SV-0089` (`Released`), schema **28 unchanged**:

⭐ **IEEE 1800-2017 Annex A footnote 21 / §35.5.5 is grammar: a DPI import function's RESULT type is one of `void`,
`byte`, `shortint`, `int`, `longint`, `real`, `shortreal`, `chandle`, `string`, or a SCALAR `bit` / `logic`.** A.2.6
`dpi_function_proto21 ::= function_prototype` carries footnote 21 — *"dpi_function_proto return types are restricted to
small values, per 35.5.5"* — and §35.5.5 (`docs/systemverilog/2017/md/section-35-direct-programming-interface.md:485-493`)
lists the types; 1800-2023 reads the same (`:493-501`, footnote 26). Through 1.0.215 the production was transcribed
without its footnote, so `import "DPI-C" function logic [7:0] f();`, `bit [1:0]`, `bit []`, `reg`, `integer`, `time`,
`realtime`, an enum / struct / union literal, `event`, a virtual interface, a parameterised class type and a typedef
name with packed dimensions all parsed as DPI results. Now `dpi_function_proto` reaches `dpi_function_prototype`, a
caller split of `function_prototype` (the `.3.31` shape — class-method, extern and pure-virtual prototypes are
untouched) whose return slot is `dpi_function_result_type`: the `data_type_or_void` shape restricted to §35.5.5,
alternative for alternative and template for template (1587 → 1593 rules, six new; grammar tier
only, no engine, no codegen; every new rule `@profiles: ["sv_2017", "sv_2023"]`, so `verilog_2005` is untouched by
construction). A NAMED type without packed dimensions (a typedef, a package- or class-scoped type identifier, a
`type(…)` reference) is admitted exactly as `data_type` admits it — what it resolves to is semantic; signing on the
atom and vector types is admitted (the subclause is silent on it); the export half of the sentence is not grammar.

**AST impact — none (schema 28 unchanged).** `dpi_result_data_type` carries `data_type`'s own templates, so every legal
import's typed tree is BYTE-IDENTICAL to 1.0.215: measured under the interpreter on 38 synthetic probes × 2 SV profiles
and on the two control pins, and on the final bytes by the HEAD-vs-release AST identity sweep over 16 335 corpus files
× 2 SV profiles (`ast_moved=0 verdict_moved=2` on each profile — 9 778 / 9 799 accepted trees byte-identical, the two verdict movers the pre-registered files). A consumer reading `import_function.proto.body.body.return_type` sees the
`data_type_or_void` shapes it already knows; the refused forms never reach it, because the file no longer parses.

**Corpus effect — exactly two rows, both pre-registered by name.** A new instrument, the AST-field census
(`AST-FIELD-CENSUS:`, TOOLBOX §1.5f — the parser's own reading of one typed slot over a file list, with a tagged regex
fallback for the files it rejects), read **1 298** DPI result slots in the 172 corpus files carrying a DPI spec string
on the 1.0.215 probe: `legal_keyword=1210 named=64 ILLEGAL=24 unclassified=0`; the 24 illegal slots sit in four files,
two of them accepting — sv2v `test/nosim/dpi.sv:35` `bit [15:0] getStimulus` and verilator
`test_regress/t/t_dpi_result_type_bad.v` (sixteen results, `:80-83` / `:104` / `:107-117`). On the final bytes
`sv_2017` and `sv_2023` each read exactly two rows `pass → fail` (the two pre-registered files) and 0 `fail → pass` (pass 9 780 → 9 778 / 9 801 → 9 799, 0 timeouts), `verilog_2005` `transitions: NONE`. ⭐ **The sv2v line is the LRM's OWN example** (§35.5.4 `:476`; 1800-2023 `:489` repeats it) — it
contradicts the normative §35.5.5 one paragraph later and the Annex A footnote, and the next edition kept the
restriction beside the unchanged example, so the example is the erratum and the normative text wins (the reverse of
the `.3.34` rule; recorded on its knowledge card). Both rows and verilator's `t_dpi_import.v` (five unconditional packed
results, already rejecting at its `ifdef` preamble) are keyed `must_reject` with the production in the adjudicator's
`EXTRA_PINNED`; the adjudicator reads `sv_2017` match 5 962 · unexplained 99 · explained 1 494, `sv_2023` match 5 954 · unexplained 99 · explained 1 502 (the bar 99).

**Certificates and gates on the final bytes.** Union certificate `1448/8/1425/15 + 1440/0` (total 1443 → 1448: the five sv_2017-visible new rules (the sv_2023-only twin is outside the sv_2017 universe); proof 8 unchanged; canonical witness 1420 → 1425 — the five new rules are witnessed from the canonical entry itself, exactly as `dpi_function_proto` is; canonical UNKNOWN 15 → 15 (the same fifteen rules as at 1.0.215); union witness 1435 → 1440, union UNKNOWN 0, residual `[]`, the six configs unchanged, identical at seeds 0 / 7 / 42); 1364-2005
certificate `1142/378/750/14` unchanged; syntax closure GREEN (defined 1556 → 1562, reachable 1592, `unreachable_rules=1`, the allow-list ONE); shape contract 18/18; reproducibility
re-derived byte-identically; parse cost: `entries` -1,472 and `memo_hits` -512 PROMOTED (falls), `committed` conserved exactly at 6 741 854 — the only entries risers are the five new rules (+32 each), 39 pre-existing rules fell, the LR-family share `2.661 %` reproduces to the anchor.

**Pins and instruments.** Fourteen `invalid_dpi_result_*.sv` REJECT pins — one per refused alternative (`packed_logic`,
`packed_bit` — the LRM's own line, `open_bit` — the text 1.0.214's union certificate witnessed the open packed dimension
with, `reg`, `integer`, `time`, `realtime`, `enum`, `struct`, `union`, `event`, `virtual_interface`, `class_type`,
`typedef_packed`), each ACCEPT on 1.0.215 → REJECT on 1.0.216 at a named byte; `control_dpi_result_types.sv` (every
admitted form, twelve `kind`-chain arms no formal can satisfy, byte-identical) and `control_dpi_result_types_sv2023.sv`
(the 2023 dynamic-override slot on a DPI prototype). NEW `docs/tasks/artifacts/sv_corpus_grad/ast_field_census/`
(`AST-FIELD-CENSUS:`, `--self-test`) and `…/dpi_result_census/` (`DPI-RESULT-CLASSIFY:`, `census.md`); pins ratchet
`ADJUDICATION-REPROS: checked=649 armed=106 listed=358 multi_profile_rows=233 failures=0`. Owed, named: `t_dpi_result_type.v:114-115` (two packed results under `ifndef NO_ARRAY`) to that row's
svpp-divergence resolution; the byte-exact pinning of a nested typed shape to `ENGINE-UNIVERSAL-SERVICES.97`.

### 1.0.215 / Contract 1.0.215 — SV-CORPUS-GRAD.3.42 (`PGEN-SV-CORPUS-GRAD-0370`, 2026-09-06), ledger `SV-0088` (`Released`), **schema 27 → 28**:

⭐ **Sixteen inline alternations `( a | b )` that sat inside groups of annotated rules are removed — fifteen lifted into eight named,
unannotated helper rules, the sixteenth respelled as its alternative's only live form — because spelled inline the group's branch N wore the ENCLOSING rule's return template N
wherever one existed — and the shipped parser emitted the host's template wrapped around a scope prefix in ACCEPTING
files.** The mechanism is `ENGINE-UNIVERSAL-SERVICES.96` (found by release 1.0.213 on a chain-step spelling and
avoided there by the same lift); its textual census had named three multi-alternative hosts. Two five-line synthetics
widened the class before this slice touched the grammar — a ONE-template host shadows branch 0, and an UNANNOTATED
alternative of an annotated host keeps the corrupted element in its raw envelope — and the mechanical census
(`INLINE-ALT-CENSUS:`, TOOLBOX §1.5d) then read **16 sites in 12 rules** on 1.0.214: `constraint_primary_sv_2017` /
`_sv_2023`, `hierarchical_btf_identifier`, `method_call_receiver_sv_2017` / `_sv_2023` (two sites each),
`nonrange_variable_lvalue`, `ps_or_hierarchical_array_identifier`, `ps_type_identifier_sv_2017` / `_sv_2023`,
`split_hierarchical_callable_receiver`, `function_body_declaration` and `task_body_declaration` (two each). Measured on
ordinary SystemVerilog: `foreach (this.arr[i])` → `array.scope = {name: {kind: "dot"}, scope: {kind: "this"}}`;
`solve this.x before this.y;` / `disable soft this.x;` / `this.arr = new[4];` → `scope: {…, select: "<invalid_sequence_access>"}`
(4 such sentinels on `sv_2017`, 7 on `sv_2023`, in one accepting file); `covergroup cg @@(begin C::m)` with class `C`
declared → `scope: {kind: "block", body: <class_scope>}`; `function void I.h();` → six `<invalid_sequence_access>` in the
raw envelope of `function_body_declaration` alternative 3. The release probe reproduced the btf node byte for byte with
the interpreter (the codegen side `.96` had left unmeasured). `1579 → 1588` rules; grammar tier only, no engine, no
codegen; every helper is `@profiles: ["sv_2017", "sv_2023"]` because every branch is SV-only syntax, so `verilog_2005`
is untouched by construction.

**AST impact — schema 27 → 28 (a REPLACEMENT of corrupted nodes by correct ones).** Every affected `scope` /
`qualifier` / `prefix` field now carries the prefix as its own node: `this .` is the pair `[{kind: "this"}, {kind: "dot"}]`,
a class scope is the `class_scope` node, a package scope the `package_scope` / `non_typedef_package_scope` node, an
interface prefix the pair `[<interface_identifier>, {kind: "dot"}]`. **Byte-identical everywhere else**, measured rather
than argued: over 16 335 corpus files × 2 SV profiles the HEAD-vs-release AST identity sweep reads `ast_moved=4 verdict_moved=0` on each profile (9 776 / 9 797 accepted trees byte-identical), the four movers being exactly the shadowed-branch files the liveness census predicted (ispras `25.07.03_01.sv`, `25.07.04_01.sv`, verilator `t_modport_export_bad.v`, `t_cast_string.v`), each differing only in its scope element (24 `<invalid_sequence_access>` sentinels gone); The non-shadowed branches (`pkg::`, `C::`), every absent optional and the class-scope
out-of-block form `function void C::g();` do not move; 610 pin × profile arms under the interpreter `verdict_moved=0
ast_moved=0`; the 393-file seeded corpus pre-screen moved **0** verdicts and **0** furthest positions over 786 arms on both the lifted candidate and the final grammar (`sv_2017` 53 accept/accept + 340 reject/reject, `sv_2023` 54 + 339). Migration: see the schema-28 row of
[Schema Versioning](schema-versioning.md#schema-version-timeline) and the json-carrier rows for the eight helpers and `hierarchical_btf_identifier`.

**No corpus verdict moves — by construction and by measurement.** A lift changes no accepted language: all three lanes on the final bytes read `transitions: NONE` (`sv_2017` 9 780 / 6 556, `sv_2023` 9 801 / 6 535, `verilog_2005` 2 306 / 300; 0 timeouts) — 0 rows moved in either direction. ⭐ **AND ONE `verilog_2005` VERDICT NARROWS, on invalid text, as a side effect worth naming**: the `class_qualifier` literal (IEEE 1800 Annex A's NONTERMINAL, extracted as a keyword — `SV-CORPUS-GRAD.13c.2m`) reached the IEEE 1364-2005 profile through `split_hierarchical_callable_receiver`'s inline group, whose host carries no `@profiles`; the lifted `split_callable_receiver_scope` is SV-gated, so `module m; initial class_qualifier a.b(); endmodule` — the `V2005-KEYWORD-FAITHFULNESS` census's witness — goes `parse_full passed` → `furthest_position=33` under `verilog_2005` (HEAD probe vs new), and the census reads **`over_acceptances 1 → 0`, `gated 24 → 25`** — the last confirmed 1364-2005 keyword over-acceptance closed; the corpus spells the literal in 0 files (the `verilog_2005` lane moved 0 rows), and the PRIMARY-position form `y = class_qualifier m;` is unchanged on every profile (`sv_2017` still the `.13c.2m` accepts-invalid, `verilog_2005` REJECT at 648 before and after).
Liveness of the lifted rules on the corpus (`LIVENESS-CENSUS:`, TOOLBOX §1.5e): of the eight helpers, three commit on **0** corpus files (`method_call_receiver_qualifier`, `implicit_handle_or_class_scope`, `nonrange_lvalue_scope`), and the four that commit (`ps_type_identifier_scope_*` 14 files, `split_callable_receiver_scope` 13, `out_of_block_tf_scope` 34 accepted, `ps_or_hierarchical_array_scope` 1) do so almost entirely through never-shadowed `pkg::` / `C::` branches — exactly **4** accepted files spell a shadowed branch (`t_cast_string.v`'s `foreach (this.m_counters[i])`, the three interface out-of-block definitions), and those are precisely the trees the sweep names.

**The sixteenth site.** `hierarchical_btf_identifier`'s third alternative is A.9.3's `[ hierarchical_identifier . | class_scope ] method_identifier`; in this grammar's longest-match tournament the dotted and bare forms always lose the tie to alternatives 1 and 2 (`@@(begin top.m)` → `tf`, `@@(begin m)` → `block`), so the alternative is spelled `class_scope method_identifier` — the only form that ever won, and the one that was corrupted. A first spelling lifted the group into a helper rule and the union certificate became SEED-DEPENDENT on it (witnessed at seeds 0 and 7 through a `coverage_event` entry's diverse pass, not at 42, because the helper's dotted branch is dead by tie and its class-scope branch is store-gated for the targeted plan); the dead forms are dropped instead, and the certificate is deterministic again.

**Instruments.** NEW `docs/tasks/artifacts/sv_corpus_grad/inline_alt_census/inline_alt_census.py` (the class census,
`--self-test` over seven tracked synthetics), `…/inline_alt_lift/ast_identity_sweep.py` and `liveness_census.py` (did
any accepted tree move between two parser binaries; which files make a rule commit); the tracked grammar-candidate
pre-screen scripts are runnable from their tracked location for the first time (they derived the repo root one level
short). Four control pins; the btf pin's arm `!block_event>block` fails on the 1.0.214 tree and holds on 1.0.215.

Residue: `.3.43` (footnote 21, DPI result types), `.3.39` (typed directive arguments), NEW `.3.44` (type the untyped
function/task body alternatives, whose raw envelope is where the interface corruption lived); the engine half and the
other families' census counts (`ebnf` 35, `semantic_annotation` 14, `vhdl` 2, `rtl_frontend` 1) are
`ENGINE-UNIVERSAL-SERVICES.96`; a byte-exact nested-shape pin is `.97`. The bar stands at **99**, unmoved (this release corrects trees, not verdicts).

### 1.0.214 / Contract 1.0.214 — SV-CORPUS-GRAD.3.38 (`PGEN-SV-CORPUS-GRAD-0369`, 2026-09-05), ledger `SV-0087` (`Released`):

⭐ **IEEE 1800-2017 Annex A footnote 20 is grammar: an unsized PACKED dimension `[]` is legal only as the SOLE
packed dimension of a formal in a DPI IMPORT declaration.** A.2.5 `packed_dimension20 ::= [ constant_range ] |
unsized_dimension` carries footnote 20 — *"In packed_dimension, unsized_dimension is permitted only as the sole
packed dimension in a DPI import declaration; see dpi_function_proto and dpi_task_proto"* — and §35.5.6.1 adds that
exported functions cannot have open arrays and that *"an unsized dimension shall be the sole packed dimension of a
formal argument"*; 1800-2023 keeps both verbatim (footnote 24). Until 1.0.213 the grammar took the unsized arm of
`packed_dimension` at every packed site: `reg [] illegal;` parsed at module level (iverilog `br_ml20181012b.v`,
Surelog `UnsizedPacked/dut.sv`), as did `input [] p` on an implicit-type port and `logic [7:0] [] x` /
`logic [] [7:0] x` inside an import. Now the import's DPI spec string opens a `dpi_import` scope and the
declaration's `;` closes it (two marker rules — the mechanism release 1.0.206 introduced for the cross body — placed
on the spec string because a marker on the shared `import` keyword would open a scope at every package import), the
unsized arm is reachable only through `dpi_open_packed_dimension` (gated `in_scope_kind(dpi_import)` as a `post`
predicate, refusing a following `[`), and the range arm refuses a following `[]`. `1576 → 1579` rules. Grammar tier
only; `verilog_2005` untouched (IEEE 1364-2005 has no unsized dimension — `SV-0033` gated it out of that profile).

**AST impact — none.** The scope markers bind the elements the annotations already bind (`spec: $2`) or discard
(the `;`), and the gated arm passes `unsized_dimension`'s node through, so a packed `dims` entry still reads
`{kind: "unsized", body: {kind: "unsized"}}` — now only inside a DPI import. Every DPI open array of §35.5.6.1's own
list, a task import, an unnamed formal (footnote 23) and both export forms parse with a typed AST byte-identical to
1.0.213 (interpreter AST JSON, HEAD grammar vs new); 593 pin × profile arms and 174 corpus candidates moved no verdict
and no typed AST beyond the two rows below. Schema **27 unchanged**. The accept set NARROWS on `sv_2017` and `sv_2023`
by exactly the footnote: an unsized packed dimension outside a DPI import, or beside a sized one.

**Corpus proof**, per path, three lanes: `sv_2017` exactly TWO rows `pass → fail` — iverilog `br_ml20181012b.v` (leaves the bar, `unexplained_accepts_invalid → match`) and Surelog `UnsizedPacked/dut.sv` (`match` under its corrected key) — **0** `fail → pass`, pass `9 782 → 9 780` (16 336 rows, 0 timeouts), match `5 961 → 5 962`, unexplained **`100 → 99`**; `sv_2023` the same two rows, pass `9 803 → 9 801`, match `5 953 → 5 954`, the Surelog ruling carried over mechanically (`packed_dimension:IDENTICAL variable_dimension:IDENTICAL`); `verilog_2005` **0** rows moved; the edition census `2017_only_pass=0`. Pins `checked=610 armed=97 listed=338 multi_profile_rows=214 failures=0` (release probe `788d7ade08d1301c…`, parser `48d68d913dee646b…`; the four `invalid_*` pins REJECT, the four controls ACCEPT with their arms — `import_function>integer_vector>unsized` and `import_task>integer_vector>unsized` PRESENT on the DPI control and `integer_vector>unsized` ABSENT on the unpacked control, so the chain discriminates the packed `[]` from the unpacked one). Gates on the final bytes: `sv_cert_recognized_union_gate` GREEN — canonical `1436/8/1414/14` (identical at seeds `[0,7,42]`, `sample_parse_failures=0`, `proof_reverify_failures=0`), union witness `1428` / UNKNOWN `0`, residual `[]`, re-stamped (24 m 34 s under the guard, peak 6 735 MB); the canonical `+1` UNKNOWN is `dpi_open_packed_dimension`, scope-excluded by construction — the planner routes it through a module-level `specparam []` (`[plannable-probe] … parsed=false witnessed_target=false sample="module m(input wire a);specparam[]\\foo =7413.78;endmodule"`, rendered correctly and refused by the scope gate) — and a SIXTH union config `dpi_import_export:sv_2017` witnesses it from the import declaration itself (`sample="import\"DPI-C\"function bit[]\\foo ;"`); expectations PRE-STAMPED from the seed-0 census (five configs: union UNKNOWN `1` = `[dpi_open_packed_dimension]`; six: `0`) and confirmed by the gate first time; `verilog_2005_conformance_gate` GREEN — cert `1142/378/750/14` UNCHANGED (the three new rules are SV-gated, the shape a profile-only narrowing must have; identical at seeds `[0,7,42]`, `spf=0 prf=0`), lint orphans 0, corpus 363 checks / 0 mismatches, aliases 2, re-stamped (32 m 27 s, peak 10 303 MB); ⚠️ both gates re-emitted the parser and rebuilt the binaries: the release probe (`788d7ade…`) and the generated parser (`48d68d91…`) came back BYTE-IDENTICAL to the binaries the pins and lanes were keyed on (`rust/target/s338/sha_after_build.txt` vs `sha_after_certs.txt`; the debug `ast_pipeline` did not, `ebc1d186… → 2e132cce…`, which nothing downstream is keyed on); `sv_syntax_closure_gate` GREEN — `defined_rule_count 1545 → 1548`, `reachable_rules 1575 → 1578`, `unreachable_rules=1` (the allow-list ONE, `casting_type`), `unreachable_branches=2`, re-stamped (1 m 40 s, peak 1 916 MB); `ast_shape_contract_gate` GREEN — `18 passed; 0 failed` (3 m 12 s, peak 9 375 MB); `check_generated_reproducibility.sh --rebaseline` TIER 2 — every family re-derives byte-identically, baseline rewritten (1 m 25 s, peak 2 011 MB); `check_corpus_key_integrity.sh` OK (499 keyed rows, 0 contradictions; provenance 716 clause-cited); `check_identity_carriers.sh` OK (`live=12 frozen=7 rows_equal=51`); `check_sv_corpus_denominator.sh` OK (`7556/2606/6174/4016/99`); `report_family_status.sh --check-all` OK (11 blocks equal a fresh derivation after re-splicing the cross-family and SystemVerilog pages); `report_grammar_certification.sh --markdown` re-spliced (`1436 rules · 1428 witness · 8 proof · 0 unknown (canonical unknown 14)`); dark audit + worklist re-derived, `WORKLIST 53` unchanged.

**Answer key.** Surelog `tests/UnsizedPacked/dut.sv` — keyed `must_accept` by a golden log that completes because
Surelog's parser takes the unsized arm at every site and its elaboration never checks the footnote (the unit names
the variable `illegal`) — is re-keyed `must_reject` with the production and the footnote in both editions; the
iverilog twin `br_ml20181012b.v` was already keyed so on the same footnote, and the two rows now agree.

Residue: `.3.42` (the three optional-group receiver spellings, sequenced before the release), `.3.39` (typed
directive arguments); the bar stands at 99.

### 1.0.213 / Contract 1.0.213 — SV-CORPUS-GRAD.3.41 (`PGEN-SV-CORPUS-GRAD-0368`, 2026-09-04), ledger `SV-0086` (`Released`), **schema 26 → 27**:

⭐ **The set-membership operator `inside { … }` is a chain element of the operand chain, and an inside expression
can be the LEFT operand of a binary operator.** Until 1.0.212 `inside_expression := expression_base inside { … }`
wrapped its left operand and was reachable only at the top of `expression`, so `y = a inside {2'b10, 2'b11} & ~b;`
and `… && b` REJECTED (`furthest_position=630`, the `&` after the `}`; Cores-VeeR-EL2 `testbench/ahb_lite_2to1_mux.sv`
and verilator `t_randomize_method_constraints.v:14` `x inside {1,2} || x inside {4,5};`), while `(a inside {…}) & ~b`
and `b & a inside {…}` accepted — and the accepted one was mis-grouped: its tree read `(b & a) inside {…}`, against
IEEE 1800-2017 Table 11-2 (11.3.2), which puts `inside` on the relational row above `==`, `&`, `^`, `|`, `&&`, `||`.
`expression_base`'s chain is now `expression_operand expression_chain_step*`, a step being the unchanged binary triple
or `inside_operator`; `expression`, `expression_or_cond_pattern` and the three `inside_expression` rules lose the
grouped form. `1575 → 1576` rules. Grammar tier only; `verilog_2005` untouched (`inside` is an identifier there,
pinned). `dist` is unchanged — IEEE 1800-2017 18.5 rules *"dist expressions cannot appear in other expressions"*.

**AST impact — schema 27, a REPLACEMENT.** REMOVED: `expression.kind == "inside"`, `expression_or_cond_pattern.kind ==
"inside"`, the `inside_expression` shape `{expr, ranges}`. ADDED: an `operand_chain.rest` entry may be the OBJECT
`{kind: "sv_2017"|"sv_2023", body: {kind: "inside", ranges}}` beside the array triples. Migration: treat an object entry
as the postfix `inside {set}` operator on the relational row; `body.ranges` is the typed `open_range_list` /
`range_list` you read before; the left operand is the chain to its left. Every `inside`-free tree is byte-identical
(interpreter AST JSON, HEAD vs 1.0.213, on an inside-free control and across the pin set).

**Corpus proof**, per path, three lanes: `sv_2017` 1 rows `fail → pass`, **0** `pass → fail`, pass `9 781 →
9 782`, match `5 960 → 5 961`, unexplained **`101 → 100`**; `sv_2023` 1 rows `fail → pass`, pass
`9 802 → 9 803`; `verilog_2005` 0 rows moved. Pins `checked=593 armed=88 listed=330 multi_profile_rows=206 failures=0`. Gates on the final bytes: pins `checked=593 armed=88 listed=330 multi_profile_rows=206 failures=0`; `sv_cert_recognized_union_gate` GREEN (canonical `1433/8/1412/13`, union witness `1425` / UNKNOWN `0`, residual `[]`, seeds `[0,7,42]`, re-stamped); `verilog_2005_conformance_gate` GREEN (`1142/378/750/14`, `spf=0 prf=0`, lint orphans 0, corpus 363 / 0 mismatches, re-stamped); `sv_syntax_closure_gate` GREEN (`unreachable_rules=1`, the allow-list ONE, re-stamped); `ast_shape_contract_gate` 18/18; reproducibility TIER 2 byte-identical, baseline rewritten; parse-cost ratchet OK — `entries` −0.08 % and `memo_hits` −0.43 % promoted, `committed` +0.44 % accepted under the new `introduced_wrapper_frames` invariant (the lifted chain-step wrapper's own frames, removable by `ENGINE-UNIVERSAL-SERVICES.96`); the release probe and generated parser byte-identical across the gates' rebuilds.
Residue: `.3.38` (footnote 20), `.3.39` (typed directive arguments); the bar stands at 100.

### 1.0.212 / Contract 1.0.212 — SV-CORPUS-GRAD.3.40 + .3.36 (`PGEN-SV-CORPUS-GRAD-0367`, 2026-09-04), ledger `SV-0085` (`Released`):

⭐ **`local::` before a subroutine call inside `randomize() with { … }` parses again.** Release 1.0.211 reserved
Annex B's 248 words and gave `local ::` the A.8.4 `class_qualifier` production on the OPERAND surface (`x < local::x`);
it thereby narrowed the one legal text that had only ever parsed with `local` bound as an identifier —
`get_x() == local::get_x()` (verilator `t_randomize_method_with_scoping.v:70`). IEEE 1800-2017 §18.7.1 applies the
qualifier to *an identifier within an in-line constraint* and lets it *qualify class scopes and type names*, and Annex A
simply has no `tf_call` production for it — an Annex gap, not a prohibition. The scoped tf-name rule
(`scoped_or_hierarchical_tf_identifier`) gains the same `local :: [ this. | C:: ]` arm, so `local::get_x()`,
`local::get_c().c1_f`, `local::K::m()` and `local::this.f()` derive at every call site (expression, postfix chain, bare
call). In the same release the dead `block_data_type_or_implicit` (orphaned when `.3.35` split its callers under
footnote 6) is deleted — `1576 → 1575` rules — and the syntax-closure allow-list returns to `casting_type` alone.
Grammar tier only; `verilog_2005` is untouched (IEEE 1364-2005 has no `local::`).

**AST impact — one kind added, nothing removed or renamed.** A tf-call name may now be
`{kind: "local_scope", scope: <instance | class_scope | null>, name: …}` beside the existing `class_scope` and
`package_scope` kinds, and only on text that had no derivation before, so no existing tree moves: the primary-position
`local::x` and the unqualified call chain are byte-identical HEAD vs new, every legal control keeps its kind multiset.
Schema **26 unchanged**. The accept set WIDENS on `sv_2017` and `sv_2023` by exactly the `local::`-before-a-call family.

**Corpus proof**, per path, three lanes: `sv_2017` exactly **one** row `fail → pass` (the verilator row), **0** `pass → fail`, pass `9 780 → 9 781`, match `5 959 → 5 960`, unexplained **`102 → 101`**; `sv_2023` the same row, pass `9 801 → 9 802`, match `5 951 → 5 952`, unexplained `102 → 101`; `verilog_2005` **0** moved; 0 timeouts; the store-gate memo dropped exactly the two rows for the file that now parses and no other verdict moved; the sv_2017 × sv_2023 census `both_pass 9 780 → 9 781`, `2017_only_pass 0`. Pins: `fixed_local_scope_subroutine_call.sv` (five spellings) ACCEPT on both SV profiles; ratchet `checked=586 armed=81 listed=326 failures=0`. **Gates**: `sv_syntax_closure_gate` GREEN — `unreachable_rules 2 → 1`, `unreachable_branches 4 → 2`, re-stamped; `sv_cert_recognized_union_gate` GREEN — canonical `1432/8/1411/13`, union witness `1424` / UNKNOWN `0`, residual `[]`, deterministic across seeds `[0,7,42]`, `sample_parse_failures=0`, re-stamped (18 m 2 s under the guard, peak 6 756 MB; expectations set from the gate's own seed-0 run — `total 1433 → 1432`, `proof 9 → 8`: the deleted dead rule had counted as a PROOF rule since 1.0.210; witness and UNKNOWN unchanged); `verilog_2005_conformance_gate` GREEN — cert `1141/378/749/14` (samples 40, deterministic across seeds `[0,7,42]`, `spf=0 prf=0`), lint orphans 0, corpus `363` checks / `0` mismatches, aliases 2, re-stamped (39 m 30 s under the guard, peak 12 175 MB; expectations set from the gate's own seed-0 run — `total 1142 → 1141`, `proof 379 → 378`, the deleted dead rule, which had no `@profiles` and so counted as a proof rule under this profile too; the `local ::` arm is gated out of it and the lane moved 0 rows); `ast_shape_contract_gate` `ast_shape_contract_gate` GREEN — `18 passed; 0 failed` (3 m 57 s under the guard, peak 8 500 MB); reproducibility `check_generated_reproducibility.sh --rebaseline` TIER 2 — every family re-derives byte-identically from HEAD, baseline rewritten (1 m 21 s, peak 1 954 MB). `PARSE-COST-RATCHET`: `entries` ROSE `401 665 513 → 402 867 861` (+0.30 %) and `memo_hits` `175 830 693 → 176 431 867` (+0.34 %) with `committed` FLAT at `6 712 187` — `Δentries == 2 × Δmemo_hits` exactly, one unmatched terminal alternative (the `kw_local` body plus its memoized `trivia` prefix wherever the scoped tf-name rule is tried), ACCEPTED under `unmatched_terminal_alternatives`; family share `2.671 → 2.664 %`; `sv_parse_cost_ratchet` OK — identity fresh for the generated parser, grammar raw ast, instrument and 192 pinned sample files; the wall-clock advisory +7.5 % within the ±50 % band on a loaded host.
Residue unchanged: `.3.41` (`inside`/`dist` at relational precedence), `.3.38` (footnote 20), `.3.39` (typed directive
arguments); the bar stands at 101.

### 1.0.211 / Contract 1.0.211 — SV-CORPUS-GRAD.3.37 + .13z (`PGEN-SV-CORPUS-GRAD-0366`, 2026-09-04), ledger `SV-0084` (`Released`):

⭐ **Every word IEEE 1800-2017 Annex B reserves is now refused as an identifier — 248 words, derived from the
LRM text, where the grammar had reserved a hand-typed 173.** `new`, `this`, `super`, `null`, `static`, `var`,
`const`, `global`, `final`, `local`, `protected`, `virtual`, every `always_*`, every assertion operator and every
coverage word were admissible identifiers: `reg global;`, `var final;`, `i.super.i = 1;` parsed, and a class
constructor written `function new; input x;` parsed as an ordinary function NAMED `new` (with `C c = new;` a
reference to it). A new doctrine, `SV-RESERVED-WORDS`, re-derives the table on every commit and refuses any drift
in either direction (`stimuli/sv/check_sv_reserved_keywords.py`). Two more Annex A footnotes become grammar:
**footnote 10** — no `automatic` on a data declaration outside a procedural context (module, package, checker and
class sites; the randsequence code block keeps its lifetime) — and **footnote 18's `parameter` half** — a body
`parameter` carries its default (the parameter port list still may omit it; `.3.35` closed `localparam`). A bare
`null` gains the `primary` alternative A.8.4 names, and `local ::` (A.8.4 `class_qualifier`, the LRM's own §18.7.1
`x < local::x` example) gains the production it never had; both had parsed only through the identifier hole.
`1569 → 1576` rules; grammar tier only. Verilog-2005 tests that name a variable `var`, `global`, `local` or `new`
now REJECT on the SV lanes — correctly — and the adjudicator re-keys each such row by parsing it under
`verilog_2005` (the edition-keyword pass).

**AST impact — one corrected derivation.** No `kind` added, removed or renamed; every legal control keeps its
kind multiset (class handles and `new`, static lifetimes, body parameters, keyword-prefixed identifiers such as
`with_y` or `bins_z`, the randsequence block). ⛔ `d = null;` / `d == null` used to derive `null` as a hierarchical
NAME node and now derive the `{kind: "null"}` primary — same verdict, corrected tree; consumers matching `null` by
name will find the literal kind. Schema **26 unchanged**. The accept set NARROWS on `sv_2017` and `sv_2023`;
`verilog_2005` is untouched (its reserved set already equalled 1364-2005 Annex B).

**Corpus proof**, per path, three lanes: `sv_2017` **53** rows `pass → fail` (13 body parameters without a default — footnote 18; 32 Annex B keywords bound as names in Verilog-2005 tests; 2 `super.new` after a statement — A.2.7; 2 `automatic` at module scope — footnote 10; one `new[4]` without an unsized dimension — A.2.4; the two intended rows; ONE legal-by-prose `local::get_x()`, a known narrowing owed by `.3.40`) and **0** `fail → pass`; pass **9 833 → 9 780**; `sv_2023` **55** / **0** (the same plus `t_param_type_fwd.v` / `_bad.v` — footnote 22), pass **9 856 → 9 801**; the axis-2 bar **112 → 102** (five rows by refusing;
four `.13z` keys corrected to `must_accept` with the production — `integer x [] = 1;`, `P::Q = 1;`, `{a,b}.f1`,
`trireg (small) t;` are all derivable, the rejecting tool elaborates); `verilog_2005` **0**. **Gates**: `sv_cert_recognized_union_gate` GREEN — canonical `1433/9/1411/13`, union `1424/0`, re-stamped (the `+5` is the seven new rules net of two sv_2023-only ones; proof and UNKNOWN unchanged); `verilog_2005_conformance_gate` GREEN — cert `1142/379/749/14`, corpus 363 checks / 0 mismatches, re-stamped, one conformance fixture's SV expectation corrected under footnote 10; `sv_syntax_closure_gate` GREEN — allow-list unchanged, `reachable_branches` 2008 → 2027, re-stamped; `ast_shape_contract_gate` 18/18; reproducibility baseline re-derived. Cost:
`entries` ROSE `400 072 681 → 401 665 513` (+0.40 %) and `memo_hits` +0.47 % — the `kw_null` / `kw_local` terminal attempts of the two admitted alternatives plus their prefix lookups, net of a 433 K-entry fall from the footnote re-spellings — ACCEPTED under the new `declared_alternatives_net_of_retired_work` invariant (`ENGINE-UNIVERSAL-SERVICES.94`); `committed` FELL −0.59 %, promoted; family share `2.679 → 2.671 %`; `sv_parse_cost_ratchet` OK.
Residue named: footnote 20 (`reg [] x;` is legal only as a DPI import's sole packed dimension — `.3.38`), typed
argument productions for the tolerated directives (`.3.39`), and the `` `begin_keywords `` row as a keyword-version
feature gap.

### 1.0.210 / Contract 1.0.210 — SV-CORPUS-GRAD.3.35 (`PGEN-SV-CORPUS-GRAD-0364`, 2026-09-04), ledger `SV-0083` (`Released`):

⭐ **Four corrections from the residual bar, read one production at a time — two widenings, two
tightenings.** A property, sequence or checker port default followed by the next port now parses (the
actual argument's event expression used to swallow the list's comma); `type T = int, T x = 0` — the
standard's own §6.20.1 example — now parses. And two of Annex A's own footnotes are enforced: an implicit
data type in a data declaration requires `var` (footnote 6), and a `localparam` always carries its default
(footnote 18).

| input | before | after |
|---|---|---|
| `property p(clk = $inferred_clock, rst = $inferred_disable);` / `checker c(string msg = "violation", coverage_level clevel = cover_all);` | `rc 1` | **`rc 0`** |
| `module top #(int N = 5, M = N * 16, type T = int, T x = 0) ();` | `rc 1` | **`rc 0`** |
| `a;`, `y = 1;`, `[1:0] x;`, `signed x;` at module level (no `var`) | `rc 0` | **`rc 1`** |
| `localparam X;`, `localparam type X;` in a body | `rc 0` | **`rc 1`** |
| `var x = 1; var [1:0] y;`, `localparam X = 1;`, `#(parameter P, type U)`, `(a, b)` as an event default (controls) | `rc 0` | `rc 0`, kinds identical |

**How.** `event_expression_no_comma` in the actual-argument arm (same node shape); a lookahead on the type
list's continuation; the declaration rules' implicit arm behind a mandatory `var`; `_with_default` lists
for every `localparam` arm (the parameter-port-list omission footnote 18 allows is untouched).

**AST impact — one corrected derivation.** No `kind` added, removed or renamed; every legal control keeps
its kind multiset. ⛔ A block whose first item is a statement (`begin x = 1; …`) used to derive it as an
implicit-type `block_data` declaration — an illegal derivation by footnote 6 — and now derives the
`statement` it is. Same verdict, corrected tree; consumers walking `seq_block.declarations` for such
blocks will see the item move to `statements`. Schema **26 unchanged**.

**Corpus proof**, per path, three lanes: `sv_2017` **7** rows `fail → pass` (Surelog `SequenceInst`, ispras `06.20.01_02`, `16.08.02_01`, `16.14.07_01`, `17.09_01/02`, `23.10.03_02`) and **10** `pass → fail` (the seven sv2v error files the footnotes target, and three `must_accept`-keyed files — verible `array.sv`, `shadow_parameter.sv`, verilator `t_final.v` — that had parsed only through the footnote-6 mis-derivation and are ruled `must_reject`); pass **9 836 → 9 833**; the axis-2 bar **137 → 112**; `sv_2023` the same 17 rows; `verilog_2005` **0**. **Gates**: `sv_cert_recognized_union_gate` GREEN — canonical `1428/9/1406/13`, union `1419/0`, re-stamped; `verilog_2005_conformance_gate` GREEN — cert `1138/377/747/14`, corpus 363 checks / 0 mismatches, re-stamped; `sv_syntax_closure_gate` GREEN — allow-list ONE → TWO (the now-dead `block_data_type_or_implicit`, deletion owed to `.3.36`), `max_unreachable_branches` 2 → 4, re-stamped; `ast_shape_contract_gate` 18/18; reproducibility baseline re-derived. Cost: `entries` FELL `414 471 371 → 400 072 681` (−3.47 %), `committed` −0.12 %, `memo_hits` −5.61 %, promoted by the rebaseline; family share `2.744 → 2.679 %`; `sv_parse_cost_ratchet` OK.
The unterminated block comment the triage found (`/*` alone parses) is the engine's layout skipper, routed
as `ENGINE-UNIVERSAL-SERVICES.93`.

### 1.0.209 / Contract 1.0.209 — SV-CORPUS-GRAD.3.34 + .3.33 (`PGEN-SV-CORPUS-GRAD-0362`, 2026-09-04), ledger `SV-0082` (`Released`):

⭐ **A parenthesised nested tagged pattern now parses under `sv_2017` — the form the standard's own
§12.6.1 examples write.** `tagged Jmp (tagged JmpU .a)` and `tagged Jmp (tagged JmpC '{.c, .a})` appear
in IEEE 1800-2017 §12.6.1 itself, yet its Annex A `pattern` has no `( pattern )` alternative — an erratum
1800-2023 corrects by spelling it first. The 2017 profile had followed Annex A literally; the `.14a`
sv_2017 × sv_2023 differential exposed it on five IEEE 1800-2012 tests that only the 2023 profile parsed.

| input | before | after |
|---|---|---|
| `tagged Jmp (tagged JmpU .a) : …` — a parenthesised nested tagged pattern, `sv_2017` | `rc 1` | **`rc 0`** |
| the same under `sv_2023` | `rc 0` | `rc 0` |
| `tagged JmpU .a`, `tagged JmpC '{.c, .a}` — unparenthesised (control), both profiles | `rc 0` | `rc 0`, AST byte-identical |
| `tagged JmpU ()` — empty parentheses where a pattern is required (invalid pin) | `rc 1` | `rc 1` |

**How.** `pattern_sv_2017` gains `( pattern )` as its first alternative, in the `sv_2023` arm's own spelling;
no new rule. And (`.3.33`) the ten single-letter keyword terminals the UDP fix of 1.0.207 left unreferenced
are deleted — 1564 → 1554 rules, the syntax-closure contract's allow-list back to one.

**AST impact — shape-neutral.** Every input that parsed before yields a byte-identical typed AST; a
newly-accepted parenthesised pattern carries `{kind: "parenthesized", body: …}`, the value the 2023 arm
has always emitted. Schema **26 unchanged**. ⛔ The accept set **widens** on `sv_2017` only.

**Corpus proof**, per path, three lanes: `sv_2017` exactly **5** rows `fail → pass` (ispras `ieee-1800-2012/12/12.06.01_05/06.sv`, `12.06.02_01/02/03.sv`), pass **9 831 → 9 836**, the axis-2 bar **142 → 137**; `sv_2023` **0** and `verilog_2005` **0** rows moved; `pass → fail` **0**. Union certificate `1417/8/1396/13`, union `1409/0` (residual `[]`, seeds 0/7/42 — ten fewer rules, all ten proof); `verilog_2005` `1130/373/743/14` (seeds 0/7/42); syntax closure `unreachable_rules=1` on an allow-list of one; AST-shape gate 18/18; every generated family re-derives byte-identically. Cost: the parse-cost ratchet reads `entries` +0.04 % and `memo_hits` +0.04 % with `committed` **exactly** unchanged — the whole rise is one sampled fixture (slang `all.sv`, still rejected) whose parse now advances 2 216 bytes past its nested tagged patterns; the other 191 sampled files are byte-identical — accepted under a new per-file invariant (`advanced_files_only`).

### 1.0.208 / Contract 1.0.208 — SV-CORPUS-GRAD.3.31 (`PGEN-SV-CORPUS-GRAD-0357`, 2026-09-03), ledger `SV-0081` (`Released`):

⭐ **An array-element member access whose index is an increment/decrement or a parenthesised operator
assignment now parses.** `clist[b[0]++].m <= 1;` and `clist[(b[0] += 1)].m` were refused on `sv_2017` and
`sv_2023` while `clist[b[0]++] = null;` (no member) and `clist[b[0]+1].m` / `clist[f()].m` (another index
kind) parsed: IEEE 1800-2017 A.8.3 gives `expression` two operands — `inc_or_dec_expression` and
`( operator_assignment )` — that `constant_expression` lacks, and the index of an element whose member is
then selected was the one index path the grammar spelled through `constant_bit_select`.

| input | before | after |
|---|---|---|
| `clist[b[0]++].m <= 1;`, `x = clist[b[0]++].m;` — an inc/dec index, member selected after it | `rc 1` | **`rc 0`** |
| `clist[(b[0] += 1)].m <= 1;` — a parenthesised operator assignment as that index | `rc 1` | **`rc 0`** |
| `clist[b[0]+1].m`, `clist[b[0] ? 0 : 1].m`, `clist[f()].m` (control) | `rc 0` | `rc 0`, AST byte-identical |
| `clist[b[0]++] = null;`, `a[b[0]++] = 1;` — the same index with no member after it (control) | `rc 0` | `rc 0`, AST byte-identical |

**How.** A caller split: `hierarchical_bit_select := ( lbrack hierarchical_index_expression rbrack )*`, whose
operand is `constant_expression`'s plus exactly the two missing alternatives, referenced at the ten
hierarchical-path sites (`hierarchical_identifier`, `hierarchical_tf_identifier` and the three method-chain
receivers); `constant_bit_select` and its constant-context callers (`bind` targets, `constant_select`,
typedef-through-instance) are untouched, so `++` stays refused where the standard makes the index constant.
Two cheaper spellings were refused by their mechanism first: an ordered choice commits on the `b[0]`
prefix and never sees the `++`; a lookahead cannot see a postfix `++` without parsing the lvalue.

**AST impact — shape-neutral.** Every input that parsed before yields a byte-identical typed AST (measured
on the eight control inputs, both profiles); a newly-accepted index carries the existing
`inc_or_dec_expression` / `operator_assignment` node shapes as the operand body. Schema **26 unchanged**.
⛔ The accept set **widens** on `sv_2017` and `sv_2023`.

**Corpus proof**, per-path and two-sided, both lanes: `sv_2017` exactly **1** row `fail → pass` — the
predicted `t_assign_automatic_bad.v` — pass **9 830 → 9 831**, the axis-2 bar **162 → 161**;
`verilog_2005` **0** rows moved; `pass → fail` **0**. Union certificate `1427/18/1396/13`, union `1409/0` (residual `[]`, deterministic across seeds 0/7/42 — the three new rules witnessed, the 13-rule canonical UNKNOWN set identical); `verilog_2005` `1140/383/743/14` with the gate's own flags (union `756/1` at seed 0), deterministic across seeds; AST-shape gate 18/18; every generated family re-derives byte-identically. Cost: the parse-cost ratchet
reads `entries` +0.33 % and `memo_hits` +0.38 % with `committed` **exactly** unchanged — the split moves
its work one for one and pays for the memo the two twin rules no longer share — accepted under a new coded
invariant (`caller_split_containment`) and routed for elimination (`ENGINE-UNIVERSAL-SERVICES.92`).

### 1.0.207 / Contract 1.0.207 — SV-CORPUS-GRAD.3.30 (`PGEN-SV-CORPUS-GRAD-0349`, 2026-09-03), ledger `SV-0080` (`Released`):

⭐ **A UDP state table written with adjacent symbols now parses, on every profile.** IEEE 1800-2017's own
tables are written that way — `(01) (10) 0 : 0 : 1 ;` (§29.3.4) — and §29.3.4 separates only the input
fields *from the output field*, by a colon. The grammar's three table rules had been transcribed from Annex
A's single-character symbol productions as references to the general keyword terminals (`trivia /0\b/` …),
and a keyword's word boundary cannot match a symbol followed by another symbol character: every spaced
table parsed, every touching one was refused, on `sv_2017`, `sv_2023` and `verilog_2005` alike.

| input | before | after |
|---|---|---|
| `(01) 0 : ? : 0 ;` — an edge indicator whose symbols touch, any profile | `rc 1` | **`rc 0`** |
| `? ?? 01 : ? : 1 ;`, `00 : 0 ;`, `0x : 0 ;`, `? ? (bx) 1 : ? : x ;` | `rc 1` | **`rc 0`** |
| the same tables with one space between every symbol (control) | `rc 0` | `rc 0`, AST byte-identical |
| `2 0 : 0 ;` / `0 a : 0 ;` — a digit or an identifier where only a symbol may stand (invalid pin) | `rc 1` | `rc 1` |

**How.** Fourteen boundary-free terminals (`udp_sym_0 := trivia /0/` … `udp_sym_n_upper := trivia /N/`), one
per symbol character, referenced only by `level_symbol`, `output_symbol` and `edge_symbol`; the general
keywords keep their boundary everywhere else it is load-bearing (`assert #0`, `$finish(0)`, `init_val`,
`scalar_constant`, `z_or_x`). The first cut named the upper-case twins `udp_sym_X` and the generated parser
did not compile — six `E0592`, because the generator uppercases rule names into `RULE_…` constants — after
the lint and the interpreter had both said green; renamed `udp_sym_<c>_upper`, and the missing lint arm is
`ENGINE-UNIVERSAL-SERVICES.91`.

**AST impact — shape-neutral.** The three rules keep their `kind` values; every table that parsed before
yields a byte-identical typed AST (measured on the spaced control, all three profiles), and a touching
table yields the AST its spaced spelling always did (equal once source spans are stripped). Schema **26
unchanged**. ⛔ The accept set **widens** on every profile.

**Corpus proof**, per-path and two-sided, both lanes: `sv_2017` **17** rows `fail → pass` (8
`unexplained_rejects_valid → match`, 9 already routed or deferred), none the other way; pass **9 813 → 9 830**,
the axis-2 bar **220 → 212**; `verilog_2005` **6** rows `unexplained_rejects_valid → match`, pass
**2 300 → 2 306**. Union certificate `1424/18/1393/13`, union `1406/0` — the ten keyword terminals the
tables no longer reference are now provably unreachable and move witness → proof; `verilog_2005`
`1137/383/740/14` with the gate's own flags (its three union configs; union `753/1`, the identical 14
UNKNOWN — a bare run without them reads `394/3`, an artifact of the declared entry universe, not of the
grammar). Cost: the parse-cost ratchet reads `entries`, `memo_hits` and `committed` exactly
unchanged — each new terminal takes what its keyword twin lost.

⭐ **How it was found is the part worth keeping**: the 199 residual rows cluster into 142 stuck-point
signatures with no family larger than five, which reads as "no family". Joining the corpus's 97 `endtable`
files to the adjudication manifest instead found 16 + 6 rows in one construct — when the clusterer says
no family, join on the construct's own keyword and let the manifest count.

### 1.0.206 / Contract 1.0.206 — SV-CORPUS-GRAD.3.29 (`PGEN-SV-CORPUS-GRAD-0347`, 2026-09-03), ledger `SV-0079` (`Released`):

⭐ **A `cross` body's two implicit typedefs, `CrossValType` and `CrossQueueType`, now parse as type names
inside the cross — and nowhere else.** IEEE 1800-2017 §19.6.1.3 says *"The types are named CrossValType and
CrossQueueType. The scope of the type names is the cross itself and the types are not accessible outside of
this scope"* and *"The cross types shall be considered as implicit typedefs in the body of the cross, even though
the syntax does not allow typedefs to appear there explicitly."* Annex A declares them nowhere, so the parser's
sound store gate — right about every name a file must declare — refused a name the standard declares. The
standard's own §19.6.1.4 example (`function CrossQueueType myFunc1(int f_lim); … bins one = myFunc1(cg_lim);`)
is vendored twice in the corpus, and both rows were in the axis-2 bar.

| input | before | after |
|---|---|---|
| `aXb : cross a, b { function CrossQueueType f(int n); … endfunction bins one = f(lim); }` — `sv_2017` | `rc 1` | **`rc 0`** |
| the same, `sv_2023` | `rc 1` | **`rc 0`** |
| `CrossValType v;` as a local inside that function (`fixed_cross_predefined_val_type.sv`) | `rc 1` | **`rc 0`** |
| the same function returning a module-level `typedef int q_t[$];` (control) | `rc 0` | `rc 0` |
| `CrossQueueType q;` at module level AFTER the endgroup (invalid pin) | `rc 1` | `rc 1` |
| `int CrossValType;` at module level (the spelling is not a keyword) | `rc 0` | `rc 0` |

**How.** The grammar opens a `cross` scope at the body's `{` and closes it at the `}` — the first scope this
grammar opens — and each of the two unscoped type use-sites gains an ordered second branch, a terminal spelling
the two names gated `post` on `in_scope_kind(cross)`, tried only after the unchanged store gate has refused the
name. Three spellings were built and refused first, each by an instrument rather than an opinion: emitting the
two names as facts into the scope leaked them after the scope closed, because `has_fact` is store-wide
(routed to the engine as `ENGINE-UNIVERSAL-SERVICES.89`, with the book's semantic-store chapter corrected);
a lookahead recognizer could not be witnessed by the certificate; a dispatcher tried first, and then a `pre`
scope gate, were refused by the parse-cost ratchet.

**AST impact — shape-neutral.** A predefined name arrives exactly where a declared type does, with the same
shape: `{type: {body: "CrossQueueType"}, dims: […]}` beside `{type: {body: "q_t"}, dims: […]}`. No `kind` is
added, removed or renamed; every input that parsed before yields a byte-identical typed AST (measured on the
two cross-body controls, both profiles). Schema **26 unchanged**. ⛔ The accept set **widens** on `sv_2017` and
`sv_2023`; `verilog_2005` is unchanged by construction (covergroups are not Verilog-2005) and by measurement.

**Corpus proof**, per-path and two-sided, both lanes: exactly the two rows above moved (`fail → pass`,
`unexplained_rejects_valid → match`); `sv_2017` pass **9 811 → 9 813**, `match 5 851 → 5 853`, the axis-2 bar
**222 → 220**; `verilog_2005` pass **2 300** unmoved. Union certificate gate green with a fifth entry
`cross_body:sv_2017` (canonical 1410 / 8 / 1389 / 13, union 1402 / 0 at seeds 0/7/42); v2005 gate green,
unchanged. Cost: `entries` +634 818 (+0.15 %) confined to the four introduced rules — one recognizer entry per
type position the store gate refuses outside a cross — accepted under the ratchet's `contained_in_introduced_subgraph`
invariant; the engine-tier elimination is `ENGINE-UNIVERSAL-SERVICES.90`.

⭐ **How it was found is the part worth keeping**: `SV-CORPUS-GRAD.13w` explained 46 store-gate rejections *by
parsing* — declaring what the parser demanded and re-parsing — and REFUSED to explain two, because the name they
demanded is one the LRM predefines. An argument would have explained them away; the parse found a grammar defect.

### 1.0.205 / Contract 1.0.205 — EBNF-FRONTEND-SILENT-TRUNCATION.6 (`PGEN-EBNF-FRONTEND-SILENT-TRUNCATION-0002`, 2026-09-03), ledger `SV-0078` (`Released`):

⛔ **From the entry `systemverilog_parseable_file` (`--entry-rule`), an `import p::*;`, a `timeunit 1ns;` or a
`` `timescale `` item no longer parses — on any profile.** `parseable_source_item := semi` is the whole rule; the
three lines after it in the grammar are `//` comments (since 2026-04-25), and `grammars/ebnf.ebnf:583-586`
defines `//` as a line comment. The hand-written EBNF frontend knew only `#`, read those lines as live
alternatives, and every release carried them. The rule is proven unreachable from the registered entry
(`rule_fire_partition`), so from the default entry nothing changes; the generated parser moves in this one rule
and the other ten generated parsers are byte-identical. The same fix stops indented comments after inline
return annotations from riding inside the annotation token (14 payloads in 11 rules), so a comment-only edit no
longer moves the semantic digest this contract is keyed on. **AST shape unchanged for every legal input.**

### 1.0.204 / Contract 1.0.204 — SV-CORPUS-GRAD.13p (`PGEN-SV-CORPUS-GRAD-0322`, 2026-09-02), ledger `SV-0077` (`Released`):

⛔ **A third delay value on an n-input or n-output gate, a pass-enable switch or a UDP instance no
longer parses — on any profile.** IEEE 1800-2017 §28.4 / §28.5 say of those gates *"The delay
specification shall be zero, one, or two delays"*, and Annex A spells `[ delay2 ]` there; three
delays belong to enable gates and switches (`[ delay3 ]`, §28.6). IEEE 1364-2005 §7.2 / §7.3 say the same. The
grammar had one `delay` rule with the three-value alternative at every site, because the LRM
extractor's footnote canonicaliser had merged Annex A's `delay2` and `delay3` into one rule — the
right keywords in the wrong arity, which no keyword census can see.

| input | before | after |
|---|---|---|
| `module test; wire a, b, o; nand #(1,2,3) g (o, a, b); endmodule` — all three profiles | `rc 0` | **`rc 1`** |
| `and #(1,2) g (o, a, b);` (control: two delays are legal) | `rc 0` | `rc 0` |
| `bufif0 #(1,2,3) g (o, a, e);` (control: an enable gate takes three) | `rc 0` | `rc 0` |

**The fix**: a `delay2` profile-split pair — the `value` and `pair_optional` alternatives only,
return annotations byte-identical to `delay`'s — referenced at the seven Annex-A `delay2` sites.
Three rules added (1540 → 1543), no profile orphans, exactly three rules' satisfiable sets moved.

**AST impact — none for legal text.** A two-delay site yields the same
`{body: {kind: "pair_optional" | "value", …}}` object; `triple_optional` becomes unreachable there.
Schema **26 unchanged**. ⛔ The accept set **narrows on every profile**, `sv_2017` included.

**Corpus proof**, per-path and two-sided: ZERO verdicts moved on either lane — 16 336 `sv_2017` rows and 2 606 `verilog_2005` rows, row sets identical, pass 9 811 / 2 300 unchanged — and the axis-2 bar moved **268 → 269** for a reason that is measured and routed: `verilator/test_regress/t/t_gate_basic.v` (already `fail`) went from `divergence:explained_svpp_conditional` to `divergence:unexplained_rejects_valid` because its parse now stops at line 25, `nand #(2,3,4)`, where no preprocessing can alter the text (the `.12a` rule), under a `must_accept` key that rests on the suite's driver (*"t_gate_basic.py expects success"*) and not on the standard, which forbids that text (§28.4). It is the first MAIN-lane member of `SV-CORPUS-GRAD.13k`'s answer-key class and is routed to `.13k`(a)+(c) with the citation; the key is not edited in a release slice, so the bar reports the honest number under the key as it stands. Tuple `7556/2606/6174/4363/268` → `…/269`, `check_sv_corpus_denominator.sh` OK after the anchor moved with it. Exactly one vendored file spells a
three-value delay on a two-delay gate (`verilator t_gate_basic.v:25`), and it already failed for an
unrelated reason.

### 1.0.203 / Contract 1.0.203 — SV-CORPUS-GRAD.13o (`PGEN-SV-CORPUS-GRAD-0319`, 2026-09-01), ledger `SV-0076` (`Released`):

⛔ **`always @(edge c)` no longer parses under `verilog_2005`.** The grammar's `edge_identifier`
carried IEEE 1800's third member `edge` with no profile gate. IEEE 1364-2005 A.7.4 is
`edge_identifier ::= posedge | negedge`, and its A.6.5 `event_expression` admits only `posedge` /
`negedge` forms — `edge` in an event control is IEEE 1800. The keyword itself IS reserved by
1364-2005 (for the timing-check `edge_control_specifier`), which is why the keyword-admission
census reported it legal: the keyword was fine, the position was not. Found by the
keyword-enumeration membership census (TOOLBOX 5.13) and confirmed by three probe legs.

| input | before | after |
|---|---|---|
| `module m; reg c; always @(edge c) ; endmodule` — `verilog_2005` | `rc 0` | **`rc 1`** |
| the same, `sv_2017` / `sv_2023` | `rc 0` | `rc 0` |
| `module m; reg c; always @(zzq c) ; endmodule` — `verilog_2005` (falsifier: the keyword was consumed as a keyword) | `rc 1` | `rc 1` |
| `module m; reg c; always @(posedge c) ; endmodule` (control, all profiles) | `rc 0` | `rc 0` |

**The fix**: the `edge` branch now references a profile-gated terminal, `kw_edge_sv_only` under
`@profiles: ["sv_2017", "sv_2023"]`, so `edge_identifier` stays a terminal-only rule. One rule added
(1539 → 1540), no profile orphans, and exactly one rule's satisfiable set moved — the new terminal.
The alias spelling used by earlier profile gates was built first and rejected by the parse-cost
ratchet: it de-memoized `edge_identifier` and made three pre-existing rules dearer.

**AST impact — none.** No `kind` value is added, removed or renamed; under `sv_2017` / `sv_2023`
the `edge_identifier` node is byte-identical (`{kind: "edge"}`), and under `verilog_2005` that value
is now unreachable. Schema **26 unchanged**. ⛔ The `verilog_2005` accept set **narrows**: text that
used to yield an AST there now rejects.

**Corpus proof**, per-path and two-sided: **zero verdicts moved** on either lane (16 336 `sv_2017`
rows, 2 606 `verilog_2005` rows, row sets identical), the verdict-coverage tuple
`7556/2606/6174/4363/268` unchanged. 0 of the 2 606 v2005-lane files spell `@(edge`; 24 of the
16 336 main-lane files do, every one an IEEE 1800 source that keeps parsing.

### 1.0.202 / Contract 1.0.202 — SV-CORPUS-GRAD.13n (`PGEN-SV-CORPUS-GRAD-0316`, 2026-09-01), ledger `SV-0075` (`Released`):

⛔ **`tri0` and `tri1` are net types again.** The grammar's `net_type` alternation shipped **ten**
of the LRM's **twelve** branches, and that branch has never existed — `git log -S 'kw_tri0'` over
the grammar returns exactly one commit, this fix. Both spellings were already in the
reserved-keyword regexes, so they lexed as keywords and had no production to land in — which is why
the failure carried no clue that anything was missing.

⚠️ **How far back the REJECTION goes is measured, and it is younger than the gap.** The two
spellings entered the reserved-keyword regexes on **2026-07-02** (`verilog_2005`) and **2026-07-04**
(`sv_2017` / `sv_2023`); before then they were ordinary identifiers, and whether `tri0 a;` parsed at
that time is **not measured here**. So the missing branch spans the file's whole history, while the
symptom you would have observed dates from those two commits.

| input | before | after |
|---|---|---|
| `module m; tri0 a; tri1 b; endmodule` — `sv_2017` | `rc 1` | **`rc 0`** |
| the same, `sv_2023` | `rc 1` | **`rc 0`** |
| the same, `verilog_2005` | `rc 1` | **`rc 0`** |
| `module m; wire a; endmodule` (control, same alternation) | `rc 0` | `rc 0` |
| `module m; triand a; trior b; endmodule` (control) | `rc 0` | `rc 0` |

All three vendored editions define them: IEEE 1364-2005 Annex A A.2.2.1:219, IEEE 1800-2023
Annex A:891, IEEE 1800-2017 Annex A A.2.2.1 (`docs/systemverilog/2017/md/section-41-data-read-api.md:899` — the PDF→markdown splitter filed that edition's Annex A under the section-41 file name, which is why a name-keyed search reports a false absence; TOOLBOX 5.8) and clause 6 (`section-6-data-types.md:924-928`).

**AST impact — additive only.** Two new `kind` values, `"tri0"` and `"tri1"`, join the ten a
`net_type` node could already carry. Schema **26 unchanged**. A consumer switching *exhaustively*
on that field must add two arms; one that matches known values and falls through is unaffected.
⛔ The accept set **widens**, so text that used to fail now yields an AST.

**Corpus proof**, per-path and two-sided: `sv_2017` pass **9 794 → 9 811**, axis-2 bar **274 → 268**;
`verilog_2005` pass **2 291 → 2 300**, its arm **66 → 57**. 18 rows moved in the main lane, 9 in the
v2005 lane, row sets identical on both sides, and **18 of 18 contain `tri0` or `tri1`**.

⭐ **How it was found is the part worth keeping**: `SV-CORPUS-GRAD.13k`(b) censused the
`verilog_2005` rows the answer key calls `must_accept` and the parser rejects, expecting to find
answer-key over-claims. It found 19 of those — and **20 rows of the opposite defect**, text the LRM
*does* define that the parser could not derive. This is one of them.

### 1.0.201 / Contract 1.0.201 — ENGINE-UNIVERSAL-SERVICES.65 (`PGEN-ENGINE-UNIVERSAL-SERVICES-0117`, 2026-08-28), ledger `SV-0074` (`Released`):

⛔ **The 4096-frame recursion ceiling is gone. What replaced it is not a bigger number — it is a
measurement of the stack this host actually gave the parse thread.** `4096` survives only as the
depth at which the guard first *asks* how much stack it has spent; from there the ceiling is
whatever the remaining headroom can carry.

**What now parses that did not** (measured at a deliberately small 96 MiB reservation — under a
third of the shipped default — so these are a floor):

| shape | before | after @ 96 MiB |
|---|---:|---:|
| `(s ? a : (…))` — mux, both arms parenthesised (worst measured shape) | 146 | **1 142** |
| `$signed(…)` | 163 | 1 550 |
| `int'(int'(…))` | 249 | 1 807 → **4 813** at the shipped default |
| `{{{…}}}` | 314 | 2 487 |
| `(((…)))` | 322 | 2 434 |
| `f(f(f(…)))` | 340 | 3 055 |

**How you can tell it is genuinely host-derived.** Vary the reservation and the boundary moves with
it, linearly. At 96 / 128 / 192 / 256 MiB the deepest accepted nested cast reads
1 807 / 2 410 / 3 611 / 4 813 — **18.82 levels per MiB, flat to 0.14 %**. No constant of any size
produces that curve. Set `PGEN_PARSE_STACK_BYTES` (bytes, floored at 96 MiB) to buy more depth on a
host that can afford it; the rejection message now names that knob itself.

**Nothing else moves.** External-corpus verdicts are identical over **16 336 files** (`pass=9794
fail=6542`, 0 per-file differences, measured on the fused graph that ships); the interpreter stays
byte-identical to the generated parser; no grammar bytes changed and AST-dump schema stays at `26`.

⚠️ **Bound — the capability is not free and the cost is quadratic in depth.** The same cast shape
parses in **0.13 s** at 250 levels and **22.24 s** at 4 000 (each doubling ≈3.95×), and refusing
input one level past the boundary now costs **37.7 s** where it cost 0.19 s — still bounded, still
ceiling-labelled. Plan for seconds, not milliseconds, at thousands of levels. Tracked as
`ENGINE-UNIVERSAL-SERVICES.68`; the reservation's remaining compiled-in default is `.69`.

### 1.0.200 / Contract 1.0.200 — ENGINE-UNIVERSAL-SERVICES.66 (`PGEN-ENGINE-UNIVERSAL-SERVICES-0115`, 2026-08-28), ledger `SV-0073` (`Released`):

⛔ **This release removes a silent acceptance limit and changes nothing else.** Every quantifier in
every generated parser carried a **10 000-iteration cap** (`SAFETY_LIMIT`). It did not refuse — it
**stopped iterating**, silently truncating the repetition, so the parse failed at the enclosing level
as an ordinary *"did not consume full input"* with no diagnostic naming the cause.

| generated shape | 10 001 items | 12 000 items | 20 000 items |
|---|---|---|---|
| flattened-netlist instances | rejected → **accepted** | rejected → **accepted** | rejected → **accepted** |
| wire declarations | rejected → **accepted** | rejected → **accepted** | rejected → **accepted** |
| `case` items | rejected → **accepted** | rejected → **accepted** | rejected → **accepted** |
| concat elements | accepted | rejected → **accepted** | rejected → **accepted** |
| module port list | accepted | rejected → **accepted** | rejected → **accepted** |

**13 of 13 measured rejections eliminated, 0 introduced.** The replacement bound is derived from the
input rather than being a bigger constant: every counted iteration must advance the cursor, so a
repetition cannot have more elements than the input has bytes. The guard still terminates a
pathological zero-width loop; it can no longer refuse conforming source.

**Nothing else moves.** External-corpus verdicts are identical over **16 336 files** (`pass=9794
fail=6542 timeout=0 crash=0`, 0 per-file differences); the recursion ceiling's accept boundaries are
byte-for-byte unchanged (322/323, 272/273, 249/250); the interpreter stays byte-identical to the
generated parser. No grammar bytes changed — the artifact is regenerated because codegen moved, and
AST-dump schema stays at `26`.

⚠️ **Bound**: this makes a large netlist *legal*, not yet *ingestible* — parse memory is
**2 500×–8 000× the input size** and linear (523 KB → ≈4.0 GB). Tracked as
`ENGINE-UNIVERSAL-SERVICES.67`.

### 1.0.199 / Contract 1.0.199 — SV-CORPUS-GRAD.13e.10(c4) RESIDUE + ENGINE-UNIVERSAL-SERVICES.50(e)/.55 (`PGEN-ENGINE-UNIVERSAL-SERVICES-0098`, 2026-08-26), ledger `SV-0072` (`Released`): **THE SEVENTEENTH AND LAST IEEE-1800-ONLY OPERATOR — THE CLASS IS NOW CLOSED, 17 OF 17 (GRAMMAR + ENGINE; `verilog_2005` ONLY — a NARROW; SCHEMA UNCHANGED at 26)**

⛔ **This release narrows one profile and touches nothing else.** The assignment-pattern / cast
apostrophe that `1.0.198` deliberately left open is now gated:

| accepted under `verilog_2005` before `1.0.199` | why the standard does not derive it |
|---|---|
| `a = '{1,2};` and the assignment-pattern / cast family reached through `'` | IEEE 1800 A.6.7.1 defines assignment patterns and A.8.4 defines casts; IEEE 1364-2005 has **neither construct** |

⭐ **`1'b0` AND `8'hff` STILL PARSE UNDER ALL THREE PROFILES, AND THAT IS THE POINT OF THE GATE'S
PLACEMENT.** Every `'` in IEEE 1364-2005 sits inside a **sized/based number literal**, which the
`number` terminal lexes as one token — it never reaches the `tick` rule. The literals are pinned as
standing controls, together with three timing-check shapes that exercise the two `!tick` guard sites.

⭐ **`sv_2017` / `sv_2023` are UNMOVED**: 10 of 10 typed ASTs byte-identical over 5 inputs × 2 SV
profiles, and the chained cast `int'(2)'(3)` still parses — verified on the **generated** parser,
because the interpreter cannot settle this question while `left_recursion_unhandled > 0`, which was
the state under test.

⛔⛔ **WHY IT TOOK A SECOND RELEASE, AND WHY THE SPLIT WAS RIGHT.** `1.0.198` could have landed all
seventeen in one commit. It would have shipped a parser that **silently lost 15 left-recursive
cycles on the profiles being released**, with every counter an operator reads showing green — because
tagging `constant_cast` makes two dialect gates **cross** on the route PGEN's eliminator rewrites, the
eliminator refused the whole candidate, and `left_recursion_unhandled` is a *warning* that leaves
`--lint-grammar` at exit 0. Three engine repairs were needed: the eliminator now **synthesizes** a
crossing route's profile intersection instead of refusing it; its own suffix wrapper takes the
**union** of its routes' gates instead of the base rule's (it was becoming a profile orphan whose
tool-derived fix nobody could apply, because there is no source line to tag); and the counter is now
**ratcheted** as a registered doctrine, so that silence cannot recur.

⭐ **The companion set is 7 source rules, not the 11 the linter names** — four are the eliminator's
own temporaries, which resolve once the engine derives their gates. **A tool-derived list is not
thereby an actionable list.**

**For consumers:** if you ran the strict profile against `1.0.198` or earlier, `'{…}` was accepted
and is now correctly rejected. Nothing else changes; the AST schema stays **26**.

### 1.0.198 / Contract 1.0.198 — SV-CORPUS-GRAD.13e.10(c4) (`PGEN-SV-CORPUS-GRAD-0305`, 2026-08-25), ledger `SV-0071` (`Released`): **SIXTEEN IEEE-1800-ONLY OPERATORS STILL REACHED THE `verilog_2005` PROFILE (GRAMMAR; `verilog_2005` ONLY — a NARROW; SCHEMA UNCHANGED at 26)**

⛔ **This release narrows one profile and touches nothing else.** Under the strict Verilog-2005
profile all of the following parsed, and IEEE 1364-2005 derives none of them:

| accepted under `verilog_2005` before `1.0.198` | why the standard does not derive it |
|---|---|
| `a += 1;` `-=` `*=` `/=` `%=` `&=` `\|=` `^=` `<<=` `>>=` `<<<=` `>>>=` | A.6.2 gives assignment `=` and `<=` and nothing else; ten of the twelve occur **0×** in the whole LRM |
| `if (a ==? b)` · `if (a !=? b)` | A.8.6 defines no wildcard equality; both occur **0×** |
| `if (a <-> b)` | the equivalence operator is IEEE 1800 only; occurs **0×** |
| `s u(.*);` | A.4.1 `named_port_connection` has no implicit form |

⭐ **THE LENS IS THE POINT: THIS CLASS IS INVISIBLE TO EVERY CENSUS THAT PRECEDED IT.** `1.0.197`'s
instrument crosses profile-filtered reachability against IEEE 1364-2005 **Annex B** — a normative
list of **keywords**. An operator appears in no keyword list, so a construct spelled entirely in
punctuation can never become a candidate. `1.0.197` found `{ << { a } }` only by accident, and named
the class rather than claiming it; this release closes it over a **derived, closed population of 19**,
of which 17 were confirmed over-acceptances and `++`/`--` correctly excluded.

⛔⛔ **ONE OF THE SEVENTEEN IS NOT FIXED HERE, AND SAYING SO IS PART OF THE RELEASE.** The assignment-
pattern / cast apostrophe (`a = '{1,2};`) still parses under `verilog_2005`. Gating it requires
tagging `constant_cast`, which puts two **crossing** dialect gates on the indirect-LR route PGEN's
eliminator rewrites; the eliminator then refuses that route and 15 left-recursive cycles fall back to
the runtime guard — a change `--lint-grammar` reports only as a **warning** while the parser silently
loses derivations under `sv_2017`/`sv_2023`. The engine repair is tracked as
`ENGINE-UNIVERSAL-SERVICES.50`, and the over-acceptance stays **pinned and watched** in the
adjudication ratchet until it lands. **A consumer of `1.0.198` should still treat `'{…}` as
SV-only when running the strict profile.**

### 1.0.197 / Contract 1.0.197 — SV-CORPUS-GRAD.13e.7 (`PGEN-SV-CORPUS-GRAD-0293`, 2026-08-24), ledger `SV-0070` (`Released`): **THIRTEEN IEEE-1800 SURFACES STILL REACHED THE `verilog_2005` PROFILE (GRAMMAR; `verilog_2005` ONLY — a NARROW; SCHEMA UNCHANGED at 26)**

⛔ **This release narrows one profile and touches nothing else.** Under the strict Verilog-2005
profile all of the following parsed, and IEEE 1364-2005 derives none of them:

| accepted under `verilog_2005` before `1.0.197` | why the standard does not derive it |
|---|---|
| `chandle c;` · `enum { A } e;` · `virtual interface I vi;` | each keyword occurs **0×** in Annex A |
| `string s;` | A.8.8 `:1218` makes `string` the **literal**, never a data type |
| `const integer i = 1;` · `var integer i;` · `static integer i;` · `automatic integer i;` | A.2.1.3 `:152` gives a declaration no qualifier and no lifetime |
| `module automatic m;` · `module static m;` | A.1.2 `:29` has no lifetime in either alternative |
| `extern module m;` | `extern` occurs 0×; A.1.2 has no extern alternative |
| `default disable iff x;` | `iff` occurs 0×; A.1.5 `:65` has no such module item |
| `fork join_any` · `fork join_none` | A.6.3 `par_block:529` — **`join` is the only terminator** |
| `generate rand integer x;` | `rand` occurs 0×; 1364-2005 has no checker construct |
| `a.sum with (item)` · `{ << { a } }` | `with` occurs 0×, and there is no streaming operator at all |

**THE POPULATION WAS MEASURED, NOT LISTED — and that is the substantive part of this release.**
The previous release fixed one over-acceptance found by hand. This one is backed by a new
instrument, `stimuli/sv/v2005_keyword_faithfulness_census.py`, which crosses profile-filtered
reachability against IEEE 1364-2005 **Annex B** — a closed, normative keyword list — and then
adjudicates every candidate on the shipped parser with three independent legs. It read **14
confirmed over-acceptances out of 24 candidates** before this release and **1** after.

⭐ **`lifetime` itself is NOT gated, and that is the load-bearing distinction.** IEEE 1364-2005
A.2.6 `:254` and A.2.7 `:275` give `function` and `task` a legitimate `[ automatic ]`. The gate sits
on the `( lifetime )?` **optional** at the declaration and module-header sites — the `1.0.196`
idiom — and two accepting controls are pinned to fail if that is ever broken.

⭐ **The streaming concatenation was found outside the census**, because `<<` and `>>` are
operators and a keyword-level lens cannot see a construct made only of punctuation.

**Consumer impact: none on `sv_2017` / `sv_2023`.** Each gate is a bare pass-through alias and the
reference site keeps its own return annotation, so no branch index moves and the typed AST is
byte-identical — the AST-shape contract gate passes 18/18 and the AST-dump schema stays at **26**.
The recognized cert union moves `total` 1388 → 1401 and `union_witness` 1380 → 1393, both **+13 =
exactly the rules added**, with `union_unknown` still **0**.

⚠️ **Two known over-acceptances remain on `verilog_2005` and are stated rather than implied**:
`class_qualifier` as an expression prefix (an IEEE 1800 *nonterminal* PGEN extracted as a keyword —
the remedy is deletion in every profile, not a profile gate) and `reg [7:0][3:0] r;` (A.2.1.3
`reg_declaration:172` carries exactly **one** `[ range ]`; a cardinality defect needs an alternative
split, not a terminal gate). Both stay pinned as watched over-acceptances.

### 1.0.196 / Contract 1.0.196 — SV-CORPUS-GRAD.13e.4 (`PGEN-SV-CORPUS-GRAD-0288`, 2026-08-24), ledger `SV-0069` (`Released`): **AN INTEGER OR TIME DECLARATION COULD CARRY `signed` / `unsigned` UNDER `verilog_2005` (GRAMMAR; `verilog_2005` ONLY — a NARROW; SCHEMA UNCHANGED at 26)**

⛔ **This release narrows one profile and touches nothing else.** IEEE 1364-2005 A.2.1.3 spells the
two declarations plainly — `integer_declaration ::= integer list_of_variable_identifiers ;` and
`time_declaration ::= time list_of_variable_identifiers ;` — and A.2.8's block forms are the same
shape. Neither `signing` nor `integer_atom_type` appears **anywhere** in that Annex; signing on an
integer atom is IEEE 1800 A.2.2.1 only. PGEN spelled the rule the way **IEEE 1800** does,
`integer_atom_type ( signing )?`, with no profile gate and at **two** sites, so under the strict
Verilog-2005 profile `integer unsigned u;`, `integer signed i;`, `time signed t;` and
`time unsigned t;` all parsed — at module scope, inside a named block, in a `function` and in a
`task`. Ten spellings the standard cannot derive.

**THE FIX GATES THE OPTIONAL, NOT THE ALTERNATIVE — and that is the difference from `1.0.195`.**
`integer_atom_type` **must** remain reachable under `verilog_2005`: that admission is exactly what
makes plain `integer i;` and `time t;` parse. So the profile split that fixed the task-port defect
one release earlier is the wrong instrument here. Instead a single new rule,
`integer_atom_signing_sv_only := signing` gated to `sv_2017`+`sv_2023`, is referenced *inside* the
optional at both sites. No `data_type` alternative is added, no branch index moves, and the
`signing` field keeps its place in the AST — it is simply always absent under `verilog_2005`.

**MEASURED, in both directions and both editions.** Under `verilog_2005` all ten illegal spellings
now REJECT, while everything the standard does derive still parses — `integer i;`, `time t;`,
`integer j = 1;`, `reg signed [7:0] r;`, `real`, `realtime`, `event`, and the block-scope forms.
Under `sv_2017` and `sv_2023` every one of those spellings still ACCEPTS. The typed AST was proven
**byte-identical** across the change on eleven SystemVerilog inputs before any code was generated.
Gates: `ast_shape_contract_gate` **18/18, drift 0**; `verilog_2005_conformance_gate` **GREEN** —
**255** file×profile checks / **0 mismatches** (fifteen of them new and two-sided: reject under
Verilog-2005, accept under both SystemVerilog profiles), `profile_orphans=0`, and certificate
coverage **byte-identical to the previous baseline** at seeds 0/7/42;
`sv_cert_recognized_union_gate` **GREEN** — canonical `UNKNOWN=11`, **union `UNKNOWN=0`, residual
`[]`**; `sv_external_corpus_triage_gate` **14/14, `parse_fail_total=0`**.

⭐ **The external corpus moved ZERO rows, and that is the finding rather than a disappointment.**
Both raw outcome files and both adjudication manifests are byte-identical across the change, against
a freshly re-measured run whose recorded parser and grammar hashes both moved. A corpus of code
written to be *compiled* cannot see this class of defect: nobody hands `integer unsigned` to a
Verilog-2005 tool, so no tool ever complained and no corpus row ever existed. The defect was found by
reading the standard against the grammar, not by burning down a residual.

**NO consumer migration, and the schema does not move.** `sv_2017` and `sv_2023` are unchanged in
accept set *and* in AST. On `verilog_2005`, code that relied on the signing was relying on
SystemVerilog, and the fix is to drop it — IEEE 1364-2005 gives `integer` and `time` no signed form.

⚠️ **What is still over-accepted here, stated rather than left to be discovered:**
`enum integer signed { A } e;` still parses under `verilog_2005`, because `enum_base_type` carries
its own optional signing and the whole `enum` construct is a *separate* over-acceptance on that
profile — gating the signing inside it would fix a leaf of a branch that has to be removed whole.
Eight further constructs were measured accepting under `verilog_2005` and are open, not overlooked:
`string`, `chandle`, `virtual interface`, `const`, `var`, `automatic`, `static`, and a second packed
dimension on `reg`. IEEE 1364-2005 `function_port_list` also still admits an `output`. All are
tracked leaves.

### 1.0.195 / Contract 1.0.195 — SV-CORPUS-GRAD.13e.5 (`PGEN-SV-CORPUS-GRAD-0287`, 2026-08-24), ledger `SV-0068` (`Released`): **A TASK OR FUNCTION PORT ITEM COULD START WITH NO DIRECTION UNDER `verilog_2005` (GRAMMAR; `verilog_2005` ONLY — a NARROW; SCHEMA UNCHANGED at 26)**

⛔ **This release narrows one profile and touches nothing else.** IEEE 1364-2005 A.2.7 spells
`task_port_item` as one of exactly three `tf_*_declaration`s, and each of those begins with its
`input` / `output` / `inout` keyword — there is no directionless alternative, and `function_port_list`
is narrower still. PGEN spelled the rule the way **IEEE 1800** does, with the direction optional and
inherited from the previous item, and applied that spelling to **every** profile. So under the strict
Verilog-2005 profile `task t(a, b);` and `task t(integer a, b);` parsed, and so did
`task t(input integer a, integer b);` — three constructs the standard cannot derive. Icarus Verilog
refuses all three from its own grammar action, `error: Missing task/function port direction.`

**THE FIX IS A PROFILE SPLIT.** `tf_port_item` and `tf_port_list` become gated pairs:
`tf_port_item_sv_only` / `tf_port_list_sv_only` for `sv_2017`+`sv_2023`, whose bodies are a
byte-for-byte re-spelling of what shipped, and a `verilog_2005` set in which the direction is
mandatory. Only the Verilog-2005 path changes.

⛔⛔ **THE OBVIOUS FIX IS WRONG, AND IT WAS MEASURED WRONG BEFORE IT SHIPPED.** *"Require a direction
on every item"* is the reading the defect invites, and it **rejects `task t(input integer a, b);`** —
which A.2.7 *does* derive, because `tf_input_declaration ::= input task_port_type
list_of_port_identifiers` takes a **list** of names, so one `input` legitimately covers both. Because
PGEN renders a port list the IEEE 1800 way — one item per comma — a continuation name arrives as its
own directionless item, indistinguishable by shape from the defect. That version was written, the
parser was regenerated, and the grammar-AST interpreter reported the rejection before any of it
reached a release. **The shipped rule admits a directionless item only when it is a bare port
identifier** — no data type, no `var` — **and requires the first item of the list to carry a
direction.** The refuting input is now a tracked reproducer, so the wrong shape cannot be re-derived
from the same reasoning.

**MEASURED, in both directions and both editions.** Under `verilog_2005`, five invalid forms REJECT
and ten legal ones ACCEPT — shared direction, mixed directions, an empty list, the non-ANSI
`task t; input a;` form, and four function analogues. Under `sv_2017`, every one of them still
ACCEPTS. External corpus: the `verilog_2005` lane moves `pass 2298 → 2295`, and a diff of the raw
outcomes names **exactly the three target files and nothing else**. Gates: `ast_shape_contract_gate`
**18/18, drift 0**; `verilog_2005_conformance_gate` **GREEN** — 240 file×profile checks / **0
mismatches**, `profile_orphans=0`, certificate coverage byte-identical at seeds 0/7/42;
`sv_cert_recognized_union_gate` **GREEN** — canonical `UNKNOWN=11`, **union `UNKNOWN=0`, residual
`[]`**; `sv_external_corpus_triage_gate` **14/14, `parse_fail_total=0`**.

**NO consumer migration, and the schema does not move.** Both variants emit the identical five-field
`tf_port_item` return, so a consumer reading a task or function port list sees exactly what it saw
before. `sv_2017` and `sv_2023` are unchanged in accept set *and* in AST. On `verilog_2005`, code
that relied on the omitted direction was relying on SystemVerilog, and the fix is to write the
direction the standard requires.

⚠️ **What is still over-accepted here, stated rather than left to be discovered:** IEEE 1364-2005
`function_port_list` admits **only** `tf_input_declaration`, so `function f(output integer b);`
remains accepted under `verilog_2005`; and `integer signed` / `integer unsigned` — which A.2.1.3 does
not admit either — was tracked separately and is **fixed in `1.0.196`** above (ledger `SV-0069`).
The `function_port_list` half is still an open leaf, not an oversight.

### 1.0.194 / Contract 1.0.194 — ENGINE-UNIVERSAL-SERVICES.43 (`PGEN-ENGINE-UNIVERSAL-SERVICES-0081`, 2026-08-21), ledger `SV-0067` (`Released`): **CROSSING THE RECURSION CEILING HUNG INSTEAD OF FAILING (ENGINE; all profiles; ZERO GRAMMAR BYTES; SCHEMA UNCHANGED at 26)**

⛔ **This release changes no language and no AST shape. It changes whether the parser RETURNS.** ~4 KB of legal-looking SystemVerilog — a deeply parenthesised expression — made the shipped parser run without producing a verdict at all. Measured on the release probe: **315** nested parens accepted in 0.15 s, **316** in 0.23 s, **317** in **4.90 s**, and **318 returned nothing in 30 s**. On the embedding path — the vehicle this parser ships to downstream consumers on — that is an unbounded hang holding the caller's thread, and it is *worse* than the process abort `1.3.1` (`SV-CORPUS-GRAD.8c.3`) replaced: a crash fails fast and loud, a hang does not.

**ROOT CAUSE — one field was carrying two kinds of fact.** `SV-CORPUS-GRAD.3.12` routes every recursion-guard verdict through one field, a parse-stack **frame index**, and refuses to cache a FAILURE whose block came from a frame outside the memoized rule (correct: that outcome depends on the caller's stack, which the memo key `(rule, position)` does not carry). `Infinite` and `LeftRecursive` genuinely name a frame. The whole-stack DEPTH CEILING names none, so it passed `0` — the index outside **every** rule — which makes that gate true for every rule at `entry_depth >= 2`. ⇒ **one trip of the ceiling disabled FAILURE memoisation for the entire remaining parse**, and the packrat parse degenerated into exponential backtracking over the expression cascade's alternative fan-out.

**THE FIX — split the channel.** Content-scoped verdicts keep the frame floor and `.3.12`'s rule exactly as they were. The depth ceiling gets its own monotone counter, and its failures ARE cached, under a **depth stamp**: re-entering the same rule at the same position from a deeper stack prunes everything the first attempt pruned and possibly more, so a failure at entry depth `D` holds at every depth `>= D`. The `.3.12` gate still runs first and returns early on any content-scoped block, so an entry reaches the depth channel only once content-dependence is excluded. Engine-level and grammar-agnostic — VHDL showed the identical cliff in a byte-identical parser, which is what proved the class was not SystemVerilog's.

⭐⭐ **THE ACCEPT SET DID NOT NARROW — IT WIDENED, and that was the surprise.** SV **317** goes 4.90 s → **0.14 s with its verdict unchanged**; SV **318**, **319** and **320** go from *no result in 30 s* to **ACCEPTED in 0.14 s**. They were never rejections — the runaway search never reached a verdict. Over-ceiling input (350 parens and beyond) now returns a clean `E_PARSE_FAILURE` in bounded, **input-length-independent** time: 800 and 2000 parens cost the same, which is the proof the search is bounded rather than merely slower.

**NO consumer migration.** No production changed, no AST shape changed, and a 60-file corpus census reads `0 depth-ceiling rejections` on every file — ordinary input does not reach the changed code path at all. Verified byte-identical interpreter/parser equivalence at seeds 0/7/42, `ast_shape_contract_gate` 18/18 drift 0, `sv_external_corpus_triage_gate` 14/14 `parse_fail_total=0`. Full detail in ledger row `SV-0067`; the VHDL twin is `VHDL-0004`.

### 1.0.193 / Contract 1.0.193 — SV-CORPUS-GRAD.13c.2y (`PGEN-SV-CORPUS-GRAD-0261`, 2026-08-20), ledger `SV-0066` (`Released`): **AN IEEE 1800 KEYWORD WAS BOUND TO THE ARROW TOKEN OF THE SAME NAME (GRAMMAR; `sv_2017`+`sv_2023`; SCHEMA 25 → 26)**

`grammars/systemverilog.ebnf` has defined `implies := trivia "->"` — a token rule named after a **reserved** keyword — since long before IEEE 1800 Annex A.2.10 was extracted into it. Extraction resolved A.2.10's property operator `implies` to that existing rule instead of minting a `kw_<name>_<hash>` terminal as it did for every other property operator, so three branches of `prop_primary_sv_2017` and `prop_primary_sv_2023` were compiled against the **arrow**. Measured on both SV profiles: `a implies b` was **REJECTED** in every shape, while `-> b` and `a |= b` in property position were **ACCEPTED**, and no Annex A production reaches either. Tool-pinned rather than read off the grammar — `--trace-rules implies` prints `Terminal '->' failed at position 60 - found 'im'`, position 60 being the first byte of the keyword.

**The fix is a precedence-table question, not a token swap.** IEEE 1800-2017 **Table 16-3** (§16.12) puts `implies` in ONE right-associative group with `until` / `s_until` / `until_with` / `s_until_with` — a group this grammar already has, because `SV-0011` built the Table 16-3 cascade `prop_until > prop_iff > prop_or > prop_and > prop_primary`. So `kw_implies_470cec58` is minted (profile-gated SV-only; the suffix is derived by the extractor's own `sha1(name)[:8]` rule, verified by reproducing four shipped tokens exactly), joins `prop_until_*` as a fifth operator, and all three defect branches are deleted. ⛔ `implies` **is** reserved — IEEE 1800-2023 Annex B lists it between `implements` and `import`, which is also exactly where the token sits in the grammar — so minting it cannot capture a legal identifier. That was checked first, because `SV-0065` one release earlier was bitten by the opposite case (`randomize` is *not* reserved).

**MEASURED.** `a implies b` REJECT→ACCEPT in three shapes on both profiles; the precedence probes `a implies b until c` / `a until b implies c` / `a iff b implies c` parse with Table 16-3 associativity; `-> b` and `a |= b` ACCEPT→REJECT; the 15 other A.2.10 rows and the `a -> b` **expression** control unchanged; all 23 probes still REJECT under `verilog_2005`. Certificate coverage, byte-identical across seeds 0/7/42: canonical `1434/8/1362/64` → `1385/7/1366/12`, union `1434/8/1373/53` → **`1385/7/1377/1`**, residual `["known_unscoped_property_identifier"]`, `sample_parse_failures=0`. `sv_external_corpus_triage_gate` **14/14, `parse_fail_total=0`**; `ast_shape_contract_gate` **18/18, drift 0**; `--lint-grammar` clean with `left_recursion_unhandled=0` held.

⭐⭐ **The headline is not the operator, it is the 52.** `| property_expr implies property_expr` was the **only** left-recursive alternative in `prop_primary_*`, so moving `implies` to its correct precedence level dissolved the `property_expr` indirect-left-recursion knot entirely: `indirect_eliminated_base_rules 3 → 2`, `indirect_clone_rules 24 → 12`, `indirect_guard_chains 3 → 2` at `surviving_cycle_rules=0`. The 43 `property_expr_lr_*` rules that dominated the certificate residual are no longer emitted, and the 9 SVA operator tokens now witness. ⛔ **Nothing was deleted from the grammar**: those rules are build-time temporaries the left-recursion eliminator mints, present in no `.ebnf` file at any commit — the SOURCE grammar goes **1484 → 1485** (+1 token, zero rules removed), while the POST-elimination census falls **1611 → 1516**, declared LR-family rule names **127 → 31**, and the generated parser sheds **8.41 MB (−6.0 %)**. An LRM-fidelity fix that also removed engine cost.

⭐ **Real-world confirmation from a control this repository did not write.** `stimuli/sv/characterization/positions.tsv` recorded Verilator's `t_property_unsup.v` stuck at byte **2452** before this work — which is `assert property (counter == 1 implies eventually[1: 2] counter == 3);`, annotated `// expected to pass` by Verilator itself. The file now parses in full, while the three other `implies`-bearing corpus files remain stuck at their exact recorded offsets, each on a named unrelated construct.

⛔ **The over-acceptance half had been named in prose four weeks earlier and left standing.** `SV-0039`'s own root cause (2026-07-23) reads *"the extractor split the `|`-prefixed operators on `|` … leaving the mangled `implies` (`->`) / `or_assign` (`|=`) remnants in `prop_primary`"*. It added the correct `|->` / `|=>` branches and removed neither remnant. A defect written into a fix's prose is not tracked work.

**Migration.** `prop_primary_sv_2017`/`_sv_2023` no longer emit `{kind:"implies_unary", body}`, `{kind:"sequence_or_assign", lhs, rhs}` or `{kind:"implies_binary", lhs, rhs}`; a consumer exhaustively matching those kinds drops all three, and one walking property operators reads `{kind:"implies", lhs, rhs}` from `prop_until_<profile>` beside its four Table 16-3 siblings. Full matrix in ledger row `SV-0066`.

### 1.0.192 / Contract 1.0.192 — SV-CORPUS-GRAD.13c.2v (`PGEN-SV-CORPUS-GRAD-0256`, 2026-08-20), ledger `SV-0065` (`Released`): **A SCOPE RANDOMIZE CARRYING A CONSTRAINT BLOCK WAS UNREACHABLE FROM EVERY EXPRESSION (GRAMMAR; `sv_2017`+`sv_2023`; SCHEMA UNCHANGED at 25)**

IEEE 1800-2017/2023 A.8.4 spells `primary`'s call alternative `function_subroutine_call`, and A.8.2 expands that to `subroutine_call ::= tf_call | system_tf_call | method_call | [ std :: ] randomize_call`. PGEN renders it as `call_primary` — the postfix-chain rule the left-recursion lift authored — and none of its eleven alternatives is `randomize_call`. That alternative lived only on the sibling wrapper `function_subroutine_call`, which no expression path reaches.

**The asymmetry made the diagnosis certain rather than plausible.** The STATEMENT path (`subroutine_call`) and the CONSTANT path (`constant_function_call`) both list the qualified `randomize_call` explicitly, so `initial randomize(a, b) with { a < b; };` parsed as a statement while the identical text failed as an expression. ⛔ And `std::randomize(a, b)` *without* the `with` clause parsed all along — through `tf_call`, whose identifier matches `randomize` because IEEE 1800 Annex B does **not** reserve it (it reserves `rand`, `randc`, `randcase`, `randsequence`). `tf_call` has no `with` tail, so the construct became unparseable at exactly the moment a constraint block appeared.

**What consumers gain.** `if (std::randomize(a, b) with { a < b; })`, `v = randomize(a, b) with { … };`, the parenthesised identifier list `randomize(a) with (a) { … }`, and the three host contexts real constrained-random code uses — negated inside an `if`, discarded through `void'(…)`, and as an immediate assertion's expression — all go REJECT → ACCEPT on `sv_2017` and `sv_2023`. `verilog_2005` is UNMOVED and that is enforced rather than hoped: the fix is lifted into a profile-gated `scope_randomize_sv_only` rather than added inline, and the `verilog_2005` adjudication manifest is byte-identical end to end, 0 of 2 459 rows moved.

⛔⛔ **POSITION IS PART OF THE FIX, and the first spelling was wrong in a way no verdict could see.** Every part of `randomize_call` is optional, so the rule matches a bare `randomize`. Placed mid-list it captured `randomize` used as an ordinary user identifier — legal under Annex B. The verdict never moved; only the AST did. The alternative is therefore `primary`'s **LAST**, which is also the cheaper spelling. The guard is a pinned control (`control_randomize_as_user_identifier.sv`) reading `scope_randomize=0 / hierarchical=4` on the shipped grammar against the broken spelling's `1 / 3`.

**Schema stays `25`, measured over the whole pinned manifest rather than argued:** `widen=6 narrow=0 shape=0` across `repro_checks=180`. ⚠️ The new `{kind: "scope_randomize"}` is a NEW node a consumer may now encounter in expression position — additive vocabulary, not a migration, but a consumer exhaustively matching `primary` kinds must add an arm for it. Corpus: SV lane pass **9 787 → 9 793**, axis-2 bar **282 → 275**. Full detail in ledger row `SV-0065`.

### 1.0.191 / Contract 1.0.191 — SV-CORPUS-GRAD.13c.2s (`PGEN-SV-CORPUS-GRAD-0242`, 2026-08-19), ledger `SV-0063` + `SV-0064` (`Released`): **A GREEDY `( X )*` IN FRONT OF AN OPTIONAL `X`-SHAPED TAIL, IN FOUR CONSTRUCTS (GRAMMAR; `sv_2017`+`sv_2023`; SCHEMA UNCHANGED at 25)**

The same starving-star mechanism as `SV-0060` one release earlier, at a different severity: there the starved element was MANDATORY so the rule matched nothing; here it is OPTIONAL, so each rule matched everything *except* the form the tail exists for.

**`SV-0063` — a system task/function call could never reach its CLOCKING EVENT argument.** IEEE 1800-2017 A.8.2 writes `system_tf_identifier ( expression { , [ expression ] } [ , [ clocking_event ] ] )`. PGEN spelled the middle group as a bare `( comma ( expression )? )*`, whose body is nullable after the comma — so on `$rose(a, @(posedge clk))` the star ate the comma, its inner optional matched EMPTY, and the clocking-event group then needed a comma and saw `@`. Every `$rose`/`$past`/`$fell`/`$stable` call carrying the sampled-value clocking argument of §16.9.3 REJECTED.

**`SV-0064` — a `let`, `property` or `sequence` instance could never take a NAMED argument after a positional one.** A.2.10 gives all three `[ X_actual_arg ] { , [ X_actual_arg ] } { , . identifier ( [ X_actual_arg ] ) }`, and all three carried the same starving star. ⛔ **This one was MASKED, which is why it needed measuring rather than reading.** The general `list_of_arguments` route rescues a mixed list *while every positional argument is also a plain expression*, so `sq(a, .q(b))` parsed all along — through the general production rather than the sequence-specific one. The moment a positional argument is something only the family rule can parse, both routes fail: `sq(a ##1 b, .q(d))` and `pr(a ##1 b, .q(d))` REJECTED, while the positional-only control `sq(a ##1 b, d)` parsed on both sides.

**Schema stays `25`, measured rather than argued:** every one of the **155** pinned reproducer checks was replayed against the pre-fix and post-fix grammars on both axes — `widen=2 narrow=0 shape=0`. ⭐ Deliberately still REJECTED and pinned so it stays that way: `$rose(a, @(posedge clk), b)` (A.8.2 puts the clocking event LAST) and `$rose(a, @)`, a bare `@` that is neither an expression nor a `clocking_event`.

### 1.0.190 / Contract 1.0.190 — SV-CORPUS-GRAD.13c.2k + .13c.2t (`PGEN-SV-CORPUS-GRAD-0237`, 2026-08-19), ledger `SV-0061` + `SV-0062` (`Released`): **A RESERVED KEYWORD PARSED IN EVERY NON-FINAL COMPONENT OF A HIERARCHICAL PATH — AND THE ASSIGNMENT-PATTERN KEY WAS NARROWER THAN THE STANDARD'S OWN EXAMPLE (GRAMMAR; all profiles; SCHEMA 24 → 25)**

Two defects landed together, one strictly-LESS-permissive and one strictly-MORE.

**`SV-0061` — the leak was POSITIONAL, not a missing keyword table.** IEEE 1800-2017 A.9.3 writes `hierarchical_identifier ::= { identifier [ [ constant_expression ] ] . } identifier` where every `identifier` is subject to §5.6.2, and PGEN guarded only the tail. So `ral.module[0].g()`, `ral.module.g()` and `module.g()` all parsed, while the method name, the final component and a declared name correctly refused the same token. The same hole admitted a reserved word as an instance name, an attribute name and a generate label. The exclusion now lives inside `identifier` itself.

⚠️ **CONSUMER ACTION — this is the direction that BREAKS a consumer, and it is the first such direction this contract has published for SystemVerilog.** Text that parsed under `1.0.183` now REJECTS: any hierarchical path with a reserved word in a non-final component, any instance named with a reserved word, any attribute or generate label so named. Sixteen pinned witnesses move ACCEPT→REJECT across the three profiles, and **six third-party corpus files that parsed on 2026-08-18 do not parse today** — every one of them correctly, adjudicated against the tracked LRMs file by file. ⭐ Two of the six are DIALECT PAIRS rather than plain rejections: `(* type=1 *)` and `begin:byte` REJECT under `sv_2017`/`sv_2023` and **ACCEPT under `verilog_2005`**, because `type` and `byte` are in IEEE 1800's Annex B and in neither case in IEEE 1364-2005's.

**`SV-0062` — `assignment_pattern_key` was narrower than the standard's own worked example.** Annex A gives `assignment_pattern_key ::= simple_type | default`, and `simple_type` cannot derive `string`; §10.9.2 prints `s2 = '{int:1, default:0, string:""};` as a worked example. The rule now derives the UNION of the two normative surfaces, so the example parses through the TYPE-key route instead of riding `member_identifier`'s acceptance of a reserved word — which is what `SV-0061` was about to take away.

⚠️ **WHY THE SCHEMA MOVES — a REPLACED shape.** An assignment-pattern key that names a type no longer arrives as `{kind:"member", body:"int"}`; it arrives as `{kind:"pattern", body:{kind:"type", body:{kind:"integer", body:{kind:"atom", body:{kind:"int"}}}}}`, and the `default` key moves from `{kind:"member", body:"default"}` to `{kind:"pattern", body:{kind:"default"}}`. **Migration:** a consumer reading the key as a bare string must read the typed node. ⭐ Cost, since a strictness fix is normally assumed to cost: the shipped spelling measures `entries` **−1.07 %**, `memo_hits` **−2.41 %** and `committed` **−5.11 %**. All three binding counters FALL.

### 1.0.189 / Contract 1.0.189 — SV-CORPUS-GRAD.13c.2q + .13c.2r (`PGEN-SV-CORPUS-GRAD-0233`, 2026-08-19), ledger `SV-0059` + `SV-0060` (`Released`): **EVERY ENABLE-GATE, PASS-ENABLE-SWITCH AND MULTI-OUTPUT GATE INSTANTIATION PARSED AS A UDP INSTANTIATION — AND THE VERDICT NEVER MOVED (GRAMMAR; all profiles; SCHEMA 23 → 24)**

⛔⛔ **THE VERDICT NEVER MOVED, AND THAT IS THE ENTRY'S POINT.** The text was accepted before and is accepted now; what changed is the production it travels, and no pass/fail oracle in this repository could see that. This is the entry to read if you want to know why an accept-set-only watch is not sufficient for a parser contract.

**`SV-0059` — eight gate/switch keywords lost their trailing DIGIT in extraction.** All three tracked LRMs write `enable_gatetype ::= bufif0 | bufif1 | notif0 | notif1` and `pass_en_switchtype ::= tranif0 | tranif1 | rtranif1 | rtranif0`; the extraction collapsed each quad to its alphabetic STEM, leaving `/bufif\b/`, `/notif\b/`, `/tranif\b/`, `/rtranif\b/` — **unmatchable by construction**, because `\b` demands a word boundary and `0` is a word character. The contrast is what makes it a targeted failure rather than a general one: `n_input_gatetype`, `mos_switchtype`, `cmos_switchtype` and `pass_switchtype` are all complete and all worked. The collapse happened exactly where the alternatives differ only by a trailing digit.

**`SV-0060` — a greedy `( X )*` immediately before a MANDATORY `X`-shaped tail.** `n_output_gate_instance` was spelled `… output_terminal ( comma output_terminal )* comma input_terminal rparen`; `output_terminal := net_lvalue` and `input_terminal := expression` overlap, and the engine does not backtrack into a committed repetition, so the star ate the mandatory tail whenever `net_lvalue` could match a prefix of the final terminal — i.e. for the COMMON form. Its sibling `n_input_gate_instance` was unaffected because its star sits at the END. Fixed with one lookahead, `( comma output_terminal &comma )*`.

⚠️ **CONSUMER ACTION — a REPLACED node type, on all three profiles. This is the largest consumer-visible change in the batch and it carries no accept-set signal at all.** `bufif0 g(o,i,e);`, `tranif0 g(o,i,e);`, `buf g(o,o2,i);` and `not g(o,i);` used to arrive as `udp_instantiation` with the terminals in an undifferentiated `instances[].inputs[]` list. They now arrive as `gate_instantiation`, whose `body` carries `gatetype: {kind:"bufif0"}` and whose instances carry **named, role-separated** `output`, `input` and `enable` fields. **Migration:** a consumer that matched `udp_instantiation` to find gates must match `gate_instantiation`; one that read positional terminals must read the named ones. Twelve pinned witnesses move on this axis and **zero** move a verdict.

### 1.0.188 / Contract 1.0.188 — SV-CORPUS-GRAD.13c.2o (`PGEN-SV-CORPUS-GRAD-0232`, 2026-08-18), ledger `SV-0058` (`Released`): **A FOOTNOTE MARKER EMITTED AS A MANDATORY TOKEN (GRAMMAR; `sv_2023` ONLY; SCHEMA UNCHANGED at 23)**

PGEN could not parse IEEE 1800-2023 covergroup inheritance unless you typed the LRM's FOOTNOTE NUMBER into your source. A.2.11 of the 2023 edition prints the alternative as `| covergroup extends covergroup_identifier ;29`, where `29` is a footnote marker whose text reads *"The extends specification of covergroup is allowed only within a class."* The extraction emitted the marker as a token — `kw_n_29_7719a1c7 := trivia /29\b/` — and placed it in the sequence with **no `?`**. Under `sv_2023` the only spelling of covergroup inheritance PGEN accepted was the one no Annex A production derives.

⚠️ **CONSUMER ACTION — `sv_2023` only, both directions.** `covergroup extends base;` inside a class flips REJECT→ACCEPT; `covergroup extends base;29` flips ACCEPT→REJECT. ⭐ `sv_2017` REJECTS both, before and after, and that was CHECKED rather than assumed: `covergroup extends` appears in 1800-2023's Annex A and **nowhere** in the 1800-2017 source. `verilog_2005` has no covergroups.

**Schema stays `23`:** the accepted form was 100 % unparseable before, so no witnessed shape moves; the footnote form is REMOVED rather than replaced, since that text no longer parses at all. The positional references renumbered `6→5` / `8→7` with every AST field name and source unchanged, and zero pinned witnesses moved on the shape axis.

### 1.0.187 / Contract 1.0.187 — SV-CORPUS-GRAD.13c.2j (`PGEN-SV-CORPUS-GRAD-0229`, 2026-08-18), ledger `SV-0057` (`Released`): **ONE ANNEX A PRODUCTION RENDERED FOUR TIMES, AND ONLY THE CANONICAL COPY WAS MAINTAINED (GRAMMAR; `sv_2017`+`sv_2023`; SCHEMA UNCHANGED at 23)**

A CLASS-SCOPED name could not be the receiver of a method call. IEEE 1800 A.8.4 writes the scope prefix once — `[ class_qualifier | package_scope ]` — and `grammars/systemverilog.ebnf` renders it **four times**. The canonical rendering `primary_hier_scope_prefix` gained a `class_scope` branch for a measured defect; the three inline copies never did, so `p::base::m.g()` REJECTED while both halves (`p::base::m` as a value, and `h.g()` through a handle) parsed alone.

⚠️ **CONSUMER ACTION — additive, `sv_2017` + `sv_2023`.** `p::base::m.g()` and `p::base::h.arr[0].g()` flip REJECT→ACCEPT; four pinned witnesses move, all in that direction, and **zero** move a shape. **Schema stays `23`:** the alternative added is one the parser could not previously reach, so no previously-emitted shape is replaced.

⛔ **What this release does NOT do, stated so it is not inferred:** the four renderings are still four. Detection of their drift is now automated (`docs/tasks/artifacts/sv_corpus_grad/scope_prefix_rendering_drift/`), and unifying them is an AST-shape change tracked as `SV-CORPUS-GRAD.13c.2n`, deliberately NOT taken before the SV release.

### 1.0.186 / Contract 1.0.186 — SV-CORPUS-GRAD.13c.2c (`PGEN-SV-CORPUS-GRAD-0227`, 2026-08-18), ledger `SV-0056` (`Released`): **A NEGATIVE LOOKAHEAD THAT COULD NEVER PASS, SO A LOOP RAN ZERO ITERATIONS FOR THE LIFE OF THE PARSER (GRAMMAR; all profiles; SCHEMA 22 → 23)**

`split_hierarchical_callable_receiver`'s member loop was guarded by `!callable_method_call_body`, meant to stop the receiver before the method name — but IEEE 1800 A.8.2 writes `array_manipulation_call ::= array_method_name { attribute_instance } [ ( list_of_arguments ) ] [ with ( expression ) ]` with the parens **optional**, so a bare member name already IS a `callable_method_call_body` and the guard fired on every identifier. ⇒ the loop ran **zero iterations for the life of the parser**, and multi-component receivers survived only through `method_call`'s `( dot method_call_body )*` chain, which carries no bit-select. That is exactly why `a.b.g()` parsed and `a.b[0].g()` did not.

⚠️ **CONSUMER ACTION — all three profiles, and the shape change reaches inputs that already parsed.** `ral.arr[0].g()` flips REJECT→ACCEPT, and so does `top.u1[0].t;` — an IEEE 1364-2005 §12.4 indexed hierarchical task enable that had been rejecting in **plain Verilog**, with nothing pointing at it. Ten pinned witnesses move a verdict.

⚠️ **WHY THE SCHEMA MOVES.** For a hierarchical call that parsed before, the components no longer travel `method_call`'s chain. Measured on `top.u1.t;`, all three profiles: the receiver's `path` was `[]` with `u1` and `t` appearing inside `chain: [[{kind:"dot"}, {kind:"built_in", body:{kind:"array_manipulation", …}}], …]`; it is now `chain: []` with `path: [[{body:"top"}, {bits:[]}, {kind:"dot"}, []]]` and the callable name `t`. **Migration:** a consumer walking `chain` to recover a hierarchical name must walk `path`.

### 1.0.185 / Contract 1.0.185 — SV-CORPUS-GRAD.13c.2f slice 4 (`PGEN-SV-CORPUS-GRAD-0221`, 2026-08-17), ledger `SV-0055` (`Released`): **A LITERAL `$` TRANSLITERATED TO THE WORD `_dollar`, AND TWO NONTERMINALS FLATTENED INTO A TOKEN (GRAMMAR; all profiles; SCHEMA UNCHANGED at 22)**

The two `PATHPULSE$` tokens of IEEE 1800 A.7.5 were **transliterated rather than transcribed**. The extraction turned the literal `$` into the word `_dollar` and, for the path-specific form, flattened two NONTERMINAL references into the token text as well, leaving `/PATHPULSE_dollar_specify_input_terminal_descriptor_dollar_specify_output_terminal_descriptor\b/`. Both alternatives of `pulse_control_specparam` could therefore fire only on source containing those literal characters, which no SystemVerilog source does. The rule NAMES are faithful transliterations of the LRM spelling and are kept; it is the matched TEXT that was wrong.

⚠️ **CONSUMER ACTION — both directions, all three profiles.** IEEE 1800-2023 §30.7.1's own example and four neighbouring spellings flip REJECT→ACCEPT (fifteen pinned witnesses, five reproducers × three profiles). ⛔ **And a spelling NARROWS**: `specparam PATHPULSE_dollar = (1, 2);` was accepted on all three profiles and now REJECTS on all three — correctly, since `PATHPULSE_dollar` is an ordinary identifier and `(1, 2)` is not a `constant_mintypmax_expression`. It parsed only because the transliterated word was a magic token.

**Schema stays `22`:** the accepted forms were 100 % unparseable before, and the narrowed form is REMOVED rather than replaced; zero pinned witnesses move on the shape axis. ⛔ One regex per token, deliberately, rather than composing `PATHPULSE$` + descriptor + `$` + descriptor from grammar elements: `trivia` skips layout before every element, so the composed form would also accept `PATHPULSE$ clk $ q`, and §A.9.3 makes `PATHPULSE$clk$q` ONE simple identifier. A contiguous regex cannot match the illegal spacing by construction.

### 1.0.184 / Contract 1.0.184 — SV-CORPUS-GRAD.13c.2d (`PGEN-SV-CORPUS-GRAD-0220`, 2026-08-17), ledger `SV-0054` (`Released`): **AN LRM TYPO TRANSCRIBED AS A LITERAL KEYWORD (GRAMMAR; `sv_2017` ONLY; SCHEMA 21 → 22)**

IEEE 1800-2017 A.2.11 writes `function_declaraton` — corrected to `function_declaration` in 1800-2023 — and the extraction emitted `kw_function_declaraton_06b7ed29 := trivia /function_declaraton\b/`. An alternative that can only fire on source literally containing those characters is unreachable, so §19.6.1's own example — a `cross` body declaring a function and using it in a `bins` — was REJECTED under `sv_2017` while `sv_2023` accepted it.

⭐ **The `;` had to move with it, which is why this is not a one-word edit.** `cross_body_sv_2017` read `lbrace ( cross_body_item semi )* rbrace`, demanding a `;` after EVERY item, so merely re-pointing the reference would then have required `endfunction ;`, which §19.6.1 does not write. The repair mirrors the `sv_2023` pair, which already carried the `;` on the `bins_selection_or_option` alternative rather than on the loop.

⚠️ **CONSUMER ACTION — `sv_2017` ONLY, and it is a REPLACED shape, which is WHY THE SCHEMA MOVES.** Each element of `cross_body_sv_2017`'s `items` array was a two-element pair `[ <cross_body_item envelope>, {kind:"semi"} ]`; it is now the `cross_body_item` envelope itself and the `;` token is gone. **Migration:** a consumer reading `items[i][0]` must read `items[i]`. Seven pinned witnesses move on this axis. **Measured per profile rather than reasoned:** under `sv_2023` an already-parsing cross body's AST is byte-identical across the fix and §19.6.1's example already parsed; under `verilog_2005` covergroups do not exist and both spellings reject on both sides. ⚠️ The accepts-invalid half of `.13c.2a` is PRESERVED by this shape rather than re-opened: `cross_body_item` is non-nullable and a bare `;` matches neither alternative, so `option.weight = 2;;` still fails.

### 1.0.183 / Contract 1.0.183 — SV-CORPUS-GRAD.13c.2e (`PGEN-SV-CORPUS-GRAD-0214`, 2026-08-12), ledger `SV-0053` (`Released`): **THE COVERGROUP SELECT CONDITION'S LITERAL LRM BRACES — THE SAME DEFECT AS `SV-0049`, ONE CLAUSE AWAY, THREE DAYS LATER (GRAMMAR; `sv_2017`+`sv_2023`; SCHEMA 20 → 21)**

**What changed.** `select_condition` (`grammars/systemverilog.ebnf:5259`) rendered IEEE
1800-2017 A.2.11's `select_condition ::= binsof ( bins_expression ) [ intersect {
covergroup_range_list } ]` as `( kw_intersect covergroup_range_list* )?`. Those braces are
**literal SystemVerilog**, and Annex A spells repetition with the same two characters — the
identical collision `SV-0049` had just fixed in `expression_or_dist`. Restored to
`( kw_intersect lbrace covergroup_range_list rbrace )?`; the `*` retires with it, since
`covergroup_range_list` already carries its own `( comma covergroup_value_range )*`. The
`intersect: $5` annotation is unchanged — the group is captured whole, exactly as the
already-correct sibling captures `dist: $2`.

**Three symptoms from one dropped delimiter.** (1) **Under-acceptance** — `covergroup_range_list
→ covergroup_value_range → covergroup_expression` reaches the general expression grammar, so
`intersect { 5, 6 }` still "worked" as a CONCATENATION; but a RANGE is not an expression, so the
standard's own §19.6.2 example `binsof(a) intersect { 5, [1:3] }` had no derivation and rejected.
(2) ⚠️ **Silent mis-parse, invisible to every pass/fail oracle** — with no closing-brace terminal
to stop it, the list ran past its end: `binsof(ca) intersect { 1 } && binsof(cb)` parsed with the
whole remainder absorbed, producing ONE `condition` node and **ZERO `and` nodes**. The trace names
it exactly — `covergroup_range_list` consumed 20 bytes `' { 1 } && binsof(cb)'`. ⭐ That is why
release `1.0.182`'s revived `&&` alternative needed PARENTHESES in its reproducer to demonstrate
anything: **one clause carried two independent defects, and the second was masking the evidence
for the first.** (3) **Over-acceptance** — the brace-less `binsof(a) intersect 5`, which A.2.11
has no production for, was accepted; it now rejects.

**SCHEMA 20 → 21**, by the test `SV-0049` states verbatim: a construct that was 100 % unparseable
cannot replace a witnessed shape, but one that PARSED before does. `intersect {5, 8}` parsed
before. The `intersect` slot goes from the 2-element `[[trivia,"intersect"], [ <one concat-valued
item> ]]` to the 4-element `[[trivia,"intersect"], {kind:"lbrace"}, [ <item>, <item> ],
{kind:"rbrace"}]`. **Migration:** read `intersect[2]` where you read `intersect[1]`; read the
individual `covergroup_value_range`s where you read one concatenation-valued item — a range now
appears there as `{kind:"range"}`, a kind that could not previously occur. The `intersect`-absent
case is unchanged (`[]`).

**MEASURED.** External corpus 16 336 files, pass **9 758 → 9 762 (+4)** with **0 pass→fail / 0
timeout / 0 crash**, and all 4 flips inside the keyed set derived *before* the run
(**flipped-but-not-keyed = 0**); `unexplained_rejects_valid` **292 → 288** with **ZERO new** by set
difference; `unexplained_accepts_invalid` **21 → 21 SET-IDENTICAL** — the control that matters for
a fix that both widens and narrows; axis-2 bar **313 → 309** with the denominator unmoved (7 556 /
46.3 % adjudicated); `sv_cert_recognized_union_gate` union UNKNOWN=0, witness 1 356, residual `[]`,
`fully_certified_via_union: true` across seeds 0/7/42 — **no re-baseline**; `--lint-grammar`
byte-identical; `sv_syntax_closure_gate` PASS with the contract byte-unchanged; adjudication oracle
`checked=26 armed=7 failures=0`; all 18 doctrines PASS. ⚠️ The over-acceptance leg is proven on a
SYNTHETIC only — the corpus carries **0** brace-less `binsof(…) intersect` rows, derived
mechanically and stated rather than implied.

⛔⛔ **THE CLASS FINDING, AND THIS TIME IT IS AN INSTRUMENT RATHER THAN A COMPLAINT.** `SV-0049`
wrote that *"the reactive posture does not converge — an exhaustive Annex-A bracket/brace sweep is
what closes the class"*, then fixed its instance and did not sweep; this is the eighth instance and
it shipped three days later. The brace half of that sweep now exists
(`docs/tasks/artifacts/sv_corpus_grad/intersect_braces/lrm_brace_transcription_sweep.py`, four
refuse-rather-than-guess controls), runs over BOTH the 2017 and 2023 LRMs, and reports
mis-transcribed literal-brace productions **2 → 0**. ⚠️ The BRACKET half is **not** closed and is
routed to `LRM-GRAMMAR-FIDELITY.1a` with its sizing measured: the brace discriminator does not
transfer, returning 83 rows / 45 productions dominated by ordinary optionals.

**LOCK ADDED:** the shape sample `select_condition_intersect_braced_list` — 0 of the then-33 locked
samples contained a `binsof`, the same blind spot `SV-0049` recorded for `dist`.

### 1.0.182 / Contract 1.0.182 — GRAMMAR-WELLFORMED.A2.5 (`PGEN-GRAMMAR-WELLFORMED-0153`, 2026-08-11), ledger `SV-0052` (`Released`): **FOUR IEEE ALTERNATIVES THAT HAD BEEN DEAD FOR THE LIFE OF THE PARSER NOW RUN (ENGINE; `sv_2017`+`sv_2023`; ZERO GRAMMAR BYTES; SCHEMA UNCHANGED at 20)**

**What changed.** PGEN's left-recursion elimination now handles the **direct** shape —
`A := A op B | …`, the self-reference written inline in the choice, which is how every standard's
Annex A writes an operator production. It previously matched only the indirect *wrapper* shape, so a
directly left-recursive alternative reached codegen intact and the runtime cycle guard **rejected**
it at the seed position. The rule still parsed its operands, so the construct looked supported until
something had to follow the first operand.

Four alternatives flip REJECT → ACCEPT: `select_expression`'s `&&`, `||` and
`with ( … ) [ matches … ]` (IEEE 1800-2017 A.2.11), and `block_event_expression`'s `or` (A.6.11,
reached from `coverage_event`'s `@@( … )` form). Each returns the AST its annotation declares,
left-nested — `{kind:"and"|"or", lhs, rhs}` and
`{kind:"with_matches", lhs, expression, matches}`.

**What a consumer must do.** Add `and` / `or` / `with_matches` arms to any exhaustive match on
`select_expression` kinds, and an `or` arm for `block_event_expression`. **The schema does not
move**, by this project's stated rule rather than by preference: all four alternatives were 100 %
unparseable before, so no previously-emitted shape is replaced — the `SV-0044` class, and the
principle `SV-0048` states verbatim (*"an additive vocabulary reachability change, not a schema
change"*). Verified: `ast_shape_contract_gate` green with its locked samples unchanged, and the two
`select_expression` inputs that parsed before still parse to the same `{kind:"condition"}` root.

⛔ **What this does NOT fix.** `binsof(a) intersect { 1 } && binsof(b)` *without* parentheses still
parses as a single `condition` with the `&&` inert — not because the continuation is dead, but
because `intersect`'s literal braces are still transcribed as EBNF repetition, so the seed swallows
the remainder (`SV-CORPUS-GRAD.13c.2a.1`, open). Parenthesize the left operand to exercise the
continuation. Operator **precedence** is also unchanged and stays a grammar concern: LR elimination
fixes the recursion, not the precedence.

**Proof.** Full external corpus 16 336 files: pass **9 752 → 9 758 (+6)** with **0 pass→fail /
0 pass→timeout / 0 pass→crash**, all six flips covergroup/cross/binsof files;
`unexplained_rejects_valid` **297 → 292** with ZERO new; `unexplained_accepts_invalid`
**21 → 21 set-identical**. Six pinned reproducers under `stimuli/sv/adjudication_repros/` carry the
`arm` that produced each ACCEPT, so the verdicts name the alternative rather than merely reporting
that the text parsed.

### 1.0.181 / Contract 1.0.181 — SV-CORPUS-GRAD.3.20 (`PGEN-SV-CORPUS-GRAD-0039`, 2026-08-09), ledger `SV-0051` (`Released`): **`verilog_2005` WAS ACCEPTING FIVE CONFIG `use`-CLAUSE FORMS IEEE 1364-2005 HAS NO PRODUCTION FOR (GRAMMAR; `verilog_2005` ONLY; SCHEMA UNCHANGED at 20)**

**What changed.** The four SystemVerilog-only parameter-override alternatives of `use_clause`
(`grammars/systemverilog.ebnf:6039`) moved into a `@profiles: ["sv_2017", "sv_2023"]` sibling,
`use_clause_param_override_sv_only`, which `use_clause` references as its first alternative. Under
`verilog_2005` the sibling is gated out of the tree, leaving the single production IEEE 1364-2005
actually declares.

**Why.** IEEE 1364-2005 gives `use_clause` exactly one alternative —
`use [library_identifier.]cell_identifier[:config]` (Annex A A.1.5, restated in 13.3) — and the
string `use #(` occurs **zero** times anywhere in the 2005 text. But `use_clause` carried no
`@profiles` directive at all, so `ast_pipeline --dump-rule-profiles` reported it
`satisfiable_under [sv_2017, sv_2023, verilog_2005]` and every override spelling was accepted under
the strict profile. Over-acceptance is a defect here, not a tolerance.

⭐ **A third defect class, distinct from the two that precede it in this ledger.** `SV-0044`/`SV-0049`
were *dropped delimiters* — the LRM was right and PGEN mis-read it. `SV-0050` was a
*self-contradicting standard* — Annex A and clause 33.4.3 of IEEE 1800 disagree. This one is neither:
PGEN read IEEE **1800** correctly and simply never asked which **edition** the construct belongs to.
An Annex-A-versus-grammar sweep of 1800 cannot find it, because 1800 is not the oracle for what
1364-2005 permits.

**Empirical pre/post** (the same 17-case matrix, now carrying a per-profile expectation column):

```text
                                    sv_2017 / sv_2023        verilog_2005
                                    before   after           before   after
use rtlLib.adder:config;            ACCEPT   ACCEPT          ACCEPT   ACCEPT   (1364-2005 legal)
use .W(8);                          ACCEPT   ACCEPT          ACCEPT   REJECT   <- tightened
use rtlLib.adder .W(8);             ACCEPT   ACCEPT          ACCEPT   REJECT   <- tightened
use #(.WIDTH(32));                  ACCEPT   ACCEPT          ACCEPT   REJECT   <- tightened
use #(32);                          REJECT   REJECT          REJECT   REJECT   (33.4.3 forbids)
adder #(8, 16) a1();                ACCEPT   ACCEPT          ACCEPT   ACCEPT   (ordinary instance)

cases differing from the LRM expectation: sv_2017 0 -> 0, sv_2023 0 -> 0, verilog_2005 9 -> 0
```

⛔ `adder #(8, 16) a1();` is the control that keeps this honest: an ordinary module instantiation's
`#( … )` parameter value assignment is legal Verilog-2001/2005 and must not move. Only the *config*
`use` clause is restricted.

**What a consumer sees.** Under `sv_2017` / `sv_2023`, **nothing** — the emitted AST is byte-identical.
A bare rule-reference alternative carrying no return annotation adds no wrapper node, which is
measured (`cmp`-identical on 15/15 accepting repro cases) rather than assumed, and confirmed
independently by the `use_clause_hash_named_override` shape lock whose observed `content_kind` did
not move. Under `verilog_2005`, a tool that was relying on PGEN to accept SystemVerilog parameter
overrides while requesting the 1364-2005 profile should request `sv_2017`/`sv_2023` — which is what
it was in fact parsing.

**Schema version:** stays at **`20`**. See *Schema Versioning* for why a restructuring is not
automatically a bump.

**Corpus proof:** **both lanes byte-inert** — 16 336 `sv_2017` files (pass 9 734 → 9 734) and 2 459
`verilog_2005` files (pass 2 181 → 2 181), **zero per-file transitions of any kind in either lane**,
both adjudication manifests `cmp`-clean, therefore **0 rows changed adjudication class anywhere**.
That was predicted before the run from the grammar's own reachability — `use_clause` is reachable
only through `config_rule_statement` inside a `config_declaration`, and **0 of the 2 459 v2005-lane
files contain the token `config`** — and then confirmed. ⚠️ A tightening that moves nothing and a fix
that silently did nothing look identical in a corpus census, which is why the repro matrix (where the
v2005 arm moves 9 → 0) is the instrument that proves the change landed.

⚠️ `defined_rule_count` **1477 → 1478**, so `sv_cert_recognized_union_gate` was re-baselined **in the
same commit** (+1 total, +1 both witness columns; proof 6, canonical UNKNOWN 11 and union UNKNOWN 0
all unchanged — fully positive), while `verilog_2005_conformance_gate` needed **no** re-baseline: its
cert census is 1122/329/779/14, unchanged, because the new rule is gated out of that profile. That
`+1 / +0` asymmetry was predicted in the task leaf and is the fingerprint that attributes the move.

### 1.0.180 / Contract 1.0.180 — SV-CORPUS-GRAD.3.19 (`PGEN-SV-CORPUS-GRAD-0037`, 2026-08-09), ledger `SV-0050` (`Released`): **THE CONFIG `use` CLAUSE COULD NOT CONSUME A `#` — AND THE REASON IS THAT IEEE 1800 CONTRADICTS ITSELF (GRAMMAR; all profiles; SCHEMA UNCHANGED at 20)**

**What changed.** `use_clause` (`grammars/systemverilog.ebnf:6039`) gained a fifth alternative,
`use #( named_parameter_assignment {, …} )[: config]`, so a configuration can override parameters
the way the LRM's own examples do.

⭐ **Why this one is unusual — the defect is in the STANDARD, not in PGEN's reading of it.** The
previous three grammar fixes in this series (`SV-0002`, `SV-0044`, `SV-0049`) were all the same
class: the LRM wrote a literal delimiter and PGEN read it as BNF metasyntax. Here PGEN's
transcription of Annex A is **correct and complete** — all three alternatives, `[lib.]` optional,
`[: config]` optional. The problem is that IEEE 1800 says two different things:

| surface | what it says | occurrences |
|---|---|---|
| Annex A / Syntax 33-4 (A.1.5) | `use_clause ::= use [lib.]cell [:config] \| use named_parameter_assignment {, …} [:config] \| use [lib.]cell named_parameter_assignment {, …} [:config]` — **no `#`** | identical in 1800-2017 and 1800-2023 |
| Clause 33.4.3, normative examples | `instance top use #(.WIDTH(32));`, `use #(.W(top.WIDTH));`, `use #(.W());`, `use #();` | **7 per revision**, in BOTH revisions |

**How that was adjudicated** — three independent grounds, so it is a reading rather than a
preference:

1. Annex A subordinates itself in its own preamble: *"The full syntax and semantics of
   SystemVerilog are not described solely using BNF. The normative text description contained
   within the clauses and annexes of this standard provide **additional details on the syntax**."*
2. The BNF-only reading makes an explicit prohibition **vacuous**. Clause 33.4.3 states
   *"Configurations may not use positional parameter notation to override parameters"* — but under
   the brace-less alternatives positional notation is not even expressible. A standard does not
   forbid what its own grammar cannot write.
3. Measured, not recalled: every one of the **12** parameter-override `use` clauses in PGEN's
   vendored external corpus is spelled `#( … )`; **zero** use the Annex-A spelling.

⇒ the parser accepts the **union** of the two normative surfaces.

**What is still rejected, on purpose.** `use #(32)` and `use #(32, .W(8))` — positional overrides,
which 33.4.3 forbids in so many words. The new alternative therefore takes
`named_parameter_assignment`, **not** `list_of_parameter_assignments`; reusing
`parameter_value_assignment` (which is literally `hash lparen ( list_of_parameter_assignments )?
rparen`) would have been the obvious move and would have silently accepted them.

**Empirical pre/post** (`--profile sv_2017`, the leaf's 17-case matrix):

```text
# before
instance top use #(.WIDTH(32));         -> REJECT, furthest_position=246 (the `#`)
instance top.a1 use #();                -> REJECT
instance top.a1 use #(32);              -> REJECT   (correct)
instance top.a1 use rtlLib.adder .W(8); -> ACCEPT   (Annex-A form)

# after
instance top use #(.WIDTH(32));         -> ACCEPT
instance top.a1 use #();                -> ACCEPT
instance top.a1 use #(32);              -> REJECT   (still correct — 33.4.3 forbids positional)
instance top.a1 use rtlLib.adder .W(8); -> ACCEPT   (unchanged)
```

**What a consumer sees.** A `config_rule_statement` of `kind: "inst_use"` (or `"cell_use"`) can now
carry a `body` containing `{kind:"hash"}`, `{kind:"lparen"}`, the named assignments, and
`{kind:"rparen"}`:

```json
{"kind": "inst_use",
 "clause": {"name": {"head": {"body": "top"}, "scope_chain": []}},
 "body": [[[], "use"], {"kind": "hash"}, {"kind": "lparen"},
          [{"name": {"body": "WIDTH"}, "value": {"…": "32"}}, []],
          {"kind": "rparen"}, []]}
```

**Schema version:** stays at **`20`**, and that is measured rather than assumed. The construct was
100 % unparseable before this release, so no previously-witnessed shape can move; the ASTs of all
**10** already-parsing cases in the repro matrix are **byte-identical** across the fix (`cmp` 10/10).

**Corpus proof:** 16 336 files, pass **9 726 → 9 734 (+8)**, **0 pass→fail / 0 pass→timeout /
0 pass→crash**; unexplained rejects-valid **304 → 296** with **zero new** by set difference;
`unexplained_accepts_invalid` set unchanged (21); the 8 flipped files set-equal to the 8 keyed
before the edit; the `verilog_2005` lane byte-inert with a `cmp`-clean manifest.
`defined_rule_count` **1477 unchanged**, so `sv_cert_recognized_union_gate` and
`verilog_2005_conformance_gate` needed **no re-baseline**. A new `use_clause_hash_named_override`
shape sample was added and **proven non-vacuous by a negative control**.

⚠️ **Known residual, tracked and being fixed next (`SV-CORPUS-GRAD.3.20`):** `use_clause` carries no
`@profiles` gate, so `verilog_2005` accepts this spelling — and the four pre-existing named-override
alternatives — none of which exist in IEEE 1364-2005, whose `use_clause` has exactly one
alternative. Pre-existing for four forms, extended to five here; measured blast radius **0 corpus
rows** (no file in the 2 459-file v2005 lane contains a config `use` clause of any spelling).

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → the 2026-08-09 `SV-CORPUS-GRAD.3.19` current-state note.

### 1.0.179 / Contract 1.0.179 — SV-CORPUS-GRAD.3.18 (`PGEN-SV-CORPUS-GRAD-0036`, 2026-08-09), ledger `SV-0049` (`Released`): **THE `dist` CONSTRAINT OPERATOR WAS WRONG THREE WAYS AT ONCE — LITERAL LRM BRACES READ AS EBNF REPETITION (GRAMMAR; `sv_2017`+`sv_2023`; SCHEMA 19 → 20)** — `expression_or_dist` (`grammars/systemverilog.ebnf:2344`) rendered IEEE 1800-2017 A.2.10's `expression_or_dist ::= expression [ dist { dist_list } ]` as `( kw_dist dist_list* )?`. Those braces are **literal SystemVerilog**, and the trap is two lines away in the LRM itself: `dist_list ::= dist_item { , dist_item }` uses the *same characters* as repetition metasyntax. ⭐ **One dropped delimiter, three symptoms — which is why the corpus scattered them across three different stuck-point clusters.** The reason is that `dist_item → value_range → expression` can itself begin with `{`, as a **concatenation**: for `{100 := 1, …}` that speculative path dies at the `:=` (the `: = NUM` cluster signature that surfaced this leaf), for `{[100:102] :/ 1}` it dies at the `[`, and for `{5, 8}` it **succeeds with the wrong tree**. So: (1) **under-acceptance** — every weighted or ranged distribution the LRM writes was unparseable, in constraint *and* assertion contexts; (2) ⚠️ **silent mis-parse** — `soft x dist {5, 8};` PARSED, but the LRM's **two** items were handed to the consumer as **one** whose value was the concatenation expression `{5, 8}` (`"kind": "concat"` in the dump). No pass/fail oracle in this repo could see that; only a shape dump can, and **no shape sample covered this rule** (0 of the then-31). A wrong tree is worse than a rejection, because the consumer has no way to notice; (3) **over-acceptance** — the brace-less `x dist 100 := 1;`, which has no production in A.2.10 at all, was accepted. **FIX** (pure grammar, existing tokens, zero new rules): restore the literal braces — `( kw_dist lbrace dist_list rbrace )?`. The `*` retires with them, since `dist_list` already carries its own `( comma dist_item )*`. **SCHEMA 19 → 20 — and unlike the same-class `SV-0044` fix, this one had to bump:** that construct was 100 % unparseable so no witnessed shape could move; this one was not. The `dist` slot goes from the 2-element `[[trivia,"dist"], [ <one concat-valued item> ]]` to the 4-element `[[trivia,"dist"], {kind:"lbrace"}, [ <item>, <item> ], {kind:"rbrace"}]`. **Migration:** read `dist[2]` where you read `dist[1]`; read individual `dist_item`s where you read one concatenation-valued item. **MEASURED:** external corpus 16 336 files, pass **9 720 → 9 726 (+6)** with **0 pass→fail / 0 pass→timeout / 0 pass→crash**; unexplained rejects-valid **310 → 304** with **ZERO new** by set difference; `unexplained_accepts_invalid` set **BYTE-IDENTICAL** (21) — the control that matters when a fix *tightens* the language; the 6 flipped files **set-equal** to the 6 keyed before the edit (**flipped-but-not-keyed = 0**); the 2 459-file `verilog_2005` lane **byte-inert** (zero transitions, manifest `cmp`-clean), which is measured rather than assumed even though constraints and SVA do not exist in IEEE 1364-2005. ⭐ `verilator/t_constraint_dist_randc_bad.v` flipping to PASS is **correct**: §18.5.4's own Limitations make `dist` on a `randc` variable an *elaboration* error, not a syntax one, so a conforming parser must parse it. ⛔⛔ **THE CLASS FINDING:** this is the **seventh** logged instance of the dropped-delimiter class and the **third in Annex A subclause A.2.10 alone** (after `boolean_abbrev` and `cycle_delay_range`/`SV-0044`). `SV-0044` recorded the same complaint and its own notes even named `dist { … }` as a residual it was setting aside; it was found again two weeks later, reactively, by a different cluster. The reactive posture does not converge — an exhaustive Annex-A bracket/brace sweep is what closes the class. **LOCK ADDED:** the shape sample `expression_or_dist_braced_list`, verified live by a negative control (breaking it fails the gate). Full matrix in ledger row `SV-0049`.

### 1.0.178 / Contract 1.0.178 — SV-CORPUS-GRAD.3.14b (`PGEN-SV-CORPUS-GRAD-0033`, 2026-08-09), ledger `SV-0048` (`Released`): **A COMPILER DIRECTIVE WAS TOLERATED ONLY AT FILE TOP LEVEL — `` `timescale `` ONE LINE INSIDE A MODULE REJECTED (GRAMMAR; cross-profile; schema 19 UNCHANGED)** — `compiler_directive` was an alternative of `source_text_item` and of **nothing else**, so no in-scope item list could reach it. ⭐ **The whole diagnosis is one line of movement:** the same directive text above `module m;` PASSES and its AST carries `{"kind":"compiler_directive"}`; moved one line inside the module it rejects at `furthest_position=9`. Same text, two placements, two verdicts ⇒ not a lexical or spelling defect. **The fix is a name WHITELIST, not blanket directive tolerance, and the reason is the transferable content of this release: the population is not uniform, and neither was the ruling that routed it.** `.3.13` handed over *"the standing law is TOLERATE"*; `.3.14a` refuted that by reading clause 22 directive by directive (tolerate 5, keep rejecting 3); `.3.14b` then refuted `.3.14a`'s own table, which had enumerated only the six names the routed population happened to contain — reading IEEE 1800-2017 clause 22 **and** IEEE 1364-2005 clause 19 end to end surfaced **two further placement-restricted families**: `` `unconnected_drive ``/`` `nounconnected_drive `` ("shall be specified outside the design element declarations", §22.9/§19.9) and `` `begin_keywords ``/`` `end_keywords `` ("can only be specified outside a design element", §22.14/§19.11). A blanket rule would have widened acceptance into **six** LRM-forbidden constructs. **Sampling a clause is not reading it.** Second, independent reason for a whitelist: a backtick line is not necessarily a directive — `` `MY_MACRO(x) `` at an item position is an unexpanded §22.13-class text MACRO, i.e. a preprocessing dependency, and a blanket rule would swallow the line, drop the items the macro expands to, and turn a correct reject into a silent pass. **TOLERATED:** `` `celldefine `` `` `endcelldefine `` `` `undef `` `` `timescale `` `` `pragma `` `` `line `` (+ `` `undefineall ``, SV profiles only — absent from 1364-2005), at module-body and class-body item positions. **STILL REJECTED, deliberately:** the six placement-restricted names, the seven text-hiding ones (`` `include ``, `` `define ``, `` `ifdef `` `` `ifndef `` `` `else `` `` `elsif `` `` `endif ``), and all unexpanded macro uses including `` `__FILE__ ``/`` `__LINE__ ``. **What consumers see:** `non_port_module_item` and `class_item` can now yield `{"kind":"compiler_directive","body":"<directive line>"}` — a kind that previously appeared only under `source_text`; a consumer exhaustively matching those item kinds must add the arm. **Why schema stays 19:** the in-scope node is byte-identical to the top-level one, so no emitted kind is added, renamed or removed — proven exhaustively over all 287 previously-passing files that contain a whitelisted directive token (287/287 dumped, 595 top-level nodes unchanged, **0** in-scope nodes). **Measured:** corpus pass **9 712 → 9 720 (+8)** with **0 pass→fail / 0 pass→timeout / 0 pass→crash** over 16 336 files, and **2 180 → 2 181 (+1)** with **0 pass→fail** over the 2 459-file `verilog_2005` lane; **accepts-invalid UNCHANGED in both lanes (21 / 14)** — the over-acceptance control. Unexplained `360 → 352` and `75 → 74`, ⛔ of which **−3 is an ADJUDICATION CORRECTION and only −5 is yield** (measured separately by running the adjudicator alone against the unchanged baseline). ⛔⛔ **The first landing silently did nothing, and the toolbox found it:** the new rule was in the generated parser with **no caller** because **a `#` comment at column 0 inside an alternation list terminates the rule and drops every `|` arm below it, with no diagnostic**; the same comment indented is absorbed. Repo-wide audit — 17 grammars, 8 comment-above-arm sites, all indented and verified live, **0** column-0 occurrences — so nothing shipped is damaged; the hazard is routed as `.3.14c`. ⚠️ **Honest bounds:** tolerance is at ITEM positions, never between two tokens of one statement; hosts are module and class bodies only (generate/interface/program/package/checker bodies routed to `.3.14d`); `` `pragma `` is tolerated as a line, not parsed as a structured §22.11 pragma. Full matrix in ledger row `SV-0048`.

### 1.0.177 / Contract 1.0.177 — SV-CORPUS-GRAD.3.12 (`PGEN-SV-CORPUS-GRAD-0029`, 2026-08-08), ledger `SV-0047` (`Released`): **A RECURSION-GUARD REJECTION WAS CACHED IN A RECURSION-BLIND MEMO, REFUSING LEGAL PARSES — `x = $clog2(P)'(P);` (ENGINE, not grammar; cross-profile AND cross-family; schema 19 UNCHANGED)** — this is the first SV release whose fix touches no grammar at all. IEEE 1800-2017 A.8.4 makes a system function call a legal `casting_type` (`casting_type ::= … | constant_primary`, `constant_primary ::= … | constant_function_call`, A.8.2 `constant_function_call ::= function_subroutine_call` ⊇ `system_tf_call`), and `systemverilog.ebnf` spells all three faithfully — yet `initial x = $clog2(P)'(P);` rejected. ⭐ **The whole diagnosis is one discriminating pair:** the SAME parser on the SAME bytes ACCEPTS under `--entry-rule cast` and REJECTS under `--entry-rule primary`. A rule that parses standalone and fails as a branch of its own parent cannot be a grammar defect, because grammar text does not change with the caller. **Root cause:** `cast` is first attempted INSIDE `call_primary`'s own frame, from where `cast → casting_type → constant_primary → constant_function_call → call_primary` re-enters `call_primary` at the same position and the cycle guard correctly rejects. That rejection is a fact about the live PARSE STACK — but the packrat memo is keyed `(rule_id, position)` and nothing else, so the derived failure of `cast` is filed under a stack-blind key and replayed later when `primary` reaches its own `cast` branch with `call_primary` no longer on the stack. A `longest_match` tournament then picks a 9-byte call over the 13-byte cast it never ran. ⛔ **The exact sibling of the store-blindness closed in `1.0.176`-era `MEMO-STORE-SOUNDNESS.2`** — same cache, second context axis. **Fix:** the guard records its blocking frame's stack index, each memoized body opens a floor scope, and a recursion-tainted FAILURE (one caused by a STRICT ANCESTOR frame) is not cached; a block owned by the rule itself or a descendant is re-created by every replay and is not taint. Mirrored at all four memo sites. **What consumers see:** strictly more permissive — `$clog2(RomSize)'(RomSize)`, `$bits(bar)'('1)`, `($bits(x) + $bits(y))'(x * y)` now parse with the unchanged `cast` shape; **corpus pass 9 694 → 9 712 (+18) with 0 pass→fail over 16 336 files**, heals spanning six independent sub-corpora, and **0 transitions** over the 2 459-file `verilog_2005` lane. Because the fix is in the SHARED engine, every generated parser inherits it. **Why schema stays 19:** no emitted kind is added, renamed or removed — the change decides which cached verdict is TRUSTED, never a return annotation. ⛔⛔ **Two earlier designs were implemented and rejected BY MEASUREMENT, and the second is the interesting one.** Subtree-wide taint took a corpus file that passes inside a 60 s budget past 300 s. Frame-scoped-but-refusing-BOTH-polarities regressed 4 files pass→fail, all a cast inside an index in a cyclic expression context — because a cached FAILURE under a guard asserts *"the search stopped here"* while a cached SUCCESS asserts *"this derivation exists, and was found"*, which stays true from any stack. The store changes what the correct answer IS; a guard changes only what the search can REACH, so the shipped gate refuses failures only. ⭐ That success replay is **load-bearing**: on cyclic rules the memo is part of the ACCEPTANCE SEMANTICS — it is how indirect left recursion parses at all — which makes acceptance there evaluation-order-dependent and is tracked as `SV-CORPUS-GRAD.11c`. ⚠️ **Honest bound:** `f(P)'(P)`, a cast whose casting type is a plain user function call, does NOT heal (it rejects on the pre-fix baseline too, and the corpus contains zero instances); closing it needs the validated form, routed to `.11c`. Full matrix in ledger row `SV-0047`.

### 1.0.176 / Contract 1.0.176 — SV-CORPUS-GRAD.3.10 (`PGEN-SV-CORPUS-GRAD-0025`, 2026-07-25), ledger `SV-0046` (`Released`): **SPACED UDP / TIMING-CHECK NUMBER LITERALS RESTORED — `initial q = 1 'b 1;` (IEEE 1800-2017 §5.7.1; CROSS-PROFILE; schema 19 UNCHANGED)** — IEEE 1800-2017 §5.7.1 says a based literal "shall be composed of up to three tokens" (an optional size, an apostrophe followed by a base format character, and the value) and then fixes exactly which interior seams may carry white space: **size↔apostrophe is OPEN** (they are separate lexical tokens; §5.3 applies and nothing carves it out), **apostrophe↔base-char is CLOSED** ("The apostrophe character and the base format character shall *not* be separated by any white space"), and **base_format↔value is OPEN explicitly** ("The unsigned number token shall immediately follow the base format, *optionally preceded by white space*"). Release `1.0.171` (`SV-0041`) gave `integral_number` exactly those two seams — but `init_val:2554` (the UDP initial value, A.5.2) and `scalar_constant:4935` (timing-check conditions, A.7.5.3) enumerate the LRM's **closed** literal set, so they never route through `integral_number`: they carry **12 fused number-literal tokens of their own** (`1'b0`…`1'BX` plus the unsized `'b0`/`'b1`/`'B0`/`'B1`) and never inherited that fix. A fused literal cannot admit interior white space — the layout skipper runs only *before* a terminal, never inside one match — so every §5.7.1-legal spaced spelling failed while the tight one parsed. ⭐ **Worse than a rejection, and this is what the trace showed:** on `1 'b 1` the eight literal branches all fail and then the LRM's **bare `1`** alternative matches, so `init_val` *succeeds* having consumed only the `1` and the parse dies downstream in `sequential_body` — a mis-parse that reports its failure in a different rule entirely. Fixed grammar-only, 12 token bodies and nothing else: `[ \t]*` at exactly the two open seams (`/1[ \t]*'b[ \t]*0\b/` …; the 4 unsized twins have only ONE interior seam, there being no size token). ⭐ Keeping them as their own tokens rather than swapping in `integral_number` is what **keeps the LRM's closed set closed** — the leading `1` stays hard-coded, so `2'b11` still cannot be a UDP `init_val`. **No new rules or tokens — census 1475 UNCHANGED**; and because these tokens carry no profile gate and IEEE 1364-2005 §3.5.1 is identical, the fix is **cross-profile** (`verilog_2005` heals too, like `SV-0041` and unlike the SVA-gated `SV-0044`/`SV-0045`). **What consumers see — two distinct gains.** (1) *Acceptance:* `initial q = 1 'b 1;`, `1 'b1`, `1'b 1`, tab-separated forms, `1 'B 1`, `1 'b x`, `1 'b X` flip **REJECT → ACCEPT** in all three profiles, emitting the **unchanged** kinds `{kind:"1'b0"}` … `{kind:"1'BX"}`. (2) ⚠️ *Shape repair that never manifested as a rejection:* `scalar_timing_check_condition` has a fallback `expression` branch, so a spaced scalar constant in a `specify` timing check **always parsed but silently degraded** — `cond == 1 'b 1` produced the flat `{kind:"expression"}` and now produces the typed `{kind:"eq", lhs, rhs:{kind:"1'b1"}}` (likewise `case_eq`/`ne`/`case_ne`, and unsized `'b 1` → `{kind:"'b1"}`). That is a correction *toward* the shape release `1.0.160` already documents as intended, not a new vocabulary — but a consumer that matched `{kind:"expression"}` for such conditions will now see the compare shape. **Why schema stays 19:** no emitted kind is renamed or removed, proven rather than assumed — an AST dump shows 4 tight/spaced pairs emitting **byte-identical** kinds (only `span_end` differs), and `ast_shape_contract_gate` passes **18/18** over the three locked samples that pin exactly these kinds. **Not over-permissive, measured:** 9 near-miss negatives still REJECT under both profiles — including the §5.7.1-*prohibited* `1' b 1` and `1 ' b 1`, plus `2'b1`, `1'b2`, `1'bz`, `1'b 11` and the digit-less `1'b` (the `SV-0030` guard). ⭐ **This is the first leaf in the series whose value is entirely LRM fidelity rather than corpus yield, and it says so up front:** the keyed corpus population was mechanically derived as **0 rows** *before* measuring (`init_val` is UDP-only; of 150 files carrying a spaced literal only 3 also contain `primitive`/`specify`, and all 3 probe as dying elsewhere), so the corpus is unchanged — rejects-valid **382 → 382 set-identical**, accepts-invalid 21 set-identical, the whole 16,336-row manifest differing by exactly one line (a `pass→timeout` proven to be contention jitter: the edited rules are entered **0** times on that file, versus 10 on the control, and it exits 0 = pass when run solo). `verilog_2005` lane re-run — **0 transitions**, manifest byte-identical. Both cert contracts **untouched** (census byte-inert: the per-rule profile map is identical across all 1475 rules), a posture predicted in writing before the gates ran. ⭐ Landed with an **exhaustive** sweep of all 14 apostrophe-bearing tokens plus the 7 neighbouring closed-literal-set rules: 12 = this defect, and **`unbased_unsized_literal` (`'0`/`'1`/`'x`/`'z`) was adjudicated CORRECT AS FUSED with a named LRM cause** — Annex A footnote 48 explicitly forbids white space after that apostrophe — which is what stopped a mechanical "add seams everywhere" pass from widening acceptance against the standard. ⚠️ A new sibling `SV-CORPUS-GRAD.3.11` is OPEN in the **opposite** direction: Annex A footnote 44 forbids white space between a `time_literal`'s number and its unit, yet `timeunit 10 ns;` wrongly ACCEPTS today. ⭐ **CORRECTED 2026-07-26 (session #207), because both halves of the deferral rationale printed here were wrong once measured.** (a) The tightening does NOT break passing files: measured over all 16,336 corpus files it is **0 pass→fail**. (b) There is no dialect question — the director settled strict-LRM-by-default in session #206, and the ecosystem writes time literals tight universally (0 spaced vs 273 tight). ⭐ And the defect is worse than an over-acceptance: because `s`/`ms`/`us`/`ns`/`ps`/`fs` are ordinary identifier spellings, `time_literal` STEALS the `<number> <white space> <signal>` pair and **REJECTS LEGAL CODE** — `#1 ps[idx] = 1'b1;`, `#2 s = ~s;`, `a ##1 s ##1 b` all fail today, and 2 of those 4 corpus files are tracked `unexplained_rejects_valid` rows. The strict fix is written and measured (382→380 rejects-valid, 0 pass→fail) but is BLOCKED on a missing PGEN primitive: the generator inserts a separator between every pair of sequence elements, so a footnote-44 constraint can only be expressed by collapsing the construct into ONE terminal — which would delete the Annex-A `time_unit` production. Tracked as the new `LEX-ADJACENCY` tree. Full matrix in ledger row `SV-0046`.

### 1.0.175 / Contract 1.0.175 — SV-CORPUS-GRAD.3.9 (`PGEN-SV-CORPUS-GRAD-0024`, 2026-07-25), ledger `SV-0045` (`Released`): **SPACED SVA CYCLE-DELAY ABBREVIATIONS RESTORED — `## [*]` / `## [+]` (IEEE 1800-2017 A.2.10 + §5.3; ⚠️ schema 18 → 19, TWO AST KINDS RENAMED)** — IEEE 1800-2017 A.2.10's third and fourth `cycle_delay_range` alternatives were single **fused literal tokens** (`kw_token_71b8cf7e := trivia "##[*]"`, `kw_token_9768502a := trivia "##[+]"`), each fusing **four** distinct SystemVerilog lexical tokens (`##` + `[` + `*`/`+` + `]`) into one atom. A fused multi-character literal cannot admit interior white space, but §5.3 says white space "shall be ignored except when [it serves] to separate other lexical tokens" — so **every** spaced spelling is legal SystemVerilog, and every one of them rejected while the tight spelling parsed. Both alternatives are now spelled structurally from the existing `kw_token_93ac8946` / `lbrack` / `star` / `plus` / `rbrack` tokens (the exact shape `consecutive_repetition:1377` already uses), and the two fused tokens are retired — census 1477 → **1475**, no new rules or tokens. **⚠️ BREAKING for AST consumers, and the reason the schema moves:** these alternatives previously emitted `{kind:"token_71b8cf7e"}` / `{kind:"token_9768502a"}` and now emit **`{kind:"star"}` / `{kind:"plus"}`**. Unlike `.3.8`'s retired `paren_range` (unreachable in practice, so additive), these kinds were reachable and genuinely emitted — `##[*]` / `##[+]` parsed before this fix — so anything matching on the old opaque hash names must update. The new names align with `consecutive_repetition` and retire two extractor artifacts from the public vocabulary. Verified: 9 spacing variants (`##[*]`, `## [*]`, `##[ *]`, `##[* ]`, `##  [  *  ]`, tab-separated, and the `+` twins) all ACCEPT; 5 near-miss negatives (`##[]`, `##[*`, `##[**]`, `##[+*]`, `##[1:2:3]`) all still REJECT; `verilog_2005` still rejects. Corpus: main pass 9,692 → 9,694 (**+1 attributable**; the other +1 and the fail +1 are proven contention jitter on files containing zero `##`), rejects-valid 383 → **382** (−1, ZERO new), accepts-invalid set byte-identical (21), **0 pass→fail per-FILE**; `verilog_2005` lane re-run and proven byte-inert. ⭐ Landed with an **exhaustive** grammar-wide sweep of all 52 multi-character literal tokens — the bracket/brace face of this defect class is now closed, `attr_open "(*"` / `attr_close "*)"` were adjudicated **correct as fused** with a named LRM cause, and a new defect under a different law (spaced number literals in `init_val` / `scalar_constant`) was measured and routed to leaf `.3.10`.

### 1.0.174 / Contract 1.0.174 — SV-CORPUS-GRAD.3.8 (`PGEN-SV-CORPUS-GRAD-0023`, 2026-07-25), ledger `SV-0044` (`Released`): **SVA BRACKETED CYCLE-DELAY RANGE RESTORED — `##[m:n]` / `##[m:$]` (IEEE 1800-2017 A.2.10; schema 18 UNCHANGED)** — the LRM defines `cycle_delay_range ::= ## constant_primary | ## [ cycle_delay_const_range_expression ] | ##[*] | ##[+]`, and alternative 2's `[ ]` are **literal SystemVerilog brackets**. The LRM's BNF meta-notation, however, uses `[ ]` for *both* literal brackets and optional items — a distinction the published PDF carries typographically and the PDF→text conversion **destroys** — so the LRM-markdown→EBNF extractor read them as grouping and emitted `kw_token_93ac8946 ( cycle_delay_const_range_expression )?` (`grammars/systemverilog.ebnf:1705`): `##` followed by an **optional, bracket-less** range. Consequently no alternative could consume the `[` of a real `##[1:3]` (alt 1 needs a `constant_primary`, alt 2 took the empty option and left `[1:3]` to `seq_unary`, alts 3/4 require the exact literals `##[*]`/`##[+]`), so **every bracketed cycle-delay range was unparseable in all profiles** while unbracketed `##1`/`##2` parsed fine — exactly the corpus split observed (`te3 ##2 te4` passing inside files that died at `##[1:5]`). That the brackets are literal is settled by the LRM's **own normative prose**: "`##[*]` is used as an equivalent representation of `##[0:$]`" and "`##[+]` … of `##[1:$]`" — forms writable only if alternative 2's brackets are literal — plus ~15 source examples (`req ##[4:32] gnt`, `req ##[4:$] gnt`, `w ##1 x ##[2:10] y;`, `(te1 ##[1:5] te2) and (te3 ##2 te4 ##2 te5)`). ⛔ **This is the sixth+ instance of the bug ledger's documented "dropped-delimiter class"** (priors include `SV-0002`'s `stream_concatenation`, the covergroup `trans_range_list` repeats, the `boolean_abbrev` repetition family, and `value_range`) — and `boolean_abbrev` sits in **A.2.10, the same Annex A subclause**, so a prior fix here did not sweep its neighbours. Fixed grammar-only (additive): alternative 2 becomes `kw_token_93ac8946 lbrack cycle_delay_const_range_expression rbrack -> {kind:"range", body:$3}`, using the existing `lbrack`/`rbrack` tokens that `value_range_sv_2017` already uses for the same LRM construct; the misleading `paren_range` kind name retires with the misreading. **ZERO new rules or tokens — census 1477 UNCHANGED.** SV-only: `verilog_2005` is byte-inert (SVA is absent from IEEE 1364-2005 Annex A). **What consumers see:** `a ##[1:3] b`, `req ##[4:$] gnt`, and the spaced `## [2:5]` flip **REJECT → ACCEPT** under `sv_2017`/`sv_2023`, emitting `{kind:"range", body:{kind:"range", lo, hi}}` or `{kind:"dollar_hi", lo}`. **Why schema stays 18:** the bracketed form was 100% unparseable before (additive), and the retired `{kind:"paren_range"}` was unreachable in practice — it required a bare `##`, which no valid SV admits — and is referenced by no shape-contract sample. **Surfaced corpus-first** by the graduation campaign (`##[` was the largest homogeneous stuck-signature cluster: 24 rows across ispras-sv-tests/verilator/Surelog) — the **sixth** defect closed end-to-end: main `sv_2017` external corpus pass 9,669 → 9,692 of 16,336 (+23), rejects-valid 406 → 383 (−23, set-difference-verified 0 new), accepts-invalid 21 unchanged (byte-identical); the 23 flips are **exactly** the keyed rows (flipped-but-not-keyed = 0). NO REGRESSION: **0 pass→fail and 0 pass→timeout per-FILE**; `verilog_2005` lane byte-inert (2,180/279, manifest `cmp`-identical). Proof (seeds 0/7/42): `sv_cert_recognized_union_gate` GREEN **with no re-baseline** (union_witness 1348 unchanged, residual `[]`, canonical UNKNOWN=11 / union 0, still `fully_certified_via_union`); `verilog_2005_conformance_gate` GREEN byte-inert (`1123/330/779/14`, matrix 240/0, orphans 0); `sv_syntax_closure_gate` 1,477 rules / 0 unreachable; `ast_shape_contract_gate` 18/0; `sv_external_corpus_triage_gate` PASS; `systemverilog_parser_book_gate` PASS. ⚠️ **A sibling remains OPEN (`SV-CORPUS-GRAD.3.9`)**: the *spaced* `## [*]` / `## [+]` abbreviations still reject, because alternatives 3/4 are **fused literal tokens** (`trivia "##[*]"`) that cannot admit the interior whitespace SystemVerilog's free-form lexing permits. Unlike this fix, that one renames **currently-produced** AST kinds (`{kind:"token_71b8cf7e"}` → `{kind:"star"}`) and is therefore expected to bump the schema. Full matrix in ledger row `SV-0044`.

### 1.0.173 / Contract 1.0.173 — SV-CORPUS-GRAD.3.7 (`PGEN-SV-CORPUS-GRAD-0022`, 2026-07-24), ledger `SV-0043` (`Released`): **DRIVE/CHARGE STRENGTH KEYWORDS RESTORED — CROSS-PROFILE (IEEE 1800-2017 §28.11 / A.8.6; schema 18 UNCHANGED)** — every SV/Verilog strength keyword is digit-suffixed (`supply0`/`supply1`, `strong0`/`strong1`, `pull0`/`pull1`, `weak0`/`weak1`, `highz0`/`highz1`; A.8.6: `strength0 ::= supply0 | strong0 | pull0 | weak0`, `strength1 ::= supply1 | strong1 | pull1 | weak1`), but the grammar's `strength` (`grammars/systemverilog.ebnf:5267`), `net_type` (`:3549`), and `drive_strength` (`:2045`) rules referenced BARE tokens (`/supply\b/`, `/strong\b/`, `/pull\b/`, `/weak\b/`, `/highz\b/`) which **cannot match a digit-suffixed keyword** (there is no word boundary between a letter and a trailing digit) — grammar-wide grep confirmed no digit-suffixed strength token existed. So the `drive_strength` / `net_strength` / `pulldown_strength` / `pullup_strength` wiring, though fully present, was **entirely DEAD in all profiles**: `wire (strong1, pull0) n = 1;`, `assign (highz0, weak1) n = 1;`, `nor (highz1, strong0) g(…);`, `pullup (supply1) p(a);`, and `supply0` net declarations all rejected. Fixed grammar-only (additive): 10 new digit-suffixed tokens + LRM-faithful **`strength0`/`strength1`** rules + the six A.8.6 opposite-digit `drive_strength` combos + `net_type` `supply0`/`supply1` + `pulldown_strength` (single = `strength0`) / `pullup_strength` (single = `strength1`), minus the 3 now-orphaned bare tokens `kw_supply`/`kw_pull`/`kw_highz` (the bare `kw_strong`/`kw_weak` are RETAINED for the SVA `strong(seq)`/`weak(seq)` operators); census `1469`→`1477`. **UNGATED** because drive strength and supply nets are IEEE 1364-2005 constructs too, so the fix is **cross-profile** (heals `verilog_2005` as well as `sv_2017`/`sv_2023`). **What consumers see:** drive-strength continuous assigns, net declarations, gate/UDP instantiations, and pullup/pulldown gates flip **REJECT → ACCEPT** across all profiles, emitting `drive_strength: {kind:"pair_strength"|"strength_highz"|"highz_strength", …}` with `strength0`/`strength1` operands, and `net_type: {kind:"supply0"|"supply1"}`; **same-digit pairs like `(strong1, strong1)` correctly REJECT** (A.8.6 requires opposite digits — the strict `strength0`/`strength1` modelling, not a single permissive `strength`, is what preserves that). **Why schema stays 18:** the strength machinery was 100% unparseable before (additive; no witnessed wire-shape change), and no shape-contract sample exercised `strength`/`drive_strength`/`net_type`-supply. **Surfaced corpus-first** by the graduation campaign (drive/charge strength = 21 rows in the v3 family map) — the **fifth** defect closed end-to-end and the **largest cross-profile heal to date**: main `sv_2017` external corpus pass 9,591 → 9,669 of 16,336 (+78), adjudication rejects-valid 426 → 406 (−20, `comm`-verified 0 new), accepts-invalid 21 unchanged (byte-identical); **`verilog_2005` lane pass 2,126 → 2,180 (+54), rejects-valid 116 → 62 (−54 = iverilog 50 / ispras 4, 0 new), accepts-invalid 14 unchanged**. NO REGRESSION: **0 pass→fail per-FILE in both lanes** (the strict accept-set is a subset of the earlier permissive pass — a same-digit `_bad` negative, `t_strength_strong1_strong1_bad.v`, is what drove the tightening from permissive to spec-faithful). Proof (all seeds 0/7/42): `sv_cert_recognized_union_gate` GREEN re-baselined (+8 positively witnessed: total 1346→1354, canonical witness 1329→1337, union witness 1340→1348; UNKNOWN 11/0, residual `[]`, still `fully_certified_via_union`); `verilog_2005_conformance_gate` GREEN re-baselined (cross-profile — cert `1115/328/773/14` → `1123/330/779/14`, +8 positively accounted, UNKNOWN 14 identical, matrix 240/0, orphans 0); `sv_stimuli_quality_gate` PASS (`closed_loop_replay_targets_total` 125→126, +1: the new `strength0`/`strength1` + `drive_strength` branches are a new closed-loop generation-coverage target — feeds `SV-REPLAY-DEBT`, not a regression); `ast_shape_contract_gate` 18/0; `sv_external_corpus_triage_gate` 14/14; `sv_syntax_closure_gate` (1,477 rules, 0 unreachable). Full matrix in ledger row `SV-0043` and the contract's § "Contract Identity" (schema 18).

### 1.0.172 / Contract 1.0.172 — SV-CORPUS-GRAD.3.6 (`PGEN-SV-CORPUS-GRAD-0021`, 2026-07-24), ledger `SV-0042` (`Released`): **`unique0` IF/CASE QUALIFIER RESTORED (IEEE 1800-2017 §12.4.2 / §12.5.3, A.6.6; schema 18 UNCHANGED)** — the `unique_priority` rule (`grammars/systemverilog.ebnf:5861`, already `@profiles ["sv_2017","sv_2023"]`) had only the `unique` and `priority` branches, **missing the third IEEE 1800-2017 A.6.6 alternative** (`unique_priority ::= unique | unique0 | priority`), and **no `unique0` token existed** (the qualifier tokens were only `kw_unique := /unique\b/` and `kw_priority := /priority\b/`; `/unique\b/` cannot match `unique0` because there is no word boundary between `e` and `0`), so `unique0 if (…)` and `unique0 case (…)` — the SV-2009 violation-checking qualifier — wrongly **REJECTED** (furthest at the `if`/`case`) while the sibling `unique if`, `priority if`, and `unique case` all parsed. Fixed grammar-only: one new profile-gated token `kw_unique0 := trivia /unique0\b/` (`@profiles ["sv_2017","sv_2023"]` — `unique0` is SV-only, absent from IEEE 1364-2005) and one branch `| kw_unique0 -> {kind:"unique0"}` added to `unique_priority` (census `1468`→`1469`). **What consumers see:** `unique0 if`/`unique0 case`/`unique0 casez`/`unique0 casex` flip **REJECT → ACCEPT** under `sv_2017`/`sv_2023`, emitting `unique_priority: {kind:"unique0"}` (alongside the existing `{kind:"unique"}`/`{kind:"priority"}`); `verilog_2005` still rejects (SV-only). **Why schema stays 18:** the `unique_priority` node shape is unchanged — a new enum value on an existing `{kind:<string>}` field, and `unique0 if`/`case` were 100% unparseable before (no lock or wire-format broken). **Surfaced corpus-first** by the graduation campaign (`SV-CORPUS-GRAD.3.2`: `unique0` is the #11 rejects-valid family) — the FOURTH defect the campaign closed end-to-end: main `sv_2017` external corpus pass 9,585 → 9,591 of 16,336 (+6), adjudication rejects-valid 432 → 426 (−6 = the whole family, `comm`-verified exactly the 6 keyed rows, ZERO new), accepts-invalid 21 unchanged (byte-identical set); NO REGRESSION: **0 pass→fail per-FILE**; `verilog_2005` lane BYTE-INERT (manifest byte-identical, rejects-valid 116 unchanged). Proof (all seeds 0/7/42): `sv_cert_recognized_union_gate` GREEN on an evidence-grounded re-baseline (the new token POSITIVELY WITNESSED: total 1345→1346, canonical witness 1328→1329, union witness 1339→1340; proof 6, UNKNOWN 11/0, residual `[]` all unchanged, still `fully_certified_via_union`); `verilog_2005_conformance_gate` GREEN byte-inert (cert `1115/328/773/14`, matrix 240/0, orphans 0); `sv_stimuli_quality_gate` PASS (`closed_loop_replay_targets_total` 125 UNCHANGED (no new closed-loop generation target — the new qualifier is immediately witnessed)); `ast_shape_contract_gate` 18/0; `sv_external_corpus_triage_gate` 14/14; `--lint-grammar` clean (1,469 rules, orphans 0). Full matrix in ledger row `SV-0042` and the contract's § "Contract Identity" (schema 18).

### 1.0.171 / Contract 1.0.171 — SV-CORPUS-GRAD.3.5 (`PGEN-SV-CORPUS-GRAD-0020`, 2026-07-23), ledger `SV-0041` (`Released`): **SPACED-BASED NUMBER LITERALS RESTORED — CROSS-PROFILE (IEEE 1800-2017 §5.7.1 / IEEE 1364-2005 §3.5.1; schema 18 UNCHANGED)** — the `integral_number` terminal (`grammars/systemverilog.ebnf:439`) was the single regex `([0-9][0-9_]*)?'[sS]?[dDhHoObB][0-9a-fA-FxXzZ?_]+`, which could not span the LRM-legal white space **between the size and the `'`** (`32 'h5`) or **between the base format and the value** (`32'h 5`) — the generated layout skipper consumes white space *before* a terminal but not *inside* one match — so every spaced based literal wrongly **REJECTED**. Because `integral_number` is not profile-split, the gap hit **all** profiles. LRM (verbatim, IEEE 1800-2017 §5.7.1 / IEEE 1364-2005 §3.5.1): *"The apostrophe character and the base format character shall not be separated by any white space. … The unsigned number token shall immediately follow the base format, optionally preceded by white space."* Fixed grammar-only (one terminal): `integral_number := ([0-9][0-9_]*[ \t]*)?'[sS]?[dDhHoObB][ \t]*[0-9a-fA-FxXzZ?_]+` — horizontal-white-space `[ \t]*` at exactly the two LRM-legal seams; the `'[sS]?[dDhHoObB]` base specifier stays **contiguous** (the LRM forbids interior white space) and the value charset is **unchanged**, so the LRM-illegal `'d -6` (minus between base and value) and interior white space inside the value both still **reject**. No new rule/token (census 1345 unchanged). **What consumers see:** based literals with white space at the two legal seams (`32 'h5`, `32'h 5`, `32 'h 12ab_f001`, unsized `'h 837FF`, `2'b ?1`, `-8'd 6`) flip **REJECT → ACCEPT** under **every** profile (`sv_2017`/`sv_2023`/`verilog_2005`); the `body` of `integral_number` is the whole matched string (now optionally including the interior spaces). **Why schema stays 18:** `integral_number` is a terminal whose `body:$1` is the whole match — an ADDITIVE lexical change, no structural shape change (spaced literals were 100% unparseable before, so no lock or wire-format is broken). **Surfaced corpus-first** by the graduation campaign (`SV-CORPUS-GRAD.3.2`: number-literal spaced-based is the #6 rejects-valid family at 22 rows; the chapter-5 lexical cluster repeatedly flagged since `.2`) — the **fourth** defect the campaign closed end-to-end and the **first cross-profile** one: main `sv_2017` external corpus pass 9,459 → 9,585 of 16,336 (+126: opentitan 83 / iverilog 14 / sv-tests 13 / ispras 8 / Surelog 6 / verilator 2; timeout 10 → 9), adjudication rejects-valid 454 → 432 (−22 = the whole #6 family, `comm`-verified exactly the 22 keyed rows, zero new), accepts-invalid 21 unchanged (byte-identical set); **cross-profile bonus — `verilog_2005` lane pass 2,107 → 2,126 (+19), rejects-valid 135 → 116 (−19, 0 new), accepts-invalid 14 unchanged**. NO REGRESSION: **0 pass→fail per-FILE in both lanes** (strictly additive). Proof (all seeds 0/7/42): `sv_cert_recognized_union_gate` GREEN byte-inert (no rule/token ⇒ census 1345 unchanged, canonical UNKNOWN=11, union residual `[]`, `fully_certified_via_union`); `verilog_2005_conformance_gate` GREEN (curated matrix 240/0 has no spaced-literal case; cert `1115/328/773/14`); `sv_stimuli_quality_gate` PASS (`closed_loop_replay_targets_total` 126 → 125, −1: one `sv_2017` closed-loop replay target newly witnessed — a replay-debt gap closed feeding `SV-REPLAY-DEBT`; a decrease is an improvement and no target can regress under a strictly-more-permissive change); `ast_shape_contract_gate` 18/0; `sv_external_corpus_triage_gate` 14/14. Full matrix in ledger row `SV-0041` and the contract's § "Contract Identity" (schema 18).

### 1.0.170 / Contract 1.0.170 — SV-CORPUS-GRAD.3.4 (`PGEN-SV-CORPUS-GRAD-0019`, 2026-07-23), ledger `SV-0040` (`Released`): **MODPORT SHARED-DIRECTION PORT LIST RESTORED + a coupled `ref` reserved-word completion (IEEE 1800-2017 A.2.9; schema 17 → 18)** — `modport_simple_ports_declaration` (`grammars/systemverilog.ebnf:526`) was **missing the LRM A.2.9 `{ , modport_simple_port }` repetition**, so a single `port_direction` admitted only ONE port and `modport master(input a, b, output c, d)` **REJECTED** at the shared-direction port `b` (`furthest_position=58`) while `modport master(input a, output c)` accepted. Fixed grammar-only in two coupled parts: (1) add the repetition — `modport_simple_ports_declaration := port_direction modport_simple_port ( comma modport_simple_port )*` with the `[$first, $rep::2*]` extraction-spread (`{direction, ports: [$2, $3::2*]}`); (2) **a coupled `ref` reserved-word completion surfaced BY the measurement** — the modport-only regen first regressed two passing verilator files (`modport phy(input addr, ref data)`) because the new greedy `( comma modport_simple_port )*` gobbled `ref` as a `port_identifier`; `ref` is a genuine SV `port_direction` (`port_direction_sv_only := kw_ref`) and an IEEE 1800 Table B.1 reserved keyword, but was **erroneously omitted from `reserved_non_keyword_identifier_sv:402`** (it fell through the `SV-0036` net because that fix added only the `verilog_2005`-reserved delta, and `ref` is SV-only-reserved). Adding `ref` to the SV list makes the repetition sound (the engine's `*` does not backtrack across the rule boundary, so `ref` must fail `port_identifier`); `verilog_2005` keeps its own list (`_v2005:405`, no `ref`), staying byte-inert. Census unchanged 1468 (no new rule/token). **What consumers see:** comma-shared-direction modports (`modport m(input a, b, output c, d)`, `modport m(input clk, rst)`, `modport m(input a, ref b, output c)`) flip **REJECT → ACCEPT** under `sv_2017`/`sv_2023`, and `modport_simple_ports_declaration` now emits a `ports` **array** (`{direction, ports:[…]}`) where it emitted a single `port` field; `verilog_2005` still rejects modport (SV-only). A consumer reading `modport_simple_ports_declaration` must adopt the `ports` array. **Why schema bumps 17 → 18:** the emitted `port: $2` → `ports: [$2, $3::2*]` is a user-visible shape change. **Surfaced corpus-first** by the graduation campaign (`SV-CORPUS-GRAD.3.2`: interface/modport is the #3 rejects-valid family at 50 rows) — the third defect the campaign closed end-to-end: full external corpus pass 9,433 → 9,459 of 16,336 (+26 net; 29 gains − 3 `deferred:v2005_profile_lane` `reg ref;` files that correctly reject under `sv_2017` and still accept under `verilog_2005`), adjudication baseline rejects-valid 481 → 454 (−27), accepts-invalid 21 unchanged (identical set), 0 new rejects-valid. Proof: `verilog_2005_conformance_gate` GREEN byte-inert (lint orphans 0, matrix 240/0, cert deterministic seeds 0/7/42, v2005 manifest byte-identical); `systemverilog_parser_book_gate` GREEN; `sv_stimuli_quality_gate` + `sv_cert_recognized_union_gate` GREEN. Full matrix in ledger row `SV-0040` and the contract's § "Contract Identity" (schema 18). Same reserved-keyword class as `SV-0035`/`SV-0036`.

### 1.0.169 / Contract 1.0.169 — SV-CORPUS-GRAD.3.3 (`PGEN-SV-CORPUS-GRAD-0018`, 2026-07-23), ledger `SV-0039` (`Released`): **SVA IMPLICATION OPERATORS `|->`/`|=>` RESTORED (IEEE 1800-2017 A.2.10; schema 16 → 17)** — the two core SVA sequence-implication operators, overlapped `|->` and non-overlapped `|=>`, were **entirely absent** from the grammar (no token, no `prop_primary` branch, never present per `git log -S`), so every `sequence_expr |-> property_expr` / `sequence_expr |=> property_expr` concurrent assertion — the most fundamental SVA construct — wrongly **REJECTED** at the operator, while an implication-free `assert property (@(posedge clk) p3)` parsed. Root cause of the omission: the LRM-markdown→EBNF extractor split the `|`-prefixed operators on `|` (the EBNF alternation metachar), leaving the mangled `implies` (`->`) / `or_assign` (`|=`) remnants in `prop_primary`; orthogonal to H.12.5.8 (whose matrix tested only the *word* operators until/or/and/intersect/within/##). Fixed grammar-only: two new tokens `overlapped_implication := "|->"` / `non_overlapped_implication := "|=>"` (profile-gated `["sv_2017","sv_2023"]` like `nonblocking_implies`/`SV-0023` — SVA is SV-only) and the two A.2.10 branches `sequence_expr <op> property_expr` mirrored into `prop_primary_sv_2017`/`_sv_2023` (census `1466`→`1468`). **What consumers see:** implication assertions (signal, named-property, or delayed-sequence RHS) flip **REJECT → ACCEPT** under `sv_2017`/`sv_2023`, emitting `{kind:"overlapped_implication", lhs, rhs}` / `{kind:"non_overlapped_implication", lhs, rhs}` in `prop_primary`; `verilog_2005` still rejects (byte-inert). **Why schema bumps 16 → 17:** two new `prop_primary` branches with new return annotations are added — consumers exhaustively matching `prop_primary` kinds must add arms for the two new kinds. **Surfaced corpus-first** by the graduation campaign (`SV-CORPUS-GRAD.3.2`: SVA implication/property is the #1 rejects-valid family at 101 rows) — the SECOND defect (after `SV-0038`) the campaign caught and closed end-to-end: full external corpus pass 9,361 → 9,433 of 16,336 (+72, timeout 9 → 9 unchanged, zero per-suite regressions — all SVA-assertion files), adjudication baseline rejects-valid 543 → 481, accepts-invalid 21 unchanged. Proof (all seeds 0/7/42): `sv_cert_recognized_union_gate` GREEN on an evidence-grounded re-baseline (2 new tokens WITNESSED: total 1343→1345, union witness 1337→1339, UNKNOWN 11/0 residual `[]`, still `fully_certified_via_union`); `verilog_2005_conformance_gate` GREEN byte-inert (cert `1115/328/773/14`, matrix 240/0, orphans 0); `sv_stimuli_quality_gate` PASS (`closed_loop_replay_targets_total` 120 → 126 — new SVA-implication generation targets feeding `SV-REPLAY-DEBT`); ast_shape 18/0; external-corpus triage 14/14. Full matrix in ledger row `SV-0039` and the contract's § "Release 1.0.169".

### 1.0.168 / Contract 1.0.168 — SV-CORPUS-GRAD.3.1 (`PGEN-SV-CORPUS-GRAD-0006`, 2026-07-22), ledger `SV-0038` (`Released`): **`interface class` RESTORED UNDER `sv_2017` (IEEE 1800-2017 §8.26; schema 16 unchanged)** — the three `interface_class_declaration` consumer wirings existed only in the `*_sv_2023` variant rules, so `interface class Foo; … endclass` — a 1800-**2017** construct (§8.26; the production is present in the 2017 LRM's section-8 source) — had no derivation path under `sv_2017` and rejected at the `class` keyword, while `sv_2023` parsed it. Fixed grammar-only: the branch mirrored into `anonymous_program_item_sv_2017`, `class_item_sv_2017`, `package_or_generate_item_declaration_sv_2017` (zero new rules, census `1466`). **What consumers see:** `interface class` declarations (plain, parameterized, `extends`-chained, `pure virtual` method prototypes) flip **REJECT → ACCEPT under `sv_2017`** at compilation-unit scope, inside classes, and in anonymous programs; `sv_2023` byte-identical; `verilog_2005` still rejects (SV-only). **Why schema stays 16:** the carrier and `{kind:"interface_class"}` item shapes already existed via the `sv_2023` wiring — only previously-rejected input gains parses. **Surfaced corpus-first** by the external-corpus graduation campaign (`SV-CORPUS-GRAD.2`/`.3.0`: adjudication manifest + stuck-point clustering; the F1 cluster ≈47 rejects-valid rows) — the first defect that pipeline caught and closed end-to-end: full external corpus pass 3,049 → 3,091 of 5,128 (+42, zero per-suite regressions), adjudication baseline 321 → 279 unexplained. Proof (all seeds 0/7/42): `sv_stimuli_quality_gate` PASS (replay-target total 120 unchanged); `sv_cert_recognized_union_gate` GREEN on an evidence-grounded re-baseline — 4 unreachability proofs CONVERT to genuine witnesses (canonical `1343/6/1326/11`, union witness `1337`, UNKNOWN unchanged `11`/`0`, residual `[]`, spf=0; count-conserving ±4); `verilog_2005_conformance_gate` GREEN byte-inert (orphans 0, matrix 240/0, cert `1115/328/773/14` byte-identical); ast_shape PASS; external-corpus triage PASS; `--lint-grammar` clean (1,466 rules). Full matrix in ledger row `SV-0038` and the contract's § "Release 1.0.168".

### 1.0.167 / Contract 1.0.167 — SV-OVER-REJECTION-FIDELITY.3 (`PGEN-SV-OVER-REJECTION-FIDELITY-0003`, 2026-07-05), ledger `SV-0023` (`Released`): **EVENT-TRIGGER `->>`/DELAY/ARRAY LRM COMPLEX RESTORED (schema 15 → 16)** — the IEEE 1800 §15.5 / A.6.5 event-trigger family was mis-encoded four ways: (1) the NONBLOCKING `->>` operator was ABSENT from the grammar, so `->> e;` and `->> #5 e;` wrongly **REJECTED**; (2) the optional `[ delay_or_event_control ]` was mis-attached to a second `->` branch, so `-> #5 e;` wrongly **ACCEPTED** (per A.6.5 the control belongs to `->>` only); (3) the consumer-visible `kind` labels were SWAPPED (`->` is BLOCKING per §15.5.3 but was labeled `non_blocking`; the control branch was labeled `blocking`); (4) the `sv_2017` rule was shared with `verilog_2005` with no bracket-select, so the IEEE 1364-2005 array trigger `-> e[0];` wrongly **REJECTED** under `verilog_2005`. Fixed grammar-only: added `nonblocking_implies := trivia "->>"` (gated `["sv_2017","sv_2023"]`, modeled on `arithmetic_shift_left`/`iff_arrow`); split the block into three profile-faithful rules — `event_trigger_sv_2017` (2017 A.6.5: `-> hei ;` / `->> [ctrl] hei ;`), `event_trigger_sv_2023` (2023 Annex A: each + trailing `nonrange_select`), `event_trigger_verilog_2005` (1364-2005: `-> hei { [expr] } ;`, trailing `bit_select`, no `->>`); moved the optional control onto the `->>` branch; corrected the labels; removed `event_trigger_control_sv_only`. Census `1465`→`1466`. **What consumers see:** `->> e;` / `->> #5 e;` flip **REJECT → ACCEPT** under `sv_2017`/`sv_2023` (REJECT under `verilog_2005`), emitting `{kind:"non_blocking", control, name[, select]}`; the plain `-> e;` still ACCEPTS everywhere but its label is CORRECTED `non_blocking` → **`blocking`** (`{kind:"blocking", name[, select]}`); `-> #5 e;` flips **ACCEPT → REJECT** under every profile; `-> e[0];` flips **REJECT → ACCEPT** under `verilog_2005` (and `sv_2023`; still REJECT under `sv_2017` — the strict 2017 BNF has no select). `event_trigger` now dispatches three profile bodies (`sv_2017`/`sv_2023`/`verilog_2005`). **Migration:** consumers matching `kind:"non_blocking"` for a plain `-> e;` must switch to `kind:"blocking"`; consumers matching `kind:"blocking"` + a `control` for `-> #N e;` must switch to the `->> #N e;` `kind:"non_blocking"` shape. **Why schema bumps 15 → 16:** the event-trigger return annotations are restructured and branches are added/removed (a user-visible shape change). Proof (all seeds 0/7/42): `sv_cert_recognized_union_gate` GREEN (canonical `1343/2/1321/20` byte-identical, union UNKNOWN=1 residual `context_member_method_call`), `verilog_2005_conformance_gate` GREEN (orphans 0, matrix `240/0` with 3 new event-trigger locks + the `event_trigger_delay.sv` re-adjudication, cert `1117/4/773/340` byte-identical), ast_shape 18/18, external corpus 14/14, the six fully-certified grammars byte-identical, clippy source strict-clean (grammar-only). Full matrix in ledger row `SV-0023` and the contract's § "Release 1.0.167". **This closes the `SV-OVER-REJECTION-FIDELITY` tree** (all three all-profile over-REJECTION defects `SV-0021`/`SV-0022`/`SV-0023` fixed).

### 1.0.166 / Contract 1.0.166 — SV-OVER-REJECTION-FIDELITY.2 (`PGEN-SV-OVER-REJECTION-FIDELITY-0002`, 2026-07-04), ledger `SV-0022` (`Released`): **SPEC-VALID SINGLE-`;` `bind` NO LONGER OVER-REJECTS (schema 15 unchanged)** — a single-semicolon `bind` directive (`bind m my_checker c1 ();` at top level, or `module m; bind m my_checker c1 (); endmodule` at module scope) previously **REJECTED** under `sv_2017`/`sv_2023` (the parser rejected known-valid IEEE 1800 input) and now correctly **ACCEPTS**. Root cause: `bind_directive` (`grammars/systemverilog.ebnf:668`/`:670`) required a trailing `semi` AFTER `bind_instantiation`, but every `bind_instantiation` alternative (`module_instantiation`/`program_instantiation`/`interface_instantiation`/`checker_instantiation`) already consumes its own `;` — the IEEE 1800-2017 §23 published BNF's `bind … bind_instantiation ;` is a REDUNDANT double-`;` (because `module_instantiation ::= … ;`) that every commercial simulator (VCS/Questa/Xcelium/Verilator/slang) resolves as a single `;`. Fixed grammar-only by removing the trailing `semi` from both `bind_directive` alternatives; ZERO new rules (census `1465`). **What consumers see:** the single-`;` `bind` flips **REJECT → ACCEPT** under `sv_2017`/`sv_2023`; it stays REJECT under `verilog_2005` (`bind` is SystemVerilog-only). The double-`;` form (`c1 ();;`) still accepts under the SV profiles — now as a legitimate `bind … ();` + trailing empty package-or-generate item (cf. `SV-0028`), not the old defect. No shape change and no previously-accepting input changes. **Why schema stays 15:** the removed `semi` was the last, *uncaptured* position of `bind_directive`, so the return-annotation `$`-refs (`instantiation: $4`/`$3`) and the `{kind:"scoped"|"single", …}` shape are byte-identical. Proof (all seeds 0/7/42): `sv_cert_recognized_union_gate` GREEN (canonical `1343/2/1321/20` byte-identical, union UNKNOWN=1 residual `context_member_method_call`), `verilog_2005_conformance_gate` GREEN (orphans 0, matrix `231/0` with `reject/bind_dir.sv` flipped to SV-accept, cert `1117/4/773/340` byte-identical), ast_shape 18/18, external corpus 14/14, the six fully-certified grammars byte-identical, clippy source strict-clean (grammar-only). Full matrix in ledger row `SV-0022` and the contract's § "Release 1.0.166".

### 1.0.165 / Contract 1.0.165 — SV-LRM-SHAPE-FIDELITY.1 (`PGEN-SV-LRM-SHAPE-FIDELITY-0001`, 2026-07-04), ledger `SV-0024` (`Released`): **UN-BRACED MULTI-IDENTIFIER PORT LISTS NO LONGER OVER-ACCEPT; BRACED CONCATENATION PORTS NOW ACCEPTED (schema 15 unchanged)** — a whitespace-separated multi-identifier port with no comma and no concatenation braces (`module m (a b);`) previously accepted under EVERY profile. IEEE `port_expression ::= port_reference | { port_reference {, port_reference} }` — the multi-reference form requires LITERAL concatenation braces. Fixed grammar-only: `port_expression`'s second alternative (`grammars/systemverilog.ebnf:4092`) changed from the un-braced `( port_reference ( comma port_reference )* )*` to `lbrace port_reference ( comma port_reference )* rbrace -> {kind:"list", body:[$2, $3::2*]}` (the `assignment_pattern` concatenation-list idiom); ZERO new rules (census `1465`). **What consumers see:** `module m (a b);` flips **ACCEPT → REJECT** under `verilog_2005` (its only reading is the non-ANSI `port_expression`); it still ACCEPTS under `sv_2017`/`sv_2023` as a legitimate ANSI interface port (`a`=interface type, `b`=port name — valid IEEE 1800 syntax, an elaboration-time check); and the braced concatenation port `module m ({a, b});` flips **REJECT → ACCEPT** under EVERY profile (the LRM's valid multi-reference form, previously unsupported because the list alternative was written un-braced), emitting `port_expression {kind:"list", body:[{name,select}, …]}`. Single / comma-separated / named `.p(a)` / ANSI declaration ports are unchanged. **Why schema stays 15:** the `{kind:"list"}` body is still typed-JSON — the braced-concat shape is newly *reachable*, not a new envelope shape. Proof (all seeds 0/7/42): `sv_cert_recognized_union_gate` GREEN (canonical `1343/2/1321/20` byte-identical, union UNKNOWN=1 residual `context_member_method_call`), `verilog_2005_conformance_gate` GREEN (orphans 0, matrix 225/0 with 2 new locks, cert `1117/4/773/340` byte-identical), ast_shape 18/18, external corpus 14/14, the six fully-certified grammars byte-identical, clippy source strict-clean (grammar-only). Full matrix in ledger row `SV-0024` and the contract's § "Release 1.0.165".

#### 1.0.165 (same release — no version bump) — SV-LRM-SHAPE-FIDELITY.2 (`PGEN-SV-LRM-SHAPE-FIDELITY-0002`, 2026-07-04), ledger `SV-0028` (`Released`): **STRAY TOP-LEVEL `;` NO LONGER OVER-ACCEPTS UNDER `verilog_2005` (schema 15 unchanged)** — a bare top-level `;` with no declaration previously accepted under `verilog_2005`. IEEE 1364-2005 A.1.2 (`source_text ::= { description }`, `description ::= module_declaration | udp_declaration | config_declaration`) admits no null/empty top-level item, so a stray `;` is invalid Verilog-2005; it accepted only via `source_text_item`'s LRM-less direct `| semi -> {kind:"semi"}` alternative (`grammars/systemverilog.ebnf:231`), now removed (ZERO rule census change — a branch, not a rule; census `1465`). **Tools-first re-adjudication** (AST dumps + IEEE LRM markdown) corrected an earlier "all-profile" framing to **`verilog_2005`-only**: under `sv_2017`/`sv_2023` a stray `;` is the **legitimate LRM empty package-or-generate item** (`description ::= … | { attribute_instance } package_item`, `package_or_generate_item_declaration ::= … | ;` — IEEE 1800-2017 §26 / 2023 Annex A) and correctly stays ACCEPT (the PEG-first `description` path shadows `:231` under the SV profiles, so removing it changes nothing there). **What consumers see:** `;` / `;;` / `module m; endmodule\n;` flip **ACCEPT → REJECT** under `verilog_2005`; unchanged ACCEPT under `sv_2017`/`sv_2023` with the empty-item AST byte-shape-identical. **No version bump** (SV profiles byte-invariant — the `SV-0026`/`SV-0031`..`SV-0034` precedent). Proof (all seeds 0/7/42): `sv_cert_recognized_union_gate` GREEN (canonical `1343/2/1321/20` byte-identical, union UNKNOWN=1), `verilog_2005_conformance_gate` GREEN (orphans 0, matrix 228/0 with 1 new lock `reject/stray_top_level_semi.sv`, cert `1117/4/773/340` byte-identical), ast_shape 18/18, external corpus 14/14, the six fully-certified grammars byte-identical, clippy source strict-clean (grammar-only). Full detail in ledger row `SV-0028`. **This closes the `SV-LRM-SHAPE-FIDELITY` tree** (both all-profile over-acceptance leaks `SV-0024` + `SV-0028` fixed).

### 1.0.164 / Contract 1.0.164 — SV-KEYWORD-PRIMARY-FIDELITY.3.2 (`PGEN-SV-KEYWORD-PRIMARY-FIDELITY-0005`, 2026-07-04), ledger `SV-0036` (`Released`): **NET/GATE/STRUCTURAL RESERVED KEYWORDS NO LONGER PARSE AS BARE EXPRESSION PRIMARIES (schema 15 unchanged)** — 91 keywords that SystemVerilog reserves but that were absent from `reserved_non_keyword_identifier_sv` (net types `wire`/`wand`/`tri`/`supply0`/…, gate primitives `and`/`or`/`nand`/`buf`/`nmos`/…, structural `always`/`assign`/`initial`/`posedge`/`genvar`/`specparam`/…) could match as a bare `primary` (`assign w = wire;`, `initial x = and;`) under `sv_2017`/`sv_2023`. IEEE 1800 Annex B reserves these; they are never an operand. Fixed grammar-only: the `_v2005 \ _sv` delta (91 words) was appended to `reserved_non_keyword_identifier_sv` (`grammars/systemverilog.ebnf:386`, `_sv` 81→172 words, `_sv` ⊇ `_v2005`). **What consumers see:** `assign w = <reserved-kw>;` and operand forms flip **ACCEPT → REJECT** under `sv_2017`/`sv_2023`; keyword-**prefixed** identifiers (`wire_en`/`input_data`/`and_gate`) still parse (the `\b` word boundary). `verilog_2005` unchanged (complete Annex B list). **Why schema stays 15:** shape-preserving (`{body:X}` unchanged — only reserved keywords change acceptance); ZERO new rules (census `1465` unchanged). A co-landed **generator prerequisite** (`rust/src/ast_pipeline/stimuli_generator.rs`) extended the RTL-FE-CLOSURE.6 keyword-exclusion guard from the fixed-literal `!kw_A…!kw_Z` shape to SystemVerilog's `!reserved_non_keyword_identifier identifier` (regex whole-word-list) shape so the stimuli generator stops out-generating its own parser (canonical `sample_parse_failures` 1→0 at seed 0; six fully-certified grammars byte-identical) — a generation-side fix with no parser-output-shape change. Proof: canonical cert `1343/2/1321/20` byte-identical, `sv_cert_recognized_union_gate` GREEN (union UNKNOWN=1, residual `context_member_method_call`), v2005 conformance orphans 0 / matrix 219/0 / cert `1117/4/773/340`, ast_shape 18/18, external corpus 14/14, clippy source strict-clean (all seeds 0/7/42). Full matrix in ledger row `SV-0036` and the contract's § "Release 1.0.164". Closes the all-profile reserved-keyword-as-primary family (`SV-0035`+`SV-0036`).

### 1.0.163 / Contract 1.0.163 — SV-KEYWORD-PRIMARY-FIDELITY.3.1 (`PGEN-SV-KEYWORD-PRIMARY-FIDELITY-0004`, 2026-07-04), ledger `SV-0037` (`Released`): **IMPLICIT-TYPE ANSI PORTS PARSE CORRECTLY (schema 15 unchanged)** — an implicit-type ANSI port such as `module m(input a);` (and `module m(input [7:0] a);`) was mis-parsed as a non-ANSI port list (`{kind:"nonansi"}`) instead of the correct ANSI port (`{kind:"ansi"}`, direction=`input`, implicit type, name=`a`). Root cause: `net_port_type_sv_2017`/`_sv_2023` alt 1 used the ungated `net_type_identifier`/`nettype_identifier`, and under the grammar's default `branch_policy=longest_match` that bare identifier out-consumed alt 0's empty implicit-type match and ate the port name. Fix: both alt-1 carriers routed to the store-gated `checked_nettype_identifier` (`grammars/systemverilog.ebnf:3537`), so only a DECLARED nettype matches there. `module m(input a);` now realizes `{kind:"ansi"}` under all profiles; declared-nettype ports are byte-identical. Consumers keying off the module `kind` for implicit-type ANSI ports should expect the corrected `ansi` shape. No-regression: canonical cert `1343/2/1321/20` byte-identical, ast_shape 18/18, union canonical=20/union=1, v2005 conformance orphans 0 / matrix 219/0 / cert `1117/4/773/340`, external corpus 14/14 (all seeds 0/7/42). Unblocks `SV-0036`/`.3.2`.

### 1.0.162 / Contract 1.0.162 — SV-KEYWORD-PRIMARY-FIDELITY.2 (`PGEN-SV-KEYWORD-PRIMARY-FIDELITY-0002`, 2026-07-04), ledger `SV-0035` (`Released`): **RESERVED TYPE KEYWORDS NO LONGER PARSE AS BARE EXPRESSION PRIMARIES (schema 15 unchanged)** — a reserved *type* keyword (`integer`/`real`/`time`/`reg`/`logic`/`int`/`bit`/`byte`/`shortint`/`longint`/`shortreal`/`string`/`chandle`/`event`/…) could match as a bare `primary` (`assign w = integer;`, `initial x = integer;`, `localparam p = integer + 1;`) under every profile at every release `1.0.0`–`1.0.161`. IEEE 1364-2005 / IEEE 1800 Annex B reserve these words as data-type keywords only — never a `primary`/operand. Fixed grammar-only: the three expression-primary bare-`identifier` carriers — `specparam_identifier` (`grammars/systemverilog.ebnf:5083`) + `genvar_identifier` (`:2375`) in `constant_primary_sv_2017`/`_sv_2023`, and the trailing name of `hierarchical_identifier` (`:2412`) in `primary_sv_2017` — now route through the keyword-excluding `non_keyword_identifier` (`:359`). **What consumers see:** `assign w = <reserved-type-kw>;` and operand forms flip **ACCEPT → REJECT** under `sv_2017`/`sv_2023` (and the reserved family under `verilog_2005`). The residual `localparam p = integer;` still ACCEPTs — that is the LRM-faithful `constant_param_expression ::= … | data_type | $` route (`:1380`), not a keyword-as-primary; likewise `reg q [integer];` under the SV profiles is the legitimate `associative_dimension ::= [ data_type ]`. **Why schema stays 15:** shape-preserving by construction (`identifier` and `non_keyword_identifier` both emit `{body:X}`; a legal name's AST is byte-identical, only reserved keywords change acceptance); ZERO new rules (census `1465` unchanged). Proof: canonical cert `1343/2/1321/UNKNOWN=20` + `verilog_2005` cert `1117/4/773/340` (seeds 0/7/42, byte-identical); `verilog_2005_conformance_gate` GREEN (orphans 0, matrix 219/0); `sv_cert_recognized_union_gate` GREEN (canonical UNKNOWN=20, union UNKNOWN=1, residual `context_member_method_call`); `ast_shape_contract_gate` 18/18; `sv_external_corpus_triage_gate` 14/14; `clippy_on_rust_change` source strict-clean; json `9/0` + regex `198/0` fully_certified. **Known residual (`SV-0036`, OPEN, owned by `.3`):** 35 net-type/gate/structural SV keywords (`wire`/`and`/`always`/…) that are reserved in SV but absent from `reserved_non_keyword_identifier_sv` still leak as primaries under `sv_2017`/`sv_2023` (a reserved-list-completeness defect); `verilog_2005` is unaffected (complete Annex B list). Full matrix in ledger row `SV-0035` and the contract's § "Release 1.0.162".

### 1.0.161 / Contract 1.0.161 — SV-DOLLAR-LRM-FIDELITY.4 (`PGEN-SV-DOLLAR-LRM-FIDELITY-0004`, 2026-07-03), ledger `SV-0029` (`Released`): **THE SV-ONLY `$` ANCHORS RESTORED — `$root` / `$unit` / severity / bare `$` (schema 14 → 15)** — the remaining 7 mangled `sv_dollar_*` keyword-token literals now carry their IEEE `$` spellings, closing the `SV-0029` family (wave 1 = the 12 timing checks at `1.0.159`). What consumers see: IEEE 1800 `$unit::name` scopes (`{kind:"dollar_unit", name:{body:"$unit"}}` inside `package_scope`), module-level elaboration severity tasks (`$fatal;` … → `{kind:"fatal"|"error"|"warning"|"info", args}`), `$root.`-anchored hierarchical names (`root: {kind:"root"}` marker — previously the whole name mis-routed as generic `kind:"system_tf"`) and `$root.`-anchored calls (the NEW `{kind:"rooted_tf", target, attributes, args}` carrier), and the bare-`$` primary (`{kind:"system_dollar"}`) all ACCEPT under `sv_2017`/`sv_2023` and REJECT under strict `verilog_2005`. Mechanics: two inline steal guards on `system_tf_call` (`!( $root . )`, `!( $unit :: )` — the generic token had been PEG-committing on the anchors), three profile-gated `_sv_only` lifts, the `!( identifier )`-guarded `rooted_tf_call_sv_only` carrier (needed because `scoped_or_hierarchical_tf_identifier`'s branch predicate is runtime-broadcast to all branches and hard-rejects rooted results — recorded as an open engine question), and a load-bearing `name:{body:"$unit"}` literal that keeps consumer typedef-exclusion predicates resolvable. The mangled `sv_dollar_*` spellings are ordinary identifiers and keep their identifier readings byte-identically (24/24 cmp-proven). Proof: the 28-probe matrix; canonical cert `1341/2/1319/UNKNOWN=20` `spf=0` seeds 0/7/42 (residual set-identical); `verilog_2005` cert `1147/4/816/327` (the `$root`/`$unit` tokens upgrade witness→PROOF; the bare-`$` token joins NO-reach-by-design); conformance 192 checks/0 mismatches (4 new locks); union re-pinned; 6 new shape samples; clippy source-clean.

### 1.0.160 / Contract 1.0.160 — SV-DOLLAR-LRM-FIDELITY.3 (`PGEN-SV-DOLLAR-LRM-FIDELITY-0003`, 2026-07-02), ledger `SV-0030` (`Released`): **LRM DIGIT + TIMING-CHECK COMPARE-SHAPE RESTORATION (schema 13 → 14, both defect sites)** — the LRM-extraction prefix-merge had truncated the IEEE digit alternatives at TWO sites at every release `1.0.0`–`1.0.159`: `scalar_constant` (the ten IEEE A.7.5.3 alternatives had become 6 digit-less branches, so the rule was dead-in-practice) and UDP `init_val` (IEEE A.5.1 — **sequential-UDP initializers `initial q = 1'b0;` / `1'b1;` REJECTED outright** while the nonsense digit-less `initial q = 1'b;` ACCEPTED); additionally the `scalar_timing_check_condition` eq/case_eq/ne/case_ne compare branches were doubly dead (listed after the bare-`expression` branch AND their greedy full-`expression` lhs consumed `== 1'b0` into its own operand chain — a pure reorder would have been inert), so `e == 1'b0` emitted a flat `{kind:"expression"}`. Fixed grammar-only: the ten digit-carrying alternatives restored at both sites (new `\b`-guarded tokens `1'b0`/`1'b1`/`1'B0`/`1'B1` + `'b0`/`'b1`/`'B0`/`'B1`; digit-less tokens removed with their last sites; `!tick` follow-guards on the bare `1`/`0` branches), and the compare branches made reachable — ordered first with the precedence-restricted `scalar_timing_check_compare_lhs` (operator tier `binary_operator_above_equality` = IEEE 1800-2017 Table 11-2 rows 3–7 only) and a `!binary_operator` follow-guard so operator-continued conditions (`e == 1'b0 && f`) and mixed-precedence forms (`a & b == 1'b0`) deliberately keep the precedence-correct flat parse (9 fallback-probe ASTs byte-identical pre↔post). Schema `13 → 14` (digit-carrying kinds; `{kind:"eq"|"case_eq"|"ne"|"case_ne", lhs, rhs}` now realizable). VERIFIED: UDP digits REJECT→ACCEPT + digit-less ACCEPT→REJECT under both dialects; canonical cert `1337/2/1315/UNKNOWN=20` `spf=0` seeds 0/7/42 (total +9 = exactly the census delta; the 20-rule residual set-identical pre↔post); `verilog_2005` cert `1147/2/819/326` — **`scalar_constant` itself leaves the UNKNOWN residual** (set-diff-proven sole change; the `VERILOG-2005-PROFILE.6.4`-adjudicated not-earnable class-D witness now genuinely earned); shape-contract 18/18 (4 new samples incl. the compare-lhs revert guard); 4 new conformance-corpus locks; external corpus green; clippy source-clean.

### 1.0.159 / Contract 1.0.159 — SV-DOLLAR-LRM-FIDELITY.2 (`PGEN-SV-DOLLAR-LRM-FIDELITY-0002`, 2026-07-02), ledger `SV-0029` (wave 1, `Fix In Progress`): **THE 12 SPECIFY TIMING-CHECK `$` SPELLINGS RESTORED** — the timing-check keyword tokens matched the mangled literal text `sv_dollar_setup` … `sv_dollar_nochange` (an LRM-extraction name-canonicalization leak) at every release `1.0.0`–`1.0.158`, so LRM specify timing checks like `$setup(d, posedge clk, 1);` — core IEEE 1364-2005 §15 / 1800-2017 §31 surface — wrongly REJECTED under every profile while the nonsense spellings accepted. Fixed grammar-only (12 token literals → `/\$setup\b/` …; token names kept): 12/12 LRM spellings now ACCEPT and 12/12 mangled spellings now REJECT under `sv_2017`, `sv_2023`, AND `verilog_2005`; procedural `$…(…)` system-TF calls unaffected (no collision — `specify_item` has no generic system-TF route). Schema stays `13` (timing-check typed kinds unchanged). Cert pins unchanged and re-verified: canonical `1328/2/1306/UNKNOWN=20` (residual set-identical), union `1325/1`, `verilog_2005` `1138/2/809/327`, seeds 0/7/42; corpus green; shape 18/18; 2 new conformance-corpus locks (accept + reject). Still open in the same ledger row: the SV-only `$root`/`$unit`/severity/bare-`$` tokens (leaf `.4`) and `SV-0030` (`scalar_constant`/`init_val` digit loss, leaf `.3`).

### 1.0.158 / Contract 1.0.158 — SV-AST-SHAPE-FIDELITY.2.7 (`PGEN-SV-AST-SHAPE-FIDELITY-0008`, 2026-07-01), ledger `SV-0020`: **CLASS-SCOPED-CALL-PREFIX AST-SHAPE CORRUPTION FIX** — a package-class-scoped static method call (`pkg::C::foo()`, reachable in statement + assignment-RHS context) emitted 2 `<invalid_sequence_access>` in the `class_scoped_call_prefix` `params`/`scope_chain` sub-shapes at every release `1.0.0`–`1.0.157`. Fixed grammar-only via the named-lift `class_scoped_call_prefix_head` (mirroring the in-file `class_scope_type_head`); `params`/`scope_chain` corrupted → clean `[]`, both profiles 2 → 0. `SV-0013` gated the alt's branches but did not lift it — `.2.7` completes that rule. Schema `12 → 13` (candidate #21 of the `SV-AST-SHAPE-FIDELITY.2` sweep, tool-proven reachable in `.2.4`; the LAST of the three reachable-corrupt candidates). Cert `total 1311→1312 / witness 1290→1291 / UNKNOWN=20` unchanged (identical residual; `class_scoped_call_prefix_head` witnessed); SV external corpus 14/14; the 6 fully-certified grammars byte-identical; `cargo test --lib` 739/0 (`systemverilog_ast_shape_contract` PASS, 2 new samples).

### 1.0.157 / Contract 1.0.157 — SV-AST-SHAPE-FIDELITY.2.6 (`PGEN-SV-AST-SHAPE-FIDELITY-0007`, 2026-07-01), ledger `SV-0019`: **NETTYPE WITH-CLAUSE AST-SHAPE CORRUPTION FIX** — both nettype rules (`net_type_declaration_sv_2017` + sv_2023 twin `nettype_declaration_sv_2023`) bound a double-nested inline alt `( non_typedef_package_scope | class_scope )?` inside the br0 `with`-clause group; `nettype logic n with pkg::f;` (a nettype with a package-scoped resolution function) emitted 3 `<invalid_sequence_access>` in the `with_clause` inner scope element at every release `1.0.0`–`1.0.156`. Fixed grammar-only via the shared named-lift `nettype_scope_prefix` (order preserved vs the reversed `scoped_type_scope_prefix`); `with_clause` inner scope corrupted → clean, both profiles 3 → 0 (br1 already clean). Schema `11 → 12` (candidate #14 of the `SV-AST-SHAPE-FIDELITY.2` sweep, tool-proven reachable in `.2.4`). Cert `total 1310→1311 / witness 1289→1290 / UNKNOWN=20` unchanged; SV external corpus 14/14; the 6 fully-certified grammars byte-identical; `cargo test --lib` 739/0 (`systemverilog_ast_shape_contract` PASS, 2 new samples).

### 1.0.156 / Contract 1.0.156 — SV-AST-SHAPE-FIDELITY.2.5 (`PGEN-SV-AST-SHAPE-FIDELITY-0006`, 2026-07-01), ledger `SV-0018`: **INTERFACE-CLASS-TYPE AST-SHAPE CORRUPTION FIX** — a class implementing a package-scoped interface class (`class C implements pkg::IC;`, common in UVM) emitted 1 `<invalid_sequence_access>` in the `interface_class_type` `params` sub-shape at every release `1.0.0`–`1.0.155`. Fixed grammar-only via the named-lift `interface_class_type_head` (mirroring the in-file `base_class_type_head`); `params` corrupted → clean `[]`, both profiles 1 → 0. Schema `10 → 11` (candidate #8 of the `SV-AST-SHAPE-FIDELITY.2` sweep, tool-proven reachable in `.2.4`). Cert `total 1309→1310 / witness 1288→1289 / UNKNOWN=20` unchanged (identical residual; `interface_class_type_head` witnessed); SV external corpus 14/14; the 6 fully-certified grammars byte-identical; `cargo test --lib` 739/0 (`systemverilog_ast_shape_contract` PASS, 2 new samples).

### 1.0.155 / Contract 1.0.155 — SV-AST-SHAPE-FIDELITY.2.3 (`PGEN-SV-AST-SHAPE-FIDELITY-0004`, 2026-07-01), ledger `SV-0017`: **SCOPED-TYPE-REFERENCE AST-SHAPE CORRUPTION FIX — a package/class-scoped type reference now emits clean nested sub-shapes (schema 9 → 10).** The single ubiquitous repro `module m; C::D::E v; endmodule` (a package/class-scoped type reference on a variable declaration) emitted **4 `<invalid_sequence_access>` sentinels** from TWO nested inline-alternation-`$N` corruptions (`ast_return_transform.rs:193/438`, the `SV-0014`/`SV-0015`/`SV-0016`/`SVPP-0001`/`RTL-FE-0002` class): (a) `class_scope_type` (`grammars/systemverilog.ebnf:1067`) bound `head: $1` to an inline 4-way alternation → `params`/`scope_chain` corrupted when the self-annotated `scoped_class_scope_identifier` branch won (the un-lifted twin of `base_class_type`/`class_type`); (b) `scoped_block_type_identifier` (`:1642`) and the byte-identical `scoped_data_type_identifier` (`:1649`) bound `scope: $1` to the shared inline alt `( class_scope | non_typedef_package_scope )` → `type`/`dims` corrupted when the self-annotated `class_scope` branch won. Found by the `SV-AST-SHAPE-FIDELITY.2` grammar-wide inline-alt-`$N` sweep (cluster #1/#5/#6). **Fix (grammar-only, the `SV-0014`/`SV-0015`/`SV-0016` named-lift idiom mirroring the in-file `class_type_head`):** TWO new un-annotated named rules `class_scope_type_head` (lifts the `class_scope_type` alt) + `scoped_type_scope_prefix` (lifts the shared alt, which occurs ONLY at the two `scoped_*_type_identifier` rules), each referenced by its parent (annotation text + positions unchanged); one shared prefix fixes both byte-identical `scoped_*_type_identifier` rules. **Schema 9 → 10:** the `class_scope_type` `params`/`scope_chain` and `scoped_data_type_identifier` `type`/`dims` sub-shapes change consumer-visibly from the corrupted `<invalid_sequence_access>` sentinels to the clean shapes. VERIFIED: `C::D::E v;` `<invalid_sequence_access>` **4 → 0** both profiles (`parse_full` still passes); SV cert `total 1307→1309 / witness 1286→1288 / UNKNOWN=20` unchanged (identical residual; both new rules witnessed; zero newly-unknown) `spf=0` seeds 0/7/42; **SV external corpus 14/14**; `--lint-grammar` clean (1428→1430 rules; `always_matches=8` unchanged); 6 fully-certified grammars byte-identical; `ast_shape_contract` GREEN (4 new scoped-type samples); `cargo test --lib` 739/0; clippy source-clean. Full matrix in ledger row `SV-0017` and the contract's § "AST-Shape Corrections — 1.0.155".
### 1.0.154 / Contract 1.0.154 — SV-AST-SHAPE-FIDELITY.2.2 (`PGEN-SV-AST-SHAPE-FIDELITY-0003`, 2026-07-01), ledger `SV-0016`: **BASE-CLASS-TYPE AST-SHAPE CORRUPTION FIX — a package-scoped base class now emits clean `params`/`scope_chain` (schema 8 → 9).** `base_class_type` (`grammars/systemverilog.ebnf:1094`) bound `head: $1` to an inline 3-way alternation `( scoped_base_class_type_identifier | known_unscoped_base_class_type_identifier | known_unscoped_base_class_type_parameter_identifier )` — the systemic inline-alternation-`$N` positional-model corruption (`ast_return_transform.rs:193/438`, the `SV-0014`/`SV-0015`/`SVPP-0001`/`RTL-FE-0002` class). A derived class whose base is package-scoped (`class D extends pkg::B;` — ubiquitous in SV/UVM, e.g. `extends uvm_pkg::uvm_driver`) emitted **2 `<invalid_sequence_access>` sentinels** in its `base_class_type` `params`/`scope_chain` sub-shapes (when the self-annotated `scoped_base_class_type_identifier` branch won, the rule's own `{head:$1, params:$2, scope_chain:$3}` annotation mis-recursed onto the following `$2`/`$3` slots); the declared-bare `extends B;` was clean. The sibling `class_type` (`:1099`) was already correct because it lifts its head into the named `class_type_head` (`:1102`) — `base_class_type` was the un-lifted twin. Found by the `SV-AST-SHAPE-FIDELITY.2` grammar-wide inline-alt-`$N` sweep (candidate #2). **Fix (grammar-only, the `SV-0014`/`SV-0015` named-lift idiom):** lifted the inline alternation into ONE new un-annotated named rule `base_class_type_head`, referenced by `base_class_type` as its head (annotation text + positions unchanged); one shared rule fixes both profiles (`class_declaration_sv_2017:955` / `_sv_2023:959` share `base_class_type` via `extends`). **Schema 8 → 9:** the `base_class_type` `params`/`scope_chain` change consumer-visibly from the corrupted `<invalid_sequence_access>` sentinels to the clean empty arrays `[]`. VERIFIED: `class D extends pkg::B;` `<invalid_sequence_access>` **2 → 0** both profiles (`parse_full` still passes), declared-bare stays clean; SV cert `total 1306→1307 / witness 1285→1286 / UNKNOWN=20` unchanged (identical residual; `base_class_type_head` witnessed; zero newly-unknown) `spf=0` seeds 0/7/42; **SV external corpus 14/14**; `--lint-grammar` clean (1427→1428 rules; `always_matches=8` unchanged); 6 fully-certified grammars byte-identical; `ast_shape_contract` 11/11 (new `base_class_type_head_scoped` + `base_class_type_scoped` samples); `cargo test --lib` 739/0; clippy source-clean. Full matrix in ledger row `SV-0016` and the contract's § "AST-Shape Corrections — 1.0.154".
### 1.0.153 / Contract 1.0.153 — SV-AST-SHAPE-FIDELITY.2.1 (`PGEN-SV-AST-SHAPE-FIDELITY-0002`, 2026-07-01), ledger `SV-0015`: **FORWARD-TYPEDEF KEYWORD AST-SHAPE CORRUPTION FIX — a forward typedef with a keyword now emits a clean `keyword` (schema 7 → 8).** `type_declaration_sv_2017` (`grammars/systemverilog.ebnf:5209`) and `type_declaration_sv_2023` (`:5225`) branch 6 bound `keyword: $2` to an inline alternation inside an optional group `( kw_enum | kw_struct | kw_union )?` — the systemic inline-alternation-`$N` positional-model corruption (`ast_return_transform.rs:193/438`, the `SV-0014`/`SVPP-0001`/`RTL-FE-0002` class). A forward typedef with a keyword (`typedef enum e_t;` / `struct` / `union` — a common cycle-breaking idiom) emitted **3–4 `<invalid_sequence_access>` sentinels** in its `keyword` sub-shape (branch-1's `class_alias` annotation mis-recursed onto the `keyword` slot); the keyword-less `typedef e_t;` was clean. Found by the `SV-AST-SHAPE-FIDELITY.2` grammar-wide inline-alt-`$N` sweep (candidates #19+#20). **Fix (grammar-only, the `SV-0014` named-lift idiom):** lifted the shared inline alternation into ONE new un-annotated named rule `forward_type_keyword := kw_enum_e338e8e3 | kw_struct_d118e5a3 | kw_union_67ad5a07`, referenced by both br6 as `forward_type_keyword?` (annotation text + positions unchanged); one shared rule fixes both profiles. **Schema 7 → 8:** the forward node's `keyword` changes consumer-visibly from the corrupted `{kind:"class_alias", …}` object to the clean fused-token `[trivia, "enum"]`. VERIFIED: `typedef enum/struct/union e_t;` `<invalid_sequence_access>` **3/4 → 0** both profiles; SV cert `total 1305→1306 / witness 1284→1285 / UNKNOWN=20` unchanged (identical residual; `forward_type_keyword` witnessed; zero newly-unknown) `spf=0` seeds 0/7/42; **SV external corpus 14/14**; `--lint-grammar` clean (1426→1427 rules; `always_matches=8` unchanged); 6 fully-certified grammars byte-identical; `ast_shape_contract` 18/18 (new `forward_typedef_keyword` + `forward_typedef_enum` samples); `cargo test --lib` 739/0; clippy source-clean. Full matrix in ledger row `SV-0015` and the contract's § "AST-Shape Corrections — 1.0.153".
### 1.0.152 / Contract 1.0.152 — SV-AST-SHAPE-FIDELITY.1 (`PGEN-SV-AST-SHAPE-FIDELITY-0001`, 2026-07-01), ledger `SV-0014`: **ANSI-PORT AST-SHAPE CORRUPTION FIX — a typed ANSI module port's `header` now emits the clean `{direction, port_type}` (schema 6 → 7).** `ansi_port_declaration` (`grammars/systemverilog.ebnf:519-520`) bound `header: $1` to an inline alternation inside an optional group `( net_port_header | interface_port_header )?` — the systemic inline-alternation-`$N` positional-model corruption (`ast_return_transform.rs:193/438`, the `SVPP-0001`/`RTL-FE-0002` class). Every typed ANSI module port (`input logic a`) emitted **3 `<invalid_sequence_access>` sentinels** in its `header` sub-shape (`net_port_header`'s clean `{direction, port_type}` mis-nested as `header.header`, the out-of-range `dims`/`default`/`name` slots corrupted); POST-SV-AUDIT did not cover this (it swept only iteration-lead `{first/rest:$N}` list rules). **Fix (grammar-only, the `pp_if_keyword`/`event_separator` named-lift idiom):** lifted the inline alternation into the new un-annotated named rule `ansi_port_header := net_port_header | interface_port_header`; `header: $1` now binds the clean named rule (annotation text unchanged, parse behavior byte-identical). **Schema 6 → 7:** `port.header` changes consumer-visibly from the corrupted sentinel-bearing object to the clean `net_port_header` `{direction, port_type}` (or `interface_port_header` `{kind:"named", name, modport}`). VERIFIED: `module m (input logic a); endmodule` `<invalid_sequence_access>` **6 → 0** (`parse_full` still passes); SV cert `total 1304→1305 / witness 1283→1284 / UNKNOWN=20` unchanged (identical residual; `ansi_port_header` witnessed; zero newly-unknown) `spf=0` seeds 0/7/42; **SV external corpus 14/14**; `--lint-grammar ordered_choice_shadowing=0` (1426 rules; `always_matches 7→8` = the now-visible pre-existing `net_port_header`-always-matches shadow, behavior-preserving); 6 fully-certified grammars byte-identical; `ast_shape_contract` GREEN (new `ansi_port_header_typed` + `ansi_port_typed` samples); `cargo test --lib` 739/0; clippy source-clean. The first member of the reachable inline-alternation-`$N` AST-shape corruption sweep (`SV-AST-SHAPE-FIDELITY`). Full matrix in ledger row `SV-0014` and the contract's § "AST-Shape Corrections — 1.0.152".

### 1.0.151 / Contract 1.0.151 — GRAMMAR-WELLFORMED.H.12.8.3 (`PGEN-GRAMMAR-WELLFORMED-0146`, 2026-06-30), ledger `SV-0013`: **CLASS-SCOPED-CALL REACH-GAP CLOSURE — an interface_class / type_parameter-scoped call now emits `class_scoped_tf` (schema stays 6, acceptance-preserving).** A class-scoped subroutine call whose head is a declared interface class or type parameter (e.g. `\foo ::\foo_0 ()` after `typedef interface class \foo ;`) previously routed through the `package_scope`/`tf` path instead of the dedicated `class_scoped_tf` path, so the `known_unscoped_class_scoped_call_interface_class_identifier` / `…type_parameter_identifier` cousin branches were reachable-but-never-witnessed (cert `UNKNOWN`). The fix (two AND-stacked `lacks_fact_attribute_equals` predicates on the branch-1 `<pkg>::<class>` prefix, the `known_unscoped_block_class_type` template) restricts branch-1 longest-match to a genuine PACKAGE scope so an interface_class / type_parameter head falls through to its dedicated cousin. **Why schema stays 6:** no new node kinds and the strings still parse — the change re-routes an existing call to the correct `class_scoped_tf` node (an acceptance-preserving AST-shape correction, the `SV-0013` category). **Consumer impact:** a class-scoped call over an interface-class / type-parameter head now emits `class_scoped_tf` (was `package_scope`/`tf`); `<pkg>::<class>::call()` over a real package is unchanged. Surfaced grammar-first by the cert-coverage `UNKNOWN`→0 / reach-gap drive (`H.12.8.3`). Full matrix in ledger row `SV-0013` and the contract's 1.0.151 schema-version note.

### 1.0.150 / Contract 1.0.150 — GRAMMAR-WELLFORMED.H.12.8.4.4 (`PGEN-GRAMMAR-WELLFORMED-0143`, 2026-06-29), ledger `SV-0012`: **LIBRARY-COHORT LRM-FIDELITY GRAMMAR FIX — `include` / `library` productions now accept real IEEE 1800 §33 file paths (schema stays 6).** The `file_path_spec` lexeme (`kw_file_path_spec_c26c9dc9 := trivia /file_path_spec\b/`) was a **literal-keyword extraction artifact**: it matched only the literal word `file_path_spec`, so the `include`/`library` library-map productions **rejected every real file path** (`include ../rtl/cpu.v;`, `library mylib /path/to/*.sv;`, `library rtl ./src/*.sv, ./pkg/*.sv -incdir ./inc;` were all rejected at releases ≤ `1.0.149`) and accepted only the literal token text. IEEE 1800-2017 §33.3.1 / Annex A.1.1 define `file_path_spec` as a file-system *path* token (absolute/relative, wildcards `?`/`*`/`...`, `/` separators) — not a keyword. The fix replaces the lexeme with the LRM-faithful path char-class `/[A-Za-z0-9_.\/?*~$+]+/`, keeping the rule's **name and arity** so the typed carrier and AST-dump schema are byte-identical. **Why schema stays 6:** the rule is a single token at the same position — every typed AST shape is unchanged; the change is strictly-more-permissive (the `SV-0008`/`SV-0010`/`SV-0011` category). **Consumer impact:** if you parse SV **library map files** (the `library_text` analysis entry), `include`/`library` statements with real paths now parse (were rejected before); the `systemverilog_file` embedding entry is unaffected — the library cohort is unreachable from it, so embedding consumers see no change. **Documented limitation:** a hyphenated filename (`uvm-1.2`) is not accepted — `-` is excluded from the class so the `-incdir` flag boundary (`minus`+`kw_incdir`) stays separable (no §33 example uses a hyphenated path; a future negative-lookahead extension could admit it). VERIFIED: the three real-path forms REJECT→PASS and the literal `include file_path_spec;` still PASS; SV canonical cert `UNKNOWN=22` + 4-config union `UNKNOWN=3` (`witness=1300`) byte-identical seeds 0/7/42 (`spf=0`, library cohort incl. `kw_incdir` still witnesses); the 6 fully-certified grammars byte-identical `fully_certified=true`; **SV external corpus 14/14** (`parse_fail_total=0`); `ast_shape_contract` 18/18; `--lint-grammar` clean (`1425` rules, pre-existing `always_matches=8` A2 backlog unchanged); clippy source-clean; `cargo test --lib` 771 passed. Surfaced grammar-first by the cert-coverage `UNKNOWN`→0 / library-cohort LRM-fidelity drive (`H.12.8.4`). Full matrix in ledger row `SV-0012` and the contract's 1.0.150 schema-version note.

### 1.0.149 / Contract 1.0.149 — GRAMMAR-WELLFORMED.H.12.5.8.3.2 (`PGEN-GRAMMAR-WELLFORMED-0133`, 2026-06-25), ledger `SV-0011`: **SVA PROPERTY PRECEDENCE-CASCADE GRAMMAR FIX — the IEEE 1800 §16 property infix operators `until` / `s_until` / `until_with` / `s_until_with` / `iff` now parse (schema stays 6).** This is the **property-layer dual** of the 1.0.148 sequence cascade. `property_expr_sv_2017` / `property_expr_sv_2023` were flat **directly-left-recursive** definitions (`property_expr OP property_expr` for `or` / `and` / `iff` / the `until`-family / `implies_binary`); PGEN's left-recursion eliminator handles only the *indirect* bare-reference wrapper-chain shape, not direct inline LR, so the runtime cycle-breaker blocked every left-recursive infix branch. The four `until`-family operators are **property-only** (no other use site), so `a until b` / `a s_until b` / `a until_with b` / `a s_until_with b` (and property-level `a iff b`, and `or`/`and` over genuine property operands like `(always a) or (always b)`) were **rejected at the operator** on every release ≤ `1.0.148` — and the four `until`-family keyword rules were the SV certificate-coverage residual. The fix restructures both profiles into the IEEE 1800-2017 §16 (Table 16-3) **binary-infix precedence cascade** — `property_expr_sv_<profile> := prop_until_sv_<profile> -> $1` over 5 new `prop_*` rules per profile, loosest→tightest `prop_until > prop_iff > prop_or > prop_and > prop_primary` (all right-associative inline `head op self`, so no separate tail rules). **`prop_primary` keeps every non-binary-infix branch byte-identical** — `sequence` / `strong` / `weak` / `paren` / `not` / `nexttime*` / `always*` / `eventually*` / `accept_on`-family / `if` / `case` / `imp_minus` (`#-#`) / `imp_assign` (`#=#`) / `sequence_or_assign` (`|=`) / `instance` / `clocking` — with their bodies still referencing the `property_expr` dispatch. **What you get now:** SVA property infix operators parse with a precedence-nested AST — e.g. `a until b iff c` → `until(a, iff(b,c))` (until is looser than iff per Table 16-3); `a and b or c` → `or(and(a,b), c)`. **Why schema stays 6:** every previously-VALID property parse keeps its exact `{kind:"sv_2017"/"sv_2023"}`-wrapped carrier (the cascade unwraps single-operand levels via `-> $1`, and `prop_primary` is byte-identical — e.g. `not not a` is unchanged); the newly-parsing infix forms **reuse the existing documented `{kind, lhs, rhs}` carriers** (`until` / `s_until` / `until_with` / `s_until_with` / `iff` / `or` / `and`), so there are **no carrier changes at all** (strictly-more-permissive — the `SV-0008` / `SV-0010` category). **Consumer impact:** if you parse SVA properties with infix `until`-family / `iff` operators (or property-operand `or`/`and`), they now succeed (were rejected before); every other property form is byte-identical. **Documented limitation:** the *tight* prefix operators `not` / `nexttime` / `s_nexttime` (Table 16-3 level 6) keep their pre-fix dispatch-body precedence (they bind their whole rhs), so `not a until b` parses as `not (a until b)` rather than the strict-Table-16-3 `(not a) until b` — this is unchanged from the pre-fix behavior (the form simply rejected before), preserves schema 6, and these forms parenthesize in practice. VERIFIED: SV cert `UNKNOWN 26 → 22` (the 4 `until`-family keywords witnessed; `total 1299→1304` (+5 cascade rules), `witness 1272→1281`; deterministic seeds 0/7/42; `spf=0`; zero newly-unknown — set-diff = exactly the 4 `until`-family keywords); the previously-rejected property infix matrix REJECT→PASS with correct precedence, previously-working `not not a` / `always a` / `a #-# b` / `accept_on(a) b` byte-identical; the 6 fully-certified grammars BYTE-IDENTICAL (grammar-only, SV-isolated); **SV external corpus 14/14** (`parse_fail_total=0`); `ast_shape_contract` 18/18; `--lint-grammar` clean (`1425` rules, +10; `non_terminating=0`, `unreachable_rules=0`, `ordered_choice_shadowing=0`, `profile_orphans=0`); clippy source-clean. This completes the `H.12.5.8` SVA infix left-recursion class (sequence layer `SV-0010` + property layer `SV-0011`). Full matrix in ledger row `SV-0011` and the contract's 1.0.149 schema-version note.

### 1.0.148 / Contract 1.0.148 — GRAMMAR-WELLFORMED.H.12.5.8.3.1.1 (`PGEN-GRAMMAR-WELLFORMED-0132`, 2026-06-25), ledger `SV-0010`: **SVA SEQUENCE PRECEDENCE-CASCADE GRAMMAR FIX — the IEEE 1800 §16 sequence infix operators `and` / `or` / `intersect` / `within` now parse (schema stays 6).** The `sequence_expr` rule was a flat **directly-left-recursive** definition (`A := A op A | …`); PGEN's left-recursion eliminator handles only the *indirect* bare-reference wrapper-chain shape, not direct inline LR, so the runtime cycle-breaker blocked every left-recursive infix branch and `a and b` / `a or b` / `a intersect b` / `a within b` (and chained / mixed-precedence forms `a ##1 b ##2 c`, `a or b and c`) were **rejected at the operator** on every release ≤ `1.0.147`. The fix restructures `sequence_expr` into the IEEE 1800-2017 §16 (Table 16-3) **precedence cascade** — `sequence_expr := seq_or_expr -> $1` over 11 new `seq_*` rules, loosest→tightest `or > and > intersect > within > throughout > ## > unary`, each operator layer a `head tail tail*` chain with a named tail rule and a `-> $1` passthrough on bare operands. A companion parser-agnostic generator change (a reach-site passthrough preference in the cert-coverage reach planner) lets the cascade's deep operands witness cleanly. **What you get now:** SVA sequence infix operators parse with a precedence-nested AST — e.g. `a or b and c` → `{kind:"or", lhs:…, rest:[{kind:"and", …}]}`. **Why schema stays 6:** every previously-VALID sequence parse keeps its carrier — the cascade unwraps single-element levels via the `-> $1` passthroughs, so a simple `a [*2]` / `first_match(a)` / `x throughout a` is byte-identical; only the previously-REJECTED infix forms gain new `{kind:"or"/"and"/"intersect"/"within", lhs, rest}` shapes (strictly-more-permissive — the `SV-0008` category, distinct from the carrier-changing `SV-0009`). **Consumer impact:** if you parse SVA sequences with infix `and`/`or`/`intersect`/`within`, they now succeed (were rejected before); the simple sequence forms are unchanged. VERIFIED: SV cert `UNKNOWN 28 → 26` (`kw_intersect`/`kw_within` witnessed; `total 1288→1299` (+11 cascade rules), `witness 1259→1272`; deterministic seeds 0/7/42; `spf=0`; zero newly-unknown); `a and b`/`a or b`/`a intersect b`/`a within b`/`a ##1 b ##2 c`/`a or b and c`/`a intersect b within c` REJECT→PASS, previously-working `first_match(a)`/`a[*2]`/`x throughout a` still PASS; the 6 fully-certified grammars BYTE-IDENTICAL (the engine change inert); **SV external corpus 14/14** (`parse_fail_total=0`); `ast_shape_contract` 18/18; `--lint-grammar` clean (`1415` rules, +11; `non_terminating=0`, `unreachable_rules=0`, `ordered_choice_shadowing=0`); clippy source-clean. The `until`-family **property**-layer operators remain (tracked `H.12.5.8.3.2`, the property cascade — same shape, reuses this engine fix). Full matrix in ledger row `SV-0010` and the contract's 1.0.148 schema-version note.

### 1.0.147 / Contract 1.0.147 — GRAMMAR-WELLFORMED.H.14.3 (`PGEN-GRAMMAR-WELLFORMED-0130`, 2026-06-25), ledger `SV-0009`: **UDP TRUTH-TABLE ENTRY SHAPE RESTORE + EBNF self-hosting 12/12 — schema `5` → `6`.** The `combinational_entry` (`-> {inputs, output}`) and `sequential_entry` (`-> {inputs, current_state, next_state}`) typed carriers, documented in this book's [JSON carrier reference](json-carrier.md) since SV-Slice-18 (`1.0.18`), had **silently regressed to the raw `Sequence` envelope at SV-Slice-66 (`1.0.66`)** — a duplicate identical `->` return annotation was added on each rule (`grammars/systemverilog.ebnf:1168` / `:4639`), and because a single EBNF rule carries only ONE rule-level `return_annotation?`, the generated parser's return-transform compiler could not parse the doubled text and fell back to raw. Deleting the redundant 2nd `->` restores the documented typed shape. **What you get now:** parsing a UDP truth-table row yields the typed object, e.g. `combinational_entry → {"inputs":[{"kind":"0"},{"kind":"0"}],"output":{"kind":"0"}}` (the `:` and `;` tokens are folded away), instead of the raw token sequence. **Why schema bumped 5 → 6:** a previously-VALID UDP parse's carrier changes — the same criterion as the covergroup `SV-0006`/`SV-0007` carrier change (distinct from the SVA `SV-0008` case, which only affected previously-invalid input). **Consumer impact:** if you wrote code against the raw `Sequence` shape these rules emitted at releases `1.0.66`–`1.0.146`, switch to walking the typed `{inputs, output}` / `{inputs, current_state, next_state}` object documented here. This fix also closes `systemverilog.ebnf`'s last EBNF self-parse gap (`error_position=72476`), bringing the generated ebnf meta-parser to **12/12** self-hosting across the tracked grammar set. VERIFIED: SV cert seeds 0/7/42 `total=1288 witness=1259 UNKNOWN=28 spf=0` (unchanged); `ast_shape_contract` 18/18; **SV external corpus 14/14** (`parse_fail_total=0`); clippy source-clean; generated-parser byte-diff is 97 lines confined to exactly the two rules. Full matrix in ledger row `SV-0009` and the contract's 1.0.147 schema-version note.

### 1.0.146 / Contract 1.0.146 — SV-SVA-PROPERTY-FIDELITY.1 (`PGEN-SV-SVA-PROPERTY-FIDELITY-0001`, 2026-06-24), ledger `SV-0008`: **LRM DELIMITER-RESTORE GRAMMAR FIX — the six SVA bounded-property operators now require their IEEE 1800 §16 / Annex A.2.10 `[ ]` brackets (schema stays 5).** Extraction had dropped the literal `[ ]` on the six bounded-property operators in BOTH profiles — `nexttime [constant_expression]`, `s_nexttime [constant_expression]`, `always [cycle_delay_const_range_expression]`, `s_always [constant_range]`, `eventually [constant_range]`, `s_eventually [cycle_delay_const_range_expression]` — each was bracket-less `( <inner> )?`, so without the brackets the optional inner range/expr could not be delimited from the following `property_expr` and the LRM bracket forms were unparseable. The `sv_2017` `nexttime` was further mis-tokenized: the LRM nonterminal `constant_expression` became the literal keyword `constant` (`kw_constant_d810ca96 := trivia /constant\b/`) + a bare `expression`, so the dead `kw_constant_d810ca96` token was reachable only via that mis-synthesis (cert `UNKNOWN`). The fix restores the `[ ]` using the proven in-grammar `lbrack … rbrack` idiom (`const_expr:$3` / `range:$3`, `body:$5`) and deletes the dead token. **Schema stays 5** — the `{kind, const_expr/range, body}` shape is byte-identical (the brackets are parsed then folded away by the return annotation); the `sv_2017` `nexttime_const.const_expr` carrier is corrected from a bogus-only `(kw_constant expression)` group to `constant_expression`, now matching `sv_2023` + LRM (affecting only previously-invalid input). **Consumer notice:** LRM-bracketed bounded-property assertions `nexttime [N] p` / `s_nexttime [N] p` / `always [R] p` / `s_always [R] p` / `eventually [R] p` / `s_eventually [R] p` now parse (were rejected at all releases ≤ `1.0.145`); the non-LRM `nexttime constant N p` is now correctly rejected; the bare `nexttime p` / `always p` forms are unchanged. VERIFIED: SV cert-coverage `UNKNOWN 30 → 29` (the `kw_constant_d810ca96` token removed; `total 1289 → 1288`; witness `1258`; deterministic seeds 0/7/42; `spf=0`; zero newly-unknown — `comm -13` empty); the six LRM forms REJECT→PASS (both profiles), bare PASS, `nexttime constant 3 a` REJECT; `--lint-grammar` clean (pre-existing `always_matches=8` A2 backlog unchanged; `1404` rules, −1); **SV external corpus 14/14** (`parse_fail_total=0`); SV shape-contract GREEN (18/0); `cargo test --lib` 768/0; 6 fully-certified grammars inert (regex `198/198`); clippy source-clean. The property sibling of the covergroup `SV-0006`/`SV-0007` (and `SV-0004`/`SV-0002`) delimiter-drop class. Surfaced + adjudicated grammar-first by the cert-coverage `UNKNOWN`→0 drive (`STORE-AWARE-GEN.4b.15` 9A-ii). Full matrix in ledger row `SV-0008` and the contract's 1.0.146 schema-version note.

### 1.0.145 / Contract 1.0.145 — SV-COVERGROUP-FIDELITY.2 (`PGEN-SV-COVERGROUP-FIDELITY-0002`, 2026-06-24), ledger `SV-0007`: **LRM BINS-SET BRACE GRAMMAR FIX — the covergroup `bins_or_options` range-list form now requires its IEEE 1800 §A.2.11 braces `= { covergroup_range_list }` (schema 4 → 5).** Extraction had dropped the mandatory `{ }` on the bins range-list (`bins_or_options:643` was bare `assign covergroup_range_list*`). Because `covergroup_range_list → covergroup_value_range → covergroup_expression → expression` and the SV expression grammar parses `with(x)` as a `plain_tf` subroutine call, the range-list's first expression swallowed the entire `with(…)` (AST `kind:"call"/"plain_tf", name:"with"`) and the dedicated `( with … with_covergroup_expression … )?` clause stayed empty (`with_expr:[]`) → `with_covergroup_expression` was reachable-but-never-witnessed (cert `UNKNOWN`). The bins sibling of the `SV-0006` (covergroup trans-repeat), `SV-0004` (sequence-repetition) and `SV-0002` (streaming) delimiter-drop class. The fix restores the braces (`… assign lbrace covergroup_range_list rbrace ( with … )? …`, `ranges:$7, with_expr:$9, iff:$10`), so `with(…)` after `}` is the dedicated clause. **Schema 4 → 5** — the typed `range_list` carrier changes: `ranges` collapses one nesting level (`[[…]]` → `[…]`, now individual `covergroup_value_range`s, not one brace-concatenation) and `with_expr` becomes populatable (was always `[]`), the same carrier-shape class as the `SV-0002` bump. **Consumer notice:** the LRM braced bins range-list `bins b = {…} [with (…)]` now parses with the correct structure (and `with(…)` witnesses `with_covergroup_expression`); a bare single value `bins b = v` still parses (via the `set` branch) and `bins b = id with(e)` via `cover_point_with`; the non-LRM bare comma-list `bins b = 1,2,3` is now correctly rejected per IEEE 1800 §A.2.11 (must be `{1,2,3}`). Consumers reading the `range_list` bins AST must adopt the schema-`5` `ranges` (flattened) / `with_expr` (populatable) shape. VERIFIED: SV cert-coverage `UNKNOWN 31 → 30` (`with_covergroup_expression` witnessed; witness `1257 → 1258`; deterministic seeds 0/7/42; `spf=0`; zero newly-unknown — `comm -13` empty; `total=1289`); braced `bins b = {3} with(e)` populates `with_expr`, `bins b = {[0:63],65}` parses the embedded range, bare/`set`/`cover_point_with` forms parse, bare comma-list rejected; **SV external corpus 14/14** (`parse_fail_total=0` — no regression; no SV corpus/test exercises bins); SV shape-contract GREEN (18/0); `cargo test --lib` 768/0; 6 fully-certified grammars inert (regex `198/198`); clippy source-clean. Surfaced + adjudicated grammar-first by the cert-coverage `UNKNOWN`→0 drive (`SV-COVERGROUP-FIDELITY.2`). Full matrix in ledger row `SV-0007` and the contract's 1.0.145 schema-version note.

### 1.0.144 / Contract 1.0.144 — SV-COVERGROUP-FIDELITY.1 (`PGEN-SV-COVERGROUP-FIDELITY-0001`, 2026-06-24), ledger `SV-0006`: **LRM DELIMITER-RESTORE GRAMMAR FIX — the covergroup `trans_range_list` repeat forms now require their IEEE 1800 §A.2.11 `[ ]` brackets (schema stays 4).** Extraction had dropped the literal `[ ]` on the covergroup trans-repeat forms — `[* repeat_range]` (consecutive), `[-> repeat_range]` (goto), `[= repeat_range]` (nonconsecutive) — the direct covergroup sibling of the `SV-0004` sequence-repetition family (and `SV-0002` streaming). Pre-fix the bracket-less `trans_item ( star repeat_range )?` / `( implies … )?` / `( assign … )?` (`star := "*"`, `implies := "->"`, `assign := "="`) collided with the multiply/arrow/eq operators, so `trans_item`'s greedy `expression` parsed `a*b` as one multiplication — the optional `repeat_range` never fired → `repeat_range` was operator-shadowed (never witnessed → cert `UNKNOWN`) and the bracketed LRM forms `(1[*2])`/`(1[->2])`/`(1[=2])`/`(1=>2[*3])` were unparseable. The fix restores the `[ ]` using the proven in-grammar `lbrack … rbrack` idiom (`range: $4`). **Schema stays 4** — the emitted `{kind, range}` shape is byte-identical (the brackets are parsed then folded away by the return annotation). **Consumer notice:** LRM-bracketed covergroup trans repetitions `(1[*N])`/`(1[->N])`/`(1[=N])` now parse (were rejected at all releases ≤ `1.0.143`); a bare `(1*2)` still parses as a `simple` multiply trans_item (no regression). VERIFIED: SV cert-coverage `UNKNOWN 32 → 31` (`repeat_range` witnessed; witness `1256 → 1257`; deterministic seeds 0/7/42; `spf=0`; zero newly-unknown; `total=1289`); `(1[*2])`/`(1[->2])`/`(1[=2])`/`(1=>2[*3])`/`([1:2][*3])` REJECT→PASS; bare `(1*2)`/`(1)`/`(1=>2)`/`(1,2)` still PASS; `--lint-grammar` clean (pre-existing `always_matches=8` A2 backlog unchanged); **SV external corpus 14/14** (`parse_fail_total=0` — no regression); SV shape-contract GREEN (18/0; no SV sample exercises trans/bins); `cargo test --lib` 768/0; 6 fully-certified grammars inert (regex `198/198`); clippy source-clean. Surfaced + adjudicated grammar-first by the cert-coverage `UNKNOWN`→0 drive (`STORE-AWARE-GEN.4b.14` 9C-iii — the delimiter-drop class, exactly as `SV-0004`/`SV-0002`). Full matrix in ledger row `SV-0006` and the contract's 1.0.144 schema-version note. (The companion `with_covergroup_expression` bins-set-brace fidelity — LRM `= { covergroup_range_list }` — is tracked as `SV-COVERGROUP-FIDELITY.2`, a separate heavier accepted-language change.)

### 1.0.143 / Contract 1.0.143 — GRAMMAR-WELLFORMED.H.12.6.1 (`PGEN-GRAMMAR-WELLFORMED-0111`, 2026-06-17), ledger `SV-0005`: **PRODUCER LEFT-RECURSION GRAMMAR FIX (AST-shape + reachability) — ternary module-path conditions now emit a clean `module_path_conditional_expression` node, and the last `no_path` budget case is retired (schema stays 4).** `module_path_conditional_expression` (IEEE 1800 Annex A.8.3) was stranded by an indirect **left-recursion** in its producer `module_path_expression` (mpce's condition referenced `module_path_expression`, whose first branch was mpce). The left-recursion eliminator rewrote the producer into `module_path_expression_lr_base`/`_lr_suffix` and left `module_path_conditional_expression` as an **unreferenced** rewritten seed (its generated parse fn defined-but-never-called → `no_path`), while the `_lr_suffix` reconstruction leaked raw internal `wrapper_specs` metadata into the ternary module-path AST. The fix makes mpce's condition the non-left-recursive `module_path_expression_operand` chain (same accepted language — the right-associative ternary reading), so `module_path_expression` is natively non-left-recursive (no elimination; the `…_lr_base`/`…_lr_suffix` synthetic rules vanish), mpce is positively reachable + **witnessed**, and the `wrapper_specs` leak is gone. **Schema stays 4** — the conditional module-path's prior `wrapper_specs`-leaking form was never a realized clean shape a consumer could build against, and the common (non-conditional) chain module-path is byte-identical. **Consumer notice:** ternary module-path conditions (`a ? b : c` in `specify` state-dependent paths) now emit a clean `{kind:"conditional", body:{condition, attributes, then_expr, else_expr}}` node instead of leaking raw `wrapper_specs`; the accepted language is unchanged (the ternary parsed before too); non-conditional module paths are byte-identical. VERIFIED: SV cert-coverage `UNKNOWN 86 → 84` (`module_path_conditional_expression` witnessed + `module_path_expression_lr_suffix` removed; witness `1204`; total `1291 → 1289`; deterministic seeds 0/7/42; `spf=0`); `no_path 20 → 19`; the genuineness oracle emits a clean `conditional` node with `0` `wrapper_specs` (was `0`/`3`); syntax-closure `unreachable_rules=0` (`max_unreachable_rules` budget `1 → 0`, contract v6 — the linter's "no unreachable rules" is now literally true for SV); **SV external corpus 14/14** (`parse_fail_total=0`); `stimuli_cross_family_platform_gate` PASS; SV shape-contract GREEN; clippy source-clean. Surfaced + adjudicated grammar-first by the cert-coverage `UNKNOWN`→0 drive (`GRAMMAR-WELLFORMED.H.12.6` `no_path` re-audit) — a producer-wiring flaw FIXED, never deleted. Full matrix in ledger row `SV-0005` and the contract's 1.0.143 schema-version note.

### 1.0.142 / Contract 1.0.142 — GRAMMAR-WELLFORMED.H.12.5.5.3.3.3 (`PGEN-GRAMMAR-WELLFORMED-0105`, 2026-06-17), ledger `SV-0004`: **LRM DELIMITER-RESTORE GRAMMAR FIX — the `boolean_abbrev` sequence-repetition family now requires its IEEE 1800 §A.8.1 `[ ]` brackets (schema stays 4).** Extraction had dropped the literal `[ ]` on `consecutive_repetition` (`[* N]` / `[*]` / `[+]`), `goto_repetition` (`[-> N]`), and `non_consecutive_repetition` (`[= N]`, both profiles), so the bracket-less forms collided with the `*` / `->` / `=` operators: the grammar **REJECTED valid SV** sequence repetitions (`a[*3]`, `a[->2]`, `a[=2]`) and wrongly accepted the bracket-less spellings; `goto_repetition` was operator-shadowed (never witnessed → cert `UNKNOWN`). The fix restores the `[ ]` using the proven in-grammar `lbrack … rbrack` idiom (`range: $3`). **Schema stays 4** — the emitted `{kind, range}` / `{range}` shape is byte-identical (the brackets are parsed then folded away by the return annotation). **Consumer notice:** LRM-bracketed sequence repetitions `[*N]`/`[*]`/`[+]`/`[->N]`/`[=N]` now parse (were rejected at all releases ≤ `1.0.141`); the bare `*N`/`->N`/`=N` sequence-repetition spellings are now correctly rejected per the LRM; a bare `a*3` still parses as a multiplication expression (no regression). VERIFIED: SV cert-coverage `UNKNOWN 87 → 86` (`goto_repetition` closed; witness `1203 → 1204`; deterministic seeds 0/7/42; `spf=0`; zero newly-unknown; 1291 rules); `--lint-grammar` clean (pre-existing `always_matches=6` A2 backlog unchanged); **SV external corpus 14/14** (`parse_fail_total=0`, both uvm cases — no regression); SV shape-contract GREEN (samples=3 aligned=3 drift=0); clippy source-clean. Surfaced + adjudicated grammar-first by the cert-coverage `UNKNOWN`→0 drive (the attribution rule's delimiter-drop class, same family as `SV-0002`). Full matrix in ledger row `SV-0004` and the contract's 1.0.142 schema-version note.

### 1.0.141 / Contract 1.0.141 — GRAMMAR-WELLFORMED.H.12.5.5.3.3.4.2.2.1 (`PGEN-GRAMMAR-WELLFORMED-0100`, 2026-06-16), ledger `SV-0003`: **STORE-GATE GRAMMAR FIX — a module-scope class-handle declaration `C a;` is no longer mis-parsed as a `net_declaration` (schema stays 4).** `checked_nettype_identifier`'s `@predicate` is tightened from `has_fact(type_name, $body)` to `fact_attribute_equals(type_name, $body, declaration_family, nettype)`, so the `net_declaration` user-nettype branch resolves ONLY to a declared nettype. A declared **class** is also a `type_name`, so the bare gate wrongly consumed `C a;` (C a declared class) as a net declaration — emitting no `variable_binding` fact and blocking class-handle member-method chains (`C a; … a.b[0].c()` was REJECTED). Post-fix `C a;` routes to `data_declaration` (a class variable — per IEEE 1800 a class is not a `net_type`) and the chain parses and witnesses `context_member_method`. **Schema stays 4** — no new shape vocabulary; `C a;` re-routes between the two *existing* `net_declaration`/`data_declaration` shapes. **Consumer notice:** acceptance + categorisation correction — a module-scope class-handle declaration now parses as `data_declaration` (was `net_declaration`) and module-scope class-handle member-method chains now parse; built-in nets (`wire a;`) and real user nettypes (`nettype logic NT; NT a;`) are unaffected. VERIFIED: SV cert-coverage byte-identical (`total=1292 witness=1203 UNKNOWN=88 spf=0`, seeds 0/7/42); `--lint-grammar` clean; SV external corpus 14/14; `stimuli_cross_family_platform_gate` PASS; SV shape-contract GREEN (+ the `systemverilog_class_handle_decl_is_data_declaration_not_net` lock). Surfaced by the cert-coverage `UNKNOWN`→0 drive as a bug-finding oracle (the `-0099` class-handle WHY+WHERE). Full matrix in ledger row `SV-0003` and the contract's 1.0.141 schema-version note.

### 1.0.140 / Contract 1.0.140 — GRAMMAR-WELLFORMED.H.12.5.5.3.3.5 (`PGEN-GRAMMAR-WELLFORMED-0096`, 2026-06-16), ledger `SV-0002`: **GRAMMAR FIX — restores the LRM stream-concatenation delimiters dropped in extraction (schema 3 → 4).** `stream_concatenation` regains its mandatory inner `{ }` (and `body` becomes a clean array of stream-expressions), `stream_expression` regains the literal `[ ]` around its `with`-clause range, and `streaming_concatenation`'s optional `slice_size` is guarded `( slice_size &lbrace )?` so it no longer greedily eats the brace-concatenation that IS the mandatory stream-concatenation. Effect: the LRM with-bracket form `{<< 4 {a with [3:0]}}` now parses (was REJECTED at every shipped release), the canonical brace forms `{>> {data}}` / `{<< 8 {a, b}}` parse with the *correct* `stream_concatenation` structure, and a bare `with X` without brackets is correctly rejected per IEEE 1800 §A.8.1. **Schema 3 → 4** — `stream_concatenation.body` changes from a raw quantified node to an array (`[$2, $3::2*]`); consumers reading streaming-concatenation ASTs must adopt the new shape (inputs with no streaming concatenation are byte-identical). Closes the certificate-coverage `array_range_expression` `UNKNOWN` (SV cert `UNKNOWN 89 → 88`, seeds 0/7/42, `spf=0`). VERIFIED: SV external corpus 14/14 (uvm's own `{>>{i}}` / `{<<{v}}` streaming forms parse — no regression); `--lint-grammar` clean; `stimuli_cross_family_platform_gate` PASS. Full matrix in ledger row `SV-0002` and the contract's 1.0.140 Highlights.

### 1.0.139 / Contract 1.0.139 — GRAMMAR-WELLFORMED.H.11.5-FIX (`PGEN-GRAMMAR-WELLFORMED-0075`, 2026-06-11), ledger `SV-0001`: **ENGINE FIX — the generated layout skipper's `#`-to-EOL comment arm no longer exists in the SV parser**, so it can no longer steal a real `#` token (delays, `#( )` parameter lists) during speculative comment attempts and memo-poison the parse. Grammar-valid `#`+comment shapes that were rejected on every shipped release now parse. Release bump, schema stays 3; acceptance widening only for grammar-valid inputs (every previously-parsed input yields a byte-identical AST). Full matrix in the ledger row.

### 1.0.136 / Contract 1.0.136 — SV-EXH-PROOF.5.2.4.1 (`PGEN-SV-EXH-PROOF-0107`, 2026-05-30): **BEHAVIOUR-TIGHTENING grammar fix — a scoped type reference `Head::member` where `Head` is a plain (non-class) `typedef` is now correctly REJECTED in `data_type` position.** Release bump, schema stays 3.

> SV-EXH-PROOF.5.2.4.1 (`PGEN-SV-EXH-PROOF-0107`, 2026-05-30). Surfaced by the `sv_semantic_scope_contract` gate's `parameter_typedef_fail` case: `module m; typedef int T; localparam int Y = T::P; endmodule` was wrongly ACCEPTED. `T` is a plain `typedef int` (not a class/package), so the scoped form `T::P` is invalid per IEEE 1800 (a non-class type has no scope members). **Root cause (tools-first, `--parse-dump-ast-pretty`):** the value `T::P` parsed via `constant_param_expression`'s `data_type` branch → `data_type`'s `provisional_unscoped_block_class_type` branch, which was gated ONLY by `has_fact(type_name, head)`; since `typedef int T` emits a `type_name` fact, `T` satisfied that gate even with the `::P` scope_chain present. **Fix:** `data_type`'s class-type branch now references the typedef-EXCLUDING sibling `known_unscoped_block_class_type` (gated `has_fact(type_name, head)` **and** `lacks_fact_attribute_equals(type_name, head, declaration_family, typedef)`) instead of the permissive `provisional_*`. AST shape UNCHANGED (same `{head, params, scope_chain}` body, same `provisional_class_type` AST kind), so schema stays 3. `block_data_type` (class-internal scoped types) keeps the permissive `provisional_*` branch (minimal blast radius). **Consumer notice (behaviour-tightening, same category as 1.0.127):** inputs that accidentally relied on `Head::member` being accepted for a non-class typedef head are now rejected; valid forms — bare typedef-as-datatype `T x;`, package scope `pkg::C`, real class scope `Class::member` — are unaffected. **EMPIRICAL VERIFICATION:** minimal repro now rejected (furthest_position=52, does not consume full input); the three valid sanity forms still pass; `sv_semantic_scope_contract` gate **20/20** (was 19/20); **SV external corpus 14/14** (no uvm regression — uvm uses scope-resolution heavily); SV shape-contract GREEN; lib (--features generated_parsers, `--lib`) **609/609**. NO-WORKAROUNDS level 1 (existing semantic-annotation gate, per the store-consultation principle).

### 1.0.135 / Contract 1.0.135 — SV-EXH-PROOF.3.3.4.b.6.2.PP.1 (`PGEN-SV-EXH-PROOF-0099`, 2026-05-28): **SV PREPROCESSOR FIX — macro string-substitution `\`"ARG[+]\`"` now handles literal text between the macro-arg and the closing `\`"`** — uvm_pkg ×{2017,2023} flip FAIL → PASS, **SV external corpus 10/14 → 12/14**! Release bump, schema stays 3, strictly-more-permissive.

> SV-EXH-PROOF.3.3.4.b.6.2.PP.1 (`PGEN-SV-EXH-PROOF-0099`, 2026-05-28). The `.PP.0` investigation (also this slice; pure-docs framing absorbed into the fix slice since the fix turned out to be ~50 lines) pinned that `.37.10`'s final 20K-bytes-from-end position was UNEXPANDED UVM `\`uvm_field_*` macro template residue. Per LRM §22.5.3.2, `\`"` opens a "macro string with substitution" region terminated by a matching `\`"`. Pre-fix `substitute_function_macro_body` in `rust/src/sv_preprocessor.rs:1714-1763` assumed the region contained ONLY a single macro-arg identifier — so UVM's `\`m_uvm_field_qda_int`'s `\`"ARG[+]\`"` pattern was mis-expanded: `[+]` was emitted bare and the closing `\`"` became literal `\`"`. Fix: scan forward to the matching closing `\`"`, substitute macro args within the entire region content, emit a single `"..."` string literal. Backwards-compat preserved via fallthrough to the legacy single-ident form when no matching close is found. **uvm_pkg ×{2017,2023} now PASS** (were the deepest residual since the SV-EXH-PROOF tree began). Tools-first per Recipe 2 + a focused source-read of the preprocessor function. **EMPIRICAL VERIFICATION:** minimal repro `\`define t(ARG) push_back(\`"ARG[+]\`")` / `\`t(abc)` produces `push_back("abc[+]")` post-fix (was `push_back("abc"[+]\`")`); lib (--features generated_parsers) **609/609 PASS**; lib (no-features) **548/548 PASS**; SV preprocessor unit tests 22/22 PASS; clippy ✅; **SV external corpus triage gate 12 PASS / 2 FAIL / 0 TIMEOUT** (was 10/4/0 — uvm_pkg ×{2017,2023} flipped FAIL → PASS). Residual 2 FAIL = uvm_compat_pkg ×{2017,2023} which is H1 territory (cross-package extends, depends on uvm_pkg producing a library artifact — now unblocked since uvm_pkg parses).

### 1.0.134 / Contract 1.0.134 — SV-EXH-PROOF.3.3.4.b.6.2.37.10 (`PGEN-SV-EXH-PROOF-0097`, 2026-05-27): SV grammar fix — new `implicit_class_rooted_method_chain` rule + integration into `call_primary` + `method_call_initial` so `this.member[i].method()` / `super.member[i].method()` 3-level chains parse. Release bump, schema stays 3, strictly-more-permissive. uvm_pkg furthest 2,229,367 → 3,006,234 — **~99.3% through, 20K bytes from the end**.

> SV-EXH-PROOF.3.3.4.b.6.2.37.10 (`PGEN-SV-EXH-PROOF-0097`, 2026-05-27). Surfaced by `.37.9`'s 2,229,367 furthest reaching `this.m_update_thread[element].kill();` at uvm_pkg line 66850. Per IEEE 1800 §A.6.6+§A.8.4 `class_qualifier` includes `implicit_class_handle .` (where `implicit_class_handle` covers `this` / `super` / `this.super`) as a primary-prefix, so 3+level chains rooted at `this`/`super` are LRM-valid. **Structural complement of `.37.5`'s `context_member_method_call` fix:** that slice handled IDENTIFIER-rooted 3+level chains (`a.b[i].method()`) via the `has_fact(variable_binding, $head)`-gated rule; this slice handles IMPLICIT-CLASS-HANDLE-rooted chains (`this.b[i].method()`) via a new parallel rule `implicit_class_rooted_method_chain` (no predicate gate needed — `this`/`super` are keywords, unambiguous). The new rule has the same `constant_bit_select &dot` member-loop shape from `.37.5` so indexed members parse, plus a `constant_bit_select` on the head identifier so the FIRST member can also be indexed. Integrated into BOTH `call_primary` (expression context) AND `method_call_initial` (statement context — `subroutine_call` path used by `this.x.kill();`-as-statement). Tools-first per Recipe 2 — byte 2,229,367 → line 66850 col 42 → minimal repro `this.arr[0].kill();` FAILed immediately. **Release bump, NO schema bump** — strictly-more-permissive (SVPP-0002/REGEX-0083). VERIFIED: 5-way repro all PASS — `arr[0].kill()` (was PASS) / `this.arr[0].kill()` (was FAIL) / `this.arr.kill()` (was PASS) / `super.arr[0].kill()` (was FAIL) / `this.x[i].sub[j].run()` (was FAIL); lib (--features generated_parsers) **609/609 PASS**; lib (no-features) **548/548 PASS**; clippy ✅; SV external corpus triage gate **10 PASS / 4 FAIL / 0 TIMEOUT** — binary stable; **uvm_pkg furthest_position 2,229,367 → 3,006,234** (+776,867 bytes deeper, +35% more; reaches ~3.006M of ~3.026M preprocessed bytes = **~99.3% through uvm_pkg, only 20K bytes from the end**). uvm_compat_pkg unchanged at 116752 (H1 territory).

### 1.0.133 / Contract 1.0.133 — SV-EXH-PROOF.3.3.4.b.6.2.37.9 (`PGEN-SV-EXH-PROOF-0096`, 2026-05-27): SV grammar fix — `base_class_type` accepts type-parameter heads so `class C #(type T) extends T;` parses. Release bump, schema stays 3, strictly-more-permissive.

> SV-EXH-PROOF.3.3.4.b.6.2.37.9 (`PGEN-SV-EXH-PROOF-0096`, 2026-05-27). Surfaced by `.37.8`'s 1,582,112 furthest reaching `uvm_port_base`'s `virtual class uvm_port_base #(type IF=uvm_void) extends IF;`. Per IEEE 1800 §A.1.2 a class's `extends class_type` accepts any `class_type` — including one whose head is a type-parameter. Pre-fix `base_class_type` (line 1090) accepted only `class` and `pkg::class` heads (gated by `fact_attribute_equals(type_name, $body, declaration_family, class)` for the unscoped branch) — missing the type-parameter alternative (gated by `declaration_family=type_parameter`). **Fix:** new sibling rule `known_unscoped_base_class_type_parameter_identifier` (gated by the type_parameter predicate) added as a third alternative in `base_class_type`'s head choice (alongside `scoped_base_class_type_identifier` and `known_unscoped_base_class_type_identifier`). Pure grammar LRM alignment; existing predicate primitive reused. Tools-first per Recipe 2 from `docs/book/src/parseability-probe-debug.md` — pinned the defect in ~2 min via furthest_position byte → line+col → minimal repro. **Release bump, NO schema bump** — strictly-more-permissive (SVPP-0002/REGEX-0083). VERIFIED: 4-way extends repro all PASS (`extends concrete` / `extends concrete#(int)` / `extends IF` / `extends IF#(int)`); lib (--features generated_parsers) **609/609 PASS**; lib (no-features) **548/548 PASS**; clippy ✅; SV external corpus triage gate **10 PASS / 4 FAIL / 0 TIMEOUT** — binary stable; **uvm_pkg furthest_position 1,582,112 → 2,229,367** (+647K bytes deeper, ~41% more — biggest single-slice gain since `.37.2`; reaches ~2.23M of ~3.0M preprocessed bytes = ~74% through uvm_pkg). uvm_compat_pkg unchanged at 116752 (H1 territory).

### 1.0.132 / Contract 1.0.132 — SV-EXH-PROOF.3.3.4.b.6.2.37.8 (`PGEN-SV-EXH-PROOF-0095`, 2026-05-27): SV grammar LRM-extraction fix — `empty_unpacked_array_concatenation` now requires the LRM literal `'{ }` (tick + braces) instead of the prior `lbrace epsilon rbrace` (which was missing the tick AND referenced an undefined `epsilon` rule, so the construct was never parseable). Release bump, schema stays 3, strictly-more-permissive.

> SV-EXH-PROOF.3.3.4.b.6.2.37.8 (`PGEN-SV-EXH-PROOF-0095`, 2026-05-27). Surfaced by `.37.7`'s 1,508,106 furthest reaching `uvm_cache::get`'s `return '{};`. Per LRM §A.6.7: `empty_unpacked_array_concatenation ::= '{ }`. pgen's rule at line 1937 was `lbrace epsilon rbrace` with TWO defects: (1) missing the leading `tick` (apostrophe), (2) `epsilon` is not a defined rule in the grammar (no `epsilon :=` exists), so the rule referenced an undefined symbol and could never match at all. Fix: replace with `tick lbrace rbrace` per LRM literal. Pure grammar LRM alignment; no engine change. Per [[feedback_tools_first_no_guessing]] — used the documented Recipe 2 from `docs/book/src/parseability-probe-debug.md` (furthest_position byte → line + col → minimal repro of the visible syntax) to pin the defect in 2 minutes without bisecting. **Release bump, NO schema bump** — strictly-more-permissive (SVPP-0002/REGEX-0083 category; previously-undefinable `'{}` now parses). VERIFIED: minimal repros `q = '{};` + `q = '{1, 2};` PASS; lib (--features generated_parsers) **609/609 PASS**; lib (no-features) **548/548 PASS**; clippy ✅; SV external corpus triage gate **10 PASS / 4 FAIL / 0 TIMEOUT** — binary stable; **uvm_pkg furthest_position 1,508,106 → 1,582,112** (+74K bytes deeper, ~5% more; reaches ~1.58M of ~3.0M preprocessed bytes = ~53% through uvm_pkg). uvm_compat_pkg unchanged at 116752 (H1 territory).

### 1.0.131 / Contract 1.0.131 — SV-EXH-PROOF.3.3.4.b.6.2.37.7 (`PGEN-SV-EXH-PROOF-0094`, 2026-05-26): SV grammar fix — `expression_or_cond_pattern` gains `inside_expression` sibling branch so `if (x inside {...})` parses (release bump, schema stays 3, strictly-more-permissive)

> SV-EXH-PROOF.3.3.4.b.6.2.37.7 (`PGEN-SV-EXH-PROOF-0094`, 2026-05-26). Surfaced by `.37.6`'s 1,278,335 furthest_position reaching `uvm_root::die` where `if (get_core_state() inside {UVM_CORE_PRE_ABORT, UVM_CORE_ABORTED})` was unparseable — `inside_expression` reachable from `expression` (`expression := expression_base | inside_expression`) but NOT from `expression_or_cond_pattern` (which used `expression_base` directly). The `cond_predicate` path used by `if`/`while`/`for` conditions therefore bypassed `inside_expression`. **Fix:** add `inside_expression` as the first sibling alternative in `expression_or_cond_pattern` (keeping `expression_base` second and `cond_pattern` third so the existing parse priority is preserved). The naive `expression_base → expression` replacement caused a uvm_pkg furthest regression 1,278,335 → 191,248 during in-slice verification (probably PEG ambiguity in deep nested expressions); the sibling-branch alternative avoids that regression entirely. **Release bump, NO schema bump** — strictly-more-permissive (previously-unparseable `if (x inside {…})` now parses); SVPP-0002/REGEX-0083 category. VERIFIED: 6-way minimal repro all PASS (`if`/`while` condition + `inside` with literal-set / range / identifier-set); lib (--features generated_parsers) **609/609 PASS**; lib (no-features) **548/548 PASS**; clippy ✅; SV external corpus triage gate **10 PASS / 4 FAIL / 0 TIMEOUT** — binary stable; **uvm_pkg furthest_position 1,278,335 → 1,508,106** (+230K bytes deeper, ~18% more; reaches ~1.51M of ~3.0M preprocessed bytes = ~50% through uvm_pkg). uvm_compat_pkg unchanged at 116752 (H1 territory).

### 1.0.130 / Contract 1.0.130 — SV-EXH-PROOF.3.3.4.b.6.2.37.6 (`PGEN-SV-EXH-PROOF-0093`, 2026-05-26): SV grammar LRM-extraction fix — `inside_expression` + `value_range` now require LRM brackets (`inside { ... }` braces around the range list, and `[ expr : expr ]` brackets around the range alternative) — release bump, schema stays 3, strictly-more-permissive

> SV-EXH-PROOF.3.3.4.b.6.2.37.6 (`PGEN-SV-EXH-PROOF-0093`, 2026-05-26). Surfaced by `.37.5`'s 1,121,290 furthest_position reaching `uvm_phase::jump`'s `phases = phases.find(item) with (item.get_state() inside {[UVM_PHASE_STARTED:UVM_PHASE_CLEANUP]});`. Two LRM-extraction defects (one rule each, both in `grammars/systemverilog.ebnf`):
>
> **(a) `value_range_sv_2017` (line 5313) + `value_range_sv_2023` (line 5317).** Per IEEE 1800 §A.8.3 the LRM defines `value_range ::= expression | [ expression : expression ]`. The pgen rule had `( expression colon expression )?` — missing the brackets AND wrapped in `?` which made the rule silently match empty as a fallback. Fix: require `lbrack expression colon expression rbrack` literally; emit `{kind: "range", lo: $2, hi: $4}` (the `lo`/`hi` convention from line 538's `constant_range_expression`). sv_2023 same fix for all 4 bracketed range variants (range, dollar_lo, dollar_hi, tolerance — each now wrapped in `lbrack ... rbrack`).
>
> **(b) `inside_expression_sv_2017` (line 2374) + `inside_expression_sv_2023` (line 2378).** Per LRM §A.8.3 the `inside_expression ::= expression inside { open_range_list }` literal `{` and `}` braces. The pgen rule had `kw_inside open_range_list*` — missing the braces AND wrapped in `*` which made the rule silently succeed without consuming the list. Fix: require `lbrace open_range_list rbrace` literally (matching the LRM exactly).
>
> Both fixes are strictly-more-permissive (previously-unparseable `inside {[1:5]}` now parses; previously-PASSING `inside {1, 2}` still parses through the same path, just now via the explicit braces). NO new annotation, NO engine change — pure grammar LRM alignment. VERIFIED: 5-way inside-variant repro all PASS (`inside {1, 2}`, `inside {[1:5]}`, `inside {[1:5], 10}`, `inside {[$:5]}`, `inside {[1:$]}`); lib (--features generated_parsers) **609/609 PASS**; lib (no-features) **548/548 PASS**; clippy ✅; SV external corpus triage gate **10 PASS / 4 FAIL / 0 TIMEOUT** — binary stable; **uvm_pkg furthest_position 1,121,290 → 1,278,335** (+157K bytes deeper, ~14% more; reaches ~1.28M of ~3.0M preprocessed bytes = ~42% through uvm_pkg). uvm_compat_pkg unchanged at 116752 (H1 territory).

### 1.0.129 / Contract 1.0.129 — SV-EXH-PROOF.3.3.4.b.6.2.37.5 (`PGEN-SV-EXH-PROOF-0092`, 2026-05-26): SV grammar fix — `context_member_method_call`'s member-loop gains `constant_bit_select` after each identifier so `obj.member[i].method(args)` 3+level chains with indexed members parse cleanly (release bump, schema stays 3, strictly-more-permissive)

> SV-EXH-PROOF.3.3.4.b.6.2.37.5 (`PGEN-SV-EXH-PROOF-0092`, 2026-05-26). Surfaced by `.37.3`'s 866292 furthest_position which reached uvm_pkg's `uvm_report_message_element_container::do_copy` foreach body: `elements.push_back(urme_container.elements[i].clone())` — the argument `c.elements[i].clone()` is a 3-level chain `instance . indexed_member . method_call`. Pre-fix the `context_member_method_call` rule (added by `.b.6.2` for identifier-rooted chains like `a.b.c(x)`) used `( dot identifier &dot )+` for intermediate members — bare identifier only, no bit-select. So `dot elements [i]` failed the `&dot` lookahead (next char is `[` not `.`) and the whole rule was rejected. **Fix (one rule edit, 1 token addition):** the member loop becomes `( dot identifier constant_bit_select &dot )+` — `constant_bit_select` is `( lbrack constant_expression rbrack )*` so matches empty when there's no `[...]`. Backwards-compatible: `a.b.c.method()` parses byte-identically to pre-fix (no shape change on the no-index path); `a.b[i].c.method()` (was unparseable) now parses with the same `{head, members, method, chain}` shape. **Release bump, NO schema bump** — strictly-more-permissive (previously-unparseable inputs now parse; no shape change); SVPP-0002/REGEX-0083 category. VERIFIED: 6-way minimal repro all PASS (`c.elements.clone()`, `c.elements[i].clone()`, `c.elements[0].clone()`, `c.a.b.c.method()`, `c.a.b[i].c.method()`, `c.a[i].b.method()`); lib (--features generated_parsers) **609/609 PASS**; lib (no-features) **548/548 PASS**; clippy ✅; SV external corpus triage gate **10 PASS / 4 FAIL / 0 TIMEOUT** — binary stable; **uvm_pkg furthest_position 866292 → 1,121,290** (+255K bytes deeper, ~30% more); uvm_compat_pkg unchanged at 116752 (H1 territory). uvm_pkg furthest now ~1.12M of ~3.0M preprocessed bytes.

### 1.0.128 / Contract 1.0.128 — SV-EXH-PROOF.3.3.4.b.6.2.{36.4, 36.5, 37.0, 37.1, 37.2, 37.3} bundled release (`PGEN-SV-EXH-PROOF-{0085, 0086, 0087, 0088, 0089, 0090}`, 2026-05-25/26): combined landing of (`.36.4`) engine memoization-delta replay + (`.36.5`) bootstrap `**` → FlattenSpread fix + (`.37.0/.37.1/.37.2`) H2 SV-stdlib auto-load (`process`/`semaphore`/`mailbox` as built-in classes via `parser_libs/sv_<profile>_std/`) + (`.37.3`) SV grammar `Class::member` as primary expression + lvalue (`primary_hier_scope_prefix` + `variable_lvalue_scope` + `nonrange_variable_lvalue` all gain a `class_scope` branch). uvm_pkg furthest_position arc 162162 → 866292 (~5.3× deeper than `.35.1`'s starting point). Corpus binary 10/14 stable (residual = uvm × 4). Release bump, schema stays 3 (strictly-more-permissive throughout).

> **Why a bundle.** `.36.4 / .36.5 / .37.2` each individually said "release-bump candidate, deferred to a combined landing"; `.37.0` was investigation; `.37.1` was data-only; `.37.3` is the natural break that lands them all together with a single 1.0.128 bump. AST shape vocabulary is unchanged across the entire bundle. The contract version moves once. The book changelog entry below documents what changed for consumers.

### 1.0.127 / Contract 1.0.127 — SV-EXH-PROOF.3.3.4.b.6.2.35.1: SV grammar fix — gate `provisional_unscoped_block_class_type` with `@predicate has_fact(type_name, $head.body)` (Slice-64 redux, lands clean now; release bump, schema stays 3)

> Slice-64 (`9375c069`, 2026-05-24) originally added this exact one-line `@predicate` annotation but was reverted (`8eae9eba`) under the (now-retired) "fact persistence" framing. Slice-69 (`.b.6.2.30`, 2026-05-25) re-diagnosed the implicit-type port-list defect using the new `--trace-rules` tool and proved the root cause is **structural**, not persistence: when parsing `function f(string a, b="");`, every gated type-identifier alternative correctly **rejected** the undeclared `b` (`has_fact(type_name, b) → false`); only the **ungated** `provisional_unscoped_block_class_type` accepted it. The semantic store was always working — the grammar was just bypassing it. This slice (`.b.6.2.35.1`) reapplies Slice-64's exact one-line annotation as the first code sub-leaf of the `.b.6.2.35` umbrella that systematically remediates the ungated-identifier-categorisation defect class (1 + 9 + 3 + 3 audit hits across `systemverilog.ebnf`). Under the current engine state (Slice-67's C3-B in place, persistence framing retired by Slice-69), the gate lands without an adjacent producer fix — exactly the outcome Slice-69's evidence predicted. **Fix (one line):** `@predicate: { name: has_fact, args: [type_name, $head.body], phase: post, view: shaped }` added directly above `provisional_unscoped_block_class_type` at `grammars/systemverilog.ebnf:1638`. The sibling `known_unscoped_block_class_type` already carries this exact predicate plus a `lacks_fact_attribute_equals(... typedef)` companion; the `provisional_*` variant differs by now ACCEPTING typedef'd type names where the `known_*` variant rejects them, but both now require the identifier to be a known type. The catch-all-for-unknowns role is gone — but that role was masking real defects. **Test adjustment** (one test, `ast_shape_contract.rs::systemverilog_context_gated_method_chain_handles_negated_and_uvm_shape`): the "uvm-shaped function" case had `uvm_seed_map seed_map;` with `uvm_seed_map` never declared. Pre-gate the catch-all accepted any bare identifier as a type; post-gate undeclared types are correctly rejected per Slice-64's stated semantics ("UnknownGarbage → rejected → forces user to fix their code"). Added `typedef class uvm_seed_map;` to the test input — a real-UVM-style forward declaration — so the gate sees a `type_name` fact for it; test purpose (3-level method-chain parsing) preserved. **Release bump, NO schema bump** — AST shape vocabulary unchanged (`{head, params, scope_chain}` shape is the same; only the acceptance criterion changes). This is a **behaviour-tightening** correctness fix (a previously-accepted incorrect parse path — undeclared types — is now correctly rejected per LRM); consumers whose inputs accidentally depended on the over-permissive acceptance will see those inputs correctly rejected. Different category from SVPP-0002/REGEX-0083 (strictly-more-permissive) — closest precedent is RGX-0087's `\89`-leading hard-reject (correctness tightening). **NO-WORKAROUNDS HIERARCHY:** this fix is **level 1** — use existing semantic-annotation primitive. No new annotation, store, or engine machinery needed. The proven `.3.3.1`/`.3.3.2`/`.3.3.3` declared-id idiom applied to one more rule. Sign-off-quality at the lowest layer in the fix hierarchy per [[feedback_no_workarounds_fix_hierarchy]]. **Architectural principle reinforced** ([[feedback_grammar_rules_must_consult_store]], user-set 2026-05-25): rules that categorise bare identifiers into tracked categories (type/class/interface/package/...) SHALL consult the store via has_fact / fact_attribute_equals / lacks_fact. This slice is the canonical exemplar; the umbrella `.b.6.2.35.{2..16}` will apply the same principle to the remaining 9 + 3 + 3 audit hits. VERIFIED: iso5/iso4 minimal repros (`module m; function bit f(string a, b="");` / `..., b=""` — the implicit-type 2nd-port-with-default that drove Slice-69's diagnosis) **PASS** (were FAIL); iso1/iso2/iso3/iso6 controls PASS (no regression on minimal repros); forward-declared `typedef class Bar; Bar x; class Bar; endclass`, declared-then-used `class MyClass; endclass MyClass obj;`, plain module, `module m #(parameter int W=8) (input logic [W-1:0] a);` parameterized header all PASS; **SV external corpus 10 PASS / 4 FAIL / 0 TIMEOUT** — no regression on the 10 known-PASS files (scr1 ×4, friscv ×4, veer ×2 with bootstrap-chained `--lib-in`); residual 4 FAIL = uvm_pkg ×{2017,2023} + uvm_compat_pkg ×{2017,2023}; **uvm_pkg deep-position advance: surface_position 113637 (F1 PEG-furthest-position-tooling-artefact, unchanged) but Slice-59's `furthest_position` tracker now shows 162162 out of ~180K** (pre-fix tracked deepest was 19378 at Slice-54 — the parser is now reaching ~90% of uvm_pkg before backtracking exhaustion; ~8.4× deeper); uvm_compat_pkg furthest_position 114993 → 116752 (+1759); **lib no-features 548/548 PASS; lib --features generated_parsers 609/609 PASS**; SV shape-contract GREEN; **regex broader corpus / RGX conformance ✅ 44/0** (non-semantic grammar, unaffected by the SV-only change). The H1 cross-file-import class (uvm-extends-chain) is a separate fix path (H1-α library-import chain or H1-β import-emits-permissive-fact) and is what gates uvm_pkg moving from FAIL to PASS; this slice clears the structural defect that was nibbling at every implicit-type-port construct in uvm-style code.

### 1.0.126 / Contract 1.0.126 — SV-EXH-PROOF.3.3.4.b.3: parser-agnostic ENGINE Layer 0 — unified quantifier engine; uniform per-iteration atomicity for `?` / `*` / `+` / `{N}` / `{N,M}` / `{N,}` / `{,M}` (release bump, schema stays 3)

> **Layer 0** — symmetric, atomic-at-quantifier-boundary semantics across every repetition operator in EBNF. The prior codegen in `rust/src/ast_pipeline/ast_based_generator.rs::generate_quantified_logic` (and `ast_code_generator.rs::generate_quantified_content`) special-cased exactly three operators (`*`, `+`, `?`) with three independently-written code paths, with an asymmetric defect: the FIRST iteration of `+` was emitted INLINE without `try_parse` wrapping. Bounded forms (`{N}` / `{N,M}` / `{N,}` / `{,M}`) declared in `grammars/ebnf.ebnf:149-189` were never implemented — the codegen's `_ => Err("Unknown quantifier")` fallthrough. User strategic line (2026-05-21): "After Layer 0, all the repetition in the EBNF shall have the right behavior." **Fix:** new `parse_quantifier_bounds(quantifier: &str) -> Option<(usize, Option<usize>)>` helper in `rust/src/ast_pipeline/mod.rs` mapping every surface form to its `(min, max)` bounds (`?` → `(0, Some(1))`, `*` → `(0, None)`, `+` → `(1, None)`, `{N}` → `(N, Some(N))`, `{N,M}` → `(N, Some(M))`, `{N,}` → `(N, None)`, `{,M}` → `(0, Some(M))`); 4 unit tests cover happy path + whitespace + invalid forms. Both codegen functions rewritten as ONE `loop` with EVERY iteration wrapped in `try_parse` (per-iteration atomicity is uniform; the prior asymmetric inline-first-iter pattern for `+` is gone). Quantifier-level atomicity layered on top: when `min > 0`, the codegen emits `let quantifier_start_position = parser.position;` before the loop and on min-failure restores `parser.position = quantifier_start_position;` before signalling `Err(Backtrack)` — the quantifier is atomic at its own boundary, so when iteration N+1 fails after N successful iters and N is insufficient for `min`, the partial successes are undone and the quantifier reports failure as if it was never tried. For `min == 0` (`*` / `?`) the bind + check are elided to avoid always-false comparisons and unused-variable warnings. Max-count enforcement (`{N}` / `{N,M}` / `{,M}`) emits `if iteration_count >= #m_lit { break; }`. Zero-length-match guard and SAFETY_LIMIT (10_000) are preserved. **Release bump, NO schema bump** — for `?` / `*` / `+` the unified codegen is BEHAVIOR-EQUIVALENT on success paths and BEHAVIOR-EQUIVALENT-OR-MORE-CORRECT on failure paths (the prior `+` first-iter inline path left the cursor where partial parse stopped before the surrounding `try_parse` rolled it back; the unified path rolls back at the quantifier boundary too). Bounded forms `{N}` / `{N,M}` / `{N,}` / `{,M}` were unparseable pre-fix; Layer 0 makes them parser-agnostic infrastructure available for any future grammar (no current grammar uses them); strictly-more-permissive (SVPP-0002/REGEX-0083 category). VERIFIED: lib tests **465/465** PASS no-features (was 461 → +4 `parse_quantifier_bounds` unit tests); SV smoke (`module m; endmodule` / `if (1) $display("ok");` / else-if chain / for-loop) PASS via fresh release `parseability_probe`; SV shape-contract test GREEN. **RGX broader corpus / conformance ✅ 44/0** via `cargo run --features generated_parsers --release --bin test_runner -- --parser regex` after a `make focus_regex` regen (proves the unified codegen produces a regex parser with identical behavior across the entire conformance suite). USER SEMANTICS, CONFIRMED IMPLEMENTED: "the current value of the cursor shall be saved at the start of an iteration, if the iteration fails we put the cursor back to the position before starting that iteration, and we check if the repetition is fulfilled or not" — per-iter cursor save+restore is done by `try_parse`; the loop continues until either an iteration fails (break) or the max bound is reached; the min check at the end determines whether the quantifier as a whole succeeds. "For `{N}`, the group must match exactly N times; if it doesn't, the cursor is left at the position it had before trying to match `<group>{N}`" — exactly what the `quantifier_start_position` save+restore on min-failure delivers. **SV external corpus `8/14 → 10/14`** — full triage gate fresh 2026-05-21 09:55: **`friscv_rv32i_core ×{2017,2023}` now PASS** (previously categorized as the `.3.3.6` statement-level residual). HONEST ROOT-CAUSE UPDATE — the friscv_rv32i blocker was NOT a separate statement-level grammar defect; it was the prior codegen's asymmetric `+`-first-iter-not-wrapped-in-`try_parse` defect. Layer 0 wraps the first iter of `+` uniformly with every other iter, so the cursor rolls back cleanly on first-iter failure — and friscv_rv32i was triggering exactly that path inside its statement parsing. THIS is why .b.3 is a corpus-mover even though it claimed to be "behavior-equivalent for `*`/`+`/`?` on success paths": the BEHAVIORAL DIFFERENCE on the prior `+` first-iter-FAILURE path was real and friscv_rv32i was hitting it. Residual 4 fails: `uvm_pkg`/`uvm_compat_pkg ×{2017,2023}` STILL fail (uvm_pkg_2017 STILL at byte 113637 — identical to `.b.1` baseline, Layer 0 preserves uvm behavior; blocked by the next LRM-extraction-defect class). The `.3.3.6` task entry can now be CLOSED — friscv_rv32i is PASS without needing a separate grammar fix.

### 1.0.125 / Contract 1.0.125 — SV-EXH-PROOF.3.3.4.b.1: first LRM-extraction-defect fix — `conditional_statement` `[ else statement_or_null ]` now truly optional per IEEE 1800 §A.6.6 (release bump, schema stays 3)

> The opening fix of a longer campaign: the SV grammar was auto-extracted from the IEEE 1800 LRM, and the extractor introduced a class of defects where LRM `[ ]` optional clauses were encoded as mandatory. `conditional_statement` at `grammars/systemverilog.ebnf:1111` was the first case identified — and per the defect-class-by-defect-class strategy, it's a class of one (the only `&X`-positive-lookahead-mandates-optional in the entire 1,448-rule grammar). Root cause verified against the authoritative IEEE 1800-2017 PDF page 1164 AND IEEE 1800-2023 PDF page 1201 — both editions specify `conditional_statement ::= [ unique_priority ] if ( cond_predicate ) statement_or_null { else if ( cond_predicate ) statement_or_null } [ else statement_or_null ]`. Square brackets = OPTIONAL; the EBNF had encoded the trailing else as mandatory via `&kw_else kw_else conditional_else_branch` with no `|` no-else alternative. Fix: helper rule `conditional_else_clause := kw_else_ae050f5b conditional_else_branch -> $2` (passes through the branch); `conditional_statement` rewritten to `( unique_priority )? kw_if lparen cond_predicate rparen statement_or_null ( conditional_else_clause )?` with return annotation `else_body: $7` (was `$9`, now nullable when else absent). Removed dead mis-attached line 1116 (a `| @sample: "if (a) ;" ... !kw_else` alternative the original author reached for as the no-else case via PEG negative lookahead, but the blank line at 1113 parented it to `conditional_else_branch` and made it unreachable). Else-if chain `{ else if (cond) stmt }` continues to be handled by the existing recursive `conditional_else_branch := conditional_statement | statement_or_null` — once the outer else is optional, the chain naturally produces zero-or-more recursively. **Release bump, NO schema bump** — strictly-more-permissive (SVPP-0002/REGEX-0083 category): no_else inputs were 100% unparseable so no AST was ever emitted; previously-parseable inputs are byte-identical; the `else_body` field is the same `conditional_else_branch` ParseNode when present, just additionally nullable when absent. Inventory **2299 → 2300** (+1: the new `conditional_else_clause` helper rule). VERIFIED: minimal failing repro `module m; task t(); if (1) $display("ok"); endtask endmodule` PASSES (was FAIL); pre-existing `if (a) ; else ;` STILL PASSES; else-if chain PASSES; `.3.3.3` minimal repro STILL PASSES; lib `461/461` no-features (+1 from `.3.3.4.a.2`'s baseline) and `516/517` with `--features generated_parsers` (only fail = pre-existing rgx_0077, `.3.3.5`-class). **regex broader corpus / RGX conformance ✅ 44/0** via `make regex_broader_corpus_proof_gate`. SV shape-contract GREEN. **SV external corpus stays 8/14** — scr1 ×4 + friscv_pipeline ×2 + veer_el2_lsu ×2 all still PASS; uvm_pkg_2017 STILL fails at the SAME byte position 113637 post-fix (with parse time 60s→14min as the parser now walks deeper into the 89K-line body, exposing OTHER LRM-extraction defects further in). This proves `conditional_statement` was NOT the blocker for uvm_pkg — the conditional_statement fix is correct on its own merits per the LRM oracle, but more LRM-extraction defects remain on the uvm/uvm_compat/friscv_rv32i paths and will need their own leaves. **Lesson learned and recorded in cross-session memory** (`feedback_verify_rule_correctness_before_runtime_hypotheses`): when a parse failure is tied to a specific rule, the FIRST diagnostic step is to read the rule against the LRM/spec — NOT to form runtime/scope hypotheses. The original `.3.3.4.b` framing as "intra-file scope tracking" cost ~30 min of binary-search effort; the rule-vs-LRM read would have revealed the defect in 30 seconds.

### 1.0.124 / Contract 1.0.124 — SV-EXH-PROOF.3.3.4.a.2: parser-agnostic ENGINE extension — non-negative integer indexed-access (`[N]`) in semantic-annotation rule references (release bump, schema stays 3)

> Companion to `.3.3.4.a.1` (which added dotted property-access `$name.body`). Real authoring concern that `.3.3.4.a.1` only half-addressed: when a rule's shaped output is `{items: [{name, body}, …]}` or `{matrix: [[…], …]}`, directives still couldn't reference array elements directly — there was no `$items[0].name` or `$matrix[0][1]` form. `.3.3.4.a.2` closes that gap with a strict, well-defined subset: dotted property + non-negative integer indexed-access only. NOT full JSONPath (no filters `[?(@.foo)]`, no wildcards `*`, no recursive descent `..`). **Engine (parser-agnostic, both surfaces in lockstep):** (1) `unified_semantic_ast.rs::parse_rule_reference` chain loop now accepts an arbitrary mix of `.<ident>` and `[<digits>]` segments via a `loop { match peek_char() { Some('.') => …, Some('[') => …, _ => break } }` form; strict-bracket policy rolls back on malformed `[…]`. (2) EBNF surface `grammars/semantic_annotation.ebnf::rule_reference_name` regex extended in lockstep — `/([a-zA-Z_][a-zA-Z0-9_]*(\.[a-zA-Z_][a-zA-Z0-9_]*|\[[0-9]+\])*|[0-9]+(\.[a-zA-Z_][a-zA-Z0-9_]*|\[[0-9]+\])*)/`. Bootstrap `semantic_annotation_parser.rs` regenerated. (3) Runtime resolver (`ast_based_generator.rs`): three new free-standing helpers emitted into every generated parser — `lex_semantic_reference_segments_suffix`, `lex_semantic_reference_segments_named`, and `parse_bracketed_index`. The two resolvers (positional + named) use the lexers and dispatch per segment: for shaped-JSON content, `serde_json::Value::get` polymorphically accepts both `&str` and `usize`; for raw-tree content (rules without `->`), a new `find_semantic_indexed_child` companion picks the N'th element of a `Sequence`/`Quantified` (or the wrapped node of an `Alternative` when N==0). **DURABLE NO-DEPTH-LIMIT GUARANTEE extends to mixed dotted+indexed depth**: structurally unbounded at every layer (EBNF `*` quantifier; hand-rolled `loop` with no max-iteration cap; lexer `while !remaining.is_empty()` iterates until input exhausted; resolver iterates over the lexed Vec). Locked by `bootstrap_semantic_indexed_rule_reference_depth_is_structurally_unbounded` test exercising **32 mixed `.<ident>` + 32 `[N]` segments** (64 total) and asserting verbatim retention of both kinds of segments (33 dot-separated chunks, 32 bracketed segments). Pairs with the `.3.3.4.a.1` dotted-only test — together they pin the guarantee across both shapes. **No SV grammar changes** — purely a language-surface expressiveness improvement; the SV directive payloads in 1.0.123 (`$name.body`, `$package.body`) continue to work byte-identically. **Release bump, NO schema bump** — strictly additive language extension; no shape changes anywhere; no annotated rules changed (the new helpers are infrastructure emitted into every generated parser, not annotated grammar rules); SVPP-0002/REGEX-0083 category. VERIFIED: lib 461/461 no-features (was 460 → +1 new indexed-access regression test); 516/517 with `--features generated_parsers` (only fail = pre-existing `rgx_0077` from `.3.3.5`-class); cross-file synthetic repro PASSES with `--lib-in` and FAILS without (the `.3.3.4.a.1` cleanup behaviour preserved); `.3.3.3` minimal repro still PASSES; both depth-unbounded regression tests (dotted-only + mixed). **regex broader corpus / RGX conformance ✅ 44/0** via `make regex_broader_corpus_proof_gate`. SV shape-contract GREEN. **SV external corpus stays 8/14** (verified: triage gate fresh 2026-05-20; veer_el2_lsu ×{2017,2023} still PASS via the existing `$name.body` directives — no SV directive uses `[N]` yet, so the new feature is dead-code in the SV path). Surfaced during this slice: a generator-area test (`generated_parser_runtime_contract_emits_post_predicate_content_flow`) asserted the literal string `current.get(segment)` from the resolver's `for segment in reference.split('.')` loop; my refactor changed the iteration to `for segment in &lexed_segments` (segments are now `&&str`), so the literal became `current.get(*segment)` — assertion updated to track the new shape while staying semantic (`current.get(`). NEXT LEAF (queued): `.3.3.4.b` — uvm self-contained same-file-package path (intra-file scope tracking, not artifact-on-disk; the cross-file MVP-0 model doesn't apply when package + uses share a file).

### 1.0.123 / Contract 1.0.123 — SV-EXH-PROOF.3.3.4.a.1: parser-agnostic ENGINE cleanup — dotted property-access in semantic-annotation rule references; revert of the `.3.3.4.a` `body:$N.body` SV shape workarounds (release bump, schema stays 3)

> Companion to `.3.3.4.a`. `.3.3.4.a` had to carry a SV shape workaround (`body: $4.body` on `package_declaration`, `body: $1.body` on both `package_import_item` branches) because the **directive-payload parser** — the hand-rolled `StructuredSemanticValueParser::parse_rule_reference` in `rust/src/ast_pipeline/unified_semantic_ast.rs` — only accepted single-identifier / single-positional refs (`$name` / `$1`), no `.` allowed. The runtime resolver already walked dotted paths via `resolve_named_semantic_reference` / `parse_semantic_reference_segments` (`ast_based_generator.rs`), splitting on `.` and traversing either the shaped JSON object-key path (rules with `->`) or the raw sub-rule-named descendant tree (rules without `->`). The gap was purely at the bootstrap-parser surface. `.3.3.4.a.1` closes it: `parse_rule_reference` gains a `while peek_char() == Some('.')` chain that accepts an arbitrary number of `.<ident>` segments. DURABLE NO-DEPTH-LIMIT GUARANTEE: depth is structurally unbounded at every layer — EBNF `(\.<ident>)*` regex (strict-math `*`), hand-rolled `while` with no max-iteration cap, resolver `for segment in split('.')` iterator-based walk. Locked by `bootstrap_semantic_dotted_rule_reference_depth_is_structurally_unbounded` test that exercises a 64-segment reference (1 head + 64 `.seg<i>` segments) and asserts the full chain text is retained verbatim — any future proposal to cap depth has to fire this test, justify the cap in its own leaf, or back off (the `feedback_recursion_ceiling_must_bound_real_stack` discipline applied to a non-stack resource). `.3.3.4.a.1` also keeps the EBNF surface consistent: `grammars/semantic_annotation.ebnf::rule_reference_name` regex extended in lockstep so the language definition mirrors the runtime; the bootstrap `semantic_annotation_parser.rs` is regenerated, though that generated parser is the EBNF-language surface for parsing semantic-annotation source — NOT the runtime path for grammar directive payloads, which goes through `unified_semantic_ast.rs::parse_bootstrap`. Keeping both surfaces consistent prevents future drift. **SV grammar revert:** `body: $4.body` removed from `package_declaration`; `body: $1.body` removed from both `package_import_item` branches; the two directives now reference the nested scalars directly — `@export_to_library: {kind:package, name_from:$name.body}` and `@import_from_library: {kind:package, name_from:$package.body}`. **Release bump, NO schema bump** — the `body` fields were a one-slice workaround carrier (added in 1.0.122, removed here); they had no consumer pre-1.0.122 and have none post-`.3.3.4.a.1` (cross-file imports still resolve, verified end-to-end). Versus 1.0.121 this is a no-op shape change. VERIFIED: end-to-end synthetic cross-file repro PASSES with `--lib-in` and FAILS without (the library is still the deciding factor — cleanup is behaviour-preserving); `.3.3.3` minimal repro still PASSES; lib tests **460/460** no-features (was 459 → +1 unbounded-depth regression test) and **515/516** with `--features generated_parsers` (only fail is the pre-existing `rgx_0077` confirmed at `.3.3.4.a` via decisive baseline → `.3.3.5`-class). **regex broader corpus / RGX conformance ✅ 44/0** via `make regex_broader_corpus_proof_gate`. SV shape-contract GREEN. **SV external corpus stays `8/14`** — `veer_el2_lsu` ×{2017,2023} still PASS via the new dotted-refs path (fresh triage gate 2026-05-20 15:41; identical to `.3.3.4.a`'s shape-workaround result). NEXT LEAF: `.3.3.4.a.2` will extend the same surfaces with `[<digits>]` indexed access (`$items[0].name`, `$matrix[0][1]`, mixed `$a.b[0].c[1].d`) under the same unbounded-depth guarantee, own leaf, separate commit.

### 1.0.122 / Contract 1.0.122 — SV-EXH-PROOF.3.3.4.a: parser-agnostic ENGINE feature — per-compilation-artifact library (MVP-0); cross-file `import pkg::*` type-name visibility; `veer_el2_lsu` ×{2017,2023} now parses (release bump, schema stays 3)

> **A second deliberate parser-agnostic ENGINE feature in the SV-EXH-PROOF tree** — and like `.3.3.3` it benefits every grammar. PGEN parsed single SV files in isolation, but real HDL is multi-file: `el2_lsu.sv` does `import el2_pkg::*;` to reference types declared in `el2_def.sv` (a separate file). The use-site `@predicate has_fact(type_name, X)` correctly fires (`.3.3.3` made it reliable) but correctly evaluates false because the type-emitting file was never parsed in the same session. The architecturally-correct fix — matching every commercial HDL tool — is **per-file compilation that writes a compact on-disk artifact** containing the exported facts of each scope-creating entity (package/module/interface/…) with **on-demand library lookup** when an importer references that entity. **MVP-0 (this slice)** narrows to packages only, JSON artifacts, user-supplied file order; future increments grow the kind set and add `$unit`/CU semantics under separate leaves. **TWO new parser-agnostic ENGINE annotations:** `@export_to_library: { kind:<static-kind>, name_from:<expr> }` fires on a rule's successful commit (atomic write of the rule's emitted-fact delta to `<lib-dir>/<kind>/<name>.facts.json`); `@import_from_library: { kind:<static-kind>, name_from:<expr> }` fires after the rule's body parses (merges the artifact's facts into the in-progress semantic-runtime transaction's state, so subsequent sibling `has_fact` checks see them). Both go through the `.3.3.3` IIFE so I/O failures `Err`-propagate and the manual restore fires cleanly — same exception-safety invariant. **TWO new parser CLI flags** on `parseability_probe`: `--lib-in DIR` (where artifacts are read) and `--lib-out DIR` (where artifacts are written). Both default off; single-file parses are byte-identical to today. **SV grammar (composite):** `@export_to_library: {kind:package, name_from:$body}` on `package_declaration` (with a new top-level `body: $4.body` shaped field, since the semantic-annotation language only accepts simple `$name` refs, not `$x.y` — the package's scalar name is surfaced at the shape's top); same idiom on `package_import_item` with `@import_from_library: {kind:package, name_from:$body}` and `body: $1.body` for both branches (explicit + wildcard). **Triage gate (composite):** the SV external-corpus triage manifest gains an optional per-case `bootstrap_files: [...]` ordered array (commercial-tool convention — the user supplies the transitive-dependency order); the gate preprocesses + parses each bootstrap file with `--lib-out <per-case-lib-dir>` THEN parses the main case with `--lib-in <per-case-lib-dir>`. `veer_el2_lsu` declares `bootstrap_files: ["…/include/el2_def.sv"]`. **Release bump, NO schema bump** — the two new top-level `body` fields are additive on shapes whose cross-file form was unparseable (no consumer had a realized AST to depend on); previously-parseable single-file inputs are byte-identical (success paths unchanged); only previously-erroring cross-file inputs now succeed (strictly-more-permissive, SVPP-0002/REGEX-0083 category). VERIFIED: end-to-end synthetic repro (`my_pkg` with two typedefs → artifact written; module that imports `my_pkg::*` and uses the types PASSES with `--lib-in`, FAILS without — proves the library path is the deciding factor); minimal `.3.3.3` repro `module m; typedef int my_t; my_t [3:0] x; endmodule` STILL PASSES; lib tests `514/515 PASS` (the only fail — `regex_parser_pgen_rgx_0077_quoted_run_quantified_pieces_flat_in_concatenation` — is confirmed PRE-EXISTING via decisive baseline: `git stash`ed all MVP-0 sources, rebuilt baseline driver from HEAD, regenerated regex+bootstrap parsers, SAME failure → tracked as `.3.3.5`); 6 new directive-parser unit tests + 4 library-module unit tests all GREEN. **regex broader corpus / RGX conformance ✅ 44/0** (the critical downstream is unaffected — non-semantic grammars hit the fast-path and the library code is dead-code-eliminated when neither annotation appears). **SV external corpus `6/14 → 8/14`** — `veer_el2_lsu` ×{2017,2023} now PASS via the bootstrap-files+library-artifact path, exactly as projected. Residual 6 fails: `uvm_pkg`/`uvm_compat_pkg` ×{2017,2023} are SELF-CONTAINED files (no separate `import` declarations the way veer does) — they fail at a deeper `.3.3.3` use-site `known_unscoped_*` residual; `friscv_rv32i_core` ×{2017,2023} is statement-level (`.3.3.6`).

### 1.0.121 / Contract 1.0.121 — SV-EXH-PROOF.3.3.3: parser-agnostic ENGINE exception-safety fix (`?`-bypasses-cleanup in `with_semantic_runtime_rule_transaction`) + SV grammar wrapper (`checked_type_identifier`); class/typedef/parameter use-site `has_fact` now works (release bump, schema stays 3)

> **The first deliberate ENGINE change in the SV-EXH-PROOF tree** — and it is parser-agnostic, benefiting every grammar. Root cause was a classic Rust `?`-bypasses-cleanup bug in the generator-emitted `with_semantic_runtime_rule_transaction` (`rust/src/ast_pipeline/ast_based_generator.rs`): it `std::mem::take`-s `self.semantic_runtime_state` (leaving `Default == new()` valid-but-EMPTY) and then performs `?`-fallible calls (`f(self)?`, `apply_semantic_runtime_effect_directive?`, `resolve_semantic_predicate_spec_against_content?`); a `?` early-return propagated out of the ENTIRE function, JUMPING OVER the trailing `if result.is_err() { self.semantic_runtime_state = original }` restore — so `self.semantic_runtime_state` was left EMPTY, silently destroying every fact emitted by prior COMMITTED sibling rules (e.g. a typedef's `type_name` fact). Every later sibling `@predicate has_fact(…)` then saw nothing. Definitively SEMTRACE-pinned (`RESTORE` fired **0×** across 1664 trace lines while `took post-body state (self.state→EMPTY)` fired **9×**) after three prior fix hypotheses (try_parse semantic checkpoint, grammar wrapper alone, memoization-cache) were each implemented, tested, and disproven. **FIX (engine, in the generator, all parsers benefit):** wrap the fallible body in an immediately-invoked closure ("try-block" emulation) so every `?` returns into `result` and the restore runs on every non-commit exit. Zero `unsafe`; zero behaviour change on the success path. (RAII Drop-guard considered for strict panic-safety; would need `*mut`+`unsafe` due to `&self`-method borrow conflicts; deferred since IIFE+Default-validity is already panic-robust and the parser never `catch_unwind`s mid-parse.) **FIX (SV grammar composite half):** added `checked_type_identifier := type_identifier -> {body:$1.body}` + `@predicate has_fact args:[type_name,$body] phase:post`, routed `known_unscoped_block/data_type_identifier` through it — the proven `.3.3.1`/`.3.3.2` declared-id idiom (required because a sub-rule-name predicate ref on a `->`-bearing rule does NOT resolve and dotted predicate args are `@predicate`-compiler-rejected). **Release bump, NO schema bump** — declarations whose post-predicate emit/restore was leaked were silently unparseable, so no such AST was ever emitted; previously-parseable inputs byte-identical (success paths unchanged); only previously-erroring inputs now succeed (strictly-more-permissive, SVPP-0002/REGEX-0083 category). VERIFIED: minimal repro `module m; typedef int my_t; my_t [3:0] x;` PASSES (unfixable through three prior attempts); SEMTRACE confirmed RESTORE fires 110× (was 0); use-site `has_fact[type_name, my_t]` resolves and returns true; class/typedef/let/`#(parameter…)`/localparam/builtin-dim no-regression (memo ON, normal config). Inventory **2297 → 2299** (+2: the new wrapper rule's `@predicate` + return-shape). SV shape-contract **GREEN**. **regex broader corpus / RGX conformance ✅ 44/0** (the critical downstream is unaffected — non-semantic grammars hit the fast-path and the IIFE region is dead code there). **external-corpus parse stays `6/14`** — `.3.3.3` is the **foundation engine fix** (the `?`-bypass state-leak was masking every downstream semantic-fact-based fix); the remaining 8 cases are blocked by DISTINCT defects tracked separately: `.3.3.4` (cross-package `import pkg::*` type-name visibility — the actual corpus-mover, ~6 cases) and `.3.3.6` (statement-level residual, friscv_rv32i ×2). Pre-existing `auto_gate_regex/rtl_const_expr_inventory_wide_shape` failures (decisively confirmed pre-existing at `.3.3.2` via git-stash baseline check) tracked as `.3.3.5`.

### 1.0.120 / Contract 1.0.120 — SV-EXH-PROOF.3.3.2: package declaration directive fixed (`declared_package_identifier`); `package … endpackage` now parses (release bump, schema stays 3)

> `package_declaration` was the **only** declaration not using the proven `declared_X_identifier := X_identifier -> {body:$1.body}` + `@emit_fact {name:$body}` idiom (every other declared-id — class/typedef/parameter/etc. — has its `declared_X_identifier`). It put `@emit_fact: { kind: package_name, name: $package_identifier }` directly on the big multi-element `package_declaration` rule, and `package_identifier` occurs twice (the declaration name + the trailing `endpackage : label`), so the named reference resolved to the `{body:…}` object / was unresolvable ⇒ *"Semantic runtime could not resolve fact name"* — **package declarations unparseable**. `PGEN-SV-EXH-PROOF-0023` adds `declared_package_identifier := package_identifier -> { body: $1.body }` with `@emit_fact: { kind: package_name, name: $body }` (mirrors `declared_class_identifier`; **emit-only — package's original directive had NO `@predicate has_fact` since a package is not a type; that behavior is preserved**) and routes the declaration-site through it; the trailing `( colon package_identifier )?` stays the plain label use-site (exactly as `class`/`module` do `( colon class_identifier )?`). **Grammar-only fix — the engine is untouched.** **Release bump, NO schema bump** — package declarations were unparseable so no such AST was ever emitted; previously-parseable inputs byte-identical; strictly-more-permissive (SVPP-0002/REGEX-0083 category). SV shape-contract **GREEN** (samples=3 aligned=3 drift=0 regression_lock=0). `package pp; endpackage`, `package pp; endpackage : pp`, `package pp; localparam int W=8; endpackage` now parse; class/typedef/localparam/let/`.3.3.1` no-regression. **external-corpus parse stays `6/14`** — honest: the minimal package repro now parses (was 100% unparseable), but `uvm_pkg`/`uvm_compat_pkg` ×{2017,2023} fail *deeper* at the `.3.3.3` use-site `known_unscoped_*` type-id defect (~`118257`) — `.3.3.2` peeled one onion layer (forward progress; the aggregate count is gated by the deeper defect). Evidence refined the categorization: `.3.3.3` (use-site type-id resolution) is now the dominant residual (veer + uvm ×4 + uvm_compat ×2).

### 1.0.119 / Contract 1.0.119 — SV-EXH-PROOF.3.3.1: declaration-site identifier directive fixed (`non_keyword_identifier`); class/package/typedef/parameter/localparam declarations now parse (release bump, schema stays 3)

> `non_keyword_identifier` is the foundation rule for every declaration-site name (class / package / typedef / parameter / localparam, via `declaration_identifier`). It returned the raw `-> $2` value, which the positional-extraction codegen leaves as an `Alternative`-wrapped node (implicit-passthrough codegen unwraps it). So every `declared_*_identifier -> {body:$1.body}` `@emit_fact`/`@predicate has_fact` directive routed through it failed *"Semantic runtime could not resolve fact name"*, making **all such declarations 100% unparseable** — a pre-existing defect masked by the `.3.2` number bug (the parser never reached declarations before). `PGEN-SV-EXH-PROOF-0022` restores the engine-proven object form `non_keyword_identifier := !reserved_non_keyword_identifier identifier -> {body: $2.body}` (mirrors `identifier := simple_identifier -> {body:$1}`). **Grammar-only fix — the codegen/runtime engine is untouched** (it is stable; the fix lives entirely in the EBNF). **Release bump, NO schema bump** — those declarations were 100% unparseable so no such AST was ever emitted; previously-parseable inputs byte-identical; only previously-erroring declarations now succeed (strictly-more-permissive — SVPP-0002/REGEX-0083 category). SV shape-contract **GREEN** (samples=3 aligned=3 drift=0 regression_lock=0 — the rule's value is consumed internally by directives, the emitted AST envelope is unchanged); annotation inventory **unchanged** (`-> $2` → `-> {body:$2.body}` is a shape change of an already-counted entry). `.3.2` number oracle (13/13) + `.3.1` declared-id family no-regression (interface/program now parse too). **external-corpus parse `4/14 → 6/14`** (`friscv_pipeline` ×{2017,2023} now fully parse). Residual uvm/friscv_rv32i/veer categorized into sub-leaves `.3.3.2` (package `@emit_fact` name resolution) / `.3.3.3` (use-site `known_unscoped_*` type-id resolution) / `.3.3.4` (statement-level `conditional_statement`/labeled `seq_block`).

### 1.0.118 / Contract 1.0.118 — SV-EXH-PROOF.3.2: number-rule decomposition restored to the generator's clean IEEE-1800 lexical form (release bump, schema stays 3)

> The 115-slice campaign + POST-SV-AUDIT had decomposed the number rules into a per-digit-token tree; the generator's blanket `\b`-append on every single-char alnum token (digits, base letters `h/b/d/o/s`, `x/z`) + a `__SV_RULE__`-sentinel-mangled `_` separator made **every** multi-digit / underscore / sized / based SV number unparseable — the external-corpus `0/14` root cause. `PGEN-SV-EXH-PROOF-0021` restores `integral_number`/`real_number`/`unsigned_number`/`decimal_number`/`binary_number`/`octal_number`/`hex_number`/`size`/`fixed_point_number` to the generator's clean single-regex IEEE-1800 (A.8.7) form (sized-first for PEG, size optional for unsized-based), keeping the typed top `number := real_number -> {kind:"real",body:$1} | integral_number -> {kind:"integral",body:$1}` (now clean Terminals ⇒ first realized number shape = flat `{kind,body:"<number>"}`). **Release bump, NO schema bump** — numbers were 100% unparseable so no number AST was ever emitted (no prior realized shape / no consumer); previously-parseable inputs byte-identical; only previously-erroring numbers now succeed (strictly-more-permissive — SVPP-0002/REGEX-0083 category). Full LRM number-form oracle (decimal/`1_000`/`8'h09`/`4'b1010`/`16'd42`/`8'o17`/`'hFF`/`8'shFF`/`4'b1x0z`/`12'hDE_AD`/`1.5`/`1.0e6`/`1e-3`/veer `2294'h…`) + `.3.1` 13-family no-regression + SV shape-contract GREEN + SV lib 8/0. **external-corpus parse `0/14 → 4/14`** (scr1 family now fully parses — first non-zero corpus parse since the SV-EXH-PROOF tree began); residual uvm/friscv/veer (10) pinned as `.3.3`. Resolves the long-DEFERRED `*_value`/`unsigned_number` shaping note.

### 1.0.117 / Contract 1.0.117 — POST-SV-AUDIT.2.4b: 11 structured-per-iteration Category-A misuses corrected via factored record rules (schema 2 → 3)

`PGEN-POST-SV-AUDIT-0006`, leaf POST-SV-AUDIT.2.4b (2026-05-17).
**AST-dump schema bumped `2 → 3`** (the affected list / branch shapes
change in a consumer-visible way).

The 11 structured-per-iteration Category-A rules in
`grammars/systemverilog.ebnf` were corrected. Each repeated
**multi-field** unit was factored into a **new annotated record rule**
and the list / branch became an extraction-spread over it. Field names
of the prior `first` record are **preserved**.

- 5 `list_of_*_identifiers`: `{first: {name, dims[, init]}, rest}`
  (raw `[[comma, id, [dim…]], …]` envelope) → clean flat record array
  via new `interface_identifier_decl` / `port_identifier_decl` /
  `variable_identifier_decl` (`-> {name, dims}`) and
  `tf_variable_identifier_decl` / `variable_port_identifier_decl`
  (`-> {name, dims, init}`); list now `[$1, $2::2*]`.
- `let_` / `property_` / `sequence_list_of_arguments`: new
  `*_named_arg := dot identifier lparen ( <x>_actual_arg )? rparen
  -> {name, value}`; **mixed** branch `{kind:"mixed", head,
  ordered_tail:[…], named_tail:[…]}`; **named_only** branch
  `{kind:"named_only", items:[…]}` (was `{kind:"named_only",
  first_name, first_value, rest}`).
- `parameter_port_list` type_only: `{kind:"type_only", first, rest}` →
  `{kind:"type_only", items:[$4, $5::3*]}` (clean `[type_assignment]`).
- `assignment_pattern` named (both `pattern_sv_2017` and
  `pattern_sv_2023` occurrences): new shared `assignment_pattern_entry
  := member_identifier colon pattern -> {name, pattern}`; branch
  `{kind:"named", entries:[$3, $4::2*]}` (was `{kind:"named",
  entries:{first:{name,pattern},rest}}`).

Annotation inventory: **2299** (was 2290, **+9 — a DELIBERATE count
change, NOT "unchanged"**). The 9 new factored record rules
(`interface_identifier_decl`, `port_identifier_decl`,
`variable_identifier_decl`, `tf_variable_identifier_decl`,
`variable_port_identifier_decl`, `let_named_arg`, `property_named_arg`,
`sequence_named_arg`, `assignment_pattern_entry`) **are annotated** —
unlike the pure-Cat-A `[$1, $2::2*]` rewrites that add no annotations;
this is the same kind of deliberate +N as SVPP-0001's `pp_if_keyword`.
**1008** distinct annotated rules (was 999, +9). Same accept set. The
reachable `list_of_*_identifiers` path is probe-verified on
`module m; wire a, b, c; logic x, y, z; endmodule` (clean `{name,
dims, init}` record list; 0 `<invalid_sequence_access>`; 0 raw `[[],
","]` separator-envelope leaks). The `*_list_of_arguments`,
`parameter_port_list` type_only, and `assignment_pattern` named
branches are likely unreachable via the strict SV root (pre-existing,
out-of-scope coverage limitation) — defensively-correct-by-construction
(identical proven idiom), **not** claimed as a fresh probe and **NOT a
bug-ledger entry** (clean Category-A, no `<invalid_sequence_access>`
corruption). Full detail: contract §
"AST-Shape Corrections — 1.0.117 (POST-SV-AUDIT)" and
`docs/POST_SV_AUDIT_LEDGER.md`.

### 1.0.116 / Contract 1.0.116 — POST-SV-AUDIT.2.4a: net_alias Cat-A raw-envelope correction + 5-number-rule defensive structural fix (schema 1 → 2)

`PGEN-POST-SV-AUDIT-0005`, leaf POST-SV-AUDIT.2.4a (2026-05-17).
**AST-dump schema bumped `1 → 2`** (the reachable, consumer-visible
`net_alias` shape change drives it).

- `net_alias` Category-A raw-envelope misuse corrected:
  `{first, second, rest}` (raw `[[assign, net_lvalue], …]` `rest`
  envelope) → clean flat `{lvalues: [$2, $4, $5::2*]}` list. Stays
  `return_object`, new `normalized_text` only. Probe-verified on
  `module m; wire a, b, c; alias a = b = c; endmodule` →
  `{"lvalues":[{…a…},{…b…},{…c…}]}`.
- Defensive structural correction of 5 number rules
  (`unsigned_number` / `non_zero_unsigned_number` / `binary_value` /
  `octal_value` / `hex_value`): the inline-alternation iteration lead
  `( kw_sv_rule_c82a06f6 | <digit> )` lifted into new **un-annotated**
  named `*_tail` rules so bare `$2` binds cleanly; the
  `{first, rest}` annotation text is **unchanged**. The corruption is
  **structurally present but NOT consumer-reproducible** (SV
  `systemverilog_file` root rejects every numeric-bearing top-level
  construct in all profiles — pre-existing, out-of-scope root coverage
  limitation), so this is a **defensive/latent** fix and is **NOT a
  bug-ledger entry**.

Annotation inventory: **2290** (UNCHANGED — `net_alias` is a text-only
`normalized_text` change; the number-rule annotation text is unchanged;
the 5 `*_tail` rules are un-annotated). **999** distinct rules
(UNCHANGED). Same accept set. Full detail: contract §
"AST-Shape Corrections — 1.0.116 (POST-SV-AUDIT)" and
`docs/POST_SV_AUDIT_LEDGER.md`.

### 1.0.115 / Contract 1.0.115 — SV-Slice-115 batch: Pattern-B ps_type_identifier_sv_2017/2023 typed (2 rules / 2 annotations)

Annotation inventory: **2290** (was 2288, +2). Same accept set.

### 1.0.114 / Contract 1.0.114 — SV-Slice-114 batch: Pattern-A number-value sequence rules typed (5 rules / 5 annotations)

Annotation inventory: **2288** (was 2283, +5). Same accept set.

### 1.0.113 / Contract 1.0.113 — SV-Slice-113 batch: method_call_receiver_sv_2017/2023 per-branch typed (2 rules / 26 annotations after codegen drops 2)

Annotation inventory: **2256** (was 2231, +25). Same accept set.

### 1.0.112 / Contract 1.0.112 — SV-Slice-112 batch: hierarchical_tf_identifier + ansi_port_declaration typed (2 rules / 4 annotations after codegen drops)

Annotation inventory: **2231** (was 2227, +4). Same accept set.

### 1.0.111 / Contract 1.0.111 — SV-Slice-111 batch: delay_sv_2017 + delay_sv_2023 typed (2 rules / 8 annotations)

Annotation inventory: **2227** (was 2219, +8). Same accept set.

### 1.0.110 / Contract 1.0.110 — SV-Slice-110 batch: drive_strength + init_val + scalar_constant typed (3 rules / 26 annotations)

Annotation inventory: **2219** (was 2193, +26). Same accept set.

### 1.0.109 / Contract 1.0.109 — SV-Slice-109 batch: pulldown_strength + pullup_strength + net_type typed (3 rules / 18 annotations)

Annotation inventory: **2193** (was 2175, +18). Same accept set.

### 1.0.108 / Contract 1.0.108 — SV-Slice-108 batch: duplicate-branch leaf rules typed (3 rules / 11 annotations)

Annotation inventory: **2175** (was 2164, +11). Same accept set.

### 1.0.107 / Contract 1.0.107 — SV-Slice-107: provisional_unscoped_block_class_type typed (1 rule / 1 annotation)

Annotation inventory: **2164** (was 2163, +1). Same accept set.

### 1.0.106 / Contract 1.0.106 — SV-Slice-106 batch: type-identifier + block_class_type chain rules typed (5 rules / 4 annotations)

Annotation inventory: **2163** (was 2159, +4). Same accept set.

### 1.0.105 / Contract 1.0.105 — SV-Slice-105 batch: scoped_X passthrough identifier rules typed (11 rules / 11 annotations)

Annotation inventory: **2159** (was 2148, +11). Same accept set.

### 1.0.104 / Contract 1.0.104 — SV-Slice-104 batch: identifier-routing wrappers typed (15 rules / 35 annotations)

Annotation inventory: **2148** (was 2113, +35). Same accept set.

### 1.0.103 / Contract 1.0.103 — SV-Slice-103 batch: operator/punctuation leaves typed (69 rules / 69 annotations)

Annotation inventory: **2113** (was 2044, +69). Same accept set.

### 1.0.102 / Contract 1.0.102 — SV-Slice-102 batch: number-leaf family typed — 12 Or rules with per-branch kind discriminators (63 annotations)

Annotation inventory: **2044** (was 1981, +63). Same accept set.

### 1.0.101 / Contract 1.0.101 — SV-Slice-101 batch: comment_only + timing_check_limit + trans_item + remaining t*_path + type wrappers + variable_port typed (15 rules / 15 annotations)

Annotation inventory: **1981** (was 1966, +15). Same accept set.

### 1.0.100 / Contract 1.0.100 — SV-Slice-100 batch: sign + statement_item + structure_pattern_key + t*_path_delay_expression + tf_port + threshold + timecheck + timeunits wrappers typed (16 rules / 26 annotations) — crosses 100-slice milestone

Annotation inventory: **1966** (was 1940, +26). **Crosses 100-slice milestone.** Same accept set.

### 1.0.99 / Contract 1.0.99 — SV-Slice-99 batch: package_scope + parameter_declaration + parameter_value_assignment + parameter_override + pattern + port_declaration + program_generate_item + property_expr + pulse_control + randomize + ref_declaration + select_condition wrappers typed (25 rules / 50 annotations)

Annotation inventory: **1940** (was 1890, +50). Same accept set.

### 1.0.98 / Contract 1.0.98 — SV-Slice-98 batch: default_skew + dynamic_override + forward_type + module_instantiation + operator_assignment + package_or_generate wrappers typed (24 rules / 39 annotations)

Annotation inventory: **1890** (was 1851, +39). Same accept set.

### 1.0.97 / Contract 1.0.97 — SV-Slice-97 batch: final/initial_construct + method_call internals + identifier_list + interface_instantiation + module_common_item wrappers typed (19 rules / 26 annotations)

Annotation inventory: **1851** (was 1825, +26). Same accept set.

### 1.0.96 / Contract 1.0.96 — SV-Slice-96 batch: constraint/covergroup/data_declaration + design + dist + elaboration + event wrappers typed (17 rules / 28 annotations)

Annotation inventory: **1825** (was 1797, +28). Same accept set.

### 1.0.95 / Contract 1.0.95 — SV-Slice-95 batch: sv_multi_entry_root + comment_only + bit_select + case + clocking + constant_* wrappers typed (12 rules / 18 annotations)

Annotation inventory: **1797** (was 1779, +18). Same accept set.

### 1.0.94 / Contract 1.0.94 — SV-Slice-94 batch: dimension family + integer_covergroup_expression typed (8 rules / 15 annotations)

Annotation inventory: **1779** (was 1764, +15). Same accept set.

### 1.0.93 / Contract 1.0.93 — SV-Slice-93 batch: anonymous_program_item + assignment_pattern + array + block_event + built_in_method + class_item wrappers typed (16 rules / 40 annotations)

Annotation inventory: **1764** (was 1724, +40). Same accept set.

### 1.0.92 / Contract 1.0.92 — SV-Slice-92 batch: terminal + cross_set + weight_specification + passthroughs typed (8 rules / 8 annotations)

Annotation inventory: **1724** (was 1716, +8). Same accept set.

### 1.0.91 / Contract 1.0.91 — SV-Slice-91 batch: rs_ + value_range wrappers typed (9 rules / 16 annotations)

Annotation inventory: **1716** (was 1700, +16). Same accept set.

### 1.0.90 / Contract 1.0.90 — SV-Slice-90 batch: production + udp_declaration + range_list wrappers typed (4 rules / 5 annotations)

Annotation inventory: **1700** (was 1695, +5). Same accept set.

### 1.0.89 / Contract 1.0.89 — SV-Slice-89 batch: profile-router wrappers typed (9 rules / 18 annotations)

Annotation inventory: **1695** (was 1677, +18). Same accept set.

### 1.0.88 / Contract 1.0.88 — SV-Slice-88 batch: constant_primary + primary wrappers typed (2 rules / 4 annotations)

Annotation inventory: **1677** (was 1673, +4). Same accept set.

### 1.0.87 / Contract 1.0.87 — SV-Slice-87 batch: module_path_operators + level_input_list typed (4 rules / 20 annotations)

Annotation inventory: **1673** (was 1653, +20). Same accept set.

### 1.0.86 / Contract 1.0.86 — SV-Slice-86 batch: let_declaration + final/initial specifiers + named_port_connection + nonconsec_rep + time_unit typed (6 rules / 13 annotations)

Annotation inventory: **1653** (was 1640, +13). Same accept set.

### 1.0.85 / Contract 1.0.85 — SV-Slice-85 batch: type_declaration + type_identifier_or_class_type + type_reference + net_alias + net_declaration typed (7 rules / 19 annotations)

Annotation inventory: **1640** (was 1621, +19). Same accept set.

### 1.0.84 / Contract 1.0.84 — SV-Slice-84 batch: net_assignment + param_expression + struct_union + inst_name typed (5 rules / 10 annotations)

Annotation inventory: **1621** (was 1611, +10). Same accept set.

### 1.0.83 / Contract 1.0.83 — SV-Slice-83 batch: block_data_declaration + base_class_type + misc typed (8 rules / 15 annotations)

Annotation inventory: **1611** (was 1596, +15). Same accept set.

### 1.0.82 / Contract 1.0.82 — SV-Slice-82 batch: dynamic_override + incomplete_class + var_data_type + timing leaves typed (10 rules / 14 annotations)

Annotation inventory: **1596** (was 1582, +14). Same accept set.

### 1.0.81 / Contract 1.0.81 — SV-Slice-81 batch: config_rule + library + hierarchical_identifier + severity typed (13 rules / 25 annotations)

Closes LRM A.1.7 config/library walk paths + various leaf rules. Annotation inventory: **1582** (was 1557, +25). Same accept set.

### 1.0.80 / Contract 1.0.80 — SV-Slice-80 batch: boolean_abbrev + repetition + elaboration + repeat_range typed (8 rules / 19 annotations)

Closes LRM A.2.10 boolean_abbrev, A.6.5 elaboration_system_task, repeat_range, and repetition leaves. Annotation inventory: **1557** (was 1538, +19). Same accept set.

### 1.0.79 / Contract 1.0.79 — SV-Slice-79 batch: event + local/type parameter + mintypmax + nettype family typed (12 rules / 18 annotations)

Closes event_expression, event_trigger, local_parameter, mintypmax, nettype_declaration, type_assignment + type_parameter_declaration sub-trees. Annotation inventory: **1538** (was 1520, +18). Same accept set.

### 1.0.78 / Contract 1.0.78 — SV-Slice-78 batch: class_constructor wrappers + let + for + named_port + parameter_port typed (21 rules / 48 annotations)

**What changed:** Closes LRM A.1.10 class_constructor wrappers, A.6.8 let, A.6.5 for-loop, A.6.5 named/ordered checker/port connections, A.1.3 parameter_port.

Annotation inventory: **1520** (was 1472, +48). Same accept set.

### 1.0.77 / Contract 1.0.77 — SV-Slice-77 batch: module_path + constraint internals + uniqueness + misc typed (16 rules / 29 annotations)

**What changed:** Closes LRM A.8.3 module_path expression sub-tree, constraint primary scope, cycle_delay_range, extern_tf, inst_clause, solve_before list, and uniqueness_constraint family.

Annotation inventory: **1472** (was 1443, +29). Same accept set.

### 1.0.76 / Contract 1.0.76 — SV-Slice-76 batch: class_scope + method_call + tf_call family typed (14 rules / 27 annotations)

**What changed:** Closes LRM A.8.4 class-scope + method-call + tf-call walk paths.

```ebnf
class_scope_type / class_scope / implicit_class_handle (3 kinds) /
method_call / method_call_initial (5 kinds) / method_call_body (3 kinds) /
method_call_receiver_sv_2017 / method_call_receiver (2 kinds) /
method_call_root (2 kinds) / plain_tf_call_with_args / tf_call_with_args /
tf_call (4 kinds) / class_scoped_call_prefix /
class_scoped_tf_call_with_args / class_scoped_tf_call (2 kinds)
```

DEFERRED: `method_call_receiver_sv_2023` (~14 kinds with parens-grouped-Or sub-expressions — task #38 risk).

Annotation inventory: **1443** (was 1416, +27). Same accept set.

### 1.0.75 / Contract 1.0.75 — SV-Slice-75 batch: net_port_type + I/O declarations + genvar typed (10 rules / 17 annotations)

**What changed:** Closes LRM A.2.1.2 net_port_type / nonansi I/O declarations / LRM A.2.1.3 genvar sub-trees.

```ebnf
genvar_declaration / genvar_expression / inout_declaration /
input_declaration (2 kinds) / output_declaration (2 kinds) /
net_port_header / net_port_type_sv_2017/2023 (3+3 kinds) /
net_port_type (2 kinds) / net_type_declaration_sv_2017
```

DEFERRED: `net_type` (duplicate-branch grammar bug).

Annotation inventory: **1416** (was 1399, +17). Same accept set.

### 1.0.74 / Contract 1.0.74 — SV-Slice-74 batch: dpi + extern_constraint + interface_class + param family typed (18 rules / 32 annotations)

**What changed:** Closes LRM A.2.6 DPI sub-tree, LRM A.1.4 extern_constraint, LRM A.1.9 interface_class, and LRM A.2.1.1 param_assignment sub-trees.

DPI: `dpi_function_import_property` (2 kinds), `dpi_function_proto`, `dpi_import_export` (4 kinds), `dpi_spec_string` (2 kinds), `dpi_task_import_property`, `dpi_task_proto`. Extern constraint: `extern_constraint_declaration_sv_2017/2023`, `extern_constraint_declaration` (2 kinds). Interface class: `interface_class_item` (5 kinds), `interface_class_method`, `interface_class_type`, `interface_port_declaration` (2 kinds), `interface_port_header` (2 kinds). Param: `ordered_parameter_assignment`, `param_assignment_sv_2017/2023` (each 2 kinds), `param_assignment` (2 kinds).

Annotation inventory: **1399** (was 1367, +32). Same accept set.

### 1.0.73 / Contract 1.0.73 — SV-Slice-73 batch: checker family typed (10 rules / 33 annotations)

**What changed:** Closes LRM A.2.2.2 checker-declaration walk path.

```ebnf
checker_declaration                    -> {name, ports, items, end_label}
checker_generate_item_sv_2017/2023     -> 4 kinds each
checker_generate_item                  -> 2 kinds
checker_instantiation                  -> {name, instance, connections}
checker_or_generate_item               -> 7 kinds
checker_or_generate_item_declaration   -> 10 kinds
checker_port_direction                 -> 2 kinds
checker_port_item                      -> {attributes, direction, formal_type, name, dims, default}
checker_port_list                      -> [$1, $2::2*]
```

Annotation inventory: **1367** (was 1334, +33). Same accept set.

### 1.0.72 / Contract 1.0.72 — SV-Slice-72 batch: sequence family typed (16 rules / 39 annotations)

**What changed:** Closes LRM A.2.10 sequence sub-tree referenced from `property_expr.kind == "sequence".body`.

```ebnf
seq_input_list / sequence_abbrev / sequence_actual_arg_sv_2017/2023 (2+3 kinds) /
sequence_actual_arg / sequence_declaration / sequence_expr (12 kinds) /
sequence_formal_type (3 kinds) / sequence_instance / sequence_list_of_arguments (2 kinds) /
sequence_lvar_port_direction (3 kinds bare) / sequence_match_item (3 kinds) /
sequence_method_call / sequence_port_item / sequence_port_list / with_covergroup_expression
```

Annotation inventory: **1334** (was 1295, +39). Same accept set.

### 1.0.71 / Contract 1.0.71 — SV-Slice-71 batch: property_expr family typed (2 rules / 72 annotations)

**What changed:** Closes LRM A.2.10 property expression sub-tree.

`property_expr_sv_2017` and `property_expr_sv_2023` each get 36 kinds covering all property operators (sequence / strong / weak / paren / not / or / and / sequence_dup / implies_unary / sequence_or_assign / if / case / imp_minus / imp_assign / nexttime / nexttime_const / s_nexttime / s_nexttime_const / always / always_range / s_always / s_eventually / eventually / s_eventually_range / until / s_until / until_with / s_until_with / implies_binary / iff / accept_on / reject_on / sync_accept_on / sync_reject_on / instance / clocking).

Annotation inventory: **1295** (was 1223, +72). Same accept set.

### 1.0.70 / Contract 1.0.70 — SV-Slice-70 batch: property family (excluding property_expr) typed (12 rules / 16 annotations)

**What changed:** Closes the property-declaration walk path.

```ebnf
assertion_variable_declaration -> {data_type, items}
let_list_of_arguments          -> 2 kinds (mixed / named_only)
property_actual_arg            -> 2 kinds
property_case_item             -> {expressions, body}
property_declaration           -> {name, ports, declarations, spec, end_label}
property_formal_type           -> 2 kinds
property_instance              -> {name, args}
property_list_of_arguments     -> 2 kinds
property_lvar_port_direction   -> 1 kind bare
property_port_item             -> {attributes, local_direction, formal_type, name, dims, default}
property_port_list             -> [$1, $2::2*]
property_spec                  -> {clocking, disable_iff, body}
```

DEFERRED: `property_expr_sv_2017/2023` (~30 kinds each) — slice 71.

Annotation inventory: **1223** (was 1207, +16). Same accept set.

### 1.0.69 / Contract 1.0.69 — SV-Slice-69 batch: cover_cross + trans + select_expression typed (12 rules / 29 annotations)

**What changed:** Closes LRM A.2.11 cross-cover and trans-list walk paths.

```ebnf
cover_cross               -> {label, items, condition, body}
cross_body_sv_2017/2023   -> 2 kinds each (block / empty)
cross_body                -> 2 kinds (sv_2017 / sv_2023)
cross_body_item_sv_2017/2023 -> 2 kinds each (function_decl / selection_or_option)
cross_body_item           -> 2 kinds
cross_item                -> 2 kinds (cover_point / variable)
trans_list                -> [$2, $4::3*]
trans_range_list          -> 4 kinds (simple / star / implies / assign)
trans_set                 -> [$1, $2::2*]
select_expression         -> 8 kinds (condition / not / and / or / paren / with_matches / cross / cross_set)
```

Annotation inventory: **1207** (was 1178, +29). Same accept set.

### 1.0.68 / Contract 1.0.68 — SV-Slice-68 batch: bins family typed (5 rules / 14 annotations)

**What changed:** Closes LRM A.2.11 bin-declaration sub-tree referenced from `cover_point.bins`.

```ebnf
bins_expression           -> 2 kinds (variable / cover_point)
bins_or_empty             -> 2 kinds (block / empty)
bins_or_options           -> 7 kinds (coverage_option / range_list / cover_point_with / set / trans_list / default / default_sequence)
bins_selection            -> {keyword, name, select, iff}
bins_selection_or_option  -> 2 kinds (option / selection)
```

Annotation inventory: **1178** (was 1164, +14). Same accept set.

### 1.0.67 / Contract 1.0.67 — SV-Slice-67 batch: covergroup declaration + coverage_event family typed (12 rules / 26 annotations)

**What changed:** Closes LRM A.2.11 covergroup-declaration walk path.

```ebnf
bins_keyword                    -> 3 kinds bare (bins / illegal_bins / ignore_bins)
covergroup_declaration_sv_2017  -> {name, ports, event, items, end_label}
covergroup_declaration_sv_2023  -> 2 kinds (single / extends)
coverage_event                  -> 3 kinds (clocking / sample_function / block_event)
coverage_option                 -> 2 kinds (option / type_option)
coverage_spec                   -> 2 kinds (point / cross)
coverage_spec_or_option         -> 2 kinds (spec / option)
cover_point                     -> {label, expression, condition, bins}
covergroup_range_list           -> [$1, $2::2*]
covergroup_value_range_sv_2017  -> 2 kinds (expression / range)
covergroup_value_range_sv_2023  -> 5 kinds (expression / range / dollar_lo / dollar_hi / tolerance)
covergroup_value_range          -> 2 kinds (sv_2017 / sv_2023)
```

DEFERRED: bins_or_empty / bins_or_options / bins_selection / bins_selection_or_option (deep multi-branch — slice 68).

Annotation inventory: **1164** (was 1138, +26). Same accept set.

### 1.0.66 / Contract 1.0.66 — SV-Slice-66 batch: UDP body/entry + udp_instance family typed (9 rules / 19 annotations)

**What changed:** Closes LRM A.5 UDP body/instance walk paths.

```ebnf
combinational_entry  -> {inputs, output}
sequential_entry     -> {inputs, current_state, next_state}
level_symbol         -> 7 kinds bare (0 / 1 / x / X / ? / b / B)
output_symbol        -> 4 kinds bare (0 / 1 / x / X)
next_state           -> 2 kinds (symbol / minus)
udp_instance         -> {name, output, inputs: [$5, $6::2*]}
udp_instantiation    -> {name, drive_strength, delay, instances: [$4, $5::2*]}
udp_port_declaration -> 3 kinds (output / input / reg)
udp_reg_declaration  -> {attributes, name}
```

DEFERRED: `init_val` (duplicate-branch grammar bug, same family as drive_strength).

Annotation inventory: **1138** (was 1119, +19). Same accept set.

### 1.0.65 / Contract 1.0.65 — SV-Slice-65 batch: timing_check internals + scalar_timing_check_condition typed (16 rules / 22 annotations)

**What changed:** Closes the body field of every `system_timing_check.kind` from slice 64.

```ebnf
scalar_timing_check_condition -> 6 kinds (expression / not / eq / case_eq / ne / case_ne)
delayed_data                  -> 2 kinds (simple / with_expr)
delayed_reference             -> 2 kinds (simple / with_expr)
12 sv_dollar_*_timing_check    -> field-typed top-level args + tail slot for nested optional envelope
```

Annotation inventory: **1119** (was 1097, +22). Same accept set.

### 1.0.64 / Contract 1.0.64 — SV-Slice-64 batch: edge + timing_check family typed (10 rules / 37 annotations)

**What changed:** Closes LRM A.7.5.3 / A.7.6 timing check sub-trees (referenced from `specify_item.kind == "system_timing".body`) and the LRM A.5 UDP edge-input descriptors.

```ebnf
edge_descriptor              -> 4 kinds (01 / 10 / z_or_x_first / digit_first)
edge_indicator               -> 2 kinds (pair / symbol)
edge_symbol                  -> 9 kinds bare (r / R / f / F / p / P / n / N / star)
edge_control_specifier       -> {descriptors}
edge_input_list              -> {leading_levels, indicator, trailing_levels}
system_timing_check          -> 12 kinds (setup / hold / setuphold / recovery / removal / recrem / skew / timeskew / fullskew / period / width / nochange)
timing_check_condition       -> 2 kinds (scalar / paren)
timing_check_event           -> {control, descriptor, condition}
timing_check_event_control   -> 4 kinds (posedge / negedge / edge / edge_control)
controlled_timing_check_event-> {control, descriptor, condition}
```

Annotation inventory: **1097** (was 1060, +37). Same accept set.

### 1.0.63 / Contract 1.0.63 — SV-Slice-63 batch: path_declaration family typed (14 rules / 25 annotations)

**What changed:** Closes LRM A.7.2 / A.7.4 path declarations (referenced from `specify_item.kind == "path".body`).

```ebnf
path_declaration                      -> 3 kinds (simple / edge_sensitive / state_dependent)
simple_path_declaration               -> 2 kinds (parallel / full)
edge_sensitive_path_declaration       -> 2 kinds (parallel / full)
state_dependent_path_declaration      -> 3 kinds (if_simple / if_edge_sensitive / ifnone)
parallel_path_description             -> {input, polarity, output}
full_path_description                 -> {inputs, polarity, outputs}
parallel_edge_sensitive_path_description_sv_2017  -> {edge, input, in_polarity, output, out_polarity, data_source}
parallel_edge_sensitive_path_description_sv_2023  -> 2 kinds (with_data_source / simple)
full_edge_sensitive_path_description_sv_2017      -> {edge, inputs, in_polarity, outputs, out_polarity, data_source}
full_edge_sensitive_path_description_sv_2023      -> 2 kinds
path_delay_value                      -> 2 kinds (bare / paren)
list_of_path_delay_expressions        -> {body}
pulsestyle_declaration                -> 2 kinds (onevent / ondetect)
showcancelled_declaration             -> 2 kinds (showcancelled / noshowcancelled)
```

Annotation inventory: **1060** (was 1035, +25). Same accept set.

### 1.0.62 / Contract 1.0.62 — SV-Slice-62 batch: specify family typed (9 rules / 15 annotations)

**What changed:** Closes LRM A.7 specify-block walk path (referenced from `non_port_module_item.kind == "specify".body`) and the LRM A.7.5.1 specparam declaration sub-tree.

```ebnf
specify_block                        -> {items}
specify_item                         -> 5 kinds (specparam / pulsestyle / showcancelled / path / system_timing)
specify_input_terminal_descriptor    -> {name, range}
specify_output_terminal_descriptor   -> {name, range}
specify_terminal_descriptor          -> 2 kinds (input / output)
specparam_assignment                 -> 2 kinds (simple {name, value} / pulse {body})
specparam_declaration                -> {dims, items}
polarity_operator                    -> 2 kinds bare (plus / minus)
```

Annotation inventory: **1035** (was 1020, +15). Same accept set.

### 1.0.61 / Contract 1.0.61 — SV-Slice-61 batch: gate_instantiation family typed (16 rules / 43 annotations — crosses 1000-annotation milestone)

**What changed:** Closes LRM A.3.1 gate instantiation walk path (referenced from `module_or_generate_item.kind == "gate_instantiation".body`).

```ebnf
gate_instantiation_sv_2017/2023  -> 9 kinds each (cmos / enable / mos / n_input / n_output / pass_en / pass / pulldown / pullup)
cmos_switchtype                  -> 2 kinds bare (cmos / rcmos)
mos_switchtype                   -> 4 kinds bare (nmos / pmos / rnmos / rpmos)
n_input_gatetype                 -> 6 kinds bare (and / nand / or / nor / xor / xnor)
n_output_gatetype                -> 2 kinds bare (buf / not)
pass_switchtype                  -> 2 kinds bare (tran / rtran)
name_of_instance                 -> {name, dims}
cmos_switch_instance             -> {name, output, input, ncontrol, pcontrol}
enable_gate_instance             -> {name, output, input, enable}
mos_switch_instance              -> {name, output, input, enable}
n_input_gate_instance            -> {name, output, inputs: [$5, $6::2*]}
n_output_gate_instance           -> {name, outputs: [$3, $4::2*], input}
pass_enable_switch_instance      -> {name, in1, in2, enable}
pass_switch_instance             -> {name, in1, in2}
pull_gate_instance               -> {name, output}
```

DEFERRED: `enable_gatetype` / `pass_en_switchtype` / `pulldown_strength` / `pullup_strength` all have duplicate-branch grammar bug (same family as drive_strength / unique_priority / delay_sv_2017/2023).

Annotation inventory: **1020** (was 977, +43). **Crosses 1000-annotation milestone.** Same accept set.

### 1.0.60 / Contract 1.0.60 — SV-Slice-60 batch: number + literal family typed (10 rules / 19 annotations)

**What changed:** Closes LRM A.8.7 number sub-tree (referenced from `primary_literal.kind == "number" / "time_literal" / "string_literal".body`).

```ebnf
number              -> 2 kinds (real / integral)
integral_number     -> 4 kinds (decimal / octal / binary / hex)
real_number         -> 2 kinds (fixed_point / exponential {mantissa, fraction, sign, exponent})
binary_number       -> {size, base, value}
octal_number        -> {size, base, value}
hex_number          -> {size, base, value}
decimal_number      -> 4 kinds (unsized / sized / x_digit / z_digit)
fixed_point_number  -> {whole, fractional}
time_literal        -> {value, unit}
string_literal      -> 2 kinds (triple_quoted / double_quoted)
```

Annotation inventory: **977** (was 958, +19). Same accept set.

### 1.0.59 / Contract 1.0.59 — SV-Slice-59 batch: always + modport family typed (11 rules / 19 annotations)

**What changed:** Closes LRM A.1.4 always-construct dispatch and LRM A.2.9 modport family.

```ebnf
always_construct        -> {keyword, body}
always_keyword          -> 4 kinds bare (always / always_comb / always_latch / always_ff)
import_export           -> 2 kinds bare (import / export)
modport_simple_ports_declaration -> {direction, port}
modport_clocking_declaration     -> {name}
modport_declaration              -> {items: [$2, $3::2*]}
modport_item                     -> {name, ports: [$3, $4::2*]}
modport_ports_declaration        -> 3 kinds (simple / tf / clocking; each {kind, attributes, body})
modport_simple_port              -> 2 kinds (identifier {name} / explicit {name, expression})
modport_tf_port                  -> 2 kinds (method_prototype {body} / tf_identifier {name})
modport_tf_ports_declaration     -> {import_export, ports: [$2, $3::2*]}
```

All new pure-list patterns use `[$N, $M::2*]` from the start per slice 58 audit conclusion.

**Calibration:** parses minimal_module.sv unchanged plus a 5-line interface-with-modport sample. Annotation inventory: **958** (was 939, +19). Same accept set.

### 1.0.58 / Contract 1.0.58 — SV-Slice-58 audit: horizontal `{first, rest}` → `[$N, $M::2*]` extraction-spread fix across 49 grammar locations

**What changed:** Horizontal correctness audit (not a typing slice). Replace `{first: $N, rest: $M}` (raw `[sep, item]` envelope) with `[$N, $M::2*]` (clean flat array of items, separators dropped) for every Category A rule (pure `X (sep X)*` with single payload per iteration). Annotation count unchanged at **939**.

**Why:** Earlier slices defaulted to `{first, rest}` which exposed `[[sep, item], ...]` to consumers. The annotation language has a first-class extraction-spread operator `$N::M*` (defined in `grammars/return_annotation.ebnf` lines 50-58 and used by the bootstrap parser itself at line 158). For pure-list patterns it gives consumers a flat item array — no envelope walking required. This audit was triggered by user feedback after slice 57's `tf_port_list` was flagged.

**Consumer-visible diff** (e.g. `tf_port_list := tf_port_item ( comma tf_port_item )*`):

```
Before  →  { first: <tf_port_item>, rest: [[<comma>, <tf_port_item>], ...] }
After   →  [ <tf_port_item>, <tf_port_item>, ... ]
```

**Affected rules:** assignment_pattern, attribute_instance, bind_target_instance_list, case_generate_item, case_item, class_constructor_arg_list_sv_2023, concatenation, cond_predicate, constant_concatenation, data_type (enum names), dist_list, let_port_list, list_of_arguments_{mixed,ordered,named}, list_of_checker_port_connections, list_of_clocking_decl_assign, list_of_cross_items (also flattened first/second/rest into a single array), list_of_defparam_assignments, list_of_genvar_identifiers, list_of_net_assignments, list_of_net_decl_assignments, list_of_param_assignments, list_of_parameter_assignments_sv_2017, list_of_parameter_value_assignments_sv_2023, list_of_path_inputs, list_of_path_outputs, list_of_port_connections, list_of_ports, list_of_specparam_assignments, list_of_type_assignments, list_of_udp_port_identifiers, list_of_variable_assignments, list_of_variable_decl_assignments, net_lvalue (concat), open_range_list_sv_2017, package_export_declaration (explicit), package_import_declaration, pattern_sv_2017/2023 (ordered), production_sv_2017, range_list_sv_2023, rs_case_item_sv_2017/2023, rs_production_sv_2023, tf_port_list, udp_declaration_port_list, udp_port_list, variable_lvalue (concat), wait_order_statement (events).

**Deferred to SV-Slice-59** (Category B — multi-payload per iteration, requires helper-rule extractions): constant_expression op-chain, expression operand_chain, list_of_*_identifiers families with dims/init payloads, pattern named-branch entries.

**Deferred to post-campaign holistic review** (Category C — `X X*` with no separator, currently `{first, rest}` works correctly but verbosely).

**Calibration:** parses minimal_module.sv unchanged. Annotation inventory at 939 (unchanged). Same accept set.

### 1.0.57 / Contract 1.0.57 — SV-Slice-57 batch: tf_port + prototypes + lvalue/decl_assignment family typed (12 rules / 23 annotations + 1 new helper rule with 2 annotations)

**What changed:** Closes the LRM A.2.7 task/function port-list family + prototype rules + LRM A.8.1 lvalue family.

```ebnf
tf_port_list                         -> {first, rest}
tf_port_item                         -> {attributes, direction, var_keyword, data_type, port_spec}
tf_port_direction_sv_2017            -> 2 kinds (port_direction / const_ref)
tf_port_direction_sv_2023            -> 2 kinds (port_direction / ref with optional const + static)
function_prototype_sv_2017           -> {return_type, name, ports}
function_prototype_sv_2023           -> {dynamic_override, return_type, name, ports}
task_prototype_sv_2017/2023          -> parallel shape
let_port_item                        -> {attributes, type, name, dims, init}
let_port_list                        -> {first, rest}
net_decl_assignment                  -> {name, dims, init}
variable_decl_assignment             -> 3 kinds (variable / dynamic_array / class)
net_lvalue                           -> 3 kinds (name / concatenation / pattern)
variable_lvalue                      -> 4 kinds (name / concatenation / pattern / streaming_concatenation)
variable_lvalue_scope (NEW)          -> 2 kinds (instance / package_scope)
```

13th use of helper-rule extraction pattern.

**Annotation inventory:** 939 entries (was 914). +25 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.57 / Contract 1.0.57 Highlights".

### 1.0.56 / Contract 1.0.56 — SV-Slice-56 batch: class_constructor_declaration family typed (4 rules / 5 annotations + 1 new helper rule with 2 annotations)

**What changed:** Closes the class constructor declaration walks for both LRM 1800-2017 and 2023 profiles.

```ebnf
class_constructor_arg_sv_2023            -> 2 kinds (tf_port_item / default)
class_constructor_arg_list_sv_2023       -> {first, rest}
class_constructor_declaration_sv_2017    -> {class_scope, ports, decls, super_call, statements, end_label}
class_constructor_declaration_sv_2023    -> parallel shape
class_constructor_super_args (NEW)       -> 2 kinds (args / default)
```

12th use of helper-rule extraction pattern (now used in 13 places total).

**Annotation inventory:** 914 entries (was 907). +7 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.56 / Contract 1.0.56 Highlights".

### 1.0.55 / Contract 1.0.55 — SV-Slice-55 batch: clocking + class_constructor_prototype + edge_identifier + method_prototype typed (10 rules / 22 annotations — crosses 900-annotation milestone)

**What changed:** Closes the LRM A.6.10 clocking declaration sub-tree.

```ebnf
class_constructor_prototype_sv_2017/2023 -> {ports}
clocking_decl_assign                     -> {name, value}
clocking_declaration                     -> {default_keyword, name, event, items, end_label}
clocking_direction                       -> 4 kinds (input / output / input_output / inout)
clocking_event_sv_2017                   -> {body}
clocking_event_sv_2023                   -> 3 kinds (ps / hierarchical / expression)
clocking_item                            -> 3 kinds (default_skew / direction / assertion)
clocking_skew                            -> 2 kinds (edge / delay)
edge_identifier                          -> 3 kinds bare (posedge / negedge / edge)
method_prototype                         -> 2 kinds (task / function)
```

**Annotation inventory:** 907 entries (was 885). +22 in this batch — crosses the 900-annotation milestone.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.55 / Contract 1.0.55 Highlights".

### 1.0.54 / Contract 1.0.54 — SV-Slice-54 batch: delay/event/strength leaves typed (10 rules / 33 annotations)

**What changed:** Closes the LRM A.6.5 / A.6.4 timing-control / event-control / strength leaves.

```ebnf
charge_strength                       -> 3 kinds bare (small / medium / large)
cycle_delay                           -> 3 kinds (number / identifier / expression)
cycle_delay_const_range_expression    -> 2 kinds (range / dollar_hi)
delay_control                         -> 2 kinds (value / mintypmax)
delay_or_event_control                -> 3 kinds (delay / event / repeat)
delay_value                           -> 5 kinds (unsigned_number / real_number / ps_identifier / time_literal / step)
event_control_sv_2017                 -> 5 kinds (event / expression / wildcard / wildcard_alt / sequence)
event_control_sv_2023                 -> 3 kinds (clocking / wildcard / wildcard_paren)
event_expression_primary              -> 3 kinds (expression / sequence / paren)
strength                              -> 4 kinds bare (supply / strong / pull / weak)
```

**DEFERRED:** drive_strength, delay_sv_2017/2023 (grammar duplicate-branch bug), event_expression (parens-Or in Quantified — task #38 risk).

**Annotation inventory:** 885 entries (was 852). +33 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.54 / Contract 1.0.54 Highlights".

### 1.0.53 / Contract 1.0.53 — SV-Slice-53 batch: array/stream/class_new/join leaf cleanup typed (9 rules / 18 annotations)

**What changed:** Closes pervasive leaf rules used across primary / streaming-concat / par_block / dynamic-array contexts.

```ebnf
array_method_name                  -> 5 kinds (method_identifier / unique / and / or / xor)
class_new                          -> 2 kinds (constructor / copy)
dynamic_array_new                  -> {size, init}
empty_unpacked_array_concatenation -> bare {kind}
join_keyword                       -> 3 kinds bare (join / join_any / join_none)
slice_size                         -> 2 kinds (simple_type / constant_expression)
stream_concatenation               -> {body}
stream_expression                  -> {expr, with_clause}
stream_operator                    -> 2 kinds bare (shift_right / shift_left)
```

**Annotation inventory:** 852 entries (was 834). +18 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.53 / Contract 1.0.53 Highlights".

### 1.0.52 / Contract 1.0.52 — SV-Slice-52 batch: simple_type + range/dist family typed (14 rules / 29 annotations)

**What changed:** Closes the simple_type / range_expression / part_select_range / dist_* / range_list / value_range walk paths.

```ebnf
simple_type                    -> 4 kinds (integer / non_integer / ps_type / ps_parameter)
range_expression               -> 2 kinds (expression / part_select_range)
part_select_range              -> 2 kinds (range / indexed_range)
constant_part_select_range     -> 2 kinds (parallel)
indexed_range                  -> 2 kinds (plus_indexed [base+:width] / minus_indexed [base-:width])
constant_indexed_range         -> 2 kinds (parallel)
dist_list                      -> {first, rest}
dist_item_sv_2017              -> {value, weight}
dist_item_sv_2023              -> 2 kinds (value / default)
dist_weight                    -> 2 kinds (equal := / proportional :/)
range_list_sv_2023             -> {first, rest}
open_range_list_sv_2017        -> {first, rest}
value_range_sv_2017            -> 2 kinds (expression / range)
value_range_sv_2023            -> 5 kinds (expression / range / dollar_lo / dollar_hi / tolerance)
```

**Annotation inventory:** 834 entries (was 805). +29 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.52 / Contract 1.0.52 Highlights".

### 1.0.51 / Contract 1.0.51 — SV-Slice-51 batch: select + constant_select + constant_range typed (4 rules / 5 annotations + 2 new helper rules with 4 annotations — crosses 800-annotation milestone)

**What changed:** Closes the select / constant_select referent used pervasively across primary's hierarchical-name suffix.

```ebnf
select                       -> {member_chain, tail}
select_tail (NEW)            -> 2 kinds (part_range / bit_select)
constant_select              -> {member_chain, tail}
constant_select_tail (NEW)   -> 2 kinds (part_range / bit_select)
constant_range               -> {lo, hi}
constant_range_expression    -> 2 kinds (expression / part_select_range)
```

**Annotation inventory:** 805 entries (was 796). +9 in this batch — crosses the 800-annotation milestone.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.51 / Contract 1.0.51 Highlights".

### 1.0.50 / Contract 1.0.50 — SV-Slice-50 batch: casting_type + bit_select + system_tf_call typed (3 rules / 9 annotations)

**What changed:** Closes the cast.type field referent and system-task-call dispatch.

```ebnf
casting_type      -> 5 kinds (simple_type / constant_primary / signing / string / const)
bit_select        -> {body}
system_tf_call    -> 3 kinds (args / data_type / expr_clocking)
```

**Annotation inventory:** 796 entries (was 787). +9 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.50 / Contract 1.0.50 Highlights".

### 1.0.49 / Contract 1.0.49 — SV-Slice-49 batch: concat / cast / call_primary / attr_spec typed (9 rules / 14 annotations)

**What changed:** Closes the leaf rules used pervasively across primary_sv_2017/2023.

```ebnf
attr_spec                          -> {name, value}
cast / constant_cast               -> {type, body}
concatenation / constant_concatenation -> {first, rest}
multiple_concatenation / constant_multiple_concatenation -> {count, body}
streaming_concatenation            -> {op, slice_size, body}
call_primary                       -> 6 kinds (split_direct_callable_method / class_scoped_tf / plain_tf / tf / direct_callable_method / system_tf)
```

**Annotation inventory:** 787 entries (was 773). +14 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.49 / Contract 1.0.49 Highlights".

### 1.0.48 / Contract 1.0.48 — SV-Slice-48 batch: primary_sv_2023 + constant_primary_sv_2023 typed (2 rules / 31 annotations)

**What changed:** Completes the parallel sv_2023 forms. Both sv_2017 and sv_2023 primary expression dispatch fully typed end-to-end.

```ebnf
primary_sv_2023           -> 15 kinds (sv_2017 kinds + LRM 2023's call select extension)
constant_primary_sv_2023  -> 16 kinds (sv_2017's 15 + empty_array_concat per LRM 2023)
```

Profile differences from sv_2017: `call` and `function_call` add optional `select` per LRM 2023, and constant_primary adds `empty_array_concat`. Reuses the 3 helper rules from SV-Slice-47.

**Annotation inventory:** 773 entries (was 742). +31 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.48 / Contract 1.0.48 Highlights".

### 1.0.47 / Contract 1.0.47 — SV-Slice-47 batch: primary_sv_2017 + constant_primary_sv_2017 typed (2 rules / 30 annotations + 3 new helper rules with 6 annotations)

**What changed:** Closes the sv_2017 primary expression dispatch reachable from expression_operand.kind=="primary" and constant_expression_operand.kind=="primary".

```ebnf
primary_sv_2017               -> 15 kinds (literal / call / hierarchical / empty_array_concat / multiple_concat / concat / let / paren / cast / assign_pattern / streaming_concat / sequence_method / this / system_dollar / null_class_assign)
constant_primary_sv_2017      -> 15 kinds (literal / ps_parameter / specparam / genvar / formal_port / enum / multiple_concat / concat / function_call / let / paren / cast / assign_pattern / type_reference / null)
primary_hier_scope_prefix (NEW) -> 2 kinds (class_qualifier / package_scope)
instance_or_class_scope (NEW)   -> 2 kinds (instance / class_scope)
enum_id_scope_prefix (NEW)      -> 2 kinds (package_scope / class_scope)
```

**Helper-rule extraction** (7th, 8th, 9th uses of pattern). Now used in 10 places total.

**DEFERRED:** primary_sv_2023 and constant_primary_sv_2023 (parallel structures — to be applied in a follow-up slice).

**Annotation inventory:** 742 entries (was 706). +36 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.47 / Contract 1.0.47 Highlights".

### 1.0.46 / Contract 1.0.46 — SV-Slice-46 batch: expression family typed (14 rules / 62 annotations — crosses 700-annotation milestone)

**What changed:** Single largest impact slice — `expression`, `constant_expression`, and their operand/operator/literal sub-rules underlie every expression-typed field across the grammar.

```ebnf
expression                       -> 3 kinds (base / inside / conditional)
expression_base                  -> 3 kinds (tagged_union / operand_chain / paren_op_assign)
expression_operand               -> 3 kinds (unary / inc_or_dec / primary)
expression_or_dist               -> {expr, dist}
constant_expression              -> {first, rest, ternary}
constant_expression_operand      -> 2 kinds (unary / primary)
inside_expression_sv_2017/2023   -> {expr, ranges}
conditional_expression           -> {predicate, attributes, then_expr, else_expr}
tagged_union_expression_sv_2017/2023 -> {name, value}
primary_literal                  -> 4 kinds (number / time_literal / unbased_unsized_literal / string_literal)
binary_operator                  -> 29 kinds bare {kind}
unary_operator                   -> 11 kinds bare {kind}
```

**Annotation inventory:** 706 entries (was 644). +62 in this batch — crosses the 700-annotation milestone.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.46 / Contract 1.0.46 Highlights".

### 1.0.45 / Contract 1.0.45 — SV-Slice-45 batch: pattern + cond_predicate family typed (6 rules / 18 annotations)

**What changed:** Closes the LRM A.6.7.1 pattern-matching walk path used by case_pattern_item, conditional_statement.condition (via cond_predicate), and constraint_expression's various forms.

```ebnf
cond_predicate              -> {first, rest}
cond_pattern                -> {expression, pattern}
expression_or_cond_pattern  -> 2 kinds (expression / cond_pattern)
pattern_sv_2017             -> 6 kinds (variable_capture / wildcard / expression / tagged / ordered / named)
pattern_sv_2023             -> 7 kinds (same 6 + parenthesized per LRM 2023)
assignment_pattern          -> {exprs: {first, rest}}
```

**Annotation inventory:** 644 entries (was 626). +18 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.45 / Contract 1.0.45 Highlights".

### 1.0.44 / Contract 1.0.44 — SV-Slice-44 batch: list_of_* family typed (20 rules / 22 annotations)

**What changed:** Uniform mini-mixed-array pattern across the small declaration-list rules. Every list_of_* rule referenced from typed parents now exposes `{first, rest}` shape.

- 12 simple `{first, rest}` rules (clocking_decl_assign / defparam / genvar / net / param / path_inputs / path_outputs / specparam / type / variable, plus 2 variable-assignment forms).
- 3 `{first: {name, dims}, rest}` rules (interface / port / variable identifiers — with trailing dimension list per item).
- 2 `{first: {name, dims, init}, rest}` rules (tf_variable / variable_port — with optional initializer per item).
- 1 `{first, second, rest}` rule (cross_items — LRM requires at least 2).
- 2 2-kind dispatches (checker_port_connections, port_connections — ordered/named).

**Annotation inventory:** 626 entries (was 604). +22 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.44 / Contract 1.0.44 Highlights".

### 1.0.43 / Contract 1.0.43 — SV-Slice-43 batch: parameter_value_assignment + arguments family typed (10 rules / 16 annotations — crosses 600-annotation milestone)

**What changed:** Closes the function/task/method-call argument and parameter-instance walks.

```ebnf
parameter_value_assignment_sv_2017/2023       -> {params}
list_of_parameter_assignments_sv_2017         -> 2 kinds (ordered / named)
list_of_parameter_value_assignments_sv_2023   -> 2 kinds (parallel)
named_parameter_assignment                    -> {name, value}
named_argument                                -> {name, value}
list_of_arguments                             -> 3 kinds (ordered / named / mixed)
list_of_arguments_ordered                     -> {first, rest}
list_of_arguments_named                       -> {first, rest}
list_of_arguments_mixed                       -> {head, named: {first, rest}}
list_of_arguments_mixed_head                  -> 2 kinds (single / chain — recursive)
```

**Annotation inventory:** 604 entries (was 588). +16 in this batch — crosses the 600-annotation milestone.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.43 / Contract 1.0.43 Highlights".

### 1.0.42 / Contract 1.0.42 — SV-Slice-42 batch: signing + struct_union + enum + type_reference + class_type internals typed (9 rules / 21 annotations + 2 new helper rules with 5 annotations)

**What changed:** Closes data_type field structural-content walks.

```ebnf
signing                          -> 2 kinds bare (signed / unsigned)
struct_union_sv_2017             -> 2 kinds (struct / union with optional tagged)
struct_union_sv_2023             -> 2 kinds (struct / union with union_modifier helper)
union_modifier (NEW)             -> 2 kinds bare (soft / tagged)
struct_union_member              -> {attributes, random_qualifier, data_type, decls}
enum_base_type                   -> 3 kinds (atom / vector / type_alias)
enum_name_declaration            -> {name, range, value}
type_reference_sv_2017/2023      -> 2 kinds each (expression / data_type[_or_incomplete_class])
class_type                       -> {head, params, suffix}
class_type_head (NEW)            -> 3 kinds (scoped / class / interface_class)
```

**Helper-rule extraction** (5th and 6th uses): `union_modifier` from `( kw_soft | kw_tagged )?`, `class_type_head` from leading 3-way parens-Or in class_type.

**Annotation inventory:** 588 entries (was 567). +21 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.42 / Contract 1.0.42 Highlights".

### 1.0.41 / Contract 1.0.41 — SV-Slice-41 batch: data_type family typed (8 rules / 36 annotations)

**What changed:** Pervasive impact — `data_type` fields appear across the entire grammar.

```ebnf
data_type                                         -> 15 kinds (integer_vector / integer_atom / non_integer / struct_union / enum / string / chandle / virtual_interface / scoped_data_type / known_unscoped_data_type / class_type / provisional_class_type / event / covergroup / type_reference)
data_type_or_implicit                             -> 2 kinds (data_type / implicit)
data_type_or_void                                 -> 2 kinds (data_type / void)
data_type_or_incomplete_class_scoped_type_sv_2023 -> 2 kinds
implicit_data_type                                -> {signing, dims}
integer_atom_type                                 -> 6 kinds bare (byte / shortint / int / longint / integer / time)
integer_vector_type                               -> 3 kinds bare (bit / logic / reg)
non_integer_type                                  -> 3 kinds bare (shortreal / real / realtime)
integer_type                                      -> 2 kinds (vector / atom)
```

**Annotation inventory:** 567 entries (was 531). +36 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.41 / Contract 1.0.41 Highlights".

### 1.0.40 / Contract 1.0.40 — SV-Slice-40 batch: simple immediate assertions + inc_or_dec + weight_specification typed (6 rules / 11 annotations)

**What changed:** Closes immediate_assertion_statement.kind=="simple", inc_or_dec_expression internals, and weight_specification_sv_2017.

```ebnf
simple_immediate_assertion_statement -> 3 kinds (assert / assume / cover)
simple_immediate_assert_statement    -> {condition, action}
simple_immediate_assume_statement    -> {condition, action}
simple_immediate_cover_statement     -> {condition, statement}
inc_or_dec_expression                -> 2 kinds (prefix {op, attributes, lvalue} / postfix {lvalue, attributes, op})
weight_specification_sv_2017         -> 3 kinds (number / identifier / expression)
```

**Annotation inventory:** 531 entries (was 520). +11 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.40 / Contract 1.0.40 Highlights".

### 1.0.39 / Contract 1.0.39 — SV-Slice-39 batch: rs_* family typed (17 rules / 31 annotations — crosses 500-annotation milestone)

**What changed:** Closes the random-sequence walk path end-to-end. After this slice, every reachable randsequence_statement → production → rules.{first,rest} → rs_rule → rs_production_list → rs_prod → ... resolves through typed shapes.

```ebnf
rs_case                         -> {expr, items: {first, rest}}
rs_case_item_sv_2017/2023       -> 2 kinds (expr_list / default)
rs_code_block                   -> {body}
rs_if_else_sv_2017/2023         -> {condition, then_body, else_body}
rs_prod_sv_2017/2023            -> 5 kinds (production_item / code_block / if_else / repeat / case)
rs_production_sv_2023           -> {return_type, name, ports, rules: {first, rest}}
rs_production_item_sv_2023      -> {name, args}
rs_production_list_sv_2017/2023 -> 2 kinds (productions / rand_join)
rs_repeat_sv_2017/2023          -> {count, body}
rs_rule_sv_2017/2023            -> {productions, weight}
rs_weight_specification_sv_2023 -> 3 kinds (number / identifier / expression)
```

**Annotation inventory:** 520 entries (was 489). +31 in this batch — crosses the 500-annotation milestone.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.39 / Contract 1.0.39 Highlights".

### 1.0.38 / Contract 1.0.38 — SV-Slice-38 batch: randsequence top-level + production typed (4 rules / 4 annotations)

**What changed:** Closes the last raw-envelope statement_item kind.

```ebnf
randsequence_statement_sv_2017 -> {start, productions: {first, rest}}
randsequence_statement_sv_2023 -> {start, productions: {first, rest}}  (uses rs_production per LRM 2023)
production_sv_2017             -> {return_type, name, ports, rules: {first, rest}}
production_item_sv_2017        -> {name, args}
```

**DEFERRED:** rs_* family internals (rs_rule, rs_prod, rs_case, rs_if_else, rs_repeat, rs_code_block) — referenced from production.rules.

**Annotation inventory:** 489 entries (was 485). +4 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.38 / Contract 1.0.38 Highlights".

### 1.0.37 / Contract 1.0.37 — SV-Slice-37 batch: blocking_assignment typed via helper-rule extraction (3 rules / 12 annotations + 1 new helper rule with 3 annotations)

**What changed:** Closes the last DEFERRED statement_item kind. After this slice, **all 20 (sv_2017) / 19 (sv_2023) statement_item kinds expose typed dispatch end-to-end**.

```ebnf
blocking_assignment_sv_2017     -> 4 kinds (delay_assign / dynamic_array_new / class_new / operator)
blocking_assignment_sv_2023     -> 5 kinds (same 4 + inc_or_dec per LRM 2023)
class_or_package_scope (NEW)    -> 3 kinds (instance / class_scope / package_scope)
```

**Helper-rule extraction (4th use of the pattern).** `class_or_package_scope` extracted from `( implicit_class_handle dot | class_scope | package_scope )?` — same pattern as if_generate_else_clause / net_strength / net_vector_scalar / conditional_else_branch.

**Annotation inventory:** 485 entries (was 473). +12 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.37 / Contract 1.0.37 Highlights" with full annotation source + helper-rule pattern table + 100% statement_item dispatch coverage.

### 1.0.36 / Contract 1.0.36 — SV-Slice-36 batch: assignments + procedural assertions + randcase typed (8 rules / 16 annotations)

**What changed:** Closes 4 more statement_item kinds. After this slice, 19 of statement_item's 19/20 kinds expose typed dispatch end-to-end (only blocking_assignment remains DEFERRED).

```ebnf
nonblocking_assignment           -> {lvalue, control, value}
procedural_continuous_assignment -> 6 kinds (assign / deassign / force_variable / force_net / release_variable / release_net)
clocking_drive                   -> {lvalue, cycle_delay, value}
randcase_statement               -> {items: {first, rest}}
randcase_item                    -> {weight, body}
procedural_assertion_statement   -> 3 kinds (concurrent / immediate / checker_instantiation)
immediate_assertion_statement    -> 2 kinds (simple / deferred)
variable_assignment              -> {lvalue, value}
```

**DEFERRED:** `blocking_assignment_sv_2017/2023` (parens-Or workaround needed, 4th use of helper-rule pattern).

**Annotation inventory:** 473 entries (was 457). +16 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.36 / Contract 1.0.36 Highlights" with full annotation source + statement_item dispatch coverage table.

### 1.0.35 / Contract 1.0.35 — SV-Slice-35 batch: conditional_statement typed via helper-rule extraction (1 rule / 1 annotation + 1 new helper rule with 2 annotations)

**What changed:** Closes the SV-Slice-34 DEFERRED `conditional_statement` typing using the helper-rule extraction pattern (third use after if_generate_else_clause and net_strength/net_vector_scalar).

```ebnf
conditional_statement      -> {unique_priority, condition, then_body, else_body}
conditional_else_branch (NEW) -> 2 kinds (elseif {body} / else {body})
```

**Annotation inventory:** 457 entries (was 454). +3 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.35 / Contract 1.0.35 Highlights".

### 1.0.34 / Contract 1.0.34 — SV-Slice-34 batch: case + loop families typed (7 rules / 18 annotations)

**What changed:** Closes case-statement and loop-statement walks.

```ebnf
case_statement           -> {unique_priority, keyword, expr, items: {first, rest}}
case_keyword             -> 3 kinds bare (case / casez / casex)
case_item                -> 2 kinds (expr_list / default)
case_pattern_item        -> 2 kinds (pattern {pattern, condition, body} / default)
case_inside_item_sv_2017 -> 2 kinds (range_list using open_range_list / default)
case_inside_item_sv_2023 -> 2 kinds (range_list using range_list per LRM 2023 / default)
loop_statement           -> 6 kinds (forever / repeat / while / for / do_while / foreach)
```

**DEFERRED:** unique_priority (grammar duplicate-branch bug), conditional_statement (parens-Or with lookahead — needs helper rule).

**Annotation inventory:** 454 entries (was 436). +18 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.34 / Contract 1.0.34 Highlights".

### 1.0.33 / Contract 1.0.33 — SV-Slice-33 batch: procedural-statement forms typed (11 rules / 26 annotations)

**What changed:** Closes 7 of statement_item's 19/20 kinds.

```ebnf
disable_statement                    -> 3 kinds (task / block / fork)
jump_statement                       -> 3 kinds (return / break / continue)
wait_statement                       -> 3 kinds (wait / wait_fork / wait_order)
event_trigger_sv_2017                -> 2 kinds (non_blocking / blocking)
event_trigger_sv_2023                -> 2 kinds (parallel; adds `select` field)
procedural_timing_control_statement  -> {control, body}
procedural_timing_control            -> 3 kinds (delay / event / cycle)
subroutine_call                      -> 5 kinds (class_scoped_tf / tf / system_tf / method / randomize)
subroutine_call_statement            -> 2 kinds (call / void_cast)
seq_block                            -> {label, declarations, statements, end_label}
par_block                            -> {label, declarations, statements, join, end_label}
```

**Annotation inventory:** 436 entries (was 410). +26 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.33 / Contract 1.0.33 Highlights".

### 1.0.32 / Contract 1.0.32 — SV-Slice-32 batch: statement_item dispatch typed (3 rules / 43 annotations — crosses 400-annotation milestone)

**What changed:** Closes statement.body field, exposing typed dispatch into all 20 (sv_2017) / 19 (sv_2023) procedural-statement forms.

```ebnf
statement_item_sv_2017  -> 20 kinds (blocking_assignment, nonblocking_assignment, procedural_continuous_assignment, case, conditional, inc_or_dec_expression, subroutine_call, disable, event_trigger, loop, jump, par_block, procedural_timing_control, seq_block, wait, procedural_assertion, clocking_drive, randsequence, randcase, expect_property)
statement_item_sv_2023  -> 19 kinds (sv_2017 minus inc_or_dec_expression — LRM 2023 subsumes into blocking_assignment with ++/--)
block_item_declaration  -> 4 kinds (block_data / local_parameter / parameter / let)
```

**Annotation inventory:** 410 entries (was 367). +43 in this batch — crosses the 400-annotation milestone.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.32 / Contract 1.0.32 Highlights".

### 1.0.31 / Contract 1.0.31 — SV-Slice-31 batch: action_block + statement framing typed (5 rules / 9 annotations)

**What changed:** Closes action_block walk path (used pervasively by assertions) and statement framing path (used by function/task bodies).

```ebnf
action_block               -> 2 kinds (always {body} / with_else {pass, fail})
statement                  -> {label, attributes, body}
statement_or_null          -> 2 kinds (statement {body} / null {attributes})
function_statement_or_null -> 2 kinds (parallel)
tf_item_declaration        -> 2 kinds (block_item / tf_port)
```

**Annotation inventory:** 367 entries (was 358). +9 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.31 / Contract 1.0.31 Highlights".

### 1.0.30 / Contract 1.0.30 — SV-Slice-30 batch: deferred immediate assertions typed (5 rules / 10 annotations)

**What changed:** Closes assertion_item.kind=="deferred_immediate" walk path.

```ebnf
deferred_immediate_assertion_item      -> {label, body}
deferred_immediate_assertion_statement -> 3 kinds (assert / assume / cover)
deferred_immediate_assert_statement    -> 2 kinds (zero_delay / final) {expression, action}
deferred_immediate_assume_statement    -> 2 kinds (parallel to assert)
deferred_immediate_cover_statement     -> 2 kinds (parallel; uses {expression, statement})
```

**Annotation inventory:** 358 entries (was 348). +10 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.30 / Contract 1.0.30 Highlights".

### 1.0.29 / Contract 1.0.29 — SV-Slice-29 batch: concurrent assertion + constraint family typed (16 rules / 28 annotations)

**What changed:** Closes assertion_item.kind=="concurrent" walk (typed in SV-Slice-24) and class_constraint walk (typed in SV-Slice-27).

```ebnf
concurrent_assertion_statement -> 5 kinds (assert_property / assume_property / cover_property / cover_sequence / restrict_property)
assert_property_statement      -> {spec, action}
assume_property_statement      -> {spec, action}
cover_property_statement       -> {spec, statement}
cover_sequence_statement       -> {clocking, disable_iff, sequence, statement}
restrict_property_statement    -> {spec}
expect_property_statement      -> {spec, action}
constraint_declaration_sv_2017 -> {static_keyword, name, block}
constraint_declaration_sv_2023 -> {static_keyword, dynamic_override, name, block}
constraint_block               -> {items}
constraint_block_item          -> 2 kinds (solve_before {before, after} / expression {body})
constraint_expression          -> 6 kinds (expression / uniqueness / implies / if / foreach / disable_soft)
constraint_prototype_sv_2017   -> {qualifier, static_keyword, name}
constraint_prototype_sv_2023   -> {qualifier, static_keyword, dynamic_override, name}
constraint_prototype_qualifier -> 2 kinds bare (extern / pure)
constraint_set                 -> 2 kinds (single {body} / block {exprs})
```

**Annotation inventory:** 348 entries (was 320). +28 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.29 / Contract 1.0.29 Highlights".

### 1.0.28 / Contract 1.0.28 — SV-Slice-28 batch: class qualifiers typed (3 rules / 6 annotations)

**What changed:** Completes SV-Slice-27's class body picture.

```ebnf
method_qualifier   -> 2 kinds (virtual {pure} / class_item_qualifier {body})
property_qualifier -> 2 kinds (random {body} / class_item_qualifier {body})
random_qualifier   -> 2 kinds (rand / randc — bare {kind})
```

**Annotation inventory:** 320 entries (was 314). +6 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.28 / Contract 1.0.28 Highlights".

### 1.0.27 / Contract 1.0.27 — SV-Slice-27 batch: class body sub-tree typed (6 rules / 30 annotations)

**What changed:** Closes the class body walk path. Method qualifiers, property kind (decl vs const), method kind (task / function / pure_virtual / extern / constructor / extern_constructor) all now `kind`-discriminated.

```ebnf
class_item_sv_2017     -> 8 kinds (property / method / constraint / class / covergroup / local_parameter / parameter / semi)
class_item_sv_2023     -> 9 kinds (same plus interface_class for LRM 2023 nested interface-class decls)
class_item_qualifier   -> 3 kinds (static / protected / local — bare {kind})
class_constraint       -> 2 kinds (prototype / declaration)
class_property         -> 2 kinds (decl {qualifiers, body} / const {qualifiers, data_type, name, init})
class_method           -> 6 kinds (task / function / pure_virtual / extern / constructor / extern_constructor)
```

**Annotation inventory:** 314 entries (was 284). +30 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.27 / Contract 1.0.27 Highlights" with full annotation source + field semantics + profile differences.

### 1.0.26 / Contract 1.0.26 — SV-Slice-26 batch: net_declaration typed via helper-rule extraction (4 rules / 10 annotations + 2 new helper rules)

**What changed:** Closes the net_declaration walk path. Two new helper rules extracted from inline parens-Or to dodge task #38 (same pattern as SV-Slice-23).

```ebnf
net_declaration_sv_2017     -> 3 kinds (wire / alias / interconnect)
net_declaration_sv_2023     -> 3 kinds (alias branch field is `nettype_id` per LRM 2023)
net_strength (NEW)          -> 2 kinds (drive {body} / charge {body})
net_vector_scalar (NEW)     -> 2 kinds (vectored / scalared, bare {kind})
```

**Annotation inventory:** 284 entries (was 274). +10 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.26 / Contract 1.0.26 Highlights" with full annotation source + helper-rule rationale.

### 1.0.25 / Contract 1.0.25 — SV-Slice-25 batch: data/function/task declarations + bodies typed (8 rules / 14 annotations)

**What changed:** Closes the data / function / task walk paths from package_or_generate_item_declaration.

```ebnf
data_declaration_sv_2017     -> 4 kinds (variable_decl / type / package_import / net_type)
data_declaration_sv_2023     -> 4 kinds (same 3 + nettype, LRM 2023 naming)
function_declaration_sv_2017 -> {lifetime, body}
function_declaration_sv_2023 -> {dynamic_override, lifetime, body}
function_body_declaration    -> {return_type, name, items, statements, end_label}
task_declaration_sv_2017     -> {lifetime, body}
task_declaration_sv_2023     -> {dynamic_override, lifetime, body}
task_body_declaration        -> {name, items, statements, end_label}
```

**DEFERRED:** `net_declaration_sv_2017/2023` typing (parens-Or workaround needed; planned for next slice with helper-rule extraction following SV-Slice-23 pattern).

**Annotation inventory:** 274 entries (was 260). +14 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.25 / Contract 1.0.25 Highlights" with full annotation source + field semantics + profile differences (net_type vs nettype).

### 1.0.24 / Contract 1.0.24 — SV-Slice-24 batch: assertion + genvar dispatch typed (7 rules / 26 annotations)

**What changed:** Closes the assertion-item walk path and the loop_generate_construct init/step typed dispatch.

```ebnf
assertion_item              -> 2 kinds (concurrent / deferred_immediate)
assertion_item_declaration  -> 3 kinds (property / sequence / let)
concurrent_assertion_item   -> 2 kinds (statement {label, body} / checker_instantiation {body})
genvar_initialization       -> {genvar_keyword, name, value}
genvar_iteration            -> 3 kinds (assign / prefix_inc_dec / postfix_inc_dec)
assignment_operator         -> 13 kinds (assign / plus_assign / ... / arithmetic_shift_right_assign) — bare {kind}
inc_or_dec_operator         -> 2 kinds (plus_plus / minus_minus) — bare {kind}
```

**Annotation inventory:** 260 entries (was 234). +26 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.24 / Contract 1.0.24 Highlights" with full annotation source + field semantics for each rule.

### 1.0.23 / Contract 1.0.23 — SV-Slice-23 batch: generate-construct internals typed (6 rules / 9 annotations + 1 new helper rule)

**What changed:** Closes the loop / conditional / case-generate dispatch path.

```ebnf
loop_generate_construct        -> {init, condition, step, block}
conditional_generate_construct -> 2 kinds (if / case)
if_generate_construct          -> {condition, then_block, else_clause}
if_generate_else_clause (NEW)  -> 2 kinds (elseif {body} / else_block {body})
case_generate_construct        -> {expr, items: {first, rest}}
case_generate_item             -> 2 kinds (expr_list {exprs: {first, rest}, block} / default {block})
```

**Notable:** New helper rule `if_generate_else_clause` extracted from inline parens-Or to dodge task #38 (parens-grouped-Or trailing-annotation attribution bug). This is now the recommended workaround for similar `( a | b )?` patterns until task #38 is fixed.

**Annotation inventory:** 234 entries (was 225). +9 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.23 / Contract 1.0.23 Highlights" with full annotation source, helper-rule rationale, and full module-item-to-generate-block walker recipe.

### 1.0.22 / Contract 1.0.22 — SV-Slice-22 batch: generate sub-tree typed (3 rules / 7 annotations)

**What changed:** Closes the generate-construct walk path. Every reachable `non_port_module_item.kind=='generate_region'` exposes typed `{items}`, every `generate_item` discriminates which form it carries, and every `generate_block` (anonymous, labeled, or bare-generate_item passthrough) exposes its name/label/items/end_label.

```ebnf
generate_region -> {items}
generate_item   -> 3 kinds: module_or_generate_item / interface_or_generate_item / checker_or_generate_item
generate_block  -> 3 kinds: anonymous {label, items, end_label} / labeled {name, label, items, end_label} / generate_item {body}
```

**Annotation inventory:** 225 entries (was 218). +7 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.22 / Contract 1.0.22 Highlights" with full annotation source + walker recipes for each generate form.

### 1.0.21 / Contract 1.0.21 — SV-Slice-21 batch: module_common_item + package_or_generate_item_declaration typed (4 rules / 55 annotations — biggest batch yet)

**What changed:** Both halves of the cascading walk path that SV-Slice-19/20 set up are now closed: every reachable `module_common_item` and every reachable `package_or_generate_item_declaration` discriminates which actual sub-construct was matched.

```ebnf
module_common_item_sv_2017                    -> 13 kinds (module_or_generate_item_declaration / interface_instantiation / program_instantiation / assertion_item / bind_directive / continuous_assign / net_alias / initial_construct / final_construct / always_construct / loop_generate_construct / conditional_generate_construct / elaboration_system_task)
module_common_item_sv_2023                    -> 13 kinds (same except elaboration_severity_system_task)
package_or_generate_item_declaration_sv_2017  -> 14 kinds (incl. local_parameter_declaration / parameter_declaration / data_declaration / task_declaration / function_declaration / class_declaration / covergroup_declaration / semi / ...)
package_or_generate_item_declaration_sv_2023  -> 15 kinds (same plus interface_class_declaration)
```

The wrapper rules `module_common_item := _sv_2017 | _sv_2023` and `package_or_generate_item_declaration := _sv_2017 | _sv_2023` stay un-annotated (transparent profile-routers, same pattern as `module_declaration` / `interface_declaration`).

**Annotation inventory:** 218 entries (was 163). +55 in this batch — biggest single-slice contribution.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.21 / Contract 1.0.21 Highlights" with full annotation source + 6-level walker recipe.

### 1.0.20 / Contract 1.0.20 — SV-Slice-20 batch: interface + program items dispatch tree typed (5 rules / 19 annotations)

**What changed:** Mirror of SV-Slice-19's module-items batch, applied to the interface and program sub-trees. Every `header.items` and `body.items` field on every typed interface/program declaration now surfaces kind-discriminated dispatch.

```ebnf
interface_item              -> 2 kinds: port_declaration / non_port_item
interface_or_generate_item  -> 2 kinds (with attributes): module_common_item / extern_tf_declaration
non_port_interface_item     -> 6 kinds: generate_region / interface_or_generate / program_declaration / modport_declaration / interface_declaration / timeunits_declaration
program_item                -> 2 kinds: port_declaration / non_port_item
non_port_program_item       -> 7 kinds: continuous_assign / module_or_generate_item_declaration / initial_construct / final_construct / concurrent_assertion_item / timeunits_declaration / program_generate_item (first 5 with attributes)
```

**Annotation inventory:** 163 entries (was 144). +19 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.20 / Contract 1.0.20 Highlights" with full annotation source + interface/program walker recipes.

### 1.0.19 / Contract 1.0.19 — SV-Slice-19 batch: module-items dispatch tree typed (5 rules / 22 annotations)

**What changed:** Largest batch yet. Every `header.items` and `body.items` field on every typed module/interface/program declaration now surfaces kind-discriminated dispatch.

```ebnf
module_item                          -> 2 kinds: port_declaration / non_port_item
module_or_generate_item              -> 5 kinds (with attributes:): parameter_override / gate_instantiation / udp_instantiation / module_instantiation / module_common_item
module_or_generate_item_declaration  -> 5 kinds: package_or_generate / genvar / clocking / default_clocking / default_disable_iff
non_port_module_item                 -> 8 kinds: generate_region / module_or_generate / specify_block / specparam_declaration / program_declaration / module_declaration / interface_declaration / timeunits_declaration
continuous_assign                    -> 2 kinds: net / variable
```

**Annotation inventory:** 144 entries (was 122). +22 in this batch — largest single-slice contribution to date.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.19 / Contract 1.0.19 Highlights" with full annotation source + 5-layer module-items walker recipe.

### 1.0.18 / Contract 1.0.18 — SV-Slice-18 batch: UDP truth-table entries typed

**What changed:** 3 rules / 3 annotations completing the UDP truth-table walk path.

```ebnf
combinational_entry   -> {inputs, output}
sequential_entry      -> {inputs, current_state, next_state}
udp_initial_statement -> {name, init_val}
```

**Annotation inventory:** 122 entries (was 119). +3.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.18 / Contract 1.0.18 Highlights".

### 1.0.17 / Contract 1.0.17 — SV-Slice-17 batch: UDP body sub-tree typed

**What changed:** 6 rules / 8 annotations completing UDP declaration internals.

```ebnf
udp_body                     -> 2 kinds: combinational/sequential
udp_input_declaration        -> {attributes, identifiers}
udp_output_declaration       -> 2 kinds: wire/reg
combinational_body           -> {entries: {first, rest}}
sequential_body              -> {initial, entries: {first, rest}}
list_of_udp_port_identifiers -> {first, rest}
```

**UDP declaration internals fully typed end-to-end** — combined with prior UDP top-level rules (SV-Slice-12) and port lists (SV-Slice-15), consumers walking a `primitive ... endprimitive` get clean typed access at every level.

**Annotation inventory:** 119 entries (was 111). +8 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.17 / Contract 1.0.17 Highlights" with full UDP walker recipe.

### 1.0.16 / Contract 1.0.16 — SV-Slice-16 batch: port + port_direction + package_import family typed

**What changed:** 4 rules / 9 annotations.

```ebnf
port                       -> 2 kinds: expression / named (dot-form)
port_direction             -> 4 kinds: input / output / inout / ref
package_import_declaration -> {items: {first, rest}}
package_import_item        -> 2 kinds: explicit / wildcard
```

**DEFERRED:** `ansi_port_declaration` per-branch typing — branch 0 starts with a parens-grouped Or `( net_port_header | interface_port_header )?` which triggers task #38's branch-attribution bug. Tracked as follow-up either via task #38 fix or grammar refactor.

**Annotation inventory:** 111 entries (was 102). +9 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.16 / Contract 1.0.16 Highlights".

### 1.0.15 / Contract 1.0.15 — SV-Slice-15 batch: port-list family + small structural rules typed

**What changed:** 6 rules / 7 annotations. Every `header.ports` field on every typed module/interface/program/UDP declaration now surfaces a typed shape.

```ebnf
list_of_ports             -> {first, rest}     (mini-mixed-array)
list_of_port_declarations -> $2 (transparent passthrough of inner optional)
udp_port_list             -> {output, inputs: {first, rest}}
udp_declaration_port_list -> {output, inputs: {first, rest}}
anonymous_program         -> {items}
package_export_declaration -> 2 kinds (wildcard / explicit with {first, rest})
```

**Annotation inventory:** 102 entries (was 95). +7 in this batch. **Crossing 100 annotations** — campaign mid-flight.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.15 / Contract 1.0.15 Highlights" with full annotation source + per-rule notes.

### 1.0.14 / Contract 1.0.14 — SV-Slice-14 batch: bind sub-tree completion + interface_class_declaration + config_declaration

**What changed:** 5 rules typed in one batch:

```ebnf
bind_target_scope          -> 2 kinds: module/interface ({kind, name})
bind_target_instance       -> {name, bit_select}
bind_target_instance_list  -> {first, rest} (mini-mixed-array workaround)
interface_class_declaration -> {name, parameters, extends, items, end_label}
config_declaration         -> {name, local_params, design, rules, end_label}
```

**Bind sub-tree fully typed** — combined with SV-Slice-13's bind_directive/bind_instantiation typing, consumers walking a bind directive get clean typed access at every level (target_scope, target_instance, instances, instantiation).

**Annotation inventory:** 95 entries (was 89). +6 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.14 / Contract 1.0.14 Highlights" with full annotation source + bind walker recipe.

### 1.0.13 / Contract 1.0.13 — SV-Slice-13 batch: bind_directive + bind_instantiation + package_item per-branch typed

**What changed:** 3 Or rules typed. Consumers gain clean kind dispatch on description's `package_item` and `bind_directive` branches.

```ebnf
bind_directive       -> 2 kinds: scoped/single
bind_instantiation   -> 4 kinds: program/module/interface/checker
package_item         -> 4 kinds: declaration/anonymous_program/export/timeunits
```

**Annotation inventory:** 89 entries (was 79). +10 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.13 / Contract 1.0.13 Highlights" with full annotation source + consumer dispatch recipe.

### 1.0.12 / Contract 1.0.12 — SV-Slice-12 batch: UDP declaration family typed

**What changed:** 4 rules typed across the UDP (User-Defined Primitive) declaration family — sibling pattern to module/interface/program with one twist: `udp_declaration_sv_*` nonansi branch has a `udp_port_declaration udp_port_declaration*` mini-mixed-array, handled with the `{first, rest}` workaround.

```ebnf
udp_ansi_declaration     -> {attributes, name, ports}
udp_nonansi_declaration  -> {attributes, name, ports}
udp_declaration_sv_2017  -> 5 per-branch kinds: nonansi/ansi/extern_nonansi/extern_ansi/wildcard
udp_declaration_sv_2023  -> mirror of sv_2017 with positional shift in wildcard
```

**Mini-mixed-array workaround:** the nonansi branch's `udp_port_declaration udp_port_declaration*` pattern uses `port_decls: {first: $2, rest: $3}` to surface the required-first + repeat shape. Same idiom as `attribute_instance: {first, rest}` from SV-Slice-6. Consumers iterate `port_decls.first` once then walk `port_decls.rest`.

**Annotation inventory:** 79 entries (was 67). +12 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.12 / Contract 1.0.12 Highlights" — full annotation source + consumer dispatch recipe + mini-mixed-array workaround documentation.

### 1.0.11 / Contract 1.0.11 — SV-Slice-11 batch: program-header sub-tree typed (sibling of module/interface headers)

**What changed:** 2 rules typed: `program_ansi_header`, `program_nonansi_header`. Both expose the same 6-field shape: `attributes`, `lifetime`, `name`, `imports`, `parameters`, `ports`. Same field names as module / interface header pairs (program is sans `keyword:` since it only has one keyword).

**Verified on `program p; endprogram\n`:** `header.name = "p"` (clean string from SV-Slice-8), all 6 fields present.

**Annotation inventory:** 67 entries (was 65). +2 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.11 / Contract 1.0.11 Highlights".

### 1.0.10 / Contract 1.0.10 — SV-Slice-10 batch: class + package + program declarations typed

**What changed:** 5 rules typed: `class_declaration_sv_2017` and `class_declaration_sv_2023` (single-sequence shapes; sv_2017 has `lifetime:`, sv_2023 has `final_specifier:` per LRM-2023 semantics), `package_declaration` (single sequence with attribute_instance* prefix), `program_declaration_sv_2017` and `program_declaration_sv_2023` (5 per-branch kinds each, mirroring module/interface).

**Verified empirically on `program p; endprogram\n`:**

```text
source_text[0]: {kind: "description", body: {
    kind: "program_declaration",
    body: {kind: "ansi", header: {...}, timeunits: [], items: [], end_label: []}
}}
```

**Module/interface/program tests still pass** with the same regenerated parser — annotations didn't introduce regressions.

**Open follow-up:** `package p; endpackage\n` parse rejected at position 0 despite `package_declaration` being in `description`'s Or set. Annotation registered correctly per the inventory; runtime parse failure appears pre-existing. Module/interface/program parsing unaffected. Tracking separately.

**Class top-level parse:** `class C; endclass\n` is also rejected — but this is expected, since class_declaration isn't directly in source_text_item's reachable set; class declarations are reached through `package_item` or other subsidiary contexts.

**Annotation inventory:** 65 entries (was 53). +12 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.10 / Contract 1.0.10 Highlights".

### 1.0.9 / Contract 1.0.9 — SV-Slice-9 batch: interface declarations typed (full mirror of module pattern)

**What changed:** 4 rules typed: `interface_ansi_header`, `interface_nonansi_header`, `interface_declaration_sv_2017` (5 per-branch kinds), `interface_declaration_sv_2023` (same 5 kinds with positional shift). Interface declarations now have the same typed surface as module declarations. 4-layer typed dispatch end-to-end + clean identifier strings.

**Empirical for `interface bus; endinterface\n`:**

```text
source_text[0]: {kind: "description", body: {
    kind: "interface_declaration",
    body: {
        kind: "ansi",
        header: {name: "bus", attributes: [], lifetime: [], imports: [], parameters: [], ports: []},
        timeunits: [], items: [], end_label: []
    }
}}
```

**Difference from module pattern:** No `keyword:` field on `interface_<form>_header` (only one `interface` keyword exists). Otherwise field names mirror `module_<form>_header`.

**Annotation inventory:** 53 entries (was 41). +12 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.9 / Contract 1.0.9 Highlights".

### 1.0.8 / Contract 1.0.8 — SV-Slice-8 batch: identifier-leaf rules typed (clean strings propagate through every identifier field)

**What changed:** 4 identifier-leaf rules typed with `-> $2` transparent passthrough. Highest-leverage slice yet — every rule that resolves to an identifier now surfaces a clean JSON string instead of the raw envelope chain.

```ebnf
simple_identifier          := trivia /[a-zA-Z_][a-zA-Z0-9_$]*/                            -> $2
escaped_identifier         := trivia /\\[!-~]+/                                            -> $2
non_keyword_identifier     := !reserved_non_keyword_identifier identifier                  -> $2
simple_identifier_no_scope := trivia /[a-zA-Z_][a-zA-Z0-9_$]*(?![ \t\r\n]*::)/             -> $2
```

**Empirical pre/post for `module m; endmodule\n`:**

```text
# Pre — header.name was raw envelope chain:
"header": {"keyword": {"kind": "module"}, "name": [[], [[], "m"]], ...}

# Post — clean string:
"header": {"keyword": {"kind": "module"}, "name": "m", ...}
```

**Propagation:** `simple_identifier` / `escaped_identifier` are leaves of `identifier` (transparent Or). `non_keyword_identifier` strips the negative lookahead. `declaration_identifier` / `module_identifier` / `class_identifier` / `package_identifier` / `interface_identifier` etc. are all transparent aliases — they automatically surface clean strings now. Every future-typed rule that exposes an identifier as a named field gets a clean string for free.

**Annotation inventory:** 41 entries (was 37). +4 in this batch.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.8 / Contract 1.0.8 Highlights".

### 1.0.7 / Contract 1.0.7 — SV-Slice-7 batch: `module_keyword` + `lifetime` + `module_ansi_header` + `module_nonansi_header` typed (4 layers of dispatch end-to-end)

**What changed:** 4-rule batch slice typing the header sub-tree of module declarations. Four layers of typed dispatch are now end-to-end.

```ebnf
module_keyword         := kw_module      -> {kind: "module"}
                        | kw_macromodule -> {kind: "macromodule"}

lifetime               := kw_static      -> {kind: "static"}
                        | kw_automatic   -> {kind: "automatic"}

module_ansi_header     := attribute_instance* module_keyword (lifetime)? module_identifier package_import_declaration* (parameter_port_list)? (list_of_port_declarations)? semi
                       -> {attributes: $1, keyword: $2, lifetime: $3, name: $4, imports: $5, parameters: $6, ports: $7}

module_nonansi_header  := attribute_instance* module_keyword (lifetime)? module_identifier package_import_declaration* (parameter_port_list)? list_of_ports semi
                       -> {attributes: $1, keyword: $2, lifetime: $3, name: $4, imports: $5, parameters: $6, ports: $7}
```

**Empirical for `module m; endmodule\n`:** the previously-raw `header:` field of the ansi-kind module_declaration_sv_2017 now resolves to a typed object with `keyword: {kind: "module"}` and named fields for all 7 components. ANSI and non-ANSI forms expose the same field names — consumer code walking the header is uniform across both.

**Annotation inventory:** 37 entries (was 31). +6 in this batch (2 module_keyword + 2 lifetime + 1 module_ansi_header + 1 module_nonansi_header).

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.7 / Contract 1.0.7 Highlights" — has the per-rule annotation source code + 4-layer consumer dispatch recipe.

### 1.0.6 / Contract 1.0.6 — SV-Slice-6 batch: `attribute_instance` + `module_declaration_sv_2017/2023` typed (3 layers of dispatch end-to-end)

**What changed:** Multi-rule batch slice. Three rules typed in one pass: `attribute_instance` (`{first, rest}` shape), `module_declaration_sv_2017` (5 per-branch kind labels: ansi/nonansi/wildcard/extern_nonansi/extern_ansi), `module_declaration_sv_2023` (same kind labels as sv_2017; wildcard branch's positional indices shift to accommodate `dot star` vs `dot_star`).

**Three layers of typed dispatch end-to-end** — `source_text_item.kind` (SV-Slice-3) → `description.kind` (SV-Slice-4) → `module_declaration_sv_<profile>.kind` (this slice). For `module m; endmodule\n`:

```json
{
  "type": "systemverilog_file",
  "source_text": [
    {
      "kind": "description",
      "body": {
        "kind": "module_declaration",
        "body": {
          "kind": "ansi",
          "header": [<module_ansi_header envelope>],
          "timeunits": [],
          "items": [],
          "end_label": []
        }
      }
    }
  ]
}
```

**Annotation inventory:** 31 entries (was 20). +11 in this batch.

**`comment_only_source_region` typing was attempted in this batch but DEFERRED** — blocked by task #38 (parens-grouped-Or trailing-annotation attribution bug). The rule's two `( a | b )` parens-grouped Or expressions cause the trailing `-> ...` annotation to fail to register on the rule. Annotation reverted; this rule's typing is gated on task #38's resolution OR a grammar refactor that flattens the parens-grouped Ors into named helper rules.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.6 / Contract 1.0.6 Highlights" — has the per-rule annotation source code + consumer dispatch recipe.

### 1.0.5 / Contract 1.0.5 — SV-Slice-5: `compiler_directive` transparent passthrough (clean directive text)

**What changed:** `compiler_directive := trivia /` `` `[^\r\n]*/`` `` `(line 226 of `grammars/systemverilog.ebnf`) annotated with `-> $2`. Drops the leading `trivia` slot and emits just the matched directive text as a clean JSON string. Consumer code receives a directly-usable string for `source_text_item.body` when `source_text_item.kind == "compiler_directive"`.

**Empirical pre/post for an input with `` `define FOO bar `` + `module m; endmodule\n`:**

```text
# Pre — body was raw envelope of `trivia regex_capture`:
"source_text": [
  {"kind": "compiler_directive", "body": [<trivia envelope>, "`define FOO bar"]}
]

# Post — body is the clean directive string:
"source_text": [
  {"kind": "compiler_directive", "body": "`define FOO bar"}
]
```

**Annotation inventory:** 20 entries (was 19). New: `compiler_directive`.

**Heterogeneous body shape per `source_text_item.kind`** — when kind is `"description"`, body is a typed object; when kind is `"compiler_directive"`, body is a string. Same pattern regex AST uses for typed atoms.

**Schema version:** stays at `1`.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.5 / Contract 1.0.5 Highlights".

### 1.0.4 / Contract 1.0.4 — SV-Slice-4: `description` per-branch typed (`kind:` discriminator on 8 branches; `attribute_instance*` preserved)

**What changed:** 8 per-branch annotations on `description` (line 957 of `grammars/systemverilog.ebnf`). Every Or branch now emits a typed object with a `kind:` discriminator. The two multi-element branches (`attribute_instance* package_item` / `attribute_instance* bind_directive`) preserve the leading attribute_instance* prefix as a separate `attributes:` field while keeping the inner construct as `body:`.

```ebnf
description := module_declaration                 -> {kind: "module_declaration", body: $1}
             | udp_declaration                    -> {kind: "udp_declaration", body: $1}
             | interface_declaration              -> {kind: "interface_declaration", body: $1}
             | program_declaration                -> {kind: "program_declaration", body: $1}
             | package_declaration                -> {kind: "package_declaration", body: $1}
             | attribute_instance* package_item   -> {kind: "package_item", attributes: $1, body: $2}
             | attribute_instance* bind_directive -> {kind: "bind_directive", attributes: $1, body: $2}
             | config_declaration                 -> {kind: "config_declaration", body: $1}
```

**Two layers of typed dispatch end-to-end** — `source_text_item.kind` (SV-Slice-3) routes to which top-level slot the item came from; `description.kind` (this slice) routes to which specific construct when the outer kind is `"description"`.

**Empirical pre/post for `module m; endmodule\n`:**

```text
# Pre — body field of the description-kind source_text_item was raw envelope:
"source_text": [
  {"kind": "description", "body": [<description Or-of-8 raw envelope>]}
]

# Post — body is itself a typed object with its own kind discriminator:
"source_text": [
  {
    "kind": "description",
    "body": {
      "kind": "module_declaration",
      "body": [<module_declaration envelope>]
    }
  }
]
```

**Consumer dispatch unlocked at the description level:**

```rust
for item in obj["source_text"].as_array().unwrap() {
    if item["kind"] == "description" {
        let desc = &item["body"];
        match desc["kind"].as_str().unwrap() {
            "module_declaration"    => walk_module(&desc["body"]),
            "udp_declaration"       => walk_udp(&desc["body"]),
            "interface_declaration" => walk_interface(&desc["body"]),
            "program_declaration"   => walk_program(&desc["body"]),
            "package_declaration"   => walk_package(&desc["body"]),
            "package_item"          => walk_package_item(&desc["attributes"], &desc["body"]),
            "bind_directive"        => walk_bind_directive(&desc["attributes"], &desc["body"]),
            "config_declaration"    => walk_config(&desc["body"]),
            other => panic!("unknown description kind: {}", other),
        }
    }
}
```

**Annotation inventory:** 19 entries (was 11). 8 new per-branch entries on `description`.

**Inner `module_declaration`, `udp_declaration`, etc. shapes still raw envelope** — per-rule typing of those is a follow-up slice.

**Schema version:** stays at `1` (additive discriminator).

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.4 / Contract 1.0.4 Highlights".

### 1.0.3 / Contract 1.0.3 — SV-Slice-3: `source_text_item` per-branch typed (`kind:` discriminator)

**What changed:** 8 per-branch annotations on `source_text_item` (lines 210-217 of `grammars/systemverilog.ebnf`). Every Or branch now emits a typed object with a `kind:` discriminator: `"description"`, `"local_parameter_declaration"`, `"parameter_declaration"`, `"package_import_declaration"`, `"timeunits_declaration"`, `"compiler_directive"`, `"comment_only_source_region"`, `"semi"`.

**Empirical pre/post for `module m; endmodule\n`:**

```text
# Pre — source_text[0] was the matched-branch shape directly:
"source_text": [
  [<description envelope>]
]

# Post — source_text[0] is a typed object with discriminator:
"source_text": [
  {"kind": "description", "body": [<description envelope>]}
]
```

**Consumer dispatch pattern:**

```rust
for item in obj["source_text"].as_array().unwrap() {
    match item["kind"].as_str().unwrap() {
        "description" => walk_description(&item["body"]),
        "local_parameter_declaration" => walk_local_param(&item["body"]),
        "parameter_declaration" => walk_param(&item["body"]),
        "package_import_declaration" => walk_package_import(&item["body"]),
        "timeunits_declaration" => walk_timeunits(&item["body"]),
        "compiler_directive" => walk_compiler_directive(&item["body"]),
        "comment_only_source_region" => walk_comment_region(&item["body"]),
        "semi" => { /* stray ; — nothing to walk */ }
        other => panic!("unknown source_text_item kind: {}", other),
    }
}
```

**Annotation inventory:** 11 entries (was 3). 8 new per-branch entries on `source_text_item`.

**Trailing `semi` dropped** in the `local_parameter_declaration semi` and `parameter_declaration semi` branches — annotations reference `$1` only.

**`@branch_policy: priority_first` and `@priority` preserved** in the rule definition.

**Schema version:** stays at `1` (additive discriminator).

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.3 / Contract 1.0.3 Highlights".

### 1.0.2 / Contract 1.0.2 — SV-Slice-2: `source_text` flatten-spread

**What changed:** `grammars/systemverilog.ebnf` line 2273's `source_text := source_text_item*` rule annotated `-> [$1**]`. The `source_text` field of `systemverilog_file` is now a flat array of `source_text_item` shapes (was a Quantified envelope).

**Empirical pre/post for `module m; endmodule\n`:**

```text
# Pre — source_text was nested Quantified envelope:
{
  "type": "systemverilog_file",
  "source_text": [<Quantified iteration wrap>]
}

# Post — source_text is a flat array (length 1 for minimal_module):
{
  "type": "systemverilog_file",
  "source_text": [<source_text_item shape>]
}
```

**Annotation inventory:** 3 entries (was 2). New: `source_text`.

**Annotation idiom:** `[$1**]` is the canonical flatten-spread form (same as regex.ebnf's `concatenation = piece+ -> [$1**]`). Verified to work for the SV grammar's first array-shaped rule.

**Schema version:** stays at `1` (additive — flat-array shape is a clean-up of the raw envelope).

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.2 / Contract 1.0.2 Highlights".

### 1.0.1 / Contract 1.0.1 — SV-Slice-1: `systemverilog_file` typed (dangling annotation rescued)

**What changed:** `grammars/systemverilog.ebnf` line 184's `systemverilog_file` rule now carries its return annotation on the same multi-line definition (was dangling between the `sv_multi_entry_root` helper rule and `systemverilog_parseable_file`). The annotation `-> {type: "systemverilog_file", source_text: $2}` now correctly latches onto `systemverilog_file`. Same slice removed the `//` prefix from `systemverilog_parseable_file`'s annotation (PGEN's EBNF dialect uses `#` for comments, not `//`, so the `//` prefix was misleading rather than effective).

**Empirical pre/post for `module m; endmodule\n`:**

```text
# Pre-SV-Slice-1 — recursive envelope:
{"content": {"Sequence": [
    {"content": {"Alternative": ...}, "rule_name": "element_0", ...},
    {"content": {"Alternative": ...}, "rule_name": "element_1", ...},
    ...
]}}

# Post-SV-Slice-1 — typed object at root:
{"content": {"Json": {
    "type": "systemverilog_file",
    "source_text": [...]
}}}
```

**Annotation inventory** (from `ast_pipeline`'s reporting): 2 entries (was 1). New: `systemverilog_file`. Existing: `systemverilog_parseable_file` (was already registered via the misleading `//` prefix; now registered via the documented path).

**Manifest update:** `rust/test_data/ast_shape_contract/systemverilog_v1.json` `current_content_kind` updated from placeholder `"sequence"` to calibrated `"json_object"`. Drift status flipped to `calibrated_2026_05_04`. Layout note about line 195 dangling annotation removed (resolved). Calibration history block added.

**Schema version:** stays at `1` (additive shape change within major version 1).

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.1 / Contract 1.0.1 Highlights".

### 1.0.0 / Contract 1.0.0 — Foundation baseline (mdbook + contract Highlights structure)

**What changed:** Initial systemverilog mdBook scaffolded at `docs/systemverilog_parser_book/`. The integration contract `docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md` was upgraded from a thin "stable surface" pointer to the same release-tracked Highlights structure used by the regex parser contract.

**Mdbook chapters landed:** welcome, quickstart, build-recipe, public-api, ast-envelope, parse-content-variants, json-carrier, walking-the-ast, rules-top-level, examples-minimal-module, schema-versioning, glossary, changelog-index. Per-rule and per-feature example chapters land as the annotation campaign progresses.

**Contract section:** [`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`](../../contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md) → "Release 1.0.0 / Contract 1.0.0 Highlights".

**Build status:** Generated SV parser is **NOT in default `cargo test` build** — produced on-demand by `sv_stimuli_quality_gate`. See [Build Recipe](build-recipe.md).

**Annotation campaign:** Not yet started. `grammars/systemverilog.ebnf` is un-annotated except for one commented-out trial annotation at line 200. First slice will land in a follow-up commit.

**Schema baseline:** `1` (corresponds to `version: 1` in `rust/test_data/ast_shape_contract/systemverilog_v1.json`).

**Public API surface:** Unchanged. See [Public API Surface](public-api.md).

**Bug ledger:** No SV-NNNN entries blocking the baseline.

## How to track per-slice changes

Each annotation slice gets:

1. A grammar change in `grammars/systemverilog.ebnf` (the `-> ...` annotation).
2. A manifest update in `rust/test_data/ast_shape_contract/systemverilog_v1.json`.
3. A parser-release / contract-version bump in `docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`.
4. A row in [Schema Versioning](schema-versioning.md) tagging the new schema version.
5. An entry in this changelog index summarising the slice.
6. A regression-lock test in `rust/src/embedding_api.rs` (or related test module) pinning the typed shape.

Per-slice commits should bundle all six (the live-book policy). See `docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md` for an example of a mature contract with 50+ Highlights sections to mirror.
