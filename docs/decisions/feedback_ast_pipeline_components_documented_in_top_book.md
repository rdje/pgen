---
name: feedback-ast-pipeline-components-documented-in-top-book
description: Director directive (2026-07-05, non-negotiable) — EVERY structural component of the AST pipeline (grammar linter, parser generator, stimuli generator, and the forthcoming grammar-AST interpreter / compile-and-run harness / scratch-registration slot) SHALL be thoroughly documented in the TOP-LEVEL mdBook; and the codebase and the mdBooks SHALL be in sync/lockstep at all times, no drift, no exception.
metadata:
  node_type: memory
  type: feedback
  director_directive: true
  created: 2026-07-05
---

**Director directive (2026-07-05, two messages, non-negotiable).**

1. *"All the structural components or parts of the AST pipeline shall be thoroughly documented in the
   top-level mdBook, like the linter, parser generator, stimuli generator, this new general
   grammar-AST interpreter (when landed), the compile-and-run harness route (when landed) and the
   scratch-registration slot (when landed)."*
2. *"The codebase and the mdBooks shall be in sync, lockstep all the time, no drift allowed,
   non-negotiable, no exception."*

**What it means (how to apply).**
- The **top-level mdBook** (`docs/book/`) — the user's primary window into the platform — MUST carry a
  thorough treatment of **each structural component of the AST pipeline**, not just an incidental
  mention. The named components: the **grammar linter** (well-formedness/certifying linter), the
  **parser generator**, the **stimuli generator**, and — WHEN LANDED — the **general grammar-AST
  interpreter**, the **compile-and-run harness route**, and the **scratch-registration slot** (the
  three `PARSE-HARNESS` deliverables). Any future structural stage of the pipeline joins this list.
- "Thorough" = same depth as the existing structural-component chapters (the linter →
  `grammar-wellformedness.md`, the stimuli generator → `stimuli-and-quality.md`, the parser generator →
  `developer-architecture.md`): what it is, where it sits in the pipeline, how it works, how to invoke
  it, worked examples, and its trust/verification story.
- **Lockstep, no drift:** the codebase and the mdBooks are in sync AT ALL TIMES. A component's
  documentation lands **in the same commit** as the component (this reinforces the standing "books are
  the user's ONLY window" rule, [[feedback_regex_book_live]]). Book drift from the codebase is a
  **tracked correctness defect**, not a cosmetic nicety — non-negotiable, no exception.

**State when recorded (2026-07-05 assessment).** The top-level book already covers the existing
structural components (linter → `grammar-wellformedness.md`; stimuli generator →
`stimuli-and-quality.md` + others; parser generator → `developer-architecture.md`). The three new
harness components are **not yet landed** (0 book coverage — expected); their top-level-book sections
are a hard acceptance criterion of the `PARSE-HARNESS` tree (D5 + acceptance criterion #6 + leaf `.9`),
landed same-commit as each component.

**Enforcement / where owned.** `docs/tasks/PARSE-HARNESS.md` (D5, acceptance criterion #6, leaf `.9`)
owns the three new components' top-level-book sections. The general codebase↔book lockstep is the
standing `COMMIT.md` `docs/book/` sync rule + [[feedback_regex_book_live]]; this record makes the
"every structural AST-pipeline component, thoroughly, in the TOP-LEVEL book" requirement explicit and
durable. Reinforces [[feedback_books_are_user_only_window]] where present and
[[feedback_ast_pipeline_parser_agnostic]] (the components are parser-agnostic → documented once,
platform-wide).
