---
id: a-stored-derivation-a-reader-tolerates-is-a-stored-derivation-that-drifts
title: A stored derivation that a reader tolerates as a SUPERSET drifts for as long as nobody needs its exact content — the parse-cost ratchet's baseline graph was sixty rules and twenty releases stale under three green predicates
answers:
  - "the ratchet's scoped invariant reports 117 re-spelled hosts on a change that touched two rules — why"
  - "is the tracked parse_cost_ratchet rule_graph.json the baseline grammar's graph"
  - "how does check_parse_cost_ratchet.sh obtain the BASELINE grammar's reference graph now"
  - "a derived artifact R1/R3 forbids storing was stored anyway and nothing refused — how can that happen"
  - "my predicate reads a tracked JSON as ground truth and every run passes — what would tell me the file is stale"
  - "the identity table records a grammar digest but no commit — how do I recover the grammar text that produced it"
tags: [parse-cost-ratchet, derived-state, R1-R3, baseline, rule-graph, staleness, containment, instruments]
date: 2026-09-06
status: current
evidence: "SV-CORPUS-GRAD.3.38 (PGEN-SV-CORPUS-GRAD-0369). docs/tasks/artifacts/engine_universal_services/parse_cost_ratchet/rule_graph.json was committed by PGEN-SV-CORPUS-GRAD-0261 for grammar raw_ast 4ebcc61b… (1 485 rules) and never promoted again — the ratchet's own comment says a derivation is looked up, never stored — while the baseline advanced to 1e2ad24a… (1 545 rules). contained_in_introduced_subgraph, declared_alternatives_net_of_retired_work and introduced_wrapper_frames read it as `base_edges` and used it only as a superset of known names, so all three stayed green across twenty releases. The first predicate to diff the two graphs (scope_gated_arm, deriving the re-spelled hosts) reported 117 hosts; with the baseline graph derived from `git show HEAD:grammars/systemverilog.ebnf` (raw_ast 1e2ad24a…) it reported exactly the two rules the change re-spelled."
reverify: "python3 -c \"import json,sys; print(json.load(open('rust/target/parse_cost_ratchet/scratch/baseline_rule_graph.json'))['grammar'])\" after `make -C rust SHELL=/bin/bash sv_parse_cost_rebaseline` on a tree whose accepted_rises.tsv names a scoped invariant — the graph names the committed revision it was derived from; `git ls-files docs/tasks/artifacts/engine_universal_services/parse_cost_ratchet/rule_graph.json` prints nothing"
---

**The file said the right thing about itself and was wrong anyway.** The ratchet's comment on
`rule_graph.json` reads *"DELIBERATELY NOT PROMOTED — an exact function of a tracked grammar, so it
is looked up, never stored"*. That rule was applied to the scratch copy and never to the tracked one:
a single commit (`PGEN-SV-CORPUS-GRAD-0261`) had stored the graph of grammar `4ebcc61b…`, and every
later release promoted `rule_costs.tsv` beside it while the graph stayed. `git log` on the file:
one commit.

**Why three predicates never noticed.** `contained_in_introduced_subgraph`,
`declared_alternatives_net_of_retired_work` and `introduced_wrapper_frames` read the tracked graph as
`base_edges` and used it for one thing — *was this rule name known before?* A graph sixty rules
behind still answers that correctly for every rule that existed in both, and the rules it lacks
are exactly the ones the change is allowed to introduce. A reader that only needs a superset is
satisfied by any superset, including a stale one, so the staleness had no observable.

**The first exact reader found it in one run.** `scope_gated_arm` (`SV-CORPUS-GRAD.3.38`) derives the
HOSTS of a change — the pre-existing rules whose reference lists differ between the baseline graph
and the measured graph — and refused with *"117 pre-existing rule(s) were re-spelled without
consulting an introduced rule"* on a change that re-spelled two. The 115 extra hosts were twenty
releases of grammar work the stored graph had never seen.

**The fix is the doctrine's own, applied to the reader as well as the writer.** The tracked file is
deleted; `check_parse_cost_ratchet.sh` now derives the baseline graph at evaluation time from the
committed grammar revision whose raw_ast digest the identity table records (`git show
<rev>:grammars/systemverilog.ebnf`, `derive_graph`, digest-verified), walking recent revisions of the
grammar until one reproduces the digest and REFUSING when none does — an acceptance evaluated
against a guessed baseline graph is worse than none. ⭐ **R1/R3 has a reader's half: a derivation
that must not be stored must not be read from a store either.** A reader that tolerates staleness is
precisely how the stored copy survives.
