# JSONTestSuite — immutable upstream snapshot (provenance)

This directory is an **immutable vendored snapshot** of the recognized external JSON
conformance corpus, kept separate from any normalized/derived output per the
`regex_corpus_bundle/` precedent and [[project_external_corpus_doctrine]].

| Field | Value |
| --- | --- |
| Project | JSONTestSuite — *"Parsing JSON is a Minefield"* |
| Author | Nicolas Seriot (`nst`) |
| Upstream | https://github.com/nst/JSONTestSuite |
| Article | https://seriot.ch/projects/parsing_json.html |
| Pinned commit | `1ef36fa01286573e846ac449e8683f8833c5b26a` |
| Vendored on | 2026-06-08 (via `git clone --depth 1`) |
| License | MIT (© 2016 Nicolas Seriot) — see `LICENSE` in this directory |

## What is vendored

- `test_parsing/` — the **318** parsing test files (the corpus actually used by PGEN's
  characterization), each prefixed by its recognized verdict class:
  - `y_*` (95) — a conforming parser **MUST accept**.
  - `n_*` (188) — a conforming parser **MUST reject**.
  - `i_*` (35) — **implementation-defined**; either outcome is acceptable.
- `LICENSE` — the upstream MIT license (attribution + permission to redistribute).

## What is NOT vendored

The upstream `parsers/`, `results/`, `article/`, `test_transform/`, and `run_tests.py`
(the multi-language driver harness and its rendered result matrices) are intentionally
omitted — PGEN runs the corpus through its own `json_corpus_bundle/scripts/run_json_corpus.sh`.
To refresh or extend, re-clone upstream at a new pinned commit and update this file.

## Integrity

The filenames are the oracle (the `y_/n_/i_` prefix). The file contents are byte-for-byte
as published at the pinned commit; some `n_*` files deliberately contain non-UTF-8 or
control bytes (that is the point of the corpus) and must not be "fixed".
