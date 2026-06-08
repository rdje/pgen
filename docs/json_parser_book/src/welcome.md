# Welcome — PGEN JSON Parser Integration Reference

This is the per-parser integration reference book for PGEN's `json` parser, generated from
`grammars/json.ebnf`. It is the canonical AST-integration surface for anyone consuming the json parser's
output, and it sits alongside the platform mastery book (`docs/book/`) and the other per-parser books
(regex, SystemVerilog, VHDL, rtl_frontend, rtl_const_expr), all linked from the platform book's
[Parser Families](../../book/src/parser-families.md) chapter.

## ⚠️ Read this first: `json` is a deliberately *simplified* grammar

Unlike the shipped parser families (regex, SV, VHDL), the `json` grammar is a **small built-in** used for
examples, onboarding, and cross-family stimuli work. **It is a deliberately simplified subset of JSON — it
is *not* a conforming RFC 8259 / ECMA-404 parser.** PGEN states this openly and backs it with evidence:

- **EBNF-internal quality is clean.** `json` is the first PGEN grammar to report `fully_certified=true`
  from the certificate-coverage gate (every rule is witnessed-reachable, zero `UNKNOWN`). See the platform
  book's [Grammar Well-Formedness](../../book/src/grammar-wellformedness.md) chapter.
- **Real-world conformance is characterized, not claimed.** Measured against the recognized
  [JSONTestSuite](https://github.com/nst/JSONTestSuite) corpus (vendored under `json_corpus_bundle/`), the
  simplified grammar accepts **81/95** must-accept files and rejects **158/188** must-reject files, and
  **3 deep-nesting files crash** the parser. The gaps (no number exponents, no string escapes, leading
  zeros allowed, loose trailing/whitespace, no recursion guard) are documented honestly in
  [External-Corpus Characterization](external-corpus-characterization.md).

So: trust this parser for the **common JSON core** in examples and tooling; do **not** treat it as a
standards-grade JSON reader until the planned `EXTERNAL-CORPUS.2a` upgrade toward RFC 8259 lands.

## What this book covers

- [Build Recipe](build-recipe.md) — how to generate and exercise the json parser.
- [Grammar and Scope](grammar-and-scope.md) — exactly what `json.ebnf` accepts and what it does not.
- [AST Envelope Structure](ast-envelope.md) — the shape of the parse tree this parser returns.
- [External-Corpus Characterization](external-corpus-characterization.md) — the measured gap vs the standard.
- [Glossary](glossary.md).
