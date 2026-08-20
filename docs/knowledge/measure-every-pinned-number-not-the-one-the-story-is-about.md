---
id: measure-every-pinned-number-not-the-one-the-story-is-about
title: A determinism probe aimed at the number under suspicion should print every number beside it — the one that moves is rarely the one you were investigating
answers:
  - "how do I check whether a baseline number is deterministic"
  - "my probe compared the metric I suspected and found nothing — what now"
  - "how do I tell a structural metric from a sampling one"
  - "is it safe to pin a coverage or debt count against a fixed threshold"
  - "a contract threshold holds at one seed — is that a defect"
  - "should a probe compare fields separately or as one line"
tags: [instruments, determinism, baselines, contracts, probes, evidence]
date: 2026-08-20
status: current
evidence: "SV-CORPUS-GRAD.13c.2x.1 / .13c.2x.3 (`PGEN-SV-CORPUS-GRAD-0252`). A probe built to test whether PGEN's synthesized post-left-recursion rule count is process- or seed-dependent found it byte-identical across 5 processes at one seed AND across 4 seeds — H1 refuted. The same output line carried `unreachable_branches`, which moved 2 / 18 / 20 / 27 at seeds 24001 / 0 / 7 / 42, and `systemverilog_syntax_closure_contract.json` pins it at 2. The suspected number was stable; the unsuspected neighbour was the sampling artifact."
reverify: "bash docs/tasks/artifacts/sv_corpus_grad/rule_count_determinism/probe.sh   # STRUCT identical on both axes; SAMPLE branch debt reported as 4 distinct values"
---

**A determinism probe is cheap to widen and expensive to narrow.** Running the instrument nine
times is the whole cost; printing four extra fields from the output you already have is free. So
print them — because the field that moves is very often not the one that made you suspicious.

A rule count in PGEN had been observed reading 1434 at one seed and 1433 at two others. Because each
seed ran in its own process, seed and process were confounded, and the open question was whether
rule *synthesis* was nondeterministic — which would have meant every rule-count contract in the
repository was pinning noise. A probe de-confounded the axes: five processes at one fixed seed,
then four seeds one process each. The rule counts came back byte-identical on both axes.

The same lines carried a neighbour nobody had asked about:

```text
seed 24001   total=1610 reachable=1607 unreachable=3      unreachable_branches=2
seed 0       total=1610 reachable=1607 unreachable=3      unreachable_branches=18
seed 7       total=1610 reachable=1607 unreachable=3      unreachable_branches=27
seed 42      total=1610 reachable=1607 unreachable=3      unreachable_branches=20
```

A tracked contract pinned that last column at **2**.

⭐ **The distinction the probe then has to make is STRUCTURAL versus SAMPLED.** Reachability over a
rule graph is a function of the grammar: it must not move, and if it does that is a defect. Branch
*debt* counts what one run's generated stimuli failed to exercise: seed-dependence there is correct
and expected. **A probe that prints both on one line and compares the line cannot tell you which
happened** — the first cut of this one announced "seed-dependence holds for this counter" when the
counter in question had not moved at all. Compare the groups separately and report them separately.

⚠️ **And the finding about the contract is real even though the metric is behaving correctly.** A
ceiling on a sampling metric holds only at the seed the contract declares. That is not wrong if the
contract declares the seed — but the field *reads* like a closure claim about the grammar, so it
must say which it is, exactly as a metric with two possible predicates must
([[one-metric-name-two-predicates-is-a-contract-defect]]).

**The rule: when you pin a number, measure its stability. When you probe one number's stability,
measure every number printed beside it.**
