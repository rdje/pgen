---
id: measure-a-policy-where-its-outcome-is
title: A count is not a specification — and a policy's over-approximation is measurable only where its OUTCOME is, not where its decision is
answers:
  - "I measured that a heuristic is wrong most of the time — why can't I fix it from that number"
  - "where should an instrument that judges a policy's decisions live"
  - "why can't I re-derive a predicate in a standalone probe to audit it"
  - "how do I turn a percentage of wrong decisions into an actionable target list"
  - "how do I know a new census instrument is not itself broken"
  - "what should I check before trusting a new instrument's list"
tags: [instrumentation, measurement, root-cause, method]
date: 2026-08-08
status: current
evidence: rust/src/ast_pipeline/stimuli_generator.rs (the `PGEN_WITNESS_BLOCKED_VERDICT_CENSUS` block inside `generate_target_witnesses`, beside `witness_target_is_store_entry_blocked` and `witness_target_is_resolved`); docs/tasks/SV-EXH-PROOF.md leaf .7.4.6.17 (CENSUS_2026-08-08) and .7.4.6.15 (MEASURED_PREMISE_2026-08-08); TOOLBOX.md 6.3
reverify: "PGEN_WITNESS_BLOCKED_VERDICT_CENSUS=1 bash docs/tasks/artifacts/sv_exh_proof/run_closed_loop_replay_stage.sh 2017 rust/target/km_census >rust/target/km_census.log 2>&1; grep -c 'classification=SPURIOUS' rust/target/km_census.log; grep -c 'classification=GENUINE' rust/target/km_census.log; grep -o 'store_entry_raises=[0-9]*' rust/target/km_census.log | tail -1"
---

**"The heuristic is wrong 94 % of the time" is a real, decisive number that you cannot fix.** It
names no subject. The instinct after measuring it is to go straight at the predicate; the move that
actually works is to spend one slice turning the count into a **named list**, because a list does
work a count cannot. PGEN's worked instance: `store_entry_raises` fell `16 → 1` and `11 → 1` once a
witness pass attempted the target's own rule before raising the entry — 94 % spurious, and no target
named. The census that named them showed **every** blocked target was a *branch* target and none was
a *rule* target, exonerating half the predicate before a line of the fix was designed; and one rule
turned out to carry both a spurious and a genuine branch, turning the next step from a sweep into a
two-case controlled comparison.

⭐ **Put the instrument where the OUTCOME is, not where the DECISION is.** The obvious harness is a
standalone probe that re-evaluates the predicate over the population. It is not buggy — it simply
cannot answer the question. *"Was this decision necessary?"* is not a property of the decision at
all; it is a property of what happened afterwards. So the instrument belongs at the one point in the
program where the decision and its outcome coexist, even when that feels like the wrong place for a
diagnostic.

> When an instrument feels awkward to place, check whether the fact you want actually exists where
> you are putting it.

⛔ **Reconcile a new instrument against numbers you already have, BEFORE trusting its list.** Give it
obligations it can fail: PGEN's census had two, from two different mechanisms measured slices apart —
its `GENUINE` count had to equal the pass's own summary counter, and its total had to equal a raise
count measured earlier by unrelated means. Both reconciled exactly on both profiles. *A census that
does not reconcile is reporting a broken instrument, not a finding*, and nothing in the list itself
tells you which one you are holding ([[feedback_instrument_needs_ground_truth]]).

⛔ **A read-only instrument must be PROVEN read-only, not asserted.** Run the subject with the
instrument ENABLED and diff every artifact against a run without it; "it is behind an env check"
is an argument, and the artifacts are the evidence. PGEN's census came out byte-identical on all
eight stage artifacts with it on.

⚠️ **Claim the measurement you made, not the one you planned.** A no-regression note drafted as
"with the flag unset the artifacts are byte-identical" described a run that had never happened —
the run that *had* happened was stronger (flag ENABLED, still byte-identical). Drafting a claim
ahead of its evidence is exactly how an unearned checklist box gets written; re-read every box
against what is actually on disk.

See also [[witness-entry-policy-store-gated-targets]] (the policy this was built to audit) and
[[deterministic-artifacts-sort-at-the-serializer]] (the same "prove it, don't argue it" discipline
applied to an artifact).
