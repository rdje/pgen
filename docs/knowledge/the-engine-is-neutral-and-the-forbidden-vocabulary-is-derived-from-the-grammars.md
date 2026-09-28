---
id: the-engine-is-neutral-and-the-forbidden-vocabulary-is-derived-from-the-grammars
title: The engine is neutral, and the words it may not know are DERIVED from the grammars on every run — a typed list would be one more thing to keep in lockstep
answers:
  - "how does PGEN guarantee the engine is EBNF / grammar / parser neutral and agnostic"
  - "what does ENGINE-NEUTRALITY refuse and what does it allow"
  - "why is quoted_string not a leak but sv_2017 in a help string is"
  - "where may a language-specific component live and what may it name"
  - "what is the responsibility boundary between the engine and a grammar"
tags: [engine, neutrality, doctrine, responsibility-boundary, derived-vocabulary, ratchet]
date: 2026-09-28
status: current
evidence: "ENGINE-UNIVERSAL-SERVICES.4 (PGEN-ENGINE-UNIVERSAL-SERVICES-0164): the census on 7ac640e94 read the engine's behaviour neutral and its example text not — six language tokens in three universal help / lint strings, fixed; the embedding API 37 hand-typed family / profile tokens (.112) and the SystemVerilog preprocessor inside the engine crate (.113), ratcheted; ten self-test arms; the gate GREEN on the tree after the fixes."
reverify: "bash scripts/check_engine_neutrality.sh --self-test && bash scripts/check_engine_neutrality.sh --census | head -20"
---

**A concern belongs to the engine iff every EBNF shares it; to the grammar iff it is a fact about one
language. The engine therefore may not know a language's name, its dialects, its keywords or its rules —
and the list of what it may not know is not written down: it is derived from the grammars every time the
guard runs, because a typed list is a second copy of the grammars that would rot.**

## What the guard reads

- Every `rust/src` source not classed as a per-family component, the registry, the API, an operator binary or
  a harness is the engine, and in production code it carries none of: the language grammars' names, their
  `@profiles` values, their `kw_*` tokens, their multi-word rule names.
- Test modules and comments are exempt — fixtures and prose are not behaviour. That distinction is what keeps
  the guard true rather than loud: 1 240 test hits and 530 comment hits on the founding run were all fine.
- The engine's own meta-languages (the EBNF meta-grammar, the annotation languages, the example grammars)
  define words the engine uses by construction — `quoted_string`, `string_literal`, `regex`, `number` — and the
  guard subtracts them. The first draft did not, and flagged thirty false leaks; the register's `meta` class is
  the measured fix.

## What the guard ratchets

The non-engine classes are a closed population: a component may name its own family only; the registry, the
API, the operator binaries and the harnesses each carry a per-file ceiling that only falls, with the leaf that
owes its zero named on the row. The end state is every ceiling at zero — the API's enumeration derived from
the registry and each parser's declared profiles (`.112`), the preprocessor in its own crate behind a neutral
stage interface (`.113`).

## Why example text counts

The founding run found the engine's behaviour neutral and its example text not: `'["sv_2017", "sv_2023"]'` as
a help example teaches every reader that the engine knows one language. Text is behaviour a user sees.
Related: [[feedback-book-is-the-users-only-window-keep-lockstep]] (the same argument for the book).
