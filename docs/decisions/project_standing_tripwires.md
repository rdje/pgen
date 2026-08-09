---
name: project-standing-tripwires
description: PROJECT (2026-08-09, MEMORY-ARCH.6) — the live traps that have each already cost a session, demoted here out of the layer-A resume pointer. Each is a mechanism that FAILS SILENTLY OR IN THE PASSING DIRECTION, so nothing warns you: a column-0 comment deletes grammar alternatives with every census still green, a corpus timeout is a fact about the instrument, an acceptance gate is vacuous for one tree shape, three gates are RED or blind on HEAD, a rule-census move silently invalidates the cert contracts, and the clippy flow prints ✅ while skipping. Read the owning leaf before acting in any of these areas.
id: project-standing-tripwires
title: The 8 live traps — every one of which fails silently or in the passing direction, so the signal you would trust is green
date: 2026-08-09
reverify: "grep -c '^| [0-9]' docs/decisions/project_standing_tripwires.md"
answers:
  - "what standing traps should I know about before touching this repo"
  - "I added an alternative to a grammar rule and it silently vanished — why"
  - "why is a comment at column 0 inside a rule body dangerous"
  - "a corpus file timed out — is that a parser defect"
  - "is the TASK-ACCEPTANCE gate actually checking my task tree"
  - "which gates are currently RED or blind on HEAD"
  - "I moved rules in a grammar — what else must I update in the same commit"
  - "clippy_on_rust_change printed a green tick — did it actually run"
  - "why did my grammar-only change skip the generated-parser lint"
metadata:
  node_type: memory
  type: project
---

**Why these live in layer C and not in the resume pointer.** They are durable facts, not "where we
are now" — and inlining them was measured at **985 bytes of a 7 168-byte cap**, i.e. 14 % of layer A
spent on content whose lifecycle is *append-once, supersede-when-fixed*
(`MEMORY_ARCHITECTURE.md` §3). Demoting them here also makes them **retrievable by question** for
the first time: in `MEMORY.md` they were reachable only by reading `MEMORY.md`.

⛔ **What every entry below has in common — and why the list is worth its length.** None of these
fails loudly. Each either fails *silently* or fails *in the passing direction*, so the signal you
would normally trust is green while the thing it measures is broken. That is the only reason a
tripwire list is justified at all: a trap that announces itself needs no note.

| # | the trap | read before acting |
|---|---|---|
| 1 | ⛔⛔ **A `#` comment at COLUMN 0 inside a rule body silently DELETES the following alternatives.** INDENT IT. Every census is blind to the loss — `--lint-grammar` stays clean and the rule count does not move. | `EBNF-FRONTEND-SILENT-TRUNCATION` (+ the `SV-CORPUS-GRAD.3.19` instrument trap, where it bit during a fix) |
| 2 | A corpus **`timeout` is a fact about the INSTRUMENT, not about the parser** — never score it as a parse verdict. | `SV-CORPUS-GRAD.11a` |
| 3 | `TASK-ACCEPTANCE` **leaf-scoping is VACUOUS for `- ID:` list-shaped trees** — measured **65 of 66**. The gate passes without checking what you think it checks. | `GENERATED-LINT-CORRECTNESS.10` |
| 4 | `ci_workflow_local_gate` **CANNOT COMPLETE**. | `CI-PARITY-GATE-ROT.20` |
| 5 | **`--lib` is RED on HEAD and no gate reads it** (`1002/1`). | `CI-PARITY-GATE-ROT.21` |
| 6 | ⛔ **A rule-census move ⇒ re-baseline the cert contract(s) `--dump-rule-profiles` names, in the SAME commit.** | `CI-PARITY-GATE-ROT.22` |
| 7 | ⛔ **`clippy_on_rust_change` prints ✅ and SKIPS on a grammar-only change** (because `generated/` is gitignored) — force it with `PGEN_CLIPPY_FORCE=1`. | `CI-PARITY-GATE-ROT.23` |
| 8 | Sweep traps — a sweep covers only the lane it was pasted into, and a control pinned to a corpus row can make the sweep refuse. | `LANG-CAPABILITY-AUDIT.10.3`, `CI-PARITY-GATE-ROT.19` / `.4b`·2 |

⭐ **Maintenance rule.** A tripwire leaves this list only when its owning leaf CLOSES — never
because it has gone quiet. Entries 4–7 in particular describe gates that are red or blind *right
now*; a quiet gate is the symptom, not the cure. When one closes, supersede the row here rather
than deleting it silently, so the audit trail stays honest
(`MEMORY_ARCHITECTURE.md` §10).
