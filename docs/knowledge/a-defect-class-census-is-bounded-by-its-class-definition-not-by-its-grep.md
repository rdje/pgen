---
id: a-defect-class-census-is-bounded-by-its-class-definition-not-by-its-grep
title: A census of a defect class finds exactly the sites its CLASS DEFINITION admits — the .96 "inline alternation typed by the enclosing rule's templates" census read 3 sites while the class had 16, because it was defined as "multi-alternative hosts" and a one-template host shadows branch 0 too
answers:
  - "the .96 textual census said three SV rules carry the inline-alternation shape — why did the lift slice find sixteen sites in twelve rules"
  - "does a rule with a SINGLE return template also corrupt an inline ( a | b )? group — which branch"
  - "an UNANNOTATED alternative of an annotated rule — is its inline alternation typed too, and where does the corrupted node show"
  - "how do I census a grammar for inline alternations that a return template can type by branch index (INLINE-ALT-CENSUS)"
  - "foreach (this.arr[i]) gave scope: {name: {kind: dot}, scope: {kind: this}} — what is that and what fixed it"
  - "function void I.h(); endfunction produced six invalid_sequence_access in an accepting file — why"
  - "@@(begin C::m) rejected on 1.0.213 — was that an over-rejection or the store gate"
  - "why did the class_scope branch of an out-of-block function definition NOT move when its group was lifted, while the interface branch did"
  - "a tracked instrument crashed with docs/stimuli/... — parents[4] vs parents[5] — was the tracked copy ever run"
  - "the union certificate witnessed a new helper rule at seeds 0 and 7 but not at 42 — why, and what fixed it"
  - "a lifted helper whose branches are dead by tie or store-gated — lift it or delete the dead forms"
  - "what result types / what forms does the corpus actually write at one AST slot — census it by grep or by the parser"
  - "my corpus grep over-matched across a comment and an escaped identifier — how do I make a corpus census exact"
tags: [systemverilog, grammar, ast-shape, census, instrument, schema-bump, interpreter, engine-defect, release-ceremony]
date: 2026-09-06
status: current
evidence: "SV-CORPUS-GRAD.3.42 (release 1.0.215, ledger SV-0088, schema 27 -> 28). ENGINE-UNIVERSAL-SERVICES.96's textual census (2026-09-04) defined the class as 'an inline alternation inside an annotated MULTI-alternative rule' and found 3 sites. Two synthetics widened it: optional_alt_single.ebnf (one template) types branch 0 -> A.m gives scope: {kind: call, name: '.', scope: 'A'}; unannot_alt.ebnf (an alternative WITHOUT a template inside a rule WITH one) types branch 0 and the corrupted element sits in the raw Sequence envelope. docs/tasks/artifacts/sv_corpus_grad/inline_alt_census/inline_alt_census.py then read 16 exposed sites in 12 SV rules on release 1.0.214 (constraint_primary_sv_2017/_2023, hierarchical_btf_identifier, method_call_receiver_sv_2017/_2023 x2 each, nonrange_variable_lvalue, ps_or_hierarchical_array_identifier, ps_type_identifier_sv_2017/_2023, split_hierarchical_callable_receiver, function_body_declaration x2, task_body_declaration x2), and real SV text through them: foreach (this.arr[i]) -> scope {name: {kind: dot}, scope: {kind: this}}; solve this.x before this.y / disable soft this.x / this.arr = new[4] -> select: <invalid_sequence_access> (4 on sv_2017, 7 on sv_2023); function void I.h(); task I.t(); -> 6 <invalid_sequence_access>; covergroup cg @@(begin C::m) with class C declared -> scope {kind: block, ...}. The release probe (codegen) reproduced the btf node byte for byte with the interpreter. Fifteen lifted into eight unannotated SV-gated helper rules, the sixteenth (hierarchical_btf_identifier alt 3) respelled as its only live form class_scope method_identifier because a helper there left the union certificate seed-dependent; interpreter HEAD vs candidate: every corruption probe MOVED with invalid_sequence_access -> 0, every control (p::q.m(), top.m, the class_qualifier pin, verible nested_member_access.sv, function void C::g()) byte-IDENTICAL."
reverify: "python3 docs/tasks/artifacts/sv_corpus_grad/inline_alt_census/inline_alt_census.py --self-test && python3 docs/tasks/artifacts/sv_corpus_grad/inline_alt_census/inline_alt_census.py grammars/systemverilog.ebnf | head -1   # self-test ok (7 controls); exposed(...)=0 on the shipped grammar; ./rust/target/debug/ast_pipeline docs/tasks/artifacts/engine_universal_services/es96_inline_alt_template/optional_alt_single.ebnf --interpret-parse docs/tasks/artifacts/engine_universal_services/es96_inline_alt_template/in_a.txt --interpret-parse-ast-json /tmp/a.json && grep -c '\"kind\": \"call\", \"name\": \".\"' /tmp/a.json   # 1 until ENGINE-UNIVERSAL-SERVICES.96 lands"
---

**A census is an instrument, and an instrument measures the class it was DEFINED for.** The
`ENGINE-UNIVERSAL-SERVICES.96` finding was correct in mechanism — an inline alternation `( a | b )`
inside a group of an annotated rule is typed with the enclosing rule's return template of the same
branch index — but its textual census was written against the reproducer's shape, *"an annotated
MULTI-alternative rule"*, because the reproducer had two alternatives. Three sites. Two five-line
synthetics measured before the lift slice touched the grammar changed the class definition:

1. **A ONE-template host shadows branch 0.** `s := ( a b | c )? d -> {kind: "call", scope: $1, name: $2}`
   on `A.m` yields `scope: {kind: "call", name: ".", scope: "A"}` — template 0 exists, branch 0 wears it;
   branch 1 (`C::`) passes through. Nothing about "multi-alternative" was load-bearing.
2. **An UNANNOTATED alternative of an annotated host is typed too, and the corruption is VISIBLE in its raw
   envelope.** `s := t -> {kind: "tagged", body: $1} | ( a b | c ) d` on `A.m` puts
   `{"body": "A", "kind": "tagged"}` into `element_0` of the raw `Sequence`.

The corrected definition — *any inline alternation nested inside a group of a rule with at least one
template, typed by branch index wherever a template of that index exists* — is what
`inline_alt_census.py` implements, and on the same grammar it read **16 sites in 12 rules**. Five of them
emitted corrupted nodes on ordinary SystemVerilog in ACCEPTING files: `foreach (this.arr[i])`,
`solve this.x before this.y`, `disable soft this.x`, `this.arr = new[4]`, `function void I.h();`. A census
that reports its class definition alongside its count can be challenged; one that reports a count alone
reads as the population.

**A shadowed branch may be LIVE only when the store agrees — do not read a store-gated rejection as an
over-rejection.** `@@(begin C::m)` REJECTED at the `)` on 1.0.213, which the leaf had filed as a second,
over-REJECTION question. `--trace-rules hierarchical_btf_identifier,class_scope` showed why: with no
`class_name` fact for `C`, `class_scope_type_head` took `C::m` as `package::class` and `class_scope` then
demanded another `::`. With `class C;` declared the same text ACCEPTS and the shadowed branch wins — and
emits `scope: {kind: "block", body: …}`, alternative 2's template around the class_scope node. The
liveness question for a store-gated branch is answered with its facts planted, never on a bare fragment.

**The lift is byte-identical exactly where nothing was wrong.** Lifting the group into a named,
unannotated rule (`.3.41`'s `expression_chain_step`) passes every branch through as its own node: the
non-shadowed branches (`C::`, `pkg::`) and the absent optional (`[]`) are byte-identical HEAD vs candidate
on both synthetics AND on the shipped grammar; only the shadowed branches move — from a wrong node to the
right one. One nuance measured rather than assumed: inside an UNANNOTATED alternative the lift wraps a
Terminal-returning branch in the helper's `Alternative{rule_name}` envelope, but a Json-returning branch
(every real SV scope rule) is byte-identical — `function void C::g();` did not move while
`function void I.h();` did.

**An instrument is tracked when the TRACKED copy has produced a number.** The grammar-candidate
pre-screen scripts were tracked by `.3.38` with `R = parents[4]` — `docs/` — so the tracked form could
never have run; `.3.38` had measured with the scratch copies and credited the tracked one. Found the
moment the tracked form was invoked from its own location. Both now derive `parents[5]` and REFUSE when
the derived root holds no `grammars/systemverilog.ebnf`.

**A helper whose only witnessable branch is store-gated cannot be certified deterministically — drop the dead forms
instead of lifting them.** The first spelling lifted `hierarchical_btf_identifier`'s group into a ninth helper and the
union certificate refused on it: the dotted branch is dead by tie everywhere (alternative 1 wins the same bytes), the
class-scope branch is store-gated for the targeted plan, and a seventh union config witnessed it at seeds 0 and 7
only — through the entry's random diverse pass, not the targeted probe. A witness that depends on the seed is a coin.
The closure discipline's own rule for dead branches applied: the alternative is spelled as its only live form,
`class_scope method_identifier`, the helper is gone, and the six-config union is deterministic again.

## A corpus census reads the parser's own AST slot; a grep is the fallback for the rows the parser cannot read (`SV-CORPUS-GRAD.3.43`, 2026-09-06)

The same bound applies to a census over a CORPUS: its class definition is the AST slot, and the exact reader of that
slot is the parser. `docs/tasks/artifacts/sv_corpus_grad/ast_field_census/ast_field_census.py` (`AST-FIELD-CENSUS:`)
dumps every file's typed AST with the release probe and reads one dotted field off every node of one `kind` — the
`.3.43` census of DPI import RESULT types read 297 slots in 95 accepting files exactly, where a regex over the same
files first over-matched across a `/*verilator sformat*/` comment and an escaped identifier `\$dpii_display`. The
regex stays, tagged, for the files the parser REJECTS (they have no tree, and they are part of the corpus count) — and
a domain classifier with an `unclassified` bucket is what shows the regex is short. A grep is not a census of the
corpus; it is a census of what the grep admits.
