<!-- Decision record (layer C) — migrated 2026-06-02 from harness-home memory
     (~/.claude/projects/.../memory/feedback_verify_rule_correctness_before_runtime_hypotheses.md) by MEMORY-ARCH.2 (PGEN-MEMORY-ARCH-0003).
     Now the tracked system of record; the ~/.claude copy is a cache. Content preserved verbatim below. -->

---
name: when-a-parse-failure-is-tied-to-a-specific-rule-verify-the-rule-against-its-spec-FIRST
description: "When investigating any parse failure tied to a specific grammar rule, the FIRST diagnostic step is to verify the rule's EBNF against the authoritative spec (the LRM, the PCRE2 manual, etc.) — BEFORE forming hypotheses about runtime / scope / semantic-state issues. Rule defects (missing `?` on LRM-optional clauses, missing `*` on zero-or-more, mis-attached `|` alternatives, mandatory consumes that the spec marks as optional) have OBVIOUS signatures readable in seconds by comparing the rule to the spec; runtime/scope hypotheses are expensive to disprove (instrumentation, SEMTRACE, binary search, baseline comparisons). Front-load the cheap check."
metadata:
  node_type: memory
  type: feedback
  originSessionId: 8c2d85c8-f843-4500-981d-c2bbf763bdc7
---

**Why:** rule defects are cheap to detect once isolated to a specific rule — read the EBNF, read the LRM/spec, compare. Runtime/scope/semantic-state hypotheses are expensive: each one needs targeted instrumentation, delta-debug, sometimes a SEMTRACE pass. If a rule-level defect explains the failure, NO amount of runtime analysis will reveal it — the analysis will repeatedly observe "the runtime correctly rejected an input the rule says is invalid." Investigation will be wasted disproving correct-but-irrelevant runtime hypotheses.

**Why specifically here:** the `grammars/systemverilog.ebnf` was auto-extracted from the IEEE 1800 LRM, and that extraction introduced a class of defects where LRM `[ X ]` (optional) clauses were encoded as mandatory. The `conditional_statement` defect (`SV-EXH-PROOF.3.3.4.b.1`, `PGEN-SV-EXH-PROOF-0028`) is one instance — there will be more. Any future parse failure on a rule named in our grammar deserves a rule-vs-LRM read FIRST.

**Concrete instances where this discipline would have saved time:**

1. **`SV-EXH-PROOF.3.3.4.b` (2026-05-20):** original hypothesis = "uvm_pkg fails due to intra-file scope tracking needs (post-`endpackage` fact visibility)." Spent investigation effort building a 7-step binary search to isolate the failing region inside the 89516-line preprocessed uvm_pkg.sv. The minimal repro was `module m; task t(); if (1) $display("ok"); endtask endmodule` — not a package issue at all. Root cause: `conditional_statement` at `grammars/systemverilog.ebnf:1111` mandates `else` via `&kw_else_ae050f5b kw_else_ae050f5b conditional_else_branch` with no `|` no-else alternative. Per IEEE 1800-2017/2023 §A.6.6, `[ else statement_or_null ]` is OPTIONAL. **30 seconds of "look at the rule + look at the LRM" would have revealed this immediately.** Instead the binary search took ~30 min.

2. **`SV-EXH-PROOF.3.2` (2026-05-19, also pre-this-memory):** original hypothesis = "module ANSI header doesn't accept package_import_declaration before the parameter_port_list." Falsified by minimal-repro delta-debug. The actual cause: per-digit token decomposition `kw_n_<d>_<hash> := trivia /<d>\b/` made every multi-digit number unparseable. Reading the `decimal_digit` rule against the IEEE 1800 §5.7 number forms would have shown the `\b` mis-encoding immediately.

**How to apply (every parse failure that has been isolated to a specific rule):**

1. **Locate the rule in the EBNF.** `grep -nE '^<rule_name>\s*:=' grammars/<grammar>.ebnf`.
2. **Locate the authoritative spec for that rule.** For SV: `docs/systemverilog/2017/md/*` (extracted) and `/Users/richarddje/Documents/github/SystemVerilog-LRM-IEEE-1800-{2017,2023}.pdf` (authoritative). For regex: PCRE2 manual + executable `pcre2test` oracle. For VHDL: IEEE 1076 LRM.
3. **Read the EBNF rule body.** Count quantifiers (`?`, `*`, `+`), `|` alternatives, lookahead operators (`&X`, `!X`).
4. **Read the spec production.** Count `[ ]` (optional), `{ }` (zero-or-more), `|` (alternative), grouping.
5. **Structural diff.** Common defect classes:
   - `[ X ]` in spec → MUST have `( X )?` or a `|` alternative in EBNF. If neither, it's the LRM-extraction "optional encoded as mandatory" defect.
   - `{ X }` in spec → MUST have `( X )*` in EBNF. If absent, it's the "zero-or-more encoded as exactly-one" defect.
   - `X | Y` in spec → ordering in PEG matters; check that the most-specific / longest-match alternative comes first.
   - `&X X required` pattern → if X is LRM-OPTIONAL, this is the positive-lookahead-mandates-optional defect (class `&X`).
6. **Only if the rule reads correctly against the spec, proceed to runtime/scope/semantic-state hypotheses.**

**Sub-tools that remain valuable (but AFTER step 1-6):**

- **Binary-search delta-debug** for ISOLATING which rule is failing inside a large failing input. Still the right tool for that. The discipline above kicks in once the rule is named.
- **`--trace`** for confirming the rule actually IS the locus of failure (vs. a contagion from a recursive sub-rule).
- **SEMTRACE / runtime instrumentation** for genuine runtime defects (state leakage, exception-safety bugs like the `.3.3.3` `?`-bypasses-cleanup) — appropriate only when steps 1-6 have ruled out a rule-level defect.

**Anti-pattern to flag during code review of any future investigation:**

> "I traced through `<runtime mechanism>` and concluded the bug is in `<runtime path>`."

If the investigation report doesn't include "I read rule `<X>` against IEEE 1800 §<Y>.Z and confirmed the rule is correct" as a prior step, **the investigation skipped the cheap check**. Push back, ask for the rule-vs-spec read first.

**Related memories:** [[feedback_root_cause_before_fix_code_last_resort]] (understand root cause fully before any fix), [[feedback_corpus_expected_from_spec_not_fix]] (derive expected from authoritative spec, not from the fix), [[feedback_report_expected_verify_against_oracle]] (verify against executable oracle), [[feedback_prove_independence_with_decisive_baseline]] (decisive baseline before recording "pre-existing"), [[feedback_ebnf_consult_annotation_docs]] (consult annotation docs before editing). This memory layers above all of them — it's the FIRST step in the diagnosis order, not a substitute.
