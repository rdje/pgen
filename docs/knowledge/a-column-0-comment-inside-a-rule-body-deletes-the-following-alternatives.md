---
id: a-column-0-comment-inside-a-rule-body-deletes-the-following-alternatives
title: A `#` comment at COLUMN 0 inside an EBNF rule body ENDS the rule and silently discards every alternative after it — indent it, because no lint, no rule count and no profile census can see the loss
answers:
  - "I added an alternative to a grammar rule and the parser behaves as if it is not there"
  - "my grammar fix regressed cases that were passing before and the fix cannot explain it"
  - "how do I check whether the frontend actually read all of a rule's alternatives"
  - "is it safe to put an explanatory comment between two alternatives of a rule"
  - "the rule count did not change so my grammar edit must be inert — is that sound"
  - "which instrument can detect language silently lost between the EBNF and the parser"
  - "how do I isolate whether `#` is mishandled in comments or in strings"
tags: [ebnf-frontend, grammar-authoring, silent-data-loss, instrument-blindness, source-of-truth, root-cause]
date: 2026-08-09
status: current
evidence: docs/tasks/EBNF-FRONTEND-SILENT-TRUNCATION.md; docs/tasks/SV-CORPUS-GRAD.md leaf .3.19; docs/tasks/artifacts/sv_corpus_grad/config_use_param_override/frontend_truncation_probe.sh (the five-case discriminator) and frontend_truncation.txt; truncation_census.py / truncation_census.txt (the repo-wide "is any language missing today" census); grammars/systemverilog.ebnf:6042 (the in-place ⛔⛔ warning at the site)
reverify: "bash docs/tasks/artifacts/sv_corpus_grad/config_use_param_override/frontend_truncation_probe.sh | grep -q 'A_col0_comment_after_alt1 *1' && python3 docs/tasks/artifacts/sv_corpus_grad/config_use_param_override/truncation_census.py >/dev/null && echo TRUNCATION-KNOWN-AND-ZERO-LIVE-SITES"
---

**The EBNF is only the single source of truth if the frontend reads all of it.** A comment line
beginning at **column 0** inside a rule body terminates that rule. Every alternative after it is
discarded — along with the rule's return annotation if it sat below them. There is no error, no
warning, and no diagnostic of any kind.

```ebnf
scratch := "a" "b"          # ⛔ WRONG — this rule now has ONE alternative
# a comment at column 0
        | "c" "d"
        | "e" "f"

scratch := "a" "b"          # ✅ RIGHT — three alternatives, as written
        # an indented comment
        | "c" "d"
        | "e" "f"
```

**Do not guess which `#` is to blame — the obvious reading is wrong.** When this first bit, the
offending edit contained a `#` in a comment *and* a `#` inside a quoted `@probe_sample` string, and
"`#` is mishandled" is the natural conclusion. It is false. Five one-rule grammars, each declaring
three alternatives and differing in exactly one thing, settle it:

| case | shape | IR alternatives (want 3) |
|---|---|---|
| baseline | no comment | **3** |
| indented comment between alts | comment, indented | **3** |
| `#` inside a `@probe_sample` STRING | `@probe_sample: "x #(y)"` | **3** |
| column-0 comment after alt 1 | comment at col 0 | **1** — the node degrades `Or` → `Sequence` |
| column-0 comment after alt 2 | comment at col 0 | **2** |

Strings are handled correctly. Only the column-0 line position matters.

**Why nothing else catches it — this is the part worth remembering.** With two alternatives silently
gone from a real SystemVerilog rule:

- `ast_pipeline --lint-grammar` — **clean**
- `defined_rule_count` — **1477, unchanged**
- `ast_pipeline --dump-rule-profiles` — **byte-identical** across all 1 477 rules and all three
  profiles

All three are *rule-level* instruments. No rule is added or removed here; only the inside of one
rule shrinks. A census that counts rules is structurally incapable of seeing it, and quoting one as
evidence of inertness is a mistake this exact leaf made minutes before the symptom appeared.

**The two things that DO see it:**

```bash
# 1. read the IR directly — count the alternatives the generator will actually consume
./rust/target/debug/ast_pipeline grammars/<g>.ebnf --generate-stimuli --count 1 --seed 0 \
    --dump-gen-ast /tmp/gen.json
python3 -c "import json;n=json.load(open('/tmp/gen.json'))['grammar_tree']['<rule>'];\
print(len(n['Or']['alternatives']) if 'Or' in n else 1)"

# 2. keep control rows in every repro matrix — rows that must NOT move
#    (this is what caught it: four already-PASSING cases flipped to REJECT)
```

**And the direction of the failure was luck.** Truncation removed alternatives, so the parser
under-accepted and a control row went red. The same mechanism applied to a rule whose later
alternatives carry `@predicate` gates or negative lookaheads removes *strictness* — the parser
silently **over-accepts**, and nothing goes red at all.

Measured at discovery: **0 truncating sites across every tracked grammar**, so nothing shipped is
wrong today. It is a live trap, not a live defect — which is exactly why it is worth retrieving
before writing a grammar comment rather than after.

See also [[a-rising-pass-rate-is-not-evidence-of-correctness]],
[[a-rule-with-no-shape-sample-is-checked-for-parseability-only]].
