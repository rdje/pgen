---
name: feedback-lockstep-means-fully-documented-not-mentioned
description: STANDING DIRECTIVE (director, 2026-08-30 — the SECOND statement of this rule, which is the point) — "Whenever I ask for the codebase to be in sync, in lockstep with the top level mdbooks, I really mean that important topics, concept shall be full documented in the proper mdbook." LOCKSTEP IS A CONTENT BAR, NOT A REFERENTIAL ONE: a mention, a name in a list, a link, or a COUNT that matches is not documentation. Two axes bind — FULLY (the depth an ordinary reader needs to understand and use the thing) and PROPER (which of the eleven books it belongs in). The prior record already carried the same bar and was still not met, so the gap is mechanism, not willingness.
metadata:
  node_type: memory
  type: feedback
  director_directive: true
  created: 2026-08-30
---

**The directive, verbatim (director, 2026-08-30):** *"Whenever I ask for the codebase to be in sync,
in lockstep with the top level mdbooks, I really mean that important topics, concept shall be full
documented in the proper mdbook."*

⛔⛔ **IT IS THE SECOND STATEMENT OF THIS RULE, AND THAT IS THE MOST IMPORTANT FACT IN THIS RECORD.**
[[feedback_ast_pipeline_components_documented_in_top_book]] (director, 2026-07-05, *"non-negotiable,
no exception"*) already said **"thoroughly documented … not just an incidental mention"**, already
defined *thorough* as *what it is, where it sits in the pipeline, how it works, how to invoke it,
worked examples, and its trust/verification story*, and already declared book drift a **tracked
correctness defect**. The rule was recorded, indexed and retrievable — and on 2026-08-30 the director
still had to ask whether the 33 enforced doctrines were documented anywhere, because **14 of them
were absent from `docs/book/src` entirely**. ⇒ **the failure is not willingness and restating the
rule a third time will not fix it.** What was missing is a mechanism that can tell *documented* from
*mentioned*.

## The debt, measured — because *"a long time ago"* is a claim and it is TRUE

Director, same exchange: *"Which mean the 33 doctrines shall have part of the proper mdbook already
and a long time ago."* Correct. Each absent doctrine's registration date, against 2026-08-30:

| enforced since | doctrines | undocumented |
|---|---|---:|
| 2026-06-22 | `DIAG-SEVERITY+DOCPATH`, `KNOWLEDGE-MAP`, `TASK-ACCEPTANCE` | **69 days** |
| 2026-07-19 | `DESTRUCTIVE-TARGET-GUARD` | 42 days |
| 2026-07-26 | `DESIGN-PRIOR-ART` | 35 days |
| 2026-07-28 | `WAIVER-ROUTING` | 33 days |
| 2026-07-29 | `ROUTING-EVIDENCE` | 32 days |
| 2026-08-01 | `LESSON-PROMOTION` | 29 days |
| 2026-08-19 | `SV-RULE-FIRE-PARTITION` | 11 days |
| 2026-08-21 | `PARSER-BOOK-CURRENCY` | 9 days |
| 2026-08-26 | `SAMPLE-PROFILE-PARITY`, `LR-ELIMINATION-RATCHET`, `GAP-CLAIM-CENSUS` | 4 days |
| 2026-08-30 | `CORPUS-CENSUS-CURRENCY` | 0 days |

**14 of 14 dated · median 32 days · max 69 days · 410 doctrine-days undocumented.**

⛔⛔ **TWO THINGS THAT TABLE SAYS AND AN APOLOGY DOES NOT.** (1) **2026-06-22 is the day the doctrine
framework itself was created** — the director's *"provable, trackable … rule enforcers that literally
force the rule"* directive. So three doctrines have been enforced-and-undocumented since the
framework's founding commit: the debt is not drift, it is an **origin defect**. (2) The 2026-07-05
lockstep directive landed **13 days after** those three were registered and **did not catch them** —
a directive with no mechanism passed straight over a live instance of exactly what it forbade. That
is the evidence for the claim above that restating the rule is not the fix.

## What it means (how to apply)

- **LOCKSTEP IS A CONTENT BAR.** "In sync" is satisfied by a reader being able to understand and use
  the thing from the book alone. It is **not** satisfied by: a name in a list, a link to a script, a
  task-leaf citation, a changelog line, or **a count that matches**.
- ⛔ **A MATCHING NUMBER IS THE MOST DANGEROUS FORM, because it looks like coverage and is gate-held.**
  The founding instance: `check_doctrines.sh`'s `<meta:book-count>` holds the book's published
  doctrine count EQUAL to the registry size and read **33** on every run of the session in which this
  directive was given, while the book named at most 16 of them and described fewer. A doctrine could
  be registered, enforced on every commit, mirrored TWO-SIDED into `DOCTRINE_ENFORCEMENT.md` §10, and
  never appear in the public book, with every meta-check green. **Holding an integer is not holding
  a population.**
- **"PROPER" IS THE SECOND AXIS, AND IT IS ROUTING.** This repository publishes ELEVEN books — the
  platform book plus one per parser family ([[project_every_parser_per_parser_book]]). A concept is
  documented in the book whose readers need it: platform-wide and parser-agnostic material (the AST
  pipeline stages, the gate flow, the enforced doctrines, the closure model) in `docs/book/`; a
  family's own AST shape, rules and integration surface in that family's book. Documenting a
  platform-wide concept only inside one family's book satisfies neither axis.
- **"IMPORTANT TOPICS, CONCEPTS" is broader than any one enumeration.** The 2026-07-05 record listed
  the AST-pipeline components; this one does not replace that list, it says the list was an INSTANCE.
  The population is every concept a reader must hold to understand or operate the platform — the
  enforced doctrines, the gate flow, the closure model, the annotation languages, the profile
  system, the memory/continuity architecture, and whatever the next structural capability turns out
  to be.
- **SAME-COMMIT, unchanged.** A capability's documentation lands in the commit that lands the
  capability ([[feedback_regex_book_live]] — the books are the director's only window). This record
  adds the *depth* bar to that *timing* bar; neither replaces the other.

## The mechanizable core (why this is not just an exhortation)

A directive that only a human can check is one that goes quiet, which is exactly what happened
between 2026-07-05 and 2026-08-30. The shape that works here is already proven twice in this
repository — `<meta:mirror>` holds `DOCTRINE_ENFORCEMENT.md` §10 two-sided against the registry, and
`FAMILY-STATUS-CURRENCY` holds eleven published blocks equal to a fresh derivation:

1. **A CLOSED, TWO-SIDED POPULATION.** Every member of the concept register has a book section, and
   every such section names a register member. A member with no section FAILS; a section naming no
   member FAILS. This is what a count cannot do.
2. **DERIVE WHAT CAN BE DERIVED.** Where the source of truth already carries the prose — the doctrine
   registry carries a full paragraph per doctrine — the book section is a RENDERING of it with a
   `--check` that re-derives and diffs, so it cannot rot. Hand-written where derivation is
   impossible, gate-held where it is not.
3. **DEPTH IS ADJUDICATED, NOT MEASURED.** No gate can rule that prose is *thorough*. What a gate CAN
   hold is presence, population closure and currency; the depth bar stays a review judgement, stated
   as an honest limit rather than pretended away.

## Where owned

`DOCTRINE-GAP-OWNERSHIP.20` (the doctrines, director-ordered 2026-08-30, design decided) and
`DOCTRINE-GAP-OWNERSHIP.21` (the census: which other important concepts are enforced or shipped but
not fully documented, and in which book each belongs). Reinforces
[[feedback_ast_pipeline_components_documented_in_top_book]], [[feedback_regex_book_live]] and
[[project_every_parser_per_parser_book]].
