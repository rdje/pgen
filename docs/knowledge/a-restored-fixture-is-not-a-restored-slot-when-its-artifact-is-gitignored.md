---
id: a-restored-fixture-is-not-a-restored-slot-when-its-artifact-is-gitignored
title: A probe that borrows the scratch slot must RE-GENERATE on restore, not just `git checkout` — the derived parser is git-ignored, so a clean `git status` is not evidence the slot is back
answers:
  - "my probe script restores the scratch grammar on exit — is the slot actually clean afterwards"
  - "why does certified_grammars_are_byte_identical fail on `scratch` when my tree has no changes"
  - "scratch_slot_parses_the_blessed_fixture test fails but git status is clean — what did I miss"
  - "is git checkout enough to undo a scratch-slot probe"
  - "how do I leave the repo handoff-ready after driving the toolbox on an arbitrary grammar"
tags: [scratch-slot, probes, toolbox, gates, handoff, parse-harness]
date: 2026-08-13
status: current
evidence: docs/tasks/artifacts/engine_universal_services/indirect_lr/probe.sh (the EXIT trap, before and after); docs/tasks/ENGINE-UNIVERSAL-SERVICES.md leaf .13 slice 4b (the measured before->after: certified_grammars_are_byte_identical FAILED -> ok, scratch_slot_parses_the_blessed_fixture FAILED -> ok, `grep -c "cast_expr\\|prim" generated/scratch_parser.rs` 128 -> 0); grammars/scratch/README.md and TOOLBOX.md 1.3
reverify: "grep -c 'cast_expr\\|prim' generated/scratch_parser.rs   # 0 on a correctly restored slot; non-zero means a probe's grammar is still compiled in"
---

**The scratch slot is a PAIR, not a file.** `grammars/scratch/scratch.ebnf` is tracked; the
`generated/scratch_parser.rs` compiled from it is **git-ignored**. Two tests read them together and
only pass when they agree:

- `parse_harness_equivalence::gate::certified_grammars_are_byte_identical` — the interpreter runs the
  `.ebnf`, the oracle runs the generated parser.
- `parser_registry::tests::scratch_slot_parses_the_blessed_fixture_to_the_known_verdict_and_ast`.

So the usual restore discipline — `git checkout -- grammars/scratch/scratch.ebnf`, which
`TOOLBOX.md` §1.3 and `grammars/scratch/README.md` both recommend — restores exactly **half** the
slot. The other half keeps whatever grammar the probe last compiled into it.

**Why it is invisible.** `git checkout` cannot restore an ignored path and `git status` cannot report
one. The tree reads clean, the commit workflow passes, the doctrine enforcer passes — and the gate
that actually reads the pair goes red and stays red until someone runs the full lib suite. Measured
once: two tests red for a day, on a tree with nothing uncommitted, after a probe script whose
acceptance checklist had explicitly ticked *"so no generated artifact or fixture drift survives."*

**The failure has a recognisable signature** — the interpreter accepts and the oracle rejects, or the
reverse, on the blessed greeting fixture:

```text
scratch  DIVERGE samples=3 agree=1 diverge=2
  · [Verdict] hello, world! :: interp.accepted=true oracle.accepted=false (interp furthest=7)
```

`furthest=7` is the comma in `hello, ` — the generated parser is not parsing that grammar at all.

**The rule.** Any script that borrows the slot restores it by **checking out AND regenerating**:

```bash
restore_scratch() {
  git checkout -- grammars/scratch/scratch.ebnf 2>/dev/null || return 0
  make -C rust SHELL=/bin/bash focus_scratch >"$WORK/restore_focus.log" 2>&1 || {
    echo "⛔ scratch RESTORE-REGENERATE failed — rerun: make -C rust SHELL=/bin/bash focus_scratch" >&2
  }
}
trap restore_scratch EXIT
```

A probe run has already paid one `focus_scratch` per probe file, so the restore adds one more (~80 s)
to a run that was never cheap. Doing it by hand instead is the same command — the point is that it
must happen on **every** exit path, including a failure and a Ctrl-C, because the state it repairs is
one no tracked-state check will ever flag.

**Generalise past the scratch slot.** Whenever a workflow's state spans a tracked file and a derived
artifact that `.gitignore` covers, "the tree is clean" says nothing about the derived half. Ask what
regenerates it, and make that part of the restore rather than part of the next person's debugging.
