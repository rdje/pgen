# External-Corpus Characterization

Per PGEN's external-corpus doctrine, every parser is proven by **two** independent oracles: the internal
stimuli generator (it manufactures inputs from the grammar) **and** an officially-recognized **external**
corpus (authored independently of the grammar, so it exposes the gap between the grammar and the real
language). For `json` the external corpus is **JSONTestSuite** (Nicolas Seriot, *"Parsing JSON is a
Minefield"*, MIT), vendored under `json_corpus_bundle/`.

## How to reproduce

```bash
PGEN_PARSEABILITY_PROBE=./rust/target/debug/parseability_probe \
  json_corpus_bundle/scripts/run_json_corpus.sh
```

The corpus's filename prefix is the fix-independent oracle: `y_*` must be accepted, `n_*` must be
rejected, `i_*` is implementation-defined.

## Measured (2026-06-08, pinned upstream commit `1ef36fa0`, after the `EXTERNAL-CORPUS.2a` upgrade)

| Class | Files | Parser result | Conformance | (was, simplified) |
| --- | --- | --- | --- | --- |
| `y_` MUST accept | 95 | 95 accept / 0 reject | **95 / 95** ✅ | 81 / 95 |
| `n_` MUST reject | 188 | 181 reject / 5 accept / 2 crash | **181 / 188** | 158 / 188 |
| `i_` impl-defined | 35 | 20 accept / 14 reject / 1 crash | n/a | 13 / 21 / 1 |
| crashes (any class) | — | **3** abort (stack overflow) | robustness defect | 3 |

**Every must-accept JSON document is now accepted (95/95)** and the must-reject rate rose 158→181. The
upgrade closed all original lexical gaps (exponents, escapes + control-char rejection, leading-zero
rejection, exact whitespace). Two residuals remain:

- **5 `n_` wrongly accepted** — all *trailing content after a complete value*: `{"a":"b"}//`,
  `{"a":"b"}/**/`, `{"a":"b"}#`, `{"a":"b"}#{}`, `{"a":/*comment*/"b"}`. Not lexical — the value parses and
  the trailing bytes are left unconsumed. Fixed by strict end-of-input enforcement (`EXTERNAL-CORPUS.2c`).
- **3 deep-nesting files abort** instead of being rejected — a robustness defect (`EXTERNAL-CORPUS.2b`).

Full per-file results: `json_corpus_bundle/results/json_corpus_results.tsv`; root-cause analysis:
`json_corpus_bundle/results/characterization.md`.

## Roadmap

- **`EXTERNAL-CORPUS.2c`** — strict end-of-input at the `json` rule so trailing garbage/comments are
  rejected (closes the remaining 5 `n_`).
- **`EXTERNAL-CORPUS.2b`** — a recursion/stack guard so deep nesting is *rejected*, not *crashed*.
