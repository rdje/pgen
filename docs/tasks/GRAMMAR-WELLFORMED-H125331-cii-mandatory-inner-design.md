# GRAMMAR-WELLFORMED.H.12.5.5.3.3.1 — C-ii mandatory-inner-structure forcing: design

Design note for the **C-ii fix** leaf (the SystemVerilog `UNKNOWN`→0 drive,
`GRAMMAR-WELLFORMED.H.12`). Opened by `H.12.5.5.3.3` (`-0091`, the M1b residual
WHY+WHERE). Tool-backed edit-surface map so the delicate certification-machinery
change lands surgically — the `-0089`→`-0090` rhythm applied to C-ii.

> Carriers: `direct_index_method_call`, `context_member_method_call`. PURE-DOCS
> setup; no code change ([[feedback_no_codebase_change_without_tool_backed_facts]],
> [[feedback_why_and_where_before_solution]]).

## Why `-0090` cannot close these two (tool-backed)

The `-0090` target-own-structure pass (`generate_target_own_structure_witnesses`,
`stimuli_generator.rs:3149`) forces, for a residual rule `R`, only directives keyed on
**R's OWN node-paths**: its root `Or` branch (`target_own_reach_sites` :5173, descends
the body spine through `Atom::Node` shells to the top-level `Or`) and its min-0
quantifiers (`collect_optional_quantifier_paths` :5205). Both C-ii carriers defeat that:

- Their transformed body root is a **`Sequence`**, not an `Or` ⇒ `root_or = None`
  ⇒ `branch_candidates = [None]` (no branch forcing).
- Their distinguishing structure lives in **referenced rules**, not in R's own tree.
  In the `--dump-gen-ast` transform, a rule reference is a leaf token
  `Atom(Token(["rule_reference", "<name>"]))` — NOT an inlined subtree — so the
  `-0090` walker (which only recurses Or/Sequence/Quantified/`Atom::Node`) never
  reaches into the referenced rule.

Per-carrier (transformed tree + grammar def + the `-0091` AST re-parse evidence):

- `direct_index_method_call` = `Seq( Or[ Seq( (ref kw_class_qualifier|ref non_typedef_package_scope)?  ref hierarchical_identifier ) | ref implicit_class_handle ]  ref dot  ref method_call_body )`.
  Distinguishing tail = `ref method_call_body`, which is its own rule
  `method_call_body := built_in_method_call | method_identifier … ( args ) | method_identifier …`
  (`systemverilog.ebnf:2793`, `@branch_policy: priority_first`). In construct-mode the
  minimal render picks the bare `method_identifier` (3rd alt) ⇒ a bare `.\foo` tail ⇒ the
  `!lparen`-guarded `select`/`bit_select` chain (`:4508`) absorbs the whole LHS. To witness,
  `method_call_body` must render a **call** form (built-in or `( args )`).
- `context_member_method_call` = `Seq( ref identifier  Seq( ref dot ref identifier ref constant_bit_select &ref dot )+  ref dot ref callable_method_call_body  Seq( ref dot ref method_call_body )* )`.
  The mandatory `( … )+` group (min-1, so it DOES generate) contains `ref constant_bit_select`,
  its own rule `constant_bit_select := ( lbrack constant_expression rbrack )*`
  (`systemverilog.ebnf:1262`) — a **min-0 `*`** that renders EMPTY by default ⇒ the group
  emits `.identifier` with no `[idx]` ⇒ a plain scoped-name path absorbs. To witness,
  `constant_bit_select` must render **≥1** `[ constant_expression ]`.

Both distinguishing sites are in a referenced rule reached through a **mandatory** position
of R (a Sequence element / a min-1 `+` group element) — never behind a min-0 quantifier and
never inside an un-forced `Or` alternative.

## The read-sides already support referenced-rule forcing (no read-side change)

`generate_or` (`~:6441`) reads `forced_branch_for(current_rule, node_path)`; `generate_quantified`
(`~:7753`) reads `forced_quantifier_min[(current_rule, node_path)]`. During generation
`current_rule` tracks the rule **currently being generated** — so when the generator descends a
`rule_reference` into `method_call_body` / `constant_bit_select`, `current_rule` becomes that
child and a directive keyed `(child_rule, path)` fires. This is exactly how
`set_reach_plan_for_rule` already forces every rule ALONG the entry→R hop path (keyed per hop's
rule). ⇒ **The fix needs no read-side change** — only the WALKER must additionally enumerate
the mandatory child rules and their distinguishing sites, keyed by the child rule name.

## Design — bounded mandatory-reference descent

Add a walker that, for residual rule `R`, collects the **mandatory referenced rules** in R's
body and, for each, its target-own forcing sites (reusing `target_own_reach_sites`), to a small
bounded depth. "Mandatory" = a `rule_reference` reached without crossing a min-0 quantifier and
without entering an un-forced `Or` alternative:

- `Sequence`: every element is on the mandatory path.
- `Quantified` with min ≥ 1 (`+`, `{N,…}` N≥1): its element is mandatory.
- `Quantified` with min 0 (`?`, `*`, `{0,…}`): NOT mandatory — skip (forcing it is the
  separate `inner_quantifier_paths` mechanism, already handled for R's own body).
- `Or`: skip (no single branch is mandatory unless one is forced — to stay sound and additive,
  do not descend un-forced `Or`s in the first cut).
- `Lookahead`: skip (materialises nothing).
- `Atom(Token(["rule_reference", C]))`: C is a mandatory child — record it.
- `Atom(Token(other))` / `Atom(Node)`: terminal / inline group — recurse the group, no child.

For each mandatory child `C` (bounded to depth ≤ 2 to avoid the array_range_expression-style
over-constraint and any cost blow-up), compute `target_own_reach_sites(C)` and add
`directives[(C, c_or_path)] = j` (non-degenerate branch, `o1..` first then `o0`) +
`forced_quantifier_min[(C, c_q_path)] = 1`. This handles BOTH carriers:
`method_call_body` (force its root `Or` to a call branch) and `constant_bit_select` (force its
`*` to ≥1).

### Edit surface (`stimuli_generator.rs`)

1. New `fn mandatory_child_rules(&self, rule, depth) -> Vec<String>` (or fold into a new
   `target_own_reach_sites_deep`): walk R's body per the rules above, returning the mandatory
   child rule names (dedup, bounded depth). Pure structural walk over `grammar_tree`.
2. In `generate_target_own_structure_witnesses` (`:3173` loop): after the existing
   `(root_or, inner_quantifier_paths)` forcing, ALSO compute the mandatory children and, for each
   `C`, its `target_own_reach_sites(C)`; install `(C, …)` directives into `plan` alongside the
   existing R-keyed ones (same `plan.directives` / `plan.forced_quantifier_min` maps).
3. Keep the per-branch parser-judged probe loop unchanged (it already only accepts `Witnessed`
   ⇒ an over-constrained `parsed=false` probe is simply rejected, never regressing).

### Why this stays inert + additive

- Runs only inside PASS 3c (residual-only — `-0090`'s architecture): a grammar the prior passes
  already certify has an empty residual ⇒ no probe. Verified inert for `rtl_const_expr`/`json`.
- Parser-judged: every probe is replayed through the real parser; only `Witnessed` counts.
  STRICTLY ADDITIVE — the pass only ever UNIONS witnesses (the `888→1717` guard).
- Keyed purely on `(rule, node_path)` ASTNode structure — NO grammar/rule semantics
  ([[feedback_ast_pipeline_parser_agnostic]]).
- Bounded depth ≤ 2 + the residual-only pass bound the cost (the `-0090` cross-grammar-cost
  lesson — measure `rtl_const_expr` cert wall-time in verification).

## Verification protocol (the `-0090` protocol)

- Rebuild DEBUG `ast_pipeline` (`cargo build --features generated_parsers,ebnf_dual_run --bin
  ast_pipeline`) — generator-only, NO parser regen.
- cert-coverage seed 0: expect `direct_index_method_call` + `context_member_method_call` to leave
  UNKNOWN (witness up), `spf=0`, and the **1201 baseline witnesses must not regress**. Then seeds
  7 and 42 (determinism). Target: SV `UNKNOWN 90 → ~88`.
- `stimuli_cross_family_platform_gate` (the reach pass touches the closed-loop driver).
- Re-measure `rtl_const_expr`/`json` cert: byte-identical `UNKNOWN=0`, no added wall-time.
- `clippy_on_rust_change` strict-source clean. GENERATOR-ONLY ⇒ no parser-regen/grammar/release/
  schema/inventory/ledger change.

## Open implementation questions

- Depth bound: start at 1 (direct child only — sufficient for both carriers: `method_call_body`
  and `constant_bit_select` are each a single hop from R). Raise to 2 only if a carrier needs a
  grandchild forced. Start MINIMAL.
- When R's root IS an `Or` (other future C-ii carriers): the mandatory children must be recomputed
  PER forced branch (the mandatory set depends on which branch is forced). For the two current
  carriers (Sequence roots) this does not arise — defer the per-branch recomputation until a
  carrier needs it.
- Interaction with `needs_rule_body_descent`: adding `(C, …)` directives makes
  `needs_rule_body_descent(C)` true, so any rule-level `@sample` on C stands down (the `-0087`
  behavior) — desired (we want C's body, not its canonical literal). Confirm at implementation.

## Implemented (`PGEN-GRAMMAR-WELLFORMED-0093`, GENERATOR-ONLY)

Landed the designed mechanism: `mandatory_child_rules` (the bounded mandatory-reference walker)
+ a purely-additive child-forcing loop in `generate_target_own_structure_witnesses` (runs only
when R-own forcing did not witness; per-child budget so a distinguishing child is reached even
when an earlier child is also forceable). **SV cert `UNKNOWN 90 → 89`** (witness `1201 → 1202`,
`spf=0`, deterministic seeds 0/7/42). Inert for the fully-certified roster (rtl_const_expr/json
re-measured `UNKNOWN=0`, no slowdown). Cross-family gate PASS; clippy source-clean.

**Outcome vs the two hypothesized C-ii carriers (the honest finding).** The mechanism is correct
and general, but it did NOT close `direct_index_method_call` or `context_member_method_call` — it
closed **`sequence_method_call`** instead (a `-0091` C-iv carrier whose mandatory child needed
structure forced). Re-parsing the forced witnesses (`parseability_probe --parse-dump-ast-pretty`)
shows WHY the two named carriers stay UNKNOWN even with their distinguishing structure rendered:

- `direct_index_method_call`: forcing `method_call_body` to a call form DOES render the call
  (`…[class_qualifier\foo.\foo(*\foo*)()]…`), but the bytes are credited to
  `split_direct_callable_method` (a `method_call` sibling in `bit_select_expression`'s ordered
  choice) — direct_index is tried first but its match fails partway, so the sibling wins.
- `context_member_method_call`: forcing `constant_bit_select` ≥1 DOES render the `[idx]`
  (`\foo.\foo[+…].\foo.\foo`), but the form is still re-attributed at the `call_primary` /
  attribute-spec parent ordered choice.

So both named C-ii carriers are ALSO sibling-absorbed at the PARENT regardless of their own (or
their child's) forced structure — the parent-commit class, not closable by structure forcing
alone. They are reclassified to a follow-up (`H.12.5.5.3.3.4`): parent-commit forcing (force the
parent ordered choice to R's branch on the generation side) and/or an ordered-choice-shadowing
adjudication (whether `direct_index_method_call` is effectively shadowed by `method_call` in
`bit_select_expression`). The mandatory-child-forcing capability stays — it is general, strictly
additive, and closed a real reachable-unwitnessed rule.
