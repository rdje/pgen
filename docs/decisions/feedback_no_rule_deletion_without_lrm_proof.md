---
name: feedback_no_rule_deletion_without_lrm_proof
description: "STANDING, emphatic (director 2026-06-17): NO grammar rule — `no_path`/unreachable or otherwise — may be deleted unless the language LRM OBJECTIVELY PROVES it has no business in the grammar. A `no_path` verdict is first read as a PRODUCER-WIRING FLAW to FIX (make the rule reachable), never a removal license. Deletion is last-last-last resort, LRM-proven-absent only."
metadata:
  node_type: memory
  type: feedback
---

Director, 2026-06-17 (emphatic, repeated): **"Every rule shall be reachable. We can't
just remove such rule. To me the `no_path` is indicative of a real issue with how the
grammar was written with respect to those `no_path` rules … deleting a `no_path` rule
shall be the last, last, last resort. We need to cross-check with language LRMs … A
`no_path` rule can't be reached maybe because the rules that are supposed to lead to them
have flaws. No rule, `no_path` or otherwise, shall be deleted unless objectively proven
otherwise (LRM, …)."**

**Why:** the EBNF is the single source of truth for the accepted language
([[project_ebnf_is_single_source_of_truth]]), and a well-formed grammar has *every rule
reachable* ([[project_grammar_wellformedness_contract]] — literal-0 is a theorem). When a
rule is `no_path` (cert-coverage: no reach path from the chosen entry), the *first*
suspicion is therefore NOT "the rule is dead, delete it" — it is "the **producer** that is
supposed to lead to it is flawed" (wrong/missing reference, un-eliminated PEG
left-recursion, mis-encoded delimiter, mis-gated branch). Deleting the stranded child
hides the real defect AND silently narrows the accepted language below what the LRM
mandates — a correctness regression masquerading as cleanup. This sharpens
[[feedback_unreachable_target_attribution_rule]]: a reach-failure is *generator deficiency*
OR *ill-formed EBNF*, adjudicated GRAMMAR-FIRST and LRM-grounded — never silently accepted,
and never resolved by deletion unless the LRM itself has no place for the rule.

**How to apply:** for ANY rule that is `no_path`/unreachable (or any deletion candidate of
any kind), BEFORE touching it:
1. **LRM cross-check (objective).** Confirm against the language LRM (IEEE 1800 SV / VHDL
   2019 / PCRE2 / RFC 8259 / …, in PDF and/or the in-repo MD workspace + extracted EBNF)
   whether the rule is a genuine LRM production. If the LRM defines it, **it belongs** —
   stop; the task is to make it reachable, not to remove it.
2. **Distinguish the THREE legitimate "unreachable-from-this-entry" cases** (the rule STAYS,
   no change): (a) rooted under a *different LRM start symbol* (e.g. SV `library_text` is a
   separate start symbol per IEEE 1800-2017 Annex A.1.1 — the library/config/include
   subtree is reachable from it, not from `source_text`); (b) *profile-relative* (a feature
   of a later LRM edition — e.g. SV-2023 interface-classes — inert under an earlier
   profile); (c) LRM-extraction decomposition artifact (a synthetic helper/keyword leaf,
   not a real production) that is blessed, not deleted-as-defect.
3. **Otherwise it is a PRODUCER-WIRING FLAW → FIX the producer** (tools-first WHY+WHERE,
   LRM-grounded) so the rule becomes reachable. Never delete to "clean up" the metric.
4. **Deletion is the last-last-last resort**, permitted ONLY when the LRM *objectively
   proves* the rule has no business in the grammar (genuinely no LRM production and no
   functional value), and even then with an explicit LRM-cited justification recorded.

Trigger/exemplar: the SV cert-coverage `no_path` set (20 rules under `sv_2017`). LRM-grounded
re-audit (`GRAMMAR-WELLFORMED.H.12.6`) found **19/20 legitimate** (10 rooted under the
`library_text` start symbol, 6 profile-relative SV-2023, 3 decomposition artifacts) and
**exactly 1 genuine producer-wiring suspect** (`module_path_conditional_expression`,
IEEE 1800-2017 Annex A.8.3 — stranded by an un-eliminated conditional left-recursion in its
producer `module_path_expression`) → routed to a FIX leaf (`H.12.6.1`), never deletion.
Reinforces [[feedback_be_alert_root_cause_fishy_immediately]],
[[feedback_tools_first_no_guessing]], [[feedback_no_codebase_change_without_tool_backed_facts]].
This also retroactively raises the bar on past "dead/subsumed rule" removals: any future
removal must clear the LRM-proven-absent bar.
