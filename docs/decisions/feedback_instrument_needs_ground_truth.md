---
name: feedback-instrument-needs-ground-truth
description: DISCIPLINE (2026-07-28, session #220, CI-PARITY-GATE-ROT.2) — a measuring instrument with no ground truth is a confident guess. The gate-reachability scanner produced SIX different confident answers (97/71/75/93/53/40 orphans) before it was right, and NOT ONE of the six defects was found by reading the code; every one was caught by requiring the output to reproduce facts the project had already measured. Those facts must become assertions INSIDE the instrument, and on mismatch it must report MISCALIBRATED and refuse rather than print a number.
metadata:
  node_type: memory
  type: feedback
---

**The founding case.** `CI-PARITY-GATE-ROT.2` had to answer a question nothing in the repository
could: *which tracked gates does anything actually invoke?* The scanner that answers it reported, in
successive versions:

| answer | the defect that produced it |
|---|---|
| 97 orphans | a **mention** counted as an **invocation** — the parity gate's surface audits *read* `ast_dump_contract_gate.sh` and `clippy_on_rust_change.sh`, and that was scored as running them |
| 71 | tightening to command position lost workflow `run:` prefixes and `COMMIT.md`'s backticked bullets — host syntax around a real invocation |
| 75 | make's `@` recipe prefix, which orphaned all twenty of `annotation_contract_gate`'s sub-gates |
| 93 | backslash line continuations, which put a script path in command position on the following line |
| 53 | a nested `make` inside a runner's argv — `run_check … make -C rust <target>` — which showed the flagship aggregate invoking **zero** sub-gates |
| 40 | a prerequisite list held in a make variable (`parser_books_gate: $(PARSER_BOOK_GATES)`) |

Every one of those numbers was produced **confidently**, printed in a clean report, and was wrong.

**The part that matters: none of the six was found by reading the code.** Each was found by taking
the output and checking it against something the project had already established the expensive way —
that `ast_dump_contract_gate` sat RED for four sessions *because nothing ran it*, that a tracked
workflow *does* run `mdbook_docs_gate`, that `clippy_on_rust_change` belongs to no aggregate and no
CI workflow. Careful review of the implementation would have caught none of them, because each
defect was a plausible reading of a real shell or make construct.

## The discipline

> **An instrument that reports a number must carry, inside itself, the facts that number has to
> reproduce — and must refuse rather than report when it cannot.**

Concretely, for any new measuring tool (a census, an inventory, a coverage report, a reachability
scan):

1. **Pin known-true facts as assertions in the tool**, drawn from things the project measured
   independently — ideally ones the tool would get wrong in *both* directions (something that must
   come out positive and something that must come out negative). The reachability check pins six,
   plus every sub-gate the SOTA policy declares required.
2. **On mismatch, print `MISCALIBRATED` and exit nonzero.** Do not print the numbers with a caveat.
   A wrong reachability report would have certified the exact rot the doctrine exists to find —
   it is strictly worse than no report.
3. **Prefer controls that break in both directions.** The scanner's first tightening fixed the
   false-positive and introduced a false-negative; only having a control on each side caught both.
4. **When the tool disagrees with you, check the tool is wrong before "fixing" it.** This bit twice
   in one session: a probe arm that stubbed one invocation of `mdbook_docs_gate` expected the
   control to break, but the parity gate *also* replays that command, so the target was genuinely
   still reachable — **the check was right and the arm was wrong.**

## Why this is not just "write tests"

A unit test asserts the tool does what its author meant. These controls assert the tool agrees with
**the world the project already measured**. That is the difference between verifying an
implementation and calibrating an instrument, and only the second catches "this is a reasonable
reading of shell syntax that happens to be false here."

Related: [[feedback_systematically_use_debug_toolbox]] (measure, do not eyeball),
[[feedback_read_prior_art_before_designing]] (re-measure before citing engine behaviour), and
`DOCTRINE-GAP-OWNERSHIP.1`'s sharper sibling — *a probe that passes for the wrong reason is worse
than no probe*, which is the same principle one level down, at the arm rather than the instrument.

Live instance: `scripts/check_gate_reachability.sh` (doctrine `GATE-REACHABILITY`), whose control
block is the reference shape. Evidence and the full defect table:
`docs/tasks/CI-PARITY-GATE-ROT.md` leaf `.2`.
