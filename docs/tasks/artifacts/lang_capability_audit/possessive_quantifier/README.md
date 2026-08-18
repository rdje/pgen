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

## `?` is possessive too — added 2026-08-18 under DIRECTOR CHALLENGE

The first cut of this artifact measured `*` only, and the leaf it backs said "`*` / `+`". Challenged,
the optional was measured and behaves the same way (`quant_backtrack_optional.ebnf`):

```bash
# s := ( a )? a b
printf 'ab'  > rust/target/in.txt   # -> accepted=false   ⛔ 0 iterations + `a` + `b` IS in the language
printf 'aab' > rust/target/in.txt   # -> accepted=true    (1 iteration + `a` + `b`)
```

⇒ `?` greedily takes the optional and never gives it back, so `ab` — which the grammar declares —
rejects. **All three quantifier forms are possessive.** This matters for the census in the owning
leaf: a lookahead written inside a `( … )?` is the same workaround as one inside a `( … )*`, which is
what took the measured count from six to eight.
