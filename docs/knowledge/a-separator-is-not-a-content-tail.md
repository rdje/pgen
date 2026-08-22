---
id: a-separator-is-not-a-content-tail
title: A whitespace separator is not a comment content tail — "has an unbounded repetition" is the wrong question, and it makes a live rule inert
answers:
  - "a rule with a '#{' or '//='-style terminal never matches and the grammar looks correct"
  - "my terminal's first character is a comment introducer — why is it eaten as trivia"
  - "the trace enters my rule at position N but attempts its first terminal at EOF"
  - "why does the layout skipper consume a token my grammar declares"
  - "when does PGEN suppress the engine's #, // or /* comment arm for a grammar"
  - "how do I tell whether a generated parser treats # as a comment"
  - "what is the blast radius of changing the comment-arm suppression predicate"
  - "how do I check which grammars claim a comment introducer as a real token"
  - "a certificate-coverage rule is UNKNOWN and its regex terminal compiles fine"
tags: [codegen, layout, comments, grammar-wellformedness, parse-harness, instruments, accept-set]
date: 2026-08-22
status: current
evidence: GRAMMAR-WELLFORMED.H.16.2 (`PGEN-GRAMMAR-WELLFORMED-0165`). `semantic_annotation`'s `set_value := "#{" /\s*/ … "}"` could never match: `@type: #{"a", "b"}` REJECT at position 7 while the object control `@type: {"a": 1}` PASSED, on both the generated parser and `--interpret-parse`. `node_is_unbounded_content` scored the `/\s*/` separator as a comment content tail, dropping the `#` claim, so the `#`-to-EOL arm was emitted. Fix = `hir_matches_only_whitespace`. Cert `115/0/82/33 spf=2` → `115/0/84/31 spf=0`; the pinned suppression matrix moved exactly one row of ten.
reverify: "./rust/target/release/parseability_probe --parse semantic_annotation <(printf '@type: #{\"a\"}')   # PASS — and grep -c \"== b'#'\" generated/semantic_annotation_parser.rs  # 0, the arm is suppressed"
---

**PGEN's engine has a built-in comment convention — `#`-to-EOL, `//`-to-EOL, `/* */` — and a grammar
can take those bytes back.** When a grammar assigns an introducer a *non-comment* meaning
(SystemVerilog's `#` delays, VHDL's `#` based-literal delimiters), codegen does not emit that comment
arm at all. The mechanism is real and it works:

| generated parser | emits the `#`-comment arm? |
|---|---|
| `systemverilog`, `vhdl`, `rtl_frontend`, `regex` | no — each claims `#` |
| `json`, `return_annotation`, `ebnf` | yes — none of them claims `#` |

The failure mode is not a *missing* mechanism. It is a working mechanism that scores one grammar
wrong — which is far quieter, because nothing is absent to notice and every other grammar votes that
the design is fine.

## The symptom: a rule entered at N, its first terminal attempted at EOF

```text
🚪 Entering branch 4/5 for rule 'structured_value' at position 7   <- set_value, standing on the '#'
💾 Memo miss for rule 39 at position 7 - computing fresh result
🔤 Attempting to match terminal '#{' at position 18 (end: 20)      <- its FIRST terminal, at EOF
❌ Terminal '#{' failed at position 18 - found '<EOF>'
```

That gap between *rule entry* and *first match attempt* is the whole diagnosis: the layout skipper ran
in between and consumed 7→18. Pair the reproducer with a control that differs only in the introducer
(`@type: {"a": 1}` for `@type: #{"a", "b"}`) and the verdict is unambiguous.

## The wrong question

An introducer-prefixed literal is treated as *comment-defining* — i.e. **not** a claim — when an
unbounded **content** terminal follows it. That is what makes `("#" | "//") comment_content` read as a
comment rather than as two tokens. The test used to be: *does the follower's regex contain an
unbounded repetition?*

`/[^\r\n]*/` and `/\s*/` both do. Only one of them can be a comment tail.

```ebnf
set_value := "#{" /\s*/ (set_element (/\s*/ "," /\s*/ set_element)*)? /\s*/ "}"
#            ^^^^  ^^^^^ a SEPARATOR, scored as a comment TAIL
```

`\s*` cannot swallow a `}`, cannot run to end of line, and cannot carry the arbitrary text that makes
introducer-plus-tail *mean* "comment". It is **layout**, not content. The property to test is
*can it carry non-whitespace text?* — answered by walking the HIR, conservative in the safe direction
(anything not *provably* whitespace-only keeps the old verdict).

## ⛔ This is the spelling-vs-property error, one layer down

The sibling check `uncompilable_regex_terminals` earned its design by asking **does the pattern
COMPILE** rather than *does it contain `(?`* — because the spelling test is unsound (`(?i)` compiles)
and incomplete (`(a)\1` does not). Three days later the same repository shipped *is there an unbounded
repetition* where the property was *can it carry text*. **Writing the lesson down did not apply it.**
When a defect class is fixed at one layer, grep the other layers for the same shape.

## Pin the derived decision, and the blast radius becomes a diff

`comment_arm_suppression_matrix_is_pinned` pins the `(#, //, /*)` verdict for all ten registered
grammars against ground truth read out of the shipped `generated/*.rs`. Changing the predicate and
running one test answered *"what else does this move?"* exactly — **one row, the predicted one, nine
unchanged** — with no regeneration and no argument.

⭐ **And the control that matters is not the thing you fixed.** The changed artifact was
`semantic_annotation_parser.rs`, and the annotation parsers are what codegen *links* to generate every
other parser. The load-bearing check was therefore that an **unrelated** grammar still regenerates
byte-identically through the changed backend (`json_parser.rs`, `6088e53d…` on both sides).

## Report an accept set that moves both ways as moving both ways

```text
@type: #{"a", "b"}    REJECT -> PASS      widen — from ∅; the rule matched nothing before
@type: 1 # trailing   PASS   -> REJECT    narrow — '#' is no longer a comment in this language
@type: 1 // trailing  PASS   -> PASS      unchanged — a different arm
```

The narrow is the correct reading of a grammar that defines `line_comment := "//"` and no `#` comment
at all — but *"correct"* is a judgement and the reach is a measurement: **zero** annotation lines
across all seventeen tracked grammars contain a `#`. Publish the direction *and* the reach, not only
the direction that flatters.
