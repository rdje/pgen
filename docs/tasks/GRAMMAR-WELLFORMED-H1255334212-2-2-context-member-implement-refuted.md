# GRAMMAR-WELLFORMED.H.12.5.5.3.3.4.2.1.2.2.2 — context_member GENUINE-witness composition: IMPLEMENT ATTEMPT → two design assumptions REFUTED (reverted)

Tools-first execution of the `-0107` safe `Presence`-prelude design (`.4.2.1.2.2.1`). The design was
implemented faithfully, measured with the parser as the only judge, and its two load-bearing
assumptions were **disproven by tool evidence**. The witness did NOT materialize and — critically — NO
false witness occurred (the safety the design was built around held: SV cert stayed `UNKNOWN=86`, the
composed probe did not even parse). Per the `-0102` discipline (commit ONLY improvements; a
predicate-gated rule's genuineness oracle is the AST node, never the `UNKNOWN` count) the implemented
generator code was **REVERTED**; this slice lands **PURE-DOCS** with the refutation + a re-scope.

> Scope: layer-B detail behind the `H.12.5.5.3.3.4.2.1.2.2.2` frontier row. Predecessors: `.4.2.1.2`
> (`-0102`, prelude-alone false witness, reverted), `.4.2.1.2.1` (`-0103`, soundness gap, deferred),
> `.4.2.1.2.2.1` (`-0107`, the safe design this attempt executes). Director rhythm: WHY+WHERE on
> tangible proof, then a TARGETED fix — not trial-and-revert.
> ([[feedback_tools_first_no_guessing]], [[feedback_why_and_where_before_solution]],
> [[feedback_no_codebase_change_without_tool_backed_facts]], [[feedback_always_signoff_decisions]],
> [[feedback_corpus_expected_from_spec_not_fix]], [[project_store_aware_generation]],
> [[project_cert_coverage_tournament_loser_leak]].)

## Baseline (reproduced this session — deterministic ⇒ signal)

```
./target/debug/ast_pipeline ../grammars/systemverilog.ebnf --report-certificate-coverage \
  --grammar-profile sv_2017 --entry-rule systemverilog_file --count 40 --seed 0
# total=1291 proof=1 witness=1204 UNKNOWN=86 fully_certified=false (sample_parse_failures=0)
```

Byte-identical to the layer-A pointer and to the `-0107` baseline.

## What was implemented (faithful to the `-0107` design)

GENERATOR-only, in `rust/src/ast_pipeline/stimuli_generator.rs`:
- a `PreludeKind { Count, Presence }` discriminator on `ReachPrelude`;
- a `gen_has_fact_gates: HashMap<String, Vec<(kind, name_ref)>>` field + `compute_gen_has_fact_gates`
  (sibling of `compute_store_aware_gen_directives`, keyed on `has_fact(K,$ref) phase: post`);
- a `Presence` branch in `compute_reach_prelude` factored with the `Count` branch into a shared
  `build_semantic_prelude(gated_rule, kind, sites, fuel, kind)`, returned DISARMED;
- every count-specific consumer (`reach_prelude_capture` / `_replay_text` /
  `_bypasses_count_prune`, and the plannable-pass arming) guarded on `kind == Count` so a Presence
  prelude is provably inert in the plannable pass;
- a composed probe in `generate_target_own_structure_witnesses` that, for a `has_fact`-gated residual,
  arms the Presence prelude (`iterations = 1`) AND forces every mandatory child's non-degenerate
  structure together, in ONE probe (armed nowhere else).

It compiled clean; the detection chain worked exactly as designed (verified with an env-gated
`PGEN_PRESENCE_DEBUG` diagnostic):
```
[presence] rule='context_member_method_call' witnessed=false child_forcings=3 gate=Some([("variable_binding","head")])
[presence] rule='context_member_method_call' set_ok=true prelude=Some((Presence, 0, ("description","root/o5/s0")))
[presence] rule='context_member_method_call' armed=true
```

## WHY+WHERE — the two refuted assumptions (parser is the judge)

`parseability_probe --parse-dump-ast-pretty systemverilog <file> --profile sv_2017`, counting
`context_member_method` AST nodes (set by `call_primary := context_member_method_call -> {kind:
"context_member_method", …}`):

| input file | parses | `context_member_method` nodes |
|---|---|---|
| `int \foo ; (*\foo =+\foo .\foo .\foo ()*);` (genuine recipe, decl name == head name) | ✅ | **1 — GENUINE** |
| `int \bar ; (*\foo =+\foo .\foo .\foo ()*);` (decl name `\bar` ≠ head `\foo`) | ✅ | **0** |
| the ACTUAL composed probe the generator emitted (below) | ❌ does not parse | n/a |

The composed probe the generator actually emitted:
```
(*\foo =+type(struct{struct{struct{struct{bit\foo ;}\foo ;}\foo ;}\foo ;})*)(*cBN=+CvtU.YB$A1[+M4B4w.f[+8320.2252e87].\foo .\foo ].utk(*Q2=+\foo ::OIVD()*)().uVw*);
```

- **Refuted assumption A — "prelude site = `source_text := source_text_item*`" (file-scope decl).**
  The innermost-first (`quantifier_sites.iter().rev()`) site scan instead picked
  `("description","root/o5/s0")` — an INNER on-path quantifier whose body graph-reaches the producer
  `variable_decl_assignment` **via a struct member** (`struct { bit \foo ; }`). So the injected
  "declaration" is a struct-scoped `variable_binding`, not the file-scope `int \foo ;` the top-level
  `context_member` chain's `has_fact` gate can see. The producer is reachable from MANY sites; only the
  file-scope one emits a fact at the chain's scope, and the innermost-first scan is the wrong selector
  for `Presence`.

- **Refuted assumption B (the real blocker) — "name-coupling is free on the canonical path
  (producer + head both render `\foo`)."** The mismatch row PROVES the `has_fact(variable_binding,
  $head)` gate is **name-sensitive**: declaring `\bar` but using head `\foo` ⇒ the gate fails ⇒ the rule
  is rejected ⇒ **0** AST nodes. And in the actual composed generation the injected declaration rendered
  `\foo` while the on-path chain head rendered `cBN` / `Q` (the prelude injection advances the seeded RNG
  past the deterministic first-identifier `\foo` that the binding-less R-own probes all got). Different
  names ⇒ the gate fails ⇒ no witness. The genuine witness REQUIRES the injected declaration's name to
  EQUAL the on-path chain head's name, and the generator has no mechanism to couple them.

## Safety held (no false witness)

The implementation could not manufacture a false witness: the composed sample did not even parse
(`witness_check` → `NotParsed` → never unioned), and SV cert stayed `UNKNOWN=86` byte-identical. The
`-0102` failure mode (a binding + a degenerate render gap-credited as witnessed) did NOT recur — the
Presence prelude was armed ONLY in the target-own pass, exactly as designed.

## The real blocker = generation-time value-selection (STORE-AWARE-GEN.4b)

Name-coupling between an injected declaration and a later on-path reference is the generation-side dual
of the parser's `has_fact(variable_binding, $head)` gate: the generator must (1) emit a
`variable_binding` fact carrying the name it rendered for the declaration, and (2) make the chain head
CONSULT that fact and render the SAME name. That is precisely the `has_fact`/value-selection
generalization scoped (tools-first) under `STORE-AWARE-GEN.4` as a MEASURED SV effort (`.4b`
value-selection), NOT a quick slice. The Presence file-scope site selection (assumption A) is moot until
B is solved, because both are needed together for a genuine witness.

## Decision + re-scope

- **REVERTED** the generator code; tree restored to baseline (`UNKNOWN=86`, byte-identical).
- `.4.2.1.2.2.2` (this IMPLEMENT attempt) is recorded `refuted`; the genuine `context_member_method_call`
  witness is **BLOCKED on `STORE-AWARE-GEN.4b`** (generation-time name/value selection) + a `Presence`
  file-scope site selector. New leaf `.4.2.1.2.2.3` owns that composed pursuit once the value-selection
  capability exists.
- SV `UNKNOWN`→0 continues on the independently-actionable frontier (`H.12.6.1`
  `module_path_conditional_expression` producer fix; then `H.12.5.6` M2 / `H.12.5.7` M3). context_member
  is parked behind the capability dependency rather than chased with a fragile name-coupling hack.

LESSON (reinforces `-0102`): for a NAME-sensitive predicate-gated rule, a witness needs not just the
fact's PRESENCE but the right SCOPE and the right NAME — and injecting a declaration whose name the
generator cannot couple to the use site cannot witness it. Confirm site + scope + name in the
armed-sample dump; never assume name-coupling is free.

## Artifacts (this session, scratch — not tracked)

`/tmp/cmmc2/` — `composed.sv` / `genuine.sv` / `mismatch.sv` + their `*.ast.json`; `/tmp/presence_dbg.txt`
(`PGEN_PRESENCE_DEBUG=1` seed-0 capture — the detection/site/armed trace + the two composed probes).
