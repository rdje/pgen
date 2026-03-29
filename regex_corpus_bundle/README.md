# regex_corpus_bundle

Repo-ready starter bundle for a **PCRE2-first** regex corpus pipeline.

Included:

- `docs/regex_corpus_plan.md`
- `manifests/upstreams.lock.json`
- `manifests/licenses.json`
- `schemas/regex_case.schema.json`
- `scripts/fetch_regex_corpora.py`
- `scripts/normalize_pcre2_testdata.py`
- `scripts/normalize_pcre2_compile_oracle.py`

Current role in PGEN:

- this bundle is the maintained **PCRE2-first** acquisition/inventory starter for future regex hardening beyond the currently closed `regex` family row
- it does **not** reopen `regex: Done` by itself
- current preflight contract:
  - `make -C rust regex_corpus_bundle_contract_gate`
- current external hardening measurements:
  - `make -C rust regex_pcre2_textsafe_corpus_gate`
  - `make -C rust regex_pcre2_compile_oracle_gate`

## Quick start

```bash
make -C rust regex_corpus_bundle_contract_gate
python3 scripts/fetch_regex_corpora.py --all
make -C rust regex_pcre2_textsafe_corpus_gate
make -C rust regex_pcre2_compile_oracle_gate
```

This fetches the pinned upstream sources, extracts only the relevant subsets, and writes file-level inventories under `corpus/pcre2/`.

Implemented case-level normalizers now include:

- PCRE2 text-safe syntax slice from `testdata/testinput*`
- PCRE2 compile-oracle slice from `testdata/testinput2` + `testdata/testoutput2`

The next normalization step is still:

- PHP `ext/pcre/tests/*.phpt`
