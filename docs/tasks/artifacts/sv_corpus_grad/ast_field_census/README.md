# The AST-field census (`SV-CORPUS-GRAD.3.43`, `AST-FIELD-CENSUS:`)

**What does a corpus actually WRITE at one typed-AST slot?** — answered by the parser's own reading, not by a grep.

`ast_field_census.py` runs the release probe's `--parse-dump-ast` (TOOLBOX §1.2) over a file list, selects every node
whose `kind` is the one named, reads a dotted field path off each and writes one TSV row per node with the field's
compact JSON verbatim. Files the probe rejects (or times out on) carry no tree: they are COUNTED as rows of their own
and, with `--fallback-regex`, scanned textually — those rows are tagged `regex` and never mixed with the parser's
reading. A domain classifier then reads the TSV (first use: `../dpi_result_census/classify.py`).

```bash
python3 docs/tasks/artifacts/sv_corpus_grad/ast_field_census/ast_field_census.py --probe rust/target/release/parseability_probe --self-test
python3 docs/tasks/artifacts/sv_corpus_grad/ast_field_census/ast_field_census.py \
    --probe rust/target/release/parseability_probe --profile sv_2017 --files rust/target/sNNN/dpi_files.txt \
    --kind import_function --field proto.body.body.return_type --out rust/target/sNNN/dpi_result_census.tsv --jobs 8 \
    --fallback-regex 'import\s+"DPI(?:-C)?"\s+(?:context|pure)?\s*(?:(?:\\\S+|\w+)\s*=\s*)?function\s+(?P<field>.+?)\s+(?:\\\S+|\w+)\s*[(;]'
# AST-FIELD-CENSUS: kind=import_function field=proto.body.body.return_type profile=sv_2017 files=172 accepted=95 rejected=77 timeout=0 nodump=0 nodes=297 distinct=73 fallback_rows=1001
```

| column | meaning |
|---|---|
| `file` | repo-relative path from the list |
| `status` | `accepted` / `rejected` / `timeout` / `nodump` — the probe's verdict on the whole file |
| `index` | the node's position among the selected nodes of that file (source order); `regexN` for a fallback row; `-` for a file with no row |
| `value` | the field's compact JSON (`sort_keys`, no spaces); the captured text for a `regex` row; `-` when nothing matched |

Exit 0; **exit 2** when the list is empty or the probe is not executable; **exit 7** when `--self-test` misses (two
synthetic files: an accepting one with two imports must yield exactly `int` and `void` in source order, a rejecting
one must yield exactly one `regex` row). `--field` walks dict keys and, for a digit segment, list indices; a missing
segment renders `<missing:seg>` rather than dropping the row.

⛔ **The census is only as strong as its file list** (the `.3.38` pre-screen rule). Build the list with a selector that
cannot miss the construct — for a DPI import the spec string `"DPI`, which every import carries — and check it against
the rows already known by name before trusting the count (`.3.43`: 48 of the 49 verilator `t_dpi_*` files selected;
the 49th, `t_dpi_accessors.v`, carries its imports in an included `.vh`, which the lanes do not parse either).

⛔ **A `regex` row sits in a file the shipped parser already rejects**, so it cannot move a lane verdict; it is in the
census so the count is of the corpus, not of the parser's accepting half. The parser's rows are exact; the regex is a
fallback, and its capture is checked by the classifier (an `unclassified` row means the regex or the classifier is
short — the `.3.43` census carried three until the name pattern admitted escaped identifiers `\$dpii_display`).
