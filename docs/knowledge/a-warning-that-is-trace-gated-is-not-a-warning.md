---
id: a-warning-that-is-trace-gated-is-not-a-warning
title: A module can SHADOW `eprintln!` into a trace call — so a warning you wrote may print nothing until someone asks for debug output
answers:
  - "I added a loud warning and the tool ran completely silently — where did it go"
  - "why does my eprintln print only under PGEN_TRACE_VERBOSITY=debug"
  - "how do I make a policy warning visible when the module's logging is trace-gated"
  - "is it safe to assume eprintln writes to stderr inside ast_pipeline"
  - "I added an opt-in flag that changes generated output — what must it print, and how do I prove it printed"
  - "how do I test that a warning actually fired"
tags: [diagnostics, logging, instrument-honesty, macros, verification, opt-in-flags]
date: 2026-08-14
status: current
evidence: rust/src/ast_pipeline/mod.rs:513 (`macro_rules! eprintln` forwarding to `pgen_trace_debug!`, in scope for every submodule declared after it, including `pub mod indirect_lr_elimination;` at :6277); docs/tasks/ENGINE-UNIVERSAL-SERVICES.md leaf .17 slice 8 RESULT 5 (a default-verbosity run of `--indirect-lr-admit-guard-feasible` was SILENT while absorbing candidates the shipped criterion refuses; the same command under PGEN_TRACE_VERBOSITY=debug printed both the banner and the `✅ Absorbing` line)
reverify: "bash docs/tasks/artifacts/engine_universal_services/guard_parses/probe.sh   # rows `1A banner 0` / `1B banner 1` assert the warning is absent on the shipped path and present on the opt-in one, at DEFAULT verbosity"
---

**`eprintln!` is not a keyword.** A crate can shadow it with a `macro_rules!` that forwards to a
level-gated tracer, and every module declared after that definition inherits the shadow — including
modules in other files. In this repository `ast_pipeline/mod.rs:513` does exactly that:

```rust
macro_rules! eprintln { ($($arg:tt)*) => { crate::pgen_trace_debug!($($arg)*) }; }
```

so every bare `eprintln!` under `ast_pipeline` — in `indirect_lr_elimination.rs`,
`ast_based_generator.rs`, all of them — is a **debug-level trace call** that prints nothing unless
`PGEN_TRACE_VERBOSITY=debug` is set.

⛔ **The failure this produces is silent and flattering.** A new opt-in flag was added that widens a
grammar-rewrite policy — it absorbs candidates the shipped criterion refuses, and the parser it
generates is genuinely different. It was given a banner saying so. The first default-verbosity run
printed **nothing at all**: no banner, no `✅ Absorbing` line, no indication the policy had changed.
The author had written the warning, read it in the source, and would have shipped it.

⭐ **The fix is one fully-qualified path, and it is load-bearing:**

```rust
std::eprintln!("⚠️  … this is NOT the shipped policy …");   // bypasses the shadow
```

⭐⭐ **But do not "make it all loud" — the shadow is right for most of what it covers.** Draw the
line by what the message is FOR:

| kind of message | belongs where |
|---|---|
| per-item narration (*one line per rewritten rule*) | trace-gated — it is O(grammar) and only useful when you asked |
| a POLICY warning (*this run is not the shipped behaviour*) | always on stderr — it changes how every downstream artifact should be read |

A warning that only fires when you already asked for debug output is not a warning; it is a trace
line with an emoji.

⛔ **And prove it fired, in both directions.** The bank that measures the flag carries two structural
rows — the shipped run must NOT print the banner, the opt-in run MUST — because the flag's *verdict
columns are identical either way*, so nothing else in the bank could tell a dropped flag from a
working one. That is [[an-instrument-that-prints-an-unjudged-column-reports-a-defect-as-green]]
applied to a side effect rather than to a value.

⭐ **How it was found, and it generalizes:** by running the new flag and reading the output, not by
reviewing the diff. The line existed, was correct, and was in the right function. Nothing about
reading the source could have revealed that the macro resolving `eprintln!` was three thousand lines
away in another file. ⇒ **for any output you add, execute the command and look — "I wrote the print"
and "the print appears" are different claims.**

Sibling of [[a-report-must-be-computed-from-the-artifact-it-describes]] (a value that was never about
the artifact) and of [[a-recorded-failure-reason-is-not-a-readable-one]] (a message that exists but
nobody can reach). This one is a correct message that is never delivered — and the retrieval key is
different: *"why did nothing print?"*, not *"is this number right?"*.
