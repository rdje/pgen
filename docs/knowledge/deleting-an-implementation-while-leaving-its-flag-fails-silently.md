---
id: deleting-an-implementation-while-leaving-its-flag-fails-silently
title: Deleting an implementation while leaving its FLAG behind is worse than deleting the flag too — the surviving flag turns a removed capability into a silent no-op that exits 0
answers:
  - "why did my check stop catching anything"
  - "why does a gate pass on every input"
  - "how do I tell if a check can actually go red"
  - "what happens when a refactor deletes a feature but keeps the CLI flag"
  - "should I remove the flag when I remove the code behind it"
  - "how do I prove a check is not vacuous"
  - "why did a fix commit introduce a regression nobody noticed"
  - "what is a check that cannot go red"
tags: [gates, refactoring, cli, silent-failure, verification, red-control, doctrine]
date: 2026-08-23
status: current
evidence: |
  MEASURED at commit c06292f4 (GRAMMAR-CERT-STATUS.1b). `-0002` rewrote the rendering path of
  `scripts/report_grammar_certification.sh` and deleted its `--check` derive-and-diff block, but the
  ARGUMENT PARSER kept `--check`. The flag went on being accepted, set MODE=markdown, printed a fresh
  table to stdout and exited 0 without ever opening the page it was supposed to check.

  FOUR arms, all exit 0, all mutually indistinguishable:

    --check <the real published page>              exit 0   (correct, but for the wrong reason)
    --check <page with a corrupted DERIVED block>  exit 0   (should be 1)
    --check <page with no DERIVED block at all>    exit 0   (should REFUSE)
    --check /nonexistent/no/such/page.md           exit 0   (should REFUSE)

  ⛔ THE TELL IS THE LAST ARM. A check that returns 0 for a path that does not exist is not
  measuring anything at all. Run that arm on any gate you inherit.

  ⭐ WHY IT WAS SILENT, and the generalisation: had the flag been deleted alongside its code, the
  next invocation would have died on an unknown argument — loud, immediate, unmissable. The
  SURVIVING FLAG is what converted a deleted feature into a passing check. So the flag is not a
  harmless leftover: it is the thing that hides the deletion.

  ⛔ THREE SURFACES WENT ON PUBLISHING THE CAPABILITY for the length of that commit — the main
  mdBook page ("`--check …` refuses when the two disagree"), the owning task leaf, and layer-A
  MEMORY.md. The sharpest evidence was that the task leaf QUOTED OUTPUT NO CODE COULD PRODUCE: a
  repo-wide search for its claimed success string `grammar-certification: OK` found it in exactly
  one place — the task file. That is the GENERATED-REPRODUCIBILITY founding defect in the doc tier:
  an artifact carrying a line its producer cannot emit.

  ⇒ THE CHEAP DETECTOR, applicable to any script: `grep -n <VAR>` the option variable. If it appears
  only where it is DECLARED and ASSIGNED and never READ, the flag is inert. Here `grep -n CHECK`
  returned exactly two lines, 26 and 30.
reverify: "grep -n 'CHECK' scripts/report_grammar_certification.sh && bash scripts/report_grammar_certification.sh --check /nonexistent/no/such/page.md; echo \"exit=$? (must be 2, never 0)\""
---

## The rule

When you remove a capability, remove **its whole surface** in the same edit — the implementation
*and* the flag, option, env var or target that reaches it. A surviving entry point does not fail; it
succeeds at nothing, and every document that described the capability stays plausible.

## The red control that would have caught it

Point the check at **a subject that cannot possibly be valid** — a path that does not exist is the
cheapest one — and require a refusal. A check that cannot be made to fail has not been shown to
work, no matter how green it is on the real subject. See
[[a-green-gate-over-a-generated-corpus-is-a-claim-about-the-corpus]] for the same argument on the
corpus axis, and [[which-pgen-grammars-are-certified-and-which-are-not]] for the subject this
particular check guards.
