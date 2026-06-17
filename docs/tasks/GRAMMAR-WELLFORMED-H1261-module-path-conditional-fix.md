# GRAMMAR-WELLFORMED.H.12.6.1 — `module_path_conditional_expression` producer left-recursion fix

The ONE genuine `no_path` producer-wiring defect routed by the `H.12.6` LRM-grounded
re-audit. A **tools-first WHY+WHERE** followed by a **TARGETED, LRM-faithful grammar fix**
(`PGEN-GRAMMAR-WELLFORMED-0111`, **GRAMMAR FIX**, SV release `1.0.142 → 1.0.143`, AST-dump
schema stays `4`, ledger `SV-0005`). The lead leaf of the director's BINDING 2026-06-17
priority order (lane 1: SV `UNKNOWN`→0). It **retires the LAST `no_path` budget case** — the
linter's headline *"no unreachable rules"* is now **literally true** for SystemVerilog
(`unreachable_rules=0`).

> Status: `done`. Owned by the SV `UNKNOWN`→0 lane (`GRAMMAR-WELLFORMED.H.12`). Read with
> [[feedback_no_rule_deletion_without_lrm_proof]] (fix the producer, never delete), the
> attribution rule (reach-failure = generator deficiency OR ill-formed EBNF), and
> [[project_ebnf_is_single_source_of_truth]].

## WHY + WHERE (tools-first, decisive — never guessed)

Baseline (reproduced, deterministic ⇒ signal):
`total=1291 proof=1 witness=1204 UNKNOWN=86` (`--report-certificate-coverage
--grammar-profile sv_2017 --entry-rule systemverilog_file --count 40 --seed 0`);
`module_path_conditional_expression` (mpce) is in **both** the 20-`no_path` set and the
86-`UNKNOWN` set.

The grammar (`grammars/systemverilog.ebnf:3109/3117`) had:

```
module_path_conditional_expression := module_path_expression question attribute_instance* module_path_expression colon module_path_expression
module_path_expression := module_path_conditional_expression -> {kind:"conditional", body:$1}
                       | module_path_expression_operand ( binary_module_path_operator attribute_instance* module_path_expression_operand )* -> {kind:"chain", first:$1, rest:$2}
```

mpce's FIRST token is `module_path_expression`, whose FIRST branch is mpce ⇒ **indirect
left-recursion**. The evidence chain (all tools):

1. **`--dump-gen-ast` (transformed grammar):** the left-recursion eliminator rewrote
   `module_path_expression` into `module_path_expression_lr_base module_path_expression_lr_suffix*`
   (where `_lr_suffix = question attrs module_path_expression colon module_path_expression`) and
   **rewrote `module_path_conditional_expression` into an unreferenced seed**. A reference-graph
   walk over the gen-AST confirmed **0 rules reference mpce** (orphan).
2. **Generated-parser grep:** `parse_module_path_conditional_expression` is DEFINED (~line 484668)
   but has **no external caller** — *defined-but-never-called*.
3. **Genuineness oracle** (`parseability_probe --parse-dump-ast-pretty` on
   `module m; specify if (a ? b : c) (x => y) = 1; endspecify endmodule`): the ternary parses
   (via `_lr_suffix`) but emits **0 `conditional` nodes** and leaks **3 `wrapper_specs`** (raw
   internal LR-elimination annotation-template metadata) into the AST.

So the grammar's comment claim ("PGEN runtime RecursionGuard handles it … the gate reserves
`max_unreachable_rules <= 1` for this budget case") was **disproven**: the rule isn't reached via
recursion-guarding — it's orphaned by elimination, its parse fn never called, and the emitted
output for a ternary module-path is malformed.

## The fix (surgical, grammar-only, LRM A.8.3-faithful — changed ONLY mpce)

```
module_path_conditional_expression := module_path_expression_operand ( binary_module_path_operator attribute_instance* module_path_expression_operand )* question attribute_instance* module_path_expression colon module_path_expression
                                   -> {condition: {kind: "chain", first: $1, rest: $2}, attributes: $4, then_expr: $5, else_expr: $7}
```

mpce's condition is now the non-left-recursive operand chain (the exact body the `chain` branch
already uses + witnesses; `binary_module_path_operator` is a named op rule, so the proven op-chain
`rest:$2` extraction holds). `module_path_expression` is **unchanged in source** (`mpce | chain`)
but is now **natively non-left-recursive** (both branches start with the operand chain), so the
eliminator no longer fires → `module_path_expression_lr_base`/`_lr_suffix` disappear, mpce is
positively referenced by mpe branch 1, and the `wrapper_specs` leak is gone. The accepted language
is identical: a ternary's condition is the lower-precedence binary chain; `then`/`else` stay full
`module_path_expression` (positions after `question`, so no LR) ⇒ right-associative ternary, exactly
IEEE 1800 Annex A.8.3. The stale "budget case" comment block (former grammar lines 82-94) was
rewritten to a "RETIRED" note same-edit.

## A/B (decisive, deterministic seeds 0/7/42 — the parser is the judge)

- Cert `UNKNOWN 86 → 84` (mpce now WITNESSED + `module_path_expression_lr_suffix` removed;
  `witness 1204`; `total 1291 → 1289`; `spf=0`, `proof_reverify_failures=0`).
- `no_path 20 → 19` — verbatim diff: the post-fix set is **exactly** the baseline minus mpce; the
  19 LRM-legitimate rules (`library_text` family, SV-2023 interface-class family, decomposition
  artifacts — all adjudicated in `H.12.6`) are untouched.
- Genuineness oracle post-fix: **1 clean `conditional` node, 0 `wrapper_specs`**;
  `parse_module_path_conditional_expression` now called externally.
- Targeted forms parse: `if (a)`, `if (a & b)`, `if (a ? b : c)`, `if (!a ? b & c : d)`, bare path.
- **SV external corpus 14/14** (`parse_fail_total=0`); `stimuli_cross_family_platform_gate` PASS;
  syntax-closure gate PASS (contract **v6**); clippy source-clean (generated non-strict).

## Lockstep (same commit)

- `grammars/systemverilog.ebnf` — the mpce rule + the retired comment block.
- `rust/test_data/grammar_quality/systemverilog_syntax_closure_contract.json` — **v5 → v6**:
  `max_unreachable_rules 1 → 0`, `blessed_unreachable_rules.lrm_mutual_recursion_cases` emptied;
  the no-regression floors (`min_total_rules 1403`, `min_reachable_rules 1404`,
  `max_unreachable_branches 2`) unchanged (actuals `defined=reachable=1405` stay above them).
- Release `1.0.143` / schema `4` (no bump — the prior leaking conditional was never a realized
  clean shape; common chain byte-identical), ledger `SV-0005`, integration contract Highlights.
- SV parser book: `changelog-index.md` + `schema-versioning.md`.
- Top-level book `docs/book/src/grammar-wellformedness.md` (the `no_path` worked-example + SV arc).
- `CHANGES.md`, `DEVELOPMENT_NOTES.md`, `MEMORY.md`, `LIVE_ACHIEVEMENT_STATUS.md`, this tree.

## Decisions

- `2026-06-17`: the `H.12.6` re-audit's 1 genuine producer-wiring suspect is FIXED by the standard
  non-left-recursive conditional-suffix transformation (condition → operand chain), **never a
  deletion** ([[feedback_no_rule_deletion_without_lrm_proof]]); the AST `wrapper_specs` leak is a
  bonus real-bug fix (ledger `SV-0005`).
- `2026-06-17`: schema stays `4` — the conditional module-path's prior `wrapper_specs`-leaking form
  was never a realized clean shape a consumer could build against (the `-0118`/`-0119`/`-0121`
  "release bump, no schema bump" precedent); the common chain module-path is byte-identical.

## Artifacts (scratch — not tracked)

`/tmp/mpce/` — `cond.sv` (`if (a ? b : c)`), `baseline.sv`, `mp_forms.sv`, `baseline_cert.txt`,
`post_cert.txt`, `sv_gen_ast.json` (the transformed-grammar dump pinning the orphan).
