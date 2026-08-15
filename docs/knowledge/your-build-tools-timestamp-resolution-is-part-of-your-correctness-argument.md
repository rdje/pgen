---
id: your-build-tools-timestamp-resolution-is-part-of-your-correctness-argument
title: GNU Make 3.81 compares mtimes at WHOLE-SECOND granularity, so a scripted edit-build loop silently reuses a stale artifact — a green run against the wrong input
answers:
  - "my build says up to date but I just changed the input"
  - "a scripted regenerate loop keeps giving me the previous run's output"
  - "make skipped a rule even though the prerequisite is newer"
  - "why does my probe agree with the grammar I tested a minute ago"
  - "is macOS's default make safe for a fast edit-build-check loop"
  - "how do I make artifact freshness independent of timestamps"
  - "an agent-driven build loop behaves differently from my manual one — why"
tags: [build, make, tooling, staleness, evidence, instruments, ops]
date: 2026-08-15
status: current
evidence: CI-PARITY-GATE-ROT.32. Apple ships GNU Make 3.81 (2006) as /usr/bin/make and it truncates mtimes to whole seconds; sub-second support landed in make 4.x. Measured on an APFS volume with nanosecond mtimes (so the filesystem was NOT the cause — that hypothesis was raised and refuted): with a prerequisite written 7 ms after its target, `make` printed "is up to date". Reproduced deterministically on a real target by pinning mtimes inside one second — the grammar said `bravo`, the frontend ran 0 times, and the regenerated parser still declared `alpha`.
reverify: "printf 'out: in\\n\\t@echo RULE_RAN\\n\\t@cp in out\\n' > Makefile && : > in && : > out && : > in && make   # prints \"is up to date\" on make 3.81 despite `in` being newer"
---

**A build tool's timestamp resolution is not a detail of the build — it is a premise of every
correctness claim you make from a build output.** If the tool cannot see that the input changed, a
green result is a green result about the *previous* input.

GNU Make **3.81** — still `/usr/bin/make` on macOS — compares mtimes at **whole-second** granularity.
Sub-second comparison arrived in make **4.x**. So any prerequisite rewritten in the same second as
its target is invisible, and the rule is skipped with exit 0.

```
$ : > in ; : > out ; : > in       # `in` ends up 7 ms NEWER than `out`
$ make
make: `out' is up to date.
```

## Why it hides from humans and ambushes scripts

A person edits a file, thinks, then types `make` — more than a second, every time. The bug is
literally unobservable in hand use. A **script or agent loop** that writes an input and immediately
rebuilds hits it on most iterations. That asymmetry is the trap: the workflow that gets automated is
the workflow that was never exercised at speed.

And it fails **silently, in the passing direction**. There is no error, no warning, no exit code —
just an artifact describing something you are no longer testing.

## ⛔ Check the tool before you blame the filesystem

The first hypothesis in the reference deployment was coarse filesystem timestamps. It was wrong, and
measuring it took one command:

```
$ stat -c '%.9Y  %n' probe_1 probe_2
1786817878.583150835  probe_1
1786817878.583396903  probe_2      # APFS, nanosecond mtimes — the filesystem is fine
$ make --version | head -1
GNU Make 3.81                       # ← this is the cause
```

Two plausible causes, one command that separates them. Without it you "fix" the wrong layer.

## The remedies, weakest to strongest

1. **`rm` the artifact before regenerating**, in the phony entry point a human invokes. Cheap, local,
   and it removes timestamps from the argument entirely. Scope it to the operator target — making a
   real file target unconditional drags every downstream consumer into a rebuild.
   ⛔ Do it as a **recipe step plus a recursive `$(MAKE)`**, not as a sibling prerequisite: siblings
   have no guaranteed order under `-j`, so the cleanup can race the build it must precede — a fix
   that reintroduces a nondeterministic version of the same bug.
2. **Require make ≥ 4.0.** Correct, and it costs every contributor an install; decide it from a
   census of which rules actually have sub-second-collidable prerequisites, not from taste.
3. ⭐ **Make freshness content-addressed instead of time-addressed.** Hash the inputs an artifact
   depends on and store the digest beside it; refuse when the digests disagree. This is immune to
   the whole class — and to submodule bumps, clock skew and copied trees as well. In the reference
   deployment the two gates built that way (a parse-cost baseline, a corpus census) were provably
   unaffected while the timestamp-driven path was silently serving stale output.

## The transferable rule

When an instrument's output feeds a claim, **the freshness of its inputs is part of the claim**. Ask
what proves the artifact describes the tree you are standing in. "The build system would have
rebuilt it" is an assumption about a tool's resolution — write the digest check instead, or delete
the artifact and pay the rebuild.
