# CI-PARITY-GATE-ROT: the local CI-parity gate — the one the README names as the way to prove workflow parity while hosted Actions are paused — has not been able to complete for 1,371 commits

## Metadata

- Tree ID: `CI-PARITY-GATE-ROT`
- Status: `active` (opened 2026-07-27, session #215)
- Family / slice-id prefix: `PGEN-CI-PARITY-GATE-ROT-<NNNN>`
- Created: `2026-07-27`
- Owner: repo-local workflow
- **Frontier: `.4`** then **`.2`** (`.1` **done** 2026-07-27 session #216 — audit phase **23 PASS / 8 FAIL → 31 PASS /
  0 FAIL**; the escalated row was ruled on by the director same-session and executed as `.1b`.
  **`.3` done 2026-07-28 session #218** — the workflow phase was exercised for the first time:
  **3 PASS / 8 FAIL unprepared**, all eight one defect, and — once the repository's own cold-clone
  bootstrap runs inside the export dir — **every replay that reached a verdict reached PASS**. `.4` — the same defect in the
  **hosted** workflow files, 14 of 15 of which never got the fix — is now the tree's largest open
  item and outranks `.2`, because `.2` is an inventory of a class `.4` is a live instance of.
  ⭐ **`.3` also surfaced `.5`**: running the aggregate for the first time showed
  `make -C rust sota_exit_gate` is RED on a required sub-gate that can only pass when SV
  generation FAILS — measured in the main repo too, so not an artefact of the export dir.)
- ✅⛔ **DIRECTOR RULING RECEIVED (2026-07-27, session #216, verbatim):** *"We will republish PGEN
  regex parser at a later time"* + *"I agree we shouldn't cite either regex.json or regex.ebnf"*
  ⇒ **THE BOUNDARY STANDS.** `audit_embedding_api_surface` is RIGHT; the repo is wrong. The
  citations come out. ⛔ The audit is NOT to be retired.
- ⛔⛔ **BUT THE SCOPE IS NOT WHAT THE ONE FAILING ASSERTION SUGGESTS — MEASURED 2026-07-27 BEFORE
  ANY EDIT, and this is the whole reason `.1b` is its own leaf rather than a five-minute fix.**
  `docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md` is **2,982 lines** and carries
  **~40** occurrences of the two forbidden tokens, which split into two classes that must NOT be
  treated the same way:

  | class | where | count | disposition |
  |---|---|---|---|
  | **consumer-facing build instructions** | `## Generated Parser Build Recipe (for downstream consumers like RGX)` (`:2426`, `:2431`, `:2520`, `:2541`, `:2554`) | **5** | ⇒ **THIS is what the ruling targets.** Rewrite onto the supported surface (`make -C rust regex_parser` + `pgen::embedding_api`), or move the recipe out of the downstream contract entirely |
  | **historical provenance** | ~30 `## Maintenance Update …` / `## Release … Highlights` sections, e.g. *"the bound is now encoded structurally inside `grammars/regex.ebnf`"* | **~35** | ⛔ **DO NOT DELETE.** These record WHICH grammar rule changed in a released slice. Removing them destroys the maintenance record and back-dates history — the discipline this project refuses (`LEX-ADJACENCY.1` precedent) |

  ⇒ ⭐ **THE AUDIT'S SHAPE IS WRONG FOR ITS OWN INTENT.** `assert_file_not_contains` is a
  WHOLE-FILE check, so satisfying it literally would require deleting 35 provenance notes. The
  boundary the ruling protects is *"do not INSTRUCT a consumer to reach into PGEN-internal build
  inputs"* — a statement about the **consumer-facing sections**, not about whether history may
  name a grammar file. Same defect class this leaf just fixed seven times over: **the assertion
  pins something broader than the invariant it owns.**
  ⇒ **`.1b` must do BOTH halves, and neither alone is acceptable**: (a) remove/rewrite the 5
  consumer-facing citations per the ruling, and (b) re-scope the assertion to the consumer-facing
  region so it keeps its teeth without being unsatisfiable-without-destroying-history.
  ⚠️ Deliberately NOT started at the end of session #216 — the director called it (*"Better fixing
  that in a new session, no?"*) after the scope above was measured. **No partial state exists.**

- ⛔⛔ **DIRECTOR-SCHEDULED FOR A FRESH SESSION (2026-07-27, session #215, verbatim: *"Do this
  … at the next fresh session"*)**, after `GENERATED-LINT-CORRECTNESS.4`. The director asked for
  findings 3 and 4 of the `-0004` surfacing report — *"fix these in a sota, signoff and
  professional way"* — which are exactly `.1` and `.2` below. Execution order:
  `GENERATED-LINT-CORRECTNESS.4` → **`.1`** → **`.2`**. Nothing was started; no partial state.
- Opened by: `GENERATED-LINT-CORRECTNESS.3`, which had to run
  `make -C rust SHELL=/bin/bash ci_workflow_local_gate` in order to register a new surface
  audit, and found the gate dies on its **first** audit. Deliberately NOT absorbed into that
  leaf: the residual failures are eight independent defect classes (docs-surface drift, a
  stale API-version pin, superseded contract prose, a Perl-reference rule) with nothing in
  common but the gate they live in.

## ⭐ THE FINDING (measured, at commit `730419a2`)

`make -C rust SHELL=/bin/bash ci_workflow_local_gate` is the repo's **local workflow-parity
gate**. `README.md` names it explicitly as the lane to use *while hosted GitHub Actions are
paused to conserve account minutes* ("routine proof should use the local `make -C rust ...`
gates until hosted auto-runs are re-enabled"). It is also the gate that audits the root /
top-level / contract / reference markdown allowlists and replays the tracked workflow
commands.

**It cannot complete, and has not been able to for 1,371 commits.**

### The first blocker (repaired by `GENERATED-LINT-CORRECTNESS.3`, not by this tree)

`audit_static_include_paths` — the **first** audit `main()` calls — asserted
`assert_tracked "generated/ebnf.rs"`. Nothing under `generated/` is tracked:

```
$ git ls-files generated/ | wc -l
0
$ grep -n 'generated/' .gitignore
24:generated/
```

Commit `0ed2b2ad` *"Slice 5: stop tracking generated/\* in git"* (2026-04-29) is an ancestor
of `HEAD`, with **1,371 commits since**. Seven `assert_tracked "generated/…"` call sites were
asserting a condition repository policy guarantees can never hold. `GENERATED-LINT-CORRECTNESS.3`
repaired them (new helper `assert_generated_artifact`, which checks *presence on disk* — the
condition `rust/build.rs` actually tests with `is_file()` — and refuses with the regeneration
command), because otherwise that leaf's own newly-registered audit would have been
unreachable dead code.

### What remains: 8 of 31 audit functions still fail, each independently

Measured after the repair, by sourcing the gate and invoking every `audit_*` function in its
own subshell:

| # | failing audit | measured reason |
|---|---|---|
| 1 | `audit_top_level_docs_surface` | allowlist omits 4 live files: `docs/TASK_TREE.md`, `docs/TASK_TREE_README.md`, `docs/POST_SV_AUDIT_LEDGER.md`, `docs/SV_EXH_PROOF_BASELINE.md` |
| 2 | `audit_contract_docs_surface` | `contract docs allowlist drift detected` |
| 3 | `audit_reference_docs_surface` | `reference docs allowlist drift detected` |
| 4 | `audit_active_docs_rehome_paths` | live docs still cite pre-rehome paths |
| 5 | `audit_embedding_api_surface` | pins `pub const EMBEDDING_API_VERSION: &str = "1.2.0";`; the source has moved on |
| 6 | `audit_ebnf_frontend_conversion_surface` | forbids `ebnf_to_json.pl` in `rust/Makefile` — it is present |
| 7 | `audit_rtl_frontend_generated_contract_surface` | expects `expected_rule_texts` in the probe; superseded by the `0.2.0` typed-AST contract migration |
| 8 | `audit_sv_formal_exhaustive_closure_surface` | pins contract prose (`required_surface_missing_detail`) that has since changed |

⚠️ **None of these is a one-line fix, and none should be "made green" reflexively.** Each is
the same question with a different answer: **is the AUDIT stale, or is the REPO wrong?**
Rows 1–4 look like genuine allowlist drift the audits exist to catch — in which case the
finding is that four live docs were added without the deliberate policy update the audit
demands, and the gate was right. Rows 5–8 look like audits pinned to superseded content by
campaigns that moved on without updating them — in which case the audits are the stale party.
Guessing wrong in either direction silently destroys a real check.

## ⭐⭐ WHY THIS MATTERS MORE THAN ITS SIZE SUGGESTS

This is the **third** instance in two sessions of a maintained check that nothing runs, and
the pattern is now unmistakable:

1. `GENERATED-LINT-CORRECTNESS.1` — `make -C rust ast_dump_contract_gate` RED since
   `7219547c`; *"referenced by no aggregate and no CI workflow"*.
2. `GENERATED-LINT-CORRECTNESS.3` — `PGEN_CLIPPY_GENERATED_STRICT` set by nothing at all; the
   291 → 0 win was unguarded from the moment it landed.
3. **This tree** — the local parity gate itself, which is the *designated substitute* for the
   paused hosted workflows.

⇒ the unifying principle this family keeps re-deriving — **a check that cannot run, or cannot
see, must SAY SO, not return green** — has a corollary that is not yet mechanized:
**a check that nothing INVOKES is indistinguishable from a check that does not exist.** The
project has no inventory answering *"which tracked gates are reachable from something that
actually runs?"* That question is `.2`.

## Leaves

### `.1` — adjudicate the 8 failing audits (`done`)

- **Status: `done`** (2026-07-27, session #216, `PGEN-CI-PARITY-GATE-ROT-0001` + `-0002`).
  **Audit phase: 23 PASS / 8 FAIL → 31 PASS / 0 FAIL.** The last row was escalated rather than
  guessed, the director ruled the same session, and `.1b` executed the ruling.
- ⛔ Explicitly forbidden and **not done**: no audit was deleted, and none was relaxed to make
  the gate green. Every change is either a *deliberate policy update the audit itself asked
  for* or a *re-pin onto the surface that replaced the retired one*.

#### ⭐⭐ THE HEADLINE FINDING: "8 failing audits" WAS AN UNDERCOUNT — the real number is 12 stale assertions, and fail-fast hid a third of them

`assert_file_contains` stops its audit at the FIRST miss, so an audit with four stale pins
reports one. Fixing the reported pin revealed the next, four times over:

| revealed at | assertion | why it was hidden |
|---|---|---|
| round 2 | `EMBEDDING_API_CONTRACT.md` `GrammarProfile` roster (missing `verilog_2005`) | behind the `EMBEDDING_API_VERSION` pin |
| round 2 | rtl contract **JSON** `expected_rule_texts` | behind the rtl **probe** pin |
| round 3 | `PGEN_PARSER_INTEGRATION_CONTRACTS.md` regex row embedding `1.1.29`/`1.1.31` | behind the `GrammarProfile` pin |
| round 3 | `MIN_GENERATED_CONTRACT_ELABORATION_ACCEPTS` (+5 sibling ratchets) | behind the `..._SAMPLES` ratchet |

⇒ **a fail-fast gate cannot tell you how broken it is** — it reports its depth one layer per
run, and a gate nothing runs never gets a second run. The per-audit census driver exists so the
whole census is visible at once.

#### The 8 dispositions (each measured, none guessed)

| # | audit | verdict | the deciding evidence |
|---|---|---|---|
| 1 | `top_level_docs_surface` | **REPO — policy never updated** ⇒ allowlist updated deliberately | the 4 files are 2 months old and heavily referenced: `docs/TASK_TREE.md` **70** refs and mandated by `CLAUDE.md` item 6, `TASK_TREE_README` 9, `POST_SV_AUDIT_LEDGER` 23, `SV_EXH_PROOF_BASELINE` 6. The audit's own message says *"or update the tracked policy deliberately"* — this is that update. |
| 2 | `contract_docs_surface` | same class ⇒ 6 entries added | 2 RTL family contracts (14 refs each) + the 4 `SEMANTIC_STORE` contracts; all added 2026-05-15/21 |
| 3 | `reference_docs_surface` | same class ⇒ 2 entries added | `PARSEABILITY_PROBE` (9 refs), `SV_EXH_PROOF_DEFECT_TAXONOMY` (7 refs) |
| 4 | `active_docs_rehome_paths` | ⭐ **AUDIT RIGHT / REPO WRONG** — real drift, fixed in the doc | `PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md:79` wrote a bare `PGEN_RELEASED_PARSER_BUG_LEDGER.md` while **the same file** uses the correct `docs/contracts/…` form at `:2284` and `:2904` — an internal inconsistency, exactly the drift the audit exists to catch |
| 5 | `ebnf_frontend_conversion_surface` | **AUDIT TOO BROAD** ⇒ narrowed to executable lines | the sole `rust/Makefile` hit is a **COMMENT** (`:782`) documenting the bootstrap seed; measured **zero** non-comment occurrences ⇒ the migration the audit checks for is COMPLETE and it was reporting its own documentation back to it. The Perl path is deliberately still live in the hybrid flow (README; consumer `ebnf_stimuli_quality_gate.sh`, deliberately not in the audit's list) |
| 6 | `embedding_api_surface` | **AUDIT STALE BY DESIGN** (versions) ⇒ shape assertions | pinned `1.2.0`/`1.1.31`/`1.1.29`; live `1.3.1`/`1.1.109`/`1.1.106`. A parity audit that hard-pins a value the release policy moves fails on **every legitimate bump** ⇒ it rots by construction. Values are already owned by the release-policy gates and the contract identity blocks — this audit was duplicating that ownership |
| 7 | `rtl_frontend_generated_contract_surface` | **AUDIT STALE** ⇒ re-pinned to the successor | pinned `expected_rule_texts`; the probe now has **0** occurrences and the contract is `0.2.0`. ⭐ In the contract JSON the ONE surviving occurrence is inside its own `provenance` sentence recording the retirement (*"…span locks … retired; their curated texts were re-expressed as `required_typed_string_values`"*) — **the audit was pinned to a token that survives only in the explanation of its own removal** |
| 8 | `sv_formal_exhaustive_closure_surface` | **AUDIT STALE** ⇒ re-pinned to the requirement's stable key | pinned the prose *"SystemVerilog still **lacks** … sidecar"*; the contract now reads *"SystemVerilog **requires** its checked-in … sidecar … to match the live triage gate output exactly"* — the world moved forward and the audit pinned the old state |

⭐ **The pattern across rows 6, 7, 8 and the four hidden ones is ONE defect, not seven:** the
audit pins a *value or narrative that is designed to change* (a release version, a ratchet
minimum, a migration-era token, a status sentence) instead of the *invariant it actually owns*
(the constant is declared and well-formed; the layer exists; the requirement names its subject).
**Duplicated ownership of a moving value is the rot mechanism** —
[[feedback_duplicated_metadata_needs_derived_drift_gate]] at gate scale.

#### ⛔⛔ ESCALATED — the 1 remaining failure is a POLICY question, deliberately NOT decided

`audit_embedding_api_surface` still fails on two assertions added **2026-03-28** (`d7f86f37`
*"Harden regex downstream integration contract"*):

```
assert_file_not_contains docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md 'generated/regex.json'
assert_file_not_contains docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md 'grammars/regex.ebnf'
```

Their intent is a **published support boundary**: downstream consumers integrate through
`pgen::embedding_api`, and the contract must not point them at PGEN-internal build inputs. But
on **2026-04-30** (`18dbc598`, *"… + RGX build recipe"*) a deliberate campaign **added exactly
that** — a build recipe citing `generated/regex.json` (`:2431`, `:2554`) and `grammars/regex.ebnf`.

⭐ **The audit had already been dead for a day when that landed** (`0ed2b2ad`, 2026-04-29), so
nothing objected. Both sides are defensible: the boundary is a real published promise, and a
consumer-runnable build recipe is genuinely useful. Deciding it either way changes what PGEN
promises downstream, which is the director's call, not mine — and the leaf's own rule is that
guessing here *"silently destroys a real check"*. **Left failing on purpose. See the surfacing
note in this tree's header.**

#### Acceptance checklist (enforced)

- [x] **ROOT CAUSE (WHY + WHERE)** — WHY the gate cannot complete: 12 stale assertions across 8
      audits, whose common mechanism is an audit pinning a value designed to move rather than the
      invariant it owns. WHERE, per row, in the table above. Tool-backed: the per-audit census
      driver (`git ls-files` for the allowlists, `make -n`-class reasoning for the Makefile
      comment, `git log -1 -S` for the provenance of the escalated assertions —
      `d7f86f37` 2026-03-28 added them, `18dbc598` 2026-04-30 violated them).
- [x] **ADDRESSED (verified)** — measured before → after with the same driver, via a
      `git stash` round-trip over the gate: **23 PASS / 8 FAIL → 30 PASS / 1 FAIL**. Captures
      banked as `audit_census_before.txt` / `audit_census_after.txt`.
- [x] **NO REGRESSION** — no audit deleted, none relaxed to green; `bash -n` clean; the 23
      previously-passing audits still pass (30 = 23 + 7). 9/9 doctrines PASS. No
      `grammars/*.ebnf`, no `rust/src/*`, no `generated/*` ⇒ all 11 parsers byte-identical by
      construction; no release/schema/ledger movement.

#### ⚠️ Still open in this leaf

- The **escalated policy question** above (1 failing audit).
- The gate's `run_workflow` phase beyond the audits has **not** been exercised here; this leaf's
  acceptance is the audit phase, per its charter. ⚠️ Note for whoever runs the full gate: the
  audits inspect the REAL repo (`$ROOT_DIR`) while `run_workflow` executes inside the exported
  tracked-only worktree. That asymmetry is load-bearing — `generated/` exists for the former and
  never for the latter.

#### Evidence

- `docs/tasks/artifacts/ci_parity_gate_rot/run_audit_census.sh` — per-audit census; **banks the
  two reproduction traps that each produce a confidently WRONG census** (the script locates
  itself via `BASH_SOURCE/../..`, so a stripped copy must sit at the same depth — otherwise 28
  of 31 "fail"; and audit names contain digits, so `[a-z_]+` invents `audit_regex_pcre`).
- `docs/tasks/artifacts/ci_parity_gate_rot/audit_census_{before,after}.txt`.

### `.2` — a reachability inventory: which tracked gates does anything actually invoke? (`todo`)

- **Status: `todo`**.
- Build the missing inventory: for every `make` gate target and every tracked
  `.github/workflows/*.yml`, record whether it is reachable from `sota_exit_gate`,
  `ci_workflow_local_gate`, the commit workflow, or a CI workflow — and flag the orphans.
- Three orphans are already known and were each found by accident, one per session:
  `ast_dump_contract_gate`, the generated-clippy strict stage, and `ci_workflow_local_gate`'s
  own audit phase. **Finding them by accident is the defect**; the inventory is the fix.
- Consider promoting the inventory to an enforced doctrine in `scripts/check_doctrines.sh`
  (a new tracked gate that belongs to no aggregate and no workflow is a commit-time warning),
  so the class closes rather than recurring a fourth time.

## Evidence

- Measured at commit `730419a2`, session #215. The census was produced by sourcing
  `rust/scripts/ci_workflow_local_gate.sh` with its trailing `main "$@"` stripped and invoking
  each `audit_*` function in its own subshell — 31 functions total, 8 failing after the
  `assert_generated_artifact` repair.
- Provenance of the first blocker: `git log -1 --format="%h %ad %s" --date=short 0ed2b2ad` →
  `0ed2b2ad 2026-04-29 Slice 5: stop tracking generated/* in git`;
  `git merge-base --is-ancestor 0ed2b2ad HEAD` → true;
  `git rev-list --count 0ed2b2ad..HEAD` → `1371`.

## Commit log

| slice | leaf | commit subject |
|---|---|---|
| `PGEN-CI-PARITY-GATE-ROT-0004` | `.3` close-out (+ `.5` opened) | the prepared replay finished 10/1 — and the 1 is `sota_exit_gate` RED on a sub-gate that can only pass when SV generation FAILS |
| `PGEN-CI-PARITY-GATE-ROT-0003` | `.3` (+ `.4` opened) | the workflow phase runs for the first time in 1,371 commits — 3/8 unprepared, one defect behind all eight, and a mistyped filter had been certifying parity while replaying nothing |
| (opened by `PGEN-GENERATED-LINT-CORRECTNESS-0004`) | (tree opened) | the local CI-parity gate has been unable to complete for 1,371 commits — first blocker repaired, 8 independent audits routed here |
| `PGEN-QUANT-PLUS-ITER-0005` | (`.1b` follow-on) | `ast_dump_contract_gate` repaired and given the surface audit that makes it reachable — audit phase 31 → 32 |
| `PGEN-CI-PARITY-GATE-ROT-0002` | `.1b` | the director ruled the boundary STANDS — the 5 consumer-facing internal-path citations removed, the assertion re-scoped to the recipe section, 31/31 |
| `PGEN-CI-PARITY-GATE-ROT-0001` | `.1` | 12 stale assertions across 8 audits adjudicated one at a time — 23/8 → 30/1, and fail-fast had hidden a third of them |

### `.1b` — execute the director's ruling on the escalated audit (`done`)

- **Status: `done`** (2026-07-27, session #216, `PGEN-CI-PARITY-GATE-ROT-0002`).
- **DIRECTOR RULING (verbatim):** *"We will republish PGEN regex parser at a later time"* +
  *"I agree we shouldn't cite either regex.json or regex.ebnf"* ⇒ **the boundary STANDS**; the
  audit is right and is NOT retired.
- **What was done, both halves (neither is sufficient alone):**
  1. **The 5 consumer-facing citations removed** from `## Generated Parser Build Recipe`: the
     internal JSON intermediate dropped from the prose, the `$(REGEX_JSON) ← grammars/regex.ebnf`
     dependency row deleted, the determinism statement re-pinned to "the same PGEN checkout", the
     troubleshooting row reworded to describe the condition without naming the internal grammar.
  2. ⭐ **The 5th site turned out to document a REMOVED mechanism.** *"Optional: typed-entry-point
     fast path"* told consumers to regenerate with `--enable-parser-hooks` for
     `parse_regex_typed()` — both **retired** by `PARSER-NEUTRALITY.1` (director ruling
     2026-07-20), as this contract's own Maintenance Update at `:123` states. Verified: zero
     `parse_regex_typed` in `rust/src/`. Replaced with an explicit *"Removed"* subsection, so the
     citation goes **and** a consumer stops being told to use a build variant that no longer
     exists.
  3. **The assertion re-scoped** whole-file → the recipe section
     (`assert_markdown_section_not_contains`). ⛔ Not a relaxation: the whole-file form could only
     be satisfied by deleting ~35 **historical provenance** notes recording which grammar rule
     changed in a released slice — back-dating the maintenance record.
- **Verified — RED/GREEN, because a re-scoped assertion is worthless unless it still bites:**
  **RED-A** a citation re-added to the recipe section ⇒ **FAIL**;
  **RED-B** the section renamed away ⇒ **FAIL with an explicit "section not found" refusal, not a
  silent pass** (the *"a check that cannot run must SAY SO"* principle applied to the new helper
  itself); **GREEN** restored ⇒ **31/31**. 10/10 doctrines PASS.

### `.3` — the gate's WORKFLOW phase has still never been exercised (`done`)

- **Status: `done`** (2026-07-28, session #218, `PGEN-CI-PARITY-GATE-ROT-0003`). Opened
  2026-07-27 session #216 by `.1b`, on measuring what `.1` did NOT cover.
  ⛔ *"Audit phase 32/32" is not "the gate completes"* — and now it has completed.
- **Charter (as written by `.1b`).** `main()` runs the 32 audits and then `copy_tracked_worktree` +
  **11 `run_workflow` replays**. `.1`/`.1b` fixed the audit phase only, so the tree's headline claim
  (*"has not been able to complete for 1,371 commits"*) was half-answered at best. The charter also
  predicted the export-visibility asymmetry would bite. **It does, and it is the whole story.**

#### ⭐⭐ THE MEASURED CENSUS — 3 PASS / 8 FAIL, and all eight are ONE defect

`run_workflow` calls `fail`, which `exit`s, so the gate can only ever name ONE broken replay per
run — the identical fail-fast blindness that hid a third of `.1`'s stale assertions. New driver
`docs/tasks/artifacts/ci_parity_gate_rot/run_workflow_census.sh` sources the gate, stubs the
already-proven audits, and runs each replay in its own subshell so the whole census is visible at
once. Full capture: `docs/tasks/artifacts/ci_parity_gate_rot/workflow_census_layer1.txt`.

| replay | verdict | first error |
|---|---|---|
| `annotation-contract-gate` | **FAIL** 31s | `error: couldn't read src/../../generated/return_annotation_parser.rs` |
| `annotation-nonbootstrap-e2e-gate` | **FAIL** 0s | `missing return annotation JSON at …/generated/return_annotation.json` |
| `branch-protection-contract-gate` | PASS 0s | — |
| `differential-regression-gate` | **FAIL** 1s | same rustc error |
| `ebnf-frontend-dual-run-diff` | **FAIL** 2s | same rustc error |
| `rtl-frontend-generated-contract-gate` | **FAIL** 1s | same rustc error (nested one log deeper, in its probe log) |
| `stimuli-cross-family-platform-gate` | **FAIL** 1s | same rustc error |
| `mdbook-docs-gate` | PASS 1s | — |
| `fixed-point-gate` | PASS 23s | — |
| `performance-gate` | **FAIL** 1s | same rustc error |
| `sota-exit-gate` | **FAIL** 53s | same rustc error, via its first failing sub-gate `annotation_contract_gate` |

⭐ **The three PASSes are exactly the three replays that never need a generated parser** —
`branch-protection-contract-gate` is shell + `jq`, `mdbook-docs-gate` is `mdbook`, and
`fixed-point-gate` builds `ast_pipeline_bootstrap` **without** `--features generated_parsers`.
Every other replay compiles the crate, and the crate cannot compile.

#### ⭐⭐⭐ ROOT CAUSE — and it is NOT eleven problems, it is one 15-day ordering accident

`rust/src/lib.rs` includes generated parsers **two different ways**, and the difference is the
whole finding:

- nine sites are `include!(env!("PGEN_<X>_PARSER_PATH_RESOLVED"))` behind a `has_generated_*` cfg
  that `rust/build.rs` sets only when the artifact `is_file()` — absence merely **disables** them;
- **two sites are literal-path** `include!("../../generated/return_annotation_parser.rs")` and
  `…/semantic_annotation_parser.rs` (`lib.rs:72,78`) with **no cfg at all** — absence is a **hard
  rustc error** that takes the entire crate down.

That asymmetry was already banked by `GENERATED-LINT-CORRECTNESS.3`; this leaf is where it bites.
`copy_tracked_worktree` exports `git ls-files` output ONLY (`git ls-files generated/ | wc -l` → `0`;
`.gitignore:24`), so the export dir has no generated parsers at all.

⭐ **The timeline is the finding, and the ORDER is why nothing objected:**

| commit | date | what it did |
|---|---|---|
| `af85a5fd` | **2026-04-14** | *"Pause hosted CI automatic triggers"* — every workflow becomes `workflow_dispatch`-only |
| `0ed2b2ad` | **2026-04-29** | *"Slice 5: stop tracking generated/\* in git"* — a fresh checkout no longer carries `generated/` |

**Fifteen days apart, in that order.** The automatic runs were switched off FIRST, and the change
that structurally breaks a fresh checkout landed SECOND — so there was no run left to fail. This is
the same `0ed2b2ad` that broke the gate's *first audit*; the audit phase and the workflow phase were
felled by one commit, and both stayed down for **1,371** commits.

#### ⭐⭐ THE SECOND FINDING — a mistyped filter CERTIFIED PARITY WHILE REPLAYING NOTHING

`is_selected` accepted any string, so a name matching nothing skipped all eleven replays and the
gate then printed *"all selected local workflow commands passed"* → `✅ Local GitHub workflow parity
gate passed`, **exit 0**. Measured verbatim:

```
$ PGEN_CI_WORKFLOW_LOCAL_FILTER=typo-that-matches-nothing make -C rust ci_workflow_local_gate
… 32 audits …
skip annotation-contract-gate (filtered)          ← ×11
all selected local workflow commands passed
✅ Local GitHub workflow parity gate passed.      GATE_EXIT=0
```

The audits still ran, so the run was **not** empty — which is exactly what made the green
convincing. This tree's own principle, one level in: *a check that cannot run must SAY SO, not
return green.*

#### ⭐⭐⭐ THE FIX — and the gate COMPLETES, for the first time in 1,371 commits

⛔ **Copying the developer's `generated/` into the export dir was REJECTED**: it would make the gate
green against artifacts a fresh checkout does not have — the exact vacuity class this tree exists to
remove. What ships instead:

1. **`prepare_generated_artifacts`** replays the repository's OWN cold-clone bootstrap inside the
   export dir — `regex_parser_bootstrap` (whose recipe is literally *"Bootstrap regex parser from
   cold clone"* and seeds `generated/ebnf.rs` when missing), then `annotation_parsers`, then the
   seven `focus_*` targets. ⭐ **This is not a new invention**: it is byte-for-byte the sequence the
   tracked hosted workflow `.github/workflows/generated-clippy-correctness-gate.yml` already runs in
   its *"Regenerate the generated parsers"* step, which `GENERATED-LINT-CORRECTNESS.3` added for
   precisely this reason. Opt in with `PGEN_CI_WORKFLOW_LOCAL_PREPARE=1`.
2. **`preflight_generated_artifacts`** derives the required set **from `rust/src/lib.rs` itself** —
   grepping the literal-path `include!()` form, which cannot drift, and which discriminates: it
   yields exactly the 2 unguarded artifacts and **none** of the 9 cfg-guarded `env!()` ones.
3. **`explain_missing_generated_failure`** attaches the cause to the failure. Before, the operator's
   entire evidence was `error: could not compile pgen (lib)` — which points the diagnosis at the
   Rust sources instead of at the export model.
4. **`assert_workflow_selection_was_real`** closes the vacuous green: an unknown filter entry, or a
   run that replayed zero workflows, now refuses and prints the roster. The roster and the run count
   are **derived by `run_workflow` from its own call sites** — no hand-kept list to drift.
5. The closing line now reports `($WORKFLOWS_RUN_COUNT replayed)`, so a narrowed run can never again
   read as a full one.

⭐ **PREPARED CENSUS — the workflow phase actually runs.** Cold-clone preparation succeeded in
**258 s** from a bare tracked tree (much cheaper than the charter feared), and the replays then went:

| replay | verdict |
|---|---|
| `annotation-contract-gate` | **PASS** 2301s |
| `annotation-nonbootstrap-e2e-gate` | **PASS** 210s |
| `branch-protection-contract-gate` | **PASS** 1s |
| `differential-regression-gate` | **PASS** 34s |
| `ebnf-frontend-dual-run-diff` | **PASS** 58s |
| `rtl-frontend-generated-contract-gate` | **PASS** 45s |
| `stimuli-cross-family-platform-gate` | **PASS** 441s |
| `mdbook-docs-gate` | **PASS** 0s |
| `fixed-point-gate` | **PASS** 18s |
| `performance-gate` | **PASS** 35s |
| `sota-exit-gate` | **FAIL** 8587s — ⭐ a **pre-existing** defect this replay surfaced, routed to `.5` |

**Final: `workflows=11 PASS=10 FAIL=1`**, guard `exit=0 peak_tree_rss=11006MB elapsed=12056s`
(against a 12,288 MB budget — 90% used, and deliberately NOT raised, per `OPS-MEMSAFE`'s ruling
that a budget below the OS-kill point buys a deterministic exit 97).

⭐ **The one failure is not about this fix — the fix is what let the replay run for 2 h 23 m and
clear 19 required sub-gates before reaching it.** It dies in `sv_failure_context_contract_gate` on
`error: expected at least one generation failure-context excerpt`, and that is **measured NOT
export-specific**: the same gate run directly in the main repo fails identically in 232 s. Routed
to `.5` rather than absorbed here — it is a defect in `sota_exit_gate`, not in the parity gate.

#### ⚠️ TWO DEFECTS THIS LEAF FOUND IN ITS OWN WORK

1. ⭐ **The first implementation REFUSED up front (exit 2) whenever `generated/` was absent — and a
   CONTROL ARM caught that it would have broken runs that currently WORK.**
   `PGEN_CI_WORKFLOW_LOCAL_FILTER=branch-protection-contract-gate` passes today (measured, 56 s):
   that replay is shell + `jq` and needs no generated parser, as do two of the eleven others.
   Refusing for all of them would have traded a bad diagnosis for a lost capability. Corrected to
   **warn-and-continue**, with the cause re-attached in `run_workflow`'s failure path — where the
   operator is actually looking. **A gate must not fail runs it can genuinely complete.**
2. ⭐ **The preparation log measured 7.1 GB** — `rust/Makefile:93-94` runs the generator as
   `--generate-parser --debug --trace`, so seven grammars' worth of PGEN trace lands in one file
   (top repeated shapes: `[PGEN][LOW] 🧭 …`, `[PGEN][HIGH] 🔎 …`, `[PGEN][DBG] 🧠 …`). Harmless on
   this 3.6 TB volume; **fatal on a hosted runner with ~14 GB free**. The capture is now bounded to
   its last 4 MiB — `make` stops AT the failing step, so the tail is exactly where the evidence is.
   ⛔ Quietening the shared recipe was rejected as out of scope: it belongs to the hosted workflow
   too, and changing what evidence it leaves is a different change with a different owner.

#### ⛔⛔ ROUTED OUT, NOT SWALLOWED — new leaf `.4`, and it is the bigger half

Measured while root-causing this: **only 1 of the 15 tracked workflow files declares a regeneration
step**, and it is the one `GENERATED-LINT-CORRECTNESS.3` fixed. `actions/checkout` produces exactly
the tracked-files-only tree this leaf reproduced, so the same eight workflows would fail the same
way on a fresh hosted runner. Full evidence and the reasoning are in `.4`; the local gate's
`PREPARE` default deliberately stays `false` until `.4` lands, because defaulting it to `true` would
make the local gate green while the hosted side stays broken — **false parity is worse than a
visible red.**

#### Acceptance checklist (enforced)

- [x] **REPRODUCE / ISSUE** — `bash docs/tasks/artifacts/ci_parity_gate_rot/run_workflow_census.sh`
      under `scripts/run_with_memory_guard.sh --budget-mb 12288` → `workflows=11 PASS=3 FAIL=8`,
      guard marker `reason=none peak_rss_mb=1442 elapsed_s=181`; and
      `PGEN_CI_WORKFLOW_LOCAL_FILTER=typo-that-matches-nothing make -C rust ci_workflow_local_gate`
      → `GATE_EXIT=0` after eleven `skip … (filtered)` lines.
- [x] **ROOT CAUSE (WHY + WHERE)** — `git ls-files generated/ | wc -l` → `0`;
      `git log -1 --format='%h %ad %s' --date=short 0ed2b2ad` → `0ed2b2ad 2026-04-29 Slice 5: stop
      tracking generated/* in git`; `git log --format='%h %ad %s' -S'workflow_dispatch' --
      .github/workflows/annotation-contract-gate.yml` → `af85a5fd 2026-04-14 Pause hosted CI
      automatic triggers`; `git rev-list --count 0ed2b2ad..HEAD` → `1371`. The compile-side locus is
      `error: couldn't read src/../../generated/return_annotation_parser.rs` → `src/lib.rs:72:9`,
      `error: could not compile pgen (lib) due to 1 previous error`, in 6 of the 8 replay logs
      directly and in the other 2 one sub-log deeper.
- [x] **FIX** — declarative first: the required-artifact set is DERIVED from `rust/src/lib.rs`
      rather than hand-listed, and the replay roster is DERIVED from `run_workflow`'s own call
      sites. No new invention — `prepare_generated_artifacts` replays the repo's existing
      cold-clone recipe, already shipped in `.github/workflows/generated-clippy-correctness-gate.yml`.
- [x] **ADDRESSED (verified)** — before→after on the census: unprepared `PASS=3 FAIL=8`
      → with `PGEN_CI_WORKFLOW_LOCAL_PREPARE`-equivalent cold-clone preparation (`prepare: OK (258s)`)
      **10 replays PASS and 0 FAIL** (see the prepared table above; the 11th, the `sota-exit-gate`
      aggregate, was still running at commit time with zero `fail` lines), i.e. the workflow phase
      executes for the first time since `0ed2b2ad`. Vacuity arm: the typo filter that returned
      `GATE_EXIT=0` now blocks with `unknown PGEN_CI_WORKFLOW_LOCAL_FILTER entry`.
      Re-runnable oracle: `bash docs/tasks/artifacts/ci_parity_gate_rot/run_workflow_phase_probes.sh`.
- [x] **NO REGRESSION** — `bash -n rust/scripts/ci_workflow_local_gate.sh` clean; the probe driver
      replays every arm against BOTH the pre-change gate (`git show HEAD:…`) and the working tree,
      and the CONTROL arms (`branch-protection-contract-gate` and `mdbook-docs-gate` replays, plus
      an unrelated forced failure that must NOT be mislabelled) are **identical on both sides**.
      No `grammars/*.ebnf`, no `rust/src/*`, no `generated/*` in this change ⇒ all 11 parsers
      byte-identical BY CONSTRUCTION; no release / schema / ledger / contract movement.
      `bash scripts/check_doctrines.sh` and `make -C rust SHELL=/bin/bash mdbook_docs_gate` green.
- [x] **LOCKSTEP** — `README.md` standard-commands entry for the gate's new `PREPARE` knob,
      `docs/book/src/operations-and-governance.md`, `CHANGES.md`, `DEVELOPMENT_NOTES.md`,
      `MEMORY.md`, and this tree (leaf `.3` + new leaf `.4`).

#### Evidence

- `docs/tasks/artifacts/ci_parity_gate_rot/run_workflow_census.sh` — the per-replay census driver.
- `docs/tasks/artifacts/ci_parity_gate_rot/run_workflow_phase_probes.sh` — RED / GREEN / CONTROL
  arms, each replayed against the pre-change gate and the working tree.
- `docs/tasks/artifacts/ci_parity_gate_rot/workflow_census_layer1.txt` — the unprepared census.
- `docs/tasks/artifacts/ci_parity_gate_rot/workflow_census_prepared.txt` — the prepared census.

### `.4` — the HOSTED workflows have the same defect, and 14 of 15 never got the fix (`todo`)

- **Status: `todo`** — opened 2026-07-28 session #218 by `.3`, from evidence gathered while
  root-causing the local gate. ⛔ **Deliberately NOT folded into `.3`**: `.3` owns the local gate,
  this owns the tracked hosted workflow surface, and the two have different verification stories.
- ⭐⭐ **THE FINDING.** `actions/checkout` produces a tracked-files-only tree — precisely what
  `copy_tracked_worktree` produces, and precisely what `.3` measured 8 replays dying against.
  Measured over the tracked workflow surface: **only `generated-clippy-correctness-gate.yml`
  declares a regeneration step** (added by `GENERATED-LINT-CORRECTNESS.3`, which hit this problem
  first). The other 14 go Checkout → Setup Rust → Cache → `make -C rust <gate>`.
  ⇒ **the same eight workflows would fail the same way on a fresh hosted runner.**
- **Why nothing objected** — the ordering in `.3`'s timeline table: hosted auto-triggers were paused
  `2026-04-14`, `generated/` was untracked `2026-04-29`. The only workflow that still auto-runs is
  `memory-architecture-gate.yml` (push + pull_request), and it is a pure shell check, so its green
  says nothing about the other fourteen.
- ⚠️ **Honest limit on the claim, stated up front.** Hosted Actions are paused and billable, so this
  was NOT proven by dispatching a run. The evidence is (a) the local reproduction against a
  tracked-files-only tree, which is what `actions/checkout` yields, and (b) the measured absence of
  a regeneration step in 14 of 15 tracked workflow files. `Swatinem/rust-cache@v2` does not close
  the gap: it caches the cargo registry and target dir, not the untracked `generated/` tree.
- **Scope when taken up:**
  1. add the shipped *"Regenerate the generated parsers"* step to each workflow measured to need it
     (the 8 from `.3`'s census; the 3 that pass need nothing and must not pay for it);
  2. consider hoisting the sequence into a composite action or a `make` target so the recipe has ONE
     home rather than N copies — it is already duplicated between the workflow and `.3`'s
     `prepare_generated_artifacts`;
  3. then, and only then, flip `PGEN_CI_WORKFLOW_LOCAL_PREPARE` to default `true`, so the local gate
     mirrors a hosted side that genuinely works;
  4. extend `audit_workflow_surface` so a workflow whose command needs generated artifacts but
     declares no regeneration step FAILS the audit — otherwise this recurs a fourth time.
- ⛔ **Do not flip the local default before (1).** A local green over a broken hosted side is false
  parity, which is worse than the visible red the gate reports today.

### `.5` — `sota_exit_gate` is RED: a required sub-gate that can only pass when SV generation FAILS (`todo`)

- **Status: `todo`** — opened 2026-07-28 session #218 by `.3`, which surfaced it by being the first
  thing in 1,371 commits actually to run the aggregate. ⛔ Deliberately NOT absorbed into `.3`:
  `.3` owns the parity gate, and the parity gate is **right** here — it is faithfully reporting a
  defect that lives in `sota_exit_gate`.
- ⭐⭐ **THE HEADLINE.** `make -C rust SHELL=/bin/bash sota_exit_gate` — the repository's flagship
  aggregate and a `README.md` Standard Command — **cannot complete.** It clears 19 required
  sub-gates and then dies in `sv_failure_context_contract_gate`:

  ```
  ==> sv_failure_context_contract_gate (required)
      systemverilog_failure_context_aggregate_contract_gate   ok
  error: expected at least one generation failure-context excerpt
  ```

- ⭐ **NOT export-specific — measured on both sides.** Run directly in the main repo
  (`scripts/run_with_memory_guard.sh --budget-mb 12288 -- make -C rust sv_failure_context_contract_gate`)
  it fails identically: guard `exit=2`, `peak_tree_rss=9870MB`, **232 s**. Both trees report
  `total_counterexamples = 0` and `by_failure_context_excerpt | length = 0`.
- ⭐⭐⭐ **AND THE COUNTEREXAMPLE PIPELINE IS NOT BROKEN — IT IS CORRECTLY REPORTING ZERO.** That
  distinction was the whole risk here (`.1`'s question: *is the AUDIT stale, or is the REPO
  wrong?*), and the two readings have **opposite** remedies. It is settled by
  `systemverilog_parseability_generation_report.json`:

  ```json
  "observed": { "requested_total": 1, "accepted_total": 1, "rejected_total": 0,
                "attempts_total": 1, "parser_rejections_total": 0,
                "generation_errors_total": 0, "acceptance_rate_percent": 100.00 }
  ```

  **One sample was requested, and the generated SV parser accepted it on the first attempt.** There
  is genuinely no failure to excerpt. `sv_failure_context_contract_gate.sh:136` then requires
  `>= 1`, so ⇒ **under the budget this gate itself configures, the assertion can only be satisfied
  if the SV parser REJECTS its own generated sample.** It passes when the system is broken and
  fails when it works.
- ⭐ **A FOURTH SHAPE FOR THIS FAMILY.** The tree has now found: a check that *cannot run* and
  returns green (the filter vacuity, `.3`); a check that *cannot see* and returns green (the
  generated-clippy vacuity, `GENERATED-LINT-CORRECTNESS.3`); a check that *nothing invokes*
  (`ast_dump_contract_gate`); and now **a check that requires a DEFECT to be present in order to
  pass.** The unifying principle needs its converse stated: *a check must not depend on the thing
  it watches being broken.*
- **The adjudication this leaf owes (⛔ do not guess — the readings have opposite fixes):**
  1. **Is the budget the bug?** `requested_total: 1` is what this gate's own failure-context quality
     state configures. If the intended budget is larger, the gate is mis-wired and the fix is the
     budget, not the assertion.
  2. **Is the assertion the bug?** If a clean SV generation run is the expected and desired outcome,
     then `>= 1 excerpt` is an inverted-vacuity check and must become "if any counterexample exists,
     its excerpt must be well-formed" — keeping the anti-vacuity intent without requiring a defect.
  3. Either way, establish **when this last passed** (`git log -S` on the assertion and on the
     budget) — the ordering will probably show the same shape as `.3`'s: a change that made the
     system better, landing after the only thing that would have objected stopped running.
- ⚠️ **Scope note.** If (1) turns out to be an SV-family question rather than a proof-surface one,
  re-home this leaf to the owning SV tree; it is filed here because *"a required sub-gate of the
  flagship aggregate has been failing undetected"* is proof-surface integrity, which is this tree.
- **Evidence:** `docs/tasks/artifacts/ci_parity_gate_rot/workflow_census_prepared.txt` (the
  replay's verdict plus the main-repo comparison and the `observed` block).
