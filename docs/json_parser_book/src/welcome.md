# Welcome — PGEN JSON Parser Integration Reference

This is the per-parser integration reference book for PGEN's `json` parser, generated from
`grammars/json.ebnf`. It is the canonical AST-integration surface for anyone consuming the json parser's
output, and it sits alongside the platform mastery book (`docs/book/`) and the other per-parser books
(regex, SystemVerilog, VHDL, rtl_frontend, rtl_const_expr), all linked from the platform book's
[Parser Families](../../book/src/parser-families.md) chapter.

## Read this first: `json` is a built-in grammar, now RFC-8259-aligned

The `json` grammar is a **small built-in** used for examples, onboarding, and cross-family stimuli work.
It began as a deliberately *simplified subset* of JSON and was upgraded (`EXTERNAL-CORPUS.2a`) to track
RFC 8259 / ECMA-404 for its lexical/syntactic core, using a recognized external corpus as the acceptance
metric. PGEN backs the claim with evidence from **two** independent oracles:

- **EBNF-internal quality is clean.** `json` was the first PGEN grammar to report `fully_certified=true`
  from the certificate-coverage gate (every rule is witnessed-reachable, zero `UNKNOWN`). See the platform
  book's [Grammar Well-Formedness](../../book/src/grammar-wellformedness.md) chapter.
- **Real-world conformance is characterized, not claimed.** Measured against the recognized
  [JSONTestSuite](https://github.com/nst/JSONTestSuite) corpus (vendored under `json_corpus_bundle/`), the
  upgraded grammar accepts **95/95** must-accept files and rejects **181/188** must-reject files. Two
  residuals remain, each its own follow-up: **5 trailing-content/comment** files are still accepted (strict
  end-of-input, `EXTERNAL-CORPUS.2c`) and **3 deep-nesting files crash** (recursion/stack guard,
  `EXTERNAL-CORPUS.2b`). Details: [External-Corpus Characterization](external-corpus-characterization.md).

So: the json parser handles the **JSON lexical/syntactic standard** (numbers with exponents, escaped
strings with control-char rejection, leading-zero rejection, exact whitespace). It is not yet hardened
against trailing-garbage or adversarially-deep nesting — see the two residual leaves above.

## What this book covers

- [Build Recipe](build-recipe.md) — how to generate and exercise the json parser.
- [Grammar and Scope](grammar-and-scope.md) — exactly what `json.ebnf` accepts and what it does not.
- [AST Envelope Structure](ast-envelope.md) — the shape of the parse tree this parser returns.
- [External-Corpus Characterization](external-corpus-characterization.md) — the measured gap vs the standard.
- [Glossary](glossary.md).
