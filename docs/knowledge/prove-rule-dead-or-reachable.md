---
id: prove-rule-dead-or-reachable
title: "How to OBJECTIVELY prove a grammar rule is dead (unreachable) vs reachable-but-unwitnessed — two independent oracles; `--lint-grammar` is multi-entry-LENIENT and does NOT prove single-entry reachability"
answers:
  - "how do I prove a grammar rule is dead / unreachable from the entry"
  - "is rule X dead in grammar Y or just never witnessed"
  - "cert-coverage marks a rule UNKNOWN — is it a dead rule or a generator-reach gap"
  - "why does --lint-grammar report unreachable_rules=0 for a rule that is clearly never used"
  - "how to tell a dead branch/rule from a reachable-but-unwitnessed one (attribution rule)"
  - "objective reachability proof for an EBNF rule from the entry"
  - "what does the --gap-report-json reachable=false reason=unreachable_from_entry field mean"
  - "the same rule name is dead in one grammar but alive in another"
date: 2026-06-09
status: current
tags: [grammar-wellformedness, cert-coverage, reachability, attribution-rule, linter, stimuli-generator, gap-report, parser-agnostic]
evidence: "GRAMMAR-WELLFORMED.H.5.2 (PGEN-GRAMMAR-WELLFORMED-0052): svpp cert-coverage left 3 UNKNOWN rules (directive_tail, trivia, line_comment). The attribution rule needs each classified dead-vs-reachable. PROOF that svpp `trivia` is dead, two independent ways: (A) static reference-graph closure — `grep -nE '(^|[^a-z_])trivia([^a-z_]|$)' grammars/systemverilog_preprocessor.ebnf` returns ONLY the definition line (zero referencers; every rule uses inline_trivia), no @include, entry != trivia ⇒ trivia is in no transitive closure from systemverilog_preprocessor_file (decidable Hopcroft-Ullman reduced-grammar reachability; PEG references are static literal names). (B) independent computational oracle — `ast_pipeline GRAMMAR.ebnf --generate-stimuli --count 1 --gap-report-json OUT.json --entry-rule ENTRY` then `jq '.unreachable_rule_debt[]? | select(.reachable==false)'` reported trivia reachable=false reason=unreachable_from_entry as the COMPLETE statically-unreachable set [trivia], while directive_tail/line_comment came back reachable=true reason=never_hit (reachable optionals = generator-reach, NOT dead). KEY GOTCHA: `--lint-grammar` reported unreachable_rules=0 — it is MULTI-ENTRY-LENIENT (treats an unreferenced rule as a candidate entry), so it does NOT flag a single-entry-dead rule; cert-coverage's single-entry witness (UNKNOWN) + the gap-report oracle are what catch it."
reverify: "ast_pipeline grammars/systemverilog_preprocessor.ebnf --generate-stimuli --count 1 --gap-report-json /tmp/g.json --entry-rule systemverilog_preprocessor_file; jq -r '[.unreachable_rule_debt[]?|select(.reachable==false)|.rule_name]|sort' /tmp/g.json"
---

## The question

cert-coverage prints a rule as `UNKNOWN` (not witnessed, not proven dead). The **attribution
rule** (`docs/book/src/grammar-wellformedness.md`) says every such rule is EXACTLY one of:
(1) a **dead rule** — genuinely unreachable from the entry → fix the GRAMMAR (remove at source); or
(2) a **reachable-but-unwitnessed** rule → a generator-reach gap (improve the generator).
You must prove which — never accept it as a silent residual. Here is how to prove it objectively.

## Do NOT trust `--lint-grammar`'s `unreachable_rules` for this

`--lint-grammar` is **multi-entry-aware / multi-entry-LENIENT**: a rule with no incoming
references is treated as a *candidate entry root* (legitimate for multi-entry grammars), so it
is **not** flagged unreachable. Therefore `unreachable_rules=0` does **not** prove a rule is
reachable from *your designated entry*. (svpp's dead `trivia` rule had `unreachable_rules=0`.)

## The two independent oracles (use both; they must agree)

**(A) Static reference-graph closure** — decidable (Hopcroft–Ullman reduced grammar). In a PEG a
rule reference is always a literal rule name in some body (no computed dispatch), so
reachable(entry) = transitive closure of the reference graph. A rule is dead iff: nothing
references it (`grep` the whole `.ebnf` — beware same-name prefixes like `inline_trivia`), there
is no `@include` pulling in an external referencer, and it is not itself the entry.

**(B) The computational gap-report oracle** — a separate code path (`generate_gap_report`) that
builds the actual reference graph and classifies reachability from the entry:

```
ast_pipeline GRAMMAR.ebnf --generate-stimuli --count 1 --gap-report-json OUT.json --entry-rule ENTRY
jq '.unreachable_rule_debt[]? | select(.reachable==false)' OUT.json   # the dead set
jq '.unreachable_rule_debt[]? | select(.reason=="never_hit")' OUT.json # reachable-but-unwitnessed
```

- `reachable: false, reason: "unreachable_from_entry"` ⇒ **dead** (category 1 → remove at source).
- `reachable: true, reason: "never_hit"` ⇒ **reachable-but-unwitnessed** (category 2 → generator-reach).

## Scope the claim to ONE grammar

A rule's deadness is **per-grammar**. svpp's `trivia` is dead, but a rule *named* `trivia` is
**alive** in `rtl_frontend.ebnf` (its entry is `rtl_frontend_file := trivia design_item* trivia`).
Same name, independently-defined rules. Always state *which* `.ebnf` the claim is about.

## See also

- [[ebnf-single-source-of-truth]] — the EBNF (+ its annotations) is the sole acceptance authority.
- The grammar-well-formedness book chapter — the attribution rule + the linter-as-adjudicator + the
  "dead rules removed at source" worked example (SV's 52 dead branches).
