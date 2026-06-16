# GRAMMAR-WELLFORMED.H.12.5.5.3.3.4.2 — `context_member_method_call`: RE-ADJUDICATION

Tools-first WHY+WHERE re-adjudication of the leaf `H.12.5.5.3.3.4.2`, opened by
`H.12.5.5.3.3.4` (`-0097`) as **"a real latent released-parser bug — repair the grammar so
`a.b[0].c()` parses."** This slice (`PGEN-GRAMMAR-WELLFORMED-0098`, PURE-DOCS) **refutes that
premise** and re-routes the leaf. No code/grammar/generated/release/schema/ledger change.

> Scope: layer-B detail behind the `H.12.5.5.3.3.4.2` row. The `-0097` adjudication made two
> tool-checkable claims about `context_member_method_call` (its "Carrier 2 / verdict (a′)"). Both
> are **false** on re-verification. This file supersedes the Carrier-2 verdict of
> [[GRAMMAR-WELLFORMED-H125534-cii-residual-parent-commit-adjudication]].
> ([[feedback_be_alert_root_cause_fishy_immediately]], [[feedback_why_and_where_before_solution]],
> [[feedback_tools_first_no_guessing]], [[feedback_no_codebase_change_without_tool_backed_facts]].)

## Baseline (reproduced this session — deterministic ⇒ signal)

```
PGEN_CERT_COVERAGE_DUMP_ALL=1 ./target/debug/ast_pipeline ../grammars/systemverilog.ebnf \
  --report-certificate-coverage --grammar-profile sv_2017 \
  --entry-rule systemverilog_file --count 40 --seed 0
# total=1292 proof=1 witness=1203 UNKNOWN=88 fully_certified=false (spf=0, proof_reverify=0)
```

Byte-identical to the post-`-0096`/`-0097` layer-A pointer. `context_member_method_call` (and
`direct_index_method_call`) confirmed present in the 88-rule UNKNOWN dump.

## Refutation 1 — the `@predicate` IS live (not "stale dead text")

`-0097` claimed: *"the generated parser carries NO `@predicate` on it … the `:2915` comment is
stale dead text."* **False.** `generated/systemverilog_parser.rs:3041-3055`:

```rust
directives_by_rule.insert(
    "context_member_method_call".to_string(),
    vec![ SemanticRuntimeDirective::Predicate(SemanticPredicateSpec {
        name: "has_fact",
        args: [ Identifier("variable_binding"), RuleReference("head") ],
        phase: Post, view: Raw, }) ],
);
```

The grammar's `@predicate: { name: has_fact, args: [variable_binding, $head], phase: post }`
(`systemverilog.ebnf:2892`) binds to `context_member_method_call` (`:2906`) across the
intervening comment block exactly as the meta-grammar prescribes (`ebnf.ebnf:70`,
[[reference_annotation_binds_following_rule]]) and **is emitted**. The `-0097` grep almost
certainly hit a one-line-grep trap: rustfmt wraps `directives_by_rule.insert(` and the rule-name
string onto separate lines, so `grep 'context_member_method_call.*insert'` returns nothing.

## Refutation 2 — `a.b[0].c()` PARSES (and witnesses the rule) with a declared head

`-0097` claimed: *"its OWN designed form `a.b[0].c()` is REJECTED parser-wide … rejected even with
a declared head."* **False.** With the head bound as a variable (`int`/`logic`/`bit`), the form
parses and the accepted AST contains a `context_member_method` node:

| input (sv_2017, `--parse`) | result | matched rule (AST) |
|---|---|---|
| `module m; int a; int x; initial x = a.b[0].c(); endmodule`     | **PARSE OK** | `context_member_method` |
| `module m; int a; int x; initial x = a.b.c();    endmodule`     | PARSE OK | `context_member_method` |
| `module m; logic [3:0] a; int x; initial x = a.b[0].c(); endmodule` | PARSE OK | `context_member_method` |
| `module m; bit a; int x; initial x = a.b[0].c(); endmodule`     | PARSE OK | `context_member_method` |
| `module m;        int x; initial x = a.b[0].c(); endmodule` (head **undeclared**) | REJECT (furthest=43) | — (predicate fails) |

The rejection in the undeclared case is **correct by design**: `context_member_method_call` is the
`post`-predicate-gated "regression firewall" the grammar comment (`:2876-2889`) describes — it
fires only for known-variable heads, so hierarchical/scoped/package names fall through to the
unchanged branches. With `int a;` declared, `has_fact(variable_binding, a)` holds (emitted by
`variable_decl_assignment`, `generated:3099-3108`), the predicate passes, and the rule witnesses.

**`-0097`'s failing tests never established the binding.** Reproduced:

| `-0097`'s form | result | why |
|---|---|---|
| `class C; function void f; int x; x = a.b[0].c(); endfunction endclass` | REJECT (47) | `a` undeclared → no binding → predicate fails (correct) |
| `module m; C a; int x; initial x = a.b[0].c(); endmodule` (C undeclared) | REJECT (13) | rejects at the `C a;` decl itself — `C` is an undeclared type, so `a` is never bound |

So the adjudication observed "rejected" but mis-attributed it to a broken rule; the real cause was
an unbound head in every test. **There is NO released-parser bug for `head.member[idx].method()`.**

## True root cause of the cert `UNKNOWN` — a store-gated WITNESS-REACH gap

`context_member_method_call` sits in `UNKNOWN` because the cert-coverage witness passes cannot
**generate** a sample that satisfies its `has_fact(variable_binding, $head)` gate: the witnessing
sample needs a fact-emitting **prelude** — a variable declaration whose *name matches the chain
head* — that no minimal reach derivation contains. This is exactly the book's **semantic-prelude
reach** class (Grammar Well-Formedness chapter, "Reaching store-gated rules"), one variant deeper
than the regex `\NN` case:

- regex `\NN` is gated by `fact_count_at_least(regex_capture_group, $index)` — a **count** gate;
  the existing C2.2 pass (`stimuli_generator.rs:2769 compute_reach_prelude`) is keyed purely on
  `gen_count_kinds` (`fact_count_at_least`) and emits *N* anonymous copies — **no name coupling**.
- `context_member_method_call` is gated by `has_fact(variable_binding, $head)` — a **has_fact**
  gate whose satisfaction requires a producer (`variable_decl_assignment`) emitting a binding whose
  **name equals** the identifier the gated rule renders as `$head`.

The C2.2 MVP handles neither `has_fact` kinds nor name-coupling, so the reach pass never builds
`int a; … a.b[0].c()` and the rule stays `UNKNOWN`. By the attribution rule this is a **generator
constructor gap**, fixed in the *generator* (the witness pass), never in the grammar — the linter
confirms the rule reachable and the parser confirms it witnessable, so chasing a grammar edit would
be a workaround ([[feedback_no_workarounds_fix_hierarchy]], [[project_ebnf_is_single_source_of_truth]]).

→ fix child **`H.12.5.5.3.3.4.2.1`** (generator capability; NO grammar/release/ledger).

## Secondary finding (separate ticket) — class-handle heads do not witness the indexed chain

A sharp, *different* boundary surfaced while characterizing the binding requirement:

| head decl (C a declared class) | chain | result | route |
|---|---|---|---|
| `C a;` | `a.b.c()`    | PARSE OK | `ident_postfix_chain` / `split_hierarchical` (binding-free) |
| `C a;` | `a.b[0].c()` | **REJECT** (furthest at the `[`) | — |
| `int a;` | `a.b[0].c()` | PARSE OK | `context_member_method` |

I.e. with an integral-typed head the indexed 3-level chain witnesses via
`context_member_method_call`; with a **class-handle**-typed head (the *realistic* uvm shape —
`urme_container.elements[i].clone()`, a class object) the same chain rejects, because `C a;` does
not lead the rule to fire (its `has_fact(variable_binding, a)` gate is not satisfied the way
`int a;`/`logic a;`/`bit a;` satisfy it — class-typed object declarations route through a different
declaration rule / binding-emission path). This is a **potential real parser gap for class-handle
member-method chains** — a candidate bug-finding-oracle hit — and is filed as its own WHY+WHERE
leaf **`H.12.5.5.3.3.4.2.2`**, NOT this leaf's cert blocker (the integral-head witness already
proves `context_member_method_call` reachable+witnessable, which is all the cert closure needs).

## Conclusion

- `-0097`'s Carrier-2 verdict (a′) "real latent released-parser bug → repair the grammar" is
  **refuted, tool-backed**: the predicate is live, the form parses with a declared head, and the
  rule witnesses. **No grammar fix, no release bump, no `PGEN_RELEASED_PARSER_BUG_LEDGER` row.**
- The cert `UNKNOWN` is a **store-gated witness-reach gap** (the `has_fact` analogue of the regex
  `\NN` semantic-prelude class) → generator fix child `.4.2.1`.
- A **separate** potential parser gap (class-handle head + indexed member-method chain) → WHY+WHERE
  child `.4.2.2`.
- Carrier 1 (`direct_index_method_call`, leaf `.4.1`) is untouched by this finding, **but** because
  this re-adjudication shows the `-0097` adjudication was unreliable on two checkable claims, the
  `.4.1` "effectively dead / subsumed" premise must be **independently re-verified** (its decisive
  A/B is mandatory regardless) before any removal.

## Fix / investigation children (split this leaf)

- `H.12.5.5.3.3.4.2.1` — witness `context_member_method_call` by extending the semantic-prelude
  reach (C2.x) to `has_fact`-gated rules: emit a binding-producer prelude (a
  `variable_decl_assignment` declaring a variable) whose emitted name is coupled to the gated
  rule's `$head` render. Closes cert `UNKNOWN 88→87`. **Generator capability, parser-agnostic, inert
  for the fully-certified roster; NO grammar/release/schema/ledger change.**
- `H.12.5.5.3.3.4.2.2` — WHY+WHERE: does a declared class-handle (`C a;`) emit `variable_binding`,
  and should `head.member[idx].method()` on a class handle parse? Tools-first; if class-typed object
  declarations do not bind the head, adjudicate real-gap-vs-by-design (the realistic uvm shape).

## Artifacts (this session, scratch — not tracked)

`rust/target/generated_logs/h1255334_2/` — `cert_seed0.txt` (DUMP_ALL baseline),
`decl_ab0c_call.sv`/`.ast.json` (the witnessing parse + AST node), the boundary-matrix `.sv`
inputs (`int_*`, `classC_*`, `adj_*`, `logic_ab0c`, `bit_ab0c`), `classC_ab0c.trace.txt`.
