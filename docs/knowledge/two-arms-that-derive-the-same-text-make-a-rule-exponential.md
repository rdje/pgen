---
id: two-arms-that-derive-the-same-text-make-a-rule-exponential
title: When two alternatives of one rule can derive the SAME text, a tournament policy makes that rule exponential in nesting depth — and the declarative cure is `ordered`, which is NOT the policy your grammar probably already uses
answers:
  - "my parser is exponential on nested if/else chains — where do I look"
  - "parse time and memory double for each extra else-if — what causes that"
  - "a 261-line file takes 12 GB to parse but a 239-line file is fine — why"
  - "which branch_policy stops PGEN exploring the losing alternative"
  - "does priority_first avoid running the full tournament"
  - "difference between longest_match, ordered and priority_first in PGEN"
  - "how do I prove a branch_policy change did not alter acceptance"
  - "is it safe to change a choice rule from longest_match to ordered"
  - "my grammar rule has two arms and one reaches the other through a wrapper chain"
  - "super-linear backtracking in a generated PEG parser — declarative fix"
tags: [grammar, performance, branch-policy, backtracking, systemverilog, acceptance, memory]
date: 2026-08-10
status: current
evidence: grammars/systemverilog.ebnf (`conditional_else_branch` + its guard comment); docs/tasks/SV-CORPUS-GRAD.md leaf .11a (`PGEN-SV-CORPUS-GRAD-0194`/`-0195`/`-0197`); rust/src/ast_pipeline/annotation_validator.rs:669 (the three legal values); rust/src/ast_pipeline/grammar_wellformedness.rs:1255 (longest_match AND priority_first both run the full tournament); rust/src/ast_pipeline/ast_based_generator.rs:4715-4720 (the keep-first-winner short circuit belongs to `ordered` alone)
reverify: "grep -B1 '^conditional_else_branch :=' grammars/systemverilog.ebnf | grep -q '@branch_policy: ordered' && echo ORDERED-STILL-PINNED"
---

**The shape to recognise: a rule whose arms OVERLAP in the language they derive, where one arm
reaches the other through a wrapper chain.** In `systemverilog.ebnf`:

```ebnf
conditional_else_branch := conditional_statement -> {kind: "elseif", body: $1}
                         | statement_or_null     -> {kind: "else",   body: $1}
```

Arm 2 reaches `conditional_statement` again via `statement_or_null → statement → statement_item →
statement_item_sv_2017`. So on an `else if`, **both arms parse the identical text and tie**. Under a
tournament policy the engine explores both, so every level of the chain parses the whole remaining
chain twice: `T(k) = 2·T(k−1)`.

Measured, and the numbers are the tell:

| chain depth n | 12 | 14 | 16 | one real 261-line file |
|---|---|---|---|---|
| before | 114 MB | 348 MB | 1313 MB | **12 362 MB, TIMEOUT** |
| after (`ordered`) | 27 MB | 29 MB | 29 MB | **118 MB, 0.13 s** |

⭐ **Diagnose it by entry count, not by profiler.** `--dump-rule-entry-counts-json` showed the choice
site entered exactly `2ⁿ − 1` times (15/31/63 at n=4/5/6) — an exact power of two is a *structural*
signature, and it names the site directly. After the fix it is exactly `n`. ⚠️ **Input size is a
decoy**: a 239-line file passed while a 261-line file took 12 GB. Nesting DEPTH was the whole
variable, so bisect on depth with a generated reproducer, not on the corpus.

⛔⛔ **`priority_first` DOES NOT FIX THIS, and it is probably what your grammar already uses.** PGEN
has three policies (`annotation_validator.rs:669`): `longest_match` (default), `ordered`,
`priority_first`. **Both `longest_match` and `priority_first` run the FULL tournament** — they differ
only in how a tie is *resolved*, not in whether the losing arm is *explored*. Only `ordered`
short-circuits on the first winner. All 29 existing `@branch_policy` uses in `systemverilog.ebnf` are
`priority_first`, so copying the surrounding idiom produces a change that looks like the fix and
measures as a no-op. **Annotate the rule with why `ordered` is load-bearing**, or the next reader
will normalise it away ([[a-constructs-guard-comment-is-part-of-the-construct]]).

⭐ **Prove neutrality at the choice site, not only corpus-wide — it is the stronger and cheaper
proof.** `ordered` and a tournament differ *exactly* where two arms tie, which here is by
construction, so the obligation is real. Protocol D settles it directly: the engine's own
`🏁 Rule '…' selected branch N/M consuming K chars` line under `ordered` was **byte-identical** to the
line recorded under `longest_match` — same arm, same lengths, at every level. Corpus-wide the
16 336-file run then showed **0 pass→non-pass transitions** with the fail set unchanged
(`6586 → 6586`), and the only movement was the 4 timeouts becoming passes.

⭐ **The linter's shadowing verdict only becomes meaningful once you adopt `ordered`.**
`ordered_choice_shadowing` fires ONLY under `@branch_policy: ordered` with no branch-phase predicates
(`grammar_wellformedness.rs:1259`) — under a backtracking policy a later arm is still reachable, so
the verdict would be unsound. ⇒ after switching a rule to `ordered`, re-run `--lint-grammar` and
treat `ordered_choice_shadowing=0` as a *newly earned* check rather than an unchanged number.

⚠️ **Fixing the cost does not mean the cost was the only defect.** These four files had been
adjudicated `divergence:explained_timeout` — i.e. an unbounded-allocation parser defect was sitting
inside the population a burn-down treats as EXPLAINED, and "timeout" was doing work it had not
earned. When the fix landed, the class emptied (4 → 0) and the rows reclassified on their real
merits. **Audit any "explained by resource limit" bucket for defects wearing a budget mask**
([[a-rising-pass-rate-is-not-evidence-of-correctness]] is the same failure direction).

See also [[a-fused-counter-is-not-evidence-about-any-of-its-parts]] (why the memo looked like it was
already collapsing this rule when it was serving 0 replays) and
[[a-memo-key-must-name-every-context-the-outcome-depends-on]].
