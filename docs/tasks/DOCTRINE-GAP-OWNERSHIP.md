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

### `.10` — `RUST_CODEBASE_ANALYSIS.md` sits AT its distinct-date ceiling, so the NEXT architecture note breaches it (`todo`, opened 2026-08-23 session #260 by `GRAMMAR-WELLFORMED.H.16.4a`)

- **WHY**: the document's charter in the register is `status` — a live architecture/state assessment,
  reviewed and updated *"whenever a task materially changes Rust architecture"* (`COMMIT.md`). Its
  actual shape is **nineteen `## Recent Architecture Change Note (<date>)` / `## Steering Note
  (<date>)` sections stacked above the assessment**, i.e. a changelog wearing a status document's
  charter. `LIVE-DOC-CURRENCY` instrument A caps a `status` surface at **20 distinct dates**, and the
  file is at **20**.
- **MEASURED, and it fired on a real slice rather than on inspection.** `H.16.4a` followed the
  standing `COMMIT.md` instruction to review/update this document for an architecture change, added a
  dated note in the document's own established convention, and the pre-commit enforcer refused: *"21
  distinct dates (> the `status` ceiling 20) — it has stopped being a status view."* ⇒ **the
  document's own convention now conflicts with its charter**, and any author who follows the
  convention is blocked.
- **HOW `-0177` DISCHARGED ITS OWN INSTANCE, which is the shape the fix should generalise**: the
  dated note was DROPPED and the durable architectural fact folded into the permanent
  `## Major Architectural Layers → 2. Parser Code Generation` section as undated prose naming its
  owning leaf. That is strictly better for a live assessment — a reader wants *what the architecture
  IS*, and the date is already in `CHANGES.md` and `DEVELOPMENT_NOTES.md` — but it was one author's
  call on one slice, not a policy, and the nineteen existing notes are untouched.
- ⛔ **The ceiling must NOT be raised** — the doctrine says so explicitly, and raising it is the
  failure restated as a policy (`README-POLICY.2`'s lesson).
- **What this leaf owes**: a ruling on the document's convention (fold dated notes into the permanent
  sections and route the history to `CHANGES.md`, versus declaring an `out_of_charter` entry in
  `rust/test_data/grammar_quality/live_document_currency_register_v0.json` with this leaf as owner),
  then the mechanical pass that implements it over the nineteen existing notes. ⚠️ Price it first: the
  notes are not uniformly disposable — several carry measurements that exist nowhere else, and a
  blind deletion would lose them. The classification is the work, exactly as
  [[feedback_classify_referents_by_requirement]] argues.
- ⭐ **HEADROOM MEASURED, not estimated** (re-derived under a director challenge, 2026-08-23, with the
  instrument's own regex `20\d{2}-\d{2}-\d{2}`): the file carries **exactly 20** distinct dates
  against a ceiling of 20 ⇒ **headroom 0**.
- ⚠️ **Not urgent, but it is a TRIPWIRE, not a wish** — stated precisely, because the loose version is
  wrong: the next architecture note carrying a date **not already among those 20** is blocked at
  pre-commit. A note dated on an existing date adds no distinct date and passes, so the block is not
  literally *"the next note"*. The cheap resolution (undated prose naming the owning leaf) is now
  demonstrated by `-0177`.

### `.11` — `DESIGN-PRIOR-ART` cannot see a directive name a grammar DECLARES, only one it USES, so it reported a 122-name-old annotation as NEW (**`done`**, `PGEN-DOCTRINE-GAP-OWNERSHIP-0001`, CODE / doctrine enforcer — opened and CLOSED 2026-08-23 session #261 by `GRAMMAR-WELLFORMED.H.16.6e`, which it blocked)

- **HOW IT SURFACED — on a real commit, not by inspection.** `H.16.6e` quoted a REAL generated
  stimulus in its leaf, `@handles : { W => %RXCy => 72e10 }`, as the evidence for its central
  finding. The pre-commit enforcer refused: *"proposes a NEW annotation surface with no recorded
  prior-art search — novel directive name(s): handles"*.
- **ROOT CAUSE (WHY + WHERE), measured.** `scripts/check_design_prior_art.sh` builds `known_names`
  from two sources: the semantic-directive registry, and `grep -rhoE '@[a-z_][a-z0-9_]*' grammars/`
  — i.e. names a grammar **USES** in `@name` form. But a grammar **DECLARES** its annotation names
  as QUOTED ALTERNATIVES: `grammars/semantic_annotation.ebnf:52` reads
  `"throws" | "catches" | "handles" | "propagates" |` among **122** such names, and
  `grammars/ebnf.ebnf:684` `optimization_directive` does the same. **None of the 122 is ever spelled
  `@name` anywhere in `grammars/`**, so the usage sweep is blind to every one of them.
- ⛔ **AND THE ONLY REMEDY THE CHECK OFFERS WOULD HAVE BEEN A FALSE RECORD.** Its instruction is
  *"add a PRIOR ART section"* — which here means writing down a search for a name the project has
  had since the grammar was authored. The doctrine's own founding lesson is that finding prior art
  is a SUCCESS; recording a search whose answer was already "yes, for years" is the opposite. This
  is the same shape as `DESIGN-PRIOR-ART.2` (the `docs/tasks/artifacts/**` exclusion), where the
  remedy was also unavailable — and it is why the fix is in the enforcer, not in the leaf.
- **THE FIX — a THIRD `known_names` source: names a grammar DECLARES.** An `awk` pass over
  `grammars/*.ebnf` harvests quoted lowercase literals from rules whose NAME mentions `annotation`
  or `directive`. ⚠️ **Widening `known_names` widens the doctrine's blind spot**, so the harvest is
  censused rather than trusted and three restrictions are deliberate: the text after `->` is dropped
  (a return annotation is a PAYLOAD, not a declaration), a trailing `#` comment is dropped (prose is
  not a declaration), and a grammar-level directive line such as `@profiles:` is skipped (it
  belongs to the NEXT rule). Loose form: **139** names, 12 of them not names at all. Shipped form: **127** — the 122
  predefined annotation names plus the 5 lowercase `optimization_directive` names — and nothing
  else. ⚠️ **HONEST FLOOR, stated not hidden**: the literal pattern is lowercase-only, so
  `pushMode` / `popMode` in that same rule are still unharvested. That leaves the check STRICT where
  it was strict, which is the safe direction for an evidence gate.

#### Acceptance checklist (`TOOLBOX.md`)

- [x] **ROOT CAUSE (WHY + WHERE)** — ops/build-flow family. WHY: `known_names` is built from `@name`
  USAGE, and a grammar DECLARES annotation names as quoted alternatives that are never spelled
  `@name`. WHERE: `scripts/check_design_prior_art.sh`, the `known_names="$( … )"` block. Tool-backed
  and not read off the source: the BEFORE arm reproduces the enforcer's own refusal verbatim
  (`novel directive name(s): handles`, rc 1), `bash -n scripts/check_design_prior_art.sh` clears the
  patched script, and the population is enumerated with `git ls-files 'grammars/*.ebnf'` — **18**
  tracked grammars, over which the `@`-usage sweep sees only **26 of the 127** declared names, leaving
  **101 invisible**. ⛔ **The first version of this box said the sweep sees ZERO of them, and that was
  wrong in my own favour** — a handful (`@throws`, `@deprecated`, `@type`, …) do appear as usages in
  grammar comments and examples, so the defect is a 101-name hole, not a total blindness. Re-derived
  by `comm` over the two sets rather than asserted.
- [x] **ADDRESSED (verified)** — two-arm control on the SAME staged set, exit codes captured from the
  command's own status rather than through a pipe
  (`docs/tasks/artifacts/doctrine_gap_ownership/design_prior_art_declared_names_arms.sh`):

  | arm | rc | output |
  |---|---|---|
  | BEFORE — HEAD's enforcer | **1** | `novel directive name(s): handles` |
  | AFTER — patched enforcer | **0** | *(clean)* |
  | RED — a name that exists nowhere, staged | **1** | `novel directive name(s): zzz_not_a_real_directive` |

  ⛔ **The first version of this control read rc=0 on ALL THREE arms and would have reported the
  defect as absent.** Two independent causes, both recorded in the script's header: `rc=$?` after a
  `| head` reads the PIPE's status, and the BEFORE arm run from a scratch directory derives its repo
  root as `dirname "$0"/..`, finds no staged files there and exits 0 — *a green arm that proves
  nothing*. The arm is now copied into `scripts/` before it runs, and removed by a `trap`.
- [x] **NO REGRESSION** — replayed over the WHOLE tracked population, not a sample: for each of the
  **161** tracked task-leaf files (`git ls-files 'docs/tasks/*.md'`, artifacts excluded) the novel-name
  verdict was recomputed under the BEFORE and AFTER `known_names`. **157 of 161 are byte-identical**,
  and the 4 that move lose exactly **two** names between them — `handles` and `export`, both declared
  predefined annotation names in `grammars/semantic_annotation.ebnf`. Nothing that was genuinely novel
  became known. ⚠️ Deliberately a STRICTER test than the enforcer runs: it scores every backticked
  `@name` in the whole file, where the enforcer scores only ADDED lines, so the population here is a
  superset. The check still fires on a name that exists nowhere (RED arm above, rc 1), and the
  widening itself is censused name-by-name (`design_prior_art_declared_names_census.txt`: 139 → 127,
  all 12 dropped traced to a return annotation, a comment or a `@profiles:` line; all 5 additions
  traced to `optimization_directive`). All 25 doctrines PASS with this change staged, `clippy` not
  applicable — no Rust or generated byte is touched.
- **`promotion: declined (per-slice enforcer mechanics — the transferable half, that an arm run from the wrong root exits 0 and proves nothing, is already carried by [[a-control-that-cannot-fail-is-not-a-control]])`**

### `.12` — the pre-commit hook stages the DERIVED Knowledge Map but not the fact FILES it was derived from, so a commit can ship a map advertising records that are not in it (`todo`, opened 2026-08-23 session #261 by `PGEN-DOCTRINE-GAP-OWNERSHIP-0001`, which produced the instance)

- **MEASURED ON A REAL COMMIT — this leaf's own predecessor.** `.githooks/pre-commit` runs
  `knowledge-map/scripts/gen_knowledge_map.sh` and then `git add`s the generated map (`:18`–`:19`).
  The regeneration reads the WORKING TREE, so it happily indexes fact files that are present but
  UNSTAGED — and stages the map that names them.
- **THE INSTANCE, proven rather than argued.** Commit `13e866c8` staged 7 files plus the
  hook-added `KNOWLEDGE_MAP.md`. Three fact records existed on disk, untracked, and were not part of
  that commit. Result at that commit: the map references
  `a-cat-glob-whose-output-matches-its-own-glob-fabricates-a-record` **10** times,
  `a-minimal-core-is-a-witness-only-for-the-vintage-it-was-shrunk-under` **9** times and
  `an-inherited-residual-may-have-died-with-the-fix-that-came-before-it` **9** times, while
  `git ls-tree -r 13e866c8 -- docs/knowledge` tracks **0** of the three. The record-file count is
  **329 at `13e866c8` and 329 at `13e866c8^`** — unchanged — beside a map that grew by 51 lines.
- ⛔ **AND THAT COMMIT FAILS ITS OWN DOCTRINE.** Checked out into a detached worktree,
  `bash knowledge-map/scripts/check_knowledge_map.sh` exits **1**: *"KNOWLEDGE_MAP.md is OUT OF SYNC
  with the fact files"*. So the pre-commit run was GREEN and the resulting commit is RED — the
  failure direction that makes a gate worse than none, because the green is what everybody reads.
- **WHY IT IS NOT SELF-HEALING IN GENERAL.** It heals only if the records happen to be committed
  later. Any commit that lands while a new record sits untracked ships a map that over-advertises,
  and a bisect, a CI job or a fresh clone at that commit sees a doctrine failure with no local cause.
- **THE SHAPE OF THE FIX (not chosen here — this leaf is routing, zero code bytes).** The check
  regenerates from the WORKING TREE while the commit is the STAGED tree, so the two disagree by
  construction. Candidates: verify sync against the staged tree (`git stash`-free, via
  `git ls-files --cached` or a temporary index), or make the hook refuse when the regeneration read
  a fact file that is not staged. ⛔ *Do not* make the hook auto-`git add` untracked fact files —
  staging files the author did not choose is a worse failure than the one it fixes.
- ⚠️ **Sibling, not duplicate**: `.9` is about a TRACKED render being stale against its TRACKED
  source. This is about a tracked render being CURRENT with sources that are not tracked at all.

### `.14` — a leaf's STATUS is prose in its heading, and NOTHING derives it from the leaf's own recorded slices, so a leaf can be FIXED and still read `todo` (`todo`, routed in 2026-08-23 by `SV-CORPUS-GRAD.13c.2d` / `PGEN-SV-CORPUS-GRAD-0280`)

> ⛔ **ROUTED, NOT WORKED** — found twice in one session while executing the SV-release frontier.
> It is a tracking-integrity gap, and the standing director preference is product/parser lanes over
> governance lanes, so it is OWNED and SCHEDULED here rather than fixed in-line.

- ⛔⛔ **TWO CONFIRMED INSTANCES, ONE SESSION, AND THE SECOND IS THE SHARPER ONE.**

  | leaf | heading said | truth at HEAD | who fixed it |
  |---|---|---|---|
  | `SV-CORPUS-GRAD.13c.2b` | `DIAGNOSED` … *"blocked on `ENGINE-UNIVERSAL-SERVICES.13`"* | fixed for **9 days** | `ENGINE-UNIVERSAL-SERVICES.17` slice 9 — **another tree** |
  | `SV-CORPUS-GRAD.13c.2d` | `` `todo` `` | fixed for **6 days** | `.13c.2f` slice 3 — **this leaf's own slice**, in a commit whose subject reads *"leaf SV-CORPUS-GRAD.13c.2d fix SHIPPED"* |

- ⭐⭐ **THE SECOND ROW IS WHY THIS IS NOT A ROUTING PROBLEM.** The obvious diagnosis for row 1 is
  that a routed-out defect leaves a pointer from VICTIM to OWNER and none the other way, so the
  owner closes and the victim never hears. That is true, and it is **not** what row 2 is: there the
  commit that shipped the fix **named the leaf in its own subject line**, and the heading still did
  not move. ⇒ the mechanism is that **a leaf's status is free prose inside its heading**, written
  once when the leaf is opened, and no instrument compares it against anything the leaf later
  records about itself.
- ⛔ **WHY IT MATTERS MORE THAN A TIDINESS ISSUE, and it is the SV release that pays.** The
  frontier is read off these headings — by `docs/TASK_TREE.md`, by `MEMORY.md`, and by every session
  that resumes. A stale `todo` **inflates the remaining-work picture for the exact question the
  director keeps asking** (*"what is left before the SV parser ships?"*), and it does so in the
  direction that is hardest to notice: the work looks bigger than it is, so nobody investigates.
  ⭐ Both instances were found by *doing* the leaf, not by reading it — which is the expensive way.
- ⛔ **THE CENSUS IS NOT AVAILABLE YET, AND THAT IS THIS LEAF'S FIRST OBLIGATION, NOT A CAVEAT.** A
  first-cut audit written while closing `.13c.2d` — match each `class=fixed` row of
  `stimuli/sv/adjudication_repros/MANIFEST.tsv` to the leaf its note names, then read that leaf's
  heading — returned **6 suspects out of 11 leaves**, and inspection showed the instrument was
  matching **several headings per leaf** (a leaf's own heading, its sub-leaves', and slice headings
  that merely mention it), so most "suspects" carried `done` *and* `todo` from different lines. ⇒ the
  number is unusable and is deliberately **not published**. This is the same trap
  [[a-heading-census-is-only-as-good-as-the-heading-grammar]] records, and it is exactly what
  `.8` already owns: **three leaf-definition conventions are in live use**, so "the heading that
  DEFINES leaf X" is not yet mechanically answerable. ⇒ **`.14` is BLOCKED ON `.8`** for its census
  arm; the finding above is independent of it and stands on two hand-verified instances.
- **WHAT THIS LEAF OWES**: (a) the census, once `.8` supplies a single leaf-definition convention —
  how many `todo`/`blocked` leaves have a subject that is already fixed at HEAD; (b) a decision on
  the mechanism, and there are two shapes worth pricing against each other — **derive** the status
  from the leaf's own recorded slices (cheap to check, but a leaf can record a slice without being
  closed, so it would need a `CLOSES` marker), or **cross-check** it against an external oracle that
  already knows the truth (`MANIFEST.tsv`'s `class` column literally reads `fixed`, and
  `run_adjudication_repros.py` re-derives it in 2.25 s — see `CI-PARITY-GATE-ROT.44`); (c) whichever
  wins, a RED arm proving it fires on a leaf whose heading is stale, reconstructed from the two
  instances above, which is the only reason they were written down here with their dates.
- ⚠️ **HONEST BOUNDS.** Two instances, both in `SV-CORPUS-GRAD`, both found incidentally rather than
  by search — so the rate is unknown and could be two. Neither instance caused a wrong technical
  decision; both were caught before any work was done on the basis of the stale status. And the
  direction of the error is the safe one for correctness (work looks bigger than it is, never
  smaller) — which is precisely why it can persist unnoticed.

### `.13` — a published QUANTITATIVE claim can reach a live surface with NO PRODUCER, and the three-way check does not ask (`todo`, opened 2026-08-23 session #261; **the director DELEGATED this decision to me and the decision recorded here is: open the leaf, do NOT ship the gate I first proposed**)

- **THE TRIGGER, twice in four slices.** `-0183` published *"~100× cheaper"*; re-derived under
  challenge it is **~3.4×** (`-0184`). `CI-PARITY-GATE-ROT.32` published an *"honest bound"* that was
  *"reasoned not measured and false in the flattering-to-my-caution direction"*. In both cases the
  number was not mis-derived — **it was never derived at all**, and that is a different defect from
  every one this repo already watches.
- ⛔ **WHY `CLAIM_VERIFICATION.md` DOES NOT COVER IT.** Leg 1 is literally *"RE-DERIVE by command"*,
  so the rule already exists. It is applied to a leaf's HEADLINE numbers and not to every number in
  the prose, and nothing asks — the exact provenance of `LESSON-PROMOTION` (*"the promotion mechanism
  existed, was wired, and was skipped 1 592 times because no gate asked"*). ⇒ **this is an
  enforcement gap, not a missing rule**, and a new rule would be the third reminder to lose.
- ⛔⛔ **MY OWN FIRST PROPOSAL IS REJECTED HERE, BY ME, AND THE REASON IS THE POINT.** I offered the
  director: *"any multiplier or ratio in a findings callout must name the two measurements it
  divides."* It fails on three counts, and the third is fatal:
  1. it scopes to the **findings callout**, an ephemeral chat surface no gate can read — the tracked
     surfaces are `docs/tasks/*.md`, `CHANGES.md`, `MEMORY.md`;
  2. *"multiplier or ratio"* is a **classification I reasoned rather than measured**, and this tree
     has already been burned twice by exactly that (`.13c.2x`(d), where a key-name classifier was
     *"wrong BOTH ways"*, and the `LEAF-ID-CENSUS` that fell 38 → 42 → 17 as conventions were learned);
  3. ⇒ **it designs an instrument on an unmeasured population to prevent publishing an unmeasured
     number.** Shipping it would have been the defect wearing the fix's clothes.
- **THE POPULATION IS REAL AND IT IS LARGE — measured, and this measurement is a FLOOR.** Marker
  counts over `CHANGES.md` (2 337 dated entries): `REFUTED` **173** · `was wrong` **86** ·
  `retracted` **32** · `overstat*` **18** · `undercount` **9** · `reasoned, not measured` **4** ·
  `mis-derived` **2** · `was stale at` **1**. ⚠️ These are *text markers*, not adjudicated events —
  a marker may name someone else's refuted hypothesis rather than a corrected published number. **No
  number here may be quoted as a defect count until the events are classified BY HAND.**
- **WHAT THIS LEAF MUST DO, IN THIS ORDER — and step 1 is not optional.**
  1. **CENSUS AND CLASSIFY** the correction events by ROOT MECHANISM, over a bounded, named window.
     The candidate classes are already visible and they are NOT one class: *never derived* (the
     `100×`), *derived then went stale* (`headline 104 stale at 137`; `bar 319 vs 318`), *derived
     from an unsound basis* (`POSITIONAL-REF-SCAN` is *"a FLOOR and partly UNSOUND"*), *derived over
     the wrong population* (grammar text vs shipped artifacts, twice in two slices), *mis-derived*
     (`22/13/9` vs `23/13/10`). ⛔ **A gate aimed at the wrong one of these is worse than none.**
  2. Only then price a mechanism against the DOMINANT class, with a **measured false-positive rate**
     over the tracked leaves — not an argued one.
  3. ⭐ The sharpest candidate property found so far, recorded so the next session does not re-derive
     it: **every legitimate number in the `-0183` callout appears VERBATIM in a tracked artifact**
     (`176/1000` in `duality_break_sweep.txt`, `0/7994` in `own_corpus_selfreject_head.txt`), and
     `100×` appears in **none**. That is mechanically checkable and it is FP-bounded by construction,
     because this repo's instruments already print greppable headlines (`SELF-REJECT:`,
     `ACCEPT-SET-LEDGER:`, `DIRECTED-GENERATION:`, `ARROW-POSITION-CENSUS:`). ⚠️ It is a **candidate**,
     not the design: it must survive step 1, and its FP rate over legitimately-quoted historical
     numbers is unmeasured.
- ⚠️ **PRIORITY, stated honestly**: this is a governance lane and the standing directive is to prefer
  product/parser work ([[feedback_prefer_feature_work_over_governance_lanes]]). It is parked behind
  the SV lane and behind `H.16.6f`/`H.23`. It is recorded because the director delegated the decision
  and *"nothing important may live only in this conversation"* — not because it outranks the frontier.

#### ✅✅ `.15` — a "nothing checks X" claim is a CENSUS claim, and it is now CHECKED (**`done`**, `PGEN-DOCTRINE-GAP-OWNERSHIP-0296`, 2026-08-26 — the **30th registered doctrine** `GAP-CLAIM-CENSUS`; OPS/ENFORCER tier, ZERO Rust / grammar / codegen / generated bytes; opened 2026-08-25 by `PGEN-SV-CORPUS-GRAD-0294`, UN-PARKED 2026-08-26 by my own sequencing ruling)

⭐⭐⭐ **SEQUENCING RULING 2026-08-26 (mine to make — `feedback_sequence_approved_work_yourself`): THIS COMES OFF THE PARK, AND THE ARGUMENT IS A THIRD OCCURRENCE RATHER THAN A PREFERENCE.**
It was parked behind the SV lane as governance work. That classification is now refuted by measurement: the habit has produced a WRONG PUBLISHED CLAIM **three times**, and every one of them landed inside the SV lane and cost a correction:

| # | claim | where it was published | cost |
|---|---|---|---|
| 1 | *"the `verilog_2005` cert denominator counts 302 rules no v2005 input can reach — nothing crosses the two"* | `SV-CORPUS-GRAD.13e.8` | leaf CLOSED AS REFUTED (`-0294`) |
| 2 | *"nothing in the doctrine set would have blocked it"* | `-0305`'s findings callout | retracted at 4 sites (`-0306`) |
| 3 | *"nothing suppresses a rule-level `@sample` on an interior hop"* | `ENGINE-UNIVERSAL-SERVICES.52`, first draft | caught pre-publication, 2026-08-26 |

⇒ **it is not a governance lane adjacent to the SV lane — it IS the SV lane's recurring failure mode.** Three occurrences in three days, two of which reached a commit and one of which reached a TRACKED CONTRACT.

⭐ **AND OCCURRENCE 3 IS THE ONE THAT SETTLES THE DESIGN QUESTION, because checking it was not merely defensive — it FOUND THE REAL MECHANISM.** The false claim would have sent a fix at a mechanism that does not exist; the census instead produced the exact predicate (`needs_rule_body_descent` keying on `o{i}`/`q` segments and therefore missing an `s{i}` hop) that `.52` then fixed. ⇒ the guard's value is not that it blocks bad prose; it is that **it converts a vague suspicion into a located defect**, which is the strongest possible argument for making it blocking rather than advisory.

⛔ **MY EARLIER RULING STANDS AND IS UNCHANGED — STAGED-DIFF-SCOPED, never over the existing corpus.** 76 of 108 pre-existing claims carry no census; a blocker firing on all of them teaches bypass (the 91 %-FP shape `.6`/`.12` refused). Scoped to claims a commit ADDS, it fires on nothing today — the same condition `SV-CORPUS-GRAD.13e.11` established for a safe blocking guard: *a guard is safe to make blocking over its whole population exactly when the fix precedes it*, and here scope-limiting plays the part the fix plays there.
⭐ **Sequenced AFTER `ENGINE-UNIVERSAL-SERVICES.53` and `SV-CORPUS-GRAD.13e.11`(b)+(c)** — both are smaller, both are already designed, and `.13e.11`(b) builds the same kind of staged guard, so its wiring is the prior art this one should reuse rather than reinvent.

**ROUTING EVIDENCE** (`ROUTING-EVIDENCE` doctrine — what was MEASURED before routing):

- ⛔ **THE DEFECT IS MINE AND IT REPEATED INSIDE THREE DAYS.** `-0288` published *"`verilog_2005`
  faithfulness is checked by NOTHING"* (false — 85 curated conformance cases; retracted by `-0289`).
  `-0290` then opened `SV-CORPUS-GRAD.13e.8` with *"nothing crosses the two"* (false —
  `gather_verified_profile_proof_covered_rules`, shipped as `VERILOG-2005-PROFILE.6.7`; retracted by
  `-0294`). **Same form, same subject, one commit after the lesson that names it.**
- **MEASURED over the closed tracked population** — 320 files under `docs/tasks/`:

  ```text
  gap-claim hits: 108 across 21 files
    BACKED by a census command in the same leaf section:   32  (30 %)
    UNBACKED:                                              76  (70 %)
  ```

  Phrase distribution: `no instrument` 28 · `nothing in the repo` 18 · `nothing reads` 15 ·
  `nothing checks` 13 · `nothing compares` 7 · `nothing measured` 7 · `nothing watches` 6 · and six
  rarer spellings. Concentrated in `SV-CORPUS-GRAD` (17), `DONE-BAR` (11),
  `ENGINE-UNIVERSAL-SERVICES` (10).
- ⭐ **CORPUS PRESSURE IS DECISIVELY ABOVE THIS REPOSITORY'S OWN BAR.** `GENERATED-LINT-CORRECTNESS.4`
  refused a widening at **2 of 304** and `.7` at **0 of 307**. 108 instances is three orders above
  either. The trigger is real, not a hypothetical.
- ⛔⛔ **MY RULING, AND IT IS MINE — NOT A DIRECTOR CALL** (recorded because I wrongly escalated it
  once): **the check is warranted, and it must be SCOPED TO THE STAGED DIFF, never to the existing
  corpus.** A blocker that fires on 76 pre-existing claims is unusable and teaches bypass — the
  failure mode `GENERATED-LINT-CORRECTNESS.6`/`.12` refused at 91 % FP. `DOCTRINE_ENFORCEMENT.md` §4
  rule 5 already prescribes the shape: a check about a *change* reads `git diff --cached` and exempts
  what it does not govern. Scoped that way the steady-state fire rate is *"claims added by this
  commit that quantify over the tree without a census"* — which for a compliant commit is **0**.
- ⚠️ **THE 70 % IS NOT A FALSE-POSITIVE RATE AND MUST NOT BE QUOTED AS ONE.** It is the rate at which
  existing claims lack an in-leaf census; whether each is *wrong* is unmeasured, and two audited so
  far were wrong. Pricing the true FP rate needs a sample adjudicated against the tree — owed by (b).
- **Owed:** (a) `scripts/check_gap_claims.sh`, staged-diff-scoped, on the `check_diagnosis_evidence.sh`
  evidence archetype (a pasted `grep -rn` / `git ls-files` / census invocation inside the same leaf
  section), registered in `check_doctrines.sh` with its refusals proven by a probe including a GREEN
  control; (b) adjudicate a sample of the 76 to price the real FP rate before the check is made
  blocking; (c) decide whether `CHANGES.md` / `DEVELOPMENT_NOTES.md` join the governed population.
- ⚠️ **PRIORITY, stated honestly**: governance lane, and the standing directive prefers product/parser
  work ([[feedback_prefer_feature_work_over_governance_lanes]]). **Parked behind the SV lane** — it is
  recorded so it survives the session, not because it outranks `SV-CORPUS-GRAD.13e.9`.

##### SHIPPED — `GAP-CLAIM-CENSUS`, the 30th registered doctrine (`scripts/check_gap_claims.sh`)

All three owed items are discharged: **(a)** the enforcer, **(b)** the adjudicated sample that prices the real false-positive rate, **(c)** the governed-population ruling. Everything below is re-derivable from the TRACKED harness `docs/tasks/artifacts/doctrine_gap_ownership/gap_claim_census_replay.sh` — deliberately tracked, because `ENGINE-UNIVERSAL-SERVICES.54` (`-0097`) closed on a leaf whose evidence was cited from an IGNORED path.

⛔ **The harness READS the predicate out of the shipped enforcer rather than re-typing it**, and REFUSES (exit 2) if it cannot — an empty regex matches every line, so that failure would have been silent in the alarming direction.

**(a) THE ENFORCER.** Staged `docs/tasks/*.md` only; ADDED lines only; judged against the **INDEX** content, never the worktree — a worktree edit that adds the census without staging it would otherwise discharge a claim that ships bare. A claim is `nothing` + a **transitive verb**, never the bare word: `nothing` occurs on **1 492** lines of `docs/tasks/` here and `nothing else` / `nothing shipped` / `nothing technical` are not claims about the tree.

```text
git grep -icE '(^|[^[:alnum:]])nothing([^[:alnum:]]|$)' -- 'docs/tasks/*.md' | awk -F: '{s+=$2}END{print s}'
  -> 1492      # the house idiom — why the predicate keys on the VERB, not the word
scripts/check_gap_claims.sh --all
  -> 242 claim line(s) across 29 file(s), 81 with no census in their own section
```

Discharge = an enumerating command in the claim's **own heading section**, or an explicit `census: not run (<why>)`. Section boundaries are **any ATX heading**, deliberately: `.8` measured that THREE leaf-heading conventions are in live use, so a predicate keyed on a leaf-id grammar would silently mis-scope, while *"nearest heading above"* needs no such grammar. ⛔ The bare word *census* is NOT a discharge — it occurs inside the claims themselves (*"nothing re-runs this census"*), so accepting it would let a claim discharge itself.

**⭐⭐ CALIBRATED BY REPLAYING REAL HISTORY, NOT AN IMAGINED PHRASING** — the `ROUTING-EVIDENCE` first-cut defect, whose check would not have caught its own founding incident. Each commit's ADDED lines are judged against **that commit's own version of the file**, which is what the hook sees:

```text
docs/tasks/artifacts/doctrine_gap_ownership/gap_claim_census_replay.sh 150
  claim lines ADDED to docs/tasks : 95, in 50 of 150 commits
  WOULD HAVE BLOCKED              : 16 line(s), in 14 of 150 commits (9.3%)
docs/tasks/artifacts/doctrine_gap_ownership/gap_claim_census_replay.sh 150 --narrow-discharge
  WOULD HAVE BLOCKED              : 30 line(s), in 22 of 150 commits (14.7%)   # the CONTROL
```

⭐ **BOTH founding incidents REPRODUCE, and getting there required RETRACTING a claim I made in the first draft of this leaf.** `-0290`'s real sentence — *"**no instrument in the tree crosses the two**"*, `493f5ea6` — is in the fired set, pinned verbatim by `--self-test` **arm 3**.

⛔⛔ **RETRACTION, within this leaf, of *"occurrence 1 carries no `nothing`-token at all"*.** I wrote that after reading `-0288`'s **task-tree** lines and its MEMORY.md **headline**, and stopped. `git show 3aba01ec:MEMORY.md` carries the literal sentence *"**faithfulness — every reachable rule DERIVABLE from IEEE 1364-2005 — is checked by nothing**"*. The retraction's paraphrase was FAITHFUL; my correction of it was the error. ⇒ **I made this doctrine's own defect while documenting this doctrine** — a universal claim (*"no token at all"*) published on a partial census of the sites. ⭐ **And it was worth catching, because the sentence was a real GAP IN THE PREDICATE, not a phrasing curiosity**: the doctrine is named after [[an-x-is-checked-by-nothing-claim-is-a-census-claim]] and its first cut could not match that card's own title. The passive voice is now a first-class alternative — measured **22** `by nothing` occurrences in `docs/tasks/`, of which the participle form catches 16 real claims and correctly skips the **2** `and by nothing else` idioms (`and` does not end in `ed`) — and **arm 3b** pins `-0288`'s sentence verbatim while **arm 3c** pins the idiom as a non-claim.

⚠️ **Occurrence 1 is STILL only half-caught, and the remaining half is structural, not lexical**: the sentence lives in `MEMORY.md`, which is out of the governed population for the byte-cap reason below. The predicate would catch it in a leaf; the population does not include the file it was written to.

**(b) THE 13 ADJUDICATED ONE BY ONE — the real false-positive rate, which the `70 %` was never a measure of.** Rows captured in `gap_claim_census_replay_blocked.txt`:

| verdict | n | what they are |
|---|---|---|
| **TRUE POSITIVE** | **12** | a genuine universal-absence claim with no census in its own section — six of them *"leg 3 NAMED, not met"* durability disclosures (`nothing WATCHES the self-rejection matrix`, `nothing re-runs this census`), four new-leaf headings, two mid-leaf assertions |
| FALSE POSITIVE — **quotation** | 1 | `c8b11f3a` quotes an EARLIER claim (*"nothing routes to them, nothing consumes them"*) rather than making one |
| FALSE POSITIVE — **states its own count, not the command** | 3 | `ab9cee04` cites `.7`'s published **224 / 29**; two `.13c.2x.8` revisions cite **59** and **63** published tuples |

⇒ **75 % true-positive** on the strict reading, the inverse of the 91 %-FP shape `GENERATED-LINT-CORRECTNESS.6`/`.12` refused. ⚠️ **Those 3 are scored as false positives CONSERVATIVELY.** Under this doctrine's own standard a number you cannot RE-RUN is not a census — the whole point is re-derivability — so the honest range is **12 to 15 of 16**, and I report the low end. ⚠️ **The 81-of-242 is NOT a false-positive rate and must not be quoted as one** — it is the rate at which EXISTING claims lack an in-section census; whether each is *wrong* is unmeasured, and the two audited so far were both wrong.

⭐ **ONE MEASURED FALSE POSITIVE WAS FIXED RATHER THAN ACCEPTED**: the widening's first replay reported *"which `grep -nw` shows is referenced by nothing"* — a claim whose census sits **in the same sentence** — because the discharge list carried `grep -r/-c/-l/-o` but not `grep -n`, by far the commonest spelling here. Added, and the block count fell 17 → 16. ⚠️ A **bare** `grep` is still not a discharge: `check_diagnosis_evidence.sh` measured that it matches 16 boxes on prose such as *"verified by grep"*. The FLAG is what makes it an invocation.

⚠️ **QUOTATION IS AN ACCEPTED, PRICED RESIDUAL — 1 in 13, measured rather than guessed.** It is the self-referential shape this repository already priced at one site and deliberately did not mechanize around ([[reference_self_referential_assertion_is_unsound]]); narrowing the predicate around quotation marks re-introduces exactly the phrasing-guessing that made `ROUTING-EVIDENCE`'s first cut miss `-0015`. The discharge is one pasted command.

**(c) THE GOVERNED POPULATION IS `docs/tasks/*.md` ONLY — a MEASURED ruling, not taste.** Over the same 150 commits, counting added claim lines and whether the SAME commit also added one to `docs/tasks/`:

| surface | added | co-occurring with a `docs/tasks` claim | escaping |
|---|---|---|---|
| `CHANGES.md` | 20 | **18 (90 %)** | 2 |
| `DEVELOPMENT_NOTES.md` | 21 | **16 (76 %)** | 5 |

Governing them would double-fire on one underlying claim in the large majority of cases and buy **7 additional lines over 150 commits**. ⚠️ Residual, named rather than implied: those 7 escape. ⛔ `MEMORY.md` is excluded **STRUCTURALLY**, not by preference — layer A is byte-capped at 7 168 B by `MEMORY-ARCH`, so requiring a pasted census inside it would set two doctrines against each other; a layer-A claim's census belongs in the leaf its pointer names. **This is where occurrence 1 landed**, which is the second reason it is a miss.

##### Acceptance checklist (`TOOLBOX.md`)

- [x] **ROOT CAUSE (WHY + WHERE)** — ops/build-flow family (no parse to trace, no run to sample, no
  compiler error: the subject is the repository's own tracking surface). **WHY**: an *"X is checked by
  NOTHING"* sentence is a **universally quantified claim over the whole tree**, and it is false the
  moment ONE reader exists — but nothing in the enforcement set asked for the enumeration behind it,
  so it could be published on two functions read and a third never looked for. **WHERE**: the write
  path into `docs/tasks/*.md`, unguarded at commit time. Tool-backed and not read off the source:
  `git ls-files 'docs/tasks/*.md'` closes the population at **321** tracked files;
  `scripts/check_gap_claims.sh --all` measures **205** claim lines across 29 of them, **74** with no
  census in their own heading section; `git log -S` located the founding commits `3aba01ec` (`-0288`)
  and `493f5ea6` (`-0290`), and reading `493f5ea6`'s own added lines produced the LITERAL sentence
  (*"no instrument in the tree crosses the two"*) rather than the retraction's paraphrase — which is
  what the predicate was then calibrated against. `bash -n scripts/check_gap_claims.sh` clears.
- [x] **ADDRESSED (verified)** — the enforcer exists, refuses, and its refusals were OBSERVED firing:
  `scripts/check_gap_claims.sh --self-test` -> **arms=10/10**, and the arm set is chosen so no arm can
  pass vacuously.

  | arm | proves |
  |---|---|
  | 1 GREEN control | a claim WITH a census in its own section PASSES — without this every red arm could be red for the fixture, not the missing census |
  | 2 RED control | the SAME fixture with the census removed is reported |
  | 3 founding incident | `-0290`'s sentence, verbatim, fires |
  | 4 scope | a pre-existing claim (not in the added set) is INERT — the whole staged-scope ruling |
  | 5 idiom | `nothing else` / `nothing shipped` / `nothing technical` do NOT fire |
  | 6 disclosure | `census: not run (<why>)` discharges — an honest *not run* is legal |
  | 7 section scope | a census in a DIFFERENT section does NOT discharge |
  | 3b passive voice | `-0288`'s *"is checked by nothing"* — the sentence this doctrine is NAMED after — fires; the first cut did NOT catch it |
  | 3c the idiom | *"and by nothing else"* does NOT fire, so the passive widening costs no false positives |
  | 8 hunk arithmetic | added lines resolve to NEW-file positions (`2 3 11`), the one step with no natural oracle |

  ⛔ **Arms 1-7 call the classifier DIRECTLY, so they bypass the git plumbing the real check runs on**
  — the INDEX read and the `-U0` hunk resolution — and arm 8 tests that arithmetic in isolation.
  Nothing tested the two TOGETHER on a genuinely staged file, so a second, END-TO-END control exists:
  `docs/tasks/artifacts/doctrine_gap_ownership/gap_claim_census_arms.sh` -> **arms=3/3**, RED (an
  unbacked staged claim is reported, rc 1, line quoted) / GREEN (the SAME file, one census line
  appended, rc 0) / HYGIENE. ⭐ It runs against a THROWAWAY `GIT_INDEX_FILE` under `rust/target/`, so
  it never touches the caller's index — and the HYGIENE arm ASSERTS that rather than assuming it.
  ⚠️ **The first cut of this control reported `arms=2/3` and then died with `GIT_INDEX_FILE: unbound
  variable`**: the hygiene arm judged the worktree while the fixture was still in it, and the `EXIT`
  trap read the variable the arm had just unset — a control that leaves residue while looking like it
  ran. Both fixed, and both recorded in the script's own header.

  Registered and reported by the driver: `bash scripts/check_doctrines.sh` -> **ALL 30 enforced
  doctrines PASS** (was 29), `<meta:mirror>` agrees at 30, `<meta:book-count>` agrees at 30, and the
  driver's own scope note now names `GAP-CLAIM-CENSUS` among the **10 of 30** staged-scope doctrines —
  derived from the enforcer's source, so the classification is not a hand-maintained list.
- [x] **NO REGRESSION** — this change touches **zero** Rust, grammar, codegen or generated bytes, so
  the clippy flow is not applicable and is skipped deliberately rather than silently:
  `git status --porcelain` lists only `scripts/`, `DOCTRINE_ENFORCEMENT.md`, `docs/book/`,
  `docs/tasks/` and the live docs. `scripts/check_generated_reproducibility.sh` -> **OK, 11 artifacts
  unmoved, emission sources unmoved since `df30690` — tier 2 last proved them byte-identical to
  HEAD**. All 30 doctrines PASS with the change staged. ⛔ **The new guard was replayed against the
  history it will govern before being made blocking**: over the last 150 commits it would have blocked
  **11**, leaving **139 untouched**, and the 13 blocked lines were adjudicated individually rather than
  sampled. The two pre-existing `command not found` lines the driver prints are `.16`, NOT this change
  — reproduced at HEAD (`git show HEAD:scripts/check_doctrines.sh` -> same two errors, line 79); this
  row shifted their reported line to 80 and added neither a backtick nor a third `|` (audited: the
  registry row carries **2** pipes and **0** backticks).
- **`promotion: declined (the transferable lesson — an "X is checked by nothing" claim is a CENSUS claim — is ALREADY the retrievable card [[an-x-is-checked-by-nothing-claim-is-a-census-claim]]; this slice mechanizes that card rather than adding a second one)`**

##### What this leaf does NOT close, stated rather than implied

- **Occurrence 1 is HALF caught, and the remaining half is STRUCTURAL.** Its literal sentence — *"is
  checked by nothing"* — now fires (arm 3b), but it was written to `MEMORY.md`, which is out of the
  governed population for the byte-cap reason above. The predicate would catch it in a leaf; the
  population does not include the file it was written to. ⚠️ And the predicate still catches the FORM
  the class usually takes, not every possible spelling of an absence claim.

  ⭐⭐ **THIS VERY BULLET WAS BLOCKED BY THE DOCTRINE IT DOCUMENTS, ON ITS OWN COMMIT** — the quotation
  false-positive class, priced at 1 in 16 above, firing on the leaf that priced it. It is discharged
  the way the tool tells an author to discharge it, with the census that actually backs the sentence:

  ```text
  git show 3aba01ec:MEMORY.md | grep -c 'is checked by nothing'                -> 1
  git grep -l 'is checked by nothing' 3aba01ec -- 'docs/tasks/*.md' | wc -l    -> 0
  git grep -l 'is checked by nothing' -- 'docs/tasks/*.md'                     -> 2 (both QUOTING it, today)
  ```

  ⇒ at the founding commit the sentence was in layer A and in **no** task leaf, which is exactly the
  structural half named above; the only task-leaf copies today are this leaf and `SV-CORPUS-GRAD`
  quoting it in retrospect. ⛔ **The first draft of this very census block wrote `-> 0 files` for the
  third line and the command returns `2`** — I typed the number I expected beside a command I had not
  re-run, which is `DERIVED_STATE_CONTAINMENT` R1/R3 in its smallest possible form, inside the census
  that exists to stop it. Caught by running it.
  ⛔ Recorded rather than worked around: the alternative was to weaken the predicate so that quoting a
  claim stops firing, and that is the phrasing-guessing `ROUTING-EVIDENCE`'s first cut died of.
- **Evidence archetype, with the bound every evidence check carries**: this verifies the census was
  RECORDED, never that it was RUN, that its population was the right one, or that its conclusion was
  correct. It would have caught `-0290`, because `-0290` recorded no census at all; it cannot tell a
  good census from a bad one.
- **The 74 pre-existing unbacked claims are REPORTED, not enforced** (`--all` is advisory by
  construction). Whether to work that backlog down is a separate decision; blocking on it teaches
  bypass, which is the whole scoping ruling.

#### ⚠️ `.17` — **THIS REPO'S COPY OF A PORTABLE STANDARD CARRIES NO ADOPTION IDENTITY: nothing here records what it was imported from, who owns changes to it, or whether a local edit forks it** (`todo`, opened 2026-08-26 by `PGEN-CLAIM-VERIFICATION-0001`; ⛔ **RE-SCOPED IN-LANE the same day — its first framing rested on an ORIGINATION claim that identical digests cannot support**)

**ROUTING EVIDENCE** (`ROUTING-EVIDENCE` doctrine — what was MEASURED before routing):

⛔⛔ **READ THE CORRECTION FIRST, BECAUSE THE ORIGINAL FRAMING OF THIS LEAF WAS WRONG AND IT WAS
WRONG IN THIS DOCUMENT'S OWN SIGNATURE WAY.** It opened by asserting that
`docs/CLAIM_VERIFICATION.md` **IS** the canonical template another project imported, on the evidence
that this repo's pre-edit digest
(`8f194574ec718749e90a402cf6e4dc6650e44f6185dd194853a54482eef68643`) equals the digest that
project's adoption note records for its import. **Identical digests prove CONTENT IDENTITY, not
ORIGINATION.** A competing hypothesis predicts exactly the same observation — *both repositories
imported the same director-selected template from a third source* — and the adopter's own wording
(*"the **director-selected** template"*) is at least as consistent with it. ⇒ the origination claim
is WITHDRAWN. What is measured is narrower and still useful: **two repositories hold byte-identical
copies of the same standard, and this one carries no record of where its copy came from.**
⭐ This is verbatim `docs/CLAIM_VERIFICATION.md` §3 leg 2 (*"if your evidence is consistent with both
hypotheses, you have not tested — you have illustrated"*), committed by the slice that ADDED that
sentence to the file.

⭐ **THE MEASURED, IN-SCOPE GAP.** The other project's copy opens with a delimited
`…-LOCAL-ADOPTION` block declaring: the authority, the authoritative path, the import digest, an
explicit *independence* statement, its local scope, and the name of the checker that enforces it.
**This repository's copy has none of that** — no adoption note, no import record, no statement of
who owns changes to it here, no marker separating a neutral body from local additions:

```text
$ head -1 docs/CLAIM_VERIFICATION.md
# Claim Verification — verify a claim three ways before you publish it     # ← straight into the body
```

⇒ a reader of THIS repository cannot tell whether the file is locally authored, locally owned, or an
import that a local edit has just forked. That is a `docs/CLAIM_VERIFICATION.md` §5B failure about
**our own** artifact, which is what makes it ours to fix.

⛔ **AND THE SLICE THAT OPENED THIS LEAF EDITED THE FILE, so the question is live rather than
theoretical.** `PGEN-CLAIM-VERIFICATION-0001` changed the body (digest now
`fdcd1ed49b22b48a3662849ee35d7870ed4be4534dbc165c33c86288eaf70622`). Whether that is *"this project
improving its own copy"* or *"this project forking a shared standard"* is exactly the question no
artifact in this repository can currently answer.

⛔⛔ **EXPLICITLY OUT OF SCOPE: another repository's state.** The first framing carried an acceptance
item to *"notify the sibling that the template moved"* and tracked its re-import as an obligation
here. ⇒ **DROPPED.** The standing rule is that cross-project standards are COPIED IN, not referenced
across repos (`LIVE_DOCUMENT_SIZE_CONTAINMENT.md`'s own adoption note states the same discipline and
measures its own leakage). A leaf in this tree that tracks another project's adoption state is that
rule inverted. If the director wants the other project informed, that is a message from the
director's desk, not a durable obligation in PGEN's task tree.

**Acceptance:** (a) give this repository's copy an adoption note in a delimited block — what it was
imported from *if that is known and evidenced*, ⛔ **stated as provenance and never as origination
unless something other than a digest establishes it**; who owns changes here; and whether local
edits are permitted or must be proposed upstream; (b) if the note pins an import by digest, **gate
the pin** (§5B) — a pin only a human re-reads is §6's *"do not edit by hand"* comment wearing a hash,
and this leaf exists because that gap was measured, not imagined; (c) ⚠️ honest bound to keep
attached: **the provenance of this repository's copy is currently UNKNOWN**, and (a) must not invent
one. *"Imported from a director-selected template; origin not recorded"* is an honest note; naming a
source on digest equality alone is the error this leaf was re-scoped for.

#### ✅✅ `.16` — the doctrine registry EXECUTES its own descriptions, and the guard added when this last happened cites the defect as prior art without checking for it (**`done`**, `PGEN-DOCTRINE-GAP-OWNERSHIP-0297`, 2026-08-26 — (a)+(b)+(c) all shipped; OPS/ENFORCER tier, ZERO Rust / grammar / codegen / generated bytes; opened 2026-08-25, opened 2026-08-25 by `PGEN-SV-CORPUS-GRAD-0300`)

**ROUTING EVIDENCE** (`ROUTING-EVIDENCE` doctrine — what was MEASURED before routing, and whether it
reproduces outside the family it is being sent to):

- **MEASURED on the tree at HEAD, by running the enforcer.** `scripts/check_doctrines.sh` line **76**
  — the `CORPUS-KEY-INTEGRITY` row, registered **two commits ago** by `CORPUS-KEY-AUDIT.2` /
  `PGEN-CORPUS-KEY-AUDIT-0004` — carries **4 UNESCAPED backticks** inside a double-quoted bash
  string. Bash therefore runs `` `match` `` and `` `syntax error` `` as **command substitutions** at
  array-definition time:
  ```
  scripts/check_doctrines.sh: line 77: match: command not found
  scripts/check_doctrines.sh: line 77: syntax: command not found
  ```
- ⛔ **AND THE SUBSTITUTION SWALLOWS THE TEXT.** The published doctrine description — the sentence a
  reader uses to learn what the doctrine actually checks — loses the words outright. Measured, source
  against report:

  | | text |
  |---|---|
  | SOURCE | ``sitting at `match` -- the STRONGEST verdict`` |
  | **PRINTED** | `sitting at  -- the STRONGEST verdict` |
  | SOURCE | ``UNTAGGED, so `syntax error`, the most parse-relevant class`` |
  | **PRINTED** | `UNTAGGED, so , the most parse-relevant class` |

  ⚠️ Both surviving sentences still read as **grammatical English**, which is why re-reading the
  report cannot catch it — the missing token is exactly the technical term the sentence was written
  to carry.
- **MECHANISM ISOLATED with a matched control** (no path games, no script copy):
  ```
  A=( "X|sitting at `match` -- the STRONGEST verdict|s.sh" )    -> "sitting at  -- the STRONGEST verdict"
  B=( "X|sitting at \`match\` -- the STRONGEST verdict|s.sh" )  -> "sitting at `match` -- the STRONGEST verdict"
  ```
- ⛔⛔ **IT FAILS IN THE PASSING DIRECTION AND THE DRIVER EXITS `0`.** `bash scripts/check_doctrines.sh
  >/dev/null 2>&1; echo $?` → **0**, "ALL 27 enforced doctrines PASS". The two errors go to stderr,
  which every caller that greps the verdict discards. `bash -n` is clean — this is valid bash doing
  exactly what it was told.
- ⭐⭐ **WHY THIS IS A DOCTRINE GAP AND NOT A TYPO.** This is the **fourth** escaping defect in this
  one registry, and the previous fix *names this exact one as its own prior art*: the comment at
  `check_doctrines.sh:84-88` says *"the third escaping defect this registry has had, after the
  unescaped backticks that ran a command substitution (`-0241`)"* — and the guard it introduced
  checks the **`|` separator count** only. ⇒ **a fix that cites a defect class in its comment and
  guards a different member of that class is an invitation for the cited one to recur**, and it
  recurred within days. Census over the registry: **1 of 27 rows** affected, 4 backticks, all in the
  row added most recently.
- ⛔ **A LATENT INJECTION SURFACE, stated plainly and NOT as the headline.** A doctrine description is
  prose written by whoever registers a doctrine; today an unescaped backtick runs a nonexistent
  command harmlessly. The population is tracked, reviewed text, so this is a *robustness* argument
  for the guard, not a live security finding — recorded so the guard is scoped to refuse rather than
  to sanitize.
- **REPRODUCES OUTSIDE THIS FAMILY:** yes — nothing about it is SystemVerilog. The registry is the
  repo-wide enforcement driver; every family's commits run through it.
- **Owed:** (a) escape the 4 backticks on line 76 (2-character-class fix, no behaviour change);
  (b) add a `<meta:registry>` arm beside the existing `|`-count arm that refuses **any** unescaped
  backtick in a registry row, with the refusal observed firing and a GREEN control on the real
  registry — the guard the `-0241` fix described but did not build; (c) decide whether the arm
  should also refuse `$(`, `${` and `\` sequences, i.e. guard the CLASS rather than this member —
  the whole point of (b).
- ⚠️ **PRIORITY, stated honestly**: governance lane, and it blocks nothing — the enforcer's
  **verdicts** are unaffected, only its printed descriptions. **Parked behind the SV lane** under the
  standing lane lock; recorded so it survives the session, not because it outranks
  `SV-CORPUS-GRAD.13e.10`(c). ⭐ (a) alone is minutes and could ride any future commit that touches
  the registry; (b) is the part that stops the fifth occurrence.

##### SHIPPED — (a) the escape, (b) the guard the third fix described and did not build, (c) the CLASS

**(a) THE FOUR BACKTICKS ARE ESCAPED.** Measured before → after on the driver's own output:

```text
BEFORE  scripts/check_doctrines.sh: line 80: match: command not found
        scripts/check_doctrines.sh: line 80: syntax: command not found
        report: "sitting at  -- the STRONGEST verdict"
        report: "UNTAGGED, so , the most parse-relevant class in the corpus"
AFTER   (no `command not found` lines at all)
        report: "sitting at `match` -- the STRONGEST verdict"
        report: "UNTAGGED, so `syntax error`, the most parse-relevant class in the corpus"
```

The edit is provably escaping-only: the patch asserts that un-escaping the result reproduces the original row byte-for-byte, so no prose changed.

**(b) `<meta:registry-escaping>`, and the one design decision that makes it work.** ⭐⭐ **It reads the driver's UNEXPANDED SOURCE TEXT, never `"${DOCTRINES[@]}"`.** By the time the array can be inspected bash has already performed the substitution — the array holds the *result*, with the offending words gone — so a guard over the expanded values is blind to this defect **by construction**. It would be inspecting the evidence after the fire.

**(c) IT GUARDS THE CLASS, WHICH IS THE WHOLE COMPLAINT AGAINST THE THIRD FIX.** Refused: any unescaped `` ` ``, `$` or `"`, and any backslash escaping something other than those four — the complete set of characters that stay live inside a double-quoted shell string. Censused over the registry before wiring it:

| | measured |
|---|---|
| rows | **30** |
| rows with an UNESCAPED backtick | **1** (line 76, `CORPUS-KEY-INTEGRITY`, 4 of them) — fixed by (a) |
| rows with CORRECTLY escaped backticks | **4**, carrying **28** `` \` `` between them |
| unescaped `$` anywhere | **0** |
| unescaped `"` anywhere | **0** |
| backslash sequences in use | only `\"` (**8**) and `` \` `` (**28**) |

⇒ the convention is already known and followed, so after (a) this arm **fires on nothing** — the `SV-CORPUS-GRAD.13e.11` condition for a guard that is safe to make blocking: *the fix precedes it*.

⛔⛔ **THE RED CONTROL FOUND A REAL DEFECT IN MY OWN GUARD, AND IT WAS THE EXACT FAILURE DIRECTION THIS TREE EXISTS TO CLOSE.** The first cut read `"${BASH_SOURCE[0]}"` — a *relative* path — and the driver `cd`s to `$ROOT` before that line runs. Executed from any other cwd, the arm read a path that no longer resolved, inspected **nothing**, and reported **✓ PASS**. The RED arm caught it because the arm asserts the *verdict*, not merely a nonzero exit: `RED FAILED — the founding defect was not refused` while `RED-2` simultaneously confirmed the substitution really had run. Two fixes, both now in the driver: `SELF` is resolved to an absolute path **before** the `cd`, and an unreadable `SELF` is a **hard FAIL** rather than a pass — the same *"a check that cannot inspect its subject must SAY SO"* rule the mirror and book-count arms already follow.

**PROVEN, not asserted** — `docs/tasks/artifacts/doctrine_gap_ownership/registry_escaping_arms.sh` → **arms=3/3**, running the REAL driver against a mutated copy so what is proven is the shipped code:

| arm | proves |
|---|---|
| GREEN | the shipped registry passes the arm — without it, RED proves only that *something* about the copy is refused |
| RED | re-un-escaping the founding row is REFUSED (`✗ <meta:registry-escaping>`) |
| RED-2 | the substitution really RAN in the fixture (`command not found` in the log), so RED is refusing the founding defect and not an artefact |

Observed separately, because a refusal nobody has seen fire is not a refusal:

```text
$ ( cd .. && bash pgen/scripts/check_doctrines.sh )   # cwd-independence, the defect above
  ✓ PASS  <meta:registry-escaping> — no registry row carries unescaped ` $ or " (30 rows)
$ SELF=/nonexistent/driver.sh                          # the unreadable-subject branch
  registry-escaping: cannot read the driver source at /nonexistent/driver.sh, so the registry text was NEVER inspected
  ✗ FAIL  <meta:registry-escaping> — the driver could not read its OWN source; the registry was not inspected
```

⚠️ **HONEST LIMIT, stated rather than discovered**: this checks the registry ROWS, not the driver's other double-quoted strings, and it cannot tell a deliberate substitution from an accidental one — it refuses both. In a registry of reviewed prose that is the right trade; a row that genuinely needs a computed value should build it above the array, not inside a description.

##### Acceptance checklist (`TOOLBOX.md`)

- [x] **ROOT CAUSE (WHY + WHERE)** — ops/build-flow family (shell, not Rust: no parse to trace, no run
  to sample, no compiler error). **WHY**: a doctrine description is prose inside a **double-quoted**
  bash string, where `$` `` ` `` `\` and `"` remain live, so bash performs command substitution at
  **array-definition time** and the description silently loses the substituted words. **WHERE**:
  `scripts/check_doctrines.sh:76`, the `CORPUS-KEY-INTEGRITY` row — 4 unescaped backticks, located by
  a per-row census over the source (`git ls-files` closes the population; the census masks `` \` ``
  before counting, which is what separates the 1 offending row from the 4 that are correctly escaped).
  ⛔ Bash attributes the error to **line 80**, the array's closing line, not to line 76 — which is why
  reading the reported line number sends you to the wrong row. `bash -n scripts/check_doctrines.sh` is
  clean before and after: this is valid bash doing exactly what it was told.
- [x] **ADDRESSED (verified)** — before → after on the driver's own output, quoted verbatim above: the
  two `command not found` lines are gone and both swallowed terms (`` `match` ``, `` `syntax error` ``)
  are back in the printed description. The guard that stops the fifth occurrence is wired and its
  refusals were OBSERVED firing: `registry_escaping_arms.sh` → **arms=3/3** (GREEN + RED + RED-2), plus
  the two branch probes above. ⛔ The RED arm **failed on its first run and was right to** — it exposed
  that my guard read a relative `${BASH_SOURCE[0]}` after a `cd` and therefore PASSED while inspecting
  nothing; both that and the unreadable-subject branch are fixed and each is now observed refusing.
- [x] **NO REGRESSION** — `bash scripts/check_doctrines.sh` → **ALL 30 enforced doctrines PASS**, exit
  0, and the run is now free of the two stderr lines it printed on every commit before this change.
  `<meta:mirror>` and `<meta:book-count>` still agree at 30 — no doctrine was added, so neither the
  registry size nor the published count moves. ZERO Rust, grammar, codegen or generated bytes are
  touched, so the clippy flow is not applicable and is skipped deliberately rather than silently;
  `scripts/check_generated_reproducibility.sh` reports the generated tree **byte-identical** to what
  HEAD's tracked source produces. The escaping edit is provably prose-preserving: the patch asserts
  that un-escaping its result reproduces the original row exactly.
- **`promotion: declined (the transferable half — a fix that cites a defect class in its comment and guards a different member of that class invites the cited one to recur — is this leaf's own ROUTING EVIDENCE and is recorded in the driver's header at the site where the next author will read it; a knowledge card would be a third copy of a lesson whose natural home is the code)`**

#### ⚠️ `.18` — **THE SLICE ID IS "THE UNIT OF COMMIT TRACEABILITY" AND IT IS NOT UNIQUE: 62 IDS NAME MORE THAN ONE COMMIT, AND THE COUNTER RESTARTED INSIDE THE ACTIVE LANE TWO COMMITS BEFORE THIS ONE** (`todo`, opened 2026-08-27 session #266 by `PGEN-ENGINE-UNIVERSAL-SERVICES-0100`, which met the collision while resolving a citation in the leaf it was resuming; ⛔ **PARKED behind the SV lane** per the standing lane lock — routed, sequenced later by me, not worked now)

**ROUTING EVIDENCE** (`ROUTING-EVIDENCE` doctrine — what was MEASURED before routing):

- ⛔ **THE CLAIM AND ITS CENSUS, IN THAT ORDER** (`GAP-CLAIM-CENSUS`: a *"nothing checks X"* claim is a
  CENSUS claim and carries its census in its own leaf). `docs/TASK_TREE.md` states that
  `PGEN-<FAMILY>-<NNNN>` slice ids *"remain the unit of commit traceability"*, and `COMMIT.md` §19
  makes the same id the sufficient identifier for a non-code slice. **Nothing derives, allocates or
  checks that id.** ⛔ **The first cut of this bullet claimed `grep` returns 0 hits and that is FALSE
  — corrected before publishing, by running it.** The pattern appears **8** times across the enforcement
  surface (`scripts/check_doctrines.sh` ×2, `.githooks/pre-commit` ×5, `.githooks/commit-msg` ×1) and
  **every one is a prose citation of a specific slice id inside a comment**, not a read of the id being
  committed. What is actually measured:
  `.githooks/commit-msg` is **13 lines** and enforces only that the subject *"begin with an
  identifier-shaped token"* (`^[A-Za-z][A-Za-z0-9._-]+`) — its own comment says it is *"permissive by
  design"* and that *"the work-unit id"* is merely *"expected"*; **0** of the **31** registered doctrines
  carry `SLICE` or a uniqueness term in their name; and **0** scripts derive a next id. ⇒ the id is
  written from memory and read by humans only.
- ⛔⛔ **AND IT HAS COLLIDED, AT SCALE.** Censused over the whole history at
  `71dde1e4`, three ways (below):

  | population | count |
  |---|---:|
  | commits in history | 3 136 |
  | commits carrying a `PGEN-…-NNNN` id | 1 773 |
  | commits carrying **no** id (older leaf-id-first convention) | 1 363 |
  | distinct ids | 1 680 |
  | ⛔ **ids naming MORE THAN ONE commit** | **62** |
  | ⛔ **commits NOT uniquely addressable by their own id** | **155** (8.7 % of id-bearing commits) |

  ⭐ The collisions are not an old-history artifact that stopped: `PGEN-ENGINE-UNIVERSAL-SERVICES-0096`
  and `-0097` were each issued **twice** — on 2026-08-26 (`5892ab46`, `df306908`) and again on
  2026-08-27 (`ebb0c34f`, `71dde1e4`) — after the counter had already reached `-0099` (`a8642b99`).
  The worst case is `PGEN-RGX-0078-0102`, on **six** commits.
- ⛔ **THE CONSEQUENCE IS ALREADY LIVE IN A TRACKED LEAF, WHICH IS WHY THIS IS NOT COSMETIC.**
  `ENGINE-UNIVERSAL-SERVICES.44` (a) cites *"`PGEN-ENGINE-UNIVERSAL-SERVICES-0096`"* as the commit that
  discharged it. That string now resolves to **two** commits — one about `.44` (a), one about a
  `.55` routing note — so the citation cannot be followed mechanically, and a reader who picks the
  first match reads the wrong commit. Every leaf citing a colliding id has the same defect.
  ⚠️ **NOT MEASURED, and this leaf must not assert it**: how many tracked leaf citations point at a
  colliding id. That census is step (a) below, and it is a `docs/tasks/`-wide grep, not a guess.
- ⭐ **THREE-WAY VERIFICATION OF THE COUNT ITSELF** (`feedback_verify_a_claim_three_ways_before_publishing_it`),
  because the first cut was WRONG in the flattering direction:
  1. **RE-DERIVE** — greedy-regex extraction of a leading id: **51** duplicate ids / 112 commits.
  2. **FALSIFY** — an independent whitespace tokenizer (no regex backtracking), leading token only:
     **51** / 112. It agreed, **and agreeing is what exposed the shared blind spot**: both methods
     read only the FIRST token, and **361** subjects carry their id later in the line. Widening the
     denominator to *"first id anywhere in the subject"* gives the honest **62 / 155**. ⇒ the two
     agreeing methods were both undercounting, exactly the *"a repeated pass repeats its own blind
     spot"* failure the directive names.
  3. **RED CONTROL** — the detector must be able to disagree with itself:
     `PGEN-ENGINE-UNIVERSAL-SERVICES-0099` reports **1** occurrence (unique) and `-0096` reports
     **2** (colliding), on the same run of the same code.
  ⛔ **HONEST BOUND**: `62` counts ids that collide, not commits that are *wrongly* numbered — some
  collisions may be a deliberate multi-commit slice (`PGEN-RGX-0078-0102` × 6 looks like one). The
  leaf's own first step must separate *"one slice, several commits"* from *"the counter restarted"*
  before any remedy is designed, because a gate that refuses the former would be wrong.
- **WHAT THIS LEAF OWES**: (a) census how many tracked `docs/tasks/` citations resolve ambiguously,
  and split the 62 into *deliberate multi-commit slices* vs *counter restarts*; (b) an ALLOCATOR —
  the next id for a family is `max(existing)+1` and is derivable by command, so the defect is that
  it is currently derived by memory; (c) decide whether a `SLICE-ID-UNIQUENESS` ratchet belongs in
  the registry, **scoped to the staged commit only** — `DOCTRINE-GAP-OWNERSHIP.15`'s ruling binds
  here: a blocker that fires on 155 pre-existing commits teaches bypass, which is the 91 %-FP shape
  `GENERATED-LINT-CORRECTNESS.6`/`.12` already refused.
  ⛔ **NOT a history rewrite.** `git` history is append-only in this repo; the 155 existing commits
  stay exactly as they are and any remedy is forward-only.
- **producer (tracked, so the claim can be re-derived rather than re-typed)**:
  `scripts/census_slice_id_collisions.sh`.

### ✅ `.20` — **14 OF THE 33 ENFORCED DOCTRINES ARE ABSENT FROM THE MAIN BOOK ENTIRELY, AND THE META-CHECK THAT LOOKS LIKE IT COVERS THIS ONLY HOLDS THE *NUMBER*** (`DONE` 2026-08-30 `-0300` — the STRUCTURAL half; the DEPTH half is measured, published and routed to `.22`, ⭐ **DIRECTOR-ORDERED**, opened 2026-08-30 by a DIRECT DIRECTOR QUESTION: *"Are all the 33 doctrines documented in the main mdbook?"*)

**The answer is NO, and it was measured rather than assumed.** Per-doctrine census over
`docs/book/src/*.md`, taking the registry itself as the population:

```
$ while IFS='|' read -r id _ _; do
    grep -rlqF -- "$id" docs/book/src/*.md || echo "ABSENT: $id"
  done < <(grep '^  "' scripts/check_doctrines.sh | sed 's/^  "//')
  ABSENT: DIAG-SEVERITY+DOCPATH · KNOWLEDGE-MAP · TASK-ACCEPTANCE · LESSON-PROMOTION ·
          DESIGN-PRIOR-ART · WAIVER-ROUTING · ROUTING-EVIDENCE · DESTRUCTIVE-TARGET-GUARD ·
          PARSER-BOOK-CURRENCY · SV-RULE-FIRE-PARTITION · SAMPLE-PROFILE-PARITY ·
          LR-ELIMINATION-RATCHET · GAP-CLAIM-CENSUS · CORPUS-CENSUS-CURRENCY
  -> present 19   absent 14
```

⛔ **FALSE-ABSENCE CHECKED, because a census is only as good as its spelling** (the lesson
`a-heading-census-is-only-as-good-as-the-heading-grammar` was paid for in this very tree). Each of
the 14 was re-searched under its component halves and under its enforcer's filename: `DOCPATH`
alone hits 1 page, every other probe hits **0**. So the 14 are genuinely absent, not differently
spelled.

⚠️ **AND THE 19 IS AN UPPER BOUND ON *DOCUMENTED*.** The census asks whether the ID appears at all.
A name in a list is a mention, not documentation — `gate-flow.md` names **16** doctrine-shaped IDs
in total, so the number of doctrines the main book actually *explains* is at most that.

⛔⛔ **WHY NOTHING CAUGHT THIS, and it is the interesting half.** `check_doctrines.sh` carries a
meta-check `<meta:book-count>` that holds `docs/book/src/gate-flow.md`'s published count EQUAL to
the registry size — it has read **33** on every run of this session. It looks like coverage and is
not: it compares ONE INTEGER. A doctrine can be registered, enforced on every commit, mirrored into
`DOCTRINE_ENFORCEMENT.md` §10 (which `<meta:mirror>` DOES hold two-sided) and never named in the
public book, with every meta-check green. ⇒ **the book's number was gated and the book's content
was not**, which is the same shape as the census this repository just spent three slices fixing one
level down.

⭐ **`CORPUS-CENSUS-CURRENCY` — registered by me today — is one of the 14.** I added a doctrine, bumped
the published count, watched `<meta:book-count>` go green, and did not notice the book never
describes it. Recording that rather than quietly fixing it: the failure is the *reliance on a
proxy*, not the missing paragraph.

⭐⭐⭐ **THE BAR IS CLARIFIED, 2026-08-30, AND IT RAISES THIS LEAF'S ACCEPTANCE**: *"Whenever I ask
for the codebase to be in sync, in lockstep with the top level mdbooks, I really mean that important
topics, concept shall be full documented in the proper mdbook"* →
[[feedback_lockstep_means_fully_documented_not_mentioned]]. So *named in the book* does NOT discharge
this leaf; **fully documented in the platform book** does. The 19 "present" families of this census
are therefore not 19 done rows — they are 19 rows needing the same depth audit as the 14 absent ones.

⛔⛔ **AND THE DEBT IS MEASURED, because the director's *"a long time ago"* is a claim and it is TRUE.**
Registration date of each absent doctrine, against 2026-08-30: **69 days** for `DIAG-SEVERITY+DOCPATH`,
`KNOWLEDGE-MAP` and `TASK-ACCEPTANCE`; 42 `DESTRUCTIVE-TARGET-GUARD`; 35 `DESIGN-PRIOR-ART`; 33
`WAIVER-ROUTING`; 32 `ROUTING-EVIDENCE`; 29 `LESSON-PROMOTION`; 11 `SV-RULE-FIRE-PARTITION`; 9
`PARSER-BOOK-CURRENCY`; 4 each for `SAMPLE-PROFILE-PARITY`, `LR-ELIMINATION-RATCHET`,
`GAP-CLAIM-CENSUS`; 0 for `CORPUS-CENSUS-CURRENCY`. **Median 32 · max 69 · 410 doctrine-days.**
⛔ **2026-06-22 is the day the doctrine framework was created**, so three of these have been
enforced-and-undocumented since its founding commit — an ORIGIN defect, not drift — and the
2026-07-05 lockstep directive landed 13 days later without catching them.

**Acceptance (when unparked):** either every registered doctrine is FULLY DOCUMENTED in the main book, or
the ones that are deliberately internal-only carry a recorded disposition; then a meta-check holds
the population TWO-SIDED — the `<meta:mirror>` shape — so the book cannot fall behind the registry
again. ⛔ The count-only check is not to be deleted; it is to be joined by a content arm, because a
number that is right is still worth holding.

⭐⭐⭐ **DIRECTOR-ORDERED, 2026-08-30, SAME SESSION — UN-PARKED.** Verbatim: *"I think the doctrines
shall be documented in the top level mdbook because you keep talking about these 33 doctrines that
are passing, so I asked myself, are these doctrines described, documented somewhere, then I thought
about the top level mdbook."* ⇒ this is no longer my routed finding, it is an instruction, and the
reason given is the binding one: **the book is the director's only window into the project**, and a
count they cannot resolve into descriptions is exactly the *"status label with no distance behind
it"* complaint that founded `DONE-BAR-VISIBILITY`, one level up.

**THE DESIGN IS DECIDED so the next session executes rather than re-litigates. ⛔ DERIVED, NEVER
HAND-WRITTEN — the registry ALREADY holds a full paragraph per doctrine** (that is what
`check_doctrines.sh`'s second `|` field is, and `<meta:mirror>` already holds `DOCTRINE_ENFORCEMENT.md`
§10 two-sided against it). So the chapter is a RENDERING of the registry, on the exact
`report_family_status.sh` / `FAMILY-STATUS-CURRENCY` shape this repository already runs:

1. `scripts/report_doctrines.sh` — reads the registry, emits a `<!-- BEGIN DERIVED -->` block: one
   section per doctrine, its ID, archetype, enforcer path, and the description the registry carries,
   with `--check FILE` to re-derive and diff.
2. A new main-book chapter (`docs/book/src/the-enforced-doctrines.md`) + its `SUMMARY.md` entry,
   carrying that block.
3. **The meta-check gains a CONTENT arm beside its integer arm** — the population held TWO-SIDED,
   `<meta:mirror>`'s shape: every registered doctrine has a section, every section names a
   registered doctrine. ⛔ The count arm is NOT deleted; a number that is right is still worth
   holding. This is what closes the defect this leaf was opened on.
4. ⚠️ Watch the book's own size policy (`README-STABILITY` / live-doc containment): 33 registry
   descriptions are large, so measure the rendered bytes before choosing between one chapter and a
   per-doctrine page set — and say which was chosen and why.

⛔ **SEQUENCING, and it is mine to make** ([[feedback_sequence_approved_work_yourself]]): this runs
FIRST, ahead of resuming `ENGINE-UNIVERSAL-SERVICES.79`(a) — it is director-ordered, bounded to one
slice, touches ZERO Rust/grammar/generated bytes, and serves the surface the director actually
reads. It does not re-open the lane question: the SV release stays the standing high-priority
objective and the lane resumes the moment this lands.

**DELIVERED 2026-08-30 (`-0300`).** `docs/book/src/the-enforced-doctrines.md` — a new Part II
chapter — publishes all **33** doctrines: a hand-written framing half (what a doctrine is, the three
archetypes and how much each is worth, the E1–E4 layers, the two honest limits, the meta-checks, how
to add one) and a **derived roster** of one section per doctrine carrying its archetype, its enforcer
path and its description. `scripts/report_doctrines.sh` renders it; `<meta:book-content>` in
`check_doctrines.sh` re-derives and diffs it on **every commit**.

⛔⛔ **THE LEAF'S OWN DECIDED DESIGN WAS CHECKED BEFORE IT WAS BUILT ON, AND IT WAS PARTLY FALSE.**
The design above says *"DERIVE the chapter from the registry (which already carries a full paragraph
per doctrine)"*. Measured first, because a premise in my own leaf is still a claim:

```
registry description bytes : min 56   median 526   max 6235   total 35737
§10 "Proves" cell bytes    : min 48   median 1755  max 12478  total 90256
registry descriptions under 120 chars: 6 of 33
```

The registry is **2.5× thinner** than the §10 mirror and **6 of 33** of its descriptions are a
single clause — **four of those six are among the 14 the census found absent**. Rendering the
registry alone would have shipped a chapter that is structurally complete and, for those six, still
only a *mention* — the exact thing the clarified bar refuses. ⇒ the producer takes the **population,
the IDs and the enforcer paths from the registry** (the source of truth) and the **archetype and
description from `DOCTRINE_ENFORCEMENT.md` §10**, whose ID set `<meta:mirror>` already holds
two-sided, so the population stays closed by construction and the producer REFUSES if the two ever
disagree.

⭐ **IT CROSS-CHECKS THE ENFORCER PATH, WHICH `<meta:mirror>` DOES NOT.** That check compares ID sets
only, so the registry and §10 could name different scripts for one doctrine with every meta-check
green. Measured before wiring it: **33 of 33 paths already agree**, so the arm fires on nothing today
— the `SV-CORPUS-GRAD.13e.11` condition for a guard safe to make blocking (the fix precedes it).

⛔ **THE `\|` TRAP WAS IMPORTED, NOT RE-SOLVED.** §10 is a markdown table and **3 of its 33 rows carry
a deliberate escaped `\|`**, two of them a shell pipe inside a code span. Splitting on every `|`
mis-reads those rows — which is precisely the trap `TABLE-ARITY-RATCHET` was founded on (its own
naive splitter read a bug ledger **3.8× over-wide**). `split_cells`/`normalise` are imported from
`docs/tasks/artifacts/live_means_live/table_pipe_repair.py`, the module `check_table_arity.sh`
already imports, and the producer REFUSES if it is unreadable rather than falling back to a splitter
of its own. ⭐ `normalise` (`\|` → `|`) is the *faithful* transform out of table context: GFM
unescapes before inline parsing, so `|` is what the table already renders — leaving the backslash in
would publish `\| xargs` inside a code span, where a backslash is literal.

⛔⛔ **AND THE DEPTH HALF IS MEASURED AND PUBLISHED RATHER THAN QUIETLY SHIPPED.** The §10 prose is
fuller for most doctrines but **not for all of them**, and the distribution is bimodal:

```
48  58  62  70  130  210  233  349  368  |  975  1367  1516 … 7064  8167  11764  12477
REGEX-SELF-HOSTING … DESTRUCTIVE-TARGET-GUARD   |  GATE-REACHABILITY … PARSE-COST-RATCHET
```

**9 of 33 are under 500 bytes and the next one up is 975 — a 2.65× gap, the widest in the lower
half**, so the threshold is READ OFF THE DISTRIBUTION rather than invented. ⭐ **All nine are
registry positions 2–12**: description depth is a function of registration **era**, not of
importance — the same shape as this leaf's own debt finding (three doctrines absent since the
framework's founding commit), and for the same reason. The chapter therefore **derives and publishes
its own depth census**, naming every description still under the floor, so thinness is *visible*
instead of silent; enriching those nine is `.22`, and because the sections render §10's prose the
chapter inherits the fix on the next commit with no edit here. ⚠️ **Length is a PROXY for depth and
is deliberately NOT gated** — a 600-byte description can be vacuous and a 300-byte one complete. It
catches the class the census actually found (the single clause) and nothing finer; depth stays a
review judgement, exactly as `.21` says of the general concept census.

**PRICED, THEN CHOSEN — the size question the design asked (item 4).** Rendered block **98 135 B**,
whole chapter **105 752 B**, HTML **152 144 B**. Against the existing distribution — `gate-flow.md`
87 448 B, `inside-parser-performance.md` 168 189 B, `grammar-wellformedness.md` 337 906 B — one
chapter sits mid-pack and *below* two shipped pages, so the per-doctrine page set was not needed and
was declined: 33 pages would put 33 more `SUMMARY.md` rows in front of a reader looking for one
roster, and the roster is the thing being asked for.

⚠️ **HONEST LIMIT, stated not discovered.** This holds the published chapter equal to what the two
tracked sources say **today**. `<meta:mirror>` gates the ID sets, not the prose, so the chapter
inherits §10's review status — reviewed, not gated. Whether any single description is *deep enough*
is a review judgement and no arm here claims otherwise.

**⛔ FOUND WHILE MEASURING, ROUTED NOT DROPPED:** `docs/book/src/gate-flow.md:343` labels the line
`bash scripts/check_doctrines.sh` with `# tier 1, ~1.0 s, every commit`. The **number is right for
the doctrine that section describes** (`GENERATED-REPRODUCIBILITY`, measured **1.09 s**) but it is
attached to the command that runs **all 33**, whose real serial cost is **20.52 s**. A reader copying
that line gets a 20× surprise. ⭐ I first read this as a stale published number and was **wrong** —
re-reading the surrounding prose showed the section is about one doctrine's tier 1. Corrected before
publishing, not after. Owned by `.23`.

## Acceptance Checklist (enforced)

- [x] **ROOT CAUSE (WHY + WHERE)** — ops/build-flow family. **WHERE:** `scripts/check_doctrines.sh`,
      the `<meta:book-count>` arm. **WHY:** it compares **one integer** — the book's published count
      against `${#DOCTRINES[@]}` — and nothing compared the book's *content* to the registry, so a
      doctrine could be registered, enforced every commit and mirrored two-sided into §10 while
      appearing nowhere in the book. Census over the **tracked** book pages, the new chapter
      excluded, at `79ebb56a`:
      ```
      while IFS='|' read -r id _; do
        git ls-files 'docs/book/src/*.md' | grep -v the-enforced-doctrines.md \
          | xargs grep -lqF -- "$id" || echo "ABSENT: $id"
      done < <(grep '^  "' scripts/check_doctrines.sh | sed 's/^  "//')
      -> present 19   absent 14
      ```
      ⭐ `git ls-files` is the population deliberately: the untracked-file case is not academic —
      the same census run against the **working tree** before staging still reported `absent 14`,
      because the new chapter was not yet tracked. The instrument was right and the reading would
      have been wrong.
- [x] **ADDRESSED (verified)** — before→after on the symptom, same command, same population:
      **absent 14 → absent 0**, and every one of the **33** is a `### ` section rather than a name in
      a list (`grep -c '^### \`' docs/book/src/the-enforced-doctrines.md` → **33**). The gate that
      keeps it that way: `<meta:book-content>` → `✓ PASS  docs/book/src/the-enforced-doctrines.md
      describes exactly the 33 registered doctrines, and equals a fresh derivation`.
- [x] **NO REGRESSION** — `bash scripts/check_doctrines.sh` → **ALL 33 enforced doctrines PASS**,
      all four meta-checks green (`<meta:registry-escaping>` 33 rows, `<meta:mirror>` 33,
      `<meta:book-count>` 33 at 2 marked sites, `<meta:book-content>` 33). `make mdbook_docs_gate`
      → **exit 0**, all **10** per-parser book gates plus the platform book. The generated tree is
      untouched: `check_generated_reproducibility.sh` → `OK (11 artifacts unmoved, emission sources
      unmoved since e974d34 — tier 2 last proved them **byte-identical** to HEAD)`, and the change
      stages **zero** bytes under `rust/src/**`, `grammars/**`, `generated/**` (empty
      `git diff --cached --stat` over those paths; +774/−0 lines across 4 files, all of them a
      script or a doc). Cost added to the per-commit driver: **0.12 s** on a **20.52 s** run (two
      `report_doctrines.sh` invocations at **61 ms** — the render and the red control).
- [x] **RED CONTROLS — 8 arms, every one seen to FAIL before its green was believed.** Arms 3–7 run
      against a **constructed repository root** in the scratchpad (a copy of the producer plus
      doctored sources), so no tracked file is mutated and a crashed probe cannot leave a dirty
      tree — the `check_family_status.sh` seam pattern.
      | # | constructed state | want | got |
      |---|---|---|---|
      | 1 | `--check` a page that does not exist | 2 | 2 `no published page at …` |
      | 2 | `--check` a page with no DERIVED block | 2 | 2 `carries no <!-- BEGIN DERIVED -->` |
      | 3 | registry missing a doctrine §10 has | 2 | 2 `…§10 but not registered: KNOWLEDGE-MAP` |
      | 4 | registry and §10 naming different enforcers | 2 | 2 `registry scripts/check_elsewhere.sh vs §10 …` |
      | 5 | registry array EMPTY | 2 | 2 `…is EMPTY. Rendering nothing would let --check pass by construction` |
      | 6 | shared `\|`-splitter unreadable | 2 | 2 `…would be mis-split … refusing instead of guessing` |
      | 7 | §10 section renamed | 2 | 2 `§10 could not be located` |
      | 8 | one archetype mutated in the published block | 1 | 1 + a unified diff naming the line |
      ⛔ Arms 5 and 6 are the two that matter most: an empty derivation would make `--check` pass
      **by construction**, and a private `|` splitter would silently mis-read the 3 escaped rows.
      Both refuse.
- [x] **LOCKSTEP** — `docs/book/src/SUMMARY.md` gains the chapter in Part II after *The Gate Flow*;
      `mdbook build` clean; **41 in-page anchors, 41 in-page links, 0 broken**, including the
      awkward `DIAG-SEVERITY+DOCPATH` → `#diag-severitydocpath`.

### ⛔ `.21` — **WHICH OTHER IMPORTANT CONCEPTS ARE ENFORCED OR SHIPPED BUT NOT FULLY DOCUMENTED, AND IN WHICH BOOK EACH BELONGS** (`todo`, opened 2026-08-30 by [[feedback_lockstep_means_fully_documented_not_mentioned]]; sequenced AFTER `.20`)

The 2026-08-30 directive is broader than the doctrines: *"important topics, concept shall be full
documented in the proper mdbook."* `.20` closes the doctrine population. This leaf asks the general
question the directive actually poses, and it must not be answered by intuition — the doctrine
instance was invisible for **69 days** to everyone including the guard that looked like it covered it.

**Two axes, both needed.** **FULLY** — a mention, a link or a matching count is not documentation.
**PROPER** — this repository publishes ELEVEN books; a platform-wide concept documented only inside
one family's book satisfies neither axis.

**Candidate population to census (not a closed list — establishing the list IS the leaf):** the
enforced doctrines (`.20`), the gate flow and its exit-code contracts, the closure/`Done` model, the
annotation languages (return + semantic) and their composition rules, the profile system, the
memory/continuity architecture, the toolbox's diagnostic protocol, the corpus/answer-key model, and
every structural AST-pipeline stage named by [[feedback_ast_pipeline_components_documented_in_top_book]].

**Acceptance:** a register of important concepts with the owning book named per concept; each
adjudicated documented / partially / absent against the FULLY bar; every gap owned by a leaf; then a
two-sided currency check over the register, on `<meta:mirror>`'s shape. ⚠️ Depth cannot be gated —
that stays a review judgement and must be stated as an honest limit, not pretended away.

⛔ **SEQUENCED after `.20` and after the SV lane resumes** — `.20` is director-ordered and bounded;
this one is a census whose size is unknown until it runs, and the SV release is the standing
high-priority objective.

### ✅ `.22` — **NINE DOCTRINE DESCRIPTIONS ARE A SINGLE CLAUSE, AND THEY ARE THE NINE OLDEST — THE BOOK NOW *MENTIONS* THEM RATHER THAN EXPLAINING THEM** (`DONE` 2026-08-30 `-0301`, opened same day by `-0300` — the DEPTH half of the director's order)

`.20` closed the STRUCTURAL half: all 33 doctrines now have a section in the platform book and a
gate re-derives it every commit. This leaf is the half that gate deliberately cannot close.

- **WHY + WHERE, measured.** The chapter renders `DOCTRINE_ENFORCEMENT.md` §10's `Proves` cell as
  each doctrine's description. Nine of those cells are one clause:
  ```
  REGEX-SELF-HOSTING 48 · KNOWLEDGE-MAP 58 · EBNF-SOURCE-OF-TRUTH 62 · DIAG-SEVERITY+DOCPATH 70
  DESTRUCTIVE-TARGET-GUARD 130 · REGEX-ORACLE-ANCHOR-SYNC 210 · DESIGN-PRIOR-ART 233
  WAIVER-ROUTING 349 · ROUTING-EVIDENCE 368        | next: GATE-REACHABILITY 975
  ```
  ⭐ **All nine are registry positions 2–12** — the oldest doctrines after `MEMORY-ARCH`, which is
  thick only because an incident later enriched it. Depth is a function of registration **era**, not
  of importance: every doctrine registered from position 13 onward carries ≥ 975 bytes. Same shape,
  same cause as `.20`'s debt finding — the practice of writing a founding measurement into the row
  was learned *after* these nine were registered, and nothing went back.
- **THE MATERIAL ALREADY EXISTS AND IS TRACKED, so this is derivation, not invention.** Every one of
  the nine enforcers carries a provenance header naming the incident it was founded on. The work is
  to read each enforcer and write its §10 row from its own header — not to invent prose about a
  doctrine.
- ⛔ **THE FIX LANDS IN §10, NOT IN THE BOOK.** The chapter is derived, so enriching a §10 row
  republishes the book section on the next commit with no edit to the chapter. Editing the chapter
  directly would be caught by `<meta:book-content>` as an out-of-sync block — correctly.
- **Acceptance:** all nine §10 rows explain what the doctrine proves, what it was founded on, and
  what it deliberately does not cover; the chapter's own derived depth line then reports
  `0 under 500 B` without that line being edited. ⚠️ **The 500-byte floor is a PROXY and must not
  become the target** — the acceptance is that a reader learns what the doctrine does, and a row
  padded to 501 bytes fails it while satisfying the number. Length is reported, never gated, for
  exactly this reason.

**DELIVERED 2026-08-30 (`-0301`).** All nine `DOCTRINE_ENFORCEMENT.md` §10 rows rewritten from
their own enforcer's tracked provenance header — **derivation, not invention**: every incident,
directive date, measured number and honest limit below already existed in the script that enforces
the doctrine, and had simply never reached the mirror the book renders.

| doctrine | before | after | what the reader now gets that a clause could not carry |
|---|---:|---:|---|
| `REGEX-SELF-HOSTING` | 48 | 1255 | *why* delegating to the `regex` crate would measure the wrong subject, both invariants, and the deliberate regex-family-only scope |
| `KNOWLEDGE-MAP` | 58 | 1233 | that the map is **re-derived, not maintained** — the derive-and-diff arm is what makes it structural |
| `EBNF-SOURCE-OF-TRUTH` | 62 | 1371 | the mechanism: the generator reads the EBNF **and nothing else**, so an out-of-band validator is invisible to it and breaks the duality |
| `DIAG-SEVERITY+DOCPATH` | 70 | 1395 | why two unrelated subjects share one doctrine — **both fail silently, in the passing direction** — and the guarded surface set with its two deliberate exclusions |
| `DESTRUCTIVE-TARGET-GUARD` | 130 | 1198 | the 2026-07-18 incident (**102.8 GiB**, reached through a dependency edge from a target that sounded safe) and why the confirmation is an env var, not a prompt |
| `REGEX-ORACLE-ANCHOR-SYNC` | 210 | 1361 | why it asserts *agreement* rather than a measurement, and the **174 stale-form occurrences** that founded it |
| `DESIGN-PRIOR-ART` | 233 | 1308 | the `@lexical_token` incident, and that the cost of a duplicate surface is permanent, not one-off |
| `WAIVER-ROUTING` | 349 | 1492 | ⭐ the durable law — **an author writing a waiver IS the gate reporting a missing capability** — and the load-bearing constraint that it must not punish honesty |
| `ROUTING-EVIDENCE` | 368 | 1461 | the question it forces (*does the finding reproduce OUTSIDE that family?*), and that its own first cut would have missed its founding incident |

- [x] **ROOT CAUSE (WHY + WHERE)** — ops/build-flow family. **WHERE:** the `Proves` cell of nine rows
      in `DOCTRINE_ENFORCEMENT.md` §10, which `scripts/report_doctrines.sh` renders as each
      doctrine's book description. **WHY:** the practice of writing a founding measurement into the
      row was learned *after* these nine were registered and nothing went back — measured by
      `git ls-files`-scoped inspection of each enforcer, **all nine are registry positions 2–12**,
      and every doctrine registered from position 13 on already carried ≥ 975 B. Depth was a
      function of registration **era**, not of importance.
- [x] **ADDRESSED (verified)** — before→after on the symptom, re-derived by the producer rather than
      re-typed: **`9 under 500 B` → `0 under 500 B`**, thinnest **48 B → 783 B**
      (`LESSON-PROMOTION`, a row this leaf did not touch), the nine landing at **1198–1492 B**.
      `bash scripts/report_doctrines.sh | head -4` prints it. ⭐ **The chapter's own published depth
      line flipped branch by itself** — from *"9 of 33 are still a single clause … therefore
      mentioned rather than explained"* to *"every one of the 33 clears the 500-byte one-clause
      floor"* — because it is derived, which is the property `.20` built it for.
- [x] **NO REGRESSION** — `bash scripts/check_doctrines.sh` → **ALL 33 PASS**, four meta-checks
      green; `TABLE-ARITY-RATCHET` PASS, which is the one that matters here since nine markdown
      table rows were rewritten (**0 arity breaches, 33 rows still 4 cells**, re-derived through the
      shared GFM splitter). `make mdbook_docs_gate` → exit 0, 10 parser books + the platform book;
      **41 anchors / 41 links / 0 broken**. Generated tree untouched —
      `check_generated_reproducibility.sh` OK, **byte-identical** to HEAD; **zero** bytes staged
      under `rust/src/**`, `grammars/**`, `generated/**`.
- [x] **RED CONTROL — supplied by the work itself, in production rather than in a probe.** With §10
      enriched and the chapter not yet re-derived, `report_doctrines.sh --check` on the **real**
      published page exited **1** with a unified diff, then **0** after republishing. `.20`'s eight
      constructed arms proved the producer *can* refuse; this proves the wiring is live on the
      shipping path — the distinction `GATE-REACHABILITY` exists for.

⚠️ **HONEST LIMIT, and it is the acceptance this leaf was written with.** The floor is a **proxy**:
a row padded to 501 bytes would satisfy the number and fail the intent. The real acceptance is that
a reader learns what each doctrine does and why it exists, which is a review judgement no gate makes
— so length is **reported and never gated**, and this paragraph is the bound rather than a
disclaimer. ⭐ The nine were written from the enforcers' own headers precisely so the prose is
*checkable against a tracked source* rather than persuasive.

### ⛔ `.23` — **THE BOOK LABELS `bash scripts/check_doctrines.sh` AS `~1.0 s` WHILE THE DRIVER'S REAL SERIAL COST IS 20.52 s — THE NUMBER IS RIGHT AND THE COMMAND IT LABELS IS NOT** (`todo`, routed 2026-08-30 by `-0300`, low severity, PARKED behind `.22` and the SV lane)

- **WHY + WHERE.** `docs/book/src/gate-flow.md:343` reads
  `bash scripts/check_doctrines.sh   # tier 1, ~1.0 s, every commit`. The surrounding section is
  about **`GENERATED-REPRODUCIBILITY`**, whose tier 1 measures **1.09 s** — so the *number is
  accurate for the doctrine being described*. It is attached to the command that runs **all 33**.
- **MEASURED**, per-enforcer, 2026-08-30: summed serial cost **20.52 s**; top rows `KNOWLEDGE-MAP`
  **4.86**, `FAMILY-STATUS-CURRENCY` **4.31**, `SV-CONTRACT-CURRENCY` **2.51**, `MEMORY-ARCH`
  **2.25**, `PARSE-COST-RATCHET` **1.09**, `GENERATED-REPRODUCIBILITY` **1.09**; the remaining 25
  sum to **3.18 s**. A reader who copies that line gets a 20× surprise.
- ⭐⭐ **I READ IT WRONG FIRST, AND THE CORRECTION IS THE POINT.** My first reading was *"a published
  performance claim is 21× stale"* — alarming, and false. Re-reading the surrounding prose showed
  the section describes one doctrine's tier 1. **Caught before publishing, not after**
  ([[feedback_stand_by_published_claims_without_re_audit]], and leg 2 of the three-way check:
  falsify the CLAIM against its source, not the instrument against itself).
- ⛔ **THE FIX IS NOT TO RE-TYPE A NUMBER.** `DERIVED_STATE_CONTAINMENT.md` R1/R3: a hand-typed
  20.52 rots the same way. Either the comment names the doctrine it prices (cheap, honest, and the
  cost claim moves nowhere) or the driver learns `--report` and the book carries a derived block.
  Price both before choosing.
- **Acceptance:** the book no longer prices the 33-doctrine driver with a one-doctrine number, and
  whichever form is chosen is stated with its reason.

### ⛔ `.19` — **`LESSON-PROMOTION`'s DECLINE ARM MATCHES ANY PRE-EXISTING TOKEN ANYWHERE IN A STAGED LEAF FILE, SO A TREE THAT DECLINED ONCE IS EXEMPT FOREVER — 10 OF THE TASK FILES ALREADY ARE** (`todo`, ROUTED + PARKED behind the SV lane, opened 2026-08-29 by `PGEN-ENGINE-UNIVERSAL-SERVICES-0131`)

- **WHY + WHERE, tool-backed.** `scripts/check_lesson_promotion.sh:107-113` decides arm (b) by
  `grep`ping each staged `docs/tasks/*.md` for `promotion: declined (...)` **over the WHOLE FILE**.
  A task tree is one long-lived file, so a token written for slice *N* satisfies the gate for slice
  *N + 1*, *N + 2*, … forever. There is no per-slice or per-diff scoping on that arm.
- **MEASURED, on this very commit.** `PGEN-ENGINE-UNIVERSAL-SERVICES-0131` staged a NEW dated lesson
  heading in `DEVELOPMENT_NOTES.md` with no promotion and no new decline, and the gate exited **0**:
  ```
  grep -c 'promotion: declined (' docs/tasks/ENGINE-UNIVERSAL-SERVICES.md   # -> 5 at 0e3bc12c,
                                                                           #    9 at 340baafd
  grep -l 'promotion: declined (' docs/tasks/*.md | wc -l                   # -> 10
  ```
  ⇒ **10 of the task files are already permanently exempt from arm (b)**, and every tree acquires the
  exemption the first time it declines. ⚠️ **The first count is dated on purpose (`-0132`): it was 5
  before `-0131` and is 9 after, because that slice wrote four more — so an undated `# -> 5` in a
  tracked artifact is stale the moment it ships.**
- ⭐ **THE GATE'S OWN COMMENT DESCRIBES THIS DEFECT ONE LEVEL DOWN AND WAS FIXED THERE.** It records
  that the first real run *"passed on a leaf that merely DOCUMENTED the token … a gate satisfied by
  its own documentation is decoration"*, and rejects the literal `(<reason>)` placeholder. The same
  shape met one level up — a gate satisfied by its own HISTORY — was not covered by that fix.
- **THE FIX SHAPE** — ⛔⛔ **AND `-0132` FOUND THE FIRST VERSION OF IT INSUFFICIENT, WHICH IS WHY IT
  IS WRITTEN TWICE.** The obvious move is what the rest of this registry already does: make arm (b)
  STAGED-DIFF-SCOPED, exactly as `GAP-CLAIM-CENSUS` and `TABLE-ARITY-RATCHET` are — the token must be
  an ADDED line in the staged diff (`git diff --cached -U0 -- docs/tasks | grep -E '^\+.*promotion: declined \('`),
  not merely present in the file. **That is necessary and NOT sufficient.** Three of the nine tokens
  now in `ENGINE-UNIVERSAL-SERVICES.md` are **PROSE ABOUT the token** — this leaf's own two command
  lines and `.67`(c)(iii)'s sentence *"this tree already carries 5 `promotion: declined (…)`
  tokens"* — and every one of them WAS an added line in `-0131`'s staged diff, so a diff-scoped arm
  would have accepted them unchanged. **A document that describes the trigger trips the trigger.**
  ⇒ the arm must ALSO require the token to HEAD a list item — `^\s*[-*] \**promotion: declined \(` —
  so a mention inside a sentence, a command line or a fenced block cannot satisfy it. ⭐ That is the
  same correction the gate's own comment records making one level down (it rejects the literal
  `(<reason>)` placeholder because *"a gate satisfied by its own documentation is decoration"*); the
  placeholder was the symptom and the substring match is the cause. ⛔ Free on day one by the `SV-CORPUS-GRAD.13e.11` rule only if the
  fix precedes the blocking: the 10 exempt files must not suddenly block on their next edit, and
  scoping to the ADDED lines achieves that by construction (an old token is not an added line, and a
  commit with no new lesson heading is not judged at all).
- ⚠️ **HONEST LIMIT, and it is why this is ROUTED rather than fixed in-slice.** The gate's own
  archetype note already says it verifies that *a decision was recorded*, not that the decision was
  correct; scoping arm (b) to the diff does not change that, it only stops the decision being
  inherited. The `promoted` arm (a) is diff-scoped already and is not affected.
- ⭐ **The obligation was discharged by hand for the slice that found it**, rather than left to the
  gate that could not see it: `docs/knowledge/a-control-validates-the-property-it-is-built-from.md`
  was written and the map regenerated (233 facts), and the remaining lessons carry an explicit
  decline in `ENGINE-UNIVERSAL-SERVICES.67`(c)(iii).
