# STRUCTURED-WITNESS-SYNTH.2 — DESIGN: the structured-witness synthesizer (PASS 3f)

- **Status:** `done (DESIGN, PURE-DOCS)` — `PGEN-STRUCTURED-WITNESS-SYNTH-0003`, session #189, 2026-07-21.
- **Owns IMPLEMENT:** `STRUCTURED-WITNESS-SYNTH.3`.
- **Foundation:** the `.1` re-baseline (`PGEN-STRUCTURED-WITNESS-SYNTH-0002`): baseline
  byte-exact (canonical `UNKNOWN=12` / union `UNKNOWN=1` / residual =
  `context_member_method_call`; solo-canonical `1343/21/1321/1`), the A–F parse
  matrix (`docs/tasks/artifacts/structured_witness_synth/ab_matrix_current_vintage.txt`),
  and the probe extract (`probe_seed0_extract.txt`).

## 1. The verified problem statement

`context_member_method_call` (`grammars/systemverilog.ebnf:3132`, gate
`@predicate has_fact(variable_binding, $head)` `:3118`, consumer `call_primary`
alt-0 `:3184`, sole producer `variable_decl_assignment` with the DOTTED emit
`@emit_fact {kind: variable_binding, name: $name.body}` `:5917`) is the last SV
recognized-union UNKNOWN. The `.1` A–F matrix proves the witness template **E**
at the current vintage:

```
int foo;(*x=foo.bar.baz()*)bind b c d();      → 1 context_member_method node
```

i.e. THREE simultaneous conditions on the natural BFS carrier (no re-route):
1. a PRECEDING-unit **typed** declaration routed through `variable_decl_assignment`
   (localparam F-refuted: parses, emits no `variable_binding`);
2. the chain **head pinned** to the declared name;
3. the chain tail on `callable_method_call_body`'s parenthesised
   `call_with_args` branch (`:3026` alt-1).

## 2. Root cause of the standing miss (code-grounded)

The generator ALREADY owns machinery for each condition **in isolation**, in
SEPARATE passes that never compose into one sample
(`rust/src/ast_pipeline/stimuli_generator.rs`):

- **Prelude arming** — `compute_name_prelude` (`:3286`): 3 gate-discovery legs
  + two-pass clean-producer selection + prelude render as a FORCED EXTRA
  ITERATION of an on-path quantifier site, with the armed-fact INTEGRITY check
  + one depth-fresh retry (`:10953–11020`, `VERILOG-2005-PROFILE.6.3.2` —
  machinery NEWER than the `.4b.19` attempt: a wrong-family prelude is now
  loud, not silent). **Why it never arms here:** the producer filter
  `emit_name_is_whole_render` (`:3405`) rejects the DOTTED `$name.body` emit,
  so `variable_binding` has NO admitted producer ⇒ `compute_name_prelude`
  returns `None` (confirmed: zero injected declarations in every `.1` probe
  sample).
- **Use-site name replay** — `reach_prelude_replay_text` (`:3957`) renders the
  gated rule as the stored name via `store_name_for_gate` (`:3650`) — but it
  replays the consumer's WHOLE render, which cannot express the multi-token
  chain (the `.4b.13` consumer-side obstacle; the (c) head-only pin is the
  proven reverted answer).
- **Target-own structure forcing** — `generate_target_own_structure_witnesses`
  (`:4507`) + `mandatory_child_rules` (`:7221`, the `-0093` child-forcing):
  CAN render the parenthesised tail (a `.1` target-own probe rendered
  `.\foo (*…*)()`), but runs WITHOUT the prelude/pin, so the gate rejects on
  re-parse and a sibling absorbs the bytes.

**⇒ The synthesizer is a COMPOSITION pass, not a new mechanism family.**

## 3. Architecture — PASS 3f `generate_structured_witnesses` (residual-only, LAST)

A new final witness sub-pass, run after PASS 3e (carrier-diversification), over
ONLY the still-unwitnessed rules (the standing residual-only discipline: it can
only union new witnesses; the parser re-check remains the SOLE witness judge).
For each residual target `R`:

**(f1) Gate + producer discovery, with the (b1) admission relaxation.**
Reuse the 3 legs of `compute_name_prelude` unchanged. Extend the producer
filter: admit a producer whose emit name is a DOTTED `$ref` when every
alternative of the producer leads with a rule-reference token
(`rule_leads_with_rule_reference` — the `.4b.19`-proven relaxation), resolving
the emitted name to the producer's leading rendered token. Scope the
relaxation to PASS 3f (the earlier passes keep the strict whole-render filter
⇒ byte-identical behavior for every currently-witnessed rule).

**(f2) Prelude with typed-branch forcing ((b2)).** Compute the prelude via the
existing site/producer machinery (two-pass clean selection, integrity check +
retry all inherited). Add to the prelude's `sub_plan` a TYPED-BRANCH directive:
for each ordered choice on the producer sub-path whose non-first alternative
is a minimal-empty escape (detected via the witness pass's min-terminal-length
table, `Some(0)` — the `data_type_or_implicit := data_type | implicit_data_type`
shape), force the first typed alternative (the `.4b.19` `optional_escape_or_typed_branch`
mechanism). This is what turns the parse-only `foo;` render into the
fact-emitting `int foo;` (cell E vs the `.4b.13.1` count=0 finding).

**(f3) Head-leaf pin ((c)).** For a MULTI-token gated consumer
(`rule_render_is_single_leading_token(R) == false`), do NOT whole-render
replay; instead pin ONLY `R`'s first head-leaf render to
`store_name_for_gate(kind, family)` (the `.4b.19` `NameGateArm.whole_render` +
`pending_head_pin` mechanism, re-landed). Single-token consumers keep the
existing whole-render replay byte-identically.

**(f4) Target-own distinguishing structure IN THE SAME PLAN.** Merge the
`-0090`/`-0093` target-own directives into the SAME reach plan as (f2)/(f3):
root-Or non-degenerate branch forcing + mandatory-child root-Or forcing
(`mandatory_child_rules`), keyed `(child, node_path)` — for `R =
context_member_method_call` this forces `callable_method_call_body` (root Or)
through its alternatives; branch order = try each root-Or branch per the
existing bounded per-target budget, parser-judged (alt-1 `call_with_args`
renders `name(…)`; alt-0 `built_in_method_call` is also legal — whichever
re-parses INTO `R` wins; no branch index is hardcoded).

**(f5) Judge.** Re-parse the composed sample (prelude ++ carrier with pinned
head + forced structure) with the REAL parser; witness only on
`witnessed_target=true` (transactional coverage record) — identical judging to
every existing pass.

### Parser-agnosticism + capability gating

No grammar-name, rule-name, or sigil constants anywhere: gates come from
`gen_name_gate` (compiled from `@predicate`), producers from `gen_emit_facts`
(compiled from `@emit_fact`), typed-branch detection from the min-length
table, structure forcing from the grammar tree. The pass no-ops structurally
when `gen_name_gate` is empty or no rule is residual ⇒ the fully-certified
roster (predicate-free grammars + the already-`UNKNOWN=0` families) is
byte-inert BY CONSTRUCTION; SV's already-witnessed rules are untouched
(residual-only). Bounded: per-target attempts ≤ the existing witness budget
knobs; no new unbounded loops.

## 4. Predicted before → after (the `.3` bar)

- Canonical solo (seed 0): `total=1343 proof=21 witness=1321 UNKNOWN=1` →
  `witness=1322 UNKNOWN=0` (`fully_certified=true` solo).
- Gate accounting: canonical `UNKNOWN 12→11`, `witness 1321→1322`; union
  `UNKNOWN 1→0`, `witness 1332→1333`, residual `[]` ⇒ the contract `done_rule`
  fires: **SV recognized `fully_certified`**.
- Deterministic seeds 0/7/42; `spf=0`; ZERO newly-UNKNOWN (strict-subset
  check); 6 fully-certified grammars byte-identical.

## 5. Bisection order if inert (each step has a LOUD tool signal)

1. Prelude arms? — the `.6.3.2` integrity error / `name-prelude spec` debug
   trace names the armed producer; `[carrier-div-probe]`-class samples must
   gain the injected `int \foo ;`-shaped prelude.
2. Fact exists at use time? — `store_name_for_gate` non-None (integrity check
   makes absence an error, not silence).
3. Head pinned? — rendered head token == stored name (debug trace).
4. Tail structural? — the sample contains the parenthesised body (render
   inspection).
5. Attribution? — re-parse `--parse-dump-ast-pretty` + scoped
   `--trace-rules context_member_method_call`: which sibling absorbs, if any.
Per the `.4b.19` lesson, parts land TOGETHER and are measured GLOBALLY; keep
only on improvement ([[project_cert_coverage_tournament_loser_leak]]).

## 6. Fix-hierarchy adjudication (re-earned at this vintage)

- Tier 1 (annotation edit): the `.4b.18` code-refutation of a declarative
  `@sample`/`@probe_sample` witness STANDS — a literal sample cannot carry the
  name-coordinated prelude+pin coupling (names must coordinate across TWO
  statements chosen at generation time), and hint-tier renders are not
  store-checked. Re-verified consistent with the `.1` evidence (the hint tier
  never arms facts).
- Tier 2–4 (store): the store side is complete — the gate and producer exist
  and behave exactly per cells B/E/F. Nothing to add.
- **Tier 5-adjacent generator capability (Level-3+ parser-agnostic):** the
  landing level — a generation-side capability composing existing proven
  mechanisms. The grammar itself is NOT touched (no accepted-language change
  ⇒ no release/schema/ledger bump).

## 7. Verification matrix for `.3` (the acceptance the IMPLEMENT must earn)

1. Canonical cert seeds 0/7/42 byte-identical + solo `UNKNOWN=0` + `spf=0`.
2. `sv_cert_recognized_union_gate` — contract re-baselined (union `1→0`) and
   green (owned by `.4`, staged with the capability landing).
3. Strict-subset ZERO newly-UNKNOWN vs the `.1` banked residual.
4. 6 fully-certified grammars: cert output byte-identical.
5. `cargo test --lib` green + new lock tests (producer admission scoped to 3f;
   typed-branch forcing; head-pin single-token bypass).
6. `sv_stimuli_quality_gate` + `stimuli_cross_family_platform_gate` PASS.
7. clippy source-strict.
