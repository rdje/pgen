# The function / task BODY census (`SV-CORPUS-GRAD.3.44`, `TF-BODY-CLASSIFY:`)

**Which IEEE 1800-2017 A.2.6 / A.2.7 alternative produced every function and task body the corpus writes, and
did the parser TYPE it or leave the raw `Sequence` envelope?** — answered by the parser's own reading of the
`{lifetime, body}` node `function_declaration_sv_2017` / `task_declaration_sv_2017` return (the
`{dynamic_override, lifetime, body}` twin under `sv_2023`), TOOLBOX §1.5f's census with the `--select-keys`
selector this slice added (the slot carries no `kind` to select on), classified by `classify.py` into one bucket
per row: `<function|task>:<alt1..4>:<typed|RAW>`.

```bash
# the census (one lane; the list is every lane file whose text carries the `function` or `task` TOKEN — 4 811 of
# 16 336 on the SV lanes, 396 of 2 606 on the verilog_2005 lane)
python3 docs/tasks/artifacts/sv_corpus_grad/ast_field_census/ast_field_census.py \
    --probe rust/target/release/parseability_probe --profile sv_2017 --files rust/target/sNNN/tf_files_all.txt \
    --select-keys lifetime,body --field body --out rust/target/sNNN/tf_census_sv_2017.tsv --jobs 6
#   … --profile sv_2023 --select-keys dynamic_override,lifetime,body …      (the 2023 declarations carry the override slot)
# the classifier
python3 docs/tasks/artifacts/sv_corpus_grad/tf_body_census/classify.py rust/target/sNNN/tf_census_sv_2017.tsv
python3 docs/tasks/artifacts/sv_corpus_grad/tf_body_census/classify.py --self-test     # 19 arms: 8 raw envelopes (one mis-shaped), 8 typed objects, 3 refusals
```

| bucket | meaning |
|---|---|
| `function:alt1:typed` / `task:alt1:typed` | the non-ported in-block form (`type f; …` / `t; …`), typed since SV-Slice-25 — `{return_type, name, items, statements, end_label}` / `{name, items, statements, end_label}` |
| `…:alt2:…` | the PORTED in-block form `f(ports); …` — `items` holds bare `block_item_declaration` nodes; typed: `ports` present |
| `…:alt3:…` | the out-of-block non-ported form `I.f; …` / `C::f; …`; typed: `scope` present (the `out_of_block_tf_scope` node) |
| `…:alt4:…` | the out-of-block ported form; typed: `scope` and `ports` present |
| `…:RAW` | the raw `Sequence` envelope — decoded to its alternative by the marker tokens (`lparen`, `semi`, the end keyword) and the element count; what alternatives 2–4 emitted through release `1.0.217` |
| `unclassified` | a rendering the reader could not decode — a defect in the classifier, never a bucket to publish over |

A raw and a typed reading of one alternative are told apart by SHAPE, never by the release under test, so the same
classifier reads the HEAD census (alternatives 2–4 RAW) and the final-probe census (every row typed), and the two
must agree row for row on the alternative. The `PREDICTION` block names every ACCEPTED file carrying at least one
RAW body — the corpus AST identity sweep (TOOLBOX §1.5e) on the release must read exactly that set as `ast_moved`
and every other accepted file as byte-identical, two-sidedly — and every accepted file whose bodies are all typed.

`census.md` carries the HEAD reading at release `1.0.217`, the prediction, and the final-probe re-read.

## The partial-typing census (`partial_typing_census.py`, `PARTIAL-TYPING-CENSUS:`)

The CLASS this slice fixed one member of: a rule whose return templates cover some top-level alternatives and not
others. It reads the frontend's `raw_ast` envelope (`ast_pipeline <grammar> --emit-raw-ast-json <raw.json>`) and, per
`PARTIAL` rule, buckets each untyped alternative by what it emits — `passthrough` (one rule reference) or `RAW_ENVELOPE`
(two or more elements: the positional `Sequence` envelope the consumer must index by hand).

```bash
python3 docs/tasks/artifacts/sv_corpus_grad/tf_body_census/partial_typing_census.py rust/target/sNNN/raw.json --list
# PARTIAL-TYPING-CENSUS: grammar=… rules=1610 multi_alternative=558 fully_typed=492 untyped=28 PARTIAL=38 partial_untyped_alternatives=64 passthrough=45 RAW_ENVELOPE=19
python3 docs/tasks/artifacts/sv_corpus_grad/tf_body_census/partial_typing_census.py --self-test
```

Measured on 1.0.217 (`census.md`): 40 PARTIAL rules, 70 untyped alternatives, 25 raw envelopes; after this slice 38 / 64
/ 19 — the residue is owned by `SV-CORPUS-GRAD.3.45`. TOOLBOX §1.5g.

