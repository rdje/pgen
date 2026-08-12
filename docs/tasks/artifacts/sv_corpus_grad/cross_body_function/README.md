# `SV-CORPUS-GRAD.13c.2d` — the pinned reproducer

`cross_function.sv` is IEEE 1800-2017 §19.6.1's own shape: a `covergroup` whose `cross` body declares
a function and then uses it in a `bins`. It is the probe that pins the defect, and it is preserved
here rather than left in a scratch directory because the measurement — not the sentence describing it
— is what the next session needs (`PGEN-SV-CORPUS-GRAD-0214`'s lesson, applied).

```bash
./rust/target/release/parseability_probe --parse systemverilog \
  docs/tasks/artifacts/sv_corpus_grad/cross_body_function/cross_function.sv --profile sv_2017
# → REJECT, furthest_position=107  (byte 107 = the newline right after the cross body's `{`)

./rust/target/release/parseability_probe --parse systemverilog \
  docs/tasks/artifacts/sv_corpus_grad/cross_body_function/cross_function.sv --profile sv_2023
# → parse_full passed
```

The profile split IS the defect: `cross_body_item_sv_2023` references the real `function_declaration`
nonterminal (`grammars/systemverilog.ebnf:1838`), while `cross_body_item_sv_2017` matches the LITERAL
text `function_declaraton` — the 1800-2017 Annex A A.2.11 typo, transcribed as a token
(`:1834` → `:6591`).

⚠️ When the fix lands, this file must flip to ACCEPT under **both** profiles, and the accepts-invalid
set must be held: `cross_body_sv_2017`'s `( cross_body_item semi )*` demands a `;` after every item,
so re-pointing the reference alone would then demand `endfunction ;`. See the leaf for the full
argument.
