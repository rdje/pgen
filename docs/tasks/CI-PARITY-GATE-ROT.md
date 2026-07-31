# CI-PARITY-GATE-ROT: the local CI-parity gate — the one the README names as the way to prove workflow parity while hosted Actions are paused — has not been able to complete for 1,371 commits

## Metadata

- Tree ID: `CI-PARITY-GATE-ROT`
- Status: `active` (opened 2026-07-27, session #215)
- Family / slice-id prefix: `PGEN-CI-PARITY-GATE-ROT-<NNNN>`
- Created: `2026-07-27`
- Owner: repo-local workflow
- ✅ **`.7` IS DONE (2026-07-30, `-0024`) — `sota_exit_gate` went GREEN END-TO-END for the first time
  (32/32 stages, 4 h 39 m).** ✅ **`.11` SLICE 1 DONE (2026-07-30, `-0026`) — the director-greenlit ONE
  SLICE before product: the flagship aggregate's SV/VHDL stimuli telemetry now reads each stage's
  STRUCTURED `summary.txt` instead of its prose log, and the change turned out to be a **repair** — in
  reuse mode the aggregate had been publishing **22 SV telemetry values as `unknown`** while printing
  `✅ SOTA exit gate passed.`.
  **Frontier: NONE in this tree — product now** (`LANG-CAPABILITY-AUDIT.4` + the regex leg-1 debt).
  Remaining flow leaves stay ROUTED behind product: `.11` (11 of 13 METRIC sites), `.18` (new), `.10`,
  `.16`, `.17`.
  ✅ **`.14` done 2026-07-29 session #222 — and the leaf's FILED SCOPE would have been the WRONG FIX.**
  It was filed as *"the aggregate still reads that gate's `summary.json`"*; the read **is** guarded —
  the guard tests a DIFFERENT FILE (`summary.txt`) and tests EXISTENCE where it needs CONTENT, and
  the failed sub-gate leaves a **0-byte** `summary.txt`. ⛔⛔ **THE CLASS IS 6 SITES AND THE FIRST
  SWEEP REPORTED 5** — it matched the `else` by INDENTATION, and the site it missed is the sub-gate
  that runs IMMEDIATELY AFTER the one that crashed run 3 ⇒ *fixing "all 5" would have reproduced the
  identical crash one sub-gate later.* Now: 6 guards corrected, the terminal message ENUMERATES the
  failing checks with their logs (it was a bare count after a 5-hour run), and `FLOW-INTEGRITY`
  invariant **(9)** stops the class returning. ⚠️ End-to-end proof (the next aggregate run reaching
  its own verdict) is **`.7`'s** acceptance, not this leaf's — stated, not implied.
  ✅ **`.15` done 2026-07-29 session #222 — the doctrine roster went 5/13 → 13/13 on the AUTOMATIC
  lane.** The one auto-running workflow named five enforcers individually instead of invoking the
  registry driver, so 8 of 13 doctrines had no automatic lane and every doctrine registered
  afterwards inherited none. ⛔ **The leaf's own opening numbers were wrong (3 named / 10 unlaned /
  6 meaningful; measured 5 / 8 / 4) and the correction is left visible** — the finding survived, the
  arithmetic did not. ⭐ The same disease was then found one level further up, in the driver's own
  claim: `check_doctrines.sh` has always stated `DOCTRINE_ENFORCEMENT.md` §10 is *"kept in
  lockstep"* and nothing checked it — 11 mirror rows against 13 registered entries. Both fixed and
  both mechanized (`FLOW-INTEGRITY` invariant **(8)**; a driver `<meta:mirror>` check that went RED
  on the untouched tree). ⚠️ Residual stated not discovered: 4 of the 13 judge a staged diff and
  evaluate NOTHING on a hosted push — the driver now **declares** that rather than passing silently,
  and `.16` owns closing it. ⭐⭐ **ACCEPTANCE RUN 3 (2026-07-29): the best run this aggregate has ever had — 5 h 05 m, 32 gates entered, 30 ok — and still RED at blocker #8.** It cleared the ENTIRE SV and VHDL blocks and `regex_parser_family_contract_gate` (⇒ **`.9`'s fix proven in its real caller**, where run 2 died), then failed on `regex_parser_family_status_gate`: *tracker alignment mismatch: computed 'In Progress' but tracker says 'Done'*. ✅ `.9` exonerated (checked first: `resolved_targets` is reporting-only in that gate; `final_targets`=31 before and after). Regex coverage debt ROUTED to `REGEX-PCRE2-FIDELITY` with a recorded cross-family check; the aggregate's misleading failure path kept here as `.14`. See `.13`. ✅ **`.12` done
  2026-07-29 session #221 — the 13th enforced doctrine `ROUTING-EVIDENCE`**: a leaf routing a finding
  OUT of its tree must record what it MEASURED, above all whether the finding reproduces outside the
  family it is being sent to. ⚠️ Its first cut would have PASSED its own founding incident (`-0015`
  names a FAMILY, never the tree file) — caught because RED-1 replays the real commit. ✅ **`.9` done 2026-07-29
  session #221** — the seventh blocker, and the routing that sent it to the regex family was
  refuted by measurement: the stimuli gates were reading the target-DRIVE summary and discarding
  the witness pass appended after it, so `resolved_targets` was a snapshot taken before the last
  pass (`723`, truly `1002`). **The gate went RED because the pipeline got better.** Fixed in all 3
  gates that carry the same reader; the invariant moved to the producer; verified end-to-end
  (`regex_parser_family_contract_gate` ✅, 543 s). ⛔ This clears a blocker, it does **not** close
  `.7` — no aggregate run has ever reached the end. The tree was closed on `.5`'s stated
  acceptance (*"`sota_exit_gate` green end-to-end … the aggregate is the claim"*) while that
  aggregate run was still executing. It finished `exit=2`: `.5`'s own sub-gate now **passes** inside
  the aggregate, and the run then died two sub-gates later on a DIFFERENT, pre-existing defect —
  the aggregate asserting that four upstream artifacts already exist at their standalone default
  paths, which disables the branch that would produce them. ⚠️ And the one such path that DID exist
  here was **three days old**, so the aggregate consumed stale evidence as current proof. Details
  and the measured staleness table in `.7`. **Committing `.5` with the run in flight was correct;
  declaring the TREE closed on an unfinished acceptance run was not.**
- ✅ **The director's ordered scope is otherwise discharged** — every leaf is `done` and the ordered
  scope is discharged in full: **`.4`** (hosted regeneration step, one home, derived fail-safe
  audit) → **`.5`** (the earned-zero replacement; the flagship aggregate's RED sub-gate) → **the
  `PREPARE` flip** (default `true`, guarded) → **`.2`** (the reachability inventory, 31 orphans
  dispositioned behind a ratchet, shipped as the 11th enforced doctrine `GATE-REACHABILITY`).
  ⛔ **Discharged WITHOUT opening a single new leaf**, per the director's explicit constraint: every
  finding met on the way — two workflows nobody had measured, two `timeout-minutes` below their own
  measured cost, ten orphaned per-parser book gates, an unsound self-referential guard — was fixed
  **in place**.
  ⭐⭐ **`.6` then adjudicated the four items that leaf surfaced, and MEASUREMENT REFUTED ONE OF
  THEM**: the AUTOMATIC tier over these gate targets is **ZERO** — 14 of 15 workflows are
  `workflow_dispatch`-only and the one that auto-runs invokes no `make` target — so *"shrink the 28
  by wiring them in"* would have been theatre, improving a number without improving coverage. The
  real lever is resuming hosted auto-triggers on a cheap subset, which spends Actions minutes and is
  therefore ESCALATED as a director call, not quietly taken.
  ⚠️ **The one honest residual:** those 28 lanes are reached by nothing, so they run only when an
  operator names them — an ACCEPTED RISK the register states in its own text, now VISIBLE and
  ratcheted instead of invisible.
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

### `.8` — make the flow un-rottable: the invariants move to the AUTOMATIC tier (`done`)

- **Status: `done`** (2026-07-29, session #220, `PGEN-CI-PARITY-GATE-ROT-0014`), on the director's
  instruction: *"put things in place to make sure it does not drift or rot ever again."*

#### ⭐⭐ THE MEASUREMENT THAT MOTIVATED IT IS UNCOMFORTABLE, AND THAT IS WHY IT COUNTS

This campaign repaired the flow and mechanized every repair. Then a census of **where those
mechanisms live** — using `.2`'s own reachability tiers — returned:

| invariant | lives in | tier |
|---|---|---|
| regeneration coverage (`.4`) | `ci_workflow_local_gate` | **OPERATOR** |
| workflow timeout floor (`.4`/`.6`) | `ci_workflow_local_gate` | **OPERATOR** |
| one home for the recipe (`.4`) | `ci_workflow_local_gate` | **OPERATOR** |
| PREPARE stays on (`.flip`) | `ci_workflow_local_gate` | **OPERATOR** |
| gate reachability (`.2`) | `scripts/check_gate_reachability.sh` | AUTOMATIC |

⇒ **four of the five sat in the tier that only runs when a human asks.** The flow had been fixed
with checks that could themselves rot — the exact failure this tree exists to end, committed by the
tree that exists to end it. And the second measurement was worse: of the **23** gate scripts that
accept artifact hand-offs, **1** verified them.

#### What shipped — the 12th enforced doctrine `FLOW-INTEGRITY`

`scripts/check_flow_integrity.sh`, registered in `scripts/check_doctrines.sh`, run by
`.githooks/pre-commit` on **every commit**. Deliberately cheap — file reads and greps, no cargo, no
build, no network — because *a check nobody minds running is a check that keeps running*. Seven
invariants, **each traced to an incident that actually happened**:

| # | invariant | the incident it replays |
|---|---|---|
| 1 | a workflow running a `make -C rust` gate declares the regeneration step — and a measured-exempt one does NOT | 14 of 15 workflows could not build (`.4`) |
| 2 | any job carrying that step budgets ≥ 30 min | the flagship budgeted 60 min for a 143-min job (`.6`) |
| 3 | the recipe keeps ONE home; no workflow re-inlines it | it was about to be copy-pasted into ten more files (`.4`) |
| 4 | the parity gate's preparation stays on by default | the identical default eroded once, unguarded (`GENERATED-LINT-CORRECTNESS.3`) |
| 5 | no hand-off points at a gate's STANDALONE default dir | a run consumed a three-day-old artifact as current proof (`.7`) |
| 6 | no assertion requires a defect in order to pass | a required sub-gate passed only when the parser FAILED (`.5`) |
| 7 | hand-off provenance coverage only improves | 1 of 23 verify; the list may only SHRINK (`.7`) |

⛔ **Derived, not hand-listed.** The workflow roster, the hand-off consumer set and both forbidden
shapes are re-read every run. Exactly **two** inputs are written down, in
`rust/test_data/grammar_quality/flow_integrity_register_v0.json`, because they are human DECISIONS
nothing can re-derive: the measured-exempt workflows, and the not-yet-verifying consumers.

⭐ **ONE SOURCE FOR THE RULES.** `ci_workflow_local_gate`'s `workflow_is_regeneration_exempt` was a
`case` block — the same knowledge the doctrine needed, in a second place. It now READS the register.
Two lists that must agree are two lists that can disagree, and `.1` found twelve assertions rotted
on exactly that shape. **CTRL-1 proves it**: delete an exemption from the register and BOTH readers
reject, from one edit.

#### ⚠️ RED-7 CAUGHT THE NEW CHECK BEING BLIND TO THE DEFECT IT WAS WRITTEN FOR

The standalone-hand-off pattern was first written as `EXISTING_…="\$\{VAR:-\$RUST_DIR/target/…`,
which requires the `${VAR:-default}` form — and therefore **missed the bare `="$RUST_DIR/target/…"`
form, which is FOUR of the eight sites the original incident actually had.** It reported `0 found`
and read as proof. ⭐ A check written for a defect that cannot see that defect's commonest shape is
worse than none. Caught only because RED-7 injects the bare form rather than the one the author had
in mind — the session's recurring lesson, now at its fourth instance: *the arms must break the
invariant the way reality breaks it, not the way the implementer imagines it.*

#### ⚠️⚠️ HONEST LIMITS, STATED IN THE CHECK'S OWN OUTPUT AND IN THE BOOK

- Invariant 7 is a **ratchet over an accepted risk**: 22 of 23 consumers still do not verify what
  they are handed. The ratchet stops that number growing and forces it down one gate at a time; it
  does **not** mean the gap is closed.
- A pre-commit hook is **bypassable** (`--no-verify`). CI is the un-bypassable layer, and while
  hosted Actions are paused the honest claim is *"holds at every commit made through the hook"*, not
  *"no matter what"*. Re-enabling an auto CI job is what would close that — escalated in `.6`.

#### Acceptance checklist (enforced)

- [x] **ROOT CAUSE (WHY + WHERE)** — WHY the flow could rot again: the invariants protecting it lived
      in the OPERATOR tier. WHERE, measured: `grep -l` over `rust/scripts/ci_workflow_local_gate.sh`
      places 4 of 5 there, against `scripts/check_gate_reachability.sh` for the 1 in the AUTOMATIC
      tier; and `grep -lE 'EXISTING_[A-Z_]+_STATE_DIR='` vs `grep -lE 'require_supplied_state_dir'`
      over `rust/scripts/*.sh` gives hand-off provenance coverage **1 / 23**.
- [x] **ADDRESSED (verified)** — before→after: the 4 operator-tier invariants are now enforced by a
      pre-commit doctrine, and 2 further shapes (standalone hand-offs, requires-a-defect assertions)
      are held at 0 by ratchet. `bash scripts/check_flow_integrity.sh` →
      `OK (11 workflow(s) regenerate, 3 measured-exempt, recipe has one home, PREPARE on,
      0 standalone-default hand-offs, 0 requires-a-defect assertions, provenance ratchet 1/23)`.
      Probes **13/13**, every RED arm replaying a real incident; RED-7 caught the check's own
      blindness before it shipped.
- [x] **NO REGRESSION** — no `grammars/*.ebnf`, no `rust/src/*`, no `generated/*` ⇒ all 11 parsers
      **byte-identical BY CONSTRUCTION**. `bash -n` clean on both edited scripts; the parity gate's
      own probes re-run **13/13** after its exemption set moved to the register;
      `bash scripts/check_doctrines.sh` → **ALL 12 enforced doctrines PASS** (was 11);
      `mdbook_docs_gate` GREEN. CTRL-2: an unrelated Makefile edit does not trip it.
- [x] **LOCKSTEP** — `DOCTRINE_ENFORCEMENT.md` §10, `scripts/check_doctrines.sh`, a new §8 in
      `docs/book/src/gate-flow.md`, `CHANGES.md`, `DEVELOPMENT_NOTES.md`, `MEMORY.md`, this tree.

#### Evidence

- `scripts/check_flow_integrity.sh` (also `--report`), `rust/test_data/grammar_quality/flow_integrity_register_v0.json`.
- `docs/tasks/artifacts/ci_parity_gate_rot/run_flow_integrity_probes.sh` + `flow_integrity_probes.txt`.

### `.9` — the seventh blocker: a gate metric that stopped meaning its own name when a later stage was appended (`done`)

- **Status: `done`** (2026-07-29, session #221, `PGEN-CI-PARITY-GATE-ROT-0016`) — opened by taking up
  the blocker `.7` routed out, and by **REFUTING THE PREMISE ON WHICH IT WAS ROUTED**.
- ⛔ **This clears the seventh blocker, it does NOT close `.7`.** `.7`'s acceptance is the aggregate
  reaching the end, and no run ever has. The defensible claim here is *"the sub-gate that stopped
  acceptance run 2 now passes end-to-end in its own right"* — the next aggregate run decides whether
  an eighth blocker sits behind it.
- ⛔⛔ **THE ROUTING WAS WRONG, AND THAT IS THE FIRST FINDING.** `.7` filed
  `regex_parser_family_contract_gate`'s `stimuli regex target accounting mismatch (723 + 31 != 1033)`
  against `docs/tasks/REGEX-PCRE2-FIDELITY.md` as *"the regex family's stimuli target-accounting
  model, not gate wiring"*. **Measurement says otherwise**: the defect is in the SHARED closed-loop
  gate `rust/scripts/ebnf_stimuli_quality_gate.sh`, it is not regex-specific (the `ebnf` row in the
  SAME run is wrong by 5 in the same way), and it is squarely gate wiring — a gate reading the wrong
  line of a log. ⇒ **routed BACK here with evidence**, and the `REGEX-PCRE2-FIDELITY` entry records
  the refutation rather than being quietly deleted.

#### ⭐⭐⭐ THE FINDING — the assertion was RIGHT all along; the number handed to it was STALE

`.7` offered two readings and refused to guess between them. **Both were wrong.** The measured answer
is a third: `resolved_targets` is read from the WRONG LINE of the stage-2 log.

```
$ grep -E "Target-driven generation:|Witness pass:" logs/regex_stage2_target_drive.log
Target-driven generation: resolved 723/1033 targets in 5000 attempts (...)
Witness pass: resolved 723 -> 1002 of 1033 reachable targets (+279 via 249 witnesses; ...)
```

**+279 — the deficit, exactly.** `1033 - 1002 = 31 = final_targets`. The equality
`resolved + final == initial` is not a bad model of the pipeline; it is an exact statement about it,
and it reconciles **5 grammars out of 5** once the right number is used:

| grammar | drive-only (what the CSV records) | post-witness (the truth) | `final` | post-witness + final | `initial` |
|---|---|---|---|---|---|
| `regex` | 723 | **1002** | 31 | 1033 | 1033 ✅ |
| `ebnf` | 55 | **60** | 8 | 68 | 68 ✅ |
| `json` | 0 | 0 | 0 | 0 | 0 ✅ |
| `builtin_return_annotation` | 15 | 15 | 0 | 15 | 15 ✅ |
| `builtin_semantic_annotation` | 29 | 29 | 0 | 29 | 29 ✅ |

⭐ The two rows that reconcile trivially are exactly the two where the drive finished inside its
attempt budget (131 and 103 attempts) and left nothing for the witness pass. The two that were wrong
are exactly the two that **exhausted** the 5,000-attempt budget (`regex`, `ebnf`) — so the witness
pass, which exists precisely to mop up what the drive could not reach, had real work to do and its
result was discarded by the reader.

#### WHY + WHERE

- **WHERE:** `rust/scripts/ebnf_stimuli_quality_gate.sh:133` — `parse_target_summary()` greps
  `"Target-driven generation: resolved N/M targets in K attempts"`, the summary of the target-DRIVE
  pass, and returns its `N` as `resolved_targets`.
- **WHY:** since `SV-EXH-PROOF.7.4.3` the stage-2 invocation runs a **second, appended pass** after
  the drive — `generate_target_witnesses` (`rust/src/main.rs:1705`, unconditional on the
  `--target-report-input` path), documented at `stimuli_generator.rs:5374-5387` as *"PURELY ADDITIVE:
  it only generates extra samples and accumulates their coverage into `self.coverage`"*. It resolves
  more targets and reports them on its **own** line, which nothing reads. So `resolved_targets` is a
  snapshot taken at the end of the drive, while `final_targets` is the residual after the drive AND
  the witness pass AND stage 3 — two quantities from different points in time, subtracted from each
  other.
- **The order is the finding, again:**

  | commit | date | event |
  |---|---|---|
  | `789beb13` | 2026-02-21 | `parse_target_summary` written — reads the drive line. **Correct then.** |
  | `ef15fac2` | 2026-03-17 | the regex family gate ships `resolved + final == initial`. **Correct then.** |
  | `be73dabc` | 2026-06-02 | `SV-EXH-PROOF.7.4.3` appends the witness pass. **The reader is not updated. The equality becomes false that day.** |

  ⇒ **the gate went RED because the pipeline got BETTER**, and stayed RED invisibly for ~2 months
  because nothing ran it: it is reachable only from `sota_exit_gate`, which had never got past the SV
  block until this campaign cleared six blockers ahead of it.

#### ⚠️ TWO SUSPICIONS RAISED AND BOTH REFUTED BY MEASUREMENT — recorded, not quietly dropped

1. *"`final_targets` may be a RECOMPUTED set, not a subset of `initial_targets`"* (`.7`'s own leading
   hypothesis). **REFUTED**: the target sets are strictly nested — `gap3 ⊆ gap1 ⊆ gap0`, with
   **0** entries in `gap1` or `gap3` absent from `gap0` (compared by target `id`).
2. *"the reachable-branch universe collapses 837 → 26, so targets may be silently dropped from the
   gap report."* **REFUTED**: recomputing every one of the 1,033 stage-0 targets directly against each
   stage's coverage artifact reproduces the gap report **exactly** at all three stages it publishes
   (1033 / 925 / 31), with **0** targets whose branch group or index is absent. Nothing is dropped;
   `final_targets` is an honest, complete count. The collapsing number is the *still-actionable*
   reachable set, not the universe.

#### ⭐ THE CLASS IS 3 SITES, NOT 1 — and the earlier sweep looked for the symptom

`.7`'s class sweep was `grep -rn 'target accounting mismatch' rust/scripts/*.sh` → 1 site. That is
the sweep for the **assertion**. The sweep for the **defect** is
`grep -rn 'parse_target_summary' rust/scripts/*.sh` → **3 sites**, all reading the same stale line:

- `rust/scripts/ebnf_stimuli_quality_gate.sh:133` (the blocker; `regex` + `ebnf` measurably wrong)
- `rust/scripts/annotation_stimuli_quality_gate.sh:145` (measured wrong too: `semantic` drive 169/170,
  witness `169 -> 170`)
- `rust/scripts/sv_preprocessor_quality_gate.sh:201` (same `--target-report-input` path ⇒ same rot)

⇒ all three are fixed here. **Sweep for the defect, not for the message it printed.**

#### The fix

**Half 1 — the metric means what its name says.** `parse_target_summary` keeps taking `total_targets`
and `target_attempts` from the drive line (attempts *are* the drive's), and takes the resolved count
from the witness line's post-pass figure. Three cross-checks make a future appended stage LOUD
instead of silent: the witness line must be **present** (absent ⇒ refuse, naming the log — a reader
that cannot see the final stage must say so rather than report an intermediate); its
`resolved_before` must **equal** the drive line's resolved; and its total must equal the drive's.
Any re-ordering or re-shaping of the pass structure tears one of the three.

⛔ **Re-deriving resolved-ness from `coverage2` in `jq` was REJECTED**, though it would be immune to
any number of appended passes: it is a second implementation of
`evaluate_target_statuses`/`current_target_successes` (the branch-group key convention, the
`required_successes.max(1)` rule), and *a second implementation of a rule is a second thing that can
drift* — `.2`'s "ask make, do not re-implement make". The pipeline's own final self-report is the
right source.

**Half 2 — the invariant moves to the source.** The producer now asserts
`resolved_targets + final_targets <= initial_targets` for **every** grammar it runs, instead of that
coherence being checked for `regex` alone in one downstream family gate.

**Half 3 — the consumer keeps teeth, honestly.** `regex_parser_family_contract_gate.sh:360`'s strict
equality is **not** restored, because it is not sound: stage 3 generates one further sample, and a
target it resolves would make `resolved + final < initial` on a perfectly healthy run. It becomes the
sound `<=`, **plus a check the old form could not make**: `final_targets < initial_targets` when
there are targets at all — i.e. *the closed loop must actually close something*. ⭐ The old equality
**passes** on `resolved=0, final=initial` — a run in which the loop achieved literally nothing — so
on the dimension that matters the replacement is strictly stronger.

⚠️ **Stated limit:** the `<=` form alone would no longer catch a future stale reader. That job moves
to Half 1's cross-checks, which catch it **at the source, for every grammar, and at the moment it is
introduced** rather than months later in one family gate.

⚠️ **Residual, named not buried:** `parse_target_summary` remains **duplicated in 3 scripts** because
`rust/scripts/` has no shared shell library (measured: 0 of 91 scripts source one) and inventing one
for three callers is a structural change beyond this blocker. This is the duplicated-moving-value
shape `.1` found rotting 12× — recorded as `.10`, not silently accepted.

#### ✅ VERIFIED END-TO-END — the gate that reported the mismatch now passes, in 543 s

```
stimuli_regex_initial_targets: 1033
stimuli_regex_resolved_targets: 1002        (was 723)
stimuli_regex_final_targets: 31
stimuli_regex_status: pass
✅ Regex parser-family contract gate passed.
memory-guard: completed exit=0 peak_tree_rss=8971MB elapsed=543s
```

⭐⭐ **AND THE VERIFICATION RUN PRODUCED A FINDING OF ITS OWN: the retired metric was not merely
stale, it was BUDGET-DEPENDENT NOISE.** This gate drives its own budget
(`PGEN_REGEX_FAMILY_CONTRACT_STIMULI_TARGET_MAX_ATTEMPTS`, default **10,000**) — twice the 5,000 the
aggregate used. Across that doubling:

| drive budget | drive-only resolved | post-witness resolved | `final` |
|---|---|---|---|
| 5,000 (aggregate run) | 723 | **1002** | 31 |
| 10,000 (this run) | 811 | **1002** | 31 |

⇒ doubling the budget moved the published number by **88** and the true one by **zero**. The
witness pass converges to the same 1,002 either way. So the figure the gates have been publishing
was an artifact of *where the drive happened to stop*, while the correct figure is a stable property
of the grammar and the loop. ⭐ It also confirms the defect is not budget-specific: the retired
assertion fails at 10,000 too (`811 + 31 != 1033`, deficit 191) — this run would have been RED for
the same reason, 191 instead of 279.

#### Probes — 15/15, and the before→after is REPLAYED, not described

`docs/tasks/artifacts/ci_parity_gate_rot/run_target_accounting_probes.sh` extracts the retired reader
from `git show HEAD:` and the live one from the working tree, so it cannot test a rule the gates do
not apply. Both are fed the same verbatim stage-2 log:

```
    OLD reader on the regex log: 723 1033 5000
    NEW reader on the regex log: 1002 1033 5000
```

⭐ **`RED-5` is the arm that prices the consumer change.** It runs the retired strict equality on a
closed loop that resolved **nothing** (`resolved=0, final=initial=1033`) — the old form **PASSES**
it, the replacement's progress check **FAILS** it. That is proved by executing both forms, not by
claiming it. RED-1..4 cover the reader's four refusal paths (witness line absent / baseline
mismatch / total mismatch / a regressed witness pass); CTRL-1 shows a run whose drive finished
inside its budget gets the **same** answer from both readers, so the fix does not over-correct.

⚠️ **`CTRL-4` records the limit rather than hiding it**: the stale `723` PASSES the new `<=`
soundness check. The `<=` form is not what catches a stale reader — the reader's own cross-checks
are, at the source and for every grammar.

## Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — `make -C rust SHELL=/bin/bash sota_exit_gate` (acceptance run 2,
  `elapsed_s=17941`): `==> regex_parser_family_contract_gate (required) fail` /
  `error: stimuli regex target accounting mismatch (723 + 31 != 1033)`.
- [x] **ROOT CAUSE (WHY + WHERE)** — `git log -S'Target-driven generation: resolved'` →
  `789beb13` (2026-02-21, the reader) vs `git log -S'Witness pass: resolved'` → `be73dabc`
  (2026-06-02, `SV-EXH-PROOF.7.4.3`, the appended pass) ⇒ **the reader predates the pass it must
  account for**. WHERE: `rust/scripts/ebnf_stimuli_quality_gate.sh:133` `parse_target_summary()`
  greps only the drive line. The log it reads carries both, 279 apart:
  `Target-driven generation: resolved 723/1033 …` / `Witness pass: resolved 723 -> 1002 of 1033 …`.
  Two competing readings were **refuted by measurement** before the fix: the target sets are
  strictly nested (`gap3 ⊆ gap1 ⊆ gap0`, 0 foreign ids) and recomputing all 1,033 stage-0 targets
  against each coverage artifact reproduces every published gap report exactly (1033/925/31, 0
  targets with an absent branch group).
- [x] **FIX** — declarative tier (shell gate scripts only; no grammar, no engine). The reader takes
  the resolved count from the pipeline's post-witness self-report and refuses if that line is
  absent or disagrees with the drive's baseline/total; the accounting invariant moves to the
  producer for every grammar; the consumer's unsound equality becomes `<=` plus a progress check.
  ⛔ Re-deriving resolved-ness in `jq` was rejected — a second implementation of
  `evaluate_target_statuses` is a second thing that can drift.
- [x] **ADDRESSED (verified)** — before→after replayed by the probe driver: the same log yields
  `723 1033 5000` from the retired reader and `1002 1033 5000` from the live one. Against the 5
  grammars of the failing run the corrected reader reconciles **5/5**
  (`regex 1002+31=1033`, `ebnf 60+8=68`, `json 0+0=0`, `builtin_return_annotation 15+0=15`,
  `builtin_semantic_annotation 29+0=29`), where the retired reader reconciled 3/5.
  End-to-end: `make -C rust SHELL=/bin/bash regex_parser_family_contract_gate` — the gate that
  reported the mismatch — recorded in
  `docs/tasks/artifacts/ci_parity_gate_rot/regex_family_contract_after.txt`.
- [x] **NO REGRESSION** — no `grammars/*.ebnf`, no `rust/src/*`, no `generated/*` in the change set
  (`git diff --stat`), so all 11 generated parsers are byte-identical **by construction**; probes
  **15/15** including CTRL-1 (a saturated run gets the identical answer from both readers, so
  healthy rows are untouched) and CTRL-2/CTRL-3 (a healthy row still passes both consumer checks);
  `bash -n` clean on all four edited scripts; the doctrine driver and the mdBook gate are re-run
  below.
- [x] **LOCKSTEP** — `docs/book/src/gate-flow.md` §7 gains the **sixth** failure shape (*a metric
  that stopped meaning its own name*) and its rule; `docs/tasks/REGEX-PCRE2-FIDELITY.md` records the
  refuted routing premise instead of deleting it; `.10` opened for the 3-way duplication.

---

### `.13` — blocker #8: the aggregate REACHED THE REGEX FAMILY and a `Done` row failed its own gate (`done` — routed)

- **Status: `done`** (2026-07-29, session #221, `PGEN-CI-PARITY-GATE-ROT-0020`) — diagnosed, recorded
  and routed. The regex debt itself is NOT adjudicated here, deliberately.
- ⭐⭐ **ACCEPTANCE RUN 3 IS THE BEST RUN THIS AGGREGATE HAS EVER HAD, AND IT IS STILL RED.**
  `guard status=completed exit=2 peak_tree_rss=10402MB elapsed_s=18282` (**5 h 05 m**).
  **32 gates entered, 30 ok, 1 fail.** It cleared the entire SV block, the entire VHDL block, and —
  ⭐ **`regex_parser_family_contract_gate (required) ok`** — so **`.9`'s fix is proven in its real
  caller**, exactly where run 2 died. It then failed on the very next gate.

```
==> regex_parser_family_contract_gate (required)   ok      <- .9's fix, proven in the aggregate
==> regex_parser_family_status_gate  (required)    fail
error: regex tracker alignment mismatch: computed 'In Progress' but tracker says 'Done'
```

#### ✅ `.9` IS EXONERATED — checked FIRST, because "my last change broke it" is the hypothesis that must be eliminated before any other

`.9` changed `resolved_targets` (723 → 1002). **`resolved_targets` appears in
`regex_parser_family_status_gate.sh` at lines 202, 454, 519, 588 — all reporting** (a read, an
`echo`, a `jq --argjson`, a JSON field). **It is in no closure criterion.** The criterion that failed
is `regex_stimuli_final_target_debt_zero` (`:308`, `final_targets == 0`), and `final_targets` was
**31 before `.9` and 31 after** — it is the same 31 recorded in run 2's own evidence
(`723 + 31 != 1033`). ⇒ pre-existing, and newly reachable only because `.9` cleared the gate in
front of it. **Fifth instance of this tree's fail-fast pattern: fixing one blocker reveals the next.**

#### The finding — a family marked `Done` whose closure criterion no longer holds

The 31 residual targets are not noise; they cluster into two named regex features:

| cluster | targets | reasons |
|---|---|---|
| **backreferences** — `named_backreference`, `numeric_backreference`, `backreference` | 10 | mostly `selected_but_failed` |
| **subroutine calls** — `subroutine_call`, `subroutine_target`, `named_subroutine_target`, `returned_capture_subroutine` | 11 | `selected_but_failed` / `never_hit` |
| **bracket/brace tokens** — `class_bracket_token(_tail)`, `literal_open_brace` | 5 | `never_selected`, `no_reach_path` |

Reason totals: **14 `selected_but_failed`, 12 `never_selected`, 5 `never_hit`.**
⭐ `selected_but_failed` dominating means the generator *tries* these and cannot produce a witness —
the stimuli generator cannot construct valid backreference/subroutine samples.

⛔⛔ **DELIBERATELY NOT ADJUDICATED, and the two readings have opposite fixes** — `.5`'s refusal is
the template:
- **(a) genuine debt:** the regex stimuli generator cannot witness backreferences/subroutines ⇒ the
  `Done` row is stale and either the debt closes or the row moves to `In Progress`;
- **(b) scope drift:** `initial_targets` was **355** when regex earned `Done` (tracker note
  2026-03-28, `final_targets=0`) and is **1033** now — the target universe roughly **tripled**, so
  some residuals may be constructs that were never in scope for the original claim.

**Deciding evidence not gathered here: when and why `initial_targets` went 355 → 1033.** That is one
`git log -S` away and belongs with whoever fixes it, not with a guess made now.

#### ⚠️ A SECOND, SEPARABLE DEFECT — the aggregate's FAILURE PATH IS MISLEADING, and that IS flow-surface

When the sub-gate failed, the aggregate did not stop cleanly. It went on to read the summary the
failed gate never wrote:

```
error: regex tracker alignment mismatch: computed 'In Progress' but tracker says 'Done'
jq: error: Could not open file .../work/regex_parser_family_status_gate/summary.json: No such file or directory
make: *** [sota_exit_gate] Error 2
```

⇒ **the LAST error a triager sees is a missing file, not the real cause.** That is how a
tracker-alignment failure gets misdiagnosed as a broken hand-off — and this tree has already spent a
session on exactly that class of misdirection. The real cause is four lines up. ⛔ Not fixed in this
leaf (it is a distinct defect in the aggregate's error handling); **routed to `.14`.**

## ROUTING EVIDENCE

For routing the regex debt to `REGEX-PCRE2-FIDELITY`. Required by `.12`'s own doctrine, which this
leaf is the first real user of.

1. **Does the finding reproduce outside the family it is being routed to?** **No — measured.**
   `grep -c 'final_target_debt_zero\|final_targets" == "0"'` returns **0** for both
   `sv_parser_family_status_gate.sh` and `vhdl_parser_family_status_gate.sh`: the criterion does not
   exist outside regex, and both of those gates passed in this same run. The residuals are named
   regex constructs (backreferences, subroutine calls) with no analogue elsewhere.
2. **What was measured, not what is plausible?** The 31 residual targets enumerated from
   `regex_gap_stage3.json` with their reason and reach classifications; the four
   `resolved_targets` sites in the status gate, all reporting-only; `final_targets = 31` identical
   before and after `.9`.
3. **What would make the routing wrong, and was it checked?** It would be wrong if the failure were
   gate wiring rather than regex coverage. Checked: the gate computes `In Progress` from a criterion
   that is *correctly* evaluating real measured data — no hand-off, no state dir, no stale artifact
   is involved. ⚠️ The **second** defect above (the misleading failure path) **is** wiring, and is
   therefore kept HERE as `.14` rather than routed.

---

### `.15` — the one auto-running workflow invoked 5 of 13 doctrine checks individually, not the driver (`done`)

- **Status: `done`** (2026-07-29, session #222, `PGEN-CI-PARITY-GATE-ROT-0022`). Opened the previous
  session while answering whether the director needed to decide on hosted Actions minutes. **They did
  not, and this is why.**

#### ⛔⛔ THE LEAF'S OWN OPENING NUMBERS WERE WRONG, AND RE-MEASURING WAS THE FIRST THING DONE

This leaf was filed saying the workflow *"executes three checks by name"*, that *"10 of the 13
enforced doctrines have NO automatic lane"*, and that *"6 further doctrines do run meaningfully"*.
**All three figures are wrong.** Derived mechanically from
`.github/workflows/memory-architecture-gate.yml` against the driver's own `DOCTRINES=(…)` array:

| claimed when filed | measured | |
|---|---|---|
| 3 checks named | **5** | + `check_regex_self_hosting.sh`, `knowledge-map/scripts/check_knowledge_map.sh` |
| 10 doctrines with no lane | **8** | `TASK-ACCEPTANCE`, `REGEX-ORACLE-ANCHOR-SYNC`, `DESIGN-PRIOR-ART`, `WAIVER-ROUTING`, `ROUTING-EVIDENCE`, `DESTRUCTIVE-TARGET-GUARD`, `GATE-REACHABILITY`, `FLOW-INTEGRITY` |
| 6 further run meaningfully | **4** | the other 4 of the 8 are staged-scope |

⭐ The **finding survives the correction intact** — the shape was right, the arithmetic was not — but
the correction is left visible rather than back-dated, because this is the **fifth** instance in two
sessions of a plausible figure surviving on a prose reading instead of a re-run
([[feedback_read_prior_art_before_designing]]: *re-measure before citing*). The leaf was written
from a reading of the workflow, not from a derivation over it.

#### ROOT CAUSE (WHY + WHERE)

`.github/workflows/memory-architecture-gate.yml` is the **only** one of the 15 tracked workflows
triggered by anything a human does not press (derived from each `on:` block: 1 auto, 14
`workflow_dispatch`-only). At `:26,31,37,42,47` it invoked five registered enforcers **by name**
instead of the registry driver `scripts/check_doctrines.sh`.

⇒ the automatic lane was **frozen at whatever was last typed into that YAML**. `ROUTING-EVIDENCE`,
registered hours earlier in `.12`, already had no lane; so did `FLOW-INTEGRITY` and
`GATE-REACHABILITY`, the two anti-rot doctrines this very tree shipped. This is precisely the rot
`.8` was opened to end — *the flow fixed with checks that could themselves rot* — one level up.

#### ⭐ AND THE SAME DISEASE WAS FOUND ONE LEVEL FURTHER UP, IN THE DRIVER'S OWN CLAIM

`scripts/check_doctrines.sh:33` has always stated that `DOCTRINE_ENFORCEMENT.md` §10 is *"kept in
lockstep"* with the registry. **Nothing checked it, and it was not true**: 3 registered doctrines had
no row (`TASK-ACCEPTANCE`, `ROUTING-EVIDENCE`, `DESTRUCTIVE-TARGET-GUARD`) and 1 row named
`DIAG-TOOLBOX-EVIDENCE`, an id the registry no longer carries — 11 rows against 13 entries. A
documented promise nobody checks is a claim. Fixed **and** mechanized in the same commit as a
driver meta-check (`<meta:mirror>`), which went RED on the untouched tree before the rows were added.

#### THE FIX — three parts, each in ONE home

1. **The workflow invokes the driver.** Five named steps → one `bash scripts/check_doctrines.sh`.
   The registry is the single source of the roster, so a doctrine added tomorrow inherits the lane
   **by construction** — the `.4` "one home" principle applied to enforcement itself. Cost: **2.4 s**
   measured, no cargo, no build, no network ⇒ no meaningful Actions cost, and **no director call**.
2. **The vacuity is declared, by the driver, not the YAML.** The staged-scope doctrines judge a
   staged diff; a hosted push has an empty index, so they exit 0 having evaluated nothing. The driver
   now derives that set from each enforcer's own source and prints a `scope:` note naming them
   whenever the index is empty. ⭐ Putting it in the driver rather than the workflow means the same
   honesty applies to a bare local run — and a staged-scope doctrine added later is named by
   construction, the identical inheritance argument.
3. **It cannot rot back:** `FLOW-INTEGRITY` gains invariant **(8)** — no auto-triggered workflow may
   name a registered enforcer, and at least one must invoke the driver.
- ⛔ **The workflow was deliberately NOT renamed** despite now running the whole roster: the name
  `memory-architecture-gate` is referenced in ~15 tracked places including a **pinned ground-truth
  control** at `scripts/check_gate_reachability.sh:397`. Perturbing a calibration control for a
  cosmetic gain is a bad trade; the header comment states what it actually runs.
- **Relevance to `DONE-BAR.4`:** this is the free half of *"the flow shall guarantee the bar"* — the
  bar's bookkeeping enforced automatically. The expensive half (periodically re-proving parsers via
  the 5 h aggregate) stays a separate, later, director-priced decision.

#### ⚠️ WHAT THIS DOES NOT BUY — stated, not discovered later

- **4 of the 13 still evaluate nothing on a hosted push** (`TASK-ACCEPTANCE`, `DESIGN-PRIOR-ART`,
  `WAIVER-ROUTING`, `ROUTING-EVIDENCE`). **9 run meaningfully.** Making the staged-scope four read a
  PR's `base..HEAD` diff is a real capability — routed to `.16`, not smuggled in here.
- The automatic tier over the **123 `make` gate targets is still ZERO** (`.6`'s measurement stands).
  This leaf moves the *doctrine* roster, not the proof lanes; the hosted-auto-trigger question
  remains the director's.

## Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — `bash docs/tasks/artifacts/ci_parity_gate_rot/run_doctrine_lane_census.sh`
  on the untouched tree: `⇒ doctrines with an AUTOMATIC lane: 5 of 13` /
  `⇒ doctrines with NO lane: 8`. Independently, the new invariant on the untouched tree:
  `(8) .github/workflows/memory-architecture-gate.yml runs automatically and invokes registered
  doctrine enforcer(s) BY NAME:` listing all five.
- [x] **ROOT CAUSE (WHY + WHERE)** — WHERE: `.github/workflows/memory-architecture-gate.yml:26,31,37,42,47`
  (five `run: bash scripts/check_*.sh` steps) vs the registry at `scripts/check_doctrines.sh:41-55`.
  WHY: the roster is enumerated in the consumer instead of inherited from the producer, so it is
  frozen at edit time. Trigger classification derived from each workflow's `on:` block — **1 of 15**
  auto-triggered. Second defect, same shape, at `scripts/check_doctrines.sh:33`: a documented
  lockstep with `DOCTRINE_ENFORCEMENT.md` §10 that nothing checked — measured **11 mirror rows vs 13
  registered**, 3 missing + 1 stale id.
- [x] **FIX** — declarative tier (1 workflow + 2 shell checks + 1 doc mirror; no grammar, no
  `rust/src/*`, no `generated/*`). Driver invoked instead of enumerated; `scope:` note derived and
  printed by the driver; `FLOW-INTEGRITY` invariant (8); registry↔mirror meta-check.
- [x] **ADDRESSED (verified)** — before→after **replayed, not described** (the census reads the
  BEFORE workflow through `git show 509db717:` — the sha is RESOLVED and pinned in the capture, not
  left as the symbolic `HEAD`, which would silently re-point once this commit lands — and runs the
  identical derivation on both sides):
  **5/13 → 13/13** doctrines on the automatic lane; named-individually **5 → 0**; driver invocations
  **0 → 1**. Driver `exit=0`, `ALL 13 enforced doctrines PASS`, **2.4 s**. The registry↔mirror
  meta-check went **RED before → GREEN after** on the same tree. Capture:
  `docs/tasks/artifacts/ci_parity_gate_rot/doctrine_lane_census.txt`.
- [x] **NO REGRESSION** — no `grammars/*.ebnf`, no `rust/src/*`, no `generated/*` in the change set,
  so all 11 generated parsers are byte-identical **by construction**. Probes **17/17**
  (`flow_integrity_probes.txt`) — the 14 pre-existing arms unchanged plus **RED-11** (the verbatim
  incident: an auto workflow naming an enforcer), **RED-12** (the roster losing its auto lane
  altogether — a *different* defect that nothing else in the repo would notice), **RED-13** (a
  commented-out driver must read as absent), and ⭐ **CTRL-3**, the false positive that would make
  invariant (8) unusable: a `workflow_dispatch`-only workflow naming an enforcer must still PASS,
  since 14 of 15 are manual. ⭐ **RED-11 audited for WHY it blocks**, not just that it blocks — it
  fires the by-name arm **1** time and the no-lane arm **0** times, so it is not passing incidentally
  through the other half of the invariant. `scripts/check_gate_reachability.sh` still reproduces its
  8 ground-truth controls (its pinned `memory-architecture-gate.yml` → `ci-workflow-auto` control is
  untouched); workflow YAML re-parsed structurally (1 job, 4 steps, triggers
  `workflow_dispatch`/`push`/`pull_request`).
- [x] **LOCKSTEP** — `DOCTRINE_ENFORCEMENT.md` §10 (3 rows added, 1 id corrected, `FLOW-INTEGRITY`
  row now 8 invariants), `README.md` (seven → eight invariants), `docs/book/src/gate-flow.md` (the
  invariant table, the AUTOMATIC-tier note and its stale "11 enforced doctrines"),
  `docs/book/src/operations-and-governance.md`; `.16` opened for the staged-scope residual.

---

### `.17` — a schema assertion that FAILS SILENTLY: exit 1, zero bytes of output (`todo`)

- **Status: `todo`** — routed in 2026-07-30 by `DONE-BAR.5e`, WITH the reproduction, not as a
  suspicion. ⛔ Deliberately not fixed there: the honest fix is a decomposition across 4 sites, and
  that leaf's scope was one criterion.
- **MEASURED (the repro, re-runnable):** take any green family-status artifact, delete ONE pinned
  criterion from `.families[0].criteria` and decrement `closure_criteria_total_count` +
  `closure_criteria_satisfied_count` to keep every arithmetic invariant satisfiable, then run the
  matching contract gate against it. Verdict: **`exit=1`, and `wc -c` on the combined stdout+stderr
  log reads `0`.**
- **ROOT CAUSE (WHY + WHERE):** `vhdl_parser_family_status_contract_gate.sh:184`,
  `regex_…:191`, `sv_…:239` — each ends a ~60-line monolithic `jq -e '<conjunction>'` with
  `… >/dev/null` under `set -euo pipefail`, with **no `||` guard and no message**. `jq -e` exits 1 on
  a false result, its output is discarded by the redirect, and `set -e` kills the script. So the gate
  reports *that* the schema is wrong and never *which* conjunct — of roughly 20 anded clauses.
- ⭐ **THIS IS THE `.14` FAMILY, ONE LAYER FURTHER IN.** `.14` fixed a consumer that read an unwritten
  `summary.json` and buried the real cause; `.2a` made a misaligned status gate emit its summary
  BEFORE exiting. This is the same lesson unlearned in the CONTRACT sibling: **a check that fails must
  say what failed.** A 5-hour aggregate that dies here hands the triager `exit 1` and an empty file.
- **CLASS PRICED: 4 sites, not 3** — `grep -rlE "^\s+' \".*\" >/dev/null\s*$" rust/scripts/*.sh` also
  names **`regex_broader_corpus_proof_gate.sh`**. (⚠️ the first count said 3: the three contract gates
  were the ones in hand, and the sweep had not been run. Corrected forward.)
- **Scope when taken up:** keep the assertion, lose the silence — evaluate the conjuncts as NAMED
  entries (an object of `{clause: bool}` filtered to the false ones, or a `first(… | select(not))`)
  and print the failing clause names before exiting 1. ⛔ In ONE shared helper across the 4 sites, per
  `.10`'s lesson. A RED arm is already written for it: the mutation above must fail with the missing
  criterion NAMED, not merely fail.

### `.16` — the staged-scope doctrines evaluate nothing on a hosted push (`todo`)

- **Status: `todo`** — opened 2026-07-29 by `.15`, which measured the residual rather than leaving it
  implied.
- **MEASURED:** 4 of the 13 registered doctrines (`TASK-ACCEPTANCE`, `DESIGN-PRIOR-ART`,
  `WAIVER-ROUTING`, `ROUTING-EVIDENCE`) scope themselves to `git diff --cached`. On a hosted push the
  index is empty, so each exits **0 having evaluated nothing** — verified by running all four against
  an empty index. Only `check_diagnosis_evidence.sh` says so (`no code change staged`); the other
  three print nothing at all.
- ⇒ they bind at **E3 only**, which `--no-verify` bypasses — precisely the leg E4 exists to cover.
- **Scope when taken up:** give the four a PR-diff mode (`base..HEAD` when
  `GITHUB_BASE_REF`/`github.event.pull_request.base.sha` is present, staged index otherwise), in ONE
  shared helper rather than four copies (`.10`'s lesson: `parse_target_summary` rotted in 3 places at
  once). ⛔ Price it first — a push to a branch with no PR still has no base, so the honest ceiling is
  *meaningful on pull_request*, not *always*.
- ⚠️ Until then the driver **declares** the vacuity on every empty-index run, so the gap is visible
  rather than hidden behind a green tick.

### `.14` — the aggregate's failure path reported a missing file instead of the real cause (`done`)

- **Status: `done`** (2026-07-29, session #222, `PGEN-CI-PARITY-GATE-ROT-0023`). Opened by `.13`.

#### ⭐ THE ROOT CAUSE IS SHARPER THAN THE LEAF WAS FILED WITH — AND THE FILED SCOPE WOULD HAVE BEEN THE WRONG FIX

Filed as *"the aggregate still reads that gate's `summary.json`, which a gate that exited early never
wrote"*, with the scope *"stop before the summary read"*. **The read is already guarded.** The defect
is that **the guard tests a DIFFERENT FILE from the one it protects, and tests EXISTENCE where it
needs CONTENT**:

```bash
if [[ ! -f "$X_SUMMARY_TXT" ]]; then     # ← tests summary.txt, and only that it EXISTS
    …  <missing> fallback …
else
    X_GATE="$(jq -r '.gate' "$X_SUMMARY_JSON")"   # ← reads summary.json
```

The preserved state directory of the failed sub-gate is the proof — `summary.txt` present at
**0 bytes**, `summary.json` absent:

```
rust/target/sota_exit_gate/work/regex_parser_family_status_gate/
  -rw-r--r--  0  summary.txt        ← `-f` TRUE, so the guard did not fire
  (no summary.json)                 ← the else-branch then died in jq
```

⇒ *"stop before the summary read"* would have removed a read that is legitimate whenever the gate
did complete. The correct fix is the form already used as **house style ~4 lines away in the same
file** (`test -s "$…/summary.txt" && test -s "$…/summary.json"`, e.g. `:1615`, `:1646`, `:2714`,
`:2814`): guard **both** artifacts, and require them **non-empty**.

#### ⛔⛔ THE CLASS IS 6 SITES, AND MY OWN SWEEP FIRST REPORTED 5

The instrument matched the `else` by **indentation**. A nested `if … else … fi` at the same indent
inside the then-block silently re-bound it to the *inner* `else`, so the scan then searched the
wrong block. It reported **5** with full confidence. ⭐⭐ **The site it missed is
`REGEX_PARSER_FAMILY_STATUS_CONTRACT` — the sub-gate that runs IMMEDIATELY AFTER the one that
crashed acceptance run 3** ⇒ *fixing "all 5" would have reproduced the identical crash one sub-gate
later*, which is the fail-fast pattern this tree has now hit six times, this once caused by the
instrument rather than the code. **Shell blocks are not indentation-delimited; track nesting depth.**
Corrected scan: **6 before → 0 after**, and that before-count is now a pinned control arm.

#### THE FIX

1. **All 6 guards** become `[[ ! -s "$X_SUMMARY_TXT" || ! -s "$X_SUMMARY_JSON" ]]`, each carrying the
   incident in a comment so the next reader does not re-derive it.
2. **The terminal message names the cause.** The old one was a *count* — `❌ SOTA exit gate failed:
   3 required check(s) failed.` — leaving a triager to hunt the CSV after a 5-hour run. It now
   enumerates each failed required check with its log path, under the line *"read these, not any
   error printed after them"*. The informational branch gets the same treatment.
3. **It cannot come back:** `FLOW-INTEGRITY` gains invariant **(9)** — a guard must test the artifact
   it reads — scanning every `rust/scripts/*.sh` and `scripts/*.sh`. Priced first
   (`GENERATED-LINT-CORRECTNESS.4`'s rule): **6 instances**, well past the one-occurrence threshold
   `.6` used to *decline* mechanizing, and my own sweep missing one is direct evidence a human
   reviewer would too.

#### ⚠️ WHAT IS NOT PROVEN HERE

The end-to-end proof is the next `sota_exit_gate` run reaching its own verdict instead of dying in
`jq` — that is a **5-hour** run and belongs to `.7`'s acceptance, not this leaf. What IS proven is
the guard behaviour on the exact preserved state, the class sweep at 0 with a reproduced before-count,
and the enumerator run for real. ⛔ Stated rather than implied, because this tree's founding error was
calling something closed on an acceptance run that had not finished.

## Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — `docs/tasks/artifacts/ci_parity_gate_rot/sota_exit_gate_after3.txt`
  (acceptance run 3, `elapsed_s=18282`): `fail (…/regex_parser_family_status_gate.log)` followed by
  `jq: error: Could not open file …/work/regex_parser_family_status_gate/summary.json: No such file
  or directory` and `make: *** [sota_exit_gate] Error 2` — the real cause two lines above the
  terminal error.
- [x] **ROOT CAUSE (WHY + WHERE)** — WHERE: `rust/scripts/sota_exit_gate.sh` at the 6 guard sites
  (pre-fix lines `1750`, `1823`, `2731`, `2832`, `3028`, `3125`). WHY: each guards on
  `[[ ! -f "$X_SUMMARY_TXT" ]]` while its else-branch `jq`-reads `$X_SUMMARY_JSON`; the failed
  sub-gate's preserved state dir carries a **0-byte** `summary.txt` and **no** `summary.json`, so the
  guard is FALSE and the else-branch runs. Confirmed by listing the preserved directory, not inferred.
- [x] **FIX** — declarative tier (1 gate script + 1 doctrine check + doc mirrors; no grammar, no
  `rust/src/*`, no `generated/*`). Both artifacts guarded, non-emptiness required; the terminal
  message enumerates the failing checks and their logs; `FLOW-INTEGRITY` invariant (9).
- [x] **ADDRESSED (verified)** — probes **11/11**
  (`docs/tasks/artifacts/ci_parity_gate_rot/failure_path_probes.txt`), guards **extracted from the
  live gate** and the retired form from `git show`, so no arm can test a rule the gate does not
  apply: RED-1 the retired guard on the observed state reaches `jq` (the crash); GREEN-1 the live
  guard takes the `<missing>` fallback so `jq` is never reached; RED-2 the inverse half-written pair
  is also refused; SWEEP 0 survivors. Invariant (9) independently: **6 findings at `HEAD` → 0 after**.
- [x] **NO REGRESSION** — ⭐ **CTRL-1 is the arm that matters**: on a *healthy* state (both artifacts
  present and non-empty) the retired and live guards behave **identically** (`JQ-READ` both), so the
  fix cannot silently change the passing path across 6 sites in a 5-hour aggregate. CTRL-2 the
  nothing-ran state takes the fallback under both forms — the retired guard was not wrong there,
  which is exactly why the defect survived. ⭐ **CTRL-4 is ground truth for the instrument itself**:
  the corrected sweep must report **6** at the pre-fix revision, or it prints MISCALIBRATED rather
  than a number it cannot back. CTRL-3 runs the real `report_failed_checks` extracted from the gate
  over a synthetic CSV (failed *required* rows only). Flow-integrity probes **19/19** including
  RED-14 (the incident, deliberately nested to defeat an indentation-based scanner) and CTRL-5 (the
  corrected form must PASS — without it the invariant could flag every guard and make its own fix
  unlandable). ⚠️ **RED-14 caught a real blind spot in invariant (9)'s first cut**: it scanned
  `git ls-files`, so an untracked script was invisible and it reported 0 as proof — switched to a
  filesystem glob, matching invariants (5)/(6)/(7). `bash -n` clean; the check does not self-trip.
- [x] **LOCKSTEP** — `README.md`, `DOCTRINE_ENFORCEMENT.md` §10, `docs/book/src/gate-flow.md`
  (invariant table row 9), and the driver's registry description all move eight → nine invariants.
- ⚠️ **A POST-COMMIT DEFECT IN THIS LEAF'S OWN DRIVER, FOUND BY RE-RUNNING IT AND RECORDED RATHER
  THAN QUIETLY PATCHED** (`PGEN-CI-PARITY-GATE-ROT-0024`). `run_failure_path_probes.sh` defaulted its
  baseline to `HEAD` and passed **11/11** — then reported **10/11** the moment the fix was committed,
  because `HEAD` had *become* the fixed revision and CTRL-4's *"the sweep must find 6"* no longer
  held. ⭐ **A probe whose baseline moves with the branch is not a baseline** — and this is the
  identical trap already fixed in `run_doctrine_lane_census.sh` earlier the same session, not carried
  across. Both drivers now pin the resolved pre-fix sha (`de94af5e` / `509db717`) and are verified
  stable across repeated runs.

### `.12` — the 13th enforced doctrine `ROUTING-EVIDENCE`: a routing decision must record what it measured (`done`)

- **Status: `done`** (2026-07-29, session #221, `PGEN-CI-PARITY-GATE-ROT-0018`), on the director's
  question *"do you have clean ways to fix these?"* about the three items `.9` surfaced. This is
  item 3.
- **THE INCIDENT IT MECHANIZES.** `-0015` routed the target-accounting blocker to the regex family
  as *"the regex family's stimuli target-accounting MODEL, not gate wiring"*. It was neither: the
  defect was in the SHARED closed-loop gate, the `ebnf` row of the same run was wrong the same way,
  and the regex accounting was never wrong. ⭐ **The deciding evidence — cross-grammar arithmetic
  over the run's own `summary.csv` — was already on disk when the routing was written.** The
  misroute cost a session.
- **THE RULE.** A staged `docs/tasks/*.md` that adds a line routing a finding **out of this tree**
  must contain a `ROUTING EVIDENCE` section, or the commit is BLOCKED and the line is quoted back.
  The section's first question is the one the misroute never asked: *does the finding reproduce
  OUTSIDE the family you are routing it to?*
- ⭐ **An honest "not checked outside this family" is a LEGAL answer.** The check must not punish
  candour — same reasoning as `WAIVER-ROUTING`: forbidding the language would delete the signal.
  What is forbidden is routing *silently*.

#### ⚠️⚠️ THE FIRST CUT WOULD HAVE PASSED ITS OWN FOUNDING INCIDENT

The first implementation required a routing verb **and the destination TREE ID on the same line**.
Replaying commit `59f810e1` showed `-0015` never names the tree — it says *"BELONGS TO ANOTHER
FAMILY"* and *"Filed against the regex family"*. **The check was calibrated against a phrasing I
imagined rather than the one that actually occurred, and it would have shipped green over the very
commit it exists to prevent.** Caught only because RED-1 replays the real diff instead of a
paraphrase.

⇒ the predicate was re-keyed onto the **semantics of leaving the tree** (`routed out`, `belongs to
another family`, `filed against the <x> family`, `route it to that family's tree`), and then
**calibrated against the whole tracked corpus** rather than against intuition:

| measurement | result |
|---|---|
| lines flagged corpus-wide | **11**, across 4 trees |
| the three real `-0015` routing lines | **all flagged** |
| intra-tree routings in the corpus (`routed to \`.N\``) | **17** |
| of those, falsely flagged | **0** |

⚠️ **And two of my own greps disagreed while measuring this** — one case-sensitive, one not,
reporting 5 lines and 11 lines for "the same" predicate. Settled by running a single predicate
consistently. *An instrument used twice must be the same instrument.*

#### Probes — 8/8, the founding incident REPLAYED from git

`docs/tasks/artifacts/ci_parity_gate_rot/run_routing_evidence_probes.sh`. It runs entirely inside a
scratch git repo (it must stage fixtures, and must never touch the real index), and it refuses up
front if the extracted `-0015` text does not contain the routing statement — so it cannot silently
test nothing.

- **RED-1** the verbatim `-0015` added lines ⇒ BLOCK. **RED-2** a different phrasing
  (*"filed against the vhdl family"*) ⇒ BLOCK, proving the rule is not pinned to one incident's words.
- **GREEN-1** the same text plus a `ROUTING EVIDENCE` section ⇒ pass.
- **CTRL-1** intra-tree routing ⇒ pass (the measured common case). **CTRL-2** the `ROUTED-IN`
  receiving side ⇒ pass. **CTRL-3** unstaged, **CTRL-4** empty staged set, **CTRL-5** the same
  language in a non-task file ⇒ pass.

⚠️⚠️ **A FALSE-POSITIVE CLASS, FOUND BY DOGFOODING IT ON THIS VERY COMMIT.** Verifying the new
doctrine's PASS was earned rather than vacuous (remove the `ROUTING EVIDENCE` section ⇒ exit 1;
restore ⇒ exit 0) showed the **5 trigger lines in this leaf are all QUOTATIONS** of the routing
phrases, not routing decisions. ⇒ **a leaf that merely DISCUSSES routing fires the check** — the
self-referential shape already recorded as [[reference_self_referential_assertion_is_unsound]], which
this repo priced at one site and deliberately did not mechanize around. **Accepted on the same
reasoning and stated in the check's own header**: the discharge is one section, it errs toward asking
rather than staying silent, and narrowing the predicate to exclude quotation would re-introduce
precisely the phrasing-guessing that made the first cut miss `-0015`. ⭐ In this leaf's case the
section was warranted anyway — `.9` really did route a finding back out of this tree.

⚠️ **HONEST LIMIT, in the check's own header:** this verifies the reasoning was **RECORDED**, not
that the reproduction was attempted or that its conclusion was right. It would have caught `-0015`
because `-0015` recorded no cross-family check at all — not because it can tell a good check from a
bad one. That is the bound every evidence-archetype doctrine carries (`DESIGN-PRIOR-ART` states the
same one).

## ROUTING EVIDENCE

Recorded because `.9` **routed a finding back out of this tree** (the `REGEX-PCRE2-FIDELITY`
`ROUTED-IN` entry), and this leaf's own doctrine applies to it.

1. **Does the finding reproduce outside the family it was sent to?** **Yes, measured.** The `ebnf`
   row of the same run reconciles only with the post-witness figure (`55` vs `60`), and the identical
   stale reader was found in `annotation_stimuli_quality_gate.sh` (measured: `semantic` 169 vs 170)
   and `sv_preprocessor_quality_gate.sh`. A defect that fires for `ebnf`, the annotation grammars and
   the SV preprocessor is not a regex-family defect.
2. **What was measured, not what was plausible?** The five-row reconciliation table in `.9`, the two
   summary lines of the stage-2 log, and the `git log -S` timeline placing the reader before the pass
   it must account for.
3. **What would make the routing wrong, and was it checked?** It would be wrong if the accounting
   were genuinely regex-specific. Checked and refuted: the defect is in a gate that runs five
   grammars, and the regex accounting itself was correct all along.

## Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — `git show 59f810e1 -- docs/tasks/CI-PARITY-GATE-ROT.md | grep '^+'`
  yields the routing statement *"ROUTED OUT, WITH EVIDENCE, BECAUSE IT GENUINELY BELONGS TO ANOTHER
  FAMILY"* with no recorded cross-family check — a routing decision landed on reasoning alone.
- [x] **ROOT CAUSE (WHY + WHERE)** — WHERE: no doctrine covered routing decisions; the enforcer
  registry `scripts/check_doctrines.sh` listed 12 checks, none of them about routing. WHY:
  `git log -S'target accounting mismatch'` and the run's own `summary.csv` were both available at
  routing time, so the failure was not missing evidence but **unrequired** evidence. Confirmed by
  `bash -n` clean replay: the first cut of the check, run against the real commit, returned exit 0.
- [x] **FIX** — declarative tier (a repo-root `scripts/check_*.sh` + one registry row; no grammar,
  no engine, no `rust/scripts/*`). Predicate derived from the corpus, not hand-imagined.
- [x] **ADDRESSED (verified)** — before→after on the founding incident: the first cut returned
  **exit 0** (PASS) on the verbatim `-0015` diff; the shipped check returns **exit 1** naming the
  file and quoting the line. Probes **8/8**.
- [x] **NO REGRESSION** — no `grammars/*.ebnf`, no `rust/src/*`, no `generated/*`, and no
  `rust/scripts/*` (deliberately: an aggregate run was in flight) ⇒ all 11 generated parsers
  byte-identical **by construction**; corpus calibration shows **0 false positives** over the 17
  intra-tree routings; `bash scripts/check_doctrines.sh` → **ALL 13 enforced doctrines PASS**
  (was 12); `bash -n` clean.
- [x] **LOCKSTEP** — registered in `scripts/check_doctrines.sh` (so `.githooks/pre-commit` and CI
  both run it); no live doc pins a doctrine count, so none needed updating — the driver reports the
  count dynamically, which is why adding a doctrine did not create a stale claim anywhere.

---

### `.11` — the stale-log-metric class: 14 further gates scrape prose log lines for values, and nobody has checked them (`in-progress` — ⭐ SLICE 1 DONE 2026-07-30 `PGEN-CI-PARITY-GATE-ROT-0026`; 11 of 13 METRIC sites remain, ROUTED behind product)

#### ⭐⭐ SEQUENCING RULING (director 2026-07-30, session #227) — this leaf is the ONE flow exception before product

Asked whether the flow was ready to be left alone for parser work, the answer was *"yes, with exactly
one exception"* — this leaf — and the director greenlit it verbatim:
*"Ok, greenlight for 'My recommendation: .11 first, one slice, then the parser for good.'"*

⛔ **WHY THIS IS NOT A VIOLATION of the standing discipline adopted the same session**
(`docs/decisions/feedback_flow_findings_are_routed_not_worked.md`, which routes flow findings by default
and works them only when they BLOCK). That rule's own criterion is *a verdict cannot be trusted*, and
that is precisely what this leaf is:

- **13 METRIC sites derive a NUMBER from PROSE**, and **4 of them sit in parser-facing cert gates**
  (`rtl_const_expr_cert_gate`, `sv_cert_recognized_union_gate`, `verilog_2005_conformance_gate`);
- **parser and codegen work is exactly what changes log wording** — it is the input this class is
  fragile to, so the risk is not generic, it is specific to what comes next;
- and it **has already bitten**: `.9` published `resolved 723` where the pipeline had resolved **1002**,
  RED for ~2 months, invisible because only the aggregate reached it.

⇒ this is the one remaining open item that can make a **parser gate lie**, which is the worst thing to
be carrying into a parser campaign. Everything else open in this tree costs triage time, not
correctness — `.17` fails with a 0-byte log (annoying, not wrong), `.10` is duplication, `.16` is
vacuous-on-push.

⚠️ **SCOPE IS ONE SLICE, and that is part of the ruling** — not a campaign. Then product, and the
remaining flow items stay routed behind it.

- **Status: `todo`** — opened 2026-07-29 session #221 by `.9`, whose defect is an instance of a
  general shape: **a value scraped out of a human-readable log line is a coupling to a pass
  structure, and nothing checks that coupling.** `.9` proved that coupling can silently break for
  two months when an upstream improvement appends a stage.
- ⚠️⚠️ **THE FIRST CENSUS I RAN WAS WRONG, AND IT WOULD HAVE PRODUCED A CONFIDENT ALL-CLEAR.**
  Requiring `BASH_REMATCH` alongside a numeric grep returned **3 of 91** — exactly the three `.9`
  had already fixed — i.e. *"the class is closed, no further exposure."* A second angle over the
  same corpus (any `grep`/`sed`/`awk` extracting from a `*_log`-named path) returns **17 of 91**,
  and the three known sites appear in both, so the wider instrument is the calibrated one.
  ⭐ **The narrow instrument agreed with the answer I wanted and was wrong** — this tree's own
  lesson ([[feedback_instrument_needs_ground_truth]]) landing on this leaf's own measurement.
- **Measured candidate set (17):** `ast_dump_contract_gate`, `annotation_stimuli_quality_gate`,
  `bin_build_integrity_gate`, `ebnf_frontend_readiness_gate`, `ebnf_stimuli_quality_gate`,
  `ebnf_frontend_dual_run_diff_gate`, `regex_broader_corpus_proof_gate`, `rtl_const_expr_cert_gate`,
  `sota_exit_gate`, `sv_cert_recognized_union_gate`, `sv_parse_full_ratio_promotion_gate`,
  `sv_declared_shadow_promotion_gate`, `sv_combined_telemetry_contract_gate`,
  `sv_preprocessor_quality_gate`, `sv_stimuli_quality_gate`, `verilog_2005_conformance_gate`,
  `vhdl_strict_promotion_gate`.
- ⛔ **17 is a CANDIDATE set, not a defect count, and the leaf must not pretend otherwise.** Reading
  a log to check it is non-empty, or to capture a failure excerpt, is legitimate and not this shape.
  **The triage is the work**: for each site, does it derive a *number or verdict* from prose, and if
  so, is that prose emitted by a pass that something could be appended after?
#### ✅ TRIAGE DONE (2026-07-29, session #221) — read-only, so it ran while `.7`'s aggregate was in flight

⛔ **THE `17 SCRIPTS` HEADLINE ABOVE IS SUPERSEDED — it was still the wrong instrument.** Two of the
17 were **false positives** and one script was **missed**:

- `sv_stimuli_quality_gate` matched on `>"$parse_log"` — a **redirect target**, i.e. the script
  WRITING a log, not reading one.
- `sv_combined_telemetry_contract_gate` matched because **`systemveriLOG`** contains `log`.
- `ci_workflow_local_gate` was **missed** entirely.

⇒ corrected, with the three `.9` sites as ground truth (all detected) and the two false positives as
controls (both excluded): **53 read-sites across 16 scripts.** ⭐ And the SITE count is the useful
number — the file count hid that two scripts carry 11 sites each and one helper feeds ~40 values.

| class | sites | what it is |
|---|---|---|
| **FIXED** | 6 | `.9`'s three scripts, two sites each |
| **METRIC** | 13 | a NUMBER parsed out of prose and used in a gate decision or published — **the real targets** |
| **VERDICT** | 23 | `if/elif` chains classifying *why* a trial failed, by grepping an exact error sentence |
| **DIAGNOSTIC** | 2 | error text echoed for humans |
| **REVIEW** | 9 | head-excerpts, string extractions, and the aggregate's generic helper |

⭐ **HIGHEST-LEVERAGE SINGLE SITE: `sota_exit_gate.sh:798`**, the generic
`summary_value_from_log()` (`sed -nE "s/^${key}: (.*)$/\1/p" … | tail -n 1`). One helper, invoked
**~40 times**, feeding the flagship aggregate's published SV and VHDL closed-loop telemetry. Fixing
that one site is worth more than the other twelve combined.

#### ⚠️⚠️ A RISK MODEL I ALMOST SHIPPED, AND IT WAS WRONG

Mid-triage the sites sorted cleanly into `head -n 1` (6) vs `tail -n 1` (19), and the tempting
headline wrote itself: *"`head -n 1` picks the earliest line, so an appended more-final pass loses —
exactly `.9`'s defect."* ⛔ **It is not.** `git show 59f810e1:rust/scripts/ebnf_stimuli_quality_gate.sh`
shows the retired reader used **`tail -n 1`**. `.9` was never a selector bug.

**The corrected model has two independent axes:**

1. **PATTERN-SCOPE risk — `.9`'s actual shape.** The pattern names *one pass's sentence*
   (`Target-driven generation: resolved …`); a newly appended pass emits a *differently worded* line
   the pattern cannot match, so the reader keeps returning a superseded value. No selector helps —
   `tail -n 1` over a pattern that never matches the new line is still wrong. **All 53 sites carry
   this risk**, because every one is anchored to prose written by another program.
2. **SELECTOR risk — narrower.** Only bites when the *same* pattern matches several lines.

⇒ the mitigation is **not** a selector convention. It is what `.9` actually did: read the terminal
pass's own line and cross-check it, or better, read a structured artifact instead of prose.

⚠️ Recorded because it is the **fourth** time this session an appealing framing survived until it was
checked, and the third instrument defect in this leaf's own census. *A taxonomy that sorts the data
neatly is not thereby true.*

#### The deciding input for the fix, and a first measurement against it

The root-cause fix — *read the structured artifact, not the prose* — is only available where a
structured carrier exists. **First check says it often does not:** the `CERTIFICATE-COVERAGE:` family
(4 of the 13 METRIC sites, in `rtl_const_expr_cert_gate`, `sv_cert_recognized_union_gate`,
`verilog_2005_conformance_gate`) has **no JSON/structured output** alongside the prose headline —
`--report-certificate-coverage` emits the line and nothing else. ⇒ for those sites the fix is either
*add a structured output to the tool* (larger, and it changes `rust/src/`) or *apply `.9`'s pattern*
(require the terminal line, cross-check it). **Per-site, this question decides the fix**, so it is
the first thing to answer for each of the 13 — not assumed.

- **Scope when taken up:** work the 13 METRIC sites first, `sota_exit_gate.sh:798` first of those;
  for each true instance either (a) move the read to the
  structured artifact the pipeline already writes (`summary.json`/`coverage*.json`), which is the
  root-cause fix, or (b) apply `.9`'s pattern — require the terminal line and cross-check it against
  the earlier one, so an appended stage tears the check. Then decide whether a doctrine check can
  express *"a metric must not be scraped from prose when a structured artifact carries it"*, and
  price it before mechanizing (`GENERATED-LINT-CORRECTNESS.4`'s rule: do not mechanize for one
  occurrence — here there are provably more than one).
- **Instrument:** `docs/tasks/artifacts/ci_parity_gate_rot/run_log_scrape_census.sh` (reproducible,
  self-calibrating — it asserts the three `.9` sites are present and the two known false positives
  absent, and prints `MISCALIBRATED` + exits nonzero rather than reporting a smaller, comfortable
  number). Capture: `log_scrape_census.txt`.
- ⛔ **CORRECTION to the triage table above (2026-07-30):** the prose said `DIAGNOSTIC 2 / REVIEW 9`.
  The instrument and its own tracked capture both say **`DIAGNOSTIC 4 / REVIEW 7`**, and the live
  re-run is byte-identical to the capture (`diff` exit 0). The prose was the stale surface, not the
  measurement. Totals (53 sites / 16 scripts / 6 FIXED / 13 METRIC / 23 VERDICT) are unchanged.
  *When a written number disagrees with the instrument that produced it, the instrument wins.*

---

#### ✅ SLICE 1 DONE (2026-07-30, session #228, `PGEN-CI-PARITY-GATE-ROT-0026`) — and it was a REPAIR, not hardening

**Scope taken:** the leaf's own highest-leverage target — `sota_exit_gate.sh`'s generic
`summary_value_from_log()` and the SV/VHDL stimuli telemetry it feeds. **CODE:**
`rust/scripts/sota_exit_gate.sh` only (no `grammars/*.ebnf`, no `rust/src/*`, no `generated/*` ⇒ all
11 parsers byte-identical BY CONSTRUCTION).

##### ⭐⭐ THE MEASUREMENT THAT DECIDED THE FIX — the structured artifact exists for 100% of the sites

The leaf named the deciding question and refused to assume it: *does a structured carrier exist?* For
the `CERTIFICATE-COVERAGE:` family it does not. **For these sites it does, for every single key:**

| stage | keys the aggregate reads | present in the stage's own `summary.txt` |
|---|---|---|
| `sv_stimuli_quality_gate` | 24 via the helper + 2 dedicated METRIC `sed` sites | **26 / 26** |
| `vhdl_stimuli_quality_gate` | 28 via `summary_value_from_log_or_txt` | **28 / 28** |

⭐ **AND THE LOG-SCRAPING READER WAS THE MINORITY OUTLIER IN ITS OWN FILE.** Measured across
`sota_exit_gate.sh`: **281** reads already went through `summary_value_from_txt` (structured artifact),
against **52** that scraped a prose log. The correct pattern was already the house style — and the same
file already applies it one level up, preferring a stage's `*_REPORT_JSON` over the log for the three
values that have one. ⇒ *this was not a design question; it was two stages that never got converted.*

##### WHY + WHERE (the fragility, measured on run 4's real artifacts)

Every stimuli gate ends by `cat`-ing its own `summary.txt` (`sv_stimuli_quality_gate.sh:3511`), so the
stage LOG carries that block **plus every other line the gate printed**. `^key: value$` is therefore
ambiguous in the log:

```
SV   : 155 summary keys, 20 of them match MORE THAN ONE line in the stage log, 0 torn
VHDL :  74 summary keys, 20 of them match MORE THAN ONE line in the stage log, 0 torn
```

The 20 are the gate's own banner echoes (`state_dir`, `closed_loop_target_max_attempts`,
`parse_full_mode`, …). `tail -n 1` is correct **only for as long as the summary stays the last thing
printed** — and `.9` is exactly that coupling breaking: an appended pass emitted a differently-worded
line, the reader kept returning a superseded value, and the gate published `resolved 723` for a run that
had resolved `1002`, RED for two months.

##### ⛔⛔ THE LIVE DEFECT — 22 PUBLISHED SV VALUES WERE `unknown`, AND THE AGGREGATE SAID "PASSED"

The fragility above is latent. **Underneath it sat a defect that is live today**, found by running the
real aggregate rather than by reading it. In **reuse mode**
(`PGEN_SOTA_EXISTING_SV_STIMULI_QUALITY_STATE_DIR`, a supported and documented mode — `.5c` and the
README both use it) the aggregate does not re-run the stage; it calls `run_check` with a
`bash -lc "test -s .../summary.txt"` probe — **and `run_check` redirects that probe's output over
`logs/sv_stimuli_quality_gate.log`.** The haystack the reader was aimed at becomes:

```
$ wc -c .../logs/sv_stimuli_quality_gate.log
33
$ cat .../logs/sv_stimuli_quality_gate.log
Bash profile loaded successfully
```

⇒ **BEFORE: 22 of the SV closed-loop / parseability telemetry values published as `unknown`. AFTER: 0.**
Both runs exit `0` and print `✅ SOTA exit gate passed.` — the aggregate was reporting a green run whose
SV telemetry it had silently failed to read, while the reused state dir carried every value
(`closed_loop_initial_targets_total: 5461`, `closed_loop_parseability_shadow_accepted_total: 9963`,
`parseability_generation_accepted_total: 16`, …) in the `summary.txt` sitting next to it.

⭐⭐ **THE ASYMMETRY *WAS* THE BUG, and it explains why nobody saw it.** VHDL's retired reader
(`summary_value_from_log_or_txt`) had the artifact as a **fallback**, so an empty log fell through to it
and VHDL came out right **by luck**. SV had no fallback at all. Two stages of the same aggregate, the
same shape of data, one silently wrong. ⇒ *a family whose telemetry is correct by accident is not
evidence that the reader is correct.*

⚠️ **Run 4 was NOT affected** — its own audit recorded **0** stages taking a `reuse existing state`
branch, so every value it published was read from a freshly-produced log. Stated so the finding is not
over-claimed: this corrupted **reuse-mode** runs, not the green end-to-end run `.7` closed on.

##### THE FIX (fix-hierarchy tier: read the structured artifact — the root-cause tier, not a selector tweak)

1. **`summary_value_from_stage key summary_txt log_file`** replaces both log-scraping readers: the
   stage's `summary.txt` decides, the log is a fallback **only** for the case where the stage died
   before writing one. 26 SV + 28 VHDL call sites converted.
2. **`SV_STIMULI_QUALITY_STAGE_SUMMARY_TXT` derived once** — it was previously built inline inside the
   reuse branch's `test -s` string, i.e. the aggregate already treated that file as the stage's
   authoritative artifact while reading its telemetry from somewhere else. One home for the path.
3. **`assert_stage_summary_matches_log`** — a statement-level tripwire, run once per stage, that
   compares **every** key the artifact carries against the log's last matching line and FAILS naming
   each offender. Artifact-first alone would have made the drift *harmless but invisible*; this makes it
   *reported*. Derived from the artifact, not a hand-listed subset, so a key that becomes read later is
   already covered.
   ⛔ **It must be called at statement level and the code says so**: `exit` inside a command substitution
   only leaves the subshell, so this check **cannot** live inside the reader — a per-key `exit 1` in
   `summary_value_from_stage` would have been swallowed and the variable set to the empty string. That
   trap was designed around, not discovered afterwards.
4. **`summary_value_from_log_or_txt` deleted**, not merely left unreferenced: a log-first reader sitting
   in the file is an invitation to reuse it (`CTRL-5` locks that).

##### Probes — 18/18, with the defect REPLAYED from `git show HEAD:`

`docs/tasks/artifacts/ci_parity_gate_rot/run_stage_telemetry_probes.sh` → capture
`stage_telemetry_probes.txt`. Both readers are **extracted from the real scripts** (working tree for
AFTER, `git show HEAD:` for BEFORE) so the probe never tests a hand-copied lookalike, and the fixture is
a **copy** of run 4's artifacts — the real logs are evidence and a diagnostic re-run must not overwrite
them (`-0022`'s custody lesson).

| arm | proves |
|---|---|
| `CAL 1-4` | the ambiguity is real (**20** multi-match keys per stage) and today's logs are **not** torn — a detector with no positive control cannot tell a clean sweep from a blind one |
| `BEFORE-1` | the retired reader returns the **appended** value `9703` over the artifact's `5461` — the defect reproduces |
| `BEFORE-2` | the VHDL reader **had the artifact and still returned the log's** `9703` — log-first precedence was the bug |
| `AFTER-1` | the shipped reader returns the artifact's `5461`; an appended line cannot corrupt it |
| `RED-1`/`1b` | the tripwire exits nonzero, **names** the key, and **counts** offenders rather than stopping at the first |
| `GREEN-1` ×2 | the tripwire passes **silently** on both stages' real run-4 artifacts — without this arm `RED-1` would be satisfied by a check that fails on everything |
| `CTRL-1` | with no artifact the reader still returns the log value ⇒ **failure-path reads unchanged** |
| `CTRL-2` | an absent key still returns empty ⇒ the callers' `:-unknown` defaults still fire |
| `CTRL-3` | with a missing artifact the tripwire is a silent **no-op** ⇒ it cannot turn a dead stage into a misattributed torn-read failure (`.14`'s lesson) |
| `CTRL-4` | on an undoctored log the reader returns the same value ⇒ not a behaviour change on a healthy run |
| `CTRL-5`/`6` | the log-first reader is **gone**, and exactly **2** raw-log reads remain (the reader's own fallback + the tripwire's comparison) |
| `REUSE-1/2/3` | the reuse-mode defect and the SV-vs-VHDL asymmetry, encoded so they cannot silently return |
| `NOREG` ×2 | all **26** SV and **28** VHDL published values **identical** before→after on run-4 evidence |

⚠️ **MY OWN PROBE FAILED TWICE FIRST, AND BOTH WERE THE PROBE'S FAULT.** (1) `ROOT` resolved three
levels up from a **four**-level-deep artifacts dir ⇒ it refused claiming `sota_exit_gate.sh` was absent —
the *same path-depth bug as `-0023`*, and the fix was to read the failure, not to retry it.
(2) `NOREG` pinned the SV read-key count at the census's **24** and measured **26** ⇒ **the expectation
was stale, not the code**: this slice also converted the two dedicated METRIC `sed` sites. The count is
left pinned at 26 so a future edit cannot quietly drop a read site.

##### ✅ VERIFIED IN THE REAL CALLER, not only in isolation

`docs/tasks/artifacts/ci_parity_gate_rot/stage_telemetry_reuse_ab.txt`. The **actual aggregate** was run
A/B (`HEAD` script vs working tree) under identical narrowed configurations against run 4's state dirs:

- **VHDL-only:** both sides `exit=0`, `✅ SOTA exit gate passed.`, **33** published telemetry lines
  **byte-identical**, tripwire active and silent.
- **SV-only:** both sides `exit=0`, `✅ SOTA exit gate passed.`, **41** telemetry lines — **22 flip
  `unknown` → real**, 0 remain `unknown`.

##### ⛔ SCOPE HELD — 11 of the 13 METRIC sites are deliberately NOT touched

This was **one slice** by director ruling, and the remaining 11 METRIC sites stay open under this leaf:
`rtl_const_expr_cert_gate:216`, `sv_cert_recognized_union_gate:208`/`:209`,
`verilog_2005_conformance_gate:195`/`:202`/`:285`, `sv_parse_full_ratio_promotion_gate:141`,
`vhdl_strict_promotion_gate:133`, and `sota_exit_gate.sh`'s 3 SV-preprocessor `diff_mismatch_count`
readers. ⚠️ **4 of those are the `CERTIFICATE-COVERAGE:` family with NO structured output at all** —
already measured in this leaf — so they need `.9`'s pattern (b) or a `rust/src/` change, which is a
different and larger decision. **A doctrine check for *"do not scrape a metric from prose when a
structured artifact carries it"* is NOT built**: it would need to know which artifacts carry which keys,
and pricing that is worth more than guessing at it (`GENERATED-LINT-CORRECTNESS.4`'s rule).

##### ⭐ TWO SEPARABLE FINDINGS ROUTED (not fixed here) — see `.18`

1. ⛔ **`sota_exit_gate.sh` crashes on a toggle combination it advertises** —
   `SV_FAILURE_CONTEXT_CONTRACT_STAGE_STATE_DIR` assigned under `RUN_SV_PREPROCESSOR_QUALITY`, echoed
   under `RUN_SV_STIMULI_QUALITY`. **Reproduces identically on `HEAD`** ⇒ pre-existing.
2. **`summary_value_from_txt_literal` is dead** — 1 occurrence repo-wide, its own definition.

##### Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — the real aggregate, narrowed to the SV stimuli stage in reuse mode against
  run 4's state dir, exits `0` and prints `✅ SOTA exit gate passed.` while publishing **22** SV
  telemetry values as `unknown`; the reused state dir's `summary.txt` carries all 22
  (`stage_telemetry_reuse_ab.txt`).
- [x] **ROOT CAUSE (WHY + WHERE)** — `rust/scripts/sota_exit_gate.sh:792` `summary_value_from_log()` read
  `^key: value$` out of the stage's **prose log** with `tail -n 1`; in reuse mode `run_check` redirects
  its `test -s` probe over that log, leaving a **33-byte** `Bash profile loaded successfully` as the
  entire haystack, so every SV read returned empty → `unknown`. VHDL escaped only because its
  `summary_value_from_log_or_txt` had an artifact **fallback**. Diagnosed by extracting both readers
  from the real scripts (`git cat-file`/`git show HEAD:`) and replaying them side by side, with
  `bash -n ` clean on the edited gate; the ambiguity that makes the log the wrong source is measured at
  **20 of 155** (SV) and **20 of 74** (VHDL) keys matching more than one log line; the guarded
  reproduction run completed with `reason=none` and its `guard.56861.marker` written.
- [x] **FIX** — root-cause tier (read the structured artifact, do not tune the selector):
  `summary_value_from_stage` prefers each stage's own `summary.txt` (26 SV + 28 VHDL sites),
  `summary_value_from_log_or_txt`'s log-first precedence is deleted, the SV summary path is derived
  once, and `assert_stage_summary_matches_log` reports drift instead of letting a selector out-vote it.
- [x] **ADDRESSED (verified)** — re-runnable oracle
  `bash docs/tasks/artifacts/ci_parity_gate_rot/run_stage_telemetry_probes.sh` → **18 pass / 0 fail**
  (`BEFORE-1` returns the appended `9703`, `AFTER-1` the true `5461`; `REUSE-1` empty → `REUSE-2` `5461`).
  End-to-end in the real caller: SV `unknown` count **22 → 0**, both sides `exit=0`.
- [x] **NO REGRESSION** — the same probe's `NOREG` arms show all **26** SV and **28** VHDL published
  values identical before→after on run-4 evidence, and the VHDL-only real-caller A/B is
  **byte-identical** across 33 telemetry lines with the tripwire active. `bash -n` clean;
  `bash scripts/check_flow_integrity.sh --report` **OK** (11 regenerate, 3 measured-exempt, 0 guards
  testing an artifact they do not read); `bash scripts/check_doctrines.sh` → **ALL 14 doctrines PASS**;
  `check_gate_reachability` OK. No `grammars/*.ebnf`, `rust/src/*` or `generated/*` touched ⇒ all 11
  generated parsers **byte-identical** by construction, so no clippy / cert-coverage surface moves.
- [x] **LOCKSTEP** — `docs/book/src/gate-flow.md` gains the artifact-vs-prose rule as a further recorded
  flow failure; `README.md`'s `sota_exit_gate` entry unchanged (no command/flag surface changed);
  `CHANGES.md` / `DEVELOPMENT_NOTES.md` / `MEMORY.md` updated. `LIVE_ACHIEVEMENT_STATUS.md`
  **unchanged** — no family row moves.

---

### `.19` — `regenerate_generated_parsers` fails on any WARM tree, in 15 seconds (`todo`)

- **Status: `todo`** — opened 2026-07-31 session #229 by `LANG-CAPABILITY-AUDIT.10.3`, which ran
  the target for a byte-identity sweep and had to fall back to the per-family `focus_*` path.
- ⛔ **The target the README's Quick Start tells a fresh clone to run, and the single home of the
  regeneration recipe (`.4`), does not work on a tree that already has `generated/`.** Measured
  end-to-end under the memory guard: `exit=2`, `peak_tree_rss=1614MB`, `elapsed=15s`.
- **WHY + WHERE, mechanically:**
  1. `regenerate_generated_parsers` (`rust/Makefile:906`) → `regex_parser_bootstrap` (`:818`).
  2. `generated/ebnf.rs` exists, so the else-branch at `:837` runs a **raw**
     `cargo build --features ebnf_dual_run --bin ast_pipeline` to check the artifact still
     compiles. That writes **`rust/target/debug/ast_pipeline`** — the same path as
     `$(RUST_AST_PIPELINE)` (`:159`), which is defined to be the **`--features
     generated_parsers`** binary — and refreshes its mtime.
  3. `$(MAKE) regex_parser` (`:851`) then finds `$(RUST_AST_PIPELINE)` newer than
     `$(AST_PIPELINE_SOURCES)`, so make declares it up to date and never rebuilds it.
  4. The generation step runs the wrong-feature binary and REFUSES, correctly and loudly:
     `REFUSED: semantic annotation '@whitespace_sensitive: true' needs the generated annotation
     backend, but this binary was built WITHOUT --features generated_parsers`. The refusal is the
     `RGX-0078.5.i.1.t1` guard doing its job — the defect is upstream of it.
- **Why a COLD clone escapes it:** with `generated/` empty, `$(RUST_AST_PIPELINE)`'s prerequisites
  (`$(SEMANTIC_ANNOTATION_PARSER)`, `$(RETURN_ANNOTATION_PARSER)`) are missing, so make *does*
  rebuild the binary with the right features. ⇒ the flow is only ever exercised in the
  configuration that hides the bug, which is why it has stood.
- ⭐ **This is the `#140`-class trap TOOLBOX.md already names — two feature sets, one output
  path — promoted from a hazard to a broken target.** `parse_harness` defends against it with a
  pre-flight `--report-feature-surface` probe; the Makefile has no equivalent.
- ⚠️ **It also leaves damage:** the run got as far as rewriting `generated/regex.json` before
  refusing, so a failed invocation mutates `generated/` and exits.
- ⚠️ **ROUTED, not worked** ([[feedback_flow_findings_are_routed_not_worked]]): it does not make
  any verdict untrustworthy — the per-family `focus_*` targets are canonical and were used
  instead, with all 11 generated parsers proven byte-identical. It costs the one-command
  regeneration path, which is a real cost for CI and for downstream consumers like RGX.
- **Scope when taken up:** give `$(RUST_AST_PIPELINE)` a feature-surface pre-flight (the same
  `--report-feature-surface` probe, rebuilding when it does not match) rather than trusting mtime;
  or stop the two feature sets sharing an output path (the dedicated
  `$(RUST_EBNF_FRONTEND_BIN)` at `:163` already models the separate-`--target-dir` answer). Then
  add a warm-tree invocation to the automatic tier so the configuration that hides the bug is no
  longer the only one tested.

---

### `.18` — two separable `sota_exit_gate.sh` defects found while working `.11` (`todo`)

- **Status: `todo`** — opened 2026-07-30 session #228 by `.11`, which measured both while verifying in
  the real caller and deliberately did not widen its slice to them.
- **(a) A supported toggle combination crashes the aggregate.**
  `SV_FAILURE_CONTEXT_CONTRACT_STAGE_STATE_DIR` is assigned inside
  `if [[ "$RUN_SV_PREPROCESSOR_QUALITY" -eq 1 ]]` but echoed inside the `RUN_SV_STIMULI_QUALITY`
  summary block, so `PGEN_SOTA_RUN_SV_PREPROCESSOR_QUALITY=0` with
  `PGEN_SOTA_RUN_SV_STIMULI_QUALITY=1` dies under `set -u`:
  `line 2253: SV_FAILURE_CONTEXT_CONTRACT_STAGE_STATE_DIR: unbound variable`.
  **Reproduces identically on `HEAD` at line 2191** ⇒ pre-existing, not introduced by `.11`.
  ⚠️ **Why this is ROUTED, not worked** (per
  `docs/decisions/feedback_flow_findings_are_routed_not_worked.md`): it does not make a verdict
  untrustworthy and it does not stop the aggregate's real configuration — it makes a *narrowed*
  configuration unusable, which costs triage convenience. ⭐ It is worth fixing because narrowing is
  exactly how a 4 h 39 m aggregate gets debugged cheaply, and `.11` had to work around it to verify.
  **Scope when taken up:** sweep every `*_STAGE_STATE_DIR` / `*_SUMMARY_*` variable for the same
  set-under-one-toggle / read-under-another split (expect more than one), and decide between
  initialising them unconditionally and guarding the echo with the toggle that owns them.
- **(b) `summary_value_from_txt_literal` is dead code.** 1 occurrence repo-wide — its own definition —
  and 0 call sites. It exists because a value containing regex metacharacters cannot be read by the
  `sed`-based readers, so deleting it may be discarding a real answer to a real problem; check whether
  any current read needs it before removing it (`DESIGN-PRIOR-ART` in the direction where it stops you
  *deleting*, not reinventing).

---

### `.10` — `parse_target_summary` is copy-pasted into 3 gates, and it just rotted in all 3 at once (`todo`)

- **Status: `todo`** — opened 2026-07-29 session #221 by `.9`, which had to apply the identical fix
  three times because the identical function exists three times.
- **Measured:** `grep -rn 'parse_target_summary' rust/scripts/*.sh` → 3 definitions, previously
  **byte-identical**, in `ebnf_stimuli_quality_gate.sh`, `annotation_stimuli_quality_gate.sh` and
  `sv_preprocessor_quality_gate.sh`. One upstream change (`SV-EXH-PROOF.7.4.3`, 2026-06-02) made all
  three wrong simultaneously; `.9` had to repair all three by hand and any future change must too.
- ⭐ This is the **duplicated-moving-value** shape `.1` found rotting 12× and `.4` answered with
  *"one home for the recipe"* — the same disease at shell-function scale.
- ⛔ **Deliberately NOT fixed inside `.9`.** `rust/scripts/` has **no shared shell library**
  (measured: 0 of 91 scripts source one), so hoisting this function means introducing that
  convention, which is a structural change to 91 scripts' house style and must be priced and
  decided on its own, not smuggled in behind a blocker fix.
- ⭐ **CROSS-LINK (2026-07-31, `LANG-CAPABILITY-AUDIT.10.11`): this decision now BLOCKS a small, otherwise-trivial fix, which is useful pricing information.** Five gate scripts execute the Perl interpreter without declaring it (`branch_protection_contract_gate.sh`, `fixed_point_bootstrap_gate.sh`, `sv_declared_shadow_promotion_gate.sh`, `sv_parse_full_ratio_promotion_gate.sh`, `vhdl_strict_promotion_gate.sh`) and have **no** `require_tool()` at all — so declaring it means introducing the function, and `git grep -c '^require_tool() {'` already finds **60 identical 6-line copies** across `rust/scripts/`. `.10.11` fixed the two scripts that already had the mechanism and deliberately refused to paste a 61st copy into the other five. ⇒ **the shared-helper question is no longer only about `parse_target_summary`** — it is now the gate on a second, unrelated repair. Whichever way it is decided, those five come with it.
- **Scope when taken up:** price a `rust/scripts/lib/` (or equivalent) shared-helper convention
  against the 91-script surface; if adopted, hoist this function first and add a doctrine check that
  a duplicated helper body cannot re-appear. If rejected, record why, and add a cheap check that the
  three bodies stay identical so they cannot silently diverge instead.

---

### `.7` — ✅ `sota_exit_gate` IS GREEN END-TO-END, FOR THE FIRST TIME (`done`, 2026-07-30 session #227, `PGEN-CI-PARITY-GATE-ROT-0024`)

#### ✅ ACCEPTANCE MET — the criterion this leaf refused to weaken is satisfied

`.7`'s stated acceptance was *"`make -C rust SHELL=/bin/bash sota_exit_gate` end-to-end … the
aggregate is the claim"*, and the leaf explicitly forbade closing on sub-gate evidence. **Acceptance
run 4 reached the end:**

```
✅ SOTA exit gate passed.
✅ SOTA exit aggregate gate passed.
memory-guard: completed exit=0 peak_tree_rss=10422MB elapsed=16759s
```

`guard status=completed reason=none exit=0 peak_rss_mb=10422 elapsed_s=16759` (**4 h 39 m**), launched
2026-07-30 02:42 under `--budget-mb 16384 --timeout-s 25200`. Evidence
`docs/tasks/artifacts/ci_parity_gate_rot/sota_exit_gate_run4_green.txt`.

⭐⭐ **AND THE GREEN IS EARNED, NOT PERMITTED — checked before it was reported, because
`allow_informational_failures: 1` is set and a passing aggregate could otherwise be hiding a failure:**

| check | result |
|---|---|
| stages entered / ok / fail | **32 / 32 / 0** (the 33rd `==>` is the run's own header line) |
| required vs informational | **31 required + 1 informational**, and the informational one (`sv_parse_full_ratio_promotion_gate`) also **passed** ⇒ the allowance permitted nothing |
| stages skipped | **0** |
| stages taking a `reuse existing … state` branch | **0** ⇒ ⭐ every stage PRODUCED its own evidence — `.7`'s own fix proven in its real caller, not just in isolation |
| run 3 comparison | 32 entered / **30 ok / 1 fail**, dying at `regex_parser_family_status_gate` |

⇒ the honest claim is no longer *"clears 30+ sub-gates and fails in the regex family"* but **"32 of 32,
including the entire SV, VHDL and regex blocks, with nothing reused and nothing skipped."**

⛔⛔ **WHY IT WENT GREEN IS NOT "the 8th blocker was fixed" — THERE WAS NO 8th BLOCKER.** Run 3 died on
`regex_parser_family_status_gate`'s *tracker alignment mismatch: computed 'In Progress' but tracker says
'Done'*. **`DONE-BAR.2b` moved that tracker row to `In Progress`** (session #224) — i.e. the blocker was
cleared by *making the tracker tell the truth*, not by changing a gate. ⭐ **The gate was right and the
tracker was stale, exactly as `.13` adjudicated**, and this run is the proof that the adjudication was
correct. Run 4 now records `regex_family_status_regex_tracker_alignment_ok: true` with the family
computing `In Progress` and **3** unmet closure criteria — a family honestly below the bar, which the
aggregate accepts because the tracker says the same thing.

⚠️ **HONEST BOUNDS, stated rather than implied:**

- **This is ONE machine and ONE run.** The aggregate is reachable only when a human asks — `.6`'s
  measurement stands: the AUTOMATIC tier over the 123 `make` gate targets is still **ZERO**. So the
  defensible claim is *"green on this machine at this commit"*, not *"green no matter what"*.
- **It does NOT close this tree.** `.10`, `.11`, `.16` and `.17` remain `todo`.
- **It does not promote any tracker row.** Three of the four families with a status gate compute a tier
  below `Done`, and the aggregate passing is consistent with that by design.
- ⭐ Sequencing that paid off: run 4 was deliberately launched **after** `DONE-BAR.5f`'s trace-default
  fix, so it is also the in-situ proof of that change — the aggregate's scratch tree finished at
  **2.8 GB** where run 3 left **198 GB**, and the 12 replay shadow logs measure **1.1–10.1 kB** each
  against the ~20 GB apiece they carried before.

#### ⛔ A THIRD REGEX CRITERION SURFACED THAT NO RECORD MENTIONS — routed, not absorbed

With alignment passing, the regex family-status gate ran to completion for the first time and published
its **full** criteria set: `closure_criteria_satisfied 8 of 11`, **3** unmet —

```
["stimuli_regex_parseability_parser_rejections_total=40 > 0",
 "stimuli_regex_final_targets=31 > 0",
 "external_corpus_conformance_pass=false (leg3_surface=<none>)"]
```

⭐ The **primary** unmet criterion is `stimuli_parseability_parser_rejections_zero` at **40** — and the
tracked record (`DONE-BAR`, `REGEX-PCRE2-FIDELITY`, `MEMORY.md`) documents regex's leg-1 debt as
`final_targets=31` **only**. A repo-wide grep for `parseability_parser_rejections_total` finds it
discussed exclusively for the **preprocessor** (`SV-EXH-PROOF`, at 3), never for regex. ⇒ **regex's
leg-1 debt is larger than anything written down, and this is the first run that could see it** — every
earlier run died before the criteria were computed. Routed to `REGEX-PCRE2-FIDELITY` as `ROUTED-IN-4`.

---

#### (historical) ⛔ THE TREE'S CLOSURE WAS PREMATURE: `sota_exit_gate` is STILL RED, one sub-gate further on

#### ✅ THE FIX IS IMPLEMENTED AND PROBED (2026-07-29, `PGEN-CI-PARITY-GATE-ROT-0013`); ⏳ the end-to-end re-run is the outstanding acceptance

**The class is bounded and was swept before fixing** — `grep -rnE 'EXISTING_[A-Z_]*STATE_DIR="…target/'`
over every `rust/scripts/*.sh` returns **8 sites in ONE file**: 4 distinct variables × the required
and informational branches of `sota_exit_gate.sh`. ⭐ **And the correct form is already used ~30
times in that same file** for the VHDL / regex / EBNF families (`"${PGEN_SOTA_EXISTING_…:-}"`), so
the SV family-status block was the sole outlier, not a house style.

**Half 1 — stop asserting existence the aggregate never established.** Measured first: the aggregate
runs **zero** of the four as its own stage (`grep -c 'run_check "<gate>"'` → 0 for all four), so it
had no in-run artifact to point at — which is *why* it pointed at standalone dirs. The four now pass
the empty value, i.e. *"not supplied"*, and `sv_parser_family_status_gate` produces them through the
`else` branch it already has and that the aggregate was suppressing. Three `PGEN_SOTA_EXISTING_*`
pass-throughs were added so an operator can still supply them deliberately. **8 → 0** bad hand-offs.

**Half 2 — a hand-off must prove its provenance, or refuse.** New
`require_supplied_state_dir` in `sv_parser_family_status_gate.sh`, applied to **all 11** of its
hand-offs in one place before any stage runs (so a hand-off added later cannot forget it):

1. absent directory or empty `summary.txt` ⇒ **refuse up front, naming the caller's variable**,
   instead of dying twenty minutes later on a downstream assertion;
2. when the caller declares `PGEN_GATE_ARTIFACT_MIN_EPOCH`, the artifact must be at least that new ⇒
   **in-run reuse passes, a pre-run leftover is refused.** `sota_exit_gate.sh` exports that epoch
   once at run start — exported, not passed per call, so new hand-offs inherit it.

⛔ The epoch is deliberately opt-in on the caller's side: a standalone operator run has no such
reference point and inventing one would refuse legitimate reuse. What is *not* optional is that the
aggregate declares it.

⚠️⚠️ **THE PROBES CAUGHT A REAL PORTABILITY DEFECT IN THE GUARD'S FIRST CUT — and it is the same
"measuring the wrong thing" class this session keeps producing.** Reading an mtime is not portable:
BSD `stat` spells it `-f %m`; GNU coreutils spells it `-c %Y` and reads `-f` as *file system
information*, which **succeeds** at printing six lines of block counts. This host has GNU `stat`, so
the BSD-first chain never fell through — it captured that block as the "timestamp" and compared it
numerically (`RED-2` and `GREEN-1` both died on `File: unbound variable`). ⛔ Reordering the chain
would have been another guess; the fix tries both and **validates the result is a bare integer**,
refusing when neither yields one. *A provenance check that cannot establish provenance must say so.*

**Probes 5/5** (`run_artifact_provenance_probes.sh`, helper extracted from the LIVE gate so it
cannot test a rule the gate does not apply): RED-1 absent ⇒ refuse; ⭐ **RED-2 an artifact stamped
`2026-07-26 00:36` — the real mtime of the leftover the aggregate consumed — ⇒ refuse**; GREEN-1 an
in-run artifact ⇒ accept; CTRL-1 no epoch declared ⇒ today's behaviour preserved; RED-3 empty
summary ⇒ refuse.

#### ✅ THE FIX IS PROVEN, AND IT PRICED ITSELF — but the aggregate is STILL RED, six blockers in

Acceptance run 2, same command, state dir wiped first:
`guard status=completed reason=none exit=2 peak_rss_mb=11208 elapsed_s=17941` (**4 h 59 m**).

**What the fix proved.** Run 1 died at `sv_parser_family_status_gate` after 4,249 s. Run 2 clears it,
`sv_parser_family_status_contract_gate`, and the **entire VHDL block**, reaching the regex family.
⇒ **the `.7` fix works.** And it prices itself: **+13,692 s**, because four sub-gates the aggregate
previously SUPPRESSED now actually run, plus everything downstream that had never executed. The
aggregate's real cost was always this; the old 4,249 s was the cost of dying early.

#### ⛔ A SIXTH PRE-EXISTING BLOCKER, NEWLY REACHABLE — AND IT IS NOT FLOW

```
==> regex_parser_family_contract_gate (required)   fail
error: stimuli regex target accounting mismatch (723 + 31 != 1033)
```

`rust/scripts/regex_parser_family_contract_gate.sh:360` asserts
`resolved_targets + final_targets == initial_targets`. Measured this run:
`initial_targets=1033 resolved_targets=723 final_targets=31` — deficit **279**.

⭐ **The PRODUCER says pass; the CONSUMER's model says mismatch.** `ebnf_stimuli_quality_gate`
records `status=pass` for that row. The failure is the consumer assuming the target set is CLOSED —
that every initial target ends either resolved or still-open. The pipeline runs stages (stage0 /
stage3, 5,000 `target_attempts`, 3,549 `stage3_successes`) with a recompute step, so `final_targets`
may be a RECOMPUTED set rather than a subset of `initial_targets`, in which case the equality is not
an invariant of the pipeline at all. ⚠️ **NOT ADJUDICATED — the two readings have opposite fixes and
this leaf refuses to guess**, exactly as `.5`'s adjudication refused before it.

**Provenance:** `git log -S'target accounting mismatch'` → `ef15fac2` (2026-03-17) *"Add regex
parser-family contract gate"* — assertion and gate shipped together, an original coupling like
`.5`'s, not drift. **Class sweep:** exactly **1** site repo-wide; the SV and VHDL family gates make
no such assertion, which is itself evidence worth weighing.

⛔⛔ **ROUTED OUT, WITH EVIDENCE, BECAUSE IT GENUINELY BELONGS TO ANOTHER FAMILY.** This is the regex
family's stimuli target-accounting model, not gate wiring: no state dir, no hand-off, no workflow, no
`generated/` dependency is involved. The director's constraint is *fix flow-surface findings in
place, route out only what belongs to another family, and then route it to that family's tree with
evidence* — this is that case. Filed against the regex family with the full capture in
`docs/tasks/artifacts/ci_parity_gate_rot/sota_exit_gate_after2.txt`.

#### ⚠️⚠️ THE HONEST STATEMENT ABOUT THE AGGREGATE

`sota_exit_gate` has now revealed **six** blockers, one per fix, each hidden behind the last:
`.3`'s missing artifacts → `.5`'s three unsatisfiable assertions → `.7`'s four bad hand-offs →
this. **Nobody knows how many remain**, because no run has ever reached the end. The only honest
claim is *"it now clears 30+ required sub-gates including the entire SV and VHDL blocks, and fails
in the regex family"* — not *"one more fix and it is green."* ⇒ this leaf's original acceptance
(*"green end-to-end"*) is **not** met and is not met by anything landed here.

✅ **DISCHARGED 2026-07-30 by acceptance run 4** (see the top of this leaf): end-to-end
`exit=0`, 32 of 32 stages, nothing reused, nothing skipped. ⭐ The cost that was *"not yet priced"* is
now measured: **16,759 s (4 h 39 m)**, against run 2's 17,941 s and run 3's 18,282 s — so the aggregate
got **cheaper** while running strictly more to completion, which `DONE-BAR.5f`'s replay-trace fix
(landed hours before this run) is the leading explanation for and which the next run can isolate.

---

- **Status: `done`** (2026-07-30 session #227, `PGEN-CI-PARITY-GATE-ROT-0024`) — opened 2026-07-28
  session #220 by the aggregate run that `.5` had declared its own acceptance and that was still
  executing when the tree was closed. **Closed on the criterion it was opened to defend: the aggregate
  itself, green end-to-end, with the green audited for whether it was earned.** ⛔ The TREE stays
  `active` — `.10`, `.11`, `.16`, `.17` are still `todo`.
- ⛔⛔ **THE HONEST CORRECTION, STATED FIRST.** `.5`'s acceptance was written as *"`make -C rust
  SHELL=/bin/bash sota_exit_gate` green end-to-end, not just this sub-gate — it was RED for the
  aggregate, and the aggregate is the claim."* **That criterion is NOT met**, and the tree was
  closed before its verdict landed. The closure is therefore **withdrawn until this leaf lands**.
  Committing `.5` with the run in flight was correct (the sub-gate fix is independently proven);
  declaring the TREE closed on an unfinished acceptance run was not.

#### What the run did prove — `.5`'s fix works inside the aggregate

```
==> sv_failure_context_contract_gate (required)
    ok
```

19+ required sub-gates cleared, **including the one that made the aggregate RED before `.5`**. The
earned-zero replacement holds end-to-end in its real caller. Then:

```
==> sv_parser_family_status_gate (required)          fail
==> sv_parser_family_status_contract_gate (required) fail
guard: status=completed reason=none exit=2 peak_rss_mb=10257 elapsed_s=4249
```

⭐ **This is the fail-fast pattern this tree has now recorded four times** (`.1`: twelve stale
assertions revealed one per run; `.3`: one broken replay per run; `.5`: three unsatisfiable
assertions, one reachable per run). **Fixing the first blocker reveals the next.** It is a
DIFFERENT, pre-existing defect — not a regression from `.5`.

#### ⭐⭐ ROOT CAUSE — and the visible failure is the *less* dangerous half

`rust/scripts/sota_exit_gate.sh:1682-1690` (and the identical informational branch at `1696-1704`)
tells `sv_parser_family_status_gate` that four upstream artifacts **already exist**, by pointing each
`PGEN_SV_FAMILY_STATUS_EXISTING_*_STATE_DIR` at that gate's **standalone default** state dir:

```
PGEN_SV_FAMILY_STATUS_EXISTING_SV_SYNTAX_CLOSURE_STATE_DIR="$RUST_DIR/target/sv_syntax_closure_gate"
PGEN_SV_FAMILY_STATUS_EXISTING_SV_PREPROCESSOR_SYNTAX_CLOSURE_STATE_DIR="$RUST_DIR/target/sv_preprocessor_syntax_closure_gate"
PGEN_SV_FAMILY_STATUS_EXISTING_SV_PREPROCESSOR_FORMAL_EXHAUSTIVE_CLOSURE_STATE_DIR="${…:-$RUST_DIR/target/sv_preprocessor_formal_exhaustive_closure_gate}"
PGEN_SV_FAMILY_STATUS_EXISTING_SV_SEMANTIC_SCOPE_CONTRACT_STATE_DIR="$RUST_DIR/target/sv_semantic_scope_contract_gate"
```

Those directories exist **only if someone previously ran those gates by hand**. `sv_parser_family_
status_gate.sh:141-210` branches on exactly these variables: a NON-EMPTY value makes it **skip its
own `else` branch — the one that would PRODUCE the artifact** — and then assert on a file nothing
created. ⇒ **a fallback that turns "not supplied" into "supplied but nonexistent", disabling the
machinery that would have produced the real thing.**

**Measured on this machine, and the second row is the finding:**

| standalone state dir | present? | consequence |
|---|---|---|
| `sv_syntax_closure_gate` | **EXISTS**, `summary.txt` dated **2026-07-26 00:36** | ⚠️⚠️ **today's "fresh" aggregate run consumed a THREE-DAY-OLD manual artifact as current proof** |
| `sv_preprocessor_syntax_closure_gate` | ABSENT | the run dies |
| `sv_semantic_scope_contract_gate` | ABSENT | the run dies |
| `sv_preprocessor_formal_exhaustive_closure_gate` | ABSENT | the run dies |

⭐⭐⭐ **THE ABSENT ONES FAIL LOUDLY; THE PRESENT ONE PASSES SILENTLY ON STALE EVIDENCE.** A machine
that had run all four by hand at some point would get a **fully green `sota_exit_gate` built on
artifacts of unknown vintage** — the release gate certifying today's tree with last week's proof.
That is a **fifth shape** for this family, and the worst of them so far: *a check that reuses
evidence it did not produce, without checking whether it still applies.*

⚠️ **A hosted runner has none of the four**, so on CI this is a hard failure, not a silent pass —
which is the same asymmetry `.3` found (the local developer tree hides what a fresh checkout
exposes), pointing the opposite way this time.

#### Scope when taken up

1. Stop asserting existence the aggregate has not established: pass EMPTY for the four, so
   `sv_parser_family_status_gate` runs each sub-gate into its own `$WORK_DIR` — the branch it
   already has and that the aggregate is currently disabling. Price the added runtime first; it is
   four more sub-gate runs inside an aggregate already measured at 4,249 s here.
2. ⭐ **Whatever the fix, an EXISTING-artifact hand-off must be REFUSED when the artifact is absent
   or older than the run that is consuming it** — a reuse that cannot verify provenance must say so
   rather than proceed. The stale-consumption path must become impossible, not merely unlikely.
3. Sweep the class: `grep -n 'EXISTING_.*STATE_DIR=' rust/scripts/sota_exit_gate.sh` — the four here
   were found only because the run died at the fourth. Assume more.
4. Re-run `make -C rust SHELL=/bin/bash sota_exit_gate` end-to-end and require it to reach the end.

⛔ **This is flow-surface, so it belongs here and is NOT routed out**: it is the aggregate's own
stage wiring in `rust/scripts/sota_exit_gate.sh`, not an SV grammar or parser defect. Every sub-gate
named above is doing exactly what it was told.

- **Evidence:** `docs/tasks/artifacts/ci_parity_gate_rot/sota_exit_gate_after.txt` (the verdict, the
  guard marker, the root-cause citations and the measured staleness table).

### `.6` — the four surfaced items, adjudicated (`done`)

- **Status: `done`** (2026-07-28, session #220, `PGEN-CI-PARITY-GATE-ROT-0011`), on the director's
  instruction to decide them rather than leave them as findings. Each was PRICED before being
  decided; two of the four turned out to rest on a premise that measurement corrected.

#### (1) `sota-exit-gate.yml` timeout ⇒ **the platform maximum, 360 — because every lower number is a guess**

`.4` raised it 60 → 300 on 2× the local measurement. That number encodes an estimate of the
hosted/local slowdown factor, and **that factor is unmeasurable while hosted Actions are paused and
billable** — so 300 was precision this leaf does not have.

**Decision: 360, the GitHub per-job cap.** It is the only figure here that is not a guess: it is
what the platform allows. It costs nothing — Actions bill minutes *consumed*, not the budget
declared — and it strictly dominates 300, which differed from the cap only for runs between 300 and
360 minutes, i.e. **exactly the band this job might land in** (local measured 143 min).

⛔ **And the decision carries its own trigger condition, so it is closed rather than deferred:** if a
hosted run ever reaches 360, the answer is NOT a larger number — the cap is the cap. It is that this
aggregate does not fit one hosted job and must be **split into per-family jobs**. At that point the
split is justified by a measurement instead of by the speculation it would be today.

#### (2) the 28 `accepted-operator-invoked` lanes ⇒ ⭐⭐ **THE PREMISE WAS WRONG, AND MEASUREMENT SAYS SO**

The finding was framed as *"28 real proof lanes that nothing runs automatically — shrinking that set
is real work I did not do."* Measuring what actually auto-runs corrects it:

```
$ for f in .github/workflows/*.yml; do <read the `on:` block>; done
memory-architecture-gate.yml: workflow_dispatch,push,pull_request
<the other 14>:               workflow_dispatch
```

⇒ **14 of the 15 tracked workflows are `workflow_dispatch`-only**, and the one that still auto-runs
executes `bash scripts/check_*.sh` directly and **no `make` target at all**. So:

> ⭐⭐⭐ **the AUTOMATIC tier over these gate targets is ZERO — not 92, and not "all but 28".**
> The automatic layer covers the **11 enforced doctrines** and **none of the 123 make-level gate
> targets.** Every proof lane in this repository — the 92 "reachable" ones exactly as much as the
> 30 orphans — runs only when a human asks.

⛔ **That kills the proposed remedy.** With the automatic tier at zero, wiring an orphan into an
aggregate or a paused workflow moves it from ORPHAN to OPERATOR and **makes nothing run**. "Shrink
the 28 by wiring" would have been theatre — motion that improves a number without improving
coverage. ⭐ This is the tree's own principle turned on the tree's own instrument: *a check that
cannot run must SAY SO*, and my first report said "reachable from something that RUNS" about 92
targets whose invokers are paused.

**What an orphan actually costs, stated narrowly and correctly:** it is unreachable from any *entry
point* — running `sota_exit_gate`, running `ci_workflow_local_gate`, or dispatching any workflow
still will not reach it. That is why wiring is worth doing when a lane is cheap (the ten book gates,
0.3 s each) and worth **not** doing when it is not: bolting an hours-long external-corpus lane onto
the release aggregate makes the aggregate unrunnable, which *removes* coverage.

⇒ **DECISION: the 28 stay accepted, on the corrected reasoning, and the real lever is ESCALATED.**
The one change that would move the automatic tier off zero is **resuming hosted auto-triggers for a
cheap subset**. That spends account Actions minutes, which is precisely the cost decision the
director made when pausing them (`af85a5fd`, README "Hosted GitHub Actions pause") — so it is a
director call, not an engineering one, and it is raised as such rather than quietly taken.

**Mechanized, not just written down:** the check now classifies workflow roots as
`ci-workflow-auto` vs `ci-workflow-manual` by reading each `on:` block, reports the AUTOMATIC and
OPERATOR tiers separately, and pins the classification with two further ground-truth controls
(`memory-architecture-gate.yml` must be auto, `sota-exit-gate.yml` must be manual). New probe
**RED-5**: give a paused workflow a `push:` trigger ⇒ the instrument must declare itself
MISCALIBRATED rather than silently re-inflate the automatic tier.

#### (3) "an instrument with no ground truth is a confident guess" ⇒ **promoted to a tracked discipline**

Not a decision so much as a durable fact that must survive this session:
`docs/decisions/feedback_instrument_needs_ground_truth.md`, indexed. It records the six wrong
answers, the six defects behind them, and the rule they produce — pin known-true facts inside the
instrument, prefer controls that break in *both* directions, and **refuse rather than report** on
mismatch. Live reference shape: the control block of `scripts/check_gate_reachability.sh`.

#### (4) "an assertion about a file cannot live inside that file" ⇒ **PRICED, and deliberately NOT mechanized**

The tempting move was an 12th doctrine. `GENERATED-LINT-CORRECTNESS.4`'s rule is to price a
candidate against the whole corpus first — its own chartered hypothesis, generalized from one clean
sample, would have admitted 2 of 304 boxes. Priced here:

```
$ for f in rust/scripts/*.sh scripts/*.sh; do <self-targeting assert_* count>; done
rust/scripts/ci_workflow_local_gate.sh: 1 self-targeting assertion(s)
```

**Exactly one site repo-wide**, and it is already written in the sound form. ⇒ **a doctrine for one
occurrence is over-mechanization; refused.** Recorded instead as
`docs/decisions/reference_self_referential_assertion_is_unsound.md` (with the mechanical form
written out, so re-pricing later is cheap) plus the in-place comment at the site.

#### Acceptance checklist (enforced)

- [x] **ROOT CAUSE (WHY + WHERE)** — item (2)'s premise located and refuted by reading the trigger
      block of each tracked workflow: `git ls-files '.github/workflows/*.yml'` = 15, of which 14 are
      `workflow_dispatch`-only and `memory-architecture-gate.yml` (the sole `push`/`pull_request`
      one) invokes **no** `make` target ⇒ AUTOMATIC tier = **0** of 123. Item (4)'s premise priced
      by sweeping every `rust/scripts/*.sh` and `scripts/*.sh` for self-targeting `assert_*` calls
      ⇒ **1** site. Item (1)'s basis is the unmeasurability of the hosted slowdown while Actions are
      paused, not a disputed number.
- [x] **ADDRESSED (verified)** — before→after on the instrument: it reported *"reachable from
      something that RUNS: 92"* and now reports `AUTOMATIC 0 / OPERATOR 92` with the pause named.
      `bash scripts/check_gate_reachability.sh` → `OK (123 targets; 92 reachable, 30 orphan +
      1 policy-only, all dispositioned; 8 ground-truth controls reproduced)` — controls 6 → 8.
      Probes **9/9** including the new RED-5 (a paused workflow given `push:` ⇒ MISCALIBRATED).
      `.4`'s probes re-run **13/13** after the timeout change.
- [x] **NO REGRESSION** — no `grammars/*.ebnf`, no `rust/src/*`, no `generated/*` ⇒ all 11 parsers
      **byte-identical BY CONSTRUCTION**; no release / schema / ledger / contract movement. All 15
      workflow YAMLs parse; `bash scripts/check_doctrines.sh` → **ALL 11 enforced doctrines PASS**.
      ⛔ `rust/Makefile` and `rust/scripts/sota_exit_gate.sh` deliberately untouched: the aggregate
      run proving `.5` was executing, and editing a Makefile or gate script under a live `make` is
      a way to invalidate a measurement without noticing.
- [x] **LOCKSTEP** — `docs/decisions/` (2 new records + `INDEX.md`), the register's
      `measured_context` / `what_an_orphan_actually_costs` / `honest_limit` fields, `CHANGES.md`,
      `DEVELOPMENT_NOTES.md`, `MEMORY.md`, and this tree.

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

### `.20` — ROUTED IN from `LIVE-MEANS-LIVE.1c2`: two fresh `ci_workflow_local_gate` audit-phase blockers (`todo`)

Measured 2026-07-31 at commit `e1c28f38` while `LIVE-MEANS-LIVE.1c2` was editing four of this
gate's audit functions. The gate **aborts in its audit phase**, so its replay phase is unreachable.
Both blockers are **pre-existing and independent of that leaf** — routed here rather than worked,
per [[feedback_flow_findings_are_routed_not_worked]], because they block no *published* claim.

#### ⭐⭐ `.20a` — one doctrine, two enforcers, opposite verdicts (a genuinely new shape for this tree)

| enforcer | tool | verdict on the same tree |
|---|---|---|
| `scripts/check_diagnostics_and_docpaths.sh` | `git grep` | **OK** |
| `rust/scripts/ci_workflow_local_gate.sh` (`audit markdown repo-path policy`, `:2660`) | `rg` | **FAIL** |

The single hit is `stimuli/generators/anvil/docs/tasks/LOCAL-REFERENCE-CACHE.md:45`, which carries an
absolute home-directory checkout path — of the form `/Users/<user>/` … ending in `pgen/`.
⚠️ **The literal is deliberately NOT reproduced here**: `docs/tasks/**` is itself on the guarded
surface, so pasting it makes this routing note trip the very doctrine it reports. (Measured — the
first draft of this paragraph did exactly that and `check_diagnostics_and_docpaths.sh` blocked the
commit. The guard is correct; it is a nice demonstration that the `git grep` arm does bite where it
can see.) It lives inside the **`anvil` git submodule**
(`git submodule status stimuli/generators/anvil` → `ecda0e78`), so:

- `git grep` / `git ls-files` do **not** descend into it ⇒ the doctrine's own enforcer is blind to it;
- `rg` walks the filesystem ⇒ the parity gate sees it;
- ⛔ and **PGEN cannot fix it from here** — the file belongs to another repository.

⇒ the open question is *scope*, not the string: **is a submodule's markdown inside PGEN's live-doc
surface or not?** Both enforcers must then agree. ⭐ This is a new failure shape for this tree:
`.1`-`.19` are all *one* enforcer that rotted. Here **neither is rotted — they disagree about what
they govern**, and a doctrine whose two enforcers disagree is not one doctrine.

##### ⭐⭐⭐ SHARPENED (2026-07-31, on the director's question *"Is ANVIL a git submodule of PGEN?"*)

It is — **fully registered**, and the answer makes this defect worse than "a scope disagreement":

```
$ git ls-files -s stimuli/generators/anvil          # 160000 ecda0e78…  (gitlink)
$ git config -f .gitmodules --get submodule.stimuli/generators/anvil.url
https://github.com/rdje/anvil
$ git ls-files -s | awk '$1=="160000"' | wc -l      # 24 submodules total
```

⚠️ ANVIL is 1 of **24**, and the only one that is not a third-party corpus — the other 23 live under
`stimuli/{sv,vhdl}/subs/` and are vendored upstreams (opentitan, verilator, ghdl, …). ANVIL is under
`stimuli/generators/` and is **the director's own repository**. So *"PGEN cannot fix it from here"* is
a **git-boundary** statement, not an ownership one: the fix is available, it just does not belong in a
PGEN commit.

**What the two arms actually govern, measured:**

| surface | markdown files |
|---|---|
| `git grep` — PGEN-tracked `.md` | **912** |
| `rg` over the tree | **2 014** — **2.2×** |
| …of which inside vendored submodules | **1 113 (55 %)** |

⇒ the `rg` arm silently claims jurisdiction over **1 113 files in 24 repositories PGEN does not
author**. That is not a defensible reading of *"every repo-INTERNAL path in a LIVE/maintained doc
surface"*; it is a glob that outgrew its charter.

**⛔ And the decisive fact — the verdict is ENVIRONMENT-DEPENDENT, which is this tree's exact axis:**

```
$ grep -rl 'actions/checkout' .github/workflows/*.yml | wc -l     # 15 workflows
$ grep -rn -A4 'actions/checkout' .github/workflows/*.yml | grep -c submodules
0
```

`actions/checkout` defaults to `submodules: false`, and **not one of the 15 workflows overrides it**.
So:

| environment | `stimuli/generators/anvil` | audit verdict |
|---|---|---|
| local clone with submodules updated | populated | ⛔ **FAIL** |
| CI (hosted) | **empty** | ✅ **PASS** |

⭐⭐ **`ci_workflow_local_gate.sh` is the CI-PARITY gate** — its entire purpose is *"run locally what
CI runs"*. It contains an audit that **cannot fail in CI and cannot pass locally**. That is a parity
break inside the parity instrument itself, and it is strictly worse than the scope disagreement first
recorded above: the guard is not merely governing the wrong set, it is governing a set whose
membership depends on whether someone ran `git submodule update`.

⇒ **the ruling this leaf owes is now narrow and obvious**: the `rg` audit must exclude submodule
paths (or simply switch to `git grep`, which already implements the correct scope and is the
doctrine's own enforcer). Either way the two arms then agree **in both environments**, which is the
acceptance test — not just "the gate goes green locally".

#### `.20b` — a second assertion staled by a landing-page shrink

`audit_regex_corpus_bundle_surface` and `audit_regex_pcre2_compile_oracle_surface` require two
literals in `README.md` that `README-POLICY.1` (`d29c3dd7`) **removed** when the README was cut to a
landing page. `git log -S` names that commit for both. This is the tree's established class (a
literal pinned against a moving surface — cf. the `1.1.29`/`1.1.31` constants and the 1 371-commit
`generated/` assertion), so the fix is likely `assert_file_matches` against the *canonical
destination* the content moved to, not re-pinning README.

⚠️ **Do not stop at the first green** — this gate is fail-fast, so `.20a` currently hides `.20b`,
which in turn may hide more. The acceptance is the gate reaching the end of its audit phase.

⭐ **Measured alongside, and worth keeping**: of the **89** `assert_file_contains` arms in the four
audits `LIVE-MEANS-LIVE.1c2` touched, **87 pass** and the only 2 failures are `.20b`'s. So the audit
phase is *nearly* green and is being held down by a small, bounded set of stale literals — a cheap
leaf, not a campaign.

## Evidence

- Measured at commit `730419a2`, session #215. The census was produced by sourcing
  `rust/scripts/ci_workflow_local_gate.sh` with its trailing `main "$@"` stripped and invoking
  each `audit_*` function in its own subshell — 31 functions total, 8 failing after the
  `assert_generated_artifact` repair.
- `.20` measured 2026-07-31 at commit `e1c28f38` (session #229) by the same technique — sourcing the
  gate's own helper + audit function bodies rather than restating their rules — plus a
  working-tree-vs-`HEAD` differential over every `assert_file_contains` arm, which is what separated
  the 2 pre-existing failures from 0 caused by the leaf that found them.
- Provenance of the first blocker: `git log -1 --format="%h %ad %s" --date=short 0ed2b2ad` →
  `0ed2b2ad 2026-04-29 Slice 5: stop tracking generated/* in git`;
  `git merge-base --is-ancestor 0ed2b2ad HEAD` → true;
  `git rev-list --count 0ed2b2ad..HEAD` → `1371`.

## Commit log

| slice | leaf | commit subject |
|---|---|---|
| `PGEN-CI-PARITY-GATE-ROT-0011` | `.6` | the four surfaced items adjudicated — and measurement refuted the premise of one: the AUTOMATIC tier over these gate targets is ZERO, so "wire the 28" would have been theatre |
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

