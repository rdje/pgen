---
name: project_sv_parser_elaborator_boundary
description: "DIRECTOR-CONFIRMED 2026-06-09 (AskUserQuestion): the SV parse phase = context-aware, SOUND name-categorization via the semantic store — REJECT a provably-undeclared type/nettype identifier at parse time, ACCEPT when it can't prove (unresolved `import pkg::*` in scope). Cross-unit binding + type/width/param/generate semantics = elaboration."
metadata:
  node_type: memory
  type: project
---

Director-confirmed 2026-06-09 (via AskUserQuestion, option "Context-aware, sound gating").
The parser/elaborator boundary for the PGEN SystemVerilog main parser.

**The unavoidable fact:** SystemVerilog is *not context-free* — `a b;`, `a (b);`,
`a #(b) c;` parse differently depending on whether `a` is a type / module / value (the
classic type-name-feedback / "lexer hack" problem, same as C/C++). So a correct SV parser
is *necessarily* context-aware; PGEN does this via the semantic store. A purely-syntactic
SV parser cannot be correct.

**The boundary (the decision):**
- **Parse phase (what PGEN is):** lexical + syntactic recognition → AST; plus *sound,
  single-translation-unit name categorization* (type / class / package / nettype …) via the
  store, using only facts establishable *within this parse* — the current TU plus any
  explicitly linked library facts (`--lib-in`). It commits to "is/isn't a type" **only when
  it can prove it**: it REJECTS a *provably*-undeclared type identifier at parse time, and
  ACCEPTS when it cannot prove (an unresolved `import pkg::*` is in scope) — the same
  *sound-not-complete* discipline the grammar linter uses ([[feedback_certifying_linter_trustworthiness]]).
- **Elaborate phase (later, not yet built):** cross-unit binding (unresolved external
  `import pkg::*`, hierarchy, libraries), type/width/parameter checking, generate unrolling
  — anything needing the *whole design* or value-level semantics.

**Consequence:** `endmodulemodule b;` (no declaration, no `import ::*` escape in scope) is a
parser bug — the parser *can* prove the identifier undeclared, so it must reject. But
`import foo_pkg::*; foo_type b;` (external, unlinked) must be ACCEPTED at parse — the parser
cannot prove `foo_type` isn't from `foo_pkg`. This is why store-gating must be a *sound*
gate (`has_fact(type_name, $1) OR wildcard_import_open`), never a hard gate.

**Why:** it makes "is this a parser bug?" decidable and consistent — the existing
`checked_type_identifier` gating already implements this boundary for most type positions;
the gaps (e.g. the `net_declaration` nettype branch) are bugs against this boundary, not
design choices. **How to apply:** when an SV grammar rule categorizes a bare identifier,
gate it on the store *soundly* (consult facts, but don't reject what an in-scope wildcard
import could explain) — [[feedback_grammar_rules_must_consult_store]]. First instance:
[[SV-PARSE-STRICT]] (the `nt a;` nettype branch). Composes with
[[feedback_fix_parser_bugs_asap_highest_priority]].
