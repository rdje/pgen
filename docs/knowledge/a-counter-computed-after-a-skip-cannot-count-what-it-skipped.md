---
id: a-counter-computed-after-a-skip-cannot-count-what-it-skipped
title: A counter incremented after a `continue` cannot count what the `continue` skipped — the gap report's reachable-branch count left out every covered branch (`SV-CORPUS-GRAD.13c.2v.15`)
answers:
  - "why did the syntax-closure gate's unreachable_branches_from_entry move with sampling luck"
  - "why was covered_reachable_branches always zero"
  - "how can a covered branch be counted unreachable"
  - "how do I read a summary counter that disagrees with another counter of the same report"
tags: [instrument, gap-report, syntax-closure, stimuli-generator, counters]
date: 2026-10-18
provenance: SV-CORPUS-GRAD.13c.2v.15 (PGEN-SV-CORPUS-GRAD-0464); rust/src/ast_pipeline/stimuli_generator.rs generate_gap_report
reverify: "cd rust && cargo test --features 'generated_parsers ebnf_dual_run' --lib gap_report_counts_a_covered_branch_as_reachable   # ok; on the pre-fix loop it fails 'a covered branch is reachable'"
---

# A counter computed after a skip cannot count what it skipped

**When a loop asks two questions of each element — "is it a debt?" and "how many are reachable?" — and skips the
non-debts with `continue` before the second question is answered, the second count silently becomes "reachable AND
still owed".** The report then contradicts itself, and the contradiction is the signature.

## The instance

`generate_gap_report`'s branch loop ran `if deficit == 0 { continue; }` before `reachable_branches += 1`. A branch
whose sampled coverage had met the threshold — a COVERED branch — never reached the counter, so it was neither
reachable nor, by `unreachable = total − reachable`, anything but unreachable. On the `systemverilog_preprocessor`
`1.0.9` grammar the syntax-closure gate read `reachable_branches: 97`, `unreachable_branches: 54`,
`unreachable_branches_from_entry: 25 > 24` — and the gap report named 37 of the 54 (8 profile-gated, 29 inside
lookaheads). The other 17 were exactly the report's `covered_branches: 17`, beside `covered_reachable_branches: 0`: a
covered branch that is not reachable cannot exist. On the `1.0.8` grammar the gate's single sample covered no
branch, so the defect was invisible there; it surfaced the first time the sample hit something.

## The rule

- **Compute a count over all elements before any per-element skip**, or compute it in a separate pass. The rule half
  of the same report did, and was right.
- **A ceiling compared with an inflated count is not a ceiling.** `max_unreachable_branches: 24` had been absorbing
  covered branches; re-measured on the fixed counter it is 8, each a branch gated out of the default profile, named in
  the contract — re-pinning to 25 would have pinned the defect.
- **Read a failing pin as a measurement before re-pinning it**: list what the number counts, by name, and account for
  every unit. Here 25 branches were claimed unreachable from the entry and the report named 8 of them; the 17 it named
  nowhere were the covered ones — the defect.

Related: [[a-fused-counter-is-not-evidence-about-any-of-its-parts]],
[[two-counters-bound-each-other-only-if-they-count-the-same-population]].
