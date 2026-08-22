---
id: check-the-property-not-the-spelling-of-the-property
title: A defect found through one instance tempts you to check the instance's spelling — check the property, and check it the way the runtime does
answers:
  - "how should I write a lint for a construct that broke something"
  - "should my check look for the syntax that failed or for the failure itself"
  - "my linter rejects valid code that looks like the bad pattern — what went wrong"
  - "how do I stop a static check from being both unsound and incomplete"
  - "should a validation re-implement the runtime or call it"
  - "is it ok to land a lint that is red at HEAD on existing code"
  - "how do I know a new check will not break an existing gate"
  - "how do I get independent confirmation of a census for free"
  - "why did my grep-based census and my compile-based census agree, and does that matter"
tags: [linting, static-analysis, validation, checks, gates, regex, evidence, grammar-authoring]
date: 2026-08-22
status: current
evidence: >
  GRAMMAR-WELLFORMED.H.17.2 (PGEN-GRAMMAR-WELLFORMED-0159). The defect was regex look-around in an
  EBNF terminal, so the tempting lint is "reject a terminal containing (?! or (?=". Measured, that
  check is unsound — (?i), (?s:.) and (?:...) share the prefix, are valid, and are used across the
  shipped grammars — and incomplete, since (a)\1 also fails to compile and a look-around-shaped check
  never sees it. The class shipped as "does this terminal COMPILE", using the runtime's own wrapping
  \A(?:{pattern}); two-sided control: (?i)[a-z]+, (?s:.)*, [[:alpha:]]+ stay rc 0, while (a)\1 and
  a(?<=b) go rc 1. Bonus: compiling every terminal reproduced, from a disjoint code path, a census
  that had been produced by grepping grammar text — agreeing exactly across 12 grammars.
reverify: "bash docs/tasks/artifacts/grammar_wellformed/regex_lookaround/probe.sh   # arm 4: ebnf=3 semantic_annotation=2 systemverilog=0"
---

**You find a defect through one instance. The instance has a shape, and the shape is the thing you
just spent hours staring at — so the check you reach for matches the shape. That check is usually
wrong in both directions at once.**

The defect: an EBNF grammar's regex terminal used look-around, which Rust's `regex` crate does not
support, so the pattern never compiled and the rule matched nothing on any input, forever. The obvious
lint writes itself:

```
reject any regex terminal containing "(?!" or "(?="
```

Measured against the real grammars, that check is:

- **unsound** — `(?i)`, `(?s:.)`, `(?:…)` all share the `(?` prefix, are perfectly valid, and are used
  across the shipped grammars. The check condemns working rules.
- **incomplete** — `(a)\1` does not compile either. A look-around-shaped check never sees it.

The property that actually matters is not *"contains look-around"*. It is ***"this terminal
compiles"***. Check that, and both failures disappear at once — the valid constructs pass because
they compile, and backreferences are caught because they do not. No enumeration of bad syntax to keep
up to date, and no list to go stale when the regex engine gains or loses a feature.

## Check it the way the runtime does, not the way you remember it

The runtime did not compile the bare pattern. It compiled an anchored, non-capturing wrapper:

```rust
regex::Regex::new(&format!(r"\A(?:{pattern})"))
```

A check calling `Regex::new(pattern)` is a **re-implementation of the thing it is checking**, free to
disagree with the parser in either direction — a false green on a pattern valid only bare, a false red
on one valid only wrapped. It agrees with your *model* of the runtime rather than the runtime. Same
failure shape as a build-freshness check that re-implements the build tool's timestamp comparison:
it will confirm your belief and miss the bug.

Pin the wrapping with a test, so the check and the runtime cannot drift apart silently.

## Two things that come free, and one that must be measured

**A second instrument, for nothing.** The population had already been sized by grepping grammar text.
Compiling every terminal sizes it again, from a code path sharing nothing with the first — and across
twelve grammars the two agreed exactly. That is independent confirmation arriving as a *by-product of
the fix*, and here it machine-confirmed two judgements that had been made by hand: that one grammar's
look-around tokens were string *literals* describing the syntax rather than uses of it, and that an
earlier repair in another family had really taken.

**A check that fires on real defects at HEAD is the tool working, not a broken gate.** Two shipped
grammars started exiting non-zero. That is correct: they contain rules that can never match.

**But the blast radius must be measured before landing, not assumed.** Find every consumer that reads
the check's exit code and verify each by name. Here there was exactly one gate reading a
`--lint-grammar` exit status; its contract named a specific grammar file and an expected exit code of
zero; that grammar reads zero. One minute of work, and it is the whole difference between "should be
fine" and "is fine".

## The rule

- **Name the property, not the symptom.** "Does it compile" beats "does it contain the token that
  broke last time."
- **Call the real thing.** If the check exists to agree with a runtime, invoke the runtime's own
  formulation — never a paraphrase of it.
- **Prove both directions.** A check with no green control is a check that might be condemning
  everything; one with no red control might be condemning nothing.

Related: [[a-permanent-defect-reported-on-the-speculative-failure-channel-is-invisible]] — the defect
this check was built for, and why a parse-time report could never have surfaced it.
