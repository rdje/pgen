# GRAMMAR-WELLFORMED.H.12.6 — `no_path` LRM-grounded re-audit + no-deletion policy

Tools-first, **LRM-grounded** re-adjudication of the SystemVerilog cert-coverage `no_path`
set, prompted by the director's emphatic 2026-06-17 directive that **no rule is deleted
unless the LRM objectively proves it has no business in the grammar**, and that a `no_path`
verdict is first read as a *producer-wiring flaw to fix*, not a removal license. This slice
(`PGEN-GRAMMAR-WELLFORMED-0108`, **PURE-DOCS** — tracker + policy, no code/grammar change)
records the policy ([[feedback_no_rule_deletion_without_lrm_proof]]), the per-rule LRM
verdict for every `no_path` rule, and routes the single genuine suspect to a fix leaf.

> Scope: layer-B detail behind a new `H.12.6` row. Owned by the SV `UNKNOWN`→0 lane
> (`GRAMMAR-WELLFORMED.H.12`). Read with [[feedback_unreachable_target_attribution_rule]]
> (reach-failure = generator deficiency OR ill-formed EBNF, grammar-first) and
> [[project_ebnf_is_single_source_of_truth]].

## What `no_path` means (the definition this re-audit rests on)

`no_path` is a **structural-reachability** verdict, distinct from `UNKNOWN`: cert-coverage
computes a shortest reach path from the **chosen entry rule** to the target; if none exists
*from that entry*, the rule is flagged `no_path` ("no reach path from the entry — dead-rule
candidate, adjudicate via the linter"). It is **not** "the parser can't parse it" and
**not** a deletion signal. Per the policy, a `no_path` resolves to exactly one of:
(1) the rule is rooted under a **different LRM start symbol**; (2) it is **profile-relative**
(a later-edition feature, inert under this profile); (3) it is an LRM-extraction
**decomposition artifact** (synthetic helper / keyword leaf); or (4) a **producer-wiring
flaw** — the rule the LRM says should lead to it is mis-wired → **FIX the producer**.
Only LRM-proven-absence (none of 1–4, no LRM production, no value) ever permits deletion.

## Baseline (reproduced this session — deterministic ⇒ signal)

```
# entry=systemverilog_file, sv_2017 — 20 no_path of 86 UNKNOWN
PGEN_CERT_COVERAGE_DUMP_ALL=1 ./target/debug/ast_pipeline ../grammars/systemverilog.ebnf \
  --report-certificate-coverage --grammar-profile sv_2017 --entry-rule systemverilog_file \
  --count 40 --seed 0
# total=1291 proof=1 witness=1204 UNKNOWN=86 (no_path WARNING lists 20)
```

**Two independent narrowing runs proved the `no_path` set is entry/profile-relative, not dead:**

- `--entry-rule sv_multi_entry_root` (the umbrella over the LRM start symbols): `no_path`
  **20 → 9**. The 11 that gained a reach path — `systemverilog_parseable_file`,
  `parseable_source_item`, `library_text`, `library_declaration`, `library_description`,
  `include_statement`, `kw_library`, `kw_include`, `kw_incdir` (+ shells) — are reachable
  from the LRM's separate `library_text` start symbol (IEEE 1800-2017 §33 / Annex A.1.1).
- `--grammar-profile sv_2023`: the interface-class family
  (`interface_class_declaration`/`_item`/`_method`, `declared_interface_class_identifier`)
  + `class_constructor_super_args` + `union_modifier` leave the residual — they **witness
  under SV-2023** (genuine 1800-2023 features, correctly inert under `sv_2017`).

## The 20-rule LRM-grounded verdict table

Citations verified against `grammars/systemverilog_2017_lrm_extracted.ebnf`,
`docs/systemverilog/2017/grammar_clean.ebnf`, and the MD workspace
`docs/systemverilog/{2017,2023}/md/`. The IEEE LRM **PDFs** are now vendored locally
(git-ignored, `-0109`) next to their MD workspaces —
`docs/systemverilog/2017/SystemVerilog-LRM-IEEE-1800-2017.pdf`,
`docs/systemverilog/2023/SystemVerilog-LRM-IEEE-1800-2023.pdf`,
`docs/vhdl/2019/VHDL-LRM-IEEE-1076-2019.pdf`,
`docs/verilog/2005/Verilog-LRM-IEEE-1364-2005.pdf` — so page-level PDF citations are now
possible; the tracked, citable ground truth remains the MD workspaces + extracted EBNF.

| # | rule | LRM status | LRM ref | producer / root start-symbol | verdict | action |
|---|---|---|---|---|---|---|
| 1 | `sv_multi_entry_root` | PGEN-synthetic | n/a | the analysis umbrella over the 3 entries | decomposition artifact | STAYS (synthetic entry) |
| 2 | `systemverilog_parseable_file` | LRM-adjacent entry | A.1.x | alternate parseable entry shell | rooted-under-different-start-symbol | STAYS |
| 3 | `parseable_source_item` | LRM-adjacent | A.1.x | child of `systemverilog_parseable_file` | rooted-under-different-start-symbol | STAYS |
| 4 | `class_constructor_super_args` | yes (2023) | A.1.x (2023) | `class_constructor_declaration_sv_2023` (`super.new(default)`) | profile-relative SV-2023 | STAYS (witnesses sv_2023) |
| 5 | `declared_interface_class_identifier` | yes (2023) | §8 / A.1.x (2023) | interface-class decl | profile-relative SV-2023 | STAYS (witnesses sv_2023) |
| 6 | `include_statement` | yes | §33 / A.1.1 | `library_description` → `library_text` | rooted-under-`library_text` | STAYS |
| 7 | `interface_class_declaration` | yes (2023) | §8 / A.1.x (2023) | `package_item`/`checker_or_generate_item` (2023) | profile-relative SV-2023 | STAYS (witnesses sv_2023) |
| 8 | `interface_class_item` | yes (2023) | §8 / A.1.x (2023) | `interface_class_declaration` | profile-relative SV-2023 | STAYS (witnesses sv_2023) |
| 9 | `interface_class_method` | yes (2023) | §8 / A.1.x (2023) | `interface_class_item` | profile-relative SV-2023 | STAYS (witnesses sv_2023) |
| 10 | `library_declaration` | yes | §33 / A.1.1 (`grammar_clean.ebnf`) | `library_description` → `library_text` | rooted-under-`library_text` | STAYS |
| 11 | `library_description` | yes | §33 / A.1.1 | `library_text` | rooted-under-`library_text` | STAYS |
| 12 | `library_text` | yes (start symbol) | §33 / A.1.1 (`section-33-…:131`) | **separate LRM start symbol** | rooted-under-`library_text` | STAYS |
| 13 | **`module_path_conditional_expression`** | **yes** | **A.8.3 (`…_2017_lrm_extracted.ebnf:681`)** | **`module_path_expression` (mutual recursion)** | **PRODUCER-WIRING FLAW (suspect)** | **FIX → `H.12.6.1`** |
| 14 | `union_modifier` | yes (2023) | A.1.x (2023) | `struct_union_sv_2023` (`union soft`/`tagged`) | profile-relative SV-2023 | STAYS (witnesses sv_2023) |
| 15 | `kw_file_path_spec_…` | keyword leaf | §33 / A.1.1 | `library_declaration`/`include_statement` | rooted-under-`library_text` | STAYS |
| 16 | `kw_incdir_…` | keyword leaf | §33 / A.1.1 | `library_declaration` (`-incdir`) | rooted-under-`library_text` | STAYS |
| 17 | `kw_include_…` | keyword leaf | §33 / A.1.1 | `include_statement` | rooted-under-`library_text` | STAYS |
| 18 | `kw_library_…` | keyword leaf | §33 / A.1.1 | `library_declaration` | rooted-under-`library_text` | STAYS |
| 19 | `kw_n_29_…` | PGEN-synthetic | n/a | LRM clause-number decomposition leaf | decomposition artifact | STAYS (blessed) |
| 20 | `kw_n_48_…` | PGEN-synthetic | n/a | LRM clause-number decomposition leaf | decomposition artifact | STAYS (blessed) |

### Summary
- **10 rooted under the `library_text` start symbol** (IEEE 1800-2017 §33 / Annex A.1.1) — STAY.
- **6 profile-relative SV-2023** (interface-class family + `class_constructor_super_args` + `union_modifier`) — STAY, witness under `sv_2023`.
- **3 LRM-extraction decomposition artifacts** (`sv_multi_entry_root`, `kw_n_29`, `kw_n_48`) — STAY (blessed synthetic).
- **1 genuine producer-wiring suspect** — `module_path_conditional_expression` — **FIX, never delete**.
- **0 deletion candidates.** None clears the LRM-proven-absent bar.

## The 1 action item — `module_path_conditional_expression` (→ `H.12.6.1`, **DONE `PGEN-GRAMMAR-WELLFORMED-0111`** — fixed; SV cert `UNKNOWN 86→84`, `no_path 20→19`, `unreachable_rules=0`, ledger `SV-0005`, release 1.0.143; detail in `GRAMMAR-WELLFORMED-H1261-module-path-conditional-fix.md`)

LRM ground truth (Annex A.8.3, `…_2017_lrm_extracted.ebnf:681`):
`module_path_conditional_expression ::= module_path_expression ? { attribute_instance } module_path_expression : module_path_expression`,
and `module_path_expression` references it — i.e. **left-recursion**. The active grammar
(`grammars/systemverilog.ebnf:3117`) keeps `module_path_expression := module_path_conditional_expression -> {kind:"conditional"} | module_path_expression_operand ( binary_op … )*`:
it eliminated the *binary-op* left-recursion (via `module_path_expression_operand`) but left
the **conditional** alternative as a left-recursive first branch, which a PEG cannot enter →
`module_path_conditional_expression` is stranded. The grammar's own comment (lines 82-94)
rationalizes this as a "blessed LRM mutual-recursion budget case … RecursionGuard handles
it"; this slice **does not accept that rationalization without proof.**

**Tools-first evidence this session (honest, inconclusive — needs a trace):** a `specify`
state-dependent path condition `if (a ? b : c)` (`/tmp/mpce/cond.sv`) **parses**
(`parse_full passed`), BUT the AST genuineness oracle
(`parseability_probe --parse-dump-ast-pretty`) shows **no `"kind":"conditional"` node** —
only `"chain"`/`"primary"` — so the `module_path_conditional_expression` branch **did not
match** (the bytes were absorbed elsewhere). This is consistent with the `no_path` verdict
and with the left-recursion-stranding hypothesis, but does **not** prove unreachability in
all contexts.

`H.12.6.1` scope (PENDING): a `--trace-rules module_path_conditional_expression`-driven
WHY+WHERE to (a) prove whether the conditional branch is ever entered, (b) LRM-ground the
fix as the non-left-recursive conditional-*suffix* idiom
(`module_path_expression := operand_chain ( ? mpe : mpe )?`, the standard PEG elimination,
same accepted language as A.8.3), and (c) land it as a TARGETED grammar fix (consumer-visible
→ regen/release/lockstep ceremony) — **never a deletion**, and retire the "budget case"
comment. Change ONE thing; decisive A/B; GLOBAL cert + `spf` at seeds 0/7/42;
`stimuli_cross_family_platform_gate`; SV external corpus 14/14.

## Verification (this slice)

- Baseline `no_path`=20 reproduced (seed 0). `sv_multi_entry_root` → 9; `sv_2023` drops the 7
  profile rules — both deterministic, byte-consistent with the layer-A pointer.
- LRM citations verified in `…_2017_lrm_extracted.ebnf` / `grammar_clean.ebnf` / MD workspace.
- PURE-DOCS ⇒ NO code/grammar/generated/release/schema/ledger change; SV stays `UNKNOWN=86`.

## Decisions

- `2026-06-17`: standing policy adopted — [[feedback_no_rule_deletion_without_lrm_proof]]
  (no deletion without LRM-proven-absence; `no_path` ⇒ fix the producer first).
- `2026-06-17`: of 20 SV `no_path` rules, 19 adjudicated LRM-legitimate (STAY) and 1
  (`module_path_conditional_expression`) routed to fix leaf `H.12.6.1` — **0 deletions**.

## Artifacts (scratch — not tracked)

`/tmp/mpce/` — `baseline.sv` (`if (a)`), `cond.sv` (`if (a ? b : c)`), `cond.ast.json`
(0 `conditional` nodes — the genuineness oracle).
