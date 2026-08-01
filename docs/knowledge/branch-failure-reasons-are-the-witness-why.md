---
id: branch-failure-reasons-are-the-witness-why
title: Per-branch `failure_reasons` is the WHY behind `selected_but_failed` — the pass-level counters structurally cannot see it
answers:
  - "a residual coverage target says selected_but_failed — where do I find WHY it failed"
  - "the witness pass reports depth_exceeded=0 but targets are still unresolved — what happened"
  - "why do the witness-pass failure counters read zero while branches are dying"
  - "where does the stimuli generator record why an OR branch could not emit"
  - "what is failure_reasons in the coverage gap report"
  - "how do I diagnose a closed-loop residual without re-running the gate"
tags: [stimuli, coverage, diagnostics, gap-report, witness-pass]
date: 2026-08-01
status: current
evidence: rust/src/ast_pipeline/stimuli_generator.rs (record_branch_failure :460 called from every OR-failure path :10461/:10557/:10577; BranchCoverageDebt.top_failure_reasons :618 rendered as `failure_reasons=[…]` :766-790; the witness loop's target-level classifier :5596-5660 and its Ok arm :5582-5595); docs/tasks/SV-EXH-PROOF.md leaf .7.4.6.9 (WHY_WHERE_CLASS_A_DEPTH_2026-08-01 — 79/79 of the SV class-A residual explained from artifacts already on disk)
reverify: "grep -n 'record_branch_failure(' rust/src/ast_pipeline/stimuli_generator.rs | grep -v 'fn record'; python3 -c \"import json;d=json.load(open('rust/target/sv_stimuli_quality_gate/work/profile_2017_replay_gap.json'));x=d['reachable_branch_debt'][0];print(x['branch_id'],x['selected_hits'],x['success_hits']);[print(' ',r['count'],r['reason']) for r in x['top_failure_reasons']]\""
---

Two different failure records exist in the stimuli generator, at two different scopes, and only one
of them is printed in the summary line everyone reads.

## The pass-level counters are TARGET-scoped

`generate_target_witnesses` prints, per pass:

```
Witness pass: resolved 872 -> 2651 of 2693 reachable targets (+1779 via 924 witnesses;
  failures depth_exceeded=0, rule_visit_limit=0, target_timeout=0, helper_timeout=0,
  other=1, no_entry=0; construct_fell_back_to_search=12)
```

Every one of those counters is incremented **only when the whole target's generation returns
`Err`** (:5596-5660). When generation returns `Ok`, the loop records a witness and moves on
(:5582-5595) — it never asks whether the target was actually credited.

⛔ **So a forced branch can fail and every counter stays 0.** The reach plan forces branch *N*
first, but `generate_or`'s forcing arm keeps all the siblings as fallbacks; branch *N* fails, a
sibling succeeds, the rule returns `Ok`, and the target-level classifier sees a success. The
measured signature of this is unmistakable and was misread for three slices: **42 targets
unresolved against exactly 1 recorded failure.**

## The per-branch record is where the WHY actually lives

`record_branch_failure` (:460) fires on *every* OR-branch failure path (:10461, :10557, :10577) and
stores the error string per `(group_key, branch_index)`. It surfaces in two places, both written by
an ordinary gate run:

- the **gap report** — `BranchCoverageDebt.top_failure_reasons` (:618), rendered into the text
  report as `failure_reasons=[reason (count), …]` (:766-790). ⚠️ **truncated to the top 3**
  (:2741) — compare the counts against `selected_hits` to know whether you are seeing all of them;
- the **coverage artifact** — `branch_groups["<rule>::<path>"].failure_reasons`, **untruncated**,
  one map per branch index. This is the better instrument when the top-3 cut matters.

Both are plain JSON on disk after any `--generate-stimuli … --gap-report-json/--coverage-output`
run, so the diagnosis needs **no gate re-run**:

```bash
python3 - <<'EOF'
import json
d = json.load(open('rust/target/sv_stimuli_quality_gate/work/profile_2017_replay_gap.json'))
for x in d['reachable_branch_debt']:
    print(x['branch_id'], x['selected_hits'], x['success_hits'])
    for r in x['top_failure_reasons']:
        print('   %6d  %s' % (r['count'], r['reason']))
EOF
```

## Worked precedent

`SV-EXH-PROOF.7.4.6.9`. The residual read `selected_but_failed` with `depth_exceeded=0`, and the
leaf concluded in writing that *"naive max_depth exhaustion is NOT what is stopping generation."*
The per-branch record said the opposite, in the artifact the same run had already written: **37 of
41 residual branches carried exactly one `Stimuli generation depth exceeded max_depth=40 …`** — one
per branch, the witness pass's single attempt. The whole class was explained from disk, in minutes,
by an instrument that had been emitting the answer for months.

**The portable rule:** an aggregate counter answers *how many*, never *why*. Before concluding that
a failure mode is absent because its counter is zero, find the record written at the scope where the
failure actually happens — and check that the counter you are reading is scoped to that event at
all. See [[coverage-gap-reason-codes-are-generator-verdicts]] for which subsystem the reason code
points at, [[stimuli-generation-error-reasons]] for the error taxonomy, and
[[sv-residual-depth-budget-cause]] for what the SV depth failures turned out to be.
