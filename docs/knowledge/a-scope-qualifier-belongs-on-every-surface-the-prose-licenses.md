---
id: a-scope-qualifier-belongs-on-every-surface-the-prose-licenses
title: A scope qualifier admitted on ONE surface (the operand) must be admitted on EVERY surface the standard's prose licenses (the call, the class scope, the handle) — and a reserved word had been hiding the missing ones
answers:
  - "local::get_x() inside randomize() with { } REJECTS on 1.0.211 while x < local::x parses — why, and what fixed it"
  - "a grammar release reserved a word and legal text stopped parsing — how do I find every surface the word was standing in for"
  - "adding A.8.4 class_qualifier's local:: to primary — which other surfaces does IEEE 1800-2017 18.7.1 license it on"
  - "which SV grammar rule carries a scoped tf-name (scoped_or_hierarchical_tf_identifier) and how does local::get_c().c1_f derive as a call chain"
  - "how do I prove a widening does not move an existing control's tree before generating (interpreter --interpret-parse-ast-json, kind multiset, byte-identity)"
  - "how was the dead block_data_type_or_implicit deleted and the syntax-closure allow-list returned to one"
  - "Annex A has no production for a text the clause prose plainly permits — annex gap or prohibition"
tags: [systemverilog, grammar, lrm-fidelity, corpus, pins, interpreter, annex-gap, release-ceremony]
date: 2026-09-04
status: current
evidence: "SV-CORPUS-GRAD.3.40 + .3.36 (PGEN-SV-CORPUS-GRAD-0367, release 1.0.212, ledger SV-0085). Release 1.0.211 reserved Annex B's 248 words and gave `local ::` its A.8.4 class_qualifier production in `primary_hier_scope_prefix` — the OPERAND surface. verilator t_randomize_method_with_scoping.v:70 `doit &= f.randomize() with { get_x() == local::get_x(); };` then REJECTED (furthest_position=1337, on the `(`): the scoped trace shows `primary_hier_scope_prefix` consuming ` local::`, `primary_sv_2017` winning as a hierarchical name, and no `call_primary` alternative admitting the prefix — `tf_call`'s scoped name rule `scoped_or_hierarchical_tf_identifier` had class_scope and package_scope arms only. It had parsed on 1.0.210 only because `local` was an admissible identifier standing in for a package name. IEEE 1800-2017 18.7.1 (section-18-constrained-random-value-generation.md:1708-1726): the qualifier applies to *an identifier within an in-line constraint* and *may be used to qualify class scopes and type names*; A.8.4 reaches primary and ps_type_identifier and no tf_call production — an ANNEX GAP by the grammar's standing rule, not a prohibition. Fix: one arm `local :: [ implicit_class_handle . | class_scope ] tf_identifier` on the scoped tf-name rule; the reproducer, the corpus row, `local::get_c().c1_f`, `local::K::m()` and `local::this.f()` all parse on sv_2017 and sv_2023; the two existing controls' interpreter ASTs are BYTE-IDENTICAL; the corpus moved exactly that one row fail -> pass on both SV lanes and nothing the other way. In the same release the dead `block_data_type_or_implicit` (orphaned by .3.35's footnote-6 split) was deleted: 1576 -> 1575 rules, sv_syntax_closure_gate unreachable_rules 2 -> 1, allow-list back to [casting_type]."
reverify: "python3 stimuli/sv/run_adjudication_repros.py | tail -1   # failures=0; fixed_local_scope_subroutine_call.sv ACCEPT on both SV profiles and control_local_scope_qualifier.sv unmoved; ./rust/target/release/parseability_probe --parse systemverilog --profile sv_2017 stimuli/sv/subs/verilator/test_regress/t/t_randomize_method_with_scoping.v"
---

**A qualifier production is a claim about EVERY surface the prose licenses, not about the one
surface a reproducer happened to exercise.** IEEE 1800-2017 §18.7.1 says `local::` modifies the
resolution of *an identifier within an in-line constraint* and *may be used to qualify class scopes
and type names*. Annex A spells that out for `primary` (A.8.4 `class_qualifier`) and for
`ps_type_identifier`, and stops. Release 1.0.211 read Annex A, added the operand arm, and shipped —
and the next corpus run said one legal row had been NARROWED: `get_x() == local::get_x()` inside
`randomize() with { … }`. The qualifier on a CALL had no production, and never had; until 1.0.211
it parsed only because `local` was an admissible identifier that `package_scope` happily consumed.

**Reserving a word turns every place it was silently standing in for a name into a hole, and the
holes are exactly the surfaces Annex A forgot.** `.3.37` already paid for the first instance of this
(`null` in `primary`); this is the second, one surface over. The discipline that catches it BEFORE
the corpus does: when a production admits a qualifier or a keyword on one surface, enumerate the
surfaces the CLAUSE text licenses — operand, call, type name, class scope, implicit handle — write
the legal twin for each, and run them on the generated parser. The pin for this slice carries all
five spellings (`local::get_x()`, `local::get_c().c1_f`, `(local::get_c).c2_f`, `local::K::m()`,
`local::this.f()`) so the next widening of the reserved set cannot narrow one of them silently.

**The toolbox told the story in three commands, and no eyeballing would have.** `--parse` gave
`furthest_position=1337` (the `(`); `--trace-rules constraint_expression` at `PGEN_TRACE_VERBOSITY=high`
showed `primary_hier_scope_prefix` consuming ` local::` (branch 4/4), `hierarchical_identifier`
consuming `get_x`, and `primary_sv_2017` winning as a hierarchical name — so the CALL surface was
the one without a derivation; `--parse-dump-ast-pretty` on the unqualified twin showed which rule
carries an unqualified call in that position (`call_primary/plain_tf`, and `call_postfix_chain` for
the `.c1_f` chain), which is where the qualified arm had to land. The minimal arm — on the scoped
tf-name rule, the shape A.8.4 gives `class_qualifier` — reaches every call site at once, including
the postfix chain, because they all consume `tf_call_with_args`.

**Prove shape-neutrality BEFORE generating, byte for byte.** A widening that changes how an
existing legal text derives is a consumer-visible change even when every verdict stays the same
(`.3.35` learned it on initialisers). The interpreter answers it in seconds:
`--interpret-parse --interpret-parse-ast-json` on the existing controls under HEAD's grammar and the
candidate, then compare — here the JSON was byte-identical for the primary-position control
(`x < local::x`) and for the unqualified call chain. Only then regenerate.

**A dead rule is a well-formedness defect, and the allow-list that carried it says when to delete
it.** `.3.35` orphaned `block_data_type_or_implicit` and allow-listed it (ONE → TWO) rather than mix a
deletion into a four-correction release, naming `.3.36` as the owner. Batched with this release, the
deletion is one grammar line and the contract returns to `["casting_type"]` by the good-news branch
— the gate re-stamped `unreachable_rules=1 unreachable_branches=2` itself. Whether a rule is dead is
decided by the gate's `unreachable_from_entry` set, never by grep: `data_type_or_implicit`, which
the leaf's opening text suspected, has six live references and the gate never listed it.
