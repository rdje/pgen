---
id: an-actionable-error-message-can-certify-the-thing-it-names
title: An actionable error message can certify the thing it names — a static analyser that reads command text cannot tell an EXECUTED command from a QUOTED one
answers:
  - "my reachability inventory says a target is covered but nothing calls it"
  - "why does a static analyser think this script invokes that target"
  - "can printing a fix-it command have side effects on my tooling"
  - "how do I tell whether a green result from an inventory is earned"
  - "my gate passes and I do not believe it — what do I check"
  - "a register refuses my exemption because the tool thinks the target is reachable"
tags: [gates, static-analysis, evidence, controls, tooling, false-positive]
date: 2026-08-16
status: current
evidence: ENGINE-UNIVERSAL-SERVICES.29 (b) / CI-PARITY-GATE-ROT.34, 2026-08-16 session #241. A new doctrine shipped an on-demand make target that no aggregate, workflow or hook invokes. The repository's gate-reachability inventory nonetheless classified it `git-hook` - its STRONGEST, "automatic" class. Cause - the inventory's root scan reads every `scripts/check_*.sh`, strips `#` comments correctly, and then extracts make targets from command text; the enforcer's refusal message contains the string "make -C rust SHELL=/bin/bash generated_reproducibility_gate" so the reader saw a command and recorded an edge. Proven by RED probe - replacing that one string with a placeholder, changing nothing else, reclassified the target to "UNTRIAGED" on the next run and the doctrine correctly failed. Exposure measured at 2 of 118 gate-named targets, exactly 1 genuinely mis-certified. The sharper half - the exemption register is a two-sided ratchet that refuses a disposition naming a target it believes is reachable, so the false badge actively BLOCKED recording the true state.
reverify: "bash scripts/check_gate_reachability.sh --report | grep generated_reproducibility_gate   # reports it reachable; remove the target's name from the error STRING in scripts/check_generated_reproducibility.sh and it becomes UNTRIAGED"
---

**Writing a good error message can make a static analyser believe the thing you mentioned is
wired up.** Not because the analyser is careless — because *"a command"* and *"the text of a
command"* are the same characters, and only a parser that models quoting can tell them apart.

The uncomfortable part is that the trigger is a **best practice**. A refusal that does not say how
to fix itself is a worse refusal, so mature checks print exactly this:

```text
      Re-derive and re-record — do NOT edit the baseline:
        make -C rust SHELL=/bin/bash generated_reproducibility_rebaseline
```

An inventory that scans check scripts for *"which targets does this invoke?"* reads that line and
records an edge. The target is now **reachable**, in the strongest class the inventory has, on the
strength of a sentence.

## Why comments are handled and strings are not

Almost every such tool strips `#` comments — it is the obvious case and it is one line of code.
Quoted strings are neither obvious nor one line: recognising them means tracking quote state across
line continuations, heredocs and nested `$( )`, which is most of a shell parser. So the comment case
gets handled, the string case does not, and the gap is invisible because both look like ordinary
text.

## The two-order failure

1. **First order — a false green.** A lane nothing runs is reported as automatically covered. That
   is precisely the condition the inventory exists to detect, so the failure is *inside the
   instrument's own charter*.
2. ⛔ **Second order — the false badge blocks the repair.** Exemption registers are usually
   two-sided ratchets: a new orphan must be dispositioned, *and* a disposition naming a
   non-orphan must be removed, so the register cannot accumulate dead exemptions. That second rule
   means the honest row cannot be written while the false edge stands — an attempt to record
   `<target>: accepted-operator-invoked` fails with *"names a target that is no longer orphaned"*.
   **The mis-classification is not cosmetic; it keeps the truth out of the record.**

## How to check whether your green is earned

The probe is one edit and one re-run:

```bash
tool --report | grep <target>              # what does it claim?
# replace the target's NAME inside the error string with a placeholder — change nothing else
tool --report | grep <target>              # if the claim moved, the claim was the string
```

⭐ **Do this when a pass contradicts something you know first-hand.** Here the tell was ordinary and
easy to skip: a target written minutes earlier, called by nothing, reported as automatically
covered. A pass that disagrees with what you personally just built is the cheapest defect report
available — and unlike a failure, nobody hands it to you.

## What NOT to do about it

⛔ **Do not remove the command from the error message.** The message is correct practice; the defect
is in the reader. Fixing the symptom would trade a real improvement in diagnosability for a cosmetic
green, and it would leave the next author to rediscover the same thing.

The fix belongs in the extraction — distinguish an executed invocation from a quoted one — and it
must be proven against the analyser's existing ground-truth controls before it is trusted, because
a change there re-classifies the whole population. Add a RED arm replaying this exact case: a target
named **only** in an error string must come out an orphan.

Related: [[measure-the-blast-radius-during-the-repair-not-after-it]],
[[the-row-that-does-not-fit-the-pattern-is-the-next-investigation]].
