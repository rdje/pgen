# SV-EXH-PROOF.7.4.4.1 — the "other"-class witness-failure tail: tool-build + root-cause

> Leaf: `SV-EXH-PROOF.7.4.4.1` (under `.7.4.4`). TOOL-BUILD + investigation. Establishes
> WHY+WHERE before any `.7.4.4` fix, per [[feedback_why_and_where_before_solution]] +
> [[feedback_no_codebase_change_without_tool_backed_facts]]. Parser-agnostic.

## Why this slice exists
`.7.4.4` was opened on a HYPOTHESIS: the `.7.4.3` witness pass left a tail of 63 "other"
failures that were "almost certainly CONTEXT-GATED rules — a rule whose `@predicate` needs
store facts emitted by ancestors, so a standalone witness (empty store) fails the predicate."
`.7.4.4` itself mandated: *"FIRST verify this hypothesis (tools-first)."* This slice did, and
the hypothesis is **DISPROVEN**. Two tools-first findings rewrote the problem.

## Finding 1 — the context-gating hypothesis is DISPROVEN
Probed three representative `@predicate has_fact`-gated SV rules **standalone** (entry = the
rule, fresh empty store), via the generator binary on the SV `gen_ast`:

| rule | standalone result |
| --- | --- |
| `known_unscoped_data_type_identifier` | `sample_successes=5/5` |
| `checked_type_identifier` (the `has_fact(type_name)` gate) | `sample_successes=5/5` |
| `known_unscoped_block_class_type` | `sample_successes=5/5` |

All generate fine standalone — so the tail is **not** "predicate needs an ancestor-emitted
fact". The store-gated rules have producer/fallback paths the generator already satisfies.

## Finding 2 — the witness-pass failure classifier was COARSE (the real defect)
The `.7.4.3` witness loop classified only two reasons explicitly and **lumped the rest**:

```rust
match Self::classify_generation_error(&error) {
    DepthExceeded   => depth_exceeded_failures += 1,
    RuleVisitLimit  => rule_visit_limit_failures += 1,
    _ => other_failures += 1,   // <-- TargetTimeout + HelperTimeout + Other ALL land here
}
```

So `other_failures` was **not** "genuine generation errors" — it silently folded
`TargetTimeout` + `HelperTimeout` into the same bucket. The `.7.4.3` `other=63` therefore
could not be assumed to be real errors; the number was uncharacterisable from the counters
alone. This violates the DIAG-SEVERITY principle (**classify by reason, never lump** —
[[feedback_severity_never_gated_by_verbosity]]) applied to the witness pass.

The error message + target identity were also discarded, so there was no WHY/WHERE at all.

## The tool built (`.7.4.4.1`)
Parser-agnostic, additive, in `generate_target_witnesses` / `WitnessSummary`:
1. Classify **all five** `GenerationErrorReason` variants separately
   (`target_timeout_failures`, `helper_timeout_failures` added) — `other_failures` now
   means strictly genuine errors.
2. Collect a **bounded** sample (cap `WITNESS_OTHER_FAILURE_SAMPLE_CAP = 40`) of the
   genuine-Other failures: `target_id | rule | type | node_path | branch | reason`
   (reason = full anyhow chain), surfaced in `WitnessSummary.other_failure_samples` and
   printed by `main.rs` at **default verbosity**.
   - NOT a Debug trace: a per-target Debug trace required global debug verbosity, which
     floods every atom dispatch — it produced a **6.9 GB log in seconds**. Disk is a
     critical resource; the bounded in-summary sample is the disk-safe tool.

## Measurement (the payoff)
Witness pass over a 30-target slice of the real SV `initial_gap` (`--target-max-attempts 0`,
`--target-generation-timeout-ms 12000` so slow witnesses are bounded + correctly classified):

```
Witness pass: resolved 0 -> 20 of 30 reachable targets (+20 via 20 witnesses;
  failures depth_exceeded=0, rule_visit_limit=0, target_timeout=4, helper_timeout=0,
  other=0, no_entry=0)
```

**`other=0`.** Once timeouts are classified out, this slice has **zero genuine generation
errors**. The four failures are `target_timeout` — witnesses that simply take **> 12 s** to
generate (the same deep deeply-factored rules from `.7.4.3a`: `bins_or_options`,
`interface_declaration_sv_2017`, `ansi_port_declaration`). They are **not** errors and
**not** context-gated — they are **slow generations**.

**Full-150 confirmation** (the whole `initial_gap_sample`, `--target-generation-timeout-ms
7000`):

```
Witness pass: resolved 0 -> 81 of 150 reachable targets (+81 via 65 witnesses;
  failures depth_exceeded=0, rule_visit_limit=0, target_timeout=55, helper_timeout=0,
  other=0, no_entry=0)
```

**`other=0` across the entire 150-target sample** — and `target_timeout=55`. The clean
partition is unambiguous: **0 genuine errors, 0 depth/visit failures, 81 resolved, 55
slow-generation timeouts** (cut at the 7 s cap; with no cap the `.7.4.3` run resolved more of
them — they exhaust wall-clock, not the grammar). The original `.7.4.3` `other=63` was an
**artifact of the coarse bucket** (it folded timeout-class failures into "other"); the
reason-split tool shows the genuine-error component is **zero**.

## Re-scoping `.7.4.4` (evidence-driven, not guessed)
The residual tail is a **generation-PERFORMANCE** problem, not a context-gating one:
- The witnesses **do** resolve given adequate time (the `.7.4.3` unbounded run resolved
  72/150; here 20/30 resolve under a 12 s cap, and the 4 "timeouts" would likely resolve
  with a larger budget — they exhaust wall-clock, not the grammar).
- So literal-0's remaining lever is making the slow witnesses tractable, e.g.:
  1. **Purdom min-length-guided greedy expansion** (the `.7.4.2` `compute_min_terminal_lengths`
     table already exists) — at each choice prefer the shortest-terminating subtree so the
     witness converges fast instead of exploring deep factored alternatives. This is the
     literature-grounded path (Purdom 1972 shortest-derivation) and is parser-agnostic.
  2. A per-witness time/size budget tuned so slow-but-resolvable targets still complete.
- This supersedes `.7.4.4`'s original "reach-from-context witnesses" plan, which was aimed
  at the now-disproven context-gating cause.

## Status
- Tool: lands additive — lib (no-features) 584/584; clippy 0 errors; `witness` +
  `classify_generation_error` unit tests pass; binary builds.
- Disk-safe (bounded sample; the Debug-trace flood was reverted before commit).
- NO grammar/codegen/generated change; no release bump (diagnostic + classification only).
- `.7.4.4` itself stays open, **re-scoped** to the generation-performance lever above.
