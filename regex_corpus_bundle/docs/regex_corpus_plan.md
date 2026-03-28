# PCRE2 Corpus Acquisition and Normalization Plan

## Purpose

This repository layout and fetch plan are for a **PCRE2-flavor** regex parser and test harness.
The acquisition pipeline is intentionally **PCRE2-first**:

1. **Canonical syntax and behavior source:** PCRE2 upstream `testdata/testinput*` and related files.
2. **Secondary PCRE2-relevant source:** PHP `ext/pcre/tests`, because PHP uses PCRE2 but wraps patterns in PHP-specific delimiters and modifiers.
3. **Non-goal for phase 1:** treating non-PCRE2 engines as syntax ground truth.

Current PGEN role:

- this bundle is the maintained acquisition/inventory lane for future regex hardening work
- it sits *beside* the already closed checked-in regex family proof stack rather than replacing it
- the current maintenance preflight is:

```bash
make -C rust regex_corpus_bundle_contract_gate
```

The main design rule is:

- `third_party/upstream/` stores **immutable upstream snapshots**.
- `corpus/` stores **normalized internal data**.
- `oracle/` stores **results produced by your own runner or the reference engine**.

This keeps provenance explicit and lets an AI coding agent re-fetch safely without overwriting hand-authored fixtures.

## Why this source selection

### Tier 1: PCRE2 upstream

PCRE2 upstream is the canonical base because its README documents:

- `testdata/testinput*` as the test data for main library tests
- `testdata/testoutput*` as the expected test results
- `testdata/grep*` as grep-oriented tests

These files are designed for `pcre2test`, which is the closest thing to a syntax-and-behavior oracle for PCRE2.

### Tier 2: PHP `ext/pcre/tests`

PHP uses PCRE2, but PHP test files contain **PHP wrapper syntax**:

- patterns are enclosed in PHP delimiters
- trailing PHP modifiers may be present
- some failure modes are wrapper-level, not raw PCRE2 parser failures

Therefore PHP tests are useful, but they must be normalized carefully.

### Tier 3: quarantine only

Any corpus from Oniguruma, TextMate grammars, RE2, or random internet regex collections must be treated as **candidate input only**.
They are not PCRE2 syntax truth.

## Repo layout

```text
third_party/
  upstream/
    pcre2/
      pcre2-10.47/
        ...
    php-src/
      php-8.4.19/
        ...

corpus/
  pcre2/
    canonical/
      pcre2_inventory.json
      pcre2_testfiles.jsonl
    php/
      php_inventory.json
      php_phpt_inventory.jsonl
    invalid/
      README.md
    quarantine/
      README.md

oracle/
  pcre2/
    README.md

manifests/
  upstreams.lock.json
  licenses.json

schemas/
  regex_case.schema.json

scripts/
  fetch_regex_corpora.py
```

## Rerunnable fetch workflow

The fetch workflow is intentionally simple so an AI coding agent can re-run it safely:

1. Read `manifests/upstreams.lock.json`.
2. Download each pinned archive.
3. Extract into `third_party/upstream/<source>/<ref>/`.
4. Copy or inventory only the relevant subsets.
5. Produce inventory files under `corpus/pcre2/`.
6. Never mutate upstream files in place.

### Recommended commands

```bash
python3 scripts/fetch_regex_corpora.py --all
python3 scripts/fetch_regex_corpora.py --inventory-only
python3 scripts/fetch_regex_corpora.py --source pcre2
python3 scripts/fetch_regex_corpora.py --source php-src
```

## Source pinning policy

Use **tag-pinned** release archives for the first version of the pipeline.
That is reproducible enough to start, and easier for an AI agent than tracking moving branches.

Initial pins:

- PCRE2: `pcre2-10.47`
- PHP: `php-8.4.19`

Later, you can tighten this by adding SHA-256 checksums and signature verification.

## Normalization model

### A. Upstream snapshots

These remain byte-for-byte faithful to upstream.

Examples:

- `third_party/upstream/pcre2/pcre2-10.47/testdata/testinput2`
- `third_party/upstream/php-src/php-8.4.19/ext/pcre/tests/preg_split_error1.phpt`

### B. File inventories

The first normalization pass should inventory source files before attempting case-level extraction.
This is deliberate because `pcre2test` input is structured, and PHP `.phpt` files embed test code.

The inventory records should answer:

- which files were fetched
- what source tier they belong to
- which downstream normalizer should handle them
- whether a file is expected to contain raw PCRE2 data, PHP wrapper syntax, or mixed content

### C. Case-level normalization

After inventories are stable, implement dedicated normalizers:

- `normalize_pcre2_testdata.py`
- `normalize_php_pcre_tests.py`

Do **not** try to use one parser for both source families.

## Canonical normalized schema

The canonical schema is `schemas/regex_case.schema.json`.
Each normalized record should be stored as one JSON object per line in JSONL.

Minimal example:

```json
{
  "id": "pcre2:testinput2:case_0041",
  "flavor": "pcre2",
  "tier": "canonical",
  "pattern": "(?<=foo)bar",
  "source": {
    "repo": "PCRE2Project/pcre2",
    "ref": "pcre2-10.47",
    "file": "testdata/testinput2",
    "case_ref": "41"
  },
  "wrapper": null,
  "compile": {
    "options": ["UTF", "UCP"],
    "newline": "ANYCRLF",
    "bsr": "UNICODE"
  },
  "expected": {
    "parse": "ok"
  },
  "subjects": [
    {
      "text": "foobar",
      "expect_match": true
    }
  ],
  "tags": ["lookbehind", "assertion"]
}
```

## PHP normalization rules

PHP `preg_*` tests must be normalized with special care.

### Preserve wrapper metadata

Keep the original PHP wrapper information before stripping it:

- delimiter
- trailing modifiers
- call site (`preg_match`, `preg_split`, `preg_replace`, etc.)
- whether the expected failure is wrapper-level or PCRE2-level

### Distinguish two error classes

1. **PHP wrapper error**
   - missing delimiter
   - invalid trailing modifier
   - malformed PHP embedding

2. **Raw PCRE2 compile error**
   - the wrapper is valid, but the inner pattern is rejected by PCRE2

A single PHPT file may contain both.

## Invalid corpus strategy

The invalid corpus should not start from random garbage.
It should start from **mutations of valid canonical patterns**.

Recommended mutation families:

- unmatched `(`, `[`, `{`
- broken named groups and named backreferences
- malformed lookaround syntax
- malformed option-setting groups
- illegal or unknown escapes
- bad leading verbs such as malformed `(*UTF)`-style items

Store these under `corpus/pcre2/invalid/` with provenance pointing back to the valid source case.

## Oracle strategy

Keep oracle outputs separate from normalized inputs.

Examples:

- `oracle/pcre2/compile_results.jsonl`
- `oracle/pcre2/match_results.jsonl`

That allows you to re-run a reference PCRE2 build without changing the original corpus.

## Licensing and provenance

Track licenses in `manifests/licenses.json`.

Important initial notes:

- PCRE2 software and docs are BSD-style licensed.
- PCRE2 states that data in the `testdata` directory is in the public domain.
- PHP source is under the PHP License.

## What the included Python script does now

`scripts/fetch_regex_corpora.py` is a **rerunnable acquisition script**.
It currently:

- reads the lockfile
- downloads pinned upstream archives
- extracts them into `third_party/upstream/`
- writes simple inventory files for fetched PCRE2 and PHP test sources

It does **not** yet perform full case-level extraction from `pcre2test` input or PHPT code.
That is the correct next step, but separating fetch from deep normalization reduces accidental breakage.

## Next implementation steps for your AI coding agent

1. Implement `normalize_pcre2_testdata.py`
   - parse `testdata/testinput*`
   - retain directives and per-case compile context
   - map file blocks to normalized `regex_case` records

2. Implement `normalize_php_pcre_tests.py`
   - parse PHPT sections
   - extract `preg_*` call sites
   - classify wrapper-level versus raw-PCRE2-level errors

3. Implement `run_pcre2_oracle.py`
   - compile normalized cases with a pinned PCRE2 build
   - write compile and match outputs into `oracle/pcre2/`

4. Implement `derive_invalid_cases.py`
   - mutate canonical valid patterns
   - keep parent-case provenance

## Suggested agent prompt

```text
Read docs/regex_corpus_plan.md and manifests/upstreams.lock.json.
Do not change third_party/upstream/ contents except by rerunning scripts/fetch_regex_corpora.py.
Implement case-level normalizers as separate scripts.
Preserve provenance for every normalized case.
Treat PHP delimiter/modifier failures as wrapper-level unless the inner pattern itself is validly isolated and rejected by raw PCRE2.
```

## References used to design this plan

- PCRE2 README: repository layout, testdata/testinput*, testoutput*
- PCRE2 LICENCE.md: testdata public-domain note
- PCRE2 docs: `pcre2`, `pcre2test`, `pcre2syntax`, `pcre2pattern`
- PHP manual: regex delimiters, pattern modifiers
- PHP release and php-src release pages for pinning the initial PHP source snapshot
