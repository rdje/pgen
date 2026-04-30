# Welcome

This book is the **canonical integration reference** for PGEN's `regex` parser. It is written for downstream consumers — most prominently RGX — who need to:

- Build the regex parser from a fresh PGEN clone with **zero friction**.
- Walk the parser's runtime AST output to extract regex semantics.
- Migrate consumer code across PGEN releases when the AST shape evolves.

If you are a downstream consumer, you should be able to read this book end-to-end and have everything you need to integrate the parser into your build, parse a regex source string, and traverse the resulting AST to produce whatever consumer-side representation you need (a compiled matcher, an evaluator, a pretty-printer, etc.).

## Book status: live

This is a **live book** — it is updated in lockstep with the regex parser. Every parser release that changes the AST shape lands together with the book updates that document the new shape, in the same commit window. Reading any commit's snapshot of this book gives you the AST description for that commit's parser. If you observe a shape that disagrees with the book, that's a documentation bug — please report.

## What this book is

- The **single source of truth** for the regex parser's runtime AST shape, by rule and by example.
- A **working integration recipe** for cold-clone builds and incremental rebuilds.
- A **migration log** for consumers who depended on earlier shapes that have since evolved.

## What this book is not

- It is **not** a regex language tutorial. PCRE2 syntax and semantics are documented upstream.
- It is **not** a PGEN platform overview. For that, see the parent PGEN mdBook under `docs/book/`.
- It is **not** the `regex.ebnf` grammar source. That lives at `grammars/regex.ebnf` and is the formal specification of what the parser accepts.

## Companion documents

| Document | What it covers |
|---|---|
| `docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md` | Versioned contract — release identity, stable API surface, schema version, support boundary. **Read this if you need the formal contract.** |
| `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md` | Per-release bug history. Read when investigating a behavior change. |
| `grammars/regex.ebnf` | The grammar source. Authoritative for what input the parser accepts. |
| `grammars/return_annotation.ebnf` | The annotation language used to shape the AST output. Affects what the parser emits. |

The contract document carries the legal-style versioned guarantees; this book carries the practical examples and walk-throughs. If they ever disagree, **the contract wins** for compliance — but if the book disagrees with the contract, that's a documentation bug; please report it.

## Reading order

If this is your first read, follow the chapters in order. The mdBook navigation on the left is in the order we recommend.

If you have a specific need:

- **"I need to build the parser right now."** → [Quickstart](quickstart.md).
- **"I need to know what the AST looks like."** → [AST Envelope Structure](ast-envelope.md), then [Walking the AST](walking-the-ast.md).
- **"I'm migrating from the old recursive-envelope shape."** → [From the Recursive Envelope](migration-from-recursive-envelope.md).
- **"I need the exact shape of rule X."** → Per-Rule Shape Reference chapters.
- **"What changed in this release?"** → [Changelog Index](changelog-index.md).
