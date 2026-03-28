# regex_corpus_bundle

Repo-ready starter bundle for a **PCRE2-first** regex corpus pipeline.

Included:

- `docs/regex_corpus_plan.md`
- `manifests/upstreams.lock.json`
- `manifests/licenses.json`
- `schemas/regex_case.schema.json`
- `scripts/fetch_regex_corpora.py`

Current role in PGEN:

- this bundle is the maintained **PCRE2-first** acquisition/inventory starter for future regex hardening beyond the currently closed `regex` family row
- it does **not** reopen `regex: Done` by itself
- current preflight contract:
  - `make -C rust regex_corpus_bundle_contract_gate`

## Quick start

```bash
make -C rust regex_corpus_bundle_contract_gate
python3 scripts/fetch_regex_corpora.py --all
```

This fetches the pinned upstream sources, extracts only the relevant subsets, and writes file-level inventories under `corpus/pcre2/`.

The next step is to implement case-level normalizers for:

- PCRE2 `testdata/testinput*`
- PHP `ext/pcre/tests/*.phpt`
