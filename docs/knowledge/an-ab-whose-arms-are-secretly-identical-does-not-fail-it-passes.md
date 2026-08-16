---
id: an-ab-whose-arms-are-secretly-identical-does-not-fail-it-passes
title: The hard part of an A/B is not the comparison, it is proving the two arms were two things — because when they are secretly one thing, the test PASSES
answers:
  - "my A/B says the two configurations agree — how do I know it really ran both"
  - "how do I prove a debug flag actually changed the code path it names"
  - "I passed the flag and the output looks right — is that enough"
  - "what makes a good tell for an instrument arm"
  - "my two arms print different things, so they must be different configurations, right"
  - "how do I compare a bare run against an instrumented run without fooling myself"
tags: [instruments, evidence, measurement, controls, observability, gates]
date: 2026-08-16
status: current
evidence: ENGINE-UNIVERSAL-SERVICES.22 acceptance (b), 2026-08-16. PGEN needed to know whether a parse observed by its coverage recorder produces the SAME derivation as an unobserved one. Two natural ways to build that A/B were tried and BOTH produced two identical BARE arms, each of which would have reported a clean agreement. (1) Passing `--dump-rule-outcome-counts-json` beside `--parse-dump-ast` — the global IS applied before dispatch, but `enable_coverage()` lives in a macro that path does not use, so no counts file is written. (2) `PGEN_REPORT_MEMO_STATS=1` as the coverage tell — it routes the parse, and the two stderr blocks visibly differ, but `sort`-ing them shows the aggregate header BYTE-IDENTICAL (`5710 success entries … 29378 total, 6170 subtree-nodes, 369 distinct rules`); the whole diff is which members of a tie group a top-30 cutoff prints. The shipped tell is the recorder's own read-back, `exercised_rules=134`, which is zero by construction when coverage is off.
reverify: "bash docs/tasks/artifacts/engine_universal_services/derivation_twin/probe.sh 10   # exit 0; the run REFUSES rather than comparing if any row cannot prove its three arms were three configurations"
---

**A control that fails is loud. An A/B whose two arms are secretly the same configuration is
silent — and it reports a PASS.** That asymmetry is the whole problem: the failure mode of a
comparison is not "it errors", it is "it agrees, for a reason you did not intend".

PGEN needed to answer a question about its own observability: does attaching the coverage recorder
to a parse change the *derivation*, or only the cost? Both arms were easy to describe and hard to
build.

```text
attempt 1  --parse-dump-ast … --dump-rule-outcome-counts-json out.json
           the flag IS parsed; the global IS applied before dispatch
           ⛔ but enable_coverage() lives in a macro this command does not use
           ⇒ no counts file written, BOTH arms bare, ASTs identical, "PASS"

attempt 2  PGEN_REPORT_MEMO_STATS=1  as the coverage tell
           it really does route the parse; the two stderr blocks really do differ
           ⛔ sort them: the aggregate header is BYTE-IDENTICAL
           ⇒ the diff is tie-order inside a top-30 cutoff, not coverage
```

Neither attempt errored. Neither produced a suspicious number. Both would have gone into a task
leaf as a green result.

## The rule

> **Every arm must print something that ONLY that arm can produce, and the harness must assert it
> per row before it compares anything.**

Not "the flag was accepted". Not "the output looks different". Something whose *existence* is
downstream of the mechanism you are claiming to have switched on.

In this case the shipped tell is the coverage recorder's own read-back — the number of rules it
recorded as exercised. With coverage off, nothing is ever pushed onto the recorder's stack, so the
number is **zero by construction**. It cannot be produced by a flag that was merely typed, by a
global that was set and ignored, or by a code path that silently fell back.

| candidate tell | what it actually proves |
|---|---|
| the flag was accepted (exit 0) | the argument parser knows the spelling |
| an output file appeared | *something* wrote a file |
| the two stderrs differ | ❌ nothing — they can differ by tie-order, timing, addresses |
| a value that is **0 unless the mechanism ran** | ✅ the mechanism ran |

## The three checks worth building in

1. **Per-arm identity assertion, per row.** The harness above checks that arm A emitted *no stderr
   at all*, arm B emitted a specific header, and arm C emitted `exercised_rules=` with a non-zero
   count. A miss is a **refusal**, not a mismatch — the two are different findings and conflating
   them turns "I could not test this" into "this is broken" or, worse, "this is fine".
2. **Report the untestable rows separately.** Files whose parse is *rejected* have no derivation to
   compare. Folding them into the pass count inflates it with rows that were never tested.
3. **Never spend a sampling bound as a prefix of an ordered population.** The same probe's `[N]`
   argument first took the first N rows of a manifest ordered `hot, lr, breadth` — so any small N
   would have reported "N files, all identical" having touched only the heaviest tier. A
   deterministic stride plus a printed tier census costs three lines.

⛔ Note where (3) happened: in the limiter of the instrument written to catch exactly this class of
defect. Related: [[a-check-whose-inputs-all-pass-has-not-been-tested]] and
[[deriving-a-control-from-the-producer-is-not-the-same-as-observing-it]].

⇒ **when a comparison agrees, the first question is not "what does this mean" but "did I actually
run two different things".**
