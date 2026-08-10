---
name: feedback-an-ops-change-is-proven-by-an-arm-matrix-not-by-its-diff
description: DISCIPLINE (2026-08-11, SV-CORPUS-GRAD.12c.3) — a shell/ops guard is verified by RUNNING it, never by reading it. Both halves of one small guard were plausible on reading and wrong on running: the whitelist variable sat inside the namespace it policed, so the guard refused itself; and `trap … EXIT` does NOT fire when bash dies on an untrapped signal, so a cleanup trap was inert against the `timeout` SIGTERM that was the only cause of the symptom. Acceptance evidence for an ops change is an ARM MATRIX — RED (defect reproduces, now refused), GREEN (legitimate paths still work), CONTROL (deliberately break the guard and confirm it fails).
metadata:
  node_type: memory
  type: feedback
id: feedback-an-ops-change-is-proven-by-an-arm-matrix-not-by-its-diff
title: An ops/script change is proven by an arm matrix, not by its diff — plus two traps that make shell guards inert
date: 2026-08-11
answers:
  - "how do I verify a shell script guard or a Makefile change"
  - "my cleanup trap does not run when the script is killed — why?"
  - "does trap EXIT fire on SIGTERM in bash?"
  - "why did my env-variable guard reject its own configuration variable"
  - "what evidence should an ops or build-flow change carry"
  - "how do I prove a guard can actually fail"
reverify: PGEN_CORPUS_OUTDIR=/tmp/x bash stimuli/run_external_corpus.sh sv; echo $?   # expect 6 + a self-explaining refusal
---

**The founding case.** `SV-CORPUS-GRAD.12c.3` made the external-corpus runner refuse an
unrecognised `PGEN_CORPUS_*` variable instead of silently ignoring it (an unread
`PGEN_CORPUS_OUTDIR` near-miss meant the run wrote to the *canonical* directory and overwrote
tracked oracles while the operator believed it was sandboxed). Two changes, both plausible on
reading, both wrong on running.

## Trap 1 — a guard whose bookkeeping lives inside its own subject matter

The whitelist of recognised names was held in a variable called `PGEN_CORPUS_KNOWN_VARS`. The guard
enumerates the environment with `${!PGEN_CORPUS_@}` — which includes the variable holding the
whitelist. First invocation:

```
external-corpus: REFUSING — unrecognised PGEN_CORPUS_* variable(s): PGEN_CORPUS_KNOWN_VARS PGEN_CORPUS_OUTDIR
```

⛔ **The fix is to move the state OUT of the policed namespace, never to special-case it.** A
special case is the enumerate-the-exceptions pattern the guard exists to remove, and it would have
grown one entry per future collision. Generally: a checker whose own bookkeeping is expressible in
the language it checks will keep finding itself.

## Trap 2 — `trap … EXIT` does not fire on an untrapped signal

The second half was "remove the scratch temps on exit, not just on the success path":

```bash
trap 'rm -f "$RAW"' EXIT          # ← INERT for the case that produces the symptom
```

**Bash runs the EXIT trap when the shell exits normally or via a *trapped* signal — not when it
dies on an untrapped one.** The killer here is `timeout`, which sends SIGTERM. So the fix covered
every case except the only one that had ever left residue. The re-measure — kill a run, then
`git status` the directory — still found the file. What works:

```bash
_cleanup() { [ -n "${RAW:-}" ] && rm -f "$RAW"; return 0; }
trap _cleanup EXIT
trap 'exit 130' INT      # each handler just exits, which THEN runs the EXIT trap
trap 'exit 143' TERM     # rm -f is idempotent, so the double call is free
trap 'exit 129' HUP
```

⚠️ SIGKILL cannot be trapped. Say so in the code rather than letting it be discovered.

## The discipline

> **For an ops/script/build-flow change, the acceptance evidence is the ARM MATRIX, not the diff.**

Three kinds of arm, and all three are needed:

| arm | asks | why it is not optional |
|---|---|---|
| **RED** | does the defect reproduce, and is it now refused? | without it you have not shown the change does anything |
| **GREEN** | do the legitimate invocations still work? | a guard that refuses everything passes every RED arm |
| **CONTROL** | deliberately break the guard — does it fail? | without it you cannot distinguish "green" from "inert" |

Both defects above share one shape: **plausible on reading, inert or self-defeating on running**,
and no amount of re-reading would have shown either. The discriminating step cost seconds in both
cases — invoke the script once; kill it once and look at the directory.

⛔ **State the honest gap in your own control.** Here the control catches a whitelist that *widens*
(mutate it to accept the near-miss → the run must exit MISCALIBRATED), but a whitelist that *drops*
a name is self-consistent and invisible to it. That failure mode refuses a legitimate variable —
loud and immediate, the safe direction — which is why it is acceptable rather than fixed. Recording
which way your control is blind is part of shipping it.

Sibling of [[feedback_instrument_needs_ground_truth]] (an instrument must carry the facts its
output has to reproduce) and [[feedback_enumerating_instrument_must_refuse]] (classify totally,
refuse the unmatched — the rule this guard implements). Those two are about a measuring
instrument's *output*; this one is about proving a guard's *behaviour* actually exists.
