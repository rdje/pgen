# `ENGINE-UNIVERSAL-SERVICES.13` slice 4 — the indirect-LR SURVEY, measured on every grammar

The instrument: `ast_pipeline <grammar> --report-indirect-lr-plan` (TOOLBOX §1.9b), pure analysis
over the post-elimination gen-AST. `PGEN_INDIRECT_LR_DUMP_ALL=1` prints every route, site and
declined cycle; `--indirect-lr-plan-json FILE` writes the machine-readable form.

Owning leaf: `docs/tasks/ENGINE-UNIVERSAL-SERVICES.md` `.13` slice 4. Re-run everything here with:

```bash
OUT=docs/tasks/artifacts/engine_universal_services/indirect_lr/survey
for g in grammars/*.ebnf; do
  rust/target/debug/ast_pipeline "$g" --report-indirect-lr-plan \
    --indirect-lr-plan-json "$OUT/$(basename "$g" .ebnf).json"
done
```

## Why a survey slice at all

Acceptance (c)/(d) is one transformation with one load-bearing free variable — **which rule absorbs
the chain**. Slice 3 measured that getting it wrong is a regression, not a near-miss: eliminating at
`casting_type` turned the accepted `int'(3)` into a rejection
([`../README.md`](../README.md), probe P2). It also named the answer as *"the consumer rule"* — but
that is a description of two hand-written six-rule synthetics, not something an engine can apply to
a 1 481-rule grammar. This slice turns it into a mechanical criterion and then runs it.

## The criterion, and the two times the measurement corrected it

A candidate base rule `X` is **STARVED** when some rule holds `X` at its left corner with a
**non-empty residual** — `cast_expr := ct "'" "(" lit ")"` is exactly that shape — because PGEN's
`*` is greedy and never retries at a lower iteration count, so the eliminated `X` can swallow text
the holder still needs. Otherwise it **MAY-ABSORB**.

Two refinements, each forced by a measurement rather than by argument:

1. ⛔ **An on-route holder is NOT exempt.** The first draft reasoned that a holder on the
   candidate's own cycle is fine because the rewrite shears the cycle edge — it shears the *clone*
   and leaves the original standing. With that exemption the survey called `ct` `MAY-ABSORB`, and
   `ct` is precisely the rule slice 3 measured as the regression.
2. ⭐ **A holder that the rewrite makes UNREACHABLE cannot starve anything.** `grammars/ebnf.ebnf`
   forced this one: `arithmetic_return := return_expression arithmetic_operator return_expression`
   reads as a hazard for `return_expression`, but it is named only from `expression_return`, which
   is named only from the very alternative the rewrite replaces with a clone. The whole holder chain
   dies with the rewrite. Without this the survey rejected `return_expression` — the knot slice 1
   named for all three `ebnf` cycles.

Both are re-checked by unit tests in `rust/src/ast_pipeline/indirect_lr_plan.rs`, against P1's
grammar, so the criterion cannot silently drift back.

## Census — every grammar in `grammars/`

| grammar | cycle rows (lint) | covered by a route | candidates | starvation-safe | declined |
|---|---|---|---|---|---|
| `systemverilog` | 30 | **30** | 12 | **5** | 18 |
| `systemverilog_lrm_profiled_wrapper` | 23 | 18 | 11 | **7** | 12 |
| `ebnf` | 5 | **5** | 2 | **2** | 3 |
| every other buildable grammar | 0 | — | 0 | 0 | 0 |

`systemverilog_2017_lrm_extracted`, `systemverilog_2023_lrm_extracted`,
`systemverilog_lrm_profiled_generated` and `verilog_2005_lrm_extracted` do not load standalone and
are outside the census, exactly as in slice 1's headline census — the numbers above reproduce it
row for row (30 / 23 / 5 / 0), which is the cross-check that this instrument counts the same
population the lint does.

Raw output: [`census.txt`](census.txt), per-grammar `*.json`, and the full
`PGEN_INDIRECT_LR_DUMP_ALL=1` transcripts `systemverilog.dumpall.txt`,
`systemverilog_lrm_profiled_wrapper.dumpall.txt`, `ebnf.dumpall.txt`.

## ⭐⭐ The finding that moves the fix's target rule AGAIN

Slice 3 concluded the target was `constant_primary`. On the real grammar it cannot be:

```text
constant_primary [no_acyclic_seed]: constant_primary -> constant_primary_sv_2017
                                    -> constant_cast -> casting_type -> constant_primary
```

`constant_primary := constant_primary_sv_2017 | constant_primary_sv_2023` — **both** alternatives
reach the cycle, so the rule has no seed to start a chain from and `X := X_base ( suffix )*` has no
`X_base`. The synthetic could not show this because its one compression — collapsing SystemVerilog's
two-hop bare-reference chain to one hop — is exactly the hop where the dialect split lives. P1's
README called that compression harmless (*"Nothing on the cycle's shape changes"*), and for
reproducing the DEFECT it was; for choosing the BASE RULE it was not.

The target is one rule deeper, and the survey derives its suffix from the shipped grammar:

```text
[candidate] constant_primary_sv_2017   routes=5  seeds=13  clone_cost=13  verdict=MAY-ABSORB
  route alt#11: constant_primary_sv_2017 -> constant_cast -> casting_type -> constant_primary
                -> constant_primary_sv_2017
                suffix: tick lparen constant_expression rparen
```

That suffix is the text `'(3)` in `int'(2)'(3)` and `'(1)` in `8'(1)` — the two victims this leaf
owns — recovered from `grammars/systemverilog.ebnf` with no hand-written synthetic in the loop.
`constant_primary_sv_2023` is the identical row for the other dialect (`routes=5`, `seeds=14`).

## ⛔ ANTLR4's blow-up objection, finally priced against this repository

Slice 2 recorded that ANTLR4 **refuses** indirect left recursion, calling classical elimination
*"wholly unworkable in practice"*, and slice 3 answered "one clone per intermediate on the cycle
path, not an exponential closure". Both were reasoning about a 3-rule and a 6-rule example. The
real numbers, per knot:

| knot | safe base rule(s) | routes | clone cost |
|---|---|---|---|
| cast / call (SV-1/2/3/5) | `constant_primary_sv_2017`, `constant_primary_sv_2023` | 5 | **13** |
| method-call receiver | `method_call_receiver_sv_2017`, `method_call_receiver_sv_2023` | 4 | **12** |
| class scope (SV-4) | `incomplete_class_scoped_type_sv_2023` | 1 | **1** |
| property (SV-6/7) | — **none** | 40 | 6 |
| `ebnf` return-expression | `return_expression`, `expression_return` | 3 | 4 |

So the honest figure is **13 clone rules for SystemVerilog's biggest knot**, not the 2 the synthetic
suggested — because the real cycle is 13 rules long (`constant_function_call -> call_primary ->
call_with_postfix_chain -> chainable_call_initial -> direct_callable_method_call -> method_call_root
-> method_call_receiver -> method_call_receiver_sv_2017 -> cast -> casting_type -> …`), not the
4-rule cycle the lint prints first. Linear in the cycle, and every one of those 13 is a **new rule
name in the typed AST**, which is the `ast_shape_contract` obligation slice 3 opened.

## ⛔⛔ The property knot has NO starvation-safe base rule

`prop_primary_sv_2017` and `prop_primary_sv_2023` are the only candidates on SV-6/SV-7, and both are
STARVED — by `prop_and_sv_2017 := prop_primary_sv_2017 kw_and prop_and_sv_2017`, a holder that stays
reachable after any rewrite. So the transformation this leaf is building **cannot close SV-6/7**, and
that is a measured limit, not a guess.

⭐ It is knot-specific, not shape-specific: the same construct in the RAW Annex A transcription
(`systemverilog_lrm_profiled_wrapper`) has `property_expr_sv_2017` / `property_expr_sv_2023` as
MAY-ABSORB candidates at `clone_cost=1`. The difference is the shipped grammar's hand-written
`prop_and`/`prop_or`/`prop_iff`/`prop_until` precedence cascade — scar tissue of exactly the kind
`.2`'s census exists to find, and the reason the engine cannot help here.

## The survey's own coverage gap, itemised

The walk only follows a **bare leading rule reference**, so a cycle closing through a nullable
prefix, a quantifier or a group is declined rather than guessed. Measured:

- `systemverilog` and `ebnf`: **0** such declines — 30/30 and 5/5 rows are covered.
- `systemverilog_lrm_profiled_wrapper`: **5**, all one knot (`module_path_expression ->
  module_path_expression_lr_base -> module_path_expression_operand -> module_path_primary ->
  module_path_concatenation`), and note that the knot passes through `module_path_expression_lr_base`
   — a rule the EXISTING elimination pass created.

Every other decline is `no_acyclic_seed` (18 in SV, 7 in the wrapper, 3 in `ebnf`): a rule all of
whose alternatives reach the cycle, which is the `constant_primary` case above and is a statement
about where the chain may be absorbed, not a gap in the walk.

## Files

| file | what it is |
|---|---|
| `census.txt` | the per-grammar table above, as emitted |
| `<grammar>.json` | machine-readable survey: routes, suffixes, clone cost, starvation sites |
| `<grammar>.dumpall.txt` | full `PGEN_INDIRECT_LR_DUMP_ALL=1` transcript for the three non-zero grammars |
