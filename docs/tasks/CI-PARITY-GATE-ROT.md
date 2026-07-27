# CI-PARITY-GATE-ROT: the local CI-parity gate — the one the README names as the way to prove workflow parity while hosted Actions are paused — has not been able to complete for 1,371 commits

## Metadata

- Tree ID: `CI-PARITY-GATE-ROT`
- Status: `active` (opened 2026-07-27, session #215)
- Family / slice-id prefix: `PGEN-CI-PARITY-GATE-ROT-<NNNN>`
- Created: `2026-07-27`
- Owner: repo-local workflow
- **Frontier: `.2`** (`.1` **done** 2026-07-27 session #216 — audit phase **23 PASS / 8 FAIL → 31 PASS /
  0 FAIL**; the escalated row was ruled on by the director same-session and executed as `.1b`)
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
| (opened by `PGEN-GENERATED-LINT-CORRECTNESS-0004`) | (tree opened) | the local CI-parity gate has been unable to complete for 1,371 commits — first blocker repaired, 8 independent audits routed here |
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
