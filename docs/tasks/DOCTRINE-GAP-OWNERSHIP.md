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
| 3 | `rust/src/ast_pipeline/unified_return_ast.rs:1883` | `ParseContent::Terminal("<array_access>"` — ⛔ **UNCLOSED, see the routed finding below** |
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
