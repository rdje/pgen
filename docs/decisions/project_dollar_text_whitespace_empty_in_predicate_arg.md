---
name: project-dollar-text-whitespace-empty-in-predicate-arg
description: LATENT PIPELINE FINDING — a `-> $text` (MatchedText) rule that matches a BARE WHITESPACE literal reaches a `post` `@predicate` positional arg as an EMPTY string, not the whitespace char, in the regex grammar (`@whitespace_sensitive: true`). Every non-whitespace literal AND every escape spelling (`\x20`/`\t`/`\040`) resolves correctly; only a bare whitespace literal empties. Surfaced tools-first during REGEX-PCRE2-FIDELITY.4.5.c (a `value_compare_codepoint("" > "!") → None` over-block); worked around at the grammar layer (whitespace endpoints excluded from `class_range_decodable_atom`), NOT root-fixed in the engine. Recorded for a future parser-agnostic pipeline investigation.
metadata:
  node_type: memory
  type: project
  created: 2026-07-09
  owning_tree: REGEX-PCRE2-FIDELITY
---

**THE FINDING (tools-first, proven by runtime trace).** In the `regex` grammar (`@whitespace_sensitive:
true` — whitespace is significant, never auto-skipped), a rule `R = <atom> -> $text` whose `<atom>`
matches a **bare whitespace literal** (a literal space / tab / … via `class_literal → whitespace`)
reaches a `post` `@predicate`'s **positional argument** as an **EMPTY string** (`""`), not the
whitespace character. Concretely, for `descending_class_range` gated by
`@predicate { name: value_compare_codepoint, args: [$1, gt, $5], phase: post }`:

```
[ -!]  →  value_compare_codepoint(""  > "!")  → None   ← $1 (space endpoint) resolves EMPTY
[a- ]  →  value_compare_codepoint("a" > "")   → None   ← $5 (space endpoint) resolves EMPTY
[b-a]  →  value_compare_codepoint("b" > "a")  → Some(true)   ← non-whitespace: correct
[\x20-!] → value_compare_codepoint("\\x20" > "!") → Some(false) ← escape spelling: correct
[α-β]  →  value_compare_codepoint("α" > "β")  → Some(false)   ← unicode literal: correct
```

The emptiness is **whitespace-literal-specific** and **position-independent** (either `$1` or `$5`).
`$text` compiles to `ParseContent::Terminal(&parser.input[start_pos..parser.position])`
(`ast_return_transform.rs:91`) — a correct source span in the RETURN codegen — but the value the `post`
predicate's positional-arg resolution hands to the runtime for a bare-whitespace match is empty (the
whitespace-only span is not recovered on that path). `scalar_text` does NOT trim (verified), so the
empty comes from arg resolution, not a downstream trim.

**WHY IT MATTERS.** A `None`/inapplicable predicate on a NEGATIVE-LOOKAHEAD alternative is
**non-blocking**, so the guarded rule SUCCEEDS — an over-block: `descending_class_range` wrongly matched
ASCENDING whitespace ranges (`[ -!]`, `[ --]`, `[\t-a]`), which surfaced as a
`regex_pcre2_compile_oracle_gate` regression (false_reject `48→50`). This is the SAME failure mode as an
undecodable escape endpoint (`\Q..\E`/`\u{}` → decode `None`).

**WHAT WAS DONE (`.4.5.c`, NOT a root fix).** Scoped at the GRAMMAR layer, matching the existing
`\Q..\E`/`\u{}` deferral: `class_range_decodable_atom` spells out the NON-WHITESPACE `class_literal`
alternatives directly (`class_range_decodable_escape | letter | digit | class_safe_special |
unicode_char`), dropping the `whitespace` branch, so `descending_class_range` never forms for a
bare-whitespace endpoint (a whitespace ESCAPE spelling `\x20`/`\t` stays gated). Descending
bare-whitespace ranges therefore stay validator-owned (join the `.4.5.d` deferred set). Restored oracle
`2189/1867/274/48`, cert `249/249 UNKNOWN=0`. This is in-hierarchy scoping ([[feedback_no_workarounds_fix_hierarchy]]),
NOT a masking of shipped-behavior — released `--parse` verdicts are byte-identical throughout.

**THE OPEN ENGINE QUESTION (for a future investigation).** Whether the empty-`$text`-for-a-bare-
whitespace-match is GENERAL across parsers/grammars (any `-> $text` over a whitespace-matching subrule
feeding a `post`-predicate positional arg) or specific to how whitespace-sensitivity interacts with the
predicate-arg raw capture (`semantic_capture_raw_for_post` in the generated parser). If general, the fix
is a parser-agnostic pipeline correction (the raw-capture path must recover the whitespace-only span)
that would benefit every grammar — [[feedback_ast_pipeline_parser_agnostic]] /
[[feedback_features_parser_agnostic_enable_all_parsers]]. Related surfaced quirk (also deferred): a POST
predicate on a TOP/ENTRY rule with a leading literal and no `->` mis-resolves its positional args (the
`.4.5.c` direct-on-`class_range` attempt). Both are recorded here as pointers, not yet root-caused.
Relates to [[project_rule_span_value_compare_primitive]] (the `value_compare_codepoint` consumer).
