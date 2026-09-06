# ENGINE-UNIVERSAL-SERVICES.96 — inline alternation under a quantifier borrows the enclosing rule's templates

Reproducer (interpreter, `ast_pipeline … --interpret-parse in.txt --interpret-parse-ast-json out.json`):

```bash
./rust/target/debug/ast_pipeline docs/tasks/artifacts/engine_universal_services/es96_inline_alt_template/inline_alt.ebnf \
    --interpret-parse docs/tasks/artifacts/engine_universal_services/es96_inline_alt_template/in.txt --interpret-parse-ast-json /tmp/x.json
# rest: [{"body": "+", "kind": "tagged"}, {"first": {"kind": "q"}, "kind": "chain", "rest": "<invalid_sequence_access>"}]   ← DEFECT
./rust/target/debug/ast_pipeline docs/tasks/artifacts/engine_universal_services/es96_inline_alt_template/lifted.ebnf \
    --interpret-parse docs/tasks/artifacts/engine_universal_services/es96_inline_alt_template/in.txt --interpret-parse-ast-json /tmp/y.json
# rest: [["+", "b"], {"kind": "q"}]                                                                                          ← CORRECT
```

`inline_alt.ebnf` spells the chain step as an inline group `( op o | q )*` inside an annotated
two-alternative rule; `lifted.ebnf` lifts the group into the rule `step`. Same language, same
verdict, different trees — and only the lifted one is right. Found by `SV-CORPUS-GRAD.3.41`
(2026-09-04) while spelling `inside { … }` as a chain element of `expression_base`.

## The OPTIONAL form reproduces too (`optional_alt.ebnf`, input `C::m` / `A.m`)

```bash
./rust/target/debug/ast_pipeline docs/tasks/artifacts/engine_universal_services/es96_inline_alt_template/optional_alt.ebnf \
    --interpret-parse docs/tasks/artifacts/engine_universal_services/es96_inline_alt_template/in_opt.txt --interpret-parse-ast-json /tmp/z.json
# C::m -> {"kind": "call", "name": "m", "scope": {"kind": "call", "name": "<invalid_sequence_access>", "scope": "C::"}}   ← template 1 on branch 1
# A.m  -> {"kind": "call", "name": "m", "scope": {"body": "A", "kind": "tagged"}}                                         ← template 0 on branch 0 (and the "." is dropped)
```

So the class is *any* inline alternation nested inside an annotated multi-alternative rule — under a
quantifier OR an optional — typed by branch index against the enclosing rule's templates. A textual
census of `grammars/*.ebnf` (2026-09-04) finds no `*`/`+` form anywhere and the OPTIONAL form in three
SystemVerilog rules: `hierarchical_btf_identifier` (alt 3, `scope: $1`) and
`method_call_receiver_sv_2017` / `_sv_2023` (alt 2, `qualifier: $1`) — the SV-side lift is
`SV-CORPUS-GRAD.3.42`.

## The SINGLE-alternative and UNANNOTATED-alternative forms reproduce too (`SV-CORPUS-GRAD.3.42`, 2026-09-06)

The `.96` textual census counted only MULTI-alternative hosts. Two more synthetics widen the class:

```bash
# a ONE-template host shadows the inner group's branch 0 (template 0 exists), branch 1 passes through
./rust/target/debug/ast_pipeline docs/tasks/artifacts/engine_universal_services/es96_inline_alt_template/optional_alt_single.ebnf \
    --interpret-parse docs/tasks/artifacts/engine_universal_services/es96_inline_alt_template/in_a.txt --interpret-parse-ast-json /tmp/a.json
# A.m  -> {"kind": "call", "name": "m", "scope": {"kind": "call", "name": ".", "scope": "A"}}   ← template 0 on branch 0
# C::m -> {"kind": "call", "name": "m", "scope": "C::"}                                          ← branch 1 passes through
# an UNANNOTATED alternative of an annotated host keeps the RAW envelope, and the corrupted element is visible in it
./rust/target/debug/ast_pipeline docs/tasks/artifacts/engine_universal_services/es96_inline_alt_template/unannot_alt.ebnf \
    --interpret-parse docs/tasks/artifacts/engine_universal_services/es96_inline_alt_template/in_a.txt --interpret-parse-ast-json /tmp/u.json
# element_0 = {"content": {"Json": {"body": "A", "kind": "tagged"}}}                              ← template 0 (of alternative 1) on branch 0
```

`optional_alt_single_lifted.ebnf` / `unannot_alt_lifted.ebnf` are the lifted twins: byte-identical on every
non-shadowed branch (`scope: "C::"` both ways; the absent optional `[]` both ways), correct on the shadowed one
(`scope: ["A", "."]`). ⚠️ Inside an UNANNOTATED alternative the lift wraps a Terminal-returning branch in the
helper's `Alternative{rule_name}` envelope node; a Json-returning branch (every real SV scope rule) is
byte-identical — measured on `function void C::g();` (`SV-CORPUS-GRAD.3.42`).

So the class is: **any inline alternation nested inside a group of a rule that carries at least one return
template, typed by branch index against that rule's templates wherever a template of that index exists.**
The mechanical census over a grammar is `docs/tasks/artifacts/sv_corpus_grad/inline_alt_census/inline_alt_census.py`
(`INLINE-ALT-CENSUS:`; `--self-test` replays the seven files here). On the shipped SystemVerilog grammar at
release 1.0.214 it read **16 exposed sites in 12 rules** where the textual census had read 3 in 3; fifteen were lifted into eight helper rules and the sixteenth respelled as its only live form by
`SV-CORPUS-GRAD.3.42` (release 1.0.215).
