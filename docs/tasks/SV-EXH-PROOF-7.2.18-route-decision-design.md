# SV-EXH-PROOF.7.2.18 — Route to literal-0 (DESIGN, no code)

Status: design-first per [[feedback_no_codebase_change_without_tool_backed_facts]].
NO code authored in this leaf. Every number is from an artifact (cited). The route-C
*code* (which touches generation) requires director sign-off before `.7.2.19+`.

## 1. The facts this design stands on (from `.7.2.17`, artifact-measured)

Residual partition (1717 residual, `/tmp/sv_gap_classified.json`, via the `.7.2.16`
classifier):

| classification | count | what it is |
|---|---|---|
| `reachable_by_plan` (branch) | 967 | a reach plan EXISTS → steerable in principle |
| `reachable_rule_not_generated` (rule) | 750 | graph-reachable rule, never generated |
| `no_reach_path` | **0** | structurally unreachable |

Conclusion already established: **literal-0 is attainable in principle** (no
structural wall). The **never_hit RULES (750) are the bigger lever**.

Scope honesty: this is the 1717 (regressed) residual, not the 888 baseline (888
coverage artifact deleted). The 888 residual is the SAME classes (fewer of each;
recorded 417 nsel-branch + 236 nhit-rule + 235 sbf, CHANGES `-0122`), all in the two
reachable buckets — so the route does not change with the baseline; only the
before-number does. Not re-measuring 888 just for a before-number that doesn't alter
the route (a 50-min gate for no decision change).

## 2. The capability gap (source-confirmed)

`set_reach_plan` / `compute_reach_path` / `try_install_reach_plan_for_status` are
**branch-only**: they require `(target_rule, node_path, branch_index)`. A never_hit
**rule** target has `node_path = None, branch_index = None`, so
`try_install_reach_plan_for_status` returns 0 for it — the reach driver
**structurally cannot target a rule today**. That is exactly why the 750 never_hit
rules are untouched by the (reverted) reach work and by the 888 baseline.

## 3. What NOT to do (ablation-proven, `.7.2.14`)

- ❌ Re-introduce on-path quantifier-forcing (`.7.2.10`). Ablation: `.7.2.10`-only
  made it WORST (replay_target_count 1982 vs 888) via **diversity collapse** —
  forced repetition → many fast, valid, near-identical samples, coverage halved.
- ❌ Aggressive rotation that starves the baseline's concentrated steering.

The 888 baseline's concentrated `.7.2.7` steering beat every ablation. Any route-C
change must be ONE change, measured against 888, instant-revert on regression.

## 4. Route C design — RULE-targeted reach WITHOUT quantifier-forcing

Goal: let the reach driver steer toward a never_hit **rule** so it gets generated,
reusing the existing (good) branch machinery, adding NO forcing.

**Key idea:** "reach a rule R" = "enter R at least once". R is entered when some
OR-decision on the path from entry selects the alternative that references R. So a
rule target reduces to a *branch* target: find an OR site + alternative whose
selection leads into R, and reuse the EXISTING `compute_reach_path` to steer to that
branch. No new forcing primitive; no quantifier expansion.

Concretely (each its own `.7.2.19+` code-leaf, ONE change + measure each):

- **`.7.2.19` (analysis only, no generation change):** add
  `compute_rule_reach_target(entry, rule) -> Option<(via_rule, node_path,
  branch_index)>` — find, via the existing rule-reference-site graph
  (`collect_rule_reference_sites`, already built in `.7.2.1`), the OR-branch on the
  shortest path that introduces `rule`, and return it as a *branch* target. Unit-test
  on a synthetic grammar (a rule reachable only as one OR-alt deep in another rule).
  This is the missing translation layer; it reuses `compute_reach_path` unchanged.
- **`.7.2.20` (driver wiring, the ONE behavior change):** in the target-drive loop,
  when the top pending target is a never_hit RULE, translate it via
  `compute_rule_reach_target` and install the SAME concentrated reach plan (no
  rotation escalation, no quantifier-forcing). Measure vs 888. Keep iff
  replay_target_count < 888; instant-revert otherwise.
- **DELIBERATELY EXCLUDED:** quantifier-forcing, multi-target rotation, any change to
  `generate_quantified`. The reach plan forces only OR choices (the part that did NOT
  cause the regression — `.7.2.11` rotation mitigated, `.7.2.10` forcing harmed).

Risk: MODERATE. It is new steering, and `.7.2.14` showed steering can collapse
diversity. Mitigation: it reuses the proven concentrated single-plan mechanism (the
888 path), adds only the rule→branch translation, and is gated by a measured
before/after vs 888 with instant-revert. NO-WORKAROUNDS: level 5 (parser-agnostic
generator capability, keyed only on the rule-reference graph), but MINIMAL — it adds
a translation, not a new forcing mode.

## 5. The honest alternative — accept-with-evidence at 888

Given `.7.2.17` proved the residual is 100% reachable, an equally-signoff outcome is:
**document the 888 residual as a classified, evidence-backed coverage gap** (not a
structural limit) and mark SV `focused_replay_target_debt` as "Mostly Done, residual
is steerable coverage debt with a per-target reachability classification." This is
honest and shippable; literal-0 then becomes a tracked future enhancement rather than
a blocker. (Director set the literal-0 bar, so this is offered, not assumed.)

## 6. Recommendation

Route C via `.7.2.19` (analysis, zero risk) **first** — it's the missing capability
and carries no generation change, so it's safe to build and unit-test now. Then bring
the `.7.2.20` driver-wiring (the one real behavior change) for sign-off + a measured
run. If `.7.2.20` doesn't beat 888, fall back to §5 (accept-with-evidence).

## 7. Decision needed from director

1. Pursue **Route C** (rule-targeted reach; start with the zero-risk `.7.2.19`
   analysis, then `.7.2.20` measured wiring), or **accept-with-evidence at 888** (§5)?
2. Confirmed constraint either way: NO quantifier-forcing; any generation change is
   ONE change measured vs 888 with instant-revert.
