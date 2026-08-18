# `LANG-CAPABILITY-AUDIT.10.18` — the possessive-quantifier demonstration

Five lines of EBNF that show PGEN's `*` never gives back an iteration.

```bash
printf 'ab'  > rust/target/in.txt
./rust/target/debug/ast_pipeline docs/tasks/artifacts/lang_capability_audit/possessive_quantifier/quant_backtrack.ebnf \
    --interpret-parse rust/target/in.txt
# INTERPRET-PARSE: … accepted=false furthest_position=1 error="Backtrack { position: 1 }"

printf 'aab' > rust/target/in.txt
./rust/target/debug/ast_pipeline docs/tasks/artifacts/lang_capability_audit/possessive_quantifier/quant_backtrack.ebnf \
    --interpret-parse rust/target/in.txt
# INTERPRET-PARSE: … accepted=false furthest_position=2 error="Backtrack { position: 2 }"
```

Both inputs are in the language `s := ( a )* a b` declares. A backtracking `*` accepts each of
them; PGEN's rejects both, and reports nothing about the grammar.

⛔ The verdict is the interpreter's (TOOLBOX 1.5b). It is authoritative here by VERIFICATION, not
by construction: `parse_harness_combinator_gate` (TOOLBOX 1.7) pins the interpreter byte-identical
to the compile-and-run oracle per structural combinator, `quant_star` included, and this grammar has
no left recursion — the one surface where that pinning is measured NOT to hold
(`ENGINE-UNIVERSAL-SERVICES.14`).
