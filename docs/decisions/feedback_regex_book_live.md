<!-- Decision record (layer C) — migrated 2026-06-02 from harness-home memory
     (~/.claude/projects/.../memory/feedback_regex_book_live.md) by MEMORY-ARCH.2 (PGEN-MEMORY-ARCH-0003).
     Now the tracked system of record; the ~/.claude copy is a cache. Content preserved verbatim below. -->

---
name: The books (top-level docs/book/ AND every per-parser book) are the user's ONLY window — keep them in lockstep with the codebase, non-negotiable
description: The user does NOT read the codebase. The mdBooks — BOTH the top-level docs/book/ platform book AND every per-parser book (regex/systemverilog/svpp/vhdl/rtl_*) — are their sole window into what PGEN does. Following ANY task/slice/lane, if a user of the project could be impacted by the behavior change, the SAME commit must update the relevant book(s), thoroughly and with examples. No drift. Non-negotiable.
type: feedback
originSessionId: f74f4acc-7183-408a-ae5d-dcdce15a103c
---

## 2026-05-16 — STRENGTHENED + SCOPE WIDENED (user, this session)

> "The book is the user facing surface to the project so it absolutely [has] to be in sync with the codebase at any time, no drift, no compromise, non negotiable. Every feature shall be thoroughly and accurately documented with load of examples... I do not and won't look into the codebase. So my only window into the exact behavior of project is the book. So please keep the book up to date... following any task/slice/lane update the book if there is any way the user using the project might be impacted by the change in behavior."
> "I mean the **top level book** and the **parser specific books**."

**Binding rules:**
- Scope is BOTH surfaces: `docs/book/` (top-level platform mastery book — gate `make -C rust SHELL=/bin/bash mdbook_docs_gate`) AND every `docs/<parser>_parser_book/` (per-parser book — gate `make -C rust <parser>_parser_book_gate`). "Keep the book in sync" = whichever of these a change touches.
- Trigger is broader than "shape-affecting": it is **any change a project user could observe** — AST shape, accept set, public API/entry points, CLI/probe behavior, gates/workflow surfaced to users, schema bumps, new/changed features, known-defect status. If a user could be impacted, the owning book chapter(s) move in the SAME commit, with concrete examples.
- The user reviews the project ONLY through the books. A book that is stale, vague, or example-poor = the user cannot see/trust the project. Treat book accuracy as a release-blocking, signoff-grade property equal to code correctness.
- Book-vs-codebase drift is itself a tracked correctness defect (e.g. DOC-ENVELOPE-0001: fabricated `AstDumpPayload` in per-parser ast-envelope chapters) — fix with the same urgency as a parser bug; never paper over.
- Don't wait to be asked. Every user-impacting slice → book sync (+ contract + ledger + CHANGES/DEVELOPMENT_NOTES/LIVE) is the default.

---
The per-parser mdBooks under `docs/<parser>_parser_book/` (currently `regex_parser_book` and `systemverilog_parser_book`; future: `vhdl_parser_book`, `rtl_*_parser_book`, etc.) are **live deliverables of utmost importance for downstream consumers**.

User has stated this multiple times. Most recently 2026-05-05: *"the parsers' specific mdbook containing all the gory details about the parser in question are of utmost importance for downstream consumers to create/update their adapters... So parsers' mdbook are very, very important."*

**Why they matter:**
- They are the **primary integration reference** for downstream walkers/adapters (RGX consumes regex; Nexsim will consume systemverilog; future consumers will consume vhdl, rtl_*).
- They document the **exact AST shape** every annotated rule emits — without this, downstream adapter authors must reverse-engineer shapes from probe dumps.
- The integration contract document gives the high-level Highlights; the **mdBook gives the per-rule, per-example gory detail** consumers actually walk against.
- A drift between the book and the actual parser output means downstream walkers break silently or chase phantom bugs.

**Default behaviour (no reminders needed):**

For ANY change that touches:
- `grammars/<parser>.ebnf` (annotation, rule structure, accept set)
- `rust/src/ast_pipeline/ast_return_transform.rs` (codegen affecting that parser's typed shape)
- `rust/test_data/ast_shape_contract/<parser>_v1.json` (manifest)
- The parser's integration contract version

…the SAME commit must update the per-parser mdBook. The standard checklist:

1. `docs/<parser>_parser_book/src/changelog-index.md` — new top-level entry for the slice/fix, RGX-style thorough explainer for major changes (TL;DR table, background/why-it-matters, fix code, post-fix matrices, consumer dispatch recipe, verification commands).
2. `docs/<parser>_parser_book/src/schema-versioning.md` — new row in the version timeline.
3. `docs/<parser>_parser_book/src/json-carrier.md` — annotated-rules table additions/edits.
4. `docs/<parser>_parser_book/src/rules-*.md` — per-rule chapter for any rule whose shape changed (with full annotation source, branch tables, consumer extraction recipes).
5. `docs/<parser>_parser_book/src/examples-*.md` — every example involving the changed shape (replace pre-fix raw-envelope shape with post-fix typed shape; explicit pre/post comparison if a regression).
6. `docs/<parser>_parser_book/src/walking-the-ast.md` — if walker recipes change.
7. `docs/<parser>_parser_book/src/rules-top-level.md` — status line summarising the slice.
8. Run `make <parser>_parser_book_gate` to rebuild HTML and assert chapter set + landing pages green.
9. Stage both source `.md` AND rendered `.html` (HTML is tracked under `docs/<parser>_parser_book-html/`).

**Also update on the same commit:**
- `docs/contracts/PGEN_<PARSER>_PARSER_INTEGRATION_CONTRACT.md` — Contract Identity bump + new Highlights section.
- `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md` — bug-ledger row when the change is a fix.
- `docs/book/src/parser-families.md` — top-level PGEN mdBook's "current public handoff" version line if this is a public handoff bump.
- `CHANGES.md` and `MEMORY.md` — at the project root.

**Symptoms that this rule was violated:**
- User asks "did you update the mdbook?" / "did you update the live docs?" / "the parsers' mdbook are very important" — means sync was missed.
- Book chapter says "X is un-annotated" but parser emits typed X.
- Book example shows pre-fix raw-envelope shape but parser emits post-fix typed shape.
- `expected_json_object_keys_present` test fails because manifest wasn't regenerated alongside grammar.
- A downstream consumer (RGX, Nexsim, etc.) reports walking against a stale book and gets surprises.

**For RGX-style downstream-handoff explanations:**

When a fix is a regression caught by the downstream consumer (like PGEN-RGX-0081 / 0082), the changelog-index entry should be **thorough enough that a fresh-context maintainer can land on it and understand**:
- What the bug was (with concrete pre-fix shape)
- Why it matters for the downstream consumer
- The fix (with full grammar diff)
- Post-fix shapes (matrix table)
- Consumer dispatch recipe (Rust example)
- Verification commands they can run on their own host

That's the same depth I delivered for the slice-1.1.74 (Optim #14/#15) explainer and the 1.1.75 (RGX 0081+0082) explainer. Use those as templates.

**Don't wait to be asked.** Every shape-affecting change → book sync (and contract + ledger + project-root docs) is the default, not an opt-in. The user shouldn't have to remind us.
