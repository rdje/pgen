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

### ⚠️ `.41` NEW `todo` — **`GENERATED-REPRODUCIBILITY` checks an 11th artifact that the canonical regeneration recipe does not produce, so every codegen change fails its own rebaseline once** (routed in 2026-08-21 session #253 by `ENGINE-UNIVERSAL-SERVICES.43`)

> ⛔ **ROUTED, NOT WORKED** — found while executing the SV-release frontier. Small, but it costs a
> confusing failed rebaseline to every engine change, and it is a one-line decision either way.

- ⛔ **MEASURED.** After `.43` regenerated everything via
  `make -C rust regenerate_generated_parsers`, `generated_reproducibility_rebaseline` REFUSED:
  ```
  generated-reproducibility: scratch  DOES NOT re-derive from HEAD:
      live 3246bfaf04f1… (261391 B) vs fresh 38ae770d6c3e… (263942 B)
  generated-reproducibility: refusing to rebaseline: tier 2 found a breach.
      Regenerate the artifacts, do not record the divergence.
  ```
  ⭐ **The other TEN are perfect** — `return_annotation`, `semantic_annotation`, `json`, `regex`,
  `systemverilog`, `systemverilog_preprocessor`, `vhdl`, `rtl_const_expr`, `rtl_frontend`, `ebnf`
  each *"re-derives byte-identically (0 sites)"*. The sole breach is `scratch`.
- ✅ **ROOT CAUSE, and the gate behaved CORRECTLY.** `scratch` is not in
  `GENERATED_PARSER_FAMILIES`, so `regenerate_generated_parsers` does not regenerate it — but
  `GENERATED-REPRODUCIBILITY` tier 2 *does* check it. ⇒ after ANY codegen change the canonical
  recipe leaves exactly one artifact stale, and the doctrine correctly refuses to record a
  divergence it cannot explain. Nothing here is broken except the population mismatch: **the
  doctrine's set and the recipe's set differ by one member.**
- **WHAT THIS LEAF OWES**: (a) pick a side and make the two sets agree — either add
  `focus_scratch` to `regenerate_generated_parsers` (⚠️ interacts with the scratch slot's
  deliberate `rm`-and-rebuild dance and the `SCRATCH-SLOT-HEADER` probe-time tier, so it is not a
  blind one-liner), or declare `scratch` out of the reproducibility population as a *throwaway probe
  slot* with the reason recorded; (b) whichever side wins, the OTHER set must be the one that names
  the decision, so this cannot drift back. ⚠️ Do NOT "fix" it by rebaselining over the divergence —
  that is the exact act the doctrine exists to refuse.
- ⚠️ **HONEST BOUND**: one occurrence, on one codegen change. It is a population mismatch by
  construction rather than a flake, so it should reproduce on every codegen change — but that has
  not been shown twice.

### ⛔⛔ `.40` NEW `todo` — **THE COLD-CLONE BOOTSTRAP IS BROKEN, AND `.24` SLICE 2 BROKE IT — a one-line fix whose own evidence ("ZERO shipped bytes, generated parsers byte-identical") was TRUE and could not see this** (routed in 2026-08-21 session #253 by `ENGINE-UNIVERSAL-SERVICES.43`, which needed a full regeneration and hit it twice)

> ⛔ **ROUTED, NOT WORKED.** Found while executing the director-ruled SV-release frontier
> (`ENGINE-UNIVERSAL-SERVICES.43`). The lane lock binds work, not discovery, so this leaf carries
> the measurement and the fix direction and is **not** implemented here. ⭐ **Recommendation: it
> should jump the rest of this tree's queue** — see BLAST RADIUS.

- ⛔ **THE SYMPTOM — one error, reproduced twice, full evidence in
  [`artifacts/ci_parity_gate_rot/cold_clone_bootstrap_break.txt`](artifacts/ci_parity_gate_rot/cold_clone_bootstrap_break.txt).**
  From a tracked-files-only tree, `make -C rust regenerate_generated_parsers` seeds
  `generated/ebnf.rs`, generates the annotation pair, and then dies with **exactly one** error and
  zero of any other class:
  ```
  error[E0425]: cannot find function `active_grammar_profile` in module `crate::parser_registry`
     --> src/parse_harness_equivalence.rs:332:50
  note: found an item that was configured out
     --> src/parser_registry.rs:443:8
  435 | #[cfg(any(has_generated_systemverilog_parser, has_generated_regex_parser))]
  error: could not compile `pgen` (lib) due to 1 previous error
  ```
  Reproduction 2 is a CONTROLLED probe that removes only the two gating artifacts and builds into a
  SEPARATE `CARGO_TARGET_DIR`, so the working build cache is untouched and the variable is isolated
  to artifact presence: `cargo check --features "generated_parsers ebnf_dual_run" --lib` → the same
  single `E0425`.
- ⛔ **THE MECHANISM IS A CIRCULAR DEPENDENCY ACROSS THREE FILES, none of them recently touched.**
  `lib.rs:36` gates `parse_harness_equivalence` on **FEATURES**; `parser_registry.rs:435` gates
  `active_grammar_profile` on **ARTIFACT PRESENCE**; `parse_harness_equivalence.rs:332` calls it
  unconditionally. Meanwhile `rust/Makefile` has `regex_parser: $(REGEX_JSON) $(RUST_AST_PIPELINE)`
  and `$(RUST_AST_PIPELINE)` builds `--features "generated_parsers ebnf_dual_run"` ⇒ **the target
  that CREATES the first gating artifact requires a build that needs one to already exist.**
- ⭐⭐ **THE FIRST VERSION OF THIS FINDING WAS WRONG, AND THE CORRECTION IS THE VALUABLE PART.** It
  was written up as *"a longstanding hole that only works because `generated/` is never empty"*.
  **The repository's own record refutes that**: `.github/actions/regenerate-parsers/action.yml`
  states *"Measured cost from a bare tracked tree: ~258 s"* (`11fcb45c`, 2026-07-28, `.4`), and the
  failing call site is OLDER — `f2ae3d8f`, 2026-07-05, `PARSE-HARNESS.5.1`. So `.4` measured a
  **successful** cold regeneration on a tree that already contained the call. Resolving that
  contradiction instead of dismissing it is what produced the real finding.
- ✅⛔ **WHAT ACTUALLY HAPPENED — `0099d0d3`, 2026-08-11, `CI-PARITY-GATE-ROT.24` slice 2**, fourteen
  days after `.4` measured the path green:
  ```
  before:  cargo build --features generated_parsers --bin ast_pipeline
  at:      cargo build --features "generated_parsers ebnf_dual_run" --bin ast_pipeline
  ```
  `parse_harness_equivalence` is gated on **both** features. With `generated_parsers` alone it was
  never compiled, so `active_grammar_profile` was never referenced. Adding `ebnf_dual_run` pulled
  the module into the **bootstrap** build. ⇒ **a fix that closed a real trap opened a different one
  in a direction its own evidence could not see** — that commit's subject reads *"ops/build-flow,
  ONE recipe line; ZERO shipped bytes, generated parsers byte-identical"*, and **both halves are
  true**. Neither is a statement about the cold path.
- ⛔⛔ **WHY NOTHING CAUGHT IT — this tree's own founding story, recurring.** All **11** hosted
  workflows using the composite action are `workflow_dispatch` only (measured: every one of the 11
  `on:` blocks), which is deliberate documented policy (Actions minutes). The one local instrument
  that would catch it — `prepare_generated_artifacts` in `rust/scripts/ci_workflow_local_gate.sh`,
  which replays the bootstrap into a tracked-files-only EXPORT DIR, explicitly *"NOT a copy from
  the developer's tree"* — is **operator-invoked, not in the automatic tier**. Every developer and
  agent run has a populated `generated/`, where the path is green. Compare `.4`'s own words about
  its defect: *"hosted auto-triggers were paused … so the breaking change landed when there was no
  automatic run left to fail."*
- ⛔⛔⛔ **BLAST RADIUS — this is not an internal-ops-only defect.** `README.md`'s Quick Start names
  `make -C rust SHELL=/bin/bash regenerate_generated_parsers` as **the first command a fresh clone
  runs**, and that command fails on a fresh clone. All 11 hosted workflows would fail at their
  first step if dispatched. A downstream consumer building PGEN from a clean checkout hits it
  immediately — which is exactly the position Nexsim is in for the SV delivery.
- **WHAT THIS LEAF OWES**: (a) the fix — the minimal candidate is a `#[cfg(not(any(...)))]`
  companion for `active_grammar_profile` returning `None`, keeping `parse_harness_equivalence`
  compiling in a parser-less tree (⚠️ a DIRECTION, priced but not implemented — (a) must confirm it
  is the only such reference rather than the first one found); (b) **an automatic instrument**, or
  this recurs a third time — the cold path is currently proven only by a gate nobody is obliged to
  run; (c) a note in `.24`'s leaf, because that leaf's evidence is not wrong and should not be read
  as if it were.
- ⚠️ **HONEST BOUNDS**: I did not verify whether `ci_workflow_local_gate` has been run since
  2026-08-11 — it cannot have been run AND passed, so either it was not run or its failure was not
  acted on; and neither reproduction was a pristine `git clone` (one wiped `generated/`, one removed
  two files), though both isolate the same variable.

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

### `.21` — ROUTED IN from `SV-EXH-PROOF.7.4.6.11`: the `--lib` suite is RED on HEAD in **both** feature configurations, and no gate reads it (`todo`)

- **Status: `todo`** — opened 2026-08-01 session #230. **Routed, not worked** (it blocks nothing:
  it is stable, pre-existing, and orthogonal to the leaf that found it) per
  [[feedback_flow_findings_are_routed_not_worked]]. ⛔ Do **not** pull it ahead of product work.
- **Measured on HEAD, twice, with a `git stash` baseline** (the `SV-EXH-PROOF.8` precedent — the
  stash is what makes "pre-existing" a measurement rather than a claim):

  | suite | baseline (stashed) | with the `.7.4.6.11` change | verdict |
  |---|---|---|---|
  | `cargo test --lib` (no features) | **883 passed / 9 failed** | 885 / **9** | identical failure set |
  | `cargo test --features generated_parsers --lib` | **985 passed / 1 failed** | 987 / **1** | identical failure set |

  The `+2` in each case is exactly the two tests `.7.4.6.11` adds. ⇒ **all 10 failures pre-date it.**

#### The two distinct classes (they are NOT one defect)

**Class 1 — 8 tests the loud-refusal guard made unconditionally red in the no-features build.**
All 8 die on the same message: `REFUSED: return annotation '…' needs the generated annotation
backend, but this binary was built WITHOUT --features generated_parsers`. They are
`ast_pipeline::tests::` — `transform_from_raw_ast_preserves_return_and_semantic_annotations`,
`…_preserves_branch_semantic_annotations`, `…_preserves_mid_sequence_semantic_annotations`,
`…_promotes_transform_semantic_payload`, `inner_group_remap_patterns_a_through_d_stay_green`,
`mixed_and_trailing_group_annotation_disambiguation_is_unchanged`,
`whole_body_group_per_branch_annotations_keep_their_branches`,
`whole_body_group_trailing_annotation_broadcasts_to_every_runtime_branch`.
The guard was introduced by `RGX-0078.5.i.1.t2` (`200cae5b`, `PGEN-RGX-0078-0079`), whose own
verification records **`lib 941/0`** — i.e. green *as measured there*. So either the measurement was
taken in a feature-on configuration only, or these 8 have gone red since. **Establish which before
choosing a fix** — the two answers imply opposite remedies (feature-gate the tests, vs. the guard is
over-broad). Candidate fix if it is the former: `#[cfg(feature = "generated_parsers")]`, matching what
`.7.4.6.11` had to do to its own real-SV test for exactly this reason.

**Class 2 — 1 test red in BOTH configurations**:
`ast_pipeline::ast_based_generator::semantic_usage_tests::unresolved_reference_codegen_emits_semantic_fallback_and_stubs_boolean_names`,
failing its `expected semantic_annotation fallback to detect '@' directives` assertion. Owned by
`LANG-CAPABILITY-AUDIT.10.4`, which introduced it green. Reproduces feature-on *and* feature-off, so
it is **not** a feature-gating artifact — a genuinely different defect from class 1.

#### ROUTING EVIDENCE

- **Does the finding reproduce outside the family it is being sent to?** **Yes — that is precisely why
  it comes here.** Class 1 is an `ast_pipeline` annotation-transform surface and class 2 an
  `ast_based_generator` codegen surface; neither is SystemVerilog, and the leaf that found them
  (`SV-EXH-PROOF.7.4.6.11`) touched only `stimuli_generator.rs`. Both reproduce with that leaf's change
  stashed. It is therefore not an SV-family defect and does not belong in `SV-EXH-PROOF`.
- **Why THIS tree.** The defect is not that 10 tests fail — it is that they have been failing while
  every gate is green, which is this tree's subject (`.2`: *a check that nothing INVOKES is
  indistinguishable from a check that does not exist*). ⭐ **`--lib` is read by no gate in the
  registry.** `SV-EXH-PROOF.3.3.5` last recorded the full workspace at **759/0** (dual-feature 788/0);
  it is now 987/1 feature-on and 885/9 feature-off, and nothing fired in between. That is the same
  ratchet-shaped gap `GATE-REACHABILITY` exists for, one level down: the *gates* are now reachable, the
  *test suite* is not.
- **Not worked here, deliberately.** Fixing it inside `.7.4.6.11` would have co-mingled an unrelated
  repair with a measured before→after, and the stash baseline is exactly what let that leaf claim
  NO REGRESSION honestly instead of inheriting someone else's red.

#### Acceptance (when this leaf is picked up)

Establish class 1's history first (was `941/0` feature-on-only, or did the 8 regress?) — **evidence
before remedy**; then fix both classes; then close the reachability gap that let it happen: a
`--lib` suite in *both* feature configurations must be read by something that RUNS, with a
two-sided ratchet on the failure count so neither a new red test nor a *paid* one still listed can
pass silently (the `.10.6` `envelope_divergence_ceiling` shape). ⛔ Prove both directions fire before
trusting the ratchet ([[feedback_instrument_needs_ground_truth]]).

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

##### ⭐⭐⭐ A THIRD defect in the same 4-line function: **`rg` missing ⇒ the audit silently PASSES**

Found on the director's follow-up *"why aren't you using `rg` for everything?"*. The call site is:

```bash
audit_markdown_repo_relative_paths() {
  note "auditing markdown repo-path policy"
  if (cd "$ROOT_DIR" && rg -n --glob '*.md' '<abs-path>' . >/dev/null 2>&1); then
    fail "absolute PGEN checkout path found in markdown docs; use relative repo paths"
  fi
}
```

`rg` is **not** a git dependency — it is an optional third-party binary — and `grep -c 'command -v rg'`
over the gate returns **0**: its presence is never checked. So when `rg` is absent the subshell exits
non-zero, the `if` is false, and the audit **reports success without having searched anything**.
Reproduced with a PATH shim returning 127: *"audit PASSES ✅ …even though rg never ran"*.

⛔ That is the **vacuous-green** class this tree exists to eliminate (`.3`), reached here through a
tool dependency rather than a bad assertion. ⭐ `git grep` **cannot** have this failure mode: git is
already a hard dependency of every gate in the repo, so the tool's absence is not a silent state.

##### ⭐⭐ Why `git grep` is the right instrument — recursion was never the difference

The director's second question — *"but `git grep` knows how to work recursively, right?"* — is the
one that settles the ruling. **Yes.** Measured: it walks 912 tracked `.md` across 117 directories
from the root, and the submodule boundary is a **declared, overridable default**, not a depth limit:

```
$ git grep -l '<pattern>' -- '*.md'                        # submodules excluded  (the default)
$ git grep --recurse-submodules -l '<pattern>' -- '*.md'   # …/anvil/docs/tasks/LOCAL-REFERENCE-CACHE.md
```

⇒ **`git grep` can express BOTH scopes explicitly; `rg` can express only one, implicitly.** For a
*doctrine enforcer* that is decisive: the governed set becomes a choice a reviewer can read in the
source, instead of an accident of which binary was reached for. It also gives three things `rg`
structurally cannot — searching any tree-ish (`git grep <pat> <commit>`), searching the **index**
(`--cached`, i.e. exactly what is being committed, which is what a pre-commit hook should test), and
immunity to untracked-file false positives (a developer's scratch note cannot fail the gate).
And on the set that actually matters it is faster, not slower: **0.02 s vs 0.10 s** (best of 3).

⇒ **Revised ruling: replace `rg` with `git grep` here**, and state the submodule scope explicitly by
the presence or absence of `--recurse-submodules`. That fixes all three defects at once — the scope
disagreement, the environment-dependent verdict, and the silent pass on a missing binary.

##### ✅ IMPLEMENTED — DIRECTOR RULING (2026-07-31), in two steps, and the SECOND one is the ruling

**Step 1** — *"switch to `git grep` and enable recursivity into submodules by default."* Implemented,
and it went red on `stimuli/generators/anvil/…/LOCAL-REFERENCE-CACHE.md`.

**Step 2 — ⛔ THE SCOPE RULING THAT SUPERSEDES IT:**

> *"ANVIL is a totally different project. A submodule is a way to access its entire codebase locally.
> For me submodules shall be treated as **READ-ONLY LINKED REPOS**. I am not even sure why you want
> to change the content of the ANVIL repo."*
> …and: *"I get that, so **do not use recursivity by default** then."*

⇒ **`--no-recurse-submodules`, passed EXPLICITLY.** This doctrine exists so *PGEN's own* docs survive
the repo moving to another path or filesystem. A linked project's docs are governed by that project.

⭐⭐ **The red gate was the argument.** With recursion the only two ways to clear it were *edit
another repository* or *carry a permanent red* — and when a check's only remedies lie outside the
project, the check has overreached. ⛔ **I proposed the first of those, and that was the error**: I
treated a foreign repo's file as a defect to be fixed rather than as evidence that PGEN's rule had
escaped its own boundary. Re-read on its own terms, the ANVIL line is ANVIL's task tree recording
what ANVIL's owner did locally — a correct entry in someone else's book.

⭐ **And the mechanical case agrees with the principle**, which is the part I had backwards. Recursion
left defect 2 (the environment-dependent verdict) **open**: CI checks out no submodules, so a
submodule-scoped rule passes in CI and fails locally, and closing it would have meant cloning 24
repos including `opentitan`, `verilator`, `ghdl`, `Surelog`. Scoping to PGEN's own tracked files
**closes defect 2 outright** — both environments see the identical set — and costs nothing.
⇒ `.20c` is **MOOT and closed** below.

**Blast radius, measured — this is what the rejected recursion WOULD have governed** (~1 108 extra
markdown files, 912 → **2 020**, across 24 submodules):

| pattern | hits under recursion |
|---|---|
| the audit's exact literal | **1** — `stimuli/generators/anvil/docs/tasks/LOCAL-REFERENCE-CACHE.md` |
| broadened to ANY user (`/Users/[^ )`]*/pgen/`), restricted to `stimuli/*` | **1** — the same file |

⇒ the 23 vendored third-party corpora are clean and ANVIL is the sole hit — but *"it happens to
find only one thing today"* was never a reason to claim jurisdiction over 1 108 files in 24 foreign
repositories. ⭐ **The scope question is settled by ownership, not by hit count.**

###### Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — `make -C rust SHELL=/bin/bash ci_workflow_local_gate` aborted in its
  audit phase: `error: absolute PGEN checkout path found in markdown docs`, from a 4-line function
  carrying three defects (scope, environment-dependence, silent-pass-on-missing-`rg`).
- [x] **ROOT CAUSE (WHY + WHERE)** — ops/build-flow tier,
  `rust/scripts/ci_workflow_local_gate.sh:269-274`. Located with `git ls-files` + `git grep` + a
  `command -v` census, and the silent-pass arm reproduced with a PATH shim:

  ```
  $ grep -c 'command -v rg' rust/scripts/ci_workflow_local_gate.sh          # 0 — never checked
  $ PATH=<shim returning 127> …  if rg …; then fail; fi
    -> audit PASSES ✅  ...even though rg never ran
  ```

  **WHY**: the call site treats *any* non-zero from `rg` as "clean", but non-zero conflates *no
  match* (1) with *the tool did not run* (127). A search tool that is absent therefore certifies the
  thing it never looked at.
- [x] **FIX** — `git grep --recurse-submodules -nI -F`, with a **three-way** exit contract replacing
  the boolean: `0` = violation ⇒ `fail` (and the offending lines are printed), `1` = the only passing
  outcome, `>=2` = the audit could not run ⇒ `fail` with the exit code named.
- [x] **ADDRESSED (verified)** — measured before → after:

  | measurement | before (`rg`) | after (`git grep --recurse-submodules`) |
  |---|---|---|
  | governed set | filesystem minus `.gitignore` — **undeclared**, incl. 24 foreign repos | **PGEN's own tracked files**, declared via explicit `--no-recurse-submodules` ✅ |
  | agrees with the doctrine's own enforcer on tool | ⛔ no (`rg` vs `git grep`) | ✅ same tool |
  | tool absent ⇒ | **silent PASS** | impossible — git is a hard dependency, and any error `>=2` REFUSES ✅ |
  | untracked scratch file can fail the gate | yes | **no** — `git grep` reads tracked content ✅ |
  | speed on the tracked set | 0.10 s | **0.02 s** ✅ |
  | verdict on the real tree | FAIL (unactionable — no file named) | **PASS** — PGEN's own docs are clean; a real violation still FAILs naming file:line ✅ |
  | environment-dependence (defect 2) | open | **CLOSED** — no submodule in scope, so CI and local see the identical set ✅ |
- [x] **EVERY ARM PROVEN TO FIRE — including two controls that I had to REBUILD because my first
  cut was wrong.** Recorded because the failures were instructive, not incidental:

  | control | result |
  |---|---|
  | `CTRL-A` recursion toggled | flips the verdict ⇒ the scope flag is load-bearing, not decoration ✅ |
  | `CTRL-B` a submodule path under the SHIPPED scope | **not reached** ⇒ ANVIL is out of scope, as ruled ✅ |
  | `RED-1` absolute path planted in a **tracked** `.md` | **fails**, naming `docs/TASK_TREE_README.md`; file restored byte-identical ✅ |
  | `RED-2` `git grep` forced to error (`GIT_DIR=/nonexistent`) | **REFUSES** — *"the audit could not run, which is NOT a pass"* ✅ |

  ⚠️ **`RED-1` first failed, and the reason is the fix's own selling point**: I planted into an
  **untracked** file, which `git grep` correctly does not see. ⚠️ **`RED-2` first failed** because I
  pointed `ROOT_DIR` at `rust/target/notagitrepo` — still *inside* the repo, so `git grep` worked
  fine. Both controls were broken, not the code. ⇒ *a control that fails must itself be
  root-caused before the code is blamed* — the same discipline `.1c2` applied to a blocked oracle.
- [x] **NO REGRESSION** — `bash -n` clean (⚠️ `shellcheck` **not installed** — stated, not implied).
  ⛔ No `grammars/*.ebnf`, no `rust/src/*`, no `generated/*` ⇒ all generated parsers byte-identical
  BY CONSTRUCTION. `bash scripts/check_doctrines.sh` → **ALL 15 PASS**. `audit_root_markdown_surface`
  still PASS. No other audit function touched.
- [x] **LOCKSTEP** — the function's own WHY block (the three defects and the refusal polarity are
  documented at the call site, not only here), this leaf, `CHANGES.md`, `DEVELOPMENT_NOTES.md`.

###### ✅ WHAT THIS CLOSES, AND THE ONE THING IT DELIBERATELY DOES NOT

All three defects are closed: scope is **declared** (and correctly bounded to PGEN), the
environment-dependent verdict is **gone** (nothing in scope differs between CI and local), and the
silent-pass-on-a-missing-binary is **structurally impossible** (git is a hard dependency, and any
error `>=2` refuses). The audit **PASSES** on the real tree.

⛔ **Deliberately not done: nothing was changed in the ANVIL repository, and nothing should be.**
A submodule is a read-only linked project. ⚠️ Left open on purpose and named here so it is not
rediscovered as a finding: `stimuli/generators/anvil/docs/tasks/LOCAL-REFERENCE-CACHE.md` contains a
machine-specific path. That is ANVIL's business under ANVIL's own policy — **not a PGEN defect, and
not a PGEN commit.**

### `.20c` — CI submodule checkout (`closed — MOOT`, 2026-07-31)

Opened when the audit still recursed: CI declares `submodules:` in **0 of 15** workflows, so a
submodule-scoped rule could never agree between CI and local. Options priced were `submodules:
recursive` everywhere (clones all 24, incl. `opentitan`/`verilator`/`ghdl`/`Surelog`), a targeted
`stimuli/generators/anvil` checkout, or accepting the rule as local-only.

⇒ **None is needed.** The director's read-only-linked-repo ruling removes submodules from the
doctrine's scope entirely, so there is nothing left to keep in sync. ⭐ **The cheapest fix to a
parity problem turned out to be shrinking the claim rather than growing the checkout** — worth
remembering the next time an enforcer's scope and its environment disagree: check whether the scope
was ever justified before paying to make the environment match it.

### `.22` — ROUTED IN from `SV-CORPUS-GRAD.3.18`: a census-moving grammar change left `sv_cert_recognized_union_gate` RED for a whole release, and nothing said so (`todo`)

- **Status: `todo`** (opened 2026-08-09, session #219). Routed, not worked — it does not block
  the SV release lane, but it prices what a green commit means.
- **WHAT HAPPENED, measured.** `SV-CORPUS-GRAD.3.14b` (commit `3e316e3c`, release `1.0.178`)
  added two grammar rules — `in_scope_compiler_directive` (satisfiable under all three profiles)
  and `in_scope_compiler_directive_sv_only` (declared `sv_2017`+`sv_2023`) — and did **not**
  re-baseline `rust/test_data/grammar_quality/systemverilog_recognized_cert_union_contract.json`,
  whose `expected_total` had been `1352` since `SV-CORPUS-GRAD.3.9` (commit `6a2c088a`). The
  gate therefore failed with `canonical total=1354 (expected 1352)` at all three seeds — and
  that state **shipped**, surviving an entire release until the next leaf happened to run the
  gate.
- **ATTRIBUTION IS MEASURED, NOT INFERRED** (this matters, because the natural and wrong reading
  is that the leaf which *found* the red caused it): `ast_pipeline --dump-rule-profiles` on the
  pre-fix and post-fix `SV-CORPUS-GRAD.3.18` grammars gives an **identical** per-profile census
  — `sv_2017` 1354 → 1354, `sv_2023` 1373 → 1373, `verilog_2005` 1122 → 1122 — with **zero**
  rules changing their satisfiable-profile set. And `git show 6a2c088a:grammars/systemverilog.ebnf`
  contains **neither** new rule. `1352 + 2 = 1354` is then arithmetic over provenance.
- ⭐ **THE ROOT CAUSE IS COVERAGE, NOT CARELESSNESS.** This gate is **operator-invoked**: the
  automatic per-push tier is the 17 doctrines plus three cheap gate targets, and the other ~120
  `make` targets run only when a human runs them. So a census-moving grammar edit can land, pass
  everything automatic, and leave a proof gate red with **no signal of any kind** until the next
  operator happens to invoke it. The gate did its job perfectly the moment it was run; the defect
  is that nothing runs it.
- **Owed:** decide the cheapest sufficient signal. Candidates, in rising cost: (a) a doctrine
  check that fails the commit when `grammars/*.ebnf` changes the rule census without touching the
  matching `*_cert_*_contract.json` — cheap, purely local, needs no cert run; (b) promote the
  census comparison (not the full cert run) into the automatic tier; (c) promote the whole gate,
  which is ~13 minutes and almost certainly too expensive per push.
- ⭐⭐ **IT WAS TWO GATES, NOT ONE — the class prediction was CONFIRMED in the same session, and
  the split is the corroboration.** The paragraph above originally ended "check the sibling
  contracts first… assume the class, not the instance"; running the rest of the batch did exactly
  that, and `verilog_2005_conformance_gate` was **also red** on the same commit —
  `cert total=1122 (expected 1121)`, `cert proof=329 (expected 328)`. **`+1` there against `+2`
  in the union contract, and that asymmetry is the fingerprint:** `.3.14b` added two rules, but
  only `in_scope_compiler_directive` (`declared_profiles: null`) is satisfiable under
  `verilog_2005` — its twin `in_scope_compiler_directive_sv_only` is `@profiles`-gated to
  `sv_2017`+`sv_2023`. Two contracts, two different deltas, one cause, both predicted by the
  rule-profile dump before either gate was re-run. Both re-baselined by `SV-CORPUS-GRAD.3.18`
  with attributing notes; the behavioural half of the v2005 gate was GREEN throughout (corpus
  matrix 240 file × profile checks, 0 mismatches, `profile_orphans 0`), so nothing about the
  *language* had regressed — only the recorded census.
- ⇒ **the fix must be the class-wide one (a), not a per-gate patch.** A contract-vs-census
  consistency check is cheap and local; two instances in one commit is enough evidence that
  hand-remembering to re-baseline does not hold.
- ✅ **`SV-CORPUS-GRAD.3.20` (2026-08-09, release `1.0.181`) is the first leaf to run the drill
  deliberately, and it worked** — the census move was PREDICTED in the leaf before the edit
  (`1477→1478`, `sv_2017`/`sv_2023` +1, `verilog_2005` +0), measured with `--dump-rule-profiles`
  *before* either gate was run, and the union contract re-baselined **in the same commit** while
  the v2005 contract was correctly left alone (its cert census `1122/329/779/14` came back
  unchanged). ⚠️ That is evidence the *procedure* works when followed, **not** evidence the gap is
  closed: it was closed by an author who had read this leaf. The class-wide check (a) is still
  the fix.

### `.23` — ROUTED IN from `SV-CORPUS-GRAD.3.20`: `clippy_on_rust_change` is structurally blind to a pure-grammar change, because the artifact it lints is gitignored (`todo`)

- **Status: `todo`** (opened 2026-08-09). Routed, not worked — it does not block the SV release
  lane ([[feedback_flow_findings_are_routed_not_worked]]), and the leaf that found it worked
  around it explicitly. But it is the same shape as every other entry in this tree: a check that
  silently does not run.
- **WHAT HAPPENED, measured.** `SV-CORPUS-GRAD.3.20` edited only `grammars/systemverilog.ebnf`,
  regenerated `generated/systemverilog_parser.rs` (131 MB of Rust), and then ran the mandatory
  `COMMIT.md` step 2:

  ```text
  $ make -C rust SHELL=/opt/homebrew/bin/bash clippy_on_rust_change
  No Rust/generated Rust changes detected; skipping clippy flow.
  ✅ clippy_on_rust_change completed.
  ```

  It exits **0** and prints a **✅**. The generated parser had just been rewritten in full.
- **ROOT CAUSE (WHY + WHERE).** `rust/scripts/clippy_on_rust_change.sh:46-66` derives its trigger
  set from git:

  ```bash
  git diff --name-only; git diff --cached --name-only; git ls-files --others --exclude-standard
  ```

  and tests each path against a glob list that *does* include `generated/*.rs`. But `generated/`
  is `.gitignore`d by repository policy (`COMMIT.md`: *"the `generated/` tree is not tracked in
  git"*), so `--exclude-standard` removes it from the untracked listing and `git diff` never sees
  it either. **The one path class the glob was written to catch is the one class git cannot
  report.** A grammar-only change therefore never triggers the flow.
- **WHY IT WAS NEVER NOTICED.** Every prior grammar leaf in this campaign also touched a tracked
  Rust file — typically `rust/src/ast_shape_contract.rs` (to add a shape sample) or a
  `rust/test_data/.../*.json` manifest — and `rust/*.rs` matched, so the flow ran for an
  incidental reason. `.3.20` is the first leaf whose only code change is the grammar plus a
  contract JSON that is **not** in the glob list, so it is the first to expose the gap.
- **WORKAROUND USED** (so the leaf's NO-REGRESSION box is honest): `PGEN_CLIPPY_FORCE=1`, which
  ran all three stages GREEN — `clippy_source_all_targets`, the STRICT
  `clippy_generated_all_targets`, and `GENERATED-CLIPPY-CORRECTNESS: ✅ POLICY-ONLY PASS`.
- **CANDIDATE FIXES** (adjudicate before implementing; do not assume the first is right):
  1. add `grammars/*.ebnf` to the trigger glob — cheap, and correct in the sense that a grammar
     edit *is* a codegen change; risks running a multi-minute clippy pass on a docs-only grammar
     comment edit;
  2. trigger on the generated artifacts' **mtime/content hash** versus a recorded stamp, so the
     flow keys on "the artifact this lints actually changed" rather than on git's view of it —
     this is the variant that matches the check's own intent;
  3. make the flow REFUSE (nonzero) rather than print ✅ when it can find no evidence either way,
     per the `LIVE-DOC-CURRENCY` precedent that an unmeasurable population must not report as a
     pass.
- ⭐ **The general lesson, and why it belongs in this tree:** `GATE-REACHABILITY` asks whether
  anything *invokes* a gate. This is the adjacent failure — the gate is invoked, exits 0, prints
  a tick, and **checked nothing**. A skip that is indistinguishable from a pass is worse than an
  orphan gate, because it manufactures evidence. Worth asking whether any other
  `*_on_*_change.sh`-style conditional flow keys on git for an artifact git cannot see.

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


### `.24` — `target/debug/ast_pipeline` has THREE feature sets and no owner: the canonical build rule predates the feature it omits by one day (`todo` — ROUTED IN from `ENGINE-UNIVERSAL-SERVICES.10`, 2026-08-11 session #217; ⛔ PARKED behind the SV lane lock)

- **Status: `todo`** — **routed, not worked.** It blocks nothing (every affected flow has a manual
  workaround: rebuild with both features), and the SV lane lock binds work, not routing. ⭐ **Re-open
  trigger:** the next time any flow reads a `--report-feature-surface`-less measurement from
  `target/debug/ast_pipeline`, or the next `sota_exit_gate` end-to-end run, whichever comes first.

#### THE FINDING — and the director's question is what produced it

The session that found this reported it as *"`make focus_scratch` silently downgrades the shared
`ast_pipeline` binary"* and asked whether that was worth a leaf. The director asked the better
question: **why was the downgrade allowed in the first place — was there a rationale?**

There was not, and the archaeology is unambiguous:

| date | commit | what happened |
|---|---|---|
| **2026-02-19** | `36678059` | `$(RUST_AST_PIPELINE)` gets its build rule: `cargo build --features generated_parsers --bin ast_pipeline`. `git show 36678059:rust/Cargo.toml \| grep -c ebnf_dual_run` → **0** — the feature **did not exist yet**. |
| **2026-02-20** | `de8ca8bc` | `ebnf_dual_run` is introduced — **one day later**. |
| **2026-03-23** | `fc63a64f` | The frontend build is moved to its own `target/ebnf_frontend_build` target-dir, *specifically so it does not clobber the main binary*. |

⇒ **nobody ever decided to omit `ebnf_dual_run` from the canonical rule.** The rule simply predates
the feature and was never revisited. The third row is the important one: the clobbering collision
**was** recognized — once, for one consumer — and solved by isolating that consumer, while the
general case was left standing.

⛔ **And the original framing was wrong, which is why the director's question mattered.**
`focus_scratch` does not downgrade anything; it is an innocent consumer. `$(SCRATCH_PARSER):
$(SCRATCH_JSON) $(RUST_AST_PIPELINE)` — like **21** other references — depends on the canonical
rule, so whenever `$(AST_PIPELINE_SOURCES)` is newer than the binary, make rebuilds it *to the rule's
declared feature set*. That is make doing its job correctly against a stale declaration.

#### WHAT IS ACTUALLY BROKEN — one path, three feature sets, last writer wins

Measured on `rust/Makefile` at this commit:

| feature set written to `rust/target/debug/ast_pipeline` | recipes |
|---|---|
| `generated_parsers` only | **1** (the canonical `$(RUST_AST_PIPELINE)` file rule — 21 targets depend on it) |
| `ebnf_dual_run` only | **3** (`regex_parser_bootstrap`'s seed/verify steps) |
| `generated_parsers ebnf_dual_run` | **2** (`parse_harness_combinator_gate`, `parse_harness_semantic_gate`) |

Nothing declares what that path is *supposed* to be, so its capability is a function of which target
ran most recently. `ast_pipeline` already knows how to answer the question
(`--report-feature-surface`, added by `PARSE-HARNESS.10` for the #140-class trap) — but that check
lives in the parse harness, i.e. in **one consumer**, not at the path.

#### THE COST, measured rather than hypothesized

The routing session's own before/after sweep returned an **empty `CERTIFICATE-COVERAGE:` line for all
18 runs on each side** (`Error: EBNF input 'grammars/json.ebnf' requires building with --features
ebnf_dual_run` — on stderr, while the sweep parsed stdout). ⛔ **Two empty sets diff clean**, so an
unnoticed run would have published "zero cert drift across every grammar × seeds 0/7/42" backed by
nothing. It was caught only because a human-scale sanity check — *"json cannot really have vanished"*
— prompted re-running `--report-feature-surface`. That is not a control.

#### ⭐ SLICE 1 LANDED (`PGEN-CI-PARITY-GATE-ROT-0027`, 2026-08-11) — the REFUSAL, on director request

The director's follow-up was the right one: *"is there any action you can take to make sure the
problem does not happen again and messed up your before→after comparisons?"* Documentation is not
that action — a control that runs **before a number is published** is.

`scripts/require_ast_pipeline_features.sh <bin> <feature>...` refuses to let a measurement proceed on
an under-featured binary, naming the exact rebuild command. Its contract is **refuse rather than
guess**: a missing binary, a binary too old to answer `--report-feature-surface`, an unparseable
surface, or an unknown feature name are all refusals — because *"I could not tell"* and *"it is fine"*
must never share an exit code.

⭐ **It is proven to FIRE, which is the part that makes it worth anything.** `--self-test` builds stub
binaries and asserts **4 controls** every run — POSITIVE (a complete binary is accepted), NEGATIVE
(`ebnf_dual_run=false` is refused), MISSING-BINARY, and MUTE-BINARY. Same positive+negative discipline
the envelope differential (TOOLBOX 1.9) runs before it publishes a number; a guard nobody has watched
refuse is indistinguishable from `true`.

```
$ scripts/require_ast_pipeline_features.sh --self-test
AST-PIPELINE-FEATURE-GUARD: self-test ok — 4/4 controls (the guard is proven to fire)
$ scripts/require_ast_pipeline_features.sh rust/target/debug/ast_pipeline generated_parsers ebnf_dual_run
AST-PIPELINE-FEATURE-GUARD: ok — 'rust/target/debug/ast_pipeline' carries [generated_parsers ebnf_dual_run]
```

⛔ **GATE-REACHABILITY disposition, stated rather than glossed:** this slice ships the guard as a
**tool**, invoked by hand and by the self-test, and **wires it into no flow yet**. That is a real
limitation — an uninvoked check is weak — and it is deliberate: wiring it into
`sv_cert_recognized_union_gate.sh` (the obvious first host) changes a gate script, whose no-regression
evidence is a ~10-minute 3-seed run this slice did not have time to take. **Owed by slice 2**, below.

#### OWED — candidates, not a chosen design (the pricing is this leaf's job)

0. ✅ **SLICE 2 LANDED (`PGEN-CI-PARITY-GATE-ROT-0028`, 2026-08-11) — option (1): the canonical rule
   now DECLARES the full feature set.** On the director's ruling once the cost was measured: *"this
   impact on speed only affects the time it takes to generate the parser's code. If that's the case
   then I do not care."* `$(RUST_AST_PIPELINE)` builds
   `--features "generated_parsers ebnf_dual_run"`, so the path's capability is a property of the
   PATH, and all 21 dependants inherit it. This removes the class rather than policing it; the guard
   from slice 1 stays as the measurement-site control (see the residual below).
   ⛔ **RESIDUAL, not closed:** the 3 `regex_parser_bootstrap` recipes still write the same path with
   `ebnf_dual_run` ALONE (losing `generated_parsers`). Deliberately untouched — that is the
   cold-clone seed path where `generated/` may be empty, so changing its feature set needs a
   cold-clone test this slice did not take. **Slice 3.**

1. **Give the path one owner.** Make the canonical rule build both features, and let the 21
   dependants inherit it. ⛔ Price first: `ebnf_dual_run` compiles the `.ebnf` frontend into every
   `focus_*` build, so this trades build time for correctness on the hot path.
2. **Or make the path self-describing at the point of use** — a tiny `require_feature_surface`
   make/shell helper the gates and any sweep call before measuring, generalizing `PARSE-HARNESS.10`'s
   pre-check from the harness to the binary path it guards.
3. **Or separate the paths** the way `fc63a64f` already did once, so a feature set never has to be
   inferred from build order.
4. ⭐ **Independent of which is chosen:** an empty metric line must never be read as a value. The
   sweep that produced this finding parsed stdout and ignored a stderr error; that shape is the
   `.11` stale-log-metric class this tree already owns.

#### ROUTING EVIDENCE

1. **Does the finding reproduce OUTSIDE the family it is being sent to?** It has no family — it is a
   property of the shared build path, and it is measured across **three unrelated consumer groups**:
   a codegen target (`focus_scratch`, via 21 `$(RUST_AST_PIPELINE)` dependants), a bootstrap flow
   (`regex_parser_bootstrap`), and two parse-harness gates. `CI-PARITY-GATE-ROT` is the right home
   because it already owns the enforced `FLOW-INTEGRITY` doctrine, whose clauses — *"the recipe keeps
   one home"* and *"a guard tests the artifact it actually READS"* — name this defect almost
   literally.
2. **What was MEASURED, not what makes it plausible?** `git show 36678059:rust/Cargo.toml | grep -c
   ebnf_dual_run` → 0, against the three dated commits above; the three-way feature-set census of
   `rust/Makefile` (1 / 3 / 2 recipes); `grep -cE '\$\(RUST_AST_PIPELINE\)' rust/Makefile` → 21; and
   the live false-green (18 empty `CERTIFICATE-COVERAGE:` lines per side, recovered by
   `--report-feature-surface` reporting `ebnf_dual_run=false`).
3. **What would make the routing WRONG, and was it checked?** It would be wrong if the single-feature
   build were a deliberate, documented cost decision — then this is a pricing question for whoever
   made it, not a rot finding. **Checked and refuted**: the feature did not exist when the rule was
   written, and no decision record, task leaf or Makefile comment mentions the trade-off. It would
   also be wrong if `PARSE-HARNESS.10` already owned the general case; it does not — its pre-check is
   scoped to `compile_and_parse`'s own probe and cannot see an ad-hoc sweep or a `focus_*` rebuild.

#### Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — a live false-green, measured: a before/after cert sweep emitted an
  empty `CERTIFICATE-COVERAGE:` row for **18 runs on each side**, because
  `./rust/target/debug/ast_pipeline grammars/json.ebnf --report-certificate-coverage` printed
  `Error: EBNF input 'grammars/json.ebnf' requires building with --features ebnf_dual_run` on stderr
  while the sweep scraped stdout. `diff before.txt after.txt` compared **EQUAL** — i.e. it would have
  published "zero cert drift across every grammar × seeds 0/7/42" backed by nothing.
- [x] **ROOT CAUSE (WHY + WHERE)** — WHY: `rust/target/debug/ast_pipeline` is written by three
  recipes with three feature sets and nothing declares which it should be, so its capability depends
  on build order. Measured on `rust/Makefile`: `generated_parsers` alone ×1 (the canonical
  `$(RUST_AST_PIPELINE)` file rule), `ebnf_dual_run` alone ×3, both ×2;
  `grep -cE '\$\(RUST_AST_PIPELINE\)' rust/Makefile` → **21** dependants. WHERE the omission entered:
  `git log -1 --format=%ad --date=short 36678059` → **2026-02-19** wrote the single-feature rule, and
  `git show 36678059:rust/Cargo.toml | grep -c ebnf_dual_run` → **0** — the feature did not exist
  until `de8ca8bc`, **2026-02-20**. ⇒ no rationale, no decision; the rule predates the feature by one
  day and was never revisited. (`fc63a64f`, 2026-03-23, then fixed the collision for exactly one
  consumer by isolating it in `target/ebnf_frontend_build`, leaving the general case.)
- [x] **FIX** — ops/build-flow tier, and the LOWEST tier available for slice 1: a standalone refusal
  (`scripts/require_ast_pipeline_features.sh`) that touches no build recipe and no gate, so it cannot
  regress anything while the design question (make the canonical rule dual-feature? separate the
  paths?) is still unpriced. ⛔ Deliberately NOT the higher-leverage fix — that is slice 2.
- [x] **ADDRESSED (verified)** — `bash -n` clean; `--self-test` reports **4/4 controls** and is
  two-sided by construction (`self-test NEGATIVE control: pass (ebnf_dual_run=false is refused)` —
  the guard is observed REFUSING the exact defect, not merely accepting a good binary); against the
  real binary, `AST-PIPELINE-FEATURE-GUARD: ok — 'rust/target/debug/ast_pipeline' carries
  [generated_parsers ebnf_dual_run]`. ⛔ Honest bound: `shellcheck` is **not installed on this host**,
  so only `bash -n` ran.
- [x] **NO REGRESSION** — the change is **one new file**, referenced by nothing: no Makefile, gate
  script, workflow or source file was edited (`git show --stat` for this commit is the proof), so no
  existing flow can behave differently. All 18 doctrines pass with it staged.
- [x] **LOCKSTEP** — this leaf (the archaeology, the three-way census, the slice-1/slice-2 split, and
  the stated GATE-REACHABILITY limitation); `TOOLBOX.md` §1.4 gains the guard beside the
  `--report-feature-surface` trap it generalizes. No book/contract/register/release move: an
  internal build-flow helper changes no user-facing surface and no family status.

#### Acceptance Checklist (enforced) — SLICE 2

- [x] **REPRODUCE / ISSUE** — the defect is triggered by a SOURCE CHANGE, which is why it hid: make
  compares mtimes, so `focus_*` skips the rebuild on a warm tree and only downgrades once a source is
  newer. Reproduced deterministically with `touch rust/src/ast_pipeline/mod.rs && make -C rust
  focus_scratch` → pre-change `AST-PIPELINE-FEATURE-SURFACE: ebnf_dual_run=false generated_parsers=true`.
- [x] **ROOT CAUSE (WHY + WHERE)** — WHERE, located by `git log -S` on the exact recipe string:
  `git log --oneline -S 'cargo build --features generated_parsers --bin ast_pipeline' -- rust/Makefile`
  → a SINGLE commit, `36678059` *"Wire raw AST annotation handling end-to-end in non-bootstrap
  pipeline"* (2026-02-19) — so `$(RUST_AST_PIPELINE)`'s feature set has never been revisited since the
  day it was written. WHY it is wrong: `git show 36678059:rust/Cargo.toml | grep -c ebnf_dual_run` →
  **0**, i.e. the feature did not exist until `de8ca8bc` (2026-02-20), one day later. 21 targets
  depend on the rule (`grep -cE '\$\(RUST_AST_PIPELINE\)' rust/Makefile`), so each of them rebuilt
  an under-featured binary whenever a source changed. Confirmed at the recipe level with
  `make -C rust -n SHELL=/bin/bash focus_scratch | grep 'cargo build.*ast_pipeline'`, which before
  this slice emitted `cargo build --features generated_parsers --bin ast_pipeline` and now emits
  `cargo build --features "generated_parsers ebnf_dual_run" --bin ast_pipeline`.
- [x] **FIX** — ops/build-flow tier, ONE line: the canonical rule declares
  `--features "generated_parsers ebnf_dual_run"`. Chosen over the guard-everywhere option because it
  removes the class instead of policing it, and over path-separation because a single owned path is
  simpler than two paths whose divergence must itself be policed.
- [x] **ADDRESSED (verified)** — `touch rust/src/ast_pipeline/mod.rs && make -C rust focus_scratch`
  now yields `ebnf_dual_run=true generated_parsers=true`, and
  `scripts/require_ast_pipeline_features.sh rust/target/debug/ast_pipeline generated_parsers
  ebnf_dual_run` → `ok`. End-to-end on the ORIGINAL symptom: the cert sweep re-run immediately after
  `focus_scratch` returns real `CERTIFICATE-COVERAGE:` rows where it previously returned **18 empty
  rows per side**. (12 rows are still empty — `ebnf`/`return_annotation`/`semantic_annotation`/
  `rtl_const_expr` × 3 seeds have no registered parser on this path; that is the pre-existing
  structural fact, unchanged and stated rather than counted as a win.)
- [x] **NO REGRESSION — and the parser-RUNTIME question answered by construction, not by assertion.**
  ⭐ The director's constraint was explicit: the regex parser's sub-1 µs runtime must not move.
  (a) `ebnf_dual_run` has **ZERO** `cfg(feature)` sites in the parser codegen
  (`ast_based_generator.rs`, `ast_pipeline/mod.rs`); every site in the generator crate is on a
  `#[test]`. (b) Empirically: `generated/regex_parser.rs` regenerated by a **single**-feature and a
  **dual**-feature generator is BYTE-IDENTICAL, md5 `943829436b0d628fa8ab21d12f0962c0` both ways;
  `generated/scratch_parser.rs` md5 `fc5a61e8ae1d5f88aff0c4a4d7f847f9`, unchanged from before this
  slice. (c) Release runtime binaries (`parseability_probe`, `regex_perf_probe`) are **separate cargo
  invocations** with their own `--features`, and Cargo features are per-invocation — a debug
  generator's feature set cannot reach them. ⇒ **no parser runtime can be affected by this rule.**
  Cross-grammar cert sweep after the change is `diff`-IDENTICAL to the verified baseline. All 18
  doctrines pass.
- [x] **COST, measured rather than waved through** — generation-time only: `focus_scratch` with a
  forced `ast_pipeline` rebuild took **55 s** dual-feature (the single-feature build of the same
  binary measured 42 s earlier in the session), i.e. roughly +15-30 s per rebuild of the generator,
  paid only when a source changes. Director's ruling recorded above.
- [x] **LOCKSTEP** — this leaf; `rust/Makefile` carries the rationale at the rule itself (so the next
  reader cannot repeat the 2026-02-19 omission); `TOOLBOX.md` 1.4 updated — the `focus_*` half of the
  #140-class trap is CLOSED, the bootstrap half is not, and it says so. `CHANGES.md`. No book,
  contract, register, release or schema move: an internal build-flow rule changes no user-facing
  surface, no family status, and no shipped byte.

### `.25` — the mandated memory guard reports `exit=0` for any command that fails behind a PIPE, so a heavy job's failure publishes as a green (`todo` — ROUTED IN from `ENGINE-UNIVERSAL-SERVICES.17` slice 1, 2026-08-13 session #225; ⛔ PARKED behind the SV lane lock)

**Why it is here and not in the SV tree.** `scripts/run_with_memory_guard.sh` is the wrapper this
repository *mandates* for every heavy job (`README.md`, `MEMORY.md`, the gate recipes). A green it
publishes is read as "the work succeeded", by a human and by any instrument scraping its line. That
makes it proof-surface machinery, which is this tree's subject — not SystemVerilog's.

#### ROUTING EVIDENCE

⛔ **It reproduces outside the family it came from, and outside any grammar at all** — the two-sided
control uses no parser, no grammar and no corpus:

```text
# CONTROL A — plain failing command: the guard reports the failure faithfully
$ scripts/run_with_memory_guard.sh --budget-mb 1024 --timeout-s 30 -- bash -c 'exit 7'
memory-guard: completed exit=7 peak_tree_rss=0MB elapsed=5s

# CONTROL B — the SAME failure behind a pipe: the guard reports SUCCESS
$ scripts/run_with_memory_guard.sh --budget-mb 1024 --timeout-s 30 -- bash -c 'exit 7 | tail -1'
memory-guard: completed exit=0 peak_tree_rss=0MB elapsed=5s
```

- **The mechanism is POSIX, not a guard bug.** A pipeline's status is its *last* command's, so the
  guard faithfully reports what the shell handed it. ⇒ the defect is that the guard's contract does
  not defend against its most common invocation shape; it is not that the guard miscounts.
- **It is not hypothetical — it fired in the session that found it.** `.17` slice 1 ran the two
  scratch-slot pair gates as `cargo test … | tail -25`; cargo failed with
  `error: unexpected argument 'scratch_slot' found`, the guard printed `completed exit=0`, and the
  measurement would have been recorded as a PASS had the log not been read line by line. The gates
  were re-run correctly and did pass — so nothing false shipped — but that was a reading habit, not
  a mechanism.
- **The class is already this tree's.** `.24` records two empty result sets diffing clean; the same
  failure direction (silent, in the PASSING direction, nothing on stdout to notice). ⇒ same family,
  and the remedy is likely shared.

**Acceptance:** (a) the guard refuses or loudly flags a masked pipeline status — candidates to price:
run the command under `set -o pipefail`, or detect a pipe in the payload and warn, or report
`PIPESTATUS` alongside the final status; (b) a self-test proving it fires, in **both** directions (a
piped failure is caught AND a piped success is not flagged), per
[[feedback_instrument_needs_ground_truth]]; (c) a sweep of tracked callers that pipe into the guard,
since each is a published green of unknown truth.

⛔ **Do NOT "fix" this by telling authors to stop piping.** That is a discipline, and
`DOCTRINE_ENFORCEMENT.md` is explicit that a doctrine which is not mechanically checked is a
suggestion. The instrument has to defend its own contract.

### `.26` — `DESIGN-PRIOR-ART` cannot fire on a design leaf that names no new directive token, which is exactly the leaf that reasoned from an unmeasured engine premise (`todo` — ROUTED IN from `ENGINE-UNIVERSAL-SERVICES.17` slice 1, 2026-08-13 session #225; ⛔ PARKED behind the SV lane lock)

#### ROUTING EVIDENCE

- **Measured on the real commit, not a synthetic.** `PGEN-ENGINE-UNIVERSAL-SERVICES-0019` is a
  DOCS-only design note that chose between two engine designs. Novel backticked at-prefixed tokens
  on its added lines: **0** (`git show e56e3eff | grep '^+' | grep -oE '`@[a-z_][a-z0-9_]*'`). The
  checker's candidate set is exactly those tokens (`scripts/check_design_prior_art.sh:58-62`) and it
  `continue`s when the set is empty ⇒ **the gate could not have fired**, and did not.
- **What that cost.** The note asserted two engine properties as the premises for choosing between a
  re-enterable `*` and a named refusal — *"PEG's ordered choice commits"* and *"[re-entry]
  reintroduces exactly the backtracking PEG removed"*. Both are false of PGEN, which runs a
  give-back `longest_match` tournament at every non-degenerate choice
  (`ast_based_generator.rs:4273`). `.17` slice 1 measured it and the front-runner design changed
  tier. One slice of rework, and the wrong design was one session from being built.
- ⛔ **It reproduces outside the family, and outside the annotation surface entirely.** The gate's
  subject is *"proposes a NEW annotation or directive surface"*; the gap is *"reasons about ENGINE
  BEHAVIOUR from a prose premise"*, which any tree can do and which no gate reads. The parent
  discipline already says the words — [[feedback_read_prior_art_before_designing]]: *"when citing
  engine behaviour: **re-measure it.** Do not quote a doc's description of what the engine does."*
  That sentence has **no enforcer**; only the directive-naming half does.

**Acceptance:** (a) decide whether the engine-premise half is mechanizable at all — the honest answer
may be *no*, and `check_design_prior_art.sh`'s own header already concedes the evidence archetype
*"cannot verify the search was thorough"*; (b) if it is, a candidate discriminator to MEASURE against
the tracked corpus before building: a design leaf making a behavioural claim about the engine must
cite a re-runnable command or a `DIAGNOSIS_SIG` token, the same way a code change must; (c) if it is
not, say so in the doctrine's honest-limits section rather than leaving the sentence looking enforced.
⛔ Calibrate against the real corpus first — `check_routing_evidence.sh`'s header records that this
check's own first cut *"would not have caught its own founding incident"*, and a checker that cries
wolf gets bypassed.

⭐ **A precision datapoint arrived by accident while this leaf was being written, and it belongs in
(b)'s calibration.** The first draft's heading contained the literal at-prefixed word *directive* in
backticks, purely as prose describing the gate. `check_design_prior_art.sh` flagged it —
*"novel directive name(s): directive"* — on a leaf proposing no surface at all. That is the
self-referential false-positive class the checker's own header prices and accepts, and it confirms
the discriminator is **lexical**, keyed on a token's spelling rather than on a proposal. Any
engine-premise discriminator built for (b) has to be measured against that same corpus, or it
inherits the same precision.

---

### `.27` — the acceptance gate audits the PRESENCE of proof, never its FALSIFIABILITY, so a check that cannot fail passes it (`todo` — ROUTED IN from `ENGINE-UNIVERSAL-SERVICES.17` slice 2, 2026-08-13 session #226; ⛔ PARKED behind the SV lane lock)

**Why it is here and not in the SV tree.** `TASK-ACCEPTANCE` (`scripts/check_diagnosis_evidence.sh`)
is the gate every code change in this repository passes, and probe banks are the artifact its
`ADDRESSED (verified)` box most often points at. Both are proof-surface machinery — this tree's
subject — and neither is SystemVerilog-specific.

#### ROUTING EVIDENCE

⛔ **It reproduces with no grammar, no parser and no corpus** — the evidence is three facts about the
repository's own enforcement surface, each read directly off the tree:

**1. The gate's archetype is `evidence`, and its contract is EXISTENCE.** `DOCTRINE_ENFORCEMENT.md`
§ archetypes: *"**Evidence (artifact)** — requires a re-checkable artifact for an action that cannot
be re-derived"*, and the checker's own header states the test: each required box must be *"TICKED
(`[x]`) and backed by real tool-output evidence"*, matched against five recognised diagnosis-signature
families. ⇒ it asks **"is there proof?"** and never **"could this proof have come out otherwise?"**

**2. Measured on this very session: four defects, all producing well-formed evidence, none catchable
by that question.** Every one was a claim whose failure mode was *unreachable* —

| # | the defect | why the gate could not see it |
|---|---|---|
| 1 | a safety branch (`routes_truncated` ⇒ refuse) that no input reaches, so it asserts a safety it never demonstrates | the box's evidence was a real, passing run |
| 2 | `render_elements` documented *"Report-only — nothing parses this back"* while `indirect_lr_elimination.rs:483` decides an ambiguity refusal by comparing its output | no check reads a doc comment, and the before/after evidence was genuine |
| 3 | a verdict (`FEASIBLE`) published without the qualifier that made it *sound* rather than *closed* | the numbers were correct |
| 4 | a probe-bank case whose expectation was derived from the value under test (`want = "$sites/$sites"`), so it would have passed on `0/0` | the bank exited 0 and printed a ✅ |

**3. ⭐⭐ THE MECHANISM ALREADY EXISTS, IS ALREADY WIRED TWICE, AND NOTHING ASKS FOR IT — the exact
shape `LESSON-PROMOTION` was created for.** Two scripts expose a falsifiability self-test:

```text
$ grep -l -- "--self-test" scripts/*.sh
scripts/preserve_scratch_probe.sh
scripts/require_ast_pipeline_features.sh

$ for f in $(find docs/tasks/artifacts -name probe.sh); do printf "%s %s\n" "$(grep -c -- --self-test $f)" "$f"; done
0 docs/tasks/artifacts/engine_universal_services/guard_feasibility/probe.sh
0 docs/tasks/artifacts/engine_universal_services/quantifier_policy/probe.sh
0 docs/tasks/artifacts/engine_universal_services/indirect_lr/probe.sh
```

⇒ **0 of 3 tracked probe banks can demonstrate their own falsifiability.** All three had it shown by
HAND and recorded as PROSE in a task leaf (`.17` slice 1: *"a deliberately flipped expectation makes
it exit rc 1"*; slice 2 likewise, twice). Prose in a leaf is not re-runnable, and the next bank
inherits nothing.

⛔ **And the knowledge layer already holds the lesson**:
[[a-check-whose-inputs-all-pass-has-not-been-tested]] is a fact card, retrievable and repeatedly
cited — `.11` slice 1, `.13` slice 5b (a criterion shipped with no test), and four times inside a
single slice here. A lesson cited that often with no enforcer is the definition of this tree's
subject.

#### ⭐⭐ THE ROOT OF IT — the roster has THREE archetypes and needs a FOURTH

`DOCTRINE_ENFORCEMENT.md` classifies every check as **structural** (re-derive an invariant from the
tree), **oracle** (re-execute a deterministic tool and assert the result), or **evidence** (require a
re-checkable artifact). All three answer the same question: *does the invariant hold right now?*

**None of them asks whether the check would NOTICE if it did not.** That is the missing archetype:

> **MUTATION** — break the thing under guard, and assert the guard goes RED.

Every defect in the table above is invisible to the first three archetypes by construction and
visible to the fourth. It is also not a new idea in this repository — it is what `.13` slice 5b did
by hand (reverting `collect_starvation_sites` left `cargo test --lib indirect_lr` at **11 passed / 0
failed**, proving the criterion that had just stopped a measured regression was untested), what `.11`
slice 1 did by hand, and what `.17` slices 1 and 2 each did by hand. Four hand-rolled mutation
proofs, none re-runnable, no shared mechanism.

#### The proposal, in tiers — cheapest first, and NOT all of it is mechanizable

- **T1 — static, ~20 lines.** In a tracked probe bank, an expectation argument must be a LITERAL: no
  `$` in the `want` slot of a `check`-style helper. Catches defect 4 exactly and by construction.
- **T2 — oracle, the load-bearing one.** Every tracked bank must expose `--self-test`, which flips
  ONE declared expectation and asserts the bank exits non-zero **naming that case**; the doctrine
  driver runs it. Turns "I flipped it by hand once" into a re-runnable artifact, and makes the
  falsifiability claim survive the session that made it.
- **T3 — structural.** A display-only renderer must be unreachable from the generation path, enforced
  by module placement rather than by a doc comment nothing reads. Catches defect 2; note the doc
  comment was *false for months* and no gate could have known.
- ⭐⭐ **T5 — DIFF-SCOPED MUTATION TESTING. The one with the widest reach, and the only tier that
  generalises past the four defects that prompted this leaf.** Run an off-the-shelf mutant harness
  (`cargo-mutants`) restricted to the files the commit changed, and require every new or changed
  function to have at least one mutant KILLED by the tests the commit ships. It mechanises exactly
  the hand proof this repository already performs and then throws away. Reach, measured against
  history rather than asserted: it catches defect 1 (the mutant *"delete the `routes_truncated`
  poison"* survives), and it catches **`.13` slice 5b's shipped defect retroactively** — the mutant
  *"revert `collect_starvation_sites` to the direct-holder scan"* survived 11 green tests, which is
  precisely how that criterion shipped untested. ⛔ Price it honestly before adopting: mutation runs
  are minutes-to-hours, so it belongs on a diff scope with a per-commit budget, never on the whole
  crate.
- **T6 — diff-scoped BRANCH COVERAGE floor.** The cheap approximation of T5: a branch added by the
  commit must be *executed* by the suite (`cargo-llvm-cov`, changed files only). Weaker than mutation
  — executed is not the same as tested — but it catches the whole "unreachable safe branch" family
  (defect 1) for a fraction of the runtime.
- **T7 — a comparison-based gate must assert its operands are NON-EMPTY.** Generalises
  `CI-PARITY-GATE-ROT.24` from one incident into a rule: *two empty result sets diff clean*, so any
  gate whose verdict is a `diff`/equality over gathered rows must first assert both sides are
  non-empty. `.24` fixed one instance (`require_ast_pipeline_features.sh`); nothing stops the next.
- **T8 — extend `reverify:` from knowledge cards to acceptance-box metrics.** Every fact card already
  carries a re-runnable `reverify:` command in its front-matter, and the KNOWLEDGE-MAP doctrine keeps
  it honest. Acceptance boxes quote numbers with no such handle, so a metric goes stale silently —
  which is how a slice-4 census (*"wrapper 23/18/11/7"*) was still being read as current two slices
  later. ⭐ Another mechanism that exists, is wired, and is not asked for one layer over.
- ⛔ **T4 — NOT mechanizable, and saying so is part of the finding.** Defect 3 — a verdict published
  without the qualifier that distinguishes *sound* from *closed* — is a judgement about whether a
  headline overstates. The narrow mechanizable residue is weaker: an instrument computing an
  over-approximation should surface the approximation flag in its own output (which `.17` slice 2 now
  does, via `~`). A gate that claimed to catch T4 in general would itself be an unfalsifiable check —
  the very thing this leaf exists to stop.

##### ⛔ ROUTED IN — a SIBLING gap measured the same way, and it is REACHABILITY not falsifiability (`ENGINE-UNIVERSAL-SERVICES.17` slice 4b, 2026-08-14 session #228)

This leaf asks *"would this check notice if the invariant did not hold?"*. Slice 4b measured the
question one step earlier: **does anything RUN the check at all?**

- **Measured:** `grep -rn` over `scripts/`, `rust/Makefile` and `.github/` for the four tracked probe
  banks — `quantifier_policy`, `guard_feasibility`, `guard_dry_run`, `guard_effectiveness` —
  returns **zero** invocations. Every one is operator-invoked only. Between them they carry the
  measured laws `.17`'s entire design rests on, including the 37 rows that decide whether the
  shipped repair regresses `initial k = int'(1);`.
- ⛔ **The `GATE-REACHABILITY` doctrine passes**, and correctly: it binds *tracked gate targets*, and
  a probe bank is not registered as one. So the roster has an exact statement of this principle —
  *"a check nothing invokes is indistinguishable from one that does not exist"* — and a population
  it provably does not reach.
- ⭐ **Why it belongs here and not in its own leaf:** `.27`'s ladder is about the QUALITY of a check's
  assertion; this is about whether the assertion is ever evaluated. They fail in the same direction
  (silently, in the passing sense) and a bank fixed for one and not the other is still worthless —
  a falsifiable bank nobody runs, or a scheduled bank that cannot fail. ⇒ **fold into this leaf's
  acceptance rather than open a fifth parked governance leaf**
  ([[feedback_prefer_feature_work_over_governance_lanes]]).
- ⚠️ **Honest bound:** the fix is not "run all four in CI" — the `guard_effectiveness` GEN arm costs
  ~11 min and ten `focus_scratch` cycles. The disposition has to be per-bank and explicit
  (cheap interpreter-only arm in the automatic tier, expensive GEN arm operator-invoked with a
  recorded reason), which is exactly the *"or carries a deliberate disposition"* half of
  `GATE-REACHABILITY` that no probe bank has ever been asked for.

⇒ acceptance (d) below.

**Acceptance:** (d) every tracked probe bank is either invoked by something that RUNS or carries a
recorded per-bank disposition naming why not and what does cover it — measured against the four that
exist today, all currently uninvoked. (a) T1 + T2 implemented and registered in the doctrine roster, with the driver
running every tracked bank's self-test; (b) each existing bank retrofitted, and RED-proven — the
retrofit is worthless unless a deliberately broken bank is shown to fail its own self-test; (c) T5
priced against a real commit's diff (runtime + kill-rate on the `.13`/`.17` history, where the answer
is already known) and adopted or refused with that number; (d) T3, T6, T7, T8 priced separately —
each is a distinct archetype, not a variant of the others; (e) **the MUTATION archetype added to
`DOCTRINE_ENFORCEMENT.md`'s archetype table**, since a roster that cannot name the kind of proof it
lacks cannot notice it is missing; (f) T4's limit recorded there too, so the roster states what it
does NOT prove.

⭐ **Ordering note for whoever takes this:** T1 and T2 are hours and close the incident. **T5 is the
one that changes the class** — it is the only tier that would have caught a defect nobody had thought
to look for, which is the whole complaint. Do not let the cheap tiers close the leaf.

⛔ **Sequencing.** PARKED behind the SV lane lock with `.25` and `.26` — it blocks no SV release
work. ⭐ But note the standing bar it touches: the `ADDRESSED (verified)` box is the one every SV
slice leans on, so this is the gate whose blind spot is most widely exercised.

### `.29` NEW `todo` — a probe bank's PROSE numbers and its report SCRAPERS both rot silently, and nothing measures either (opened 2026-08-14 session #229 by `ENGINE-UNIVERSAL-SERVICES.17` slices 5 + 6; ⛔ PARKED behind the SV lane lock)

**ROUTING EVIDENCE** (`ROUTING-EVIDENCE` doctrine — what was MEASURED before routing, and whether it
reproduces outside the family it is being sent to):

- **Two instances, one week apart, in the same bank family, each found by accident.**
  - `.17` slice 5: `guard_feasibility` C7 pinned *"157 of 157 sites over-approximated"*. The
    extraction was `grep -c 'first='`, which counts LINES containing that substring — and the
    per-candidate summary line `guard: … suffix_first=…` carries it too. The real site count is
    **129**; `157` was 129 sites + 28 candidate rows. It surfaced only because slice 5 added a
    `seed_first=` line and the count jumped to 185.
  - `.17` slice 6: `guard_effectiveness`'s header prose claimed *"19 cases must ACCEPT and 18 must
    REJECT"* for a bank that was **21/16**, and its README claimed **8/13** for an even earlier
    shape. Its README also pointed at *"leaf `.19`"*, which does not exist — slice 4's decision (d)
    explicitly refused to open it.
  - ⭐⭐ **A THIRD instance, one day later, and it is the sharpest of the three** (`.17` slice 7,
    2026-08-14 session #230). `guard_dry_run`'s new rows D9/D10 pin what the guard planner EMITS,
    and the report they scrape built its `[positions]` string from the plan's own booleans rather
    than from the grammar `apply_plan` wrote. Planting the exact defect the row exists to catch —
    delete the trailing-lookahead emission — left the bank **`GUARD-DRY-RUN: 14/14` GREEN** while
    describing a lookahead the emitted grammar no longer carried. ⇒ **a bank can be falsifiable in
    form and blind in fact, when its input narrates a decision instead of measuring an artifact.**
    Fixed in place by deriving each position from whether the rule that should carry it ends in a
    `Lookahead`; the same plant now flips D10 by name.
    ⛔ It also names a SECOND mechanizable half this leaf did not have: **the falsifiability plant
    was run against `cargo test` and not against the bank**, and only the CLI-level plant separated
    them. A plant that never rebuilds the binary measures the assertion layer and leaves every
    report-scraping row untested.
  - ⭐⭐ **A FOURTH instance the very next day, and it is the cleanest EVIDENCE FOR (b)** (`.17`
    slice 8, 2026-08-14 session #231). The new `guard_parses` bank scrapes the emitted parser for
    guard rule functions, and its first anchors were `^\s*fn parse_…` while the codegen emits
    **`pub fn parse_…`**. They matched nothing and the run reported `guard_rule_count=0` on the arm
    that has three of them. ⛔ **It was caught on the FIRST run and cost nothing**, and the reason is
    the whole of (b)'s value: the row **DECLARES 3**, so a scraper that finds nothing FAILS instead
    of printing a confident zero. ⇒ the transferable rule is narrower and stronger than *"anchor on
    structure"*: **a structural scraper needs a declared non-zero expectation, because "found
    nothing" and "there is nothing" are the same output.** A bank whose structural rows merely PRINT
    what they found cannot distinguish a broken anchor from a true absence, and is exactly the class
    of row nobody re-reads.
    ⛔ The same run carried a second, purely mechanical instance: `grep -c … || echo 0` appends a
    SECOND zero line, because `grep -c` prints its own zero AND exits 1. Every comparison then ran
    against a two-line value and failed for the wrong reason — which is the benign direction of the
    same defect, and the only reason it was legible.
- **All three classes fail in the FLATTERING direction, which is why no one noticed.** An inflated
  denominator makes an `N of N` ratio read as broader coverage; a stale prose count reads as a
  bank that is bigger and better-balanced than it is; a plan-derived report reads as an emission
  that happened. None breaks a row, so a green run is indistinguishable from a correct one.
- **It reproduces outside `ENGINE-UNIVERSAL-SERVICES` by construction**, because the cause is
  structural, not family-specific: (a) any bank that scrapes a human-readable report by SUBSTRING is
  coupled to every line that report will ever grow, and (b) any bank that states a total in prose
  states something no code derives. The repository has **five** tracked banks under
  `docs/tasks/artifacts/engine_universal_services/` alone and many more elsewhere; the two audited
  here were the two that happened to be extended.
- ⛔ **The existing roster is blind to both.** `GATE-REACHABILITY` asks whether a check is INVOKED,
  `.27` asks whether it is FALSIFIABLE, `KNOWLEDGE-MAP` re-derives from record files, and
  `LIVE-DOC-CURRENCY` watches declared `Last updated:` stamps. None asks whether a bank's own
  arithmetic describes the bank, or whether its extractor anchors on the row it means.

**Acceptance:** (a) a check that a tracked probe bank states no total its own run does not DERIVE —
the two banks fixed by hand in `.17` slice 6 are the calibration pair, and it must be RED against the
tree at `d4e6cca9` and GREEN after; (b) a check, or a documented refusal with a reason, that a bank's
report extraction anchors on a structural feature of the row (a bracket group, a marker in its only
legal position) rather than on a bare substring — ⛔ this half may not be mechanizable cheaply, and
saying so with evidence is an acceptable outcome — ⭐ and `.17` slice 8 supplies (b) with its
cheapest partial answer, which may be all of (b) that is worth mechanizing: **a structural row must
declare a non-zero expectation**, so a broken anchor fails instead of reporting a plausible zero.
That IS checkable structurally (a bank whose every structural expectation is `0` is unfalsifiable),
unlike "is this substring the right anchor", which is not; (c) a sweep for DANGLING task-leaf pointers
(`.19`'s class) across `docs/tasks/**`, which is the same shape as `.28`'s stale-id sweep and should
share its implementation; **(d) NEW (`.17` slice 7) — the DERIVED-FROM-THE-ARTIFACT half**: a check,
or a documented refusal with a reason, that a value a bank pins is computed from the thing it names
rather than from a decision about it. ⛔ The calibration pair is `GuardChain::summary` plan-derived vs
tree-derived, and it ships as a TRACKED PATCH —
`docs/tasks/artifacts/engine_universal_services/guard_dry_run/plan_derived_summary.patch` — because
the defective version was **never committed**. It was found and fixed inside one slice, so
`git log -S "self.loop_guard, self.trailing_guard"` returns nothing, and this criterion's first draft
cited a "RED/GREEN pair in git" that no implementer could have found. ⇒ a gate for (d) must be **RED
with the patch applied and GREEN without it**; the patch header carries the measured 2×2 and the
exact commands.

⛔⛔ **AND THAT IS A ROUTING FINDING IN ITS OWN RIGHT, one level up from this leaf's other three.** A
defect found and fixed within a single slice leaves **no reproducer behind** — the acceptance box
records that it happened, and the artifact that would let a future gate be calibrated against it is
gone by the time the commit lands. Every earlier instance in this family (`.17` slice 5's `157`,
slice 6b's two totals, slice 7's plan-derived summary) has the same hole. ⇒ **acceptance (e): when a
slice finds and fixes an instrument defect before committing, the pre-fix state is preserved as a
tracked patch or the leaf states why it is not worth preserving.** Cheap — one `git diff` before the
fix — and it is the difference between a gate that can be calibrated and one that has to be argued
about.

⭐ **The durable half is already promoted** —
[[a-report-scraper-must-anchor-on-structure-not-on-a-substring]] carries the discipline and its
`reverify:` command. This leaf owns turning it into a GATE, because a card is retrievable and a gate
is unavoidable.

### `.28` NEW `todo` — superseding a decision record leaves its old id cited as LIVE, and no doctrine measures that (opened 2026-08-14 session #228 by `ENGINE-UNIVERSAL-SERVICES.17` slice 4b)

**ROUTING EVIDENCE** (`ROUTING-EVIDENCE` doctrine — what was MEASURED before routing, and whether it
reproduces outside the family it is sent to).

- **It reproduces here, at full size, one commit after the supersede.** `.17` slice 4 superseded
  [[project_pgen_gives_back_at_the_choice_but_not_at_the_quantifier]], updated the surfaces its
  author recalled, and passed **18/18 doctrines** plus `mdbook_docs_gate`. A `grep -rl` for the
  refuted id afterwards returned **12** files, of which **2** still presented the law as live —
  including `rust/src/ast_pipeline/indirect_lr_plan.rs:323`, a module doc comment that *specified
  the superseded design*, i.e. the next implementer reading the code rather than the leaf would have
  built the shape measured to regress `initial k = int'(1);`.
- **It is not family-specific and not `.17`-specific.** The mechanism is that `MEMORY_ARCHITECTURE.md`
  mandates *"supersede, don't mutate"* and supplies no instrument for the second half: nothing
  enumerates who still cites the superseded id. Any record superseded by any tree has the same
  exposure; `docs/decisions/` currently holds several.
- ⛔ **The nearest existing doctrine is measured NOT to cover it.** `KNOWLEDGE-MAP` verifies the
  derived map is in sync with its **fact sources** — it re-derives from the record files, so a stale
  *citation elsewhere* is invisible to it. It passed with both defects present, before and after
  regeneration. This is structurally the same gap `LESSON-PROMOTION` was created to close (a
  mechanism existed, was wired, and nothing asked whether it had been used).
- ⭐ **The check is cheap and the hard part is the classification, not the grep.** Of the 12
  citations, **10 are correct and must stay**: derived artifacts (`KNOWLEDGE_MAP.md`), append-only
  history that must quote the name it was written with (`CHANGES.md`, `docs/TASK_TREE.md`), files
  carrying the correction *beside* the original (both `quantifier_policy` files, the owning leaf),
  and the two records naming each other. ⇒ a blanket rewrite would do HARM
  ([[feedback_a_named_call_site_is_a_category_of_call_sites]] — classify by what the referent
  REQUIRES, per [[feedback_classify_referents_by_requirement]]).

**Acceptance:** (a) a record carrying `superseded_by:` is machine-detectable, and every citation of
its id outside an allow-classified set (derived / append-only history / the superseding record /
a citation adjacent to an explicit supersede marker) FAILS the gate by name; (b) the check is RED
against the tree as it stood at `96b0d60a` — the two real stale citations — and GREEN after the
4b fix, so it is proven to discriminate rather than merely to pass; (c) the classification rule is
written down where the next supersede will read it, in `MEMORY_ARCHITECTURE.md` beside
*"supersede, don't mutate"*: **re-point what SPECIFIES, mark what NARRATES, never touch history**.

⛔ **Sequencing.** PARKED behind the SV lane lock with `.25`, `.26` and `.27` — it blocks no SV
release work, and `.17` slice 4b already fixed the live instance by hand. ⭐ Note what it shares with
`.27`: both are gaps where the roster audits the PRESENCE of a structure and never whether the
structure was actually reached.

### `.31` ✅ `done` — the SHIPPING generation recipe hardcoded `--debug --trace`, so every parser build emitted a full trace: 6.89 GB per regeneration locally and the same volume streamed into CI (opened 2026-08-14 session #232 by `ENGINE-UNIVERSAL-SERVICES.17` slice 9; ⛔ **DIRECTOR-RULED 2026-08-14**; CLOSED 2026-08-14 session #233, `PGEN-CI-PARITY-GATE-ROT-0029`)

⛔⛔ **DIRECTOR RULING, VERBATIM (2026-08-14): _"remove `--debug --trace` for CI streams. Because
these 2 options are only for debug, tracing purposes. Again, they make sense only during debug
runs."_ and _"CI are not debug runs, especially if each run dumps 17GB, that's way too much."_**
⇒ this is not a proposal to price; it is a decision to implement. The leaf exists to make it
task-tree-owned before the code moves, per `COMMIT.md`'s Code-Change Doctrine.

**ROUTING EVIDENCE** (`ROUTING-EVIDENCE` doctrine — what was MEASURED before routing):

- **Measured, in this session, by accident.** `rust/Makefile:93` defines
  `RUST_GENERATOR = $(RUST_AST_PIPELINE) --generate-parser --debug --trace --eliminate-left-recursion`,
  and every `focus_<family>` / `regenerate_generated_parsers` invocation inherits it. Three
  regenerations produced **17 GB** of captured log — `regen.log` 5.7 GB, `regen_final.log` 6.4 GB,
  `narrow_build.log` 5.0 GB — for artifacts that are byte-identical with or without the flags.
- **CI runs the identical recipe.** `.github/actions/regenerate-parsers/action.yml` is a bare
  `make -C rust SHELL=/bin/bash regenerate_generated_parsers` with no redirection, and
  **11 of 15 tracked workflows use that action** (`.4`'s own measurement). So the same multi-GB
  stream goes into the Actions log on every one of them.
- **The blast radius is a LOG surface, not a disk one, and saying which matters.** GitHub streams
  step output rather than storing it on the runner's disk, so this is not a runner
  disk-exhaustion bug; it is a log-volume bug — a step whose output is truncated is a step whose
  failure evidence may not survive, which is this tree's entire subject.
- ⛔ **It also SLOWS the build**, measurably: writing ~6 GB per regeneration is I/O the artifact does
  not need. Not separately quantified here; the fix removes it either way.
- **The flags are debug affordances on the SHIPPING path**, which is the defect in one sentence: the
  recipe every deliverable parser is generated by asks the generator to narrate itself.

**Acceptance:** (a) remove `--debug --trace` from `RUST_GENERATOR` (and audit `RUST_GENERATOR_BOOTSTRAP`
and any sibling that inherited them) so the shipping path is quiet by default; (b) ⛔ **PROVE the
artifacts are unchanged** — regenerate all 11 parsers and assert `shasum -a 256 generated/*.rs` is
byte-identical to the pre-change snapshot. If any byte moves, `--debug`/`--trace` were affecting
CODEGEN and not just output, which would be a far larger finding and would immediately become this
leaf's subject (⭐ and would be a live hypothesis for `ENGINE-UNIVERSAL-SERVICES.19`'s unexplained
99 747 bytes — the two leaves should read each other); (c) keep a documented way to get the trace
back for actual debugging (an opt-in variable, e.g. `PGEN_GENERATOR_TRACE=1`), because deleting the
capability is not the ruling — moving it off the default path is; (d) measure the before→after log
volume of one full regeneration and record both numbers, so the fix has a size rather than an
adjective.

---

#### ✅ CLOSED 2026-08-14 (session #233) — all four acceptance clauses met

Full census, every figure derived this session:
`docs/tasks/artifacts/ci_parity_gate_rot/generator_trace_volume.txt`.

**(a) THE FLAGS ARE OFF THE SHIPPING PATH — and the audit found a THIRD site the leaf had not
named.** The routing evidence named `RUST_GENERATOR` (`rust/Makefile:93`); auditing
`RUST_GENERATOR_BOOTSTRAP` as instructed turned up `:94` carrying `--debug`, and one further
**inline** invocation at `:861` — the cold-clone `generated/ebnf.rs` seed, which is not reached by
any `focus_*` target and therefore could not have been found by measuring a regeneration. It runs
**only** when `generated/ebnf.rs` is absent, i.e. exactly on the fresh checkout CI gives itself. All
three are now quiet.

**(b) THE ARTIFACTS DID NOT MOVE — 33 of 33 byte-identical, against a determinism CONTROL.**
`shasum -a 256 generated/*.rs generated/*.json` (11 parsers + 22 JSON artifacts) at the baseline, after
a regeneration **with** the flags, and after one **without**: all three identical. ⭐ The middle
comparison is the control that makes the third mean something — without it, "identical" cannot be
distinguished from a codegen that is merely stable, nor "different" from nondeterminism.

⛔ **And the leaf's own hypothesis is REFUTED, which is worth saying plainly.** The acceptance text
warned that a byte moving here would be a far larger finding and *"a live hypothesis for
`ENGINE-UNIVERSAL-SERVICES.19`'s unexplained 99 747 bytes"*. No byte moved. **`.19` must look
elsewhere** — the two leaves were told to read each other, so this is that answer, delivered rather
than left for `.19` to re-derive.

⭐⭐ **PRECISION ADDED 2026-08-14, same day, after re-reading `.19` — the conclusion stands, the
REASON above was not the one that does the work, and saying which leg carries a refutation is the
difference between a result and an assertion.** `.19`'s two artifacts (`dae09343…` 131 542 908 B and
`4330ff8e…` 131 642 655 B) were **BOTH generated with `--debug --trace`** — the flags are constant
across its comparison. So flag *presence* was never a candidate explanation for its gap; the only way
these flags could have produced it is by making codegen **NON-DETERMINISTIC**. ⇒ the leg that
actually refutes it is not the with/without comparison but the **determinism control**: a
regeneration with the flags still on reproduced the baseline byte-for-byte across all 33 artifacts.
⛔ Two bounds stated rather than left implicit: (i) this was measured on the SHIPPED admission, not
under `--indirect-lr-admit-starvation-safe-only`, and it generalises only because flag-handling is a
property of the logging path (`resolve_trace_verbosity`) and not of the admission policy; (ii) **both
of `.19`'s own named hypotheses are UNTOUCHED** — (a) the slice-9 diff perturbing SV codegen under
the narrow admission, and (b) `generated/systemverilog.json` having silently moved. `.19` is narrowed
by exactly one candidate, and that candidate was mine, not one it had listed.

The structural half of *why* it cannot move a byte, since a null result deserves a mechanism:

```text
$ grep -rn 'config\.debug\|config\.trace[^_]' rust/src/
rust/src/main.rs:1097:    config.debug = args.debug || trace_verbosity >= TraceVerbosity::High;
rust/src/main.rs:1098:    config.trace = args.trace || trace_verbosity >= TraceVerbosity::Debug;
```

Two writes, **zero reads**. The flags' only live path is
`resolve_trace_verbosity()` → `set_global_trace_verbosity()` (`rust/src/main.rs:936-938`), which
gates the `pgen_trace!` macros and nothing else. ⇒ `--trace`'s own CLI help, *"Enable trace mode in
generated parser (detailed debug logging)"* (`rust/src/main.rs:441-443`), **describes a wire that is
not connected**: the `trace_enabled()` gates inside a generated parser are emitted unconditionally
and consult the parser's RUNTIME verbosity, never the generator's.

**(c) THE TRACE IS OPT-IN — and NO NEW KNOB WAS INVENTED.** The acceptance suggested
`PGEN_GENERATOR_TRACE=1`. Prior art refuses it (`DESIGN-PRIOR-ART`): `resolve_trace_verbosity`
already consults `PGEN_TRACE_VERBOSITY` / `PGEN_VERBOSITY` whenever no `--verbosity` is passed, and
no recipe in `rust/Makefile` passes one — so the engine's documented knob (`TOOLBOX.md` §2.1) *is*
the opt-in, and it is strictly more expressive than a boolean. Measured on `focus_json`, warm:

| invocation | log bytes |
|---|---|
| default (quiet) | **366** |
| `PGEN_TRACE_VERBOSITY=high make -C rust focus_json` | **16 003 019** — the old `RUST_GENERATOR_BOOTSTRAP` level |
| `PGEN_TRACE_VERBOSITY=debug make -C rust focus_json` | **21 955 687** — the old `RUST_GENERATOR` level |

against **21 955 519 B** measured direct from `--debug --trace` on the same input. The capability is
intact to within the make wrapper's own echo lines.

**(d) THE SIZE.** One full regeneration (annotation pair + 7 families), same harness both sides:

| | log bytes | wall |
|---|---|---|
| BEFORE (`0994c3c0`) | **6 894 576 244** (6.89 GB) | 154 s |
| AFTER | **14 194 408** (14.19 MB) | 131 s |

**485.7× smaller, 14.9 % faster.** ⚠️ Stated honestly: **14 189 570 of the after total is ONE
`ast_pipeline` cargo rebuild**, not generator output — both sides paid it, and it lands in the
`focus_json` row only because that is the first family target after the annotation pair is rewritten.
Net of it the generator's own share is **~6.88 GB → 4 838 B**; `focus_json` re-run warm emits 366 B.
The 485.7× figure is the one measured end-to-end on both sides and is the one to quote.

⛔ **The measured sequence is `regenerate_generated_parsers` MINUS its `regex_parser_bootstrap`
step**, whose unconditional `cargo build` downgrades `rust/target/debug/ast_pipeline` and makes the
documented recipe fail in 15 s on any warm tree. That is **`.30`**, still open, not fixed here —
stated as a deviation rather than left for a reader to notice, exactly as `.17` slice 8 did.

#### ⭐ THE RATCHET — `FLOW-INTEGRITY` gains invariant (10)

A director ruling that lives only in a diff is one revert from being undone, and this tree exists
because repairs rot. `scripts/check_flow_integrity.sh` now fails any tracked Makefile line that
invokes `--generate-parser` while carrying `--debug` or `--trace`. Replayed against the pre-fix tree
it names all three sites **and nothing else**:

```text
$ git show 0994c3c0:rust/Makefile > rust/Makefile && bash scripts/check_flow_integrity.sh
(10) rust/Makefile:93 invokes the generator with --debug --trace on the SHIPPING path:
      RUST_GENERATOR = $(RUST_AST_PIPELINE) --generate-parser --debug --trace --eliminate-left-recursion
(10) rust/Makefile:94 invokes the generator with --debug on the SHIPPING path:
      RUST_GENERATOR_BOOTSTRAP = $(RUST_AST_PIPELINE_BOOTSTRAP) --generate-parser --bootstrap-mode --debug --eliminate-left-recursion
(10) rust/Makefile:861 invokes the generator with --debug on the SHIPPING path:
      $(RUST_AST_PIPELINE) --generate-parser --bootstrap-mode --debug --eliminate-left-recursion $(GENERATED_DIR)/ebnf.json -o $(GENERATED_DIR)/ebnf.rs; \
```

⛔ **Line-scoped on purpose, and three CONTROLS prove the scoping is not over-broad** — a rule that
condemned the escape hatch or its own explanation would be unlandable, and would teach the next
author to waive the gate rather than fix anything (`GENERATED-LINT-CORRECTNESS.4`'s measured lesson).
`--trace-rules` is a live TOOLBOX instrument and must pass; a *variable* holding the flags for the
opt-in path must pass; a *comment* quoting them must pass. **Honest bound, in the check's own
source:** it scans tracked `Makefile`s — the shipping recipe's only home — and NOT
`rust/scripts/*.sh`, where a gate capturing a trace on purpose is legitimate.

#### ⛔ TWO FINDINGS SURFACED WHILE MEASURING

1. **The mtime trap fired on me, live, and volume is no longer a tell.**
   `PGEN_TRACE_VERBOSITY=high` first measured **62 B**. That was not a result: `make` had judged
   `generated/json_parser.rs` up to date because the `touch` landed **27 ms** after the previous run
   wrote it, so the recipe never ran and exited 0. This is verbatim
   [[feedback_verify_sv_parser_regen_mtime]], and it was caught **only** because the number was
   implausible. ⭐ That record's own explanation cited the *"huge `--debug --trace` output"* as part
   of why the trap is easy to miss — this leaf removes that output, so the one incidental tell is
   gone and the mtime assertion is now the only cheap one. The record was updated to say so, in this
   commit, rather than left to describe a world that no longer exists.
2. **A rejected-alternative comment that was reasoned backwards.**
   `rust/scripts/ci_workflow_local_gate.sh` recorded that *"quietening the recipe was rejected as out
   of scope: that recipe is SHARED with the tracked hosted workflow, and changing what evidence it
   leaves is a different change with a different owner."* Being shared with the hosted workflow is
   precisely why it had to be quietened — the sharing is the blast radius, not a firewall. The
   comment is corrected in place **with its old reasoning quoted**, not deleted, because a wrong
   rejection that sat unread is the more useful artifact.
   ⚠️ **The charitable half, added on re-audit rather than left out.** That note had a *second*
   clause — *"changing what evidence it leaves is a different change with a different owner"* — and
   that concern was legitimate: this change does reduce what a CI failure leaves behind. It is
   answered rather than dismissed. The director ruled on it directly; the capability is one env var
   away (`PGEN_TRACE_VERBOSITY=debug`); the parity gate's 4 MiB tail bound stays; and the evidence
   being "reduced" was, at 6.89 GB, already being truncated by the log surface it was streamed into,
   which is the opposite of preserved. Only the SHARED-recipe half of the rejection was backwards.

#### Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — `make --dry-run` over the shipping recipe, against the pre-fix tree
  (`git show 0994c3c0:rust/Makefile > rust/Makefile`), shows the flags on the deliverable command
  line itself — `make -C rust SHELL=/bin/bash --dry-run focus_systemverilog`:

  ```text
  ./target/debug/ast_pipeline_bootstrap --generate-parser --bootstrap-mode --debug --eliminate-left-recursion ../generated/semantic_annotation.json -o ../generated/semantic_annotation_parser.rs
  ./target/debug/ast_pipeline_bootstrap --generate-parser --bootstrap-mode --debug --eliminate-left-recursion ../generated/return_annotation.json -o ../generated/return_annotation_parser.rs
  ./target/debug/ast_pipeline --generate-parser --debug --trace --eliminate-left-recursion ../generated/systemverilog.json -o ../generated/systemverilog_parser.rs
  ```

  and the same command on the fixed tree prints the three lines with the flags gone. Captured cost of
  one full regeneration at `0994c3c0`: **6 894 576 244 B / 154 s**
  (`docs/tasks/artifacts/ci_parity_gate_rot/generator_trace_volume.txt` §1).
- [x] **ROOT CAUSE (WHY + WHERE)** — WHERE: three sites, derived not guessed —
  `git ls-files '*Makefile' | xargs grep -n -- '--generate-parser'` → `rust/Makefile:93`
  (`RUST_GENERATOR`), `:94` (`RUST_GENERATOR_BOOTSTRAP`), `:861` (the inline cold-clone `ebnf.rs`
  seed). WHY: the debug affordances sit in the variable every `focus_*` and
  `regenerate_generated_parsers` expands, so the deliverable path inherits them; and they are free of
  any consequence an artifact keeps, which is what let them sit unnoticed —
  `grep -rn 'config\.debug\|config\.trace[^_]' rust/src/` returns **two writes and zero reads**
  (`rust/src/main.rs:1097-1098`), so their only live effect is
  `resolve_trace_verbosity()` → the `pgen_trace!` gate at `rust/src/main.rs:936-938`.
  ⭐ **HARDENED same day**: that grep is BINDING-scoped and would miss a read through any other
  binding, which is a weaker instrument than the claim needs. Re-done at the FIELD level —
  `grep -rnE '\.(debug|trace)\b' rust/src/` plus `grep -rn -B12 'pub debug: bool' rust/src/` —
  **`PipelineConfig` is the only struct in the crate with a `pub debug: bool`**; its field is written
  at three sites (`main.rs:1097-1098`, the `Default` impl `mod.rs:2634-2635`, `bin/pgen_ast.rs:109-110`)
  and read at **none**. The `self.debug` reads that the loose search surfaces belong to
  `ReturnAnnotationHandler` (`return_annotation_handler.rs:52`) and to `ast_generator_direct.rs`,
  different structs with their own private fields. The conclusion did not change; the evidence now
  matches its strength.
- [x] **FIX** — declarative tier (build recipe + one structural enforcer; **zero** grammar bytes,
  one comment-only `rust/src/` edit, no `generated/*` in the change set). The flags are removed from
  all three sites; the opt-in is the engine's existing `PGEN_TRACE_VERBOSITY`, not a new variable
  (`DESIGN-PRIOR-ART`: `resolve_trace_verbosity` already reads it and no recipe passes
  `--verbosity`); `FLOW-INTEGRITY` invariant (10) stops the flags returning.
- [x] **ADDRESSED (verified)** — before→after **measured on both sides in this session**, not
  described: one full regeneration **6 894 576 244 B → 14 194 408 B** (485.7×; net of the one shared
  `ast_pipeline` cargo rebuild, ~6.88 GB → **4 838 B**) and **154 s → 131 s**. Re-runnable oracle:
  `bash scripts/check_flow_integrity.sh --report` → `generator invocations w/ --debug|--trace :
  0 (shipping path quiet)` and `flow-integrity: OK (… shipping generation recipe quiet across 1
  Makefile(s))`, RED-provable by `git show 0994c3c0:rust/Makefile > rust/Makefile`, which fails
  naming all three sites. Opt-in proven live: `PGEN_TRACE_VERBOSITY=debug make -C rust focus_json` →
  **21 955 687 B** vs **366 B** quiet.
- [x] **NO REGRESSION** — ⭐ the strongest form available here: **all 33 generated artifacts
  byte-identical** (`shasum -a 256 generated/*.rs generated/*.json`, 11 parsers + 22 JSON), verified
  against a **same-session determinism control** — a regeneration *with* the flags first reproduced
  the baseline exactly, so the after-comparison is not confounded by codegen stability. Direct
  generator A/B at all three sites, same `-o` path both times (the output path is embedded in the
  artifact, so differing paths differ for an unrelated reason — a mistake made once here and
  recorded): `ast_pipeline --generate-parser` json 21 955 519 B → 177 B identical; the `ebnf.rs`
  bootstrap seed 280 784 B → 171 B identical; `ast_pipeline_bootstrap` semantic-annotation
  205 807 B → 184 B identical. `bash scripts/check_doctrines.sh` green (all 18). Probe suite
  `run_flow_integrity_probes.sh` **24 arms / 24 PASS** — the 19 pre-existing arms unchanged plus
  RED-15 (the pre-fix Makefile replayed from the PINNED sha `0994c3c0`, not a hand-written
  imitation), RED-16 (`--debug` alone — the half a partial revert restores), and CTRL-6/7/8, the
  three false positives that would make invariant (10) unusable. `make -C rust clippy_on_rust_change`
  clean.
- [x] **LOCKSTEP** — `docs/book/src/gate-flow.md` (the log-bounding passage rewritten with the new
  numbers + the opt-in recipes; invariant table nine → **ten**), `docs/book/src/parse-harness.md`
  (the harness's codegen call is no longer "the recipe minus the logging flags" — it is now the same
  command line), `DOCTRINE_ENFORCEMENT.md` §10 (`FLOW-INTEGRITY` row, ten invariants),
  `rust/src/parse_harness.rs` (same correction at the source), `rust/scripts/ci_workflow_local_gate.sh`
  (the backwards rejected-alternative note), `docs/decisions/feedback_verify_sv_parser_regen_mtime.md`
  (its `--debug --trace` tell is gone; the trap is not), `rust/Makefile` (both comment blocks
  re-measured). New tracked artifact: `generator_trace_volume.txt`. ⛔ No DONE-BAR row moves: this
  changes what the build PRINTS, not what any parser accepts.


### `.30` NEW `todo` — `regenerate_generated_parsers` DOWNGRADES the binary it declares fully-featured, so the repository's own quick-start recipe fails on any warm tree (opened 2026-08-14 session #231 by `ENGINE-UNIVERSAL-SERVICES.17` slice 8; ⛔ PARKED behind the SV lane lock)

**ROUTING EVIDENCE** (`ROUTING-EVIDENCE` doctrine — what was MEASURED before routing, and whether it
reproduces outside the family it is being sent to):

- **Measured, on the shipped recipe, at `c66c7a32` + slice 8's diff.**
  `make -C rust SHELL=/bin/bash regenerate_generated_parsers` — the exact command `README.md`'s Quick
  Start prints and `.github/actions/regenerate-parsers` runs — **fails in 15 s**:

  ```text
  Error: REFUSED: semantic annotation '@whitespace_sensitive: true' needs the generated annotation
  backend, but this binary was built WITHOUT `--features generated_parsers` …
  make[2]: *** [regex_parser] Error 1
  ```

- **ROOT CAUSE, located rather than inferred.** `regex_parser_bootstrap` (`rust/Makefile:847`) runs
  `cargo build --features ebnf_dual_run --bin ast_pipeline` **unconditionally in both branches** —
  including the `else` branch whose only stated purpose is *"verifying it still compiles against
  these sources"*. Cargo features are per-invocation, so that call REPLACES
  `rust/target/debug/ast_pipeline` with a binary missing `generated_parsers`. It then runs
  `$(MAKE) regex_parser`, whose prerequisite `$(RUST_AST_PIPELINE)` — the rule that OWNS that path and
  declares the full feature set — is skipped, because make compares mtimes and the downgraded binary
  is now the newest thing in sight. Measured on the failing run:

  ```text
  2026-08-14 17:44  generated/return_annotation_parser.rs      ← $(RUST_AST_PIPELINE) prerequisites
  2026-08-14 17:44  generated/semantic_annotation_parser.rs
  2026-08-14 17:52  rust/target/debug/ast_pipeline             ← rebuilt UNDER-FEATURED by the bootstrap
  AST-PIPELINE-FEATURE-SURFACE: ebnf_dual_run=true generated_parsers=false
  ```

- ⭐⭐ **WHY CI NEVER SEES IT, AND WHY THAT IS THE WHOLE FINDING.** On a cold clone —
  `actions/checkout` with `generated/` untracked — those two prerequisites **do not exist**, so make
  MUST build them and the featured rule fires afterwards. The recipe is therefore correct exactly on
  the tree CI has and wrong on the tree every developer has. ⛔ A gate that only runs cold cannot
  observe a defect that only appears warm.

- ⛔ **It is NOT silent, and that is the one piece of good news — now MEASURED rather than argued.**
  The obvious fear is that an under-featured binary quietly generates a DIFFERENT artifact for some
  grammar that happens not to need the annotation backend. Tested directly, by pointing the
  under-featured `target/ebnf_frontend_build/debug/ast_pipeline`
  (`AST-PIPELINE-FEATURE-SURFACE: generated_parsers=false`) at both the SIMPLEST and one of the
  largest families:

  ```text
  json    refused (rc=1)
  vhdl    refused (rc=1)
  Error: REFUSED: semantic annotation '@entry: true' needs the generated annotation backend …
  ```

  ⭐⭐ **The trigger is `@entry: true`, which EVERY grammar in the repository carries** — so an
  under-featured binary cannot generate any parser at all, and silent artifact drift is impossible
  BY CONSTRUCTION rather than by luck. ⇒ severity is *"the documented recipe is broken for every
  local user"*, and NOT *"artifacts silently diverge"*. ⛔ Stated with its evidence because the two
  readings differ by an order of magnitude in urgency, and the slice that routed this leaf would
  otherwise have handed on a fear instead of a fact.

- **It reproduces outside `ENGINE-UNIVERSAL-SERVICES` by construction**, because nothing about it is
  grammar- or family-specific: any warm tree, any developer, any `make -C rust
  regenerate_generated_parsers`. `rust/scripts/ci_workflow_local_gate.sh`'s
  `prepare_generated_artifacts` calls the same target, so the local workflow-parity gate inherits it.

- ⛔ **This leaf's own family already fixed this exact class once**, which is why it belongs here:
  `.24` slice 2 made `$(RUST_AST_PIPELINE)` declare `generated_parsers` AND `ebnf_dual_run` precisely
  so *"the path's capability is a property of the path, not of whichever target happened to run
  last"* (`rust/Makefile:159-176`). The bootstrap's inline `cargo build` re-introduces the defect the
  rule was rewritten to prevent — one file apart, and the comment explaining the fix is 90 lines above
  the line that undoes it.

**Acceptance:** (a) `make -C rust regenerate_generated_parsers` succeeds on a WARM tree, RED-provable
against HEAD (the 15-second failure above is the calibration case, and it needs no fixture — a warm
tree is the default state); (b) the fix makes the path's capability structural rather than ordered —
candidates: have the bootstrap's verification step build the FULL feature set, or build it into a
separate `--target-dir` as `$(RUST_EBNF_FRONTEND_BIN)` already does, or make `$(RUST_AST_PIPELINE)`
order-only-independent of it; ⛔ *"remember to rebuild afterwards"* is not a candidate; (c) a check
that no recipe in `rust/Makefile` writes `rust/target/debug/ast_pipeline` with a feature set narrower
than the rule that owns that path declares — the mechanizable half, and the one that stops the third
instance.

⛔ **Sequencing.** PARKED behind the SV lane lock with `.25`–`.29`. It blocks no SV release work:
`.17` slice 8 got its no-regression measurement by building the featured binary explicitly and
invoking `annotation_parsers` + the per-family `focus_*` targets directly, which is the same sequence
`regenerate_generated_parsers` runs minus the downgrading bootstrap step — stated in that slice's NO
REGRESSION box rather than left as an unexplained deviation from the documented recipe.

---

### `.32` — GNU Make **3.81** compares timestamps at WHOLE-SECOND granularity, so a scripted edit→build loop silently consumes a STALE artifact (`done` — acute path `PGEN-CI-PARITY-GATE-ROT-0030` 2026-08-15 session #236; the repo-wide sweep (d) + acceptance (c) `PGEN-CI-PARITY-GATE-ROT-0032` 2026-08-15 session #237)

#### ⛔ HOW IT WAS FOUND — a CONTROL failed, and the control was right

Found while auditing `ENGINE-UNIVERSAL-SERVICES.21` slice 2's own published findings at the
director's challenge. An adversarial batch of four scratch-slot grammars reported LR rule names that
**did not exist in the grammars being tested** — shape `D1` (rules `y`, `w`) reported `expr_lr_base`,
which belongs to the *previous* shape. The batch's deliberate control `D2` was what exposed it.

⚠️ The first root cause was WRONG and is recorded because the correction is the point: the initial
hypothesis was coarse filesystem mtime. **Refuted by measurement** — the repository volume is APFS
with nanosecond mtimes:

```
$ stat -c '%.9Y  %n' rust/target/_mtime_probe_*
1786817878.583150835  …_1      1786817878.583396903  …_2      1786817878.583439942  …_3
```

#### ⛔⛔ THE ACTUAL ROOT CAUSE — the tool, not the filesystem

```
$ make --version | head -1
GNU Make 3.81
$ type -a make ; ls /opt/homebrew/bin/gmake
make is /usr/bin/make            # the ONLY make on this host — no 4.x anywhere
$ printf 'out: in\n\t@echo RULE_RAN\n\t@cp in out\n' > Makefile
$ : > in ; : > out ; : > in      # `in` is now NEWER than `out`
$ make
make: `out' is up to date.
$ stat -c '%.9Y %n' in out
1786817895.235307994 in          # 7 ms NEWER
1786817895.228333746 out
```

GNU Make **3.81** (2006, the version Apple ships) truncates mtimes to whole seconds. Sub-second
support arrived in make **4.x**. ⇒ **any prerequisite rewritten in the same second as its target is
invisible to every rule in this repository.**

#### ⭐ REPRODUCED DETERMINISTICALLY ON A REAL TARGET — no race, explicit timestamps

```
after ALPHA  : rules = "alpha"
  1786817945.900000000 grammars/scratch/scratch.ebnf     # prereq, 0.8 s NEWER
  1786817945.100000000 generated/scratch.json            # target
  did the frontend run? 0                                 # make skipped it entirely
after BRAVO  : rules = "alpha"   <-- the slot says `bravo`
```

⇒ the probe is driven against the **previous grammar**, silently, with `make` exiting 0.

#### ⚠️ WHY THIS HAS BEEN INVISIBLE, AND WHY IT IS NOT NEW

A human editing a grammar and typing `make` takes longer than a second, so hand use never sees it.
A **scripted or agent-driven loop** — which is exactly how the scratch slot is meant to be driven
(TOOLBOX 1.3) — hits it constantly. It fails **silently and in the PASSING direction**: a green run
against the wrong input.

⛔ **It is a second, independent mechanism for a symptom this repository has already paid for once.**
`GENERATED-LINT-CORRECTNESS.13` records session #218 losing time to `UNKNOWN=9` / every probe
`parsed=false` on a correct grammar, diagnosed then as a build-ORDER defect (`focus_scratch` builds
`ast_pipeline` before regenerating). That diagnosis was correct for that incident. This is a
different cause with the same signature, which is precisely why one fix did not prevent the other.

#### THE FIX (acute path) — and why it is scoped rather than global

`focus_scratch` deletes `$(SCRATCH_JSON)` and `$(SCRATCH_PARSER)` and then re-enters make. Timestamps
stop being part of the correctness argument for the probe path.

- ⛔ **A recipe step + recursive `$(MAKE)`, not a sibling prerequisite.** Sibling prerequisites have
  no guaranteed order under `-j`, so an `rm` sibling could race the generation it must precede — a
  fix that reintroduces a nondeterministic version of the same defect.
- ⛔ **Not applied to `$(SCRATCH_PARSER)` itself.** That would rewrite a file `build.rs` keys on and
  drag cargo into a rebuild on every unrelated `make`. `focus_scratch` is the phony operator entry
  point whose caller has *just* edited the grammar, so an unconditional regeneration is what they
  asked for. Cost: always ~2-4 s instead of ~0 s when up to date.

#### ⚠️ HONEST BOUND — WHAT IS **NOT** FIXED

Only the scratch probe path. **Every other rule in this repository still compares whole seconds**,
including the per-family `focus_*` targets and the aggregate regeneration recipe. Those are minutes
long, so a same-second prerequisite collision is unlikely — *unlikely, not impossible, and not
measured*. That sweep is acceptance (d) and is `todo`.

> ⛔⛔ **SUPERSEDED BY MEASUREMENT (`-0032`), AND THE PARAGRAPH ABOVE IS KEPT BECAUSE BEING WRONG IN
> PUBLIC IS THE POINT.** *"Those are minutes long, so a collision is unlikely"* names the wrong
> duration: the gap a sequential driver must beat is the work AFTER the target is written, not the
> total build. On the `parser ← json` edge that gap is the FRONTEND step — 0.006-0.111 s for every
> family — so **10 of 10 are exposed, SystemVerilog (28.5 s to regenerate, 0.054 s to re-emit its
> json) included**. All ten instances are guarded as of (d) below.

⭐ What is provably immune, and worth stating because it is the right pattern: **`PARSE-COST-RATCHET`
hashes CONTENT, not mtime**, so its identity tier cannot be fooled this way. Likewise
`SV-CORPUS-DENOMINATOR`'s byte-identical re-run. Content-addressed freshness is the general answer;
`rm` is the local one.

**Acceptance:** (a) ✅ the acute scratch path cannot serve a stale artifact — done, verified below;
(b) ✅ record the mechanism where the next reader will meet it — ⛔ **THIS BOX WAS TICKED EARLY AND
WAS FALSE FOR TWO OF ITS THREE SURFACES, AND THE CORRECTION IS RECORDED RATHER THAN QUIETLY
BACKFILLED.** `-0030` ticked it naming `TOOLBOX.md` 1.3, the book's *Parse Harness* chapter and a
knowledge card; `git show --stat 222e89d5` lists **neither `TOOLBOX.md` nor `docs/book/`** among its
9 files. Only the knowledge card was real. Both missing surfaces were written in `-0032` (this
slice), so the box is now true — but it was published false for two commits, and it is the same
class as the `-0039` overstatement this tree already records: *a claim about a surface, published
without opening the surface* → [[feedback_verify_a_claim_three_ways_before_publishing_it]] leg 1
(re-derive by command) would have caught it in one `git show --stat`;
(c) ✅ **`make >= 4.0` is PRICED, NOT ADOPTED — and (d)'s census is what made the call derivable
rather than a matter of taste.** It fixes all 65 file rules at once and needs no per-rule reasoning.
Against that: every contributor pays an install plus a `gmake`-vs-`make` rename across the README,
the book, `COMMIT.md` and 15 workflows; GitHub's ubuntu runners ship make 4.x while its macOS runners
ship 3.81, so requiring 4.0 buys a **CI-parity split** in the one repository whose named subject is
CI parity. The exact-window guard adopted in (d) reaches all ten instances for one 150-line script
and nine one-line recipe additions, with zero cost on an up-to-date tree — so the upgrade buys
nothing the guard does not already have, at a cost the guard does not have. ⇒ **declined, revisit
only if a future rule shape cannot be guarded**;
(d) ✅ swept — see below.

#### Acceptance Checklist (enforced) — `.32` (acute path)

- [x] **REPRODUCE / ISSUE** — reproduced deterministically, not observed once. With the slot's mtime
  pinned 0.8 s NEWER than `generated/scratch.json` inside a single second,
  `make -C rust SHELL=/bin/bash focus_scratch` ran the frontend **0** times and the regenerated
  parser still declared `alpha` while the grammar said `bravo`. Independently corroborated by the
  batch that surfaced it: shapes `D1` and `D4` reported LR rule names belonging to the *previous*
  shape (`expr_lr_*` for a grammar containing no rule `expr`).
- [x] **ROOT CAUSE (WHY + WHERE)** — WHY: GNU Make 3.81 truncates mtimes to whole seconds, so a
  prerequisite rewritten in the same second as its target never looks newer. WHERE, by the
  ops/build-flow toolbox — and note the FIRST hypothesis was refuted by it:

  ```
  $ stat -c '%.9Y  %n' rust/target/_mtime_probe_1 rust/target/_mtime_probe_2
  1786817878.583150835  rust/target/_mtime_probe_1     # APFS nanosecond mtimes — the
  1786817878.583396903  rust/target/_mtime_probe_2     # "coarse filesystem" hypothesis is REFUTED
  $ make --version | head -1
  GNU Make 3.81                                        # the tool, not the filesystem
  $ : > in ; : > out ; : > in ; make
  make: `out' is up to date.                           # with `in` 7 ms NEWER than `out`
  ```

  The affected rule is `rust/Makefile`'s `$(SCRATCH_JSON): $(SCRATCH_EBNF) …`.
- [x] **FIX** — fix-hierarchy tier = **ops/build-flow**; zero engine, grammar or generated bytes.
  `focus_scratch` becomes a recipe that `rm -f`s both scratch artifacts and re-enters via recursive
  `$(MAKE)`, so the probe path never depends on a timestamp comparison. Scoped to the phony operator
  entry point for the two reasons recorded above (`-j` ordering; not dragging cargo into a rebuild).
- [x] **ADDRESSED (verified)** — measured before→after on the identical deterministic reproducer.
  **BEFORE:** frontend ran **0** times, parser declared `alpha` for a grammar saying `bravo`.
  **AFTER:** frontend ran **1** time, parser declares `bravo`. The two shapes invalidated by the bug
  were re-run under a forced clean and now report their own rules — `D1` → `w_lr_base w_lr_suffix`
  (was `expr_lr_*`), `D4` → `e_lr_base e_lr_suffix` (was `a_lr_*`/`b_lr_*`).
- [x] **NO REGRESSION** — `bash scripts/check_doctrines.sh` → **ALL 20 enforced doctrines PASS**.
  The `SCRATCH-SLOT-HEADER` guard still fires through the restructured recipe (header-less slot →
  `make` rc **2**; default fixture → rc **0**), so `PARSE-HARNESS.11` is intact. The slot is
  byte-identical to HEAD and regenerates the default fixture with 0 LR rules.
  `make -C rust SHELL=/bin/bash mdbook_docs_gate` passes. ⭐ **Freshness audit of the claims this
  session already committed**: `PGEN-ENGINE-UNIVERSAL-SERVICES-0039`'s probes A/B/C each ran the
  frontend and the generator exactly once (`grep -c` on their captured logs = 1/1 each), so none of
  them rode a stale artifact and none of that leaf's published findings are affected.
- promotion: `docs/knowledge/your-build-tools-timestamp-resolution-is-part-of-your-correctness-argument.md` **NEW**.

#### ✅ `.32` (d) — THE SWEEP, DISCHARGED IN ONE SLICE (director-ordered 2026-08-15, **TIME-BOXED**: *"continue the make rules, but please make sure it does not last days"*, then return to the SV lane)

**Derived by command, so the next session resumes without re-discovering it** (`rust/Makefile` is the
only tracked Makefile — `git ls-files '*Makefile*'`):

| population | count | how derived |
|---|---:|---|
| file-target rules | **65** | `awk '/^[^\t#[:space:]][^:=]*:[^=]/' rust/Makefile`, minus `.`-prefixed specials |
| rules with an **authored** prerequisite (`*.ebnf` / `AST_PIPELINE_SOURCES` / `build.rs`) | **13** | the same scan filtered on `EBNF\|GRAMMARS_DIR\|SOURCES\|\.ebnf` |
| ⭐ of those, the **identical shape** to the one that bit `.32` | **10** | `grep -cE '^\$\([A-Z_]+_JSON\): \$\([A-Z_]+_EBNF\) \$\(RUST_EBNF_FRONTEND_BIN\)'` |
| `focus_*` operator entry points | **8** | `focus_json focus_regex focus_rtl_const_expr focus_rtl_frontend focus_scratch focus_systemverilog focus_systemverilog_preprocessor focus_vhdl` |

⇒ **`$(X_JSON): $(X_EBNF) $(RUST_EBNF_FRONTEND_BIN)` is a TEN-INSTANCE family, and `scratch` was
simply the one an agent drove in a loop.** The acute fix landed on `focus_scratch` only, so the other
nine carry the same exposure today.

⛔ **The exposure criterion, stated so the pricing is not done by feel.** Make 3.81 skips a rule iff
`floor(mtime(target)) >= floor(mtime(prereq))`. The prerequisite here is edited by a human or a
script *between* builds, so the hazard is **"can the previous build's target write and the next
prerequisite edit land in the same wall-clock second?"** ⇒ exposure is a function of **how fast the
rule's own build is**, not of how important it is. `scratch` regenerates in ~2 s and was hit
constantly; `systemverilog` takes minutes and is very unlikely to be hit *by a human* — ⚠️ but that
is an argument about the DRIVER, not the rule, and an agent loop changes the driver. Not yet measured.

**What (d) owed, and what each item became** (DISCHARGED 2026-08-15 session #237,
`PGEN-CI-PARITY-GATE-ROT-0032`; one slice, as the director's time-box required):

1. ✅ **Price the ten** — and the pricing question in this list was itself WRONG. It said *"measure
   each `focus_*`'s wall time"*. Total wall time is the wrong quantity and would have declared
   SystemVerilog safe. A sequential driver cannot edit the grammar before `make` returns, so the gap
   it must beat is **the work that happens AFTER the target is written**, which splits the chain into
   two edges with completely different exposure. Criterion, exact rather than felt: make skips iff
   `floor(target) >= floor(prereq)`, so a gap `>= 1.000 s` guarantees `floor(prereq) > floor(target)`
   ⇒ provably immune.

   | edge | target ← prereq | the gap a driver must beat | measured | verdict |
   |---|---|---|---:|---|
   | **A** | `$(X_JSON)` ← `$(X_EBNF)` | the GENERATOR step | 0.062-28.461 s | **5 of 10 exposed** |
   | **B** | `$(X_PARSER)` ← `$(X_JSON)` | the FRONTEND step, on the NEXT build | 0.006-0.111 s | ⛔ **10 of 10 exposed** |

   ⛔⛔ **EDGE B WAS NOT IN THIS LEAF'S POPULATION TABLE AT ALL, AND IT REFUTES THIS LEAF'S OWN
   "HONEST BOUND".** `-0030` wrote that the minute-long families are *"unlikely to be hit — unlikely,
   not impossible, and not measured"*. Measured: on edge B the frontend step is **0.054 s for
   SystemVerilog**, so the family that takes 28.5 s to regenerate is exposed like every other. And
   the failure it produces reads WORSE than the original: a **fresh json beside a stale parser**, so
   the artifact the gates and censuses inspect is current while the artifact that actually parses is
   not. Exposed on edge A: `return_annotation` 0.325 s, `json` 0.139 s, `scratch` 0.062 s,
   `rtl_const_expr` 0.332 s, `systemverilog_preprocessor` 0.468 s.
   Instrument (TRACKED, per `ENGINE-UNIVERSAL-SERVICES.21` (e)'s lesson):
   `docs/tasks/artifacts/ci_parity_gate_rot/run_make_freshness_window_census.sh` →
   `make_freshness_window_census.txt`. n=3, minimum reported (immunity must hold on the fastest run).
   It never touches `generated/`, and its identity control reproduced all 10 shipped json+parser
   pairs from the tracked grammars — the timings were paid on the real work.

2. ✅ **ONE fix shape, uniform — but NOT the one this list proposed.** The proposal was to generalise
   `focus_scratch`'s unconditional `rm` to all 8 entry points, with the honest warning that
   `focus_systemverilog` would then always pay its full regeneration. That warning is now MEASURED
   and is disqualifying: **three tracked gates call `focus_*` purely to ENSURE an artifact exists** —
   `rust/scripts/sv_cert_recognized_union_gate.sh:177` and `verilog_2005_conformance_gate.sh:172`
   (`focus_systemverilog`, 28.5 s of generation) and `rtl_const_expr_cert_gate.sh:179`. Uniform `rm`
   would charge each of them a full regeneration per run for a hazard window they are not in.
   ⇒ **`scripts/make_freshness_guard.sh`** instead: it removes an artifact only where make's own
   comparison is **provably wrong** —
   `floor(target) == floor(prereq)` **AND** `mtime(prereq) > mtime(target)`, read as **nanosecond
   integers** (a float `st_mtime` near a 1.79e9 epoch cannot order two writes a few hundred µs apart
   — a guard for a precision defect must not have a precision defect). Same-second-but-target-newer
   is make being RIGHT, and the guard leaves it alone. ⇒ **zero cost on an up-to-date tree**, which
   is what makes it affordable on all ten instances including SV. Wired into all 7 remaining
   `focus_*` targets plus the two annotation flows (`--json-only` there: their recipe regenerates the
   parser unconditionally, so edge B does not exist and removing it would only let
   `$(RUST_AST_PIPELINE)`'s placeholder logic drop a stub in its place). `focus_scratch` keeps its
   stronger unconditional `rm` — its caller has just edited the slot by construction, and
   `PARSE-HARNESS.11`'s probe-time header check rides that rule.
   ⛔ Recipe step + recursive `$(MAKE)`, never a sibling prerequisite (`-j` ordering), per `.32`(a).

3. ✅ **Stop it recurring — `FLOW-INTEGRITY` invariant (11), not a 21st doctrine**, exactly as this
   item proposed. Population DERIVED from the Makefile (`$(<FAM>_JSON): $(<FAM>_EBNF)` → 10 rules),
   so an eleventh family added tomorrow is covered by construction rather than by someone remembering
   to extend a list; a family counts as covered if its `$(<FAM>_JSON)` is named on a guard invocation
   **or** on an unconditional `rm` (both shapes are legitimate, and CTRL-9/9b prove the check accepts
   the second — otherwise it would be demanding a spelling rather than the property). Cost: greps,
   no build; the doctrine driver still runs in seconds.

4. ✅ **`make >= 4.0` PRICED, NOT ADOPTED** — recorded in acceptance (c) above, and the census is what
   made it a derivation instead of a preference.

#### Acceptance Checklist (enforced) — `.32` (d), the sweep

- [x] **REPRODUCE / ISSUE** — reproduced on a family the acute fix never reached (`json`), pinned not
  raced, and read through **make's own verdict** rather than through this fix's model of make:
  `make -q <target>` exits 0 for *"up to date"*, 1 for *"would rebuild"*, and executes nothing.

  ```
  RED-A1  grammar pinned 0.8 s NEWER than generated/json.json, same whole second
          make -C rust -q ../generated/json.json          → rc 0   ⛔ make will SKIP the rule
  RED-B1  json pinned 0.8 s NEWER than generated/json_parser.rs, same whole second
          make -C rust -q ../generated/json_parser.rs     → rc 0   ⛔ fresh json, STALE parser
  ```

  Population: 10 instances of `$(X_JSON): $(X_EBNF) $(RUST_EBNF_FRONTEND_BIN)`, 5 exposed on edge A
  and 10 on edge B (census above).
- [x] **ROOT CAUSE (WHY + WHERE)** — WHY: GNU Make 3.81 truncates mtimes to whole seconds
  (`make --version` → `GNU Make 3.81`; the APFS-nanosecond hypothesis was refuted in `.32`(a) by
  `stat -c '%.9Y'`), so it cannot order two writes inside one second and answers *"up to date"*.
  WHERE, by the **ops/build-flow** toolbox — and the instrument is deliberately make itself, because
  a defect IN MAKE'S COMPARISON cannot be diagnosed by a re-implementation of that comparison, which
  would agree with the bug:

  ```
  $ (cd rust && make -q ../generated/json.json) ; echo rc=$?      # make's own verdict, runs nothing
  rc=1                                                            # 1 = it would rebuild
  $ (cd rust && make -d -q ../generated/json.json) | grep -E 'newer than target|Must remake'
   Prerequisite `target/ebnf_frontend_build/debug/ast_pipeline' is newer than target `../generated/json.json'.
  Must remake target `../generated/json.json'.
  $ (cd rust && make -n focus_json) | head -1                      # the guard's own wiring, dry-run
  bash ../scripts/make_freshness_guard.sh --chain ../grammars/json.ebnf …
  ```

  Under the pinned same-second collision that same `make -q ` returns **rc 0** — make declaring a
  target up to date while its prerequisite is genuinely newer. WHERE, precisely: both edges of all
  ten `rust/Makefile` chains — `$(X_JSON): $(X_EBNF) $(RUST_EBNF_FRONTEND_BIN)` and
  `$(X_PARSER): $(X_JSON) $(RUST_AST_PIPELINE)` — with exposure a function of the per-edge TAIL.
- [x] **FIX** — fix-hierarchy tier = **ops/build-flow**; zero engine, grammar or generated bytes.
  `scripts/make_freshness_guard.sh` (exact-window, nanosecond integers, two edges, cascade) called
  from 9 entry points; `FLOW-INTEGRITY` invariant (11) keeps the population covered.
  ⛔ **`make -n` caught a defect in the guard's FIRST CUT before it shipped, and it was the silent
  kind**: the script `cd`s to the repository root like every other script here, while `make` runs
  from `rust/` and passes `../generated/json.json` — so every path would have resolved OUTSIDE the
  repository, every `stat` would have come back absent, every edge would have answered *"nothing to
  do"*, and the guard would have exited 0 having inspected **nothing**. Fixed by capturing the
  invocation cwd and re-anchoring, and — because that class must not return quietly — a **missing
  grammar is now a hard refusal (exit 2)**, with CTRL-3 asserting it.
  ⭐ **`make -q` / `make -d` REGISTERED in `check_diagnosis_evidence.sh`'s ops/build-flow vocabulary,
  because the gate CORRECTLY refused this very leaf.** `make -n` was already a token; `-q` and `-d`
  — the instruments that actually produced this diagnosis — were not, so the box was blocked. That
  is the lockstep obligation the enforcer's own header records (`ENGINE-UNIVERSAL-SERVICES.11`
  slice 1): register the instrument, never cite a tool that did not produce the diagnosis and never
  waive. ⛔ PRICED as a widening must be: across all **404** ticked ROOT CAUSE boxes in
  `docs/tasks/`, `make -q ` / `make -d ` occur **once** corpus-wide — in this leaf. Zero existing
  boxes newly qualify. Its four probe suites re-run GREEN afterwards (6 + 10 + 13 + 9 = 38 arms).
- [x] **ADDRESSED (verified)** — measured before→after on the identical pinned reproducer, oracle
  `make -q` throughout. **18/18 arms**
  (`docs/tasks/artifacts/ci_parity_gate_rot/run_make_freshness_probes.sh`, output
  `make_freshness_probes.txt`):

  | arm | before | after the guard |
  |---|---|---|
  | edge A (`json ← grammar`) | `make -q` rc **0** — will skip | json + parser removed → rc **1**; `make focus_json` runs frontend ×1, generator ×1 |
  | edge B (`parser ← json`) | `make -q` rc **0** — will skip | parser removed, **json kept** → rc 1; `make focus_json` runs frontend ×**0**, generator ×1 |
  | determinism | — | the regenerated parser is **byte-identical** to the pre-probe artifact |
  | zero-cost control | `make focus_json` = no-op | still a no-op: frontend ×0, generator ×0, guard prints nothing |
  | over-fire control (grammar newer by a whole second) | make already sees it (rc 1) | guard stays out of the way |
  | over-fire control (same second, TARGET newer) | make is RIGHT (rc 0) | guard does not fire |

  ⛔ The two over-fire controls are load-bearing: without them *"always force"* would pass every RED
  arm above, and always-force is precisely the option item 2 rejected. Guard self-test **8/8**
  (`--self-test`), every arm pinning mtimes so nothing depends on machine speed.
- [x] **NO REGRESSION** — `bash scripts/check_doctrines.sh` → **ALL 20 enforced doctrines PASS**
  (`FLOW-INTEGRITY` now reports `10/10 EBNF→json rules guarded against make-3.81 whole seconds`),
  re-run WITH the change staged so the 5 staged-scope doctrines evaluated it rather than nothing.
  `run_flow_integrity_probes.sh` → **29/29** (was 24; +RED-17 a family loses its guard, +RED-18 an
  eleventh family arrives unguarded, +RED-19 the guard survives only as a comment, +CTRL-9/9b the
  `rm` shape still counts). The touched enforcer's own four probe suites stay GREEN —
  `run_diag_evidence_probes.sh` 6/6, `…_root_kw_probes.sh` 10/10, `…_family5_probes.sh` 13/13,
  `…_leaf_scope_probes.sh` 9/9. `make -C rust SHELL=/bin/bash mdbook_docs_gate` passes. The probe restores
  the grammar's mtime and both artifacts' bytes and mtimes through an exit trap; `git status` clean
  for `generated/` inputs afterwards.
- promotion: `docs/knowledge/your-build-tools-timestamp-resolution-is-part-of-your-correctness-argument.md`
  UPDATED — the two-edge measurement (the "slow targets are safe" reasoning names the wrong duration)
  and the exact-window guard as a fourth, stronger remedy.

### `.33` NEW `todo` — a ticked acceptance box may name a SURFACE the commit never touched, and nothing compares the claim against the diff (opened 2026-08-15 session #237 by `.32`(d), which found one)

**ROUTING EVIDENCE** (`ROUTING-EVIDENCE` doctrine — what was measured, and whether it reproduces
outside the leaf that surfaced it):

- **A real instance, in this tree, found by command not by suspicion.** `.32` acceptance (b) was
  ticked reading *"record the mechanism where the next reader will meet it (`TOOLBOX.md` 1.3, the
  book's *Parse Harness* chapter, a knowledge card) — done"*:

  ```
  $ git show --stat 222e89d5 | tail -12
   CHANGES.md · DEVELOPMENT_NOTES.md · KNOWLEDGE_MAP.md · MEMORY.md · docs/TASK_TREE.md
   docs/knowledge/your-build-tools-…-argument.md · docs/tasks/CI-PARITY-GATE-ROT.md
   docs/tasks/ENGINE-UNIVERSAL-SERVICES.md · rust/Makefile
  ```

  **Neither `TOOLBOX.md` nor anything under `docs/book/` is in the commit.** Two of the three named
  surfaces were never written; only the knowledge card was real. The box was true for one third of
  what it claimed and it passed every gate.
- ⛔ **Why no gate saw it.** `TASK-ACCEPTANCE` audits that the boxes EXIST, are TICKED and carry a
  diagnosis SIGNATURE; `.27` (already open) adds that it never audits FALSIFIABILITY. This is a third
  axis neither covers: whether a box's own claim about WHERE something was recorded matches the
  files the commit actually touched. Nothing in the driver reads the diff for that.
- ⭐ **It is mechanically checkable, which is what makes it a defect rather than a fact of life.** A
  ticked box naming a tracked path (`` `TOOLBOX.md` ``, `` `docs/book/src/x.md` ``) inside a
  record/document/write claim is a testable assertion against `git diff --cached --name-only`. The
  hard part is not detection, it is the FALSE-POSITIVE rate: boxes legitimately name files as
  CONTEXT (*"the affected rule is `rust/Makefile`'s `$(SCRATCH_JSON)`"*) far more often than as a
  promise, and a checker that cannot tell those apart teaches waivers — the failure
  `GENERATED-LINT-CORRECTNESS.6`/`.12` document and the reason `.4` (2/304) and `.7` (0/307) REFUSED
  two proposed additions on measurement.
- **Reproduces outside this tree?** ⚠️ **Unmeasured, and that measurement is acceptance (a).** One
  instance is an anecdote. The corpus is ~404 ticked ROOT CAUSE boxes plus their siblings across 162
  task files; the question *"how many ticked boxes name a surface their own commit did not touch"*
  is answerable by script and has never been asked.
- **Class, stated plainly:** this is the same family as `.21`'s GAP 1 and `-0039`'s overstatement —
  **a claim published without opening the surface it claims about.** Three instances in three days,
  each caught by a human re-reading rather than by a gate.

**Acceptance:** (a) ⛔ FIRST, MEASURE THE CORPUS before designing anything — for every ticked box in
`docs/tasks/` that names a tracked path inside a record/document/update claim, join against the
commit that introduced it (`git log -L` / `git blame` on the box line) and report how many name a
file that commit did not touch. That number decides whether this is a class or an incident, and
`.4`/`.7` are the precedent for REFUSING to build on a thin one; (b) if the population justifies it,
the check must distinguish a PROMISE (*"recorded in X"*, *"documented in X"*) from CONTEXT (*"the
affected rule is in X"*) — and the arm that proves it is a control box naming a file as context,
which must PASS; (c) ⭐ whatever (a) decides, `.32`(b) itself is already corrected in place with the
false claim recorded rather than backfilled — the record of the miss is the fixture (b) would reuse.


### `.34` NEW `todo` — a make target NAMED inside a check script's actionable error message is counted REACHABLE, and the false badge then BLOCKS recording the truth (opened 2026-08-16 session #241 by `ENGINE-UNIVERSAL-SERVICES.29`(b), which hit it while registering a new doctrine)

**ROUTING EVIDENCE** (`ROUTING-EVIDENCE` doctrine — what was measured, and whether it reproduces
outside the leaf that surfaced it):

- **The mechanism, located.** `scripts/check_gate_reachability.sh` root scan **(R3)** reads every
  `.githooks/*` and `scripts/check_*.sh` and calls `invoked_targets()` on the result, which extracts
  make targets from *command segments*. It strips shell comments first — so a target named in a
  `#` comment is correctly ignored — but a target named inside a **quoted error STRING** is parsed
  as a command and becomes an edge. ⇒ printing an actionable *"fix it with: make -C rust
  SHELL=/bin/bash <target>"* certifies `<target>` as reachable, in the strongest class
  (`git-hook`, which the inventory counts as **AUTOMATIC**).
- **Measured, with a RED probe rather than by reading.** `ENGINE-UNIVERSAL-SERVICES.29`(b) added
  `generated_reproducibility_gate` (tier 2 of the new `GENERATED-REPRODUCIBILITY` doctrine, invoked
  by no aggregate, workflow or hook). The inventory reported:

  ```text
  generated_reproducibility_gate               git-hook
  ```

  Replacing the target's name in `check_generated_reproducibility.sh`'s error string with a
  placeholder — changing nothing else — reclassified it immediately:

  ```text
  generated_reproducibility_gate               ⛔ UNTRIAGED
  gate-reachability: … Either wire the target into an aggregate / CI workflow / hook, or record a
  deliberate disposition for it in …/gate_reachability_register_v0.json.
  ```

  ⇒ the sole reason it was ever classified reachable is a string in an error message.
- ⛔⛔ **THE SHARPER HALF: the false badge PREVENTS the corrective action.** The register is a
  two-sided ratchet — it fails on a disposition naming a target the inventory believes is reachable
  (*"1 register entr(ies) name a target that is no longer orphaned … remove them so the register
  cannot accumulate dead exemptions"*). So the honest row could **not** be recorded while the false
  edge stands: an attempt to write `generated_reproducibility_gate:
  accepted-operator-invoked` fails the doctrine. The mis-classification is therefore not cosmetic —
  it actively keeps the true state out of the register.
- **The exposed population is SMALL, and that is measured, not assumed.** Of **118** `*_gate` make
  targets, exactly **2** are named in a `make …` phrase inside `scripts/check_*.sh` or
  `.githooks/*`: `fixed_point_gate` (whose mention is inside a `#` comment in
  `check_gate_reachability.sh` — stripped, and which is genuinely reachable via the aggregate and
  two workflows regardless) and `generated_reproducibility_gate`. ⇒ **exactly one target is
  currently mis-certified, and it is the one that surfaced this.** No pre-existing lane is
  falsely green.
- **It reproduces outside this instance by construction**, which is why it is worth a leaf despite
  the population of one: the trigger is *"a check script prints an actionable make command"*, which
  this repository actively encourages — `check_parse_cost_ratchet.sh` and
  `check_generated_reproducibility.sh` both do it because a refusal that does not say how to fix
  itself is a worse refusal. Every future doctrine that follows that good practice AND ships a
  `*_gate` target inherits the false badge.
- ⚠️ **Not fixed here, and the reason is stated rather than implied.** `invoked_targets()` /
  `command_segments()` are the functions whose own source documents **five** calibration defects
  found while writing them, and the inventory is guarded by 8 ground-truth controls precisely
  because it produced six different confident answers. Editing that parser as a side effect of
  landing an unrelated doctrine is the scope-widening this repository has a file of incidents about.
  `ENGINE-UNIVERSAL-SERVICES.29` records the true disposition in prose in the meantime.

**Acceptance:** (a) make R3's extraction distinguish an EXECUTED make invocation from one QUOTED
inside a message — the natural discriminator is that the latter sits inside a string literal, and
whatever is chosen must be proven on the existing 8 ground-truth controls **before** it is trusted,
since the population it re-classifies is 125 targets; (b) a RED arm added to the controls that
replays exactly this case — a target named only in an error string must come out ORPHAN — because
the defect is invisible in the passing direction and was found only by someone probing their own
green result; (c) once (a) lands, add the deliberate row
`generated_reproducibility_gate: accepted-operator-invoked` to the register, which is the row this
leaf's existence is currently substituting for; (d) ⛔ do NOT fix this by removing the target name
from the error message — the message is correct practice and the defect is in the reader.

#### ✅ CLOSED 2026-08-16 (session #242, `PGEN-CI-PARITY-GATE-ROT-0033`) — all four clauses met

**(a) THE READER NOW MODELS SHELL QUOTING — and the design was chosen by measurement, not by
reading.** The obvious fix ("track double-quote parity across lines") was implemented first and
**measured on the real corpus before being trusted**: it mis-classifies **1 862 lines** across 22
gate scripts, because three live shapes all break a parity counter —

| shape | example | what a parity counter does |
|---|---|---|
| 1. the message string | `breach "…\n … make -C rust … \n …"` | correct — this is the target |
| 2. `x="$( … )"` substitution | 20 gate scripts | ⛔ blanks a body that IS executed |
| 3. multi-line `'…'` jq/awk/perl + heredoc payloads | 12 + 13 gate scripts | ⛔ desyncs on stray `"` |

So `string_data_lines()` models the four contexts a shell body actually has — `NORMAL`, `'…'`,
`"…"` (inside which `$(` and a backtick **re-enter** `NORMAL`), and heredoc bodies — and suppresses
only lines that **BEGIN** inside a quoted string or a heredoc payload. ⭐ Blanking string INTERIORS
was priced and REFUSED: **433** lines carry both a double quote and a make/script mention, so
interior-blanking would drop edges wholesale (`bash "$ROOT/scripts/x.sh"` is a quoted command word).
⛔ `COMMIT.md` is scanned with `shell_syntax=False`: prose is not shell, and the stack model calls
**38 of its 204 lines** "string data" off **4** apostrophes — missing today's six `make` lines by
luck alone.

**THE FULL 125-TARGET REPORT MOVES EXACTLY ONE ROW**, as this leaf predicted, and all 8 pre-existing
ground-truth controls reproduce:

```text
- reachable at all: 94        →  reachable at all: 93
- ORPHANS: 30                 →  ORPHANS: 31
- generated_reproducibility_gate               git-hook
+ generated_reproducibility_gate               ⛔ UNTRIAGED
```

⭐ Three variants (dq-only / +single-quote / +heredoc) were each run to completion and **all three
produce the identical single move** ⇒ the sq and heredoc arms are inert on today's corpus, so the
choice among them is semantic, not empirical. All three are suppressed, because a heredoc body and
a single-quoted program are data exactly as a double-quoted string is, and the failure direction is
safe: **a lost edge yields a FALSE ORPHAN, which fails the gate loudly; a false edge is the silent
one this defect was.**

**(b) SIX GROUND-TRUTH ARMS, EVERY ONE OBSERVED FIRING.** `SYNTAX_CONTROLS` runs on every invocation
(microseconds, no subprocess): 2 RED (a target named only in a multi-line message / only in a heredoc
payload ⇒ no invocation) and 4 GREEN (a plain invocation; the three corpus shapes above). The control
count in the OK line is derived, `8 → 14`.
⛔ **A control nobody has watched fail is not ground truth**, so
`docs/tasks/artifacts/ci_parity_gate_rot/run_gate_reachability_string_reader_probes.sh` proves each
one — by **MUTATING the live reader** with `sed`, never by re-typing the old one
(`GENERATED-LINT-CORRECTNESS.4`'s hand-copied-rule trap). 8 arms, all green:
`BASE` · `RED-1a/1b` (suppression off = the pre-fix reader) · `RED-2` (sq state off) · `RED-3` (`$(`
re-entry off) · `RED-4a/4b` (heredoc tracking off) · `DIFF` (HEAD vs tree = exactly one row).
Transcript: `docs/tasks/artifacts/ci_parity_gate_rot/gate_reachability_string_reader_probes.txt`.

⭐⭐ **AND THE ARMS CAUGHT THREE DEFECTS IN THIS LEAF'S OWN WORK, WHICH IS THE POINT OF WRITING THEM
FIRST.** The first cut of arms 4, 5 and 6 was **non-discriminating** — each "passed" under the
mutation meant to break it, because its fixture happened to balance its quotes (`{a: "x"}`, a
`print("…")`, a payload whose `make` was not at command position). A control that cannot fail is
the exact disease this doctrine exists to name, shipped inside the fix for it. Rebuilt against the
real corpus shapes: an awk body with `gsub(/"/, "")` (**odd** quote count), a heredoc payload with
one stray `"`, and a payload naming the target at command position.

⚠️ **AND ONE ARM FAILED FOR A REASON THAT WAS NOT MINE — a PRE-EXISTING blind spot, measured and
ROUTED rather than fixed here.** `out="$(make -C rust … x)"` on ONE line yields no edge in the
pre-fix reader either: `out="$(make` is eaten whole as a `VAR_ASSIGN` token, leaving `-C` as the
command word. Measured population in the scanned corpus: **0**. Routed as `.35` rather than widened
into this commit — this leaf's own ROUTING EVIDENCE says editing that parser as a side effect is the
scope-widening this repository has a file of incidents about, and it binds on its author too.

**(c) THE REGISTER ROW IS RECORDED** — `generated_reproducibility_gate: accepted-operator-invoked`,
32 entries, gate GREEN:
```text
gate-reachability: OK (125 targets; 93 reachable, 31 orphan + 1 policy-only, all dispositioned;
                       14 ground-truth controls reproduced)
```

**(d) THE MESSAGE WAS NOT TOUCHED.** `git diff --stat` carries no change to
`scripts/check_generated_reproducibility.sh`.

⛔⛔ **UNSOUGHT, AND THE BIGGER FINDING: FIXING THE READER EXPOSED THAT FOUR LIVE SURFACES PUBLISH A
ZERO AUTOMATIC TIER THAT HAS BEEN 14 SINCE 2026-07-30.** Removing the false `git-hook` edge left
**zero** targets in that class — correct — and made it obvious that the 14 remaining AUTOMATIC
targets are all `ci-workflow-auto`. Cause: `DONE-BAR.4` (director-approved 2026-07-30) enabled
`push:` on the three lanes needing no `generated/` regeneration, so **4 of 15** workflows
auto-trigger, not 1. The instrument was right on every run; four PROSE COPIES of its answer were
wrong. Two were inside this commit's blast radius and are corrected here (this script's own R2
comment; `docs/book/src/gate-flow.md` §6, whose table this leaf had to edit anyway); the remaining
copies are routed as `.36`. ⇒ this is a fresh instance of a rule already written down —
`docs/DERIVED_STATE_CONTAINMENT.md` **R1** (a derivable-exact field must not be hand-written) and
**R3** (carry the derivation, not the value) — which is why `.36` acceptance (b) prefers a pointer
to `--report` over a re-measured figure.

#### Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — `bash scripts/check_gate_reachability.sh --report | grep
  generated_reproducibility` → `generated_reproducibility_gate               git-hook`, i.e. a target
  invoked by nothing is certified AUTOMATIC. Attempting the honest register row failed the two-sided
  ratchet ("1 register entr(ies) name a target that is no longer orphaned").
- [x] **ROOT CAUSE (WHY + WHERE)** — **WHERE:** `scripts/check_gate_reachability.sh`
  `command_segments()`, which scanned **line by line** and neutralised only *single*-quoted literals
  (the line-local `SQ` regex). **WHY:** the third physical line of
  `check_generated_reproducibility.sh:266-268`'s multi-line **double**-quoted `breach "…"` message
  therefore presented `make` as its command word. Established by CAUSAL REPLAY, not by reading — the
  live reader was `sed`-mutated back to its pre-fix form and the arm re-run:
  ```text
  GATE-REACHABILITY-PROBE: arms=8 red_arms_fired=6 verdict=ALL-EXPECTED
    RED-1a … control fired: RED — a target named only in a multi-line error message is NOT an invocation
  ```
  i.e. with `data = set()` the target IS extracted and with the context scanner it is not, with
  nothing else changed. ⛔ The instrument is new, so its token `GATE-REACHABILITY-PROBE:` is
  registered in `scripts/check_diagnosis_evidence.sh` `DIAGNOSIS_SIG` **and** `TOOLBOX.md`'s family-5
  row **in this same commit** — the two-way obligation TOOLBOX.md states; citing a tool that did not
  produce the diagnosis would have been the dishonest alternative. Ops/build-flow defect (family 5).
- [x] **FIX** — the minimal tier is the READER, never the message (clause (d)): a 4-context shell
  scanner suppressing only lines that BEGIN inside a quoted string or heredoc payload, plus
  `shell_syntax=False` for prose. Why no lower tier: a quote-parity counter is the cheaper design and
  was **measured wrong on 1 862 corpus lines**; deleting the target name from the message is refused
  by (d).
- [x] **ADDRESSED (verified)** — `generated_reproducibility_gate`: `git-hook` → `ORPHAN` →
  dispositioned. Full-report diff **exactly 1 target of 125** moved (probe arm `DIFF`, which
  re-derives it from `git show HEAD:` rather than trusting a saved file). Gate now exits 0:
  `gate-reachability: OK (125 targets; 93 reachable, 31 orphan + 1 policy-only, all dispositioned;
  14 ground-truth controls reproduced)`.
- [x] **NO REGRESSION** — all 8 pre-existing ground-truth controls reproduce (`ast_dump_contract_gate`
  ORPHAN, `clippy_on_rust_change` policy-only, `mdbook_docs_gate` / `branch_protection_contract_gate`
  / `ci_workflow_local_gate` / `json_parser_book_gate` reachable, both workflow trigger classes) plus
  the SOTA-policy required-checks control; 6 new arms GREEN and each proven RED under mutation
  (`GATE-REACHABILITY-PROBE: arms=8 red_arms_fired=6 verdict=ALL-EXPECTED`). No parser surface can
  have moved and it is MEASURED rather than argued —
  `generated-reproducibility: OK (10 artifacts unmoved, emission sources unmoved since 070ade5 —
  tier 2 last proved them **byte-identical** to HEAD)`. All 21 doctrines PASS with the change staged
  (`bash scripts/check_doctrines.sh`). Clippy N/A and deliberately not cited: zero Rust bytes —
  `git diff --stat` is confined to the two check scripts, the register, `TOOLBOX.md`, the two book
  chapters, the knowledge card, the probe pair and the trackers.
- [x] **LOCKSTEP** — `docs/book/src/gate-flow.md` §6 (tier table re-derived + the string-vs-command
  paragraph), `docs/book/src/operations-and-governance.md` (the seventh calibration answer),
  `TOOLBOX.md` family-5 row (the new instrument's token, same commit as the instrument),
  `docs/knowledge/a-check-whose-inputs-all-pass-has-not-been-tested.md` (the fixture half of the
  lesson — deduped into the existing card rather than forked), `KNOWLEDGE_MAP.md` regenerated,
  `CHANGES.md`, `DEVELOPMENT_NOTES.md`, `MEMORY.md`, `docs/TASK_TREE.md`.

---

### `.35` NEW `todo` — a make invocation inside a SINGLE-LINE command substitution on an assignment's right-hand side is invisible to the reachability reader (opened 2026-08-16 session #242 by `.34`, whose own control arm hit it)

**ROUTING EVIDENCE** (`ROUTING-EVIDENCE` doctrine):

- **The mechanism, located.** In `scripts/check_gate_reachability.sh` `command_segments()`, the
  leader-stripping loop pops any token matching `^[A-Za-z_][A-Za-z0-9_]*=`. For
  `out="$(make -C rust SHELL=/bin/bash x)"` the whole of `out="$(make` is ONE whitespace-delimited
  token, so it is popped as a variable assignment and the command word becomes **`-C`** — neither
  `make` nor a runner, so the segment is skipped entirely.
- **Measured, not assumed.** Token replay prints `segment: out="$(make -C rust SHELL=/bin/bash
  probe_synthetic_gate)" -> command word: -C`. It is **PRE-EXISTING**: `.34` changed nothing on this
  path, and the same replay against the pre-`.34` reader gives the same answer.
- **Population today: ZERO.**
  `grep -rnE '^[[:space:]]*[A-Za-z_][A-Za-z0-9_]*=.*\$\(.*\bmake\b'` over `.githooks/`,
  `scripts/check_*.sh`, `rust/scripts/*.sh`, `rust/Makefile` and `.github/workflows/*.yml` → **0
  hits**. ⇒ nothing is mis-classified today; this is a latent hole, which is why it was routed rather
  than folded into `.34`.
- **Failure direction is SAFE, and that is why it can wait.** A missed edge makes a reachable target
  look ORPHAN, and an untriaged orphan **fails the gate loudly**. The silent direction — a false
  edge — is what `.34` fixed.
- **Why not fixed in `.34`.** `.34`'s own routing evidence refuses side-effect edits to this parser
  ("five calibration defects … six different confident answers"), and that refusal binds on its
  author. The multi-line sibling `x="$(` … `)"` IS covered and is pinned by a `.34` control arm.

**Acceptance:** (a) make the leader-stripping loop split `NAME=` from a following `$(`/backtick so
the substitution body is scanned in command position, proven on all 14 ground-truth controls;
(b) a control arm pinning the single-line form, watched failing first; (c) re-diff the full report
and state how many targets move (expected 0 given the population, which makes (c) a *falsifiable*
prediction rather than a formality).

---

### `.36` NEW `todo` — four live surfaces publish "the AUTOMATIC tier is ZERO" and it has been 14 since `DONE-BAR.4` landed on 2026-07-30 (opened 2026-08-16 session #242 by `.34`, which exposed it while removing the last `git-hook` edge)

**ROUTING EVIDENCE** (`ROUTING-EVIDENCE` doctrine):

- **The measurement.** `scripts/check_gate_reachability.sh --json` → 14 targets carry an AUTOMATIC
  invoker class, every one of them `ci-workflow-auto`: `branch_protection_contract_gate`,
  `fixed_point_gate`, `mdbook_docs_gate`, `parser_books_gate` and the ten per-parser book gates.
  Reading each workflow's `on:` block directly: **4 of 15** auto-trigger
  (`branch-protection-contract-gate`, `fixed-point-gate`, `mdbook-docs-gate` on `push`;
  `memory-architecture-gate` on `push` + `pull_request`), not 1.
- **Cause, and it is not a defect — it is an executed director decision.** Each of the three carries
  an in-file rationale: *"AUTOMATIC LANE (DONE-BAR.4, director-approved 2026-07-30 session #227) …
  Enabled to move the AUTOMATIC tier over the make gate targets off ZERO."* The wiring landed; the
  prose describing it did not move.
- **The stale copies, enumerated.** (1) `.github/workflows/memory-architecture-gate.yml` header —
  *"This is the ONLY workflow still triggered by `push` + `pull_request`; the other 14 are
  `workflow_dispatch`-only"*; (2) `rust/test_data/grammar_quality/gate_reachability_register_v0.json`
  `measured_context` + `honest_limit` — *"the AUTOMATIC tier over these gate targets is ZERO"*,
  *"with the AUTOMATIC tier at zero, wiring moves a target from ORPHAN to OPERATOR and makes nothing
  run"*, and its escalation *"the lever … is RESUMING HOSTED AUTO-TRIGGERS … a director decision"* —
  which was **already taken**; (3) `scripts/check_gate_reachability.sh` R2 comment ✅ FIXED in `.34`;
  (4) `docs/book/src/gate-flow.md` §6 tier table + callout ✅ FIXED in `.34`.
- **Direction of the error: UNDERSTATING coverage.** Safer than the reverse, but it is still a false
  published claim, and it makes a discharged escalation read as open — the register invites a
  director call that was answered 17 days earlier.
- **The general trap, which is the reusable half.** The instrument DERIVES the split on every run and
  was right throughout; what rotted were hand-written copies of its answer. Same class as
  `CORPUS_FAMILY_PROVENANCE` (`ENGINE-UNIVERSAL-SERVICES.26`) and the `2.741/8.9` pair.

**Acceptance:** (a) correct copies (1) and (2), and re-word the register's escalation to state that
the cheap-subset lever was EXERCISED by `DONE-BAR.4` and what remains unexercised; (b) decide whether
the register's `measured_context` should carry a derived figure at all, or only a pointer to
`--report` — prefer the pointer, per `docs/DERIVED_STATE_CONTAINMENT.md` R1/R3; (c) sweep for further
copies of "automatic tier is zero" across tracked surfaces and report the count found, so this is a
census rather than a spot fix.

### `.37` NEW `todo` (⛔ PARKED — the make lane is CLOSED by director ordering 2026-08-15) — the whole-second mtime trap also sits on the `sources → GENERATOR BINARY` edge, which `.32`'s sweep never censused, and it silently generated two artifacts from a stale emitter (opened 2026-08-17 session #243 by `ENGINE-UNIVERSAL-SERVICES.31` slice 2, which hit it mid-regeneration)

**ROUTING EVIDENCE** (`ROUTING-EVIDENCE` doctrine):

- **What happened, in one run.** `.31` slice 2 edited the emitter
  (`rust/src/ast_pipeline/ast_based_generator.rs`) and ran
  `make -C rust regenerate_generated_parsers`. The target exited **0** and printed
  `SOTA parser generated:` for all nine artifacts — and **two of them,
  `return_annotation_parser.rs` and `semantic_annotation_parser.rs`, came out byte-identical to the
  PRE-edit emission**, because `$(RUST_AST_PIPELINE_BOOTSTRAP)` was never rebuilt.
- **The mechanism, measured to the millisecond** (not inferred):

  ```text
  prerequisite  rust/src/ast_pipeline/ast_based_generator.rs   1786965923.233193
  target        rust/target/debug/ast_pipeline_bootstrap       1786965923.105944
                                                               ------------------
  prerequisite is 127.2 ms NEWER; GNU Make 3.81 compares WHOLE SECONDS,
  so it sees 1786965923 vs 1786965923 = EQUAL and skips the rule.
  ```
  `make -C rust -n target/debug/ast_pipeline_bootstrap` → `` `target/debug/ast_pipeline_bootstrap' is
  up to date`` against a tree where `find rust/src -name '*.rs' -newer
  rust/target/debug/ast_pipeline_bootstrap` names that exact file. `/usr/bin/make --version` →
  **GNU Make 3.81**.
- **Why `.32`'s sweep did not cover it.** `.32`(d) censused the `grammar → json` and `json → parser`
  edges and wired `scripts/make_freshness_guard.sh` onto them. The edge here is
  `$(AST_PIPELINE_SOURCES) → $(RUST_AST_PIPELINE_BOOTSTRAP)` (rule at `rust/Makefile:215`) and its
  sibling `→ $(RUST_AST_PIPELINE)` (`:241`) — the rules that build the GENERATOR itself. They carry
  no guard, and the census
  (`docs/tasks/artifacts/ci_parity_gate_rot/make_freshness_window_census.txt`) does not list them.
- **Reproduces outside SystemVerilog: yes, and it is family-independent by construction.** The edge
  is the generator binary, so every family generated after a same-second edit inherits the stale
  emitter. In this instance the eight families escaped only by luck: `$(RUST_AST_PIPELINE)` also
  depends on the annotation pair, which had just been rewritten, so *that* rule had a genuinely
  newer prerequisite and rebuilt. ⛔ Nothing about that is a guarantee.
- **The gap a driver must beat is SMALLER here than on the grammar edges.** `.32`(d)'s own finding
  was that the window is the work AFTER the target is written; for a `cargo build` the target is
  written at the very end of a multi-second link, so the exposed window is essentially the whole
  edit — which is why an agent loop (edit, build, regenerate, in one second) hits it and a human
  typing at a prompt does not.
- **Failure direction: PASSING, and silent.** `make` exits 0, the log is indistinguishable from a
  real run (generation has been QUIET since `.31`), and the artifact is well-formed. The only tell
  was an **mtime/sha assertion against an independently-generated arm** —
  [[feedback_verify_sv_parser_regen_mtime]] again, on a new edge.
- **Blast radius beyond `make`: it can also defeat a doctrine.** `GENERATED-REPRODUCIBILITY` tier 2
  re-derives the eight family artifacts with the tree's own `rust/target/debug/ast_pipeline`,
  checking only its FEATURE SURFACE and never that it is current with HEAD — so a stale generator
  that produced the artifacts also produces the re-derivation, and the gate passes by construction.
  ⛔ That half is **not** parked: it is a soundness gap in a registered doctrine on the SV release
  path, and it is owned by `ENGINE-UNIVERSAL-SERVICES.32` in the ACTIVE lane.

**Workaround until this is worked** (used by `.31` slice 2, and the only one proven here): after
editing any emission source, `rm -f rust/target/debug/ast_pipeline_bootstrap
rust/target/debug/ast_pipeline` before invoking `make`, then assert the result — compare the
regenerated artifacts against an arm generated by a binary you built directly, never against the
target's exit code.

**Acceptance:** (a) extend the `.32` census to the `sources → generator binary` edges and publish
the count found, so this is a census rather than a spot fix; (b) wire
`scripts/make_freshness_guard.sh` (or the `focus_scratch`-style unconditional `rm`) onto those
rules, priced — a generator rebuild is minutes, so an unconditional `rm` is NOT free here and the
exact-window guard is the right shape; (c) state in `TOOLBOX.md` that `make`'s exit 0 is not
evidence a generator was rebuilt, next to the existing grammar-edge warning.

### `.38` NEW `todo` (⛔ PARKED — the make lane is CLOSED by director ordering 2026-08-15) — **eighteen** hand-spelled copies of the generator recipe live in the gate tier, eleven of them standing in for a SHIPPED parser, and the Makefile holds three copies of the flag list itself (opened 2026-08-17 session #244 by `ENGINE-UNIVERSAL-SERVICES.33` slice 1, whose acceptance (c) census produced the population)

**ROUTING EVIDENCE** (`ROUTING-EVIDENCE` doctrine):

- **The census is an instrument, re-runnable, not a count**:
  `docs/tasks/artifacts/engine_universal_services/es33_makefile_mirror/census.sh` (+ `census.txt`).
  Derived at `9f856ac6`:

  | | |
  |---|---:|
  | homes for the flag list inside `rust/Makefile` itself | **3** at the census, ✅ **2 since `ENGINE-UNIVERSAL-SERVICES.16`** (`:122`, `:123`, and `:980`'s inline `generated/ebnf.rs` seed — the third is gone) |
  | hand-spelled generator invocations in `scripts/` + `rust/scripts/` + `.githooks/` | **18** |
  | — bucket **M**: feeds its parser back through `PGEN_<FAMILY>_PARSER_PATH`, so it stands in for a SHIPPED parser | **11** in 10 files |
  | — bucket **S**: own-artifact probe, drift changes what the gate measured not what ships | **7** in 4 files |
  | DIFFERING from the shipped recipe | **5** |
  | — differing ONLY by the inert `--eliminate-left-recursion` (`ENGINE-UNIVERSAL-SERVICES.34`) | **5** |
  | — differing in a way that COULD change emission | **0** |

- ⛔ **AT RISK, NOT CURRENTLY WRONG — and the distinction is measured, not hedged.** Every difference
  found is the inert flag, so no gate in the population is measuring a parser the project does not
  ship *today*. The hazard is the ordinary one: the next flag added to `RUST_GENERATOR` is real, and
  eighteen sites keep their own copy of the list that would have to be edited with it.
- **Failure direction: PASSING, and specifically for bucket M.** A bucket-M gate generates a parser,
  substitutes it as the family's via `PGEN_<FAMILY>_PARSER_PATH`, builds against it and reports a
  verdict *about the family*. If its flag list falls behind `RUST_GENERATOR`, that verdict is about a
  parser the project does not ship — and it reports GREEN, because the parser it built is internally
  consistent. Nothing compares it to the shipping recipe.
- **The ten bucket-M files**: `ebnf_frontend_dual_run_diff_gate.sh`, `ebnf_frontend_readiness_gate.sh`,
  `ebnf_stimuli_quality_gate.sh` (×2), `hdl_frontend_readiness_gate.sh`,
  `sv_external_corpus_triage_gate.sh`, `sv_preprocessor_quality_gate.sh`,
  `sv_semantic_scope_contract_gate.sh`, `sv_stimuli_quality_gate.sh`,
  `vhdl_external_corpus_triage_gate.sh`, `vhdl_stimuli_quality_gate.sh` — all under `rust/scripts/`.
- **Reproduces outside SystemVerilog: yes, and that is the point** — the population spans ebnf, hdl,
  sv, svpp and vhdl. The recipe is engine-universal, so this is not an SV finding.
- ⭐ **ONE site is already fixed and is the proof the shape is fixable cheaply**:
  `scripts/check_generated_reproducibility.sh` went from 2 hand-spelled lists to **0** by reading
  `RUST_GENERATOR` / `RUST_GENERATOR_BOOTSTRAP` out of `rust/Makefile` and refusing on any shape it
  cannot resolve — `ENGINE-UNIVERSAL-SERVICES.33` slice 1, 18/18 self-test arms. The derivation is
  ~30 lines of shell and costs nothing per run.
- ✅ **THE MAKEFILE'S OWN THIRD COPY IS GONE (2026-08-17, `ENGINE-UNIVERSAL-SERVICES.16` slice 1).**
  `rust/Makefile:980` spelled `--generate-parser --bootstrap-mode --eliminate-left-recursion` inline
  because it needs `$(RUST_AST_PIPELINE)` in bootstrap MODE, so it could not use
  `$(RUST_GENERATOR_BOOTSTRAP)`, which names the bootstrap *binary*. `.33` had priced the general case
  (a resolver for make variables) and declined it; the fix that landed is the cheap one this row
  itself called *"the obvious shape … un-priced"* — split the FLAGS out of the binary. All three
  spellings now reference `GENERATOR_FLAGS_BOOTSTRAP` / `GENERATOR_FLAGS`. ⛔ Note what did NOT
  change: the general make-expansion resolver is still refused, and a composed variable of any other
  shape is a REFUSAL rather than a silent mis-read.

**Acceptance:** ✅ **(a) DISCHARGED 2026-08-17 session #244 by `ENGINE-UNIVERSAL-SERVICES.16` slice 1**
(`PGEN-ENGINE-UNIVERSAL-SERVICES-0077`) — factor the flag list into ONE Makefile variable the three
Makefile spellings all reference, so `rust/Makefile` holds one home rather than three. Shipped as
`GENERATOR_FLAGS` / `GENERATOR_FLAGS_BOOTSTRAP`, with `RUST_GENERATOR` / `RUST_GENERATOR_BOOTSTRAP`
composed from them and the `generated/ebnf.rs` seed referencing the bootstrap list directly; census
re-run **3 homes → 2**, and `make -p` confirms both composed variables expand to byte-identical
command lines. ⭐ It was done there rather than here because `.16` REQUIRED it: the seed recipe is the
third copy, and covering `generated/ebnf.rs` in `GENERATED-REPRODUCIBILITY` meant deriving its flags.
⛔ `scripts/check_generated_reproducibility.sh` now also asserts each composed variable is exactly
`<binary> $(<flag-variable>)`, so re-inlining the flags is a refusal rather than a silent divergence; (b) for the
**11 bucket-M** invocations, either derive the flags from the Makefile (the `.33` shape, which is
`sed` + a refusal) or state per-gate why its parser need not match what ships, and gate the residue;
(c) for the **7 bucket-S** invocations, publish the disposition rather than fixing them — a
probe-only recipe may legitimately differ, and an exemption is only safe once it is written down;
(d) fire a RED arm per fixed site, because `.33` slice 1's own new arm found a bug that had made
**seven** sibling arms pass for an accidental reason; (e) re-run `census.sh` and show the counts move,
so this closes on a measurement rather than on a claim.

---

### `.39` NEW `todo` (⛔ PARKED — the make lane is CLOSED by director ordering 2026-08-15) — `GATE-REACHABILITY` is GREEN over a gate whose only callers are multi-hour aggregates, so it reported a REGRESSION ~2.5 weeks late (routed in 2026-08-18 session #244 by `SV-CORPUS-GRAD.13c.2i`(d))

**ROUTING EVIDENCE** (`ROUTING-EVIDENCE` doctrine — what was MEASURED before routing, and whether it
reproduces outside the family it is being sent to):

- **The incident, measured.** `ebnf_frontend_dual_run_gate` was RED on `systemverilog`
  (`151 > ceiling 150`) from **2026-08-09** — the commit `019e1739` that added the 151st
  divergence — until it was found on **2026-08-18** by a slice that ran the gate for an unrelated
  lockstep reason. Nine days by commit date; the last SV grammar edit before that was 2026-08-12,
  so several grammar-changing commits landed over a RED two-sided ratchet.
- ⛔ **`GATE-REACHABILITY` was GREEN throughout, and correctly so.** Its rule is that every tracked
  gate target is invoked by something that RUNS. ⚠️ **The first write-up of this row said "both
  callers" and there are THREE**, re-derived across `*.sh`, `Makefile` and `.github/workflows/`:

  | invoker | what it runs | trigger |
  |---|---|---|
  | `rust/scripts/sota_exit_gate.sh:1116` | the STRICT gate | operator / `workflow_dispatch:` |
  | `rust/scripts/regex_parser_family_contract_gate.sh:202` | the STRICT gate (`PGEN_EBNF_DUAL_RUN_STRICT=1`) | operator |
  | `.github/workflows/ebnf-frontend-dual-run-diff.yml:42` | ⛔ `ebnf_frontend_dual_run_diff` — the **REPORT-ONLY** target | `workflow_dispatch:` |

  ⭐⭐ **The third row is the sharpest fact and the first write-up missed it entirely: the workflow
  NAMED AFTER THE GATE does not run the gate.** It runs the report-only sibling, so even a dispatched
  run would not have failed on the ceiling breach — the `_gate` string in that file is only an
  artifact-upload path. ⛔ And **both** workflows are `workflow_dispatch:` only — no `push`,
  `pull_request` or `schedule` trigger — so all three invokers are operator-driven, and
  `sota_exit_gate` is the flagship, measured in `MEMORY.md` as *"not re-proven end-to-end since
  `CI-PARITY-GATE-ROT.7`"*. ⇒ **reachable by an aggregate nobody runs is not watched**, and the
  doctrine cannot see the difference because it measures the call graph, not the call FREQUENCY.
- **Reproduces outside this gate: NOT MEASURED, and that is the leaf's first job.** The same shape
  applies to every target whose only callers are `sota_exit_gate` / a family contract gate, and the
  reachability register already enumerates the call edges — so the census is a join, not a new
  instrument.
- ⚠️ **This is NOT an argument for running everything on every commit.** `sota_exit_gate` is hours;
  `CI-PARITY-GATE-ROT`'s own history is full of gates that taught bypasses by being too expensive
  for the moment they fired. The question is which *cheap* members of an expensive aggregate deserve
  promotion to the automatic tier — `ebnf_frontend_dual_run_gate` is **101 s** with a warm build,
  measured on the run that closed `.13c.2i`.

**Acceptance:** (a) census every tracked gate target by its callers' TIER (automatic / cheap-operator
/ heavy-aggregate), joining the reachability register against the doctrine driver and the workflows —
derived, never hand-listed; (b) for each target whose only tier is *heavy aggregate*, record its
measured wall-clock cost, because that is the number that decides promotion; (c) propose a promotion
set with its total cost, and ⛔ price it against the automatic tier's current budget rather than
asserting it is affordable; (d) state the honest bound: this closes the *reporting-latency* half of
`GATE-REACHABILITY`, not the coverage half, and the two are different properties.
