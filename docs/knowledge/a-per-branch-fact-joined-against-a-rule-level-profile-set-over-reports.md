---
id: a-per-branch-fact-joined-against-a-rule-level-profile-set-over-reports
title: "`@profiles` is RULE-level, so joining a per-BRANCH fact against `satisfiable_under` over-reports by construction — on SystemVerilog every single row that join produced was a false positive"
answers:
  - "my @sample does not parse under a profile — is that a defect"
  - "how do I tell a dead alternative from a wrong annotation"
  - "why does my profile census report defects that are all correct"
  - "how do I get per-alternative profile satisfiability out of PGEN"
  - "can a rule be live under a profile while some of its alternatives are dead"
  - "how do I join branch_semantic_annotations against profile data"
  - "why did gating a terminal make an unrelated rule go UNKNOWN"
tags: [profiles, annotations, sample, branch-satisfiability, census, instrument-soundness, false-positives, engine-universal, sv-corpus-grad]
date: 2026-08-26
status: current
evidence: "SV-CORPUS-GRAD.13e.11(a), PGEN-SV-CORPUS-GRAD-0307. `@profiles` is a rule-level annotation, but 43 of SystemVerilog's 52 `@sample` annotations are branch-level. Measured over the whole grammar with docs/tasks/artifacts/sv_corpus_grad/sample_profile_parity/sample_profile_parity.py: the RULE-level join (`satisfiable_under`) reports 8 defects over 115 checks and ALL EIGHT ARE FALSE POSITIVES; the BRANCH-level join (`branch_satisfiable_under`, added by this leaf) reports 0 over 101. The 8 are exactly the 7 SV-CORPUS-GRAD.13e.9 had adjudicated BY HAND as 'branch DEAD under v2005 — sample honest' (block_data_type 3/4, data_type 3/4, case_statement 1, module_declaration_sv_2017 3/4) plus the 1 that PGEN-SV-CORPUS-GRAD-0305 created (module_declaration_sv_2017 branch 2, whose mandatory dot_star was gated out of verilog_2005) — a derived predicate agreeing with an independent hand adjudication on 7 of 7 and supplying the 8th. `-0305` had mis-filed that 8th as a DEFECT precisely by joining on the owning RULE, and published the wrong attribution to a task leaf AND to the tracked contract rust/test_data/grammar_quality/verilog_2005_conformance_contract_v0.json, both corrected here. derive_branch_profiles (rust/src/ast_pipeline/grammar_wellformedness.rs) reuses compute_sat_by_profile/node_satisfiable — the profile-orphan lint's own fixpoint — so there is no second model of the grammar; the union of a rule's per-branch sets equals its satisfiable_under on 1537 of 1537 SV rules."
reverify: "./rust/target/debug/ast_pipeline grammars/systemverilog.ebnf --dump-rule-profiles rp.json >/dev/null && python3 -c \"import json;d=json.load(open('rp.json'))['rules']['module_declaration_sv_2017'];print('rule:',d['satisfiable_under']);print('branches:',d['branch_satisfiable_under'])\"   # the rule reads [sv_2017, verilog_2005]; branches 2/3/4 read [sv_2017] alone. Then: python3 docs/tasks/artifacts/sv_corpus_grad/sample_profile_parity/sample_profile_parity.py --grammar grammars/systemverilog.ebnf   (0 defects / 101 checks) versus --join rule   (8 defects / 115 checks, all false)."
---

# A per-branch fact joined against a rule-level profile set over-reports by construction

**Question it answers:** my `@sample` (or any other per-alternative obligation) does not parse under
one of the profiles its rule is live in. Is that a defect?

**Answer: you cannot tell from the rule's profile set, and asking it will tell you "yes" when the
answer is "no".** `@profiles` is declared per RULE. The profile filter removes rules, so a rule
survives under profile `P` as long as **some** alternative survives — while its other alternatives
are dead there. Any fact attached to a *branch* and checked against the *rule*'s `satisfiable_under`
therefore inherits an over-report that has nothing to do with the fact being checked.

## Measured — and the false-positive rate is 100 %, not "some"

| join | checks | reported | TRUE defects | false positives |
|---|---|---|---|---|
| rule-level (`satisfiable_under`) | 115 | 8 | **0** | **8 — every row it produced** |
| branch-level (`branch_satisfiable_under`) | **101** | **0** | **0** | **0** |

The canonical shape is one rule holding all three verdicts at once:

```
module_declaration_sv_2017
  declared_profiles         ["sv_2017", "verilog_2005"]
  satisfiable_under         ["sv_2017", "verilog_2005"]     <- the rule-level view: BLIND
  branch_satisfiable_under  [["sv_2017","verilog_2005"],    <- 0  module m; endmodule
                             ["sv_2017","verilog_2005"],    <- 1  module m(a); endmodule
                             ["sv_2017"],                   <- 2  module m(.*); endmodule
                             ["sv_2017"],                   <- 3  extern module m(a);
                             ["sv_2017"]]                   <- 4  extern module m;
```

Branches 2, 3 and 4 each carry an `@sample` spelling SystemVerilog that IEEE 1364-2005 cannot parse.
None of the three is a defect: those alternatives are not live under `verilog_2005` at all.

## Why this is worth a record rather than a footnote

The same repository adjudicated branches **3 and 4 by hand** and correctly called them honest — then,
one commit later, put **branch 2** in the *defect* class, because that pass joined on the owning
rule. The wrong attribution reached a task leaf and a **tracked contract file**, where it stood as
the published explanation of a certificate baseline. Nothing about branch 2 differs from its two
neighbours except which pass looked at it.

⇒ **derive the per-branch set; never widen a branch obligation to its rule.** In PGEN that is
`--dump-rule-profiles`' `branch_satisfiable_under` (indexed in the same space as
`branch_semantic_annotations`, so the two join on the index).

## The invariant that makes the new field cheap to trust

`node_satisfiable` on an `Or` is exactly *"some alternative is satisfiable"*, so the **union** of a
rule's per-branch sets **equals** its `satisfiable_under` — always. Measured over all 1 537 SV rules:
0 violations. That is what proves the branch field is a *refinement* of the rule field rather than a
rival model, which matters because a second model of the grammar is the thing you must not build.

## Two traps that ride along

- ⛔ **It is bottom-up SATISFIABILITY, never REACHABILITY.** Inherits TOOLBOX 5.11's standing trap
  verbatim. Right for a `@sample` join (a sample is parsed *at* its owning rule as the entry); wrong
  for any entry-relative claim.
- ⛔ **A branch-annotation vector can stop indexing its own body.** Left-recursion elimination
  rewrites a top-level `Or`, which is why the engine keeps `pre_lr_elim_branch_return_annotations`.
  Two SV rules (`block_event_expression`, `select_expression`) are already in that state. An
  instrument must REFUSE such a rule, not join it on a guessed index.

**See also:** [[adding-a-profile-invalidates-every-profile-blind-annotation]] — why these literals go
stale in the first place; [[a-probe-sample-says-where-generation-ended-up-never-what-the-plan-asked-for]]
— the instrument that showed the certificate half of this story had a different cause entirely;
[[an-x-is-checked-by-nothing-claim-is-a-census-claim]].
