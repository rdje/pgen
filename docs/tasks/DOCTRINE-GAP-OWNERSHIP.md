# DOCTRINE-GAP-OWNERSHIP: a known defect written into a decision record is not tracked work — 58 commits proved it

## Metadata

- Tree ID: `DOCTRINE-GAP-OWNERSHIP`
- Status: `active` (opened 2026-07-27, session #215, **by direct director order**)
- Family / slice-id prefix: `PGEN-DOCTRINE-GAP-OWNERSHIP-<NNNN>`
- Created: `2026-07-27`
- Owner: repo-local workflow
- **Frontier: `.2`** (mechanize it — `.1` DONE session #217: 143 orphans → 18 triaged residue,
  a content-keyed register, and a `--check` ratchet with 11/11 proven arms)

## ⛔ THE DIRECTOR'S ORDER (2026-07-27, session #215, verbatim)

On being shown that the enforcer's box-scoping hole had been known and unfixed:

> *"Why did you we address this issue in the enforcer in the first place ? To me that's a
> critical defect, it shouldn't have happened. please make sure they are all task-tree owned,
> tracked and addressed head on when time permit."*

Two obligations, and they are separable: **(1) OWNERSHIP + TRACKING is due now** — every
recorded-but-unowned gap gets a task-tree leaf; **(2) addressing them head-on is "when time
permits"** — the director explicitly allowed that half to be scheduled, not rushed.

## ⭐ THE PROVENANCE (measured, not asserted)

| fact | evidence |
|---|---|
| the gap was FOUND and WRITTEN DOWN | commit `82e8ff32`, **2026-07-22** (`PGEN-BIN-BUILD-INTEGRITY-0002`) |
| where it was written | a **"Watch item"** paragraph in `docs/decisions/project_build_integrity_compiler_root_cause_signature.md` |
| what it said | *"the checks grep whole files, not the ticked bullet … that whole-file looseness affects all three signature groups and is worth a future box-scoped hardening slice"* |
| was it ever given a task-tree leaf? | **No.** `git log --all -S "box-scoped" -- docs/tasks/` returns only `5c5a0ca0`, the fix itself |
| commits that elapsed | **58** |
| how it was eventually fixed | an UNRELATED leaf (`GENERATED-LINT-CORRECTNESS.3`) tripped over the same hole and re-derived it from scratch |

⛔⛔ **ROOT-CAUSE FRAMING CORRECTED IN PLACE (director, same session).** The first version of
this tree said the cause was that *"no rule says a recorded gap must become a leaf"*. **That is a
deflection and it is false.** The rule already exists, twice over: `CLAUDE.md` item 6 (*"Track ALL
work in task-trees"*) and the standing surfacing directive (*"open a tracked task for it. I should
never be the one who has to notice"*). Director, verbatim: *"But it is your job to do that,
systematically, that's your job. If you see something that isn't correct, you should systematically
task-tree own it. But for some reason you didn't nothing. that's not cool."*

⇒ **ROOT CAUSE: a COMPLIANCE failure, not a missing rule** — and not only in 2026-07-22. This
session repeated it: `GENERATED-LINT-CORRECTNESS.3` *read* that watch item during its prior-art
search, *used* it, and *fixed* it — and still never asked "what else is sitting orphaned like
this?" until the director asked. The mechanization in `.2` is a **backstop for a discipline that
is already mandatory**, never a substitute for it.

⇒ **Contributing structural factor (real, but secondary): `docs/decisions/` is a MEMORY surface,
not a WORK QUEUE.** A defect recorded
there is durable but inert — no frontier points at it, no index lists it as open, no gate asks
about it. The author who records a gap discharges their honesty obligation and the project
silently absorbs a known defect. The task-tree doctrine exists precisely to stop this, and it
was bypassed not by carelessness but because **there is no rule saying a recorded gap must
become a leaf.**

## ⭐⭐ WHY THIS IS THE SESSION'S THIRD INSTANCE OF ONE PATTERN

1. `GENERATED-LINT-CORRECTNESS.1` — `ast_dump_contract_gate`: a check **nothing runs**.
2. `CI-PARITY-GATE-ROT` — `ci_workflow_local_gate`: a gate **nothing runs**, 1,371 commits.
3. **This tree** — a defect **nothing tracks**, 58 commits.

The unifying statement, now covering both axes: **an instrument that nothing invokes, and a
defect that nothing tracks, are both indistinguishable from absent.** `CI-PARITY-GATE-ROT.2`
owns the mechanization for the first; this tree owns it for the second.

## Leaves

### `.1` — triage the measured gap census and give every real orphan a leaf (`done`)

- **Status: `done`** (session #217, 2026-07-28, `PGEN-DOCTRINE-GAP-OWNERSHIP-0002`).
  ⛔ **DIRECTOR-SCHEDULED FOR THIS SESSION** (2026-07-27, verbatim: *"Please plan all of that
  for the next session, there is no sufficient context to handle those things in this
  session"*).

#### ⭐⭐⭐ THE DIRECTIVE THAT RESHAPED THIS LEAF MID-EXECUTION (director, session #217, verbatim)

> *"An elite core, you shouldn't let any issue, even the smallest slide. You should task-tree
> own, track it for a latter activation. I mean do whatever it takes to be 100% sure to address
> it either now or later but do not let it slide unaddressed."*

⛔ This **retired the plan `.1` was written with.** The charter said the `history-narrative`
surface was *"almost certainly NOT backlog … confirm the rule, then exclude the surface
wholesale"* — i.e. drop 63 hits with one judgement call. Under the directive, a wholesale
exclusion is exactly the thing that lets an issue slide: it is a decision **nobody can re-check
and nothing re-applies.** ⇒ the deliverable became a **per-hit terminal disposition for all 143**,
half mechanical (re-derived every run) and half hand-authored (recorded in a tracked register).

⭐⭐ **AND THE DIRECTIVE WAS IMMEDIATELY VINDICATED BY MEASUREMENT.** The wholesale-exclusion
rule was tested before being applied, and it **did not hold as written**: of the 64
`history-narrative` hits, the first ancestor-heading test resolved only **23**. Only after
correcting the test to walk the full ancestor CHAIN (a topical `### Important Boundaries` nested
under a dated `## <entry>`) did it reach **60/64** — and the residual 4 needed hand-triage. Had
the surface been dropped wholesale on the charter's say-so, that correction never happens and the
rule ships wrong.

#### ⭐⭐⭐ THE HEADLINE: A CENSUS IS ITSELF A RECORDED-BUT-INERT FACT

The founding defect sat 58 commits because it was **written down instead of tracked**. A one-shot
census of *"140 orphans"* is the **same disease one level up**: nobody re-runs it, no frontier
points at it, and the next session re-triages 143 hits from zero. ⇒ `.1`'s deliverable is not a
count. It is a **RATCHET**:

```
sweep the repo → mechanically resolve what can be resolved → join the residue
against a TRACKED register → exit nonzero on anything UNTRIAGED
```

⇒ **a newly-recorded gap must be classified or mechanically resolvable, or the check fails.**
Nothing can slide by being merely written down. `docs/decisions/` stops being a place where
defects go quiet.

#### The result: 143 orphans → 18 hand-triaged residue, each with a terminal disposition

| verdict | count | how it is decided |
|---|---|---|
| `FENCE` | 4 | inside a ``` block — example code, not a claim about the repo |
| `PROVENANCE` | 122 | under a DATED/VERSIONED ancestor heading — records what a PAST commit did |
| `OWNED` | 89 | an owner token within ±12 lines |
| **`RESIDUE`** | **18** | **must appear in `gap_triage_register.tsv` with a terminal disposition** |

Residue dispositions: `ROUTED` 7 · `FALSE-POSITIVE` 4 · `BOUNDARY` 3 · `PROVENANCE` 2 ·
`DELIBERATE` 1 · `OWNED` 1. ⇒ **the charter's estimate of "10–25 real" was right in magnitude:
7 genuine orphans, now owned by `.3`/`.4`/`.5`.**

#### ⭐⭐ FOUR DRIVER DEFECTS, EACH OF WHICH PRODUCED A CONFIDENTLY WRONG CENSUS

⛔ These matter more than the count: **every one of them silently marked a real defect as clean,
or a clean line as a defect.** The instrument was the thing that needed auditing.

1. **`whack-a-mole` matched `HACK`** (`grammars/systemverilog.ebnf:426`) — no `\b` on the code
   markers. A prose comment became a code marker.
2. **Quoted grammar TERMINALS matched as gap markers** — `semantic_annotation.ebnf:62` legitimately
   *defines* the annotation names `"todo" | "fixme" | "bug"`. The grammar's own accepted language
   was read as a backlog note.
3. **The owner window was 3 lines** — a real closure annotation measured sitting **5 lines above**
   its hit (`project_every_parser_per_parser_book.md`, corrected last session) was reported as an
   ORPHAN. ⇒ widened to ±12.
4. **Gap phrases inside ``` fenced blocks were matched** — 4 `walking-the-ast.md` walker examples
   whose comments read *"so future slices don't break the walker"*.

⚠️ **A FIFTH DEFECT WAS INTRODUCED BY THE FIX AND CAUGHT BY THE PROBES.** Generalising the code
markers to `\bXXX\b` (from the original `XXX:`) made the combinator suite's own test literals
`("xxx", true)` match — **4 fresh false positives in one edit**. ⇒ **a marker convention needs its
colon to be a marker**; both regressions are now CONTROL arms (8 and 9), so neither can return.

⚠️⚠️ **AND A SIXTH, WHICH WAS MINE, IN THE AUDIT ITSELF.** While auditing whether the OWNER test
was sound, the re-implementation applied `re.I` — and ordinary Rust method calls (`code.push`,
`base.as`, `self.generate`) began matching the task-leaf-ID pattern, marking all 5 real `rust-src`
TODOs as OWNED. The **driver was never case-insensitive**; the audit was. ⇒ the finding was
withdrawn and re-run faithfully. **Recorded because the failure shape is the standing one: an
instrument that disagrees with the thing it is auditing is measuring itself.**

#### Verification — the ratchet BLOCKS (11/11 arms, and each blocks for its claimed reason)

Driver: `docs/tasks/artifacts/doctrine_gap_ownership/run_gap_ratchet_probes.sh` (tracked). Every
arm runs against a **hermetic synthetic repo** (`PGEN_GAP_SWEEP_ROOT`), so no probe can mutate the
real tree.

| arm | asserts | exit |
|---|---|---|
| RED 1 | a new unowned, undated gap phrase BLOCKS | 1 |
| RED 2 | a non-terminal disposition BLOCKS | 1 |
| RED 3 | a register covering only SOME residue still BLOCKS | 1 |
| RED 11 | a real unowned `TODO` in Rust source BLOCKS | 1 |
| GREEN 4 | every residue row triaged PASSES | 0 |
| CTRL 5 | gap under a DATED heading = `PROVENANCE`, no block | 0 |
| CTRL 6 | gap WITH a named owner = `OWNED`, no block | 0 |
| CTRL 7 | gap phrase inside a fenced block = `FENCE`, no block | 0 |
| CTRL 8 | `whack-a-mole` is NOT a `HACK` marker (regression pin) | 0 |
| CTRL 9 | test literal `"xxx"` is NOT an `XXX` marker (regression pin) | 0 |
| CTRL 10 | quoted grammar terminals are not gap markers | 0 |

⭐ **THE RED ARMS WERE THEN AUDITED FOR *WHY* THEY BLOCKED, NOT JUST THAT THEY DID** — RED 2/3
extract a key with `awk`, and a silently-empty extraction would have blocked on `UNTRIAGED`
instead of the condition under test, passing for the wrong reason. Measured: RED 1
`UNTRIAGED=1 bad=0`, RED 2 `UNTRIAGED=0 bad=1`, RED 3 `UNTRIAGED=1 bad=0`. Each blocks on its own
condition.

## Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — `bash docs/tasks/artifacts/doctrine_gap_ownership/run_gap_ownership_sweep.sh`
      at `99b871c8` reported `total hits : 235 / OWNED 92 / ORPHAN 143`, with **no per-hit
      disposition anywhere in the repo** — i.e. the census was itself a recorded-but-inert fact.
- [x] **ROOT CAUSE (WHY + WHERE)** — ops/build-flow family. The census could not be trusted
      *and* could not persist. Four measured driver defects, each located: `git ls-files`-driven
      sweep matched inside ``` fences (4 sites, `walking-the-ast.md`); `grep -nEi` with no `\b`
      turned `whack-a-mole` into a `HACK` marker (`grammars/systemverilog.ebnf:426`); the
      `awk -v s=$((ln-3))` owner window was 3 lines while a real closure annotation sat 5 lines
      above (`docs/decisions/project_every_parser_per_parser_book.md`); and `sort` aborted with
      `string comparison failed: Illegal byte sequence` on non-UTF-8 bytes in the TSV, truncating
      any tally. WHERE: `run_gap_ownership_sweep.sh` lines 22–26 (patterns) and 47–54 (`scan`).
- [x] **FIX** — declarative-first, no engine change. Sweep re-implemented as
      `gap_ownership_sweep.py` with fence-awareness, `\b`-anchored markers, quoted-terminal
      stripping for `.ebnf`, a ±12 owner window, and a case-SENSITIVE owner test; plus the
      tracked `gap_triage_register.tsv` + a `--check` ratchet. The `.sh` entry point is retained
      as a thin wrapper so the path this task file names does not move.
- [x] **ADDRESSED (verified)** — before→after on the symptom: **143 undispositioned orphans → 0
      untriaged**, via 4 `FENCE` + 122 `PROVENANCE` + 89 `OWNED` mechanically re-derived each run,
      and **18 residue rows each carrying a terminal disposition** in the tracked register.
      `run_gap_ownership_sweep.sh --check` → **exit 0**; deleting any register row → **exit 1**
      (RED 3).
- [x] **NO REGRESSION** — `run_gap_ratchet_probes.sh` **11/11**, each RED arm audited to block on
      its own condition (RED 1 `UNTRIAGED=1 bad=0`; RED 2 `UNTRIAGED=0 bad=1`; RED 3
      `UNTRIAGED=1 bad=0`). `bash scripts/check_doctrines.sh` **10/10 doctrines PASS**. No
      `grammars/*.ebnf`, no `rust/src/*`, no `generated/*` touched ⇒ **all 11 generated parsers
      byte-identical BY CONSTRUCTION**; no release / schema / ledger / contract movement.
- [x] **LOCKSTEP** — `MEMORY.md`, `CHANGES.md`, `DEVELOPMENT_NOTES.md` updated; no book chapter
      changed because no user-facing parser surface moved (the three routed leaves `.3`/`.4`/`.5`
      own the user-facing corrections, deliberately NOT fixed here per the tree's own
      *"own them, do NOT fix them here"*).

#### The census (measured 2026-07-27 at `18be844e`, whole repository)

#### The census (measured 2026-07-27 at `18be844e`, whole repository)

Driver: `docs/tasks/artifacts/doctrine_gap_ownership/run_gap_ownership_sweep.sh` (tracked).
**225 hits total — 85 OWNED, 140 ORPHAN.** By surface:

| surface | orphans | triage prior |
|---|---|---|
| `history-narrative` (`CHANGES.md` 30, `DEVELOPMENT_NOTES.md` 25, book changelog-indexes 8) | **63** | ⭐ **almost certainly NOT backlog** — these are append-only historical records; *"not fixed here"* in a changelog entry is **provenance of what that commit did**, not open work. Confirm the rule, then exclude the surface wholesale. |
| `root-docs` | 22 | live — triage each |
| `contracts` (`SV` 11, `regex` 9) | 21 | live — likely published support boundaries, not defects; check |
| `task-trees` | 13 | live — a gap in a tree is usually *already* owned by that tree; verify |
| `decisions` | 10 | **SAMPLED — see below** |
| `rust-src` (TODO/FIXME/HACK) | 5 | live code markers — real candidates |
| `reference` / `grammars` / `book` | 3 / 2 / 1 | live — triage each |

#### ⭐⭐ THE SAMPLE THAT SAYS "DO NOT TREAT 140 AS A BACKLOG"

`docs/decisions/` was triaged by hand this session — **10 raw orphans resolved to 1**:

- **6 FALSE POSITIVES** from the regex — doctrine prose (*"the tool-build is its own task-tree
  leaf"*), a leaf ID in a format the owner-pattern missed (`.b.6.2.36.2`), and 4 hits inside
  `project_codegen_emission_root_cause_signature.md` that are the **closure text** for the very
  gap being searched for.
- **2 STALE-CLOSED**, ⭐ **and this is the inversion worth carrying forward: the real defects were
  records claiming work is OPEN when it is DONE**, not untracked open work. (a) the build-integrity
  *"Watch item"* — closed by `5c5a0ca0` but still reading as open; (b)
  `project_every_parser_per_parser_book.md` still listing `ebnf` / `return_annotation` /
  `semantic_annotation` as *"Still missing"* when **all three books exist** and `README.md` already
  calls the directive complete. A future session would have rebuilt three books that already exist.
  **Both corrected in place this session.**
- **1 OWNED-BUT-UNLINKED** — the `eprintln!` shadows latent finding, genuinely owned by the
  `DIAG-SEVERITY` tree, just never cross-referenced. **Cross-reference added this session** (still
  live: 171 sites in `mod.rs`, 76 in `ast_based_generator.rs`).

⇒ **0 genuinely untracked open gaps in `docs/decisions/`.** Expect the same shape elsewhere: the
actionable count is plausibly 10–25, not 140. ⛔ **Do not open 140 leaves.**

#### How to run `.1`

1. Re-run the driver; work surface by surface, **cheapest exclusions first** (`history-narrative`).
2. Classify every hit into exactly one of: **false positive** (tighten the driver's patterns —
   the driver is the deliverable too), **stale-closed** (annotate the record CLOSED + the commit
   that closed it), **owned-but-unlinked** (add the cross-reference), **genuinely orphaned**
   (open/attach a leaf).
3. ⛔ **Own them, do NOT fix them here.** The deliverable is the triaged census + the leaves +
   a tightened driver whose ORPHAN count is then *meaningful*.
4. Record the age of each genuine orphan (the commit that first recorded it). The 58-commit
   latency of the founding case is the number that made this a tree; the distribution is the
   argument for `.2`.

### `.2` — mechanize it: a recorded gap must name its owning leaf (`todo`)

- **Status: `todo`**, after `.1`.
- Candidate enforced doctrine (would be the **11th**, alongside `CI-PARITY-GATE-ROT.2`'s
  reachability doctrine): a staged `docs/decisions/*.md` or `docs/tasks/*.md` that ADDS
  recorded-gap language must cite an owning leaf ID, or the commit is blocked — the same shape
  as `DESIGN-PRIOR-ART`, which already blocks a novel `@name` without a `PRIOR ART` section.
- ⚠️ Honest limit to state up front, exactly as `DESIGN-PRIOR-ART` does: this can only check
  that an owner was NAMED, not that the owner is real or that the work will happen. That is
  still strictly better than prose nobody indexes.
- ⛔ Design constraint: it must not punish honesty. If recording a gap becomes expensive,
  authors stop recording gaps — which is far worse than an untracked one. Prefer a one-token
  citation over a required section.
- ⭐ `.1` shipped the **mechanism** this leaf needs (a content-keyed register + a `--check`
  ratchet with proven RED arms). What remains for `.2` is the **wiring**: register the check in
  `scripts/check_doctrines.sh` so a staged commit is blocked, not merely a manual run.
- ⭐⭐ **THE HONEST LIMIT ABOVE IS NO LONGER HYPOTHETICAL, AND ITS HARDER HALF IS MECHANIZABLE
  AFTER ALL — measured 2026-08-10 (`SV-CORPUS-GRAD.11a`, `PGEN-SV-CORPUS-GRAD-0197`).** That leaf
  surfaced three findings and recorded them all in a session summary. Audited immediately
  afterwards, at the director's challenge: **only 1 of the 3 was actually task-tree owned.** One
  cited an owner — *"Owner: `GENERATED-LINT-CORRECTNESS.8`"* — written **from inside the SV tree**,
  while `.8` contained zero mention of it; the other existed only in the chat summary and a passing
  line in a knowledge card. ⇒ **naming an owner is not routing; creating the owning leaf is.** Both
  were repaired into real leaves (`GENERATED-LINT-CORRECTNESS.12`, `SV-CORPUS-GRAD.12`), but
  nothing would have caught them.
- ⛔ **So `.2` should enforce BOTH DIRECTIONS, and the second is a cheap structural check** —
  contrary to the "can only check that an owner was NAMED" limit stated above:
  1. *(as designed)* recorded-gap language must CITE an owning leaf ID; and
  2. *(new)* the cited ID must **RESOLVE** — a leaf with that ID must exist in the named tree.
     That is a `grep` for the leaf heading in `docs/tasks/<TREE>.md`, no judgement required, and it
     is exactly the meta-check `scripts/check_doctrines.sh` already performs on its own registry
     (*"a registry entry pointing at a check that does not exist is a dangling promise"*).
  ⚠️ Direction 2 still cannot prove the owner is the RIGHT one, or that the destination leaf
  describes the finding — a mis-routed-but-existing ID passes. State that limit rather than imply
  the check is complete; it converts a silent loss into a wrong address, which is strictly better.
- ⚠️ **Scope note for the design:** the SV instance was cited in PROSE inside another tree's bullet,
  not in a `docs/decisions/` record, so a check scoped to decision records would have missed it.
  The cheapest sound trigger is any staged `docs/tasks/*.md` line naming a `TREE.leaf` id that does
  not resolve — which also catches the far more common typo/renumber drift.

### `.3` — the return-annotation CODEGEN placeholder class: 5 paths that succeed silently (`todo`)

- **Status: `todo`** — opened by `.1` (session #217). ⛔ **OWNERSHIP ONLY; not fixed here** per
  this tree's own rule and the director's split (*"addressing them head-on is when time permits"*).
- ⭐⭐ **THE DEFECT SHAPE — `Ok(...)` WITH ZERO DIAGNOSTICS.** Five codegen paths cannot emit
  working code for the construct they were handed, and each returns **success** carrying a
  placeholder instead of failing or warning. MEASURED: no `warn`, no `error`, no `eprintln`, no
  diagnostic anywhere in the surrounding region.

| # | site | what it emits |
|---|---|---|
| 1 | `rust/src/ast_pipeline/return_annotation_handler.rs:370` | `/* TODO: Implement spread for: … */` |
| 2 | `rust/src/ast_pipeline/unified_return_ast.rs:1872` | `ParseContent::Terminal("<property_access>")` |
| 3 | `rust/src/ast_pipeline/unified_return_ast.rs:1883` | `ParseContent::Terminal("<array_access>")` — ✅ the missing `)` FIXED by `.3a`; the silent-`Ok` half remains `.3`'s |
| 4 | `rust/src/ast_pipeline/unified_return_ast.rs:1903` | `ParseContent::Terminal("<last_extraction>")` (`ExtractionTarget::Last`) |
| 5 | `rust/src/ast_pipeline/unified_semantic_ast.rs:301` | transform expressions parsed as RAW STRINGS via `trimmed.contains("::parse::<")` — function calls / type parameters unmodelled |

- ⭐ **THIS IS A NAMED, PRECEDENTED DEFECT CLASS, NOT A NEW ONE.** `TOOLBOX.md:177` already calls
  `<invalid_sequence_access>` *"the return-annotation corruption class"*, and
  `SV-AST-SHAPE-FIDELITY.2.x` fixed **9 reachable-corrupt sites** of it. ⇒ the sentinel family is
  known; these five are the **CODEGEN half nothing tracks** (the 7 tracked siblings in
  `ast_return_transform.rs` are RUNTIME fallbacks).
- ✅ **BOUND, measured and stated up front: no shipped parser carries any of these today**
  (`grep -rlF` over `generated/` → **0 files** for each of `<property_access>`, `<array_access>`,
  `<last_extraction>`, and the spread TODO). These are **unreached paths for the 11 tracked
  grammars** — a latent trap for a grammar AUTHOR, not a live corruption in a shipped artifact.
  That is why this is `todo` and not urgent.
- ⭐ **WHY IT STILL MATTERS — it is the session's standing principle verbatim: *a check that
  cannot run, or cannot see, must SAY SO, not return green.*** Here codegen cannot HONOUR the
  construct and returns green. Same family as `ANNOTATION-PLACEMENT`'s silently-dropped
  mid-sequence `@emit_fact`.
- **The likely fix tier is declarative, not engine**: refuse at grammar-load with an actionable
  message naming the unsupported construct (the `QUANT-PLUS-ITER.2` step-C precedent), rather
  than implementing four half-features. ⚠️ Decide that in the leaf, after pricing.
- ⛔⛔ **ROUTED IN 2026-07-29 (session #225) from `DONE-BAR.5c` — SITE 3 IS WORSE THAN THIS TABLE
  RECORDED: IT EMITS SYNTACTICALLY INVALID RUST.** Measured while building the silent-success sentinel
  gate: `:1883`'s format string ends `...ParseContent::Terminal(\"<array_access>\"` — **the closing
  paren is missing**, while its two siblings `:1872` (`<property_access>`) and `:1903`
  (`<last_extraction>`) are balanced. ⇒ that path does not merely hand back a placeholder that
  *compiles*; it returns `Ok(...)` carrying **un-compilable source**, still with **zero diagnostics**.
  ⭐ **ROUTING EVIDENCE (does it reproduce outside the family it is sent to?):** the defect is in the
  EMITTER, not in any grammar — it is reachable from any grammar whose return annotation takes an
  array access on this legacy `generate_code` lane, and **0 shipped artifacts carry it today**
  (`grep -rlF '<array_access>' generated/` → 0), so it is LATENT exactly like the rest of this table.
  It is therefore the same defect class, same owner, and needs no new leaf — but the severity note
  belongs here so whoever fixes `.3` does not "fix" a placeholder and leave a syntax error behind.
  ⚠️ **`.3`'s bound is unchanged and still correct** (no shipped parser is affected); what changed is
  what the fix must produce.
  ✅ **FIXED 2026-07-29 by `.3a` on the director's order** (`PGEN-DOCTRINE-GAP-OWNERSHIP-0003`) — the
  emitted-code half only. ⛔ **`.3` STAYS OPEN**: all five paths still return `Ok(...)` with a
  placeholder and zero diagnostics, which is the silent-success defect this tree owns. Balancing a
  paren made the emission *compile*; it did not make it *honest*.
##### `.3a` — the `<array_access>` arm emits SYNTACTICALLY INVALID Rust (`done`, 2026-07-29 session #225, `PGEN-DOCTRINE-GAP-OWNERSHIP-0003`)

- **Director-ordered** (verbatim: *"You will fix that right"*) on the finding routed in from `DONE-BAR.5c`.
- **Scope:** the emitted-code defect ONLY. `.3`'s broader question — whether these five paths should
  REFUSE at grammar-load instead of returning `Ok` with a placeholder — stays OPEN and unprejudiced.
- **FIX:** `unified_return_ast.rs:1883` gains the missing `)`. Its siblings `:1872`
  (`<property_access>`) and `:1903` (`<last_extraction>`) were already balanced; this arm alone
  emitted `ParseContent::Terminal("<array_access>"`, an unclosed call.
- ⭐ **WHY IT SURVIVED — measured, and it is the reusable lesson:** `generate_code` has **ZERO
  production callers repo-wide** (4 recursive self-calls + 2 `#[cfg(test)]` calls, `grep -rn` over
  every crate), and the two tests exercise only `PositionalRef` and `StringLiteral` — **never
  `ArrayAccess`**. ⇒ *a dead path with no test is a place where the compiler cannot help you*: the
  emitted string is data, so no amount of `cargo build` on PGEN itself can see that its CONTENT is
  not valid Rust.
- ⭐⭐ **THE REGRESSION TEST IS THE REAL DELIVERABLE, and it is stronger than the fix.** New
  `unified_return_ast_generate_code_emits_syntactically_valid_rust` parses **every** arm's emitted
  code with `syn::parse_str::<syn::Expr>` (already a dependency). It is **DERIVED, not a hand-list**:
  it walks a table of every `UnifiedReturnAST` variant, so a variant added tomorrow that emits
  malformed code fails without anyone remembering to extend the test — the `GATE-REACHABILITY`
  "derive the roster" discipline applied to a unit test.
- ✅ **NO ARTIFACT MOVEMENT BY CONSTRUCTION AND BY MEASUREMENT:** the path is dead in production, so
  all 11 generated parsers must be byte-identical — asserted by sha256 over `generated/`, not assumed.

###### Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — the emitted string was measured, not read: a parens census over the
  three sibling arms showed `:1872` and `:1903` balanced and `:1883` carrying `Terminal(` with **no**
  escaped closing `")`. Confirmed end-to-end by the new test at the pre-fix revision, which prints the
  emission verbatim: `ParseContent::Terminal("<array_access>"`.
- [x] **ROOT CAUSE (WHY + WHERE)** — CODEGEN-EMISSION family. **WHERE:**
  `rust/src/ast_pipeline/unified_return_ast.rs:1883`, the `UnifiedReturnAST::ArrayAccess` arm of
  `generate_code`. **WHY IT EXISTED AND WHY NOTHING CAUGHT IT:** the arm builds Rust source as a
  `format!` **string**, so the emission is *data* — `cargo build` type-checks the `format!` call, never
  its content; and the path is unreachable in production (`grep -rn "generate_code" --include=*.rs`
  over every crate → **4 recursive self-calls + 2 `#[cfg(test)]` calls, ZERO production callers**),
  with those two tests covering only `PositionalRef` and `StringLiteral`. ⇒ **a string-emitting dead
  path with no test is a blind spot no compiler can cover.**
- [x] **FIX** — one character: the missing `)`. ⛔ Deliberately NOT widened to make the arm refuse
  instead of returning `Ok(placeholder)` — that is the silent-success half and it stays `.3`'s, so
  this leaf cannot prejudge it.
- [x] **ADDRESSED (verified)** — measured **before → after** on the same test, not described:
  **BEFORE** (defect restored in place) `unified_return_ast_generate_code_emits_syntactically_valid_rust`
  **FAILS**, naming the arm and printing the invalid emission —
  *"ArrayAccess: generate_code emitted code that is NOT valid Rust (cannot parse string into token
  stream)"*; **AFTER** it passes. The test parses each arm's output as a real `syn::Expr` and covers
  **all 15 variants** (+ all 3 `ExtractionTarget`s); `variant_tag`'s wildcard-free `match` makes a
  future variant a **compile error** until a sample is added, so coverage cannot silently lapse.
- [x] **NO REGRESSION** — `cargo test --lib --features generated_parsers` → **986 passed / 0 failed /
  21 ignored**; `cargo clippy --lib --all-targets` exit **0**, no warning on the added code.
  ⭐ **ARTIFACT NEUTRALITY MEASURED, NOT ASSERTED** — this leaf edits `rust/src/*`, so the usual
  *"byte-identical BY CONSTRUCTION"* shortcut is **NOT available**. Both sides were regenerated
  (`make -C rust regenerate_generated_parsers`, 246 s each, the fix `git stash`ed out for the second)
  and compared per artifact: **all 11 `*.rs` parsers BYTE-IDENTICAL**; the 9 differing `*.json` differ
  in **exactly one leaf field each — `.metadata.generated_at`** — across **49,207** compared leaves
  (regex 7,065 / systemverilog 31,670 / vhdl 3,395 / …), i.e. the wall-clock stamp and nothing else.
- ⚠️ **INCIDENTAL FINDING, and it defeated this leaf's own first instrument:** the raw-AST JSON
  artifacts embed `metadata.generated_at`, so **`generated/*.json` is NOT byte-reproducible across
  runs** and any `sha256`-over-`generated/` comparison reports a difference for every regeneration
  regardless of whether anything changed. My baseline hash did exactly that and briefly looked like a
  real artifact movement. ⭐ Worse, the obvious sanity check *hides* it: two regenerations issued
  back-to-back land in the SAME second and compare byte-identical, which reads as proof of
  determinism. ⇒ **an artifact-identity check over `generated/` must exclude that field (or compare
  `*.rs` only), and a determinism probe must not be run twice within its own timestamp granularity.**
  Routed as a note here rather than a new tree: no gate currently hashes `generated/*.json`
  (`grep -rn` over `rust/scripts/`, `scripts/` → the identity checks that exist read `*.rs` or use the
  cfg census), so nothing is measurably wrong today — but a future gate that hashes the JSON would be
  flaky from its first run.

- **Companion, already dispositioned as `BOUNDARY`**: `docs/return_annotation_parser_book/src/operators.md:116`
  (*"Known limitations of spread forms"*) is the honest USER-FACING half of the same gap. The book
  is not wrong; the codegen is silent.

### `.4` — `PGEN_USER_GUIDE.md` presents a 75-release-stale regex version pair as CURRENT (`todo`)

- **Status: `todo`** — opened by `.1` (session #217). Ownership only.
- **MEASURED**: `PGEN_USER_GUIDE.md:4198-4200` reads *"the current downstream regex release
  aligned with that hardening slice is: parser release version `1.1.29` / integration contract
  version `1.1.31`"*. The contract's own identity block
  (`docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md:222`) reads *"parser release stays
  `1.1.104`, contract `1.1.106`"*. ⇒ **~75 releases stale, in a top-level user-facing doc,
  presented as current.**
- ⚠️ **This is the CONTENT side of a defect whose GATE side was already found and fixed.**
  `CI-PARITY-GATE-ROT.1` measured the identical stale pair (`1.1.31`/`1.1.29` vs
  `1.1.109`/`1.1.106`) as an **audit assertion pinned to a value designed to change**, and
  re-pinned the assertion. Nothing looked at the prose that carried the same numbers. ⇒ ⭐ **fixing
  a gate's pin does not fix the document the pin was reading.**
- The adjacent compile-oracle baseline in the same block (`false_accept_total=307`,
  `false_reject_total=45`) is also suspect — `DEVELOPMENT_NOTES.md:23374-23378` records
  `314`/`67` for the same `cases_executed=2195`. **Re-measure before re-pinning; do not copy
  either number forward.**
- ⛔ The right fix is almost certainly **not** a fresh hand-typed version pair (that just resets
  the same clock). Prefer removing the version claim or deriving it, per
  `CI-PARITY-GATE-ROT.1`'s finding that pinning a value designed to change duplicates ownership
  already held by the release-policy gates.

#### ⭐⭐ SCOPE EXTENDED 2026-07-29 (session #221) — a FOURTH stale figure family in the SAME block, and it sharpens the conclusion above

Routed in from `CI-PARITY-GATE-ROT.9`, which arrived at this paragraph from the opposite direction
(a gate metric, not a doc audit) and landed on the same prose. **Two independent investigations, one
paragraph.**

`PGEN_USER_GUIDE.md:3816-3830` publishes, under the heading **"Current measured operational
baseline"**:

| published as current | measured 2026-07-29 |
|---|---|
| `initial_targets=804` | **1033** |
| `resolved_targets=804` | **1002** |
| `final_targets=0` | **31** |
| `target_attempts=6526` | 10000 (the family gate's budget) |

⭐⭐⭐ **AND THIS INSTANCE KILLS THE "JUST KEEP IT IN SYNC" FIX OUTRIGHT.** The other three stale
figures in this block went stale because nobody re-copied them. This one is different: until
`CI-PARITY-GATE-ROT.9` landed, **the gate itself was publishing the wrong number** — a diligent
maintainer re-copying from `summary.csv` on any day between 2026-06-02 and 2026-07-29 would have
copied `723`, or `811`, depending only on the attempt budget that run happened to use. ⇒ *a
hand-copied measured value cannot be kept correct by diligence, because the source can be wrong and
the copy has no way to know.*

- ⛔ **Do NOT simply refresh these four numbers to today's values** — that is the treadmill that
  produced the other three, and `.9` proved the treadmill can hand you a wrong number.
- ⭐ **Preferred fix, consistent with what this leaf already concluded about the version pair:** stop
  presenting a moving measured value as a *current* claim. Either (a) name the command and the
  artifact that produce it (`make -C rust SHELL=/bin/bash regex_parser_family_contract_gate` →
  `stimuli_regex_*` in its `summary.txt`) and let the reader measure, or (b) if a figure is genuinely
  useful in prose, stamp it with the date and commit it was measured at, so it can never silently
  become a false current claim.
- **Class question this raises, and it is not answered here:** how many other tracked docs publish an
  un-dated measured figure as current? Not measured — measuring it is part of this leaf's scope when
  taken up, and the answer decides whether a doctrine check (*"a live doc must not present an
  un-dated measured metric"*) is worth mechanizing or is over-mechanization for a handful of sites
  (`GENERATED-LINT-CORRECTNESS.4`'s rule).
- ⚠️ **The dated 2026-03-28 tracker notes in `LIVE_ACHIEVEMENT_STATUS.md:1504-1508` citing
  `resolved_targets=355`/`233` are NOT part of this.** They predate the witness pass (2026-06-02) and
  were correct when written. Rewriting them would be back-dating the record — the discipline this
  project refuses.

### `.5` — the `??` coalesce operator is PROMISED IN A PUBLISHED BOOK and owned by nothing (`todo`)

- **Status: `todo`** — opened by `.1` (session #217). Ownership only.
- **MEASURED**: `docs/regex_parser_book/src/rules-quantifier.md:29` — a **shipped, user-visible**
  chapter states *"A future slice will introduce a coalesce operator in the annotation language so
  `greediness: $2 ?? "greedy"` becomes the literal string"*. Restated at
  `DEVELOPMENT_NOTES.md:14739` as *"A coalesce operator: `greediness: $2 ?? "greedy"`. Future
  slice (option C)"*. **No task tree owns it.**
- ⭐ **This is the sharpest instance of the tree's founding disease in the whole census**: not a
  gap buried in an internal record, but a **promise made to users in a published book** with
  nothing behind it. A reader can reasonably plan against it.
- ⚠️ **`DESIGN-PRIOR-ART` applies before any design work here** — `??` is a new annotation-language
  surface, so the prior-art search over `grammars/ebnf.ebnf` → `docs/decisions/` → `docs/tasks/`
  → `docs/book/` is mandatory, and the enforcer will block a novel backticked surface without it.
- ⚠️ The zero-cost/neutrality acceptance test applies: a coalesce operator must **lower at
  codegen** to existing primitives, so non-users pay ZERO (see
  [[project_capability_growth_is_zero_cost_and_neutral]]).
- ⛔ Two legitimate outcomes, and the leaf must pick one deliberately: **build it**, or **retract
  the promise from the book**. Leaving a published forward-reference unowned is the one outcome
  this tree exists to forbid.

### `.6` — `TASK-ACCEPTANCE` is BLIND to the adjudicator, the one file class that can move the SV graduation bar with zero parser bytes (`todo`, routed by `SV-CORPUS-GRAD.3.15`, 2026-08-09)

- **MEASURED, not inferred.** With the whole `.3.15` change staged,
  `bash scripts/check_diagnosis_evidence.sh` printed
  *"diag-evidence: OK (no code change staged; task-acceptance checklist not required)"* and
  exited 0 — so `TASK-ACCEPTANCE`'s PASS in that commit's 17/17 is **vacuous**. The staged set
  included `stimuli/sv/adjudicate_external_corpus.py` and both tracked adjudication manifests.
- **WHY + WHERE.** `scripts/check_diagnosis_evidence.sh:69-78` sets `code_changed=1` for
  `grammars/*.ebnf`, `rust/src/*`, `generated/*`, the ast-shape contracts, `scripts/check_*.sh`,
  `rust/scripts/*.sh`, `.githooks/*`, `.github/workflows/*.yml`, `rust/build.rs` and the two
  Makefiles. `stimuli/**` matches nothing. The narrowing is **deliberate and documented** at
  `:64-68` — *"ordinary tooling, corpora and helper scripts are not [in scope]"*.
- ⛔ **The classification is what is wrong, not the narrowing.** The comment at `:62` states the
  doctrine's own principle: ***"A change to a gate is a change to what 'verified' MEANS."***
  `adjudicate_external_corpus.py` assigns the EXPECTED VERDICT for all 16 336 + 2 459 corpus
  rows — it is the file that defines what counts as a defect, and the sum of its two
  `unexplained` classes **is** the `.5` graduation bar. It is a proof surface, not ordinary
  tooling. A wrong pin lowers the bar silently and in the passing direction, which is precisely
  the failure `SV-CORPUS-GRAD.3.13` named as *"the trap this leaf must not fall into"*.
- **The blind spot is not hypothetical — it is the recent norm.** `.3.13` (`-0031`), `.3.14a`
  and `.3.15` (`-0034`) each moved the SV bar with **zero parser bytes**, so each was a commit
  the acceptance enforcer could not see. Those three leaves happen to carry the checklist
  anyway, by author discipline; nothing checked that they did.
- **Reproduces outside SV** (`ROUTING-EVIDENCE`): the predicate is family-agnostic — it is a
  path list with no family term — so any family's adjudication/expectation tooling under
  `stimuli/**` is equally invisible. The VHDL lane's `CORPUS-GRAD-ALL.2` expectation work will
  land in the same hole.
- **Owed:** extend the code-change set to the expectation-defining surfaces (at minimum
  `stimuli/*/adjudicate_*.py` and the tracked manifests they emit), with the same
  waiver/routing escape the existing arms have — then re-run it against `-0031`/`-0034` to prove
  it would now bind. ⛔ Do NOT widen to all of `stimuli/**`: the corpora themselves are data and
  the `:64-68` narrowing is right about them.
- **Sibling, same class:** `GENERATED-LINT-CORRECTNESS.11` — `clippy_on_rust_change` cannot fire
  on a grammar-only commit because `generated/` is gitignored, so the union it greps is 0 by
  construction. Both are *a gate structurally blind to the commit class that matters most to it*.
- **Priority: parked.** Governance, and it does not block the SV release lane
  ([[feedback_prefer_feature_work_over_governance_lanes]]).

### `.7` — 224 `[[wikilink]]` citations point at 29 records that do not exist, and no gate reads a link (`todo`, routed in by `ENGINE-UNIVERSAL-SERVICES.13` slice 1, 2026-08-12 session #221)

- **THE GAP.** Layer C (`docs/decisions/`) and the Knowledge Map (`docs/knowledge/`) are addressed
  by `[[name]]` from every other tracked surface — that is how a leaf cites a standing directive.
  **Nothing checks that a `[[name]]` names anything.** `KNOWLEDGE-MAP` verifies the derived map
  matches its fact SOURCES; it never asks whether a *reference* reached a source. So a citation that
  resolves to nothing fails **silently, in the passing direction, and still reads like a citation** —
  the exact archetype this tree was opened for.
- **MEASURED** (`docs/tasks/artifacts/doctrine_gap_ownership/dangling_wikilink_census.py`, read-only,
  over every tracked `.md` in `docs/` plus the root continuity files):

  ```text
  scanned 2,691 [[…]] occurrences against 236 records
    bucket            targets  occurrences   meaning
    MISSING                29          224   ⛔ the defect bucket — a citation that reaches nothing
    SPELLING                9           12   hyphens where the record uses underscores
    TASK-TREE               7            7   a real docs/tasks/<X>.md, wrong namespace
    DOC-REFERENT            1            1   a tracked document, referenced as if it were a record
    NEAR-MISS               1            1   probable typo, one edit from a real record
    NOT-A-WIKILINK          6           21   false positive by construction (`[[bin]]`, `[[link]]`, …)
  ```

- ⛔ **THE HEADLINE IS THE CLASSIFICATION, NOT THE RAW TOTAL** — 266 dangling occurrences would be a
  wrong number to publish, because 41 of them are not broken links at all. This is
  [[feedback_classify_referents_by_requirement]] applied at the point of measurement rather than
  after someone acts on an inflated figure; the script's **exit code is the MISSING count** so a
  future ratchet keys on the only bucket that is a defect.
- ⭐ **THE WORST ROWS ARE STANDING DIRECTIVES, WHICH IS WHY THIS IS NOT COSMETIC.**
  `feedback_research_grounded_sota_no_trial_and_revert` is cited **52 times across 34 files**;
  `feedback_stick_to_agreed_plan_no_silent_drift` **22 times**;
  `feedback_never_edit_generated_artifacts` **12**. A session told to *"read the linked directive"*
  finds nothing, and — worse — a reader who does not check assumes the rule is recorded somewhere.
  ⛔ Those three are not forward markers: a marker is cited once, at the point of intent.
- **OWED, in the order that de-risks it:** (1) decide per MISSING target whether the record was
  **renamed** (repoint the citations), **never written** (write it, or drop the citation), or is a
  deliberate forward marker (say so *in the record's place*, not by silence); (2) fix the three
  mechanical buckets — SPELLING is a `sed`, TASK-TREE/DOC-REFERENT want a distinct link form so a
  tree citation stops looking like a record citation; (3) add the resolution check to `KNOWLEDGE-MAP`
  or as its own doctrine, ratcheted on the MISSING count, so the class cannot regrow.
- ⛔ **SCHEDULE (a NAMED TRIGGER, not a bare `todo` —
  [[feedback_every_finding_is_owned_and_scheduled_never_just_logged]]).** Governance, so it is
  **parked by the SV lane lock** ([[feedback_prefer_feature_work_over_governance_lanes]]) and blocks
  no SV release claim. Trigger: **the first governance slice worked after the SV lane lock lifts**,
  taken together with `.2` (which mechanizes gap-ownership) — they land the same kind of check and
  splitting them would build the same enforcer twice. ⚠️ Until then the census script is the
  standing measurement: it is read-only, takes under a second, and exits non-zero, so it can be
  wired into a gate the moment the leaf opens.
- **HONEST BOUND:** the classifier decides `MISSING` by *"no file of that name in
  `docs/decisions/` or `docs/knowledge/`"*. It cannot tell a renamed record from one never written —
  that is the per-target judgement item (1) owes, and it is deliberately left to a human rather than
  guessed by the instrument.

### `.8` — a `<TREE>.<leaf>` id is UNCHECKED exactly like a `[[wikilink]]`, and there is no single leaf-definition convention to check it against (`todo`, opened 2026-08-22 session #256 by `GRAMMAR-WELLFORMED.H.17.1`, whose session produced a FALSE finding by getting this wrong)

- **THE GAP — this is `.7` in the other namespace.** `.7` measured that nothing verifies a
  `[[name]]` citation reaches a record. **Nothing verifies a `<TREE>.<leaf>` citation either.**
  `docs/TASK_TREE.md` names leaf ids as the index of all tracked work; a session resumes through
  them. An id naming no leaf fails silently, in the passing direction, and still reads like an index
  row — the archetype this tree exists for. ⭐ `.7`'s own census already carries a `TASK-TREE` bucket
  (7 targets / 7 occurrences, *"a real `docs/tasks/<X>.md`, wrong namespace"*), so the two questions
  were already touching; this leaf names the second one properly.
- ⛔⛔ **THE HEADLINE IS NOT A COUNT — IT IS THAT THE QUESTION IS NOT MECHANICALLY ANSWERABLE TODAY.**
  Measured 2026-08-22: **three** leaf-definition conventions are in live use, and a checker that
  knows one reports the other two as missing.

  | | convention | example |
  |---|---|---|
  | A | backticked id in a heading, optional free-text/emoji prefix | ``### ⛔⛔ `.40` NEW `todo` — …`` |
  | B | numbered prose heading naming the FULLY-QUALIFIED id, not backticked | `## 23. PARSE-HARNESS.11 — the blessed scratch slot's …` |
  | C | a body FIELD rather than a heading | ``- ID: `SV-EXH-PROOF.3.3.4.b.6.2.15` `` |

  ⇒ **owed item (1) is a convention decision, not a script.** Until "what defines a leaf" has one
  answer, any ratchet built on it encodes whichever spellings its author happened to have seen.
- **MEASURED** — [`dangling_leaf_id_census.py`](artifacts/doctrine_gap_ownership/dangling_leaf_id_census.py),
  read-only, exit code = the MISSING count (same contract as `.7`'s census):

  ```text
  LEAF-ID-CENSUS: scanned 222 `<TREE>.<leaf>` references in docs/TASK_TREE.md against 111 tree files
    bucket           count   meaning
    MISSING            17   ⛔ the defect bucket — an id the index names with no leaf heading
    DETAIL-DOC          2   written up in a sibling detail document, not in the tree file
    RESOLVED          132   resolves to a leaf-defining heading in its tree
    NOT-A-LEAF-ID      71   false positive by construction (`<TREE>.md` links, prose)
  ```

- ⛔ **THE CENSUS IS ALSO BLIND TO THE INDEX'S SECOND REFERENCE FORM, WHICH IS HOW IT MISSED THE ONE
  ROW THIS SESSION KNEW ABOUT.** `docs/TASK_TREE.md` names leaves BOTH fully-qualified
  (`CI-PARITY-GATE-ROT.40`) and **bare inside the owning tree's own row** (`` `.43` NEW `todo` ``).
  The census matches only the qualified form, so `CI-PARITY-GATE-ROT.43` — added by `-0161` with no
  leaf section, the very instance that prompted this leaf — reads as RESOLVED-by-absence: it is never
  scanned at all. ⇒ 17 undercounts on this axis while over-counting on the convention axis, and the
  two errors do not cancel. Fixing the reference form is part of owed item (1).
- ⚠️ **17 IS AN UPPER BOUND ON ITS OWN AXIS, AND IT IS PUBLISHED AS ONE.** It fell **38 → 42 → 17** across three
  iterations of the same session, each drop caused by learning another convention rather than by
  anything changing in the tree. A fourth convention would lower it again. ⛔ Do not quote 17 as
  "17 leaves are missing"; quote it as *"at most 17, under the three conventions the census knows"*.
- ⛔⛔ **FOUNDING DEFECT — THIS LEAF EXISTS BECAUSE THE NAIVE VERSION PRODUCED A FALSE FINDING AND I
  PUBLISHED IT.** Session #256 reported to the director that `CI-PARITY-GATE-ROT.40`/`.41`/`.42`
  had no leaf sections and that "four are owed". **All three are present** —
  `docs/tasks/CI-PARITY-GATE-ROT.md` lines **249 / 146 / 186**. The grep was
  `^### \x60\.40\x60`, which anchors the backtick immediately after the hashes; those headings carry a
  status emoji first (`### ⛔⛔ \x60.40\x60 NEW \x60todo\x60 — …`). ⭐⭐ **AND THE HAND-CHECK RUN TO CONFIRM
  IT USED THE SAME PATTERN**, so it could not have disagreed — a control that cannot fail, in the
  same session that promoted [[a-control-that-cannot-fail-is-not-a-control]] for the same mistake at
  the instrument level. The retraction is recorded here rather than only in conversation precisely
  because a future session re-running that grep would re-derive the same false gap.
  → [[a-heading-census-is-only-as-good-as-the-heading-grammar]].
- **OWED, in the order that de-risks it:** (1) **decide the convention** — one spelling that defines a
  leaf, with the other two either migrated or explicitly registered as legal; (2) per MISSING id,
  decide *renamed* (repoint the index) / *never written* (write it, or drop the row) / *deliberate
  forward marker* (say so in the leaf's place, not by silence) — the same triage `.7` owes; (3) wire
  the census as a ratchet on the MISSING count. ⭐ **Do (3) TOGETHER WITH `.7` and `.2`**, which owe
  the same kind of enforcer over the same corpus — building it three times is how three different
  answers to "is this owned?" appear.
- ⛔ **SCHEDULE (a NAMED TRIGGER, not a bare `todo`).** Governance ⇒ **parked by the SV lane lock**,
  blocks no SV release claim. Trigger: **the same governance slice that opens `.7`** — they are one
  enforcer over two namespaces. ⚠️ Until then the census is the standing measurement: read-only,
  sub-second, non-zero exit, so it can be gated the moment the leaf opens.
- ⭐ **ONE ROW IS ALREADY OWED BY A NAMED LEAF AND IS NOT WAITING FOR THIS ONE**:
  `CI-PARITY-GATE-ROT.43` was written into the index by `-0161` without a leaf section, and `-0163`
  supplied it — so the class's newest instance was closed the moment it was measured, rather than
  joining the backlog it belongs to.

### `.9` — every per-parser book's TRACKED HTML can be stale against its own TRACKED source, and `tracked_html_check` is a control that cannot go red (`todo`, opened 2026-08-23 session #259 by `GRAMMAR-WELLFORMED.H.20`, found by a director lockstep prompt)

- **HOW IT WAS FOUND**: `H.20` touched one main-book page and ran `mdbook_docs_gate` for lockstep. The
  gate passed — and left **17 dirty files** in `docs/semantic_annotation_parser_book-html/`, a book
  whose source this slice never touched.
- ⛔ **MEASURED DRIFT**: `docs/semantic_annotation_parser_book/src/values-and-references.md` was last
  committed **2026-08-23** (`2b5b26ea`, `GRAMMAR-WELLFORMED.H.16.6b`); its rendered
  `…-html/values-and-references.html` was last committed **2026-07-17** (`ce15385c`) — **37 days**
  earlier. So the published HTML still described the `=>` map-key defect as LIVE one commit after
  `-0173` fixed it, on the surface the director has named as their only window into the project
  (*"I review the book, not the code"*).
- ⭐⭐ **ROOT CAUSE — THE CHECK RUNS AFTER THE THING THAT WOULD MAKE IT FAIL.**
  `rust/scripts/<family>_parser_book_gate.sh` runs `mdbook build` (which WRITES into the tracked HTML
  directory) and then `tracked_html_check` asserts that three landing files **exist**. The build just
  created them. ⇒ the check verifies **presence**, never **currency**, and cannot go red for the defect
  it appears to guard. Same failure shape as
  [[feedback_an_instrument_that_can_only_return_one_reading_is_not_a_measurement]] — and it is
  *worse* than no check, because the gate's `pass:` line reads as a currency verdict.
- ⭐ **CLOSED POPULATION, MEASURED**: the gate rebuilds all **10** per-parser books; exactly **1 of 10**
  (`semantic_annotation`) was stale. So this is a live, low-frequency, silent drift — not a systemic
  rot — which is precisely the profile a presence-check will keep missing.
- ⚠️ **THE DRIFT ITSELF IS REPAIRED** by `-0176` (the regenerated HTML is committed in that slice, since
  leaving a tracked artifact contradicting its own tracked source is a live falsehood). **What is NOT
  repaired is the gate**, which is what this leaf owns.
- **SHAPE OF THE FIX** (design, not yet ruled): after `mdbook build`, assert the tracked HTML is
  byte-identical to what the build produced — i.e. `git diff --quiet -- docs/<family>_parser_book-html`
  — and fail naming the file and the regeneration command. ⛔ Price the determinism first: mdBook embeds
  content-hashed asset names (`searchindex-<hash>.js`), so prove a no-op rebuild is byte-stable across
  two runs on one tree **before** making it binding, or the gate becomes a flake. ⭐ **THE FIRST DATUM IS
  ALREADY IN**: `-0176` ran `mdbook_docs_gate` **twice** on one tree and the second run added **no new
  churn at all** — no second `searchindex-<hash>.js`, no re-touched page — so a no-op rebuild is
  byte-stable *on one machine, one mdBook version, one tree*. ⚠️ That is one arm, not the proof: the
  hash is content-derived, so the untested axis is a **different mdBook version**, which is exactly the
  axis a hosted runner would vary. ⛔ And decide the
  population deliberately: the main `docs/book/` has **no** tracked HTML, so the rule binds the ten
  per-parser books only, and that asymmetry must be stated rather than discovered.
- **ACCEPTANCE**: a control arm that goes RED (edit one `.md`, do not rebuild, gate fails naming it) and
  a GREEN arm (rebuild, gate passes) — a check that refuses everything passes every RED arm, so both
  arms are required ([[feedback_an_ops_change_is_proven_by_an_arm_matrix_not_by_its_diff]]).

## Evidence

- `git log -1 -S "whole-file grep, not box-scoped" -- docs/decisions/project_build_integrity_compiler_root_cause_signature.md`
  → `82e8ff32 2026-07-22`.
- `git log --all -S "box-scoped" -- docs/tasks/` → only `5c5a0ca0` (the fix), i.e. no leaf ever
  owned it.
- 58 commits between the record and the fix.

## Commit log

| slice | leaf | commit subject |
|---|---|---|
| `PGEN-DOCTRINE-GAP-OWNERSHIP-0001` | (tree opened) | a known defect filed in a decision record is not tracked work — 58 commits proved it |
| `PGEN-DOCTRINE-GAP-OWNERSHIP-0002` | `.1` | the census was itself a recorded-but-inert fact — 143 orphans triaged to 18, behind a ratchet that BLOCKS |
