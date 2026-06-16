# GRAMMAR-WELLFORMED.H.12.5.5.3.3 — M1b residual (parent-commit): WHY+WHERE

Tools-first WHY+WHERE for the **6 M1b carriers the `-0090` target-own-structure
pass could NOT close** (the SystemVerilog `UNKNOWN`→0 drive, `GRAMMAR-WELLFORMED.H.12`).
Opened by `H.12.5.5.3.2`. Investigation slice `PGEN-GRAMMAR-WELLFORMED-0091`
(PURE-DOCS — no code change; the `.3.3` leaf already owns the surface).

> Scope: layer-B detail behind the `H.12.5.5.3.3` row. Establishes, per carrier,
> the parent ordered-choice and the sibling that absorbs the bytes, then partitions
> the 6 into fix mechanisms — the prerequisite map before any fix
> ([[feedback_why_and_where_before_solution]], [[feedback_tools_first_no_guessing]],
> [[feedback_no_codebase_change_without_tool_backed_facts]]).

## Baseline (reproduced this session — deterministic ⇒ signal)

```
PGEN_CERT_COVERAGE_DUMP_ALL=1 ./target/debug/ast_pipeline ../grammars/systemverilog.ebnf \
  --report-certificate-coverage --grammar-profile sv_2017 \
  --entry-rule systemverilog_file --count 40 --seed 0
# total=1292 proof=1 witness=1201 UNKNOWN=90 fully_certified=false
#   (sample_parse_failures=0, proof_reverify_failures=0)
```

Byte-identical to the post-`-0090` state ⇒ the metric is signal. The 6 carriers are
all in the 90-rule UNKNOWN set: `array_range_expression`, `direct_index_method_call`,
`goto_repetition`, `context_member_method_call`, `class_scoped_tf_call`,
`sequence_method_call`.

## Decisive discriminator: the carrier is ABSENT from the parsed AST of its OWN sample

For each carrier I took a representative generated witness sample (from
`PGEN_CERT_COVERAGE_DEBUG_PROBES=1`) and re-parsed it with
`parseability_probe --parse-dump-ast-pretty systemverilog <sample> --profile sv_2017`.
**All 6 parse OK (full consume) but contain ZERO nodes of the carrier rule.** So the
carrier rule is genuinely NOT exercised — a sibling absorbs the bytes. This is NOT the
`H.10.2.2` memo-hit coverage-record bug (there the rule WAS in the AST, only the record
was lost on a cache hit); here the rule never matches. The `-0088` "parent-commit"
hypothesis is **confirmed** and refined into 4 mechanisms below.

## Per-carrier evidence

| Carrier | Sample (re-parsed) | Absorbing structure in the AST | Why R never matches |
|---|---|---|---|
| `goto_repetition` | `sequence\foo ;017.22->4096_.10e394endsequence` | `operand_chain`/`base` with `rest:[implies, primary]` — the `->` is an **infix expression operator** | `goto_repetition := ( implies const_or_range_expression )` = bare `-> expr` (no `[ ]`). `implies := trivia "->"`. The expression-level `implies` operand-chain operator eats `-> 4096`; the `( boolean_abbrev )?` host (@4565 on `expression_or_dist`) is never entered for this byte pattern. |
| `array_range_expression` | `program p(input logic a);assign{>>2621.24057with 430.400}=...` | `streaming_concatenation` parsed; no `array_range_expression`/`with`-clause node | `array_range_expression := expression` (degenerate alias). Host @4781 `stream_expression := expression ( kw_with ( array_range_expression )? )?` — the `( array_range_expression )?` optional was not entered. `-0090`'s target-own forcing deep-forced the alias `expression` body into a `:`-bearing form → `parsed=false` (over-constrained, wrong level). |
| `direct_index_method_call` | `program p(input logic a);assign\foo .\foo [\foo .\foo ].\foo =5852.59844;...` | `name dot lbrack ... rbrack dot hierarchical bit_select` — a **hierarchical-identifier + `bit_select`** select chain (`select` @4508 is `!lparen`-guarded) | def `:= (... hierarchical_identifier \| implicit_class_handle) dot method_call_body`; `method_call_body` (@2793) distinguishing alts require a **method call** (`method_identifier (args)` / built-in). Rendered tail is a bare `.\foo` (no `(args)`) ⇒ the `!lparen`-guarded select path wins. Distinguishing structure is a MANDATORY tail sub-rule (`method_call_body`'s call alt) the `-0090` walker does not force. |
| `context_member_method_call` | `(*\foo =+\foo .\foo .\foo *);` | `ps_parameter generate_scoped dot dot bit_select` — a plain **scoped-name** chain | def `:= identifier ( dot identifier constant_bit_select &dot )+ dot callable_method_call_body ( dot method_call_body )*`. The `( … )+` group REQUIRES a `constant_bit_select` (`[idx]`) after each identifier. Rendered `\foo.\foo.\foo` has NO `[…]` ⇒ the `+` group can't match ⇒ plain scoped name absorbs. `-0090` target-own added a 4th `.\foo` (still no `[…]`). Distinguishing structure is MANDATORY inner content of a `+` group the walker does not force. |
| `class_scoped_tf_call` | `sequence\foo ;(1.08e+744_8,\foo ::\foo )endsequence` | `scoped_or_hierarchical_bare package_scope package tf subroutine_call` — a **package-scope `tf_call`** | def `:= class_scoped_tf_call_with_args := class_scoped_call_prefix tf_identifier … ( args )`. `class_scoped_call_prefix` (@6202) is **STORE-GATED** on `known_unscoped_class_scoped_call_class_identifier` / `…interface_class…` / `…type_parameter…`. `\foo` is not established as a class in the store ⇒ the gated prefix fails ⇒ `tf_call` (package_scope `\foo::`) absorbs. This is store-faithfulness, NOT a pure parent-commit. |
| `sequence_method_call` | `module\foo (.\foo (+\foo .\foo ));endmodule` | `named_dot ansi` (a **named port connection**) + `built_in array_manipulation call` | def `:= sequence_instance dot method_identifier`, referenced only in property/sequence-expr choices (@2847/2864/3855/3909). The reach (BFS shortest-hop) routed the witness into a **module named-port-connection** host where `.\foo(expr)` is a port connection, not a sequence method call. Wrong host context — a reach-path SELECTION problem. |

## Partition into fix mechanisms (4 classes)

- **C-i — operator-shadow (1): `goto_repetition`.** The carrier's only form is a bare
  token (`->`) that is ALSO an expression operator, so the expression path eats it before
  the `( boolean_abbrev )?` host is entered. Fix needs either (a) a reach into the
  boolean_abbrev host with a form the operator can't absorb, and/or (b) an LRM bracket
  adjudication — IEEE 1800 `goto_repetition ::= '[' '->' const_or_range_expression ']'`
  has brackets; the grammar's `( implies const_or_range_expression )` does not. **Must
  LRM-ground the bracket question against `docs/systemverilog/2017` BEFORE any grammar
  edit** ([[project_ebnf_is_single_source_of_truth]]). (Note: sibling
  `consecutive_repetition := ( star const_or_range_expression )` is bare too — `*` is also
  an operator — so this is a family question, not a one-rule typo.)

- **C-ii — mandatory-inner-structure-not-forced (2): `direct_index_method_call`,
  `context_member_method_call`.** `-0090`'s target-own walker forces the root `Or` branch
  + min-0 (`?`/`*`) optionals, but NOT the distinguishing content of a MANDATORY sub-rule
  (a `dot method_call_body` tail's `(args)` alt; a `( … constant_bit_select … )+` group's
  `[idx]`). This is the natural EXTENSION of the `-0090` pass: descend mandatory `Seq`
  tails / `+`-group bodies and force their distinguishing alt too. Generator-only,
  parser-agnostic (keyed on `(R, node_path)` structure), strictly additive.

- **C-iii — store-gated (1): `class_scoped_tf_call`.** Its distinguishing prefix is gated
  on a store fact (the identifier must be a known class). Witnessing requires a sample
  that first establishes that fact — **STORE-AWARE-GEN territory**
  ([[feedback_grammar_rules_must_consult_store]]). RECLASSIFY out of M1b into
  `H.12.5.6` (M2 over-gen/store-unfaithful), where the B1 store-gated cluster already
  lives — it was mis-bucketed as M1b in `-0088` because its reach descends correctly.

- **C-iv — reach-path-selection (2): `array_range_expression`, `sequence_method_call`.**
  The reach lands in a host where the bytes can't be the target (a module port for
  `sequence_method_call`; the alias-body deep-force for `array_range_expression` instead
  of forcing the parent `stream_expression … with (…)?` optional). Same reach-path
  SELECTION-honesty family as `RTL-FE-CLOSURE.5.3`/`.5.4` and the M1a `H.12.5.5.2` lineage
  — choose a host/optional where the target positively renders.

## Fix children (split this leaf)

- `H.12.5.5.3.3.1` — C-ii mandatory-inner-structure forcing (generator; extends `-0090`).
- `H.12.5.5.3.3.2` — C-iv reach-path selection (generator reach-honesty).
- `H.12.5.5.3.3.3` — C-i `goto_repetition` operator-shadow (LRM bracket adjudication FIRST).
- `class_scoped_tf_call` reclassified to `H.12.5.6` (M2/store), removed from the M1b set.

Tractability order: C-ii (largest, reuses the `-0090` machinery) → C-iv → C-i (needs LRM
grounding). Each fix child: change ONE thing; rebuild DEBUG `ast_pipeline` (generator-only,
NO parser regen unless a grammar edit is LRM-grounded); measure GLOBAL cert + `spf` at
seeds 0/7/42 + `stimuli_cross_family_platform_gate` (the reach pass touches the closed-loop
driver). Watch the `888→1717` strictly-additive guard.

## Artifacts (this session, scratch)

`rust/target/generated_logs/h125533_baseline_seed0.txt` (DUMP_ALL),
`…/h125533_probes_seed0.txt` (DEBUG_PROBES), `…/samples/*.{sv,ast.json,parse.log}`
(the 6 re-parsed witnesses). Scratch only — not tracked.
