---
id: a-mis-cited-production-reproduces-as-a-success
title: A mis-cited production is more dangerous than a missing one — it reproduces as a SUCCESS, so the test that should catch it passes
answers:
  - "why did a grammar fix pass its test and still be wrong"
  - "how can a rule be wrong in two opposite directions at once"
  - "should I check the clause number when a grammar comment quotes an LRM production"
  - "why does PGEN accept '{} but reject {}"
  - "what is an empty unpacked array concatenation in SystemVerilog"
  - "how do I tell over-acceptance from under-acceptance in the same rule"
tags: [grammar-authoring, lrm-fidelity, systemverilog, verification, citations, strictness]
date: 2026-08-10
status: current
evidence: docs/tasks/SV-CORPUS-GRAD.md leaf .3.26 (repro, root cause, the 5-row corpus delta, the acceptance checklist); grammars/systemverilog.ebnf rule empty_unpacked_array_concatenation (the two annotated arms + the corrected A.8.1 citation); docs/systemverilog/2017/md/section-41-data-read-api.md:2907 (A.8.1), :3551 (footnote 35), :2292-:2295 (all four assignment_pattern alternatives non-empty); the superseded fix SV-EXH-PROOF.3.3.4.b.6.2.37.8
reverify: "printf 'module m;\\n int q[$];\\n initial q = {};\\nendmodule\\n' > /tmp/e.sv && ./rust/target/release/parseability_probe --parse systemverilog /tmp/e.sv --profile sv_2017"
---

A fix that quotes a standard's production has two independent things that can be wrong: the
**transcription** and the **citation**. Tests check the first. Nothing checks the second — and the
second failing is the worse case, because it produces a passing test.

## The worked instance

`SV-EXH-PROOF.3.3.4.b.6.2.37.8` found `empty_unpacked_array_concatenation := lbrace epsilon rbrace`
— genuinely broken, since `epsilon` is not a rule in this grammar, so the production could never
match anything. It replaced the body with `tick lbrace rbrace`, citing *"§A.6.7"*.

The production is at **A.8.1** and reads:

```ebnf
empty_unpacked_array_concatenation35 ::= { }
```

bare braces, **no apostrophe** — with footnote 35 (*"{ } shall denote an empty unpacked array
concatenation … and shall not be used in any other form of concatenation"*) and §7.10's prose
(*"The empty queue can be denoted by an empty unpacked array concatenation {}"*) both saying the
same thing. Meanwhile `'{}` has **no derivation anywhere in Annex A**: all four
`assignment_pattern` alternatives require at least one `expression`.

So one rule was wrong in **both directions at once** — under-accepting the LRM form, over-accepting
a form the standard cannot derive.

⭐ **Why it survived.** The motivating input was uvm's `return '{};`, which genuinely uses the
apostrophe. The fix was tested against it, it passed, and **a passing test on a wrong literal is
indistinguishable from a passing test on a right one.** A *missing* fix announces itself the next
time the corpus runs; a *mis-cited* fix banks a green result and goes quiet. It took the corpus
moving two unrelated rows' `furthest_position` deeper onto the construct to surface it, two campaigns
later.

## The habit

> **When a fix quotes a production, quote the clause number too — and open the clause.** The
> citation is the one part of the change no test can verify for you.

Corollaries that paid off here:

- **Check the neighbours.** Confirming `{ }` at A.8.1 was not enough; the claim *"`'{}` is not
  derivable"* required reading all four `assignment_pattern` alternatives to see that none is empty.
  A production's absence is proved by enumerating what is present.
- **The annex keeps half its rules in footnotes** — see
  [[annex-a-footnotes-license-derivations-the-productions-cannot-derive]] for the sibling case where
  a footnote *licenses* a derivation the productions cannot express. Here footnote 35 *confirms* one.
- **A one-rule fix whose corpus delta is entirely its own construct is the shape to aim for.** This
  one moved exactly 5 rows, all `unexplained_rejects_valid → match`, from four independent suites,
  and none the other way.

## When both directions are real, they are not one decision

Fixing under-acceptance is unambiguous. Removing over-acceptance is not, and the two must be
adjudicated separately rather than "cleaned up" together. Here the tolerated arm stayed, because two
standing rulings partition the problem: the parser is strict-LRM **by default** with a tolerance
switch **deferred, not rejected**, *and* a parser failure on UVM is the parser's defect — and UVM
writes `return '{};`. That makes the apostrophe arm **dialect tolerance the ecosystem relies on**,
i.e. exactly what the deferred switch exists to serve, rather than a bug to delete.

⛔ So keep it, **name it in the grammar**, and route it — never leave a deliberately tolerated arm
looking like a second production someone transcribed. The comment is the only thing distinguishing
"we accept this on purpose" from "we accept this by mistake", and the mistake is what this whole card
is about.
