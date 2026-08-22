---
id: a-control-that-clears-your-hypothesis-has-not-cleared-the-symptom
title: A sound control answers only the question you asked it — "not caused by X" is not "not a defect", and writing the negative result down as a non-finding is what buries it
answers:
  - "my control ruled out the cause I suspected, can I stop investigating"
  - "I proved a symptom is unrelated to the defect I am chasing — is it still a defect"
  - "how did a recorded negative result hide a real bug"
  - "is it safe to write 'recorded so it is not re-found' next to a dismissed symptom"
  - "why did a typed AST field come back empty for every input"
  - "a positional $N in a return annotation resolves to the empty string"
  - "does $N count whitespace regexes as elements"
  - "my AST-identity sweep says byte-identical for every input, should I trust it"
  - "what makes an instrument vacuous rather than clean"
tags: [controls, claim-verification, evidence, return-annotations, typed-ast, instruments, grammar-wellformedness]
date: 2026-08-22
status: current
evidence: |
  GRAMMAR-WELLFORMED.H.16.6 → H.16.6a → H.16.7 (`PGEN-GRAMMAR-WELLFORMED-0168`, `-0171`).
  `H.16.6` was diagnosing a `=>` ambiguity and noticed that `@type: 1 => 2` yields a typed AST whose
  `value` is the empty string — which reads exactly like `implication_expr`'s `-> $1` discarding its
  right operand. It ran the correct control: plain `@type: 1` and `@type: 1 + 2` yield `value: ""`
  too, so the empty value is NOT caused by `=>`. That verdict is sound and still stands. The leaf
  then recorded it under the heading *"A THIRD 'FINDING' WAS KILLED BY ITS OWN CONTROL, AND IS
  RECORDED SO IT IS NOT RE-FOUND"*, concluding the empty value was *"this annotation shape's normal
  reporting"*.
  It is not normal reporting; it is a one-character defect the control was never asked about.
  `semantic_annotation := "@" /\s*/ annotation_name /\s*/ ":" /\s*/ annotation_value` carries
  `-> {type: "semantic_annotation", name: $3, value: $6}`, and `$N` indexes EVERY top-level element
  including the layout regexes: `1="@" 2=/\s*/ 3=annotation_name 4=/\s*/ 5=":" 6=/\s*/
  7=annotation_value`. `$6` is the third separator, which always matches empty. Measured on the
  SHIPPED generated parser, 12 of 12 value shapes (integer, string, boolean, null, identifier,
  array, object, map, type, rule-ref, symbol-ref, function call) publish `value: ""`; `:36`
  `annotation` carries the identical defect; the family's own book
  (`docs/semantic_annotation_parser_book/src/ast-envelope.md:8/:17/:34`) documents a populated
  `value` with a worked example. Confirmed at the IR codegen consumes (`--dump-gen-ast`: element 6
  is `regex \s*`, element 7 is `rule_reference annotation_value`) and by fixing `$6`→`$7` on a
  scratch copy, which restores the full payload.
  It cost a second session too: in `-0171` this same empty field silently made an AST-identity sweep
  VACUOUS — it reported `ast_moved=0` for an arm deliberately built to move every map AST, because
  nothing below the entry rule can reach a dump whose only payload field is always `""`.
reverify: "printf '@type: [1, 2]' > rust/target/kmv.txt && ./rust/target/release/parseability_probe --parse-dump-ast-pretty semantic_annotation rust/target/kmv.txt rust/target/kmv.json >/dev/null && cat rust/target/kmv.json   # value:\"\" — and there is no `=>` anywhere in the input, which is exactly what the original control proved and exactly why that proof settled nothing"
---

**A control has a scope, and the scope is the hypothesis you handed it.** When it comes back negative
it has retired *that hypothesis* — nothing more. The failure mode is quiet because the control was
right: you did good work, got a clean answer, and then let the answer cover a question it never saw.

The shape, in one line each:

- **Symptom:** `@type: 1 => 2` publishes `value: ""`.
- **Hypothesis:** the `=>` handling is discarding the right operand.
- **Control:** `@type: 1` and `@type: 1 + 2` publish `value: ""` too. ⇒ **not caused by `=>`.** Sound.
- **Conclusion drawn:** *"the empty value is this annotation shape's normal reporting."* ⛔ Unearned —
  the control never compared the field against what it is supposed to hold.
- **Actual cause:** `value: $6`, and `$6` is a `/\s*/` separator. The value is `$7`.

The distance between the last two lines is the whole lesson. *Not caused by the thing I was testing*
and *not a defect* are separated by an entire second investigation, and nothing about a clean control
tells you that investigation is still owed.

## Why this one is more expensive than an ordinary miss

Two multipliers, both worth recognising in your own write-ups:

**It was filed under a heading that discourages re-examination.** The note read *"recorded so it is
not re-found."* That instruction is normally excellent — it stops sessions re-walking dead ends. Here
it converted a half-answered question into a signpost saying *do not look here*. ⇒ when you record a
negative result, record **what the control ruled out**, never *"not a finding"*. The first is a fact
with a scope; the second is a verdict with none.

**A silently-empty field makes downstream instruments vacuous, not noisy.** Because the entry rule's
only payload field is always `""`, an AST-identity sweep at that entry compares `{name, type,
value:""}` against `{name, type, value:""}` forever. It reported `1156/1156 byte-identical,
ast_moved=0` — for an arm whose `map_entry` had been retyped specifically to move every map AST. A
defect that empties a field does not announce itself; it makes everything downstream agree.

## The mechanism, which is worth knowing on its own

`$N` in a return annotation counts **every top-level element of the rule body**, layout regexes
included. That makes off-by-one the default failure rather than an exotic one:

```ebnf
semantic_annotation := "@" /\s*/ annotation_name /\s*/ ":" /\s*/ annotation_value
#                      $1    $2       $3          $4   $5    $6         $7
    -> {type: "semantic_annotation", name: $3, value: $6}    # $3 ✅   $6 ✗ — that is the separator
```

It resolves, it is in range, it always matches the empty string, and every gate stays green: the
grammar lints clean, the rule is witnessed, the parse ACCEPTS, and the certificate tuple does not
move. Distinct from the tracked out-of-range `<invalid_sequence_access>` class (see
`SV-AST-SHAPE-FIDELITY`, `POST-SV-AUDIT`, `INLINE-ALT-FIX`), which at least leaves a sentinel string
in the AST — this variant leaves nothing at all.

⇒ a static scan is the right instrument, and `docs/tasks/artifacts/grammar_wellformed/
arrow_disambiguation/positional_ref_scan.py` is the one built here: it reads the gen-AST and flags
every `$N` landing on an always-empty separator or out of range. ⛔ Its own bound is a FLOOR — it
indexes only flat top-level `Sequence` bodies (45 of 110 rules on this grammar) and mis-indexes rules
the LR eliminator rewrote.

## The practice

1. Write the control's **scope** into the record, not its verdict: *"ruled out `=>` as the cause"*,
   not *"not a finding"*.
2. When a control clears your suspect, ask the second question explicitly: **is the symptom itself
   correct behaviour?** If you cannot name what the field is *supposed* to contain, you have not
   answered it.
3. If a symptom is an always-empty or always-constant value, treat every downstream comparison over
   it as suspect until a red control proves that comparison can fail. See
   [[a-control-that-cannot-fail-is-not-a-control]].
