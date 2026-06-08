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

## Measured (2026-06-08, pinned upstream commit `1ef36fa0`)

| Class | Files | Parser result | Conformance |
| --- | --- | --- | --- |
| `y_` MUST accept | 95 | 81 accept / 14 reject | **81 / 95** |
| `n_` MUST reject | 188 | 158 reject / 28 accept / 2 crash | **158 / 188** |
| `i_` impl-defined | 35 | 13 accept / 21 reject / 1 crash | n/a |
| crashes (any class) | — | **3** abort (stack overflow) | robustness defect |

So the simplified grammar is **right on the common JSON core** but diverges from the standard exactly where
[Grammar and Scope](grammar-and-scope.md) says it would:

- **14 `y_` wrongly rejected** — all number-exponent forms (`y_number_0e1`, `…_real_exponent`, …) plus two
  escaped-string cases (`y_string_allowed_escapes`, `y_string_backslash_doublequotes`).
- **28 `n_` wrongly accepted** — 3 leading-zero numbers, 20 invalid strings (raw control chars / bad
  escapes / bad `\u` surrogates), and 6 trailing-garbage / comment / form-feed structures.
- **3 deep-nesting files abort** instead of being rejected — a robustness defect, not a conformance one.

These are **grammar-scope limits, not engine bugs**: the PEG engine faithfully implements `json.ebnf`. The
full per-file results live in `json_corpus_bundle/results/json_corpus_results.tsv` and the root-cause
analysis in `json_corpus_bundle/results/characterization.md`.

## Roadmap

- **`EXTERNAL-CORPUS.2a`** — upgrade `json.ebnf` toward RFC 8259 (number exponent, `0|[1-9][0-9]*`,
  escaped-string production, exact whitespace, strict no-trailing). Acceptance metric = re-running this
  characterization (target `y_` 95/95, non-crash `n_` 186/186).
- **`EXTERNAL-CORPUS.2b`** — a recursion/stack guard so deep nesting is *rejected*, not *crashed*.
