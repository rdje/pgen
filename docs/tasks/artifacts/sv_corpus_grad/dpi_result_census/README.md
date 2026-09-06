# The DPI import RESULT-type census (`SV-CORPUS-GRAD.3.43`, `DPI-RESULT-CLASSIFY:`)

`classify.py` reads the TSV of `../ast_field_census/ast_field_census.py` (kind `import_function`, field
`proto.body.body.return_type`) and buckets every DPI import function result the corpus writes against IEEE 1800-2017
§35.5.5 (`docs/systemverilog/2017/md/section-35-direct-programming-interface.md:485-493`; 1800-2023 `:493-501` reads
the same): **legal_keyword** (the eleven listed types, signing admitted), **named** (a type identifier or `type(...)`
without packed dimensions — semantic, admitted), **ILLEGAL** (what the letter excludes and a parser can see), and
**unclassified** (a rendering the classifier does not know — a census with any is not finished).

```bash
python3 docs/tasks/artifacts/sv_corpus_grad/dpi_result_census/classify.py rust/target/sNNN/dpi_result_census_sv_2017.tsv [more.tsv]
# DPI-RESULT-CLASSIFY: <tsv> rows=1298 parser=297 regex=1000 legal_keyword=1210 named=64 ILLEGAL=24 unclassified=0
```

The measured census at release `1.0.215` (the grammar this slice changed) is `census.md`; every ILLEGAL row in an
ACCEPTING file is a pre-registered `pass → fail` corpus row of release `1.0.216`.
