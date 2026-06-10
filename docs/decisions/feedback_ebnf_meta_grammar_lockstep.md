# feedback_ebnf_meta_grammar_lockstep

- **Category:** feedback (standing doctrine — director, 2026-06-10, emphatic)
- **Status:** binding, non-negotiable

## Context

While fixing the vhdl based-literal parser bug (`PGEN-GRAMMAR-WELLFORMED-0067`),
the strict `ebnf_frontend_dual_run_gate` turned out to be red because the
GENERATED ebnf parser cannot parse the `**` flatten-spread that `regex.ebnf`
has used since RGX-0074. The follow-up completeness audit
(`PGEN-GRAMMAR-WELLFORMED-0068`, recorded in `docs/tasks/GRAMMAR-WELLFORMED.md`
Decisions) found **six** such gaps: per-branch return annotations (the dominant
one — 5 of 6 failing grammars stop there), `::N*` extraction-spread, `**`
flatten-spread, the `[> …]`/`[>! …]` lexical annotations, and dotted + indexed
`$refs` in return annotations. The pattern behind all six is the same: a new
EBNF-format feature was added (frontend + codegen + docs) while working on some
grammar, but the feature was never ported into `grammars/ebnf.ebnf`, the
meta-grammar that formally defines the EBNF language itself.

## Decision (the doctrine)

> **Every time a new feature or construct is added to the EBNF format while
> working on any `XYZ.ebnf`, that feature/construct MUST be ported to
> `grammars/ebnf.ebnf` systematically, in the same wave. Failing to do that is
> unacceptable.** (Director, 2026-06-10.)

Concretely, a slice that extends the EBNF language surface (new syntax, new
annotation form, new directive payload shape, new reference syntax in
annotations) is **not complete** until `grammars/ebnf.ebnf` parses the new
construct — the meta-grammar is part of the feature's definition of done, the
same way the books and contracts are.

## Why

- `grammars/ebnf.ebnf` is the formal, self-hosting definition of the EBNF
  language (the EBNF-source-of-truth invariant applied to the meta level).
  When it lags, the generated ebnf parser silently stops being able to read
  the project's own grammars, and the self-hosting doctrine (handwritten
  frontend = bootstrap scaffolding only) rots invisibly.
- The gap class is silent: production compilation flows through the
  hand-written frontend, so nothing user-facing breaks when the meta-grammar
  lags — only the dual-run differential can see it, and only if it is run.

## How to apply

1. Any slice adding an EBNF-format feature also edits `grammars/ebnf.ebnf`
   (and regenerates `generated/ebnf.rs`) in the same wave, with a minimal
   probe grammar exercising the new construct through `ebnf_dual_run_diff`
   (`parse_full` must pass).
2. **Mechanical enforcement (the real backstop):** once the catch-up leaf
   ports the six audited gaps and `ebnf_frontend_dual_run_gate` goes
   strict-green, that gate must join the standard proof path (local + CI).
   From then on, any unported construct used by a shipped grammar turns the
   gate red automatically — the doctrine stops depending on memory.
3. Until the catch-up leaf lands, treat the audit table in
   `docs/tasks/GRAMMAR-WELLFORMED.md` (Decisions, 2026-06-10) as the known
   debt list; do not add to it.

## Consequences

- The six-gap catch-up port is owned work (ticketed under GRAMMAR-WELLFORMED;
  promotable to its own tree when implementation starts).
- `ebnf_dual_run_diff` minimal-probe testing becomes part of the acceptance
  checklist for every EBNF-language-surface slice.
