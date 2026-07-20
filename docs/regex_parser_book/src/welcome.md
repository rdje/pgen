# Welcome

This book is the **canonical integration reference** for PGEN's `regex` parser. It is written for downstream consumers — most prominently RGX — who need to:

- Build the regex parser from a fresh PGEN clone with **zero friction**.
- Walk the parser's runtime AST output to extract regex semantics.
- Migrate consumer code across PGEN releases when the AST shape evolves.

If you are a downstream consumer, you should be able to read this book end-to-end and have everything you need to integrate the parser into your build, parse a regex source string, and traverse the resulting AST to produce whatever consumer-side representation you need (a compiled matcher, an evaluator, a pretty-printer, etc.).

## Book status: live

This is a **live book** — it is updated in lockstep with the regex parser. Every parser release that changes the AST shape lands together with the book updates that document the new shape, in the same commit window. Reading any commit's snapshot of this book gives you the AST description for that commit's parser. If you observe a shape that disagrees with the book, that's a documentation bug — please report.

## Implementation note: the regex parser is self-hosting

As of 2026-06-08 (REGEX-SELF-HOSTING), the regex parser is **self-hosting**: `grammars/regex.ebnf` is
expressed entirely with native EBNF terminals (literal strings, char literals, ordered-choice alternations,
the `builtin_any_char`/`builtin_ascii_char` matchers, and the `$text`/`$0` whole-match and `@transform`
annotations) — it contains **no `/.../` regex literals**. Consequently the generated `regex_parser.rs`
**does not use or even link Rust's `regex` crate** (a guard, `scripts/check_regex_self_hosting.sh`, enforces
this). This is purely an implementation property: it does **not** change the accepted language, the runtime
AST shape, the error codes, or any version in the contract — every conversion step was verified
byte-identical against the `pcre2test` oracle. Downstream consumers need not change anything; the note is
here because some integrators care that the regex parser carries no Rust-regex-engine dependency. (Every
*other* PGEN parser remains free to use Rust's regex engine; only the regex parser is held to this bar.)

## Parse-time performance (live note, updated 2026-07-13)

The regex parser is under an active, tracked **speed campaign** (`RGX-0078`): as of 2026-07-20 the
measured parse cost is a geomean of **≈1.71µs per pattern** on the 8-pattern RGX bench corpus
(release build, fat-LTO, mimalloc-class allocator, noise-floor-minimum statistic) — down **≈290×**
from ≈496µs at the campaign's activation. On the external PCRE2 corpus the **maximum** observed
parse is ≈484µs and the corpus geomean ≈1.10µs. Every speed lever lands under a
hard **byte-identical constraint**: the accepted language, verdicts, error codes, and the runtime
AST are bit-for-bit unchanged (proven per lever by the differential-equivalence,
certificate-coverage, and PCRE2-compile-oracle gates), so **performance work never moves the
AST-dump schema** — exactly the "pure performance optimizations" carve-out in the
schema-versioning chapter.

Integration guidance for consumers:

- Parse cost is **per pattern compile**. If your workload re-compiles recurring patterns, cache
  `pattern → AST` on your side — the AST is a plain value, safe to clone and reuse. (A PGEN-side
  persistent parse cache is on the campaign roadmap, `RGX-0078.7`.)
- The campaign's methodology, scoreboard, and honest ceiling analysis live in the top-level book
  chapter *Inside parser performance* (`docs/book/src/inside-parser-performance.md`); live status
  and steering in `docs/tasks/RGX-0078.md`.

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
