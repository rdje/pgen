# GRAMMAR-WELLFORMED.H.12.5.5.3.3.4.2.1.2.2 — context_member GENUINE-witness composition: WHY+WHERE + SAFE DESIGN

Tools-first resolution of the **open composition question** the `-0101` design explicitly handed forward
and the `-0102` FIX-ATTEMPT got wrong (false witness). This slice (`PGEN-GRAMMAR-WELLFORMED-0107`,
**PURE-DOCS**) pins, with the parser as the only judge, (1) the EXACT minimal genuine-witness recipe for
`context_member_method_call`, (2) the two concrete gaps that block it today, and (3) a **safe composition
design** that cannot re-trigger the `-0102` false witness or the `.4.2.1.2.1` deferred soundness gap. It
splits `.4.2.1.2.2` into this DESIGN (`.4.2.1.2.2.1`, done here) + a tightly-scoped IMPLEMENT
(`.4.2.1.2.2.2`, the new frontier).

> Scope: layer-B detail behind the `H.12.5.5.3.3.4.2.1.2.2` row. Opened by `.4.2.1.2` (`-0102`), which
> proved the prelude-alone approach false-witnesses, and `.4.2.1.2.1` (`-0103`), which pinned the engine
> soundness gap (now DEFERRED — a correct-but-wide-blast fix). The director's rhythm on this rule is
> WHY+WHERE/design-first, then a TARGETED fix on tangible proof — not trial-and-revert.
> ([[feedback_research_grounded_sota_no_trial_and_revert]], [[feedback_why_and_where_before_solution]],
> [[feedback_tools_first_no_guessing]], [[feedback_no_codebase_change_without_tool_backed_facts]],
> [[feedback_always_signoff_decisions]], [[feedback_never_edit_generated_artifacts]],
> [[project_cert_coverage_tournament_loser_leak]].)

## Baseline (reproduced this session — deterministic ⇒ signal)

```
./target/debug/ast_pipeline ../grammars/systemverilog.ebnf --report-certificate-coverage \
  --grammar-profile sv_2017 --entry-rule systemverilog_file --count 40 --seed 0
# total=1291 proof=1 witness=1204 UNKNOWN=86 fully_certified=false (sample_parse_failures=0, proof_reverify_failures=0)
```

Byte-identical to the layer-A pointer. `context_member_method_call` is in the 86-rule `UNKNOWN` set
(29 probe lines reference it across the plannable + target-own passes).

## WHY+WHERE 1 — the EXACT minimal genuine-witness recipe (parser is the judge)

`parseability_probe --parse-dump-ast-pretty systemverilog <s> --profile sv_2017`, counting
`context_member_method` AST nodes (set explicitly by `call_primary := context_member_method_call ->
{kind: "context_member_method", …}`, so a committed match shows exactly one):

| input | parses | `context_member_method` nodes |
|---|---|---|
| `int \foo ; (*\foo =+\foo .\foo [0].\foo ()*);` (Test c2) | ✅ | **1 — GENUINE** |
| `module m; int \foo ; int \bar ; initial \bar = \foo .\foo [0].\foo (); endmodule` (Test b) | ✅ | **1 — GENUINE** |
| **`int \foo ; (*\foo =+\foo .\foo .\foo ()*);`** (decl + member + CALL, **no index**) | ✅ | **1 — GENUINE** |
| `(*\foo =+\foo .\foo .\foo ()*);` (CALL but **no decl**) | ✅ | **0** |
| `(*\foo =+\foo .\foo .\foo *);` (bare ref, no decl, no call) | ✅ | **0** |

**The minimal genuine witness needs exactly two ingredients and NOTHING more:**

1. **A top-level declaration of the chain head** (`int \foo ;`) so the parser emits the `variable_binding`
   fact and the `@predicate has_fact(variable_binding, $head) phase: post` gate on
   `context_member_method_call` (`systemverilog.ebnf:2888`) PASSES.
2. **`callable_method_call_body` rendered as a real CALL** (`.\foo()`), i.e. the chain is
   `head ( .member )+ .method()`. `callable_method_call_body := built_in_method_call | method_identifier
   attribute_instance* lparen list_of_arguments rparen` (`systemverilog.ebnf:2797`) — **both branches are
   calls**, so the `()` form is the second branch (`o1`).

The `[idx]` (`constant_bit_select`) is **NOT required** (the no-index row witnesses) — this refines the
`-0101`/`-0102` framing that emphasized `.foo[0].foo()`. The `(dot method_call_body)*` trailing chain is
also not required (min-1 member + one call suffices).

## WHY+WHERE 2 — the two gaps, pinned on the SHIPPED generator (`PGEN_CERT_COVERAGE_DEBUG_PROBES=1`)

Every `context_member_method_call` probe the generator emits today (16 lines; reach path routes through an
`attribute_instance` `(* attr = const_expr *)`):

| pass | sample(s) | parsed | witnessed |
|---|---|---|---|
| plannable | `(*\foo =+\foo .\foo .\foo *)` | true | false |
| target-own (R-own) | `(*\foo =+\foo .\foo .\foo .\foo *)`, `(*p4cSo=+p.TMeZJ.ro.ME*)` | true | false |
| target-own (child: `constant_bit_select`) | `(*\foo =+\foo .\foo [+…].\foo .\foo *)` | true | false |
| target-own (child: `callable_method_call_body`) | `(*\foo =+\foo .\foo .\foo (*…*)().\foo *)` | true | false |

- **Gap A — no binding prelude.** Not a single probe declares `\foo`. SV's `gen_count_kinds` is empty, so
  `compute_reach_prelude` (`stimuli_generator.rs:2776`) returns `None` — SV gets **no prelude of any
  kind**. Without the declaration the `has_fact` post-gate fails ⇒ the rule rejects ⇒ the committed parse
  routes the bytes to a SIBLING (`call_primary`'s lower branches) ⇒ never witnessed.
- **Gap B — structure forced one-child-per-probe, never together, never with a binding.** The
  `.3.3.1` mandatory-child forcing (`generate_target_own_structure_witnesses:3266`) DOES render `[idx]` in
  one probe and a `()` call in a *different* probe — but **never both in one sample, and never alongside a
  declared head.** So no single probe is ever c2-shaped.

**The false-witness landmine (why this is delicate).** A probe that has the binding but renders a
**bare ref** (`int \foo ; (*\foo =+\foo .\foo .\foo *)`) is exactly the `.4.2.1.2.1` reproducer: the
`has_fact` post-gate PASSES, the rule structurally fails, the committed parse omits it (0 AST nodes) — and
the **deferred tournament/transactional-coverage soundness gap credits it as witnessed anyway** (a FALSE
witness). The cert driver's `witness_check` is `parse_and_cover` (gap-affected), so **the `UNKNOWN` count
alone cannot tell a genuine witness from a false one for this predicate-gated rule.** The genuineness
oracle is the `context_member_method` AST node, never the count ([[feedback_corpus_expected_from_spec_not_fix]]).

## WHERE (code) — the precise edit surface (read, not guessed)

- `ReachPrelude` (`stimuli_generator.rs:1123`) is count-MVP-shaped: `site`, `gated_rule`, `sub_plan`,
  `iterations` (0 = disarmed), `captured` (phase-1 numeric capture). It has **no kind discriminator**.
- `compute_reach_prelude` (`:2769`) short-circuits on empty `gen_count_kinds`; its producer scan +
  innermost-on-path-quantifier site selection is exactly what a `has_fact` prelude also needs.
- The plannable driver arms the prelude **only when `prelude.captured` is `Some`** (`:3079-3089`). A
  `has_fact` (Presence) prelude has `captured == None`, so the plannable driver **never arms it** — i.e. a
  Presence prelude is inert in the plannable pass *with no change to that driver*. This is the structural
  reason the safe design below cannot re-create the `-0102` plannable false witness.
- `compute_store_aware_gen_directives` (`:9437`) parses `@emit_fact` + `fact_count_at_least` post-gates
  into `(gen_emit_facts, gen_count_kinds)`. A `has_fact(K, $ref)` post-gate is the exact analogue
  (`SemanticRuntimeDirective::Predicate`, `name == "has_fact"`, `args[0]=Identifier(K)`,
  `args[1]=RuleReference(ref)`).
- `mandatory_child_rules(context_member_method_call)` (`:5356`) = `[identifier, dot, constant_bit_select,
  callable_method_call_body]`; the call-shaped child is `callable_method_call_body` (root `Or`, branch
  `o1` = the `lparen … rparen` form).
- The target-own driver already re-installs the reach plan per child (`:3292`) and forces the child's own
  `Or` branch / inner quantifiers — the hook a Presence-prelude arm + an all-mandatory-children pass can
  attach to.

## DESIGN — the SAFE composition (the `.4.2.1.2.2.2` IMPLEMENT)

Parser-agnostic, keyed purely on grammar structure (`has_fact` post-gate + `@emit_fact` producers +
rule-reference graph + on-path quantifier sites + mandatory-child walk). Inert for the fully-certified
roster (no `has_fact`-gated `UNKNOWN` rule on a reach path; the target-own pass has an empty residual).

1. **Detect `has_fact`-gated rules.** Extend `compute_store_aware_gen_directives` (or a sibling) to also
   return `gen_has_fact_gates: HashMap<String, Vec<(String /*kind*/, String /*name_ref*/)>>`. Does NOT
   set `store_aware_gen` (no count-prune semantics).

2. **Add a `kind: PreludeKind { Count, Presence }` field to `ReachPrelude`.** `Count` = the existing
   two-phase capture/replay. `Presence` = `iterations` armed to `1`, `captured = None` always.

3. **`has_fact` branch in `compute_reach_prelude`.** When no count-gated rule is on the path but a
   `has_fact`-gated rule is, build a `Presence` prelude: producer = first sorted rule whose `@emit_fact`
   emits `K` (= `variable_binding` ⇒ `variable_decl_assignment`); site = innermost on-path quantifier
   whose body graph-reaches the producer (= `source_text := source_text_item*` ⇒ a top-level `int \foo ;`);
   **return it DISARMED (`iterations = 0`)** so the plannable driver leaves it inert (Gap A stays closed in
   the plannable pass → no `-0102` trap).

4. **Arm + compose ONLY in the target-own pass, ONLY on a call-forced probe.** In
   `generate_target_own_structure_witnesses`, for a residual rule that is `has_fact`-gated, add ONE
   composed probe that, on top of `set_reach_plan_for_rule`:
   - **arms** the Presence prelude (`iterations = 1`) → injects the top-level declaration of the head; AND
   - forces **all** mandatory children's non-degenerate structure together (so
     `callable_method_call_body` renders its `o1` CALL branch — the `()`), producing a c2-shaped
     `int \foo ; (*\foo =+\foo .\foo[..].\foo()*)`.
   **The prelude is armed on NO other probe** — never on a bare-ref/R-own probe — so a binding can never
   co-occur with a degenerate render ⇒ the soundness gap cannot manufacture a false witness here.

5. **Name-coupling is free on the canonical path** (producer + head both render `\foo` in construct mode);
   verify in the armed-sample dump rather than assume (a mismatch can only fail loudly, never
   false-witness).

### Acceptance for the IMPLEMENT (`.4.2.1.2.2.2`)

- **GENUINENESS ORACLE FIRST (hard commit gate):** dump the armed sample
  (`PGEN_CERT_COVERAGE_DEBUG_PROBES=1`) and confirm `parseability_probe --parse-dump-ast-pretty` shows
  **≥1 `context_member_method` AST node** on it. The `UNKNOWN` count is necessary but NOT sufficient. If
  0 nodes ⇒ FALSE witness ⇒ revert (the `-0102` discipline).
- SV cert `UNKNOWN 86 → 85` (`context_member_method_call` witnessed GENUINELY), deterministic at seeds
  0/7/42, `spf` unchanged (0).
- Fully-certified roster (rtl_const_expr/json/regex/vhdl/svpp/rtl_frontend) BYTE-IDENTICAL (`UNKNOWN=0`,
  no added pass cost); `stimuli_cross_family_platform_gate` PASS; `clippy_on_rust_change` strict-source
  clean. GENERATOR-ONLY ⇒ no grammar/EBNF/parser-regen/release/schema/ledger change.

## Conclusion

- The genuine witness is REAL and minimal (decl + ≥1 member + CALL; no index) — tool-proven, not assumed.
- The two gaps are pinned; the composition the `-0101` design left open is RESOLVED: **arm a Presence
  prelude only in the target-own pass, only on the all-mandatory-children (call-forced) probe** — which
  structurally cannot reproduce the `-0102` plannable false witness or trip the deferred soundness gap.
- Split: `.4.2.1.2.2.1` (this DESIGN, done) + `.4.2.1.2.2.2` (the GENERATOR IMPLEMENT, frontier next).
- PURE-DOCS ⇒ NO code/grammar/generated/release/schema/ledger change; clippy not invoked; SV stays the
  only non-fully-certified shipped grammar (`UNKNOWN=86`, unchanged).

## Artifacts (this session, scratch — not tracked)

`/tmp/cmmc/` — `c2.sv`/`b.sv`/`a.sv`/`min.sv`/`min_nodecl.sv` + their `*.ast.json` (the witness matrix);
`/tmp/cert_probes.txt` (`PGEN_CERT_COVERAGE_DUMP_ALL=1 PGEN_CERT_COVERAGE_DEBUG_PROBES=1
PGEN_REACH_PATH_DUMP=1` seed-0 cert capture — the 86 UNKNOWN list + every context_member probe).
