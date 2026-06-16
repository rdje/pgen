# GRAMMAR-WELLFORMED.H.12.5.5.3.3.4.2.1.1 — `context_member_method_call` semantic-prelude WHY+WHERE + DESIGN

Tools-first WHY+WHERE + implementation design for witnessing the `has_fact`-gated rule
`context_member_method_call` in cert-coverage. This slice (`PGEN-GRAMMAR-WELLFORMED-0101`,
PURE-DOCS) is the DESIGN half of the split leaf `H.12.5.5.3.3.4.2.1`; the generator FIX is the
child `H.12.5.5.3.3.4.2.1.2`. **No code/grammar/generated/release/schema/ledger change.**

> Scope: layer-B detail behind the `H.12.5.5.3.3.4.2.1` row. Opened by `H.12.5.5.3.3.4.2`
> (`-0098`), which root-caused the cert `UNKNOWN` to a store-gated WITNESS-REACH gap and routed
> the fix to the generator (the `has_fact` analogue of the regex `\NN` semantic-prelude class).
> This slice does the tools-first WHY+WHERE on the *actual reach path* (which `-0098` had not yet
> dumped) and discovers the design is more constrained than the leaf row assumed — then locks a
> design that the parser empirically validates.
> ([[feedback_why_and_where_before_solution]], [[feedback_tools_first_no_guessing]],
> [[feedback_no_codebase_change_without_tool_backed_facts]], [[project_ebnf_is_single_source_of_truth]].)

## Baseline (reproduced this session — deterministic ⇒ signal)

```
PGEN_CERT_COVERAGE_DEBUG_PROBES=1 PGEN_REACH_PATH_DUMP=1 \
  ./target/debug/ast_pipeline ../grammars/systemverilog.ebnf \
  --report-certificate-coverage --grammar-profile sv_2017 \
  --entry-rule systemverilog_file --count 40 --seed 0
# total=1292 proof=1 witness=1203 UNKNOWN=88 fully_certified=false (spf=0, proof_reverify=0)
```

Byte-identical to the post-`-0100` layer-A pointer. `context_member_method_call` is in the
88-rule `UNKNOWN` set.

## WHY+WHERE 1 — the reach path routes through an `attribute_instance`, and the minimal probe is not even a method call

`PGEN_REACH_PATH_DUMP=1` shows the BFS shortest-hop reach path the plannable pass installs for
`context_member_method_call`:

```
systemverilog_file → source_text (root/q) → source_text_item → description (root/o5/s0/q)
  → attribute_instance → attr_spec → constant_expression → … → constant_function_call
  → call_primary → context_member_method_call (root/s3)
```

i.e. the rule's shallowest reach is inside an **attribute instance** `(* attr = const_expr *)`,
not a module/program procedural statement. The plannable + target-own probes (`PGEN_CERT_COVERAGE_DEBUG_PROBES=1`):

```
[plannable-probe]   rule='context_member_method_call' parsed=true witnessed_target=false sample="(*\foo =+\foo .\foo .\foo *);"
[target-own-probe]  rule='context_member_method_call' parsed=true witnessed_target=false sample="(*\foo =+\foo .\foo .\foo .\foo *);"
```

Two facts jump out:

1. **The probe is not a method call.** `\foo .\foo .\foo` is a bare hierarchical reference — no
   `()`. `context_member_method_call := identifier ( dot identifier constant_bit_select &dot )+
   dot callable_method_call_body …` needs the trailing `.callable_method_call_body` to be a real
   *method-call* form. So even structurally the minimal probe cannot be this rule; the bytes parse
   as a plain hierarchical/constant primary (the PEG re-attributes them to a sibling).
2. **Identifiers render CANONICALLY as `\foo`.** Every identifier on the construct-mode plannable
   path is the same escaped identifier `\foo`. (Some *target-own* probes use random identifiers —
   `(*u6H=+P.Mp.Q6.DlM9*)` — when the distinguishing-structure pass re-rolls terminals, but the
   construct-mode/minimal path is canonical `\foo`.)

## WHY+WHERE 2 — empirical witness matrix (the parser is the judge)

Built the witnessing requirements by direct `parseability_probe --parse-dump-ast-pretty
systemverilog <sample> --profile sv_2017`, checking for a `context_member_method` AST node:

| # | sample | parses | witnesses `context_member_method` |
|---|---|---|---|
| a | `(*\foo =+\foo .\foo .\foo *);` (the bare probe) | yes | **no** (no method call, no binding) |
| b | `module m; int \foo ; int \bar ; initial \bar = \foo .\foo [0].\foo (); endmodule` | yes | **YES** |
| c | `int \foo ; (*\foo =+\foo .\foo .\foo *);` (top-level decl, but bare ref) | yes | **no** (still no method-call structure) |
| c2 | `int \foo ; (*\foo =+\foo .\foo [0].\foo ()*);` (top-level decl + real method call) | yes | **YES** |
| d | `module m; initial \bar = \foo .\foo [0].\foo (); endmodule` (head UNDECLARED) | **REJECT** (pos 0) | — |
| e | `module m; int \foo ; initial \bar = \foo .\foo .\foo (); endmodule` (declared, no index) | yes | **YES** |

The three operative facts this nails down:

- **The declaration of the head is necessary** (d rejects; b/c2/e accept) — confirming the
  `has_fact(variable_binding, $head)` post-predicate is the gate, exactly as `-0098` proved.
- **The chain must be a real method call** (a/c don't witness for lack of `()`; c2/e do) — the
  rule's mandatory `.callable_method_call_body` distinguishing structure.
- **★ A TOP-LEVEL `variable_binding` fact IS visible to `has_fact` at a later
  `description`-level attribute_instance** (c2 witnesses — the `int \foo ;` decl at `$unit`/root
  scope satisfies the gate inside the *later* `(* … *)`). This means **the existing
  attribute-spec reach path is salvageable** — a top-level binding-producer prelude can satisfy
  the gate without re-selecting the reach path toward a module body. (This is the durable
  store-scoping fact recorded in KM card
  [sv-store-fact-scope-and-canonical-name-coupling](../knowledge/sv-store-fact-scope-and-canonical-name-coupling.md).)

So the leaf-row premise ("emit a binding-producer prelude … coupled to `$head`") is correct AND
realizable on the existing reach path — but it has two coupled sub-requirements the count-gated
MVP never faced: (i) the prelude must be a TOP-LEVEL `source_text_item` (the on-path quantifier
site is `source_text := source_text_item*`, not a statement list), and (ii) the gated rule must
*also* render its mandatory method-call structure in the same sample.

## WHERE (code) — `compute_reach_prelude` is `fact_count_at_least`-only

`stimuli_generator.rs`:

- `gen_count_kinds: HashMap<String, Vec<String>>` (`:1445`) — rule → fact-kinds K of its
  `fact_count_at_least(K,$ref)` post-predicates. **There is no `has_fact` analogue.**
- `compute_reach_prelude` (`:2769`) returns `None` immediately when `gen_count_kinds.is_empty()`
  (`:2776`) — true for SV — so SV gets no prelude today. Its `gated_rule` scan, producer
  selection (`gen_emit_facts` → rules whose `@emit_fact` emit K), and site selection (innermost
  on-path quantifier whose direct-rule-reference body graph-reaches the producer) are otherwise
  exactly what the `has_fact` case needs.
- The two-phase count flow (`reach_prelude_capture` `:2898` numeric value → arm `iterations=v`
  `:3079`; `reach_prelude_replay_text` `:2868`; `reach_prelude_bypasses_count_prune` `:2859`) is
  count-specific and **not needed** for `has_fact` (see design).

Producer + consumer grammar facts (`grammars/systemverilog.ebnf`):

- consumer `context_member_method_call` (`:2906`), `@predicate has_fact(variable_binding, $head)`
  (`:2892`), `$head = $1.body` = the first `identifier`'s body.
- producer `variable_decl_assignment` (`:5404`), `@emit_fact { kind: variable_binding, name:
  $name.body }` (`:5403`), name = the `variable_identifier`'s body. Both head and producer-name
  derive from the same identifier machinery → both render canonical `\foo` under construct mode
  ⇒ **name-coupling is free** on the minimal path (to be re-verified in the FIX, not assumed).

## DESIGN — extend the semantic-prelude reach to `has_fact` (the `.4.2.1.2` FIX)

Parser-agnostic, keyed purely on grammar structure (predicate kind + `@emit_fact` producers +
rule-reference graph + on-path quantifier sites), never on grammar/rule names; inert for the
fully-certified roster (which has no `has_fact` post-predicate gating an `UNKNOWN` rule on a
plannable path).

1. **Detect `has_fact`-gated rules (new gen-side map).** Precompute (alongside `gen_count_kinds`)
   a map `gen_has_fact_gates: rule → (kind, name_ref_capture)` from each rule's `has_fact(K,
   $ref)` *post*-predicate where `$ref` is a rule-reference capture (the name to couple). For
   `context_member_method_call`: `(variable_binding, head)`.

2. **`has_fact` branch in `compute_reach_prelude`.** When no count-gated rule is on the path,
   scan for the first `has_fact`-gated rule on the path (same hop-order scan). Pick the producer
   (first sorted rule whose `@emit_fact` emits K = `variable_binding` → `variable_decl_assignment`)
   and the site (innermost on-path quantifier whose direct-rule-reference body graph-reaches the
   producer). Here the on-path quantifier `source_text := source_text_item*` (`site=(source_text,
   root/q)`, `body=source_text_item`) graph-reaches `variable_decl_assignment` via a top-level data
   declaration — so the prelude is a top-level `int \foo ;`-shaped item. Build the same
   body→producer `sub_plan`.

3. **Fixed `iterations = 1`, no capture phase, no replay.** One declaration satisfies `has_fact`
   (vs the count case's `v` copies). Arm `iterations = 1` from the start — no numeric capture, no
   `count`-prune bypass (SV is not `store_aware_gen`; the gate is a PARSE-time post-predicate, so
   the prelude only needs to produce the right *text* — the real parser emits the
   `variable_binding` fact when it parses the injected declaration). The existing
   `reach_prelude_for_site` injection in `generate_quantified` then prepends one producer-steered
   `source_text_item` before the on-path `description`.

4. **Name-coupling.** On the canonical construct-mode path the producer renders `\foo` and the
   head renders `\foo`, so the gate is satisfied for free. The FIX must *verify* this holds in the
   armed sample (parser is the judge — a mismatch can only fail loudly, never false-witness); if a
   non-canonical render ever breaks coupling, add explicit coupling (capture the gated rule's
   `$head` render, force the producer's identifier to it) — but only if empirically needed.

### Open composition question for the FIX (the real risk to resolve, not guess)

The prelude is attached in the **plannable** pass (`set_reach_plan_for_rule`), which renders the
gated rule MINIMALLY — i.e. *without* the mandatory `.method()` structure (probe a's shape). The
rule's distinguishing method-call structure is forced by the *separate* **target-own** pass (PASS
3c), which carries no prelude. A witnessing sample needs BOTH the binding prelude AND the
method-call structure in ONE generation (Test c2's shape). So the FIX must make the two compose —
either (a) attach the `has_fact` prelude to the target-own pass as well, or (b) have the plannable
prelude path also force the gated rule's mandatory `callable_method_call_body` child structure
(the `.3.3.1` mandatory-child-forcing lineage). This is the decisive design choice the FIX slice
must settle tools-first (dump the armed sample; confirm it carries decl + `.method()`), not the
prelude detection itself.

### Acceptance for the FIX (`.4.2.1.2`)

- SV cert-coverage `UNKNOWN 88 → 87` (`context_member_method_call` witnessed), deterministic at
  seeds 0/7/42, `spf` unchanged (0).
- Strictly additive (the `888→1717` guard); fully-certified roster (rtl_const_expr/json/regex/
  vhdl/svpp/rtl_frontend) byte-identical (`UNKNOWN=0`, no new pass cost — the roster has no
  `has_fact`-gated `UNKNOWN` rule on a plannable path).
- `stimuli_cross_family_platform_gate` PASS; `clippy_on_rust_change` strict-source clean.
- GENERATOR-ONLY ⇒ no grammar/EBNF/parser-regen/release/schema/ledger change (off-reach
  byte-identical by construction).

## Conclusion

- The `-0098` routing (generator constructor gap, `has_fact` analogue of regex `\NN`) is
  **confirmed and realizable on the existing reach path** — a top-level binding-producer prelude
  satisfies the gate (Test c2), and name-coupling is free via canonical `\foo`.
- The leaf splits into this WHY+WHERE+DESIGN (`.4.2.1.1`, done here) and the generator FIX
  (`.4.2.1.2`, frontier next) — the FIX's first job is the prelude↔method-call **composition**
  question above.
- PURE-DOCS ⇒ no code/grammar/generated/release/schema/ledger change; clippy not invoked; SV
  stays the only non-fully-certified shipped grammar (`UNKNOWN=88`, unchanged).

## Artifacts (this session, scratch — not tracked)

`rust/target/generated_logs/h1255334_421/` — `probe_seed0.txt` (DEBUG_PROBES + REACH_PATH_DUMP
baseline), `a.sv`/`b.sv`/`c.sv`/`c2.sv`/`d.sv`/`e.sv` (the witness-matrix inputs) and their
`*.ast.json` / `*.err`.
