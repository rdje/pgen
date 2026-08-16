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
evidence: ENGINE-UNIVERSAL-SERVICES.22 acceptance (b), 2026-08-16. PGEN needed to know whether a parse observed by its coverage recorder produces the SAME derivation as an unobserved one. Two natural ways to build that A/B were tried and BOTH would have reported a clean agreement — for two DIFFERENT reasons, which is the sharper lesson.  (1) Passing `--dump-rule-outcome-counts-json` beside `--parse-dump-ast` — the global IS applied before dispatch, but `enable_coverage()` lives in a macro that path does not use, so no counts file is written. (2) `PGEN_REPORT_MEMO_STATS=1` as the coverage tell — this one DOES produce a genuinely second arm (measured: 0 B of stderr bare vs 4 031 B with a MEMO STATS block, so the parse really moved off the fused graph), and that is exactly why it is dangerous: the arm is real, the TELL is not. Falsified by `sort`-ing both files, which shows the aggregate header BYTE-IDENTICAL (`5710 success entries … 29378 total, 6170 subtree-nodes, 369 distinct rules`); the whole diff is which members of a tie group a top-30 cutoff prints. The shipped tell is the recorder's own read-back, `exercised_rules=134`, which is zero by construction when coverage is off.
reverify: "bash docs/tasks/artifacts/engine_universal_services/derivation_twin/probe.sh 10   # exit 0; the run REFUSES rather than comparing if any row cannot prove its three arms were three configurations"
---

**A control that fails is loud. An A/B that did not really run two things is silent — and it
reports a PASS.** That asymmetry is the whole problem: the failure mode of a comparison is not
"it errors", it is "it agrees, for a reason you did not intend".

⚠️ **And there are TWO ways to not-really-run-two-things, which is easy to conflate and worth
separating** — this card's own first draft conflated them, and a director challenge caught it:

| failure | what is wrong | why it is hard to see |
|---|---|---|
| **identical arms** | the second configuration never engaged at all | nothing distinguishes the runs, so of course they agree |
| **a real arm with a false TELL** | the arms genuinely differ, but not along the axis you are claiming | the arms DO differ visibly, which reads as confirmation |

The second is the more dangerous one, because the evidence *looks* stronger.

PGEN needed to answer a question about its own observability: does attaching the coverage recorder
to a parse change the *derivation*, or only the cost? Both arms were easy to describe and hard to
build.

```text
attempt 1  --parse-dump-ast … --dump-rule-outcome-counts-json out.json
           the flag IS parsed; the global IS applied before dispatch
           ⛔ but enable_coverage() lives in a macro this command does not use
           ⇒ no counts file written, BOTH arms bare, ASTs identical, "PASS"

attempt 2  PGEN_REPORT_MEMO_STATS=1  as the COVERAGE tell            <- a REAL arm,
           measured: 0 B stderr bare vs 4 031 B here, so the parse         a FALSE tell
           genuinely moved off the fused graph — the arm is real
           ⛔ but sort the two files: the aggregate header is BYTE-IDENTICAL
           ⇒ the visible diff is tie-order inside a top-30 cutoff. It proves
             ROUTING, and says nothing whatever about COVERAGE.
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
