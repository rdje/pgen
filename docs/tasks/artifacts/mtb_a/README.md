# MTB-A derivation-tape emitter — preserved reusable infra (RGX-0078.5.i.7)

Recovered by slice `PGEN-RGX-0078-0100` (session #132, 2026-07-16) from the
session-#129 scratchpad (`…/3cacb54e-…/scratchpad/mtb_a/`) — which turned out to
be the ONLY surviving copy: contrary to the `-0094`/session-#131 record, the git
reflog holds NO trace of the emitter (the `-0094` revert was a working-tree
restore, never a commit/stash; verified: zero reset/checkout/stash reflog
entries on 2026-07-15 and no dangling commit/blob contains `DerivEvent`).

## What this is

The FULL match-then-build (MTB-A, `-0093` design / `-0094` emission) derivation-tape
infrastructure — REFUTED **as a standalone** increment (−0.9%, land-iff-faster failed)
but the designated FOUNDATION for **MTB-B** (cyclic-spine fold + segment thin memo,
⛔ the session-#49 memo bound): A+B land JOINTLY, since the alloc BYTES live in the
cyclic spine that B folds (95.6% of doomed bytes — `regex_alloc_census_probe`, `-0092`).

## Files

| File | What |
|---|---|
| `mtb_a_emitter.diff` | the complete 2230-line emitter diff (engine `DerivEvent` POD enum + `cascade.rs` `cascade_match_*`/`cascade_build_*`/orchestrator families + A/B partition-drift tripwires + plan-gated tape struct fields; 9/9 cascade unit tests incl. 3 MTB tests). sha256 `34f5db0c567c39b535c66db63c0596f440169ec3a8e58b7d83551c9d27eefce6` |
| `alloc_census_mtb.txt` | the alloc census ON the MTB-A artifact (in_alloc 9518 vs D2-B 9706 = −1.9% — the ROOT-CAUSE evidence for the refutation: MTB-A folds 84.1% of discarded ENTRIES but ~2% of ALLOCATIONS) |
| `round{1..5}_{base,cand}.txt` | the `-0094` fat-LTO alternated 5×2000 bench rounds (base 14002.6ns → cand 13882.0ns = −0.9%, straddling) |
| `analyze_speed.py`, `battery.sh`, `build_and_sweep.sh` | the session-#129 bench/battery scaffolding (reusable for the MTB-B measurement) |

NOT preserved (regenerable, and `generated/` is doctrinally untracked): the MTB
generated-parser artifacts (`regex_parser_mtba_c793e5b1.rs`, 41 MB, + the 10-grammar
`mtb_artifacts_stash/`) — reproduce by applying the diff and running the canonical
`make -C rust focus_*` regen train.

## Applying the diff to a post-`-0099` tree (KNOWN conflict, one hunk)

`git apply --check` fails on exactly ONE hunk: `rust/src/ast_pipeline/ast_based_generator.rs`
@ ~:1733 (the `bare_parse` observability-twin clause) — the P-env slice (`-0099`)
rewrote the context line the diff expects:

- diff expects: `&& std::env::var("PGEN_REPORT_MEMO_STATS").is_err();`
- current tree: `&& !crate::ast_pipeline::report_memo_stats_enabled();`

Resolution: apply that hunk manually, keeping the CURRENT (P-env) spelling of the
memo-stats term and adding the hunk's MTB additions around it verbatim. The
`cascade.rs` hunks and the `mod.rs` hunk (offset ~19 lines) apply cleanly.

## The MTB-B build order (from the `-0093`/`-0094`/`-0097`/`-0098` record)

1. Re-apply this infra (match fns = fused control flow minus value construction,
   appending OrWinner/QuantCount/OptPresent/Boundary events; build fns construct
   values ONCE over the committed tape; sub-roots = mark→match→build→truncate
   orchestrators at an unchanged signature).
2. Extend the fold across the CYCLIC SPINE (the B increment): segment thin memo
   `(end, event-segment, boundary-segment)` under the UNCHANGED taint/epoch
   machinery — ⛔ #49: cycle-participating rules never lose memo protection.
3. ⚠️ The `ThinMemoEntry` payload change is the `-0090` bootstrap-drift class:
   additive transient migration only, NEVER a bootstrap-binary regen mid-change.
4. Measure A+B JOINTLY on the mimalloc fat-LTO basis (alternated 5×2000
   geomean-of-mins), land-iff-faster.
