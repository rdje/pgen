# GENERATED-LINT-CORRECTNESS: the generated-parser clippy stage can never be made strict, so a real correctness lint there would be invisible forever

## Metadata

- Tree ID: `GENERATED-LINT-CORRECTNESS`
- Status: `active` (opened 2026-07-27, session #213, by **direct director order** —
  *"take the right decision, you have information to take the proper signoff, sota
  decision"*, on being shown the 291-error generated-clippy stage)
- Family / slice-id prefix: `PGEN-GENERATED-LINT-CORRECTNESS-<NNNN>`
- Created: `2026-07-27`
- Owner: repo-local workflow
- Opened by: `LANG-CAPABILITY-AUDIT.10.4`, whose commit-workflow clippy run surfaced it.
  Deliberately NOT absorbed into that leaf — it is a separate defect class with a
  different owner (codegen emission shape, not the builtin allowlist).

## ⭐ THE ADJUDICATION (measured first, decided second)

⛔ **The 291 errors are NOT parser defects, and the generated output must NOT be
"fixed" to satisfy the lint.** Both classes were root-caused to their codegen sites:

| lint (count) | emitted in | codegen site | why it fires |
|---|---|---|---|
| `clippy::eq_op` — *"equal expressions as operands to `==`"* (158) | `generated/systemverilog_parser.rs` | `ast_based_generator.rs:4687` — `} else if #branch_policy_mode == "priority_first" {` | `#branch_policy_mode` is the grammar's configured branch policy, interpolated as a **string literal at codegen time**. When the grammar's policy *is* `priority_first`, the emission is literally `"priority_first" == "priority_first"` — a tautology. |
| `clippy::eq_op` (132) | `generated/rtl_frontend_parser.rs` | same | same |
| `clippy::overly_complex_bool_expr` — *"this boolean expression contains a logic bug"* (1) | `generated/systemverilog_preprocessor_parser.rs:41040` | `ast_based_generator.rs:7016` — `if skip_leading_whitespace && #allow_layout_skip_for_regexes {` | `#allow_layout_skip_for_regexes` is a **compile-time bool**, `false` for the whitespace-sensitive preprocessor grammar, so the guard reads `skip_leading_whitespace && false` and the block is **intentionally** dead for that grammar. |

⇒ in both cases the "tautology" / "dead branch" **IS the intended specialization**: codegen
resolves a per-grammar configuration constant and emits the already-decided form.
`clippy` is correct that the *expression* is degenerate and wrong that it is a bug.

⚠️ **The real gap is not the 291 — it is what the 291 make impossible.** `eq_op` and
`overly_complex_bool_expr` are **correctness-category** lints (`deny` by default). While
they fire on every build, the generated stage can never be run strictly
(`PGEN_CLIPPY_GENERATED_STRICT=1`), which means **a genuine correctness lint appearing in
a generated parser would be permanently invisible** — buried in 84k warnings and 291
pre-existing errors that nobody can triage. That is a live blind spot over the largest
shipped artifacts in the repo (`generated/systemverilog_parser.rs` is 150 MB).

This is the same shape as `LANG-CAPABILITY-AUDIT.10.1`'s finding about the linter
allowlist: **a check that cannot be run strictly is a check that reports nothing.**

## The decision (signoff-grade, director-delegated)

**Emit the folded form, do not silence the lint.** Ranked, with the reasoning:

1. ⭐ **PREFERRED — fold at codegen time.** The condition is already decided when the
   parser is generated, so emit the decided branch instead of the comparison: drop the
   `else if` chain arm entirely when `#branch_policy_mode` is known, and omit the
   guarded block entirely when `#allow_layout_skip_for_regexes` is `false`. This is
   strictly better on every axis — the lint stops firing **because the degenerate code
   is gone**, the generated artifacts shrink (relevant to the ⭐ SPEED north-star: the
   SV parser is 150 MB and these arms are in hot branch-selection code), and rustc was
   folding them anyway so runtime behaviour is unchanged.
2. **ACCEPTABLE FALLBACK — a scoped `#[allow(...)]` on the emitted item**, if (1) turns
   out to entangle the branch-policy emission more than it is worth. Scoped to the
   specific lint on the specific item, never a crate-level blanket.
3. ⛔ **REJECTED — leave it.** That is the status quo, and it is what makes the strict
   stage unreachable. It also silently trains every future reader to ignore the stage.
4. ⛔ **REJECTED — silence the whole generated stage / lower the lints to `warn`
   crate-wide.** That converts a measurable gap into an unmeasurable one and would hide
   a future genuine correctness lint, which is the entire problem being solved.

**Acceptance for the tree:** `PGEN_CLIPPY_GENERATED_STRICT=1` passes on all 11 generated
parsers, with byte-identity **deliberately NOT expected** (the generated output changes
by construction — that is the point), so the proof obligation is instead: every parser
still compiles, the full gate battery stays green, and parse verdicts are unchanged on
the tracked corpora.

## Leaves

### `.1` — fold the two known emission sites and re-measure (`todo`)

- **Status: `todo`** — frontier. Code change in `ast_based_generator.rs` (sites `:4687`
  and `:7016`), then regenerate all 11 parsers.
- Verify: `PGEN_CLIPPY_GENERATED_STRICT=1` generated-clippy error count **291 → 0**;
  artifact sizes recorded before → after; the tracked gate battery green; parse verdicts
  unchanged (the emitted arms were tautological, so any verdict change is a real
  regression and a hard stop).

### `.2` — sweep for other statically-degenerate emissions (`todo`)

- **Status: `todo`**, blocked on `.1`. `.1` fixes the two classes the clippy run happened
  to surface on the grammars that were built. Other grammars / other configuration
  constants may emit the same shape without having been caught yet.
- ⚠️ Do the sweep **at the codegen sites**, not by reading generated output: the pattern
  is "a `#interpolated` constant used in a runtime conditional", and it is enumerable in
  the generator source. Reading 150 MB of emitted Rust is the wrong instrument.

### `.3` — promote the generated-clippy correctness subset to a gate (`todo`)

- **Status: `todo`**, blocked on `.1` + `.2`. Once the count is 0, make it **stay** 0:
  run the generated stage strictly for the **correctness category only** (not all of
  clippy — the 84k style warnings on generated code are genuinely not worth chasing and
  gating on them would be noise, not signal).
- Wire it where the other maintained gates live so it runs in `ci_workflow_local_gate`,
  and record the chosen lint subset in the contract so it cannot silently narrow.

## Evidence

- `rust/target/clippy_gate/logs/clippy_generated_all_targets.log` — the source
  measurement (untracked build output; regenerate with
  `make -C rust SHELL=/opt/homebrew/bin/bash clippy_on_rust_change`, which needs
  `scripts/run_with_memory_guard.sh --budget-mb 16384` per
  `docs/decisions/feedback_host_ram_budget_all_jobs.md` — it peaks ≈12.5 GB and the
  12288 default kills it).
- Error census, measured 2026-07-26 at commit `779dca06`:
  `158` `eq_op` in `systemverilog_parser.rs`, `132` `eq_op` in `rtl_frontend_parser.rs`,
  `1` `overly_complex_bool_expr` in `systemverilog_preprocessor_parser.rs` = **291**.
  Zero errors in any `rust/src/**` file (the strict source stage is green).
- Provenance note: the count is **pre-existing**, not introduced by `.10.4` — that leaf
  left all 11 generated parsers byte-identical, so the same bytes carried the same 291
  errors before it.
