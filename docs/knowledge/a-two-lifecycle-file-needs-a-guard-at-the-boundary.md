---
id: a-two-lifecycle-file-needs-a-guard-at-the-boundary
title: One path holding two lifecycles — a part meant to be overwritten and a part that must never be — loses the second part, and if the workflow RESTORES the path the loss is invisible to every commit-time gate
answers:
  - "how do I stop a template's header being destroyed when someone fills in the body"
  - "a fixture file has a documented header and a throwaway body — how do I protect the header"
  - "my gate cannot see the damage because the workflow reverts the file before committing"
  - "where should a guard run when the failure never reaches a commit"
  - "is adding a do-not-edit comment enough to protect part of a file"
  - "how do I let a document be reworded but not destroyed"
  - "should a guard that refuses also offer to repair"
tags: [gates, doctrines, fixtures, tooling, enforcement, workflow, evidence]
date: 2026-08-15
status: current
evidence: PARSE-HARNESS.11. PGEN's blessed scratch slot `grammars/scratch/scratch.ebnf` is one tracked path holding a 32-line operating manual and a body documented as free to overwrite. An agent loading a probe grammar replaced the whole file. All 19 registered doctrines stayed GREEN over the destroyed manual, the scratch integration test passed (it asserts the BODY parses), and `git diff --quiet` on that path exited 0 after the normal restore — so no commit-time gate could ever have seen it. Closed by a two-tier check whose `--probe-time` tier hangs off the make rule that depends on the file, i.e. it runs exactly when the file has changed.
reverify: "bash scripts/check_scratch_slot_header.sh --self-test   # 8 passed, 0 failed — including the incident reproduced exactly, a single missing anchor, and a proof the repair preserves the probe body"
---

**When one file holds two parts with opposite lifecycles — one meant to be replaced, one meant to
survive — the replaceable part eventually eats the other.** Whole-file writing is the natural way
to load new content, and nothing in the path distinguishes the halves.

That much is ordinary. What makes this shape worth a card is the second half:

⛔ **If the correct workflow RESTORES the file, the loss can never reach a commit — so no
commit-time gate can see it, no matter how good the gate is.** PGEN's scratch slot is overwritten
with a probe grammar, driven, then `git checkout`-ed back. The commit shows **no diff on that
path**. Every registered doctrine was green throughout; the integration test over that very file
passed, because it asserts the *body* parses. The damage lives entirely inside a window that ends
before the gate opens.

## Where the guard has to go

Not at commit time. **At the moment the file is in its modified state**, which means hanging the
check off whatever already reads it:

- PGEN wired it to the build rule that *depends on* the file. Make re-runs that rule exactly when
  the file has changed since the last generation — precisely the window, with no polling and no
  always-run phony target.
- And to the tool that snapshots the file as evidence. That one matters more than it looks: the
  snapshot tool's own instruction to the next reader is *"copy this back over the slot"*, so a
  snapshot taken from a damaged file **re-creates the damage every time someone follows it**.

Keep a commit-time tier as well, for the case where the damage *is* committed — but know that it is
the weaker leg here, not the backstop it usually is.

## Four things that make the guard hold

1. **Derive the protected part; never keep a second copy.** Compare against `git show HEAD:<path>`.
   A copy of the thing you are protecting is one more thing to drift — and this doctrine exists
   because the first copy was lost.
2. **Split "destroyed" from "edited".** Rewording the protected part is legitimate work. So the
   commit-time tier checks only that it is *structurally still itself* (present, and still naming
   the operational anchors that make it useful), while the byte-identity check applies only during
   the overwrite window, with a declared env escape for a deliberate edit.
3. **Anchor on commands and paths, not prose phrases.** Enumerating wordings is how the same
   population gets measured as 10, then 16, then 18 — every miss silent in the passing direction.
   Ask what the protected text *operationally* provides and pin that.
4. ⭐ **Make the refusal a one-command repair.** A guard that only says NO invites a hand
   reconstruction, and a hand-reconstructed document quietly loses a line. PGEN's re-attaches the
   committed header above whatever body is loaded — the probe survives, the manual returns.

## ⛔ What does NOT work: adding a sentence

The instinct is to write *"do not delete this header"* at the top. Resist it. In this incident the
header already said *"Edit the grammar body below"*, in a section headed `HOW TO USE`, and it was
overwritten anyway. A second suggestion in a file whose first suggestion was just ignored buys
nothing — *a rule nothing checks is a suggestion*. The reader who needed the warning is the reader
who did not read it.
