# PARSER-NEUTRALITY.1 — hook-mechanism removal verification

Session #175, 2026-07-20. Director ruling executed same-session
(`docs/decisions/feedback_no_parser_hooks_full_neutrality.md`): the hook
mechanism and everything built on it removed; scope per
`docs/tasks/PARSER-NEUTRALITY.md`.

## What was removed (code)

| surface | disposition |
|---|---|
| `rust/src/ast_pipeline/parser_hooks.rs` (`ParserHooks` trait + `ParserHookRegistry` + `ParserImplContext`) | deleted |
| `rust/src/parser_hooks/` (`mod.rs`, `regex.rs` — the only handler) | deleted |
| `--enable-parser-hooks` CLI flag + binary-boundary registration (`main.rs`) | deleted |
| generator threading (`parser_hook_registry` + `ebnf_grammar_name` fields, the `extend_parser_impl` extension point, `#extension_impl` splice, `generate_parser_ast_based_with_hooks`) | deleted; single hook-free `generate_parser_ast_based` remains |
| `regex_typed_differential_gate` + `regex_typed_perf_probe` binaries, their Cargo `[[bin]]`/feature entries, and both Makefile targets | deleted (Makefile tombstone note points at the ruling) |
| `src/parser_hooks/*.rs` entry in the Makefile source wildcard | deleted |

`--emit-typed-entry-skeleton` (pipeline-internal, grammar-agnostic, off by
default, unused by any maintained target) is NOT part of the hook mechanism
and stays; queued as the `.3` dead-scaffolding follow-up question rather
than silently widening this slice.

## Emission-identity proof

- Rebuilt hook-free `ast_pipeline`; regenerated `generated/regex_parser.rs`
  → **`1a5f7018…`**, byte-identical to the artifact the hook-CAPABLE tool
  emitted WITHOUT the flag earlier this session (custody bank
  `artifacts_pre_neutrality.sha256` vs the `-0200` train's first pass) —
  the removal changed nothing but the extension point itself.
- Control regens from the hook-free tool: `json_parser.rs` and `ebnf.rs`
  byte-identical modulo the artifacts' embedded own-output-path strings.
- The regex artifact now carries **0** `parse_*_typed` occurrences (was
  769 in the hooks-form `b535be2c…`); the canonical on-disk form is the
  DEFAULT emit for **all 11 parsers** — the per-parser build-configuration
  fork is gone, and with it the typed-gate silent-restore trap class and
  the `-0200` regen-train step-5b interim rule (both now historical).

## Compiler-neutrality proof

Per-file warning-count comparison (stash-based before/after, line-shift
tolerant): IDENTICAL for every file; the only difference is the deleted
`src/parser_hooks/regex.rs`'s own single warning disappearing. Zero new
warnings or errors introduced.

## Performance-neutrality proof (custody band, NOT a floor re-baseline)

The release probe rebuilt against the default-emit artifact is NOT
byte-identical to the preserved `92fba055` floor probe (`948cbd63…`) — the
lib itself changed (deleted modules shift layout), so the `#140`
byte-identity precedent does not transfer and the proof is empirical:

- bench floor-validation, 3 × 2000-sample rounds, geomean-of-best-mins:
  **1840.4 ns vs banked 1818.3 = +1.21%** (inside the ±6% custody band and
  the ≈2.3% observed noise span);
- full canonical corpus sweep (2,189 cells): geomean **1172.1 ns vs the
  `-0200` candidate's 1153.5 = +1.61%** (inside the noise span), verdict
  flips **0/2,189** against the banked `-0200` candidate per-cell JSONL,
  MAX **474,792 ≤ settled 483,583 ns** (same worst cell `line_725`).

Both drift readings share sign and magnitude consistent with same-session
thermal/session drift; both are inside noise. The ACCEPTED campaign floor
(corpus geomean **1,153.5 ns**, bench ≈1,818.3 ns) is UNCHANGED — this
evidence is a neutrality custody check, not a measurement leaf.

Evidence files (scratch-born, banked here): `neutrality_floorval_r{1,2,3}.txt`,
`neutrality_corpus_stdout.txt`, `neutrality_corpus.jsonl` (per-cell rows).

## Battery

See `battery_summary.txt` (recorded post-run, same commit).
