---
id: provenance-disambiguates-a-verdict-it-does-not-gate-the-work
title: A freshness check belongs AFTER the measurement, not before it — refusing on "stale" blocks exactly the run that would have resolved it
answers:
  - "where in a gate should the provenance or freshness check run"
  - "my staleness check is blocking commits — how do I keep the guarantee without the friction"
  - "should a gate refuse when its baseline is out of date"
  - "how do I stop a provenance block from becoming busywork"
  - "can a gate re-baseline itself safely"
  - "how do I bound staleness without making it fatal"
  - "an identity check made my pre-commit hook fail on a comment"
tags: [provenance, identity, gates, doctrine, friction, baselines, ratchets]
date: 2026-08-20
status: current
evidence: "SV-CORPUS-GRAD.13c.2x.4 (`PGEN-SV-CORPUS-GRAD-0253`). BASELINE-IDENTITY shipped with the identity check as a precondition. Measured by appending one comment line to grammars/systemverilog.ebnf and restoring it byte-identically: check_baseline_identity.sh rc=1, check_doctrines.sh rc=1, 'commit/merge blocked' — for an edit the EBNF frontend strips, leaving the generated parser byte-identical. After the redesign the same comment edit PASSES silently, a real semantic edit is a printed note (rc=0), and a deliberately-staled contract run through its gate came back GREEN and re-stamped itself: 'the baseline was STALE and every constraint still held, so this run re-stamped it'."
reverify: "bash docs/tasks/artifacts/sv_corpus_grad/baseline_identity/probe.sh   # 26 arms; 14 and 15 pin the UNCONFIRMED/STALE split, 16 pins the semantic digest"
---

**A provenance block answers "can I trust this number?", which is a question about a verdict — so
it belongs beside the verdict, not in front of the work that produces one.** Putting it in front
inverts its value: the check fires precisely in the case where *running the gate would have
resolved everything for free*.

The failure is easy to reach and hard to see coming, because a precondition looks like the careful
choice. In PGEN it meant an enforcer running from `.githooks/pre-commit` treated a moved input as a
failure — so **one comment line in a grammar blocked every commit in the repository**, for an edit
the frontend strips before the code generator ever sees it.

⭐ **The shape that works is a matrix, and only one cell needs a person:**

| identity | constraints | verdict |
|---|---|---|
| fresh | green | pass |
| fresh | **red** | **a real regression** — the sharp verdict provenance exists to enable |
| **stale** | green | the baseline was stale and *still correct* — **re-stamp it from this run** |
| **stale** | **red** | ambiguous: regression or stale baseline? **This** is the human's cell |
| adjudicated-wrong | — | refuse *before* measuring; running teaches nothing |

The third row is where the friction goes. A green run against a stale baseline has already
established everything a confirmation asserts, so re-stamping is a *derived* act — **the gate is
the confirming run**. That also deletes the escape hatch such designs otherwise need for
bootstrapping, so the fix removes a bypass surface instead of adding one.

Three guards keep it from becoming a rubber stamp:

- **Auto-re-stamp only after a fully-evaluated green.** A vacuous pass — a skipped stage, an input
  that could not be digested — must never stamp. *("A check that cannot see must say so, not
  pass.")*
- **Bound the staleness.** Not-a-failure must not mean not-a-problem: derive how long a baseline
  has been stale (commits touching its declared inputs) and hard-fail past a budget. The defect
  being prevented is **rot**, and rot is what an unbounded note becomes.
- **Kill false staleness at the source.** Digest what the consumer actually reads — a grammar's
  parsed envelope rather than its bytes — so edits that provably cannot change the artifact never
  raise the question at all → [[declare-the-producers-not-the-consumers-in-a-dependency-set]].

⚠️ **The general lesson is about where a guarantee is enforced, not whether.** Nothing here is
weaker: the same divergences are still detected, still reported, and still unbypassable. What
changed is that the check now spends its refusals only on the case a person can actually settle.

See also [[a-provenance-block-must-say-whether-the-numbers-were-ever-right]] and
[[a-cap-with-no-headroom-is-a-cap-about-to-be-raised]].
