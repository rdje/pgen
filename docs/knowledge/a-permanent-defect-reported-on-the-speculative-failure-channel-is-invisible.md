---
id: a-permanent-defect-reported-on-the-speculative-failure-channel-is-invisible
title: A permanent authoring error reported through the speculative-failure channel is indistinguishable from routine backtracking — and survives being fixed elsewhere
answers:
  - "a grammar rule matches nothing and no error is printed — where do I look"
  - "why does a broken regex in a grammar fail silently at runtime"
  - "does PGEN support lookahead inside a regex terminal"
  - "my regex terminal with (?!...) never matches — is that a parser bug"
  - "how do I tell an unconditional grammar defect from an alternative that just did not match"
  - "a defect was fixed in one family and is still live in another — how do I stop that"
  - "what severity should a compile failure inside a backtracking parser have"
  - "why did nobody notice a rule had been dead for months"
  - "should a static grammar error be reported at parse time or at lint time"
tags: [diagnostics, severity, backtracking, peg, regex, grammar-authoring, linting, evidence]
date: 2026-08-22
status: current
evidence: >
  GRAMMAR-WELLFORMED.H.17 (PGEN-GRAMMAR-WELLFORMED-0158). Rust's `regex` crate does not support
  look-around, so an EBNF terminal such as /((?:[^*]|\*(?!\/))*)/ fails to COMPILE on every
  invocation. The rule reports "Invalid regex pattern … look-around … not supported" at [PGEN][LOW]
  through the ordinary speculative-parse failure path, and the PEG engine backtracks past it exactly
  as past a branch that simply did not match — 0 occurrences at default verbosity. Consequence: the
  `ebnf` meta-grammar cannot parse ANY block comment, `/* x */` included, confirmed identically by the
  interpreter and the generated parser down to furthest_position. SystemVerilog was fixed for the SAME
  construct (SV-EXH-PROOF.3.3.4.b.6.2.15, where it was the dominant source of catastrophic
  backtracking behind a >180 s hang); the sweep was never done, and 6 uses across 3 grammars remained.
  `--lint-grammar` reports it 0 times.
reverify: "bash docs/tasks/artifacts/grammar_wellformed/regex_lookaround/probe.sh   # REGEX-LOOKAROUND: REPRODUCED — 4 arms incl. a live census"
---

**A backtracking parser's whole job is to swallow failures. Route a permanent defect through that
channel and you have built a perfect hiding place — one that survives the defect being found and
fixed in a sibling family.**

PGEN grammars can use a regex terminal. Rust's `regex` crate — deliberately, for linear-time
guarantees — does not support look-around. So this rule:

```ebnf
block_comment_content := /((?:[^*]|\*(?!\/))*)/
```

does not merely behave oddly. Its pattern **fails to compile, on every invocation, forever.** The
engine reports it:

```text
❌ Exiting rule 'block_comment_content' with error: ContextualError { message:
   "Invalid regex pattern '((?:[^*]|\*(?!\/))*)': regex parse error: … look-around … not supported"
```

…at `[PGEN][LOW]`, on the **speculative-parse failure path** — the same path a perfectly healthy
*"this alternative didn't match, try the next"* takes. At default verbosity it appears **zero** times.

## The two facts have different lifetimes, and the channel erases the difference

| | a branch that didn't match | a terminal that cannot compile |
|---|---|---|
| depends on the input | yes | **no** |
| will differ next run | yes | **no** |
| means the grammar is wrong | no | **yes** |
| how it is reported | rule-exit error, swallowed | *identical* |

An **input-dependent, expected, transient** event and a **static, unconditional, permanent** one share
one wire. Nothing downstream can separate them, so no amount of reading the output finds the second.

The consequence in this case: the EBNF meta-grammar — the grammar PGEN reads *every other grammar*
with — cannot parse any block comment at all. Not an exotic form; `/* x */` itself. Two independent
engines agreed down to `furthest_position`. The rules had been inert for as long as they had existed.

## The tell that it is this class, not a normal miss

**Run the rule in isolation against an input it must match.** A branch that "didn't match" matches
*something*, somewhere. A terminal that cannot compile matches **nothing on every input** — including
inputs that exercise none of the suspicious syntax:

```
block_comment_content  on 'abc'  -> accepted=false furthest_position=0
block_comment_content  on ' x '  -> accepted=false furthest_position=0
block_comment_content  on '*'    -> accepted=false furthest_position=0
```

`furthest_position=0` on an input with no `*` in it is not a matching failure. It is a rule that never
ran. **Then bisect the construct, controls first**, so "the lookahead is the cause" is measured rather
than assumed — `([^*]*)`, `((?:[^*])*)`, `((?:[^*]|x)*)` and `(a(?:b)?)` all accept; every arm
carrying `(?!…)` or `(?=…)` rejects, including `(a(?!b))` on the input `a`, where the assertion is
trivially satisfied.

## Why it recurred after being fixed

The same construct had already been found and repaired in another family, with the fix and its
rationale written into that grammar as a comment. It had cost a **>180 s hang** there: every failed
compile forced a backtrack, and the retries concentrated — 96 invocations at one byte offset in a
five-second window — making it the dominant source of catastrophic backtracking. So this class is a
**speed** defect as much as a correctness one.

None of that helped the other grammars, because the fix landed as *an edit*, not as *a check*. Six
uses across three grammars survived it.

⭐ **A defect class that has already been fixed once and is still live elsewhere is telling you the
fix was in the wrong layer.** The durable move is to make the property *static and checked*: "this
regex terminal compiles" is provable from the grammar text alone — no input, no parse, no generated
parser — which puts it exactly where a linter's `undefined-reference` and `non-terminating` errors
already live. Fixing the instances without adding the check buys one cycle.

## The rule

- **Never report a permanent, static fact on a channel designed to swallow transient ones.** If the
  condition cannot change between runs, it is not a parse event.
- **Prefer the earliest layer that can prove it.** Anything decidable from the source belongs in the
  static check, where it is reported once and loudly, not at parse time where it is reported
  constantly and inaudibly.
- **When you fix an instance, census the class in the same commit.** The census is cheap; the second
  discovery is not.

Related: [[a-refusal-message-that-names-a-cause-instead-of-its-condition-oversizes-the-gap]] —
the same family of failure, where the *message* rather than the *channel* hid the size of the gap.
