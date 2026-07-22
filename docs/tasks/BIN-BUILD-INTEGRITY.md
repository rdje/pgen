# BIN-BUILD-INTEGRITY — every tracked binary must still COMPILE

- **Status:** `active` (created 2026-07-22, session #196, from a
  `SV-CORPUS-GRAD.8c.3` compile-time discovery)
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

- **Status: `todo`** — the systemic fix, deliberately split from `.1`
  (one commit = one defect).
- **Intent:** add an all-targets build check to the maintained gate
  surface so `required-features` binaries cannot rot silently again —
  `cargo check --workspace --all-targets` per feature combination that any
  tracked `[[bin]]` declares (today: default, `generated_parsers`,
  `ebnf_dual_run`, `bootstrap`, `normal`, and the dual-feature build),
  wired into `ci_workflow_local_gate` (which already audits doc/allowlist
  drift, so binary-build drift is its natural sibling).
- **Design notes to honor:** it must be a CHECK (not a full build) to stay
  cheap; it must be honest about feature combinations rather than testing
  only the union (a bin excluded by `required-features` in the union build
  is exactly the case that rotted); and the memory-guard directive applies
  (a multi-combination check is a heavy job).
- **Open question for the frontier:** whether the same hole exists for
  `#[cfg(feature = ...)]` TEST modules (a test that never compiles under
  any run configuration is the same silent-loss class).

### `.3` — `ebnf_frontend_dual_run_gate` has been RED since 2026-07-15 (second instance of the same silence)

- **Status: `todo`** — discovered while verifying `.1` (session #196);
  tool-pinned, not yet fixed (one commit = one defect, and this one needs
  a design call the fix for `.1` must not smuggle in).
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

## Acceptance Criteria (tree)

1. Every tracked `[[bin]]` compiles under the feature combination it
   declares (`.1` proves the current census; `.2` makes it standing).
2. The proof is mechanical and part of a maintained gate, not a manual
   `cargo check` someone happens to run (`.2`).
