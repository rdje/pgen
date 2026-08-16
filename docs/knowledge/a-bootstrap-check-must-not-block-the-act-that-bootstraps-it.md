---
id: a-bootstrap-check-must-not-block-the-act-that-bootstraps-it
title: Adding a new input to a baseline's identity block deadlocks the gate — the check fails because the row is missing, and the only operation that writes the row refuses to run while the check fails
answers:
  - "I added a field to my baseline and now nothing can regenerate the baseline"
  - "my gate refuses on a stale artifact and the rebaseline refuses because the gate refused"
  - "how do I widen a freshness/identity check without hand-editing the artifact it guards"
  - "should a rebaseline be allowed to ignore the failure it exists to resolve"
  - "my co-publication check covers a file that my own tool regenerates — is that safe"
  - "how do I tell an operator-declared adoption apart from a real breach"
tags: [gates, ratchets, baselines, identity, bootstrap, instruments, evidence]
date: 2026-08-16
status: current
evidence: ENGINE-UNIVERSAL-SERVICES.21, TWICE, in two different slices of one leaf. Slice 1 made the instrument a fourth identity input of `PARSE-COST-RATCHET`'s baseline; the `instrument` row cannot exist until a rebaseline writes it, and the rebaseline was guarded by `if REBASELINE and not failures`, so identity divergence was a hard failure on EVERY path including the one whose purpose is to resolve it. Slice 4 hit the identical shape one surface over — `cost.md` is both a co-publication surface and an artifact the rebaseline REGENERATES, so its anchor could not be correct until the copy it was blocking had happened. Both fixed the same narrow way, and the second one was predicted by the first.
reverify: "grep -n 'REBASELINE' scripts/check_parse_cost_ratchet.sh   # identity_problem() and copublication_problem() both downgrade to a NOTE under PGEN_PARSE_COST_REBASELINE=1, and ONLY there; the ratchet's own breach check is untouched, so a rebaseline still refuses if a binding counter rose"
---

**A gate that guards an artifact has one operation that is allowed to change what the artifact
says.** If that operation is subject to the gate's own verdict without exception, then any check
that fails *because the artifact has not been regenerated yet* is unreachable by construction.

The shape is easy to miss because it only appears the first time you WIDEN the check:

```
1. add a new row/field/surface to what the gate verifies
2. gate: FAIL — "the baseline has no `<row>`, so that input is unguarded"
3. operator: run the rebaseline, which is exactly the fix
4. rebaseline: `if REBASELINE and not failures: promote(...)`  ->  failures is non-empty
5. goto 2
```

Nothing is broken; every individual rule is correct. The gate is right that the row is missing, and
the rebaseline is right to refuse to promote a tree that fails its checks. Composed, they are a
deadlock whose only exits are **hand-editing the artifact** (which defeats the derivation) or
**deleting the new check** (which defeats the widening).

## The fix, and the line it must not cross

Under the explicit, operator-declared rebaseline flag, downgrade the *adoption-shaped* failures to
NOTES and let the promotion decide. On every other path they stay hard failures, unchanged.

The line is what counts as adoption-shaped:

| failure | under rebaseline | why |
|---|---|---|
| a missing / stale IDENTITY row | note | the row's whole content is "this is the tree we adopted"; the rebaseline is the act of adopting it |
| a stale anchor in an artifact THE REBASELINE REGENERATES | note | it is describing the file about to be overwritten |
| a stale anchor in a HAND-WRITTEN document | **failure** | nothing in the rebaseline will fix it; only a human will |
| the RATCHET itself — a measured value rose | **failure** | this is the thing the gate exists for; a rebaseline is not a waiver |

⛔ **A rebaseline is a deliberate, env-gated operator act meaning *"this tree is the new
reference"* — it is not a bypass, and the moment it silences a real breach it has become one.** The
discriminator is not severity; it is *"will running this operation make the statement true?"* If yes,
it is bootstrap. If no, the gate is reporting something the operation cannot fix and must hold.

## Why this is worth a card rather than a comment

It was measured **twice inside one leaf**, in slices a day apart — and the second time by
someone who had *read* the first fix's write-up, not written it. That distinction matters:
the record of instance 1 was sitting in the same file, correct and specific, and it still did
not transfer. The shape does not announce itself as the shape you
already know: the first instance was an identity HASH ROW, the second a prose ANCHOR in a generated
report, and they look nothing alike until you ask the one question that names the class —

⭐ **"Is this file written by the thing my check is blocking?"**

Ask it of every surface a gate covers, at the moment you add the surface. Related:
[[a-check-that-cannot-run-must-say-so]] is the sibling failure — a gate that cannot evaluate must
report NOT EVALUATED rather than pass; this card is about a gate that CAN evaluate, evaluates
correctly, and thereby prevents its own repair.
