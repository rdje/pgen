# The compiler-directive census (`SV-CORPUS-GRAD.3.39`, `DIRECTIVE-CLASSIFY:`)

**What does the corpus actually WRITE on its compiler-directive lines, and which of those lines do the
IEEE 1800 clause 22 / IEEE 1364-2005 clause 19 syntax boxes refuse?** — answered by the parser's own
reading of the `compiler_directive` slot (TOOLBOX §1.5f, `ast_field_census.py --kind compiler_directive
--field body`), classified by `classify.py` against the boxes, one bucket per row.

```bash
# the census (one lane; the list is every lane file whose text carries a typed-name directive TOKEN — `\`(name|…)\b` anywhere,
# NOT a column-anchored line: .3.39's anchored selector missed 19 files, 4 of them lane movers; .vh excluded — not lane rows)
python3 docs/tasks/artifacts/sv_corpus_grad/ast_field_census/ast_field_census.py \
    --probe rust/target/release/parseability_probe --profile sv_2017 --files rust/target/sNNN/directive_files_lane.txt \
    --kind compiler_directive --field body --out rust/target/sNNN/directive_census_sv_2017.tsv --jobs 6 \
    --fallback-regex '(?m)^[ \t]*(?P<field>`(?:resetall|undef|undefineall|timescale|default_nettype|unconnected_drive|nounconnected_drive|celldefine|endcelldefine|pragma|line|begin_keywords|end_keywords)\b[^\r\n]*)'
# the classifier (the version_specifier list follows the profile's edition)
python3 docs/tasks/artifacts/sv_corpus_grad/directive_census/classify.py rust/target/sNNN/directive_census_sv_2017.tsv --profile sv_2017
python3 docs/tasks/artifacts/sv_corpus_grad/directive_census/classify.py --self-test     # 62 arms + a RED arm + 2 macro_body arms
```

| bucket | meaning |
|---|---|
| `legal:<name>` | the arguments match the clause's syntax box — the typed grammar accepts the line |
| `ILLEGAL:<name>:<reason>` | they do not — the typed grammar refuses the line (`bad_time_unit`, `bad_level`, `unknown_version`, …) |
| `TRAILING:<name>` | the directive is legal and non-comment text follows on the same line; the grammar hands that text to the NEXT item, so the row is listed for reading rather than judged |
| `opaque:<name>` | `include`, `define`, the conditional family, a text-macro use, a vendor directive — the residue the parser still tolerates as one opaque line at file top level (`.3.14b`), untouched by `.3.39` |
| `macro_body:<name>` | the line is a CONTINUATION line of a `` `define `` (the previous physical line ends in `\`): macro text, preprocessor-owned |
| `no_node` | the file list is a TEXT selection and the parser read no directive node in this accepted file (its backtick lines sit inside a comment or a string) |
| `unclassified` | the reader could not name the directive — a defect in the classifier, never a bucket to publish over |

The `PREDICTION` block names every ACCEPTED file carrying an `ILLEGAL` / `TRAILING` typed row — the release
build must move exactly those `pass -> fail` — and every already-REJECTED file whose regex rows carry one,
whose failure position may move (the adjudicator's explanation is re-read for each).

⛔ **Two transcriptions of one clause agree on a transcription slip and never on a shared misreading.** The
classifier and the grammar were both written from the boxes; their agreement over the corpus is the check
that neither dropped a form, and the LRM line numbers sit beside every validator so the reader re-reads the
clause rather than the classifier. `census.md` carries the HEAD reading, the prediction, and the final-probe
re-read.
