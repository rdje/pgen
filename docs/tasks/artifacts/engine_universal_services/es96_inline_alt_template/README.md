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
