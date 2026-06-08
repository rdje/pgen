# PGEN `json` parser — external-corpus characterization

- Corpus: **JSONTestSuite** (`test_parsing/`, pinned commit `1ef36fa0`, MIT) — 318 files.
- Parser: PGEN generated `json` parser from `grammars/json.ebnf` (`make -C rust focus_json`).
- Runner: `json_corpus_bundle/scripts/run_json_corpus.sh` (deterministic; the corpus's own
  `y_/n_/i_` prefixes are the fix-independent oracle).
- Measured: 2026-06-08 — **after the `EXTERNAL-CORPUS.2a` RFC-8259 upgrade** (`PGEN-EXTERNAL-CORPUS-0003`).

## Headline result — does `json.ebnf` match the official JSON standard? **Now: yes for the lexical/syntactic core.**

`grammars/json.ebnf` was originally a *simplified subset*; `EXTERNAL-CORPUS.2a` upgraded its terminals to
track RFC 8259 / ECMA-404, using this corpus as the acceptance metric. Result:

| Class | Files | Parser result | Conformance | (was, simplified) |
| --- | --- | --- | --- | --- |
| `y_` (MUST accept) | 95 | **95 accept**, 0 reject | **95 / 95** ✅ | 81 / 95 |
| `n_` (MUST reject) | 188 | **181 reject**, 5 accept, 2 crash | **181 / 188** | 158 / 188 |
| `i_` (impl-defined) | 35 | 20 accept, 14 reject, 1 crash | n/a (either valid) | 13 / 21 / 1 |
| **crashes (any class)** | — | **3** abort (stack overflow) | robustness defect | 3 |

**Every must-accept JSON document is now accepted (95/95)**, and the must-reject rate rose 158→181. The
upgrade closed all of the original lexical divergences: number **exponents** (`1e10`, `0e1`, `2.5E-3`) now
parse; string **escapes** (`\" \\ \/ \b \f \n \r \t \uXXXX`) are modelled and **raw control characters /
bad escapes are rejected**; **leading zeros** (`01`) are rejected; and **whitespace** is the exact JSON set
`[ \t\n\r]` (form-feed no longer slips through).

## Remaining divergences (2 known classes, each its own follow-up leaf)

### 1. Trailing content after a complete value — 5 `n_` wrongly accepted (`EXTERNAL-CORPUS.2c`)
`{"a":"b"}//`, `{"a":"b"}/**/`, `{"a":"b"}#`, `{"a":"b"}#{}`, and `{"a":/*comment*/"b"}` are still
**accepted**. These are NOT a lexical issue — the value parses and the trailing bytes (a comment marker /
`#` / garbage) are left unconsumed without the parse being rejected. The fix is **strict end-of-input
enforcement** at the top rule (reject any unconsumed trailing content). Owned by `EXTERNAL-CORPUS.2c`.

### 2. Deep nesting aborts — 3 files crash (`EXTERNAL-CORPUS.2b`)
`i_structure_500_nested_arrays`, `n_structure_100000_opening_arrays`, `n_structure_open_array_object`
crash the recursive-descent parser (stack overflow) instead of being rejected. A parser must *reject*, not
*crash on*, adversarial input. The regex family solved this class with a dedicated worker stack (RGX-0085);
the json parser needs an equivalent guard / depth bound. Owned by `EXTERNAL-CORPUS.2b`.

## Internal-generator note (honest, not a parser defect)

The EBNF-internal certificate-coverage gate still reports `fully_certified=true` for json (all 9 rules
witnessed, `UNKNOWN=0`), and the **closed-loop** stimuli generator (`--generate-stimuli`) produces
**40/40 valid samples at 100% rule+branch coverage**. However the cert-coverage **raw witness pass**
(`generate_many`, unfiltered) now reports `sample_parse_failures≈31/200` (was 0 with the permissive
grammar): the stricter string/number regexes expose the known **G.4.7/F2 raw-witness-generation fidelity**
limitation — the raw witness sampler does not perfectly satisfy a complex regex terminal (alternation /
negated class / `\uXXXX`). This is a *generator-tooling* residual (not a parser bug and not a closed-loop
generator bug); the parser is more correct, as the external corpus proves.

## History

The pre-upgrade (simplified-grammar) measurement was `y_` 81/95, `n_` 158/188 — superseded by this run.
The simplified grammar's gaps (no exponent, no escapes, leading zeros, loose whitespace) are recorded in
git history and were the motivation for `EXTERNAL-CORPUS.2a`.
