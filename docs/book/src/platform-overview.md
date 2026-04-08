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
