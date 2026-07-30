# `tests/bootstrap_tests/` — 15 annotation-grammar fixtures, currently with no runner

These 15 `.ebnf` fixtures encode expected **accept / reject** behaviour for the two
bootstrap annotation surfaces, split by outcome:

| directory | fixtures | what the fixture asserts |
|---|---|---|
| `return_annotation/success/` | 5 | the bootstrap return-annotation parser handles the shape |
| `return_annotation/failure/` | 4 | the shape exceeds the bootstrap subset and must fall back |
| `semantic_annotation/success/` | 3 | the bootstrap semantic-annotation parser handles the shape |
| `semantic_annotation/failure/` | 3 | the shape must fall back to a raw string |

## Why there is no runner right now

Two runner scripts (`run_bootstrap_tests.sh`, `run_simple_tests.sh`) drove these fixtures
through the **Perl** EBNF frontend, which `LANG-CAPABILITY-AUDIT.10.6` retired and `.10.7`
deleted. They were **retired rather than re-pointed at the Rust frontend**, because
measurement convicted them three ways — re-pointing would have preserved an instrument that
lies:

1. ⛔ **It reported success while its own output said every test failed.**
   `run_bootstrap_tests.sh` printed `Total tests: 4 / Passed: 0`, three `❌` lines, then
   `🎉 ALL TESTS BEHAVED AS EXPECTED!` and **exit 0**. Its verdict is `exit $UNEXPECTED_RESULTS`
   (`:194`), and the `❌ UNCLEAR` and `❌ FAILED` paths increment counters that the verdict
   never reads.
2. ⛔ **It only ever reached 4 of these 15 fixtures.** `set -e` plus a `return 1` from the
   per-test function aborts each category at its first failing case — measured 1 of 5, 1 of 4,
   1 of 3, 1 of 3.
3. ⛔ **It wrote generated artifacts into these tracked fixture directories** (`.json`,
   `_parser.rs`, `.log` next to each `.ebnf`), against the repository's artifact-locality
   policy. Its sibling `run_simple_tests.sh` had been fixed to use `rust/target/`; this one
   never was.

Nothing tracked invoked either script — no gate, no Makefile target, no workflow — which is
the only reason a harness in that state never misled anyone.

## What to do with them

The fixtures themselves are grammar files and are unaffected by any of the above; the
behaviour they describe is still real and still worth gating. Deciding between **writing a
runner that actually gates them** and **deleting the corpus** is tracked as its own work item
rather than settled by side effect — see `docs/tasks/LANG-CAPABILITY-AUDIT.md` leaf `.10.8`.

The Rust frontend is a drop-in replacement for the retired step 1, measured on these very
fixtures:

```bash
rust/target/debug/ast_pipeline <fixture>.ebnf --emit-raw-ast-json <out>.json
rust/target/debug/ast_pipeline --generate-parser --debug <out>.json -o <out>_parser.rs
```

⭐ A runner built on that must fix the three defects above — above all, **its exit code must
be a function of every failure path it prints**, not of one counter out of three.
