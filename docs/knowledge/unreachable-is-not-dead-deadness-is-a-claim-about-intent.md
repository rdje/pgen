---
id: unreachable-is-not-dead-deadness-is-a-claim-about-intent
title: "Unreachable" is not "dead" — reachability is a fact about the grammar, deadness is a claim about INTENT, and intent is answered by provenance, never by a reach analysis
answers:
  - "is this unreachable rule dead code"
  - "can I delete a rule nothing references"
  - "how do I tell a placeholder from debris"
  - "should I remove unreachable grammar rules"
  - "why is a rule unreachable but not a defect"
  - "how do I find out why a rule was written"
  - "what evidence proves code is dead"
  - "is an unused rule a defect"
tags: [grammars, certification, reachability, dead-code, provenance, judgement, doctrine]
date: 2026-08-23
status: current
evidence: |
  MEASURED 2026-08-23 (GRAMMAR-CERT-STATUS.3/.4). 31 rules across PGEN's two annotation grammars
  were reported UNKNOWN by `--report-certificate-coverage` — no reach path from the entry. The
  reachability analysis was rigorous and seed-stable (identical counts AND rule sets at seeds
  0/7/42). On that basis I recommended DELETING 22 of them, with what looked like strong support:

    - nothing routes to them (`annotation_value` offers exactly four value families);
    - nothing consumes them (0 references across docs/contracts/ and rust/src/);
    - the grammar's own documented spelling for those annotation kinds ALREADY PARSES.

  ⛔ EVERY ONE OF THOSE FACTS WAS TRUE, AND TOGETHER THEY DO NOT ESTABLISH DEADNESS. The director
  refused the deletion and asked whether they were future-feature placeholders. One command settled
  it, and it was a command about HISTORY, not about structure:

    $ git log --oneline --all --diff-filter=A -- grammars/semantic_annotation.ebnf
    a68cc773 feat: Complete bootstrap build system with file-based placeholder targets

  The file was CREATED with all 22 rules already in it — original design authored alongside the
  grammar, never wired — and the founding commit says "placeholder" in its own subject. The content
  agrees: a semver regex with prerelease AND build metadata, a time_unit ladder down to microseconds,
  a full memory_unit set. That is deliberate design work, not debris.

  ⭐ THE DISCRIMINATOR, and it is cheap:
      reachability  -> a fact about the artifact AS IT IS   (a reach analysis answers it)
      deadness      -> a claim about INTENT                  (only provenance and the author do)
  A reach analysis can tell you a rule cannot fire. It cannot tell you whether someone meant to wire
  it later. Those are different questions and only one of them licenses a delete.

  ⇒ BEFORE PROPOSING TO DELETE ANYTHING, RUN THE PROVENANCE PAIR:
      git log --diff-filter=A -- <file>      # was it born with the file, or added later?
      git log -S'<symbol>' -- <file>         # which commit introduced it, and what did it say?
  Born-with-the-file plus a design-shaped body is the signature of a placeholder. Added-later in a
  test-infrastructure commit is the signature of scaffolding. They deserve different verdicts.

  ⭐ THE SAFE MOVE WHEN INTENT IS UNCLEAR is to FLAG, not remove: a greppable marker in the source
  itself, carrying the provenance and the open question, costs nothing and survives the session. For
  a grammar, a comment flag is provably free — the frontend strips comments, so `--emit-raw-ast-json`
  is byte-identical and the generated parser cannot move (measured: 37,041 B and 6,610 B, identical).
reverify: "git log --oneline --all --diff-filter=A -- grammars/semantic_annotation.ebnf && grep -c 'UNWIRED-PENDING-REVIEW' grammars/semantic_annotation.ebnf grammars/return_annotation.ebnf"
---

## The trap in one line

A reach analysis is evidence about *the artifact*. A delete is a decision about *someone's intent*.
Do not let the first authorize the second.

## What "flag, don't delete" buys

The marker records what the analysis found **and** what it could not decide, at the place the next
editor will actually look. The rule keeps costing nothing — an unreachable rule is emitted with its
definition and a by-name dispatcher entry and no production call site, so it can never be invoked by
a parse — while the open question stops being re-derived from scratch every time someone runs the
certification report.

See [[which-pgen-grammars-are-certified-and-which-are-not]] for the status this analysis feeds, and
[[deleting-an-implementation-while-leaving-its-flag-fails-silently]] for the opposite failure — a
removal that *did* happen and hid itself.
