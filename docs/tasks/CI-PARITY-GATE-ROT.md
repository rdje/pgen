# CI-PARITY-GATE-ROT: the local CI-parity gate — the one the README names as the way to prove workflow parity while hosted Actions are paused — has not been able to complete for 1,371 commits

## Metadata

- Tree ID: `CI-PARITY-GATE-ROT`
- Status: `active` (opened 2026-07-27, session #215)
- Family / slice-id prefix: `PGEN-CI-PARITY-GATE-ROT-<NNNN>`
- Created: `2026-07-27`
- Owner: repo-local workflow
- ✅✅ **TREE CLOSED — 2026-07-28, session #220.** Every leaf is `done` and the director's ordered
  scope is discharged in full: **`.4`** (hosted regeneration step, one home, derived fail-safe
  audit) → **`.5`** (the earned-zero replacement; the flagship aggregate's RED sub-gate) → **the
  `PREPARE` flip** (default `true`, guarded) → **`.2`** (the reachability inventory, 31 orphans
  dispositioned behind a ratchet, shipped as the 11th enforced doctrine `GATE-REACHABILITY`).
  ⛔ **Discharged WITHOUT opening a single new leaf**, per the director's explicit constraint: every
  finding met on the way — two workflows nobody had measured, two `timeout-minutes` below their own
  measured cost, ten orphaned per-parser book gates, an unsound self-referential guard — was fixed
  **in place**.
  ⚠️ **The one honest residual, recorded and not called done:** 28 of the 31 dispositions are
  `accepted-operator-invoked`, i.e. real proof lanes that nothing runs automatically. That is an
  ACCEPTED RISK the register states in its own text; shrinking it by wiring lanes into aggregates as
  their cost allows is genuine remaining work, now VISIBLE and ratcheted instead of invisible.
  (`.5` **done** 2026-07-28 session #220 — all three `expected at least one …` assertions replaced
  with the earned-zero form; measured: ALL THREE were unsatisfiable, and the sub-gate that made
  `sota_exit_gate` RED now passes end-to-end.)
  (`.4` **done** 2026-07-28 session #220 — the hosted surface: **11 of 15** tracked workflows need
  the regeneration step and only **1** declared it; the recipe now has one home
  (`make regenerate_generated_parsers` + the `regenerate-parsers` composite action), a derived
  fail-safe audit holds it, and two workflows were found declaring timeouts below their own
  measured cost.
  `.1` **done** 2026-07-27 session #216 — audit phase **23 PASS / 8 FAIL → 31 PASS /
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

### `.flip` — `PGEN_CI_WORKFLOW_LOCAL_PREPARE` defaults to `true` (`done`)

- **Status: `done`** (2026-07-28, session #220, `PGEN-CI-PARITY-GATE-ROT-0009`). This is step (c) of
  the director's ordered scope, and it is a one-line change with a three-session prerequisite.
- ⭐ **WHY IT COULD NOT BE DONE EARLIER, AND WHY IT CAN BE DONE NOW.** `.3` shipped the knob
  defaulting to `false` deliberately: at that point **14 of the 15** tracked hosted workflows had no
  regeneration step, so a `true` default would have produced a green local gate standing in for a
  hosted side that was still broken — *false parity, worse than the visible red the gate reported.*
  `.4` removed the condition (11 of 15 need generated parsers; all 11 now regenerate through the
  composite action), so a local green and a hosted green mean the same thing again.
- **Cost, stated honestly.** Preparation engages whenever the export dir lacks an artifact
  `rust/src/lib.rs` includes by literal path — which, since the export is `git ls-files` output, is
  always. Measured **236 s**. For a full gate run that is exactly right. ⚠️ For a NARROWED run it can
  be waste: three replays never compile the crate. The gate cannot decide that for the operator (the
  replay roster is only complete after the replays have run), so with a filter set it now **prints
  the opt-out** rather than guessing.
- ⭐ **THE DEFAULT IS GUARDED, because this project has watched exactly this erosion before**:
  `PGEN_CLIPPY_GENERATED_STRICT` defaulted to `0` and was set by no gate, no aggregate and no
  workflow, leaving a 291 → 0 correctness win unguarded from the day it landed
  (`GENERATED-LINT-CORRECTNESS.3`). `audit_workflow_regeneration_surface` now fails if this default
  is not `true`.
- ⚠️⚠️ **AND THE OBVIOUS WAY TO WRITE THAT GUARD IS UNSOUND — caught by this leaf's own probe arms.**
  The first cut was `assert_file_not_contains <this file> '<the forbidden literal>'`, which puts the
  forbidden literal INTO the file it forbids it from: **8 of 13 arms failed on the audit tripping
  over its own source.** The positive form is no better — it would match its own text and pass
  vacuously. ⇒ **an assertion about a file cannot live inside that file as a literal**; the
  `PGEN_CLIPPY_GENERATED_STRICT` precedent only works because the file it asserts on is a DIFFERENT
  one. The guard now extracts the declared default and compares the **value**, with the pattern
  anchored at column 0 so neither the guard nor its comment can match itself.
- **Verified**: `run_regeneration_surface_probes.sh` **13/13**, with new **RED-9** flipping the
  default back to `false` and requiring the audit to block; `bash -n` clean; `mdbook_docs_gate`
  GREEN; 10/10 doctrines.

### `.2` — a reachability inventory: which tracked gates does anything actually invoke? (`done`)

- **Status: `done`** (2026-07-28, session #220, `PGEN-CI-PARITY-GATE-ROT-0010`).
  **123 targets inventoried: 92 reachable, 30 orphan, 1 policy-only — all 31 dispositioned, behind
  a ratchet, as the 11th enforced doctrine `GATE-REACHABILITY`.**

#### ⭐⭐ THE INSTRUMENT GAVE SIX DIFFERENT CONFIDENT ANSWERS, AND ONLY GROUND TRUTH CAUGHT IT

This is the headline, because it is the part that would have shipped a lie. Successive versions of
the reachability scan reported **97, 71, 75, 93, 53 and 40** orphans. Every one of those numbers was
produced confidently, and **not one of the six defects was found by reading the code** — each was
caught by requiring the output to reproduce facts this project had already established the expensive
way:

| # | defect | what it did |
|---|---|---|
| 1 | a MENTION counted as an INVOCATION | the parity gate's SURFACE AUDITS read `ast_dump_contract_gate.sh` and `clippy_on_rust_change.sh`; counting that as invocation reported both **reachable** — contradicting the measured fact that `ast_dump_contract_gate` sat RED for four sessions *because nothing runs it* |
| 2 | host syntax around a real command | in a workflow the line is `run: make …`, in `COMMIT.md` a backticked bullet ⇒ `mdbook_docs_gate` and `branch_protection_contract_gate` came out ORPHAN although a tracked workflow runs each |
| 3 | make's `@` recipe prefix | `annotation_contract_gate` composes twenty sub-gates as `@$(MAKE) -C …`; missing the prefix orphaned all twenty |
| 4 | backslash line continuations | a continued `assert_file_contains \` put the SCRIPT PATH in command position on the next line — the source of defect 1's surviving edge |
| 5 | a nested `make` inside a runner's argv | `sota_exit_gate.sh` calls its sub-gates as `run_check … make -C rust <target>`; once continuations were correctly joined, the command word became `run_check` and the aggregate showed **zero** sub-gates |
| 6 | a prerequisite list held in a make variable | `parser_books_gate: $(PARSER_BOOK_GATES)` — re-implementing `$(wildcard)`/`$(patsubst)` here would be a second copy of make's semantics, so the check now **asks make** via a read-only `print-%` target |

⇒ **the controls are now part of the check**: six pinned facts plus every check the SOTA policy
marks REQUIRED must come out reachable, and if they do not the check prints **MISCALIBRATED and
exits nonzero** instead of reporting its numbers. A reachability report nobody can check is a
confident guess, and a *wrong* one would certify the exact rot this doctrine exists to find.

#### The design, and why each choice is the non-rotting one

- **The universe is derived two ways and unioned**: a `*_gate` target, or any target whose recipe
  runs a `rust/scripts/*.sh`. The second arm is what catches lanes that are gates in everything but
  name — `clippy_on_rust_change` — which a name-only filter would have silently excluded.
- **The edges are derived** from `rust/Makefile`, `rust/scripts/*.sh`, `.github/workflows/*.yml`,
  `.githooks/*`, `scripts/check_*.sh` and `COMMIT.md`, under a **command-position** rule: a
  reference counts only when the thing is being executed.
- **`COMMIT.md` policy is its own invoker class**, never merged with the machine-enforced ones. A
  gate whose only invoker is a prose instruction is precisely the class this leaf exists to surface:
  nothing fails if it is skipped. Exactly one target is in it — `clippy_on_rust_change`, the residual
  `.2`'s charter already named.
- ⛔ **A RATCHET, NOT A REPORT.** `DOCTRINE-GAP-OWNERSHIP.1`'s lesson applies directly: a one-shot
  census is the same disease one level up, because it gets written down and nothing re-applies it.
  The orphan set is re-derived every run and joined against a tracked register; an **untriaged
  orphan fails**, and so does a **register entry that no longer names an orphan**, so the exemption
  list can neither be bypassed nor accumulate dead weight.

#### The one class that was WIRED rather than accepted — and it was ten gates

The inventory's first real output: **all ten per-parser book gates were orphaned**, while `README`
carries a standing director directive that every PGEN parser ships its own live mdBook. Measured cost
to run one: **0.3 s**. They are now a derived `parser_books_gate` (from `wildcard
rust/scripts/*_parser_book_gate.sh`, so an eleventh book is covered without anyone remembering)
hanging off `mdbook_docs_gate` — which a tracked workflow, the commit workflow and the parity gate's
replay all already invoke. Total addition: **~3 s**. ⛔ They are PREREQUISITES, not a shell loop:
a `for g in …; do $(MAKE) "$$g"; done` runs them correctly but makes the edges invisible to any
static reader, which is a weaker version of the very problem being solved.

#### The 31 dispositions — and the honest limit on 28 of them

`rust/test_data/grammar_quality/gate_reachability_register_v0.json`:

| disposition | n | meaning |
|---|---|---|
| `accepted-operator-invoked` | 28 | a real proof lane, deliberately outside an aggregate because of cost or because it is a campaign/closure instrument (regex external-corpus + oracle lanes, SV/VHDL family closure + telemetry lanes, the parse-harness oracles, `bin_build_integrity_gate`, `duality_hunt_gate`, `verilog_2005_conformance_gate`, the AST-shape contracts, `ast_dump_contract_gate`) |
| `policy-invoked` | 1 | `clippy_on_rust_change` — COMMIT.md step 2 and nothing machine-enforced |
| `alias-of-reachable` | 1 | `generated_clippy_correctness_policy` — its real work runs on every Rust change as `--policy-only` from a reachable path |
| `not-a-proof-lane` | 1 | `create-placeholders` — a bootstrap helper that asserts nothing; in the universe only because the universe over-approximates on purpose |

⚠️⚠️ **`accepted-operator-invoked` IS AN ACCEPTED RISK, NOT A CLEAN BILL OF HEALTH, and the register
says so in its own text.** Nothing fails if one of those 28 is never run again. What this leaf buys
is that the set is **visible, decided, and cannot grow silently** — which is exactly the gap that let
three gates rot undetected, one per session. **Shrinking the 28 by wiring lanes into aggregates as
their cost allows is real remaining work**, and it is recorded as such rather than being called done.

#### Acceptance checklist (enforced)

- [x] **ROOT CAUSE (WHY + WHERE)** — WHY nothing had this answer: no artifact in the repository maps
      gate targets to invokers, so "which gates does anything run?" could only be answered by
      accident, three times, one per session. WHERE, measured this run:
      `git ls-files '.github/workflows/*.yml'` = 15 workflow roots, `rust/Makefile` = 123 in-universe
      targets, of which **30 have no invoker at all** and 1 is invoked only by prose. The six
      calibration defects were each located by a failing ground-truth control, not by inspection —
      e.g. `grep -n 'clippy_on_rust_change' rust/scripts/ci_workflow_local_gate.sh` showing the only
      hits are `assert_tracked` / `assert_file_contains` arguments, i.e. reads, not runs.
- [x] **ADDRESSED (verified)** — before→after on the orphan set: **40 → 30** by wiring the ten book
      gates (`make -C rust parser_books_gate` → *"All 10 per-parser book gates passed"*, 1.1 s;
      `mdbook_docs_gate` end-to-end 2.9 s), and the remaining 31 dispositioned so
      `bash scripts/check_gate_reachability.sh` → `OK (123 targets; 92 reachable, 30 orphan +
      1 policy-only, all dispositioned; 6 ground-truth controls reproduced)`. Probes **8/8**,
      including RED-1 (a gate target that did not exist when the check was written ⇒ blocks — the
      arm that proves the set is derived, not a snapshot of the register), RED-3 (a dead exemption
      entry ⇒ blocks) and CTRL-2 (giving a pinned-ORPHAN control an invoker ⇒ the check declares
      itself **MISCALIBRATED** rather than quietly reporting a new number).
- [x] **NO REGRESSION** — no `grammars/*.ebnf`, no `rust/src/*`, no `generated/*` ⇒ all 11 parsers
      **byte-identical BY CONSTRUCTION**; no release / schema / ledger / contract movement.
      `bash -n` clean on the new check; `bash scripts/check_doctrines.sh` → **ALL 11 enforced
      doctrines PASS** (was 10); `make -C rust SHELL=/bin/bash mdbook_docs_gate` GREEN with the ten
      book gates now inside it. CTRL-3: an unrelated Makefile edit does not change the verdict.
- [x] **LOCKSTEP** — `DOCTRINE_ENFORCEMENT.md` §10 registry row, `scripts/check_doctrines.sh`,
      `README.md`, `docs/book/src/operations-and-governance.md`, `CHANGES.md`,
      `DEVELOPMENT_NOTES.md`, `MEMORY.md`, and this tree.

#### Evidence

- `scripts/check_gate_reachability.sh` — the check (also `--report` for the human inventory).
- `rust/test_data/grammar_quality/gate_reachability_register_v0.json` — the 31 dispositions.
- `docs/tasks/artifacts/ci_parity_gate_rot/run_gate_reachability_probes.sh` + `…_probes.txt`.
- `docs/tasks/artifacts/ci_parity_gate_rot/gate_reachability_inventory.txt` — the full inventory.

#### Original charter
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
| `PGEN-CI-PARITY-GATE-ROT-0010` | `.2` (TREE CLOSED) | the reachability inventory nobody had — 123 targets, 30 orphans + 1 policy-only, all dispositioned behind a ratchet as the 11th enforced doctrine; ten orphaned book gates wired |
| `PGEN-CI-PARITY-GATE-ROT-0009` | `.flip` | PGEN_CI_WORKFLOW_LOCAL_PREPARE defaults to `true` now the hosted side genuinely works — and the guard for that default could not be written as a literal inside the file it guards |
| `PGEN-CI-PARITY-GATE-ROT-0008` | `.5` | the flagship aggregate's RED sub-gate could only pass when SV generation FAILED — all three assertions replaced with "the zero must be EARNED", and all three were unsatisfiable |
| `PGEN-CI-PARITY-GATE-ROT-0007` | `.4` | 11 of 15 hosted workflows needed the regeneration step and 1 had it — one home for the recipe, a derived fail-safe audit, and two timeouts below their own measured cost |
| `PGEN-CI-PARITY-GATE-ROT-0005` | `.4`/`.5` prep (docs only) | the director work order banked verbatim, `.5` adjudicated (the ASSERTION is the bug, not the budget), and both leaves given a self-contained work list |
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

### `.4` — the HOSTED workflows have the same defect, and 14 of 15 never got the fix (`done`)

- **Status: `done`** (2026-07-28, session #220, `PGEN-CI-PARITY-GATE-ROT-0007`).
  **11 of the 15 tracked workflows need the regeneration step; before this leaf exactly ONE
  declared it. All eleven now do, through a single shared definition.**

#### ⭐⭐ THE TWO UNMEASURED WORKFLOWS ARE MEASURED, AND BOTH NEED THE STEP

`.3`'s census could only see the 11 workflows the parity gate replays, so `rtl-const-expr-cert-gate`
and `sv-cert-recognized-union-gate` sat outside every instrument. Measured here against a
tracked-files-only export (`git ls-files` copy — the exact shape `actions/checkout` yields), each
under `scripts/run_with_memory_guard.sh --budget-mb 12288`:

```
$ make -C rust SHELL=/bin/bash rtl_const_expr_cert_gate
==> ensure_generated_rtl_const_expr_artifacts
make[1]: *** No rule to make target `../generated/ebnf.rs', needed by
         `target/ebnf_frontend_build/debug/ast_pipeline'.  Stop.
guard: exit=2 reason=none elapsed_s=5

$ make -C rust SHELL=/bin/bash sv_cert_recognized_union_gate
==> ensure_generated_systemverilog_parser
make[1]: *** No rule to make target `../generated/ebnf.rs', needed by
         `target/ebnf_frontend_build/debug/ast_pipeline'.  Stop.
guard: exit=2 reason=none elapsed_s=5
```

⭐ **Both fail EARLIER than `.3`'s eight** — not at the rustc include, but at `make` itself: nothing
in the Makefile can produce `generated/ebnf.rs`, because only `regex_parser_bootstrap` seeds it.
Both scripts then go on to `cargo build --features "generated_parsers ebnf_dual_run" --bin
ast_pipeline` (`rtl_const_expr_cert_gate.sh:181`, `sv_cert_recognized_union_gate.sh:179`), so they
would have hit the annotation-parser include too. ⇒ **the count is 11 of 15, not 8 of 15.**

| verdict | workflows |
|---|---|
| **need the step** (11) | annotation-contract, annotation-nonbootstrap-e2e, differential-regression, ebnf-frontend-dual-run-diff, generated-clippy-correctness *(already had it)*, performance, rtl-const-expr-cert, rtl-frontend-generated-contract, sota-exit, stimuli-cross-family-platform, sv-cert-recognized-union |
| **measured-exempt** (3) | branch-protection-contract (shell+jq, PASS 0s), fixed-point (bootstrap binary WITHOUT `--features generated_parsers`, PASS 23s), mdbook-docs (mdbook only, PASS 1s) |
| **outside the predicate** (1) | memory-architecture — runs `bash scripts/check_*.sh` only, never `make -C rust`, so it needs no exemption entry at all |

#### ⭐ ONE HOME — AND IT IS TWO ARTIFACTS BECAUSE A STEP AND A RECIPE ARE DIFFERENT THINGS

Adding the eight-line recipe to ten more files would have made **twelve copies** of a sequence whose
drift nothing could detect. It now has exactly one definition, split by what can consume it:

- **the RECIPE** → `rust/Makefile`'s new `regenerate_generated_parsers` target (with
  `GENERATED_PARSER_FAMILIES` as the derived family list). This is the only place the sequence is
  written. `prepare_generated_artifacts` in `rust/scripts/ci_workflow_local_gate.sh` no longer
  spells it out — it calls this target, so the local gate and the hosted side cannot diverge.
- **the STEP** → `.github/actions/regenerate-parsers/action.yml`, a composite action that runs that
  target. A composite action cannot be called from a shell gate and a `make` target cannot carry a
  step name, comment, or `uses:` — so each owns exactly the half the other cannot.

⭐ **Verified end-to-end, not asserted**: `make -C rust SHELL=/bin/bash regenerate_generated_parsers`
run in the bare tracked export produced all ten artifacts plus the `ebnf.rs` seed —
guard marker `status=completed reason=none exit=0 peak_rss_mb=4266 elapsed_s=236`.

#### ⭐⭐ A SECOND HOSTED-SIDE DEFECT THE SAME MEASUREMENT EXPOSED — TIMEOUTS BELOW MEASURED COST

Pricing the step against each job's declared budget showed two workflows that **could never have
completed even before this leaf**, using `.3`'s own prepared-census timings:

| workflow | measured replay (`.3`) | declared `timeout-minutes` | now |
|---|---|---|---|
| `sota-exit-gate` | **8,587 s = 2 h 23 m** (and that run FAILED partway, so a green run is longer) | **60** | **300** |
| `annotation-contract-gate` | **2,301 s = 38 m** | **30** | **90** |
| `rtl-frontend-generated-contract-gate` | 45 s | 20 | 30 (the floor) |

⭐ **The flagship aggregate has carried a timeout below a quarter of its own measured cost** ever
since hosted triggers were paused — invisible for the same reason everything else in this tree was:
nothing ran it. Fixed in place per the director's constraint, not routed onward.
⚠️ **Honest limit, recorded rather than papered over:** hosted runners are slower than this volume
and hosted Actions are paused/billable, so 300 minutes is 2× the *local* measurement, not a hosted
one, and it is the one job where that margin may still not be enough. GitHub's job cap is 360.

#### The audit that stops this recurring a fourth time

New `audit_workflow_regeneration_surface` (parity audit phase **32 → 33**). Four properties, each
chosen against a failure this tree already recorded:

1. **The roster is DERIVED** — `git ls-files '.github/workflows/*.yml'`, not a hand-list. This is the
   direct fix for how the last two workflows stayed unmeasured: a hand-list cannot see a file
   nobody added to it.
2. **The polarity is FAIL-SAFE** — running a `make -C rust` gate means the step is REQUIRED by
   default; not needing it requires a measured entry in `workflow_is_regeneration_exempt`. The
   opposite polarity (list the ones that need it) is the moving-value duplication that rotted
   twelve assertions in `.1`.
3. **Exemption is asserted in BOTH directions** — `.4`'s scope says *"the 3 that pass must not pay
   for it"*, so an exempt workflow that acquires the step also fails. A 236 s regeneration bolted
   onto a 1 s shell check is a real cost regression.
4. **It refuses when it inspects nothing** — an empty roster is a `fail`, not a pass. That is this
   tree's own principle applied to the new audit itself.

Plus a **30-minute floor** on any job carrying the step (the step alone measured 236–258 s locally,
warm, on a fast volume; a hosted runner starts cold). ⛔ Deliberately a FLOOR and not a per-gate cost
table: pinning each workflow's measured runtime in the audit would be exactly the duplicated-moving-
value shape `.1` found rotting.

#### ⚠️ THE PROBE ARMS CAUGHT A DEFECT IN THIS LEAF'S OWN WORK — AND IT IS THE "RIGHT REASON" CLASS

First run of `run_regeneration_surface_probes.sh`: **10 of 12 arms failed**, and every RED arm was
reaching `FAIL` — the verdict it wanted. It was failing for the WRONG REASON: the new
`.github/actions/regenerate-parsers/action.yml` was untracked, so `assert_tracked` fired first and
every arm died on `required tracked file missing from git index` before touching the invariant it
was testing. Had the driver only compared PASS/FAIL, it would have reported **8 RED arms green over
an audit that never evaluated a single one of them.**

⭐ It was caught only because each arm also asserts a **substring of the message it expects**
(`declares no regeneration step`, `below the 30-minute floor`, `matched no workflows at all`, …).
This is `DOCTRINE-GAP-OWNERSHIP.1`'s lesson holding a second time: *a probe that passes for the
wrong reason is worse than no probe.* Also a genuine finding in its own right — the composite action
must be TRACKED or `copy_tracked_worktree` and `actions/checkout` both produce a tree where the
`uses:` reference dangles.

#### Acceptance checklist (enforced)

- [x] **REPRODUCE / ISSUE** — the two previously-unmeasured workflows run against a tracked-only
      export: both `exit=2` in 5 s with `make[1]: *** No rule to make target '../generated/ebnf.rs'`,
      guard markers `reason=none`. Export built with `git ls-files` (5,159 files, no `generated/`).
- [x] **ROOT CAUSE (WHY + WHERE)** — WHY: `generated/` is untracked, so `actions/checkout` yields a
      tracked-files-only tree; `git ls-files generated/ | wc -l` → `0`. WHERE: `rust/Makefile` has no
      rule producing `generated/ebnf.rs` (only `regex_parser_bootstrap` seeds it), and
      `rust/src/lib.rs:72,78` include the annotation parsers by literal path under
      `#[cfg(feature = "generated_parsers")]`, so under that feature their absence is a hard rustc
      error. Measured per-workflow, not inferred: guard markers `reason=none exit=2 elapsed_s=5` for
      both, and `git ls-files '.github/workflows/*.yml'` → 15 files of which only one declared a
      regeneration step. Timeline unchanged from `.3`: `git rev-list --count 0ed2b2ad..HEAD` → 1371.
- [x] **ADDRESSED (verified)** — before→after on the audit and on the recipe. BEFORE: 1 of 15
      workflows declared the step. AFTER: `regeneration surface: 11 workflow(s) require the step,
      3 measured-exempt`, all 11 wired through one action. The recipe itself verified end-to-end
      from a bare tracked tree — `regenerate_generated_parsers` guard marker
      `status=completed reason=none exit=0 peak_rss_mb=4266 elapsed_s=236`, 10 artifacts emitted.
      Probe arms **12/12** (`run_regeneration_surface_probes.sh`), including RED-6 (a workflow file
      that did not exist when the audit was written) and RED-8 (empty roster ⇒ refusal).
- [x] **NO REGRESSION** — no `grammars/*.ebnf`, no `rust/src/*`, no `generated/*` in this change ⇒
      all 11 parsers **byte-identical BY CONSTRUCTION**; no release / schema / ledger / contract
      movement. `bash -n rust/scripts/ci_workflow_local_gate.sh` clean; all 15 workflow YAMLs plus
      the new action parse; `make -C rust branch_protection_contract_gate` GREEN (the required-check
      contract still matches the workflow surface). CONTROL arms: an unrelated workflow edit does
      NOT trip the new audit, the pre-existing `audit_workflow_surface` still passes, and it returns
      the **identical verdict against HEAD's gate and the working tree** — no over-binding.
- [x] **LOCKSTEP** — `README.md`, `docs/book/src/operations-and-governance.md`, `CHANGES.md`,
      `DEVELOPMENT_NOTES.md`, `MEMORY.md`, and this tree.

#### Evidence

- `docs/tasks/artifacts/ci_parity_gate_rot/run_regeneration_surface_probes.sh` — the 12 arms.
- `docs/tasks/artifacts/ci_parity_gate_rot/regeneration_surface_probes.txt` — their capture.
- `docs/tasks/artifacts/ci_parity_gate_rot/unmeasured_workflow_census.txt` — the two measurements,
  plus a **post-commit confirmation** appended in the same session: re-run in the SAME export tree
  once `regenerate_generated_parsers` had prepared it, both newly-wired gates go **FAIL (5 s, no
  `generated/`) → PASS** (`rtl_const_expr_cert_gate` 393 s peak 10,402 MB;
  `sv_cert_recognized_union_gate` 558 s peak 5,892 MB), and both fit their declared timeouts under
  the stated rule. That is the end-to-end proof for exactly the two workflows nobody had measured.

---

#### Original charter (kept verbatim — the work list `.4` was handed)

- ⛔⛔⛔ **DIRECTOR WORK ORDER (2026-07-28, session #218→#219, verbatim):** *"Please fix these once
  and for all, so that we can get back to real work … 1. `make -C rust sota_exit_gate` is RED right
  now — and the sub-gate can only pass when SV generation FAILS (new leaf `.5`) and The hosted
  workflows have the same defect — 14 of 15 never got the fix (new leaf `.4`) … so that we can set
  `PGEN_CI_WORKFLOW_LOCAL_PREPARE` back to true again. It is taking way too long. Fix all these
  issues in a sota, signoff way, so that we can come back to real coding activities!"*
  ⇒ **ORDERED SCOPE FOR THE NEXT SESSION: `.4` → `.5` → flip the default to `true`, then this tree
  is CLOSED and the frontier returns to product work.** The director has ruled on the open
  question `.3` escalated: the flip IS wanted; the condition is simply that `.4` lands first so it
  is a real green rather than false parity.
- ⛔⛔⛔ **THE BAR, REAFFIRMED AND WIDENED (director, same exchange, verbatim):** *"You need to
  resolve all these flow related issue in a clean, sota and signoff, once and for all, so that we
  can return to doing real coding activities."*
  ⇒ ⭐ **THIS IS BROADER THAN THE TWO NAMED LEAVES, and the next session must read it that way.**
  *"All these flow related issues"* + *"once and for all"* means **the `CI-PARITY-GATE-ROT` tree is
  to be CLOSED**, not advanced: that is `.2` (the gate-reachability inventory, still `todo`) as well
  as `.4`, `.5`, and the `PREPARE` flip.
  ⛔ **AND IT CONSTRAINS THE METHOD: do not discharge this leaf by opening more leaves.** Every
  session in this family has ended by routing a fresh finding onward — `.1`→`.2`, `.3`→`.4`+`.5` —
  and the director is explicitly calling time on that pattern. A new finding met during this work
  is to be FIXED inside the tree if it belongs to the flow surface. Only a defect that genuinely
  belongs to another family (an SV grammar bug, say) may be routed out, and then it must be routed
  to that family's tree with evidence, not parked as a new leaf here.
  ⚠️ **Corollary for `.2`:** its charter floats a *"candidate 11th enforced doctrine"*. Closing the
  tree does NOT require inventing a doctrine — it requires the inventory to exist and the orphans it
  names to be dispositioned. Mechanising it is a judgement call to make on the evidence, and
  `GENERATED-LINT-CORRECTNESS.4`'s lesson applies: **price a candidate against the whole corpus
  before adopting it.**
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

#### The exact work list, so `.4` needs nothing from the session that found it

**The step to add** — copy VERBATIM from `.github/workflows/generated-clippy-correctness-gate.yml`,
which already ships it (it is also what `prepare_generated_artifacts` runs in
`rust/scripts/ci_workflow_local_gate.sh`):

```yaml
      - name: Regenerate the generated parsers
        run: |
          set -euo pipefail
          make -C rust SHELL=/bin/bash regex_parser_bootstrap
          make -C rust SHELL=/bin/bash annotation_parsers
          for g in json regex systemverilog systemverilog_preprocessor vhdl rtl_const_expr rtl_frontend; do
            make -C rust SHELL=/bin/bash "focus_${g}"
          done
```

Measured cost from a bare tracked tree: **258 s** (`.3`'s prepared census).

**Which workflows need it — measured, not guessed** (`.3`'s layer-1 census; the 8 that failed on the
missing artifacts vs the 3 that never compile the crate):

| workflow file | needs the step? | why |
|---|---|---|
| `annotation-contract-gate.yml` | ✅ **yes** | rustc: missing `generated/return_annotation_parser.rs` |
| `annotation-nonbootstrap-e2e-gate.yml` | ✅ **yes** | missing `generated/return_annotation.json` |
| `differential-regression-gate.yml` | ✅ **yes** | builds `--features generated_parsers` |
| `ebnf-frontend-dual-run-diff.yml` | ✅ **yes** | same rustc error |
| `rtl-frontend-generated-contract-gate.yml` | ✅ **yes** | same, one sub-log deeper |
| `stimuli-cross-family-platform-gate.yml` | ✅ **yes** | same rustc error |
| `performance-gate.yml` | ✅ **yes** | same rustc error |
| `sota-exit-gate.yml` | ✅ **yes** | first failing sub-gate is `annotation_contract_gate` |
| `branch-protection-contract-gate.yml` | ⛔ **no** | shell + `jq` only |
| `mdbook-docs-gate.yml` | ⛔ **no** | `mdbook` only |
| `fixed-point-gate.yml` | ⛔ **no** | builds `ast_pipeline_bootstrap` WITHOUT `--features generated_parsers` |
| `generated-clippy-correctness-gate.yml` | ✅ **already has it** | added by `GENERATED-LINT-CORRECTNESS.3` |
| `memory-architecture-gate.yml` | ⛔ **no** | pure shell check; ⚠️ the ONLY workflow still on push/pull_request, so its green says nothing about the rest |
| `rtl-const-expr-cert-gate.yml` | ❓ **UNMEASURED** | not among the 11 the parity gate replays — check before deciding |
| `sv-cert-recognized-union-gate.yml` | ❓ **UNMEASURED** | not among the 11 the parity gate replays — check before deciding |

⚠️ **The last two are genuinely unmeasured** — `.3`'s census covers only the 11 workflows
`ci_workflow_local_gate` replays, and 4 tracked workflows are outside that set. Determine their need
the same way (does the command compile the crate?) rather than assuming.

⭐ **Give the recipe ONE home.** It is already duplicated between the shipped workflow and
`prepare_generated_artifacts`; adding it to 8 more files makes 10 copies. Prefer a composite action
(`.github/actions/regenerate-parsers/action.yml`) or a single `make` target that both the workflows
and the parity gate call, so the sequence has exactly one definition.


### `.5` — `sota_exit_gate` is RED: a required sub-gate that can only pass when SV generation FAILS (`done`)

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
#### ✅ EXECUTED (session #220, `PGEN-CI-PARITY-GATE-ROT-0008`) — all three assertions replaced

**Status: `done`.** The adjudication below (#219) was correct and is now implemented.

⭐⭐ **THE SHADOW ASSERTION WAS ALSO UNSATISFIABLE, AND THE PRESERVED RUN PROVED IT WITHOUT A
RE-RUN.** `.5` flagged the shadow and preprocessor assertions as UNMEASURED because the gate dies at
the first. But the run that died left its state dir intact, and the shadow surface's own report is
in it:

```
$ jq '.observed' …/systemverilog_closed_loop_parseability_shadow_report.json
{ "requested_total": 20, "attempts_total": 20, "accepted_total": 20, "rejected_total": 0,
  "parser_rejections_total": 0, "generation_errors_total": 0, "acceptance_rate_percent": 100.00 }
$ jq '.total_counterexamples, (.by_failure_context_excerpt|length)' …_shadow_counterexample_triage.json
0
0
```

⇒ **20 samples, all accepted, zero rejections, zero excerpts — assertion 2 would have failed
identically.** That is the fail-fast blindness `.1` recorded ("a fail-fast gate cannot tell you how
broken it is") answered without paying for a second run. The **preprocessor** surface was never
produced by that run and was NOT inferred — the end-to-end run below measures it.

⭐⭐⭐ **AND IT CONFIRMS THE WORST CASE: ALL THREE ASSERTIONS WERE UNSATISFIABLE, NOT ONE.** The
completed run reports every surface earning its zero:

```
    generation:    0 counterexamples, EARNED (attempts=1,   rejections=0)
    replay-shadow: 0 counterexamples, EARNED (attempts=20,  rejections=0)
    preprocessor:  0 counterexamples, EARNED (attempts=125, rejections=0)
✅ SystemVerilog failure-context contract gate passed.
guard: status=completed reason=none exit=0 peak_rss_mb=10746 elapsed_s=609
```

⇒ the preprocessor surface attempts **125** samples and the preprocessor accepts all of them, so its
assertion demanded a failure just as impossible as the first two. **Fixing only the reported one
would have revealed the second, then the third** — exactly the fail-fast trap `.5`'s charter warned
about. The class of three is closed together, and the run reaches the end.

#### What shipped

A single helper, `assert_failure_context_zero_is_earned`, applied to all three surfaces. Per surface
it requires:

1. **the surface was EXERCISED** — `attempts_total >= 1`. A zero counterexample count from zero
   attempts is the vacuous green this tree exists to remove, and the old assertion could not express
   it at all.
2. **the zero is CONSISTENT** — `total_counterexamples == 0` is acceptable **only** when the surface
   recorded no rejections and no generation errors. If anything failed and no excerpt appeared, the
   capture path IS broken, which is the real defect the old assertion was groping for.
3. **a present excerpt is WELL-FORMED** — counterexamples ⇒ at least one distinct context excerpt
   **and** a non-empty `.sample_previews[0].failure_context_excerpt`, because that value is what the
   gate publishes as its headline evidence. An empty string there is a silent evidence hole.

⭐ **THE NUMBERS ARE NOT RE-DERIVED — that would have been the same defect one level down.** The
counts come from the owning aggregate gate's own `summary.json`, and the report paths from that
gate's own `proof_surfaces` block, so this gate can never judge a different run than the triage it
just read. Hand-writing the paths here is precisely the duplicated-moving-value shape `.1` found
rotting twelve times.

⚠️ **THE THIRD SURFACE HAS A DIFFERENT SHAPE AND THAT IS STATED, NOT PAPERED OVER.** The preprocessor
parseability report carries `.summary.{attempts,accepted,rejected,parser_rejections}` and has **no**
`generation_errors` counter, so its rejection term is `parser_rejections` alone — read from the
preprocessor aggregate's own `metrics`. Defaulting an absent field to `0` and saying nothing would
have been a small instance of exactly the vacuity class.

⚠️ **A KNOCK-ON THE FIX ITSELF CREATED, AND CAUGHT BEFORE IT SHIPPED.** With a legitimate zero there
is no `sample_previews[0]`, so the gate's own summary-emission lines (`extract_json_string … | jq -er
… | strings`) would abort on the null — a green assertion followed by a crash two lines later. They
now record `<none: zero counterexamples, earned>`, which says WHY the field is empty rather than
leaving a bare `""` a reader would have to interpret.

#### ⭐ PROVED A CORRECTION, NOT A RELAXATION — and the old form is replayed, not described

`docs/tasks/artifacts/ci_parity_gate_rot/run_earned_zero_probes.sh`, **10/10**. The driver extracts
the helper **from the live gate file** and the old assertion **from `git show HEAD:`**, so it cannot
end up testing a rule the gate does not apply — `GENERATED-LINT-CORRECTNESS.4` found a probe driver
that had hand-copied its rule and was measuring a stale one.

| arm | fixture | old form | new form | what it proves |
|---|---|---|---|---|
| GREEN-1 / CTRL-1 | healthy zero, `attempts=1`, `rejections=0` | **FAIL** | **PASS** | the fix: it stops failing when the system works |
| RED-A | `attempts=0` (vacuous run) | FAIL (wrong reason) | **FAIL** — *"never exercised"* | ⭐ strictly stronger |
| RED-B / CTRL-3 | `rejections=4`, zero excerpts | FAIL (wrong reason) | **FAIL** — *"left no excerpt"* | ⭐ strictly stronger |
| RED-C | counterexamples, no excerpt | FAIL | **FAIL** | still bites |
| RED-D | excerpt present but EMPTY preview | **PASS** | **FAIL** | ⭐ strictly stronger |
| GREEN-2 / CTRL-2 | a real counterexample | PASS | **PASS** | not weakened where the old form was right |
| SWEEP | `grep -rn 'expected at least one' rust/scripts/*.sh` | — | **0 survivors** | the class is closed |

⭐⭐ **THE DECISIVE ROW IS THE PAIR RED-A / RED-B / GREEN-1.** All three fixtures have
`by_failure_context_excerpt | length == 0`. The old assertion reads only that number, so it **cannot
tell a healthy run, a vacuous run, and a broken capture path apart** — it fails all three
identically. Separating them is the whole content of the change.

#### Acceptance checklist (enforced)

- [x] **REPRODUCE / ISSUE** — `make -C rust SHELL=/bin/bash sv_failure_context_contract_gate` under
      the memory guard: `error: expected at least one generation failure-context excerpt`, guard
      `exit=2 reason=none peak_rss_mb=9870 elapsed_s=232` (main repo, `.3` measured it identically in
      the export dir).
- [x] **ROOT CAUSE (WHY + WHERE)** — WHERE: `rust/scripts/sv_failure_context_contract_gate.sh:137`
      (generation), `:141` (replay-shadow), `:171` (preprocessor) — the complete class, established
      by sweep, not assumed. WHY: each demanded `(.by_failure_context_excerpt | length) >= 1` while
      the contract driving the surface is documented *"one-profile, one-sample"* with
      `"sample_count": 1`, and the surface's own report reads
      `requested_total: 1, accepted_total: 1, attempts_total: 1, parser_rejections_total: 0` ⇒ **the
      assertion could only be satisfied if the SV parser rejected its own generated sample.**
      Provenance: `git log -S'expected at least one generation failure-context excerpt'` and
      `git log -S'"sample_count": 1'` both land on `74fc5cb6` (2026-03-15) — assertion and
      one-sample contract shipped in the SAME commit, so the coupling is original, not drift.
- [x] **ADDRESSED (verified)** — before→after replayed against the real old form, not described:
      probes **10/10**, with CTRL-1 showing the retired assertion FAILS on the same healthy-zero
      input the new form PASSES, and RED-A/RED-B/RED-D showing three failures the old form could not
      see. End-to-end, same command that failed, state dir wiped first:
      `make -C rust SHELL=/bin/bash sv_failure_context_contract_gate` goes
      `exit=2 … error: expected at least one generation failure-context excerpt` (232 s) →
      `✅ … passed`, guard `status=completed reason=none exit=0 peak_rss_mb=10746 elapsed_s=609`,
      with all three surfaces reporting `EARNED` and the preprocessor one measured for the first
      time (125 attempts, 0 rejections). ⏳ The aggregate `make -C rust SHELL=/bin/bash
      sota_exit_gate` — the acceptance this leaf is ultimately about — is running at commit time and
      its verdict is recorded in `sota_exit_gate_after.txt`; the sub-gate that made it RED is the
      one fixed and re-proved here.
- [x] **NO REGRESSION** — no `grammars/*.ebnf`, no `rust/src/*`, no `generated/*` ⇒ all 11 parsers
      **byte-identical BY CONSTRUCTION**; no release / schema / ledger / contract movement — in
      particular the one-sample contract `systemverilog_failure_context_v0_contract.json` is
      deliberately **unchanged**, because the adjudication found the budget correct and the assertion
      wrong. `bash -n` clean. GREEN-2/CTRL-2 prove the counterexample path still passes both forms.
- [x] **LOCKSTEP** — `docs/book/src/operations-and-governance.md`, `CHANGES.md`,
      `DEVELOPMENT_NOTES.md`, `MEMORY.md`, and this tree.

#### Evidence

- `docs/tasks/artifacts/ci_parity_gate_rot/run_earned_zero_probes.sh` — the 10 arms.
- `docs/tasks/artifacts/ci_parity_gate_rot/earned_zero_probes.txt` — their capture.
- `docs/tasks/artifacts/ci_parity_gate_rot/sota_exit_gate_after.txt` — the end-to-end aggregate run.

#### ✅ THE ADJUDICATION (session #219, diagnosis only — implemented above)

The two readings had opposite fixes, so `.3` refused to guess. **Measurement settles it: reading (2)
— the assertion is the bug. Reading (1) is REFUTED by the contract's own description field**,
`rust/test_data/grammar_quality/systemverilog_failure_context_v0_contract.json`, verbatim:

> *"Focused SystemVerilog parser failure-context contract for lightweight aggregate-report
> validation with **one-profile, one-sample** closed-loop replay and realistic-corpus disabled."*

with `"sample_count": 1`, `"seed_base": 12001`. ⇒ **the one-sample budget is deliberate, documented,
and named in the contract's own prose — it is not a mis-wiring.** The gate therefore asks a
deliberately-one-sample run against a parser that accepts its own generated sample to produce a
failure, which it cannot.

⭐ **And the assertion and the one-sample contract landed in the SAME commit** —
`74fc5cb6` *"Add dedicated SV failure-context contract gate"* (2026-03-15), for both
`git log -S'expected at least one generation failure-context excerpt'` and `git log -S'"sample_count": 1'`.
The gate was born with this coupling. ⚠️ Whether it ever passed is NOT yet established — the
plausible story is that SV generation used to produce a counterexample even at one sample and
stopped doing so as the parser improved, but that is a hypothesis, not a measurement, and the fix
does not depend on it.

**Scope of the class is bounded — the sweep is done.** `grep -rn 'expected at least one' rust/scripts/*.sh`
returns **exactly three** hits, all in this one script: generation (`:137`), replay-shadow (`:141`),
preprocessor (`:171`). There is no fourth instance elsewhere in the gate surface.
⚠️ Only the **generation** one is measured live; the run died there, so the shadow and preprocessor
assertions' current state is **UNMEASURED** — the fix must cover all three, and the verification
must actually reach them.

#### The designed fix — replace *"a failure must exist"* with *"the zero must be EARNED"*

⛔ Do NOT simply delete the assertions: the anti-vacuity intent behind them is real and worth
keeping. If the triage silently stopped producing excerpts, the whole failure-context surface would
be dead and nothing else would notice — the gate goes on to publish
`.sample_previews[0].failure_context_excerpt` as its headline evidence.

The replacement keeps that intent and drops the dependence on a defect. For each of the three
surfaces:

1. **The surface must have been exercised** — the generation report's `observed` must show
   `requested_total >= 1` and `attempts_total >= 1`. A run that generated nothing is exactly the
   vacuity the original assertion was groping for, and this catches it *directly*.
2. **Zero must be consistent** — `counterexamples == 0` is acceptable **only** when
   `parser_rejections_total == 0` and `generation_errors_total == 0`. If anything failed and no
   excerpt was produced, the excerpt machinery IS broken and the gate must still fail.
3. **A present excerpt must be well-formed** — when counterexamples do exist, require
   `by_failure_context_excerpt >= 1` and a non-empty `sample_previews[0].failure_context_excerpt`.

⭐ **This is a CORRECTION, not a relaxation, and the leaf must prove it that way**: the new form
fails in cases the old one could not even see (a vacuous run that generated nothing; an excerpt
present but empty), and stops failing only in the one case where the old form was wrong — a healthy
run with nothing to report. The RED arms must include a synthesised `parser_rejections_total > 0`
with zero excerpts, which must BLOCK.
- ⚠️ **Scope note.** If (1) turns out to be an SV-family question rather than a proof-surface one,
  re-home this leaf to the owning SV tree; it is filed here because *"a required sub-gate of the
  flagship aggregate has been failing undetected"* is proof-surface integrity, which is this tree.
- **Evidence:** `docs/tasks/artifacts/ci_parity_gate_rot/workflow_census_prepared.txt` (the
  replay's verdict plus the main-repo comparison and the `observed` block).

#### Reproduce in one command, and where the numbers live

```bash
scripts/run_with_memory_guard.sh --budget-mb 12288 --timeout-s 7200 -- \
  make -C rust SHELL=/bin/bash sv_failure_context_contract_gate
```

232 s, guard `exit=2`, peak 9,870 MB. The three artifacts the assertions read, all under
`rust/target/sv_failure_context_contract_gate/work/`:

- `sv_parser_aggregate_contract_gate/work/systemverilog_parseability_generation_counterexample_triage.json`
  → `.by_failure_context_excerpt | length` (the assertion at `:136`)
- `sv_parser_aggregate_contract_gate/work/systemverilog_closed_loop_parseability_shadow_counterexample_triage.json`
  → the assertion at `:141`
- `sv_preprocessor_aggregate_contract_gate/work/systemverilog_preprocessor_parseability_counterexample_triage.json`
  → the assertion at `:171` (⚠️ never reached; the run dies at `:136`)

and the report that settles whether a zero is EARNED:
`systemverilog_failure_context_quality_state/work/systemverilog_parseability_generation_report.json`
→ `.observed`.

⚠️ **Do not stop at the first green.** The run dies at the FIRST of three assertions, so fixing only
that one will simply reveal the second — the same fail-fast blindness `.1` recorded (*"a fail-fast
gate cannot tell you how broken it is"*). Fix all three, then require the run to reach the end.
⛔ And the acceptance for this leaf is **`make -C rust SHELL=/bin/bash sota_exit_gate` green
end-to-end**, not just this sub-gate — it was RED for the aggregate, and the aggregate is the claim.

