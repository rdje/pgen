---
id: a-guard-that-can-never-pass-is-invisible-from-outside
title: A guard that can never pass and a loop that is never needed look identical from outside — probe the guard's SUBJECT, don't read the guard
answers:
  - "my rule has a lookahead guard — how do I know it ever fires"
  - "this alternative parses but never takes the branch I wrote it for"
  - "a repetition in my grammar seems to run zero times — how do I check"
  - "how do I tell a dead negative lookahead from one that is simply not needed"
  - "the rule is selected on every input, so why does the construct still reject"
  - "how do I sweep a grammar for guards that can never pass"
  - "why did nothing in the test suite notice this dead code path"
tags: [grammar-authoring, dead-code, negative-lookahead, quantifiers, instrument-honesty, sweep]
date: 2026-08-18
evidence: docs/tasks/SV-CORPUS-GRAD.md leaf .13c.2c (`split_hierarchical_callable_receiver`'s `!callable_method_call_body` — zero loop iterations for its whole life; axis-2 bar 300 -> 289, corpus pass 9 776 -> 9 786, 0 rows worsened); docs/tasks/artifacts/sv_corpus_grad/dead_negative_lookahead/ (the sweep, its four controls, and sweep_before.txt / sweep_after.txt = DEAD-RISK 1 -> 0); docs/tasks/LANG-CAPABILITY-AUDIT.md leaf .10.18 (why the guard existed at all — PGEN's `*` is possessive)
reverify: "python3 docs/tasks/artifacts/sv_corpus_grad/dead_negative_lookahead/sweep_dead_guards.py   # exit 0, dead_risk=0; add --grammar <pre-fix revision> to watch it exit 1"
status: current
---

**A negative lookahead `!R` guarding a position where an identifier is legal is DEAD if `R` accepts
a bare identifier — and nothing about the parse will tell you.** The rule still parses. Its branch
is still selected. The linter is still quiet. The tests still pass. What you lose is silent: the
guarded loop never iterates, so the rule quietly degenerates into a shorter rule than the one you
wrote.

Measured, in `grammars/systemverilog.ebnf`:

```ebnf
split_hierarchical_callable_receiver :=
    qual? ( identifier constant_bit_select dot !callable_method_call_body )* identifier constant_bit_select
```

The guard was meant to stop the receiver before the method name. But IEEE 1800 A.8.2 writes
`array_manipulation_call ::= array_method_name { attribute_instance } [ ( list_of_arguments ) ]
[ with ( expression ) ]` — **the parens are optional** — so a bare member name already satisfies
`callable_method_call_body`, and `!` fired on *every* identifier. The `*` loop ran **zero
iterations for its entire life**, and a rule written to accept `a.b[0].c[1].m()` accepted only
`a.m()`. It rejected `a.b[0].m()` — including, on the same code path, IEEE 1364-2005 §12.4's
indexed hierarchical task enable `top.u1[0].t;`, i.e. plain Verilog.

## Why no instrument saw it

Because the two hypotheses are observationally identical:

| | the guard is dead | the loop is not needed on this input |
|---|---|---|
| rule parses | yes | yes |
| branch selected | yes | yes |
| `--lint-grammar` | quiet | quiet |
| loop iterations | 0 | 0 |

Nothing distinguishes them from the outside, and "0 iterations" is the *expected* reading of the
second. A coverage report over rules cannot help: the rule is covered. Only a probe of the guard's
**subject**, in isolation, separates them.

## The discriminator: probe the subject, don't read the guard

Ask the decidable question — *does `R` accept the token this position expects?* — and answer it by
**parsing**, with the subject as the start symbol:

```bash
printf 'x' > rust/target/probe.txt
./rust/target/debug/ast_pipeline grammars/<g>.ebnf --interpret-parse rust/target/probe.txt \
    --interpret-entry-rule callable_method_call_body
# accepted=true  ->  the guard is vacuously false wherever an identifier is legal
```

Reading the grammar cannot answer this: the subject's nullability lives several rules away, behind
an optional group in a production the standard wrote three clauses later. Reading is how the guard
came to be written wrong in the first place.

## Sweep the class, not the instance

The set of `!R` sites in a grammar is small, enumerable and cheap to probe, so there is no excuse
for fixing one. The SV grammar had **8** guard subjects and **5** inline `!( … )` group forms;
probing each against a bare identifier found exactly one dead guard, and the class closed in a
single pass. Ship it as a re-runnable instrument with controls, not as a paragraph — and prove it
can go RED by pointing it at the pre-fix revision.

⛔ **Honest bound, and it is real**: this decides one shape. A guard can also be dead because its
subject accepts some *other* token that always occupies the guarded position, and inline `!( … )`
groups have no rule name to enter. Print those in full rather than summarising them, so the sweep's
blind spot is visible in its own output.

## The root cause behind the workaround

Ask why the guard existed at all. In PGEN it exists because `*` and `+` are **possessive** — a
quantifier never gives back an iteration, so an author who wants a loop to leave a tail must
re-state, in a lookahead, what the rest of the sequence needs ([[a-hypothesis-list-is-a-snapshot-of-what-you-knew-that-day]]
for why that re-statement rots). Four such lookaheads had been written independently into one
grammar. When you find yourself writing the fifth, the finding is the engine contract, not the
rule — route it there ([[a-cap-that-preserves-the-count-still-destroys-the-diagnosis]] is the same
shape one layer up: the instrument, not the subject).
