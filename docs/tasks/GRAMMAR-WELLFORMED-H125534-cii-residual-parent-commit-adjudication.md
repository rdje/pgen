# GRAMMAR-WELLFORMED.H.12.5.5.3.3.4 — C-ii residual (parent-commit): ADJUDICATION

Tools-first WHY+WHERE/adjudication for the **2 C-ii carriers `-0093` could not close**
(`direct_index_method_call`, `context_member_method_call`) — the SystemVerilog
`UNKNOWN`→0 drive (`GRAMMAR-WELLFORMED.H.12`). Opened by `H.12.5.5.3.3.1`.
Adjudication slice `PGEN-GRAMMAR-WELLFORMED-0097` (PURE-DOCS — no code change).

> Scope: layer-B detail behind the `H.12.5.5.3.3.4` row. The leaf asked one question
> per carrier: **(a)** is it an ordered-choice SHADOWING / well-formedness defect (fix the
> grammar), or **(b)** a parent-commit GENERATOR-forcing gap (force the parent `Or` to commit
> to R's branch)? The answer for BOTH is grammar-class — **(b) is refuted for both** —
> but they need *different* grammar fixes ([[feedback_why_and_where_before_solution]],
> [[feedback_tools_first_no_guessing]], [[feedback_no_codebase_change_without_tool_backed_facts]],
> [[feedback_be_alert_root_cause_fishy_immediately]]).

## Baseline (reproduced this session — deterministic ⇒ signal)

```
PGEN_CERT_COVERAGE_DUMP_ALL=1 ./target/debug/ast_pipeline ../grammars/systemverilog.ebnf \
  --report-certificate-coverage --grammar-profile sv_2017 \
  --entry-rule systemverilog_file --count 40 --seed 0
# total=1292 proof=1 witness=1203 UNKNOWN=88 fully_certified=false
#   (sample_parse_failures=0, proof_reverify_failures=0)
```

Byte-identical to the post-`-0096` layer-A pointer ⇒ the metric is signal. Both carriers are
in the 88-rule UNKNOWN set (confirmed in the dump).

## Adjudication (a) first — the linter is the adjudicator

`./target/debug/ast_pipeline ../grammars/systemverilog.ebnf --lint-grammar --grammar-profile sv_2017`:

```
ordered_choice_shadowing=0 (error), always_matches_shadowing=6 (warning, A2 backlog)
```

The 6 `always_matches` warnings are all UNRELATED rules (`sv_multi_entry_root`,
`ansi_port_declaration`, `net_port_type*`). The **sound, decidable** shadowing checks
(exact-duplicate + fixed-terminal-prefix + always-succeeds) find **zero** shadowing on either
carrier. This is expected and correct: the relationship here is **FIRST-set subsumption**, which
the book documents as *deliberately omitted* (unsound for PEG — an earlier arm may match-then-fail
and backtrack). So the linter cannot auto-decide it; this is the **manual-adjudication remainder**
the certifying model names. Manual adjudication, tool-backed, below.

## Carrier 1 — `direct_index_method_call`: effectively DEAD (subsumed by `method_call`)

Grammar (`:655`/`:670`, `bit_select_expression` branch_policy=`priority_first`):

```
direct_index_method_call := ( (kw_class_qualifier|non_typedef_package_scope)? hierarchical_identifier
                            | implicit_class_handle ) dot method_call_body          # default longest_match
bit_select_expression := direct_index_method_call   # branch 1 (tried FIRST)
                       | method_call                 # branch 2
                       | kw_dollar                    # branch 3
                       | expression                   # branch 4
bit_select := ( lbrack bit_select_expression rbrack )*
```

`direct_index_method_call` is **branch 1** under `priority_first`, so if it ever matched a full
bracket content it would WIN. It never does. Decisive routing matrix (`x[<body>]`, re-parsed with
`parseability_probe --parse-dump-ast-pretty`; `direct_index_method` node count in the AST):

| `x[body]` | accepted | `direct_index_method` in AST | wins via |
|---|---|---|---|
| `b.c()`        | yes | 0 | `method_call` (branch 2) |
| `this.foo()`   | yes | 0 | `method_call` (branch 2) |
| `this.foo`     | yes | 0 | `method_call` |
| `super.bar()`  | yes | 0 | `method_call` |
| `b.c`          | yes | 0 | `method_call`/`expression` |
| `a.b.c()`      | yes | 0 | `method_call` |

**Mechanism (trace, `PGEN_TRACE_VERBOSITY=high --trace-rules`):** branch 1 IS attempted first, but
`direct_index_method_call`'s `method_call_body` settles on the **bare** method name (`method_bare`
arm — e.g. `this.foo`), leaving the `()` args; the rule consumes `recv.method` and the parent
`bit_select` then fails its `]` because `(` follows. `bit_select_expression` falls to **branch 2
`method_call`**, which consumes the full `recv.method()`. Structural reason: `method_call :=
method_call_initial ( dot method_call_body )*` and `method_call_initial` includes `direct_method_call
:= method_call_root dot method_call_body` and `split_direct_callable_method_call` — so
`method_call`'s language is a **superset** of `direct_index_method_call`'s. The default
`longest_match` on the receiver `Or` makes the greedy `hierarchical_identifier` eat the dotted path,
so the mandatory `dot method_call_body` tail cannot complete distinctively. Net: `direct_index_method_call`
is **never the selected branch for any tested input** — an effectively-dead, subsumed alternative.

**LRM grounding (single source of truth):** IEEE 1800 §11.5.1 `bit_select ::= { [ expression ] }` —
the bracket content is just an `expression`. SV `expression` (branch 4) + `method_call` (branch 2) +
`kw_dollar` (branch 3, the `q[$]` form) cover the LRM. `direct_index_method_call` is a **redundant
grammar-synthesis artifact**, not an LRM construct.

**VERDICT (a):** dead/subsumed → **REMOVE `direct_index_method_call`** from `bit_select_expression`
(and the now-orphan rule), the attribution-rule "delete the dead branch at the source" path
(precedent: the 25 shadow branches `.7.4.6.7`, the 48 number-literal orphans `H.12.1`, `white_space`).
Closes the `direct_index_method_call` `UNKNOWN` (`88→87`). **Removal MUST be proven parse-neutral by a
decisive A/B** (regen SV parser → external corpus 14/14 byte-identical + realistic + cert witness count
unchanged) in the fix slice — empirical non-selection is strong but not a proof of deadness (undecidable
in general), so the A/B is the proof. Referenced ONLY at `:670` ⇒ removal blast radius is one site.
Parse-neutral ⇒ expected NO release/schema bump.

→ fix child **`H.12.5.5.3.3.4.1`**.

## Carrier 2 — `context_member_method_call`: a real latent PARSE GAP (bug-finding-oracle hit)

> ⛔ **SUPERSEDED by `PGEN-GRAMMAR-WELLFORMED-0098` (`.4.2` RE-ADJUDICATION,
> [[GRAMMAR-WELLFORMED-H1255334-42-context-member-readjudication]]).** This Carrier-2 verdict (a′)
> is **WRONG on both checkable claims**: (1) the `@predicate has_fact(variable_binding, $head)` IS
> live in the generated parser (`generated/systemverilog_parser.rs:3041-3055` — the no-predicate
> grep hit a one-line-grep trap); and (2) `a.b[0].c()` PARSES and witnesses `context_member_method_call`
> with a *declared* head (`int a; … a.b[0].c()`) — the `-0097` failing tests never bound the head
> (`C` was an undeclared type). There is **no released-parser bug**; the cert `UNKNOWN` is a
> store-gated **witness-reach gap** (the `has_fact` analogue of the regex `\NN` semantic-prelude
> class), re-routed to generator child `.4.2.1`. Read the re-adjudication file as authoritative for
> Carrier 2. (Carrier 1 below stands but its premise must be independently re-verified given this error.)

Grammar (`:2906`/`:2959`, `call_primary` branch_policy=`priority_first`, branch 1):

```
context_member_method_call := identifier ( dot identifier constant_bit_select &dot )+
                              dot callable_method_call_body ( dot method_call_body )*
call_primary := context_member_method_call | implicit_class_rooted_method_chain
              | call_with_postfix_chain | identifier_rooted_method_chain
              | split_direct_callable_method_call | ... (10 branches)
```

**Refuted the store-gate hypothesis (C-iii):** the rule's `:2915` comment claims it is "gated by
`has_fact(variable_binding, $head)`", but the **generated parser carries NO `@predicate`** on it
(grep of `generated/systemverilog_parser.rs` — the comment is stale dead text), and the form is
rejected even with a declared head (`class C; function void f; a.b[0].c(); ...`, `module m; C a; ...`).
So this is **not** the store class.

**The decisive finding — the rule's own designed form is REJECTED parser-wide.** Sharp gap boundary
(`assign v = <body>` and other contexts):

| body | accepted |
|---|---|
| `a.c()`        | yes |
| `a[0].c()`     | yes  (head-indexed: handled by `split_direct_callable_method_call`) |
| `a[0].c`       | yes |
| `a.b[0].c`     | yes  (no trailing call) |
| `a.b.c()`      | yes  (routes to `split_direct_callable` sibling — `context_member` never wins) |
| **`a.b[0].c()`** | **NO** (rejected in every context: assign-RHS, `initial` stmt, `begin/end`, class-fn, `$display(...)` arg) |

`a.b[0].c()` is `head.member[idx].method()` — a **non-head member indexed, then a method call** — valid
SV (`bus.packets[0].crc()`, `q[i].randomize()`). It is exactly the form `context_member_method_call`
was *authored* for (its `:2895` comment cites uvm `urme_container.elements[i].clone()`). The choke
trace (`furthest_position` at end; deepest rule_stack) shows `context_member_method_call` IS entered
(matches `a.b[0]`, attempts `.c()` via `callable_method_call_body`) but backtracks, and **no sibling
recovers** → full-input rejection. So `context_member_method_call` is **broken** — it cannot match its
own designed form — and that is *why* it never witnesses (the cert `UNKNOWN` is the SYMPTOM).

**Why the external corpus never caught it:** `uvm-core-2020.3.1/src/uvm_pkg.sv` has **0** instances of
the `X[i].method()` shape (`grep -cE '\[[^]]*\]\.[A-Za-z_][A-Za-z_0-9]*\('` → 0). SV corpus stays 14/14
because the form is unexercised — exactly the bug-finding-oracle pattern: the cert pass surfaces a
latent defect the corpus did not.

**VERDICT (a′):** NOT generator parent-commit forcing, NOT store-gated — a real **latent released-parser
bug** (valid `head.member[idx].method()` rejected). Per the director principle (*fix parser bugs ASAP =
highest priority*) and the attribution rule (grammar-first), the fix is to **root-cause the
`a.b[0].c()` rejection and repair the grammar** so the form parses — which then witnesses
`context_member_method_call`. This is a language change ⇒ full grammar-edit lockstep (regen, release
bump, `PGEN_RELEASED_PARSER_BUG_LEDGER` row, contract + parser-book + book sync) in the fix slice.
HIGHER priority than the `direct_index` removal.

→ fix child **`H.12.5.5.3.3.4.2`** (highest priority of the two).

## Conclusion — both carriers are grammar-class; (b) parent-commit forcing is REFUTED

The leaf's hypothesized branch (b) — adding generator machinery to force the *parent* `Or` to commit
to R's branch — is the **wrong fix for both**. `direct_index_method_call` is genuinely dead (forcing
the parent would only manufacture a sibling-ambiguous form the PEG still re-attributes); and
`context_member_method_call` cannot be witnessed by any forcing because the parser *rejects* its
form — the fix must restore acceptance. The attribution rule routed both to the GRAMMAR, exactly as
the grammar-first suspicion order prescribes.

## Fix children (split this leaf)

- `H.12.5.5.3.3.4.1` — remove the dead/subsumed `direct_index_method_call` from `bit_select_expression`
  (grammar; decisive A/B parse-neutral; closes `UNKNOWN 88→87`; expected NO release/schema bump).
- `H.12.5.5.3.3.4.2` — repair the `head.member[idx].method()` parse gap (`context_member_method_call`
  broken; real released-parser bug → root-cause the `a.b[0].c()` rejection, fix the grammar, witnesses
  the rule; full grammar-edit lockstep + release + ledger). **Highest priority** (parser bug).

Each fix child: change ONE thing; rebuild DEBUG `ast_pipeline`; measure GLOBAL cert + `spf` at seeds
0/7/42 + `stimuli_cross_family_platform_gate` + SV external corpus 14/14.

## Artifacts (this session, scratch — not tracked)

`rust/target/generated_logs/h125534_baseline_seed0.txt` (DUMP_ALL),
`…/h125534_probes_seed0.txt` (DEBUG_PROBES), `…/dimc_sample.{sv,ast.json}`,
`…/dimc_min.sv`, `…/dimc_this.sv`, `rust/systemverilog_ast.json` (re-parse dumps),
`rust/t.sv` (the routing/boundary probes).
