# Platform Overview

PGEN turns grammar work into a quality-engineering discipline rather than a one-off parser hack.

## Core Flow

The canonical PGEN shape is:

1. `grammars/*.ebnf`
2. raw grammar AST / JSON artifact
3. generated parser artifacts
4. generated or in-memory stimuli
5. deterministic proof through gates, coverage, replay, and contracts

The system is intentionally doctrine-driven:

- deliverable parsers are expected to be EBNF-backed,
- generated parsers return ASTs,
- return annotations shape those ASTs,
- semantic annotations steer parser-generation behavior,
- closure claims need executable proof, not narrative confidence.

## Bootstrap And Normal EBNF Flow

PGEN has one intentional exception to the "generated parsers everywhere" rule: EBNF itself.

To avoid a chicken-and-egg problem, `grammars/ebnf.ebnf` always needs a bootstrap-safe frontend so PGEN can parse the grammar that is used to regenerate the EBNF parser. Today that bootstrap-safe role is handled by `rust/src/ebnf_frontend.rs`. Over time that layer may shrink, evolve, or become thinner, but some bootstrap-safe path must continue to exist for `ebnf.ebnf` itself.

That creates two related long-term flows.

1. Bootstrap lane for EBNF itself:
   - `grammars/ebnf.ebnf`
   - bootstrap-safe EBNF frontend
   - raw grammar AST / JSON
   - parser code generation
   - `generated/ebnf.rs`
2. Normal lane for ordinary grammars:
   - `grammars/foolang.ebnf`
   - generated EBNF parser (`generated/ebnf.rs` or its maintained successor)
   - raw grammar AST / JSON
   - parser code generation
   - `generated/foolang_parser.rs`

The emitted parser name (`foolang.rs` versus `foolang_parser.rs`) is only a naming policy choice. The architectural point is the same either way: ordinary grammars should ultimately flow through the generated EBNF parser rather than through a permanently hand-written general frontend.

PGEN is still in a hybrid state today:

- `generated/ebnf.rs` already exists and already participates in verifier, backstop, and readiness roles,
- `rust/src/ebnf_frontend.rs` still owns the main Rust-native `.ebnf -> raw_ast` adapter path,
- and some bootstrap/testing seams still retain explicit fallback behavior.

The intended destination is not to keep that hybrid forever. The long-term doctrine is:

- keep a bootstrap-safe path for `grammars/ebnf.ebnf`,
- and let ordinary grammar files use the generated EBNF parser path.

## Current Product Identity

PGEN is not just a parser generator. It is a parser and stimuli platform with:

- parser generation,
- AST shaping,
- semantic steering,
- stimuli generation,
- coverage and gap analysis,
- differential replay,
- downstream integration contracts,
- release and maintenance policy.

## Canonical Entry Docs

The most important repo docs behind this chapter are:

- `README.md`
- `PGEN_USER_GUIDE.md`
- `docs/reference/PGEN_SOTA_IMPLEMENTATION_ROADMAP.md`
- `LIVE_ACHIEVEMENT_STATUS.md`

Use the book for the curated story. Use those files for the raw operational truth.
