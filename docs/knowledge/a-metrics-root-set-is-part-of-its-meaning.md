---
id: a-metrics-root-set-is-part-of-its-meaning
title: A reachability metric's ROOT SET is part of its meaning — `unreachable_rules=0` does not mean no rule is unreachable from your entry
answers:
  - "the linter says unreachable_rules=0 but the certificate pass says N rules have no reach path from the entry — which one is wrong"
  - "how do I adjudicate the dead-rule candidates --report-certificate-coverage warns about"
  - "why does --lint-grammar not flag a rule nothing references"
  - "is an unreferenced rule unreachable"
  - "which tool finds rules that are reachable only from an orphan root"
  - "my grammar lint is green but rules are still UNKNOWN in cert coverage"
  - "how do I tell a grammar's dead code from PGEN's own LR-elimination residue"
  - "a rule is UNKNOWN and the lint is clean — what do I run next"
  - "does a green well-formedness lint mean the grammar has no unreachable rules"
tags: [grammar-wellformedness, linter, certificate-coverage, reachability, instruments, left-recursion, ast-pipeline]
date: 2026-08-22
status: current
evidence: GRAMMAR-WELLFORMED.H.16.1 (`PGEN-GRAMMAR-WELLFORMED-0164`). `ebnf` / `return_annotation` / `semantic_annotation` each read `--lint-grammar` `unreachable_rules=0, exit 0` while `--report-certificate-coverage` named **31 / 2 / 31** rules with `NO reach path from the entry`. Cause, in the detector's own doc-comment (`rust/src/ast_pipeline/grammar_wellformedness.rs:386`): roots = the canonical entry PLUS every rule NOTHING references. Both readings are correct; they quantify over different root sets. The 64 split further into **9 LR-elimination residue + 55 source orphans** once the PRE-elimination raw AST is compared against the POST-elimination gen-AST.
reverify: "./rust/target/debug/ast_pipeline grammars/ebnf.ebnf --lint-grammar | grep -o 'unreachable_rules=[0-9]*'   # 0 — then: python3 docs/tasks/artifacts/grammar_wellformed/residual_island_census/probe.py --grammar ebnf --entry grammar_file   # post_outside=31 lr_residue=4 source_orphans=27 islands=14"
---

⛔ **PRIOR ART, and this card was written without finding it** (corrected `PGEN-GRAMMAR-WELLFORMED-0170`):
`LANG-CAPABILITY-AUDIT.2` (`done`, session #208) had already resolved exactly this — *"the linter is
NOT wrong, and neither is the closure"* — quoting the same `grammar_wellformedness.rs` doc-comment.
`LANG-CAPABILITY-AUDIT.1` had already measured the same 27 `ebnf` productions. What the 2026-08-22
re-derivation adds is the **LR-residue vs source-orphan split** and an exact cross-method
corroboration (27 = 27). ⭐ The card stands, and so does the lesson about searching the trees, ADRs and
KM cards *before* declaring a reading missing.

**A green `unreachable_rules=0` is not the statement "every rule is reachable from the entry".** It
is the statement *"every rule is reachable from **some root**"* — and PGEN's root set is the
canonical entry **plus every rule that nothing references**. An unreferenced orphan is therefore a
ROOT, never an "unreachable rule", and everything hanging off it inherits that pass.

This is deliberate and documented. `detect_unreachable_rules` says so itself:

> Roots = the canonical entry (`rule_order[0]`) PLUS every rule that NOTHING references (a secondary
> entry, e.g. a `*_multi_entry_root` that unions in the alternative start symbols). … Multi-entry-SAFE
> (an unreferenced top is a root, never a false "unreachable") and conservative (an unreferenced dead
> orphan is treated as a root → not flagged; only referenced-but-unreachable dead ISLANDS are caught
> — false negatives are safe, false positives would wrongly reject a good grammar).

The trap is not that the lint is wrong. **The trap is that a deliberately conservative instrument
reports its conservatism as a PASS.** Nothing in `unreachable_rules=0` says *"…and 31 of your rules
cannot be reached from the entry you declared"*. Measured on three grammars at once:

```text
grammars/ebnf.ebnf                --lint-grammar  unreachable_rules=0  exit 0
                                  --report-certificate-coverage        31 rules, NO reach path from 'grammar_file'
grammars/return_annotation.ebnf   --lint-grammar  unreachable_rules=0  exit 0
                                  --report-certificate-coverage         2 rules, NO reach path
grammars/semantic_annotation.ebnf --lint-grammar  unreachable_rules=0  exit 0
                                  --report-certificate-coverage        31 rules, NO reach path
```

## The two instruments are COMPLEMENTS, and most defects live in the seam

They partition the space, and neither is a superset of the other:

| the rule is… | `--lint-grammar` | the entry-relative reading |
|---|---|---|
| reachable from the entry | pass | pass |
| reachable **only** from an unreferenced orphan root | **pass** (the orphan is a root) | **flagged** |
| reachable only from another rule that is itself unreachable, with **no** orphan root (a dead cycle) | **flagged** | *hands it back* |

Proven both ways on a synthetic control — `start := "a"`, `dead1 := ("b" dead2 | "z")`,
`dead2 := "c" dead1`. Because `dead1` and `dead2` reference *each other*, neither is an orphan root,
so the lint reads `unreachable_rules=2` while the island census reports *"the partition is NOT
closed — adjudicate these by hand"*. On the three real grammars the readings are exactly reversed.

## Answering the entry-relative question

Two tools, in this order:

1. `PGEN_CERT_RESIDUAL_CLASSIFICATION=1` on `--report-certificate-coverage` (TOOLBOX 4.6) sorts the
   residual into `profile_entry_unreachable` / `store_unproducible_under_profile` / `genuine`.
2. `docs/tasks/artifacts/grammar_wellformed/residual_island_census/probe.py` groups the
   entry-unreachable set into **islands** under the orphan root that reaches each one, and splits
   them by comparing the grammar with itself in two forms.

## ⛔ Compare the grammar AS WRITTEN with the grammar AS CONSUMED, or you will record engine artifacts as grammar facts

`--emit-raw-ast-json` is the frontend's view of the source; `--dump-gen-ast` is what codegen and the
certificate pass actually consume — **after** the left-recursion eliminator has rewritten referrers
into `<base>_lr_base` / `_lr_suffix` / `_lr_seed_*` helpers. A rule the source wires can be orphaned
by that rewrite alone:

```text
ebnf                 pre_outside=27  post_outside=31   lr_residue=4   source_orphans=27
return_annotation    pre_outside=1   post_outside=2    lr_residue=1   source_orphans=1
semantic_annotation  pre_outside=27  post_outside=31   lr_residue=4   source_orphans=27
```

Nine of the sixty-four are **PGEN's own residue**, not grammar debt — the lint names each base rule
on its `left_recursion_eliminated` info line (`ebnf` `return_expression` indirect,
`return_annotation` `accessor_base`, `semantic_annotation` `type_reference`). The same class had
already been adjudicated once for `verilog_2005` (*"19 are PGEN's own LR residue"*); finding it in
three more grammars is what makes it an engine-wide accounting property rather than one family's
curiosity. Read the POST arm alone and those nine become a grammar-debt number that no grammar edit
can ever retire.

## The general rule

Before choosing a reachability instrument, ask **what it roots at** — and before trusting a green
count, ask whether the thing you care about is inside its quantifier. A conservative metric that is
correct on its own terms will report a population it cannot see as a clean bill of health.
