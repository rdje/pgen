# json_corpus_bundle

The **external** test-corpus surface for PGEN's `json` parser — the outside-derived half of the
[external-corpus doctrine](../docs/decisions/project_external_corpus_doctrine.md): *every PGEN parser is
exercised by BOTH the internal stimuli generator AND an officially-recognized external corpus.* This
bundle mirrors the `regex_corpus_bundle/` (PCRE2) precedent for JSON.

Owned by the `EXTERNAL-CORPUS` task tree (`docs/tasks/EXTERNAL-CORPUS.md`), leaf `.2`.

## Layout

```
json_corpus_bundle/
├── README.md                                    (this file)
├── scripts/run_json_corpus.sh                   (the reproducible runner)
├── results/
│   ├── characterization.md                      (the dated, root-caused measurement)
│   └── json_corpus_results.tsv                  (per-file: name, class, outcome, exit code)
└── third_party/upstream/JSONTestSuite/          (immutable vendored snapshot)
    ├── PROVENANCE.md                            (upstream URL, pinned commit, license)
    ├── LICENSE                                  (MIT, © 2016 Nicolas Seriot)
    └── test_parsing/                            (318 files: 95 y_ / 188 n_ / 35 i_)
```

## The corpus

[**JSONTestSuite**](https://github.com/nst/JSONTestSuite) — Nicolas Seriot's *"Parsing JSON is a
Minefield"* (MIT). The de-facto recognized JSON conformance corpus. Each `test_parsing/` file's prefix is
the fix-independent **oracle**:

- `y_*` — a conforming parser **MUST accept**
- `n_*` — a conforming parser **MUST reject**
- `i_*` — **implementation-defined** (either outcome is acceptable)

## Run it

```bash
# 1) build the json parser + the probe (heavy parsers skipped)
make -C rust SHELL=/bin/bash focus_json
cd rust && PGEN_SYSTEMVERILOG_PARSER_PATH=/nonexistent PGEN_VHDL_PARSER_PATH=/nonexistent \
  PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_PATH=/nonexistent PGEN_RTL_CONST_EXPR_PARSER_PATH=/nonexistent \
  PGEN_RTL_FRONTEND_PARSER_PATH=/nonexistent PGEN_REGEX_PARSER_PATH=/nonexistent \
  cargo build --features generated_parsers,ebnf_dual_run --bin parseability_probe && cd ..

# 2) characterize
PGEN_PARSEABILITY_PROBE=./rust/target/debug/parseability_probe \
  json_corpus_bundle/scripts/run_json_corpus.sh --tsv json_corpus_bundle/results/json_corpus_results.tsv
```

## Current characterization (2026-06-08, after the `EXTERNAL-CORPUS.2a` RFC-8259 upgrade)

`grammars/json.ebnf` was upgraded from a simplified subset to track RFC 8259 / ECMA-404, using this corpus
as the acceptance metric. Measured:

| Class | Files | Result | Conformance | (was, simplified) |
| --- | --- | --- | --- | --- |
| `y_` MUST accept | 95 | 95 accept / 0 reject | **95 / 95** ✅ | 81 / 95 |
| `n_` MUST reject | 188 | 181 reject / 5 accept / 2 crash | **181 / 188** | 158 / 188 |
| `i_` impl-defined | 35 | 20 accept / 14 reject / 1 crash | n/a | 13 / 21 / 1 |

**Every must-accept JSON document is now accepted (95/95)**; the upgrade closed all lexical gaps (number
exponents, string escapes + control-char rejection, leading-zero rejection, exact whitespace). Two known
residuals remain, each its own follow-up leaf: **5 `n_` trailing-content/comment files** still accepted
(strict end-of-input — `EXTERNAL-CORPUS.2c`) and **3 deep-nesting files crash** (recursion/stack guard —
`EXTERNAL-CORPUS.2b`). Full root-cause analysis: [`results/characterization.md`](results/characterization.md).
