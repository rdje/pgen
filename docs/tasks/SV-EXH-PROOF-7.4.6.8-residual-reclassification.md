# SV-EXH-PROOF.7.4.6.8 — the literal-0 residual, RE-CLASSIFIED on a deterministic metric

> Investigation slice. **No code change.** Pure docs.
> Owner leaf: `SV-EXH-PROOF.7.4.6.8` (see `docs/tasks/SV-EXH-PROOF.md`).
> Standing discipline: [[feedback_why_and_where_before_solution]],
> [[feedback_no_codebase_change_without_tool_backed_facts]], TOOLBOX-FIRST.

---

## 0. TL;DR

Three findings, each tool-backed, each new:

1. **`.7.4.6.8`'s original mandate is already discharged — cross-tree.** `.7.4.6.7` closed
   with *"Next leaf (`.7.4.6.8`): make the residual DETERMINISTIC … so the count is
   signal"*. That work landed under a **different tree** as
   `GRAMMAR-WELLFORMED.B1` (`PGEN-GRAMMAR-WELLFORMED-0005`, 2026-06-05): the wall-clock
   generation deadline was replaced by a step-counter budget, and the gate residual
   measured **84 twice, identically**. The literal-0 metric is signal, not noise.
2. **The residual then drifted 84 → 127 while this lane was parked, and nothing noticed.**
   The canonical gate has **no ratchet** on `closed_loop_replay_targets_total`: the residual
   is the *"Mostly Done"* gap, not a gate failure, so the gate PASSES at any residual. A
   grammar change that adds branches silently raises it.
3. **The residual's dominant failure class has completely flipped, and it is now ONE defect.**
   Every prior slice fought a `target_timeout` tail (91–99 % of the residual). That tail is
   **gone — `target_timeout=0`.** What remains is `attempted=true`, generated without error,
   and *still not credited*. And **40 of the 62** (profile_2017) sit in a single sub-grammar:
   the transitive cone of **`property_expr`** — the SVA property-expression cascade landed
   on 2026-06-25.

⇒ literal-0 for SV is no longer a generation-performance problem. It is **three named
defects**, one of which (class A) is a single connected sub-grammar.

---

## 1. Method (tools-first)

| # | Instrument | What it answered |
|---|---|---|
| 1 | `rust/target/sv_stimuli_quality_gate/summary.txt` (canonical gate output) | the canonical residual + the gate's own posture (`closed_loop_target_generation_timeout_ms`, `target_max_attempts`) |
| 2 | `profile_{2017,2023}_replay_gap.json` (`jq`) | the residual target list with `reason` / `reach_classification` / `node_path` / `branch_index` |
| 3 | `profile_2017_closed_loop_replay.log` (gate-logged run) | the **witness-pass summary line** + the `still-UNRESOLVED` / `other-failure` sample blocks (the `.7.4.6.4` tooling) |
| 4 | `stimuli_generator.rs` source read | the exact definitions of `never_selected` / `selected_but_failed` (`:2686-2698`) and of the `node_path` encoding (`:7447-7461`) |
| 5 | `git log -S` on `grammars/systemverilog.ebnf` | when the dominant residual rule entered the grammar |

⚠️ **A correction made during this slice, kept deliberately.** An early `awk` extraction of
`net_declaration_sv_2017` bled past the rule's end into `net_declaration_sv_2023`, making the
rule look like it carried three exact-duplicate dead alternatives — i.e. a regression of the
`.7.4.6.7` de-dup. It does not. A bounded re-extraction against the raw grammar text shows
**4 alternatives, no duplicates**. Recorded because it is exactly the failure mode the
TOOLBOX-FIRST directive exists to prevent: *do not eyeball a grammar.*

---

## 2. Finding 1 — the metric is already deterministic (cross-tree)

`GRAMMAR-WELLFORMED.B1` (`PGEN-GRAMMAR-WELLFORMED-0005`) removed `std::time::{Duration,
Instant}` from the stimuli generator. `GenerationTimeoutBudget` / `ActiveGenerationDeadline`
now carry a **step** budget (`stimuli_generator.rs:100-118`), bumped once per
`generation_deadline_exceeded()` call (`:2011-2021`), with the ms config mapped through
`generation_steps_per_ms()` (default 1000, env `PGEN_GENERATION_STEPS_PER_MS`).

⇒ the only non-seeded input to generation is gone; a seeded run yields the same residual
every time. B1 confirmed it by running the canonical gate **twice → 84 both times**.

**Consequence for this tree:** `.7.4.6.7`'s planned `.7.4.6.8` is discharged. This leaf is
re-scoped to *what the now-trustworthy number actually says*.

---

## 3. Finding 2 — the residual drifted 84 → 127, unratcheted

| Date | Commit / event | `closed_loop_replay_targets_total` | `closed_loop_initial_targets_total` |
|---|---|---|---|
| 2026-06-04 | `.7.4.6.3` construction | 97 | 5289 |
| 2026-06-05 | `.7.4.6.7` de-dup | 105 | 5243 |
| 2026-06-05 | **`GRAMMAR-WELLFORMED.B1`** (deterministic) | **84** (×2, identical) | — |
| 2026-07-25 | canonical gate run (`summary.txt`) | **127** | **5461** |
| 2026-08-01 | canonical gate run, **this slice** (exit 0, 33 min, peak RSS 7 025 MB) | **127** (62 + 65) | **5461** |

⭐ **The 2026-08-01 run reproduces 2026-07-25 exactly** — same total, same per-profile split,
same three-class partition, and a **byte-identical** witness-pass summary line for
profile_2017 — across a **seven-day gap and an intervening grammar change**
(`PGEN-QUANT-PLUS-ITER-0004`, 2026-07-26, `@entry: true` made mandatory). That is an
independent confirmation of `GRAMMAR-WELLFORMED.B1`'s determinism claim, stronger than B1's
own back-to-back double run, and it is why every number in §4 is stated without a tolerance.

Gate verdict for the fresh run: **PASS** — `closed_loop_profiles_passed: 2/2`, realistic
corpus 730/730, `parse_full` failures 0. **The gate passes at residual 127**, which is
precisely finding (2).

The universe grew **+218 targets** and the residual grew **+43**. The dominant new
contributor entered on **2026-06-25** via `PGEN-GRAMMAR-WELLFORMED-0133` — the SVA property
precedence cascade (`prop_primary_sv_2017` / `prop_primary_sv_2023`), which alone accounts
for **52 of the 127**.

**The structural point is not the number, it is the absence of a ratchet.** The gate treats
the residual as a reported metric, never as a constraint (`closed_loop_replay_targets_total`
is echoed at `sv_stimuli_quality_gate.sh:3368`, never compared). So:

> Any grammar change may raise the literal-0 residual by an arbitrary amount and every gate
> still passes.

This is the same class of defect the project has repeatedly ruled against — an instrument
that reports without refusing ([[feedback_enumerating_instrument_must_refuse]],
[[feedback_instrument_needs_ground_truth]]). ⇒ routed as its own leaf (`.7.4.6.10`).

---

## 4. Finding 3 — the residual is now three named defects

### 4.1 The timeout tail is gone

Witness-pass summaries, canonical gate run 2026-08-01
(`profile_{2017,2023}_closed_loop_replay.log`):

```
# profile_2017
Witness pass: resolved 872 -> 2631 of 2693 reachable targets (+1759 via 925 witnesses;
  failures depth_exceeded=0, rule_visit_limit=0, target_timeout=0, helper_timeout=0,
  other=1, no_entry=0; construct_fell_back_to_search=12)

# profile_2023
Witness pass: resolved 568 -> 2703 of 2768 reachable targets (+2135 via 1071 witnesses;
  failures depth_exceeded=0, rule_visit_limit=0, target_timeout=0, helper_timeout=0,
  other=1, no_entry=0; construct_fell_back_to_search=12)
```

`target_timeout=0`. Every prior slice in this campaign (`.7.4.4` → `.7.4.6.5`) was fighting a
tail that was 91–99 % timeouts. The combination of B1's step budget, `.7.4.6.5`'s `Cow`
speedup and `.7.4.6.3`'s backtrack-free construction **closed that class entirely.**

⇒ **2 693 − 2 631 = 62 unresolved, against exactly 1 recorded failure.** The residual is no
longer "generation failed". It is *"a witness was generated, and the target still was not
credited"*.

### 4.2 The partition

Classification of the 62 (profile_2017) by `target_type` × `node_path` shape, cross-checked
against the grammar:

| Class | n (2017) | n (2023) | Signature | Mechanism |
|---|---|---|---|---|
| **A — the `property_expr` cone** | **40** | 40 | `selected_but_failed`, `node_path=root` | witness generated, target not credited |
| **B — inner choice under a quantifier** | **20** | 23 | `node_path` contains `/q` (17 `never_selected` + 3 `selected_but_failed`) | the enclosing optional/repeat group is never expanded |
| **C — store-gated unsatisfiable** | **2** | 2 | `never_hit` (rule) + its consequent branch | `fact_count_at_least` predicate unsatisfiable from an empty store |

40 + 20 + 2 = **62** ✅ — the partition is exact and total, and near-identical across both
LRM profiles (a strong signal that all three are structural, not sampling artifacts).

`node_path` grammar is source-confirmed at `stimuli_generator.rs:7447-7461`: `/o{i}` = `Or`
alternative *i*, `/s{i}` = `Sequence` element *i*, `/q` = **inside a `Quantified` node**.

### 4.3 Class A — one defect, not forty

Every class-A target lies in the transitive cone of **`property_expr`**. Verified by bounded
extraction of each residual alternative from the grammar:

| Residual target | its alternative | reaches `property_expr` via |
|---|---|---|
| `prop_primary_sv_2017#3..27,29` (26) | the temporal/implication operators | **directly** |
| `prop_iff_sv_2017#0` | `prop_or_sv_2017 iff prop_iff_sv_2017` | the cascade itself |
| `property_actual_arg#0` | `property_expr` | directly |
| `property_case_item#0` | `… colon property_expr semi` | directly |
| `concurrent_assertion_statement#0,1,2,4` | assert/assume/cover/restrict `_property_statement` | `property_spec` |
| `concurrent_assertion_item#0` | `(block_identifier colon)? concurrent_assertion_statement` | ↑ |
| `procedural_assertion_statement#0` | `concurrent_assertion_statement` | ↑ |
| `assertion_item_declaration#0` | `property_declaration` | `property_spec` |
| `clocking_item#2` | `attribute_instance* assertion_item_declaration` | ↑ |
| `checker_or_generate_item_declaration#3` | `assertion_item_declaration` | ↑ |
| `package_or_generate_item_declaration_sv_2017#13` | `assertion_item_declaration` | ↑ |
| `statement_item_sv_2017#19` | `expect_property_statement` | `property_spec` |

**The complement is the proof.** `prop_primary_sv_2017` has 30 alternatives. Exactly four are
covered — **0** (`sequence_expr`), **1** (`strong(sequence_expr)`), **2**
(`weak(sequence_expr)`) and **28** (`property_instance`) — and those four are *precisely the
four that do not mention `property_expr`*. All 26 that do are residual. The partition is
perfect and mechanical:

> A `prop_primary` alternative is coverable **iff** it does not re-enter the `property_expr`
> recursion cycle.

**WHERE (named, from the gate log).** Witness construction rooted at the property-statement
rules dies inside one rule:

```
Exit generate_from_entry(entry='cover_property_statement')   with error:
  Stimuli generation depth exceeded max_depth=40 while expanding rule 'tagged_union_expression_sv_2017'
Exit generate_from_entry(entry='expect_property_statement')  … same
Exit generate_from_entry(entry='property_declaration')       … same
Exit generate_from_entry(entry='restrict_property_statement')… same
```

⚠️ **Honest bound — this is a located symptom, not yet the closed causal chain.** The
witness summary reports `depth_exceeded=0` while these lines appear in the same run, so the
failures were *absorbed* (`construct_fell_back_to_search=12`) and the fallback search then
produced a witness that did not credit the target. The remaining open question — **why the
generated witness is not credited to the forced branch** — is the fix leaf's first job, and
must be answered with a branch-selection trace (TOOLBOX Protocol D, the
`🏁 selected branch N/M` line under `--trace-rules prop_primary_sv_2017`), **not** by
reasoning from the grammar.

### 4.4 Class B — inner choices under an optional group

All 20 carry `/q` in their `node_path`. ⚠️ **The class is not homogeneous** — it splits
**17 `never_selected` + 3 `selected_but_failed`**, and the fix leaf must treat the two
sub-shapes separately:

```
# B1 — never_selected (17): the group is never entered at all
constraint_primary_sv_2017#0,#1              @root/s0/q
event_expression#0,#1                        @root/s1/q/s0
hierarchical_btf_identifier#0,#1             @root/o2/s0/q
method_call_receiver_sv_2017#0,#1            @root/o1/s0/q  and  @root/o13/s5/q
nonrange_variable_lvalue#0,#1,#2             @root/s0/q
ps_or_hierarchical_array_identifier#0,#1,#2  @root/s0/q
split_hierarchical_callable_receiver#1       @root/s0/q

# B2 — selected_but_failed (3): the group WAS entered, the sample then failed
ps_type_identifier_sv_2017#1,#2              @root/s0/q
randomize_call#0                             @root/s2/q/s1/q
```

e.g. `nonrange_variable_lvalue := ( implicit_class_handle dot | package_scope | class_scope )?
hierarchical_variable_identifier nonrange_select` — the three inner alternatives are only
reachable if the `?` group is **taken**. The witness pass forces the target *rule*, but
nothing forces the enclosing quantifier to expand, so the generator takes the zero-repeat
path and the inner choice is never selected.

⛔ **The obvious fix is already implemented — do not re-implement it.** `ActiveReachPlan`
carries `forced_quantifier_min` (`stimuli_generator.rs:1107`), populated to `1` from
`Self::quantifier_sites_along_path(...)` at `:3076` (plannable-rule plans) and `:3402`
(structured-witness sub-plans). So class B is **not** "quantifier forcing is missing"; it is
"the existing forcing does not reach these sites".

**Where the gap most likely is** — stated as the fix leaf's *first question*, not as a
conclusion: both call sites collect quantifier sites along **inter-rule hops**
(`hop_rule, hop_site_path`), while every class-B target sits at an **intra-rule** `/q` site
inside the target rule's own body. Whether `set_reach_plan_for_rule` collects intra-rule
quantifier sites on the way to the target branch is what must be measured first.

Still likely the cheapest of the three to close, and general — it benefits every PGEN grammar.

### 4.5 Class C — the store-gated pair

```
target_id='rule::wildcard_escape_nettype_identifier' … reason=STORE-AWARE-GEN:
  rule 'wildcard_escape_nettype_identifier' fact_count_at_least predicate unsatisfiable (zero source facts)
```

plus its consequent `net_declaration_sv_2017#2`, whose alternative *is*
`wildcard_escape_nettype_identifier ( delay_control )? list_of_net_decl_assignments semi`.
One cause, two targets. This is STORE-AWARE-GEN territory
([[feedback_grammar_rules_must_consult_store]]): the witness must first emit a `pkg::*`
wildcard import so the fact count is non-zero.

---

## 5. What this means for literal-0

The bar (`focused_replay_target_debt_zero` ⇒ SV main parser `Mostly Done` → `Done`) is
unchanged. What changed is that it is now **a bounded, named work list** rather than an
open-ended sampling fight:

| Class | n | Fix direction | Risk |
|---|---|---|---|
| B | 20 / 23 | force enclosing quantifiers to expand on the reach path | low, general, parser-agnostic |
| C | 2 / 2 | store-aware witness prelude (emit the enabling fact) | medium; existing STORE-AWARE-GEN lane |
| A | 40 / 40 | **WHY+WHERE first** (Protocol D branch-selection trace) — non-credit mechanism in the `property_expr` cone | unknown until traced; largest |

Plus the cross-cutting **ratchet** finding (§3), which is what lets any of this *stay* fixed.

⛔ Sequencing note for whoever takes this: class A is 65 % of the residual but is the only
one whose mechanism is **not yet closed**. Per [[feedback_why_and_where_before_solution]] its
fix leaf must open with the trace, not with a change.

---

## 6. Reproduce

```bash
# the canonical residual (≈25 min, memory-guarded)
scripts/run_with_memory_guard.sh --budget-mb 12288 --timeout-s 5400 -- \
  make -C rust SHELL=/bin/bash sv_stimuli_quality_gate
grep closed_loop_replay_targets_total rust/target/sv_stimuli_quality_gate/summary.txt

# the partition (per profile)
jq -r '.targets[] | if .target_type=="rule" then "C" elif (.node_path|test("/q")) then "B" else "A" end' \
  rust/target/sv_stimuli_quality_gate/work/profile_2017_replay_gap.json | sort | uniq -c

# the witness-pass verdict + the still-UNRESOLVED sample block
grep -A45 "^Witness pass:" rust/target/sv_stimuli_quality_gate/logs/profile_2017_closed_loop_replay.log
```
