# BIN-BUILD-INTEGRITY — every tracked binary must still COMPILE

- **Status:** `done` (created 2026-07-22, session #196, from a
  `SV-CORPUS-GRAD.8c.3` compile-time discovery; all five leaves closed
  2026-07-23, session #197)
- **Family:** platform build/ops integrity (parser-agnostic)
- **Why this tree exists:** a tracked `[[bin]]` target
  (`ebnf_dual_run_diff`) has not compiled since the `-0212` result-carrier
  slimming (2026-07-21) and NOTHING caught it. The breakage was found only
  because `.8c.3` happened to run `cargo check --bins` with both features
  while validating an unrelated change. A binary that no longer builds is
  a silent capability loss: the tool is simply unavailable the day someone
  reaches for it, and the failure surfaces as "the diff tool is gone"
  rather than "a type migration missed a call site."

## Root cause of the SILENCE (the systemic half)

The batteries build what they need, not what the repo ships:

- the lib battery builds `--lib`/`--tests`, not `--bins`;
- the clippy flow lints the source tree, but its build graph likewise does
  not force every `[[bin]]` under every feature combination;
- `ebnf_dual_run_diff` is `required-features = ["ebnf_dual_run"]`, i.e. it
  is EXCLUDED from every default-feature build; it only compiles when a
  command explicitly opts into that feature AND asks for bins.

So the intersection "bins × non-default features" is a build-coverage hole
that no gate covered. Any `required-features` binary can rot in it.

## Leaves

### `.1` — Repair `ebnf_dual_run_diff` (the `-0212` migration miss)

- **Status: `done`** (`PGEN-BIN-BUILD-INTEGRITY-0001`, session #196).
- **ROOT CAUSE (WHY + WHERE):** `PGEN-RGX-0078-0212` (the ParseNode
  72 → 48 B result-carrier slimming, 2026-07-21) changed `ParseNode.span`
  to `Span { start: u32, end: u32 }`
  (`rust/src/ast_pipeline/mod.rs:1177`). Every in-tree consumer was
  migrated to the house idiom `node.span.start as usize`
  (e.g. `parse_harness_interpreter.rs:1039/1147`) EXCEPT this binary,
  which is invisible to default-feature builds. Compiler-pinned:
  `error[E0308] expected usize, found u32` at
  `rust/src/bin/ebnf_dual_run_diff.rs:167` and `:168`, where the `u32`
  span fields are assigned into `ParseAttempt`'s
  `span_start`/`span_end: Option<usize>` report fields.
- **FIX (level: source call-site, the narrowest level that works):** apply
  the same `as usize` conversion at the two construction sites. The report
  struct's `Option<usize>` fields are DELIBERATELY left as-is so the
  emitted JSON report contract stays byte-stable and stays consistent with
  its sibling `error_position: Option<usize>` (serde emits an identical
  number either way; changing the field types would have been a wider,
  contract-touching change for no gain).
- **ACCEPTANCE CHECKLIST:**
  - [x] **ROOT CAUSE (WHY + WHERE)** — compiler-pinned above (E0308 ×2,
    exact file:line, exact type migration, exact reason the hole exists:
    `required-features` bins are excluded from default builds).
  - [x] **ADDRESSED (verified, before → after)** — before:
    `cargo check --features "generated_parsers ebnf_dual_run" --bins --lib`
    fails, `could not compile pgen (bin "ebnf_dual_run_diff") due to 2
    previous errors`. After: the same command exits 0 with ALL binaries
    building. FUNCTIONAL proof (not just compile-proof): the rebuilt
    binary runs end-to-end on `grammars/json.ebnf` and its report carries
    the repaired fields with correct values — `parse_full.ok=true`,
    `root_rule=grammar_file`, `span_start=0`, `span_end=1013` over a
    1,014-byte input.
  - [x] **NO REGRESSION** — the full-feature `--bins --lib` check is green
    across every tracked binary (the census, not just the repaired one);
    the repaired binary runs and emits a well-formed report;
    `clippy_on_rust_change` re-run for this change: source-strict stage
    **0 errors**, generated-parser stage debt **291 unchanged** and still
    entirely under `generated/*`; lib battery unaffected — no library code
    was touched (the change is two `as usize` casts inside one bin, so it
    cannot reach any other target).
  - ⚠️ **HONEST SCOPE NOTE — `ebnf_frontend_dual_run_gate` does NOT pass,
    for a PRE-EXISTING reason this leaf does not own** (see `.3`). It
    fails at its own bootstrap step, before the repaired binary is ever
    reached. Proven independent of this change AND of
    `PGEN-SV-CORPUS-GRAD-0015`: the gate builds `ast_pipeline` with
    `--features ebnf_dual_run` ONLY
    (`rust/scripts/ebnf_frontend_dual_run_diff_gate.sh:77`), and since
    commit `200cae5b` (2026-07-15) generating a parser for an
    annotation-bearing grammar through a binary lacking
    `generated_parsers` is REFUSED by design; the very same binary
    completes that exact step at exit 0 under the documented
    `PGEN_ALLOW_BOOTSTRAP_ANNOTATION_FALLBACK=1` override — an
    annotation-backend policy refusal, not a compile or runtime defect.
  - [x] **LOCKSTEP** — this tree + TASK_TREE index + CHANGES /
    DEVELOPMENT_NOTES / MEMORY this commit. No book/contract surface: the
    binary's CLI and JSON report shape are unchanged by construction.

### `.2` — Close the build-coverage hole (a gate that builds EVERY binary)

- **Status: `done`** (`PGEN-BIN-BUILD-INTEGRITY-0007`, session #197,
  2026-07-23) — the systemic fix; the whole tree's raison d'être.
- **What landed:** a new maintained gate
  `rust/scripts/bin_build_integrity_gate.sh` +
  `make -C rust bin_build_integrity_gate`, doing two mechanical things:
  1. **A coverage PROOF, first, before any build time is spent.** The
     binary census is derived from `cargo metadata --no-deps` (NOT a
     hand-list — that is what stops the census silently drifting out from
     under the gate), and the gate asserts every declared binary's
     `required-features` are satisfied by at least one planned
     configuration. A new binary nobody planned for FAILS the gate by name,
     with the exact remedy printed.
  2. **`cargo check --all-targets` per configuration** — bins + lib +
     tests + benches + examples, so a feature-gated test module (the `.5`
     class) is swept by the same pass as a `required-features` bin (the
     `.1` class).
- **The configuration plan (5, each one the repo genuinely builds):**
  `default` (feature `normal`); `bootstrap`
  (`--no-default-features --features bootstrap`, the Makefile's
  `ast_pipeline_bootstrap`); `generated_parsers` (the `focus_*` regen
  path); `ebnf_dual_run` (the `.ebnf` frontend path); and
  `generated_parsers ebnf_dual_run` (the documented dual-feature toolbox
  binary). The plan lives in the script's `CONFIG_*` arrays with a
  per-configuration "why the repo builds this" comment, so a configuration
  that ceases to exist is removed rather than left to rot.
- **Why a SET of configurations, not one union build (decided, not
  defaulted):** `--all-features` is INVALID here (`mimalloc_perf` and
  `never_free_arena_perf` both install a `#[global_allocator]`), and a
  union build hides exactly the failure mode that rotted — code reachable
  only under ONE feature. `.5` proved this twice: a cfg-gated test
  appears/disappears with the feature set, so a union check can run 9/10
  and report ok. The plan checks configurations the repo actually issues,
  so each MUST compile.
- **Not wired into `ci_workflow_local_gate` after all (the `.1` charter's
  guess, corrected on cost evidence):** the full 5-configuration check
  measured **~2 min for the dual configuration ALONE** and distinct
  feature sets share no build artifacts, so folding it into that
  frequently-run parity gate would make the common gate heavy. It is a
  standalone maintained gate run under the memory guard instead — the
  honest home for a heavy job.
- **⭐ The gate's open question is now ANSWERED and CLOSED by `.5`:** yes,
  cfg-gated TEST modules rot in the same hole; the `--all-targets` sweep
  covers them, and `.5` is the worked instance.
- **ACCEPTANCE CHECKLIST:**
  - [x] **REPRODUCE / ISSUE** — the hole is demonstrated by its three
    victims (`.1`/`.3`/`.5`), each a target that stopped compiling/running
    with every battery still green; before this leaf, no maintained gate
    built `--bins` or `--all-targets` under the non-default feature
    combinations.
  - [x] **ROOT CAUSE (WHY + WHERE)** — the systemic root cause stated in
    this tree's header: `--lib`/`--tests` × default features is a strict
    subset of the shipped surface (bins excluded always; `required-features`
    bins and `#![cfg(feature)]` tests excluded under default features).
  - [x] **FIX** — the gate above; a CHECK not a build (cheap-as-possible
    for a heavy job), honest about feature combinations rather than the
    union, run under the memory guard.
  - [x] **ADDRESSED (verified, guarded)** — `make -C rust
    bin_build_integrity_gate` exits 0: coverage proof green (**19 binaries,
    `uncovered_bins: []`** — every declared binary, including the `.1`
    victim `ebnf_dual_run_diff`, covered by ≥1 configuration), and all **5
    configurations pass** (`default`, `bootstrap`, `generated_parsers`,
    `ebnf_dual_run`, `generated_parsers+ebnf_dual_run`). ⭐ Earning this box
    caught a real bug in the gate's OWN first draft: the dual configuration's
    feature flags were space-separated in the plan string, which word-splits
    to `--features generated_parsers ebnf_dual_run` and makes cargo read
    `ebnf_dual_run` as a stray positional (`error: unexpected argument
    'ebnf_dual_run' found`) — the gate correctly reported that configuration
    as failing-to-compile; fixed by comma-joining the feature set
    (`generated_parsers,ebnf_dual_run`, one argv token), re-run green. That
    the gate flagged its own misconfiguration as a compile failure is the
    strongest evidence it does what it claims.
  - [x] **NO REGRESSION** — the gate is purely additive (a new script + a
    new `make` target + a help line); it touches no Rust, grammar,
    generated, or contract surface, so no parser, release, schema, or ledger
    can move. Determinism: the coverage proof is a pure function of `cargo
    metadata` (sorted output), and `cargo check` is deterministic on a fixed
    tree. Cost (honest): fast on a warm tree (`cargo check`), heavy on a cold
    one — the dual configuration alone measured ~2 min cold, and the five
    feature sets share no build artifacts; run under the memory guard, not
    in pre-commit.
  - [x] **LOCKSTEP** — new gate documented in the book (operations &
    governance → Build Integrity), Makefile help line, tree + TASK_TREE +
    CHANGES / DEVELOPMENT_NOTES / MEMORY this commit.

### `.3` — `ebnf_frontend_dual_run_gate` has been RED since 2026-07-15 (second instance of the same silence)

- **Status: `done`** (`PGEN-BIN-BUILD-INTEGRITY-0005`, session #197,
  2026-07-22) — discovered while verifying `.1` (session #196); the design
  call it owned is adjudicated below **on measurement, not preference**.
- **Symptom:** `make -C rust SHELL=/bin/bash ebnf_frontend_dual_run_gate`
  exits 1 at `==> Regenerating EBNF frontend artifacts for dual-run
  harness`, before the dual-run comparison ever runs.
- **ROOT CAUSE (tool-pinned, `bash -x` trace + git archaeology):** the
  gate builds its `ast_pipeline` with `--features ebnf_dual_run` only
  (`ebnf_frontend_dual_run_diff_gate.sh:77`), then asks that binary to
  `--generate-parser` from `grammars/ebnf.ebnf` — which carries return
  annotations. Commit `200cae5b` (2026-07-15, the RGX-0078.5.i.1.t1
  `null` → `"null"` drift incident) made exactly that combination a hard
  REFUSAL (bootstrap annotation surface may silently re-interpret
  constructs beyond its subset). The gate script itself has not been
  touched since 2026-04-06, so it never learned about the new refusal.
- **Why nobody noticed:** same class as `.1` — a maintained gate that no
  battery runs. It is not in `sota_exit_gate`'s critical path and the
  recent SV/regex campaigns had no reason to invoke it.
- **The design call this leaf owns (do NOT guess it in a hurry):** either
  (a) the gate builds its `ast_pipeline` with `generated_parsers` too (the
  canonical backend — matches how `focus_*` regen works, but changes what
  the "bootstrap" arm of a Perl-vs-Rust differential actually exercises),
  or (b) the gate sets `PGEN_ALLOW_BOOTSTRAP_ANNOTATION_FALLBACK=1`
  deliberately, with a comment stating that its bootstrap artifacts are
  non-canonical BY CONSTRUCTION and are compared, never trusted or
  shipped. (b) is the likely-correct reading of the gate's INTENT (it is a
  differential harness, not a regen path), but it must be adjudicated
  against the `200cae5b` incident's reasoning rather than chosen for
  convenience.
- **⭐ Standing risk this exposes (for the `.2` design):** two independent
  maintained surfaces (`ebnf_dual_run_diff` the binary, and the gate that
  drives it) both rotted silently in the SAME blind spot. `.2` should
  therefore cover "every maintained gate still RUNS", not merely "every
  binary still COMPILES".

#### The adjudication — **(a) build with `generated_parsers`**, decided on measurement

The `.3` charter above leaned toward (b) ("the likely-correct reading of
the gate's INTENT") but required it be adjudicated against the `200cae5b`
reasoning rather than chosen for convenience. Adjudicated: **(a)**.

**First, the measurement that removes "which output is right?" from the
question.** The same Perl-frontend JSON was fed to `--generate-parser`
twice — once through the canonical backend (the dual-feature binary), once
through the opted-in bootstrap fallback (`ebnf_dual_run`-only +
`PGEN_ALLOW_BOOTSTRAP_ANNOTATION_FALLBACK=1`) — writing to the SAME output
path both times, so the ~3k path-embedded diagnostic strings are
controlled. Result: **BYTE-IDENTICAL, sha256
`56cf0763c13bbbd4bfe3c02a52fe3fb9e342a3650bcb61307afbf6d13e3638c0`, both
12,445,288 B.** So (a) costs nothing measurable today; the choice is purely
about which one is SOUND to depend on tomorrow.

Four grounds, all checkable:

1. **A differential must vary exactly one thing.** This harness exists to
   compare the Perl frontend against the Rust frontend. A degradable
   annotation backend on the Rust arm is a SECOND, uncontrolled variable —
   a future divergence could then be the backend, not the frontend, and the
   gate could not tell you which.
2. **The fallback's own license does not cover a standing gate.** The
   refusal text licenses NON-CANONICAL artifacts "that MUST be re-derived
   canonically and pass `make -C rust parse_harness_equivalence_gate`
   before being trusted". A gate that runs unattended cannot satisfy that
   precondition on each run, so opting in inside the gate would breach the
   license's own terms while appearing to honor them.
3. **It restores the `200cae5b` record's stated invariant instead of
   carving an exception into it.** That decision record
   (`docs/decisions/project_bootstrap_annotation_fallback_loud_refusal.md`)
   justifies the refusal partly on "the `ebnf_dual_run`-only frontend
   binary performs only standalone raw-AST export (no annotation
   parsing)". That sentence is false for exactly ONE call site in the
   repository — this gate's `--generate-parser` step. (a) makes the
   sentence true again; (b) would make it permanently false.
4. **⭐ It also removes a documented recurring trap.** `TOOLBOX.md` defines
   `rust/target/debug/ast_pipeline` as the DUAL-feature binary, but the
   gate was overwriting that shared path with a single-feature build — the
   trap `MEMORY.md` records being re-hit as recently as `-0015`. Verified
   after this change: the shared binary reports
   `AST-PIPELINE-FEATURE-SURFACE: ebnf_dual_run=true generated_parsers=true`
   once the gate finishes, so running the gate no longer degrades the
   toolbox.

Not a cold-clone hazard: `rust/build.rs` presence-gates EVERY generated
parser include behind its own `has_generated_*` cfg (`:90`–`:168`), so
`--features generated_parsers` compiles whether or not the artifacts exist.
Honest bound: in a PARTIALLY-populated mid-bootstrap tree the dual-feature
combination can still fail (`active_grammar_profile` is cfg-gated on
regex/SV presence, `parser_registry.rs:435`) — the documented cold-bootstrap
ordering trap, which the standard toolbox binary already shares, and not a
state in which a Perl-vs-Rust differential is meaningful.

Scope kept minimal: only the `ast_pipeline` build line changes. The
`ebnf_dual_run_diff` build stays single-feature deliberately — that binary
only PARSES grammar files (it never calls the pipeline's annotation
extraction), so it cannot reach the refusal, and widening its features
would add compile cost for nothing.

- **ACCEPTANCE CHECKLIST:**
  - [x] **REPRODUCE / ISSUE** — `make -C rust SHELL=/bin/bash
    ebnf_frontend_dual_run_gate` (guarded) exits 1 at
    `==> Regenerating EBNF frontend artifacts for dual-run harness`; with
    `.4` landed, the run now names the cause verbatim:
    `Error: REFUSED: return annotation '{type: "grammar_file", elements:
    [$1*]}' needs the generated annotation backend, but this binary was
    built WITHOUT `--features generated_parsers` …`.
  - [x] **ROOT CAUSE (WHY + WHERE)** — `ebnf_frontend_dual_run_diff_gate.sh`
    built its `ast_pipeline` with `--features ebnf_dual_run` only, then
    asked that binary to `--generate-parser` from the annotation-bearing
    `grammars/ebnf.ebnf` (`:92`); `200cae5b` (2026-07-15) made exactly that
    combination a hard refusal at
    `rust/src/ast_pipeline/mod.rs:3925` (`require_bootstrap_annotation_fallback_license`).
    The gate script had been untouched since 2026-04-06, so it never
    learned about the new refusal.
  - [x] **FIX** — build that binary with
    `--features "generated_parsers ebnf_dual_run"` (the canonical
    annotation backend, and the feature set `TOOLBOX.md` documents for this
    path), with the adjudication recorded in-script as a comment. One line
    of behavior; no Rust, grammar, or generated surface touched.
  - [x] **ADDRESSED (verified, before → after)** — before: exit 1 at the
    bootstrap step, no grammar ever compared. After: **exit 0, STRICT mode
    (`PGEN_EBNF_DUAL_RUN_STRICT=1`), all three tracked grammars pass** —
    `ebnf` 131/131 raw_ast parity, 23787/23788 bytes (100.00%);
    `json` 19/19 parity, 1013/1014 (99.90%); `regex` full parse parity,
    157306/157306 (100.00%); `✅ EBNF dual-run differential passed for all
    tracked grammars`. Those per-grammar numbers REPRODUCE the last
    known-green baseline recorded on 2026-06-25 (`ebnf` 131/131 parity,
    `json` 99.90%, `regex` 100%), i.e. the measurement itself is unchanged
    by the feature switch — the gate was restored, not redefined.
  - [x] **NO REGRESSION** — the gate is deterministic across two
    independent guarded runs: `summary.csv` and the generated
    `bootstrap_ebnf.rs` (12.4 MB) both **BYTE-IDENTICAL** run-1 vs run-2;
    the canonical-vs-fallback artifact A/B above is byte-identical, so the
    backend switch provably changes no emitted parser; the shared
    `ast_pipeline` is left DUAL-feature (trap removed, feature surface
    asserted); `bash -n` clean; no Rust/grammar/generated file touched ⇒ no
    parser, release, schema, or ledger can move. Guarded peak 1,434 MB /
    25 s.
  - [x] **LOCKSTEP** — `LIVE_ACHIEVEMENT_STATUS.md` KNOWN-RED tracker note
    resolved (the gate is a green proof surface again); tree + TASK_TREE
    index + CHANGES / DEVELOPMENT_NOTES / MEMORY this commit. No book
    change: `docs/book/src/grammar-wellformedness.md` describes what the
    gate proves (self-hosting over ebnf/json/regex), which is unchanged —
    only how its binary is built.

- **⭐ FINDING recorded while verifying (not a defect of this fix; no
  action taken):** the `regex` row passes as `perl_under_reports 25` — the
  PERL reference arm reports 251 unique rules where the Rust arm reports
  276. Root-caused rather than waved through: the 25 are a fixed
  construct-class blind spot of the legacy Perl frontend (long
  single-character alternation lists — `letter`/`digit`/`hex_digit`/
  `octal_digit`/`whitespace`/`special_char`; the embedded `code_*`
  code-block cohort; and `unicode_char`, the `!builtin_ascii_char`
  negative-lookahead rule), NOT truncation and NOT grammar growth: running
  the same Perl frontend over the 2026-06-25 vintage of `regex.ebnf`
  (202 rule definitions vs 275 today, 28 commits apart) yields the
  **identical 25 missing names**. The gate treats `perl_under_reports` as a
  PASS by design — the Rust frontend is a superset and is the sole
  direction of travel — but it means the differential's raw_ast leg is a
  weaker check for `regex` than the word "parity" suggests. Worth knowing
  before anyone leans on this gate as evidence for a regex-frontend claim.

### `.4` — The gate's own failure diagnostics are DEAD under `set -e`

- **Status: `done`** (`PGEN-BIN-BUILD-INTEGRITY-0004`, session #197,
  2026-07-22) — opened while reproducing `.3`; a THIRD instance of the same
  theme: a maintained surface that silently does not do what it claims.
- **Why it is its own leaf:** it is a different defect from `.3` (that one
  is *why* the gate fails; this one is *why nobody can see why*), and it is
  verifiable RIGHT NOW against the still-RED gate — so it lands first, and
  `.3`'s own evidence gets better because of it.
- **What was CLAIMED:** the 2026-04-06 CI-observability fix (recorded in
  `LIVE_ACHIEVEMENT_STATUS.md`, tracker note of that date) states the script
  "now prints the hidden log path and bounded head/tail excerpts whenever
  either bootstrap step fails, so future GitHub repros should expose the
  real underlying stderr directly in the primary job log". That capability
  has never once fired.
- **ROOT CAUSE (WHY + WHERE):** `rust/scripts/ebnf_frontend_dual_run_diff_gate.sh`
  runs under `set -euo pipefail` (`:2`). Inside the helper
  `run_logged_or_dump` (`:61`) the work was invoked UNGUARDED —
  `"$@" >"$log_path" 2>&1` followed by `local status=$?` (`:66`–`:67`).
  Under `errexit` a failing command in a function body whose CALL is not in
  a tested context aborts the shell AT that command, so `local status=$?`,
  `print_log_excerpt_on_failure`, and `return "$status"` are all
  unreachable: the diagnostic block is dead code.
  - Tool-pinned two ways. (1) The live RED gate (`.3`) printed only
    `==> Regenerating EBNF frontend artifacts for dual-run harness` then
    `make: *** [ebnf_frontend_dual_run_gate] Error 1`, while the real cause
    sat unread in `logs/bootstrap_generate_ebnf_parser.log`.
    (2) A minimal isolating probe of the exact shape (`set -euo pipefail` +
    the same helper + `false`) exits 1 without printing its `REPORT-RAN`
    marker — errexit, not a logic bug in the excerpt printer.
- **FIX (level: script, narrowest that works):** capture the status without
  arming errexit — `local status=0; "$@" >"$log_path" 2>&1 || status=$?` —
  and leave every other line untouched. The helper still RETURNS the real
  exit code, so the caller (an unguarded call under `set -e`) still fails
  fast at the same step with the same code; the only behavior change is
  that the already-written excerpt now reaches stderr.
- **SIBLING CENSUS (this is a lone defect, not a class):** every other
  `<var>=$?` capture across the gate surface is correctly guarded —
  `set +e`/`set -e` brackets (`regex_embedded_code_block_contract_gate.sh`
  `:154`/`:204`, `sv_preprocessor_reference_runner.sh` `:197`/`:200`),
  `if/else` arms (`sv_preprocessor_quality_gate.sh` `:876`/`:883`,
  `sv_preprocessor_curated_differential_gate.sh` `:168`,
  `sv_preprocessor_template_differential_gate.sh` `:371`,
  `sv_stimuli_quality_gate.sh` `:2910`/`:2917`,
  `sv_declared_shadow_promotion_gate.sh` `:269`,
  `sv_parse_full_ratio_promotion_gate.sh` `:271`,
  `vhdl_strict_promotion_gate.sh` `:255`), EXIT-trap handlers where `$?` is
  the trap's own status (`ci_workflow_local_gate.sh` `:47`,
  `vhdl_stimuli_quality_gate.sh` `:205`), an explicit `|| rc=$?`
  (`verilog_2005_conformance_gate.sh` `:193`), or a script that does not set
  `errexit` at all (`stimuli/run_external_corpus.sh` `:78`). 13 sibling
  sites inspected, 0 further defects.
- **ACCEPTANCE CHECKLIST:**
  - [x] **REPRODUCE / ISSUE** — `make -C rust SHELL=/bin/bash
    ebnf_frontend_dual_run_gate` (guarded) exits 1 printing NO cause: the
    `error: … failed with exit code …` line, the `log:` path, and the
    `--- begin … log ---` excerpt are all absent from the run output.
  - [x] **ROOT CAUSE (WHY + WHERE)** — `set -e` + unguarded
    `"$@" >"$log_path" 2>&1` at `ebnf_frontend_dual_run_diff_gate.sh:66`
    makes `:67`–`:73` unreachable; isolating bash probe reproduces the
    shape standalone (marker line never printed, exit 1).
  - [x] **ADDRESSED (verified, before → after)** — same command, same RED
    gate, same exit code 1, but the run now prints
    `error: bootstrap EBNF parser generation failed with exit code 1`, the
    `log:` path, and the full excerpt carrying the verbatim
    `Error: REFUSED: return annotation '{type: "grammar_file", elements:
    [$1*]}' needs the generated annotation backend …` line. The failure is
    self-explaining for the first time since the capability was claimed
    (2026-04-06).
  - [x] **NO REGRESSION** — `bash -n` clean; fail-fast preserved (the gate
    still stops at the same step with exit code 1, propagated through the
    helper's `return "$status"`); the pass path is untouched
    (`status` stays 0 ⇒ early `return 0`); no Rust/grammar/generated
    surface touched, so no parser, gate contract, or artifact can move.
    Sibling census above found no second instance to regress.
  - [x] **LOCKSTEP** — tree + TASK_TREE index + CHANGES /
    DEVELOPMENT_NOTES / MEMORY this commit. No book/contract surface: the
    gate's command, contract, and verdicts are unchanged — only its
    stderr on failure.

### `.5` — A `generated_parsers`-only INTEGRATION TEST has not compiled since 2026-07-12

- **Status: `in_progress`** (opened 2026-07-22, session #197) — found by
  `.2`'s own instrument before that gate had even landed, which is the
  strongest possible argument for the gate.
- **This ANSWERS `.2`'s open question** ("whether the same hole exists for
  `#[cfg(feature = ...)]` TEST modules") — **yes, measured, with an
  instance**: `rust/tests/auto_return_annotation_shape_gate_integration.rs`
  carries `#![cfg(feature = "generated_parsers")]` (`:26`), so it compiles
  to an empty crate under the default features that plain `cargo test`
  uses, while every battery that DOES enable `generated_parsers` builds
  `--lib` and never `--tests`. The intersection is empty: nothing in the
  repository compiles this file.
- **ROOT CAUSE (WHY + WHERE):** commit `961b1781` (2026-07-12,
  `PGEN-RGX-0078-0031`, the node-arena landing that bought regex −21.9%)
  gave every generated parser's constructor a second parameter —
  `pub fn new(input: &'input str, arena: &'input NodeArena<'input>, logger:
  Box<dyn Logger>)` (e.g. `generated/regex_parser.rs:1065`). In-tree
  callers were migrated to the house idiom (`parser_registry.rs:522`+:
  `let node_arena = crate::ast_pipeline::NodeArena::new();` then
  `&node_arena`), but this test file — last touched 2026-04-27 — was not.
  Compiler-pinned: **10 × `error[E0061]: this function takes 3 arguments
  but 2 arguments were supplied`, "argument #2 of type `&NodeArena<'_>` is
  missing"**, at `:86`, `:121`, `:149`, `:173`, `:194`, `:222`, `:241`,
  `:261`, `:290`, `:318` — one per grammar the gate covers.
- **Why this one matters more than a stale test:** the file is the
  per-grammar integration proof for the auto-generated return-annotation
  shape gate across TEN grammars (regex, return_annotation,
  semantic_annotation, rtl_const_expr, rtl_frontend, json, vhdl,
  systemverilog_preprocessor, systemverilog, ebnf). Ten days of "green"
  batteries never ran it once.
- **FIX (level: source call-site, the same narrowest level as `.1`):** apply
  the house idiom at each of the 10 sites — `let node_arena =
  pgen::ast_pipeline::NodeArena::new();` then pass `&node_arena` as
  argument 2, mirroring `parser_registry.rs:522`+ verbatim. The arena is
  constructed INSIDE each parse closure, so it outlives the parser and is
  dropped per sample; `to_json_value()` returns an owned value before the
  borrow ends. No test logic, sample, assertion, or logger channel changed —
  the diff is 10 arena constructions, 10 argument insertions, and the line
  wraps they force.
- **⭐ The tenth site needs BOTH features:** `auto_gate_ebnf_inventory_wide_shape`
  is additionally `#[cfg(feature = "ebnf_dual_run")]` (`:325`), so a
  `generated_parsers`-only run compiles and runs only 9 of the 10. That is
  a second, independent argument for `.2` checking CONFIGURATIONS rather
  than one union build — and it is why the verification below was run
  twice.
- **ACCEPTANCE CHECKLIST:**
  - [x] **REPRODUCE / ISSUE** — `cargo check --all-targets --features
    "generated_parsers ebnf_dual_run"` fails:
    `error: could not compile pgen (test
    "auto_return_annotation_shape_gate_integration") due to 10 previous
    errors` (guarded run, 126 s, peak 12,096 MB).
  - [x] **ROOT CAUSE (WHY + WHERE)** — compiler-pinned above: 10 ×
    `error[E0061] … argument #2 of type &NodeArena<'_> is missing`, each
    line named, against the constructor signature `961b1781` introduced on
    2026-07-12 (`generated/regex_parser.rs:1065`); the file's own
    `#![cfg(feature = "generated_parsers")]` (`:26`) plus the batteries'
    `--lib`-only habit is the exact reason nothing surfaced it.
  - [x] **ADDRESSED (verified, before → after)** — before: 10 compile
    errors, test never built. After, run TWICE: under
    `--features generated_parsers`, **9 passed / 0 failed** (the ebnf case
    correctly excluded by its own cfg); under
    `--features "generated_parsers ebnf_dual_run"`, **10 passed / 0
    failed** — every repaired call site executed, not merely compiled.
    Guarded (207 s / 10,839 MB and 216 s / 11,520 MB).
  - [x] **NO REGRESSION** — `clippy_on_rust_change` source-strict PASS
    (generated-stage tracked debt unchanged at 291, still entirely under
    `generated/`); no library, grammar, generated artifact, or contract
    surface touched — the change lives entirely inside one integration
    test, so no parser, release, schema, or ledger can move; the tests
    themselves are the oracle and they pass on real generated parsers
    across all ten grammars.
  - [x] **LOCKSTEP** — tree + TASK_TREE index + CHANGES /
    DEVELOPMENT_NOTES / MEMORY this commit. No book/contract surface: a
    test regained its ability to compile; no documented behavior changed.

### `.6` — ⭐⭐ A HAND-WRITTEN BINARY HAS NEVER BEEN TRACKED, and the gate's census cannot see it (routed in by `SV-CORPUS-GRAD.12c.1`, 2026-08-11)

- **Status: `todo`, opened 2026-08-11.** The one-file symptom is already FIXED in
  `SV-CORPUS-GRAD.12c.1` (that leaf had to wire this binary, so it could not leave it
  uncommittable). ⛔ What is NOT fixed, and is what this leaf owns, is the CLASS.
- **THE FINDING, measured before anything was changed.** `rust/src/bin/generated_parse_probe.rs`
  is a hand-written diagnostic binary — `git log --all -- <path>` returns **empty**, so it has
  **never been tracked** and no fresh clone has ever contained it. Cause:
  `.gitignore:231` carries `generated_*.rs`, a pattern with no `/`, which git therefore matches at
  **any depth**. Measured blast radius: **exactly one** file on disk outside `**/target/` matches
  it, and it is this one; the generated tree it was presumably written for is covered by
  `generated/` at `.gitignore:24` and none of that tree's files even start with `generated_`. ⇒ the
  pattern's only live effect was the harm.
- ⭐⭐ **WHY `.2`'s GATE CANNOT CATCH THIS, and this is the part worth keeping.** `.2` fixed the
  census-drift problem by deriving the binary list from `cargo metadata --no-deps` instead of a
  hand-list — the right call, and it holds. But `cargo metadata` reports the **WORKING TREE**:
  cargo auto-discovers `src/bin/*.rs`, so on this machine the census is **19 binaries including
  `generated_parse_probe`**, and the gate is perfectly green. On a fresh clone the file is absent,
  cargo reports **18**, and the gate is *still* perfectly green — it simply never hears about the
  binary it should be building. ⛔ **A census derived from the working tree cannot detect a file
  the working tree has and git does not.** That is a different axis from the one `.2` closed, and
  it is invisible in exactly the direction that looks healthy.
- **THE SHAPE OF THE REAL FIX** (deliberately not taken here, because it is a repo-wide policy
  question and not an SV-release one): assert that every path `cargo metadata` reports as a target
  source is TRACKED (`git ls-files --error-unmatch`), which is a cheap, deterministic, feature-free
  check that runs in milliseconds and needs no build. ⚠️ Consider also whether the file should
  simply be RENAMED out of the artifact namespace — it is a hand-written tool wearing a generated
  artifact's name, and the negation now in `.gitignore` leaves the trap armed for the next one.
- **WHAT `.12c.1` DID** — the minimum that made its own deliverable true, no more: a targeted
  `!rust/src/bin/generated_parse_probe.rs` negation with the measurement recorded inline, so the
  file is committable and the "all 7 user-source readers decode" claim is actually true in git.

## Acceptance Criteria (tree)

1. Every tracked `[[bin]]` compiles under the feature combination it
   declares (`.1` proved the census; `.2` makes it standing). ✅
2. The proof is mechanical and part of a maintained gate, not a manual
   `cargo check` someone happens to run (`.2` — the gate derives the census
   from `cargo metadata` and checks `--all-targets` per configuration). ✅
3. **(Learned in-flight)** The same coverage extends to feature-gated TEST
   modules, not just binaries — `.2`'s `--all-targets` sweep covers them and
   `.5` is the worked instance; and every maintained GATE still RUNS (`.3`
   restored `ebnf_frontend_dual_run_gate`; `.4` made its failures legible). ✅
4. **(Learned 2026-08-11, `.6`)** Every path `cargo metadata` reports as a
   target source is TRACKED IN GIT. Criterion 1 says "every *tracked* bin
   compiles"; that is silent about a bin git has never heard of, and `.2`'s
   census is derived from the working tree, so it reports 19 binaries here
   and 18 on a fresh clone — green both times. ⛔ **Not yet met.**

## Tree status

⚠️ **Re-opened 2026-08-11 by `.6`** — routed in by `SV-CORPUS-GRAD.12c.1`: a hand-written binary
has never been tracked, and `.2`'s `cargo metadata` census cannot see it because that census
reports the WORKING TREE, not git. The five original leaves stand; the tree is no longer closed.

All five original leaves are `done` (`.1` binary repair, `.2` the standing gate,
`.3` the RED gate's design call, `.4` the dead-diagnostics fix, `.5` the
rotted integration test). The tree found and closed **three** independently
rotted surfaces in one blind spot and left a maintained gate that makes the
whole class un-rottable going forward. The gate is intentionally NOT in the
pre-commit path (it is heavy); re-enabling it as an auto CI job is the E4
"no matter what" backstop for this doctrine, tracked as a future ops slice
alongside the other CI re-enablement work.
